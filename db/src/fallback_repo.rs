use crate::connection::DbConnection;
use utils::Result;
use sqlx::{sqlite::SqliteRow, Row};

/// 降级配置项
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct FallbackConfig {
    pub primary_model_id: String,
    pub fallback_model_id: String,
    pub priority: i32,
    pub trigger_condition: String,
}

/// 降级配置仓库
pub struct FallbackRepo<'a> {
    db: &'a DbConnection,
}

impl<'a> FallbackRepo<'a> {
    pub fn new(db: &'a DbConnection) -> Self {
        Self { db }
    }

    /// 获取指定主模型的所有降级选项，按优先级排列
    pub async fn get_fallbacks_for_model(&self, model_id: &str) -> Result<Vec<String>> {
        let fallbacks = sqlx::query(
            "SELECT fallback_model_id FROM model_fallbacks WHERE primary_model_id = ? ORDER BY priority ASC"
        )
        .bind(model_id)
        .map(|row: SqliteRow| row.get::<String, _>(0))
        .fetch_all(&self.db.pool)
        .await?;

        Ok(fallbacks)
    }

    /// 增加降级规则
    pub async fn add_fallback(&self, primary: &str, fallback: &str, priority: i32) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO model_fallbacks (id, primary_model_id, fallback_model_id, priority) VALUES (?, ?, ?, ?)"
        )
        .bind(id)
        .bind(primary)
        .bind(fallback)
        .bind(priority)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}
