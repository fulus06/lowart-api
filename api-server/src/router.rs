use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::handlers;
use crate::admin_handlers;
use crate::auth_middleware::auth_middleware;
use crate::limit_middleware::limit_middleware;
use crate::admin_middleware::admin_middleware;
use crate::stats_middleware::stats_middleware;
use crate::metrics_middleware::metrics_middleware;
use metrics_exporter_prometheus::PrometheusHandle;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

use lowart_core::{ModelManager, RhaiEngine};
use std::sync::Arc;


/// 全局应用状态
#[derive(Clone)]
pub struct AppState {
    pub model_manager: Arc<ModelManager>,
    pub rhai_engine: Arc<RhaiEngine>,
    pub mcp_manager: Arc<lowart_core::McpManager>,
    pub agent_orchestrator: Arc<lowart_core::AgentOrchestrator>,
    pub rate_limit_cache: Arc<dashmap::DashMap<(String, i64), i64>>, // (user_id, minute_timestamp) -> count
    pub user_cache: moka::future::Cache<String, db::User>, // api_key -> user
    pub circuit_breaker: Arc<lowart_core::CircuitBreaker>,
}





pub fn create_router(state: AppState, metrics_handle: PrometheusHandle) -> Router {
    // 公开接口
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/admin/login", post(admin_handlers::login))
        .route("/metrics", get(move || async move {
            metrics_handle.render()
        }));

    // 管理接口 (需 Auth + Admin 权限)
    let admin_routes = Router::new()
        .route("/users", 
            get(admin_handlers::list_users)
            .post(admin_handlers::create_user)
            .put(admin_handlers::update_user)
            .delete(admin_handlers::delete_user)
        )
        .route("/users/quota", post(admin_handlers::update_user_quota))
        .route("/models", 
            get(admin_handlers::list_models)
            .post(admin_handlers::create_model)
            .put(admin_handlers::update_model)
            .delete(admin_handlers::delete_model)
        )
        .route("/stats", get(admin_handlers::list_stats))
        .route("/policies", post(admin_handlers::update_tool_policy))
        .route("/mcp/register", post(admin_handlers::register_mcp))
        .route("/mcp/unregister", post(admin_handlers::unregister_mcp))
        .layer(middleware::from_fn(admin_middleware));


    // 标准 API 接口 (需 Auth)
    let api_routes = Router::new()
        .route("/chat/completions", post(handlers::chat_completions))
        .route("/tools/confirm", post(handlers::confirm_tool_call))
        .route("/jobs", get(handlers::list_jobs))
        .route("/jobs/{id}", get(handlers::get_job))
        .layer(middleware::from_fn_with_state(state.clone(), limit_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), stats_middleware));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET, 
            Method::POST, 
            Method::PUT, 
            Method::DELETE, 
            Method::OPTIONS,
            Method::PATCH,
        ])
        .allow_headers(Any);

    // 合并受保护的接口并应用鉴权中间件
    let protected_routes = Router::new()
        .nest("/admin", admin_routes)
        .nest("/v1", api_routes)
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(middleware::from_fn_with_state(state.clone(), metrics_middleware))
        .layer(cors)
        .with_state(state)
}

