use serde::{Deserialize, Serialize};

const MAXIMUM_LENGTH: usize = 63;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GardenName(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GardenNameError {
    #[error("garden name is empty")]
    Empty,
    #[error("garden name has {length} characters, the maximum is {maximum}")]
    TooLong { length: usize, maximum: usize },
    #[error(
        "garden name contains {character:?}, only lowercase letters, digits and hyphens are allowed"
    )]
    InvalidCharacter { character: char },
    #[error("garden name must not start or end with a hyphen")]
    HyphenAtEdge,
}

impl GardenName {
    pub fn new(candidate: impl Into<String>) -> Result<Self, GardenNameError> {
        let candidate = candidate.into();
        validate_length(&candidate)?;
        validate_characters(&candidate)?;
        validate_edges(&candidate)?;
        Ok(Self(candidate))
    }

    #[must_use]
    pub const fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

const fn validate_length(candidate: &str) -> Result<(), GardenNameError> {
    if candidate.is_empty() {
        return Err(GardenNameError::Empty);
    }
    if candidate.len() > MAXIMUM_LENGTH {
        return Err(GardenNameError::TooLong {
            length: candidate.len(),
            maximum: MAXIMUM_LENGTH,
        });
    }
    Ok(())
}

fn validate_characters(candidate: &str) -> Result<(), GardenNameError> {
    candidate
        .chars()
        .find(|character| !is_allowed_character(*character))
        .map_or(Ok(()), |character| {
            Err(GardenNameError::InvalidCharacter { character })
        })
}

const fn is_allowed_character(character: char) -> bool {
    character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
}

fn validate_edges(candidate: &str) -> Result<(), GardenNameError> {
    if candidate.starts_with('-') || candidate.ends_with('-') {
        return Err(GardenNameError::HyphenAtEdge);
    }
    Ok(())
}

impl TryFrom<String> for GardenName {
    type Error = GardenNameError;

    fn try_from(candidate: String) -> Result<Self, Self::Error> {
        Self::new(candidate)
    }
}

impl From<GardenName> for String {
    fn from(name: GardenName) -> Self {
        name.0
    }
}

impl std::fmt::Display for GardenName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}
