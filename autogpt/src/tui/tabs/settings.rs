// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::tui::state::TuiState,
    crate::tui::theme::ThemePalette,
    ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Modifier, Style},
        symbols,
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    },
    unicode_width::{UnicodeWidthChar, UnicodeWidthStr},
};

/// Renders the settings tab.
///
/// Splits horizontally into a boolean toggles panel (left) and a text-input
/// configuration panel (right).
pub fn render_settings_tab(
    frame: &mut Frame,
    area: Rect,
    state: &TuiState,
    palette: &ThemePalette,
) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    render_toggle_settings(frame, split[0], state, palette);
    render_text_settings(frame, split[1], state, palette);
}

/// Renders the left settings pane: boolean toggles for YOLO, Internet,
/// Auto-browse, and Verbose mode.
fn render_toggle_settings(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Behavior Toggles",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(5)])
        .split(inner);

    let toggles = [
        (
            "⚡ Yolo Mode",
            state.settings_yolo,
            "Auto-approve all agent actions",
        ),
        (
            "🌐 Internet",
            state.settings_internet,
            "Allow web search during tasks",
        ),
        (
            "🔍 Auto-browse",
            state.settings_auto_browse,
            "Open browser when app is ready",
        ),
        (
            "📝 Verbose",
            state.settings_verbose,
            "Show extra debug logging",
        ),
        (
            "🧠 Metacognition",
            state.settings_metacognition,
            "Periodic self-assessment between tasks",
        ),
    ];

    let items: Vec<ListItem> = toggles
        .iter()
        .enumerate()
        .map(|(i, (label, enabled, description))| {
            let is_selected = state.settings_focus_idx == i;
            let (check, color) = if *enabled {
                ("[✓]", palette.ok)
            } else {
                ("[ ]", palette.muted)
            };

            let label_style = if is_selected {
                Style::default()
                    .fg(palette.tab_active_fg)
                    .bg(palette.tab_active_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD)
            };

            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(
                        format!("  {} ", check),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(label.to_string(), label_style),
                ]),
                Line::from(Span::styled(
                    format!("      {}", description),
                    Style::default().fg(palette.muted),
                )),
                Line::from(Span::raw("")),
            ])
        })
        .collect();

    frame.render_widget(List::new(items), vertical[0]);

    let hints = Paragraph::new(vec![
        Line::from(Span::styled(
            "  ↑ ↓  navigate  │  Space  toggle  │  s  save",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  Changes saved to ~/.autogpt/settings.json",
            Style::default().fg(palette.muted),
        )),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(hints, vertical[1]);
}

/// Renders the right settings pane: editable text fields for Provider, Model,
/// Max Retries, and Workspace. Positions the terminal cursor in the active field.
fn render_text_settings(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Agent Configuration",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut all_lines: Vec<Line> = vec![Line::from(Span::raw(""))];

    let fields = vec![
        (
            "  Provider       ",
            state.settings_provider_input.value(),
            palette.accent,
            state.settings_focus_idx == 5,
        ),
        (
            "  Model          ",
            state.settings_model_input.value(),
            palette.chart_1,
            state.settings_focus_idx == 6,
        ),
        (
            "  Max Retries    ",
            state.settings_retries_input.value(),
            palette.fg,
            state.settings_focus_idx == 7,
        ),
        (
            "  Workspace      ",
            state.settings_workspace_input.value(),
            palette.fg,
            state.settings_focus_idx == 8,
        ),
    ];

    let mut cursor_pos = None;
    let mut current_y = inner.y + 1;

    for (label, value, color, is_selected) in fields {
        all_lines.push(Line::from(Span::styled(
            label.to_string(),
            Style::default().fg(palette.muted),
        )));

        let val_disp = if value.is_empty() { "default" } else { value };
        let mut val_style = Style::default().fg(color).add_modifier(Modifier::BOLD);
        if is_selected {
            val_style = val_style
                .fg(palette.tab_active_fg)
                .bg(palette.tab_active_bg);

            let prefix = "    ";
            let prefix_cols = UnicodeWidthStr::width(prefix) as u16;
            let input_obj = match state.settings_focus_idx {
                5 => &state.settings_provider_input,
                6 => &state.settings_model_input,
                7 => &state.settings_retries_input,
                8 => &state.settings_workspace_input,
                _ => &state.settings_provider_input,
            };
            let visual_idx = input_obj.visual_cursor();
            let cursor_col: u16 = input_obj
                .value()
                .chars()
                .take(visual_idx)
                .map(|c| UnicodeWidthChar::width(c).unwrap_or(1) as u16)
                .sum();
            cursor_pos = Some((inner.x + prefix_cols + cursor_col, current_y + 1));
        }

        all_lines.push(Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(val_disp.to_string(), val_style),
        ]));
        all_lines.push(Line::from(Span::styled(
            "  ───────────────────────",
            Style::default().fg(palette.border),
        )));

        current_y += 3;
    }

    frame.render_widget(Paragraph::new(all_lines).wrap(Wrap { trim: true }), inner);

    if let Some((cx, cy)) = cursor_pos {
        frame.set_cursor_position((cx, cy));
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
