use openai_agents::{Agent, Result as AgentResult, Runner};

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create a basic agent
    let agent = Agent::builder("Assistant")
        .instructions(
            "You are a helpful assistant. Keep your responses short and use a haiku format.",
        )
        .build();

    println!("Asking the agent for a haiku...");
    let result: AgentResult<_> = Runner::run(&agent, "Help me understand recursion.").await;
    let result = result?;

    println!("\nResponse:");
    println!("─────────");
    println!("{}", result.final_output());

    Ok(())
}
