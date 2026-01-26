# Quick Start Checklist

[← CLAUDE.md](25-claude-md.md) | [Back to Overview →](00-overview.md)

---

## Prerequisites

- [ ] Install Rust nightly toolchain: `rustup toolchain install nightly`
- [ ] Install cargo-audit: `cargo install cargo-audit`
- [ ] Install cargo-outdated: `cargo install cargo-outdated`

---

## Core Setup (Files 01-05)

- [ ] Copy directory structure template ([03-project-structure.md](03-project-structure.md))
- [ ] Update `Cargo.toml` with service name and dependencies ([05-dependencies.md](05-dependencies.md))
- [ ] Create `rustfmt.toml` with formatting rules ([02-code-quality-config.md](02-code-quality-config.md))
- [ ] Add clippy lints to `main.rs` ([02-code-quality-config.md](02-code-quality-config.md))
- [ ] Set up `config.rs` with environment variables ([06-configuration.md](06-configuration.md))
- [ ] Set up tracing in `main.rs` ([08-main-and-tracing.md](08-main-and-tracing.md))

---

## API Layer (Files 09-14)

- [ ] Create `api/mod.rs` with router and OpenAPI setup ([09-api-router.md](09-api-router.md))
- [ ] Add health check endpoints `/health` and `/ready` ([10-health-checks.md](10-health-checks.md))
- [ ] Configure CORS layer ([11-cors.md](11-cors.md))
- [ ] Build API handlers with JWT authentication ([13-api-handlers.md](13-api-handlers.md))
- [ ] Create error types with proper conversions ([07-error-handling.md](07-error-handling.md))
- [ ] Add multipart handlers if needed ([14-file-uploads.md](14-file-uploads.md))

---

## Domain Layer (Files 15-19)

- [ ] Define domain interfaces in `domain/interfaces/` ([15-repository-pattern.md](15-repository-pattern.md))
- [ ] Implement domain services as free functions ([16-domain-services.md](16-domain-services.md))
- [ ] Create value objects with validation ([17-value-objects.md](17-value-objects.md))
- [ ] Implement DTO conversions (TryFrom, From) ([18-dto-conversion.md](18-dto-conversion.md))
- [ ] Add background job support if needed ([19-background-jobs.md](19-background-jobs.md))

---

## Infrastructure (Files 20-21)

- [ ] Create repository implementations in `infrastructure/`
- [ ] Define event schemas ([20-event-schema.md](20-event-schema.md))
- [ ] Configure Kafka producer if needed ([21-kafka-producer.md](21-kafka-producer.md))

---

## Testing & DevOps (Files 22-25)

- [ ] Write test utilities in `tests/common.rs` ([22-testing.md](22-testing.md))
- [ ] Set up `docker-compose.yaml` ([23-docker-compose.md](23-docker-compose.md))
- [ ] Create `run.sh` with environment variables ([24-run-script.md](24-run-script.md))
- [ ] Run `./scripts/install-hooks.sh` to install git hooks ([01-prerequisites.md](01-prerequisites.md))
- [ ] Create `CLAUDE.md` for AI assistance ([25-claude-md.md](25-claude-md.md))

---

## Optional Features

- [ ] File uploads ([14-file-uploads.md](14-file-uploads.md))
- [ ] Background jobs ([19-background-jobs.md](19-background-jobs.md))
- [ ] Kafka events ([20-event-schema.md](20-event-schema.md), [21-kafka-producer.md](21-kafka-producer.md))
- [ ] External HTTP API integrations (reqwest)

---

## Placeholders Reference

Replace these placeholders when using the template:

| Placeholder | Example | Description |
|-------------|---------|-------------|
| `{service-name}` | `user-service` | Kebab-case service name |
| `{service_name}` | `user_service` | Snake_case service name |
| `{SERVICE_NAME}` | `USER_SERVICE` | SCREAMING_SNAKE_CASE for env vars |
| `{feature}` | `user` | Snake_case feature/entity name |
| `{Feature}` | `User` | PascalCase feature/entity name |
| `{FEATURE}` | `USER` | SCREAMING_SNAKE_CASE for constants |
| `{features}` | `users` | Plural snake_case |
| `{Features}` | `Users` | Plural PascalCase |
| `{port}` | `8080` | Service port |
| `{db_port}` | `5432` | Database port |

---

## Placeholder Usage Examples

```rust
// {service-name} → user-service (Cargo.toml name, URLs)
// {service_name} → user_service (Rust module names, crate imports)
// {SERVICE_NAME} → USER_SERVICE (environment variable prefix)

// {feature} → transaction (function names, file names)
// {Feature} → Transaction (struct names, type names)
// {FEATURE} → TRANSACTION (constants)

// {features} → transactions (table names, route paths)
// {Features} → Transactions (plural type names)
```

---

[← CLAUDE.md](25-claude-md.md) | [Back to Overview →](00-overview.md)
