//! C-27 contract: Testing-Trust T(-1) DeterminismConfig + T-0 WiringContract.
//!
//! Pins:
//!   * SCHEMA_VERSION_PUBLIC == 59 (atomic 9-tuple lockstep bump per
//!     workflow/reasoning/C-27-decisions.md D1).
//!   * Three new SsbActionType variants — WiringEvidence, WiringViolation,
//!     DeterminismDrift — round-trip via `ssb_action_type_from_str_public`
//!     and `SsbActionType::as_sql_str`.
//!   * WiringViolationKind { StubPattern, UnreachableSymbol, BrokenObservable }.
//!   * DeterminismConfig::from_env() succeeds with explicit ELAI_SEED=42.
//!   * DriftComparator: identical sequences -> zero rows; divergent -> one
//!     row with correct first_divergence_index.
//!   * StubBanAuditor detects todo!() / unimplemented!() / exact-line
//!     Ok(Default::default()) in synthetic non-test content; skips test paths.
//!   * WiringEntrypointSetLoader: missing [wiring.entrypoints] -> empty set,
//!     no panic.
//!   * DkgReachabilityAuditor: reachable -> WiringEvidence; missing ->
//!     WiringViolation(UnreachableSymbol).
//!   * ObservableEffectRunner: timeout -> WiringViolation(BrokenObservable);
//!     does NOT panic.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SsbPayload, WiringViolationKind,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    parse_wiring_contract_from_markdown, ActionTypeSequence, AuditorSink, DeterminismConfig,
    DkgReachabilityAuditor, DriftComparator, DriftSink, HardwareClass, ObservableEffect,
    ObservableEffectRunner, ReachabilityGraph, StubBanAuditor, WiringContract,
    WiringEntrypointSet, WiringEntrypointSetLoader,
};

// ---------------------------------------------------------------------------
// SSB schema + variant pins
// ---------------------------------------------------------------------------

#[test]
fn c27_schema_version_public_is_59() {
    // C-28: changed from exact `== 59` to floor `>= 59` so future
    // SCHEMA bumps (C-28 went 59 -> 60) do not break this older
    // C-27 contract. See workflow/reasoning/C-28-decisions.md
    // DECISION-D13 (floor precedent).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 59,
        "C-27 atomic 9-tuple lockstep bump landed at 59 (post-bumps allowed). \
         Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn c27_new_action_types_round_trip_through_str() {
    let cases = [
        (SsbActionType::WiringEvidence, "wiring_evidence"),
        (SsbActionType::WiringViolation, "wiring_violation"),
        (SsbActionType::DeterminismDrift, "determinism_drift"),
    ];
    for (variant, sql) in cases {
        assert_eq!(variant.as_sql_str(), sql);
        let parsed = ssb_action_type_from_str_public(sql)
            .unwrap_or_else(|e| panic!("from_str must accept {sql:?}: {e}"));
        assert_eq!(parsed, variant);
    }
}

#[test]
fn c27_wiring_violation_kind_has_three_variants() {
    // Exhaustiveness check — compiler enforces, but the explicit match
    // future-proofs against silent additions that would skip the audit
    // mapping logic.
    for k in [
        WiringViolationKind::StubPattern,
        WiringViolationKind::UnreachableSymbol,
        WiringViolationKind::BrokenObservable,
    ] {
        match k {
            WiringViolationKind::StubPattern => {}
            WiringViolationKind::UnreachableSymbol => {}
            WiringViolationKind::BrokenObservable => {}
        }
    }
}

// ---------------------------------------------------------------------------
// DeterminismConfig
// ---------------------------------------------------------------------------

#[test]
fn c27_determinism_config_from_env_with_explicit_seed() {
    // Set ELAI_SEED inside the test, restore the previous value after.
    let prior = std::env::var("ELAI_SEED").ok();
    // SAFETY: the test crate runs single-threaded for env mutations; the
    // contract suite does not parallelise env-touching tests by convention.
    unsafe {
        std::env::set_var("ELAI_SEED", "42");
    }
    let result = DeterminismConfig::from_env();
    let _restore = scopeguard_set("ELAI_SEED", prior);
    let cfg = result.expect("from_env must succeed");
    assert_eq!(cfg.seed, 42);
    // hardware_class defaults to Unknown when ELAI_HARDWARE_CLASS unset.
    match cfg.hardware_class {
        HardwareClass::Unknown | HardwareClass::Cpu | HardwareClass::Gpu(_) => {}
    }
}

// Minimal RAII env restore — tests must not leak ELAI_SEED into peer tests.
struct ScopeGuard {
    var: &'static str,
    prior: Option<String>,
}
impl Drop for ScopeGuard {
    fn drop(&mut self) {
        unsafe {
            match &self.prior {
                Some(v) => std::env::set_var(self.var, v),
                None => std::env::remove_var(self.var),
            }
        }
    }
}
fn scopeguard_set(var: &'static str, prior: Option<String>) -> ScopeGuard {
    ScopeGuard { var, prior }
}

// ---------------------------------------------------------------------------
// DriftComparator
// ---------------------------------------------------------------------------

#[test]
fn c27_drift_comparator_zero_rows_for_identical_sequences() {
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    let buf_clone = buf.clone();
    std::thread::spawn(move || {
        while let Some(p) = rx.blocking_recv() {
            if let Ok(mut g) = buf_clone.lock() {
                g.push(p);
            }
        }
    });
    let sink = DriftSink::Channel(tx);
    let cmp = DriftComparator::with_audit_id(42, sink, uuid::Uuid::nil());
    let a = ActionTypeSequence::new(vec![
        "file_write".into(),
        "tool_result".into(),
        "phase_transition".into(),
    ]);
    let b = a.clone();
    let verdict = cmp.compare(&a, &b);
    assert!(verdict.is_none(), "identical sequences must produce zero drift");
    // Allow the drain thread to settle.
    std::thread::sleep(Duration::from_millis(50));
    let captured = buf.lock().expect("lock");
    assert_eq!(captured.len(), 0);
}

#[test]
fn c27_drift_comparator_emits_one_row_on_divergence() {
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    let buf_clone = buf.clone();
    std::thread::spawn(move || {
        while let Some(p) = rx.blocking_recv() {
            if let Ok(mut g) = buf_clone.lock() {
                g.push(p);
            }
        }
    });
    let cmp = DriftComparator::with_audit_id(42, DriftSink::Channel(tx), uuid::Uuid::nil());
    let a = ActionTypeSequence::new(vec!["file_write".into(), "tool_result".into()]);
    let b = ActionTypeSequence::new(vec!["file_write".into(), "finding".into()]);
    let verdict = cmp.compare(&a, &b).expect("must emit drift");
    assert_eq!(verdict.first_divergence_index, 1);
    assert_eq!(verdict.seed, 42);
}

// ---------------------------------------------------------------------------
// StubBanAuditor
// ---------------------------------------------------------------------------

#[test]
fn c27_stub_ban_detects_todo_macro_in_production_path() {
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = AuditorSink::Channel(channel_drain(buf.clone()));
    let auditor =
        StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("stub_ban auditor");
    let count = auditor.scan_file_contents(
        Path::new("crates/foo/src/lib.rs"),
        "fn x() {\n    todo!()\n}\n",
    );
    assert_eq!(count, 1);
    std::thread::sleep(Duration::from_millis(50));
    let captured = buf.lock().expect("lock");
    assert_eq!(captured.len(), 1);
    match &captured[0] {
        SsbPayload::WiringViolation(p) => {
            assert_eq!(p.violation_kind, WiringViolationKind::StubPattern);
            assert_eq!(p.line, 2);
        }
        other => panic!("unexpected payload {other:?}"),
    }
}

#[test]
fn c27_stub_ban_skips_test_paths() {
    assert!(StubBanAuditor::should_skip_path(Path::new(
        "crates/foo/tests/bar.rs"
    )));
    assert!(StubBanAuditor::should_skip_path(Path::new(
        "crates/foo/src/bar_contract.rs"
    )));
    assert!(!StubBanAuditor::should_skip_path(Path::new(
        "crates/foo/src/lib.rs"
    )));
}

#[test]
fn c27_stub_ban_anchored_ok_default_default() {
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = AuditorSink::Channel(channel_drain(buf.clone()));
    let auditor =
        StubBanAuditor::new(&[], sink, uuid::Uuid::nil()).expect("stub_ban auditor");
    // Exact-line match — must fire.
    let n1 = auditor.scan_file_contents(
        Path::new("crates/foo/src/lib.rs"),
        "fn a() -> Result<X, E> {\n    Ok(Default::default())\n}\n",
    );
    assert_eq!(n1, 1, "exact-line Ok(Default::default()) must trip");
    // Inline use — must NOT fire on the ok_default_default rule.
    let buf2: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sink2 = AuditorSink::Channel(channel_drain(buf2.clone()));
    let auditor2 =
        StubBanAuditor::new(&[], sink2, uuid::Uuid::nil()).expect("stub_ban auditor");
    auditor2.scan_file_contents(
        Path::new("crates/foo/src/lib.rs"),
        "let x = Ok(Default::default()); // legit\n",
    );
    std::thread::sleep(Duration::from_millis(50));
    let captured2 = buf2.lock().expect("lock");
    for p in captured2.iter() {
        if let SsbPayload::WiringViolation(v) = p {
            assert_ne!(
                v.pattern, "ok_default_default_exact_line",
                "inline Ok(Default::default()) must not trip exact-line rule"
            );
        }
    }
}

fn channel_drain(buf: Arc<Mutex<Vec<SsbPayload>>>) -> tokio::sync::mpsc::UnboundedSender<SsbPayload> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<SsbPayload>();
    std::thread::spawn(move || {
        while let Some(p) = rx.blocking_recv() {
            if let Ok(mut g) = buf.lock() {
                g.push(p);
            }
        }
    });
    tx
}

// ---------------------------------------------------------------------------
// WiringEntrypointSetLoader
// ---------------------------------------------------------------------------

#[test]
fn c27_entrypoint_loader_missing_section_returns_err_fail_closed() {
    // WAVE-1 fail-close (R2): an absent [wiring.entrypoints] section is a hard
    // FAIL, not a silent empty default. A dead gate input must scream.
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let cfg = tmp.path().join("elai_cc.toml");
    std::fs::write(&cfg, "[tools.routing_rules]\nenabled = true\n").expect("write");
    let loader = WiringEntrypointSetLoader::new(&cfg);
    let err = loader
        .load()
        .expect_err("absent [wiring.entrypoints] section must be a fail-closed Err");
    assert!(
        matches!(
            err,
            elai_orchestrator::WiringContractError::AbsentEntrypointSet(_)
        ),
        "expected AbsentEntrypointSet, got {err:?}"
    );
}

#[test]
fn c27_entrypoint_loader_present_section_returns_symbols() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let cfg = tmp.path().join("elai_cc.toml");
    std::fs::write(
        &cfg,
        "[wiring.entrypoints]\nsymbols = [\"crates/cli/src/main.rs::main\"]\n",
    )
    .expect("write");
    let loader = WiringEntrypointSetLoader::new(&cfg);
    let set = loader.load().expect("load");
    assert_eq!(set.entrypoints.len(), 1);
}

// ---------------------------------------------------------------------------
// DkgReachabilityAuditor
// ---------------------------------------------------------------------------

#[test]
fn c27_dkg_reachability_emits_evidence_on_hit_and_violation_on_miss() {
    let mut g = ReachabilityGraph::new();
    g.add_edge("ep1", "DeterminismConfig");

    // Reachable -> evidence.
    let buf_a: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let auditor = DkgReachabilityAuditor::new(
        Arc::new(g.clone()),
        AuditorSink::Channel(channel_drain(buf_a.clone())),
        uuid::Uuid::nil(),
    );
    let contract = WiringContract {
        bn_id: "C-27".into(),
        produces_symbols: vec!["DeterminismConfig".into()],
        ..Default::default()
    };
    let entrypoints = WiringEntrypointSet::new(vec!["ep1".into()]);
    let outcome = auditor.audit_contract(&contract, &entrypoints);
    assert_eq!(outcome.evidence_emitted, 1);
    assert_eq!(outcome.violations_emitted, 0);

    // Missing -> violation(UnreachableSymbol).
    let buf_b: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let auditor2 = DkgReachabilityAuditor::new(
        Arc::new(ReachabilityGraph::new()),
        AuditorSink::Channel(channel_drain(buf_b.clone())),
        uuid::Uuid::nil(),
    );
    let bad_contract = WiringContract {
        bn_id: "C-27".into(),
        produces_symbols: vec!["NotInGraph".into()],
        ..Default::default()
    };
    let outcome2 = auditor2.audit_contract(&bad_contract, &entrypoints);
    assert_eq!(outcome2.evidence_emitted, 0);
    assert_eq!(outcome2.violations_emitted, 1);
}

// ---------------------------------------------------------------------------
// ObservableEffectRunner
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn c27_observable_effect_runner_timeout_emits_broken_observable() {
    let tmp = tempfile::TempDir::new().expect("tempdir");
    let db_path = tmp.path().join("ssb.db"); // intentionally missing
    let buf: Arc<Mutex<Vec<SsbPayload>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = AuditorSink::Channel(channel_drain(buf.clone()));
    let runner = ObservableEffectRunner::new(db_path, sink, uuid::Uuid::nil())
        .with_poll_interval(Duration::from_millis(10));
    let effect = ObservableEffect {
        trigger: "elai wiring audit".into(),
        expect_ssb: Some("nonexistent_action".into()),
        expect_artifact: None,
        timeout_secs: 1,
    };
    let result = runner.run_effect(&effect, "C-27").await;
    assert!(result.is_err(), "runner must err on timeout (no panic)");
    // Drain settle.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let captured = buf.lock().expect("lock");
    let saw_broken = captured.iter().any(|p| match p {
        SsbPayload::WiringViolation(v) => {
            v.violation_kind == WiringViolationKind::BrokenObservable
        }
        _ => false,
    });
    assert!(saw_broken, "must emit BrokenObservable violation");
}

// ---------------------------------------------------------------------------
// Markdown WIRING_CONTRACT parser
// ---------------------------------------------------------------------------

#[test]
fn c27_markdown_parser_extracts_block() {
    let md = r#"
## WIRING_CONTRACT

produces_symbols:
  - DeterminismConfig
consumes_symbols:
  - SeededRng
stub_ban:
  - prefix_cache_key MUST use blake3
observable_effect:
  trigger: "elai wiring audit"
  expect_ssb: "wiring_evidence"
  timeout_secs: 30
"#;
    let parsed = parse_wiring_contract_from_markdown("C-27", md).expect("parse");
    assert_eq!(parsed.produces_symbols.len(), 1);
    assert_eq!(parsed.consumes_symbols.len(), 1);
    assert_eq!(parsed.stub_ban.len(), 1);
    assert_eq!(parsed.observable_effects.len(), 1);
}
