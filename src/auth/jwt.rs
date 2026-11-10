use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub user_id: i64,
    pub nama: String,
    pub email: String,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt(
    user_id: i64,
    nama: &str,
    email: &str,
    username: &str,
    role: &str,
    secret: &str,
    expiration_seconds: i64,
) -> Result<String, String> {
    let now = Utc::now().timestamp() as usize;
    let exp = (Utc::now().timestamp() + expiration_seconds) as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        user_id,
        nama: nama.to_string(),
        email: email.to_string(),
        username: username.to_string(),
        role: role.to_string(),
        exp,
        iat: now,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| format!("Gagal mengenkripsi token JWT: {}", e))
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, String> {
    let validation = Validation::default();
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|e| format!("Token JWT tidak valid atau kedaluwarsa: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_create_and_verify() {
        let secret = "kunci-rahasia-uji-coba-smarttraffic-bandar-lampung-2026";
        let token = create_jwt(
            1,
            "Ir. Hendra",
            "admin@smarttraffic.go.id",
            "admin",
            "admin",
            secret,
            3600,
        )
        .expect("Pembuatan JWT gagal");

        let claims = verify_jwt(&token, secret).expect("Verifikasi JWT gagal");
        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.role, "admin");
        assert_eq!(claims.email, "admin@smarttraffic.go.id");
    }
}

