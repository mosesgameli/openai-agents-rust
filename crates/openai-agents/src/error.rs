//! Error types for the OpenAI Agents SDK

use thiserror::Error;

/// The main error type for the OpenAI Agents SDK
#[derive(Error, Debug)]
pub enum AgentError {
    /// Maximum number of turns exceeded
    #[error("Max turns exceeded: {0}")]
    MaxTurnsExceeded(usize),

    /// Input guardrail was triggered
    #[error("Input guardrail triggered: {0}")]
    InputGuardrailTriggered(String),

    /// Output guardrail was triggered
    #[error("Output guardrail triggered: {0}")]
    OutputGuardrailTriggered(String),

    /// Tool input guardrail was triggered
    #[error("Tool input guardrail triggered: {0}")]
    ToolInputGuardrailTriggered(String),

    /// Tool output guardrail was triggered
    #[error("Tool output guardrail triggered: {0}")]
    ToolOutputGuardrailTriggered(String),

    /// Tool execution failed
    #[error("Tool execution failed: {tool_name}: {reason}")]
    ToolExecutionFailed { tool_name: String, reason: String },

    /// Tool timeout
    #[error("Tool timeout: {0}")]
    ToolTimeout(String),

    /// Model API error
    #[error("Model error: {0}")]
    ModelError(String),

    /// Session/memory error
    #[error("Session error: {0}")]
    SessionError(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// User-provided error
    #[error("User error: {0}")]
    UserError(String),

    /// Model behavior error (unexpected response format)
    #[error("Model behavior error: {0}")]
    ModelBehaviorError(String),

    /// Generic error with context
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result type alias for the OpenAI Agents SDK
pub type Result<T> = std::result::Result<T, AgentError>;

impl AgentError {
    /// Create a new tool execution error
    pub fn tool_failed(tool_name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ToolExecutionFailed {
            tool_name: tool_name.into(),
            reason: reason.into(),
        }
    }

    /// Check if this error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            AgentError::ModelError(_) | AgentError::SessionError(_)
        )
    }
}

// Implement From for common error types
impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::SerializationError(err.to_string())
    }
}

impl From<async_openai::error::OpenAIError> for AgentError {
    fn from(err: async_openai::error::OpenAIError) -> Self {
        AgentError::ModelError(err.to_string())
    }
}

#[cfg(feature = "sqlite-session")]
impl From<sqlx::Error> for AgentError {
    fn from(err: sqlx::Error) -> Self {
        AgentError::SessionError(err.to_string())
    }
}

#[cfg(feature = "redis-session")]
impl From<redis::RedisError> for AgentError {
    fn from(err: redis::RedisError) -> Self {
        AgentError::SessionError(err.to_string())
    }
}
