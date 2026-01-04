use std::fs;
use std::io::{self, Write};
use std::process::Command;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use crate::config::{Config, save_config, get_project_id, get_local_agentmem_dir};
use crate::db::get_connection;

/// Global credentials directory
fn get_global_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agentmem")
}

/// Claude Code plugins directory
fn get_claude_plugins_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("plugins")
}

/// Check if AgentMem plugin is installed in Claude Code
fn is_plugin_installed() -> bool {
    get_claude_plugins_dir().join("agentmem").exists()
}

/// Install AgentMem plugin to Claude Code plugins directory
fn install_plugin() -> Result<()> {
    let plugin_dest = get_claude_plugins_dir().join("agentmem");

    // Find plugin source - could be in current dir or relative to binary
    let plugin_source = find_plugin_source()?;

    // Create plugins directory if needed
    fs::create_dir_all(get_claude_plugins_dir())?;

    // Copy plugin directory
    copy_dir_recursive(&plugin_source, &plugin_dest)?;

    // Also install skills to ~/.claude/skills/
    install_skills(&plugin_source)?;

    Ok(())
}

/// Get Claude skills directory
fn get_claude_skills_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("skills")
}

/// Install AgentMem skills to user skills directory
fn install_skills(plugin_source: &Path) -> Result<()> {
    let skills_dir = get_claude_skills_dir();
    fs::create_dir_all(&skills_dir)?;

    // Install memory-persistence skill
    let memory_skill_src = plugin_source.join("skills/memory-persistence/SKILL.md");
    if memory_skill_src.exists() {
        let memory_skill_dir = skills_dir.join("agentmem-memory");
        fs::create_dir_all(&memory_skill_dir)?;

        // Read, modify name, and write
        let content = fs::read_to_string(&memory_skill_src)?;
        let modified = content.replace("name: memory-persistence", "name: agentmem-memory");
        fs::write(memory_skill_dir.join("SKILL.md"), modified)?;
    }

    // Install plan-to-tasks skill
    let plan_skill_src = plugin_source.join("skills/plan-to-tasks/SKILL.md");
    if plan_skill_src.exists() {
        let plan_skill_dir = skills_dir.join("agentmem-plan");
        fs::create_dir_all(&plan_skill_dir)?;

        // Read, modify name, and write
        let content = fs::read_to_string(&plan_skill_src)?;
        let modified = content.replace("name: plan-to-tasks", "name: agentmem-plan");
        fs::write(plan_skill_dir.join("SKILL.md"), modified)?;
    }

    Ok(())
}

/// Find the plugin source directory
fn find_plugin_source() -> Result<PathBuf> {
    // Check relative to current directory (for development)
    let dev_path = PathBuf::from("plugin");
    if dev_path.exists() && dev_path.join(".claude-plugin").exists() {
        return Ok(dev_path);
    }

    // Check relative to binary location (for installed version)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let installed_path = exe_dir.join("plugin");
            if installed_path.exists() {
                return Ok(installed_path);
            }
            // Also check ../share/agentmem/plugin for system installs
            let share_path = exe_dir.join("../share/agentmem/plugin");
            if share_path.exists() {
                return Ok(share_path);
            }
        }
    }

    anyhow::bail!("Plugin source not found. Run from AgentMem directory or ensure plugin is installed.")
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

/// Global credentials file path
fn get_credentials_path() -> PathBuf {
    get_global_config_dir().join("credentials")
}

/// Check if Docker is installed
fn check_docker_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Docker daemon is running
fn check_docker_running() -> bool {
    Command::new("docker")
        .arg("info")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if Qdrant container exists
fn check_qdrant_container_exists() -> bool {
    Command::new("docker")
        .args(["ps", "-a", "--format", "{{.Names}}"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|name| name == "agentmem-qdrant")
        })
        .unwrap_or(false)
}

/// Check if Qdrant container is running
fn check_qdrant_running() -> bool {
    Command::new("docker")
        .args(["ps", "--format", "{{.Names}}"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|name| name == "agentmem-qdrant")
        })
        .unwrap_or(false)
}

/// Start existing Qdrant container
fn start_qdrant_container() -> Result<()> {
    Command::new("docker")
        .args(["start", "agentmem-qdrant"])
        .output()
        .context("Failed to start Qdrant container")?;
    Ok(())
}

/// Create and start new Qdrant container
fn create_qdrant_container() -> Result<()> {
    let output = Command::new("docker")
        .args([
            "run", "-d",
            "--name", "agentmem-qdrant",
            "-p", "6333:6333",
            "-p", "6334:6334",
            "-v", "agentmem-qdrant-data:/qdrant/storage",
            "qdrant/qdrant:latest"
        ])
        .output()
        .context("Failed to create Qdrant container")?;

    if !output.status.success() {
        anyhow::bail!("Failed to create Qdrant container: {}",
            String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Wait for Qdrant to be healthy
fn wait_for_qdrant(timeout_secs: u64) -> bool {
    for _ in 0..timeout_secs {
        if let Ok(output) = Command::new("curl")
            .args(["-s", "http://localhost:6333/health"])
            .output()
        {
            if output.status.success() {
                return true;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    false
}

/// Check if OpenAI API key is available
fn get_openai_key() -> Option<String> {
    // 1. Check environment variable
    if let Ok(key) = std::env::var("OPENAI_API_KEY") {
        if !key.is_empty() {
            return Some(key);
        }
    }

    // 2. Check credentials file
    let creds_path = get_credentials_path();
    if creds_path.exists() {
        if let Ok(content) = fs::read_to_string(&creds_path) {
            for line in content.lines() {
                if let Some(key) = line.strip_prefix("OPENAI_API_KEY=") {
                    if !key.is_empty() {
                        return Some(key.to_string());
                    }
                }
            }
        }
    }

    None
}

/// Prompt user for OpenAI API key
fn prompt_openai_key() -> Result<Option<String>> {
    println!();
    println!("AgentMem uses OpenAI for embeddings and memory extraction.");
    println!("Get your API key from: https://platform.openai.com/api-keys");
    println!();
    print!("Enter your OpenAI API key (or press Enter to skip): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let key = input.trim();

    if key.is_empty() {
        return Ok(None);
    }

    // Validate key format
    if !key.starts_with("sk-") {
        println!("Warning: API key doesn't start with 'sk-'. Saving anyway.");
    }

    // Save to credentials file
    let global_dir = get_global_config_dir();
    fs::create_dir_all(&global_dir)?;

    let creds_path = get_credentials_path();
    fs::write(&creds_path, format!("OPENAI_API_KEY={}\n", key))?;

    // Set restrictive permissions (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&creds_path, fs::Permissions::from_mode(0o600))?;
    }

    // Also set environment variable for current session
    std::env::set_var("OPENAI_API_KEY", key);

    Ok(Some(key.to_string()))
}

/// Interactive prompt for yes/no
fn prompt_yes_no(question: &str, default: bool) -> bool {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", question, suffix);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return default;
    }

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        "" => default,
        _ => default,
    }
}

pub fn run_init(quiet: bool, embedding: Option<String>, model: Option<String>) -> Result<()> {
    // Always use local directory for init, not discovered parent
    let am_dir = get_local_agentmem_dir();

    if !quiet {
        println!();
        println!("Initializing AgentMem...");
        println!();
    }

    // Step 1: Create .agentmem directory
    if am_dir.exists() {
        if !quiet {
            println!("  .agentmem directory already exists. Re-initializing...");
        }
    } else {
        fs::create_dir_all(&am_dir).context("Failed to create .agentmem directory")?;
        if !quiet {
            println!("  {} Created .agentmem/", "✓");
        }
    }

    // Create hooks directory
    let hooks_dir = am_dir.join("hooks");
    if !hooks_dir.exists() {
        fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;
    }

    // Create .gitignore
    let gitignore_path = am_dir.join(".gitignore");
    if !gitignore_path.exists() {
        fs::write(&gitignore_path, "*.db\nhooks/\n").context("Failed to create .gitignore")?;
    }

    // Step 2: Check Docker and Qdrant
    if !quiet {
        println!();
        println!("Checking dependencies...");
    }

    let docker_installed = check_docker_installed();
    let docker_running = docker_installed && check_docker_running();

    if !quiet {
        if docker_installed {
            println!("  {} Docker installed", "✓");
            if docker_running {
                println!("  {} Docker daemon running", "✓");
            } else {
                println!("  {} Docker daemon not running", "!");
            }
        } else {
            println!("  {} Docker not installed (required for semantic search)", "!");
        }
    }

    // Handle Qdrant setup
    let mut qdrant_running = false;
    if docker_running {
        if check_qdrant_running() {
            qdrant_running = true;
            if !quiet {
                println!("  {} Qdrant container running", "✓");
            }
        } else if check_qdrant_container_exists() {
            if !quiet {
                println!("  Starting existing Qdrant container...");
            }
            start_qdrant_container()?;
            if wait_for_qdrant(10) {
                qdrant_running = true;
                if !quiet {
                    println!("  {} Qdrant started", "✓");
                }
            }
        } else if !quiet {
            // Ask to start Qdrant
            println!();
            if prompt_yes_no("Start Qdrant container for semantic search?", true) {
                println!("  Starting Qdrant (this may take a moment on first run)...");
                if let Err(e) = create_qdrant_container() {
                    println!("  {} Failed to start Qdrant: {}", "!", e);
                } else if wait_for_qdrant(30) {
                    qdrant_running = true;
                    println!("  {} Qdrant started", "✓");
                } else {
                    println!("  {} Qdrant may still be starting up", "!");
                }
            }
        }
    }

    // Step 3: Check OpenAI API key
    let embedding_provider = embedding.clone().unwrap_or_else(|| {
        if get_openai_key().is_some() {
            "openai".to_string()
        } else {
            "none".to_string()
        }
    });

    if embedding_provider != "none" && get_openai_key().is_none() {
        if !quiet {
            match prompt_openai_key()? {
                Some(_) => println!("  {} OpenAI API key saved", "✓"),
                None => {
                    println!("  {} Skipped OpenAI setup. Semantic search disabled.", "!");
                }
            }
        }
    } else if !quiet && get_openai_key().is_some() {
        println!("  {} OpenAI API key found", "✓");
    }

    // Step 4: Create config.yaml
    let config_path = am_dir.join("config.yaml");
    let config_exists = config_path.exists();

    let mut config = if config_exists {
        crate::config::load_config(&config_path).unwrap_or_default()
    } else {
        Config::default()
    };

    // Set project-specific Qdrant collection name
    if !config_exists {
        let project_id = get_project_id();
        config.qdrant.collection = format!("agentmem_{}", project_id);
        if !quiet {
            println!("  {} Using Qdrant collection: {}", "✓", config.qdrant.collection);
        }
    }

    // Update embedding provider
    if let Some(provider) = embedding {
        config.embedding.provider = provider;
    } else if get_openai_key().is_some() && config.embedding.provider == "none" {
        config.embedding.provider = "openai".to_string();
    }

    if let Some(m) = model {
        config.embedding.model = Some(m);
    }

    save_config(&config_path, &config).context("Failed to save config.yaml")?;
    if !quiet && !config_exists {
        println!("  {} Created config.yaml", "✓");
    }

    // Step 5: Initialize database
    let db_path = am_dir.join("agentmem.db");
    let _conn = get_connection(db_path).context("Failed to initialize database")?;
    if !quiet {
        println!("  {} Initialized database", "✓");
    }

    // Create agentmem.jsonl (placeholder if doesn't exist)
    let jsonl_path = am_dir.join("agentmem.jsonl");
    if !jsonl_path.exists() {
        fs::write(&jsonl_path, "").context("Failed to create agentmem.jsonl")?;
    }

    // Step 6: Install Claude Code plugin
    let mut plugin_installed = is_plugin_installed();
    if !quiet && !plugin_installed {
        println!();
        if prompt_yes_no("Install AgentMem plugin for Claude Code?", true) {
            match install_plugin() {
                Ok(_) => {
                    plugin_installed = true;
                    println!("  {} Installed Claude Code plugin", "✓");
                    println!("  {} Installed AgentMem skills", "✓");
                }
                Err(e) => {
                    println!("  {} Failed to install plugin: {}", "!", e);
                    println!("    You can install manually: cp -r plugin ~/.claude/plugins/agentmem");
                }
            }
        }
    } else if plugin_installed && !quiet {
        println!("  {} Claude Code plugin already installed", "✓");
    }

    // Step 7: Print success message
    if !quiet {
        println!();
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  AgentMem initialized successfully!");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!();

        if plugin_installed {
            println!("  Claude Code plugin is installed!");
            println!("  AgentMem will automatically:");
            println!("    - Inject context before each prompt");
            println!("    - Sync data when sessions end");
            println!();
            println!("  Available commands:");
            println!("    /agentmem:remember <type> <title> - Add a memory");
            println!("    /agentmem:protect <file>          - Protect a file");
            println!("    /agentmem:context                 - Show current context");
            println!("    /agentmem:sync                    - Sync to git");
            println!();
        } else {
            println!("  Next steps:");
            println!();
            println!("  1. Install hooks for your AI agent:");
            println!("     am hook install claude-code");
            println!();
        }

        println!("  CLI commands:");
        println!("    am mem add decision \"Title\" --content \"Details\"");
        println!("    am task create \"Task title\"");
        println!("    am context --query \"search term\"");
        println!("    am doctor");
        println!();

        if !qdrant_running {
            println!("  Note: Semantic search is disabled (Qdrant not running).");
            println!("        Run 'docker start agentmem-qdrant' to enable it.");
            println!();
        }
    }

    Ok(())
}
