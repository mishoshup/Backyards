#[derive(Debug, thiserror::Error)]
#[error("{source}")]
pub struct ExternalError {
    #[source]
    source: Box<dyn std::error::Error + Send + Sync>,
}

impl ExternalError {
    pub fn new(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self {
            source: Box::new(source),
        }
    }
}
