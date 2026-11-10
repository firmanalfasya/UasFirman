use crate::auth::jwt::{verify_jwt, Claims};
use crate::config::Config;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{Html, IntoResponse, Redirect, Response},
};

pub fn extract_token_from_cookies(req: &Request) -> Option<String> {
    let cookie_header = req.headers().get(header::COOKIE)?.to_str().ok()?;
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(token) = pair.strip_prefix("smarttraffic_session=") {
            return Some(token.to_string());
        }
    }
    None
}

pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    let config = req
        .extensions()
        .get::<Config>()
        .cloned()
        .unwrap_or_else(Config::from_env);

    let mut claims_opt = None;
    if let Some(token) = extract_token_from_cookies(&req) {
        if let Ok(claims) = verify_jwt(&token, &config.jwt_secret) {
            req.extensions_mut().insert(claims.clone());
            claims_opt = Some(claims);
        }
    }
    req.extensions_mut().insert(claims_opt);

    next.run(req).await
}

pub async fn require_login_middleware(req: Request, next: Next) -> Response {
    let claims = req.extensions().get::<Claims>().cloned();
    if claims.is_none() {
        return Redirect::to("/login").into_response();
    }
    next.run(req).await
}

/// Helper untuk render halaman 403 Forbidden
pub fn render_forbidden() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1.0"/>
    <title>403 - Akses Ditolak | SmartTraffic Bandar Lampung</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap" rel="stylesheet"/>
</head>
<body class="bg-slate-50 font-sans text-slate-800 min-h-screen flex items-center justify-center p-4">
    <div class="max-w-md w-full bg-white rounded-xl shadow-lg border border-slate-200 p-8 text-center">
        <div class="w-16 h-16 rounded-full bg-red-100 text-red-600 flex items-center justify-center mx-auto mb-4 font-bold text-2xl">
            403
        </div>
        <h1 class="text-xl font-bold text-slate-900 mb-2">403 - Akses Ditolak</h1>
        <p class="text-xs text-slate-600 mb-6 leading-relaxed">
            Anda tidak memiliki izin otorisasi untuk mengakses halaman ini. Halaman ini dibatasi hanya untuk peran tertentu sesuai kebijakan RBAC SmartTraffic Kota Bandar Lampung.
        </p>
        <div class="flex flex-col gap-2">
            <a href="/" class="w-full bg-[#1E3A8A] hover:bg-blue-900 text-white font-medium py-2.5 px-4 rounded-lg text-xs transition">
                Kembali ke Beranda
            </a>
            <a href="/logout" class="w-full bg-slate-100 hover:bg-slate-200 text-slate-700 font-medium py-2 px-4 rounded-lg text-xs transition">
                Ganti Akun (Keluar)
            </a>
        </div>
        <footer class="mt-8 pt-4 border-t border-slate-100 text-[11px] text-slate-400">
            Copyright©2026 by Firman Alfasya Arief dan Dr. Erliyan Redy Susanto, M.Kom
        </footer>
    </div>
</body>
</html>"#;
    (StatusCode::FORBIDDEN, Html(html)).into_response()
}

/// Helper untuk memeriksa RBAC role
pub fn check_role_permission(claims: &Claims, allowed_roles: &[&str]) -> bool {
    allowed_roles.contains(&claims.role.as_str())
}
