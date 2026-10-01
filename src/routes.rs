use axum::{Router, extract::DefaultBodyLimit, routing::{get, post}};

use crate::{
    handlers::{auth, health, tasks},
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/users/me", get(auth::me))
        .route("/api/v1/tasks", post(tasks::create_task).get(tasks::list_tasks))
        .route(
            "/api/v1/tasks/{id}",
            get(tasks::get_task)
                .patch(tasks::update_task)
                .delete(tasks::delete_task),
        )
        .layer(DefaultBodyLimit::max(64 * 1024))
        .with_state(state)
}
