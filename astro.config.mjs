// @ts-check
import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import tailwindcss from '@tailwindcss/vite';

// Zero-RAM static output: cPanel hanya menerima file statis (lihat TRD ADR-002).
export default defineConfig({
  output: 'static',
  site: 'https://limun.web.id',
  integrations: [svelte()],
  vite: {
    plugins: [tailwindcss()],
  },
  build: {
    // Inline stylesheet kecil, sisanya jadi file terpisah agar cache-friendly.
    inlineStylesheets: 'auto',
  },
});
