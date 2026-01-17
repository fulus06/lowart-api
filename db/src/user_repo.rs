use crate::models::User;
use crate::connection::DbConnection;
use utils::Result;

/// 用户资源仓库
/// 实现逻辑: 提供对 `users` 表的增删改查操作。
pub struct UserRepo<'a> {
    pub db: &'a DbConnection,
}

impl<'a> UserRepo<'a> {
    pub fn new(db: &'a DbConnection) -> Self {
        Self { db }
    }

    /// 根据 API Key 获取用户
    pub async fn find_by_api_key(&self, api_key: &str) -> Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE api_key = ?")
            .bind(api_key)
            .fetch_optional(&self.db.pool)
            .await?;
        Ok(user)
    }

    /// 创建用户
    pub async fn create(&self, user_id: &str, api_key: &str, is_admin: bool) -> Result<()> {
        sqlx::query(
            "INSERT INTO users (id, api_key, is_admin, rpm_limit, token_quota, token_used) VALUES (?, ?, ?, 60, 1000000, 0)"
        )
        .bind(user_id)
        .bind(api_key)
        .bind(is_admin)
        .execute(&self.db.pool).await?;
        Ok(())
    }

    /// 设置管理员状态
    pub async fn set_admin(&self, user_id: &str, is_admin: bool) -> Result<()> {
        sqlx::query("UPDATE users SET is_admin = ? WHERE id = ?")
            .bind(is_admin)
            .bind(user_id)
            .execute(&self.db.pool).await?;
        Ok(())
    }

    /// 获取所有用户
    pub async fn list_all(&self) -> Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch_all(&self.db.pool).await?;
        Ok(users)
    }

    /// 更新用户配额
    pub async fn update_quota(&self, user_id: &str, rpm_limit: i64, token_quota: i64) -> Result<()> {
        sqlx::query("UPDATE users SET rpm_limit = ?, token_quota = ? WHERE id = ?")
            .bind(rpm_limit)
            .bind(token_quota)
            .bind(user_id)
            .execute(&self.db.pool).await?;
        Ok(())
    }

    /// 增加用户 Token 使用量
    pub async fn increment_token_usage(&self, user_id: &str, amount: i64) -> Result<()> {
        sqlx::query("UPDATE users SET token_used = token_used + ? WHERE id = ?")
            .bind(amount)
            .bind(user_id)
            .execute(&self.db.pool)
            .await?;
        Ok(())
    }
}
