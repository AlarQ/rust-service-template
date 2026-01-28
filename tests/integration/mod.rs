pub mod tasks;

use axum::{body::Body, http::Request};
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::common;
use axum::Router;
use rust_service_template::{
    common::UserId,
    domain::{
        interfaces::task_repository::TaskRepository,
        task::models::{Task, TaskPriority},
    },
    infrastructure::task::PostgresTaskRepository,
};

/// Helper function to make unauthenticated HTTP requests
///
/// Creates and executes an HTTP request to the test application.
///
/// # Arguments
/// - `app`: The axum Router to send the request to
/// - `method`: HTTP method (e.g., "GET", "POST", "PUT", "DELETE")
/// - `uri`: Request URI path (e.g., "/tasks", "/tasks/123")
/// - `body`: Optional request body for POST/PUT requests
///
/// # Returns
/// A tuple containing:
/// - Status code as u16 (e.g., 200, 404, 500)
/// - Response body as Vec<u8>
pub async fn make_request(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<Body>,
) -> (u16, Vec<u8>) {
    let mut request_builder = Request::builder().method(method).uri(uri);

    if body.is_some() {
        request_builder = request_builder.header("Content-Type", "application/json");
    }

    let request = if let Some(body) = body {
        request_builder.body(body).unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    let response: axum::response::Response = app.clone().oneshot(request).await.unwrap();
    let status = response.status().as_u16();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, body_bytes.to_vec())
}

/// Helper function to create a JSON request body from a string
///
/// Converts a JSON string into a Body for HTTP requests.
///
/// # Arguments
/// - `data`: JSON string (e.g., r#"{"title": "Test"}"#)
///
/// # Returns
/// An axum Body containing the JSON data
pub fn create_json_body(data: &str) -> Body {
    Body::from(data.to_string())
}

/// Helper function to parse JSON response bytes into a Value
///
/// Attempts to parse response bytes as JSON. If parsing fails,
/// creates a default error response structure.
///
/// # Arguments
/// - `body_bytes`: Raw response body bytes
///
/// # Returns
/// A serde_json::Value containing the parsed response
pub fn parse_json_response(body_bytes: &[u8]) -> Value {
    serde_json::from_slice(body_bytes).unwrap_or_else(|_| {
        // If we can't parse as JSON, check if it's a plain text error from Axum
        let text = String::from_utf8_lossy(body_bytes);
        if text.contains("missing field") || text.contains("Failed to deserialize") {
            // This is an Axum JSON deserialization error, return BadRequest
            serde_json::json!({
                "code": "BadRequest"
            })
        } else {
            // If we can't parse as JSON, create a default error response
            serde_json::json!({
                "code": "InvalidResponse"
            })
        }
    })
}

/// Helper function to verify error response contains expected error code
///
/// Parses response as JSON and asserts the "code" field matches expected_code.
///
/// # Arguments
/// - `body_bytes`: Raw response body bytes
/// - `expected_code`: Expected error code string (e.g., "NotFound", "BadRequest")
///
/// # Panics
/// If response is not valid JSON or code doesn't match
pub fn verify_error_response(body_bytes: &[u8], expected_code: &str) {
    let body: serde_json::Value =
        serde_json::from_slice(body_bytes).expect("Response should be valid JSON");
    let code = body["code"]
        .as_str()
        .expect("Error code should be a string");
    assert_eq!(
        code, expected_code,
        "Expected error code {} but got {}",
        expected_code, code
    );
}

/// Helper function to create a test task and insert it into the database
///
/// Creates a Task domain object and persists it using the repository.
/// This is useful for setting up test data without going through the API.
///
/// # Arguments
/// - `pool`: Database connection pool
/// - `user_id`: User ID to associate the task with
/// - `title`: Task title
/// - `description`: Optional task description
/// - `priority`: Task priority level
///
/// # Returns
/// The created Task object with all fields populated
pub async fn create_test_task(
    pool: &sqlx::PgPool,
    user_id: UserId,
    title: &str,
    description: Option<String>,
    priority: TaskPriority,
) -> Task {
    let task = Task::new(user_id, title.to_string(), description, priority).unwrap();
    let repo = PostgresTaskRepository::new(pool.clone());
    repo.create(task.clone()).await.unwrap();
    task
}

/// Helper function to check if a task exists in the database
///
/// Queries the database to verify a task with the given ID exists.
///
/// # Arguments
/// - `pool`: Database connection pool
/// - `task_id`: Task ID to check for existence
///
/// # Returns
/// true if task exists, false otherwise
pub async fn task_exists_in_db(pool: &sqlx::PgPool, task_id: &Uuid) -> bool {
    sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM tasks WHERE id = $1)")
        .bind(task_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Helper function to generate a unique task title
///
/// Creates a unique title by combining a prefix with a UUID.
/// Useful for preventing duplicate task title issues in tests.
///
/// # Arguments
/// - `prefix`: Title prefix (e.g., "test_task", "my_task")
///
/// # Returns
/// A unique task title string
pub fn generate_unique_title(prefix: &str) -> String {
    format!("{}_{:x}", prefix, Uuid::new_v4())
}
