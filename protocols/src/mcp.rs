use serde::{Serialize, Deserialize};
use serde_json::Value;

/// JSON-RPC 2.0 基础消息结构
#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
    pub id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

/// MCP 工具定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Value, // JSON Schema
}

/// MCP 资源定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpResource {
    pub uri: String,
    pub name: String,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

/// MCP 提示词模板
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpPrompt {
    pub name: String,
    pub description: Option<String>,
    pub arguments: Option<Vec<McpArgument>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpArgument {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

/// MCP 服务器元数据
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpServerMeta {
    pub name: String,
    pub version: String,
    pub capabilities: Value,
}
/// MCP 客户端接口
#[async_trait::async_trait]
pub trait McpClient: Send + Sync {
    /// 初始化连接
    async fn initialize(&self, meta: McpServerMeta) -> utils::Result<Value>;
    /// 获取可用工具列表
    async fn list_tools(&self) -> utils::Result<Vec<McpTool>>;
    /// 调用工具
    async fn call_tool(&self, name: &str, arguments: Value) -> utils::Result<Value>;
    /// 获取资源列表
    async fn list_resources(&self) -> utils::Result<Vec<McpResource>>;
}
