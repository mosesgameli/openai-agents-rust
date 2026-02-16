//! Tool trait and implementations

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;

/// Trait for tools that can be used by agents
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the name of the tool
    fn name(&self) -> &str;

    /// Get the description of the tool
    fn description(&self) -> &str;

    /// Get the JSON schema for the tool's parameters
    fn parameters_schema(&self) -> Value;

    /// Execute the tool with the given arguments
    async fn execute(&self, args: Value) -> Result<Value>;
}

/// A function-based tool implementation
pub struct FunctionTool {
    name: String,
    description: String,
    schema: Value,
    // TODO: Add function pointer/closure for execution
}

impl FunctionTool {
    /// Create a new function tool
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        schema: Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            schema,
        }
    }
}

#[async_trait]
impl Tool for FunctionTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn parameters_schema(&self) -> Value {
        self.schema.clone()
    }

    async fn execute(&self, _args: Value) -> Result<Value> {
        // TODO: Implement execution
        Ok(Value::Null)
    }
}
