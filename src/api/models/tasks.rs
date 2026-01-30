use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::task::models::{Task, TaskPriority, TaskStatus};

// Schema types for OpenAPI documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = TaskStatus)]
pub enum TaskStatusSchema {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[schema(as = TaskPriority)]
pub enum TaskPrioritySchema {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TaskResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    #[schema(value_type = TaskStatusSchema)]
    pub status: TaskStatus,
    #[schema(value_type = TaskPrioritySchema)]
    pub priority: TaskPriority,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id.to_string(),
            user_id: task.user_id.to_string(),
            title: task.title.into_inner(),
            description: task.description,
            status: task.status,
            priority: task.priority,
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
            completed_at: task
                .completed_at
                .map(|dt: chrono::DateTime<chrono::Utc>| dt.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    #[schema(value_type = TaskPrioritySchema)]
    pub priority: Option<TaskPriority>,
}

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListTasksQuery {
    pub user_id: Option<String>,
}
