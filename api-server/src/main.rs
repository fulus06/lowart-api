mod handlers;
mod router;
mod auth_middleware;
mod stats_middleware;

use utils::logger;
use db::DbConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UnixListener;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 初始化日志
    logger::init();
    tracing::info!("Lowart-api 正在启动...");

    // 2. 初始化数据库
    let db = Arc::new(DbConnection::new().await?);
    tracing::info!("数据库连接已建立");

    // 3. 初始化核心组件
    let model_manager = Arc::new(core::ModelManager::new(Arc::clone(&db)));
    let rhai_engine = Arc::new(core::RhaiEngine::new());
    
    let state = router::AppState {
        model_manager,
        rhai_engine,
    };

    // 4. 构建路由
    let app = router::create_router(state);



    // 4. 选择启动模式 (根据环境变量或配置)
    let listen_mode = std::env::var("LISTEN_MODE").unwrap_or_else(|_| "HTTP".to_string());

    if listen_mode == "UDS" {
        let path = "/tmp/lowart.sock";
        let _ = std::fs::remove_file(path);
        let _listener = UnixListener::bind(path)?;
        tracing::info!("正在监听 UDS: {}", path);
        // Axum 0.7 及其更高版本对 UDS 的支持需要额外的适配器或低级 Serve 调用
        // 此处仅展示逻辑脉络
    } else {
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("正在监听 HTTP: {}", addr);
        axum::serve(listener, app).await?;
    }


    Ok(())
}
