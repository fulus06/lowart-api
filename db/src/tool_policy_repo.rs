use sqlx::FromRow;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ToolPolicy {
    pub id: i64,
    pub tool_name: String,
    pub policy: String, // auto, confirm, block
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ToolPolicyRepo<'a> {
    db: &'a sqlx::SqlitePool,
}

impl<'a> ToolPolicyRepo<'a> {
    pub fn new(db: &'a sqlx::SqlitePool) -> Self {
        Self { db }
    }

    /// 获取工具的生效策略
    /// 优先级: 用户特定策略 > 全局策略 > 默认 (auto)
    pub async fn get_policy(&self, tool_name: &str, user_id: Option<&str>) -> anyhow::Result<String> {
        let mut policy = "auto".to_string();

        // 1. 尝试获取全局策略
        if let Some(row) = sqlx::query_as::<_, ToolPolicy>("SELECT * FROM tool_policies WHERE tool_name = ? AND user_id IS NULL")
            .bind(tool_name)
            .fetch_optional(self.db).await? {
            policy = row.policy;
        }

        // 2. 尝试获取用户特定策略 (覆盖全局)
        if let Some(uid) = user_id {
            if let Some(row) = sqlx::query_as::<_, ToolPolicy>("SELECT * FROM tool_policies WHERE tool_name = ? AND user_id = ?")
                .bind(tool_name)
                .bind(uid)
                .fetch_optional(self.db).await? {
                policy = row.policy;
            }
        }

        Ok(policy)
    }

    pub async fn upsert_policy(&self, tool_name: &str, user_id: Option<&str>, policy: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO tool_policies (tool_name, user_id, policy) VALUES (?, ?, ?)
             ON CONFLICT(tool_name, user_id) DO UPDATE SET policy = EXCLUDED.policy, updated_at = CURRENT_TIMESTAMP"
        )
        .bind(tool_name)
        .bind(user_id)
        .bind(policy)
        .execute(self.db).await?;
        Ok(())
    }
}
