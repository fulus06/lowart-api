# Lowart-api 使用教程 (Tutorial)

本教程将引导您完成 Lowart-api 的基本配置与日常使用，包括用户管理、模型注册以及各种 API 调用方式。

---

## 1. 基础准备

在开始之前，请确保服务已启动：
```bash
cargo run -p api-server
```
默认监听地址：`http://localhost:8080` (注意：根据启动日志，当前端口可能为 8080 或 3000)。

> [!NOTE]
> 在测试环境中，您可以直接操作 SQLite 数据库 (`lowart.db`) 进行初始化。在正式使用中，建议通过管理员 API 进行管理。

---

## 2. 用户管理 (Admin API)

系统使用 API Key 进行认证，API 请求头格式为 `Authorization: Bearer <API_KEY>`。

### 2.1 添加用户/更新配额
管理员可以通过 `/admin/users/quota` 接口管理用户限额。
- **请求示例**：
```bash
curl -X POST http://localhost:8080/admin/users/quota \
  -H "Authorization: Bearer <ADMIN_KEY>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_123",
    "rpm_limit": 100,
    "token_quota": 5000000
  }'
```

---

## 3. 模型配置 (Model Configuration)

在使用对话功能前，必须在数据库中注册模型。

### 3.1 核心字段说明
| 字段 | 说明 | 示例 |
| :--- | :--- | :--- |
| `model_id` | 客户端请求时使用的名称 | `gpt-4o` |
| `vendor_type` | 适配器类型 | `OpenAI`, `Anthropic`, `ComfyUI`, `Mock` |
| `base_url` | 供应商 API 基础地址 | `https://api.openai.com/v1` |
| `api_key` | 供应商密钥 | `sk-xxxx` |

### 3.2 设置模型降级 (Fallback)
您可以设置当 `gpt-4o` 故障时自动降级到 `gpt-3.5-turbo`。
- **逻辑**：在 `model_fallbacks` 表中关联主模型 ID 与备用模型 ID。

---

## 4. 对话请求方式 (Chat Completions)

Lowart-api 兼容主流 LLM SDK 的调用方式。

### 4.1 标准同步请求
- **请求方法**：`POST /v1/chat/completions`
- **请求体**：
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "user", "content": "用一句话介绍 Rust 语言。"}
  ]
}
```
- **返回示例**：
```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4o",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Rust 是一门专注于安全、并发和性能的现代化系统编程语言。"
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 15,
    "completion_tokens": 20,
    "total_tokens": 35
  }
}
```

### 4.2 流式请求 (SSE)
- **请求体**：增加 `"stream": true`。
- **说明**：系统会实时推送 Token，并在流结束后异步计费。

### 4.3 异步生成请求 (Jobs)
适用于图片生成或长耗时推理。
- **请求体**：增加 `"async": true`。
- **返回示例**：
```json
{
  "status": "async_started",
  "job_id": "job_uuid_abc",
  "model": "comfyui-stable-diffusion"
}
```
- **查询进度**：`GET /v1/jobs/{job_id}`

---

## 5. 工具调用与人机协同 (Tools & HITL)

如果模型建议调用工具，且该工具策略为 `confirm`：

1. **响应状态**：返回 `require_confirmation` 及 `session_id`。
2. **人工授权**：
```bash
curl -X POST http://localhost:8080/v1/tools/confirm \
  -H "Authorization: Bearer <USER_KEY>" \
  -d '{
    "session_id": "sess_xyz",
    "approved_ids": ["call_id_1"]
  }'
```

---

## 6. 监控与运维

### 6.1 查看实时指标
访问 `http://localhost:8080/metrics` 即可获取。
- 关注 `gateway_tokens_total` 了解各模型消耗情况。
- 关注 `http_request_duration_seconds` 了解延迟。

---

## 💡 常见问题 (FAQ)

**Q: 为什么请求返回 429？**
- A: 您触发了 RPM (每分钟请求数) 限制，请稍后再试或调整用户配额。

**Q: 如何启用 UDS 模式？**
- A: 启动时设置环境变量 `LISTEN_MODE=UDS`，并在反代 (Nginx/Envoy) 中配置指向生成的 `.sock` 文件。
- A: 使用`UDS_PATH='/tmp/lowart.sock'`设置`.sock`文件
