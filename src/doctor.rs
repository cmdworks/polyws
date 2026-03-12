use anyhow::Result;
use std::process::Command;

use crate::config::{find_existing_config_path, known_config_paths, WorkspaceConfig};
use crate::utils;

pub async fn run() -> Result<()> {
    utils::print_section("Environment");
    check_tool("git", &["--version"], "git installed");
    check_tool("ssh", &["-V"], "ssh available");
    check_tool("rustc", &["--version"], "rustc installed");
    check_tool("cargo", &["--version"], "cargo available");

    utils::print_section("Connectivity");
    check_internet();

    utils::print_section("Workspace");
    check_workspace();

    utils::print_section("Disk");
    check_disk();

    Ok(())
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

fn check_tool(bin: &str, args: &[&str], label: &str) {
    match Command::new(bin).args(args).output() {
        // ssh -V writes to stderr; accept non-zero exit with stderr output.
        Ok(o) if o.status.success() || !o.stderr.is_empty() => utils::print_ok(label),
        Ok(_) => {
            utils::print_fail(&format!("{} not working", bin));
            suggest_install(bin);
        }
        Err(_) => {
            utils::print_fail(label);
            suggest_install(bin);
        }
    }
}

fn suggest_install(bin: &str) {
    let hint = if cfg!(windows) {
        match bin {
            "git" => "winget install Git.Git  |  choco install git",
            "rustc" | "cargo" => "download rustup from https://rustup.rs/",
            "ssh" => {
                "Windows 10+ includes OpenSSH — enable it in Settings > Apps > Optional Features"
            }
            _ => "see your package manager (winget, choco, scoop)",
        }
    } else {
        match bin {
            "git" => "brew install git  |  sudo apt-get install git",
            "rustc" | "cargo" => "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh",
            "ssh" => "sudo apt-get install openssh-client",
            _ => "see your package manager",
        }
    };
    utils::print_info(&format!("Fix: {}", hint));
}

fn check_internet() {
    // A simple TCP connect to a well-known DNS resolver on port 53.
    match std::net::TcpStream::connect_timeout(
        &"8.8.8.8:53".parse().unwrap(),
        std::time::Duration::from_secs(3),
    ) {
        Ok(_) => utils::print_ok("internet reachable"),
        Err(_) => utils::print_fail("internet not reachable"),
    }
}

fn check_workspace() {
    if find_existing_config_path().is_none() {
        utils::print_warn(&format!(
            "no workspace config found (tried: {})",
            known_config_paths().join(", ")
        ));
        return;
    }

    match WorkspaceConfig::load() {
        Err(e) => utils::print_fail(&format!("workspace config invalid: {}", e)),
        Ok(cfg) => {
            utils::print_ok(&format!("workspace config valid ({})", cfg.name));
            check_repos(&cfg);

            if cfg.vm.is_some() {
                utils::print_ok("vm config present");
            }
        }
    }
}

fn check_repos(cfg: &WorkspaceConfig) {
    for project in &cfg.projects {
        let path = std::path::Path::new(project.local_dir());
        if crate::git::is_repo(path) {
            utils::print_ok(&format!("repo '{}' present", project.name));
        } else if path.exists() {
            utils::print_fail(&format!("'{}' exists but is not a git repo", project.name));
        } else {
            utils::print_warn(&format!(
                "repo '{}' not cloned — run `polyws pull`",
                project.name
            ));
        }
    }
}

fn check_disk() {
    // Write a temporary probe file to confirm the CWD is writable.
    let probe = std::path::Path::new(".polyws_disk_probe");
    match std::fs::write(probe, b"probe") {
        Ok(_) => {
            let _ = std::fs::remove_file(probe);
            utils::print_ok("working directory is writable");
        }
        Err(e) => utils::print_fail(&format!("working directory not writable: {}", e)),
    }
}
