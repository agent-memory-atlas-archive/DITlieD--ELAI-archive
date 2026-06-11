//! G-5 — reviewer-cockpit Diff-Surface-to-Review (DSR) recompute contract.
//!
//! Asserts the DSR done-gate (DD1/DD2/GD-2):
//!   (a) the DSR computes as un-vouched changed lines / total changed lines over
//!       a fixture diff where some files carry a passing-test (vouched) SSB
//!       signal and some do not (un-vouched) — the ratio MUST equal the expected
//!       value;
//!   (b) the score RE-COMPUTES BYTE-IDENTICALLY from the SAME pinned diff bytes +
//!       the SAME seq-ASC SSB stream (GD-2): the two passes serialize identically;
//!   (c) a `review_surface_scored` row LANDS in a live SSB and reads back, and
//!       its DSR + counts re-derive from the pinned git-diff + the ASC stream
//!       (the off-agent artifact, R4);
//!   (d) `total_changed_lines` (the GD-2 denominator field) is PRESENT and
//!       non-zero — the DSR is BUILT from a persisted denominator, not faked;
//!   (e) NO LLM-opinion text appears in the score (findings carry only typed
//!       deterministic kinds);
//!   (f) the SCHEMA floor includes 212 (>= pin, never ==).
//!
//! The diff is parsed by the REAL in-tree `DiffParser` (the same parser the CLI
//! uses); the vouch set is built by the REAL `build_file_signals` from a
//! seq-ASC SSB stream. The contract re-implements NONE of the F-gates (GD-1): it
//! decodes the same SSB rows the F-4 producer writes.
//!
//! No #[ignore] / stubs / placeholder assertions.

use std::collections::BTreeMap;
use std::sync::Arc;

use elai_orchestrator::focus_tunnel::diff::DiffParser;
use elai_orchestrator::review_surface::{build_file_signals, score_review_surface};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ReviewSurfaceScoredPayload, Ssb, SsbActionType, SsbConfig,
    SsbPayload, TestIntegrityEventKind, TestIntegrityEventPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::SsbReader;
use elai_core::clock::OsClock;
use tempfile::TempDir;

/// A two-file unified diff: `vouched.rs` (+3/-1 = 4 changed lines) and
/// `unvouched.rs` (+5/-1 = 6 changed lines). Total = 10 changed lines.
const FIXTURE_DIFF: &str = "\
--- a/vouched.rs
+++ b/vouched.rs
@@ -10,1 +10,3 @@
-let old = 1;
+let a = 1;
+let b = 2;
+let c = 3;
--- a/unvouched.rs
+++ b/unvouched.rs
@@ -20,1 +20,5 @@
-let z = 9;
+let p = 1;
+let q = 2;
+let r = 3;
+let s = 4;
+let t = 5;
";

fn ssb_config(dir: &TempDir, session: &str) -> SsbConfig {
    SsbConfig {
        db_path: dir.path().join("g5_dsr_ssb.db"),
        pool_size: Some(2),
        project_id: "g5-contract".to_string(),
        session_id: session.to_string(),
    }
}

/// Seed a passing F-4 TestIntegrityEvent attributed (via the ssb_files junction)
/// to `vouched.rs`, so that file is VOUCHED and `unvouched.rs` is not.
async fn seed_vouch_stream(ssb: &Ssb) {
    let payload = SsbPayload::TestIntegrityEvent(TestIntegrityEventPayload {
        test_name: "vouched::ok".to_string(),
        passed: true,
        stdout_hash: Some("abc1234567890def".to_string()),
        kind: TestIntegrityEventKind::Verified,
    });
    ssb.write_payload("test:executor", 0, payload, vec!["vouched.rs".to_string()])
        .await
        .expect("write passing TestIntegrityEvent");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dsr_is_unvouched_over_total_and_row_lands() {
    let dir = TempDir::new().expect("tempdir");
    let session = "g5-session";
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config(&dir, session), clock).expect("ssb init");

    // GD-1: the F-4 gate already ran and left this row; G-5 only decodes it.
    seed_vouch_stream(&ssb).await;

    // Parse the pinned fixture diff with the REAL parser.
    let hunks = DiffParser.parse(FIXTURE_DIFF).expect("parse fixture diff");
    assert_eq!(hunks.len(), 2, "fixture must yield two hunks");

    // Build the vouch set from the seq-ASC SSB stream + the file junction.
    let stream = ssb
        .query_session_stream_asc(session)
        .expect("read session stream");
    let files_by_seq = ssb.session_files_by_seq(session).expect("read file junction");
    let signals: BTreeMap<String, _> = build_file_signals(&stream, &files_by_seq);

    let score = score_review_surface(&hunks, &signals);

    // (a) DSR = un-vouched / total = 6 / 10 = 0.6.
    assert_eq!(score.total_changed_lines, 10, "denominator = all changed lines");
    assert_eq!(score.attention_lines, 6, "numerator = un-vouched changed lines");
    assert_eq!(score.vouched_hunks, 1);
    assert_eq!(score.unvouched_hunks, 1);
    assert!(
        (score.dsr - 0.6).abs() < 1e-12,
        "DSR must be 6/10 = 0.6, got {}",
        score.dsr
    );

    // (d) the denominator is PRESENT + non-zero (the GD-2 field G-5 builds).
    assert!(score.total_changed_lines > 0, "total_changed_lines must be persisted/non-zero");

    // (e) NO LLM opinion: the only file with a fired-gate/failing-test is none
    // here (the un-vouched file is silent-by-default), so findings is empty.
    assert!(
        score.findings.is_empty(),
        "a clean un-vouched file is NOT a finding (silent-by-default)"
    );

    // (b) GD-2 — recompute byte-identically from the SAME pinned inputs.
    let stream2 = ssb
        .query_session_stream_asc(session)
        .expect("re-read session stream");
    let files2 = ssb.session_files_by_seq(session).expect("re-read junction");
    let signals2 = build_file_signals(&stream2, &files2);
    let hunks2 = DiffParser.parse(FIXTURE_DIFF).expect("re-parse fixture diff");
    let score2 = score_review_surface(&hunks2, &signals2);
    let j1 = serde_json::to_string(&score).expect("serialize score 1");
    let j2 = serde_json::to_string(&score2).expect("serialize score 2");
    assert_eq!(j1, j2, "DSR score must recompute byte-identically (GD-2)");

    // (c) emit the off-agent row through the live Ssb + read it back.
    let base_commit = "0123456789abcdef0123456789abcdef01234567".to_string();
    let payload = SsbPayload::ReviewSurfaceScored(ReviewSurfaceScoredPayload {
        session_id: session.to_string(),
        dsr: score.dsr,
        total_changed_lines: score.total_changed_lines,
        attention_lines: score.attention_lines,
        vouched_hunks: score.vouched_hunks,
        unvouched_hunks: score.unvouched_hunks,
        base_commit: base_commit.clone(),
        diff_available: true,
        diff_unavailable_reason: None,
    });
    let seq = ssb
        .write_payload("cli:review-cockpit", 0, payload, Vec::new())
        .await
        .expect("write review_surface_scored row");
    assert!(seq > 0, "row must land with a positive seq");

    // action_type bijection holds for the new variant.
    assert_eq!(
        SsbActionType::ReviewSurfaceScored.as_sql_str(),
        "review_surface_scored"
    );
    assert_eq!(
        ssb_action_type_from_str_public("review_surface_scored")
            .expect("from_str accepts the wire tag"),
        SsbActionType::ReviewSurfaceScored
    );

    // Read the row back; its DSR + counts re-derive the SAME numbers.
    let rows = ssb
        .query_by_type(SsbActionType::ReviewSurfaceScored)
        .expect("query_by_type");
    assert_eq!(rows.len(), 1, "exactly one review_surface_scored row");
    let decoded: SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode row payload");
    match decoded {
        SsbPayload::ReviewSurfaceScored(p) => {
            assert!((p.dsr - 0.6).abs() < 1e-12, "row DSR re-reads as 0.6");
            assert_eq!(p.total_changed_lines, 10);
            assert_eq!(p.attention_lines, 6);
            assert_eq!(p.vouched_hunks, 1);
            assert_eq!(p.unvouched_hunks, 1);
            assert_eq!(p.base_commit, base_commit, "pinned base commit recorded");
            // GD-5 honesty fix: a real scored diff is `available` with no reason.
            assert!(p.diff_available, "a real scored diff is diff_available=true");
            assert!(
                p.diff_unavailable_reason.is_none(),
                "an available diff carries no unavailable reason"
            );
        }
        other => panic!("expected ReviewSurfaceScored, got {other:?}"),
    }
}

#[test]
fn empty_diff_is_dsr_zero_nothing_to_review() {
    let hunks = DiffParser.parse("").unwrap_or_default();
    let score = score_review_surface(&hunks, &BTreeMap::new());
    assert_eq!(score.total_changed_lines, 0);
    assert!(
        (score.dsr - 0.0).abs() < 1e-12,
        "an empty diff has nothing to review -> DSR 0.0"
    );
}

#[test]
fn schema_version_floor_includes_review_surface() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 212,
        "G-5 lands at SCHEMA_VERSION 212; floor pin (>=) must hold: got {SCHEMA_VERSION_PUBLIC}"
    );
}

/// FIX 1 (hollow-cockpit) — the vouch read MUST cross a real `proj-<hash>` run
/// bucket. Before the fix the CLI hardcoded `project_id = "default"`, while a
/// production `elai run` journals rows under `project_id =
/// elai_core::project_id_from_path(project_root)` = `proj-<blake3-16hex>`. Both
/// vouch readers filter `WHERE session_id = ?1 AND project_id = ?2`
/// (ssb.rs:14983 / 15008), so a `"default"` reader saw an EMPTY vouch set
/// against any real run -> every hunk un-vouched -> DSR always ~1.0.
///
/// This test journals a passing F-4 vouch under a `proj-<hash>` id (exactly the
/// id `project_id_from_path` derives), reads the vouch stream back through a
/// reader configured with the SAME derived id, and asserts the vouch set is
/// NON-empty -> DSR < 1.0 WITH a vouched hunk. It also asserts the OLD broken
/// behaviour: a reader on `"default"` finds NOTHING against the same DB.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn vouch_read_crosses_proj_hash_run_bucket_dsr_below_one() {
    let dir = TempDir::new().expect("tempdir");
    let session = "g5-proj-hash-session";
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);

    // The canonical id a real `elai run` would journal under (runtime_setup.rs:257
    // calls this exact helper on the project_root). Use the tempdir as the root.
    let project_id = elai_core::project_id_from_path(dir.path());
    assert!(
        project_id.starts_with("proj-"),
        "the run bucket id must be the derived proj-<hash> shape, got {project_id}"
    );
    assert_ne!(
        project_id, "default",
        "the run bucket must NOT be the literal that the pre-fix CLI hardcoded"
    );

    let db_path = dir.path().join("g5_proj_ssb.db");

    // WRITER (the run): journal a passing F-4 vouch for `vouched.rs` under the
    // derived proj-<hash> bucket, exactly as a production run would.
    let writer_cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: Some(2),
        project_id: project_id.clone(),
        session_id: session.to_string(),
    };
    let writer = Ssb::new(writer_cfg, clock.clone()).expect("writer ssb init");
    seed_vouch_stream(&writer).await;

    // READER (the FIXED cockpit): open the SAME db with the SAME derived id and
    // read the vouch inputs. This is the read FIX 1 makes the CLI perform.
    let reader_cfg = SsbConfig {
        db_path: db_path.clone(),
        pool_size: Some(2),
        project_id: project_id.clone(),
        session_id: session.to_string(),
    };
    let reader = Ssb::new(reader_cfg, clock.clone()).expect("reader ssb init");
    let stream = reader
        .query_session_stream_asc(session)
        .expect("read session stream under proj-hash id");
    let files_by_seq = reader
        .session_files_by_seq(session)
        .expect("read file junction under proj-hash id");
    assert!(
        !stream.is_empty(),
        "FIX 1: the vouch read MUST find the run's events under the proj-<hash> bucket"
    );
    let signals: BTreeMap<String, _> = build_file_signals(&stream, &files_by_seq);

    let hunks = DiffParser.parse(FIXTURE_DIFF).expect("parse fixture diff");
    let score = score_review_surface(&hunks, &signals);
    assert_eq!(score.vouched_hunks, 1, "vouched.rs must be vouched via the run's F-4 row");
    assert!(
        score.dsr < 1.0,
        "FIX 1: a crossed run bucket yields DSR < 1.0 (vouched hunks present), got {}",
        score.dsr
    );

    // The pre-fix behaviour reproduced: a `"default"` reader finds NOTHING in the
    // same DB (this is precisely the hollow-cockpit bug FIX 1 closes).
    let broken_cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "default".to_string(),
        session_id: session.to_string(),
    };
    let broken = Ssb::new(broken_cfg, clock).expect("broken-id ssb init");
    let broken_stream = broken
        .query_session_stream_asc(session)
        .expect("read under the broken default id");
    assert!(
        broken_stream.is_empty(),
        "regression guard: a 'default' reader (the pre-fix bug) finds zero rows in a proj-<hash> run bucket"
    );
    let broken_signals: BTreeMap<String, _> =
        build_file_signals(&broken_stream, &BTreeMap::new());
    let broken_score = score_review_surface(&hunks, &broken_signals);
    assert!(
        (broken_score.dsr - 1.0).abs() < 1e-12,
        "regression guard: the broken 'default' read scores the hollow DSR ~1.0, got {}",
        broken_score.dsr
    );
}

/// FIX 2 (fabricated reassurance) — a git error MUST be a DISTINCT state from a
/// genuine empty diff at the off-agent artifact level. The CLI distinguishes
/// them via the `diff_available` payload field: a git error writes
/// `diff_available = false` + a reason; a genuine clean tree writes
/// `diff_available = true` with `dsr = 0.0` over 0 lines. This test asserts the
/// two rows READ BACK distinctly (a git-error row never reads as a clean DSR=0).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn git_error_row_is_distinct_from_clean_empty_diff() {
    let dir = TempDir::new().expect("tempdir");
    let session = "g5-diff-status-session";
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Ssb::new(ssb_config(&dir, session), clock).expect("ssb init");

    // Row A: a GIT ERROR. The DSR/counts are sentinels; diff_available = false.
    let git_error_reason = "`git diff` exited 128: not a git repository".to_string();
    let row_a = SsbPayload::ReviewSurfaceScored(ReviewSurfaceScoredPayload {
        session_id: session.to_string(),
        dsr: 0.0,
        total_changed_lines: 0,
        attention_lines: 0,
        vouched_hunks: 0,
        unvouched_hunks: 0,
        base_commit: "HEAD".to_string(),
        diff_available: false,
        diff_unavailable_reason: Some(git_error_reason.clone()),
    });

    // Row B: a GENUINE empty diff (clean tree). DSR 0.0 over 0 lines is HONEST.
    let row_b = SsbPayload::ReviewSurfaceScored(ReviewSurfaceScoredPayload {
        session_id: session.to_string(),
        dsr: 0.0,
        total_changed_lines: 0,
        attention_lines: 0,
        vouched_hunks: 0,
        unvouched_hunks: 0,
        base_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
        diff_available: true,
        diff_unavailable_reason: None,
    });

    ssb.write_payload("cli:review-cockpit", 0, row_a, Vec::new())
        .await
        .expect("write git-error row");
    ssb.write_payload("cli:review-cockpit", 1, row_b, Vec::new())
        .await
        .expect("write clean-empty row");

    let rows = ssb
        .query_by_type(SsbActionType::ReviewSurfaceScored)
        .expect("query_by_type");
    assert_eq!(rows.len(), 2, "both rows must land");

    let mut saw_git_error = false;
    let mut saw_clean_empty = false;
    for r in &rows {
        let decoded: SsbPayload =
            rmp_serde::from_slice(&r.payload).expect("decode review row");
        if let SsbPayload::ReviewSurfaceScored(p) = decoded {
            // Both rows have dsr == 0.0; the ONLY way to tell a git error from a
            // clean review is `diff_available`. This is the field FIX 2 adds so a
            // git error never reads as a clean DSR=0.
            assert!((p.dsr - 0.0).abs() < 1e-12, "both fixtures carry dsr 0.0");
            if p.diff_available {
                saw_clean_empty = true;
                assert!(
                    p.diff_unavailable_reason.is_none(),
                    "a clean empty diff carries no error reason"
                );
            } else {
                saw_git_error = true;
                assert_eq!(
                    p.diff_unavailable_reason.as_deref(),
                    Some(git_error_reason.as_str()),
                    "a git-error row carries its reason verbatim"
                );
            }
        }
    }
    assert!(saw_git_error, "the git-error row must read back as diff_available=false");
    assert!(
        saw_clean_empty,
        "the genuine-empty row must read back as diff_available=true (distinct, honest DSR-0)"
    );
}
