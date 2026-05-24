//! C-49 — Iterative-distillation experience module.
//!
//! Heuristic adaptation of arXiv:2605.14443 (Sayana, Todi, Jash 2026 —
//! "Prompting Policies for Multi-step Reasoning and Tool-Use in Black-box
//! LLMs with Iterative Distillation of Experience"). See
//! `workflow/papers/iterative-distillation-interface.md` for the paper
//! interface map.
//!
//! IMPORTANT GROUNDING CAVEATS (recorded here so the next reader is not
//! misled by the module name):
//!   - The paper's method trains a SMALL PROMPTER POLICY via RL
//!     (Group-Relative Policy Optimization + KL anchor to a reference
//!     policy). This module ships only the HEURISTIC counting-based
//!     fragment distiller. The full RL-trained prompter is documented as
//!     `RUNPOD-PENDING: iterative_distillation_prompter_rl_qwen3_8b`
//!     (Section 5.2 of the paper specifies Adam + lr=1e-5; reproducing it
//!     requires GPU + a feedback-model + critique chain we do not yet
//!     wire).
//!   - The paper assumes a BLACK-BOX worker accessed through API calls.
//!     ELAI's local sidecar is controllable; the heuristic distiller does
//!     not exploit that fact (so it works equally well for black-box and
//!     controllable workers).
//!   - The paper reports per-table-row numbers (e.g. Dyck 63 -> 91 with
//!     buffer, Web of Lies 52 -> 90 WITHOUT buffer). Do NOT cite "55 -> 90
//!     overall" or "74 -> 91 overall" as expected ELAI outcomes — those
//!     are abstract-level aggregate ranges across BBEH + Tau-bench and
//!     vary by task.
//!   - The paper's Web of Lies result (sec 5.3, Table 1 footnote)
//!     explicitly ran WITHOUT the experience buffer. Buffer-vs-no-buffer
//!     is task-dependent. The heuristic distiller exposes the buffer as
//!     a per-skill bin; downstream consumers MAY ignore the bin (matching
//!     the no-buffer regime).
//!   - The paper's Section 5.5 cross-model transfer table shows poor
//!     transfer to a LARGER worker (Pro: +7.5 vs in-distrib Flash:
//!     +28.0). Per-skill distillation here keeps fragments scoped to one
//!     `skill_name` so cross-model sharing is intentionally avoided.
//!
//! Wire-dark hook: distillation is gated behind the
//! `ELAI_EXPERIENCE_DISTILL_ENABLE` environment variable (default off).
//! Empty corpus / unknown skill is a transparent no-op — never panics.
//!
//! No new external dependencies. Reuses `blake3` (already in workspace
//! via `skill_registry::skill_name_hash`) for fragment hashing.

pub mod corpus;
pub mod distiller;
pub mod fragment;
pub mod lookup;
pub mod trace;

pub use corpus::{record_success, CorpusError, ExperienceCorpus, MAX_TRACES_PER_SKILL};
pub use distiller::{
    distill_fragments, DistillationConfig, DEFAULT_MIN_FREQ, DEFAULT_MIN_SUCCESS_RATE,
};
pub use fragment::PromptFragment;
pub use lookup::lookup_distilled_fragments;
pub use trace::SuccessTrace;

/// Environment-variable gate (default off — wire-dark pattern). When set
/// to a truthy value (`1`, `true`, `on`, case-insensitive), task_runner's
/// post-audit hook will invoke [`record_success`] with the captured
/// trace. Off-state is a transparent no-op.
pub const ENV_ENABLE: &str = "ELAI_EXPERIENCE_DISTILL_ENABLE";

/// Truth-check for [`ENV_ENABLE`]. Pulled into its own helper so the
/// hook site and the CLI surface agree on the parse rule. Case-
/// insensitive; recognises `1` / `true` / `on` / `yes`.
#[must_use]
pub fn distillation_enabled() -> bool {
    match std::env::var(ENV_ENABLE) {
        Ok(raw) => {
            let lower = raw.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "on" | "yes")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod env_tests {
    use super::*;

    #[test]
    fn enabled_when_env_set_truthy() {
        let original = std::env::var(ENV_ENABLE).ok();
        // SAFETY: tests run in-process; the helper restores the prior
        // value on scope exit to keep parallel-test ordering deterministic.
        std::env::set_var(ENV_ENABLE, "1");
        assert!(distillation_enabled());
        std::env::set_var(ENV_ENABLE, "TRUE");
        assert!(distillation_enabled());
        std::env::set_var(ENV_ENABLE, "no");
        assert!(!distillation_enabled());
        std::env::remove_var(ENV_ENABLE);
        if let Some(prev) = original {
            std::env::set_var(ENV_ENABLE, prev);
        }
    }
}
