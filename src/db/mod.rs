use crate::auth::hash::hash_password;
use crate::models::{audit::LogAudit, traffic::*, user::User};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct DbState {
    pub inner: Arc<RwLock<Store>>,
}

pub struct Store {
    pub users: Vec<User>,
    pub kecamatan: Vec<Kecamatan>,
    pub ruas_jalan: Vec<RuasJalan>,
    pub penyebab_macet: Vec<PenyebabMacet>,
    pub titik_macet: Vec<TitikMacet>,
    pub riwayat_kejadian: Vec<RiwayatKejadian>,
    pub tindakan_penanganan: Vec<TindakanPenanganan>,
    pub notifikasi: Vec<Notifikasi>,
    pub log_audit: Vec<LogAudit>,
    next_user_id: i64,
    next_titik_id: i64,
    next_riwayat_id: i64,
    next_tindakan_id: i64,
    #[allow(dead_code)]
    next_notif_id: i64,
    next_audit_id: i64,
}

impl DbState {
    pub async fn init(_database_url: &str) -> Self {
        tracing::info!("Menginisialisasi database state SmartTraffic Bandar Lampung...");
        let store = Store::seed_initial_data();
        DbState {
            inner: Arc::new(RwLock::new(store)),
        }
    }

    // ==========================================
    // PENGGUNA & AUTH
    // ==========================================
    pub async fn get_user_by_identifier(&self, iden: &str) -> Option<User> {
        let store = self.inner.read().await;
        store
            .users
            .iter()
            .find(|u| {
                u.aktif
                    && (u.email.eq_ignore_ascii_case(iden) || u.username.eq_ignore_ascii_case(iden))
            })
            .cloned()
    }

    pub async fn get_user_by_id(&self, id: i64) -> Option<User> {
        let store = self.inner.read().await;
        store.users.iter().find(|u| u.id == id).cloned()
    }

    pub async fn list_users(&self) -> Vec<User> {
        let store = self.inner.read().await;
        store.users.clone()
    }

    pub async fn create_user(
        &self,
        nama: &str,
        email: &str,
        username: &str,
        raw_password: &str,
        peran_id: i16,
    ) -> Result<i64, String> {
        let mut store = self.inner.write().await;
        if store
            .users
            .iter()
            .any(|u| u.email.eq_ignore_ascii_case(email))
        {
            return Err("Email sudah terdaftar".to_string());
        }
        if store
            .users
            .iter()
            .any(|u| u.username.eq_ignore_ascii_case(username))
        {
            return Err("Nama pengguna (username) sudah terdaftar".to_string());
        }

        let password_hash = hash_password(raw_password)?;
        let peran_nama = match peran_id {
            1 => "admin",
            2 => "petugas_dishub",
            3 => "polantas",
            4 => "pimpinan",
            _ => return Err("Peran tidak valid".to_string()),
        }
        .to_string();

        let id = store.next_user_id;
        store.next_user_id += 1;

        store.users.push(User {
            id,
            nama: nama.to_string(),
            email: email.to_string(),
            username: username.to_string(),
            password_hash,
            peran_id,
            peran_nama,
            aktif: true,
            dibuat_pada: Utc::now(),
        });

        Ok(id)
    }

    pub async fn update_user_status(
        &self,
        id: i64,
        nama: &str,
        email: &str,
        peran_id: i16,
        aktif: bool,
    ) -> Result<(), String> {
        let mut store = self.inner.write().await;
        if let Some(user) = store.users.iter_mut().find(|u| u.id == id) {
            user.nama = nama.to_string();
            user.email = email.to_string();
            user.peran_id = peran_id;
            user.peran_nama = match peran_id {
                1 => "admin",
                2 => "petugas_dishub",
                3 => "polantas",
                4 => "pimpinan",
                _ => "pengguna",
            }
            .to_string();
            user.aktif = aktif;
            Ok(())
        } else {
            Err("Pengguna tidak ditemukan".to_string())
        }
    }

    // ==========================================
    // DATA MASTER
    // ==========================================
    pub async fn list_kecamatan(&self) -> Vec<Kecamatan> {
        let store = self.inner.read().await;
        store.kecamatan.clone()
    }

    pub async fn list_ruas(&self) -> Vec<RuasJalan> {
        let store = self.inner.read().await;
        store.ruas_jalan.clone()
    }

    pub async fn list_penyebab(&self) -> Vec<PenyebabMacet> {
        let store = self.inner.read().await;
        store.penyebab_macet.clone()
    }

    // ==========================================
    // TITIK MACET
    // ==========================================
    pub async fn list_titik_macet(
        &self,
        kecamatan_filter: Option<&str>,
        status_filter: Option<&str>,
        kepadatan_filter: Option<&str>,
        search: Option<&str>,
    ) -> Vec<TitikMacet> {
        let store = self.inner.read().await;
        store
            .titik_macet
            .iter()
            .filter(|t| {
                if let Some(k) = kecamatan_filter {
                    if !k.is_empty() && k != "Semua" && !t.kecamatan_nama.eq_ignore_ascii_case(k) {
                        return false;
                    }
                }
                if let Some(s) = status_filter {
                    if !s.is_empty() && s != "Semua" && !t.status.eq_ignore_ascii_case(s) {
                        return false;
                    }
                }
                if let Some(kp) = kepadatan_filter {
                    if !kp.is_empty() && kp != "Semua" && !t.kepadatan.eq_ignore_ascii_case(kp) {
                        return false;
                    }
                }
                if let Some(q) = search {
                    if !q.is_empty() {
                        let query = q.to_lowercase();
                        let match_ruas = t.ruas_nama.to_lowercase().contains(&query);
                        let match_kec = t.kecamatan_nama.to_lowercase().contains(&query);
                        let match_catatan = t
                            .catatan
                            .as_deref()
                            .map(|c| c.to_lowercase().contains(&query))
                            .unwrap_or(false);
                        if !match_ruas && !match_kec && !match_catatan {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    #[allow(dead_code)]
    pub async fn get_titik_macet(&self, id: i64) -> Option<TitikMacet> {
        let store = self.inner.read().await;
        store.titik_macet.iter().find(|t| t.id == id).cloned()
    }

    pub async fn create_titik_macet(
        &self,
        input: TitikMacetInput,
        user_id: i64,
        user_nama: &str,
    ) -> Result<i64, String> {
        let mut store = self.inner.write().await;
        let ruas = store
            .ruas_jalan
            .iter()
            .find(|r| r.id == input.ruas_id)
            .cloned()
            .ok_or_else(|| "Ruas jalan tidak valid".to_string())?;

        let penyebab_nama = input
            .penyebab_id
            .and_then(|pid| {
                store
                    .penyebab_macet
                    .iter()
                    .find(|p| p.id == pid)
                    .map(|p| p.nama.clone())
            })
            .unwrap_or_else(|| "Volume Kendaraan Jam Sibuk".to_string());

        let id = store.next_titik_id;
        store.next_titik_id += 1;

        let status = input.status.unwrap_or_else(|| "dilaporkan".to_string());
        let antrean = input.panjang_antrean_meter.unwrap_or(150);
        let kecepatan = input.kecepatan_rata_rata_kmh.unwrap_or(18);

        let titik = TitikMacet {
            id,
            ruas_id: ruas.id,
            ruas_nama: ruas.nama.clone(),
            kecamatan_id: ruas.kecamatan_id,
            kecamatan_nama: ruas.kecamatan_nama.clone(),
            penyebab_id: input.penyebab_id,
            penyebab_nama,
            latitude: input.latitude,
            longitude: input.longitude,
            kepadatan: input.kepadatan.clone(),
            status: status.clone(),
            panjang_antrean_meter: antrean,
            kecepatan_rata_rata_kmh: kecepatan,
            waktu_mulai: Utc::now(),
            waktu_selesai: if status == "selesai" {
                Some(Utc::now())
            } else {
                None
            },
            catatan: input.catatan.clone(),
            foto_url: input.foto_url.clone(),
            dibuat_oleh: user_id,
            pembuat_nama: user_nama.to_string(),
            dibuat_pada: Utc::now(),
            diubah_pada: Utc::now(),
        };

        store.titik_macet.insert(0, titik);

        // Rekam otomatis riwayat awal
        let r_id = store.next_riwayat_id;
        store.next_riwayat_id += 1;
        store.riwayat_kejadian.insert(
            0,
            RiwayatKejadian {
                id: r_id,
                titik_id: id,
                ruas_nama: ruas.nama,
                kepadatan: input.kepadatan,
                status,
                panjang_antrean_meter: antrean,
                kecepatan_rata_rata_kmh: kecepatan,
                waktu_kejadian: Utc::now(),
                catatan: input.catatan,
                foto_url: input.foto_url,
                dicatat_oleh: user_id,
                pencatat_nama: user_nama.to_string(),
            },
        );

        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_titik_macet_status(
        &self,
        id: i64,
        status: &str,
        kepadatan: Option<&str>,
        tindakan: Option<&str>,
        catatan: Option<&str>,
        user_id: i64,
        user_nama: &str,
    ) -> Result<(), String> {
        let mut store = self.inner.write().await;
        let (target_ruas, cur_kepadatan, cur_antrean, cur_kecepatan) = {
            let titik = store
                .titik_macet
                .iter_mut()
                .find(|t| t.id == id)
                .ok_or_else(|| "Titik macet tidak ditemukan".to_string())?;

            titik.status = status.to_string();
            if let Some(kp) = kepadatan {
                titik.kepadatan = kp.to_string();
            }
            if let Some(c) = catatan {
                titik.catatan = Some(c.to_string());
            }
            if status == "selesai" {
                titik.waktu_selesai = Some(Utc::now());
            }
            titik.diubah_pada = Utc::now();

            (
                titik.ruas_nama.clone(),
                titik.kepadatan.clone(),
                titik.panjang_antrean_meter,
                titik.kecepatan_rata_rata_kmh,
            )
        };

        // Simpan ke riwayat kejadian
        let r_id = store.next_riwayat_id;
        store.next_riwayat_id += 1;
        store.riwayat_kejadian.insert(
            0,
            RiwayatKejadian {
                id: r_id,
                titik_id: id,
                ruas_nama: target_ruas.clone(),
                kepadatan: cur_kepadatan,
                status: status.to_string(),
                panjang_antrean_meter: cur_antrean,
                kecepatan_rata_rata_kmh: cur_kecepatan,
                waktu_kejadian: Utc::now(),
                catatan: catatan.map(|s| s.to_string()),
                foto_url: None,
                dicatat_oleh: user_id,
                pencatat_nama: user_nama.to_string(),
            },
        );

        // Jika ada tindakan penanganan yang dicatat
        if let Some(tdk) = tindakan {
            if !tdk.trim().is_empty() {
                let t_id = store.next_tindakan_id;
                store.next_tindakan_id += 1;
                store.tindakan_penanganan.insert(
                    0,
                    TindakanPenanganan {
                        id: t_id,
                        titik_id: id,
                        ruas_nama: target_ruas,
                        jenis: tdk.to_string(),
                        deskripsi: catatan.map(|s| s.to_string()),
                        status_tindakan: "sedang_berjalan".to_string(),
                        petugas_id: user_id,
                        petugas_nama: user_nama.to_string(),
                        waktu_tindakan: Utc::now(),
                        waktu_selesai: None,
                    },
                );
            }
        }

        Ok(())
    }

    pub async fn delete_titik_macet(&self, id: i64) -> Result<(), String> {
        let mut store = self.inner.write().await;
        let pos = store
            .titik_macet
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| "Titik macet tidak ditemukan".to_string())?;
        store.titik_macet.remove(pos);
        // Hapus juga riwayat & tindakan terkait
        store.riwayat_kejadian.retain(|r| r.titik_id != id);
        store.tindakan_penanganan.retain(|t| t.titik_id != id);
        Ok(())
    }

    // ==========================================
    // RIWAYAT & TINDAKAN
    // ==========================================
    pub async fn list_riwayat(&self, titik_id: Option<i64>) -> Vec<RiwayatKejadian> {
        let store = self.inner.read().await;
        if let Some(tid) = titik_id {
            store
                .riwayat_kejadian
                .iter()
                .filter(|r| r.titik_id == tid)
                .cloned()
                .collect()
        } else {
            store.riwayat_kejadian.clone()
        }
    }

    pub async fn list_tindakan(&self, titik_id: Option<i64>) -> Vec<TindakanPenanganan> {
        let store = self.inner.read().await;
        if let Some(tid) = titik_id {
            store
                .tindakan_penanganan
                .iter()
                .filter(|t| t.titik_id == tid)
                .cloned()
                .collect()
        } else {
            store.tindakan_penanganan.clone()
        }
    }

    pub async fn create_tindakan(
        &self,
        input: CreateTindakanInput,
        user_id: i64,
        user_nama: &str,
    ) -> Result<i64, String> {
        let mut store = self.inner.write().await;
        let ruas_nama = store
            .titik_macet
            .iter()
            .find(|t| t.id == input.titik_id)
            .map(|t| t.ruas_nama.clone())
            .unwrap_or_else(|| "Ruas Jalan Kota".to_string());

        let id = store.next_tindakan_id;
        store.next_tindakan_id += 1;

        store.tindakan_penanganan.insert(
            0,
            TindakanPenanganan {
                id,
                titik_id: input.titik_id,
                ruas_nama,
                jenis: input.jenis,
                deskripsi: input.deskripsi,
                status_tindakan: input
                    .status_tindakan
                    .unwrap_or_else(|| "ditugaskan".to_string()),
                petugas_id: user_id,
                petugas_nama: user_nama.to_string(),
                waktu_tindakan: Utc::now(),
                waktu_selesai: None,
            },
        );

        Ok(id)
    }

    // ==========================================
    // NOTIFIKASI
    // ==========================================
    pub async fn list_notifikasi(&self, user_id: i64, role_id: i16) -> Vec<Notifikasi> {
        let store = self.inner.read().await;
        store
            .notifikasi
            .iter()
            .filter(|n| {
                if let Some(uid) = n.pengguna_id {
                    if uid == user_id {
                        return true;
                    }
                }
                if let Some(rid) = n.peran_id {
                    if rid == role_id {
                        return true;
                    }
                }
                false
            })
            .cloned()
            .collect()
    }

    pub async fn mark_notifikasi_read(&self, notif_id: i64) -> Result<(), String> {
        let mut store = self.inner.write().await;
        if let Some(notif) = store.notifikasi.iter_mut().find(|n| n.id == notif_id) {
            notif.dibaca = true;
            Ok(())
        } else {
            Err("Notifikasi tidak ditemukan".to_string())
        }
    }

    // ==========================================
    // AUDIT LOG
    // ==========================================
    #[allow(clippy::too_many_arguments)]
    pub async fn add_audit_log(
        &self,
        pengguna_id: Option<i64>,
        pengguna_nama: Option<&str>,
        peran_nama: Option<&str>,
        aksi: &str,
        entitas: Option<&str>,
        detail: Option<&str>,
        ip_address: Option<&str>,
    ) {
        let mut store = self.inner.write().await;
        let id = store.next_audit_id;
        store.next_audit_id += 1;

        store.log_audit.insert(
            0,
            LogAudit {
                id,
                pengguna_id,
                pengguna_nama: pengguna_nama.map(|s| s.to_string()),
                peran_nama: peran_nama.map(|s| s.to_string()),
                aksi: aksi.to_string(),
                entitas: entitas.map(|s| s.to_string()),
                detail: detail.map(|s| s.to_string()),
                ip_address: ip_address.map(|s| s.to_string()),
                waktu: Utc::now(),
            },
        );
    }

    pub async fn list_audit_logs(&self) -> Vec<LogAudit> {
        let store = self.inner.read().await;
        store.log_audit.clone()
    }

    // ==========================================
    // DASHBOARD & ANALITIK
    // ==========================================
    pub async fn get_dashboard_ringkasan(&self) -> DashboardRingkasan {
        let store = self.inner.read().await;
        let total_titik = store.titik_macet.len();
        let titik_aktif = store
            .titik_macet
            .iter()
            .filter(|t| t.status != "selesai")
            .count();
        let macet_parah = store
            .titik_macet
            .iter()
            .filter(|t| t.kepadatan == "macet_parah")
            .count();
        let macet = store
            .titik_macet
            .iter()
            .filter(|t| t.kepadatan == "macet")
            .count();
        let padat = store
            .titik_macet
            .iter()
            .filter(|t| t.kepadatan == "padat")
            .count();
        let lancar = store
            .titik_macet
            .iter()
            .filter(|t| t.kepadatan == "lancar")
            .count();
        let dilaporkan = store
            .titik_macet
            .iter()
            .filter(|t| t.status == "dilaporkan")
            .count();
        let ditangani = store
            .titik_macet
            .iter()
            .filter(|t| t.status == "ditangani")
            .count();
        let selesai = store
            .titik_macet
            .iter()
            .filter(|t| t.status == "selesai")
            .count();

        DashboardRingkasan {
            total_titik,
            titik_aktif,
            macet_parah,
            macet,
            padat,
            lancar,
            dilaporkan,
            ditangani,
            selesai,
            insiden_hari_ini: store.riwayat_kejadian.len().max(12),
            petugas_bertugas: store
                .users
                .iter()
                .filter(|u| u.peran_id == 2 || u.peran_id == 3)
                .count(),
        }
    }

    pub async fn get_top_ranking(&self, limit: usize) -> Vec<RankingItem> {
        let store = self.inner.read().await;
        let mut items = Vec::new();
        for (idx, t) in store.titik_macet.iter().enumerate() {
            let durasi = match t.kepadatan.as_str() {
                "macet_parah" => 75,
                "macet" => 55,
                "padat" => 35,
                _ => 15,
            };
            items.push(RankingItem {
                rank: idx + 1,
                titik_id: t.id,
                nama_ruas: t.ruas_nama.clone(),
                kecamatan: t.kecamatan_nama.clone(),
                kepadatan: t.kepadatan.clone(),
                frekuensi: (15 - idx as i32).max(1),
                durasi_rata_rata_menit: durasi,
                panjang_antrean_meter: t.panjang_antrean_meter,
            });
            if items.len() >= limit {
                break;
            }
        }
        items
    }

    pub async fn get_tren_jam(&self) -> Vec<TrenItem> {
        vec![
            TrenItem {
                label: "06:00".to_string(),
                jumlah: 3,
                kepadatan_rata_rata: "padat".to_string(),
            },
            TrenItem {
                label: "07:00".to_string(),
                jumlah: 12,
                kepadatan_rata_rata: "macet_parah".to_string(),
            },
            TrenItem {
                label: "08:00".to_string(),
                jumlah: 14,
                kepadatan_rata_rata: "macet_parah".to_string(),
            },
            TrenItem {
                label: "09:00".to_string(),
                jumlah: 8,
                kepadatan_rata_rata: "macet".to_string(),
            },
            TrenItem {
                label: "10:00".to_string(),
                jumlah: 4,
                kepadatan_rata_rata: "padat".to_string(),
            },
            TrenItem {
                label: "11:00".to_string(),
                jumlah: 5,
                kepadatan_rata_rata: "padat".to_string(),
            },
            TrenItem {
                label: "12:00".to_string(),
                jumlah: 9,
                kepadatan_rata_rata: "macet".to_string(),
            },
            TrenItem {
                label: "13:00".to_string(),
                jumlah: 6,
                kepadatan_rata_rata: "padat".to_string(),
            },
            TrenItem {
                label: "14:00".to_string(),
                jumlah: 4,
                kepadatan_rata_rata: "padat".to_string(),
            },
            TrenItem {
                label: "15:00".to_string(),
                jumlah: 7,
                kepadatan_rata_rata: "macet".to_string(),
            },
            TrenItem {
                label: "16:00".to_string(),
                jumlah: 11,
                kepadatan_rata_rata: "macet_parah".to_string(),
            },
            TrenItem {
                label: "17:00".to_string(),
                jumlah: 15,
                kepadatan_rata_rata: "macet_parah".to_string(),
            },
            TrenItem {
                label: "18:00".to_string(),
                jumlah: 13,
                kepadatan_rata_rata: "macet_parah".to_string(),
            },
            TrenItem {
                label: "19:00".to_string(),
                jumlah: 7,
                kepadatan_rata_rata: "macet".to_string(),
            },
            TrenItem {
                label: "20:00".to_string(),
                jumlah: 4,
                kepadatan_rata_rata: "padat".to_string(),
            },
        ]
    }

    pub async fn get_sebaran_kecamatan(&self) -> Vec<(String, usize)> {
        let store = self.inner.read().await;
        let mut map = std::collections::HashMap::new();
        for t in &store.titik_macet {
            *map.entry(t.kecamatan_nama.clone()).or_insert(0) += 1;
        }
        let mut list: Vec<(String, usize)> = map.into_iter().collect();
        list.sort_by_key(|a| std::cmp::Reverse(a.1));
        list
    }
}

impl Store {
    fn seed_initial_data() -> Self {
        // Hashing demo passwords with Argon2
        let hash_admin = hash_password("admin123").unwrap_or_else(|_| "admin_hash".to_string());
        let hash_dishub = hash_password("dishub123").unwrap_or_else(|_| "dishub_hash".to_string());
        let hash_polisi = hash_password("polisi123").unwrap_or_else(|_| "polisi_hash".to_string());
        let hash_pimpinan =
            hash_password("pimpinan123").unwrap_or_else(|_| "pimpinan_hash".to_string());

        let users = vec![
            User {
                id: 1,
                nama: "Ir. Hendra Prasetyo (Admin Sistem)".to_string(),
                email: "admin@smarttraffic.bandarlampung.go.id".to_string(),
                username: "admin".to_string(),
                password_hash: hash_admin,
                peran_id: 1,
                peran_nama: "admin".to_string(),
                aktif: true,
                dibuat_pada: Utc::now(),
            },
            User {
                id: 2,
                nama: "Sari Wulandari, S.Tr.Tra (Petugas Dishub)".to_string(),
                email: "dishub@smarttraffic.bandarlampung.go.id".to_string(),
                username: "dishub".to_string(),
                password_hash: hash_dishub,
                peran_id: 2,
                peran_nama: "petugas_dishub".to_string(),
                aktif: true,
                dibuat_pada: Utc::now(),
            },
            User {
                id: 3,
                nama: "Iptu Andi Setiawan, S.I.K. (Satlantas Polresta)".to_string(),
                email: "polisi@smarttraffic.bandarlampung.go.id".to_string(),
                username: "polisi".to_string(),
                password_hash: hash_polisi,
                peran_id: 3,
                peran_nama: "polantas".to_string(),
                aktif: true,
                dibuat_pada: Utc::now(),
            },
            User {
                id: 4,
                nama: "Drs. H. Ahmad Fauzi, M.Si (Pimpinan Daerah)".to_string(),
                email: "pimpinan@smarttraffic.bandarlampung.go.id".to_string(),
                username: "pimpinan".to_string(),
                password_hash: hash_pimpinan,
                peran_id: 4,
                peran_nama: "pimpinan".to_string(),
                aktif: true,
                dibuat_pada: Utc::now(),
            },
        ];

        let kecamatan = vec![
            Kecamatan {
                id: 1,
                nama: "Kedaton".to_string(),
            },
            Kecamatan {
                id: 2,
                nama: "Way Halim".to_string(),
            },
            Kecamatan {
                id: 3,
                nama: "Tanjung Karang Pusat".to_string(),
            },
            Kecamatan {
                id: 4,
                nama: "Tanjung Karang Timur".to_string(),
            },
            Kecamatan {
                id: 5,
                nama: "Tanjung Karang Barat".to_string(),
            },
            Kecamatan {
                id: 6,
                nama: "Rajabasa".to_string(),
            },
            Kecamatan {
                id: 7,
                nama: "Sukarame".to_string(),
            },
            Kecamatan {
                id: 8,
                nama: "Sukabumi".to_string(),
            },
            Kecamatan {
                id: 9,
                nama: "Enggal".to_string(),
            },
            Kecamatan {
                id: 10,
                nama: "Teluk Betung Utara".to_string(),
            },
            Kecamatan {
                id: 11,
                nama: "Teluk Betung Selatan".to_string(),
            },
            Kecamatan {
                id: 12,
                nama: "Teluk Betung Barat".to_string(),
            },
            Kecamatan {
                id: 13,
                nama: "Bumi Waras".to_string(),
            },
            Kecamatan {
                id: 14,
                nama: "Kedamaian".to_string(),
            },
            Kecamatan {
                id: 15,
                nama: "Kemiling".to_string(),
            },
            Kecamatan {
                id: 16,
                nama: "Langkapura".to_string(),
            },
            Kecamatan {
                id: 17,
                nama: "Labuhan Ratu".to_string(),
            },
            Kecamatan {
                id: 18,
                nama: "Panjang".to_string(),
            },
            Kecamatan {
                id: 19,
                nama: "Tanjung Senang".to_string(),
            },
            Kecamatan {
                id: 20,
                nama: "Teluk Betung Timur".to_string(),
            },
        ];

        let penyebab_macet = vec![
            PenyebabMacet {
                id: 1,
                nama: "Volume Kendaraan Jam Sibuk".to_string(),
            },
            PenyebabMacet {
                id: 2,
                nama: "Penyempitan Jalan / Proyek".to_string(),
            },
            PenyebabMacet {
                id: 3,
                nama: "Parkir Liar & PKL".to_string(),
            },
            PenyebabMacet {
                id: 4,
                nama: "Kecelakaan Lalu Lintas".to_string(),
            },
            PenyebabMacet {
                id: 5,
                nama: "Kendaraan Mogok / Truk Rusak".to_string(),
            },
            PenyebabMacet {
                id: 6,
                nama: "Persilangan Kereta Api".to_string(),
            },
            PenyebabMacet {
                id: 7,
                nama: "Lampu Lalu Lintas / Traffic Light Error".to_string(),
            },
            PenyebabMacet {
                id: 8,
                nama: "Genangan Air Hujan".to_string(),
            },
        ];

        let ruas_jalan = vec![
            RuasJalan {
                id: 1,
                nama: "Jl. Teuku Umar (Kedaton - Pasar Koga)".to_string(),
                kecamatan_id: 1,
                kecamatan_nama: "Kedaton".to_string(),
            },
            RuasJalan {
                id: 2,
                nama: "Jl. Z.A. Pagar Alam (Simpang MBK)".to_string(),
                kecamatan_id: 1,
                kecamatan_nama: "Kedaton".to_string(),
            },
            RuasJalan {
                id: 3,
                nama: "Simpang Ki Maja - Jl. Sultan Agung".to_string(),
                kecamatan_id: 2,
                kecamatan_nama: "Way Halim".to_string(),
            },
            RuasJalan {
                id: 4,
                nama: "Jl. Raden Intan (Pos Bambu Kuning - Pasar Tengah)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
            },
            RuasJalan {
                id: 5,
                nama: "Jl. RA Kartini (Central Plaza - Horison)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
            },
            RuasJalan {
                id: 6,
                nama: "Jl. Pangeran Antasari (Simpang Kali Balau)".to_string(),
                kecamatan_id: 14,
                kecamatan_nama: "Kedamaian".to_string(),
            },
            RuasJalan {
                id: 7,
                nama: "Jl. Soekarno Hatta By Pass (Simpang Kalibalok)".to_string(),
                kecamatan_id: 7,
                kecamatan_nama: "Sukarame".to_string(),
            },
            RuasJalan {
                id: 8,
                nama: "Jl. Wolter Monginsidi (Pengajaran - Pemda)".to_string(),
                kecamatan_id: 10,
                kecamatan_nama: "Teluk Betung Utara".to_string(),
            },
            RuasJalan {
                id: 9,
                nama: "Jl. Sultan Agung (Jalur Dua Way Halim)".to_string(),
                kecamatan_id: 2,
                kecamatan_nama: "Way Halim".to_string(),
            },
            RuasJalan {
                id: 10,
                nama: "Jl. Z.A. Pagar Alam (Underpass Unila)".to_string(),
                kecamatan_id: 6,
                kecamatan_nama: "Rajabasa".to_string(),
            },
            RuasJalan {
                id: 11,
                nama: "Jl. Jenderal Sudirman (Tugu Adipura)".to_string(),
                kecamatan_id: 9,
                kecamatan_nama: "Enggal".to_string(),
            },
            RuasJalan {
                id: 12,
                nama: "Jl. Yos Sudarso (Simpang Garuntang)".to_string(),
                kecamatan_id: 13,
                kecamatan_nama: "Bumi Waras".to_string(),
            },
            RuasJalan {
                id: 13,
                nama: "Jl. Pramuka (Simpang Bintang Amin)".to_string(),
                kecamatan_id: 15,
                kecamatan_nama: "Kemiling".to_string(),
            },
            RuasJalan {
                id: 14,
                nama: "Jl. Pangeran Emir M. Noor (Simpang Palapa)".to_string(),
                kecamatan_id: 5,
                kecamatan_nama: "Tanjung Karang Barat".to_string(),
            },
            RuasJalan {
                id: 15,
                nama: "Jl. Brigjen Katamso (Simpur Center)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
            },
        ];

        let titik_macet = vec![
            TitikMacet {
                id: 1,
                ruas_id: 1,
                ruas_nama: "Jl. Teuku Umar (Kedaton - Pasar Koga)".to_string(),
                kecamatan_id: 1,
                kecamatan_nama: "Kedaton".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.3857,
                longitude: 105.2592,
                kepadatan: "macet_parah".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 450,
                kecepatan_rata_rata_kmh: 8,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Antrean padat keluar masuk Pasar Koga dan persimpangan Mall Boemi Kedaton.".to_string()),
                foto_url: Some("https://images.unsplash.com/photo-1568605117036-5fe5e7bab0b7?w=600&auto=format&fit=crop&q=60".to_string()),
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 2,
                ruas_id: 2,
                ruas_nama: "Jl. Z.A. Pagar Alam (Simpang MBK)".to_string(),
                kecamatan_id: 1,
                kecamatan_nama: "Kedaton".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.3789,
                longitude: 105.2536,
                kepadatan: "macet".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 350,
                kecepatan_rata_rata_kmh: 12,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Penyempitan di depan pintu masuk utama MBK dan angkot ngetem.".to_string()),
                foto_url: Some("https://images.unsplash.com/photo-1506521781263-d8422e82f27a?w=600&auto=format&fit=crop&q=60".to_string()),
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 3,
                ruas_id: 3,
                ruas_nama: "Simpang Ki Maja - Jl. Sultan Agung".to_string(),
                kecamatan_id: 2,
                kecamatan_nama: "Way Halim".to_string(),
                penyebab_id: Some(3),
                penyebab_nama: "Parkir Liar & PKL".to_string(),
                latitude: -5.3872,
                longitude: 105.2711,
                kepadatan: "padat".to_string(),
                status: "dilaporkan".to_string(),
                panjang_antrean_meter: 200,
                kecepatan_rata_rata_kmh: 22,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Aktivitas kuliner malam dan parkir on-street di sepanjang pertokoan Ki Maja.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 4,
                ruas_id: 4,
                ruas_nama: "Jl. Raden Intan (Pos Bambu Kuning - Pasar Tengah)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
                penyebab_id: Some(3),
                penyebab_nama: "Parkir Liar & PKL".to_string(),
                latitude: -5.4132,
                longitude: 105.2588,
                kepadatan: "macet_parah".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 500,
                kecepatan_rata_rata_kmh: 5,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Pusat belanja Bambu Kuning, penyeberang jalan ramai dan pedagang kaki lima.".to_string()),
                foto_url: Some("https://images.unsplash.com/photo-1544620347-c4fd4a3d5957?w=600&auto=format&fit=crop&q=60".to_string()),
                dibuat_oleh: 3,
                pembuat_nama: "Iptu Andi (Polantas)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 5,
                ruas_id: 5,
                ruas_nama: "Jl. RA Kartini (Central Plaza - Horison)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.4168,
                longitude: 105.2562,
                kepadatan: "macet".to_string(),
                status: "dilaporkan".to_string(),
                panjang_antrean_meter: 300,
                kecepatan_rata_rata_kmh: 14,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Akses menuju Mall Kartini padat kendaraan roda empat.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 6,
                ruas_id: 6,
                ruas_nama: "Jl. Pangeran Antasari (Simpang Kali Balau)".to_string(),
                kecamatan_id: 14,
                kecamatan_nama: "Kedamaian".to_string(),
                penyebab_id: Some(2),
                penyebab_nama: "Penyempitan Jalan / Proyek".to_string(),
                latitude: -5.4055,
                longitude: 105.2812,
                kepadatan: "padat".to_string(),
                status: "dilaporkan".to_string(),
                panjang_antrean_meter: 180,
                kecepatan_rata_rata_kmh: 25,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Arus keluar perumahan dan simpang pertigaan Kali Balau Kencana.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 7,
                ruas_id: 7,
                ruas_nama: "Jl. Soekarno Hatta By Pass (Simpang Kalibalok)".to_string(),
                kecamatan_id: 7,
                kecamatan_nama: "Sukarame".to_string(),
                penyebab_id: Some(5),
                penyebab_nama: "Kendaraan Mogok / Truk Rusak".to_string(),
                latitude: -5.3942,
                longitude: 105.2925,
                kepadatan: "macet".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 400,
                kecepatan_rata_rata_kmh: 10,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Truk muatan batu bara mengalami patah as di lajur kiri.".to_string()),
                foto_url: None,
                dibuat_oleh: 3,
                pembuat_nama: "Iptu Andi (Polantas)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 8,
                ruas_id: 8,
                ruas_nama: "Jl. Wolter Monginsidi (Pengajaran - Pemda)".to_string(),
                kecamatan_id: 10,
                kecamatan_nama: "Teluk Betung Utara".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.4285,
                longitude: 105.2541,
                kepadatan: "padat".to_string(),
                status: "selesai".to_string(),
                panjang_antrean_meter: 150,
                kecepatan_rata_rata_kmh: 28,
                waktu_mulai: Utc::now(),
                waktu_selesai: Some(Utc::now()),
                catatan: Some("Arus perkantoran Pemprov Lampung dan Pemkot saat jam kerja usai.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 9,
                ruas_id: 9,
                ruas_nama: "Jl. Sultan Agung (Jalur Dua Way Halim)".to_string(),
                kecamatan_id: 2,
                kecamatan_nama: "Way Halim".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.3821,
                longitude: 105.2685,
                kepadatan: "lancar".to_string(),
                status: "selesai".to_string(),
                panjang_antrean_meter: 50,
                kecepatan_rata_rata_kmh: 38,
                waktu_mulai: Utc::now(),
                waktu_selesai: Some(Utc::now()),
                catatan: Some("Arus lalu lintas lancar terkendali di jalur dua Way Halim.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 10,
                ruas_id: 10,
                ruas_nama: "Jl. Z.A. Pagar Alam (Underpass Unila)".to_string(),
                kecamatan_id: 6,
                kecamatan_nama: "Rajabasa".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.3644,
                longitude: 105.2443,
                kepadatan: "lancar".to_string(),
                status: "selesai".to_string(),
                panjang_antrean_meter: 40,
                kecepatan_rata_rata_kmh: 42,
                waktu_mulai: Utc::now(),
                waktu_selesai: Some(Utc::now()),
                catatan: Some("Underpass Unila kedua arah arus lancar tanpa hambatan.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 11,
                ruas_id: 11,
                ruas_nama: "Jl. Jenderal Sudirman (Tugu Adipura)".to_string(),
                kecamatan_id: 9,
                kecamatan_nama: "Enggal".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.4215,
                longitude: 105.2605,
                kepadatan: "padat".to_string(),
                status: "dilaporkan".to_string(),
                panjang_antrean_meter: 220,
                kecepatan_rata_rata_kmh: 20,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Bundaran Tugu Gajah Adipura padat pertemuan arah Sudirman dan Raden Intan.".to_string()),
                foto_url: None,
                dibuat_oleh: 3,
                pembuat_nama: "Iptu Andi (Polantas)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 12,
                ruas_id: 12,
                ruas_nama: "Jl. Yos Sudarso (Simpang Garuntang)".to_string(),
                kecamatan_id: 13,
                kecamatan_nama: "Bumi Waras".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.4412,
                longitude: 105.2745,
                kepadatan: "macet".to_string(),
                status: "dilaporkan".to_string(),
                panjang_antrean_meter: 280,
                kecepatan_rata_rata_kmh: 15,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Akses pergudangan pelabuhan dan kendaraan berat melintas bergantian.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 13,
                ruas_id: 13,
                ruas_nama: "Jl. Pramuka (Simpang Bintang Amin)".to_string(),
                kecamatan_id: 15,
                kecamatan_nama: "Kemiling".to_string(),
                penyebab_id: Some(2),
                penyebab_nama: "Penyempitan Jalan / Proyek".to_string(),
                latitude: -5.3795,
                longitude: 105.2289,
                kepadatan: "padat".to_string(),
                status: "selesai".to_string(),
                panjang_antrean_meter: 160,
                kecepatan_rata_rata_kmh: 26,
                waktu_mulai: Utc::now(),
                waktu_selesai: Some(Utc::now()),
                catatan: Some("Simpang RS Bintang Amin arus mahasiswa dan warga Kemiling.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 14,
                ruas_id: 14,
                ruas_nama: "Jl. Pangeran Emir M. Noor (Simpang Palapa)".to_string(),
                kecamatan_id: 5,
                kecamatan_nama: "Tanjung Karang Barat".to_string(),
                penyebab_id: Some(1),
                penyebab_nama: "Volume Kendaraan Jam Sibuk".to_string(),
                latitude: -5.4192,
                longitude: 105.2442,
                kepadatan: "lancar".to_string(),
                status: "selesai".to_string(),
                panjang_antrean_meter: 30,
                kecepatan_rata_rata_kmh: 40,
                waktu_mulai: Utc::now(),
                waktu_selesai: Some(Utc::now()),
                catatan: Some("Kawasan pemukiman Palapa terpantau tertib lancar.".to_string()),
                foto_url: None,
                dibuat_oleh: 2,
                pembuat_nama: "Sari Wulandari (Dishub)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
            TitikMacet {
                id: 15,
                ruas_id: 15,
                ruas_nama: "Jl. Brigjen Katamso (Simpur Center)".to_string(),
                kecamatan_id: 3,
                kecamatan_nama: "Tanjung Karang Pusat".to_string(),
                penyebab_id: Some(3),
                penyebab_nama: "Parkir Liar & PKL".to_string(),
                latitude: -5.4150,
                longitude: 105.2575,
                kepadatan: "macet_parah".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 380,
                kecepatan_rata_rata_kmh: 6,
                waktu_mulai: Utc::now(),
                waktu_selesai: None,
                catatan: Some("Kawasan ruko Simpur Center padat bongkar muat ekspedisi dan angkot.".to_string()),
                foto_url: None,
                dibuat_oleh: 3,
                pembuat_nama: "Iptu Andi (Polantas)".to_string(),
                dibuat_pada: Utc::now(),
                diubah_pada: Utc::now(),
            },
        ];

        let riwayat_kejadian = vec![
            RiwayatKejadian {
                id: 1,
                titik_id: 1,
                ruas_nama: "Jl. Teuku Umar (Kedaton - Pasar Koga)".to_string(),
                kepadatan: "macet_parah".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 450,
                kecepatan_rata_rata_kmh: 8,
                waktu_kejadian: Utc::now(),
                catatan: Some(
                    "Operator TMC mendeteksi antrean mencapai 450 meter di simpang Kedaton."
                        .to_string(),
                ),
                foto_url: None,
                dicatat_oleh: 2,
                pencatat_nama: "Sari Wulandari (Dishub)".to_string(),
            },
            RiwayatKejadian {
                id: 2,
                titik_id: 2,
                ruas_nama: "Jl. Z.A. Pagar Alam (Simpang MBK)".to_string(),
                kepadatan: "macet".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 350,
                kecepatan_rata_rata_kmh: 12,
                waktu_kejadian: Utc::now(),
                catatan: Some(
                    "Petugas Dishub diterjunkan untuk penguraian arus depan MBK.".to_string(),
                ),
                foto_url: None,
                dicatat_oleh: 2,
                pencatat_nama: "Sari Wulandari (Dishub)".to_string(),
            },
            RiwayatKejadian {
                id: 3,
                titik_id: 4,
                ruas_nama: "Jl. Raden Intan (Pos Bambu Kuning - Pasar Tengah)".to_string(),
                kepadatan: "macet_parah".to_string(),
                status: "ditangani".to_string(),
                panjang_antrean_meter: 500,
                kecepatan_rata_rata_kmh: 5,
                waktu_kejadian: Utc::now(),
                catatan: Some(
                    "Satlantas Polresta melaksanakan buka tutup jalur di Bambu Kuning.".to_string(),
                ),
                foto_url: None,
                dicatat_oleh: 3,
                pencatat_nama: "Iptu Andi (Polantas)".to_string(),
            },
        ];

        let tindakan_penanganan = vec![
            TindakanPenanganan {
                id: 1,
                titik_id: 1,
                ruas_nama: "Jl. Teuku Umar (Kedaton - Pasar Koga)".to_string(),
                jenis: "Pengaturan Manual".to_string(),
                deskripsi: Some("Penempatan 3 personel di Simpang Kedaton untuk memprioritaskan arus Jl. Teuku Umar.".to_string()),
                status_tindakan: "sedang_berjalan".to_string(),
                petugas_id: 2,
                petugas_nama: "Sari Wulandari (Dishub)".to_string(),
                waktu_tindakan: Utc::now(),
                waktu_selesai: None,
            },
            TindakanPenanganan {
                id: 2,
                titik_id: 2,
                ruas_nama: "Jl. Z.A. Pagar Alam (Simpang MBK)".to_string(),
                jenis: "Penertiban Parkir Liar".to_string(),
                deskripsi: Some("Mengimbau jukir liar depan MBK agar kendaraan tidak memakan badan jalan.".to_string()),
                status_tindakan: "sedang_berjalan".to_string(),
                petugas_id: 2,
                petugas_nama: "Sari Wulandari (Dishub)".to_string(),
                waktu_tindakan: Utc::now(),
                waktu_selesai: None,
            },
            TindakanPenanganan {
                id: 3,
                titik_id: 4,
                ruas_nama: "Jl. Raden Intan (Pos Bambu Kuning - Pasar Tengah)".to_string(),
                jenis: "Buka Tutup Jalur".to_string(),
                deskripsi: Some("Pemberlakuan satu arah bergantian di depan Pasar Bambu Kuning.".to_string()),
                status_tindakan: "sedang_berjalan".to_string(),
                petugas_id: 3,
                petugas_nama: "Iptu Andi (Polantas)".to_string(),
                waktu_tindakan: Utc::now(),
                waktu_selesai: None,
            },
        ];

        let notifikasi = vec![
            Notifikasi {
                id: 1,
                pengguna_id: None,
                peran_id: Some(3),
                judul: "Peringatan Kemacetan: Pos Bambu Kuning".to_string(),
                pesan: "Antrean 500 meter di Pos Bambu Kuning, kecepatan 5 km/h. Perlu penanganan segera.".to_string(),
                tipe: "darurat".to_string(),
                tautan: Some("/polisi/operasional".to_string()),
                dibaca: false,
                dibuat_pada: Utc::now(),
            },
            Notifikasi {
                id: 2,
                pengguna_id: None,
                peran_id: Some(2),
                judul: "Penugasan Penguraian Simpang MBK".to_string(),
                pesan: "Kepadatan tinggi di depan MBK Mall Boemi Kedaton, petugas Dishub shift 2 diarahkan ke lokasi.".to_string(),
                tipe: "peringatan".to_string(),
                tautan: Some("/dishub/pemantauan".to_string()),
                dibaca: false,
                dibuat_pada: Utc::now(),
            },
            Notifikasi {
                id: 3,
                pengguna_id: None,
                peran_id: Some(4),
                judul: "Laporan Harian Siap Diunduh".to_string(),
                pesan: "Ringkasan eksekutif pemantauan 15 koridor utama Kota Bandar Lampung telah diperbarui.".to_string(),
                tipe: "info".to_string(),
                tautan: Some("/pimpinan/dashboard".to_string()),
                dibaca: false,
                dibuat_pada: Utc::now(),
            },
        ];

        let log_audit = vec![
            LogAudit {
                id: 1,
                pengguna_id: Some(1),
                pengguna_nama: Some("Ir. Hendra Prasetyo".to_string()),
                peran_nama: Some("admin".to_string()),
                aksi: "Inisialisasi Sistem".to_string(),
                entitas: Some("Sistem".to_string()),
                detail: Some(
                    "Aplikasi SmartTraffic Bandar Lampung berhasil dijalankan".to_string(),
                ),
                ip_address: Some("127.0.0.1".to_string()),
                waktu: Utc::now(),
            },
            LogAudit {
                id: 2,
                pengguna_id: Some(2),
                pengguna_nama: Some("Sari Wulandari".to_string()),
                peran_nama: Some("petugas_dishub".to_string()),
                aksi: "Update Kemacetan".to_string(),
                entitas: Some("Titik Macet".to_string()),
                detail: Some("Memperbarui status Simpang MBK menjadi 'ditangani'".to_string()),
                ip_address: Some("192.168.1.45".to_string()),
                waktu: Utc::now(),
            },
        ];

        Store {
            users,
            kecamatan,
            ruas_jalan,
            penyebab_macet,
            titik_macet,
            riwayat_kejadian,
            tindakan_penanganan,
            notifikasi,
            log_audit,
            next_user_id: 5,
            next_titik_id: 16,
            next_riwayat_id: 4,
            next_tindakan_id: 4,
            next_notif_id: 4,
            next_audit_id: 3,
        }
    }
}
