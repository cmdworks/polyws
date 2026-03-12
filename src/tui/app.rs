use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use ratatui::widgets::TableState;

use crate::config::{normalize_local_dir, Project, WorkspaceConfig};
use crate::{git, snapshot, sync as sync_mod};

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
    pub(super) status_msg: Option<(String, bool)>, // (msg, is_error)
    pub(super) last_tick: Instant,
    pub(super) confirm_delete: bool,
    pub(super) confirm_delete_name: Option<String>,
    pub(super) confirm_delete_path: Option<String>,
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
            status_msg: None,
            last_tick: Instant::now(),
            confirm_delete: false,
            confirm_delete_name: None,
            confirm_delete_path: None,
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
        if let Some(p) = self.selected_project() {
            let name = p.name.clone();
            let url = p.url.clone();
            let branch = p.branch.clone();
            let local_dir = p.local_dir().to_string();
            let path = Path::new(&local_dir);
            let res = if path.exists() {
                if git::is_repo(path) {
                    git::pull_repo(path, &branch, false)
                } else if is_dir_empty(path) {
                    git::clone_repo(&url, path)
                } else {
                    git::init_repo_in_dir(&url, &branch, path)
                }
            } else {
                git::clone_repo(&url, path)
            };
            match res {
                Ok(_) => self.set_status(format!("✔ {} synced", name), false),
                Err(e) => self.set_status(format!("✘ {}: {}", name, e), true),
            }
            self.refresh_statuses();
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
        self.log_lines.clear();
        if let Some(cfg) = self.config.clone() {
            for p in &cfg.projects {
                let path = Path::new(p.local_dir());
                let res = if path.exists() {
                    if git::is_repo(path) {
                        git::pull_repo(path, &p.branch, false)
                    } else if is_dir_empty(path) {
                        git::clone_repo(&p.url, path)
                    } else {
                        git::init_repo_in_dir(&p.url, &p.branch, path)
                    }
                } else {
                    git::clone_repo(&p.url, path)
                };
                match res {
                    Ok(_) => self.log_lines.push(format!("✔ {}", p.name)),
                    Err(e) => self.log_lines.push(format!("✘ {}: {}", p.name, e)),
                }
            }
            self.refresh_statuses();
            self.set_status("Pull all complete".to_string(), false);
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
