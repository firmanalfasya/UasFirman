use askama::Template;
use axum::{
    extract::{Extension, Form, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::auth::jwt::Claims;
use crate::db::DbState;
use crate::middleware::auth::{check_role_permission, render_forbidden};
use crate::models::traffic::*;

#[derive(Template)]
#[template(path = "polisi/operasional.html")]
pub struct PolisiOperasionalTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub titik_list: &'a [TitikMacet],
    pub ringkasan: &'a DashboardRingkasan,
    pub notif_count: usize,
}

#[derive(Template)]
#[template(path = "polisi/notifikasi.html")]
pub struct PolisiNotifikasiTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub notifikasi_list: &'a [Notifikasi],
}

#[derive(Template)]
#[template(path = "polisi/penugasan.html")]
pub struct PolisiPenugasanTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub tindakan_list: &'a [TindakanPenanganan],
    pub titik_list: &'a [TitikMacet],
    pub message: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "polisi/laporan_lapangan.html")]
pub struct PolisiLaporanLapanganTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub ruas_list: &'a [RuasJalan],
    pub penyebab_list: &'a [PenyebabMacet],
    pub message: Option<&'a str>,
}

pub async fn operasional(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    let titik_list = db.list_titik_macet(None, None, None, None).await;
    let ringkasan = db.get_dashboard_ringkasan().await;
    let notifikasi_list = db.list_notifikasi(claims.user_id, 3).await;

    let template = PolisiOperasionalTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "polisi_operasional",
        titik_list: &titik_list,
        ringkasan: &ringkasan,
        notif_count: notifikasi_list.len(),
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

pub async fn notifikasi(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    let notifikasi_list = db.list_notifikasi(claims.user_id, 3).await;
    let template = PolisiNotifikasiTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "polisi_notifikasi",
        notifikasi_list: &notifikasi_list,
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

pub async fn mark_notifikasi_read(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    let _ = db.mark_notifikasi_read(id).await;
    Redirect::to("/polisi/notifikasi").into_response()
}

pub async fn penugasan(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    let tindakan_list = db.list_tindakan(None).await;
    let titik_list = db.list_titik_macet(None, None, None, None).await;

    let template = PolisiPenugasanTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "polisi_penugasan",
        tindakan_list: &tindakan_list,
        titik_list: &titik_list,
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

pub async fn create_penugasan(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Form(payload): Form<CreateTindakanInput>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    match db
        .create_tindakan(payload, claims.user_id, &claims.nama)
        .await
    {
        Ok(id) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Terbitkan Penugasan",
                Some("Tindakan Penanganan"),
                Some(&format!(
                    "Satlantas menerbitkan penugasan personel ID: {}",
                    id
                )),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/polisi/penugasan").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal menerbitkan penugasan: {}", e),
        )
            .into_response(),
    }
}

pub async fn laporan_lapangan_page(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    let ruas_list = db.list_ruas().await;
    let penyebab_list = db.list_penyebab().await;

    let template = PolisiLaporanLapanganTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "polisi_laporan",
        ruas_list: &ruas_list,
        penyebab_list: &penyebab_list,
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

pub async fn laporan_lapangan_submit(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Form(payload): Form<TitikMacetInput>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "polantas"]) {
        return render_forbidden();
    }

    match db
        .create_titik_macet(payload, claims.user_id, &claims.nama)
        .await
    {
        Ok(id) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Laporan Lapangan Satlantas",
                Some("Titik Macet"),
                Some(&format!(
                    "Personel Satlantas melaporkan insiden lapangan ID: {}",
                    id
                )),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/polisi/operasional").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal mengirim laporan: {}", e),
        )
            .into_response(),
    }
}
