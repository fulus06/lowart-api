-- 创建异步任务追踪表
CREATE TABLE IF NOT EXISTS async_jobs (
    job_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL, -- pending, running, completed, failed
    payload TEXT, -- 原始请求
    result TEXT, -- 任务结果 (JSON)
    error TEXT, -- 错误信息
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_jobs_user ON async_jobs(user_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON async_jobs(status);
