use sqlx::FromRow;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ToolSession {
    pub session_id: String,
    pub user_id: String,
    pub model_id: String,
    pub payload: String, // 序列化的 Value
    pub pending_tool_calls: String, // 序列化的 Vec<Value>
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

pub struct SessionRepo<'a> {
    db: &'a sqlx::SqlitePool,
}

impl<'a> SessionRepo<'a> {
    pub fn new(db: &'a sqlx::SqlitePool) -> Self {
        Self { db }
    }

    /// 创建或重置会话
    pub async fn save_session(
        &self,
        session_id: &str,
        user_id: &str,
        model_id: &str,
        payload: &Value,
        pending_tool_calls: &Vec<Value>
    ) -> anyhow::Result<()> {
        let payload_str = serde_json::to_string(payload)?;
        let tools_str = serde_json::to_string(pending_tool_calls)?;
        let expires = Utc::now() + Duration::hours(1);

        sqlx::query(
            "INSERT INTO tool_confirm_sessions (session_id, user_id, model_id, payload, pending_tool_calls, expires_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(session_id) DO UPDATE SET 
                payload = EXCLUDED.payload, 
                pending_tool_calls = EXCLUDED.pending_tool_calls,
                expires_at = EXCLUDED.expires_at"
        )
        .bind(session_id)
        .bind(user_id)
        .bind(model_id)
        .bind(payload_str)
        .bind(tools_str)
        .bind(expires)
        .execute(self.db).await?;

        Ok(())
    }

    /// 加载生效内的会话
    pub async fn load_session(&self, session_id: &str) -> anyhow::Result<Option<ToolSession>> {
        let session = sqlx::query_as::<_, ToolSession>(
            "SELECT * FROM tool_confirm_sessions WHERE session_id = ? AND expires_at > ?"
        )
        .bind(session_id)
        .bind(Utc::now())
        .fetch_optional(self.db).await?;
        
        Ok(session)
    }

    /// 删除会话 (完成后清理)
    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM tool_confirm_sessions WHERE session_id = ?")
            .bind(session_id)
            .execute(self.db).await?;
        Ok(())
    }
}
