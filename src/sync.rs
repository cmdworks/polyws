use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use crate::config::WorkspaceConfig;
use crate::git;
use crate::utils;

const PID_FILE: &str = ".polyws/sync.pid";
const POLYWS_DIR: &str = ".polyws";
const SYNC_LOG: &str = ".polyws/sync.log";

// ---------------------------------------------------------------------------
// start / stop / status
// ---------------------------------------------------------------------------

/// Spawn a detached background process that runs the sync daemon loop.
pub fn start() -> Result<()> {
    let msg = start_silent()?;
    if msg.contains("already running") {
        println!("{}", msg);
    } else {
        utils::print_ok(&msg);
    }
    Ok(())
}

/// Start the daemon without printing to stdout/stderr.
/// Returns a user-facing status line for TUI logging.
pub fn start_silent() -> Result<String> {
    if is_running()? {
        return Ok("Sync daemon is already running.".to_string());
    }

    fs::create_dir_all(POLYWS_DIR).context("Failed to create .polyws directory")?;

    let current_exe =
        std::env::current_exe().context("Failed to determine the path to the polyws binary")?;

    // Spawn itself with the hidden `sync-daemon` sub-command and detach.
    let child = Command::new(&current_exe)
        .arg("sync-daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .context("Failed to spawn sync daemon")?;

    let pid = child.id();
    fs::write(PID_FILE, pid.to_string()).context("Failed to write PID file")?;
    Ok(format!("Sync daemon started (PID {})", pid))
}

pub fn stop() -> Result<()> {
    let msg = stop_silent()?;
    if msg.contains("not running") {
        println!("{}", msg);
    } else if msg.contains("Could not signal") {
        utils::print_warn(&msg);
    } else {
        utils::print_ok(&msg);
    }
    Ok(())
}

/// Stop the daemon without printing to stdout/stderr.
/// Returns a user-facing status line for TUI logging.
pub fn stop_silent() -> Result<String> {
    if !Path::new(PID_FILE).exists() {
        return Ok("Sync daemon is not running.".to_string());
    }

    let pid_str = fs::read_to_string(PID_FILE).context("Failed to read PID file")?;
    let pid: u32 = pid_str.trim().parse().context("Invalid PID in file")?;

    #[cfg(unix)]
    {
        let ok = Command::new("kill")
            .arg(pid.to_string())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        let msg = if ok {
            format!("Sync daemon stopped (PID {})", pid)
        } else {
            "Could not signal daemon — it may have already stopped.".to_string()
        };
        let _ = fs::remove_file(PID_FILE);
        return Ok(msg);
    }
    #[cfg(windows)]
    {
        let ok = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        let msg = if ok {
            format!("Sync daemon stopped (PID {})", pid)
        } else {
            "Could not signal daemon — it may have already stopped.".to_string()
        };
        let _ = fs::remove_file(PID_FILE);
        return Ok(msg);
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = fs::remove_file(PID_FILE);
        return Ok("Process signalling is not supported on this platform.".to_string());
    }
}

pub fn status() -> Result<()> {
    if is_running()? {
        let pid_str = fs::read_to_string(PID_FILE).unwrap_or_default();
        utils::print_ok(&format!("Sync daemon running (PID {})", pid_str.trim()));
    } else {
        utils::print_warn("Sync daemon is not running");
    }
    if let Some(line) = last_log_line() {
        println!("Last sync: {}", line);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// sync_now — run one mirror-push pass immediately
// ---------------------------------------------------------------------------

pub fn sync_now() -> Result<()> {
    for line in sync_now_silent()? {
        println!("{}", line);
    }
    Ok(())
}

/// Run one mirror sync pass without direct terminal output.
/// Returns the lines that can be shown in the TUI log panel.
pub fn sync_now_silent() -> Result<Vec<String>> {
    let _lock = utils::acquire_repo_lock("sync-now")?;
    let config = WorkspaceConfig::load()?;
    let mut synced = 0usize;
    let mut lines = Vec::new();

    for project in &config.projects {
        let sync_url = match &project.sync_url {
            Some(u) => u,
            None => continue,
        };
        let path = Path::new(project.local_dir());
        if !path.exists() {
            let msg = format!("'{}' not found, skipping", project.name);
            lines.push(format!("⚠ {}", msg));
            log_sync_line(&format!("sync: {}", msg));
            continue;
        }
        match git::push_sync_branch(path, &project.branch, sync_url) {
            Ok(_) => {
                let msg = format!("{} → {}", project.name, sync_url);
                lines.push(format!("✔ {}", msg));
                log_sync_line(&format!("sync: {}", msg));
                synced += 1;
            }
            Err(e) => {
                let msg = format!("{}: {}", project.name, e);
                lines.push(format!("✘ {}", msg));
                log_sync_line(&format!("sync: {}", msg));
            }
        }
    }

    if synced == 0 {
        let has_sync = config.projects.iter().any(|p| p.sync_url.is_some());
        if !has_sync {
            lines.push("No projects have a sync_url configured.".to_string());
        }
    }
    Ok(lines)
}

// ---------------------------------------------------------------------------
// run_daemon — long-running loop called by the hidden SyncDaemon command
// ---------------------------------------------------------------------------

/// Runs inside the detached background process spawned by `start()`.
pub async fn run_daemon() -> Result<()> {
    let mut last_synced: HashMap<String, Instant> = HashMap::new();

    loop {
        let config =
            WorkspaceConfig::load().context("Sync daemon: failed to load workspace config")?;
        let default_interval = config.sync_interval_minutes.unwrap_or(5).max(1);

        let mut any_due = false;
        for project in &config.projects {
            if project.sync_url.is_none() {
                continue;
            }
            let interval_minutes = project.sync_interval.unwrap_or(default_interval).max(1);
            let is_due = last_synced
                .get(&project.name)
                .map(|last| last.elapsed() >= Duration::from_secs(interval_minutes * 60))
                .unwrap_or(true);
            if is_due {
                any_due = true;
                break;
            }
        }

        if !any_due {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            continue;
        }

        let mut warned = false;
        let lock = loop {
            match utils::try_acquire_repo_lock("sync-daemon")? {
                Some(lock) => break lock,
                None => {
                    if !warned {
                        log_sync_line("sync: waiting for other git operations");
                        warned = true;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        };

        for project in &config.projects {
            let sync_url = match &project.sync_url {
                Some(u) => u,
                None => continue,
            };

            let interval_minutes = project.sync_interval.unwrap_or(default_interval).max(1);
            let is_due = last_synced
                .get(&project.name)
                .map(|last| last.elapsed() >= Duration::from_secs(interval_minutes * 60))
                .unwrap_or(true);

            if !is_due {
                continue;
            }

            let path = Path::new(project.local_dir());
            if !path.exists() {
                log_sync_line(&format!(
                    "sync: '{}' not found, skipping this interval",
                    project.name
                ));
                last_synced.insert(project.name.clone(), Instant::now());
                continue;
            }

            match git::push_sync_branch(path, &project.branch, sync_url) {
                Ok(_) => log_sync_line(&format!("sync: {} → {}", project.name, sync_url)),
                Err(e) => log_sync_line(&format!("sync: {}: {}", project.name, e)),
            }

            last_synced.insert(project.name.clone(), Instant::now());
        }

        drop(lock);
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

pub fn is_mutagen_installed() -> bool {
    Command::new("mutagen").arg("version").output().is_ok()
}

pub fn last_log_line() -> Option<String> {
    let content = fs::read_to_string(SYNC_LOG).ok()?;
    content.lines().last().map(|s| s.to_string())
}

/// Non-fallible helper for the TUI — returns `false` on any error.
pub fn is_daemon_running() -> bool {
    is_running().unwrap_or(false)
}

fn is_running() -> Result<bool> {
    if !Path::new(PID_FILE).exists() {
        return Ok(false);
    }
    let pid_str = fs::read_to_string(PID_FILE)?;
    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => return Ok(false),
    };

    #[cfg(unix)]
    {
        // `kill -0 PID` succeeds only when the process exists.
        let alive = Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        Ok(alive)
    }
    #[cfg(windows)]
    {
        let alive = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false);
        Ok(alive)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        Ok(false)
    }
}

fn log_sync_line(line: &str) {
    let _ = fs::create_dir_all(POLYWS_DIR);
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(SYNC_LOG)
    {
        let _ = writeln!(file, "{}", line);
    }
}
