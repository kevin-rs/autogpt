#![doc(
    html_logo_url = "https://raw.githubusercontent.com/kevin-rs/autogpt/refs/heads/main/iac-rs/assets/logo.webp",
    html_favicon_url = "https://github.com/kevin-rs/autogpt/blob/main/iac-rs/assets/favicon.png"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

pub mod client;
pub mod crypto;
pub mod message;
pub mod prelude;
pub mod server;
pub mod transport;
