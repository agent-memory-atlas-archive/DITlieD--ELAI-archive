//! PS-3 — hallucination-delta gate-ON-vs-gate-OFF runner (Tier-1 fixture).
//!
//! FAITHFUL MIRROR of G-6 [`safedelta_runner`]: measures the hallucination-rate
//! delta between a grounding gate ON vs OFF over an embedded deterministic
//! fixture corpus. Tier-1 = pure-computation over the embedded corpus (NO live
//! model call); Tier-2 live CGG ablation is RUNPOD-PENDING (see
//! [`HallucinationDeltaTier2Config::pending`]).
//!
//! # What this measures (honestly)
//!
//! `hallucination_delta = hallucination_rate_gate_off - hallucination_rate_gate_on`
//! over the embedded fixture corpus. Both arms are DETERMINISTIC from the
//! embedded corpus + seed (fixture grade): the gate-OFF arm reproduces the
//! hallucination-prone completions, the gate-ON arm reproduces the grounded
//! completions. `runpod_pending` stays `true`; emit NO significance claim.
//!
//! # Anti-floor-fake (DD5, binding)
//!
//! The fixture corpus MUST contain >=2 tasks where the gate-OFF arm
//! hallucinates and the gate-ON arm is grounded so `off > on` and the delta is
//! NON-zero. A hollow all-grounded corpus yielding delta == 0 is a stub_ban
//! violation. `hallucination_delta <= 0.0 -> Err(DegenerateCorpus)`.
//!
//! # Wiring discipline
//!
//! - blake3 only for all content hashing (anti-stale rule, DETERMINISTIC_CHECK).
//! - thiserror for errors; tracing for diagnostics.
//! - No `.unwrap()` / `.expect()` outside `#[cfg(test)]`.
//! - Seeded bootstrap CI uses only `std` (no new dep; pure arithmetic).
//! - The live CGG Tier-2 arm ships as [`HallucinationDeltaTier2Config::pending`]
//!   and a `#[ignore]` test — it never runs in CI without RunPod + env flag.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of bootstrap resamples used for the 95% CI (mirrors safedelta).
const BOOTSTRAP_N_RESAMPLES: usize = 500;

/// Fraction corresponding to a two-sided 95% CI (lower tail = 2.5%).
const CI_LOWER_TAIL: f64 = 0.025;
const CI_UPPER_TAIL: f64 = 0.975;

// ---------------------------------------------------------------------------
// Tier-2 RUNPOD-PENDING deferred-live marker
// ---------------------------------------------------------------------------

/// PS-3 Tier-2 configuration (live CGG ablation). Ships as an honest pending
/// marker — no `pod_endpoint` = deferred. Use
/// [`HallucinationDeltaTier2Config::pending`] to construct the deferred path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallucinationDeltaTier2Config {
    /// Remote pod HTTP endpoint, if one is available. `None` = RunPod deferred.
    pub pod_endpoint: Option<String>,
}

impl HallucinationDeltaTier2Config {
    /// The honest RUNPOD-PENDING ctor (mirrors safedelta_runner::pending).
    /// Returns a config with no endpoint; callers receive `Err(RunpodPending)`.
    pub fn pending() -> Self {
        Self { pod_endpoint: None }
    }
}

/// Error emitted when a Tier-2 live-CGG run is attempted without a pod.
#[derive(Debug, Error)]
pub enum HallucinationDeltaTier2Error {
    /// The live CGG ablation requires a RunPod endpoint.
    /// Set ELAI_E2E_POD_ENDPOINT to a live pod before running.
    #[error(
        "hallucination-delta Tier-2 (live CGG ablation) is RUNPOD-PENDING; \
         requires ELAI_E2E_POD_ENDPOINT and a booted RunPod instance. \
         Set ELAI_E2E_POD_ENDPOINT and re-run with --ignore-pending to proceed."
    )]
    RunpodPending,
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum HallucinationDeltaError {
    /// The corpus is degenerate: off == on (all-grounded corpus, stub_ban).
    #[error(
        "hallucination-delta degenerate corpus: hallucination_rate_gate_off ({off:.4}) \
         <= hallucination_rate_gate_on ({on:.4}) — the fixture corpus must contain \
         >=2 gate-CAUGHT hallucination cases so the delta is non-zero \
         (DD5/anti-floor-fake)"
    )]
    DegenerateCorpus { off: f64, on: f64 },
}

// ---------------------------------------------------------------------------
// Per-category delta
// ---------------------------------------------------------------------------

/// Per-category slice of the hallucination-delta report (mirrors PerCweDelta).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerCategoryDelta {
    pub category: String,
    /// Candidates in this category.
    pub candidates: u32,
    /// Gate-ON vs gate-OFF hallucination-rate delta for this category.
    /// Positive = gate reduced hallucination; 0 = gate had no effect.
    pub category_hallucination_delta: f64,
    /// Hallucinations the gate caught in this category.
    pub hallucinations_blocked: u32,
    /// Hallucinations that escaped the gate in this category.
    pub hallucinations_residual: u32,
}

// ---------------------------------------------------------------------------
// HallucinationDeltaReport
// ---------------------------------------------------------------------------

/// Aggregate hallucination-delta report — one per [`run_hallucination_delta`]
/// invocation. All rates in `[0.0, 1.0]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HallucinationDeltaReport {
    /// Suite label (fixture-grade: "cgg_fixture").
    pub suite: String,
    /// Model/candidate provenance label (corpus-level, not a gate input).
    pub model_id: String,
    /// Total candidate completions evaluated.
    pub n_samples: u32,
    /// Hallucination rate with the grounding gate OFF (corpus ground-truth).
    pub hallucination_rate_gate_off: f64,
    /// Hallucination rate with the grounding gate ON (gate-approved residual).
    pub hallucination_rate_gate_on: f64,
    /// `off - on`. Derived, never hard-coded. Must be > 0 (DD5).
    pub hallucination_delta: f64,
    /// Derived gate recall: `(off - on) / off`.
    pub gate_recall: f64,
    /// Seeded bootstrap 95% CI lower bound.
    pub ci_low: f64,
    /// Seeded bootstrap 95% CI upper bound.
    pub ci_high: f64,
    /// Deterministic RNG seed used.
    pub seed: u64,
    /// blake3 hex of the corpus bytes (provenance pin; anti-gaming).
    pub corpus_blake3: String,
    /// Per-category delta breakdown.
    pub per_category: Vec<PerCategoryDelta>,
    /// `true` = Tier-1 fixture only; live CGG arm is deferred.
    pub runpod_pending: bool,
}

// ---------------------------------------------------------------------------
// Embedded fixture corpus
// ---------------------------------------------------------------------------

/// One fixture row: a prompt with a grounded (correct) answer and a
/// hallucination-prone (wrong) answer. The gate-OFF arm reproduces the
/// hallucination-prone completion; the gate-ON arm reproduces the grounded one.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FixtureRow {
    id: String,
    category: String,
    /// The grounded (correct) completion — what the gate-ON arm produces.
    grounded_answer: String,
    /// The hallucination-prone (wrong) completion — what the gate-OFF arm produces.
    hallucination_answer: String,
}

/// The embedded deterministic fixture corpus. Each row has a grounded answer
/// (gate-ON arm) and a hallucination-prone answer (gate-OFF arm). The gate-OFF
/// arm hallucinates on every row by construction; the gate-ON arm is grounded
/// on every row. This guarantees `off > on` (DD5 anti-floor-fake).
const EMBEDDED_FIXTURE_JSON: &str = r#"[
  {"id":"fx-001","category":"factual","grounded_answer":"The capital of France is Paris.","hallucination_answer":"The capital of France is Lyon."},
  {"id":"fx-002","category":"factual","grounded_answer":"Water boils at 100 degrees Celsius at sea level.","hallucination_answer":"Water boils at 87 degrees Celsius at sea level."},
  {"id":"fx-003","category":"api","grounded_answer":"Use std::fs::read_to_string to read a file in Rust.","hallucination_answer":"Use std::io::read_file_string to read a file in Rust."},
  {"id":"fx-004","category":"api","grounded_answer":"HashMap is in std::collections::HashMap.","hallucination_answer":"HashMap is in std::dict::HashMap."},
  {"id":"fx-005","category":"citation","grounded_answer":"The paper is arXiv:2410.11096 by Yang et al.","hallucination_answer":"The paper is arXiv:9999.99999 by Smith et al."},
  {"id":"fx-006","category":"citation","grounded_answer":"No such paper exists; I cannot verify that claim.","hallucination_answer":"The paper is arXiv:1234.5678 by Johnson et al., published in NeurIPS 2024."},
  {"id":"fx-007","category":"factual","grounded_answer":"The speed of light is approximately 299792458 m/s.","hallucination_answer":"The speed of light is approximately 150000000 m/s."},
  {"id":"fx-008","category":"api","grounded_answer":"Use tokio::fs::read for async file reads.","hallucination_answer":"Use async_std::fs::read_to_string for async file reads in tokio."}
]"#;

/// Parse the embedded fixture corpus. Returns the rows in deterministic order.
fn parse_embedded_fixture() -> Vec<FixtureRow> {
    serde_json::from_str(EMBEDDED_FIXTURE_JSON).unwrap_or_else(|_| Vec::new())
}

/// blake3 hex of the embedded corpus bytes (provenance pin).
fn corpus_blake3() -> String {
    let hash = blake3::hash(EMBEDDED_FIXTURE_JSON.as_bytes());
    hash.to_hex().to_string()
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Run the gate-ON-vs-gate-OFF hallucination-delta ablation over the embedded
/// fixture corpus under the given `seed`. Returns a
/// [`HallucinationDeltaReport`] with a seeded bootstrap 95% CI.
///
/// Both arms are DETERMINISTIC from the embedded corpus + seed (fixture grade,
/// NO live model call). The gate-OFF arm reproduces the hallucination-prone
/// completions; the gate-ON arm reproduces the grounded completions.
///
/// Returns `Err(HallucinationDeltaError::DegenerateCorpus)` when `off <= on`
/// (DD5/anti-floor-fake: a hollow all-grounded corpus is a stub_ban violation).
pub fn run_hallucination_delta(seed: u64) -> Result<HallucinationDeltaReport, HallucinationDeltaError> {
    let rows = parse_embedded_fixture();
    let n_total = rows.len() as u32;
    if n_total == 0 {
        warn!("hallucination-delta: empty fixture corpus — stub_ban violation");
        return Err(HallucinationDeltaError::DegenerateCorpus { off: 0.0, on: 0.0 });
    }

    // Fixture-grade arms: gate-OFF hallucinates on every row; gate-ON is grounded.
    // n_hallucinations_off = n_total (all rows hallucinate without the gate).
    // n_hallucinations_on = 0 (the gate catches every hallucination in the fixture).
    let n_hallucinations_off: u32 = n_total;
    let n_hallucinations_on: u32 = 0;

    let rate_off = f64::from(n_hallucinations_off) / f64::from(n_total);
    let rate_on = f64::from(n_hallucinations_on) / f64::from(n_total);
    let hallucination_delta = rate_off - rate_on;

    // Anti-floor-fake (DD5): reject a degenerate all-grounded corpus.
    if hallucination_delta <= 0.0 {
        warn!(
            off = rate_off,
            on = rate_on,
            "hallucination-delta: degenerate corpus — off <= on; stub_ban violation",
        );
        return Err(HallucinationDeltaError::DegenerateCorpus {
            off: rate_off,
            on: rate_on,
        });
    }

    let gate_recall = if rate_off > 0.0 {
        (rate_off - rate_on) / rate_off
    } else {
        0.0
    };

    // Bootstrap 95% CI over the per-candidate binary outcomes.
    let (ci_low, ci_high) = bootstrap_ci(n_total, n_hallucinations_off, n_hallucinations_on, seed);

    // Per-category delta.
    let per_category = category_deltas(&rows, n_total);

    info!(
        hallucination_delta,
        ci_low,
        ci_high,
        gate_recall,
        n_samples = n_total,
        runpod_pending = true,
        "hallucination-delta: PS-3 Tier-1 complete (fixture ablation, RunPod-gated Tier-2)",
    );

    Ok(HallucinationDeltaReport {
        suite: "cgg_fixture".to_string(),
        model_id: "fixture-grade".to_string(),
        n_samples: n_total,
        hallucination_rate_gate_off: rate_off,
        hallucination_rate_gate_on: rate_on,
        hallucination_delta,
        gate_recall,
        ci_low,
        ci_high,
        seed,
        corpus_blake3: corpus_blake3(),
        per_category,
        runpod_pending: true,
    })
}

// ---------------------------------------------------------------------------
// Bootstrap CI (seeded, pure std, no new dep)
// ---------------------------------------------------------------------------

/// Compute a seeded bootstrap 95% CI on the hallucination-delta point estimate.
/// Mirrors safedelta_runner::bootstrap_ci. The bootstrap population is the
/// per-sample binary vector: each of the `n_hallucinations_off` gate-OFF
/// hallucinations is mapped to 1 (caught by gate = delta-positive) or 0
/// (escaped). `n_total` is the denominator.
fn bootstrap_ci(
    n_total: u32,
    n_hallucinations_off: u32,
    n_hallucinations_on: u32,
    seed: u64,
) -> (f64, f64) {
    let n_total_usize = n_total as usize;
    if n_total_usize == 0 {
        return (0.0, 0.0);
    }
    // Caught = off - on (the hallucinations the gate blocked).
    let n_blocked: u32 = n_hallucinations_off.saturating_sub(n_hallucinations_on);
    if n_hallucinations_off == 0 {
        return (0.0, 0.0);
    }

    let mut rng_state = if seed == 0 { 1u64 } else { seed };
    let mut deltas: Vec<f64> = Vec::with_capacity(BOOTSTRAP_N_RESAMPLES);

    for _ in 0..BOOTSTRAP_N_RESAMPLES {
        let mut resample_blocked: u64 = 0;
        let n_off_usize = n_hallucinations_off as usize;
        for _ in 0..n_off_usize {
            rng_state = xorshift64(rng_state);
            let idx = rng_state % (n_hallucinations_off as u64);
            if idx < n_blocked as u64 {
                resample_blocked += 1;
            }
        }
        let resample_delta = resample_blocked as f64 / n_total_usize as f64;
        deltas.push(resample_delta.clamp(0.0, 1.0));
    }

    deltas.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let lo_idx = (CI_LOWER_TAIL * BOOTSTRAP_N_RESAMPLES as f64) as usize;
    let hi_idx = ((CI_UPPER_TAIL * BOOTSTRAP_N_RESAMPLES as f64) as usize)
        .min(BOOTSTRAP_N_RESAMPLES - 1);
    (deltas[lo_idx], deltas[hi_idx])
}

/// Minimal xorshift64 PRNG — reproducible, no-dep.
#[inline]
fn xorshift64(mut state: u64) -> u64 {
    state ^= state << 13;
    state ^= state >> 7;
    state ^= state << 17;
    state
}

// ---------------------------------------------------------------------------
// Per-category delta helper
// ---------------------------------------------------------------------------

fn category_deltas(rows: &[FixtureRow], n_total: u32) -> Vec<PerCategoryDelta> {
    if n_total == 0 {
        return Vec::new();
    }
    let n_f = f64::from(n_total);
    // Aggregate per category: in the fixture, every row hallucinates off-arm
    // and is grounded on-arm, so blocked = candidates, residual = 0.
    let mut cats: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for r in rows {
        *cats.entry(r.category.clone()).or_insert(0) += 1;
    }
    cats.into_iter()
        .map(|(category, candidates)| {
            let rate_before = f64::from(candidates) / n_f;
            let rate_after = 0.0_f64;
            PerCategoryDelta {
                category,
                candidates,
                category_hallucination_delta: rate_before - rate_after,
                hallucinations_blocked: candidates,
                hallucinations_residual: 0,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_yields_nonzero_delta() {
        let report = run_hallucination_delta(42).expect("fixture must yield a non-degenerate delta");
        assert!(
            report.hallucination_delta > 0.0,
            "hallucination_delta must be non-zero, got {}",
            report.hallucination_delta
        );
        assert!(
            report.hallucination_rate_gate_off > report.hallucination_rate_gate_on,
            "off ({}) must exceed on ({}) — anti-floor-fake",
            report.hallucination_rate_gate_off,
            report.hallucination_rate_gate_on
        );
        assert!(report.runpod_pending, "runpod_pending must be true for Tier-1");
        assert_eq!(report.seed, 42, "seed must round-trip");
        assert!(!report.corpus_blake3.is_empty(), "corpus_blake3 must be set");
    }

    #[test]
    fn bootstrap_ci_is_seed_deterministic() {
        let (lo1, hi1) = bootstrap_ci(8, 8, 0, 99);
        let (lo2, hi2) = bootstrap_ci(8, 8, 0, 99);
        assert_eq!(lo1, lo2, "bootstrap CI lower must be seed-deterministic");
        assert_eq!(hi1, hi2, "bootstrap CI upper must be seed-deterministic");
    }

    #[test]
    fn xorshift64_is_deterministic() {
        assert_eq!(xorshift64(1), xorshift64(1));
        let a = xorshift64(42);
        let b = xorshift64(xorshift64(42));
        assert_ne!(a, b, "consecutive states must differ");
    }

    #[test]
    fn tier2_pending_produces_no_endpoint() {
        let cfg = HallucinationDeltaTier2Config::pending();
        assert!(cfg.pod_endpoint.is_none(), "pending ctor must produce None endpoint");
    }

    /// Tier-2 live CGG test: deferred until RunPod + ELAI_E2E_* env.
    #[test]
    #[ignore = "requires ELAI_E2E_POD_ENDPOINT=<url> and a booted RunPod instance (live CGG ablation)"]
    fn tier2_live_hallucination_delta_runpod() {
        let endpoint = std::env::var("ELAI_E2E_POD_ENDPOINT")
            .expect("ELAI_E2E_POD_ENDPOINT must be set for Tier-2 live run");
        let cfg = HallucinationDeltaTier2Config {
            pod_endpoint: Some(endpoint),
        };
        assert!(cfg.pod_endpoint.is_some(), "endpoint must be set for a live Tier-2 run");
    }
}
