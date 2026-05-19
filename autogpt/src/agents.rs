// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Agents module.
//!
//! This module contains sub-modules representing different built-in AutoGPT agents.
//!

pub mod agent;
#[cfg(feature = "gpt")]
pub mod architect;
#[cfg(feature = "gpt")]
pub mod backend;
#[cfg(feature = "gpt")]
pub mod designer;
#[cfg(feature = "gpt")]
pub mod frontend;
#[cfg(feature = "cli")]
pub mod generic;
#[cfg(feature = "gpt")]
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "cli")]
pub mod intent;
#[cfg(feature = "gpt")]
#[cfg(feature = "mail")]
pub mod mailer;
#[cfg(feature = "gpt")]
pub mod manager;
#[cfg(feature = "mta")]
pub mod metacognition;
#[cfg(feature = "mop")]
pub mod mop;
#[cfg(feature = "gpt")]
pub mod optimizer;
#[cfg(feature = "gpt")]
pub mod types;
