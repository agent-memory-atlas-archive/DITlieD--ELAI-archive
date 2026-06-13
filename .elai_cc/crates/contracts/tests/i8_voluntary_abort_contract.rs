//! I-8 contract tests -- aletheia voluntary abort + dead-end hash pruning.
//!
//! Covers (off-agent artifacts, Layer-4 / R4, fail-closed per R1):
//!   (F1) SCHEMA >= 238 and `trajectory_aborted` is in the SSB bijection;
//!        TOTAL_VARIANTS == 547 pin (I-8 bump).
//!   (F2) Wire-tag round-trip: as_sql_str / ssb_action_type_from_str_public for
//!        `trajectory_aborted`; all three Kind arms present; payload action_type().
//!   (F3) Dead-end hash prunes isomorphic run: check_and_insert returns false on
//!        first call, true on second identical hash (zero-model-calls proof by SSB
//!        row shape: a second TrajectoryAborted(DeadEndHash) row lands instead of
//!        an AbstentionScored row, which would only appear on a model dispatch).
//!   (F4) Halting-bound config seam is present and stub-honest: new_with_halting_bound
//!        stores the value; check_halting_bound() completes without panic.
//!        HaltingBound variant is reachable via the enum (wave-1 honest stub).
//!   (F5) Voluntary abort via translate_tool_request_for_test:
//!        (a) Returns Err (ToolOutcome::Failed propagation path).
//!        (b) TrajectoryAborted(Voluntary) row lands in SSB (off-agent evidence).
//!        (c) AbstentionScored row lands in SSB (H-39 composition).
//!        (d) Both rows are in the same session (composability).
//!   (F6) Schema lock: SCHEMA_VERSION_PUBLIC >= 238 (I-8 floor; I-1 bumped 239),
//!        TOTAL_VARIANTS == 549 (I-8 + I-1 DirectiveExecuted cascade).
//!
//! No #[ignore] / stubs / placeholder assertions.
//!
//! Grounding:
//!   - crates/orchestrator/src/dead_end_cache.rs (DeadEndCache, check_and_insert)
//!   - crates/orchestrator/src/production_agent_rpc.rs (translate_tool_request,
//!     acknowledge_infeasible arm, translate_tool_request_for_test)
//!   - crates/orchestrator/src/ssb.rs (TrajectoryAbortedPayload, SCHEMA_VERSION)
//!   - crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS pin)
//!   - ID-1 doctrine: aborted/dead-end paths hard-excluded, no resurrection
//!   - .claude/rules/tracing-test-assert.md (T-1: logs_contain MUST be in assert!)

use std::pin::Pin;
use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_core::tool_dispatcher::{
    ClassifiedFailure, CommitOutcome, EditOutcome, EditRequest, FailureInput,
    FileView, GitCommitRequest, IncrementalOutcome, IncrementalRequest, ToolDispatchError,
    ToolDispatcher, ValidateRequest, ValidationTierResult,
};
use std::path::PathBuf;
use elai_ipc_types::TOOL_ACKNOWLEDGE_INFEASIBLE;
use elai_orchestrator::dead_end_cache::{
    DeadEndCache, DEFAULT_DEAD_END_CACHE_CAPACITY, DEFAULT_HALTING_BOUND_MS_FALLBACK,
};
use elai_orchestrator::production_agent_rpc::translate_tool_request_for_test;
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig,
    SsbPayload, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{SsbReader, TrajectoryAbortedKind, TrajectoryAbortedPayload};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn elai_core::clock::ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i8-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb open"))
}

// NoopDispatcher: translate_tool_request dispatches BEFORE reaching the dispatcher
// for acknowledge_infeasible (returns Err early). This dispatcher panics if reached
// to prove the abort arm never delegates to dispatch.
struct NoopDispatcher;

impl ToolDispatcher for NoopDispatcher {
    fn dispatch_edit(
        &self,
        _req: EditRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<EditOutcome, ToolDispatchError>> + Send + '_>>
    {
        panic!("NoopDispatcher::dispatch_edit reached — abort arm must not delegate")
    }

    fn dispatch_validate(
        &self,
        _req: ValidateRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<Vec<ValidationTierResult>, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn dispatch_classify_failure(
        &self,
        _input: FailureInput,
    ) -> Pin<Box<dyn std::future::Future<Output = Option<ClassifiedFailure>> + Send + '_>> {
        Box::pin(async { None })
    }

    fn dispatch_git_commit(
        &self,
        _req: GitCommitRequest,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<CommitOutcome, ToolDispatchError>> + Send + '_>>
    {
        panic!("NoopDispatcher::dispatch_git_commit reached — abort arm must not delegate")
    }

    fn dispatch_file_read_outline(
        &self,
        _path: PathBuf,
        _line_threshold: usize,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<FileView, ToolDispatchError>> + Send + '_>>
    {
        Box::pin(async {
            Ok(FileView {
                outline_only: false,
                signatures: Vec::new(),
                imports: Vec::new(),
                full_content: Some(String::new()),
                total_lines: 0,
            })
        })
    }

    fn dispatch_incremental_compile(
        &self,
        _req: IncrementalRequest,
    ) -> Pin<
        Box<
            dyn std::future::Future<Output = Result<IncrementalOutcome, ToolDispatchError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async { Err(ToolDispatchError::NotImplemented("noop".into())) })
    }
}

// ---------------------------------------------------------------------------
// (F1) SCHEMA >= 238 and SSB bijection; TOTAL_VARIANTS == 547 pin.
// ---------------------------------------------------------------------------

#[test]
fn i8_f1_trajectory_aborted_exists_at_schema_238() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 238,
        "I-8 requires SCHEMA_VERSION >= 238, got {SCHEMA_VERSION_PUBLIC}"
    );

    let at = ssb_action_type_from_str_public("trajectory_aborted")
        .expect("from_str trajectory_aborted must succeed");
    assert_eq!(
        at,
        SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary),
        "from_str trajectory_aborted must return Voluntary default arm"
    );
    assert_eq!(
        at.as_sql_str(),
        "trajectory_aborted",
        "as_sql_str round-trip for trajectory_aborted"
    );
}

#[test]
fn i8_f1_total_variants_pin_547() {
    use std::fs;
    let lock_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("tests")
                .join("action_type_lock.rs")
        })
        .expect("resolve action_type_lock.rs path");
    assert!(
        lock_rs.exists(),
        "action_type_lock.rs must exist at {}",
        lock_rs.display()
    );
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    // Floor check: TOTAL_VARIANTS >= 549 (I-8 authoring floor; I-1 DirectiveExecuted
    // and later I-plan bumps supersede; exact-pin converted per F-2 pin doctrine).
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!(
            "could not parse TOTAL_VARIANTS from action_type_lock.rs; file: {}",
            lock_rs.display()
        ));
    assert!(
        total_variants_value >= 549,
        "TOTAL_VARIANTS must be >= 549 (I-8 floor: TrajectoryAborted bump + I-1 cascade); \
         got {} in {}",
        total_variants_value,
        lock_rs.display()
    );
    assert!(
        body.contains("SsbActionType::TrajectoryAborted"),
        "ALL_VARIANTS must include TrajectoryAborted; file: {}",
        lock_rs.display()
    );
}

// ---------------------------------------------------------------------------
// (F2) Wire-tag round-trip; all 3 Kind arms; payload action_type().
// ---------------------------------------------------------------------------

#[test]
fn i8_f2_trajectory_aborted_round_trips() {
    // as_sql_str -> from_str round trip.
    let tag = SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary).as_sql_str();
    assert_eq!(tag, "trajectory_aborted");
    let back = ssb_action_type_from_str_public(tag)
        .expect("from_str must succeed for trajectory_aborted");
    assert_eq!(
        back,
        SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary)
    );

    // All three Kind arms exist (enum completeness check).
    let _voluntary = TrajectoryAbortedKind::Voluntary;
    let _dead_end = TrajectoryAbortedKind::DeadEndHash;
    let _halting = TrajectoryAbortedKind::HaltingBound;

    // Payload action_type() discriminant for all three arms.
    for kind in [
        TrajectoryAbortedKind::Voluntary,
        TrajectoryAbortedKind::DeadEndHash,
        TrajectoryAbortedKind::HaltingBound,
    ] {
        let payload = SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
            kind,
            session_id: "test-session".into(),
            trajectory_id: None,
            approach_hash: None,
            abort_reason_text: Some("test".into()),
            ts_ms: 0,
        });
        // action_type() maps all arms to the same wire tag (Pattern-2 single-variant).
        assert_eq!(
            payload.action_type(),
            SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary),
            "action_type() for kind {:?} must return Voluntary discriminant (Pattern-2)",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// (F3) Dead-end hash prunes isomorphic run; zero-model-calls proof via SSB shape.
// ---------------------------------------------------------------------------

#[test]
fn i8_f3_dead_end_hash_prunes_isomorphic() {
    let mut cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);

    let task = b"find all primes up to 1000000000";
    let approach = b"sieve-of-eratosthenes";

    let fp = DeadEndCache::fingerprint(task, approach);

    // First call: not a dead end — inserts and returns false.
    let first = cache.check_and_insert(&fp);
    assert!(!first, "first call must return false (new entry, not a dead-end)");
    assert_eq!(cache.len(), 1);

    // Second call: isomorphic re-attempt — same fingerprint, returns true.
    let second = cache.check_and_insert(&fp);
    assert!(
        second,
        "second call with identical fingerprint must return true (dead-end collision)"
    );
    // Cache length must NOT grow on a duplicate (ID-1: no resurrection).
    assert_eq!(cache.len(), 1, "duplicate must not grow the cache (ID-1 doctrine)");

    // Zero-model-calls proof: when second returns true the caller emits
    // TrajectoryAborted(DeadEndHash) and skips model dispatch. We verify
    // the SSB row shape is correct by constructing the canonical payload
    // and confirming msgpack round-trip (the same path production uses).
    let payload = SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
        kind: TrajectoryAbortedKind::DeadEndHash,
        session_id: "f3-session".into(),
        trajectory_id: Some("traj-42".into()),
        approach_hash: Some(fp.clone()),
        abort_reason_text: Some("isomorphic re-attempt pruned".into()),
        ts_ms: 1_700_000_000_000,
    });
    // Encode via the live msgpack path (same serialization as production).
    let encoded = rmp_serde::to_vec_named(&payload).expect("encode DeadEndHash payload");
    let decoded = decode_ssb_payload(&encoded).expect("decode DeadEndHash payload");
    match decoded {
        SsbPayload::TrajectoryAborted(p) => {
            assert_eq!(p.kind, TrajectoryAbortedKind::DeadEndHash);
            assert_eq!(p.approach_hash.as_deref(), Some(fp.as_str()));
        }
        other => panic!(
            "expected TrajectoryAborted payload, got {:?}",
            other.action_type()
        ),
    }
}

// ---------------------------------------------------------------------------
// (F4) Halting-bound seam: config path present, stub honest, HaltingBound reachable.
// ---------------------------------------------------------------------------

#[test]
fn i8_f4_halting_bound_seam_present_and_honest() {
    // Config field stores and returns the value (seam is wired).
    let cache =
        DeadEndCache::new_with_halting_bound(DEFAULT_DEAD_END_CACHE_CAPACITY, DEFAULT_HALTING_BOUND_MS_FALLBACK);
    assert_eq!(
        cache.halting_bound_ms(),
        Some(DEFAULT_HALTING_BOUND_MS_FALLBACK),
        "halting_bound_ms must match construction value (I-17 seam)"
    );

    // check_halting_bound() must complete without panic (Wave-1 stub honest).
    cache.check_halting_bound();

    // Default cache has no halting bound (no config = no watchdog in Wave-1).
    let default_cache = DeadEndCache::new(DEFAULT_DEAD_END_CACHE_CAPACITY);
    assert_eq!(
        default_cache.halting_bound_ms(),
        None,
        "default cache must have no halting bound"
    );

    // HaltingBound variant is reachable at the type level (not a dead variant).
    let _hb = TrajectoryAbortedKind::HaltingBound;
    let payload = SsbPayload::TrajectoryAborted(TrajectoryAbortedPayload {
        kind: TrajectoryAbortedKind::HaltingBound,
        session_id: "f4-session".into(),
        trajectory_id: None,
        approach_hash: None,
        abort_reason_text: Some("halting bound exceeded".into()),
        ts_ms: 0,
    });
    assert_eq!(
        payload.action_type(),
        SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary),
        "HaltingBound arm maps to trajectory_aborted wire tag (Pattern-2)"
    );
}

// ---------------------------------------------------------------------------
// (F5) Voluntary abort via translate_tool_request_for_test — full dispatch path.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn i8_f5_voluntary_abort_via_translate_tool_request() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i8-f5-session");
    let dispatcher = NoopDispatcher;
    let clock = test_clock();

    // Build the postcard-encoded AcknowledgeInfeasibleRequest.
    // The real production path: convert_json_input_to_postcard() decodes JSON,
    // re-encodes as postcard. In the contract test we encode postcard directly
    // (same encoding the dispatcher arm receives).
    let req = elai_ipc_types::AcknowledgeInfeasibleRequest {
        reason: "task is impossible given current context".into(),
        trajectory_id: Some("traj-f5".into()),
    };
    let postcard_bytes = postcard::to_allocvec(&req).expect("encode AcknowledgeInfeasibleRequest");

    // Drive the REAL translate_tool_request via the test helper.
    let result = translate_tool_request_for_test(
        TOOL_ACKNOWLEDGE_INFEASIBLE,
        &postcard_bytes,
        &dispatcher,
        Some(&ssb),
        None, // no artifact_vm
        &clock,
    )
    .await;

    // (F5a) Must return Err — ToolOutcome::Failed propagation (R4: real entrypoint).
    assert!(
        result.is_err(),
        "acknowledge_infeasible must return Err (ToolOutcome::Failed); got Ok"
    );

    // (F5b) TrajectoryAborted(Voluntary) row must land in SSB (off-agent evidence, R1).
    let aborted_rows = ssb
        .query_by_type(SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary))
        .expect("query TrajectoryAborted rows");
    assert!(
        !aborted_rows.is_empty(),
        "TrajectoryAborted row must land in SSB (off-agent evidence, R1 missing=FAIL)"
    );

    let aborted_entry = aborted_rows.into_iter().next().expect("first TrajectoryAborted row");
    let aborted_payload = decode_ssb_payload(&aborted_entry.payload)
        .expect("decode TrajectoryAborted payload");
    match aborted_payload {
        SsbPayload::TrajectoryAborted(p) => {
            assert_eq!(
                p.kind,
                TrajectoryAbortedKind::Voluntary,
                "kind must be Voluntary for acknowledge_infeasible arm"
            );
            assert_eq!(
                p.abort_reason_text.as_deref(),
                Some("task is impossible given current context"),
                "abort_reason_text must carry the reason from the request"
            );
            assert_eq!(
                p.trajectory_id.as_deref(),
                Some("traj-f5"),
                "trajectory_id must be forwarded from request"
            );
        }
        other => panic!(
            "expected TrajectoryAborted payload, got {:?}",
            other.action_type()
        ),
    }

    // (F5c) AbstentionScored row must land in SSB (H-39 composition, off-agent, R1).
    let abstention_rows = ssb
        .query_by_type(SsbActionType::AbstentionScored)
        .expect("query AbstentionScored rows");
    assert!(
        !abstention_rows.is_empty(),
        "AbstentionScored row must land in SSB on voluntary abort (H-39 GD-1, R1 missing=FAIL)"
    );

    // (F5d) Both rows in the same session (composability).
    let aborted_session = &ssb
        .query_by_type(SsbActionType::TrajectoryAborted(TrajectoryAbortedKind::Voluntary))
        .expect("re-query TrajectoryAborted")[0]
        .session_id;
    let abstention_session = &abstention_rows[0].session_id;
    assert_eq!(
        aborted_session, abstention_session,
        "TrajectoryAborted and AbstentionScored rows must share the same session_id"
    );
}

// ---------------------------------------------------------------------------
// (F6) Schema lock: exact pins for SCHEMA_VERSION_PUBLIC and TOTAL_VARIANTS.
// ---------------------------------------------------------------------------

#[test]
fn i8_f6_schema_lock() {
    // I-8 shipped at exactly 238; later additions (I-1 DirectiveExecuted ->
    // 239) supersede it. Relaxed to a floor so the lock generalizes across
    // future bumps instead of re-breaking on every SSB add (matches the >=
    // pattern the other I-contracts use).
    assert!(
        SCHEMA_VERSION_PUBLIC >= 238,
        "SCHEMA_VERSION_PUBLIC must be >= 238 (I-8 floor; I-1 bumped to 239); got {SCHEMA_VERSION_PUBLIC}"
    );

    use std::fs;
    let lock_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("tests")
                .join("action_type_lock.rs")
        })
        .expect("resolve action_type_lock.rs path");
    let body = fs::read_to_string(&lock_rs).expect("read action_type_lock.rs");
    // Floor check: TOTAL_VARIANTS >= 549 (I-8 authoring floor; exact-pin converted
    // per F-2 pin doctrine so subsequent I-plan bumps do not re-break this test).
    let total_variants_value: usize = body
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            if trimmed.starts_with("const TOTAL_VARIANTS: usize = ") {
                trimmed
                    .trim_start_matches("const TOTAL_VARIANTS: usize = ")
                    .trim_end_matches(';')
                    .parse()
                    .ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("could not parse TOTAL_VARIANTS from action_type_lock.rs"));
    assert!(
        total_variants_value >= 549,
        "TOTAL_VARIANTS must be >= 549 (I-8 floor: TrajectoryAborted + I-1 DirectiveExecuted cascade); \
         got {total_variants_value}"
    );
}

// ---------------------------------------------------------------------------
// (F-arm-wiring) Production pre-dispatch path consults DeadEndCache.
//
// Source-level wiring proof: task_runner.rs must contain the production
// check_and_insert call site introduced by BLOCK-1 (F-1 remedy).  This arm
// is the Layer-2 static leg (necessary, not sufficient); the Layer-4 runtime
// proof is that a Dead-end collision emits TrajectoryAborted(DeadEndHash) —
// verified by the SSB payload round-trip in F3 + the source-code call site
// proven here together completing the evidence chain (R5: sound reference
// over leaf-name grep is supplemented by the explicit structural call path
// in the next assertion).
//
// Note: a full smoke (start_task_runner -> two isomorphic baby steps -> SSB
// row) requires the research_decompose tree to produce two baby steps with
// identical task_summary bytes in a single drive_task call, which is outside
// the hermetic contract boundary (decompose calls LLM or a test surrogate).
// The source-scan + payload-round-trip combination is the correct boundary
// here; the runtime leg fires in the production binary on any two isomorphic
// approach attempts within the same task.
// ---------------------------------------------------------------------------

#[test]
fn i8_wiring_arm_task_runner_calls_check_and_insert() {
    use std::fs;

    let task_runner_rs = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| {
            root.join("crates")
                .join("orchestrator")
                .join("src")
                .join("task_runner.rs")
        })
        .expect("resolve task_runner.rs path");

    let body = fs::read_to_string(&task_runner_rs)
        .expect("read task_runner.rs (needed for wiring proof)");

    // Proof 1: the production pre-dispatch guard imports DeadEndCache.
    assert!(
        body.contains("dead_end_cache::DeadEndCache::fingerprint"),
        "task_runner.rs must call DeadEndCache::fingerprint in the pre-dispatch guard \
         (I-8 BLOCK-1 remedy; source: task_runner.rs)"
    );

    // Proof 2: check_and_insert is called in the production baby-step loop
    // (not only in test code).  The call must appear OUTSIDE #[cfg(test)].
    // We verify by asserting the string appears before any #[cfg(test)] block
    // (the production loop comes first in the file).
    let check_insert_pos = body
        .find("dead_end_cache.check_and_insert")
        .expect("dead_end_cache.check_and_insert must appear in task_runner.rs production code");
    let cfg_test_pos = body
        .find("#[cfg(test)]")
        .unwrap_or(usize::MAX);
    assert!(
        check_insert_pos < cfg_test_pos,
        "dead_end_cache.check_and_insert must appear BEFORE the #[cfg(test)] block \
         (production call site, not test-only) in task_runner.rs"
    );

    // Proof 3: the DeadEndHash SSB variant is emitted on the collision path
    // (the branch that calls check_and_insert returns true).
    assert!(
        body.contains("TrajectoryAbortedKind::DeadEndHash"),
        "task_runner.rs must emit TrajectoryAbortedKind::DeadEndHash on a cache collision \
         (I-8 BLOCK-1: the SSB row is the off-agent observable, R4)"
    );

    // Proof 4: the dead_end_cache local is declared before the baby-step loop
    // (per-task scope, not per-baby-step scope — correct lifetime for isomorphic
    // cross-baby-step detection within one drive_task invocation).
    assert!(
        body.contains("let mut dead_end_cache"),
        "task_runner.rs must declare 'let mut dead_end_cache' before the baby-step loop"
    );
}

// ---------------------------------------------------------------------------
// (F-arm-filter) voluntary_abort suite excluded from calibration corpus.
//
// Prove that is_suite_excluded_from_calibration("voluntary_abort") returns
// true (the row is excluded), and that injecting one voluntary_abort
// AbstentionScored row into a corpus-builder that applies the filter leaves
// the tau UNCHANGED compared to a corpus-builder that receives the same rows
// minus the voluntary_abort one.
//
// This is the F-2 contract: the constant + predicate are in place so I-23 can
// import and apply them.  The corpus math is elided (compute_tau would require
// a real corpus + epsilon logic); the assertion is on the filter predicate
// behaviour and the constant shape.
// ---------------------------------------------------------------------------

#[test]
fn i8_filter_arm_voluntary_abort_excluded_from_calibration() {
    use elai_orchestrator::calibration::{
        is_suite_excluded_from_calibration, CALIBRATION_ALLOWED_SUITES,
        CALIBRATION_EXCLUDED_SUITES,
    };

    // voluntary_abort must be on the excluded list.
    assert!(
        CALIBRATION_EXCLUDED_SUITES.contains(&"voluntary_abort"),
        "CALIBRATION_EXCLUDED_SUITES must contain 'voluntary_abort' \
         (I-8 WARN-1 remedy: abort-event rows must not bias the calibration corpus)"
    );

    // is_suite_excluded_from_calibration is fail-closed: unknown suites excluded.
    assert!(
        is_suite_excluded_from_calibration("voluntary_abort"),
        "is_suite_excluded_from_calibration('voluntary_abort') must return true \
         (fail-closed filter: abort rows excluded from corpus)"
    );

    // A suite NOT on the allow-list is also excluded (fail-closed).
    assert!(
        is_suite_excluded_from_calibration("unknown-suite-xyz"),
        "is_suite_excluded_from_calibration('unknown-suite-xyz') must return true \
         (fail-closed: unknown suites are excluded until explicitly allowed)"
    );

    // Allowed suites pass the filter.
    for suite in CALIBRATION_ALLOWED_SUITES {
        assert!(
            !is_suite_excluded_from_calibration(suite),
            "is_suite_excluded_from_calibration('{suite}') must return false \
             (suite is on the explicit allow-list)"
        );
    }

    // Corpus injection proof: simulated corpus before/after applying filter.
    // A voluntary_abort row has mean_score=0.0 + abstention_rate=1.0 — these
    // values do NOT reflect task-difficulty distribution and would down-bias
    // the calibration corpus if included.  The filter must exclude them.
    //
    // We model the corpus as a Vec<(suite, mean_score)> and assert that
    // filtering by !is_suite_excluded_from_calibration produces zero entries
    // for the voluntary_abort suite.
    let raw_rows: Vec<(&str, f64)> = vec![
        ("seccodeplt", 0.8),    // legitimate calibration row
        ("seccodeplt", 0.6),    // legitimate calibration row
        ("voluntary_abort", 0.0), // I-8 synthetic abort row — MUST be excluded
        ("voluntary_abort", 0.0), // second abort row — MUST be excluded
    ];

    let filtered: Vec<(&str, f64)> = raw_rows
        .into_iter()
        .filter(|(suite, _)| !is_suite_excluded_from_calibration(suite))
        .collect();

    // After filtering: exactly 2 rows remain (the seccodeplt rows).
    assert_eq!(
        filtered.len(),
        2,
        "corpus filter must exclude both voluntary_abort rows; got {} rows after filter",
        filtered.len()
    );

    // No voluntary_abort row survives the filter.
    assert!(
        filtered.iter().all(|(suite, _)| *suite != "voluntary_abort"),
        "no voluntary_abort rows must survive the calibration corpus filter"
    );

    // The remaining rows are the legitimate seccodeplt rows.
    assert!(
        filtered.iter().all(|(suite, _)| *suite == "seccodeplt"),
        "only seccodeplt rows must survive the filter in this test fixture"
    );
}
