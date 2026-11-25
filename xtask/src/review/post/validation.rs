use std::collections::BTreeSet;
use std::fmt;

use crate::review::ReviewerMode;
use crate::review::diff::AddedLines;
use crate::review::model::Violation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropReason {
    PathNotInDiff,
    LineNotAdded,
    UnknownRuleIdentifier,
    BodyMissingRuleIdentifier,
}

impl fmt::Display for DropReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PathNotInDiff => "path is not in the diff",
            Self::LineNotAdded => "line is not an added line",
            Self::UnknownRuleIdentifier => "rule identifier is not defined",
            Self::BodyMissingRuleIdentifier => "body does not name its rule identifier",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DroppedFinding {
    pub violation: Violation,
    pub reason: DropReason,
}

#[derive(Debug, Default)]
pub struct ValidationOutcome {
    pub accepted: Vec<Violation>,
    pub dropped: Vec<DroppedFinding>,
}

#[must_use]
pub fn validate_violations(
    violations: Vec<Violation>,
    mode: ReviewerMode,
    added: &AddedLines,
    allowed_rules: &BTreeSet<&str>,
) -> ValidationOutcome {
    let mut outcome = ValidationOutcome::default();
    for violation in violations {
        match find_drop_reason(&violation, mode, added, allowed_rules) {
            Some(reason) => outcome.dropped.push(DroppedFinding { violation, reason }),
            None => outcome.accepted.push(violation),
        }
    }
    outcome
}

fn find_drop_reason(
    violation: &Violation,
    mode: ReviewerMode,
    added: &AddedLines,
    allowed_rules: &BTreeSet<&str>,
) -> Option<DropReason> {
    if !added.contains_path(&violation.path) {
        return Some(DropReason::PathNotInDiff);
    }
    if !added.contains(&violation.path, violation.line) {
        return Some(DropReason::LineNotAdded);
    }
    find_rule_drop_reason(violation, mode, allowed_rules)
}

fn find_rule_drop_reason(
    violation: &Violation,
    mode: ReviewerMode,
    allowed_rules: &BTreeSet<&str>,
) -> Option<DropReason> {
    if !mode.should_include_rule_files() {
        return None;
    }
    if !allowed_rules.contains(violation.rule_identifier.as_str()) {
        return Some(DropReason::UnknownRuleIdentifier);
    }
    if violation.body.contains(&violation.rule_identifier) {
        None
    } else {
        Some(DropReason::BodyMissingRuleIdentifier)
    }
}
