use axum::{
    routing::{get, post},
    Router,
    middleware,
};
use crate::handlers::{chat_completions, health_check};
use crate::auth_middleware::auth_middleware;
use crate::stats_middleware::stats_middleware;
use core::{ModelManager, RhaiEngine};
use std::sync::Arc;

/// 全局应用状态
#[derive(Clone)]
pub struct AppState {
    pub model_manager: Arc<ModelManager>,
    pub rhai_engine: Arc<RhaiEngine>,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/chat/completions", post(chat_completions))
        // 应用中间件 (通过 from_fn_with_state 注入 AppState)
        .layer(middleware::from_fn_with_state(state.clone(), stats_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}



