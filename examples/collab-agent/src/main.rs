// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Collab Agent Example
//!
//! Demonstrates attaching a [`CollabPool`] to [`AutoGPT`] so the orchestrator
//! executes agents sequentially in pool-determined round-robin order.
//!
//! Each agent in `agents![...]` corresponds to an entry in the pool's provider
//! list.  `AutoGPT::run()` picks a random start via [`CollabSelection::Random`]
//! and iterates through the agents in that order.
//!
//! ## Running
//!
//! ```sh
//! export GEMINI_API_KEY=your_gemini_key
//! export OPENAI_API_KEY=your_openai_key
//!
//! cargo run
//! ```

use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Lead Architect";
    let behavior = r#"Generate an architectural diagram for a task management system
    with a REST API, PostgreSQL database, Redis cache, and a React frontend."#;

    let pool = CollabPool::from_providers(vec!["gemini".to_string(), "openai".to_string()]);

    println!("Collab pool: {} provider(s)", pool.len());
    for idx in 0..pool.len() {
        println!("  [{idx}] {}", pool.provider_name(idx));
    }

    let agent_one = ArchitectGPT::new(persona, behavior).await;
    let agent_two = ArchitectGPT::new(persona, behavior).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent_one, agent_two])
        .with_collab_pool(pool)
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("Agent error: {:?}", err),
    }
}
