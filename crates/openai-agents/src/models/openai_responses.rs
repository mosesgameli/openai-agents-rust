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

    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream> {
        use crate::models::{StreamChunk, ToolCallDelta};
        use futures::StreamExt;

        let mut openai_request = self.convert_request(request);

        // Enable streaming
        openai_request.stream = Some(true);

        let stream = self
            .client
            .chat()
            .create_stream(openai_request)
            .await
            .map_err(|e| AgentError::ModelError(e.to_string()))?;

        // Convert OpenAI stream to our StreamChunk format
        let converted_stream = stream.map(|result| {
            result
                .map_err(|e| AgentError::ModelError(e.to_string()))
                .and_then(|response| {
                    let choice = response.choices.first();

                    let delta = choice
                        .and_then(|c| c.delta.content.as_ref())
                        .map(|s| s.to_string());

                    let tool_call_deltas = choice
                        .and_then(|c| c.delta.tool_calls.as_ref())
                        .map(|calls| {
                            calls
                                .iter()
                                .map(|tc| ToolCallDelta {
                                    index: tc.index as usize,
                                    id: tc.id.clone(),
                                    name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                    arguments: tc
                                        .function
                                        .as_ref()
                                        .and_then(|f| f.arguments.clone()),
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let finish_reason = choice
                        .and_then(|c| c.finish_reason.as_ref())
                        .map(|r| format!("{:?}", r));

                    Ok(StreamChunk {
                        delta,
                        tool_call_deltas,
                        finish_reason,
                    })
                })
        });

        Ok(CompletionStream::new(converted_stream))
    }
}
