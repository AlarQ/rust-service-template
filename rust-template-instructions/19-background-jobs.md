# Background Jobs Pattern

[← DTO Conversion](18-dto-conversion.md) | [Next: Event Schema →](20-event-schema.md)

---
ONLY WHEN REPOSITORY FUNCTIONALITY NEEDS THAT
## Job Entity

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: JobId,
    pub user_id: UserId,
    pub status: JobStatus,
    pub total_items: usize,
    pub processed_items: usize,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl Job {
    pub fn new(user_id: UserId, total_items: usize) -> Self {
        let now = Utc::now();
        Self {
            id: JobId(Uuid::new_v4()),
            user_id,
            status: JobStatus::Pending,
            total_items,
            processed_items: 0,
            error_message: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.total_items == 0 {
            return 0.0;
        }
        (self.processed_items as f64 / self.total_items as f64) * 100.0
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled
        )
    }
}
```

---

## Job Repository Interface

```rust
use async_trait::async_trait;

#[async_trait]
pub trait JobRepository: Send + Sync + std::fmt::Debug {
    async fn create(&self, job: Job) -> Result<Job, JobError>;
    async fn get(&self, id: JobId) -> Result<Option<Job>, JobError>;
    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Job>, JobError>;
    async fn update_status(&self, id: JobId, status: JobStatus) -> Result<(), JobError>;
    async fn update_progress(&self, id: JobId, processed: usize) -> Result<(), JobError>;
    async fn mark_completed(&self, id: JobId) -> Result<(), JobError>;
    async fn mark_failed(&self, id: JobId, error: String) -> Result<(), JobError>;
    async fn cleanup_old_jobs(&self, older_than: DateTime<Utc>) -> Result<usize, JobError>;
}
```

---

## Job Management Service

```rust
use std::sync::Arc;
use tokio::sync::mpsc;

pub async fn start_job(
    user_id: UserId,
    items: Vec<Item>,
    job_repo: Arc<dyn JobRepository>,
    processor: Arc<dyn ItemProcessor>,
) -> Result<JobId, JobError> {
    // Create job record
    let job = Job::new(user_id.clone(), items.len());
    let job = job_repo.create(job).await?;
    let job_id = job.id.clone();

    // Spawn background task
    tokio::spawn(async move {
        process_job(job_id.clone(), items, job_repo, processor).await;
    });

    Ok(job_id)
}

async fn process_job(
    job_id: JobId,
    items: Vec<Item>,
    job_repo: Arc<dyn JobRepository>,
    processor: Arc<dyn ItemProcessor>,
) {
    // Update status to processing
    if let Err(e) = job_repo.update_status(job_id.clone(), JobStatus::Processing).await {
        tracing::error!("Failed to update job status: {}", e);
        return;
    }

    let mut processed = 0;

    for item in items {
        match processor.process(item).await {
            Ok(_) => {
                processed += 1;
                // Update progress periodically (every 10 items or so)
                if processed % 10 == 0 {
                    let _ = job_repo.update_progress(job_id.clone(), processed).await;
                }
            }
            Err(e) => {
                tracing::error!("Failed to process item: {}", e);
                let _ = job_repo.mark_failed(job_id.clone(), e.to_string()).await;
                return;
            }
        }
    }

    // Mark as completed
    if let Err(e) = job_repo.mark_completed(job_id.clone()).await {
        tracing::error!("Failed to mark job as completed: {}", e);
    }
}
```

---

## Job Status Endpoint

```rust
#[utoipa::path(
    get,
    path = "/v1/jobs/{job_id}",
    security(("jwt" = [])),
    responses(
        (status = 200, body = JobStatusResponse),
        (status = 404, body = ApiErrorResponse),
    )
)]
pub async fn get_job_status(
    State(app_state): State<Arc<AppState>>,
    Path(job_id): Path<Uuid>,
    JwtExtractor(claims): JwtExtractor,
) -> Result<JobStatusResponse, ApiErrorResponse> {
    let job = app_state
        .job_repo
        .get(JobId(job_id))
        .await?
        .ok_or_else(|| ApiErrorResponse::not_found("Job", job_id.to_string()))?;

    // Verify user owns this job
    claims.validate_user_id(job.user_id.0)?;

    Ok(job.into())
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct JobStatusResponse {
    pub id: Uuid,
    pub status: String,
    pub progress_percentage: f64,
    pub total_items: usize,
    pub processed_items: usize,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
```

---

[← DTO Conversion](18-dto-conversion.md) | [Next: Event Schema →](20-event-schema.md)
