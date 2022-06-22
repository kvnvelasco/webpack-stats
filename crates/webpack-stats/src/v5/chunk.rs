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

use crate::common::SizeBytes;
use crate::v5::module::Modules;
use crate::v5::reason::Reasons;
use serde::Deserialize;
use std::borrow::Cow;

use crate::chunk::{
    ChunkChild, ChunkChildren, ChunkInitial, ChunkModules, ChunkParentOrSibling, Files,
};
use crate::common::chunk::{ChunkId, ChunkName};
use crate::common::import::SourceFilePath;
use crate::common::module::{ModuleId, ModuleIdentifier, RelativeModulePath};
use meshed::prelude::*;

#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Chunks<'a> {
    #[serde(borrow)]
    pub chunks: Vec<Chunk<'a>>,
}

impl<'a> crate::common::chunk::Chunks<Chunk<'a>> for Chunks<'a> {}

impl<'a> Query<ChunkId, Chunk<'a>> for Chunks<'a> {
    fn query(&self, identifier: &ChunkId) -> Option<&Chunk<'a>> {
        self.chunks
            .iter()
            .find(|chunk| &chunk.get_id() == identifier)
    }

    fn all(&self) -> Vec<&Chunk<'a>> {
        self.chunks.iter().collect()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Chunk<'a> {
    pub id: ChunkId,
    pub entry: bool,
    pub initial: bool,
    #[serde(borrow)]
    pub modules: Modules<'a>,
    pub files: Vec<SourceFilePath<'a>>,
    #[serde(borrow)]
    pub names: Vec<ChunkName<'a>>,
    #[serde(default)]
    pub origins: Vec<Origin<'a>>,
    pub parents: Vec<ChunkId>,
    pub siblings: Vec<ChunkId>,
    pub children: Vec<ChunkId>,
    pub rendered: bool,
    pub size: SizeBytes,
}

impl<'a> ExtractData<ChunkId> for Chunk<'a> {
    fn extract_data(&self) -> ChunkId {
        self.id
    }
}

impl<'a> ExtractData<ChunkChildren> for Chunk<'a> {
    fn extract_data(&self) -> ChunkChildren {
        ChunkChildren(self.children.clone())
    }
}

impl<'a> ExtractData<SizeBytes> for Chunk<'a> {
    fn extract_data(&self) -> SizeBytes {
        self.size
    }
}

impl<'a> ExtractData<ChunkModules> for Chunk<'a> {
    fn extract_data(&self) -> ChunkModules {
        self.modules
            .modules
            .iter()
            .map(|module| module.identifier.clone())
            .collect()
    }
}

impl<'a> Label for Chunk<'a> {
    type Label = ChunkId;

    fn label(&self) -> Self::Label {
        self.id
    }
}

impl<'a> ExtractData<ChunkInitial> for Chunk<'a> {
    fn extract_data(&self) -> ChunkInitial {
        ChunkInitial(self.initial)
    }
}

impl<'a> ExtractData<Files> for Chunk<'a> {
    fn extract_data(&self) -> Files {
        Files(
            self.files
                .iter()
                .map(|f| f.0.to_string_lossy().to_string())
                .collect(),
        )
    }
}

impl<'a> crate::common::chunk::Chunk for Chunk<'a> {}

impl<'a> Identifiable<ChunkId> for Chunk<'a> {
    fn get_id(&self) -> ChunkId {
        self.id
    }
}

impl<'a> Edges<ChunkId, ChunkParentOrSibling> for Chunk<'a> {
    fn next_edge(
        &self,
        previous_edge_index: Option<usize>,
    ) -> Option<Edge<ChunkId, ChunkParentOrSibling>> {
        let next_idx = previous_edge_index.map(|e| e + 1).unwrap_or_default();
        let data = if next_idx < self.siblings.len() {
            self.siblings.get(next_idx).cloned()?
        } else {
            self.parents.get(next_idx - self.siblings.len()).cloned()?
        };

        Some(Edge::new(
            self.get_id(),
            data,
            next_idx,
            ChunkParentOrSibling,
        ))
    }
}

impl<'a> Edges<ChunkId, ChunkChild> for Chunk<'a> {
    fn next_edge(&self, previous_edge_index: Option<usize>) -> Option<Edge<ChunkId, ChunkChild>> {
        let next_idx = previous_edge_index.map(|e| e + 1).unwrap_or_default();
        let data = self.children.get(next_idx).cloned()?;

        Some(Edge::new(self.get_id(), data, next_idx, ChunkChild))
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Origin<'a> {
    #[serde(borrow)]
    pub loc: Cow<'a, str>,
    pub module_identifier: ModuleIdentifier,
    #[serde(default)]
    pub module_id: Option<ModuleId>,
    pub module_name: RelativeModulePath<'a>,
    #[serde(default)]
    pub reasons: Reasons<'a>,
}
