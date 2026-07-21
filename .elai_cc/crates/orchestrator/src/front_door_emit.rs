// J-32 slice-C: IntentRouted SSB-emit helper (front-door routing observability).
//
// Mirrors auth_broker_emit.rs / providers_byok_ssb.rs: a thin orchestrator-layer
// shim that maps a FrontDoorDecision into an IntentRouted SSB row and fires it
// best-effort from a sync context via tokio::runtime::Handle::try_current().
// A missed SSB row is never a correctness failure (R1 fail-closed is the gate's
// job, not the emit path's).
//
// slice-D wired the repl bare-message path (classify_bare_message_with_ssb).
// start_chat_stream wiring is still a future slice; until it lands, the
// producer-without-consumer R6/R7 leg for this helper is satisfied by the
// repl bare-message call site (cli/src/repl.rs).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::front_door::{FrontDoorDecision, FrontDoorLane};
use crate::gate::RiskLevel;
use crate::ssb::{IntentRoutedKind, IntentRoutedPayload, Ssb, SsbPayload};

pub fn emit_intent_routed(ssb: &Arc<Ssb>, decision: &FrontDoorDecision, risk: RiskLevel) {
    let kind = if decision.slash_bypass.is_some() {
        IntentRoutedKind::Bypassed
    } else {
        IntentRoutedKind::Classified
    };
    let ts_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);
    let payload = SsbPayload::IntentRouted(IntentRoutedPayload {
        lane: decision.lane,
        tier: decision.tier,
        confidence: decision.confidence,
        arg_spans: decision.arg_spans.clone(),
        kind,
        risk,
        ts_ms,
    });
    let ssb = Arc::clone(ssb);
    let emit = async move {
        if let Err(e) = ssb.write_payload("front_door", 0, payload, vec![]).await {
            tracing::warn!(
                front_door.ssb_err = %e,
                "IntentRouted SSB emit failed (routing unaffected)"
            );
        }
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(emit);
        }
        Err(_) => {
            // No async runtime (the repl bare-message dispatch arm is sync,
            // same shape as HandoffLifecycle / C-30 emit_project_detection_signals).
            // Build a transient current_thread runtime and block_on the write
            // so the IntentRouted row still lands; do NOT spawn-and-drop.
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(emit);
            }
        }
    }
}

/// Map a routed lane onto a RiskLevel for the IntentRouted SSB row.
///
/// Grounded in J-32_elai_frontdoor_router.md:50-52 (read-only lanes auto-run
/// chipless; MEDIUM chip) and the observable_effect fixtures at lines 119-124
/// ("fix the failing test in sum.rs" = Fix lane, chip-visible = MEDIUM;
/// "wipe the old sessions" = HIGH destructive). The plan gives no explicit
/// lane->risk table; this is the smallest closed map grounded in the plan
/// text:
///   Chat / Research / Explore -> Low  (read-only, auto-run chipless, line 50)
///   Task / Fix               -> Medium (mutating, chip in interactive, line 50)
///
/// HIGH is RECON_NEEDED: a destructive-content matcher (the "wipe" fixture) is
/// a future slice and is NOT invented here. Do NOT add a HIGH branch without
/// grounding it in a real destructive matcher.
pub fn risk_for_lane(lane: FrontDoorLane) -> RiskLevel {
    match lane {
        FrontDoorLane::Chat | FrontDoorLane::Research | FrontDoorLane::Explore => RiskLevel::Low,
        FrontDoorLane::Task | FrontDoorLane::Fix => RiskLevel::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::front_door::{FrontDoorRouter, FrontDoorLane, FrontDoorTier};
    use crate::ssb::{SsbActionType, SsbConfig, SsbReader, decode_ssb_payload};
    use elai_core::clock::{ClockProvider, OsClock};
    use tempfile::TempDir;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn j32_slice_c_classified_row_lands_for_fix_input() {
        let dir = TempDir::new().expect("tempdir");
        let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);
        let ssb = Arc::new(
            Ssb::new(
                SsbConfig {
                    db_path: dir.path().join("j32-slice-c-ssb.db"),
                    pool_size: Some(2),
                    project_id: "j32-slice-c".into(),
                    session_id: "j32-slice-c".into(),
                },
                clock,
            )
            .expect("ssb open"),
        );

        let router = FrontDoorRouter;
        let decision = router.resolve("fix the failing test in sum.rs", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Fix);
        assert_eq!(decision.tier, FrontDoorTier::T1);

        emit_intent_routed(&ssb, &decision, risk_for_lane(decision.lane));

        tokio::time::sleep(std::time::Duration::from_millis(80)).await;

        let rows = ssb
            .query_by_type(SsbActionType::IntentRouted)
            .expect("query_by_type IntentRouted");
        assert!(
            !rows.is_empty(),
            "expected >= 1 IntentRouted row; R1 missing evidence = FAIL"
        );
        let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
        match decoded {
            SsbPayload::IntentRouted(p) => {
                assert_eq!(p.kind, IntentRoutedKind::Classified);
                assert_eq!(p.lane, FrontDoorLane::Fix);
                assert_eq!(p.tier, FrontDoorTier::T1);
                assert_eq!(p.risk, RiskLevel::Medium);
                assert!(!p.arg_spans.is_empty());
                assert!(p.ts_ms > 0);
            }
            other => panic!("expected IntentRouted; got {:?}", other),
        }
    }
}
