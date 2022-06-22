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

use crate::chunk::ChunkId;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize, Default, Debug)]
pub struct EntryPoint<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    pub chunks: Cow<'a, [ChunkId]>,
}

impl<'a> crate::common::entry::Entrypoint for EntryPoint<'a> {
    fn chunks(&self) -> &[ChunkId] {
        self.chunks.as_ref()
    }

    fn name(&self) -> &str {
        self.name.as_ref()
    }
}
