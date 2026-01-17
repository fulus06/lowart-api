-- 创建工具治理策略表
CREATE TABLE IF NOT EXISTS tool_policies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tool_name TEXT NOT NULL,
    policy TEXT NOT NULL CHECK(policy IN ('auto', 'confirm', 'block')),
    user_id TEXT, -- 如果为 NULL，则为全局策略
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 创建索引以加速检索
CREATE INDEX IF NOT EXISTS idx_tool_policies_name ON tool_policies(tool_name);
CREATE INDEX IF NOT EXISTS idx_tool_policies_user ON tool_policies(user_id);

-- 插入一些默认示例策略
-- 假设 'google_search' 是白名单
-- INSERT INTO tool_policies (tool_name, policy) VALUES ('google_search', 'auto');
-- 假设 'delete_file' 强制要求确认
-- INSERT INTO tool_policies (tool_name, policy) VALUES ('delete_file', 'confirm');
