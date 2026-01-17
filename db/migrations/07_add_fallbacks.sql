-- 模型降级关系表
CREATE TABLE IF NOT EXISTS model_fallbacks (
    id TEXT PRIMARY KEY,
    primary_model_id TEXT NOT NULL,
    fallback_model_id TEXT NOT NULL,
    priority INTEGER DEFAULT 0, -- 优先级，数字越小越先尝试
    trigger_condition TEXT DEFAULT 'error', -- 触发降级的条件: error, timeout, always
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (primary_model_id) REFERENCES model_configs(model_id),
    FOREIGN KEY (fallback_model_id) REFERENCES model_configs(model_id)
);

CREATE INDEX IF NOT EXISTS idx_fallback_primary ON model_fallbacks(primary_model_id);
