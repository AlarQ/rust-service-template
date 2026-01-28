use async_trait::async_trait;
use sqlx::PgPool;
use std::{convert::TryFrom, fmt::Debug};
use uuid::Uuid;

use crate::{
    common::UserId,
    domain::{
        errors::DomainError,
        interfaces::task_repository::TaskRepository,
        task::models::{Task, TaskId, TaskPriority, TaskStatus},
    },
};

#[derive(Clone)]
pub struct PostgresTaskRepository {
    pool: PgPool,
}

impl Debug for PostgresTaskRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresTaskRepository")
            .field("pool", &"PgPool")
            .finish()
    }
}

impl PostgresTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for PostgresTaskRepository {
    async fn create(&self, entity: Task) -> Result<Task, DomainError> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            "#,
        )
        .bind(entity.id.into_inner())
        .bind(entity.user_id.into_inner())
        .bind(entity.title.into_inner())
        .bind(&entity.description)
        .bind(TaskStatusDb::from(entity.status))
        .bind(TaskPriorityDb::from(entity.priority))
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .bind(entity.completed_at)
        .fetch_one(&self.pool)
        .await
        .map_err(DomainError::from)
        .and_then( Task::try_from)
    }

    async fn get(&self, id: TaskId) -> Result<Option<Task>, DomainError> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id.into_inner())
        .fetch_optional(&self.pool)
        .await
        .map_err(DomainError::from)
        .and_then(|row| {
            row.map(Task::try_from)
                .transpose()
        })
    }

    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Task>, DomainError> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            FROM tasks
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.into_inner())
        .fetch_all(&self.pool)
        .await
        .map_err(DomainError::from)
        .and_then(|rows| {
            rows.into_iter()
                .map(Task::try_from)
                .collect::<Result<Vec<_>, _>>()
        })
    }

    async fn update(&self, entity: &Task) -> Result<(), DomainError> {
        sqlx::query(
            r#"
            UPDATE tasks
            SET title = $2, description = $3, status = $4, priority = $5, updated_at = $6, completed_at = $7
            WHERE id = $1
            "#,
        )
        .bind(entity.id.into_inner())
        .bind(entity.title.value())
        .bind(&entity.description)
        .bind(TaskStatusDb::from(entity.status))
        .bind(TaskPriorityDb::from(entity.priority))
        .bind(entity.updated_at)
        .bind(entity.completed_at)
        .execute(&self.pool)
        .await
        .map_err(DomainError::from)?;
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<(), DomainError> {
        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id.into_inner())
            .execute(&self.pool)
            .await
            .map_err(DomainError::from)?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), DomainError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(DomainError::from)?;
        Ok(())
    }
}

// Infrastructure-specific enum types for database mapping
#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "task_status", rename_all = "SCREAMING_SNAKE_CASE")]
enum TaskStatusDb {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, sqlx::Type)]
#[sqlx(type_name = "task_priority", rename_all = "SCREAMING_SNAKE_CASE")]
enum TaskPriorityDb {
    Low,
    Medium,
    High,
    Critical,
}

impl From<TaskStatusDb> for TaskStatus {
    fn from(status: TaskStatusDb) -> Self {
        match status {
            TaskStatusDb::Pending => TaskStatus::Pending,
            TaskStatusDb::InProgress => TaskStatus::InProgress,
            TaskStatusDb::Completed => TaskStatus::Completed,
            TaskStatusDb::Cancelled => TaskStatus::Cancelled,
        }
    }
}

impl From<TaskStatus> for TaskStatusDb {
    fn from(status: TaskStatus) -> Self {
        match status {
            TaskStatus::Pending => TaskStatusDb::Pending,
            TaskStatus::InProgress => TaskStatusDb::InProgress,
            TaskStatus::Completed => TaskStatusDb::Completed,
            TaskStatus::Cancelled => TaskStatusDb::Cancelled,
        }
    }
}

impl From<TaskPriorityDb> for TaskPriority {
    fn from(priority: TaskPriorityDb) -> Self {
        match priority {
            TaskPriorityDb::Low => TaskPriority::Low,
            TaskPriorityDb::Medium => TaskPriority::Medium,
            TaskPriorityDb::High => TaskPriority::High,
            TaskPriorityDb::Critical => TaskPriority::Critical,
        }
    }
}

impl From<TaskPriority> for TaskPriorityDb {
    fn from(priority: TaskPriority) -> Self {
        match priority {
            TaskPriority::Low => TaskPriorityDb::Low,
            TaskPriority::Medium => TaskPriorityDb::Medium,
            TaskPriority::High => TaskPriorityDb::High,
            TaskPriority::Critical => TaskPriorityDb::Critical,
        }
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    status: TaskStatusDb,
    priority: TaskPriorityDb,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TryFrom<TaskRow> for Task {
    type Error = DomainError;

    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        use crate::domain::task::models::Title;

        Ok(Self {
            id: TaskId::from(row.id),
            user_id: UserId::from(row.user_id),
            title: Title::new(row.title).map_err(|e| {
                DomainError::external_error(format!(
                    "Invalid title data in database: {}. This indicates data corruption or migration issue.",
                    e
                ))
            })?,
            description: row.description,
            status: row.status.into(),
            priority: row.priority.into(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            completed_at: row.completed_at,
        })
    }
}
