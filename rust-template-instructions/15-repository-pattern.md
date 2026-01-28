# Repository Pattern

[← File Uploads](14-file-uploads.md) | [Next: Domain Services →](16-domain-services.md)

---

## Interface (`domain/interfaces/{feature}_repository.rs`)

```rust
use async_trait::async_trait;
use std::fmt::Debug;

use crate::{
    common::UserId,
    domain::{feature}::models::{Feature, FeatureError, FeatureId},
};

#[async_trait]
pub trait {Feature}Repository: Send + Sync + Debug {
    async fn create(&self, entity: Feature) -> Result<Feature, FeatureError>;
    async fn get(&self, id: FeatureId) -> Result<Option<Feature>, FeatureError>;
    async fn delete(&self, id: FeatureId) -> Result<(), FeatureError>;
    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Feature>, FeatureError>;
    async fn update(&self, entity: &Feature) -> Result<(), FeatureError>;
    async fn health_check(&self) -> Result<(), sqlx::Error>;
}
```

---

## Implementation (`infrastructure/{feature}.rs`)

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use std::fmt::Debug;

use crate::{
    common::UserId,
    domain::{
        interfaces::{feature}_repository::{Feature}Repository,
        {feature}::models::{Feature, FeatureError, FeatureId},
    },
};

#[derive(Clone)]
pub struct Postgres{Feature}Repository {
    pool: PgPool,
}

impl Debug for Postgres{Feature}Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Postgres{Feature}Repository")
            .field("pool", &"PgPool")
            .finish()
    }
}

impl Postgres{Feature}Repository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl {Feature}Repository for Postgres{Feature}Repository {
    async fn create(&self, entity: Feature) -> Result<Feature, FeatureError> {
        let row = sqlx::query_as!(
            FeatureRow,
            r#"
            INSERT INTO {features} (id, user_id, name, description, created_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, user_id, name, description, created_at
            "#,
            entity.id.0,
            entity.user_id.0,
            entity.name,
            entity.description,
            entity.created_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.into())
    }

    async fn get(&self, id: FeatureId) -> Result<Option<Feature>, FeatureError> {
        let row = sqlx::query_as!(
            FeatureRow,
            r#"
            SELECT id, user_id, name, description, created_at
            FROM {features}
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    async fn delete(&self, id: FeatureId) -> Result<(), FeatureError> {
        sqlx::query!("DELETE FROM {features} WHERE id = $1", id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_by_user(&self, user_id: UserId) -> Result<Vec<Feature>, FeatureError> {
        let rows = sqlx::query_as!(
            FeatureRow,
            r#"
            SELECT id, user_id, name, description, created_at
            FROM {features}
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id.0,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&self, entity: &Feature) -> Result<(), FeatureError> {
        sqlx::query!(
            r#"
            UPDATE {features}
            SET name = $2, description = $3
            WHERE id = $1
            "#,
            entity.id.0,
            entity.name,
            entity.description,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

// Row type for SQLx
struct FeatureRow {
    id: Uuid,
    user_id: Uuid,
    name: String,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl From<FeatureRow> for Feature {
    fn from(row: FeatureRow) -> Self {
        Self {
            id: FeatureId(row.id),
            user_id: UserId(row.user_id),
            name: row.name,
            description: row.description,
            created_at: row.created_at,
        }
    }
}
```

---

## Key Points

| Pattern | Purpose |
|---------|---------|
| Trait in `domain/interfaces/` | Abstraction for testing and flexibility |
| `Send + Sync + Debug` bounds | Required for async + Arc usage |
| Row type with `From` impl | Clean conversion from database rows |
| `query_as!` macro | Compile-time checked SQL |

---

[← File Uploads](14-file-uploads.md) | [Next: Domain Services →](16-domain-services.md)
