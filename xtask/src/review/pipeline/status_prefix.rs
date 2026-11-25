use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPrefix(String);

impl StatusPrefix {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StatusPrefix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for StatusPrefix {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if is_valid_status_prefix(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(
                "status prefix must use lowercase letters, digits, '/', '_' and '-' only"
                    .to_owned(),
            )
        }
    }
}

fn is_valid_status_prefix(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(is_allowed_status_character)
        && is_alphanumeric_edge(value.chars().next())
        && is_alphanumeric_edge(value.chars().last())
}

const fn is_allowed_status_character(character: char) -> bool {
    character.is_ascii_lowercase()
        || character.is_ascii_digit()
        || matches!(character, '/' | '_' | '-')
}

fn is_alphanumeric_edge(character: Option<char>) -> bool {
    character.is_some_and(|symbol| symbol.is_ascii_alphanumeric())
}
