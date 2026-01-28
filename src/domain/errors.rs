use thiserror::Error;

/// Domain errors representing business logic failures
///
/// These errors are converted to API responses via `From<DomainError> for ApiErrorResponse`
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
        #[source]
        source: Option<anyhow::Error>,
    },

    /// Access control violations
    #[error("Unauthorized access: {message}")]
    Unauthorized { message: String },
}

impl From<sqlx::Error> for DomainError {
    fn from(error: sqlx::Error) -> Self {
        Self::ExternalError {
            message: format!("Database error: {error}"),
            source: Some(error.into()),
        }
    }
}

impl DomainError {
    /// Create a not found error
    pub fn not_found(resource_type: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            resource_type: resource_type.into(),
            id: id.into(),
        }
    }

    /// Create a validation error without a specific field
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: None,
        }
    }

    /// Create a validation error for a specific field
    pub fn field_validation_error(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            message: message.into(),
            field: Some(field.into()),
        }
    }

    /// Create a business rule violation error
    pub fn business_rule_violation(rule: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BusinessRuleViolation {
            rule: rule.into(),
            message: message.into(),
        }
    }

    /// Create an external system error
    pub fn external_error(message: impl Into<String>) -> Self {
        Self::ExternalError {
            message: message.into(),
            source: None,
        }
    }

    /// Create an unauthorized error
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }
}
