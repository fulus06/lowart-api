# Lowart-api
[English](./README.md)
Lowart-api 是一个基于 Rust 开发的轻量级、高性能 AI API 服务。它旨在同时兼容云端 AI 供应商和本地 AI 工作流，为 AI 交互提供统一的接口。

## 核心特性

- **模块化架构**：基于 Cargo Workspace 构建，模块职责分离清晰。
- **多模型支持**：内置 OpenAI、Anthropic 和 ComfyUI 标准适配器。
- **动态格式转换**：集成 **Rhai** 脚本引擎，支持在不重启服务的情况下动态转换请求和响应负载。
- **高效路由**：通过 `ModelManager` 根据数据库配置动态选择适配器。
- **流式输出**：全链路支持 Server-Sent Events (SSE) 流式对话。
- **完善的使用统计**：自动追踪 Token 消耗、请求耗时及频率，并存储于 SQLite。
- **双模连接**：支持标准 HTTP 端口监听及极低延迟的 Unix Domain Socket (UDS)。
- **安全鉴权**：内置中间件实现的 API Key 校验机制。

## 架构说明

项目由多个专业 crate 组成：
- `api-server`：核心网关，处理 HTTP/UDS 路由及中间件。
- `core`：引擎核心，管理 `ModelManager`、`TokenCounter` 及 `RhaiEngine`。
- `models`：基于 Trait 定义的多厂商模型实现。
- `db`：数据库层，使用 SQLite (SQLx) 进行持久化存储。
- `auth`：身份管理与鉴权逻辑。
- `protocols`：支持 SSE、A2A 及 MCP 等协议。
- `utils`：通用的日志记录与错误处理工具。

## 快速入门

### 环境要求
- Rust (最新稳定版)
- SQLite

### 安装步骤
1. 克隆仓库：
   ```bash
   git clone <repository_url>
   cd lowart-api
   ```
2. 数据库初始化：
   应用在首次运行时会自动创建 `lowart.db` 并初始化相关表结构。

### 启动服务
```bash
# 以 HTTP 模式启动（默认）
cargo run -p api-server

# 或设置 UDS 模式
LISTEN_MODE=UDS cargo run -p api-server
```

## 使用示例

### 对话请求（阻塞）
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer <您的API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "你好！"}]
  }'
```

### 对话请求（流式）
```bash
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Authorization: Bearer <您的API_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gpt-4",
    "stream": true,
    "messages": [{"role": "user", "content": "详细介绍一下 Rust。"}]
  }'
```

## 贡献
欢迎提交 Pull Request 以持续优化项目！

## 许可证
MIT License
