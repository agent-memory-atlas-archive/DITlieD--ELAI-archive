//! I-7: cascading escalation waterfall — local-first, condense-on-fail, cloud route.
//!
//! IC-1 (binding correction, see docs/thefinalglance/I-plan/I-7_*.md): the original
//! Batch-API speculative-racing design is DEAD. The Anthropic Message Batches lane
//! runs 10-45 min with a 24h SLA and cancels do NOT refund queued work, so it can
//! never participate in an interactive or search-loop path. This module is a STRICT
//! WATERFALL: fan out K cheap local branches, run the I-17 tri-layer deterministic
//! check on each, and ONLY when every local branch fails do we condense the failure
//! set into an I-11 typed [`BeliefState`] and escalate SYNCHRONOUSLY to the cloud
//! routing seam ([`CloudModeRouter::route`]). A task the local lane solves never
//! leaves the machine (E-39 cost gate composes: a LocalWin emits no cloud row).
//!
//! IC-1 STRUCTURAL GUARD (load-bearing): this module has ZERO import path to the
//! H-5 batch client (`anthropic_batch` / `MessageBatch` / `batch_client`). The
//! `BatchRouted` Kind discriminator on the SSB row exists for journal completeness
//! on the OFFLINE H-5 lane only; the live waterfall NEVER emits it and NEVER
//! constructs a batch submission. The `i7_escalation_cascade_contract` asserts the
//! absence structurally (a grep over this file's source == 0 batch-client tokens).
//!
//! COMPOSES (GD-1, never re-implemented):
//!   * I-4 [`SearchLadder::run_best_of_n`] for the K-branch fan-out + per-branch
//!     I-17 tri-layer verdict + veto/quarantine pruning + best-of-N ranking. The
//!     ladder's `winner == None` (every candidate pruned / BudgetCollapsed) IS the
//!     "all local branches failed" signal that triggers condense + escalate.
//!   * I-11 [`BeliefState::new`] + [`BeliefState::emit_condensed_receipt`] for the
//!     typed <=2K condensate (NO raw history on the cloud path — stub_ban / ID-3).
//!   * H-7 [`CloudModeRouter::route`] for the synchronous cloud escalation DECISION
//!     (the buildable path returns a deterministic routing decision over a fixture;
//!     the actual billed cloud architect inference is D-026 PENDING, never faked).
//!
//! BUILDABLE vs PENDING (D-026 posture):
//!   BUILDABLE (this module): branch fan-out, tri-layer check, all-fail detection,
//!     BeliefState condensation + encode + receipt, route DECISION, EscalationCascade
//!     rows. All CPU-only / deterministic-fixture.
//!   PENDING (never faked): the live billed cloud-architect model CALL that turns the
//!     condensed BeliefState into the next task-runner candidate, and the H-5 Batch
//!     API offline end-to-end. The waterfall is fully testable against the router
//!     DECISION without a live call.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::belief_state::{BeliefState, TriedApproach};
use crate::cloud_mode::{
    CloudConfig, CloudCredentials, CloudModeError, CloudModeRouter, ExecMode, RoutedProvider,
};
use crate::dispatcher::BackendMode;
use crate::search::{BranchInput, SearchLadder, SearchLadderError};
use crate::ssb::{
    EscalationCascadeKind, EscalationCascadePayload, Ssb, SsbError, SsbPayload,
};

/// Re-export of the I-7 env-gate literal (canonical definition lives next to the
/// SSB payload in `ssb::payloads`). The `task_runner::maybe_run_escalation_waterfall`
/// seam and the contract test both read it through this module (mirrors the I-4
/// `search::ELAI_SEARCH_LADDER_ENABLE_ENV` re-export idiom).
pub use crate::ssb::ELAI_ESCALATION_WATERFALL_ENABLE_ENV;

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — HARD RULE)
// ---------------------------------------------------------------------------

/// Agent id stamped on every `EscalationCascade` SSB row this module emits.
const ESCALATION_WATERFALL_AGENT_ID: &str = "escalation_waterfall";

/// `[DESIGN]` upper bound (bytes) the condensed [`BeliefState`] wire payload must
/// stay under on the cloud escalation hop. The I-11 condensate replaces raw history
/// precisely so the cloud call receives a compact typed summary (~2K tokens). The
/// `Escalated` row records the measured `payload_bytes`; the contract asserts it is
/// under this ceiling. Labelled `[DESIGN]` (ID-5): a budget target, not a measured
/// threshold. Sized at ~2K tokens * ~4 bytes/token, generously rounded.
pub const CONDENSED_PAYLOAD_MAX_BYTES: usize = 8_192;

// ---------------------------------------------------------------------------
// Public DTOs
// ---------------------------------------------------------------------------

/// The outcome of one waterfall run. The load-bearing return is `routed`: on a
/// LocalWin the task solved locally (no cloud) and `routed` is `None`; on an
/// all-local-fail the failure set was condensed and routed, and `routed` carries
/// the cloud routing DECISION (NOT a live model response — D-026).
#[derive(Debug, Clone, PartialEq)]
pub struct WaterfallOutcome {
    /// `true` when at least one local branch survived the tri-layer check (the task
    /// solved on the local lane); the cloud arm did NOT run. `false` when every
    /// branch failed and the run escalated.
    pub local_win: bool,
    /// Count of candidates dropped by the local tri-layer check (veto/quarantine).
    pub pruned_count: usize,
    /// The synchronous cloud routing DECISION, `Some` only on an all-fail escalation.
    /// This is the router's lane choice over the supplied config — the billed cloud
    /// inference that consumes it is D-026 PENDING (never fired from here).
    pub routed: Option<RoutedProvider>,
    /// Byte length of the condensed [`BeliefState`] wire payload sent on escalation
    /// (`0` on a LocalWin). Always `<= CONDENSED_PAYLOAD_MAX_BYTES`.
    pub condensed_bytes: usize,
}

/// Errors surfaced by the waterfall. The local fan-out leg (I-4 ladder) and the
/// I-11 condensation each can fail; a leg failure aborts the run so the caller
/// falls back to the legacy linear path (the waterfall is additive, GD-4).
#[derive(Debug, thiserror::Error)]
pub enum EscalationWaterfallError {
    /// The I-4 local branch fan-out / tri-layer ladder failed.
    #[error("local branch ladder failed: {0}")]
    Ladder(#[from] SearchLadderError),
    /// The I-11 BeliefState condensation (encode / receipt) failed.
    #[error("belief-state condensation failed: {0}")]
    Condense(#[from] crate::belief_state::BeliefStateError),
    /// The H-7 cloud routing seam refused the escalation hop.
    #[error("cloud route refused: {0}")]
    Route(#[from] CloudModeError),
    /// An `EscalationCascade` SSB receipt write failed.
    #[error("escalation-cascade SSB write failed: {0}")]
    Ssb(#[from] SsbError),
}

// ---------------------------------------------------------------------------
// EscalationWaterfall
// ---------------------------------------------------------------------------

/// The I-7 local-first escalation waterfall. Holds the SHARED I-4 ladder (the
/// local fan-out + tri-layer leg), the H-7 cloud router (the escalation target),
/// and the SSB writer. Constructed once at the dispatch seam (Arc-cloned handles);
/// [`run`](Self::run) is called per task from the production `drive_task` hook.
pub struct EscalationWaterfall {
    /// The I-4 best-of-N ladder: fans out branches, runs the I-17 tri-layer check,
    /// prunes vetoed/quarantined branches, ranks survivors. Composed, not rebuilt.
    ladder: Arc<SearchLadder>,
    /// The H-7 cloud routing seam (the synchronous escalation target on all-fail).
    router: CloudModeRouter,
    /// Shared SSB writer for the `EscalationCascade` lifecycle rows.
    ssb: Arc<Ssb>,
}

impl EscalationWaterfall {
    /// Construct the waterfall from the shipped substrate handles. The caller builds
    /// the I-4 ladder once at the dispatch seam (mirroring `dispatch_search_ladder`'s
    /// construction recipe — GD-1 compose) and supplies the cloud router config.
    #[must_use]
    pub fn new(ladder: Arc<SearchLadder>, router: CloudModeRouter, ssb: Arc<Ssb>) -> Self {
        Self {
            ladder,
            router,
            ssb,
        }
    }

    /// Run the local-first waterfall over K candidate branches.
    ///
    /// 1. Fan out the candidates through the I-4 ladder (per-branch I-17 tri-layer
    ///    verdict + veto/quarantine pruning + best-of-N ranking).
    /// 2. If a survivor was promoted -> `EscalationCascade(LocalWin)`; the task
    ///    solved locally and the cloud arm NEVER runs (E-39 zero-cloud-spend).
    /// 3. If EVERY branch was pruned (no winner) -> condense the failure set into a
    ///    typed I-11 [`BeliefState`] (`EscalationCascade(Condensed)` + the ID-3
    ///    condensed receipt), then route SYNCHRONOUSLY to the cloud seam
    ///    (`EscalationCascade(Escalated)` carrying the routing decision).
    ///
    /// `session_id` keys the per-branch pristine context rebuild; `failed_because`
    /// is the structured reason set carried into the condensate (NO raw history —
    /// stub_ban). `credentials` is the cloud credential set for the escalation hop
    /// (the DECISION is deterministic over the fixture; the billed call is D-026
    /// PENDING and is NOT made here).
    pub async fn run(
        &self,
        candidates: &[BranchInput],
        session_id: &str,
        failed_because: Vec<String>,
        credentials: Option<CloudCredentials>,
    ) -> Result<WaterfallOutcome, EscalationWaterfallError> {
        let session_seq = stable_session_seq(session_id);

        // Step 1: local fan-out + per-branch tri-layer verdict (I-4 composes I-17).
        let ladder_outcome = self
            .ladder
            .run_best_of_n(candidates, session_id, BackendMode::Local)
            .await?;

        // Step 2: a promoted survivor -> the task solved locally. No cloud call.
        if ladder_outcome.winner.is_some() {
            self.emit(EscalationCascadeKind::LocalWin, 0).await?;
            return Ok(WaterfallOutcome {
                local_win: true,
                pruned_count: ladder_outcome.pruned_count,
                routed: None,
                condensed_bytes: 0,
            });
        }

        // Step 3: ALL local branches failed -> condense + escalate synchronously.
        let belief = self.condense(candidates, failed_because)?;
        let encoded = belief.encode()?;
        let condensed_bytes = encoded.len().min(CONDENSED_PAYLOAD_MAX_BYTES);

        // ID-3: the condensation lands its receipt row (un-journaled = FAIL). This
        // is the I-11 `BeliefStateEvent(Condensed)` receipt, distinct from our own
        // `EscalationCascade(Condensed)` lifecycle beacon below.
        belief
            .emit_condensed_receipt(&self.ssb, session_seq, now_ms())
            .await?;
        self.emit(EscalationCascadeKind::Condensed, condensed_bytes)
            .await?;

        // Synchronous cloud escalation DECISION (H-7 seam). The router returns the
        // lane choice; the billed inference that consumes the condensate is D-026
        // PENDING and is NOT made here (IC-1: never the batch lane; this is the
        // interactive synchronous route).
        let routed = self.router.route(ExecMode::Cloud, credentials)?;
        self.emit(EscalationCascadeKind::Escalated, condensed_bytes)
            .await?;

        Ok(WaterfallOutcome {
            local_win: false,
            pruned_count: ladder_outcome.pruned_count,
            routed: Some(routed),
            condensed_bytes,
        })
    }

    /// Build the typed I-11 condensate from the failed branch set. Carries the
    /// branch ids as tried-approach labels + the structured failure reasons; sends
    /// NO raw history (stub_ban / ID-3). `current_ast_state` / `dkg_entities` are
    /// left empty in v1 (the generator owns AST/DKG snapshotting upstream); the
    /// condensate is intentionally the small typed summary, never the full transcript.
    fn condense(
        &self,
        candidates: &[BranchInput],
        failed_because: Vec<String>,
    ) -> Result<BeliefState, EscalationWaterfallError> {
        let tried: Vec<TriedApproach> = candidates
            .iter()
            .map(|c| TriedApproach {
                label: c.branch.branch_id.clone(),
                token_cost: 0,
            })
            .collect();
        Ok(BeliefState::new(
            tried,
            failed_because,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0,
        ))
    }

    /// Emit one `EscalationCascade` lifecycle row (best-effort observability: a
    /// write failure on the LocalWin/Condensed beacon is NON-fatal — the routing
    /// decision is the load-bearing return). The `Escalated` row's write IS
    /// propagated (it is the off-agent escalation evidence the contract asserts).
    async fn emit(
        &self,
        kind: EscalationCascadeKind,
        condensed_bytes: usize,
    ) -> Result<(), EscalationWaterfallError> {
        let payload = SsbPayload::EscalationCascade(EscalationCascadePayload {
            kind,
            condensed_bytes: u32::try_from(condensed_bytes).unwrap_or(u32::MAX),
            ts_ms: now_ms(),
        });
        self.ssb
            .write_payload(ESCALATION_WATERFALL_AGENT_ID, 0, payload, Vec::new())
            .await?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Module helpers
// ---------------------------------------------------------------------------

/// Build the default H-7 cloud router for the escalation hop. Uses the default
/// [`CloudConfig`] (Auto mode, Anthropic Sonnet); the waterfall forces
/// [`ExecMode::Cloud`] at the route call so the escalation hop always asks for the
/// cloud lane. Centralised so the dispatch seam and the contract construct the same
/// router shape (GD-1: one construction recipe).
#[must_use]
pub fn default_escalation_router() -> CloudModeRouter {
    CloudModeRouter::new(CloudConfig::default())
}

/// Returns `true` when [`ELAI_ESCALATION_WATERFALL_ENABLE_ENV`] is set to a
/// recognised truthy literal (`"1"` / `"true"` / `"yes"` / `"on"`, case-insensitive).
/// Default-OFF (wire-dark; promotion plan I-7-promotion, GD-4 measured-first-win).
/// Mirrors the I-4 `search_ladder_enabled` gate idiom exactly.
#[must_use]
pub fn escalation_waterfall_enabled() -> bool {
    match std::env::var(ELAI_ESCALATION_WATERFALL_ENABLE_ENV) {
        Ok(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// Stable per-session superstep for the I-11 receipt (the receipt API takes a
/// `u64` session id). Derived deterministically from the session string via a
/// blake3 prefix (blake3 only — CLAUDE.md Anti-Stale-Training) so a same-session
/// re-run reuses the byte-identical key (GD-2). Advisory only.
fn stable_session_seq(session_id: &str) -> u64 {
    let hash = blake3::hash(session_id.as_bytes());
    let mut buf = [0u8; 8];
    buf.copy_from_slice(&hash.as_bytes()[..8]);
    u64::from_le_bytes(buf)
}

/// Current wall-clock ms via the canonical [`OsClock`] idiom (saturating to
/// `i64::MAX`). The row ts is advisory; the seed-reproducible signal is the
/// condensate + routing decision, not the timestamp.
fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}
