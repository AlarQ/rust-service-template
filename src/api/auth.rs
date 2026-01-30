/// JWT authentication module for handling token-based authentication.
/// This module provides functionality for JWT token validation and generation.
use axum::{http::request::Parts, RequestPartsExt};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    api::error::{ApiErrorResponse, ErrorCode},
    config::AppState,
};

/// Minimum length required for JWT secret
const MIN_SECRET_LENGTH: usize = 32;

fn get_keys(secret: &str) -> Result<Keys, ApiErrorResponse> {
    if secret.len() < MIN_SECRET_LENGTH {
        return Err(ApiErrorResponse::from(ErrorCode::InternalServerError));
    }
    Ok(Keys::new(secret.as_bytes()))
}

/// Extract JWT claims from a token string using a secret
pub fn extract_jwt_claims(token: &str, secret: &str) -> Result<JwtClaims, ApiErrorResponse> {
    let mut validation = Validation::default();
    validation.set_audience(&["rust-service-template"]);
    // Allow empty sub field for service-to-service authentication
    validation.sub = None;

    decode::<JwtClaims>(token, &get_keys(secret)?.decoding, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|err| {
            tracing::error!("Invalid token: {}", err);
            ApiErrorResponse::from(ErrorCode::InvalidToken)
        })
}

/// Custom JWT extractor that uses app state to get the secret
pub struct JwtExtractor(pub JwtClaims);

impl axum::extract::FromRequestParts<Arc<AppState>> for JwtExtractor {
    type Rejection = ApiErrorResponse;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        tracing::info!("Checking for JWT token in request");

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| {
                tracing::error!("Token not found in request");
                ApiErrorResponse::from(ErrorCode::TokenNotFound)
            })?;

        tracing::debug!("Processing JWT token for authentication");

        let claims = extract_jwt_claims(bearer.token(), &state.env.jwt_secret)?;

        tracing::info!("Token decoded successfully");

        Ok(Self(claims))
    }
}

struct Keys {
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, utoipa::ToSchema, Clone)]
pub struct JwtClaims {
    pub sub: Option<String>,
    pub aud: Option<String>,
    pub exp: usize,
    pub iss: Option<String>,
    pub session_id: Option<String>,
}

impl JwtClaims {
    /// Get the session ID if present
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Validate that the `user_id` from the path matches the subject claim in the JWT token.
    /// Returns an error if the claims don't have a subject or if it doesn't match the `user_id`.
    pub fn validate_user_id(&self, user_id: Uuid) -> Result<(), ApiErrorResponse> {
        let claims_user_id = self
            .sub
            .as_ref()
            .ok_or_else(|| {
                tracing::error!("JWT token missing subject claim");
                ApiErrorResponse::from(ErrorCode::Unauthorized)
            })?
            .parse::<Uuid>()
            .map_err(|_| {
                tracing::error!("Invalid user_id format in JWT subject claim");
                ApiErrorResponse::from(ErrorCode::Unauthorized)
            })?;

        if claims_user_id != user_id {
            tracing::warn!(
                "User ID mismatch: token user_id={}, path user_id={}",
                claims_user_id,
                user_id
            );
            return Err(ApiErrorResponse::from(ErrorCode::Unauthorized));
        }

        Ok(())
    }
}
