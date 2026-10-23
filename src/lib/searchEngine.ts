/**
 * Search engine adapter — "Smart Loader".
 *
 * Tries to load the Rust/WASM engine built by CI (`src/wasm-pkg/portal_wasm.js`).
 * If the artifact is absent (the normal state on a Windows dev box with no Rust
 * toolchain — see README), it silently falls back to the pure-TypeScript
 * implementation. Callers never see an error and never branch on the result.
 */
import { searchProjects as tsSearch, normalize } from './fallbackSearch';
import type { EngineKind, Project, ScoredProject, SearchEngineHandle, SearchQuery } from './types';

const WASM_ENTRY = '../wasm-pkg/portal_wasm.js';
const WASM_VERSION = '0.1.0';

interface WasmModule {
  default: (input?: unknown) => Promise<unknown>;
  search_projects?: (payload: string) => string;
  engine_version?: () => string;
}

const tsEngine: SearchEngineHandle = {
  kind: 'typescript',
  version: 'ts-fallback/1.1.0',
  search: (projects, query) => tsSearch(projects, query),
};

let cached: SearchEngineHandle | null = null;
let inflight: Promise<SearchEngineHandle> | null = null;

/**
 * Shape the query into the flat JSON the Rust serde structs expect.
 * Rust has no notion of optional TS unions, so we send concrete arrays.
 */
function toWasmPayload(projects: Project[], query: SearchQuery) {
  return JSON.stringify({
    text: query.text ?? '',
    stacks: query.stacks ?? [],
    categories: query.categories ?? [],
    statuses: query.statuses ?? [],
    featured_only: Boolean(query.featuredOnly),
    sort: query.sort ?? 'relevance',
    projects: projects.map((p) => ({
      id: p.id,
      title: p.title,
      pitch: p.pitch,
      description: p.description,
      stack: p.stack,
      categories: p.categories,
      status: p.status,
      latency: p.latency,
      featured: p.featured,
      year: p.year,
    })),
  });
}

/** Re-attach the full project objects to the scored ids returned by Rust. */
function rehydrate(projects: Project[], raw: unknown): ScoredProject[] {
  const rows = (raw ?? []) as Array<{ id: string; score: number; matched_field: string }>;
  const byId = new Map(projects.map((p) => [p.id, p]));
  const out: ScoredProject[] = [];
  for (const row of rows) {
    const project = byId.get(row.id);
    if (!project) continue;
    out.push({ project, score: row.score, matchedField: row.matched_field ?? 'none' });
  }
  return out;
}

/**
 * Vite resolves `import.meta.glob` at BUILD time: when `src/wasm-pkg/` exists
 * (CI, after wasm-pack) the module is bundled and `portal_wasm_bg.wasm` is
 * emitted into `_astro/`; when it does not exist (local dev, Zero-Local-Rust)
 * the glob simply returns `{}` and nothing breaks.
 *
 * Do NOT switch this back to `import(specifier)` with `@vite-ignore` — that
 * skipped bundling entirely, so production requested a URL that never existed
 * and silently fell back to TypeScript forever.
 */
const wasmModules = import.meta.glob<WasmModule>('../wasm-pkg/portal_wasm.js');

async function loadWasmEngine(): Promise<SearchEngineHandle | null> {
  try {
    const load = wasmModules['../wasm-pkg/portal_wasm.js'];
    if (!load) return null; // crate not compiled yet — expected locally

    const mod = await load();
    const init = mod.default;
    const searchFn = mod.search_projects;
    if (typeof init !== 'function' || typeof searchFn !== 'function') return null;

    await init();
    const version = typeof mod.engine_version === 'function' ? mod.engine_version() : WASM_VERSION;

    const handle: SearchEngineHandle = {
      kind: 'wasm',
      version: `rust-wasm/${version}`,
      search(projects, query) {
        try {
          const result = searchFn(toWasmPayload(projects, query));
          return rehydrate(projects, JSON.parse(result));
        } catch (error) {
          // A poisoned WASM call must never brick the UI.
          console.warn('[searchEngine] WASM call failed, degrading to TS:', error);
          return tsSearch(projects, query);
        }
      },
    };
    return handle;
  } catch {
    // Expected on localhost: no `src/wasm-pkg` until CI compiles the crate.
    return null;
  }
}

/**
 * Resolve the best available engine. Safe to call from `$effect` — repeated
 * calls reuse the same promise, and the TS fallback resolves immediately so
 * the first paint is never blocked on the WASM fetch.
 */
export function getSearchEngine(): Promise<SearchEngineHandle> {
  if (cached) return Promise.resolve(cached);
  if (inflight) return inflight;

  inflight = loadWasmEngine().then((wasm) => {
    cached = wasm ?? tsEngine;
    inflight = null;
    return cached;
  });

  return inflight;
}

/** Synchronous engine for the very first render (always TS until WASM resolves). */
export function getSyncEngine(): SearchEngineHandle {
  return cached ?? tsEngine;
}

/** Exposed for the telemetry footer and for tests. */
export const engineMeta = {
  wasmEntry: WASM_ENTRY,
  fallback: tsEngine.version,
  normalize,
};

export type { SearchEngineHandle, EngineKind, ScoredProject, SearchQuery };
export { tsSearch as fallbackSearch };
