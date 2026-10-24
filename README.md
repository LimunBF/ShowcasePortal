# 🕹️ DevArcade — Polyglot Project Showcase Hub

Portal *launchpad* bergaya arcade (terinspirasi Friv.com) untuk memamerkan proyek
fullstack yang berjalan di subdomain terpisah. Dibangun dengan **Astro 5 static output
+ Svelte 5 Islands + Tailwind CSS v4**, dengan mesin pencarian **Rust WebAssembly**
dan fallback TypeScript.

**Live:** [limun.web.id](https://limun.web.id)

----

## ✨ Fitur

- **Fuzzy search & filter** — dieksekusi di browser, tanpa satu pun request jaringan.
  Rust/WASM kalau tersedia, TypeScript kalau belum ter-compile. Hasilnya identik.
- **Live status badge** — setiap kartu melakukan probe ringan ke subdomain-nya dan
  menampilkan `● Live (24ms)` / `Degraded` / `Offline` secara dinamis.
- **Bento arcade grid** — 1 kolom di HP, 3 di tablet, 4 di desktop, dengan kartu
  `wide`/`large` untuk variasi layout.
- **Quick-preview modal** — diagram arsitektur, daftar fitur, akun demo, CTA Launch.
  Navigasi `←`/`→` antar proyek, tutup dengan `Esc`.
- **Command palette** — `Ctrl/Cmd + K` fokus ke kolom pencarian dari mana saja.
- **Zero server RAM** — output murni statis. Tidak ada Node.js di server.

---

## 🛠️ Tech Stack

| Layer | Teknologi | Fungsi |
| :--- | :--- | :--- |
| Core Framework | [Astro 5](https://astro.build/) (`output: 'static'`) | SSG, asset optimization |
| Interactive UI | [Svelte 5](https://svelte.dev/) (Runes) + [Tailwind v4](https://tailwindcss.com/) | Bento grid, modal, reactive state |
| Search (WASM) | Rust + `wasm-bindgen` | Fuzzy search, filter, scoring |
| Search (fallback) | TypeScript | Implementasi identik, untuk dev tanpa Rust |
| CI/CD | GitHub Actions | Compile WASM → build → deploy statis |
| Hosting | cPanel (Apache/LiteSpeed) | Static delivery |

---

## 🚀 Mulai (Development Lokal)

### Prasyarat

- **Node.js 20+**
- Rust **tidak perlu** — lihat [Zero-Local-Rust](#-zero-local-rust).

```bash
npm install
npm run dev      # http://localhost:4321
```

### Scripts

| Perintah | Fungsi |
| :--- | :--- |
| `npm run dev` | Dev server dengan HMR |
| `npm run build` | Build statis ke `dist/` |
| `npm run preview` | Preview hasil build |
| `npm run check` | Type & template check (harus 0 error) |
| `npm test` | Self-check algoritma pencarian TypeScript |

---

## 🔒 Zero-Local-Rust

Aturan arsitektur: **`cargo`, `rustup`, dan `wasm-pack` tidak dijalankan di mesin lokal**
(hemat disk, hemat waktu). Konsekuensinya:

- `src/lib/searchEngine.ts` mencoba `import('../wasm-pkg/portal_wasm.js')`.
- File itu **tidak ada** di lokal → import gagal → otomatis pakai fallback TypeScript.
- Tidak ada error yang bocor ke UI. Portal jalan normal di localhost.
- Kompilasi Rust terjadi **hanya di GitHub Actions**, hasilnya `src/wasm-pkg/` saat build.

Kalau `npm run dev` menampilkan `404 /src/wasm-pkg/portal_wasm.js` di log, itu
**normal dan diharapkan** — bukan bug.

---

## 📁 Struktur

```
/
├── crates/portal-wasm/          # Rust WASM crate (search + filter + scoring)
│   ├── src/lib.rs
│   └── Cargo.toml
├── src/
│   ├── components/
│   │   ├── ArcadeGrid.svelte    # Grid, filter, sort, pencarian (island utama)
│   │   ├── ProjectCard.svelte   # Satu kartu + live status badge
│   │   ├── ProjectModal.svelte  # Quick-preview modal
│   │   └── Header.astro         # Command bar (zero hydration)
│   ├── data/
│   │   ├── projects.json        # ⚠️ SEMUA data proyek di sini
│   │   └── README.md            # Petunjuk edit data
│   ├── layouts/Layout.astro
│   ├── lib/
│   │   ├── healthMonitor.ts     # Probe status subdomain
│   │   ├── searchEngine.ts      # Adapter WASM → TS fallback
│   │   ├── fallbackSearch.ts    # Algoritma pencarian TypeScript
│   │   ├── types.ts
│   │   └── fallbackSearch.test.ts
│   ├── pages/
│   │   ├── index.astro
│   │   └── 404.astro
│   └── styles/global.css        # Design tokens
├── public/.htaccess             # MIME wasm, cache, HTTPS redirect
└── .github/workflows/build-deploy.yml
```

---

## ➕ Menambah / Mengubah Proyek

Semua data ada di **`src/data/projects.json`**. Tambah satu object ke array, selesai.

Panduan lengkap (arti tiap field, jebakan JSON, di file mana harus edit apa):

**→ [`src/data/README.md`](src/data/README.md)**

Verifikasi cepat:

```bash
node -e "JSON.parse(require('fs').readFileSync('src/data/projects.json','utf8'));console.log('JSON OK')"
```

---

## 🎨 Desain

**Cyber-Arcade Brutalism** — dark mode, hairline border, aksen neon, tipografi
Space Grotesk + JetBrains Mono. Spesifikasi lengkap: [`DESIGN.md`](DESIGN.md).

| Token | Nilai |
| :--- | :--- |
| Background | `#090A0F` |
| Surface | `#12141D` |
| Border | `#232738` |
| Accent | `#FF5D01` (astro orange) / `#00F0FF` (cyber cyan) |
| Online / Warn / Offline | `#10B981` / `#F59E0B` / `#EF4444` |

---

## 🚢 Deploy

Build **tidak boleh** dijalankan di cPanel (risiko OOM kill, RAM 1024 MB shared).

1. Push ke `main` → GitHub Actions compile Rust → `npm run build` → upload artifact.
2. Isi folder `dist/` diunggah ke `public_html/`.
3. `public/.htaccess` ikut ter-copy ke `dist/` dan mengatur MIME `application/wasm`,
   kompresi, cache, serta redirect HTTPS.

Auto-deploy via FTPS opsional — set secrets `CPANEL_FTP_SERVER`, `CPANEL_FTP_USERNAME`,
`CPANEL_FTP_PASSWORD` di repo settings.

---

## 📄 Dokumen

| File | Isi |
| :--- | :--- |
| [`src/data/README.md`](src/data/README.md) | Panduan edit data proyek |
| [`DESIGN.md`](DESIGN.md) | Design system |

> Spesifikasi produk (PRD), technical requirements (TRD), dan aturan internal AI agent
> disimpan di **repo privat terpisah** dan tidak dipublikasikan di sini.

---

## Lisensi

MIT
