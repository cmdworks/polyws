use std::fs;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use ratatui::widgets::TableState;

use crate::config::{normalize_local_dir, Project, WorkspaceConfig};
use crate::{git, snapshot, sync as sync_mod, utils};

use super::helpers::list_snap_files;
use super::types::{AddForm, Tab};

pub(super) struct App {
    pub(super) tab: Tab,
    pub(super) table_state: TableState, // Projects tab
    pub(super) snap_state: TableState,  // Snapshots tab
    pub(super) config: Option<WorkspaceConfig>,
    pub(super) repo_statuses: Vec<String>, // one per project, filled lazily
    pub(super) snap_files: Vec<String>,    // snapshot filenames
    pub(super) log_lines: Vec<String>,     // doctor / exec output
    pub(super) sync_running: bool,
    pub(super) show_add_form: bool,
    pub(super) show_help: bool,
    pub(super) add_form: AddForm,
    pub(super) show_exec_prompt: bool,
    pub(super) exec_input: String,
    pub(super) show_commit_prompt: bool,
    pub(super) commit_input: String,
    pub(super) status_msg: Option<(String, bool)>, // (msg, is_error)
    pub(super) last_tick: Instant,
    pub(super) confirm_delete: bool,
    pub(super) confirm_delete_name: Option<String>,
    pub(super) confirm_delete_path: Option<String>,
    pub(super) task_running: bool,
    pub(super) task_label: Option<String>,
    task_rx: Option<Receiver<TaskMsg>>,
    pending_push: Option<PushScope>,
}

enum TaskMsg {
    Log(String),
    Done { ok: bool, msg: String },
}

enum PushScope {
    All,
    Selected(Project),
}

impl App {
    pub(super) fn new() -> Self {
        let mut t = TableState::default();
        t.select(Some(0));
        let mut s = TableState::default();
        s.select(Some(0));

        let config = WorkspaceConfig::load().ok();
        let n = config.as_ref().map(|c| c.projects.len()).unwrap_or(0);
        let repo_statuses = vec!["…".to_string(); n];

        let snap_files = list_snap_files();

        App {
            tab: Tab::Dashboard,
            table_state: t,
            snap_state: s,
            config,
            repo_statuses,
            snap_files,
            log_lines: Vec::new(),
            sync_running: sync_mod::is_daemon_running(),
            show_add_form: false,
            show_help: false,
            add_form: AddForm::default(),
            show_exec_prompt: false,
            exec_input: String::new(),
            show_commit_prompt: false,
            commit_input: String::new(),
            status_msg: None,
            last_tick: Instant::now(),
            confirm_delete: false,
            confirm_delete_name: None,
            confirm_delete_path: None,
            task_running: false,
            task_label: None,
            task_rx: None,
            pending_push: None,
        }
    }

    pub(super) fn tick(&mut self) {
        if let Some(rx) = &self.task_rx {
            loop {
                match rx.try_recv() {
                    Ok(TaskMsg::Log(line)) => self.log_lines.push(line),
                    Ok(TaskMsg::Done { ok, msg }) => {
                        self.task_running = false;
                        self.task_label = None;
                        self.task_rx = None;
                        self.refresh_statuses();
                        self.set_status(msg, !ok);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.task_running = false;
                        self.task_label = None;
                        self.task_rx = None;
                        self.set_status("Background task ended".to_string(), false);
                        break;
                    }
                }
            }
        }
    }

    pub(super) fn reload(&mut self) {
        self.config = WorkspaceConfig::load().ok();
        let n = self.config.as_ref().map(|c| c.projects.len()).unwrap_or(0);
        self.repo_statuses = vec!["…".to_string(); n];
        self.snap_files = list_snap_files();
        self.sync_running = sync_mod::is_daemon_running();
        self.set_status("Reloaded workspace config".to_string(), false);
    }

    pub(super) fn refresh_statuses(&mut self) {
        if let Some(cfg) = &self.config {
            self.repo_statuses = cfg
                .projects
                .iter()
                .map(|p| {
                    let path = Path::new(p.local_dir());
                    if !path.exists() {
                        "\x1b[31mmissing\x1b[0m".to_string()
                    } else {
                        git::repo_status(path).unwrap_or_else(|_| "?".to_string())
                    }
                })
                .collect();
        }
    }

    pub(super) fn set_status(&mut self, msg: String, error: bool) {
        self.status_msg = Some((msg, error));
        self.last_tick = Instant::now();
    }

    pub(super) fn open_add_form(&mut self) {
        if self.config.is_none() {
            self.set_status(
                "No workspace loaded. Press  i  to initialize first.".to_string(),
                true,
            );
            return;
        }
        self.show_help = false;
        self.show_add_form = true;
        self.add_form = Default::default();
    }

    pub(super) fn selected_project(&self) -> Option<&Project> {
        let cfg = self.config.as_ref()?;
        let i = self.table_state.selected()?;
        cfg.projects.get(i)
    }

    pub(super) fn pull_selected(&mut self) {
        self.pull_selected_with_force(false);
    }

    pub(super) fn pull_selected_with_force(&mut self, force: bool) {
        if self.task_running {
            self.set_status("Another task is already running".to_string(), true);
            return;
        }
        if let Some(p) = self.selected_project() {
            let project = p.clone();
            self.start_pull_task(
                if force {
                    "pull-selected --force".to_string()
                } else {
                    "pull-selected".to_string()
                },
                project.name.clone(),
                move || vec![project],
                force,
            );
        }
    }

    pub(super) fn remove_selected(&mut self) {
        if let Some(p) = self.selected_project() {
            let name = p.name.clone();
            if let Some(cfg) = self.config.as_mut() {
                cfg.projects.retain(|x| x.name != name);
                if let Err(e) = cfg.save() {
                    self.set_status(format!("Save failed: {}", e), true);
                    return;
                }
            }

            let len = self.config.as_ref().map(|c| c.projects.len()).unwrap_or(0);
            let sel = self.table_state.selected().unwrap_or(0);
            if len == 0 {
                self.table_state.select(None);
            } else {
                self.table_state.select(Some(sel.min(len - 1)));
            }
            self.repo_statuses = vec!["…".to_string(); len];
            self.set_status(format!("Removed '{}'", name), false);
        }
    }

    pub(super) fn request_delete_selected(&mut self) {
        if let Some(p) = self.selected_project() {
            let name = p.name.clone();
            let local_dir = p.local_dir().to_string();
            let normalized = normalize_local_dir(&local_dir);
            if normalized == "." {
                self.set_status(
                    "Refusing to delete workspace root (path '.')".to_string(),
                    true,
                );
                return;
            }
            if !Path::new(&local_dir).exists() {
                self.set_status(format!("Local copy for '{}' not found", name), true);
                return;
            }
            self.confirm_delete = true;
            self.confirm_delete_name = Some(name.clone());
            self.confirm_delete_path = Some(local_dir);
            self.set_status(
                format!(
                    "Delete local copy of '{}' ? Press y to confirm, n/Esc to cancel",
                    name
                ),
                true,
            );
        }
    }

    pub(super) fn cancel_delete(&mut self) {
        self.confirm_delete = false;
        self.confirm_delete_name = None;
        self.confirm_delete_path = None;
        self.set_status("Delete cancelled".to_string(), false);
    }

    pub(super) fn confirm_delete(&mut self) {
        if !self.confirm_delete {
            return;
        }
        let name = match self.confirm_delete_name.clone() {
            Some(n) => n,
            None => {
                self.cancel_delete();
                return;
            }
        };
        let path = match self.confirm_delete_path.clone() {
            Some(p) => p,
            None => {
                self.cancel_delete();
                return;
            }
        };

        match fs::remove_dir_all(&path) {
            Ok(_) => {
                self.set_status(format!("Deleted local copy of '{}'", name), false);
                self.refresh_statuses();
            }
            Err(e) => self.set_status(format!("Delete failed for '{}': {}", name, e), true),
        }

        self.confirm_delete = false;
        self.confirm_delete_name = None;
        self.confirm_delete_path = None;
    }

    fn start_pull_task<F>(&mut self, label: String, display: String, projects_fn: F, force: bool)
    where
        F: FnOnce() -> Vec<Project> + Send + 'static,
    {
        self.log_lines.clear();
        self.tab = Tab::Logs;
        self.task_running = true;
        self.task_label = Some(label.clone());

        let (tx, rx) = mpsc::channel();
        self.task_rx = Some(rx);
        self.set_status(format!("{} started — see Logs tab", label), false);

        thread::spawn(move || {
            let projects = projects_fn();
            let _ = tx.send(TaskMsg::Log(format!("-- {} --", display)));
            let mut warned = false;
            let lock = loop {
                match utils::try_acquire_repo_lock("tui-pull") {
                    Ok(Some(lock)) => break lock,
                    Ok(None) => {
                        if !warned {
                            let _ = tx.send(TaskMsg::Log(
                                "⚠ waiting for other git operations to finish...".to_string(),
                            ));
                            warned = true;
                        }
                        thread::sleep(Duration::from_millis(500));
                    }
                    Err(e) => {
                        let _ = tx.send(TaskMsg::Log(format!("✘ failed to acquire lock: {}", e)));
                        let _ = tx.send(TaskMsg::Done {
                            ok: false,
                            msg: "pull failed (lock error)".to_string(),
                        });
                        return;
                    }
                }
            };
            let mut any_failed = false;

            for p in projects {
                let path = Path::new(p.local_dir());
                let res = if path.exists() {
                    if git::is_repo(path) {
                        git::pull_repo(path, &p.branch, force)
                    } else if is_dir_empty(path) {
                        git::clone_repo(&p.url, path)
                    } else {
                        git::init_repo_in_dir(&p.url, &p.branch, path)
                    }
                } else {
                    git::clone_repo(&p.url, path)
                };

                match res {
                    Ok(_) => {
                        let _ = tx.send(TaskMsg::Log(format!("✔ {}", p.name)));
                    }
                    Err(e) => {
                        any_failed = true;
                        let _ = tx.send(TaskMsg::Log(format!("✘ {}: {}", p.name, e)));
                    }
                }
            }

            drop(lock);
            let msg = if any_failed {
                format!("{} finished with errors", label)
            } else {
                format!("{} complete", label)
            };
            let _ = tx.send(TaskMsg::Done {
                ok: !any_failed,
                msg,
            });
        });
    }

    fn start_push_task(&mut self, label: String, display: String, commit_msg: Option<String>) {
        if self.task_running {
            self.set_status("Another task is already running".to_string(), true);
            return;
        }
        let cfg = match self.config.clone() {
            Some(cfg) => cfg,
            None => {
                self.set_status("No workspace loaded".to_string(), true);
                return;
            }
        };

        self.log_lines.clear();
        self.tab = Tab::Logs;
        self.task_running = true;
        self.task_label = Some(label.clone());

        let (tx, rx) = mpsc::channel();
        self.task_rx = Some(rx);
        self.set_status(format!("{} started — see Logs tab", label), false);

        thread::spawn(move || {
            let levels = cfg.execution_levels().unwrap_or_default();
            let _ = tx.send(TaskMsg::Log(format!("-- {} --", display)));
            let mut warned = false;
            let lock = loop {
                match utils::try_acquire_repo_lock("tui-push") {
                    Ok(Some(lock)) => break lock,
                    Ok(None) => {
                        if !warned {
                            let _ = tx.send(TaskMsg::Log(
                                "⚠ waiting for other git operations to finish...".to_string(),
                            ));
                            warned = true;
                        }
                        thread::sleep(Duration::from_millis(500));
                    }
                    Err(e) => {
                        let _ = tx.send(TaskMsg::Log(format!("✘ failed to acquire lock: {}", e)));
                        let _ = tx.send(TaskMsg::Done {
                            ok: false,
                            msg: "push failed (lock error)".to_string(),
                        });
                        return;
                    }
                }
            };
            let mut any_failed = false;

            for (level_idx, level) in levels.into_iter().enumerate() {
                if level.len() > 1 {
                    let _ = tx.send(TaskMsg::Log(format!(
                        "level {}: {} repos",
                        level_idx + 1,
                        level.len()
                    )));
                }

                for p in level {
                    let path = Path::new(p.local_dir());
                    if !path.exists() {
                        any_failed = true;
                        let _ = tx.send(TaskMsg::Log(format!(
                            "✘ {}: missing (run pull first)",
                            p.name
                        )));
                        continue;
                    }
                    if !git::is_repo(path) {
                        any_failed = true;
                        let _ = tx.send(TaskMsg::Log(format!("✘ {}: not a git repo", p.name)));
                        continue;
                    }

                    if let Some(msg) = commit_msg.as_deref() {
                        match git::has_uncommitted_changes(path) {
                            Ok(true) => {
                                if let Err(e) = git::commit_all(path, msg) {
                                    any_failed = true;
                                    let _ = tx.send(TaskMsg::Log(format!(
                                        "✘ {}: commit failed: {}",
                                        p.name, e
                                    )));
                                    continue;
                                } else {
                                    let _ =
                                        tx.send(TaskMsg::Log(format!("● {}: committed", p.name)));
                                }
                            }
                            Ok(false) => {}
                            Err(e) => {
                                any_failed = true;
                                let _ = tx.send(TaskMsg::Log(format!("✘ {}: {}", p.name, e)));
                                continue;
                            }
                        }
                    } else if let Ok(true) = git::has_uncommitted_changes(path) {
                        any_failed = true;
                        let _ = tx.send(TaskMsg::Log(format!(
                            "✘ {}: has uncommitted changes (commit required)",
                            p.name
                        )));
                        continue;
                    }

                    match git::push_repo(path, &p.branch) {
                        Ok(_) => {
                            let _ = tx.send(TaskMsg::Log(format!("✔ {} pushed", p.name)));
                        }
                        Err(e) => {
                            any_failed = true;
                            let _ = tx.send(TaskMsg::Log(format!("✘ {}: {}", p.name, e)));
                        }
                    }
                }
            }

            drop(lock);
            let msg = if any_failed {
                format!("{} finished with errors", label)
            } else {
                format!("{} complete", label)
            };
            let _ = tx.send(TaskMsg::Done {
                ok: !any_failed,
                msg,
            });
        });
    }

    pub(super) fn submit_add_form(&mut self) {
        let name = self.add_form.fields[0].trim().to_string();
        let path = {
            let p = self.add_form.fields[1].trim().to_string();
            if p.is_empty() || p == name {
                None
            } else {
                Some(p)
            }
        };
        let url = self.add_form.fields[2].trim().to_string();
        let branch = {
            let b = self.add_form.fields[3].trim().to_string();
            if b.is_empty() {
                "main".to_string()
            } else {
                b
            }
        };
        let sync_url = {
            let s = self.add_form.fields[4].trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };
        let depends_on: Option<Vec<String>> = {
            let d = self.add_form.fields[5].trim().to_string();
            if d.is_empty() {
                None
            } else {
                Some(d.split(',').map(|s| s.trim().to_string()).collect())
            }
        };

        if name.is_empty() || url.is_empty() {
            self.set_status("Name and URL are required".to_string(), true);
            return;
        }

        if let Some(cfg) = self.config.as_mut() {
            if cfg.projects.iter().any(|p| p.name == name) {
                self.set_status(format!("'{}' already exists", name), true);
                return;
            }
            let requested_dir = path.as_deref().unwrap_or(&name);
            if Path::new(requested_dir).is_absolute() {
                self.set_status(
                    "project path must be relative to workspace root".to_string(),
                    true,
                );
                return;
            }
            let requested = normalize_local_dir(requested_dir);
            if cfg
                .projects
                .iter()
                .any(|p| normalize_local_dir(p.local_dir()) == requested)
            {
                self.set_status(
                    format!("project path '{}' is already used", requested_dir),
                    true,
                );
                return;
            }
            cfg.projects.push(Project {
                name: name.clone(),
                path,
                url,
                branch,
                depends_on,
                sync_url,
                sync_interval: None,
            });
            if let Err(e) = cfg.save() {
                self.set_status(format!("Save failed: {}", e), true);
                return;
            }
            self.repo_statuses.push("…".to_string());
            self.set_status(format!("Added '{}'", name), false);
        } else {
            self.set_status(
                "No workspace loaded. Run `polyws init` first.".to_string(),
                true,
            );
            return;
        }

        self.show_add_form = false;
        self.add_form = AddForm::default();
    }

    pub(super) fn pull_all(&mut self) {
        if self.task_running {
            self.set_status("Another task is already running".to_string(), true);
            return;
        }
        let cfg = match self.config.clone() {
            Some(cfg) => cfg,
            None => {
                self.set_status("No workspace loaded".to_string(), true);
                return;
            }
        };
        self.start_pull_task(
            "pull-all".to_string(),
            cfg.name.clone(),
            move || {
                cfg.execution_levels()
                    .map(|levels| levels.into_iter().flatten().cloned().collect())
                    .unwrap_or_default()
            },
            false,
        );
    }

    pub(super) fn request_push_all(&mut self) {
        if self.task_running {
            self.set_status("Another task is already running".to_string(), true);
            return;
        }
        let cfg = match self.config.clone() {
            Some(cfg) => cfg,
            None => {
                self.set_status("No workspace loaded".to_string(), true);
                return;
            }
        };

        let mut any_dirty = false;
        for p in &cfg.projects {
            let path = Path::new(p.local_dir());
            if !path.exists() || !git::is_repo(path) {
                continue;
            }
            match git::has_uncommitted_changes(path) {
                Ok(true) => {
                    any_dirty = true;
                    break;
                }
                Ok(false) => {}
                Err(e) => {
                    self.set_status(format!("{}: {}", p.name, e), true);
                    return;
                }
            }
        }

        if any_dirty {
            self.show_commit_prompt = true;
            self.commit_input.clear();
            self.pending_push = Some(PushScope::All);
        } else {
            self.start_push_task("push-all".to_string(), cfg.name.clone(), None);
        }
    }

    pub(super) fn request_push_selected(&mut self) {
        if self.task_running {
            self.set_status("Another task is already running".to_string(), true);
            return;
        }
        let p = match self.selected_project() {
            Some(p) => p.clone(),
            None => return,
        };
        let path = Path::new(p.local_dir());
        if !path.exists() {
            self.set_status(
                format!("'{}' not found — run pull first", p.local_dir()),
                true,
            );
            return;
        }
        if !git::is_repo(path) {
            self.set_status(format!("'{}' is not a git repo", p.local_dir()), true);
            return;
        }
        match git::has_uncommitted_changes(path) {
            Ok(true) => {
                self.show_commit_prompt = true;
                self.commit_input.clear();
                self.pending_push = Some(PushScope::Selected(p));
            }
            Ok(false) => {
                self.start_push_task("push-selected".to_string(), p.name.clone(), None);
            }
            Err(e) => self.set_status(format!("{}: {}", p.name, e), true),
        }
    }

    pub(super) fn cancel_commit_prompt(&mut self) {
        self.show_commit_prompt = false;
        self.commit_input.clear();
        self.pending_push = None;
        self.set_status("Push cancelled".to_string(), false);
    }

    pub(super) fn confirm_commit_prompt(&mut self) {
        let msg = self.commit_input.trim().to_string();
        if msg.is_empty() {
            self.set_status("Commit message required".to_string(), true);
            return;
        }
        let scope = self.pending_push.take();
        self.show_commit_prompt = false;
        self.commit_input.clear();
        match scope {
            Some(PushScope::All) => {
                if let Some(cfg) = self.config.clone() {
                    self.start_push_task("push-all".to_string(), cfg.name.clone(), Some(msg));
                } else {
                    self.set_status("No workspace loaded".to_string(), true);
                }
            }
            Some(PushScope::Selected(p)) => {
                self.start_push_task("push-selected".to_string(), p.name.clone(), Some(msg));
            }
            None => {
                self.set_status("No push action pending".to_string(), true);
            }
        }
    }

    pub(super) fn create_snapshot(&mut self) {
        match snapshot::create() {
            Ok(_) => {
                self.snap_files = list_snap_files();
                self.set_status("Snapshot created".to_string(), false);
            }
            Err(e) => self.set_status(format!("Snapshot failed: {}", e), true),
        }
    }

    pub(super) fn restore_snap_at(&mut self, i: usize) {
        if let Some(file) = self.snap_files.get(i) {
            let f = file.clone();
            match snapshot::restore(&f, false, true) {
                Ok(_) => {
                    self.refresh_statuses();
                    self.set_status(format!("Restored {}", f), false);
                }
                Err(e) => self.set_status(format!("Restore failed: {}", e), true),
            }
        }
    }

    pub(super) fn toggle_sync(&mut self) {
        if self.sync_running {
            match sync_mod::stop() {
                Ok(_) => {
                    self.sync_running = false;
                    self.set_status("Sync daemon stopped".to_string(), false);
                }
                Err(e) => self.set_status(format!("{}", e), true),
            }
        } else {
            match sync_mod::start() {
                Ok(_) => {
                    self.sync_running = true;
                    self.set_status("Sync daemon started".to_string(), false);
                }
                Err(e) => self.set_status(format!("{}", e), true),
            }
        }
    }

    pub(super) fn sync_now(&mut self) {
        match sync_mod::sync_now() {
            Ok(_) => self.set_status("Mirror sync complete".to_string(), false),
            Err(e) => self.set_status(format!("{}", e), true),
        }
    }

    pub(super) fn run_doctor(&mut self) {
        self.log_lines.clear();
        use std::process::Command;

        let checks: &[(&str, &[&str], &str)] = &[
            ("git", &["--version"], "git installed"),
            ("ssh", &["-V"], "ssh available"),
            ("rustc", &["--version"], "rustc installed"),
            ("cargo", &["--version"], "cargo available"),
        ];

        for (bin, args, label) in checks {
            let ok = Command::new(bin)
                .args(*args)
                .output()
                .map(|o| o.status.success() || !o.stderr.is_empty())
                .unwrap_or(false);
            if ok {
                self.log_lines.push(format!("✔ {}", label));
            } else {
                self.log_lines.push(format!("✘ {}", label));
            }
        }

        let inet = std::net::TcpStream::connect_timeout(
            &"8.8.8.8:53".parse().expect("hardcoded socket address"),
            Duration::from_secs(3),
        )
        .is_ok();
        self.log_lines.push(if inet {
            "✔ internet reachable".to_string()
        } else {
            "✘ internet not reachable".to_string()
        });

        if let Some(cfg) = &self.config {
            self.log_lines
                .push(format!("✔ workspace '{}' loaded", cfg.name));
            for p in &cfg.projects {
                let path = Path::new(p.local_dir());
                if git::is_repo(path) {
                    self.log_lines.push(format!("✔ repo '{}' present", p.name));
                } else if path.exists() {
                    self.log_lines
                        .push(format!("✘ '{}' exists but not a git repo", p.name));
                } else {
                    self.log_lines
                        .push(format!("⚠ repo '{}' not cloned", p.name));
                }
            }
        } else {
            self.log_lines
                .push("⚠ no workspace config found".to_string());
        }

        self.set_status("Doctor complete".to_string(), false);
        self.tab = Tab::Logs;
    }
}

fn is_dir_empty(path: &Path) -> bool {
    fs::read_dir(path)
        .map(|mut i| i.next().is_none())
        .unwrap_or(false)
}
