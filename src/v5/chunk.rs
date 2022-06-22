use crate::common::{
    ChunkId, ChunkName, ModuleId, ModuleIdentifier, ModuleName, RelativeModulePath, SizeBytes,
};
use crate::v5::module::Module;
use crate::v5::reason::Reason;
use crate::v5::source::SourceFilePath;
use serde::Deserialize;
use std::borrow::Cow;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Chunk<'a> {
    pub id: u32,
    pub entry: bool,
    pub initial: bool,
    #[serde(borrow)]
    pub modules: Vec<Module<'a>>,
    pub files: Vec<SourceFilePath<'a>>,
    #[serde(borrow)]
    pub names: Vec<ChunkName<'a>>,
    #[serde(borrow)]
    pub origins: Vec<Origin<'a>>,
    pub parents: Vec<ChunkId>,
    pub rendered: bool,
    pub size: SizeBytes,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Origin<'a> {
    #[serde(borrow)]
    pub loc: Cow<'a, str>,
    pub module_identifier: ModuleIdentifier<'a>,
    pub module_id: Option<ModuleId>,
    pub module_name: RelativeModulePath<'a>,
    pub reasons: Vec<Reason<'a>>,
}
