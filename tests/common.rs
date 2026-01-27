use std::sync::Arc;

use axum::Router;
use rust_service_template::{
    api::build_app_router,
    config::{AppConfig, AppState},
    infrastructure::task::PostgresTaskRepository,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT: std::sync::Once = std::sync::Once::new();

/// Test app setup with database connection and migrations
///
/// This function:
/// - Initializes environment variables once (using Once)
/// - Sets up test configuration
/// - Creates database connection pool
/// - Runs migrations
/// - Initializes AppState with test repositories
/// - Returns the application router and database pool
///
/// # Returns
/// A tuple containing:
/// - `Router`: The axum application router
/// - `Arc<PgPool>`: The database connection pool for test assertions
///
/// # Example
/// ```no_run
/// let (app, pool) = app().await;
/// // Make requests to app, use pool for DB assertions
/// ```
pub async fn app() -> (Router, Arc<sqlx::PgPool>) {
    INIT.call_once(|| {
        // Set JWT secret for tests
        std::env::set_var(
            "RUST_SERVICE_TEMPLATE__JWT_SECRET",
            "this_is_a_very_long_secret_key_for_testing_purposes_only",
        );

        // Set server configuration for tests
        std::env::set_var("RUST_SERVICE_TEMPLATE__SERVER_HOST", "127.0.0.1");
        std::env::set_var("RUST_SERVICE_TEMPLATE__SERVER_PORT", "8080");
        std::env::set_var(
            "RUST_SERVICE_TEMPLATE__DATABASE_URL",
            "postgresql://postgres:postgres@localhost:5445/rust_service_template",
        );

        std::env::set_var(
            "RUST_LOG",
            "rust_service_template=debug,sqlx=debug,tower_http=debug,axum::rejection=trace",
        );

        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                    "rust_service_template=debug,tower_http=debug,axum::rejection=trace,sqlx=debug"
                        .into()
                }),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    });

    let config: AppConfig = AppConfig::init().expect("Failed to initialize config");

    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run migrations");

    let db_arc = Arc::new(db_pool.clone());
    let task_repo = Arc::new(PostgresTaskRepository::new(db_pool.clone()));

    let app_state = Arc::new(AppState {
        db_pool,
        env: config,
        task_repository: task_repo,
    });

    (build_app_router(app_state).await, db_arc)
}
