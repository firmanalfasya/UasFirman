# SmartTraffic – Sistem Informasi Pemantauan Lalu Lintas Kota Bandar Lampung

## Deskripsi
Aplikasi **SmartTraffic** adalah sistem informasi pemantauan lalu lintas berbasis web untuk Dinas Perhubungan (Dishub) dan Polisi Lalu Lintas Kota Bandar Lampung. Aplikasi dibangun dengan **Rust (Axum + Tokio)** sebagai backend, **PostgreSQL** (SQLx) untuk penyimpanan, dan **Askama + HTMX + Tailwind CSS** untuk UI server‑rendered. Peta interaktif menggunakan **Leaflet + OpenStreetMap**, grafik menggunakan **Chart.js**.

## Teknologi
- **Backend**: Rust 1.70+, Axum, Tokio, SQLx, Argon2, JWT, CSRF, Rate‑limit, Security headers
- **Database**: PostgreSQL (opsional, mode *standalone* tersedia dengan data in‑memory)
- **Frontend**: Askama (templating), HTMX (ajax), Tailwind CSS (CDN), Leaflet, Chart.js, Inter font
- **Deploy**: Stateless, siap dipasang ke Vercel / Cloud Run, konfigurasi lewat variabel lingkungan
- **CI**: `cargo fmt`, `cargo clippy`, `cargo test`, `cargo build`

## Prasyarat
| Komponen | Versi minimum |
|----------|---------------|
| Rust toolchain (`rustc` & `cargo`) | 1.70 |
| PostgreSQL | 13 (hanya bila tidak memakai mode *standalone*) |
| Node.js (optional, hanya untuk Tailwind JIT) | 18 |
| Git | any |

## Pengaturan Lingkungan
Buat file `.env` di root proyek (contoh di `.env.example`). Semua variabel sensitif **tidak pernah** disimpan di repo.

```dotenv
# -------------------------------------------------
# Server & aplikasi
# -------------------------------------------------
APP_HOST=127.0.0.1
APP_PORT=8080
# Mode demo (tanpa PostgreSQL). Set ke false untuk produksi.
STANDALONE_DEMO=true

# -------------------------------------------------
# Basis data (hanya dipakai jika STANDALONE_DEMO=false)
# -------------------------------------------------
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=smarttraffic
POSTGRES_USER=smart_user
POSTGRES_PASSWORD=secret_password

# -------------------------------------------------
# JWT
# -------------------------------------------------
JWT_SECRET=changeme_to_a_strong_random_secret
JWT_EXPIRES_IN_SECONDS=86400

# -------------------------------------------------
# Argon2 (hash password)
# -------------------------------------------------
ARGON2_SALT=changeme_to_random_salt
ARGON2_ITERATIONS=3
ARGON2_MEMORY_KIB=65536
ARGON2_PARALLELISM=1

# -------------------------------------------------
# Rate limiting (login)
# -------------------------------------------------
RATE_LIMIT_WINDOW_SECONDS=60
RATE_LIMIT_MAX_REQUESTS=5
```

File `.env` harus **ditambahkan ke `.gitignore`** (sudah ada di repositori).

## Menjalankan Aplikasi (Mode Standalone)
```bash
# 1. Clone repo & masuk ke direktori
git clone <repo-url>
cd smarttraffic

# 2. Install dependensi Rust (akan mengunduh crate)
cargo fetch   # (perlu koneksi internet, pertama kali)

# 3. Jalankan migrasi (hanya bila menggunakan PostgreSQL)
#    Pada mode demo tidak diperlukan.
#    cargo install sqlx-cli --no-default-features --features postgres
#    sqlx migrate run

# 4. Jalankan server
cargo run
```
Buka browser dan kunjungi `http://localhost:8080`.

## Akun Demo
| Peran | Username | Password |
|------|----------|----------|
| Admin | admin | admin123 |
| Petugas Dishub | dishub | dishub123 |
| Polisi Lalu Lintas | polisi | polisi123 |
| Pimpinan | pimpinan | pimpinan123 |

## Struktur Folder
```
smarttraffic/
├─ src/
│  ├─ auth/          # hashing, JWT, middleware
│  ├─ db/            # SQLx pool / in‑memory store
│  ├─ handlers/      # route handlers per peran
│  ├─ middleware/    # security, rate‑limit, RBAC
│  ├─ models/        # domain structs & validation
│  ├─ routes/        # Axum router definisi
│  └─ main.rs        # entry point
├─ migrations/       # SQLx migasi schema & seed data
├─ templates/        # Askama HTML template
├─ static/           # img, css, js (leaflet, chart.js)
├─ Cargo.toml        # dependensi & metadata
├─ .env.example
└─ README.md
```

## Fitur Utama
- **Autentikasi & Otorisasi** (JWT di HttpOnly cookie, 4 peran)
- **CRUD Titik Macet** (admin)
- **Input Data Kemacetan** (petugas Dishub) – foto disimpan sebagai URL atau kolom `bytea` kecil
- **Dashboard Interaktif** – peta warna berdasarkan tingkat kemacetan, filter kecamatan / waktu, top‑5 titik terparah
- **Halaman Per‑Peran**
  - Admin: manajemen pengguna, titik macet, log audit, pengaturan
  - Dishub: pemantauan, input data, riwayat, rekomendasi
  - Polisi: notifikasi, penugasan, laporan lapangan
  - Pimpinan: KPI, grafik tren, donut per kecamatan, eksport PDF/Excel (read‑only)
- **Endpoint `/health`** dan halaman error 403/404/500
- **Logging terstruktur** (tracing + fmt)
- **Responsive design** (mobile 390 px – desktop 1440 px) dengan warna brand:
  - Primary `#1E3A8A`
  - Secondary `#0F766E`
  - Lancar `#16A34A`
  - Padat `#EAB308`
  - Macet `#EA580C`
  - Macet Parah `#DC2626`
- **Header**: logo Pemda + logo UTI + teks *SmartTraffic*
- **Footer** (tepat): `Copyright©2026 by Firman Alfasya Arief dan Dr. Erliyan Redy Susanto, M.Kom`

## Testing & Build
```bash
# Format & lint
cargo fmt -- --check
cargo clippy -- -D warnings

# Unit tests (pastikan disk space cukup)
cargo test

# Build release binary
cargo build --release
```
Semua perintah di atas harus lulus tanpa error.

## Screenshot (placeholder)
> Tambahkan screenshot UI pada folder `docs/` setelah menjalankan aplikasi.

---
*Dibuat dengan ❤️ oleh tim pengembang SmartTraffic.*

