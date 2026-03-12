use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "polyws", version, about = "Polyrepo workspace orchestrator")]
pub struct Cli {
    /// Run the interactive TUI when no subcommand is given
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new workspace in the current directory
    Init,
    /// Add a project repository to the workspace
    Add {
        /// Project name (identifier used in dependencies)
        name: String,
        /// Local directory path relative to workspace root (defaults to name)
        #[arg(long)]
        path: Option<String>,
        /// Git clone URL
        url: String,
        /// Branch to track
        #[arg(long, default_value = "main")]
        branch: String,
        /// Dependencies (repeat flag or pass comma-separated names)
        #[arg(long, value_delimiter = ',')]
        depends_on: Vec<String>,
        /// Mirror/backup remote URL for sync daemon
        #[arg(long)]
        sync_url: Option<String>,
    },
    /// Remove a project from the workspace config
    Remove {
        /// Project name to remove
        name: String,
    },
    /// List all projects in the workspace
    List,
    /// Clone missing repos and pull existing ones
    Pull {
        /// Operate on a specific project only
        name: Option<String>,
        /// Allow destructive hard reset when needed (discarding tracked changes/commits)
        #[arg(long)]
        force: bool,
    },
    /// Clone workspace repositories (alias for pull)
    Clone {
        /// Operate on a specific project only
        name: Option<String>,
        /// Allow destructive hard reset when needed (discarding tracked changes/commits)
        #[arg(long)]
        force: bool,
    },
    /// Push local commits to origin for workspace repositories
    Push {
        /// Operate on a specific project only
        name: Option<String>,
    },
    /// Show git status of all repositories
    Status,
    /// Execute a shell command across all repositories
    Exec {
        /// Shell command to run
        cmd: String,
    },
    /// Visualise the project dependency graph
    Graph,
    /// Manage workspace snapshots
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },
    /// Validate the development environment
    Doctor,
    /// Repair a broken workspace
    Repair,
    /// Run doctor then pull all repos (quick-start)
    Bootstrap,
    /// Manage the mirror-sync daemon
    Sync {
        #[command(subcommand)]
        action: SyncAction,
    },
    /// Manage the remote development VM
    Vm {
        #[command(subcommand)]
        action: VmAction,
    },
    /// Update polyws to the latest release
    Update,

    /// Internal: run the sync daemon loop (do not call directly)
    #[command(hide = true)]
    SyncDaemon,
}

#[derive(Subcommand, Debug)]
pub enum SnapshotAction {
    /// Capture the current commit of every repository
    Create,
    /// Restore repositories to a saved snapshot
    Restore {
        /// Path to the snapshot JSON file
        file: String,
        /// Show what would change without resetting repositories
        #[arg(long)]
        dry_run: bool,
        /// Skip interactive confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },
    /// List available snapshots
    List,
}

#[derive(Subcommand, Debug)]
pub enum SyncAction {
    /// Start the background mirror-sync daemon
    Start,
    /// Stop the background mirror-sync daemon
    Stop,
    /// Show whether the daemon is running
    Status,
    /// Run a mirror sync immediately (one-shot)
    Now,
}

#[derive(Subcommand, Debug)]
pub enum VmAction {
    /// Check the VM environment and installed dependencies
    Doctor,
    /// Bootstrap the VM with required dependencies and workspace directory
    Setup,
    /// Start continuous file synchronisation to the VM
    SyncStart,
    /// Stop continuous file synchronisation to the VM
    SyncStop,
    /// Execute a command on the VM inside the workspace path
    Exec {
        /// Command to run remotely
        cmd: String,
    },
    /// Open an interactive SSH shell on the VM
    Shell,
    /// Delete and recreate the remote workspace directory
    Reset,
}
