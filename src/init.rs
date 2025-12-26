use std::fs;
use anyhow::{Result, Context};
use crate::config::{Config, get_agentmem_dir, get_config_path, get_db_path, save_config};
use crate::db::get_connection;

pub fn run_init(quiet: bool, embedding: Option<String>, model: Option<String>) -> Result<()> {
    let am_dir = get_agentmem_dir();
    
    if am_dir.exists() {
        if !quiet {
            println!(".agentmem directory already exists. Re-initializing...");
        }
    } else {
        fs::create_dir_all(&am_dir).context("Failed to create .agentmem directory")?;
        if !quiet {
            println!("✓ Created .agentmem/");
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
        fs::write(gitignore_path, "*.db\n").context("Failed to create .gitignore")?;
    }

    // Create config.yaml
    let config_path = get_config_path();
    if !config_path.exists() {
        let mut config = Config::default();
        if let Some(provider) = embedding {
            config.embedding.provider = provider;
        }
        if let Some(m) = model {
            config.embedding.model = Some(m);
        }
        save_config(&config_path, &config).context("Failed to save config.yaml")?;
        if !quiet {
            println!("✓ Created config.yaml");
        }
    }

    // Initialize database
    let db_path = get_db_path();
    let _conn = get_connection(db_path).context("Failed to initialize database")?;
    if !quiet {
        println!("✓ Initialized database");
    }

    // Create agentmem.jsonl (placeholder if doesn't exist)
    let jsonl_path = am_dir.join("agentmem.jsonl");
    if !jsonl_path.exists() {
        fs::write(jsonl_path, "").context("Failed to create agentmem.jsonl")?;
    }

    if !quiet {
        println!("✓ AgentMem initialized successfully!");
    }

    Ok(())
}

