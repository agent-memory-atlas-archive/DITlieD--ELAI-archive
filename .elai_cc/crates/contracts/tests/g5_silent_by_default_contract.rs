//! G-5 — reviewer-cockpit silent-by-default contract (DD7).
//!
//! Asserts the silent-by-default finding rule:
//!   (a) a clean file with NO deterministic signal is UN-VOUCHED (counts toward
//!       the DSR numerator) but is NOT surfaced as a finding;
//!   (b) a file with a FIRED F-1 VibeGateBlocked verdict IS surfaced as a
//!       finding (and is un-vouched);
//!   (c) a file with a FIRED F-2a SecurityRuleBlocked verdict IS surfaced as a
//!       finding;
//!   (d) a file with a FAILING / FabricationBlocked F-4 test IS surfaced as a
//!       finding (a passing-test signal does NOT vouch it);
//!   (e) every finding carries a TYPED deterministic kind — there is NO
//!       free-text LLM-opinion field anywhere in the score (no naive-LLM
//!       review-comment branch, stub_ban).
//!
//! The vouch/finding set is built by the REAL `build_file_signals` from a
//! seq-ASC SSB stream containing the REAL F-1 / F-2a / F-4 payloads (GD-1: the
//! gates already ran; G-5 only decodes their rows).
//!
//! No #[ignore] / stubs / placeholder assertions.

use std::collections::BTreeMap;
use std::sync::Arc;

use elai_orchestrator::focus_tunnel::diff::DiffParser;
use elai_orchestrator::review_surface::{build_file_signals, score_review_surface, FindingKind};
use elai_orchestrator::ssb::{
    SecurityRuleBlockedPayload, Ssb, SsbConfig, SsbPayload, TestIntegrityEventKind,
    TestIntegrityEventPayload, VibeGateBlockedKind, VibeGateBlockedPayload,
};
use elai_core::clock::OsClock;
use tempfile::TempDir;

/// Four single-hunk files: clean.rs (no signal), gate.rs (F-1 fired),
/// sec.rs (F-2a fired), failtest.rs (F-4 failing).
const FIXTURE_DIFF: &str = "\
--- a/clean.rs
+++ b/clean.rs
@@ -1,0 +1,2 @@
+let clean1 = 1;
+let clean2 = 2;
--- a/gate.rs
+++ b/gate.rs
@@ -1,0 +1,2 @@
+let secret = \"x\";
+let y = 2;
--- a/sec.rs
+++ b/sec.rs
@@ -1,0 +1,2 @@
+exec(cmd);
+let z = 3;
--- a/failtest.rs
+++ b/failtest.rs
@@ -1,0 +1,2 @@
+let f = 1;
+let g = 2;
";

fn ssb_config(dir: &TempDir, session: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join("g5_silent_ssb.db"),
        pool_size: Some(2),
        project_id: "g5-silent".to_string(),
        session_id: session.to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn silent_by_default_only_fired_gates_and_failing_tests_surface() {
    let dir = TempDir::new().expect("tempdir");
    let session = "g5-silent-session";
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config(&dir, session), clock).expect("ssb init");

    // F-1 VibeGateBlocked fired for gate.rs (file_path on payload).
    ssb.write_payload(
        "vibegate:executor",
        0,
        SsbPayload::VibeGateBlocked(VibeGateBlockedPayload {
            kind: VibeGateBlockedKind::Secret,
            rule_id: Some("hardcoded_secret".into()),
            cwe: Some("CWE-798".into()),
            file_path: "gate.rs".into(),
            source_blake3: "deadbeefcafef00d".into(),
            citation: None,
            fsm_state: Some("executing".into()),
        }),
        Vec::new(),
    )
    .await
    .expect("write VibeGateBlocked");

    // F-2a SecurityRuleBlocked fired for sec.rs.
    ssb.write_payload(
        "secrule:executor",
        0,
        SsbPayload::SecurityRuleBlocked(SecurityRuleBlockedPayload {
            rule_id: "no_raw_exec".into(),
            cwe: "CWE-78".into(),
            file_path: "sec.rs".into(),
            language: "rust".into(),
            candidate_blake3: "0011223344556677".into(),
        }),
        Vec::new(),
    )
    .await
    .expect("write SecurityRuleBlocked");

    // F-4 FAILING test attributed to failtest.rs (via the ssb_files junction).
    ssb.write_payload(
        "test:executor",
        0,
        SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
            test_name: "failtest::broken".into(),
            passed: false,
            stdout_hash: Some("00ff00ff00ff00ff".into()),
            kind: TestIntegrityEventKind::FabricationBlocked,
        }),
        vec!["failtest.rs".into()],
    )
    .await
    .expect("write failing TestIntegrityEvent");

    let hunks = DiffParser.parse(FIXTURE_DIFF).expect("parse fixture diff");
    assert_eq!(hunks.len(), 4, "fixture must yield four hunks");

    let stream = ssb
        .query_session_stream_asc(session)
        .expect("read session stream");
    let files_by_seq = ssb.session_files_by_seq(session).expect("read file junction");
    let signals: BTreeMap<String, _> = build_file_signals(&stream, &files_by_seq);
    let score = score_review_surface(&hunks, &signals);

    // (a) clean.rs is un-vouched but produces NO finding.
    let clean_finding = score.findings.iter().any(|f| f.file_path == "clean.rs");
    assert!(!clean_finding, "a clean un-vouched file must NOT be a finding (silent-by-default)");

    // (b)/(c)/(d): exactly three findings, one per fired/failing signal.
    let gate = score
        .findings
        .iter()
        .find(|f| f.file_path == "gate.rs")
        .expect("gate.rs must surface a finding");
    assert_eq!(gate.kind, FindingKind::VibeGateFired);

    let sec = score
        .findings
        .iter()
        .find(|f| f.file_path == "sec.rs")
        .expect("sec.rs must surface a finding");
    assert_eq!(sec.kind, FindingKind::SecurityRuleFired);

    let failt = score
        .findings
        .iter()
        .find(|f| f.file_path == "failtest.rs")
        .expect("failtest.rs must surface a finding");
    assert_eq!(failt.kind, FindingKind::TestFailed);

    assert_eq!(score.findings.len(), 3, "exactly the 3 deterministic findings, no LLM noise");

    // (e) NO free-text LLM-opinion field: serialize the score and assert no
    // finding object carries any key other than the typed (file_path, kind). A
    // naive-LLM review comment would add a free-text body; this asserts none.
    let json = serde_json::to_value(&score).expect("serialize score");
    let findings = json
        .get("findings")
        .and_then(|v| v.as_array())
        .expect("findings is an array");
    for f in findings {
        let obj = f.as_object().expect("finding is an object");
        let keys: Vec<&String> = obj.keys().collect();
        assert_eq!(
            keys.len(),
            2,
            "a finding carries ONLY (file_path, kind); no LLM-opinion body: {keys:?}"
        );
        assert!(obj.contains_key("file_path"));
        assert!(obj.contains_key("kind"));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn passing_and_failing_test_on_same_file_does_not_vouch() {
    // A file with BOTH a passing AND a failing test signal is conservatively
    // NOT vouched (the failing signal wins) and IS surfaced as a finding.
    let dir = TempDir::new().expect("tempdir");
    let session = "g5-conflict-session";
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config(&dir, session), clock).expect("ssb init");

    ssb.write_payload(
        "test:executor",
        0,
        SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
            test_name: "flaky::pass".into(),
            passed: true,
            stdout_hash: Some("aaaa1111bbbb2222".into()),
            kind: TestIntegrityEventKind::Verified,
        }),
        vec!["flaky.rs".into()],
    )
    .await
    .expect("write passing test");
    ssb.write_payload(
        "test:executor",
        0,
        SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
            test_name: "flaky::fail".into(),
            passed: false,
            stdout_hash: Some("cccc3333dddd4444".into()),
            kind: TestIntegrityEventKind::Verified,
        }),
        vec!["flaky.rs".into()],
    )
    .await
    .expect("write failing test");

    let diff = "\
--- a/flaky.rs
+++ b/flaky.rs
@@ -1,0 +1,3 @@
+let a = 1;
+let b = 2;
+let c = 3;
";
    let hunks = DiffParser.parse(diff).expect("parse diff");
    let stream = ssb.query_session_stream_asc(session).expect("stream");
    let files = ssb.session_files_by_seq(session).expect("junction");
    let signals = build_file_signals(&stream, &files);
    let score = score_review_surface(&hunks, &signals);

    assert_eq!(score.vouched_hunks, 0, "a failing test wins: not vouched");
    assert_eq!(score.unvouched_hunks, 1);
    assert!(
        score.findings.iter().any(|f| f.kind == FindingKind::TestFailed),
        "the failing test surfaces a finding"
    );
}
