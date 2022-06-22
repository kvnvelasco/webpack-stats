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

extern crate core;

// # Webpack stats
mod common;
pub(crate) mod rc;

pub use common::*;
use std::borrow::Cow;

#[cfg(feature = "v5")]
pub mod v5;

#[derive(serde::Deserialize)]
pub struct Version<'a> {
    pub version: Cow<'a, str>,
}

pub enum WebpackStats<'a> {
    #[cfg(feature = "v5")]
    V5(v5::Stats<'a>),
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("Could not get version number from json")]
    VersionDeserializationError,
    #[error("Could not deserialize stats file: {0}")]
    StatsDeserializationError(#[from] serde_json::Error),
    #[error("Unsupported webpack stats version.")]
    UnsupportedVersion,
}

pub fn deserialize_any_version<'a>(
    source: &'a str,
) -> Result<WebpackStats<'a>, DeserializationError> {
    // get a version
    let version: Version<'a> = serde_json::from_str(source)
        .map_err(|_| DeserializationError::VersionDeserializationError)?;

    let version_major = version
        .version
        .trim()
        .chars()
        .next()
        .ok_or(DeserializationError::VersionDeserializationError)?;

    match version_major {
        #[cfg(feature = "v5")]
        '5' => Ok(WebpackStats::V5(serde_json::from_str(source)?)),
        _ => Err(DeserializationError::UnsupportedVersion),
    }
}
