use crate::traits::AiModel;
use async_trait::async_trait;
use serde_json::{Value, json};
use utils::{Result, Error};
use futures::Stream;
use std::pin::Pin;

/// 模拟模型适配器，用于单元测试与集成测试
pub struct MockAdapter {
    pub model_id: String,
    pub response_delay: std::time::Duration,
    pub should_fail: bool,
    pub error_message: String,
    pub mock_response: Value,
    pub tool_call_response: Option<Value>,
}

impl MockAdapter {
    pub fn with_tool_call(call_id: &str, tool_name: &str, args: &str) -> Self {
        Self {
            model_id: "mock-tool-model".to_string(),
            response_delay: std::time::Duration::from_millis(0),
            should_fail: false,
            error_message: String::new(),
            mock_response: json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": null,
                        "tool_calls": [{
                            "id": call_id,
                            "type": "function",
                            "function": {
                                "name": tool_name,
                                "arguments": args
                            }
                        }]
                    },
                    "finish_reason": "tool_calls"
                }]
            }),
            tool_call_response: None,
        }
    }

    pub fn success() -> Self {
        Self {
            model_id: "mock-model".to_string(),
            response_delay: std::time::Duration::from_millis(0),
            should_fail: false,
            error_message: String::new(),
            mock_response: json!({
                "choices": [{
                    "message": {
                        "role": "assistant",
                        "content": "Hello! I am a mock AI."
                    }
                }]
            }),
            tool_call_response: None,
        }
    }

    pub fn fail(msg: &str) -> Self {
        Self {
            model_id: "mock-fail".to_string(),
            response_delay: std::time::Duration::from_millis(0),
            should_fail: true,
            error_message: msg.to_string(),
            mock_response: json!({}),
            tool_call_response: None,
        }
    }
}


#[async_trait]
impl AiModel for MockAdapter {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn chat_completions(&self, _payload: Value) -> Result<Value> {
        if self.response_delay.as_millis() > 0 {
            tokio::time::sleep(self.response_delay).await;
        }

        if self.should_fail {
            return Err(utils::anyhow!(self.error_message.clone()));
        }

        Ok(self.mock_response.clone())
    }

    async fn chat_completions_stream(&self, _payload: Value) -> Result<Pin<Box<dyn Stream<Item = Result<Value>> + Send>>> {
        if self.should_fail {
            return Err(utils::anyhow!(self.error_message.clone()));
        }


        // 简单的流式模拟：发送两个 chunk
        let chunks = vec![
            Ok(json!({"choices": [{"delta": {"content": "M"}}]} )),
            Ok(json!({"choices": [{"delta": {"content": "ock"}}]} )),
        ];
        
        let stream = futures::stream::iter(chunks);
        Ok(Box::pin(stream))
    }
}
