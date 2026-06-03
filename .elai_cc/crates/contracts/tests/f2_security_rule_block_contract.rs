//! F-2a security rule block contract tests.
//!
//! Loads the 4 security TOML files (sqli / xss / weak_crypto / missing_authz),
//! evaluates a Python f-string SQL concatenation through the DefaultRuleEngine,
//! and asserts a security_rule_blocked outcome with the SEC-SQLI-001 rule and
//! CWE-89. This exercises the full GrepPattern evaluation path on the F-2a seed.

use elai_orchestrator::rule_loader::load_security_seed_rules_builtin;
use elai_orchestrator::vibegate::{evaluate_vibegate, VIBESEC_CODE_VULN_CATEGORY};
use elai_rules::predicate::{AstError, AstEvaluator, AstMatch, DkgError, DkgQuerier, ReachabilityMatch, ReachabilityParams, VerdictResult};
use elai_rules::rule_engine::{DefaultRuleEngine, EvaluationContext, FsmPhaseBit, RuleEngine};
use elai_orchestrator::hook_dispatcher::HookVerdictKind;
use rustc_hash::FxHashMap;

// ---------------------------------------------------------------------------
// NoOp AST / DKG (F-2a rules are all GrepPattern, AST/DKG not needed)
// ---------------------------------------------------------------------------

struct NoOpAst;
impl AstEvaluator for NoOpAst {
    fn evaluate_query(&self, _lang: &str, _src: &str, _q: &str) -> Result<Vec<AstMatch>, AstError> {
        Ok(vec![])
    }
    fn validate_query(&self, _lang: &str, _q: &str) -> Result<(), AstError> {
        Ok(())
    }
}

struct NoOpDkg;
impl DkgQuerier for NoOpDkg {
    fn query_reachability(&self, _p: &ReachabilityParams) -> Result<Vec<ReachabilityMatch>, DkgError> {
        Ok(vec![])
    }
    fn validate_params(&self, _p: &ReachabilityParams) -> Result<(), DkgError> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Contract 1: security TOMLs load (at least 12 rules) + SQLi predicate fires
// ---------------------------------------------------------------------------

#[test]
fn contract_security_seed_rules_load_at_least_12() {
    let engine = DefaultRuleEngine::new(Box::new(NoOpAst), Box::new(NoOpDkg));
    let rules = load_security_seed_rules_builtin(&engine);
    assert!(
        rules.len() >= 12,
        "security seed must load at least 12 rules (4 TOMLs × 3 each); got {}",
        rules.len()
    );
}

#[test]
fn contract_security_rule_ids_include_all_families() {
    let engine = DefaultRuleEngine::new(Box::new(NoOpAst), Box::new(NoOpDkg));
    let rules = load_security_seed_rules_builtin(&engine);
    let ids: Vec<&str> = rules.iter().map(|(r, _)| r.rule_id.as_str()).collect();

    for expected_prefix in &["SEC-SQLI-", "SEC-XSS-", "SEC-WCRYPTO-", "SEC-AUTHZ-"] {
        assert!(
            ids.iter().any(|id| id.starts_with(expected_prefix)),
            "security seed must include {expected_prefix}* rules; found ids: {ids:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Contract 2: Python f-string SQL injection triggers CWE-89 block
// ---------------------------------------------------------------------------

#[test]
fn contract_python_fstring_sqli_triggers_cwe89_block() {
    let engine = DefaultRuleEngine::new(Box::new(NoOpAst), Box::new(NoOpDkg));
    let rules = load_security_seed_rules_builtin(&engine);

    // SEC-SQLI-001 pattern: execute(f"SELECT ... {var}")
    let source = r#"cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")"#;
    let context = EvaluationContext {
        action: "Edit",
        source,
        file_path: "app/db.py",
        fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: FxHashMap::default(),
        language_agnostic: false,
    };

    let verdicts = engine.evaluate(&rules, &context).expect("evaluate must not error");
    let block = verdicts
        .iter()
        .find(|v| matches!(v.result, VerdictResult::Block) && v.rule_id.starts_with("SEC-SQLI-"));
    assert!(
        block.is_some(),
        "Python f-string SQLi must trigger a Block verdict with a SEC-SQLI-* rule; got: {verdicts:?}"
    );
    let block = block.unwrap();
    // The rule id that fires must be SEC-SQLI-001 (f-string variant).
    assert_eq!(
        block.rule_id, "SEC-SQLI-001",
        "f-string SQLi must be caught by SEC-SQLI-001 specifically"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: evaluate_vibegate surfaces the sqli block as a vibesec_code_vuln
// ---------------------------------------------------------------------------

#[test]
fn contract_evaluate_vibegate_surfaces_sqli_as_code_vuln() {
    let source = r#"cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")"#;
    let findings = evaluate_vibegate("Edit", source, "app/queries.py");
    let block = findings.iter().find(|f| {
        f.verdict == HookVerdictKind::Block && f.category == VIBESEC_CODE_VULN_CATEGORY
    });
    assert!(
        block.is_some(),
        "evaluate_vibegate must surface SQLi as a vibesec_code_vuln Block; got: {findings:?}"
    );
    let b = block.unwrap();
    assert_eq!(
        b.rule_id.as_deref(),
        Some("SEC-SQLI-001"),
        "rule_id must be SEC-SQLI-001"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: Python .format() SQL injection also triggers a block
// ---------------------------------------------------------------------------

#[test]
fn contract_python_format_sqli_triggers_block() {
    let source = r#""SELECT * FROM users WHERE id = %s" % (user_id,)"#;
    let findings = evaluate_vibegate("Write", source, "app/legacy.py");
    let block = findings
        .iter()
        .find(|f| f.verdict == HookVerdictKind::Block && f.category == VIBESEC_CODE_VULN_CATEGORY);
    assert!(
        block.is_some(),
        ".format()/%% SQLi must trigger a vibesec_code_vuln Block; got: {findings:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 5: domains are always included — security rules are ALWAYS_INCLUDED
// ---------------------------------------------------------------------------

#[test]
fn contract_security_domain_is_always_included() {
    use elai_rules::rule_engine::ALWAYS_INCLUDED_DOMAINS;
    assert!(
        ALWAYS_INCLUDED_DOMAINS.contains(&"security"),
        "security domain must be in ALWAYS_INCLUDED_DOMAINS"
    );
}
