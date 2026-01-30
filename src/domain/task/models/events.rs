use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::UserId,
    domain::task::models::{TaskId, TaskPriority, TaskStatus},
};

/// Event types for task lifecycle events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskEventType {
    Created,
    Updated,
    Deleted,
}

/// Data payload for task events containing task fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEventData {
    pub id: TaskId,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Metadata for event tracking and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub source_service: String,
    pub correlation_id: String,
    pub user_id: UserId,
}

/// Complete task event structure for publishing to Kafka
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskEvent {
    pub event_type: TaskEventType,
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub old_data: Option<TaskEventData>,
    pub data: TaskEventData,
    pub metadata: EventMetadata,
}

impl TaskEvent {
    const CURRENT_VERSION: &str = "1.0";

    /// Create a new task created event
    pub fn new_created(data: TaskEventData, correlation_id: String) -> Self {
        let user_id = data.user_id;
        Self {
            event_type: TaskEventType::Created,
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: Self::CURRENT_VERSION.to_string(),
            old_data: None,
            data,
            metadata: EventMetadata {
                source_service: "rust-service-template".to_string(),
                correlation_id,
                user_id,
            },
        }
    }

    /// Create a new task updated event with old and new data
    pub fn new_updated(
        data: TaskEventData,
        old_data: TaskEventData,
        correlation_id: String,
    ) -> Self {
        let user_id = data.user_id;
        Self {
            event_type: TaskEventType::Updated,
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: Self::CURRENT_VERSION.to_string(),
            old_data: Some(old_data),
            data,
            metadata: EventMetadata {
                source_service: "rust-service-template".to_string(),
                correlation_id,
                user_id,
            },
        }
    }

    /// Create a new task deleted event
    pub fn new_deleted(data: TaskEventData, correlation_id: String) -> Self {
        let user_id = data.user_id;
        Self {
            event_type: TaskEventType::Deleted,
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: Self::CURRENT_VERSION.to_string(),
            old_data: None,
            data,
            metadata: EventMetadata {
                source_service: "rust-service-template".to_string(),
                correlation_id,
                user_id,
            },
        }
    }
}
