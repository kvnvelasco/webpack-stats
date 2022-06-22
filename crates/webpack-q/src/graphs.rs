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

pub mod ser;

use meshed::graph::GraphDefinition;
use webpack_stats::chunk::{
    ChunkChild, ChunkChildren, ChunkId, ChunkInitial, ChunkModules, ChunkParentOrSibling, Files,
};
use webpack_stats::import::{ImportType, ResolvedModule};
use webpack_stats::module::{ModuleChunks, ModuleIdentifier, ModuleName};
use webpack_stats::SizeBytes;

pub struct ModuleParentGraph;

impl GraphDefinition for ModuleParentGraph {
    type Id = ModuleIdentifier;
    type Label = ModuleName;
    type EdgeMeta = (ImportType, ResolvedModule);
    type NodeData = ModuleChunks;
}

pub struct ChunkGraph;

impl GraphDefinition for ChunkGraph {
    type Id = ChunkId;
    type Label = ChunkId;
    type EdgeMeta = ChunkChild;
    type NodeData = ChunkModules;
}
pub struct ChunkImportPathGraph;

impl GraphDefinition for ChunkImportPathGraph {
    type Id = ChunkId;
    type Label = ChunkId;
    type EdgeMeta = ChunkParentOrSibling;
    type NodeData = ();
}

pub struct ChunkLoadGraph;

impl GraphDefinition for ChunkLoadGraph {
    type Id = ChunkId;
    type Label = ChunkId;
    type EdgeMeta = ChunkChild;
    type NodeData = (ChunkChildren, SizeBytes, ChunkInitial, Files);
}
