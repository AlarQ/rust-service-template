# Testing Pattern

[← Kafka Producer](21-kafka-producer.md) | [Next: Docker Compose →](23-docker-compose.md)

---

## `tests/common.rs`

```rust
use anyhow::Result;
use async_trait::async_trait;
use axum::Router;
use config::{Config, ConfigError};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use uuid::Uuid;

use {service_name}::{
    api::build_app_router,
    config::{AppConfig, AppState},
    domain::interfaces::event_producer::EventProducer,
    infrastructure::{feature}::Postgres{Feature}Repository,
};

// Mock event service for tests
#[derive(Debug)]
pub struct MockEventService;

#[async_trait]
impl EventProducer for MockEventService {
    async fn publish_{feature}_event(&self, _event: FeatureEvent) -> Result<(), FeatureError> {
        tracing::debug!("Mock event service: would publish event");
        Ok(())
    }
}

use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT: std::sync::Once = std::sync::Once::new();

pub fn test_config(database_url: String) -> Result<AppConfig, ConfigError> {
    let config = Config::builder()
        .set_override("database_url", database_url)?
        .set_override("pool_config.max_connections", 5)?
        .set_override("pool_config.min_connections", 1)?
        .set_override("pool_config.acquire_timeout", 30)?
        .set_override("pool_config.idle_timeout", 300)?
        .set_override("pool_config.max_lifetime", 600)?
        .set_override("server_host", "0.0.0.0")?
        .set_override("server_port", 8080)?
        .set_override("jwt_secret", "this_is_a_very_long_secret_key_for_testing_purposes_only")?
        .set_override("kafka_config.bootstrap_servers", "localhost:9092")?
        .set_override("kafka_config.{feature}_topic", "test-events")?
        .set_override("kafka_config.client_id", "test-{service-name}")?
        .build()?;

    config.try_deserialize()
}

#[derive(serde::Serialize)]
struct TestJwtClaims {
    sub: String,
    exp: usize,
    iat: usize,
    jti: String,
    aud: String,
    iss: String,
    client_id: String,
    scope: String,
    session_id: String,
}

pub async fn app() -> (Router, Arc<PgPool>) {
    INIT.call_once(|| {
        std::env::set_var("{SERVICE_NAME}__DATABASE_URL", "postgresql://postgres:postgres@localhost:5433/postgres");
        std::env::set_var("{SERVICE_NAME}__JWT_SECRET", "this_is_a_very_long_secret_key_for_testing_purposes_only");
        std::env::set_var("{SERVICE_NAME}__SERVER_HOST", "127.0.0.1");
        std::env::set_var("{SERVICE_NAME}__SERVER_PORT", "8080");
        std::env::set_var("RUST_LOG", "{service_name}=debug,tower_http=info,sqlx=info");

        tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "{service-name}=debug,tower_http=debug".into()))
            .with(tracing_subscriber::fmt::layer())
            .init();
    });

    let config = test_config("postgresql://postgres:postgres@localhost:5433/postgres".to_string())
        .expect("Failed to initialize config");

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&config.database_url)
        .await
        .unwrap();

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    let db_arc = Arc::new(db);
    let {feature}_repo = Arc::new(Postgres{Feature}Repository::new(db_arc.as_ref().clone()));

    let app_state = Arc::new(AppState {
        {feature}_repo,
        event_service: Arc::new(MockEventService),
        env: config,
    });

    (build_app_router(app_state).await, db_arc)
}

pub fn generate_test_token(user_id: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = TestJwtClaims {
        sub: user_id.to_string(),
        exp: now + 3600,
        iat: now,
        jti: Uuid::new_v4().to_string(),
        aud: "{service-name}".to_string(),
        iss: "user-service".to_string(),
        client_id: "user-client".to_string(),
        scope: "read:{features} write:{features}".to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("this_is_a_very_long_secret_key_for_testing_purposes_only".as_bytes()),
    )
    .unwrap()
}

pub fn generate_expired_token(user_id: &str) -> String {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = TestJwtClaims {
        sub: user_id.to_string(),
        exp: now - 3600, // Expired
        iat: now,
        jti: Uuid::new_v4().to_string(),
        aud: "{service-name}".to_string(),
        iss: "user-service".to_string(),
        client_id: "user-client".to_string(),
        scope: "read:{features} write:{features}".to_string(),
        session_id: Uuid::new_v4().to_string(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("this_is_a_very_long_secret_key_for_testing_purposes_only".as_bytes()),
    )
    .unwrap()
}

pub async fn make_authenticated_request(
    app: &Router,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<axum::body::Body>,
) -> (u16, Vec<u8>) {
    use axum::http::Request;

    let mut request_builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token));

    if body.is_some() {
        request_builder = request_builder.header("Content-Type", "application/json");
    }

    let request = if let Some(body) = body {
        request_builder.body(body).unwrap()
    } else {
        request_builder.body(axum::body::Body::empty()).unwrap()
    };

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, body_bytes.to_vec())
}

pub fn parse_json_response(body_bytes: &[u8]) -> Value {
    serde_json::from_slice(body_bytes).unwrap()
}
```

---

## Fluent Test Builder

```rust
pub struct TestBuilder {
    app: Router,
    pool: Arc<PgPool>,
    user_id: String,
    token: String,
}

impl TestBuilder {
    pub async fn new() -> Self {
        let (app, pool) = app().await;
        let user_id = Uuid::new_v4().to_string();
        let token = generate_test_token(&user_id);

        Self { app, pool, user_id, token }
    }

    pub async fn get(&self, path: &str) -> (u16, Vec<u8>) {
        make_authenticated_request(&self.app, "GET", path, &self.token, None).await
    }

    pub async fn post(&self, path: &str, body: Option<Value>) -> (u16, Vec<u8>) {
        let body = body.map(|b| axum::body::Body::from(b.to_string()));
        make_authenticated_request(&self.app, "POST", path, &self.token, body).await
    }

    pub async fn put(&self, path: &str, body: Option<Value>) -> (u16, Vec<u8>) {
        let body = body.map(|b| axum::body::Body::from(b.to_string()));
        make_authenticated_request(&self.app, "PUT", path, &self.token, body).await
    }

    pub async fn delete(&self, path: &str) -> (u16, Vec<u8>) {
        make_authenticated_request(&self.app, "DELETE", path, &self.token, None).await
    }

    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    pub fn app(&self) -> &Router {
        &self.app
    }

    pub fn db(&self) -> Arc<PgPool> {
        self.pool.clone()
    }

    pub fn parse_json(&self, bytes: &[u8]) -> Value {
        parse_json_response(bytes)
    }
}
```

---

## Example Test (`tests/integration/{feature}/mod.rs`)

```rust
use crate::common::{TestBuilder, parse_json_response};

#[tokio::test]
async fn test_create_{feature}() {
    let test = TestBuilder::new().await;

    let body = serde_json::json!({
        "name": "Test {Feature}",
        "description": "Test description"
    });

    let (status, response_bytes) = test
        .post(&format!("/v1/users/{}/{features}", test.user_id()), Some(body))
        .await;

    assert_eq!(status, 200);
    let response = parse_json_response(&response_bytes);
    assert!(response.get("{feature}Id").is_some());
}

#[tokio::test]
async fn test_get_{feature}_not_found() {
    let test = TestBuilder::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let (status, _) = test
        .get(&format!("/v1/users/{}/{features}/{}", test.user_id(), fake_id))
        .await;

    assert_eq!(status, 404);
}

#[tokio::test]
async fn test_unauthorized_without_token() {
    let test = TestBuilder::new().await;

    use axum::http::Request;
    use tower::ServiceExt;

    let request = Request::builder()
        .method("GET")
        .uri(&format!("/v1/users/{}/{features}", test.user_id()))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = test.app().clone().oneshot(request).await.unwrap();
    assert_eq!(response.status().as_u16(), 401);
}
```

---

## Database Verification Helpers

```rust
/// Verify entity exists in database
pub async fn {feature}_exists_in_db(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM {features} WHERE id = $1) as "exists!""#,
        id
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

/// Verify entity belongs to user
pub async fn {feature}_belongs_to_user(pool: &PgPool, id: Uuid, user_id: Uuid) -> bool {
    sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM {features} WHERE id = $1 AND user_id = $2) as "exists!""#,
        id,
        user_id
    )
    .fetch_one(pool)
    .await
    .unwrap_or(false)
}

/// Clean up test data for a user
pub async fn cleanup_user_data(pool: &PgPool, user_id: Uuid) {
    sqlx::query!("DELETE FROM {features} WHERE user_id = $1", user_id)
        .execute(pool)
        .await
        .ok();
}
```

---

## Multipart Request Helpers

```rust
/// Make authenticated multipart request for file uploads
pub async fn make_authenticated_multipart_request(
    app: &Router,
    uri: &str,
    token: &str,
    boundary: &str,
    body: Vec<u8>,
) -> (u16, Vec<u8>) {
    use axum::http::Request;

    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Authorization", format!("Bearer {}", token))
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(axum::body::Body::from(body))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, body_bytes.to_vec())
}

/// Build multipart body for file upload
pub fn build_multipart_file_body(
    file_content: &[u8],
    file_name: &str,
    content_type: &str,
) -> (String, Vec<u8>) {
    let boundary = format!("----boundary{}", Uuid::new_v4().to_string().replace("-", ""));

    let mut body = Vec::new();
    body.extend_from_slice(format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"{file_name}\"\r\n\
         Content-Type: {content_type}\r\n\r\n",
        boundary = boundary,
        file_name = file_name,
        content_type = content_type
    ).as_bytes());
    body.extend_from_slice(file_content);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n", boundary = boundary).as_bytes());

    (boundary, body)
}
```

---

[← Kafka Producer](21-kafka-producer.md) | [Next: Docker Compose →](23-docker-compose.md)
