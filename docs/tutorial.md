# Lowart-api User Tutorial

This guide will walk you through the basic configuration and daily usage of Lowart-api, including user management, model registration, and various API interaction methods.

---

## 1. Prerequisites

Before you begin, ensure the service is running:
```bash
cargo run -p api-server
```
Default listen address: `http://localhost:8080` (Note: Check logs for Port 8080 or 3000).

> [!NOTE]
> In a test environment, you can directly manipulate the SQLite database (`lowart.db`) for initialization. For production, it is recommended to use the Administrator APIs.

---

## 2. User Management (Admin API)

The system uses API Keys for authentication. The header format is `Authorization: Bearer <API_KEY>`.

### 2.1 Add User / Update Quota
Administrators can manage user limits via the `/admin/users/quota` endpoint.
- **Request Example**:
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

## 3. Model Configuration

Models must be registered in the database before they can be used for chat completions.

### 3.1 Core Field Description
| Field | Description | Example |
| :--- | :--- | :--- |
| `model_id` | Name used in client requests | `gpt-4o` |
| `vendor_type` | Adapter type | `OpenAI`, `Anthropic`, `ComfyUI`, `Mock` |
| `base_url` | Vendor API base URL | `https://api.openai.com/v1` |
| `api_key` | Vendor API Key | `sk-xxxx` |

### 3.2 Setting Up Fallbacks
You can configure automatic failover from `gpt-4o` to `gpt-3.5-turbo`.
- **Logic**: Associate the primary model ID with the fallback model ID in the `model_fallbacks` table.

---

## 4. Chat Completion Methods

Lowart-api is compatible with major LLM SDK request formats.

### 4.1 Standard Synchronous Request
- **Endpoint**: `POST /v1/chat/completions`
- **Payload**:
```json
{
  "model": "gpt-4o",
  "messages": [
    {"role": "user", "content": "Introduce the Rust language in one sentence."}
  ]
}
```
- **Response Example**:
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
        "content": "Rust is a modern systems programming language focused on safety, concurrency, and performance."
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

### 4.2 Streaming Request (SSE)
- **Payload**: Add `"stream": true`.
- **Note**: Tokens are pushed in real-time, and billing occurs asynchronously after the stream finishes.

### 4.3 Async Generation (Jobs)
Ideal for image generation or long-running inference.
- **Payload**: Add `"async": true`.
- **Response Example**:
```json
{
  "status": "async_started",
  "job_id": "job_uuid_abc",
  "model": "comfyui-stable-diffusion"
}
```
- **Query Progress**: `GET /v1/jobs/{job_id}`

---

## 5. Tool Calls & HITL

If the model suggests a tool call and the policy is set to `confirm`:

1. **Status**: Returns `require_confirmation` along with a `session_id`.
2. **Authorization**:
```bash
curl -X POST http://localhost:8080/v1/tools/confirm \
  -H "Authorization: Bearer <USER_KEY>" \
  -d '{
    "session_id": "sess_xyz",
    "approved_ids": ["call_id_1"]
  }'
```

---

## 6. Monitoring & Ops

### 6.1 View Real-time Metrics
Access `http://localhost:8080/metrics`.
- Use `gateway_tokens_total` to track consumption per model.
- Use `http_request_duration_seconds` to monitor latency.

---

## 💡 FAQ

**Q: Why am I getting a 429 response?**
- A: You've hit the RPM (Requests Per Minute) limit. Wait or adjust the user quota.

**Q: How do I enable UDS mode?**
- A: Set `LISTEN_MODE=UDS` as an environment variable and configure your reverse proxy (Nginx/Envoy) to point to the generated `.sock` file.
