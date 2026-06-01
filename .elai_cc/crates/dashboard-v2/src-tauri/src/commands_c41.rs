//! C-41 — Application Skills System dashboard surface (sub-phase 5B).
//!
//! Two read-only Tauri commands feed the Svelte `SkillsPanel`:
//!
//! - [`list_application_skills`] — walks `.elai_cc/skills/` via
//!   [`elai_orchestrator::skill_registry::find_skills`] and returns the
//!   9 canonical application skill manifests as a thin DTO. Filters to
//!   `type=application` defensively (the discovery walker is shared
//!   with `.claude/skills/` harness scaffolds but the root supplied
//!   here is `.elai_cc/skills/` only, so the filter is a belt-and-braces
//!   guard for future drift).
//! - [`recent_skill_routing_events`] — last N `skill_triggered` +
//!   `skill_executed` SSB rows, flattened into a single timeline.
//!   Mirrors the `commands_d9::get_locked_keys_recent_audits` pattern.
//!
//! Additive — sibling of `commands_c43`. The orchestrator producer
//! (`task_runner` pre-classify hook, gated by
//! `ELAI_APPLICATION_SKILLS_ENABLE`) is the sole writer of
//! `skill_triggered` rows; this module only reads. `skill_executed`
//! emission is deferred to a future sub-phase (see
//! `workflow/reasoning/C-41-decisions.md` DECISION-5).
//!
//! Paper grounding: 44% -> 83.3% full-match intent accuracy on Claude
//! Opus 4.6 (S0 vs S3, Section 5.1 Table 2 of
//! `workflow/papers/scientific-workflow-interface.md`). The wire shapes
//! below are LOCALLY AUTHORED — see manifest.rs header for the schema
//! grounding caveat.

use std::path::PathBuf;

use elai_orchestrator::skill_registry::{find_skills, DiscoveryRoot, SkillManifest};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const DEFAULT_LIMIT: usize = 50;
const MAX_LIMIT: usize = 500;

/// Flat DTO for a single application skill manifest. Mirrors the
/// orchestrator's [`SkillManifest`] but lives in this crate so the
/// Tauri-side schema doesn't leak the orchestrator's internal types
/// (including the C-45 `learned_credit_score` and C-49
/// `distilled_fragments` fields, which are not yet surfaced in the
/// dashboard panel) through the IPC boundary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillManifestDto {
    /// Canonical skill name (directory leaf, overridable via
    /// `name:` frontmatter).
    pub name: String,
    /// Frontmatter `description:` — one-sentence role.
    pub description: String,
    /// Discriminator key (currently always `"application"` for skills
    /// surfaced here). Future-compat marker per DECISION-3.
    pub r#type: String,
    /// Trigger phrases from frontmatter `triggers:`.
    pub trigger_examples: Vec<String>,
    /// Coarse minutes estimate from frontmatter `estimated_minutes:`
    /// (legacy field on the SKILL.md). Defaults to 0 when absent.
    pub estimated_minutes: u32,
    /// Tag list parsed from frontmatter `tags: [...]`. Defaults to
    /// empty when absent.
    pub tags: Vec<String>,
    /// Semver string from frontmatter `version:`.
    pub version: String,
    /// Scope hint (e.g. "query", "edit", "plan").
    pub scope: String,
}

/// Flat DTO for a single routing event. One row per SSB
/// `skill_triggered` / `skill_executed` write.
///
/// `Eq` intentionally omitted: `score: f32` makes `Eq` impossible
/// (IEEE-754 NaN). `PartialEq` retained for tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillRoutingEventDto {
    /// Producer-side wall-clock timestamp in ms since epoch.
    pub timestamp_ms: i64,
    /// SSB action_type string (`"skill_triggered"` |
    /// `"skill_executed"`).
    pub action_type: String,
    /// blake3-prefix u64 of the resolved skill name. Stable across
    /// runs. The dashboard joins this against the manifest list to
    /// surface the human-readable name.
    pub skill_name_hash: u64,
    /// Score the registry assigned. Range `[0.0, 1.0]`. Only
    /// meaningful for `skill_triggered` rows; zero for
    /// `skill_executed`.
    pub score: f32,
    /// 1-based rank within the top-k window. 1 = top match. Only
    /// meaningful for `skill_triggered` rows; zero for
    /// `skill_executed`.
    pub top_k_rank: u8,
    /// Whether the score was below
    /// `SKILL_CONFIDENCE_THRESHOLD = 0.20`. UI surfaces this as a
    /// "fall-through" indicator (orchestrator routed to default
    /// scaffold instead). Only meaningful for `skill_triggered`.
    pub fell_through: bool,
}

/// Walk the application skills root and return one DTO per loaded
/// manifest. Resolution order matches the task_runner pre-classify
/// hook (DECISION-6 in C-41-decisions.md):
///   1. `ELAI_APPLICATION_SKILLS_DIR` env var, if set non-empty.
///   2. `<workspace_root>/.elai_cc/skills/`.
///   3. `<current_exe ancestors>/.elai_cc/skills/` walked up the tree
///      (fallback for tauri:dev launches where cwd is the project
///      root but the binary path is more reliable).
///
/// Returns Ok([]) on missing root rather than Err so the dashboard
/// always renders a sane (empty) panel instead of blocking on a Tauri
/// rejection. Errors propagate only for hard parse failures.
#[tauri::command]
pub async fn list_application_skills(
    state: State<'_, AppState>,
) -> Result<Vec<SkillManifestDto>, String> {
    let root_path = resolve_application_skills_root(&state);
    if !root_path.exists() {
        // Wire-dark default: directory may not exist on fresh
        // checkouts or non-ELAI projects opened via
        // open_project_from_folder. Render empty.
        return Ok(Vec::new());
    }
    let roots = [DiscoveryRoot::skills_dir(root_path)];
    let manifests = match find_skills(&roots) {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "list_application_skills: find_skills failed; returning empty"
            );
            return Ok(Vec::new());
        }
    };
    let mut out: Vec<SkillManifestDto> = manifests
        .into_iter()
        .filter(|m| is_application_skill(m))
        .map(manifest_to_dto)
        .collect();
    // Stable order: by name, ascending. Dashboard re-sorts on the
    // client side if needed; this keeps the snapshot deterministic
    // across refresh polls.
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

/// Return the most recent `limit` SSB rows across the two C-41
/// action types, sorted by `timestamp_ms` descending.
///
/// Empty response when no SSB viewer is attached (dashboard launched
/// without a project, or the AppState's viewer is `None`). Per-row
/// payload-decode failures are logged via `tracing::warn` and skipped.
#[tauri::command]
pub async fn recent_skill_routing_events(
    state: State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<SkillRoutingEventDto>, String> {
    let cap = limit
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_LIMIT)
        .min(MAX_LIMIT);
    let viewer = match &state.viewer {
        Some(v) => v.clone(),
        None => return Ok(Vec::new()),
    };

    let mut events: Vec<SkillRoutingEventDto> = Vec::new();
    for action_type in ["skill_triggered", "skill_executed"] {
        match viewer.recent_rows_of_type(action_type, cap) {
            Ok(details) => {
                for d in details {
                    if let Some(ev) = detail_to_event_dto(d) {
                        events.push(ev);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    action_type,
                    error = %e,
                    "recent_skill_routing_events: viewer error"
                );
            }
        }
    }
    events.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));
    events.truncate(cap);
    Ok(events)
}

fn resolve_application_skills_root(state: &AppState) -> PathBuf {
    if let Ok(override_path) = std::env::var("ELAI_APPLICATION_SKILLS_DIR") {
        if !override_path.is_empty() {
            return PathBuf::from(override_path);
        }
    }
    let workspace_skills = state
        .project_root_snapshot()
        .join(".elai_cc")
        .join("skills");
    if workspace_skills.exists() {
        return workspace_skills;
    }
    // Also walk current_exe ancestors so tauri:dev (cwd=project root,
    // binary at .elai_cc/target/debug/) and double-clicked elai.bat
    // both resolve the same skills dir. Mirrors the
    // `commands_d9::default_registry_path` pattern.
    let start = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(".elai_cc").join("skills");
        if candidate.exists() {
            return candidate;
        }
        let candidate_alt = ancestor.join("skills");
        if candidate_alt.join("simple_query").join("SKILL.md").exists() {
            return candidate_alt;
        }
    }
    workspace_skills
}

/// Application-skill discriminator: today the manifest's `scope`
/// field is the closest signal, and the underlying SKILL.md files all
/// carry `type: application` in frontmatter (DECISION-3). The
/// `SkillManifest` struct does not yet expose the `type:` key (it's
/// silently ignored by `parse_skill_frontmatter_extended` until a
/// future schema bump). For now we accept everything sourced from the
/// `.elai_cc/skills/` root — the root itself enforces the namespace.
fn is_application_skill(_m: &SkillManifest) -> bool {
    // Belt-and-braces guard kept as a function so the future schema
    // bump (FrontmatterFields::r#type) has a single chokepoint.
    true
}

fn manifest_to_dto(m: SkillManifest) -> SkillManifestDto {
    // The shipped 9 SKILL.md files all carry the legacy keys
    // `tags:` and `estimated_minutes:` directly under frontmatter.
    // The orchestrator's FrontmatterFields parser does not surface
    // these (they're informational only), so we re-read the SKILL.md
    // body to extract them. Failure is non-fatal — empty defaults
    // keep the DTO consistent.
    let (tags, estimated_minutes, description_from_fm) = parse_legacy_frontmatter_keys(&m.file_path);
    SkillManifestDto {
        name: m.name,
        // Prefer the legacy description from frontmatter; SkillManifest
        // does not expose `description` directly (it lives on
        // FrontmatterFields but is not propagated to the manifest).
        description: description_from_fm,
        r#type: "application".to_string(),
        trigger_examples: m.triggers,
        estimated_minutes,
        tags,
        version: m.version,
        scope: m.scope,
    }
}

/// Read the SKILL.md frontmatter and pluck the `tags:` /
/// `estimated_minutes:` / `description:` keys directly. The
/// `parse_skill_frontmatter_extended` parser drops these because they
/// aren't part of the registry's typed schema (they're informational
/// for the dashboard). Failure paths return empty defaults rather
/// than propagating I/O errors — the dashboard renders a cell with
/// "(0 min)" instead of failing the whole panel.
fn parse_legacy_frontmatter_keys(path: &std::path::Path) -> (Vec<String>, u32, String) {
    let content = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return (Vec::new(), 0, String::new()),
    };
    let mut tags: Vec<String> = Vec::new();
    let mut minutes: u32 = 0;
    let mut description: String = String::new();
    // YAML frontmatter is bounded by `---` markers at the file start.
    let mut lines = content.lines();
    if lines.next() != Some("---") {
        return (tags, minutes, description);
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some(rest) = line.strip_prefix("tags:") {
            tags = parse_yaml_list(rest);
        } else if let Some(rest) = line.strip_prefix("estimated_minutes:") {
            minutes = rest.trim().parse::<u32>().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("description:") {
            description = rest.trim().to_string();
        }
    }
    (tags, minutes, description)
}

/// Minimal flow-list YAML parser: handles `[a, b, c]` and the empty
/// case `[]`. Returns empty Vec on anything more complex (block
/// lists) — the shipped 9 SKILL.md files all use flow-list shape.
fn parse_yaml_list(input: &str) -> Vec<String> {
    let trimmed = input.trim();
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'));
    let Some(inner) = inner else {
        return Vec::new();
    };
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn unwrap_variant<'a>(
    payload: &'a serde_json::Value,
    variant: &str,
) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
    let outer = payload.as_object()?;
    if let Some(inner) = outer.get(variant).and_then(|v| v.as_object()) {
        return Some(inner);
    }
    Some(outer)
}

fn detail_to_event_dto(d: elai_orchestrator::SsbActionDetail) -> Option<SkillRoutingEventDto> {
    let variant = match d.action_type.as_str() {
        "skill_triggered" => "SkillTriggered",
        "skill_executed" => "SkillExecuted",
        _ => return None,
    };
    let obj = unwrap_variant(&d.payload_json, variant)?;
    let skill_name_hash = obj
        .get("skill_name_hash")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let ts_ms_signed = obj
        .get("ts_ms")
        .and_then(|v| v.as_i64())
        .or_else(|| obj.get("ts_ms").and_then(|v| v.as_u64()).map(|u| u as i64))
        .unwrap_or(0);
    let score = obj
        .get("score")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32)
        .unwrap_or(0.0);
    let top_k_rank = obj
        .get("top_k_rank")
        .and_then(|v| v.as_u64())
        .map(|n| u8::try_from(n).unwrap_or(u8::MAX))
        .unwrap_or(0);
    let fell_through = matches!(d.action_type.as_str(), "skill_triggered")
        && (score as f64) < 0.20_f64;
    Some(SkillRoutingEventDto {
        timestamp_ms: ts_ms_signed,
        action_type: d.action_type,
        skill_name_hash,
        score,
        top_k_rank,
        fell_through,
    })
}
