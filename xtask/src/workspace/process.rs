use std::path::Path;
use std::process::{Command, Output};

use crate::report::GateError;
use crate::report::write_report_line;

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
            stderr: "see the output above".to_owned(),
        })
    }
}

fn describe_command(program: &str, arguments: &[&str]) -> String {
    std::iter::once(program)
        .chain(arguments.iter().copied())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn capture_command(
    directory: &Path,
    program: &str,
    arguments: &[&str],
    accepted_exit_codes: &[i32],
) -> Result<String, GateError> {
    let description = describe_command(program, arguments);
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .output()
        .map_err(|source| GateError::RunCommand {
            command: description.clone(),
            source,
        })?;
    ensure_accepted_exit(&output, accepted_exit_codes, description)?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn ensure_accepted_exit(
    output: &Output,
    accepted_exit_codes: &[i32],
    description: String,
) -> Result<(), GateError> {
    if output
        .status
        .code()
        .is_some_and(|code| accepted_exit_codes.contains(&code))
    {
        return Ok(());
    }
    Err(GateError::CommandFailed {
        command: description,
        status: output.status,
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    })
}
