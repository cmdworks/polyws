/// Full-screen interactive TUI for polyws.
///
/// Launched when `polyws` is invoked with no sub-command from inside a workspace.
/// Navigation: Tab/Shift-Tab or number keys switch tabs.
/// Each tab shows a table; actions are driven by the keys shown in the bottom bar.
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, TableState,
        Tabs,
    },
    Frame, Terminal,
};
use rayon::prelude::*;

use crate::config::{Project, WorkspaceConfig};
use crate::git;
use crate::snapshot;
use crate::sync as sync_mod;
use crate::workspace;

// ─────────────────────────────────────────────────────────
// Tab index
// ─────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Dashboard = 0,
    Projects = 1,
    Graph = 2,
    Snapshots = 3,
    Sync = 4,
    Logs = 5,
}

impl Tab {
    fn titles() -> &'static [&'static str] {
        &["1:Dash", "2:Proj", "3:Graph", "4:Snap", "5:Sync", "6:Logs"]
    }
    fn from_usize(i: usize) -> Self {
        match i {
            0 => Tab::Dashboard,
            1 => Tab::Projects,
            2 => Tab::Graph,
            3 => Tab::Snapshots,
            4 => Tab::Sync,
            _ => Tab::Logs,
        }
    }
}

// ─────────────────────────────────────────────────────────
// Add-project form state
// ─────────────────────────────────────────────────────────

#[derive(Default, Clone)]
struct AddForm {
    fields: [String; 5], // name, url, branch, sync_url, depends_on
    focused: usize,      // which field
}

const FIELD_LABELS: [&str; 5] = ["Name", "URL", "Branch", "Sync URL", "Depends On"];

// ─────────────────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────────────────

struct App {
    tab: Tab,
    table_state: TableState, // Projects tab
    snap_state: TableState,  // Snapshots tab
    config: Option<WorkspaceConfig>,
    repo_statuses: Vec<String>, // one per project, filled lazily
    snap_files: Vec<String>,    // snapshot filenames
    log_lines: Vec<String>,     // doctor / exec output
    sync_running: bool,
    show_add_form: bool,
    show_help: bool,
    add_form: AddForm,
    show_exec_prompt: bool,
    exec_input: String,
    status_msg: Option<(String, bool)>, // (msg, is_error)
    last_tick: Instant,
}

impl App {
    fn new() -> Self {
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
        }
    }

    fn reload(&mut self) {
        self.config = WorkspaceConfig::load().ok();
        let n = self.config.as_ref().map(|c| c.projects.len()).unwrap_or(0);
        self.repo_statuses = vec!["…".to_string(); n];
        self.snap_files = list_snap_files();
        self.sync_running = sync_mod::is_daemon_running();
        self.set_status("Reloaded workspace config".to_string(), false);
    }

    fn refresh_statuses(&mut self) {
        if let Some(cfg) = &self.config {
            self.repo_statuses = cfg
                .projects
                .iter()
                .map(|p| {
                    let path = Path::new(&p.name);
                    if !path.exists() {
                        "\x1b[31mmissing\x1b[0m".to_string()
                    } else {
                        git::repo_status(path).unwrap_or_else(|_| "?".to_string())
                    }
                })
                .collect();
        }
    }

    fn set_status(&mut self, msg: String, error: bool) {
        self.status_msg = Some((msg, error));
        self.last_tick = Instant::now();
    }

    fn selected_project(&self) -> Option<&Project> {
        let cfg = self.config.as_ref()?;
        let i = self.table_state.selected()?;
        cfg.projects.get(i)
    }

    fn pull_selected(&mut self) {
        if let Some(p) = self.selected_project() {
            let name = p.name.clone();
            let url = p.url.clone();
            let branch = p.branch.clone();
            let path = Path::new(&name);
            let res = if path.exists() {
                git::pull_repo(path, &branch, false)
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

    fn remove_selected(&mut self) {
        if let Some(p) = self.selected_project() {
            let name = p.name.clone();
            if let Some(cfg) = self.config.as_mut() {
                cfg.projects.retain(|x| x.name != name);
                if let Err(e) = cfg.save() {
                    self.set_status(format!("Save failed: {}", e), true);
                    return;
                }
            }
            // adjust selection
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

    fn submit_add_form(&mut self) {
        let name = self.add_form.fields[0].trim().to_string();
        let url = self.add_form.fields[1].trim().to_string();
        let branch = {
            let b = self.add_form.fields[2].trim().to_string();
            if b.is_empty() {
                "main".to_string()
            } else {
                b
            }
        };
        let sync_url = {
            let s = self.add_form.fields[3].trim().to_string();
            if s.is_empty() {
                None
            } else {
                Some(s)
            }
        };
        let depends_on: Option<Vec<String>> = {
            let d = self.add_form.fields[4].trim().to_string();
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
            cfg.projects.push(Project {
                name: name.clone(),
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

    fn pull_all(&mut self) {
        self.log_lines.clear();
        if let Some(cfg) = self.config.clone() {
            for p in &cfg.projects {
                let path = Path::new(&p.name);
                let res = if path.exists() {
                    git::pull_repo(path, &p.branch, false)
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

    fn create_snapshot(&mut self) {
        match snapshot::create() {
            Ok(_) => {
                self.snap_files = list_snap_files();
                self.set_status("Snapshot created".to_string(), false);
            }
            Err(e) => self.set_status(format!("Snapshot failed: {}", e), true),
        }
    }

    fn restore_snap_at(&mut self, i: usize) {
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

    fn toggle_sync(&mut self) {
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

    fn sync_now(&mut self) {
        match sync_mod::sync_now() {
            Ok(_) => self.set_status("Mirror sync complete".to_string(), false),
            Err(e) => self.set_status(format!("{}", e), true),
        }
    }

    fn run_doctor(&mut self) {
        self.log_lines.clear();
        // Run the checks inline and capture output to log_lines.
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
        // internet
        let inet = std::net::TcpStream::connect_timeout(
            &"8.8.8.8:53".parse().unwrap(),
            Duration::from_secs(3),
        )
        .is_ok();
        self.log_lines.push(if inet {
            "✔ internet reachable".to_string()
        } else {
            "✘ internet not reachable".to_string()
        });
        // workspace
        if let Some(cfg) = &self.config {
            self.log_lines
                .push(format!("✔ workspace '{}' loaded", cfg.name));
            for p in &cfg.projects {
                let path = Path::new(&p.name);
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

// ─────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────

pub fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        crossterm::terminal::SetTitle("polyws"),
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    // Force dark background so we never render on a light terminal
    terminal.clear()?;

    let mut app = App::new();
    app.refresh_statuses();

    let tick = Duration::from_millis(500);
    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        if event::poll(tick)? {
            if let Event::Key(key) = event::read()? {
                if handle_key(&mut app, key.code, key.modifiers) {
                    break;
                }
            }
        }

        // Clear status after 4 seconds.
        if app.status_msg.is_some() && app.last_tick.elapsed() > Duration::from_secs(4) {
            app.status_msg = None;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// ─────────────────────────────────────────────────────────
// Overlay: Help
// ─────────────────────────────────────────────────────────

fn draw_help(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Help / Shortcuts  [Esc=Close] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0, 210, 255))); // Cyan

    let text = vec![
        Line::from(vec![Span::styled(
            " Global",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   1-5 or Tab : Switch Tabs"),
        Line::from("   h          : Toggle this Help popup"),
        Line::from("   r          : Reload configuration"),
        Line::from("   q          : Quit polyws"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Projects Tab",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   p          : Pull all projects"),
        Line::from("   s          : Create global snapshot"),
        Line::from("   d          : Run Doctor checks"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Graph Tab",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   ↑ / ↓      : Navigate list"),
        Line::from("   a          : Add new project"),
        Line::from("   d / Del    : Delete selected project"),
        Line::from("   p / Enter  : Pull selected project"),
        Line::from("   e          : Execute shell command in dependents"),
        Line::from("   s          : Refresh git statuses"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Snapshots / Sync / Logs Tabs",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   s (Sync)   : Start / Stop daemon"),
        Line::from("   n (Sync)   : Force Sync Now"),
        Line::from("   c (Snaps)  : Create snapshot"),
        Line::from("   r (Snaps)  : Restore selected snapshot"),
        Line::from("   d (Logs)   : Run Doctor checks"),
    ];

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, popup_area);
}

// ─────────────────────────────────────────────────────────
// Key handling
// ─────────────────────────────────────────────────────────

/// Returns true when the app should quit.
fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    // Handle Esc key for all overlays first
    if code == KeyCode::Esc {
        if app.show_add_form {
            app.show_add_form = false;
            app.add_form = AddForm::default();
            return false;
        }
        if app.show_exec_prompt {
            app.show_exec_prompt = false;
            app.exec_input.clear();
            return false;
        }
        if app.show_help {
            app.show_help = false;
            return false;
        }
    }

    // ── Add-form overlay ──────────────────────────────────
    if app.show_add_form {
        match code {
            KeyCode::Tab => {
                app.add_form.focused = (app.add_form.focused + 1) % 5;
            }
            KeyCode::BackTab => {
                app.add_form.focused = (app.add_form.focused + 4) % 5;
            }
            KeyCode::Enter => app.submit_add_form(),
            KeyCode::Backspace => {
                let f = app.add_form.focused;
                app.add_form.fields[f].pop();
            }
            KeyCode::Char(c) => {
                let f = app.add_form.focused;
                app.add_form.fields[f].push(c);
            }
            _ => {}
        }
        return false;
    }

    // ── Exec prompt overlay ───────────────────────────────
    if app.show_exec_prompt {
        match code {
            KeyCode::Esc => {
                app.show_exec_prompt = false;
                app.exec_input.clear();
            }
            KeyCode::Enter => {
                let cmd = app.exec_input.trim().to_string();
                app.show_exec_prompt = false;
                app.exec_input.clear();
                if !cmd.is_empty() {
                    run_exec_cmd(app, &cmd);
                }
            }
            KeyCode::Backspace => {
                app.exec_input.pop();
            }
            KeyCode::Char(c) => {
                app.exec_input.push(c);
            }
            _ => {}
        }
        return false;
    }

    // ── Global keys ───────────────────────────────────────
    match code {
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => return true,
        KeyCode::Char('c') if mods.contains(KeyModifiers::CONTROL) => return true,

        // Init workspace when none is loaded
        KeyCode::Char('i') if app.config.is_none() => match workspace::init_silent() {
            Ok(name) => {
                app.reload();
                app.set_status(
                    format!("Workspace '{}' initialized — add projects with  a ", name),
                    false,
                );
            }
            Err(e) => app.set_status(format!("Init failed: {}", e), true),
        },

        // Tab switching via number keys
        KeyCode::Char('1') => app.tab = Tab::Dashboard,
        KeyCode::Char('2') => app.tab = Tab::Projects,
        KeyCode::Char('3') => app.tab = Tab::Graph,
        KeyCode::Char('4') => app.tab = Tab::Snapshots,
        KeyCode::Char('5') => app.tab = Tab::Sync,
        KeyCode::Char('6') => app.tab = Tab::Logs,

        // Tab / Shift-Tab
        KeyCode::Tab => {
            let next = (app.tab as usize + 1) % 6;
            app.tab = Tab::from_usize(next);
        }
        KeyCode::BackTab => {
            let prev = (app.tab as usize + 5) % 6;
            app.tab = Tab::from_usize(prev);
        }

        KeyCode::Char('r') | KeyCode::Char('R') => app.reload(),

        _ => {}
    }

    // ── Tab-specific keys ─────────────────────────────────
    // ── Tab-specific keys ─────────────────────────────────
    match app.tab {
        Tab::Dashboard => match code {
            KeyCode::Char('p') => app.pull_all(),
            KeyCode::Char('d') => app.run_doctor(),
            KeyCode::Char('s') => app.create_snapshot(),
            _ => {}
        },

        Tab::Projects => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let n = project_count(app);
                move_up(&mut app.table_state, n);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let n = project_count(app);
                move_down(&mut app.table_state, n);
            }
            KeyCode::Char('a') => {
                app.show_add_form = true;
                app.add_form = AddForm::default();
                app.add_form.fields[2] = "main".to_string();
            }
            KeyCode::Char('d') | KeyCode::Delete => app.remove_selected(),
            KeyCode::Char('p') | KeyCode::Enter => app.pull_selected(),
            KeyCode::Char('e') => {
                app.show_exec_prompt = true;
                app.exec_input.clear();
            }
            KeyCode::Char('s') => {
                app.refresh_statuses();
                app.set_status("Statuses refreshed".to_string(), false);
            }
            _ => {}
        },

        Tab::Graph => {}

        Tab::Snapshots => match code {
            KeyCode::Up | KeyCode::Char('k') => {
                let n = app.snap_files.len();
                move_up(&mut app.snap_state, n);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let n = app.snap_files.len();
                move_down(&mut app.snap_state, n);
            }
            KeyCode::Char('c') => app.create_snapshot(),
            KeyCode::Enter | KeyCode::Char('r') => {
                let i = app.snap_state.selected().unwrap_or(0);
                app.restore_snap_at(i);
            }
            _ => {}
        },

        Tab::Sync => match code {
            KeyCode::Char('s') => app.toggle_sync(),
            KeyCode::Char('n') => app.sync_now(),
            _ => {}
        },

        Tab::Logs => {
            if let KeyCode::Char('d') = code {
                app.run_doctor();
            }
        }
    }

    false
}

fn run_exec_cmd(app: &mut App, cmd: &str) {
    app.log_lines.clear();
    app.tab = Tab::Logs; // reuse log view
    if let Some(cfg) = app.config.clone() {
        let levels = match cfg.execution_levels() {
            Ok(lvls) => lvls,
            Err(e) => {
                app.log_lines
                    .push(format!("✘ failed to sort dependencies: {}", e));
                return;
            }
        };

        for (level_idx, level) in levels.into_iter().enumerate() {
            app.log_lines.push(format!("-- Level {} --", level_idx + 1));

            // Run all projects in this level concurrently, capturing output.
            let results: Vec<(String, Result<std::process::Output, std::io::Error>)> = level
                .par_iter()
                .map(|proj| {
                    let path = std::path::Path::new(&proj.name);
                    if !path.exists() {
                        return (
                            proj.name.clone(),
                            Err(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "directory missing",
                            )),
                        );
                    }
                    let out = if cfg!(windows) {
                        std::process::Command::new("cmd")
                            .args(["/C", cmd])
                            .current_dir(path)
                            .output()
                    } else {
                        std::process::Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .current_dir(path)
                            .output()
                    };
                    (proj.name.clone(), out)
                })
                .collect();

            // Append captured output sequentially to log buffer
            for (name, out_result) in results {
                match out_result {
                    Ok(o) if o.status.success() => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        app.log_lines.push(format!("── {} ──", name));
                        for line in stdout.lines() {
                            app.log_lines.push(format!("   {}", line));
                        }
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        app.log_lines.push(format!("✘ {} failed", name));
                        for line in stderr.lines() {
                            app.log_lines.push(format!("   {}", line));
                        }
                    }
                    Err(e) => {
                        app.log_lines.push(format!("✘ {}: {}", name, e));
                    }
                }
            }
        }
    }
    app.set_status(format!("exec: {}", cmd), false);
}

const LOGO_LINES: [&str; 4] = ["┏┓  ┓  ┓ ┏ ", "┃┃┏┓┃┓┏┃┃┃┏", "┣┛┗┛┗┗┫┗┻┛┛", "      ┛    "];

const LOGO_COLORS: [Color; 4] = [
    Color::Rgb(0, 210, 255),  // Cyan
    Color::Rgb(58, 123, 213), // Soft Blue
    Color::Rgb(128, 90, 215), // Purple
    Color::Rgb(255, 66, 161), // Pink/Magenta
];

fn draw_logo(f: &mut Frame, area: Rect) {
    let lines: Vec<Line> = LOGO_LINES
        .iter()
        .zip(LOGO_COLORS.iter())
        .map(|(text, color)| {
            Line::from(Span::styled(
                *text,
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            ))
        })
        .collect();
    let para = Paragraph::new(lines).centered();
    f.render_widget(para, area);
}

// ─────────────────────────────────────────────────────────
// Draw
// ─────────────────────────────────────────────────────────

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Fill entire area with dark background first
    let bg = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(bg, area);

    // ── Outer double-border screen frame ──────────────────
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(0, 210, 255))); // Cyan outer frame
    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let is_small_screen = inner.width < 90;
    let top_bar_height = if is_small_screen { 2 } else { 1 };

    // ── Root layout inside the double frame ───────────────
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),              // ASCII logo + 1 empty line
            Constraint::Length(top_bar_height), // inline or multiline tab bar
            Constraint::Length(1),              // 1 empty line before content
            Constraint::Min(0),                 // content
            Constraint::Length(1),              // status / key hints
        ])
        .split(inner);

    // ── ASCII logo ────────────────────────────────────────
    draw_logo(f, chunks[0]);

    // ── Tab bar ───────────────────────────────────────────
    let workspace_name = app
        .config
        .as_ref()
        .map(|c| c.name.clone())
        .unwrap_or_else(|| "(x)".to_string());

    let badge_text = format!("polyws {}", workspace_name);
    let badge = Paragraph::new(badge_text).style(
        Style::default()
            .bg(Color::Rgb(0, 210, 255))
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    let sep = Paragraph::new("|").style(Style::default().fg(Color::DarkGray));

    let version_str = format!("v{}", env!("CARGO_PKG_VERSION"));
    let version = Paragraph::new(version_str)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(ratatui::layout::Alignment::Right);

    let tab_titles: Vec<Line> = Tab::titles().iter().map(|t| Line::from(*t)).collect();
    let tabs = Tabs::new(tab_titles)
        .select(app.tab as usize)
        .divider(Span::raw("|"))
        .style(
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(0, 210, 255))
                .add_modifier(Modifier::BOLD),
        );

    if is_small_screen {
        let top_rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(chunks[1]);

        let row1 = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((7 + workspace_name.len()) as u16), // dynamic width roughly matching badge text length
                Constraint::Min(0),                                    // right-aligned version
            ])
            .split(top_rows[0]);

        let badge_rect = Rect {
            width: 7 + workspace_name.len() as u16,
            ..row1[0]
        };
        f.render_widget(badge, badge_rect);
        f.render_widget(version, row1[1]);

        f.render_widget(tabs, top_rows[1]);
    } else {
        let top_bar_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length((7 + workspace_name.len()) as u16), // badge + workspace size
                Constraint::Length(1),                                 // "|"
                Constraint::Min(0),                                    // Tabs
                Constraint::Length(6),                                 // "v1.0.0"
            ])
            .split(chunks[1]);

        let badge_rect = Rect {
            width: 7 + workspace_name.len() as u16,
            ..top_bar_chunks[0]
        };
        f.render_widget(badge, badge_rect);
        f.render_widget(sep, top_bar_chunks[1]);
        f.render_widget(tabs, top_bar_chunks[2]);
        f.render_widget(version, top_bar_chunks[3]);
    }

    // ── Content ───────────────────────────────────────────
    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[3]),
        Tab::Projects => draw_projects(f, app, chunks[3]),
        Tab::Graph => draw_graph(f, app, chunks[3]),
        Tab::Snapshots => draw_snapshots(f, app, chunks[3]),
        Tab::Sync => draw_sync(f, app, chunks[3]),
        Tab::Logs => draw_log(f, app, chunks[3]),
    }

    // ── Status / hint bar ─────────────────────────────────
    let hint = if app.config.is_none() {
        " i:Init workspace  q:Quit"
    } else {
        match app.tab {
            Tab::Dashboard => " p:Pull-all  s:Snapshot  d:Doctor  r:Reload  Tab:Switch  q:Quit",
            Tab::Projects => " a:Add  d:Delete  p/↵:Pull  e:Exec  s:Refresh  ↑↓:Move  q:Quit",
            Tab::Graph => " r:Reload  Tab:Switch  q:Quit",
            Tab::Snapshots => " c:Create  ↵/r:Restore  ↑↓:Move  q:Quit",
            Tab::Sync => " s:Start/Stop  n:Sync Now  r:Reload  q:Quit",
            Tab::Logs => " d:Run Doctor  r:Reload  q:Quit",
        }
    };

    let (status_text, status_style) = if let Some((msg, is_err)) = &app.status_msg {
        let style = if *is_err {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };
        (format!(" {}", msg), style)
    } else {
        (hint.to_string(), Style::default().fg(Color::DarkGray))
    };

    let status_bar = Paragraph::new(status_text).style(status_style);
    f.render_widget(status_bar, chunks[4]);

    // ── Overlays ──────────────────────────────────────────
    if app.show_add_form {
        draw_add_form(f, app, area);
    }
    if app.show_exec_prompt {
        draw_exec_prompt(f, app, area);
    }
    if app.show_help {
        draw_help(f, area);
    }
}

// ─────────────────────────────────────────────────────────
// Tab: Projects
// ─────────────────────────────────────────────────────────

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Left: repo summary table
    let cfg_ref = app.config.as_ref();
    let rows: Vec<Row> = cfg_ref
        .map(|cfg| {
            cfg.projects
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let status = app.repo_statuses.get(i).cloned().unwrap_or_default();
                    let (status_str, status_color) =
                        if status.contains("missing") || status.starts_with('✘') {
                            (status.clone(), Color::Rgb(255, 66, 161)) // Pink/Magenta
                        } else if status.contains("modified") {
                            (status.clone(), Color::Rgb(128, 90, 215)) // Purple
                        } else {
                            (status.clone(), Color::Rgb(0, 210, 255)) // Cyan
                        };
                    Row::new(vec![
                        Cell::from(p.name.clone()).style(
                            Style::default()
                                .fg(Color::Rgb(0, 210, 255))
                                .add_modifier(Modifier::BOLD),
                        ),
                        Cell::from(p.branch.clone()).style(Style::default().fg(Color::White)),
                        Cell::from(status_str).style(Style::default().fg(status_color)),
                    ])
                })
                .collect()
        })
        .unwrap_or_default();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(20),
            Constraint::Percentage(45),
        ],
    )
    .header(
        Row::new(vec!["Repository", "Branch", "Status"])
            .style(
                Style::default()
                    .fg(Color::Rgb(0, 210, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(" Repositories ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    );
    f.render_widget(table, cols[0]);

    // Right: info panel
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(7), Constraint::Min(0)])
        .split(cols[1]);

    let total = cfg_ref.map(|c| c.projects.len()).unwrap_or(0);
    let snaps = app.snap_files.len();
    let daemon = if app.sync_running {
        "running"
    } else {
        "stopped"
    };

    let info_items = vec![
        ListItem::new(format!(" Projects    : {}", total)),
        ListItem::new(format!(" Snapshots   : {}", snaps)),
        ListItem::new(format!(" Sync daemon : {}", daemon)),
    ];
    let info = List::new(info_items).block(
        Block::default()
            .title(" Overview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    );
    f.render_widget(info, right_chunks[0]);

    // Recent snapshots list
    let snap_items: Vec<ListItem> = app
        .snap_files
        .iter()
        .rev()
        .take(10)
        .map(|s| {
            let short = s.rsplit('/').next().unwrap_or(s);
            ListItem::new(format!(" {}", short))
        })
        .collect();
    let snap_list = List::new(snap_items).block(
        Block::default()
            .title(" Recent Snapshots ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    );
    f.render_widget(snap_list, right_chunks[1]);
}

// ─────────────────────────────────────────────────────────
// Tab: Projects
// ─────────────────────────────────────────────────────────

fn draw_projects(f: &mut Frame, app: &mut App, area: Rect) {
    let cfg = match &app.config {
        Some(c) => c,
        None => {
            let p = Paragraph::new(
                "\n  No workspace found in this directory.\n\n  Press  i  to initialize a new workspace here.",
            )
            .block(
                Block::default()
                    .title(" Projects ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::Yellow));
            f.render_widget(p, area);
            return;
        }
    };

    let rows: Vec<Row> = cfg
        .projects
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let status = app.repo_statuses.get(i).cloned().unwrap_or_default();
            let status_color = if status.contains("missing") {
                Color::Rgb(255, 66, 161) // Pink/Magenta
            } else if status.contains("modified") {
                Color::Rgb(128, 90, 215) // Purple
            } else {
                Color::Rgb(0, 210, 255) // Cyan
            };
            let deps = p
                .depends_on
                .as_deref()
                .map(|d| d.join(", "))
                .unwrap_or_default();
            let mirror = p
                .sync_url
                .as_deref()
                .map(|u| {
                    if u.len() > 30 {
                        format!("{}…", &u[..29])
                    } else {
                        u.to_string()
                    }
                })
                .unwrap_or_else(|| "—".to_string());

            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(p.branch.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(status).style(Style::default().fg(status_color)),
                Cell::from(deps).style(Style::default().fg(Color::Magenta)),
                Cell::from(mirror).style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(18),
            Constraint::Percentage(12),
            Constraint::Percentage(22),
            Constraint::Percentage(18),
            Constraint::Percentage(30),
        ],
    )
    .header(
        Row::new(vec!["Name", "Branch", "Status", "Depends On", "Mirror URL"])
            .style(
                Style::default()
                    .fg(Color::Rgb(0, 210, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(format!(" Projects ({}) ", cfg.projects.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.table_state);
}

// ─────────────────────────────────────────────────────────
// Tab: Graph
// ─────────────────────────────────────────────────────────

fn draw_graph(f: &mut Frame, app: &App, area: Rect) {
    let cfg = match &app.config {
        Some(c) => c,
        None => {
            let p = Paragraph::new(
                "\n  No workspace found in this directory.\n\n  Press  i  to initialize a new workspace here.",
            )
            .block(
                Block::default()
                    .title(" Graph ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::Yellow));
            f.render_widget(p, area);
            return;
        }
    };

    let levels = match cfg.execution_levels() {
        Ok(l) => l,
        Err(e) => {
            let p = Paragraph::new(format!("\n  Error calculating execution levels:\n  {}", e))
                .style(Style::default().fg(Color::Rgb(255, 66, 161)))
                .block(
                    Block::default()
                        .title(" Graph ")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::Rgb(255, 66, 161))),
                );
            f.render_widget(p, area);
            return;
        }
    };

    let mut items = Vec::new();
    items.push(ListItem::new(""));
    for (i, level) in levels.iter().enumerate() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            format!("  Level {}", i + 1),
            Style::default()
                .fg(Color::Rgb(0, 210, 255))
                .add_modifier(Modifier::BOLD),
        )])));

        let project_names: Vec<String> = level.iter().map(|p| p.name.clone()).collect();
        items.push(
            ListItem::new(format!("   └─ {}", project_names.join(", ")))
                .style(Style::default().fg(Color::White)),
        );
        items.push(ListItem::new(""));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Execution Dependency Graph ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    );
    f.render_widget(list, area);
}

// ─────────────────────────────────────────────────────────
// Tab: Snapshots
// ─────────────────────────────────────────────────────────

fn draw_snapshots(f: &mut Frame, app: &mut App, area: Rect) {
    use crate::snapshot::Snapshot;
    use std::fs;

    let rows: Vec<Row> = app
        .snap_files
        .iter()
        .map(|path| {
            let short = path.rsplit('/').next().unwrap_or(path).to_string();
            let (created, repos) = fs::read_to_string(path)
                .ok()
                .and_then(|c| serde_json::from_str::<Snapshot>(&c).ok())
                .map(|s| (s.created_at.clone(), s.commits.len()))
                .unwrap_or_else(|| ("?".to_string(), 0));
            Row::new(vec![
                Cell::from(short),
                Cell::from(created).style(Style::default().fg(Color::DarkGray)),
                Cell::from(repos.to_string()).style(Style::default().fg(Color::Cyan)),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(45),
            Constraint::Percentage(15),
        ],
    )
    .header(
        Row::new(vec!["File", "Created At", "Repos"])
            .style(
                Style::default()
                    .fg(Color::Rgb(0, 210, 255))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(format!(" Snapshots ({}) ", app.snap_files.len()))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.snap_state);
}

// ─────────────────────────────────────────────────────────
// Tab: Sync
// ─────────────────────────────────────────────────────────

fn draw_sync(f: &mut Frame, app: &App, area: Rect) {
    let is_mutagen_installed = crate::sync::is_mutagen_installed();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            if !is_mutagen_installed {
                Constraint::Length(8)
            } else {
                Constraint::Length(5)
            },
            Constraint::Min(0),
        ])
        .split(area);

    let (daemon_label, daemon_color) = if app.sync_running {
        ("● RUNNING", Color::Rgb(0, 210, 255)) // Cyan
    } else {
        ("○ STOPPED", Color::Rgb(255, 66, 161)) // Pink
    };

    let mut status_items = vec![ListItem::new(Line::from(vec![
        Span::raw(" Daemon status:  "),
        if is_mutagen_installed {
            Span::styled(
                daemon_label,
                Style::default()
                    .fg(daemon_color)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                "⚠ MUTAGEN NOT INSTALLED",
                Style::default()
                    .fg(Color::Rgb(255, 66, 161))
                    .add_modifier(Modifier::BOLD),
            )
        },
    ]))];

    if !is_mutagen_installed {
        status_items.push(ListItem::new(""));
        status_items.push(ListItem::new(Line::from(vec![
            Span::raw(" The sync daemon requires "),
            Span::styled("mutagen", Style::default().fg(Color::Rgb(0, 210, 255))),
            Span::raw(" to be installed and available in your PATH."),
        ])));
        status_items.push(ListItem::new(
            " macOS: brew install mutagen-io/mutagen/mutagen",
        ));
        status_items.push(ListItem::new(
            " Linux/Windows: Download from https://mutagen.io",
        ));
    }

    let status_block = List::new(status_items).block(
        Block::default()
            .title(" Sync Daemon ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
    );
    f.render_widget(status_block, chunks[0]);

    // Mirror table
    let cfg = match &app.config {
        Some(c) => c,
        None => return,
    };
    let rows: Vec<Row> = cfg
        .projects
        .iter()
        .filter(|p| p.sync_url.is_some())
        .map(|p| {
            Row::new(vec![
                Cell::from(p.name.clone()),
                Cell::from(p.sync_url.clone().unwrap_or_default())
                    .style(Style::default().fg(Color::Rgb(0, 210, 255))), // Cyan
                Cell::from(
                    p.sync_interval
                        .map(|i| format!("{}m", i))
                        .unwrap_or_else(|| "default".to_string()),
                )
                .style(Style::default().fg(Color::DarkGray)),
            ])
        })
        .collect();

    if rows.is_empty() {
        let p = Paragraph::new(" No projects have a sync_url configured.").block(
            Block::default()
                .title(" Mirrors ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
        );
        f.render_widget(p, chunks[1]);
    } else {
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(60),
                Constraint::Percentage(15),
            ],
        )
        .header(
            Row::new(vec!["Project", "Mirror URL", "Interval"])
                .style(
                    Style::default()
                        .fg(Color::Rgb(0, 210, 255)) // Cyan
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Mirrors ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(58, 123, 213))), // Soft Blue
        );
        f.render_widget(table, chunks[1]);
    }
}

// ─────────────────────────────────────────────────────────
// Tab: Logs
// ─────────────────────────────────────────────────────────

fn draw_log(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .log_lines
        .iter()
        .map(|line| {
            let style = if line.starts_with('✔') {
                Style::default().fg(Color::Green)
            } else if line.starts_with('✘') {
                Style::default().fg(Color::Red)
            } else if line.starts_with('⚠') {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::from(Span::styled(format!(" {}", line), style)))
        })
        .collect();

    if items.is_empty() {
        let p = Paragraph::new(
            " Press  d  to run doctor checks, or  e  on the Projects tab to exec a command.",
        )
        .block(
            Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        f.render_widget(p, area);
    } else {
        let list = List::new(items).block(
            Block::default()
                .title(" Output ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );
        f.render_widget(list, area);
    }
}

// ─────────────────────────────────────────────────────────
// Overlay: Add Project Form
// ─────────────────────────────────────────────────────────

fn draw_add_form(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(60, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Add Project  [Tab=next field  Enter=save  Esc=cancel] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, popup_area);

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(vec![Constraint::Length(3); 5])
        .split(popup_area);

    for (i, label) in FIELD_LABELS.iter().enumerate() {
        let is_focused = app.add_form.focused == i;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::DarkGray
        };
        let value = &app.add_form.fields[i];
        let display = if is_focused {
            format!("{}_", value)
        } else {
            value.clone()
        };
        let field = Paragraph::new(display)
            .block(
                Block::default()
                    .title(format!(" {} ", label))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(Style::default().fg(if is_focused {
                Color::White
            } else {
                Color::DarkGray
            }));
        if i < inner.len() {
            f.render_widget(field, inner[i]);
        }
    }
}

// ─────────────────────────────────────────────────────────
// Overlay: Exec prompt
// ─────────────────────────────────────────────────────────

fn draw_exec_prompt(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(55, 25, area);
    f.render_widget(Clear, popup);

    let prompt = Paragraph::new(format!(" $ {}_", app.exec_input))
        .block(
            Block::default()
                .title(" Execute command in all repos  [Enter=run  Esc=cancel] ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(prompt, popup);
}

// ─────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vert[1])[1]
}

fn move_up(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    state.select(Some(if i == 0 { len - 1 } else { i - 1 }));
}
fn move_down(state: &mut TableState, len: usize) {
    if len == 0 {
        return;
    }
    let i = state.selected().unwrap_or(0);
    state.select(Some((i + 1) % len));
}

fn project_count(app: &App) -> usize {
    app.config.as_ref().map(|c| c.projects.len()).unwrap_or(0)
}

fn list_snap_files() -> Vec<String> {
    let dir = std::path::Path::new(".polyws/snapshots");
    if !dir.exists() {
        return vec![];
    }
    let mut files: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();
    files.sort();
    files
}
