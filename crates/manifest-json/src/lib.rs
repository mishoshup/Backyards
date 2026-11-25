use manifest::{Manifest, ManifestReadError, ManifestReader};

#[derive(Clone, Copy, Debug, Default)]
pub struct JsonManifestReader;

impl ManifestReader for JsonManifestReader {
    fn read(&self, text: &str) -> Result<Manifest, ManifestReadError> {
        serde_json::from_str(text).map_err(ManifestReadError::new)
    }
}
