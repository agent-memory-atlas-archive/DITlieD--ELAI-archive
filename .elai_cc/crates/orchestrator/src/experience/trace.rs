//! C-49 — Successful-execution trace record.
//!
//! `SuccessTrace` captures one successful task execution under a single
//! `skill_name`. The distiller (`crate::experience::distiller`) reads
//! many traces for a skill and emits ranked
//! [`crate::experience::PromptFragment`] suggestions.
//!
//! Grounding: arXiv:2605.14443 sec 4.2.1 specifies trajectories as
//! `T = (p, C, r)` (prompt, critiques, scalar reward). C-49's heuristic
//! adaptation does NOT carry the dense critique set `C` or the reward
//! `r` — we record only the audit-pass boolean and the prompt fragments
//! that were observed. The richer trajectory is RUNPOD-PENDING (would
//! need the RL prompter wired).

use serde::{Deserialize, Serialize};

use super::fragment::PromptFragment;

/// Per-execution trace pinned to one skill name.
///
/// The `task_summary_hash` / `prompt_template_id` fields are u64 hashes
/// (caller-side blake3-prefix); the raw strings never cross this struct
/// so we can serialise the trace to SSB without leaking task-content
/// detail. The actual textual fragments live in `fragments` so the
/// distiller can do its counting work locally, then drop them when
/// projecting to the wire surface (the SSB beacon carries only counts +
/// hashes per the C-45 / C-46 / C-47 precedent).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SuccessTrace {
    /// Skill identity (canonical snake_case skill name; matches
    /// [`crate::skill_registry::SkillManifest::name`]).
    pub skill_name: String,
    /// blake3-prefix u64 over the snake-case task description. Caller
    /// computes via `skill_registry::skill_name_hash(task_summary)` or
    /// equivalent.
    pub task_summary_hash: u64,
    /// blake3-prefix u64 over the source prompt template id (if the
    /// trace came from a `compiled_pipeline::CompiledStep::template_prompt`
    /// or similar). `0` when there is no template (free-form prompt).
    pub prompt_template_id: u64,
    /// Audit verdict for the trace. `record_success` only inserts
    /// traces with `audit_pass = true`; the field is retained on the
    /// struct so future code paths (e.g. CAST-style contrastive
    /// learning) can store negatives without changing the schema.
    pub audit_pass: bool,
    /// Producer-side wall-clock milliseconds since UNIX epoch. Caller
    /// supplies the value per the entropy-quarantine project rule.
    pub ts_ms: u64,
    /// Prompt fragments observed in this trace. Caller-supplied; the
    /// distiller aggregates by `text` across many traces.
    pub fragments: Vec<PromptFragment>,
}

impl SuccessTrace {
    /// Convenience constructor. Validates `audit_pass = true` is the
    /// expected insertion case but does NOT enforce it (negatives can be
    /// stored to support future contrastive workflows; the distiller
    /// filters on `audit_pass` at projection time).
    #[must_use]
    pub fn new(
        skill_name: String,
        task_summary_hash: u64,
        prompt_template_id: u64,
        audit_pass: bool,
        ts_ms: u64,
        fragments: Vec<PromptFragment>,
    ) -> Self {
        Self {
            skill_name,
            task_summary_hash,
            prompt_template_id,
            audit_pass,
            ts_ms,
            fragments,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_round_trips_fields() {
        let frag = PromptFragment::new("step by step".into(), 1, 1.0, 100);
        let t = SuccessTrace::new(
            "planning_workflow".into(),
            0xAAAA,
            0xBBBB,
            true,
            12345,
            vec![frag.clone()],
        );
        assert_eq!(t.skill_name, "planning_workflow");
        assert_eq!(t.task_summary_hash, 0xAAAA);
        assert_eq!(t.prompt_template_id, 0xBBBB);
        assert!(t.audit_pass);
        assert_eq!(t.ts_ms, 12345);
        assert_eq!(t.fragments.len(), 1);
        assert_eq!(t.fragments[0].text, "step by step");
    }

    #[test]
    fn negative_trace_is_storable_but_marked() {
        let t = SuccessTrace::new("x".into(), 0, 0, false, 0, vec![]);
        assert!(!t.audit_pass);
    }
}
