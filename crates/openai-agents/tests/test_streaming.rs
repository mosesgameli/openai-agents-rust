use async_trait::async_trait;
use futures::{stream, StreamExt};
use openai_agents::error::Result;
use openai_agents::models::{
    CompletionRequest, CompletionResponse, CompletionStream, ModelProvider, StreamChunk,
    ToolCallDelta,
};
use openai_agents::{Agent, RunConfig, Runner, StreamEvent};
use serde_json::{json, Value};
use std::sync::Arc;

struct MockModel {
    chunks: Vec<StreamChunk>,
}

#[async_trait]
impl ModelProvider for MockModel {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
        Ok(CompletionResponse {
            content: Some("Mock response".to_string()),
            tool_calls: vec![],
            finish_reason: Some("stop".to_string()),
        })
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<CompletionStream> {
        let stream = stream::iter(self.chunks.clone().into_iter().map(Ok)).boxed();
        Ok(CompletionStream::new(stream))
    }
}

#[tokio::test]
async fn test_runner_run_streamed_basic() {
    let chunks = vec![
        StreamChunk {
            delta: Some("Hello".to_string()),
            tool_call_deltas: vec![],
            finish_reason: None,
        },
        StreamChunk {
            delta: Some(" world".to_string()),
            tool_call_deltas: vec![],
            finish_reason: None,
        },
        StreamChunk {
            delta: None,
            tool_call_deltas: vec![],
            finish_reason: Some("stop".to_string()),
        },
    ];

    let mock_model = Arc::new(MockModel { chunks });
    let agent = Agent::builder("Test Agent").model("mock").build();

    let config = RunConfig {
        model_override: Some(mock_model),
        ..Default::default()
    };

    let result = Runner::run_streamed_with_config(&agent, "Hi", config)
        .await
        .unwrap();
    let mut events = result.stream_events();

    let mut collected_text = String::new();
    let mut message_output_count = 0;

    while let Some(event) = events.next().await {
        match event {
            StreamEvent::RawResponse(raw) => {
                collected_text.push_str(&raw.data);
            }
            StreamEvent::RunItem(item_event) => {
                if let openai_agents::RunItem::MessageOutput { content } = item_event.item {
                    assert_eq!(content, "Hello world");
                    message_output_count += 1;
                }
            }
            _ => {}
        }
    }

    assert_eq!(collected_text, "Hello world");
    assert_eq!(message_output_count, 1);
}

#[tokio::test]
async fn test_runner_run_streamed_tool_call() {
    let chunks = vec![
        StreamChunk {
            delta: None,
            tool_call_deltas: vec![ToolCallDelta {
                index: 0,
                id: Some("call_1".to_string()),
                name: Some("get_weather".to_string()),
                arguments: Some("{\"location\":".to_string()),
            }],
            finish_reason: None,
        },
        StreamChunk {
            delta: None,
            tool_call_deltas: vec![ToolCallDelta {
                index: 0,
                id: None,
                name: None,
                arguments: Some("\"London\"}".to_string()),
            }],
            finish_reason: Some("tool_calls".to_string()),
        },
    ];

    let mock_model = Arc::new(MockModel { chunks });

    // Define a fake tool
    use openai_agents::tool::Tool;
    use serde_json::json;

    struct WeatherTool;
    #[async_trait]
    impl Tool for WeatherTool {
        fn name(&self) -> &str {
            "get_weather"
        }
        fn description(&self) -> &str {
            "Get weather"
        }
        fn parameters_schema(&self) -> Value {
            json!({})
        }
        async fn execute(&self, _args: Value) -> Result<Value> {
            Ok(json!({"temp": 20}))
        }
    }

    let agent = Agent::builder("Test Agent")
        .model("mock")
        .tool(WeatherTool)
        .build();

    let config = RunConfig {
        model_override: Some(mock_model),
        ..Default::default()
    };

    let result = Runner::run_streamed_with_config(&agent, "Hi", config)
        .await
        .unwrap();
    let mut events = result.stream_events();

    let mut tool_called = false;
    let mut tool_output_received = false;

    while let Some(event) = events.next().await {
        match event {
            StreamEvent::RunItem(item_event) => match item_event.item {
                openai_agents::RunItem::ToolCall { name, arguments } => {
                    assert_eq!(name, "get_weather");
                    assert_eq!(arguments, json!({"location": "London"}));
                    tool_called = true;
                }
                openai_agents::RunItem::ToolOutput { name, output } => {
                    assert_eq!(name, "get_weather");
                    assert!(output.contains("20"));
                    tool_output_received = true;
                }
                _ => {}
            },
            _ => {}
        }
    }

    assert!(tool_called);
    assert!(tool_output_received);
}
