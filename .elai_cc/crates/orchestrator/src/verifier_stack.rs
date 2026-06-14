//! I-17 — tri-layer evaluation stack (the veto-supreme branch scorer).
//!
//! Composes the SHIPPED verification substrate into ONE veto stack the search
//! ladder (I-4, Wave-4) can rank branches by. The substrate is NOT re-implemented
//! here — every layer calls into the existing surface (GD-1 compose-not-restate):
//!
//!   L1 HARD VETO (supreme, ID-4): a compound deterministic gate =
//!       D-11 [`crate::checker_registry::CheckerRegistry::run_all_applicable`]
//!     + F-1 [`crate::vibegate::evaluate_vibegate`]
//!     + F-9 [`crate::vibesec::dep_writetime_gate::DepWriteTimeGate::analyze_collect`].
//!     ANY veto sets the branch score to a LITERAL `0.0` and returns EARLY —
//!     L2 and L3 NEVER see a vetoed branch (`l2_call_count == 0` is the
//!     off-agent observable proof). There is NO soft veto: a vetoed branch
//!     scores `0.0`, never `0.001` (a neural scorer resurrecting a vetoed
//!     branch is the exact failure mode ID-4 exists to prevent).
//!
//!   L2 CALIBRATED SCORER: runs ONLY on L1 survivors. Reads the I-23 unified
//!     conformal-calibration facade ([`crate::calibration::compute_tau`]); on a
//!     `ConformalError` (e.g. empty corpus) it degrades to
//!     [`crate::calibration::TAU_DEFAULT_FALLBACK`] rather than panicking.
//!
//!   L3 DISAGREEMENT ESCALATION: fires when L2 scores among survivors disagree
//!     past [`L3_VARIANCE_THRESHOLD`] (a `[DESIGN]` constant, measurement-before-
//!     gate per ID-5). Routes to the D-13 cross-family adversary
//!     ([`crate::adversary::Adversary::generate_counter`]) for a concrete
//!     counter-example, not just a score.
//!
//! FLAKE QUARANTINE: an oracle that flips across seeded reruns is ledgered out
//! of voting (the exclusion is a HARD gate, never a soft weight). The ledger key
//! is `blake3(oracle_id ++ seed_range)` — blake3 ONLY (never sha2/blake2).
//!
//! Each branch verdict emits exactly one [`crate::ssb::SsbActionType::TriLayerVerdict`]
//! row. Wire-dark by default ([`tri_layer_enabled`] reads `ELAI_TRI_LAYER_ENABLE`);
//! the production consumer is the I-4 search ladder (Wave-4). See
//! `docs/thefinalglance/I-plan/I-17_tri_layer_evaluation_stack.md` and
//! `workflow/reasoning/I-17-decisions.md`.

use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;

use elai_core::clock::{ClockProvider, OsClock};

use crate::adversary::{AdversaryConfig, AdversaryError, DynAdversary};
use crate::calibration::{compute_tau, CalibrationCorpus, DEFAULT_EPSILON, TAU_DEFAULT_FALLBACK};
use crate::canary_injector::{
    canary_injector_enabled, CanaryInjector, CanaryInjectorConfig, SCORER_ID_CONFORMAL_TAU,
};
use crate::checker_registry::{CheckerRegistry, CheckerResult};
use crate::claim::{Claim, ClaimType};
use crate::ssb::{GateId, Ssb, SsbPayload, TriLayerVerdictKind, TriLayerVerdictPayload};
use crate::vibesec::dep_writetime_gate::DepWriteTimeGate;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — HARD RULE)
// ---------------------------------------------------------------------------

/// Env var gating the tri-layer stack's SSB emission + (Wave-4) the I-4 ladder
/// dispatch hook. Default-OFF: registered in `wire-dark-allowlist.toml` (R9)
/// with promotion plan I-4. Recognised truthy values: `"1"`, `"true"`, `"yes"`,
/// `"on"` (case-insensitive). The literal MUST match the
/// `tri_layer_env_gate_matches` contract assertion.
pub const TRI_LAYER_ENABLE_ENV: &str = "ELAI_TRI_LAYER_ENABLE";

/// `[DESIGN]` L3 escalation threshold: the maximum variance (population variance
/// over the survivors' L2 scores in `[0.0, 1.0]`) tolerated before the D-13
/// cross-family adversary is routed for a counter-example.
///
/// Conservative default `0.0625` (a standard-deviation of `0.25` on the score
/// scale, squared). Labelled `[DESIGN]` per ID-5 (measurement-before-gate): this
/// is NOT a measured gate threshold yet — it must be calibrated against real
/// runs before it is promoted from a routing hint to a hard gate. NEVER inline a
/// bare literal at the comparison site.
pub const L3_VARIANCE_THRESHOLD: f64 = 0.0625;

/// The literal score a vetoed branch receives. ID-4: hard veto is binary; there
/// is NO soft-veto penalty weight. A named constant so the veto-supremacy
/// invariant is greppable and a future edit cannot silently introduce a
/// "strong negative weight" (the broken-arm contract fixture catches that).
pub const VETO_SCORE: f64 = 0.0;

/// Maximum branch `source_snippet` length forwarded into the L1 axes + the
/// synthesised [`Claim`] text. Bounds SSB row + claim sizes predictably.
const MAX_BRANCH_SOURCE_CHARS: usize = 8_192;

// ---------------------------------------------------------------------------
// Public DTOs (the I-4 API-stability surface)
// ---------------------------------------------------------------------------

/// A candidate branch to score. Thin data-transfer object (FORK-C): I-17 defines
/// it so the production wiring does not block on I-4 (Wave-4). The Wave-4 search
/// ladder populates `source_snippet` with the candidate's generated code.
///
/// STABILITY CONTRACT (frozen for I-4): `branch_id` + at least one content field
/// (`source_snippet`). I-4 calls [`TriLayerStack::run_verdict`] with a `&Branch`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
    /// Stable identifier for this branch (the SSB row + VerdictRow key).
    pub branch_id: String,
    /// Candidate source under evaluation (drives the L1 dep-gate + VibeGate
    /// axes and the synthesised checker claim). May be empty for a
    /// no-source probe branch.
    pub source_snippet: String,
}

impl Branch {
    /// Construct a branch from an id + source. The source is truncated to
    /// [`MAX_BRANCH_SOURCE_CHARS`] so downstream claim/SSB sizes stay bounded.
    #[must_use]
    pub fn new(branch_id: impl Into<String>, source_snippet: impl Into<String>) -> Self {
        let mut source_snippet = source_snippet.into();
        if source_snippet.len() > MAX_BRANCH_SOURCE_CHARS {
            source_snippet.truncate(MAX_BRANCH_SOURCE_CHARS);
        }
        Self {
            branch_id: branch_id.into(),
            source_snippet,
        }
    }
}

/// The verdict for one branch. Returned by [`TriLayerStack::run_verdict`] and
/// mirrored 1:1 onto the emitted [`TriLayerVerdictPayload`] SSB row.
///
/// STABILITY CONTRACT (frozen for I-4): `branch_id`, `verdict`, `score`, `gate`.
/// `l2_call_count` is NOT part of the I-4 contract (it is the off-agent
/// veto-supremacy proof the auditor reads) but is retained for observability.
#[derive(Debug, Clone, PartialEq)]
pub struct VerdictRow {
    /// Which branch this verdict scores.
    pub branch_id: String,
    /// The tri-layer arm that produced the verdict.
    pub verdict: TriLayerVerdictKind,
    /// Final score in `[0.0, 1.0]`. Literal [`VETO_SCORE`] for `HardVeto`.
    pub score: f64,
    /// The gate that vetoed (`Some` only for `HardVeto`).
    pub gate: Option<GateId>,
    /// Number of L2 calibrated-scorer invocations for this branch. `0` for a
    /// vetoed branch — the observable proof L2 never saw it (ID-4).
    pub l2_call_count: u32,
}

impl VerdictRow {
    /// True iff this branch was hard-vetoed at L1 (score is [`VETO_SCORE`] and
    /// neither L2 nor L3 ran). Read-side VerdictRow predicate the I-4 (Wave-4)
    /// SearchLadder ranker consumes to drop hard-vetoed branches from selection.
    /// WIRED (I-4): `crate::search::SearchLadder::is_pruned` calls it per branch
    /// via `run_best_of_n`, reached from `task_runner::dispatch_search_ladder`.
    #[must_use]
    pub fn is_vetoed(&self) -> bool {
        matches!(self.verdict, TriLayerVerdictKind::HardVeto)
    }
}

/// Errors emitted by the tri-layer stack. L1/L2 are infallible at the stack
/// boundary (gate errors degrade to a typed verdict, not a stack error); only
/// the L3 adversary route can surface a hard error worth propagating.
#[derive(Debug, thiserror::Error)]
pub enum VerifierStackError {
    /// The L3 cross-family adversary route failed (runtime unavailable / schema
    /// parse failure). The caller decides retry vs. accept-without-counter.
    #[error("l3 adversary route failed: {0}")]
    Adversary(#[from] AdversaryError),
}

// ---------------------------------------------------------------------------
// FlakeQuarantineLedger
// ---------------------------------------------------------------------------

/// SSB-backed quarantine ledger for flaky oracles. An oracle whose verdict flips
/// across seeded reruns is ledgered out of voting until rehabilitated; the
/// exclusion is a HARD gate (the quarantine check is consulted before every
/// oracle read), never a soft weight.
///
/// The key is `blake3(oracle_id ++ ":" ++ lo ++ ":" ++ hi)` over a canonical byte
/// encoding (blake3 ONLY — never sha2/blake2 per HARD RULE). The write path emits
/// a [`TriLayerVerdictKind::FlakeQuarantined`] SSB row so the quarantine survives
/// process restarts (a re-run reads the row back via the SSB viewer surface).
pub struct FlakeQuarantineLedger {
    ssb: Arc<Ssb>,
    /// In-session set of quarantine keys already written this process lifetime.
    /// Cross-restart durability is via the emitted SSB rows; this set is the
    /// fast in-process membership check.
    quarantined: std::collections::HashSet<String>,
}

impl FlakeQuarantineLedger {
    /// Construct a ledger backed by the shared SSB writer.
    #[must_use]
    pub fn new(ssb: Arc<Ssb>) -> Self {
        Self {
            ssb,
            quarantined: std::collections::HashSet::new(),
        }
    }

    /// Canonical blake3 quarantine key for an `(oracle_id, seed_range)` pair.
    /// blake3 ONLY (HARD RULE); the byte encoding is length-stable so distinct
    /// inputs never collide on a shared prefix.
    #[must_use]
    pub fn quarantine_key(oracle_id: &str, seed_range: (u64, u64)) -> String {
        let mut buf = Vec::with_capacity(oracle_id.len() + 17);
        buf.extend_from_slice(oracle_id.as_bytes());
        buf.push(b':');
        buf.extend_from_slice(&seed_range.0.to_le_bytes());
        buf.extend_from_slice(&seed_range.1.to_le_bytes());
        blake3::hash(&buf).to_hex().to_string()
    }

    /// True iff the `(oracle_id, seed_range)` is currently quarantined. Read-side
    /// check the I-4 (Wave-4) SearchLadder vote consults before ranking a branch.
    /// WIRED (I-4): `crate::search::SearchLadder::is_pruned` calls it per branch
    /// via `run_best_of_n`, reached from `task_runner::dispatch_search_ladder`.
    #[must_use]
    pub fn is_quarantined(&self, oracle_id: &str, seed_range: (u64, u64)) -> bool {
        let key = Self::quarantine_key(oracle_id, seed_range);
        self.quarantined.contains(&key)
    }

    /// Ledger an `(oracle_id, seed_range)` as quarantined and emit the
    /// `FlakeQuarantined` SSB row. Idempotent: re-quarantining an already-known
    /// key still emits a row (the row is the durable signal) but the in-process
    /// set membership is unchanged.
    pub async fn quarantine(&mut self, oracle_id: &str, seed_range: (u64, u64)) {
        let key = Self::quarantine_key(oracle_id, seed_range);
        self.quarantined.insert(key.clone());
        let payload = SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
            // The quarantine key doubles as the branch_id for this row family so
            // the dashboard can correlate a quarantine to its oracle deterministically.
            branch_id: key,
            kind: TriLayerVerdictKind::FlakeQuarantined,
            gate: None,
            l2_call_count: 0,
            score: VETO_SCORE,
            ts_ms: now_ms(),
        });
        emit_row(&self.ssb, payload).await;
    }
}

// ---------------------------------------------------------------------------
// TriLayerStack
// ---------------------------------------------------------------------------

/// The veto-supreme tri-layer branch scorer. Constructed once at boot (the L1
/// substrate handles are cloned/shared); [`run_verdict`](Self::run_verdict) is
/// called per branch by the I-4 search ladder (Wave-4) and the task-runner
/// per-branch audit-dispatch hook.
///
/// The stack OWNS no verification logic — it composes the SHIPPED surfaces
/// (GD-1). The adversary is a trait object so the production wiring injects a
/// `CloudAdversary` while tests inject a `MockAdversary`.
pub struct TriLayerStack {
    /// D-11 checker registry (L1 compile/test/checker axis). Shared.
    checker_registry: Arc<CheckerRegistry>,
    /// F-9 write-time dependency gate (L1 dep axis). Shared.
    dep_gate: Arc<DepWriteTimeGate>,
    /// I-23 conformal calibration corpus (L2 uncertainty source). `None`
    /// degrades L2 to [`TAU_DEFAULT_FALLBACK`].
    calibration_corpus: Option<Arc<CalibrationCorpus>>,
    /// D-13 cross-family adversary (L3 escalation target). `None` skips L3
    /// (a disagreement is reported as the L2 verdict without a counter). Stored
    /// as [`DynAdversary`] (object-safe boxed-future shape) because [`Adversary`]
    /// uses native async-fn-in-trait and is not dyn-compatible; callers wrap a
    /// concrete adversary via [`crate::adversary::AdversaryErased::new`].
    adversary: Option<Arc<dyn DynAdversary>>,
    /// D-13 adversary config (model family + severity floor).
    adversary_config: AdversaryConfig,
    /// Shared SSB writer for the per-branch verdict rows.
    ssb: Arc<Ssb>,
    /// I-24 isomorphic canary injector. `None` when
    /// `ELAI_CANARY_INJECTOR_ENABLE` is not set or the fixture file is absent.
    /// Wrapped in a `TokioMutex` so `run_verdict` (which takes `&self`) can
    /// mutate the injector's consecutive-detection counter.
    canary_injector: TokioMutex<Option<CanaryInjector>>,
}

impl TriLayerStack {
    /// Construct the stack from the shipped substrate handles. `adversary` and
    /// `calibration_corpus` are optional (L3 / L2 degrade gracefully when
    /// absent); the checker registry, dep-gate, and SSB writer are mandatory.
    /// Construct the stack from the shipped substrate handles. `adversary` and
    /// `calibration_corpus` are optional (L3 / L2 degrade gracefully when
    /// absent); the checker registry, dep-gate, and SSB writer are mandatory.
    ///
    /// `fixture_dir` is the directory that contains `canary_pairs.json` (the
    /// `fixtures/` subdirectory of the project root). When `None` or when the
    /// file is absent, the I-24 canary injector is disabled for this stack.
    /// The injector is also disabled when `ELAI_CANARY_INJECTOR_ENABLE` is not
    /// set (default-OFF, R9 gate — see `ELAI_CANARY_INJECTOR_ENABLE_ENV`).
    #[must_use]
    pub fn new(
        checker_registry: Arc<CheckerRegistry>,
        dep_gate: Arc<DepWriteTimeGate>,
        calibration_corpus: Option<Arc<CalibrationCorpus>>,
        adversary: Option<Arc<dyn DynAdversary>>,
        adversary_config: AdversaryConfig,
        ssb: Arc<Ssb>,
    ) -> Self {
        Self::new_with_fixture_dir(
            checker_registry,
            dep_gate,
            calibration_corpus,
            adversary,
            adversary_config,
            ssb,
            None::<&std::path::Path>,
        )
    }

    /// Extended constructor that accepts a fixture directory path for the I-24
    /// canary injector. Prefer [`Self::new`] in production code; this variant
    /// is used by the task-runner bootstrap (which knows the project root) and
    /// by integration tests that supply a temp-dir fixture.
    pub fn new_with_fixture_dir(
        checker_registry: Arc<CheckerRegistry>,
        dep_gate: Arc<DepWriteTimeGate>,
        calibration_corpus: Option<Arc<CalibrationCorpus>>,
        adversary: Option<Arc<dyn DynAdversary>>,
        adversary_config: AdversaryConfig,
        ssb: Arc<Ssb>,
        project_root: Option<&std::path::Path>,
    ) -> Self {
        // I-24: build the canary injector when the gate is enabled AND a
        // project_root is supplied. Falls back gracefully to None on any error.
        let canary = if canary_injector_enabled() {
            project_root.and_then(|root| {
                let fixture_path = root.join("fixtures").join("canary_pairs.json");
                let seed = elai_seed_from_env();
                let config = CanaryInjectorConfig::load(root, 0.05, seed);
                match CanaryInjector::new(&fixture_path, config, Arc::clone(&ssb)) {
                    Ok(inj) => inj,
                    Err(e) => {
                        tracing::warn!(
                            canary.err = %e,
                            "I-24 canary injector failed to load fixture; canary disabled"
                        );
                        None
                    }
                }
            })
        } else {
            None
        };
        Self {
            checker_registry,
            dep_gate,
            calibration_corpus,
            adversary,
            adversary_config,
            ssb,
            canary_injector: TokioMutex::new(canary),
        }
    }

    /// Score one branch through the tri-layer stack and emit its verdict row.
    ///
    /// CONTRACT (ID-4 veto supremacy): if L1 vetoes, the returned [`VerdictRow`]
    /// has `verdict == HardVeto`, `score == VETO_SCORE` (literal 0.0),
    /// `l2_call_count == 0`, and `gate == Some(..)` — L2 and L3 are NEVER
    /// invoked. The early return is the structural enforcement; there is no code
    /// path where a vetoed branch reaches L2.
    pub async fn run_verdict(&self, branch: &Branch) -> Result<VerdictRow, VerifierStackError> {
        // ---- L1: compound hard veto (supreme). Any axis veto -> early return.
        if let Some(gate) = self.run_l1_veto(branch).await {
            let row = VerdictRow {
                branch_id: branch.branch_id.clone(),
                verdict: TriLayerVerdictKind::HardVeto,
                score: VETO_SCORE,
                gate: Some(gate),
                // ID-4 OBSERVABLE PROOF: a vetoed branch never reaches L2.
                l2_call_count: 0,
            };
            self.emit_verdict(&row).await;
            return Ok(row);
        }

        // ---- L2: calibrated scorer (survivors only). One invocation per branch.
        let score = self.run_l2_score();
        let l2_call_count: u32 = 1;

        // ---- I-24 canary injection (L2 anti-gaming, fire-and-forget).
        // The injector wraps the same run_l2_score closure so it uses the
        // IDENTICAL scorer path (GD-1). The wire-dark gate (ELAI_CANARY_INJECTOR_ENABLE)
        // is checked inside maybe_inject; no-op when the gate is off or the
        // injector is absent.
        //
        // WIRE-DARK[I-24]: the live dual-score through PpmClient is PENDING until
        // D-026 wires PpmClient into run_l2_score. This call validates SSB plumbing,
        // threshold logic, and neutralize state machine against the conformal-tau stub.
        {
            let mut guard = self.canary_injector.lock().await;
            if let Some(ref mut inj) = *guard {
                let corpus_clone = self.calibration_corpus.clone();
                inj.maybe_inject(
                    |_source| {
                        // Scorer closure: run the same conformal-tau path.
                        // The `_source` parameter is provided for future D-026 wiring
                        // where the neural scorer uses the branch source.
                        match corpus_clone.as_ref() {
                            Some(c) => compute_tau(c, DEFAULT_EPSILON)
                                .unwrap_or(TAU_DEFAULT_FALLBACK)
                                .clamp(0.0, 1.0),
                            None => TAU_DEFAULT_FALLBACK,
                        }
                    },
                    SCORER_ID_CONFORMAL_TAU,
                    "",      // agent_id: not tracked at stack level
                    0_i64,   // superstep: not tracked at stack level
                )
                .await;
            }
        }

        // ---- L3: disagreement escalation. A single-branch run cannot disagree;
        // the variance gate is evaluated by the multi-branch caller via
        // [`Self::needs_l3_escalation`]. For the per-branch path we surface the
        // L2 verdict directly (ScoredSurvivor) — the caller escalates the SET.
        let row = VerdictRow {
            branch_id: branch.branch_id.clone(),
            verdict: TriLayerVerdictKind::ScoredSurvivor,
            score,
            gate: None,
            l2_call_count,
        };
        self.emit_verdict(&row).await;
        Ok(row)
    }

    /// L3 ENTRY: given the L2 scores of a survivor set, decide whether the
    /// disagreement exceeds [`L3_VARIANCE_THRESHOLD`] and, if so, route the D-13
    /// adversary for a counter-example and emit a `DisagreementEscalated` row.
    ///
    /// Returns `Ok(true)` when escalation fired (a counter-example was produced
    /// or the adversary was absent but the variance gate tripped), `Ok(false)`
    /// when the survivors agreed closely enough to skip L3. Separated from
    /// [`run_verdict`](Self::run_verdict) because disagreement is a property of
    /// the SET, not a single branch (the per-branch verdict cannot see siblings).
    /// The task-runner production hook scores ONE candidate branch, which cannot
    /// disagree (needs >=2 survivors); the real caller is the I-4 (Wave-4)
    /// SearchLadder ranking a survivor SET. Allowlist ELAI_TRI_LAYER_ENABLE (I-4).
    /// WIRED (I-4): `crate::search::SearchLadder::finish` calls it over the ranked
    /// survivor set via `run_best_of_n`, reached from
    /// `task_runner::dispatch_search_ladder`.
    pub async fn escalate_on_disagreement(
        &self,
        survivors: &[VerdictRow],
    ) -> Result<bool, VerifierStackError> {
        if !Self::needs_l3_escalation(survivors) {
            return Ok(false);
        }
        // Route the cross-family adversary for a concrete counter-example on the
        // highest-scoring survivor (the one most at risk of a false positive).
        let target = survivors
            .iter()
            .max_by(|a, b| a.score.total_cmp(&b.score));
        let counter_id = match (self.adversary.as_ref(), target) {
            (Some(adv), Some(target)) => {
                let claim = synth_claim(&target.branch_id, "");
                let counter = adv
                    .generate_counter_dyn(&claim, &self.adversary_config)
                    .await?;
                Some(counter.counter_id)
            }
            // No adversary plumbed: the variance gate still tripped, so we still
            // emit the escalation row (observable), just without a counter id.
            _ => None,
        };
        let branch_id = target
            .map(|t| t.branch_id.clone())
            .unwrap_or_else(|| "disagreement-set".to_owned());
        let row = VerdictRow {
            branch_id,
            verdict: TriLayerVerdictKind::DisagreementEscalated,
            // The escalation row carries the disputed survivor's score so the
            // dashboard can show what was contested; the counter (if any) is the
            // durable artifact, surfaced via the adversary_generated row family.
            score: target.map(|t| t.score).unwrap_or(VETO_SCORE),
            gate: None,
            // L3 followed >=1 L2 score per disputed survivor; record the set size
            // as the L2 invocation count for this escalation row.
            l2_call_count: survivors.len().try_into().unwrap_or(u32::MAX),
        };
        self.emit_verdict(&row).await;
        let _ = counter_id; // counter is emitted by the adversary path itself
        Ok(true)
    }

    /// Pure decision: do the survivors' L2 scores disagree past the `[DESIGN]`
    /// variance threshold? Population variance over the scores; `< 2` survivors
    /// trivially agree.
    #[must_use]
    pub fn needs_l3_escalation(survivors: &[VerdictRow]) -> bool {
        if survivors.len() < 2 {
            return false;
        }
        let n = survivors.len() as f64;
        let mean = survivors.iter().map(|r| r.score).sum::<f64>() / n;
        let variance = survivors
            .iter()
            .map(|r| {
                let d = r.score - mean;
                d * d
            })
            .sum::<f64>()
            / n;
        variance > L3_VARIANCE_THRESHOLD
    }

    // ---- L1 internals -----------------------------------------------------

    /// Run the L1 compound veto. Returns `Some(GateId)` of the FIRST axis to
    /// veto (compile/checker -> dep-gate -> vibe), or `None` if every axis
    /// passed. Order is deterministic so the emitted `gate` is stable across runs.
    async fn run_l1_veto(&self, branch: &Branch) -> Option<GateId> {
        // Axis 1: D-11 checker registry (compile/test/regex/numeric checkers).
        // Any non-pass, non-inapplicable result is a veto. A CompileRust failure
        // is attributed to L1Compile; any other checker Fail to L1Checker.
        let claim = synth_claim(&branch.branch_id, &branch.source_snippet);
        for (_id, kind, result) in self.checker_registry.run_all_applicable(&claim).await {
            match result {
                CheckerResult::Fail { .. } | CheckerResult::Error { .. } => {
                    return Some(checker_kind_to_gate(kind));
                }
                CheckerResult::Pass | CheckerResult::Inapplicable { .. } => {}
            }
        }

        // Axis 2: F-9 write-time dependency gate (slopsquat / stale dep). A Block
        // finding is a veto. analyze_collect is sync (no SSB emission — we own
        // the row family) and returns (findings, blocked_records).
        let (dep_findings, _blocked) = self
            .dep_gate
            .analyze_collect(&branch.source_snippet, "branch.rs");
        if dep_findings.iter().any(|f| f.is_block()) {
            return Some(GateId::L1DepGate);
        }

        // Axis 3: F-1 VibeGate (secret axis + code-vuln axis). A Block finding is
        // a veto. evaluate_vibegate is sync.
        let vibe_findings =
            crate::vibegate::evaluate_vibegate("Write", &branch.source_snippet, "branch.rs");
        if vibe_findings.iter().any(|f| f.is_block()) {
            return Some(GateId::L1VibeGate);
        }

        None
    }

    // ---- L2 internals -----------------------------------------------------

    /// Run the L2 calibrated scorer on an L1 survivor. Reads the I-23 conformal
    /// facade ([`compute_tau`]); degrades to [`TAU_DEFAULT_FALLBACK`] on an empty
    /// corpus or any [`crate::calibration::ConformalError`] rather than panicking.
    /// `tau` (the conformal quantile) is the survivor's calibrated score.
    fn run_l2_score(&self) -> f64 {
        let tau = match self.calibration_corpus.as_ref() {
            Some(corpus) => compute_tau(corpus, DEFAULT_EPSILON).unwrap_or(TAU_DEFAULT_FALLBACK),
            None => TAU_DEFAULT_FALLBACK,
        };
        tau.clamp(0.0, 1.0)
    }

    // ---- SSB emission -----------------------------------------------------

    /// Emit one [`TriLayerVerdictPayload`] row for a verdict (best-effort).
    async fn emit_verdict(&self, row: &VerdictRow) {
        let payload = SsbPayload::TriLayerVerdict(TriLayerVerdictPayload {
            branch_id: row.branch_id.clone(),
            kind: row.verdict,
            gate: row.gate,
            l2_call_count: row.l2_call_count,
            score: row.score,
            ts_ms: now_ms(),
        });
        emit_row(&self.ssb, payload).await;
    }
}

// ---------------------------------------------------------------------------
// Module helpers
// ---------------------------------------------------------------------------

/// Returns `true` when [`TRI_LAYER_ENABLE_ENV`] is set to a recognised truthy
/// literal (`"1"` / `"true"` / `"yes"` / `"on"`, case-insensitive). Default-OFF
/// (wire-dark; promotion plan I-4). Mirrors the D-N runtime-config gate idiom.
#[must_use]
pub fn tri_layer_enabled() -> bool {
    match std::env::var(TRI_LAYER_ENABLE_ENV) {
        Ok(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// Map a D-11 [`crate::checker_registry::CheckerKind`] veto to its [`GateId`].
/// A CompileRust checker maps to `L1Compile`; every other checker to `L1Checker`.
fn checker_kind_to_gate(kind: crate::checker_registry::CheckerKind) -> GateId {
    match kind {
        crate::checker_registry::CheckerKind::CompileRust => GateId::L1Compile,
        _ => GateId::L1Checker,
    }
}

/// Synthesise the [`Claim`] the L1 checker axis evaluates. The branch source is
/// the claim text (the registry's per-checker `applies_to` decides which
/// checkers fire); `ClaimType::Procedural` matches the task-completion shape used
/// by the D-11 task hook. Evidence is empty — the checkers are the ground truth.
fn synth_claim(branch_id: &str, source: &str) -> Claim {
    Claim {
        claim_id: format!("branch-{branch_id}"),
        text: source.to_owned(),
        claim_type: ClaimType::Procedural,
        evidence: Vec::new(),
        ts_ms: 0,
    }
}

/// Agent id stamped on every `TriLayerVerdict` SSB row emitted by this module.
const VERIFIER_STACK_AGENT_ID: &str = "verifier_stack";

/// Current wall-clock ms via the canonical [`OsClock`] idiom (saturating to
/// `i64::MAX` past year 292278994, matching the rest of the orchestrator's
/// timestamp helpers). The injected-clock `SeededClock` path is reserved for
/// the determinism harness; the per-branch verdict ts is advisory.
fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

/// Best-effort SSB row emission: a write failure is logged at `warn` and does
/// NOT fail the verdict (the verdict is the load-bearing return value; the SSB
/// row is the observability signal). Mirrors the D-N hook emit posture.
async fn emit_row(ssb: &Arc<Ssb>, payload: SsbPayload) {
    if let Err(e) = ssb
        .write_payload(VERIFIER_STACK_AGENT_ID, 0, payload, Vec::new())
        .await
    {
        tracing::warn!(error = %e, "verifier_stack: TriLayerVerdict SSB emit failed (verdict unaffected)");
    }
}

/// Parse `ELAI_SEED` from the environment as a `u64`. Returns `0` when the
/// var is absent or non-numeric. Used by the I-24 canary injector for GD-2
/// (deterministic pair selection + injection probability under the same seed).
fn elai_seed_from_env() -> u64 {
    std::env::var("ELAI_SEED")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adversary::MockAdversary;
    use crate::checker_registry::CheckerKind;

    fn survivor(id: &str, score: f64) -> VerdictRow {
        VerdictRow {
            branch_id: id.into(),
            verdict: TriLayerVerdictKind::ScoredSurvivor,
            score,
            gate: None,
            l2_call_count: 1,
        }
    }

    #[test]
    fn tri_layer_disabled_by_default() {
        std::env::remove_var(TRI_LAYER_ENABLE_ENV);
        assert!(!tri_layer_enabled());
    }

    #[test]
    fn veto_score_is_literal_zero() {
        // ID-4: a hard veto is binary, never a soft penalty weight.
        assert_eq!(VETO_SCORE, 0.0);
    }

    #[test]
    fn quarantine_key_is_blake3_and_input_sensitive() {
        let a = FlakeQuarantineLedger::quarantine_key("oracle-x", (0, 100));
        let b = FlakeQuarantineLedger::quarantine_key("oracle-x", (0, 100));
        let c = FlakeQuarantineLedger::quarantine_key("oracle-x", (0, 101));
        let d = FlakeQuarantineLedger::quarantine_key("oracle-y", (0, 100));
        assert_eq!(a, b, "same input -> same key (deterministic)");
        assert_ne!(a, c, "different seed range -> different key");
        assert_ne!(a, d, "different oracle id -> different key");
        // blake3 hex is 64 chars.
        assert_eq!(a.len(), 64);
    }

    #[test]
    fn checker_compile_fail_maps_to_l1_compile() {
        assert_eq!(checker_kind_to_gate(CheckerKind::CompileRust), GateId::L1Compile);
        assert_eq!(checker_kind_to_gate(CheckerKind::RegexMatch), GateId::L1Checker);
    }

    #[test]
    fn agreeing_survivors_skip_l3() {
        let set = vec![survivor("a", 0.70), survivor("b", 0.72), survivor("c", 0.71)];
        assert!(
            !TriLayerStack::needs_l3_escalation(&set),
            "tight cluster (var << threshold) must NOT escalate"
        );
    }

    #[test]
    fn diverging_survivors_trigger_l3() {
        let set = vec![survivor("a", 0.05), survivor("b", 0.95)];
        assert!(
            TriLayerStack::needs_l3_escalation(&set),
            "0.05 vs 0.95 (var = 0.2025 >> 0.0625) must escalate"
        );
    }

    #[test]
    fn single_survivor_never_escalates() {
        let set = vec![survivor("a", 0.5)];
        assert!(!TriLayerStack::needs_l3_escalation(&set));
    }

    // The MockAdversary is reachable for the contract test's L3 arm; assert it
    // satisfies the Adversary trait object bound the stack stores.
    #[test]
    fn mock_adversary_is_trait_object_compatible() {
        let counter = crate::adversary::CounterEvidenceSchema {
            counter_id: "c1".into(),
            original_claim_id: "branch-a".into(),
            counter_text: "counter".into(),
            counter_evidence: Vec::new(),
            severity: 0.9,
        };
        let _adv: Arc<dyn DynAdversary> = Arc::new(crate::adversary::AdversaryErased::new(
            MockAdversary::with_counter(crate::adversary::ModelFamily::Anthropic, counter),
        ));
    }
}
