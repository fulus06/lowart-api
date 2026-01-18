use crate::models::ModelConfig;
use crate::connection::DbConnection;
use utils::Result;

/// 模型配置资源仓库
pub struct ConfigRepo<'a> {
    pub db: &'a DbConnection,
}

impl<'a> ConfigRepo<'a> {
    pub fn new(db: &'a DbConnection) -> Self {
        Self { db }
    }

    /// 获取所有激活的模型配置
    pub async fn list_active(&self) -> Result<Vec<ModelConfig>> {
        let configs = sqlx::query_as::<_, ModelConfig>("SELECT * FROM model_configs WHERE is_active = 1")
            .fetch_all(&self.db.pool)
            .await?;
        Ok(configs)
    }

    /// 根据 model_id 获取配置
    pub async fn find_by_model_id(&self, model_id: &str) -> Result<Option<ModelConfig>> {
        let config = sqlx::query_as::<_, ModelConfig>("SELECT * FROM model_configs WHERE model_id = ? AND is_active = 1")
            .bind(model_id)
            .fetch_optional(&self.db.pool)
            .await?;
        Ok(config)
    }

    /// 创建或重置模型配置
    pub async fn create(&self, config: &ModelConfig) -> Result<()> {
        sqlx::query(
            "INSERT INTO model_configs (id, title, model_id, api_key, base_url, vendor_type, cost_per_1k_tokens, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&config.id)
        .bind(&config.title)
        .bind(&config.model_id)
        .bind(&config.api_key)
        .bind(&config.base_url)
        .bind(&config.vendor_type)
        .bind(config.cost_per_1k_tokens)
        .bind(config.is_active)
        .bind(config.created_at)
        .execute(&self.db.pool).await?;
        Ok(())
    }
}

