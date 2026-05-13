//! Local subprocess helpers.

use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

pub struct CapturedOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

/// Execute a local command, streaming output.
pub fn run_local<I, S>(command: I, label: &str) -> Result<i32>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let argv: Vec<_> = command.into_iter().collect();
    let (first, rest) = argv
        .split_first()
        .context("run_local: empty command argv")?;
    let pretty = argv
        .iter()
        .map(|s| s.as_ref().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(" ");
    eprintln!();
    eprintln!("[yoyopod-target] local={label}");
    eprintln!("[yoyopod-target] cmd={pretty}");
    eprintln!();
    let status = Command::new(first.as_ref())
        .args(rest.iter().map(AsRef::as_ref))
        .status()
        .with_context(|| format!("spawn {pretty}"))?;
    Ok(status.code().unwrap_or(1))
}

/// Execute a local command, capturing output.
pub fn run_local_capture<I, S>(command: I) -> Result<CapturedOutput>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let argv: Vec<_> = command.into_iter().collect();
    let (first, rest) = argv
        .split_first()
        .context("run_local_capture: empty command argv")?;
    let output = Command::new(first.as_ref())
        .args(rest.iter().map(AsRef::as_ref))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("spawn local command")?;
    Ok(CapturedOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}
