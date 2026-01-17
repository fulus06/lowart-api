use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use utils::Result;
use std::env;

/// 数据库连接管理
/// 实现原理: 使用 SQLx 的 SqlitePool 维护连接池，支持并发访问。
pub struct DbConnection {
    pub pool: SqlitePool,
}

impl DbConnection {
    /// 初始化数据库连接并执行迁移
    pub async fn new() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:lowart.db?mode=rwc".to_string());

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await?;

        // 执行数据库迁移
        tracing::info!("正在执行数据库迁移...");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await?;
        tracing::info!("数据库迁移完成");

        Ok(Self { pool })
    }
}


