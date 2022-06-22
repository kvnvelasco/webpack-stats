use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::path::Path;
use std::str::FromStr;

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct SourceFilePath<'a>(#[serde(borrow)] Cow<'a, Path>);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct SourceText<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Debug)]
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
    /// Required by default as an entrypoint
    Entry,
    /// The value was missing from the stats file
    Empty,
}

impl Default for ImportType {
    fn default() -> Self {
        Self::Empty
    }
}

use thiserror::Error;

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
            "require" => Ok(Self::Require),
            "entry" => Ok(Self::Entry),
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
