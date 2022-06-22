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

use crate::common::chunk::ChunkId;
use crate::common::chunk::ChunkName;
use std::borrow::Cow;
use zerovec::ZeroVec;

use crate::common::SizeBytes;
use serde::Deserialize;

// # Assets
// (Link to webpack docs)[https://webpack.js.org/api/stats/#asset-objects]
//
// Each assets object represents an output file emitted from the compilation.
// They all follow a similar structure:

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Asset<'a> {
    /// Undocumented by webpack.
    #[serde(borrow)]
    pub r#type: Cow<'a, str>,
    /// The `output` filename
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    /// The chunks this asset contains
    pub chunk_names: Vec<ChunkName<'a>>,
    /// Undocumented by webpack
    #[serde(skip)]
    pub chunk_id_hints: &'a [()],
    /// The chunk IDs this asset contains
    #[serde(borrow)]
    pub chunks: ZeroVec<'a, ChunkId>,
    /// Indicates whether or not the asset was compared with the same file on the output file system
    pub compared_for_emit: bool,
    /// The size of the file in bytes
    pub size: SizeBytes,
    pub info: AssetInfo<'a>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct AssetInfo<'a> {
    /// A flag telling whether the asset can be long term cached (contains a hash)
    pub immutable: bool,
    /// The size in bytes, only becomes available after asset has been emitted
    pub size: SizeBytes,
    /// A flag telling whether the asset is only used for development and doesn't count towards user-facing assets
    pub development: bool,
    pub hot_module_replacement: bool,
    #[serde(borrow)]
    pub source_filename: Cow<'a, str>,
    pub javascript_module: bool,
    pub minimized: bool,
}
