use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::domain::errors::DomainError;

/// API error response returned to clients
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ApiErrorResponse {
    #[schema(value_type = String)]
    pub code: ErrorCode,
}

/// Error codes returned in API responses
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
