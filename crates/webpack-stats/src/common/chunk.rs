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

use crate::module::ModuleIdentifier;
use crate::SizeBytes;
use meshed::prelude::*;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use zerovec::ule::AsULE;

#[derive(Deserialize, Serialize, Debug, Clone, Hash, PartialOrd, PartialEq, Eq)]
#[serde(transparent)]
pub struct ChunkName<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize, Serialize, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct ChunkId(pub u32);

impl Display for ChunkId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Identity for ChunkId {}

impl AsULE for ChunkId {
    type ULE = <u32 as AsULE>::ULE;

    fn to_unaligned(self) -> Self::ULE {
        u32::to_unaligned(self.0)
    }

    fn from_unaligned(unaligned: Self::ULE) -> Self {
        Self(u32::from_unaligned(unaligned))
    }
}

pub struct ChunkChild;
pub struct ChunkParentOrSibling;

#[repr(transparent)]
pub struct ChunkChildren(pub Vec<ChunkId>);
pub type ChunkModules = Vec<ModuleIdentifier>;
#[derive(Debug)]
pub struct ChunkInitial(pub bool);

#[derive(Debug)]
pub struct Files(pub Vec<String>);

pub trait Chunk:
    Identifiable<ChunkId>
    + Edges<ChunkId, ChunkChild>
    + Edges<ChunkId, ChunkParentOrSibling>
    + ExtractData<ChunkId>
    + ExtractData<ChunkChildren>
    + ExtractData<SizeBytes>
    + ExtractData<ChunkModules>
    + ExtractData<ChunkInitial>
    + ExtractData<Files>
    + Label<Label = ChunkId>
{
}

pub trait Chunks<T>: Query<ChunkId, T>
where
    T: Chunk,
{
}
