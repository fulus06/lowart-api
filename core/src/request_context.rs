use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 通用请求/响应上下文
/// 实现原理: 该结构体作为 Lowart-api 内部流转的标准格式。所有厂商请求都会转换为此格式，处理后再转回目标格式。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub user_id: String,
    pub model_id: String,
    pub request_format: String,      // 原始请求格式
    pub expect_response_format: String, // 期望响应格式
    pub payload: serde_json::Value,  // 统一后的 OpenAI 兼容负载
    pub mcp_tools: Vec<protocols::mcp::McpTool>, // 注入的 MCP 工具
    pub metadata: HashMap<String, String>,
}


impl RequestContext {
    pub fn new(user_id: String, model_id: String, payload: serde_json::Value) -> Self {
        Self {
            user_id,
            model_id,
            request_format: "openai".to_string(),
            expect_response_format: "openai".to_string(),
            payload,
            mcp_tools: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

