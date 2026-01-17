-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,          -- UUID
    username TEXT NOT NULL UNIQUE,
    api_key TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,         -- Active, Inactive, Blocked
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 模型配置表
CREATE TABLE IF NOT EXISTS model_configs (
    id TEXT PRIMARY KEY,          -- UUID
    title TEXT NOT NULL,
    model_id TEXT NOT NULL,       -- 如 gpt-4, claude-3
    api_key TEXT NOT NULL,        -- 加密存储的 API Key
    base_url TEXT NOT NULL,       -- 厂商 API 地址
    vendor_type TEXT NOT NULL,    -- OpenAI, Anthropic, ComfyUI
    cost_per_1k_tokens INTEGER DEFAULT 0, -- 每千 Token 成本 (分)
    request_script TEXT,          -- 请求转换脚本 (Rhai)
    response_script TEXT,         -- 响应转换脚本 (Rhai)
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);


-- 使用统计表
CREATE TABLE IF NOT EXISTS usage_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    request_tokens INTEGER DEFAULT 0,
    response_tokens INTEGER DEFAULT 0,
    request_count INTEGER DEFAULT 1,
    response_count INTEGER DEFAULT 0,
    duration_ms INTEGER DEFAULT 0,  -- 请求耗时
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id)
);
