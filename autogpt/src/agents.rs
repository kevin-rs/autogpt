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
#[cfg(feature = "gpt")]
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "gpt")]
#[cfg(feature = "mail")]
pub mod mailer;
#[cfg(feature = "gpt")]
pub mod manager;
#[cfg(feature = "gpt")]
pub mod optimizer;
#[cfg(feature = "gpt")]
pub mod types;
