// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::cli::session::TaskStatus,
    crate::tui::state::{AppTab, TuiState},
    crate::tui::theme::ThemePalette,
    ratatui::{
        Frame,
        layout::{Alignment, Constraint, Direction, Layout, Rect},
        style::{Modifier, Style},
        symbols,
        text::{Line, Span},
        widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    },
    unicode_width::UnicodeWidthStr,
};

/// Renders the top navigation tab bar.
pub fn render_tab_bar(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let tabs: Vec<Line> = AppTab::all()
        .into_iter()
        .map(|tab| {
            let is_active = tab == state.active_tab;
            let style = if is_active {
                Style::default()
                    .fg(palette.tab_active_fg)
                    .bg(palette.tab_active_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.muted)
            };
            Line::from(Span::styled(tab.title(), style))
        })
        .collect();

    let active_tab_idx = AppTab::all()
        .iter()
        .position(|t| *t == state.active_tab)
        .unwrap_or(0);

    let tab_widget = Tabs::new(tabs)
        .select(active_tab_idx)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(palette.border))
                .style(Style::default().bg(palette.bg)),
        )
        .highlight_style(
            Style::default()
                .fg(palette.tab_active_fg)
                .bg(palette.tab_active_bg)
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::styled(" │ ", Style::default().fg(palette.border)));

    frame.render_widget(tab_widget, area);
}

/// Renders the bottom status bar with agent mode, stats, and help.
pub fn render_status_bar(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let mode_color = match state.agent_mode_label.as_str() {
        "Idle" => palette.muted,
        "Planning" => palette.warn,
        "Executing" => palette.accent,
        "Reflecting" => palette.chart_2,
        "Synthesizing" => palette.chart_1,
        "MetaCognizing" => palette.chart_2,
        _ => palette.fg,
    };

    let internet_icon = if state.settings_internet {
        "🌐"
    } else {
        "✗web"
    };
    let yolo_badge = if state.settings_yolo { " ⚡YOLO" } else { "" };

    let spinner_icon = if state.agent_mode_label == "Idle" {
        " ❯ ".to_string()
    } else {
        let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let idx = (state.tick_count as usize / 2) % chars.len();
        format!(" {} ", chars[idx])
    };

    let left = Line::from(vec![
        Span::styled(
            spinner_icon,
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}{}", state.agent_mode_label, yolo_badge),
            Style::default().fg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  │  ", Style::default().fg(palette.border)),
        Span::styled(internet_icon, Style::default().fg(palette.muted)),
        Span::styled("  │  ", Style::default().fg(palette.border)),
        Span::styled(
            format!(
                "↑{}req  ↓{}res  ~{}tk↑  ~{}tk↓",
                state.stats.requests,
                state.stats.responses,
                state.stats.tokens_sent,
                state.stats.tokens_received,
            ),
            Style::default().fg(palette.muted),
        ),
    ]);

    let help_text = "  Tab/Shift+Tab: switch tabs  │  Esc: interrupt  │  q: quit  ";
    let right = Line::from(Span::styled(help_text, Style::default().fg(palette.muted)));

    let bar_block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = bar_block.inner(area);
    frame.render_widget(bar_block, area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Min(60)])
        .split(inner);

    frame.render_widget(Paragraph::new(left), cols[0]);
    frame.render_widget(Paragraph::new(right).alignment(Alignment::Right), cols[1]);
}

pub fn render_logo(palette: &ThemePalette) -> Vec<Line<'static>> {
    let logo_lines = [
        " █████╗ ██╗   ██╗████████╗ ██████╗  ██████╗ ██████╗ ████████╗",
        "██╔══██╗██║   ██║╚══██╔══╝██╔═══██╗██╔════╝ ██╔══██╗╚══██╔══╝",
        "███████║██║   ██║   ██║   ██║   ██║██║  ███╗██████╔╝   ██║   ",
        "██╔══██║██║   ██║   ██║   ██║   ██║██║   ██║██╔═══╝    ██║   ",
        "██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╔╝██║        ██║   ",
        "╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═╝        ╚═╝   ",
    ];
    logo_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let (r, g, b) = palette.logo_gradient[i];
            Line::from(Span::styled(
                line.to_string(),
                Style::default()
                    .fg(ratatui::style::Color::Rgb(r, g, b))
                    .add_modifier(Modifier::BOLD),
            ))
        })
        .collect()
}

/// Renders the main dashboard (log + tasks + input).
pub fn render_main_tab(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let mut constraints = Vec::new();
    let has_update = state.update_available.is_some();
    let has_home_warning = state.home_dir_warning;

    if has_update {
        constraints.push(Constraint::Length(4));
    }
    if has_home_warning {
        constraints.push(Constraint::Length(5));
    }
    constraints.push(Constraint::Length(8));
    constraints.push(Constraint::Fill(1));
    constraints.push(Constraint::Length(3));

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut current_idx = 0;

    if has_update {
        if let Some((current, latest)) = state.update_available.as_ref() {
            let msg = format!(
                "AutoGPT update available! {} → {}\nRun `cargo install autogpt --all-features` to update.",
                current, latest
            );
            let block = Block::default()
                .borders(Borders::ALL)
                .border_set(symbols::border::ROUNDED)
                .border_style(Style::default().fg(palette.warn))
                .style(Style::default().bg(palette.bg));

            let p = Paragraph::new(msg)
                .style(Style::default().fg(palette.warn))
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(p, vertical[current_idx]);
        }
        current_idx += 1;
    }

    if has_home_warning {
        let msg = "You are running AutoGPT in your home directory.\nIt is recommended to run AutoGPT from a project-specific directory\nso that generated files are scoped correctly.";
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(Style::default().fg(palette.err))
            .style(Style::default().bg(palette.bg));

        let p = Paragraph::new(msg)
            .style(Style::default().fg(palette.err))
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(p, vertical[current_idx]);
        current_idx += 1;
    }

    let top_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(65), Constraint::Fill(1)])
        .split(vertical[current_idx]);

    let logo_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(palette.bg));
    let logo_inner = logo_block.inner(top_split[0]);
    frame.render_widget(logo_block, top_split[0]);

    let logo_lines = render_logo(palette);
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(Alignment::Left),
        logo_inner,
    );

    render_stats_pane(frame, top_split[1], state, palette);

    let mid_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(vertical[current_idx + 1]);

    render_log_pane(frame, mid_split[0], state, palette);
    render_task_pane(frame, mid_split[1], state, palette);

    let input_area = vertical[current_idx + 2];
    render_input_bar(frame, input_area, state, palette);

    if state.slash_autocomplete_active && !state.slash_matches.is_empty() {
        render_slash_popup(frame, input_area, state, palette);
    }
}

/// Renders the session stats panel (provider, model, token counts, YOLO) in the top-right corner.
fn render_stats_pane(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Session Stats",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default()),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // let yolo_text = if state.settings_yolo { "⚡ ON" } else { "OFF" };
    // let yolo_color = if state.settings_yolo { palette.warn } else { palette.muted };
    // let internet_text = if state.settings_internet { "🌐 ON" } else { "✗ OFF" };
    // let internet_color = if state.settings_internet { palette.ok } else { palette.muted };

    let lines = vec![
        Line::from(vec![
            Span::styled("  Provider   ", Style::default().fg(palette.muted)),
            Span::styled(
                state.settings_provider.clone(),
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  Model  ", Style::default().fg(palette.muted)),
            Span::styled(
                if state.settings_model.is_empty() {
                    "default".to_string()
                } else {
                    state.settings_model.clone()
                },
                Style::default().fg(palette.chart_1),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Requests   ", Style::default().fg(palette.muted)),
            Span::styled(
                state.stats.requests.to_string(),
                Style::default().fg(palette.fg),
            ),
            Span::styled("  Responses  ", Style::default().fg(palette.muted)),
            Span::styled(
                state.stats.responses.to_string(),
                Style::default().fg(palette.fg),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tokens ↑   ", Style::default().fg(palette.muted)),
            Span::styled(
                state.stats.tokens_sent.to_string(),
                Style::default().fg(palette.chart_1),
            ),
            Span::styled("  Tokens ↓   ", Style::default().fg(palette.muted)),
            Span::styled(
                state.stats.tokens_received.to_string(),
                Style::default().fg(palette.chart_2),
            ),
        ]),
        // Line::from(vec![
        //     Span::styled("  Yolo       ", Style::default().fg(palette.muted)),
        //     Span::styled(yolo_text, Style::default().fg(yolo_color).add_modifier(Modifier::BOLD)),
        //     Span::styled("  Internet   ", Style::default().fg(palette.muted)),
        //     Span::styled(internet_text, Style::default().fg(internet_color)),
        // ]),
        Line::from(vec![
            Span::styled("  Tasks      ", Style::default().fg(palette.muted)),
            Span::styled(
                format!(
                    "{} / {}",
                    state
                        .tasks
                        .iter()
                        .filter(|t| t.status == TaskStatus::Completed)
                        .count(),
                    state.total_tasks
                ),
                Style::default().fg(palette.ok),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

/// Choose the appropriate ratatui `Style` and rendering mode for a single
/// activity-log line based on its structured markdown token prefix.
fn log_line_style(s: &str, palette: &ThemePalette) -> (Style, u8) {
    if s.starts_with("❯ ") || s.starts_with("◆ ") {
        (
            Style::default()
                .fg(palette.accent)
                .add_modifier(Modifier::BOLD),
            0,
        )
    } else if s.starts_with("  [code]") {
        (Style::default().fg(palette.chart_2), 1)
    } else if s.starts_with("  [table]") {
        (Style::default().fg(palette.fg), 2)
    } else if s.starts_with("  [quote]") {
        (
            Style::default()
                .fg(palette.muted)
                .add_modifier(Modifier::ITALIC),
            3,
        )
    } else if s.to_lowercase().contains("error")
        || s.starts_with('\u{2717}')
        || s.starts_with('\u{26a0}')
    {
        (Style::default().fg(palette.err), 0)
    } else if s.to_lowercase().contains("success")
        || s.starts_with('\u{2713}')
        || s.starts_with("\u{2705}")
    {
        (Style::default().fg(palette.ok), 0)
    } else if s.starts_with('\u{2500}') {
        (Style::default().fg(palette.border), 0)
    } else if s.starts_with("- ") || s.starts_with("* ") {
        (Style::default().fg(palette.chart_1), 0)
    } else {
        (Style::default().fg(palette.fg), 0)
    }
}

/// Renders the scrollable activity-log panel on the left half of the main tab.
fn render_log_pane(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Activity Log",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "  PgUp/PgDn scroll  ←/→ horizontal ",
                Style::default().fg(palette.muted),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let height = inner.height as usize;
    let width = inner.width as usize;
    let total = state.log_lines.len();
    let visible_start = if total > height {
        total
            .saturating_sub(height)
            .saturating_sub(state.log_scroll_offset)
    } else {
        0
    };
    let h_offset = state.log_h_scroll_offset;
    let log_lines: Vec<Line> = state
        .log_lines
        .iter()
        .skip(visible_start)
        .take(height)
        .map(|s| {
            let (style, render_mode) = log_line_style(s, palette);
            match render_mode {
                1 => {
                    let inner = s.trim_start_matches("  [code]");
                    let visible: String = inner.chars().skip(h_offset).take(width).collect();
                    Line::from(Span::styled(format!("  {visible}"), style))
                }
                2 => {
                    let row_text = s.trim_start_matches("  [table]");
                    let visible: String = row_text.chars().skip(h_offset).take(width).collect();
                    let trimmed = visible.trim().trim_matches('|');
                    let cells: Vec<&str> = trimmed.split('|').collect();
                    let mut spans = Vec::new();
                    spans.push(Span::styled("|", Style::default().fg(palette.border)));
                    for (i, cell) in cells.iter().enumerate() {
                        spans.push(Span::styled(cell.to_string(), style));
                        spans.push(Span::styled("|", Style::default().fg(palette.border)));
                        let _ = i;
                    }
                    Line::from(spans)
                }
                3 => {
                    let inner = s.trim_start_matches("  [quote]");
                    let visible: String = inner.chars().skip(h_offset).take(width).collect();
                    Line::from(vec![
                        Span::styled("  ▌ ", Style::default().fg(palette.border)),
                        Span::styled(visible, style),
                    ])
                }
                _ => {
                    let visible: String = s.chars().skip(h_offset).take(width).collect();
                    Line::from(Span::styled(visible, style))
                }
            }
        })
        .collect();

    frame.render_widget(Paragraph::new(log_lines), inner);
}

/// Renders the task-progress list on the right half of the main tab.
fn render_task_pane(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Tasks",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                if state.total_tasks > 0 {
                    format!(
                        " {}/{}",
                        state
                            .tasks
                            .iter()
                            .filter(|t| t.status == TaskStatus::Completed)
                            .count(),
                        state.total_tasks
                    )
                } else {
                    String::new()
                },
                Style::default().fg(palette.muted),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.tasks.is_empty() {
        let placeholder = Paragraph::new(Line::from(Span::styled(
            "  No tasks yet. Enter a prompt below.",
            Style::default().fg(palette.muted),
        )));
        frame.render_widget(placeholder, inner);
        return;
    }

    let items: Vec<ListItem> = state
        .tasks
        .iter()
        .skip(state.task_scroll_offset)
        .map(|row| {
            let (icon, color) = match row.status {
                TaskStatus::Pending => ("○", palette.muted),
                TaskStatus::InProgress => ("●", palette.warn),
                TaskStatus::Completed => ("✓", palette.ok),
                TaskStatus::Failed => ("✗", palette.err),
                TaskStatus::Skipped => ("⊘", palette.muted),
            };
            let desc: String = row
                .description
                .chars()
                .take(area.width as usize - 6)
                .collect();
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!(" {} ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(desc, Style::default().fg(palette.fg)),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), inner);
}

/// Renders the command-input bar at the bottom of the main tab.
///
/// Shows a placeholder when empty, an inline ghost-text autocomplete hint
/// when `/` autocomplete is active, and positions the terminal cursor.
fn render_input_bar(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.input_bg))
        .title(Line::from(vec![Span::styled(
            " Request ",
            Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
        )]))
        .title_alignment(Alignment::Left);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = state.input_buffer.value();
    let cursor_pos = state.input_buffer.visual_cursor();
    let display_width = inner.width as usize;
    let start = cursor_pos.saturating_sub(display_width.saturating_sub(3));
    let visible: String = input_text
        .chars()
        .skip(start)
        .take(display_width - 2)
        .collect();

    let placeholder_visible = visible.is_empty();
    let line = if placeholder_visible {
        Line::from(vec![
            Span::styled(
                "❯ ",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "Type your request, or /help for commands...",
                Style::default().fg(palette.muted),
            ),
        ])
    } else {
        let text = visible.as_str();
        let ghost = if state.slash_autocomplete_active && !state.slash_matches.is_empty() {
            let full = state.slash_matches[state.slash_match_idx];
            full.strip_prefix(text).unwrap_or("")
        } else {
            ""
        };
        Line::from(vec![
            Span::styled(
                "❯ ",
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(visible.clone(), Style::default().fg(palette.fg)),
            Span::styled(ghost.to_string(), Style::default().fg(palette.muted)),
        ])
    };

    frame.render_widget(Paragraph::new(line), inner);

    let input_prefix = "\u{276F} ";
    let prefix_width = UnicodeWidthStr::width(input_prefix) as u16;
    let cursor_x = inner.x + prefix_width + (cursor_pos - start).min(display_width - 3) as u16;
    frame.set_cursor_position((cursor_x, inner.y));
}

/// Renders the floating slash-command autocomplete popup above the input bar.
fn render_slash_popup(
    frame: &mut Frame,
    input_area: Rect,
    state: &TuiState,
    palette: &ThemePalette,
) {
    let popup_height = (state.slash_matches.len() as u16).min(8) + 2;
    let popup_y = input_area.y.saturating_sub(popup_height);
    let popup_x = input_area.x + 2;
    let popup_w = 30u16.min(input_area.width.saturating_sub(4));

    let popup_area = Rect {
        x: popup_x,
        y: popup_y,
        width: popup_w,
        height: popup_height,
    };

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.accent))
        .style(Style::default().bg(palette.bg))
        .title(Span::styled(
            " Commands ",
            Style::default().fg(palette.muted),
        ));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    let items: Vec<ListItem> = state
        .slash_matches
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let is_selected = i == state.slash_match_idx;
            let style = if is_selected {
                Style::default()
                    .fg(palette.tab_active_fg)
                    .bg(palette.tab_active_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.fg)
            };
            ListItem::new(Line::from(Span::styled(format!("  {}", cmd), style)))
        })
        .collect();

    frame.render_widget(List::new(items).highlight_style(Style::default()), inner);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
