// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Collaborative multi-provider agent pool.
//!
//! `CollabPool` coordinates one [`GenericAgent`] per available LLM provider
//! (when compiled with the `cli` feature) and distributes planned task items
//! across them in round-robin order.  When a provider stalls or returns a
//! rate-limit error the pool first cycles through alternative models for that
//! provider before falling back to the next healthy provider.
//!
//! Without the `cli` feature the pool compiles as a lightweight provider
//! registry that records provider names and supports round-robin selection,
//! but has no agent-execution capability.

use std::collections::HashSet;

#[cfg(all(feature = "col", feature = "cli"))]
use crate::agents::generic::GenericAgent;
#[cfg(all(feature = "col", feature = "cli"))]
use crate::cli::models::{default_model, provider_models};
#[cfg(all(feature = "col", feature = "cli"))]
use crate::common::utils::{ClientType, PROVIDER_ENV_MAP, PROVIDER_NAMES, Task};
#[cfg(all(feature = "col", feature = "cli"))]
use crate::tui::state::TuiEvent;
#[cfg(all(feature = "col", feature = "cli"))]
use std::env;
#[cfg(all(feature = "col", feature = "cli"))]
use std::sync::Arc;
#[cfg(all(feature = "col", feature = "cli"))]
use tokio::sync::Mutex;
#[cfg(all(feature = "col", feature = "cli"))]
use tokio::sync::mpsc::{Receiver, UnboundedSender};

/// Determines how the starting provider is selected when a collab session begins.
#[cfg(feature = "col")]
#[derive(Debug, Clone)]
pub enum CollabSelection {
    /// Choose the starting provider at random.
    Random,
    /// Require exactly the named provider to start.
    Explicit(String),
}

/// Runtime state for a single provider slot inside a [`CollabPool`].
///
/// Only available when both `col` and `cli` features are enabled, because it
/// pairs provider metadata with a [`GenericAgent`] which requires `cli`.
#[cfg(all(feature = "col", feature = "cli"))]
#[derive(Debug, Clone)]
pub struct ProviderSlot {
    /// Human-readable provider name (e.g. `"gemini"`).
    pub name: String,
    /// Ordered list of model IDs available for this provider.
    pub models: Vec<String>,
    /// Index into `models` currently in use.
    pub model_cursor: usize,
    /// Number of consecutive failures on the currently active model.
    pub failure_count: u8,
    /// Whether this slot has been exhausted (all models failed).
    pub exhausted: bool,
}

#[cfg(all(feature = "col", feature = "cli"))]
impl ProviderSlot {
    /// Returns the model ID that is currently active for this slot.
    pub fn current_model(&self) -> &str {
        self.models
            .get(self.model_cursor)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    /// Attempts to advance to the next model in the provider's model list.
    ///
    /// Returns `true` when a new model was selected, `false` when all models
    /// have been exhausted and the slot should be considered dead.
    pub fn try_next_model(&mut self) -> bool {
        if self.model_cursor + 1 < self.models.len() {
            self.model_cursor += 1;
            self.failure_count = 0;
            true
        } else {
            self.exhausted = true;
            false
        }
    }
}

/// A pool that coordinates multiple LLM providers for collaborative task execution.
///
/// When compiled with both `col` and `cli` features, the pool spawns one
/// [`GenericAgent`] per discovered provider and distributes tasks in round-robin
/// order with automatic fallback.
///
/// When compiled with `col` only (no `cli`), the pool acts as a lightweight
/// provider registry with round-robin selection but no agent execution.
#[cfg(feature = "col")]
#[derive(Clone, Debug)]
pub struct CollabPool {
    /// Paired provider metadata and agent, only available with `cli`.
    #[cfg(feature = "cli")]
    pub slots: Vec<(ProviderSlot, GenericAgent)>,

    /// Provider names only, available when `cli` is disabled.
    #[cfg(not(feature = "cli"))]
    pub provider_names: Vec<String>,

    /// Round-robin cursor pointing at the next slot to receive a task.
    pub cursor: usize,

    /// Indices of slots that have been permanently exhausted.
    pub exhausted: HashSet<usize>,

    /// Optional TUI event sender for broadcasting pool state, only with `cli`.
    #[cfg(feature = "cli")]
    pub event_tx: Option<UnboundedSender<TuiEvent>>,
}

/// Core API available with just the `col` feature (no `cli` required).
#[cfg(feature = "col")]
impl CollabPool {
    /// Creates a pool from a list of provider name strings.
    ///
    /// Useful in library mode where no `cli` feature and no agent execution
    /// is needed, but provider routing is still desired.
    #[cfg(not(feature = "cli"))]
    pub fn from_providers(provider_names: Vec<String>) -> Self {
        Self {
            provider_names,
            cursor: 0,
            exhausted: HashSet::new(),
        }
    }

    /// Returns the number of providers in the pool.
    pub fn len(&self) -> usize {
        #[cfg(feature = "cli")]
        return self.slots.len();
        #[cfg(not(feature = "cli"))]
        return self.provider_names.len();
    }

    /// Returns `true` when the pool has no providers.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the provider name for slot `idx`, or `""` if out of bounds.
    pub fn provider_name(&self, idx: usize) -> &str {
        #[cfg(feature = "cli")]
        return self
            .slots
            .get(idx)
            .map(|(s, _)| s.name.as_str())
            .unwrap_or("");
        #[cfg(not(feature = "cli"))]
        return self
            .provider_names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or("");
    }

    /// Returns the index of the starting agent given a [`CollabSelection`].
    ///
    /// When `Random` is requested a pseudo-random index is derived from the
    /// current system time modulo the pool size.  When `Explicit` is requested
    /// and the named provider is not found index `0` is returned.
    pub fn pick_start(&self, selection: &CollabSelection) -> usize {
        let pool_size = self.len().max(1);
        match selection {
            CollabSelection::Random => {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .subsec_nanos() as usize;
                ts % pool_size
            }
            CollabSelection::Explicit(name) => {
                for idx in 0..self.len() {
                    if self.provider_name(idx) == name.as_str() {
                        return idx;
                    }
                }
                0
            }
        }
    }
}

/// Extended API available only when both `col` and `cli` features are enabled.
///
/// These methods rely on [`GenericAgent`], [`TuiEvent`], and provider discovery
/// utilities that come from the `cli` module tree.
#[cfg(all(feature = "col", feature = "cli"))]
impl CollabPool {
    /// Discovers all configured LLM providers from the environment and builds
    /// one [`GenericAgent`] per provider.
    ///
    /// # Arguments
    ///
    /// * `persona`   - Agent persona label shared across all pool agents.
    /// * `behavior`  - Agent behavior / mission prompt shared across all pool agents.
    /// * `workspace` - File-system workspace root for generated artefacts.
    /// * `yolo`      - When `true` all plan-approval gates are skipped.
    /// * `verbose`   - Enable verbose activity logging.
    /// * `event_tx`  - Optional channel for forwarding [`TuiEvent`]s to the TUI.
    /// * `input_rx`  - Optional channel for receiving user approval input from the TUI.
    ///
    /// Returns a pool containing one slot per provider with a valid API key in
    /// the environment.  Returns an empty pool when no providers are configured.
    pub fn from_env(
        persona: &str,
        behavior: &str,
        workspace: &str,
        yolo: bool,
        verbose: bool,
        event_tx: Option<UnboundedSender<TuiEvent>>,
        input_rx: Option<Arc<Mutex<Receiver<String>>>>,
    ) -> Self {
        let mut slots = Vec::new();

        for provider in PROVIDER_NAMES {
            let env_key = match PROVIDER_ENV_MAP.get(provider).copied() {
                Some(k) => k,
                None => continue,
            };
            if env::var(env_key).is_err() {
                continue;
            }

            let models: Vec<String> = provider_models(provider)
                .into_iter()
                .map(|m| m.id)
                .collect();
            let model = if models.is_empty() {
                default_model(provider)
            } else {
                models[0].clone()
            };
            let models = if models.is_empty() {
                vec![model.clone()]
            } else {
                models
            };

            unsafe {
                env::set_var("AI_PROVIDER", provider);
            }
            if let Some(model_id) = models.first() {
                let provider_upper = provider.to_uppercase();
                unsafe {
                    env::set_var(format!("{}_MODEL", provider_upper), model_id);
                }
            }

            let client = ClientType::from_env();
            let mut agent = GenericAgent::default();
            agent.agent.persona = persona.to_string().into();
            agent.agent.behavior = behavior.to_string().into();
            agent.yolo = yolo;
            agent.workspace = workspace.to_string();
            agent.model = models.first().cloned().unwrap_or_default();
            agent.provider = provider.to_string();
            agent.verbose = verbose;
            agent.event_tx = event_tx.clone();
            agent.input_rx = input_rx.clone();
            agent.internet_access = true;
            agent.client = client;

            let slot = ProviderSlot {
                name: provider.to_string(),
                models,
                model_cursor: 0,
                failure_count: 0,
                exhausted: false,
            };

            slots.push((slot, agent));
        }

        if let Some(tx) = &event_tx {
            let provider_list: Vec<(String, String)> = slots
                .iter()
                .map(|(s, _)| (s.name.clone(), s.current_model().to_string()))
                .collect();
            let _ = tx.send(TuiEvent::CollabPool(provider_list));
        }

        Self {
            slots,
            cursor: 0,
            exhausted: HashSet::new(),
            event_tx,
        }
    }

    /// Returns the next non-exhausted slot index using round-robin dispatch.
    ///
    /// Returns `None` when every slot in the pool has been exhausted.
    pub fn next_available(&mut self) -> Option<usize> {
        let len = self.slots.len();
        if self.exhausted.len() >= len {
            return None;
        }
        for _ in 0..len {
            let idx = self.cursor % len;
            self.cursor += 1;
            if !self.exhausted.contains(&idx) {
                return Some(idx);
            }
        }
        None
    }

    /// Records a failure for the agent at `idx`.
    ///
    /// First attempts to rotate to the next model within the same provider.
    /// When no further models are available the slot is permanently exhausted.
    ///
    /// Returns `true` when a fallback model is now active, `false` when the
    /// entire provider is exhausted.
    pub fn mark_failure(&mut self, idx: usize) -> bool {
        if let Some((slot, agent)) = self.slots.get_mut(idx) {
            slot.failure_count += 1;
            if slot.try_next_model() {
                let new_model = slot.current_model().to_string();
                agent.model = new_model.clone();
                if let Some(tx) = &self.event_tx {
                    let _ = tx.send(TuiEvent::Log(format!(
                        "\u{1f504} [Collab] {} \u{2192} switching to model {}",
                        slot.name, new_model
                    )));
                }
                true
            } else {
                self.exhausted.insert(idx);
                if let Some(tx) = &self.event_tx {
                    let _ = tx.send(TuiEvent::Log(format!(
                        "\u{26a0} [Collab] {} exhausted all models, removing from pool.",
                        slot.name
                    )));
                }
                false
            }
        } else {
            false
        }
    }

    /// Distributes `tasks` across pool slots in round-robin order, starting from `start_idx`.
    ///
    /// Returns a vector of `(slot_index, task)` pairs ready for sequential execution.
    pub fn distribute(&mut self, tasks: Vec<Task>, start_idx: usize) -> Vec<(usize, Task)> {
        let len = self.slots.len();
        tasks
            .into_iter()
            .enumerate()
            .map(|(i, task)| {
                let slot_idx = (start_idx + i) % len.max(1);
                (slot_idx, task)
            })
            .collect()
    }

    /// Returns a mutable reference to the [`GenericAgent`] at `idx`.
    pub fn agent_mut(&mut self, idx: usize) -> Option<&mut GenericAgent> {
        self.slots.get_mut(idx).map(|(_, agent)| agent)
    }
}
