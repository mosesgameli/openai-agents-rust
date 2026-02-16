//! Handoff mechanism for transferring control between agents

use std::sync::Arc;

use crate::agent::Agent;

/// A handoff to another agent
#[derive(Clone)]
pub struct Handoff {
    /// The target agent to hand off to
    pub target_agent: Arc<Agent>,
    /// Optional description of when to use this handoff
    pub description: Option<String>,
}

impl Handoff {
    /// Create a new handoff to the given agent
    pub fn new(agent: Agent) -> Self {
        Self {
            target_agent: Arc::new(agent),
            description: None,
        }
    }

    /// Set the description for this handoff
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}
