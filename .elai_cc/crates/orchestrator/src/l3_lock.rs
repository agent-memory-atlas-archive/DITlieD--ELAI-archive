//! E-7: L3 lock-list extension (D-9 follow-on).
//!
//! Per E-7 plan task 10: the D-9 L3 frozen-surface lock-list grows by 4
//! entries:
//!   - process-supervision logs (SSB audit trail)
//!   - trusted-output sandbox tier (test execution sandbox config)
//!   - verifier-set definitions
//!   - perplexity-LM model path
//!
//! The pre-commit hook integration that blocks lock-list-modification
//! PRs without auditor sign-off is RUNPOD-PENDING.
//!
//! Paper grounding:
//!   - arXiv:1906.01820 (Hubinger mesa-optimization) — grounds open
//!     question on lock-list completeness. The L3 lock cannot enumerate
//!     every reachable surface; this module ships the canonical entries
//!     known at E-7 ship time + an extension point for future entries.
//!
//! Wire-dark: compiles only when `cfg(feature = "darwinian-evolve")`
//! is set.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Lock-list entries. Closed enum -- adding an entry is a coordinated
/// SCHEMA bump (mirrors the TripwireKind closed-enum invariant per
/// DECISION-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockListEntry {
    /// SSB audit trail (process-supervision logs). Once frozen, the
    /// self-mod loop cannot rewrite the audit trail.
    ProcessSupervisionLogs,
    /// Test execution sandbox config (D-9 trusted-output sandbox tier).
    TrustedOutputSandboxConfig,
    /// Verifier-set definitions (D-3 invariant — the verifier set is
    /// itself L3-frozen).
    VerifierSetDefinitions,
    /// Perplexity-LM model path (per DECISION-7 the in-loop perplexity
    /// emit is feature-gated; the out-of-loop perplexity-LM path is
    /// L3-frozen so the self-mod loop cannot swap the reference model).
    PerplexityLmModelPath,
}

impl LockListEntry {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ProcessSupervisionLogs => "process_supervision_logs",
            Self::TrustedOutputSandboxConfig => "trusted_output_sandbox_config",
            Self::VerifierSetDefinitions => "verifier_set_definitions",
            Self::PerplexityLmModelPath => "perplexity_lm_model_path",
        }
    }
}

/// L3 lock-list. Carries per-entry blake3 hashes of the locked content
/// so the assert_lock_intact predicate can detect drift.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockList {
    pub process_supervision_logs: Option<[u8; 32]>,
    pub trusted_output_sandbox_config: Option<[u8; 32]>,
    pub verifier_set_definitions: Option<[u8; 32]>,
    pub perplexity_lm_model_path: Option<[u8; 32]>,
}

/// Lock violation reason.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LockViolation {
    #[error("lock entry {0:?} hash mismatch -- frozen surface was edited")]
    HashMismatch(LockListEntry),
    #[error("lock entry {0:?} missing -- L3 lock not seeded for this surface")]
    Missing(LockListEntry),
}

/// Assert the lock-list is intact against an expected snapshot.
/// Returns `Ok(())` if all entries match; `Err(LockViolation)` on
/// the first mismatch.
pub fn assert_lock_intact(
    current: &LockList,
    expected: &LockList,
) -> Result<(), LockViolation> {
    check_entry(
        LockListEntry::ProcessSupervisionLogs,
        current.process_supervision_logs,
        expected.process_supervision_logs,
    )?;
    check_entry(
        LockListEntry::TrustedOutputSandboxConfig,
        current.trusted_output_sandbox_config,
        expected.trusted_output_sandbox_config,
    )?;
    check_entry(
        LockListEntry::VerifierSetDefinitions,
        current.verifier_set_definitions,
        expected.verifier_set_definitions,
    )?;
    check_entry(
        LockListEntry::PerplexityLmModelPath,
        current.perplexity_lm_model_path,
        expected.perplexity_lm_model_path,
    )?;
    Ok(())
}

fn check_entry(
    which: LockListEntry,
    current: Option<[u8; 32]>,
    expected: Option<[u8; 32]>,
) -> Result<(), LockViolation> {
    match (current, expected) {
        (Some(c), Some(e)) if c == e => Ok(()),
        (Some(_), Some(_)) => Err(LockViolation::HashMismatch(which)),
        (None, Some(_)) | (Some(_), None) => Err(LockViolation::HashMismatch(which)),
        (None, None) => Err(LockViolation::Missing(which)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_lock() -> LockList {
        LockList {
            process_supervision_logs: Some([1u8; 32]),
            trusted_output_sandbox_config: Some([2u8; 32]),
            verifier_set_definitions: Some([3u8; 32]),
            perplexity_lm_model_path: Some([4u8; 32]),
        }
    }

    #[test]
    fn matching_locks_intact() {
        let l = seeded_lock();
        assert!(assert_lock_intact(&l, &l).is_ok());
    }

    #[test]
    fn mismatched_hash_violates() {
        let current = seeded_lock();
        let mut expected = seeded_lock();
        expected.verifier_set_definitions = Some([99u8; 32]);
        assert_eq!(
            assert_lock_intact(&current, &expected),
            Err(LockViolation::HashMismatch(
                LockListEntry::VerifierSetDefinitions
            ))
        );
    }

    #[test]
    fn missing_entry_violates() {
        let current = seeded_lock();
        let mut expected = seeded_lock();
        expected.perplexity_lm_model_path = None;
        assert!(assert_lock_intact(&current, &expected).is_err());
    }
}
