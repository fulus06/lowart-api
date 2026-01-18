use crate::models::ApiKey;
use crate::connection::DbConnection;
use utils::Result;
use chrono::Utc;

/// API Key 资源仓库
pub struct ApiKeyRepo<'a> {
    pub db: &'a DbConnection,
}

impl<'a> ApiKeyRepo<'a> {
    pub fn new(db: &'a DbConnection) -> Self {
        Self { db }
    }

    /// 获取用户的所有 API Key
    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<ApiKey>> {
        let keys = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.db.pool)
            .await?;
        Ok(keys)
    }

    /// 创建新的 API Key
    pub async fn create(&self, user_id: &str, label: &str) -> Result<String> {
        let new_key = format!("la-{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        sqlx::query(
            "INSERT INTO api_keys (user_id, api_key, label, status) VALUES (?, ?, ?, 'Active')"
        )
        .bind(user_id)
        .bind(&new_key)
        .bind(label)
        .execute(&self.db.pool).await?;
        Ok(new_key)
    }

    /// 重置 (重新生成) 指定 ID 的 API Key
    pub async fn reset(&self, id: i64) -> Result<String> {
        let new_key = format!("la-{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        sqlx::query("UPDATE api_keys SET api_key = ?, created_at = CURRENT_TIMESTAMP WHERE id = ?")
            .bind(&new_key)
            .bind(id)
            .execute(&self.db.pool).await?;
        Ok(new_key)
    }

    /// 删除 API Key
    pub async fn delete(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id)
            .execute(&self.db.pool).await?;
        Ok(())
    }

    /// 根据 Key 查找 (用于鉴权)
    pub async fn find_by_key(&self, api_key: &str) -> Result<Option<ApiKey>> {
        let key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE api_key = ? AND status = 'Active'")
            .bind(api_key)
            .fetch_optional(&self.db.pool)
            .await?;
        Ok(key)
    }

    /// 根据 ID 查找
    pub async fn find_by_id(&self, id: i64) -> Result<Option<ApiKey>> {
        let key = sqlx::query_as::<_, ApiKey>("SELECT * FROM api_keys WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.db.pool)
            .await?;
        Ok(key)
    }

    /// 更新最后使用时间
    pub async fn update_last_used(&self, id: i64) -> Result<()> {
        sqlx::query("UPDATE api_keys SET last_used_at = ? WHERE id = ?")
            .bind(Utc::now())
            .bind(id)
            .execute(&self.db.pool).await?;
        Ok(())
    }
}
