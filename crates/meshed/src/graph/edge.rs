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
use crate::graph::GraphDefinition;
use crate::identify::{Identifiable, Identity};
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

pub struct Edge<T: GraphDefinition> {
    pub origin: Node<T>,
    pub target: Node<T>,
    pub meta: Rc<T::EdgeMeta>,
    pub graph_type: PhantomData<T>,
}

impl<T: GraphDefinition> Edge<T> {
    pub fn new(from: Node<T>, to: Node<T>, meta: Rc<T::EdgeMeta>) -> Self {
        Self {
            origin: from,
            target: to,
            meta,
            graph_type: Default::default(),
        }
    }
}

impl<T: GraphDefinition> Clone for Edge<T> {
    fn clone(&self) -> Self {
        Self {
            origin: self.origin.clone(),
            target: self.target.clone(),
            meta: Rc::clone(&self.meta),
            graph_type: Default::default(),
        }
    }
}

impl<T: GraphDefinition> PartialEq for Edge<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get_id() == other.get_id()
    }
}

impl<T: GraphDefinition> PartialOrd for Edge<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.get_id().partial_cmp(&other.get_id())
    }
}

impl<T: GraphDefinition> Eq for Edge<T> {}

impl<T: GraphDefinition> Ord for Edge<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_id().cmp(&other.get_id())
    }
}

#[derive(Clone, PartialOrd, PartialEq, Eq, Ord, Hash, Debug)]
pub struct EdgeIdentity<I: Identity>(I, I);

impl<I: Identity> Display for EdgeIdentity<I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{} -> {} ]", &self.0, &self.1)
    }
}

impl<I: Identity> Identity for EdgeIdentity<I> {}

// Edges who's metadata is identifiable (unique) are identifiable.
impl<T> Identifiable<EdgeIdentity<T::Id>> for Edge<T>
where
    T: GraphDefinition,
{
    fn get_id(&self) -> EdgeIdentity<T::Id> {
        EdgeIdentity(self.origin.get_id(), self.target.get_id())
    }
}
