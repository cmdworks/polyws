/// Self-update: downloads the latest GitHub release binary and replaces this executable.
use anyhow::{Context, Result};
use std::process::Command;

const REPO: &str = "cmdworks/polyws";

pub fn run() -> Result<()> {
    let platform = detect_platform()?;
    let version = resolve_latest_version()?;

    println!("▸ Platform  : {}", platform);
    println!("▸ Latest    : {}", version);

    let current_version = concat!("v", env!("CARGO_PKG_VERSION"));
    if version == current_version {
        println!("✔ Already on the latest version ({})", current_version);
        return Ok(());
    }
    println!("▸ Upgrading {} → {}…", current_version, version);

    let url = format!(
        "https://github.com/{}/releases/download/{}/polyws-{}.tar.gz",
        REPO, version, platform
    );

    // ── temp dir ─────────────────────────────────────────
    let tmpdir = std::env::temp_dir().join(format!("polyws-update-{}", std::process::id()));
    std::fs::create_dir_all(&tmpdir).context("Failed to create temp dir")?;
    let archive = tmpdir.join("polyws.tar.gz");

    // ── download ─────────────────────────────────────────
    println!("▸ Downloading {}…", url);
    let status = Command::new("curl")
        .args(["-fsSL", "--progress-bar", "-o"])
        .arg(&archive)
        .arg(&url)
        .status()
        .context("curl is required for self-update")?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmpdir);
        anyhow::bail!("Download failed — check the URL and your internet connection");
    }

    // ── extract ───────────────────────────────────────────
    let status = Command::new("tar")
        .args(["-xzf"])
        .arg(&archive)
        .arg("-C")
        .arg(&tmpdir)
        .status()
        .context("tar is required for self-update")?;

    if !status.success() {
        let _ = std::fs::remove_dir_all(&tmpdir);
        anyhow::bail!("Extraction failed");
    }

    let new_bin = tmpdir.join("polyws");
    if !new_bin.exists() {
        let _ = std::fs::remove_dir_all(&tmpdir);
        anyhow::bail!("Extracted archive did not contain a 'polyws' binary");
    }

    // ── make executable (Unix) ────────────────────────────
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&new_bin)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&new_bin, perms)?;
    }

    // ── atomic replace ────────────────────────────────────
    let current_exe =
        std::env::current_exe().context("Cannot determine current executable path")?;

    // Copy to a sibling path then rename (atomic on Unix)
    let staged = current_exe.with_extension("new");
    std::fs::copy(&new_bin, &staged)
        .with_context(|| format!("Cannot write to {:?} — try running with sudo", staged))?;
    std::fs::rename(&staged, &current_exe)
        .with_context(|| format!("Cannot replace {:?} — try running with sudo", current_exe))?;

    let _ = std::fs::remove_dir_all(&tmpdir);

    println!("✔ polyws updated to {}", version);
    Ok(())
}

fn detect_platform() -> Result<String> {
    let os = match std::env::consts::OS {
        "linux" => "unknown-linux-gnu",
        "macos" => "apple-darwin",
        other => anyhow::bail!("Unsupported OS for self-update: {}", other),
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        other => anyhow::bail!("Unsupported architecture for self-update: {}", other),
    };
    Ok(format!("{}-{}", arch, os))
}

fn resolve_latest_version() -> Result<String> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", REPO);
    let output = Command::new("curl")
        .args(["-fsSL", &url])
        .output()
        .context("curl is required for self-update")?;
    if !output.status.success() {
        anyhow::bail!("Failed to fetch latest release info from GitHub");
    }
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("Invalid JSON from GitHub API")?;
    let tag = json["tag_name"]
        .as_str()
        .context("No tag_name field in GitHub API response")?
        .to_string();
    Ok(tag)
}
