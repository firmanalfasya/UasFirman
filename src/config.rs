use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_seconds: i64,
    #[allow(dead_code)]
    pub standalone_demo: bool,
}

impl Config {
    pub fn from_env() -> Self {
        // Muat berkas .env jika ada
        dotenvy::dotenv().ok();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(3000);

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/smarttraffic_bdl".to_string()
        });

        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| {
            "kunci-rahasia-default-smarttraffic-bandar-lampung-2026-teknokrat".to_string()
        });

        let jwt_expiration_seconds = env::var("JWT_EXPIRATION_SECONDS")
            .ok()
            .and_then(|e| e.parse::<i64>().ok())
            .unwrap_or(86400);

        let standalone_demo = env::var("STANDALONE_DEMO")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(true);

        Config {
            port,
            database_url,
            jwt_secret,
            jwt_expiration_seconds,
            standalone_demo,
        }
    }
}
