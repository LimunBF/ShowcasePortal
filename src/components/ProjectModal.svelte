<!--
  ProjectModal — game-launcher quick view (DESIGN.md §4.C).

  Runes: $props / $state / $derived / $effect. Handles focus trapping, ESC,
  background scroll lock and ←/→ navigation between projects so the
  launcher feels like the arcade cabinet it imitates.

  The parent owns the selection: this component is fully controlled and only
  reports intent through `onClose` / `onSelect`.
-->
<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import type { Project } from '../lib/types';

  interface Props {
    selected: Project | null;
    /** Reordered project list (matches the current grid order) for ←/→ nav. */
    projects: Project[];
    onClose: () => void;
    onSelect: (project: Project) => void;
  }

  const { selected, projects, onClose, onSelect } = $props<Props>();

  let dialogEl = $state<HTMLDivElement | null>(null);
  let copyState = $state<'idle' | 'copied' | 'failed'>('idle');
  let closeButton = $state<HTMLButtonElement | null>(null);

  const statusLabel: Record<Project['status'], { text: string; tone: string; dot: string }> = {
    online: { text: 'Online', tone: 'text-online border-online/40 bg-online/10', dot: 'bg-online' },
    degraded: { text: 'Degraded', tone: 'text-amber-warn border-amber-warn/40 bg-amber-warn/10', dot: 'bg-amber-warn' },
    maintenance: { text: 'Cold start', tone: 'text-amber-warn border-amber-warn/40 bg-amber-warn/10', dot: 'bg-amber-warn' },
    offline: { text: 'Offline', tone: 'text-crimson border-crimson/40 bg-crimson/10', dot: 'bg-crimson' },
  };

  const index = $derived(selected ? projects.findIndex((p) => p.id === selected.id) : -1);

  // Scroll lock + focus management, tied to `selected`.
  $effect(() => {
    if (!selected) return;

    const { body } = document;
    const previousOverflow = body.style.overflow;
    body.style.overflow = 'hidden';

    // Focus the primary CTA after the dialog paints.
    const raf = requestAnimationFrame(() => closeButton?.focus());

    return () => {
      body.style.overflow = previousOverflow;
      cancelAnimationFrame(raf);
    };
  });

  // Reset transient copy state whenever the modal switches project.
  $effect(() => {
    void selected?.id;
    copyState = 'idle';
  });

  // Keyboard: ESC to close, ←/→ to walk the arcade.
  $effect(() => {
    if (!selected) return;

    const onKeydown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onClose();
        return;
      }
      if (projects.length < 2) return;

      const current = projects.findIndex((p) => p.id === selected?.id);
      if (current === -1) return;

      if (event.key === 'ArrowRight') {
        event.preventDefault();
        onSelect(projects[(current + 1) % projects.length]);
      } else if (event.key === 'ArrowLeft') {
        event.preventDefault();
        onSelect(projects[(current - 1 + projects.length) % projects.length]);
      }
    };

    window.addEventListener('keydown', onKeydown);
    return () => window.removeEventListener('keydown', onKeydown);
  });

  /** Step through the current result set (footer ← / → buttons). */
  function step(delta: number) {
    if (projects.length === 0) return;
    const current = index === -1 ? 0 : index;
    const target = projects[(current + delta + projects.length) % projects.length];
    if (target) onSelect(target);
  }

  async function copyDemo() {
    if (!selected?.demo) return;
    const text = `${selected.demo.user} / ${selected.demo.pass}`;
    try {
      await navigator.clipboard.writeText(text);
      copyState = 'copied';
    } catch {
      copyState = 'failed';
    }
  }

  function onBackdrop(event: MouseEvent) {
    if (event.target === event.currentTarget) onClose();
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== 'Tab' || !dialogEl) return;
    const focusables = dialogEl.querySelectorAll<HTMLElement>(
      'a[href], button:not([disabled]), input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    if (focusables.length === 0) return;
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

{#if selected}
  {@const meta = statusLabel[selected.status]}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div
    class="fixed inset-0 z-50 flex items-end justify-center bg-black/70 p-0 backdrop-blur-md sm:items-center sm:p-6"
    role="presentation"
    onclick={onBackdrop}
    transition:fade={{ duration: 140 }}
  >
    <div
      bind:this={dialogEl}
      role="dialog"
      aria-modal="true"
      aria-labelledby="modal-title"
      aria-describedby="modal-desc"
      class="relative flex max-h-[92vh] w-full max-w-2xl flex-col overflow-hidden rounded-t-lg border border-hairline bg-slate-obsidian shadow-2xl sm:rounded-lg"
      style="--accent: {selected.accent};"
      onkeydown={trapFocus}
      transition:fly={{ y: 18, duration: 200 }}
    >
      <!-- Accent bar -->
      <span
        class="h-1 w-full shrink-0"
        style="background: linear-gradient(90deg, {selected.accent}, transparent 85%);"
        aria-hidden="true"
      ></span>

      <!-- Header -->
      <div class="flex items-start gap-3 border-b border-hairline px-4 py-4 sm:px-6">
        <span
          class="grid size-11 shrink-0 place-items-center rounded border font-display text-base font-bold"
          style="color: {selected.accent}; border-color: color-mix(in oklab, {selected.accent} 45%, transparent); background: color-mix(in oklab, {selected.accent} 12%, transparent);"
          aria-hidden="true"
        >
          {selected.title.slice(0, 2).toUpperCase()}
        </span>

        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-2">
            <h2 id="modal-title" class="font-display text-xl font-bold tracking-[-0.02em] text-crisp">
              {selected.title}
            </h2>
            <span class="inline-flex items-center gap-1.5 rounded border px-1.5 py-0.5 font-mono text-[10px] {meta.tone}">
              <span class="size-1.5 rounded-full {meta.dot} {selected.status === 'online' ? 'animate-pulse-dot' : ''}" aria-hidden="true"></span>
              {meta.text}
              {#if selected.latency > 0}· {selected.latency}ms{/if}
            </span>
          </div>
          <p class="mt-0.5 truncate font-mono text-[11px] text-muted">{selected.url.replace('https://', '')}</p>
        </div>

        <button
          bind:this={closeButton}
          type="button"
          onclick={onClose}
          class="shrink-0 rounded border border-hairline p-1.5 text-muted transition-colors hover:border-crimson hover:text-crimson"
          aria-label="Tutup pratinjau"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" class="size-4" aria-hidden="true">
            <path d="M18 6 6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Body (scrollable) -->
      <div class="flex-1 space-y-5 overflow-y-auto px-4 py-4 sm:px-6">
        <p id="modal-desc" class="text-sm leading-relaxed text-muted">{selected.description}</p>

        <!-- Architecture diagram -->
        <section aria-labelledby="modal-arch">
          <h3 id="modal-arch" class="mb-2 font-mono text-[10px] uppercase tracking-[0.16em] text-muted">Arsitektur</h3>
          <ol class="flex flex-wrap items-center gap-1.5">
            {#each selected.architecture as node, i (node)}
              <li class="flex items-center gap-1.5">
                <span class="rounded border border-hairline bg-void/70 px-2 py-1 font-mono text-[11px] text-crisp">{node}</span>
                {#if i < selected.architecture.length - 1}
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" class="size-3 text-muted/60" aria-hidden="true">
                    <path d="M5 12h14M13 6l6 6-6 6" />
                  </svg>
                {/if}
              </li>
            {/each}
          </ol>
        </section>

        <!-- Features -->
        <section aria-labelledby="modal-features">
          <h3 id="modal-features" class="mb-2 font-mono text-[10px] uppercase tracking-[0.16em] text-muted">Yang bisa kamu coba</h3>
          <ul class="space-y-1.5">
            {#each selected.features as feature (feature)}
              <li class="flex gap-2 text-sm text-muted">
                <span class="mt-1.5 size-1 shrink-0 rounded-full" style="background: {selected.accent};" aria-hidden="true"></span>
                <span class="leading-relaxed">{feature}</span>
              </li>
            {/each}
          </ul>
        </section>

        <!-- Stack -->
        <section aria-labelledby="modal-stack">
          <h3 id="modal-stack" class="mb-2 font-mono text-[10px] uppercase tracking-[0.16em] text-muted">Tech stack</h3>
          <div class="flex flex-wrap gap-1.5">
            {#each selected.stack as tech (tech)}
              <span class="tech-badge">{tech}</span>
            {/each}
            {#each selected.categories as category (category)}
              <span class="tech-badge" style="color: {selected.accent}; border-color: color-mix(in oklab, {selected.accent} 35%, transparent);">
                #{category}
              </span>
            {/each}
          </div>
        </section>

        <!-- Demo credentials -->
        {#if selected.demo}
          <section aria-labelledby="modal-demo" class="rounded border border-hairline bg-void/60 p-3">
            <h3 id="modal-demo" class="mb-2 font-mono text-[10px] uppercase tracking-[0.16em] text-muted">Akun demo</h3>
            <div class="flex flex-wrap items-center gap-2">
              <dl class="flex flex-1 flex-wrap gap-x-4 gap-y-1 font-mono text-xs">
                <div class="flex gap-1.5">
                  <dt class="text-muted">user</dt>
                  <dd class="text-crisp">{selected.demo.user}</dd>
                </div>
                <div class="flex gap-1.5">
                  <dt class="text-muted">pass</dt>
                  <dd class="text-crisp">{selected.demo.pass}</dd>
                </div>
              </dl>
              <button
                type="button"
                onclick={copyDemo}
                class="rounded border border-hairline px-2 py-1 font-mono text-[11px] text-muted transition-colors hover:border-cyber hover:text-cyber"
              >
                {#if copyState === 'copied'}Tersalin ✓{:else if copyState === 'failed'}Gagal{:else}Copy{/if}
              </button>
            </div>
          </section>
        {/if}

        {#if selected.metrics.stars !== undefined || selected.metrics.requests}
          <dl class="grid grid-cols-2 gap-px overflow-hidden rounded border border-hairline bg-hairline sm:grid-cols-3">
            <div class="bg-slate-obsidian px-3 py-2">
              <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Tahun</dt>
              <dd class="font-mono text-sm text-crisp">{selected.year}</dd>
            </div>
            {#if selected.metrics.stars !== undefined}
              <div class="bg-slate-obsidian px-3 py-2">
                <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Stars</dt>
                <dd class="font-mono text-sm text-crisp">{selected.metrics.stars}</dd>
              </div>
            {/if}
            {#if selected.metrics.requests}
              <div class="bg-slate-obsidian px-3 py-2">
                <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Traffic</dt>
                <dd class="truncate font-mono text-sm text-crisp">{selected.metrics.requests}</dd>
              </div>
            {/if}
          </dl>
        {/if}
      </div>

      <!-- Footer CTAs -->
      <div class="flex shrink-0 flex-col gap-2 border-t border-hairline bg-void/50 px-4 py-3 sm:flex-row sm:items-center sm:px-6">
        <a
          href={selected.url}
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex flex-1 items-center justify-center gap-2 rounded px-4 py-2.5 font-display text-sm font-bold transition-all hover:brightness-110"
          style="background: {selected.accent}; color: #090A0F;"
        >
          Launch App
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" class="size-4" aria-hidden="true">
            <path d="M7 17 17 7M9 7h8v8" />
          </svg>
        </a>
        <a
          href={selected.repo}
          target="_blank"
          rel="noopener noreferrer"
          class="inline-flex items-center justify-center gap-2 rounded border border-hairline px-4 py-2.5 font-mono text-xs text-crisp transition-colors hover:border-cyber hover:text-cyber"
        >
          <svg viewBox="0 0 24 24" fill="currentColor" class="size-3.5" aria-hidden="true">
            <path d="M12 2a10 10 0 0 0-3.16 19.49c.5.09.68-.22.68-.48v-1.7c-2.78.6-3.37-1.34-3.37-1.34-.45-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.89 1.53 2.34 1.09 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.56-1.11-4.56-4.95 0-1.09.39-1.98 1.03-2.68-.1-.25-.45-1.27.1-2.65 0 0 .84-.27 2.75 1.02a9.5 9.5 0 0 1 5 0c1.91-1.29 2.75-1.02 2.75-1.02.55 1.38.2 2.4.1 2.65.64.7 1.03 1.59 1.03 2.68 0 3.85-2.34 4.7-4.57 4.94.36.31.68.92.68 1.85v2.74c0 .27.18.58.69.48A10 10 0 0 0 12 2Z" />
          </svg>
          Source Code
        </a>

        {#if projects.length > 1}
          <div class="flex items-center justify-center gap-1 font-mono text-[11px] text-muted">
            <button
              type="button"
              onclick={() => step(-1)}
              class="rounded border border-hairline px-1.5 py-1 transition-colors hover:border-cyber hover:text-cyber"
              aria-label="Proyek sebelumnya"
            >←</button>
            <span class="px-1">{index + 1}/{projects.length}</span>
            <button
              type="button"
              onclick={() => step(1)}
              class="rounded border border-hairline px-1.5 py-1 transition-colors hover:border-cyber hover:text-cyber"
              aria-label="Proyek berikutnya"
            >→</button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
