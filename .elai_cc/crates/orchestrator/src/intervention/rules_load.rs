// J-12 S5: load protocol rules from rules.db (domain=intervention) into ActionGate.

use crate::intervention::{ActionGate, ProtocolRule, RuleLifecycleStage};
use elai_core::clock::OsClock;
use elai_core::paths::rules_db_path;
use elai_rules::schema::open_rules_db;
use rusqlite::Connection;
use serde_json::Value;
use std::path::Path;
use tracing::warn;

fn load_protocol_rules_from_db(conn: &Connection) -> Vec<ProtocolRule> {
    let mut stmt = match conn.prepare(
        "SELECT rule_id, predicate, status FROM rules \
         WHERE domain = 'intervention' AND status IN ('ACTIVE', 'SHADOW')",
    ) {
        Ok(s) => s,
        Err(e) => {
            warn!("rules_load: prepare query failed: {e}");
            return Vec::new();
        }
    };

    let rows = stmt.query_map([], |row| {
        let rule_id: String = row.get(0)?;
        let predicate: String = row.get(1)?;
        let status: String = row.get(2)?;
        Ok((rule_id, predicate, status))
    });

    let mut rules = Vec::new();
    match rows {
        Ok(iter) => {
            for row in iter {
                let (rule_id, predicate_json, status) = match row {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("rules_load: row read error: {e}");
                        continue;
                    }
                };

                let stage = match status.as_str() {
                    "ACTIVE" => RuleLifecycleStage::Active,
                    "SHADOW" => RuleLifecycleStage::Shadow,
                    _ => continue,
                };

                let pred: Value = match serde_json::from_str(&predicate_json) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("rules_load: rule {} predicate parse failed: {e}", rule_id);
                        continue;
                    }
                };

                let tool_name = match pred.get("tool_name").and_then(|v| v.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        warn!("rules_load: rule {} missing tool_name", rule_id);
                        continue;
                    }
                };

                let required_args: Vec<String> = pred
                    .get("required_args")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|a| a.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                rules.push(ProtocolRule {
                    id: rule_id,
                    tool_name,
                    required_args,
                    stage,
                });
            }
        }
        Err(e) => {
            warn!("rules_load: query execution failed: {e}");
        }
    }
    rules
}

pub fn load_action_gate(project_root: &Path) -> ActionGate {
    let db_path = rules_db_path(project_root);
    let conn = match open_rules_db(&db_path, &OsClock) {
        Ok(c) => c,
        Err(e) => {
            warn!("rules_load: open rules.db at {} failed: {e}", db_path.display());
            return ActionGate::empty();
        }
    };
    let rules = load_protocol_rules_from_db(&conn);
    ActionGate::new(rules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use elai_rules::schema::open_rules_db;
    use rusqlite::params;
    use std::path::Path;

    fn open_test_db() -> Connection {
        open_rules_db(Path::new(":memory:"), &SeededClock::new())
            .expect("in-memory DB should open")
    }

    fn insert_rule(
        conn: &Connection,
        rule_id: &str,
        domain: &str,
        status: &str,
        predicate: &str,
    ) {
        conn.execute(
            "INSERT INTO rules (rule_id, domain, tier, status, title, description, \
             predicate, severity, mutable, created_at, source) \
             VALUES (?1, ?2, 1, ?3, 'Test', 'desc', ?4, 'WARN', 1, 1000, 'SEED')",
            params![rule_id, domain, status, predicate],
        )
        .unwrap();
    }

    #[test]
    fn empty_db_returns_empty_gate() {
        let conn = open_test_db();
        let rules = load_protocol_rules_from_db(&conn);
        assert!(rules.is_empty());
    }

    #[test]
    fn active_rule_missing_arg_blocks() {
        let conn = open_test_db();
        insert_rule(
            &conn,
            "r1",
            "intervention",
            "ACTIVE",
            r#"{"type":"protocol_gate","tool_name":"write_file","required_args":["path","content"]}"#,
        );
        let rules = load_protocol_rules_from_db(&conn);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].stage, RuleLifecycleStage::Active);
        let gate = ActionGate::new(rules);
        let v = gate.check("write_file", r#"{"path":"/tmp/x"}"#);
        match v {
            crate::intervention::GateVerdict::Block { rule_id, .. } => {
                assert_eq!(rule_id, "r1");
            }
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn shadow_rule_exec_not_block() {
        let conn = open_test_db();
        insert_rule(
            &conn,
            "r2",
            "intervention",
            "SHADOW",
            r#"{"type":"protocol_gate","tool_name":"write_file","required_args":["path","content"]}"#,
        );
        let rules = load_protocol_rules_from_db(&conn);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].stage, RuleLifecycleStage::Shadow);
        let gate = ActionGate::new(rules);
        assert_eq!(
            gate.check("write_file", r#"{"path":"/tmp/x"}"#),
            crate::intervention::GateVerdict::Exec
        );
    }

    #[test]
    fn non_intervention_domain_skipped() {
        let conn = open_test_db();
        insert_rule(
            &conn,
            "r3",
            "trading",
            "ACTIVE",
            r#"{"type":"protocol_gate","tool_name":"write_file","required_args":["path"]}"#,
        );
        let rules = load_protocol_rules_from_db(&conn);
        assert!(rules.is_empty());
    }

    #[test]
    fn malformed_predicate_skipped() {
        let conn = open_test_db();
        insert_rule(&conn, "r4", "intervention", "ACTIVE", "not json");
        let rules = load_protocol_rules_from_db(&conn);
        assert!(rules.is_empty());
    }
}
