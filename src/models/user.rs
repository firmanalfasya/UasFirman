use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i16,
    pub nama: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub nama: String,
    pub email: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub peran_id: i16,
    pub peran_nama: String,
    pub aktif: bool,
    pub dibuat_pada: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub remember_me: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserPayload {
    pub nama: String,
    pub email: String,
    pub username: String,
    pub password: String,
    pub peran_id: i16,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpdateUserPayload {
    pub nama: String,
    pub email: String,
    pub peran_id: i16,
    pub aktif: bool,
}
