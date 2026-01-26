# DTO Conversion Pattern

[← Value Objects](17-value-objects.md) | [Next: Background Jobs →](19-background-jobs.md)

---

## Request DTOs with TryFrom

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Request DTO - uses camelCase for API consistency
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Create{Feature}Request {
    pub amount: rust_decimal::Decimal,
    pub description: String,
    pub category_id: uuid::Uuid,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Convert request DTO to domain entity with validation
impl TryFrom<Create{Feature}Request> for crate::domain::{feature}::models::{Feature} {
    type Error = crate::domain::{feature}::models::ValidationError;

    fn try_from(req: Create{Feature}Request) -> Result<Self, Self::Error> {
        use crate::domain::{feature}::models::*;

        Ok({Feature} {
            id: {Feature}Id::new(),
            user_id: UserId::default(), // Set by handler from JWT
            amount: Amount::new(req.amount)?,
            description: Description::new(req.description)?,
            category_id: CategoryId::from(req.category_id),
            created_at: chrono::Utc::now(),
        })
    }
}
```

---

## Response DTOs with From

```rust
use axum::response::IntoResponse;

/// Response DTO
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct {Feature}Response {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub amount: rust_decimal::Decimal,
    pub description: String,
    pub category_id: uuid::Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Convert domain entity to response DTO
impl From<crate::domain::{feature}::models::{Feature}> for {Feature}Response {
    fn from(entity: crate::domain::{feature}::models::{Feature}) -> Self {
        Self {
            id: entity.id.0,
            user_id: entity.user_id.0,
            amount: entity.amount.value(),
            description: entity.description.value().to_string(),
            category_id: entity.category_id.0,
            created_at: entity.created_at,
        }
    }
}

/// Wrapper for create response
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Create{Feature}Response {
    pub {feature}_id: uuid::Uuid,
}

impl From<crate::domain::{feature}::models::{Feature}> for Create{Feature}Response {
    fn from(entity: crate::domain::{feature}::models::{Feature}) -> Self {
        Self {
            {feature}_id: entity.id.0,
        }
    }
}

/// Implement IntoResponse for direct return from handlers
impl IntoResponse for Create{Feature}Response {
    fn into_response(self) -> axum::response::Response {
        (axum::http::StatusCode::CREATED, axum::Json(self)).into_response()
    }
}

/// List response wrapper
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct List{Features}Response {
    pub {features}: Vec<{Feature}Response>,
    pub total_count: usize,
}
```

---

## Update Request with Partial Fields

```rust
/// Update request - all fields optional for partial updates
#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Update{Feature}Request {
    pub amount: Option<rust_decimal::Decimal>,
    pub description: Option<String>,
    pub category_id: Option<uuid::Uuid>,
    pub notes: Option<String>,
}
```

---

## Conversion Flow

```
Request DTO  →  TryFrom  →  Domain Entity  →  From  →  Response DTO
     ↓              ↓              ↓             ↓           ↓
  camelCase    Validation    Business      Extract      camelCase
   JSON          Logic        Logic        Values        JSON
```

---

[← Value Objects](17-value-objects.md) | [Next: Background Jobs →](19-background-jobs.md)
