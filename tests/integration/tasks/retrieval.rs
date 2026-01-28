use super::super::*;
use rust_service_template::domain::task::models::{TaskPriority, TaskStatus};

// Helper functions to convert domain enums to database string representations
fn status_to_db_string(status: TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "PENDING",
        TaskStatus::InProgress => "IN_PROGRESS",
        TaskStatus::Completed => "COMPLETED",
        TaskStatus::Cancelled => "CANCELLED",
    }
}

fn priority_to_db_string(priority: TaskPriority) -> &'static str {
    match priority {
        TaskPriority::Low => "LOW",
        TaskPriority::Medium => "MEDIUM",
        TaskPriority::High => "HIGH",
        TaskPriority::Critical => "CRITICAL",
    }
}

#[tokio::test]
async fn test_get_task_returns_200_for_existing_task() {
    // Objective: Verify retrieving an existing task returns correct data
    // Positive test: GET request with valid ID should return task
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("get_task");

    // Arrange: Create a test task in database
    let task = create_test_task(&pool, user_id, &title, None, TaskPriority::Medium).await;

    // Act: Send GET request for the task
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task.id), None).await;

    // Assert: Verify 200 OK with correct task data
    assert_eq!(status, 200, "Should return 200 OK for existing task");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["id"], task.id.to_string(), "ID should match");
    assert_eq!(body["title"], title, "Title should match");
    assert_eq!(body["status"], "Pending", "Status should be Pending");
    assert_eq!(body["priority"], "Medium", "Priority should be Medium");
    assert_eq!(body["user_id"], user_id.to_string(), "User ID should match");
}

#[tokio::test]
async fn test_get_task_returns_404_for_non_existent_task() {
    // Objective: Verify non-existent task returns 404
    // Negative test: GET request with invalid ID should fail
    let (app, _) = common::app().await;

    // Arrange: Use a random UUID that doesn't exist in DB
    let fake_id = uuid::Uuid::new_v4();

    // Act: Send GET request for non-existent task
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", fake_id), None).await;

    // Assert: Verify 404 Not Found
    assert_eq!(
        status, 404,
        "Should return 404 Not Found for non-existent task"
    );
    verify_error_response(&body_bytes, "NotFound");
}

#[tokio::test]
async fn test_get_task_returns_400_for_invalid_uuid_format() {
    // Objective: Verify invalid UUID format is rejected
    // Negative test: Malformed UUID should return 400
    let (app, _) = common::app().await;

    // Arrange: Use invalid UUID format
    let invalid_id = "not-a-uuid";

    // Act: Send GET request with invalid UUID
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", invalid_id), None).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(
        status, 400,
        "Should return 400 Bad Request for invalid UUID"
    );
    verify_error_response(&body_bytes, "BadRequest");
}

#[tokio::test]
async fn test_get_task_returns_200_for_task_with_empty_description() {
    // Objective: Verify task with no description is retrieved correctly
    // Positive test: Tasks without description should be returned
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("no_desc");

    // Arrange: Create task without description
    let task = create_test_task(&pool, user_id, &title, None, TaskPriority::High).await;

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task.id), None).await;

    // Assert: Verify 200 OK with null description
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(
        body["description"],
        serde_json::Value::Null,
        "Description should be null"
    );
    assert_eq!(body["title"], title, "Title should match");
    assert_eq!(body["priority"], "High", "Priority should be High");
}

#[tokio::test]
async fn test_get_task_returns_200_for_task_with_description() {
    // Objective: Verify task with description is retrieved correctly
    // Positive test: Task description should be preserved
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("with_desc");
    let description = "This is a test description".to_string();

    // Arrange: Create task with description
    let task = create_test_task(
        &pool,
        user_id,
        &title,
        Some(description.clone()),
        TaskPriority::Low,
    )
    .await;

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task.id), None).await;

    // Assert: Verify 200 OK with description
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["description"], description, "Description should match");
    assert_eq!(body["title"], title, "Title should match");
}

#[tokio::test]
async fn test_get_task_returns_200_for_completed_task() {
    // Objective: Verify completed task is retrieved with correct status
    // Positive test: Completed tasks should have correct status field
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("completed_task");

    // Arrange: Create a completed task directly in DB
    let task_id = uuid::Uuid::new_v4();
    let completed_at = chrono::Utc::now();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
        VALUES ($1, $2, $3, $4, $5::task_status, $6::task_priority, $7, $8, $9)
        "#,
    )
    .bind(task_id)
    .bind(user_id.into_inner())
    .bind(title)
    .bind::<Option<String>>(None)
    .bind(status_to_db_string(TaskStatus::Completed))
    .bind(priority_to_db_string(TaskPriority::Medium))
    .bind(completed_at)
    .bind(completed_at)
    .bind(completed_at)
    .execute(&*pool)
    .await
    .unwrap();

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task_id), None).await;

    // Assert: Verify 200 OK with completed status and completed_at
    assert_eq!(status, 200, "Should return 200 OK for completed task");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["status"], "Completed", "Status should be Completed");
    assert!(
        body["completed_at"].is_string(),
        "completed_at should be a string timestamp"
    );
    assert!(
        body.get("completed_at").is_some(),
        "Should have completed_at field"
    );
}

#[tokio::test]
async fn test_get_task_returns_200_for_task_with_in_progress_status() {
    // Objective: Verify task with InProgress status is retrieved correctly
    // Positive test: Different task statuses should be preserved
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("in_progress");

    // Arrange: Create InProgress task directly in DB
    let task_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
        VALUES ($1, $2, $3, $4, $5::task_status, $6::task_priority, $7, $8, $9)
        "#,
    )
    .bind(task_id)
    .bind(user_id.into_inner())
    .bind(title)
    .bind::<Option<String>>(None)
    .bind(status_to_db_string(TaskStatus::InProgress))
    .bind(priority_to_db_string(TaskPriority::High))
    .bind(now)
    .bind(now)
    .bind::<Option<chrono::DateTime<chrono::Utc>>>(None)
    .execute(&*pool)
    .await
    .unwrap();

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task_id), None).await;

    // Assert: Verify 200 OK with InProgress status
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["status"], "InProgress", "Status should be InProgress");
    assert_eq!(body["priority"], "High", "Priority should be High");
}

#[tokio::test]
async fn test_get_task_returns_200_for_task_with_cancelled_status() {
    // Objective: Verify cancelled task is retrieved correctly
    // Positive test: Cancelled status should be preserved
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("cancelled");

    // Arrange: Create Cancelled task directly in DB
    let task_id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    sqlx::query(
        r#"
        INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
        VALUES ($1, $2, $3, $4, $5::task_status, $6::task_priority, $7, $8, $9)
        "#,
    )
    .bind(task_id)
    .bind(user_id.into_inner())
    .bind(title)
    .bind::<Option<String>>(None)
    .bind(status_to_db_string(TaskStatus::Cancelled))
    .bind(priority_to_db_string(TaskPriority::Low))
    .bind(now)
    .bind(now)
    .bind::<Option<chrono::DateTime<chrono::Utc>>>(None)
    .execute(&*pool)
    .await
    .unwrap();

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task_id), None).await;

    // Assert: Verify 200 OK with Cancelled status
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["status"], "Cancelled", "Status should be Cancelled");
}

#[tokio::test]
async fn test_get_task_with_critical_priority() {
    // Objective: Verify task with Critical priority is retrieved correctly
    // Positive test: All priority levels should be preserved
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let title = generate_unique_title("critical_get");

    // Arrange: Create task with Critical priority
    let task = create_test_task(&pool, user_id, &title, None, TaskPriority::Critical).await;

    // Act: Send GET request
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks/{}", task.id), None).await;

    // Assert: Verify 200 OK with Critical priority
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["priority"], "Critical", "Priority should be Critical");
}
