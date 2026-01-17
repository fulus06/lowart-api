use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    extract::State,
};
use db::User;
use chrono::Utc;

use crate::router::AppState;

/// 速率限制与配额检查中间件
/// 实现逻辑: 
/// 1. 从 Request Extensions 中提取用户信息。
/// 2. 检查用户的 RPM (每分钟请求数) 是否超限。
/// 3. 检查用户的 Token 配额是否已用完。
pub async fn limit_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. 提取用户信息
    let user = req.extensions()
        .get::<User>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 2. RPM 速率限制检查
    let now = Utc::now();
    let minute_ts = now.timestamp() / 60; // 当前分钟的时间戳
    let key = (user.id.clone(), minute_ts);

    let count = {
        let mut entry = state.rate_limit_cache.entry(key).or_insert(0);
        *entry += 1;
        *entry
    };

    if count > user.rpm_limit {
        tracing::warn!("用户 {} 请求过快: {}/{}", user.username, count, user.rpm_limit);
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    // 3. 配额检查 (Token Quota)
    if user.token_used >= user.token_quota {
        tracing::warn!("用户 {} 配额已用完: {}/{}", user.username, user.token_used, user.token_quota);
        return Err(StatusCode::PAYMENT_REQUIRED); // 或者使用 403
    }

    // 定期清理过期的缓存 (这里简单处理，实际生产中应有后台任务清理)
    // if state.rate_limit_cache.len() > 10000 { ... }

    Ok(next.run(req).await)
}
