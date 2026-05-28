//! E-19 reasoning quality probe primitives (data types only, no IO).
//!
//! Houses the per-layer scorers + thresholds used by the 6-layer reasoning
//! pipeline (L1 deterministic / L2 micro-reflection / L3 interrogation tree /
//! L4 AB-MCTS / L5 cloud diagnostic / L6 cloud generation). Each submodule is
//! a deterministic, IO-free primitive that the `elai_orchestrator::reasoning`
//! wrappers compose into per-task dispatchers.
//!
//! Wire-dark by default: all production code paths read
//! `ELAI_REASONING_QUALITY_PROBE_ENABLE` and short-circuit when unset.
//! See `workflow/reasoning/E-19-decisions.md`.

pub mod self_certainty;
pub mod ttc_budget;
pub mod typed_cot;

pub use self_certainty::{compute_score, ConfidenceGate, GateVerdict, L2_CONFIDENCE_THRESHOLD};
pub use ttc_budget::{Difficulty, DifficultyEstimator, TtcBudget};
pub use typed_cot::{TypedCotTriple, TYPED_COT_SCHEMA_VERSION};
