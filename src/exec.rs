use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;
use std::process::Command;

use crate::config::WorkspaceConfig;
use crate::utils;

/// Execute `cmd` across all workspace repositories, respecting dependency order.
///
/// Projects in the same dependency level run in parallel via rayon.
/// Each level is fully complete before the next level starts.
pub async fn run(cmd: String) -> Result<()> {
    if utils::repo_lock_exists() {
        utils::print_warn("Another git operation is running; waiting...");
    }
    let _lock = utils::acquire_repo_lock("exec")?;
    let config = WorkspaceConfig::load()?;

    if config.projects.is_empty() {
        println!("No projects in workspace. Add some with `polyws add`.");
        return Ok(());
    }

    let levels = config.execution_levels()?;

    println!(
        "Executing \x1b[1m{}\x1b[0m across {} project(s) in {} level(s)\n",
        cmd,
        config.projects.len(),
        levels.len()
    );

    let mut any_failed = false;

    for (level_idx, level) in levels.iter().enumerate() {
        if level.len() > 1 {
            utils::print_info(&format!(
                "Level {} — running {} projects in parallel",
                level_idx + 1,
                level.len()
            ));
        }

        // Run all projects in this level concurrently.
        let results: Vec<(String, Result<()>)> = level
            .par_iter()
            .map(|proj| {
                let result = run_in_project(&proj.name, proj.local_dir(), &cmd);
                (proj.name.clone(), result)
            })
            .collect();

        for (name, result) in results {
            match result {
                Ok(_) => utils::print_ok(&format!("{} succeeded", name)),
                Err(e) => {
                    utils::print_fail(&format!("{} failed: {}", name, e));
                    any_failed = true;
                }
            }
        }
    }

    if any_failed {
        anyhow::bail!("one or more projects failed");
    }

    Ok(())
}

fn run_in_project(name: &str, dir: &str, cmd: &str) -> Result<()> {
    let path = Path::new(dir);
    if !path.exists() {
        anyhow::bail!(
            "directory '{}' for project '{}' not found — run `polyws pull` first",
            dir,
            name
        );
    }

    let status = if cfg!(windows) {
        Command::new("cmd")
            .args(["/C", cmd])
            .current_dir(path)
            .status()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(path)
            .status()
    }
    .map_err(|e| anyhow::anyhow!("Failed to spawn command in '{}': {}", dir, e))?;

    if !status.success() {
        anyhow::bail!("command exited with status {}", status.code().unwrap_or(-1));
    }
    Ok(())
}
