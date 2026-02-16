//! Session management example

use openai_agents::{Agent, RunConfig, Runner, SqliteSession};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let agent = Agent::builder("Assistant")
        .instructions("You are a helpful assistant. Reply concisely.")
        .build();

    // Create a session
    let session = SqliteSession::new("user_123", "conversations.db").await?;
    let session_arc = Arc::new(session);

    let config = RunConfig {
        session: Some(session_arc.clone()),
        ..Default::default()
    };

    // First turn
    println!("First turn:");
    let result = Runner::run_with_config(
        &agent,
        "What city is the Golden Gate Bridge in?",
        config.clone(),
    )
    .await?;
    println!("Response: {}\n", result.final_output());

    // Second turn - agent should remember context
    println!("Second turn (with context):");
    let result = Runner::run_with_config(&agent, "What state is it in?", config.clone()).await?;
    println!("Response: {}\n", result.final_output());

    // Third turn
    println!("Third turn (with context):");
    let result = Runner::run_with_config(&agent, "What's the population?", config).await?;
    println!("Response: {}", result.final_output());

    Ok(())
}
