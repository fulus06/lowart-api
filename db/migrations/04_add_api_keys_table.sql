-- 创建 API Key 表
CREATE TABLE IF NOT EXISTS api_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    api_key TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'Active',
    last_used_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, label)
);

-- 数据迁移：将 users 表中现有的 api_key 移动到新表，作为 "Default" Key
INSERT INTO api_keys (user_id, api_key, label, status)
SELECT id, api_key, 'Default', status FROM users;

-- 验证数据迁移是否成功（可选，这里仅作为备份逻辑思考）
-- 注意：此时不立即删除 users.api_key 字段，以保持平滑过渡，直到逻辑更新完成。
