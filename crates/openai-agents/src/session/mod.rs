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
pub use redis_impl::RedisSession;

/// In-memory session implementation (for testing and simple use cases)
pub struct InMemorySession {
    items: std::sync::Arc<tokio::sync::Mutex<Vec<Value>>>,
}

impl InMemorySession {
    /// Create a new in-memory session
    pub fn new() -> Self {
        Self {
            items: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }
}

impl Default for InMemorySession {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Session for InMemorySession {
    async fn get_items(&self, limit: Option<usize>) -> Result<Vec<Value>> {
        let items = self.items.lock().await;
        if let Some(n) = limit {
            let start = items.len().saturating_sub(n);
            Ok(items[start..].to_vec())
        } else {
            Ok(items.clone())
        }
    }

    async fn add_items(&self, new_items: Vec<Value>) -> Result<()> {
        let mut items = self.items.lock().await;
        items.extend(new_items);
        Ok(())
    }

    async fn pop_item(&self) -> Result<Option<Value>> {
        let mut items = self.items.lock().await;
        Ok(items.pop())
    }

    async fn clear_session(&self) -> Result<()> {
        let mut items = self.items.lock().await;
        items.clear();
        Ok(())
    }
}
