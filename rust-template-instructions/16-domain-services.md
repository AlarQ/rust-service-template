# Domain Services Pattern

[← Repository Pattern](15-repository-pattern.md) | [Next: Value Objects →](17-value-objects.md)

---

## `domain/{feature}/operations.rs`

```rust
use std::sync::Arc;

use super::models::{Feature, FeatureError, FeatureEvent, FeatureId};
use crate::{
    common::UserId,
    domain::interfaces::{
        event_producer::EventProducer,
        {feature}_repository::{Feature}Repository,
    },
};

/// Macro to reduce repetitive error handling in domain functions
macro_rules! map_db_error {
    ($result:expr) => {
        $result.map_err(|e| FeatureError::external_error(e.to_string()))
    };
}

pub async fn create_{feature}(
    entity: Feature,
    repo: Arc<dyn {Feature}Repository>,
    events: Arc<dyn EventProducer>,
) -> Result<Feature, FeatureError> {
    let created = map_db_error!(repo.create(entity).await)?;

    // Publish event
    let event = FeatureEvent::new_created(&created, uuid::Uuid::new_v4());
    if let Err(e) = events.publish_{feature}_event(event).await {
        // Log error but don't fail the operation
        tracing::error!("Failed to publish {feature} created event: {}", e);
    }

    Ok(created)
}

pub async fn get_{feature}(
    id: FeatureId,
    repo: Arc<dyn {Feature}Repository>,
) -> Result<Feature, FeatureError> {
    match repo.get(id.clone()).await.map_err(|e| FeatureError::external_error(e.to_string())) {
        Ok(Some(entity)) => Ok(entity),
        Ok(None) => Err(FeatureError::not_found("Feature", id.to_string())),
        Err(e) => Err(e),
    }
}

pub async fn get_{features}_by_user(
    user_id: UserId,
    repo: Arc<dyn {Feature}Repository>,
) -> Result<Vec<Feature>, FeatureError> {
    repo.get_by_user(user_id)
        .await
        .map_err(|e| FeatureError::external_error(e.to_string()))
}

pub async fn delete_{feature}(
    id: FeatureId,
    repo: Arc<dyn {Feature}Repository>,
    events: Arc<dyn EventProducer>,
) -> Result<(), FeatureError> {
    // Get entity details before deletion for the event
    let entity = match repo.get(id.clone()).await {
        Ok(Some(e)) => e,
        Ok(None) => return Err(FeatureError::not_found("Feature", id.to_string())),
        Err(e) => return Err(FeatureError::external_error(e.to_string())),
    };

    repo.delete(id.clone()).await?;

    // Publish event
    let event = FeatureEvent::new_deleted(id, entity.user_id, uuid::Uuid::new_v4());
    if let Err(e) = events.publish_{feature}_event(event).await {
        tracing::error!("Failed to publish {feature} deleted event: {}", e);
    }

    Ok(())
}

pub async fn update_{feature}(
    id: FeatureId,
    entity: Feature,
    repo: Arc<dyn {Feature}Repository>,
    events: Arc<dyn EventProducer>,
) -> Result<Feature, FeatureError> {
    // Fetch existing entity
    let existing = match repo.get(id.clone()).await {
        Ok(Some(e)) => e,
        Ok(None) => return Err(FeatureError::not_found("Feature", id.to_string())),
        Err(e) => return Err(FeatureError::external_error(e.to_string())),
    };

    // Merge request data with existing entity (preserve immutable fields)
    let mut updated = entity;
    updated.id = existing.id.clone();
    updated.user_id = existing.user_id.clone();
    updated.created_at = existing.created_at;

    repo.update(&updated).await?;

    // Publish event
    let event = FeatureEvent::new_updated(&existing, &updated, uuid::Uuid::new_v4());
    if let Err(e) = events.publish_{feature}_event(event).await {
        tracing::error!("Failed to publish {feature} updated event: {}", e);
    }

    Ok(updated)
}
```

---

## Key Patterns

| Pattern | Purpose |
|---------|---------|
| Free functions | No service structs - pure functions with injected deps |
| `Arc<dyn Trait>` | Dependency injection via trait objects |
| Non-blocking events | Log event failures but don't fail operation |
| Fetch before delete | Capture data for event before deletion |

---

[← Repository Pattern](15-repository-pattern.md) | [Next: Value Objects →](17-value-objects.md)
