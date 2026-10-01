use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use securetask_api::{routes, state::AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

fn test_state(pool: PgPool) -> AppState {
    AppState::new(
        pool,
        "test-secret-with-more-than-thirty-two-bytes".to_string(),
        15,
    )
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response body should be readable");

    serde_json::from_slice(&bytes).expect("response body should contain valid JSON")
}

async fn post_json(
    app: &axum::Router,
    uri: &str,
    body: Value,
    bearer: Option<&str>,
) -> axum::response::Response {
    let mut builder = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = bearer {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    app.clone()
        .oneshot(builder.body(Body::from(body.to_string())).unwrap())
        .await
        .unwrap()
}

async fn login(app: &axum::Router, email: &str) -> String {
    let response = post_json(
        app,
        "/api/v1/auth/login",
        json!({
            "email": email,
            "password": "RustSeguro123"
        }),
        None,
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await["access_token"]
        .as_str()
        .expect("login should return access_token")
        .to_string()
}

#[sqlx::test(migrations = "./migrations")]
async fn complete_authenticated_task_flow(pool: PgPool) {
    let app = routes::router(test_state(pool));

    let register = post_json(
        &app,
        "/api/v1/auth/register",
        json!({
            "name": "Diogo",
            "email": "diogo@example.com",
            "password": "RustSeguro123"
        }),
        None,
    )
    .await;

    assert_eq!(register.status(), StatusCode::CREATED);

    let token = login(&app, "diogo@example.com").await;

    let create_task = post_json(
        &app,
        "/api/v1/tasks",
        json!({
            "title": "Review ownership",
            "description": "Prepare notes for the Rust interview"
        }),
        Some(&token),
    )
    .await;

    assert_eq!(create_task.status(), StatusCode::CREATED);
    let task = json_body(create_task).await;
    let task_id = task["id"].as_str().unwrap();

    let get_task = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/tasks/{task_id}"))
                .header(header::AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_task.status(), StatusCode::OK);

    let second_register = post_json(
        &app,
        "/api/v1/auth/register",
        json!({
            "name": "Other User",
            "email": "other@example.com",
            "password": "RustSeguro123"
        }),
        None,
    )
    .await;

    assert_eq!(second_register.status(), StatusCode::CREATED);
    let second_token = login(&app, "other@example.com").await;

    let forbidden_read = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/tasks/{task_id}"))
                .header(
                    header::AUTHORIZATION,
                    format!("Bearer {second_token}"),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(forbidden_read.status(), StatusCode::NOT_FOUND);
}
