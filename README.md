# OpenAI Agents SDK for Rust

[![Crates.io](https://img.shields.io/crates/v/openai-agents.svg)](https://crates.io/crates/openai-agents)
[![Documentation](https://docs.rs/openai-agents/badge.svg)](https://docs.rs/openai-agents)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A lightweight yet powerful framework for building multi-agent workflows in Rust. This is a feature-for-feature port of the [OpenAI Agents Python SDK](https://github.com/openai/openai-agents-python).

## Features

- 🤖 **Multi-agent workflows** with handoffs
- 🛠️ **Function tools** with procedural macros
- 🛡️ **Guardrails** for input/output validation
- 💾 **Session management** (SQLite, Redis)
- 📊 **Tracing** infrastructure
- 🔄 **Streaming** support
- 🎯 **Type-safe** with Rust's type system
- ⚡ **Async/await** with Tokio

## Configuration

Set your OpenAI API key using a `.env` file:

```bash
# Copy the example file
cp .env.example .env

# Edit .env and add your API key
# OPENAI_API_KEY=sk-...
```

Or set it as an environment variable:

```bash
export OPENAI_API_KEY=sk-...
```

The examples will automatically load the `.env` file if present.

```rust
use openai_agents::{Agent, Runner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::builder("Assistant")
        .instructions("You are a helpful assistant.")
        .build();

    let result = Runner::run(&agent, "Hello!").await?;
    println!("{}", result.final_output());
    
    Ok(())
}
```

### With Tools

```rust
use openai_agents::{Agent, Runner, function_tool};

#[function_tool]
async fn get_weather(city: String) -> String {
    format!("The weather in {} is sunny", city)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::builder("Assistant")
        .instructions("You are a helpful agent.")
        .tool(get_weather)
        .build();

    let result = Runner::run(&agent, "What's the weather in Tokyo?").await?;
    println!("{}", result.final_output());
    
    Ok(())
}
```

### With Handoffs

```rust
use openai_agents::{Agent, Runner, Handoff};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spanish_agent = Agent::builder("Spanish Agent")
        .instructions("You only speak Spanish.")
        .build();

    let english_agent = Agent::builder("English Agent")
        .instructions("You only speak English.")
        .build();

    let triage_agent = Agent::builder("Triage Agent")
        .instructions("Hand off to the appropriate agent based on language.")
        .handoff(Handoff::new(spanish_agent))
        .handoff(Handoff::new(english_agent))
        .build();

    let result = Runner::run(&triage_agent, "Hola, ¿cómo estás?").await?;
    println!("{}", result.final_output());
    
    Ok(())
}
```

## Configuration

Set your OpenAI API key:

```rust
use openai_agents::set_default_openai_key;

// Option 1: Set programmatically
set_default_openai_key("sk-...");

// Option 2: Use environment variable (automatic)
// export OPENAI_API_KEY=sk-...
```

## Features

### Default Features

- `sqlite-session` - SQLite-based session storage

### Optional Features

- `redis-session` - Redis-based session storage
- `full` - All features enabled

Enable features in `Cargo.toml`:

```toml
[dependencies]
openai-agents = { version = "0.1", features = ["redis-session"] }
```

## Session Management

```rust
use openai_agents::{Agent, Runner, RunConfig, SqliteSession};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Agent::builder("Assistant").build();
    
    let session = SqliteSession::new("user_123", "conversations.db").await?;
    
    let config = RunConfig {
        session: Some(Arc::new(session)),
        ..Default::default()
    };
    
    let result = Runner::run_with_config(&agent, "Hello!", config).await?;
    println!("{}", result.final_output());
    
    Ok(())
}
```

## Examples

See the [`examples/`](examples/) directory for more examples:

- `basic/hello_world.rs` - Simple hello world
- More examples coming soon...

## Documentation

Full API documentation is available at [docs.rs/openai-agents](https://docs.rs/openai-agents).

## Comparison with Python SDK

| Python | Rust |
|--------|------|
| `from agents import Agent, Runner` | `use openai_agents::{Agent, Runner};` |
| `@function_tool` | `#[function_tool]` |
| `async def main()` | `#[tokio::main] async fn main()` |
| `result.final_output` | `result.final_output()` |
| Exception handling | `Result<T, E>` with `?` |

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example hello_world

# Build documentation
cargo doc --open
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgements

This is a Rust port of the [OpenAI Agents Python SDK](https://github.com/openai/openai-agents-python). Special thanks to the OpenAI team and the Rust community.
