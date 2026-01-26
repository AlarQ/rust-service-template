# Error Handling Pattern

[← Configuration](06-configuration.md) | [Next: Main & Tracing →](08-main-and-tracing.md)

---

## Domain Errors (`domain/{feature}/models/errors.rs`)

```rust
use thiserror::Error;

/// Simplified error system with meaningful categories
#[derive(Debug, Error)]
pub enum DomainError {
    /// Resource not found
    #[error("Resource not found: {resource_type} with id '{id}'")]
    NotFound { resource_type: String, id: String },

    /// Input validation failures and malformed data
    #[error("Validation error: {message}")]
    ValidationError {
        message: String,
        field: Option<String>,
    },

    /// Domain logic violations (already exists, invalid transitions, limits)
    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String, rule: String },

    /// External system failures (database, external APIs)
    #[error("External system error: {message}")]
    ExternalError {
        message: String,
        source: Option<anyhow::Error>,
    },

    /// Access control violations
    #[error("Unauthorized access: {message}")]
    Unauthorized { message: String },
}

// Automatic conversions from external errors
impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        Self::ExternalError {
            message: format!("Database error: {error}"),
            source: Some(error.into()),
        }
    }
}

impl DomainError {
    // Convenience constructors
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
        }
    }

    pub fn field_validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    pub fn business_rule_violation(rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BusinessRuleViolation {
            rule: rule.into(),
            message: message.into(),
        }
    }

    pub fn external_error(message: impl Into<String>) -> Self {
        Self::ExternalError {
            message: message.into(),
            source: None,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }
}
```

---

## API Error Response (`api/error.rs`)

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiErrorResponse {
    #[schema(value_type = String)]
    pub code: ErrorCode,
}

#[derive(Debug, Serialize, Clone, utoipa::ToSchema)]
pub enum ErrorCode {
    NotFound,
    ValidationError,
    BadRequest,
    Unauthorized,
    InvalidToken,
    TokenNotFound,
    InternalServerError,
    DatabaseError,
    UnprocessableEntity,
    // Add feature-specific error codes...
}

impl From<ErrorCode> for ApiErrorResponse {
    fn from(code: ErrorCode) -> Self {
        Self { code }
    }
}

impl IntoResponse for ApiErrorResponse {
    fn into_response(self) -> Response {
        let status_code = match self.code {
            ErrorCode::NotFound => StatusCode::NOT_FOUND,
            ErrorCode::ValidationError | ErrorCode::BadRequest => StatusCode::BAD_REQUEST,
            ErrorCode::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorCode::Unauthorized | ErrorCode::TokenNotFound | ErrorCode::InvalidToken => {
                StatusCode::UNAUTHORIZED
            }
            ErrorCode::InternalServerError | ErrorCode::DatabaseError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (status_code, Json(self)).into_response()
    }
}

impl From<DomainError> for ApiErrorResponse {
    fn from(error: DomainError) -> Self {
        let code = match error {
            DomainError::NotFound { resource_type, id } => {
                tracing::error!(
                    error_type = "NotFound",
                    resource_type = %resource_type,
                    resource_id = %id,
                    "Resource not found"
                );
                ErrorCode::NotFound
            }
            DomainError::ValidationError { message, field } => {
                tracing::error!(
                    error_type = "ValidationError",
                    field = ?field,
                    error_message = %message,
                    "Validation error"
                );
                ErrorCode::ValidationError
            }
            DomainError::BusinessRuleViolation { message, rule } => {
                tracing::error!(
                    error_type = "BusinessRuleViolation",
                    rule = %rule,
                    error_message = %message,
                    "Business rule violation"
                );
                ErrorCode::BadRequest
            }
            DomainError::ExternalError { message, source } => {
                tracing::error!(
                    error_type = "ExternalError",
                    error_message = %message,
                    has_source = source.is_some(),
                    "External system error"
                );
                if message.contains("Database") {
                    ErrorCode::DatabaseError
                } else {
                    ErrorCode::InternalServerError
                }
            }
            DomainError::Unauthorized { message } => {
                tracing::error!(
                    error_type = "Unauthorized",
                    error_message = %message,
                    "Unauthorized access attempt"
                );
                ErrorCode::Unauthorized
            }
        };
        Self { code }
    }
}
```

---

## Error Flow

```
Domain Error → From<DomainError> → ApiErrorResponse → IntoResponse → HTTP Response
```

---

[← Configuration](06-configuration.md) | [Next: Main & Tracing →](08-main-and-tracing.md)
