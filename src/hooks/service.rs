use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde_json::{json, Value};

use crate::hooks::templates::{
    CLAUDE_PRE_PROMPT_HOOK, CLAUDE_POST_SESSION_HOOK, CLAUDE_MD_SECTION,
    GEMINI_PRE_TOOL_HOOK, GEMINI_SESSION_END_HOOK, GEMINI_MD_CONTENT,
    CODEX_WRAPPER_SCRIPT, CODEX_INSTRUCTIONS,
    CURSOR_MDC_RULE, CURSOR_RULES_LEGACY,
};
use crate::config::get_agentmem_dir;

/// Supported agent types for hook installation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AgentType {
    ClaudeCode,
    GeminiCli,
    CodexCli,
    Cursor,
}

impl AgentType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace("-", "").replace("_", "").as_str() {
            "claudecode" | "claude" => Some(AgentType::ClaudeCode),
            "geminicli" | "gemini" => Some(AgentType::GeminiCli),
            "codexcli" | "codex" => Some(AgentType::CodexCli),
            "cursor" => Some(AgentType::Cursor),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "Claude Code",
            AgentType::GeminiCli => "Gemini CLI",
            AgentType::CodexCli => "Codex CLI",
            AgentType::Cursor => "Cursor",
        }
    }

    pub fn cli_name(&self) -> &'static str {
        match self {
            AgentType::ClaudeCode => "claude-code",
            AgentType::GeminiCli => "gemini-cli",
            AgentType::CodexCli => "codex-cli",
            AgentType::Cursor => "cursor",
        }
    }

    /// Check if this agent is installed on the system
    pub fn is_installed(&self) -> bool {
        match self {
            AgentType::ClaudeCode => {
                // Check for claude command or .claude directory
                Command::new("claude").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
                    || Path::new(".claude").exists()
                    || dirs::home_dir().map(|h| h.join(".claude").exists()).unwrap_or(false)
            }
            AgentType::GeminiCli => {
                // Check for gemini command
                Command::new("gemini").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
            }
            AgentType::CodexCli => {
                // Check for codex command
                Command::new("codex").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
            }
            AgentType::Cursor => {
                // Check for Cursor app or .cursor directory
                Path::new(".cursor").exists()
                    || Path::new("/Applications/Cursor.app").exists()
                    || dirs::home_dir().map(|h| h.join(".cursor").exists()).unwrap_or(false)
            }
        }
    }
}

/// Install hooks for the specified agent
pub fn install_hooks(agent: &str) -> Result<()> {
    let agent_type = AgentType::from_str(agent)
        .with_context(|| format!(
            "Unknown agent: '{}'. Supported: claude-code, gemini-cli, codex-cli, cursor",
            agent
        ))?;

    match agent_type {
        AgentType::ClaudeCode => install_claude_code_hooks()?,
        AgentType::GeminiCli => install_gemini_cli_hooks()?,
        AgentType::CodexCli => install_codex_cli_hooks()?,
        AgentType::Cursor => install_cursor_hooks()?,
    }

    Ok(())
}

// =============================================================================
// CLAUDE CODE INSTALLATION
// =============================================================================

fn install_claude_code_hooks() -> Result<()> {
    let hooks_dir = get_agentmem_dir().join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Get absolute path to project root for hook commands
    let project_root = std::env::current_dir()
        .context("Failed to get current directory")?;
    let abs_hooks_dir = project_root.join(".agentmem/hooks");

    // Write hook files
    let pre_prompt_path = hooks_dir.join("pre-prompt.js");
    fs::write(&pre_prompt_path, CLAUDE_PRE_PROMPT_HOOK)?;
    make_executable(&pre_prompt_path)?;

    let post_session_path = hooks_dir.join("post-session.js");
    fs::write(&post_session_path, CLAUDE_POST_SESSION_HOOK)?;
    make_executable(&post_session_path)?;

    // Configure .claude/settings.json with new matcher-based format
    let settings_dir = PathBuf::from(".claude");
    fs::create_dir_all(&settings_dir)?;

    let settings_path = settings_dir.join("settings.json");
    let mut settings: Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    // Build absolute paths for hook commands (works from any subdirectory)
    let pre_prompt_cmd = format!("node {}/pre-prompt.js", abs_hooks_dir.display());
    let post_session_cmd = format!("node {}/post-session.js", abs_hooks_dir.display());

    // Add hooks configuration with new matcher format
    let hooks = settings.as_object_mut().unwrap()
        .entry("hooks").or_insert_with(|| json!({}));
    let hooks_obj = hooks.as_object_mut().unwrap();

    // New format: {"UserPromptSubmit": [{"matcher": "*", "hooks": [{"type": "command", "command": "..."}]}]}
    hooks_obj.insert("UserPromptSubmit".to_string(), json!([
        {
            "matcher": "*",
            "hooks": [
                {
                    "type": "command",
                    "command": pre_prompt_cmd
                }
            ]
        }
    ]));
    hooks_obj.insert("SessionEnd".to_string(), json!([
        {
            "matcher": "*",
            "hooks": [
                {
                    "type": "command",
                    "command": post_session_cmd
                }
            ]
        }
    ]));

    fs::write(&settings_path, serde_json::to_string_pretty(&settings)?)?;

    // Update CLAUDE.md if exists
    update_context_file("CLAUDE.md", CLAUDE_MD_SECTION, "## AgentMem Integration")?;

    println!("Installed Claude Code hooks:");
    println!("  - .agentmem/hooks/pre-prompt.js");
    println!("  - .agentmem/hooks/post-session.js");
    println!("  - .claude/settings.json (updated)");

    Ok(())
}

// =============================================================================
// GEMINI CLI INSTALLATION
// =============================================================================

fn install_gemini_cli_hooks() -> Result<()> {
    let hooks_dir = get_agentmem_dir().join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Write hook files
    let pre_tool_path = hooks_dir.join("gemini-pre-tool.js");
    fs::write(&pre_tool_path, GEMINI_PRE_TOOL_HOOK)?;
    make_executable(&pre_tool_path)?;

    let session_end_path = hooks_dir.join("gemini-session-end.js");
    fs::write(&session_end_path, GEMINI_SESSION_END_HOOK)?;
    make_executable(&session_end_path)?;

    // Create GEMINI.md for context
    fs::write("GEMINI.md", GEMINI_MD_CONTENT)?;

    // Note: Gemini CLI hooks are not automatically configured in settings.json
    // because the format varies between versions. The user should manually add
    // the hooks if needed, or use the GEMINI.md file for context.

    println!("Installed Gemini CLI hooks:");
    println!("  - .agentmem/hooks/gemini-pre-tool.js");
    println!("  - .agentmem/hooks/gemini-session-end.js");
    println!("  - GEMINI.md (context file)");

    Ok(())
}

// =============================================================================
// CODEX CLI INSTALLATION
// =============================================================================

fn install_codex_cli_hooks() -> Result<()> {
    let hooks_dir = get_agentmem_dir().join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    // Write wrapper script
    let wrapper_path = hooks_dir.join("am-codex");
    fs::write(&wrapper_path, CODEX_WRAPPER_SCRIPT)?;
    make_executable(&wrapper_path)?;

    // Create AGENTS.md with instructions
    update_context_file("AGENTS.md", CODEX_INSTRUCTIONS, "# AgentMem Integration")?;

    // Note: Codex uses config.toml, which we could update, but the wrapper approach is simpler

    println!("Installed Codex CLI integration:");
    println!("  - .agentmem/hooks/am-codex (wrapper script)");
    println!("  - AGENTS.md (instructions)");
    println!("");
    println!("Usage: Use the wrapper script instead of codex directly:");
    println!("  .agentmem/hooks/am-codex \"your prompt\"");
    println!("");
    println!("Or add to PATH:");
    println!("  export PATH=\"$PWD/.agentmem/hooks:$PATH\"");
    println!("  am-codex \"your prompt\"");

    Ok(())
}

// =============================================================================
// CURSOR INSTALLATION
// =============================================================================

fn install_cursor_hooks() -> Result<()> {
    // Create .cursor/rules directory for MDC format
    let rules_dir = PathBuf::from(".cursor/rules");
    fs::create_dir_all(&rules_dir)?;

    // Write MDC rule file
    let mdc_path = rules_dir.join("agentmem.mdc");
    fs::write(&mdc_path, CURSOR_MDC_RULE)?;

    // Also create legacy .cursorrules for backward compatibility
    let cursorrules_path = PathBuf::from(".cursorrules");
    if !cursorrules_path.exists() {
        fs::write(&cursorrules_path, CURSOR_RULES_LEGACY)?;
        println!("  - .cursorrules (legacy format)");
    }

    println!("Installed Cursor integration:");
    println!("  - .cursor/rules/agentmem.mdc");

    Ok(())
}

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Make a file executable (Unix only)
fn make_executable(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms)?;
    }
    Ok(())
}

/// Update or create a context file with AgentMem section
fn update_context_file(filename: &str, content: &str, marker: &str) -> Result<()> {
    let path = Path::new(filename);

    if path.exists() {
        let existing = fs::read_to_string(path)?;
        if !existing.contains(marker) {
            let updated = format!("{}\n{}", existing.trim(), content);
            fs::write(path, updated)?;
            println!("  - {} (updated)", filename);
        }
    } else {
        fs::write(path, content)?;
        println!("  - {} (created)", filename);
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

            // Include .js files and executable scripts
            let is_hook = path.extension().map(|e| e == "js").unwrap_or(false)
                || path.file_name().map(|n| n.to_string_lossy().starts_with("am-")).unwrap_or(false);

            if is_hook {
                let name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                hooks.push(HookInfo {
                    name: name.clone(),
                    path: path.display().to_string(),
                    hook_type: detect_hook_type(&name),
                    agent: detect_hook_agent(&name),
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
    pub agent: String,
}

fn detect_hook_type(name: &str) -> String {
    if name.contains("pre-prompt") || name.contains("pre-tool") {
        "PrePrompt".to_string()
    } else if name.contains("post-session") || name.contains("session-end") {
        "SessionEnd".to_string()
    } else if name.starts_with("am-") {
        "Wrapper".to_string()
    } else {
        "Unknown".to_string()
    }
}

fn detect_hook_agent(name: &str) -> String {
    if name.contains("gemini") {
        "Gemini CLI".to_string()
    } else if name.contains("codex") || name.starts_with("am-codex") {
        "Codex CLI".to_string()
    } else {
        "Claude Code".to_string()
    }
}

/// Detect which agents are installed on the system
pub fn detect_installed_agents() -> Vec<AgentType> {
    let all_agents = [
        AgentType::ClaudeCode,
        AgentType::GeminiCli,
        AgentType::CodexCli,
        AgentType::Cursor,
    ];

    all_agents.into_iter().filter(|a| a.is_installed()).collect()
}

/// Test hook execution (dry run)
pub fn test_hook(hook_type: &str) -> Result<String> {
    match hook_type {
        "pre-prompt" => {
            let test_query = "test query";
            println!("Testing pre-prompt hook with query: '{}'", test_query);
            println!("Running: am context --query \"{}\" --json", test_query);

            let output = Command::new("am")
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
