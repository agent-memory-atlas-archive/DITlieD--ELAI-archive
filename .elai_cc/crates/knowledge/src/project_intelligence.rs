// Project Intelligence Tier Hooks (phase-3L).
//
// Three hooks layered on top of `ProjectProfile` + `ConventionStats` that
// emit prompt-side CONVENTIONS / IMPORTS / PROFILE blocks for the Telescope
// + reasoning prompt builder pipeline.
//
// Design constraints (locked in workflow/papers/tiered_analysis-interface.md
// + workflow/papers/project_profile-interface.md):
//   - Pi-prefixed types to avoid name collision with the SSB-write rule
//     family in elai_plugins::tiered_analysis (T1Hook/T2Hook/T3Hook).
//   - Mirror the tiered_analysis hook shape: private fields + new /
//     with_* / set_* constructors + a single `evaluate(...) -> Result`
//     method (PiT1Hook + PiT2Hook); T3 emits via `render_block`.
//   - Reuse `HookVerdict` from elai_plugins (do NOT fork).
//   - T3 NEVER emits BLOCK (categorical invariant, mirrors T3Hook).
//   - T2 returns Ok(None) on circuit-breaker (>5000 lines of synthesised
//     prompt material) OR unrecognised language.
//   - VERSION_GATE: T2 refuses to seed when the on-disk
//     `CONVENTIONS_SCHEMA_VERSION` is below the in-binary constant; instead
//     it queues a migration via the conventions_migration_v1.sql payload.
//   - LazyLock-style cached resources (no once_cell, no lazy_static).
//
// Wire points (phase-3L Step 7):
//   - `assemble_telescope_with_hooks(profile, hooks)` (knowledge/src/telescope.rs)
//     iterates the registered hooks in priority order (T1 conventions >
//     T2 imports > T3 profile metadata) and concatenates `render_block`
//     output while respecting cumulative token budget.
//   - reasoning::prompt_builder::build_with_conventions calls into PiT1Hook
//     to inject a CONVENTIONS section into the SYSTEM prompt.

use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;

use rusqlite::Connection;
use tracing::{debug, warn};

use elai_core::clock::ClockProvider;
use elai_core::project_profile::{
    Convention, ErrorHandlingProfile, NamingConventions, ProjectProfile,
};
use elai_plugins::tiered_analysis::HookVerdict;
use elai_rules::project_rule_seeder::ProjectRuleSeeder;

use crate::conventions::{
    extract_conventions, ConventionStats, CorpusView, CONVENTIONS_SCHEMA_VERSION,
};
use crate::error::KnowledgeError;
use crate::DkgNodeInfo;

// ---------------------------------------------------------------------------
// Budgets + identifiers
// ---------------------------------------------------------------------------

/// Approximate token budget for the CONVENTIONS prompt block (spec Step 4).
/// Char budget = tokens * 4 (English approximation, consistent with the
/// existing TelescopeContext::estimated_tokens convention).
pub const PI_T1_TOKEN_BUDGET: usize = 200;

/// Approximate token budget for the IMPORTS prompt block.
pub const PI_T2_TOKEN_BUDGET: usize = 160;

/// Approximate token budget for the PROFILE prompt block.
pub const PI_T3_TOKEN_BUDGET: usize = 140;

/// T2 circuit-breaker: if the candidate corpus exceeds this many lines of
/// synthesised prompt material, T2 returns `Ok(None)` (mirrors the
/// T2Hook::evaluate contract in elai_plugins).
pub const PI_T2_CIRCUIT_BREAKER_LINES: usize = 5_000;

/// Stable identifier of the T1 (conventions) hook.
pub const PI_T1_ID: &str = "pi_t1_conventions";
/// Stable identifier of the T2 (imports) hook.
pub const PI_T2_ID: &str = "pi_t2_imports";
/// Stable identifier of the T3 (profile) hook.
pub const PI_T3_ID: &str = "pi_t3_profile";

/// Priority anchors. Lower numbers render LATER in the assembled telescope
/// block; T1 (conventions) wins ordering by being numerically highest.
pub const PI_T1_PRIORITY: u8 = 100;
pub const PI_T2_PRIORITY: u8 = 80;
pub const PI_T3_PRIORITY: u8 = 60;

/// Embedded migration payload for the conventions schema (v1). Used by
/// `PiT2Hook::run` when the VERSION_GATE detects an older on-disk schema.
pub const CONVENTIONS_MIGRATION_V1_SQL: &str =
    include_str!("conventions_migration_v1.sql");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ProjectIntelligenceError {
    /// VERSION_GATE: on-disk schema is older than CONVENTIONS_SCHEMA_VERSION.
    #[error("conventions schema v{found} older than v{expected}; migration queued")]
    SchemaTooOld { found: u32, expected: u32 },

    /// Underlying SQLite error.
    #[error("rusqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Rule seeder propagated a rules.db error.
    #[error("rule seeder error: {0}")]
    RuleSeeder(#[from] elai_rules::error::RulesDbError),

    /// Knowledge engine propagated an extraction error.
    #[error("knowledge engine error: {0}")]
    Knowledge(#[from] KnowledgeError),
}

// ---------------------------------------------------------------------------
// ProjectIntelligenceHook trait
// ---------------------------------------------------------------------------

/// Render-side contract for Project Intelligence Tier hooks.
///
/// Mirrors the `T1Hook/T2Hook/T3Hook` family shape from elai_plugins but is
/// scoped to prompt assembly (no SSB writes, no rule predicate evaluation).
/// Native async fn in traits is the codebase standard (CLAUDE.md
/// anti-stale-training); kept sync here because every implementation is
/// pure CPU + bounded SQL.
pub trait ProjectIntelligenceHook: Send + Sync {
    /// Stable hook identifier (e.g. `"pi_t1_conventions"`).
    fn id(&self) -> &'static str;

    /// Render the hook's prompt block from a `ProjectProfile`. Returns
    /// `None` when the profile carries no signal worth emitting (e.g. all
    /// naming conventions are `Convention::Unknown`).
    fn render_block(&self, profile: &ProjectProfile) -> Option<String>;

    /// Priority anchor used by `assemble_telescope_with_hooks` to order
    /// concatenation. Higher = rendered first.
    fn priority(&self) -> u8;

    /// Char-based token-cost estimate (token = ceil(chars/4)). Callers
    /// MUST treat this as an upper bound when computing budget admission.
    fn token_cost_estimate(&self) -> usize;
}

// ---------------------------------------------------------------------------
// PiT1Hook — CONVENTIONS loader
// ---------------------------------------------------------------------------

/// Tier 1 PI hook: loads conventions from rules.db (or directly from the
/// passed `ConventionStats`) and renders a CONVENTIONS prompt block.
pub struct PiT1Hook {
    stats: Option<Arc<ConventionStats>>,
    project_id: String,
}

impl PiT1Hook {
    /// Construct an empty T1 hook. `render_block` falls back to the
    /// profile's own naming / error_handling fields when `stats` is None.
    pub fn new() -> Self {
        Self {
            stats: None,
            project_id: String::new(),
        }
    }

    /// Construct with pre-loaded convention stats (production wiring path).
    pub fn with_stats(stats: ConventionStats) -> Self {
        Self {
            stats: Some(Arc::new(stats)),
            project_id: String::new(),
        }
    }

    /// Set the project id used as the SELECT key in `load`.
    pub fn set_project_id(&mut self, id: impl Into<String>) {
        self.project_id = id.into();
    }

    /// Load convention stats from a conventions.db opened by the caller.
    ///
    /// Returns an `Err(SchemaTooOld)` when the VERSION_GATE rejects the
    /// on-disk schema. On Ok the loaded T1Hook holds the latest snapshot.
    ///
    /// `project_id` is stored so the dashboard render path can scope log
    /// lines to the right project root.
    pub fn load(
        project_id: &str,
        conn: &Connection,
    ) -> Result<Self, ProjectIntelligenceError> {
        check_schema_version(conn)?;
        // Snapshot loading is intentionally light: we only need to confirm
        // the table is reachable and pull a count so the dashboard can
        // surface "T1 has N convention rows". The actual NamingConventions
        // payload comes from the live ProjectProfile passed to
        // `render_block`, not from this connection.
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM conventions WHERE project_id = ?1",
                rusqlite::params![project_id],
                |row| row.get(0),
            )
            .unwrap_or(0);
        debug!(project_id, count, "PiT1Hook::load conventions row count");
        let mut hook = Self::new();
        hook.set_project_id(project_id);
        Ok(hook)
    }
}

impl Default for PiT1Hook {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectIntelligenceHook for PiT1Hook {
    fn id(&self) -> &'static str {
        PI_T1_ID
    }

    fn render_block(&self, profile: &ProjectProfile) -> Option<String> {
        let summary = format_conventions_block(
            self.stats.as_deref(),
            &profile.naming,
            &profile.error_handling,
        )?;
        Some(truncate_to_budget(summary, PI_T1_TOKEN_BUDGET * 4))
    }

    fn priority(&self) -> u8 {
        PI_T1_PRIORITY
    }

    fn token_cost_estimate(&self) -> usize {
        PI_T1_TOKEN_BUDGET
    }
}

// ---------------------------------------------------------------------------
// PiT2Hook — IMPORTS seeder
// ---------------------------------------------------------------------------

/// Tier 2 PI hook: extracts AST imports from a DKG node slice and seeds
/// auto-import rules via `ProjectRuleSeeder::seed_rules`.
pub struct PiT2Hook {
    circuit_breaker_lines: usize,
}

impl PiT2Hook {
    /// Default constructor uses the spec circuit-breaker line count.
    pub fn new() -> Self {
        Self {
            circuit_breaker_lines: PI_T2_CIRCUIT_BREAKER_LINES,
        }
    }

    /// Override the circuit breaker (test-only knob; production callers
    /// should use `new`).
    pub fn with_circuit_breaker(lines: usize) -> Self {
        Self {
            circuit_breaker_lines: lines,
        }
    }

    /// Evaluate the T2 hook against an in-memory corpus view + clock.
    ///
    /// Returns `Ok(None)` when the synthesised prompt material would
    /// exceed `circuit_breaker_lines` OR when the corpus' primary language
    /// is unrecognised (mirrors the T2Hook::evaluate Ok(None) semantics
    /// per workflow/papers/tiered_analysis-interface.md).
    pub fn evaluate(
        &self,
        corpus: &CorpusView<'_>,
    ) -> Result<Option<HookVerdict>, ProjectIntelligenceError> {
        if !LANGUAGES_KNOWN_TO_PI_T2.contains(&corpus.primary_language) {
            return Ok(None);
        }
        let stats = extract_conventions(corpus);
        let synth_lines = estimate_imports_lines(&stats, corpus.dkg_nodes);
        if synth_lines > self.circuit_breaker_lines {
            return Ok(None);
        }
        // Verdict is purely advisory at the PI layer (T1 already handles
        // hard blocks via the SSB-write rule family). Emit WARN when at
        // least one convention crystallised; Allow otherwise.
        let verdict = if any_strong_convention(&stats.naming) {
            HookVerdict::Warn {
                message: format!("PiT2 seeded {} convention slots", count_known(&stats.naming)),
                pattern_hash: blake3::hash(stats.source.as_bytes()).to_hex().to_string(),
            }
        } else {
            HookVerdict::Allow
        };
        Ok(Some(verdict))
    }

    /// Seed auto-import rules from a `ProjectProfile` into rules.db.
    ///
    /// VERSION_GATE: refuses to seed when the on-disk
    /// CONVENTIONS_SCHEMA_VERSION is older than the in-binary constant.
    /// On gate failure, queues the embedded migration payload via
    /// `Connection::execute_batch` (idempotent — every CREATE in the
    /// migration uses IF NOT EXISTS) and returns Err(SchemaTooOld).
    pub fn run(
        &self,
        _project_id: &str,
        conn: &mut Connection,
        profile: &ProjectProfile,
        clock: &impl ClockProvider,
    ) -> Result<usize, ProjectIntelligenceError> {
        // VERSION_GATE: queue migration on stale schema.
        if let Err(ProjectIntelligenceError::SchemaTooOld { found, expected }) =
            check_schema_version(conn)
        {
            warn!(
                found,
                expected, "conventions schema stale; running migration v1"
            );
            conn.execute_batch(CONVENTIONS_MIGRATION_V1_SQL)?;
            return Err(ProjectIntelligenceError::SchemaTooOld { found, expected });
        }
        let inserted = ProjectRuleSeeder::seed_rules(conn, profile, clock)?;
        debug!(inserted, "PiT2Hook::run seeded rules");
        Ok(inserted)
    }
}

impl Default for PiT2Hook {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectIntelligenceHook for PiT2Hook {
    fn id(&self) -> &'static str {
        PI_T2_ID
    }

    fn render_block(&self, profile: &ProjectProfile) -> Option<String> {
        let raw = format_imports_block(profile)?;
        Some(truncate_to_budget(raw, PI_T2_TOKEN_BUDGET * 4))
    }

    fn priority(&self) -> u8 {
        PI_T2_PRIORITY
    }

    fn token_cost_estimate(&self) -> usize {
        PI_T2_TOKEN_BUDGET
    }
}

// ---------------------------------------------------------------------------
// PiT3Hook — PROFILE metadata
// ---------------------------------------------------------------------------

/// Tier 3 PI hook: emits a compact PROFILE block (vision + architecture +
/// build/test commands) sourced from the persisted
/// `metrics/project_profile.toml` snapshot. NEVER emits BLOCK (categorical
/// invariant — mirrors `T3Hook` in elai_plugins).
pub struct PiT3Hook;

impl PiT3Hook {
    pub fn new() -> Self {
        Self
    }

    /// Render an advisory block; analogous to `T3Hook::evaluate`. Returns
    /// `None` (NOT `Some(Block)`) when the profile carries no Tier-3
    /// material.
    pub fn evaluate(&self, profile: &ProjectProfile) -> Option<HookVerdict> {
        // T3 is advisory; map render-output presence to a Warn. The
        // T3-never-blocks invariant is upheld by this match exhausting only
        // None/Warn variants — no Block constructor is reachable.
        self.render_block(profile).map(|msg| HookVerdict::Warn {
            message: msg,
            pattern_hash: blake3::hash(b"pi_t3_profile_render").to_hex().to_string(),
        })
    }
}

impl Default for PiT3Hook {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectIntelligenceHook for PiT3Hook {
    fn id(&self) -> &'static str {
        PI_T3_ID
    }

    fn render_block(&self, profile: &ProjectProfile) -> Option<String> {
        let raw = format_profile_block(profile)?;
        Some(truncate_to_budget(raw, PI_T3_TOKEN_BUDGET * 4))
    }

    fn priority(&self) -> u8 {
        PI_T3_PRIORITY
    }

    fn token_cost_estimate(&self) -> usize {
        PI_T3_TOKEN_BUDGET
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

static LANGUAGES_KNOWN_TO_PI_T2: LazyLock<[&'static str; 6]> = LazyLock::new(|| {
    ["rust", "python", "typescript", "javascript", "go", "svelte"]
});

fn check_schema_version(conn: &Connection) -> Result<(), ProjectIntelligenceError> {
    let found: u32 = conn
        .query_row(
            "SELECT MAX(schema_version) FROM conventions_meta",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    if found < CONVENTIONS_SCHEMA_VERSION {
        return Err(ProjectIntelligenceError::SchemaTooOld {
            found,
            expected: CONVENTIONS_SCHEMA_VERSION,
        });
    }
    Ok(())
}

fn format_conventions_block(
    stats: Option<&ConventionStats>,
    naming: &NamingConventions,
    error_handling: &ErrorHandlingProfile,
) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();

    let render = |label: &str, c: &Convention| -> Option<String> {
        match c {
            Convention::Strong(s) => Some(format!("{label}:{s} (strong)")),
            Convention::Moderate(s) => Some(format!("{label}:{s} (moderate)")),
            _ => None,
        }
    };

    if let Some(s) = render("fn", &naming.functions) {
        parts.push(s);
    }
    if let Some(s) = render("types", &naming.types) {
        parts.push(s);
    }
    if let Some(s) = render("const", &naming.constants) {
        parts.push(s);
    }
    if let Some(s) = render("modules", &naming.modules) {
        parts.push(s);
    }
    if let Some(crate_name) = &error_handling.error_crate {
        parts.push(format!("errors:{crate_name}"));
    }

    // Top-3 anti-patterns: derived from inverse of detected conventions
    // (e.g. "fn detected as snake_case -> avoid camelCase for fns"). Stable
    // ordering: matches the parts iteration above.
    let anti = collect_anti_patterns(naming);
    if parts.is_empty() && anti.is_empty() && stats.is_none() {
        return None;
    }

    let mut block = String::new();
    block.push_str("CONVENTIONS\n");
    if !parts.is_empty() {
        block.push_str("  ");
        block.push_str(&parts.join(", "));
        block.push('\n');
    }
    if !anti.is_empty() {
        block.push_str("  avoid: ");
        block.push_str(&anti.join(", "));
        block.push('\n');
    }
    Some(block)
}

fn format_imports_block(profile: &ProjectProfile) -> Option<String> {
    if profile.dep_profile.declared.is_empty() && profile.frameworks.is_empty() {
        return None;
    }
    let mut block = String::new();
    block.push_str("IMPORTS\n");
    if !profile.frameworks.is_empty() {
        let names: Vec<String> = profile
            .frameworks
            .iter()
            .take(5)
            .map(|f| f.name.clone())
            .collect();
        block.push_str("  frameworks: ");
        block.push_str(&names.join(", "));
        block.push('\n');
    }
    if !profile.dep_profile.declared.is_empty() {
        // Stable: declared is already a Vec<String>; cap at 8 to fit budget.
        let mut deps: Vec<String> = profile.dep_profile.declared.iter().take(8).cloned().collect();
        deps.sort();
        block.push_str("  declared: ");
        block.push_str(&deps.join(", "));
        block.push('\n');
    }
    Some(block)
}

fn format_profile_block(profile: &ProjectProfile) -> Option<String> {
    let has_signal = profile.project_vision.is_some()
        || profile.architecture.is_some()
        || !profile.domains.is_empty()
        || !profile.commands.build.is_empty()
        || !profile.commands.test.is_empty();
    if !has_signal {
        return None;
    }
    let mut block = String::new();
    block.push_str("PROFILE\n");
    if let Some(v) = &profile.project_vision {
        block.push_str("  vision: ");
        block.push_str(v);
        block.push('\n');
    }
    if let Some(arch) = &profile.architecture {
        block.push_str(&format!("  arch: {arch:?}\n"));
    }
    if !profile.domains.is_empty() {
        let labels: Vec<&'static str> = profile.domains.iter().map(|d| d.domain.label()).collect();
        block.push_str("  domains: ");
        block.push_str(&labels.join(", "));
        block.push('\n');
    }
    if let Some(b) = profile.commands.build.first() {
        block.push_str("  build: ");
        block.push_str(&b.cmd);
        block.push('\n');
    }
    if let Some(t) = profile.commands.test.first() {
        block.push_str("  test: ");
        block.push_str(&t.cmd);
        block.push('\n');
    }
    Some(block)
}

fn collect_anti_patterns(naming: &NamingConventions) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let push_anti = |out: &mut Vec<String>, label: &str, c: &Convention| match c {
        Convention::Strong(style) | Convention::Moderate(style) => {
            // Universally-mutually-exclusive case styles.
            for alt in ["snake_case", "camelCase", "PascalCase", "SCREAMING_SNAKE_CASE"] {
                if alt != style.as_str() && !out.iter().any(|s| s.contains(alt)) {
                    out.push(format!("{label}:{alt}"));
                    break;
                }
            }
        }
        _ => {}
    };
    push_anti(&mut out, "fn", &naming.functions);
    push_anti(&mut out, "types", &naming.types);
    push_anti(&mut out, "const", &naming.constants);
    out.truncate(3);
    out
}

fn estimate_imports_lines(
    stats: &ConventionStats,
    dkg_nodes: Option<&[DkgNodeInfo]>,
) -> usize {
    let nodes_lines = dkg_nodes.map(|s| s.len()).unwrap_or(0);
    // Each convention category contributes at most one synth line.
    let convention_lines =
        count_known(&stats.naming) + usize::from(stats.error_handling.error_crate.is_some());
    nodes_lines + convention_lines
}

fn count_known(naming: &NamingConventions) -> usize {
    [
        &naming.functions,
        &naming.types,
        &naming.constants,
        &naming.modules,
    ]
    .iter()
    .filter(|c| !matches!(c, Convention::Unknown))
    .count()
}

fn any_strong_convention(naming: &NamingConventions) -> bool {
    [
        &naming.functions,
        &naming.types,
        &naming.constants,
        &naming.modules,
    ]
    .iter()
    .any(|c| matches!(c, Convention::Strong(_)))
}

/// Truncate a rendered block to `max_chars`, keeping char-boundary
/// safety + breaking at the last newline so partial lines never leak into
/// the prompt.
fn truncate_to_budget(mut block: String, max_chars: usize) -> String {
    if block.len() <= max_chars {
        return block;
    }
    let mut boundary = max_chars;
    while boundary > 0 && !block.is_char_boundary(boundary) {
        boundary -= 1;
    }
    block.truncate(boundary);
    if let Some(last_nl) = block.rfind('\n') {
        block.truncate(last_nl + 1);
    }
    block
}

// ---------------------------------------------------------------------------
// Persistence helper: write a ProjectProfile snapshot to disk.
//
// Spec phase-3L Step 6 cites `metrics/project_profile.toml`. Per the
// CLAUDE.md project boundary rule the canonical path is anchored at
// `.elai_cc/metrics/project_profile.toml`. Callers pass the workspace root
// (typically `.elai_cc/`) as `out_root`.
// ---------------------------------------------------------------------------

/// Write `profile` to `<out_root>/metrics/project_profile.toml`.
/// Creates the `metrics/` directory if missing. Returns the canonical path
/// written.
pub fn persist_profile(
    profile: &ProjectProfile,
    out_root: &Path,
) -> Result<std::path::PathBuf, elai_core::project_profile::ProjectProfileError> {
    let metrics_dir = out_root.join("metrics");
    if !metrics_dir.exists() {
        std::fs::create_dir_all(&metrics_dir)
            .map_err(elai_core::project_profile::ProjectProfileError::Io)?;
    }
    let path = metrics_dir.join("project_profile.toml");
    let serialized = crate::project_profile::serialize_profile(profile)?;
    std::fs::write(&path, serialized)
        .map_err(elai_core::project_profile::ProjectProfileError::Io)?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::project_profile::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn rust_profile() -> ProjectProfile {
        ProjectProfile {
            schema_version: PROFILE_SCHEMA_VERSION,
            generated_at: "2026-05-22T00:00:00Z".to_owned(),
            project_root: PathBuf::from("/test"),
            build_system: BuildSystemProfile::default(),
            languages: LanguageProfile {
                primary: Language::Rust,
                secondary: Vec::new(),
                distribution: BTreeMap::new(),
                tree_sitter_coverage: 0.0,
            },
            frameworks: Vec::new(),
            test_profile: TestProfile::default(),
            lint_tools: Vec::new(),
            entry_points: Vec::new(),
            ci: None,
            naming: NamingConventions {
                functions: Convention::Strong("snake_case".to_owned()),
                types: Convention::Strong("PascalCase".to_owned()),
                constants: Convention::Strong("SCREAMING_SNAKE_CASE".to_owned()),
                modules: Convention::Unknown,
            },
            error_handling: ErrorHandlingProfile {
                pattern: ErrorPattern::ResultBased,
                propagation: PropagationStyle::QuestionMark,
                error_crate: Some("thiserror".to_owned()),
            },
            module_organization: ModuleOrganization {
                pattern: OrgPattern::Layered,
            },
            dep_profile: DependencyProfile::default(),
            project_vision: Some("a rust web backend".to_owned()),
            architecture: Some(ArchitecturePattern::Layered),
            domains: vec![DomainClassification {
                domain: ProjectDomain::WebBackend,
                confidence: 0.9,
                signals: vec!["axum".to_owned()],
            }],
            commands: BuildCommandRegistry {
                build: vec![BuildCommand {
                    cmd: "cargo build".to_owned(),
                    working_dir: PathBuf::from("."),
                    env: BTreeMap::new(),
                    timeout_s: 120,
                    scope: CommandScope::Full,
                    verified: false,
                }],
                test: vec![BuildCommand {
                    cmd: "cargo test".to_owned(),
                    working_dir: PathBuf::from("."),
                    env: BTreeMap::new(),
                    timeout_s: 120,
                    scope: CommandScope::Full,
                    verified: false,
                }],
                lint: Vec::new(),
                format: Vec::new(),
                run: Vec::new(),
                clean: Vec::new(),
            },
            fix_template_keys: Vec::new(),
            constraints: Vec::new(),
            overrides_applied: false,
        }
    }

    #[test]
    fn pi_t1_id_and_priority() {
        let h = PiT1Hook::new();
        assert_eq!(h.id(), PI_T1_ID);
        assert_eq!(h.priority(), PI_T1_PRIORITY);
        assert!(h.token_cost_estimate() <= PI_T1_TOKEN_BUDGET);
    }

    #[test]
    fn pi_t2_id_and_priority() {
        let h = PiT2Hook::new();
        assert_eq!(h.id(), PI_T2_ID);
        assert_eq!(h.priority(), PI_T2_PRIORITY);
    }

    #[test]
    fn pi_t3_id_and_priority_and_never_blocks() {
        let h = PiT3Hook::new();
        assert_eq!(h.id(), PI_T3_ID);
        assert_eq!(h.priority(), PI_T3_PRIORITY);
        let profile = rust_profile();
        let v = h.evaluate(&profile).expect("profile has signal");
        // T3 NEVER emits Block — categorical invariant.
        assert!(!matches!(v, HookVerdict::Block { .. }));
    }

    #[test]
    fn pi_t1_render_block_contains_conventions_header() {
        let profile = rust_profile();
        let h = PiT1Hook::new();
        let block = h.render_block(&profile).expect("strong naming -> block");
        assert!(block.starts_with("CONVENTIONS\n"));
        assert!(block.contains("snake_case"));
        // Budget cap upheld.
        assert!(block.len() <= PI_T1_TOKEN_BUDGET * 4 + 1);
    }

    #[test]
    fn pi_t1_render_block_is_none_on_empty_profile() {
        let mut profile = rust_profile();
        profile.naming = NamingConventions::default();
        profile.error_handling = ErrorHandlingProfile::default();
        let h = PiT1Hook::new();
        // All Unknown + no error crate -> nothing worth emitting.
        assert!(h.render_block(&profile).is_none());
    }

    #[test]
    fn pi_t2_render_block_returns_none_when_no_imports_signal() {
        // Empty deps + empty frameworks -> None.
        let mut profile = rust_profile();
        profile.dep_profile = DependencyProfile::default();
        profile.frameworks.clear();
        let h = PiT2Hook::new();
        assert!(h.render_block(&profile).is_none());
    }

    #[test]
    fn pi_t2_evaluate_unrecognised_language_returns_none() {
        let dir = tempfile::TempDir::new().unwrap();
        let view = CorpusView::new(None, dir.path(), "fortran");
        let h = PiT2Hook::new();
        let out = h.evaluate(&view).expect("ok");
        assert!(out.is_none(), "unrecognised language must return Ok(None)");
    }

    #[test]
    fn pi_t2_circuit_breaker_returns_none() {
        let dir = tempfile::TempDir::new().unwrap();
        // Force a tiny breaker so the synthesised line count trips it.
        let h = PiT2Hook::with_circuit_breaker(0);
        let nodes: Vec<DkgNodeInfo> = (0..50)
            .map(|i| DkgNodeInfo {
                node_id: i,
                symbol_name: format!("name_{i}"),
                file_path: PathBuf::from("/x.rs"),
                signature: "fn name() {}".to_owned(),
                kind: crate::SymbolKind::Function,
                start_line: 1,
                end_line: 1,
            })
            .collect();
        let view = CorpusView::new(Some(&nodes), dir.path(), "rust");
        let out = h.evaluate(&view).expect("ok");
        assert!(out.is_none(), "circuit-breaker must return Ok(None)");
    }

    #[test]
    fn pi_t3_render_block_starts_with_profile_header() {
        let profile = rust_profile();
        let h = PiT3Hook::new();
        let block = h.render_block(&profile).expect("profile has signal");
        assert!(block.starts_with("PROFILE\n"));
        assert!(block.contains("cargo build"));
    }

    #[test]
    fn persist_profile_writes_canonical_path() {
        let dir = tempfile::TempDir::new().unwrap();
        let profile = rust_profile();
        let written = persist_profile(&profile, dir.path()).expect("write");
        assert!(written.ends_with("metrics/project_profile.toml"));
        assert!(written.exists());
    }
}
