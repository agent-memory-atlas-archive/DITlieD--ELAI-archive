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
    let cwd = std::env::current_dir()?;
    let path = rules_db_path(&cwd);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = open_rules_db(Path::new(&path), &OsClock)?;
    // J-109 A2.1: flagged admission probe IMMEDIATELY BEFORE the insert_user_rule
    // write, over the rule surface (title + description + predicate_json)
    // composed exactly as the UserRuleSpec below persists it and as
    // lifecycle::insert_user_rule itself scans it. Same observe-only check A1
    // wired at the E-5 capture front (compose, never duplicate). DEFAULT-OFF;
    // when ON, OBSERVE-ONLY: the insert below proceeds REGARDLESS of the
    // emitted verdict (RT-6; block-mode is a later gate).
    if crate::user_memory_admission::admission_hardening_enabled() {
        let rule_surface = format!("{title}\n{description}\n{predicate}");
        let clock: std::sync::Arc<dyn elai_core::clock::ClockProvider> =
            std::sync::Arc::new(OsClock);
        crate::user_memory_admission::emit_admission_verdict_for_seam(
            &clock,
            &cwd,
            &rule_surface,
            crate::user_memory_admission::ADMISSION_SEAM_I12_RULE_ADD,
        );
    }
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

// ---------------------------------------------------------------------------
// J-109 UNIT A2 (falsifiable-mutation hook, plan A2.2 -- mirror of the UNIT A1
// seam test in user_memory_capture)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // J-109 A2 fix 1: take the ONE crate-shared CWD lock (user_memory_capture)
    // so this probe cannot race the user_memory_capture j109 probe's temp-root
    // setup/open inside the same parallel test binary.
    use crate::user_memory_capture::CWD_LOCK;

    /// J-109 seam test (the R8 falsifiable-mutation hook): drive the REAL
    /// `add_rule` -- the exact fn the `CliAction::RuleAdd` dispatch (main.rs:769)
    /// calls -- with the admission flag ON against a temp project root whose cwd
    /// CLAUDE.md carries one doctrine line, adding a rule whose composed surface
    /// (title + description + predicate_json) IS that doctrine. Asserts BOTH
    /// halves of the observe-only pair: (1) the `admission_verdict` SSB row
    /// landed for the I-12 rule front, stamped with
    /// `ADMISSION_SEAM_I12_RULE_ADD` and carrying recomputable receipts (GD-2),
    /// and (2) the rule STILL persisted to the rules store at PROPOSED (RT-6:
    /// the insert proceeds regardless of the reject verdict). Deleting the
    /// pre-`insert_user_rule` probe block inside `add_rule` turns exactly THIS
    /// test red while the contracts-crate pure-fn arms stay green -- that
    /// asymmetry is the point.
    #[test]
    fn j109_rule_add_admission_probe_emits_row_and_insert_still_persists() {
        let _cwd = CWD_LOCK.lock().expect("cwd lock");
        let tmp = tempfile::tempdir().expect("tempdir");
        // One doctrine manifest member BY CONSTRUCTION (core/config.rs:316 seeds
        // the cwd CLAUDE.md into admission_scan), carrying BOTH the doctrine
        // sentence AND the rule's predicate-JSON text, so the composed rule
        // surface tokenizes fully inside one <=48-token span window ->
        // lexical_overlap 1.0 >= T_IMMUTABLE_LEXICAL (max-margin reject).
        std::fs::write(
            tmp.path().join("CLAUDE.md"),
            "# Doctrine\n\nAlways run the auditor before every merge to main.\n\nRule predicate example: {\"type\":\"grep_pattern\",\"pattern\":\"auditor\",\"scope\":null,\"context_lines\":0}\n",
        )
        .expect("write doctrine CLAUDE.md");

        let prev_cwd = std::env::current_dir().expect("cwd");
        std::env::set_current_dir(tmp.path()).expect("set cwd");

        // Serialize the process-global flag for the drive; admission hardening
        // ON so the probe fires (no other flag gates the rule-add arm).
        let flag = crate::user_memory_admission::ADMISSION_HARDENING_FLAG;
        let prev_adm = std::env::var(flag).ok();
        std::env::set_var(flag, "1");

        let rule_id = "j109_seam_rule".to_string();
        let title = "auditor before merge".to_string();
        let description = "always run the auditor before every merge to main".to_string();
        let predicate =
            r#"{"type":"grep_pattern","pattern":"auditor","scope":null,"context_lines":0}"#
                .to_string();
        let add = add_rule(
            rule_id.clone(),
            "testing".to_string(),
            2,
            // Canonical severity literal: the rules store schema CHECKs
            // severity IN ('BLOCK','WARN','INFO') (rules/src/schema.rs:45) and
            // the production path passes the CLI string through unnormalized.
            "WARN".to_string(),
            title.clone(),
            description.clone(),
            predicate.clone(),
            CliOutputFormat::Text,
        );

        // Restore flag + cwd BEFORE asserting (nothing above early-returns).
        match prev_adm {
            Some(v) => std::env::set_var(flag, v),
            None => std::env::remove_var(flag),
        }
        std::env::set_current_dir(prev_cwd).expect("restore cwd");

        // (1) The add itself must still SUCCEED (observe-only: a reject verdict
        // never refuses the write).
        add.expect("doctrine-duplicate rule add must still succeed");

        // (2) The admission_verdict SSB row landed in the temp project's
        // stream, stamped with the I-12 seam label and recomputable receipts
        // (GD-2). Exactly ONE row: this probe is the rule-add path's only
        // admission_verdict emitter.
        let ssb_db = elai_core::paths::sessions_dir_path(tmp.path()).join("ssb.db");
        let conn = rusqlite::Connection::open(&ssb_db).expect("open ssb.db");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ssb WHERE action_type = 'admission_verdict'",
                [],
                |row| row.get(0),
            )
            .expect("count admission_verdict rows");
        assert_eq!(count, 1, "exactly one admission_verdict row behind one rule add");
        let blob: Vec<u8> = conn
            .query_row(
                "SELECT payload FROM ssb WHERE action_type = 'admission_verdict'",
                [],
                |row| row.get(0),
            )
            .expect("admission_verdict row must exist (deleting the add_rule probe block makes THIS fetch fail)");
        let decoded =
            elai_orchestrator::ssb::decode_ssb_payload(&blob).expect("decode admission_verdict payload");
        match decoded {
            elai_orchestrator::ssb::SsbPayload::AdmissionVerdict(p) => {
                assert_eq!(p.verdict, "reject_immutable_span");
                assert_eq!(p.surface_leg, "lexical");
                assert_eq!(
                    p.originating_seam,
                    crate::user_memory_admission::ADMISSION_SEAM_I12_RULE_ADD
                );
                let rule_surface = format!("{title}\n{description}\n{predicate}");
                assert_eq!(
                    p.candidate_hash,
                    blake3::hash(rule_surface.as_bytes()).to_hex().to_string(),
                    "candidate_hash = blake3 of the composed rule surface (GD-2)"
                );
            }
            other => panic!("expected AdmissionVerdict payload, got {other:?}"),
        }

        // (3) The rule persisted DESPITE the reject verdict (observe-only).
        let rules_conn =
            open_rules_db(Path::new(&rules_db_path(tmp.path())), &OsClock).expect("open rules db");
        let rules = fetch_all_rules(&rules_conn).expect("fetch_all_rules");
        assert_eq!(rules.len(), 1, "the doctrine-duplicate rule still persisted");
        assert_eq!(rules[0].rule_id, rule_id);
        assert_eq!(rules[0].status, "PROPOSED");
    }
}
