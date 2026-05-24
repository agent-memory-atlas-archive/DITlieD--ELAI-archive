// D-11 contract: constructive-checker registry surface freeze.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 120 (D-11 substrate landed at the
//    cluster's current head; closure pass adds NO schema bump per
//    D-11-decisions.md DECISION-1).
// 2. SsbActionType::CheckerResultEmitted round-trips via as_sql_str
//    / ssb_action_type_from_str_public — preserves the consolidated
//    single-variant SSB shape per D-11-decisions.md DECISION-2.
// 3. CheckerResultEmittedPayload serializes via msgpack + JSON without
//    loss (round-trip).
// 4. CheckerKind is exhaustive at 8 variants (+CompilePy +RegistryCheck
//    atomic extension per D-11-decisions.md DECISION-3 / DECISION-5)
//    and the as_str/from_str inverse round-trip holds.
// 5. CheckerRegistry::load_from_toml_str is deterministic: same TOML
//    body + same root yields the same registry ids in the same sort
//    order.
// 6. SHELLOUT_TIMEOUT (30s) is enforced — a script that sleeps past
//    the wall-clock budget surfaces as CheckerResult::Error with a
//    "timeout" source. Skipped if `sh` / `sleep` is unavailable
//    (Windows without git-bash).

use std::path::PathBuf;
use std::time::Duration;

use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, CheckerResultEmittedPayload, SsbActionType, SsbPayload,
    SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{CheckerKind, CheckerRegistry, CheckerResult};

/// D-11 substrate landed at the cluster's current head (SCHEMA 120).
/// Closure pass adds zero SSB variants; floor stays at 120.
const D11_SCHEMA_FLOOR: i64 = 120;

#[test]
fn schema_version_includes_d11_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D11_SCHEMA_FLOOR,
        "schema must include D-11 CheckerResultEmitted variant (got {SCHEMA_VERSION_PUBLIC}, floor {D11_SCHEMA_FLOOR})"
    );
}

#[test]
fn checker_result_emitted_action_type_roundtrip() {
    let s = SsbActionType::CheckerResultEmitted.as_sql_str();
    assert_eq!(s, "checker_result_emitted");
    let decoded = ssb_action_type_from_str_public(s)
        .expect("ssb_action_type_from_str_public checker_result_emitted");
    assert_eq!(decoded, SsbActionType::CheckerResultEmitted);
}

#[test]
fn checker_result_emitted_payload_msgpack_roundtrip() {
    let p = CheckerResultEmittedPayload {
        checker_id: "registry-check".into(),
        checker_kind: "registry_check".into(),
        result_kind: "pass".into(),
        claim_id: "c-001".into(),
        ts_ms: 1_700_000_000_000,
        reason: "pypi: 'requests'==2.34.2 published 10d ago".into(),
    };
    let payload = SsbPayload::CheckerResultEmitted(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::CheckerResultEmitted);

    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack serialize");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack deserialize");
    assert_eq!(decoded.action_type(), SsbActionType::CheckerResultEmitted);

    if let SsbPayload::CheckerResultEmitted(d) = decoded {
        assert_eq!(d, p);
    } else {
        panic!("expected CheckerResultEmitted payload");
    }
}

#[test]
fn checker_result_emitted_payload_json_roundtrip() {
    let p = CheckerResultEmittedPayload {
        checker_id: "compile-py".into(),
        checker_kind: "compile_py".into(),
        result_kind: "fail".into(),
        claim_id: "c-json".into(),
        ts_ms: 1_700_000_000_500,
        reason: "SyntaxError: invalid syntax".into(),
    };
    let json = serde_json::to_string(&p).expect("json serialize");
    for field in [
        "\"checker_id\"",
        "\"checker_kind\"",
        "\"result_kind\"",
        "\"claim_id\"",
        "\"ts_ms\"",
        "\"reason\"",
    ] {
        assert!(json.contains(field), "missing field {field} in {json}");
    }
    let back: CheckerResultEmittedPayload = serde_json::from_str(&json).expect("json deserialize");
    assert_eq!(back, p);
}

#[test]
fn checker_kind_is_exhaustive_at_eight_variants() {
    // D-11 closure DECISION-3 + DECISION-5: +2 atomic extensions
    // (CompilePy, RegistryCheck) landed alongside helper scripts.
    let all = CheckerKind::all();
    assert_eq!(all.len(), 8, "expected 8 closed-enum variants, got {}", all.len());
    let labels: Vec<&str> = all.iter().map(|k| k.as_str()).collect();
    for required in [
        "sympy",
        "z3",
        "compile_rust",
        "regex_match",
        "json_schema_validate",
        "numeric_range",
        "compile_py",
        "registry_check",
    ] {
        assert!(
            labels.contains(&required),
            "CheckerKind::all missing variant {required}"
        );
    }
}

#[test]
fn checker_kind_as_str_from_str_round_trip() {
    for k in CheckerKind::all() {
        let s = k.as_str();
        let back = CheckerKind::from_str(s)
            .unwrap_or_else(|| panic!("from_str returned None for {s:?}"));
        assert_eq!(back, k);
        let serde_s = serde_json::to_string(&k).expect("serialize");
        let stripped = serde_s.trim_matches('"');
        assert_eq!(stripped, s, "as_str / serde mismatch for {k:?}");
    }
}

#[test]
fn checker_kind_from_str_rejects_unknown_label() {
    assert!(CheckerKind::from_str("not-a-kind").is_none());
    assert!(CheckerKind::from_str("").is_none());
}

#[test]
fn registry_loader_is_deterministic_for_same_toml_body() {
    // Two identical loads must produce the same ids in the same sorted
    // order. We point the loader at a synthetic root that does NOT
    // contain real scripts; the loader does not validate existence
    // (existence is checked at dispatch time via ScriptChecker::check),
    // so the registry shape is deterministic regardless.
    let toml_body = r#"
[[checker]]
id          = "zeta-script"
kind        = "sympy"
script      = ".elai_cc/checkers/math/sympy.py"
interpreter = "python3"
applies_to  = ["computational"]
version     = "0.1.0"

[[checker]]
id          = "alpha-script"
kind        = "regex_match"
script      = ".elai_cc/checkers/text/regex_assertion.py"
interpreter = "python3"
applies_to  = []
version     = "0.1.0"

[[checker]]
id          = "mu-script"
kind        = "registry_check"
script      = ".elai_cc/checkers/registry/registry_check.py"
interpreter = "python3"
applies_to  = ["factual", "procedural"]
version     = "0.1.0"
"#;
    let root = PathBuf::from(".");
    let reg_a = CheckerRegistry::load_from_toml_str(toml_body, &root).expect("load A");
    let reg_b = CheckerRegistry::load_from_toml_str(toml_body, &root).expect("load B");

    let ids_a = reg_a.ids_sorted();
    let ids_b = reg_b.ids_sorted();
    assert_eq!(ids_a, ids_b, "registry ids must match across identical loads");
    assert_eq!(ids_a, vec!["alpha-script", "mu-script", "zeta-script"]);

    let entries_a = reg_a.entries_sorted();
    let entries_b = reg_b.entries_sorted();
    assert_eq!(entries_a, entries_b, "entries_sorted must match");
    assert_eq!(entries_a[0].1, CheckerKind::RegexMatch);
    assert_eq!(entries_a[1].1, CheckerKind::RegistryCheck);
    assert_eq!(entries_a[2].1, CheckerKind::Sympy);
}

#[test]
fn registry_loader_rejects_unknown_kind() {
    let bad_toml = r#"
[[checker]]
id          = "bad-row"
kind        = "not-a-real-kind"
script      = ".elai_cc/checkers/math/sympy.py"
interpreter = "python3"
applies_to  = []
version     = "0.1.0"
"#;
    let root = PathBuf::from(".");
    let err = CheckerRegistry::load_from_toml_str(bad_toml, &root)
        .expect_err("loader must reject unknown kind");
    let msg = format!("{err}");
    assert!(msg.contains("not-a-real-kind"), "error mentions bad kind: {msg}");
}

#[test]
fn registry_loader_rejects_unknown_claim_type() {
    let bad_toml = r#"
[[checker]]
id          = "bad-claim-row"
kind        = "sympy"
script      = ".elai_cc/checkers/math/sympy.py"
interpreter = "python3"
applies_to  = ["not_a_claim_type"]
version     = "0.1.0"
"#;
    let root = PathBuf::from(".");
    let err = CheckerRegistry::load_from_toml_str(bad_toml, &root)
        .expect_err("loader must reject unknown applies_to claim type");
    let msg = format!("{err}");
    assert!(msg.contains("not_a_claim_type"), "error mentions bad claim type: {msg}");
}

/// Confirms SHELLOUT_TIMEOUT (30s) gates a runaway helper. We pick
/// a `sleep 60` which exceeds the 30s budget. Skipped if `sh` is
/// unavailable; the underlying registry runner uses the same sh
/// invocation as compile_rust.sh / compile_py.sh.
///
/// Uses a tokio runtime + a tighter test budget (35s) so a real
/// timeout will surface before this test's own wall clock expires.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shellout_timeout_returns_error_within_budget() {
    use elai_orchestrator::ScriptChecker;

    // `which sh` probe. Avoid panicking on systems without sh.
    let sh_available = std::process::Command::new("sh")
        .arg("-c")
        .arg("exit 0")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !sh_available {
        eprintln!("skipping shellout_timeout_returns_error_within_budget: sh not available");
        return;
    }

    // Write a sleep-then-output helper to a tempdir. Avoids polluting
    // the workspace tree.
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("sleep_long.sh");
    let script_body = "#!/usr/bin/env sh\nsleep 60\nprintf '%s\\n' '{\"result\":\"pass\"}'\n";
    std::fs::write(&script_path, script_body).expect("write sleep script");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms).expect("chmod");
    }

    let checker = ScriptChecker::new(
        CheckerKind::Sympy, // any kind — the test asserts on result shape
        PathBuf::from("sh"),
        script_path,
        None,
    );
    let mut reg = CheckerRegistry::new();
    reg.register("sleep-runaway", Box::new(checker));

    let claim = elai_orchestrator::Claim {
        claim_id: "c-timeout".into(),
        text: "x = 1".into(),
        claim_type: elai_orchestrator::ClaimType::Computational,
        evidence: Vec::new(),
        ts_ms: 1_700_000_000_000,
    };

    // The internal SHELLOUT_TIMEOUT is 30s; bound the test's own
    // wall clock at 45s so we surface a clear failure rather than a
    // CI-killing hang.
    let dispatch = tokio::time::timeout(Duration::from_secs(45), reg.run_all_applicable(&claim))
        .await
        .expect("checker dispatch did not return within 45s test budget");
    assert_eq!(dispatch.len(), 1);
    let result = &dispatch[0].2;
    assert_eq!(
        result.result_kind(),
        "error",
        "runaway helper must surface as CheckerResult::Error, got {result:?}"
    );
    if let CheckerResult::Error { source } = result {
        assert!(
            source.contains("timeout"),
            "error source must mention timeout, got: {source}"
        );
    } else {
        panic!("expected CheckerResult::Error, got {result:?}");
    }
}
