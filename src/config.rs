use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required".to_string())?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET is required".to_string())?;

        if jwt_secret.as_bytes().len() < 32 {
            return Err("JWT_SECRET must contain at least 32 bytes".into());
        }

        let jwt_ttl_minutes = env::var("JWT_TTL_MINUTES")
            .unwrap_or_else(|_| "15".into())
            .parse::<i64>()
            .map_err(|_| "JWT_TTL_MINUTES must be an integer".to_string())?;

        if !(1..=1440).contains(&jwt_ttl_minutes) {
            return Err("JWT_TTL_MINUTES must be between 1 and 1440".into());
        }

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse::<u16>()
            .map_err(|_| "PORT must be a valid u16".to_string())?;

        Ok(Self {
            database_url,
            jwt_secret,
            jwt_ttl_minutes,
            port,
        })
    }
}
