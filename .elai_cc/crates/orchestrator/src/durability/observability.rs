//! E-44 -- Producer-side SSB emit fns for the durability observability
//! substrate.
//!
//! Each `notify_*` free async fn:
//!   1. Checks [`durability_observability_enabled`] -- silently returns
//!      `Ok(None)` if the env gate
//!      [`crate::ssb::ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`] is not
//!      set to a truthy value (wire-dark default).
//!   2. Builds a [`crate::ssb::DurabilityEventPayload`] with the
//!      provided Kind discriminator + free-form details_json.
//!   3. Writes one SSB row via the supplied `&Ssb` handle's async
//!      `write_payload` method.
//!
//! Single-writer producer-pure split per the planning_decomposition +
//! web_orchestrator convention (the modules in this folder take no
//! `&Ssb`; producer helpers live here). 10 fire-points covered by the
//! single `DurabilityEvent` variant + 10-arm Kind discriminator per
//! Pattern-2 (E-20 / E-21 / E-36 / E-42 / E-46 / E-48).
//!
//! 5 of the 10 `notify_*` helpers are stubs in this commit because their
//! producer modules (saga compensation log, reliability metrics rollup,
//! tool-error 3-bucket recovery) are NOT-SHIPPED at SCHEMA_VERSION 186.
//! The wire-tag space + Kind arms still exist so when the producers
//! land they can call straight through without a SCHEMA bump. The
//! present commit exposes the 5 helpers that pair with shipped modules
//! (idempotent_dispatcher, continue_as_new, checkpoint_migrator,
//! compact_lock).
//!
//! All `notify_*` fns return `Result<Option<i64>, SsbError>`:
//!   * `Ok(None)` -- env gate off, no row written, no error.
//!   * `Ok(Some(seq))` -- row written; `seq` is the assigned SSB
//!     auto-increment sequence number.
//!   * `Err(SsbError)` -- propagate the underlying write failure.

use serde_json::json;

use crate::ssb::{
    DurabilityEventKind, DurabilityEventPayload, Ssb, SsbError, SsbPayload,
    ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV,
};

#[cfg(test)]
thread_local! {
    static DURABILITY_OBSERVABILITY_ENABLED_OVERRIDE: std::cell::Cell<Option<bool>> =
        std::cell::Cell::new(None);
}

#[cfg(test)]
pub(crate) fn set_durability_observability_enabled_for_test(value: Option<bool>) -> Option<bool> {
    DURABILITY_OBSERVABILITY_ENABLED_OVERRIDE.with(|override_value| {
        let previous = override_value.get();
        override_value.set(value);
        previous
    })
}

/// Returns true iff the env var
/// [`ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV`] is set to a truthy value
/// (`"1"` / `"true"` / `"yes"` / `"on"`, case-insensitive).
///
/// Wire-dark by default: callers receive `false` unless an operator
/// explicitly opts in. Mirrors
/// `crate::planning_decomposition::is_enabled` /
/// `crate::research::web_orchestrator::is_enabled`.
#[must_use]
pub fn durability_observability_enabled() -> bool {
    #[cfg(test)]
    if let Some(value) = DURABILITY_OBSERVABILITY_ENABLED_OVERRIDE.with(std::cell::Cell::get) {
        return value;
    }

    match std::env::var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.trim().to_ascii_lowercase();
            matches!(lower.as_str(), "1" | "true" | "yes" | "on")
        }
        Err(_) => false,
    }
}

/// Emit a `tool_idempotency_hit` row. Producer:
/// `crate::durability::idempotent_dispatcher` cache-hit path.
pub async fn notify_tool_idempotency_hit(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    call_shape_hash: u64,
    tool_id: &str,
    cache_age_ms: u64,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::ToolIdempotencyHit,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "call_shape_hash": format!("{:016x}", call_shape_hash),
            "tool_id": tool_id,
            "cache_age_ms": cache_age_ms,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `tool_idempotency_replayed_irreversible` row. Producer:
/// `crate::durability::idempotent_dispatcher` for cached
/// side-effect-bearing calls elided to preserve at-most-once semantics.
pub async fn notify_tool_idempotency_replayed_irreversible(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    call_shape_hash: u64,
    tool_id: &str,
    effect_class: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::ToolIdempotencyReplayedIrreversible,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "call_shape_hash": format!("{:016x}", call_shape_hash),
            "tool_id": tool_id,
            "effect_class": effect_class,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `session_dag_continue_as_new` row. Producer:
/// `crate::durability::continue_as_new::continue_as_new_if_needed` on
/// rollover.
#[allow(clippy::too_many_arguments)]
pub async fn notify_session_dag_continue_as_new(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    old_session_id: &str,
    new_session_id: &str,
    trigger: &str,
    event_count: u64,
    phase_count: u32,
    ceiling: u64,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::SessionDagContinueAsNew,
        session_id: Some(old_session_id.to_owned()),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "old_session_id": old_session_id,
            "new_session_id": new_session_id,
            "trigger": trigger,
            "event_count": event_count,
            "phase_count": phase_count,
            "ceiling": ceiling,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `checkpoint_lazy_migrated` row. Producer:
/// `crate::durability::checkpoint_migrator::migrate_checkpoint` on
/// successful transform chain application.
#[allow(clippy::too_many_arguments)]
pub async fn notify_checkpoint_lazy_migrated(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    from_version: u32,
    to_version: u32,
    transform_count: u32,
    bytes_before: u64,
    bytes_after: u64,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::CheckpointLazyMigrated,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "from_version": from_version,
            "to_version": to_version,
            "transform_count": transform_count,
            "bytes_before": bytes_before,
            "bytes_after": bytes_after,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `compact_lock_contended` row. Producer:
/// `crate::durability::compact_lock::CompactLock::try_acquire` when the
/// PreCompact hook and the 85% compact path race.
#[allow(clippy::too_many_arguments)]
pub async fn notify_compact_lock_contended(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    lock_path: &str,
    holder_pid: u32,
    wait_ms: u64,
    outcome: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::CompactLockContended,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "lock_path": lock_path,
            "holder_pid": holder_pid,
            "wait_ms": wait_ms,
            "outcome": outcome,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `compensation_logged` row. Producer:
/// `crate::durability::saga::SagaLogger::log_compensation` (Tranche 7A).
/// Logged BEFORE the irreversible step runs — ordering enforced by the
/// `SagaTicket<Pending>` typestate in saga.rs.
#[allow(clippy::too_many_arguments)]
pub async fn notify_compensation_logged(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    saga_id: &str,
    action_id: &str,
    step_index: u32,
    compensate_tool: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::CompensationLogged,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "saga_id": saga_id,
            "action_id": action_id,
            "step_index": step_index,
            "compensate_tool": compensate_tool,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `compensation_replayed` row. Producer:
/// `crate::durability::saga::SagaLogger::replay_pending` (Tranche 7A).
#[allow(clippy::too_many_arguments)]
pub async fn notify_compensation_replayed(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    saga_id: &str,
    action_id: &str,
    outcome: &str,
    duration_ms: u64,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::CompensationReplayed,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "saga_id": saga_id,
            "action_id": action_id,
            "outcome": outcome,
            "duration_ms": duration_ms,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `compensation_failed` row. Producer:
/// `crate::durability::saga::SagaLogger::record_failure` (Tranche 7A).
#[allow(clippy::too_many_arguments)]
pub async fn notify_compensation_failed(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    saga_id: &str,
    action_id: &str,
    error_class: &str,
    error_message_truncated_200: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::CompensationFailed,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "saga_id": saga_id,
            "action_id": action_id,
            "error_class": error_class,
            "error_message": error_message_truncated_200,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit an `agent_reliability_metrics` row. Producer:
/// `crate::durability::reliability::emit_rollup` (Tranche 7C).
#[allow(clippy::too_many_arguments)]
pub async fn notify_agent_reliability_metrics(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    phase_id: &str,
    success_rate: f32,
    p50_ms: u64,
    p95_ms: u64,
    sample_window: u32,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::AgentReliabilityMetrics,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "agent_id": agent_id,
            "phase_id": phase_id,
            "success_rate": success_rate,
            "p50_latency_ms": p50_ms,
            "p95_latency_ms": p95_ms,
            "sample_window": sample_window,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

/// Emit a `tool_error_classified` row. Producer:
/// `crate::durability::error_taxonomy::emit_classification` (Tranche 7C).
#[allow(clippy::too_many_arguments)]
pub async fn notify_tool_error_classified(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    session_id: Option<&str>,
    tool_id: &str,
    error_class: &str,
    error_message_truncated_200: &str,
    recovery_action: &str,
    ts_ms: i64,
) -> Result<Option<i64>, SsbError> {
    if !durability_observability_enabled() {
        return Ok(None);
    }
    let payload = SsbPayload::DurabilityEvent(DurabilityEventPayload {
        kind: DurabilityEventKind::ToolErrorClassified,
        session_id: session_id.map(str::to_owned),
        agent_id: Some(agent_id.to_owned()),
        details_json: json!({
            "tool_id": tool_id,
            "error_class": error_class,
            "error_message": error_message_truncated_200,
            "recovery_action": recovery_action,
        }),
        ts_ms,
    });
    let seq = ssb
        .write_payload(agent_id, superstep, payload, Vec::new())
        .await?;
    Ok(Some(seq))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_env_default_off() {
        // Save current value (test isolation).
        let prior = std::env::var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV).ok();
        // SAFETY: tests run single-threaded for env mutation by convention;
        // the surrounding crate has many similar gate tests (planning_decomposition,
        // web_orchestrator) that follow the same pattern.
        unsafe {
            std::env::remove_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV);
        }
        assert!(!durability_observability_enabled());
        if let Some(v) = prior {
            unsafe {
                std::env::set_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV, v);
            }
        }
    }

    #[test]
    fn enable_env_truthy_variants() {
        let prior = std::env::var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV).ok();
        for v in ["1", "true", "TRUE", "yes", "YES", "on", "On"] {
            unsafe {
                std::env::set_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV, v);
            }
            assert!(durability_observability_enabled(), "expected truthy for {v}");
        }
        for v in ["0", "false", "no", "off", ""] {
            unsafe {
                std::env::set_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV, v);
            }
            assert!(!durability_observability_enabled(), "expected falsy for {v:?}");
        }
        unsafe {
            std::env::remove_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV);
        }
        if let Some(v) = prior {
            unsafe {
                std::env::set_var(ELAI_DURABILITY_OBSERVABILITY_ENABLE_ENV, v);
            }
        }
    }
}
