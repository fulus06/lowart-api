use axum::{
    body::Body,
    http::Request,
    middleware::Next,
    response::Response,
    extract::State,
};
use std::time::Instant;
use db::{StatsRepo, UsageStat};
use chrono::Utc;
use crate::router::AppState;

/// 使用统计中间件
/// 实现原理: 在请求处理前后记录时间，并异步将统计信息写入 SQLite。
pub async fn stats_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let user_id = req.extensions().get::<db::User>().map(|u| u.id.clone()).unwrap_or_else(|| "system".to_string());

    // 处理请求
    let response = next.run(req).await;
    let duration = start.elapsed().as_millis() as i64;

    // 从响应扩展中提取 ModelId
    let model_id = response.extensions().get::<crate::handlers::ModelId>()
        .map(|m| m.0.clone())
        .unwrap_or_else(|| "unknown".to_string());
    
    // 异步记录统计
    let db = state.model_manager.db();
    tokio::spawn(async move {
        let repo = StatsRepo::new(&db);
        let stat = UsageStat {
            id: 0, // 自动递增
            user_id,
            model_id,
            request_tokens: 0,
            response_tokens: 0,
            request_count: 1,
            response_count: 1,
            duration_ms: duration,
            timestamp: Utc::now(),
        };
        if let Err(e) = repo.record(stat).await {
            tracing::error!("记录统计数据失败: {}", e);
        }
    });

    
    response
}
