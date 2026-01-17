use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use utils::Result;
use std::env;

/// 数据库连接管理
/// 实现原理: 使用 SQLx 的 SqlitePool 维护连接池，支持并发访问。
pub struct DbConnection {
    pub pool: SqlitePool,
}

impl DbConnection {
    /// 初始化数据库连接
    /// 尝试从环境变量 `DATABASE_URL` 获取连接字符串，默认为 `sqlite:lowart.db`
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:lowart.db?mode=rwc".to_string());

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;

        // 执行初始化脚本
        let schema = include_str!("../schema.sql");
        sqlx::query(schema).execute(&pool).await?;

        Ok(Self { pool })
    }
}
