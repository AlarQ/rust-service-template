use std::sync::Arc;

use super::models::{Task, TaskId};
use crate::{
    common::UserId,
    domain::{errors::DomainError, interfaces::task_repository::TaskRepository},
};

/// Retrieve a task by ID
///
/// Returns an error if the task is not found.
pub async fn get_task(id: TaskId, repo: Arc<dyn TaskRepository>) -> Result<Task, DomainError> {
    let result: Option<Task> = repo.get(id).await?;
    result.ok_or_else(|| DomainError::not_found("Task", id.to_string()))
}

/// List all tasks for a user
///
/// Returns tasks ordered by creation date (newest first).
pub async fn list_tasks_by_user(
    user_id: UserId,
    repo: Arc<dyn TaskRepository>,
) -> Result<Vec<Task>, DomainError> {
    repo.get_by_user(user_id).await
}

/// Create a new task
///
/// Validates business rules:
/// - Task title must be valid (enforced by Title value object)
/// - No duplicate task validation (can be added if needed)
pub async fn create_task(task: Task, repo: Arc<dyn TaskRepository>) -> Result<Task, DomainError> {
    // Business rule: Task creation is validated through the Task::new constructor
    // which ensures title is valid and other invariants are met.
    // Additional business rules can be added here:
    // - Check for duplicate tasks (same title for same user)
    // - Enforce maximum tasks per user
    // - Validate user permissions

    repo.create(task).await
}
