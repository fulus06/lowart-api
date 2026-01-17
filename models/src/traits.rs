use async_trait::async_trait;
use utils::Result;
use serde_json::Value;
use futures::Stream;
use std::pin::Pin;

/// SSE 流类型定义
pub type BoxStream<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

/// AI 模型通用接口
/// 实现原则: 高模块化，通过 Trait 定义统一的模型调用行为，便于支持多种 AI 厂商。
#[async_trait]
pub trait AiModel: Send + Sync {
    /// 发送对话请求
    async fn chat_completions(&self, payload: Value) -> Result<Value>;

    /// 发送流式对话请求
    async fn chat_completions_stream(&self, payload: Value) -> Result<BoxStream<Result<Value>>>;

    /// 获取模型标识符
    fn model_id(&self) -> &str;
}

