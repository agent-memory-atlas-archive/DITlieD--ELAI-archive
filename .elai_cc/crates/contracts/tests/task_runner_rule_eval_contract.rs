// Session 4E.14 contract tests: runtime rule evaluation wired into the
// dashboard-facing task runner.
//
// The 4E.14 surface exposes [`RuleEvaluator`] on
// [`elai_orchestrator::TaskRunnerConfig`]. The full task_runner integration
// path (drive_task -> research_decompose -> PRE_GATE rule eval -> Finding)
// requires a live sidecar for decomposition (see 4D.10 grounded_client),
// which is intentionally out of scope for hermetic contract tests. These
// contracts therefore exercise [`RuleEvaluator::evaluate_at_pre_gate`]
// directly — the pure function the task_runner calls — plus verify the
// wiring surface (field present, default None, rule_count observable).
//
// Covers:
//   1. `TaskRunnerConfig::new_minimal` leaves `rule_evaluator` = None so the
//      pre-4E.14 no-op behaviour stays the default.
//   2. `RuleEvaluator::evaluate_at_pre_gate` filters by `phase_mask` —
//      rules whose mask excludes the requested phase do NOT produce a
//      verdict, even when the domain and source content would otherwise
//      match.
//   3. `RuleEvaluator::evaluate_at_pre_gate` respects domain filtering —
//      a `"web"` rule fires only when the file domain list contains `"web"`
//      OR the rule is in the always-included `"engineering"/"security"`
//      set.
//   4. A BLOCK verdict short-circuits evaluation of remaining rules (first
//      BLOCK stops the loop, matching the rules-engine spec in phase-3F).

use std::sync::Arc;

use rustc_hash::FxHashMap;

use elai_orchestrator::{
    load_seed_rules_from_dir, RuleEvaluator, TaskRunnerConfig, BABY_STEP_PRE_GATE_ACTION,
};
use elai_rules::predicate::{EscalationTarget, VerdictResult};
use elai_rules::rule_engine::{
    CompiledPredicate, DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine,
};
use elai_rules::stubs::{EmptyDkgQuerier, PassAstEvaluator};

const EXECUTING_BIT: u32 = 1u32 << 3;
const AUDITING_BIT: u32 = 1u32 << 4;

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

fn make_rule(
    rule_id: &str,
    domain: &str,
    severity: &str,
    pattern: &str,
    phase_mask: u32,
) -> Rule {
    Rule {
        rule_id: rule_id.to_owned(),
        domain: domain.to_owned(),
        tier: 1,
        severity: severity.to_owned(),
        title: "test rule".to_owned(),
        predicate_json: format!(
            r#"{{"type":"grep_pattern","pattern":"{pattern}","scope":null,"context_lines":0}}"#
        ),
        phase_mask,
        escalate_target: None,
    }
}

fn ctx<'a>(
    action: &'a str,
    source: &'a str,
    file_path: &'a str,
    phase: FsmPhaseBit,
) -> EvaluationContext<'a> {
    EvaluationContext {
        action,
        source,
        file_path,
        fsm_phase: phase,
        metric_values: FxHashMap::default(),
    }
}

// ---------------------------------------------------------------------------
// Contract 1: TaskRunnerConfig::new_minimal leaves rule_evaluator=None
// ---------------------------------------------------------------------------

#[test]
fn new_minimal_config_leaves_rule_evaluator_none() {
    let cfg = TaskRunnerConfig::new_minimal(std::path::PathBuf::from("/tmp/cursor.json"));
    assert!(cfg.rule_evaluator.is_none());
}

// ---------------------------------------------------------------------------
// Contract 2: phase_mask filtering
// ---------------------------------------------------------------------------

#[test]
fn evaluate_at_pre_gate_skips_rules_outside_phase_mask() {
    let engine = new_engine();
    let matching_rule = compile(
        &engine,
        make_rule("R-EXEC", "engineering", "BLOCK", "forbidden", EXECUTING_BIT),
    );
    let auditing_only_rule = compile(
        &engine,
        make_rule("R-AUDIT", "engineering", "BLOCK", "forbidden", AUDITING_BIT),
    );
    let evaluator = RuleEvaluator::new(
        engine.clone(),
        Arc::new(vec![matching_rule, auditing_only_rule]),
    );

    let context = ctx(
        BABY_STEP_PRE_GATE_ACTION,
        "source contains forbidden text",
        "",
        FsmPhaseBit::EXECUTING,
    );
    let verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &context);

    let rule_ids: Vec<_> = verdicts.iter().map(|v| v.rule_id.as_str()).collect();
    assert!(
        rule_ids.contains(&"R-EXEC"),
        "EXECUTING rule should have fired; got {:?}",
        rule_ids,
    );
    assert!(
        !rule_ids.contains(&"R-AUDIT"),
        "AUDITING-only rule must not fire when phase=EXECUTING; got {:?}",
        rule_ids,
    );
}

// ---------------------------------------------------------------------------
// Contract 3: domain filtering with always-included override
// ---------------------------------------------------------------------------

#[test]
fn evaluate_at_pre_gate_domain_filter_and_always_included() {
    let engine = new_engine();
    let web_rule = compile(
        &engine,
        make_rule("R-WEB", "web", "WARN", "forbidden", EXECUTING_BIT),
    );
    let engineering_rule = compile(
        &engine,
        make_rule("R-ENG", "engineering", "WARN", "forbidden", EXECUTING_BIT),
    );
    let evaluator = RuleEvaluator::new(
        engine.clone(),
        Arc::new(vec![web_rule, engineering_rule]),
    );

    let context = ctx(
        BABY_STEP_PRE_GATE_ACTION,
        "source forbidden",
        "",
        FsmPhaseBit::EXECUTING,
    );

    // Empty domain list: only the always-included "engineering" rule fires.
    let non_web = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &context);
    let non_web_ids: Vec<_> = non_web.iter().map(|v| v.rule_id.as_str()).collect();
    assert!(non_web_ids.contains(&"R-ENG"));
    assert!(
        !non_web_ids.contains(&"R-WEB"),
        "web rule must be skipped when domain list is empty; got {:?}",
        non_web_ids,
    );

    // Adding "web" to the domain list activates the web rule.
    let web =
        evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &["web"], &context);
    let web_ids: Vec<_> = web.iter().map(|v| v.rule_id.as_str()).collect();
    assert!(web_ids.contains(&"R-WEB"));
    assert!(web_ids.contains(&"R-ENG"));
}

// ---------------------------------------------------------------------------
// Contract 4: first BLOCK short-circuits evaluation
// ---------------------------------------------------------------------------

#[test]
fn evaluate_at_pre_gate_first_block_short_circuits() {
    let engine = new_engine();
    let rules: Vec<(Rule, CompiledPredicate)> = vec![
        compile(
            &engine,
            make_rule("R-BLOCK", "engineering", "BLOCK", "forbidden", EXECUTING_BIT),
        ),
        compile(
            &engine,
            make_rule(
                "R-AFTER",
                "engineering",
                "WARN",
                "forbidden",
                EXECUTING_BIT,
            ),
        ),
    ];
    let evaluator = RuleEvaluator::new(engine, Arc::new(rules));

    let context = ctx(
        BABY_STEP_PRE_GATE_ACTION,
        "source forbidden",
        "",
        FsmPhaseBit::EXECUTING,
    );
    let verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &context);

    assert_eq!(
        verdicts.len(),
        1,
        "expected short-circuit after first BLOCK; got {verdicts:?}",
    );
    assert_eq!(verdicts[0].rule_id, "R-BLOCK");
    assert!(matches!(verdicts[0].result, VerdictResult::Block));
}

// ---------------------------------------------------------------------------
// Contract 5: rules_len mirrors the compiled rule count
// ---------------------------------------------------------------------------

#[test]
fn rules_len_reflects_compiled_rule_count() {
    let engine = new_engine();
    let rules = vec![
        compile(
            &engine,
            make_rule("R1", "engineering", "WARN", "x", EXECUTING_BIT),
        ),
        compile(
            &engine,
            make_rule("R2", "engineering", "WARN", "y", EXECUTING_BIT),
        ),
        compile(
            &engine,
            make_rule("R3", "web", "BLOCK", "z", EXECUTING_BIT),
        ),
    ];
    let evaluator = RuleEvaluator::new(engine, Arc::new(rules));
    assert_eq!(evaluator.rules_len(), 3);
}

// ---------------------------------------------------------------------------
// Contract 6: Escalate verdict surfaces as a distinct result (audit BLOCK fix)
// ---------------------------------------------------------------------------

#[test]
fn evaluate_at_pre_gate_surfaces_escalate_verdicts() {
    let engine = new_engine();
    let escalate_rule = Rule {
        rule_id: "R-ESCALATE".to_owned(),
        domain: "engineering".to_owned(),
        tier: 1,
        severity: "ESCALATE".to_owned(),
        title: "escalating rule".to_owned(),
        // Predicate must fire so the severity-verdict path reaches the
        // Escalate branch in evaluate_single (severity="ESCALATE" routes
        // Allow->Allow, Fires->Escalate(target)).
        predicate_json: r#"{"type":"grep_pattern","pattern":"forbidden","scope":null,"context_lines":0}"#
            .to_owned(),
        phase_mask: EXECUTING_BIT,
        escalate_target: Some(EscalationTarget::Human),
    };
    let compiled = engine
        .compile_predicate(&escalate_rule.predicate_json)
        .expect("compile");
    let evaluator = RuleEvaluator::new(
        Arc::new(DefaultRuleEngine::new(
            Box::new(PassAstEvaluator),
            Box::new(EmptyDkgQuerier),
        )),
        Arc::new(vec![(escalate_rule, compiled)]),
    );

    let context = ctx(
        BABY_STEP_PRE_GATE_ACTION,
        "contains forbidden marker",
        "",
        FsmPhaseBit::EXECUTING,
    );
    let verdicts = evaluator.evaluate_at_pre_gate(FsmPhaseBit::EXECUTING, &[], &context);

    assert_eq!(verdicts.len(), 1);
    assert!(
        matches!(
            verdicts[0].result,
            VerdictResult::Escalate(EscalationTarget::Human),
        ),
        "expected Escalate(Human), got {:?}",
        verdicts[0].result,
    );
}

// ---------------------------------------------------------------------------
// Contract 7: load_seed_rules_from_dir — happy / empty / unreadable paths
// ---------------------------------------------------------------------------

#[test]
fn load_seed_rules_from_dir_returns_compiled_pairs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let toml_path = tmp.path().join("seed.toml");
    std::fs::write(
        &toml_path,
        concat!(
            "[[rules]]\n",
            "rule_id = \"SEED-1\"\n",
            "domain = \"engineering\"\n",
            "tier = 1\n",
            "severity = \"WARN\"\n",
            "title = \"test\"\n",
            "predicate_json = '{\"type\":\"grep_pattern\",\"pattern\":\"x\",\"scope\":null,\"context_lines\":0}'\n",
            "phase_mask = 4294967295\n",
        ),
    )
    .expect("write seed toml");

    let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
    let compiled = load_seed_rules_from_dir(tmp.path(), &engine);
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].0.rule_id, "SEED-1");
}

#[test]
fn load_seed_rules_from_dir_skips_missing_directory() {
    let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
    let compiled =
        load_seed_rules_from_dir(std::path::Path::new("/nonexistent/path/does/not/exist"), &engine);
    assert!(compiled.is_empty());
}

#[test]
fn load_seed_rules_from_dir_skips_unparseable_file_but_keeps_siblings() {
    let tmp = tempfile::tempdir().expect("tempdir");
    // Good rule file.
    std::fs::write(
        tmp.path().join("good.toml"),
        concat!(
            "[[rules]]\n",
            "rule_id = \"GOOD\"\n",
            "domain = \"engineering\"\n",
            "tier = 1\n",
            "severity = \"WARN\"\n",
            "title = \"ok\"\n",
            "predicate_json = '{\"type\":\"grep_pattern\",\"pattern\":\"y\",\"scope\":null,\"context_lines\":0}'\n",
            "phase_mask = 4294967295\n",
        ),
    )
    .expect("write good");
    // Malformed TOML — must not abort the whole directory walk.
    std::fs::write(tmp.path().join("bad.toml"), "not valid = = toml [[[").expect("write bad");
    // Non-toml — must be ignored regardless.
    std::fs::write(tmp.path().join("readme.md"), "# not a rule").expect("write readme");

    let engine = DefaultRuleEngine::new(Box::new(PassAstEvaluator), Box::new(EmptyDkgQuerier));
    let compiled = load_seed_rules_from_dir(tmp.path(), &engine);
    assert_eq!(compiled.len(), 1);
    assert_eq!(compiled[0].0.rule_id, "GOOD");
}
