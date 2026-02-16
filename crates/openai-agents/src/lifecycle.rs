//! Agent lifecycle hooks
//!
//! Hooks allow you to receive callbacks on various events during an agent run.
//! This is useful for logging, tracing, or modifying the standard execution flow.

use async_trait::async_trait;
use serde_json::Value;

use crate::agent::Agent;
use crate::models::{CompletionResponse, Message};

/// Hooks for events related to a specific agent
#[async_trait]
pub trait AgentHooks: Send + Sync {
    /// Called before the agent is invoked
    async fn on_start(&self, agent: &Agent) -> crate::error::Result<()> {
        let _ = agent;
        Ok(())
    }

    /// Called after the agent produces a final output
    async fn on_end(&self, agent: &Agent, output: &str) -> crate::error::Result<()> {
        let _ = agent;
        let _ = output;
        Ok(())
    }

    /// Called before the agent makes an LLM call
    async fn on_llm_start(&self, agent: &Agent, messages: &[Message]) -> crate::error::Result<()> {
        let _ = agent;
        let _ = messages;
        Ok(())
    }

    /// Called after the agent receives an LLM response
    async fn on_llm_end(
        &self,
        agent: &Agent,
        response: &CompletionResponse,
    ) -> crate::error::Result<()> {
        let _ = agent;
        let _ = response;
        Ok(())
    }

    /// Called before a tool is executed
    async fn on_tool_start(
        &self,
        agent: &Agent,
        tool_name: &str,
        arguments: &Value,
    ) -> crate::error::Result<()> {
        let _ = agent;
        let _ = tool_name;
        let _ = arguments;
        Ok(())
    }

    /// Called after a tool is executed
    async fn on_tool_end(
        &self,
        agent: &Agent,
        tool_name: &str,
        result: &Value,
    ) -> crate::error::Result<()> {
        let _ = agent;
        let _ = tool_name;
        let _ = result;
        Ok(())
    }

    /// Called when the agent hands off to another agent
    async fn on_handoff(&self, from_agent: &Agent, to_agent: &Agent) -> crate::error::Result<()> {
        let _ = from_agent;
        let _ = to_agent;
        Ok(())
    }
}

/// Global hooks for an entire runner session
#[async_trait]
pub trait RunHooks: Send + Sync {
    /// Called when any agent starts
    async fn on_agent_start(&self, agent: &Agent) -> crate::error::Result<()> {
        let _ = agent;
        Ok(())
    }

    /// Called when any agent ends
    async fn on_agent_end(&self, agent: &Agent, output: &str) -> crate::error::Result<()> {
        let _ = agent;
        let _ = output;
        Ok(())
    }

    /// Called when any handoff occurs
    async fn on_handoff(&self, from_agent: &Agent, to_agent: &Agent) -> crate::error::Result<()> {
        let _ = from_agent;
        let _ = to_agent;
        Ok(())
    }
}
