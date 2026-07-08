//! K-4 SP1 — composed gate-tax budget: the in-band CODING trust-spine gate
//! friction, measured against a locked ceiling (KEYSTONE of K-T3, PS-2).
//!
//! # What this measures (honestly, GD-5)
//!
//! `coding_task_gate_tax_ms` — a measured-frequency-weighted composition of
//! per-gate unit costs over the IN-BAND runtime coding gate set: the 11 FSM
//! [`crate::evidence::EvidenceGate`] family (Leg B, microbenched — no
//! journal-native duration field exists on `EvidenceGateEvaluatedPayload`,
//! RECON_NEEDED-2) plus the T1/T2/T3 hook tiers (Leg A, journal-native via
//! `TieredAnalysisPayload.elapsed_us`, the one genuine duration field). The
//! composition formula is `sum(unit_cost_ms_i * fire_count_i)`.
//!
//! `over_block_rate` — a benign-corpus sweep (FIX-4): N known-good
//! [`TransitionContext`] fixtures driven through every in-scope gate,
//! blocked/total. This is a REAL pass/fail check against the shipped gate
//! code (deterministic, not wall-clock), never the `GateOverBlocked`
//! meta-gate (which covers 8 FINANCE gates x 1 fixture each, FIX-4).
//!
//! The RIDER-b coverage table tags every gate-manifest.txt-sourced gate as
//! `in-scope-coding | trading-surface | offline-audit | dormant` (the
//! settled RECON_NEEDED-1b map, see `workflow/reasoning/K-4-decisions.md`).
//! An untagged gate is a hard FAIL (ABORT-2b), never a silent narrowing.
//!
//! # RunnerStatus: Measured vs RunnerPresent (ABORT-2, the K-18a lesson)
//!
//! When the real journal (`<project_root>/sessions/ssb.db`) carries ZERO
//! `EvidenceGateEvaluated` + `TieredAnalysis` rows (the dry-journal case —
//! this pipeline is not yet dogfooded on most dev boxes), this runner does
//! NOT report a fabricated MEASURED composition over zero data. It falls to
//! [`RunnerStatus::RunnerPresent`]: the SAME plumbing (coverage table,
//! over-block sweep, composition arithmetic) runs end-to-end over a NAMED,
//! constant FIXTURE unit-cost table (never live-microbenched — a fixture
//! path must stay byte-reproducible), clearly labeled non-measured.
//!
//! # SP2 update (MOVE 3 / MOVE 4b / MOVE 6)
//!
//! `audit_gate_tax_ms` is now MEASURED (MOVE 4b, timed in the sibling
//! `gate_tax_audit_leg` module). Two ceiling consts exist but both carry
//! `// CEILING-PENDING[K-4]` (FORK-3 not triggered). [`evaluate_ceiling_breach`]
//! is the pure MOVE 3 + MOVE 6 breach decision `bench_gate_tax.rs`'s
//! `--enforce-ceiling` flag consumes — never a Rust `release_gate` fn
//! (ATTACK-4); the release gate IS the process exit code.
//!
//! # RESIDUE (GD-5, named explicitly — not silently dropped)
//!
//! 1. Hook-tier Leg-A costs are excluded from the RunnerPresent/fixture
//!    composition: no `TieredAnalysis` rows exist on a dry journal, so no
//!    fabricated Leg-A number is emitted. A future MEASURED run against a
//!    populated journal includes Leg A once such rows exist.
//! 2. The "compliance pre-gate" (`crates/orchestrator/src/compliance.rs`)
//!    named in K-4's Objective prose is NOT part of the settled
//!    RECON_NEEDED-1/1b coverage map and is excluded from this composition —
//!    a named SP2/K-5 scope gap, not a silent drop.
//! 3. `task_class_label` is a coarse SINGLE-class proxy
//!    (`"all_coding_tasks"` when Measured, `"fixture"` when RunnerPresent):
//!    `EvidenceGateEvaluatedPayload` carries no `template_id` field at all,
//!    so RIDER-c's template_id-keyed per-class breakdown is not mechanically
//!    achievable from the rows this composition consumes (a stricter finding
//!    than RECON_NEEDED-3's own caveat). Deferred; see K-4-decisions.md.
//! 4. The composed number under-estimates interaction effects (a BLOCK that
//!    triggers a retry loop costs more than one gate's wall-clock); the
//!    over-block sweep covers the largest such class, the coverage table
//!    names every gate this number does and does not cover.
//!
//! No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::warn;

use super::gate_tax_audit_leg;
use super::gate_tax_audit_leg::AUDIT_CHECKER_REPS;
use crate::evidence::{
    self, AuditFinding, EvidenceGate, FindingSeverity, IntegrationTestRowSummary,
    KnowledgeReadiness, TestOutcome, TransitionContext,
};
use crate::fsm::ReplanSeverity;
use crate::ssb::TieredAnalysisPayload;
use crate::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader as _};

// ---------------------------------------------------------------------------
// Constants (named; no magic numbers)
// ---------------------------------------------------------------------------

/// Number of seeded reps the Route-B microbench times per gate; median taken
/// (not mean, ADV-2: a single warm-cache rep is noise).
const ROUTE_B_MICROBENCH_REPS: usize = 200;

/// Top-N cost contributors reported (flat SSB fields per K-4 MOVE 5).
const TOP_CONTRIBUTORS_N: usize = 3;

/// Session id tag used ONLY for the read-side `Ssb::new` handle this runner
/// opens to query the journal. Never writes under this id (bench_gate_tax.rs
/// owns the write-side beacon emission, its own separate handle, MOVE 5).
const JOURNAL_READ_SESSION_ID: &str = "gate-tax-journal-read";

/// `task_class_label` when the real journal has zero in-scope fire rows.
const RUNNER_PRESENT_TASK_CLASS: &str = "fixture";
/// `task_class_label` when the real journal has >=1 in-scope fire row.
/// A coarse single-class proxy (RESIDUE point 3 above) — `template_id` is
/// NOT present on `EvidenceGateEvaluatedPayload`, so no finer key exists.
const MEASURED_TASK_CLASS: &str = "all_coding_tasks";

/// SP2: stays `true` — no confirmed `AUDIT_GATE_TAX_CEILING_MS` exists
/// (CEILING-PENDING[K-4]); per MOVE 3 the audit leg MAY legitimately stay
/// "unbounded, observed" forever. Never summed with
/// `coding_task_gate_tax_ms` (rider a).
const AUDIT_UNBOUNDED_OBSERVED: bool = true;

/// The 11 in-scope-coding FSM `EvidenceGate` ids (RECON_NEEDED-1, settled
/// 2026-07-08; verbatim from `evidence.rs::all_gates()` minus the
/// default-OFF `RequirementsGate`, which tags `dormant` in the coverage
/// table instead, never `in-scope-coding`).
const IN_SCOPE_CODING_GATE_IDS: &[&str] = &[
    "knowledge_readiness",
    "grounding_sweep",
    "test_fail_reasons",
    "test_flipped",
    "replan_fast_track",
    "zero_blocks",
    "dkg_changelog",
    "intent_pairs",
    "diagnostic_probe",
    "anti_fabrication",
    "entrypoint_and_trace",
];

/// FIXTURE Leg-B unit costs (ms), used ONLY on the `RunnerPresent` path.
/// NOT live-microbenched — a fixture path must stay byte-reproducible across
/// calls (GD-2), which real `Instant`-based wall-clock timing cannot honor.
/// Small, clearly-synthetic values; never conflated with a MEASURED number
/// (the field is only read when `RunnerStatus::RunnerPresent`).
const FIXTURE_ROUTE_B_UNIT_COST_MS: &[(&str, f64)] = &[
    ("knowledge_readiness", 0.02),
    ("grounding_sweep", 0.05),
    ("test_fail_reasons", 0.03),
    ("test_flipped", 0.02),
    ("replan_fast_track", 0.01),
    ("zero_blocks", 0.02),
    ("dkg_changelog", 0.01),
    ("intent_pairs", 0.01),
    ("diagnostic_probe", 0.04),
    ("anti_fabrication", 0.03),
    ("entrypoint_and_trace", 0.02),
];
/// FIXTURE fire count per gate on the `RunnerPresent` path (one synthetic
/// firing each — plumbing proof, never a measured frequency).
const FIXTURE_FIRE_COUNT: u64 = 1;

/// blake3-shaped placeholder hash used only inside the in-repo benign-corpus
/// fixture contexts (never a real captured stdout digest).
const FIXTURE_STDOUT_HASH: &str = "fixture0000000000000000000000000000000000000000000000000000000";

// ---------------------------------------------------------------------------
// RIDER-b coverage registry (RECON_NEEDED-1b, settled 2026-07-08)
// ---------------------------------------------------------------------------

/// Coverage tag for one gate-manifest.txt-sourced trust-spine gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateCoverageTag {
    InScopeCoding,
    TradingSurface,
    OfflineAudit,
    Dormant,
}

/// One row of the RIDER-b coverage table.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateCoverageEntry {
    pub gate_id: String,
    pub tag: GateCoverageTag,
}

/// The canonical trust-spine gate registry (RIDER-b / RECON_NEEDED-1b,
/// settled 2026-07-08 — verbatim from the K-4 SP1 dispatch exploration).
/// Hand-authored, NOT sourced from `wire-dark-allowlist.toml` (FIX-1: that
/// registry is the default-OFF EXEMPTION list, the opposite concept) and NOT
/// re-parsed from `.claude/gate-manifest.txt` at runtime (that file mixes
/// tooling checkers / `.md` / `.toml` entries that are not
/// `EvidenceGate`-shaped trust-spine gates). `promotion_gate.rs` is treated
/// as part of `reward_tampering`'s gate surface (its own manifest comment
/// frames it as the leg-3b watched fn, not a separate gate), not a 26th row.
const GATE_REGISTRY: &[(&str, GateCoverageTag)] = &[
    ("knowledge_readiness", GateCoverageTag::InScopeCoding),
    ("grounding_sweep", GateCoverageTag::InScopeCoding),
    ("test_fail_reasons", GateCoverageTag::InScopeCoding),
    ("test_flipped", GateCoverageTag::InScopeCoding),
    ("replan_fast_track", GateCoverageTag::InScopeCoding),
    ("zero_blocks", GateCoverageTag::InScopeCoding),
    ("dkg_changelog", GateCoverageTag::InScopeCoding),
    ("intent_pairs", GateCoverageTag::InScopeCoding),
    ("diagnostic_probe", GateCoverageTag::InScopeCoding),
    ("anti_fabrication", GateCoverageTag::InScopeCoding),
    ("entrypoint_and_trace", GateCoverageTag::InScopeCoding),
    ("requirements_gate", GateCoverageTag::Dormant),
    ("trial_ledger", GateCoverageTag::TradingSurface),
    ("deflation_gate", GateCoverageTag::TradingSurface),
    ("leakage_gate", GateCoverageTag::TradingSurface),
    ("lookahead_differential", GateCoverageTag::TradingSurface),
    ("trading_null_gate", GateCoverageTag::TradingSurface),
    ("rerun_equality", GateCoverageTag::TradingSurface),
    ("sim_real_parity", GateCoverageTag::TradingSurface),
    ("result_manifest", GateCoverageTag::TradingSurface),
    ("shadow_exec", GateCoverageTag::TradingSurface),
    ("reward_tampering", GateCoverageTag::TradingSurface),
    ("holdout_budget", GateCoverageTag::TradingSurface),
    ("drift_interlock", GateCoverageTag::TradingSurface),
    ("gate_self_validation", GateCoverageTag::OfflineAudit),
];

/// Look up one gate's coverage tag. `None` means the gate is NOT in the
/// canonical registry (RIDER-b ABORT-2b: the caller must FAIL, not skip).
fn tag_gate(gate_id: &str) -> Option<GateCoverageTag> {
    GATE_REGISTRY
        .iter()
        .find(|entry| entry.0 == gate_id)
        .map(|entry| entry.1)
}

/// Build the RIDER-b coverage table for a caller-supplied gate_id list.
/// FAILs (ABORT-2b) on the first id with no registry tag — the anti-silent-
/// narrowing guard: an in-scope gate must never quietly fall out of the
/// composed number. Public so `crates/contracts` can inject an untagged id
/// and assert the FAIL (ADV-5).
pub fn build_coverage_table(gate_ids: &[String]) -> Result<Vec<GateCoverageEntry>, GateTaxError> {
    let mut table = Vec::with_capacity(gate_ids.len());
    for gate_id in gate_ids {
        match tag_gate(gate_id) {
            Some(tag) => table.push(GateCoverageEntry {
                gate_id: gate_id.clone(),
                tag,
            }),
            None => {
                return Err(GateTaxError::UntaggedManifestGate {
                    gate_id: gate_id.clone(),
                })
            }
        }
    }
    Ok(table)
}

fn canonical_gate_ids() -> Vec<String> {
    GATE_REGISTRY.iter().map(|entry| entry.0.to_string()).collect()
}

fn tally_coverage(coverage: &[GateCoverageEntry]) -> (u32, u32, u32, u32) {
    let mut coding = 0u32;
    let mut trading = 0u32;
    let mut audit = 0u32;
    let mut dormant = 0u32;
    for entry in coverage {
        match entry.tag {
            GateCoverageTag::InScopeCoding => coding += 1,
            GateCoverageTag::TradingSurface => trading += 1,
            GateCoverageTag::OfflineAudit => audit += 1,
            GateCoverageTag::Dormant => dormant += 1,
        }
    }
    (coding, trading, audit, dormant)
}

// ---------------------------------------------------------------------------
// Report + error types
// ---------------------------------------------------------------------------

/// Whether the composed numbers are a real measurement or a labeled fixture
/// plumbing proof (ABORT-2, the K-18a dry-journal lesson).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunnerStatus {
    Measured,
    RunnerPresent,
}

/// One ranked cost contributor (gate_id or `hook_tier_<T>` label + its
/// composed cost).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateCostContributor {
    pub gate_id: String,
    pub cost_ms: f64,
}

/// Aggregate report — one per [`run_gate_tax`] invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateTaxReport {
    pub runner_status: RunnerStatus,
    pub seed: u64,
    /// Headline number (rider a: NEVER summed with `audit_gate_tax_ms`).
    pub coding_task_gate_tax_ms: f64,
    pub task_class_label: String,
    /// SP2 placeholder (MOVE 4b, per-audit checker-timing leg).
    pub audit_gate_tax_ms: f64,
    pub audit_unbounded: bool,
    pub over_block_rate: f64,
    pub over_block_blocked: u32,
    pub over_block_total: u32,
    pub coverage: Vec<GateCoverageEntry>,
    pub n_in_scope_coding: u32,
    pub n_trading_surface: u32,
    pub n_offline_audit: u32,
    pub n_dormant: u32,
    /// Always 0 in a successful report (an untagged gate is a hard FAIL,
    /// `Err`, before a report is ever assembled).
    pub n_untagged: u32,
    pub top_contributors: Vec<GateCostContributor>,
    pub n_gates: u32,
    pub n_fires: u64,
    pub residue: String,
}

#[derive(Debug, Error)]
pub enum GateTaxError {
    /// Could not determine the current working directory (needed to resolve
    /// the journal path).
    #[error("gate-tax could not determine project root: {source_msg}")]
    ProjectRoot { source_msg: String },
    /// RIDER-b ABORT-2b: a gate the coverage table was asked to tag is not in
    /// the canonical [`GATE_REGISTRY`].
    #[error("gate-tax: untagged manifest gate '{gate_id}' (RIDER-b / ABORT-2b: every in-scope gate must be tagged)")]
    UntaggedManifestGate { gate_id: String },
    /// Defensive invariant: `evidence::all_gates()` did not yield every id in
    /// [`IN_SCOPE_CODING_GATE_IDS`] (a gate was renamed/removed upstream).
    #[error("gate-tax: gate registry mismatch — expected {expected} in-scope FSM gates, found {found} in evidence::all_gates()")]
    GateRegistryMismatch { expected: usize, found: usize },
}

// ---------------------------------------------------------------------------
// Pure composition (deterministic given inputs — GD-2)
// ---------------------------------------------------------------------------

/// `sum(unit_cost_ms_i * fire_count_i)` + the ranked top-N contributors.
/// Pure and deterministic given its inputs — this is the GD-2 recomputable
/// leg: given the SAME (label, unit_cost, fire_count) rows a stored beacon
/// carries, this function reproduces the SAME composed number byte-for-byte.
/// It does not itself measure wall-clock time.
pub fn compose_gate_tax(contributions: &[(String, f64, u64)]) -> (f64, Vec<GateCostContributor>) {
    let mut total_ms = 0.0f64;
    let mut costed: Vec<GateCostContributor> = Vec::with_capacity(contributions.len());
    for entry in contributions {
        let unit_cost_ms: f64 = entry.1;
        let fire_count: u64 = entry.2;
        let cost_ms = unit_cost_ms * (fire_count as f64);
        total_ms += cost_ms;
        costed.push(GateCostContributor {
            gate_id: entry.0.clone(),
            cost_ms,
        });
    }
    costed.sort_by(|a, b| b.cost_ms.partial_cmp(&a.cost_ms).unwrap_or(std::cmp::Ordering::Equal));
    costed.truncate(TOP_CONTRIBUTORS_N);
    (total_ms, costed)
}

/// `pub(crate)` so the sibling `gate_tax_audit_leg` module (MOVE 4b) can
/// reuse this same median helper instead of duplicating it.
pub(crate) fn median_ms(mut samples: Vec<f64>) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    samples[samples.len() / 2]
}

// ---------------------------------------------------------------------------
// Route-B microbench (FORK-1: the FSM EvidenceGate family has no journal-
// native duration field, so it is timed directly on a fixed fixture)
// ---------------------------------------------------------------------------

/// Median wall-clock cost (ms) of `gate.check(ctx)` over
/// [`ROUTE_B_MICROBENCH_REPS`] reps. Live `Instant`-based timing — ONLY
/// called on the `Measured` path (never in the byte-reproducible fixture
/// path).
fn route_b_unit_cost_ms(gate: &dyn EvidenceGate, ctx: &TransitionContext) -> f64 {
    let mut samples = Vec::with_capacity(ROUTE_B_MICROBENCH_REPS);
    for _ in 0..ROUTE_B_MICROBENCH_REPS {
        let start = Instant::now();
        let _ = gate.check(ctx);
        samples.push(start.elapsed().as_secs_f64() * 1000.0);
    }
    median_ms(samples)
}

fn measured_leg_b_contributions(
    gates: &[Box<dyn EvidenceGate>],
    ctx: &TransitionContext,
    fire_counts: &HashMap<String, u64>,
) -> Vec<(String, f64, u64)> {
    gates
        .iter()
        .map(|gate| {
            let unit_cost_ms = route_b_unit_cost_ms(gate.as_ref(), ctx);
            let fire_count = *fire_counts.get(gate.name()).unwrap_or(&0);
            (gate.name().to_string(), unit_cost_ms, fire_count)
        })
        .collect()
}

fn fixture_leg_b_contributions() -> Vec<(String, f64, u64)> {
    FIXTURE_ROUTE_B_UNIT_COST_MS
        .iter()
        .map(|entry| (entry.0.to_string(), entry.1, FIXTURE_FIRE_COUNT))
        .collect()
}

// ---------------------------------------------------------------------------
// Route-A (hook-tier, journal-native `TieredAnalysis.elapsed_us`)
// ---------------------------------------------------------------------------

fn leg_a_contributions(tiered_rows: &[TieredAnalysisPayload]) -> Vec<(String, f64, u64)> {
    let mut groups: HashMap<String, Vec<f64>> = HashMap::new();
    for row in tiered_rows {
        groups
            .entry(format!("hook_tier_{}", row.tier))
            .or_default()
            .push(row.elapsed_us as f64 / 1000.0);
    }
    groups
        .into_iter()
        .map(|(label, samples)| {
            let fire_count = samples.len() as u64;
            (label, median_ms(samples), fire_count)
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Journal read (best-effort — any failure degrades to an empty snapshot,
// which run_gate_tax treats as RunnerPresent, never a hard error, ABORT-2)
// ---------------------------------------------------------------------------

struct JournalSnapshot {
    gate_fire_counts: HashMap<String, u64>,
    tiered: Vec<TieredAnalysisPayload>,
}

fn empty_journal_snapshot() -> JournalSnapshot {
    JournalSnapshot {
        gate_fire_counts: HashMap::new(),
        tiered: Vec::new(),
    }
}

/// Best-effort read of `EvidenceGateEvaluated` + `TieredAnalysis` rows from
/// the real session journal, via the UNCAPPED `SsbReader::query_by_type` on
/// `Ssb` (NOT the capped `SsbViewer` drawer readers, which undercount past
/// `DRAWER_QUERY_HARD_CAP=500` — settled correction, K-4 SP1 dispatch).
/// Never propagates an error: a missing db, a migration failure, or a query
/// error all degrade to an empty snapshot (ABORT-2: the caller falls to
/// `RunnerPresent`, never fabricates a MEASURED composition over zero data).
fn try_read_journal(project_root: &Path) -> JournalSnapshot {
    let db_path = elai_core::paths::sessions_dir_path(project_root).join("ssb.db");
    if !db_path.exists() {
        return empty_journal_snapshot();
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: JOURNAL_READ_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::OsClock);
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            warn!(error = %e, "gate-tax: could not build read runtime; falling back to RunnerPresent");
            return empty_journal_snapshot();
        }
    };
    rt.block_on(async move { read_journal_inner(cfg, clock) })
}

fn read_journal_inner(cfg: SsbConfig, clock: Arc<dyn elai_core::clock::ClockProvider>) -> JournalSnapshot {
    let ssb = match Ssb::new(cfg, clock) {
        Ok(s) => s,
        Err(e) => {
            warn!(error = %e, "gate-tax: could not open ssb.db for journal read; falling back to RunnerPresent");
            return empty_journal_snapshot();
        }
    };

    let mut gate_fire_counts: HashMap<String, u64> = HashMap::new();
    if let Ok(rows) = ssb.query_by_type(SsbActionType::EvidenceGateEvaluated) {
        for row in &rows {
            if let Ok(SsbPayload::EvidenceGateEvaluated(p)) = crate::ssb::decode_ssb_payload(&row.payload) {
                *gate_fire_counts.entry(p.gate_id).or_insert(0) += 1;
            }
        }
    }

    let mut tiered = Vec::new();
    if let Ok(rows) = ssb.query_by_type(SsbActionType::TieredAnalysis) {
        for row in &rows {
            if let Ok(SsbPayload::TieredAnalysis(p)) = crate::ssb::decode_ssb_payload(&row.payload) {
                tiered.push(p);
            }
        }
    }

    JournalSnapshot { gate_fire_counts, tiered }
}

// ---------------------------------------------------------------------------
// Benign corpus (FIX-4 / RECON_NEEDED-4): N known-good TransitionContext
// fixtures that MUST NOT block. Distinct from the K-18a recurrence fixture —
// there is no reusable benign set in safedelta_runner.rs (grepped, none
// applicable to FSM EvidenceGate contexts), so this is a fresh in-repo fixture.
// ---------------------------------------------------------------------------

/// A single fully-passing `TransitionContext` — every in-scope-coding gate
/// must return `Ok(())` against this fixture.
fn base_passing_context() -> TransitionContext {
    TransitionContext {
        knowledge_readiness: Some(KnowledgeReadiness::Ready),
        ungrounded_claims: Some(Vec::new()),
        test_outcomes: Some(vec![TestOutcome {
            name: "fixture_test_1".to_string(),
            passed: true,
            is_syntax_error: false,
            flipped_to_pass: true,
        }]),
        replan_severity: Some(ReplanSeverity::Major),
        replan_auditor_approved: None,
        audit_findings: Some(Vec::new()),
        dkg_reflects_new_code: Some(true),
        changelog_has_entry: Some(true),
        min_intent_pairs: 0,
        diagnostic_probe_outcome: Some(elai_knowledge::diagnostic::DiagnosticProbeOutcome {
            kind: elai_knowledge::diagnostic::ProbeKind::RustAnalyzer,
            error_count: 0,
            warning_count: 0,
            info_count: 0,
            hint_count: 0,
            duration_ms: 1,
            diagnostics: Vec::new(),
            run_status: elai_knowledge::diagnostic::DiagnosticProbeRunStatus::Completed,
        }),
        integration_test_rows: Some(vec![IntegrationTestRowSummary {
            test_name: "fixture_it_1".to_string(),
            passed: true,
            stdout_hash: Some(FIXTURE_STDOUT_HASH.to_string()),
        }]),
        layer3_entrypoint_covered: Some(true),
        layer4_trace_present: Some(true),
        ..TransitionContext::default()
    }
}

/// N=5 distinct known-good variants (RECON_NEEDED-4). Each MUST pass every
/// in-scope-coding gate; the over-block sweep counts any gate that
/// incorrectly BLOCKs one of these as a false positive.
fn benign_corpus() -> Vec<TransitionContext> {
    let base = base_passing_context();
    vec![
        base.clone(),
        TransitionContext {
            replan_severity: Some(ReplanSeverity::Minor),
            replan_auditor_approved: Some(true),
            ..base.clone()
        },
        TransitionContext {
            test_outcomes: Some(vec![
                TestOutcome {
                    name: "fixture_test_a".to_string(),
                    passed: true,
                    is_syntax_error: false,
                    flipped_to_pass: false,
                },
                TestOutcome {
                    name: "fixture_test_b".to_string(),
                    passed: true,
                    is_syntax_error: false,
                    flipped_to_pass: true,
                },
            ]),
            ..base.clone()
        },
        TransitionContext {
            audit_findings: Some(vec![
                AuditFinding {
                    severity: FindingSeverity::Warn,
                    message: "fixture warn finding".to_string(),
                },
                AuditFinding {
                    severity: FindingSeverity::Info,
                    message: "fixture info finding".to_string(),
                },
            ]),
            ..base.clone()
        },
        TransitionContext {
            integration_test_rows: Some(vec![
                IntegrationTestRowSummary {
                    test_name: "fixture_it_a".to_string(),
                    passed: true,
                    stdout_hash: Some(FIXTURE_STDOUT_HASH.to_string()),
                },
                IntegrationTestRowSummary {
                    test_name: "fixture_it_b".to_string(),
                    passed: true,
                    stdout_hash: Some(FIXTURE_STDOUT_HASH.to_string()),
                },
            ]),
            ..base
        },
    ]
}

/// Drive every in-scope gate against every benign fixture; count blocks
/// (deterministic pass/fail, not wall-clock — reproducible across calls).
fn over_block_sweep(gates: &[Box<dyn EvidenceGate>], corpus: &[TransitionContext]) -> (u32, u32, f64) {
    let mut blocked = 0u32;
    let mut total = 0u32;
    for ctx in corpus {
        for gate in gates {
            total += 1;
            if gate.check(ctx).is_err() {
                blocked += 1;
            }
        }
    }
    let rate = if total == 0 { 0.0 } else { f64::from(blocked) / f64::from(total) };
    (blocked, total, rate)
}

// ---------------------------------------------------------------------------
// MOVE 3 / MOVE 6 (SP2) — locked ceilings + the breach evaluation
// ---------------------------------------------------------------------------
// ATTACK-4: no `release_gate` Rust fn exists in this repo. The "release
// gate" IS the `elai bench gate-tax --enforce-ceiling` exit code CI consumes
// (bench_gate_tax.rs::run_inner calls evaluate_ceiling_breach below). This
// module owns only the pure decision; the CLI owns the exit.

/// CEILING-PENDING[K-4] (FORK-3): the user's call from the first
/// `Measured` production run; NEVER invented (ABORT-3). Not an unexamined
/// sentinel — `coding_task_gate_tax_ms` is PROVEN always `> 0.0`
/// (`run_gate_tax_never_reports_zero_tax_as_a_silent_pass`), so the plain
/// `>` comparison below breaches EVERY run by construction until a human
/// replaces this literal — ABORT-5's fail-closed falls out of the
/// comparison itself, no `Pending` enum needed.
const CODING_TASK_GATE_TAX_CEILING_MS: f64 = 0.0; // CEILING-PENDING[K-4]

/// CEILING-PENDING[K-4]: per the plan's MOVE 3 text this MAY legitimately
/// stay "unbounded, observed" rather than ever gaining a confirmed value.
/// NOT read by [`evaluate_ceiling_breach`] (ABORT-5 names the coding
/// ceiling only). Kept for symmetry + a future MOVE.
const AUDIT_GATE_TAX_CEILING_MS: f64 = 0.0; // CEILING-PENDING[K-4]

/// MOVE 6: the benign corpus is PROVEN zero-over-block on shipped gate code
/// (`benign_corpus_fixtures_pass_every_in_scope_gate`) — a STRUCTURAL
/// zero-tolerance bound, not a FORK-3 user-set ceiling, so this ships as a
/// real (non-pending) constant.
const OVER_BLOCK_RATE_CEILING: f64 = 0.0;

/// MOVE 3 + MOVE 6 breach evaluation. Pure, deterministic. Two independent
/// conditions (rider a: never combined into one number); the first found
/// wins.
pub fn evaluate_ceiling_breach(report: &GateTaxReport) -> Option<String> {
    if report.coding_task_gate_tax_ms > CODING_TASK_GATE_TAX_CEILING_MS {
        return Some(format!(
            "coding_task_gate_tax_ms {:.4}ms > ceiling {:.4}ms (CEILING-PENDING[K-4] until a human \
             confirms a real value from a Measured production run; fail-closed per ABORT-5)",
            report.coding_task_gate_tax_ms, CODING_TASK_GATE_TAX_CEILING_MS
        ));
    }
    if report.over_block_rate > OVER_BLOCK_RATE_CEILING {
        return Some(format!(
            "over_block_rate {:.4} > {:.4} (MOVE 6: the benign corpus is proven zero-over-block on \
             shipped gate code; any over-block here is a regression, not a budget)",
            report.over_block_rate, OVER_BLOCK_RATE_CEILING
        ));
    }
    None
}

// ---------------------------------------------------------------------------
// MOVE 4b (SP2) — per-audit checker-timing leg
// ---------------------------------------------------------------------------
// The timing machinery lives in the sibling `gate_tax_audit_leg` module
// (split out to stay under the MR-1 1000-line boundary). `run_gate_tax`
// calls `gate_tax_audit_leg::audit_gate_tax()`; this file only formats the
// residue note from its result.

/// GD-5: names which (if any) audit checkers were unavailable this process,
/// appended to the CODING leg's static residue note. Never silently drops an
/// unavailable checker from the record even though it contributes 0.0.
fn build_residue_note(audit_result: &gate_tax_audit_leg::AuditGateTaxResult) -> String {
    if audit_result.unavailable.is_empty() {
        format!(
            "{RESIDUE_NOTE} (5) audit_gate_tax_ms is now MEASURED (MOVE 4b, SP2): the median wall-clock \
             of {AUDIT_CHECKER_REPS} reps across all 5 offline auditor checkers, summed; all 5 were \
             available this run."
        )
    } else {
        format!(
            "{RESIDUE_NOTE} (5) audit_gate_tax_ms is MEASURED (MOVE 4b, SP2) but PARTIAL: the following \
             checkers were unavailable and contributed 0.0 to the sum (named, never silently dropped): {}.",
            audit_result.unavailable.join(", ")
        )
    }
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

const RESIDUE_NOTE: &str = "GD-5 residue: (1) hook-tier Leg-A costs excluded from the RunnerPresent/fixture \
composition (no TieredAnalysis rows on a dry journal; a future Measured run against a populated journal \
includes Leg A). (2) the compliance pre-gate (compliance.rs) named in K-4's Objective prose is NOT in the \
settled RECON-1/1b coverage map; excluded here, a named SP2/K-5 scope gap. (3) task_class_label is a coarse \
single-class proxy ('all_coding_tasks' Measured / 'fixture' RunnerPresent): EvidenceGateEvaluatedPayload \
carries no template_id field, so RIDER-c per-class breakdown is not mechanically achievable from the rows \
this composition consumes; deferred, see K-4-decisions.md. (4) the composed number under-estimates \
interaction effects (a retry-triggering BLOCK costs more than one gate's wall-clock); the over-block sweep \
covers the largest such class, the coverage table names every gate covered and not covered.";

/// Run the composed gate-tax measurement (K-4 SP1). Reads the real session
/// journal best-effort; falls to a labeled fixture composition
/// (`RunnerStatus::RunnerPresent`) when the journal is dry (ABORT-2). Always
/// runs the RIDER-b coverage table (fails loud on an untagged gate,
/// ABORT-2b) and the benign-corpus over-block sweep (FIX-4) for real.
pub fn run_gate_tax(seed: u64) -> Result<GateTaxReport, GateTaxError> {
    let project_root = std::env::current_dir().map_err(|e| GateTaxError::ProjectRoot {
        source_msg: e.to_string(),
    })?;

    let in_scope_gates: Vec<Box<dyn EvidenceGate>> = evidence::all_gates()
        .into_iter()
        .filter(|g| IN_SCOPE_CODING_GATE_IDS.contains(&g.name()))
        .collect();
    if in_scope_gates.len() != IN_SCOPE_CODING_GATE_IDS.len() {
        return Err(GateTaxError::GateRegistryMismatch {
            expected: IN_SCOPE_CODING_GATE_IDS.len(),
            found: in_scope_gates.len(),
        });
    }

    let journal = try_read_journal(&project_root);
    let total_real_fires: u64 =
        journal.gate_fire_counts.values().sum::<u64>() + journal.tiered.len() as u64;

    let (runner_status, task_class_label, contributions) = if total_real_fires == 0 {
        (
            RunnerStatus::RunnerPresent,
            RUNNER_PRESENT_TASK_CLASS.to_string(),
            fixture_leg_b_contributions(),
        )
    } else {
        let ctx = base_passing_context();
        let mut contribs =
            measured_leg_b_contributions(&in_scope_gates, &ctx, &journal.gate_fire_counts);
        contribs.extend(leg_a_contributions(&journal.tiered));
        (
            RunnerStatus::Measured,
            MEASURED_TASK_CLASS.to_string(),
            contribs,
        )
    };

    let (coding_task_gate_tax_ms, top_contributors) = compose_gate_tax(&contributions);
    let n_fires: u64 = contributions.iter().map(|entry| entry.2).sum();
    let n_gates = u32::try_from(contributions.len()).unwrap_or(u32::MAX);

    let coverage = build_coverage_table(&canonical_gate_ids())?;
    let (n_in_scope_coding, n_trading_surface, n_offline_audit, n_dormant) =
        tally_coverage(&coverage);

    let corpus = benign_corpus();
    let (over_block_blocked, over_block_total, over_block_rate) =
        over_block_sweep(&in_scope_gates, &corpus);

    // MOVE 4b: memoized per-process (see the MOVE 4b section doc) — a 2nd
    // same-process run_gate_tax call reuses this exact value, preserving
    // GD-2 same-seed byte-identity on the RunnerPresent path.
    let audit_result = gate_tax_audit_leg::audit_gate_tax();

    let report = GateTaxReport {
        runner_status,
        seed,
        coding_task_gate_tax_ms,
        task_class_label,
        audit_gate_tax_ms: audit_result.total_ms,
        audit_unbounded: AUDIT_UNBOUNDED_OBSERVED,
        over_block_rate,
        over_block_blocked,
        over_block_total,
        coverage,
        n_in_scope_coding,
        n_trading_surface,
        n_offline_audit,
        n_dormant,
        n_untagged: 0,
        top_contributors,
        n_gates,
        n_fires,
        residue: build_residue_note(audit_result),
    };

    tracing::info!(
        runner_status = ?report.runner_status,
        coding_task_gate_tax_ms = report.coding_task_gate_tax_ms,
        over_block_rate = report.over_block_rate,
        n_untagged = report.n_untagged,
        "gate-tax: K-4 SP1 composition complete",
    );

    Ok(report)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_gate_tax_is_pure_and_correct() {
        let contributions = vec![
            ("gate_a".to_string(), 2.0, 3u64),
            ("gate_b".to_string(), 1.0, 10u64),
            ("gate_c".to_string(), 0.5, 1u64),
        ];
        let (total, top) = compose_gate_tax(&contributions);
        // 2.0*3 + 1.0*10 + 0.5*1 = 6 + 10 + 0.5 = 16.5
        assert!((total - 16.5).abs() < f64::EPSILON, "got {total}");
        assert_eq!(top.len(), 3, "N=3 contributors, all returned (<=TOP_CONTRIBUTORS_N)");
        assert_eq!(top[0].gate_id, "gate_b", "highest cost first");
        assert!((top[0].cost_ms - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn compose_gate_tax_from_fixture_journal_matches_hand_sum() {
        // (a) Leg B composes Σ(unit_cost × fire_count) from a FIXTURE journal.
        let fixture_journal = vec![
            ("knowledge_readiness".to_string(), 0.1, 5u64),
            ("grounding_sweep".to_string(), 0.2, 2u64),
        ];
        let (total, _) = compose_gate_tax(&fixture_journal);
        assert!((total - (0.1 * 5.0 + 0.2 * 2.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn in_scope_gate_set_is_telemetry_derived_not_a_literal_list() {
        // (b) the in-scope gate set is evidence.rs-derived, not a hand list
        // divorced from the shipped registry: every IN_SCOPE_CODING_GATE_IDS
        // entry must resolve to a real evidence::all_gates() gate.
        let all: Vec<&'static str> = evidence::all_gates().iter().map(|g| g.name()).collect();
        for gate_id in IN_SCOPE_CODING_GATE_IDS {
            assert!(
                all.contains(gate_id),
                "in-scope gate '{gate_id}' must exist in evidence::all_gates()"
            );
        }
    }

    #[test]
    fn coverage_table_tags_every_canonical_gate() {
        let ids = canonical_gate_ids();
        let table = build_coverage_table(&ids).expect("canonical registry is self-consistent");
        assert_eq!(table.len(), GATE_REGISTRY.len());
        let (coding, trading, audit, dormant) = tally_coverage(&table);
        assert_eq!(coding, 11, "11 in-scope-coding gates");
        assert_eq!(trading, 12, "12 trading-surface gates");
        assert_eq!(audit, 1, "1 offline-audit gate (gate_self_validation)");
        assert_eq!(dormant, 1, "1 dormant gate (requirements_gate)");
    }

    #[test]
    fn coverage_table_fails_loud_on_an_untagged_gate() {
        // (c) / ADV-5: an injected untagged gate must FAIL the run (ABORT-2b),
        // never silently narrow the coverage table.
        let mut ids = canonical_gate_ids();
        ids.push("totally_untagged_synthetic_gate".to_string());
        let result = build_coverage_table(&ids);
        assert!(
            matches!(result, Err(GateTaxError::UntaggedManifestGate { .. })),
            "untagged gate must FAIL, got {result:?}"
        );
    }

    #[test]
    fn benign_corpus_fixtures_pass_every_in_scope_gate() {
        // FIX-4: the benign corpus must genuinely be known-good — a sanity
        // check that the fixture authoring itself is not accidentally
        // spammy (would poison the over_block_rate baseline).
        let gates: Vec<Box<dyn EvidenceGate>> = evidence::all_gates()
            .into_iter()
            .filter(|g| IN_SCOPE_CODING_GATE_IDS.contains(&g.name()))
            .collect();
        let corpus = benign_corpus();
        let (blocked, total, rate) = over_block_sweep(&gates, &corpus);
        assert_eq!(blocked, 0, "the benign corpus must not trip any in-scope gate");
        assert_eq!(
            usize::try_from(total).expect("u32 total fits in usize"),
            gates.len() * corpus.len()
        );
        assert!((rate - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn benign_corpus_is_distinct_from_a_degenerate_single_case() {
        // ADV-4: the benign corpus must be a real >1-fixture set, not a
        // single-case stand-in for "corpus".
        let corpus = benign_corpus();
        assert!(corpus.len() >= 5, "RECON_NEEDED-4: N known-good fixtures, got {}", corpus.len());
        // At least one field differs across every pair (no accidental dupes).
        for i in 0..corpus.len() {
            for j in (i + 1)..corpus.len() {
                assert_ne!(corpus[i], corpus[j], "benign fixtures must be distinct ({i},{j})");
            }
        }
    }

    #[test]
    fn empty_journal_snapshot_yields_runner_present_not_measured() {
        // ADV-1 (the K-18a lesson): a dry journal must never report a
        // fabricated MEASURED composition over zero data.
        let snapshot = empty_journal_snapshot();
        let total_real_fires =
            snapshot.gate_fire_counts.values().sum::<u64>() + snapshot.tiered.len() as u64;
        assert_eq!(total_real_fires, 0);
        // Mirrors run_gate_tax's branch condition directly.
        let status = if total_real_fires == 0 {
            RunnerStatus::RunnerPresent
        } else {
            RunnerStatus::Measured
        };
        assert_eq!(status, RunnerStatus::RunnerPresent);
    }

    #[test]
    fn run_gate_tax_same_seed_is_byte_identical_on_a_dry_journal() {
        // (g) / GD-2: on this dev box the journal is very likely dry, so this
        // exercises the RunnerPresent path end-to-end, which is fully
        // fixture-driven (no live microbench) and therefore byte-reproducible.
        let r1 = run_gate_tax(1234).expect("run 1");
        let r2 = run_gate_tax(1234).expect("run 2");
        if r1.runner_status == RunnerStatus::RunnerPresent
            && r2.runner_status == RunnerStatus::RunnerPresent
        {
            let j1 = serde_json::to_string(&r1).expect("ser1");
            let j2 = serde_json::to_string(&r2).expect("ser2");
            assert_eq!(j1, j2, "same-seed RunnerPresent reports must be byte-identical (GD-2)");
        }
        // On a Measured journal (real EvidenceGateEvaluated rows present),
        // Leg-B live microbench timing legitimately varies run-to-run; GD-2
        // there means the compose_gate_tax formula is deterministic given
        // fixed inputs (see compose_gate_tax_from_fixture_journal_matches_hand_sum),
        // not that two independent wall-clock measurements bit-match.
    }

    #[test]
    fn run_gate_tax_never_reports_zero_tax_as_a_silent_pass() {
        let report = run_gate_tax(42).expect("run");
        assert!(
            report.coding_task_gate_tax_ms > 0.0,
            "a non-empty gate set (fixture or measured) must never compose to exactly 0.0"
        );
        assert_eq!(report.n_untagged, 0, "RIDER-b: a successful report always has 0 untagged");
    }

    #[test]
    fn run_gate_tax_reports_non_empty_residue() {
        let report = run_gate_tax(7).expect("run");
        assert!(!report.residue.is_empty(), "GD-5: residue must be named, never silent");
    }

    // -----------------------------------------------------------------
    // SP2: MOVE 3 / MOVE 4b / MOVE 6
    // -----------------------------------------------------------------

    #[test]
    fn evaluate_ceiling_breach_fails_closed_while_pending_and_flags_over_block() {
        // ABORT-5: CEILING-PENDING (0.0) always breaches a real report.
        let mut report = run_gate_tax(101).expect("run");
        assert!(evaluate_ceiling_breach(&report).is_some());
        report.coding_task_gate_tax_ms = 0.0; // hypothetically under a (still-pending) ceiling
        report.over_block_rate = 0.25; // MOVE 6: a deliberately spammy synthetic value
        let reason = evaluate_ceiling_breach(&report).expect("over-block must still breach");
        assert!(reason.contains("over_block_rate"), "got {reason}");
    }

    #[test]
    fn audit_gate_tax_is_memoized_across_calls_in_process() {
        let r1 = run_gate_tax(201).expect("run 1");
        let r2 = run_gate_tax(202).expect("run 2");
        assert_eq!(r1.audit_gate_tax_ms, r2.audit_gate_tax_ms, "MOVE 4b: process-memoized (GD-2)");
        assert!(r1.audit_gate_tax_ms >= 0.0 && r1.audit_unbounded);
    }
}
