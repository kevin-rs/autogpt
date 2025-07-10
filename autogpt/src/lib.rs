#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod agents;
pub mod common;
pub mod macros;
pub mod prelude;
#[cfg(feature = "gpt")]
pub mod prompts;
pub mod traits;

#[cfg(feature = "net")]
pub mod collaboration;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub mod orchestrator;

#[cfg(feature = "cli")]
pub mod message;
