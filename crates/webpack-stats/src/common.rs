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

pub mod chunk;
pub mod entry;
pub mod import;
pub mod module;

use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Add;

/// Represents a duration in miliseconds
#[derive(Debug, Default)]
pub struct DurationMillis(std::time::Duration);

impl<'de> Deserialize<'de> for DurationMillis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let miliseconds = u64::deserialize(deserializer)?;
        Ok(Self(std::time::Duration::from_millis(miliseconds)))
    }
}

/// Represents a size in bytes.
#[derive(Default, Deserialize, Copy, Clone, PartialOrd, PartialEq)]
#[repr(transparent)]
pub struct SizeBytes(pub f32);

impl Display for SizeBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0 > (1024.0 * 1024.0 * 1024.0) {
            write!(f, "{} GiB", self.0 / (1024.0 * 1024.0 * 1024.0))
        } else if self.0 > (1024.0 * 1024.0) {
            write!(f, "{} MiB", self.0 / (1024.0 * 1024.0))
        } else if self.0 > 1024.0 {
            write!(f, "{} KiB", self.0 / 1024.0)
        } else {
            write!(f, "{} B", self.0)
        }
    }
}

impl Debug for SizeBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self, f)
    }
}

impl Add for SizeBytes {
    type Output = SizeBytes;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

/// Abstracts over json values that can be a string or a list of strings
/// When possible, will store values as a &'de str when deserializing
pub enum Strings<'a, I>
where
    I: ?Sized + ToOwned,
{
    Str(Cow<'a, I>),
    Strs(Vec<Cow<'a, I>>),
}

impl<'a, I> IntoIterator for Strings<'a, I>
where
    I: ?Sized + ToOwned,
{
    type Item = Cow<'a, I>;
    type IntoIter = <Vec<Cow<'a, I>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Strings::Str(v) => vec![v].into_iter(),
            Strings::Strs(v) => v.into_iter(),
        }
    }
}

impl<'a, I> Debug for Strings<'a, I>
where
    I: ?Sized + ToOwned + Debug,
    Cow<'a, I>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Strings::Str(v) => Debug::fmt(v, f),
            Strings::Strs(v) => Debug::fmt(v, f),
        }
    }
}

impl<I> Default for Strings<'_, I>
where
    I: ToOwned + ?Sized,
    <I as ToOwned>::Owned: Default,
{
    fn default() -> Self {
        Self::Str(Cow::default())
    }
}

impl<'de, 'a> Deserialize<'de> for Strings<'a, str>
where
    'de: 'a,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visit<'a>(PhantomData<&'a str>);

        impl<'v> Visitor<'v> for Visit<'v> {
            type Value = Strings<'v, str>;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("a List<str> or an str")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Self::Value::Str(Cow::Owned(v.to_owned())))
            }

            fn visit_borrowed_str<E>(self, v: &'v str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Self::Value::Str(Cow::Borrowed(v)))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'v>,
            {
                let mut value = vec![];
                loop {
                    let next = seq.next_element()?;
                    match next {
                        Some(v) => {
                            value.push(v);
                        }
                        None => {
                            break;
                        }
                    }
                }

                Ok(Self::Value::Strs(value))
            }
        }

        deserializer.deserialize_any(Visit(PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Strings;
    use serde::Deserialize;

    #[test]
    fn item_or_n_items() {
        let json_str = r#"
          {
            "key": "value\"",
            "k": ["s", "v"]
          }
        "#;
        #[derive(Deserialize, Debug)]
        struct Test<'a> {
            #[serde(borrow)]
            key: Strings<'a, str>,
            k: Strings<'a, str>,
        }

        let value: Test<'_> = serde_json::from_str(json_str).unwrap();

        assert_eq!(value.key.into_iter().count(), 1);
        assert_eq!(value.k.into_iter().count(), 2);
    }
}
