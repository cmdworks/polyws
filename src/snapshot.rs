use anyhow::{Context, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::config::WorkspaceConfig;
use crate::git;
use crate::utils;

const SNAPSHOTS_DIR: &str = ".polyws/snapshots";

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub created_at: String,
    pub workspace: String,
    /// Map from project name → abbreviated commit hash.
    pub commits: HashMap<String, String>,
}

// ---------------------------------------------------------------------------
// create
// ---------------------------------------------------------------------------

pub fn create() -> Result<()> {
    let config = WorkspaceConfig::load()?;
    let mut commits = HashMap::new();

    // Iterate in dependency order so the snapshot output is deterministic.
    for project in config.topological_order()? {
        let path = Path::new(project.local_dir());
        if path.exists() {
            match git::get_commit_hash(path) {
                Ok(hash) => {
                    commits.insert(project.name.clone(), hash);
                }
                Err(e) => {
                    utils::print_warn(&format!("Could not snapshot '{}': {}", project.name, e))
                }
            }
        } else {
            utils::print_warn(&format!("'{}' not found, skipping", project.name));
        }
    }

    let snapshot = Snapshot {
        created_at: Utc::now().to_rfc3339(),
        workspace: config.name.clone(),
        commits,
    };

    fs::create_dir_all(SNAPSHOTS_DIR).context("Failed to create snapshots directory")?;
    let ts = Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}/{}.json", SNAPSHOTS_DIR, ts);
    let content = serde_json::to_string_pretty(&snapshot)?;
    fs::write(&filename, &content)?;

    utils::print_ok(&format!("Snapshot saved → {}", filename));
    for (name, hash) in &snapshot.commits {
        println!("    {} @ {}", name, hash);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// restore
// ---------------------------------------------------------------------------

pub fn restore(file: &str, dry_run: bool, yes: bool) -> Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Could not read snapshot file: {}", file))?;
    let snapshot: Snapshot =
        serde_json::from_str(&content).context("Invalid snapshot file format")?;

    println!(
        "Restoring snapshot from \x1b[1m{}\x1b[0m (created {})",
        file, snapshot.created_at
    );
    println!("Projects in snapshot: {}", snapshot.commits.len());

    if dry_run {
        utils::print_info("Dry-run mode enabled. No repositories will be modified.");
    } else if !yes {
        println!("\x1b[31mWARNING:\x1b[0m This performs a hard reset in each listed repository.");
        print!("Type 'restore' to continue: ");
        io::stdout().flush().ok();
        let mut confirm = String::new();
        io::stdin().read_line(&mut confirm)?;
        if confirm.trim() != "restore" {
            utils::print_warn("Restore cancelled.");
            return Ok(());
        }
    }

    let cfg = WorkspaceConfig::load().ok();

    for (name, hash) in &snapshot.commits {
        let path_str = cfg
            .as_ref()
            .and_then(|c| c.find_project(name))
            .map(|p| p.local_dir())
            .unwrap_or(name.as_str());
        let path = Path::new(path_str);
        if !path.exists() {
            utils::print_warn(&format!(
                "'{}' ({}) not found, skipping",
                name,
                path.display()
            ));
            continue;
        }

        if dry_run {
            utils::print_info(&format!("would restore {} → {}", name, hash));
            continue;
        }

        match git::checkout_commit(path, hash) {
            Ok(_) => utils::print_ok(&format!("{} → {}", name, hash)),
            Err(e) => utils::print_fail(&format!("Failed to restore {}: {}", name, e)),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

pub fn list() -> Result<()> {
    let dir = Path::new(SNAPSHOTS_DIR);
    if !dir.exists() {
        println!("No snapshots found.");
        return Ok(());
    }

    let mut entries: Vec<_> = fs::read_dir(dir)
        .context("Failed to read snapshots directory")?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if entries.is_empty() {
        println!("No snapshots found.");
        return Ok(());
    }

    println!("Snapshots ({}):", entries.len());
    for entry in &entries {
        let path = entry.path();
        let meta = match fs::read_to_string(&path)
            .ok()
            .and_then(|c| serde_json::from_str::<Snapshot>(&c).ok())
        {
            Some(s) => format!("  created {}  ({} repos)", s.created_at, s.commits.len()),
            None => String::new(),
        };
        println!("  • {}{}", path.display(), meta);
    }
    Ok(())
}
