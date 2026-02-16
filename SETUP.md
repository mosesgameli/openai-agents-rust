# Setup Instructions

## 1. Configure API Key

Copy the example environment file:
```bash
cp .env.example .env
```

Edit `.env` and add your OpenAI API key:
```bash
OPENAI_API_KEY=sk-your-actual-api-key-here
```

## 2. Build the Project

```bash
cargo build
```

## 3. Run Tests

```bash
cargo test
```

## 4. Run Examples

All examples will automatically load the `.env` file:

```bash
# Basic agent
cargo run --example hello_world

# Multi-agent handoffs
cargo run --example handoffs

# Conversation history
cargo run --example sessions
```

## 5. Using in Your Code

The examples show how to use `dotenvy::dotenv()` to load the `.env` file:

```rust
use openai_agents::{Agent, Runner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenvy::dotenv().ok();
    
    let agent = Agent::builder("Assistant")
        .instructions("You are helpful.")
        .build();
    
    let result = Runner::run(&agent, "Hello!").await?;
    println!("{}", result.final_output());
    
    Ok(())
}
```

## Troubleshooting

**Error: "No API key found"**
- Make sure `.env` file exists in the project root
- Verify `OPENAI_API_KEY` is set in `.env`
- Check that you're calling `dotenvy::dotenv().ok()` before using the agent

**Error: "Invalid API key"**
- Verify your API key is correct
- Make sure there are no extra spaces or quotes in `.env`

**Build errors**
- Run `cargo clean` and `cargo build` again
- Make sure Rust 1.75+ is installed: `rustc --version`
