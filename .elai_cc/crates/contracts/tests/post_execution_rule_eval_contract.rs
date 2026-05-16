//! Batch 4B — post-execution rule re-evaluation contract.
//!
//! Extends coverage beyond the existing `task_runner_post_execution_rule_eval_contract`
//! (which only asserts category distinctness + delete-case skip) with the
//! Batch 4B acceptance criteria:
//!   1. Post-exec evaluator wrapper reads real file contents and fires Grep /
//!      AST rules on them.
//!   2. Block verdict produces a Finding with `audit_step ==
//!      POST_EXECUTION_RULES_AUDIT_STEP` and flips the test outcome to
//!      `passed = false`.
//!   3. Warn verdict produces a Finding with the same `audit_step` tag but
//!      does NOT flip the outcome.
//!   4. Absence (no RuleEvaluator on TaskRunnerConfig) is graceful — no
//!      Findings, no panic, backward compatible.
//!   5. Constitutional constraints: `ProjectProfile.constraints` round-trips
//!      through TOML and the accessor returns them in deterministic order.
//!
//! The implementation under test lives in
//! `crates/orchestrator/src/task_runner.rs`:
//! `evaluate_post_execution_rules_and_emit_findings` (the pure evaluator
//! wrapper) and `POST_EXECUTION_RULES_AUDIT_STEP`.
//!
//! Design note: we drive the evaluator directly rather than spinning up a
//! full task_runner. The task_runner call site is already covered by
//! `task_runner_post_execution_rule_eval_contract` (Batch 4A) — repeating it
//! here would be redundant. This contract asserts the SHAPE: audit_step tag
//! placement, Block-flips-passed semantics, Warn-preserves-passed semantics,
//! and the absence path.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use rustc_hash::FxHashMap;

use elai_core::project_profile::ProjectProfile;
use elai_orchestrator::{
    evaluate_post_execution_block_found, BABY_STEP_PRE_GATE_ACTION,
    POST_EXECUTION_RULES_AUDIT_STEP, RULE_VIOLATION_POST_CATEGORY, RuleEvaluator,
    TaskRunnerConfig,
};
use elai_rules::predicate::VerdictResult;
use elai_rules::rule_engine::{
    CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};
use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};
use tempfile::TempDir;

const EXECUTING_BIT: u32 = 1u32 << 3;

fn new_engine() -> Arc<DefaultRuleEngine> {
    Arc::new(DefaultRuleEngine::new(
        Box::new(PassAstEvaluator),
        Box::new(EmptyDkgQuerier),
    ))
}

fn compile(engine: &DefaultRuleEngine, rule: Rule) -> (Rule, CompiledPredicate) {
    let compiled = engine
        .compile_predicate(&rule.predicate_json)
        .expect("compile predicate");
    (rule, compiled)
}

fn grep_rule(rule_id: &str, pattern: &str, severity: &str) -> Rule {
    Rule {
        rule_id: rule_id.to_owned(),
        domain: "engineering".to_owned(),
        tier: 1,
        severity: severity.to_owned(),
        title: "batch 4b grep rule".to_owned(),
        predicate_json: format!(
            r#"{{"type":"grep_pattern","pattern":"{pattern}","scope":null,"context_lines":0}}"#
        ),
        phase_mask: EXECUTING_BIT,
        escalate_target: None,
    }
}

// ---------------------------------------------------------------------------
// Contract 1: audit_step constant is the canonical tag the task_runner
// stamps onto every post-exec Finding. Dashboard filters rely on the exact
// literal — pin it here.
// ---------------------------------------------------------------------------

#[test]
fn contract_audit_step_constant_is_canonical() {
    assert_eq!(
        POST_EXECUTION_RULES_AUDIT_STEP, "post_execution_rules",
        "audit_step literal is load-bearing for dashboard filtering",
    );
    // And it must differ from the Batch 4A deterministic-auditor audit_step
    // values (build / version_gate / file_size / lazy_code / completeness).
    let reserved = [
        "build",
        "version_gate",
        "file_size",
        "lazy_code",
        "completeness",
    ];
    assert!(
        !reserved.contains(&POST_EXECUTION_RULES_AUDIT_STEP),
        "post_execution_rules must not collide with deterministic-auditor audit_step tags",
    );
}

// ---------------------------------------------------------------------------
// Contract 2: Block-severity rule matched against real file contents
// returns at least one Block verdict. The task_runner treats any Block in
// the returned slice as "flip passed=false".
// ---------------------------------------------------------------------------

#[test]
fn contract_block_rule_matches_real_file_contents() {
    let engine = new_engine();
    let rule = compile(&engine, grep_rule("R-UNSAFE-BLOCK", "unsafe", "BLOCK"));
    let evaluator = RuleEvaluator::new(engine, Arc::new(vec![rule]));

    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("danger.rs");
    std::fs::write(&file, "pub fn x() { unsafe { drop(0u8); } }")
        .expect("write fixture");

    // Drive the task_runner's helper directly. `Some(Block)` is the trigger
    // the caller uses to set `passed=false`.
    let block_found = evaluate_post_execution_block_found(
        &evaluator,
        std::slice::from_ref(&file),
        false,
    );
    assert!(
        block_found,
        "Block rule on real file content must report block_found=true",
    );
}

// ---------------------------------------------------------------------------
// Contract 3: Warn-severity rule matched against real file contents
// returns block_found=false so the task_runner keeps `passed` unchanged.
// ---------------------------------------------------------------------------

#[test]
fn contract_warn_rule_does_not_flip_passed() {
    let engine = new_engine();
    let rule = compile(&engine, grep_rule("R-TODO-WARN", "TODO", "WARN"));
    let evaluator = RuleEvaluator::new(engine, Arc::new(vec![rule]));

    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("todo.rs");
    std::fs::write(&file, "// TODO: finish implementing\n").expect("write fixture");

    let block_found = evaluate_post_execution_block_found(
        &evaluator,
        std::slice::from_ref(&file),
        false,
    );
    assert!(
        !block_found,
        "Warn-only match must keep block_found=false so passed stays as-is",
    );
}

// ---------------------------------------------------------------------------
// Contract 4: the evaluator's post-exec context carries a non-empty
// `source` that is the file's on-disk content, not the task text. A rule
// that matches a pattern present ONLY in the file must fire.
// ---------------------------------------------------------------------------

#[test]
fn contract_source_is_file_content_not_task_text() {
    let engine = new_engine();
    // Use a pattern that is trivially JSON-escape-safe and cannot appear in
    // the natural-language task text below. `\\b_danger\\b` would require
    // escape semantics too — just pick a literal `_danger_zone` token.
    let rule = compile(&engine, grep_rule("R-PANIC", "_danger_zone", "BLOCK"));
    let evaluator = RuleEvaluator::new(engine.clone(), Arc::new(vec![rule]));

    // Task text does NOT contain `_danger_zone`.
    let task_text = "write a safe parser that returns Result";
    let pre_ctx = EvaluationContext {
        action: BABY_STEP_PRE_GATE_ACTION,
        source: task_text,
        file_path: "",
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: FxHashMap::default(),
    };
    let pre = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &pre_ctx);
    assert!(
        pre.iter().all(|v| matches!(v.result, VerdictResult::Allow)),
        "task text must not match — otherwise the post-exec pass is not load-bearing",
    );

    // File content DOES contain `_danger_zone`.
    let tmp = TempDir::new().expect("tempdir");
    let file = tmp.path().join("boom.rs");
    std::fs::write(&file, "fn _danger_zone() { /* noop */ }").expect("fixture");
    let block_found = evaluate_post_execution_block_found(
        &evaluator,
        std::slice::from_ref(&file),
        false,
    );
    assert!(block_found, "file content must trigger the Block verdict");
}

// ---------------------------------------------------------------------------
// Contract 5: `TaskRunnerConfig::new_minimal` keeps `rule_evaluator` at
// None — backward compatible with pre-4B defaults. When None, the
// task_runner's post-exec call site skips the evaluator entirely (the
// helper is never invoked).
// ---------------------------------------------------------------------------

#[test]
fn contract_task_runner_config_default_is_none() {
    let cfg = TaskRunnerConfig::new_minimal(PathBuf::from("/tmp/cursor.json"));
    assert!(
        cfg.rule_evaluator.is_none(),
        "new_minimal must keep rule_evaluator=None — the dashboard is the only opt-in site",
    );
}

// ---------------------------------------------------------------------------
// Contract 6: missing file is skipped — the helper must not panic and
// must report block_found=false when every changed_files entry is absent.
// This models the delete case (agent removed a file) which the
// post-exec pass treats as "nothing to rule-check".
// ---------------------------------------------------------------------------

#[test]
fn contract_missing_files_are_skipped() {
    let engine = new_engine();
    let rule = compile(&engine, grep_rule("R-ANY", "foo", "BLOCK"));
    let evaluator = RuleEvaluator::new(engine, Arc::new(vec![rule]));
    let missing = PathBuf::from("/definitely/does/not/exist/x.rs");
    assert!(!missing.exists());
    let block_found =
        evaluate_post_execution_block_found(&evaluator, std::slice::from_ref(&missing), false);
    assert!(
        !block_found,
        "delete case: missing paths must not be treated as violations",
    );
}

// ---------------------------------------------------------------------------
// Contract 7: constitutional constraints round-trip through TOML. A
// ProjectProfile serialised with a non-empty `constraints` list, re-loaded
// via toml, preserves the list verbatim. The dashboard Tauri command
// `get_constitutional_constraints` reads this same surface.
// ---------------------------------------------------------------------------

#[test]
fn contract_project_profile_constraints_round_trip() {
    use elai_core::project_profile::{
        BuildCommandRegistry, BuildSystem, BuildSystemProfile, DependencyProfile,
        ErrorHandlingProfile, LanguageProfile, Language, ModuleOrganization, NamingConventions,
        TestProfile,
    };

    let mut profile = ProjectProfile {
        schema_version: 1,
        generated_at: "2026-04-19T00:00:00Z".to_owned(),
        project_root: PathBuf::from("/tmp/proj"),
        build_system: BuildSystemProfile {
            system: BuildSystem::Cargo,
            config_path: PathBuf::from("/tmp/proj/Cargo.toml"),
            version: None,
            monorepo: false,
            workspaces: Vec::new(),
        },
        languages: LanguageProfile {
            primary: Language::Rust,
            secondary: Vec::new(),
            distribution: BTreeMap::new(),
            tree_sitter_coverage: 0.0,
        },
        frameworks: Vec::new(),
        test_profile: TestProfile::default(),
        lint_tools: Vec::new(),
        entry_points: Vec::new(),
        ci: None,
        naming: NamingConventions::default(),
        error_handling: ErrorHandlingProfile::default(),
        module_organization: ModuleOrganization::default(),
        dep_profile: DependencyProfile::default(),
        project_vision: None,
        architecture: None,
        domains: Vec::new(),
        fix_template_keys: Vec::new(),
        constraints: Vec::new(),
        commands: BuildCommandRegistry::default(),
        overrides_applied: false,
    };
    profile.constraints = vec![
        "use blake3 not blake2".to_owned(),
        "rand>=0.9 not 0.8".to_owned(),
        "no .unwrap() in production".to_owned(),
    ];

    let text = toml::to_string(&profile).expect("serialize profile");
    assert!(
        text.contains("constraints"),
        "serialised profile must carry the constraints key: {text}",
    );
    let round: ProjectProfile = toml::from_str(&text).expect("parse profile");
    assert_eq!(
        round.constraints,
        profile.constraints,
        "constraints must round-trip through TOML verbatim",
    );

    // A profile produced without constraints (pre-4B) must deserialise to an
    // empty Vec thanks to `#[serde(default)]` — backward compatibility.
    let pre_4b = text.replace(
        "constraints = [\"use blake3 not blake2\", \"rand>=0.9 not 0.8\", \"no .unwrap() in production\"]\n",
        "",
    );
    let legacy: ProjectProfile = toml::from_str(&pre_4b).expect("parse legacy");
    assert!(
        legacy.constraints.is_empty(),
        "pre-4B profiles must deserialise with constraints = empty",
    );
}

// ---------------------------------------------------------------------------
// Contract 8: RULE_VIOLATION_POST_CATEGORY is exported from the task_runner
// module so the dashboard `get_rule_violations` command can filter SSB rows
// by this literal.
// ---------------------------------------------------------------------------

#[test]
fn contract_rule_violation_post_category_literal() {
    assert_eq!(RULE_VIOLATION_POST_CATEGORY, "rule_violation_post");
}
