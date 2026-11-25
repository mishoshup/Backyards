use kernel::ExternalError;

use crate::{GardenName, GardenSpecification};

#[derive(Debug, thiserror::Error)]
#[error("garden backend failed: {source}")]
pub struct GardenBackendError {
    #[source]
    source: ExternalError,
}

impl GardenBackendError {
    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: ExternalError::new(source),
        }
    }
}

pub trait GardenBackend {
    fn list_managed(
        &self,
    ) -> impl Future<Output = Result<Vec<GardenName>, GardenBackendError>> + Send;

    fn create(
        &self,
        specification: &GardenSpecification,
    ) -> impl Future<Output = Result<(), GardenBackendError>> + Send;

    fn remove(
        &self,
        name: &GardenName,
    ) -> impl Future<Output = Result<(), GardenBackendError>> + Send;
}
