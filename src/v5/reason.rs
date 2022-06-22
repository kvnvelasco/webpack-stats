//! Modules or chunks may be in the graph for a multitude of
//! [reasons](Reason). This type attempts to abstract over the constellation
//! of boolean flags in webpack stats to provide some semantics
//!
//! [`Reason`] is meant to be used in the context of collections. It does
//! not provide a default value.

use super::source::{ImportString, ImportType};
use crate::common::{ModuleId, ModuleName, RelativeModulePath};
use empty_type::{Container, Empty, EmptyType};
use serde::{Deserialize, Deserializer};
use std::borrow::{BorrowMut, Cow};

#[derive(Debug)]
pub enum Reason<'a> {
    /// This module was requested by some other module
    Imported(ImportedReason<'a>),
    /// The module was directly required by an entrypoint.
    /// This corresponds to having no module_id present
    /// in the stats file
    Entry,
}

impl<'de: 'a, 'a> Deserialize<'de> for Reason<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Container<'a> = Empty<ImportedReason<'a>>;

        let value = <Container<'a> as Deserialize>::deserialize(deserializer)?;

        if value.module_id.is_none() {
            Ok(Self::Entry)
        } else {
            Ok(Self::Imported(value.resolve()))
        }
    }
}

/// Metadata to describe the source of an import.
/// Normally helps locate the upstream modules that
/// required this one
#[derive(Deserialize, Debug, Default, EmptyType)]
#[serde(rename_all = "camelCase", default)]
#[empty(bounds = "'a", deserialize, default)]
pub struct ImportedReason<'a> {
    #[serde(borrow)]
    pub loc: Cow<'a, str>,
    pub module: RelativeModulePath<'a>,
    pub module_id: ModuleId,
    pub module_name: ModuleName<'a>,
    pub r#type: ImportType,
    pub user_request: ImportString<'a>,
}
