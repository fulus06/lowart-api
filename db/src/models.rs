use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};


#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]

pub struct User {
    pub id: String,
    pub username: String,
    pub api_key: String, // TODO: Deprecate after migration
    pub status: String,
    pub rpm_limit: i64,
    pub token_quota: i64,
    pub token_used: i64,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: i64,
    pub user_id: String,
    pub api_key: String,
    pub label: String,
    pub status: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}



#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
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
    pub stat_type: String,
    pub timestamp: DateTime<Utc>,
}
