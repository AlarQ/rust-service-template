#!/bin/bash

# Set environment variables for local development
export RUST_BACKTRACE="full"
export RUST_LOG="rust_service_template=debug,sqlx=info"

# Database configuration
export RUST_SERVICE_TEMPLATE__DATABASE_URL="postgres://postgres:postgres@localhost:5445/rust_service_template"

# Server configuration
export RUST_SERVICE_TEMPLATE__SERVER_HOST="0.0.0.0"
export RUST_SERVICE_TEMPLATE__SERVER_PORT="8080"

# JWT secret for authentication
export RUST_SERVICE_TEMPLATE__JWT_SECRET="this_is_a_very_long_secret_key_for_testing_purposes_only"

# Database pool configuration
export RUST_SERVICE_TEMPLATE__POOL_CONFIG__MAX_CONNECTIONS="20"
export RUST_SERVICE_TEMPLATE__POOL_CONFIG__MIN_CONNECTIONS="5"
export RUST_SERVICE_TEMPLATE__POOL_CONFIG__ACQUIRE_TIMEOUT="30"
export RUST_SERVICE_TEMPLATE__POOL_CONFIG__IDLE_TIMEOUT="300"
export RUST_SERVICE_TEMPLATE__POOL_CONFIG__MAX_LIFETIME="1800"

# Kafka configuration
export RUST_SERVICE_TEMPLATE__KAFKA_CONFIG__BOOTSTRAP_SERVERS="localhost:9092"
export RUST_SERVICE_TEMPLATE__KAFKA_CONFIG__CLIENT_ID="rust-service-template"

# CORS configuration (uncomment to customize)
# export RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_ORIGINS="*"
# export RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_METHODS="GET,POST,PUT,DELETE,OPTIONS"
# export RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOWED_HEADERS="*"
# export RUST_SERVICE_TEMPLATE__CORS_CONFIG__ALLOW_CREDENTIALS="false"
# export RUST_SERVICE_TEMPLATE__CORS_CONFIG__MAX_AGE="3600"

# Run the service
cargo run
