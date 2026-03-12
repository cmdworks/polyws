use anyhow::Result;
use clap::Parser;

mod cli;
mod config;
mod doctor;
mod exec;
mod git;
mod snapshot;
mod sync;
mod tui;
mod update;
mod utils;
mod vm;
mod workspace;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Cli::parse();

    match args.command {
        None => tui::run()?,
        Some(cmd) => match cmd {
            // ── workspace management ───────────────────────────────────────────
            cli::Commands::Init => workspace::init()?,
            cli::Commands::Add {
                name,
                url,
                branch,
                depends_on,
                sync_url,
            } => workspace::add(name, url, branch, depends_on, sync_url)?,
            cli::Commands::Remove { name } => workspace::remove(&name)?,
            cli::Commands::List => workspace::list()?,
            cli::Commands::Pull { name, force } => workspace::pull(name, force)?,
            cli::Commands::Clone { name, force } => workspace::clone_repos(name, force)?,
            cli::Commands::Push { name } => workspace::push(name)?,
            cli::Commands::Status => workspace::status()?,
            cli::Commands::Graph => workspace::graph()?,
            cli::Commands::Repair => workspace::repair()?,
            cli::Commands::Bootstrap => workspace::bootstrap().await?,

            // ── execution ─────────────────────────────────────────────────────
            cli::Commands::Exec { cmd } => exec::run(cmd).await?,

            // ── snapshots ─────────────────────────────────────────────────────
            cli::Commands::Snapshot { action } => match action {
                cli::SnapshotAction::Create => snapshot::create()?,
                cli::SnapshotAction::Restore { file, dry_run, yes } => {
                    snapshot::restore(&file, dry_run, yes)?
                }
                cli::SnapshotAction::List => snapshot::list()?,
            },

            // ── doctor ────────────────────────────────────────────────────────
            cli::Commands::Doctor => doctor::run().await?,

            // ── mirror sync ───────────────────────────────────────────────────
            cli::Commands::Sync { action } => match action {
                cli::SyncAction::Start => sync::start()?,
                cli::SyncAction::Stop => sync::stop()?,
                cli::SyncAction::Status => sync::status()?,
                cli::SyncAction::Now => sync::sync_now()?,
            },

            // ── VM ────────────────────────────────────────────────────────────
            cli::Commands::Vm { action } => {
                let cfg = config::WorkspaceConfig::load()
                    .map_err(|e| anyhow::anyhow!("Could not load workspace config: {}", e))?;
                let vm_cfg = cfg
                    .vm
                    .ok_or_else(|| anyhow::anyhow!("No [vm] section in .polyws"))?;

                match action {
                    cli::VmAction::Doctor => vm::doctor(&vm_cfg).await?,
                    cli::VmAction::Setup => vm::setup(&vm_cfg).await?,
                    cli::VmAction::SyncStart => vm::sync_start(&vm_cfg)?,
                    cli::VmAction::SyncStop => vm::sync_stop(&vm_cfg)?,
                    cli::VmAction::Exec { cmd } => vm::executor::exec_on_vm(&vm_cfg, &cmd)?,
                    cli::VmAction::Shell => vm::executor::open_shell(&vm_cfg)?,
                    cli::VmAction::Reset => vm::reset(&vm_cfg)?,
                }
            }

            // ── self-update ───────────────────────────────────────────────────
            cli::Commands::Update => update::run()?,

            // ── internal daemon (hidden from help) ────────────────────────────
            cli::Commands::SyncDaemon => sync::run_daemon().await?,
        },
    }

    Ok(())
}
