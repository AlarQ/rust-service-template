# Cargo.toml Dependencies Template

[← Architecture Rules](04-architecture-rules.md) | [Next: Configuration →](06-configuration.md)

---

> **Note**: Search web for newest versions before using.

```toml
[package]
name = "{service-name}"
version = "0.1.0"
edition = "2021"

[[test]]
name = "integration_tests"
path = "tests/integration_tests.rs"
harness = true

[dependencies]
# Web Framework
axum = { version = "0.8", features = ["macros", "multipart"] }
axum-extra = { version = "0.12", features = ["typed-header"] }
tower-http = { version = "0.6", features = ["trace", "cors"] }

# Async Runtime
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio",
    "postgres",
    "macros",
    "chrono",
    "uuid",
    "rust_decimal"
] }
rust_decimal = { version = "1" }

# Types
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Error Handling
thiserror = "2"
anyhow = "1"

# API Documentation
utoipa = { version = "5", features = ["uuid", "decimal", "chrono"] }
utoipa-swagger-ui = { version = "9", features = ["axum"] }

# Authentication
jsonwebtoken = "9"

# Configuration
config = "0.15"
dotenvy = "0.15"

# Event Streaming (optional)
rdkafka = { version = "0.36", features = ["ssl-vendored"] }

# HTTP Client (for external API calls, optional)
reqwest = { version = "0.12", features = ["json"] }

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

[dev-dependencies]
http-body-util = "0.1"
tower = "0.5"
```

---

## Optional Dependencies

| Dependency | Use Case |
|------------|----------|
| `rdkafka` | Event streaming with Kafka |
| `reqwest` | External HTTP API calls |

---

[← Architecture Rules](04-architecture-rules.md) | [Next: Configuration →](06-configuration.md)
