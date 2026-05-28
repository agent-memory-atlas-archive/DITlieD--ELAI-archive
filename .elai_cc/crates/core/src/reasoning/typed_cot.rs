//! E-19 task 6 — Typed-CoT triples emitted by L2 (re-sample-with-prior).
//!
//! Paper grounding: arXiv:2510.01069 (Typed Chain-of-Thought, Curry-Howard
//! framework). The 50.3pp lift reported on GSM8K via Certified Self-
//! Consistency is NOT a math-eval claim for ELAI; we adopt the typed-CoT
//! frame purely as a wiring-quality gain (one triple per L2 output gives the
//! auditor `{rule_applied, types_in, types_out}` to validate against the
//! WIRING_CONTRACT block).
//!
//! IO-free. The XGrammar JSON schema lives in `JSON_SCHEMA` and is consumed
//! by the E-18 structured-decoding pipeline at L2 inference time.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Wire-format schema version. Bumped on any breaking shape change to
/// [`TypedCotTriple`]. Co-versioned with SCHEMA_VERSION 166 (the E-19
/// SSB bump) so dashboard / replay consumers can sanity-check.
pub const TYPED_COT_SCHEMA_VERSION: u32 = 1;

/// Soft cap on the number of types listed in either `types_in` or
/// `types_out`. Caller MUST truncate; deserializer accepts longer lists
/// for forward-compat but the auditor MAY refuse.
pub const MAX_TYPES_PER_SIDE: usize = 32;

/// One typed CoT certificate emitted by L2 micro-reflection (re-sample-with-
/// prior path).
///
/// `rule_applied` is the canonical rule slug (e.g. `match_arm_added`,
/// `unwrap_replaced_with_ok_or`); `types_in` / `types_out` are the input /
/// output type names in canonical form (Rust path syntax for Rust changes;
/// the corresponding canonical form for other languages).
///
/// The orchestrator wrapper serializes this struct directly into the
/// `ReasoningQualityProbe { kind: ProbeKind::L2TypedCotEmitted }` SSB row
/// payload extras.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypedCotTriple {
    /// Canonical rule slug, snake_case. Empty string = unspecified.
    pub rule_applied: String,
    /// Input types, canonical form. Ordered (auditor may diff position).
    pub types_in: Vec<String>,
    /// Output types, canonical form. Ordered.
    pub types_out: Vec<String>,
    /// Optional extras — kept BTreeMap for deterministic ordering; never
    /// HashMap (ELAI no-HashMap rule). Reserved for forward-compat.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extras: BTreeMap<String, String>,
}

impl TypedCotTriple {
    /// Build a triple from raw slug + type lists, truncating at
    /// [`MAX_TYPES_PER_SIDE`] (defense-in-depth; XGrammar should already
    /// constrain at decode time).
    #[must_use]
    pub fn new(rule_applied: impl Into<String>, types_in: Vec<String>, types_out: Vec<String>) -> Self {
        Self {
            rule_applied: rule_applied.into(),
            types_in: truncate(types_in),
            types_out: truncate(types_out),
            extras: BTreeMap::new(),
        }
    }

    /// True when all three required fields are populated. The auditor
    /// rejects partial triples.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        !self.rule_applied.is_empty() && !self.types_in.is_empty() && !self.types_out.is_empty()
    }
}

fn truncate(mut v: Vec<String>) -> Vec<String> {
    if v.len() > MAX_TYPES_PER_SIDE {
        v.truncate(MAX_TYPES_PER_SIDE);
    }
    v
}

/// JSON schema for XGrammar-constrained decoding at L2 inference time.
/// E-18 ships the XGrammar primitive; E-19 pins this schema string so the
/// L2 prompt + decoder agree on the wire shape.
pub const JSON_SCHEMA: &str = r#"{
  "type": "object",
  "additionalProperties": false,
  "required": ["rule_applied", "types_in", "types_out"],
  "properties": {
    "rule_applied": {"type": "string", "minLength": 1},
    "types_in": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 32},
    "types_out": {"type": "array", "items": {"type": "string"}, "minItems": 1, "maxItems": 32},
    "extras": {"type": "object", "additionalProperties": {"type": "string"}}
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triple_round_trip_via_serde() {
        let t = TypedCotTriple::new(
            "match_arm_added",
            vec!["FsmState".into()],
            vec!["FsmState".into(), "Result<()>".into()],
        );
        let s = serde_json::to_string(&t).expect("serialize");
        let back: TypedCotTriple = serde_json::from_str(&s).expect("deserialize");
        assert_eq!(t, back);
    }

    #[test]
    fn triple_complete_predicate() {
        let t = TypedCotTriple::new("rule", vec!["A".into()], vec!["B".into()]);
        assert!(t.is_complete());
        let bad = TypedCotTriple::new("", vec!["A".into()], vec!["B".into()]);
        assert!(!bad.is_complete());
    }

    #[test]
    fn truncates_at_cap() {
        let big: Vec<String> = (0..100).map(|i| format!("T{i}")).collect();
        let t = TypedCotTriple::new("x", big.clone(), big);
        assert_eq!(t.types_in.len(), MAX_TYPES_PER_SIDE);
        assert_eq!(t.types_out.len(), MAX_TYPES_PER_SIDE);
    }

    #[test]
    fn schema_is_parseable_json() {
        let v: serde_json::Value = serde_json::from_str(JSON_SCHEMA).expect("schema is valid JSON");
        assert_eq!(v["type"], "object");
    }
}
