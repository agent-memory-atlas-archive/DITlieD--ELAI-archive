//! E-21 peer-systems harness surface (Tier-B addendum, completion-drive
//! s-25120). Closes peer-benchmark gaps documented in
//! `workflow/improvement-sweep-2026-05-19/reports/21-peer-systems-llm-as-worker.md`
//! without duplicating shipped C-3/C-38 PageRank infrastructure or the
//! existing role-time `mini_critic` lint.
//!
//! Modules:
//!   - [`critic_rerank`] -- inference-time N-candidate reranker (OpenHands
//!     Nov 2025 method, distinct from [`crate::mini_critic`] which is the
//!     role-time signature/lint critic).
//!   - [`sbfl`] -- pure-module Ochiai spectrum-based fault-localization
//!     compute. No coverage-harness dep added in this pass; caller
//!     assembles the [`sbfl::CoverageReport`] from any source.
//!   - [`repo_state_store`] -- `.elai_cc/.elai/state/` writers for
//!     compaction-survivable state (current_phase / last_decisions /
//!     runtime_choices).
//!   - [`aci_audit`] -- pure emitter for the workflow/audits/
//!     aci-tool-shape markdown artifact (SWE-agent ACI command-shape
//!     audit, arXiv:2405.15793).
//!
//! Wire-dark by default. Production emission of [`crate::ssb::SsbActionType::
//! PeerSystemsEvent`] is gated on `ELAI_PEER_SYSTEMS_ENABLE=1`.
//!
//! Plan: docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md
//! Decisions: workflow/reasoning/E-21-decisions.md (Tier-B addendum).

pub mod aci_audit;
pub mod critic_rerank;
pub mod repo_state_store;
pub mod sbfl;

/// Environment-variable name that gates production emission of
/// `SsbActionType::PeerSystemsEvent` rows. Wire-dark default: emission
/// is silently skipped until callers set this to a truthy value
/// (`"1"` / `"true"` / `"yes"`).
///
/// Mirrors the precedent set by other Pattern-2 substrates
/// (`ELAI_SKILL_SYSTEM_ENABLE`, `ELAI_LOCAL_CLOUD_ROUTING_ENABLE`,
/// `ELAI_VLM_IMAGE_VALIDATION_ENABLE`).
pub const ELAI_PEER_SYSTEMS_ENABLE_ENV: &str = "ELAI_PEER_SYSTEMS_ENABLE";

/// Returns true when the wire-dark gate is OFF (default) -- callers
/// should skip the SSB emit. Returns false when the operator has set
/// the env var to a truthy value.
///
/// Truthy values: `"1"`, `"true"`, `"yes"` (case-insensitive). Any
/// other value (including unset / empty) is treated as OFF.
#[must_use]
pub fn peer_systems_emit_disabled() -> bool {
    !peer_systems_emit_enabled()
}

/// Companion of [`peer_systems_emit_disabled`]; inlined separately so
/// callers can write `if peer_systems_emit_enabled() { ... }` without a
/// double negation.
#[must_use]
pub fn peer_systems_emit_enabled() -> bool {
    match std::env::var(ELAI_PEER_SYSTEMS_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes")
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Cargo runs unit tests within one crate in parallel. The three env-
    // var tests below mutate the SAME process-global env var
    // (ELAI_PEER_SYSTEMS_ENABLE) and race with each other -- e.g.
    // gate_default_is_disabled removes the var while gate_one_enables
    // is between set + assert, flipping the assertion result. Serialize
    // env-mutating tests via this module-level mutex. .unwrap() is
    // acceptable in #[cfg(test)] per CLAUDE.md ("acceptable ONLY in:
    // test code").
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn gate_default_is_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        // SAFETY: tests set/clear the env var; std::env::remove_var is
        // safe under the ENV_LOCK serialization above. We use a unique
        // sentinel value to avoid collision with developer env.
        unsafe {
            std::env::remove_var(ELAI_PEER_SYSTEMS_ENABLE_ENV);
        }
        assert!(peer_systems_emit_disabled());
        assert!(!peer_systems_emit_enabled());
    }

    #[test]
    fn gate_one_enables() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var(ELAI_PEER_SYSTEMS_ENABLE_ENV, "1");
        }
        assert!(peer_systems_emit_enabled());
        assert!(!peer_systems_emit_disabled());
        unsafe {
            std::env::remove_var(ELAI_PEER_SYSTEMS_ENABLE_ENV);
        }
    }

    #[test]
    fn gate_garbage_value_disabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var(ELAI_PEER_SYSTEMS_ENABLE_ENV, "garbage");
        }
        assert!(peer_systems_emit_disabled());
        unsafe {
            std::env::remove_var(ELAI_PEER_SYSTEMS_ENABLE_ENV);
        }
    }
}
