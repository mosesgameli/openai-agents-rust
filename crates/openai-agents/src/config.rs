//! Configuration management for the OpenAI Agents SDK

use std::sync::OnceLock;

use async_openai::{Client, config::OpenAIConfig};

static DEFAULT_CLIENT: OnceLock<Client<OpenAIConfig>> = OnceLock::new();
static DEFAULT_API_KEY: OnceLock<String> = OnceLock::new();

/// Set the default OpenAI API key
///
/// This is equivalent to Python's `set_default_openai_key()`
///
/// # Example
///
/// ```rust,no_run
/// use openai_agents::set_default_openai_key;
///
/// set_default_openai_key("sk-...");
/// ```
pub fn set_default_openai_key(key: impl Into<String>) {
    DEFAULT_API_KEY.set(key.into()).ok();
}

/// Get or create the default OpenAI client
///
/// Falls back to OPENAI_API_KEY environment variable if no key was set
pub fn get_default_client() -> Client<OpenAIConfig> {
    DEFAULT_CLIENT
        .get_or_init(|| {
            if let Some(key) = DEFAULT_API_KEY.get() {
                let config = OpenAIConfig::new().with_api_key(key);
                Client::with_config(config)
            } else {
                // Falls back to OPENAI_API_KEY env var
                Client::new()
            }
        })
        .clone()
}

/// Set a custom default OpenAI client
///
/// # Example
///
/// ```rust,no_run
/// use async_openai::{Client, config::OpenAIConfig};
/// use openai_agents::set_default_openai_client;
///
/// let config = OpenAIConfig::new()
///     .with_api_key("sk-...")
///     .with_org_id("org-...");
/// let client = Client::with_config(config);
/// set_default_openai_client(client);
/// ```
pub fn set_default_openai_client(client: Client<OpenAIConfig>) {
    DEFAULT_CLIENT.set(client).ok();
}
