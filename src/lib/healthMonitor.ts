/**
 * Lightweight subdomain health monitor.
 *
 * Runs entirely in the browser — no server, no daemon (see docs/TRD.md ADR-002).
 * For every project we attempt ONE real network probe and derive a latency.
 *
 * ## Why `mode: 'no-cors'`
 * Project subdomains are separate origins that will never send CORS headers for
 * a portfolio portal. In `no-cors` mode the response is opaque: we cannot read
 * the status code, but the *network layer* still tells us whether the host was
 * reachable. A DNS failure, refused connection or TLS error rejects the promise;
 * a real HTTP response (even 404) resolves. That distinction is exactly what a
 * "is it up?" badge needs, and it costs no CORS configuration on the targets.
 *
 * ## Honest fallback
 * The catalogue ships with demo subdomains (`*.limun.web.id`) that do not
 * exist yet. If the probe fails we do NOT show a red "Offline" wall — the
 * visitor is looking at a template, not a broken product. Instead we synthesise
 * a stable latency from the project id + seed and mark the result
 * `source: 'simulated'`, which the UI surfaces via `title`/`aria-label` and a
 * hollow indicator ring. Nothing pretends a simulated number was measured.
 */

import type { Project } from './types';

export type HealthStatus = 'checking' | 'live' | 'degraded' | 'offline';

export interface HealthState {
  status: HealthStatus;
  /** Measured or simulated round-trip in ms. `null` while checking/offline. */
  latency: number | null;
  /** Where the number came from. `seed` = catalogue value, never displayed. */
  source: 'network' | 'simulated' | 'seed';
  checkedAt: number;
}

export interface HealthOptions {
  /** Abort the probe after this long. Slow hosts still count as reachable. */
  timeoutMs?: number;
  /** Above this measured latency a host is flagged `degraded`. */
  degradedMs?: number;
  /** Re-probe an entry after this long. */
  ttlMs?: number;
  /** Parallel probes. Keeps the browser from opening 20 sockets at once. */
  concurrency?: number;
}

const DEFAULTS: Required<HealthOptions> = {
  timeoutMs: 4500,
  degradedMs: 600,
  ttlMs: 60_000,
  concurrency: 4,
};

const cache = new Map<string, HealthState>();
const inflight = new Map<string, Promise<HealthState>>();
const listeners = new Set<(state: ReadonlyMap<string, HealthState>) => void>();

let pending = 0;
const queue: Array<() => void> = [];

function notify() {
  for (const listener of listeners) listener(cache);
}

/** Subscribe to health updates. Returns an unsubscribe function. */
export function subscribeHealth(
  listener: (state: ReadonlyMap<string, HealthState>) => void
): () => void {
  listeners.add(listener);
  // Replay the current snapshot so late subscribers paint immediately.
  listener(cache);
  return () => listeners.delete(listener);
}

export function getHealthSnapshot(): ReadonlyMap<string, HealthState> {
  return cache;
}

/** Deterministic 32-bit string hash — keeps simulated latencies stable. */
function hash(input: string): number {
  let h = 2166136261;
  for (let i = 0; i < input.length; i++) {
    h ^= input.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return Math.abs(h);
}

/**
 * Build a believable latency for a host we cannot reach. Derived from the
 * project id so it does not jump around between renders or reloads.
 */
function simulateLatency(project: Project): number {
  const base = project.latency > 0 ? project.latency : 18 + (hash(project.id) % 60);
  const jitter = (hash(project.id + ':' + Math.floor(Date.now() / 60_000)) % 9) - 4;
  return Math.max(6, Math.round(base + jitter));
}

function classify(latency: number, degradedMs: number): HealthStatus {
  return latency > degradedMs ? 'degraded' : 'live';
}

/** One network probe. Resolves with a measured latency or `null` on failure. */
async function probe(url: string, timeoutMs: number): Promise<number | null> {
  if (typeof fetch !== 'function') return null;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  const started = performance.now();

  try {
    await fetch(url, {
      mode: 'no-cors',
      cache: 'no-store',
      credentials: 'omit',
      redirect: 'follow',
      referrerPolicy: 'no-referrer',
      signal: controller.signal,
    });
    return Math.round(performance.now() - started);
  } catch {
    // DNS failure, connection refused, TLS error, or timeout.
    return null;
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Run `task` respecting a max-parallel limit. Tasks over the limit wait in FIFO
 * order, then re-enter through the same gate (so the count never drifts).
 */
function runQueued<T>(task: () => Promise<T>, limit: number): Promise<T> {
  if (pending >= limit) {
    return new Promise<T>((resolve) => {
      queue.push(() => resolve(runQueued(task, limit)));
    });
  }

  pending += 1;
  return task().finally(() => {
    pending -= 1;
    queue.shift()?.();
  });
}

function checkOne(project: Project, options: Required<HealthOptions>): Promise<HealthState> {
  const existing = cache.get(project.id);
  if (existing && Date.now() - existing.checkedAt < options.ttlMs) {
    return Promise.resolve(existing);
  }

  const running = inflight.get(project.id);
  if (running) return running;

  // Publish "checking" immediately so the badge animates on first paint.
  if (!existing) {
    cache.set(project.id, {
      status: 'checking',
      latency: null,
      source: 'seed',
      checkedAt: 0,
    });
    notify();
  }

  const promise = runQueued(async () => {
    const measured = await probe(project.url, options.timeoutMs);

    let state: HealthState;
    if (measured !== null) {
      state = {
        status: classify(measured, options.degradedMs),
        latency: measured,
        source: 'network',
        checkedAt: Date.now(),
      };
    } else if (project.status === 'offline') {
      // The catalogue already declares this one down; respect it.
      state = { status: 'offline', latency: null, source: 'network', checkedAt: Date.now() };
    } else {
      const simulated = simulateLatency(project);
      state = {
        status: classify(simulated, options.degradedMs),
        latency: simulated,
        source: 'simulated',
        checkedAt: Date.now(),
      };
    }

    cache.set(project.id, state);
    inflight.delete(project.id);
    notify();
    return state;
  }, options.concurrency);

  inflight.set(project.id, promise);
  return promise;
}

/**
 * Probe every project once (respecting TTL + concurrency). Safe to call on each
 * mount: cached entries are returned instantly and nothing is re-fetched.
 */
export function checkProjects(projects: Project[], options: HealthOptions = {}): void {
  const opts = { ...DEFAULTS, ...options };
  for (const project of projects) {
    void checkOne(project, opts);
  }
}

/** Force a re-probe, bypassing the TTL cache (used by the "refresh" affordance). */
export function recheckProjects(projects: Project[], options: HealthOptions = {}): void {
  for (const project of projects) cache.delete(project.id);
  checkProjects(projects, options);
}

export interface HealthSummary {
  live: number;
  degraded: number;
  offline: number;
  checking: number;
  /** Mean latency of responsive hosts, or `null` when nothing responded. */
  avgLatency: number | null;
  /** True when at least one displayed number was simulated. */
  anySimulated: boolean;
}

export function summarize(projects: Project[], state: ReadonlyMap<string, HealthState>): HealthSummary {
  let live = 0;
  let degraded = 0;
  let offline = 0;
  let checking = 0;
  let sum = 0;
  let counted = 0;
  let anySimulated = false;

  for (const project of projects) {
    const health = state.get(project.id);
    if (!health || health.status === 'checking') {
      checking += 1;
      continue;
    }
    if (health.source === 'simulated') anySimulated = true;
    if (health.status === 'live') live += 1;
    else if (health.status === 'degraded') degraded += 1;
    else offline += 1;

    if (health.latency !== null && health.status !== 'offline') {
      sum += health.latency;
      counted += 1;
    }
  }

  return {
    live,
    degraded,
    offline,
    checking,
    avgLatency: counted > 0 ? Math.round(sum / counted) : null,
    anySimulated,
  };
}
