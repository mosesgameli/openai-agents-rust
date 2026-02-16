//! Runner for executing agents

use std::sync::Arc;

use crate::{
    agent::Agent,
    error::{AgentError, Result},
    models::{CompletionRequest, Message, ModelProvider, OpenAIResponsesModel, ToolDefinition},
    result::RunResult,
    session::Session,
};

/// Configuration for running an agent
#[derive(Clone)]
pub struct RunConfig {
    /// Maximum number of turns before stopping
    pub max_turns: usize,
    /// Optional session for conversation history
    pub session: Option<Arc<dyn Session>>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            max_turns: 100,
            session: None,
        }
    }
}

/// Runner for executing agents
pub struct Runner;

impl Runner {
    /// Run an agent with the given input
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openai_agents::{Agent, Runner};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let agent = Agent::builder("Assistant")
    ///     .instructions("You are helpful.")
    ///     .build();
    ///
    /// let result = Runner::run(&agent, "Hello!").await?;
    /// println!("{}", result.final_output());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run(agent: &Agent, input: impl Into<String>) -> Result<RunResult> {
        Self::run_with_config(agent, input, RunConfig::default()).await
    }

    /// Run an agent with custom configuration
    pub async fn run_with_config(
        agent: &Agent,
        input: impl Into<String>,
        config: RunConfig,
    ) -> Result<RunResult> {
        let input = input.into();
        let mut messages = Vec::new();

        // Add system message if agent has instructions
        if !agent.instructions.is_empty() {
            messages.push(Message {
                role: "system".to_string(),
                content: agent.instructions.clone(),
            });
        }

        // Load session history if available
        if let Some(session) = &config.session {
            let history = session.get_items(None).await?;
            for item in history {
                if let Ok(msg) = serde_json::from_value::<Message>(item) {
                    messages.push(msg);
                }
            }
        }

        // Add user input
        messages.push(Message {
            role: "user".to_string(),
            content: input.clone(),
        });

        // Create model provider
        let model = OpenAIResponsesModel::new();

        // Convert agent tools to tool definitions
        let tools = if !agent.tools.is_empty() {
            Some(
                agent
                    .tools
                    .iter()
                    .map(|tool| ToolDefinition {
                        name: tool.name().to_string(),
                        description: tool.description().to_string(),
                        parameters: tool.parameters_schema(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        // Main agent loop
        for turn in 0..config.max_turns {
            let request = CompletionRequest {
                messages: messages.clone(),
                model: agent.model.clone(),
                tools: tools.clone(),
                max_tokens: None,
                temperature: None,
            };

            let response = model.complete(request).await?;

            // Check if we have a final output (no tool calls)
            if let Some(content) = &response.content {
                if response.tool_calls.is_empty() {
                    // Save conversation to session if available
                    if let Some(session) = &config.session {
                        let assistant_msg = Message {
                            role: "assistant".to_string(),
                            content: content.clone(),
                        };
                        session
                            .add_items(vec![
                                serde_json::to_value(&messages[messages.len() - 1])?,
                                serde_json::to_value(&assistant_msg)?,
                            ])
                            .await?;
                    }

                    return Ok(RunResult::new(content.clone()));
                }

                // Add assistant message with content
                messages.push(Message {
                    role: "assistant".to_string(),
                    content: content.clone(),
                });
            }

            // Handle tool calls
            if !response.tool_calls.is_empty() {
                for tool_call in &response.tool_calls {
                    // Find the tool
                    let tool = agent
                        .tools
                        .iter()
                        .find(|t| t.name() == tool_call.name)
                        .ok_or_else(|| {
                            AgentError::tool_failed(
                                &tool_call.name,
                                format!("Tool '{}' not found", tool_call.name),
                            )
                        })?;

                    // Execute the tool
                    let result = tool
                        .execute(tool_call.arguments.clone())
                        .await
                        .map_err(|e| AgentError::tool_failed(&tool_call.name, e.to_string()))?;

                    // Add tool result as a message
                    let result_str = serde_json::to_string(&result)?;
                    messages.push(Message {
                        role: "tool".to_string(),
                        content: format!("Tool '{}' returned: {}", tool_call.name, result_str),
                    });
                }
            }

            // Check if we've exceeded max turns
            if turn >= config.max_turns - 1 {
                return Err(AgentError::MaxTurnsExceeded(config.max_turns));
            }
        }

        Err(AgentError::MaxTurnsExceeded(config.max_turns))
    }

    /// Run an agent with streaming enabled
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use openai_agents::{Agent, Runner};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let agent = Agent::builder("Assistant")
    ///     .instructions("You are helpful.")
    ///     .build();
    ///
    /// let result = Runner::run_streamed(&agent, "Hello!").await?;
    ///
    /// // Stream events
    /// let mut stream = result.stream_events();
    /// while let Some(event) = stream.next().await {
    ///     // Process event
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_streamed(
        agent: &Agent,
        input: impl Into<String>,
    ) -> Result<crate::streaming::StreamedRunResult> {
        Self::run_streamed_with_config(agent, input, RunConfig::default()).await
    }

    /// Run an agent with streaming and custom configuration
    pub async fn run_streamed_with_config(
        agent: &Agent,
        input: impl Into<String>,
        config: RunConfig,
    ) -> Result<crate::streaming::StreamedRunResult> {
        use crate::stream_events::{
            RawResponseEvent, RunItem, RunItemEventName, RunItemStreamEvent, StreamEvent,
        };
        use tokio::sync::mpsc;

        let (tx, rx) = mpsc::unbounded_channel();
        let input = input.into();

        // Spawn a task to run the agent and emit events
        let agent = agent.clone();
        tokio::spawn(async move {
            let mut messages = Vec::new();

            // Add system message if agent has instructions
            if !agent.instructions.is_empty() {
                messages.push(Message {
                    role: "system".to_string(),
                    content: agent.instructions.clone(),
                });
            }

            // Load session history if available
            if let Some(session) = &config.session {
                if let Ok(history) = session.get_items(None).await {
                    for item in history {
                        if let Ok(msg) = serde_json::from_value::<Message>(item) {
                            messages.push(msg);
                        }
                    }
                }
            }

            // Add user input
            messages.push(Message {
                role: "user".to_string(),
                content: input.clone(),
            });

            // Create model provider
            let model = OpenAIResponsesModel::new();

            // Convert agent tools to tool definitions
            let tools = if !agent.tools.is_empty() {
                Some(
                    agent
                        .tools
                        .iter()
                        .map(|tool| ToolDefinition {
                            name: tool.name().to_string(),
                            description: tool.description().to_string(),
                            parameters: tool.parameters_schema(),
                        })
                        .collect(),
                )
            } else {
                None
            };

            // Main agent loop with streaming
            for _turn in 0..config.max_turns {
                let request = CompletionRequest {
                    messages: messages.clone(),
                    model: agent.model.clone(),
                    tools: tools.clone(),
                    max_tokens: None,
                    temperature: None,
                };

                // Use actual streaming from model provider
                let mut stream = match model.stream(request).await {
                    Ok(s) => s,
                    Err(_e) => break,
                };

                let mut accumulated_content = String::new();
                let mut accumulated_tool_calls: Vec<(String, String, String)> = Vec::new(); // (id, name, args)

                // Stream chunks and emit events
                while let Some(chunk_result) = stream.next().await {
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(_) => break,
                    };

                    // Emit text deltas as raw response events
                    if let Some(delta) = &chunk.delta {
                        accumulated_content.push_str(delta);
                        let _ = tx.send(StreamEvent::RawResponse(RawResponseEvent {
                            data: delta.clone(),
                        }));
                    }

                    // Accumulate tool call deltas
                    for tc_delta in &chunk.tool_call_deltas {
                        // Ensure we have enough space in the vector
                        while accumulated_tool_calls.len() <= tc_delta.index {
                            accumulated_tool_calls.push((
                                String::new(),
                                String::new(),
                                String::new(),
                            ));
                        }

                        let (id, name, args) = &mut accumulated_tool_calls[tc_delta.index];
                        if let Some(ref delta_id) = tc_delta.id {
                            id.push_str(delta_id);
                        }
                        if let Some(ref delta_name) = tc_delta.name {
                            name.push_str(delta_name);
                        }
                        if let Some(ref delta_args) = tc_delta.arguments {
                            args.push_str(delta_args);
                        }
                    }

                    // Check for finish
                    if chunk.finish_reason.is_some() {
                        break;
                    }
                }

                // Check if we have a final output (no tool calls)
                if !accumulated_content.is_empty() && accumulated_tool_calls.is_empty() {
                    // Emit message output event
                    let _ = tx.send(StreamEvent::RunItem(RunItemStreamEvent {
                        name: RunItemEventName::MessageOutputCreated,
                        item: RunItem::MessageOutput {
                            content: accumulated_content.clone(),
                        },
                    }));

                    // Save conversation to session if available
                    if let Some(session) = &config.session {
                        let assistant_msg = Message {
                            role: "assistant".to_string(),
                            content: accumulated_content.clone(),
                        };
                        let _ = session
                            .add_items(vec![
                                serde_json::to_value(&messages[messages.len() - 1]).unwrap(),
                                serde_json::to_value(&assistant_msg).unwrap(),
                            ])
                            .await;
                    }

                    break;
                }

                // Add assistant message if we have content
                if !accumulated_content.is_empty() {
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: accumulated_content.clone(),
                    });
                }

                // Handle tool calls
                if !accumulated_tool_calls.is_empty() {
                    for (_id, name, args_str) in &accumulated_tool_calls {
                        if name.is_empty() {
                            continue;
                        }

                        let arguments =
                            serde_json::from_str(args_str).unwrap_or(serde_json::json!({}));

                        // Emit tool call event
                        let _ = tx.send(StreamEvent::RunItem(RunItemStreamEvent {
                            name: RunItemEventName::ToolCalled,
                            item: RunItem::ToolCall {
                                name: name.clone(),
                                arguments: arguments.clone(),
                            },
                        }));

                        // Find and execute the tool
                        if let Some(tool) = agent.tools.iter().find(|t| t.name() == name) {
                            if let Ok(result) = tool.execute(arguments.clone()).await {
                                let result_str = serde_json::to_string(&result).unwrap_or_default();

                                // Emit tool output event
                                let _ = tx.send(StreamEvent::RunItem(RunItemStreamEvent {
                                    name: RunItemEventName::ToolOutput,
                                    item: RunItem::ToolOutput {
                                        name: name.clone(),
                                        output: result_str.clone(),
                                    },
                                }));

                                // Add tool result as a message
                                messages.push(Message {
                                    role: "tool".to_string(),
                                    content: format!("Tool '{}' returned: {}", name, result_str),
                                });
                            }
                        }
                    }
                }
            }

            // Channel will close when tx is dropped
        });

        Ok(crate::streaming::StreamedRunResult::new(rx))
    }
}
