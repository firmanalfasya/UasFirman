-- ==============================================================================
-- Migrasi 2: Data Awal (Seed Data) SmartTraffic Bandar Lampung
-- ==============================================================================

-- 1. Seed Peran Sistem
INSERT INTO peran (id, nama) VALUES
(1, 'admin'),
(2, 'petugas_dishub'),
(3, 'polantas'),
(4, 'pimpinan')
ON CONFLICT (id) DO NOTHING;

-- 2. Seed Kecamatan Kota Bandar Lampung
INSERT INTO kecamatan (id, nama) VALUES
(1, 'Kedaton'),
(2, 'Way Halim'),
(3, 'Tanjung Karang Pusat'),
(4, 'Tanjung Karang Timur'),
(5, 'Tanjung Karang Barat'),
(6, 'Rajabasa'),
(7, 'Sukarame'),
(8, 'Sukabumi'),
(9, 'Enggal'),
(10, 'Teluk Betung Utara'),
(11, 'Teluk Betung Selatan'),
(12, 'Teluk Betung Barat'),
(13, 'Bumi Waras'),
(14, 'Kedamaian'),
(15, 'Kemiling'),
(16, 'Langkapura'),
(17, 'Labuhan Ratu'),
(18, 'Panjang'),
(19, 'Tanjung Senang'),
(20, 'Teluk Betung Timur')
ON CONFLICT (id) DO NOTHING;

-- 3. Seed Penyebab Kemacetan
INSERT INTO penyebab_macet (id, nama) VALUES
(1, 'Volume Kendaraan Jam Sibuk'),
(2, 'Penyempitan Jalan / Proyek'),
(3, 'Parkir Liar & PKL'),
(4, 'Kecelakaan Lalu Lintas'),
(5, 'Kendaraan Mogok / Truk Rusak'),
(6, 'Persilangan Kereta Api'),
(7, 'Lampu Lalu Lintas / Traffic Light Error'),
(8, 'Genangan Air Hujan')
ON CONFLICT (id) DO NOTHING;

-- 4. Seed Ruas Jalan Utama
INSERT INTO ruas_jalan (id, nama, kecamatan_id) VALUES
(1, 'Jl. Teuku Umar (Kedaton - Pasar Koga)', 1),
(2, 'Jl. Z.A. Pagar Alam (Simpang MBK Mall Boemi Kedaton)', 1),
(3, 'Simpang Ki Maja - Jl. Sultan Agung', 2),
(4, 'Jl. Raden Intan (Pos Bambu Kuning - Pasar Tengah)', 3),
(5, 'Jl. RA Kartini (Central Plaza - Horison)', 3),
(6, 'Jl. Pangeran Antasari (Simpang Kali Balau Kencana)', 14),
(7, 'Jl. Soekarno Hatta By Pass (Simpang Kalibalok)', 7),
(8, 'Jl. Wolter Monginsidi (Pengajaran - Simpang Pemda)', 10),
(9, 'Jl. Sultan Agung (Jalur Dua Way Halim)', 2),
(10, 'Jl. Z.A. Pagar Alam (Underpass Unila - Rajabasa)', 6),
(11, 'Jl. Jenderal Sudirman (Tugu Adipura / Bundaran Gajah)', 9),
(12, 'Jl. Yos Sudarso (Simpang Garuntang)', 13),
(13, 'Jl. Pramuka (Simpang Bintang Amin Kemiling)', 15),
(14, 'Jl. Pangeran Emir M. Noor (Simpang Palapa)', 5),
(15, 'Jl. Brigjen Katamso (Simpur Center)', 3)
ON CONFLICT (id) DO NOTHING;

-- 5. Seed 4 Akun Demo Pengguna
-- Catatan: Password default di-hash dengan Argon2id (admin123, dishub123, polisi123, pimpinan123)
INSERT INTO pengguna (id, nama, email, username, password_hash, peran_id, aktif) VALUES
(1, 'Ir. Hendra Prasetyo (Admin Sistem)', 'admin@smarttraffic.bandarlampung.go.id', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$MmdhZGVtb3NhbHRzbWFydA$Pq7wL1nU3GqQYQY5N1n2k3m4n5o6p7q8r9s0t1u2v3w', 1, TRUE),
(2, 'Sari Wulandari, S.Tr.Tra (Petugas Dishub)', 'dishub@smarttraffic.bandarlampung.go.id', 'dishub', '$argon2id$v=19$m=19456,t=2,p=1$MmdhZGVtb3NhbHRzbWFydA$Pq7wL1nU3GqQYQY5N1n2k3m4n5o6p7q8r9s0t1u2v3w', 2, TRUE),
(3, 'Iptu Andi Setiawan, S.I.K. (Satlantas Polresta)', 'polisi@smarttraffic.bandarlampung.go.id', 'polisi', '$argon2id$v=19$m=19456,t=2,p=1$MmdhZGVtb3NhbHRzbWFydA$Pq7wL1nU3GqQYQY5N1n2k3m4n5o6p7q8r9s0t1u2v3w', 3, TRUE),
(4, 'Drs. H. Ahmad Fauzi, M.Si (Kepala Dinas / Pimpinan)', 'pimpinan@smarttraffic.bandarlampung.go.id', 'pimpinan', '$argon2id$v=19$m=19456,t=2,p=1$MmdhZGVtb3NhbHRzbWFydA$Pq7wL1nU3GqQYQY5N1n2k3m4n5o6p7q8r9s0t1u2v3w', 4, TRUE)
ON CONFLICT (id) DO NOTHING;

-- 6. Seed 15 Titik Macet Nyata Bandar Lampung
INSERT INTO titik_macet (id, ruas_id, penyebab_id, latitude, longitude, kepadatan, status, panjang_antrean_meter, kecepatan_rata_rata_kmh, catatan, dibuat_oleh) VALUES
(1, 1, 1, -5.3857, 105.2592, 'macet_parah', 'ditangani', 450, 8, 'Antrean padat keluar masuk Pasar Koga dan persimpangan Mall Boemi Kedaton.', 2),
(2, 2, 1, -5.3789, 105.2536, 'macet', 'ditangani', 350, 12, 'Penyempitan di depan pintu masuk utama MBK dan angkot berhenti sembarangan.', 2),
(3, 3, 3, -5.3872, 105.2711, 'padat', 'dilaporkan', 200, 22, 'Aktivitas kuliner malam dan parkir on-street di sepanjang pertokoan Ki Maja.', 2),
(4, 4, 3, -5.4132, 105.2588, 'macet_parah', 'ditangani', 500, 5, 'Pusat belanja Bambu Kuning, penyeberang jalan ramai dan pedagang kaki lima.', 3),
(5, 5, 1, -5.4168, 105.2562, 'macet', 'dilaporkan', 300, 14, 'Akses menuju Mall Kartini dan Central Plaza padat kendaraan roda empat.', 2),
(6, 6, 2, -5.4055, 105.2812, 'padat', 'dilaporkan', 180, 25, 'Arus keluar perumahan dan simpang pertigaan Kali Balau Kencana.', 2),
(7, 7, 5, -5.3942, 105.2925, 'macet', 'ditangani', 400, 10, 'Truk muatan batu bara mengalami patah as roda di lajur kiri bypass.', 3),
(8, 8, 1, -5.4285, 105.2541, 'padat', 'selesai', 150, 28, 'Arus perkantoran Pemprov Lampung dan Pemkot Bandar Lampung saat jam pulang.', 2),
(9, 9, 1, -5.3821, 105.2685, 'lancar', 'selesai', 50, 38, 'Arus lalu lintas lancar terkendali di jalur dua Way Halim.', 2),
(10, 10, 1, -5.3644, 105.2443, 'lancar', 'selesai', 40, 42, 'Underpass Unila arus bebas hambatan kedua arah.', 2),
(11, 11, 1, -5.4215, 105.2605, 'padat', 'dilaporkan', 220, 20, 'Bundaran Tugu Gajah Adipura padat pertemuan dari arah Sudirman dan Raden Intan.', 3),
(12, 12, 1, -5.4412, 105.2745, 'macet', 'dilaporkan', 280, 15, 'Akses pergudangan pelabuhan dan kendaraan berat melintas.', 2),
(13, 13, 2, -5.3795, 105.2289, 'padat', 'selesai', 160, 26, 'Simpang RS Bintang Amin arus mahasiswa dan warga Kemiling.', 2),
(14, 14, 1, -5.4192, 105.2442, 'lancar', 'selesai', 30, 40, 'Kawasan pemukiman Palapa terpantau tertib lancar.', 2),
(15, 15, 3, -5.4150, 105.2575, 'macet_parah', 'ditangani', 380, 6, 'Kawasan ruko Simpur Center padat bongkar muat barang ekspedisi.', 3)
ON CONFLICT (id) DO NOTHING;

-- 7. Seed Riwayat Kejadian Awal
INSERT INTO riwayat_kejadian (titik_id, kepadatan, status, panjang_antrean_meter, kecepatan_rata_rata_kmh, catatan, dicatat_oleh) VALUES
(1, 'macet_parah', 'ditangani', 450, 8, 'Operator TMC mendeteksi antrean mencapai 450 meter di simpang Kedaton.', 2),
(2, 'macet', 'ditangani', 350, 12, 'Petugas Dishub diterjunkan untuk penguraian arus depan MBK.', 2),
(4, 'macet_parah', 'ditangani', 500, 5, 'Satlantas Polresta melaksanakan buka tutup jalur di Bambu Kuning.', 3),
(7, 'macet', 'ditangani', 400, 10, 'Truk mogok dalam penanganan mobil derek Dishub.', 3);

-- 8. Seed Penugasan Petugas Lapangan
INSERT INTO tindakan_penanganan (titik_id, jenis, deskripsi, status_tindakan, petugas_id) VALUES
(1, 'Pengaturan Manual', 'Penempatan 3 personel di Simpang Kedaton untuk memprioritaskan arus Jl. Teuku Umar.', 'sedang_berjalan', 2),
(2, 'Penertiban Parkir Liar', 'Mengimbau jukir liar depan MBK agar kendaraan tidak memakan badan jalan.', 'sedang_berjalan', 2),
(4, 'Buka Tutup Jalur', 'Pemberlakuan satu arah bergantian di depan Pasar Bambu Kuning.', 'sedang_berjalan', 3),
(7, 'Evakuasi Kendaraan Mogok', 'Mobil derek TMC diterjunkan untuk mengevakuasi truk di bypass Kalibalok.', 'sedang_berjalan', 3);

-- 9. Seed Notifikasi Awal
INSERT INTO notifikasi (peran_id, judul, pesan, tipe, tautan) VALUES
(3, 'Peringatan Kemacetan Berat: Bambu Kuning', 'Terjadi antrean 500 meter di Pos Bambu Kuning kecepatan rerata 5 km/h. Perlu penanganan segera.', 'darurat', '/polisi/operasional'),
(2, 'Penugasan Penguraian Simpang MBK', 'Kepadatan tinggi di depan MBK Mall Boemi Kedaton, petugas Dishub shift 2 diarahkan ke lokasi.', 'peringatan', '/dishub/pemantauan'),
(4, 'Laporan Harian Tersedia', 'Ringkasan eksekutif pemantauan 15 koridor utama Kota Bandar Lampung telah diperbarui.', 'info', '/pimpinan/dashboard');

