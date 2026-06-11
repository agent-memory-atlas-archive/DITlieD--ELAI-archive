//! E-23 -- trust telemetry plane.
//!
//! Aggregates productivity / verification / trust signals from existing SSB
//! rows into a single TrustReport surfaced through the dashboard-v2
//! TrustTelemetryPanel. The plan body explicitly states "no new SSB row
//! schema needed for the metrics layer (only the visualization)" -- this
//! module reads, never produces, except for the [`DeveloperDiscourseEvent`]
//! roll-up row that records that an aggregation ran. That row is
//! wire-dark unless `ELAI_DEVELOPER_DISCOURSE_ENABLE` is set to `"1"` or
//! `"true"`, mirroring E-22 / E-43 / E-10 / E-19 / E-55 gate semantics.
//!
//! Signal layout (5 cards in the Svelte panel):
//!   1. `tasks_per_hour` -- count of phase_transition rows into COMPLETE
//!      divided by elapsed session-hours (productivity proxy).
//!   2. `findings_per_task` -- auditor `finding` rows / completed task
//!      (verification density).
//!   3. `revert_rate` -- `file_write` rows tagged with `revert: true`
//!      vs total file_write rows (rollback signal).
//!   4. `ungrounded_count` -- counter from mcp_think calls with empty
//!      `grounding` field (read out of the `claim` SSB action_type when
//!      the payload's `grounding` field is empty). HARD RULE per
//!      CLAUDE.md: ungrounded reasoning is a defect; this count is the
//!      only visible counter the user has.
//!   5. `stagnation_dist` -- HashMap<pattern_label, count> read from the
//!      stagnation_spinning_detected + stagnation_oscillation_detected
//!      rows (C-2 / B-22b producer-wire).
//!
//! See `workflow/reasoning/E-23-decisions.md` for the design tradeoffs +
//! the rejection of a counter-measurement claim to METR (paired-task data
//! is missing; this is a BASELINE artifact, not a "speedup proven" claim).

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::anti_spiral_extended::StagnationPattern;
use crate::ssb::{
    DeveloperDiscourseEventPayload, DiscourseEventKind, ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV,
};

/// One row of the read-side SSB row shape consumed by [`aggregate_session`].
/// The dashboard already speaks JSON, so we deliberately keep the input
/// shape as a thin DTO over `SsbViewer::RecentRow` -- callers translate
/// the viewer row into this struct so this module stays infrastructure-
/// free and trivially unit-testable.
#[derive(Debug, Clone)]
pub struct TrustRowInput {
    /// Action-type wire tag (e.g. `"phase_transition"`).
    pub action_type: String,
    /// Wall-clock at row write, milliseconds since UNIX_EPOCH.
    pub ts_ms: i64,
    /// Free-form payload JSON the producer wrote.
    pub payload_json: serde_json::Value,
}

/// Productivity / verification / trust roll-up consumed by the dashboard.
///
/// All counts are session-scoped. The aggregation is a read-only fold
/// over [`TrustRowInput`]s; the caller is expected to pull rows from
/// `SsbViewer::recent_rows_of_type` for the action-types this module
/// actually reads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrustReport {
    /// Completed-task rate (proxy for productivity). Computed as
    /// `complete_count / elapsed_hours`. Zero when no completion rows
    /// are present.
    pub tasks_per_hour: f64,
    /// Auditor finding rows per completed task. Zero when no
    /// completions are recorded.
    pub findings_per_task: f64,
    /// Reverted file-write ratio (rollback signal). Zero when no
    /// file_write rows are present.
    pub revert_rate: f64,
    /// Count of ungrounded claim rows (mcp_think with empty
    /// `grounding`). HARD RULE: this should trend to zero.
    pub ungrounded_count: u64,
    /// Per-pattern stagnation distribution. Keys are the snake_case
    /// labels emitted by [`StagnationPattern::as_str`].
    pub stagnation_dist: FxHashMap<String, u64>,
}

impl TrustReport {
    /// Empty report -- the default when the gate is wire-dark or no
    /// rows are present.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            tasks_per_hour: 0.0,
            findings_per_task: 0.0,
            revert_rate: 0.0,
            ungrounded_count: 0,
            stagnation_dist: FxHashMap::default(),
        }
    }
}

/// E-23 -- env-gate check. Returns `true` when the user has flipped
/// `ELAI_DEVELOPER_DISCOURSE_ENABLE` to `"1"` / `"true"` (case-
/// insensitive). Any other value (including unset) leaves emission off.
/// Mirrors the E-22 / E-43 / E-19 / E-55 gate-check shape.
#[must_use]
pub fn discourse_enabled() -> bool {
    match std::env::var(ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV) {
        Ok(v) => {
            let lower = v.to_lowercase();
            lower == "1" || lower == "true"
        }
        Err(_) => false,
    }
}

/// Aggregate trust signals from a flat list of SSB rows.
///
/// The caller is expected to have pre-filtered the rows for the active
/// session (the SSB viewer can supply per-session rows via the existing
/// `recent_rows_of_type` shape). This function is intentionally pure --
/// no I/O, no clock reads, no writes -- so the contract test can drive
/// it with a synthetic row set.
///
/// `elapsed_hours` is the wall-clock span the rows cover, used as the
/// denominator for `tasks_per_hour`. Pass a small floor (e.g. 1.0/60.0
/// minute) to avoid division-by-zero on very fresh sessions.
#[must_use]
pub fn aggregate_session(rows: &[TrustRowInput], elapsed_hours: f64) -> TrustReport {
    if !discourse_enabled() {
        // Wire-dark default: emit an empty report so the dashboard panel
        // renders a sensible "telemetry disabled" hint rather than
        // showing stale numbers. Mirrors E-22 / E-43 gate semantics.
        return TrustReport::empty();
    }

    let mut completions: u64 = 0;
    let mut findings: u64 = 0;
    let mut file_writes: u64 = 0;
    let mut reverts: u64 = 0;
    let mut ungrounded: u64 = 0;
    let mut stagnation_dist: FxHashMap<String, u64> = FxHashMap::default();

    // Seed the four canonical stagnation labels so the dashboard
    // renders a stable grid even when no stagnation events landed yet.
    for p in [
        StagnationPattern::Spinning,
        StagnationPattern::Oscillation,
        StagnationPattern::NoDrift,
        StagnationPattern::DiminishingReturns,
    ] {
        stagnation_dist.insert(p.as_str().to_owned(), 0);
    }

    for row in rows {
        match row.action_type.as_str() {
            // phase_transition rows carry a `to` field. We count the
            // transitions INTO COMPLETE as the productivity numerator.
            "phase_transition" => {
                let to = row
                    .payload_json
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if to.eq_ignore_ascii_case("COMPLETE") {
                    completions = completions.saturating_add(1);
                }
            }
            "finding" => {
                findings = findings.saturating_add(1);
            }
            "file_write" => {
                file_writes = file_writes.saturating_add(1);
                let is_revert = row
                    .payload_json
                    .get("revert")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if is_revert {
                    reverts = reverts.saturating_add(1);
                }
            }
            "claim" => {
                let grounding = row
                    .payload_json
                    .get("grounding")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if grounding.is_empty() {
                    ungrounded = ungrounded.saturating_add(1);
                }
            }
            // C-2 / B-22b stagnation rows: the `pattern` field on the
            // payload is the snake_case label we key the distribution
            // on. Same shape as the dashboard get_stagnation_events
            // command consumes.
            "stagnation_spinning_detected" | "stagnation_oscillation_detected" => {
                if let Some(label) = row
                    .payload_json
                    .get("pattern")
                    .and_then(|v| v.as_str())
                {
                    let entry = stagnation_dist
                        .entry(label.to_owned())
                        .or_insert(0);
                    *entry = entry.saturating_add(1);
                }
            }
            _ => {
                // Ignore other action types -- read-only aggregation.
            }
        }
    }

    let denom = elapsed_hours.max(1.0_f64 / 60.0);
    let tasks_per_hour = if completions == 0 {
        0.0
    } else {
        completions as f64 / denom
    };
    let findings_per_task = if completions == 0 {
        0.0
    } else {
        findings as f64 / completions as f64
    };
    let revert_rate = if file_writes == 0 {
        0.0
    } else {
        reverts as f64 / file_writes as f64
    };

    TrustReport {
        tasks_per_hour,
        findings_per_task,
        revert_rate,
        ungrounded_count: ungrounded,
        stagnation_dist,
    }
}

/// Build a [`DeveloperDiscourseEventPayload`] of kind
/// [`DiscourseEventKind::TrustTelemetryAggregated`] from a [`TrustReport`].
///
/// The writer side (dashboard-v2 Tauri command) takes this payload and
/// hands it to the SSB writer; the gate is checked at write time by the
/// caller via [`discourse_enabled`]. Kept here so the payload shape +
/// the report shape stay in lockstep.
#[must_use]
pub fn build_aggregated_payload(
    report: &TrustReport,
    session_id: Option<String>,
    ts_ms: i64,
) -> DeveloperDiscourseEventPayload {
    DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::TrustTelemetryAggregated,
        session_id,
        tasks_per_hour: report.tasks_per_hour,
        findings_per_task: report.findings_per_task,
        revert_rate: report.revert_rate,
        ungrounded_count: report.ungrounded_count,
        prev_badge: String::new(),
        next_badge: String::new(),
        token_budget_pct: 0.0,
        threshold_label: String::new(),
        details_json: serde_json::json!({
            "stagnation_dist": &report.stagnation_dist,
            "caller_site": "aggregate_session",
        }),
        ts_ms,
    }
}

/// Build a [`DeveloperDiscourseEventPayload`] of kind
/// [`DiscourseEventKind::RuntimeChoiceBadgeEmitted`].
///
/// Wired into the dashboard-v2 status-row badge transition when the
/// CommandWindow detects a `runtime_fallback` SSB row whose `succeeded`
/// backend crosses the local / cloud partition.
#[must_use]
pub fn build_badge_payload(
    prev_badge: &str,
    next_badge: &str,
    session_id: Option<String>,
    ts_ms: i64,
) -> DeveloperDiscourseEventPayload {
    DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::RuntimeChoiceBadgeEmitted,
        session_id,
        tasks_per_hour: 0.0,
        findings_per_task: 0.0,
        revert_rate: 0.0,
        ungrounded_count: 0,
        prev_badge: prev_badge.to_owned(),
        next_badge: next_badge.to_owned(),
        token_budget_pct: 0.0,
        threshold_label: String::new(),
        details_json: serde_json::json!({
            "caller_site": "build_badge_payload",
        }),
        ts_ms,
    }
}

/// Build a [`DeveloperDiscourseEventPayload`] of kind
/// [`DiscourseEventKind::ContextStateWarning`] from a token-budget
/// percentage. The `threshold_label` is derived deterministically.
#[must_use]
pub fn build_context_warning_payload(
    token_budget_pct: f32,
    session_id: Option<String>,
    ts_ms: i64,
) -> DeveloperDiscourseEventPayload {
    let threshold_label = if token_budget_pct >= crate::ssb::CONTEXT_BUDGET_RED_PCT {
        "red"
    } else if token_budget_pct >= crate::ssb::CONTEXT_BUDGET_YELLOW_PCT {
        "yellow"
    } else {
        ""
    };
    DeveloperDiscourseEventPayload {
        kind: DiscourseEventKind::ContextStateWarning,
        session_id,
        tasks_per_hour: 0.0,
        findings_per_task: 0.0,
        revert_rate: 0.0,
        ungrounded_count: 0,
        prev_badge: String::new(),
        next_badge: String::new(),
        token_budget_pct,
        threshold_label: threshold_label.to_owned(),
        details_json: serde_json::json!({
            "caller_site": "build_context_warning_payload",
        }),
        ts_ms,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Process-global env var means tests must serialize the gate flip.
    // Without a lock parallel tests race on the env var state and the
    // empty-report assertion flaps.
    static GATE_LOCK: Mutex<()> = Mutex::new(());

    fn row(action: &str, payload: serde_json::Value, ts_ms: i64) -> TrustRowInput {
        TrustRowInput {
            action_type: action.to_owned(),
            ts_ms,
            payload_json: payload,
        }
    }

    fn enable_gate() {
        std::env::set_var(ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV, "1");
    }

    fn disable_gate() {
        std::env::remove_var(ELAI_DEVELOPER_DISCOURSE_ENABLE_ENV);
    }

    #[test]
    fn empty_report_when_gate_off() {
        let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        disable_gate();
        let rows = vec![row(
            "phase_transition",
            serde_json::json!({"to": "COMPLETE"}),
            1_000,
        )];
        let r = aggregate_session(&rows, 1.0);
        assert_eq!(r, TrustReport::empty());
    }

    #[test]
    fn aggregates_each_signal_with_gate_on() {
        let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        enable_gate();
        let rows = vec![
            row(
                "phase_transition",
                serde_json::json!({"to": "COMPLETE"}),
                1,
            ),
            row(
                "phase_transition",
                serde_json::json!({"to": "COMPLETE"}),
                2,
            ),
            row("finding", serde_json::json!({}), 3),
            row("finding", serde_json::json!({}), 4),
            row("finding", serde_json::json!({}), 5),
            row("file_write", serde_json::json!({"revert": false}), 6),
            row("file_write", serde_json::json!({"revert": true}), 7),
            row("claim", serde_json::json!({"grounding": ""}), 8),
            row("claim", serde_json::json!({"grounding": "file:foo"}), 9),
            row(
                "stagnation_spinning_detected",
                serde_json::json!({"pattern": "spinning"}),
                10,
            ),
            row(
                "stagnation_oscillation_detected",
                serde_json::json!({"pattern": "oscillation"}),
                11,
            ),
        ];
        let r = aggregate_session(&rows, 1.0);
        disable_gate();
        assert!((r.tasks_per_hour - 2.0).abs() < f64::EPSILON);
        assert!((r.findings_per_task - 1.5).abs() < f64::EPSILON);
        assert!((r.revert_rate - 0.5).abs() < f64::EPSILON);
        assert_eq!(r.ungrounded_count, 1);
        assert_eq!(r.stagnation_dist.get("spinning"), Some(&1));
        assert_eq!(r.stagnation_dist.get("oscillation"), Some(&1));
        // canonical seed labels still present even when count is zero
        assert!(r.stagnation_dist.contains_key("no_drift"));
        assert!(r.stagnation_dist.contains_key("diminishing_returns"));
    }

    #[test]
    fn elapsed_hours_floor_avoids_div_by_zero() {
        let _lock = GATE_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        enable_gate();
        let rows = vec![row(
            "phase_transition",
            serde_json::json!({"to": "COMPLETE"}),
            0,
        )];
        // Pass 0.0 elapsed -- aggregate_session must floor to 1/60 min.
        let r = aggregate_session(&rows, 0.0);
        disable_gate();
        // 1 completion / (1/60 hour) = 60 tasks/hour upper bound. The
        // assertion checks finiteness; the exact value is brittle if the
        // floor constant changes.
        assert!(r.tasks_per_hour.is_finite());
        assert!(r.tasks_per_hour > 0.0);
    }

    #[test]
    fn build_payload_round_trip() {
        let report = TrustReport {
            tasks_per_hour: 2.5,
            findings_per_task: 0.8,
            revert_rate: 0.1,
            ungrounded_count: 3,
            stagnation_dist: {
                let mut m = FxHashMap::default();
                m.insert("spinning".into(), 2);
                m
            },
        };
        let p = build_aggregated_payload(&report, Some("s-test".into()), 1_700_000_171_500);
        assert_eq!(p.kind, DiscourseEventKind::TrustTelemetryAggregated);
        assert_eq!(p.session_id.as_deref(), Some("s-test"));
        assert!((p.tasks_per_hour - 2.5).abs() < f64::EPSILON);
        assert_eq!(p.ungrounded_count, 3);
    }

    #[test]
    fn context_warning_threshold_labels() {
        let red = build_context_warning_payload(90.0, None, 0);
        assert_eq!(red.threshold_label, "red");
        let yellow = build_context_warning_payload(75.0, None, 0);
        assert_eq!(yellow.threshold_label, "yellow");
        let clean = build_context_warning_payload(20.0, None, 0);
        assert_eq!(clean.threshold_label, "");
    }

    #[test]
    fn badge_payload_records_transition() {
        let p = build_badge_payload("local", "cloud_billed", Some("s".into()), 42);
        assert_eq!(p.prev_badge, "local");
        assert_eq!(p.next_badge, "cloud_billed");
        assert_eq!(p.kind, DiscourseEventKind::RuntimeChoiceBadgeEmitted);
    }

    // E-23 off-agent artifact smoke: exercises the write path that
    // `get_trust_telemetry` (commands.rs) now performs and reads the row back
    // from a temp Ssb to prove the emit is not wire-dark.
    //
    // This test is intentionally in-crate (not in crates/orchestrator/tests/)
    // because the tests/ integration harness does not link here due to
    // pre-existing ort/ndarray rlib conflicts.
    #[tokio::test]
    async fn trust_telemetry_aggregated_ssb_emit_round_trip() {
        use crate::ssb::{Ssb, SsbActionType, SsbConfig, SsbPayload, SsbReader};
        use elai_core::clock::OsClock;
        use std::sync::Arc;
        use tempfile::TempDir;

        let dir = TempDir::new().expect("tempdir");
        let cfg = SsbConfig {
            db_path: dir.path().join("trust_telem_smoke.db"),
            pool_size: None,
            project_id: "e23-smoke".into(),
            session_id: "s-e23-smoke".into(),
        };
        let ssb = Ssb::new(cfg, Arc::new(OsClock)).expect("ssb init");

        let report = TrustReport {
            tasks_per_hour: 1.5,
            findings_per_task: 0.4,
            revert_rate: 0.05,
            ungrounded_count: 1,
            stagnation_dist: {
                let mut m = FxHashMap::default();
                m.insert("spinning".into(), 1u64);
                m
            },
        };
        let payload = SsbPayload::DeveloperDiscourseEvent(build_aggregated_payload(
            &report,
            Some("s-e23-smoke".into()),
            1_700_000_000_000,
        ));

        ssb.write_payload("trust-telem-smoke", 0, payload, vec![])
            .await
            .expect("write_payload must succeed");

        let rows = ssb
            .query_by_type(SsbActionType::DeveloperDiscourseEvent)
            .expect("query_by_type must succeed");

        assert!(!rows.is_empty(), "expected at least one DeveloperDiscourseEvent row");
        let entry = &rows[0];
        assert_eq!(entry.action_type, SsbActionType::DeveloperDiscourseEvent);

        let decoded: SsbPayload = rmp_serde::from_slice(&entry.payload)
            .expect("payload must deserialize");
        match decoded {
            SsbPayload::DeveloperDiscourseEvent(p) => {
                assert_eq!(
                    p.kind,
                    DiscourseEventKind::TrustTelemetryAggregated,
                    "kind must be TrustTelemetryAggregated"
                );
                assert!(
                    (p.tasks_per_hour - 1.5).abs() < f64::EPSILON,
                    "tasks_per_hour must round-trip"
                );
            }
            other => panic!("expected DeveloperDiscourseEvent, got {:?}", other),
        }
    }
}
