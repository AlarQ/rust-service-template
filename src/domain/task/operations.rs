use std::sync::Arc;

use super::models::{Task, TaskId};
use crate::common::UserId;
use crate::domain::errors::DomainError;
use crate::domain::interfaces::task_repository::TaskRepository;

pub async fn get_task(id: TaskId, repo: Arc<dyn TaskRepository>) -> Result<Task, DomainError> {
    let result = repo.get(id).await?;
    result.ok_or_else(|| DomainError::not_found("Task", id.to_string()))
}

pub async fn list_tasks_by_user(
    user_id: UserId,
    repo: Arc<dyn TaskRepository>,
) -> Result<Vec<Task>, DomainError> {
    repo.get_by_user(user_id).await.map_err(DomainError::from)
}

pub async fn create_task(task: Task, repo: Arc<dyn TaskRepository>) -> Result<Task, DomainError> {
    repo.create(task).await.map_err(DomainError::from)
}
