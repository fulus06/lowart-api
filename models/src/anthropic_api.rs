use crate::traits::{AiModel, BoxStream};
use async_trait::async_trait;
use utils::{Result, anyhow};
use serde_json::Value;
use reqwest::Client;
use futures::StreamExt;

/// Anthropic API 适配器
/// 实现原理: 封装 Anthropic Messages API 调用。支持阻塞和流式输出。
pub struct AnthropicAdapter {
    pub model_id: String,
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl AnthropicAdapter {
    pub fn new(model_id: String, api_key: String, base_url: String) -> Self {
        Self {
            model_id,
            api_key,
            base_url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl AiModel for AnthropicAdapter {
    async fn chat_completions(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        
        let response = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("Anthropic 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic 响应错误 ({}): {}", status, error_text));
        }

        let result = response.json::<Value>().await?;
        Ok(result)
    }

    async fn chat_completions_stream(&self, payload: Value) -> Result<BoxStream<Result<Value>>> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        
        let mut stream_payload = payload.clone();
        if let Some(obj) = stream_payload.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        let response = self.client.post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&stream_payload)
            .send()
            .await
            .map_err(|e| anyhow!("Anthropic 流请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("Anthropic 流响应错误 ({}): {}", status, error_text));
        }

        let stream = response.bytes_stream().map(|item| {
            item.map_err(|e| anyhow!("流读取错误: {}", e))
                .and_then(|bytes| {
                    let text = String::from_utf8_lossy(&bytes);
                    Ok(serde_json::json!({ "raw": text }))
                })
        });

        Ok(Box::pin(stream))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

