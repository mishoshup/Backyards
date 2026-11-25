use std::collections::HashSet;

use crate::{GardenName, GardenSpecification, GardensSection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanAction {
    Create(GardenSpecification),
    Keep(GardenName),
    LeaveUnmanaged(GardenName),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    actions: Vec<PlanAction>,
}

impl Plan {
    #[must_use]
    pub fn actions(&self) -> &[PlanAction] {
        &self.actions
    }
}

#[must_use]
pub fn plan_gardens(section: &GardensSection, existing: &[GardenName]) -> Plan {
    let existing_names: HashSet<&GardenName> = existing.iter().collect();
    let mut actions = actions_for_declared_gardens(section, &existing_names);
    actions.extend(actions_for_unmanaged_gardens(section, existing));
    Plan { actions }
}

fn actions_for_declared_gardens(
    section: &GardensSection,
    existing: &HashSet<&GardenName>,
) -> Vec<PlanAction> {
    let mut actions = Vec::with_capacity(section.gardens().len());
    for specification in section.gardens() {
        actions.push(action_for_declared_garden(specification, existing));
    }
    actions
}

fn action_for_declared_garden(
    specification: &GardenSpecification,
    existing: &HashSet<&GardenName>,
) -> PlanAction {
    if existing.contains(&specification.name) {
        PlanAction::Keep(specification.name.clone())
    } else {
        PlanAction::Create(specification.clone())
    }
}

fn actions_for_unmanaged_gardens(
    section: &GardensSection,
    existing: &[GardenName],
) -> Vec<PlanAction> {
    let declared: HashSet<&GardenName> = section
        .gardens()
        .iter()
        .map(|specification| &specification.name)
        .collect();
    let mut actions = Vec::new();
    for name in existing {
        if !declared.contains(name) {
            actions.push(PlanAction::LeaveUnmanaged(name.clone()));
        }
    }
    actions
}
