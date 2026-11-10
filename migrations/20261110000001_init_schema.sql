-- ==============================================================================
-- Migrasi 1: Inisialisasi Skema SmartTraffic Bandar Lampung
-- ==============================================================================

-- 1. Tabel Peran Sistem (RBAC)
CREATE TABLE IF NOT EXISTS peran (
    id SMALLSERIAL PRIMARY KEY,
    nama VARCHAR(30) NOT NULL UNIQUE
);

-- 2. Tabel Pengguna (Users)
CREATE TABLE IF NOT EXISTS pengguna (
    id BIGSERIAL PRIMARY KEY,
    nama VARCHAR(100) NOT NULL,
    email VARCHAR(150) NOT NULL UNIQUE,
    username VARCHAR(50) UNIQUE,
    password_hash TEXT NOT NULL,
    peran_id SMALLINT NOT NULL REFERENCES peran(id),
    aktif BOOLEAN NOT NULL DEFAULT TRUE,
    dibuat_pada TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 3. Master Kecamatan di Kota Bandar Lampung
CREATE TABLE IF NOT EXISTS kecamatan (
    id SMALLSERIAL PRIMARY KEY,
    nama VARCHAR(80) NOT NULL UNIQUE
);

-- 4. Master Ruas Jalan
CREATE TABLE IF NOT EXISTS ruas_jalan (
    id BIGSERIAL PRIMARY KEY,
    nama VARCHAR(150) NOT NULL,
    kecamatan_id SMALLINT NOT NULL REFERENCES kecamatan(id),
    UNIQUE (nama, kecamatan_id)
);

-- 5. Master Penyebab Macet
CREATE TABLE IF NOT EXISTS penyebab_macet (
    id SMALLSERIAL PRIMARY KEY,
    nama VARCHAR(80) NOT NULL UNIQUE
);

-- 6. Tabel Inti Titik Macet
CREATE TABLE IF NOT EXISTS titik_macet (
    id BIGSERIAL PRIMARY KEY,
    ruas_id BIGINT NOT NULL REFERENCES ruas_jalan(id),
    penyebab_id SMALLINT REFERENCES penyebab_macet(id),
    latitude DOUBLE PRECISION NOT NULL CHECK (latitude BETWEEN -6.0 AND -5.0),
    longitude DOUBLE PRECISION NOT NULL CHECK (longitude BETWEEN 104.5 AND 106.0),
    kepadatan VARCHAR(20) NOT NULL CHECK (kepadatan IN ('lancar','padat','macet','macet_parah','ringan','sedang','berat')),
    status VARCHAR(20) NOT NULL DEFAULT 'dilaporkan' CHECK (status IN ('dilaporkan','ditangani','selesai')),
    panjang_antrean_meter INTEGER DEFAULT 0,
    kecepatan_rata_rata_kmh INTEGER DEFAULT 0,
    waktu_mulai TIMESTAMPTZ NOT NULL DEFAULT now(),
    waktu_selesai TIMESTAMPTZ,
    catatan TEXT,
    foto_url TEXT,
    dibuat_oleh BIGINT NOT NULL REFERENCES pengguna(id),
    dibuat_pada TIMESTAMPTZ NOT NULL DEFAULT now(),
    diubah_pada TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (waktu_selesai IS NULL OR waktu_selesai >= waktu_mulai)
);

CREATE INDEX IF NOT EXISTS idx_titik_status ON titik_macet(status);
CREATE INDEX IF NOT EXISTS idx_titik_kepadatan ON titik_macet(kepadatan);
CREATE INDEX IF NOT EXISTS idx_titik_waktu ON titik_macet(waktu_mulai);
CREATE INDEX IF NOT EXISTS idx_titik_ruas ON titik_macet(ruas_id);

-- 7. Tabel Riwayat Kejadian / Laporan Kemacetan
CREATE TABLE IF NOT EXISTS riwayat_kejadian (
    id BIGSERIAL PRIMARY KEY,
    titik_id BIGINT NOT NULL REFERENCES titik_macet(id) ON DELETE CASCADE,
    kepadatan VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    panjang_antrean_meter INTEGER DEFAULT 0,
    kecepatan_rata_rata_kmh INTEGER DEFAULT 0,
    waktu_kejadian TIMESTAMPTZ NOT NULL DEFAULT now(),
    catatan TEXT,
    foto_url TEXT,
    dicatat_oleh BIGINT NOT NULL REFERENCES pengguna(id)
);

CREATE INDEX IF NOT EXISTS idx_riwayat_titik ON riwayat_kejadian(titik_id);
CREATE INDEX IF NOT EXISTS idx_riwayat_waktu ON riwayat_kejadian(waktu_kejadian);

-- 8. Tabel Tindakan Penanganan & Penugasan Petugas Lapangan
CREATE TABLE IF NOT EXISTS tindakan_penanganan (
    id BIGSERIAL PRIMARY KEY,
    titik_id BIGINT NOT NULL REFERENCES titik_macet(id) ON DELETE CASCADE,
    jenis VARCHAR(60) NOT NULL,
    deskripsi TEXT,
    status_tindakan VARCHAR(30) NOT NULL DEFAULT 'ditugaskan',
    petugas_id BIGINT NOT NULL REFERENCES pengguna(id),
    waktu_tindakan TIMESTAMPTZ NOT NULL DEFAULT now(),
    waktu_selesai TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_tindakan_titik ON tindakan_penanganan(titik_id);
CREATE INDEX IF NOT EXISTS idx_tindakan_petugas ON tindakan_penanganan(petugas_id);

-- 9. Tabel Notifikasi & Peringatan Operasional
CREATE TABLE IF NOT EXISTS notifikasi (
    id BIGSERIAL PRIMARY KEY,
    pengguna_id BIGINT REFERENCES pengguna(id) ON DELETE CASCADE,
    peran_id SMALLINT REFERENCES peran(id) ON DELETE CASCADE,
    judul VARCHAR(150) NOT NULL,
    pesan TEXT NOT NULL,
    tipe VARCHAR(30) NOT NULL DEFAULT 'info',
    tautan TEXT,
    dibaca BOOLEAN NOT NULL DEFAULT FALSE,
    dibuat_pada TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_notifikasi_user ON notifikasi(pengguna_id, dibaca);

-- 10. Tabel Lampiran Foto Kejadian
CREATE TABLE IF NOT EXISTS lampiran_foto (
    id BIGSERIAL PRIMARY KEY,
    titik_id BIGINT NOT NULL REFERENCES titik_macet(id) ON DELETE CASCADE,
    nama_berkas VARCHAR(200) NOT NULL,
    tipe VARCHAR(30) NOT NULL CHECK (tipe IN ('image/jpeg','image/png','image/webp')),
    ukuran_byte INTEGER NOT NULL CHECK (ukuran_byte <= 2097152),
    url TEXT NOT NULL,
    diunggah_oleh BIGINT NOT NULL REFERENCES pengguna(id),
    diunggah_pada TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 11. Tabel Log Audit Aktivitas Sistem & Keamanan
CREATE TABLE IF NOT EXISTS log_audit (
    id BIGSERIAL PRIMARY KEY,
    pengguna_id BIGINT REFERENCES pengguna(id) ON DELETE SET NULL,
    aksi VARCHAR(60) NOT NULL,
    entitas VARCHAR(60),
    detail JSONB,
    ip_address VARCHAR(45),
    waktu TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_audit_waktu ON log_audit(waktu DESC);

