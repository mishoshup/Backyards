use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawManifest")]
pub struct Manifest {
    sections: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct RawManifest {
    schema_version: u32,
    #[serde(flatten)]
    sections: BTreeMap<String, Value>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    #[error("schema_version {found} is not supported, expected {supported}")]
    UnsupportedSchemaVersion { found: u32, supported: u32 },
    #[error("section {name} is invalid: {message}")]
    InvalidSection { name: String, message: String },
}

impl Manifest {
    pub fn section<T: DeserializeOwned>(&self, name: &str) -> Result<Option<T>, ManifestError> {
        self.sections
            .get(name)
            .map(|value| read_section(name, value))
            .transpose()
    }

    #[must_use]
    pub fn unclaimed_sections(&self, claimed: &[&str]) -> Vec<&str> {
        self.sections
            .keys()
            .map(String::as_str)
            .filter(|name| !claimed.contains(name))
            .collect()
    }
}

fn read_section<T: DeserializeOwned>(name: &str, value: &Value) -> Result<T, ManifestError> {
    T::deserialize(value).map_err(|source| ManifestError::InvalidSection {
        name: name.to_owned(),
        message: source.to_string(),
    })
}

impl TryFrom<RawManifest> for Manifest {
    type Error = ManifestError;

    fn try_from(raw: RawManifest) -> Result<Self, Self::Error> {
        validate_schema_version(raw.schema_version)?;
        Ok(Self {
            sections: raw.sections,
        })
    }
}

const fn validate_schema_version(found: u32) -> Result<(), ManifestError> {
    if found == SUPPORTED_SCHEMA_VERSION {
        return Ok(());
    }
    Err(ManifestError::UnsupportedSchemaVersion {
        found,
        supported: SUPPORTED_SCHEMA_VERSION,
    })
}
