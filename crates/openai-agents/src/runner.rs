//! Runner for executing agents

use std::sync::Arc;

use crate::{
    agent::Agent,
    error::{AgentError, Result},
    models::{CompletionRequest, Message, ModelProvider, OpenAIResponsesModel, ToolDefinition},
    result::RunResult,
    session::Session,
    tool::Tool,
};

/// Configuration for running an agent
#[derive(Clone)]
pub struct RunConfig {
    /// Maximum number of turns before stopping
    pub max_turns: usize,
    /// Optional session for conversation history
    pub session: Option<Arc<dyn Session>>,
    /// Optional model provider override (useful for testing)
    pub model_override: Option<Arc<dyn ModelProvider>>,
    /// Global lifecycle hooks for the run
    pub run_hooks: Vec<Arc<dyn crate::lifecycle::RunHooks>>,
}

impl Default for RunConfig {
    fn default() -> Self {
        Self {
            max_turns: 100,
            session: None,
            model_override: None,
            run_hooks: Vec::new(),
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

        // Initialize model provider
        let model: Arc<dyn ModelProvider> = if let Some(m) = config.model_override.clone() {
            m
        } else {
            Arc::new(OpenAIResponsesModel::new())
        };

        // Convert agent tools and handoffs to tool definitions
        let mut tool_definitions = Vec::new();

        for tool in &agent.tools {
            tool_definitions.push(ToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                parameters: tool.parameters_schema(),
            });
        }

        for handoff in &agent.handoffs {
            tool_definitions.push(ToolDefinition {
                name: handoff.name().to_string(),
                description: handoff.description().to_string(),
                parameters: handoff.parameters_schema(),
            });
        }

        let tools = if !tool_definitions.is_empty() {
            Some(tool_definitions)
        } else {
            None
        };

        // We use Arc to manage the current agent so we can easily swap it during handoffs
        let mut current_agent: Arc<Agent> = Arc::new(agent.clone());
        let mut current_tools = tools;

        // Main agent loop
        for turn in 0..config.max_turns {
            // Trigger on_agent_start and on_start hooks
            for hook in &config.run_hooks {
                hook.on_agent_start(&current_agent).await?;
            }
            for hook in &current_agent.hooks {
                hook.on_start(&current_agent).await?;
            }

            let request = CompletionRequest {
                messages: messages.clone(),
                model: current_agent.model.clone(),
                tools: current_tools.clone(),
                max_tokens: None,
                temperature: None,
                response_format: current_agent.output_schema.as_ref().map(|schema| {
                    crate::models::ResponseFormat::JsonSchema {
                        json_schema: crate::models::JsonSchemaFormat {
                            name: current_agent
                                .output_name
                                .clone()
                                .unwrap_or_else(|| "output".to_string()),
                            description: None,
                            schema: schema.clone(),
                            strict: Some(true),
                        },
                    }
                }),
            };

            // Trigger on_llm_start hooks
            for hook in &current_agent.hooks {
                hook.on_llm_start(&current_agent, &messages).await?;
            }

            let response = model.complete(request).await?;

            // Trigger on_llm_end hooks
            for hook in &current_agent.hooks {
                hook.on_llm_end(&current_agent, &response).await?;
            }

            // Check if we have a final output (no tool calls)
            if let Some(content) = &response.content {
                if response.tool_calls.is_empty() {
                    // Trigger on_agent_end and on_end hooks
                    for hook in &config.run_hooks {
                        hook.on_agent_end(&current_agent, content).await?;
                    }
                    for hook in &current_agent.hooks {
                        hook.on_end(&current_agent, content).await?;
                    }

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
                    // Trigger on_tool_start hooks
                    for hook in &current_agent.hooks {
                        hook.on_tool_start(&current_agent, &tool_call.name, &tool_call.arguments)
                            .await?;
                    }

                    // Check if it's a regular tool or a handoff
                    let mut tool_result = None;
                    let mut handed_off_to = None;

                    // Search in agent tools
                    if let Some(tool) = current_agent
                        .tools
                        .iter()
                        .find(|t| t.name() == tool_call.name)
                    {
                        let result = tool
                            .execute(tool_call.arguments.clone())
                            .await
                            .map_err(|e| AgentError::tool_failed(&tool_call.name, e.to_string()))?;
                        tool_result = Some(result);
                    } else if let Some(handoff) = current_agent
                        .handoffs
                        .iter()
                        .find(|h| h.name() == tool_call.name)
                    {
                        let result = handoff
                            .execute(tool_call.arguments.clone())
                            .await
                            .map_err(|e| AgentError::tool_failed(&tool_call.name, e.to_string()))?;

                        if let Some(_target_name) = result.get("assistant").and_then(|v| v.as_str())
                        {
                            handed_off_to = Some(handoff.target_agent.clone());
                        }
                        tool_result = Some(result);
                    }

                    let result = tool_result.ok_or_else(|| {
                        AgentError::tool_failed(
                            &tool_call.name,
                            format!("Tool '{}' not found", tool_call.name),
                        )
                    })?;

                    // Trigger on_tool_end hooks
                    for hook in &current_agent.hooks {
                        hook.on_tool_end(&current_agent, &tool_call.name, &result)
                            .await?;
                    }

                    // Add tool result as a message
                    let result_str = serde_json::to_string(&result)?;
                    messages.push(Message {
                        role: "tool".to_string(),
                        content: format!("Tool '{}' returned: {}", tool_call.name, result_str),
                    });

                    // If we handed off, update current agent and rebuild tools for next turn
                    if let Some(new_agent) = handed_off_to {
                        // Trigger on_handoff hooks
                        for hook in &config.run_hooks {
                            hook.on_handoff(&current_agent, &new_agent).await?;
                        }
                        for hook in &current_agent.hooks {
                            hook.on_handoff(&current_agent, &new_agent).await?;
                        }

                        current_agent = new_agent;

                        // Synchronize system message with the new agent's instructions
                        if !messages.is_empty() && messages[0].role == "system" {
                            messages[0].content = current_agent.instructions.clone();
                        } else if !current_agent.instructions.is_empty() {
                            messages.insert(
                                0,
                                Message {
                                    role: "system".to_string(),
                                    content: current_agent.instructions.clone(),
                                },
                            );
                        }

                        // Rebuild tools for the new agent
                        let mut next_tool_definitions = Vec::new();
                        for tool in &current_agent.tools {
                            next_tool_definitions.push(ToolDefinition {
                                name: tool.name().to_string(),
                                description: tool.description().to_string(),
                                parameters: tool.parameters_schema(),
                            });
                        }
                        for handoff in &current_agent.handoffs {
                            next_tool_definitions.push(ToolDefinition {
                                name: handoff.name().to_string(),
                                description: handoff.description().to_string(),
                                parameters: handoff.parameters_schema(),
                            });
                        }
                        current_tools = if !next_tool_definitions.is_empty() {
                            Some(next_tool_definitions)
                        } else {
                            None
                        };
                    }
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
    /// use futures::StreamExt;
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
        let (streamed_result, shared_state) = crate::streaming::StreamedRunResult::new(rx);
        let input = input.into();

        // Initialize model provider
        let model: Arc<dyn ModelProvider> = if let Some(m) = config.model_override.clone() {
            m
        } else {
            Arc::new(OpenAIResponsesModel::new())
        };

        // Spawn a task to run the agent and emit events
        let agent = agent.clone();
        let shared_state_bg = shared_state.clone();
        tokio::spawn(async move {
            let mut messages = Vec::new();
            let mut final_output = String::new();

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
                content: input.to_string(),
            });

            // Initial current agent and tools
            let mut current_agent = Arc::new(agent.clone());

            // Convert agent tools and handoffs to tool definitions
            let mut tool_definitions = Vec::new();
            for tool in &current_agent.tools {
                tool_definitions.push(ToolDefinition {
                    name: tool.name().to_string(),
                    description: tool.description().to_string(),
                    parameters: tool.parameters_schema(),
                });
            }
            for handoff in &current_agent.handoffs {
                tool_definitions.push(ToolDefinition {
                    name: handoff.name().to_string(),
                    description: handoff.description().to_string(),
                    parameters: handoff.parameters_schema(),
                });
            }

            let mut current_tools = if !tool_definitions.is_empty() {
                Some(tool_definitions)
            } else {
                None
            };

            // Main agent loop with streaming
            for _turn in 0..config.max_turns {
                // Trigger on_agent_start and on_start hooks
                for hook in &config.run_hooks {
                    let _ = hook.on_agent_start(&current_agent).await;
                }
                for hook in &current_agent.hooks {
                    let _ = hook.on_start(&current_agent).await;
                }

                let request = CompletionRequest {
                    messages: messages.clone(),
                    model: current_agent.model.clone(),
                    tools: current_tools.clone(),
                    max_tokens: None,
                    temperature: None,
                    response_format: current_agent.output_schema.as_ref().map(|schema| {
                        crate::models::ResponseFormat::JsonSchema {
                            json_schema: crate::models::JsonSchemaFormat {
                                name: current_agent
                                    .output_name
                                    .clone()
                                    .unwrap_or_else(|| "output".to_string()),
                                description: None,
                                schema: schema.clone(),
                                strict: Some(true),
                            },
                        }
                    }),
                };

                // Trigger on_llm_start hooks
                for hook in &current_agent.hooks {
                    let _ = hook.on_llm_start(&current_agent, &messages).await;
                }

                // Use actual streaming from model provider
                let mut stream = match model.stream(request).await {
                    Ok(s) => s,
                    Err(e) => {
                        let _ = tx.send(StreamEvent::RawResponse(RawResponseEvent {
                            data: format!("Error: {}", e),
                        }));
                        break;
                    }
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
                            data: delta.to_string(),
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

                // Add assistant message if we have content
                if !accumulated_content.is_empty() {
                    final_output = accumulated_content.clone();
                    messages.push(Message {
                        role: "assistant".to_string(),
                        content: accumulated_content.clone(),
                    });

                    // Emit message output event
                    let _ = tx.send(StreamEvent::RunItem(RunItemStreamEvent {
                        name: RunItemEventName::MessageOutputCreated,
                        item: RunItem::MessageOutput {
                            content: accumulated_content.clone(),
                        },
                    }));
                }

                // Handle tool calls
                if !accumulated_tool_calls.is_empty() {
                    let mut handed_off = false;
                    for (_id, name, args_str) in accumulated_tool_calls {
                        if name.is_empty() {
                            continue;
                        }

                        let arguments =
                            serde_json::from_str(&args_str).unwrap_or(serde_json::json!({}));

                        // Trigger on_tool_start hooks
                        for hook in &current_agent.hooks {
                            let _ = hook.on_tool_start(&current_agent, &name, &arguments).await;
                        }

                        // Emit tool call event
                        let _ = tx.send(StreamEvent::RunItem(RunItemStreamEvent {
                            name: RunItemEventName::ToolCalled,
                            item: RunItem::ToolCall {
                                name: name.clone(),
                                arguments: arguments.clone(),
                            },
                        }));

                        let mut tool_result = None;
                        let mut handed_off_to = None;

                        // Search in agent tools
                        if let Some(tool) = current_agent.tools.iter().find(|t| t.name() == name) {
                            if let Ok(result) = tool.execute(arguments.clone()).await {
                                tool_result = Some(result);
                            }
                        } else if let Some(handoff) =
                            current_agent.handoffs.iter().find(|h| h.name() == name)
                        {
                            if let Ok(result) = handoff.execute(arguments.clone()).await {
                                if let Some(_target_name) =
                                    result.get("assistant").and_then(|v| v.as_str())
                                {
                                    handed_off_to = Some(handoff.target_agent.clone());
                                }
                                tool_result = Some(result);
                            }
                        }

                        if let Some(result) = tool_result {
                            // Trigger on_tool_end hooks
                            for hook in &current_agent.hooks {
                                let _ = hook.on_tool_end(&current_agent, &name, &result).await;
                            }

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

                            // If we handed off, update current agent and rebuild tools
                            if let Some(new_agent) = handed_off_to {
                                // Trigger on_handoff hooks
                                for hook in &config.run_hooks {
                                    let _ = hook.on_handoff(&current_agent, &new_agent).await;
                                }
                                for hook in &current_agent.hooks {
                                    let _ = hook.on_handoff(&current_agent, &new_agent).await;
                                }

                                current_agent = new_agent;
                                handed_off = true;

                                // Emit agent updated event
                                use crate::stream_events::AgentUpdatedEvent;
                                let _ = tx.send(StreamEvent::AgentUpdated(AgentUpdatedEvent {
                                    new_agent: (*current_agent).clone(),
                                }));

                                // Synchronize system message
                                if !messages.is_empty() && messages[0].role == "system" {
                                    messages[0].content = current_agent.instructions.clone();
                                } else if !current_agent.instructions.is_empty() {
                                    messages.insert(
                                        0,
                                        Message {
                                            role: "system".to_string(),
                                            content: current_agent.instructions.clone(),
                                        },
                                    );
                                }

                                // Rebuild tools
                                let mut next_tool_definitions = Vec::new();
                                for tool in &current_agent.tools {
                                    next_tool_definitions.push(ToolDefinition {
                                        name: tool.name().to_string(),
                                        description: tool.description().to_string(),
                                        parameters: tool.parameters_schema(),
                                    });
                                }
                                for handoff in &current_agent.handoffs {
                                    next_tool_definitions.push(ToolDefinition {
                                        name: handoff.name().to_string(),
                                        description: handoff.description().to_string(),
                                        parameters: handoff.parameters_schema(),
                                    });
                                }
                                current_tools = if !next_tool_definitions.is_empty() {
                                    Some(next_tool_definitions)
                                } else {
                                    None
                                };
                            }
                        }
                    }

                    if !handed_off {
                        // If we had tool calls but no handoff, and the model might want to say more,
                        // we let the loop continue to the next OpenAI turn.
                    }
                } else {
                    // Trigger on_agent_end and on_end hooks
                    for hook in &config.run_hooks {
                        let _ = hook
                            .on_agent_end(&current_agent, &accumulated_content)
                            .await;
                    }
                    for hook in &current_agent.hooks {
                        let _ = hook.on_end(&current_agent, &accumulated_content).await;
                    }

                    // No tool calls and we have content (or already handled content)
                    // This is a final response
                    break;
                }
            }

            // Set final result
            let mut state = shared_state_bg.lock().unwrap();
            state.final_result = Some(RunResult::new(final_output));
        });

        Ok(streamed_result)
    }
}
