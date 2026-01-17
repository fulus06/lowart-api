use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use db::User;

/// 管理员鉴权中间件
/// 实现逻辑: 从请求扩展中提取已认证的用户对象，检查其 `is_admin` 标记。
pub async fn admin_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // 获取 auth_middleware 已经解析出来的用户
    let user = req.extensions().get::<User>().ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.is_admin {
        tracing::warn!("用户 {} 尝试访问管理接口被拒绝", user.id);
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
