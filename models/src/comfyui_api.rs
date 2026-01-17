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
        let base = self.base_url.trim_end_matches('/');
        let prompt_url = format!("{}/prompt", base);
        
        let response = self.client.post(&prompt_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| anyhow!("ComfyUI 请求失败: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(anyhow!("ComfyUI 提交失败 ({}): {}", status, error_text));
        }

        let run_res = response.json::<Value>().await?;
        let prompt_id = run_res["prompt_id"].as_str()
            .ok_or_else(|| anyhow!("ComfyUI 未返回 prompt_id"))?;

        // 开始轮询状态 (简单实现)
        let history_url = format!("{}/history/{}", base, prompt_id);
        let mut attempts = 0;
        let max_attempts = 60; // 5 分钟 (5s * 60)

        while attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            let hist_res = self.client.get(&history_url).send().await?;
            if hist_res.status().is_success() {
                let hist_data = hist_res.json::<Value>().await?;
                if !hist_data[prompt_id].is_null() {
                    // 任务已完成
                    return Ok(hist_data[prompt_id].clone());
                }
            }
            attempts += 1;
        }

        Err(anyhow!("ComfyUI 任务超时 (prompt_id: {})", prompt_id))
    }


    async fn chat_completions_stream(&self, _payload: Value) -> Result<crate::traits::BoxStream<Result<Value>>> {
        Err(anyhow!("ComfyUI 目前不支持流式输出"))
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
