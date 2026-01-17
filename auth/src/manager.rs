use db::{DbConnection, UserRepo};

use utils::{Result, anyhow};

use std::sync::Arc;

/// 身份管理服务
/// 实现逻辑: 负责用户身份的校验、API Key 的生命周期管理。
pub struct AuthManager {
    pub db: Arc<DbConnection>,
}

impl AuthManager {
    pub fn new(db: Arc<DbConnection>) -> Self {
        Self { db }
    }


    /// 校验 API Key 并返回完整用户信息
    pub async fn authenticate(&self, api_key: &str) -> Result<db::User> {
        let repo = UserRepo::new(&self.db);
        let user = repo.find_by_api_key(api_key).await?
            .ok_or_else(|| anyhow!("无效的 API Key"))?;

        if user.status != "Active" {
            return Err(anyhow!("用户状态异常: {}", user.status));
        }

        Ok(user)
    }

}

