//! Hello World example

use openai_agents::{Agent, Runner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    let agent = Agent::builder("Assistant")
        .instructions("You only respond in haikus.")
        .build();

    let result = Runner::run(&agent, "Tell me about recursion in programming.").await?;
    println!("{}", result.final_output());

    Ok(())
}
