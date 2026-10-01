use std::{
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

static CRYPTO_PROVIDER: OnceLock<()> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: usize,
    iat: usize,
    iss: String,
}

pub async fn hash_password(password: String) -> Result<String, AppError> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| {
                tracing::error!(error = %error, "password hashing failed");
                AppError::Internal
            })
    })
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "password hashing task failed");
        AppError::Internal
    })?
}

pub async fn verify_password(password: String, password_hash: String) -> Result<bool, AppError> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&password_hash).map_err(|error| {
            tracing::error!(error = %error, "stored password hash is invalid");
            AppError::Internal
        })?;

        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(|error| {
        tracing::error!(error = %error, "password verification task failed");
        AppError::Internal
    })?
}

pub fn create_token(user_id: Uuid, state: &AppState) -> Result<String, AppError> {
    ensure_crypto_provider();

    let now = unix_timestamp()?;
    let exp = now + (state.jwt_ttl_minutes * 60) as usize;

    let claims = Claims {
        sub: user_id,
        iat: now,
        exp,
        iss: "securetask-api".into(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )
    .map_err(|error| {
        tracing::error!(error = %error, "JWT encoding failed");
        AppError::Internal
    })
}

fn decode_token(token: &str, state: &AppState) -> Result<Uuid, AppError> {
    ensure_crypto_provider();

    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&["securetask-api"]);
    validation.leeway = 10;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims.sub)
    .map_err(|_| AppError::Unauthorized)
}

fn ensure_crypto_provider() {
    CRYPTO_PROVIDER.get_or_init(|| {
        let _ = jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER.install_default();
    });
}

fn unix_timestamp() -> Result<usize, AppError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as usize)
        .map_err(|_| AppError::Internal)
}

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;

        if token.is_empty() {
            return Err(AppError::Unauthorized);
        }

        Ok(Self {
            id: decode_token(token, &state)?,
        })
    }
}
