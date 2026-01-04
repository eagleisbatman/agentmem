use std::process::Command;
use anyhow::{Result, Context};

/// Result of git sync operation
pub enum GitSyncResult {
    Synced,
    SyncedWithPull,
    NoChanges,
    NotAGitRepo,
    PullConflict,
}

/// Check if we're in a git repo
pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Pull remote changes (returns true if pulled successfully or no remote)
pub fn git_pull() -> Result<bool> {
    // Check if there's a remote configured
    let remote_check = Command::new("git")
        .args(&["remote"])
        .output()
        .context("Failed to check git remote")?;

    if remote_check.stdout.is_empty() {
        return Ok(true); // No remote, nothing to pull
    }

    // Try to pull
    let status = Command::new("git")
        .args(&["pull", "--rebase", "--autostash"])
        .status()
        .context("Failed to run git pull")?;

    Ok(status.success())
}

/// Check if JSONL file was updated by pull (for triggering import)
pub fn jsonl_needs_import() -> bool {
    // Check if the file was changed in the last pull
    let output = Command::new("git")
        .args(&["diff", "HEAD@{1}", "--name-only", "--", ".agentmem/agentmem.jsonl"])
        .output();

    match output {
        Ok(o) => !o.stdout.is_empty(),
        Err(_) => false,
    }
}

pub fn git_sync(push: bool, message: Option<&str>) -> Result<GitSyncResult> {
    let msg = message.unwrap_or("agentmem: sync");

    // Check if we're in a git repo first
    if !is_git_repo() {
        return Ok(GitSyncResult::NotAGitRepo);
    }

    // Pull first if pushing (to avoid conflicts)
    let mut pulled = false;
    if push {
        if !git_pull()? {
            return Ok(GitSyncResult::PullConflict);
        }
        pulled = jsonl_needs_import();
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
    } else if !pulled {
        // No changes to commit and didn't pull
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

    if pulled {
        Ok(GitSyncResult::SyncedWithPull)
    } else {
        Ok(GitSyncResult::Synced)
    }
}

