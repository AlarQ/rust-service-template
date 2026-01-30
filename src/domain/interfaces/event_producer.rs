use async_trait::async_trait;

use crate::domain::{errors::DomainError, task::models::events::TaskEvent};

#[async_trait]
pub trait EventProducer: Send + Sync {
    async fn publish_task_event(&self, event: TaskEvent) -> Result<(), DomainError>;
}
