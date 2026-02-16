# OpenAI Agents Rust - Quick Reference

## Installation

```toml
[dependencies]
openai-agents = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Basic Usage

### Simple Agent
```rust
use openai_agents::{Agent, Runner};

let agent = Agent::builder("Assistant")
    .instructions("You are helpful.")
    .build();

let result = Runner::run(&agent, "Hello!").await?;
println!("{}", result.final_output());
```

### With Configuration
```rust
let agent = Agent::builder("Assistant")
    .instructions("You are helpful.")
    .model("gpt-4")
    .parallel_tool_calls(true)
    .build();
```

### With Session
```rust
use openai_agents::{RunConfig, SqliteSession};
use std::sync::Arc;

let session = SqliteSession::new("user_123", "db.db").await?;
let config = RunConfig {
    session: Some(Arc::new(session)),
    max_turns: 50,
};

let result = Runner::run_with_config(&agent, "Hello!", config).await?;
```

### Multi-Agent Handoffs
```rust
use openai_agents::Handoff;

let specialist = Agent::builder("Specialist").build();

let triage = Agent::builder("Triage")
    .handoff(Handoff::new(specialist))
    .build();
```

## API Key Setup

**Using .env file (recommended):**
```bash
# Copy the example file
cp .env.example .env

# Edit .env and add your key
# OPENAI_API_KEY=sk-...
```

**Or use environment variable:**
```bash
export OPENAI_API_KEY=sk-...
```

All examples automatically load the `.env` file using `dotenvy::dotenv()`.

## Error Handling

```rust
use openai_agents::{AgentError, Result};

match Runner::run(&agent, input).await {
    Ok(result) => println!("{}", result.final_output()),
    Err(AgentError::MaxTurnsExceeded(n)) => {
        eprintln!("Exceeded {} turns", n);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Features

```toml
# Default (includes SQLite sessions)
openai-agents = "0.1"

# With Redis sessions
openai-agents = { version = "0.1", features = ["redis-session"] }

# All features
openai-agents = { version = "0.1", features = ["full"] }
```

## Common Commands

```bash
# Build
cargo build

# Test
cargo test

# Run example
cargo run --example hello_world

# Generate docs
cargo doc --open

# Format code
cargo fmt

# Lint
cargo clippy
```

## Module Structure

- `openai_agents::Agent` - Agent configuration
- `openai_agents::Runner` - Execution engine
- `openai_agents::Tool` - Tool trait
- `openai_agents::Handoff` - Agent handoffs
- `openai_agents::Session` - Session management
- `openai_agents::AgentError` - Error types
- `openai_agents::RunResult` - Result types

## Examples Location

- `examples/basic/hello_world.rs`
- `examples/basic/tools.rs`
- `examples/basic/handoffs.rs`
- `examples/basic/sessions.rs`
