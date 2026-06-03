//! F-9 / Slice D4 — DepWriteTimeGate contract tests.
//!
//! Validates:
//!   1. Hallucinated name -> Block(NotFound) via StubRegistryChecker.
//!   2. Known-real name  -> Allow (no Block findings) via StubRegistryChecker.
//!   3. Unknown name (TooNew proxy) -> Block via StubRegistryChecker conservative offline mode.
//!   4. Unreachable      -> Warn, NOT Block (W-10 fail-open policy).
//!   5. Fixture files: f9_hallucinated_import.rs and f9_hallucinated_cargo.toml
//!      both produce at least one Block finding.
//!
//! All tests use `StubRegistryChecker` or a minimal in-test stub —
//! no subprocess or network calls in this file.
//! `StubRegistryChecker` is a legitimate test double (deterministic oracle),
//! NOT a banned stub (it fully implements `RegistryExistenceChecker`).

use std::sync::Arc;

use elai_orchestrator::vibesec::dep_writetime_gate::{
    DepWriteTimeGate, RegistryExistenceChecker, RegistryVerdict, StubRegistryChecker,
    SubprocessRegistryChecker,
};
use elai_orchestrator::ssb::WriteTimeSlopVerdictKind;
use elai_orchestrator::hook_dispatcher::HookVerdictKind;

// ---------------------------------------------------------------------------
// In-test checker for Unreachable arm
// ---------------------------------------------------------------------------

/// Stub that always returns `Unreachable`.
struct UnreachableChecker;

impl RegistryExistenceChecker for UnreachableChecker {
    fn check(&self, _name: &str, _ecosystem: &str) -> RegistryVerdict {
        RegistryVerdict::Unreachable
    }
}

// ---------------------------------------------------------------------------
// Contract 1: hallucinated name -> Block(NotFound)
// ---------------------------------------------------------------------------

#[test]
fn contract_not_found_produces_block() {
    let checker = Arc::new(StubRegistryChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    // `reqquests` is in StubRegistryChecker::KNOWN_TYPOS -> NotFound
    let src = "import reqquests\n";
    let findings = gate.analyze(src, "script.py");

    assert!(
        !findings.is_empty(),
        "expected at least one finding for hallucinated dep"
    );
    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "expected Block verdict for NotFound dep, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 2: real name -> Allow (no Block)
// ---------------------------------------------------------------------------

#[test]
fn contract_real_name_no_block() {
    let checker = Arc::new(StubRegistryChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    // `requests` is in StubRegistryChecker::KNOWN_REAL -> Found
    let src = "import requests\n";
    let findings = gate.analyze(src, "script.py");

    assert!(
        !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "unexpected Block for known-real dep: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 3: TooNew (stale/abandoned) -> Block(TooNew)
//
// BLOCK-1 reconcile wired a real TooNew arm: the SubprocessRegistryChecker now
// maps registry_check.py `result=="fail"` + a "stale"/">…d" reason to
// RegistryVerdict::TooNew, and analyze() Blocks it with a WriteTimeSlopVerdictKind::TooNew
// SSB row. The offline StubRegistryChecker never returns TooNew (it has no age
// data), so this contract drives a minimal in-test checker that returns TooNew
// directly, asserting the gate Blocks on the stale arm.
// ---------------------------------------------------------------------------

struct TooNewChecker;

impl RegistryExistenceChecker for TooNewChecker {
    fn check(&self, _name: &str, _ecosystem: &str) -> RegistryVerdict {
        RegistryVerdict::TooNew { age_days: Some(900) }
    }
}

#[test]
fn contract_too_new_produces_block() {
    let checker = Arc::new(TooNewChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    let src = "import some_stale_pkg\n";
    let findings = gate.analyze(src, "script.py");

    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "expected Block for TooNew (stale/abandoned) dep, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 3b: WARN-3 — unknown real-looking names are NOT blocked offline.
//
// The WARN-3 stub fix makes offline mode safe on legitimate code: only curated
// known-hallucinated names Block; everything else is Found. A plausible real
// dep must NOT produce a Block under the offline StubRegistryChecker.
// ---------------------------------------------------------------------------

#[test]
fn contract_unknown_real_name_not_blocked_offline() {
    let checker = Arc::new(StubRegistryChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    let src = "import some_plausible_real_pkg_v2\n";
    let findings = gate.analyze(src, "script.py");

    assert!(
        !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "WARN-3: offline mode must not Block an unknown real-looking dep, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 4: Unreachable -> Warn, NOT Block (W-10 fail-open)
// ---------------------------------------------------------------------------

#[test]
fn contract_unreachable_warns_not_blocks() {
    let checker = Arc::new(UnreachableChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    let src = "import requests\n";
    let findings = gate.analyze(src, "script.py");

    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Warn),
        "expected Warn verdict for Unreachable registry, got: {:?}",
        findings
    );
    assert!(
        !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "W-10: must NOT Block on registry Unreachable, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 4b: revised W-10 — Unreachable leaves a DURABLE advisory record.
//
// The fail-open must be observable. analyze_collect() (the CLI durable path)
// now emits a BlockedDepRecord{RegistryUnreachable} for the Unreachable case so
// the CLI dep-check writes a durable advisory row. The verdict STAYS Warn (not
// Block) — fail-open is preserved; only the trace is added.
// ---------------------------------------------------------------------------

#[test]
fn contract_unreachable_emits_durable_advisory_record() {
    use elai_orchestrator::vibesec::dep_writetime_gate::DepWriteTimeGate as Gate;

    let checker = Arc::new(UnreachableChecker);
    let gate = Gate::new(None, checker);

    // `requests` is a known-real name, so the only reason a record exists at
    // all is the revised Unreachable advisory.
    let src = "import requests\n";
    let (findings, blocked) = gate.analyze_collect(src, "script.py");

    // Verdict still fail-open: Warn, never Block.
    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Warn),
        "revised W-10: Unreachable verdict must stay Warn, got: {:?}",
        findings
    );
    assert!(
        !findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "revised W-10: Unreachable must NOT Block (fail-open), got: {:?}",
        findings
    );

    // But a durable advisory record now exists so the fail-open is observable.
    assert!(
        blocked
            .iter()
            .any(|r| r.verdict_kind == WriteTimeSlopVerdictKind::RegistryUnreachable
                && r.registry_age_days.is_none()),
        "revised W-10: analyze_collect must emit a RegistryUnreachable advisory record \
         so the CLI fail-open leaves a durable trace; got: {:?}",
        blocked
            .iter()
            .map(|r| r.verdict_kind)
            .collect::<Vec<_>>()
    );

    // The advisory record maps to the registry_unreachable wire label via
    // build_slop_payload (the durable CLI write path).
    let advisory = blocked
        .iter()
        .find(|r| r.verdict_kind == WriteTimeSlopVerdictKind::RegistryUnreachable)
        .expect("advisory record present");
    let payload = Gate::build_slop_payload(advisory);
    assert_eq!(
        payload.verdict_kind, "registry_unreachable",
        "advisory payload verdict_kind must be registry_unreachable"
    );
    assert!(
        payload.registry_age_days.is_none(),
        "advisory payload carries no age_days"
    );
}

// ---------------------------------------------------------------------------
// Contract 5a: f9_hallucinated_import.rs fixture produces Block
// ---------------------------------------------------------------------------

#[test]
fn contract_fixture_hallucinated_rs_produces_block() {
    let fixture = include_str!("../../../tests/fixtures/f9_hallucinated_import.rs");
    let checker = Arc::new(StubRegistryChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    let findings = gate.analyze(fixture, "f9_hallucinated_import.rs");

    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "fixture f9_hallucinated_import.rs must produce at least one Block finding, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 5b: f9_hallucinated_cargo.toml fixture produces Block
// ---------------------------------------------------------------------------

#[test]
fn contract_fixture_hallucinated_cargo_produces_block() {
    let fixture = include_str!("../../../tests/fixtures/f9_hallucinated_cargo.toml");
    let checker = Arc::new(StubRegistryChecker);
    let gate = DepWriteTimeGate::new(None, checker);

    let findings = gate.analyze(fixture, "f9_hallucinated_cargo.toml");

    assert!(
        findings.iter().any(|f| f.verdict == HookVerdictKind::Block),
        "fixture f9_hallucinated_cargo.toml must produce at least one Block finding, got: {:?}",
        findings
    );
}

// ---------------------------------------------------------------------------
// Contract 6: BLOCK-1 reconcile — REAL subprocess against a STUBBED
// registry_check.py (deterministic, no network).
//
// Proves the SubprocessRegistryChecker speaks the shipped D-11 JSON contract
// ({claim, candidate.ecosystem} stdin -> {result, reason} stdout) end-to-end
// through a real `std::process::Command`. A tiny python stub stands in for the
// network-bound registry_check.py and emits each of the four contract arms
// keyed on the inbound `claim`, so the test runs offline and deterministically.
// This is the deterministic subprocess fixture harness the F-9 ACCEPTANCE
// (W-10 item 2) requires; the prior tests only exercised the StubRegistryChecker
// and never the subprocess parse path that BLOCK-1 fixed.
//
// Requires a `python`/`python3` on PATH; skips (passes) if none is found, since
// the contract crate must build/run in python-less CI. The real wiring is still
// proven on any dev box / RUNPOD with python present.
// ---------------------------------------------------------------------------

fn find_python() -> Option<String> {
    for cand in ["python3", "python"] {
        let ok = std::process::Command::new(cand)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            return Some(cand.to_string());
        }
    }
    None
}

#[test]
fn contract_subprocess_reconcile_all_arms() {
    let Some(python) = find_python() else {
        eprintln!("contract_subprocess_reconcile_all_arms: no python on PATH; skipping");
        return;
    };

    // Stub registry_check.py: reads {claim, candidate} on stdin, branches on the
    // claim name, emits the exact D-11 {result, reason} shape. No network.
    let stub_py = r#"
import json, sys
payload = json.loads(sys.stdin.read() or "{}")
claim = payload.get("claim", "")
if claim == "realpkg":
    print(json.dumps({"result": "pass", "reason": "cargo: 'realpkg' last updated 12d ago"}))
elif claim == "ghostpkg":
    print(json.dumps({"result": "fail", "reason": "cargo 404: crate 'ghostpkg' does not exist (likely slopsquat)"}))
elif claim == "stalepkg":
    print(json.dumps({"result": "fail", "reason": "pypi: 'stalepkg'==1.0 last published 900d ago (>730d stale)"}))
else:
    print(json.dumps({"result": "inapplicable", "reason": "claim (package name) is empty"}))
"#;

    let dir = tempfile::TempDir::new().expect("tempdir");
    let script = dir.path().join("registry_check_stub.py");
    std::fs::write(&script, stub_py).expect("write stub");

    let checker = SubprocessRegistryChecker::new(&python, &script);

    // pass -> Found{12}
    assert_eq!(
        checker.check("realpkg", "cargo"),
        RegistryVerdict::Found { age_days: Some(12) },
        "pass arm should map to Found with parsed age"
    );
    // fail + 404/does-not-exist -> NotFound (the live 404 block this BLOCK-1 fixes)
    assert_eq!(
        checker.check("ghostpkg", "cargo"),
        RegistryVerdict::NotFound,
        "404 fail arm should map to NotFound (live block of a hallucinated crate)"
    );
    // fail + stale -> TooNew{900}
    assert_eq!(
        checker.check("stalepkg", "pypi"),
        RegistryVerdict::TooNew { age_days: Some(900) },
        "stale fail arm should map to TooNew with parsed age"
    );
    // inapplicable -> Unreachable (fail-open, W-10)
    assert_eq!(
        checker.check("", "pypi"),
        RegistryVerdict::Unreachable,
        "inapplicable should map to Unreachable (fail-open)"
    );
}

#[test]
fn contract_subprocess_missing_script_is_unreachable() {
    // Script path that does not exist -> Unreachable (fail-open per W-10),
    // not a panic and not a false Block.
    let checker = SubprocessRegistryChecker::new(
        "python",
        std::path::Path::new("/nonexistent/registry_check_missing.py"),
    );
    assert_eq!(checker.check("anything", "cargo"), RegistryVerdict::Unreachable);
}
