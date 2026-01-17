use axum::{Json, response::IntoResponse, extract::State, Extension};


use serde_json::{Value, json};


use utils::Result;
use axum::response::sse::{Event, Sse};
use std::sync::Arc;
use crate::router::AppState;

#[derive(Clone)]
pub struct ModelId(pub String);



use std::pin::Pin;


use std::task::{Context, Poll};
use futures::Stream;

struct TokenAccountingStream<S> {
    inner: S,
    user: db::User,
    model_id: String,
    req_tokens: usize,
    accumulated_content: String,
    db: Arc<db::DbConnection>,
}

impl<S> Stream for TokenAccountingStream<S> 
where 
    S: Stream<Item = Result<Value, utils::Error>> + Unpin 
{
    type Item = Result<Event, std::convert::Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(val))) => {
                // 累加内容 (适配 OpenAI 格式)
                if let Some(choices) = val.get("choices").and_then(|v| v.as_array()) {
                    if let Some(delta) = choices.get(0).and_then(|c| c.get("delta")) {
                        if let Some(content) = delta.get("content").and_then(|t| t.as_str()) {
                            self.accumulated_content.push_str(content);
                        }
                    }
                }
                Poll::Ready(Some(Ok(Event::default().data(val.to_string()))))
            }
            Poll::Ready(Some(Err(e))) => {
                Poll::Ready(Some(Ok(Event::default().event("error").data(e.to_string()))))
            }
            Poll::Ready(None) => {
                // 流结束，触发异步计费与统计
                let content = std::mem::take(&mut self.accumulated_content);
                let user = self.user.clone();
                let model_id = self.model_id.clone();
                let req_tokens = self.req_tokens;
                let db = Arc::clone(&self.db);
                
                tokio::spawn(async move {
                    use core::TokenCounter;
                    let res_tokens = TokenCounter::count_tokens(&content);
                    let total = req_tokens + res_tokens;
                    
                    if total > 0 {
                        let user_repo = db::UserRepo::new(&db);
                        let _ = user_repo.increment_token_usage(&user.id, total as i64).await;
                    }
                    
                    let stats_repo = db::StatsRepo::new(&db);
                    let _ = stats_repo.record(db::UsageStat {
                        id: 0,
                        user_id: user.id,
                        model_id,
                        request_tokens: req_tokens as i64,
                        response_tokens: res_tokens as i64,
                        request_count: 1,
                        response_count: 1,
                        duration_ms: 0,
                        timestamp: chrono::Utc::now(),
                    }).await;
                });
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// AI 请求处理器
/// 实现逻辑: 执行请求格式转换 -> 路由到目标模型 -> 执行响应转换。
/// 支持“流式输出”：如果请求中包含 `stream: true`，则返回 SSE。
pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(user): Extension<db::User>,
    Json(payload): Json<Value>
) -> impl IntoResponse {
    let model_manager = &state.model_manager;
    let rhai_engine = &state.rhai_engine;

    let model_id = payload.get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("gpt-4");

    // 1. 获取动态模型适配器及转换脚本
    let (model, req_script, res_script) = match model_manager.get_model_with_scripts(model_id).await {
        Ok(res) => res,
        Err(e) => return (axum::http::StatusCode::NOT_FOUND, e.to_string()).into_response(),
    };

    // 2. 注入 MCP 工具 (如果模型支持)

    // 聚合所有 MCP 客户端的工具并转换为 OpenAI 格式
    let mut final_payload = payload.clone();
    if let Ok(mcp_tools) = state.mcp_manager.list_all_tools().await {
        if !mcp_tools.is_empty() {
            let mut openai_tools = Vec::new();
            for tool in mcp_tools {
                openai_tools.push(json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description,
                        "parameters": tool.input_schema
                    }
                }));
            }
            
            // 合并到 payload
            if let Some(existing_tools) = final_payload.get_mut("tools") {
                if let Some(tools_arr) = existing_tools.as_array_mut() {
                    tools_arr.extend(openai_tools);
                }
            } else {
                final_payload["tools"] = json!(openai_tools);
            }
        }
    }

    // 3. 执行请求转换 (如果配置了脚本)

    if let Some(script) = req_script {
        match rhai_engine.transform(&script, final_payload) {
            Ok(new_payload) => final_payload = new_payload,
            Err(e) => return (axum::http::StatusCode::BAD_REQUEST, format!("请求转换失败: {}", e)).into_response(),
        }
    }


    // 3. 判断是否为流式请求
    let is_stream = final_payload.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);

    if is_stream {
        match model.chat_completions_stream(final_payload.clone()).await {
            Ok(stream) => {
                use core::TokenCounter;
                let req_tokens = if let Some(msgs) = final_payload.get("messages") {
                    TokenCounter::count_messages_tokens(msgs)
                } else {
                    0
                };

                let accounting_stream = TokenAccountingStream {
                    inner: stream,
                    user: user.clone(),
                    model_id: model_id.to_string(),
                    req_tokens,
                    accumulated_content: String::new(),
                    db: state.model_manager.db(),
                };
                let mut res = Sse::new(accounting_stream).into_response();
                res.extensions_mut().insert(ModelId(model_id.to_string()));
                res
            },
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        match model.chat_completions(final_payload.clone()).await {
            Ok(res) => {
                // 4. 执行响应转换 (如果配置了脚本)
                let mut final_res_val = res.clone();
                if let Some(script) = res_script {
                    match rhai_engine.transform(&script, final_res_val) {
                        Ok(new_res) => final_res_val = new_res,
                        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("响应转换失败: {}", e)).into_response(),
                    }
                }

                // 5. 计费逻辑 (非流式)
                let db = state.model_manager.db();
                let model_id_str = model_id.to_string();
                let payload_clone = final_payload.clone();
                let res_json = res.clone(); // 这里是原始响应用于计算 token
                
                tokio::spawn(async move {
                    use core::TokenCounter;
                    // 计算请求 Token (简单处理消息列表)
                    let req_tokens = if let Some(msgs) = payload_clone.get("messages") {
                        TokenCounter::count_messages_tokens(msgs)
                    } else {
                        0
                    };
                    
                    // 计算响应 Token
                    let res_tokens = if let Some(choices) = res_json.get("choices").and_then(|v| v.as_array()) {
                       if let Some(content) = choices.get(0).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|t| t.as_str()) {
                           TokenCounter::count_tokens(content)
                       } else {
                           0
                       }
                    } else {
                        0
                    };

                    let total = req_tokens + res_tokens;
                    if total > 0 {
                        let user_repo = db::UserRepo::new(&db);
                        let _ = user_repo.increment_token_usage(&user.id, total as i64).await;
                    }

                    // 记录详细统计
                    let stats_repo = db::StatsRepo::new(&db);
                    let _ = stats_repo.record(db::UsageStat {
                        id: 0,
                        user_id: user.id,
                        model_id: model_id_str,
                        request_tokens: req_tokens as i64,
                        response_tokens: res_tokens as i64,
                        request_count: 1,
                        response_count: 1,
                        duration_ms: 0, // 这里的时长可以后续优化
                        timestamp: chrono::Utc::now(),
                    }).await;
                });

                let mut response = Json(final_res_val).into_response();
                response.extensions_mut().insert(ModelId(model_id.to_string()));
                response
            },
            Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    }
}




pub async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
