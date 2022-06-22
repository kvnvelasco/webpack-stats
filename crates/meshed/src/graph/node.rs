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

use std::cell::Cell;
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;

use crate::anymap::AnymapCell;
use crate::graph::edge::Edge;
use crate::graph::{GraphDefinition, SimpleGraphDefinition};
use crate::identify::Identifiable;
use std::rc::Rc;

pub struct Node<T: GraphDefinition> {
    id: T::Id,
    label: Rc<T::Label>,
    data: Rc<T::NodeData>,
    annotations: Rc<AnymapCell>,
    edges: Rc<Edges<T>>,
    graph_type: PhantomData<T>,
}

type Edges<T> = Cell<Vec<Edge<T>>>;

impl Node<SimpleGraphDefinition> {
    pub fn new_bare(identity: <SimpleGraphDefinition as GraphDefinition>::Id) -> Self {
        Self::new(
            identity,
            identity as <SimpleGraphDefinition as GraphDefinition>::Label,
            (),
        )
    }
}

impl<T: GraphDefinition> Node<T> {
    pub fn new(identity: T::Id, label: T::Label, data: T::NodeData) -> Self {
        Self {
            id: identity,
            annotations: Default::default(),
            label: Rc::new(label),
            data: Rc::new(data),
            edges: Default::default(),
            graph_type: Default::default(),
        }
    }

    pub fn label(&self) -> &T::Label {
        self.label.as_ref()
    }

    pub fn annotate<A: 'static>(&self, value: A) -> Option<A> {
        self.annotations.insert(value)
    }

    pub fn get_annotation<A: Clone + 'static>(&self) -> Option<A> {
        self.annotations.get()
    }

    pub fn node_data(&self) -> &T::NodeData {
        self.data.as_ref()
    }

    pub fn new_derived(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
            edges: Default::default(),
            label: self.label.clone(),
            annotations: Default::default(), // We don't preserve annotations
            id: self.id.clone(),
            graph_type: Default::default(),
        }
    }

    pub fn new_derived_with_annotations(&self) -> Self {
        Self {
            data: Rc::clone(&self.data),
            edges: Default::default(),
            label: self.label.clone(),
            annotations: Rc::clone(&self.annotations), // We don't preserve annotations
            id: self.id.clone(),
            graph_type: Default::default(),
        }
    }
}

impl<T> Node<T>
where
    T: GraphDefinition,
{
    pub fn derive_edge(&self, target: Self, source: &Edge<T>) {
        let mut edges = self.edges.take();
        edges.push(Edge::new(self.clone(), target, source.meta.clone()));
        self.edges.set(edges);
    }
    pub fn insert_edge(&self, target: Self, meta: Rc<T::EdgeMeta>) {
        let mut edges = self.edges.take();
        edges.push(Edge::new(self.clone(), target, meta));
        self.edges.set(edges);
    }

    pub fn get_edge_ref(&self, index: usize) -> Option<Edge<T>> {
        let edges = self.edges.take();
        let edge = edges.iter().nth(index).cloned();
        self.edges.set(edges);
        edge
    }

    pub fn get_edge(&self, id: &T::Id) -> Option<Edge<T>> {
        let edges = self.edges.take();
        let edge = edges.iter().find(|edge| &edge.target.id == id).cloned();
        self.edges.set(edges);
        edge
    }

    pub fn with_edges_iter<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut dyn Iterator<Item = &Edge<T>>) -> R,
    {
        let edges = self.edges.take();
        let mut iter = edges.iter();
        let ret = f(&mut iter);
        self.edges.set(edges);
        ret
    }

    pub fn find_edge<F>(&self, mut find_fn: F) -> Option<Edge<T>>
    where
        F: FnMut(&Edge<T>) -> bool,
    {
        self.with_edges_iter(|edges| {
            for edge in edges {
                if find_fn(edge) {
                    return Some(edge.clone());
                }
            }
            None
        })
    }

    pub fn get_edges(&self) -> Vec<Edge<T>> {
        let mut output = vec![];
        // let mut iterator = edges.iter().map(|n| n.deref());
        self.with_edges_iter(|edges| output.extend(edges.cloned()));

        output
    }
}

impl<T> Clone for Node<T>
where
    T: GraphDefinition,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            data: Rc::clone(&self.data),
            label: Rc::clone(&self.label),
            annotations: Rc::clone(&self.annotations),
            edges: Rc::clone(&self.edges),
            graph_type: Default::default(),
        }
    }
}

impl<T> Debug for Node<T>
where
    T: GraphDefinition,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut edge_list = std::string::String::new();

        self.with_edges_iter(|edge_iter| {
            for edge in edge_iter {
                edge_list += &edge.target.get_id().to_string();
                edge_list += ", "
            }
        });
        let rs = write!(f, "[[ {:?} | [{}] ]]", &self.id, edge_list);
        rs
    }
}

impl<T> Identifiable<T::Id> for Node<T>
where
    T: GraphDefinition,
{
    fn get_id(&self) -> T::Id {
        self.id.clone()
    }
}

impl<T> PartialEq for Node<T>
where
    T: GraphDefinition,
{
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl<T> Eq for Node<T> where T: GraphDefinition {}

impl<T> PartialOrd for Node<T>
where
    T: GraphDefinition,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_id().partial_cmp(&other.get_id())
    }
}

impl<T> Ord for Node<T>
where
    T: GraphDefinition,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_id().cmp(&other.get_id())
    }
}
