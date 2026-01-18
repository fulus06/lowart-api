-- 添加请求类型字段到使用统计表
ALTER TABLE usage_stats ADD COLUMN stat_type TEXT NOT NULL DEFAULT 'unknown';
