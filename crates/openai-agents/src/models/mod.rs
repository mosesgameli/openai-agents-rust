//! Model provider abstraction and implementations

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

pub mod openai_chat_completions;
pub mod openai_responses;

pub use openai_chat_completions::OpenAIChatCompletionsModel;
pub use openai_responses::OpenAIResponsesModel;

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
    pub response_format: Option<ResponseFormat>,
}

/// Response format for structured outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    Text,
    JsonObject,
    JsonSchema { json_schema: JsonSchemaFormat },
}

/// JSON schema format for structured outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaFormat {
    pub name: String,
    pub description: Option<String>,
    pub schema: Value,
    pub strict: Option<bool>,
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
    inner: Box<dyn futures::Stream<Item = Result<StreamChunk>> + Send + Unpin>,
}

/// A chunk from a streaming response
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Text delta (incremental content)
    pub delta: Option<String>,
    /// Tool call deltas
    pub tool_call_deltas: Vec<ToolCallDelta>,
    /// Finish reason if this is the last chunk
    pub finish_reason: Option<String>,
}

/// A delta for a tool call
#[derive(Debug, Clone)]
pub struct ToolCallDelta {
    pub index: usize,
    pub id: Option<String>,
    pub name: Option<String>,
    pub arguments: Option<String>,
}

impl CompletionStream {
    pub fn new<S>(stream: S) -> Self
    where
        S: futures::Stream<Item = Result<StreamChunk>> + Send + Unpin + 'static,
    {
        Self {
            inner: Box::new(stream),
        }
    }

    /// Get the next chunk from the stream
    pub async fn next(&mut self) -> Option<Result<StreamChunk>> {
        use futures::StreamExt;
        self.inner.next().await
    }
}
