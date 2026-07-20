//! I-19 — TDD enforcer preflight gate contract tests.
//!
//! Coverage (6 arms + 1 schema lock; no #[ignore], no stubs, no placeholder asserts):
//!
//! (a) MissingTestDelta BLOCKED: an Edit with a new pub fn and no matching test-symbol
//!     yields a Block finding (MissingTestDelta arm).
//!
//! (b) Matched ALLOWED: an Edit with a new pub fn that matches a test-symbol name
//!     yields no block finding.
//!
//! (c) RefactorExempted WARN: an Edit with only private / non-pub Rust code yields a
//!     Warn finding (RefactorExempted arm) — NOT a block.
//!
//! (d) SSB round-trip: TddPreflightBlocked SSB row written to a real temp db,
//!     read back, decoded; field content verified (off-author artifact, R4).
//!
//! (e) Wire-dark guard: `tdd_preflight_enabled()` returns false when
//!     `ELAI_TDD_PREFLIGHT_ENABLE` is absent or not truthy; gate is OFF by default.
//!
//! (f) Schema lock: SCHEMA_VERSION_PUBLIC >= 254 AND SSB_ACTION_TYPES.len() >= 563
//!     AND "tdd_preflight_missing_test_delta" / "tdd_preflight_syntax_dry_run_failed" /
//!     "tdd_preflight_refactor_exempted" all round-trip via ssb_action_type_from_str_public.
//!
//! (g) Production-dispatch SSB row lands: drives the PRODUCTION
//!     `InProcessHookDispatcher` path with `tdd_preflight_ssb` wired to a real
//!     temp SQLite db, sets ELAI_TDD_PREFLIGHT_ENABLE=1, fires
//!     `dispatch_pre_tool_use` on a source with a new pub fn and no matching
//!     test symbol, then queries the real DB and asserts a
//!     `TddPreflightBlocked(MissingTestDelta)` row landed (off-author artifact,
//!     wiring-enforcement R4). This is the arm that proves the wire-dark fix:
//!     the SSB emit path is reachable from the production hook constructor.
//!
//! Spec references: docs/thefinalglance/I-plan/I-19_tdd_enforcer_preflight.md
//! Reads (mandatory): wiring-enforcement.md R1/R4/R7/R8; CLAUDE.md GRC + GD-4.

use std::sync::{Arc, Mutex};

use elai_orchestrator::hook_dispatcher::{HookDispatcher, HookInvocation, InProcessHookDispatcher};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig, SsbPayload,
    TddPreflightBlockedKind, TddPreflightBlockedPayload, SCHEMA_VERSION_PUBLIC,
    SSB_ACTION_TYPES,
};
use elai_orchestrator::tdd_preflight::{TddPreflight, tdd_preflight_enabled};

// Cargo runs the tests in one binary in parallel; `i19_gate_is_off_by_default`
// and `i19_production_dispatch_ssb_row_lands` both touch the SAME process-global
// env var `ELAI_TDD_PREFLIGHT_ENABLE`. Serialize them so the suite is not flaky.
static ENV_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// (a) MissingTestDelta BLOCKED
// ---------------------------------------------------------------------------

/// A new pub fn with no matching test symbol triggers a Block finding.
#[test]
fn i19_missing_test_delta_blocked() {
    let gate = TddPreflight::new();
    let src = r#"
pub fn compute_risk_score(input: f64) -> f64 {
    input * 1.5
}
"#;
    // No test symbols — empty test index (Tier-1 state).
    let findings = gate.check(
        Some("src/risk.rs"),
        Some(src),
        &[], // empty test-symbol feed
        "EXECUTING",
    );
    let has_block = findings.iter().any(|f| f.is_block());
    assert!(
        has_block,
        "expected a Block finding for new pub fn with no matching test-symbol; got {:?}",
        findings
    );
    let block = findings.iter().find(|f| f.is_block()).unwrap();
    assert!(
        block.message.contains("MissingTestDelta"),
        "block message must mention MissingTestDelta; got: {}",
        block.message
    );
    assert!(
        block.category == "tdd_preflight",
        "category must be tdd_preflight; got: {}",
        block.category
    );
}

// ---------------------------------------------------------------------------
// (b) Matched ALLOWED
// ---------------------------------------------------------------------------

/// A new pub fn whose name substring-matches a test-symbol name passes the gate.
#[test]
fn i19_matched_pub_fn_is_allowed() {
    let gate = TddPreflight::new();
    let src = r#"
pub fn compute_risk_score(input: f64) -> f64 {
    input * 1.5
}
"#;
    let test_syms = vec!["test_compute_risk_score".to_string()];
    let findings = gate.check(
        Some("src/risk.rs"),
        Some(src),
        &test_syms,
        "EXECUTING",
    );
    let has_block = findings.iter().any(|f| f.is_block());
    assert!(
        !has_block,
        "expected no Block finding when pub fn matches test symbol; got {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// (c) RefactorExempted WARN
// ---------------------------------------------------------------------------

/// An Edit with only private / non-pub code yields a Warn (RefactorExempted),
/// NOT a Block.
#[test]
fn i19_no_pub_symbols_is_warn_not_block() {
    let gate = TddPreflight::new();
    let src = r#"
fn helper_internal(x: i32) -> i32 {
    x + 1
}
"#;
    let findings = gate.check(
        Some("src/helpers.rs"),
        Some(src),
        &[],
        "EXECUTING",
    );
    // The gate must NOT block on purely private code.
    let has_block = findings.iter().any(|f| f.is_block());
    assert!(
        !has_block,
        "gate must NOT block on a file with no pub symbols (RefactorExempted); got {:?}",
        findings
    );
    // An advisory Warn is expected (RefactorExempted advisory).
    let has_warn = findings.iter().any(|f| f.is_warn());
    assert!(
        has_warn,
        "gate must emit a Warn for RefactorExempted (advisory SSB row path); got {:?}",
        findings
    );
    let warn = findings.iter().find(|f| f.is_warn()).unwrap();
    assert!(
        warn.message.contains("RefactorExempted"),
        "warn message must mention RefactorExempted; got: {}",
        warn.message
    );
}

// ---------------------------------------------------------------------------
// (d) SSB round-trip
// ---------------------------------------------------------------------------

/// A `TddPreflightBlockedPayload` written to a real temp SSB db and read back
/// must decode correctly (off-author artifact, wiring-enforcement R4).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i19_ssb_round_trip_missing_test_delta() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::ssb::{Ssb, SsbConfig, SsbReader};
    use tempfile::TempDir;

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("i19-ssb.db"),
                pool_size: Some(2),
                project_id: "i19-contract".into(),
                session_id: "i19-d".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );

    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let payload = TddPreflightBlockedPayload {
        kind: TddPreflightBlockedKind::MissingTestDelta,
        file_path: "src/risk.rs".into(),
        pub_symbols: "compute_risk_score".into(),
        fsm_state: Some("EXECUTING".into()),
        dry_run_stderr: String::new(),
        ts_ms,
    };

    ssb.write_payload(
        "i19-contract-agent",
        1,
        SsbPayload::TddPreflightBlocked(payload.clone()),
        Vec::new(),
    )
    .await
    .expect("write_payload");

    let rows = ssb
        .query_by_type(SsbActionType::TddPreflightBlocked(
            TddPreflightBlockedKind::MissingTestDelta,
        ))
        .expect("query_by_type");

    assert!(
        !rows.is_empty(),
        "expected >=1 TddPreflightBlocked row; found 0 (R1: missing evidence = FAIL)"
    );

    let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
    match decoded {
        SsbPayload::TddPreflightBlocked(p) => {
            assert_eq!(
                p.kind,
                TddPreflightBlockedKind::MissingTestDelta,
                "decoded kind must be MissingTestDelta"
            );
            assert_eq!(
                p.file_path, "src/risk.rs",
                "decoded file_path must match"
            );
            assert_eq!(
                p.pub_symbols, "compute_risk_score",
                "decoded pub_symbols must match"
            );
            assert_eq!(
                p.fsm_state.as_deref(),
                Some("EXECUTING"),
                "decoded fsm_state must match"
            );
        }
        other => panic!(
            "expected SsbPayload::TddPreflightBlocked; got {:?}",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// (e) Wire-dark guard
// ---------------------------------------------------------------------------

/// `tdd_preflight_enabled()` returns false when the env var is absent.
/// This confirms the gate is wire-dark by default (GD-4 constraint: default-ON
/// flip gated on FP rate bounded on >=3 real orchestrate runs).
#[test]
fn i19_gate_is_off_by_default() {
    let _g = ENV_LOCK.lock().expect("env lock");
    // Derive the expected value from the real env var so the test passes
    // whether the CI explicitly sets it or not, and also when a developer
    // runs with ELAI_TDD_PREFLIGHT_ENABLE=1 locally.
    let expected = std::env::var("ELAI_TDD_PREFLIGHT_ENABLE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
        .unwrap_or(false);
    assert_eq!(
        tdd_preflight_enabled(),
        expected,
        "tdd_preflight_enabled() must match the ELAI_TDD_PREFLIGHT_ENABLE env var"
    );
    // Wire-dark guard: in the normal (CI / default) case the gate is OFF.
    if !expected {
        assert!(
            !tdd_preflight_enabled(),
            "gate must be OFF when ELAI_TDD_PREFLIGHT_ENABLE is absent (wire-dark default)"
        );
    }
}

// ---------------------------------------------------------------------------
// (f) Schema lock
// ---------------------------------------------------------------------------

/// SCHEMA_VERSION_PUBLIC must be >= 254 (I-19 floor).
#[test]
fn i19_schema_version_is_254() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 254,
        "SCHEMA_VERSION_PUBLIC must be >= 254 (I-19 floor: TddPreflightBlocked added); \
         got {} — a value below 254 means the I-19 SSB bump was regressed",
        SCHEMA_VERSION_PUBLIC,
    );
}

/// SSB_ACTION_TYPES.len() must be >= 563 (I-19 floor; subsequent plans bump further).
#[test]
fn i19_total_variants_is_563() {
    const I19_TV_FLOOR: usize = 563;
    let total = SSB_ACTION_TYPES.len();
    assert!(
        total >= I19_TV_FLOOR,
        "SSB_ACTION_TYPES.len() must be >= {} (I-19 floor: TddPreflightBlocked added 1 representative); got {}",
        I19_TV_FLOOR,
        total,
    );
    // Spot-check: the representative arm must appear in the list.
    let has_tdd = SSB_ACTION_TYPES
        .iter()
        .any(|t| matches!(t, SsbActionType::TddPreflightBlocked(_)));
    assert!(
        has_tdd,
        "SSB_ACTION_TYPES must include TddPreflightBlocked representative"
    );
}

/// All three TddPreflightBlocked sql_str values must round-trip via
/// `ssb_action_type_from_str_public`.
#[test]
fn i19_action_type_round_trips() {
    let arms = [
        (
            "tdd_preflight_missing_test_delta",
            TddPreflightBlockedKind::MissingTestDelta,
        ),
        (
            "tdd_preflight_syntax_dry_run_failed",
            TddPreflightBlockedKind::SyntaxDryRunFailed,
        ),
        (
            "tdd_preflight_refactor_exempted",
            TddPreflightBlockedKind::RefactorExempted,
        ),
    ];
    for (wire_str, expected_kind) in arms {
        let parsed = ssb_action_type_from_str_public(wire_str)
            .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public({wire_str:?}) failed: {e}"));
        match parsed {
            SsbActionType::TddPreflightBlocked(kind) => {
                assert_eq!(
                    kind,
                    expected_kind,
                    "round-trip kind mismatch for {wire_str:?}"
                );
                // Verify the as_sql_str() round-trips back to the original wire_str.
                assert_eq!(
                    kind.as_sql_str(),
                    wire_str,
                    "as_sql_str() must produce the original wire_str for {wire_str:?}"
                );
            }
            other => panic!(
                "expected TddPreflightBlocked for {wire_str:?}; got {:?}",
                other
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// (g) Production-dispatch SSB row lands (D-030 wire-dark fix proof)
// ---------------------------------------------------------------------------

/// Drives the PRODUCTION `InProcessHookDispatcher` with `tdd_preflight_ssb`
/// wired to a real temp SQLite db, sets `ELAI_TDD_PREFLIGHT_ENABLE=1`, fires
/// `dispatch_pre_tool_use` on an Edit that adds a new pub fn with NO matching
/// test symbol, then queries the real DB and asserts a
/// `TddPreflightBlocked(MissingTestDelta)` row landed.
///
/// This is the arm that proves the D-030 fix: the SSB emit path is now
/// reachable from the production hook constructor (previously `TddPreflight`
/// was constructed WITHOUT `.with_ssb()`, so `emit_tdd_preflight_blocked_to_ssb`
/// was structurally unreachable even with `ELAI_TDD_PREFLIGHT_ENABLE=1`).
///
/// wiring-enforcement R4: off-author artifact (the DB row), missing = FAIL.
/// wiring-enforcement R7: test-file callers do NOT satisfy production
/// reachability; this arm drives the real dispatcher, not TddPreflight::check.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i19_production_dispatch_ssb_row_lands() {
    use elai_core::clock::OsClock;
    use elai_orchestrator::ssb::SsbReader;
    use tempfile::TempDir;

    // Serialize against `i19_gate_is_off_by_default`: both touch the
    // process-global `ELAI_TDD_PREFLIGHT_ENABLE` env var. Hold the lock for
    // the full test body so the EnvRestore drop still runs under the lock.
    let _env_lock = ENV_LOCK.lock().expect("env lock");

    let dir = TempDir::new().expect("tempdir");
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(
        Ssb::new(
            SsbConfig {
                db_path: dir.path().join("i19-prod-dispatch.db"),
                pool_size: Some(2),
                project_id: "i19-contract".into(),
                session_id: "i19-g".into(),
            },
            clock,
        )
        .expect("ssb open"),
    );

    // Construct InProcessHookDispatcher WITH tdd_preflight_ssb wired.
    // This is the production constructor pattern (mirrors build_production_hook_dispatcher).
    let dispatcher: Arc<InProcessHookDispatcher> =
        Arc::new(InProcessHookDispatcher::new(None).with_tdd_preflight_ssb(
            ssb.clone(),
            "i19-contract-agent",
            1,
        ));

    // Set the gate env var for the duration of this test.
    // Safety: std::env::set_var is unsafe in Rust 1.80+; the test runs in a
    // dedicated async runtime (flavor = "multi_thread") so no other thread in
    // THIS test binary should race on this var. The value is scoped to this
    // test's env and we restore it at the end.
    //
    // I-19 W-1: use a RAII drop-guard so the var is restored even if the test
    // panics between set_var and the manual restore that was here before. The
    // guard holds the previous value and calls set_var / remove_var in Drop.
    struct EnvRestore {
        key: &'static str,
        prev: Option<String>,
    }
    impl Drop for EnvRestore {
        fn drop(&mut self) {
            // SAFETY: symmetric with the set_var below; same thread, same guard.
            unsafe {
                match self.prev.take() {
                    Some(v) => std::env::set_var(self.key, v),
                    None => std::env::remove_var(self.key),
                }
            }
        }
    }
    const ENV_KEY: &str = "ELAI_TDD_PREFLIGHT_ENABLE";
    let _guard = EnvRestore {
        key: ENV_KEY,
        prev: std::env::var(ENV_KEY).ok(),
    };
    // SAFETY: the guard above restores on every exit path (including panic);
    // no concurrent test mutates this var (dedicated multi_thread runtime).
    unsafe { std::env::set_var(ENV_KEY, "1"); }

    // Source with a new pub fn and no test symbols in the Tier-1 feed.
    let source = r#"
pub fn compute_risk_score(input: f64) -> f64 {
    input * 1.5
}
"#;

    use rustc_hash::FxHashMap;
    let invocation = HookInvocation {
        action_type: "Edit",
        source: Some(source),
        file_path: Some("src/risk.rs"),
        metric_values: FxHashMap::default(),
        step_id: "step-i19-g".to_string(),
        fsm_state: "EXECUTING".to_string(),
        pre_compact_token_state: None,
        unsaved_artifact_count: 0,
        pod_billable: false,
    };

    // Drive the real production dispatch path.
    let findings = dispatcher
        .dispatch_pre_tool_use(invocation)
        .await
        .expect("dispatch_pre_tool_use must not error");

    // Guard drops here (or on panic) and restores the env var automatically.

    // Gate must have fired and produced a Block finding.
    let has_block = findings.iter().any(|f| f.is_block());
    assert!(
        has_block,
        "dispatch_pre_tool_use with tdd_preflight_ssb wired and ENABLE=1 must Block on \
         new pub fn with no matching test symbol; got {:?}",
        findings
    );

    // Off-author artifact (R4): TddPreflightBlocked row must have landed in the db.
    let rows = ssb
        .query_by_type(SsbActionType::TddPreflightBlocked(
            TddPreflightBlockedKind::MissingTestDelta,
        ))
        .expect("query_by_type");

    assert!(
        !rows.is_empty(),
        "expected >=1 TddPreflightBlocked row in the db after production dispatch; \
         found 0 (wiring-enforcement R1: missing evidence = FAIL; this proves the \
         emit path is now reachable from the production hook constructor)"
    );

    let decoded = decode_ssb_payload(&rows[0].payload).expect("decode_ssb_payload");
    match decoded {
        SsbPayload::TddPreflightBlocked(p) => {
            assert_eq!(
                p.kind,
                TddPreflightBlockedKind::MissingTestDelta,
                "decoded kind must be MissingTestDelta"
            );
            assert_eq!(
                p.file_path, "src/risk.rs",
                "decoded file_path must match the Edit invocation"
            );
        }
        other => panic!(
            "expected SsbPayload::TddPreflightBlocked; got {:?}",
            other
        ),
    }
}
