use anyhow::{Context, Result};
use git2::{Repository, ResetType};
use std::path::Path;
use std::process::Command;

/// Clone a repository from `url` into `path`.
pub fn clone_repo(url: &str, path: &Path) -> Result<()> {
    println!("  Cloning {} → {}", url, path.display());
    let status = Command::new("git")
        .args(["clone", url, &path.to_string_lossy()])
        .status()
        .context("Failed to run git clone")?;
    if !status.success() {
        anyhow::bail!("git clone failed for {}", url);
    }
    Ok(())
}

/// Fetch all remotes then update the given branch for the repo at `path`.
///
/// Strategy:
///   1. `git fetch --all --prune`
///   2. In safe mode (default), skip dirty worktrees.
///   3. If behind remote: safe mode uses `git merge --ff-only`.
///      `--force` mode uses `git reset --hard origin/<branch>`.
///   4. If diverged: fail in safe mode; `--force` hard-resets.
pub fn pull_repo(path: &Path, branch: &str, force: bool) -> Result<()> {
    // 1. Fetch
    let fetch = Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(path)
        .status()
        .context("Failed to run git fetch")?;
    if !fetch.success() {
        anyhow::bail!("git fetch failed in {}", path.display());
    }

    let remote_ref = format!("origin/{}", branch);

    // 2. Check whether origin/<branch> even exists yet (new empty repo edge case).
    let remote_exists = Command::new("git")
        .args(["rev-parse", "--verify", &remote_ref])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !remote_exists {
        // Nothing to pull yet — remote branch doesn't exist (e.g. empty repo).
        return Ok(());
    }

    if !force && is_worktree_dirty(path)? {
        anyhow::bail!(
            "working tree has local changes; skipping safe pull (rerun with --force to discard changes)"
        );
    }

    // 3. Count commits that are local-only (ahead of remote).
    let ahead_output = Command::new("git")
        .args(["log", "--oneline", &format!("{}..HEAD", remote_ref)])
        .current_dir(path)
        .output()
        .context("Failed to check ahead commits")?;
    let local_only = String::from_utf8_lossy(&ahead_output.stdout);
    let ahead = local_only.lines().count();

    // 4. Count commits the remote is ahead of us (behind).
    let behind_output = Command::new("git")
        .args(["log", "--oneline", &format!("HEAD..{}", remote_ref)])
        .current_dir(path)
        .output()
        .context("Failed to check behind commits")?;
    let remote_only = String::from_utf8_lossy(&behind_output.stdout);
    let behind = remote_only.lines().count();

    match (ahead, behind) {
        (0, 0) => {
            // Already up to date — nothing to do.
        }
        (0, _) => {
            if force {
                hard_reset_to(path, &remote_ref)?;
            } else {
                let ff_merge = Command::new("git")
                    .args(["merge", "--ff-only", &remote_ref])
                    .current_dir(path)
                    .status()
                    .context("Failed to run git merge --ff-only")?;
                if !ff_merge.success() {
                    anyhow::bail!(
                        "fast-forward merge failed for '{}'; rerun with --force to hard reset to {}",
                        path.display(),
                        remote_ref
                    );
                }
            }
        }
        (_, 0) => {
            // Local branch is ahead only — nothing to pull.
        }
        (ahead, behind) => {
            if force {
                hard_reset_to(path, &remote_ref)?;
            } else {
                anyhow::bail!(
                    "branch '{}' has diverged: {} local commit(s) not on remote, \
                     {} remote commit(s) not local. Resolve manually or rerun with --force to hard reset",
                    branch,
                    ahead,
                    behind
                );
            }
        }
    }

    Ok(())
}

/// Push the local branch to origin for the repo at `path`.
pub fn push_repo(path: &Path, branch: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["push", "origin", branch])
        .current_dir(path)
        .status()
        .context("Failed to run git push")?;
    if !status.success() {
        anyhow::bail!("git push origin {} failed", branch);
    }
    Ok(())
}

fn hard_reset_to(path: &Path, remote_ref: &str) -> Result<()> {
    let reset = Command::new("git")
        .args(["reset", "--hard", remote_ref])
        .current_dir(path)
        .status()
        .context("Failed to run git reset --hard")?;
    if !reset.success() {
        anyhow::bail!("git reset --hard {} failed", remote_ref);
    }
    Ok(())
}

fn is_worktree_dirty(path: &Path) -> Result<bool> {
    let out = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(path)
        .output()
        .context("Failed to run git status --porcelain")?;
    if !out.status.success() {
        anyhow::bail!("git status failed in {}", path.display());
    }
    Ok(!String::from_utf8_lossy(&out.stdout).trim().is_empty())
}

/// Return the abbreviated HEAD commit hash (7 chars) for the repo at `path`.
pub fn get_commit_hash(path: &Path) -> Result<String> {
    let repo = Repository::open(path)
        .with_context(|| format!("Failed to open repository at {}", path.display()))?;
    let head = repo.head().context("Failed to get HEAD")?;
    let commit = head
        .peel_to_commit()
        .context("HEAD does not point at a commit")?;
    let full = commit.id().to_string();
    Ok(full[..7].to_string())
}

/// Hard-reset the repo at `path` to the given commit hash or ref.
pub fn checkout_commit(path: &Path, hash: &str) -> Result<()> {
    let repo = Repository::open(path)
        .with_context(|| format!("Failed to open repository at {}", path.display()))?;
    let obj = repo
        .revparse_single(hash)
        .with_context(|| format!("Commit '{}' not found", hash))?;
    repo.reset(&obj, ResetType::Hard, None)
        .with_context(|| format!("Failed to reset to '{}'", hash))?;
    Ok(())
}

/// Push all refs to `mirror_url` using `git push --mirror`.
pub fn push_mirror(path: &Path, mirror_url: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["push", "--mirror", mirror_url])
        .current_dir(path)
        .status()
        .context("Failed to run git push --mirror")?;
    if !status.success() {
        anyhow::bail!("git push --mirror failed for {}", mirror_url);
    }
    Ok(())
}

/// Returns a human-readable status string for the repo at `path`.
/// Format: `<branch> (clean)` or `<branch> (<n> modified)`.
pub fn repo_status(path: &Path) -> Result<String> {
    let repo = Repository::open(path)
        .with_context(|| format!("Not a git repository: {}", path.display()))?;

    let statuses = repo
        .statuses(None)
        .context("Failed to get repository status")?;

    let dirty = statuses
        .iter()
        .filter(|s| s.status() != git2::Status::CURRENT)
        .count();

    let branch = repo
        .head()
        .ok()
        .as_ref()
        .and_then(|h| h.shorthand().map(|s| s.to_string()))
        .unwrap_or_else(|| "HEAD detached".to_string());

    if dirty == 0 {
        Ok(format!("{} (clean)", branch))
    } else {
        Ok(format!("{} ({} modified)", branch, dirty))
    }
}

/// Returns `true` when `path` is a valid git repository.
pub fn is_repo(path: &Path) -> bool {
    Repository::open(path).is_ok()
}

/// Returns the configured remote URL for `origin`, or an error.
pub fn get_remote_url(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()
        .context("Failed to run git remote get-url")?;
    if !output.status.success() {
        anyhow::bail!("Could not get remote URL for {}", path.display());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
