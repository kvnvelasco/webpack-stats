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

//! # Wepback V5 Stats
//!
//! As much of the webpack stats file as described in
//! [webpack docs](https://webpack.js.org/api/stat)
//!

use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::common::chunk::ChunkName;
use asset::Asset;

use crate::v5::chunk::Chunks;
use crate::v5::module::Modules;
use crate::DurationMillis;

use crate::v5::entry_point::EntryPoint;
use emit::AssetPath;

pub mod asset;
pub mod chunk;
pub mod emit;
pub mod entry_point;
pub mod module;
pub mod reason;

/// # Webpack stats file
///
/// Deserialized representation of the webpack v5 stats file. Will
/// try to borrow as much as it can from the underlying buffer.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Stats<'a> {
    /// Version of webpack used for the compilation (5.x.x)
    pub version: Cow<'a, str>,
    /// Compilation specific hash
    pub hash: Cow<'a, str>,
    /// Compilation time in milliseconds
    pub time: DurationMillis,
    /// Undocumented by webpack
    pub public_path: Cow<'a, str>,
    /// path to webpack output directory
    pub output_path: Cow<'a, str>,
    /// Chunk name to emitted asset(s) mapping
    #[serde(borrow)]
    pub assets_by_chunk_name: ChunkMapping<'a>,
    pub entrypoints: HashMap<Cow<'a, str>, EntryPoint<'a>>,
    pub assets: Vec<Asset<'a>>,
    pub chunks: Chunks<'a>,
    pub modules: Modules<'a>,
    // pub entry_points: Vec<EntryPoint<'a>>

    // TODO:
    #[serde(skip)]
    errors: Vec<()>,
    errors_count: usize,
    #[serde(skip)]
    warnings: Vec<()>,
    warnings_count: usize,
    children: Vec<Self>,
}

type ChunkMapping<'a> = HashMap<ChunkName<'a>, Vec<AssetPath<'a>>>;

#[cfg(test)]
mod tests;
