use std::path::Path;
use std::process::Command;

use crate::error::GateError;
use crate::finding::write_report_line;

pub fn run_command(directory: &Path, program: &str, arguments: &[&str]) -> Result<(), GateError> {
    let description = describe_command(program, arguments);
    write_report_line(&format!("$ {description}"))?;
    let status = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .status()
        .map_err(|source| GateError::RunCommand {
            command: description.clone(),
            source,
        })?;
    if status.success() {
        Ok(())
    } else {
        Err(GateError::CommandFailed {
            command: description,
            status,
        })
    }
}

fn describe_command(program: &str, arguments: &[&str]) -> String {
    std::iter::once(program)
        .chain(arguments.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}
