//! Enhanced tool example with function_tool macro

use openai_agents::{function_tool, Agent, Result, Runner};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    city: String,
    temperature_range: String,
    conditions: String,
}

/// Get the current weather information for a specified city.
#[function_tool]
async fn get_weather(city: String) -> Weather {
    println!("[debug] get_weather called for {}", city);
    Weather {
        city,
        temperature_range: "14-20C".to_string(),
        conditions: "Sunny with wind.".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create an agent with the weather tool
    // The macro generates GET_WEATHERTool struct
    let agent = Agent::builder("Weather Assistant")
        .instructions(
            "You are a helpful weather assistant. Use the tools provided to answer questions.",
        )
        .tool(GET_WEATHERTool)
        .build();

    println!("Asking the agent about the weather...");
    let result = Runner::run(&agent, "What's the weather like in Nairobi today?").await?;

    println!("\nFinal Response:");
    println!("────────────────");
    println!("{}", result.final_output());

    Ok(())
}
