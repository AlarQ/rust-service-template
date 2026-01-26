# API Router and Server Startup

[← Main & Tracing](08-main-and-tracing.md) | [Next: Health Checks →](10-health-checks.md)

---

## `api/mod.rs`

```rust
use std::sync::Arc;

use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppState;

// Import route modules
use {feature}::handlers::{feature}_routes;

pub mod auth;
pub mod error;
pub mod models;
pub mod {feature};

/// OpenAPI documentation definition
#[derive(OpenApi)]
#[openapi(
    paths(
        // Health endpoints
        health_check,
        readiness_check,
        // Feature endpoints
        {feature}::handlers::create_{feature},
        {feature}::handlers::get_{feature},
        {feature}::handlers::get_{features},
        {feature}::handlers::update_{feature},
        {feature}::handlers::delete_{feature},
    ),
    components(schemas(
        // Request/Response DTOs
        models::{feature}::Create{Feature}Request,
        models::{feature}::Create{Feature}Response,
        models::{feature}::Get{Feature}Response,
        models::{feature}::List{Features}Response,
        models::{feature}::Update{Feature}Request,
        // Error types
        error::ApiErrorResponse,
        error::ErrorCode,
        // Auth
        auth::JwtClaims,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "{features}", description = "{Feature} management endpoints"),
    ),
    security(
        ("jwt" = [])
    )
)]
pub struct ApiDoc;

/// Build the complete application router with all routes and middleware
pub async fn build_app_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // Health endpoints (no auth required)
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .with_state(app_state.clone())
        // Feature routes
        .merge({feature}_routes(app_state.clone()))
        // OpenAPI documentation
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
        .route("/api-docs/openapi.json", get(openapi_json_handler))
        // Middleware layers (applied in reverse order)
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(trace_404_middleware))
        .layer(CorsLayer::permissive()) // See CORS section for production config
}

/// Server startup function
pub async fn server_start(
    app_state: Arc<AppState>,
    config: crate::config::AppConfig,
) -> anyhow::Result<()> {
    let app = build_app_router(app_state).await;

    let bind_address = format!("{}:{}", config.server_host, config.server_port);

    let listener = tokio::net::TcpListener::bind(&bind_address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {bind_address}: {e}"))?;

    tracing::info!("Listening on: {}", listener.local_addr()?);
    tracing::info!("Swagger UI: http://{}/swagger-ui", bind_address);

    Ok(axum::serve(listener, app).await?)
}

/// OpenAPI JSON endpoint with pretty-printed output
#[utoipa::path(
    get,
    path = "/api-docs/openapi.json",
    tag = "docs",
    responses(
        (status = 200, description = "OpenAPI specification")
    )
)]
async fn openapi_json_handler() -> impl IntoResponse {
    let openapi = ApiDoc::openapi();
    let pretty_json = serde_json::to_string_pretty(&openapi)
        .unwrap_or_else(|_| serde_json::to_string(&openapi).unwrap());

    (
        StatusCode::OK,
        [("Content-Type", "application/json")],
        pretty_json,
    )
}

/// Custom middleware to log 404 responses for debugging
async fn trace_404_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let uri = request.uri().clone();
    let method = request.method().clone();

    let response = next.run(request).await;

    if response.status() == StatusCode::NOT_FOUND {
        tracing::warn!("404 Not Found: {} {}", method, uri);
    }

    response
}
```

---

## Key Components

| Component | Purpose |
|-----------|---------|
| `ApiDoc` | OpenAPI schema definition |
| `build_app_router` | Assembles all routes and middleware |
| `server_start` | Binds and starts the HTTP server |
| `trace_404_middleware` | Logs 404 responses for debugging |

---

[← Main & Tracing](08-main-and-tracing.md) | [Next: Health Checks →](10-health-checks.md)
