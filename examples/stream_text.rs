use openai_agents::{Agent, Runner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            openai_agents::StreamEvent::RawResponse(raw) => {
                println!("📡 Raw Response: {}", raw.data);
            }
            openai_agents::StreamEvent::RunItem(item_event) => {
                match item_event.item {
                    openai_agents::RunItem::MessageOutput { content } => {
                        println!("💬 Message Output: {}", content);
                    }
                    openai_agents::RunItem::ToolCall { name, arguments } => {
                        println!("🔧 Tool Call: {} with args: {}", name, arguments);
                    }
                    openai_agents::RunItem::ToolOutput { name, output } => {
                        println!("📤 Tool Output from {}: {}", name, output);
                    }
                    openai_agents::RunItem::HandoffRequested { target } => {
                        println!("🔄 Handoff to: {}", target);
                    }
                }
            }
            openai_agents::StreamEvent::AgentUpdated(agent_event) => {
                println!("🤖 Agent Updated: switching to new agent");
            }
        }
    }

    println!("─────────────────");
    println!("\n✅ Streaming completed!");

    Ok(())
}
