use axum::{Json, response::IntoResponse, extract::State};
use serde_json::json;
use crate::router::AppState;
use db::{UserRepo, ToolPolicyRepo};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateQuotaRequest {
    pub user_id: String,
    pub rpm_limit: i64,
    pub token_quota: i64,
}

#[derive(Deserialize)]
pub struct UpdatePolicyRequest {
    pub tool_name: String,
    pub user_id: Option<String>,
    pub policy: String, // auto, confirm, block
}

/// 获取所有用户列表
pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);
    match user_repo.list_all().await {
        Ok(users) => Json(users).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 更新用户配额
pub async fn update_user_quota(
    State(state): State<AppState>,
    Json(payload): Json<UpdateQuotaRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);
    match user_repo.update_quota(&payload.user_id, payload.rpm_limit, payload.token_quota).await {
        Ok(_) => Json(json!({"status": "success"})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 设置工具治理策略
pub async fn update_tool_policy(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePolicyRequest>
) -> impl IntoResponse {
    let db_conn = state.model_manager.db();
    let policy_repo = ToolPolicyRepo::new(&db_conn.pool);
    
    match policy_repo.upsert_policy(&payload.tool_name, payload.user_id.as_deref(), &payload.policy).await {
        Ok(_) => Json(json!({"status": "success"})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
