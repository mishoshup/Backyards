use std::collections::BTreeSet;
use std::fmt;

use crate::review::ReviewerMode;
use crate::review::diff::AddedLines;
use crate::review::model::{ReviewAnswer, Violation};
use crate::review::prompt::{RuleFile, collect_allowed_rule_identifiers};

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

const DROPPED_BODY_PREVIEW_CHARACTERS: usize = 300;

impl DroppedFinding {
    #[must_use]
    pub fn describe(&self) -> String {
        let preview: String = self
            .violation
            .body
            .chars()
            .take(DROPPED_BODY_PREVIEW_CHARACTERS)
            .collect();
        format!(
            "{}:{} {}: {preview}",
            self.violation.path, self.violation.line, self.reason
        )
    }
}

#[must_use]
pub fn validate_answer(
    answer: ReviewAnswer,
    mode: ReviewerMode,
    added: &AddedLines,
    rules: &[RuleFile],
) -> ValidationOutcome {
    let allowed = collect_allowed_rule_identifiers(rules);
    validate_violations(answer.violations, mode, added, &allowed)
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
    if !is_line_reviewable(added, mode, violation) {
        return Some(DropReason::LineNotAdded);
    }
    find_rule_drop_reason(violation, mode, allowed_rules)
}

fn is_line_reviewable(added: &AddedLines, mode: ReviewerMode, violation: &Violation) -> bool {
    if mode.should_include_rule_files() {
        added.contains(&violation.path, violation.line)
    } else {
        added.contains_visible(&violation.path, violation.line)
    }
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
