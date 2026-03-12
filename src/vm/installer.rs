use anyhow::Result;

use crate::utils;
use crate::vm::detect_os::OsType;
use crate::vm::ssh::SshSession;

/// Check whether `dep` is on the remote `$PATH`.
pub fn check_dependency(ssh: &SshSession, dep: &str) -> bool {
    let cmd = format!("command -v {} >/dev/null 2>&1", dep);
    ssh.exec(&cmd).map(|o| o.status.success()).unwrap_or(false)
}

/// Update the remote package index for the detected OS.
pub fn update_packages(ssh: &SshSession, os: &OsType) -> Result<()> {
    let cmd = match os {
        OsType::Ubuntu | OsType::Debian => "sudo apt-get update -y -q",
        OsType::Arch => "sudo pacman -Sy --noconfirm",
        OsType::MacOs => "brew update",
        OsType::Unknown(_) => return Ok(()),
    };

    match ssh.exec(cmd) {
        Ok(o) if o.status.success() => utils::print_ok("package lists updated"),
        Ok(_) => utils::print_warn("package update finished with warnings"),
        Err(e) => utils::print_warn(&format!("package update skipped: {}", e)),
    }
    Ok(())
}

/// Install a single dependency on the remote host using the OS package manager.
///
/// `dep` must be a simple package name (no spaces or shell metacharacters).
pub fn install_dependency(ssh: &SshSession, os: &OsType, dep: &str) -> Result<()> {
    // Validate that dep contains only safe characters to prevent injection.
    if !dep
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        anyhow::bail!("Unsafe dependency name: '{}'", dep);
    }

    let cmd = match os {
        OsType::Ubuntu | OsType::Debian => {
            format!("sudo apt-get install -y -q {}", dep)
        }
        OsType::Arch => {
            format!("sudo pacman -S --noconfirm {}", dep)
        }
        OsType::MacOs => {
            format!("brew install {}", dep)
        }
        OsType::Unknown(name) => {
            anyhow::bail!("Cannot install '{}': unsupported OS '{}'", dep, name);
        }
    };

    let output = ssh.exec(&cmd)?;
    if output.status.success() {
        utils::print_ok(&format!("{} installed", dep));
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        utils::print_fail(&format!("Failed to install '{}': {}", dep, stderr.trim()));
    }
    Ok(())
}
