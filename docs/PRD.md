# Product Requirements Document (PRD): DevArcade Showcase Portal

## 1. Visi Produk
DevArcade adalah portal web interaktif bergaya arcade (terinspirasi dari *Friv.com*) yang berfungsi sebagai *central hub* untuk memamerkan berbagai proyek aplikasi web mandiri milik pengguna di bawah satu domain utama. Setiap "game tile" di dalam portal mewakili satu proyek *live* yang di-hosting di subdomain terpisah (misalnya: `shopify.domain.com`).

## 2. Target Audiens
- **Tech Recruiter & Hiring Manager:** Ingin melihat bukti nyata kemampuan *coding*, variasi *tech stack*, dan aplikasi yang langsung bisa dicoba tanpa harus melakukan *setup* lokal.
- **Sesama Engineer / Klien:** Mengeksplorasi arsitektur sistem, melihat status *live server*, dan mengakses repositori kode sumber.

## 3. Fitur Utama (Core Features)
1. **Interactive Bento/Arcade Grid:**
   - Menampilkan semua proyek dalam bentuk *grid* kartu visual yang padat dan interaktif.
   - Efek *hover* menampilkan animasi mikro, ringkasan fitur, dan *badge* teknologi (Rust, Go, React, Svelte, Redis).
2. **WASM-Powered Search & Filtering Engine (Rust):**
   - Pencarian instan (*fuzzy search*) dan pemfilteran proyek berdasarkan kategori atau *tech stack* yang dieksekusi langsung di *browser* menggunakan modul WebAssembly (Rust).
3. **Real-Time Subdomain Health Monitor:**
   - Indikator lampu status (*Live*, *Degraded*, *Offline*) dan pengukur *latency* (`ms`) pada tiap kartu proyek yang melakukan *ping* ke masing-masing subdomain.
4. **Quick-Preview Drawer / Modal:**
   - Saat kartu diklik, muncul *modal* bergaya *game launcher* yang menampilkan deskripsi arsitektur, tangkapan layar/GIF, status server, tombol **"Launch App"** (buka subdomain), dan **"Source Code"** (buka GitHub).
5. **Resource & System Telemetry Bar:**
   - Baris status di bagian bawah (*footer*) yang menampilkan metrik ringan dari server cPanel atau informasi *runtime* WASM.

## 4. Kebutuhan Non-Fungsional (Non-Functional Requirements)
- **Zero Server RAM Overhead untuk Frontend:** Halaman utama harus di-*build* menjadi aset statis murni (HTML/CSS/JS/WASM) agar tidak memakan kuota RAM 1024 MB di cPanel Shared Hosting.
- **Performa Tinggi:** Skor Google Lighthouse minimal 95+ untuk Performance, Accessibility, dan Best Practices.
- **GitHub Language Breakdown:** Struktur proyek wajib menyertakan kode Rust yang signifikan (WASM crate + optional Axum telemetry binary) agar terdeteksi di statistik bahasa GitHub.