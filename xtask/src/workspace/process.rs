use std::io::Read;
use std::path::Path;
use std::process::{Child, ChildStderr, Command, Output, Stdio};
use std::thread::JoinHandle;

const STDERR_CAPTURE_LIMIT: u64 = 65_536;

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
    let stderr_drain = start_stderr_drain(&mut child);
    let captured = match read_or_discard(&mut child, limit + 1, &description) {
        Ok(captured) => captured,
        Err(error) => {
            join_stderr_drain(stderr_drain);
            return Err(error);
        }
    };
    if captured.len() > limit {
        discard_child(child, stderr_drain, &description)?;
    } else {
        finish_child(child, stderr_drain, description)?;
    }
    Ok(String::from_utf8_lossy(&captured).into_owned())
}

fn read_or_discard(
    child: &mut Child,
    limit: usize,
    description: &str,
) -> Result<Vec<u8>, GateError> {
    match read_at_most(child, limit, description) {
        Ok(captured) => Ok(captured),
        Err(error) => {
            kill_child(child, description)?;
            Err(error)
        }
    }
}

fn finish_child(
    mut child: Child,
    stderr_drain: Option<JoinHandle<Vec<u8>>>,
    description: String,
) -> Result<(), GateError> {
    let status = child.wait().map_err(|source| GateError::RunCommand {
        command: description.clone(),
        source,
    })?;
    let stderr = stderr_drain
        .and_then(|drain| drain.join().ok())
        .unwrap_or_default();
    ensure_accepted_exit(
        &Output {
            status,
            stdout: Vec::new(),
            stderr,
        },
        &[0],
        description,
    )
}

fn start_stderr_drain(child: &mut Child) -> Option<JoinHandle<Vec<u8>>> {
    let stderr = child.stderr.take()?;
    Some(std::thread::spawn(move || read_then_discard(stderr)))
}

fn read_then_discard(mut stderr: ChildStderr) -> Vec<u8> {
    let mut collected = Vec::new();
    if let Err(error) = drain_stderr(&mut stderr, &mut collected) {
        collected.extend_from_slice(format!("[stderr read failed: {error}]").as_bytes());
    }
    collected
}

fn drain_stderr(stderr: &mut ChildStderr, collected: &mut Vec<u8>) -> std::io::Result<()> {
    Read::take(&mut *stderr, STDERR_CAPTURE_LIMIT).read_to_end(collected)?;
    std::io::copy(stderr, &mut std::io::sink())?;
    Ok(())
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

fn discard_child(
    mut child: Child,
    stderr_drain: Option<JoinHandle<Vec<u8>>>,
    description: &str,
) -> Result<(), GateError> {
    let killed = kill_child(&mut child, description);
    join_stderr_drain(stderr_drain);
    killed
}

fn join_stderr_drain(stderr_drain: Option<JoinHandle<Vec<u8>>>) {
    if let Some(drain) = stderr_drain {
        drop(drain.join());
    }
}

fn kill_child(child: &mut Child, description: &str) -> Result<(), GateError> {
    child
        .kill()
        .and_then(|()| child.wait().map(|_| ()))
        .map_err(|source| GateError::RunCommand {
            command: description.to_owned(),
            source,
        })
}
