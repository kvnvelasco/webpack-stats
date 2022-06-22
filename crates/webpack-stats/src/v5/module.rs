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
use crate::common::{DurationMillis, SizeBytes};
use std::collections::{HashMap, HashSet};
use std::iter::once;
// use crate::v5::asset::Asset;
use crate::v5::reason::Reasons;
use empty_type::{Empty, EmptyType};
use serde::{Deserialize, Deserializer};

use crate::common::import::{ImportType, SourceText};
use crate::common::module::{ModuleId, ModuleIdentifier, ModuleName};
use crate::import::ResolvedModule;
use crate::module::{IncludedModuleNames, ModuleChunks};
use meshed::prelude::*;

#[derive(Debug, Default)]
pub struct Modules<'a> {
    pub modules: Vec<Module<'a>>,
}

impl<'a> crate::common::module::Modules<Module<'a>> for Modules<'a> {}

impl<'de: 'a, 'a> Deserialize<'de> for Modules<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let empty_vec: Vec<Empty<Module<'a>>> = Deserialize::deserialize(deserializer)?;

        let materialized_vec = empty_vec.into_iter().map(|i| i.resolve()).collect();
        Ok(Self {
            modules: materialized_vec,
        })
    }
}

impl<'a> Query<ModuleIdentifier, Module<'a>> for Modules<'a> {
    fn query(&self, _identifier: &ModuleIdentifier) -> Option<&Module<'a>> {
        panic!("Query should be called on the module's index not on the modules iterator itself.")
    }

    fn all(&self) -> Vec<&Module<'a>> {
        self.modules
            .iter()
            .flat_map(|module| module.modules.all().into_iter().chain(once(module)))
            .collect()
    }

    fn create_index(&self) -> HashMap<ModuleIdentifier, Link<ModuleIdentifier, Module<'a>>> {
        let mut map: HashMap<_, _> = Default::default();

        for module in self.modules.iter() {
            for child in module.modules.modules.iter() {
                map.insert(child.get_id(), Link::Link(module.get_id()));
            }
            map.insert(module.get_id(), Link::Value(module));
        }
        map
    }
}

impl<'a> ExtractData<IncludedModuleNames> for Module<'a> {
    fn extract_data(&self) -> IncludedModuleNames {
        let mut included_names: HashSet<_> = Default::default();
        included_names.insert(self.name.0.to_string());

        for child in self.modules.all() {
            included_names.insert(child.name.0.to_string());
            let data: IncludedModuleNames = child.extract_data();
            included_names.extend(data.0.into_iter());
        }

        IncludedModuleNames(included_names)
    }
}

impl<'a> ExtractData<ModuleChunks> for Module<'a> {
    fn extract_data(&self) -> ModuleChunks {
        HashSet::from_iter(self.chunks.iter().cloned())
    }
}

impl<'a> Label for Module<'a> {
    type Label = ModuleName;

    fn label(&self) -> Self::Label {
        self.name.clone()
    }
}

impl<'a> crate::common::module::Module for Module<'a> {}

impl<'a> Identifiable<ModuleIdentifier> for Module<'a> {
    fn get_id(&self) -> ModuleIdentifier {
        self.identifier.clone()
    }
}

impl<'a> Edges<ModuleIdentifier, (ImportType, ResolvedModule)> for Module<'a> {
    fn next_edge(
        &self,
        previous_edge_index: Option<usize>,
    ) -> Option<Edge<ModuleIdentifier, (ImportType, ResolvedModule)>> {
        let next_index = previous_edge_index.map(|e| e + 1).unwrap_or_default();
        let reason = self.reasons.get(next_index)?;
        Some(Edge::new(
            self.get_id(),
            reason.module_identifier.clone(),
            next_index,
            (
                reason.r#type,
                ResolvedModule(reason.resolved_module.clone()),
            ),
        ))
    }
}

#[derive(Deserialize, Debug, EmptyType)]
#[serde(rename_all = "camelCase")]
#[empty(bounds = "'a", deserialize)]
pub struct Module<'a> {
    // #[serde(borrow)]
    pub assets: Vec<serde_json::Value>,
    /// Indicates that the module went through loaders,
    /// Parsing, and Code Generation
    pub built: bool,
    #[empty(fail_safe)]
    pub cacheable: bool,
    pub chunks: Vec<ChunkId>,

    // Webpack naming is bad.
    #[serde(rename = "errors")]
    pub error_count: u32,
    #[serde(rename = "warnings")]
    pub warning_count: u32,

    pub failed: bool,
    /// Possibly a relic of the past? Also undocumented by webpack. ModuleIdentifier / identifier
    /// is a better unique name. Use that if possible.
    #[empty(fail_safe)]
    pub id: Option<ModuleId>,
    pub identifier: ModuleIdentifier,
    pub name: ModuleName,
    pub optional: bool,
    #[serde(default)]
    pub prefetched: bool,
    /// Every module also contains a list of reasons objects describing why
    /// that module was included in the dependency graph. Each "reason" is similar to the origins
    #[serde(borrow)]
    pub reasons: Reasons<'a>,
    pub size: SizeBytes,
    pub source: Option<SourceText<'a>>,
    #[empty(default)]
    pub profile: Profile,
    #[empty(default)]
    pub modules: Modules<'a>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Profile {
    pub building: DurationMillis,
    pub dependencies: DurationMillis,
    pub factory: DurationMillis,
}
