# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Rust port of OpenAI Agents Python SDK
- Core agent system with builder pattern
- Runner with complete execution loop
- Tool system with `#[function_tool]` macro
- Handoff mechanism for multi-agent workflows
- Guardrails (input, output, tool input, tool output)
- Session management with SQLite support
- OpenAI API integration via `async-openai`
- Comprehensive error handling with `thiserror`
- Examples: hello_world, tools, handoffs, sessions
- Unit tests for agent and error modules
- Documentation and README

### Planned
- Full tracing infrastructure
- Streaming support
- Redis session storage
- Enhanced function_tool macro with JSON schema generation
- Integration tests
- MCP (Model Context Protocol) support
- Voice features
- Realtime API support

## [0.1.0] - 2026-02-16

### Added
- Initial release with core functionality
- Basic agent execution
- Tool support
- Session management
- Multi-agent handoffs
