use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse,
};
use tower::ServiceExt;
use serde_json::{json, Value};
use std::sync::Arc;

use api_server::router::{AppState, create_router};
use lowart_core::{ModelManager, RhaiEngine, McpManager, AgentOrchestrator, CircuitBreaker};
use db::{DbConnection, UserRepo, ConfigRepo, FallbackRepo};


async fn setup_test_app() -> (axum::Router, Arc<DbConnection>) {
    // 1. 设置测试数据库 (使用临时文件)
    let db_path = format!("tests/ignore/test_{}.db", uuid::Uuid::new_v4());
    let db_url = format!("sqlite:{}?mode=rwc", db_path);
    
    let db = DbConnection::new_with_url(&db_url).await.expect("Failed to create test DB");

    let db_arc = Arc::new(db);

    // 2. 初始化 AppState
    let model_manager = Arc::new(ModelManager::new(Arc::clone(&db_arc)));
    let rhai_engine = Arc::new(RhaiEngine::new());
    let agent_orchestrator = Arc::new(AgentOrchestrator::new());
    let mcp_manager = Arc::new(McpManager::new(Arc::clone(&agent_orchestrator)));
    let rate_limit_cache = Arc::new(dashmap::DashMap::new());
    let user_cache = moka::future::Cache::builder()
        .max_capacity(100)
        .build();
    let circuit_breaker = Arc::new(CircuitBreaker::new(2, std::time::Duration::from_millis(100)));

    let state = AppState {
        model_manager: model_manager.clone(),
        rhai_engine,
        mcp_manager,
        agent_orchestrator,
        rate_limit_cache,
        user_cache,
        circuit_breaker,
    };

    // 3. 构建路由 (Mock Prometheus)
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install metrics recorder");
    });
    
    // 获取一个可渲染的 handle (虽然这里渲染不出来全局的，但 router 需要它)
    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .build_recorder()
        .handle();

    let app = create_router(state, metrics_handle);

    (app, db_arc)
}


#[tokio::test]
async fn test_auth_and_simple_chat() {
    let (app, db) = setup_test_app().await;

    // 1. 准备数据：创建一个测试用户和模型配置
    let api_key = "test-token-123";
    let user_id = "user-1";
    let user_repo = UserRepo::new(&db);
    user_repo.create(user_id, "testuser", api_key, false).await.unwrap();

    let config_repo = ConfigRepo::new(&db);
    config_repo.create(&db::ModelConfig {
        id: "m1".to_string(),
        title: "Mock Title".to_string(),
        model_id: "mock-model".to_string(),
        api_key: "any".to_string(),
        base_url: "any".to_string(),
        vendor_type: "Mock".to_string(),
        cost_per_1k_tokens: 0,
        request_script: None,
        response_script: None,
        is_active: true,
        created_at: chrono::Utc::now(),
    }).await.unwrap();

    // 2. 尝试无授权访问
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "mock-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // 3. 携带正确授权访问
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "mock-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["choices"][0]["message"]["content"], "Hello! I am a mock AI.");
}

#[tokio::test]
async fn test_model_fallback_logic() {
    let (app, db) = setup_test_app().await;
    let api_key = "test-token-fallback";
    UserRepo::new(&db).create("user-2", "user2", api_key, false).await.unwrap();

    // 配置一个必然失败的主模型和一个成功的备选模型
    let config_repo = ConfigRepo::new(&db);
    config_repo.create(&db::ModelConfig {
        id: "m-fail".to_string(),
        title: "Fail Title".to_string(),
        model_id: "fail-model".to_string(),
        api_key: "any".to_string(),
        base_url: "any".to_string(),
        vendor_type: "MockFail".to_string(),
        cost_per_1k_tokens: 0,
        request_script: None,
        response_script: None,
        is_active: true,
        created_at: chrono::Utc::now(),
    }).await.unwrap();

    config_repo.create(&db::ModelConfig {
        id: "m-success".to_string(),
        title: "Success Title".to_string(),
        model_id: "success-model".to_string(),
        api_key: "any".to_string(),
        base_url: "any".to_string(),
        vendor_type: "Mock".to_string(),
        cost_per_1k_tokens: 0,
        request_script: None,
        response_script: None,
        is_active: true,
        created_at: chrono::Utc::now(),
    }).await.unwrap();

    // 设置降级规则
    FallbackRepo::new(&db).add_fallback("fail-model", "success-model", 1).await.unwrap();

    // 发起请求
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "fail-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["choices"][0]["message"]["content"], "Hello! I am a mock AI.");
}

#[tokio::test]
async fn test_circuit_breaker_integration() {
    let (app, db) = setup_test_app().await;
    let api_key = "test-token-cb";
    UserRepo::new(&db).create("user-3", "user3", api_key, false).await.unwrap();

    let config_repo = ConfigRepo::new(&db);
    config_repo.create(&db::ModelConfig {
        id: "m-cb".to_string(),
        title: "CB Title".to_string(),
        model_id: "cb-model".to_string(),
        api_key: "any".to_string(),
        base_url: "any".to_string(),
        vendor_type: "MockFail".to_string(),
        cost_per_1k_tokens: 0,
        request_script: None,
        response_script: None,
        is_active: true,
        created_at: chrono::Utc::now(),
    }).await.unwrap();

    // 第 1 次请求: 应该返回 500 (模型失败)
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "cb-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 第 2 次请求: 应该返回 503 (因为阈值是 2，第 1 次失败后，第 2 次还是尝试但失败。
    // 注意：CircuitBreaker::is_allowed 在 Closed 状态下总是允许，只有失败数达到阈值后才会开。
    // 在我们的实现中，报第 2 次失败后，才会进入 Open。所以第 2 次请求应该还是 500。
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "cb-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // 第 3 次请求: 应该返回 503 (熔断触发)
    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "cb-model", "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_sse_billing_integration() {
    let (app, db) = setup_test_app().await;
    let api_key = "test-token-sse";
    UserRepo::new(&db).create("user-4", "user4", api_key, false).await.unwrap();

    ConfigRepo::new(&db).create(&db::ModelConfig {
        id: "m-sse".to_string(),
        title: "SSE Title".to_string(),
        model_id: "sse-model".to_string(),
        api_key: "any".to_string(),
        base_url: "any".to_string(),
        vendor_type: "Mock".to_string(),
        cost_per_1k_tokens: 100,
        request_script: None,
        response_script: None,
        is_active: true,
        created_at: chrono::Utc::now(),
    }).await.unwrap();

    let req = Request::builder()
        .uri("/v1/chat/completions")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({"model": "sse-model", "stream": true, "messages": [{"role": "user", "content": "hi"}]}).to_string()))
        .unwrap();
    
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    // 消耗流 (将会触发 TokenAccountingStream 的 drop/计费逻辑)
    let body = response.into_body();
    let _ = axum::body::to_bytes(body, 10 * 1024).await.unwrap();



    // 验证计费是否已更新
    // 注意：计费是异步进行的，使用重试机制等待更新
    let mut success = false;
    for _ in 0..10 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let user = UserRepo::new(&db).find_by_api_key(api_key).await.unwrap().unwrap();
        if user.token_used > 0 {
            success = true;
            break;
        }
    }
    assert!(success, "Billing was not updated after 1 second for SSE stream");
}

#[tokio::test]
async fn test_mcp_dynamic_registration() {
    let (app, db) = setup_test_app().await;
    // 创建管理员用户
    let api_key = "admin-token-mcp";
    UserRepo::new(&db).create("admin-1", "admin", api_key, true).await.unwrap();

    // 1. 动态注册一个 MCP Server (通过 Python 一个小脚本模拟响应 initialize 请求)
    let req = Request::builder()
        .uri("/admin/mcp/register")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "test-server",
            "command": "python3",
            "args": ["-c", "import sys, json; req=sys.stdin.readline(); print(json.dumps({'jsonrpc': '2.0', 'id': json.loads(req)['id'], 'result': {'protocolVersion': '2024-11-05', 'capabilities': {}, 'serverInfo': {'name': 'test', 'version': '1.0'}}}))"]
        }).to_string()))
        .unwrap();
    
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // 2. 尝试下线它
    let req = Request::builder()
        .uri("/admin/mcp/unregister")
        .method("POST")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .body(Body::from(json!({
            "name": "test-server"
        }).to_string()))
        .unwrap();
    
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}





