use askama::Template;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

use crate::auth::jwt::Claims;
use crate::db::DbState;
use crate::middleware::auth::{check_role_permission, render_forbidden};
use crate::models::traffic::*;

#[derive(Template)]
#[template(path = "pimpinan/dashboard.html")]
pub struct PimpinanDashboardTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub ringkasan: &'a DashboardRingkasan,
    pub ranking_list: &'a [RankingItem],
    pub tren_list: &'a [TrenItem],
    pub sebaran_list: &'a [(String, usize)],
}

#[derive(Template)]
#[template(path = "pimpinan/analitik.html")]
pub struct PimpinanAnalitikTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub ringkasan: &'a DashboardRingkasan,
    pub titik_list: &'a [TitikMacet],
}

#[derive(Template)]
#[template(path = "pimpinan/peta.html")]
pub struct PimpinanPetaTemplate<'a> {
    pub user_nama: &'a str,
    pub user_role: &'a str,
    pub active_menu: &'a str,
    pub titik_list: &'a [TitikMacet],
}

pub async fn dashboard(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(
        &claims,
        &["admin", "petugas_dishub", "polantas", "pimpinan"],
    ) {
        return render_forbidden();
    }

    let ringkasan = db.get_dashboard_ringkasan().await;
    let ranking_list = db.get_top_ranking(5).await;
    let tren_list = db.get_tren_jam().await;
    let sebaran_list = db.get_sebaran_kecamatan().await;

    let template = PimpinanDashboardTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "pimpinan_dashboard",
        ringkasan: &ringkasan,
        ranking_list: &ranking_list,
        tren_list: &tren_list,
        sebaran_list: &sebaran_list,
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

pub async fn analitik(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(
        &claims,
        &["admin", "petugas_dishub", "polantas", "pimpinan"],
    ) {
        return render_forbidden();
    }

    let ringkasan = db.get_dashboard_ringkasan().await;
    let titik_list = db.list_titik_macet(None, None, None, None).await;

    let template = PimpinanAnalitikTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "pimpinan_analitik",
        ringkasan: &ringkasan,
        titik_list: &titik_list,
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

pub async fn peta(
    Extension(db): Extension<DbState>,
    Extension(claims): Extension<Claims>,
) -> Response {
    if !check_role_permission(
        &claims,
        &["admin", "petugas_dishub", "polantas", "pimpinan"],
    ) {
        return render_forbidden();
    }

    let titik_list = db.list_titik_macet(None, None, None, None).await;

    let template = PimpinanPetaTemplate {
        user_nama: &claims.nama,
        user_role: &claims.role,
        active_menu: "pimpinan_peta",
        titik_list: &titik_list,
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
