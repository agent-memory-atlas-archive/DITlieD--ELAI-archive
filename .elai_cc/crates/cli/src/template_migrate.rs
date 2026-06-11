//! D-1 — `elai template migrate` CLI driver.
//!
//! Bootstraps the append-only reasoning-template archive
//! (`.elai_cc/archive/reasoning-templates.jsonl`) from a fixed set of
//! canonical reasoning patterns, writing one [`ArchiveEntry`] per pattern with
//! `archive_status = Frozen` (the initial set is immutable until D-5 BAI is
//! wired to score it — see workflow/reasoning/D-1-decisions.md DECISION-c).
//!
//! Surfaces the D-1 schema (`elai_orchestrator::reasoning_template` +
//! `elai_orchestrator::archive_chain`) through a top-level subcommand + the
//! off-agent `ArchiveEntryAdded` SSB row that the dashboard MutationArchivePanel
//! consumes.
//!
//! Source-of-truth note: the original D-1 plan assumed the migration would
//! scan `crates/prompts/templates/*.toml` + `agent_definitions/*.md`. That
//! directory layout never materialised in `.elai_cc` (there is no `prompts`
//! crate and no `templates/*.toml` reasoning files). Rather than depend on a
//! fragile path that would yield zero rows (an R1 fail-closed hazard — the
//! observable_effect requires >=1 ArchiveEntryAdded), the bootstrap ships a
//! built-in canonical seed set (the `Domain x PatternKind` reasoning patterns
//! the closed schema models). An optional `--source DIR` additively folds in
//! any `*.md` / `*.toml` files found under a caller-supplied directory, with
//! each file's contents becoming one template body (truncated-rejected if it
//! exceeds the `BoundedString<1024>` bound).
//!
//! Wiring discipline:
//!   producer  : `elai_orchestrator::archive_chain::append_and_emit_ssb`
//!               (D-1; appends the JSONL line AND emits the SSB row).
//!   consumer  : this file — constructs the templates + drives the append.
//!   ssb row   : `SsbActionType::ArchiveEntryAdded` (D-1 SCHEMA_VERSION 64+).
//!   artifact  : `<cwd>/archive/reasoning-templates.jsonl`.
//!
//! Idempotency: each pattern's `self_hash` is computed from `(body, parent)`.
//! On re-run we load the existing chain, build the set of present `self_hash`
//! values, and skip any pattern whose hash is already on disk. A second run on
//! unchanged input therefore extracts zero new entries (plan Task 5 + the
//! `stub_ban` idempotency rule).
//!
//! Exit codes:
//!   0 = migration completed (zero or more new entries appended).
//!   1 = infrastructural failure (unable to write the archive / SSB writer
//!       construction failed in a way that left the chain inconsistent).
//!
//! No `.unwrap()` / `.expect()` in non-test code. All I/O + schema errors flow
//! through `anyhow::Error` at the CLI boundary and `TemplateError` /
//! `SsbError` internally.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_core::paths::sessions_dir_path;
use elai_orchestrator::archive_chain::{append_and_emit_ssb, ArchiveChain, ArchiveEntry};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Blake3Hash, Domain, PatternKind, Predicate,
    ReasoningTemplate, TemplateBody, TemplateId, VerifierKind, TEMPLATE_BODY_BOUND,
};
use elai_orchestrator::{Ssb, SsbConfig};

// ---------------------------------------------------------------------------
// Canonical bootstrap reasoning patterns.
// ---------------------------------------------------------------------------

/// One built-in canonical reasoning pattern. The migration turns each row into
/// a `Frozen` [`ReasoningTemplate`]. Bodies are deliberately short, declarative
/// pattern descriptions (well within the 1024-byte bound) so the bootstrap is
/// deterministic + git-diffable.
struct SeedPattern {
    domain: Domain,
    pattern_kind: PatternKind,
    verifier: VerifierKind,
    body: &'static str,
}

/// The canonical seed set. These map the closed `Domain x PatternKind` schema
/// onto the four reasoning strategies the orchestrator selects between. They
/// are the L3-frozen bootstrap (per spec §4); D-5 BAI scores *candidate*
/// children of these, never these themselves.
const SEED_PATTERNS: &[SeedPattern] = &[
    SeedPattern {
        domain: Domain::Code,
        pattern_kind: PatternKind::Decomposition,
        verifier: VerifierKind::Compile,
        body: "Break the coding task into the smallest compiling vertical slices. \
For each slice name the producer, its in-plan consumer, and the observable \
artifact it leaves. Refuse a slice that ships a producer without a consumer.",
    },
    SeedPattern {
        domain: Domain::Code,
        pattern_kind: PatternKind::Verification,
        verifier: VerifierKind::Test,
        body: "Drive the change from a real entrypoint, then assert on an \
artifact you did not author (a runtime row, a coverage hit, a test the auditor \
owns). Treat missing evidence as failure, never as a skip.",
    },
    SeedPattern {
        domain: Domain::Math,
        pattern_kind: PatternKind::Reflection,
        verifier: VerifierKind::Prm,
        body: "After each derivation step restate the invariant the step must \
preserve, then check it holds. If a step cannot be justified from a prior \
line, mark it unsupported and stop rather than guessing.",
    },
    SeedPattern {
        domain: Domain::Math,
        pattern_kind: PatternKind::Verification,
        verifier: VerifierKind::Prm,
        body: "Re-evaluate the candidate answer by an independent method and \
require agreement within the stated tolerance. Divergence is a hard failure, \
not a tie to be averaged away.",
    },
    SeedPattern {
        domain: Domain::Prose,
        pattern_kind: PatternKind::Synthesis,
        verifier: VerifierKind::Refuse,
        body: "Synthesize only from grounded sources you can cite. Where a \
claim has no source, label it ungrounded and exclude it from the conclusion. \
Prefer abstention over an unsupported assertion.",
    },
    SeedPattern {
        domain: Domain::Mixed,
        pattern_kind: PatternKind::Decomposition,
        verifier: VerifierKind::Test,
        body: "Split a mixed task along its verifier boundaries: code parts get \
a compile/test gate, prose parts get a citation gate. Route each sub-result to \
the verifier its kind requires before merging.",
    },
    SeedPattern {
        domain: Domain::Planning,
        pattern_kind: PatternKind::Decomposition,
        verifier: VerifierKind::Compile,
        body: "Decompose the goal into an ordered plan of steps with explicit \
preconditions and a done-gate per step. A step whose done-gate cannot be \
observed is under-specified and must be re-decomposed.",
    },
    SeedPattern {
        domain: Domain::ToolUse,
        pattern_kind: PatternKind::Reflection,
        verifier: VerifierKind::Test,
        body: "Before each tool call state the expected effect; after it, \
compare the observed result to the expectation. A mismatch triggers a rethink, \
never a blind retry of the same call.",
    },
];

/// Migration outcome counts surfaced to the operator + the `--json` report.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct MigrateReport {
    pub archive_path: String,
    /// New entries appended this run.
    pub extracted: u32,
    /// Patterns skipped because their `self_hash` already exists in the chain
    /// (idempotency).
    pub skipped_duplicate: u32,
    /// Patterns / source files skipped because the body exceeded the
    /// `BoundedString<1024>` bound.
    pub skipped_too_long: u32,
    /// Total entries on disk after the run.
    pub total_after: u32,
}

// ---------------------------------------------------------------------------
// CLI entry — sync wrapper called from main.rs.
// ---------------------------------------------------------------------------

/// Default JSONL archive path matching D-1's writer + the `elai archive verify`
/// default. The CLI is invoked from `.elai_cc/` (operator / CI cwd).
fn default_archive_path(project_root: &Path) -> PathBuf {
    project_root.join("archive").join("reasoning-templates.jsonl")
}

/// CLI entry — `elai template migrate [--source DIR] [--path PATH] [--json]`.
///
/// Returns `Ok(())` on clean exit (rc=0). Returns `Err(anyhow::Error)` only on
/// an infrastructural failure (archive append failed); the top-level
/// dispatcher formats it through the standard error path + exits non-zero.
pub(crate) fn run(
    source: Option<String>,
    path: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let archive_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| default_archive_path(&project_root));

    // Current-thread tokio runtime — `append_and_emit_ssb` is async (the SSB
    // writer requires a runtime context). Sync entry pattern matches
    // `archive_verify::run` / `ppm_health::run`.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!("building tokio runtime for template migrate: {e}"))?;

    // `Ssb::new` spawns a single-writer tokio task, so it MUST be constructed
    // from within the runtime context (mirrors `archive_verify::run`, which
    // builds its writer inside `rt.block_on`).
    let report = rt.block_on(async {
        let ssb = build_ssb(&project_root)
            .map_err(|e| anyhow::anyhow!("opening SSB writer for template migrate: {e}"))?;
        migrate(&archive_path, source.as_deref(), &ssb).await
    })?;

    render_report(json, &report)?;
    Ok(())
}

/// Construct the SSB writer bound to the active project's session DB.
fn build_ssb(project_root: &Path) -> anyhow::Result<Arc<Ssb>> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    let parent = db_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("ssb.db has no parent dir; cannot prepare path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|e| anyhow::anyhow!("creating ssb sessions dir: {e}"))?;
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: "cli-template-migrate".to_string(),
    };
    let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(cfg, clock).map_err(|e| anyhow::anyhow!("opening SSB writer: {e}"))?;
    Ok(Arc::new(ssb))
}

// ---------------------------------------------------------------------------
// Core migration (testable: takes an explicit archive path + SSB writer).
// ---------------------------------------------------------------------------

/// Core migration: load the existing chain, append every not-yet-present
/// canonical pattern (+ optional `--source` files) as a `Frozen` entry via
/// [`append_and_emit_ssb`] (which emits the `ArchiveEntryAdded` SSB row), and
/// return the outcome counts.
///
/// Idempotent: a pattern whose `self_hash` is already on disk is counted as
/// `skipped_duplicate`, not re-appended.
pub(crate) async fn migrate(
    archive_path: &Path,
    source: Option<&str>,
    ssb: &Arc<Ssb>,
) -> anyhow::Result<MigrateReport> {
    // 1. Load existing chain (missing file = empty). Build the present-hash set
    //    for idempotency.
    let chain = ArchiveChain::load(archive_path.to_path_buf())
        .map_err(|e| anyhow::anyhow!("loading existing archive chain: {e}"))?;
    let mut present: std::collections::HashSet<Blake3Hash> =
        chain.entries().iter().map(|e| e.self_hash).collect();

    let mut working = chain;
    let mut report = MigrateReport {
        archive_path: archive_path.to_string_lossy().into_owned(),
        ..MigrateReport::default()
    };

    let created_at_ms = i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX);

    // 2. Built-in canonical seed set.
    for seed in SEED_PATTERNS {
        match build_template(seed.domain, seed.pattern_kind, seed.verifier, seed.body, created_at_ms)
        {
            Ok(template) => {
                append_one(&mut working, ssb, &mut present, template, &mut report).await?;
            }
            Err(BodyTooLong) => {
                report.skipped_too_long = report.skipped_too_long.saturating_add(1);
            }
        }
    }

    // 3. Optional additive source scan. Each `*.md` / `*.toml` file becomes one
    //    Prose/Synthesis template body. Over-long files are counted, not
    //    truncated (BoundedString rejects them).
    if let Some(dir) = source {
        let files = scan_source_dir(Path::new(dir))?;
        for (idx, body) in files {
            let _ = idx; // file index reserved for future per-file reporting.
            match build_template(
                Domain::Mixed,
                PatternKind::Synthesis,
                VerifierKind::Refuse,
                &body,
                created_at_ms,
            ) {
                Ok(template) => {
                    append_one(&mut working, ssb, &mut present, template, &mut report).await?;
                }
                Err(BodyTooLong) => {
                    report.skipped_too_long = report.skipped_too_long.saturating_add(1);
                }
            }
        }
    }

    report.total_after = u32::try_from(working.len()).unwrap_or(u32::MAX);
    Ok(report)
}

/// Append `template` as a root entry IF its `self_hash` is not already present.
/// Updates the present-set + report counters.
async fn append_one(
    chain: &mut ArchiveChain,
    ssb: &Arc<Ssb>,
    present: &mut std::collections::HashSet<Blake3Hash>,
    template: ReasoningTemplate,
    report: &mut MigrateReport,
) -> anyhow::Result<()> {
    let entry = ArchiveEntry::new_root(template);
    if present.contains(&entry.self_hash) {
        report.skipped_duplicate = report.skipped_duplicate.saturating_add(1);
        return Ok(());
    }
    let self_hash = entry.self_hash;
    // append_and_emit_ssb appends the JSONL line AND emits the
    // ArchiveEntryAdded SSB row (the off-agent observable effect).
    append_and_emit_ssb(chain, ssb, "cli:template-migrate", 0, entry)
        .await
        .map_err(|e| anyhow::anyhow!("appending archive entry: {e}"))?;
    present.insert(self_hash);
    report.extracted = report.extracted.saturating_add(1);
    Ok(())
}

/// Sentinel: the body exceeded the `BoundedString<1024>` bound.
struct BodyTooLong;

/// Build a `Frozen` [`ReasoningTemplate`] with a recomputed content hash.
/// Returns [`BodyTooLong`] (NOT a panic) when the body overflows the bound.
fn build_template(
    domain: Domain,
    pattern_kind: PatternKind,
    verifier: VerifierKind,
    body: &str,
    created_at_ms: i64,
) -> Result<ReasoningTemplate, BodyTooLong> {
    if body.len() > TEMPLATE_BODY_BOUND {
        return Err(BodyTooLong);
    }
    let template_body = TemplateBody::try_new(body).map_err(|_| BodyTooLong)?;
    let content_hash = compute_content_hash(template_body.as_str(), None);
    Ok(ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain,
        pattern_kind,
        template_body,
        eligibility_predicate: Predicate::Always,
        verifier_required: verifier,
        content_hash,
        created_at_ms,
        parent_id: None,
        archive_status: ArchiveStatus::Frozen,
    })
}

/// Read `*.md` / `*.toml` files directly under `dir` (non-recursive), returning
/// `(index, contents)` pairs in a deterministic (sorted-by-name) order. A
/// missing directory is an empty result (not an error) so the seed-only path
/// still runs.
fn scan_source_dir(dir: &Path) -> anyhow::Result<Vec<(usize, String)>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut paths: Vec<PathBuf> = Vec::new();
    let read = std::fs::read_dir(dir)
        .map_err(|e| anyhow::anyhow!("reading source dir {}: {e}", dir.display()))?;
    for entry in read {
        let entry = entry.map_err(|e| anyhow::anyhow!("reading dir entry: {e}"))?;
        let p = entry.path();
        if p.is_file() {
            let ext = p
                .extension()
                .and_then(|e| e.to_str())
                .map(str::to_ascii_lowercase)
                .unwrap_or_default();
            if ext == "md" || ext == "toml" {
                paths.push(p);
            }
        }
    }
    paths.sort();
    let mut out = Vec::with_capacity(paths.len());
    for (idx, p) in paths.into_iter().enumerate() {
        let contents = std::fs::read_to_string(&p)
            .map_err(|e| anyhow::anyhow!("reading {}: {e}", p.display()))?;
        out.push((idx, contents));
    }
    Ok(out)
}

fn render_report(json: bool, report: &MigrateReport) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }
    println!(
        "TEMPLATE MIGRATE: extracted={} skipped_duplicate={} skipped_too_long={} total={} path={}",
        report.extracted,
        report.skipped_duplicate,
        report.skipped_too_long,
        report.total_after,
        report.archive_path
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests (in-crate; the env note bans orchestrator integration tests — the
// off-agent smoke is a single sequential #[tokio::test] reading the SSB row
// back).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use elai_orchestrator::ssb::{SsbActionType, SsbReader};
    use tempfile::TempDir;

    fn temp_ssb(dir: &TempDir) -> Arc<Ssb> {
        let cfg = SsbConfig {
            db_path: dir.path().join("ssb.db"),
            pool_size: None,
            project_id: "default".to_string(),
            session_id: "test-template-migrate".to_string(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("open ssb"))
    }

    #[test]
    fn seed_bodies_fit_the_bound() {
        for seed in SEED_PATTERNS {
            assert!(
                seed.body.len() <= TEMPLATE_BODY_BOUND,
                "seed body for {:?}/{:?} is {} bytes (>{})",
                seed.domain,
                seed.pattern_kind,
                seed.body.len(),
                TEMPLATE_BODY_BOUND
            );
        }
    }

    #[test]
    fn build_template_rejects_overlong_body_no_panic() {
        let big = "x".repeat(TEMPLATE_BODY_BOUND + 1);
        let res = build_template(
            Domain::Code,
            PatternKind::Decomposition,
            VerifierKind::Compile,
            &big,
            0,
        );
        assert!(res.is_err(), "overlong body must be rejected, not appended");
    }

    /// observable_effect smoke (off-agent): drive `migrate` against a temp
    /// archive + temp SSB, then read the ArchiveEntryAdded rows back FROM the
    /// SSB DB (an artifact this code path wrote, not its own stdout). This is
    /// the single sequential tokio test mandated by the env note — it does not
    /// touch the shared ELAI_*_ENABLE env.
    #[tokio::test]
    async fn migrate_writes_jsonl_and_emits_archive_entry_added_rows() {
        let dir = TempDir::new().expect("tempdir");
        let archive_path = dir.path().join("archive").join("reasoning-templates.jsonl");
        let ssb = temp_ssb(&dir);

        // First run: appends the full seed set.
        let report = migrate(&archive_path, None, &ssb)
            .await
            .expect("migrate ok");
        assert_eq!(
            report.extracted as usize,
            SEED_PATTERNS.len(),
            "first run extracts the whole seed set"
        );
        assert_eq!(report.skipped_duplicate, 0);
        assert_eq!(report.total_after as usize, SEED_PATTERNS.len());

        // Artifact 1: the JSONL file exists and verify_chain passes.
        assert!(archive_path.exists(), "jsonl artifact must exist");
        let reloaded = ArchiveChain::load(archive_path.clone()).expect("reload");
        assert_eq!(reloaded.len(), SEED_PATTERNS.len());
        reloaded.verify_chain().expect("clean chain");

        // Artifact 2 (off-agent): the SSB DB carries one ArchiveEntryAdded row
        // per appended entry, read back via query_by_type (NOT our own stdout).
        let rows = SsbReader::query_by_type(&*ssb, SsbActionType::ArchiveEntryAdded)
            .expect("query ssb");
        assert_eq!(
            rows.len(),
            SEED_PATTERNS.len(),
            "one ArchiveEntryAdded row per appended template"
        );
        for row in &rows {
            assert_eq!(row.action_type, SsbActionType::ArchiveEntryAdded);
        }

        // Idempotency: a second run on unchanged input extracts zero new
        // entries and emits zero new rows.
        let report2 = migrate(&archive_path, None, &ssb)
            .await
            .expect("migrate ok 2");
        assert_eq!(report2.extracted, 0, "second run is idempotent");
        assert_eq!(report2.skipped_duplicate as usize, SEED_PATTERNS.len());
        assert_eq!(report2.total_after as usize, SEED_PATTERNS.len());

        let rows_after = SsbReader::query_by_type(&*ssb, SsbActionType::ArchiveEntryAdded)
            .expect("query ssb 2");
        assert_eq!(
            rows_after.len(),
            SEED_PATTERNS.len(),
            "idempotent re-run emits no extra rows"
        );
    }
}
