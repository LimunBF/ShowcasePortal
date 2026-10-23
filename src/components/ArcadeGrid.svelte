<!--
  ArcadeGrid — the only heavily interactive island on the page.

  Svelte 5 Runes only (AGENTS.md §3.1):
    $props   — projects passed from the Astro page (serialized JSON)
    $state   — query, active filter, modal target, live health map
    $derived — filtered rows, live counters
    $effect  — engine resolution, CustomEvent bridge, health probing

  The grid is the page's whole job now: header + search/filter + bento tiles.
  No explanatory sections below it — Friv-style, the games ARE the content.
-->
<script lang="ts">
  import type { EngineKind, Project, ScoredProject, SearchEngineHandle, SortMode } from '../lib/types';
  import { getSearchEngine, getSyncEngine } from '../lib/searchEngine';
  import {
    checkProjects,
    recheckProjects,
    subscribeHealth,
    summarize,
    type HealthState,
  } from '../lib/healthMonitor';
  import ProjectCard from './ProjectCard.svelte';
  import ProjectModal from './ProjectModal.svelte';

  interface Props {
    projects: Project[];
    /** Filter pills rendered in the grid toolbar (derived in index.astro). */
    stacks?: string[];
  }

  const { projects, stacks = [] } = $props<Props>();

  // --- Reactive state -------------------------------------------------------
  let query = $state('');
  let activeStack = $state('All');
  let sort = $state<SortMode>('relevance');
  let engineKind = $state<EngineKind>('typescript');
  let engineVersion = $state('ts-fallback');
  let selected = $state<Project | null>(null);
  let ready = $state(false);
  let health = $state<ReadonlyMap<string, HealthState>>(new Map());

  // Captured once by the resolving $effect, then re-read by $derived blocks.
  // Kept in runes (not a plain const) so the derived cache invalidates the
  // moment the WASM module (or its fallback) is confirmed.
  let engine = $state<SearchEngineHandle | null>(null);

  // --- Derived --------------------------------------------------------------
  // `engine.search` is the adapter: TS until WASM resolves, then Rust. Either
  // way the return shape is identical, so the template never branches.
  const rows = $derived<ScoredProject[]>(
    (engine ?? getSyncEngine()).search(projects, {
      text: query,
      stacks: activeStack === 'All' ? [] : [activeStack],
      sort,
    })
  );

  const total = $derived(projects.length);
  const shown = $derived(rows.length);
  const isFiltered = $derived(query.trim().length > 0 || activeStack !== 'All');

  // Counters come from the live probe, not from the static JSON.
  const stats = $derived(summarize(projects, health));
  const checking = $derived(stats.checking > 0);

  // --- Effects --------------------------------------------------------------
  $effect(() => {
    let cancelled = false;
    getSearchEngine().then((resolved) => {
      if (cancelled) return;
      engineKind = resolved.kind;
      engineVersion = resolved.version;
      engine = resolved;
      ready = true;
      syncFooterBadge(
        resolved.kind === 'wasm' ? `rust-wasm · ${resolved.version}` : 'typescript fallback'
      );
    });
    return () => {
      cancelled = true;
    };
  });

  // Bridge from the static header input (CustomEvent, no shared store needed).
  $effect(() => {
    const onQuery = (event: Event) => {
      const detail = (event as CustomEvent<{ text?: string }>).detail;
      query = detail?.text ?? '';
    };
    window.addEventListener('devarcade:query', onQuery as EventListener);
    return () => window.removeEventListener('devarcade:query', onQuery as EventListener);
  });

  // Live health: subscribe first (replays the snapshot), then probe once on mount.
  $effect(() => {
    const unsubscribe = subscribeHealth((snapshot) => {
      health = new Map(snapshot);
    });
    checkProjects(projects);
    return unsubscribe;
  });

  function syncFooterBadge(text: string) {
    const badge = document.getElementById('engine-badge');
    if (badge) {
      badge.textContent = `engine: ${text}`;
      badge.dataset.engine = engineKind;
    }
  }

  function refreshHealth() {
    recheckProjects(projects);
  }

  function resetFilters() {
    query = '';
    activeStack = 'All';
    sort = 'relevance';
    const input = document.getElementById('global-search') as HTMLInputElement | null;
    if (input) input.value = '';
  }

  function openModal(project: Project) {
    selected = project;
  }

  function closeModal() {
    selected = null;
  }
</script>

<section id="arcade" class="mx-auto w-full max-w-7xl px-4 pb-12 pt-8 sm:px-6" aria-labelledby="arcade-title">
  <!-- Hero strip -->
  <div class="mb-6 flex flex-col gap-5 border-b border-hairline pb-5 lg:flex-row lg:items-end lg:justify-between">
    <div class="max-w-2xl">
      <p
        class="mb-2 inline-flex items-center gap-2 rounded border border-hairline bg-slate-obsidian px-2 py-1 font-mono text-[10px] uppercase tracking-[0.18em] text-cyber"
        title={stats.anySimulated ? 'Sebagian angka berasal dari estimasi: subdomain demo belum aktif.' : undefined}
      >
        <span class="size-1.5 rounded-full bg-online text-online animate-pulse-dot" aria-hidden="true"></span>
        {#if checking}
          memeriksa {total} proyek…
        {:else}
          {stats.live}/{total} proyek live
          {#if stats.degraded > 0}· {stats.degraded} lambat{/if}
          {#if stats.offline > 0}· {stats.offline} mati{/if}
        {/if}
      </p>
      <h1 id="arcade-title" class="font-display text-3xl font-bold leading-[1.05] tracking-[-0.03em] text-crisp sm:text-4xl lg:text-5xl">
        Pilih <span class="text-astro">arcade</span>, langsung main.
      </h1>
    </div>

    <dl class="grid grid-cols-3 gap-px overflow-hidden rounded border border-hairline bg-hairline lg:min-w-[300px]">
      <div class="bg-slate-obsidian px-3 py-2.5">
        <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Proyek</dt>
        <dd class="mt-0.5 font-mono text-lg font-bold text-crisp">
          {shown}<span class="text-muted/70">/{total}</span>
        </dd>
      </div>
      <div class="bg-slate-obsidian px-3 py-2.5">
        <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Avg ping</dt>
        <dd class="mt-0.5 font-mono text-lg font-bold {stats.degraded > 0 ? 'text-amber-warn' : 'text-online'}">
          {#if stats.avgLatency !== null}
            {stats.avgLatency}<span class="text-xs text-muted/70">ms</span>
          {:else}
            <span class="text-muted/70">—</span>
          {/if}
        </dd>
      </div>
      <div class="bg-slate-obsidian px-3 py-2.5">
        <dt class="font-mono text-[10px] uppercase tracking-wider text-muted">Engine</dt>
        <dd class="mt-0.5 font-mono text-xs font-bold {engineKind === 'wasm' ? 'text-astro' : 'text-cyber'}">
          {engineKind === 'wasm' ? 'RUST/WASM' : 'TS FALLBACK'}
        </dd>
      </div>
    </dl>
  </div>

  <!-- Filter toolbar -->
  <div class="mb-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
    <div class="flex flex-wrap items-center gap-1.5" role="group" aria-label="Filter tech stack">
      <button
        type="button"
        onclick={() => (activeStack = 'All')}
        aria-pressed={activeStack === 'All'}
        class="rounded border px-2.5 py-1 font-mono text-xs transition-colors {activeStack === 'All'
          ? 'border-astro bg-astro/10 text-astro'
          : 'border-hairline bg-slate-obsidian text-muted hover:text-crisp'}"
      >
        All
      </button>
      {#each stacks as stack (stack)}
        <button
          type="button"
          onclick={() => (activeStack = stack)}
          aria-pressed={activeStack === stack}
          class="rounded border px-2.5 py-1 font-mono text-xs transition-colors {activeStack === stack
            ? 'border-cyber bg-cyber/10 text-cyber'
            : 'border-hairline bg-slate-obsidian text-muted hover:text-crisp'}"
        >
          {stack}
        </button>
      {/each}
    </div>

    <div class="flex items-center gap-2">
      <button
        type="button"
        onclick={refreshHealth}
        title="Periksa ulang status semua proyek"
        class="rounded border border-hairline px-2 py-1 font-mono text-xs text-muted transition-colors hover:border-cyber hover:text-cyber"
      >
        <span class="inline-flex items-center gap-1">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.2"
            stroke-linecap="round"
            class="size-3 {checking ? 'animate-spin' : ''}"
            aria-hidden="true"
          >
            <path d="M21 12a9 9 0 1 1-2.6-6.4M21 3v6h-6" />
          </svg>
          Ping
        </span>
      </button>

      <label for="sort-mode" class="font-mono text-[11px] uppercase tracking-wider text-muted">Urut</label>
      <select
        id="sort-mode"
        bind:value={sort}
        class="rounded border border-hairline bg-slate-obsidian px-2 py-1 font-mono text-xs text-crisp focus:border-cyber focus:outline-none"
      >
        <option value="relevance">Relevansi</option>
        <option value="title">Nama A–Z</option>
        <option value="latency">Latensi</option>
        <option value="newest">Terbaru</option>
      </select>
      {#if isFiltered}
        <button
          type="button"
          onclick={resetFilters}
          class="rounded border border-hairline px-2 py-1 font-mono text-xs text-muted transition-colors hover:border-crimson hover:text-crimson"
        >
          Reset
        </button>
      {/if}
    </div>
  </div>

  <!-- Bento grid -->
  {#if rows.length > 0}
    <ul class="grid grid-cols-1 gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4" aria-label="Daftar proyek">
      {#each rows as { project, matchedField } (project.id)}
        <ProjectCard
          {project}
          health={health.get(project.id)}
          {matchedField}
          hasQuery={query.trim().length > 0}
          onOpen={openModal}
        />
      {/each}
    </ul>
  {:else}
    <div class="brutal-panel flex flex-col items-center justify-center gap-3 rounded px-6 py-16 text-center">
      <span class="grid size-10 place-items-center rounded border border-hairline text-muted" aria-hidden="true">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" class="size-5">
          <circle cx="11" cy="11" r="7" />
          <path d="m20 20-3.5-3.5" />
        </svg>
      </span>
      <p class="font-display text-lg font-bold text-crisp">Tidak ada arcade yang cocok</p>
      <p class="max-w-sm text-sm text-muted">
        {#if query.trim()}
          Query <span class="font-mono text-cyber">"{query}"</span> tidak menemukan proyek. Coba kata kunci lain seperti
          <span class="font-mono">rust</span> atau <span class="font-mono">golang</span>.
        {:else}
          Filter <span class="font-mono text-cyber">{activeStack}</span> kosong untuk saat ini.
        {/if}
      </p>
      <button
        type="button"
        onclick={resetFilters}
        class="mt-1 rounded border border-astro bg-astro/10 px-3 py-1.5 font-mono text-xs text-astro transition-colors hover:bg-astro/20"
      >
        Reset filter
      </button>
    </div>
  {/if}

  <p class="mt-6 flex flex-wrap items-center gap-x-2 gap-y-1 font-mono text-[11px] text-muted/70">
    <span class="size-1 rounded-full {ready && engineKind === 'wasm' ? 'bg-astro' : 'bg-cyber'}" aria-hidden="true"></span>
    {#if ready && engineKind === 'wasm'}
      Search: Rust WebAssembly ({engineVersion}).
    {:else}
      Search: fallback TypeScript (mesin Rust belum ter-compile).
    {/if}
    <span aria-hidden="true">·</span>
    <span>Status: probe browser{#if stats.anySimulated} · sebagian estimasi{:else} · terukur{/if}</span>
  </p>
</section>

<ProjectModal
  {selected}
  projects={rows.map((r) => r.project)}
  onClose={closeModal}
  onSelect={(project) => (selected = project)}
/>
