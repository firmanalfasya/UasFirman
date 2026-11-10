use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct RateLimiter {
    // Map IP ke (jumlah percobaan, waktu reset)
    attempts: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
    max_attempts: u32,
    window: Duration,
}

#[allow(dead_code)]
impl RateLimiter {
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        RateLimiter {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            max_attempts,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check_and_increment(&self, ip: &str) -> bool {
        let mut map = match self.attempts.lock() {
            Ok(guard) => guard,
            Err(_) => return true,
        };

        let now = Instant::now();
        // Bersihkan entri lama
        map.retain(|_, (_, reset_at)| *reset_at > now);

        let entry = map.entry(ip.to_string()).or_insert((0, now + self.window));
        if entry.0 >= self.max_attempts {
            false
        } else {
            entry.0 += 1;
            true
        }
    }
}

#[allow(dead_code)]
pub async fn login_rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    let rate_limiter = req
        .extensions()
        .get::<RateLimiter>()
        .cloned()
        .unwrap_or_else(|| RateLimiter::new(10, 60));

    let ip = addr.ip().to_string();
    if !rate_limiter.check_and_increment(&ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            "Terlalu banyak percobaan masuk. Silakan coba lagi dalam beberapa saat demi alasan keamanan.",
        )
            .into_response();
    }

    next.run(req).await
}
