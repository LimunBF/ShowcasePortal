/**
 * Self-check for the TypeScript search engine.
 *
 * Run with: npx tsx src/lib/fallbackSearch.test.ts
 * (or `node --experimental-strip-types src/lib/fallbackSearch.test.ts` on Node 22+)
 *
 * No test framework on purpose: the crate has `cargo test`, and the TS mirror
 * only needs to prove it agrees with it. `assert` failures exit non-zero.
 */
import assert from 'node:assert/strict';
import { fuzzyRatio, normalize, searchProjects } from './fallbackSearch.ts';
import type { Project } from './types.ts';

const projects: Project[] = [
  {
    id: 'mini-shopify',
    title: 'Mini-Shopify SaaS',
    pitch: 'Multi-tenant storefront.',
    description: 'E-commerce platform.',
    url: 'https://shopify.test',
    repo: 'https://github.com/x/y',
    stack: ['React', 'Golang', 'Redis'],
    categories: ['fullstack'],
    status: 'online',
    latency: 24,
    accent: '#00F0FF',
    size: 'large',
    featured: true,
    year: 2025,
    architecture: ['React SPA', 'Go API'],
    features: ['Checkout'],
    demo: null,
    metrics: { stars: 1 },
  },
  {
    id: 'telemetry',
    title: 'Telemetry Daemon',
    pitch: 'Axum health checker.',
    description: 'Rust daemon probing subdomains.',
    url: 'https://status.test',
    repo: 'https://github.com/x/z',
    stack: ['Rust', 'Axum'],
    categories: ['infra'],
    status: 'offline',
    latency: 0,
    accent: '#F59E0B',
    size: 'normal',
    featured: false,
    year: 2024,
    architecture: ['Tokio', 'Axum'],
    features: ['Probe'],
    demo: null,
    metrics: {},
  },
];

// --- primitives -------------------------------------------------------------
assert.equal(normalize('  CAFÉ  '), 'cafe');
assert.equal(normalize('Mini-Shopify'), 'mini-shopify');
assert.equal(fuzzyRatio('shopify', 'xyz'), 0, 'non-subsequence must score 0');

// Regression: leading offset must not sink a single-typo match.
const typo = fuzzyRatio('mini-shopify', 'shopfy');
assert.ok(typo > 0.45, `expected typo ratio > 0.45, got ${typo}`);

// --- exactness ordering -----------------------------------------------------
const exact = searchProjects(projects, { text: 'telemetry' });
assert.equal(exact[0]?.project.id, 'telemetry', 'exact title hit ranks first');
assert.equal(exact[0]?.matchedField, 'title');

// --- filters ----------------------------------------------------------------
assert.equal(searchProjects(projects, { stacks: ['rust'] }).length, 1);
assert.equal(searchProjects(projects, { statuses: ['online'] }).length, 1);
assert.equal(searchProjects(projects, { featuredOnly: true }).length, 1);
assert.equal(
  searchProjects(projects, { stacks: ['rust'], statuses: ['online'] }).length,
  0,
  'stack and status filters AND together'
);

// --- text + filter combined -------------------------------------------------
assert.equal(searchProjects(projects, { text: 'rust', stacks: ['go'] }).length, 0);

// --- empty query returns everything ----------------------------------------
assert.equal(searchProjects(projects, { text: '   ' }).length, 2);

// --- sorting ----------------------------------------------------------------
assert.equal(searchProjects(projects, { sort: 'title' })[0]?.project.id, 'mini-shopify');
assert.equal(searchProjects(projects, { sort: 'newest' })[0]?.project.id, 'mini-shopify');
assert.equal(
  searchProjects(projects, { sort: 'latency' })[0]?.project.id,
  'mini-shopify',
  'zero latency (no signal) sorts last'
);

// --- no match ---------------------------------------------------------------
assert.equal(searchProjects(projects, { text: 'zzzqqq' }).length, 0);

console.log('fallbackSearch: all assertions passed');
