use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use db::{DbConnection, ConfigRepo};

use models::{AiModel, OpenAiAdapter, AnthropicAdapter, ComfyUiAdapter};
use utils::{Result, anyhow};

/// 模型管理器
/// 实现原理: 负责维护模型适配器的生命周期和缓存。它根据 model_id 从数据库查询配置，
/// 并动态实例化对应的具体适配器。使用 Arc<RwLock<...>> 确保多线程安全访问。
pub struct ModelManager {
    db: Arc<DbConnection>,
    // 缓存已实例化的模型，避免重复解析配置和创建客户端
    cache: RwLock<HashMap<String, Arc<dyn AiModel>>>,
}

impl ModelManager {
    pub fn new(db: Arc<DbConnection>) -> Self {
        Self {
            db,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// 获取模型适配器及其转换脚本
    pub async fn get_model_with_scripts(&self, model_id: &str) -> Result<(Arc<dyn AiModel>, Option<String>, Option<String>)> {
        // 1. 先查数据库获取完整配置 (包含脚本)
        let repo = ConfigRepo::new(&self.db);
        let config = repo.find_by_model_id(model_id).await?
            .ok_or_else(|| anyhow!("模型配置未找到: {}", model_id))?;

        let request_script = config.request_script.clone();
        let response_script = config.response_script.clone();

        // 2. 查缓存或实例化适配器
        let adapter = {
            let cache = self.cache.read().await;
            if let Some(model) = cache.get(model_id) {
                Some(Arc::clone(model))
            } else {
                None
            }
        };

        if let Some(adapter) = adapter {
            return Ok((adapter, request_script, response_script));
        }

        // 3. 实例化适配器
        let model: Arc<dyn AiModel> = match config.vendor_type.as_str() {
            "OpenAI" => Arc::new(OpenAiAdapter::new(
                config.model_id.clone(),
                config.api_key.clone(),
                config.base_url.clone(),
            )),
            "Anthropic" => Arc::new(AnthropicAdapter::new(
                config.model_id.clone(),
                config.api_key.clone(),
                config.base_url.clone(),
            )),
            "ComfyUI" => Arc::new(ComfyUiAdapter::new(
                config.model_id.clone(),
                config.base_url.clone(),
            )),
            _ => return Err(anyhow!("不支持的厂商类型: {}", config.vendor_type)),
        };

        // 4. 写入缓存并返回
        let mut cache = self.cache.write().await;
        cache.insert(model_id.to_string(), Arc::clone(&model));
        
        Ok((model, request_script, response_script))
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
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::info!("模型缓存已清除");
    }
}
