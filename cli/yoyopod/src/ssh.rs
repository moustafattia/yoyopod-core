//! SSH transport: build SSH commands and run them.
//!
//! Mirrors `remote_transport.py`. All SSH calls wrap the remote command
//! with `bash -lc '<cmd>'` so login-profile shells are used on the Pi.
//! The optional `workdir` is `cd`'d into before the remote command runs;
//! pass `None` to skip the `cd`.

use std::ffi::OsString;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

use crate::deploy_config::RemoteConnection;
use crate::quoting::{quote_remote_project_dir, shell_quote};

/// One captured shell result.
pub struct CapturedOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl CapturedOutput {
    pub fn success(&self) -> bool {
        self.status.success()
    }
}

/// Selector for the working directory used on the Pi before running the command.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // Explicit() is unused in Round 1 but kept for future SSH callers.
pub enum RemoteWorkdir<'a> {
    /// Use the connection's `project_dir`.
    Default,
    /// Use this explicit path.
    Explicit(&'a str),
    /// Run directly without `cd`.
    None,
}

fn wrap_command(remote_command: &str, workdir: Option<&str>) -> String {
    match workdir {
        Some(dir) => {
            let quoted = quote_remote_project_dir(dir);
            format!("cd {quoted} && {remote_command}")
        }
        None => remote_command.to_string(),
    }
}

/// Build the `ssh ...` argv used to execute a command on the Pi.
pub fn build_ssh_command(
    conn: &RemoteConnection,
    remote_command: &str,
    tty: bool,
    workdir: RemoteWorkdir<'_>,
) -> Vec<OsString> {
    let resolved = match workdir {
        RemoteWorkdir::Default => Some(conn.project_dir.as_str()),
        RemoteWorkdir::Explicit(s) => Some(s),
        RemoteWorkdir::None => None,
    };
    let wrapped = wrap_command(remote_command, resolved);
    let mut argv: Vec<OsString> = vec!["ssh".into()];
    if tty {
        argv.push("-t".into());
    }
    argv.push(conn.ssh_target().into());
    argv.push(format!("bash -lc {}", shell_quote(&wrapped)).into());
    argv
}

/// Execute a command on the Pi via SSH, streaming its output.
/// Returns the exit code.
pub fn run_remote(
    conn: &RemoteConnection,
    remote_command: &str,
    tty: bool,
    workdir: RemoteWorkdir<'_>,
) -> Result<i32> {
    let argv = build_ssh_command(conn, remote_command, tty, workdir);
    let (first, rest) = argv.split_first().expect("argv has at least 1 element");
    let resolved_workdir = match &workdir {
        RemoteWorkdir::Default => Some(conn.project_dir.as_str()),
        RemoteWorkdir::Explicit(s) => Some(*s),
        RemoteWorkdir::None => None,
    };
    eprintln!();
    eprintln!("[yoyopod-target] host={}", conn.ssh_target());
    eprintln!(
        "[yoyopod-target] dir={}",
        resolved_workdir.unwrap_or("(direct)")
    );
    eprintln!("[yoyopod-target] cmd={remote_command}");
    eprintln!();
    let status = Command::new(first)
        .args(rest)
        .status()
        .with_context(|| format!("spawn ssh: {first:?}"))?;
    Ok(status.code().unwrap_or(1))
}

/// Execute an SSH command and capture stdout/stderr.
pub fn run_remote_capture(
    conn: &RemoteConnection,
    remote_command: &str,
    workdir: RemoteWorkdir<'_>,
) -> Result<CapturedOutput> {
    let argv = build_ssh_command(conn, remote_command, false, workdir);
    let (first, rest) = argv.split_first().expect("argv has at least 1 element");
    let output = Command::new(first)
        .args(rest)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .with_context(|| format!("spawn ssh: {first:?}"))?;
    Ok(CapturedOutput {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// Validate that the connection has a host before we attempt SSH.
pub fn validate_connection(conn: &RemoteConnection) -> Result<()> {
    if conn.host.is_empty() {
        anyhow::bail!(
            "missing target host: set it with `yoyopod target config edit`, \
             pass --host, or set YOYOPOD_PI_HOST"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> RemoteConnection {
        RemoteConnection {
            host: "pi.local".to_string(),
            user: "pi".to_string(),
            project_dir: "/opt/yoyopod-dev/checkout".to_string(),
            branch: "main".to_string(),
        }
    }

    fn argv_strs(argv: &[OsString]) -> Vec<String> {
        argv.iter()
            .map(|s| s.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn ssh_command_default_workdir() {
        let c = conn();
        let argv = build_ssh_command(&c, "echo hi", false, RemoteWorkdir::Default);
        let s = argv_strs(&argv);
        assert_eq!(s[0], "ssh");
        assert_eq!(s[1], "pi@pi.local");
        // Last arg is the bash -lc invocation with cd
        assert!(s[2].starts_with("bash -lc "));
        assert!(s[2].contains("cd /opt/yoyopod-dev/checkout && echo hi"));
    }

    #[test]
    fn ssh_command_no_workdir() {
        let c = conn();
        let argv = build_ssh_command(&c, "uptime", false, RemoteWorkdir::None);
        let s = argv_strs(&argv);
        assert!(!s[2].contains("cd "));
        assert!(s[2].contains("uptime"));
    }

    #[test]
    fn ssh_command_tty_flag() {
        let c = conn();
        let argv = build_ssh_command(&c, "echo", true, RemoteWorkdir::Default);
        let s = argv_strs(&argv);
        assert_eq!(s[0], "ssh");
        assert_eq!(s[1], "-t");
        assert_eq!(s[2], "pi@pi.local");
    }

    #[test]
    fn ssh_target_without_user() {
        let mut c = conn();
        c.user = String::new();
        assert_eq!(c.ssh_target(), "pi.local");
    }

    #[test]
    fn validate_rejects_empty_host() {
        let mut c = conn();
        c.host = String::new();
        let err = validate_connection(&c).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("missing target host"));
    }
}
