# Event Schema Pattern

[← Background Jobs](19-background-jobs.md) | [Next: Kafka Producer →](21-kafka-producer.md)

---

## Event Types Enum

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    {Feature}Created,
    {Feature}Updated,
    {Feature}Deleted,
}
```

---

## Full Event Structure

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {Feature}Event {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Type of event
    pub event_type: EventType,

    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,

    /// Schema version for evolution
    pub version: String,

    /// Current entity data
    pub data: {Feature}EventData,

    /// Previous entity data (for updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_data: Option<{Feature}EventData>,

    /// Event metadata
    pub metadata: EventMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {Feature}EventData {
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub description: String,
    pub category_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Service that produced the event
    pub source_service: String,

    /// Correlation ID for distributed tracing
    pub correlation_id: Uuid,

    /// User who triggered the action
    pub user_id: Uuid,
}
```

---

## Event Factory Methods

```rust
impl {Feature}Event {
    const VERSION: &'static str = "1.0";
    const SOURCE: &'static str = "{service-name}";

    pub fn new_created(
        entity: &{Feature},
        category_name: String,
        correlation_id: Uuid,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: EventType::{Feature}Created,
            timestamp: Utc::now(),
            version: Self::VERSION.to_string(),
            data: {Feature}EventData {
                id: entity.id.0,
                user_id: entity.user_id.0,
                amount: entity.amount.value(),
                description: entity.description.value().to_string(),
                category_name,
            },
            old_data: None,
            metadata: EventMetadata {
                source_service: Self::SOURCE.to_string(),
                correlation_id,
                user_id: entity.user_id.0,
            },
        }
    }

    pub fn new_updated(
        old_entity: &{Feature},
        new_entity: &{Feature},
        old_category: String,
        new_category: String,
        correlation_id: Uuid,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: EventType::{Feature}Updated,
            timestamp: Utc::now(),
            version: Self::VERSION.to_string(),
            data: {Feature}EventData {
                id: new_entity.id.0,
                user_id: new_entity.user_id.0,
                amount: new_entity.amount.value(),
                description: new_entity.description.value().to_string(),
                category_name: new_category,
            },
            old_data: Some({Feature}EventData {
                id: old_entity.id.0,
                user_id: old_entity.user_id.0,
                amount: old_entity.amount.value(),
                description: old_entity.description.value().to_string(),
                category_name: old_category,
            }),
            metadata: EventMetadata {
                source_service: Self::SOURCE.to_string(),
                correlation_id,
                user_id: new_entity.user_id.0,
            },
        }
    }

    pub fn new_deleted(
        id: {Feature}Id,
        user_id: UserId,
        correlation_id: Uuid,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: EventType::{Feature}Deleted,
            timestamp: Utc::now(),
            version: Self::VERSION.to_string(),
            data: {Feature}EventData {
                id: id.0,
                user_id: user_id.0,
                amount: rust_decimal::Decimal::ZERO,
                description: String::new(),
                category_name: String::new(),
            },
            old_data: None,
            metadata: EventMetadata {
                source_service: Self::SOURCE.to_string(),
                correlation_id,
                user_id: user_id.0,
            },
        }
    }
}
```

---

## Event Schema Benefits

| Field | Purpose |
|-------|---------|
| `event_id` | Deduplication, idempotency |
| `version` | Schema evolution support |
| `old_data` | Enables change detection for updates |
| `correlation_id` | Distributed tracing across services |

---

[← Background Jobs](19-background-jobs.md) | [Next: Kafka Producer →](21-kafka-producer.md)
