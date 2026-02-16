//! Result types for agent runs

/// Result of running an agent
#[derive(Debug, Clone)]
pub struct RunResult {
    final_output: String,
    structured_output: Option<serde_json::Value>,
    // TODO: Add more fields (usage, traces, etc.)
}

impl RunResult {
    /// Create a new run result
    pub fn new(final_output: impl Into<String>) -> Self {
        Self {
            final_output: final_output.into(),
            structured_output: None,
        }
    }

    /// Create a new run result with structured output
    pub fn with_structured(final_output: impl Into<String>, structured: serde_json::Value) -> Self {
        Self {
            final_output: final_output.into(),
            structured_output: Some(structured),
        }
    }

    /// Get the final output from the agent
    pub fn final_output(&self) -> &str {
        &self.final_output
    }

    /// Get the structured output if available
    pub fn structured_output(&self) -> Option<&serde_json::Value> {
        self.structured_output.as_ref()
    }

    /// Parse the final output as a specific type
    pub fn final_output_as<T: serde::de::DeserializeOwned>(&self) -> crate::error::Result<T> {
        if let Some(structured) = &self.structured_output {
            serde_json::from_value(structured.clone())
                .map_err(|e| crate::error::AgentError::ModelBehaviorError(e.to_string()))
        } else {
            serde_json::from_str(&self.final_output)
                .map_err(|e| crate::error::AgentError::ModelBehaviorError(e.to_string()))
        }
    }
}

/// Streaming result of running an agent
pub struct RunResultStreaming {
    // TODO: Implement streaming result
}
