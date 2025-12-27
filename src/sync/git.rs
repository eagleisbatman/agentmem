use std::process::Command;
use anyhow::{Result, Context};

pub fn git_sync(push: bool, message: Option<&str>) -> Result<()> {
    let msg = message.unwrap_or("agentmem: sync");

    // git add .agentmem/agentmem.jsonl
    let status = Command::new("git")
        .args(&["add", ".agentmem/agentmem.jsonl"])
        .status()
        .context("Failed to run git add")?;

    if !status.success() {
        // Not a git repo or other error, but we'll just log it for now
        return Ok(());
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

    Ok(())
}

