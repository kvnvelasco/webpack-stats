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

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::HashMap;

#[derive(Default)]
pub struct AnymapCell(Cell<Anymap>);

impl AnymapCell {
    pub fn insert<A: 'static>(&self, value: A) -> Option<A> {
        let mut map = self.0.take();
        let output = map.insert(value);
        self.0.set(map);
        output
    }

    pub fn get<A: Clone + 'static>(&self) -> Option<A> {
        let mut map = self.0.take();
        let output = map.get();
        self.0.set(map);
        output
    }
}

#[derive(Default)]
pub struct Anymap(HashMap<TypeId, Box<dyn Any>>);

impl Anymap {
    pub fn insert<A: 'static>(&mut self, value: A) -> Option<A> {
        let annotations = &mut self.0;
        let type_id = value.type_id();

        let anno = annotations.remove(&type_id);
        annotations.insert(type_id, Box::new(value) as Box<dyn Any>);

        anno.map(|d| *(d.downcast::<A>().unwrap()))
    }

    pub fn remove<A: 'static>(&mut self) -> Option<A> {
        let annotations = &mut self.0;
        let type_id = TypeId::of::<A>();

        let anno = annotations.remove(&type_id);

        anno.map(|d| *(d.downcast::<A>().unwrap()))
    }

    pub fn get<A: Clone + 'static>(&mut self) -> Option<A> {
        let item: A = self.remove()?;
        let cloned = item.clone();
        self.insert(cloned);

        Some(item)
    }
}
