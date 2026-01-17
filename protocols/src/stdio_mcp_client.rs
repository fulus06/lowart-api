use crate::mcp::{McpClient, McpTool, McpResource, McpServerMeta};
use async_trait::async_trait;
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use std::process::Stdio;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;
use utils::{Result, anyhow};

/// 基于标准输入输出的 MCP 客户端
pub struct StdioMcpClient {
    #[allow(dead_code)]
    child: Arc<Mutex<Child>>,
    stdin: Arc<Mutex<tokio::process::ChildStdin>>,
    stdout_reader: Arc<Mutex<BufReader<tokio::process::ChildStdout>>>,
    request_id: Arc<Mutex<u64>>,
}

impl StdioMcpClient {
    pub async fn spawn(command: &str, args: &[&str]) -> anyhow::Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| anyhow!("无法获取 stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| anyhow!("无法获取 stdout"))?;

        Ok(Self {
            child: Arc::new(Mutex::new(child)),
            stdin: Arc::new(Mutex::new(stdin)),
            stdout_reader: Arc::new(Mutex::new(BufReader::new(stdout))),
            request_id: Arc::new(Mutex::new(0)),
        })
    }

    async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let mut id_lock = self.request_id.lock().await;
        *id_lock += 1;
        let id = *id_lock;
        drop(id_lock);

        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id
        });

        let mut stdin = self.stdin.lock().await;
        let line = format!("{}\n", request.to_string());
        stdin.write_all(line.as_bytes()).await.map_err(|e| anyhow!("写入 stdin 失败: {}", e))?;
        stdin.flush().await.map_err(|e| anyhow!("刷新 stdin 失败: {}", e))?;
        drop(stdin);

        let mut reader = self.stdout_reader.lock().await;
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.map_err(|e| anyhow!("读取 stdout 失败: {}", e))?;
        
        let response: Value = serde_json::from_str(&response_line).map_err(|e| anyhow!("反序列化响应失败: {}", e))?;
        if let Some(error) = response.get("error") {
            return Err(anyhow!("MCP 错误: {}", error));
        }

        Ok(response.get("result").cloned().unwrap_or(Value::Null))
    }
}

#[async_trait]
impl McpClient for StdioMcpClient {
    async fn initialize(&self, meta: McpServerMeta) -> Result<Value> {
        self.call("initialize", serde_json::to_value(meta).map_err(|e| anyhow!("序列化元数据失败: {}", e))?).await
    }

    async fn list_tools(&self) -> Result<Vec<McpTool>> {
        let res = self.call("tools/list", json!({})).await?;
        let tools: Vec<McpTool> = serde_json::from_value(res.get("tools").cloned().unwrap_or(json!([])))
            .map_err(|e| anyhow!("反序列化工具列表失败: {}", e))?;
        Ok(tools)
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value> {
        self.call("tools/call", json!({
            "name": name,
            "arguments": arguments
        })).await
    }

    async fn list_resources(&self) -> Result<Vec<McpResource>> {
        let res = self.call("resources/list", json!({})).await?;
        let resources: Vec<McpResource> = serde_json::from_value(res.get("resources").cloned().unwrap_or(json!([])))
            .map_err(|e| anyhow!("反序列化资源列表失败: {}", e))?;
        Ok(resources)
    }
}
