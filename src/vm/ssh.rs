use anyhow::{Context, Result};
use std::process::{Command, Output, Stdio};

/// A thin wrapper around the system `ssh` binary.
///
/// Uses `BatchMode=yes` for non-interactive commands so that scripts never
/// hang waiting for a password prompt.
pub struct SshSession {
    pub host: String,
    pub user: String,
}

impl SshSession {
    pub fn new(user: &str, host: &str) -> Self {
        Self {
            host: host.to_string(),
            user: user.to_string(),
        }
    }

    fn target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }

    /// Common SSH flags used for all non-interactive calls.
    fn base_args(&self) -> Vec<&'static str> {
        vec![
            "-o",
            "StrictHostKeyChecking=no",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
        ]
    }

    /// Run a command on the remote host and capture its output.
    pub fn exec(&self, cmd: &str) -> Result<Output> {
        let mut args = self.base_args();
        args.push(self.target().as_str());
        // Note: target() returns a temporary; we build owned args instead.
        let target = self.target();
        Command::new("ssh")
            .args(self.base_args())
            .arg(&target)
            .arg(cmd)
            .output()
            .with_context(|| format!("Failed to SSH to {}", target))
    }

    /// `cd <remote_path> && <cmd>` — run a command inside the workspace directory.
    pub fn exec_in_path(&self, remote_path: &str, cmd: &str) -> Result<Output> {
        // Use printf to avoid shell injection on the path; the path comes from
        // our own config, but we still quote it defensively.
        let full_cmd = format!("cd {} && {}", shell_escape(remote_path), cmd);
        self.exec(&full_cmd)
    }

    /// Open an interactive shell on the remote host (replaces the current process).
    pub fn interactive_shell(&self) -> Result<()> {
        let target = self.target();
        let status = Command::new("ssh")
            .args(["-o", "StrictHostKeyChecking=no"])
            .arg(&target)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("Failed to connect to {}", target))?;

        if !status.success() {
            anyhow::bail!("SSH session ended with a non-zero status");
        }
        Ok(())
    }

    /// Returns `true` when the host is reachable and SSH authentication succeeds.
    pub fn check_connectivity(&self) -> bool {
        let target = self.target();
        Command::new("ssh")
            .args([
                "-o",
                "StrictHostKeyChecking=no",
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=5",
            ])
            .arg(&target)
            .arg("exit 0")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Retrieve OS identification from the remote host.
    pub fn detect_os(&self) -> Result<String> {
        let output = self.exec("uname -s && cat /etc/os-release 2>/dev/null || true")?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Minimal single-argument shell escaping: wraps the value in single quotes
/// and escapes any embedded single quotes.
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
