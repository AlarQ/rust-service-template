# JWT Authentication

[← CORS](11-cors.md) | [Next: API Handlers →](13-api-handlers.md)

---

## `api/auth.rs`

```rust
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

const MIN_SECRET_LENGTH: usize = 32;

fn get_keys(secret: &str) -> Keys {
    assert!(
        secret.len() >= MIN_SECRET_LENGTH,
        "JWT_SECRET must be at least {MIN_SECRET_LENGTH} characters long"
    );
    Keys::new(secret.as_bytes())
}

pub fn extract_jwt_claims(token: &str, secret: &str) -> Result<JwtClaims, ApiErrorResponse> {
    let mut validation = Validation::default();
    validation.set_audience(&["{service-name}"]);
    validation.sub = None;

    decode::<JwtClaims>(token, &get_keys(secret).decoding, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|err| {
            tracing::error!("Invalid token: {}", err);
            ApiErrorResponse::from(ErrorCode::InvalidToken)
        })
}

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
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

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
```

---

## Usage in Handlers

```rust
async fn my_handler(
    JwtExtractor(claims): JwtExtractor,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiErrorResponse> {
    // Validate that the user_id from the path matches the JWT claims
    claims.validate_user_id(user_id)?;
    
    // Handler logic...
}
```

---

[← CORS](11-cors.md) | [Next: API Handlers →](13-api-handlers.md)
