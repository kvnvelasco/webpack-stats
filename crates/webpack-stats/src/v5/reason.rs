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

//! Modules or chunks may be in the graph for a multitude of
//! [reasons](Reason). This type attempts to abstract over the constellation
//! of boolean flags in webpack stats to provide some semantics
//!
//! [`Reason`] is meant to be used in the context of collections. It does
//! not provide a default value.

use crate::common::import::{ImportString, ImportType};
use crate::common::module::{ModuleId, ModuleIdentifier, ModuleName, RelativeModulePath};
use empty_type::{Empty, EmptyType};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Debug, Default)]
pub struct Reasons<'a> {
    reasons: Vec<Reason<'a>>,
}

impl<'a> Deref for Reasons<'a> {
    type Target = Vec<Reason<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.reasons
    }
}

impl<'de: 'a, 'a> Deserialize<'de> for Reasons<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        type Container<'a> = Vec<Empty<Reason<'a>>>;

        let value = <Container<'a> as Deserialize>::deserialize(deserializer)?;

        let resolved = value
            .into_iter()
            .filter(|module| module.module_identifier.is_some())
            .map(Empty::resolve)
            .collect();
        Ok(Self { reasons: resolved })
    }
}

/// Metadata to describe the source of an import.
/// Normally helps locate the upstream modules that
/// required this one
#[derive(Deserialize, Debug, EmptyType)]
#[serde(rename_all = "camelCase", default)]
#[empty(bounds = "'a", deserialize)]
pub struct Reason<'a> {
    #[serde(borrow)]
    #[empty(fail_safe)]
    pub loc: Cow<'a, str>,
    pub module: RelativeModulePath<'a>,
    pub module_id: Option<ModuleId>,
    pub module_name: ModuleName,
    pub resolved_module: ModuleName,
    pub module_identifier: ModuleIdentifier,
    pub r#type: ImportType,
    #[empty(fail_safe)]
    pub user_request: ImportString<'a>,
}
