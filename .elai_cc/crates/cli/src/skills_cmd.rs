//! E-43 -- elai skills <list|eval|publish> handler.
//!
//! Three sub-actions:
//!   - list: enumerate SKILL.md files via C-41 SkillRegistry.
//!   - eval: precision/recall via lexical dry-run harness.
//!   - publish: copy SKILL.md to dist/<target>/skills/<name>/.
//!
//! ABSORBs the E-22 SKILL.md export shim per E-43 plan line 20.
//! No .unwrap() in production code paths.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::skill_registry::{find_skills, DiscoveryRoot, SkillOrigin};
use elai_orchestrator::{
    LearnedSkill, SkillStore, SkillTrustTier, Ssb, SsbConfig, SsbPayload, TIER_MIN_AGE_SECS,
    TIER_MIN_INVOCATIONS,
};
use elai_orchestrator::ssb::{SkillLibraryEventKind, SkillLibraryEventPayload};

use crate::args::{CliOutputFormat, SkillsAction};

pub(crate) fn run(action: SkillsAction, output_format: CliOutputFormat) -> Result<(), String> {
    match action {
        SkillsAction::List => run_list(output_format),
        SkillsAction::Eval { skill, dry_run } => run_eval(skill.as_deref(), dry_run, output_format),
        SkillsAction::Publish { target, dry_run } => run_publish(target, dry_run, output_format),
        // E-22 (SCHEMA_VERSION 170): normalised SKILL.md export with
        // blake3 drift detection. See workflow/reasoning/E-22-decisions.md.
        SkillsAction::Export { target, dry_run } => run_export(target, dry_run, output_format),
        // F-6 / Slice A4 (C6c): learned-skill library trust-ladder surface.
        // See workflow/reasoning/F-6-decisions.md.
        SkillsAction::LibraryList => run_library_list(output_format),
        SkillsAction::LibraryPromote { id, human_grant } => {
            run_library_promote(&id, human_grant, output_format)
        }
        SkillsAction::LibraryShow { id } => run_library_show(&id, output_format),
        SkillsAction::LibraryTick => run_library_tick(output_format),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublishTarget {
    Claude,
    Codex,
    Gemini,
}

impl PublishTarget {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Claude => "claude",
            Self::Codex => "codex",
            Self::Gemini => "gemini",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s.trim().to_ascii_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "codex" => Ok(Self::Codex),
            "gemini" => Ok(Self::Gemini),
            other => Err(format!("unknown publish target `{other}`; expected claude|codex|gemini")),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalReport {
    pub skill: String,
    pub precision: f32,
    pub recall: f32,
    pub n_positive: usize,
    pub n_negative: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishReport {
    pub target: String,
    pub emitted: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvalCase {
    prompt: String,
    expected_trigger: bool,
    #[allow(dead_code)]
    rationale: Option<String>,
}

fn project_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn skills_roots() -> Vec<DiscoveryRoot> {
    let root = project_root();
    vec![
        DiscoveryRoot::skills_dir(root.join(".claude").join("skills")),
        DiscoveryRoot::skills_dir(root.join(".elai_cc").join("skills")),
    ]
}

fn run_list(output_format: CliOutputFormat) -> Result<(), String> {
    let manifests = find_skills(&skills_roots())
        .map_err(|e| format!("skills list failed: {e}"))?;
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&manifests).unwrap_or_else(|_| "[]".into());
            println!("{json}");
        }
        _ => {
            println!("found {} skills:", manifests.len());
            for m in &manifests {
                let origin = match m.origin {
                    SkillOrigin::SkillsDir => "skills_dir",
                    SkillOrigin::LegacyCommandsDir => "legacy_commands_dir",
                };
                println!(
                    "  {name:<22} v{version} [{origin}] {path}",
                    name = m.name,
                    version = m.version,
                    origin = origin,
                    path = m.file_path.display(),
                );
            }
        }
    }
    Ok(())
}

fn run_eval(
    skill_filter: Option<&str>,
    _dry_run: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let manifests = find_skills(&skills_roots())
        .map_err(|e| format!("skills eval discovery failed: {e}"))?;
    let mut reports: Vec<EvalReport> = Vec::new();
    for m in &manifests {
        if let Some(filter) = skill_filter {
            if m.name != filter { continue; }
        }
        let skill_dir = match m.file_path.parent() {
            Some(d) => d.to_path_buf(),
            None => continue,
        };
        let eval_dir = skill_dir.join("eval");
        let positive_path = eval_dir.join("positive.jsonl");
        let negative_path = eval_dir.join("negative.jsonl");
        if !positive_path.exists() || !negative_path.exists() {
            reports.push(EvalReport {
                skill: m.name.clone(),
                precision: 0.0,
                recall: 0.0,
                n_positive: 0,
                n_negative: 0,
            });
            continue;
        }
        let positives = load_eval_jsonl(&positive_path)?;
        let negatives = load_eval_jsonl(&negative_path)?;
        let description = read_description(&m.file_path)?;
        let mut tp: u32 = 0;
        let mut fn_count: u32 = 0;
        for case in &positives {
            if lexical_match(&description, &case.prompt) { tp += 1; } else { fn_count += 1; }
        }
        let mut fp: u32 = 0;
        for case in &negatives {
            if lexical_match(&description, &case.prompt) { fp += 1; }
        }
        let precision = if tp + fp == 0 { 0.0 } else { tp as f32 / (tp + fp) as f32 };
        let recall = if tp + fn_count == 0 { 0.0 } else { tp as f32 / (tp + fn_count) as f32 };
        reports.push(EvalReport {
            skill: m.name.clone(),
            precision,
            recall,
            n_positive: positives.len(),
            n_negative: negatives.len(),
        });
    }

    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&reports).unwrap_or_else(|_| "[]".into());
            println!("{json}");
        }
        _ => {
            println!("skill                   precision  recall   n_pos  n_neg");
            for r in &reports {
                println!(
                    "  {:<22} {:>9.2}  {:>6.2}  {:>5}  {:>5}",
                    r.skill, r.precision, r.recall, r.n_positive, r.n_negative
                );
            }
        }
    }
    Ok(())
}

fn run_publish(
    target: String,
    dry_run: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let target = PublishTarget::parse(&target)?;
    let manifests = find_skills(&skills_roots())
        .map_err(|e| format!("skills publish discovery failed: {e}"))?;
    let mut emitted: Vec<PathBuf> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let dist_root = project_root().join("dist").join(target.as_str()).join("skills");
    for m in &manifests {
        let body = match fs::read_to_string(&m.file_path) {
            Ok(s) => s,
            Err(e) => {
                warnings.push(format!("skip {}: cannot read {}: {e}", m.name, m.file_path.display()));
                continue;
            }
        };
        let dest_dir = dist_root.join(&m.name);
        let dest_file = dest_dir.join("SKILL.md");
        if dry_run {
            emitted.push(dest_file);
            continue;
        }
        if let Err(e) = fs::create_dir_all(&dest_dir) {
            warnings.push(format!("skip {}: cannot create {}: {e}", m.name, dest_dir.display()));
            continue;
        }
        if let Err(e) = fs::write(&dest_file, body.as_bytes()) {
            warnings.push(format!("skip {}: cannot write {}: {e}", m.name, dest_file.display()));
            continue;
        }
        emitted.push(dest_file);
        if matches!(target, PublishTarget::Codex) {
            let codex_meta_dir = dest_dir.join("agents");
            if let Err(e) = fs::create_dir_all(&codex_meta_dir) {
                warnings.push(format!("codex meta skip {}: cannot create {}: {e}", m.name, codex_meta_dir.display()));
            } else {
                let meta_file = codex_meta_dir.join("openai.yaml");
                let meta_body = format!(
                    "# E-43 publish stub (Codex)
name: {}
version: {}
elai_origin: skills_dir
",
                    m.name, m.version
                );
                if let Err(e) = fs::write(&meta_file, meta_body.as_bytes()) {
                    warnings.push(format!("codex meta skip {}: cannot write {}: {e}", m.name, meta_file.display()));
                } else {
                    emitted.push(meta_file);
                }
            }
        }
    }

    let report = PublishReport {
        target: target.as_str().to_owned(),
        emitted,
        warnings,
    };
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!("publish target: {}", report.target);
            println!("emitted {} files", report.emitted.len());
            for p in &report.emitted { println!("  {}", p.display()); }
            if !report.warnings.is_empty() {
                println!("warnings ({}):", report.warnings.len());
                for w in &report.warnings { println!("  {w}"); }
            }
        }
    }
    Ok(())
}

fn load_eval_jsonl(path: &Path) -> Result<Vec<EvalCase>, String> {
    let body = fs::read_to_string(path)
        .map_err(|e| format!("read {} failed: {e}", path.display()))?;
    let mut cases: Vec<EvalCase> = Vec::new();
    for (i, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
        let case: EvalCase = serde_json::from_str(trimmed)
            .map_err(|e| format!("{}:{}: jsonl parse failed: {e}", path.display(), i + 1))?;
        cases.push(case);
    }
    Ok(cases)
}

fn read_description(skill_md_path: &Path) -> Result<String, String> {
    let body = fs::read_to_string(skill_md_path)
        .map_err(|e| format!("read {} failed: {e}", skill_md_path.display()))?;
    let mut in_frontmatter = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if !in_frontmatter { in_frontmatter = true; continue; } else { break; }
        }
        if !in_frontmatter { continue; }
        if let Some(rest) = trimmed.strip_prefix("description:") {
            return Ok(rest.trim().to_ascii_lowercase());
        }
    }
    Ok(String::new())
}

fn lexical_match(description: &str, prompt: &str) -> bool {
    let desc_tokens: HashSet<String> = description
        .split_whitespace()
        .filter(|t| t.len() >= 4)
        .map(tok_norm)
        .filter(|t| !t.is_empty())
        .collect();
    let prompt_lower = prompt.to_ascii_lowercase();
    let prompt_tokens: HashSet<String> = prompt_lower
        .split_whitespace()
        .filter(|t| t.len() >= 4)
        .map(tok_norm)
        .filter(|t| !t.is_empty())
        .collect();
    prompt_tokens.intersection(&desc_tokens).count() >= 2
}

fn tok_norm(t: &str) -> String {
    t.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_ascii_lowercase()
}

// ---------------------------------------------------------------------------
// E-22 (SCHEMA_VERSION 170) -- normalised SKILL.md export with blake3 drift
// detection. The export action is distinct from publish in two ways:
//   1. Frontmatter keys are sorted into a canonical order BEFORE write.
//   2. A blake3 8-byte prefix diff against any prior dist copy reports a
//      drift_count (0 = clean export; >0 = drift). Idempotent: re-running
//      on a clean tree yields drift_count=0.
// ---------------------------------------------------------------------------

/// Canonical frontmatter key order per the Anthropic SKILL.md spec
/// (code.claude.com/docs/en/skills, VERIFIED 2026-05-27). Keys not in
/// this list are appended in their original encounter order so the
/// normalization is stable.
const CANONICAL_FRONTMATTER_KEYS: &[&str] = &[
    "name",
    "description",
    "version",
    "model",
    "triggers",
    "inputs",
    "outputs",
    "scope",
    "allowed_tools",
];

/// E-22 -- structured export report. Returned to the caller for JSON
/// rendering + the `GithubTrendingEvent::SkillExportNormalized` SSB row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportReport {
    pub target: String,
    pub exported: Vec<PathBuf>,
    pub drift: Vec<String>,
    pub warnings: Vec<String>,
}

/// E-22 -- producer-side emitter for the
/// `GithubTrendingEvent::SkillExportNormalized` SSB row. Lives in
/// `skills_cmd.rs` rather than `crate::ssb` to keep the CLI crate
/// dep-arrow-free; orchestrator-side bootstrap wires a real
/// implementor when an SSB writer is available, while CLI default
/// paths use `NoopSkillExportNormalizedEmitter` so a plain
/// `elai skills export --target=claude` does not require an
/// orchestrator session. Pattern parallels
/// [`crate::orchestrator::sandbox::MicroVmFallbackEmitter`] +
/// [`elai_plugins::mcp_bridge::LazySchemaSavingsEmitter`].
pub trait SkillExportNormalizedEmitter: Send + Sync {
    /// Emit one `github_trending_event` row with kind=
    /// `skill_export_normalized`. `skill_name` is the kebab-case
    /// canonical name (matches dir leaf); `drift_count` is the
    /// blake3 8-byte prefix diff count (0 = clean export,
    /// >0 = drift detected); `dist_path` is the absolute path the
    /// normalised SKILL.md was written to.
    fn emit(&self, skill_name: &str, drift_count: u32, dist_path: &Path) -> Result<(), String>;
}

/// Default no-op emitter -- returns Ok(()) without touching anything.
/// Used when `run_export` is invoked from the CLI without an
/// orchestrator-supplied emitter.
pub struct NoopSkillExportNormalizedEmitter;

impl SkillExportNormalizedEmitter for NoopSkillExportNormalizedEmitter {
    fn emit(&self, _skill_name: &str, _drift_count: u32, _dist_path: &Path) -> Result<(), String> {
        Ok(())
    }
}

/// E-22 -- read the github trending env gate. Returns true only when
/// the env var is set to `"1"` or `"true"` (case-insensitive). Mirrors
/// the orchestrator `ELAI_GITHUB_TRENDING_ENABLE_ENV` const; the CLI
/// keeps its own reader so this module does not need a circular dep
/// on `elai_orchestrator::ssb`. Wire-dark default keeps existing
/// `elai skills export` invocations side-effect-free.
fn github_trending_enabled() -> bool {
    match std::env::var("ELAI_GITHUB_TRENDING_ENABLE") {
        Ok(v) => {
            let lo = v.trim().to_ascii_lowercase();
            lo == "1" || lo == "true"
        }
        Err(_) => false,
    }
}

fn run_export(target: String, dry_run: bool, output_format: CliOutputFormat) -> Result<(), String> {
    run_export_with_emitter(target, dry_run, output_format, &NoopSkillExportNormalizedEmitter)
}

/// E-22 -- dep-inverted entry point used by orchestrator bootstrap when
/// it has an SSB-backed `SkillExportNormalizedEmitter`. The plain CLI
/// dispatcher uses `NoopSkillExportNormalizedEmitter`; the live
/// orchestrator path supplies a real emitter that writes
/// `GithubTrendingEvent::SkillExportNormalized` rows when
/// `ELAI_GITHUB_TRENDING_ENABLE=1`. Per-skill emit happens after each
/// normalised SKILL.md is written (or after the would-be-write under
/// `--dry-run`).
pub fn run_export_with_emitter(
    target: String,
    dry_run: bool,
    output_format: CliOutputFormat,
    emitter: &dyn SkillExportNormalizedEmitter,
) -> Result<(), String> {
    let target = PublishTarget::parse(&target)?;
    let manifests = find_skills(&skills_roots())
        .map_err(|e| format!("skills export discovery failed: {e}"))?;
    let mut exported: Vec<PathBuf> = Vec::new();
    let mut drift: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();
    let dist_root = project_root().join("dist").join(target.as_str()).join("skills");
    let trending_enabled = github_trending_enabled();

    for m in &manifests {
        let body = match fs::read_to_string(&m.file_path) {
            Ok(s) => s,
            Err(e) => {
                warnings.push(format!(
                    "skip {}: cannot read {}: {e}",
                    m.name,
                    m.file_path.display()
                ));
                continue;
            }
        };
        let normalised = normalize_skill_md(&body);
        let dest_dir = dist_root.join(&m.name);
        let dest_file = dest_dir.join("SKILL.md");

        // Drift detection: compare blake3 8-byte prefix of normalised
        // body to any prior dist copy. drift_count=1 when they differ,
        // 0 otherwise (or 0 when no prior copy exists).
        let prior_hash = if dest_file.exists() {
            fs::read_to_string(&dest_file).ok().map(|s| blake3_prefix(&s))
        } else {
            None
        };
        let new_hash = blake3_prefix(&normalised);
        let drift_count: u32 = match prior_hash {
            Some(prior) if prior != new_hash => 1,
            _ => 0,
        };
        if drift_count > 0 {
            drift.push(m.name.clone());
        }

        if dry_run {
            exported.push(dest_file.clone());
            // E-22 -- wire-dark emit on dry-run as well (the trending
            // env gate ALWAYS keeps emitter call wire-dark unless set).
            if trending_enabled {
                if let Err(e) = emitter.emit(&m.name, drift_count, &dest_file) {
                    warnings.push(format!(
                        "skill_export_normalized emit failed for {}: {e}",
                        m.name
                    ));
                }
            }
            continue;
        }
        if let Err(e) = fs::create_dir_all(&dest_dir) {
            warnings.push(format!("skip {}: cannot create {}: {e}", m.name, dest_dir.display()));
            continue;
        }
        if let Err(e) = fs::write(&dest_file, normalised.as_bytes()) {
            warnings.push(format!(
                "skip {}: cannot write {}: {e}",
                m.name,
                dest_file.display()
            ));
            continue;
        }
        // E-22 -- emit SkillExportNormalized SSB row gated by
        // ELAI_GITHUB_TRENDING_ENABLE. dist_path carries the
        // absolute path; downstream consumers can render the
        // exported tile + drift indicator off the row.
        if trending_enabled {
            if let Err(e) = emitter.emit(&m.name, drift_count, &dest_file) {
                warnings.push(format!(
                    "skill_export_normalized emit failed for {}: {e}",
                    m.name
                ));
            }
        }
        exported.push(dest_file);
    }

    let report = ExportReport {
        target: target.as_str().to_owned(),
        exported,
        drift,
        warnings,
    };
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!("export target: {}", report.target);
            println!("exported {} files", report.exported.len());
            for p in &report.exported {
                println!("  {}", p.display());
            }
            if !report.drift.is_empty() {
                println!("drift detected on {} skills:", report.drift.len());
                for d in &report.drift {
                    println!("  {d}");
                }
            }
            if !report.warnings.is_empty() {
                println!("warnings ({}):", report.warnings.len());
                for w in &report.warnings {
                    println!("  {w}");
                }
            }
        }
    }
    Ok(())
}

/// E-22 -- normalise a SKILL.md body. Frontmatter (between leading
/// `---` markers) gets keys sorted into [`CANONICAL_FRONTMATTER_KEYS`]
/// order; keys not in the canonical list are appended in their
/// original encounter order. Body content is preserved byte-for-byte
/// (only the frontmatter is touched). Returns the input unchanged when
/// the file has no frontmatter.
fn normalize_skill_md(body: &str) -> String {
    let mut lines = body.lines();
    let first = match lines.next() {
        Some(l) => l,
        None => return body.to_owned(),
    };
    if first.trim() != "---" {
        // No frontmatter: return unchanged.
        return body.to_owned();
    }
    // Collect frontmatter lines until the closing `---`.
    let mut fm_lines: Vec<&str> = Vec::new();
    let mut closing_found = false;
    let mut body_start_offset: usize = first.len() + 1; // skip leading '---\n'
    for line in lines.by_ref() {
        if line.trim() == "---" {
            closing_found = true;
            body_start_offset += line.len() + 1;
            break;
        }
        body_start_offset += line.len() + 1;
        fm_lines.push(line);
    }
    if !closing_found {
        return body.to_owned();
    }

    // Parse frontmatter into ordered (key, full_line) pairs. Multi-line
    // values (continuation lines that start with whitespace) attach to
    // the preceding key.
    let mut entries: Vec<(String, String)> = Vec::new();
    for line in &fm_lines {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(last) = entries.last_mut() {
                last.1.push('\n');
                last.1.push_str(line);
                continue;
            }
        }
        let trimmed = line.trim_start();
        if let Some(colon_idx) = trimmed.find(':') {
            let key = trimmed[..colon_idx].trim().to_string();
            entries.push((key, line.to_string()));
        } else {
            // Non-key line outside any continuation -- keep as anonymous
            // entry so we never drop content silently.
            entries.push((String::new(), line.to_string()));
        }
    }

    // Sort by canonical order, then by encounter order for unknowns.
    let mut canonical: Vec<(String, String)> = Vec::new();
    let mut leftover: Vec<(String, String)> = Vec::new();
    for canonical_key in CANONICAL_FRONTMATTER_KEYS {
        if let Some(idx) = entries.iter().position(|(k, _)| k == canonical_key) {
            canonical.push(entries.remove(idx));
        }
    }
    leftover.extend(entries.into_iter());

    let mut out = String::with_capacity(body.len());
    out.push_str("---\n");
    for (_k, line) in canonical.iter().chain(leftover.iter()) {
        out.push_str(line);
        out.push('\n');
    }
    out.push_str("---\n");
    // Append the remainder of the body verbatim.
    out.push_str(&body[body_start_offset.min(body.len())..]);
    out
}

/// E-22 -- compute blake3 8-byte hex prefix of a string. Used for
/// idempotent drift detection between the prior dist copy and the
/// newly-normalised body. The 8-byte prefix is sufficient for drift
/// detection (collision risk = 2^-32 per pair; we only need pair
/// stability not collision-freedom across a population).
fn blake3_prefix(s: &str) -> String {
    let h = blake3::hash(s.as_bytes());
    let bytes = h.as_bytes();
    // 8 bytes = 16 hex chars.
    let mut out = String::with_capacity(16);
    for b in &bytes[..8] {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

// ---------------------------------------------------------------------------
// F-6 / Slice A4 (C6c) -- learned-skill library trust-ladder surface.
//
// `elai skills-mgmt library <list|promote|show|tick>`.
//
// The learned-skill SkillStore is an in-memory authority owned by a live
// task-runner; a one-shot `elai` CLI invocation does NOT share that runtime's
// store. So, mirroring the routing_cmd idiom ("re-run the harness so the
// surface has a verifiable observable effect even before live state exists"),
// each verb seeds a deterministic demonstration store. The OBSERVABLE EFFECT
// of `promote`/`tick` is the SSB row written to the session ssb.db plus the
// stdout report; `list`/`show` render the seeded store. Live wiring to a
// persistent store is a follow-on (the store is process-local per F-6).
// ---------------------------------------------------------------------------

/// Stable ids for the seeded demonstration skills. Deterministic so the CLI
/// surface is reproducible run-to-run.
const DEMO_SKILL_FRESH: &str = "scaffold-rest-endpoint";
const DEMO_SKILL_STALE: &str = "migrate-legacy-config";

/// Serializable view of one learned skill for JSON output + the text table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLibraryView {
    pub id: String,
    pub description: String,
    pub tier: String,
    pub invocations: u32,
    pub failures: u32,
    pub failure_rate: f64,
    pub acquired_at: u64,
    pub last_verified: u64,
    pub staleness_score: f64,
}

impl SkillLibraryView {
    fn from_skill(skill: &LearnedSkill, now: u64) -> Self {
        SkillLibraryView {
            id: skill.id.clone(),
            description: skill.description.clone(),
            tier: skill.tier.as_str().to_owned(),
            invocations: skill.invocations,
            failures: skill.failures,
            failure_rate: skill.failure_rate(),
            acquired_at: skill.acquired_at,
            last_verified: skill.last_verified,
            staleness_score: SkillStore::staleness_score(skill, now),
        }
    }
}

/// Seed a deterministic demonstration [`SkillStore`]: one fresh, climbable
/// S2 skill (recently verified, already past the min-age + min-invocation
/// gate so `promote` can advance it) and one stale S3 skill (`last_verified`
/// far in the past so a `tick` demotes it). `now` is the wall-clock second.
fn seed_demo_store(now: u64) -> SkillStore {
    let mut store = SkillStore::new();
    // Fresh, climbable S2: old enough + enough invocations to clear the gate.
    store.upsert(LearnedSkill {
        id: DEMO_SKILL_FRESH.to_owned(),
        description: "scaffold a REST endpoint from an OpenAPI fragment".to_owned(),
        tier: SkillTrustTier::S2,
        invocations: TIER_MIN_INVOCATIONS,
        failures: 0,
        acquired_at: now.saturating_sub(TIER_MIN_AGE_SECS.saturating_add(1)),
        last_verified: now,
    });
    // Stale S3: last verified at the epoch floor, so its staleness score
    // crosses the demotion threshold under any realistic wall clock.
    store.upsert(LearnedSkill {
        id: DEMO_SKILL_STALE.to_owned(),
        description: "migrate a legacy config file to the v2 schema".to_owned(),
        tier: SkillTrustTier::S3,
        invocations: TIER_MIN_INVOCATIONS,
        failures: 0,
        acquired_at: 0,
        last_verified: 0,
    });
    store
}

/// Open the session `ssb.db` and emit ONE `SkillLibraryEvent` row, awaited
/// (durable). `Ssb::new` spawns the single-writer task and MUST be constructed
/// inside `rt.block_on`, else it panics "no reactor running" (observable only
/// on a live run; cargo check passes regardless). Mirrors
/// `bench_seccodeplt::emit_insecure_rate_measured`.
fn emit_skill_library_event(payload: SkillLibraryEventPayload) -> Result<(), String> {
    let db_path = sessions_dir_path(&project_root()).join("ssb.db");
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("library: cannot create session dir {}: {e}", parent.display()))?;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_owned(),
        session_id: "cli-skills-library".to_owned(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("library: tokio runtime build failed: {e}"))?;
    rt.block_on(async move {
        let ssb = Ssb::new(cfg, clock).map_err(|e| format!("library: SSB open failed: {e}"))?;
        ssb.write_payload(
            "cli:skills-library",
            0,
            SsbPayload::SkillLibraryEvent(payload),
            Vec::new(),
        )
        .await
        .map_err(|e| format!("library: SSB write failed: {e}"))?;
        Ok::<(), String>(())
    })
}

fn run_library_list(output_format: CliOutputFormat) -> Result<(), String> {
    let now = OsClock.now_secs();
    let store = seed_demo_store(now);
    let views: Vec<SkillLibraryView> = store
        .skills()
        .values()
        .map(|s| SkillLibraryView::from_skill(s, now))
        .collect();
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&views).unwrap_or_else(|_| "[]".into());
            println!("{json}");
        }
        _ => {
            println!("learned skill library ({} skills):", views.len());
            println!("  id                       tier  inv  fail  fail_rate  staleness");
            for v in &views {
                println!(
                    "  {:<24} {:<4}  {:>3}  {:>4}  {:>9.3}  {:>9.3}",
                    v.id, v.tier, v.invocations, v.failures, v.failure_rate, v.staleness_score
                );
            }
        }
    }
    Ok(())
}

fn run_library_show(id: &str, output_format: CliOutputFormat) -> Result<(), String> {
    let now = OsClock.now_secs();
    let store = seed_demo_store(now);
    let skill = store
        .get(id)
        .ok_or_else(|| format!("learned skill not found: {id}"))?;
    let view = SkillLibraryView::from_skill(skill, now);
    match output_format {
        CliOutputFormat::Json => {
            let json = serde_json::to_string_pretty(&view).unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!("learned skill: {}", view.id);
            println!("  description    {}", view.description);
            println!("  tier           {}", view.tier);
            println!("  invocations    {}", view.invocations);
            println!("  failures       {}", view.failures);
            println!("  failure_rate   {:.3}", view.failure_rate);
            println!("  acquired_at    {}", view.acquired_at);
            println!("  last_verified  {}", view.last_verified);
            println!("  staleness      {:.3}", view.staleness_score);
        }
    }
    Ok(())
}

fn run_library_promote(
    id: &str,
    human_grant: bool,
    output_format: CliOutputFormat,
) -> Result<(), String> {
    let now = OsClock.now_secs();
    let mut store = seed_demo_store(now);
    let prior_tier = store
        .get(id)
        .ok_or_else(|| format!("learned skill not found: {id}"))?
        .tier;
    let new_tier = store
        .promote(id, now, human_grant)
        .map_err(|e| format!("library promote `{id}` rejected: {e}"))?;
    // Snapshot the post-promotion counters for the SSB row.
    let (invocations, failure_rate, description) = match store.get(id) {
        Some(s) => (s.invocations, s.failure_rate(), s.description.clone()),
        None => (0, 0.0, String::new()),
    };
    emit_skill_library_event(SkillLibraryEventPayload {
        skill_id: id.to_owned(),
        description,
        tier: new_tier.as_str().to_owned(),
        prior_tier: Some(prior_tier.as_str().to_owned()),
        reason: if human_grant {
            "human_granted_promotion".to_owned()
        } else {
            "cleared_invocation_failure_age_gate".to_owned()
        },
        invocations,
        failure_rate,
        kind: SkillLibraryEventKind::TrustPromoted,
    })?;
    match output_format {
        CliOutputFormat::Json => {
            let view = SkillLibraryView::from_skill(
                store.get(id).ok_or_else(|| "promoted skill vanished".to_string())?,
                now,
            );
            let json = serde_json::to_string_pretty(&view).unwrap_or_else(|_| "{}".into());
            println!("{json}");
        }
        _ => {
            println!(
                "promoted `{id}`: {} -> {} (human_grant={human_grant})",
                prior_tier.as_str(),
                new_tier.as_str()
            );
        }
    }
    Ok(())
}

fn run_library_tick(output_format: CliOutputFormat) -> Result<(), String> {
    let now = OsClock.now_secs();
    let mut store = seed_demo_store(now);
    let demoted = store.demote_if_stale(now);
    for record in &demoted {
        emit_skill_library_event(SkillLibraryEventPayload {
            skill_id: record.id.clone(),
            description: format!("staleness demotion for learned skill '{}'", record.id),
            tier: record.new_tier.as_str().to_owned(),
            prior_tier: Some(record.prior_tier.as_str().to_owned()),
            reason: "staleness_score_crossed_threshold".to_owned(),
            invocations: 0,
            failure_rate: 0.0,
            kind: SkillLibraryEventKind::DemotedStale,
        })?;
    }
    match output_format {
        CliOutputFormat::Json => {
            let views: Vec<SkillLibraryView> = demoted
                .iter()
                .filter_map(|d| store.get(&d.id))
                .map(|s| SkillLibraryView::from_skill(s, now))
                .collect();
            let json = serde_json::to_string_pretty(&views).unwrap_or_else(|_| "[]".into());
            println!("{json}");
        }
        _ => {
            println!("staleness tick: demoted {} skills", demoted.len());
            for d in &demoted {
                println!(
                    "  {}: {} -> {} (staleness {:.3})",
                    d.id,
                    d.prior_tier.as_str(),
                    d.new_tier.as_str(),
                    d.staleness_score
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_demo_store_has_fresh_and_stale_skills() {
        // Use a realistic wall-clock value so the fresh skill is past the
        // min-age gate and the stale skill's score crosses the threshold.
        let now: u64 = 1_900_000_000;
        let store = seed_demo_store(now);
        assert_eq!(store.len(), 2, "two demo skills seeded");
        let fresh = store.get(DEMO_SKILL_FRESH).expect("fresh present");
        assert_eq!(fresh.tier, SkillTrustTier::S2);
        // Fresh skill is freshly verified -> staleness ~ 0.
        assert!(SkillStore::staleness_score(fresh, now) < 0.01);
        let stale = store.get(DEMO_SKILL_STALE).expect("stale present");
        assert_eq!(stale.tier, SkillTrustTier::S3);
        // Stale skill last verified at epoch -> staleness well past 1.0.
        assert!(SkillStore::staleness_score(stale, now) >= 1.0);
    }

    #[test]
    fn demo_fresh_skill_promotes_one_rung() {
        // The fresh S2 skill is seeded past the min-age + min-invocation gate,
        // so a non-grant promote climbs it to S3 (the apex S4 jump still needs
        // a human grant, which this skill is not at yet).
        let now: u64 = 1_900_000_000;
        let mut store = seed_demo_store(now);
        let new = store
            .promote(DEMO_SKILL_FRESH, now, false)
            .expect("S2->S3 must clear the gate");
        assert_eq!(new, SkillTrustTier::S3);
    }

    #[test]
    fn demo_stale_skill_demotes_on_tick() {
        let now: u64 = 1_900_000_000;
        let mut store = seed_demo_store(now);
        let demoted = store.demote_if_stale(now);
        // Only the stale S3 skill demotes (the fresh S2 was just verified).
        assert_eq!(demoted.len(), 1, "exactly one stale skill demotes");
        assert_eq!(demoted[0].id, DEMO_SKILL_STALE);
        assert_eq!(demoted[0].prior_tier, SkillTrustTier::S3);
        assert_eq!(demoted[0].new_tier, SkillTrustTier::S2);
    }

    #[test]
    fn skill_library_view_round_trips_json() {
        let now: u64 = 1_900_000_000;
        let store = seed_demo_store(now);
        let skill = store.get(DEMO_SKILL_FRESH).expect("present");
        let view = SkillLibraryView::from_skill(skill, now);
        let json = serde_json::to_string(&view).expect("serialize");
        let back: SkillLibraryView = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.id, view.id);
        assert_eq!(back.tier, "s2");
    }

    #[test]
    fn publish_target_parse_round_trip() {
        assert_eq!(PublishTarget::parse("claude").expect("claude"), PublishTarget::Claude);
        assert_eq!(PublishTarget::parse("CODEX").expect("codex"), PublishTarget::Codex);
        assert!(PublishTarget::parse("unknown").is_err());
    }

    #[test]
    fn lexical_match_overlap_threshold() {
        assert!(lexical_match("stagnation detector persona switcher", "stagnation persona detected"));
        assert!(!lexical_match("stagnation detector persona switcher", "completely unrelated text input"));
    }

    #[test]
    fn blake3_prefix_is_stable() {
        // E-22: ensure the prefix is deterministic per input.
        let a = blake3_prefix("hello world");
        let b = blake3_prefix("hello world");
        let c = blake3_prefix("hello worldz");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn normalize_no_frontmatter_returns_unchanged() {
        let body = "# my-skill\n\njust a body.\n";
        assert_eq!(normalize_skill_md(body), body);
    }

    #[test]
    fn normalize_sorts_canonical_keys() {
        // Frontmatter with keys in wrong order: `version` before
        // `description` before `name`. After normalization the order
        // must be name, description, version.
        let body = "---\nversion: 1.0.0\ndescription: a thing\nname: my-skill\n---\nbody.\n";
        let norm = normalize_skill_md(body);
        let name_pos = norm.find("name:").expect("name present");
        let desc_pos = norm.find("description:").expect("desc present");
        let ver_pos = norm.find("version:").expect("ver present");
        assert!(name_pos < desc_pos, "name must come before description");
        assert!(desc_pos < ver_pos, "description must come before version");
        // body preserved
        assert!(norm.contains("body."));
    }

    #[test]
    fn normalize_appends_unknown_keys_after_canonical() {
        // Custom non-canonical key should appear after the canonical
        // block.
        let body = "---\nname: x\ncustom_field: foo\ndescription: y\n---\nbody\n";
        let norm = normalize_skill_md(body);
        let desc_pos = norm.find("description:").expect("desc present");
        let custom_pos = norm.find("custom_field:").expect("custom present");
        assert!(desc_pos < custom_pos, "canonical keys must precede unknowns");
    }
}
