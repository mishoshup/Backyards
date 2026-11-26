use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryName(String);

impl RepositoryName {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for RepositoryName {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.split_once('/') {
            Some((owner, name)) if is_name_part(owner) && is_name_part(name) => {
                Ok(Self(text.to_owned()))
            }
            _ => Err("repository must be written as owner/name".to_owned()),
        }
    }
}

fn is_name_part(part: &str) -> bool {
    !part.is_empty()
        && part.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}
