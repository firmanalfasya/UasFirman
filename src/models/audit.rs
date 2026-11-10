use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAudit {
    pub id: i64,
    pub pengguna_id: Option<i64>,
    pub pengguna_nama: Option<String>,
    pub peran_nama: Option<String>,
    pub aksi: String,
    pub entitas: Option<String>,
    pub detail: Option<String>,
    pub ip_address: Option<String>,
    pub waktu: DateTime<Utc>,
}

