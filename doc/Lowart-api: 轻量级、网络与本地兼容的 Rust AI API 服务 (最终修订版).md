**Lowart-api: 轻量级、网络与本地兼容的 Rust AI API 服务 (最终修订版)**

**项目目标:**

*   提供一个轻量级、高性能的 AI API 服务。
*   支持多模式部署：灵活配置为非端口/本地调用（如 UDS）或标准网络端口调用。
*   使用 Rust 开发，确保内存安全和高并发。
*   集成 Rhai 脚本引擎，实现灵活的客户端参数转换和响应结果转换。
*   支持多种 AI 协议，包括 SSE、Agent-to-Agent (A2A) 和 Model Context Protocol (MCP)。
*   提供模块化和可扩展的 Rust 架构，支持未来功能扩展。
*   重点支持远程 AI API 调用和 ComfyUI API 接入。
*   引入用户管理、基于 SQLite 的持久化配置，**并添加用户请求/响应统计功能。**

---

**一、 Rust 项目架构设计**

为了支持用户使用统计，我们需要在 `db` crate 中增加统计相关的表，并在 `api-server` 和 `core` 中加入统计逻辑。

```
lowart-api/
├── Cargo.toml                 // Workspace 定义
├── api-server/                // 主应用，API 网关和服务器
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── config.rs
│       ├── router.rs
│       ├── handlers.rs
│       ├── auth_middleware.rs
│       ├── stats_middleware.rs // 统计中间件 (新增)
│       └── error.rs
├── core/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── agent_manager.rs
│       ├── context_manager.rs
│       ├── model_manager.rs
│       ├── protocol_handler.rs
│       ├── rhai_engine.rs
│       ├── request_context.rs
│       └── token_counter.rs   // Token 计算逻辑 (新增)
├── protocols/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── sse.rs
│       ├── a2a/
│       ├── mcp/
│       └── grpc/ (可选)
├── models/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── traits.rs
│       ├── openai_api.rs
│       ├── anthropic_api.rs
│       ├── comfyui_api.rs
│       └── remote_common.rs
├── db/                        // 数据库访问层 (新增统计相关表)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── schema.rs          // 包含 stats 表
│       ├── models.rs
│       ├── connection.rs
│       ├── user_repo.rs
│       ├── config_repo.rs
│       ├── stats_repo.rs      // 统计数据操作 (新增)
│       └── init.rs
├── auth/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── user.rs
│       ├── manager.rs
│       ├── token_manager.rs
│       └── permissions.rs
└── utils/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── logger.rs
```

**架构调整说明:**

*   **`api-server/stats_middleware.rs`:** 这是一个新的中间件，用于在请求进入和响应离开时捕获必要的数据（如用户ID、模型ID、请求体/响应体大小、时间戳）。
*   **`core/token_counter.rs`:** 负责实现不同模型（尤其是 LLM）的 Token 计算逻辑。例如，OpenAI 和 Anthropic 对 Token 的计算方式可能不同。
*   **`db/stats_repo.rs`:** 新增数据库操作，用于存储和查询用户的请求/响应统计数据。
*   **`db/schema.rs`:** 数据库 schema 将包含一个用于存储统计数据的表，例如 `usage_stats`。

---

**二、 详细功能列表 (更新与扩展)**

**1. 核心服务框架:**

*   **Rust async/await 框架:** 基于 Tokio，实现高并发、非阻塞 I/O。
*   **轻量级 HTTP/UDS 服务器:**
    *   基于 Actix-web, Warp, 或 Axum，提供高性能的服务。
    *   灵活的调用模式配置：支持 Unix Domain Sockets (UDS) 和标准网络端口调用。
*   **配置管理 (强化):**
    *   持久化配置存储：使用 **SQLite 数据库**作为主要的配置存储后端。
    *   加载优先级：文件配置 -> 数据库配置。
    *   支持热更新：服务运行时可以从数据库动态加载和更新配置。
*   **日志系统:** 使用 `tracing` 或 `log` crate，提供结构化日志输出。
*   **健康检查端点:** 提供 `/health` 或 `/status` 端点。

**2. 用户管理:**

*   **用户认证与鉴权:** 基于 API Key 的认证机制。
*   **用户数据存储:** 在 SQLite 数据库中存储用户数据。
*   **用户模型:** `ID` (UUID/Integer), `username` (String), `status` (Enum: Active, Inactive, Blocked)。
*   **管理 API (可选):** 提供独立的管理 API 或 CLI 工具来管理用户。

**3. 用户使用统计 (新增核心功能):**

*   **统计维度:**
    *   **按用户统计:** 每个用户的总请求/响应 Token、请求/响应次数。
    *   **按模型统计:** 针对每个模型（由 `model_id` 标识）的请求/响应 Token、请求/响应次数。
    *   **按时间维度:** 支持按天、周、月等粒度进行统计聚合。
*   **统计字段:**
    *   **`request_tokens` (Integer):** 每次请求中包含的 Token 数量。
    *   **`response_tokens` (Integer):** 每次响应中包含的 Token 数量。
    *   **`request_count` (Integer):** 每次请求记为 1 次。
    *   **`response_count` (Integer):** 每次成功响应记为 1 次。
    *   **`user_id` (关联用户):** 记录是哪个用户发起的请求。
    *   **`model_id` (关联模型):** 记录请求是哪个模型处理的。
    *   **`timestamp` (DateTime):** 请求发生的时间。
*   **Token 计算器:**
    *   集成 Token 计算库（例如，对于 OpenAI 模型使用 `tiktoken-rs` 或自定义实现）来准确计算不同模型文本的 Token 数量。
    *   对于图片生成或其他非文本模型，Token 统计可能不适用，可以按请求/响应次数计，或定义其他量化单位（例如图片数量）。
*   **数据存储:** 统计数据持久化到 SQLite 数据库中。
*   **统计 API (可选):** 提供 API 端点供管理员或用户查询自己的使用统计数据（例如 `/v1/usage/me`, `/v1/usage/models`）。

**4. API 接口设计 (包含 A2A/MCP):**

*   **RESTful API:** 提供标准的 RESTful 接口。
    *   **通用 AI 接口:** `/v1/chat/completions`, `/v1/images/generations` 等。
    *   **A2A 协议接口:** `/v1/agents/{agent_id}/message`。
    *   **MCP 协议接口:** `/v1/contexts/{context_id}` (管理上下文)，`/v1/contexts/{context_id}/query` (基于上下文查询)。
    *   **ComfyUI 特定接口:** `/v1/comfyui/run_workflow` 或 `/v1/comfyui/execute_prompt`。
    *   **用户管理 API (可选):** `/v1/users` (需高级权限)。
    *   **配置管理 API (可选):** `/v1/config` (需高级权限)。
    *   **使用统计查询 API (可选):** `/v1/usage` (需适当权限)。
*   **统一请求/响应格式:** 规范化请求体和响应体结构，通常采用 JSON 格式。
*   **错误处理:** 定义明确的错误码和错误信息。

**5. AI 协议支持:**

*   **SSE (Server-Sent Events) 支持:** 流式传输。
*   **Agent-to-Agent (A2A) Protocol 支持:** Agent 身份标识、消息路由、消息格式定义、Agent 行为协调、异步通信。
*   **Model Context Protocol (MCP) 支持:** 上下文存储与检索、上下文生命周期管理、上下文操作、语义搜索/检索增强生成 (RAG) 支持。
*   **WebSockets (可选):** 双向实时通信。
*   **GRPC (可选):** 根据性能和多语言集成需求。
*   **其他协议扩展性:** 插件化或模块化架构。

**6. Rhai 脚本引擎集成 (增强):**

*   **客户端参数转换:** 请求前转换。
*   **响应端结果转换:** 响应后转换。
*   **协议消息处理钩子:** 允许 Rhai 脚本介入 A2A 和 MCP 消息的处理流程。
*   **模型调用前/后钩子:** 允许 Rhai 脚本在调用远程 AI API 或 ComfyUI 前修改输入，或在它们返回结果后处理输出。
*   **Token 计算集成 (新增):** Rhai 脚本可以访问 Token 计算器，在自定义转换逻辑中获取 Token 数量，以便进行更精细的控制或统计（例如，统计 Rhai 脚本生成的额外 Token）。
*   安全沙箱、脚本配置、脚本版本管理 (未来扩展)。

**7. AI 模型集成:**

*   **模块化 AI 驱动:** 定义 `AiModel` trait。
*   **远程 AI 模型接入 (重点功能):** 通过 API 调用各种远程 AI 服务。可配置的认证和限流。统一接口适配。
*   **ComfyUI API 接入 (重点功能):** 工作流触发、参数传递、结果获取、状态查询。
*   **模型配置 (强化):**
    *   `title` (String), `model_id` (String), `api_key` (String, 加密存储), `base_url` (String), `vendor_type` (Enum)。
    *   **Token 成本定义 (新增):** 可以为每个模型配置 Token 的成本单价（例如，每千 Token 的美元价格），用于将来实现计费或配额管理。

**8. 安全与鉴权:**

*   API Key 鉴权 (基于用户管理)。
*   Agent 身份验证。
*   传输层安全 (HTTPS/UDS)。
*   敏感信息加密 (API Key 等)。

**9. 性能与资源管理:**

*   内存优化、并发控制、资源限制、性能监控。

**10. 工具链与开发体验:**

*   Cargo 构建、单元/集成测试、文档生成 (`rustdoc`)、命令行界面 (CLI) (用于用户管理、配置管理、数据库迁移、**查看统计**等)、Dockerfile、示例代码。

---

通过引入用户使用统计，Lowart-api 的功能集更加完善，能够更好地支持多用户、多模型场景下的资源管理和成本控制