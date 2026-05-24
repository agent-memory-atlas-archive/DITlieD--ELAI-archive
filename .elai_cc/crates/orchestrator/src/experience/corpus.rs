//! C-49 — Experience corpus (per-skill trace store).
//!
//! `ExperienceCorpus` stores up to `MAX_TRACES_PER_SKILL` (=100)
//! [`SuccessTrace`]s per skill in a deterministic LRU shape. The cap
//! matches the C-47 `CASE_BASE_LRU_CAP` precedent — same memory budget,
//! same eviction semantics (oldest-first). Empty corpus / unknown skill
//! is a transparent no-op throughout the API (no panics on greenfield).
//!
//! Paper grounding caveat: arXiv:2605.14443 sec 4.2.2 specifies a
//! threshold-based insertion gate `r_t^(i) >= r_max - epsilon` (Eq. 5).
//! The heuristic distiller in this module SIMPLIFIES the gate to
//! "insert any trace with `audit_pass = true`". The richer gate is
//! RUNPOD-PENDING (would require the feedback model `M_FB` and a
//! scalar reward signal we do not yet compute). See module header.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::trace::SuccessTrace;

/// Maximum traces retained per skill. Matches the C-47
/// `elai_plugins::CASE_BASE_LRU_CAP` precedent (=100). Sized for a small,
/// stable per-skill bin that fits in memory across a long-running ELAI
/// session.
pub const MAX_TRACES_PER_SKILL: usize = 100;

/// Per-skill bin of recent successful traces.
///
/// `skill_traces` keys by canonical snake_case skill name. The
/// per-skill `VecDeque` is push-back / pop-front so the OLDEST trace
/// gets evicted when the cap is reached (stub_ban from the plan:
/// "evict OLDEST when over budget, not newest").
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExperienceCorpus {
    /// Per-skill bin. `BTreeMap` (not `HashMap`) so iteration order is
    /// deterministic — every other ELAI surface that exposes a per-skill
    /// view (`skill_registry::find_skills`, `top_k_skills`, etc) is
    /// already deterministic; this matches.
    skill_traces: BTreeMap<String, VecDeque<SuccessTrace>>,
}

/// Errors surfaced by the corpus API. Empty input + unknown skill are
/// NOT errors (transparent no-op). The variants here cover cases where
/// a caller violates an explicit invariant (e.g. `skill_name` empty).
#[derive(Debug, Error)]
pub enum CorpusError {
    /// Caller passed an empty `skill_name`. We reject empties so the
    /// `BTreeMap` key space stays meaningful.
    #[error("empty skill_name is not a valid corpus key")]
    EmptySkillName,
}

impl ExperienceCorpus {
    /// Construct an empty corpus. No allocations beyond the `BTreeMap`
    /// shell.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a trace under `skill_name`, evicting the oldest trace if
    /// the per-skill bin is at cap. Empty `skill_name` returns
    /// [`CorpusError::EmptySkillName`].
    pub fn record_trace(
        &mut self,
        skill_name: &str,
        trace: SuccessTrace,
    ) -> Result<(), CorpusError> {
        if skill_name.is_empty() {
            return Err(CorpusError::EmptySkillName);
        }
        let bin = self
            .skill_traces
            .entry(skill_name.to_string())
            .or_default();
        // Evict OLDEST first per WIRING_CONTRACT stub_ban — explicit
        // pop_front, never pop_back.
        if bin.len() >= MAX_TRACES_PER_SKILL {
            bin.pop_front();
        }
        bin.push_back(trace);
        Ok(())
    }

    /// Read-only view of the traces for one skill. Returns an empty
    /// slice-like iterator when the skill has never been recorded
    /// (greenfield: NO PANIC, transparent skip — plan stub_ban).
    pub fn traces_for(&self, skill_name: &str) -> &VecDeque<SuccessTrace> {
        // Stash a small const empty deque per Rust idiom so the
        // returned reference is always live. `OnceLock` keeps this
        // safe across multiple readers without unsafe code.
        use std::sync::OnceLock;
        static EMPTY: OnceLock<VecDeque<SuccessTrace>> = OnceLock::new();
        match self.skill_traces.get(skill_name) {
            Some(v) => v,
            None => EMPTY.get_or_init(VecDeque::new),
        }
    }

    /// Iterator over `(skill_name, trace_count)` pairs in deterministic
    /// `BTreeMap` order. Used by the CLI `list` surface.
    pub fn skills(&self) -> impl Iterator<Item = (&str, usize)> {
        self.skill_traces
            .iter()
            .map(|(k, v)| (k.as_str(), v.len()))
    }

    /// Total trace count across every skill bin. Useful for the CLI
    /// surface + tests; O(N_skills) — not a hot path.
    #[must_use]
    pub fn total_traces(&self) -> usize {
        self.skill_traces.values().map(VecDeque::len).sum()
    }

    /// Clear every skill bin. Used by the CLI `reset` action.
    pub fn clear(&mut self) {
        self.skill_traces.clear();
    }
}

/// Free-function entry-point used by `task_runner` post-audit hook +
/// the CLI surface. Returns `Ok(())` on insertion, `Err` on invalid
/// input (empty skill name).
///
/// `audit_pass = false` traces are ALSO accepted (the corpus retains
/// them so future contrastive code paths can use them) but the
/// heuristic distiller in [`super::distiller`] filters on
/// `audit_pass = true` at projection time.
pub fn record_success(
    corpus: &mut ExperienceCorpus,
    skill_name: &str,
    trace: SuccessTrace,
) -> Result<(), CorpusError> {
    corpus.record_trace(skill_name, trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experience::fragment::PromptFragment;

    fn sample_trace(ts: u64) -> SuccessTrace {
        SuccessTrace::new(
            "planning_workflow".into(),
            0xAAAA,
            0xBBBB,
            true,
            ts,
            vec![PromptFragment::new("think step by step".into(), 1, 1.0, ts)],
        )
    }

    #[test]
    fn record_then_read_round_trips() {
        let mut c = ExperienceCorpus::new();
        c.record_trace("planning_workflow", sample_trace(100))
            .expect("record");
        let bin = c.traces_for("planning_workflow");
        assert_eq!(bin.len(), 1);
        assert_eq!(bin.front().expect("trace").ts_ms, 100);
    }

    #[test]
    fn unknown_skill_returns_empty_no_panic() {
        let c = ExperienceCorpus::new();
        let bin = c.traces_for("unknown_skill");
        assert!(bin.is_empty(), "greenfield must be transparent skip");
    }

    #[test]
    fn empty_skill_name_rejected() {
        let mut c = ExperienceCorpus::new();
        assert!(matches!(
            c.record_trace("", sample_trace(0)),
            Err(CorpusError::EmptySkillName)
        ));
    }

    #[test]
    fn lru_evicts_oldest_first() {
        let mut c = ExperienceCorpus::new();
        for ts in 0..(MAX_TRACES_PER_SKILL as u64 + 5) {
            c.record_trace("skill_x", sample_trace(ts))
                .expect("record");
        }
        let bin = c.traces_for("skill_x");
        assert_eq!(bin.len(), MAX_TRACES_PER_SKILL);
        // Oldest 5 (ts 0..5) were evicted; the front trace should be ts=5.
        assert_eq!(bin.front().expect("front").ts_ms, 5);
        // Newest trace is the last we pushed.
        assert_eq!(
            bin.back().expect("back").ts_ms,
            MAX_TRACES_PER_SKILL as u64 + 4
        );
    }

    #[test]
    fn record_success_helper_routes_to_record_trace() {
        let mut c = ExperienceCorpus::new();
        record_success(&mut c, "planning_workflow", sample_trace(50))
            .expect("record_success");
        assert_eq!(c.total_traces(), 1);
    }

    #[test]
    fn skills_iter_is_deterministic() {
        let mut c = ExperienceCorpus::new();
        c.record_trace("z_skill", sample_trace(0)).expect("rec");
        c.record_trace("a_skill", sample_trace(0)).expect("rec");
        c.record_trace("m_skill", sample_trace(0)).expect("rec");
        let names: Vec<&str> = c.skills().map(|(n, _)| n).collect();
        assert_eq!(names, vec!["a_skill", "m_skill", "z_skill"]);
    }
}
