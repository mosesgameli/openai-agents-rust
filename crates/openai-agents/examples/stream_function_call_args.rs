use anyhow::Result;
use futures::StreamExt;
use openai_agents::{Agent, Runner, StreamEvent};

#[openai_agents::function_tool]
async fn write_file(filename: String, _content: String) -> String {
    format!("File {} written successfully", filename)
}

#[openai_agents::function_tool]
async fn create_config(
    project_name: String,
    version: String,
    _dependencies: Option<Vec<String>>,
) -> String {
    format!("Config for {} v{} created", project_name, version)
}

#[tokio::main]
async fn main() -> Result<()> {
    let agent = Agent::builder("CodeGenerator")
        .instructions("You are a helpful coding assistant. Use the provided tools to create files and configurations.")
        .tool(WRITE_FILETool)
        .tool(CREATE_CONFIGTool)
        .build();

    println!("🚀 Function Call Arguments Streaming Demo (Rust)");

    let result = Runner::run_streamed(
        &agent,
        "Create a Python web project called 'my-app' with FastAPI. Version 1.0.0, dependencies: fastapi, uvicorn",
    ).await?;

    let mut stream = result.stream_events();

    while let Some(event) = stream.next().await {
        match event {
            StreamEvent::RawResponse(raw) => {
                // In this implementation, RawResponse only carries text deltas for now.
                // We'll update the Runner to emit tool call deltas as well.
                print!("{}", raw.data);
                use std::io::{Write, stdout};
                stdout().flush().unwrap();
            }
            StreamEvent::RunItem(item_event) => {
                use openai_agents::{RunItem, stream_events::RunItemEventName};
                match item_event.name {
                    RunItemEventName::ToolCalled => {
                        if let RunItem::ToolCall { name, arguments } = item_event.item {
                            println!("\n📞 Tool called: {}({})", name, arguments);
                        }
                    }
                    RunItemEventName::ToolOutput => {
                        if let RunItem::ToolOutput { name, output } = item_event.item {
                            println!("✅ Tool output received: {} -> {}", name, output);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    println!("\nFinal Output: {}", result.final_output().await?);

    Ok(())
}
