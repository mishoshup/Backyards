use kernel::ExternalError;

use crate::Manifest;

#[derive(Debug, thiserror::Error)]
#[error("manifest is invalid: {source}")]
pub struct ManifestReadError {
    #[source]
    source: ExternalError,
}

impl ManifestReadError {
    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: ExternalError::new(source),
        }
    }
}

pub trait ManifestReader {
    fn read(&self, text: &str) -> Result<Manifest, ManifestReadError>;
}
