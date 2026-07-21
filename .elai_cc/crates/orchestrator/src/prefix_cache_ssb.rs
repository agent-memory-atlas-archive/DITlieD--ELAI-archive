//! J-36: LocalPrefixCacheEvent SSB emit helper.
//!
//! Mirrors `context_calibration::emit_context_calibration_event`: a missed
//! SSB row is never a correctness failure. When no tokio runtime is
//! available, a transient current-thread runtime is built so the row still
//! lands (do NOT spawn-and-drop).

use std::sync::Arc;

use crate::ssb::{
    system_time_ms_i64, LocalPrefixCacheEventPayload, LocalPrefixCacheKind, Ssb, SsbPayload,
};

/// Emit a `LocalPrefixCacheEvent` SSB row.
///
/// Non-fatal: warns on write failure, never propagates. The `agent_id` is
/// `"local_prefix_cache"`, superstep is 0. The timestamp comes from
/// [`crate::ssb::system_time_ms_i64`].
pub fn emit_local_prefix_cache_event(
    ssb: &Arc<Ssb>,
    kind: LocalPrefixCacheKind,
    head_prefill_tokens: u64,
    tail_prefill_tokens: u64,
    eligibility_reason: impl Into<String>,
) {
    let ts_ms = system_time_ms_i64();
    let payload = SsbPayload::LocalPrefixCacheEvent(LocalPrefixCacheEventPayload {
        kind,
        head_prefill_tokens,
        tail_prefill_tokens,
        eligibility_reason: eligibility_reason.into(),
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let agent_id = "local_prefix_cache";
    let emit = async move {
        if let Err(e) = ssb.write_payload(agent_id, 0, payload, vec![]).await {
            tracing::warn!(
                prefix_cache.ssb_err = %e,
                "LocalPrefixCacheEvent SSB emit failed (prefix cache unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_as_str_roundtrip() {
        assert_eq!(LocalPrefixCacheKind::HeadCacheHit.as_str(), "head_cache_hit");
        assert_eq!(LocalPrefixCacheKind::HeadCacheMiss.as_str(), "head_cache_miss");
        assert_eq!(LocalPrefixCacheKind::TailReprefill.as_str(), "tail_reprefill");
        assert_eq!(LocalPrefixCacheKind::CacheIneligible.as_str(), "cache_ineligible");
    }

    #[test]
    fn payload_action_type_maps_correctly() {
        let payload = LocalPrefixCacheEventPayload {
            kind: LocalPrefixCacheKind::CacheIneligible,
            head_prefill_tokens: 0,
            tail_prefill_tokens: 0,
            eligibility_reason: "prefix_kv_ineligible".into(),
            ts_ms: 1_700_000_000_000,
        };
        let wrapped = SsbPayload::LocalPrefixCacheEvent(payload);
        let action_type = wrapped.action_type();
        assert_eq!(
            action_type.as_sql_str(),
            "local_prefix_cache_event"
        );
    }
}
