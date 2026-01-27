use super::super::*;

#[tokio::test]
async fn test_create_task_returns_201_with_valid_data() {
    // Objective: Verify task creation succeeds with valid request data
    // Positive test: Create task with valid title, description, and priority
    let (app, _pool) = common::app().await;
    let title = generate_unique_title("valid_task");

    // Arrange: Create valid task request
    let body = format!(
        r#"{{"title": "{}", "description": "Test description", "priority": "High"}}"#,
        title
    );

    // Act: Send POST request to create task
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created response with task data
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["title"], title, "Title should match request");
    assert_eq!(body["description"], "Test description", "Description should match");
    assert_eq!(body["priority"], "High", "Priority should be High");
    assert_eq!(body["status"], "Pending", "Status should default to Pending");
    assert!(body.get("id").is_some(), "Response should include task ID");
    assert!(body.get("user_id").is_some(), "Response should include user_id");
    assert!(body.get("created_at").is_some(), "Response should include created_at");
}

#[tokio::test]
async fn test_create_task_returns_400_with_empty_title() {
    // Objective: Verify empty title is rejected
    // Negative test: Empty string should fail validation
    let (app, _) = common::app().await;

    // Arrange: Create request with empty title
    let body = r#"{"title": "", "description": "Test description"}"#;

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(body))).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(status, 400, "Should return 400 Bad Request for empty title");
    verify_error_response(&body_bytes, "ValidationError");
}

#[tokio::test]
async fn test_create_task_returns_400_with_title_too_long() {
    // Objective: Verify title length limit is enforced
    // Negative test: Title > 200 characters should fail
    let (app, _) = common::app().await;

    // Arrange: Create request with title > 200 characters
    let long_title = "a".repeat(201);
    let body = format!(r#"{{"title": "{}"}}"#, long_title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(
        status, 400,
        "Should return 400 Bad Request for title > 200 chars"
    );
    verify_error_response(&body_bytes, "ValidationError");
}

#[tokio::test]
async fn test_create_task_returns_400_with_whitespace_only_title() {
    // Objective: Verify whitespace-only title is rejected
    // Negative test: Title with only spaces should fail
    let (app, _) = common::app().await;

    // Arrange: Create request with whitespace-only title
    let body = r#"{"title": "   ", "description": "Test description"}"#;

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(body))).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(
        status, 400,
        "Should return 400 Bad Request for whitespace-only title"
    );
    verify_error_response(&body_bytes, "ValidationError");
}

#[tokio::test]
async fn test_create_task_returns_201_with_unicode_characters() {
    // Objective: Verify unicode characters are supported in title
    // Positive test: Unicode should be handled correctly
    let (app, _) = common::app().await;
    let title = "Test tâsk with spëcial çharacters 日本語";

    // Arrange: Create request with unicode title
    let body = format!(r#"{{"title": "{}"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created
    assert_eq!(status, 201, "Should return 201 Created for unicode title");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["title"], title, "Title should preserve unicode characters");
}

#[tokio::test]
async fn test_create_task_returns_201_with_special_characters_in_description() {
    // Objective: Verify special characters in description are supported
    // Positive test: Special chars in description should work
    let (app, _) = common::app().await;
    let title = generate_unique_title("special_chars");

    // Arrange: Create request with special characters in description
    let body = format!(
        r#"{{"title": "{}", "description": "Test with <script>alert('xss')</script> & \"quotes\" and 'apostrophes'"}}"#,
        title
    );

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created
    assert_eq!(
        status, 201,
        "Should return 201 Created with special characters"
    );
    let body: Value = parse_json_response(&body_bytes);
    assert!(body["description"]
        .as_str()
        .unwrap()
        .contains("<script>alert('xss')</script>"));
}

#[tokio::test]
async fn test_create_task_with_low_priority() {
    // Objective: verify task creation with Low priority
    // Positive test: Low priority should be accepted
    let (app, _) = common::app().await;
    let title = generate_unique_title("low_priority");

    // Arrange: Create request with Low priority
    let body = format!(r#"{{"title": "{}", "priority": "Low"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with Low priority
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["priority"], "Low", "Priority should be Low");
}

#[tokio::test]
async fn test_create_task_with_medium_priority() {
    // Objective: Verify task creation with Medium priority
    // Positive test: Medium priority should be accepted
    let (app, _) = common::app().await;
    let title = generate_unique_title("medium_priority");

    // Arrange: Create request with Medium priority
    let body = format!(r#"{{"title": "{}", "priority": "Medium"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with Medium priority
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["priority"], "Medium", "Priority should be Medium");
}

#[tokio::test]
async fn test_create_task_with_high_priority() {
    // Objective: Verify task creation with High priority
    // Positive test: High priority should be accepted
    let (app, _) = common::app().await;
    let title = generate_unique_title("high_priority");

    // Arrange: Create request with High priority
    let body = format!(r#"{{"title": "{}", "priority": "High"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with High priority
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["priority"], "High", "Priority should be High");
}

#[tokio::test]
async fn test_create_task_with_critical_priority() {
    // Objective: Verify task creation with Critical priority
    // Positive test: Critical priority should be accepted
    let (app, _) = common::app().await;
    let title = generate_unique_title("critical_priority");

    // Arrange: Create request with Critical priority
    let body = format!(r#"{{"title": "{}", "priority": "Critical"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with Critical priority
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(body["priority"], "Critical", "Priority should be Critical");
}

#[tokio::test]
async fn test_create_task_with_default_priority() {
    // Objective: Verify default priority is Medium when not specified
    // Positive test: Missing priority should default to Medium
    let (app, _) = common::app().await;
    let title = generate_unique_title("default_priority");

    // Arrange: Create request without priority field
    let body = format!(r#"{{"title": "{}"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with Medium as default
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(
        body["priority"], "Medium",
        "Priority should default to Medium"
    );
}

#[tokio::test]
async fn test_create_task_with_missing_description() {
    // Objective: Verify task creation works without description
    // Positive test: Optional description field should work
    let (app, _) = common::app().await;
    let title = generate_unique_title("no_description");

    // Arrange: Create request without description
    let body = format!(r#"{{"title": "{}"}}"#, title);

    // Act: Send POST request
    let (status, body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(&body))).await;

    // Assert: Verify 201 Created with None/null description
    assert_eq!(status, 201, "Should return 201 Created");
    let body: Value = parse_json_response(&body_bytes);
    assert_eq!(
        body["description"], serde_json::Value::Null,
        "Description should be null"
    );
}

#[tokio::test]
async fn test_create_task_returns_422_with_missing_title_field() {
    // Objective: Verify missing required field is rejected
    // Negative test: Missing title should return 422 (JSON deserialization error)
    let (app, _) = common::app().await;

    // Arrange: Create request without title field
    let body = r#"{"description": "Test description"}"#;

    // Act: Send POST request
    let (status, _) = make_request(&app, "POST", "/tasks", Some(create_json_body(body))).await;

    // Assert: Verify 422 Unprocessable Entity
    assert_eq!(
        status, 422,
        "Should return 422 Unprocessable Entity for missing title field"
    );
}

#[tokio::test]
async fn test_create_task_returns_400_with_malformed_json() {
    // Objective: Verify malformed JSON is rejected
    // Negative test: Invalid JSON should return 400
    let (app, _) = common::app().await;

    // Arrange: Create malformed JSON (missing closing brace)
    let body = r#"{"title": "test", "description": "desc""#;

    // Act: Send POST request
    let (status, _) = make_request(&app, "POST", "/tasks", Some(create_json_body(body))).await;

    // Assert: Verify 400 Bad Request
    assert_eq!(status, 400, "Should return 400 Bad Request for malformed JSON");
}

#[tokio::test]
async fn test_create_task_returns_422_with_invalid_priority_type() {
    // Objective: Verify invalid priority value is rejected
    // Negative test: Invalid priority enum value should fail
    let (app, _) = common::app().await;

    // Arrange: Create request with invalid priority value
    let body = r#"{"title": "Test", "priority": "InvalidPriority"}"#;

    // Act: Send POST request
    let (status, _body_bytes) =
        make_request(&app, "POST", "/tasks", Some(create_json_body(body))).await;

    // Assert: Verify 422 Unprocessable Entity (JSON deserialization error)
    assert_eq!(
        status, 422,
        "Should return 422 Unprocessable Entity for invalid priority type"
    );
}
