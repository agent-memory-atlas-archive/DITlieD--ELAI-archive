//! J-94 T3-D — attention batch-invariance gate on RRA T1 claims.
//! Fail-closed: T1 is allowed only when attention is EXPLICITLY proven
//! batch-invariant on the sealed determinism basis. Hardware residue
//! (Ampere/FA2 non-determinism) stays; we only stop over-claiming T1.

/// Env-control name that proves attention batch-invariance on the sealed
/// DeterminismApplied basis. Must appear in `env_controls` (exact match).
pub const ATTENTION_BATCH_INVARIANT_FLAG: &str = "ELAI_ATTENTION_BATCH_INVARIANT";

/// Stable downgrade reason when T1 is refused because attention is not
/// proven batch-invariant (CL-4 / GD-5 honesty).
pub const ATTENTION_NOT_BATCH_INVARIANT_REASON: &str =
    "attention_not_batch_invariant";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttentionDeterminismTier {
    /// Attention proven batch-invariant — T1 is ELIGIBLE (other legs may
    /// still downgrade, e.g. token-stream blake3 not wired).
    T1Eligible,
    /// Attention not proven batch-invariant — force T0 (ActionFaithful).
    T0Forced {
        reason: &'static str,
    },
}

/// Decide max RRA tier allowed by attention batch-invariance.
/// Fail-closed: default caller passes false when the flag is absent.
pub fn attention_determinism_tier(attention_batch_invariant: bool) -> AttentionDeterminismTier {
    if attention_batch_invariant {
        AttentionDeterminismTier::T1Eligible
    } else {
        AttentionDeterminismTier::T0Forced {
            reason: ATTENTION_NOT_BATCH_INVARIANT_REASON,
        }
    }
}

/// Read the sealed basis: true ONLY when env_controls contains the exact flag.
pub fn attention_batch_invariant_from_controls(env_controls: &[String]) -> bool {
    env_controls
        .iter()
        .any(|c| c == ATTENTION_BATCH_INVARIANT_FLAG)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn false_forces_t0_with_stable_reason() {
        assert_eq!(
            attention_determinism_tier(false),
            AttentionDeterminismTier::T0Forced {
                reason: ATTENTION_NOT_BATCH_INVARIANT_REASON,
            }
        );
    }

    #[test]
    fn true_is_t1_eligible() {
        assert_eq!(
            attention_determinism_tier(true),
            AttentionDeterminismTier::T1Eligible
        );
    }

    #[test]
    fn controls_are_fail_closed() {
        assert!(!attention_batch_invariant_from_controls(&[]));
        assert!(!attention_batch_invariant_from_controls(&[
            "ELAI_SEED".to_owned(),
            "ELAI_HARDWARE_CLASS".to_owned(),
        ]));
        assert!(attention_batch_invariant_from_controls(&[
            ATTENTION_BATCH_INVARIANT_FLAG.to_owned(),
        ]));
    }
}
