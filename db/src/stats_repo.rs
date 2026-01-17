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

    /// 便捷方法：仅记录 Token 和模型标识，其他字段使用默认值
    pub async fn record_usage(&self, user_id: &str, model_id: &str, req_tokens: i64, res_tokens: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO usage_stats (user_id, model_id, request_tokens, response_tokens, request_count, response_count, duration_ms, timestamp) 
             VALUES (?, ?, ?, ?, 1, 1, 0, ?)"
        )
        .bind(user_id)
        .bind(model_id)
        .bind(req_tokens)
        .bind(res_tokens)
        .bind(chrono::Utc::now())
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }
}
