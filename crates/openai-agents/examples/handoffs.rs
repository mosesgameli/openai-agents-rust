// Handoffs example - multi-agent workflow

use openai_agents::{Agent, Handoff, Runner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create specialized agents
    let spanish_agent = Agent::builder("Spanish Agent")
        .instructions("You only speak Spanish. Respond to all queries in Spanish.")
        .build();

    let english_agent = Agent::builder("English Agent")
        .instructions("You only speak English. Respond to all queries in English.")
        .build();

    // Create triage agent that hands off to specialists
    let triage_agent = Agent::builder("Triage Agent")
        .instructions(
            "Determine the language of the user's message and hand off to the appropriate agent. \
             If the message is in Spanish, hand off to the Spanish agent. \
             If the message is in English, hand off to the English agent.",
        )
        .handoff(Handoff::new(spanish_agent).with_description("For Spanish language queries"))
        .handoff(Handoff::new(english_agent).with_description("For English language queries"))
        .build();

    // Test with Spanish input
    println!("Testing with Spanish input:");
    let result = Runner::run(&triage_agent, "Hola, ¿cómo estás?").await?;
    println!("Response: {}\n", result.final_output());

    // Test with English input
    println!("Testing with English input:");
    let result = Runner::run(&triage_agent, "Hello, how are you?").await?;
    println!("Response: {}", result.final_output());

    Ok(())
}
