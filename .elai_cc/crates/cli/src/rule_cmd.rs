//! Handlers for the `elai rule` command.

use std::path::{Path, PathBuf};
use elai_core::paths::{dkg_db_path, rules_db_path};
use elai_core::OsClock;
use elai_rules::lifecycle::{fetch_all_active_rules, fetch_all_rules, fetch_rule_for_test, insert_user_rule, UserRuleSpec};
use elai_rules::schema::open_rules_db;
use serde::Serialize;
use rustc_hash::FxHashMap;
use elai_rules::predicate::{Predicate, VerdictResult};
use elai_rules::rule_engine::{DefaultRuleEngine, EvaluationContext, FsmPhaseBit, Rule, RuleEngine};
use elai_rules::stubs::EmptyDkgQuerier;
use elai_knowledge::{DkgQuerierAdapter, TreeSitterAstEvaluator};
use crate::args::CliOutputFormat;

#[derive(Debug, Serialize)]
struct RuleListRow {
    rule_id: String, domain: String, tier: u8, severity: String, title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
}
#[derive(Debug, Serialize)] struct RuleListResponse { rules: Vec<RuleListRow> }

pub(crate) fn print_rule_list(output_format: CliOutputFormat, all: bool) -> anyhow::Result<()> {
    let path = rules_db_path(&std::env::current_dir()?);
    let rows = if path.exists() {
        let conn = open_rules_db(Path::new(&path), &OsClock)?;
        if all {
            fetch_all_rules(&conn)?.into_iter().map(|r| RuleListRow { rule_id:r.rule_id, domain:r.domain, tier:r.tier, severity:r.severity, title:r.title, status:Some(r.status) }).collect()
        } else {
            fetch_all_active_rules(&conn)?.into_iter().map(|r| RuleListRow { rule_id:r.rule_id, domain:r.domain, tier:r.tier, severity:r.severity, title:r.title, status:None }).collect()
        }
    } else { Vec::new() };
    match output_format {
        CliOutputFormat::Json => println!("{}", serde_json::to_string(&RuleListResponse { rules: rows })?),
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            if rows.is_empty() { println!("no rules"); } else { for row in rows { println!("{}  tier={}  {}  {}  {}{}", row.rule_id, row.tier, row.severity, row.domain, row.title, row.status.map(|s| format!("  {s}")).unwrap_or_default()); } }
        }
    }
    Ok(())
}

pub(crate) fn add_rule(
    rule_id: String, domain: String, tier: u8, severity: String, title: String,
    description: String, predicate: String, output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    let path = rules_db_path(&std::env::current_dir()?);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = open_rules_db(Path::new(&path), &OsClock)?;
    insert_user_rule(&conn, &UserRuleSpec { rule_id: rule_id.clone(), domain, tier, severity, title, description, predicate_json: predicate }, &OsClock)?;
    match output_format {
        CliOutputFormat::Json => println!("{}", serde_json::json!({"added": rule_id, "status": "PROPOSED"})),
        CliOutputFormat::Text | CliOutputFormat::Toml => println!("added {rule_id} (PROPOSED)"),
    }
    Ok(())
}


fn needs_dkg(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::GraphReachability { .. } => true,
        Predicate::Composite { children, .. } => children.iter().any(needs_dkg),
        _ => false,
    }
}

#[derive(Debug, Serialize)]
struct RuleTestResponse {
    rule_id: String,
    fixture: String,
    fired: bool,
    verdicts: Vec<elai_rules::predicate::Verdict>,
}

pub(crate) fn test_rule(
    rule_id: String,
    fixture: PathBuf,
    output_format: CliOutputFormat,
) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let path = rules_db_path(&cwd);
    if !path.exists() {
        anyhow::bail!("no rules database: author a rule first with `elai rule add`");
    }
    let conn = open_rules_db(Path::new(&path), &OsClock)?;
    let subject = fetch_rule_for_test(&conn, &rule_id)?;
    let source = std::fs::read_to_string(&fixture)?;
    let parsed = elai_rules::predicate::from_json(&subject.predicate_json)
        .map_err(|e| anyhow::anyhow!("invalid predicate: {e}"))?;
    let dkg_path = dkg_db_path(&cwd);
    let requires_dkg = needs_dkg(&parsed);
    if requires_dkg && !dkg_path.exists() {
        anyhow::bail!(
            "unsupported predicate: GraphReachability requires the DKG index (run `elai index` first)"
        );
    }
    let dkg_querier: Box<dyn elai_rules::predicate::DkgQuerier + Send + Sync> = if dkg_path.exists() {
        match DkgQuerierAdapter::new(&dkg_path) {
            Ok(adapter) => Box::new(adapter),
            Err(e) => {
                if requires_dkg {
                    anyhow::bail!(
                        "unsupported predicate: GraphReachability requires the DKG index, but opening it failed: {e} (rebuild with `elai index`)"
                    );
                }
                eprintln!("[warn] DKG adapter init failed, using empty querier: {e}");
                Box::new(EmptyDkgQuerier)
            }
        }
    } else {
        Box::new(EmptyDkgQuerier)
    };
    let engine = DefaultRuleEngine::new(Box::new(TreeSitterAstEvaluator::new()), dkg_querier);
    let compiled = engine.compile_predicate(&subject.predicate_json)?;
    let file_path = fixture.to_str().unwrap_or_default();
    let context = EvaluationContext {
        action: "test", source: &source, file_path, fsm_phase: FsmPhaseBit::EXECUTING,
        metric_values: FxHashMap::default(), language_agnostic: false,
    };
    let rule = Rule {
        rule_id: subject.rule_id.clone(), domain: subject.domain, tier: subject.tier,
        severity: subject.severity, title: subject.title, predicate_json: subject.predicate_json,
        phase_mask: subject.phase_mask, escalate_target: None,
    };
    let verdicts = engine.evaluate(&[(rule, compiled)], &context)?;
    let fired = verdicts.iter().any(|verdict| !matches!(verdict.result, VerdictResult::Allow));
    match output_format {
        CliOutputFormat::Json => println!("{}", serde_json::to_string(&RuleTestResponse {
            rule_id, fixture: file_path.to_string(), fired, verdicts,
        })?),
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            if fired {
                for verdict in verdicts {
                    if !matches!(verdict.result, VerdictResult::Allow) {
                        println!("{} FIRED  {:?}  {}", rule_id, verdict.result, verdict.evidence);
                    }
                }
            } else {
                println!("{} did not fire on {}", rule_id, file_path);
            }
        }
    }
    Ok(())
}
