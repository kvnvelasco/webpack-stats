use serde::de::{Error, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use zerovec::ule::AsULE;

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
#[derive(Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct SizeBytes(u64);

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
    use std::borrow::Cow;

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

/// An internal webpack identifier for the module.
/// Docs say: "(webpack)\\test\\browsertest\\lib\\index.web.js"
#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct ModuleIdentifier<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct ModuleName<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct ModuleId(u32);

#[derive(Deserialize, Debug, Default)]
#[serde(transparent)]
pub struct RelativeModulePath<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize, Debug, Clone, Hash, PartialOrd, PartialEq, Eq)]
#[serde(transparent)]
pub struct ChunkName<'a>(#[serde(borrow)] Cow<'a, str>);

#[derive(Deserialize, Debug, Copy, Clone)]
#[serde(transparent)]
pub struct ChunkId(u32);

impl AsULE for ChunkId {
    type ULE = <u32 as AsULE>::ULE;

    fn to_unaligned(self) -> Self::ULE {
        u32::to_unaligned(self.0)
    }

    fn from_unaligned(unaligned: Self::ULE) -> Self {
        Self(u32::from_unaligned(unaligned))
    }
}
