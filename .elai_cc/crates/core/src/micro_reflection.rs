//! Layer 2 micro-reflection: one-shot LLM diagnosis for errors that
//! Layer 1 (deterministic taxonomy) cannot fully resolve.
//!
//! Protocol: assemble <=500 token context (goal + code + error + types),
//! send to local model with JSON-constrained output, store result in
//! MistakeBook. Temperature 0.3 via GenerationProfile::MicroReflection.
//!
//! Research grounding:
//!   ARLCP (ICLR 2026, arXiv:2602.12113): long reflection harms sub-10B.
//!   "First Try Matters" (arXiv:2510.08308): overturn-on-checking risk.

use serde::{Deserialize, Serialize};

use crate::mistake_book::{FailureCategory, MistakeEntry, DiffHash};

// ---------------------------------------------------------------------------
// Token budget constants (~4 chars per token approximation)
// ---------------------------------------------------------------------------

/// Total context budget for micro-reflection input (tokens).
pub const CONTEXT_BUDGET_TOKENS: usize = 500;

/// Budget allocation: baby step goal description.
pub const GOAL_BUDGET_TOKENS: usize = 50;

/// Budget allocation: failed code snippet (function body only).
pub const CODE_BUDGET_TOKENS: usize = 200;

/// Budget allocation: verbatim error message.
pub const ERROR_BUDGET_TOKENS: usize = 100;

/// Budget allocation: relevant type signatures from DKG.
pub const TYPES_BUDGET_TOKENS: usize = 100;

/// Chars-per-token approximation for budget enforcement.
pub const CHARS_PER_TOKEN: usize = 4;

/// Maximum output tokens for the constrained JSON response.
pub const MAX_OUTPUT_TOKENS: usize = 50;

/// Maximum words in the root_cause field.
pub const MAX_ROOT_CAUSE_WORDS: usize = 30;

/// Maximum words in the action field.
pub const MAX_ACTION_WORDS: usize = 20;

/// JSON schema version for forward compatibility.
pub const REFLECTION_SCHEMA_VERSION: u32 = 1;

// ---------------------------------------------------------------------------
// MicroReflectionRequest — input to Layer 2
// ---------------------------------------------------------------------------

/// Assembled context for a single micro-reflection LLM call.
/// Each field is pre-truncated to its token budget before construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroReflectionRequest {
    /// Baby step goal (what the code should accomplish), <=GOAL_BUDGET_TOKENS.
    pub baby_step_goal: String,
    /// Failed code snippet (function body only), <=CODE_BUDGET_TOKENS.
    pub code: String,
    /// Verbatim compiler/test error, <=ERROR_BUDGET_TOKENS.
    pub error: String,
    /// Relevant type signatures from the dependency knowledge graph.
    pub context_types: Vec<String>,
}

impl MicroReflectionRequest {
    /// Build a request with automatic truncation to budget.
    /// Each field is truncated to its char budget (tokens * CHARS_PER_TOKEN).
    pub fn assemble(
        baby_step_goal: &str,
        code: &str,
        error: &str,
        context_types: &[String],
    ) -> Self {
        Self {
            baby_step_goal: truncate_to_budget(baby_step_goal, GOAL_BUDGET_TOKENS),
            code: truncate_to_budget(code, CODE_BUDGET_TOKENS),
            error: truncate_to_budget(error, ERROR_BUDGET_TOKENS),
            context_types: pack_types(context_types, TYPES_BUDGET_TOKENS),
        }
    }

    /// Total estimated token count for this assembled context.
    pub fn estimated_tokens(&self) -> usize {
        let chars = self.baby_step_goal.len()
            + self.code.len()
            + self.error.len()
            + self.context_types.iter().map(|t| t.len()).sum::<usize>();
        // Ceiling division
        (chars + CHARS_PER_TOKEN - 1) / CHARS_PER_TOKEN
    }

    /// Format as the user-turn prompt sent to the local model.
    /// System prompt is fixed: "You are diagnosing a code error. Answer in JSON only."
    pub fn to_prompt(&self) -> String {
        let types_block = self.context_types.join("\n");
        format!(
            "Goal: {goal}\nCode: {code}\nError: {error}\nContext types:\n{types}",
            goal = self.baby_step_goal,
            code = self.code,
            error = self.error,
            types = types_block,
        )
    }

    /// The fixed system prompt for micro-reflection calls.
    pub fn system_prompt() -> &'static str {
        "You are diagnosing a code error. Answer in JSON only."
    }
}

// ---------------------------------------------------------------------------
// MicroReflectionResponse — JSON-constrained output from Layer 2
// ---------------------------------------------------------------------------

/// Structured diagnosis from micro-reflection. JSON-constrained output.
/// Compatible with constrained decoding (B-4, future).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MicroReflectionResponse {
    /// Root cause description, max 30 words.
    pub root_cause: String,
    /// Line number where the fault is localized.
    pub faulty_line: u32,
    /// Corrective action, max 20 words.
    pub action: String,
}

impl MicroReflectionResponse {
    /// Validate word count constraints. Returns first violation or Ok.
    pub fn validate(&self) -> Result<(), ReflectionError> {
        let rc_words = self.root_cause.split_whitespace().count();
        if rc_words > MAX_ROOT_CAUSE_WORDS {
            return Err(ReflectionError::RootCauseTooLong {
                words: rc_words,
                max: MAX_ROOT_CAUSE_WORDS,
            });
        }
        let action_words = self.action.split_whitespace().count();
        if action_words > MAX_ACTION_WORDS {
            return Err(ReflectionError::ActionTooLong {
                words: action_words,
                max: MAX_ACTION_WORDS,
            });
        }
        if self.root_cause.is_empty() {
            return Err(ReflectionError::EmptyField { field: "root_cause" });
        }
        if self.action.is_empty() {
            return Err(ReflectionError::EmptyField { field: "action" });
        }
        Ok(())
    }

    /// Convert to a MistakeEntry for storage in the MistakeBook.
    /// `attempt_number`: which retry attempt triggered this reflection.
    /// `diff`: the failed patch text (hashed for forbidden_diffs).
    /// `failure_category`: from Layer 1 taxonomy.
    /// `timestamp`: unix seconds from ClockProvider.
    pub fn to_mistake_entry(
        &self,
        attempt_number: u32,
        diff: &str,
        failure_category: FailureCategory,
        timestamp: u64,
    ) -> MistakeEntry {
        // Serialize the action only (no code snippets per CTIM-Rover finding).
        let root_cause_json = serde_json::json!({
            "action": &self.action
        })
        .to_string();

        MistakeEntry {
            attempt_number,
            diff_hash: DiffHash::from_diff(diff),
            failure_category,
            faulty_line: Some(self.faulty_line),
            root_cause_json: Some(root_cause_json),
            action_taken: "micro_reflection".to_string(),
            timestamp,
        }
    }

    /// JSON schema for constrained decoding (compatible with vLLM
    /// json_schema / grammar-guided generation). Returns the schema
    /// as a serde_json::Value for direct injection into generation params.
    pub fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "MicroReflectionResponse",
            "description": "Layer 2 micro-reflection diagnosis output",
            "type": "object",
            "properties": {
                "root_cause": {
                    "type": "string",
                    "description": "Root cause, max 30 words",
                    "maxLength": 300
                },
                "faulty_line": {
                    "type": "integer",
                    "minimum": 0,
                    "description": "Line number of the fault"
                },
                "action": {
                    "type": "string",
                    "description": "Corrective action, max 20 words",
                    "maxLength": 200
                }
            },
            "required": ["root_cause", "faulty_line", "action"],
            "additionalProperties": false
        })
    }
}

// ---------------------------------------------------------------------------
// ReflectionError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, thiserror::Error)]
pub enum ReflectionError {
    #[error("root_cause too long: {words} words (max {max})")]
    RootCauseTooLong { words: usize, max: usize },

    #[error("action too long: {words} words (max {max})")]
    ActionTooLong { words: usize, max: usize },

    #[error("empty field: {field}")]
    EmptyField { field: &'static str },

    #[error("JSON parse error: {reason}")]
    JsonParse { reason: String },

    #[error("context exceeds budget: {tokens} tokens (max {max})")]
    ContextOverBudget { tokens: usize, max: usize },
}

// ---------------------------------------------------------------------------
// Utility: truncation and packing helpers
// ---------------------------------------------------------------------------

/// Truncate a string to fit within a token budget (chars = tokens * CHARS_PER_TOKEN).
/// Truncates at char boundary, appends "..." if truncated.
fn truncate_to_budget(s: &str, token_budget: usize) -> String {
    let char_budget = token_budget * CHARS_PER_TOKEN;
    if s.len() <= char_budget {
        return s.to_string();
    }
    // Reserve 3 bytes for "..." suffix. Find the last char that ends
    // at or before max_content so total len (content + "...") <= char_budget.
    let max_content = char_budget.saturating_sub(3);
    let boundary = s
        .char_indices()
        .map(|(i, c)| i + c.len_utf8()) // end-of-char byte offset
        .take_while(|&end| end <= max_content)
        .last()
        .unwrap_or(0);
    let mut result = s[..boundary].to_string();
    result.push_str("...");
    result
}

/// Pack type signatures into the types budget, greedily.
/// Each type is included whole or not at all (no partial types).
fn pack_types(types: &[String], token_budget: usize) -> Vec<String> {
    let char_budget = token_budget * CHARS_PER_TOKEN;
    let mut packed = Vec::new();
    let mut used = 0;
    for t in types {
        // +1 for newline separator in prompt assembly
        let cost = t.len() + 1;
        if used + cost > char_budget {
            break;
        }
        packed.push(t.clone());
        used += cost;
    }
    packed
}

/// Parse a JSON string into a MicroReflectionResponse with validation.
pub fn parse_response(json_str: &str) -> Result<MicroReflectionResponse, ReflectionError> {
    let response: MicroReflectionResponse = serde_json::from_str(json_str)
        .map_err(|e| ReflectionError::JsonParse {
            reason: e.to_string(),
        })?;
    response.validate()?;
    Ok(response)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assemble_within_budget() {
        let req = MicroReflectionRequest::assemble(
            "implement fibonacci function",
            "fn fib(n: u32) -> u32 { n }",
            "assertion failed: fib(5) == 5, expected 5",
            &["fn fib(n: u32) -> u32".to_string()],
        );
        assert!(req.estimated_tokens() <= CONTEXT_BUDGET_TOKENS);
    }

    #[test]
    fn assemble_truncates_long_code() {
        let long_code = "x".repeat(CODE_BUDGET_TOKENS * CHARS_PER_TOKEN + 100);
        let req = MicroReflectionRequest::assemble("goal", &long_code, "err", &[]);
        assert!(req.code.len() <= CODE_BUDGET_TOKENS * CHARS_PER_TOKEN);
        assert!(req.code.ends_with("..."));
    }

    #[test]
    fn assemble_truncates_long_error() {
        let long_err = "e".repeat(ERROR_BUDGET_TOKENS * CHARS_PER_TOKEN + 50);
        let req = MicroReflectionRequest::assemble("goal", "code", &long_err, &[]);
        assert!(req.error.len() <= ERROR_BUDGET_TOKENS * CHARS_PER_TOKEN);
    }

    #[test]
    fn pack_types_respects_budget() {
        let types: Vec<String> = (0..20)
            .map(|i| format!("fn func_{i}(x: i32) -> Result<i32, Error>"))
            .collect();
        let packed = pack_types(&types, TYPES_BUDGET_TOKENS);
        let total_chars: usize = packed.iter().map(|t| t.len() + 1).sum();
        assert!(total_chars <= TYPES_BUDGET_TOKENS * CHARS_PER_TOKEN);
    }

    #[test]
    fn response_json_roundtrip() {
        let resp = MicroReflectionResponse {
            root_cause: "missing base case in recursion".to_string(),
            faulty_line: 3,
            action: "add if n <= 1 return n guard".to_string(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let back: MicroReflectionResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, back);
    }

    #[test]
    fn response_validation_passes() {
        let resp = MicroReflectionResponse {
            root_cause: "wrong operator".to_string(),
            faulty_line: 10,
            action: "change + to *".to_string(),
        };
        assert!(resp.validate().is_ok());
    }

    #[test]
    fn response_validation_rejects_long_root_cause() {
        let long_rc = (0..MAX_ROOT_CAUSE_WORDS + 5)
            .map(|_| "word")
            .collect::<Vec<_>>()
            .join(" ");
        let resp = MicroReflectionResponse {
            root_cause: long_rc,
            faulty_line: 1,
            action: "fix it".to_string(),
        };
        assert!(matches!(
            resp.validate(),
            Err(ReflectionError::RootCauseTooLong { .. })
        ));
    }

    #[test]
    fn response_validation_rejects_empty_action() {
        let resp = MicroReflectionResponse {
            root_cause: "some cause".to_string(),
            faulty_line: 1,
            action: String::new(),
        };
        assert!(matches!(
            resp.validate(),
            Err(ReflectionError::EmptyField { field: "action" })
        ));
    }

    #[test]
    fn parse_response_valid_json() {
        let json = r#"{"root_cause":"missing semicolon","faulty_line":42,"action":"add semicolon"}"#;
        let resp = parse_response(json).unwrap();
        assert_eq!(resp.faulty_line, 42);
        assert_eq!(resp.root_cause, "missing semicolon");
    }

    #[test]
    fn parse_response_invalid_json() {
        let bad = r#"{"root_cause": 123}"#;
        assert!(matches!(
            parse_response(bad),
            Err(ReflectionError::JsonParse { .. })
        ));
    }

    #[test]
    fn json_schema_is_valid() {
        let schema = MicroReflectionResponse::json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["root_cause"].is_object());
        assert!(schema["properties"]["faulty_line"].is_object());
        assert!(schema["properties"]["action"].is_object());
        assert_eq!(schema["additionalProperties"], false);
    }

    #[test]
    fn to_mistake_entry_integration() {
        let resp = MicroReflectionResponse {
            root_cause: "off by one in loop bound".to_string(),
            faulty_line: 15,
            action: "change < to <=".to_string(),
        };
        let entry = resp.to_mistake_entry(
            2,
            "--- a/lib.rs\n+++ b/lib.rs\n-old\n+new",
            FailureCategory::LogicError,
            1700000000,
        );
        assert_eq!(entry.attempt_number, 2);
        assert_eq!(entry.failure_category, FailureCategory::LogicError);
        assert_eq!(entry.faulty_line, Some(15));
        assert!(entry.root_cause_json.is_some());
        assert_eq!(entry.action_taken, "micro_reflection");
        // Verify the JSON contains action, not code
        let json: serde_json::Value =
            serde_json::from_str(entry.root_cause_json.as_ref().unwrap()).unwrap();
        assert_eq!(json["action"], "change < to <=");
    }

    #[test]
    fn to_prompt_format() {
        let req = MicroReflectionRequest::assemble(
            "add two numbers",
            "fn add(a: i32, b: i32) -> i32 { a - b }",
            "assertion failed: add(2, 3) != 5",
            &["fn add(a: i32, b: i32) -> i32".to_string()],
        );
        let prompt = req.to_prompt();
        assert!(prompt.contains("Goal: add two numbers"));
        assert!(prompt.contains("Code: fn add"));
        assert!(prompt.contains("Error: assertion failed"));
        assert!(prompt.contains("Context types:"));
    }

    #[test]
    fn truncate_to_budget_preserves_short_strings() {
        let s = "short string";
        assert_eq!(truncate_to_budget(s, 100), s);
    }

    #[test]
    fn truncate_to_budget_handles_utf8() {
        // Multi-byte chars: each is 3 bytes
        let s = "aaaa\u{00e9}\u{00e9}\u{00e9}"; // a a a a e-acute x3
        let result = truncate_to_budget(&s, 2); // 2 tokens = 8 chars budget
        // Should not panic on char boundaries
        assert!(result.len() <= 2 * CHARS_PER_TOKEN);
    }
}
