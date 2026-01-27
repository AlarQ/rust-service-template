use super::super::*;
use rust_service_template::domain::task::models::{TaskPriority, TaskStatus};

#[tokio::test]
async fn test_list_tasks_returns_200_with_tasks() {
    // Objective: Verify listing tasks by user_id returns all user's tasks
    // Positive test: GET request with valid user_id should return tasks
    let (app, pool) = common::app().await;
    let user_id = UserId::new();

    // Arrange: Create multiple tasks for the same user
    let task1 = create_test_task(
        &pool,
        user_id,
        "Task 1",
        Some("Description 1".to_string()),
        TaskPriority::High,
    )
    .await;
    let task2 = create_test_task(
        &pool,
        user_id,
        "Task 2",
        Some("Description 2".to_string()),
        TaskPriority::Low,
    )
    .await;
    let task3 = create_test_task(&pool, user_id, "Task 3", None, TaskPriority::Medium).await;

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with array of tasks
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert!(body.is_array(), "Response should be an array");
    assert_eq!(body.as_array().unwrap().len(), 3, "Should return 3 tasks");

    // Verify tasks are in descending order by created_at (most recent first)
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks[0]["title"], task3.title.0);
    assert_eq!(tasks[1]["title"], task2.title.0);
    assert_eq!(tasks[2]["title"], task1.title.0);
}

#[tokio::test]
async fn test_list_tasks_returns_200_empty_for_new_user() {
    // Objective: Verify listing tasks for user with no tasks returns empty array
    // Positive test: Empty result should return 200 with empty array
    let (app, _) = common::app().await;
    let user_id = UserId::new();

    // Arrange: Use a user_id with no tasks
    // (No setup needed - user has no tasks)

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with empty array
    assert_eq!(status, 200, "Should return 200 OK for empty list");
    let body: Value = parse_json_response(&body_bytes);
    assert!(body.is_array(), "Response should be an array");
    assert_eq!(body.as_array().unwrap().len(), 0, "Array should be empty");
}

#[tokio::test]
async fn test_list_tasks_returns_400_missing_user_id() {
    // Objective: Verify missing user_id query parameter is rejected
    // Negative test: Required query parameter missing should return 400
    let (app, _) = common::app().await;

    // Arrange: Send request without user_id query param
    // (No setup needed)

    // Act: Send GET request without user_id
    let (status, body_bytes) = make_request(&app, "GET", "/tasks", None).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(
        status, 400,
        "Should return 400 Bad Request for missing user_id"
    );
    verify_error_response(&body_bytes, "BadRequest");
}

#[tokio::test]
async fn test_list_tasks_returns_400_invalid_user_id_format() {
    // Objective: Verify invalid UUID format for user_id is rejected
    // Negative test: Malformed UUID should return 400
    let (app, _) = common::app().await;

    // Arrange: Use invalid user_id format
    let invalid_user_id = "not-a-valid-uuid";

    // Act: Send GET request with invalid user_id
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", invalid_user_id), None).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(
        status, 400,
        "Should return 400 Bad Request for invalid user_id format"
    );
    verify_error_response(&body_bytes, "BadRequest");
}

#[tokio::test]
async fn test_list_tasks_with_different_statuses() {
    // Objective: Verify tasks with different statuses are all returned
    // Positive test: All status types should be included in results
    let (app, pool) = common::app().await;
    let user_id = UserId::new();
    let now = chrono::Utc::now();

    // Arrange: Create tasks with different statuses directly in DB
    for (i, status) in [
        TaskStatus::Pending,
        TaskStatus::InProgress,
        TaskStatus::Completed,
        TaskStatus::Cancelled,
    ]
    .iter()
    .enumerate()
    {
        let task_id = uuid::Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO tasks (id, user_id, title, description, status, priority, created_at, updated_at, completed_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(task_id)
        .bind(user_id.0)
        .bind(format!("Task {}", i))
        .bind::<Option<String>>(None)
        .bind(*status)
        .bind(TaskPriority::Medium)
        .bind(now)
        .bind(now)
        .bind(if *status == TaskStatus::Completed {
            Some(now)
        } else {
            None
        })
        .execute(&*pool)
        .await
        .unwrap();
    }

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with all tasks
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks.len(), 4, "Should return all 4 tasks");

    // Verify all statuses are present
    let statuses: Vec<&str> = tasks
        .iter()
        .map(|t| t["status"].as_str().unwrap())
        .collect();
    assert!(statuses.contains(&"Pending"));
    assert!(statuses.contains(&"InProgress"));
    assert!(statuses.contains(&"Completed"));
    assert!(statuses.contains(&"Cancelled"));
}

#[tokio::test]
async fn test_list_tasks_with_different_priorities() {
    // Objective: Verify tasks with different priorities are all returned
    // Positive test: All priority types should be included in results
    let (app, pool) = common::app().await;
    let user_id = UserId::new();

    // Arrange: Create tasks with different priorities
    let _task_low = create_test_task(&pool, user_id, "Low Task", None, TaskPriority::Low).await;
    let _task_medium =
        create_test_task(&pool, user_id, "Medium Task", None, TaskPriority::Medium).await;
    let _task_high =
        create_test_task(&pool, user_id, "High Task", None, TaskPriority::High).await;
    let _task_critical = create_test_task(
        &pool,
        user_id,
        "Critical Task",
        None,
        TaskPriority::Critical,
    )
    .await;

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with all tasks
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks.len(), 4, "Should return all 4 tasks");

    // Verify all priorities are present
    let priorities: Vec<&str> = tasks
        .iter()
        .map(|t| t["priority"].as_str().unwrap())
        .collect();
    assert!(priorities.contains(&"Low"));
    assert!(priorities.contains(&"Medium"));
    assert!(priorities.contains(&"High"));
    assert!(priorities.contains(&"Critical"));
}

#[tokio::test]
async fn test_list_tasks_only_returns_user_tasks() {
    // Objective: Verify tasks are filtered by user_id correctly
    // Positive test: Only tasks for specified user should be returned
    let (app, pool) = common::app().await;
    let user_id_1 = UserId::new();
    let user_id_2 = UserId::new();

    // Arrange: Create tasks for two different users
    let _task1_user1 =
        create_test_task(&pool, user_id_1, "User 1 Task 1", None, TaskPriority::High).await;
    let _task2_user1 =
        create_test_task(&pool, user_id_1, "User 1 Task 2", None, TaskPriority::Low).await;
    let _task1_user2 =
        create_test_task(&pool, user_id_2, "User 2 Task 1", None, TaskPriority::Medium).await;

    // Act: List tasks for user_id_1 only
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id_1), None).await;

    // Assert: Verify 200 OK with only user 1's tasks
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks.len(), 2, "Should return only 2 tasks for user 1");

    // Verify the tasks belong to user_id_1
    let task_ids: Vec<String> = tasks
        .iter()
        .map(|t| t["user_id"].as_str().unwrap().to_string())
        .collect();
    assert!(task_ids.iter().all(|id| id == &user_id_1.to_string()));
    assert!(!task_ids.contains(&user_id_2.to_string()));
}

#[tokio::test]
async fn test_list_tasks_with_and_without_descriptions() {
    // Objective: Verify tasks with and without descriptions are handled correctly
    // Positive test: Mixed description states should work
    let (app, pool) = common::app().await;
    let user_id = UserId::new();

    // Arrange: Create tasks with and without descriptions
    let _task_with_desc = create_test_task(
        &pool,
        user_id,
        "Task with description",
        Some("This has a description".to_string()),
        TaskPriority::High,
    )
    .await;
    let _task_without_desc =
        create_test_task(&pool, user_id, "Task without description", None, TaskPriority::Medium)
            .await;

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with both tasks
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    let tasks = body.as_array().unwrap();
    assert_eq!(tasks.len(), 2, "Should return 2 tasks");

    // Verify one has description and one is null
    let descriptions: Vec<&serde_json::Value> =
        tasks.iter().map(|t| &t["description"]).collect();
    let has_null = descriptions.iter().any(|d| d.is_null());
    let has_value = descriptions.iter().any(|d| d.is_string() && d.as_str().is_some());
    assert!(has_null && has_value, "Should have one null and one string description");
}

#[tokio::test]
async fn test_list_tasks_with_single_task() {
    // Objective: Verify single task is returned correctly
    // Positive test: Array with single element should work
    let (app, pool) = common::app().await;
    let user_id = UserId::new();

    // Arrange: Create single task
    let _task = create_test_task(
        &pool,
        user_id,
        "Single Task",
        Some("Only task".to_string()),
        TaskPriority::Critical,
    )
    .await;

    // Act: Send GET request to list tasks
    let (status, body_bytes) =
        make_request(&app, "GET", &format!("/tasks?user_id={}", user_id), None).await;

    // Assert: Verify 200 OK with single task
    assert_eq!(status, 200, "Should return 200 OK");
    let body: Value = parse_json_response(&body_bytes);
    assert!(body.is_array(), "Response should be an array");
    assert_eq!(body.as_array().unwrap().len(), 1, "Should return 1 task");
    assert_eq!(
        body[0]["title"], "Single Task",
        "Task title should match"
    );
}
