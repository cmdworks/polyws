use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
    },
    Frame,
};

use super::app::App;
use super::helpers::centered_rect;
use super::types::{Tab, FIELD_HINTS, FIELD_LABELS};

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

pub(super) fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();

    // Fill entire area with dark background first.
    let bg = Block::default().style(Style::default().bg(Color::Black));
    f.render_widget(bg, area);

    // Outer double-border screen frame.
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Rgb(0, 210, 255)));
    let inner = outer_block.inner(area);
    f.render_widget(outer_block, area);

    let is_small_screen = inner.width < 90;
    let top_bar_height = if is_small_screen { 2 } else { 1 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(top_bar_height),
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(inner);

    draw_logo(f, chunks[0]);

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
                Constraint::Length((7 + workspace_name.len()) as u16),
                Constraint::Min(0),
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
                Constraint::Length((7 + workspace_name.len()) as u16),
                Constraint::Min(0),
                Constraint::Length(6),
            ])
            .split(chunks[1]);

        let badge_rect = Rect {
            width: 7 + workspace_name.len() as u16,
            ..top_bar_chunks[0]
        };
        f.render_widget(badge, badge_rect);
        f.render_widget(tabs, top_bar_chunks[1]);
        f.render_widget(version, top_bar_chunks[2]);
    }

    match app.tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[3]),
        Tab::Projects => draw_projects(f, app, chunks[3]),
        Tab::Graph => draw_graph(f, app, chunks[3]),
        Tab::Snapshots => draw_snapshots(f, app, chunks[3]),
        Tab::Sync => draw_sync(f, app, chunks[3]),
        Tab::Logs => draw_log(f, app, chunks[3]),
    }

    let compact_hints = chunks[4].width < 110;
    let hint = if app.config.is_none() {
        if compact_hints {
            "i:Init q:Quit"
        } else {
            "i:Init workspace q:Quit"
        }
    } else {
        match app.tab {
            Tab::Dashboard => {
                if compact_hints {
                    "p:Pull s:Snap d:Doc r:Reload Tab q"
                } else {
                    "p:Pull-all s:Snapshot d:Doctor r:Reload Tab:Switch q:Quit"
                }
            }
            Tab::Projects => {
                if compact_hints {
                    "a:Add d:Del p/↵:Pull e:Exec s:Ref ↑↓:Move q"
                } else {
                    "a:Add d:Delete p/↵:Pull e:Exec s:Refresh ↑↓:Move q:Quit"
                }
            }
            Tab::Graph => {
                if compact_hints {
                    "r:Reload Tab q"
                } else {
                    "r:Reload Tab:Switch q:Quit"
                }
            }
            Tab::Snapshots => {
                if compact_hints {
                    "c:Create r/↵:Restore ↑↓:Move q"
                } else {
                    "c:Create ↵/r:Restore ↑↓:Move q:Quit"
                }
            }
            Tab::Sync => {
                if compact_hints {
                    "s:Start/Stop n:Now r q"
                } else {
                    "s:Start/Stop n:SyncNow r:Reload q:Quit"
                }
            }
            Tab::Logs => {
                if compact_hints {
                    "d:RunDoctor r q"
                } else {
                    "d:Run Doctor r:Reload q:Quit"
                }
            }
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

fn draw_help(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(74, 78, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Help / Shortcuts  [Esc=Close] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0, 210, 255)));

    let text = vec![
        Line::from(vec![Span::styled(
            " Global",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   1-6 / Tab     : Switch tabs"),
        Line::from("   h             : Toggle help"),
        Line::from("   r             : Reload config"),
        Line::from("   q             : Quit"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Projects",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   ↑↓ / j k      : Move selection"),
        Line::from("   a             : Add project"),
        Line::from("   d / Del       : Delete selected"),
        Line::from("   p / Enter     : Pull selected"),
        Line::from("   e             : Exec command"),
        Line::from("   s             : Refresh statuses"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Add Form",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   Tab / ShiftTab: Next/prev field"),
        Line::from("   ↑↓            : Next/prev field"),
        Line::from("   Enter         : Save"),
        Line::from("   Esc           : Cancel"),
        Line::from(""),
        Line::from(vec![Span::styled(
            " Other Tabs",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from("   Dashboard: p pull-all, s snapshot, d doctor"),
        Line::from("   Snapshots: c create, r/Enter restore"),
        Line::from("   Sync     : s start/stop, n sync now"),
        Line::from("   Doctor   : d run doctor checks"),
    ];

    let p = Paragraph::new(text).wrap(Wrap { trim: true }).block(block);
    f.render_widget(p, popup_area);
}

fn draw_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

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
                            (status.clone(), Color::Rgb(255, 66, 161))
                        } else if status.contains("modified") {
                            (status.clone(), Color::Rgb(128, 90, 215))
                        } else {
                            (status.clone(), Color::Rgb(0, 210, 255))
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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    );
    f.render_widget(table, cols[0]);

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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    );
    f.render_widget(info, right_chunks[0]);

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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    );
    f.render_widget(snap_list, right_chunks[1]);
}

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
                Color::Rgb(255, 66, 161)
            } else if status.contains("modified") {
                Color::Rgb(128, 90, 215)
            } else {
                Color::Rgb(0, 210, 255)
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
            let display_name = if p.local_dir() != p.name {
                format!("{} ({})", p.name, p.local_dir())
            } else {
                p.name.clone()
            };

            Row::new(vec![
                Cell::from(display_name),
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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.table_state);
}

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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    );
    f.render_widget(list, area);
}

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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    f.render_stateful_widget(table, area, &mut app.snap_state);
}

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
        ("● RUNNING", Color::Rgb(0, 210, 255))
    } else {
        ("○ STOPPED", Color::Rgb(255, 66, 161))
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
            .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
    );
    f.render_widget(status_block, chunks[0]);

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
                    .style(Style::default().fg(Color::Rgb(0, 210, 255))),
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
                .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
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
                        .fg(Color::Rgb(0, 210, 255))
                        .add_modifier(Modifier::BOLD),
                )
                .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(" Mirrors ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(58, 123, 213))),
        );
        f.render_widget(table, chunks[1]);
    }
}

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

fn draw_add_form(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(74, 88, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Add Project [Tab/↑↓=field Enter=save Esc=cancel] ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, popup_area);

    let margin: u16 = 2;
    let field_count = FIELD_LABELS.len() as u16;
    // Keep at least one visible text row per field on shorter terminals.
    let usable_height = popup_area.height.saturating_sub(margin * 2);
    let field_height = if usable_height >= field_count * 4 {
        4
    } else {
        3
    };

    let inner = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints(vec![Constraint::Length(field_height); FIELD_LABELS.len()])
        .split(popup_area);

    for (i, label) in FIELD_LABELS.iter().enumerate() {
        let is_focused = app.add_form.focused == i;
        let is_branch = i == 3;
        let border_color = if is_focused {
            Color::Yellow
        } else {
            Color::Gray
        };
        let raw_value = app.add_form.fields[i].as_str();
        let value = if is_branch && raw_value.trim().is_empty() {
            "main"
        } else {
            raw_value
        };
        let is_empty = value.trim().is_empty();

        let (display, text_style) = if is_focused {
            (
                format!("{}|", value),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
        } else if is_empty {
            if is_branch {
                ("main".to_string(), Style::default().fg(Color::White))
            } else {
                (
                    FIELD_HINTS[i].to_string(),
                    Style::default().fg(Color::DarkGray),
                )
            }
        } else {
            (value.to_string(), Style::default().fg(Color::White))
        };

        let field = Paragraph::new(display)
            .block(
                Block::default()
                    .title(Line::from(Span::styled(
                        format!(" {} ", label),
                        Style::default().fg(border_color),
                    )))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            )
            .style(text_style);
        if i < inner.len() {
            f.render_widget(field, inner[i]);
        }
    }
}

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
