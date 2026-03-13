use crossterm::event::{KeyCode, KeyModifiers};
use rayon::prelude::*;

use crate::workspace;

use super::app::App;
use super::helpers::{move_down, move_up, project_count};
use super::types::{Tab, FIELD_LABELS};

fn handle_confirm_delete(app: &mut App, code: KeyCode) -> bool {
    if !app.confirm_delete {
        return false;
    }

    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => app.confirm_delete(),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.cancel_delete(),
        _ => {}
    }
    true
}

fn close_overlay_on_escape(app: &mut App, code: KeyCode) -> bool {
    if code != KeyCode::Esc {
        return false;
    }

    if app.show_add_form {
        app.show_add_form = false;
        app.add_form = Default::default();
        return true;
    }
    if app.show_exec_prompt {
        app.show_exec_prompt = false;
        app.exec_input.clear();
        return true;
    }
    if app.show_help {
        app.show_help = false;
        return true;
    }
    false
}

fn handle_add_form_overlay_key(app: &mut App, code: KeyCode) -> bool {
    if !app.show_add_form {
        return false;
    }

    match code {
        KeyCode::Tab | KeyCode::Down => {
            app.add_form.focused = (app.add_form.focused + 1) % FIELD_LABELS.len();
        }
        KeyCode::BackTab | KeyCode::Up => {
            app.add_form.focused =
                (app.add_form.focused + FIELD_LABELS.len() - 1) % FIELD_LABELS.len();
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
    true
}

fn handle_exec_prompt_overlay_key(app: &mut App, code: KeyCode) -> bool {
    if !app.show_exec_prompt {
        return false;
    }

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

    true
}

fn handle_commit_prompt_overlay_key(app: &mut App, code: KeyCode) -> bool {
    if !app.show_commit_prompt {
        return false;
    }

    match code {
        KeyCode::Esc => app.cancel_commit_prompt(),
        KeyCode::Enter => app.confirm_commit_prompt(),
        KeyCode::Backspace => {
            app.commit_input.pop();
        }
        KeyCode::Char(c) => {
            app.commit_input.push(c);
        }
        _ => {}
    }
    true
}

fn handle_global_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    match code {
        KeyCode::Char('h') | KeyCode::Char('H') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => app.open_add_form(),
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

    false
}

fn handle_tab_key(app: &mut App, code: KeyCode) {
    match app.tab {
        Tab::Dashboard => match code {
            KeyCode::Char('p') => app.pull_all(),
            KeyCode::Char('u') => app.request_push_all(),
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
            KeyCode::Char('d') | KeyCode::Delete => app.remove_selected(),
            KeyCode::Char('x') | KeyCode::Char('X') => app.request_delete_selected(),
            KeyCode::Char('p') | KeyCode::Enter => app.pull_selected(),
            KeyCode::Char('f') => app.pull_selected_with_force(true),
            KeyCode::Char('F') => app.request_force_push_selected(),
            KeyCode::Char('l') | KeyCode::Char('L') => app.flush_selected(),
            KeyCode::Char('i') | KeyCode::Char('I') => app.restore_remote_selected(),
            KeyCode::Char('u') => app.request_push_selected(),
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
}

/// Returns true when the app should quit.
pub(super) fn handle_key(app: &mut App, code: KeyCode, mods: KeyModifiers) -> bool {
    if handle_confirm_delete(app, code) {
        return false;
    }
    if close_overlay_on_escape(app, code) {
        return false;
    }
    if handle_add_form_overlay_key(app, code) {
        return false;
    }
    if handle_commit_prompt_overlay_key(app, code) {
        return false;
    }
    if handle_exec_prompt_overlay_key(app, code) {
        return false;
    }
    if handle_global_key(app, code, mods) {
        return true;
    }
    handle_tab_key(app, code);
    false
}

fn run_exec_cmd(app: &mut App, cmd: &str) {
    app.log_lines.clear();
    app.tab = Tab::Logs; // reuse log view
    if let Some(cfg) = app.config.clone() {
        let levels = match cfg.execution_levels() {
            Ok(lvls) => lvls,
            Err(e) => {
                app.push_log_line(format!("✘ failed to sort dependencies: {}", e));
                return;
            }
        };

        for (level_idx, level) in levels.into_iter().enumerate() {
            app.push_log_line(format!("-- Level {} --", level_idx + 1));

            // Run all projects in this level concurrently, capturing output.
            let results: Vec<(String, Result<std::process::Output, std::io::Error>)> = level
                .par_iter()
                .map(|proj| {
                    let path = std::path::Path::new(proj.local_dir());
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

            // Append captured output sequentially to log buffer.
            for (name, out_result) in results {
                match out_result {
                    Ok(o) if o.status.success() => {
                        let stdout = String::from_utf8_lossy(&o.stdout);
                        app.push_log_line(format!("── {} ──", name));
                        for line in stdout.lines() {
                            app.push_log_line(format!("   {}", line));
                        }
                    }
                    Ok(o) => {
                        let stderr = String::from_utf8_lossy(&o.stderr);
                        app.push_log_line(format!("✘ {} failed", name));
                        for line in stderr.lines() {
                            app.push_log_line(format!("   {}", line));
                        }
                    }
                    Err(e) => {
                        app.push_log_line(format!("✘ {}: {}", name, e));
                    }
                }
            }
        }
    }
    app.set_status(format!("exec: {}", cmd), false);
}
