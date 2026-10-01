use std::sync::Arc;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub jwt_secret: Arc<str>,
    pub jwt_ttl_minutes: i64,
}

impl AppState {
    pub fn new(pool: PgPool, jwt_secret: String, jwt_ttl_minutes: i64) -> Self {
        Self {
            pool,
            jwt_secret: Arc::from(jwt_secret),
            jwt_ttl_minutes,
        }
    }
}
