// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::agents::agent::AgentGPT;
#[cfg(feature = "mop")]
use crate::agents::mop::run_mixture;
use crate::common::utils::{
    ClientType, ContextManager, Knowledge, Message, Persona, Planner, Reflection, Scope, Status,
    Task, TaskScheduler, Tool, is_yes, strip_code_blocks,
};
#[allow(unused_imports)]
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
#[cfg(feature = "cli")]
use crate::prelude::*;
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Functions, ReqResponse};
use async_trait::async_trait;
use auto_derive::Auto;
use serde::Deserialize;
use std::borrow::Cow;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc::Receiver};
use tracing::{error, info, warn};

#[cfg(feature = "net")]
use crate::collaboration::Collaborator;

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "cli")]
use {
    crate::cli::models::{default_model, default_provider, model_index, provider_models},
    crate::cli::readline::{ReadlineResult, SLASH_COMMANDS, read_line},
    crate::cli::session::{Session, SessionManager, SessionTask, TaskStatus as SessionTaskStatus},
    crate::cli::settings::SettingsManager,
    crate::cli::skills::SkillStore,
    crate::common::utils::{
        ActionRequest, ActionResult, AgentIntent, ReasoningResult, ReflectionOutcome,
        ReflectionResult,
    },
    crate::prompts::generic::{
        FOLLOWUP_SYNTHESIS_PROMPT, GENERIC_SYSTEM_PROMPT, IMPLEMENTATION_PLAN_PROMPT,
        INTENT_DETECTION_PROMPT, LESSON_EXTRACTION_PROMPT, REASONING_PROMPT, REFLECTION_PROMPT,
        STATE_SUMMARIZATION_PROMPT, TASK_EXECUTION_PROMPT, TASK_SYNTHESIS_PROMPT,
        WALKTHROUGH_PROMPT,
    },
    crate::tui::state::TuiEvent,
    crate::tui::utils::{
        TaskStatus as TuiTaskStatus, create_spinner, print_agent_msg, print_banner, print_error,
        print_greeting, print_section, print_success, print_task_item, print_warning,
        render_help_table, render_help_table_to_log, render_markdown, render_model_selector,
    },
    anyhow::anyhow,
    colored::Colorize,
    duckduckgo,
    std::env,
    std::fs,
    std::io::{self, BufRead, Write as IoWrite},
    std::path::{Path, PathBuf},
    std::process::Stdio,
    std::time::Duration,
    termimad::crossterm::event::{self, Event, KeyCode},
    termimad::crossterm::terminal::{disable_raw_mode, enable_raw_mode},
    tokio::io::{AsyncBufReadExt, BufReader},
    tokio::process::Command,
    tokio::sync::mpsc::UnboundedSender,
    tokio::time::{interval, sleep},
};

#[cfg(all(feature = "cli", feature = "mcp"))]
use {
    crate::cli::autogpt::commands::mcp as mcp_cmd,
    crate::mcp::client::McpClient,
    crate::mcp::types::McpServerInfo,
    crate::tui::utils::{
        render_mcp_help_entries, render_mcp_help_entries_to_log, render_mcp_inspect,
        render_mcp_inspect_to_log, render_mcp_list, render_mcp_list_to_log,
    },
};

#[cfg(all(feature = "cli", feature = "col"))]
use crate::agents::collab::{CollabPool, CollabSelection};

#[cfg(all(feature = "cli", feature = "mta"))]
use crate::prompts::generic::METACOGNITION_PROMPT;

#[cfg(feature = "cli")]
const MAX_CONSECUTIVE_FAILURES: u8 = 3;

#[derive(Deserialize)]
struct IntentResponse {
    intent: String,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    args: Option<serde_json::Value>,
}

#[cfg(all(feature = "cli", feature = "mta"))]
#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct MetacognitionResponse {
    assessment: String,
    adjustment: String,
    confidence: String,
    priority: String,
}

/// A general-purpose autonomous agent that operates interactively from the CLI.
///
/// `GenericAgent` is the default agent launched when running `autogpt` with no subcommand
/// (and without `--net`). It is invoked by the `run_generic_agent_loop` REPL - which reads
/// the user's prompt, creates a `Task` with it, and calls `Executor::execute`.
///
/// Inside `execute`, the agent runs the full pipeline:
///   1. Synthesise a numbered task list from the prompt.
///   2. Generate a markdown implementation plan.
///   3. Optionally gate on user approval (skipped in yolo mode via `_execute = false`).
///   4. Execute each task by prompting the LLM for `ActionRequest` JSON and running each action.
///   5. Reflect on every task's output; retry up to `max_tries` times before skipping.
///   6. Write a session walkthrough document on completion.
///   7. When the `mta` feature is active: run a metacognition evaluation after each completed task.
#[cfg(feature = "cli")]
#[derive(Debug, Default, Clone, Auto)]
pub struct GenericAgent {
    pub agent: AgentGPT,
    pub client: ClientType,
    /// Whether to skip the approval gate (mirrors the `--yolo` flag).
    pub yolo: bool,
    /// Workspace root directory for generated files.
    pub workspace: String,
    /// Human-readable model name (for session metadata).
    pub model: String,
    /// Provider name (gemini / openai / etc.) for session metadata.
    pub provider: String,
    /// Whether web search (DuckDuckGo) is enabled (mirrors `--no-internet` flag inversion).
    pub internet_access: bool,
    /// Whether to emit verbose reasoning and reflection logs to the Activity Log.
    pub verbose: bool,
    /// Whether the metacognition engine is active for this session.
    pub metacognition_enabled: bool,
    /// Optional sender channel for streaming events to the TUI render thread.
    pub event_tx: Option<UnboundedSender<TuiEvent>>,
    /// Token shared with the TUI to interrupt agent execution on 'Esc'.
    pub abort_token: Option<Arc<AtomicBool>>,
    /// Channel for reading user input forwarded from the TUI command bar.
    pub input_rx: Option<Arc<Mutex<Receiver<String>>>>,
}

#[cfg(feature = "cli")]
#[async_trait]
impl Executor for GenericAgent {
    /// Runs the full AutoGPT pipeline for a single user prompt.
    ///
    /// `task.description` is the user's prompt.
    /// `execute`  - when `false` the plan-approval gate is skipped (yolo mode).
    /// `_browse`  - reserved for future web-search integration.
    /// `max_tries` - maximum retry attempts per sub-task on reflection failure.
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        _browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        let prompt = task.description.to_string();
        if prompt.is_empty() {
            return Ok(());
        }

        let max_retries = max_tries.clamp(1, 5) as u8;
        let session_mgr = SessionManager::default();
        session_mgr.ensure_dirs()?;

        let workspace = if self.workspace.is_empty() {
            env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".autogpt")
                    .join("workspace")
                    .to_string_lossy()
                    .to_string()
            })
        } else {
            self.workspace.clone()
        };
        fs::create_dir_all(&workspace)?;
        let workspace_path = PathBuf::from(&workspace);

        let skills_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("skills");

        let skills = SkillStore::load_for_domain(&prompt, skills_dir.clone())
            .unwrap_or_else(|_| SkillStore::new(skills_dir.clone()));
        let skills_context = skills.to_prompt_context();

        let model = if self.model.is_empty() {
            "gemini-3.0-flash".to_string()
        } else {
            self.model.clone()
        };
        let provider = if self.provider.is_empty() {
            "gemini".to_string()
        } else {
            self.provider.clone()
        };

        let mut session = Session::new(&prompt, &workspace, &model, &provider);
        session.add_message("user", &prompt);

        task.description = format!("[AutoGPT] {prompt}").into();

        print_section("🔬 Synthesizing Task List");
        self.emit_event(TuiEvent::AgentMode("Synthesizing".to_string()));
        self.emit_event(TuiEvent::Log(
            "🔬 Synthesizing task list from prompt...".to_string(),
        ));
        let task_spinner = create_spinner(
            "Decomposing your request into actionable tasks...",
            self.event_tx.is_some(),
        );

        let workspace_snapshot = self.scan_workspace(&workspace_path).await;

        let tasks = match self
            .synthesize_tasks(&prompt, "", &skills_context, &workspace_snapshot)
            .await
        {
            Ok(t) if !t.is_empty() => t,
            Ok(_) => {
                task_spinner.finish_and_clear();
                print_error("LLM returned an empty task list.");
                self.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                return Ok(());
            }
            Err(e) => {
                task_spinner.finish_and_clear();
                self.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                return Err(e);
            }
        };
        task_spinner.finish_and_clear();

        session.set_tasks(tasks.clone());
        session.add_message(
            "assistant",
            &tasks
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{}. {}", i + 1, t.description))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        session_mgr.save(&session)?;

        print_section("📋 Task Plan");
        let total_synth = tasks.len();
        for (i, t) in tasks.iter().enumerate() {
            print_task_item(&t.description, TuiTaskStatus::Pending);
            self.emit_event(TuiEvent::TaskUpdate {
                index: i,
                total: total_synth,
                description: t.description.clone(),
                status: SessionTaskStatus::Pending,
            });
        }

        print_section("🏗️  Generating Implementation Plan");
        self.emit_event(TuiEvent::AgentMode("Planning".to_string()));
        self.emit_event(TuiEvent::Log(
            "🏗️  Generating implementation plan...".to_string(),
        ));
        let plan_spinner = create_spinner(
            "Architecting a production-grade solution...",
            self.event_tx.is_some(),
        );

        let plan = match self.generate_plan(&prompt, &tasks).await {
            Ok(p) => p,
            Err(e) => {
                plan_spinner.finish_and_clear();
                print_error(&format!("Plan generation failed: {e}"));
                return Err(e);
            }
        };
        plan_spinner.finish_and_clear();

        session.set_plan(&plan);
        session.add_message("assistant", &plan);

        if self.event_tx.is_none() {
            print_section("📑 Implementation Plan");
            render_markdown(&plan);
        } else {
            self.emit_event(TuiEvent::Log("◆ Implementation Plan".to_string()));
            self.emit_event(TuiEvent::Log(plan.clone()));
        }
        session_mgr.save(&session)?;

        if execute && !self.yolo {
            let approval = if let Some(rx_lock) = &self.input_rx {
                self.emit_event(TuiEvent::Log(
                    "❓ Approve this plan and begin execution? Type  yes  or  no  in the input bar below.".to_string(),
                ));
                self.emit_event(TuiEvent::AgentMode("Awaiting approval".to_string()));
                let mut rx = rx_lock.lock().await;
                rx.recv().await.unwrap_or_default()
            } else if self.event_tx.is_some() {
                self.emit_event(TuiEvent::Log(
                    "⚠ Auto-approving plan (TUI mode, no input channel attached).".to_string(),
                ));
                "yes".to_string()
            } else {
                info!(
                    "{}  Approve this plan and begin execution? {} ",
                    "?".bright_cyan().bold(),
                    "(yes / no)".bright_black()
                );
                print!("> ");
                io::stdout().flush()?;
                let mut line = String::new();
                io::stdin().lock().read_line(&mut line)?;
                line
            };

            if !is_yes(approval.trim()) {
                self.emit_event(TuiEvent::Log(
                    "⛔ Plan not approved. Ready for next prompt.".to_string(),
                ));
                self.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                session.add_message("user", "Plan rejected.");
                session_mgr.save(&session)?;
                return Ok(());
            }
        }
        if self.event_tx.is_none() {
            print_section("⚙️  Executing Tasks via AutoGPT");
        }
        self.emit_event(TuiEvent::AgentMode("Executing".to_string()));

        let tasks_snapshot = session.tasks.clone();
        let total = tasks_snapshot.len();

        self.emit_event(TuiEvent::NewSession);
        for (idx, task) in tasks_snapshot.iter().enumerate() {
            self.emit_event(TuiEvent::TaskUpdate {
                index: idx,
                total,
                description: task.description.clone(),
                status: task.status,
            });
        }

        let mut project_summary = String::new();

        let mut consecutive_failures = 0;
        'task_loop: for (idx, task_item) in tasks_snapshot.iter().enumerate() {
            let abort_clone = self
                .abort_token
                .clone()
                .unwrap_or_else(|| Arc::new(AtomicBool::new(false)));
            if abort_clone.load(Ordering::SeqCst) {
                self.emit_event(TuiEvent::Log("⛔ Execution aborted by user.".to_string()));
                self.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                break 'task_loop;
            }
            if self.event_tx.is_none() {
                print_task_item(&task_item.description, TuiTaskStatus::InProgress);
            }
            session.update_task_status(idx, SessionTaskStatus::InProgress);
            session_mgr.save(&session)?;
            self.emit_event(TuiEvent::TaskUpdate {
                index: idx,
                total,
                description: task_item.description.clone(),
                status: SessionTaskStatus::InProgress,
            });
            self.emit_event(TuiEvent::Log(format!(
                "⚙ [{}/{}] {}",
                idx + 1,
                total,
                task_item.description
            )));

            let exec_spinner = create_spinner(
                &format!(
                    "Task {}/{}: {}",
                    idx + 1,
                    total,
                    &task_item.description[..task_item.description.len().min(55)]
                ),
                self.event_tx.is_some(),
            );

            let completed_descs: Vec<String> = session
                .tasks
                .iter()
                .take(idx)
                .map(|t| t.description.clone())
                .collect();
            let completed_refs: Vec<&str> = completed_descs.iter().map(|s| s.as_str()).collect();

            if idx > 0 && idx % 3 == 0 {
                project_summary = self.summarize_state(&prompt, &completed_refs).await;
            } else if project_summary.is_empty() {
                project_summary = completed_refs
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}. {}", i + 1, t))
                    .collect::<Vec<_>>()
                    .join("\n");
            }

            let reasoning = self
                .reason_about_task(
                    task_item,
                    &plan,
                    &project_summary,
                    idx + 1,
                    total,
                    &workspace,
                )
                .await;
            session.add_reasoning(&reasoning.thought);

            if !reasoning.thought.trim().is_empty() {
                self.emit_event(TuiEvent::Thinking(
                    reasoning.thought.chars().take(300).collect::<String>(),
                ));
                if self.verbose {
                    self.emit_event(TuiEvent::Log(format!(
                        "> {}",
                        reasoning.thought.chars().take(300).collect::<String>()
                    )));
                }
                info!(
                    "  \x1b[2m> {}\x1b[0m",
                    reasoning.thought.chars().take(300).collect::<String>()
                );
            }
            if !reasoning.approach.trim().is_empty() {
                if self.verbose {
                    self.emit_event(TuiEvent::Log(format!(
                        "> Approach: {}",
                        reasoning.approach.chars().take(200).collect::<String>()
                    )));
                }
                info!(
                    "  \x1b[2m> Approach: {}\x1b[0m",
                    reasoning.approach.chars().take(200).collect::<String>()
                );
            }

            let execute_result = self
                .execute_task(
                    &prompt,
                    task_item,
                    idx + 1,
                    total,
                    &plan,
                    &project_summary,
                    &workspace_path,
                    &mut session,
                    &reasoning,
                    &exec_spinner,
                )
                .await;

            let mut results = match execute_result {
                Ok(r) => r,
                Err(e) => {
                    exec_spinner.finish_and_clear();
                    let err_str = e.to_string();
                    if err_str.contains("User aborted execution") {
                        exec_spinner.finish_and_clear();
                        return Err(e);
                    }
                    if err_str.contains("402")
                        || err_str.contains("Payment")
                        || err_str.contains("quota")
                        || err_str.contains("credits")
                        || err_str.contains("Unauthorized")
                        || err_str.contains("Missing")
                        || err_str.contains("401")
                    {
                        print_error(&format!("LLM provider error: {err_str}"));
                        print_error(
                            "Aborting task execution, resolve the provider issue and retry.",
                        );
                        session.update_task_status(idx, SessionTaskStatus::Failed);
                        self.emit_event(TuiEvent::TaskUpdate {
                            index: idx,
                            total,
                            description: task_item.description.clone(),
                            status: SessionTaskStatus::Failed,
                        });
                        let _ = session_mgr.save(&session);
                        return Err(e);
                    }
                    print_warning(&format!("Task execution error (continuing): {err_str}"));
                    vec![]
                }
            };

            exec_spinner.finish_and_clear();

            let files_before = session.files_created.len();
            let any_success = results.iter().any(|r| r.success);

            if !any_success && session.files_created.len() == files_before {
                for r in &results {
                    if !r.stdout.trim().is_empty() {
                        warn!(
                            "  \x1b[90m[{}] stdout: {}\x1b[0m",
                            r.action_type,
                            r.stdout.trim().chars().take(300).collect::<String>()
                        );
                    }
                    if !r.stderr.trim().is_empty() {
                        warn!(
                            "  \x1b[90m[{}] stderr: {}\x1b[0m",
                            r.action_type,
                            r.stderr.trim().chars().take(300).collect::<String>()
                        );
                    }
                }

                let mut backoff_success = false;
                for attempt in 0..MAX_CONSECUTIVE_FAILURES {
                    let delay_secs = [1, 5, 10].get(attempt as usize).copied().unwrap_or(10);
                    let sleep_duration = Duration::from_secs(delay_secs);
                    print_warning(&format!(
                        "LLM returned empty response for task {}/{} (attempt {}/{}).\
                         Retrying in {}s. Check rate limits or token quota if this persists.",
                        idx + 1,
                        total,
                        attempt + 1,
                        MAX_CONSECUTIVE_FAILURES,
                        delay_secs,
                    ));

                    let mut aborted = false;

                    if self.event_tx.is_some() {
                        sleep(sleep_duration).await;
                    } else {
                        let mut sleep_interval = interval(Duration::from_millis(150));
                        let sleep_start = std::time::Instant::now();
                        let _ = enable_raw_mode();
                        while sleep_start.elapsed() < sleep_duration {
                            tokio::select! {
                                _ = sleep_interval.tick() => {
                                    while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                                        if let Ok(Event::Key(key)) = event::read()
                                            && (key.code == KeyCode::Esc || (key.modifiers.contains(event::KeyModifiers::CONTROL) && key.code == KeyCode::Char('c'))) {
                                                aborted = true;
                                                break;
                                        }
                                    }
                                }
                            }
                            if aborted {
                                break;
                            }
                        }
                        let _ = disable_raw_mode();
                    }
                    if aborted {
                        return Err(anyhow!("User aborted execution."));
                    }

                    let retry_spinner = create_spinner(
                        &format!("Retrying task {}/{}", idx + 1, total),
                        self.event_tx.is_some(),
                    );
                    let retry_result = self
                        .execute_task(
                            &prompt,
                            task_item,
                            idx + 1,
                            total,
                            &plan,
                            &project_summary,
                            &workspace_path,
                            &mut session,
                            &reasoning,
                            &retry_spinner,
                        )
                        .await;

                    match retry_result {
                        Ok(retry_results) if retry_results.iter().any(|r| r.success) => {
                            results = retry_results;
                            backoff_success = true;
                            break;
                        }
                        Err(e) if e.to_string().contains("User aborted execution") => {
                            return Err(e);
                        }
                        _ => {
                            consecutive_failures += 1;
                        }
                    }
                }

                if !backoff_success {
                    if consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                        if self.event_tx.is_none() {
                            print_error(&format!(
                                "Aborting: {} consecutive tasks produced no output.",
                                MAX_CONSECUTIVE_FAILURES
                            ));
                        }
                        for fi in idx..total {
                            session.update_task_status(fi, SessionTaskStatus::Failed);
                            self.emit_event(TuiEvent::TaskUpdate {
                                index: fi,
                                total,
                                description: session.tasks[fi].description.clone(),
                                status: SessionTaskStatus::Failed,
                            });
                        }
                        let _ = session_mgr.save(&session);
                        break 'task_loop;
                    }

                    if self.event_tx.is_none() {
                        print_task_item(&task_item.description, TuiTaskStatus::Skipped);
                    }
                    session.update_task_status(idx, SessionTaskStatus::Failed);
                    self.emit_event(TuiEvent::TaskUpdate {
                        index: idx,
                        total,
                        description: task_item.description.clone(),
                        status: SessionTaskStatus::Failed,
                    });
                    let _ = session_mgr.save(&session);
                    continue 'task_loop;
                }
            }

            consecutive_failures = 0;

            let mut retry_count: u8 = 0;
            loop {
                let reflection = self
                    .reflect_on_task(task_item, &results, retry_count)
                    .await
                    .unwrap_or(ReflectionResult {
                        outcome: ReflectionOutcome::Success,
                        reasoning: "Assumed success.".into(),
                        corrective_actions: vec![],
                    });

                match reflection.outcome {
                    ReflectionOutcome::Success => {
                        if self.event_tx.is_none() {
                            print_task_item(&task_item.description, TuiTaskStatus::Completed);
                        }
                        session.update_task_status(idx, SessionTaskStatus::Completed);
                        self.emit_event(TuiEvent::TaskUpdate {
                            index: idx,
                            total,
                            description: task_item.description.clone(),
                            status: SessionTaskStatus::Completed,
                        });
                        self.emit_event(TuiEvent::Log(format!(
                            "✓ Completed: {}",
                            task_item.description
                        )));
                        break;
                    }
                    ReflectionOutcome::Retry if retry_count < max_retries => {
                        retry_count += 1;
                        if self.event_tx.is_none() {
                            print_warning(&format!(
                                "Retry {retry_count}/{max_retries} - {}",
                                reflection.reasoning
                            ));
                        }
                        results = Vec::new();
                        for action in &reflection.corrective_actions {
                            let r = self
                                .run_action(action, &workspace_path, &mut session)
                                .await
                                .unwrap_or_else(|e| ActionResult {
                                    action_type: "Error".into(),
                                    path: None,
                                    stdout: String::new(),
                                    stderr: e.to_string(),
                                    success: false,
                                });
                            results.push(r);
                        }
                    }
                    ReflectionOutcome::Retry | ReflectionOutcome::Skip => {
                        if self.event_tx.is_none() {
                            print_task_item(&task_item.description, TuiTaskStatus::Skipped);
                            warn!("  ⊘  Skipped: {}", reflection.reasoning.bright_black());
                        }
                        session.update_task_status(idx, SessionTaskStatus::Skipped);
                        self.emit_event(TuiEvent::TaskUpdate {
                            index: idx,
                            total,
                            description: task_item.description.clone(),
                            status: SessionTaskStatus::Skipped,
                        });
                        break;
                    }
                }
            }

            #[cfg(feature = "mta")]
            {
                let current_task_status = &session.tasks[idx].status;
                let outcome_str = match current_task_status {
                    SessionTaskStatus::Completed => "success",
                    SessionTaskStatus::Failed => "failed",
                    SessionTaskStatus::Skipped => "skip",
                    _ => "unknown",
                };

                let entry = self
                    .agent
                    .record_task_outcome(&task_item.description, outcome_str, 0);

                self.emit_event(TuiEvent::Thinking(format!(
                    "🧠 [Metacognition] {}",
                    entry.insight
                )));
                if self.verbose {
                    self.emit_event(TuiEvent::Log(format!("> 🧠 {}", entry.insight)));
                }

                if self.metacognition_enabled && self.agent.should_adjust_strategy() {
                    self.emit_event(TuiEvent::AgentMode("MetaCognizing".to_string()));
                    if let Ok(adjustment) = self.run_metacognition(&prompt, idx, total).await
                        && !adjustment.is_empty()
                        && adjustment != "none"
                    {
                        self.emit_event(TuiEvent::Thinking(format!(
                            "🧠 [Strategy] {}",
                            adjustment
                        )));
                        self.emit_event(TuiEvent::Log(format!("◆ Metacognition: {}", adjustment)));
                        if self.verbose {
                            self.emit_event(TuiEvent::Log(format!(
                                "> Strategy adjustment: {}",
                                adjustment
                            )));
                        }
                    }
                    self.emit_event(TuiEvent::AgentMode("Executing".to_string()));
                }
            }

            let _ = session_mgr.save(&session);
        }

        let build_succeeded = self
            .build_and_verify(&workspace_path, &mut session, 3)
            .await;
        if !build_succeeded && self.event_tx.is_none() {
            print_warning("Build verification failed after all attempts.");
        }

        if self.event_tx.is_none() {
            print_section("📓 Generating Session Walkthrough");
        }
        let wt_spinner =
            create_spinner("Composing walkthrough document...", self.event_tx.is_some());

        let tasks_status = session
            .tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {} [{}]", i + 1, t.description, t.status.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        let files_str = session
            .files_created
            .iter()
            .map(|f| format!("{} ({})", f.path, f.action))
            .collect::<Vec<_>>()
            .join("\n");

        let wt_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            WALKTHROUGH_PROMPT
                .replace("{SESSION_ID}", &session.id)
                .replace(
                    "{DATE}",
                    &session.created_at.format("%Y-%m-%d %H:%M UTC").to_string()
                )
                .replace("{MODEL}", &model)
                .replace("{WORKSPACE}", &workspace)
                .replace("{PROMPT}", &prompt)
                .replace("{TASK_LIST_WITH_STATUSES}", &tasks_status)
                .replace("{FILES_CREATED}", &files_str)
        );

        let walkthrough = match self.generate_tracked(&wt_prompt).await {
            Ok(w) if !w.trim().is_empty() => w,
            _ => SessionManager::generate_walkthrough(&session),
        };

        wt_spinner.finish_and_clear();
        session.set_walkthrough(&walkthrough);
        let _ = session_mgr.save(&session);

        print_section("📓 Session Walkthrough");
        if let Some(_tx) = &self.event_tx {
            // let _ = tx.send(TuiEvent::Log(format!(
            //     "=== Walkthrough ===\n{}\n===================",
            //     walkthrough
            // )));
        } else {
            render_markdown(&walkthrough);
        }

        let wt_path = session_mgr.base_dir.join("walkthrough.md");
        fs::write(&wt_path, &walkthrough)?;

        if let Some((domain, lessons, anti_patterns)) = self
            .extract_lessons(&prompt, &session.tasks, &tasks_status)
            .await
            && (!lessons.is_empty() || !anti_patterns.is_empty())
        {
            let mut store = SkillStore::new(skills_dir);
            for lesson in &lessons {
                let _ = store.save_lesson(&domain, lesson, None);
            }
            for ap in &anti_patterns {
                let _ = store.save_lesson(&domain, "", Some(ap));
            }
        }

        print_section("✅ Session Complete");
        print_success(&format!("Walkthrough → {}", wt_path.display()));
        print_success(&format!(
            "Session     → {}",
            session_mgr.session_dir(&session.id).display()
        ));
        print_success(&format!("Workspace   → {workspace}"));
        print_agent_msg(
            "AutoGPT",
            "All tasks complete. Ready for your next request.",
        );
        self.emit_event(TuiEvent::AgentMode("Idle".to_string()));
        self.emit_event(TuiEvent::Log(
            "✅ All tasks complete. Ready for next request.".to_string(),
        ));

        Ok(())
    }
}

#[cfg(feature = "cli")]
impl GenericAgent {
    async fn generate_safe(&mut self, prompt: &str) -> anyhow::Result<String> {
        let mut interval = interval(Duration::from_millis(150));
        let abort_clone = self
            .abort_token
            .clone()
            .unwrap_or_else(|| std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));
        let is_tui_mode = self.event_tx.is_some();
        let llm_future = self.generate(prompt);
        tokio::pin!(llm_future);

        let timeout_future = sleep(Duration::from_secs(30));
        tokio::pin!(timeout_future);

        let _ = (!is_tui_mode).then(enable_raw_mode);
        let res = loop {
            tokio::select! {
                res = &mut llm_future => {
                    break res.map_err(|e| anyhow!("LLM Generation failed: {e}"));
                }
                _ = &mut timeout_future => {
                    error!("LLM API request timed out after 30 seconds.");
                    break Err(anyhow!("LLM request timed out"));
                }
                _ = interval.tick() => {
                    let mut aborted = false;

                    if abort_clone.load(Ordering::SeqCst) {
                        aborted = true;
                    } else if !is_tui_mode {
                        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
                            if let Ok(Event::Key(key)) = event::read() {
                                let is_abort = key.code == KeyCode::Esc
                                    || (key.modifiers.contains(event::KeyModifiers::CONTROL)
                                        && key.code == KeyCode::Char('c'));
                                if is_abort {
                                    aborted = true;
                                    break;
                                }
                            }
                        }
                    }
                    if aborted {
                        break Err(anyhow!("User aborted execution."));
                    }
                }
            }
        };
        let _ = (!is_tui_mode).then(disable_raw_mode);
        res
    }

    /// Sends an event to the TUI render thread when connected, otherwise logs via tracing.
    pub fn emit_event(&self, event: TuiEvent) {
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Queries the LLM for a metacognitive strategy assessment based on recent task outcomes.
    ///
    /// Builds the `METACOGNITION_PROMPT` with the accumulated task history from
    /// `self.agent.metacognition_context()`, invokes `generate_safe`, parses the JSON response,
    /// and returns the `adjustment` field. Returns `"none"` when no adjustment is needed
    /// or when JSON parsing fails gracefully.
    #[cfg(feature = "mta")]
    async fn run_metacognition(
        &mut self,
        original_request: &str,
        idx: usize,
        total: usize,
    ) -> anyhow::Result<String> {
        let history = self.agent.metacognition_context();
        let consecutive = self.agent.consecutive_failures();
        let remaining = total.saturating_sub(idx + 1);

        let prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            METACOGNITION_PROMPT
                .replace("{ORIGINAL_REQUEST}", original_request)
                .replace("{TASK_HISTORY}", &history)
                .replace("{CONSECUTIVE_FAILURES}", &consecutive.to_string())
                .replace("{TASKS_REMAINING}", &remaining.to_string())
        );

        let raw = self.generate_tracked(&prompt).await.unwrap_or_default();
        let clean = strip_code_blocks(&raw);

        let parsed: MetacognitionResponse = serde_json::from_str(clean.trim()).unwrap_or_default();

        if parsed.adjustment.trim().is_empty() || parsed.adjustment == "none" {
            return Ok(String::new());
        }

        Ok(parsed.adjustment)
    }

    /// Calls the LLM and records request/response token estimates on `session.stats`.
    ///
    /// When connected to the TUI, streams chunks over the event channel and falls back
    /// to `generate_safe` when streaming is unavailable. Emits `IncRequest`,
    /// `IncTokens` (sent), `IncTokens` (recv), and `IncResponse` events.
    async fn generate_and_track(
        &mut self,
        prompt: &str,
        session: &mut Session,
    ) -> anyhow::Result<String> {
        session.record_request(prompt.len());
        self.emit_event(TuiEvent::IncRequest);
        self.emit_event(TuiEvent::IncTokens {
            sent: (prompt.len() / 4).max(1) as u64,
            recv: 0,
        });

        let mut full_response = String::new();

        if let Some(event_tx) = self.event_tx.as_ref() {
            let tx = event_tx.clone();
            match self.stream(prompt).await {
                Ok(ReqResponse(Some(mut rx))) => {
                    let mut first_chunk = true;
                    while let Some(chunk) = rx.recv().await {
                        let chunk_str: String = chunk;
                        if !chunk_str.trim().is_empty() || !first_chunk {
                            if first_chunk {
                                tx.send(TuiEvent::Log(format!("\u{1f916} {}", chunk_str)))
                                    .ok();
                                first_chunk = false;
                            } else {
                                tx.send(TuiEvent::LogAppend(chunk_str.clone())).ok();
                            }
                        }
                        full_response.push_str(&chunk_str);
                    }
                    if full_response.is_empty() {
                        tx.send(TuiEvent::Log(
                            "\u{1f916} (streaming returned empty, retrying...)".to_string(),
                        ))
                        .ok();
                        match self.generate_safe(prompt).await {
                            Ok(resp) if !resp.is_empty() => {
                                full_response = resp.clone();
                                tx.send(TuiEvent::Log(format!("\u{1f916} {}", resp))).ok();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    let resp = self.generate_safe(prompt).await?;
                    full_response = resp;
                    tx.send(TuiEvent::Log(format!("\u{1f916} {}", full_response)))
                        .ok();
                }
            }
        } else {
            full_response = self.generate_safe(prompt).await?;
        }

        session.record_response(full_response.len());
        self.emit_event(TuiEvent::IncTokens {
            sent: 0,
            recv: (full_response.len() / 4).max(1) as u64,
        });
        self.emit_event(TuiEvent::IncResponse);

        Ok(full_response)
    }

    /// Wraps `generate_safe` with request/token/response event emission for pipeline
    /// phases that do not stream (plan generation, reasoning, reflection, etc.).
    ///
    /// Emits `IncRequest`, `IncTokens` (sent), `IncTokens` (recv), and `IncResponse`
    /// so that every LLM call is reflected in the stats panel.
    async fn generate_tracked(&mut self, prompt: &str) -> anyhow::Result<String> {
        self.emit_event(TuiEvent::IncRequest);
        self.emit_event(TuiEvent::IncTokens {
            sent: (prompt.len() / 4).max(1) as u64,
            recv: 0,
        });
        let result = self.generate_safe(prompt).await?;
        self.emit_event(TuiEvent::IncTokens {
            sent: 0,
            recv: (result.len() / 4).max(1) as u64,
        });
        self.emit_event(TuiEvent::IncResponse);
        Ok(result)
    }

    async fn synthesize_tasks(
        &mut self,
        prompt: &str,
        history: &str,
        skills_context: &str,
        workspace_snapshot: &str,
    ) -> Result<Vec<SessionTask>> {
        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            TASK_SYNTHESIS_PROMPT
                .replace("{PROMPT}", prompt)
                .replace("{HISTORY}", history)
                .replace("{WORKSPACE_SNAPSHOT}", workspace_snapshot)
                .replace("{SKILLS_CONTEXT}", skills_context)
        );

        self.emit_event(TuiEvent::AgentMode("Synthesizing".to_string()));
        let raw: String = self.generate_tracked(&full_prompt).await?;

        let numbered: Vec<SessionTask> = raw
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty()
                    && t.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            })
            .map(|l| {
                let desc = l
                    .trim()
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim()
                    .to_string();
                SessionTask {
                    description: desc,
                    status: SessionTaskStatus::Pending,
                }
            })
            .filter(|t| Self::is_valid_task_desc(&t.description))
            .collect();

        if !numbered.is_empty() {
            return Ok(numbered);
        }

        let clean = strip_code_blocks(&raw);
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(clean.trim()) {
            let from_json: Vec<SessionTask> = arr
                .into_iter()
                .filter(|s| Self::is_valid_task_desc(s))
                .map(|s| SessionTask {
                    description: s,
                    status: SessionTaskStatus::Pending,
                })
                .collect();
            if !from_json.is_empty() {
                return Ok(from_json);
            }
        }

        Err(anyhow!(
            "LLM returned malformed task list.",
            // &raw[..raw.len().min(200)]
        ))
    }

    fn is_valid_task_desc(desc: &str) -> bool {
        let words: Vec<&str> = desc.split_whitespace().collect();
        words.len() >= 3 && desc.len() >= 15
    }

    async fn generate_plan(&mut self, prompt: &str, tasks: &[SessionTask]) -> Result<String> {
        self.emit_event(TuiEvent::AgentMode("Planning".to_string()));
        let task_list = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t.description))
            .collect::<Vec<_>>()
            .join("\n");

        let title: String = prompt.chars().take(60).collect();

        let full_prompt = IMPLEMENTATION_PLAN_PROMPT
            .replace("{TITLE}", &title)
            .replace("{PROMPT}", prompt)
            .replace("{TASK_LIST}", &task_list);

        let first_attempt = self.generate_tracked(&full_prompt).await;

        let plan = match first_attempt {
            Ok(p) if !p.trim().is_empty() => p,
            _ => {
                self.emit_event(TuiEvent::Log(
                    "⚠ Plan generation returned empty, retrying...".to_string(),
                ));
                match self.generate_tracked(&full_prompt).await {
                    Ok(p) if !p.trim().is_empty() => p,
                    _ => {
                        let fallback: String = tasks
                            .iter()
                            .enumerate()
                            .map(|(i, t)| {
                                format!(
                                    "## Task {}: {}\n- Implement as described in the prompt.\n- Verify correctness and integrate with existing code.\n",
                                    i + 1,
                                    t.description
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n");
                        format!("# Plan: {title}\n\n{fallback}")
                    }
                }
            }
        };

        Ok(plan)
    }

    async fn reason_about_task(
        &mut self,
        task: &SessionTask,
        plan: &str,
        completed_tasks: &str,
        task_num: usize,
        task_total: usize,
        workspace: &str,
    ) -> ReasoningResult {
        self.emit_event(TuiEvent::AgentMode("Reasoning".to_string()));
        let plan_lines: Vec<&str> = plan.lines().collect();
        let search_key = &task.description[..task.description.len().min(40)];
        let plan_excerpt: String = plan_lines
            .iter()
            .skip_while(|l| !l.to_lowercase().contains(&search_key.to_lowercase()))
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let effective_excerpt = if plan_excerpt.is_empty() {
            plan.lines().take(15).collect::<Vec<_>>().join("\n")
        } else {
            plan_excerpt
        };

        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            REASONING_PROMPT
                .replace("{TASK_NUM}", &task_num.to_string())
                .replace("{TASK_TOTAL}", &task_total.to_string())
                .replace("{TASK_DESCRIPTION}", &task.description)
                .replace("{PLAN_EXCERPT}", &effective_excerpt)
                .replace("{COMPLETED_TASKS}", completed_tasks)
                .replace("{WORKSPACE}", workspace)
        );

        let raw = self
            .generate_tracked(&full_prompt)
            .await
            .unwrap_or_default();
        let clean = strip_code_blocks(&raw);

        let parsed = serde_json::from_str::<ReasoningResult>(clean.trim());
        match parsed {
            Ok(r) if !r.thought.trim().is_empty() => r,
            _ => {
                let thought = if raw.trim().is_empty() {
                    format!(
                        "Executing task {task_num}/{task_total}: {}.",
                        task.description
                    )
                } else {
                    raw.chars().take(400).collect()
                };
                ReasoningResult {
                    thought,
                    approach: format!("Emit action directives for: {}", task.description),
                    risks: vec![],
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_task(
        &mut self,
        prompt: &str,
        task: &SessionTask,
        task_num: usize,
        task_total: usize,
        plan: &str,
        completed_tasks: &str,
        workspace: &Path,
        session: &mut Session,
        reasoning: &ReasoningResult,
        spinner: &indicatif::ProgressBar,
    ) -> Result<Vec<ActionResult>> {
        let plan_lines: Vec<&str> = plan.lines().collect();
        let plan_excerpt: String = plan_lines
            .iter()
            .skip_while(|l| !l.contains(&task.description[..task.description.len().min(30)]))
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        self.agent.truncate_memory(20_000);
        let reasoning_text = if reasoning.thought.is_empty() {
            String::new()
        } else {
            format!(
                "Thought: {}\nApproach: {}\nRisks: {}",
                reasoning.thought,
                reasoning.approach,
                reasoning.risks.join(", ")
            )
        };

        let execution_prompt = TASK_EXECUTION_PROMPT
            .replace("{WORKSPACE}", &workspace.to_string_lossy())
            .replace("{PROMPT}", prompt)
            .replace("{TASK_NUM}", &task_num.to_string())
            .replace("{TASK_TOTAL}", &task_total.to_string())
            .replace("{TASK_DESCRIPTION}", &task.description)
            .replace(
                "{PLAN_EXCERPT}",
                if plan_excerpt.is_empty() {
                    plan
                } else {
                    &plan_excerpt
                },
            )
            .replace("{COMPLETED_TASKS}", completed_tasks)
            .replace("{REASONING}", &reasoning_text);

        let mcp_tools_context = {
            #[cfg(all(feature = "cli", feature = "mcp"))]
            {
                let settings = SettingsManager::new().load().unwrap_or_default();
                let server_names: Vec<String> =
                    settings.mcp.keys().map(|k| format!("{}:*", k)).collect();
                if server_names.is_empty() {
                    "none".to_string()
                } else {
                    server_names.join(", ")
                }
            }
            #[cfg(not(all(feature = "cli", feature = "mcp")))]
            {
                "none".to_string()
            }
        };

        let combined = format!(
            "{}\n\nYou are operating inside workspace: {}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            workspace.display(),
            execution_prompt.replace("{MCP_TOOLS}", &mcp_tools_context)
        );

        self.emit_event(TuiEvent::AgentMode("Executing".to_string()));
        let raw = self.generate_and_track(&combined, session).await?;
        let clean = strip_code_blocks(&raw);
        let actions: Vec<ActionRequest> = serde_json::from_str(clean.trim()).unwrap_or_default();

        spinner.finish_and_clear();

        let mut results = Vec::new();
        for action in &actions {
            let result = self
                .run_action(action, workspace, session)
                .await
                .unwrap_or_else(|e| ActionResult {
                    action_type: "Error".into(),
                    path: None,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    success: false,
                });
            let success = result.success;
            results.push(result);
            if !success {
                break;
            }
        }

        Ok(results)
    }

    /// Dispatches a single `ActionRequest` to the appropriate operation.
    pub async fn run_action(
        &mut self,
        action: &ActionRequest,
        workspace: &Path,
        session: &mut Session,
    ) -> Result<ActionResult> {
        let yolo = self.yolo;
        let internet_access = self.internet_access;
        match action {
            ActionRequest::CreateDir { path } => {
                let abs = workspace.join(path);
                Ok(match fs::create_dir_all(&abs) {
                    Ok(_) => {
                        if self.event_tx.is_some() {
                            self.emit_event(TuiEvent::Log(format!("  📁 {}", path.bright_blue())));
                        } else {
                            info!("  {} {}", "📁".bright_cyan(), path.bright_blue());
                        }
                        ActionResult {
                            action_type: "CreateDir".into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "CreateDir".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::CreateFile { path, content }
            | ActionRequest::WriteFile { path, content } => {
                let abs = workspace.join(path);
                let action_type = match action {
                    ActionRequest::CreateFile { .. } => "CreateFile",
                    _ => "WriteFile",
                };
                if let Some(parent) = abs.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                Ok(match fs::write(&abs, content) {
                    Ok(_) => {
                        if self.event_tx.is_some() {
                            self.emit_event(TuiEvent::Log(format!("  📄 {}", path.bright_blue())));
                        } else {
                            info!("  {} {}", "📄".bright_cyan(), path.bright_blue());
                        }
                        session.record_file(path, action_type);
                        ActionResult {
                            action_type: action_type.into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: action_type.into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::ReadFile { path } => {
                let abs = workspace.join(path);
                Ok(match fs::read_to_string(&abs) {
                    Ok(content) => {
                        if self.event_tx.is_some() {
                            self.emit_event(TuiEvent::Log(format!("  📖 {}", path.bright_blue())));
                        } else {
                            info!("  {} {}", "📖".bright_cyan(), path.bright_blue());
                        }
                        ActionResult {
                            action_type: "ReadFile".into(),
                            path: Some(path.clone()),
                            stdout: content,
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "ReadFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::PatchFile {
                path,
                old_text,
                new_text,
            } => {
                let abs = workspace.join(path);
                Ok(match fs::read_to_string(&abs) {
                    Ok(content) => {
                        if !content.contains(old_text.as_str()) {
                            return Ok(ActionResult {
                                action_type: "PatchFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: format!(
                                    "patch anchor not found in {path}. \
                                     Use ReadFile first to confirm the exact text."
                                ),
                                success: false,
                            });
                        }
                        let patched = content.replacen(old_text.as_str(), new_text.as_str(), 1);
                        match fs::write(&abs, &patched) {
                            Ok(_) => {
                                if self.event_tx.is_some() {
                                    self.emit_event(TuiEvent::Log(format!(
                                        "  ✏️  {}",
                                        path.bright_blue()
                                    )));
                                } else {
                                    info!("  {} {}", "✏️ ".bright_cyan(), path.bright_blue());
                                }
                                session.record_file(path, "PatchFile");
                                ActionResult {
                                    action_type: "PatchFile".into(),
                                    path: Some(path.clone()),
                                    stdout: format!("Patched {path} successfully."),
                                    stderr: String::new(),
                                    success: true,
                                }
                            }
                            Err(e) => ActionResult {
                                action_type: "PatchFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            },
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "PatchFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::AppendFile { path, content } => {
                let abs = workspace.join(path);
                if let Some(parent) = abs.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                Ok(
                    match fs::OpenOptions::new().create(true).append(true).open(&abs) {
                        Ok(mut file) => match file.write_all(content.as_bytes()) {
                            Ok(_) => {
                                if self.event_tx.is_some() {
                                    self.emit_event(TuiEvent::Log(format!(
                                        "  ➕ {}",
                                        path.bright_blue()
                                    )));
                                } else {
                                    info!("  {} {}", "➕".bright_cyan(), path.bright_blue());
                                }
                                session.record_file(path, "AppendFile");
                                ActionResult {
                                    action_type: "AppendFile".into(),
                                    path: Some(path.clone()),
                                    stdout: String::new(),
                                    stderr: String::new(),
                                    success: true,
                                }
                            }
                            Err(e) => ActionResult {
                                action_type: "AppendFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            },
                        },
                        Err(e) => ActionResult {
                            action_type: "AppendFile".into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        },
                    },
                )
            }

            ActionRequest::ListDir { path } => {
                let abs = workspace.to_path_buf();
                Ok(match fs::read_dir(&abs) {
                    Ok(entries) => {
                        let mut lines = Vec::new();
                        for entry in entries.flatten() {
                            let entry: std::fs::DirEntry = entry;
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_dir = entry
                                .file_type()
                                .map(|t: std::fs::FileType| t.is_dir())
                                .unwrap_or(false);
                            lines.push(if is_dir { format!("{name}/") } else { name });
                        }
                        lines.sort();
                        ActionResult {
                            action_type: "ListDir".into(),
                            path: Some(path.clone()),
                            stdout: lines.join("\n"),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "ListDir".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::FindInFile { path, pattern } => {
                let abs = workspace.join(path);
                Ok(match fs::read_to_string(&abs) {
                    Ok(content) => {
                        let matches: Vec<String> = content
                            .lines()
                            .enumerate()
                            .filter(|(_, line)| line.contains(pattern.as_str()))
                            .map(|(i, line)| format!("{}:{}", i + 1, line))
                            .collect();
                        ActionResult {
                            action_type: "FindInFile".into(),
                            path: Some(path.clone()),
                            stdout: matches.join("\n"),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "FindInFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::RunCommand { cmd, args, cwd } => {
                let working_dir = cwd
                    .as_ref()
                    .map(|c| workspace.join(c))
                    .unwrap_or_else(|| workspace.to_path_buf());

                if self.event_tx.is_some() {
                    self.emit_event(TuiEvent::Log(format!(
                        "  ⚡ {} {}",
                        cmd.bright_cyan().bold(),
                        args.join(" ").bright_white()
                    )));
                } else {
                    info!(
                        "  {} {} {}",
                        "⚡".bright_magenta(),
                        cmd.bright_cyan().bold(),
                        args.join(" ").bright_white()
                    );
                }

                if !yolo {
                    let approval = if let Some(rx_lock) = &self.input_rx {
                        self.emit_event(TuiEvent::Log(format!(
                            "❓ Run this command? {} Type yes or no",
                            cmd.bright_cyan().bold()
                        )));
                        self.emit_event(TuiEvent::AgentMode("Awaiting approval".to_string()));
                        let mut rx = rx_lock.lock().await;
                        let val = rx.recv().await.unwrap_or_default();
                        self.emit_event(TuiEvent::AgentMode("Executing".to_string()));
                        val
                    } else {
                        info!(
                            "  {} Run this command? {} ",
                            "?".bright_cyan().bold(),
                            "(yes / no)".bright_black()
                        );
                        print!("> ");
                        io::stdout().flush()?;
                        let mut approval = String::new();
                        let _ = std::io::stdin().lock().read_line(&mut approval)?;
                        approval
                    };

                    if !is_yes(approval.trim()) {
                        self.emit_event(TuiEvent::AgentMode("Executing".to_string()));
                        return Ok(ActionResult {
                            action_type: "RunCommand".into(),
                            path: None,
                            stdout: String::new(),
                            stderr: "Command skipped by user.".into(),
                            success: false,
                        });
                    }
                    self.emit_event(TuiEvent::AgentMode("Executing".to_string()));
                }

                let mut command = Command::new(cmd);
                command.args(args).current_dir(&working_dir);

                let venv_dir = workspace.join(".venv");
                if venv_dir.exists() {
                    let venv_bin = venv_dir.join("bin");
                    if let Some(path) = std::env::var_os("PATH") {
                        let mut new_path = venv_bin.into_os_string();
                        new_path.push(":");
                        new_path.push(path);
                        command.env("PATH", new_path);
                    } else {
                        command.env("PATH", venv_bin);
                    }
                    command.env("VIRTUAL_ENV", venv_dir.as_os_str());
                }

                Ok(
                    match command
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                    {
                        Ok(mut child) => {
                            let stdout = child.stdout.take().expect("Failed to grab stdout");
                            let stderr = child.stderr.take().expect("Failed to grab stderr");

                            let stdout_handle = tokio::spawn(async move {
                                let mut reader = BufReader::new(stdout).lines();
                                let mut out = String::new();
                                while let Ok(Some(line)) = reader.next_line().await {
                                    out.push_str(&line);
                                    out.push('\n');
                                    info!("    {}", line.bright_black());
                                }
                                out
                            });

                            let stderr_handle = tokio::spawn(async move {
                                let mut reader = BufReader::new(stderr).lines();
                                let mut err = String::new();
                                while let Ok(Some(line)) = reader.next_line().await {
                                    err.push_str(&line);
                                    err.push('\n');
                                    error!("    {}", line.bright_red());
                                }
                                err
                            });

                            match child.wait().await {
                                Ok(status) => {
                                    let stdout = stdout_handle.await.unwrap_or_default();
                                    let stderr = stderr_handle.await.unwrap_or_default();
                                    let success = status.success();

                                    ActionResult {
                                        action_type: "RunCommand".into(),
                                        path: None,
                                        stdout,
                                        stderr,
                                        success,
                                    }
                                }
                                Err(e) => {
                                    error!("  {} Command failed: {}", "✗".bright_red(), e);
                                    ActionResult {
                                        action_type: "RunCommand".into(),
                                        path: None,
                                        stdout: String::new(),
                                        stderr: e.to_string(),
                                        success: false,
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("  {} Command failed: {}", "✗".bright_red(), e);
                            ActionResult {
                                action_type: "RunCommand".into(),
                                path: None,
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            }
                        }
                    },
                )
            }

            ActionRequest::GitCommit { message } => {
                let _ = Command::new("git")
                    .args(["add", "-A"])
                    .current_dir(workspace)
                    .output()
                    .await;

                Ok(
                    match Command::new("git")
                        .args(["commit", "-m", message])
                        .current_dir(workspace)
                        .output()
                        .await
                    {
                        Ok(out) => {
                            let success = out.status.success();
                            if success {
                                info!("  {} git: {}", "🔖".bright_cyan(), message.bright_black());
                            }
                            ActionResult {
                                action_type: "GitCommit".into(),
                                path: None,
                                stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                                stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                                success,
                            }
                        }
                        Err(e) => ActionResult {
                            action_type: "GitCommit".into(),
                            path: None,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        },
                    },
                )
            }

            ActionRequest::GlobFiles { pattern } => {
                let mut matched: Vec<String> = Vec::new();
                Self::walk_glob(workspace, workspace, pattern, &mut matched);
                matched.sort();
                Ok(ActionResult {
                    action_type: "GlobFiles".into(),
                    path: None,
                    stdout: matched.join("\n"),
                    stderr: String::new(),
                    success: true,
                })
            }

            ActionRequest::MultiPatch { path, patches } => {
                let abs = workspace.join(path);
                Ok(match fs::read_to_string(&abs) {
                    Ok(original) => {
                        let mut content = original;
                        let mut applied = 0usize;
                        let mut errors: Vec<String> = Vec::new();
                        for (old_text, new_text) in patches {
                            if content.contains(old_text.as_str()) {
                                content = content.replacen(old_text.as_str(), new_text.as_str(), 1);
                                applied += 1;
                            } else {
                                errors.push(format!(
                                    "anchor not found: {:?}",
                                    &old_text[..old_text.len().min(60)]
                                ));
                            }
                        }
                        if let Err(e) = fs::write(&abs, &content) {
                            return Ok(ActionResult {
                                action_type: "MultiPatch".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            });
                        }
                        let success = errors.is_empty();
                        if success {
                            info!(
                                "  {} {} ({} patches)",
                                "✏️ ".bright_cyan(),
                                path.bright_blue(),
                                applied
                            );
                            session.record_file(path, "MultiPatch");
                        }
                        ActionResult {
                            action_type: "MultiPatch".into(),
                            path: Some(path.clone()),
                            stdout: format!("{applied} patches applied."),
                            stderr: errors.join("\n"),
                            success,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "MultiPatch".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                })
            }

            ActionRequest::WebSearch { query } => {
                if !internet_access {
                    info!(
                        "  {} Web search disabled (--no-internet). Skipping: {}",
                        "🌐".bright_black(),
                        query.bright_black()
                    );
                    return Ok(ActionResult {
                        action_type: "WebSearch".into(),
                        path: None,
                        stdout: String::new(),
                        stderr: "Web search disabled via --no-internet flag.".into(),
                        success: false,
                    });
                }

                info!(
                    "  {} Searching: {}",
                    "🌐".bright_blue(),
                    query.bright_cyan()
                );

                let browser = duckduckgo::browser::Browser::new();
                let user_agent = duckduckgo::user_agents::get("firefox").unwrap_or("Mozilla/5.0");
                Ok(
                    match browser
                        .lite_search(query, "wt-wt", Some(5), user_agent)
                        .await
                    {
                        Ok(results) if !results.is_empty() => {
                            let formatted = results
                                .iter()
                                .enumerate()
                                .map(|(i, r)| {
                                    format!(
                                        "{}. {}\n   {}\n   {}",
                                        i + 1,
                                        r.title,
                                        r.snippet.trim(),
                                        r.url
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n\n");
                            info!(
                                "  {} {} results for {:?}",
                                "✓".bright_green(),
                                results.len(),
                                query
                            );
                            ActionResult {
                                action_type: "WebSearch".into(),
                                path: None,
                                stdout: formatted,
                                stderr: String::new(),
                                success: true,
                            }
                        }
                        Ok(_) => ActionResult {
                            action_type: "WebSearch".into(),
                            path: None,
                            stdout: String::new(),
                            stderr: format!("No results found for query: {query}"),
                            success: false,
                        },
                        Err(e) => {
                            warn!("  {} DuckDuckGo search failed: {}", "⚠".bright_yellow(), e);
                            ActionResult {
                                action_type: "WebSearch".into(),
                                path: None,
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            }
                        }
                    },
                )
            }

            ActionRequest::McpCall { server, tool, args } => {
                info!(
                    "  {} {}::{} {:?}",
                    "🔌".bright_magenta(),
                    server.bright_cyan(),
                    tool.bright_white(),
                    args
                );
                #[cfg(all(feature = "cli", feature = "mcp"))]
                {
                    let server_name = server.clone();
                    let tool_name = tool.clone();
                    let call_args: std::collections::HashMap<String, serde_json::Value> = args
                        .as_object()
                        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                        .unwrap_or_default();

                    let result = tokio::task::spawn_blocking(move || {
                        let settings = SettingsManager::new().load()?;
                        let config = settings.mcp.get(&server_name).cloned().ok_or_else(|| {
                            anyhow::anyhow!("MCP server '{}' not configured", server_name)
                        })?;
                        let mut client = McpClient::new(&server_name);
                        client.connect(&config)?;
                        client.call_tool(&tool_name, call_args)
                    })
                    .await;

                    Ok(match result {
                        Ok(Ok(tool_result)) => {
                            info!("  {} MCP call succeeded", "✓".bright_green());
                            ActionResult {
                                action_type: "McpCall".into(),
                                path: None,
                                stdout: tool_result.content,
                                stderr: tool_result.error.unwrap_or_default(),
                                success: tool_result.success,
                            }
                        }
                        Ok(Err(e)) => {
                            warn!("  {} MCP call failed: {}", "⚠".bright_yellow(), e);
                            ActionResult {
                                action_type: "McpCall".into(),
                                path: None,
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            }
                        }
                        Err(join_err) => {
                            warn!("  {} MCP task panicked: {}", "⚠".bright_yellow(), join_err);
                            ActionResult {
                                action_type: "McpCall".into(),
                                path: None,
                                stdout: String::new(),
                                stderr: join_err.to_string(),
                                success: false,
                            }
                        }
                    })
                }
                #[cfg(not(all(feature = "cli", feature = "mcp")))]
                {
                    Ok(ActionResult {
                        action_type: "McpCall".into(),
                        path: None,
                        stdout: String::new(),
                        stderr: "MCP feature not enabled.".into(),
                        success: false,
                    })
                }
            }
        }
    }

    fn walk_glob(workspace: &Path, dir: &Path, pattern: &str, results: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::walk_glob(workspace, &path, pattern, results);
            } else if let Ok(rel) = path.strip_prefix(workspace) {
                let rel_str = rel.to_string_lossy();
                if pattern_matches(pattern, &rel_str) {
                    results.push(rel_str.to_string());
                }
            }
        }
    }

    fn detect_build_system(workspace: &Path) -> Option<ActionRequest> {
        if workspace.join("Cargo.toml").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "cargo".into(),
                args: vec!["build".into()],
                cwd: None,
            });
        }
        if workspace.join("package.json").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "npm".into(),
                args: vec!["run".into(), "build".into()],
                cwd: None,
            });
        }
        if workspace.join("pyproject.toml").exists()
            || workspace.join("setup.py").exists()
            || workspace.join("requirements.txt").exists()
        {
            return Some(ActionRequest::RunCommand {
                cmd: "python3".into(),
                args: vec!["-m".into(), "compileall".into(), "-q".into(), ".".into()],
                cwd: None,
            });
        }
        if workspace.join("go.mod").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "go".into(),
                args: vec!["build".into(), "./...".into()],
                cwd: None,
            });
        }
        if workspace.join("Makefile").exists() || workspace.join("makefile").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "make".into(),
                args: vec![],
                cwd: None,
            });
        }
        None
    }

    async fn build_and_verify(
        &mut self,
        workspace: &Path,
        session: &mut Session,
        max_attempts: u8,
    ) -> bool {
        let build_action = match Self::detect_build_system(workspace) {
            Some(a) => a,
            None => return true,
        };

        print_section("🔨 Verifying Build");

        for attempt in 0..max_attempts {
            session.increment_build_attempt();
            let result = self
                .run_action(&build_action, workspace, session)
                .await
                .unwrap_or_else(|e| ActionResult {
                    action_type: "BuildError".into(),
                    path: None,
                    stdout: String::new(),
                    stderr: e.to_string(),
                    success: false,
                });

            if result.success {
                print_success(&format!("Build passed on attempt {}", attempt + 1));
                return true;
            }

            if attempt + 1 >= max_attempts {
                break;
            }

            print_warning(&format!(
                "Build attempt {} failed. Asking LLM for fix...",
                attempt + 1
            ));

            let error_context = format!(
                "Build failed:\nstdout: {}\nstderr: {}",
                &result.stdout[..result.stdout.len().min(800)],
                &result.stderr[..result.stderr.len().min(800)]
            );

            let fix_prompt = format!(
                "{}\n\nYou are operating in workspace: {}\n\n\
                The project build just failed. Analyze the error output and emit a JSON array \
                of action directives that fix the compilation/syntax errors.\n\n\
                Error context:\n{error_context}\n\n\
                Output only a valid JSON array starting with `[` and ending with `]`.",
                GENERIC_SYSTEM_PROMPT,
                workspace.display()
            );

            let raw = self.generate_safe(&fix_prompt).await.unwrap_or_default();
            let clean = strip_code_blocks(&raw);
            let fix_actions: Vec<ActionRequest> =
                serde_json::from_str(clean.trim()).unwrap_or_default();

            for action in &fix_actions {
                let _ = self.run_action(action, workspace, session).await;
            }
        }

        false
    }

    async fn reflect_on_task(
        &mut self,
        task: &SessionTask,
        results: &[ActionResult],
        retry_attempt: u8,
    ) -> Result<ReflectionResult> {
        let actions_str = results
            .iter()
            .map(|r| format!("- {} {:?}", r.action_type, r.path))
            .collect::<Vec<_>>()
            .join("\n");

        let outputs_str = results
            .iter()
            .map(|r| {
                format!(
                    "[{}] success={}\nstdout: {}\nstderr: {}",
                    r.action_type,
                    r.success,
                    r.stdout.chars().take(600).collect::<String>(),
                    r.stderr.chars().take(400).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let full_prompt = REFLECTION_PROMPT
            .replace("{TASK_DESCRIPTION}", &task.description)
            .replace("{ACTIONS_EXECUTED}", &actions_str)
            .replace("{COMMAND_OUTPUTS}", &outputs_str)
            .replace("{RETRY_ATTEMPT}", &retry_attempt.to_string());

        let raw = self
            .generate_tracked(&full_prompt)
            .await
            .unwrap_or_default();
        let clean = strip_code_blocks(&raw);

        match serde_json::from_str::<ReflectionResult>(clean.trim()) {
            Ok(result) => Ok(result),
            Err(_) => Ok(ReflectionResult {
                outcome: ReflectionOutcome::Success,
                reasoning: "Could not parse reflection - assuming success.".into(),
                corrective_actions: vec![],
            }),
        }
    }

    async fn extract_lessons(
        &mut self,
        prompt: &str,
        tasks: &[SessionTask],
        results_summary: &str,
    ) -> Option<(String, Vec<String>, Vec<String>)> {
        let tasks_str = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {} [{}]", i + 1, t.description, t.status.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        let full_prompt = LESSON_EXTRACTION_PROMPT
            .replace("{ORIGINAL_PROMPT}", prompt)
            .replace("{TASKS}", &tasks_str)
            .replace("{RESULTS}", results_summary);

        let raw = self.generate_tracked(&full_prompt).await.ok()?;
        let clean = strip_code_blocks(&raw);

        serde_json::from_str::<LessonOutput>(clean.trim())
            .ok()
            .map(|l| (l.domain, l.lessons, l.anti_patterns))
    }

    /// Scans the workspace up to two directory levels deep and reads common config files.
    ///
    /// Returns a compact string suitable for injection into synthesis prompts so the LLM
    /// knows what already exists before emitting task directives. The total output is
    /// capped at roughly 2 000 characters to limit token overhead.
    async fn scan_workspace(&self, workspace: &Path) -> String {
        let mut lines: Vec<String> = Vec::new();
        Self::list_tree(workspace, workspace, 0, 2, &mut lines);

        let config_names = [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "requirements.txt",
            "go.mod",
            "Makefile",
            "README.md",
        ];
        let mut config_snippets: Vec<String> = Vec::new();
        for name in &config_names {
            let path = workspace.join(name);
            if let Ok(content) = fs::read_to_string(&path) {
                let preview: String = content.lines().take(12).collect::<Vec<_>>().join("\n");
                if !preview.trim().is_empty() {
                    config_snippets.push(format!("### {name}\n```\n{preview}\n```"));
                }
            }
        }

        let tree = lines.join("\n");
        let configs = config_snippets.join("\n\n");
        let combined = if configs.is_empty() {
            tree
        } else {
            format!("{tree}\n\n{configs}")
        };

        if combined.is_empty() {
            String::new()
        } else {
            combined.chars().take(2000).collect()
        }
    }

    fn list_tree(
        workspace: &Path,
        dir: &Path,
        depth: usize,
        max_depth: usize,
        out: &mut Vec<String>,
    ) {
        if depth > max_depth {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let mut items: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                !p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            })
            .collect();
        items.sort();
        for path in items {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let rel = path
                .strip_prefix(workspace)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| name.to_string());
            let indent = "  ".repeat(depth);
            if path.is_dir() {
                out.push(format!("{indent}{rel}/"));
                Self::list_tree(workspace, &path, depth + 1, max_depth, out);
            } else {
                out.push(format!("{indent}{rel}"));
            }
        }
    }

    /// Synthesises a delta task list for a follow-up request on an existing session.
    ///
    /// Uses `FOLLOWUP_SYNTHESIS_PROMPT` which instructs the LLM to emit only the
    /// _new_ tasks required to satisfy the request, without re-scaffolding anything
    /// already built.
    async fn synthesize_followup_tasks(
        &mut self,
        user_request: &str,
        prior_session: &Session,
        skills_context: &str,
        workspace_snapshot: &str,
    ) -> Result<Vec<SessionTask>> {
        let prior_context = prior_session.session_context_summary();

        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            FOLLOWUP_SYNTHESIS_PROMPT
                .replace("{USER_REQUEST}", user_request)
                .replace("{PRIOR_CONTEXT}", &prior_context)
                .replace("{WORKSPACE_SNAPSHOT}", workspace_snapshot)
                .replace("{SKILLS_CONTEXT}", skills_context)
        );

        let raw: String = self.generate_tracked(&full_prompt).await?;

        let numbered: Vec<SessionTask> = raw
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty()
                    && t.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            })
            .map(|l| {
                let desc = l
                    .trim()
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim()
                    .to_string();
                SessionTask {
                    description: desc,
                    status: SessionTaskStatus::Pending,
                }
            })
            .filter(|t| Self::is_valid_task_desc(&t.description))
            .collect();

        if !numbered.is_empty() {
            return Ok(numbered);
        }

        let clean = strip_code_blocks(&raw);
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(clean.trim()) {
            let from_json: Vec<SessionTask> = arr
                .into_iter()
                .filter(|s| Self::is_valid_task_desc(s))
                .map(|s| SessionTask {
                    description: s,
                    status: SessionTaskStatus::Pending,
                })
                .collect();
            if !from_json.is_empty() {
                return Ok(from_json);
            }
        }

        Err(anyhow::anyhow!(
            "LLM returned an empty follow-up task list. Raw: {}",
            &raw[..raw.len().min(200)]
        ))
    }

    async fn summarize_state(&mut self, prompt: &str, completed: &[&str]) -> String {
        let task_list = completed
            .iter()
            .enumerate()
            .map(|(i, &t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        let summary_prompt = STATE_SUMMARIZATION_PROMPT
            .replace("{PROMPT}", prompt)
            .replace("{COMPLETED_TASKS}", &task_list);

        match self.generate_tracked(&summary_prompt).await {
            Ok(s) if !s.trim().is_empty() => s,
            _ => task_list,
        }
    }
}

/// Lesson data extracted at the end of a session for the skill store.
#[cfg(feature = "cli")]
#[derive(Deserialize)]
struct LessonOutput {
    domain: String,
    #[serde(default)]
    lessons: Vec<String>,
    #[serde(default)]
    anti_patterns: Vec<String>,
}

/// Returns `true` when a file path matches a simple glob pattern.
///
/// Supports `*` (any characters within one path segment) and `**` (any path).
/// Case-sensitive. Used by the `GlobFiles` action handler.
#[cfg(feature = "cli")]
fn pattern_matches(pattern: &str, path: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if !pattern.contains('*') {
        return path.ends_with(pattern) || path == pattern;
    }
    let parts: Vec<&str> = pattern.splitn(2, '*').collect();
    let prefix = parts[0];
    let suffix = if parts.len() > 1 { parts[1] } else { "" };
    path.starts_with(prefix)
        && (suffix.is_empty() || path.ends_with(suffix))
        && path.len() >= prefix.len() + suffix.len()
}

/// Configuration for the generic agent loop.
#[cfg(feature = "cli")]
#[derive(Default)]
pub struct GenericAgentLoopConfig {
    pub yolo: bool,
    pub internet_access: bool,
    pub session_id: Option<String>,
    pub mixture: bool,
    pub custom_workspace: Option<String>,
    pub event_tx: Option<UnboundedSender<TuiEvent>>,
    pub input_rx: Option<Receiver<String>>,
    pub abort_token: Option<Arc<AtomicBool>>,
    #[cfg(feature = "col")]
    pub collab: bool,
}

/// Runs the interactive AutoGPT CLI loop.
///
/// This is a thin REPL shell that:
///   1. Prints the TUI banner, tips, and startup warnings.
///   2. Reads the user's slash commands or prompts in a loop.
///   3. On a real prompt, builds a `Task` and calls `agent.execute()` - which contains
///      the full synthesize → plan → approve → execute → reflect → walkthrough pipeline.
///   4. On follow-up prompts in the same session, passes the prior session context
///      to `synthesize_followup_tasks` so the LLM works on top of existing code.
///
/// Slash commands (`/help`, `/sessions`, `/models`, `/clear`, `/status`, `/workspace`,
/// `/provider`) are handled here and never reach the executor.
#[cfg(feature = "cli")]
pub async fn run_generic_agent_loop(config: GenericAgentLoopConfig) -> anyhow::Result<()> {
    let GenericAgentLoopConfig {
        yolo,
        internet_access,
        session_id,
        mixture: _mixture,
        custom_workspace,
        event_tx,
        input_rx,
        abort_token,
        #[cfg(feature = "col")]
        collab,
    } = config;
    print_banner();
    print_greeting();

    if yolo {
        warn!(
            "{}",
            "⚡  YOLO mode active - all plans will be auto-approved."
                .bright_yellow()
                .bold()
        );
        info!("");
    }

    let session_mgr = SessionManager::default();
    session_mgr.ensure_dirs()?;

    let settings_mgr = SettingsManager::new();
    let mut settings = settings_mgr.load().unwrap_or_default();
    settings.internet_access = internet_access;
    let _ = settings_mgr.save(&settings);

    let mut active_session: Option<Session> = None;

    if let Some(id) = &session_id {
        match session_mgr.load(id) {
            Ok(session) => {
                print_section("📂 Resuming Session");
                info!(
                    "  {} {} - {} tasks, {} messages",
                    "▸".bright_cyan(),
                    session.title.white().bold(),
                    session.tasks.len(),
                    session.messages.len()
                );
                info!("");
                for msg in &session.messages {
                    info!("  {}", format!("[{}]", msg.role).bright_magenta());
                    if let Some(tx) = &event_tx {
                        let _ = tx.send(TuiEvent::Log(format!("[{}]: {}", msg.role, msg.content)));
                    } else {
                        render_markdown(&msg.content);
                    }
                }
                info!("");
            }
            Err(e) => print_warning(&format!("Could not load session {id}: {e}")),
        }
    }

    let workspace = env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
        if custom_workspace.as_deref() == Some(".") {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .to_string_lossy()
                .to_string()
        } else if let Some(path) = custom_workspace {
            path
        } else {
            PathBuf::from(".")
                .join("workspace")
                .to_string_lossy()
                .to_string()
        }
    });
    fs::create_dir_all(&workspace)?;

    let mut current_provider = default_provider();
    let mut current_model = default_model(&current_provider);
    let mut available_models = provider_models(&current_provider);
    let mut current_model_idx = model_index(&available_models, &current_model);

    let shared_input_rx = input_rx.map(|rx| Arc::new(tokio::sync::Mutex::new(rx)));

    let mut agent = GenericAgent {
        yolo,
        internet_access: settings.internet_access,
        workspace: workspace.clone(),
        model: current_model.clone(),
        provider: current_provider.clone(),
        event_tx: event_tx.clone(),
        abort_token: abort_token.clone(),
        input_rx: shared_input_rx.clone(),
        ..Default::default()
    };

    let mut input_history: Vec<String> = Vec::new();

    'outer: loop {
        let settings = SettingsManager::new().load().unwrap_or_default();
        agent.yolo = yolo || settings.yolo;
        agent.internet_access = internet_access && settings.internet_access;
        agent.verbose = settings.verbose;
        agent.metacognition_enabled = settings.metacognition;
        if let Some(ref tok) = agent.abort_token {
            tok.store(false, Ordering::SeqCst);
        }

        let cwd_str = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string());

        let (sent_kb, recv_kb, reqs) = if let Some(sess) = &active_session {
            (
                sess.stats.tokens_sent / 4 / 1024,
                sess.stats.tokens_received / 4 / 1024,
                sess.stats.requests,
            )
        } else {
            (0, 0, 0)
        };
        let internet_badge = if agent.internet_access {
            "\x1b[32m🌐\x1b[0m"
        } else {
            "\x1b[90m🚫\x1b[0m"
        };
        let stats_seg = if reqs > 0 {
            format!("  \x1b[93m↑{sent_kb}KB ↓{recv_kb}KB\x1b[0m \x1b[90m{reqs}req\x1b[0m")
        } else {
            String::new()
        };
        let status_line = format!(
            "  \x1b[94m{}\x1b[0m  \x1b[95m{}\x1b[0m  \x1b[90m{}\x1b[0m{}  {}",
            cwd_str, current_model, current_provider, stats_seg, internet_badge
        );

        #[allow(unused_mut)]
        let mut input = if let Some(ref rx_lock) = shared_input_rx {
            let mut rx = rx_lock.lock().await;
            match rx.recv().await {
                Some(s) if s == "\x1b" => {
                    break;
                }
                Some(s) => s,
                None => break,
            }
        } else {
            match read_line(
                &status_line,
                "Type your request, or /help for commands",
                SLASH_COMMANDS,
                &input_history,
            ) {
                ReadlineResult::Submit(s) => s,
                ReadlineResult::Interrupted => {
                    print_success("Session saved. Goodbye!");
                    break;
                }
                ReadlineResult::Error(e) => {
                    print_error(&format!("Input error: {e}"));
                    continue;
                }
            }
        };

        if input.is_empty() {
            print_warning("Please enter a prompt to work on.");
            continue;
        }

        if !input.starts_with('/')
            && !input.eq_ignore_ascii_case("exit")
            && !input.eq_ignore_ascii_case("quit")
        {
            input_history.push(input.clone());
        }

        #[cfg(feature = "mop")]
        if _mixture
            && !input.starts_with('/')
            && !is_yes(&input)
            && let Some((provider, response)) = run_mixture(&input).await
        {
            print_success(&format!("MoP selected response from: {provider}"));
            render_markdown(&response);
            input = format!("High-quality context from {provider}: {response}\n\nTask: {input}");
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            print_success("Session saved. Goodbye!");
            break;
        }

        if input.starts_with('/')
            && handle_slash_command(
                &input,
                &mut agent,
                &session_mgr,
                &workspace,
                &mut active_session,
            )
            .await?
        {
            continue;
        }

        if input.eq_ignore_ascii_case("/status") {
            let cwd = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(tx) = &event_tx {
                let _ = tx.send(TuiEvent::Log(format!("Directory: {}", cwd)));
                let _ = tx.send(TuiEvent::Log(format!("Model: {}", current_model)));
                let _ = tx.send(TuiEvent::Log(format!("Provider: {}", current_provider)));
                let _ = tx.send(TuiEvent::Log(format!("Workspace: {}", workspace)));
            } else {
                info!("{} {}", "Directory:".bright_cyan(), cwd.bright_white());
                info!(
                    "{} {}",
                    "Model:".bright_cyan(),
                    current_model.bright_magenta()
                );
                info!(
                    "{} {}",
                    "Provider:".bright_cyan(),
                    current_provider.bright_white()
                );
                info!(
                    "{} {}",
                    "Workspace:".bright_cyan(),
                    workspace.bright_white()
                );
            }
            continue;
        }

        if input.eq_ignore_ascii_case("/provider") {
            if let Some(tx) = &event_tx {
                let _ = tx.send(TuiEvent::Log(
                    "Please use the 'Settings' tab (4) to manage providers and models in TUI mode."
                        .to_string(),
                ));
                continue;
            }
            let providers = ["gemini", "openai", "anthropic", "xai", "cohere"];
            info!("");
            info!("{}", "Select Provider:".bright_cyan().bold());
            for (i, p) in providers.iter().enumerate() {
                if *p == current_provider {
                    info!(
                        "  {} {} {}",
                        format!("{}.", i + 1).bright_cyan(),
                        p.bright_white().bold(),
                        "(active)".bright_green()
                    );
                } else {
                    info!(
                        "  {} {}",
                        format!("{}.", i + 1).bright_black(),
                        p.bright_white()
                    );
                }
            }
            info!("");
            print!("> Enter number: ");
            let _ = io::stdout().flush();
            let mut pick = String::new();
            let _ = io::stdin().lock().read_line(&mut pick);
            if let (Ok(n), current_len) = (pick.trim().parse::<usize>(), providers.len())
                && n >= 1
                && n <= current_len
            {
                current_provider = providers[n - 1].to_string();
                agent.provider = current_provider.clone();

                available_models = provider_models(&current_provider);
                current_model = default_model(&current_provider);
                current_model_idx = model_index(&available_models, &current_model);
                agent.model = current_model.clone();

                print_success(&format!("Switched to provider: {current_provider}"));
            }
            continue;
        }

        if input.eq_ignore_ascii_case("/models") {
            if let Some(tx) = &event_tx {
                let _ = tx.send(TuiEvent::Log(
                    "Please use the 'Settings' tab (4) to manage your model in TUI mode."
                        .to_string(),
                ));
                continue;
            }
            if available_models.is_empty() {
                print_warning(
                    "No models available for the current provider. Try setting the appropriate API key.",
                );
                continue;
            }
            let selected = render_model_selector(&available_models, current_model_idx);
            current_model_idx = selected;
            current_model = available_models[selected].id.clone();
            agent.model = current_model.clone();
            print_success(&format!(
                "Model set to: {}",
                available_models[selected].display_name
            ));
            continue;
        }

        if input.starts_with('/')
            && handle_slash_command(
                &input,
                &mut agent,
                &session_mgr,
                &workspace,
                &mut active_session,
            )
            .await?
        {
            continue;
        }

        if let Some(ref prior) = active_session {
            info!(
                "  {} {}",
                "↩".bright_cyan(),
                format!("Follow-up on: {}", prior.title).bright_black()
            );
        }

        agent.agent.behavior = input.clone().into();

        let workspace_path = PathBuf::from(&workspace);
        let skills_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("skills");
        let skills = SkillStore::load_for_domain(&input, skills_dir).unwrap_or_else(|_| {
            SkillStore::new(
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".autogpt")
                    .join("skills"),
            )
        });
        let skills_context = skills.to_prompt_context();

        if active_session.is_none() {
            let ws_snapshot = {
                let wp = PathBuf::from(&workspace);
                agent.scan_workspace(&wp).await
            };

            agent.emit_event(TuiEvent::AgentMode("Classifying".to_string()));

            let intent_prompt = INTENT_DETECTION_PROMPT
                .replace("{USER_PROMPT}", &input)
                .replace("{WORKSPACE}", &ws_snapshot)
                .replace(
                    "{MCP_TOOLS}",
                    "list_dir, read_file, search_web, run_command",
                );

            let intent: AgentIntent = agent
                .generate_safe(&intent_prompt)
                .await
                .ok()
                .and_then(|raw| {
                    let clean = strip_code_blocks(&raw);
                    serde_json::from_str::<IntentResponse>(clean.trim())
                        .ok()
                        .map(|p| match p.intent.as_str() {
                            "direct_answer" => AgentIntent::DirectAnswer,
                            "tool_call" => AgentIntent::ToolCall {
                                tool: p.tool.unwrap_or_else(|| "list_dir".to_string()),
                                args: p.args.unwrap_or(serde_json::Value::Null),
                            },
                            _ => AgentIntent::TaskPlan,
                        })
                })
                .unwrap_or(AgentIntent::TaskPlan);

            match intent {
                AgentIntent::DirectAnswer => {
                    agent.emit_event(TuiEvent::AgentMode("Answering".to_string()));
                    // agent.emit_event(TuiEvent::Log("💬 Direct answer. Responding without task planning.".to_string()));
                    let resp = if let Some(ref mut sess) = active_session {
                        agent
                            .generate_and_track(&input, sess)
                            .await
                            .unwrap_or_else(|_| {
                                "Sorry, I couldn't generate a response.".to_string()
                            })
                    } else {
                        agent.generate_safe(&input).await.unwrap_or_default()
                    };
                    agent.emit_event(TuiEvent::Log(format!(
                        "🤖 {}",
                        resp.chars().take(500).collect::<String>()
                    )));
                    agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                    if agent.event_tx.is_none() {
                        render_markdown(&resp);
                    }
                    continue;
                }
                AgentIntent::ToolCall { tool: _, args: _ } => {
                    agent.emit_event(TuiEvent::AgentMode("Tool".to_string()));
                    // agent.emit_event(TuiEvent::Log(format!("🔧 Tool: {}. Routing to task planning.", tool)));
                    agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                }
                AgentIntent::TaskPlan => { /* continue */ }
            }
        }

        if let Some(ref prior) = active_session {
            let snapshot = agent.scan_workspace(&workspace_path).await;
            let tasks = match agent
                .synthesize_followup_tasks(&input, prior, &skills_context, &snapshot)
                .await
            {
                Ok(t) if !t.is_empty() => t as Vec<SessionTask>,
                Ok(_) => {
                    print_error("LLM returned an empty follow-up task list.");
                    continue;
                }
                Err(e) => {
                    print_error(&format!("Follow-up synthesis failed: {e}"));
                    continue;
                }
            };

            print_section("📋 Follow-up Task Plan");
            for t in &tasks {
                print_task_item(&t.description, TuiTaskStatus::Pending);
            }

            if !agent.yolo {
                let approval = if let Some(ref rx_lock) = shared_input_rx {
                    agent.emit_event(TuiEvent::Log(
                        "❓ Approve follow-up tasks and begin execution? Type  yes  or  no"
                            .to_string(),
                    ));
                    agent.emit_event(TuiEvent::AgentMode("Awaiting approval".to_string()));
                    let mut rx = rx_lock.lock().await;
                    let val = rx.recv().await.unwrap_or_default();
                    agent.emit_event(TuiEvent::AgentMode("Executing".to_string()));
                    val
                } else {
                    info!(
                        "{}  Approve and execute these tasks? {} ",
                        "?".bright_cyan().bold(),
                        "(yes / no)".bright_black()
                    );
                    print!("> ");
                    io::stdout().flush()?;
                    let mut approval = String::new();
                    io::stdin().lock().read_line(&mut approval)?;
                    approval
                };
                if !is_yes(approval.trim()) {
                    print_warning("Tasks not approved.");
                    if agent.event_tx.is_some() {
                        agent.emit_event(TuiEvent::Log(
                            "⛔ Follow-up tasks not approved.".to_string(),
                        ));
                        agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                    }
                    continue;
                }
            }

            let model = if agent.model.is_empty() {
                "gemini-2.5-flash".to_string()
            } else {
                agent.model.clone()
            };
            let provider = if agent.provider.is_empty() {
                "gemini".to_string()
            } else {
                agent.provider.clone()
            };

            let mut new_session = Session::new(&input, &workspace, &model, &provider);
            new_session.add_message("user", &input);
            new_session.set_tasks(tasks.clone());

            let plan = prior.plan.clone().unwrap_or_default();
            print_section("⚙️  Executing Follow-up Tasks");
            let total = tasks.len();
            let tasks_snap = tasks.clone();
            for (idx, task_item) in tasks_snap.iter().enumerate() {
                print_task_item(&task_item.description, TuiTaskStatus::InProgress);
                new_session.update_task_status(idx, SessionTaskStatus::InProgress);
                agent.emit_event(TuiEvent::TaskUpdate {
                    index: idx,
                    total,
                    description: task_item.description.clone(),
                    status: SessionTaskStatus::InProgress,
                });

                let completed_descs: Vec<String> = tasks_snap
                    .iter()
                    .take(idx)
                    .map(|t| t.description.clone())
                    .collect();
                let completed_refs: Vec<&str> =
                    completed_descs.iter().map(|s| s.as_str()).collect();

                let completed_tasks = completed_refs
                    .iter()
                    .enumerate()
                    .map(|(i, t)| format!("{}. {}", i + 1, t))
                    .collect::<Vec<_>>()
                    .join("\n");

                let exec_spinner = create_spinner(
                    &format!(
                        "Task {}/{}: {}",
                        idx + 1,
                        total,
                        &task_item.description[..task_item.description.len().min(55)]
                    ),
                    agent.event_tx.is_some(),
                );

                let reasoning = agent
                    .reason_about_task(
                        task_item,
                        &plan,
                        &completed_tasks,
                        idx + 1,
                        total,
                        &workspace,
                    )
                    .await;
                new_session.add_reasoning(&reasoning.thought);

                if !reasoning.thought.trim().is_empty() {
                    info!(
                        "  \x1b[2m> {}\x1b[0m",
                        reasoning.thought.chars().take(300).collect::<String>()
                    );
                }
                if !reasoning.approach.trim().is_empty() {
                    info!(
                        "  \x1b[2m> Approach: {}\x1b[0m",
                        reasoning.approach.chars().take(200).collect::<String>()
                    );
                }

                let exec_result = agent
                    .execute_task(
                        &input,
                        task_item,
                        idx + 1,
                        total,
                        &plan,
                        &completed_tasks,
                        &workspace_path,
                        &mut new_session,
                        &reasoning,
                        &exec_spinner,
                    )
                    .await;

                let results = match exec_result {
                    Ok(r) => r,
                    Err(e) => {
                        let err_str = e.to_string();
                        if err_str.contains("402")
                            || err_str.contains("Payment")
                            || err_str.contains("quota")
                            || err_str.contains("credits")
                            || err_str.contains("Unauthorized")
                            || err_str.contains("Missing")
                            || err_str.contains("401")
                        {
                            print_error(&format!("LLM provider error: {err_str}"));
                            print_error(
                                "Aborting follow-up tasks, resolve the provider issue and retry.",
                            );
                            new_session.update_task_status(idx, SessionTaskStatus::Failed);
                            agent.emit_event(TuiEvent::TaskUpdate {
                                index: idx,
                                total,
                                description: task_item.description.clone(),
                                status: SessionTaskStatus::Failed,
                            });
                            session_mgr.save(&new_session)?;
                            active_session = Some(new_session);
                            continue 'outer;
                        }
                        print_warning(&format!("Task execution error (continuing): {err_str}"));
                        vec![]
                    }
                };

                let files_before = new_session.files_created.len();
                let any_success = results.iter().any(|r| r.success);
                if !any_success && new_session.files_created.len() == files_before {
                    print_warning(
                        "No concrete actions succeeded and no files were written. \
                         The LLM may have returned empty output.",
                    );
                }

                let reflection = agent
                    .reflect_on_task(task_item, &results, 0)
                    .await
                    .unwrap_or(ReflectionResult {
                        outcome: ReflectionOutcome::Success,
                        reasoning: "Assumed success.".into(),
                        corrective_actions: vec![],
                    });

                match reflection.outcome {
                    ReflectionOutcome::Success => {
                        print_task_item(&task_item.description, TuiTaskStatus::Completed);
                        new_session.update_task_status(idx, SessionTaskStatus::Completed);
                        agent.emit_event(TuiEvent::TaskUpdate {
                            index: idx,
                            total,
                            description: task_item.description.clone(),
                            status: SessionTaskStatus::Completed,
                        });
                    }
                    _ => {
                        print_task_item(&task_item.description, TuiTaskStatus::Skipped);
                        new_session.update_task_status(idx, SessionTaskStatus::Skipped);
                        agent.emit_event(TuiEvent::TaskUpdate {
                            index: idx,
                            total,
                            description: task_item.description.clone(),
                            status: SessionTaskStatus::Skipped,
                        });
                    }
                }

                session_mgr.save(&new_session)?;
            }

            let _ = agent
                .build_and_verify(&workspace_path, &mut new_session, 3)
                .await;

            session_mgr.save(&new_session)?;
            active_session = Some(new_session);
            continue;
        }

        if agent.event_tx.is_some() {
            #[cfg(feature = "col")]
            if collab {
                let mut pool = CollabPool::from_env(
                    &agent.agent.persona,
                    &agent.agent.behavior,
                    &workspace,
                    yolo,
                    agent.verbose,
                    event_tx.clone(),
                    shared_input_rx.clone(),
                );
                if pool.len() > 1 {
                    let start = pool.pick_start(&CollabSelection::Random);
                    let mut task_obj = Task {
                        description: input.clone().into(),
                        scope: Some(Scope {
                            crud: true,
                            auth: false,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };
                    let start_provider_name = pool.provider_name(start).to_string();
                    agent.emit_event(TuiEvent::Log(format!(
                        "🤝 Collab: starting synthesis on provider '{}'",
                        start_provider_name
                    )));

                    let mut start_ok = false;
                    'start_retry: loop {
                        if let Some(start_agent) = pool.agent_mut(start) {
                            match Executor::execute(start_agent, &mut task_obj, !yolo, false, 2)
                                .await
                            {
                                Ok(_) => {
                                    start_ok = true;
                                    break 'start_retry;
                                }
                                Err(e) => {
                                    let err_str = e.to_string();
                                    let is_rate_limit = err_str.contains("402")
                                        || err_str.contains("quota")
                                        || err_str.contains("credits")
                                        || err_str.contains("permission")
                                        || err_str.contains("monthly limit")
                                        || err_str.contains("Unauthorized")
                                        || err_str.contains("does not have permission")
                                        || err_str.contains("401");
                                    agent.emit_event(TuiEvent::Log(format!(
                                        "⚠ Collab start agent error: {e}"
                                    )));
                                    if is_rate_limit && pool.mark_failure(start) {
                                        continue 'start_retry;
                                    }
                                    pool.exhausted.insert(start);
                                    break 'start_retry;
                                }
                            }
                        } else {
                            break 'start_retry;
                        }
                    }

                    if !start_ok {
                        while let Some(next_idx) = pool.next_available() {
                            let next_name = pool.provider_name(next_idx).to_string();
                            agent.emit_event(TuiEvent::Log(format!(
                                "🔀 Collab: falling back to provider '{next_name}'"
                            )));
                            'fallback_retry: loop {
                                if let Some(fb_agent) = pool.agent_mut(next_idx) {
                                    match Executor::execute(
                                        fb_agent,
                                        &mut task_obj,
                                        !yolo,
                                        false,
                                        2,
                                    )
                                    .await
                                    {
                                        Ok(_) => break 'fallback_retry,
                                        Err(e) => {
                                            let err_str = e.to_string();
                                            let is_rate_limit = err_str.contains("402")
                                                || err_str.contains("quota")
                                                || err_str.contains("credits")
                                                || err_str.contains("permission")
                                                || err_str.contains("monthly limit")
                                                || err_str.contains("Unauthorized")
                                                || err_str.contains("does not have permission")
                                                || err_str.contains("401");
                                            agent.emit_event(TuiEvent::Log(format!(
                                                "⚠ Collab [{next_name}] error: {e}"
                                            )));
                                            if is_rate_limit && pool.mark_failure(next_idx) {
                                                continue 'fallback_retry;
                                            }
                                            break 'fallback_retry;
                                        }
                                    }
                                } else {
                                    break 'fallback_retry;
                                }
                            }
                        }
                        if pool.exhausted.len() >= pool.slots.len() {
                            agent.emit_event(TuiEvent::Log(
                                "⚠ All collab providers exhausted.".to_string(),
                            ));
                        }
                    }
                } else {
                    agent.emit_event(TuiEvent::Log(
                        "⚠ Collab: fewer than two providers found. Falling back to single-provider mode.".to_string(),
                    ));
                    let mut task = Task {
                        description: input.clone().into(),
                        scope: Some(Scope {
                            crud: true,
                            auth: false,
                            external: true,
                        }),
                        urls: None,
                        frontend_code: None,
                        backend_code: None,
                        api_schema: None,
                    };
                    let current_yolo = agent.yolo;
                    match Executor::execute(&mut agent, &mut task, !current_yolo, false, 2).await {
                        Ok(_) => {
                            if let Ok(entries) = session_mgr.list()
                                && let Some(entry) = entries.first()
                                && let Ok(s) = session_mgr.load(&entry.id)
                            {
                                active_session = Some(s);
                            }
                        }
                        Err(e) if e.to_string().contains("User aborted") => {
                            agent.emit_event(TuiEvent::Log("⚠ Execution interrupted.".to_string()));
                            agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                            if let Some(ref tok) = abort_token {
                                tok.store(false, Ordering::SeqCst);
                            }
                            continue 'outer;
                        }
                        Err(e) => {
                            agent.emit_event(TuiEvent::Log(format!("✗ Agent error: {e}")));
                            agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                        }
                    }
                }
            } else {
                let mut task = Task {
                    description: input.clone().into(),
                    scope: Some(Scope {
                        crud: true,
                        auth: false,
                        external: true,
                    }),
                    urls: None,
                    frontend_code: None,
                    backend_code: None,
                    api_schema: None,
                };
                let current_yolo = agent.yolo;
                match Executor::execute(&mut agent, &mut task, !current_yolo, false, 2).await {
                    Ok(_) => {
                        if let Ok(entries) = session_mgr.list()
                            && let Some(entry) = entries.first()
                            && let Ok(s) = session_mgr.load(&entry.id)
                        {
                            active_session = Some(s);
                        }
                    }
                    Err(e) if e.to_string().contains("User aborted") => {
                        agent.emit_event(TuiEvent::Log("⚠ Execution interrupted.".to_string()));
                        agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                        if let Some(ref tok) = abort_token {
                            tok.store(false, Ordering::SeqCst);
                        }
                        continue 'outer;
                    }
                    Err(e) => {
                        agent.emit_event(TuiEvent::Log(format!("✗ Agent error: {e}")));
                        agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                    }
                }
            }
            #[cfg(not(feature = "col"))]
            {
                let mut task = Task {
                    description: input.clone().into(),
                    scope: Some(Scope {
                        crud: true,
                        auth: false,
                        external: true,
                    }),
                    urls: None,
                    frontend_code: None,
                    backend_code: None,
                    api_schema: None,
                };
                let current_yolo = agent.yolo;
                match Executor::execute(&mut agent, &mut task, !current_yolo, false, 2).await {
                    Ok(_) => {
                        if let Ok(entries) = session_mgr.list()
                            && let Some(entry) = entries.first()
                            && let Ok(s) = session_mgr.load(&entry.id)
                        {
                            active_session = Some(s);
                        }
                    }
                    Err(e) if e.to_string().contains("User aborted") => {
                        agent.emit_event(TuiEvent::Log("⚠ Execution interrupted.".to_string()));
                        agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                        if let Some(ref tok) = abort_token {
                            tok.store(false, Ordering::SeqCst);
                        }
                        continue 'outer;
                    }
                    Err(e) => {
                        agent.emit_event(TuiEvent::Log(format!("✗ Agent error: {e}")));
                        agent.emit_event(TuiEvent::AgentMode("Idle".to_string()));
                    }
                }
            }
        } else {
            let arc_agent: Arc<Mutex<Box<dyn AgentFunctions>>> =
                Arc::new(Mutex::new(Box::new(agent.clone())));

            let autogpt = AutoGPT::default()
                .execute(!agent.yolo)
                .max_tries(2)
                .with(vec![arc_agent])
                .build()
                .expect("Failed to build AutoGPT");

            let tok_clone = abort_token.clone();
            let tui_mode = event_tx.is_some();
            let mut interrupt_handle = tokio::spawn(async move {
                loop {
                    if let Some(tok) = &tok_clone {
                        if tok.load(Ordering::SeqCst) {
                            return;
                        }
                    } else if !tui_mode
                        && matches!(event::poll(Duration::from_millis(100)), Ok(true))
                        && let Ok(Event::Key(key)) = event::read()
                        && key.code == KeyCode::Esc
                    {
                        return;
                    }
                    sleep(Duration::from_millis(50)).await;
                }
            });

            tokio::select! {
                res = autogpt.run() => {
                    interrupt_handle.abort();
                    match res {
                        Ok(msg) => {
                            info!("{}", msg.bright_green());
                            if let Ok(entries) = session_mgr.list()
                                && let Some(entry) = entries.first()
                                    && let Ok(s) = session_mgr.load(&entry.id) {
                                        active_session = Some(s);
                                }
                        }
                        Err(e) if e.to_string().contains("User aborted execution") => {
                            print_warning("Execution interrupted by user (ESC pressed).");
                            continue 'outer;
                        }
                        Err(e) => {
                            print_error(&format!("Agent error: {e}"));
                        },
                    }
                }
                _ = &mut interrupt_handle => {
                    print_warning("Execution interrupted by user (ESC pressed).");
                    continue 'outer;
                }
            }
        }
    }

    Ok(())
}

/// Asynchronously handles the slash command provided by the user.
///
/// This function parses the input for various commands starting with a forward slash ('/')
/// and executes the corresponding logic, such as listing sessions, managing MCP servers,
/// or displaying help.
pub async fn handle_slash_command(
    input: &str,
    _agent: &mut GenericAgent,
    session_mgr: &SessionManager,
    workspace: &str,
    active_session: &mut Option<Session>,
) -> Result<bool> {
    if input.eq_ignore_ascii_case("/help") {
        if let Some(tx) = &_agent.event_tx {
            render_help_table_to_log(tx);
        } else {
            render_help_table();
        }
        return Ok(true);
    }

    if input.eq_ignore_ascii_case("/clear") {
        if let Some(tx) = &_agent.event_tx {
            let _ = tx.send(TuiEvent::ClearLog);
        } else {
            print!("{}[2J{}[1;1H", 27 as char, 27 as char);
            print_banner();
        }
        return Ok(true);
    }

    if input.eq_ignore_ascii_case("/workspace") {
        if _agent.event_tx.is_none() {
            print_section("📂 Current Workspace");
            info!("  {} {}", "▸".bright_cyan(), workspace.white().bold());
        }
        return Ok(true);
    }

    if input.eq_ignore_ascii_case("/status") {
        print_section("📊 Session Status");
        if let Some(s) = active_session.as_ref() {
            info!("  {}   {}", "Title:".bright_black(), s.title.white().bold());
            info!(
                "  {}      {}",
                "ID:".bright_black(),
                s.id.to_string().bright_black()
            );
            let task_count = s.tasks.len();
            let completed_count = s
                .tasks
                .iter()
                .filter(|t| t.status == SessionTaskStatus::Completed)
                .count();
            info!(
                "  {}  {}",
                "Progress:".bright_black(),
                format!("{} / {} tasks completed", completed_count, task_count).bright_green()
            );
        } else {
            print_warning("No active session. Start a project to see status.");
        }
        return Ok(true);
    }

    if input.eq_ignore_ascii_case("/sessions") {
        match session_mgr.list() {
            Ok(entries) if !entries.is_empty() => {
                if let Some(tx) = &_agent.event_tx {
                    tx.send(TuiEvent::Log("📁 Recent Sessions:".to_string()))
                        .ok();
                    let sessions_for_picker: Vec<(String, String, String)> = entries
                        .iter()
                        .enumerate()
                        .map(|(i, entry)| {
                            let label = format!(
                                "  {}. {} ({}/{}) {}",
                                i + 1,
                                entry.title,
                                entry.completed_count,
                                entry.task_count,
                                entry.updated_at.format("%Y-%m-%d %H:%M")
                            );
                            tx.send(TuiEvent::Log(label)).ok();
                            (
                                entry.id.clone(),
                                entry.title.clone(),
                                format!("{}/{}", entry.completed_count, entry.task_count),
                            )
                        })
                        .collect();
                    tx.send(TuiEvent::SessionsPick(sessions_for_picker)).ok();
                } else {
                    print_section("📁 Recent Sessions");
                    for (i, entry) in entries.iter().enumerate() {
                        info!(
                            "  {} {} {} {}",
                            format!("{}.", i + 1).bright_cyan(),
                            entry.title.white().bold(),
                            format!("({}/{})", entry.completed_count, entry.task_count)
                                .bright_green(),
                            entry
                                .updated_at
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                                .bright_black()
                        );
                        info!("     {} {}", "↳".bright_black(), entry.id.bright_black());
                    }
                }
            }
            Ok(_) => {
                if let Some(tx) = &_agent.event_tx {
                    tx.send(TuiEvent::Log("No previous sessions found.".to_string()))
                        .ok();
                } else {
                    print_warning("No previous sessions found.");
                }
            }
            Err(e) => {
                if let Some(tx) = &_agent.event_tx {
                    tx.send(TuiEvent::Log(format!("Failed to list sessions: {e}")))
                        .ok();
                } else {
                    print_error(&format!("Failed to list sessions: {e}"));
                }
            }
        }
        return Ok(true);
    }

    if input.starts_with("/mcp") {
        #[cfg(all(feature = "cli", feature = "mcp"))]
        {
            let parts: Vec<&str> = input.split(' ').collect();
            match parts.as_slice() {
                ["/mcp"] | ["/mcp", "list"] => {
                    let mgr = SettingsManager::new();
                    match mgr.load() {
                        Ok(settings) => {
                            let infos = tokio::task::spawn_blocking(move || {
                                let mut handles: Vec<std::thread::JoinHandle<McpServerInfo>> =
                                    Vec::new();
                                for (name, config) in settings.mcp {
                                    let mut config = config.clone();
                                    if config.timeout_ms > 60000 {
                                        config.timeout_ms = 60000;
                                    }
                                    let name_clone = name.clone();
                                    handles.push(std::thread::spawn(move || {
                                        let mut client = McpClient::new(name_clone);
                                        let _ = client.connect(&config);
                                        let desc = config.description.clone().unwrap_or_default();
                                        client.to_server_info(&desc)
                                    }));
                                }
                                handles
                                    .into_iter()
                                    .map(|h| h.join().unwrap())
                                    .collect::<Vec<_>>()
                            })
                            .await
                            .unwrap();
                            if let Some(tx) = &_agent.event_tx {
                                render_mcp_list_to_log(tx, &infos);
                            } else {
                                render_mcp_list(&infos);
                            }
                        }
                        Err(e) => {
                            if let Some(tx) = &_agent.event_tx {
                                tx.send(TuiEvent::Log(format!("Failed to load settings: {e}")))
                                    .ok();
                            } else {
                                print_error(&format!("Failed to load settings: {e}"));
                            }
                        }
                    }
                }
                ["/mcp", "inspect", name] => {
                    let mgr = SettingsManager::new();
                    match mgr.load() {
                        Ok(settings) => {
                            if let Some(config) = settings.mcp.get(*name) {
                                let config = config.clone();
                                let name_str = name.to_string();
                                let tx_clone = _agent.event_tx.clone();
                                tokio::task::spawn_blocking(move || {
                                    let mut config = config.clone();
                                    if config.timeout_ms > 60000 {
                                        config.timeout_ms = 60000;
                                    }
                                    let mut client = McpClient::new(name_str);
                                    let _ = client.connect(&config);
                                    let desc = config.description.clone().unwrap_or_default();
                                    let info = client.to_server_info(&desc);
                                    if let Some(tx) = &tx_clone {
                                        render_mcp_inspect_to_log(tx, &info, &config);
                                    } else {
                                        render_mcp_inspect(&info, &config);
                                    }
                                })
                                .await
                                .unwrap();
                            } else {
                                if let Some(tx) = &_agent.event_tx {
                                    tx.send(TuiEvent::Log(format!(
                                        "MCP server '{}' not found.",
                                        name
                                    )))
                                    .ok();
                                } else {
                                    print_warning(&format!(
                                        "MCP server '{}' not found. Run `/mcp list` to see all.",
                                        name
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            if let Some(tx) = &_agent.event_tx {
                                tx.send(TuiEvent::Log(format!("Failed to load settings: {e}")))
                                    .ok();
                            } else {
                                print_error(&format!("Failed to load settings: {e}"));
                            }
                        }
                    }
                }
                ["/mcp", "remove", name] => {
                    let mgr = SettingsManager::new();
                    match mgr.remove_mcp_server(name) {
                        Ok((_, true)) => print_success(&format!("MCP server '{}' removed.", name)),
                        Ok((_, false)) => {
                            print_warning(&format!("MCP server '{}' was not registered.", name))
                        }
                        Err(e) => print_error(&format!("Failed to remove server: {e}")),
                    }
                }
                ["/mcp", "call", server, tool, ..] => {
                    let server_str = server.to_string();
                    let tool_str = tool.to_string();
                    let args = parts[4..].iter().map(|s| s.to_string()).collect();
                    if let Err(e) = tokio::task::spawn_blocking(move || {
                        mcp_cmd::cmd_mcp_call(&server_str, &tool_str, args)
                    })
                    .await
                    .unwrap()
                    {
                        print_error(&format!("Failed to call tool: {e}"));
                    }
                }
                _ => {
                    if let Some(tx) = &_agent.event_tx {
                        render_mcp_help_entries_to_log(tx);
                    } else {
                        render_mcp_help_entries();
                    }
                }
            }
        }
        #[cfg(not(any(feature = "cli", feature = "mcp")))]
        {
            if let Some(tx) = &_agent.event_tx {
                tx.send(TuiEvent::Log(
                    "MCP support requires the `mcp` or `cli` feature.".to_string(),
                ))
                .ok();
            } else {
                print_warning("MCP support requires the `mcp` or `cli` feature.");
            }
        }
        return Ok(true);
    }

    if input.starts_with("/resume ") {
        let session_id = input.trim_start_matches("/resume ").trim();
        if session_id.is_empty() {
            if let Some(tx) = &_agent.event_tx {
                tx.send(TuiEvent::Log("Usage: /resume <session-id>".to_string()))
                    .ok();
            } else {
                print_warning("Usage: /resume <session-id>");
            }
            return Ok(true);
        }
        match session_mgr.load(session_id) {
            Ok(session) => {
                let msg = format!(
                    "📂 Resuming: {} ({} tasks)",
                    session.title,
                    session.tasks.len()
                );
                if let Some(tx) = &_agent.event_tx {
                    tx.send(TuiEvent::Log(msg)).ok();
                    let total = session.tasks.len();
                    for (i, t) in session.tasks.iter().enumerate() {
                        tx.send(TuiEvent::TaskUpdate {
                            index: i,
                            total,
                            description: t.description.clone(),
                            status: t.status,
                        })
                        .ok();
                    }
                    tx.send(TuiEvent::AgentMode("Idle".to_string())).ok();
                } else {
                    print_section(&msg);
                }
                *active_session = Some(session);
            }
            Err(e) => {
                if let Some(tx) = &_agent.event_tx {
                    tx.send(TuiEvent::Log(format!("✗ Could not load session: {e}")))
                        .ok();
                } else {
                    print_error(&format!("Could not load session '{session_id}': {e}"));
                }
            }
        }
        return Ok(true);
    }

    if input.starts_with('/') {
        if let Some(tx) = &_agent.event_tx {
            tx.send(TuiEvent::Log(format!(
                "Unknown command: `{}`. Type /help for available commands.",
                input
            )))
            .ok();
        } else {
            print_warning(&format!(
                "Unknown command: `{}`. Type /help for available commands.",
                input
            ));
        }
        return Ok(true);
    }

    Ok(false)
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
