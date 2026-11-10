use axum::{
    extract::{Extension, Query},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::db::DbState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub timestamp: String,
    pub database: &'static str,
    pub version: &'static str,
}

#[derive(Deserialize)]
pub struct LaporanQuery {
    pub format: Option<String>,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "SmartTraffic Bandar Lampung",
        timestamp: chrono::Utc::now().to_rfc3339(),
        database: "PostgreSQL / Memory Sync Active",
        version: "2.6.0",
    })
}

pub async fn api_titik_macet(Extension(db): Extension<DbState>) -> Response {
    let list = db.list_titik_macet(None, None, None, None).await;
    Json(json!({
        "status": "success",
        "total": list.len(),
        "data": list
    }))
    .into_response()
}

pub async fn api_ringkasan(Extension(db): Extension<DbState>) -> Response {
    let r = db.get_dashboard_ringkasan().await;
    Json(json!({
        "status": "success",
        "ringkasan": r
    }))
    .into_response()
}

pub async fn api_tren(Extension(db): Extension<DbState>) -> Response {
    let tren = db.get_tren_jam().await;
    Json(json!({
        "status": "success",
        "tren_per_jam": tren
    }))
    .into_response()
}

pub async fn api_ranking(Extension(db): Extension<DbState>) -> Response {
    let rank = db.get_top_ranking(10).await;
    Json(json!({
        "status": "success",
        "ranking": rank
    }))
    .into_response()
}

pub async fn api_laporan(
    Extension(db): Extension<DbState>,
    Query(q): Query<LaporanQuery>,
) -> Response {
    let list = db.list_titik_macet(None, None, None, None).await;

    let format = q.format.unwrap_or_else(|| "csv".to_string());
    if format == "csv" {
        let mut wtr = csv::Writer::from_writer(vec![]);
        // Header CSV
        let _ = wtr.write_record([
            "ID",
            "Ruas Jalan",
            "Kecamatan",
            "Latitude",
            "Longitude",
            "Tingkat Kepadatan",
            "Status Penanganan",
            "Panjang Antrean (m)",
            "Kecepatan Rerata (km/h)",
            "Penyebab",
            "Catatan",
            "Waktu Mulai",
        ]);

        for t in list {
            let _ = wtr.write_record(&[
                t.id.to_string(),
                t.ruas_nama,
                t.kecamatan_nama,
                t.latitude.to_string(),
                t.longitude.to_string(),
                t.kepadatan,
                t.status,
                t.panjang_antrean_meter.to_string(),
                t.kecepatan_rata_rata_kmh.to_string(),
                t.penyebab_nama,
                t.catatan.unwrap_or_default(),
                t.waktu_mulai.to_rfc3339(),
            ]);
        }

        let csv_data = match wtr.into_inner() {
            Ok(bytes) => bytes,
            Err(_) => {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Gagal membuat CSV").into_response()
            }
        };

        (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"laporan_titik_macet_bandar_lampung.csv\"",
                ),
            ],
            csv_data,
        )
            .into_response()
    } else {
        Json(json!({
            "status": "success",
            "data": list
        }))
        .into_response()
    }
}
