# 📝 CARA MENAMBAH & MENGUBAH PROYEK

Panduan singkat untuk kamu (atau AI agent lain) yang mau mengedit isi portal ini.

---

## Di mana datanya?

**Satu file, satu sumber kebenaran:**

```
src/data/projects.json
```

Grid, filter pill, pencarian, dan modal **semuanya** membaca dari file ini. Tidak ada
data lain yang perlu disentuh. Tidak ada database, tidak ada API.

---

## Menambah proyek baru

Buka `src/data/projects.json`, tambahkan satu object di dalam array `[...]`.
Salin template ini dan isi:

```json
{
  "id": "nama-unik-proyek",
  "title": "Judul Proyek",
  "pitch": "Satu kalimat singkat yang muncul di kartu.",
  "description": "Paragraf lebih panjang, muncul di dalam modal.",
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
  "features": [
    "Fitur pertama yang bisa dicoba pengunjung",
    "Fitur kedua"
  ],
  "demo": null,
  "metrics": { "stars": 10, "requests": "1k/hari" }
}
```

### Arti setiap field

| Field | Wajib? | Keterangan |
| :--- | :--- | :--- |
| `id` | **Ya** | Unik. Dipakai sebagai key Svelte + identitas probe. Jangan ada yang sama. |
| `title` | **Ya** | Nama proyek di kartu dan modal. |
| `pitch` | **Ya** | Satu baris, tampil di kartu. Kalau kepanjangan, otomatis dipotong 2 baris. |
| `description` | **Ya** | Muncul di modal. Boleh panjang. |
| `url` | **Ya** | **Dipakai ping untuk badge status.** Lihat bagian di bawah. |
| `repo` | **Ya** | Tombol "Source Code" di modal. |
| `stack` | **Ya** | Deretan badge teknologi + sumber filter pill. |
| `categories` | **Ya** | Tag, tampil di modal dengan `#`. |
| `status` | **Ya** | `online` / `degraded` / `maintenance` / `offline`. Lihat catatan. |
| `latency` | **Ya** | Angka ms. `0` = tidak ada sinyal (dianggap offline saat sort). |
| `accent` | **Ya** | Warna hex aksen kartu (border hover, tombol Launch, ikon). |
| `size` | **Ya** | `normal` (1 kolom), `wide` (2 kolom), `large` (2 kolom, lebih tinggi). |
| `featured` | **Ya** | `true` = dapat bonus skor di pencarian. |
| `year` | **Ya** | Tampil di kaki kartu kalau sedang tidak ada query. |
| `architecture` | **Ya** | Array node, dirender jadi diagram `A → B → C` di modal. |
| `features` | **Ya** | List di modal. Kosongkan `[]` kalau tidak ada. |
| `demo` | **Ya** | `null`, atau `{ "user": "...", "pass": "..." }` untuk akun demo. |
| `metrics` | **Ya** | `{ "stars": 10, "requests": "1k/hari" }`. Keduanya opsional di dalamnya. |

---

## Menghapus proyek

Hapus object-nya dari array. Selesai. Filter pill dihitung otomatis dari `stack`
yang tersisa, jadi pill yang tidak terpakai ikut hilang sendiri.

---

## Soal `url` dan badge status

Badge "Live (24ms)" di pojok kanan atas kartu **bukan** angka dari `latency`.
Browser benar-benar melakukan `fetch` ke `url` saat kartu dimuat.

- **URL hidup** → latency terukur asli, tooltip: `24ms · terukur langsung dari browser`.
- **URL mati / belum ada** → latency **disimulasikan** dari hash `id`, tooltip:
  `24ms · estimasi (subdomain demo belum aktif)`.
- **`status: "offline"`** → selalu tampil merah "Offline", tidak disimulasikan.

Simulasi ada supaya portal tidak terlihat rusak saat subdomain belum dipasang.
Begitu subdomain asli aktif, badge otomatis jadi nyata — **tanpa ubah kode apa pun**.

---

## Di mana harus edit kalau…

| Mau ubah | File |
| :--- | :--- |
| Daftar proyek | `src/data/projects.json` ← **paling sering** |
| Nama domain / canonical | `astro.config.mjs` (`site:`) |
| Logo, judul header, link Source | `src/components/Header.astro` |
| Tampilan kartu | `src/components/ProjectCard.svelte` |
| Isi modal | `src/components/ProjectModal.svelte` |
| Filter, sort, pencarian | `src/components/ArcadeGrid.svelte` |
| Logika ping | `src/lib/healthMonitor.ts` |
| Algoritma pencarian (TS) | `src/lib/fallbackSearch.ts` |
| Algoritma pencarian (Rust) | `crates/portal-wasm/src/lib.rs` |
| Warna, font, animasi | `src/styles/global.css` |
| Footer | `src/layouts/Layout.astro` |

---

## ⚠️ Dua jebakan yang mudah bikin error

**1. Koma di JSON.** Object terakhir di array **tidak boleh** diakhiri koma.
Kalau `npm run build` gagal dengan `Unexpected token`, hampir pasti ini penyebabnya.

Cek cepat tanpa build:
```bash
node -e "JSON.parse(require('fs').readFileSync('src/data/projects.json','utf8'));console.log('OK')"
```

**2. Kalau mengubah algoritma pencarian, ubah DUA tempat.**
`src/lib/fallbackSearch.ts` (TypeScript) dan `crates/portal-wasm/src/lib.rs` (Rust)
harus punya bobot skor yang sama. Kalau berbeda, hasil pencarian di localhost akan
beda dengan hasil di production (yang pakai WASM). Lihat komentar `WEIGHTS` di kedua file.

---

## Verifikasi setelah edit

```bash
npm run dev     # cek visual di http://localhost:4321
npm run check   # type check, harus 0 error
npm run build   # build produksi
```
