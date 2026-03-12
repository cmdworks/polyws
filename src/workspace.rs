use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::process::Command;

use crate::config::{find_existing_config_path, normalize_local_dir, Project, WorkspaceConfig};
use crate::git;
use crate::utils;

enum PullOutcome {
    Updated,
    Cloned,
}

// ---------------------------------------------------------------------------
// init
// ---------------------------------------------------------------------------

fn init_inner() -> Result<String> {
    ensure_git_installed()?;

    if let Some(path) = find_existing_config_path() {
        anyhow::bail!("workspace config already exists: {}", path);
    }

    let name = std::env::current_dir()?
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let config = WorkspaceConfig {
        name: name.clone(),
        sync_interval_minutes: None,
        projects: vec![],
        vm: None,
    };
    config.save()?;
    Ok(name)
}

fn ensure_git_installed() -> Result<()> {
    let ok = Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !ok {
        anyhow::bail!(
            "git is required but was not found. Install git first, then run `polyws init` again."
        );
    }
    Ok(())
}

pub fn init() -> Result<()> {
    let name = init_inner()?;
    println!("Initialized workspace '{}'", name);
    Ok(())
}

/// Initialize workspace config without printing to stdout.
/// Useful for interactive UIs where direct terminal output would corrupt rendering.
pub fn init_silent() -> Result<String> {
    init_inner()
}

// ---------------------------------------------------------------------------
// add / remove / list
// ---------------------------------------------------------------------------

pub fn add(
    name: String,
    path: Option<String>,
    url: String,
    branch: String,
    depends_on: Vec<String>,
    sync_url: Option<String>,
) -> Result<()> {
    let mut config = WorkspaceConfig::load()?;
    let path = path
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty() && p != &name);

    if config.projects.iter().any(|p| p.name == name) {
        anyhow::bail!("Project '{}' already exists in the workspace", name);
    }
    if let Some(ref pth) = path {
        if Path::new(pth).is_absolute() {
            anyhow::bail!("Project path must be relative to workspace root");
        }
        let requested = normalize_local_dir(pth);
        if config
            .projects
            .iter()
            .any(|p| normalize_local_dir(p.local_dir()) == requested)
        {
            anyhow::bail!("Project path '{}' is already used by another project", pth);
        }
    } else {
        let requested = normalize_local_dir(&name);
        if config
            .projects
            .iter()
            .any(|p| normalize_local_dir(p.local_dir()) == requested)
        {
            anyhow::bail!("Project path '{}' is already used by another project", name);
        }
    }

    if depends_on.iter().any(|dep| dep == &name) {
        anyhow::bail!("Project '{}' cannot depend on itself", name);
    }
    for dep in &depends_on {
        if config.find_project(dep).is_none() {
            anyhow::bail!(
                "Dependency '{}' not found. Add it first or remove it from --depends-on.",
                dep
            );
        }
    }

    config.projects.push(Project {
        name: name.clone(),
        path,
        url,
        branch,
        depends_on: if depends_on.is_empty() {
            None
        } else {
            Some(depends_on)
        },
        sync_url,
        sync_interval: None,
    });
    config.save()?;
    utils::print_ok(&format!("Added project '{}'", name));
    Ok(())
}

pub fn remove(name: &str) -> Result<()> {
    let mut config = WorkspaceConfig::load()?;
    let before = config.projects.len();
    config.projects.retain(|p| p.name != name);

    if config.projects.len() == before {
        anyhow::bail!("Project '{}' not found in the workspace", name);
    }
    config.save()?;
    utils::print_ok(&format!("Removed project '{}'", name));
    Ok(())
}

pub fn list() -> Result<()> {
    let config = WorkspaceConfig::load()?;
    println!(
        "Workspace: \x1b[1m{}\x1b[0m  ({} projects)",
        config.name,
        config.projects.len()
    );

    let rows: Vec<Vec<String>> = config
        .projects
        .iter()
        .map(|p| {
            let deps = p
                .depends_on
                .as_deref()
                .filter(|d| !d.is_empty())
                .map(|d| d.join(", "))
                .unwrap_or_else(|| "—".to_string());
            let path = p.local_dir().to_string();
            let mirror = p.sync_url.as_deref().unwrap_or("—").to_string();
            vec![p.name.clone(), path, p.branch.clone(), deps, mirror]
        })
        .collect();

    utils::print_table(
        &["Name", "Path", "Branch", "Depends On", "Mirror URL"],
        &rows,
        &["", "", "36", "35", "2"],
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// pull / clone
// ---------------------------------------------------------------------------

pub fn pull(name: Option<String>, force: bool) -> Result<()> {
    let config = WorkspaceConfig::load()?;
    let mut any_failed = false;

    match &name {
        Some(n) => {
            let p = config
                .find_project(n)
                .with_context(|| format!("Project '{}' not found", n))?;
            match run_pull_for_project(p, force) {
                Ok(PullOutcome::Updated) => utils::print_ok(&format!("{} updated", p.name)),
                Ok(PullOutcome::Cloned) => utils::print_ok(&format!("{} cloned", p.name)),
                Err(e) => {
                    utils::print_fail(&format!("{}: {}", p.name, e));
                    any_failed = true;
                }
            }
        }
        None => {
            let levels = config.execution_levels()?;
            for (level_idx, level) in levels.iter().enumerate() {
                if level.len() > 1 {
                    utils::print_info(&format!(
                        "Pull level {} — running {} projects in parallel",
                        level_idx + 1,
                        level.len()
                    ));
                }

                let results: Vec<(String, Result<PullOutcome>)> = level
                    .par_iter()
                    .map(|project| (project.name.clone(), run_pull_for_project(project, force)))
                    .collect();

                for (name, result) in results {
                    match result {
                        Ok(PullOutcome::Updated) => utils::print_ok(&format!("{} updated", name)),
                        Ok(PullOutcome::Cloned) => utils::print_ok(&format!("{} cloned", name)),
                        Err(e) => {
                            utils::print_fail(&format!("{}: {}", name, e));
                            any_failed = true;
                        }
                    }
                }
            }
        }
    }

    if any_failed {
        anyhow::bail!("one or more projects failed during pull");
    }

    Ok(())
}

fn run_pull_for_project(project: &Project, force: bool) -> Result<PullOutcome> {
    let path = Path::new(project.local_dir());
    if path.exists() {
        if git::is_repo(path) {
            git::pull_repo(path, &project.branch, force)?;
            return Ok(PullOutcome::Updated);
        }
        if is_dir_empty(path) {
            git::clone_repo(&project.url, path)?;
            return Ok(PullOutcome::Cloned);
        }
        anyhow::bail!(
            "'{}' exists but is not a git repository",
            project.local_dir()
        );
    }

    git::clone_repo(&project.url, path)?;
    Ok(PullOutcome::Cloned)
}

fn is_dir_empty(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
}

/// `clone` is a user-facing alias for `pull`.
pub fn clone_repos(name: Option<String>, force: bool) -> Result<()> {
    pull(name, force)
}

// ---------------------------------------------------------------------------
// push
// ---------------------------------------------------------------------------

pub fn push(name: Option<String>) -> Result<()> {
    let config = WorkspaceConfig::load()?;
    let mut any_failed = false;

    match &name {
        Some(n) => {
            let p = config
                .find_project(n)
                .with_context(|| format!("Project '{}' not found", n))?;
            match run_push_for_project(p) {
                Ok(_) => utils::print_ok(&format!("{} pushed", p.name)),
                Err(e) => {
                    utils::print_fail(&format!("{}: {}", p.name, e));
                    any_failed = true;
                }
            }
        }
        None => {
            let levels = config.execution_levels()?;
            for (level_idx, level) in levels.iter().enumerate() {
                if level.len() > 1 {
                    utils::print_info(&format!(
                        "Push level {} — running {} projects in parallel",
                        level_idx + 1,
                        level.len()
                    ));
                }

                let results: Vec<(String, Result<()>)> = level
                    .par_iter()
                    .map(|project| (project.name.clone(), run_push_for_project(project)))
                    .collect();

                for (name, result) in results {
                    match result {
                        Ok(_) => utils::print_ok(&format!("{} pushed", name)),
                        Err(e) => {
                            utils::print_fail(&format!("{}: {}", name, e));
                            any_failed = true;
                        }
                    }
                }
            }
        }
    }

    if any_failed {
        anyhow::bail!("one or more projects failed during push");
    }

    Ok(())
}

fn run_push_for_project(project: &Project) -> Result<()> {
    let path = Path::new(project.local_dir());
    if !path.exists() {
        anyhow::bail!(
            "directory '{}' not found — run `polyws pull` first",
            project.local_dir()
        );
    }
    git::push_repo(path, &project.branch)
}

// ---------------------------------------------------------------------------
// status
// ---------------------------------------------------------------------------

pub fn status() -> Result<()> {
    let config = WorkspaceConfig::load()?;
    println!("Workspace: \x1b[1m{}\x1b[0m", config.name);

    let rows: Vec<Vec<String>> = config
        .projects
        .iter()
        .map(|project| {
            let path = Path::new(project.local_dir());
            let (status_text, _) = if path.exists() {
                match git::repo_status(path) {
                    Ok(s) => (s, false),
                    Err(_) => ("not a git repo".to_string(), true),
                }
            } else {
                ("missing — run `polyws pull`".to_string(), true)
            };
            vec![project.name.clone(), project.branch.clone(), status_text]
        })
        .collect();

    utils::print_table(&["Name", "Branch", "Status"], &rows, &["", "36", ""]);
    Ok(())
}

// ---------------------------------------------------------------------------
// graph
// ---------------------------------------------------------------------------

pub fn graph() -> Result<()> {
    let config = WorkspaceConfig::load()?;

    // Build dependency→dependents map.
    let children: HashMap<String, Vec<String>> = config.dependent_map();

    // Roots = projects that do not appear as a dependency of any other project AND
    // have no dependencies themselves (pure roots) OR all projects with an empty
    // depends_on when there are no pure roots.
    let roots: Vec<&str> = {
        let mut r: Vec<&str> = config
            .projects
            .iter()
            .filter(|p| p.depends_on.as_ref().map(|d| d.is_empty()).unwrap_or(true))
            .map(|p| p.name.as_str())
            .collect();

        // Fallback: show every project if no clear root exists.
        if r.is_empty() {
            r = config.projects.iter().map(|p| p.name.as_str()).collect();
        }
        r
    };

    println!("Dependency graph — \x1b[1m{}\x1b[0m\n", config.name);

    for root in roots {
        println!("{}", root);
        if let Some(kids) = children.get(root) {
            for (i, kid) in kids.iter().enumerate() {
                let last = i == kids.len() - 1;
                print_tree(kid, &children, "", last);
            }
        }
    }
    Ok(())
}

fn print_tree(name: &str, children: &HashMap<String, Vec<String>>, prefix: &str, is_last: bool) {
    let connector = if is_last { "└─" } else { "├─" };
    println!("{}{} {}", prefix, connector, name);

    let child_prefix = if is_last {
        format!("{}   ", prefix)
    } else {
        format!("{}│  ", prefix)
    };

    if let Some(kids) = children.get(name) {
        for (i, kid) in kids.iter().enumerate() {
            print_tree(kid, children, &child_prefix, i == kids.len() - 1);
        }
    }
}

// ---------------------------------------------------------------------------
// repair
// ---------------------------------------------------------------------------

pub fn repair() -> Result<()> {
    let config = WorkspaceConfig::load()?;
    println!("Repairing workspace '\x1b[1m{}\x1b[0m'...", config.name);

    for project in &config.projects {
        let path = Path::new(project.local_dir());

        if !path.exists() {
            utils::print_info(&format!(
                "Re-cloning missing repository: {} ({})",
                project.name,
                project.local_dir()
            ));
            match git::clone_repo(&project.url, path) {
                Ok(_) => utils::print_ok(&format!("{} cloned", project.name)),
                Err(e) => utils::print_fail(&format!("Failed to clone {}: {}", project.name, e)),
            }
            continue;
        }

        if !git::is_repo(path) {
            utils::print_fail(&format!(
                "'{}' ({}) exists but is not a git repository",
                project.name,
                project.local_dir()
            ));
            utils::print_info("Remove the directory and run `polyws repair` again to reclone.");
            continue;
        }

        // Verify and fix the remote URL.
        match git::get_remote_url(path) {
            Ok(current_url) if current_url == project.url => {
                utils::print_ok(&format!("{} OK", project.name));
            }
            Ok(current_url) => {
                utils::print_info(&format!(
                    "Fixing remote URL for '{}': {} → {}",
                    project.name, current_url, project.url
                ));
                let fixed = Command::new("git")
                    .args(["remote", "set-url", "origin", &project.url])
                    .current_dir(path)
                    .status()?;
                if fixed.success() {
                    utils::print_ok(&format!("{} remote fixed", project.name));
                } else {
                    utils::print_fail(&format!("Could not fix remote for {}", project.name));
                }
            }
            Err(_) => utils::print_fail(&format!("Could not check remote for '{}'", project.name)),
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// bootstrap
// ---------------------------------------------------------------------------

pub async fn bootstrap() -> Result<()> {
    println!("\x1b[1mBootstrapping workspace\x1b[0m");
    crate::doctor::run().await?;
    println!();
    pull(None, false)?;
    Ok(())
}
