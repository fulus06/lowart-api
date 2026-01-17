use axum::{
    extract::Request,

    middleware::Next,
    response::IntoResponse,
};
use std::time::Instant;
use metrics::{counter, histogram};
use crate::handlers::ModelId;

/// 指标中间件
/// 实现原理: 在请求进入和返回时分别记录时间戳，计算耗时。
/// 同时提取响应扩展中的 ModelId，关联用户 ID，记录多维度的监控指标。
pub async fn metrics_middleware(
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    // 尝试获取 ModelId 扩展 (如果有)
    let model_id = response.extensions()
        .get::<ModelId>()
        .map(|m| m.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // 记录通用指标
    counter!("http_requests_total", 
        "method" => method, 
        "path" => path, 
        "status" => status,
        "model" => model_id.clone()
    ).increment(1);

    histogram!("http_request_duration_seconds", 
        "model" => model_id
    ).record(latency);

    response
}
