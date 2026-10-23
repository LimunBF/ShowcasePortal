# AI Coding Agent Guidelines (AGENTS.md)

Dokumen ini adalah panduan wajib bagi AI Coding Assistant (Cursor, Windsurf, Claude Code, GitHub Copilot) yang menulis kode di repositori ini.

## 1. Kondisi Lingkungan Lokal (PENTING!)
- **OS Pengguna:** Windows (Node.js v21.7.2, npm 10.5.0, Git 2.45.1).
- **Keterbatasan Disk (Zero-Local-Rust):** Pengguna **TIDAK** menginstal Rust (`cargo` / `rustc` / `wasm-pack`) di laptop lokal untuk menghemat penyimpanan Drive C:\.
- **Aturan Eksekusi Terminal Lokal:** 
  - **DILARANG** menjalankan perintah `cargo`, `rustup`, atau `wasm-pack` di terminal lokal pengguna.
  - Seluruh *development* lokal hanya menggunakan `npm install` dan `npm run dev`.

## 2. Arsitektur Hybrid WASM + JS Fallback (Agar Jalan di Lokal & Tetap Terbaca Rust di GitHub)
1. **Tulis Kode Rust Lengkap:** Tetap buat folder `crates/portal-wasm/` (`Cargo.toml` dan `src/lib.rs`) berisi algoritma pencarian (*fuzzy search*), *filtering*, dan kalkulasi skor proyek dalam bahasa Rust yang valid dan substansial (agar persentase bahasa Rust di GitHub tinggi).
2. **Smart Adapter di Frontend (`src/lib/searchEngine.ts`):**
   - Buat *wrapper* di TypeScript yang mencoba memuat modul WASM (`../wasm-pkg/portal_wasm.js`) secara dinamis.
   - Jika file WASM belum ter-compile (seperti saat jalan di localhost Windows pengguna), *wrapper* otomatis menggunakan *fallback* fungsi pencarian TypeScript murni tanpa memunculkan error.
3. **Kompilasi Rust Otomatis di Cloud (`.github/workflows/build-deploy.yml`):**
   - Buat workflow GitHub Actions yang otomatis menginstal Rust + `wasm-pack` di server GitHub setiap kali pengguna melakukan `git push`, mengompilasi `crates/portal-wasm`, lalu menjalankan `npm run build` untuk menghasilkan folder statis `dist/` yang siap diunggah ke cPanel.

## 3. Aturan Teknis Frontend
1. **Svelte 5 Runes Only:** Gunakan sintaks Svelte 5 terbaru (`$state`, `$derived`, `$effect`, `$props()`).
2. **Astro Static Output:** Pastikan `astro.config.mjs` menggunakan `output: 'static'` agar konsumsi RAM di cPanel adalah 0 MB.
3. **Desain Visual:** Ikuti palet warna dan konsep *Cyber-Arcade Brutalism* ala Friv.com sesuai dokumen `DESIGN.md`.

## 4. Urutan Eksekusi (Step-by-Step)
- **Tahap 1:** Inisiasi proyek Astro + Svelte 5 + Tailwind CSS di folder saat ini, lalu bangun UI *Bento Arcade Grid*, *Header Search Bar*, dan *Quick-Preview Modal* beserta `src/lib/searchEngine.ts` (dengan JS fallback).
- **Tahap 2:** Tulis *crate* Rust lengkap di `crates/portal-wasm/` dan buatkan `.github/workflows/build-deploy.yml` serta `.htaccess` untuk cPanel.