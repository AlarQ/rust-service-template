# File Upload Pattern

[← API Handlers](13-api-handlers.md) | [Next: Repository Pattern →](15-repository-pattern.md)

---

## Handler for File Upload

```rust
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};
use std::sync::Arc;
use tracing::{field::Empty, instrument};

use crate::{
    api::{auth::JwtExtractor, error::ApiErrorResponse, models::{feature}::*},
    config::AppState,
};

#[utoipa::path(
    post,
    path = "/v1/{features}/upload",
    request_body(content_type = "multipart/form-data"),
    security(("jwt" = [])),
    responses(
        (status = 200, body = FileUploadResponse),
        (status = 400, body = ApiErrorResponse, description = "Invalid file"),
        (status = 401, body = ApiErrorResponse, description = "Unauthorized"),
    )
)]
#[instrument(
    name = "[Upload File]",
    skip(app_state, claims, multipart),
    fields(file_size = Empty)
)]
pub async fn upload_file(
    State(app_state): State<Arc<AppState>>,
    JwtExtractor(claims): JwtExtractor,
    multipart: Multipart,
) -> Result<FileUploadResponse, ApiErrorResponse> {
    // Extract file from multipart
    let (file_name, file_bytes) = extract_file(multipart).await?;

    tracing::Span::current().record("file_size", file_bytes.len());

    // Process file...
    let result = domain::{feature}::process_file(
        file_bytes,
        file_name,
    )
    .await?;

    Ok(result.into())
}
```

---

## Multipart Field Extraction Helper

```rust
use axum::extract::Multipart;

/// Extract a single file from multipart form data
pub async fn extract_file(
    mut multipart: Multipart,
) -> Result<(String, Vec<u8>), ApiErrorResponse> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiErrorResponse::validation_error(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();

        if name == "file" {
            let file_name = field
                .file_name()
                .map(String::from)
                .unwrap_or_else(|| "upload".to_string());

            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiErrorResponse::validation_error(format!("Failed to read file: {e}")))?;

            return Ok((file_name, bytes.to_vec()));
        }
    }

    Err(ApiErrorResponse::validation_error("No file field found in multipart request"))
}

/// Extract multiple fields from multipart (file + JSON data)
pub async fn extract_multipart_fields(
    mut multipart: Multipart,
) -> Result<MultipartFields, ApiErrorResponse> {
    let mut file_name = None;
    let mut file_bytes = None;
    let mut json_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiErrorResponse::validation_error(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or_default().to_string();

        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(String::from);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| ApiErrorResponse::validation_error(format!("File read error: {e}")))?
                        .to_vec(),
                );
            }
            "data" | "metadata" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| ApiErrorResponse::validation_error(format!("Text field error: {e}")))?;
                json_data = Some(text);
            }
            _ => {
                tracing::debug!("Ignoring unknown multipart field: {}", name);
            }
        }
    }

    Ok(MultipartFields {
        file_name: file_name.unwrap_or_else(|| "upload".to_string()),
        file_bytes: file_bytes.ok_or_else(|| {
            ApiErrorResponse::validation_error("No file field in multipart request")
        })?,
        json_data,
    })
}

pub struct MultipartFields {
    pub file_name: String,
    pub file_bytes: Vec<u8>,
    pub json_data: Option<String>,
}
```

---

## Response DTOs

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FileUploadResponse {
    pub file_id: uuid::Uuid,
    pub file_name: String,
    pub file_size: usize,
    pub content_type: Option<String>,
}
```

---

[← API Handlers](13-api-handlers.md) | [Next: Repository Pattern →](15-repository-pattern.md)
