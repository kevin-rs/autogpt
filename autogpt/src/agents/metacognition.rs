// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use serde::{Deserialize, Serialize};

/// A single metacognitive observation recorded after a task completes.
///
/// Captures the task description, its outcome, retry count, and any strategy
/// adjustments the engine derived for subsequent tasks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetacognitionEntry {
    pub task_description: String,
    pub outcome: String,
    pub retry_count: u8,
    pub insight: String,
    pub strategy_adjustment: Option<String>,
}

/// Accumulates metacognitive entries across all tasks in a session.
///
/// `MetacognitionEngine` is embedded in `GenericAgent` when the `mta` feature
/// is active. It tracks outcome patterns and surfaces strategy adjustments to
/// the agent loop every few tasks or after consecutive failures.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MetacognitionEngine {
    pub entries: Vec<MetacognitionEntry>,
    pub consecutive_failures: u8,
    pub consecutive_successes: u8,
}

impl MetacognitionEngine {
    /// Creates a new empty metacognition engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a task outcome and updates internal failure and success counters.
    ///
    /// Returns a compact insight string summarising the pattern observed so far.
    pub fn record(
        &mut self,
        task_description: &str,
        outcome: &str,
        retry_count: u8,
    ) -> MetacognitionEntry {
        let is_failure = outcome == "skip" || outcome == "failed";
        let is_success = outcome == "success" || outcome == "completed";

        if is_failure {
            self.consecutive_failures += 1;
            self.consecutive_successes = 0;
        } else if is_success {
            self.consecutive_successes += 1;
            self.consecutive_failures = 0;
        }

        let insight = self.derive_insight(retry_count);
        let strategy_adjustment = if self.should_adjust() {
            Some(self.derive_strategy_hint())
        } else {
            None
        };

        let entry = MetacognitionEntry {
            task_description: task_description.to_string(),
            outcome: outcome.to_string(),
            retry_count,
            insight: insight.clone(),
            strategy_adjustment: strategy_adjustment.clone(),
        };

        self.entries.push(entry.clone());
        entry
    }

    /// Returns true when the engine has enough data to suggest a strategy change.
    ///
    /// This triggers an LLM metacognition call in the agent loop.
    pub fn should_adjust(&self) -> bool {
        self.consecutive_failures >= 2
            || self.entries.len().is_multiple_of(3) && !self.entries.is_empty()
    }

    /// Generates a brief human-readable insight based on the current outcome history.
    fn derive_insight(&self, retry_count: u8) -> String {
        if self.consecutive_failures >= 2 {
            format!(
                "{} consecutive failures detected. Strategy recalibration recommended.",
                self.consecutive_failures
            )
        } else if retry_count > 0 {
            format!(
                "Task completed after {} retries. Consider adjusting action granularity.",
                retry_count
            )
        } else if self.consecutive_successes >= 3 {
            "Execution pipeline flowing well. Maintaining current strategy.".to_string()
        } else {
            "Task outcome recorded. Monitoring for patterns.".to_string()
        }
    }

    /// Produces a focused strategy hint based on observed failure patterns.
    fn derive_strategy_hint(&self) -> String {
        let failure_tasks: Vec<&str> = self
            .entries
            .iter()
            .filter(|e| e.outcome == "skip" || e.outcome == "failed")
            .map(|e| e.task_description.as_str())
            .collect();

        if failure_tasks.is_empty() {
            return "Review task decomposition for upcoming steps.".to_string();
        }

        let themes: Vec<String> = failure_tasks
            .iter()
            .take(3)
            .map(|t| format!("- {}", &t[..t.len().min(60)]))
            .collect();

        format!(
            "Recurring pattern in failed tasks:\n{}\nConsider breaking these into smaller actions.",
            themes.join("\n")
        )
    }

    /// Produces a compact context string for the LLM metacognition prompt.
    ///
    /// Includes the last 5 entry summaries and the current strategy hint.
    pub fn to_prompt_context(&self) -> String {
        let recent: Vec<String> = self
            .entries
            .iter()
            .rev()
            .take(5)
            .rev()
            .map(|e| {
                format!(
                    "  task: {} | outcome: {} | retries: {}",
                    &e.task_description[..e.task_description.len().min(60)],
                    e.outcome,
                    e.retry_count
                )
            })
            .collect();

        let hint = self.derive_strategy_hint();

        format!(
            "Recent task history:\n{}\n\nCurrent strategy pattern:\n{}",
            recent.join("\n"),
            hint
        )
    }
}
