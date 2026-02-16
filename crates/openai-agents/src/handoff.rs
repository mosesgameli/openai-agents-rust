use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::agent::Agent;
use crate::error::Result;
use crate::tool::Tool;

/// A handoff to another agent
#[derive(Clone)]
pub struct Handoff {
    /// The target agent to hand off to
    pub target_agent: Arc<Agent>,
    /// Optional description of when to use this handoff
    pub description: Option<String>,
    /// The name of the tool representing this handoff
    pub name: String,
}

impl Handoff {
    /// Create a new handoff to the given agent
    pub fn new(agent: Agent) -> Self {
        let name = format!(
            "transfer_to_{}",
            agent.name.to_lowercase().replace(' ', "_")
        );
        Self {
            target_agent: Arc::new(agent),
            description: None,
            name,
        }
    }

    /// Set the description for this handoff
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the name for this handoff tool
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

#[async_trait]
impl Tool for Handoff {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        self.description
            .as_deref()
            .unwrap_or("Handoff to another agent")
    }

    fn parameters_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {},
            "additionalProperties": false
        })
    }

    async fn execute(&self, _args: Value) -> Result<Value> {
        Ok(json!({
            "assistant": self.target_agent.name
        }))
    }
}
