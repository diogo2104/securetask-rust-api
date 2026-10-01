use axum::{Json, extract::State};
use serde::Serialize;

use crate::{error::AppError, state::AppState};

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
}

pub async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, AppError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await?;

    Ok(Json(HealthResponse { status: "ok" }))
}
