use std::io::Write;

use crate::report::GateError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub area: &'static str,
    pub severity: Severity,
    pub message: String,
}

impl Finding {
    #[must_use]
    pub const fn error(area: &'static str, message: String) -> Self {
        Self {
            area,
            severity: Severity::Error,
            message,
        }
    }

    #[must_use]
    pub const fn warning(area: &'static str, message: String) -> Self {
        Self {
            area,
            severity: Severity::Warning,
            message,
        }
    }
}

#[must_use]
pub fn pluralize(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

#[must_use]
pub fn count_errors(findings: &[Finding]) -> usize {
    findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count()
}

pub fn write_report_line(message: &str) -> Result<(), GateError> {
    writeln!(std::io::stderr().lock(), "{message}").map_err(GateError::WriteReport)
}

pub fn report_findings(area: &'static str, findings: &[Finding]) -> Result<(), GateError> {
    let mut lines: Vec<String> = findings.iter().map(ToString::to_string).collect();
    lines.sort();
    for line in &lines {
        write_report_line(line)?;
    }
    match count_errors(findings) {
        0 => Ok(()),
        count => Err(GateError::Violations { area, count }),
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.severity {
            Severity::Error => write!(formatter, "{}: {}", self.area, self.message),
            Severity::Warning => write!(formatter, "{}: warning: {}", self.area, self.message),
        }
    }
}
