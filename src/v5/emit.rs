use serde::Deserialize;
use std::borrow::Cow;
#[derive(Deserialize, Debug, Clone, Hash, PartialOrd, PartialEq)]
#[serde(transparent)]
pub struct AssetPath<'a>(#[serde(borrow)] Cow<'a, str>);
