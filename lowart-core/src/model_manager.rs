use std::sync::Arc;
use db::{DbConnection, ConfigRepo};
use moka::future::Cache;
use std::time::Duration;

use models::{AiModel, OpenAiAdapter, AnthropicAdapter, ComfyUiAdapter};
use utils::{Result, anyhow};

/// 模型管理器缓存项
#[derive(Clone)]
struct ModelCacheItem {
    adapter: Arc<dyn AiModel>,
    request_script: Option<String>,
    response_script: Option<String>,
}

/// 模型管理器
/// 实现原理: 负责维护模型适配器的生命周期和缓存。
/// 使用 moka 高性能缓存，支持过期自动清理，减少数据库压力和解密运算。
pub struct ModelManager {
    db: Arc<DbConnection>,
    // 聚合缓存: model_id -> (适配器, 转换脚本)
    cache: Cache<String, ModelCacheItem>,
}

impl ModelManager {
    pub fn new(db: Arc<DbConnection>) -> Self {
        Self {
            db,
            cache: Cache::builder()
                .max_capacity(100)
                .time_to_live(Duration::from_secs(3600)) // 1小时过期
                .build(),
        }
    }

    /// 获取模型适配器及其转换脚本
    pub async fn get_model_with_scripts(&self, model_id: &str) -> Result<(Arc<dyn AiModel>, Option<String>, Option<String>)> {
        // 1. 尝试从缓存获取
        if let Some(item) = self.cache.get(model_id).await {
            return Ok((item.adapter, item.request_script, item.response_script));
        }

        // 2. 缓存未命中，查数据库获取完整配置
        let repo = ConfigRepo::new(&self.db);
        let config = repo.find_by_model_id(model_id).await?
            .ok_or_else(|| anyhow!("模型配置未找到: {}", model_id))?;

        let request_script = config.request_script.clone();
        let response_script = config.response_script.clone();

        // 3. 实例化适配器并解密 Key
        let decrypted_key = match utils::Crypto::decrypt(&config.api_key) {
            Ok(key) => key,
            Err(_) => config.api_key.clone(),
        };

        let adapter: Arc<dyn AiModel> = match config.vendor_type.as_str() {
            "OpenAI" => Arc::new(OpenAiAdapter::new(
                config.model_id.clone(),
                decrypted_key.clone(),
                config.base_url.clone(),
            )),
            "Anthropic" => Arc::new(AnthropicAdapter::new(
                config.model_id.clone(),
                decrypted_key.clone(),
                config.base_url.clone(),
            )),
            "ComfyUI" => Arc::new(ComfyUiAdapter::new(
                config.model_id.clone(),
                config.base_url.clone(),
            )),
            _ => return Err(anyhow!("不支持的厂商类型: {}", config.vendor_type)),
        };

        // 4. 写入缓存并返回
        let item = ModelCacheItem {
            adapter: Arc::clone(&adapter),
            request_script: request_script.clone(),
            response_script: response_script.clone(),
        };
        self.cache.insert(model_id.to_string(), item).await;
        
        Ok((adapter, request_script, response_script))
    }

    /// 获取模型适配器 (向下兼容)
    pub async fn get_model(&self, model_id: &str) -> Result<Arc<dyn AiModel>> {
        let (adapter, _, _) = self.get_model_with_scripts(model_id).await?;
        Ok(adapter)
    }

    pub fn db(&self) -> Arc<DbConnection> {
        Arc::clone(&self.db)
    }

    /// 清除缓存 (用于配置热更新场景)
    pub async fn clear_cache(&self) {
        self.cache.invalidate_all();
        tracing::info!("模型管理器缓存已清除");
    }
}
