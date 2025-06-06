//! # Agents module.
//!
//! This module contains sub-modules representing different built-in AutoGPT agents.
//!

pub mod agent;
pub mod architect;
pub mod backend;
#[cfg(feature = "img")]
pub mod designer;
pub mod frontend;
#[cfg(feature = "git")]
pub mod git;
#[cfg(feature = "mail")]
pub mod mailer;
pub mod manager;
pub mod optimizer;
pub mod types;
