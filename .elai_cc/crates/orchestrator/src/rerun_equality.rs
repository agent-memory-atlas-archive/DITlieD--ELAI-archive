// cross-lang-skip: rust-py-sidecar
//! H-23 (SCHEMA_VERSION 228): same-seed re-run hash-equality METRIC_GATE.
//!
//! The reproducibility centerpiece (T-7). `ELAI_SEED` asserts INTENT (the run
//! was configured deterministically) but nothing RE-RUNS the experiment and
//! compares the DECLARED metric outputs. A hidden unseeded RNG (np/torch/random,
//! a DataLoader shuffle, a missing determinism env var) silently breaks
//! reproducibility while every existing gate stays green. This gate blake3-hashes
//! the DECLARED metric outputs of TWO same-seed re-runs and asserts the digests
//! are byte-identical: a clean seeded experiment hashes Equal twice; a hidden
//! unseeded RNG makes run #2's digest differ (`Divergent` -> Block), naming the
//! first divergent metric.
//!
//! # Composition (GD-1: REUSE, do NOT re-implement)
//!   - blake3 hashing convention: the per-metric digest hex follows the RRA
//!     16-hex prefix convention ([`crate::rra`]). blake3 ONLY (CLAUDE.md).
//!   - first-divergence reporting: the two per-metric digest sequences are
//!     wrapped as [`crate::determinism::ActionTypeSequence`] and compared via
//!     [`crate::determinism::DriftComparator::compare`], whose first-divergence
//!     loop names the first diverging metric for free. The gate does NOT
//!     re-implement that loop.
//!
//! # Tier boundary (cross-lang-skip rationale)
//! Tier-1 (THIS gate) consumes two metric-digest lists as PLAIN DATA — it runs no
//! experiment. WHO produces the two digest lists is the DEFERRED Tier-2 Python
//! experiment-runner (re-executes the real entrypoint under the recorded env +
//! `ELAI_SEED` and hashes its declared outputs). That sidecar is RunPod-pending
//! this run (cloud-only override); no stub is shipped. The
//! `// cross-lang-skip: rust-py-sidecar` header opts this file out of the
//! Rust<->Python pair drift check until the Tier-2 runner ships. The Tier-1
//! done-gate is exercisable WITHOUT a sidecar via the
//! `elai replay --verify --metric-digest` fixture path.
//!
//! # The three fail-OPEN hazards this gate is built to close
//!   A. SEED FAIL-OPEN. [`crate::determinism::DeterminismConfig::from_env`]
//!      silently defaults the seed to 0 when `ELAI_SEED` is unset. A determinism
//!      gate that inherited that path would "pass" a run whose seed was never
//!      declared. This gate reads `ELAI_SEED` EXPLICITLY and FAILS-CLOSED when it
//!      is absent/empty — "cannot guarantee determinism" => Block, never seed=0.
//!   B. NON-EMPTY PAYLOAD. `blake3::hash(b"")` is a fixed constant, so two EMPTY
//!      runs would hash Equal and fail-OPEN (a sandbagger emits trivial output
//!      and "passes" reproducibility). This gate asserts the declared-metric-
//!      output set is non-empty/non-trivial BEFORE an `Equal` can be a PASS.
//!   C. NO NEW ATTESTED FIELD. The two digests are the gate's INPUT (supplied by
//!      the re-run entrypoint), NOT a persisted [`crate::rra::RunArtifact`] field.
//!      No new RunArtifact field is added, so the canonical bundle tuple, the RRA
//!      format version, the G-8 attestation merkle, and the ts-rs RunArtifact
//!      bindings are untouched (no cross-stack drift).
//!
//! # FAIL-CLOSED (R1/R2/W-1 — "cannot prove reproducibility" => Block)
//! Conditions that short-circuit to a hard `Divergent` Block with an emitted SSB
//! row, NEVER a silent pass:
//!   1. the second run's declared-output digest list is missing (`None`) or empty
//!      (a single run cannot self-certify reproducibility — W-1/R1),
//!   2. the declared seed is absent (hazard A),
//!   3. either run's declared-output set is empty/trivial (hazard B),
//!   4. the two sets have a mismatched metric-name list (a missing/extra metric
//!      IS a divergence, not silently skipped),
//!   5. any per-metric digest differs across the two runs (a hidden unseeded RNG).
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::RerunHashEquality`] row on EVERY decision
//! (Equal / Divergent). Even an Equal row is evidence. The two digests are the
//! agent-un-authorable off-agent artifact (the re-run is orchestrator-driven).
//!
//! # SCOPE (TD-5, honest residue)
//! Same-env/same-seed equality ONLY. Cross-hardware GPU/FP bitwise equality is
//! physics (FMA/SIMD reassociation differs across SKUs); this gate is NEVER run
//! across hardware, and a cross-hardware delta is recorded, never asserted.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic numbers).
//!
//! References:
//!   - crates/orchestrator/src/lookahead_differential.rs (H-21 pure-gate shape).
//!   - crates/orchestrator/src/rra.rs (blake3 16-hex digest convention; REUSE).
//!   - crates/orchestrator/src/determinism.rs:364 (DriftComparator first-
//!     divergence — REUSED here over the two metric-digest sequences).
//!   - workflow/reasoning/H-23-decisions.md.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::determinism::{ActionTypeSequence, DriftComparator, DriftSink};
use crate::ssb::{RerunHashEqualityKind, RerunHashEqualityPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants
// ---------------------------------------------------------------------------

/// Canonical rule id for the re-run hash-equality gate (documentary parity with
/// the sibling gates' `*_RULE_ID`). The equality DECISION is pure (no rules.db
/// threshold); this id pins the two seeded sub-gates (seed-coverage AST +
/// env-pin SchemaRequired) authored under `crates/rules/seed/repro/`.
pub const RERUN_EQUALITY_RULE_ID: &str = "REPRO-RERUN-001";

/// Minimum number of declared metric outputs required before an `Equal` verdict
/// can be a PASS (hazard B: blake3 of an empty input is a fixed constant, so a
/// zero-metric run would fail-OPEN). A reproducibility claim over ZERO declared
/// metrics is vacuous; at least one declared metric must be hashed.
pub const MIN_DECLARED_METRICS: usize = 1;

/// SSB `action_type` wire tag for the off-agent artifact (matches
/// [`crate::ssb::SsbActionType::RerunHashEquality::as_sql_str`]).
const RERUN_EQUALITY_ACTION_TYPE: &str = "rerun_hash_equality";

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// One declared metric output of an experiment run: a metric name + the blake3
/// digest (16-hex prefix) of that metric's serialized output bytes.
///
/// The digest is computed over the DECLARED metric outputs ONLY (R4), NEVER over
/// stdout. The Tier-2 experiment-runner produces these by hashing each declared
/// output artifact; the Tier-1 fixture path supplies them directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricDigest {
    /// Declared metric name (must match across the two re-runs).
    pub name: String,
    /// blake3 (16-hex prefix) of this metric's declared output bytes.
    pub digest: String,
}

impl MetricDigest {
    /// Construct a `MetricDigest` by hashing raw declared-output bytes with the
    /// RRA 16-hex blake3 convention. blake3 ONLY (CLAUDE.md HARD RULE).
    #[must_use]
    pub fn from_bytes(name: impl Into<String>, output_bytes: &[u8]) -> Self {
        let full = blake3::hash(output_bytes).to_hex();
        let digest = full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect();
        MetricDigest {
            name: name.into(),
            digest,
        }
    }
}

/// One run's full set of declared-metric-output digests, in declaration order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunDigests {
    /// Per-metric digests in declaration order.
    pub metrics: Vec<MetricDigest>,
}

impl RunDigests {
    /// Construct from `(name, digest)` pairs (digests already computed).
    #[must_use]
    pub fn from_pairs(pairs: Vec<(String, String)>) -> Self {
        RunDigests {
            metrics: pairs
                .into_iter()
                .map(|(name, digest)| MetricDigest { name, digest })
                .collect(),
        }
    }

    /// Number of declared metric outputs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    /// True when there are no declared metric outputs (hazard B fail-closed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    /// The ordered metric NAMES (the column-set the two runs must agree on).
    fn names(&self) -> Vec<String> {
        self.metrics.iter().map(|m| m.name.clone()).collect()
    }

    /// The ordered per-metric DIGEST sequence, joined as
    /// `"<name>=<digest>"` so a same-name-different-digest divergence is caught
    /// by the reused [`DriftComparator`] first-divergence loop.
    fn digest_sequence(&self) -> Vec<String> {
        self.metrics
            .iter()
            .map(|m| format!("{}={}", m.name, m.digest))
            .collect()
    }

    /// True when at least one declared metric carries a non-empty digest
    /// (a trivial run that declared a metric but hashed empty bytes is rejected
    /// by [`is_trivial`]). This is the structural non-trivial check, hazard B.
    fn is_trivial(&self) -> bool {
        // No metrics at all, OR every metric digest is empty/whitespace.
        self.metrics.is_empty()
            || self.metrics.iter().all(|m| m.digest.trim().is_empty())
    }
}

/// Input bundle for [`verify_rerun_digest`].
#[derive(Debug, Clone)]
pub struct RerunInput {
    /// The experiment/strategy under test (repo-relative path or id).
    pub strategy_path: String,
    /// Run #1's declared-metric-output digests.
    pub run_a: RunDigests,
    /// Run #2's declared-metric-output digests. `None` => fail-closed
    /// `Divergent` Block (the second re-run never happened — W-1/R1).
    pub run_b: Option<RunDigests>,
    /// The DECLARED `ELAI_SEED` for this run. `None` => fail-closed `Divergent`
    /// Block (hazard A: a missing seed cannot guarantee determinism; the gate
    /// NEVER defaults it to 0).
    pub seed: Option<u64>,
}

/// Verdict kind returned by [`verify_rerun_digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RerunVerdictKind {
    /// The two same-seed re-runs produced byte-identical declared-metric-output
    /// digests over a non-empty/non-trivial output set: reproducible. PASS.
    Equal,
    /// The two digests differ, OR a fail-closed precondition failed (missing
    /// second run, missing seed, empty/trivial declared outputs, metric-name
    /// mismatch). BLOCK. Carries the first divergent metric (when one exists).
    Divergent,
}

/// Full verdict from [`verify_rerun_digest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RerunVerdict {
    /// Equal / Divergent.
    pub kind: RerunVerdictKind,
    /// Machine-readable reason code ("equal" | "digest_divergent" |
    /// "missing_second_run" | "missing_seed" | "empty_declared_outputs" |
    /// "metric_set_mismatch").
    pub reason: String,
    /// Name of the first divergent metric (empty on Equal / pre-comparison
    /// fail-closed).
    pub divergent_metric: String,
    /// Index of the first divergent per-metric digest (0 on Equal/fail-closed).
    pub first_divergence_index: u32,
    /// Number of declared metric outputs hashed (0 on the empty/trivial path).
    pub declared_metric_count: u32,
    /// run #1's combined declared-output digest (empty on a pre-compute path).
    pub digest_a: String,
    /// run #2's combined declared-output digest (empty on missing second run).
    pub digest_b: String,
    /// Free-text evidence string.
    pub evidence: String,
}

/// Errors from [`verify_rerun_digest`]. Note: a missing second run, a missing
/// seed, empty/trivial outputs, and metric-set mismatches are NOT errors — they
/// are fail-closed `Divergent` VERDICTS (each emits an SSB row). Only an SSB
/// write failure is an error here.
#[derive(Debug, Error)]
pub enum RerunError {
    /// Writing the off-agent RerunHashEquality SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: verify_rerun_digest
// ---------------------------------------------------------------------------

/// Verify that two same-seed re-runs hash equal over their DECLARED metric
/// outputs (Tier-1 reproducibility METRIC_GATE). Emits exactly one
/// [`SsbPayload::RerunHashEquality`] row on every decision (the off-agent
/// artifact, R4).
///
/// # Order of checks (FAIL-CLOSED first)
///   1. `seed.is_none()` => `Divergent` Block (reason `missing_seed`, hazard A):
///      the gate NEVER inherits the silent `DeterminismConfig::from_env` seed=0
///      default. A run whose seed is undeclared cannot be asserted deterministic.
///   2. `run_b.is_none()` => `Divergent` Block (reason `missing_second_run`,
///      W-1/R1): a single run cannot self-certify reproducibility.
///   3. EITHER run empty/trivial => `Divergent` Block (reason
///      `empty_declared_outputs`, hazard B): `blake3(b"")` is a fixed constant,
///      so two empty runs would hash Equal and fail-OPEN.
///   4. Metric-name set mismatch (different names/order/count) => `Divergent`
///      Block (reason `metric_set_mismatch`): a missing/extra declared metric IS
///      a divergence.
///   5. Per-metric digest diff (REUSING [`DriftComparator::compare`]'s first-
///      divergence over the two digest sequences): any metric whose digest
///      differs across the two runs => `Divergent` Block (reason
///      `digest_divergent`), naming the first divergent metric.
///   6. All digests equal over a non-empty/non-trivial set => `Equal` PASS.
pub async fn verify_rerun_digest(
    ssb: &Arc<Ssb>,
    input: RerunInput,
) -> Result<RerunVerdict, RerunError> {
    // -- Step 1: FAIL-CLOSED on a missing/undeclared seed (hazard A). -------
    // The gate reads the DECLARED seed off the input; it does NOT fall back to
    // DeterminismConfig::from_env (which silently defaults to 0). A None here is
    // "the seed was never declared" => cannot guarantee determinism => Block.
    let seed = match input.seed {
        Some(s) => s,
        None => {
            warn!(
                strategy = %input.strategy_path,
                "rerun_equality: ELAI_SEED undeclared -> Divergent Block \
                 (fail-closed, hazard A; the gate NEVER defaults the seed to 0)"
            );
            let verdict = divergent_verdict(
                "missing_seed",
                "The determinism seed (ELAI_SEED) is undeclared: same-seed \
                 reproducibility cannot be guaranteed. The gate fails closed \
                 rather than inheriting the silent seed=0 default (R2, hazard A).",
                String::new(),
                String::new(),
                0,
            );
            emit_ssb(ssb, build_payload(&input, &verdict, 0)).await?;
            return Ok(verdict);
        }
    };

    // -- Step 2: FAIL-CLOSED on a missing second run (W-1/R1). --------------
    let run_b = match &input.run_b {
        Some(r) => r,
        None => {
            warn!(
                strategy = %input.strategy_path,
                "rerun_equality: second re-run absent -> Divergent Block \
                 (fail-closed, W-1/R1; a single run cannot self-certify)"
            );
            let verdict = divergent_verdict(
                "missing_second_run",
                "The second same-seed re-run is absent: a single run cannot \
                 self-certify reproducibility. Fail-closed Block (W-1/R1 \
                 missing-evidence=FAIL).",
                String::new(),
                String::new(),
                0,
            );
            emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
            return Ok(verdict);
        }
    };
    let run_a = &input.run_a;

    // -- Step 3: FAIL-CLOSED on empty/trivial declared outputs (hazard B). --
    // blake3 of an empty input is a fixed constant; two empty runs would hash
    // Equal and fail-OPEN. Reject BEFORE any Equal verdict can be reached.
    if run_a.is_trivial() || run_b.is_trivial() || run_a.len() < MIN_DECLARED_METRICS {
        warn!(
            strategy = %input.strategy_path,
            run_a_metrics = run_a.len(),
            run_b_metrics = run_b.len(),
            "rerun_equality: empty/trivial declared outputs -> Divergent Block \
             (fail-closed, hazard B; blake3 of empty is a fixed constant)"
        );
        let verdict = divergent_verdict(
            "empty_declared_outputs",
            "At least one re-run declared no (or only empty) metric outputs. \
             blake3 of an empty input is a fixed constant, so two empty runs \
             would hash Equal and fail-OPEN. A reproducibility claim over zero \
             declared metrics is vacuous. Fail-closed Block (hazard B).",
            String::new(),
            String::new(),
            0,
        );
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 4: metric-NAME set guard (same names, same order, same count). -
    // A missing/extra declared metric IS a divergence, never silently skipped.
    if run_a.names() != run_b.names() {
        warn!(
            strategy = %input.strategy_path,
            "rerun_equality: declared-metric set mismatch -> Divergent Block"
        );
        let verdict = divergent_verdict(
            "metric_set_mismatch",
            "The two re-runs declared different metric names/order: a \
             missing/extra declared metric IS a divergence. Fail-closed Block.",
            String::new(),
            String::new(),
            0,
        );
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 5: per-metric digest diff (REUSE DriftComparator first-div). ---
    // Wrap each run's per-metric "<name>=<digest>" list as an ActionTypeSequence
    // so DriftComparator::compare names the FIRST diverging metric for free
    // (GD-1: reuse the shipped first-divergence, do not re-implement it). The
    // comparator emits its own DeterminismDrift row on divergence; we route that
    // through a closed (drop) channel because the gate's canonical off-agent
    // artifact is the RerunHashEquality row, not a DeterminismDrift row.
    let seq_a = ActionTypeSequence::new(run_a.digest_sequence());
    let seq_b = ActionTypeSequence::new(run_b.digest_sequence());
    let comparator = DriftComparator::new(seed, drift_sink_drop());
    let combined_a = combined_digest(run_a);
    let combined_b = combined_digest(run_b);

    if let Some(drift) = comparator.compare(&seq_a, &seq_b) {
        let idx = usize::try_from(drift.first_divergence_index).unwrap_or(usize::MAX);
        let divergent_metric = run_a
            .metrics
            .get(idx)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        warn!(
            strategy = %input.strategy_path,
            metric = %divergent_metric,
            index = drift.first_divergence_index,
            "rerun_equality: declared-metric digest changed across same-seed \
             re-runs -> Divergent Block (a hidden unseeded RNG)"
        );
        let verdict = divergent_verdict(
            "digest_divergent",
            &format!(
                "Declared metric '{}' (index {}) produced a different blake3 \
                 digest across two same-seed re-runs: a hidden unseeded RNG \
                 broke reproducibility. Fail-closed Block.",
                divergent_metric, drift.first_divergence_index
            ),
            combined_a,
            combined_b,
            drift.first_divergence_index,
        );
        let verdict = RerunVerdict {
            divergent_metric,
            ..verdict
        };
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 6: Equal PASS (all digests match over a non-trivial set). -----
    debug!(
        strategy = %input.strategy_path,
        metrics = run_a.len(),
        "rerun_equality: all declared-metric digests match across same-seed \
         re-runs -> Equal (reproducible)"
    );
    let verdict = RerunVerdict {
        kind: RerunVerdictKind::Equal,
        reason: "equal".to_string(),
        divergent_metric: String::new(),
        first_divergence_index: 0,
        declared_metric_count: u32::try_from(run_a.len()).unwrap_or(u32::MAX),
        digest_a: combined_a,
        digest_b: combined_b,
        evidence: format!(
            "Equal: all {} declared metric outputs hashed byte-identically \
             across two same-seed (seed={}) re-runs. Reproducible (same-env \
             scope; cross-hardware FP not asserted, TD-5).",
            run_a.len(),
            seed
        ),
    };
    emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a fail-closed / divergent `Divergent` verdict.
fn divergent_verdict(
    reason: &str,
    evidence: &str,
    digest_a: String,
    digest_b: String,
    first_divergence_index: u32,
) -> RerunVerdict {
    RerunVerdict {
        kind: RerunVerdictKind::Divergent,
        reason: reason.to_string(),
        divergent_metric: String::new(),
        first_divergence_index,
        declared_metric_count: 0,
        digest_a,
        digest_b,
        evidence: evidence.to_string(),
    }
}

/// The combined declared-output digest for a run: blake3 (16-hex) over the joined
/// per-metric `"<name>=<digest>"` sequence. This is the run-level fingerprint
/// surfaced on the SSB row (mirrors the RRA root-over-bundle pattern). blake3
/// ONLY (CLAUDE.md HARD RULE).
fn combined_digest(run: &RunDigests) -> String {
    let joined = run.digest_sequence().join("\n");
    let full = blake3::hash(joined.as_bytes()).to_hex();
    full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect()
}

/// A drop sink for the reused [`DriftComparator`]: the comparator emits its own
/// DeterminismDrift row on divergence, but the canonical off-agent artifact for
/// THIS gate is the RerunHashEquality row. We route the comparator's emit through
/// a channel whose receiver is dropped (the send is a no-op warn, not a failure),
/// so the DriftComparator first-divergence logic is reused WITHOUT emitting a
/// duplicate DeterminismDrift row.
fn drift_sink_drop() -> DriftSink {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    // Drop the receiver immediately; the comparator's send becomes a no-op
    // (DriftSink::Channel warns-and-continues on a closed channel — it never
    // panics or fails the gate).
    drop(rx);
    DriftSink::Channel(tx)
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: RerunHashEqualityPayload,
) -> Result<(), RerunError> {
    ssb.write_payload(
        RERUN_EQUALITY_ACTION_TYPE,
        0,
        SsbPayload::RerunHashEquality(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| RerunError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`RerunHashEqualityPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    input: &RerunInput,
    verdict: &RerunVerdict,
    seed: u64,
) -> RerunHashEqualityPayload {
    let kind = match verdict.kind {
        RerunVerdictKind::Equal => RerunHashEqualityKind::Equal,
        RerunVerdictKind::Divergent => RerunHashEqualityKind::Divergent,
    };

    let declared_metric_count = if verdict.declared_metric_count > 0 {
        verdict.declared_metric_count
    } else {
        // On Equal the verdict carries the count; on fail-closed paths it is 0.
        u32::try_from(input.run_a.len()).unwrap_or(u32::MAX)
    };

    let corpus_key = format!(
        "{}:{}:{}:{}",
        input.strategy_path, verdict.reason, verdict.first_divergence_index, seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    RerunHashEqualityPayload {
        kind,
        strategy_path: input.strategy_path.clone(),
        digest_a: verdict.digest_a.clone(),
        digest_b: verdict.digest_b.clone(),
        first_divergence_index: verdict.first_divergence_index,
        divergent_metric: verdict.divergent_metric.clone(),
        declared_metric_count,
        verdict: verdict.reason.clone(),
        seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (digest equality + first-divergence wrap + the two guards)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;

    fn test_ssb() -> Arc<Ssb> {
        let dir = std::env::temp_dir().join(format!("h23-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h23-unit".to_string(),
            session_id: "h23-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        // A monotonic per-process counter mixed with the nanosecond clock so two
        // tests that init their Ssb in the same nanosecond still get distinct db
        // files (avoids a SQLite "database is locked" collision under the
        // parallel test runner).
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now() // entropy-allow: test-only tempdir db filename suffix, never byte-compared by replay
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        nanos ^ (COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E3779B97F4A7C15))
    }

    fn rd(pairs: Vec<(&str, &str)>) -> RunDigests {
        RunDigests::from_pairs(
            pairs
                .into_iter()
                .map(|(n, d)| (n.to_string(), d.to_string()))
                .collect(),
        )
    }

    fn input(run_a: RunDigests, run_b: Option<RunDigests>, seed: Option<u64>) -> RerunInput {
        RerunInput {
            strategy_path: "exp.py".to_string(),
            run_a,
            run_b,
            seed,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn identical_digests_pass_equal() {
        let ssb = test_ssb();
        let a = rd(vec![("sharpe", "aaaa"), ("maxdd", "bbbb")]);
        let b = rd(vec![("sharpe", "aaaa"), ("maxdd", "bbbb")]);
        let v = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Equal);
        assert_eq!(v.reason, "equal");
        assert_eq!(v.declared_metric_count, 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn divergent_digest_blocks_naming_first_metric() {
        let ssb = test_ssb();
        let a = rd(vec![("sharpe", "aaaa"), ("maxdd", "bbbb")]);
        // run #2's maxdd digest differs (a hidden unseeded RNG).
        let b = rd(vec![("sharpe", "aaaa"), ("maxdd", "cccc")]);
        let v = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "digest_divergent");
        assert_eq!(v.divergent_metric, "maxdd");
        assert_eq!(v.first_divergence_index, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_seed_is_fail_closed_block_hazard_a() {
        let ssb = test_ssb();
        let a = rd(vec![("sharpe", "aaaa")]);
        let b = rd(vec![("sharpe", "aaaa")]);
        // seed=None must NOT fall back to seed=0; it must Block.
        let v = verify_rerun_digest(&ssb, input(a, Some(b), None))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "missing_seed");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_outputs_are_fail_closed_block_hazard_b() {
        let ssb = test_ssb();
        // Two EMPTY runs would hash Equal (blake3 of empty is constant); Block.
        let a = RunDigests::default();
        let b = RunDigests::default();
        let v = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "empty_declared_outputs");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn trivial_empty_digest_is_fail_closed_block_hazard_b() {
        let ssb = test_ssb();
        // A declared metric whose digest is empty/whitespace is trivial.
        let a = rd(vec![("sharpe", "")]);
        let b = rd(vec![("sharpe", "")]);
        let v = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "empty_declared_outputs");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_second_run_is_fail_closed_block() {
        let ssb = test_ssb();
        let a = rd(vec![("sharpe", "aaaa")]);
        let v = verify_rerun_digest(&ssb, input(a, None, Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "missing_second_run");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn metric_set_mismatch_is_block() {
        let ssb = test_ssb();
        let a = rd(vec![("sharpe", "aaaa"), ("maxdd", "bbbb")]);
        let b = rd(vec![("sharpe", "aaaa")]);
        let v = verify_rerun_digest(&ssb, input(a, Some(b), Some(42)))
            .await
            .expect("verify");
        assert_eq!(v.kind, RerunVerdictKind::Divergent);
        assert_eq!(v.reason, "metric_set_mismatch");
    }

    #[test]
    fn from_bytes_uses_blake3_16hex_prefix() {
        let md = MetricDigest::from_bytes("sharpe", b"some declared output bytes");
        assert_eq!(md.name, "sharpe");
        assert_eq!(md.digest.len(), crate::rra::RRA_HEX_PREFIX_LEN);
        // Distinct inputs produce distinct digests; same input is stable.
        let md2 = MetricDigest::from_bytes("sharpe", b"some declared output bytes");
        assert_eq!(md.digest, md2.digest);
        let md3 = MetricDigest::from_bytes("sharpe", b"different bytes");
        assert_ne!(md.digest, md3.digest);
        // blake3 of empty is the fixed constant the non-empty guard exists for.
        let empty = MetricDigest::from_bytes("x", b"");
        assert_eq!(empty.digest.len(), crate::rra::RRA_HEX_PREFIX_LEN);
    }
}
