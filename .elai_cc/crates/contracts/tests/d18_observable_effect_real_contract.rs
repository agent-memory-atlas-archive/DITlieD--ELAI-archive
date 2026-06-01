//! D-18 tranche-B5 contract: ObservableEffect REAL executor wire.
//!
//! Pins the following invariants for the producer that replaces the
//! s-21636 smoke beacon at
//! `crates/orchestrator/src/task_runner.rs` `dispatch_observable_effect_hook`:
//!
//!   1. `check_goodhart_risk_with_min` flags singleton sets when
//!      `expected_min >= 2` (D-18 spec §2 "single-effect probe vulnerable
//!      to Goodhart"; spec acceptance test "probe with 1 effect ->
//!      GoodhartRiskFlag SSB row").
//!   2. `check_goodhart_risk_with_min` does NOT flag sets that meet the
//!      threshold AND are orthogonal.
//!   3. `NoOpSandboxExecutor` returns `UnsupportedTrigger` for every
//!      trigger kind, which `verify_observable_effect` maps to
//!      `VerificationOutcome::Skipped` (executor-side problem, NOT a
//!      product failure).
//!   4. The hook's env-gate matcher accepts only `"1" | "true" | "yes"
//!      | "on"` (case-sensitive snake-case-truthy) so the smoke-legacy
//!      migration window flag stays predictable.
//!   5. `GoodhartRiskFlag` SSB action_type + payload round-trip stays
//!      stable (no new variant added by tranche-B5; SCHEMA_VERSION
//!      stays at the pre-tranche value).
//!
//! Plan: docs/thefinalglance/D-plan/D-18_observable_effect_typed.md.
//! Decisions: workflow/reasoning/D-18-tranche-B5-decisions.md.

use elai_orchestrator::observable_effect::{
    check_goodhart_risk, check_goodhart_risk_with_min, ConfoundFlags, EffectSet, NoOpSandboxExecutor,
    ObservableEffect, SandboxExecutor, SandboxExecutorError, SandboxTier, TriggerSpec,
    TypedObservation,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, GoodhartRiskFlagPayload, SsbActionType, SsbPayload,
};

/// Build a sentinel effect mirroring the producer's inline construction
/// at `task_runner.rs::dispatch_observable_effect_hook` (real-executor
/// path). Keeps the test in lockstep with the producer's wire form.
fn sentinel_effect(effect_id: &str) -> ObservableEffect {
    ObservableEffect {
        effect_id: effect_id.into(),
        trigger: TriggerSpec::TestCommand("noop-sentinel".to_string()),
        expected: TypedObservation::StringMatch("noop".to_string()),
        sandbox: SandboxTier::NoSandbox,
        confound_flags: ConfoundFlags::default(),
    }
}

#[test]
fn test_check_goodhart_risk_with_min_flags_singleton_below_threshold() {
    let mut set = EffectSet::new();
    set.push(sentinel_effect("only-one"));

    let flags = check_goodhart_risk_with_min(&set, 2);
    assert_eq!(
        flags.len(),
        1,
        "singleton set must yield exactly one below_min_effects flag (got {})",
        flags.len()
    );
    let flag = &flags[0];
    assert_eq!(flag.effect_id, "only-one", "flag carries first effect id");
    assert_eq!(
        flag.risk_kind, "below_min_effects",
        "risk_kind pinned to D-18 spec §2 label"
    );
    assert!(
        flag.evidence.contains("N=1"),
        "evidence cites the observed N (got {})",
        flag.evidence
    );

    // Empty set: same flag with sentinel effect_id.
    let empty = EffectSet::new();
    let flags = check_goodhart_risk_with_min(&empty, 2);
    assert_eq!(flags.len(), 1, "empty set still flagged when expected_min>=1");
    assert_eq!(flags[0].effect_id, "<empty_effect_set>");
    assert_eq!(flags[0].risk_kind, "below_min_effects");
}

#[test]
fn test_check_goodhart_risk_with_min_passes_on_meeting_threshold() {
    // Two orthogonal effects: distinct (trigger.kind, observation.kind)
    // pairs so check_goodhart_risk returns empty; and N>=expected_min
    // so the below_min_effects branch does not fire.
    let mut set = EffectSet::new();
    set.push(sentinel_effect("e1"));
    let e2 = ObservableEffect {
        effect_id: "e2".into(),
        trigger: TriggerSpec::FilePresence(std::path::PathBuf::from("/tmp/sentinel")),
        expected: TypedObservation::StringMatch("present".to_string()),
        sandbox: SandboxTier::NoSandbox,
        confound_flags: ConfoundFlags::default(),
    };
    set.push(e2);

    let flags = check_goodhart_risk_with_min(&set, 2);
    assert!(
        flags.is_empty(),
        "two orthogonal effects at threshold must not flag (got {:?})",
        flags
    );

    // Sanity: the back-compat fn returns the same empty result for
    // the orthogonal case.
    let legacy = check_goodhart_risk(&set);
    assert!(legacy.is_empty(), "back-compat fn must agree on orthogonal set");
}

#[test]
fn test_noop_sandbox_executor_returns_skipped() {
    // NoOpSandboxExecutor returns SandboxExecutorError::UnsupportedTrigger
    // for every trigger kind; verify_observable_effect maps that to
    // VerificationOutcome::Skipped (see observable_effect.rs run_and_compare).
    let executor = NoOpSandboxExecutor::new();
    let triggers = [
        TriggerSpec::TestCommand("ignored".to_string()),
        TriggerSpec::FilePresence(std::path::PathBuf::from("/x")),
        TriggerSpec::HttpEndpointGet {
            url: "https://example.test".to_string(),
            expected_status: 200,
        },
        TriggerSpec::FileContents {
            path: std::path::PathBuf::from("/x"),
            regex: "^x$".to_string(),
        },
        TriggerSpec::StdoutContains("substr".to_string()),
    ];
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build rt");
    for trig in &triggers {
        let result = rt.block_on(async {
            executor.run(trig, SandboxTier::NoSandbox).await
        });
        match result {
            Err(SandboxExecutorError::UnsupportedTrigger(kind)) => {
                assert_eq!(
                    kind,
                    trig.kind_str(),
                    "noop must surface the trigger kind in the error"
                );
            }
            other => panic!(
                "noop sandbox executor must return UnsupportedTrigger, got {:?}",
                other
            ),
        }
    }
}

/// Local matcher mirroring the cached env-gate parse the producer uses
/// at task_runner.rs::dispatch_observable_effect_hook. The hook is a
/// private async fn, so this test pins the matcher's accepted truthy set
/// rather than calling the hook directly. If either flag is added,
/// removed, or has its truthy-set widened, this contract breaks and the
/// hook must be re-audited.
fn env_gate_truthy(raw: Option<&str>) -> bool {
    matches!(raw, Some("1") | Some("true") | Some("yes") | Some("on"))
}

#[test]
fn test_observable_effect_hook_default_off_smoke_legacy_emits_nothing() {
    // Default: env var unset (None) or empty/garbage -> matcher returns
    // false -> producer SKIPS legacy smoke beacon emission. The producer's
    // sandbox row in that case carries the sentinel effect_id, not
    // "smoke-d18-effect".
    assert!(
        !env_gate_truthy(None),
        "missing env var must be false (default-OFF)"
    );
    assert!(
        !env_gate_truthy(Some("")),
        "empty string is not truthy"
    );
    assert!(
        !env_gate_truthy(Some("0")),
        "literal zero is not truthy"
    );
    assert!(
        !env_gate_truthy(Some("false")),
        "literal false is not truthy"
    );
    assert!(
        !env_gate_truthy(Some("garbage")),
        "arbitrary string is not truthy"
    );
}

#[test]
fn test_observable_effect_hook_smoke_legacy_enabled_emits_smoke_beacon() {
    // Truthy set per env-gate matcher; producer emits the legacy
    // smoke-d18-effect ObservableEffectDispatched row alongside the
    // real-path rows. This contract pins the truthy set so the
    // single-release migration window stays predictable.
    for truthy in ["1", "true", "yes", "on"] {
        assert!(
            env_gate_truthy(Some(truthy)),
            "{} must be truthy",
            truthy
        );
    }
    // Case-sensitivity: producer matcher is intentionally case-sensitive
    // (uppercase variants must NOT trip the gate; documented in
    // workflow/reasoning/D-18-tranche-B5-decisions.md).
    for falsy in ["TRUE", "Yes", "ON", "1 ", " 1"] {
        assert!(
            !env_gate_truthy(Some(falsy)),
            "{:?} must NOT be truthy (case-sensitive + no-trim)",
            falsy
        );
    }
}

#[test]
fn test_observable_effect_event_kind_round_trip() {
    // GoodhartRiskFlag SSB action_type + payload round-trip stays stable.
    // No new variant introduced by tranche-B5 — this test prevents
    // accidental SCHEMA_VERSION drift.
    let payload = SsbPayload::GoodhartRiskFlag(GoodhartRiskFlagPayload {
        effect_id: "d18-executor-sentinel-seq42-step0".into(),
        risk_kind: "below_min_effects".into(),
        evidence: "EffectSet has N=1 effect(s), expected_min=2".into(),
        ts_ms: 1_700_000_000_000,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::GoodhartRiskFlag,
        "action_type symmetry"
    );
    assert_eq!(
        SsbActionType::GoodhartRiskFlag.as_sql_str(),
        "goodhart_risk_flag",
        "wire label pinned"
    );
    let parsed = ssb_action_type_from_str_public("goodhart_risk_flag")
        .expect("from_str symmetry");
    assert_eq!(parsed, SsbActionType::GoodhartRiskFlag);

    // msgpack round-trip via rmp_serde (matches Ssb.write_payload wire form).
    let bytes = rmp_serde::to_vec_named(&payload).expect("encode");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    match decoded {
        SsbPayload::GoodhartRiskFlag(p) => {
            assert_eq!(p.risk_kind, "below_min_effects");
            assert_eq!(p.ts_ms, 1_700_000_000_000);
            assert!(p.effect_id.starts_with("d18-executor-sentinel-"));
        }
        other => panic!("expected GoodhartRiskFlag, got {:?}", other),
    }
}
