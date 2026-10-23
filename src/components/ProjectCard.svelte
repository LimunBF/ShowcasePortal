<!--
  ProjectCard — one arcade tile (DESIGN.md §4.B).

  The live badge in the top-right corner is NOT the static value from
  projects.json: it reflects the async probe in `healthMonitor.ts` and animates
  through `checking → live/degraded/offline` once the browser has measured (or
  simulated) a round-trip. Runes: $props / $derived.
-->
<script lang="ts">
  import type { HealthState } from '../lib/healthMonitor';
  import type { Project } from '../lib/types';

  interface Props {
    project: Project;
    /** Live health for this project; `undefined` until the first probe lands. */
    health?: HealthState;
    /** Search field that produced the hit, shown in the footer meta line. */
    matchedField?: string;
    /** True when a text query is active (switches the meta line to match info). */
    hasQuery?: boolean;
    onOpen: (project: Project) => void;
  }

  const { project, health, matchedField = 'none', hasQuery = false, onOpen } = $props<Props>();

  const sizeClass: Record<Project['size'], string> = {
    normal: '',
    wide: 'sm:col-span-2',
    large: 'sm:col-span-2 xl:col-span-2',
  };

  // While the probe runs we keep the catalogue's declared status visible, but
  // the badge explicitly says "checking" so it never looks like a measurement.
  const status = $derived(health?.status ?? 'checking');

  const badge = $derived.by(() => {
    switch (status) {
      case 'live':
        return {
          label: 'Live',
          dot: 'bg-online text-online',
          ring: 'border-online/30',
          text: 'text-online',
          pulse: true,
        };
      case 'degraded':
        return {
          label: 'Degraded',
          dot: 'bg-amber-warn text-amber-warn',
          ring: 'border-amber-warn/30',
          text: 'text-amber-warn',
          pulse: false,
        };
      case 'offline':
        return {
          label: 'Offline',
          dot: 'bg-crimson text-crimson',
          ring: 'border-crimson/30',
          text: 'text-crimson',
          pulse: false,
        };
      default:
        return {
          label: 'Checking',
          dot: 'bg-muted text-muted',
          ring: 'border-hairline',
          text: 'text-muted',
          pulse: true,
        };
    }
  });

  // Simulated numbers are flagged in the tooltip so the badge stays honest.
  const badgeTitle = $derived.by(() => {
    if (!health) return 'Mengukur status…';
    if (health.status === 'checking') return 'Mengukur status…';
    if (health.status === 'offline') return 'Host tidak merespons';
    const ms = health.latency === null ? '—' : `${health.latency}ms`;
    return health.source === 'simulated'
      ? `${ms} · estimasi (subdomain demo belum aktif)`
      : `${ms} · terukur langsung dari browser`;
  });

  const latencyLabel = $derived(
    health?.latency !== null && health?.latency !== undefined ? `${health.latency}ms` : null
  );
</script>

<li class={sizeClass[project.size]}>
  <button
    type="button"
    onclick={() => onOpen(project)}
    class="group relative flex h-full w-full flex-col overflow-hidden rounded border border-hairline bg-slate-obsidian p-4 text-left transition-all duration-200 hover:-translate-y-1 hover:bg-slate-elevated focus-visible:-translate-y-1"
    style="--accent: {project.accent};"
    aria-label="Buka pratinjau {project.title}"
  >
    <!-- Accent hairline that lights up on hover -->
    <span
      class="pointer-events-none absolute inset-x-0 top-0 h-px opacity-0 transition-opacity duration-200 group-hover:opacity-100"
      style="background: linear-gradient(90deg, transparent, {project.accent}, transparent);"
      aria-hidden="true"
    ></span>

    <div class="mb-3 flex items-start justify-between gap-2">
      <span
        class="grid size-9 shrink-0 place-items-center rounded border border-hairline font-display text-sm font-bold transition-colors"
        style="color: {project.accent}; background: color-mix(in oklab, {project.accent} 12%, transparent);"
        aria-hidden="true"
      >
        {project.title.slice(0, 2).toUpperCase()}
      </span>

      <!-- Live status badge (async) -->
      <span
        class="flex shrink-0 items-center gap-1.5 rounded border bg-void/70 px-1.5 py-0.5 font-mono text-[10px] {badge.ring} {badge.text}"
        title={badgeTitle}
        aria-label="Status: {badge.label}{latencyLabel ? `, ${latencyLabel}` : ''}"
        role="status"
        aria-live="polite"
      >
        <span
          class="size-1.5 rounded-full {badge.dot} {badge.pulse ? 'animate-pulse-dot' : ''}"
          aria-hidden="true"
        ></span>
        <span>{badge.label}</span>
        {#if latencyLabel}
          <span class="text-muted/80">({latencyLabel})</span>
        {/if}
      </span>
    </div>

    <h3 class="font-display text-base font-bold leading-tight tracking-[-0.02em] text-crisp">
      {project.title}
    </h3>
    <p class="mt-1.5 line-clamp-2 text-xs leading-relaxed text-muted">{project.pitch}</p>

    <div class="mt-auto flex flex-wrap gap-1 pt-3">
      {#each project.stack.slice(0, 4) as tech (tech)}
        <span class="tech-badge">{tech}</span>
      {/each}
      {#if project.stack.length > 4}
        <span class="tech-badge">+{project.stack.length - 4}</span>
      {/if}
    </div>

    <div class="mt-3 flex items-center justify-between border-t border-hairline pt-2.5">
      <span class="font-mono text-[10px] uppercase tracking-wider text-muted">
        {#if matchedField !== 'none' && hasQuery}
          match: {matchedField}
        {:else}
          {project.year}
        {/if}
      </span>
      <span class="flex items-center gap-1 font-mono text-[11px] font-bold text-muted transition-colors group-hover:text-[var(--accent)]">
        Launch
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" class="size-3" aria-hidden="true">
          <path d="M7 17 17 7M9 7h8v8" />
        </svg>
      </span>
    </div>
  </button>
</li>
