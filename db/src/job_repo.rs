use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AsyncJob {
    pub job_id: String,
    pub user_id: String,
    pub status: String,
    pub payload: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct JobRepo<'a> {
    db: &'a sqlx::SqlitePool,
}

impl<'a> JobRepo<'a> {
    pub fn new(db: &'a sqlx::SqlitePool) -> Self {
        Self { db }
    }

    pub async fn create_job(&self, job: &AsyncJob) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO async_jobs (job_id, user_id, status, payload) VALUES (?, ?, ?, ?)"
        )
        .bind(&job.job_id)
        .bind(&job.user_id)
        .bind(&job.status)
        .bind(&job.payload)
        .execute(self.db).await?;
        Ok(())
    }

    pub async fn update_status(&self, job_id: &str, status: &str, result: Option<&str>, error: Option<&str>) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE async_jobs SET status = ?, result = ?, error = ?, updated_at = CURRENT_TIMESTAMP WHERE job_id = ?"
        )
        .bind(status)
        .bind(result)
        .bind(error)
        .bind(job_id)
        .execute(self.db).await?;
        Ok(())
    }

    pub async fn find_by_id(&self, job_id: &str) -> anyhow::Result<Option<AsyncJob>> {
        let job = sqlx::query_as::<_, AsyncJob>("SELECT * FROM async_jobs WHERE job_id = ?")
            .bind(job_id)
            .fetch_optional(self.db).await?;
        Ok(job)
    }

    pub async fn list_by_user(&self, user_id: &str) -> anyhow::Result<Vec<AsyncJob>> {
        let jobs = sqlx::query_as::<_, AsyncJob>("SELECT * FROM async_jobs WHERE user_id = ? ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(self.db).await?;
        Ok(jobs)
    }
}
