use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize)]
pub struct EntryPoint<'a> {
    #[serde(borrow)]
    pub name: Cow<'a, str>,
}
