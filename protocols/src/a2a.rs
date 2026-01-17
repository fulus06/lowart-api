use serde::{Serialize, Deserialize};
use serde_json::Value;


// Agent-to-Agent 协议基础实现
// 后续在此扩展消息格式和路由逻辑

/// Agent 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    TaskAssign,    // 任务分配
    TaskStatus,    // 状态更新
    TaskResult,    // 任务结果
    Query,         // 信息查询
    Response,      // 信息回复
}

/// Agent 通用消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,           // 消息唯一标识
    pub sender: String,       // 发送者 Agent ID
    pub receiver: String,     // 接收者 Agent ID (广播为 "*")
    pub msg_type: MessageType,
    pub content: Value,       // 消息具体内容
    pub timestamp: i64,       // 时间戳
}

/// Agent 参与者接口
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// 获取 Agent ID
    fn id(&self) -> &str;
    /// 接收并处理消息
    async fn handle_message(&self, msg: AgentMessage) -> utils::Result<Option<AgentMessage>>;
}

/// Agent 消息总线接口 (用于分发)
#[async_trait::async_trait]
pub trait AgentBus: Send + Sync {
    /// 广播消息
    async fn broadcast(&self, msg: AgentMessage) -> utils::Result<()>;
    /// 点对点发送消息
    async fn send_to(&self, receiver_id: &str, msg: AgentMessage) -> utils::Result<()>;
}
