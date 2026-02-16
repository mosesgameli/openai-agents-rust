//! Model provider abstraction and implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

pub mod openai_responses;
pub mod openai_chat_completions;

pub use openai_responses::OpenAIResponsesModel;
pub use openai_chat_completions::OpenAIChatCompletionsModel;

/// Trait for model providers
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Complete a request
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;

    /// Stream a completion
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
}

/// A completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub tools: Option<Vec<ToolDefinition>>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// A message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

/// Tool definition for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

/// A completion response
#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
}

/// A tool call from the model
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// A streaming completion response
pub struct CompletionStream {
    // TODO: Implement streaming
}

impl CompletionStream {
    pub fn new(_stream: impl Send) -> Self {
        Self {}
    }
}
