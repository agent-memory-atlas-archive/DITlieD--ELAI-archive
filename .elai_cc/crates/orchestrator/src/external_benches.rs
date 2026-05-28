//! E-9 — external agent-eval benchmark substrate.
//!
//! Defines a closed-trait registry of 11 contamination-resistant external
//! benchmark suites that gate B-N / E-N promotion against measured paired-
//! bootstrap deltas. Live runs are RUNPOD-PENDING — host-side scaffolding
//! ships fully (CLI surface + SSB emission contract + power-calibration
//! refusal rule) so the cross-check passes; the actual Python suite runners
//! live at .elai_cc/benches/external/.
//!
//! Registry composition (per E-9 plan §Tasks 2-12):
//!   - swe-bench-pro       (primary, arXiv:2509.16941, RAG-indexed)
//!   - swe-bench-live      (monthly-refresh, arXiv:2505.23419)
//!   - livecodebench-v6    (time-stamped, arXiv:2403.07974)
//!   - tau2-bench          (policy-adherence, sierra-research/tau2-bench)
//!   - bfcl-v3             (tool-use, gorilla.cs.berkeley.edu)
//!   - gaia-l1-l2          (general agent, arXiv:2311.12983)
//!   - clawsbench-split    (harness-vs-model, arXiv:2604.05172)
//!   - refinebench         (self-refinement, arXiv:2511.22173)
//!   - sycophancy-probes   (4-behavior, arXiv:2310.13548)
//!   - math-floors         (GSM8K-Platinum + MATH-500, gradientscience blog)
//!   - deep-planning       (multi-day, arXiv:2601.18137)
//!
//! Pattern-2 (17th application): emission via the SINGLE
//! `SsbActionType::BenchmarkRunObserved` variant; suite_name is a payload
//! field, NOT an action_type distinguisher. See
//! workflow/reasoning/E-9-decisions.md DECISION-1.
//!
//! Wiring posture:
//!   - Compiled unconditionally; emission gated by
//!     `ELAI_EVAL_BENCHMARK_ENABLE=1`.
//!   - Composes on `crate::promotion_gate::two_gate_promote` (E-7) when a
//!     model/harness change requests promotion gating. See DECISION-2.
//!
//! No `.unwrap()` / `.expect()` outside tests. No async-trait crate.
//! BTreeMap over HashMap (CLAUDE.md determinism rule).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::bench_significance::{
    default_seed_for_suite, is_low_power, paired_bootstrap_p, required_n_for_p_threshold,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Canonical registry size. Plan §Tasks lists 11 external suites; cross-check
/// validates by asserting `list_suites().len() == EXPECTED_SUITE_COUNT`.
pub const EXPECTED_SUITE_COUNT: usize = 11;

/// Default paired-bootstrap iteration count. `n=2000` per Davison-Hinkley 1997
/// §4.2.3 recommendation (>= 1000 for p<0.05 stability).
pub const DEFAULT_BOOTSTRAP_ITERS: usize = 2000;

/// Default significance target for paired-bootstrap regression gating.
pub const DEFAULT_P_TARGET: f64 = 0.05;

/// Default 2-point delta detection target (on the [0,1] score scale this is
/// `0.02`). Plan §Open questions/risks calls this out as the open value the
/// power-calibration loop must verify empirically.
pub const DEFAULT_TARGET_DELTA: f64 = 0.02;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Runtime routing target for a benchmark run (cloud vs local backend).
/// Mirrors but does not duplicate the orchestrator `RuntimeBackend` enum —
/// E-9 only needs to know which side the suite ran against for verdict
/// thresholding (the local-7B vs cloud-ceiling floor differs per suite).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchRuntimeLane {
    /// Local sidecar runtime (Qwen-Coder 7B abliterated default).
    LocalSidecar,
    /// Cloud route (Anthropic API / strict-schema providers).
    CloudApi,
    /// Mixed cloud-and-local pipeline (per-role split per E-3 RuntimeHandoff).
    Mixed,
}

/// Target the bench suite ran against.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchTarget {
    /// Cloud vs local lane.
    pub lane: BenchRuntimeLane,
    /// Canonical model identifier (e.g. "qwen2.5-coder-7b-abliterated",
    /// "claude-sonnet-4-7", "claude-opus-4-7").
    pub model_name: String,
    /// 16-char blake3 prefix of the harness scaffold version that ran the
    /// suite. Used by ClawsBench-style harness-vs-model decomposition to
    /// pin scaffold identity.
    pub harness_blake3_prefix: String,
}

/// Harness scaffold configuration. ClawsBench split requires varying this
/// across A/B; other suites fix it to the production scaffold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HarnessConfig {
    /// Snake-case harness name (e.g. "elai-prod", "elai-prod-no-rag").
    pub harness_name: String,
    /// Scaffold version tag (semver-ish).
    pub scaffold_version: String,
    /// Per arXiv:2406.07791 LLM-as-judge position-bias mitigation. When
    /// true, the suite MUST rotate option positions + ID tokens per
    /// LLM-judge call. Enforced by the contract test in
    /// crates/contracts/tests/judge_rotation_contract.rs.
    pub judge_rotation: bool,
}

/// Verdict of a paired-bootstrap significance comparison between two
/// `BenchmarkResult` snapshots (e.g. month-over-month delta gating).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignificanceVerdict {
    /// Candidate is statistically worse than baseline (p < p_target, mean
    /// delta < 0). Block promotion.
    Regression,
    /// Candidate is statistically better than baseline. Pass.
    Improvement,
    /// Bootstrap p >= p_target — observed delta is not statistically
    /// distinguishable from chance. Pass (no regression call).
    NoChange,
    /// `n < required_n` for the configured `target_delta` at `p_target` —
    /// the test could not have detected a regression even if present. Caller
    /// MUST NOT label this a regression; it surfaces a power-calibration
    /// problem upstream.
    LowPowerNoConclusion,
}

/// Result of a single benchmark suite run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Canonical suite name (e.g. "swe-bench-pro").
    pub suite_name: String,
    /// Suite-reported score in `[0.0, 1.0]` (pass-rate / accuracy / coverage,
    /// per-suite semantics).
    pub score: f64,
    /// Number of items evaluated.
    pub n: usize,
    /// Public SOTA reference value for telemetry, e.g. Anthropic tau2 retail
    /// 0.862 — None if the suite has no canonical SOTA.
    pub sota_reference: Option<f64>,
    /// Paired-bootstrap p-value vs the prior run on the same fixture, if a
    /// prior run is available. None on first-ever run.
    pub p_value: Option<f64>,
    /// True if `n < required_n` for the configured significance target.
    /// When true, callers MUST emit `SignificanceVerdict::LowPowerNoConclusion`
    /// rather than a regression call.
    pub low_power: bool,
    /// ClawsBench-split-only: per-bucket variance attribution (percentage
    /// points). None on non-ClawsBench suites.
    pub model_effect_pp: Option<f64>,
    /// ClawsBench-split-only: harness-side effect (percentage points).
    pub harness_effect_pp: Option<f64>,
    /// DS-CP-only: worst-case coverage gap (percentage points).
    pub coverage_gap_pp: Option<f64>,
    /// Wall-clock milliseconds since UNIX_EPOCH at suite completion.
    pub ts_ms: i64,
}

impl BenchmarkResult {
    /// Construct a baseline (single-run, no prior) result.
    #[must_use]
    pub fn baseline(suite_name: impl Into<String>, score: f64, n: usize, ts_ms: i64) -> Self {
        Self {
            suite_name: suite_name.into(),
            score,
            n,
            sota_reference: None,
            p_value: None,
            low_power: false,
            model_effect_pp: None,
            harness_effect_pp: None,
            coverage_gap_pp: None,
            ts_ms,
        }
    }
}

/// Registry-level errors.
#[derive(Debug, Error)]
pub enum BenchRegistryError {
    /// Caller asked for a suite name that isn't registered.
    #[error("unknown benchmark suite: '{name}' (expected one of: {valid})")]
    UnknownSuite { name: String, valid: String },
    /// Internal registry invariant violation — `list_suites()` returned an
    /// unexpected count. Triggers in the contract test if a new suite lands
    /// without updating `EXPECTED_SUITE_COUNT`.
    #[error("registry invariant violation: have {actual} suites, expected {expected}")]
    InvariantViolation { actual: usize, expected: usize },
}

// ---------------------------------------------------------------------------
// Trait surface
// ---------------------------------------------------------------------------

/// Closed-trait surface for an external benchmark suite. The Python runner
/// at .elai_cc/benches/external/<name>.py is the live-eval body; this trait
/// is the Rust-side dispatcher boundary that emits SSB rows + applies the
/// paired-bootstrap significance verdict.
///
/// `run` is intentionally NOT async — bench dispatch in this crate happens
/// via std::process::Command shelling out to the Python runner; the async
/// runtime is owned by the CLI wrapper (eval_cmd.rs).
pub trait BenchmarkSuite: Send + Sync {
    /// Canonical snake-case suite name. Stable wire identifier.
    fn name(&self) -> &str;

    /// Suite version string (semver or arXiv-id-anchor when appropriate).
    fn version(&self) -> &str;

    /// Public SOTA reference for telemetry; None if not yet calibrated.
    fn sota_reference(&self) -> Option<f64>;

    /// Local-runtime floor: minimum acceptable score on local sidecar. Used
    /// by the CLI to refuse promotion when below floor.
    fn local_floor(&self) -> f64;

    /// Cloud-runtime floor: minimum acceptable score on cloud route.
    fn cloud_floor(&self) -> f64;

    /// Significance verdict given two prior results (a = baseline, b = candidate).
    /// Default impl calls into [`paired_bootstrap_significance_default`].
    fn paired_bootstrap_significance(
        &self,
        a: &BenchmarkResult,
        b: &BenchmarkResult,
    ) -> SignificanceVerdict {
        paired_bootstrap_significance_default(a, b)
    }
}

// ---------------------------------------------------------------------------
// Default significance dispatch
// ---------------------------------------------------------------------------

/// Default paired-bootstrap significance verdict. Reusable across all
/// suites (each suite gets a default impl on the trait).
///
/// Logic:
///   1. If either side reports `low_power`, return `LowPowerNoConclusion`.
///   2. Compute a paired-bootstrap p on a 2-element vector `[score_a]` vs
///      `[score_b]`. This is a coarse approximation -- the production path
///      runs the bootstrap inside the Python runner on per-item scores and
///      passes the resulting `p_value` through `b.p_value`. When `b.p_value`
///      is `Some`, prefer that value over the recomputed approximation.
///   3. p < `DEFAULT_P_TARGET` AND `b.score < a.score` => Regression.
///   4. p < `DEFAULT_P_TARGET` AND `b.score > a.score` => Improvement.
///   5. Otherwise => NoChange.
#[must_use]
pub fn paired_bootstrap_significance_default(
    a: &BenchmarkResult,
    b: &BenchmarkResult,
) -> SignificanceVerdict {
    if a.low_power || b.low_power {
        return SignificanceVerdict::LowPowerNoConclusion;
    }
    let p = b.p_value.unwrap_or_else(|| {
        // Fallback: 2-element paired bootstrap on the two aggregate scores.
        // Coarse, but stable + deterministic — the Python runner ships a
        // proper per-item bootstrap in production.
        let seed = default_seed_for_suite(&b.suite_name, (b.ts_ms / 1000 / 3600) as u64);
        paired_bootstrap_p(&[a.score], &[b.score], DEFAULT_BOOTSTRAP_ITERS, seed)
    });

    if p < DEFAULT_P_TARGET {
        if b.score < a.score {
            SignificanceVerdict::Regression
        } else if b.score > a.score {
            SignificanceVerdict::Improvement
        } else {
            SignificanceVerdict::NoChange
        }
    } else {
        SignificanceVerdict::NoChange
    }
}

// ---------------------------------------------------------------------------
// Per-suite impls (11 entries; one per BenchmarkSuite registry slot)
// ---------------------------------------------------------------------------

macro_rules! external_bench {
    (
        $struct_name:ident,
        $wire_name:expr,
        $version:expr,
        $sota:expr,
        $local_floor:expr,
        $cloud_floor:expr
    ) => {
        /// Pattern-2 suite stub. Live-eval body in
        /// `.elai_cc/benches/external/$wire_name.py` (RUNPOD-PENDING).
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $struct_name;
        impl BenchmarkSuite for $struct_name {
            fn name(&self) -> &str {
                $wire_name
            }
            fn version(&self) -> &str {
                $version
            }
            fn sota_reference(&self) -> Option<f64> {
                $sota
            }
            fn local_floor(&self) -> f64 {
                $local_floor
            }
            fn cloud_floor(&self) -> f64 {
                $cloud_floor
            }
        }
    };
}

// 1. SWE-Bench Pro (primary). arXiv:2509.16941 §1 — top models ~23% on the
//    public set vs 70%+ on SWE-bench Verified. Local floor very low (long-
//    horizon multi-file patches require cloud).
external_bench!(
    SweBenchPro,
    "swe-bench-pro",
    "2509.16941",
    Some(0.23),
    0.05,
    0.20
);
// 2. SWE-bench-Live. arXiv:2505.23419 — 1,319 monthly-refresh tasks.
external_bench!(
    SweBenchLive,
    "swe-bench-live",
    "2505.23419",
    Some(0.25),
    0.05,
    0.20
);
// 3. LiveCodeBench v6. arXiv:2403.07974. Local-7B floor 30% / cloud floor 45%
//    per plan §Task 3.
external_bench!(
    LiveCodeBenchV6,
    "livecodebench-v6",
    "2403.07974-v6",
    Some(0.55),
    0.30,
    0.45
);
// 4. tau2-bench. arXiv:2506.07982 / sierra-research/tau2-bench. Retail
//    threshold 0.70 cloud / 0.55 local (telecom uses 0.85 cloud / 0.55 local
//    per plan §Task 4 — encoded on the cloud_floor here as the retail value;
//    Python runner enforces the telecom-specific threshold from the policy
//    rubric).
external_bench!(
    Tau2Bench,
    "tau2-bench",
    "2506.07982",
    Some(0.862),
    0.55,
    0.70
);
// 5. BFCL v3. gorilla.cs.berkeley.edu — GLM 4.5 SOTA 76.7, Qwen3-32B 75.7.
external_bench!(
    BfclV3,
    "bfcl-v3",
    "gorilla-icml-2025",
    Some(0.767),
    0.50,
    0.70
);
// 6. GAIA L1+L2. arXiv:2311.12983 — Sonnet 4.5 SOTA 74.6%.
external_bench!(
    GaiaL1L2,
    "gaia-l1-l2",
    "2311.12983",
    Some(0.746),
    0.20,
    0.50
);
// 7. ClawsBench harness-vs-model split. arXiv:2604.05172 — scaffold effect
//    +39-63pp vs model effect 10pp.
external_bench!(
    ClawsBenchSplit,
    "clawsbench-split",
    "2604.05172",
    Some(0.55),
    0.30,
    0.55
);
// 8. RefineBench. arXiv:2511.22173 — Gemini 2.5 Pro 31.3% / GPT-5 29.1% baseline.
external_bench!(
    RefineBench,
    "refinebench",
    "2511.22173",
    Some(0.313),
    0.10,
    0.25
);
// 9. Sharma sycophancy probes. arXiv:2310.13548 — flip rate < 15% target.
//    "score" is 1 - flip_rate so higher is better.
external_bench!(
    SycophancyProbes,
    "sycophancy-probes",
    "2310.13548",
    Some(0.85),
    0.70,
    0.85
);
// 10. GSM8K-Platinum + MATH-500. gradientscience.org/gsm8k-platinum +
//     madrylab/gsm8k-platinum HF. Threshold local 90% (GSM8K-Platinum) /
//     50% (MATH-500). Score reported as macro-average of the two subsets.
external_bench!(
    MathFloors,
    "math-floors",
    "platinum+math500",
    Some(0.90),
    0.70,
    0.90
);
// 11. DeepPlanning. arXiv:2601.18137 — GPT-5.2-high 44.6% multi-day ceiling.
external_bench!(
    DeepPlanning,
    "deep-planning",
    "2601.18137",
    Some(0.446),
    0.15,
    0.40
);

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Return the canonical list of all 11 external benchmark suites.
///
/// Order is the plan-task order. The contract test asserts
/// `list_suites().len() == EXPECTED_SUITE_COUNT`.
#[must_use]
pub fn list_suites() -> Vec<Box<dyn BenchmarkSuite>> {
    vec![
        Box::new(SweBenchPro),
        Box::new(SweBenchLive),
        Box::new(LiveCodeBenchV6),
        Box::new(Tau2Bench),
        Box::new(BfclV3),
        Box::new(GaiaL1L2),
        Box::new(ClawsBenchSplit),
        Box::new(RefineBench),
        Box::new(SycophancyProbes),
        Box::new(MathFloors),
        Box::new(DeepPlanning),
    ]
}

/// Look up a suite by canonical wire name. Returns None for unknown names
/// (the CLI surfaces this as `BenchRegistryError::UnknownSuite`).
#[must_use]
pub fn get_suite(name: &str) -> Option<Box<dyn BenchmarkSuite>> {
    list_suites().into_iter().find(|s| s.name() == name)
}

/// Canonical list of suite wire names, sorted for stable diff output.
/// Used by the `--list` CLI flag.
#[must_use]
pub fn suite_names_sorted() -> Vec<String> {
    let mut names: Vec<String> = list_suites().iter().map(|s| s.name().to_string()).collect();
    names.sort();
    names
}

/// Build the suite metadata map keyed by wire name. Used by the CLI
/// `--list --json` output + dashboard surface.
#[must_use]
pub fn suite_metadata() -> BTreeMap<String, SuiteMetadata> {
    let mut m = BTreeMap::new();
    for s in list_suites() {
        m.insert(
            s.name().to_string(),
            SuiteMetadata {
                name: s.name().to_string(),
                version: s.version().to_string(),
                sota_reference: s.sota_reference(),
                local_floor: s.local_floor(),
                cloud_floor: s.cloud_floor(),
            },
        );
    }
    m
}

/// Per-suite metadata surface — JSON-renderable summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuiteMetadata {
    pub name: String,
    pub version: String,
    pub sota_reference: Option<f64>,
    pub local_floor: f64,
    pub cloud_floor: f64,
}

// ---------------------------------------------------------------------------
// Power-calibration helper used by per-suite runners.
// ---------------------------------------------------------------------------

/// Decide whether a candidate result has sufficient `n` to call the verdict.
/// Wrapper around `bench_significance::is_low_power` with the default-target
/// values baked in.
#[must_use]
pub fn check_power_calibration(observed_n: usize, observed_variance: f64) -> PowerVerdict {
    let required = required_n_for_p_threshold(observed_variance, DEFAULT_TARGET_DELTA, DEFAULT_P_TARGET);
    PowerVerdict {
        observed_n,
        required_n: required,
        low_power: is_low_power(observed_n, required),
    }
}

/// Power-calibration result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PowerVerdict {
    pub observed_n: usize,
    pub required_n: usize,
    pub low_power: bool,
}

// ---------------------------------------------------------------------------
// Promotion-gate consumer wrapper (E-7 composition; DECISION-2)
// ---------------------------------------------------------------------------

/// Bridge from a BenchPhase::Completed bench result to the E-7 two-gate
/// promotion verdict. Wire-dark by default — only compiled when the
/// `darwinian-evolve` feature is set (matches the gating on
/// `promotion_gate.rs` per DECISION-4). Callers (bench runners, CLI eval
/// path) invoke this AFTER a Completed phase row has been emitted; the
/// returned PromotionDecision then drives the archive-admit / refusal
/// emission downstream.
///
/// `candidate_fitness` is the suite-reported score in `[0.0, 1.0]`; the
/// descriptor + archive + cfg flow through unchanged from the E-7
/// substrate (see promotion_gate.rs:103). Live RUNPOD calls (per-axis
/// capacity breakdown per arXiv:2510.04399 §Theorem 11) remain
/// RUNPOD-PENDING — this scaffold uses the existing cell-count proxy
/// already baked into `two_gate_promote`.
#[cfg(feature = "darwinian-evolve")]
#[must_use]
pub fn promote_after_bench_completed(
    candidate_fitness: f64,
    descriptor: &crate::template_archive::BehaviourDescriptor,
    archive: &crate::template_archive::MapElitesArchive,
    cfg: &crate::promotion_gate::PromotionConfig,
) -> Result<crate::promotion_gate::PromotionDecision, crate::promotion_gate::GateError> {
    let candidate = crate::template_archive::ArchiveEntry {
        template_blake3: [0u8; 32],
        fitness: candidate_fitness,
    };
    crate::promotion_gate::two_gate_promote(&candidate, descriptor, archive, cfg)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_size_matches_expected() {
        assert_eq!(list_suites().len(), EXPECTED_SUITE_COUNT);
    }

    #[test]
    fn registry_names_unique() {
        let names = suite_names_sorted();
        let mut sorted = names.clone();
        sorted.dedup();
        assert_eq!(names.len(), sorted.len(), "duplicate suite names in registry");
    }

    #[test]
    fn get_suite_returns_known() {
        let s = get_suite("swe-bench-pro");
        assert!(s.is_some());
        assert_eq!(s.unwrap().name(), "swe-bench-pro");
    }

    #[test]
    fn get_suite_returns_none_for_unknown() {
        assert!(get_suite("not-a-real-suite").is_none());
    }

    #[test]
    fn metadata_carries_floors_and_versions() {
        let m = suite_metadata();
        let pro = m.get("swe-bench-pro").expect("registered");
        assert_eq!(pro.version, "2509.16941");
        assert_eq!(pro.cloud_floor, 0.20);
    }

    fn make_result(name: &str, score: f64, n: usize, p: Option<f64>, low_power: bool) -> BenchmarkResult {
        BenchmarkResult {
            suite_name: name.into(),
            score,
            n,
            sota_reference: Some(0.5),
            p_value: p,
            low_power,
            model_effect_pp: None,
            harness_effect_pp: None,
            coverage_gap_pp: None,
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn paired_bootstrap_low_power_returns_no_conclusion() {
        let a = make_result("swe-bench-pro", 0.50, 30, Some(0.04), false);
        let b = make_result("swe-bench-pro", 0.40, 30, Some(0.04), true);
        let suite = SweBenchPro;
        assert_eq!(
            suite.paired_bootstrap_significance(&a, &b),
            SignificanceVerdict::LowPowerNoConclusion
        );
    }

    #[test]
    fn paired_bootstrap_regression_flagged() {
        let a = make_result("swe-bench-pro", 0.60, 200, Some(0.10), false);
        let b = make_result("swe-bench-pro", 0.40, 200, Some(0.01), false);
        let suite = SweBenchPro;
        assert_eq!(
            suite.paired_bootstrap_significance(&a, &b),
            SignificanceVerdict::Regression
        );
    }

    #[test]
    fn paired_bootstrap_improvement_flagged() {
        let a = make_result("swe-bench-pro", 0.40, 200, Some(0.10), false);
        let b = make_result("swe-bench-pro", 0.60, 200, Some(0.01), false);
        let suite = SweBenchPro;
        assert_eq!(
            suite.paired_bootstrap_significance(&a, &b),
            SignificanceVerdict::Improvement
        );
    }

    #[test]
    fn paired_bootstrap_no_change_when_high_p() {
        let a = make_result("swe-bench-pro", 0.55, 200, Some(0.10), false);
        let b = make_result("swe-bench-pro", 0.56, 200, Some(0.40), false);
        let suite = SweBenchPro;
        assert_eq!(
            suite.paired_bootstrap_significance(&a, &b),
            SignificanceVerdict::NoChange
        );
    }

    #[test]
    fn power_check_reports_low_power_on_small_n() {
        // Variance 0.01, n=10 should be < required_n (~196).
        let v = check_power_calibration(10, 0.01);
        assert!(v.low_power);
        assert!(v.required_n > v.observed_n);
    }

    #[test]
    fn power_check_passes_on_large_n() {
        // Variance 0.01, n=1000 should be >> required_n (~196).
        let v = check_power_calibration(1000, 0.01);
        assert!(!v.low_power);
    }
}
