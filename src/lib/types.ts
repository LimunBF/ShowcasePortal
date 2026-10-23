export type ProjectStatus = 'online' | 'degraded' | 'maintenance' | 'offline';

export type ProjectSize = 'normal' | 'wide' | 'large';

export interface DemoCredentials {
  user: string;
  pass: string;
}

export interface ProjectMetrics {
  stars?: number;
  requests?: string;
}

export interface Project {
  id: string;
  title: string;
  pitch: string;
  description: string;
  url: string;
  repo: string;
  stack: string[];
  categories: string[];
  status: ProjectStatus;
  latency: number;
  accent: string;
  size: ProjectSize;
  featured: boolean;
  year: number;
  architecture: string[];
  features: string[];
  demo: DemoCredentials | null;
  metrics: ProjectMetrics;
}

export type SortMode = 'relevance' | 'title' | 'latency' | 'newest';

export interface SearchQuery {
  /** Free-text query. Empty string means "no text constraint". */
  text?: string;
  /** OR-filter across the project stack (case-insensitive). */
  stacks?: string[];
  /** OR-filter across categories. */
  categories?: string[];
  /** OR-filter across statuses. */
  statuses?: ProjectStatus[];
  /** Only projects flagged as featured. */
  featuredOnly?: boolean;
  sort?: SortMode;
}

export interface ScoredProject {
  project: Project;
  /** 0..1000 — higher is a better match. 1000 means exact title hit. */
  score: number;
  /** Field that produced the strongest contribution: title|stack|pitch|description|category. */
  matchedField: string;
}

export type EngineKind = 'wasm' | 'typescript';

export interface SearchEngineHandle {
  kind: EngineKind;
  /** Semver-ish string reported by the engine, shown in the telemetry footer. */
  version: string;
  search(projects: Project[], query: SearchQuery): ScoredProject[];
}
