use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};

use crate::hooks::templates::{PRE_PROMPT_HOOK, POST_SESSION_HOOK, CLAUDE_MD_SECTION};
use crate::config::get_agentmem_dir;

/// Supported agent types for hook installation
#[derive(Debug, Clone, Copy)]
pub enum AgentType {
    ClaudeCode,
    Cursor,
}

impl AgentType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "claude-code" | "claudecode" | "claude" => Some(AgentType::ClaudeCode),
            "cursor" => Some(AgentType::Cursor),
            _ => None,
        }
    }

    pub fn settings_dir(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => ".claude",
            AgentType::Cursor => ".cursor",
        }
    }

    pub fn settings_file(&self) -> &'static str {
        "settings.json"
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::Cursor => "Cursor",
        }
    }
}

/// Install hooks for the specified agent
pub fn install_hooks(agent: &str) -> Result<()> {
    let agent_type = AgentType::from_str(agent)
        .with_context(|| format!("Unknown agent: '{}'. Supported: claude-code, cursor", agent))?;

    // 1. Generate hook files
    generate_hook_files()?;

    // 2. Configure agent settings
    configure_agent_settings(agent_type)?;

    // 3. Update CLAUDE.md if it exists (for claude-code)
    if matches!(agent_type, AgentType::ClaudeCode) {
        update_claude_md()?;
    }

    println!("Installed {} hooks:", agent_type.display_name());
    println!("  - .agentmem/hooks/pre-prompt.js");
    println!("  - .agentmem/hooks/post-session.js");
    println!("  - {}/settings.json (updated)", agent_type.settings_dir());

    Ok(())
}

/// Generate JavaScript hook files in .agentmem/hooks/
fn generate_hook_files() -> Result<()> {
    let hooks_dir = get_agentmem_dir().join("hooks");

    // Ensure hooks directory exists
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;
    }

    // Write pre-prompt hook
    let pre_prompt_path = hooks_dir.join("pre-prompt.js");
    fs::write(&pre_prompt_path, PRE_PROMPT_HOOK)
        .context("Failed to write pre-prompt.js")?;

    // Write post-session hook
    let post_session_path = hooks_dir.join("post-session.js");
    fs::write(&post_session_path, POST_SESSION_HOOK)
        .context("Failed to write post-session.js")?;

    Ok(())
}

/// Configure agent-specific settings file
fn configure_agent_settings(agent_type: AgentType) -> Result<()> {
    let settings_dir = PathBuf::from(agent_type.settings_dir());
    let settings_path = settings_dir.join(agent_type.settings_file());

    // Create settings directory if needed
    if !settings_dir.exists() {
        fs::create_dir_all(&settings_dir)
            .context(format!("Failed to create {} directory", agent_type.settings_dir()))?;
    }

    // Read existing settings or create new
    let mut settings: Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)
            .context("Failed to read existing settings")?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Add/update hooks configuration
    let hooks = settings.as_object_mut()
        .unwrap()
        .entry("hooks")
        .or_insert_with(|| json!({}));

    let hooks_obj = hooks.as_object_mut().unwrap();

    // Set UserPromptSubmit hook
    hooks_obj.insert(
        "UserPromptSubmit".to_string(),
        json!([".agentmem/hooks/pre-prompt.js"])
    );

    // Set SessionEnd hook (for post-session)
    hooks_obj.insert(
        "SessionEnd".to_string(),
        json!([".agentmem/hooks/post-session.js"])
    );

    // Write updated settings
    let formatted = serde_json::to_string_pretty(&settings)?;
    fs::write(&settings_path, formatted)
        .context("Failed to write settings file")?;

    Ok(())
}

/// Update CLAUDE.md with AgentMem section if not already present
fn update_claude_md() -> Result<()> {
    let claude_md_path = Path::new("CLAUDE.md");

    if claude_md_path.exists() {
        let content = fs::read_to_string(claude_md_path)
            .context("Failed to read CLAUDE.md")?;

        // Check if AgentMem section already exists
        if !content.contains("## AgentMem Integration") {
            let updated = format!("{}\n{}", content.trim(), CLAUDE_MD_SECTION);
            fs::write(claude_md_path, updated)
                .context("Failed to update CLAUDE.md")?;
            println!("  - CLAUDE.md (updated with AgentMem section)");
        }
    }

    Ok(())
}

/// List installed hooks
pub fn list_hooks() -> Result<Vec<HookInfo>> {
    let hooks_dir = get_agentmem_dir().join("hooks");
    let mut hooks = Vec::new();

    if hooks_dir.exists() {
        for entry in fs::read_dir(&hooks_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "js").unwrap_or(false) {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                hooks.push(HookInfo {
                    name,
                    path: path.display().to_string(),
                    hook_type: detect_hook_type(&path),
                });
            }
        }
    }

    Ok(hooks)
}

#[derive(Debug, serde::Serialize)]
pub struct HookInfo {
    pub name: String,
    pub path: String,
    pub hook_type: String,
}

fn detect_hook_type(path: &Path) -> String {
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match name {
        "pre-prompt" => "UserPromptSubmit".to_string(),
        "post-session" => "SessionEnd".to_string(),
        "post-compact" => "PostCompact".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Test hook execution (dry run)
pub fn test_hook(hook_type: &str) -> Result<String> {
    match hook_type {
        "pre-prompt" => {
            // Simulate what the hook would return
            let test_query = "test query";
            println!("Testing pre-prompt hook with query: '{}'", test_query);
            println!("Running: am context --query \"{}\" --json", test_query);

            // Actually run the context command
            let output = std::process::Command::new("am")
                .args(["context", "--query", test_query, "--json"])
                .output()
                .context("Failed to run am context")?;

            let stdout = String::from_utf8_lossy(&output.stdout);
            Ok(stdout.to_string())
        }
        "post-session" => {
            println!("Testing post-session hook (would run am sync)");
            Ok("Post-session hook would run 'am sync' asynchronously".to_string())
        }
        _ => {
            anyhow::bail!("Unknown hook type: '{}'. Supported: pre-prompt, post-session", hook_type)
        }
    }
}
