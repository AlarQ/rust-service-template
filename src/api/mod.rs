pub mod error;
pub mod models;
pub mod tasks;

use std::sync::Arc;

use axum::{extract::State, http::StatusCode, middleware, response::IntoResponse, routing::get, Router};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    api::{
        error::{ApiErrorResponse, ErrorCode},
        tasks::handlers::{
            __path_create_task_handler, __path_get_task_handler, __path_list_tasks_handler,
            create_task_handler, get_task_handler, list_tasks_handler,
        },
    },
    config::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        readiness_check,
        get_task_handler,
        list_tasks_handler,
        create_task_handler,
    ),
    components(schemas(
        ApiErrorResponse,
        ErrorCode,
        crate::api::models::tasks::TaskResponse,
        crate::api::models::tasks::CreateTaskRequest,
        crate::api::models::tasks::TaskStatusSchema,
        crate::api::models::tasks::TaskPrioritySchema,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "tasks", description = "Task management endpoints"),
    )
)]
pub struct ApiDoc;

/// Build the complete application router with all routes and middleware
pub async fn build_app_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/tasks", get(list_tasks_handler).post(create_task_handler))
        .route("/tasks/{id}", get(get_task_handler))
        .with_state(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        .route("/api-docs/openapi.json", get(openapi_json_handler))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(trace_404_middleware))
}

/// Health check endpoint
#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy")
    )
)]
async fn health_check() -> &'static str {
    "OK"
}

/// Readiness check endpoint - verifies database connectivity
#[utoipa::path(
    get,
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Service is ready"),
        (status = 503, description = "Service not ready")
    )
)]
pub async fn readiness_check(State(app_state): State<Arc<AppState>>) -> impl IntoResponse {
    match crate::domain::task::check_readiness(&app_state.task_repository).await {
        Ok(()) => (StatusCode::OK, "Ready"),
        Err(e) => {
            tracing::error!("Readiness check failed: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "Database unavailable")
        }
    }
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

/// Start the HTTP server
pub async fn server_start(
    state: Arc<AppState>,
    config: crate::config::AppConfig,
) -> anyhow::Result<()> {
    let app = build_app_router(state).await;

    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Starting server on {}", addr);
    tracing::info!("Swagger UI: http://{}/swagger-ui", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
