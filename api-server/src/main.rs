mod handlers;
mod router;
mod auth_middleware;
mod stats_middleware;
mod limit_middleware;
mod admin_handlers;
mod admin_middleware;

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
    let agent_orchestrator = Arc::new(core::AgentOrchestrator::new());
    let mcp_manager = Arc::new(core::McpManager::new(Arc::clone(&agent_orchestrator)));
    let rate_limit_cache = Arc::new(dashmap::DashMap::new());

    
    let state = router::AppState {
        model_manager,
        rhai_engine,
        mcp_manager,
        agent_orchestrator,
        rate_limit_cache,
    };




    // 4. 构建路由
    let app = router::create_router(state);



    // 4. 选择启动模式
    let uds_path = std::env::var("UDS_PATH").ok();

    if let Some(path) = uds_path {
        // UDS 模式
        let _ = std::fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        tracing::info!("正在监听 UDS: {}", path);

        use hyper_util::server::conn::auto::Builder;
        use hyper_util::rt::TokioExecutor;
        use tower::Service;

        loop {
            let (stream, _addr) = listener.accept().await?;
            let app_clone = app.clone();

            tokio::spawn(async move {
                let io = hyper_util::rt::TokioIo::new(stream);
                let service = app_clone.clone();
                
                let tower_service = tower::service_fn(move |req: axum::http::Request<hyper::body::Incoming>| {
                    let mut service = service.clone();
                    async move {
                        let req = req.map(axum::body::Body::new);
                        service.call(req).await
                    }
                });

                let hyper_service = hyper_util::service::TowerToHyperService::new(tower_service);

                if let Err(err) = Builder::new(TokioExecutor::new())
                    .serve_connection(io, hyper_service)
                    .await
                {
                    tracing::error!("UDS 连接处理错误: {}", err);
                }
            });

        }

    } else {
        // TCP 模式
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or(8080);
            
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("正在监听 HTTP: {}", addr);
        axum::serve(listener, app).await?;
    }




    Ok(())
}
