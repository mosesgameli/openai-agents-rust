//! Unit tests for error handling

use openai_agents::{AgentError, Result};

#[test]
fn test_error_creation() {
    let err = AgentError::tool_failed("test_tool", "test reason");
    assert!(matches!(err, AgentError::ToolExecutionFailed { .. }));
}

#[test]
fn test_error_display() {
    let err = AgentError::MaxTurnsExceeded(100);
    assert_eq!(err.to_string(), "Max turns exceeded: 100");
}

#[test]
fn test_error_is_retriable() {
    let model_err = AgentError::ModelError("test".to_string());
    assert!(model_err.is_retriable());

    let max_turns_err = AgentError::MaxTurnsExceeded(10);
    assert!(!max_turns_err.is_retriable());
}

#[test]
fn test_result_type() {
    fn returns_result() -> Result<String> {
        Ok("success".to_string())
    }

    let result = returns_result();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "success");
}
