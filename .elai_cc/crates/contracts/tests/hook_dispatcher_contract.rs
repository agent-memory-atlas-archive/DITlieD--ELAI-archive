//! Batch 1A — hook dispatcher contract tests.
//!
//! Covers the public surface described in
//! `docs/mvp-wiring-audit-2026-04-18.md` section 3 + MVP Batch 1A task:
//!   1. Trait shape is dyn-safe — `Arc<dyn HookDispatcher>` holds.
//!   2. `TaskRunnerConfig::new_minimal` leaves `hook_dispatcher = None` so
//!      pre-batch-1a behavior is preserved.
//!   3. `InProcessHookDispatcher::new(None)` returns empty findings on every
//!      dispatch method (no-tiered-backend path).
//!   4. `ELAI_HOOKS_DISABLED` env var honored as a hard-off switch.
//!   5. `HookEventPayload` round-trips through `SsbPayload` msgpack encoding
//!      and writes to the SSB through the standard `write_payload` path.
//!   6. `HookEvent` action_type renders via `ssb_action_type_from_str_public`
//!      (schema migration 9 → 10 preserved existing discriminants).
//!   7. `HookFinding` verdict predicates (`is_block` / `is_warn`) correctly
//!      classify findings.

use std::path::PathBuf;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::hook_dispatcher::{
    HookDispatcher, HookFinding, HookInvocation, HookPoint, HookTier, HookVerdictKind,
    InProcessHookDispatcher, ELAI_HOOKS_DISABLED_ENV, HOOK_EVENT_CATEGORY,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, HookEventPayload, Ssb, SsbActionType, SsbConfig, SsbPayload,
    SsbReader,
};
use elai_orchestrator::TaskRunnerConfig;
use tempfile::TempDir;
use tokio::sync::Mutex as _AsyncMutex;

// ---------------------------------------------------------------------------
// Contract 1: trait is dyn-safe
// ---------------------------------------------------------------------------

#[test]
fn contract_hook_dispatcher_is_dyn_safe() {
    let d: Arc<dyn HookDispatcher> = Arc::new(InProcessHookDispatcher::new(None));
    assert!(Arc::strong_count(&d) >= 1);
}

// ---------------------------------------------------------------------------
// Contract 2: TaskRunnerConfig::new_minimal leaves hook_dispatcher = None
// ---------------------------------------------------------------------------

#[test]
fn contract_new_minimal_leaves_hook_dispatcher_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(
        cfg.hook_dispatcher.is_none(),
        "new_minimal must keep hook_dispatcher=None so the dashboard is the only opt-in site"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: no-backend dispatcher returns empty findings for every point
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_no_backend_returns_empty_findings() {
    let d: Arc<dyn HookDispatcher> = Arc::new(InProcessHookDispatcher::new(None));
    let invocation = HookInvocation::for_pre_tool_use("baby_step_pre_gate", "noop", "step_1");
    assert!(d.dispatch_pre_tool_use(invocation).await.expect("ok").is_empty());

    let invocation = HookInvocation::for_post_tool_use(
        "file_write",
        "fn main() {}",
        "src/main.rs",
        "step_1",
    );
    assert!(d.dispatch_post_tool_use(invocation).await.expect("ok").is_empty());

    let invocation = HookInvocation::for_task_level("task_stop", "task_7");
    assert!(d.dispatch_stop(invocation).await.expect("ok").is_empty());

    let invocation = HookInvocation::for_task_level("pre_compact", "task_7");
    assert!(d.dispatch_pre_compact(invocation).await.expect("ok").is_empty());
}

// ---------------------------------------------------------------------------
// Contract 4: ELAI_HOOKS_DISABLED env var is honored
// ---------------------------------------------------------------------------

#[tokio::test]
async fn contract_env_var_disables_dispatcher() {
    // Env-var scope is per-process; use a unique sentinel and restore.
    let prev = std::env::var(ELAI_HOOKS_DISABLED_ENV).ok();
    // SAFETY: these helpers are safe on all platforms used by this crate.
    // The test crate is not parallel-unsafe — cargo test default is single
    // thread per test file when touching process-global env, but this is
    // deliberately narrow and immediately restored.
    std::env::set_var(ELAI_HOOKS_DISABLED_ENV, "1");
    let d = InProcessHookDispatcher::new(None);
    assert!(d.is_disabled(), "env=1 must toggle the disabled bit");
    // Even if a backend were installed, is_disabled short-circuits analyze.

    std::env::set_var(ELAI_HOOKS_DISABLED_ENV, "0");
    let d = InProcessHookDispatcher::new(None);
    assert!(!d.is_disabled(), "env=0 must NOT toggle disabled");

    // Restore
    match prev {
        Some(v) => std::env::set_var(ELAI_HOOKS_DISABLED_ENV, v),
        None => std::env::remove_var(ELAI_HOOKS_DISABLED_ENV),
    }
}

// ---------------------------------------------------------------------------
// Contract 5: HookEventPayload round-trips through SsbPayload msgpack
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_hook_event_payload_roundtrips() {
    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "hook-test".into(),
        session_id: "hook-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let payload = SsbPayload::HookEvent(HookEventPayload {
        tier: HookTier::T1.as_str().to_string(),
        verdict: HookVerdictKind::Block.as_str().to_string(),
        rule_id: Some("rule-42".to_string()),
        category: HOOK_EVENT_CATEGORY.to_string(),
        message: "example block".to_string(),
        step_id: "baby_step_7".to_string(),
        hook_point: HookPoint::PreToolUse.as_str().to_string(),
    });

    // Sanity: discriminant roundtrip via action_type() matches the string form
    // we registered in the SQL CHECK constraint.
    assert_eq!(payload.action_type(), SsbActionType::HookEvent);

    ssb.write_payload("task_runner", 0, payload.clone(), Vec::new())
        .await
        .expect("write hook_event");

    let rows = ssb
        .query_by_type(SsbActionType::HookEvent)
        .expect("query hook_event rows");
    assert_eq!(rows.len(), 1, "exactly one HookEvent row persisted");
    let decoded: SsbPayload = rmp_serde::from_slice(&rows[0].payload).expect("decode payload");
    match decoded {
        SsbPayload::HookEvent(p) => {
            assert_eq!(p.tier, "T1");
            assert_eq!(p.verdict, "Block");
            assert_eq!(p.rule_id.as_deref(), Some("rule-42"));
            assert_eq!(p.category, HOOK_EVENT_CATEGORY);
            assert_eq!(p.step_id, "baby_step_7");
            assert_eq!(p.hook_point, "PreToolUse");
        }
        other => panic!("expected HookEvent, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 6: ssb_action_type_from_str_public knows "hook_event"
// ---------------------------------------------------------------------------

#[test]
fn contract_hook_event_action_type_string_roundtrip() {
    let at = ssb_action_type_from_str_public("hook_event").expect("hook_event must parse");
    assert_eq!(at, SsbActionType::HookEvent);
    assert_eq!(at.as_sql_str(), "hook_event");
}

// ---------------------------------------------------------------------------
// Contract 7: HookFinding verdict predicates
// ---------------------------------------------------------------------------

#[test]
fn contract_hook_finding_verdict_predicates() {
    let block = HookFinding {
        tier: HookTier::T2,
        verdict: HookVerdictKind::Block,
        rule_id: Some("r".into()),
        category: HOOK_EVENT_CATEGORY.into(),
        message: "m".into(),
        fix_template: None,
    };
    assert!(block.is_block());
    assert!(!block.is_warn());

    let warn = HookFinding {
        tier: HookTier::T3,
        verdict: HookVerdictKind::Warn,
        rule_id: None,
        category: HOOK_EVENT_CATEGORY.into(),
        message: "m".into(),
        fix_template: Some("fix".into()),
    };
    assert!(!warn.is_block());
    assert!(warn.is_warn());

    let allow = HookFinding {
        tier: HookTier::Composite,
        verdict: HookVerdictKind::Allow,
        rule_id: None,
        category: HOOK_EVENT_CATEGORY.into(),
        message: "m".into(),
        fix_template: None,
    };
    assert!(!allow.is_block());
    assert!(!allow.is_warn());
}

// ---------------------------------------------------------------------------
// Contract 8: HookInvocation builders populate the expected fields
// ---------------------------------------------------------------------------

#[test]
fn contract_hook_invocation_builders() {
    let pre = HookInvocation::for_pre_tool_use("baby_step_pre_gate", "source", "step_1");
    assert_eq!(pre.action_type, "baby_step_pre_gate");
    assert_eq!(pre.source, Some("source"));
    assert!(pre.file_path.is_none());
    assert_eq!(pre.step_id, "step_1");

    let post = HookInvocation::for_post_tool_use(
        "file_write",
        "fn f() {}",
        "src/x.rs",
        "step_9",
    );
    assert_eq!(post.action_type, "file_write");
    assert_eq!(post.source, Some("fn f() {}"));
    assert_eq!(post.file_path, Some("src/x.rs"));

    let task = HookInvocation::for_task_level("task_stop", "task_1");
    assert!(task.source.is_none());
    assert!(task.file_path.is_none());
    assert_eq!(task.action_type, "task_stop");
    assert_eq!(task.step_id, "task_1");
}

// ---------------------------------------------------------------------------
// Contract 9: HookPoint / HookTier / HookVerdictKind string forms stable
// ---------------------------------------------------------------------------

#[test]
fn contract_string_forms_are_stable() {
    assert_eq!(HookPoint::PreToolUse.as_str(), "PreToolUse");
    assert_eq!(HookPoint::PostToolUse.as_str(), "PostToolUse");
    assert_eq!(HookPoint::Stop.as_str(), "Stop");
    assert_eq!(HookPoint::PreCompact.as_str(), "PreCompact");

    assert_eq!(HookTier::T1.as_str(), "T1");
    assert_eq!(HookTier::T2.as_str(), "T2");
    assert_eq!(HookTier::T3.as_str(), "T3");
    assert_eq!(HookTier::Composite.as_str(), "Composite");

    assert_eq!(HookVerdictKind::Allow.as_str(), "Allow");
    assert_eq!(HookVerdictKind::Warn.as_str(), "Warn");
    assert_eq!(HookVerdictKind::Block.as_str(), "Block");
}

// ---------------------------------------------------------------------------
// Contract 10: emit_hook_events_to_ssb writes exactly N rows for N findings
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contract_emit_hook_events_writes_one_row_per_finding() {
    use elai_orchestrator::hook_dispatcher::emit_hook_events_to_ssb;

    let tmp = TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "emit-test".into(),
        session_id: "emit-session".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(cfg, clock).expect("ssb"));

    let findings = vec![
        HookFinding {
            tier: HookTier::T1,
            verdict: HookVerdictKind::Warn,
            rule_id: Some("r1".into()),
            category: HOOK_EVENT_CATEGORY.into(),
            message: "warn1".into(),
            fix_template: None,
        },
        HookFinding {
            tier: HookTier::Composite,
            verdict: HookVerdictKind::Block,
            rule_id: Some("r2".into()),
            category: HOOK_EVENT_CATEGORY.into(),
            message: "block1".into(),
            fix_template: None,
        },
    ];
    let count = emit_hook_events_to_ssb(
        &ssb,
        "task_runner",
        0,
        HookPoint::PostToolUse,
        "baby_step_1",
        &findings,
    )
    .await;
    assert_eq!(count, 2, "one row per finding");

    let rows = ssb
        .query_by_type(SsbActionType::HookEvent)
        .expect("query hook_event rows");
    assert_eq!(rows.len(), 2);
    let decoded: Vec<SsbPayload> = rows
        .iter()
        .map(|r| rmp_serde::from_slice(&r.payload).expect("decode"))
        .collect();
    let tiers: Vec<String> = decoded
        .iter()
        .map(|p| match p {
            SsbPayload::HookEvent(e) => e.tier.clone(),
            other => panic!("expected HookEvent, got {other:?}"),
        })
        .collect();
    assert!(tiers.contains(&"T1".to_string()));
    assert!(tiers.contains(&"Composite".to_string()));
}

// Silence the unused-import warning if the async-mutex helper ever goes unused.
#[allow(dead_code)]
fn _asyncmutex_probe<T>(_m: _AsyncMutex<T>) {}
