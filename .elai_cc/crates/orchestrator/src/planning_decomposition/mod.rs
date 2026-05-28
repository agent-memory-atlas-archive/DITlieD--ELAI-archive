//! E-46 — Planning decomposition substrate.
//!
//! Closes the prior PARTIAL/MISSING re-audit on E-46_planning_decomposition by
//! shipping the additive substrate that the cross-check flagged as absent:
//!
//!   - [`reflection_buffer`]: Reflexion-style per-node textual reflection
//!     buffer with cap-N eviction (arXiv:2303.11366).
//!   - [`decomposition_entropy`]: Shannon-entropy ratio over parent/children
//!     token sets at decomposition forks (arXiv:2503.13657 "improper task
//!     decomposition" failure-mode mitigation).
//!   - [`skill_prior`]: Voyager-style skill-prior surfacing for the
//!     scope-planner decomposer (arXiv:2305.16291).
//!   - [`dual_ledger_ext`]: additive Magentic-One {facts, guesses, plan}
//!     and {current_node, assignment, blockers_rich} extensions layered
//!     atop the shipped [`crate::dual_ledger`] structs (arXiv:2411.04468).
//!   - [`goal_drift_ext`]: term-frequency cosine arm + B-40 FramedIdea
//!     coupling extension to the shipped [`crate::goal_drift_canary`]
//!     Jaccard module (arXiv:2505.02709).
//!   - [`audit_score`]: deterministic LATS-style process-reward scalar
//!     computed from a [`crate::auditor_mode::LevelAuditVerdict`]
//!     (arXiv:2310.04406).
//!
//! Paper grounding (all arXiv IDs verified via WebSearch May 2026):
//!
//!   * arXiv:2411.04468 — Magentic-One dual-ledger orchestrator pattern
//!     (TaskLedger + ProgressLedger; canonical replan-trigger structure).
//!   * arXiv:2407.01489 — AGENTLESS 32.00% SWE-bench Lite at $0.70/issue;
//!     consumed by the [`crate::agentless_shortcut`] free-fn predicate.
//!   * arXiv:2303.11366 — Reflexion verbal-RL episodic reflection buffer.
//!   * arXiv:2505.02709 — Goal-drift evaluation paper (AIES 2025); PRM
//!     supervision corrects deviations.
//!   * arXiv:2503.13657 — Why Multi-Agent LLM Systems Fail; "improper task
//!     decomposition (too granular, too broad, not serializable)" is a top
//!     failure mode addressed by the entropy-stop rule.
//!   * arXiv:2310.04406 — LATS value-function as auditor at every fork.
//!   * arXiv:2305.16291 — Voyager skill library + automatic curriculum.
//!   * arXiv:2311.05772 — ADaPT As-Needed Decomposition (recursive
//!     intervention only when the LLM cannot execute a sub-task) — the
//!     entropy-stop rule + AGENTLESS shortcut are the ELAI equivalent of
//!     ADaPT's conditional-decomposition primitive.
//!   * arXiv:2305.04091 — Plan-and-Solve prompting (ACL 2023); the
//!     [`dual_ledger_ext::PlanStep`] + LedgerFact pattern mirrors the
//!     plan-and-solve "first plan, then carry out subtasks" structure.
//!   * arXiv:2305.18323 — ReWOO planner / worker / solver decoupling;
//!     supports the bubble-up `LedgerReset` cascade shape.
//!
//! Wire-dark by default: every producer-side emit fn (`emit_*` in
//! [`producers`]) checks [`PLANNING_DECOMPOSITION_ENABLE_ENV`] and silently
//! returns if the env var is not set to a truthy value. Read-side
//! consumers (panel queries, contract tests) remain operational.
//!
//! Backward compatibility:
//!   * No existing struct field renamed or removed.
//!   * All new ledger / verdict fields default to `Vec::new()` / `0.0` /
//!     `None` via `#[serde(default)]`.
//!   * New SSB action_type is added as a single `PlanningDecompositionEvent`
//!     variant carrying a Kind-discriminator payload (Pattern-2, 36th
//!     application). SCHEMA_VERSION bumps 183 -> 184.
//!
//! See `workflow/reasoning/E-46-decisions.md` for the full decision log.

pub mod audit_score;
pub mod decomposition_entropy;
pub mod dual_ledger_ext;
pub mod goal_drift_ext;
pub mod producers;
pub mod reflection_buffer;
pub mod skill_prior;

/// Env-var name gating SSB emission for the planning-decomposition substrate.
///
/// Wire-dark default: producers in [`producers`] check this gate and
/// silently return without writing an SSB row unless the env var is set to
/// a truthy value (`"1"` / `"true"` / `"yes"` / `"on"`). Mirrors the
/// precedent set by [`crate::ssb::ELAI_LOCAL_CLOUD_ROUTING_ENABLE_ENV`] +
/// [`crate::ssb::ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV`].
///
/// CONTRACT: The literal string MUST match
/// [`crate::ssb::ELAI_PLANNING_DECOMPOSITION_ENABLE_ENV`] verbatim. A
/// drift between the two would silently disable the producers without any
/// SSB row to point at — strict-equality assertion lives in the
/// `planning_decomposition_env_gate_matches_ssb` contract test.
pub const PLANNING_DECOMPOSITION_ENABLE_ENV: &str = "ELAI_PLANNING_DECOMPOSITION_ENABLE";

/// Returns `true` when [`PLANNING_DECOMPOSITION_ENABLE_ENV`] is set to a
/// recognised truthy literal.
///
/// Recognised truthy values: `"1"`, `"true"`, `"yes"`, `"on"` (case
/// insensitive). Anything else (including unset, empty, `"0"`,
/// `"false"`, garbled) returns `false`.
#[must_use]
pub fn is_enabled() -> bool {
    match std::env::var(PLANNING_DECOMPOSITION_ENABLE_ENV) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Tests in this module manipulate the process-global env var so they
    /// must run serially with one another to avoid cross-test interference.
    /// A poisoned mutex still lets subsequent tests proceed (we recover the
    /// guard) — we just need mutual exclusion.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn is_enabled_default_off() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
        assert!(!is_enabled());
    }

    #[test]
    fn is_enabled_truthy_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        for v in ["1", "TRUE", "Yes", "on"] {
            std::env::set_var(PLANNING_DECOMPOSITION_ENABLE_ENV, v);
            assert!(is_enabled(), "{v} should enable");
        }
        std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
    }

    #[test]
    fn is_enabled_falsy_set() {
        let _g = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        for v in ["0", "false", "", "garbage"] {
            std::env::set_var(PLANNING_DECOMPOSITION_ENABLE_ENV, v);
            assert!(!is_enabled(), "{v} should NOT enable");
        }
        std::env::remove_var(PLANNING_DECOMPOSITION_ENABLE_ENV);
    }
}
