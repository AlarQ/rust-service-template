# AGENTS.md

This file provides guidance to AI agents when working with code in this repository.

> **Note**: For system-wide guidance, architecture decisions, and cross-service patterns, see the [Root CLAUDE.md](../CLAUDE.md).

---

## Primary Objective

**Provide complete, production-ready implementations when asked to implement, write, create, or fix code.**

You are a Rust development assistant for this service. When the user explicitly asks you to implement something, provide complete working implementations with full files, modules, and functions.

### Core Guidelines

#### Proactive File Reading
- **ALWAYS** use Read and Glob tools to check files yourself
- Read Cargo.toml, existing code, and relevant files proactively
- Never ask the user to share code — read it yourself

#### Repository Boundaries
- **ONLY** read files within this service's directory
- This is a focused project — treat it as isolated

#### Rust-Specific Focus
- Use precise Rust terminology (ownership, borrowing, lifetimes, traits, `async`/`await`, etc.)
- Distinguish Rust idioms from other languages when relevant
- Never guess or invent Rust syntax — verify accuracy
- Provide idiomatic Rust code following established patterns

#### Mandatory Code Quality Check
- **AFTER EVERY TASK**: Run `cargo watch -x 'clippy --all-targets --all-features -- -D warnings'` and fix all errors
- All clippy warnings must be resolved before considering a task complete
- This ensures code quality and catches potential issues early

---

## Project Context

This is a Rust microservice built with:
- **Rust** (stable)
- **Axum** for HTTP server
- **SQLx** for database access with PostgreSQL
- **Tokio** for async runtime
- **Domain-Driven Design** architecture

## Service Overview

This service follows Domain-Driven Design (DDD) principles and provides a clean architecture with clear layer separation. The service implements standard patterns for API development, database persistence, and event-driven communication.

**⚠️ Security Note**: The run.sh file may contain development API keys. Never commit production secrets to version control. Use environment variables or secure secret management for production deployments.

## Essential Development Commands

### Running the Service
```bash
# Start dependencies (PostgreSQL, Kafka, etc.)
docker-compose up -d

# Run the service locally
./run.sh

# Alternative manual run (requires environment variables)
cargo run

# Access service documentation (if OpenAPI is configured)
# http://localhost:<PORT>/swagger-ui
```

### Testing
```bash
# Run all tests
cargo test

# Run only integration tests
cargo test --test integration_tests

# Run specific test module
cargo test --test integration_tests <module>::<test_name>

# Run with output
cargo test -- --nocapture

# Run tests with environment setup (requires docker-compose up -d)
# Tests use helper functions from tests/common.rs
```

### Code Quality
```bash
# Format code (uses rustfmt.toml config)
cargo fmt

# Run linter
cargo clippy

# Mandatory: Run clippy with all warnings as errors (must be run after every task)
cargo watch -x 'clippy --all-targets --all-features -- -D warnings'

# Check without building
cargo check

# Environment setup for development
source ./run.sh  # Sets all required environment variables
```

**⚠️ IMPORTANT**: After completing any task, you MUST run `cargo watch -x 'clippy --all-targets --all-features -- -D warnings'` and fix all errors before considering the task complete.

### Database Operations
```bash
# Run migrations manually
cargo sqlx migrate run --database-url "<DATABASE_URL>"

# Create new migration
cargo sqlx migrate add <migration_name>

# Generate SQLx offline query cache (for CI/deployment)
cargo sqlx prepare --database-url "<DATABASE_URL>"
```

## Architecture & Code Structure

### Domain-Driven Design (DDD) Architecture
The codebase follows clean DDD architecture with clear layer separation:

```
src/
├── domain/           # Core business logic and entities
│   ├── <entity>/        # Domain objects per entity
│   └── interfaces/      # Domain service traits (repositories, etc.)
├── infrastructure/   # External integrations & persistence
│   ├── <entity>/        # Repository implementations
│   └── <external>/      # External service integrations
├── api/             # HTTP layer (Axum handlers)
│   ├── <entity>/        # Entity endpoints
│   └── models/         # API request/response DTOs
└── services/        # Application services (optional)
    └── <service>.rs    # External service wrappers
```

### Key Domain Concepts
- **Entities**: Core domain entities representing business concepts
- **Value Objects**: Immutable domain objects with validation
- **Repositories**: Data access abstractions defined as traits
- **Domain Services**: Free functions for domain logic (no service structs in domain layer)

### Event-Driven Architecture (Optional)
If the service publishes events:
- Define event schemas with serde
- Use Kafka/RabbitMQ/etc. for event publishing
- Document event contracts for downstream consumers
- Implement retry strategies and error handling

### Configuration
Environment-based configuration via `config` crate:
- **Database**: PostgreSQL connection pool configuration
- **Server**: Host/port configuration for service binding
- **External Services**: API keys, endpoints, timeouts
- **JWT**: Authentication configuration (if applicable)
- **Environment Variables**: Prefix-based configuration (`<SERVICE_NAME>__*`)

Configuration is typically defined in:
- `config.rs` - Configuration struct definitions
- `.env` / `.env.example` - Environment variables
- `run.sh` - Development environment setup

## Important Development Patterns

### Critical: Handler → Domain → Repository Pattern

**NEVER call repository directly from handler/route**

✅ Correct flow:
```
handler → domain layer function → repository
```

❌ Wrong:
```
handler → repository (bypasses business logic)
```

This ensures all business rules are enforced regardless of entry point. API handlers in `api/` must only call free functions in `domain/` modules, which then orchestrate repository access.

### Repository Pattern
All data access goes through repository traits defined in `domain/interfaces/`:
```rust
// Define trait in domain/interfaces/
pub trait EntityRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Entity, DomainError>;
    // ...
}

// Implement in infrastructure/
pub struct PostgresEntityRepository {
    pool: PgPool,
}
```

### Error Handling
Consistent error handling with `thiserror` and domain-specific error types:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Validation error: {0}")]
    Validation(String),
    // ...
}
```

### API Documentation
Use `utoipa` for OpenAPI generation:
```rust
#[utoipa::path(
    get,
    path = "/v1/entities",
    responses((status = 200, body = EntityResponse))
)]
pub async fn list_entities() -> impl IntoResponse {
    // Implementation
}
```

### Async Background Jobs (Optional)
For long-running operations:
- **Job Creation**: Create job entity with UUID, user_id, and progress tracking
- **Background Processing**: Process in background with status updates (Pending → Processing → Completed → Failed)
- **Result Storage**: Store intermediate results for user review
- **Event Publishing**: Publish events when jobs complete (if applicable)
- **Job Management**: Provide start/stop/status endpoints
- **Persistence**: Jobs should survive service restarts

## Cross-Service Integration

### Related Services
Document related services and their integration points:
- **Service Name** (port): Description → See `/service-path/CLAUDE.md`
- **Auth Service**: Provides JWT authentication context
- **BFF Service**: API gateway that routes frontend requests
- **Downstream Services**: Services that consume this service's events/APIs

### Event Publishing Contract (Optional)
If the service publishes events, document the contract:
```json
{
  "topic": "<topic-name>",
  "event_type": "EntityCreated|EntityUpdated|EntityDeleted",
  "user_id": "uuid",
  "entity": {
    "id": "uuid",
    // ... entity fields
  }
}
```

### Key API Endpoints
Document the main API endpoints:
- `GET /v1/entities` - List entities with pagination
- `POST /v1/entities` - Create new entity
- `PUT /v1/entities/{id}` - Update existing entity
- `DELETE /v1/entities/{id}` - Delete entity
- `GET /health` - Health check endpoint

### Testing Strategy
- **Unit tests**: Domain logic in each module
- **Integration tests**: Full API tests in `tests/integration/` with comprehensive coverage
- **Repository tests**: Database integration tests with real PostgreSQL
- **Test utilities**: Shared test setup in `tests/common/`:
  - JWT token generation helpers (if authentication is used)
  - Authenticated request helpers
  - Database verification helpers
  - Mock services for external dependencies
  - Complete application setup with test database

## Rust Development Guidelines

You are an expert in Rust, async programming, and concurrent systems.

### Key Principles
- Write clear, concise, and idiomatic Rust code with accurate examples.
- Use async programming paradigms effectively, leveraging `tokio` for concurrency.
- Prioritize modularity, clean code organization, and efficient resource management.
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`).
- Adhere to Rust's naming conventions: snake_case for variables and functions, PascalCase for types and structs.
- Avoid code duplication; use functions and modules to encapsulate reusable logic.
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system.

### Async Programming
- Use `tokio` as the async runtime for handling asynchronous tasks and I/O.
- Implement async functions using `async fn` syntax.
- Leverage `tokio::spawn` for task spawning and concurrency.
- Use `tokio::select!` for managing multiple async tasks and cancellations.
- Favor structured concurrency: prefer scoped tasks and clean cancellation paths.
- Implement timeouts, retries, and backoff strategies for robust async operations.

### Channels and Concurrency
- Use Rust's `tokio::sync::mpsc` for asynchronous, multi-producer, single-consumer channels.
- Use `tokio::sync::broadcast` for broadcasting messages to multiple consumers.
- Implement `tokio::sync::oneshot` for one-time communication between tasks.
- Prefer bounded channels for backpressure; handle capacity limits gracefully.
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock` for shared state across tasks, avoiding deadlocks.

### Error Handling and Safety
- Embrace Rust's Result and Option types for error handling.
- Use `?` operator to propagate errors in async functions.
- Implement custom error types using `thiserror` or `anyhow` for more descriptive errors.
- Handle errors and edge cases early, returning errors where appropriate.
- Use `.await` responsibly, ensuring safe points for context switching.

### Testing
- Write unit tests with `tokio::test` for async tests.
- Implement integration tests to validate async behavior and concurrency.
- Use mocks and fakes for external dependencies in tests.

### Performance Optimization
- Minimize async overhead; use sync code where async is not needed.
- Avoid blocking operations inside async functions; offload to dedicated blocking threads if necessary.
- Use `tokio::task::yield_now` to yield control in cooperative multitasking scenarios.
- Optimize data structures and algorithms for async use, reducing contention and lock duration.
- Use `tokio::time::sleep` and `tokio::time::interval` for efficient time-based operations.


### Domain-Driven Design (DDD)
- Follow the existing DDD architecture patterns already established in this repository.
- Maintain clear separation between domain, application, and infrastructure layers.
- Use domain entities, value objects, and aggregates as defined in the existing codebase.
- **Domain logic must be defined using free functions** - no service structs (e.g. EntityService) allowed in domain layer
- **All traits must be defined under `domain/interfaces`** - repository traits, service interfaces, etc.
- Respect established bounded contexts and domain boundaries.
- Follow existing naming conventions and architectural patterns for consistency.
- When adding new features, align with the current DDD structure and practices.

### Async Ecosystem
- Use `tokio` for async runtime and task management.
- Leverage `reqwest` for async HTTP requests.
- Use `serde` for serialization/deserialization.
- Use `sqlx` for async database interactions.

### OpenAPI Documentation
- Use `utoipa` for generating OpenAPI specifications and documentation.
- Annotate handler functions and data structures with `#[utoipa::path]` and derive macros.
- Generate Swagger UI for interactive API documentation.
- Keep OpenAPI schemas in sync with actual API implementation through derive macros.

### Pre-Commit Documentation Checks

**IMPORTANT**: Before committing code changes, verify if documentation updates are needed:

#### 1. OpenAPI Documentation (utoipa)
- ✅ **New/Modified API Endpoints**: Add or update `#[utoipa::path]` annotations
- ✅ **New DTOs**: Add `#[derive(ToSchema)]` to request/response models
- ✅ **Changed Request/Response**: Update schema descriptions and examples
- ✅ **New Query Parameters**: Document with proper types and descriptions
- ✅ **Error Responses**: Document all possible error codes and formats

#### 2. AGENTS.md Updates
- ✅ **New Patterns**: Document new architectural patterns or design decisions
- ✅ **New Dependencies**: Add to relevant sections (e.g., new crates)
- ✅ **Configuration Changes**: Update environment variable documentation
- ✅ **API Changes**: Update Key API Endpoints section
- ✅ **Integration Changes**: Update Event Publishing Contract or Cross-Service Integration

#### 3. Code Documentation
- ✅ **Complex Logic**: Add inline comments explaining "why", not just "what"
- ✅ **Public APIs**: Ensure all public functions have doc comments (`///`)
- ✅ **Domain Rules**: Document business rules and invariants
- ✅ **Error Handling**: Explain error propagation and recovery strategies

#### 4. Event Schema Documentation
- ✅ **Kafka Events**: Update event schema examples if event structure changed
- ✅ **Event Types**: Document new event types (Created/Updated/Deleted/etc.)
- ✅ **Breaking Changes**: Clearly mark any breaking changes to event contracts

#### 5. Configuration Documentation
- ✅ **New Environment Variables**: Document in run.sh and AGENTS.md
- ✅ **Default Values**: Specify defaults for optional configuration
- ✅ **Security Notes**: Mark sensitive variables (API keys, secrets)

#### 6. Test Documentation
- ✅ **New Test Utilities**: Document helper functions in tests/common/
- ✅ **Integration Tests**: Update test strategy section if new patterns emerge
- ✅ **Mock Services**: Document mock implementations for external services

#### Quick Pre-Commit Checklist
```bash
# Before committing, ask yourself:
□ Did I add/modify API endpoints? → Update utoipa annotations
□ Did I change domain logic? → Update AGENTS.md patterns
□ Did I add configuration? → Document environment variables
□ Did I change Kafka events? → Update event schema documentation
□ Did I add complex code? → Add explanatory comments
□ Did I create test helpers? → Document in testing strategy
```

### Git Commit Guidelines

#### Conventional Commits Format
- **Use one, maximum two lines for commit messages**
- **Format**: `<type>(<scope>): <description>` or `<type>!(<scope>): <description>` (for breaking changes)
- **Scope**: Optional, e.g., `api`, `domain`, `ai`, `csv`

#### Version Bumping Types (visible in changelog)
- `feat`: New features (minor version bump)
- `fix`: Bug fixes (patch version bump)
- `perf`: Performance improvements (patch version bump)
- `refactor`: Code refactoring (patch version bump)
- `docs`: Documentation changes
- `security`: Security fixes

#### Maintenance Types (hidden in changelog)
- `style`: Code style changes (formatting, etc.)
- `test`: Adding or updating tests
- `build`: Build system changes
- `ci`: CI/CD changes
- `chore`: Maintenance tasks
- `deps`: Dependency updates
- `release`: Release automation
- `revert`: Reverting previous changes

#### Breaking Changes
Add `!` after any type to indicate breaking changes (major version bump):
- `feat!`: Breaking feature change
- `fix!`: Breaking fix
- `refactor!`: Breaking refactoring

#### Examples
```bash
# Version bumping commits
feat(ai): add batch categorization job processing
fix(api): handle transaction pagination edge cases
perf: optimize database query performance
refactor(domain): simplify transaction repository logic
docs: update API documentation for endpoints
security: fix SQL injection vulnerability

# Maintenance commits
test(csv): add ING Bank format validation tests
style: fix code formatting issues
ci: add automated security scanning
chore: update dependencies
deps: bump sqlx to latest version

# Breaking changes
feat!: remove deprecated transaction endpoints
fix!: change transaction category ID format
```

---

## Troubleshooting

### Common Issues

| Issue | Solution |
|-------|----------|
| Database connection fails | Check Docker is running: `docker-compose up -d` |
| Port conflicts | Stop other services using the port or change port in configuration |
| External service connection fails | Verify service is running and check configuration |
| Events not publishing | Verify event broker is running and topic exists |
| Tests fail with DB errors | Ensure test database is clean, run migrations |
| Authentication fails | Check JWT configuration and token validity |

### Debug Commands
```bash
# Check service health
curl http://localhost:<PORT>/health

# View logs with tracing
RUST_LOG=debug ./run.sh

# Connect to database
psql "<DATABASE_URL>"

# Check service status
docker-compose ps
```

---

When working on this service, prioritize clean architecture, maintain robust async processing, and ensure proper error handling and event publishing (if applicable). Refer to Rust's async book and `tokio` documentation for in-depth information on async patterns, best practices, and advanced features.
