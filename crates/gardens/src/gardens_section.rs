use std::collections::HashSet;

use serde::Deserialize;

use crate::{GardenName, GardenSpecification};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(try_from = "Vec<GardenSpecification>")]
pub struct GardensSection {
    gardens: Vec<GardenSpecification>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GardensSectionError {
    #[error("garden {name} is declared more than once")]
    DuplicateGardenName { name: GardenName },
}

impl GardensSection {
    pub const NAME: &'static str = "gardens";

    #[must_use]
    pub fn gardens(&self) -> &[GardenSpecification] {
        &self.gardens
    }
}

impl TryFrom<Vec<GardenSpecification>> for GardensSection {
    type Error = GardensSectionError;

    fn try_from(gardens: Vec<GardenSpecification>) -> Result<Self, Self::Error> {
        validate_unique_names(&gardens)?;
        Ok(Self { gardens })
    }
}

fn validate_unique_names(gardens: &[GardenSpecification]) -> Result<(), GardensSectionError> {
    let mut seen = HashSet::with_capacity(gardens.len());
    for specification in gardens {
        if !seen.insert(&specification.name) {
            return Err(GardensSectionError::DuplicateGardenName {
                name: specification.name.clone(),
            });
        }
    }
    Ok(())
}
