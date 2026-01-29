use async_trait::async_trait;
use std::fmt::Debug;

use crate::{
    common::UserId,
    domain::{
        errors::DomainError,
        task::models::{Task, TaskId},
    },
};

#[async_trait]
pub trait TaskRepository: Send + Sync + Debug {
    async fn create(&self, entity: Task) -> Result<Task, DomainError>;
    async fn get(&self, id: TaskId) -> Result<Option<Task>, DomainError>;
    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Task>, DomainError>;
    async fn update(&self, entity: &Task) -> Result<(), DomainError>;
    async fn delete(&self, id: TaskId) -> Result<(), DomainError>;
    async fn health_check(&self) -> Result<(), DomainError>;
}
