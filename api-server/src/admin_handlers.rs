use axum::{Json, response::IntoResponse, extract::{State, Extension}};
use serde_json::json;
use crate::router::AppState;
use db::{UserRepo, ToolPolicyRepo, ConfigRepo, StatsRepo, models::User, models::ModelConfig};
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

#[derive(Deserialize)]
pub struct RegisterMcpRequest {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Deserialize)]
pub struct UnregisterMcpRequest {
    pub name: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub api_key: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub api_key: String,
    pub is_admin: bool,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub user_id: String,
    pub username: String,
    pub api_key: String,
    pub status: String,
}

#[derive(Deserialize)]
pub struct DeleteUserRequest {
    pub user_id: String,
}

#[derive(Deserialize)]
pub struct CreateModelRequest {
    pub title: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: String,
    pub vendor_type: String,
    pub cost_per_1k_tokens: i64,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct UpdateModelRequest {
    pub id: String,
    pub title: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: String,
    pub vendor_type: String,
    pub cost_per_1k_tokens: i64,
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct DeleteModelRequest {
    pub id: String,
}

#[derive(Deserialize)]
pub struct CreateKeyRequest {
    pub user_id: String,
    pub label: String,
}

#[derive(Deserialize)]
pub struct ResetKeyRequest {
    pub key_id: i64,
}

#[derive(Deserialize)]
pub struct DeleteKeyRequest {
    pub key_id: i64,
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

/// 动态注册 MCP 客户端
pub async fn register_mcp(
    State(state): State<AppState>,
    Json(payload): Json<RegisterMcpRequest>
) -> impl IntoResponse {
    use protocols::stdio_mcp_client::StdioMcpClient;
    use protocols::mcp::{McpServerMeta, McpClient};
    use std::sync::Arc;

    let args_ref: Vec<&str> = payload.args.iter().map(|s| s.as_str()).collect();
    match StdioMcpClient::spawn(&payload.command, &args_ref).await {
        Ok(client) => {
            let client: Arc<dyn McpClient> = Arc::new(client);
            // 初始化

            let meta = McpServerMeta {
                name: payload.name.clone(),
                version: "1.0.0".to_string(),
                capabilities: json!({}),
            };

            if let Err(e) = client.initialize(meta).await {
                return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("MCP 初始化失败: {}", e)).into_response();
            }
            state.mcp_manager.register_client(payload.name, client).await;
            Json(json!({"status": "success"})).into_response()
        }
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("MCP 启动失败: {}", e)).into_response(),
    }
}

/// 动态注销 MCP 客户端
pub async fn unregister_mcp(
    State(state): State<AppState>,
    Json(payload): Json<UnregisterMcpRequest>
) -> impl IntoResponse {
    state.mcp_manager.unregister_client(&payload.name).await;
    Json(json!({"status": "success"})).into_response()
}

/// 获取所有模型列表
pub async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.model_manager.db();
    let config_repo = ConfigRepo::new(&db);
    match config_repo.list_all().await {
        Ok(configs) => Json(configs).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 获取最近调用日志
pub async fn list_stats(State(state): State<AppState>) -> impl IntoResponse {
    let db = state.model_manager.db();
    let stats_repo = StatsRepo::new(&db);
    match stats_repo.list_recent(100).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 登录验证 (校验 API Key 是否为管理员)
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);
    
    match user_repo.find_by_api_key(&payload.api_key).await {
        Ok(Some(user)) => {
            if user.is_admin {
                Json(json!({
                    "status": "success",
                    "user": user
                })).into_response()
            } else {
                (axum::http::StatusCode::FORBIDDEN, "仅限管理员登录").into_response()
            }
        }
        Ok(None) => (axum::http::StatusCode::UNAUTHORIZED, "无效的 API Key").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 创建新用户
pub async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);
    
    // 检查用户名唯一性
    match user_repo.exists_by_username(&payload.username, None).await {
        Ok(true) => return (axum::http::StatusCode::CONFLICT, "用户名已存在").into_response(),
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        _ => {}
    }

    let user_id = uuid::Uuid::new_v4().to_string();
    match user_repo.create(&user_id, &payload.username, &payload.api_key, payload.is_admin).await {
        Ok(_) => Json(json!({"status": "success", "user_id": user_id})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 更新用户信息 (用户名、API Key、状态)
pub async fn update_user(
    State(state): State<AppState>,
    Json(payload): Json<UpdateUserRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);

    // 检查用户名唯一性 (排除自己)
    match user_repo.exists_by_username(&payload.username, Some(&payload.user_id)).await {
        Ok(true) => return (axum::http::StatusCode::CONFLICT, "用户名已存在").into_response(),
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        _ => {}
    }

    match user_repo.update_info(&payload.user_id, &payload.username, &payload.api_key, &payload.status).await {
        Ok(_) => Json(json!({"status": "success"})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 删除用户
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<User>,
    Json(payload): Json<DeleteUserRequest>
) -> impl IntoResponse {
    // 保护根管理员
    if payload.user_id == "admin_root_id" {
        return (axum::http::StatusCode::FORBIDDEN, "根管理员不可删除").into_response();
    }

    // 防止删除自己
    if payload.user_id == current_user.id {
        return (axum::http::StatusCode::FORBIDDEN, "不能删除当前登录的账号").into_response();
    }

    let db = state.model_manager.db();
    let user_repo = UserRepo::new(&db);
    match user_repo.delete(&payload.user_id).await {
        Ok(_) => Json(json!({"status": "success"})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 创建新模型配置
pub async fn create_model(
    State(state): State<AppState>,
    Json(payload): Json<CreateModelRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let config_repo = ConfigRepo::new(&db);
    
    let config = ModelConfig {
        id: uuid::Uuid::new_v4().to_string(),
        title: payload.title,
        model_id: payload.model_id,
        api_key: payload.api_key,
        base_url: payload.base_url,
        vendor_type: payload.vendor_type,
        cost_per_1k_tokens: payload.cost_per_1k_tokens,
        request_script: None,
        response_script: None,
        is_active: payload.is_active,
        created_at: chrono::Utc::now(),
    };

    match config_repo.create(&config).await {
        Ok(_) => {
            state.model_manager.clear_cache().await;
            Json(json!({"status": "success", "id": config.id})).into_response()
        },
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 更新模型配置
pub async fn update_model(
    State(state): State<AppState>,
    Json(payload): Json<UpdateModelRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let config_repo = ConfigRepo::new(&db);
    
    let config = ModelConfig {
        id: payload.id,
        title: payload.title,
        model_id: payload.model_id,
        api_key: payload.api_key,
        base_url: payload.base_url,
        vendor_type: payload.vendor_type,
        cost_per_1k_tokens: payload.cost_per_1k_tokens,
        request_script: None,
        response_script: None,
        is_active: payload.is_active,
        created_at: chrono::Utc::now(), // In a real app, we might want to keep the original created_at
    };

    match config_repo.update(&config).await {
        Ok(_) => {
            state.model_manager.clear_cache().await;
            Json(json!({"status": "success"})).into_response()
        },
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 删除模型配置
pub async fn delete_model(
    State(state): State<AppState>,
    Json(payload): Json<DeleteModelRequest>
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let config_repo = ConfigRepo::new(&db);
    
    match config_repo.delete(&payload.id).await {
        Ok(_) => {
            state.model_manager.clear_cache().await;
            Json(json!({"status": "success"})).into_response()
        },
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 获取用户的所有 API Key
pub async fn list_user_keys(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let key_repo = db::ApiKeyRepo::new(&db);
    match key_repo.list_by_user(&user_id).await {
        Ok(keys) => Json(keys).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 为用户创建新的 API Key
pub async fn create_user_key(
    State(state): State<AppState>,
    Json(payload): Json<CreateKeyRequest>,
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let key_repo = db::ApiKeyRepo::new(&db);
    match key_repo.create(&payload.user_id, &payload.label).await {
        Ok(key) => Json(json!({"status": "success", "key": key})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 重置 API Key
pub async fn reset_user_key(
    State(state): State<AppState>,
    Json(payload): Json<ResetKeyRequest>,
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let key_repo = db::ApiKeyRepo::new(&db);

    // 1. 获取旧 Key 用于清除缓存
    if let Ok(Some(old_key)) = key_repo.find_by_id(payload.key_id).await {
        state.user_cache.invalidate(&old_key.api_key).await;
    }

    // 2. 执行重置
    match key_repo.reset(payload.key_id).await {
        Ok(key) => Json(json!({"status": "success", "key": key})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// 删除 API Key
pub async fn delete_user_key(
    State(state): State<AppState>,
    Json(payload): Json<DeleteKeyRequest>,
) -> impl IntoResponse {
    let db = state.model_manager.db();
    let key_repo = db::ApiKeyRepo::new(&db);

    // 1. 获取旧 Key 用于清除缓存
    if let Ok(Some(old_key)) = key_repo.find_by_id(payload.key_id).await {
        state.user_cache.invalidate(&old_key.api_key).await;
    }

    // 2. 执行删除
    match key_repo.delete(payload.key_id).await {
        Ok(_) => Json(json!({"status": "success"})).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
