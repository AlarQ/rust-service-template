# CORS Configuration

[← Health Checks](10-health-checks.md) | [Next: JWT Authentication →](12-jwt-authentication.md)

---

## Development (Permissive)

```rust
use tower_http::cors::CorsLayer;

// In build_app_router()
.layer(CorsLayer::permissive())
```

---

## Production (Restricted)

```rust
use tower_http::cors::{Any, CorsLayer};
use axum::http::{header, Method};

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin([
            "https://yourdomain.com".parse().unwrap(),
            "https://app.yourdomain.com".parse().unwrap(),
        ])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(3600))
}

// In build_app_router()
.layer(cors_layer())
```

---

## Configuration-Driven CORS

```rust
// In config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_secs: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allow_credentials: true,
            max_age_secs: 3600,
        }
    }
}

// In api/mod.rs
fn cors_layer_from_config(config: &CorsConfig) -> CorsLayer {
    let origins: Vec<_> = config
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(config.allow_credentials)
        .max_age(std::time::Duration::from_secs(config.max_age_secs))
}
```

---

[← Health Checks](10-health-checks.md) | [Next: JWT Authentication →](12-jwt-authentication.md)
