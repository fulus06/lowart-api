use crate::models::UsageStat;
use crate::connection::DbConnection;
use utils::Result;

/// 使用统计资源仓库
pub struct StatsRepo<'a> {
    pub db: &'a DbConnection,
}

impl<'a> StatsRepo<'a> {
    pub fn new(db: &'a DbConnection) -> Self {
        Self { db }
    }

    /// 记录一次完整请求的统计
    pub async fn record(&self, stat: UsageStat) -> Result<()> {
        sqlx::query(
            "INSERT INTO usage_stats (user_id, model_id, request_tokens, response_tokens, request_count, response_count, duration_ms, timestamp) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(stat.user_id)
        .bind(stat.model_id)
        .bind(stat.request_tokens)
        .bind(stat.response_tokens)
        .bind(stat.request_count)
        .bind(stat.response_count)
        .bind(stat.duration_ms)
        .bind(stat.timestamp)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}
