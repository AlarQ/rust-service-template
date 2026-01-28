# Main & Tracing Setup

[← Error Handling](07-error-handling.md) | [Next: API Router →](09-api-router.md)

---

## `main.rs`

```rust
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]

use anyhow::Result;
use std::{env, sync::Arc};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use {service_name}::{
    api::server_start,
    config::{AppConfig, AppState},
    infrastructure::{
        {feature}::Postgres{Feature}Repository,
        kafka_producer::KafkaEventService,
    },
};

pub mod api;
pub mod common;
pub mod config;
pub mod domain;
pub mod infrastructure;

#[tokio::main]
async fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "{service_name}=debug,tower_http=debug,axum::rejection=trace,sqlx=info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting {service-name}");

    let config = AppConfig::init()
        .map_err(|e| anyhow::anyhow!("Configuration error: {e}"))?;

    tracing::info!("Connecting to database {}", config.database_url);

    // Create database pool with configuration
    let pool_options = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.pool_config.max_connections)
        .min_connections(config.pool_config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.pool_config.acquire_timeout))
        .idle_timeout(std::time::Duration::from_secs(config.pool_config.idle_timeout))
        .max_lifetime(std::time::Duration::from_secs(config.pool_config.max_lifetime));

    let db_pool = pool_options
        .connect(&config.database_url)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {e}"))?;

    tracing::info!("Database connected with pool config: {:?}", config.pool_config);

    let {feature}_repo = Postgres{Feature}Repository::new(db_pool.clone());

    tracing::info!("Running migrations...");
    sqlx::migrate!().run(&db_pool).await?;
    tracing::info!("Migrations finished");

    // Initialize Kafka event service
    let event_service = Arc::new(
        KafkaEventService::new(&config.kafka_config)
            .map_err(|e| anyhow::anyhow!("Failed to initialize Kafka event service: {e}"))?,
    );

    let app_state = Arc::new(AppState {
        {feature}_repo: Arc::new({feature}_repo),
        event_service,
        env: config.clone(),
    });

    server_start(app_state, config).await
}
```

---

## `lib.rs`

```rust
pub mod api;
pub mod common;
pub mod config;
pub mod domain;
pub mod infrastructure;
```

---

## Tracing Configuration

The tracing filter defaults to:
- `{service_name}=debug` - Debug logs for your service
- `tower_http=debug` - HTTP request/response logging
- `axum::rejection=trace` - Detailed rejection information
- `sqlx=info` - Database query info

Override with `RUST_LOG` environment variable.

---

[← Error Handling](07-error-handling.md) | [Next: API Router →](09-api-router.md)
