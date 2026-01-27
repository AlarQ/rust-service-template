use async_trait::async_trait;
use std::fmt::Debug;

use crate::common::UserId;
use crate::domain::task::models::{Task, TaskId};

#[async_trait]
pub trait TaskRepository: Send + Sync + Debug {
    async fn create(&self, entity: Task) -> Result<Task, sqlx::Error>;
    async fn get(&self, id: TaskId) -> Result<Option<Task>, sqlx::Error>;
    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Task>, sqlx::Error>;
    async fn update(&self, entity: &Task) -> Result<(), sqlx::Error>;
    async fn delete(&self, id: TaskId) -> Result<(), sqlx::Error>;
    async fn health_check(&self) -> Result<(), sqlx::Error>;
}
