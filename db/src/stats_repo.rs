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
            "INSERT INTO usage_stats (user_id, model_id, request_tokens, response_tokens, request_count, response_count, duration_ms, stat_type, timestamp) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(stat.user_id)
        .bind(stat.model_id)
        .bind(stat.request_tokens)
        .bind(stat.response_tokens)
        .bind(stat.request_count)
        .bind(stat.response_count)
        .bind(stat.duration_ms)
        .bind(stat.stat_type)
        .bind(stat.timestamp)
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    /// 便捷方法：仅记录 Token 和模型标识，其他字段使用默认值
    pub async fn record_usage(&self, user_id: &str, model_id: &str, req_tokens: i64, res_tokens: i64, stat_type: &str, duration_ms: i64) -> Result<()> {
        sqlx::query(
            "INSERT INTO usage_stats (user_id, model_id, request_tokens, response_tokens, request_count, response_count, duration_ms, stat_type, timestamp) 
             VALUES (?, ?, ?, ?, 1, 1, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(model_id)
        .bind(req_tokens)
        .bind(res_tokens)
        .bind(duration_ms)
        .bind(stat_type)
        .bind(chrono::Utc::now())
        .execute(&self.db.pool)
        .await?;
        Ok(())
    }

    /// 获取最近的使用记录
    pub async fn list_recent(&self, limit: i64) -> Result<Vec<UsageStat>> {
        let stats = sqlx::query_as::<_, UsageStat>("SELECT * FROM usage_stats ORDER BY timestamp DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.db.pool)
            .await?;
        Ok(stats)
    }
}
