use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use protocols::a2a::{Agent, AgentMessage, AgentBus};
use utils::{Result, anyhow};
use async_trait::async_trait;


/// Agent 编排器
/// 实现逻辑: 核心任务路由中心。管理所有的 Agent 实例，并充当消息总线的角色。
pub struct AgentOrchestrator {
    agents: RwLock<HashMap<String, Arc<dyn Agent>>>,
    // 基础的总线功能可以集成在这里，或者使用专门的总线实现
}

impl AgentOrchestrator {
    pub fn new() -> Self {
        Self {
            agents: RwLock::new(HashMap::new()),
        }
    }

    /// 注册 Agent
    pub async fn register_agent(&self, agent: Arc<dyn Agent>) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id().to_string(), agent);
    }

    /// 获取特定 Agent
    pub async fn get_agent(&self, id: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.agents.read().await;
        agents.get(id).map(Arc::clone)
    }

    /// 发送并路由消息
    pub async fn dispatch(&self, msg: AgentMessage) -> Result<()> {
        let receiver_id = msg.receiver.clone();
        
        if receiver_id == "*" {
            // 广播逻辑
            self.broadcast(msg).await?;
        } else {
            // 点对点逻辑
            self.send_to(&receiver_id, msg).await?;
        }
        
        Ok(())
    }
}

#[async_trait]
impl AgentBus for AgentOrchestrator {

    async fn broadcast(&self, msg: AgentMessage) -> Result<()> {
        let agents = self.agents.read().await;
        for agent in agents.values() {
            if agent.id() != msg.sender {
                let agent_clone = Arc::clone(agent);
                let msg_clone = msg.clone();
                // 异步处理，防止死锁或阻塞
                tokio::spawn(async move {
                    if let Err(e) = agent_clone.handle_message(msg_clone).await {
                        tracing::error!("Agent {} 处理广播消息失败: {}", agent_clone.id(), e);
                    }
                });
            }
        }
        Ok(())
    }

    async fn send_to(&self, receiver_id: &str, msg: AgentMessage) -> Result<()> {
        let agents = self.agents.read().await;
        if let Some(agent) = agents.get(receiver_id) {
            let agent_clone = Arc::clone(agent);
            // 这里根据业务需求可以是同步等待或是异步分发
            tokio::spawn(async move {
                if let Err(e) = agent_clone.handle_message(msg).await {
                    tracing::error!("Agent {} 处理点对点消息失败: {}", agent_clone.id(), e);
                }
            });
            Ok(())
        } else {
            Err(anyhow!("未找到目标 Agent: {}", receiver_id))
        }
    }
}
