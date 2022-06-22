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

use crate::graph::edge::Edge;

use std::cell::Cell;

use crate::graph::{Graph, GraphDefinition};

use crate::anymap::AnymapCell;
use std::collections::{HashSet, VecDeque};

use std::rc::Rc;

use crate::graph::node::Node;
use crate::identify::{Identifiable, Identity};

pub type GraphTraversal<T: GraphDefinition> = TraversalLog<T::Id>;

#[derive(Debug)]
/// A small datastructure showing all the nodes touched in a traversal
pub struct TraversalLog<I: Identity> {
    pub nodes: HashSet<I>,
    // Origin, target, meta
    pub edges: HashSet<(I, I)>,
}

impl<I> Default for TraversalLog<I>
where
    I: Identity,
{
    fn default() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }
}

impl<I: Identity> From<Vec<(I, I)>> for TraversalLog<I> {
    fn from(v: Vec<(I, I)>) -> Self {
        let mut log = Self {
            nodes: Default::default(),
            edges: Default::default(),
        };

        for item in v {
            log.edges.insert(item.clone());
            log.nodes.insert(item.0);
            log.nodes.insert(item.1);
        }

        log
    }
}

impl<I> TraversalLog<I>
where
    I: Identity,
{
    /// Take a traversal graph with identity type I and materialize it into a graph preserving
    /// node metadata and annotations.
    ///
    /// This guarantees that edge direction, size, and order are preserved between the traversal
    /// and the output graph
    ///
    /// This function guarantees that
    /// 1. size(traversal) == size(output_graph)
    /// 2. order(traversal) == order(input_graph)
    ///
    /// ```rust
    /// # type TestGraph = meshed::graph::Graph<SimpleGraphDefinition>;
    /// # use std::collections::{HashMap, HashSet};
    /// # use meshed::graph::SimpleGraphDefinition;
    /// # use meshed::prelude::*;
    /// use meshed::graph::traversal::{traverse_graph, Instruction, Mode, Pathing};
    /// # let mut graph: TestGraph = Default::default();
    /// # graph.insert_node(0);
    /// # graph.insert_node(1);
    /// # graph.insert_node(2);
    /// # graph.insert_node(3);
    /// # graph.insert_node(4);
    /// # // Guarantee that all edges are in acending order.
    /// # // this will never have cycles if the left number is always larger
    /// # // than the right
    /// # graph.insert_edge(0, 1);
    /// # graph.insert_edge(0, 1);
    /// # graph.insert_edge(1, 2);
    /// # graph.insert_edge(2, 3);
    /// # graph.insert_edge(3, 4);
    ///
    ///
    /// // 0 -> 1, 0 -> 2, 1 -> 2, 2, -> 3, 3 -> 4;
    /// # let input_node = graph.query(&0).unwrap().clone();
    /// let traversal_log = traverse_graph(input_node).set_pathing(Pathing::BFS).execute(|meta, _edge| {
    ///     if meta.depth() > 1 {
    ///        return Instruction::Backtrack(());
    ///     } else {
    ///       return Instruction::Continue(());
    ///     }       
    /// });
    ///
    /// let traversal_graph = traversal_log.project_into_graph(&graph);
    ///
    /// assert_eq!(traversal_graph.all_nodes().map(|node| node.get_id()).collect::<HashSet<_>>(), HashSet::from([0, 2, 1]));
    /// # let one_node = traversal_graph.query(&0).unwrap();
    /// # let edges: Vec<_> = one_node.get_edges().iter().map(|edge| (edge.origin.get_id(), edge.target.get_id())).collect();
    /// # assert_eq!(edges, vec![(0, 1), (0, 1)])
    ///
    /// ```
    pub fn project_into_graph<G>(&self, graph: &Graph<G>) -> Graph<G>
    where
        G: GraphDefinition<Id = I>,
    {
        let mut next_graph: Graph<G> = Default::default();

        for (key, value) in graph.nodes.iter() {
            if !self.nodes.contains(key) {
                continue;
            }
            let next_origin = next_graph
                .nodes
                .entry(key.clone())
                .or_insert_with(|| value.new_derived_with_annotations())
                .clone();

            // figure out which edges we can keep
            value.with_edges_iter(|edges| {
                for edge in edges {
                    if !self
                        .edges
                        .contains(&(edge.origin.get_id(), edge.target.get_id()))
                    {
                        continue;
                    }

                    let target_node = next_graph
                        .nodes
                        .entry(edge.target.get_id())
                        .or_insert_with(|| edge.target.new_derived_with_annotations())
                        .clone();

                    next_origin.insert_edge(target_node, edge.meta.clone());
                }
            });
        }

        next_graph
    }

    /// This is distinct from a truncation in that we don't care too much about
    /// edge direction. We only care about the set of nodes visited being identical.
    ///
    /// This function guarantees that
    /// 1. size(traversal) >= size(output_graph)
    /// 2. order(traversal) == order(output_graph)
    pub fn prune_graph<G>(&self, graph: &Graph<G>) -> Graph<G>
    where
        G: GraphDefinition<Id = I>,
    {
        let nodes = graph.nodes.iter();

        let mut next_graph = Graph::default();

        for (identity, node) in nodes {
            if self.nodes.contains(identity) {
                // we need to insert this node into the graph, but we prune all of it's edges to only contain
                // other nodes in the graph.
                let entry = next_graph
                    .nodes
                    .entry(identity.clone())
                    .or_insert_with(|| node.new_derived_with_annotations());

                node.with_edges_iter(|edges| {
                    for edge in edges {
                        if self.nodes.contains(&edge.target.get_id()) {
                            entry.insert_edge(
                                // we want to make sure we don't include any transitive edges
                                edge.target.new_derived_with_annotations(),
                                edge.meta.clone(),
                            )
                        }
                    }
                });
            }
        }

        next_graph
    }
    // merge two traversals together. Retain own I
    pub fn merge_with(mut self, other: Self) -> Self {
        self.edges.extend(other.edges.into_iter());
        self.nodes.extend(other.nodes.into_iter());
        self
    }
}

/// Given a node will traverse all edges while testing each traversal with the provided function.
/// The returned [`Instruction`] determines the behavior of the graph traversal.
///
/// Each invocation of `next` will also return any data outputted from the test function.
///
/// Does not track any "seen" nodes or edges. Will get stuck in infinite loops if there are cycles.
/// See [`AcyclicTraversal`] for a safer traversal algorithm
/// ```
/// # type TestGraph = meshed::graph::Graph<SimpleGraphDefinition>;
/// # use std::collections::{HashMap, HashSet};
/// # use meshed::graph::SimpleGraphDefinition;
/// # use meshed::prelude::*;
/// use meshed::graph::traversal::{traverse_graph, Instruction, Pathing};
/// # let mut graph: TestGraph = Default::default();
/// # graph.insert_node(0);
/// # graph.insert_node(1);
/// # graph.insert_node(2);
/// # graph.insert_node(3);
/// # // Guarantee that all edges are in acending order.
/// # // this will never have cycles if the left number is always larger
/// # // than the right
/// # graph.insert_edge(0, 1);
/// # graph.insert_edge(0, 2);
/// # graph.insert_edge(1, 2);
/// # graph.insert_edge(2, 3);
/// # graph.insert_edge(1, 3);
///
/// # let node = graph.query(&0).unwrap().clone();
///   let log = traverse_graph(node)
///               .set_pathing(Pathing::BFS)
///               .execute(|_depth, _edge| Instruction::Continue(()));
///  
///
///# assert_eq!(log.nodes, HashSet::from([0, 1, 2, 3]));
///# assert_eq!(log.edges, HashSet::from([(0, 1), (0, 2), (1, 2), (1, 3),  (2, 3)]))
///```
pub struct TraverseGraph<T: GraphDefinition, Mode> {
    traversal_log: TraversalLog<T::Id>,
    queue: VecDeque<(TraversalMeta<T::Id>, Edge<T>)>,
    mode: Mode,
}

#[derive(Copy, Clone)]
pub struct New(Mode, Pathing);

#[derive(Copy, Clone)]
pub struct Started(Mode, Pathing);

#[derive(Copy, Clone)]
pub enum Pathing {
    DFS,
    BFS,
}

/// Instructions to the graph traversal algorithms.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Instruction<R> {
    /// Immediately halt all traversal and clear traversal queues
    Halt(R),
    /// Halt exploration of the current node, reverse back up the queue
    /// Do not traverse through it's children
    Backtrack(R),
    /// Skip this edge, do not emit it to the traversal log
    Skip(R),
    /// Continue traversal
    Continue(R),
}

#[derive(Copy, Clone)]
pub enum Mode {
    Acyclic,
    Simple,
}

pub fn traverse_graph<T: GraphDefinition>(start_node: Node<T>) -> TraverseGraph<T, New> {
    let mut queue = VecDeque::new();

    start_node.with_edges_iter(|edge| {
        queue.extend(edge.cloned().map(|edge| (TraversalMeta::new(1), edge)))
    });

    let mut nodes = HashSet::new();
    nodes.insert(start_node.get_id());
    TraverseGraph {
        traversal_log: TraversalLog {
            edges: Default::default(),
            nodes,
        },
        queue,
        mode: New(Mode::Simple, Pathing::BFS),
    }
}

impl<T> TraverseGraph<T, New>
where
    T: GraphDefinition,
{
    pub fn set_pathing(mut self, pathing: Pathing) -> Self {
        self.mode.1 = pathing;
        self
    }

    pub fn set_mode(mut self, mode: Mode) -> Self {
        self.mode.0 = mode;
        self
    }

    pub fn execute<F>(self, mut test_fn: F) -> TraversalLog<T::Id>
    where
        F: FnMut(TraversalMeta<T::Id>, Edge<T>) -> Instruction<()>,
    {
        let mut started = self.start();
        loop {
            if started
                .drive_edge(|meta, edge| test_fn(meta, edge))
                .is_none()
            {
                break;
            }
        }

        started.traversal_log
    }

    pub fn start(self) -> TraverseGraph<T, Started> {
        TraverseGraph {
            traversal_log: self.traversal_log,
            queue: self.queue,
            mode: Started(self.mode.0, self.mode.1),
        }
    }

    pub fn into_iterator<F, R>(self, test_fn: F) -> GraphIterator<T, F, R>
    where
        F: FnMut(TraversalMeta<T::Id>, Edge<T>) -> Instruction<R>,
    {
        GraphIterator {
            iterator: self.start(),
            test_fn,
        }
    }
}

impl<T> TraverseGraph<T, Started>
where
    T: GraphDefinition,
{
    pub fn drive_edge<F, R>(&mut self, mut test_fn: F) -> Option<R>
    where
        F: FnMut(TraversalMeta<T::Id>, Edge<T>) -> Instruction<R>,
    {
        let (meta, edge) = match self.mode.1 {
            Pathing::BFS => self.queue.pop_front()?,
            Pathing::DFS => self.queue.pop_back()?,
        };

        match (self.mode, test_fn(meta.clone(), edge.clone())) {
            // Backtrack if acyclic node has been previously traversed
            (Started(Mode::Acyclic, _), Instruction::Continue(result))
                if self.traversal_log.nodes.contains(&edge.target.get_id()) =>
            {
                self.traversal_log.nodes.insert(edge.target.get_id());
                self.traversal_log.nodes.insert(edge.origin.get_id());
                self.traversal_log
                    .edges
                    .insert((edge.origin.get_id(), edge.target.get_id()));

                Some(result)
            }
            (_, Instruction::Halt(result)) => {
                self.queue.clear();
                Some(result)
            }
            (_, Instruction::Skip(result)) => Some(result),
            (_, Instruction::Backtrack(result)) => {
                self.traversal_log.nodes.insert(edge.target.get_id());
                self.traversal_log.nodes.insert(edge.origin.get_id());
                self.traversal_log
                    .edges
                    .insert((edge.origin.get_id(), edge.target.get_id()));

                Some(result)
            }
            (_, Instruction::Continue(result)) => {
                self.traversal_log.nodes.insert(edge.target.get_id());
                self.traversal_log.nodes.insert(edge.origin.get_id());
                self.traversal_log
                    .edges
                    .insert((edge.origin.get_id(), edge.target.get_id()));

                let mut child_meta = meta;
                child_meta.depth += 1;
                edge.target.with_edges_iter(|edges| {
                    edges
                        .cloned()
                        .for_each(|edge| self.queue.push_back((child_meta.clone(), edge)))
                });
                Some(result)
            }
        }
    }

    pub fn drive_node<F, R>(&mut self, mut test_fn: F) -> Option<R>
    where
        F: FnMut(TraversalMeta<T::Id>, Node<T>) -> Instruction<R>,
    {
        self.drive_edge(|meta, edge| test_fn(meta, edge.target))
    }
}

pub struct GraphIterator<T, F, R>
where
    T: GraphDefinition,
    F: FnMut(TraversalMeta<T::Id>, Edge<T>) -> Instruction<R>,
{
    iterator: TraverseGraph<T, Started>,
    test_fn: F,
}

impl<T, F, R> Iterator for GraphIterator<T, F, R>
where
    T: GraphDefinition,
    F: FnMut(TraversalMeta<T::Id>, Edge<T>) -> Instruction<R>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.iterator
            .drive_edge(|meta, edge| (self.test_fn)(meta, edge))
    }
}

#[derive(Clone)]
pub struct TraversalMeta<I> {
    depth: usize,
    path: Rc<Cell<Vec<I>>>,
    any_meta: Rc<AnymapCell>,
}

impl<I> TraversalMeta<I>
where
    I: Identity,
{
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            path: Default::default(),
            any_meta: Rc::new(Default::default()),
        }
    }

    pub fn include_path(&mut self, target: I) {
        let mut path = self.path.take();
        path.push(target);
        self.path.set(path);
    }

    pub fn annotate<A: 'static>(&self, annotation: A) -> Option<A> {
        self.any_meta.insert(annotation)
    }

    pub fn get_annotation<A: Clone + 'static>(&self) -> Option<A> {
        self.any_meta.get()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }
}
