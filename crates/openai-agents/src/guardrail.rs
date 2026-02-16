//! Guardrail system for input and output validation

use async_trait::async_trait;

use crate::error::Result;

/// Result of a guardrail check
#[derive(Debug, Clone)]
pub enum GuardrailResult {
    /// Allow the content to pass through
    Allow,
    /// Block the content with a reason
    Block { reason: String },
    /// Modify the content
    Modify { new_content: String },
}

/// Trait for input guardrails
#[async_trait]
pub trait InputGuardrail: Send + Sync {
    /// Check the input and return a guardrail result
    async fn check(&self, input: &str) -> Result<GuardrailResult>;
}

/// Trait for output guardrails
#[async_trait]
pub trait OutputGuardrail: Send + Sync {
    /// Check the output and return a guardrail result
    async fn check(&self, output: &str) -> Result<GuardrailResult>;
}

/// Trait for tool input guardrails
#[async_trait]
pub trait ToolInputGuardrail: Send + Sync {
    /// Check the tool input
    async fn check(&self, tool_name: &str, input: &serde_json::Value) -> Result<GuardrailResult>;
}

/// Trait for tool output guardrails
#[async_trait]
pub trait ToolOutputGuardrail: Send + Sync {
    /// Check the tool output
    async fn check(&self, tool_name: &str, output: &serde_json::Value) -> Result<GuardrailResult>;
}
