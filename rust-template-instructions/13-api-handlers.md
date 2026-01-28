# API Handler Pattern

[← JWT Authentication](12-jwt-authentication.md) | [Next: File Uploads →](14-file-uploads.md)

---

## Handler (`api/{feature}/handlers.rs`)

```rust
use std::sync::Arc;

use axum::{
    extract::{Json, Path, State},
    routing::{delete, get, post, put},
    Router,
};
use tracing::{field::Empty, instrument};
use uuid::Uuid;

use crate::{
    api::{auth::JwtExtractor, error::ApiErrorResponse, models::*},
    common::UserId,
    config::AppState,
    domain::{self, {feature}::models::{Feature}Id},
};

pub fn {feature}_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/users/{user_id}/{features}", post(create_{feature}))
        .route("/v1/users/{user_id}/{features}", get(get_{features}))
        .route("/v1/users/{user_id}/{features}/{id}", get(get_{feature}))
        .route("/v1/users/{user_id}/{features}/{id}", put(update_{feature}))
        .route("/v1/users/{user_id}/{features}/{id}", delete(delete_{feature}))
        .with_state(app_state)
}

#[utoipa::path(
    post,
    path = "/v1/users/{user_id}/{features}",
    request_body = Create{Feature}Request,
    security(("jwt" = [])),
    responses(
        (status = 200, body = Create{Feature}Response),
        (status = 400, body = ApiErrorResponse, description = "Bad request"),
        (status = 401, body = ApiErrorResponse, description = "Unauthorized"),
        (status = 500, body = ApiErrorResponse, description = "Internal server error"),
    )
)]
#[instrument(
    name = "[Create {Feature}]",
    skip(app_state, claims),
    fields(
        user_id = %user_id,
        description = Empty
    )
)]
async fn create_{feature}(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    JwtExtractor(claims): JwtExtractor,
    Json(request): Json<Create{Feature}Request>,
) -> Result<Create{Feature}Response, ApiErrorResponse> {
    // Validate that the user_id from the path matches the JWT claims
    claims.validate_user_id(user_id)?;

    // Set span fields with actual values
    tracing::Span::current().record("description", &request.description);

    let mut entity: domain::{feature}::models::{Feature} = request.try_into()?;
    entity.user_id = UserId::from(user_id);

    domain::{feature}::create_{feature}(
        entity,
        app_state.{feature}_repo.clone(),
        app_state.event_service.clone(),
    )
    .await
    .map_err(ApiErrorResponse::from)
    .map(Into::into)
}

#[utoipa::path(
    get,
    path = "/v1/users/{user_id}/{features}/{id}",
    security(("jwt" = [])),
    responses(
        (status = 200, body = Get{Feature}Response),
        (status = 401, body = ApiErrorResponse, description = "Unauthorized"),
        (status = 404, body = ApiErrorResponse, description = "Not found"),
    )
)]
#[instrument(
    name = "[Get {Feature}]",
    skip(app_state, claims),
    fields(user_id = %user_id, {feature}_id = %id)
)]
async fn get_{feature}(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, id)): Path<(Uuid, Uuid)>,
    JwtExtractor(claims): JwtExtractor,
) -> Result<Get{Feature}Response, ApiErrorResponse> {
    claims.validate_user_id(user_id)?;

    let entity = domain::{feature}::get_{feature}(
        {Feature}Id::from(id),
        app_state.{feature}_repo.clone(),
    )
    .await?;

    Ok(entity.into())
}

#[utoipa::path(
    get,
    path = "/v1/users/{user_id}/{features}",
    security(("jwt" = [])),
    responses(
        (status = 200, body = List{Features}Response),
        (status = 401, body = ApiErrorResponse, description = "Unauthorized"),
    )
)]
#[instrument(
    name = "[Get {Features}]",
    skip(app_state, claims),
    fields(user_id = %user_id)
)]
async fn get_{features}(
    State(app_state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
    JwtExtractor(claims): JwtExtractor,
) -> Result<List{Features}Response, ApiErrorResponse> {
    claims.validate_user_id(user_id)?;

    let entities = domain::{feature}::get_{features}_by_user(
        UserId::from(user_id),
        app_state.{feature}_repo.clone(),
    )
    .await?;

    Ok(List{Features}Response {
        {features}: entities.into_iter().map(Into::into).collect(),
    })
}

#[utoipa::path(
    delete,
    path = "/v1/users/{user_id}/{features}/{id}",
    security(("jwt" = [])),
    responses(
        (status = 200, description = "Deleted successfully"),
        (status = 401, body = ApiErrorResponse, description = "Unauthorized"),
        (status = 404, body = ApiErrorResponse, description = "Not found"),
    )
)]
#[instrument(
    name = "[Delete {Feature}]",
    skip(app_state, claims),
    fields(user_id = %user_id, {feature}_id = %id)
)]
async fn delete_{feature}(
    State(app_state): State<Arc<AppState>>,
    Path((user_id, id)): Path<(Uuid, Uuid)>,
    JwtExtractor(claims): JwtExtractor,
) -> Result<(), ApiErrorResponse> {
    claims.validate_user_id(user_id)?;

    domain::{feature}::delete_{feature}(
        {Feature}Id::from(id),
        app_state.{feature}_repo.clone(),
        app_state.event_service.clone(),
    )
    .await
    .map_err(ApiErrorResponse::from)
}
```

---

## Handler Pattern Summary

| Component | Purpose |
|-----------|---------|
| `#[utoipa::path]` | OpenAPI documentation |
| `#[instrument]` | Tracing spans with fields |
| `JwtExtractor` | Extract and validate JWT token |
| `claims.validate_user_id()` | Verify path user matches token |
| Domain function call | Delegate business logic |
| `Into::into` | Convert domain → response DTO |

---

[← JWT Authentication](12-jwt-authentication.md) | [Next: File Uploads →](14-file-uploads.md)
