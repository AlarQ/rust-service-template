use config::{Config, ConfigError, Environment};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;

use crate::domain::interfaces::task_repository::TaskRepository;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub env: AppConfig,
    pub task_repository: Arc<dyn TaskRepository>,
}

/// Application configuration loaded from environment variables
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    #[serde(default)]
    pub pool_config: DatabasePoolConfig,
    #[serde(default = "default_server_host")]
    pub server_host: String,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    pub jwt_secret: String,
    #[serde(default)]
    pub kafka_config: KafkaConfig,
    #[serde(default)]
    pub cors_config: CorsConfig,
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    3000
}

/// Database connection pool configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabasePoolConfig {
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
    #[serde(default = "default_acquire_timeout")]
    pub acquire_timeout: u64,
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,
    #[serde(default = "default_max_lifetime")]
    pub max_lifetime: u64,
}

fn default_max_connections() -> u32 {
    10
}
fn default_min_connections() -> u32 {
    2
}
fn default_acquire_timeout() -> u64 {
    30
}
fn default_idle_timeout() -> u64 {
    300
}
fn default_max_lifetime() -> u64 {
    1800
}

impl Default for DatabasePoolConfig {
    fn default() -> Self {
        Self {
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            acquire_timeout: default_acquire_timeout(),
            idle_timeout: default_idle_timeout(),
            max_lifetime: default_max_lifetime(),
        }
    }
}

/// Kafka configuration for event streaming
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    #[serde(default = "default_bootstrap_servers")]
    pub bootstrap_servers: String,
    #[serde(default = "default_client_id")]
    pub client_id: String,
}

fn default_bootstrap_servers() -> String {
    "localhost:9092".to_string()
}

fn default_client_id() -> String {
    "rust-service-template".to_string()
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: default_bootstrap_servers(),
            client_id: default_client_id(),
        }
    }
}

/// CORS (Cross-Origin Resource Sharing) configuration
///
/// Controls which origins, methods, and headers are allowed for cross-origin requests.
/// This is essential for browser-based clients accessing the API from different domains.
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    /// List of allowed origins for CORS requests
    /// Use ["*"] to allow all origins (development only)
    /// Use specific origins like ["https://example.com"] for production
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,
    /// List of allowed HTTP methods
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,
    /// List of allowed HTTP headers
    /// Use ["*"] to allow all headers (development only)
    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,
    /// Whether to allow credentials (cookies, authorization headers) in CORS requests
    #[serde(default = "default_allow_credentials")]
    pub allow_credentials: bool,
    /// Maximum age (in seconds) for CORS preflight cache
    #[serde(default = "default_max_age")]
    pub max_age: u64,
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allowed_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_allowed_headers() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allow_credentials() -> bool {
    false
}

fn default_max_age() -> u64 {
    3600
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: default_allowed_origins(),
            allowed_methods: default_allowed_methods(),
            allowed_headers: default_allowed_headers(),
            allow_credentials: default_allow_credentials(),
            max_age: default_max_age(),
        }
    }
}

impl AppConfig {
    /// Initialize configuration from environment variables
    ///
    /// Environment variables use the prefix `RUST_SERVICE_TEMPLATE__` with `__` as separator.
    ///
    /// # Examples
    /// - `RUST_SERVICE_TEMPLATE__DATABASE_URL`
    /// - `RUST_SERVICE_TEMPLATE__SERVER_PORT`
    /// - `RUST_SERVICE_TEMPLATE__POOL_CONFIG__MAX_CONNECTIONS`
    /// - `RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_ORIGINS` (comma-separated)
    /// - `RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_METHODS` (comma-separated)
    /// - `RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_HEADERS` (comma-separated)
    /// - `RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOW_CREDENTIALS`
    /// - `RUST_SERVICE_TEMPLATE__CORS_CONFIG__MAX_AGE`
    pub fn init() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let config = Config::builder()
            .add_source(
                Environment::with_prefix("RUST_SERVICE_TEMPLATE")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        config.try_deserialize()
    }
}
