//! Deterministic compact JSON emission for J-82.
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub fn estimate_tokens(s: &str) -> u32 { ((s.chars().count() as u32).saturating_add(3)) / 4 }

fn sorted(v: &Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut out = Map::new();
            let sorted: BTreeMap<_, _> = m.iter().map(|(k, v)| (k, v)).collect();
            for (k, child) in sorted { out.insert(k.clone(), sorted(child)); }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(sorted).collect()),
        _ => v.clone(),
    }
}

/// Emit deterministic, whitespace-free ELAI compact JSON (keys are sorted).
pub fn emit_token_efficient(value: &Value) -> String {
    serde_json::to_string(&sorted(value)).unwrap_or_else(|_| "null".to_string())
}

pub fn parse_token_efficient(s: &str) -> Result<Value, String> {
    serde_json::from_str(s).map_err(|e| e.to_string())
}

pub fn maybe_compact_tool_output(raw: &str) -> String {
    let compacted = match serde_json::from_str::<Value>(raw) { Ok(v) => emit_token_efficient(&v), Err(_) => raw.to_string() };
    #[cfg(feature = "mcp")]
    {
        return elai_plugins::enforce_output_budget(&compacted, elai_plugins::DEFAULT_OUTPUT_BUDGET_TOKENS).0;
    }
    #[cfg(not(feature = "mcp"))]
    compacted
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn object_round_trip() {
        let v: Value = serde_json::json!({"z": 1, "a": [true, "x"]});
        assert_eq!(parse_token_efficient(&emit_token_efficient(&v)).ok(), Some(sorted(&v)));
    }
    #[test]
    fn array_round_trip() {
        let v: Value = serde_json::json!([{"b":2,"a":1},{"a":3,"b":4}]);
        assert_eq!(parse_token_efficient(&emit_token_efficient(&v)).ok(), Some(sorted(&v)));
    }
}
