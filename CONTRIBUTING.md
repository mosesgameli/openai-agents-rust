# Contributing to OpenAI Agents Rust

Thank you for your interest in contributing to the OpenAI Agents Rust SDK!

## Development Setup

1. **Install Rust** (1.75 or newer):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/openai-agents-rust
   cd openai-agents-rust
   ```

3. **Build the project**:
   ```bash
   cargo build
   ```

4. **Run tests**:
   ```bash
   cargo test
   ```

## Project Structure

- `crates/openai-agents/` - Main library crate
- `crates/openai-agents-macros/` - Procedural macros
- `examples/` - Example programs
- `tests/` - Integration tests

## Running Examples

Set your OpenAI API key:
```bash
export OPENAI_API_KEY=sk-...
```

Run an example:
```bash
cargo run --example hello_world
cargo run --example handoffs
cargo run --example sessions
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_agent_builder

# Run tests with features
cargo test --all-features
```

## Code Style

We use `rustfmt` and `clippy`:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## Pull Requests

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run tests and linting
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## Guidelines

- Write clear, concise commit messages
- Add tests for new features
- Update documentation as needed
- Follow Rust naming conventions
- Keep PRs focused on a single feature/fix

## Questions?

Open an issue or start a discussion!
