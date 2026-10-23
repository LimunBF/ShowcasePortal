//! # portal-wasm
//!
//! Client-side search, filtering and telemetry scoring engine for the
//! **DevArcade** showcase portal.
//!
//! ## Why Rust here?
//! The portal ships as pure static files on shared cPanel hosting (see
//! ADR-002/ADR-003). All the ranking work therefore happens in
//! the visitor's browser. Doing it in Rust keeps the algorithm fast, explicit
//! and free of the fuzzy-search library churn that plagues JS bundles — the
//! compiled artefact is a few dozen kilobytes.
//!
//! ## Contract with the frontend
//! `src/lib/searchEngine.ts` tries to `import('../wasm-pkg/portal_wasm.js')`.
//! If that file is missing (a normal local dev box without a Rust toolchain),
//! the TypeScript implementation in `src/lib/fallbackSearch.ts` takes over.
//! **Both implementations must agree on weights, thresholds and tie-breaking.**
//!
//! ## Entry points
//! * [`search_projects`] — JSON in, JSON out. The only function the UI calls.
//! * [`engine_version`] — semver string rendered in the page footer.
//! * [`score_project`] — single-record scoring helper, handy for debugging.
//!
//! ## Tests
//! `cargo test` (runs natively, no wasm target needed) covers the fuzzy matcher,
//! the filter pipeline and the ranking order.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Engine semantic version. Bump when scoring weights change, because the
/// TypeScript fallback is expected to be updated in lockstep.
pub const ENGINE_VERSION: &str = "0.1.0";

// ---------------------------------------------------------------------------
// Scoring weights — mirrored 1:1 in `src/lib/fallbackSearch.ts` (WEIGHTS).
// Changing one without the other causes visible ranking drift between
// environments (WASM in CI, TS fallback locally).
// ---------------------------------------------------------------------------
const W_TITLE_EXACT: f32 = 1000.0;
const W_TITLE_PREFIX: f32 = 800.0;
const W_TITLE_SUBSTRING: f32 = 650.0;
const W_TITLE_FUZZY: f32 = 600.0;
const W_STACK_EXACT: f32 = 520.0;
const W_STACK_PREFIX: f32 = 440.0;
const W_STACK_FUZZY: f32 = 320.0;
const W_CATEGORY: f32 = 300.0;
const W_PITCH_SUBSTRING: f32 = 220.0;
const W_PITCH_FUZZY: f32 = 90.0;
const W_DESCRIPTION_SUBSTRING: f32 = 120.0;
const W_DESCRIPTION_FUZZY: f32 = 50.0;
const W_TOKEN_COVERAGE: f32 = 40.0;
const W_TOKEN_COVERAGE_MAX: f32 = 160.0;
const W_FEATURED: f32 = 25.0;
const W_ONLINE: f32 = 15.0;

/// Minimum fuzzy ratios before a fuzzy hit is considered meaningful.
const FUZZY_TITLE_MIN: f32 = 0.45;
const FUZZY_STACK_MIN: f32 = 0.55;
const FUZZY_PITCH_MIN: f32 = 0.60;
const FUZZY_DESCRIPTION_MIN: f32 = 0.65;

// ---------------------------------------------------------------------------
// Wire types
// ---------------------------------------------------------------------------

/// A project as the frontend ships it to the engine.
/// Fields are deliberately a flattened subset of `src/data/projects.json`:
/// the engine only needs what it ranks on.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectRecord {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub pitch: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub stack: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub latency: u32,
    #[serde(default)]
    pub featured: bool,
    #[serde(default)]
    pub year: u32,
}

fn default_status() -> String {
    "offline".to_string()
}

/// Incoming query payload. `sort` accepts the same literals as the TS union
/// (`relevance`, `title`, `latency`, `newest`); unknown values degrade to
/// relevance so a stale client never blanks the grid.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub stacks: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub statuses: Vec<String>,
    #[serde(default)]
    pub featured_only: bool,
    #[serde(default)]
    pub sort: String,
    #[serde(default)]
    pub projects: Vec<ProjectRecord>,
}

/// One ranked row returned to the frontend. `id` is re-joined to the full
/// project object in TypeScript (`rehydrate`), so we never ship duplicate
/// payloads across the WASM boundary.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoredRow {
    pub id: String,
    pub score: u32,
    pub matched_field: String,
}

/// Which field produced the strongest contribution. Rendered as a hint in the
/// command palette ("matched in stack").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Title,
    Stack,
    Category,
    Pitch,
    Description,
    None,
}

impl Field {
    fn as_str(self) -> &'static str {
        match self {
            Field::Title => "title",
            Field::Stack => "stack",
            Field::Category => "category",
            Field::Pitch => "pitch",
            Field::Description => "description",
            Field::None => "none",
        }
    }
}

// ---------------------------------------------------------------------------
// Primitives
// ---------------------------------------------------------------------------

/// Lowercase + strip diacritics, mirroring `normalize()` in TypeScript.
///
/// Only Latin-1 supplement / Latin Extended-A combining marks are handled,
/// which covers the project catalogue. Anything exotic passes through
/// unchanged rather than being mangled.
pub fn normalize(input: &str) -> String {
    let lowered = input.to_lowercase();
    let mut out = String::with_capacity(lowered.len());
    for ch in lowered.chars() {
        match ch {
            // Combining diacritical marks: drop them. Covers text that arrived
            // pre-decomposed (NFD), e.g. "e" + U+0301.
            '\u{0300}'..='\u{036F}' => continue,
            _ => match fold_latin(ch) {
                Some(base) => out.push(base),
                None => out.push(ch),
            },
        }
    }
    out.trim().to_string()
}

/// Fold precomposed Latin letters to their ASCII base.
///
/// Rust's stdlib has no Unicode normalisation and `to_lowercase()` does not
/// decompose "É" (U+00C9) into "E" + U+0301, so without this the result was
/// `"café"` while the TypeScript mirror returned `"cafe"` — the two engines
/// then ranked differently, which breaks the whole point of having a mirror.
///
/// Every entry below was generated from Unicode NFD data and verified to
/// decompose to `<ascii base> + combining marks`. Letters that do NOT decompose
/// that way in JS either (`ø`, `ł`, `đ`, `ħ`, `ŧ`, `ı`, `ŋ`, `ß`) are
/// deliberately absent, otherwise Rust would fold what TS leaves intact.
///
/// Do NOT rewrite this as char RANGES: `'ñ'..='ň'` spans U+00F1–U+0148 and
/// silently swallows ö/ü/ý in between ("Zürich" became "znrich").
fn fold_latin(ch: char) -> Option<char> {
    Some(match ch {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'ǎ' | 'ǟ' | 'ǡ' | 'ǻ' | 'ȁ' | 'ȃ' | 'ȧ' | 'ḁ' | 'ạ' | 'ả' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'a',
        'ḃ' | 'ḅ' | 'ḇ' => 'b',
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' | 'ḉ' => 'c',
        'ď' | 'ḋ' | 'ḍ' | 'ḏ' | 'ḑ' | 'ḓ' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' | 'ȅ' | 'ȇ' | 'ȩ' | 'ḕ' | 'ḗ' | 'ḙ' | 'ḛ' | 'ḝ' | 'ẹ' | 'ẻ' | 'ẽ' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'e',
        'ḟ' => 'f',
        'ĝ' | 'ğ' | 'ġ' | 'ģ' | 'ǧ' | 'ǵ' | 'ḡ' => 'g',
        'ĥ' | 'ȟ' | 'ḣ' | 'ḥ' | 'ḧ' | 'ḩ' | 'ḫ' | 'ẖ' => 'h',
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ǐ' | 'ȉ' | 'ȋ' | 'ḭ' | 'ḯ' | 'ỉ' | 'ị' => 'i',
        'ĵ' | 'ǰ' => 'j',
        'ķ' | 'ǩ' | 'ḱ' | 'ḳ' | 'ḵ' => 'k',
        'ĺ' | 'ļ' | 'ľ' | 'ḷ' | 'ḹ' | 'ḻ' | 'ḽ' => 'l',
        'ḿ' | 'ṁ' | 'ṃ' => 'm',
        'ñ' | 'ń' | 'ņ' | 'ň' | 'ǹ' | 'ṅ' | 'ṇ' | 'ṉ' | 'ṋ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ŏ' | 'ő' | 'ơ' | 'ǒ' | 'ǫ' | 'ǭ' | 'ȍ' | 'ȏ' | 'ȫ' | 'ȭ' | 'ȯ' | 'ȱ' | 'ṍ' | 'ṏ' | 'ṑ' | 'ṓ' | 'ọ' | 'ỏ' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'o',
        'ṕ' | 'ṗ' => 'p',
        'ŕ' | 'ŗ' | 'ř' | 'ȑ' | 'ȓ' | 'ṙ' | 'ṛ' | 'ṝ' | 'ṟ' => 'r',
        'ś' | 'ŝ' | 'ş' | 'š' | 'ș' | 'ṡ' | 'ṣ' | 'ṥ' | 'ṧ' | 'ṩ' => 's',
        'ţ' | 'ť' | 'ț' | 'ṫ' | 'ṭ' | 'ṯ' | 'ṱ' | 'ẗ' => 't',
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' | 'ư' | 'ǔ' | 'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ȕ' | 'ȗ' | 'ṳ' | 'ṵ' | 'ṷ' | 'ṹ' | 'ṻ' | 'ụ' | 'ủ' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'u',
        'ṽ' | 'ṿ' => 'v',
        'ŵ' | 'ẁ' | 'ẃ' | 'ẅ' | 'ẇ' | 'ẉ' | 'ẘ' => 'w',
        'ẋ' | 'ẍ' => 'x',
        'ý' | 'ÿ' | 'ŷ' | 'ȳ' | 'ẏ' | 'ẙ' | 'ỳ' | 'ỵ' | 'ỷ' | 'ỹ' => 'y',
        'ź' | 'ż' | 'ž' | 'ẑ' | 'ẓ' | 'ẕ' => 'z',
        _ => return None,
    })
}

/// Subsequence fuzzy match with gap penalty. Returns a 0.0..=1.0 quality
/// ratio, or 0.0 when `needle` is not a subsequence of `haystack`.
///
/// The scoring shape intentionally matches the JS version:
/// `density * 0.85 + streak_bonus - gap_penalty * 0.35`, clamped to `0..=1`.
/// Keeping both identical is what makes "same query, same order" hold whether
/// the visitor is served the WASM build or the TS fallback.
///
/// The measured span runs from the FIRST matched character to the LAST one, not
/// from index 0. A leading offset is not a quality signal: prefix and substring
/// hits are scored separately, and penalising it pushed single-typo queries
/// ("shopfy" -> "mini-shopify") below every fuzzy threshold.
pub fn fuzzy_ratio(haystack: &str, needle: &str) -> f32 {
    if needle.is_empty() || haystack.is_empty() || needle.len() > haystack.len() {
        return 0.0;
    }

    let hay: Vec<char> = haystack.chars().collect();
    let ndl: Vec<char> = needle.chars().collect();

    let mut hi = 0usize;
    let mut matched = 0usize;
    let mut gaps = 0usize;
    let mut streak = 0usize;
    let mut first: Option<usize> = None;
    let mut last = 0usize;

    for (ni, &target) in ndl.iter().enumerate() {
        let mut found: Option<usize> = None;
        for h in hi..hay.len() {
            if hay[h] == target {
                found = Some(h);
                break;
            }
        }

        let Some(pos) = found else {
            return 0.0; // not a subsequence
        };

        if ni == 0 {
            first = Some(pos);
        }
        if ni > 0 {
            let gap = pos.saturating_sub(hi);
            if gap > 0 {
                gaps += gap;
                streak = 0;
            } else {
                // Consecutive match (gap == 0 means it sits right after the
                // previous index because `hi` already advanced past it).
                streak += 1;
            }
        }

        matched += 1;
        last = pos;
        hi = pos + 1;
    }

    let start = first.unwrap_or(0);
    // Span covers only the matched region, so leading/trailing noise is free.
    let span = (last + 1).saturating_sub(start).max(1) as f32;
    let density = matched as f32 / span;
    let gap_penalty = gaps as f32 / span;
    let streak_bonus = (streak as f32 / ndl.len().max(1) as f32).min(1.0) * 0.15;

    (density * 0.85 + streak_bonus - gap_penalty * 0.35).clamp(0.0, 1.0)
}

/// Score one normalized token against one project.
fn score_token(project: &ProjectRecord, token: &str) -> (f32, Field) {
    let title = normalize(&project.title);
    if title == token {
        return (W_TITLE_EXACT, Field::Title);
    }
    if title.starts_with(token) {
        return (W_TITLE_PREFIX, Field::Title);
    }
    if title.contains(token) {
        return (W_TITLE_SUBSTRING, Field::Title);
    }

    let stacks: Vec<String> = project.stack.iter().map(|s| normalize(s)).collect();
    for stack in &stacks {
        if stack == token {
            return (W_STACK_EXACT, Field::Stack);
        }
        if stack.starts_with(token) {
            return (W_STACK_PREFIX, Field::Stack);
        }
    }

    for category in &project.categories {
        let normalized = normalize(category);
        if normalized == token || normalized.contains(token) {
            return (W_CATEGORY, Field::Category);
        }
    }

    let pitch = normalize(&project.pitch);
    if pitch.contains(token) {
        return (W_PITCH_SUBSTRING, Field::Pitch);
    }

    let description = normalize(&project.description);
    if description.contains(token) {
        return (W_DESCRIPTION_SUBSTRING, Field::Description);
    }

    // No literal hit — try fuzzy, strongest fields first.
    let title_ratio = fuzzy_ratio(&title, token);
    if title_ratio > FUZZY_TITLE_MIN {
        return (W_TITLE_FUZZY * title_ratio, Field::Title);
    }
    for stack in &stacks {
        let ratio = fuzzy_ratio(stack, token);
        if ratio > FUZZY_STACK_MIN {
            return (W_STACK_FUZZY * ratio, Field::Stack);
        }
    }
    let pitch_ratio = fuzzy_ratio(&pitch, token);
    if pitch_ratio > FUZZY_PITCH_MIN {
        return (W_PITCH_FUZZY * pitch_ratio, Field::Pitch);
    }
    let description_ratio = fuzzy_ratio(&description, token);
    if description_ratio > FUZZY_DESCRIPTION_MIN {
        return (W_DESCRIPTION_FUZZY * description_ratio, Field::Description);
    }

    (0.0, Field::None)
}

/// Apply the hard filters. Returns `false` when the record must be dropped.
fn passes_filters(project: &ProjectRecord, request: &SearchRequest) -> bool {
    if request.featured_only && !project.featured {
        return false;
    }

    if !request.stacks.is_empty() {
        let own: Vec<String> = project.stack.iter().map(|s| normalize(s)).collect();
        let hit = request
            .stacks
            .iter()
            .any(|want| own.iter().any(|have| *have == normalize(want)));
        if !hit {
            return false;
        }
    }

    if !request.categories.is_empty() {
        let own: Vec<String> = project.categories.iter().map(|c| normalize(c)).collect();
        let hit = request
            .categories
            .iter()
            .any(|want| own.iter().any(|have| *have == normalize(want)));
        if !hit {
            return false;
        }
    }

    if !request.statuses.is_empty() {
        let status = normalize(&project.status);
        let hit = request.statuses.iter().any(|s| normalize(s) == status);
        if !hit {
            return false;
        }
    }

    true
}

fn status_rank(status: &str) -> u8 {
    match normalize(status).as_str() {
        "online" => 0,
        "degraded" => 1,
        "maintenance" => 2,
        _ => 3,
    }
}

/// Public single-record helper: score a project against a raw query string.
/// Used by tests and by anyone poking at the engine from the console.
pub fn score_project(project: &ProjectRecord, text: &str) -> u32 {
    let tokens: Vec<String> = normalize(text)
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    if tokens.is_empty() {
        return 0;
    }

    let (total, _field) = score_tokens(project, &tokens);
    total.round().min(W_TITLE_EXACT) as u32
}

/// Shared by `score_project` and `rank`: returns the total plus the strongest
/// contributing field. `coverage` multipliers are applied here.
fn score_tokens(project: &ProjectRecord, tokens: &[String]) -> (f32, Field) {
    let mut total = 0.0f32;
    let mut best_score = 0.0f32;
    let mut best_field = Field::None;
    let mut matched = 0usize;

    for token in tokens {
        let (score, field) = score_token(project, token);
        if score > 0.0 {
            total += score;
            matched += 1;
            if score > best_score {
                best_score = score;
                best_field = field;
            }
        }
    }

    if matched == 0 {
        return (0.0, Field::None);
    }

    // Partial matches still surface, but weaker.
    let coverage = matched as f32 / tokens.len() as f32;
    total *= coverage;
    total += (matched as f32 * W_TOKEN_COVERAGE).min(W_TOKEN_COVERAGE_MAX);

    if project.featured {
        total += W_FEATURED;
    }
    if normalize(&project.status) == "online" {
        total += W_ONLINE;
    }

    (total, best_field)
}

/// Rank a full request. Exposed for native tests; the WASM entry point is the
/// thin JSON wrapper below.
pub fn rank(request: &SearchRequest) -> Vec<ScoredRow> {
    let tokens: Vec<String> = normalize(&request.text)
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect();

    let mut rows: Vec<(ScoredRow, &ProjectRecord)> = Vec::new();

    for project in &request.projects {
        if !passes_filters(project, request) {
            continue;
        }

        if tokens.is_empty() {
            rows.push((
                ScoredRow {
                    id: project.id.clone(),
                    score: 0,
                    matched_field: Field::None.as_str().to_string(),
                },
                project,
            ));
            continue;
        }

        let (total, field) = score_tokens(project, &tokens);
        if total <= 0.0 {
            continue;
        }

        rows.push((
            ScoredRow {
                id: project.id.clone(),
                score: total.round().min(W_TITLE_EXACT) as u32,
                matched_field: field.as_str().to_string(),
            },
            project,
        ));
    }

    let sort = normalize(&request.sort);
    match sort.as_str() {
        "title" => rows.sort_by(|a, b| {
            a.1.title
                .to_lowercase()
                .cmp(&b.1.title.to_lowercase())
                .then_with(|| a.0.id.cmp(&b.0.id))
        }),
        "latency" => rows.sort_by(|a, b| {
            // latency 0 means "no signal" → sort last, like the TS engine.
            let la = if a.1.latency == 0 { u32::MAX } else { a.1.latency };
            let lb = if b.1.latency == 0 { u32::MAX } else { b.1.latency };
            la.cmp(&lb).then_with(|| a.0.id.cmp(&b.0.id))
        }),
        "newest" => rows.sort_by(|a, b| {
            b.1.year
                .cmp(&a.1.year)
                .then_with(|| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
        }),
        // "relevance" and any unknown value.
        _ => rows.sort_by(|a, b| {
            b.0.score
                .cmp(&a.0.score)
                .then_with(|| status_rank(&a.1.status).cmp(&status_rank(&b.1.status)))
                .then_with(|| a.1.title.to_lowercase().cmp(&b.1.title.to_lowercase()))
        }),
    }

    rows.into_iter().map(|(row, _)| row).collect()
}

// ---------------------------------------------------------------------------
// WASM surface
// ---------------------------------------------------------------------------

/// Search + filter the project catalogue.
///
/// * Input: JSON matching [`SearchRequest`].
/// * Output: JSON array of `{ id, score, matched_field }`, already sorted.
///
/// Malformed input returns `"[]"` instead of throwing: a broken payload must
/// never take the grid down, and the frontend already treats an empty result
/// as "no matches".
#[wasm_bindgen]
pub fn search_projects(payload: &str) -> String {
    match serde_json::from_str::<SearchRequest>(payload) {
        Ok(request) => match serde_json::to_string(&rank(&request)) {
            Ok(json) => json,
            Err(_) => "[]".to_string(),
        },
        Err(_) => "[]".to_string(),
    }
}

/// Engine version, surfaced in the portal's telemetry footer so you can tell at
/// a glance whether the page is running the Rust engine or the TS fallback.
#[wasm_bindgen]
pub fn engine_version() -> String {
    ENGINE_VERSION.to_string()
}

/// Aggregate counts for the footer: `{"projects":8,"stacks":19,"categories":8}`.
#[wasm_bindgen]
pub fn index_stats(payload: &str) -> String {
    #[derive(Deserialize)]
    struct StatsRequest {
        #[serde(default)]
        projects: Vec<ProjectRecord>,
    }

    let Ok(request) = serde_json::from_str::<StatsRequest>(payload) else {
        return "{}".to_string();
    };

    let mut stacks: Vec<String> = request
        .projects
        .iter()
        .flat_map(|p| p.stack.iter().map(|s| normalize(s)))
        .collect();
    stacks.sort();
    stacks.dedup();

    let mut categories: Vec<String> = request
        .projects
        .iter()
        .flat_map(|p| p.categories.iter().map(|c| normalize(c)))
        .collect();
    categories.sort();
    categories.dedup();

    format!(
        r#"{{"projects":{},"stacks":{},"categories":{}}}"#,
        request.projects.len(),
        stacks.len(),
        categories.len()
    )
}

// ---------------------------------------------------------------------------
// Native tests — `cargo test` works without a wasm target.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn project(id: &str, title: &str, stack: &[&str], status: &str) -> ProjectRecord {
        ProjectRecord {
            id: id.to_string(),
            title: title.to_string(),
            pitch: format!("{title} does something useful"),
            description: "A longer description used for fuzzy fallbacks.".to_string(),
            stack: stack.iter().map(|s| s.to_string()).collect(),
            categories: vec!["web".to_string()],
            status: status.to_string(),
            latency: 20,
            featured: false,
            year: 2025,
        }
    }

    #[test]
    fn normalize_strips_case_and_accents() {
        assert_eq!(normalize("  CAFÉ  "), "cafe");
        assert_eq!(normalize("Mini-Shopify"), "mini-shopify");
    }

    /// The TS mirror uses `.normalize('NFD')` then strips combining marks, which
    /// also folds PRECOMPOSED letters. Rust's stdlib cannot do that, so
    /// `fold_latin` must stay in sync. This guards the contract that makes WASM
    /// and TS return the same ranking.
    ///
    /// `Zürich` and `Malmö` are here on purpose: an earlier version used char
    /// RANGES (`'ñ'..='ň'` covers U+00F1–U+0148), which swallowed `ö`/`ü`/`ý`
    /// and produced "znrich". Keep these assertions.
    #[test]
    fn normalize_matches_typescript_contract() {
        assert_eq!(normalize("CAFÉ"), "cafe");
        assert_eq!(normalize("naïve"), "naive");
        assert_eq!(normalize("Zürich"), "zurich");
        assert_eq!(normalize("Malmö"), "malmo");
        assert_eq!(normalize("Löwe"), "lowe");
        assert_eq!(normalize("Ångström"), "angstrom");
        assert_eq!(normalize("Señor"), "senor");
        assert_eq!(normalize("Dvořák"), "dvorak");
        assert_eq!(normalize("Reykjavík"), "reykjavik");
        assert_eq!(normalize("Besançon"), "besancon");
        assert_eq!(normalize("Crème Brûlée"), "creme brulee");

        // Letters JS's NFD does NOT decompose must survive intact, exactly as
        // they do in TypeScript. Folding these would desync the two engines.
        // NB: `ö` is NOT one of them — NFD decomposes it to o + U+0308, so
        // "Größe" correctly becomes "große" (only the ß survives).
        assert_eq!(normalize("Łódź"), "łodz");
        assert_eq!(normalize("Größe"), "große");
        assert_eq!(normalize("Đán"), "đan");

        // Already-decomposed input keeps working (NFD combining marks).
        assert_eq!(normalize("cafe\u{0301}"), "cafe");
        // Non-Latin text passes through untouched.
        assert_eq!(normalize("ワルン"), "ワルン");
    }

    #[test]
    fn fuzzy_ratio_rejects_non_subsequence() {
        assert_eq!(fuzzy_ratio("shopify", "xyz"), 0.0);
    }

    #[test]
    fn fuzzy_ratio_rewards_contiguous_match() {
        let contiguous = fuzzy_ratio("shopify", "shop");
        let scattered = fuzzy_ratio("shopify", "sop");
        assert!(contiguous > scattered, "{contiguous} vs {scattered}");
        assert!(contiguous > 0.0 && contiguous <= 1.0);
    }

    /// Regression: a leading offset used to dominate the span and sink
    /// single-typo queries below every threshold.
    #[test]
    fn fuzzy_ratio_ignores_leading_offset() {
        let ratio = fuzzy_ratio("mini-shopify", "shopfy");
        assert!(
            ratio > FUZZY_TITLE_MIN,
            "expected a usable ratio for a 1-char typo, got {ratio}"
        );
    }

    #[test]
    fn exact_title_outranks_stack_hit() {
        let exact = project("a", "rust", &["golang"], "online");
        let stacked = project("b", "telemetry daemon", &["rust"], "online");

        let exact_score = score_project(&exact, "rust");
        let stack_score = score_project(&stacked, "rust");
        assert!(exact_score > stack_score, "{exact_score} vs {stack_score}");
        assert_eq!(exact_score, W_TITLE_EXACT as u32);
    }

    #[test]
    fn filters_drop_non_matching_status_and_stack() {
        let request = SearchRequest {
            text: String::new(),
            stacks: vec!["rust".to_string()],
            categories: vec![],
            statuses: vec!["online".to_string()],
            featured_only: false,
            sort: "relevance".to_string(),
            projects: vec![
                project("keep", "Daemon", &["rust"], "online"),
                project("wrong-status", "Offline Thing", &["rust"], "offline"),
                project("wrong-stack", "React App", &["react"], "online"),
            ],
        };

        let ids: Vec<String> = rank(&request).into_iter().map(|r| r.id).collect();
        assert_eq!(ids, vec!["keep".to_string()]);
    }

    #[test]
    fn empty_text_returns_everything_that_passes_filters() {
        let request = SearchRequest {
            text: "   ".to_string(),
            stacks: vec![],
            categories: vec![],
            statuses: vec![],
            featured_only: false,
            sort: "title".to_string(),
            projects: vec![
                project("b", "Beta", &["go"], "online"),
                project("a", "Alpha", &["go"], "degraded"),
            ],
        };

        let ids: Vec<String> = rank(&request).into_iter().map(|r| r.id).collect();
        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn multi_token_query_requires_at_least_one_hit() {
        let request = SearchRequest {
            text: "nonexistent zzz".to_string(),
            stacks: vec![],
            categories: vec![],
            statuses: vec![],
            featured_only: false,
            sort: "relevance".to_string(),
            projects: vec![project("a", "Shopify Clone", &["react"], "online")],
        };

        assert!(rank(&request).is_empty());
    }

    #[test]
    fn relevance_ties_break_on_status_then_title() {
        let request = SearchRequest {
            text: "shopify".to_string(),
            stacks: vec![],
            categories: vec![],
            statuses: vec![],
            featured_only: false,
            sort: "relevance".to_string(),
            projects: vec![
                project("z", "Shopify Z", &["go"], "offline"),
                project("a", "Shopify A", &["go"], "online"),
            ],
        };

        let ids: Vec<String> = rank(&request).into_iter().map(|r| r.id).collect();
        assert_eq!(ids, vec!["a".to_string(), "z".to_string()]);
    }

    #[test]
    fn wasm_entry_point_survives_bad_json() {
        assert_eq!(search_projects("{not json"), "[]");
        assert_eq!(search_projects("{}"), "[]");
    }

    #[test]
    fn engine_version_is_exposed() {
        assert_eq!(engine_version(), ENGINE_VERSION);
    }

    #[test]
    fn index_stats_dedupes_stacks() {
        let payload = r#"{"projects":[
            {"id":"a","title":"A","stack":["Rust","Go"],"categories":["web"]},
            {"id":"b","title":"B","stack":["rust"],"categories":["Web","tools"]}
        ]}"#;
        let stats = index_stats(payload);
        assert_eq!(stats, r#"{"projects":2,"stacks":2,"categories":2}"#);
    }
}
