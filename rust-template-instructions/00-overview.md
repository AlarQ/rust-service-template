# Rust Microservice Template

A comprehensive template for creating production-ready Rust microservices following Domain-Driven Design (DDD) patterns.

---

## Table of Contents

### Phase 1: Prerequisites and Project Setup
1. [Prerequisites & Git Hooks](01-prerequisites.md) - Required tools, git hooks configuration
2. [Code Quality Config](02-code-quality-config.md) - Clippy and Rustfmt configuration
3. [Project Structure](03-project-structure.md) - DDD directory structure
4. [Architecture Rules](04-architecture-rules.md) - Critical architecture rules
5. [Dependencies](05-dependencies.md) - Cargo.toml template

### Phase 2: Core Application Setup
6. [Configuration](06-configuration.md) - AppConfig, AppState, environment
7. [Error Handling](07-error-handling.md) - Domain and API error patterns
8. [Main & Tracing](08-main-and-tracing.md) - main.rs and tracing setup

### Phase 3: API Layer
9. [API Router](09-api-router.md) - Router, server startup, OpenAPI
10. [Health Checks](10-health-checks.md) - Health and readiness endpoints
11. [CORS](11-cors.md) - CORS configuration
12. [JWT Authentication](12-jwt-authentication.md) - JWT extractor and validation
13. [API Handlers](13-api-handlers.md) - Handler pattern with examples
14. [File Uploads](14-file-uploads.md) - Multipart file upload handling

### Phase 4: Domain Layer
15. [Repository Pattern](15-repository-pattern.md) - Repository interface and implementation
16. [Domain Services](16-domain-services.md) - Free function domain services
17. [Value Objects](17-value-objects.md) - Value objects with validation
18. [DTO Conversion](18-dto-conversion.md) - Request/Response DTO patterns
19. [Background Jobs](19-background-jobs.md) - Async background job pattern

### Phase 5: Infrastructure
20. [Event Schema](20-event-schema.md) - Event schema and factory methods
21. [Kafka Producer](21-kafka-producer.md) - Kafka producer configuration

### Phase 6: Testing and DevOps
22. [Testing](22-testing.md) - Test utilities, helpers, examples
23. [Docker Compose](23-docker-compose.md) - Docker Compose template
24. [Run Script](24-run-script.md) - run.sh template
25. [CLAUDE.md](25-claude-md.md) - CLAUDE.md template
26. [Quick Start Checklist](26-quick-start-checklist.md) - Checklist and placeholder reference
27. [Git Workflows](27-git-workflows.md) - GitHub Actions CI/CD and releases

---

## How to Use This Template

1. **Read in order**: Files are numbered to follow the logical setup sequence
2. **Replace placeholders**: See [26-quick-start-checklist.md](26-quick-start-checklist.md) for placeholder reference
3. **Skip optional sections**: File uploads, background jobs, and Kafka are optional based on your needs

---

[Next: Prerequisites & Git Hooks →](01-prerequisites.md)
