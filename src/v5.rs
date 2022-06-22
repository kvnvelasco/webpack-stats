//! # Wepback V5 Stats
//!
//! As much of the webpack stats file as described in
//! [webpack docs](https://webpack.js.org/api/stat)
//!

use serde::Deserialize;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::common::ChunkName;
use asset::Asset;

use crate::v5::module::Module;
use crate::DurationMillis;
use chunk::Chunk;
use emit::AssetPath;

pub mod asset;
pub mod chunk;
pub mod emit;
pub mod entry_point;
pub mod module;
pub mod reason;
pub mod source;

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
    pub assets: Vec<Asset<'a>>,
    pub chunks: Vec<Chunk<'a>>,
    pub modules: Vec<Module<'a>>,
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
