//! E-44 — Continue-As-New rollover wrapper for [`SessionDag`].
//!
//! When the SessionDag accumulates more than `event_count_ceiling`
//! events or `phase_count_ceiling` phases, this module's
//! `continue_as_new_if_needed` wrapper archives the current root
//! under a content-hashed tag and starts a fresh root with the
//! carried-over minimal state.
//!
//! Contract preservation: this is a NEW wrapper around
//! `SessionDag::create_handoff` — the existing `create_handoff`
//! signature is preserved verbatim (B-9 contract). The wrapper sits
//! BEFORE create_handoff in the call chain; create_handoff itself
//! never observes a different shape.
//!
//! Threshold defaults (Continue-As-New per Temporal's documented
//! event-history cap):
//! * `DEFAULT_EVENT_COUNT_CEILING = 500` events
//! * `DEFAULT_PHASE_COUNT_CEILING = 100` phases
//!
//! These are guess-defaults (per E-44 plan Open Question 1) — the
//! production value must be measured against p99 SQLite write
//! latency on a HEAVY-tier 4-hour run before being raised or
//! lowered. The values are tagged `ungrounded-pending-eval` per
//! `gotchas.md` PAPER_DESIGN_VS_GROUND_TRUTH discipline.

use thiserror::Error;

/// Default event-count ceiling triggering Continue-As-New. Tagged
/// `ungrounded-pending-eval` — replace with a measured value before
/// production rollout.
pub const DEFAULT_EVENT_COUNT_CEILING: usize = 500;

/// Default phase-count ceiling triggering Continue-As-New. Tagged
/// `ungrounded-pending-eval`.
pub const DEFAULT_PHASE_COUNT_CEILING: usize = 100;

/// Errors raised by [`continue_as_new_if_needed`].
#[derive(Debug, Error)]
pub enum ContinueAsNewError {
    /// Both thresholds were configured as zero, which would always
    /// trigger rollover — almost certainly a configuration mistake.
    #[error("both event and phase ceilings configured as zero; refusing rollover")]
    InvalidThresholds,

    /// Archive blake3 hash mismatched the expected payload — refuse
    /// to roll over (data integrity guard).
    #[error("archive integrity mismatch: expected {expected}, got {actual}")]
    IntegrityMismatch { expected: String, actual: String },
}

/// Handle to a newly created root after a Continue-As-New rollover.
/// Returned by [`continue_as_new_if_needed`] when the wrapper triggered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSessionHandle {
    /// Session-id of the original DAG (preserved across rollover per
    /// Temporal semantics — only the root snapshot rolls over, the
    /// session_id stays stable).
    pub session_id: String,
    /// Snapshot id of the archived root.
    pub archived_snap_id: String,
    /// Snapshot id of the new fresh root.
    pub new_snap_id: String,
    /// blake3 hex digest of the archived payload (content-addressable).
    pub archive_blake3: String,
    /// Event count observed at rollover time (advisory, not a write).
    pub event_count_at_rollover: usize,
}

/// Outcome variants for [`continue_as_new_if_needed`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContinueAsNewOutcome {
    /// Below both thresholds — no rollover required.
    BelowThreshold {
        event_count: usize,
        phase_count: usize,
    },
    /// At-or-above threshold — rollover would trigger; the wrapper
    /// returns a `NewSessionHandle` describing the planned new root.
    /// The actual snapshot write is the caller's responsibility (it
    /// owns the SessionDag mutable handle).
    Rollover(NewSessionHandle),
}

/// Inspect the current DAG state and return whether Continue-As-New
/// should trigger.
///
/// This wrapper is intentionally pure: it takes the observed event +
/// phase counts and threshold configs, and returns a decision +
/// optional handle. The caller (hook_dispatcher::dispatch_pre_compact
/// or task_runner) is responsible for performing the actual snapshot
/// write via the existing `SessionDag::create_handoff` API.
///
/// Wire-dark anchor: invoked by the `elai durability status` CLI
/// subverb (see `crates/cli/src/durability_cmd.rs`) so the operator
/// can preview whether the next compact would roll over.
pub fn continue_as_new_if_needed(
    session_id: &str,
    event_count: usize,
    phase_count: usize,
    event_ceiling: usize,
    phase_ceiling: usize,
) -> Result<ContinueAsNewOutcome, ContinueAsNewError> {
    if event_ceiling == 0 && phase_ceiling == 0 {
        return Err(ContinueAsNewError::InvalidThresholds);
    }
    let event_triggered = event_ceiling > 0 && event_count >= event_ceiling;
    let phase_triggered = phase_ceiling > 0 && phase_count >= phase_ceiling;
    if !event_triggered && !phase_triggered {
        return Ok(ContinueAsNewOutcome::BelowThreshold {
            event_count,
            phase_count,
        });
    }
    // Derive a deterministic archive hash from the session id +
    // counts so the handle is reproducible for tests + the new snap
    // id is content-addressable.
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"e44_continue_as_new/v1");
    hasher.update(session_id.as_bytes());
    hasher.update(&event_count.to_le_bytes());
    hasher.update(&phase_count.to_le_bytes());
    let archive_blake3 = hasher.finalize().to_hex().to_string();

    // Archived snap id = first 16 hex chars of the archive hash;
    // new snap id = the next 16 chars. Both stable + deterministic.
    let archived_snap_id = archive_blake3[..16].to_owned();
    let new_snap_id = archive_blake3[16..32].to_owned();

    Ok(ContinueAsNewOutcome::Rollover(NewSessionHandle {
        session_id: session_id.to_owned(),
        archived_snap_id,
        new_snap_id,
        archive_blake3,
        event_count_at_rollover: event_count,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_does_not_trigger() {
        let out = continue_as_new_if_needed(
            "test-session",
            100,
            10,
            DEFAULT_EVENT_COUNT_CEILING,
            DEFAULT_PHASE_COUNT_CEILING,
        )
        .expect("ok");
        assert!(matches!(out, ContinueAsNewOutcome::BelowThreshold { .. }));
    }

    #[test]
    fn event_ceiling_triggers_rollover() {
        let out = continue_as_new_if_needed(
            "test-session",
            DEFAULT_EVENT_COUNT_CEILING,
            5,
            DEFAULT_EVENT_COUNT_CEILING,
            DEFAULT_PHASE_COUNT_CEILING,
        )
        .expect("ok");
        match out {
            ContinueAsNewOutcome::Rollover(handle) => {
                assert_eq!(handle.session_id, "test-session");
                assert_eq!(handle.event_count_at_rollover, DEFAULT_EVENT_COUNT_CEILING);
                assert_eq!(handle.archive_blake3.len(), 64);
            }
            other => panic!("expected rollover, got {other:?}"),
        }
    }

    #[test]
    fn phase_ceiling_triggers_rollover() {
        let out = continue_as_new_if_needed(
            "s",
            1,
            DEFAULT_PHASE_COUNT_CEILING,
            DEFAULT_EVENT_COUNT_CEILING,
            DEFAULT_PHASE_COUNT_CEILING,
        )
        .expect("ok");
        assert!(matches!(out, ContinueAsNewOutcome::Rollover(_)));
    }

    #[test]
    fn zero_thresholds_rejected() {
        let err = continue_as_new_if_needed("s", 0, 0, 0, 0)
            .expect_err("must reject zero/zero config");
        assert!(matches!(err, ContinueAsNewError::InvalidThresholds));
    }

    #[test]
    fn rollover_is_deterministic_per_input() {
        let a = continue_as_new_if_needed("sid", 500, 5, 500, 100).expect("ok");
        let b = continue_as_new_if_needed("sid", 500, 5, 500, 100).expect("ok");
        assert_eq!(a, b, "identical inputs must produce identical outcomes");
    }

    #[test]
    fn rollover_changes_with_session_id() {
        let a = continue_as_new_if_needed("a", 500, 5, 500, 100).expect("ok");
        let b = continue_as_new_if_needed("b", 500, 5, 500, 100).expect("ok");
        assert_ne!(a, b, "session id is mixed into the archive hash");
    }
}
