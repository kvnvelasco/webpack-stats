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

use crate::module::ModuleName;
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct SourceFilePath<'a>(#[serde(borrow)] pub Cow<'a, Path>);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct SourceText<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Debug, Copy, Clone)]
pub enum ImportType {
    // TODO: Figure out what the actual webpack values here are. The documentation is
    // sparse as hell.
    /// Webpack require.context call.
    RequireContext,
    /// ES6 import statement
    Import,
    /// Deferred (async) import statement import()
    ImportDynamic,
    /// CJS Require statement
    Require,
    CJSSelfExport,
    /// Required by default as an entrypoint
    Entry,
    // Harmony imports
    Es6SideEffect,
    // export { } from "..."
    Es6ExportImport,

    // ?????
    ModuleDecorator,

    Url,

    ///
    AmdRequire,
    /// The value was missing from the stats file
    Empty,
}

impl Default for ImportType {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Error, Debug)]
#[error("Invalid import type: {msg}")]
pub struct ImportTypeError {
    msg: String,
}

impl FromStr for ImportType {
    type Err = ImportTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "require.context" => Ok(Self::RequireContext),
            "import" => Ok(Self::Import),
            "import()" => Ok(Self::ImportDynamic),
            "require" | "cjs require" | "cjs full require" => Ok(Self::Require),
            "entry" => Ok(Self::Entry),
            "harmony side effect evaluation" => Ok(Self::Es6SideEffect),
            "harmony import specifier" => Ok(Self::Import),

            "cjs self exports reference" => Ok(Self::CJSSelfExport),
            "cjs export require" => Ok(Self::CJSSelfExport),
            "harmony export imported specifier" => Ok(Self::Es6ExportImport),

            "module decorator" => Ok(Self::ModuleDecorator),
            "new URL()" => Ok(Self::Url),

            "amd require" => Ok(Self::AmdRequire),
            _ => Err(ImportTypeError { msg: s.to_owned() }),
        }
    }
}

impl<'de> Deserialize<'de> for ImportType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = <&str>::deserialize(deserializer)?;

        Self::from_str(v).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct ImportString<'a>(Cow<'a, str>);

pub struct ResolvedModule(pub ModuleName);
