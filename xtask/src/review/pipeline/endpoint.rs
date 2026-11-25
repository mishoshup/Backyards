use crate::review::ReviewerMode;
use crate::review::error::ReviewError;
use crate::review::model::ModelEndpoint;

const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_MODEL: &str = "deepseek-flash";

pub fn endpoint_for(mode: ReviewerMode) -> Result<ModelEndpoint, ReviewError> {
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .map_err(|error| ReviewError::unusable_environment("DEEPSEEK_API_KEY", &error))?;
    let base_url =
        std::env::var("DEEPSEEK_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_owned());
    let model = std::env::var(model_variable(mode)).unwrap_or_else(|_| DEFAULT_MODEL.to_owned());
    Ok(ModelEndpoint::new(
        format!("{base_url}/chat/completions"),
        api_key,
        model,
    ))
}

const fn model_variable(mode: ReviewerMode) -> &'static str {
    match mode {
        ReviewerMode::Inline => "REVIEW_MODEL_INLINE",
        ReviewerMode::Holistic => "REVIEW_MODEL_HOLISTIC",
    }
}
