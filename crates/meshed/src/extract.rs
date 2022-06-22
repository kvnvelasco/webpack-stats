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

use crate::identify::{Identifiable, Identity};

use std::collections::HashMap;
use std::fmt::Display;
use std::marker::PhantomData;
use std::rc::Rc;

/// Be able to query some type given an identifier. It must return a type that
/// can be queried for more values like itself.
///
/// Query a datastore for a type that can be used to find more like it
pub trait Query<I, V>
where
    I: Identity,
    V: Identifiable<I>,
{
    fn query(&self, identifier: &I) -> Option<&V>;

    fn all(&self) -> Vec<&V>;

    fn create_index(&self) -> HashMap<I, Link<I, V>> {
        let mut map: HashMap<_, _> = Default::default();
        let values = self.all();
        for value in values {
            map.insert(value.get_id(), Link::Value(value));
        }
        map
    }
}

impl<'a, I, V, T> Query<I, V> for &'a T
where
    T: Query<I, V>,
    I: Identity,
    V: Identifiable<I>,
{
    fn query(&self, identifier: &I) -> Option<&V> {
        T::query(self, identifier)
    }
    fn all(&self) -> Vec<&V> {
        T::all(self)
    }
}

pub struct Edge<I: Identity, Meta> {
    pub source: I,
    pub sink: I,
    pub order: usize,
    pub meta: Rc<Meta>,
}

impl<I: Identity, M> Clone for Edge<I, M> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            sink: self.sink.clone(),
            order: self.order,
            meta: Rc::clone(&self.meta),
        }
    }
}

impl<I, M> Edge<I, M>
where
    I: Identity,
{
    pub fn new(source: I, sink: I, order: usize, meta: M) -> Self {
        Self {
            source,
            sink,
            order,
            meta: Rc::new(meta),
        }
    }
}

/// Be able to derive a set of identifiers from a given item. Semantically this is
/// Get "children" but not all graph operations are parent-child.
///
/// Given a type, find the next set of identities I that I need to traverse
/// Metadata may also be optionally returned via the E value
pub trait Edges<I, D>
where
    I: Identity,
{
    fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<I, D>>;
    fn edges(&self) -> EdgeIterator<'_, I, D, Self>
    where
        Self: Sized,
    {
        EdgeIterator {
            last_edge_index: None,
            node_meta: Default::default(),
            edges: &self,
        }
    }
}

pub trait ExtractData<D: 'static> {
    fn extract_data(&self) -> D;
}

impl<T> ExtractData<()> for T {
    fn extract_data(&self) {}
}

macro_rules! extract_data_impl {
    ($(($($tup:tt), +)) +) => {
        $(
            impl <T, $($tup),+> ExtractData<($($tup),+)> for T
            where
                $(T: ExtractData<$tup>), *,
                $($tup: 'static), *

            {
                fn extract_data(&self) -> ($($tup),+) {
                    ( $(<Self as ExtractData<$tup>>::extract_data(&self)), *  )
                }
            }
        )+
    }
}

extract_data_impl! {
    (D1, D2)
    (D1, D2, D3)
    (D1, D2, D3, D4)
    (D1, D2, D3, D4, D5)
    (D1, D2, D3, D4, D5, D6)
    (D1, D2, D3, D4, D5, D6, D7)
    (D1, D2, D3, D4, D5, D6, D7, D8)
    (D1, D2, D3, D4, D5, D6, D7, D8, D9)
    (D1, D2, D3, D4, D5, D6, D7, D8, D9, D10)
}

/// Label for a node. The label type must be very cheap to clone and move around.
/// for strings, Arc<str> or Rc<str> is preferred when implementing this type.
///
/// ```rust
///# use std::fmt::Formatter;
/// use meshed::prelude::*;
/// use std::rc::Rc;
///
/// #[derive(Clone)]
/// struct Identifier(Rc<str>);
///
/// impl std::fmt::Display for Identifier {
///   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         todo!()
///   }
/// }
///
/// struct Data {
///     id: Identifier
/// }
///
/// impl Label for Data {
///     type Label = Identifier;
///     fn label(&self) -> Self::Label {
///        self.id.clone()
///     }
/// }
///
/// ```
pub trait Label {
    type Label: Display + Clone;
    fn label(&self) -> Self::Label;
}

pub struct EdgeIterator<'a, I: Identity, E, ED>
where
    ED: Edges<I, E>,
{
    last_edge_index: Option<usize>,
    node_meta: PhantomData<(I, E)>,
    edges: &'a ED,
}

impl<'a, I, E, Ed> Iterator for EdgeIterator<'a, I, E, Ed>
where
    I: Identity,
    Ed: Edges<I, E>,
{
    type Item = Edge<I, E>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.edges.next_edge(self.last_edge_index.take())?;
        self.last_edge_index = Some(next.order);
        Some(next)
    }
}

pub enum Link<'a, I, V> {
    Link(I),
    Value(&'a V),
}

impl<'a, I, V> Query<I, V> for HashMap<I, Link<'a, I, V>>
where
    I: Identity,
    V: Identifiable<I>,
{
    fn query(&self, identifier: &I) -> Option<&V> {
        let mut current = self.get(identifier)?;

        loop {
            match current {
                Link::Link(follow) => {
                    current = self.get(follow).expect("Unable to follow link in index");
                }
                Link::Value(output) => return Some(output),
            }
        }
    }

    fn all(&self) -> Vec<&V> {
        self.values()
            .filter_map(|value| match value {
                Link::Link(_) => None,
                Link::Value(v) => Some(*v),
            })
            .collect()
    }
}
