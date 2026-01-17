use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    extract::State,
};
use auth::AuthManager;
use crate::router::AppState;

/// 身份认证中间件
/// 实现逻辑: 从 Authorization Header 提取 API Key 并通过 AuthManager 校验。
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "));

    match auth_header {
        Some(api_key) => {
            let manager = AuthManager::new(state.model_manager.db()); // 假设 ModelManager 暴露了 db()
            match manager.authenticate(api_key).await {
                Ok(_user) => Ok(next.run(req).await),
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

