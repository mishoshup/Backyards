use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use xtask::command::{Cli, run_task};
use xtask::report::write_report_line;
use xtask::workspace::Repository;

#[derive(Clone, Copy)]
enum ExitStatus {
    Success = 0,
    Violations = 1,
    ReportWriteFailure = 2,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        Self::from(status as u8)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repository = Repository::rooted_at(
        cli.repository_root
            .clone()
            .unwrap_or_else(locate_repository_root),
    );
    match run_task(&cli.task, &repository) {
        Ok(()) => ExitStatus::Success.into(),
        Err(error) => report_failure(&error.to_string()),
    }
}

fn locate_repository_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map_or_else(|| Path::new(".").to_path_buf(), Path::to_path_buf)
}

fn report_failure(message: &str) -> ExitCode {
    match write_report_line(message) {
        Ok(()) => ExitStatus::Violations.into(),
        Err(_) => ExitStatus::ReportWriteFailure.into(),
    }
}
