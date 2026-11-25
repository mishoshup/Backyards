use std::io::Read;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};

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

pub fn capture_command_limited(
    directory: &Path,
    program: &str,
    arguments: &[&str],
    limit: usize,
) -> Result<String, GateError> {
    let description = describe_command(program, arguments);
    let mut child = spawn_with_piped_output(directory, program, arguments, &description)?;
    let captured = read_at_most(&mut child, limit + 1, &description)?;
    if captured.len() > limit {
        discard_child(child, &description)?;
        return Ok(String::from_utf8_lossy(&captured).into_owned());
    }
    let output = wait_for_output(child, &description)?;
    ensure_accepted_exit(&output, &[0], description)?;
    Ok(String::from_utf8_lossy(&captured).into_owned())
}

fn spawn_with_piped_output(
    directory: &Path,
    program: &str,
    arguments: &[&str],
    description: &str,
) -> Result<Child, GateError> {
    Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| GateError::RunCommand {
            command: description.to_owned(),
            source,
        })
}

fn read_at_most(child: &mut Child, limit: usize, description: &str) -> Result<Vec<u8>, GateError> {
    let mut captured = Vec::new();
    if let Some(stdout) = child.stdout.take() {
        stdout
            .take(limit as u64)
            .read_to_end(&mut captured)
            .map_err(|source| GateError::RunCommand {
                command: description.to_owned(),
                source,
            })?;
    }
    Ok(captured)
}

fn discard_child(mut child: Child, description: &str) -> Result<(), GateError> {
    child
        .kill()
        .and_then(|()| child.wait().map(|_| ()))
        .map_err(|source| GateError::RunCommand {
            command: description.to_owned(),
            source,
        })
}

fn wait_for_output(child: Child, description: &str) -> Result<Output, GateError> {
    child
        .wait_with_output()
        .map_err(|source| GateError::RunCommand {
            command: description.to_owned(),
            source,
        })
}
