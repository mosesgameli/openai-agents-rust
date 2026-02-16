//! Enhanced tool example with function_tool macro

use openai_agents::{Agent, Runner};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Weather {
    city: String,
    temperature_range: String,
    conditions: String,
}

// Simple function tool (without macro for now, until macro is fully implemented)
async fn get_weather_impl(city: String) -> Weather {
    Weather {
        city,
        temperature_range: "14-20C".to_string(),
        conditions: "Sunny with wind.".to_string(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // For now, we'll create a simple agent without tools
    // Once the macro is complete, we can use #[function_tool]
    let agent = Agent::builder("Assistant")
        .instructions("You are a helpful weather assistant.")
        .build();

    let result = Runner::run(&agent, "What's the weather like today?").await?;
    println!("{}", result.final_output());

    Ok(())
}
