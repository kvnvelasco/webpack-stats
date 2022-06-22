/*
 * Copyright [2022] [Kevin Velasco]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use crate::graphs::{ChunkGraph, ChunkImportPathGraph, ChunkLoadGraph, ModuleParentGraph};
use meshed::graph::traversal::{
    traverse_graph, GraphTraversal, Instruction, Mode, Pathing, TraversalLog,
};
use meshed::prelude::*;

use std::collections::{HashMap, HashSet};
use std::fmt::{write, Display, Formatter};

use meshed::graph::node::Node;
use meshed::graph::traversal::Mode::Acyclic;
use meshed::graph::traversal::Pathing::DFS;
use meshed::graph::{Graph, GraphDefinition, Inverted};
use thiserror::Error;
use webpack_stats::chunk::{Chunk, ChunkId, Chunks, Files};
use webpack_stats::entry::Entrypoint;
use webpack_stats::module::{Module, ModuleIdentifier, ModuleName, Modules};
use webpack_stats::SizeBytes;

#[derive(Debug, Error)]
pub enum EntrypointTraversalError {
    #[error("Module {id} does not exist")]
    NoEntrypoint { id: String },
    #[error("Entrypoint {id} contains invalid chunks: {chunks:?}. Expected chunk {expected}")]
    InvalidEntrypointChunks {
        chunks: HashSet<ChunkId>,
        id: String,
        expected: ChunkId,
    },

    #[error("An unexpected error occured when traversing the graph")]
    GraphError,
}

pub fn find_possible_chunk_for(
    node: &Node<ModuleParentGraph>,
    origin_chunk_id: ChunkId,
    origin_chunk: Option<Node<ChunkGraph>>,
    import_paths: &Graph<ChunkImportPathGraph>,
) -> Option<ChunkId> {
    let chunks = node.node_data();

    // Included in current chunk
    if chunks.is_empty() || chunks.contains(&origin_chunk_id) {
        return Some(origin_chunk_id);
    }

    // only has one option really
    if chunks.len() == 1 {
        return chunks.iter().next().cloned();
    }

    let origin_chunk = origin_chunk?;

    // Origin chunk's children
    if let Some(edge) = origin_chunk.find_edge(|edge| chunks.contains(&edge.target.get_id())) {
        return Some(edge.target.get_id());
    }

    let origin_chunk_node = import_paths
        .query(&origin_chunk.get_id())
        .cloned()
        .expect("Origin chunk did not exist in inverted chunk graph");

    let traversal = traverse_graph(origin_chunk_node)
        .set_pathing(Pathing::DFS)
        .set_mode(Mode::Acyclic)
        .into_iterator(|_, edge| {
            tracing::trace!(
                "Check ancestor {} -> {}",
                edge.origin.label(),
                edge.target.label()
            );
            if chunks.contains(&edge.target.get_id()) {
                Instruction::Halt(Some(edge.target.get_id()))
            } else {
                Instruction::Continue(None)
            }
        });

    traversal.flatten().next()
}

fn annotate_with_chunk<T: GraphDefinition>(
    node: &Node<T>,
    chunk: Option<ChunkId>,
    fallback: ChunkId,
) {
    #[derive(Clone)]
    struct Defaulted;

    if let Some(chunk) = chunk {
        tracing::trace!("Annotate {}", &chunk);
        if let Some(existing_annotation) = node.get_annotation::<ChunkId>() {
            if existing_annotation != chunk && node.get_annotation::<Defaulted>().is_none() {
                tracing::warn!("Subsequent traversal of {} resulted in inconsistent chunk assignment {} -> {}. Module belongs to multiple chunks in traversal", node.label(), &existing_annotation, &chunk);
            }
        }
        node.annotate(chunk);
    } else if node.get_annotation::<ChunkId>().is_some() {
    } else {
        node.annotate(Defaulted);
        node.annotate(fallback);
    }
}

pub fn traverse_entrypoint(
    entrypoint_id: ModuleIdentifier,
    initial_chunk_id: ChunkId,
    module_graph: &Inverted<ModuleParentGraph>,
    truncated_chunk_graph: &Graph<ChunkGraph>,
    import_paths: &Graph<ChunkImportPathGraph>,
) -> Result<GraphTraversal<ModuleParentGraph>, EntrypointTraversalError> {
    let entrypoint = module_graph.inner().query(&entrypoint_id).ok_or(
        EntrypointTraversalError::NoEntrypoint {
            id: entrypoint_id.to_string(),
        },
    )?;

    {
        let module_chunks = entrypoint.node_data();
        if module_chunks.is_empty() {
            entrypoint.annotate(initial_chunk_id);
        } else if module_chunks.len() == 1 {
            entrypoint.annotate(module_chunks.iter().cloned().next().unwrap());
        } else if !module_chunks.contains(&initial_chunk_id) {
            return Err(EntrypointTraversalError::InvalidEntrypointChunks {
                chunks: module_chunks.clone(),
                id: entrypoint_id.to_string(),
                expected: initial_chunk_id.clone(),
            });
        } else {
            entrypoint.annotate(initial_chunk_id);
        }
    }

    let traversal = traverse_graph(entrypoint.clone())
        .set_pathing(Pathing::DFS)
        .set_mode(Mode::Acyclic)
        .execute(|_depth, edge| {
            tracing::trace!(
                "Evaluate {} -> {}",
                edge.origin.label(),
                edge.target.label()
            );
            let origin_chunk = edge
                .origin
                .get_annotation::<ChunkId>()
                .expect("Traversal did not have a source chunk");

            let target_node = &edge.target;

            // find all the outgoing edges in the chunk graph (subtraversal)
            let origin_chunk_node = truncated_chunk_graph.query(&origin_chunk).cloned();

            let chunk = find_possible_chunk_for(
                target_node,
                origin_chunk.clone(),
                origin_chunk_node,
                import_paths,
            );
            annotate_with_chunk(&edge.target, chunk, origin_chunk);
            Instruction::Continue(())
        });

    Ok(traversal)
}

pub fn traverse_entry_chunk<M, C, Mv, Cv, E>(
    modules: M,
    chunks: C,
    entrypoint: &E,
) -> Result<Inverted<ModuleParentGraph>, EntrypointTraversalError>
where
    M: Modules<Mv>,
    Mv: Module,
    C: Chunks<Cv>,
    Cv: Chunk,
    E: Entrypoint,
{
    let chunk_ids = entrypoint.chunks();
    let mut traversal: Option<GraphTraversal<ModuleParentGraph>> = None;
    let chunk_graph = ChunkGraph::build_graph(&chunks);
    let valid_import_graph = ChunkImportPathGraph::build_graph(&chunks);

    let module_graph = ModuleParentGraph::build_graph(&modules).invert();

    for entrypoint_id in chunk_ids.iter().cloned() {
        let chunk = chunk_graph
            .query(&entrypoint_id)
            .cloned()
            .ok_or(EntrypointTraversalError::GraphError)?;

        let chunk_traversal = {
            traverse_graph(chunk.clone())
                .set_mode(Acyclic)
                .execute(|_depth, _edge| Instruction::Continue(()))
        };

        let truncated_chunk_graph = chunk_traversal.project_into_graph(&chunk_graph);
        let import_paths = chunk_traversal.prune_graph(&valid_import_graph);

        let entrypoints = chunk.node_data();

        for entrypoint in entrypoints {
            let traversal_log = traverse_entrypoint(
                entrypoint.clone(),
                entrypoint_id,
                &module_graph,
                &truncated_chunk_graph,
                &import_paths,
            );

            match traversal_log {
                Ok(traversal_log) => {
                    if let Some(old_traversal) = traversal.take() {
                        let next = old_traversal.merge_with(traversal_log);
                        traversal = Some(next);
                    } else {
                        traversal = Some(traversal_log)
                    }
                }
                Err(err) => {
                    tracing::warn!("{}. Skipping", err);
                }
            }
        }
    }

    let traversal = traversal.unwrap();
    Ok(module_graph.map_project(traversal))
}

pub struct Entrypoints<'a> {
    entries: HashMap<&'a str, &'a [ChunkId]>,
}

impl<'a> Display for Entrypoints<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (name, chunks) in self.entries.iter() {
            writeln!(f, "{}:", name)?;
            writeln!(f, "  Chunks:")?;
            for chunk in chunks.iter() {
                writeln!(f, "    {}", chunk)?;
            }
        }
        Ok(())
    }
}

pub fn display_entrypoints<'a, E>(entrypoints: &'a [&'a E]) -> Entrypoints<'a>
where
    E: webpack_stats::entry::Entrypoint,
{
    let mut map = HashMap::new();
    for entry in entrypoints {
        map.insert(entry.name(), entry.chunks());
    }

    Entrypoints { entries: map }
}

pub fn paths_to_chunk<E, C, Cv, M, Mv>(
    entrypoint: &E,
    target_chunk: ChunkId,
    chunks: &C,
    modules: &M,
) -> Inverted<ModuleParentGraph>
where
    M: Modules<Mv>,
    Mv: Module,
    C: Chunks<Cv>,
    Cv: Chunk,
    E: Entrypoint,
{
    let chunk_graph = ChunkGraph::build_graph(&chunks);
    let import_chunk_graph = ChunkImportPathGraph::build_graph(&chunks);
    let module_graph = ModuleParentGraph::build_graph(&modules).invert();
    // traverse every chunk entrypoint until we hit  the target chunk. Store the paths.
    let mut paths = vec![] as Vec<Vec<(ModuleIdentifier, ModuleIdentifier)>>;
    for root_chunk in entrypoint.chunks() {
        if root_chunk == &target_chunk {
            continue;
        }
        let chunk_node = chunk_graph.query(root_chunk).unwrap().clone();
        // each module
        for module in chunk_node.node_data() {
            let module_node = module_graph.inner().query(module).unwrap().clone();
            module_node.annotate(chunk_node.get_id());

            let traversal = traverse_graph(module_node)
                .set_mode(Acyclic)
                .set_pathing(DFS);

            traversal.execute(|meta, edge| {
                let origin_chunk = edge
                    .origin
                    .get_annotation::<ChunkId>()
                    .expect("Did not have an origin chunk");
                let origin_chunk_node = chunk_graph.query(&&origin_chunk).cloned();
                let mut path = meta
                    .get_annotation::<Vec<(ModuleIdentifier, ModuleIdentifier)>>()
                    .unwrap_or_default();
                path.push((edge.origin.get_id(), edge.target.get_id()));

                let node_chunk = find_possible_chunk_for(
                    &edge.target,
                    origin_chunk.clone(),
                    origin_chunk_node,
                    &import_chunk_graph,
                );

                annotate_with_chunk(&edge.target, node_chunk.clone(), origin_chunk);

                // check if we've arrived at the target chunk
                if let Some(chunk) = node_chunk {
                    if chunk == target_chunk {
                        paths.push(path);
                        return Instruction::Backtrack(());
                    }
                }
                edge.target.annotate(path);
                Instruction::Continue(())
            });
        }
    }

    let mut log = TraversalLog::default();
    for path in paths {
        let next_log = TraversalLog::from(path);
        log = log.merge_with(next_log);
    }

    module_graph.map_project(log)
    // merge into a single traversal
}

pub struct EntrypointDescription<'a> {
    name: &'a str,
    initial_load_size: SizeBytes,
    roots: Vec<Node<ChunkLoadGraph>>,
}

impl<'a> Display for EntrypointDescription<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", &self.name)?;
        writeln!(
            f,
            "Initial size (uncompressed): {}",
            &self.initial_load_size
        )?;
        writeln!(f, "Chunk Imports (* denotes asynchronous chunk):")?;
        for root in self.roots.iter() {
            let traversal = traverse_graph(root.clone())
                .set_pathing(Pathing::DFS)
                .set_mode(Mode::Acyclic)
                .into_iterator(|meta, edge| Instruction::Continue((meta, edge)));

            #[derive(Copy, Clone)]
            struct Async;

            write!(f, "├── {} ({}) [", root.get_id(), root.node_data().1)?;

            for file in root.node_data().3 .0.iter() {
                write!(f, "{} ", file)?;
            }
            writeln!(f, "]")?;

            for (meta, edge) in traversal {
                let depth = meta.depth();
                let node = edge.target;

                let indent = (depth * 4) + 4;

                if !node.node_data().2 .0 {
                    meta.annotate(Async);
                }

                if meta.get_annotation::<Async>().is_some() {
                    write!(f, "{:>indent$}", "├*- ")?;
                } else {
                    write!(f, "{:>indent$}", "├── ",)?;
                }

                write!(f, "{} ({}) [", node.label(), node.node_data().1)?;

                for file in node.node_data().3 .0.iter() {
                    write!(f, "{},", file)?;
                }
                writeln!(f, "]")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum EntrypointDescriptionError {}
pub fn describe_entrypoints<'a, C, Cv>(
    chunks: C,
    entrypoint_name: &'a str,
    entrypoints: Entrypoints,
) -> Result<EntrypointDescription<'a>, EntrypointDescriptionError>
where
    C: Chunks<Cv>,
    Cv: Chunk,
{
    let graph = ChunkLoadGraph::build_graph(&chunks);

    let entrypoint = *entrypoints.entries.get(entrypoint_name).unwrap();
    let mut output_traversal = None as Option<GraphTraversal<ChunkLoadGraph>>;
    let mut root_nodes = vec![];
    for chunk in entrypoint.iter() {
        let chunk_node = graph.query(chunk);
        let chunk_node = match chunk_node {
            None => {
                continue;
            }
            Some(node) => node.clone(),
        };

        let truncated = traverse_graph(chunk_node.clone())
            .set_pathing(Pathing::DFS)
            .set_mode(Mode::Acyclic)
            .execute(|_, edge| {
                let initial = &edge.target.node_data().2;
                if !initial.0 {
                    Instruction::Skip(())
                } else {
                    Instruction::Continue(())
                }
            });

        let unique_paths = traverse_graph(chunk_node.clone())
            .set_pathing(DFS)
            .set_mode(Mode::Acyclic)
            .execute(|meta, edge| {
                if let Some(mut seen_in_this_path) = meta.get_annotation::<HashSet<ChunkId>>() {
                    if seen_in_this_path.contains(&edge.target.get_id()) {
                        return Instruction::Skip(());
                    } else {
                        seen_in_this_path.insert(edge.target.get_id());
                    }
                    meta.annotate(seen_in_this_path);
                } else {
                    meta.annotate::<HashSet<ChunkId>>(HashSet::from([edge.origin.get_id()]));
                }
                Instruction::Continue(())
            });

        let projection = unique_paths.project_into_graph(&graph);

        let root_node = projection.query(&chunk_node.get_id()).expect("");
        root_nodes.push(root_node.clone());
        if let Some(tra) = output_traversal.take() {
            output_traversal = Some(tra.merge_with(truncated))
        } else {
            output_traversal = Some(truncated)
        }
    }

    let output_traversal = output_traversal.unwrap();
    let output_graph = output_traversal.project_into_graph(&graph);

    let initial_size = output_graph
        .all_nodes()
        .fold(SizeBytes::default(), |acc, n| acc + n.node_data().1);

    Ok(EntrypointDescription {
        name: entrypoint_name,
        roots: root_nodes,
        initial_load_size: initial_size,
    })
}

#[derive(Debug)]
pub struct ChunkDescription {
    id: ChunkId,
    size: SizeBytes,
    files: Files,
    modules: Vec<ModuleName>,
}

impl Display for ChunkDescription {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk: {}", &self.id)?;
        writeln!(f, "size: {}", &self.size)?;
        writeln!(f, "Files:")?;
        for file in self.files.0.iter() {
            writeln!(f, "  {}", &file)?;
        }

        writeln!(f, "Modules:")?;
        for file in self.modules.iter() {
            writeln!(f, "  {}", &file)?;
        }

        Ok(())
    }
}

pub fn describe_chunk<C: Chunks<Cv>, Cv: Chunk, M: Modules<Mv>, Mv: Module>(
    chunk_id: ChunkId,
    chunks: &C,
    modules: &M,
) -> Option<ChunkDescription> {
    let node = chunks.query(&chunk_id)?;

    let index = modules.create_index();
    let modules: Vec<ModuleIdentifier> = node.extract_data();
    let names = modules.iter().filter_map(|id| {
        let module = index.query(id)?;
        Some(module.label())
    });
    Some(ChunkDescription {
        id: chunk_id,
        size: node.extract_data(),
        files: node.extract_data(),
        modules: names.collect(),
    })
}
