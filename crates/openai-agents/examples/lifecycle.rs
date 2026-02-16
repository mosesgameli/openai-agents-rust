use async_trait::async_trait;
use dotenvy::dotenv;
use openai_agents::{function_tool, Agent, AgentHooks, RunConfig, RunHooks, Runner};
use std::sync::Arc;

#[function_tool]
async fn get_weather(location: String) -> String {
    format!("The weather in {} is 22°C and sunny.", location)
}

struct LoggerHooks;

#[async_trait]
impl AgentHooks for LoggerHooks {
    async fn on_start(&self, agent: &Agent) -> openai_agents::error::Result<()> {
        println!("🚀 Agent '{}' starting...", agent.name);
        Ok(())
    }

    async fn on_end(&self, agent: &Agent, output: &str) -> openai_agents::error::Result<()> {
        println!("🏁 Agent '{}' finished with output: {}", agent.name, output);
        Ok(())
    }

    async fn on_tool_start(
        &self,
        agent: &Agent,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> openai_agents::error::Result<()> {
        println!(
            "🛠️ Agent '{}' calling tool '{}' with args: {}",
            agent.name, tool_name, arguments
        );
        Ok(())
    }
}

struct GlobalHooks;

#[async_trait]
impl RunHooks for GlobalHooks {
    async fn on_agent_start(&self, agent: &Agent) -> openai_agents::error::Result<()> {
        println!("🌐 Global: Agent '{}' is now active", agent.name);
        Ok(())
    }

    async fn on_handoff(&self, from: &Agent, to: &Agent) -> openai_agents::error::Result<()> {
        println!("🤝 Global: Handoff from '{}' to '{}'", from.name, to.name);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let weather_agent = Agent::builder("Weather Agent")
        .instructions("You are a weather expert. Use the get_weather tool.")
        .tool(GET_WEATHERTool)
        .hook(LoggerHooks)
        .build();

    let triage_agent = Agent::builder("Triage Agent")
        .instructions("If the user asks about weather, hand off to the Weather Agent.")
        .handoff(openai_agents::Handoff::new(weather_agent))
        .hook(LoggerHooks)
        .build();

    let config = RunConfig {
        run_hooks: vec![Arc::new(GlobalHooks)],
        ..Default::default()
    };

    println!("--- Starting run ---");
    let result =
        Runner::run_with_config(&triage_agent, "What is the weather in London?", config).await?;
    println!("--- Run complete ---");
    println!("Final Result: {}", result.final_output());

    Ok(())
}
