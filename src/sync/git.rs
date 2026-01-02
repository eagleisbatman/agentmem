use std::process::Command;
use anyhow::{Result, Context};

/// Result of git sync operation
pub enum GitSyncResult {
    Synced,
    NoChanges,
    NotAGitRepo,
}

pub fn git_sync(push: bool, message: Option<&str>) -> Result<GitSyncResult> {
    let msg = message.unwrap_or("agentmem: sync");

    // Check if we're in a git repo first
    let check_git = Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .context("Failed to run git")?;

    if !check_git.status.success() {
        return Ok(GitSyncResult::NotAGitRepo);
    }

    // git add .agentmem/agentmem.jsonl (use -f to add even if gitignored)
    let status = Command::new("git")
        .args(&["add", "-f", ".agentmem/agentmem.jsonl"])
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        anyhow::bail!("git add failed");
    }

    // git commit -m "..."
    // We check if there are changes to commit first
    let output = Command::new("git")
        .args(&["diff", "--cached", "--quiet"])
        .status()
        .context("Failed to run git diff")?;

    if !output.success() {
        // Changes exist, so commit
        let status = Command::new("git")
            .args(&["commit", "-m", msg])
            .status()
            .context("Failed to run git commit")?;

        if !status.success() {
            anyhow::bail!("Git commit failed");
        }
    } else {
        // No changes to commit
        return Ok(GitSyncResult::NoChanges);
    }

    if push {
        let status = Command::new("git")
            .args(&["push"])
            .status()
            .context("Failed to run git push")?;

        if !status.success() {
            anyhow::bail!("Git push failed");
        }
    }

    Ok(GitSyncResult::Synced)
}

