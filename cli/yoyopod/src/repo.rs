//! Repository root discovery.
//!
//! The CLI is expected to run from inside a checkout of the yoyopod-core
//! repository. We walk up from the current working directory looking for
//! `.git/` (the canonical marker). The Python CLI used the package's
//! `__file__` parent; the Rust CLI doesn't have that luxury because it
//! may be installed to `~/.cargo/bin/yoyopod` outside the repo.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

/// Find the repository root by walking up from `start` until `.git` is found.
///
/// Returns an error if no enclosing repository is found.
pub fn find_repo_root(start: &Path) -> Result<PathBuf> {
    let mut current = start
        .canonicalize()
        .with_context(|| format!("canonicalize {}", start.display()))?;

    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                return Err(anyhow!(
                    "could not find a yoyopod-core repository (no .git/ found above {})",
                    start.display()
                ));
            }
        }
    }
}

/// Convenience: find the repo root starting from the current working directory.
pub fn current_repo_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("read current working directory")?;
    find_repo_root(&cwd)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn finds_root_at_self() {
        let tmp = tempdir().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        let found = find_repo_root(tmp.path()).unwrap();
        assert_eq!(found, tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn finds_root_from_subdir() {
        let tmp = tempdir().unwrap();
        fs::create_dir(tmp.path().join(".git")).unwrap();
        let sub = tmp.path().join("a/b/c");
        fs::create_dir_all(&sub).unwrap();
        let found = find_repo_root(&sub).unwrap();
        assert_eq!(found, tmp.path().canonicalize().unwrap());
    }

    #[test]
    fn errors_when_no_git_anywhere() {
        // Skip on hosts where the tempdir parents already contain a `.git/`
        // directory (e.g. CI runners, dev machines with a checkout under /tmp).
        // The behaviour we care about is: walking up to / without finding
        // `.git` yields an error. Anything else is environment-specific.
        let tmp = tempdir().unwrap();
        let mut cursor = tmp.path().canonicalize().unwrap();
        loop {
            if cursor.join(".git").exists() {
                eprintln!(
                    "skipping errors_when_no_git_anywhere: ancestor {} has .git",
                    cursor.display()
                );
                return;
            }
            match cursor.parent() {
                Some(parent) => cursor = parent.to_path_buf(),
                None => break,
            }
        }
        let err = find_repo_root(tmp.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("could not find"));
    }
}
