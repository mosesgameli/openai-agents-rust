use futures::StreamExt;
use openai_agents::{Agent, RunItem, Runner, StreamEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create a simple agent
    let agent = Agent::builder("Chef")
        .instructions("You are a helpful chef. Suggest a simple recipe.")
        .model("gpt-4o-mini")
        .build();

    println!("Starting streaming agent to get a recipe...\n");

    // Run the agent with streaming
    let result = Runner::run_streamed(&agent, "Suggest a simple pasta recipe.").await?;

    // Stream events as they arrive
    let mut stream = result.stream_events();

    println!("Items created during the run:");
    println!("────────────────────────────");

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::RunItem(item_event) => match item_event.item {
                RunItem::MessageOutput { content } => {
                    println!("💬 [Message Output]\n{}", content);
                }
                RunItem::ToolCall { name, arguments } => {
                    println!("🔧 [Tool Call] {}({})", name, arguments);
                }
                RunItem::ToolOutput { name, output } => {
                    println!("📤 [Tool Output] {} -> {}", name, output);
                }
                RunItem::HandoffRequested { agent_name } => {
                    println!("🔄 [Handoff Requested] -> {}", agent_name);
                }
                RunItem::HandoffOccurred { agent_name } => {
                    println!("🔄 [Handoff Occurred] -> {}", agent_name);
                }
            },
            _ => {
                // Ignore raw response deltas in this example
            }
        }
    }

    println!("────────────────────────────");
    println!("\n✅ Run completed!");

    Ok(())
}
