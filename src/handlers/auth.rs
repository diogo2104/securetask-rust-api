use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;

use crate::{
    auth::{AuthUser, create_token, hash_password, verify_password},
    dto::{AuthResponse, LoginRequest, RegisterRequest},
    error::AppError,
    models::{UserCredentials, UserPublic},
    state::AppState,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<UserPublic>), AppError> {
    let name = normalize_name(&payload.name)?;
    let email = normalize_email(&payload.email)?;
    validate_password(&payload.password)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)",
    )
    .bind(&email)
    .fetch_one(&state.pool)
    .await?;

    if exists {
        return Err(AppError::Conflict("email is already registered".into()));
    }

    let password_hash = hash_password(payload.password).await?;

    let user = sqlx::query_as::<_, UserPublic>(
        r#"
        INSERT INTO users (id, name, email, password_hash)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, email, created_at
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(name)
    .bind(email)
    .bind(password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|error| {
        if let sqlx::Error::Database(db) = &error {
            if db.is_unique_violation() {
                return AppError::Conflict("email is already registered".into());
            }
        }
        AppError::from(error)
    })?;

    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let email = normalize_email(&payload.email)?;

    let user = sqlx::query_as::<_, UserCredentials>(
        "SELECT id, password_hash FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !verify_password(payload.password, user.password_hash).await? {
        return Err(AppError::Unauthorized);
    }

    let token = create_token(user.id, &state)?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in: state.jwt_ttl_minutes * 60,
    }))
}

pub async fn me(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<UserPublic>, AppError> {
    let user = sqlx::query_as::<_, UserPublic>(
        "SELECT id, name, email, created_at FROM users WHERE id = $1",
    )
    .bind(auth.id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    Ok(Json(user))
}

fn normalize_name(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if !(2..=80).contains(&value.chars().count()) {
        return Err(AppError::BadRequest("name must contain 2 to 80 characters".into()));
    }
    Ok(value.to_string())
}

fn normalize_email(value: &str) -> Result<String, AppError> {
    let value = value.trim().to_lowercase();
    if value.len() > 254 || !value.contains('@') || value.starts_with('@') || value.ends_with('@') {
        return Err(AppError::BadRequest("invalid email".into()));
    }
    Ok(value)
}

fn validate_password(value: &str) -> Result<(), AppError> {
    if value.len() < 12 || value.len() > 128 {
        return Err(AppError::BadRequest(
            "password must contain 12 to 128 characters".into(),
        ));
    }

    let has_upper = value.chars().any(char::is_uppercase);
    let has_lower = value.chars().any(char::is_lowercase);
    let has_digit = value.chars().any(|c| c.is_ascii_digit());

    if !(has_upper && has_lower && has_digit) {
        return Err(AppError::BadRequest(
            "password must include uppercase, lowercase and a number".into(),
        ));
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_email() {
        assert_eq!(normalize_email(" User@Example.COM ").unwrap(), "user@example.com");
    }

    #[test]
    fn rejects_weak_password() {
        assert!(validate_password("short").is_err());
        assert!(validate_password("alllowercase123").is_err());
    }

    #[test]
    fn accepts_reasonable_password() {
        assert!(validate_password("RustSeguro123").is_ok());
    }
}
