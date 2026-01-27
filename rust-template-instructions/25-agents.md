# AGENTS.md Template

[← Run Script](24-run-script.md) | [Next: Quick Start Checklist →](26-quick-start-checklist.md)

---

## Template

````markdown
# AGENTS.md

## Service Overview

{Brief description of the service's purpose and responsibilities}

## Essential Commands

```bash
# Start dependencies
docker-compose up -d

# Run service locally
./run.sh

# Run all tests
cargo test

# Run integration tests only
cargo test --test integration_tests

# Code quality
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

## Architecture

- **Port**: {port}
- **Database**: PostgreSQL on port {db_port}
- **Events**: Kafka topic `int.{feature}.event`

## Key API Endpoints

- `GET /v1/users/{user_id}/{features}` - List all {features}
- `POST /v1/users/{user_id}/{features}` - Create {feature}
- `GET /v1/users/{user_id}/{features}/{id}` - Get {feature} by ID
- `PUT /v1/users/{user_id}/{features}/{id}` - Update {feature}
- `DELETE /v1/users/{user_id}/{features}/{id}` - Delete {feature}
- `GET /health` - Health check
- `GET /ready` - Readiness check (includes database)
- `GET /swagger-ui` - API documentation

## Domain Concepts

- **{Feature}**: {Description}
- **{Entity}**: {Description}

## Critical Architecture Rules

1. **Handler -> Domain -> Repository pattern**: Never call repository from handler
2. **Free functions in domain**: No service structs in domain layer
3. **All traits in domain/interfaces**: Repository traits, service interfaces, etc.

## Testing

```bash
cargo test --test integration_tests
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| DB connection fails | `docker-compose up -d` |
| Port in use | Stop other services or change port |
| Kafka connection fails | Ensure Kafka is running |
| Tests fail with DB errors | Ensure test database is clean |

## Git Commit Guidelines

- **Format**: `<type>(<scope>): <description>`
- **Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`
- **Examples**:
  - `feat(api): add {feature} endpoints`
  - `fix(domain): handle validation edge cases`
  - `test({feature}): add integration tests`
````

---

## Purpose

The AGENTS.md file provides AI assistants with:
- Quick reference for common commands
- Architecture overview
- Key endpoints and domain concepts
- Troubleshooting guide
- Commit message conventions

---

[← Run Script](24-run-script.md) | [Next: Quick Start Checklist →](26-quick-start-checklist.md)
