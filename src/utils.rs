use anyhow::{Context, Result};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

const GIT_LOCK_FILE: &str = ".polyws/git.lock";

pub struct RepoLock {
    path: PathBuf,
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

pub fn repo_lock_exists() -> bool {
    Path::new(GIT_LOCK_FILE).exists()
}

pub fn try_acquire_repo_lock(label: &str) -> Result<Option<RepoLock>> {
    fs::create_dir_all(".polyws").context("Failed to create .polyws directory")?;

    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(GIT_LOCK_FILE)
    {
        Ok(mut file) => {
            let _ = writeln!(file, "pid={}", std::process::id());
            let _ = writeln!(file, "label={}", label);
            Ok(Some(RepoLock {
                path: PathBuf::from(GIT_LOCK_FILE),
            }))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if let Some(pid) = read_lock_pid() {
                if !is_pid_alive(pid) {
                    let _ = fs::remove_file(GIT_LOCK_FILE);
                    return try_acquire_repo_lock(label);
                }
            }
            Ok(None)
        }
        Err(e) => Err(e).context("Failed to create git lock file"),
    }
}

pub fn acquire_repo_lock(label: &str) -> Result<RepoLock> {
    loop {
        if let Some(lock) = try_acquire_repo_lock(label)? {
            return Ok(lock);
        }
        thread::sleep(Duration::from_millis(500));
    }
}

fn read_lock_pid() -> Option<u32> {
    let content = fs::read_to_string(GIT_LOCK_FILE).ok()?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("pid=") {
            if let Ok(pid) = rest.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

fn is_pid_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = pid;
        false
    }
}

/// Print a success line with a green checkmark.
pub fn print_ok(msg: &str) {
    println!("  \x1b[32m✔\x1b[0m {}", msg);
}

/// Print a failure line with a red cross.
pub fn print_fail(msg: &str) {
    println!("  \x1b[31m✘\x1b[0m {}", msg);
}

/// Print a warning line with a yellow triangle.
pub fn print_warn(msg: &str) {
    println!("  \x1b[33m⚠\x1b[0m {}", msg);
}

/// Print an informational line with a blue arrow.
pub fn print_info(msg: &str) {
    println!("  \x1b[34m→\x1b[0m {}", msg);
}

/// Print a bold section header.
pub fn print_section(title: &str) {
    println!("\n\x1b[1m[{}]\x1b[0m", title);
}

// ─────────────────────────────────────────────────────────
// Table renderer (plain terminal, no ratatui dependency)
// ─────────────────────────────────────────────────────────

/// Print an aligned table to stdout.
///
/// `headers`  — column header labels  
/// `rows`     — each row is a `Vec<String>` with the same number of columns  
/// `colors`   — optional ANSI color code per column (e.g. `"32"` for green);
///              use `""` for default terminal color.
///
/// # Example
/// ```
/// print_table(
///     &["Name", "Branch", "Status"],
///     &[vec!["foo".into(), "main".into(), "clean".into()]],
///     &["", "36", "32"],
/// );
/// ```
pub fn print_table(headers: &[&str], rows: &[Vec<String>], colors: &[&str]) {
    // Compute column widths as the max of header length and any cell length.
    let ncols = headers.len();
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < ncols {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    let horiz = |widths: &[usize]| {
        let inner: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        inner.join("┼")
    };

    // Top border
    {
        let inner: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        println!("┌{}┐", inner.join("┬"));
    }

    // Header row
    {
        let cells: Vec<String> = headers
            .iter()
            .zip(widths.iter())
            .map(|(h, w)| format!(" \x1b[1m{:<width$}\x1b[0m ", h, width = w))
            .collect();
        println!("│{}│", cells.join("│"));
    }

    // Header/body separator
    println!("├{}┤", horiz(&widths));

    // Data rows
    for row in rows {
        let cells: Vec<String> = (0..ncols)
            .map(|i| {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let color = colors.get(i).copied().unwrap_or("");
                if color.is_empty() {
                    format!(" {:<width$} ", cell, width = widths[i])
                } else {
                    // color the text but pad without escape codes affecting width
                    format!(
                        " \x1b[{}m{}\x1b[0m{} ",
                        color,
                        cell,
                        " ".repeat(widths[i].saturating_sub(cell.len()))
                    )
                }
            })
            .collect();
        println!("│{}│", cells.join("│"));
    }

    // Bottom border
    {
        let inner: Vec<String> = widths.iter().map(|w| "─".repeat(w + 2)).collect();
        println!("└{}┘", inner.join("┴"));
    }
}
