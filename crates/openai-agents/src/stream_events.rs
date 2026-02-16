use crate::agent::Agent;
use serde::{Deserialize, Serialize};

/// Types of run items that can be streamed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunItem {
    /// A message output from the agent
    MessageOutput {
        /// The message content
        content: String,
    },
    /// A tool call request
    ToolCall {
        /// Tool name
        name: String,
        /// Tool arguments as JSON
        arguments: serde_json::Value,
    },
    /// A tool call output/result
    ToolOutput {
        /// Tool name
        name: String,
        /// Tool output
        output: String,
    },
    /// A handoff request to another agent
    HandoffRequested {
        /// Target agent name
        agent_name: String,
    },
    /// A handoff occurred
    HandoffOccurred {
        /// Target agent name
        agent_name: String,
    },
}

/// Raw streaming event from the LLM
#[derive(Debug, Clone)]
pub struct RawResponseEvent {
    /// The raw event data (text delta, etc.)
    pub data: String,
}

/// Streaming event that wraps a RunItem
#[derive(Debug, Clone)]
pub struct RunItemStreamEvent {
    /// The name of the event
    pub name: RunItemEventName,
    /// The item that was created
    pub item: RunItem,
}

/// Names for run item stream events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunItemEventName {
    /// A message output was created
    MessageOutputCreated,
    /// A handoff was requested
    HandoffRequested,
    /// A handoff occurred
    HandoffOccurred,
    /// A tool was called
    ToolCalled,
    /// A tool output was received
    ToolOutput,
}

/// Event that notifies a new agent is running
#[derive(Clone)]
pub struct AgentUpdatedEvent {
    /// The new agent
    pub new_agent: Agent,
}

/// A streaming event from an agent
#[derive(Clone)]
pub enum StreamEvent {
    /// Raw response event from the LLM
    RawResponse(RawResponseEvent),
    /// Run item event
    RunItem(RunItemStreamEvent),
    /// Agent updated event
    AgentUpdated(AgentUpdatedEvent),
}

impl StreamEvent {
    /// Get the event type as a string
    pub fn event_type(&self) -> &'static str {
        match self {
            StreamEvent::RawResponse(_) => "raw_response_event",
            StreamEvent::RunItem(_) => "run_item_stream_event",
            StreamEvent::AgentUpdated(_) => "agent_updated_stream_event",
        }
    }
}
