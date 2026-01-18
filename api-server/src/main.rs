use api_server::router;


use metrics_exporter_prometheus::PrometheusBuilder;

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

    // 2. 初始化监控 指标 (Prometheus)
    let metrics_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("无法安装 Prometheus recorder");
    tracing::info!("指标监控系统 (Prometheus) 已启动");

    // 3. 初始化数据库
    let db = Arc::new(DbConnection::new().await?);
    tracing::info!("数据库连接已建立");

    // 4. 初始化核心组件
    let model_manager = Arc::new(lowart_core::ModelManager::new(Arc::clone(&db)));
    let rhai_engine = Arc::new(lowart_core::RhaiEngine::new());
    let agent_orchestrator = Arc::new(lowart_core::AgentOrchestrator::new());
    let mcp_manager = Arc::new(lowart_core::McpManager::new(Arc::clone(&agent_orchestrator)));
    let rate_limit_cache = Arc::new(dashmap::DashMap::new());
    let user_cache = moka::future::Cache::builder()
        .max_capacity(1000)
        .time_to_live(std::time::Duration::from_secs(600)) // 10分钟过期
        .build();
    let circuit_breaker = Arc::new(lowart_core::CircuitBreaker::new(5, std::time::Duration::from_secs(30)));
    
    let state = router::AppState {
        model_manager,
        rhai_engine,
        mcp_manager,
        agent_orchestrator,
        rate_limit_cache,
        user_cache,
        circuit_breaker,
    };




    // 5. 构建路由
    let app = router::create_router(state, metrics_handle);



    // 5. 选择启动模式
    let listen_mode = std::env::var("LISTEN_MODE").unwrap_or_default();

    if listen_mode == "UDS" {
        // UDS 模式
        let path = std::env::var("UDS_PATH").unwrap_or_else(|_| "/tmp/lowart.sock".to_string());
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
