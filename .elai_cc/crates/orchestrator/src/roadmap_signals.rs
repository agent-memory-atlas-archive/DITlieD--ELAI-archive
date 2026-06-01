//! E-38 (SCHEMA_VERSION 175) -- 2026 AI roadmap signals substrate.
//!
//! This module owns the producers for two of the five
//! [`crate::ssb::RoadmapSignalKind`] arms:
//!   - `MrcrUnverified` -- emitted at backend init when a runtime
//!     declares >= 100K context tokens without a measured
//!     `mrcr_v2_score`. Grounds the report's "1M-token-lie" finding
//!     (Claude Opus 4.7 advertises 1M but scores 32.2% MRCR v2; Llama 4
//!     Scout 10M scores 15.6%).
//!   - `PostureDocStale` -- emitted at session start when
//!     `docs/strategy/ELAI-2026-POSTURE.md` mtime is older than 90 days
//!     (per [`crate::ssb::POSTURE_DOC_STALE_DAYS`]).
//!
//! The other three arms (`ComplianceBlocked` / `ComplianceVerified` from
//! [`crate::compliance`], `McpTransportVersionLogged` from
//! [`elai_plugins::mcp_bridge::DefaultMcpBridge::new`]) live with their
//! producers.
//!
//! All emission is wire-dark by default; flip
//! `ELAI_AI_ROADMAP_SIGNALS_ENABLE=1` at process start to switch the
//! producers on. See [`crate::ssb::ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV`].
//!
//! Grounding (every cite WebSearch-verified May 2026):
//!   - arXiv:2409.12640 -- MRCR v2 multi-needle long-context benchmark.
//!   - arXiv:2510.12357 -- MoBiLE consumer-GPU MoE inference (Oct 2025).
//!   - arXiv:2602.03495 -- DALI workload-aware MoE offloading framework
//!     (Feb 2026; documents the expert-offload design space for
//!     `docs/strategy/MoE-on-Ampere-tier.md`).
//!   - medium.com/@administrator_54541/the-1-million-token-lie (MRCR v2
//!     numbers: Opus 4.7 32.2% at 1M; Scout 15.6% at 10M).
//!   - artificialintelligenceact.eu/implementation-timeline (high-risk
//!     Annex III postponed to Dec 2 2027 per Digital Omnibus May 7 2026).
//!
//! See `workflow/reasoning/E-38-decisions.md`.

use std::path::Path;
use std::time::UNIX_EPOCH;

use elai_core::clock::{ClockProvider, OsClock};

use crate::ssb::{
    AiRoadmapSignalEventPayload, ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV,
    MRCR_UNVERIFIED_CONTEXT_THRESHOLD, POSTURE_DOC_STALE_DAYS, RoadmapSignalKind,
};

/// Returns true when the E-38 emission env-gate is set to `"1"` or
/// `"true"` (case-insensitive). Mirrors the C-35 `ELAI_TOOL_GATE_OFF`
/// truthiness parser pattern.
#[must_use]
pub fn roadmap_signals_enabled() -> bool {
    match std::env::var(ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV) {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "yes"
        }
        Err(_) => false,
    }
}

/// Current wall-clock millis since UNIX_EPOCH. Used by the producers
/// when no [`elai_core::clock::ClockProvider`] is injected (e.g. the
/// dashboard tile spawning a one-shot posture-staleness probe). Returns
/// `i64::MAX` if the system clock is broken; the caller is expected to
/// emit the row anyway since downstream consumers tolerate clamped
/// timestamps.
fn now_ms_fallback() -> i64 {
    i64::try_from(OsClock.now_ms()).unwrap_or(i64::MAX)
}

/// Build an [`AiRoadmapSignalEventPayload`] for the
/// [`RoadmapSignalKind::MrcrUnverified`] arm.
///
/// Callers (typically the runtime adapter init path) invoke this when
/// they detect a backend declares `declared_max_context_tokens` >=
/// [`MRCR_UNVERIFIED_CONTEXT_THRESHOLD`] without a measured
/// `mrcr_v2_score`. The payload is shipped to SSB via
/// `crate::ssb::Ssb::write_payload` only when [`roadmap_signals_enabled`]
/// returns true; this function does NOT itself touch the wire.
#[must_use]
pub fn build_mrcr_unverified_payload(
    backend_name: impl Into<String>,
    declared_context_tokens: u32,
    mrcr_score_label: impl Into<String>,
) -> AiRoadmapSignalEventPayload {
    AiRoadmapSignalEventPayload {
        kind: RoadmapSignalKind::MrcrUnverified,
        plan_session_id: String::new(),
        assertion_label: String::new(),
        hint: String::new(),
        assertions_checked: 0,
        mcp_transport_current: String::new(),
        mcp_transport_target: String::new(),
        backend_name: backend_name.into(),
        declared_context_tokens,
        mrcr_score_label: mrcr_score_label.into(),
        posture_doc_path: String::new(),
        posture_doc_mtime_ms: 0,
        age_days: 0,
        details_json: serde_json::json!({
            "threshold": MRCR_UNVERIFIED_CONTEXT_THRESHOLD,
            "paper_arxiv_id": "2409.12640",
            "report_url": "medium.com/@administrator_54541/the-1-million-token-lie"
        }),
        ts_ms: now_ms_fallback(),
    }
}

/// Probe the posture doc's mtime and return a payload when it crosses
/// [`POSTURE_DOC_STALE_DAYS`]. Returns `None` when the doc is fresh,
/// missing, or its mtime cannot be read (the missing case is handled by
/// returning `None` rather than emitting a noisy row -- the strategy doc
/// is itself authored by E-38 and absence is a separate signal).
#[must_use]
pub fn check_posture_doc_age(
    posture_doc_path: &Path,
) -> Option<AiRoadmapSignalEventPayload> {
    let meta = std::fs::metadata(posture_doc_path).ok()?;
    let mtime = meta.modified().ok()?;
    let mtime_ms = mtime
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .ok()?;
    let now = now_ms_fallback();
    let age_ms = now.saturating_sub(mtime_ms).max(0);
    let age_days_u64 = (age_ms / 86_400_000).max(0) as u64;
    let age_days = u32::try_from(age_days_u64).unwrap_or(u32::MAX);
    if age_days < POSTURE_DOC_STALE_DAYS {
        return None;
    }
    Some(AiRoadmapSignalEventPayload {
        kind: RoadmapSignalKind::PostureDocStale,
        plan_session_id: String::new(),
        assertion_label: String::new(),
        hint: String::new(),
        assertions_checked: 0,
        mcp_transport_current: String::new(),
        mcp_transport_target: String::new(),
        backend_name: String::new(),
        declared_context_tokens: 0,
        mrcr_score_label: String::new(),
        posture_doc_path: posture_doc_path.to_string_lossy().into_owned(),
        posture_doc_mtime_ms: mtime_ms,
        age_days,
        details_json: serde_json::json!({
            "threshold_days": POSTURE_DOC_STALE_DAYS,
            "refresh_target": "every-90-days-or-on-major-roadmap-shift"
        }),
        ts_ms: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mrcr_unverified_payload_carries_backend_metadata() {
        let p = build_mrcr_unverified_payload("claude-opus-4-7", 1_000_000, "vendor-claim-only");
        assert_eq!(p.kind, RoadmapSignalKind::MrcrUnverified);
        assert_eq!(p.backend_name, "claude-opus-4-7");
        assert_eq!(p.declared_context_tokens, 1_000_000);
        assert_eq!(p.mrcr_score_label, "vendor-claim-only");
        assert_eq!(p.assertions_checked, 0);
        assert!(p.posture_doc_path.is_empty());
    }

    #[test]
    fn check_posture_doc_age_returns_none_for_missing_file() {
        let nonexistent = Path::new("Z:/this/path/does/not/exist/posture.md");
        assert!(check_posture_doc_age(nonexistent).is_none());
    }

    #[test]
    fn roadmap_signals_disabled_when_env_unset() {
        // SAFETY: removing the var if it happens to be set in the env.
        // SAFETY: env mutation in single-threaded test scope.
        unsafe {
            std::env::remove_var(ELAI_AI_ROADMAP_SIGNALS_ENABLE_ENV);
        }
        assert!(!roadmap_signals_enabled());
    }
}
