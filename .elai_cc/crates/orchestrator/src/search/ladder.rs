//! I-4 v1 best-of-N dispatcher. See the module docs in `search/mod.rs`.

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};

use crate::context_budget::ContextBudget;
use crate::dispatcher::BackendMode;
use crate::mutation_budget::MutationBudget;
use crate::pristine_path_filter::{PristineFilterError, PristinePathFilter};
use crate::ssb::{SearchNodeEventKind, SearchNodeEventPayload, Ssb, SsbPayload};
use crate::verifier_stack::{
    Branch, FlakeQuarantineLedger, TriLayerStack, VerdictRow, VerifierStackError,
};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers — HARD RULE)
// ---------------------------------------------------------------------------

/// Re-export of the I-4 env-gate literal (canonical definition lives next to the
/// SSB payload in `ssb::payloads`). The `task_runner::dispatch_search_ladder`
/// seam and the contract test both read it through this module.
pub use crate::ssb::ELAI_SEARCH_LADDER_ENABLE_ENV;

/// `[DESIGN]` best-of-N candidate width for v1. The ladder ranks however many
/// candidate branches it is handed; this is the advisory generation width the
/// production seam requests from the generator. Labelled `[DESIGN]` per ID-5
/// (measurement-before-gate): it is NOT a measured threshold, it is the v1
/// fan-out. A future calibration may tune it; never inline a bare literal.
pub const BEST_OF_N_DEFAULT_WIDTH: usize = 4;

/// Context window (in model tokens) the per-branch [`ContextBudget`] is built
/// with when the caller does not supply one. A conservative local-lane default
/// so the blake3 breakpoint re-anchor inside `build_branch_context` runs over a
/// realistic window. Named so the value is greppable and never a bare literal.
const DEFAULT_BRANCH_CONTEXT_WINDOW_TOKENS: usize = 32_000;

/// Agent id stamped on every `SearchNodeEvent` SSB row this module emits.
const SEARCH_LADDER_AGENT_ID: &str = "search_ladder";

// ---------------------------------------------------------------------------
// Public DTOs
// ---------------------------------------------------------------------------

/// One candidate fed to the ladder: the [`Branch`] to score plus the optional
/// per-branch receipts the ranker uses as tie-breakers.
///
/// `mutation_outcomes` is the I-20 cone-selected per-mutant kill set (`true` =
/// caught) the [`MutationBudget`] Wilson-CI sampler scores into a kill-rate
/// (rank-2). `progress_score` is the rank-3 tie-breaker in `[0.0, 1.0]`. Both
/// are `None`/`0.0`-defaulted when a generator does not supply them; the ladder
/// never fabricates a receipt (ID-4 / no-fabricated-measurements).
#[derive(Debug, Clone)]
pub struct BranchInput {
    /// The candidate branch (id + generated source) to score.
    pub branch: Branch,
    /// I-20 per-mutant kill outcomes over the branch's cone (`true` = killed).
    /// Empty -> the branch's mutation kill-rate is `0.0` (no receipt supplied).
    pub mutation_outcomes: Vec<bool>,
    /// Rank-3 tie-breaker progress score in `[0.0, 1.0]`.
    pub progress_score: f64,
}

impl BranchInput {
    /// Construct an input from a branch with no extra receipts (kill-rate and
    /// progress default to `0.0`). Convenience for the rank-1-only path.
    #[must_use]
    pub fn from_branch(branch: Branch) -> Self {
        Self {
            branch,
            mutation_outcomes: Vec::new(),
            progress_score: 0.0,
        }
    }
}

/// A surviving branch with its full ranking key. Ordered by the ladder's
/// lexicographic (tri-layer score, mutation kill-rate, progress score) priority.
#[derive(Debug, Clone, PartialEq)]
pub struct RankedBranch {
    /// The branch's tri-layer verdict receipt (rank-1 source).
    pub verdict: VerdictRow,
    /// I-20 mutation kill-rate (Wilson-CI point estimate, rank-2) in `[0.0,1.0]`.
    pub mutation_kill_rate: f64,
    /// Progress score (rank-3) in `[0.0, 1.0]`.
    pub progress_score: f64,
}

/// The result of one best-of-N run.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchLadderOutcome {
    /// The best-of-N winner (highest-ranked non-vetoed survivor), or `None` when
    /// every candidate was pruned (a `BudgetCollapsed` row was emitted).
    pub winner: Option<RankedBranch>,
    /// Count of candidates dropped (hard-veto or flake-quarantine).
    pub pruned_count: usize,
    /// Whether the survivor set tripped L3 disagreement escalation (I-17).
    pub escalated: bool,
    /// The survivor set, ranked best-first (empty when all pruned).
    pub ranked: Vec<RankedBranch>,
}

/// Errors surfaced by the ladder. The pristine-context and tri-layer legs can
/// each fail; a leg failure on one branch aborts the run (the caller falls back
/// to the linear retry).
#[derive(Debug, thiserror::Error)]
pub enum SearchLadderError {
    /// The I-5 pristine per-branch context rebuild failed.
    #[error("pristine context build failed for branch {branch_id}: {source}")]
    PristineContext {
        /// The branch whose context rebuild failed.
        branch_id: String,
        /// The underlying I-5 error.
        #[source]
        source: PristineFilterError,
    },
    /// The I-17 tri-layer L3 escalation route failed.
    #[error("tri-layer escalation failed: {0}")]
    Verifier(#[from] VerifierStackError),
}

// ---------------------------------------------------------------------------
// SearchLadder
// ---------------------------------------------------------------------------

/// The v1 best-of-N ladder. Holds the SHARED substrate handles (constructed once
/// at the dispatch seam, Arc-cloned across branches) and ranks candidates by
/// verifier receipts. The I-2 workspace ([`crate::WorktreePoolImpl`], FORK-A) is
/// the isolation substrate; v1 ranks statelessly and does not touch the H-49
/// SessionDag (FORK-B).
pub struct SearchLadder {
    /// The I-17 tri-layer stack (the per-branch ranker). Shared.
    stack: Arc<TriLayerStack>,
    /// The I-5 pristine per-branch context filter. Shared.
    pristine: Arc<PristinePathFilter>,
    /// The I-17 flake-quarantine ledger (consulted before ranking a branch).
    quarantine: Arc<FlakeQuarantineLedger>,
    /// The I-20 mutation-budget scorer (kill-rate from per-branch cone outcomes).
    mutation_budget: MutationBudget,
    /// Shared SSB writer for the `SearchNodeEvent` rows.
    ssb: Arc<Ssb>,
}

impl SearchLadder {
    /// Construct the ladder from the shipped substrate handles. The caller builds
    /// these once at boot (the I-17 stack mirrors
    /// `dispatch_tri_layer_verdict`'s construction recipe — GD-1 compose).
    #[must_use]
    pub fn new(
        stack: Arc<TriLayerStack>,
        pristine: Arc<PristinePathFilter>,
        quarantine: Arc<FlakeQuarantineLedger>,
        mutation_budget: MutationBudget,
        ssb: Arc<Ssb>,
    ) -> Self {
        Self {
            stack,
            pristine,
            quarantine,
            mutation_budget,
            ssb,
        }
    }

    /// Rank K candidate branches by verifier receipts and select the best-of-N.
    ///
    /// `session_id` keys the per-branch pristine context rebuild; `mode` selects
    /// the regime context window (H-50, via I-5). Each branch is expanded (I-5
    /// context + I-17 verdict, `Expanded` row), vetoed/quarantined branches are
    /// dropped (`Pruned` row, terminal per ID-4), the survivor SET is L3-escalated
    /// (I-17), and the highest-ranked survivor is `Promoted`. An empty survivor
    /// set emits `BudgetCollapsed` and returns `winner == None`.
    pub async fn run_best_of_n(
        &self,
        candidates: &[BranchInput],
        session_id: &str,
        mode: BackendMode,
    ) -> Result<SearchLadderOutcome, SearchLadderError> {
        let mut survivors: Vec<RankedBranch> = Vec::with_capacity(candidates.len());
        let mut pruned_count = 0usize;

        for input in candidates {
            // Step 1: rebuild the pristine per-branch context (I-5 — closes the
            // I-5 dark; excludes sibling chatter, re-anchors blake3 breakpoints).
            self.build_pristine_context(input, session_id, mode).await?;

            // Step 2: score the branch through the I-17 tri-layer stack.
            let verdict = self.stack.run_verdict(&input.branch).await?;

            // Step 3: DROP vetoed (ID-4 terminal) or flake-quarantined branches.
            if self.is_pruned(&verdict) {
                pruned_count += 1;
                self.emit(SearchNodeEventKind::Pruned, &verdict, 0.0, 0.0).await;
                continue;
            }

            // Survivor: compute the rank-2/rank-3 receipts + emit Expanded.
            let kill_rate = self.kill_rate(input);
            self.emit(
                SearchNodeEventKind::Expanded,
                &verdict,
                kill_rate,
                input.progress_score,
            )
            .await;
            survivors.push(RankedBranch {
                verdict,
                mutation_kill_rate: kill_rate,
                progress_score: input.progress_score,
            });
        }

        self.finish(survivors, pruned_count).await
    }

    /// Sort the survivor set best-first, L3-escalate it, and emit the winner's
    /// `Promoted` row (or `BudgetCollapsed` when empty).
    async fn finish(
        &self,
        mut survivors: Vec<RankedBranch>,
        pruned_count: usize,
    ) -> Result<SearchLadderOutcome, SearchLadderError> {
        if survivors.is_empty() {
            self.emit_collapsed(pruned_count).await;
            return Ok(SearchLadderOutcome {
                winner: None,
                pruned_count,
                escalated: false,
                ranked: Vec::new(),
            });
        }

        Self::rank_survivors(&mut survivors);

        // L3 disagreement escalation over the SURVIVOR SET (I-17 — the real
        // multi-branch caller; closes the escalate_on_disagreement dark).
        let verdict_set: Vec<VerdictRow> = survivors.iter().map(|s| s.verdict.clone()).collect();
        let escalated = self.stack.escalate_on_disagreement(&verdict_set).await?;

        // best-of-N: the highest-ranked survivor.
        let winner = survivors[0].clone();
        self.emit(
            SearchNodeEventKind::Promoted,
            &winner.verdict,
            winner.mutation_kill_rate,
            winner.progress_score,
        )
        .await;

        Ok(SearchLadderOutcome {
            winner: Some(winner),
            pruned_count,
            escalated,
            ranked: survivors,
        })
    }

    /// Lexicographic rank: tri-layer score desc, then mutation kill-rate desc,
    /// then progress score desc. Deterministic total order (`total_cmp` on the
    /// f64 keys) so a same-seed re-run picks the byte-identical winner (GD-2).
    /// ID-4: a vetoed branch never reaches here (the caller drops it), so the
    /// rank-1 key is always a real survivor score, never `VETO_SCORE`.
    fn rank_survivors(survivors: &mut [RankedBranch]) {
        survivors.sort_by(|a, b| {
            b.verdict
                .score
                .total_cmp(&a.verdict.score)
                .then(b.mutation_kill_rate.total_cmp(&a.mutation_kill_rate))
                .then(b.progress_score.total_cmp(&a.progress_score))
                // Final tie-break on branch_id keeps the order stable + greppable.
                .then(a.verdict.branch_id.cmp(&b.verdict.branch_id))
        });
    }

    /// True iff the branch must be dropped from selection: a hard-veto (ID-4
    /// terminal, [`VerdictRow::is_vetoed`]) OR a flake-quarantined oracle
    /// ([`FlakeQuarantineLedger::is_quarantined`], consulted on the branch id as
    /// the oracle key over the full seed range).
    fn is_pruned(&self, verdict: &VerdictRow) -> bool {
        verdict.is_vetoed()
            || self
                .quarantine
                .is_quarantined(&verdict.branch_id, (u64::MIN, u64::MAX))
    }

    /// I-20 mutation kill-rate (Wilson-CI point estimate) for a branch's cone
    /// outcomes. Empty outcomes -> `0.0` (no receipt; never fabricated).
    fn kill_rate(&self, input: &BranchInput) -> f64 {
        if input.mutation_outcomes.is_empty() {
            return 0.0;
        }
        let (interval, _verdict) = self.mutation_budget.sample_with_ci(&input.mutation_outcomes);
        interval.point
    }

    /// Rebuild the I-5 pristine per-branch context (closes the I-5 dark). A fresh
    /// [`ContextBudget`] per branch with no sibling payloads/traceback in v1 (the
    /// generator owns sibling exclusion upstream); the call still re-anchors the
    /// blake3 breakpoints and emits the I-5 `ContextPruned` telemetry.
    async fn build_pristine_context(
        &self,
        input: &BranchInput,
        session_id: &str,
        mode: BackendMode,
    ) -> Result<(), SearchLadderError> {
        let mut budget = ContextBudget::new(DEFAULT_BRANCH_CONTEXT_WINDOW_TOKENS);
        self.pristine
            .build_branch_context(
                &mut budget,
                &input.branch.branch_id,
                session_id,
                &[],
                None,
                mode,
            )
            .await
            .map(|_outcome| ())
            .map_err(|source| SearchLadderError::PristineContext {
                branch_id: input.branch.branch_id.clone(),
                source,
            })
    }

    /// Emit one `SearchNodeEvent` row for a branch transition (best-effort: a
    /// write failure is logged, never aborts the ladder — the ranking is the
    /// load-bearing return, the row is observability).
    async fn emit(
        &self,
        kind: SearchNodeEventKind,
        verdict: &VerdictRow,
        kill_rate: f64,
        progress: f64,
    ) {
        let payload = SsbPayload::SearchNodeEvent(SearchNodeEventPayload {
            kind,
            branch_id: verdict.branch_id.clone(),
            tri_layer_verdict: tri_layer_tag(verdict).to_owned(),
            tri_layer_score: verdict.score,
            mutation_kill_rate: kill_rate,
            progress_score: progress,
            ts_ms: now_ms(),
        });
        self.write(payload).await;
    }

    /// Emit the honest no-winner `BudgetCollapsed` beacon (every candidate was
    /// pruned). Carries no branch receipt (there is no survivor).
    async fn emit_collapsed(&self, pruned_count: usize) {
        let payload = SsbPayload::SearchNodeEvent(SearchNodeEventPayload {
            kind: SearchNodeEventKind::BudgetCollapsed,
            branch_id: format!("collapsed-{pruned_count}-pruned"),
            tri_layer_verdict: String::new(),
            tri_layer_score: 0.0,
            mutation_kill_rate: 0.0,
            progress_score: 0.0,
            ts_ms: now_ms(),
        });
        self.write(payload).await;
    }

    /// Best-effort SSB write shared by `emit` / `emit_collapsed`.
    async fn write(&self, payload: SsbPayload) {
        if let Err(e) = self
            .ssb
            .write_payload(SEARCH_LADDER_AGENT_ID, 0, payload, Vec::new())
            .await
        {
            tracing::warn!(error = %e, "search_ladder: SearchNodeEvent SSB emit failed (ranking unaffected)");
        }
    }
}

// ---------------------------------------------------------------------------
// Module helpers
// ---------------------------------------------------------------------------

/// Returns `true` when [`ELAI_SEARCH_LADDER_ENABLE_ENV`] is set to a recognised
/// truthy literal (`"1"` / `"true"` / `"yes"` / `"on"`, case-insensitive).
/// Default-OFF (wire-dark; promotion plan I-4-promotion, GD-4 measured-first-win).
/// Mirrors the I-17 `tri_layer_enabled` gate idiom exactly.
#[must_use]
pub fn search_ladder_enabled() -> bool {
    match std::env::var(ELAI_SEARCH_LADDER_ENABLE_ENV) {
        Ok(raw) => matches!(
            raw.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// The tri-layer verdict KIND label recorded on the `SearchNodeEvent` row (the
/// row's action_type already carries the `tri_layer_verdict` wire tag; this
/// names WHICH arm produced the receipt so the branch-board can show why a
/// branch ranked where it did).
fn tri_layer_tag(verdict: &VerdictRow) -> &'static str {
    use crate::ssb::TriLayerVerdictKind;
    match verdict.verdict {
        TriLayerVerdictKind::HardVeto => "hard_veto",
        TriLayerVerdictKind::ScoredSurvivor => "scored_survivor",
        TriLayerVerdictKind::DisagreementEscalated => "disagreement_escalated",
        TriLayerVerdictKind::FlakeQuarantined => "flake_quarantined",
    }
}

/// Current wall-clock ms via the canonical [`OsClock`] idiom (saturating to
/// `i64::MAX`). The per-branch row ts is advisory; the seed-reproducible signal
/// is the receipt fields, not the timestamp.
fn now_ms() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}
