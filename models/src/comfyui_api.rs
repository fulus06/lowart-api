use crate::traits::AiModel;
use async_trait::async_trait;
use utils::{Result, anyhow};
use serde_json::Value;
use reqwest::Client;

/// ComfyUI API 适配器
/// 实现原理: 接入 ComfyUI 的 API 端点 (如 /prompt)，用于触发 AI 工作流。
pub struct ComfyUiAdapter {
    pub model_id: String,
    pub base_url: String,
    pub client: Client,
}

impl ComfyUiAdapter {
    pub fn new(model_id: String, base_url: String) -> Self {
        Self {
            model_id,
            base_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AiModel for ComfyUiAdapter {
    async fn chat_completions(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/prompt", self.base_url.trim_end_matches('/'));
        
        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("ComfyUI 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("ComfyUI 响应错误 ({}): {}", status, error_text));
        }

        let result = response.json::<Value>().await?;
        Ok(result)
    }

    async fn chat_completions_stream(&self, _payload: Value) -> Result<crate::traits::BoxStream<Result<Value>>> {
        Err(anyhow!("ComfyUI 目前不支持流式输出"))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
