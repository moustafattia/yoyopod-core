//! `yoyopod target config edit`

use std::path::PathBuf;
use std::process::Command;

use anyhow::{anyhow, Context, Result};

use crate::cli::ConfigCommand;
use crate::repo::current_repo_root;

pub fn run(cmd: &ConfigCommand) -> Result<i32> {
    match cmd {
        ConfigCommand::Edit => edit_local_yaml(),
    }
}

fn edit_local_yaml() -> Result<i32> {
    let repo_root = current_repo_root()?;
    let path: PathBuf = repo_root.join("deploy/pi-deploy.local.yaml");
    if !path.exists() {
        // Pre-populate with a minimal template the user can fill in.
        std::fs::write(
            &path,
            "# Per-machine override for deploy/pi-deploy.yaml.\n\
             # Edit values below to point yoyopod target at your Pi.\n\
             host: \"\"\n\
             user: \"\"\n\
             # branch: main\n",
        )
        .with_context(|| format!("create {}", path.display()))?;
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("spawn editor: {editor}"))?;
    if !status.success() {
        return Err(anyhow!(
            "{editor} exited with {} while editing {}",
            status.code().unwrap_or(-1),
            path.display()
        ));
    }
    println!("edited {}", path.display());
    Ok(0)
}
