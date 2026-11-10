use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kecamatan {
    pub id: i16,
    pub nama: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuasJalan {
    pub id: i64,
    pub nama: String,
    pub kecamatan_id: i16,
    pub kecamatan_nama: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PenyebabMacet {
    pub id: i16,
    pub nama: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitikMacet {
    pub id: i64,
    pub ruas_id: i64,
    pub ruas_nama: String,
    pub kecamatan_id: i16,
    pub kecamatan_nama: String,
    pub penyebab_id: Option<i16>,
    pub penyebab_nama: String,
    pub latitude: f64,
    pub longitude: f64,
    pub kepadatan: String, // lancar, padat, macet, macet_parah
    pub status: String,    // dilaporkan, ditangani, selesai
    pub panjang_antrean_meter: i32,
    pub kecepatan_rata_rata_kmh: i32,
    pub waktu_mulai: DateTime<Utc>,
    pub waktu_selesai: Option<DateTime<Utc>>,
    pub catatan: Option<String>,
    pub foto_url: Option<String>,
    pub dibuat_oleh: i64,
    pub pembuat_nama: String,
    pub dibuat_pada: DateTime<Utc>,
    pub diubah_pada: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiwayatKejadian {
    pub id: i64,
    pub titik_id: i64,
    pub ruas_nama: String,
    pub kepadatan: String,
    pub status: String,
    pub panjang_antrean_meter: i32,
    pub kecepatan_rata_rata_kmh: i32,
    pub waktu_kejadian: DateTime<Utc>,
    pub catatan: Option<String>,
    pub foto_url: Option<String>,
    pub dicatat_oleh: i64,
    pub pencatat_nama: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TindakanPenanganan {
    pub id: i64,
    pub titik_id: i64,
    pub ruas_nama: String,
    pub jenis: String,
    pub deskripsi: Option<String>,
    pub status_tindakan: String,
    pub petugas_id: i64,
    pub petugas_nama: String,
    pub waktu_tindakan: DateTime<Utc>,
    pub waktu_selesai: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notifikasi {
    pub id: i64,
    pub pengguna_id: Option<i64>,
    pub peran_id: Option<i16>,
    pub judul: String,
    pub pesan: String,
    pub tipe: String, // info, peringatan, darurat
    pub tautan: Option<String>,
    pub dibaca: bool,
    pub dibuat_pada: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct TitikMacetInput {
    pub ruas_id: i64,
    pub penyebab_id: Option<i16>,
    pub latitude: f64,
    pub longitude: f64,
    pub kepadatan: String,
    pub status: Option<String>,
    pub panjang_antrean_meter: Option<i32>,
    pub kecepatan_rata_rata_kmh: Option<i32>,
    pub catatan: Option<String>,
    pub foto_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusInput {
    pub status: String,
    pub kepadatan: Option<String>,
    pub tindakan: Option<String>,
    pub catatan: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTindakanInput {
    pub titik_id: i64,
    pub jenis: String,
    pub deskripsi: Option<String>,
    pub status_tindakan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRingkasan {
    pub total_titik: usize,
    pub titik_aktif: usize,
    pub macet_parah: usize,
    pub macet: usize,
    pub padat: usize,
    pub lancar: usize,
    pub dilaporkan: usize,
    pub ditangani: usize,
    pub selesai: usize,
    pub insiden_hari_ini: usize,
    pub petugas_bertugas: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingItem {
    pub rank: usize,
    pub titik_id: i64,
    pub nama_ruas: String,
    pub kecamatan: String,
    pub kepadatan: String,
    pub frekuensi: i32,
    pub durasi_rata_rata_menit: i32,
    pub panjang_antrean_meter: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrenItem {
    pub label: String,
    pub jumlah: i32,
    pub kepadatan_rata_rata: String,
}

