//! I-4 — evidence-ranked search ladder (Wave-4 KEYSTONE).
//!
//! Nothing public ships tree search; what ships is wide parallelism plus an
//! LLM-judge selector. ELAI's verifier stack (I-17) lets selection be BY
//! RECEIPTS instead (IC-6): the LLM is demoted to a stateless hypothesis
//! generator, the orchestrator owns the state, and the SSB records WHY each
//! branch won or lost. This module is the tiered ladder's first rung (v1
//! best-of-N); v2 (tournament) and v3 (PUCT tree over the H-49 SessionDag) are
//! GATED behind a measured v1 win (GD-4) and are NOT shipped here.
//!
//! ## v1 best-of-N (this module)
//! Given K divergent candidate [`crate::verifier_stack::Branch`]es, the ladder:
//!   1. builds each branch's PRISTINE per-branch context (I-5
//!      [`crate::pristine_path_filter::PristinePathFilter::build_branch_context`]
//!      — failed-sibling chatter excluded, tracebacks compressed), emitting an
//!      `Expanded` row;
//!   2. runs each branch through the I-17 tri-layer stack
//!      ([`crate::verifier_stack::TriLayerStack::run_verdict`]);
//!   3. DROPS a branch the stack hard-vetoed
//!      ([`crate::verifier_stack::VerdictRow::is_vetoed`]) or whose oracle is
//!      flake-quarantined
//!      ([`crate::verifier_stack::FlakeQuarantineLedger::is_quarantined`]),
//!      emitting a `Pruned` row (terminal — ID-4: a vetoed branch is NEVER
//!      resurrected, regardless of any neural score);
//!   4. calls
//!      [`crate::verifier_stack::TriLayerStack::escalate_on_disagreement`] over
//!      the SURVIVOR SET (a single-branch run cannot disagree; the ladder is the
//!      first real multi-branch caller I-17 was built for);
//!   5. RANKS the survivors by (deterministic tri-layer score, then I-20
//!      mutation kill-rate, then progress score) and emits `Promoted` for the
//!      best-of-N winner carrying its verifier-derived evidence — or
//!      `BudgetCollapsed` when every candidate was pruned (the honest no-winner
//!      beacon; the caller falls back to the linear-retry path).
//!
//! ## Doctrine (ID-4 / ID-2 / GD-4, enforced structurally)
//! - Selection is purely evidence-ranked (ID-4): the rank key is the
//!   [`VerdictRow`] receipt, never an LLM-judge score alone. A `Promoted` row
//!   always carries verifier-derived evidence — the broken `--llm-judge-only`
//!   arm (which would rank a compile-fail branch first) produces zero `Promoted`
//!   rows and a non-zero exit.
//! - A `HardVeto` is terminal (ID-4): [`SearchLadder::rank_survivors`] only ever
//!   sees non-vetoed rows.
//! - Per-branch isolation is the I-2 substrate's job (the concrete
//!   [`crate::WorktreePoolImpl`], FORK-A); v1 ranks statelessly and does NOT
//!   touch the H-49 SessionDag (FORK-B — the DAG tree store enters at v3).
//! - No v2/v3 code lands before a measured v1 win (GD-4): this module contains
//!   no tournament/MCTS path.
//!
//! Default-OFF (`ELAI_SEARCH_LADDER_ENABLE`, R9 experiment): the multi-branch
//! ladder consumes more compute, so it ships behind a measured-first-win gate;
//! the production caller is `task_runner::dispatch_search_ladder`.

mod ladder;
pub mod score;

pub use ladder::{
    search_ladder_enabled, BranchInput, PrunedBranch, RankedBranch, SearchLadder, SearchLadderError,
    SearchLadderOutcome, BEST_OF_N_DEFAULT_WIDTH, ELAI_SEARCH_LADDER_ENABLE_ENV,
};
pub use score::{
    emit_progress_reward_to_ssb, progress_reward_enabled, DeltaProgressScorer,
    FirstErrorTruncator,
};
