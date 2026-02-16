use openai_agents::{Agent, InMemorySession, RunConfig, Runner, Session};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create an agent with session support
    let agent = Agent::builder("MemoryAgent")
        .instructions("You are a friendly assistant who remembers things about the user.")
        .build();

    // Create a session
    let session = Arc::new(InMemorySession::new());
    println!("Starting new session...");

    // First turn
    println!("\nFirst turn:");
    let config1 = RunConfig {
        session: Some(session.clone()),
        ..Default::default()
    };
    let result =
        Runner::run_with_config(&agent, "My name is Moses and I like Rust.", config1).await?;
    println!("Assistant: {}", result.final_output());

    // Second turn - agent should remember the name
    println!("\nSecond turn:");
    let config2 = RunConfig {
        session: Some(session.clone()),
        ..Default::default()
    };
    let result =
        Runner::run_with_config(&agent, "What is my name and what do I like?", config2).await?;
    println!("Assistant: {}", result.final_output());

    println!(
        "\nConversation history has {} items.",
        session.get_items(None).await?.len()
    );

    Ok(())
}
