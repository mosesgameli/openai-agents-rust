//! OpenAI Chat Completions API model provider

use async_openai::{Client, config::OpenAIConfig};
use async_trait::async_trait;

use crate::{
    error::Result,
    models::{CompletionRequest, CompletionResponse, CompletionStream, ModelProvider},
};

use super::openai_responses::OpenAIResponsesModel;

/// OpenAI Chat Completions API model provider
///
/// Note: This uses the same implementation as OpenAIResponsesModel
/// since both APIs use the same endpoint
pub struct OpenAIChatCompletionsModel {
    inner: OpenAIResponsesModel,
}

impl OpenAIChatCompletionsModel {
    /// Create a new OpenAI Chat Completions model using the default client
    pub fn new() -> Self {
        Self {
            inner: OpenAIResponsesModel::new(),
        }
    }

    /// Create with a specific API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            inner: OpenAIResponsesModel::with_api_key(api_key),
        }
    }

    /// Create with a custom client
    pub fn with_client(client: Client<OpenAIConfig>) -> Self {
        Self {
            inner: OpenAIResponsesModel::with_client(client),
        }
    }
}

impl Default for OpenAIChatCompletionsModel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelProvider for OpenAIChatCompletionsModel {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        self.inner.complete(request).await
    }

    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        self.inner.stream(request).await
    }
}
