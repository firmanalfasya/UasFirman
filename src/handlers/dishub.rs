use askama::Template;
use axum::{
    extract::{Extension, Form, Path},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use crate::auth::jwt::Claims;
use crate::db::DbState;
use crate::middleware::auth::{check_role_permission, render_forbidden};
use crate::models::{traffic::*, validation::validate_coordinates};

#[derive(Template)]
#[template(path = "dishub/pemantauan.html")]
pub struct DishubPemantauanTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub titik_list: &'a [TitikMacet],
    pub ringkasan: &'a DashboardRingkasan,
}

#[derive(Template)]
#[template(path = "dishub/input_kemacetan.html")]
pub struct DishubInputKemacetanTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub ruas_list: &'a [RuasJalan],
    pub penyebab_list: &'a [PenyebabMacet],
    pub message: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "dishub/riwayat.html")]
pub struct DishubRiwayatTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub riwayat_list: &'a [RiwayatKejadian],
}

#[derive(Template)]
#[template(path = "dishub/rekomendasi.html")]
pub struct DishubRekomendasiTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
}

pub async fn pemantauan(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "petugas_dishub"]) {
        return render_forbidden();
    }

    let titik_list = db.list_titik_macet(None, None, None, None).await;
    let ringkasan = db.get_dashboard_ringkasan().await;

    let template = DishubPemantauanTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "dishub_pemantauan",
        titik_list: &titik_list,
        ringkasan: &ringkasan,
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

pub async fn input_kemacetan_page(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "petugas_dishub"]) {
        return render_forbidden();
    }

    let ruas_list = db.list_ruas().await;
    let penyebab_list = db.list_penyebab().await;

    let template = DishubInputKemacetanTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "dishub_input",
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

pub async fn input_kemacetan_submit(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Form(input): Form<TitikMacetInput>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "petugas_dishub"]) {
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
                "Input Kemacetan",
                Some("Titik Macet"),
                Some(&format!(
                    "Petugas Dishub mencatat kemacetan baru ID: {}",
                    id
                )),
                Some("127.0.0.1"),
            )
            .await;
            Redirect::to("/dishub/pemantauan").into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal menyimpan kemacetan: {}", e),
        )
            .into_response(),
    }
}

pub async fn update_status(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Form(payload): Form<UpdateStatusInput>,
) -> Response {
    if !check_role_permission(&claims, &["admin", "petugas_dishub", "polantas"]) {
        return render_forbidden();
    }

    match db
        .update_titik_macet_status(
            id,
            &payload.status,
            payload.kepadatan.as_deref(),
            payload.tindakan.as_deref(),
            payload.catatan.as_deref(),
            claims.user_id,
            &claims.nama,
        )
        .await
    {
        Ok(_) => {
            db.add_audit_log(
                Some(claims.user_id),
                Some(&claims.nama),
                Some(&claims.role),
                "Update Status Kemacetan",
                Some("Titik Macet"),
                Some(&format!(
                    "Memperbarui status titik ID: {} menjadi {}",
                    id, payload.status
                )),
                Some("127.0.0.1"),
            )
            .await;

            if claims.role == "polantas" {
                Redirect::to("/polisi/operasional").into_response()
            } else {
                Redirect::to("/dishub/pemantauan").into_response()
            }
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            format!("Gagal update status: {}", e),
        )
            .into_response(),
    }
}

pub async fn riwayat(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(
        &claims,
        &["admin", "petugas_dishub", "polantas", "pimpinan"],
    ) {
        return render_forbidden();
    }

    let riwayat_list = db.list_riwayat(None).await;
    let template = DishubRiwayatTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "dishub_riwayat",
        riwayat_list: &riwayat_list,
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

pub async fn rekomendasi(Extension(claims): Extension<Claims>) -> Response {
    if !check_role_permission(
        &claims,
        &["admin", "petugas_dishub", "polantas", "pimpinan"],
    ) {
        return render_forbidden();
    }

    let template = DishubRekomendasiTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "dishub_rekomendasi",
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
