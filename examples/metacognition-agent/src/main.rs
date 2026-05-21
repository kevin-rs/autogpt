// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Metacognition Agent Example
//!
//! Demonstrates the [`MetacognitionEngine`] embedded inside [`AgentGPT`]:
//! records task outcomes, reads per-task insights, and injects the strategy
//! context into the LLM prompt, then runs the agent via [`AutoGPT`].
//!
//! ## Running
//!
//! ```sh
//! export GEMINI_API_KEY=your_gemini_key
//!
//! cargo run
//! ```

use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Research Analyst";
    let behavior = "Summarise academic papers and extract key insights.";

    let mut probe = AgentGPT::new_borrowed(persona, behavior);

    let outcomes = [
        ("Fetch and parse PDF paper", "success", 0u8),
        ("Extract methodology section", "success", 0u8),
        ("Summarise results", "failed", 1u8),
        ("Generate bibliography", "failed", 2u8),
        ("Write executive summary", "success", 0u8),
    ];

    println!("=== Metacognition Engine ===\n");

    for (task, outcome, retries) in outcomes {
        let entry = probe.record_task_outcome(task, outcome, retries);
        println!("Task    : {task}");
        println!("Outcome : {outcome}  (retries: {retries})");
        println!("Insight : {}", entry.insight);
        if let Some(adj) = &entry.strategy_adjustment {
            println!("Strategy: {adj}");
        }
        println!();
    }

    println!("Should adjust? {}", probe.should_adjust_strategy());
    println!("Consecutive failures: {}", probe.consecutive_failures());
    println!("\n--- Context injected into LLM prompt ---");
    println!("{}", probe.metacognition_context());

    let agent = ArchitectGPT::new(persona, behavior).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => println!("{}", response),
        Err(err) => eprintln!("Agent error: {:?}", err),
    }
}
