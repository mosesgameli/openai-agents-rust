//! Agent definition and builder

use std::sync::Arc;

use crate::{
    guardrail::{InputGuardrail, OutputGuardrail},
    handoff::Handoff,
    tool::Tool,
};

/// An agent that can process inputs and produce outputs
#[derive(Clone)]
pub struct Agent {
    /// Name of the agent
    pub name: String,

    /// System instructions for the agent
    pub instructions: String,

    /// Model to use (e.g., "gpt-4", "gpt-3.5-turbo")
    pub model: String,

    /// Tools available to the agent
    pub tools: Vec<Arc<dyn Tool>>,

    /// Agents this agent can hand off to
    pub handoffs: Vec<Handoff>,

    /// Input guardrails
    pub input_guardrails: Vec<Arc<dyn InputGuardrail>>,

    /// Output guardrails
    pub output_guardrails: Vec<Arc<dyn OutputGuardrail>>,

    /// Whether to allow parallel tool calls
    pub parallel_tool_calls: bool,
}

impl Agent {
    /// Create a new agent builder
    pub fn builder(name: impl Into<String>) -> AgentBuilder {
        AgentBuilder::new(name)
    }
}

/// Builder for creating agents
pub struct AgentBuilder {
    name: String,
    instructions: Option<String>,
    model: Option<String>,
    tools: Vec<Arc<dyn Tool>>,
    handoffs: Vec<Handoff>,
    input_guardrails: Vec<Arc<dyn InputGuardrail>>,
    output_guardrails: Vec<Arc<dyn OutputGuardrail>>,
    parallel_tool_calls: bool,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            instructions: None,
            model: None,
            tools: Vec::new(),
            handoffs: Vec::new(),
            input_guardrails: Vec::new(),
            output_guardrails: Vec::new(),
            parallel_tool_calls: true,
        }
    }

    /// Set the agent's instructions
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set the model to use
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Add a tool to the agent
    pub fn tool(mut self, tool: impl Tool + 'static) -> Self {
        self.tools.push(Arc::new(tool));
        self
    }

    /// Add a handoff to another agent
    pub fn handoff(mut self, handoff: Handoff) -> Self {
        self.handoffs.push(handoff);
        self
    }

    /// Add an input guardrail
    pub fn input_guardrail(mut self, guardrail: impl InputGuardrail + 'static) -> Self {
        self.input_guardrails.push(Arc::new(guardrail));
        self
    }

    /// Add an output guardrail
    pub fn output_guardrail(mut self, guardrail: impl OutputGuardrail + 'static) -> Self {
        self.output_guardrails.push(Arc::new(guardrail));
        self
    }

    /// Set whether to allow parallel tool calls
    pub fn parallel_tool_calls(mut self, parallel: bool) -> Self {
        self.parallel_tool_calls = parallel;
        self
    }

    /// Build the agent
    pub fn build(self) -> Agent {
        Agent {
            name: self.name,
            instructions: self.instructions.unwrap_or_default(),
            model: self.model.unwrap_or_else(|| "gpt-4".to_string()),
            tools: self.tools,
            handoffs: self.handoffs,
            input_guardrails: self.input_guardrails,
            output_guardrails: self.output_guardrails,
            parallel_tool_calls: self.parallel_tool_calls,
        }
    }
}
