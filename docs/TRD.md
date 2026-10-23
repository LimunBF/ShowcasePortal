# Technical Requirements Document (TRD)

> **Portal Portofolio Multi-Subdomain** — Astro + Svelte 5 + Rust (WASM/Health Daemon)

## Daftar Isi
1. [Arsitektur Tech Stack](#1-arsitektur-tech-stack)
2. [Struktur Direktori Monorepo](#2-struktur-direktori-monorepo)
3. [Spesifikasi Target Hosting](#3-spesifikasi-target-hosting-griv-cloud-cpanel--expert-tier)
4. [Konfigurasi `.htaccess`](#4-konfigurasi-htaccess)
5. [Architecture Decision Records (ADR)](#5-architecture-decision-records-adr)

---

## 1. Arsitektur Tech Stack

| Layer | Teknologi | Keterangan |
|---|---|---|
| **Core Web Framework** | Astro 5.x (`output: 'static'`) | Static Site Generation (SSG) |
| **Interactive UI Islands** | Svelte 5 (Runes: `$state`, `$derived`, `$effect`) | Hidrasi selektif per komponen |
| **Styling & Icons** | Tailwind CSS + Lucide Icons | — |
| **Rust — Layer 1** | `crates/portal-wasm` (`wasm-pack`) | *Fuzzy search*, skor relevansi filter proyek, dan *parsing* metrik kesehatan subdomain di sisi *client* |
| **Rust — Layer 2 (Opsional)** | `crates/health-daemon` (Axum) | Microservice super ringan untuk cek *uptime* seluruh subdomain, disimpan di Redis[cite: 1] |

---

## 2. Struktur Direktori Monorepo

```text
/
├── crates/
│   ├── portal-wasm/              # Rust WASM crate (pencarian & filter cepat)
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   └── health-daemon/            # Rust Axum microservice (health-check API + Redis)
│       ├── src/main.rs
│       └── Cargo.toml
├── src/
│   ├── components/
│   │   ├── ArcadeGrid.svelte     # Komponen utama Bento Grid & Filter (Svelte 5)
│   │   ├── ProjectModal.svelte
│   │   └── Header.astro
│   ├── data/
│   │   └── projects.json         # Data statis daftar proyek & subdomain
│   ├── layouts/
│   │   └── Layout.astro
│   └── pages/
│       └── index.astro           # Entry point utama
├── public/
│   └── .htaccess                 # Konfigurasi routing & caching cPanel
├── astro.config.mjs
├── package.json
└── Cargo.toml                    # Rust Workspace root
```

---

## 3. Spesifikasi Target Hosting (GRIV Cloud cPanel — Expert Tier)

### 3.1 Environment
- **OS:** CloudLinux Shared Hosting
- **Akses:** Jailed SSH
- **Resource:** 1 Core CPU · 1024 MB RAM · 5 GB Storage

### 3.2 Aturan Build & Deployment

> ⚠️ **DILARANG** menjalankan `cargo build` atau `npm run build` di dalam server cPanel untuk menghindari **Out-of-Memory (OOM) Kill**.

1. Seluruh proses kompilasi dilakukan di **komputer lokal** atau melalui **GitHub Actions CI/CD**.
2. Hasil `npm run build` (folder `dist/`) diunggah langsung ke `/home/<username>/public_html/`.
3. Jika menggunakan `health-daemon` Rust, **cross-compile di lokal** menggunakan target `x86_64-unknown-linux-musl`, lalu jalankan di latar belakang.

---

## 4. Konfigurasi `.htaccess`

File `.htaccess` di `public_html` **wajib** menyertakan konfigurasi berikut:

```apache
AddType application/wasm .wasm
<IfModule mod_deflate.c>
  AddOutputFilterByType DEFLATE application/wasm text/html text/css application/javascript
</IfModule>
```

---

## 5. Architecture Decision Records (ADR)

### ADR-001: Penggunaan Subdomain dibanding Subpath untuk Proyek Portofolio
- **Status:** Disetujui
- **Konteks:** Hosting memiliki 1 domain utama tetapi menyediakan kuota hingga 25 subdomain.
- **Keputusan:** Setiap proyek ditempatkan di subdomain masing-masing (contoh: `shopify.domain.com`), sedangkan domain utama (`domain.com`) menjadi portal *selector*.
- **Alasan:**
  - Mengisolasi setiap proyek di foldernya sendiri di cPanel.
  - Menghindari konflik `.htaccess` dan *base path* pada React/Vite Router.
  - Mempermudah manajemen CORS dengan backend Golang.

### ADR-002: Astro (Static) + Svelte 5 dibanding Next.js / Full SSR
- **Status:** Disetujui
- **Konteks:** Batas RAM server adalah 1024 MB (berbagi untuk seluruh website dan database di dalam akun).
- **Keputusan:** Menggunakan Astro Static Site Generation (SSG) dengan *Islands Architecture* menggunakan Svelte 5.
- **Alasan:**
  - Output statis tidak membutuhkan proses *runtime* Node.js di server (RAM = 0 MB).
  - Kuota RAM 1024 MB dihemat sepenuhnya untuk proses binary Golang, Rust, dan database proyek utama.

### ADR-003: Integrasi Rust via WebAssembly (WASM) + Workspace
- **Status:** Disetujui
- **Konteks:** Pemilik proyek ingin repositori GitHub menampilkan penggunaan bahasa Rust yang nyata sekaligus menjaga kecepatan pembuatan UI dengan Astro + Svelte.
- **Keputusan:** Memindahkan logika *search indexing*, *filtering*, dan kalkulasi telemetri ke dalam crate Rust (`portal-wasm`) yang dipanggil oleh komponen Svelte.
- **Alasan:**
  - Memberikan pengalaman belajar integrasi Rust-ke-Frontend (WASM).
  - Meningkatkan persentase kode Rust di GitHub.
  - Tetap mempertahankan kemudahan desain UI di Svelte/Tailwind.

### ADR-004: Local / CI-CD Cross-Compilation
- **Status:** Disetujui
- **Konteks:** *Compiler* Rust (`rustc`) dan bundler JS membutuhkan CPU dan RAM tinggi saat kompilasi.
- **Keputusan:** Semua proses *build* dilakukan di luar cPanel (lokal atau GitHub Actions). Server cPanel hanya menerima file statis (`dist/`) dan *compiled binary*.
- **Alasan:** Menghindari OOM Kill pada resource shared hosting yang terbatas.