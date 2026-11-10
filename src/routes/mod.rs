use axum::{
    extract::Extension,
    http::StatusCode,
    middleware::from_fn,
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

use crate::auth::jwt::Claims;
use crate::handlers::{admin, api, auth, dishub, pimpinan, polisi};
use crate::middleware::auth::require_login_middleware;

pub async fn root_handler(Extension(claims): Extension<Option<Claims>>) -> Response {
    if let Some(user) = claims {
        let path = match user.role.as_str() {
            "admin" => "/admin/dashboard",
            "petugas_dishub" => "/dishub/pemantauan",
            "polantas" => "/polisi/operasional",
            "pimpinan" => "/pimpinan/dashboard",
            _ => "/login",
        };
        Redirect::to(path).into_response()
    } else {
        Redirect::to("/login").into_response()
    }
}

pub async fn fallback_404() -> Response {
    let html = include_str!("../../templates/404.html");
    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

pub fn create_routes() -> Router {
    // Rute terproteksi yang mewajibkan pengguna login
    let protected_routes = Router::new()
        // Admin
        .route("/admin/dashboard", get(admin::dashboard))
        .route(
            "/admin/pengguna",
            get(admin::pengguna).post(admin::create_pengguna),
        )
        .route("/admin/pengguna/:id/toggle", post(admin::toggle_pengguna))
        .route(
            "/admin/titik-macet",
            get(admin::titik_macet).post(admin::create_titik_macet),
        )
        .route(
            "/admin/titik-macet/:id/hapus",
            post(admin::delete_titik_macet),
        )
        .route("/admin/audit", get(admin::audit))
        .route("/admin/pengaturan", get(admin::pengaturan))
        // Dishub
        .route("/dishub/pemantauan", get(dishub::pemantauan))
        .route(
            "/dishub/input-kemacetan",
            get(dishub::input_kemacetan_page).post(dishub::input_kemacetan_submit),
        )
        .route(
            "/dishub/titik-macet/:id/status",
            post(dishub::update_status),
        )
        .route("/dishub/riwayat", get(dishub::riwayat))
        .route("/dishub/rekomendasi", get(dishub::rekomendasi))
        // Polisi Lalu Lintas
        .route("/polisi/operasional", get(polisi::operasional))
        .route("/polisi/notifikasi", get(polisi::notifikasi))
        .route(
            "/polisi/notifikasi/:id/baca",
            post(polisi::mark_notifikasi_read),
        )
        .route(
            "/polisi/penugasan",
            get(polisi::penugasan).post(polisi::create_penugasan),
        )
        .route(
            "/polisi/laporan-lapangan",
            get(polisi::laporan_lapangan_page).post(polisi::laporan_lapangan_submit),
        )
        // Pimpinan
        .route("/pimpinan/dashboard", get(pimpinan::dashboard))
        .route("/pimpinan/analitik", get(pimpinan::analitik))
        .route("/pimpinan/peta", get(pimpinan::peta))
        .route_layer(from_fn(require_login_middleware));

    Router::new()
        // Beranda & Root
        .route("/", get(root_handler))
        // Autentikasi Publik
        .route("/login", get(auth::login_page).post(auth::login_submit))
        .route("/logout", get(auth::logout))
        // Gabungkan Rute Terproteksi
        .merge(protected_routes)
        // REST API & Health
        .route("/health", get(api::health_check))
        .route("/api/titik-macet", get(api::api_titik_macet))
        .route("/api/dashboard/ringkasan", get(api::api_ringkasan))
        .route("/api/dashboard/tren", get(api::api_tren))
        .route("/api/dashboard/ranking", get(api::api_ranking))
        .route("/api/laporan", get(api::api_laporan))
        // Static assets
        .nest_service("/static", ServeDir::new("static"))
        .fallback(fallback_404)
}
