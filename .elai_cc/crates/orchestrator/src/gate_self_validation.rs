//! H-27 (SCHEMA_VERSION 235): gate self-validation — the verify-the-verifiers META
//! gate.
//!
//! A "compiles + unit tests pass" gate is structurally blind to a fail-OPEN gate: a
//! gate that has silently stopped BLOCKING the violation it exists to catch (the REGRET
//! symmetric-vs-one-sided-band incident) still compiles and still passes its own unit
//! tests. This module mechanically PROVES that the 8 Track-3 trust gates each still kill
//! their own planted mutant — i.e. each gate, driven with the exact known-bad input it
//! is designed to catch, returns a BLOCK-class verdict. A surviving mutant means the
//! gate is fail-open and the meta-gate BLOCKS promotion, naming the broken gate.
//!
//! # The planted mutants are the gates' OWN demo-violated fixtures
//! Each Track-3 gate ships a CLI `*-check --demo-*-violated` fixture: the exact input
//! that SHOULD make the gate Block. H-27 replicates that fixture as data and drives the
//! gate's REAL async `evaluate_*` fn over it. The gate's own demo-violation IS the
//! planted mutant; "killed" = the gate returned a BLOCK-class verdict. This grounds the
//! mutant corpus in the gates' own design intent rather than an out-of-band guess.
//!
//! # Reuse of mutation-score SEMANTICS, not the runner (GD-1 corrected)
//! [`crate::mutation_testing::MutationRunner`] shells out to `cargo-mutants` over disk
//! source files; it cannot score "a gate fn + a planted input." H-27 is a NEW in-process
//! planted-fixture driver. It reuses only the `mutation_score == 100.0` SEMANTICS: every
//! planted mutant must be killed (no survivors) for the verify-the-verifiers PASS.
//!
//! # Fail-closed (R1/R2)
//!   * EMPTY registry => `EmptyRegistry` Block (a self-validation that checks ZERO gates
//!     is itself a FAIL — ArchUnit `failOnEmptyShould`).
//!   * a gate that fails to block its violation fixture => `GateFailedToBlock` Block
//!     (the mutant SURVIVED; the gate is fail-open). Load-bearing catch.
//!   * a gate that blocks its clean fixture => `GateOverBlocked` Block (false-positive
//!     over-block).
//!   * a gate whose evaluate fn errors / cannot be run => `GateUnreachable` Block
//!     (not-runnable = never ran = FAIL, never UNVERIFIED).
//!
//! # Self-reference (TD-4)
//! H-27 validates the 8 OTHER Track-3 gates, NEVER itself — else infinite regress (the
//! validator validating the validator…). The verifier-of-verifiers is the irreducible
//! TOP of the stack; its soundness is routed to a human-authored frozen corpus + R8
//! gate-freeze, NOT to self-application. See `workflow/reasoning/H-27-decisions.md`.
//!
//! # No .unwrap()/.expect() in non-test code (CLAUDE.md HARD RULE). blake3 only.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::{debug, warn};

use crate::ssb::{GateSelfValidationKind, GateSelfValidationPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants (CLAUDE.md no-magic-numbers)
// ---------------------------------------------------------------------------

/// The mutation score (percent) required for the verify-the-verifiers PASS: EVERY
/// planted mutant must be killed (no survivors). Reused SEMANTICS from
/// [`crate::mutation_testing`] — a kill rate below 100.0 means a gate is fail-open.
pub const REQUIRED_MUTATION_SCORE_PCT: f64 = 100.0;

/// Agent id stamped on the emitted [`SsbPayload::GateSelfValidation`] row.
const SELF_VALIDATION_AGENT_ID: &str = "gate-self-validation";

/// Superstep stamped on the emitted row (single-shot meta-gate; not part of a graph
/// superstep loop).
const SELF_VALIDATION_SUPERSTEP: i64 = 0;

/// The 8 Track-3 gates this META gate validates (it validates the OTHER gates, NEVER
/// itself — no infinite regress). This is the FROZEN should-set; an empty registry is a
/// fail-closed `EmptyRegistry` Block (R1).
const REGISTRY: &[GateEntry] = &[
    GateEntry { id: GateId::Deflation, has_clean_fixture: false },
    GateEntry { id: GateId::Leakage, has_clean_fixture: true },
    GateEntry { id: GateId::Lookahead, has_clean_fixture: true },
    GateEntry { id: GateId::TradingNull, has_clean_fixture: true },
    GateEntry { id: GateId::Rerun, has_clean_fixture: true },
    GateEntry { id: GateId::SimReal, has_clean_fixture: true },
    GateEntry { id: GateId::ResultManifest, has_clean_fixture: true },
    GateEntry { id: GateId::ShadowExec, has_clean_fixture: true },
];

// ---------------------------------------------------------------------------
// Gate identity + registry entry
// ---------------------------------------------------------------------------

/// The 8 Track-3 trust gates the meta-gate validates.
///
/// H-27 validates these OTHER gates, never itself (TD-4 no-regress).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateId {
    /// H-18 deflated-Sharpe / PBO promotion gate (`evaluate_deflation`).
    Deflation,
    /// H-20 data-leakage negative-reachability gate (`evaluate_leakage`).
    Leakage,
    /// H-21 look-ahead future-truncation differential gate (`diff_future_truncation`).
    Lookahead,
    /// H-22 trading metamorphic NULL-test gate (`evaluate_trading_null`).
    TradingNull,
    /// H-23 same-seed re-run hash-equality gate (`verify_rerun_digest`).
    Rerun,
    /// H-24 sim-to-real parity gate (`verify_replay_golden`).
    SimReal,
    /// H-25 result-manifest re-derivation gate (`verify_result`).
    ResultManifest,
    /// H-26 shadow-exec + financial-invariant MR gate (`diff_shadow_impls`).
    ShadowExec,
}

impl GateId {
    /// Stable lower-snake name for the SSB row / log / verdict.
    pub fn name(self) -> &'static str {
        match self {
            GateId::Deflation => "deflation",
            GateId::Leakage => "leakage",
            GateId::Lookahead => "lookahead",
            GateId::TradingNull => "trading_null",
            GateId::Rerun => "rerun",
            GateId::SimReal => "sim_real",
            GateId::ResultManifest => "result_manifest",
            GateId::ShadowExec => "shadow_exec",
        }
    }
}

/// A frozen registry row: a gate + whether it ships a deterministic CLEAN fixture for
/// the over-block check. Deflation's Allow path is numerically fragile (the DSR
/// composite gate at threshold 0.95 depends on the exact Sharpe/T math), so it carries
/// no clean fixture — its meta-check is the load-bearing kill-the-mutant leg only.
#[derive(Debug, Clone, Copy)]
pub struct GateEntry {
    /// The gate this row validates.
    pub id: GateId,
    /// Whether the gate ships a deterministic clean fixture for the over-block check.
    pub has_clean_fixture: bool,
}

impl GateEntry {
    /// Construct a registry row (for callers / tests that drive an explicit registry
    /// slice through [`validate_gates_with`]).
    pub fn new(id: GateId, has_clean_fixture: bool) -> Self {
        Self { id, has_clean_fixture }
    }
}

/// Which fixture to drive a gate with: the planted VIOLATION (must be killed = Blocked)
/// or the CLEAN fixture (must NOT be over-blocked = Allowed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FixtureKind {
    /// The gate's own demo-violated input (the planted mutant). A correct gate BLOCKS.
    Violation,
    /// The gate's own demo-clean input. A correct gate ALLOWS (no false-positive).
    Clean,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the gate self-validation driver. An error driving any gate is mapped to
/// a `GateUnreachable` verdict (fail-closed R2), NOT propagated as a meta-gate crash —
/// "not runnable = never ran = FAIL," but the meta-gate still emits its row.
#[derive(Debug, Error)]
pub enum GateSelfValidationError {
    /// SSB write failure for the meta-gate's own row.
    #[error("SSB error: {0}")]
    Ssb(#[from] crate::ssb::SsbError),

    /// A trial-ledger open/register failure while building a gate fixture (deflation /
    /// trading_null open a `TrialLedger` on disk).
    #[error("trial ledger error: {0}")]
    Ledger(String),
}

// ---------------------------------------------------------------------------
// Outcome
// ---------------------------------------------------------------------------

/// The verify-the-verifiers outcome. `kind == AllGatesValid` is the only PASS.
#[derive(Debug, Clone, PartialEq)]
pub struct GateSelfValidationOutcome {
    /// The verdict class (one PASS arm + four fail-closed BLOCK arms).
    pub kind: GateSelfValidationKind,
    /// Number of gates in the frozen registry (the should-set).
    pub gates_total: u32,
    /// Number of gates that BLOCKED their planted mutant (killed it).
    pub gates_killed: u32,
    /// `gates_killed / gates_total * 100` (reused mutation-score semantics). 0.0 when
    /// the registry is empty (never the vacuous 100.0 pass).
    pub mutation_score_pct: f64,
    /// Name of the first gate that failed (escaped / over-blocked / unreachable); empty
    /// on AllGatesValid / EmptyRegistry.
    pub escaped_gate: String,
    /// Reason / fixture code of the first failure; empty when not applicable.
    pub escaped_fixture: String,
}

impl GateSelfValidationOutcome {
    /// `true` iff the meta-gate ALLOWS gate promotion (every gate killed its mutant and
    /// none over-blocked). Any fail-closed arm => `false`.
    pub fn is_pass(&self) -> bool {
        self.kind == GateSelfValidationKind::AllGatesValid
    }
}

// ---------------------------------------------------------------------------
// Public driver
// ---------------------------------------------------------------------------

/// Drive the FROZEN registry of Track-3 gates and emit ONE
/// [`SsbPayload::GateSelfValidation`] row (the off-agent done-gate artifact, R4).
///
/// For each registered gate: replay its OWN demo-violation fixture through its REAL
/// async `evaluate_*` fn and require a BLOCK-class verdict (kill the mutant). If the
/// gate carries a clean fixture, replay that too and require an ALLOW-class verdict (no
/// over-block). Fail-closed everywhere (R1/R2).
///
/// `inject_weakened_gate` simulates a fail-OPEN gate (the `--demo-gate-broken` CLI
/// path): a synthetic gate whose violation fixture is NOT blocked, proving the meta-gate
/// surfaces a surviving mutant as `GateFailedToBlock`.
///
/// # Arguments
/// * `ssb`     — the live SSB the meta-gate emits its own row into (and that the inner
///   gates emit their per-gate rows into as a side effect of being driven).
/// * `db_path` — the SSB db path; deflation / trading_null open a `TrialLedger` here.
/// * `run_id`  — declared run id stamped on the row.
/// * `seed`    — declared `ELAI_SEED` (GD-2 reproducibility); 0 when undeclared.
/// * `inject_weakened_gate` — `true` only on the `--demo-gate-broken` demo path.
pub async fn validate_gates(
    ssb: &Arc<Ssb>,
    db_path: &Path,
    run_id: &str,
    seed: u64,
    inject_weakened_gate: bool,
) -> Result<GateSelfValidationOutcome, GateSelfValidationError> {
    // R1: an EMPTY registry is a FAIL, never a vacuous pass. (REGISTRY is a frozen
    // const, so this is a fail-closed guard for the future / for a mis-edit, and is
    // exercised directly by validate_gates_with for the contract test.)
    let outcome = validate_gates_with(ssb, db_path, REGISTRY, inject_weakened_gate).await?;
    emit_outcome(ssb, &outcome, run_id, seed).await?;
    Ok(outcome)
}

/// Core driver over an EXPLICIT registry slice (so the contract test can drive an empty
/// registry, a single-gate registry, etc.). Does NOT emit the SSB row — `validate_gates`
/// owns the single emission. Pure scoring; fail-closed on every leg.
pub async fn validate_gates_with(
    ssb: &Arc<Ssb>,
    db_path: &Path,
    registry: &[GateEntry],
    inject_weakened_gate: bool,
) -> Result<GateSelfValidationOutcome, GateSelfValidationError> {
    // The weakened synthetic gate (demo / contract only) counts toward the should-set
    // so its survival is a real GateFailedToBlock, not a silently-dropped gate.
    let gates_total: u32 = registry
        .len()
        .saturating_add(if inject_weakened_gate { 1 } else { 0 })
        .try_into()
        .unwrap_or(u32::MAX);

    // R1 EMPTY-REGISTRY GUARD: zero gates to validate is itself a FAIL.
    if gates_total == 0 {
        warn!("gate_self_validation: EMPTY registry -> EmptyRegistry Block (R1)");
        return Ok(GateSelfValidationOutcome {
            kind: GateSelfValidationKind::EmptyRegistry,
            gates_total: 0,
            gates_killed: 0,
            mutation_score_pct: 0.0,
            escaped_gate: String::new(),
            escaped_fixture: String::new(),
        });
    }

    let mut gates_killed: u32 = 0;

    for entry in registry {
        // -- Kill-the-mutant leg: the gate MUST BLOCK its planted violation fixture.
        match run_gate(ssb, db_path, entry.id, FixtureKind::Violation).await {
            Ok(true) => {
                // Mutant killed (gate Blocked the violation).
                gates_killed = gates_killed.saturating_add(1);
                debug!(gate = entry.id.name(), "gate_self_validation: mutant killed");
            }
            Ok(false) => {
                // R1 SURVIVING-MUTANT GUARD: the gate is fail-open. BLOCK, name it.
                warn!(
                    gate = entry.id.name(),
                    "gate_self_validation: mutant SURVIVED (gate did not block its \
                     violation fixture) -> GateFailedToBlock Block"
                );
                return Ok(fail(
                    GateSelfValidationKind::GateFailedToBlock,
                    gates_total,
                    gates_killed,
                    entry.id.name(),
                    "violation_not_blocked",
                ));
            }
            Err(e) => {
                // R2 UNREACHABLE GUARD: not-runnable = never ran = FAIL.
                warn!(
                    gate = entry.id.name(),
                    error = %e,
                    "gate_self_validation: gate un-runnable -> GateUnreachable Block (R2)"
                );
                return Ok(fail(
                    GateSelfValidationKind::GateUnreachable,
                    gates_total,
                    gates_killed,
                    entry.id.name(),
                    "evaluate_errored",
                ));
            }
        }

        // -- Over-block leg: a clean fixture must NOT be blocked (no false-positive).
        if entry.has_clean_fixture {
            match run_gate(ssb, db_path, entry.id, FixtureKind::Clean).await {
                Ok(false) => {
                    // Clean allowed (correct: no over-block).
                    debug!(gate = entry.id.name(), "gate_self_validation: clean allowed");
                }
                Ok(true) => {
                    // OVER-BLOCK GUARD: the gate rejected a known-good input.
                    warn!(
                        gate = entry.id.name(),
                        "gate_self_validation: clean fixture OVER-BLOCKED -> \
                         GateOverBlocked Block"
                    );
                    return Ok(fail(
                        GateSelfValidationKind::GateOverBlocked,
                        gates_total,
                        gates_killed,
                        entry.id.name(),
                        "clean_over_blocked",
                    ));
                }
                Err(e) => {
                    warn!(
                        gate = entry.id.name(),
                        error = %e,
                        "gate_self_validation: clean-fixture run errored -> \
                         GateUnreachable Block (R2)"
                    );
                    return Ok(fail(
                        GateSelfValidationKind::GateUnreachable,
                        gates_total,
                        gates_killed,
                        entry.id.name(),
                        "clean_evaluate_errored",
                    ));
                }
            }
        }
    }

    // -- The synthetic weakened gate (demo / contract): its violation fixture is NOT
    // blocked by construction, so it must surface as a surviving mutant. This is the
    // `--demo-gate-broken` proof that the meta-gate catches a fail-open gate.
    if inject_weakened_gate {
        warn!(
            "gate_self_validation: injected WEAKENED gate did not block its violation \
             fixture -> GateFailedToBlock Block (demo / proof)"
        );
        return Ok(fail(
            GateSelfValidationKind::GateFailedToBlock,
            gates_total,
            gates_killed,
            "weakened_demo_gate",
            "violation_not_blocked",
        ));
    }

    // Every mutant killed, no over-block, non-empty registry => mutation_score 100.0.
    let score = mutation_score(gates_killed, gates_total);
    if (score - REQUIRED_MUTATION_SCORE_PCT).abs() > f64::EPSILON {
        // Defensive: should be unreachable (a survivor returns early above), but a kill
        // count below total is a fail-closed GateFailedToBlock, never a silent pass.
        warn!(
            gates_killed,
            gates_total,
            score,
            "gate_self_validation: mutation score below 100% with no early survivor -> \
             GateFailedToBlock Block (defensive)"
        );
        return Ok(fail(
            GateSelfValidationKind::GateFailedToBlock,
            gates_total,
            gates_killed,
            "unknown",
            "mutation_score_below_100",
        ));
    }

    debug!(
        gates_total,
        gates_killed,
        "gate_self_validation: AllGatesValid (every mutant killed, no over-block)"
    );
    Ok(GateSelfValidationOutcome {
        kind: GateSelfValidationKind::AllGatesValid,
        gates_total,
        gates_killed,
        mutation_score_pct: score,
        escaped_gate: String::new(),
        escaped_fixture: String::new(),
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Compute `killed / total * 100`. Returns 0.0 for an empty should-set (the
/// EmptyRegistry path handles that earlier; this never emits the vacuous 100.0).
fn mutation_score(killed: u32, total: u32) -> f64 {
    if total == 0 {
        return 0.0;
    }
    f64::from(killed) / f64::from(total) * 100.0
}

/// Build a fail-closed BLOCK outcome naming the offending gate + reason.
fn fail(
    kind: GateSelfValidationKind,
    gates_total: u32,
    gates_killed: u32,
    gate: &str,
    fixture: &str,
) -> GateSelfValidationOutcome {
    GateSelfValidationOutcome {
        kind,
        gates_total,
        gates_killed,
        mutation_score_pct: mutation_score(gates_killed, gates_total),
        escaped_gate: gate.to_string(),
        escaped_fixture: fixture.to_string(),
    }
}

/// Emit the single off-agent [`SsbPayload::GateSelfValidation`] row. The
/// `corpus_blake3` binds the verdict to the registry + outcome so it is recomputable
/// (GD-2). BLAKE3 ONLY.
async fn emit_outcome(
    ssb: &Arc<Ssb>,
    outcome: &GateSelfValidationOutcome,
    run_id: &str,
    seed: u64,
) -> Result<(), GateSelfValidationError> {
    let key = format!(
        "{}:{}:{}:{}:{}:{}",
        run_id,
        outcome.gates_total,
        outcome.gates_killed,
        outcome.escaped_gate,
        outcome.escaped_fixture,
        seed
    );
    let corpus_blake3 = blake3::hash(key.as_bytes()).to_hex().to_string();

    let payload = SsbPayload::GateSelfValidation(GateSelfValidationPayload {
        kind: outcome.kind,
        gates_total: outcome.gates_total,
        gates_killed: outcome.gates_killed,
        mutation_score_pct: outcome.mutation_score_pct,
        escaped_gate: outcome.escaped_gate.clone(),
        escaped_fixture: outcome.escaped_fixture.clone(),
        run_id: run_id.to_string(),
        seed,
        corpus_blake3,
    });

    ssb.write_payload(
        SELF_VALIDATION_AGENT_ID,
        SELF_VALIDATION_SUPERSTEP,
        payload,
        vec!["gate_self_validation.rs".to_string()],
    )
    .await?;
    Ok(())
}

/// Drive one gate with one fixture and return `Ok(true)` iff the gate produced a
/// BLOCK-class verdict. Each arm replicates that gate's OWN demo fixture (the same input
/// shape the gate's `*-check` CLI uses) and calls the gate's REAL async `evaluate_*` fn.
async fn run_gate(
    ssb: &Arc<Ssb>,
    db_path: &Path,
    gate: GateId,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    match gate {
        GateId::Deflation => run_deflation(ssb, db_path, fixture).await,
        GateId::Leakage => run_leakage(ssb, fixture).await,
        GateId::Lookahead => run_lookahead(ssb, fixture).await,
        GateId::TradingNull => run_trading_null(ssb, db_path, fixture).await,
        GateId::Rerun => run_rerun(ssb, fixture).await,
        GateId::SimReal => run_sim_real(ssb, fixture).await,
        GateId::ResultManifest => run_result_manifest(ssb, fixture).await,
        GateId::ShadowExec => run_shadow_exec(ssb, fixture).await,
    }
}

// --- per-gate fixture drivers (replicate each gate's OWN demo fixture) --------

/// Deflation (H-18): the planted mutant is `pbo: None` => the gate must Block (R1/R2
/// fail-closed on absent PBO). No clean fixture (the Allow path is numerically fragile);
/// `Clean` is never requested for this gate.
async fn run_deflation(
    ssb: &Arc<Ssb>,
    db_path: &Path,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::deflation_gate::{evaluate_deflation, DeflationInput, GateVerdictKind};
    use crate::trial_ledger::TrialLedger;

    const OBJECTIVE: &str = "h27_selfval_deflation";
    const WINDOW: &str = "0..1";
    const SESSION: &str = "elai-h27-gate-selfval";

    let ledger = TrialLedger::open(db_path, SESSION)
        .map_err(|e| GateSelfValidationError::Ledger(format!("{e}")))?;
    let clock: Arc<dyn elai_core::clock::ClockProvider> =
        Arc::new(elai_core::clock::OsClock);
    // Register one distinct trial so the TD-1 fail-closed N read succeeds (N >= 1).
    ledger
        .register_trial(ssb, &clock, OBJECTIVE, WINDOW, b"{\"h27\":\"deflation\"}", 42)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("{e}")))?;

    // The planted mutant: pbo absent => the gate MUST Block (the value error a fail-open
    // gate would let through). `Clean` is never used for deflation.
    let pbo = match fixture {
        FixtureKind::Violation => None,
        FixtureKind::Clean => Some(0.1),
    };
    let input = DeflationInput {
        objective: OBJECTIVE.to_string(),
        data_window: WINDOW.to_string(),
        sr_observed: 0.10,
        t_periods: 252,
        skew: 0.0,
        excess_kurt: 0.0,
        sharpe_std: 0.05,
        sr_benchmark: 0.0,
        pbo,
        seed: 42,
    };
    let verdict = evaluate_deflation(&ledger, ssb, input)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("deflation: {e}")))?;
    Ok(matches!(verdict.kind, GateVerdictKind::Block))
}

/// Leakage (H-20): the planted mutant is an in-memory DKG where the fit-site reaches
/// BOTH the train and test sinks (the L1.2 leak) => SoundBlock. The clean fixture is a
/// DKG where fit reaches only the train sink => Allow.
async fn run_leakage(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::leakage_gate::{evaluate_leakage, LeakageInput, LeakageVerdictKind, SplitBoundary};
    use elai_rules::predicate::{DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams};

    const MODULE: &str = "h27_pipeline.py";

    /// In-memory DKG whose fit-site reaches BOTH sinks (the leak). Mirrors the
    /// `--demo-fit-reaches-both` `DemoLeakyDkg` in `leakage_check_cmd.rs`.
    struct LeakyDkg;
    impl DkgQuerier for LeakyDkg {
        fn query_reachability(
            &self,
            params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            if params.sink_pattern == "*" {
                return Ok(vec![ReachabilityMatch {
                    path: vec![MODULE.to_string()],
                    edge_labels: vec!["data_flow".to_string()],
                }]);
            }
            // fit -> train AND fit -> test both reach => leak.
            Ok(vec![ReachabilityMatch {
                path: vec![
                    "sklearn.StandardScaler.fit_transform".to_string(),
                    format!("X_{}", params.sink_pattern),
                ],
                edge_labels: vec!["data_flow".to_string()],
            }])
        }
        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    /// In-memory DKG where fit reaches ONLY the train sink (clean): the presence probe
    /// and the train sink return a match, the test sink returns empty => Allow.
    struct CleanDkg;
    impl DkgQuerier for CleanDkg {
        fn query_reachability(
            &self,
            params: &ReachabilityParams,
        ) -> Result<Vec<ReachabilityMatch>, DkgError> {
            if params.sink_pattern == "*" || params.sink_pattern == "train" {
                return Ok(vec![ReachabilityMatch {
                    path: vec![MODULE.to_string()],
                    edge_labels: vec!["data_flow".to_string()],
                }]);
            }
            // test sink: no path => fit does not reach the test set => no leak.
            Ok(Vec::new())
        }
        fn validate_params(&self, _params: &ReachabilityParams) -> Result<(), DkgError> {
            Ok(())
        }
    }

    let input = LeakageInput {
        module_path: MODULE.to_string(),
        declared_split: Some(SplitBoundary {
            split_symbol: "sklearn.train_test_split".to_string(),
            file: MODULE.to_string(),
            line: 1,
        }),
        fit_source_pattern: "fit_transform".to_string(),
        is_feature_selection: false,
        residue_flag: None,
        dkg_available: true,
        seed: 42,
    };
    // now_unix: a fixed timestamp; no suppressions => every sound block stands.
    const NOW_UNIX: i64 = 1_717_600_000;
    let suppressions: Vec<crate::leakage_gate::LeakageSuppression> = Vec::new();

    let verdict = match fixture {
        FixtureKind::Violation => {
            evaluate_leakage(&LeakyDkg, ssb, input, &suppressions, NOW_UNIX)
                .await
                .map_err(|e| GateSelfValidationError::Ledger(format!("leakage: {e}")))?
        }
        FixtureKind::Clean => {
            evaluate_leakage(&CleanDkg, ssb, input, &suppressions, NOW_UNIX)
                .await
                .map_err(|e| GateSelfValidationError::Ledger(format!("leakage: {e}")))?
        }
    };
    // Block-class = anything that is not a plain Allow. SoundBlock / DkgUnavailable /
    // HumanReviewRequired are all NON-Allow; the clean fixture must produce Allow.
    Ok(!matches!(verdict.kind, LeakageVerdictKind::Allow))
}

/// Lookahead (H-21): the planted mutant is the `iloc[t+1]` future-leak column pair (the
/// truncated run differs at the boundary row) => FutureLeak. The clean fixture is a
/// backward-only trailing-SMA pair => Causal Allow.
async fn run_lookahead(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::lookahead_differential::{
        diff_future_truncation, ColumnSet, FpTolerance, LookaheadInput, LookaheadVerdictKind,
    };

    const BASE: &[f64] = &[10.0, 11.0, 12.0, 13.0, 14.0];
    const PEEKED_FUTURE: f64 = 99.0;

    let n = BASE.len();
    let (full, truncated) = match fixture {
        FixtureKind::Violation => {
            // iloc[t+1] leak: boundary row reads the peeked future on the full run and
            // falls back to the backward value on the truncated run => differs at n-1.
            let mut leak_full = Vec::with_capacity(n);
            let mut leak_trunc = Vec::with_capacity(n);
            for t in 0..n {
                if t + 1 < n {
                    leak_full.push(BASE[t + 1]);
                    leak_trunc.push(BASE[t + 1]);
                } else {
                    leak_full.push(PEEKED_FUTURE);
                    leak_trunc.push(BASE[t]);
                }
            }
            let full = ColumnSet::from_pairs(vec![
                ("sma_causal".to_string(), BASE.to_vec()),
                ("next_close_leak".to_string(), leak_full),
            ]);
            let truncated = ColumnSet::from_pairs(vec![
                ("sma_causal".to_string(), BASE.to_vec()),
                ("next_close_leak".to_string(), leak_trunc),
            ]);
            (full, truncated)
        }
        FixtureKind::Clean => {
            // Backward-only trailing mean: identical in both runs over the overlap.
            let mut sma = Vec::with_capacity(n);
            let mut acc = 0.0;
            for (i, &v) in BASE.iter().enumerate() {
                acc += v;
                sma.push(acc / (i as f64 + 1.0));
            }
            let full = ColumnSet::from_pairs(vec![
                ("close".to_string(), BASE.to_vec()),
                ("trailing_sma".to_string(), sma.clone()),
            ]);
            let truncated = ColumnSet::from_pairs(vec![
                ("close".to_string(), BASE.to_vec()),
                ("trailing_sma".to_string(), sma),
            ]);
            (full, truncated)
        }
    };

    let input = LookaheadInput {
        strategy_path: "h27_features.py".to_string(),
        full_run: full,
        truncated_run: Some(truncated),
        tolerance: FpTolerance::default(),
        seed: 42,
    };
    let verdict = diff_future_truncation(ssb, input)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("lookahead: {e}")))?;
    // Block-class = anything not Causal (FutureLeak / Unprovable).
    Ok(!matches!(verdict.kind, LookaheadVerdictKind::Causal))
}

/// Trading-null (H-22): the planted mutant is the constant-long signal on the zero-mean
/// alternating market (the spurious edge SURVIVES the null) => EdgeSurvivedNull Block.
/// The clean fixture is a sign-aligned signal whose edge COLLAPSES to null => Allow.
async fn run_trading_null(
    ssb: &Arc<Ssb>,
    db_path: &Path,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::trading_null_gate::{
        evaluate_trading_null, TradingNullInput, TradingNullVerdictKind, DEFAULT_BLOCK_SIZE,
        K_MIN_PERMUTATIONS,
    };
    use crate::trial_ledger::TrialLedger;

    const OBJECTIVE: &str = "h27_selfval_null";
    const WINDOW: &str = "0..1";
    const SESSION: &str = "elai-h27-gate-selfval-null";
    const SERIES_LEN: usize = 120;
    const RETURN_AMP: f64 = 1.0;

    let ledger = TrialLedger::open(db_path, SESSION)
        .map_err(|e| GateSelfValidationError::Ledger(format!("{e}")))?;
    let clock: Arc<dyn elai_core::clock::ClockProvider> =
        Arc::new(elai_core::clock::OsClock);
    ledger
        .register_trial(ssb, &clock, OBJECTIVE, WINDOW, b"{\"h27\":\"null\"}", 42)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("{e}")))?;

    // Zero-mean alternating market (mirrors null_test_cmd::demo_market_returns).
    let returns: Vec<f64> = (0..SERIES_LEN)
        .map(|i| {
            let mag = ((i % 3) + 1) as f64 * RETURN_AMP;
            if i % 2 == 0 {
                mag
            } else {
                -mag
            }
        })
        .collect();
    let signals: Vec<f64> = match fixture {
        // Overfit: constant-long => P&L = market (zero-mean) => edge survives the null.
        FixtureKind::Violation => vec![1.0; SERIES_LEN],
        // Genuine: sign-aligned => permuting destroys the alignment => collapses to null.
        FixtureKind::Clean => returns
            .iter()
            .map(|&r| if r >= 0.0 { 1.0 } else { -1.0 })
            .collect(),
    };

    let input = TradingNullInput {
        strategy_path: "h27_strategy.py".to_string(),
        objective: OBJECTIVE.to_string(),
        data_window: WINDOW.to_string(),
        signals,
        returns,
        k_permutations: K_MIN_PERMUTATIONS,
        block_size: DEFAULT_BLOCK_SIZE,
        seed: 42,
    };
    let verdict = evaluate_trading_null(&ledger, ssb, input)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("trading_null: {e}")))?;
    // Block-class: the edge survived the null (or unprovable). Allow = collapsed-to-null.
    Ok(!matches!(verdict.kind, TradingNullVerdictKind::EdgeCollapsedToNull))
}

/// Rerun (H-23): the planted mutant is the nondeterministic digest pair (run #2's sharpe
/// output bytes differ) => Divergent. The clean fixture is byte-identical digests =>
/// Equal.
async fn run_rerun(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::rerun_equality::{
        verify_rerun_digest, MetricDigest, RerunInput, RerunVerdictKind, RunDigests,
    };

    let run_a = RunDigests {
        metrics: vec![
            MetricDigest::from_bytes("sharpe", b"sharpe=1.42;window=0..1;seed=42"),
            MetricDigest::from_bytes("max_drawdown", b"max_drawdown=-0.18;window=0..1;seed=42"),
        ],
    };
    let run_b = match fixture {
        FixtureKind::Violation => RunDigests {
            metrics: vec![
                // run #2 sharpe bytes differ (an unseeded RNG draw) => Divergent.
                MetricDigest::from_bytes("sharpe", b"sharpe=1.39;window=0..1;rng=0x9e3779b9"),
                MetricDigest::from_bytes("max_drawdown", b"max_drawdown=-0.18;window=0..1;seed=42"),
            ],
        },
        FixtureKind::Clean => RunDigests {
            metrics: vec![
                MetricDigest::from_bytes("sharpe", b"sharpe=1.42;window=0..1;seed=42"),
                MetricDigest::from_bytes("max_drawdown", b"max_drawdown=-0.18;window=0..1;seed=42"),
            ],
        },
    };

    let input = RerunInput {
        strategy_path: "h27_experiment.py".to_string(),
        run_a,
        run_b: Some(run_b),
        seed: Some(42),
    };
    let verdict = verify_rerun_digest(ssb, input)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("rerun: {e}")))?;
    // Block-class = Divergent. Allow = Equal.
    Ok(matches!(verdict.kind, RerunVerdictKind::Divergent))
}

/// Sim-real (H-24): the planted mutant is the golden-gap event log (the replay's fill
/// event differs) => ReplayGoldenMismatch. The clean fixture is a byte-identical replay
/// => Parity.
async fn run_sim_real(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::sim_real_parity::{
        verify_replay_golden, EventDigest, EventLog, SimRealInput, SimRealVerdictKind,
    };

    let golden = EventLog {
        events: vec![
            EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
            EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 187.34;seed=42"),
        ],
    };
    let replay = match fixture {
        FixtureKind::Violation => EventLog {
            events: vec![
                EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
                // the fill event differs (a different backtest fill model) => mismatch.
                EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 186.91;backtest_fill_model"),
            ],
        },
        FixtureKind::Clean => EventLog {
            events: vec![
                EventDigest::from_bytes("order:AAPL", b"order AAPL 100 @ market;seed=42"),
                EventDigest::from_bytes("fill:AAPL", b"fill AAPL 100 @ 187.34;seed=42"),
            ],
        },
    };

    let input = SimRealInput {
        strategy_path: "h27_strategy.py".to_string(),
        golden,
        replay: Some(replay),
        seed: 42,
    };
    let verdict = verify_replay_golden(ssb, input)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("sim_real: {e}")))?;
    // Block-class = ReplayGoldenMismatch. Allow = Parity.
    Ok(matches!(verdict.kind, SimRealVerdictKind::ReplayGoldenMismatch))
}

/// Result-manifest (H-25): the planted mutant is the tampered manifest (sharpe declared
/// 2.99 but the captured output says 1.85) => FabricationDetected. The clean fixture
/// declares the true values => ReDerived.
async fn run_result_manifest(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::result_manifest::{
        verify_result, ReDerivedInputs, ResultEntry, ResultManifest, ResultVerdictKind,
    };

    let captured = b"backtest complete\nsharpe=1.85\nmaxdd=0.07\n".to_vec();
    let captured_blake3 = {
        let full = blake3::hash(&captured).to_hex();
        full.chars().take(16).collect::<String>()
    };
    const DATA_HASH: &str = "data0000";
    const ENV_HASH: &str = "env00000";
    const COMMIT: &str = "commitabc";

    let entry = |name: &str, value: &str| ResultEntry {
        metric_name: name.to_string(),
        value: value.to_string(),
        captured_output_blake3: captured_blake3.clone(),
        parser_expr: format!("labeled-float:{name}="),
        data_hash: DATA_HASH.to_string(),
        code_commit: COMMIT.to_string(),
        env_lock_hash: ENV_HASH.to_string(),
        n_trials: 100,
        dsr_pvalue: Some(0.01),
        pbo: Some(0.12),
    };
    let rederived = ReDerivedInputs {
        captured_outputs: vec![
            ("sharpe".to_string(), captured.clone()),
            ("maxdd".to_string(), captured.clone()),
        ],
        data_bytes_hash: DATA_HASH.to_string(),
        env_bytes_hash: ENV_HASH.to_string(),
        code_commit_observed: COMMIT.to_string(),
    };
    let entries = match fixture {
        // Tampered: sharpe declared 2.99 but the capture says 1.85 => does not re-derive.
        FixtureKind::Violation => vec![entry("sharpe", "2.99"), entry("maxdd", "0.07")],
        FixtureKind::Clean => vec![entry("sharpe", "1.85"), entry("maxdd", "0.07")],
    };
    let manifest = ResultManifest {
        strategy_path: "h27_experiment.py".to_string(),
        run_id: "h27-result-manifest".to_string(),
        elai_seed: Some(42),
        entries,
    };
    let verdict = verify_result(ssb, manifest, Some(rederived))
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("result_manifest: {e}")))?;
    // Block-class = FabricationDetected. Allow = ReDerived.
    Ok(matches!(verdict.kind, ResultVerdictKind::FabricationDetected))
}

/// Shadow-exec (H-26): the planted mutant breaks the PnL-additivity accounting identity
/// by exactly 1 cent (declared total != sum of per-trade PnL) => InvariantViolated. The
/// clean fixture is the consistent run agreeing with its shadow => InvariantHeld.
async fn run_shadow_exec(
    ssb: &Arc<Ssb>,
    fixture: FixtureKind,
) -> Result<bool, GateSelfValidationError> {
    use crate::shadow_exec::{
        diff_shadow_impls, DerivedFloat, FinancialInvariantInputs, OptionStyle,
        PutCallParityInput, ShadowOutcome, ShadowResultSet, ShadowVerdictKind, TradeRecord,
    };

    let derived = vec![DerivedFloat {
        name: "option_price".to_string(),
        value: 12.345_678_9,
    }];
    let mut primary = ShadowResultSet {
        strategy_path: "h27_pnl_engine.rs".to_string(),
        run_id: "h27-shadow".to_string(),
        elai_seed: Some(42),
        trades: vec![
            TradeRecord {
                shares_in: 10,
                shares_out: 10,
                cash_cents: 100_00,
                position_qty: 10,
                price_cents: 50_00,
                equity_cents: 100_00 + 10 * 50_00,
                realized_pnl_cents: 1_50,
            },
            TradeRecord {
                shares_in: 5,
                shares_out: 5,
                cash_cents: 200_00,
                position_qty: 4,
                price_cents: 25_00,
                equity_cents: 200_00 + 4 * 25_00,
                realized_pnl_cents: 2_25,
            },
        ],
        declared_total_pnl_cents: 1_50 + 2_25,
        derived_floats: derived.clone(),
    };
    if fixture == FixtureKind::Violation {
        // Break PnL additivity by exactly 1 cent: declared total != sum of per-trade.
        primary.declared_total_pnl_cents += 1;
    }
    let shadow = ShadowOutcome::Produced(Box::new(ShadowResultSet {
        derived_floats: derived,
        ..primary.clone()
    }));

    // A parity-consistent European put-call-parity input (held in both fixtures).
    let spot: f64 = 100.0;
    let strike: f64 = 95.0;
    let rate: f64 = 0.02;
    let time: f64 = 0.5;
    let rhs = spot - strike * (-rate * time).exp();
    let put: f64 = 3.0;
    let call = put + rhs;
    let invariants = FinancialInvariantInputs {
        put_call_parity: vec![PutCallParityInput {
            call,
            put,
            spot,
            strike,
            rate,
            time,
            style: OptionStyle::European,
        }],
        numeric_stability: vec![],
    };

    let verdict = diff_shadow_impls(ssb, primary, shadow, invariants, 42)
        .await
        .map_err(|e| GateSelfValidationError::Ledger(format!("shadow_exec: {e}")))?;
    // Block-class = ShadowDiverged / InvariantViolated. Allow = InvariantHeld.
    Ok(!matches!(verdict.kind, ShadowVerdictKind::InvariantHeld))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::SsbConfig;
    use tempfile::TempDir;

    fn test_ssb(dir: &TempDir) -> (Arc<Ssb>, std::path::PathBuf) {
        let db_path = dir.path().join("ssb.db");
        let cfg = SsbConfig {
            db_path: db_path.clone(),
            pool_size: None,
            project_id: "h27-test".to_string(),
            session_id: "h27-test-session".to_string(),
        };
        let clock: Arc<dyn elai_core::clock::ClockProvider> =
            Arc::new(elai_core::clock::OsClock);
        let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb init"));
        (ssb, db_path)
    }

    #[test]
    fn mutation_score_is_killed_over_total() {
        assert_eq!(mutation_score(8, 8), 100.0);
        assert_eq!(mutation_score(7, 8), 87.5);
        assert_eq!(mutation_score(0, 0), 0.0); // never the vacuous 100.0 pass
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_registry_is_fail_closed() {
        let dir = TempDir::new().expect("tempdir");
        let (ssb, db_path) = test_ssb(&dir);
        let outcome = validate_gates_with(&ssb, &db_path, &[], false)
            .await
            .expect("driver");
        assert_eq!(outcome.kind, GateSelfValidationKind::EmptyRegistry);
        assert!(!outcome.is_pass());
        assert_eq!(outcome.gates_total, 0);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn injected_weakened_gate_surfaces_as_failed_to_block() {
        let dir = TempDir::new().expect("tempdir");
        let (ssb, db_path) = test_ssb(&dir);
        // A single real gate (lookahead) PLUS the injected weakened gate. The real gate
        // kills its mutant; the weakened gate does not => GateFailedToBlock.
        let registry = &[GateEntry { id: GateId::Lookahead, has_clean_fixture: true }];
        let outcome = validate_gates_with(&ssb, &db_path, registry, true)
            .await
            .expect("driver");
        assert_eq!(outcome.kind, GateSelfValidationKind::GateFailedToBlock);
        assert_eq!(outcome.escaped_gate, "weakened_demo_gate");
        assert!(!outcome.is_pass());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn lookahead_kills_its_own_mutant() {
        let dir = TempDir::new().expect("tempdir");
        let (ssb, _db) = test_ssb(&dir);
        // The lookahead gate must BLOCK its future-leak fixture (kill) and ALLOW its
        // clean fixture (no over-block) — the real evaluate fn is invoked.
        assert!(
            run_gate(&ssb, dir.path(), GateId::Lookahead, FixtureKind::Violation)
                .await
                .expect("violation run"),
            "lookahead must block its future-leak mutant"
        );
        assert!(
            !run_gate(&ssb, dir.path(), GateId::Lookahead, FixtureKind::Clean)
                .await
                .expect("clean run"),
            "lookahead must allow its causal clean fixture"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn deflation_blocks_absent_pbo_mutant() {
        let dir = TempDir::new().expect("tempdir");
        let (ssb, db_path) = test_ssb(&dir);
        assert!(
            run_gate(&ssb, &db_path, GateId::Deflation, FixtureKind::Violation)
                .await
                .expect("deflation violation"),
            "deflation must block its absent-PBO mutant (R1 fail-closed)"
        );
    }
}
