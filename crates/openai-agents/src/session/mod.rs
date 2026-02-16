//! Session management for conversation history

use async_trait::async_trait;
use serde_json::Value;

use crate::error::Result;

/// Trait for session storage
#[async_trait]
pub trait Session: Send + Sync {
    /// Get items from the session
    async fn get_items(&self, limit: Option<usize>) -> Result<Vec<Value>>;

    /// Add items to the session
    async fn add_items(&self, items: Vec<Value>) -> Result<()>;

    /// Remove and return the most recent item
    async fn pop_item(&self) -> Result<Option<Value>>;

    /// Clear all items from the session
    async fn clear_session(&self) -> Result<()>;
}

/// Session settings
#[derive(Debug, Clone)]
pub struct SessionSettings {
    /// Maximum number of items to keep in session
    pub max_items: Option<usize>,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self { max_items: None }
    }
}

#[cfg(feature = "sqlite-session")]
pub mod sqlite;

#[cfg(feature = "sqlite-session")]
pub use sqlite::SqliteSession;

#[cfg(feature = "redis-session")]
pub mod redis_impl;

#[cfg(feature = "redis-session")]
pub use redis_impl::RedisSession;
