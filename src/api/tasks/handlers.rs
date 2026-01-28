use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use std::sync::Arc;

use crate::{
    api::{
        error::{ApiErrorResponse, ErrorCode},
        models::tasks::{CreateTaskRequest, ListTasksQuery, TaskResponse},
    },
    common::UserId,
    config::AppState,
    domain::task::{
        models::Task,
        operations::{create_task, get_task, list_tasks_by_user},
    },
};

#[utoipa::path(
    get,
    path = "/tasks/{id}",
    tag = "tasks",
    params(
        ("id" = String, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = TaskResponse),
        (status = 404, description = "Task not found", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
pub async fn get_task_handler(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<TaskResponse>, ApiErrorResponse> {
    let task_id =
        uuid::Uuid::parse_str(&id).map_err(|_| ApiErrorResponse::from(ErrorCode::BadRequest))?;

    let task = get_task(task_id.into(), state.task_repository.clone())
        .await
        .map_err(ApiErrorResponse::from)?;

    Ok(Json(task.into()))
}

#[utoipa::path(
    get,
    path = "/tasks",
    tag = "tasks",
    params(ListTasksQuery),
    responses(
        (status = 200, description = "List of tasks", body = Vec<TaskResponse>),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
pub async fn list_tasks_handler(
    Query(query): Query<ListTasksQuery>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<TaskResponse>>, ApiErrorResponse> {
    let user_id = query
        .user_id
        .ok_or_else(|| ApiErrorResponse::from(ErrorCode::BadRequest))?;

    let user_id = uuid::Uuid::parse_str(&user_id)
        .map_err(|_| ApiErrorResponse::from(ErrorCode::BadRequest))?;

    let tasks = list_tasks_by_user(user_id.into(), state.task_repository.clone())
        .await
        .map_err(ApiErrorResponse::from)?;

    Ok(Json(tasks.into_iter().map(|t| t.into()).collect()))
}

#[utoipa::path(
    post,
    path = "/tasks",
    tag = "tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created", body = TaskResponse),
        (status = 400, description = "Invalid request", body = ApiErrorResponse),
        (status = 500, description = "Internal server error", body = ApiErrorResponse)
    )
)]
pub async fn create_task_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), ApiErrorResponse> {
    let user_id = UserId::new();

    let task = Task::new(
        user_id,
        request.title,
        request.description,
        request.priority.unwrap_or_default(),
    )
    .map_err(ApiErrorResponse::from)?;

    let created = create_task(task, state.task_repository.clone())
        .await
        .map_err(ApiErrorResponse::from)?;

    Ok((StatusCode::CREATED, Json(created.into())))
}
