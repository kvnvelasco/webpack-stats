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

use crate::graphs::ModuleParentGraph;
use meshed::graph::node::Node;
use meshed::graph::Inverted;
use std::collections::HashSet;

use meshed::prelude::*;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use std::marker::PhantomData;
use webpack_stats::chunk::ChunkId;
use webpack_stats::import::ImportType;

pub struct GraphSerialization<T, K> {
    graph: T,
    kind: PhantomData<K>,
}

impl<T, K> GraphSerialization<T, K> {
    pub fn new<Kind>(graph: T) -> GraphSerialization<T, Kind> {
        GraphSerialization {
            graph,
            kind: Default::default(),
        }
    }
}

pub struct NodeEdge;

type ModuleEdgeSer = GraphSerialization<Inverted<ModuleParentGraph>, NodeEdge>;

impl Serialize for ModuleEdgeSer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_serializer = serializer.serialize_map(Some(2))?;
        map_serializer.serialize_key("nodes")?;

        struct NodeSerializer(Node<ModuleParentGraph>);

        impl Serialize for NodeSerializer {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_key("id")?;
                map.serialize_value(&self.0.get_id().to_string())?;
                if let Some(value) = self.0.get_annotation::<ChunkId>() {
                    map.serialize_key("chunk")?;
                    map.serialize_value(&value.0)?;
                } else {
                    map.serialize_key("chunk")?;
                    map.serialize_value(&None as &Option<()>)?;
                }

                map.serialize_key("label")?;
                map.serialize_value(&self.0.label().to_string())?;
                map.end()
            }
        }

        let nodes = self
            .graph
            .inner()
            .all_nodes()
            .map(NodeSerializer)
            .collect::<Vec<_>>();

        map_serializer.serialize_value(&nodes)?;

        struct EdgeSerializer(meshed::graph::edge::Edge<ModuleParentGraph>);

        impl Serialize for EdgeSerializer {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                let mut map = serializer.serialize_map(Some(4))?;

                map.serialize_key("source")?;
                map.serialize_value(self.0.origin.get_id().0.as_ref())?;
                map.serialize_key("target")?;
                map.serialize_value(self.0.target.get_id().0.as_ref())?;
                map.serialize_key("async")?;
                map.serialize_value(&matches!(
                    self.0.meta.as_ref().0,
                    ImportType::RequireContext | ImportType::Import | ImportType::ImportDynamic
                ))?;

                map.serialize_key("importer")?;
                map.serialize_value(&self.0.meta.as_ref().1 .0)?;

                map.end()
            }
        }
        let mut seen_set = HashSet::new();
        let edges = self
            .graph
            .inner()
            .all_edges()
            .filter(|edge| {
                if seen_set.contains(&edge.get_id()) {
                    false
                } else {
                    seen_set.insert(edge.get_id());
                    true
                }
            })
            .map(EdgeSerializer)
            .collect::<Vec<_>>();

        map_serializer.serialize_key("edges")?;
        map_serializer.serialize_value(&edges)?;
        map_serializer.end()
    }
}
