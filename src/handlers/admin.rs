use askama::Template;
use axum::{
    extract::{Extension, Form, Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;

use crate::auth::jwt::Claims;
use crate::db::DbState;
use crate::middleware::auth::{check_role_permission, render_forbidden};
use crate::models::{
    audit::LogAudit,
    traffic::*,
    user::{CreateUserPayload, User},
    validation::validate_coordinates,
};

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
pub struct AdminDashboardTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub total_pengguna: usize,
    pub ringkasan: &'a DashboardRingkasan,
    pub titik_list: &'a [TitikMacet],
    pub audit_list: &'a [LogAudit],
}

#[derive(Template)]
#[template(path = "admin/pengguna.html")]
pub struct AdminPenggunaTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub users: &'a [User],
    pub message: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "admin/titik_macet.html")]
pub struct AdminTitikMacetTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub titik_list: &'a [TitikMacet],
    pub kecamatan_list: &'a [Kecamatan],
    pub ruas_list: &'a [RuasJalan],
    pub penyebab_list: &'a [PenyebabMacet],
    pub query_q: &'a str,
    pub query_kecamatan: &'a str,
    pub query_kepadatan: &'a str,
}

#[derive(Template)]
#[template(path = "admin/audit.html")]
pub struct AdminAuditTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub audit_list: &'a [LogAudit],
}

#[derive(Template)]
#[template(path = "admin/pengaturan.html")]
pub struct AdminPengaturanTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
}

#[derive(Deserialize)]
pub struct TitikFilterQuery {
    pub q: Option<String>,
    pub kecamatan: Option<String>,
    pub kepadatan: Option<String>,
}

pub async fn dashboard(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    let ringkasan = db.get_dashboard_ringkasan().await;
    let titik_list = db.list_titik_macet(None, None, None, None).await;
    let audit_list = db.list_audit_logs().await;
    let users = db.list_users().await;

    let template = AdminDashboardTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "admin_dashboard",
        total_pengguna: users.len(),
        ringkasan: &ringkasan,
        titik_list: &titik_list,
        audit_list: &audit_list,
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn pengguna(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    let users = db.list_users().await;
    let template = AdminPenggunaTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "admin_pengguna",
        users: &users,
        message: None,
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn create_pengguna(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Form(payload): Form<CreateUserPayload>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    match db
        .create_user(
            &payload.nama,
            &payload.email,
            &payload.username,
            &payload.password,
            payload.peran_id,
        )
        .await
    {
        Ok(new_id) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Tambah Pengguna",
                Some("Pengguna"),
                Some(&format!(
                    "Menambahkan pengguna baru ID: {} ({})",
                    new_id, payload.username
                )),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/admin/pengguna").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal membuat pengguna: {}", e),
        )
            .into_response(),
    }
}

pub async fn toggle_pengguna(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    if let Some(user) = db.get_user_by_id(id).await {
        let new_status = !user.aktif;
        let _ = db
            .update_user_status(id, &user.nama, &user.email, user.peran_id, new_status)
            .await;

        db.add_audit_log(
            Some(claims.user_id),
            Some(&claims.nama),
            Some(&claims.role),
            "Ubah Status Pengguna",
            Some("Pengguna"),
            Some(&format!(
                "Mengubah status akun pengguna ID {} ({}) menjadi {}",
                id,
                user.username,
                if new_status { "Aktif" } else { "Nonaktif" }
            )),
            Some("127.0.0.1"),
        )
        .await;
    }

    Redirect::to("/admin/pengguna").into_response()
}

pub async fn titik_macet(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Query(filter): Query<TitikFilterQuery>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    let q_ref = filter.q.as_deref().unwrap_or("");
    let kec_ref = filter.kecamatan.as_deref().unwrap_or("Semua");
    let kep_ref = filter.kepadatan.as_deref().unwrap_or("Semua");

    let titik_list = db
        .list_titik_macet(
            if kec_ref == "Semua" {
                None
            } else {
                Some(kec_ref)
            },
            None,
            if kep_ref == "Semua" {
                None
            } else {
                Some(kep_ref)
            },
            if q_ref.is_empty() { None } else { Some(q_ref) },
        )
        .await;

    let kecamatan_list = db.list_kecamatan().await;
    let ruas_list = db.list_ruas().await;
    let penyebab_list = db.list_penyebab().await;

    let template = AdminTitikMacetTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "admin_titik_macet",
        titik_list: &titik_list,
        kecamatan_list: &kecamatan_list,
        ruas_list: &ruas_list,
        penyebab_list: &penyebab_list,
        query_q: q_ref,
        query_kecamatan: kec_ref,
        query_kepadatan: kep_ref,
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn create_titik_macet(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Form(input): Form<TitikMacetInput>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "petugas_dishub", "polantas"]) {
        return render_forbidden();
    }

    if let Err(e) = validate_coordinates(input.latitude, input.longitude) {
        return (StatusCode::BAD_REQUEST, e).into_response();
    }

    match db
        .create_titik_macet(input, claims.user_id, &claims.nama)
        .await
    {
        Ok(id) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Tambah Titik Macet",
                Some("Titik Macet"),
                Some(&format!("Menambahkan titik kemacetan baru ID: {}", id)),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/admin/titik-macet").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal membuat titik macet: {}", e),
        )
            .into_response(),
    }
}

pub async fn delete_titik_macet(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    match db.delete_titik_macet(id).await {
        Ok(_) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Hapus Titik Macet",
                Some("Titik Macet"),
                Some(&format!("Menghapus titik macet ID: {}", id)),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/admin/titik-macet").into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("Gagal menghapus: {}", e)).into_response(),
    }
}

pub async fn audit(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    let audit_list = db.list_audit_logs().await;
    let template = AdminAuditTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "admin_audit",
        audit_list: &audit_list,
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Render error: {}", e),
        )
            .into_response(),
    }
}

pub async fn pengaturan(Extension(claims): Extension<Claims>) -> Response {
    if !check_role_permission(&claims, &["admin"]) {
        return render_forbidden();
    }

    let template = AdminPengaturanTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "admin_pengaturan",
    };

    match template.render() {
        Ok(html) => (StatusCode::OK, Html(html)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Render error: {}", e),
        )
            .into_response(),
    }
}
