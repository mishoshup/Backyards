use std::path::Path;
use std::process::ExitCode;

use clap::Parser;
use xtask::cli::{Cli, run_task};
use xtask::finding::write_report_line;
use xtask::repository::Repository;

const REPORT_WRITE_FAILURE_STATUS: u8 = 2;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let repository = Repository::rooted_at(
        cli.repository_root
            .clone()
            .unwrap_or_else(locate_repository_root),
    );
    match run_task(&cli.task, &repository) {
        Ok(()) => ExitCode::SUCCESS,
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
        Ok(()) => ExitCode::FAILURE,
        Err(_) => ExitCode::from(REPORT_WRITE_FAILURE_STATUS),
    }
}
