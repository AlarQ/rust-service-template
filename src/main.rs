#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use std::{env, sync::Arc};

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rust_service_template::{
    api::server_start,
    config::{AppConfig, AppState},
};

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "rust_service_template=debug,tower_http=debug,axum::rejection=trace,sqlx=info"
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting rust-service-template");

    let config = AppConfig::init()
        .map_err(|e| anyhow::anyhow!("Configuration error: {e}"))?;

    tracing::info!("Connecting to database...");

    // Create database pool with configuration
    let pool_options = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.pool_config.max_connections)
        .min_connections(config.pool_config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.pool_config.acquire_timeout,
        ))
        .idle_timeout(std::time::Duration::from_secs(
            config.pool_config.idle_timeout,
        ))
        .max_lifetime(std::time::Duration::from_secs(
            config.pool_config.max_lifetime,
        ));

    let db_pool = pool_options
        .connect(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {e}"))?;

    tracing::info!(
        "Database connected with pool config: {:?}",
        config.pool_config
    );

    tracing::info!("Running migrations...");
    sqlx::migrate!().run(&db_pool).await?;
    tracing::info!("Migrations finished");

    let app_state = Arc::new(AppState {
        db_pool,
        env: config.clone(),
    });

    server_start(app_state, config).await
}
