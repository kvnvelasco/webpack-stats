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

use crate::graph::node::Node;

use edge::Edge;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;

use dot::{Id, Nodes};
use std::rc::Rc;

use crate::extract::{Edges, ExtractData, Label, Query};

use crate::graph::traversal::GraphTraversal;
use crate::identify::{Identifiable, Identity};

pub mod edge;
pub mod node;

pub mod traversal;

pub type Graph<T> = ConcreteGraph<T>;

pub trait GraphDefinition {
    type Id: Identity;
    type Label: Display + Clone;
    type EdgeMeta;
    type NodeData: 'static;

    fn build_graph<V, Q>(source: &Q) -> ConcreteGraph<Self>
    where
        Q: Query<Self::Id, V>,
        V: Identifiable<Self::Id>
            + Edges<Self::Id, Self::EdgeMeta>
            + ExtractData<Self::NodeData>
            + Label<Label = Self::Label>,
        Self: Sized,
    {
        let source = source.create_index();

        let mut top_level_edges = source.all().into_iter().map(|value| value.get_id());

        let mut output = HashMap::new();
        let mut queue: Vec<Self::Id> = vec![];
        let mut current: Option<Self::Id> = top_level_edges.next();
        let mut seen_set = HashSet::new();
        while let Some(current_identity) = current.take() {
            // get next edges
            if let Some(next) = queue.pop() {
                current = Some(next);
            } else if let Some(id) = top_level_edges.next() {
                current = Some(id);
            } else {
                current = None;
            }

            if seen_set.contains(&current_identity) {
                continue;
            } else {
                seen_set.insert(current_identity.clone());
            }

            let value: &V = source
                .query(&current_identity)
                .expect("Created a value that doesn't exist in the set");
            let current_node = output
                .entry(current_identity.clone() as Self::Id)
                .or_insert_with(|| {
                    Node::new(
                        current_identity.clone(),
                        value.label(),
                        value.extract_data(),
                    )
                })
                .clone();

            let edge_iterator = value.edges();

            for edge in edge_iterator {
                // add the value to the queue
                queue.push(edge.sink.clone());

                let node = output
                    .entry(edge.sink.clone() as Self::Id)
                    .or_insert_with(|| {
                        let data = source.query(&edge.sink);

                        assert!(
                            data.is_some(),
                            "Created node that does not exist {}",
                            edge.sink.clone()
                        );
                        let data = data.unwrap();
                        Node::new(edge.sink, data.label(), data.extract_data())
                    })
                    .clone();

                current_node.insert_edge(node.clone(), edge.meta);
            }
        }
        output.into_values().collect()
    }
}

pub struct SimpleGraphDefinition;
impl GraphDefinition for SimpleGraphDefinition {
    type Id = i32;
    type Label = Self::Id;
    type EdgeMeta = ();
    type NodeData = ();
}

trait AbstractGraph {
    type Definition: GraphDefinition;
}

pub struct ConcreteGraph<T>
where
    T: GraphDefinition,
{
    pub(crate) nodes: HashMap<T::Id, Node<T>>,
}

impl<T> ConcreteGraph<T>
where
    T: GraphDefinition,
{
    pub fn all_nodes(&self) -> impl Iterator<Item = Node<T>> + '_ {
        self.nodes.values().cloned()
    }

    // all of the edges in the entire graph
    pub fn all_edges(&self) -> impl Iterator<Item = Edge<T>> + '_ {
        self.nodes
            .values()
            .flat_map(|node| node.get_edges().into_iter())
    }

    pub fn order(&self) -> usize {
        self.nodes.len()
    }
}

impl<T: GraphDefinition> Default for ConcreteGraph<T> {
    fn default() -> Self {
        Self {
            nodes: Default::default(),
        }
    }
}

impl<T> FromIterator<Node<T>> for ConcreteGraph<T>
where
    T: GraphDefinition,
{
    fn from_iter<I: IntoIterator<Item = Node<T>>>(iter: I) -> Self {
        let mut map = HashMap::default();

        for node in iter {
            map.insert(node.get_id(), node);
        }

        Self { nodes: map }
    }
}

impl<'a, T: GraphDefinition> IntoIterator for &'a ConcreteGraph<T> {
    type Item = Node<T>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let nodes: Vec<_> = self.nodes.values().cloned().collect();
        nodes.into_iter()
    }
}

impl<T> Query<T::Id, Node<T>> for ConcreteGraph<T>
where
    T: GraphDefinition,
{
    fn query(&self, identifier: &T::Id) -> Option<&Node<T>> {
        self.nodes.get(identifier)
    }

    fn all(&self) -> Vec<&Node<T>> {
        self.nodes.values().collect()
    }
}

impl<T> ConcreteGraph<T>
where
    T: GraphDefinition,
{
    // takes the current nodes and makes every child point to it's parent and vv
    pub fn invert(&self) -> Inverted<T> {
        let edges = self.all_nodes(); // a graph always has edges.

        let mut output_graph = Self::default();

        for existing_node in edges {
            let node = output_graph
                .nodes
                .entry(existing_node.get_id())
                .or_insert_with(|| existing_node.new_derived())
                .clone();

            existing_node.with_edges_iter(|edge| {
                for child in edge {
                    let child_node = output_graph
                        .nodes
                        .entry(child.target.get_id())
                        .or_insert_with(|| child.target.new_derived())
                        .clone();

                    child_node.derive_edge(node.clone(), &child);
                }
            });
        }

        Inverted {
            graph: output_graph,
        }
    }
}

pub struct Inverted<T>
where
    T: GraphDefinition,
{
    graph: ConcreteGraph<T>,
}

impl<T> Inverted<T>
where
    T: GraphDefinition,
{
    pub fn inner(&self) -> &ConcreteGraph<T> {
        &self.graph
    }

    pub fn map_project(mut self, traversal: GraphTraversal<T>) -> Self {
        self.graph = traversal.project_into_graph(&self.graph);
        self
    }
}

impl Graph<SimpleGraphDefinition> {
    pub fn insert_node(&mut self, node_id: <SimpleGraphDefinition as GraphDefinition>::Id) {
        self.nodes
            .entry(node_id.clone())
            .or_insert_with(|| Node::<SimpleGraphDefinition>::new_bare(node_id));
    }

    pub fn insert_edge(
        &mut self,
        from: <SimpleGraphDefinition as GraphDefinition>::Id,
        to: <SimpleGraphDefinition as GraphDefinition>::Id,
    ) {
        let node = self
            .nodes
            .entry(from.clone())
            .or_insert_with(|| Node::<SimpleGraphDefinition>::new_bare(from))
            .clone();
        let target = self
            .nodes
            .entry(to.clone())
            .or_insert_with(|| Node::<SimpleGraphDefinition>::new_bare(to))
            .clone();
        node.insert_edge(target, Rc::new(()));
    }
}

#[cfg(test)]
mod test {
    type SimpleGraph = Graph<SimpleGraphDefinition>;

    use crate::graph::node::Node;
    use crate::graph::{Graph, GraphDefinition, SimpleGraphDefinition};
    use std::collections::HashSet;

    use crate::extract::{Edge, Edges, Label, Query};
    use crate::identify::Identifiable;

    type Id = i32;

    struct Datastore {
        store: Vec<Data>,
    }

    struct Data {
        id: Id,
        edges: Vec<Id>,
    }

    impl Label for Data {
        type Label = Id;
        fn label(&self) -> Self::Label {
            self.id
        }
    }

    impl Identifiable<Id> for Data {
        fn get_id(&self) -> Id {
            self.id.clone()
        }
    }

    impl Query<Id, Data> for Datastore {
        fn query(&self, identifier: &Id) -> Option<&Data> {
            self.store.iter().find(|data| data.id == *identifier)
        }

        fn all(&self) -> Vec<&Data> {
            self.store.iter().collect()
        }
    }

    impl<'a> Edges<Id, ()> for Data {
        fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<Id, ()>> {
            let next_idx = previous_edge_index.map(|e| e + 1).unwrap_or_default();
            let edge = self.edges.get(next_idx)?;
            Some(Edge::new(self.get_id(), edge.clone(), next_idx, ()))
        }
    }

    type TestGraph = Graph<SimpleGraph>;
    #[test]
    fn graph_can_be_linked_together() {
        let store = Datastore {
            store: vec![
                Data {
                    id: (1),
                    edges: vec![(2), (3)],
                },
                Data {
                    id: (2),
                    edges: vec![(4)],
                },
                Data {
                    id: (3),
                    edges: vec![(4)],
                },
                Data {
                    id: (4),
                    edges: vec![(1)],
                },
            ],
        };

        let graph = SimpleGraphDefinition::build_graph(&store);

        let node_edge_compare = |node: &Node<SimpleGraphDefinition>| {
            let mut coll = vec![];
            node.with_edges_iter(|edges| {
                coll.extend(edges.cloned().map(|edge| edge.target.get_id()))
            });
            coll
        };

        let one: Node<SimpleGraphDefinition> = graph
            .into_iter()
            .find(|node: &Node<SimpleGraphDefinition>| node.get_id() == (1))
            .unwrap();

        assert_eq!(node_edge_compare(&one), vec![(2), (3)]);
        let three = one.get_edge_ref(1).unwrap().target;

        assert_eq!(node_edge_compare(&three), vec![(4)]);
        let two = one.get_edge_ref(0).unwrap().target;
        assert_eq!(node_edge_compare(&two), vec![(4)]);
        let four = two.get_edge_ref(0).unwrap().target;
        let alt_four = three.get_edge_ref(0).unwrap().target;
        assert_eq!(node_edge_compare(&alt_four), node_edge_compare(&four));

        assert_eq!(node_edge_compare(&four), vec![(1)]);
        assert_eq!(node_edge_compare(&alt_four), vec![(1)]);
    }

    #[test]
    fn graph_can_be_inverted() {
        let store = Datastore {
            store: vec![
                Data {
                    id: (1),
                    edges: vec![(2), (3)],
                },
                Data {
                    id: (2),
                    edges: vec![(4)],
                },
                Data {
                    id: (3),
                    edges: vec![(4)],
                },
                Data {
                    id: (4),
                    edges: vec![(1)],
                },
            ],
        };

        let graph = SimpleGraphDefinition::build_graph(&store);
        let graph = graph.invert();

        let one = graph
            .graph
            .into_iter()
            .find(|node| node.get_id() == (1))
            .unwrap();

        let node_edge_compare = |node: &Node<SimpleGraphDefinition>| {
            let mut coll = vec![];
            node.with_edges_iter(|edges| {
                coll.extend(edges.cloned().map(|edge| edge.target.get_id()))
            });
            coll
        };

        assert_eq!(node_edge_compare(&one), vec![(4)]);

        let four = one.get_edge_ref(0).unwrap().target;
        let three = four.get_edge_ref(1).unwrap().target;
        let two = four.get_edge_ref(0).unwrap().target;

        assert_eq!(node_edge_compare(&three), vec![(1)]);
        assert_eq!(node_edge_compare(&two), vec![(1)]);

        let alt_one = two.get_edge_ref(0).unwrap().target;
        assert_eq!(node_edge_compare(&alt_one), node_edge_compare(&one));

        assert_eq!(
            HashSet::from_iter(node_edge_compare(&four).into_iter()),
            HashSet::from([2, 3])
        );
        assert_eq!(node_edge_compare(&alt_one), vec![(4)]);
    }
}
impl<'a, T: GraphDefinition> dot::GraphWalk<'a, Node<T>, Edge<T>> for ConcreteGraph<T> {
    fn nodes(&'a self) -> Nodes<'a, Node<T>> {
        let nodes = self.all_nodes();
        Nodes::Owned(nodes.collect())
    }

    fn edges(&'a self) -> dot::Edges<'a, Edge<T>> {
        dot::Edges::Owned(self.all_edges().collect())
    }

    fn source(&'a self, edge: &Edge<T>) -> Node<T> {
        edge.origin.clone()
    }

    fn target(&'a self, edge: &Edge<T>) -> Node<T> {
        edge.target.clone()
    }
}

impl<'a, T: GraphDefinition> dot::Labeller<'a, Node<T>, Edge<T>> for ConcreteGraph<T> {
    fn graph_id(&'a self) -> Id<'a> {
        Id::new("webpack_stats").unwrap()
    }

    fn node_id(&'a self, n: &Node<T>) -> Id<'a> {
        Id::new(n.get_id().escaped()).unwrap()
    }
}
