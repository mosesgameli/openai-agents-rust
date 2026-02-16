use futures::StreamExt;
use openai_agents::{Agent, RunItem, Runner, StreamEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    // Create a simple agent
    let agent = Agent::builder("Assistant")
        .instructions("You are a helpful assistant. Be concise.")
        .model("gpt-4o-mini")
        .build();

    println!("Starting streaming agent...\n");

    // Run the agent with streaming
    let result = Runner::run_streamed(&agent, "Tell me a short joke about programming.").await?;

    // Stream events as they arrive
    let mut stream = result.stream_events();

    println!("Streaming events:");
    println!("─────────────────");

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::RawResponse(raw) => {
                print!("{}", raw.data);
                use std::io::{Write, stdout};
                stdout().flush().unwrap();
            }
            StreamEvent::RunItem(item_event) => match item_event.item {
                RunItem::MessageOutput { content: _ } => {
                    // Final content is collected in result
                }
                RunItem::ToolCall { name, arguments } => {
                    println!("\n🔧 Tool Call: {} with args: {}", name, arguments);
                }
                RunItem::ToolOutput { name, output } => {
                    println!("📤 Tool Output from {}: {}", name, output);
                }
                RunItem::HandoffRequested { agent_name } => {
                    println!("\n🔄 Handoff requested to: {}", agent_name);
                }
                RunItem::HandoffOccurred { agent_name } => {
                    println!("🔄 Handoff occurred to: {}", agent_name);
                }
            },
            StreamEvent::AgentUpdated(agent_event) => {
                println!(
                    "\n🤖 Agent Updated: switching to assistant '{}'",
                    agent_event.new_agent.name
                );
            }
        }
    }

    println!("\n─────────────────");
    println!("\n✅ Streaming completed!");

    Ok(())
}
