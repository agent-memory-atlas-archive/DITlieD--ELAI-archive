//! Handlers for the `elai rule` command.

use std::path::Path;
use elai_core::paths::rules_db_path;
use elai_core::OsClock;
use elai_rules::lifecycle::{fetch_all_active_rules, fetch_all_rules, insert_user_rule, UserRuleSpec};
use elai_rules::schema::open_rules_db;
use serde::Serialize;
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
