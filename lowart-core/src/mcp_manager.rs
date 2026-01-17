use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use protocols::mcp::{McpClient, McpTool, McpResource};
use protocols::a2a::{AgentMessage, MessageType};
use crate::agent_orchestrator::AgentOrchestrator;
use utils::{Result, anyhow};
use serde_json::json;

/// MCP 管理器
/// 实现逻辑: 维护多个 MCP 客户端连接，聚合可用的工具和资源。
/// 同时集成了 A2A 路由能力，作为 AI 触发跨 Agent 调用的人造入口。
pub struct McpManager {
    clients: RwLock<HashMap<String, Arc<dyn McpClient>>>,
    orchestrator: Arc<AgentOrchestrator>,
}

impl McpManager {
    pub fn new(orchestrator: Arc<AgentOrchestrator>) -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
            orchestrator,
        }
    }

    /// 注册一个新的 MCP 客户端
    pub async fn register_client(&self, name: String, client: Arc<dyn McpClient>) {
        let mut clients = self.clients.write().await;
        clients.insert(name, client);
    }

    /// 注销一个已有的 MCP 客户端
    pub async fn unregister_client(&self, name: &str) {
        let mut clients = self.clients.write().await;
        clients.remove(name);
        tracing::info!("MCP 客户端 {} 已下线", name);
    }


    /// 聚合所有客户端的可用的工具 (包含内置系统工具)
    pub async fn list_all_tools(&self) -> Result<Vec<McpTool>> {
        let clients = self.clients.read().await;
        let mut all_tools = Vec::new();
        
        // 1. 添加内置虚拟工具: route_to_agent
        all_tools.push(McpTool {
            name: "route_to_agent".to_string(),
            description: Some("将任务路由给另一个专门的 Agent（如 ComfyUI 画画 Agent）。".to_string()),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "target_agent": { "type": "string", "description": "目标 Agent 的标识符 (如 comfyui_painter)" },
                    "message": { "type": "string", "description": "要发送的具体任务内容 (Prompt)" }
                },
                "required": ["target_agent", "message"]
            }),
        });

        // 2. 爬取所有 MCP 客户端的工具
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

    /// 全局查找并调用工具 (支持虚拟工具)
    pub async fn call_tool_any(&self, tool_name: &str, arguments: serde_json::Value) -> Result<serde_json::Value> {
        // 1. 拦截内置虚拟工具
        if tool_name == "route_to_agent" {
            let target = arguments["target_agent"].as_str()
                .ok_or_else(|| anyhow!("缺少目标 Agent"))?;
            let content = arguments["message"].as_str()
                .ok_or_else(|| anyhow!("缺少消息内容"))?;

            let msg = AgentMessage {
                id: uuid::Uuid::new_v4().to_string(),
                sender: "gateway".to_string(),
                receiver: target.to_string(),
                msg_type: MessageType::TaskAssign,
                content: json!({ "prompt": content }),
                timestamp: chrono::Utc::now().timestamp(),
            };

            let job_id = msg.id.clone();
            self.orchestrator.dispatch(msg).await?;
            return Ok(json!({ "status": "sent", "target": target, "job_id": job_id }));

        }

        // 2. 查找外部工具
        let clients = self.clients.read().await;
        for (name, client) in clients.iter() {
            if let Ok(tools) = client.list_tools().await {
                if tools.iter().any(|t| t.name == tool_name) {
                    tracing::debug!("命中工具 {} -> 客户端 {}", tool_name, name);
                    return client.call_tool(tool_name, arguments).await;
                }
            }
        }
        Err(anyhow!("全局未找到工具: {}", tool_name))
    }
}
