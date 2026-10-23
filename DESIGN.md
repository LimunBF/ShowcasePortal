# Design System Specification: "Cyber-Arcade Brutalism"

## 1. Filosofi Visual
Menggabungkan nostalgia *game launcher* padat ala Friv.com dengan estetika *developer tool* modern (seperti Linear, Vercel, dan Raycast). Tampilan berfokus pada mode gelap (*dark mode*), garis batas tegas (*hairline borders*), dan aksen warna neon yang tajam.

## 2. Palet Warna (CSS Variables / Tailwind)
- **Background Primary:** `#090A0F` (Deep Void Black)
- **Background Surface / Card:** `#12141D` (Obsidian Slate)
- **Background Surface Hover:** `#1A1D2B` (Elevated Slate)
- **Border Default:** `#232738` (1px solid)
- **Border Active / Hover:** `#FF5D01` (Astro/Rust Orange Glow) atau `#00F0FF` (Cyber Cyan)
- **Text Primary:** `#F3F4F6` (Crisp White)
- **Text Secondary / Muted:** `#9CA3AF` (Cool Gray)
- **Status Indicators:**
  - Online / Healthy: `#10B981` (Emerald Green + pulse animation)
  - Maintenance / Cold Start: `#F59E0B` (Amber)
  - Offline: `#EF4444` (Crimson)

## 3. Tipografi
- **Heading & Display:** `'Space Grotesk'` atau `'JetBrains Mono'` (Bold, tight tracking `-0.03em`).
- **Body Text:** `'Plus Jakarta Sans'` atau `'Inter'`.
- **Monospace (Badges, Latency, Metrics):** `'JetBrains Mono'`.

## 4. Tata Letak & Komponen Utama
### A. Top Command Bar
- Berisi Logo Portal di kiri, *Search Input* (`Cmd/Ctrl + K`) di tengah, dan filter *Tech Stack Pills* (`All`, `Golang`, `React`, `Rust`, `Fullstack`) di kanan.

### B. Arcade Project Tile (Svelte Island)
- Rasio kartu fleksibel di dalam *Bento Grid* (responsif: 1 kolom di HP, 3 kolom di Tablet, 4-5 kolom di Desktop).
- **Bagian Atas Kartu:** Thumbnail/ikon visual proyek + *Live Ping Badge* di pojok kanan atas (contoh: `● 24ms`).
- **Bagian Bawah Kartu:** Judul proyek, *1-line pitch*, dan deretan *icon badge* teknologi.
- **Interaksi:** Saat *hover*, kartu sedikit terangkat (`translate-y-[-4px]`), *border* menyala sesuai warna aksen proyek, dan muncul tombol cepat `Launch ↗`.

### C. Game-Launcher Modal (Quick View)
- Muncul dengan *backdrop blur* (`backdrop-blur-md bg-black/70`).
- Menampilkan diagram singkat arsitektur proyek (misal: `React SPA -> Go API -> Redis`), daftar fitur yang bisa dicoba pengunjung, akun *demo login* (jika ada), serta tombol CTA utama berukuran besar.