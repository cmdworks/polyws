use anyhow::{Context, Result};
use chrono::Local;
use git2::{Repository, ResetType};
use std::path::Path;
use std::process::{Command, Output};

/// Clone a repository from `url` into `path`.
pub fn clone_repo(url: &str, path: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["clone", url, &path.to_string_lossy()])
        .output()
        .context("Failed to run git clone")?;
    if !output.status.success() {
        anyhow::bail!(
            "git clone failed for {}: {}",
            url,
            summarize_git_failure(&output)
        );
    }
    Ok(())
}

/// Initialize an existing directory as a git repo and sync from origin.
/// Useful when the target directory already exists (e.g. workspace root).
pub fn init_repo_in_dir(url: &str, branch: &str, path: &Path) -> Result<()> {
    let init = Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .context("Failed to run git init")?;
    if !init.status.success() {
        anyhow::bail!(
            "git init failed in {}: {}",
            path.display(),
            summarize_git_failure(&init)
        );
    }

    // Ensure origin points to the right URL (add or set).
    let origin_ok = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let remote_cmd = if origin_ok {
        Command::new("git")
            .args(["remote", "set-url", "origin", url])
            .current_dir(path)
            .output()
    } else {
        Command::new("git")
            .args(["remote", "add", "origin", url])
            .current_dir(path)
            .output()
    }
    .context("Failed to configure git remote")?;

    if !remote_cmd.status.success() {
        anyhow::bail!(
            "git remote setup failed in {}: {}",
            path.display(),
            summarize_git_failure(&remote_cmd)
        );
    }

    let fetch = Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(path)
        .output()
        .context("Failed to run git fetch")?;
    if !fetch.status.success() {
        anyhow::bail!(
            "git fetch failed in {}: {}",
            path.display(),
            summarize_git_failure(&fetch)
        );
    }

    let remote_ref = format!("origin/{}", branch);
    let remote_exists = Command::new("git")
        .args(["rev-parse", "--verify", &remote_ref])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !remote_exists {
        anyhow::bail!("remote branch '{}' not found for {}", branch, url);
    }

    let checkout = Command::new("git")
        .args(["checkout", "-B", branch, &remote_ref])
        .current_dir(path)
        .output()
        .context("Failed to run git checkout")?;
    if !checkout.status.success() {
        anyhow::bail!(
            "git checkout failed in {}: {}",
            path.display(),
            summarize_git_failure(&checkout)
        );
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
        .output()
        .context("Failed to run git fetch")?;
    if !fetch.status.success() {
        anyhow::bail!(
            "git fetch failed in {}: {}",
            path.display(),
            summarize_git_failure(&fetch)
        );
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
                    .output()
                    .context("Failed to run git merge --ff-only")?;
                if !ff_merge.status.success() {
                    anyhow::bail!(
                        "fast-forward merge failed for '{}': {}. Rerun with --force to hard reset to {}",
                        path.display(),
                        summarize_git_failure(&ff_merge),
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

/// Returns true if the repo has uncommitted changes.
pub fn has_uncommitted_changes(path: &Path) -> Result<bool> {
    is_worktree_dirty(path)
}

/// Stage all changes and commit with the given message.
pub fn commit_all(path: &Path, message: &str) -> Result<()> {
    let add = Command::new("git")
        .args(["add", "-A"])
        .current_dir(path)
        .output()
        .context("Failed to run git add -A")?;
    if !add.status.success() {
        anyhow::bail!(
            "git add failed in {}: {}",
            path.display(),
            summarize_git_failure(&add)
        );
    }

    if !is_worktree_dirty(path)? {
        anyhow::bail!("no changes to commit");
    }

    let commit = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(path)
        .output()
        .context("Failed to run git commit")?;
    if !commit.status.success() {
        anyhow::bail!(
            "git commit failed in {}: {}",
            path.display(),
            summarize_git_failure(&commit)
        );
    }
    Ok(())
}

/// Push the local branch to origin for the repo at `path`.
pub fn push_repo(path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "origin", branch])
        .current_dir(path)
        .output()
        .context("Failed to run git push")?;
    if !output.status.success() {
        anyhow::bail!(
            "git push origin {} failed: {}",
            branch,
            summarize_git_failure(&output)
        );
    }
    Ok(())
}

/// Push a dedicated `sync` branch to the given sync remote.
/// The sync branch is recreated from the base branch and includes uncommitted
/// local changes (committed with a standard message).
pub fn push_sync_branch(path: &Path, base_branch: &str, sync_url: &str) -> Result<()> {
    let head_branch = current_branch(path)?;
    let head_commit = rev_parse(path, "HEAD")?;
    let base_ref = if ref_exists(path, &format!("refs/heads/{}", base_branch)) {
        base_branch.to_string()
    } else if ref_exists(path, &format!("refs/remotes/origin/{}", base_branch)) {
        format!("origin/{}", base_branch)
    } else {
        return Err(anyhow::anyhow!(
            "base branch '{}' not found (local or origin)",
            base_branch
        ));
    };

    let dirty = is_worktree_dirty(path)?;
    let mut stashed = false;

    if dirty {
        let stash = Command::new("git")
            .args(["stash", "push", "-u", "-m", "polyws sync"])
            .current_dir(path)
            .output()
            .context("Failed to run git stash")?;
        if !stash.status.success() {
            anyhow::bail!(
                "git stash failed in {}: {}",
                path.display(),
                summarize_git_failure(&stash)
            );
        }
        stashed = true;
    }

    let sync_result = (|| -> Result<()> {
        let checkout = Command::new("git")
            .args(["checkout", "-B", "sync", &base_ref])
            .current_dir(path)
            .output()
            .context("Failed to checkout sync branch")?;
        if !checkout.status.success() {
            anyhow::bail!(
                "git checkout sync failed in {}: {}",
                path.display(),
                summarize_git_failure(&checkout)
            );
        }

        if stashed {
            let apply = Command::new("git")
                .args(["stash", "apply"])
                .current_dir(path)
                .output()
                .context("Failed to apply stash")?;
            if !apply.status.success() {
                anyhow::bail!(
                    "git stash apply failed in {}: {}",
                    path.display(),
                    summarize_git_failure(&apply)
                );
            }
        }

        if is_worktree_dirty(path)? {
            let add = Command::new("git")
                .args(["add", "-A"])
                .current_dir(path)
                .output()
                .context("Failed to run git add -A")?;
            if !add.status.success() {
                anyhow::bail!(
                    "git add failed in {}: {}",
                    path.display(),
                    summarize_git_failure(&add)
                );
            }

            let ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
            let sync_msg = format!("polyws sync {}", ts);
            let commit = Command::new("git")
                .args(["commit", "-m", &sync_msg])
                .current_dir(path)
                .output()
                .context("Failed to run git commit")?;
            if !commit.status.success() {
                anyhow::bail!(
                    "git commit failed in {}: {}",
                    path.display(),
                    summarize_git_failure(&commit)
                );
            }
        }

        let push = Command::new("git")
            .args(["push", "--force", sync_url, "sync:sync"])
            .current_dir(path)
            .output()
            .context("Failed to run git push for sync")?;
        if !push.status.success() {
            anyhow::bail!(
                "git push sync failed for {}: {}",
                sync_url,
                summarize_git_failure(&push)
            );
        }

        Ok(())
    })();

    let mut cleanup_errors = Vec::new();

    let checkout_back = if head_branch == "HEAD" {
        Command::new("git")
            .args(["checkout", "--detach", &head_commit])
            .current_dir(path)
            .output()
    } else {
        Command::new("git")
            .args(["checkout", &head_branch])
            .current_dir(path)
            .output()
    }
    .context("Failed to restore original branch")?;
    if !checkout_back.status.success() {
        cleanup_errors.push(format!(
            "restore branch failed: {}",
            summarize_git_failure(&checkout_back)
        ));
    }

    if stashed {
        let pop = Command::new("git")
            .args(["stash", "pop"])
            .current_dir(path)
            .output()
            .context("Failed to pop stash")?;
        if !pop.status.success() {
            cleanup_errors.push(format!("stash pop failed: {}", summarize_git_failure(&pop)));
        }
    }

    if !cleanup_errors.is_empty() {
        let cleanup = cleanup_errors.join("; ");
        if let Err(err) = sync_result {
            anyhow::bail!("sync failed: {} | cleanup failed: {}", err, cleanup);
        } else {
            anyhow::bail!("sync cleanup failed: {}", cleanup);
        }
    }

    sync_result
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

fn current_branch(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(path)
        .output()
        .context("Failed to read current branch")?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse HEAD failed in {}: {}",
            path.display(),
            summarize_git_failure(&output)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn rev_parse(path: &Path, rev: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", rev])
        .current_dir(path)
        .output()
        .context("Failed to run git rev-parse")?;
    if !output.status.success() {
        anyhow::bail!(
            "git rev-parse {} failed in {}: {}",
            rev,
            path.display(),
            summarize_git_failure(&output)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn ref_exists(path: &Path, reference: &str) -> bool {
    Command::new("git")
        .args(["rev-parse", "--verify", reference])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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

fn summarize_git_failure(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let mut message = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!(
            "git exited with status {}",
            output.status.code().unwrap_or(-1)
        )
    };

    if let Some(hint) = classify_git_hint(&message) {
        message.push_str(" | Hint: ");
        message.push_str(hint);
    }
    message
}

fn classify_git_hint(message: &str) -> Option<&'static str> {
    let lower = message.to_ascii_lowercase();
    if lower.contains("permission denied (publickey)") {
        return Some(
            "SSH auth failed. Check your SSH key, ssh-agent, and host alias in ~/.ssh/config.",
        );
    }
    if lower.contains("authentication failed") {
        return Some("Authentication failed. Verify HTTPS credentials/token or SSH key access.");
    }
    if lower.contains("repository not found") {
        return Some(
            "Repository was not found or access is missing. Verify repo URL and permissions.",
        );
    }
    if lower.contains("could not resolve host")
        || lower.contains("name or service not known")
        || lower.contains("temporary failure in name resolution")
    {
        return Some("Host lookup failed. Check repository URL/host alias and network DNS.");
    }
    if lower.contains("connection timed out") || lower.contains("connection refused") {
        return Some("Connection failed. Check network/VPN/firewall and remote availability.");
    }
    None
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

/// Push the local branch to origin with `--force`.
pub fn force_push_repo(path: &Path, branch: &str) -> Result<()> {
    let output = Command::new("git")
        .args(["push", "--force", "origin", branch])
        .current_dir(path)
        .output()
        .context("Failed to run git push --force")?;
    if !output.status.success() {
        anyhow::bail!(
            "git push --force origin {} failed: {}",
            branch,
            summarize_git_failure(&output)
        );
    }
    Ok(())
}

/// Commit all pending changes with an auto-generated timestamp message and
/// force-push to origin.  Useful as an instant snapshot push.
pub fn flush_repo(path: &Path, branch: &str) -> Result<()> {
    if is_worktree_dirty(path)? {
        let ts = Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let msg = format!("polyws flush {}", ts);
        commit_all(path, &msg)?;
    }
    force_push_repo(path, branch)
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
