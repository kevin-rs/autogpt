// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::cli::settings::{GlobalSettings, SettingsManager},
    crate::common::utils::is_yes,
    crate::tui::{
        state::{AppTab, TuiEvent, TuiState},
        tabs::{
            analytics::render_analytics_tab,
            main::{render_main_tab, render_status_bar, render_tab_bar},
            prompts::render_prompts_tab,
            settings::render_settings_tab,
            theme::render_theme_tab,
        },
        theme::{ALL_THEME_VARIANTS, ThemeVariant},
    },
    anyhow::Result,
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    ratatui::{
        Terminal,
        backend::CrosstermBackend,
        layout::{Constraint, Direction, Layout},
    },
    std::process::Command,
    std::sync::Arc,
    std::sync::atomic::{AtomicBool, Ordering},
    std::{io, time::Duration},
    tokio::sync::mpsc::{Sender, UnboundedReceiver},
    tui_input::backend::crossterm::EventHandler,
};

/// Main application controller for the TUI.
pub struct TuiApp {
    /// The terminal abstraction.
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Mutable application state.
    state: TuiState,
    /// Token for interrupting agent execution.
    abort_token: Arc<AtomicBool>,
}

impl TuiApp {
    /// Initialize a new TUI application.
    pub fn new(
        receiver: UnboundedReceiver<TuiEvent>,
        settings: &GlobalSettings,
        abort_token: Arc<AtomicBool>,
        update_available: Option<(String, String)>,
    ) -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let state = TuiState::new(receiver, settings, update_available);

        Ok(Self {
            terminal,
            state,
            abort_token,
        })
    }

    /// Main event loop for the TUI.
    pub fn run(mut self, input_tx: Sender<String>) -> Result<()> {
        self.terminal.hide_cursor()?;

        loop {
            self.state.tick_count = self.state.tick_count.wrapping_add(1);
            self.state.tick();
            self.state.process_events();

            let palette = self.state.theme_config.palette();

            self.terminal.draw(|frame| {
                let area = frame.area();

                let vertical = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(2),
                        Constraint::Fill(1),
                        Constraint::Length(2),
                    ])
                    .split(area);

                render_tab_bar(frame, vertical[0], &self.state, &palette);

                match self.state.active_tab {
                    AppTab::Main => render_main_tab(frame, vertical[1], &self.state, &palette),
                    AppTab::Analytics => {
                        render_analytics_tab(frame, vertical[1], &self.state, &palette)
                    }
                    AppTab::Theme => render_theme_tab(frame, vertical[1], &self.state, &palette),
                    AppTab::Settings => {
                        render_settings_tab(frame, vertical[1], &self.state, &palette)
                    }
                    AppTab::Prompts => {
                        render_prompts_tab(frame, vertical[1], &self.state, &palette)
                    }
                }

                render_status_bar(frame, vertical[2], &self.state, &palette);
            })?;

            if self.state.should_quit {
                break;
            }

            if event::poll(Duration::from_millis(50))?
                && let Ok(Event::Key(key)) = event::read()
                && key.kind == KeyEventKind::Press
            {
                self.handle_key(key.code, key.modifiers, &input_tx)?;
            }
        }

        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Global key handler.
    fn handle_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        input_tx: &Sender<String>,
    ) -> Result<()> {
        if self.state.prompt_editing {
            return self.handle_prompt_edit_key(code, modifiers);
        }

        match code {
            KeyCode::Char('q') if modifiers.is_empty() => {
                self.state.should_quit = true;
            }
            KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.should_quit = true;
            }
            KeyCode::Tab => {
                self.state.active_tab = self.state.active_tab.next();
            }
            KeyCode::BackTab => {
                self.state.active_tab = self.state.active_tab.prev();
            }
            KeyCode::Char('1') if modifiers.contains(KeyModifiers::ALT) => {
                self.state.active_tab = AppTab::Main
            }
            KeyCode::Char('2') if modifiers.contains(KeyModifiers::ALT) => {
                self.state.active_tab = AppTab::Analytics
            }
            KeyCode::Char('3') if modifiers.contains(KeyModifiers::ALT) => {
                self.state.active_tab = AppTab::Theme
            }
            KeyCode::Char('4') if modifiers.contains(KeyModifiers::ALT) => {
                self.state.active_tab = AppTab::Settings
            }
            KeyCode::Char('5') if modifiers.contains(KeyModifiers::ALT) => {
                self.state.active_tab = AppTab::Prompts
            }
            _ => match self.state.active_tab {
                AppTab::Main => self.handle_main_key(code, modifiers, input_tx),
                AppTab::Theme => self.handle_theme_key(code),
                AppTab::Settings => self.handle_settings_key(code, modifiers),
                AppTab::Prompts => self.handle_prompts_key(code, modifiers),
                AppTab::Analytics => {}
            },
        }
        Ok(())
    }

    /// Input and navigation handler for the Main tab.
    fn handle_main_key(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
        input_tx: &Sender<String>,
    ) {
        if self.state.slash_autocomplete_active {
            match code {
                KeyCode::Up => {
                    if self.state.slash_match_idx > 0 {
                        self.state.slash_match_idx -= 1;
                    } else {
                        self.state.slash_match_idx =
                            self.state.slash_matches.len().saturating_sub(1);
                    }
                    return;
                }
                KeyCode::Down | KeyCode::Tab => {
                    let len = self.state.slash_matches.len();
                    self.state.slash_match_idx = (self.state.slash_match_idx + 1) % len.max(1);
                    return;
                }
                KeyCode::Enter => {
                    if let Some(&cmd) = self.state.slash_matches.get(self.state.slash_match_idx) {
                        self.state.input_buffer =
                            tui_input::Input::default().with_value(cmd.to_string());
                        let move_end =
                            Event::Key(KeyEvent::new(KeyCode::End, KeyModifiers::empty()));
                        self.state.input_buffer.handle_event(&move_end);
                        self.state.slash_autocomplete_active = false;
                        self.state.slash_matches.clear();
                    }
                    return;
                }
                KeyCode::Esc => {
                    self.state.slash_autocomplete_active = false;
                    self.state.slash_matches.clear();
                    return;
                }
                _ => {}
            }
        }

        match code {
            KeyCode::Enter => {
                let text = self.state.input_buffer.value().trim().to_string();
                if !text.is_empty() {
                    if self.state.sessions_picking {
                        if text == "0" || text.eq_ignore_ascii_case("cancel") {
                            self.state.sessions_picking = false;
                            self.state.sessions_list.clear();
                            self.state.input_buffer = tui_input::Input::default();
                            return;
                        }
                        if let Ok(n) = text.parse::<usize>()
                            && (1..=self.state.sessions_list.len()).contains(&n)
                        {
                            let session_id = self.state.sessions_list[n - 1].0.clone();
                            let session_title = self.state.sessions_list[n - 1].1.clone();
                            self.state.sessions_picking = false;
                            self.state.sessions_list.clear();
                            self.state
                                .push_log(format!("▸ Resuming session: {}", session_title));
                            let _ = input_tx.try_send(format!("/resume {}", session_id));
                            self.state.input_buffer = tui_input::Input::default();
                            return;
                        }
                        self.state
                            .push_log("Invalid selection. Type a number or 'cancel'.".to_string());
                        self.state.input_buffer = tui_input::Input::default();
                        return;
                    }

                    let is_approval = self.state.agent_mode_label == "Awaiting approval";
                    self.state.push_log(format!("❯ {}", text));

                    if is_approval && is_yes(&text) {
                        self.state.agent_mode_label = "Executing".to_string();
                    } else if !is_approval && !text.starts_with('/') {
                        self.state.agent_mode_label = "Synthesizing".to_string();
                        // self.state.tasks.clear();
                        // self.state.total_tasks = 0;
                    }

                    let _ = input_tx.try_send(text);
                    self.state.input_buffer = tui_input::Input::default();
                    self.state.log_scroll_offset = 0;
                    self.state.log_h_scroll_offset = 0;
                    self.state.slash_autocomplete_active = false;
                    self.state.slash_matches.clear();
                }
            }
            KeyCode::Esc => {
                if self.state.sessions_picking {
                    self.state.sessions_picking = false;
                    self.state.sessions_list.clear();
                    self.state
                        .push_log("Session selection cancelled.".to_string());
                } else {
                    self.abort_token.store(true, Ordering::SeqCst);
                }
            }
            KeyCode::PageUp => {
                let height: usize = 20;
                self.state.log_scroll_offset = self
                    .state
                    .log_scroll_offset
                    .saturating_add(height.saturating_sub(2));
                let max_scroll = self.state.log_lines.len().saturating_sub(5);
                if self.state.log_scroll_offset > max_scroll {
                    self.state.log_scroll_offset = max_scroll;
                }
            }
            KeyCode::PageDown => {
                let height: usize = 20;
                self.state.log_scroll_offset = self
                    .state
                    .log_scroll_offset
                    .saturating_sub(height.saturating_sub(2));
            }
            KeyCode::Up if modifiers.is_empty() => {
                self.state.log_scroll_offset = self.state.log_scroll_offset.saturating_add(1);
                let max_scroll = self.state.log_lines.len().saturating_sub(5);
                if self.state.log_scroll_offset > max_scroll {
                    self.state.log_scroll_offset = max_scroll;
                }
            }
            KeyCode::Down if modifiers.is_empty() => {
                self.state.log_scroll_offset = self.state.log_scroll_offset.saturating_sub(1);
            }
            KeyCode::Up if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.task_scroll_offset = self.state.task_scroll_offset.saturating_sub(1);
            }
            KeyCode::Down if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.task_scroll_offset = self.state.task_scroll_offset.saturating_add(1);
            }
            KeyCode::Left if modifiers.is_empty() => {
                self.state.log_h_scroll_offset = self.state.log_h_scroll_offset.saturating_sub(8);
            }
            KeyCode::Right if modifiers.is_empty() => {
                self.state.log_h_scroll_offset = self.state.log_h_scroll_offset.saturating_add(8);
            }
            KeyCode::Tab => {
                let text = self.state.input_buffer.value().to_string();
                if text.is_empty() {
                    self.state.active_tab = self.state.active_tab.next();
                }
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                let clipboard = get_clipboard();
                if !clipboard.is_empty() {
                    let mut val = self.state.input_buffer.value().to_string();
                    let cursor = self.state.input_buffer.cursor();
                    val.insert_str(cursor, &clipboard);
                    self.state.input_buffer =
                        tui_input::Input::from(val).with_cursor(cursor + clipboard.len());
                    self.state.update_slash_autocomplete();
                }
            }
            other => {
                let ev = Event::Key(KeyEvent::new(other, modifiers));
                self.state.input_buffer.handle_event(&ev);
                self.state.update_slash_autocomplete();
            }
        }
    }

    /// Handle keystrokes while the Theme tab is active.
    fn handle_theme_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up if self.state.selected_theme_idx > 0 => {
                self.state.selected_theme_idx -= 1;
            }
            KeyCode::Down if self.state.selected_theme_idx + 1 < ALL_THEME_VARIANTS.len() => {
                self.state.selected_theme_idx += 1;
            }
            KeyCode::Enter => {
                let variant = ALL_THEME_VARIANTS[self.state.selected_theme_idx];
                self.state.apply_theme_variant(variant);
            }
            KeyCode::Char('r') => {
                self.state.apply_theme_variant(ThemeVariant::Dark);
                self.state.selected_theme_idx = 0;
            }
            _ => {}
        }
    }

    /// Handle keystrokes while the Settings tab is active.
    ///
    /// Toggles booleans for fields 0-4 and delegates text input to the
    /// focused input widget for fields 5-8.
    fn handle_settings_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let total_fields = 9;

        if self.state.settings_focus_idx >= 5 {
            let handled = match code {
                KeyCode::Up => {
                    self.state.settings_focus_idx = self.state.settings_focus_idx.saturating_sub(1);
                    true
                }
                KeyCode::Down => {
                    if self.state.settings_focus_idx + 1 < total_fields {
                        self.state.settings_focus_idx += 1;
                    }
                    true
                }
                KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                    self.save_settings();
                    true
                }
                _ => false,
            };

            if !handled {
                let ev =
                    crossterm::event::Event::Key(crossterm::event::KeyEvent::new(code, modifiers));
                match self.state.settings_focus_idx {
                    5 => {
                        if code == KeyCode::Char('v') && modifiers.contains(KeyModifiers::CONTROL) {
                            let clipboard = get_clipboard();
                            let mut val = self.state.settings_provider_input.value().to_string();
                            let cursor = self.state.settings_provider_input.cursor();
                            val.insert_str(cursor, &clipboard);
                            self.state.settings_provider_input =
                                tui_input::Input::from(val).with_cursor(cursor + clipboard.len());
                        } else {
                            self.state.settings_provider_input.handle_event(&ev);
                        }
                    }
                    6 => {
                        if code == KeyCode::Char('v') && modifiers.contains(KeyModifiers::CONTROL) {
                            let clipboard = get_clipboard();
                            let mut val = self.state.settings_model_input.value().to_string();
                            let cursor = self.state.settings_model_input.cursor();
                            val.insert_str(cursor, &clipboard);
                            self.state.settings_model_input =
                                tui_input::Input::from(val).with_cursor(cursor + clipboard.len());
                        } else {
                            self.state.settings_model_input.handle_event(&ev);
                        }
                    }
                    7 => {
                        self.state.settings_retries_input.handle_event(&ev);
                    }
                    8 => {
                        if code == KeyCode::Char('v') && modifiers.contains(KeyModifiers::CONTROL) {
                            let clipboard = get_clipboard();
                            let mut val = self.state.settings_workspace_input.value().to_string();
                            let cursor = self.state.settings_workspace_input.cursor();
                            val.insert_str(cursor, &clipboard);
                            self.state.settings_workspace_input =
                                tui_input::Input::from(val).with_cursor(cursor + clipboard.len());
                        } else {
                            self.state.settings_workspace_input.handle_event(&ev);
                        }
                    }
                    _ => {}
                }
                self.state.settings_provider =
                    self.state.settings_provider_input.value().to_string();
                self.state.settings_model = self.state.settings_model_input.value().to_string();
                if let Ok(val) = self.state.settings_retries_input.value().parse::<u8>() {
                    self.state.settings_max_retries = val;
                }
                self.state.settings_workspace =
                    self.state.settings_workspace_input.value().to_string();
            }
            return;
        }

        match code {
            KeyCode::Char(' ') | KeyCode::Enter => match self.state.settings_focus_idx {
                0 => self.state.settings_yolo = !self.state.settings_yolo,
                1 => self.state.settings_internet = !self.state.settings_internet,
                2 => self.state.settings_auto_browse = !self.state.settings_auto_browse,
                3 => self.state.settings_verbose = !self.state.settings_verbose,
                4 => self.state.settings_metacognition = !self.state.settings_metacognition,
                _ => {}
            },
            KeyCode::Up if self.state.settings_focus_idx > 0 => {
                self.state.settings_focus_idx -= 1;
            }
            KeyCode::Down if self.state.settings_focus_idx + 1 < total_fields => {
                self.state.settings_focus_idx += 1;
            }
            KeyCode::Char('s') => {
                self.save_settings();
            }
            _ => {}
        }
    }

    /// Persist all settings fields to disk via `SettingsManager`.
    fn save_settings(&self) {
        let mgr = SettingsManager::new();
        if let Ok(mut s) = mgr.load() {
            s.yolo = self.state.settings_yolo;
            s.internet_access = self.state.settings_internet;
            s.auto_browse = self.state.settings_auto_browse;
            s.verbose = self.state.settings_verbose;
            s.metacognition = self.state.settings_metacognition;
            s.max_retries = self.state.settings_max_retries;
            s.provider = self.state.settings_provider.clone();
            if !self.state.settings_model.is_empty() {
                s.model = Some(self.state.settings_model.clone());
            }
            let _ = mgr.save(&s);
        }
    }

    /// Handle keystrokes while the Prompts tab is active (list navigation,
    /// edit, and reset).
    fn handle_prompts_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) {
        let total = TuiState::prompt_names().len();
        match code {
            KeyCode::Up if self.state.selected_prompt_idx > 0 => {
                self.state.selected_prompt_idx -= 1;
            }
            KeyCode::Down if self.state.selected_prompt_idx + 1 < total => {
                self.state.selected_prompt_idx += 1;
            }
            KeyCode::Char('e') => {
                let text = self.state.get_prompt_text(self.state.selected_prompt_idx);
                self.state.prompt_edit_buffer = tui_input::Input::from(text);
                self.state.prompt_editing = true;
            }
            KeyCode::Char('r') => {
                self.state.reset_prompt();
            }
            _ => {}
        }
    }

    /// Handle keystrokes while the prompt text editor is active.
    ///
    /// Supports cursor navigation, `Ctrl+S` to save, `Esc` to cancel,
    /// `Alt+Enter` to insert a newline, and `r` to reset.
    fn handle_prompt_edit_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match code {
            KeyCode::Up => {
                let pos = self.state.prompt_edit_buffer.cursor();
                let text = self.state.prompt_edit_buffer.value();
                let before = &text[..pos];
                if let Some(last_nl) = before.rfind('\n') {
                    let prev_line_start = before[..last_nl].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let col = pos - last_nl - 1;
                    let target = (prev_line_start + col).min(last_nl);
                    self.state.prompt_edit_buffer =
                        self.state.prompt_edit_buffer.clone().with_cursor(target);
                }

                self.recalculate_prompt_scroll();
            }
            KeyCode::Down => {
                let pos = self.state.prompt_edit_buffer.cursor();
                let text = self.state.prompt_edit_buffer.value();
                let after = &text[pos..];
                if let Some(next_nl) = after.find('\n') {
                    let current_line_start = text[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let col = pos - current_line_start;
                    let next_line_start = pos + next_nl + 1;
                    let next_line_end = text[next_line_start..]
                        .find('\n')
                        .map(|i| next_line_start + i)
                        .unwrap_or(text.len());
                    let target = (next_line_start + col).min(next_line_end);
                    self.state.prompt_edit_buffer =
                        self.state.prompt_edit_buffer.clone().with_cursor(target);
                }

                self.recalculate_prompt_scroll();
            }
            KeyCode::PageUp => {
                self.state.prompt_scroll_offset =
                    self.state.prompt_scroll_offset.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.state.prompt_scroll_offset =
                    self.state.prompt_scroll_offset.saturating_add(10);
            }
            KeyCode::Esc => {
                self.state.prompt_editing = false;
            }
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                self.state.save_prompt_edit();
            }
            KeyCode::Enter if modifiers.contains(KeyModifiers::ALT) => {
                let pos = self.state.prompt_edit_buffer.cursor();
                let mut val = self.state.prompt_edit_buffer.value().to_string();
                val.insert(pos, '\n');
                self.state.prompt_edit_buffer = tui_input::Input::from(val).with_cursor(pos + 1);
            }
            KeyCode::Char('r') if modifiers.is_empty() => {
                self.state.prompt_editing = false;
                self.state.reset_prompt();
            }
            KeyCode::Char('v') if modifiers.contains(KeyModifiers::CONTROL) => {
                let clipboard = get_clipboard();
                if !clipboard.is_empty() {
                    let mut val = self.state.prompt_edit_buffer.value().to_string();
                    let cursor = self.state.prompt_edit_buffer.cursor();
                    val.insert_str(cursor, &clipboard);
                    self.state.prompt_edit_buffer =
                        tui_input::Input::from(val).with_cursor(cursor + clipboard.len());
                }
            }
            other => {
                let ev = Event::Key(KeyEvent::new(other, modifiers));
                self.state.prompt_edit_buffer.handle_event(&ev);
            }
        }
        Ok(())
    }

    /// Recalculate `prompt_scroll_offset` to keep the cursor visible.
    fn recalculate_prompt_scroll(&mut self) {
        let pos = self.state.prompt_edit_buffer.cursor();
        let text = self.state.prompt_edit_buffer.value();
        let before = &text[..pos.min(text.len())];
        let cur_row = before.chars().filter(|&c| c == '\n').count() as u16;

        let visible_height = 18;
        if cur_row < self.state.prompt_scroll_offset {
            self.state.prompt_scroll_offset = cur_row;
        } else if cur_row >= self.state.prompt_scroll_offset + visible_height {
            self.state.prompt_scroll_offset = cur_row - visible_height + 1;
        }
    }
}

fn get_clipboard() -> String {
    let commands = [
        ("xclip", vec!["-selection", "clipboard", "-o"]),
        ("xsel", vec!["-ob"]),
        ("wl-paste", vec![]),
    ];

    for (cmd, args) in commands {
        if let Ok(output) = Command::new(cmd).args(args).output()
            && output.status.success()
        {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    String::new()
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
