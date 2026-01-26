# DDD Architecture Directory Structure

[← Code Quality Config](02-code-quality-config.md) | [Next: Architecture Rules →](04-architecture-rules.md)

---

## Directory Structure

```
src/
├── domain/                    # Pure business logic (NO external deps)
│   ├── mod.rs
│   ├── interfaces/            # ALL traits defined here
│   │   ├── mod.rs
│   │   ├── {feature}_repository.rs
│   │   └── event_producer.rs
│   └── {feature}/
│       ├── mod.rs
│       ├── services.rs        # FREE FUNCTIONS (no service structs!)
│       └── models/
│           ├── mod.rs
│           ├── {entity}.rs    # Entities & value objects
│           ├── errors.rs      # Domain errors (thiserror)
│           └── validation.rs
│
├── infrastructure/            # External integrations
│   ├── mod.rs
│   ├── {feature}.rs           # Repository implementations
│   └── kafka_producer.rs      # Event publishing
│
├── api/                       # HTTP layer (Axum)
│   ├── mod.rs                 # Router + OpenAPI setup
│   ├── auth.rs                # JWT extractor
│   ├── error.rs               # API error -> HTTP status
│   ├── models/                # DTOs (request/response)
│   │   └── {feature}.rs
│   └── {feature}/
│       └── handlers.rs
│
├── common.rs                  # Shared types (UserId, etc.)
├── config.rs                  # AppState & AppConfig
├── lib.rs                     # Module exports
└── main.rs                    # Entry point

tests/
├── integration_tests.rs       # Test entry point
├── common.rs                  # Shared test utilities
└── integration/
    ├── mod.rs
    ├── auth/
    ├── {feature}/
    └── health.rs
```

---

## Layer Responsibilities

| Layer | Purpose | Dependencies |
|-------|---------|--------------|
| `domain/` | Pure business logic | None (only stdlib) |
| `domain/interfaces/` | Trait definitions | Domain models only |
| `infrastructure/` | External integrations | Domain + external crates |
| `api/` | HTTP handlers | Domain + infrastructure |

---

[← Code Quality Config](02-code-quality-config.md) | [Next: Architecture Rules →](04-architecture-rules.md)
