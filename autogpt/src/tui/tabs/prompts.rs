// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::prompts::generic::{
        FOLLOWUP_SYNTHESIS_PROMPT, GENERIC_SYSTEM_PROMPT, IMPLEMENTATION_PLAN_PROMPT,
        INTENT_DETECTION_PROMPT, LESSON_EXTRACTION_PROMPT, REASONING_PROMPT, REFLECTION_PROMPT,
        STATE_SUMMARIZATION_PROMPT, TASK_EXECUTION_PROMPT, TASK_SYNTHESIS_PROMPT,
        WALKTHROUGH_PROMPT,
    },
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
    unicode_width::UnicodeWidthChar,
};

/// Renders the prompt editor tab.
pub fn render_prompts_tab(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(area);

    render_prompt_list(frame, split[0], state, palette);
    render_prompt_editor(frame, split[1], state, palette);
}

/// Renders the left prompt-list pane (prompt names + keyboard hint footer).
fn render_prompt_list(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Prompts",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let list_h = inner.height.saturating_sub(6);
    let list_area = Rect {
        height: list_h,
        ..inner
    };
    let hint_area = Rect {
        y: inner.y + list_h,
        height: 6,
        ..inner
    };

    let names = TuiState::prompt_names();
    let items: Vec<ListItem> = names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i == state.selected_prompt_idx;
            let has_override = state.get_prompt_text(i) != {
                match i {
                    0 => GENERIC_SYSTEM_PROMPT.to_string(),
                    1 => TASK_SYNTHESIS_PROMPT.to_string(),
                    2 => FOLLOWUP_SYNTHESIS_PROMPT.to_string(),
                    3 => IMPLEMENTATION_PLAN_PROMPT.to_string(),
                    4 => REASONING_PROMPT.to_string(),
                    5 => TASK_EXECUTION_PROMPT.to_string(),
                    6 => REFLECTION_PROMPT.to_string(),
                    7 => LESSON_EXTRACTION_PROMPT.to_string(),
                    8 => WALKTHROUGH_PROMPT.to_string(),
                    9 => STATE_SUMMARIZATION_PROMPT.to_string(),
                    _ => INTENT_DETECTION_PROMPT.to_string(),
                }
            };

            let style = if is_selected {
                Style::default()
                    .fg(palette.tab_active_fg)
                    .bg(palette.tab_active_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.fg)
            };
            let modified_mark = if has_override { " *" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", name), style),
                Span::styled(modified_mark, Style::default().fg(palette.warn)),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), list_area);

    let hints = Paragraph::new(vec![
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            "  ↑ ↓  select prompt",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  e    edit in panel",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  r    reset default",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  * = custom override",
            Style::default().fg(palette.warn),
        )),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(hints, hint_area);
}

/// Renders the right prompt-editor pane.
///
/// Shows the prompt text in read-only or edit mode. In edit mode it also
/// positions the terminal cursor at the caret location.
fn render_prompt_editor(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let prompt_name = TuiState::prompt_names()
        .get(state.selected_prompt_idx)
        .copied()
        .unwrap_or("Unknown");

    let border_color = if state.prompt_editing {
        palette.accent
    } else {
        palette.border
    };

    let title_suffix = if state.prompt_editing {
        " [EDITING] "
    } else {
        " [read-only] "
    };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                prompt_name,
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                title_suffix,
                Style::default().fg(if state.prompt_editing {
                    palette.warn
                } else {
                    palette.muted
                }),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content_area = Rect {
        height: inner.height.saturating_sub(2),
        ..inner
    };
    let hint_area = Rect {
        y: inner.y + inner.height.saturating_sub(2),
        height: 2,
        ..inner
    };

    let prompt_text = if state.prompt_editing {
        state.prompt_edit_buffer.value().to_string()
    } else {
        state.get_prompt_text(state.selected_prompt_idx)
    };

    let lines: Vec<Line> = prompt_text
        .lines()
        .map(|l| Line::from(Span::styled(l.to_string(), Style::default().fg(palette.fg))))
        .collect();

    frame.render_widget(
        Paragraph::new(lines)
            .scroll((state.prompt_scroll_offset, 0))
            .wrap(Wrap { trim: false }),
        content_area,
    );

    if state.prompt_editing {
        let byte_cursor = state.prompt_edit_buffer.cursor();
        let text_val = state.prompt_edit_buffer.value();

        let safe_cursor = if text_val.is_char_boundary(byte_cursor) {
            byte_cursor
        } else {
            let mut i = byte_cursor;
            while i > 0 && !text_val.is_char_boundary(i) {
                i -= 1;
            }
            i
        };

        let text_before = &text_val[..safe_cursor.min(text_val.len())];
        let panel_width = content_area.width.max(1) as usize;

        let mut cur_row: u16 = 0;
        let mut cur_col: u16 = 0;
        for ch in text_before.chars() {
            if ch == '\n' {
                cur_row += 1;
                cur_col = 0;
            } else {
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(1) as u16;
                cur_col += ch_width;
                if cur_col as usize >= panel_width {
                    cur_row += 1;
                    cur_col = 0;
                }
            }
        }

        let target_x = content_area.x + cur_col;
        let target_y = content_area.y + cur_row.saturating_sub(state.prompt_scroll_offset);
        if target_y >= content_area.y
            && target_y < content_area.y + content_area.height
            && target_x < content_area.x + content_area.width
        {
            frame.set_cursor_position((target_x, target_y));
        }
    }

    let hint_text = if state.prompt_editing {
        "  Ctrl+S: save  │  Esc: cancel edit  │  r: reset to default"
    } else {
        "  e: start editing  │  r: reset to default  │  ↑ ↓ PageUp PageDown: scroll"
    };

    frame.render_widget(
        Paragraph::new(Span::styled(hint_text, Style::default().fg(palette.muted))),
        hint_area,
    );
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
