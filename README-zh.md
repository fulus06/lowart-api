# Lowart-api
[English](./README.md)

Lowart-api 是一个基于 Rust 开发的轻量级、企业级 AI API 网关。它旨在通过统一的接口管理云端 AI 供应商（如 OpenAI, Anthropic）与本地 AI 工作流（如 ComfyUI），并提供工业级的稳定性保障、动态治理与可观测性。

## 🌟 核心特性

### 1. 高可用与弹性 (High Availability)
- **熔断机制 (Circuit Breaker)**：基于滑动窗口监测供应商健康度，自动隔离故障模型，防止故障级联。
- **多级降级 (Fallback)**：支持配置模型优先级链条，主模型异常时秒级切换至备用模型，确保业务零中断。
- **异步任务追踪 (Jobs)**：支持将长耗时生成任务（如 ComfyUI）转为后台 Job，支持状态轮询与结果持久化。

### 2. 工具治理与 Agent 协作 (Governance & Agents)
- **MCP 深度集成**：完整支持 Model Context Protocol，支持 Stdio 模式动态接入各类工具服务器（Node.js, Python 等）。
- **人机协同 (HITL)**：内置工具执行策略（自动、需确认、禁止），支持会话级的人工授权重放。
- **Agent 总线 (A2A)**：提供异步消息总线，支持多个 AI Agent 之间的任务分发与协作。

### 3. 动态扩展与安全 (Extensibility & Security)
- **Rhai 脚本引擎**：无需重启即可热更新请求/响应转换逻辑，适配各种非标协议。
- **商用级统计与计费**：
  - **SSE 全量精准计费**：攻克了流式输出无法直接统计 Token 的难题。
  - **配额管理**：支持基于 Token 的阶梯计费与 RPM 速率限制。
- **安全加固**：基于 AES-256-GCM 加密存储供应商 API Keys，内置管理员 RBAC 权限控制。

### 4. 工业级性能与观测 (Performance & Observability)
- **极速缓存**：集成 `moka` 高性能缓存，针对鉴权与配置实现毫秒级响应。
- **双模通信**：支持标准 TCP 协议与低延迟 Unix Domain Socket (UDS)。
- **Prometheus 指标**：实时暴露模型流量、响应耗时及 Token 消耗等核心运营数据。

---

## 🚀 使用场景

### 场景一：企业级 AI 统一网关
企业内部有多个开发团队需要调用 LLM，Lowart-api 可作为中台：
- **场景描述**：统一管理不同供应商的 API Key，对不同部门进行 Token 配额控制与速率限制。
- **价值点**：通过熔断和降级机制，避免因某个供应商宕机导致业务全面瘫痪。

### 场景二：复杂 Agent 系统工具箱
为自主 Agent 提供强大的工具调用能力：
- **场景描述**：通过 MCP 接入本地数据库、搜索插件或复杂的 Python 脚本工具。
- **价值点**：利用三级治理策略（HITL），确保 Agent 在调用敏感操作（如删除、转账）时必须经过人工确认。

### 场景三：生成式 AI 异步流水线
处理长耗时的图片、视频生成任务：
- **场景描述**：接入 ComfyUI 等生成引擎，将 HTTP 请求转为异步 Job，用户通过 Job ID 轮询结果。
- **价值点**：内置的 Rhai 脚本可动态适配复杂的 ComfyUI JSON 工作流。

---

## 🏗 架构说明

项目采用模块化 Workspace 结构，各 crate 职责如下：
- **`api-server`**：基于 Axum 0.8 构建，负责 UDS/TCP 监听、路由分发与 Admin 中间件。
- **`lowart-core`**：系统心脏，包含 `CircuitBreaker`、`ModelManager`、`TokenCounter` 及 `RhaiEngine`。
- **`models`**：供应商适配层，实现了 OpenAI、Anthropic 及 ComfyUI 的协议转换。
- **`db`**：持久化层，基于 SQLite (SQLx) 实现了联合 Schema 的自动化迁移。
- **`auth`**：身份安全层，负责 API Key 校验及管理员权限验证。
- **`protocols`**：协议定义层，涵盖 SSE 流式协议、A2A 总线及 MCP 客户端通信。

---

## 🛠 快速上手

### 环境要求
- Rust (1.80+)
- SQLite

### 安装与运行
1. **克隆并准备**：
   ```bash
   git clone <repository_url>
   cd lowart-api
   ```
2. **初始化配置**：
   应用首次运行会自动创建 `lowart.db` 并应用单一 Schema 迁移文件 `01_initial_schema.sql`。
3. **启动服务**：
   ```bash
   # 标准 HTTP 模式
   cargo run -p api-server
   # 查看详细日志
   RUST_LOG=debug cargo run -p api-server

   # 高性能 UDS 模式
   LISTEN_MODE=UDS cargo run -p api-server
   ```

### 管理端操作 (示例)
- **注册 MCP 工具服务器**：
  ```bash
  curl -X POST http://localhost:3000/admin/mcp/register \
    -H "Authorization: Bearer <ADMIN_KEY>" \
    -d '{"name": "os-tools", "command": "python3", "args": ["tools.py"]}'
  ```

---

## 📈 运维观测
访问 `http://localhost:3000/metrics` 即可获取 Prometheus 格式的实时监控数据，包括：
- `http_requests_total`: 请求总数
- `gateway_tokens_total`: 按模型统计的 Token 吞吐量
- `http_request_duration_seconds`: 响应耗时分布

## 🖥 Web 管理界面 (Admin UI)
我们提供了一个基于 Nuxt.js 构建的现代化管理控制台，位于 `lowart-admin` 目录：
- **功能**：可视化管理用户、配置模型、查看实时监控及在线聊天测试。
- **启动方式**：
  ```bash
  cd lowart-admin
  # 首次运行请安装依赖
  npm install
  # 启动开发服务器
  npm run dev
  ```
- **访问**：默认访问地址为 `http://localhost:3000`。

## 📑 使用教程
想要了解更多关于用户管理、模型配置及 API 调用细节？请阅读 [使用教程](./docs/tutorial-zh.md)。

## 📝 许可证
Apache 2.0 License
