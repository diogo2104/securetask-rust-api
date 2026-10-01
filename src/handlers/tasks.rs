use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    dto::{CreateTaskRequest, UpdateTaskRequest},
    error::AppError,
    models::Task,
    state::AppState,
};

pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let title = validate_title(&payload.title)?;
    let description = normalize_description(payload.description)?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        INSERT INTO tasks (id, user_id, title, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(auth.id)
    .bind(title)
    .bind(description)
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn list_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Task>>, AppError> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, title, description, completed, created_at, updated_at
        FROM tasks
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .bind(auth.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(tasks))
}

pub async fn get_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(task_id): Path<Uuid>,
) -> Result<Json<Task>, AppError> {
    let task = sqlx::query_as::<_, Task>(
        r#"
        SELECT id, title, description, completed, created_at, updated_at
        FROM tasks
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(task_id)
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(task))
}

pub async fn update_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<Task>, AppError> {
    if payload.title.is_none() && payload.description.is_none() && payload.completed.is_none() {
        return Err(AppError::BadRequest("no fields to update".into()));
    }

    let title = payload.title.as_deref().map(validate_title).transpose()?;
    let description = payload
        .description
        .map(normalize_description)
        .transpose()?;

    let task = sqlx::query_as::<_, Task>(
        r#"
        UPDATE tasks
        SET title = COALESCE($3, title),
            description = CASE WHEN $4 THEN $5 ELSE description END,
            completed = COALESCE($6, completed),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, title, description, completed, created_at, updated_at
        "#,
    )
    .bind(task_id)
    .bind(auth.id)
    .bind(title)
    .bind(description.is_some())
    .bind(description.flatten())
    .bind(payload.completed)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(task_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(task_id)
        .bind(auth.id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_title(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 120 {
        return Err(AppError::BadRequest(
            "title must contain 1 to 120 characters".into(),
        ));
    }
    Ok(value.to_string())
}

fn normalize_description(value: Option<String>) -> Result<Option<String>, AppError> {
    match value {
        Some(value) => {
            let value = value.trim();
            if value.chars().count() > 2000 {
                return Err(AppError::BadRequest(
                    "description must contain at most 2000 characters".into(),
                ));
            }
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value.to_string()))
            }
        }
        None => Ok(None),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_title() {
        assert_eq!(validate_title("  learn rust  ").unwrap(), "learn rust");
    }

    #[test]
    fn rejects_empty_title() {
        assert!(validate_title("   ").is_err());
    }

    #[test]
    fn empty_description_becomes_none() {
        assert_eq!(normalize_description(Some("   ".into())).unwrap(), None);
    }
}
