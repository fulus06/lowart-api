-- Lowart-api 统一数据库初始架构 (Consolidated Schema)

-- 1. 用户表
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,          -- UUID
    username TEXT NOT NULL UNIQUE,
    api_key TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,         -- Active, Inactive, Blocked
    is_admin BOOLEAN DEFAULT 0,    -- 管理员标记
    rpm_limit INTEGER DEFAULT 60,   -- 每分钟请求数
    token_quota INTEGER DEFAULT 1000000, -- 总 Token 配额 (分)
    token_used INTEGER DEFAULT 0,  -- 已使用 Token (分)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 2. 模型配置表
CREATE TABLE IF NOT EXISTS model_configs (
    id TEXT PRIMARY KEY,          -- UUID
    title TEXT NOT NULL,
    model_id TEXT NOT NULL UNIQUE, -- 如 gpt-4, claude-3 (必须唯一以供外键引用)
    api_key TEXT NOT NULL,        -- 加密存储的 API Key
    base_url TEXT NOT NULL,       -- 厂商 API 地址
    vendor_type TEXT NOT NULL,    -- OpenAI, Anthropic, ComfyUI, Mock
    cost_per_1k_tokens INTEGER DEFAULT 0, -- 每千 Token 成本 (分)
    request_script TEXT,          -- 请求转换脚本 (Rhai)
    response_script TEXT,         -- 响应转换脚本 (Rhai)
    is_active BOOLEAN DEFAULT 1,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 3. 使用统计表
CREATE TABLE IF NOT EXISTS usage_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    request_tokens INTEGER DEFAULT 0,
    response_tokens INTEGER DEFAULT 0,
    request_count INTEGER DEFAULT 1,
    response_count INTEGER DEFAULT 0,
    duration_ms INTEGER DEFAULT 0, -- 请求耗时
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id)
);

-- 4. 工具治理策略表
CREATE TABLE IF NOT EXISTS tool_policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_name TEXT NOT NULL,
    policy TEXT NOT NULL CHECK(policy IN ('auto', 'confirm', 'block')),
    user_id TEXT, -- 如果为 NULL，则为全局策略
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_tool_policies_name ON tool_policies(tool_name);
CREATE INDEX IF NOT EXISTS idx_tool_policies_user ON tool_policies(user_id);

-- 5. 工具确认会话表 (HITL)
CREATE TABLE IF NOT EXISTS tool_confirm_sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    payload TEXT NOT NULL, -- 序列化后的请求全文
    pending_tool_calls TEXT NOT NULL, -- JSON 数组
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON tool_confirm_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON tool_confirm_sessions(user_id);

-- 6. 异步任务追踪表
CREATE TABLE IF NOT EXISTS async_jobs (
    job_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL, -- pending, running, completed, failed
    payload TEXT,
    result TEXT,
    error TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_jobs_user ON async_jobs(user_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON async_jobs(status);

-- 7. 模型降级关系表 (Fallback)
CREATE TABLE IF NOT EXISTS model_fallbacks (
    id TEXT PRIMARY KEY,
    primary_model_id TEXT NOT NULL,
    fallback_model_id TEXT NOT NULL,
    priority INTEGER DEFAULT 0, -- 优先级
    trigger_condition TEXT DEFAULT 'error',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (primary_model_id) REFERENCES model_configs(model_id),
    FOREIGN KEY (fallback_model_id) REFERENCES model_configs(model_id)
);
CREATE INDEX IF NOT EXISTS idx_fallback_primary ON model_fallbacks(primary_model_id);
