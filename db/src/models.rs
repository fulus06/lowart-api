use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub api_key: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ModelConfig {
    pub id: String,
    pub title: String,
    pub model_id: String,
    pub api_key: String,
    pub base_url: String,
    pub vendor_type: String,
    pub cost_per_1k_tokens: i64,
    pub request_script: Option<String>,
    pub response_script: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}


#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageStat {
    pub id: i64,
    pub user_id: String,
    pub model_id: String,
    pub request_tokens: i64,
    pub response_tokens: i64,
    pub request_count: i64,
    pub response_count: i64,
    pub duration_ms: i64,
    pub timestamp: DateTime<Utc>,
}
