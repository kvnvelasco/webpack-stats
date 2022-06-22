use crate::common::ChunkId;
use crate::common::{DurationMillis, ModuleId, ModuleIdentifier, ModuleName, SizeBytes};
use crate::v5::asset::Asset;
use crate::v5::reason::Reason;
use crate::v5::source::SourceText;
use empty_type::{Empty, EmptyType};
use serde::{Deserialize, Deserializer};

#[derive(Debug)]
pub enum Module<'a> {
    /// Modules that can be pointed back to source code
    Output(OutputModule<'a>),
    /// Webpack outputs its own runtime modules
    Runtime { name: ModuleName<'a> },
}

impl<'a, 'de: 'a> Deserialize<'de> for Module<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // attempt to deserialize an empty output module
        let mut output_module: Empty<OutputModule> = Deserialize::deserialize(deserializer)?;

        use empty_type::Container;

        if output_module.id.is_none() {
            Ok(Self::Runtime {
                name: output_module.name.open(),
            })
        } else {
            Ok(Self::Output(output_module.resolve()))
        }
    }
}

#[derive(Deserialize, Debug, Default, EmptyType)]
#[serde(rename_all = "camelCase")]
#[empty(bounds = "'a", deserialize, default)]
pub struct OutputModule<'a> {
    #[serde(borrow)]
    pub assets: Vec<Asset<'a>>,
    /// Indicates that the module went through loaders,
    /// Parsing, and Code Generation
    pub built: bool,
    pub cacheable: bool,
    pub chunks: Vec<ChunkId>,

    // Webpack naming is bad.
    #[serde(rename = "errors")]
    pub error_count: u32,
    #[serde(rename = "warnings")]
    pub warning_count: u32,

    pub failed: bool,
    pub id: ModuleId,
    pub identifier: ModuleIdentifier<'a>,
    /// Undocumented by webpack in any meaningful way.
    // TODO: What even this webpack? Do you even docs?
    pub name: ModuleName<'a>,
    pub optional: bool,
    pub prefetched: bool,
    /// Every module also contains a list of reasons objects describing why
    /// that module was included in the dependency graph. Each "reason" is similar to the origins
    pub reasons: Vec<Reason<'a>>,
    pub size: SizeBytes,
    pub source: Option<SourceText<'a>>,
    pub profile: Profile,
}

#[derive(Deserialize, Debug, Default)]
pub struct Profile {
    pub building: DurationMillis,
    pub dependencies: DurationMillis,
    pub factory: DurationMillis,
}
