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
use crate::import::{ImportType, ResolvedModule};
use meshed::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use crate::rc::RefCount;

pub struct IncludedModuleNames(pub HashSet<String>);
pub type ModuleChunks = HashSet<ChunkId>;

impl IncludedModuleNames {
    pub fn included_name_suffix(&self, pattern: impl AsRef<str>) -> Option<&String> {
        self.0.iter().find(|name| name.ends_with(pattern.as_ref()))
    }

    pub fn included_names_contains(&self, pattern: impl AsRef<str>) -> Option<&String> {
        self.0.iter().find(|name| name.contains(pattern.as_ref()))
    }
}

/// An internal webpack identifier for the module. Guaranteed to uniquely identify it
/// Docs say: "(webpack)\\test\\browsertest\\lib\\index.web.js"
#[derive(Deserialize, Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ModuleIdentifier(pub RefCount<str>);

impl Clone for ModuleIdentifier {
    fn clone(&self) -> Self {
        Self(RefCount::clone(&self.0))
    }
}

impl Display for ModuleIdentifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Identity for ModuleIdentifier {}

#[derive(Deserialize, Serialize, Debug)]
#[serde(transparent)]
pub struct ModuleName(pub RefCount<str>);

impl Default for ModuleName {
    fn default() -> Self {
        Self("".into())
    }
}

impl Display for ModuleName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <str as Display>::fmt(&self.0, f)
    }
}

impl Clone for ModuleName {
    fn clone(&self) -> Self {
        Self(RefCount::clone(&self.0))
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct ModuleId(u32);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct RelativeModulePath<'a>(#[serde(borrow)] Cow<'a, str>);

pub trait Modules<T>: Query<ModuleIdentifier, T>
where
    T: Module,
{
}
pub trait Module:
    Identifiable<ModuleIdentifier>
    + Edges<ModuleIdentifier, (ImportType, ResolvedModule)>
    + ExtractData<IncludedModuleNames>
    + ExtractData<ModuleChunks>
    + Label<Label = ModuleName>
{
}
