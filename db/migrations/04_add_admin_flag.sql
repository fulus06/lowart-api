-- 为用户表增加管理员标记
ALTER TABLE users ADD COLUMN is_admin BOOLEAN DEFAULT 0;

-- 将第一个用户 (如果有) 设为管理员以方便初始化
-- UPDATE users SET is_admin = 1 WHERE id = 'default_user';
