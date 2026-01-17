-- 为用户表增加配额与限速字段
ALTER TABLE users ADD COLUMN rpm_limit INTEGER DEFAULT 60;        -- 每分钟请求数
ALTER TABLE users ADD COLUMN token_quota INTEGER DEFAULT 1000000;  -- 总 Token 配额 (默认100万)
ALTER TABLE users ADD COLUMN token_used INTEGER DEFAULT 0;         -- 已使用 Token
