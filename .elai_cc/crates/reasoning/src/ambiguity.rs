//! B-38: Continuous ambiguity-score metric (ouroboros port).
//!
//! Source: `github.com/Q00/ouroboros src/ouroboros/bigbang/ambiguity.py` (MIT).
//! Interface map: `workflow/papers/ouroboros-ambiguity-interface.md`.
//!
//! Provides:
//! - constants (weights, floors, threshold, streak requirement) ported verbatim,
//! - [`AmbiguityScore`] / [`ScoreBreakdown`] / [`ComponentScore`] value types,
//! - [`Milestone`] enum,
//! - [`AmbiguityScorer`] service that calls an [`elai_providers::LlmProvider`]
//!   and parses a JSON scoring response,
//! - [`AmbiguityError`] thiserror enum (provider failure + JSON parse failures).
//!
//! Backwards-compat: GroundingSweepGate gracefully degrades to B-17 behaviour
//! when `ambiguity_score` is `None` (see `elai_orchestrator::evidence`).

use std::sync::Arc;

use elai_providers::{LlmProvider, OllamaConfig};
use regex::Regex;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Constants — ported verbatim from ouroboros ambiguity.py.
// ---------------------------------------------------------------------------

/// Ambiguity gate threshold. `overall_score <= AMBIGUITY_THRESHOLD` -> READY.
pub const AMBIGUITY_THRESHOLD: f32 = 0.2;

/// Consecutive READY snapshots required before auto-completion can fire.
pub const AUTO_COMPLETE_STREAK_REQUIRED: usize = 2;

/// Greenfield: minimum goal-clarity component score.
pub const GOAL_CLARITY_FLOOR: f32 = 0.75;
/// Greenfield: minimum constraint-clarity component score.
pub const CONSTRAINT_CLARITY_FLOOR: f32 = 0.65;
/// Greenfield: minimum success-criteria-clarity component score.
pub const SUCCESS_CRITERIA_CLARITY_FLOOR: f32 = 0.70;
/// Brownfield-only: minimum context-clarity component score.
pub const BROWNFIELD_CONTEXT_CLARITY_FLOOR: f32 = 0.60;

/// LLM scoring temperature (low for reproducibility — by design, do not raise).
pub const SCORING_TEMPERATURE: f32 = 0.1;

// Greenfield weights (must sum to 1.0).
pub const GOAL_CLARITY_WEIGHT: f32 = 0.40;
pub const CONSTRAINT_CLARITY_WEIGHT: f32 = 0.30;
pub const SUCCESS_CRITERIA_CLARITY_WEIGHT: f32 = 0.30;

// Brownfield weights (must sum to 1.0).
pub const BROWNFIELD_GOAL_CLARITY_WEIGHT: f32 = 0.35;
pub const BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT: f32 = 0.25;
pub const BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT: f32 = 0.25;
pub const BROWNFIELD_CONTEXT_CLARITY_WEIGHT: f32 = 0.15;

/// Default initial max-tokens budget for adaptive token allocation.
pub const DEFAULT_INITIAL_MAX_TOKENS: u32 = 2048;
/// Default cap on truncation-triggered doublings.
pub const DEFAULT_MAX_TRUNCATION_DOUBLINGS: u32 = 5;
/// Default cap on parse-error retries.
pub const DEFAULT_MAX_FORMAT_ERROR_RETRIES: u32 = 5;

// ---------------------------------------------------------------------------
// Public helpers (also used by contract tests)
// ---------------------------------------------------------------------------

/// Clamp `x` into `[0.0, 1.0]`. NaN maps to 0.0.
#[must_use]
pub fn clamp01(x: f32) -> f32 {
    if x.is_nan() {
        0.0
    } else {
        x.clamp(0.0, 1.0)
    }
}

/// Round to 4 decimal places. Equivalent to ouroboros `round(value, 4)`.
#[must_use]
pub fn round_to_4dp(x: f32) -> f32 {
    (x * 10_000.0).round() / 10_000.0
}

// ---------------------------------------------------------------------------
// Value types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentScore {
    pub name: String,
    pub clarity_score: f32,
    pub weight: f32,
    pub justification: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoreBreakdown {
    pub goal_clarity: ComponentScore,
    pub constraint_clarity: ComponentScore,
    pub success_criteria_clarity: ComponentScore,
    /// `None` for greenfield; `Some(_)` for brownfield.
    pub context_clarity: Option<ComponentScore>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AmbiguityScore {
    /// `1.0 - sum(clarity_i * weight_i)`, rounded to 4 decimal places.
    pub overall_score: f32,
    pub breakdown: ScoreBreakdown,
}

/// Milestone bands. Tightest-first matching: `<=0.20`/Ready, `<=0.30`/Refined,
/// `<=0.40`/Progress, else /Initial.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Milestone {
    Initial,
    Progress,
    Refined,
    Ready,
}

impl AmbiguityScore {
    /// `overall_score <= AMBIGUITY_THRESHOLD`.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.overall_score <= AMBIGUITY_THRESHOLD
    }

    /// Return the per-dimension component names whose clarity score fell below
    /// their floor. Brownfield additionally checks `context_clarity`.
    #[must_use]
    pub fn floor_failures(&self, brownfield: bool) -> Vec<String> {
        let mut out = Vec::new();
        if self.breakdown.goal_clarity.clarity_score < GOAL_CLARITY_FLOOR {
            out.push("goal_clarity".to_string());
        }
        if self.breakdown.constraint_clarity.clarity_score < CONSTRAINT_CLARITY_FLOOR {
            out.push("constraint_clarity".to_string());
        }
        if self.breakdown.success_criteria_clarity.clarity_score
            < SUCCESS_CRITERIA_CLARITY_FLOOR
        {
            out.push("success_criteria_clarity".to_string());
        }
        if brownfield {
            if let Some(ctx) = &self.breakdown.context_clarity {
                if ctx.clarity_score < BROWNFIELD_CONTEXT_CLARITY_FLOOR {
                    out.push("context_clarity".to_string());
                }
            }
        }
        out
    }

    /// Both threshold AND no floor failures.
    #[must_use]
    pub fn qualifies_for_advance(&self, brownfield: bool) -> bool {
        self.is_ready() && self.floor_failures(brownfield).is_empty()
    }

    /// Milestone band lookup. Tightest-first iteration to honor ouroboros
    /// gotcha #1 (don't shadow READY with INITIAL).
    #[must_use]
    pub fn milestone(&self) -> Milestone {
        let s = self.overall_score;
        if s <= 0.20 {
            Milestone::Ready
        } else if s <= 0.30 {
            Milestone::Refined
        } else if s <= 0.40 {
            Milestone::Progress
        } else {
            Milestone::Initial
        }
    }
}

// ---------------------------------------------------------------------------
// AmbiguityError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AmbiguityError {
    #[error("llm provider error: {0}")]
    LlmError(String),

    #[error("json parse error after {retries} retries: {reason}")]
    ParseError { retries: u32, reason: String },

    #[error("missing required field in scorer response: {name}")]
    MissingField { name: String },

    #[error("scorer budget exhausted after {retries} retries")]
    BudgetExhausted { retries: u32 },
}

// ---------------------------------------------------------------------------
// AmbiguityScorer
// ---------------------------------------------------------------------------

/// LLM-driven scorer.
///
/// Note on truncation handling: the `LlmProvider` trait in
/// `crates/providers/src/providers/ollama.rs` does not yet expose a max-tokens
/// knob, so this scorer tracks `current_max_tokens` as a *budgeting* signal
/// only — it doubles on parse failures that look like truncation but cannot
/// actually constrain the provider. Provider-side max-tokens plumb-through is
/// tracked as a B-38 follow-up. See `workflow/reasoning/B-38-decisions.md`
/// decision #3.
pub struct AmbiguityScorer {
    llm: Arc<dyn LlmProvider>,
    model: String,
    temperature: f32,
    initial_max_tokens: u32,
    max_truncation_doublings: u32,
    max_format_error_retries: u32,
}

impl AmbiguityScorer {
    #[must_use]
    pub fn new(llm: Arc<dyn LlmProvider>, model: String) -> Self {
        Self {
            llm,
            model,
            temperature: SCORING_TEMPERATURE,
            initial_max_tokens: DEFAULT_INITIAL_MAX_TOKENS,
            max_truncation_doublings: DEFAULT_MAX_TRUNCATION_DOUBLINGS,
            max_format_error_retries: DEFAULT_MAX_FORMAT_ERROR_RETRIES,
        }
    }

    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Score a plan. Returns an [`AmbiguityScore`] or [`AmbiguityError`].
    ///
    /// `deferrals` are passed to the system prompt with an instruction that
    /// they MUST NOT penalise clarity (matches ouroboros invariant).
    pub async fn score(
        &self,
        plan_text: &str,
        grounding_summary: &str,
        brownfield: bool,
        deferrals: &[String],
    ) -> Result<AmbiguityScore, AmbiguityError> {
        let prompt = build_prompt(plan_text, grounding_summary, brownfield, deferrals);
        let config = OllamaConfig {
            temperature: self.temperature,
            ..OllamaConfig::default()
        };

        let mut format_error_count: u32 = 0;
        let mut truncation_count: u32 = 0;
        let mut current_max_tokens = self.initial_max_tokens;
        let mut last_reason = String::from("no response received");

        loop {
            if format_error_count > self.max_format_error_retries {
                return Err(AmbiguityError::ParseError {
                    retries: format_error_count,
                    reason: last_reason,
                });
            }
            if truncation_count > self.max_truncation_doublings {
                return Err(AmbiguityError::BudgetExhausted {
                    retries: truncation_count,
                });
            }

            tracing::debug!(
                model = %self.model,
                temperature = self.temperature,
                current_max_tokens,
                "ambiguity_scorer: calling llm.complete"
            );

            let chunks = self
                .llm
                .complete(&prompt, &config)
                .await
                .map_err(|e| AmbiguityError::LlmError(format!("{e}")))?;
            let raw = chunks.join("");

            match parse_response(&raw, brownfield) {
                Ok(score) => return Ok(score),
                Err(ParseStep::MissingField { name }) => {
                    // Hard error per ouroboros gotcha #3.
                    return Err(AmbiguityError::MissingField { name });
                }
                Err(ParseStep::Truncated { reason }) => {
                    truncation_count = truncation_count.saturating_add(1);
                    current_max_tokens = current_max_tokens.saturating_mul(2);
                    last_reason = reason;
                    tracing::warn!(
                        truncation_count,
                        current_max_tokens,
                        reason = %last_reason,
                        "ambiguity_scorer: response looks truncated; retrying with larger budget"
                    );
                    continue;
                }
                Err(ParseStep::Malformed { reason }) => {
                    format_error_count = format_error_count.saturating_add(1);
                    last_reason = reason;
                    tracing::warn!(
                        format_error_count,
                        reason = %last_reason,
                        "ambiguity_scorer: parse failed; retrying"
                    );
                    continue;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Prompt construction
// ---------------------------------------------------------------------------

const DEFERRAL_INSTRUCTION: &str =
    "Items in DEFERRALS are decide-later and MUST NOT penalize clarity.";

fn build_prompt(
    plan_text: &str,
    grounding_summary: &str,
    brownfield: bool,
    deferrals: &[String],
) -> String {
    let mode = if brownfield { "brownfield" } else { "greenfield" };
    let mut s = String::with_capacity(plan_text.len() + grounding_summary.len() + 1024);
    s.push_str("[SYSTEM]\nYou are an ambiguity scorer. ");
    s.push_str(DEFERRAL_INSTRUCTION);
    s.push_str(" Mode: ");
    s.push_str(mode);
    s.push_str(".\n\n[PLAN]\n");
    s.push_str(plan_text);
    s.push_str("\n\n[GROUNDING]\n");
    s.push_str(grounding_summary);
    s.push_str("\n\n[DEFERRALS]\n");
    if deferrals.is_empty() {
        s.push_str("(none)");
    } else {
        for d in deferrals {
            s.push_str("- ");
            s.push_str(d);
            s.push('\n');
        }
    }
    s.push_str(
        "\n\nReturn a JSON object with goal_clarity / constraint_clarity / \
         success_criteria_clarity (and context_clarity for brownfield), each \
         with clarity_score in [0,1] and a justification string.",
    );
    s
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

#[derive(Debug)]
enum ParseStep {
    /// Hard error — missing a required `clarity_score` field. No retry.
    MissingField { name: String },
    /// Soft error — looks like the response was truncated mid-JSON.
    Truncated { reason: String },
    /// Soft error — looks malformed in a non-truncation way (re-prompt).
    Malformed { reason: String },
}

fn parse_response(raw: &str, brownfield: bool) -> Result<AmbiguityScore, ParseStep> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ParseStep::Truncated {
            reason: "empty response".to_string(),
        });
    }

    let json_slice = extract_json_slice(trimmed).ok_or_else(|| ParseStep::Malformed {
        reason: "no JSON object found in response".to_string(),
    })?;

    // Truncation heuristic: number of unmatched `{` > `}` in the candidate
    // slice strongly suggests the response was cut off.
    if unbalanced_braces(&json_slice) {
        return Err(ParseStep::Truncated {
            reason: "unbalanced braces (response likely truncated)".to_string(),
        });
    }

    let value: serde_json::Value = match serde_json::from_str(&json_slice) {
        Ok(v) => v,
        Err(e) => {
            return Err(ParseStep::Malformed {
                reason: format!("serde_json: {e}"),
            });
        }
    };

    let goal = read_component(&value, "goal_clarity")?;
    let constraint = read_component(&value, "constraint_clarity")?;
    let success = read_component(&value, "success_criteria_clarity")?;
    let context = if brownfield {
        Some(read_component(&value, "context_clarity")?)
    } else {
        None
    };

    let (gw, cw, sw, ctw) = if brownfield {
        (
            BROWNFIELD_GOAL_CLARITY_WEIGHT,
            BROWNFIELD_CONSTRAINT_CLARITY_WEIGHT,
            BROWNFIELD_SUCCESS_CRITERIA_CLARITY_WEIGHT,
            BROWNFIELD_CONTEXT_CLARITY_WEIGHT,
        )
    } else {
        (
            GOAL_CLARITY_WEIGHT,
            CONSTRAINT_CLARITY_WEIGHT,
            SUCCESS_CRITERIA_CLARITY_WEIGHT,
            0.0,
        )
    };

    let goal_score = clamp01(goal.0);
    let constraint_score = clamp01(constraint.0);
    let success_score = clamp01(success.0);
    let context_score = context.as_ref().map(|c| clamp01(c.0)).unwrap_or(0.0);

    let weighted =
        goal_score * gw + constraint_score * cw + success_score * sw + context_score * ctw;
    let overall = round_to_4dp(1.0 - weighted);

    Ok(AmbiguityScore {
        overall_score: overall,
        breakdown: ScoreBreakdown {
            goal_clarity: ComponentScore {
                name: "goal_clarity".to_string(),
                clarity_score: goal_score,
                weight: gw,
                justification: goal.1,
            },
            constraint_clarity: ComponentScore {
                name: "constraint_clarity".to_string(),
                clarity_score: constraint_score,
                weight: cw,
                justification: constraint.1,
            },
            success_criteria_clarity: ComponentScore {
                name: "success_criteria_clarity".to_string(),
                clarity_score: success_score,
                weight: sw,
                justification: success.1,
            },
            context_clarity: context.map(|(_v, j)| ComponentScore {
                name: "context_clarity".to_string(),
                clarity_score: context_score,
                weight: ctw,
                justification: j,
            }),
        },
    })
}

/// Returns `(clarity_score, justification)` for `component_name`.
/// `MissingField` is propagated when `clarity_score` is absent (hard error).
fn read_component(
    value: &serde_json::Value,
    component_name: &str,
) -> Result<(f32, String), ParseStep> {
    let obj = value.get(component_name).ok_or_else(|| ParseStep::MissingField {
        name: component_name.to_string(),
    })?;
    let score_v = obj.get("clarity_score").ok_or_else(|| ParseStep::MissingField {
        name: format!("{component_name}.clarity_score"),
    })?;
    let score = score_v.as_f64().ok_or_else(|| ParseStep::MissingField {
        name: format!("{component_name}.clarity_score (non-numeric)"),
    })?;
    // Justification is soft: missing -> fallback string.
    let just = obj
        .get("justification")
        .and_then(|j| j.as_str())
        .map_or_else(
            || format!("{component_name} justification not provided by model."),
            std::string::ToString::to_string,
        );
    // NARROWING-OK: clamping to [0,1] is performed by callers; f64->f32 cast
    // is bounded by the subsequent clamp01().
    #[allow(clippy::cast_possible_truncation)]
    Ok((score as f32, just))
}

fn extract_json_slice(raw: &str) -> Option<String> {
    // Step 1: fenced ```json ... ``` (or ``` ... ```).
    // Note: regex is OnceLock-cached in production code paths but for B-38 we
    // build on every call. Scorer is invoked at most a handful of times per
    // gate evaluation; if profiling shows hot-path cost, cache via OnceLock.
    let fenced = Regex::new(r"(?s)```(?:json)?\s*(\{.*?\})\s*```").ok()?;
    if let Some(cap) = fenced.captures(raw) {
        if let Some(m) = cap.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    // Step 2: greedy first-`{` to last-`}` slice.
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end >= start {
        Some(raw[start..=end].to_string())
    } else {
        None
    }
}

fn unbalanced_braces(s: &str) -> bool {
    let mut open: i64 = 0;
    let mut in_str = false;
    let mut escape = false;
    for c in s.chars() {
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_str => escape = true,
            '"' => in_str = !in_str,
            '{' if !in_str => open = open.saturating_add(1),
            '}' if !in_str => open = open.saturating_sub(1),
            _ => {}
        }
    }
    open != 0
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ambiguity_zero_when_all_clarity_one() {
        let s = AmbiguityScore {
            overall_score: round_to_4dp(
                1.0 - (1.0 * GOAL_CLARITY_WEIGHT
                    + 1.0 * CONSTRAINT_CLARITY_WEIGHT
                    + 1.0 * SUCCESS_CRITERIA_CLARITY_WEIGHT),
            ),
            breakdown: ScoreBreakdown {
                goal_clarity: ComponentScore {
                    name: "goal_clarity".into(),
                    clarity_score: 1.0,
                    weight: GOAL_CLARITY_WEIGHT,
                    justification: "j".into(),
                },
                constraint_clarity: ComponentScore {
                    name: "constraint_clarity".into(),
                    clarity_score: 1.0,
                    weight: CONSTRAINT_CLARITY_WEIGHT,
                    justification: "j".into(),
                },
                success_criteria_clarity: ComponentScore {
                    name: "success_criteria_clarity".into(),
                    clarity_score: 1.0,
                    weight: SUCCESS_CRITERIA_CLARITY_WEIGHT,
                    justification: "j".into(),
                },
                context_clarity: None,
            },
        };
        assert!(s.is_ready());
        assert!(s.floor_failures(false).is_empty());
        assert!(s.qualifies_for_advance(false));
        assert!(matches!(s.milestone(), Milestone::Ready));
    }

    #[test]
    fn clamp01_handles_nan_inf_and_bounds() {
        assert_eq!(clamp01(f32::NAN), 0.0);
        assert_eq!(clamp01(f32::INFINITY), 1.0);
        assert_eq!(clamp01(f32::NEG_INFINITY), 0.0);
        assert_eq!(clamp01(0.5), 0.5);
    }

    #[test]
    fn extract_json_slice_handles_fenced_and_raw() {
        let fenced = "blah\n```json\n{\"a\":1}\n```\nblah";
        assert_eq!(extract_json_slice(fenced).as_deref(), Some("{\"a\":1}"));
        let raw = "garbage{\"b\":2}garbage";
        assert_eq!(extract_json_slice(raw).as_deref(), Some("{\"b\":2}"));
        assert!(extract_json_slice("no braces").is_none());
    }

    #[test]
    fn unbalanced_braces_detects_truncation() {
        assert!(!unbalanced_braces("{\"a\":1}"));
        assert!(unbalanced_braces("{\"a\":1"));
        assert!(unbalanced_braces("{\"a\":\"}\""), "string-escaped brace should not count");
    }

    #[test]
    fn parse_missing_score_is_hard_error() {
        let raw = r#"{"goal_clarity":{"justification":"x"},"constraint_clarity":{"clarity_score":0.8,"justification":"y"},"success_criteria_clarity":{"clarity_score":0.8,"justification":"z"}}"#;
        let res = parse_response(raw, false);
        match res {
            Err(ParseStep::MissingField { name }) => {
                assert!(name.contains("goal_clarity"), "got {name}");
            }
            other => panic!("expected MissingField, got {other:?}"),
        }
    }

    #[test]
    fn parse_missing_justification_is_soft_fallback() {
        let raw = r#"{"goal_clarity":{"clarity_score":0.9},"constraint_clarity":{"clarity_score":0.8,"justification":"y"},"success_criteria_clarity":{"clarity_score":0.85,"justification":"z"}}"#;
        let s = parse_response(raw, false).expect("should parse despite missing justification");
        assert!(s.breakdown.goal_clarity.justification.contains("not provided"));
    }
}
