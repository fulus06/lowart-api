use axum::{Json, response::IntoResponse, extract::State, Extension};
use axum::response::sse::{Event, Sse};
use serde::Deserialize;
use serde_json::{Value, json};
use db::{JobRepo, AsyncJob};
use utils::Result;

use std::sync::Arc;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::Stream;
use metrics::counter;


#[derive(Clone)]
pub struct ModelId(pub String);

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
    S: Stream<Item = Result<Value>> + Unpin 
{
    type Item = std::result::Result<Event, std::convert::Infallible>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(val))) => {
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
                let content = self.accumulated_content.clone();
                let user_id = self.user.id.clone();
                let model_id = self.model_id.clone();
                let req_tokens = self.req_tokens;
                let db = self.db.clone();

                tokio::spawn(async move {
                    use lowart_core::TokenCounter;
                    let res_tokens = TokenCounter::count_tokens(&content);
                    let total_tokens = (req_tokens + res_tokens) as i64;
                    
                    // 增加实时指标记录
                    counter!("gateway_tokens_total", "type" => "request", "model" => model_id.clone()).increment(req_tokens as u64);
                    counter!("gateway_tokens_total", "type" => "response", "model" => model_id.clone()).increment(res_tokens as u64);

                    let _ = db::UserRepo::new(&db).increment_token_usage(&user_id, total_tokens).await;
                    let _ = db::StatsRepo::new(&db).record_usage(&user_id, &model_id, req_tokens as i64, res_tokens as i64).await;
                });


                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

pub async fn chat_completions(
    State(state): State<crate::router::AppState>,
    Extension(user): Extension<db::User>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let model_id = match payload.get("model").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => return (axum::http::StatusCode::BAD_REQUEST, "Missing model").into_response(),
    };

    let (model, request_script, _response_script) = match state.model_manager.get_model_with_scripts(model_id).await {
        Ok(m) => m,
        Err(e) => return (axum::http::StatusCode::NOT_FOUND, e.to_string()).into_response(),
    };

    // 应用 Rhai 转换 (Request)
    let payload_val: Value = if let Some(script) = request_script {
        match state.rhai_engine.transform(&script, payload.clone()) {
            Ok(p) => p,
            Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        }
    } else {
        payload.clone()
    };

    let stream_mode = payload_val.get("stream").and_then(|s| s.as_bool()).unwrap_or(false);
    let async_mode = payload_val.get("async").and_then(|a| a.as_bool()).unwrap_or(false);

    // --- 处理异步任务模式 ---
    if async_mode {
        let job_id = uuid::Uuid::new_v4().to_string();
        let db_conn = state.model_manager.db();
        let job_repo = JobRepo::new(&db_conn.pool);
        
        let job = AsyncJob {
            job_id: job_id.clone(),
            user_id: user.id.clone(),
            status: "pending".to_string(),
            payload: Some(payload_val.to_string()),
            result: None,
            error: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if let Err(e) = job_repo.create_job(&job).await {
            return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
        }

        // 后端执行
        let user_id = user.id.clone();
        let model_id_str = model_id.to_string();
        let db_clone = Arc::clone(&db_conn);
        let model_clone = Arc::clone(&model);
        let payload_clone = payload_val.clone();
        let job_id_clone = job_id.clone();

        tokio::spawn(async move {
            let job_repo = JobRepo::new(&db_clone.pool);
            let _ = job_repo.update_status(&job_id_clone, "running", None, None).await;

            match model_clone.chat_completions(payload_clone).await {
                Ok(res) => {
                    let res_str = res.to_string();
                    let _ = job_repo.update_status(&job_id_clone, "completed", Some(&res_str), None).await;
                    
                    // Token 统计
                    use lowart_core::TokenCounter;
                    let req_tokens = TokenCounter::count_messages_tokens(payload_val.get("messages").unwrap_or(&json!([])));
                    let res_tokens = TokenCounter::count_tokens(res.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("message")).and_then(|m| m.get("content")).and_then(|c| c.as_str()).unwrap_or_default());

                    
                    let _ = db::UserRepo::new(&db_clone).increment_token_usage(&user_id, (req_tokens + res_tokens) as i64).await;
                    let _ = db::StatsRepo::new(&db_clone).record_usage(&user_id, &model_id_str, req_tokens as i64, res_tokens as i64).await;
                }
                Err(e) => {
                    let _ = job_repo.update_status(&job_id_clone, "failed", None, Some(&e.to_string())).await;
                }
            }
        });

        return Json(json!({
            "status": "async_started",
            "job_id": job_id
        })).into_response();
    }

    if stream_mode {
        match model.chat_completions_stream(payload_val.clone()).await {
            Ok(stream) => {
                use lowart_core::TokenCounter;
                let req_tokens = payload_val.get("messages")

                    .map(|m| TokenCounter::count_messages_tokens(m))
                    .unwrap_or(0);

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
        let mut current_payload = payload_val.clone();
        let mut total_req_tokens = 0;
        let mut total_res_tokens = 0;
        let max_iterations = 5;

        for _ in 0..max_iterations {
            match model.chat_completions(current_payload.clone()).await {
                Ok(res) => {
                    use lowart_core::TokenCounter;
                    if let Some(msgs) = current_payload.get("messages") {

                        total_req_tokens += TokenCounter::count_messages_tokens(msgs);
                    }

                    let choices = res.get("choices").and_then(|v| v.as_array());
                    let choice = choices.and_then(|a| a.get(0));
                    let message_obj = choice.and_then(|c| c.get("message"));
                    let tool_calls = message_obj.and_then(|m| m.get("tool_calls")).and_then(|t| t.as_array());

                    if let Some(calls) = tool_calls {
                        if !calls.is_empty() {
                            let db_conn = state.model_manager.db();
                            let policy_repo = db::ToolPolicyRepo::new(&db_conn.pool);
                            
                            let mut tool_results = Vec::new();
                            let mut requires_confirm = Vec::new();

                            for call in calls {
                                let call_id = call["id"].as_str().unwrap_or_default();
                                let tool_name = call["function"]["name"].as_str().unwrap_or_default();
                                let arguments = call["function"]["arguments"].clone();
                                
                                let policy = policy_repo.get_policy(tool_name, Some(&user.id)).await.unwrap_or_else(|_| "auto".to_string());
                                
                                match policy.as_str() {
                                    "block" => {
                                        tool_results.push(json!({
                                            "role": "tool",
                                            "tool_call_id": call_id,
                                            "name": tool_name,
                                            "content": "执行失败: 该工具已被审计策略禁用"
                                        }));
                                    },
                                    "confirm" => {
                                        requires_confirm.push(call.clone());
                                    },
                                    _ => {
                                        let result = match state.mcp_manager.call_tool_any(tool_name, arguments).await {
                                            Ok(out) => out.to_string(),
                                            Err(e) => format!("工具调用失败: {}", e),
                                        };
                                        tool_results.push(json!({
                                            "role": "tool",
                                            "tool_call_id": call_id,
                                            "name": tool_name,
                                            "content": result
                                        }));
                                    }
                                }
                            }

                            if !requires_confirm.is_empty() {
                                let session_id = uuid::Uuid::new_v4().to_string();
                                let session_repo = db::SessionRepo::new(&db_conn.pool);
                                
                                let mut messages = current_payload["messages"].as_array().unwrap().clone();
                                if let Some(m) = message_obj {
                                     messages.push(m.clone());
                                }
                                let mut save_payload = current_payload.clone();
                                save_payload["messages"] = json!(messages);

                                if let Err(e) = session_repo.save_session(
                                    &session_id, 
                                    &user.id, 
                                    model_id, 
                                    &save_payload, 
                                    &requires_confirm
                                ).await {
                                    tracing::error!("保存会话状态失败: {}", e);
                                    return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "保存授权上下文失败").into_response();
                                }

                                return Json(json!({
                                    "status": "require_confirmation",
                                    "session_id": session_id,
                                    "user_id": user.id,
                                    "model_id": model_id,
                                    "tool_calls": requires_confirm
                                })).into_response();
                            }

                            let mut msgs = current_payload["messages"].as_array().unwrap().clone();
                            if let Some(m) = message_obj {
                                msgs.push(m.clone());
                            }
                            msgs.extend(tool_results);
                            current_payload["messages"] = json!(msgs);
                            continue;
                        }
                    }

                    if let Some(choices_arr) = res.get("choices").and_then(|v| v.as_array()) {
                        if let Some(choice_first) = choices_arr.get(0) {
                            if let Some(m) = choice_first.get("message") {
                                if let Some(content) = m.get("content").and_then(|v| v.as_str()) {
                                    total_res_tokens += lowart_core::TokenCounter::count_tokens(content);
                                }
                            }
                        }
                    }


                    let user_id = user.id.clone();
                    let model_repo_id = model_id.to_string();
                    let db = state.model_manager.db();
                    tokio::spawn(async move {
                         // 增加实时指标记录
                         counter!("gateway_tokens_total", "type" => "request", "model" => model_repo_id.clone()).increment(total_req_tokens as u64);
                         counter!("gateway_tokens_total", "type" => "response", "model" => model_repo_id.clone()).increment(total_res_tokens as u64);

                         let _ = db::UserRepo::new(&db).increment_token_usage(&user_id, (total_req_tokens + total_res_tokens) as i64).await;
                         let _ = db::StatsRepo::new(&db).record_usage(&user_id, &model_repo_id, total_req_tokens as i64, total_res_tokens as i64).await;
                    });


                    let mut axum_res = Json(res).into_response();
                    axum_res.extensions_mut().insert(ModelId(model_id.to_string()));
                    return axum_res;
                },
                Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            }
        }
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Max iterations reached").into_response()
    }
}

pub async fn confirm_tool_call(
    State(state): State<crate::router::AppState>,
    Extension(user): Extension<db::User>,
    Json(payload): Json<ToolConfirmRequest>
) -> impl IntoResponse {
    let db_conn = state.model_manager.db();
    let session_repo = db::SessionRepo::new(&db_conn.pool);

    let session = match session_repo.load_session(&payload.session_id).await {
        Ok(Some(s)) => s,
        Ok(None) => return (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response(),
        Err(e) => return (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    if session.user_id != user.id {
        return (axum::http::StatusCode::FORBIDDEN, "Forbidden").into_response();
    }

    let mut current_payload: Value = serde_json::from_str(&session.payload).unwrap();
    let pending_calls: Vec<Value> = serde_json::from_str(&session.pending_tool_calls).unwrap();

    let mut tool_results = Vec::new();
    let mut messages = current_payload["messages"].as_array().unwrap().clone();
    
    for call in pending_calls {
        let call_id = call["id"].as_str().unwrap_or_default();
        let tool_name = call["function"]["name"].as_str().unwrap_or_default();
        let arguments = call["function"]["arguments"].clone();

        if payload.approved_ids.contains(&call_id.to_string()) {
            let result = match state.mcp_manager.call_tool_any(tool_name, arguments).await {
                Ok(out) => out.to_string(),
                Err(e) => format!("Error: {}", e),
            };
            tool_results.push(json!({
                "role": "tool",
                "tool_call_id": call_id,
                "name": tool_name,
                "content": result
            }));
        } else {
            tool_results.push(json!({
                "role": "tool",
                "tool_call_id": call_id,
                "name": tool_name,
                "content": "Rejected by user"
            }));
        }
    }

    messages.extend(tool_results);
    current_payload["messages"] = json!(messages);

    let _ = session_repo.delete_session(&payload.session_id).await;

    (axum::http::StatusCode::OK, Json(json!({
        "status": "success",
        "next_payload": current_payload
    }))).into_response()
}

#[derive(Deserialize)]
pub struct ToolConfirmRequest {
    pub session_id: String,
    pub approved_ids: Vec<String>,
}

pub async fn get_job(
    State(state): State<crate::router::AppState>,
    Extension(user): Extension<db::User>,
    axum::extract::Path(job_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let db_conn = state.model_manager.db();
    let job_repo = JobRepo::new(&db_conn.pool);

    match job_repo.find_by_id(&job_id).await {
        Ok(Some(job)) => {
            if job.user_id != user.id {
                return (axum::http::StatusCode::FORBIDDEN, "Forbidden").into_response();
            }
            Json(job).into_response()
        }
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Job not found").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn list_jobs(
    State(state): State<crate::router::AppState>,
    Extension(user): Extension<db::User>,
) -> impl IntoResponse {
    let db_conn = state.model_manager.db();
    let job_repo = JobRepo::new(&db_conn.pool);

    match job_repo.list_by_user(&user.id).await {
        Ok(jobs) => Json(jobs).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({"status": "ok"}))
}
