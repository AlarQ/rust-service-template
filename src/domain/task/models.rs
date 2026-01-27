use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::UserId;
use crate::domain::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct TaskId(pub Uuid);

impl TaskId {
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Uuid> for TaskId {
    fn from(id: Uuid) -> Self {
        Self(id)
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema, Default,
)]
#[sqlx(type_name = "task_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema, Default,
)]
#[sqlx(type_name = "task_priority", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Title(pub String);

impl Title {
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 200;

    pub fn new(value: String) -> Result<Self, DomainError> {
        let trimmed = value.trim().to_string();
        if trimmed.len() < Self::MIN_LENGTH {
            return Err(DomainError::field_validation_error(
                "title",
                "Title cannot be empty",
            ));
        }
        if trimmed.len() > Self::MAX_LENGTH {
            return Err(DomainError::field_validation_error(
                "title",
                format!("Title cannot exceed {} characters", Self::MAX_LENGTH),
            ));
        }
        Ok(Self(trimmed))
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub user_id: UserId,
    pub title: Title,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(
        user_id: UserId,
        title: String,
        description: Option<String>,
        priority: TaskPriority,
    ) -> Result<Self, DomainError> {
        let now = Utc::now();
        Ok(Self {
            id: TaskId::new(),
            user_id,
            title: Title::new(title)?,
            description: description
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
            status: TaskStatus::Pending,
            priority,
            created_at: now,
            updated_at: now,
            completed_at: None,
        })
    }
}
