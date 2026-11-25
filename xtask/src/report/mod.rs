mod error;
mod finding;
mod line_scan;

pub use error::GateError;
pub use finding::{Finding, Severity, count_errors, pluralize, report_findings, write_report_line};
pub use line_scan::{ScanRule, describe_matches, report_matching_lines};
