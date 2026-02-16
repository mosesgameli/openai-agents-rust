//! Agent definition and builder
use schemars::JsonSchema;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use crate::{
    guardrail::{InputGuardrail, OutputGuardrail},
    handoff::Handoff,
    lifecycle::AgentHooks,
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

    /// Hooks for agent lifecycle events
    pub hooks: Vec<Arc<dyn AgentHooks>>,

    /// Expected output schema (if using structured outputs)
    pub output_schema: Option<serde_json::Value>,

    /// Name of the output schema
    pub output_name: Option<String>,
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
    hooks: Vec<Arc<dyn AgentHooks>>,
    output_schema: Option<serde_json::Value>,
    output_name: Option<String>,
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
            hooks: Vec::new(),
            output_schema: None,
            output_name: None,
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

    /// Add a lifecycle hook to the agent
    pub fn hook(mut self, hook: impl AgentHooks + 'static) -> Self {
        self.hooks.push(Arc::new(hook));
        self
    }

    /// Set the expected output type for structured outputs
    pub fn output_type<T: JsonSchema + DeserializeOwned + Serialize + 'static>(mut self) -> Self {
        let generator = schemars::r#gen::SchemaGenerator::default();
        let schema = generator.into_root_schema_for::<T>();

        // OpenAI expects a specific format for the JSON schema
        let mut schema_json = serde_json::to_value(schema).unwrap_or_default();

        // OpenAI requirement: all objects must have additionalProperties: false
        fn ensure_no_additional_properties(value: &mut serde_json::Value) {
            if let Some(obj) = value.as_object_mut() {
                if obj.contains_key("type") && obj["type"] == "object" {
                    obj.insert(
                        "additionalProperties".to_string(),
                        serde_json::Value::Bool(false),
                    );
                }
                for (_, v) in obj.iter_mut() {
                    ensure_no_additional_properties(v);
                }
            } else if let Some(arr) = value.as_array_mut() {
                for v in arr.iter_mut() {
                    ensure_no_additional_properties(v);
                }
            }
        }
        ensure_no_additional_properties(&mut schema_json);

        self.output_schema = Some(schema_json);

        // Sanitize type name for OpenAI: ^[a-zA-Z0-9_-]+$
        let type_full_name = std::any::type_name::<T>();
        let type_name = type_full_name.split("::").last().unwrap_or(type_full_name);
        let sanitized_name = type_name
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>();

        self.output_name = Some(sanitized_name);
        self
    }

    /// Build the agent
    pub fn build(self) -> Agent {
        Agent {
            name: self.name,
            instructions: self.instructions.unwrap_or_default(),
            model: self.model.unwrap_or_else(|| "gpt-4o-mini".to_string()),
            tools: self.tools,
            handoffs: self.handoffs,
            input_guardrails: self.input_guardrails,
            output_guardrails: self.output_guardrails,
            parallel_tool_calls: self.parallel_tool_calls,
            hooks: self.hooks,
            output_schema: self.output_schema,
            output_name: self.output_name,
        }
    }
}
