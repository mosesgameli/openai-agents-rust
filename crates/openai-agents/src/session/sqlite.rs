//! SQLite session implementation

use async_trait::async_trait;
use serde_json::Value;
use sqlx::{sqlite::SqlitePool, Row};

use crate::error::{AgentError, Result};

use super::Session;

/// SQLite-based session storage
pub struct SqliteSession {
    session_id: String,
    pool: SqlitePool,
}

impl SqliteSession {
    /// Create a new SQLite session
    pub async fn new(session_id: impl Into<String>, db_path: impl AsRef<str>) -> Result<Self> {
        let pool = SqlitePool::connect(db_path.as_ref())
            .await
            .map_err(|e| AgentError::SessionError(e.to_string()))?;

        // Create table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT NOT NULL,
                item_index INTEGER NOT NULL,
                item_data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (session_id, item_index)
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            session_id: session_id.into(),
            pool,
        })
    }
}

#[async_trait]
impl Session for SqliteSession {
    async fn get_items(&self, limit: Option<usize>) -> Result<Vec<Value>> {
        let query = if let Some(limit) = limit {
            format!(
                "SELECT item_data FROM sessions WHERE session_id = ? ORDER BY item_index DESC LIMIT {}",
                limit
            )
        } else {
            "SELECT item_data FROM sessions WHERE session_id = ? ORDER BY item_index ASC"
                .to_string()
        };

        let rows = sqlx::query(&query)
            .bind(&self.session_id)
            .fetch_all(&self.pool)
            .await?;

        let mut items = Vec::new();
        for row in rows {
            let data: String = row.try_get("item_data")?;
            let value: Value = serde_json::from_str(&data)?;
            items.push(value);
        }

        Ok(items)
    }

    async fn add_items(&self, items: Vec<Value>) -> Result<()> {
        for item in items {
            let data = serde_json::to_string(&item)?;

            // Get the next index
            let next_index: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(item_index), -1) + 1 FROM sessions WHERE session_id = ?",
            )
            .bind(&self.session_id)
            .fetch_one(&self.pool)
            .await?;

            sqlx::query("INSERT INTO sessions (session_id, item_index, item_data) VALUES (?, ?, ?)")
                .bind(&self.session_id)
                .bind(next_index)
                .bind(data)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    async fn pop_item(&self) -> Result<Option<Value>> {
        let row: Option<(i64, String)> = sqlx::query_as(
            "SELECT item_index, item_data FROM sessions WHERE session_id = ? ORDER BY item_index DESC LIMIT 1"
        )
        .bind(&self.session_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((index, data)) = row {
            sqlx::query("DELETE FROM sessions WHERE session_id = ? AND item_index = ?")
                .bind(&self.session_id)
                .bind(index)
                .execute(&self.pool)
                .await?;

            let value: Value = serde_json::from_str(&data)?;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    async fn clear_session(&self) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE session_id = ?")
            .bind(&self.session_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
