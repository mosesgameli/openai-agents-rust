//! OpenAI Agents SDK for Rust
//!
//! A lightweight yet powerful framework for building multi-agent workflows.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use openai_agents::{Agent, Runner};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let agent = Agent::builder("Assistant")
//!         .instructions("You are a helpful assistant.")
//!         .build();
//!
//!     let result = Runner::run(&agent, "Hello!").await?;
//!     println!("{}", result.final_output());
//!     Ok(())
//! }
//! ```

pub mod agent;
pub mod config;
pub mod error;
pub mod guardrail;
pub mod handoff;
pub mod lifecycle;
pub mod models;
pub mod result;
pub mod runner;
pub mod session;
pub mod stream_events;
pub mod streaming;
pub mod tool;
pub mod tracing_impl;

// Re-exports for convenience
pub use agent::{Agent, AgentBuilder};
pub use config::{get_default_client, set_default_openai_client, set_default_openai_key};
pub use error::{AgentError, Result};
pub use guardrail::{
    GuardrailResult, InputGuardrail, OutputGuardrail, ToolInputGuardrail, ToolOutputGuardrail,
};
pub use handoff::Handoff;
pub use lifecycle::{AgentHooks, RunHooks};
pub use models::{
    CompletionRequest, CompletionResponse, ModelProvider, OpenAIChatCompletionsModel,
    OpenAIResponsesModel,
};
pub use result::{RunResult, RunResultStreaming};
pub use runner::{RunConfig, Runner};
pub use session::{InMemorySession, Session, SessionSettings};

#[cfg(feature = "sqlite-session")]
pub use session::SqliteSession;

pub use stream_events::{
    AgentUpdatedEvent, RawResponseEvent, RunItem, RunItemEventName, RunItemStreamEvent, StreamEvent,
};
pub use streaming::StreamedRunResult;
pub use tool::{FunctionTool, Tool};

// Re-export macros
pub use openai_agents_macros::function_tool;

/// The version of this crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
