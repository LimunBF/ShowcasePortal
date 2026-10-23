# ⚠️ FILE DATA PROYEK

File di folder ini (`projects.json`) adalah **satu-satunya sumber data** untuk grid,
filter, pencarian, dan modal di portal.

**Sebelum mengedit, baca dulu:** [`docs/EDITING-PROJECTS.md`](../../docs/EDITING-PROJECTS.md)

## Ringkasan 30 detik

- **Tambah proyek** → tambah satu object `{ ... }` di dalam array.
- **Hapus proyek** → hapus object-nya.
- **Object terakhir TIDAK boleh pakai koma** — ini penyebab error nomor satu.
- Field `url` dipakai untuk **ping status**, bukan `latency`.

## Cek cepat tanpa build

```bash
node -e "JSON.parse(require('fs').readFileSync('src/data/projects.json','utf8'));console.log('JSON OK')"
```

## Kenapa tidak ada komentar di dalam JSON?

Format JSON tidak mendukung komentar (`//` atau `/* */`). Menambahkannya akan membuat
`npm run build` gagal. Karena itu dokumentasi ditaruh di file terpisah ini.
