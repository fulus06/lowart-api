-- 为系统初始化一个默认管理员
INSERT INTO users (id, username, api_key, status, is_admin, rpm_limit, token_quota)
VALUES (
    'admin_root_id',
    'admin',
    'admin_default_key',
    'Active',
    1,
    999999,
    100000000
) ON CONFLICT(id) DO NOTHING;
