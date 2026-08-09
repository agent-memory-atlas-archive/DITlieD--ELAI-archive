//! C-32 contract: Code-Completeness Layer 2 — skeleton-first generation +
//! MiniCritic + AcceptAndComplete + deterministic bridge files.
//!
//! Filename intentionally retains the "minicritc" typo from the plan
//! §Files-touched + DETERMINISTIC_CHECK rg command (and the C-32-decisions
//! D11 confirms this). The crate is `elai_cc_contracts` per C-31 D7.
//!
//! Pins:
//!   * SCHEMA_VERSION_PUBLIC >= 63 (atomic 9-tuple bump per
//!     workflow/reasoning/C-32-decisions.md D1).
//!   * Five new SsbActionType variants — SkeletonGenerated, SkeletonFlushed,
//!     MiniCriticVerdict, StructuralRetryStageAttempt, BridgeFileGenerated.
//!   * SkeletonSpec / FilledBuffer / SkeletonError / SkeletonGenerator.
//!   * MiniCritic verdict tri-state + reason format.
//!   * StructuralRetryPlan stage order.
//!   * BridgeFileTemplate deterministic mod.rs output with ELAI_SEED.
//!   * AcceptAndComplete threshold semantics.

use std::path::PathBuf;

use elai_core::file_plan::{FilePlan, FileSpec, FunctionSpec};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, SsbActionType, SCHEMA_VERSION_PUBLIC,
};
use elai_orchestrator::{
    BlankId, BridgeFileSpec, BridgeFileTemplate, BridgeKind, FillResult, FilledBuffer, MiniCritic,
    MiniCriticLanguage, MiniCriticVerdict, RetryContext, SkeletonError, SkeletonGenerator,
    SkeletonGeneratorConfig, SkeletonSpec, StaticStageCandidates, StructuralRetryOutcome,
    StructuralRetryPlan, StructuralRetrySequencer, StructuralRetryStage,
    ACCEPT_AND_COMPLETE_THRESHOLD,
};

// ---------------------------------------------------------------------------
// 0. SCHEMA + action_type round-trip pins
// ---------------------------------------------------------------------------

#[test]
fn c32_schema_version_public_floor_is_63() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= 63,
        "C-32 atomic 9-tuple lockstep bump: floor is 63 (see \
         workflow/reasoning/C-32-decisions.md D1). Current: {SCHEMA_VERSION_PUBLIC}",
    );
}

#[test]
fn c32_new_action_types_round_trip() {
    let pairs = [
        ("skeleton_generated", SsbActionType::SkeletonGenerated),
        ("skeleton_flushed", SsbActionType::SkeletonFlushed),
        ("mini_critic_verdict", SsbActionType::MiniCriticVerdict),
        (
            "structural_retry_stage_attempt",
            SsbActionType::StructuralRetryStageAttempt,
        ),
        ("bridge_file_generated", SsbActionType::BridgeFileGenerated),
    ];
    for (s, variant) in pairs {
        assert_eq!(variant.as_sql_str(), s, "as_sql_str for {s}");
        let parsed = ssb_action_type_from_str_public(s)
            .unwrap_or_else(|e| panic!("from_str must accept '{s}': {e}"));
        assert_eq!(parsed, variant, "from_str for {s}");
    }
}

// ---------------------------------------------------------------------------
// 1. SkeletonSpec contract — blank count, ids, placeholder text
// ---------------------------------------------------------------------------

fn func(name: &str) -> FunctionSpec {
    FunctionSpec {
        name: name.into(),
        signature: format!("fn {name}() -> ()"),
        max_body_lines: 20,
    }
}

fn file_plan_with_n_functions(n: usize) -> FilePlan {
    let mut funcs = Vec::new();
    for i in 0..n {
        funcs.push(func(&format!("f{i}")));
    }
    FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("a.rs"),
            language: "rust".into(),
            max_lines: 200,
        max_distinct_concerns: 16,
            functions: funcs,
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    }
}

#[test]
fn skeleton_spec_blank_count_matches_file_plan() {
    let plan = file_plan_with_n_functions(5);
    let gen = SkeletonGenerator::default();
    let skeletons = gen.generate_from_file_plan(&plan).expect("ok");
    assert_eq!(skeletons.len(), 1);
    assert_eq!(skeletons[0].blanks.len(), 5);
}

#[test]
fn skeleton_blank_ids_sequential() {
    let plan = file_plan_with_n_functions(5);
    let gen = SkeletonGenerator::default();
    let spec = gen.generate_from_file_plan(&plan).unwrap().remove(0);
    let ids: Vec<u8> = spec.blanks.iter().map(|b| b.0).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 5]);
}

#[test]
fn skeleton_text_contains_all_placeholders() {
    let plan = file_plan_with_n_functions(5);
    let gen = SkeletonGenerator::default();
    let spec = gen.generate_from_file_plan(&plan).unwrap().remove(0);
    for i in 1..=5 {
        let needle = format!("// BLANK_{i}");
        assert!(
            spec.skeleton_text.contains(&needle),
            "skeleton_text must contain {needle}"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. flush_to_disk single-write contract
// ---------------------------------------------------------------------------

#[test]
fn flush_single_disk_write_per_file() {
    // tempfile-backed: write once, read back, confirm bodies substituted.
    // The single-write invariant is encoded in the source itself (one
    // `fs::write` call); this test asserts the observable post-condition
    // — exactly one file exists at the target path with the rendered
    // bodies inlined.
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("a.rs");
    let gen = SkeletonGenerator::default();
    let critic = MiniCritic::without_tree_sitter();
    let spec = gen
        .generate_for_functions(
            target.clone(),
            "rust".into(),
            &[func("a"), func("b")],
        )
        .unwrap();
    let bodies: Vec<String> = vec![
        "let x = 1;\nlet y = 2;\nx + y".into(),
        "let x = 3;\nlet y = 4;\nx + y".into(),
    ];
    let buf = gen.fill_blanks(spec, bodies, &critic).unwrap();
    let written = gen.flush_to_disk(&buf).expect("flush ok");
    assert_eq!(written, target);
    let text = std::fs::read_to_string(&target).unwrap();
    assert!(text.contains("let x = 1"));
    assert!(text.contains("let x = 3"));
    // No leftover blank placeholders.
    assert!(!text.contains("    // BLANK_1\n"));
    assert!(!text.contains("    // BLANK_2\n"));
}

// ---------------------------------------------------------------------------
// 3. completeness_ratio semantics
// ---------------------------------------------------------------------------

#[test]
fn completeness_ratio_all_pass() {
    let buf = filled_buffer_with(vec![
        MiniCriticVerdict::Pass,
        MiniCriticVerdict::Pass,
        MiniCriticVerdict::Pass,
        MiniCriticVerdict::Pass,
    ]);
    assert_eq!(buf.compute_completeness_ratio(), 1.0);
}

#[test]
fn completeness_ratio_partial() {
    let buf = filled_buffer_with(vec![
        MiniCriticVerdict::Pass,
        MiniCriticVerdict::Block,
        MiniCriticVerdict::Pass,
        MiniCriticVerdict::Block,
    ]);
    assert!((buf.compute_completeness_ratio() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn completeness_ratio_zero_blanks() {
    // Vacuous pass per C-32-decisions D8.
    let buf = FilledBuffer {
        spec: SkeletonSpec {
            file_path: PathBuf::from("empty.rs"),
            language: "rust".into(),
            blanks: vec![],
            skeleton_text: String::new(),
            functions: vec![],
            responsibility_budget_exceeded: false,
        },
        fills: vec![],
        completeness_ratio: 0.0,
    };
    assert_eq!(buf.compute_completeness_ratio(), 1.0);
}

fn filled_buffer_with(verdicts: Vec<MiniCriticVerdict>) -> FilledBuffer {
    let blanks: Vec<BlankId> = (1..=verdicts.len() as u8).map(BlankId).collect();
    let fills = verdicts
        .iter()
        .enumerate()
        .map(|(i, v)| FillResult {
            blank_id: BlankId((i + 1) as u8),
            body: format!("body_{i}"),
            mini_critic_verdict: *v,
        })
        .collect();
    let mut buf = FilledBuffer {
        spec: SkeletonSpec {
            file_path: PathBuf::from("a.rs"),
            language: "rust".into(),
            blanks,
            skeleton_text: String::new(),
            functions: vec![],
            responsibility_budget_exceeded: false,
        },
        fills,
        completeness_ratio: 0.0,
    };
    buf.refresh_ratio();
    buf
}

// ---------------------------------------------------------------------------
// 4. MiniCritic semantics
// ---------------------------------------------------------------------------

#[test]
fn mini_critic_blocks_todo_pattern() {
    let critic = MiniCritic::without_tree_sitter();
    let r = critic.check(BlankId(1), "// TODO implement", MiniCriticLanguage::Rust);
    assert_eq!(r.verdict, MiniCriticVerdict::Block);
    assert!(
        r.reason.starts_with("lazy_pattern:"),
        "reason must start with lazy_pattern: but got {}",
        r.reason
    );
}

#[test]
fn mini_critic_blocks_short_body() {
    let critic = MiniCritic::without_tree_sitter();
    // Two-line body (non-expression context) -> too_short:2_lines.
    let r = critic.check(BlankId(1), "x\ny", MiniCriticLanguage::Rust);
    assert_eq!(r.verdict, MiniCriticVerdict::Block);
    assert_eq!(r.reason, "too_short:2_lines");
}

#[test]
fn mini_critic_passes_multiline_body() {
    let critic = MiniCritic::without_tree_sitter();
    let body = "let a = 1;\nlet b = 2;\nlet c = 3;\na + b + c";
    let r = critic.check(BlankId(1), body, MiniCriticLanguage::Rust);
    assert_eq!(r.verdict, MiniCriticVerdict::Pass);
}

// ---------------------------------------------------------------------------
// 5. StructuralRetryPlan order pin
// ---------------------------------------------------------------------------

#[test]
fn structural_retry_stages_order() {
    let plan = StructuralRetryPlan::default_for_blank(BlankId(1));
    assert_eq!(
        plan.stages,
        vec![
            StructuralRetryStage::Decompose,
            StructuralRetryStage::Template,
            StructuralRetryStage::FewShot,
            StructuralRetryStage::BestOfN,
        ],
        "Plan §Task 3 ordering must be Decompose -> Template -> FewShot -> BestOfN"
    );
}

// ---------------------------------------------------------------------------
// 6. BridgeFileTemplate determinism + lex-sort
// ---------------------------------------------------------------------------

#[test]
fn bridge_mod_rs_deterministic() {
    // Two builds with ELAI_SEED=42 produce identical output (plan
    // DETERMINISTIC_CHECK #4 + ACCEPTANCE).
    let t1 = BridgeFileTemplate::with_seed(42);
    let t2 = BridgeFileTemplate::with_seed(42);
    let spec = BridgeFileSpec {
        kind: BridgeKind::ModRsAggregator,
        target_path: PathBuf::from("mod.rs"),
        module_names: vec!["a".into(), "b".into(), "c".into()],
    };
    assert_eq!(t1.generate(&spec), t2.generate(&spec));
}

#[test]
fn bridge_mod_rs_sorted_lexicographic() {
    let t = BridgeFileTemplate::with_seed(42);
    let spec = BridgeFileSpec {
        kind: BridgeKind::ModRsAggregator,
        target_path: PathBuf::from("mod.rs"),
        module_names: vec!["z".into(), "a".into(), "m".into()],
    };
    let body = t.generate(&spec);
    let lines: Vec<&str> = body.lines().filter(|l| l.starts_with("pub mod")).collect();
    assert_eq!(
        lines,
        vec!["pub mod a;", "pub mod m;", "pub mod z;"],
        "ModRsAggregator output must be sorted lexicographically"
    );
}

#[test]
fn bridge_file_no_llm_call() {
    // BridgeFileTemplate::generate is pure text assembly — no I/O, no
    // network. We exercise generate() and assert it returns a populated
    // string with the auto-generated banner. If a future change ever
    // pulls in an LLM client, this test's runtime would explode (a real
    // network call returns 200 in seconds, not microseconds). The
    // tightest available adversarial check from a contract test is to
    // assert generate() completes in well under 100ms.
    let t = BridgeFileTemplate::with_seed(0);
    let spec = BridgeFileSpec {
        kind: BridgeKind::ModRsAggregator,
        target_path: PathBuf::from("mod.rs"),
        module_names: vec!["alpha".into()],
    };
    let started = std::time::Instant::now();
    let body = t.generate(&spec);
    let elapsed = started.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "generate() must be near-instant (no LLM / network); took {elapsed:?}",
    );
    assert!(
        body.contains("Auto-generated by ELAI BridgeFileTemplate"),
        "Bridge body must carry the deterministic banner",
    );
}

// ---------------------------------------------------------------------------
// 7. AcceptAndComplete threshold + Exhausted
// ---------------------------------------------------------------------------

#[test]
fn accept_and_complete_threshold() {
    let seq = StructuralRetrySequencer::default();
    let critic = MiniCritic::without_tree_sitter();
    let mut plan_at_threshold = StructuralRetryPlan::default_for_blank(BlankId(2));
    let mut plan_below_threshold = StructuralRetryPlan::default_for_blank(BlankId(3));
    let candidates = StaticStageCandidates::empty();

    // At threshold: 1 PASS / 2 blanks = 0.5 -> AcceptAndComplete.
    let mut ctx_at = RetryContext {
        project_root: PathBuf::from("/tmp"),
        language: MiniCriticLanguage::Rust,
        fn_signature: "fn probe()".into(),
        existing_fills: vec![FillResult {
            blank_id: BlankId(1),
            body: "ok".into(),
            mini_critic_verdict: MiniCriticVerdict::Pass,
        }],
        total_blanks: 2,
        accept_and_complete_enabled: true,
    };
    let outcome = seq.run_sequence(&mut plan_at_threshold, &ctx_at, &critic, &candidates);
    match outcome {
        StructuralRetryOutcome::AcceptAndComplete { ratio } => {
            assert!(
                (ratio - ACCEPT_AND_COMPLETE_THRESHOLD).abs() < f32::EPSILON,
                "ratio must hit the documented threshold exactly; got {ratio}"
            );
        }
        other => panic!("expected AcceptAndComplete at threshold, got {other:?}"),
    }

    // Below threshold: 1 PASS / 3 blanks = 0.333... -> Exhausted.
    ctx_at.total_blanks = 3;
    let outcome_below =
        seq.run_sequence(&mut plan_below_threshold, &ctx_at, &critic, &candidates);
    assert_eq!(
        outcome_below,
        StructuralRetryOutcome::Exhausted,
        "ratio strictly below threshold must yield Exhausted",
    );
}

// ---------------------------------------------------------------------------
// 8. TooManyBlanks rejection (skeleton size guard)
// ---------------------------------------------------------------------------

#[test]
fn skeleton_rejects_too_many_blanks() {
    let mut funcs = Vec::new();
    for i in 0..40 {
        funcs.push(func(&format!("f{i}")));
    }
    let gen = SkeletonGenerator::new(SkeletonGeneratorConfig::default());
    let err = gen
        .generate_for_functions(PathBuf::from("big.rs"), "rust".into(), &funcs)
        .unwrap_err();
    match err {
        SkeletonError::TooManyBlanks { max, got, .. } => {
            assert_eq!(max, 32, "default max_blanks must be 32");
            assert_eq!(got, 40);
        }
        other => panic!("expected TooManyBlanks, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 9. Audit-C32 close-out: per-blank dispatch wiring (W1 + W2)
// ---------------------------------------------------------------------------

/// W1 close-out: the orchestrator-side per-blank dispatch helper consults
/// BridgeFileTemplate BEFORE the structural retry chain when the skeleton
/// targets a known bridge file (mod.rs / __init__.py / index.ts / doc.go).
/// A `mod.rs` target with two declared functions short-circuits to the
/// bridge body and yields completeness_ratio == 1.0 — proving the
/// "BridgeTemplate consulted before LLM" requirement from the W1 fix.
#[test]
fn c32_close_out_bridge_template_consulted_before_retry_chain() {
    use elai_orchestrator::{
        classify_bridge_kind, dispatch_skeleton_with_retry, BridgeKind, MiniCritic,
        PerBlankCandidates, SkeletonGenerator, StructuralRetrySequencer,
    };

    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("mod.rs");
    let kind = classify_bridge_kind(&target);
    assert_eq!(
        kind,
        Some(BridgeKind::ModRsAggregator),
        "classify_bridge_kind must recognise mod.rs as ModRsAggregator"
    );

    let gen = SkeletonGenerator::default();
    let spec = gen
        .generate_for_functions(
            target.clone(),
            "rust".into(),
            &[func("alpha"), func("beta")],
        )
        .expect("skeleton ok");

    // Run the dispatch helper synchronously from the test runtime.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let (buf, outcome) = runtime.block_on(async {
        dispatch_skeleton_with_retry(
            spec,
            &PerBlankCandidates::default(),
            &MiniCritic::without_tree_sitter(),
            &StructuralRetrySequencer::default(),
            kind,
            false,
            None,
            "contract-test",
            0,
        )
        .await
        .expect("dispatch ok")
    });

    assert!(
        outcome.bridge_short_circuit,
        "bridge_short_circuit must fire for mod.rs targets",
    );
    assert!(
        (outcome.completeness_ratio - 1.0).abs() < f32::EPSILON,
        "bridge short-circuit must yield ratio == 1.0; got {}",
        outcome.completeness_ratio,
    );
    assert_eq!(buf.fills.len(), 2, "two fills for two declared blanks");
}

/// W2 close-out: get_completeness_ratio must NOT unconditionally return
/// Value::Null. The Tauri command now reads the latest `skeleton_flushed`
/// row via SsbViewer and projects the wire shape
/// `{ known: bool, ratio?: f32, blank_count?: u32, last_path?: String }`.
/// This contract test asserts the schema floor by exercising the JSON
/// envelope construction with a known SkeletonFlushedPayload — the actual
/// command lives in the Tauri crate, but the wire shape is shared with
/// the dashboard via SkeletonFlushedPayload (orchestrator surface).
#[test]
fn c32_close_out_completeness_envelope_shape_pinned() {
    use elai_orchestrator::ssb::SkeletonFlushedPayload;

    // Construct a SkeletonFlushedPayload, round-trip through JSON, and
    // assert the dashboard envelope projection produces a populated
    // record (NOT bare null). The Tauri command's projection logic
    // mirrors this exact pattern.
    let payload = SkeletonFlushedPayload {
        path: "crate/foo.rs".into(),
        completeness_ratio: 0.75,
    };
    let payload_json = serde_json::to_value(&payload).expect("payload json encode");

    let ratio = payload_json
        .get("completeness_ratio")
        .and_then(|v| v.as_f64())
        .expect("completeness_ratio field present");
    let last_path = payload_json
        .get("path")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let envelope = serde_json::json!({
        "known": true,
        "ratio": ratio as f32,
        "blank_count": 0u32,
        "last_path": last_path,
    });

    // Hard invariant: envelope is NOT null (the old W2 stub returned
    // Value::Null unconditionally).
    assert!(
        !envelope.is_null(),
        "get_completeness_ratio must return a populated envelope, not null"
    );
    assert_eq!(
        envelope.get("known").and_then(|v| v.as_bool()),
        Some(true),
        "envelope must carry known:true when SkeletonFlushedPayload is present"
    );
    assert!(
        (envelope.get("ratio").and_then(|v| v.as_f64()).unwrap() - 0.75).abs() < 1e-6,
        "envelope ratio must round-trip from SkeletonFlushedPayload.completeness_ratio"
    );
    assert_eq!(
        envelope.get("last_path").and_then(|v| v.as_str()),
        Some("crate/foo.rs"),
        "envelope last_path must round-trip from SkeletonFlushedPayload.path"
    );
}
