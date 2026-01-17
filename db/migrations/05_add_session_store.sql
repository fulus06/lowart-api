-- 创建工具确认会话表
CREATE TABLE IF NOT EXISTS tool_confirm_sessions (
    session_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    payload TEXT NOT NULL, -- 序列化后的请求全文 (包含消息历史)
    pending_tool_calls TEXT NOT NULL, -- 待授权执行的工具调用列表 (JSON 数组)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL -- 会话有效期 (建议 1 小时)
);

-- 创建索引以加速检索和清理
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON tool_confirm_sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_sessions_user ON tool_confirm_sessions(user_id);
