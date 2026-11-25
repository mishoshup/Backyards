use std::num::{NonZeroU8, NonZeroU32};

use serde::{Deserialize, Serialize};

use crate::{GardenName, NetworkPolicy};

const DEFAULT_CPU_COUNT: NonZeroU8 = NonZeroU8::MIN;
const DEFAULT_MEMORY_MEBIBYTES: u32 = 512;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GardenSpecification {
    pub name: GardenName,
    pub image: String,
    #[serde(default = "default_cpu_count")]
    pub cpu_count: NonZeroU8,
    #[serde(default = "default_memory_mebibytes")]
    pub memory_mebibytes: NonZeroU32,
    #[serde(default)]
    pub network: NetworkPolicy,
}

const fn default_cpu_count() -> NonZeroU8 {
    DEFAULT_CPU_COUNT
}

fn default_memory_mebibytes() -> NonZeroU32 {
    NonZeroU32::new(DEFAULT_MEMORY_MEBIBYTES).unwrap_or(NonZeroU32::MIN)
}
