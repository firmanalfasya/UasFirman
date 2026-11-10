use axum::{middleware::from_fn, Extension};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod routes;

use config::Config;
use db::DbState;
use middleware::{auth::auth_middleware, security::security_headers_middleware};
use routes::create_routes;

#[tokio::main]
async fn main() {
    // Inisialisasi konfigurasi dari .env
    let config = Config::from_env();

    // Inisialisasi structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "smarttraffic=info,tower_http=info,axum=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("============================================================");
    tracing::info!("SmartTraffic - Kota Bandar Lampung (v2.6)");
    tracing::info!("Dinas Perhubungan & Satlantas Polresta Bandar Lampung");
    tracing::info!("Dikembangkan bersama Universitas Teknokrat Indonesia (UTI)");
    tracing::info!("============================================================");

    // Inisialisasi basis data state
    let db = DbState::init(&config.database_url).await;

    // Susun router dan pasang middleware global
    let app = create_routes()
        .layer(from_fn(auth_middleware))
        .layer(from_fn(security_headers_middleware))
        .layer(Extension(config.clone()))
        .layer(Extension(db));

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!(
        "Server SmartTraffic aktif dan mendengarkan di http://{}",
        addr
    );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("Gagal mengikat port {}: {}", config.port, e));

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{
        hash::{hash_password, verify_password},
        jwt::{create_jwt, verify_jwt},
    };
    use crate::models::validation::validate_coordinates;

    #[test]
    fn test_argon2_hashing_and_verification() {
        let password = "admin123BandarLampung!";
        let hash = hash_password(password).expect("Hashing gagal");
        assert!(verify_password(password, &hash));
        assert!(!verify_password("passwordSalah", &hash));
    }

    #[test]
    fn test_jwt_lifecycle_and_claims() {
        let secret = "kunci-rahasia-uji-coba-smarttraffic-bandar-lampung-2026-minimal32karakter";
        let token = create_jwt(
            1,
            "Ir. Hendra",
            "admin@smarttraffic.bandarlampung.go.id",
            "admin",
            "admin",
            secret,
            3600,
        )
        .expect("JWT creation failed");

        let claims = verify_jwt(&token, secret).expect("JWT verification failed");
        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.username, "admin");
    }

    #[test]
    fn test_bandar_lampung_coordinates_validation() {
        // Tugu Adipura Bandar Lampung
        assert!(validate_coordinates(-5.4215, 105.2605).is_ok());
        // Simpang MBK Mall Boemi Kedaton
        assert!(validate_coordinates(-5.3789, 105.2536).is_ok());
        // Di luar Lampung (contoh Monas Jakarta)
        assert!(validate_coordinates(-6.1754, 106.8272).is_err());
    }

    #[tokio::test]
    async fn test_db_store_initialization_and_seed() {
        let db = DbState::init("").await;
        let users = db.list_users().await;
        assert_eq!(users.len(), 4);

        let titik = db.list_titik_macet(None, None, None, None).await;
        assert_eq!(titik.len(), 15);

        let ringkasan = db.get_dashboard_ringkasan().await;
        assert_eq!(ringkasan.total_titik, 15);
        assert!(ringkasan.macet_parah > 0);
    }
}
