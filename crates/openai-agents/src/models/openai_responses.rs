//! OpenAI Responses API model provider

use async_openai::{config::OpenAIConfig, types::*, Client};
use async_trait::async_trait;

use crate::{
    config::get_default_client,
    error::{AgentError, Result},
    models::{CompletionRequest, CompletionResponse, CompletionStream, ModelProvider, ToolCall},
};

/// OpenAI Responses API model provider
pub struct OpenAIResponsesModel {
    client: Client<OpenAIConfig>,
}

impl OpenAIResponsesModel {
    /// Create a new OpenAI Responses model using the default client
    pub fn new() -> Self {
        Self {
            client: get_default_client(),
        }
    }

    /// Create with a specific API key
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        Self {
            client: Client::with_config(config),
        }
    }

    /// Create with a custom client
    pub fn with_client(client: Client<OpenAIConfig>) -> Self {
        Self { client }
    }

    fn convert_request(&self, request: CompletionRequest) -> CreateChatCompletionRequest {
        let messages: Vec<ChatCompletionRequestMessage> = request
            .messages
            .into_iter()
            .map(|m| match m.role.as_str() {
                "user" => ChatCompletionRequestUserMessageArgs::default()
                    .content(m.content)
                    .build()
                    .unwrap()
                    .into(),
                "assistant" => ChatCompletionRequestAssistantMessageArgs::default()
                    .content(m.content)
                    .build()
                    .unwrap()
                    .into(),
                "system" => ChatCompletionRequestSystemMessageArgs::default()
                    .content(m.content)
                    .build()
                    .unwrap()
                    .into(),
                _ => ChatCompletionRequestUserMessageArgs::default()
                    .content(m.content)
                    .build()
                    .unwrap()
                    .into(),
            })
            .collect();

        CreateChatCompletionRequestArgs::default()
            .model(&request.model)
            .messages(messages)
            .build()
            .unwrap()
    }

    fn convert_response(
        &self,
        response: CreateChatCompletionResponse,
    ) -> Result<CompletionResponse> {
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AgentError::ModelBehaviorError("No choices in response".to_string()))?;

        let content = choice.message.content.clone();
        let tool_calls = choice
            .message
            .tool_calls
            .as_ref()
            .map(|calls| {
                calls
                    .iter()
                    .map(|call| ToolCall {
                        id: call.id.clone(),
                        name: call.function.name.clone(),
                        arguments: serde_json::from_str(&call.function.arguments)
                            .unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CompletionResponse {
            content,
            tool_calls,
            finish_reason: choice.finish_reason.as_ref().map(|r| format!("{:?}", r)),
        })
    }
}

impl Default for OpenAIResponsesModel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelProvider for OpenAIResponsesModel {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let openai_request = self.convert_request(request);

        let response = self
            .client
            .chat()
            .create(openai_request)
            .await
            .map_err(|e| AgentError::ModelError(e.to_string()))?;

        self.convert_response(response)
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        // TODO: Implement streaming
        Ok(CompletionStream::new(()))
    }
}
