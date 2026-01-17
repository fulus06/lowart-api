use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use protocols::mcp::{McpClient, McpTool, McpResource};
use utils::{Result, anyhow};

/// MCP 管理器
/// 实现逻辑: 维护多个 MCP 客户端连接，聚合可用的工具和资源。
pub struct McpManager {
    clients: RwLock<HashMap<String, Arc<dyn McpClient>>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// 注册一个新的 MCP 客户端
    pub async fn register_client(&self, name: String, client: Arc<dyn McpClient>) {
        let mut clients = self.clients.write().await;
        clients.insert(name, client);
    }

    /// 聚合所有客户端的可用的工具
    pub async fn list_all_tools(&self) -> Result<Vec<McpTool>> {
        let clients = self.clients.read().await;
        let mut all_tools = Vec::new();
        
        for client in clients.values() {
            match client.list_tools().await {
                Ok(tools) => all_tools.extend(tools),
                Err(e) => tracing::error!("获取工具列表失败: {}", e),
            }
        }
        
        Ok(all_tools)
    }

    /// 聚合所有客户端的可用资源
    pub async fn list_all_resources(&self) -> Result<Vec<McpResource>> {
        let clients = self.clients.read().await;
        let mut all_resources = Vec::new();
        
        for client in clients.values() {
            match client.list_resources().await {
                Ok(resources) => all_resources.extend(resources),
                Err(e) => tracing::error!("获取资源列表失败: {}", e),
            }
        }
        
        Ok(all_resources)
    }

    /// 调用特定客户端的工具
    pub async fn call_tool(&self, client_name: &str, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        let clients = self.clients.read().await;
        let client = clients.get(client_name)
            .ok_or_else(|| anyhow!("未找到 MCP 客户端: {}", client_name))?;
            
        client.call_tool(tool_name, arguments).await
    }
}
