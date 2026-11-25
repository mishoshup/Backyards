use std::time::Duration;

pub const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
pub const IDLE_TIMEOUT: Duration = Duration::from_secs(90);
pub const TOTAL_TIMEOUT: Duration = Duration::from_mins(15);
pub const RETRY_DELAY: Duration = Duration::from_secs(5);
pub const PROGRESS_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Clone)]
pub struct ModelEndpoint {
    pub url: String,
    pub api_key: String,
    pub model: String,
    pub retry_delay: Duration,
    pub idle_timeout: Duration,
}

impl ModelEndpoint {
    #[must_use]
    pub const fn new(url: String, api_key: String, model: String) -> Self {
        Self {
            url,
            api_key,
            model,
            retry_delay: RETRY_DELAY,
            idle_timeout: IDLE_TIMEOUT,
        }
    }
}

impl std::fmt::Debug for ModelEndpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ModelEndpoint")
            .field("url", &self.url)
            .field("api_key", &"<redacted>")
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}
