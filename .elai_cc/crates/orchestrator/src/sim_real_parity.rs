// cross-lang-skip: rust-py-sidecar
//! H-24 (SCHEMA_VERSION 229): sim-to-real parity gate (T-8).
//!
//! Nothing asserts that the research/backtest path is the SAME path that runs
//! live, nor that a seeded replay reproduces a frozen golden event log. A
//! backtest that takes an `if backtest:` fast-path (a different fill model, a
//! duplicate router) produces an edge that does not exist live -- the classic
//! sim-real divergence.
//!
//! H-24 is split across two Tier-1 surfaces + a deferred Tier-2:
//!   * PART A (SEEDED, mutable=0): a mode-divergent-branch AST ban SCOPED to the
//!     order/signal execution path + a single-entrypoint SchemaRequired manifest.
//!     These live as seed rows (crates/rules/seed/repro/sim_real_parity.toml +
//!     [`crate`]'s sibling rules-engine predicates), NOT in this file. The AST ban
//!     is SCOPED (LiveMode is legitimate for notifications/logging per LEAN's
//!     official doc; a blanket ban false-positive-blocks every conditional).
//!   * PART C (THIS file, PURE Rust): [`verify_replay_golden`] blake3-hashes a
//!     seeded replay event log and a FROZEN golden event log and asserts they are
//!     byte-IDENTICAL (EXACT equality -- NOT a tolerance band; blake3 either
//!     matches or it does not). Any event whose digest differs PROVES a sim-real
//!     divergence (`ReplayGoldenMismatch` -> Block), naming the first via the
//!     reused [`crate::determinism::DriftComparator`] first-divergence.
//!   * PART B (DEFERRED Tier-2, NOT built here): numeric live-vs-backtest
//!     reconciliation (Sharpe/return/maxDD gap) needs a live paper-trade feed
//!     (GD-5, cloud-only override). Structural parity (A/C) is
//!     NECESSARY-NOT-SUFFICIENT: it does NOT close the market-impact /
//!     partial-fill / latency gap (LEAN: market impact unmodeled, backtest assumes
//!     complete fills). See workflow/reasoning/H-24-decisions.md.
//!
//! # Composition (GD-1: REUSE, do NOT re-implement)
//!   - blake3 hashing convention: the per-event digest hex follows the RRA
//!     16-hex prefix convention ([`crate::rra`]). blake3 ONLY (CLAUDE.md).
//!   - first-divergence reporting: the two per-event digest sequences are wrapped
//!     as [`crate::determinism::ActionTypeSequence`] and compared via
//!     [`crate::determinism::DriftComparator::compare`], whose first-divergence
//!     loop names the first diverging event for free. The gate does NOT
//!     re-implement that loop.
//!   - the seeded replay artifact is PRODUCED by the shipped
//!     [`crate::ssb::replay_resume`] substrate in the CLI driver; this gate
//!     consumes the two event-digest lists as PLAIN DATA.
//!
//! # Tier boundary (cross-lang-skip rationale)
//! Tier-1 (THIS gate) consumes two event-digest lists as PLAIN DATA. The DEFERRED
//! Tier-2 numeric reconciliation needs a live/paper-trade Python feed (RunPod-
//! pending; no stub shipped). The `// cross-lang-skip: rust-py-sidecar` header
//! opts this file out of the Rust<->Python pair drift check until that runner
//! ships. The Tier-1 done-gate is exercisable WITHOUT a live feed via the
//! `elai replay --verify --golden <demo>` fixture path.
//!
//! # The fail-OPEN hazards this gate is built to close
//!   A. MISSING REPLAY. A single artifact cannot self-certify parity: if no replay
//!      was produced, the gate FAILS-CLOSED (`ReplayGoldenMismatch` Block, reason
//!      `missing_replay`), never a silent pass (W-1/R1).
//!   B. EMPTY/TRIVIAL GOLDEN. `blake3::hash(b"")` is a fixed constant, so two EMPTY
//!      event logs would hash Equal and fail-OPEN (a sandbagger emits a trivial
//!      golden and "passes" parity). This gate asserts the golden event set is
//!      non-empty/non-trivial BEFORE a `Parity` can be a PASS.
//!   C. EVENT-SET MISMATCH. A missing/extra event IS a divergence; the gate Blocks
//!      on a mismatched event-name/order set, never silently skips it.
//!
//! # FAIL-CLOSED (R1/R2/W-1 -- "cannot prove parity" => Block)
//! Conditions that short-circuit to a hard Block with an emitted SSB row, NEVER a
//! silent pass:
//!   1. the replay event-digest list is missing (`None`) or empty (a single
//!      artifact cannot self-certify -- W-1/R1, hazard A),
//!   2. the golden event-digest set is empty/trivial (hazard B),
//!   3. the two sets have a mismatched event-name/order list (hazard C),
//!   4. any per-event blake3 digest differs (a sim-real divergence).
//!
//! # SSB emission (off-agent artifact, R4)
//! Emits one [`crate::ssb::SsbPayload::SimRealParity`] row on EVERY decision
//! (Parity / ModeDivergentBranch / ReplayGoldenMismatch / SingleEntrypointMissing).
//! Even a Parity row is evidence. The golden is gate-manifest-pinned (frozen, not
//! agent-authored); the replay is orchestrator-driven via `replay_resume`.
//!
//! # SCOPE (TD-5, honest residue)
//! Same-env/same-seed equality ONLY. Structural parity (single entrypoint + no
//! mode-divergent execution branch + byte-reproducible replay) is NECESSARY but
//! NOT SUFFICIENT for behavioral parity; the market-impact / partial-fill /
//! latency gap is the DEFERRED Part B (GD-5). Cross-hardware FP is never asserted.
//!
//! # HARD RULES
//!   - No `.unwrap()`/`.expect()` outside `#[cfg(test)]` (CLAUDE.md).
//!   - blake3 only for hashing (CLAUDE.md anti-stale).
//!   - No new Rust crate; named constants (no magic numbers).
//!
//! References:
//!   - crates/orchestrator/src/rerun_equality.rs (H-23 pure-gate shape; mirrored).
//!   - crates/orchestrator/src/rra.rs (blake3 16-hex digest convention; REUSE).
//!   - crates/orchestrator/src/determinism.rs:364 (DriftComparator first-
//!     divergence -- REUSED here over the two event-digest sequences).
//!   - crates/orchestrator/src/ssb.rs::replay_resume (the replay substrate).
//!   - workflow/papers/lean-sim-real-parity-interface.md (LEAN grounding).
//!   - workflow/reasoning/H-24-decisions.md.

use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::determinism::{ActionTypeSequence, DriftComparator, DriftSink};
use crate::ssb::{SimRealParityKind, SimRealParityPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// LOCKED named constants
// ---------------------------------------------------------------------------

/// Canonical rule id for the sim-to-real parity gate (documentary parity with
/// the sibling gates' `*_RULE_ID`). The replay==golden DECISION is pure (no
/// rules.db threshold); this id pins the two seeded Part-A sub-gates (the
/// mode-divergent-branch AST ban + the single-entrypoint SchemaRequired manifest)
/// authored under `crates/rules/seed/repro/sim_real_parity.toml`.
pub const SIM_REAL_PARITY_RULE_ID: &str = "FIN-SIMREAL-001";

/// Companion rule id for the single-entrypoint SchemaRequired manifest sub-gate
/// (Part A). Documentary parity with the FIN-SIMREAL-001 AST ban.
pub const SIM_REAL_ENTRYPOINT_RULE_ID: &str = "FIN-SIMREAL-002";

/// Minimum number of golden events required before a `Parity` verdict can be a
/// PASS (hazard B: blake3 of an empty input is a fixed constant, so a zero-event
/// golden would fail-OPEN). A parity claim over ZERO replayed events is vacuous;
/// at least one event must be hashed.
pub const MIN_GOLDEN_EVENTS: usize = 1;

/// SSB `action_type` wire tag for the off-agent artifact (matches
/// [`crate::ssb::SsbActionType::SimRealParity::as_sql_str`]).
const SIM_REAL_PARITY_ACTION_TYPE: &str = "sim_real_parity";

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// One event of a replay/golden event log: an event name (or sequence label) +
/// the blake3 digest (16-hex prefix) of that event's serialized bytes.
///
/// The digest is computed over the event's recorded bytes ONLY (R4), NEVER over
/// stdout. The orchestrator's `replay_resume` produces the replay events by
/// hashing each replayed event; the frozen golden supplies its events directly.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventDigest {
    /// Event name / sequence label (must match across replay and golden).
    pub name: String,
    /// blake3 (16-hex prefix) of this event's serialized bytes.
    pub digest: String,
}

impl EventDigest {
    /// Construct an `EventDigest` by hashing raw event bytes with the RRA 16-hex
    /// blake3 convention. blake3 ONLY (CLAUDE.md HARD RULE).
    #[must_use]
    pub fn from_bytes(name: impl Into<String>, event_bytes: &[u8]) -> Self {
        let full = blake3::hash(event_bytes).to_hex();
        let digest = full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect();
        EventDigest {
            name: name.into(),
            digest,
        }
    }
}

/// One event log's full set of per-event digests, in recorded order. Used for
/// both the frozen golden and the seeded replay.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventLog {
    /// Per-event digests in recorded order.
    pub events: Vec<EventDigest>,
}

impl EventLog {
    /// Construct from `(name, digest)` pairs (digests already computed).
    #[must_use]
    pub fn from_pairs(pairs: Vec<(String, String)>) -> Self {
        EventLog {
            events: pairs
                .into_iter()
                .map(|(name, digest)| EventDigest { name, digest })
                .collect(),
        }
    }

    /// Number of events in the log.
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True when there are no events (hazard B fail-closed).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// The ordered event NAMES (the event-set the two logs must agree on).
    fn names(&self) -> Vec<String> {
        self.events.iter().map(|e| e.name.clone()).collect()
    }

    /// The ordered per-event DIGEST sequence, joined as `"<name>=<digest>"` so a
    /// same-name-different-digest divergence is caught by the reused
    /// [`DriftComparator`] first-divergence loop.
    fn digest_sequence(&self) -> Vec<String> {
        self.events
            .iter()
            .map(|e| format!("{}={}", e.name, e.digest))
            .collect()
    }

    /// True when there are no events, OR every event digest is empty/whitespace
    /// (a trivial golden whose blake3 would collide on the empty constant). This
    /// is the structural non-trivial check, hazard B.
    fn is_trivial(&self) -> bool {
        self.events.is_empty()
            || self.events.iter().all(|e| e.digest.trim().is_empty())
    }
}

/// Input bundle for [`verify_replay_golden`].
#[derive(Debug, Clone)]
pub struct SimRealInput {
    /// The experiment/strategy under test (repo-relative path or id).
    pub strategy_path: String,
    /// The FROZEN golden event log (gate-manifest-pinned, not agent-authored).
    pub golden: EventLog,
    /// The seeded replay event log produced by `replay_resume`. `None` =>
    /// fail-closed `ReplayGoldenMismatch` Block (the replay never happened --
    /// W-1/R1, hazard A).
    pub replay: Option<EventLog>,
    /// The DECLARED `ELAI_SEED` the replay was produced under (GD-2 same-env).
    pub seed: u64,
}

/// Verdict kind returned by [`verify_replay_golden`] (Part C; mirrors the inner
/// [`SimRealParityKind`] but with only the two Part-C-reachable arms -- the two
/// Part-A arms (`ModeDivergentBranch` / `SingleEntrypointMissing`) are emitted by
/// the SEEDED rule engine path, not by this pure replay gate).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimRealVerdictKind {
    /// The seeded replay reproduced the frozen golden event log byte-for-byte
    /// over a non-empty/non-trivial event set: structural+replay parity. PASS.
    Parity,
    /// The replay diverged from the golden, OR a fail-closed precondition failed
    /// (missing replay, empty/trivial golden, event-set mismatch). BLOCK. Carries
    /// the first divergent event (when one exists).
    ReplayGoldenMismatch,
}

/// Full verdict from [`verify_replay_golden`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimRealVerdict {
    /// Parity / ReplayGoldenMismatch.
    pub kind: SimRealVerdictKind,
    /// Machine-readable reason code ("parity" | "replay_golden_mismatch" |
    /// "missing_replay" | "empty_golden" | "event_set_mismatch").
    pub reason: String,
    /// Name of the first divergent event (empty on Parity / pre-comparison
    /// fail-closed).
    pub divergent_event: String,
    /// Index of the first divergent per-event digest (0 on Parity/fail-closed).
    pub first_divergence_index: u32,
    /// Number of golden events compared (0 on the empty/trivial path).
    pub declared_event_count: u32,
    /// The golden's combined event-log digest (empty on a pre-compute path).
    pub golden_digest: String,
    /// The replay's combined event-log digest (empty on missing replay).
    pub replay_digest: String,
    /// Free-text evidence string.
    pub evidence: String,
}

/// Errors from [`verify_replay_golden`]. Note: a missing replay, an empty/trivial
/// golden, and event-set mismatches are NOT errors -- they are fail-closed
/// `ReplayGoldenMismatch` VERDICTS (each emits an SSB row). Only an SSB write
/// failure is an error here.
#[derive(Debug, Error)]
pub enum SimRealParityError {
    /// Writing the off-agent SimRealParity SSB row failed.
    #[error("SSB write error: {0}")]
    SsbWrite(String),
}

// ---------------------------------------------------------------------------
// Main gate: verify_replay_golden (Part C)
// ---------------------------------------------------------------------------

/// Verify that a seeded replay reproduces a frozen golden event log byte-for-byte
/// (Tier-1 Part-C parity METRIC_GATE). Emits exactly one
/// [`SsbPayload::SimRealParity`] row on every decision (the off-agent artifact,
/// R4).
///
/// # Order of checks (FAIL-CLOSED first)
///   1. `replay.is_none()` => `ReplayGoldenMismatch` Block (reason
///      `missing_replay`, W-1/R1, hazard A): a single artifact cannot self-certify
///      parity.
///   2. golden empty/trivial OR `< MIN_GOLDEN_EVENTS` => `ReplayGoldenMismatch`
///      Block (reason `empty_golden`, hazard B): `blake3(b"")` is a fixed
///      constant, so two empty logs would hash equal and fail-OPEN.
///   3. event-NAME set mismatch (different names/order/count) =>
///      `ReplayGoldenMismatch` Block (reason `event_set_mismatch`, hazard C): a
///      missing/extra event IS a divergence.
///   4. Per-event digest diff (REUSING [`DriftComparator::compare`]'s first-
///      divergence over the two event-digest sequences): any event whose digest
///      differs => `ReplayGoldenMismatch` Block (reason `replay_golden_mismatch`),
///      naming the first divergent event.
///   5. All digests equal over a non-empty/non-trivial set => `Parity` PASS.
pub async fn verify_replay_golden(
    ssb: &Arc<Ssb>,
    input: SimRealInput,
) -> Result<SimRealVerdict, SimRealParityError> {
    let seed = input.seed;

    // -- Step 1: FAIL-CLOSED on a missing replay (W-1/R1, hazard A). --------
    let replay = match &input.replay {
        Some(r) => r,
        None => {
            warn!(
                strategy = %input.strategy_path,
                "sim_real_parity: replay artifact absent -> ReplayGoldenMismatch \
                 Block (fail-closed, W-1/R1; a single artifact cannot self-certify \
                 parity)"
            );
            let verdict = mismatch_verdict(
                "missing_replay",
                "No seeded replay artifact was produced: a single artifact cannot \
                 self-certify sim-to-real parity. Fail-closed Block (W-1/R1 \
                 missing-evidence=FAIL, hazard A).",
                String::new(),
                String::new(),
                0,
            );
            emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
            return Ok(verdict);
        }
    };
    let golden = &input.golden;

    // -- Step 2: FAIL-CLOSED on an empty/trivial golden (hazard B). ---------
    // blake3 of an empty input is a fixed constant; two empty logs would hash
    // equal and fail-OPEN. Reject BEFORE any Parity verdict can be reached.
    if golden.is_trivial() || golden.len() < MIN_GOLDEN_EVENTS {
        warn!(
            strategy = %input.strategy_path,
            golden_events = golden.len(),
            "sim_real_parity: empty/trivial golden event log -> \
             ReplayGoldenMismatch Block (fail-closed, hazard B; blake3 of empty \
             is a fixed constant)"
        );
        let verdict = mismatch_verdict(
            "empty_golden",
            "The frozen golden event log is empty or carries only empty digests. \
             blake3 of an empty input is a fixed constant, so an empty golden \
             would hash equal to an empty replay and fail-OPEN. A parity claim \
             over zero golden events is vacuous. Fail-closed Block (hazard B).",
            String::new(),
            String::new(),
            0,
        );
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 3: event-NAME set guard (same names, same order, same count). -
    // A missing/extra event IS a divergence, never silently skipped (hazard C).
    if golden.names() != replay.names() {
        warn!(
            strategy = %input.strategy_path,
            "sim_real_parity: replay/golden event-set mismatch -> \
             ReplayGoldenMismatch Block"
        );
        let verdict = mismatch_verdict(
            "event_set_mismatch",
            "The replay and the frozen golden declared different event \
             names/order/count: a missing/extra recorded event IS a sim-real \
             divergence. Fail-closed Block (hazard C).",
            combined_digest(golden),
            combined_digest(replay),
            0,
        );
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 4: per-event digest diff (REUSE DriftComparator first-div). ----
    // Wrap each log's per-event "<name>=<digest>" list as an ActionTypeSequence
    // so DriftComparator::compare names the FIRST diverging event for free
    // (GD-1: reuse the shipped first-divergence, do not re-implement it). The
    // comparator emits its own DeterminismDrift row on divergence; we route that
    // through a closed (drop) channel because the gate's canonical off-agent
    // artifact is the SimRealParity row, not a DeterminismDrift row.
    let seq_golden = ActionTypeSequence::new(golden.digest_sequence());
    let seq_replay = ActionTypeSequence::new(replay.digest_sequence());
    let comparator = DriftComparator::new(seed, drift_sink_drop());
    let combined_golden = combined_digest(golden);
    let combined_replay = combined_digest(replay);

    if let Some(drift) = comparator.compare(&seq_golden, &seq_replay) {
        let idx = usize::try_from(drift.first_divergence_index).unwrap_or(usize::MAX);
        let divergent_event = golden
            .events
            .get(idx)
            .map(|e| e.name.clone())
            .unwrap_or_else(|| "<unknown>".to_string());
        warn!(
            strategy = %input.strategy_path,
            event = %divergent_event,
            index = drift.first_divergence_index,
            "sim_real_parity: seeded replay diverged from the frozen golden event \
             log -> ReplayGoldenMismatch Block (a sim-real divergence)"
        );
        let verdict = SimRealVerdict {
            divergent_event,
            ..mismatch_verdict(
                "replay_golden_mismatch",
                &format!(
                    "Event index {} produced a different blake3 digest in the \
                     seeded replay than in the frozen golden event log: the \
                     recorded path is not byte-reproducible (a sim-real \
                     divergence). Fail-closed Block.",
                    drift.first_divergence_index
                ),
                combined_golden,
                combined_replay,
                drift.first_divergence_index,
            )
        };
        emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
        return Ok(verdict);
    }

    // -- Step 5: Parity PASS (all digests match over a non-trivial set). ----
    debug!(
        strategy = %input.strategy_path,
        events = golden.len(),
        "sim_real_parity: seeded replay reproduced the frozen golden event log \
         byte-for-byte -> Parity (structural+replay parity)"
    );
    let verdict = SimRealVerdict {
        kind: SimRealVerdictKind::Parity,
        reason: "parity".to_string(),
        divergent_event: String::new(),
        first_divergence_index: 0,
        declared_event_count: u32::try_from(golden.len()).unwrap_or(u32::MAX),
        golden_digest: combined_golden,
        replay_digest: combined_replay,
        evidence: format!(
            "Parity: all {} golden events were reproduced byte-identically by a \
             seeded (seed={}) replay. Structural+replay parity (same-env scope; \
             the numeric market-impact/partial-fill/latency gap is the DEFERRED \
             Part B, GD-5; cross-hardware FP not asserted, TD-5).",
            golden.len(),
            seed
        ),
    };
    emit_ssb(ssb, build_payload(&input, &verdict, seed)).await?;
    Ok(verdict)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a fail-closed / mismatch `ReplayGoldenMismatch` verdict.
fn mismatch_verdict(
    reason: &str,
    evidence: &str,
    golden_digest: String,
    replay_digest: String,
    first_divergence_index: u32,
) -> SimRealVerdict {
    SimRealVerdict {
        kind: SimRealVerdictKind::ReplayGoldenMismatch,
        reason: reason.to_string(),
        divergent_event: String::new(),
        first_divergence_index,
        declared_event_count: 0,
        golden_digest,
        replay_digest,
        evidence: evidence.to_string(),
    }
}

/// The combined event-log digest for a log: blake3 (16-hex) over the joined
/// per-event `"<name>=<digest>"` sequence. This is the log-level fingerprint
/// surfaced on the SSB row (mirrors the RRA root-over-bundle pattern). blake3
/// ONLY (CLAUDE.md HARD RULE).
fn combined_digest(log: &EventLog) -> String {
    let joined = log.digest_sequence().join("\n");
    let full = blake3::hash(joined.as_bytes()).to_hex();
    full.chars().take(crate::rra::RRA_HEX_PREFIX_LEN).collect()
}

/// A drop sink for the reused [`DriftComparator`]: the comparator emits its own
/// DeterminismDrift row on divergence, but the canonical off-agent artifact for
/// THIS gate is the SimRealParity row. We route the comparator's emit through a
/// channel whose receiver is dropped (the send is a no-op warn, not a failure),
/// so the DriftComparator first-divergence logic is reused WITHOUT emitting a
/// duplicate DeterminismDrift row.
fn drift_sink_drop() -> DriftSink {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    // Drop the receiver immediately; the comparator's send becomes a no-op
    // (DriftSink::Channel warns-and-continues on a closed channel -- it never
    // panics or fails the gate).
    drop(rx);
    DriftSink::Channel(tx)
}

// ---------------------------------------------------------------------------
// SSB emission
// ---------------------------------------------------------------------------

async fn emit_ssb(
    ssb: &Arc<Ssb>,
    payload: SimRealParityPayload,
) -> Result<(), SimRealParityError> {
    ssb.write_payload(
        SIM_REAL_PARITY_ACTION_TYPE,
        0,
        SsbPayload::SimRealParity(payload),
        Vec::new(),
    )
    .await
    .map_err(|e| SimRealParityError::SsbWrite(format!("{e}")))?;
    Ok(())
}

/// Build the off-agent [`SimRealParityPayload`] from inputs + verdict.
///
/// `corpus_blake3` pins the exact inputs so the verdict is recomputable (GD-2).
/// blake3 ONLY (CLAUDE.md HARD RULE).
fn build_payload(
    input: &SimRealInput,
    verdict: &SimRealVerdict,
    seed: u64,
) -> SimRealParityPayload {
    let kind = match verdict.kind {
        SimRealVerdictKind::Parity => SimRealParityKind::Parity,
        SimRealVerdictKind::ReplayGoldenMismatch => SimRealParityKind::ReplayGoldenMismatch,
    };

    let declared_event_count = if verdict.declared_event_count > 0 {
        verdict.declared_event_count
    } else {
        // On Parity the verdict carries the count; on fail-closed paths it is 0.
        u32::try_from(input.golden.len()).unwrap_or(u32::MAX)
    };

    let corpus_key = format!(
        "{}:{}:{}:{}",
        input.strategy_path, verdict.reason, verdict.first_divergence_index, seed
    );
    let corpus_blake3 = blake3::hash(corpus_key.as_bytes()).to_hex().to_string();

    SimRealParityPayload {
        kind,
        strategy_path: input.strategy_path.clone(),
        golden_digest: verdict.golden_digest.clone(),
        replay_digest: verdict.replay_digest.clone(),
        first_divergence_index: verdict.first_divergence_index,
        divergent_event: verdict.divergent_event.clone(),
        declared_event_count,
        verdict: verdict.reason.clone(),
        seed,
        corpus_blake3,
    }
}

// ---------------------------------------------------------------------------
// Unit tests (replay==golden equality + first-divergence wrap + the guards)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;

    fn test_ssb() -> Arc<Ssb> {
        let dir = std::env::temp_dir().join(format!("h24-unit-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfg = SsbConfig {
            db_path: dir.join(format!("ssb-{}.db", rand_suffix())),
            pool_size: Some(2),
            project_id: "h24-unit".to_string(),
            session_id: "h24-unit-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
        Arc::new(Ssb::new(cfg, clock).expect("ssb init"))
    }

    fn rand_suffix() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = SystemTime::now() // entropy-allow: test-tmpdir-collision-avoidance-nonce-not-replay-compared
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        nanos ^ (COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_mul(0x9E3779B97F4A7C15))
    }

    fn el(pairs: Vec<(&str, &str)>) -> EventLog {
        EventLog::from_pairs(
            pairs
                .into_iter()
                .map(|(n, d)| (n.to_string(), d.to_string()))
                .collect(),
        )
    }

    fn input(golden: EventLog, replay: Option<EventLog>, seed: u64) -> SimRealInput {
        SimRealInput {
            strategy_path: "strat.py".to_string(),
            golden,
            replay,
            seed,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn identical_replay_passes_parity() {
        let ssb = test_ssb();
        let golden = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
        let replay = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
        let v = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::Parity);
        assert_eq!(v.reason, "parity");
        assert_eq!(v.declared_event_count, 2);
        assert_eq!(v.golden_digest, v.replay_digest);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn divergent_replay_blocks_naming_first_event() {
        let ssb = test_ssb();
        let golden = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
        // replay's fill digest differs (a different fill model in backtest).
        let replay = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "cccc")]);
        let v = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::ReplayGoldenMismatch);
        assert_eq!(v.reason, "replay_golden_mismatch");
        assert_eq!(v.divergent_event, "fill:AAPL");
        assert_eq!(v.first_divergence_index, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn missing_replay_is_fail_closed_block_hazard_a() {
        let ssb = test_ssb();
        let golden = el(vec![("order:AAPL", "aaaa")]);
        let v = verify_replay_golden(&ssb, input(golden, None, 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::ReplayGoldenMismatch);
        assert_eq!(v.reason, "missing_replay");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_golden_is_fail_closed_block_hazard_b() {
        let ssb = test_ssb();
        // An empty golden would hash equal to an empty replay (blake3 of empty
        // is constant); Block.
        let golden = EventLog::default();
        let replay = EventLog::default();
        let v = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::ReplayGoldenMismatch);
        assert_eq!(v.reason, "empty_golden");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn trivial_empty_digest_golden_is_fail_closed_block_hazard_b() {
        let ssb = test_ssb();
        let golden = el(vec![("order:AAPL", "")]);
        let replay = el(vec![("order:AAPL", "")]);
        let v = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::ReplayGoldenMismatch);
        assert_eq!(v.reason, "empty_golden");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn event_set_mismatch_is_block_hazard_c() {
        let ssb = test_ssb();
        let golden = el(vec![("order:AAPL", "aaaa"), ("fill:AAPL", "bbbb")]);
        let replay = el(vec![("order:AAPL", "aaaa")]);
        let v = verify_replay_golden(&ssb, input(golden, Some(replay), 42))
            .await
            .expect("verify");
        assert_eq!(v.kind, SimRealVerdictKind::ReplayGoldenMismatch);
        assert_eq!(v.reason, "event_set_mismatch");
    }

    #[test]
    fn from_bytes_uses_blake3_16hex_prefix() {
        let ed = EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market");
        assert_eq!(ed.name, "order:AAPL");
        assert_eq!(ed.digest.len(), crate::rra::RRA_HEX_PREFIX_LEN);
        // Same input is stable; distinct inputs produce distinct digests.
        let ed2 = EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market");
        assert_eq!(ed.digest, ed2.digest);
        let ed3 = EventDigest::from_bytes("order:AAPL", b"order AAPL 200 @ market");
        assert_ne!(ed.digest, ed3.digest);
    }
}
