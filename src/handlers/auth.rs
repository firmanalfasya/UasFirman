use crate::auth::{
    hash::verify_password,
    jwt::{create_jwt, Claims},
};
use crate::config::Config;
use crate::db::DbState;
use crate::models::user::LoginPayload;
use askama::Template;
use axum::{
    extract::{Extension, Form},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate<'a> {
    pub error_message: Option<&'a str>,
    pub last_username: &'a str,
}

pub async fn login_page(Extension(claims): Extension<Option<Claims>>) -> Response {
    if let Some(user) = claims {
        let redirect_url = match user.role.as_str() {
            "admin" => "/admin/dashboard",
            "petugas_dishub" => "/dishub/pemantauan",
            "polantas" => "/polisi/operasional",
            "pimpinan" => "/pimpinan/dashboard",
            _ => "/login",
        };
        return Redirect::to(redirect_url).into_response();
    }

    let template = LoginTemplate {
        error_message: None,
        last_username: "",
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Gagal merender template login: {}", e),
        )
            .into_response(),
    }
}

pub async fn login_submit(
    Extension(db): Extension<DbState>,
    Extension(config): Extension<Config>,
    Form(payload): Form<LoginPayload>,
) -> Response {
    let username = payload.username.trim();
    let password = payload.password.trim();

    let user = match db.get_user_by_identifier(username).await {
        Some(u) => u,
        None => {
            let template = LoginTemplate {
                error_message: Some(
                    "Nama pengguna, NIP, atau kata sandi tidak valid. Silakan periksa kembali.",
                ),
                last_username: username,
            };
            return match template.render() {
                Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Render error").into_response(),
            };
        }
    };

    if !verify_password(password, &user.password_hash) {
        let template = LoginTemplate {
            error_message: Some("Kata sandi yang Anda masukkan salah. Silakan coba kembali."),
            last_username: username,
        };
        return match template.render() {
            Ok(html) => (StatusCode::UNAUTHORIZED, Html(html)).into_response(),
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Render error").into_response(),
        };
    }

    // Catat log audit login berhasil
    db.add_audit_log(
        Some(user.id),
        Some(&user.nama),
        Some(&user.peran_nama),
        "Login Berhasil",
        Some("Autentikasi"),
        Some(&format!(
            "Pengguna {} berhasil masuk ke sistem",
            user.username
        )),
        Some("127.0.0.1"),
    )
    .await;

    // Buat JWT Token
    let token = match create_jwt(
        user.id,
        &user.nama,
        &user.email,
        &user.username,
        &user.peran_nama,
        &config.jwt_secret,
        config.jwt_expiration_seconds,
    ) {
        Ok(t) => t,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, e).into_response();
        }
    };

    let cookie = format!(
        "smarttraffic_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        token, config.jwt_expiration_seconds
    );

    let redirect_url = match user.peran_nama.as_str() {
        "admin" => "/admin/dashboard",
        "petugas_dishub" => "/dishub/pemantauan",
        "polantas" => "/polisi/operasional",
        "pimpinan" => "/pimpinan/dashboard",
        _ => "/login",
    };

    let mut response = Redirect::to(redirect_url).into_response();
    if let Ok(val) = cookie.parse() {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}

pub async fn logout() -> Response {
    let cookie = "smarttraffic_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    let mut response = Redirect::to("/login").into_response();
    if let Ok(val) = cookie.parse() {
        response.headers_mut().insert(header::SET_COOKIE, val);
    }
    response
}
