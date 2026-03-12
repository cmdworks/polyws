pub mod detect_os;
pub mod executor;
pub mod installer;
pub mod ssh;

use anyhow::{Context, Result};
use std::process::Command;

use crate::config::{VmConfig, WorkspaceConfig};
use crate::utils;
use ssh::SshSession;

// ---------------------------------------------------------------------------
// vm doctor
// ---------------------------------------------------------------------------

pub async fn doctor(vm: &VmConfig) -> Result<()> {
    utils::print_section("VM Doctor");

    let ssh = SshSession::new(&vm.user, &vm.host);

    if !ssh.check_connectivity() {
        utils::print_fail(&format!("Cannot connect to {}@{}", vm.user, vm.host));
        utils::print_info("Ensure SSH keys are loaded: ssh-add ~/.ssh/id_rsa");
        return Ok(());
    }
    utils::print_ok(&format!("connected to {}@{}", vm.user, vm.host));

    let os_info = ssh.detect_os().unwrap_or_default();
    let os = detect_os::detect(&os_info);
    utils::print_ok(&format!("OS: {}", os));

    utils::print_section("VM Dependencies");
    for dep in &vm.dependencies {
        if installer::check_dependency(&ssh, dep) {
            utils::print_ok(&format!("{} available", dep));
        } else {
            utils::print_fail(&format!("{} not installed", dep));
        }
    }

    utils::print_section("VM Workspace");
    let path_check = ssh.exec(&format!("test -d {} && echo ok", vm.path));
    match path_check {
        Ok(o) if o.status.success() => {
            utils::print_ok(&format!("workspace directory exists: {}", vm.path))
        }
        _ => utils::print_warn(&format!(
            "workspace directory missing: {} — run `polyws vm setup`",
            vm.path
        )),
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// vm setup
// ---------------------------------------------------------------------------

pub async fn setup(vm: &VmConfig) -> Result<()> {
    utils::print_section("VM Setup");

    let ssh = SshSession::new(&vm.user, &vm.host);

    if !ssh.check_connectivity() {
        anyhow::bail!("Cannot connect to {}@{}", vm.user, vm.host);
    }
    utils::print_ok(&format!("connected to {}@{}", vm.user, vm.host));

    let os_info = ssh.detect_os().unwrap_or_default();
    let os = detect_os::detect(&os_info);
    utils::print_info(&format!("detected OS: {}", os));

    utils::print_section("Package Lists");
    installer::update_packages(&ssh, &os)?;

    utils::print_section("Dependencies");
    for dep in &vm.dependencies {
        if installer::check_dependency(&ssh, dep) {
            utils::print_ok(&format!("{} already installed", dep));
        } else {
            utils::print_info(&format!("installing {}…", dep));
            installer::install_dependency(&ssh, &os, dep)?;
        }
    }

    utils::print_section("Workspace Directory");
    let mkdir = ssh.exec(&format!("mkdir -p {}", vm.path))?;
    if mkdir.status.success() {
        utils::print_ok(&format!("workspace directory ready: {}", vm.path));
    } else {
        utils::print_fail(&format!("could not create directory: {}", vm.path));
    }

    utils::print_section("Git Config");
    let _ = ssh.exec("git config --global init.defaultBranch main 2>/dev/null || true");
    utils::print_ok("git configured");

    Ok(())
}

// ---------------------------------------------------------------------------
// vm sync start / stop
// ---------------------------------------------------------------------------

pub fn sync_start(vm: &VmConfig) -> Result<()> {
    match vm.sync.as_str() {
        "mutagen" => start_mutagen(vm),
        _ => start_rsync(vm),
    }
}

fn start_mutagen(vm: &VmConfig) -> Result<()> {
    let config = WorkspaceConfig::load()?;
    let remote = format!("{}@{}:{}", vm.user, vm.host, vm.path);

    let status = Command::new("mutagen")
        .args(["sync", "create", "--name", &config.name, ".", &remote])
        .status()
        .context("Failed to run mutagen (is it installed?)")?;

    if status.success() {
        utils::print_ok("mutagen sync session started");
    } else {
        utils::print_fail("mutagen sync start failed");
    }
    Ok(())
}

fn start_rsync(vm: &VmConfig) -> Result<()> {
    utils::print_info("rsync is one-shot; syncing now…");
    let remote = format!("{}@{}:{}", vm.user, vm.host, vm.path);

    let status = Command::new("rsync")
        .args([
            "-avz",
            "--exclude=target/",
            "--exclude=.git/",
            "--exclude=.polyws/",
            "./",
            &remote,
        ])
        .status()
        .context("Failed to run rsync (is it installed?)")?;

    if status.success() {
        utils::print_ok("rsync completed");
    } else {
        utils::print_fail("rsync failed");
    }
    Ok(())
}

pub fn sync_stop(vm: &VmConfig) -> Result<()> {
    match vm.sync.as_str() {
        "mutagen" => {
            let config = WorkspaceConfig::load()?;
            let status = Command::new("mutagen")
                .args(["sync", "terminate", &config.name])
                .status()
                .context("Failed to run mutagen")?;

            if status.success() {
                utils::print_ok("mutagen sync session stopped");
            } else {
                utils::print_fail("failed to stop mutagen session");
            }
        }
        _ => {
            utils::print_warn("No persistent sync session to stop for rsync mode.");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// vm reset
// ---------------------------------------------------------------------------

pub fn reset(vm: &VmConfig) -> Result<()> {
    println!(
        "\x1b[31mWARNING:\x1b[0m This will permanently delete the remote workspace at\n  {}:{}\n",
        vm.host, vm.path
    );
    println!("Press Ctrl-C to cancel.  Proceeding in 5 seconds…");
    std::thread::sleep(std::time::Duration::from_secs(5));

    let ssh = SshSession::new(&vm.user, &vm.host);
    let cmd = format!("rm -rf {path} && mkdir -p {path}", path = vm.path);
    let output = ssh.exec(&cmd)?;

    if output.status.success() {
        utils::print_ok("VM workspace reset");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        utils::print_fail(&format!("Reset failed: {}", stderr.trim()));
    }
    Ok(())
}
