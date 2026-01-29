pub mod models;
pub mod operations;

use std::sync::Arc;

use crate::domain::{
    errors::DomainError,
    interfaces::task_repository::TaskRepository,
};

/// Check if the task service is ready (database connectivity)
pub async fn check_readiness(
    repository: &Arc<dyn TaskRepository>,
) -> Result<(), DomainError> {
    repository.health_check().await
}
