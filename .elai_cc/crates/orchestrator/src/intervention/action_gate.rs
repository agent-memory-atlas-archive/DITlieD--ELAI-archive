// J-12 H2: runtime intervention harness action gate.
// Deterministic pre-execution protocol check on tool calls. Never LLM.
// A GateVerdict is Exec or Block{rule_id, message}; J-67 will upgrade this
// enum to a richer GateAction. Retired/Proposed/Candidate rules are ignored
// (not live). Shadow rules never Block (observe-only). Active rules Block
// only on CLEAR protocol violations (missing required arg, tool name
// mismatch for a matched rule). Ambiguous-but-possibly-correct actions are
// NEVER blocked (paper hard constraint, J-12 stub_ban).

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateVerdict {
    Exec,
    Block { rule_id: String, message: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleLifecycleStage {
    Proposed,
    Candidate,
    Shadow,
    Active,
    Retired,
}

#[derive(Debug, Clone)]
pub struct ProtocolRule {
    pub id: String,
    pub tool_name: String,
    pub required_args: Vec<String>,
    pub stage: RuleLifecycleStage,
}

pub struct ActionGate {
    rules: Vec<ProtocolRule>,
}

impl ActionGate {
    pub fn new(rules: Vec<ProtocolRule>) -> Self {
        Self { rules }
    }

    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn check(&self, tool_name: &str, args_json: &str) -> GateVerdict {
        for rule in &self.rules {
            if rule.tool_name != tool_name {
                continue;
            }
            match rule.stage {
                RuleLifecycleStage::Proposed
                | RuleLifecycleStage::Candidate
                | RuleLifecycleStage::Retired => continue,
                RuleLifecycleStage::Shadow => continue,
                RuleLifecycleStage::Active => {
                    let parsed: Result<Value, _> = serde_json::from_str(args_json);
                    match parsed {
                        Ok(Value::Object(map)) => {
                            for req in &rule.required_args {
                                if !map.contains_key(req) {
                                    return GateVerdict::Block {
                                        rule_id: rule.id.clone(),
                                        message: format!(
                                            "missing required argument: {}",
                                            req
                                        ),
                                    };
                                }
                            }
                        }
                        _ => {
                            return GateVerdict::Block {
                                rule_id: rule.id.clone(),
                                message: "malformed arguments: expected json object".to_string(),
                            };
                        }
                    }
                }
            }
        }
        GateVerdict::Exec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_rule(id: &str, tool: &str, args: &[&str]) -> ProtocolRule {
        ProtocolRule {
            id: id.to_string(),
            tool_name: tool.to_string(),
            required_args: args.iter().map(|s| s.to_string()).collect(),
            stage: RuleLifecycleStage::Active,
        }
    }

    fn shadow_rule(id: &str, tool: &str, args: &[&str]) -> ProtocolRule {
        ProtocolRule {
            id: id.to_string(),
            tool_name: tool.to_string(),
            required_args: args.iter().map(|s| s.to_string()).collect(),
            stage: RuleLifecycleStage::Shadow,
        }
    }

    #[test]
    fn empty_gate_always_exec() {
        let gate = ActionGate::empty();
        assert_eq!(gate.check("any_tool", "{}"), GateVerdict::Exec);
        assert_eq!(gate.check("any_tool", "garbage"), GateVerdict::Exec);
    }

    #[test]
    fn active_rule_missing_arg_blocks() {
        let gate = ActionGate::new(vec![active_rule("r1", "write_file", &["path", "content"])]);
        let v = gate.check("write_file", r#"{"path": "/tmp/x"}"#);
        match v {
            GateVerdict::Block { rule_id, .. } => assert_eq!(rule_id, "r1"),
            _ => panic!("expected Block"),
        }
    }

    #[test]
    fn shadow_rule_missing_arg_exec() {
        let gate = ActionGate::new(vec![shadow_rule("r2", "write_file", &["path", "content"])]);
        assert_eq!(gate.check("write_file", r#"{"path": "/tmp/x"}"#), GateVerdict::Exec);
    }

    #[test]
    fn active_rule_all_args_present_exec() {
        let gate = ActionGate::new(vec![active_rule("r3", "write_file", &["path", "content"])]);
        assert_eq!(
            gate.check("write_file", r#"{"path": "/tmp/x", "content": "hi"}"#),
            GateVerdict::Exec
        );
    }

    #[test]
    fn wrong_tool_name_does_not_apply_rule() {
        let gate = ActionGate::new(vec![active_rule("r4", "write_file", &["path"])]);
        assert_eq!(gate.check("read_file", r#"{}"#), GateVerdict::Exec);
    }
}
