# Configuration Pattern

[← Dependencies](05-dependencies.md) | [Next: Error Handling →](07-error-handling.md)

---

## `config.rs`

```rust
use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use std::sync::Arc;

use crate::domain::interfaces::{
    event_producer::EventProducer,
    {feature}_repository::{Feature}Repository,
};
use crate::infrastructure::{
    {feature}::Postgres{Feature}Repository,
};

#[derive(Clone)]
pub struct AppState {
    pub {feature}_repo: Arc<Postgres{Feature}Repository>,
    pub event_service: Arc<dyn EventProducer>,
    pub env: AppConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub pool_config: DatabasePoolConfig,
    pub server_host: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub kafka_config: KafkaConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabasePoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

// skip if not needed
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub {feature}_topic: String,
    pub client_id: String,
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: 30,
            idle_timeout: 300,
            max_lifetime: 1800,
        }
    }
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            {feature}_topic: "{feature}-events".to_string(),
            client_id: "{service-name}".to_string(),
        }
    }
}

impl AppConfig {
    pub fn init() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(
                Environment::with_prefix("{SERVICE_NAME}")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}
```

---

## Environment Variable Naming

The configuration uses `{SERVICE_NAME}__` prefix with `__` as separator:

| Config Field | Environment Variable |
|--------------|---------------------|
| `database_url` | `{SERVICE_NAME}__DATABASE_URL` |
| `server_host` | `{SERVICE_NAME}__SERVER_HOST` |
| `pool_config.max_connections` | `{SERVICE_NAME}__POOL_CONFIG__MAX_CONNECTIONS` |
| `kafka_config.bootstrap_servers` | `{SERVICE_NAME}__KAFKA_CONFIG__BOOTSTRAP_SERVERS` |

---

[← Dependencies](05-dependencies.md) | [Next: Error Handling →](07-error-handling.md)
