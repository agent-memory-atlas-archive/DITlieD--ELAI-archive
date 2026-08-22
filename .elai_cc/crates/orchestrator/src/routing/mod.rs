//! C-46 — Uno-Orchestra parsimonious agent routing (selective delegation).
//!
//! Implements the selective-delegation primitive from Cui et al 2026
//! ("Uno-Orchestra: Parsimonious Agent Routing via Selective
//! Delegation", arXiv:2605.05007). The module provides three composable
//! pieces that sit at the spawn boundary of ELAI's `AgentPool`:
//!
//!   - [`TaskDifficultyClassifier`] — pure heuristic classifier emitting
//!     a scalar difficulty score in `[0, 1]`. Includes optional async
//!     hooks for DKG repo-centrality + sidecar embedding cosine; if both
//!     are unavailable the classifier collapses gracefully to the
//!     heuristic features (token count, code-block presence, question-
//!     complexity keywords, file count, multi-turn flag).
//!   - [`SelectiveDelegator`] — given a [`RoutingTask`], an available
//!     `BTreeMap<AgentRole, RuntimeAdapter>` worker pool, and (optionally)
//!     per-role credit weights from C-45, decides which `(AgentRole,
//!     Primitive)` pairs to dispatch. Returns an empty `Vec` on lazy-mode
//!     direct-answer (paper's "trivial queries collapse to one turn at
//!     near-zero dispatch cost" invariant).
//!   - [`RoutingPolicy`] — typed knobs loaded from elai_cc.toml's
//!     `[agent.selective_delegation]` section (enabled, alpha, S_max,
//!     max_workers_per_query). Construction enforces paper
//!     invariants `alpha in [0, 1]` and `S_max <= 0.10` (raising
//!     S_max above 0.10 enables cost-hacking incorrect answers per
//!     paper Section 5).
//!
//! Pilot scope (mirrors C-45): the policy ROUTES across the 8 base
//! `AgentRole` variants (Executor, Auditor, Researcher, Designer,
//! PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator). The
//! 8 newer variants (B-40 IdeaFramer/IdeaCritic, B-41 five per-level
//! Auditor* variants) are EXCLUDED from the pilot; they fall through to
//! the existing always-spawn path. See
//! [`crate::credit_attribution::partition::pilot_roles`] for the
//! canonical pilot list.
//!
//! Paper-invariant enforcement: critical paper assertions are encoded
//! as [`invariants::InvariantError`] at construction time + verified by
//! the contract test [`crates/contracts/tests/c46_uno_orchestra_routing_contract.rs`](../../../contracts/tests/c46_uno_orchestra_routing_contract.rs).
//!
//! Composition with C-45: the [`SelectiveDelegator::select_workers`]
//! signature takes an `Option<BTreeMap<AgentRole, f32>>` of credit
//! weights so the cold-start path (no credits yet — `[agent.credit_attribution].enabled = false`)
//! collapses to a difficulty-only policy without panic. The boundary
//! copies the C-45 `CreditReport.per_role: HashMap` into a
//! `BTreeMap<AgentRole, f32>` for deterministic iteration order, per
//! CLAUDE.md anti-stale-training (BTreeMap-for-iteration) rule.

pub mod budget_controller;
pub mod cost_profile;
pub mod difficulty;
pub mod invariants;
pub mod policy;
pub mod router_artifact;
pub mod selective;

pub use difficulty::{
    HeuristicFeatures, RoutingTask, TaskDifficultyClassifier,
};
pub use invariants::{InvariantError, ALPHA_MAX, ALPHA_MIN, S_MAX_CAP};
pub use policy::{RoutingPolicy, RoutingPolicyToml};
pub use selective::{Primitive, SelectiveDelegator};

pub use cost_profile::{cost_profiler_enabled, CostProfileVerdict, CostProfiler, ENV_COST_PROFILER_ENABLE};


pub use router_artifact::{artifact_config_hash, artifact_data_window, configured_artifact_path, resolve_router_artifact, register_router_promotion, RouterArtifactVerdict, ENV_ROUTER_ARTIFACT_PATH, ROUTER_OBJECTIVE};
