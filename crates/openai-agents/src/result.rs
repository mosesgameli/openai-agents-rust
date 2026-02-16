//! Result types for agent runs

/// Result of running an agent
#[derive(Debug, Clone)]
pub struct RunResult {
    final_output: String,
    // TODO: Add more fields (usage, traces, etc.)
}

impl RunResult {
    /// Create a new run result
    pub fn new(final_output: impl Into<String>) -> Self {
        Self {
            final_output: final_output.into(),
        }
    }

    /// Get the final output from the agent
    pub fn final_output(&self) -> &str {
        &self.final_output
    }
}

/// Streaming result of running an agent
pub struct RunResultStreaming {
    // TODO: Implement streaming result
}
