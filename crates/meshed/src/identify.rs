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

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::Deref;

/// Marker trait that some type corresponds to the identity of
/// some value. Used for graph traversal book-keeping (seen set, querying, etc)
///
/// This type must be cheaply clonable. Clones that re-allocate will negatively affect
/// performance of graph traversal
pub trait Identity: Hash + PartialEq + Eq + PartialOrd + Ord + Clone + Display + Debug {
    fn escaped(&self) -> String {
        let id = self.to_string();

        let mut output = String::new();
        let mut chars = id.chars();

        if let Some(first) = chars.next() {
            if first.is_ascii_alphabetic() || first == '_' {
                output.push(first);
            } else if first.is_ascii_alphanumeric() {
                output.push('_');
                output.push(first);
            } else {
                output.push('_');
            }
        }

        for character in chars {
            if character.is_ascii_alphanumeric() || character == '_' {
                output.push(character)
            } else {
                output.push('_')
            }
        }

        output
    }
}

macro_rules! impl_identity {
    ($($t:ty) +) => {
        $(impl Identity for $t {})*
    }
}

impl_identity! {
    i8 u8 i16 u16 i32 u32 i64 u64
    usize isize
    &'static str
}

/// A type is able to extract an Identifer from itself. The identifier
/// type is provided as an argument to the trait
///
/// struct Module {
///   id: Identifier
/// }
///
/// impl Identity for Identity {}
///
/// impl Identifialbe<Identity> for Module {
///   fn get_id(&self) -> Identity::Id {
///     &self.id
///   }
/// }
pub trait Identifiable<T: Identity> {
    fn get_id(&self) -> T;
}

impl<T, I: Identity, D> Identifiable<I> for D
where
    T: Identifiable<I>,
    D: Deref<Target = T>,
{
    fn get_id(&self) -> I {
        self.deref().get_id()
    }
}
