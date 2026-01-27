use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;
use uuid::Uuid;

use crate::common::UserId;
use crate::domain::interfaces::task_repository::TaskRepository;
use crate::domain::task::models::{Task, TaskId, TaskPriority, TaskStatus};

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
    async fn create(&self, entity: Task) -> Result<Task, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            "#,
        )
        .bind(entity.id.0)
        .bind(entity.user_id.0)
        .bind(entity.title.0.clone())
        .bind(&entity.description)
        .bind(entity.status)
        .bind(entity.priority)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .bind(entity.completed_at)
        .fetch_one(&self.pool)
        .await
        .map(Task::from)
    }

    async fn get(&self, id: TaskId) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            FROM tasks
            WHERE id = $1
            "#,
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(Task::from))
    }

    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, user_id, title, description, status, priority, created_at, updated_at, completed_at
            FROM tasks
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.0)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Task::from).collect())
    }

    async fn update(&self, entity: &Task) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE tasks
            SET title = $2, description = $3, status = $4, priority = $5, updated_at = $6, completed_at = $7
            WHERE id = $1
            "#,
        )
        .bind(entity.id.0)
        .bind(entity.title.0.as_str())
        .bind(&entity.description)
        .bind(entity.status)
        .bind(entity.priority)
        .bind(entity.updated_at)
        .bind(entity.completed_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: TaskId) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM tasks WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    user_id: Uuid,
    title: String,
    description: Option<String>,
    status: TaskStatus,
    priority: TaskPriority,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl From<TaskRow> for Task {
    fn from(row: TaskRow) -> Self {
        use crate::domain::task::models::Title;

        Self {
            id: TaskId(row.id),
            user_id: UserId(row.user_id),
            title: Title(row.title),
            description: row.description,
            status: row.status,
            priority: row.priority,
            created_at: row.created_at,
            updated_at: row.updated_at,
            completed_at: row.completed_at,
        }
    }
}
