/**
 * Pure-TypeScript search engine.
 *
 * This is the *reference implementation* of the scoring contract that
 * `crates/portal-wasm/src/lib.rs` mirrors. It exists so the portal runs on
 * machines without a Rust toolchain (see AGENTS.md §2 — "Zero-Local-Rust").
 *
 * Keep the two in sync: same weights, same tie-breaking, same sort order.
 */
import type { Project, ScoredProject, SearchQuery, SortMode } from './types';

const WEIGHTS = {
  titleExact: 1000,
  titlePrefix: 800,
  titleSubstring: 650,
  titleFuzzy: 600, // scaled down by gap penalty
  stackExact: 520,
  stackPrefix: 440,
  stackFuzzy: 320,
  category: 300,
  pitchSubstring: 220,
  pitchFuzzy: 90,
  descriptionSubstring: 120,
  descriptionFuzzy: 50,
  tokenCoverage: 40,
  tokenCoverageMax: 160,
  featured: 25,
  online: 15,
} as const;

/** Lowercase + strip diacritics so "café" matches "cafe". */
export function normalize(input: string): string {
  return input
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
    .trim();
}

/**
 * Subsequence fuzzy match with a gap penalty.
 * Returns a 0..1 quality ratio, or 0 when the needle is not a subsequence.
 * Mirrors `fuzzy_ratio` in the Rust crate.
 *
 * The measured span runs from the FIRST matched character to the LAST one, not
 * from index 0: a leading offset is irrelevant here because prefix and substring
 * hits are already scored separately, and penalising it made single-typo
 * queries like "shopfy" → "mini-shopify" fall below the match threshold.
 */
export function fuzzyRatio(haystack: string, needle: string): number {
  if (needle.length === 0) return 0;
  if (haystack.length === 0) return 0;
  if (needle.length > haystack.length) return 0;

  let hi = 0;
  let matched = 0;
  let gaps = 0;
  let streak = 0;
  let first = -1;
  let last = -1;

  for (let ni = 0; ni < needle.length; ni++) {
    const target = needle[ni];
    let found = -1;
    for (let h = hi; h < haystack.length; h++) {
      if (haystack[h] === target) {
        found = h;
        break;
      }
    }
    if (found === -1) return 0;

    if (ni === 0) first = found;
    if (ni > 0) {
      const gap = found - hi;
      if (gap > 0) {
        gaps += gap;
        streak = 0;
      } else {
        streak += 1;
      }
    }
    matched += 1;
    last = found;
    hi = found + 1;
  }

  // Span covers only the matched region, so leading/trailing noise is free.
  const span = Math.max(last - first + 1, 1);
  const density = matched / span;
  const gapPenalty = gaps / span;
  const streakBonus = Math.min(streak / Math.max(needle.length, 1), 1) * 0.15;

  const ratio = density * 0.85 + streakBonus - gapPenalty * 0.35;
  return Math.min(Math.max(ratio, 0), 1);
}

function scoreToken(project: Project, token: string): { score: number; field: string } {
  const title = normalize(project.title);
  const pitch = normalize(project.pitch);
  const description = normalize(project.description);
  const stacks = project.stack.map(normalize);
  const categories = project.categories.map(normalize);

  if (title === token) return { score: WEIGHTS.titleExact, field: 'title' };
  if (title.startsWith(token)) return { score: WEIGHTS.titlePrefix, field: 'title' };
  if (title.includes(token)) return { score: WEIGHTS.titleSubstring, field: 'title' };

  for (const stack of stacks) {
    if (stack === token) return { score: WEIGHTS.stackExact, field: 'stack' };
    if (stack.startsWith(token)) return { score: WEIGHTS.stackPrefix, field: 'stack' };
  }
  for (const category of categories) {
    if (category === token || category.includes(token)) {
      return { score: WEIGHTS.category, field: 'category' };
    }
  }
  if (pitch.includes(token)) return { score: WEIGHTS.pitchSubstring, field: 'pitch' };
  if (description.includes(token)) {
    return { score: WEIGHTS.descriptionSubstring, field: 'description' };
  }

  // Nothing exact-ish: fall back to fuzzy across the strongest fields.
  const fuzzyTitle = fuzzyRatio(title, token);
  if (fuzzyTitle > 0.45) {
    return { score: WEIGHTS.titleFuzzy * fuzzyTitle, field: 'title' };
  }
  for (const stack of stacks) {
    const r = fuzzyRatio(stack, token);
    if (r > 0.55) return { score: WEIGHTS.stackFuzzy * r, field: 'stack' };
  }
  const fuzzyPitch = fuzzyRatio(pitch, token);
  if (fuzzyPitch > 0.6) return { score: WEIGHTS.pitchFuzzy * fuzzyPitch, field: 'pitch' };
  const fuzzyDescription = fuzzyRatio(description, token);
  if (fuzzyDescription > 0.65) {
    return { score: WEIGHTS.descriptionFuzzy * fuzzyDescription, field: 'description' };
  }

  return { score: 0, field: 'none' };
}

function passesFilters(project: Project, query: SearchQuery): boolean {
  if (query.featuredOnly && !project.featured) return false;

  const stacks = query.stacks ?? [];
  if (stacks.length > 0) {
    const own = project.stack.map(normalize);
    const hit = stacks.some((s) => own.includes(normalize(s)));
    if (!hit) return false;
  }

  const categories = query.categories ?? [];
  if (categories.length > 0) {
    const own = project.categories.map(normalize);
    const hit = categories.some((c) => own.includes(normalize(c)));
    if (!hit) return false;
  }

  const statuses = query.statuses ?? [];
  if (statuses.length > 0 && !statuses.includes(project.status)) return false;

  return true;
}

const STATUS_ORDER: Record<Project['status'], number> = {
  online: 0,
  degraded: 1,
  maintenance: 2,
  offline: 3,
};

function compare(sort: SortMode) {
  return (a: ScoredProject, b: ScoredProject): number => {
    switch (sort) {
      case 'title':
        return a.project.title.localeCompare(b.project.title);
      case 'latency':
        // 0ms means "no signal" (offline/maintenance) → push to the end.
        return (a.project.latency || Infinity) - (b.project.latency || Infinity);
      case 'newest':
        return b.project.year - a.project.year || a.project.title.localeCompare(b.project.title);
      case 'relevance':
      default:
        return (
          b.score - a.score ||
          STATUS_ORDER[a.project.status] - STATUS_ORDER[b.project.status] ||
          a.project.title.localeCompare(b.project.title)
        );
    }
  };
}

/**
 * Run the full query against a project list.
 * An empty text query with no filters returns every project, sorted as requested.
 */
export function searchProjects(projects: Project[], query: SearchQuery = {}): ScoredProject[] {
  const text = normalize(query.text ?? '');
  const tokens = text.split(/\s+/).filter((t) => t.length > 0);
  const sort: SortMode = query.sort ?? 'relevance';

  const results: ScoredProject[] = [];

  for (const project of projects) {
    if (!passesFilters(project, query)) continue;

    if (tokens.length === 0) {
      // No text constraint: neutral score, ordering handled by sort mode.
      results.push({ project, score: 0, matchedField: 'none' });
      continue;
    }

    let total = 0;
    let bestField = 'none';
    let bestScore = 0;
    let matchedTokens = 0;

    for (const token of tokens) {
      const { score, field } = scoreToken(project, token);
      if (score > 0) {
        total += score;
        matchedTokens += 1;
        if (score > bestScore) {
          bestScore = score;
          bestField = field;
        }
      }
    }

    if (matchedTokens === 0) continue;

    // Partial matches still surface, but weaker (e.g. "rust wasm" where only one hits).
    const coverage = matchedTokens / tokens.length;
    total *= coverage;
    total += Math.min(matchedTokens * WEIGHTS.tokenCoverage, WEIGHTS.tokenCoverageMax);
    if (project.featured) total += WEIGHTS.featured;
    if (project.status === 'online') total += WEIGHTS.online;

    results.push({
      project,
      score: Math.round(Math.min(total, WEIGHTS.titleExact)),
      matchedField: bestField,
    });
  }

  return results.sort(compare(sort));
}

/** Lightweight stats for the telemetry footer. */
export function indexStats(projects: Project[]) {
  const stacks = new Set<string>();
  const categories = new Set<string>();
  for (const p of projects) {
    p.stack.forEach((s) => stacks.add(s));
    p.categories.forEach((c) => categories.add(c));
  }
  return {
    projects: projects.length,
    stacks: stacks.size,
    categories: categories.size,
  };
}
