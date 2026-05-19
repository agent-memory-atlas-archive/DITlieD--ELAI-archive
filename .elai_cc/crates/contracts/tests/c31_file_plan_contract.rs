//! C-31 — Layer 1 FilePlan / FileLineTracker / MistakeRecorder contract tests.
//!
//! Pins the C-31 surfaces so the design stays executable:
//!   - FilePlan / FileSpec / FileModSpec / FunctionSpec construction + serde
//!   - FilePlan::validate (line budget + function count caps)
//!   - BabyStepSpec.file_plan #[serde(default)] backward-compat
//!   - TreeNavigatorState.cumulative_lines_by_step + TOCTOU step_write_generation
//!   - T2 Check 3 function-count guard with slack
//!   - MistakeRecorder injection into T1Hook / T2Hook + BLOCK event recording
//!   - FileDisciplineConfig.language_limit_overrides
//!
//! All tests fully deterministic — no clocks, no I/O.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use elai_core::file_plan::{
    FileModSpec, FilePlan, FilePlanError, FileSpec, FunctionSpec, LanguageLimitsMap,
    MAX_FUNCTIONS_PER_FILE,
};
use elai_core::mistake_book::FailureCategory;
use elai_orchestrator::pipeline_tree_navigator::{NavigatorError, TreeNavigatorState};
use elai_orchestrator::research_tree::{BabyStepGrounding, BabyStepId, BabyStepSpec};
use elai_plugins::tiered_analysis::{
    FileDisciplineConfig, HookVerdict, LanguageLimits, MistakeRecorder, T1Hook, T2Hook,
    FUNCTION_COUNT_SLACK, MAX_FUNCTIONS_PER_FILE as PLUGIN_MAX_FUNCTIONS_PER_FILE,
};
use elai_rules::rule_engine::{DefaultRuleEngine, FsmPhaseBit};
use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

fn rust_only_limits() -> LanguageLimitsMap {
    let mut m = HashMap::new();
    m.insert("rust".to_string(), 400u16);
    m.insert("python".to_string(), 480u16);
    m
}

/// Captures every MistakeRecorder event for assertion.
#[derive(Default, Clone)]
struct RecordingMistakeRecorder {
    events: Arc<Mutex<Vec<(String, FailureCategory, PathBuf, String)>>>,
}

impl MistakeRecorder for RecordingMistakeRecorder {
    fn record_mistake_event(
        &self,
        baby_step_id: &str,
        category: FailureCategory,
        file_path: &Path,
        snippet: &str,
    ) {
        self.events.lock().expect("mistake recorder mutex").push((
            baby_step_id.to_string(),
            category,
            file_path.to_path_buf(),
            snippet.to_string(),
        ));
    }
}

impl RecordingMistakeRecorder {
    fn snapshot(&self) -> Vec<(String, FailureCategory, PathBuf, String)> {
        self.events.lock().expect("mistake recorder mutex").clone()
    }
}

fn empty_t1() -> T1Hook {
    let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
    T1Hook::new(Box::new(engine), FsmPhaseBit::EXECUTING)
}

// ---------------------------------------------------------------------------
// 1. Core type construction
// ---------------------------------------------------------------------------

#[test]
fn file_plan_types_compile() {
    let fn_spec = FunctionSpec {
        name: "compute".into(),
        signature: "fn compute(x: u32) -> u32".into(),
        max_body_lines: 30,
    };
    let file_spec = FileSpec {
        path: PathBuf::from("src/lib.rs"),
        language: "rust".into(),
        max_lines: 200,
        functions: vec![fn_spec.clone()],
    };
    let mod_spec = FileModSpec {
        path: PathBuf::from("src/mod.rs"),
        language: "rust".into(),
        max_added_lines: 50,
        functions: vec![],
    };
    let plan = FilePlan {
        new_files: vec![file_spec.clone()],
        modified_files: vec![mod_spec.clone()],
        version: FilePlan::CURRENT_VERSION,
    };
    assert_eq!(plan.new_files.len(), 1);
    assert_eq!(plan.modified_files.len(), 1);
    assert_eq!(plan.new_files[0].functions[0].name, "compute");
}

// ---------------------------------------------------------------------------
// 2. JSON round-trip
// ---------------------------------------------------------------------------

#[test]
fn file_plan_json_round_trip() {
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/repo.rs"),
            language: "rust".into(),
            max_lines: 150,
            functions: vec![FunctionSpec {
                name: "save".into(),
                signature: "fn save(&self) -> Result<()>".into(),
                max_body_lines: 25,
            }],
        }],
        modified_files: vec![FileModSpec {
            path: PathBuf::from("src/api.rs"),
            language: "rust".into(),
            max_added_lines: 40,
            functions: vec![],
        }],
        version: FilePlan::CURRENT_VERSION,
    };
    let json = serde_json::to_string(&plan).expect("serialize");
    let back: FilePlan = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(plan, back, "JSON round-trip must be lossless");
}

// ---------------------------------------------------------------------------
// 3-5. validate paths
// ---------------------------------------------------------------------------

#[test]
fn validate_accepts_within_limits() {
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("a.rs"),
            language: "rust".into(),
            max_lines: 100,
            functions: vec![],
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    };
    assert!(plan.validate(&rust_only_limits()).is_ok());
}

#[test]
fn validate_rejects_over_limit() {
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("big.rs"),
            language: "rust".into(),
            max_lines: 500,
            functions: vec![],
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    };
    let err = plan.validate(&rust_only_limits()).unwrap_err();
    assert!(
        matches!(err, FilePlanError::LineBudgetExceeded { .. }),
        "expected LineBudgetExceeded, got {err:?}"
    );
}

#[test]
fn validate_rejects_over_function_count() {
    let mut funcs = Vec::new();
    for i in 0..(MAX_FUNCTIONS_PER_FILE as usize + 5) {
        funcs.push(FunctionSpec {
            name: format!("f{i}"),
            signature: "fn() -> ()".into(),
            max_body_lines: 5,
        });
    }
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("crowded.rs"),
            language: "rust".into(),
            max_lines: 100,
            functions: funcs,
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    };
    let err = plan.validate(&rust_only_limits()).unwrap_err();
    assert!(
        matches!(err, FilePlanError::FunctionCountExceeded { .. }),
        "expected FunctionCountExceeded, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// 6-8. BabyStepSpec serde
// ---------------------------------------------------------------------------

fn dummy_baby_step(file_plan: Option<FilePlan>) -> BabyStepSpec {
    BabyStepSpec {
        id: BabyStepId(1),
        task: "C-31 contract".into(),
        estimated_tokens: 100,
        grounding: BabyStepGrounding::default(),
        tdd: None,
        acceptance: None,
        preferred_mode: None,
        target_file: None,
        edit_region: None,
        intent_pairs: elai_orchestrator::IntentPairSet::default(),
        file_plan,
    }
}

#[test]
fn baby_step_spec_file_plan_field_present() {
    let spec = dummy_baby_step(None);
    let json = serde_json::to_string(&spec).expect("serialize");
    let back: BabyStepSpec = serde_json::from_str(&json).expect("deserialize");
    assert!(back.file_plan.is_none());
}

#[test]
fn baby_step_spec_file_plan_populated() {
    let plan = FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/main.rs"),
            language: "rust".into(),
            max_lines: 200,
            functions: vec![],
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    };
    let spec = dummy_baby_step(Some(plan.clone()));
    let json = serde_json::to_string(&spec).expect("serialize");
    let back: BabyStepSpec = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.file_plan, Some(plan));
}

#[test]
fn baby_step_spec_legacy_deserializes() {
    // Legacy JSON without the file_plan key MUST still parse with
    // file_plan == None (serde(default) coverage).
    let legacy = r#"{
        "id": 7,
        "task": "edit legacy",
        "estimated_tokens": 100,
        "grounding": {"concept_name":"","phase_name":"","step_name":"","top_citations":[],"compressed_tokens":0},
        "tdd": null,
        "acceptance": null
    }"#;
    let parsed: BabyStepSpec =
        serde_json::from_str(legacy).expect("deserialize legacy baby step");
    assert!(parsed.file_plan.is_none(), "missing field must default to None");
}

// ---------------------------------------------------------------------------
// 9-10. FileLineTracker
// ---------------------------------------------------------------------------

#[test]
fn file_line_tracker_record_and_total() {
    let mut state = TreeNavigatorState::new();
    let gen0 = state.begin_line_write();
    state
        .record_lines_written(BabyStepId(1), 50, gen0)
        .expect("record 1");

    let gen1 = state.begin_line_write();
    state
        .record_lines_written(BabyStepId(2), 30, gen1)
        .expect("record 2");

    let gen2 = state.begin_line_write();
    state
        .record_lines_written(BabyStepId(3), 20, gen2)
        .expect("record 3");

    assert_eq!(state.total_lines_written(), 100);
    assert_eq!(state.cumulative_lines_by_step.len(), 3);
}

#[test]
fn file_line_tracker_toctou_guard() {
    let mut state = TreeNavigatorState::new();
    let stale_gen = state.begin_line_write();
    // Simulate a concurrent successful write between begin and commit.
    state
        .record_lines_written(BabyStepId(1), 10, stale_gen)
        .expect("first write should succeed");
    // Same captured token now stale — second commit must reject.
    let err = state
        .record_lines_written(BabyStepId(2), 20, stale_gen)
        .unwrap_err();
    assert!(matches!(err, NavigatorError::WriteGeneration { .. }));
}

#[test]
fn file_line_tracker_state_round_trips() {
    let mut state = TreeNavigatorState::new();
    let g0 = state.begin_line_write();
    state.record_lines_written(BabyStepId(1), 50, g0).unwrap();

    let json = serde_json::to_string(&state).expect("serialize state");
    let back: TreeNavigatorState =
        serde_json::from_str(&json).expect("deserialize state");
    assert_eq!(back.total_lines_written(), 50);
    assert_eq!(back.step_write_generation, state.step_write_generation);
}

// ---------------------------------------------------------------------------
// 11-13. T2 Check 3 function-count guard
// ---------------------------------------------------------------------------

/// Seven Rust functions with non-empty bodies (avoids the T2 empty-body
/// WARN that would otherwise mask the Check 3 verdict).
fn rust_seven_functions() -> &'static str {
    r#"
fn a() { let _ = 42; }
fn b() { let _ = 42; }
fn c() { let _ = 42; }
fn d() { let _ = 42; }
fn e() { let _ = 42; }
fn f() { let _ = 42; }
fn g() { let _ = 42; }
"#
}

#[test]
fn t2_check3_fires_on_excess_functions() {
    let mut t2 = T2Hook::new(5000);
    let plan = Arc::new(FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/lib.rs"),
            language: "rust".into(),
            max_lines: 200,
            functions: vec![
                FunctionSpec { name: "a".into(), signature: "fn a()".into(), max_body_lines: 5 },
                FunctionSpec { name: "b".into(), signature: "fn b()".into(), max_body_lines: 5 },
                FunctionSpec { name: "c".into(), signature: "fn c()".into(), max_body_lines: 5 },
            ],
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    });
    t2.set_file_plan(plan);
    // Plan declares 3 functions; source has 7. Limit = 3 + slack(2) = 5. 7 > 5 -> BLOCK.
    let verdict = t2
        .evaluate(rust_seven_functions(), "src/lib.rs")
        .expect("evaluate")
        .expect("verdict");
    assert!(
        verdict.is_block(),
        "function count 7 exceeds plan 3+slack must BLOCK, got {verdict:?}"
    );
    if let HookVerdict::Block { message, .. } = verdict {
        assert!(
            message.contains("function count") && message.contains("FILE_PLAN"),
            "BLOCK message must reference FILE_PLAN, got: {message}"
        );
    }
}

#[test]
fn t2_check3_skipped_when_no_file_plan() {
    let t2 = T2Hook::new(5000);
    // No file_plan installed -> Check 3 silently skipped. Result is Allow.
    let source = "fn a() { let _ = 1; } fn b() { let _ = 1; } fn c() { let _ = 1; } fn d() { let _ = 1; }";
    let verdict = t2.evaluate(source, "src/lib.rs").expect("evaluate").expect("verdict");
    assert_eq!(verdict, HookVerdict::Allow);
}

#[test]
fn t2_check3_slack_of_two_accepted() {
    let mut t2 = T2Hook::new(5000);
    let plan = Arc::new(FilePlan {
        new_files: vec![FileSpec {
            path: PathBuf::from("src/lib.rs"),
            language: "rust".into(),
            max_lines: 200,
            functions: vec![
                FunctionSpec { name: "a".into(), signature: "fn a()".into(), max_body_lines: 5 },
                FunctionSpec { name: "b".into(), signature: "fn b()".into(), max_body_lines: 5 },
                FunctionSpec { name: "c".into(), signature: "fn c()".into(), max_body_lines: 5 },
                FunctionSpec { name: "d".into(), signature: "fn d()".into(), max_body_lines: 5 },
                FunctionSpec { name: "e".into(), signature: "fn e()".into(), max_body_lines: 5 },
            ],
        }],
        modified_files: vec![],
        version: FilePlan::CURRENT_VERSION,
    });
    t2.set_file_plan(plan);
    // Plan declares 5 functions; source has 7 = 5 + slack(2). 7 > 7 is false ->
    // Check 3 must NOT BLOCK. Other T2 sub-checks (magic numbers etc.) may
    // still surface WARN findings on the fixture — those are out of scope.
    let verdict = t2
        .evaluate(rust_seven_functions(), "src/lib.rs")
        .expect("evaluate")
        .expect("verdict");
    assert!(
        !verdict.is_block(),
        "Check 3 must not fire at slack boundary, got BLOCK: {verdict:?}"
    );
    if let HookVerdict::Block { message, .. } = &verdict {
        assert!(
            !message.contains("function count"),
            "BLOCK from Check 3 path: {message}"
        );
    }
}

// ---------------------------------------------------------------------------
// 14-15. MistakeRecorder T1 wiring
// ---------------------------------------------------------------------------

#[test]
fn mistake_recorder_file_size_exceeded_block_invoked() {
    let mut t1 = empty_t1();
    let rec = RecordingMistakeRecorder::default();
    t1.set_mistake_recorder(Arc::new(rec.clone()));
    t1.set_current_baby_step_id("bs-1");

    // Generate >400 lines of Rust source to trigger the hard_block path.
    let mut source = String::new();
    for i in 0..450 {
        source.push_str(&format!("// line {i}\n"));
    }
    let verdict = t1
        .evaluate("file_write", &source, "src/oversized.rs", FxHashMap::default())
        .expect("evaluate");
    assert!(verdict.is_block(), "oversized file must BLOCK, got {verdict:?}");

    let events = rec.snapshot();
    assert!(
        events.iter().any(|(bs, cat, _, _)| bs == "bs-1"
            && *cat == FailureCategory::FileSizeExceeded),
        "FileSizeExceeded MistakeRecorder event missing: {events:?}"
    );
}

#[test]
fn mistake_recorder_lazy_code_warn_does_not_record() {
    // Per spec L1224-1255: T1 only WARNs on lazy patterns; the BLOCK
    // happens at T2. The recorder must NOT be invoked at the T1 WARN site.
    let mut t1 = empty_t1();
    let rec = RecordingMistakeRecorder::default();
    t1.set_mistake_recorder(Arc::new(rec.clone()));
    t1.set_current_baby_step_id("bs-warn");

    let source = "fn main() {\n    // TODO implement\n}\n";
    let verdict = t1
        .evaluate("file_write", source, "src/main.rs", FxHashMap::default())
        .expect("evaluate");
    assert!(verdict.is_warn(), "TODO comment must WARN, got {verdict:?}");
    assert!(
        rec.snapshot().is_empty(),
        "T1 WARN must not invoke MistakeRecorder"
    );
}

// ---------------------------------------------------------------------------
// 16. FileDisciplineConfig.language_limit_overrides
// ---------------------------------------------------------------------------

#[test]
fn language_limit_overrides_applied() {
    // Default Rust limits: production_warn=100, hard_block=400.
    // Override: production_warn=10, hard_block=20. A 25-line file should
    // BLOCK under the override but WARN/ALLOW under defaults.
    let mut cfg = FileDisciplineConfig::default();
    let mut overrides: FxHashMap<String, LanguageLimits> = FxHashMap::default();
    overrides.insert(
        "rust".to_string(),
        LanguageLimits {
            production_warn: 10,
            test_warn: 15,
            hard_block: 20,
        },
    );
    cfg.language_limit_overrides = overrides;

    let mut t1 = empty_t1();
    t1.set_file_discipline(cfg);

    // 25 lines > 20 hard_block override -> BLOCK.
    let mut source = String::new();
    for i in 0..25 {
        source.push_str(&format!("// row {i}\n"));
    }
    let verdict = t1
        .evaluate("file_write", &source, "src/small.rs", FxHashMap::default())
        .expect("evaluate");
    assert!(
        verdict.is_block(),
        "override hard_block=20 must apply over default 400, got {verdict:?}"
    );
}

#[test]
fn language_limit_overrides_absent_defaults_hold() {
    // No overrides installed (DRIFT-6 OPT-C): defaults apply.
    let cfg = FileDisciplineConfig::default();
    assert!(
        cfg.language_limit_overrides.is_empty(),
        "default FileDisciplineConfig must have empty language_limit_overrides"
    );
    let mut t1 = empty_t1();
    t1.set_file_discipline(cfg);
    // 100 lines should be a WARN under default Rust limits (warn=100, hard=400) — boundary.
    // 50 lines clearly Allow.
    let mut source = String::new();
    for i in 0..50 {
        source.push_str(&format!("// row {i}\n"));
    }
    let verdict = t1
        .evaluate("file_write", &source, "src/small.rs", FxHashMap::default())
        .expect("evaluate");
    assert_eq!(verdict, HookVerdict::Allow);
}

// ---------------------------------------------------------------------------
// Constants match between core + plugins (single source of truth invariant)
// ---------------------------------------------------------------------------

#[test]
fn function_count_constants_match_across_crates() {
    assert_eq!(MAX_FUNCTIONS_PER_FILE, PLUGIN_MAX_FUNCTIONS_PER_FILE);
    assert_eq!(FUNCTION_COUNT_SLACK, 2);
}
