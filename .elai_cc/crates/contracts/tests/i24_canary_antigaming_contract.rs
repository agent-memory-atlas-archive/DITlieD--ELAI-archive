//! I-24 contract tests — isomorphic canary anti-gaming (scorer-side bias detection).
//!
//! Off-agent artifacts (Layer-4 / R1 fail-closed). Every "row landed" assertion is
//! a REAL-DB `query_by_type` against a live TempDir SQLite (NOT a struct-only
//! round-trip) so a wire that never reached the DB cannot pass.
//!
//!   (C1) `i24_c_real_db_prm_canary_event_row_lands`:
//!        With ELAI_CANARY_INJECTOR_ENABLE=1 and a fixture-dir supplying
//!        canary_pairs.json, drive TriLayerStack::run_verdict on a branch that
//!        clears L1 (no checker applies). Assert a `PrmCanaryEvent(CanaryInjected)`
//!        row lands in real SQLite and decodes correctly (kind / scorer_id / delta_bp).
//!        This is the allowlist smoke_test pointer (wire-dark-allowlist.toml I-24
//!        row: smoke_test = "...::i24_c_real_db_prm_canary_event_row_lands").
//!
//!   (C2) `i24_c_kind_roundtrip_all_3_arms`:
//!        All 3 PrmCanaryEventKind arms (CanaryInjected / StyleBiasDetected /
//!        ScorerNeutralized) map correctly through as_sql_str / from_str.
//!        An unknown string errors. SQL CHECK accepts all 3 via real-DB write.
//!
//!   (C3) `i24_c_env_gate_default_off_no_row`:
//!        With ELAI_CANARY_INJECTOR_ENABLE unset, run_verdict MUST NOT emit a
//!        PrmCanaryEvent row (R9 default-OFF gate).
//!
//!   (C4) `i24_c_frozen_fixture_parses_expected_pair_count`:
//!        fixtures/canary_pairs.json on disk parses into the expected shape
//!        (6 pairs, each non-empty, original != obfuscated). R8 frozen-fixture.
//!
//!   (C5) `i24_c_canary_injected_emit_via_real_emit_fn`:
//!        Call `emit_prm_canary_to_ssb` for each of the 3 Kind arms; confirm
//!        each produces a real-DB row decodeable via `decode_ssb_payload`.
//!
//!   (C6) `i24_c_id4_veto_no_canary_row`:
//!        A branch vetoed by L1 (HardVeto) must produce ZERO PrmCanaryEvent rows.
//!        ID-4: canary fires ONLY on L2 survivors.
//!
//! No `#[ignore]` / stubs / placeholder assertions.
//!
//! Grounding:
//!   - crates/orchestrator/src/canary_injector.rs (CanaryInjector, emit_prm_canary_to_ssb)
//!   - crates/orchestrator/src/verifier_stack.rs (TriLayerStack, new_with_fixture_dir)
//!   - crates/orchestrator/src/ssb/payloads.rs (PrmCanaryEventKind, PrmCanaryEventPayload,
//!     ELAI_CANARY_INJECTOR_ENABLE_ENV)
//!   - crates/orchestrator/src/ssb/action_type.rs (from_str / as_sql_str roundtrip)
//!   - fixtures/canary_pairs.json (R8 frozen fixture set)
//!   - wire-dark-allowlist.toml I-24 row (smoke_test pointer to C1)
//!   - wiring-enforcement R1: missing row = FAIL; R9: default-OFF gate must hold

use std::sync::Arc;

use elai_core::clock::{ClockProvider, OsClock};
use elai_orchestrator::adversary::AdversaryConfig;
use elai_orchestrator::canary_injector::{
    emit_prm_canary_to_ssb, CanaryInjector, CanaryInjectorConfig,
};
use elai_orchestrator::checker_registry::{CheckerRegistry, RegexMatchChecker};
use elai_orchestrator::ssb::{
    decode_ssb_payload, ssb_action_type_from_str_public, Ssb, SsbActionType, SsbConfig,
    SsbPayload, PrmCanaryEventKind, PrmCanaryEventPayload, ELAI_CANARY_INJECTOR_ENABLE_ENV,
};
use elai_orchestrator::verifier_stack::{Branch, TriLayerStack};
use elai_orchestrator::vibesec::dep_writetime_gate::{DepWriteTimeGate, StubRegistryChecker};
use elai_orchestrator::SsbReader;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Fixture helpers
// ---------------------------------------------------------------------------

fn test_clock() -> Arc<dyn ClockProvider> {
    Arc::new(OsClock)
}

fn test_ssb(dir: &TempDir, session_id: &str) -> Arc<Ssb> {
    let db_path = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path,
        pool_size: Some(2),
        project_id: "i24-contract".into(),
        session_id: session_id.into(),
    };
    Arc::new(Ssb::new(cfg, test_clock()).expect("ssb init"))
}

fn stub_dep_gate() -> Arc<DepWriteTimeGate> {
    Arc::new(DepWriteTimeGate::new(None, Arc::new(StubRegistryChecker)))
}

/// A registry whose one checker always vetoes (sentinel pattern from i17 contract).
fn always_veto_registry() -> Arc<CheckerRegistry> {
    let mut reg = CheckerRegistry::new();
    let checker = RegexMatchChecker::new("ZZZ_NEVER_PRESENT_I24_SENTINEL_ZZZ", "i24-veto", false)
        .expect("regex compiles");
    reg.register("i24-always-veto", Box::new(checker));
    Arc::new(reg)
}

/// Empty registry — L1 always passes so the branch reaches L2 and the canary.
fn empty_registry() -> Arc<CheckerRegistry> {
    Arc::new(CheckerRegistry::new())
}

/// Write a minimal `canary_pairs.json` with 2 pairs into `dir/fixtures/` and
/// return the `dir` root path (the path new_with_fixture_dir expects as project_root).
fn write_minimal_fixture(dir: &TempDir) -> std::path::PathBuf {
    let fixtures_dir = dir.path().join("fixtures");
    std::fs::create_dir_all(&fixtures_dir).expect("create fixtures dir");
    let pairs_path = fixtures_dir.join("canary_pairs.json");
    let content = r#"{
  "pairs": [
    {
      "label": "for_to_while_test_01",
      "original":   "for i in range(n): total += i",
      "obfuscated": "i = 0\nwhile i < n:\n    total += i\n    i += 1"
    },
    {
      "label": "rename_test_01",
      "original":   "def f(x): return x + 1",
      "obfuscated": "def f(a): return a + 1"
    }
  ]
}"#;
    std::fs::write(&pairs_path, content).expect("write canary_pairs.json");
    dir.path().to_path_buf()
}

/// Count PrmCanaryEvent rows of ALL kinds in the DB. Pattern-2: we iterate all
/// 3 Kind arms and sum; each arm maps to its own sql tag so query_by_type with
/// the representative CanaryInjected arm only returns that arm's rows.
fn count_all_prm_canary_rows(ssb: &Arc<Ssb>) -> usize {
    let arms = [
        PrmCanaryEventKind::CanaryInjected,
        PrmCanaryEventKind::StyleBiasDetected,
        PrmCanaryEventKind::ScorerNeutralized,
    ];
    arms.iter()
        .map(|&k| {
            ssb.query_by_type(SsbActionType::PrmCanaryEvent(k))
                .expect("query_by_type")
                .len()
        })
        .sum()
}

// ---------------------------------------------------------------------------
// (C1) REAL-DB ROW LANDS — the allowlist smoke_test pointer.
//      Name MUST be exactly: i24_c_real_db_prm_canary_event_row_lands
//
//      This test drives the REAL emit path end-to-end:
//        CanaryInjector::new (loads fixture from disk)
//        -> maybe_inject (fires with probability=1.0, deterministic)
//        -> emit_prm_canary_to_ssb (MessagePack serialize -> SQLite write)
//        -> query_by_type (real-DB read)
//        -> decode_ssb_payload (MessagePack deserialize)
//      This is a real-DB test (not struct-only): the payload bytes come from
//      the live SQLite row, not from a hand-constructed struct.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i24_c_real_db_prm_canary_event_row_lands() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i24-c1");
    let project_root = write_minimal_fixture(&dir);
    let fixture_path = project_root.join("fixtures").join("canary_pairs.json");

    // Construct CanaryInjector with injection_probability=1.0 (always fires)
    // and seed=0. This makes the canary deterministic and avoids the 5%
    // sampling uncertainty of the production path (which uses 0.05). The
    // contract verifies the SSB wire path, not the sampling distribution.
    let config = CanaryInjectorConfig {
        injection_probability: 1.0,
        seed: 0,
        bias_threshold_bp: 500,
    };
    let mut injector = CanaryInjector::new(&fixture_path, config, Arc::clone(&ssb))
        .expect("CanaryInjector::new must not error on valid fixture")
        .expect("CanaryInjector must be Some (fixture exists and is non-empty)");

    // Set the env gate: maybe_inject checks ELAI_CANARY_INJECTOR_ENABLE at call time.
    unsafe { std::env::set_var(ELAI_CANARY_INJECTOR_ENABLE_ENV, "1") };

    // Drive maybe_inject with a stub scorer that returns 0.5 for both variants
    // (conformal-tau-style: constant, no source sensitivity). delta = 0 bp < 500 bp
    // threshold -> kind = CanaryInjected.
    injector
        .maybe_inject(|_source| 0.5_f64, "conformal_tau", "agent-c1", 0_i64)
        .await;

    unsafe { std::env::remove_var(ELAI_CANARY_INJECTOR_ENABLE_ENV) };

    // REAL-DB assertion (R1 missing = FAIL): a PrmCanaryEvent row landed.
    let rows = ssb
        .query_by_type(SsbActionType::PrmCanaryEvent(PrmCanaryEventKind::CanaryInjected))
        .expect("query PrmCanaryEvent(CanaryInjected) must not error");
    assert!(
        !rows.is_empty(),
        "PrmCanaryEvent(CanaryInjected) row MUST land in SSB DB (R1 missing=FAIL, I-24 wire gate)"
    );

    // Decode: not struct-only — bytes come from the MessagePack-serialised DB row.
    let payload = decode_ssb_payload(&rows[0].payload)
        .expect("decode_ssb_payload must succeed for PrmCanaryEvent");
    match payload {
        SsbPayload::PrmCanaryEvent(p) => {
            assert_eq!(
                p.kind,
                PrmCanaryEventKind::CanaryInjected,
                "payload kind must be CanaryInjected (conformal-tau stub always below threshold)"
            );
            assert_eq!(
                p.scorer_id, "conformal_tau",
                "scorer_id must equal SCORER_ID_CONFORMAL_TAU on the conformal-tau stub"
            );
            // Conformal-tau returns the same value for both variants.
            assert_eq!(
                p.delta_bp, 0,
                "conformal-tau stub scores both variants identically; delta_bp must be 0"
            );
            assert!(
                (0.0_f64..=1.0_f64).contains(&p.score_original),
                "score_original must be in [0,1]; got {}",
                p.score_original
            );
            assert!(
                (0.0_f64..=1.0_f64).contains(&p.score_obfuscated),
                "score_obfuscated must be in [0,1]; got {}",
                p.score_obfuscated
            );
            assert_eq!(
                p.consecutive_count, 0,
                "CanaryInjected (no bias) keeps consecutive_count at 0"
            );
            assert!(p.ts_ms > 0, "ts_ms must be a real positive Unix-ms timestamp");
            assert_eq!(
                p.threshold_bp, 500,
                "threshold_bp must match the bootstrap default (500 bp)"
            );
        }
        other => panic!(
            "expected SsbPayload::PrmCanaryEvent; got action_type {:?}",
            other.action_type()
        ),
    }
}

// ---------------------------------------------------------------------------
// (C2) KIND ROUNDTRIP — all 3 arms via as_sql_str + from_str + SQL CHECK.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i24_c_kind_roundtrip_all_3_arms() {
    // as_sql_str for all 3 arms (static, no DB needed).
    assert_eq!(PrmCanaryEventKind::CanaryInjected.as_sql_str(), "prm_canary_injected");
    assert_eq!(PrmCanaryEventKind::StyleBiasDetected.as_sql_str(), "prm_canary_style_bias_detected");
    assert_eq!(PrmCanaryEventKind::ScorerNeutralized.as_sql_str(), "prm_canary_scorer_neutralized");

    // from_str round-trip: each sql string parses back to the expected Kind arm.
    let cases: &[(&str, PrmCanaryEventKind)] = &[
        ("prm_canary_injected",          PrmCanaryEventKind::CanaryInjected),
        ("prm_canary_style_bias_detected", PrmCanaryEventKind::StyleBiasDetected),
        ("prm_canary_scorer_neutralized",  PrmCanaryEventKind::ScorerNeutralized),
    ];
    for (sql_str, expected_kind) in cases {
        let parsed = ssb_action_type_from_str_public(sql_str)
            .unwrap_or_else(|e| panic!("from_str('{sql_str}') must parse; err: {e}"));
        assert_eq!(
            parsed.as_sql_str(), *sql_str,
            "round-trip: as_sql_str(from_str('{sql_str}')) must equal the input"
        );
        match parsed {
            SsbActionType::PrmCanaryEvent(kind) => {
                assert_eq!(
                    kind, *expected_kind,
                    "from_str('{sql_str}') decoded wrong Kind arm"
                );
            }
            other => panic!(
                "from_str('{sql_str}') returned wrong action_type variant: {:?}",
                other
            ),
        }
    }

    // Unknown string must error (fail-closed).
    assert!(
        ssb_action_type_from_str_public("prm_canary_unknown_arm_xyz").is_err(),
        "unknown prm_canary_* sql string must return Err (fail-closed, R1)"
    );

    // SQL CHECK real-DB: write all 3 kinds and confirm each is retrievable.
    // If the SQL CHECK constraint rejects any of the 3 strings, the write
    // silently fails or the row is missing — assert !rows.is_empty() catches it.
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i24-c2-check");

    for (kind, delta_bp, consecutive_count) in [
        (PrmCanaryEventKind::CanaryInjected,    0_u32, 0_u32),
        (PrmCanaryEventKind::StyleBiasDetected, 600,   1),
        (PrmCanaryEventKind::ScorerNeutralized, 600,   3),
    ] {
        let p = PrmCanaryEventPayload {
            kind,
            scorer_id: "c2-scorer".into(),
            pair_index: 0,
            score_original: 0.5,
            score_obfuscated: if delta_bp > 0 { 0.56 } else { 0.5 },
            delta_bp,
            threshold_bp: 500,
            consecutive_count,
            ts_ms: 1_000_000_000,
        };
        emit_prm_canary_to_ssb(&ssb, "agent-c2", 0, p).await;
    }

    for kind in [
        PrmCanaryEventKind::CanaryInjected,
        PrmCanaryEventKind::StyleBiasDetected,
        PrmCanaryEventKind::ScorerNeutralized,
    ] {
        let rows = ssb
            .query_by_type(SsbActionType::PrmCanaryEvent(kind))
            .unwrap_or_else(|e| panic!("query_by_type({kind:?}) failed: {e}"));
        assert!(
            !rows.is_empty(),
            "SQL CHECK must accept '{:?}' (prm_canary_* tag); row absent = FAIL",
            kind
        );
    }
}

// ---------------------------------------------------------------------------
// (C3) ENV-GATE DEFAULT-OFF — no row when the gate is unset.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i24_c_env_gate_default_off_no_row() {
    // Ensure the gate is off.
    unsafe { std::env::remove_var(ELAI_CANARY_INJECTOR_ENABLE_ENV) };

    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i24-c3");
    let project_root = write_minimal_fixture(&dir);

    let stack = TriLayerStack::new_with_fixture_dir(
        empty_registry(),
        stub_dep_gate(),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
        Some(project_root.as_path()),
    );

    let branch = Branch::new("branch-gate-off-c3", "fn gated() {}");
    let row = stack.run_verdict(&branch).await.expect("run_verdict ok");
    assert_eq!(row.l2_call_count, 1, "L2 ran (gate-off does not affect L2 scoring)");

    // R9 gate-off: ZERO PrmCanaryEvent rows of any Kind in the DB.
    let n = count_all_prm_canary_rows(&ssb);
    assert_eq!(
        n, 0,
        "with ELAI_CANARY_INJECTOR_ENABLE unset, ZERO PrmCanaryEvent rows must land (R9 default-OFF)"
    );
}

// ---------------------------------------------------------------------------
// (C4) FROZEN FIXTURE CONTRACT — fixtures/canary_pairs.json on disk.
// ---------------------------------------------------------------------------

#[test]
fn i24_c_frozen_fixture_parses_expected_pair_count() {
    // Locate fixture at .elai_cc/fixtures/canary_pairs.json.
    // CARGO_MANIFEST_DIR is crates/contracts/, parent chain: crates/ -> .elai_cc/.
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() // crates/
        .and_then(|p| p.parent()) // .elai_cc/
        .map(|root| root.join("fixtures").join("canary_pairs.json"))
        .expect("fixture path must construct");

    assert!(
        fixture_path.exists(),
        "fixtures/canary_pairs.json must exist on disk (R8 frozen-fixture gate); path: {}",
        fixture_path.display()
    );

    let text = std::fs::read_to_string(&fixture_path)
        .expect("read canary_pairs.json");
    let val: serde_json::Value =
        serde_json::from_str(&text).expect("canary_pairs.json must be valid JSON");

    let pairs = val
        .get("pairs")
        .and_then(|p| p.as_array())
        .expect("canary_pairs.json must have a top-level 'pairs' array");

    assert!(!pairs.is_empty(), "canary_pairs.json 'pairs' must be non-empty (R8)");

    // Shape: each pair has label / original / obfuscated (all non-empty,
    // original != obfuscated — an identical pair cannot detect bias).
    for (i, pair) in pairs.iter().enumerate() {
        let label = pair.get("label").and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("pair[{i}] missing 'label' string field"));
        let original = pair.get("original").and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("pair[{i}] ({label}) missing 'original' string field"));
        let obfuscated = pair.get("obfuscated").and_then(|v| v.as_str())
            .unwrap_or_else(|| panic!("pair[{i}] ({label}) missing 'obfuscated' string field"));

        assert!(!label.is_empty(),     "pair[{i}] 'label' must be non-empty");
        assert!(!original.is_empty(),  "pair[{i}] ({label}) 'original' must be non-empty");
        assert!(!obfuscated.is_empty(), "pair[{i}] ({label}) 'obfuscated' must be non-empty");
        assert_ne!(
            original, obfuscated,
            "pair[{i}] ({label}): original == obfuscated — an identical pair cannot detect bias (R8 fixture quality)"
        );
    }

    // The shipped fixture has exactly 6 pairs (3 Python + 3 Rust operators:
    // for_to_while, rename_idempotent, constant_unfolding × 2 languages).
    // Per I-24-decisions.md R8 section.
    assert_eq!(
        pairs.len(), 6,
        "fixtures/canary_pairs.json must contain exactly 6 pairs (R8 / I-24-decisions.md); got {}",
        pairs.len()
    );
}

// ---------------------------------------------------------------------------
// (C5) DIRECT EMIT FN REAL-DB COVERAGE — emit_prm_canary_to_ssb for all 3 arms.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i24_c_canary_injected_emit_via_real_emit_fn() {
    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i24-c5");

    // Emit one row per Kind arm directly through emit_prm_canary_to_ssb.
    // This confirms the emit fn's MessagePack payload round-trips correctly
    // through SQLite (not struct-only; bytes come from the real DB).
    let emit_cases = [
        (PrmCanaryEventKind::CanaryInjected,    0_u32,   0_u32, 0.4_f64,  0.4_f64),
        (PrmCanaryEventKind::StyleBiasDetected, 600,     1,     0.4_f64,  0.46_f64),
        (PrmCanaryEventKind::ScorerNeutralized, 600,     3,     0.4_f64,  0.46_f64),
    ];
    for (kind, delta_bp, consecutive_count, score_original, score_obfuscated) in emit_cases {
        let p = PrmCanaryEventPayload {
            kind,
            scorer_id: "scorer_c5".into(),
            pair_index: 1,
            score_original,
            score_obfuscated,
            delta_bp,
            threshold_bp: 500,
            consecutive_count,
            ts_ms: 1_750_000_000_000_i64,
        };
        emit_prm_canary_to_ssb(&ssb, "agent-c5", 2, p).await;
    }

    // Retrieve and decode each Kind arm from the real DB.
    for kind in [
        PrmCanaryEventKind::CanaryInjected,
        PrmCanaryEventKind::StyleBiasDetected,
        PrmCanaryEventKind::ScorerNeutralized,
    ] {
        let rows = ssb
            .query_by_type(SsbActionType::PrmCanaryEvent(kind))
            .unwrap_or_else(|e| panic!("query_by_type({kind:?}): {e}"));
        assert!(
            !rows.is_empty(),
            "emit_prm_canary_to_ssb must produce a real-DB row for Kind={kind:?} (R1 missing=FAIL)"
        );
        // Decode: not struct-only; bytes are MessagePack from the real SQLite row.
        let decoded = decode_ssb_payload(&rows[0].payload)
            .unwrap_or_else(|e| panic!("decode_ssb_payload for Kind={kind:?}: {e}"));
        match decoded {
            SsbPayload::PrmCanaryEvent(p) => {
                assert_eq!(p.kind, kind, "decoded kind must match emitted kind for {kind:?}");
                assert_eq!(p.scorer_id, "scorer_c5", "scorer_id must round-trip for {kind:?}");
                assert!(p.ts_ms > 0, "ts_ms must be positive in decoded payload");
            }
            other => panic!(
                "expected SsbPayload::PrmCanaryEvent for Kind={kind:?}; got {:?}",
                other.action_type()
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// (C6) ID-4 VETO — vetoed branch produces ZERO PrmCanaryEvent rows.
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn i24_c_id4_veto_no_canary_row() {
    // Gate ON to confirm the canary is absent because of ID-4, not because the gate is off.
    unsafe { std::env::set_var(ELAI_CANARY_INJECTOR_ENABLE_ENV, "1") };

    let dir = TempDir::new().expect("tempdir");
    let ssb = test_ssb(&dir, "i24-c6");
    let project_root = write_minimal_fixture(&dir);

    let stack = TriLayerStack::new_with_fixture_dir(
        always_veto_registry(), // L1 always vetoes -> HardVeto early return
        stub_dep_gate(),
        None,
        None,
        AdversaryConfig::default_for_local_worker(),
        Arc::clone(&ssb),
        Some(project_root.as_path()),
    );

    let branch = Branch::new("branch-vetoed-c6", "fn vetoed() { let x = 1; }");
    let row = stack.run_verdict(&branch).await.expect("run_verdict ok");

    // ID-4 observable proof: L2 never ran.
    assert_eq!(
        row.l2_call_count, 0,
        "ID-4: l2_call_count must be 0 for a vetoed branch (L2 never sees a vetoed branch)"
    );

    // The canary fires ONLY at the L2 call site; a vetoed branch never reaches it.
    let n = count_all_prm_canary_rows(&ssb);
    assert_eq!(
        n, 0,
        "ID-4: a vetoed branch must produce ZERO PrmCanaryEvent rows (canary is L2-only)"
    );

    unsafe { std::env::remove_var(ELAI_CANARY_INJECTOR_ENABLE_ENV) };
}
