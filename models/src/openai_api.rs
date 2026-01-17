use crate::traits::{AiModel, BoxStream};
use async_trait::async_trait;
use utils::{Result, anyhow};
use serde_json::Value;
use reqwest::Client;
use futures::StreamExt;

/// OpenAI API 适配器
/// 实现原理: 封装标准 OpenAI Chat Completions 协议调用。支持阻塞和流式输出。
pub struct OpenAiAdapter {
    pub model_id: String,
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAiAdapter {
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
impl AiModel for OpenAiAdapter {
    async fn chat_completions(&self, payload: Value) -> Result<Value> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("OpenAI 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI 响应错误 ({}): {}", status, error_text));
        }

        let result = response.json::<Value>().await?;
        Ok(result)
    }

    async fn chat_completions_stream(&self, payload: Value) -> Result<BoxStream<Result<Value>>> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        
        // 确保 payload 中包含 stream: true
        let mut stream_payload = payload.clone();
        if let Some(obj) = stream_payload.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }

        let response = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&stream_payload)
            .send()
            .await
            .map_err(|e| anyhow!("OpenAI 流请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI 流响应错误 ({}): {}", status, error_text));
        }

        // 使用 bytes_stream 处理 SSE 数据
        let stream = response.bytes_stream().map(|item| {
            item.map_err(|e| anyhow!("流读取错误: {}", e))
                .and_then(|bytes| {
                    // 这里简化了处理：实际需要根据 SSE 规范解析 data: 字段
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

