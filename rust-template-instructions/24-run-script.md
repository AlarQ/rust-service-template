# Run Script Template

[← Docker Compose](23-docker-compose.md) | [Next: CLAUDE.md →](25-claude-md.md)

---

## `run.sh`

```bash
#!/bin/bash

# Set environment variables for local development
export RUST_BACKTRACE="full"
export RUST_LOG="{service_name}=debug,sqlx=info"

# Database configuration
export {SERVICE_NAME}__DATABASE_URL="postgresql://postgres:postgres@localhost:{db_port}/postgres?schema={service_name}"

# Server configuration
export {SERVICE_NAME}__SERVER_HOST="0.0.0.0"
export {SERVICE_NAME}__SERVER_PORT="{port}"

# JWT secret for authentication (shared across all services)
export {SERVICE_NAME}__JWT_SECRET="this_is_a_very_long_secret_key_for_testing_purposes_only"

# Database pool configuration
export {SERVICE_NAME}__POOL_CONFIG__MAX_CONNECTIONS="20"
export {SERVICE_NAME}__POOL_CONFIG__MIN_CONNECTIONS="5"
export {SERVICE_NAME}__POOL_CONFIG__ACQUIRE_TIMEOUT="30"
export {SERVICE_NAME}__POOL_CONFIG__IDLE_TIMEOUT="300"
export {SERVICE_NAME}__POOL_CONFIG__MAX_LIFETIME="1800"

# Kafka configuration
export {SERVICE_NAME}__KAFKA_CONFIG__BOOTSTRAP_SERVERS="localhost:9092"
export {SERVICE_NAME}__KAFKA_CONFIG__{FEATURE}_TOPIC="int.{feature}.event"
export {SERVICE_NAME}__KAFKA_CONFIG__CLIENT_ID="{service-name}"

# Run the service
cargo run
```

---

## Make Executable

```bash
chmod +x run.sh
```

---

## Usage

```bash
# Start dependencies first
docker-compose up -d

# Run the service
./run.sh
```

---

## Environment Variable Naming Convention

| Pattern | Example |
|---------|---------|
| `{SERVICE_NAME}__FIELD` | `USER_SERVICE__DATABASE_URL` |
| `{SERVICE_NAME}__NESTED__FIELD` | `USER_SERVICE__POOL_CONFIG__MAX_CONNECTIONS` |

The `__` separator maps to nested config structures.

---

[← Docker Compose](23-docker-compose.md) | [Next: CLAUDE.md →](25-claude-md.md)
