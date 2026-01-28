# Critical Architecture Rules

[← Project Structure](03-project-structure.md) | [Next: Dependencies →](05-dependencies.md)

---

## Rule 1: Handler -> Domain -> Repository Pattern

**NEVER call repository directly from handler/route**

```rust
// WRONG: Handler calls repository directly
async fn handler(State(app): State<Arc<AppState>>) {
    app.repo.create(entity).await  // Bypasses business logic!
}

// CORRECT: Handler -> Domain function -> Repository
async fn handler(State(app): State<Arc<AppState>>) {
    domain::feature::create_entity(
        entity,
        app.repo.clone(),
        app.event_service.clone(),
    ).await
}
```

---

## Rule 2: Free Functions in Domain (No Service Structs)

```rust
// WRONG: Service struct in domain
pub struct {Feature}Service {
    repo: Arc<dyn {Feature}Repository>,
}

impl {Feature}Service {
    pub async fn create(&self, entity: {Feature}) -> Result<{Feature}, Error> {
        // ...
    }
}

// CORRECT: Free function with injected dependencies
pub async fn create_{feature}(
    entity: {Feature},
    repo: Arc<dyn {Feature}Repository>,
    events: Arc<dyn EventProducer>,
) -> Result<{Feature}, {Feature}Error> {
    let created = repo.create(entity).await?;

    // Publish event
    let event = {Feature}Event::new_created(&created, Uuid::new_v4());
    if let Err(e) = events.publish_{feature}_event(event).await {
        tracing::error!("Failed to publish event: {}", e);
    }

    Ok(created)
}
```

---

## Rule 3: All Traits in `domain/interfaces`

```rust
// src/domain/interfaces/{feature}_repository.rs
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait {Feature}Repository: Send + Sync + Debug {
    async fn create(&self, entity: {Feature}) -> Result<{Feature}, Error>;
    async fn get(&self, id: {Feature}Id) -> Result<Option<{Feature}>, Error>;
    async fn delete(&self, id: {Feature}Id) -> Result<(), Error>;
    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<{Feature}>, Error>;
    async fn update(&self, entity: &{Feature}) -> Result<(), Error>;
    async fn health_check(&self) -> Result<(), sqlx::Error>;
}
```

---

## Summary

| Rule | Description |
|------|-------------|
| Handler -> Domain -> Repository | All business logic in domain layer |
| Free functions | No service structs in domain |
| Traits in interfaces | All repository/service traits in `domain/interfaces/` |

---

[← Project Structure](03-project-structure.md) | [Next: Dependencies →](05-dependencies.md)
