# Health Check Endpoints

[← API Router](09-api-router.md) | [Next: CORS →](11-cors.md)

---

Health checks are essential for Kubernetes deployments and load balancers.

## Liveness vs Readiness

| Endpoint | Purpose | Checks | When to fail |
|----------|---------|--------|--------------|
| `/health` | Liveness | Service is running | Service is deadlocked |
| `/ready` | Readiness | Dependencies available | DB/Kafka unavailable |

---

## Implementation

```rust
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use std::sync::Arc;

use crate::config::AppState;

/// Liveness probe - returns 200 if service is running
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is running")
    )
)]
pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Readiness probe - checks database connectivity
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service not ready")
    )
)]
pub async fn readiness_check(
    State(app_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Check database connectivity
    match app_state.{feature}_repo.health_check().await {
        Ok(()) => (StatusCode::OK, "Ready"),
        Err(e) => {
            tracing::error!("Database health check failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "Database unavailable")
        }
    }
}
```

---

## Repository Health Check Method

Add to your repository trait and implementation:

```rust
// In domain/interfaces/{feature}_repository.rs
#[async_trait]
pub trait {Feature}Repository: Send + Sync + Debug {
    // ... other methods
    async fn health_check(&self) -> Result<(), sqlx::Error>;
}

// In infrastructure/{feature}.rs
async fn health_check(&self) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(&self.pool).await?;
    Ok(())
}
```

---

[← API Router](09-api-router.md) | [Next: CORS →](11-cors.md)
