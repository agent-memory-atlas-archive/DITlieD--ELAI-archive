//! Read-only handler for `elai rule list`.

use std::path::Path;

use elai_core::paths::rules_db_path;
use elai_core::OsClock;
use elai_rules::lifecycle::fetch_all_active_rules;
use elai_rules::schema::open_rules_db;
use serde::Serialize;

use crate::args::CliOutputFormat;

#[derive(Debug, Serialize)]
struct RuleListRow {
    rule_id: String,
    domain: String,
    tier: u8,
    severity: String,
    title: String,
}

#[derive(Debug, Serialize)]
struct RuleListResponse {
    rules: Vec<RuleListRow>,
}

/// Print active user-authorable rules without creating a rules database.
pub(crate) fn print_rule_list(output_format: CliOutputFormat) -> anyhow::Result<()> {
    let project_root = std::env::current_dir()?;
    let path = rules_db_path(&project_root);
    let rows = if path.exists() {
        let conn = open_rules_db(Path::new(&path), &OsClock)?;
        fetch_all_active_rules(&conn)?
            .into_iter()
            .map(|rule| RuleListRow {
                rule_id: rule.rule_id,
                domain: rule.domain,
                tier: rule.tier,
                severity: rule.severity,
                title: rule.title,
            })
            .collect()
    } else {
        Vec::new()
    };

    match output_format {
        CliOutputFormat::Json => {
            println!("{}", serde_json::to_string(&RuleListResponse { rules: rows })?);
        }
        CliOutputFormat::Text | CliOutputFormat::Toml => {
            if rows.is_empty() {
                println!("no rules");
            } else {
                for row in rows {
                    println!(
                        "{}  tier={}  {}  {}  {}",
                        row.rule_id, row.tier, row.severity, row.domain, row.title
                    );
                }
            }
        }
    }
    Ok(())
}
