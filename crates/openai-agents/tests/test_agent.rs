//! Unit tests for the agent module

use openai_agents::{Agent, Handoff};

#[test]
fn test_agent_builder() {
    let agent = Agent::builder("Test Agent")
        .instructions("Test instructions")
        .model("gpt-4")
        .build();

    assert_eq!(agent.name, "Test Agent");
    assert_eq!(agent.instructions, "Test instructions");
    assert_eq!(agent.model, "gpt-4");
    assert!(agent.tools.is_empty());
    assert!(agent.handoffs.is_empty());
}

#[test]
fn test_agent_with_handoff() {
    let target_agent = Agent::builder("Target").build();
    let handoff = Handoff::new(target_agent);

    let agent = Agent::builder("Main")
        .handoff(handoff)
        .build();

    assert_eq!(agent.handoffs.len(), 1);
}

#[test]
fn test_agent_default_model() {
    let agent = Agent::builder("Test").build();
    assert_eq!(agent.model, "gpt-4");
}

#[test]
fn test_agent_parallel_tool_calls() {
    let agent = Agent::builder("Test")
        .parallel_tool_calls(false)
        .build();

    assert!(!agent.parallel_tool_calls);
}
