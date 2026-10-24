# ⚠️ DATA PROYEK — BACA DULU

`projects.json` di folder ini adalah **satu-satunya sumber data** untuk grid, filter,
pencarian, dan modal. Tidak ada database, tidak ada API.

---

## Tambah proyek

Tambah satu object ke dalam array. Template:

```json
{
  "id": "nama-unik-proyek",
  "title": "Judul Proyek",
  "pitch": "Satu kalimat singkat, tampil di kartu.",
  "description": "Paragraf panjang, tampil di dalam modal.",
  "url": "https://subdomain.limun.web.id",
  "repo": "https://github.com/LimunBF/nama-repo",
  "stack": ["Rust", "Astro", "Svelte 5"],
  "categories": ["web", "tools"],
  "status": "online",
  "latency": 25,
  "accent": "#00F0FF",
  "size": "normal",
  "featured": false,
  "year": 2025,
  "architecture": ["Client", "API", "Database"],
  "features": ["Fitur pertama", "Fitur kedua"],
  "demo": null,
  "metrics": { "stars": 10, "requests": "1k/hari" }
},
{
    "id": "mini-shopify",
    "title": "Mini-Shopify SaaS",
    "pitch": "Multi-tenant storefront dengan checkout instan & dashboard penjual real-time.",
    "description": "Platform e-commerce multi-tenant yang meniru inti Shopify: katalog produk, keranjang persisten, checkout, dan dashboard analitik penjual. Setiap tenant mendapat storefront terisolasi dengan tema sendiri, sementara backend Golang menangani konkurensi checkout dan Redis menyimpan session + cart.",
    "url": "https://shopify.limun.web.id",
    "repo": "https://github.com/LimunBF/mini-shopify",
    "stack": ["React", "Golang", "Redis", "MySQL"],
    "categories": ["fullstack", "saas"],
    "status": "online",
    "latency": 24,
    "accent": "#00F0FF",
    "size": "large",
    "featured": true,
    "year": 2025,
    "architecture": ["React SPA", "Go Fiber API", "Redis Cache", "MySQL"],
    "features": [
      "Storefront multi-tenant dengan subdomain tema terpisah",
      "Checkout dengan optimistic UI dan idempotency key",
      "Dashboard penjual: revenue, AOV, dan produk terlaris",
      "Webhook order yang tahan retry & replay"
    ],
    "demo": { "user": "demo@shopify.test", "pass": "demo1234" },
    "metrics": { "stars": 128, "requests": "12.4k/hari" }
  }
```

## Arti field

| Field | Wajib | Keterangan |
| :--- | :--- | :--- |
| `id` | Ya | **Unik.** Key Svelte + identitas probe. Duplikat = kartu hilang. |
| `title` | Ya | Nama proyek. |
| `pitch` | Ya | Satu baris, tampil di kartu. |
| `description` | Ya | Teks panjang di modal. |
| `url` | Ya | **Dipakai untuk ping badge status.** |
| `repo` | Ya | Tombol "Source Code". |
| `stack` | Ya | Badge teknologi + sumber filter pill (otomatis). |
| `categories` | Ya | Tag `#` di modal. |
| `status` | Ya | `online` / `degraded` / `maintenance` / `offline`. |
| `latency` | Ya | Angka ms. `0` = tanpa sinyal. |
| `accent` | Ya | Hex warna aksen kartu. |
| `size` | Ya | `normal` (1 kolom) / `wide` / `large` (2 kolom). |
| `featured` | Ya | `true` = bonus skor pencarian. |
| `year` | Ya | Tampil di kaki kartu. |
| `architecture` | Ya | Array node → diagram `A → B → C`. |
| `features` | Ya | List di modal, boleh `[]`. |
| `demo` | Ya | `null` atau `{ "user": "...", "pass": "..." }`. |
| `metrics` | Ya | `{ "stars": n, "requests": "..." }`, keduanya opsional. |

## Hapus proyek

Hapus object-nya. Filter pill dihitung ulang otomatis.

---

## Soal `url` dan badge status

Badge "Live (24ms)" **bukan** dari field `latency` — browser benar-benar `fetch` ke `url`.

- URL hidup → latency terukur asli.
- URL mati → latency **disimulasikan** dari hash `id`, ditandai di tooltip
  (`estimasi (subdomain demo belum aktif)`).
- `status: "offline"` → selalu merah "Offline", tidak disimulasikan.

Begitu subdomain asli aktif, badge otomatis jadi nyata **tanpa ubah kode**.

---

## ⚠️ Jebakan nomor satu

**Object terakhir TIDAK boleh diakhiri koma.** JSON tidak mengizinkan trailing comma.
Gejalanya: `npm run build` gagal dengan `Unexpected token`.

Cek cepat:
```bash
node -e "JSON.parse(require('fs').readFileSync('src/data/projects.json','utf8'));console.log('JSON OK')"
```

**Kenapa tidak ada komentar di file JSON?** Format JSON tidak mendukung `//` atau `/* */`.
Menambahkannya langsung membuat build gagal.

---

## Di mana edit kalau mau ubah…

| Mau ubah | File |
| :--- | :--- |
| Daftar proyek | `src/data/projects.json` |
| Domain / canonical | `astro.config.mjs` (`site:`) |
| Judul header, link Source | `src/components/Header.astro` |
| Tampilan kartu | `src/components/ProjectCard.svelte` |
| Isi modal | `src/components/ProjectModal.svelte` |
| Filter, sort, pencarian | `src/components/ArcadeGrid.svelte` |
| Logika ping | `src/lib/healthMonitor.ts` |
| Pencarian (TS) | `src/lib/fallbackSearch.ts` |
| Pencarian (Rust) | `crates/portal-wasm/src/lib.rs` |
| Warna, font, animasi | `src/styles/global.css` |
| Footer | `src/layouts/Layout.astro` |

---

## ⚠️ Ubah algoritma pencarian = ubah DUA file

`src/lib/fallbackSearch.ts` (TypeScript) dan `crates/portal-wasm/src/lib.rs` (Rust)
harus punya bobot skor identik. Kalau beda, hasil di localhost ≠ hasil di production.
Lihat konstanta `WEIGHTS` di kedua file.

---

## Verifikasi

```bash
npm run dev     # cek visual
npm run check   # harus 0 error
npm run build   # build produksi
```
