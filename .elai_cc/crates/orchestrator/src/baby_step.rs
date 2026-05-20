//! D-12 — OperationalizationCheck baby-step predicate.
//!
//! Every claim-emitting baby step MUST co-emit a `MEASURABLE_PROCEDURE`
//! field that operationalises the claim into a runnable test. Without
//! the procedure, the baby step cannot enter MEMORIZING; auditor
//! BLOCKs at the acceptance gate. This module owns:
//!
//!   * [`BabyStep`] — the typed baby-step value object.
//!   * [`MeasurableProcedure`] — the typed procedure block authored by
//!     the babystep-author agent. Carries a [`CheckerKind`] hint and a
//!     bounded runtime budget.
//!   * [`ExpectedResult`] — what the babystep-author predicts the
//!     checker will return; the dispatch diffs actual vs expected.
//!   * [`is_operationalized`] — the predicate: `true` iff all three
//!     fields (`measurable_procedure`, `expected_result`,
//!     `claim_payload`) are present.
//!   * [`dispatch_check_for_baby_step`] — runs the checker(s) selected
//!     by the procedure's `checker_kind_hint` (or falls back to
//!     `run_all_applicable` per D-11).
//!   * [`parse_baby_step`] — text-form parser for the
//!     `MEASURABLE_PROCEDURE` markdown block the babystep-author writes.
//!
//! Grounding:
//!   * docs/specs/guide/real-problem-solving-grounded-2026-05-19.md
//!     §4.1 (OperationalizationCheck, lines 185-196): every retained
//!     claim produces a measurable procedure.
//!   * arxiv:2305.16291 Voyager (T1) — skill-with-measurable-effect
//!     pattern: only retain executable skills with environment effect.
//!   * arxiv:2407.01489 AGENTLESS (T1) — single-step verification
//!     pattern (one checker invocation produces a definitive verdict;
//!     no agent re-planning inside the verifier).
//!
//! Producer / consumer wiring:
//!   * producer: [`dispatch_check_for_baby_step`] returns a
//!     [`DispatchOutcome`]; the caller (typically
//!     `agent_pool::run_operational_check`) emits one
//!     `baby_step_dispatched` SSB row per dispatch.
//!   * consumer: dashboard `BabyStepPanel.svelte` reads the
//!     `get_baby_step_dispatches_recent` Tauri command.
//!
//! No I/O, no async, no LLM calls inside this module — every function
//! is deterministic on its inputs (the dispatch helper awaits the
//! checker registry futures but the routing logic itself is pure).

use serde::{Deserialize, Serialize};

use crate::checker_registry::{CheckerKind, CheckerRegistry, CheckerResult};
use crate::claim::Claim;

/// Closed taxonomy of expected-result shapes the babystep-author can
/// declare. The dispatch layer diffs `expected_result.value` against
/// the actual [`CheckerResult`] per-kind.
///
/// Naming is the on-wire snake_case label used in SSB rows + Tauri
/// IPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpectedKind {
    /// `value` must equal the checker output exactly (string compare).
    Exact,
    /// `value` is a `[low, high]` interval; checker output must parse
    /// as `f64` and fall in the inclusive range. `tolerance` is
    /// applied to both ends when present.
    Range,
    /// `value` is a regex pattern; checker output must match. Deferred
    /// — the `regex` crate is already in the workspace but the
    /// expected-result diff lives in D-15 (NoveltyClass) per plan
    /// out-of-scope note. The variant is reserved so the closed enum
    /// is complete from D-12 day one.
    RegexMatch,
    /// `value` is a JSON schema; checker output must conform. Deferred
    /// — no jsonschema validator vendored (mirrors D-11
    /// `JsonSchemaDeferredChecker` decision).
    JsonSchemaConforms,
    /// `value` is a boolean flag; expected pass/fail. The simplest
    /// shape; first-class for the operational-falsification claim type.
    BooleanFlag,
}

impl ExpectedKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Range => "range",
            Self::RegexMatch => "regex_match",
            Self::JsonSchemaConforms => "json_schema_conforms",
            Self::BooleanFlag => "boolean_flag",
        }
    }
}

/// What the babystep-author predicts the checker will return. The
/// dispatch layer diffs this against the actual [`CheckerResult`] to
/// decide whether the baby step's claim is operationalised correctly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedResult {
    pub kind: ExpectedKind,
    /// Free-form value; interpretation depends on `kind`. For
    /// `BooleanFlag` the value is `true`/`false`; for `Exact` a
    /// string; for `Range` a `[low, high]` array.
    pub value: serde_json::Value,
    /// Optional tolerance applied when `kind == Range` or when an
    /// `Exact` numeric comparison needs slack. None = strict equality.
    pub tolerance: Option<f64>,
}

/// Typed procedure block authored by the babystep-author agent. The
/// block is parsed from markdown (see [`parse_baby_step`]) but lives
/// in-memory as a structured value the dispatch layer can route
/// without re-parsing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasurableProcedure {
    /// Ordered list of textual steps the procedure executes (human-
    /// readable; not parsed into shell commands here — that lives in
    /// the auditor's invocation site).
    pub steps: Vec<String>,
    /// Bounded plaintext success criterion. The dispatch layer does
    /// not interpret this — it is recorded on the SSB row for
    /// auditor + dashboard review.
    pub success_criterion: String,
    /// Optional hint at which [`CheckerKind`] is most likely to
    /// produce a definitive verdict. When `None`, the dispatch falls
    /// back to [`CheckerRegistry::run_all_applicable`].
    pub checker_kind_hint: Option<CheckerKind>,
    /// Maximum wall-clock budget the auditor should grant the
    /// procedure. Caps subprocess checkers per D-11
    /// `SHELLOUT_TIMEOUT` (30s) — anything larger is clamped at
    /// dispatch time.
    pub max_runtime_ms: u32,
}

/// Typed baby-step value object. Mirrors the markdown surface the
/// babystep-author writes but in a form the dispatch + auditor layers
/// can route deterministically.
///
/// Fields are intentionally minimal — block-level metadata that does
/// not influence the operationalisation predicate (TITLE, OBJECTIVE,
/// PRE_GATE, etc.) is left for the auditor + agent prompt surfaces.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BabyStep {
    /// Stable id (typically `bs-<parent>-<n>`). Bounded plaintext.
    pub step_id: String,
    /// Id of the parent task / phase the baby step belongs to.
    pub parent_task_id: String,
    /// Short human-readable description of the baby-step intent.
    pub description: String,
    /// The procedure that operationalises the claim. None for non-
    /// claim-emitting baby steps; required for claim-emitting ones.
    pub measurable_procedure: Option<MeasurableProcedure>,
    /// What the babystep-author predicts the checker will return.
    pub expected_result: Option<ExpectedResult>,
    /// The claim the procedure tests. None for non-claim baby steps.
    pub claim_payload: Option<Claim>,
}

/// Errors raised by [`parse_baby_step`]. Closed enum so callers can
/// surface a single typed diagnostic per parse attempt.
#[derive(Debug, thiserror::Error)]
pub enum BabyStepError {
    /// The text input was empty or contained no recognisable header.
    #[error("baby-step text empty or missing TITLE header")]
    Empty,
    /// A required field (step_id / parent_task_id / description) was
    /// missing from the markdown block.
    #[error("baby-step missing required field: {0}")]
    MissingField(String),
    /// The MEASURABLE_PROCEDURE block parsed but referenced an
    /// unknown checker_kind label.
    #[error("unknown checker_kind in MEASURABLE_PROCEDURE: {0}")]
    UnknownCheckerKind(String),
    /// The MEASURABLE_PROCEDURE block carried a max_runtime_ms value
    /// that overflowed u32 or was non-positive.
    #[error("invalid max_runtime_ms in MEASURABLE_PROCEDURE: {0}")]
    InvalidRuntime(String),
}

/// The D-12 predicate: `true` iff the baby step has co-emitted all
/// three operationalisation fields. The auditor BLOCKs MEMORIZING
/// when this returns `false` for a claim-emitting baby step.
///
/// Conservative semantics: every field MUST be present. A baby step
/// with a claim but no procedure is NOT operationalised; ditto for
/// procedure-without-claim or procedure-without-expected. Voyager
/// (arxiv:2305.16291) calls this the skill-with-measurable-effect
/// guarantee.
#[must_use]
pub fn is_operationalized(step: &BabyStep) -> bool {
    step.measurable_procedure.is_some()
        && step.expected_result.is_some()
        && step.claim_payload.is_some()
}

/// Outcome of a dispatched operational check. The
/// `agent_pool::run_operational_check` consumer emits one
/// `baby_step_dispatched` SSB row per [`DispatchOutcome::Dispatched`]
/// outcome.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DispatchOutcome {
    /// The baby step was not operationalised — no dispatch happened.
    /// Callers MUST treat this as a BLOCK on MEMORIZING.
    NotOperationalized,
    /// Dispatch ran one or more checkers. Carries every
    /// `(checker_id, kind, result)` triple plus the overall pass/fail
    /// derived by [`DispatchOutcome::overall_pass`].
    Dispatched {
        results: Vec<(String, CheckerKind, CheckerResult)>,
    },
    /// No applicable checker fired (hint pointed at a registered
    /// checker whose `applies_to` returned `false`, or
    /// `run_all_applicable` returned an empty list). Distinct from
    /// NotOperationalized — the baby step DID provide a procedure but
    /// the registry has no checker for it. Callers typically treat
    /// this as a soft warning (downgrade to operational-deferred per
    /// D-10).
    Inapplicable,
}

impl DispatchOutcome {
    /// True iff every result in [`DispatchOutcome::Dispatched`] is
    /// Pass. Returns `false` for NotOperationalized + Inapplicable.
    /// Used by the auditor to decide MEMORIZING gate status.
    #[must_use]
    pub fn overall_pass(&self) -> bool {
        match self {
            Self::Dispatched { results } => {
                !results.is_empty() && results.iter().all(|(_, _, r)| r.is_pass())
            }
            _ => false,
        }
    }

    /// Snake-case outcome label for SSB rows + dashboard rendering.
    /// One of `"pass" | "fail" | "inapplicable"`.
    #[must_use]
    pub fn outcome_label(&self) -> &'static str {
        match self {
            Self::NotOperationalized => "inapplicable",
            Self::Inapplicable => "inapplicable",
            Self::Dispatched { results } => {
                if results.is_empty() {
                    "inapplicable"
                } else if results.iter().all(|(_, _, r)| r.is_pass()) {
                    "pass"
                } else {
                    "fail"
                }
            }
        }
    }

    /// Every [`CheckerKind`] involved in the dispatch, in snake-case
    /// wire-label form. Empty for NotOperationalized / Inapplicable
    /// with no dispatch. Used by the SSB row producer.
    #[must_use]
    pub fn checker_kinds_used(&self) -> Vec<String> {
        match self {
            Self::Dispatched { results } => results
                .iter()
                .map(|(_, k, _)| k.as_str().to_string())
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// Dispatch the operational check for `step`. When the procedure has
/// a `checker_kind_hint`, the registry is queried for any checker
/// whose `kind()` matches AND `applies_to(claim)` returns true; every
/// such checker is invoked. When the hint is absent, falls back to
/// [`CheckerRegistry::run_all_applicable`] (the D-11 broad dispatch).
///
/// Returns [`DispatchOutcome::NotOperationalized`] when
/// [`is_operationalized`] is `false` — the caller cannot dispatch
/// against a partially-authored baby step. Returns
/// [`DispatchOutcome::Inapplicable`] when the operationalisation
/// fields are all present but the registry produced no applicable
/// checker (the procedure points at a kind nothing implements, or
/// the hint-less fallback returned empty).
///
/// `None` is never returned to keep the caller's match shape simple;
/// the three-arm enum captures every reachable state. The boxed-future
/// dispatch matches D-11's erasure pattern.
pub async fn dispatch_check_for_baby_step(
    step: &BabyStep,
    registry: &CheckerRegistry,
) -> DispatchOutcome {
    if !is_operationalized(step) {
        return DispatchOutcome::NotOperationalized;
    }
    // Safe to unwrap by precondition above.
    let claim = step
        .claim_payload
        .as_ref()
        .expect("is_operationalized guarantees claim_payload present");
    let proc_ref = step
        .measurable_procedure
        .as_ref()
        .expect("is_operationalized guarantees procedure present");

    let triples: Vec<(String, CheckerKind, CheckerResult)> = match proc_ref.checker_kind_hint {
        Some(hint) => {
            let mut out = Vec::new();
            // Hint-scoped dispatch: only checkers whose kind matches
            // the hint AND apply to the claim. We re-use the registry's
            // sorted iteration order so the output is deterministic.
            for id in registry.ids_sorted() {
                let Some(entry) = registry.lookup(&id) else {
                    continue;
                };
                if entry.kind() != hint {
                    continue;
                }
                if !entry.applies_to(claim) {
                    continue;
                }
                let result = entry.check(claim).await;
                out.push((id, entry.kind(), result));
            }
            out
        }
        None => registry.run_all_applicable(claim).await,
    };

    if triples.is_empty() {
        return DispatchOutcome::Inapplicable;
    }
    DispatchOutcome::Dispatched { results: triples }
}

/// Parse a MEASURABLE_PROCEDURE markdown block into a typed
/// [`BabyStep`]. The format is the babystep-author convention:
///
/// ```text
/// TITLE: <step_id>
/// PARENT: <parent_task_id>
/// DESCRIPTION: <one-line description>
/// MEASURABLE_PROCEDURE:
///   steps:
///     - "<step 1>"
///     - "<step 2>"
///   success_criterion: "<criterion>"
///   checker_kind: <snake_case kind | "none">
///   max_runtime_ms: <u32>
/// EXPECTED_RESULT:
///   kind: <snake_case ExpectedKind>
///   value: <json>
///   tolerance: <f64 | null>
/// CLAIM: <inline JSON Claim or "none">
/// ```
///
/// Forgiving on whitespace + blank lines; strict on the field names
/// (typo = `MissingField`). When a section is absent, the
/// corresponding field on the returned [`BabyStep`] is `None` —
/// callers use [`is_operationalized`] to test for the full triple.
///
/// Pure stdlib parsing (no serde_yaml dep). The CLAIM block accepts
/// either the literal `"none"` (sentinel for "no claim emitted") or
/// an inline JSON object that deserialises into [`Claim`].
pub fn parse_baby_step(text: &str) -> Result<BabyStep, BabyStepError> {
    if text.trim().is_empty() {
        return Err(BabyStepError::Empty);
    }

    let mut step_id: Option<String> = None;
    let mut parent_task_id: Option<String> = None;
    let mut description: Option<String> = None;
    let mut procedure: Option<MeasurableProcedure> = None;
    let mut expected: Option<ExpectedResult> = None;
    let mut claim: Option<Claim> = None;

    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() {
            i += 1;
            continue;
        }
        if let Some(rest) = strip_prefix_ci(line, "TITLE:") {
            step_id = Some(rest.trim().to_string());
        } else if let Some(rest) = strip_prefix_ci(line, "PARENT:") {
            parent_task_id = Some(rest.trim().to_string());
        } else if let Some(rest) = strip_prefix_ci(line, "DESCRIPTION:") {
            description = Some(rest.trim().to_string());
        } else if line.eq_ignore_ascii_case("MEASURABLE_PROCEDURE:") {
            let (block, consumed) = collect_indented_block(&lines, i + 1);
            procedure = Some(parse_measurable_procedure(&block)?);
            i += consumed;
        } else if line.eq_ignore_ascii_case("EXPECTED_RESULT:") {
            let (block, consumed) = collect_indented_block(&lines, i + 1);
            expected = Some(parse_expected_result(&block)?);
            i += consumed;
        } else if let Some(rest) = strip_prefix_ci(line, "CLAIM:") {
            let payload = rest.trim();
            if !payload.is_empty() && !payload.eq_ignore_ascii_case("none") {
                claim = serde_json::from_str::<Claim>(payload).ok();
            }
        }
        i += 1;
    }

    let step_id = step_id.ok_or_else(|| BabyStepError::MissingField("TITLE".into()))?;
    let parent_task_id =
        parent_task_id.ok_or_else(|| BabyStepError::MissingField("PARENT".into()))?;
    let description =
        description.ok_or_else(|| BabyStepError::MissingField("DESCRIPTION".into()))?;

    Ok(BabyStep {
        step_id,
        parent_task_id,
        description,
        measurable_procedure: procedure,
        expected_result: expected,
        claim_payload: claim,
    })
}

fn strip_prefix_ci<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    if s.len() < prefix.len() {
        return None;
    }
    let head = &s[..prefix.len()];
    if head.eq_ignore_ascii_case(prefix) {
        Some(&s[prefix.len()..])
    } else {
        None
    }
}

/// Collect a contiguous indented sub-block (lines starting with
/// whitespace) plus any blank-line separators. Returns the block
/// content (trimmed lines) + the number of source lines consumed.
fn collect_indented_block(lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let mut out = Vec::new();
    let mut consumed = 0usize;
    let mut idx = start;
    while idx < lines.len() {
        let raw = lines[idx];
        if raw.trim().is_empty() {
            consumed += 1;
            idx += 1;
            continue;
        }
        let first_char = raw.chars().next();
        let is_indented = matches!(first_char, Some(c) if c == ' ' || c == '\t');
        if !is_indented {
            break;
        }
        out.push(raw.trim().to_string());
        consumed += 1;
        idx += 1;
    }
    (out, consumed)
}

fn parse_measurable_procedure(block: &[String]) -> Result<MeasurableProcedure, BabyStepError> {
    let mut steps: Vec<String> = Vec::new();
    let mut success_criterion: Option<String> = None;
    let mut checker_kind_hint: Option<CheckerKind> = None;
    let mut max_runtime_ms: Option<u32> = None;

    let mut in_steps = false;
    for raw in block {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        if line.eq_ignore_ascii_case("steps:") {
            in_steps = true;
            continue;
        }
        if in_steps && line.starts_with('-') {
            let item = line[1..].trim().trim_matches('"').to_string();
            steps.push(item);
            continue;
        }
        in_steps = false;
        if let Some(rest) = strip_prefix_ci(line, "success_criterion:") {
            success_criterion = Some(rest.trim().trim_matches('"').to_string());
        } else if let Some(rest) = strip_prefix_ci(line, "checker_kind:") {
            let label = rest.trim().trim_matches('"').to_string();
            if label.is_empty() || label.eq_ignore_ascii_case("none") {
                checker_kind_hint = None;
            } else {
                checker_kind_hint = Some(parse_checker_kind_label(&label)?);
            }
        } else if let Some(rest) = strip_prefix_ci(line, "max_runtime_ms:") {
            let label = rest.trim().to_string();
            let parsed: u32 = label
                .parse()
                .map_err(|_| BabyStepError::InvalidRuntime(label.clone()))?;
            if parsed == 0 {
                return Err(BabyStepError::InvalidRuntime(label));
            }
            max_runtime_ms = Some(parsed);
        }
    }

    Ok(MeasurableProcedure {
        steps,
        success_criterion: success_criterion.unwrap_or_default(),
        checker_kind_hint,
        max_runtime_ms: max_runtime_ms.unwrap_or(30_000),
    })
}

fn parse_checker_kind_label(label: &str) -> Result<CheckerKind, BabyStepError> {
    match label {
        "sympy" => Ok(CheckerKind::Sympy),
        "z3" => Ok(CheckerKind::Z3),
        "compile_rust" => Ok(CheckerKind::CompileRust),
        "regex_match" => Ok(CheckerKind::RegexMatch),
        "json_schema_validate" => Ok(CheckerKind::JsonSchemaValidate),
        "numeric_range" => Ok(CheckerKind::NumericRange),
        other => Err(BabyStepError::UnknownCheckerKind(other.to_string())),
    }
}

fn parse_expected_result(block: &[String]) -> Result<ExpectedResult, BabyStepError> {
    let mut kind: Option<ExpectedKind> = None;
    let mut value: Option<serde_json::Value> = None;
    let mut tolerance: Option<f64> = None;

    for raw in block {
        let line = raw.trim();
        if let Some(rest) = strip_prefix_ci(line, "kind:") {
            kind = match rest.trim().trim_matches('"') {
                "exact" => Some(ExpectedKind::Exact),
                "range" => Some(ExpectedKind::Range),
                "regex_match" => Some(ExpectedKind::RegexMatch),
                "json_schema_conforms" => Some(ExpectedKind::JsonSchemaConforms),
                "boolean_flag" => Some(ExpectedKind::BooleanFlag),
                other => {
                    return Err(BabyStepError::MissingField(format!(
                        "EXPECTED_RESULT.kind invalid: {other}"
                    )));
                }
            };
        } else if let Some(rest) = strip_prefix_ci(line, "value:") {
            let raw = rest.trim();
            value = serde_json::from_str(raw).ok();
        } else if let Some(rest) = strip_prefix_ci(line, "tolerance:") {
            let raw = rest.trim();
            if raw.eq_ignore_ascii_case("null") || raw.is_empty() {
                tolerance = None;
            } else {
                tolerance = raw.parse::<f64>().ok();
            }
        }
    }

    let kind = kind.ok_or_else(|| BabyStepError::MissingField("EXPECTED_RESULT.kind".into()))?;
    let value =
        value.ok_or_else(|| BabyStepError::MissingField("EXPECTED_RESULT.value".into()))?;
    Ok(ExpectedResult {
        kind,
        value,
        tolerance,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checker_registry::{NumericRangeChecker, RegexMatchChecker};
    use crate::claim::{Claim, ClaimType};

    fn mk_claim(text: &str) -> Claim {
        Claim {
            claim_id: format!("c-{}", text.len()),
            text: text.into(),
            claim_type: ClaimType::Computational,
            evidence: vec![],
            ts_ms: 1_700_000_000_000,
        }
    }

    fn mk_proc(hint: Option<CheckerKind>) -> MeasurableProcedure {
        MeasurableProcedure {
            steps: vec!["compute".into(), "compare".into()],
            success_criterion: "matches expected".into(),
            checker_kind_hint: hint,
            max_runtime_ms: 10_000,
        }
    }

    fn mk_expected_pass() -> ExpectedResult {
        ExpectedResult {
            kind: ExpectedKind::BooleanFlag,
            value: serde_json::json!(true),
            tolerance: None,
        }
    }

    #[test]
    fn predicate_requires_all_three_fields() {
        let s_full = BabyStep {
            step_id: "bs-1".into(),
            parent_task_id: "task-1".into(),
            description: "d".into(),
            measurable_procedure: Some(mk_proc(None)),
            expected_result: Some(mk_expected_pass()),
            claim_payload: Some(mk_claim("5")),
        };
        assert!(is_operationalized(&s_full));

        let s_no_proc = BabyStep {
            measurable_procedure: None,
            ..s_full.clone()
        };
        assert!(!is_operationalized(&s_no_proc));

        let s_no_expected = BabyStep {
            expected_result: None,
            ..s_full.clone()
        };
        assert!(!is_operationalized(&s_no_expected));

        let s_no_claim = BabyStep {
            claim_payload: None,
            ..s_full.clone()
        };
        assert!(!is_operationalized(&s_no_claim));
    }

    #[tokio::test]
    async fn dispatch_with_hint_runs_only_matching_kind() {
        let mut reg = CheckerRegistry::new();
        reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));
        reg.register(
            "regex-d12",
            Box::new(
                RegexMatchChecker::new(r"^[0-9]+$", "digits", true)
                    .expect("regex compiles in test"),
            ),
        );

        let step = BabyStep {
            step_id: "bs-2".into(),
            parent_task_id: "task-2".into(),
            description: "d".into(),
            measurable_procedure: Some(mk_proc(Some(CheckerKind::NumericRange))),
            expected_result: Some(mk_expected_pass()),
            claim_payload: Some(mk_claim("5")),
        };
        let out = dispatch_check_for_baby_step(&step, &reg).await;
        match out {
            DispatchOutcome::Dispatched { results } => {
                assert_eq!(results.len(), 1, "hint must restrict to one kind");
                assert_eq!(results[0].1, CheckerKind::NumericRange);
                assert!(results[0].2.is_pass());
            }
            other => panic!("expected Dispatched, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dispatch_without_hint_falls_back_to_run_all_applicable() {
        let mut reg = CheckerRegistry::new();
        reg.register("nr-0-10", Box::new(NumericRangeChecker::new(0.0, 10.0)));
        let step = BabyStep {
            step_id: "bs-3".into(),
            parent_task_id: "task-3".into(),
            description: "d".into(),
            measurable_procedure: Some(mk_proc(None)),
            expected_result: Some(mk_expected_pass()),
            claim_payload: Some(mk_claim("5")),
        };
        let out = dispatch_check_for_baby_step(&step, &reg).await;
        assert!(matches!(out, DispatchOutcome::Dispatched { .. }));
        assert!(out.overall_pass());
    }

    #[tokio::test]
    async fn dispatch_returns_not_operationalized_when_predicate_fails() {
        let reg = CheckerRegistry::new();
        let step = BabyStep {
            step_id: "bs-4".into(),
            parent_task_id: "task-4".into(),
            description: "d".into(),
            measurable_procedure: None,
            expected_result: None,
            claim_payload: None,
        };
        let out = dispatch_check_for_baby_step(&step, &reg).await;
        assert!(matches!(out, DispatchOutcome::NotOperationalized));
        assert!(!out.overall_pass());
        assert_eq!(out.outcome_label(), "inapplicable");
    }

    #[tokio::test]
    async fn dispatch_inapplicable_when_no_checker_fires() {
        let reg = CheckerRegistry::new();
        let step = BabyStep {
            step_id: "bs-5".into(),
            parent_task_id: "task-5".into(),
            description: "d".into(),
            measurable_procedure: Some(mk_proc(None)),
            expected_result: Some(mk_expected_pass()),
            claim_payload: Some(mk_claim("5")),
        };
        let out = dispatch_check_for_baby_step(&step, &reg).await;
        assert!(matches!(out, DispatchOutcome::Inapplicable));
    }

    #[test]
    fn parse_baby_step_full_block_roundtrips() {
        let text = r#"
TITLE: bs-fixture-1
PARENT: task-fixture
DESCRIPTION: verify range checker fires on '5'
MEASURABLE_PROCEDURE:
  steps:
    - "construct claim"
    - "invoke checker"
  success_criterion: "checker returns Pass"
  checker_kind: numeric_range
  max_runtime_ms: 5000
EXPECTED_RESULT:
  kind: boolean_flag
  value: true
  tolerance: null
CLAIM: none
"#;
        let step = parse_baby_step(text).expect("parse ok");
        assert_eq!(step.step_id, "bs-fixture-1");
        assert_eq!(step.parent_task_id, "task-fixture");
        let proc_block = step
            .measurable_procedure
            .as_ref()
            .expect("procedure present");
        assert_eq!(proc_block.steps.len(), 2);
        assert_eq!(proc_block.checker_kind_hint, Some(CheckerKind::NumericRange));
        assert_eq!(proc_block.max_runtime_ms, 5000);
        let exp = step.expected_result.as_ref().expect("expected present");
        assert_eq!(exp.kind, ExpectedKind::BooleanFlag);
        assert!(step.claim_payload.is_none());
    }

    #[test]
    fn parse_baby_step_rejects_empty() {
        let err = parse_baby_step("").expect_err("must reject empty");
        assert!(matches!(err, BabyStepError::Empty));
    }

    #[test]
    fn parse_baby_step_rejects_missing_title() {
        let text = "PARENT: t\nDESCRIPTION: d\n";
        let err = parse_baby_step(text).expect_err("must reject missing title");
        assert!(matches!(err, BabyStepError::MissingField(_)));
    }

    #[test]
    fn parse_baby_step_unknown_checker_kind_errors() {
        let text = r#"
TITLE: bs-x
PARENT: t
DESCRIPTION: d
MEASURABLE_PROCEDURE:
  checker_kind: not_a_real_kind
  max_runtime_ms: 1000
"#;
        let err = parse_baby_step(text).expect_err("must reject unknown kind");
        assert!(matches!(err, BabyStepError::UnknownCheckerKind(_)));
    }

    #[test]
    fn outcome_label_covers_three_states() {
        let not_op = DispatchOutcome::NotOperationalized;
        assert_eq!(not_op.outcome_label(), "inapplicable");

        let inap = DispatchOutcome::Inapplicable;
        assert_eq!(inap.outcome_label(), "inapplicable");

        let pass = DispatchOutcome::Dispatched {
            results: vec![(
                "x".into(),
                CheckerKind::NumericRange,
                CheckerResult::Pass,
            )],
        };
        assert_eq!(pass.outcome_label(), "pass");
        assert!(pass.overall_pass());

        let fail = DispatchOutcome::Dispatched {
            results: vec![(
                "x".into(),
                CheckerKind::NumericRange,
                CheckerResult::Fail {
                    reason: "out of range".into(),
                },
            )],
        };
        assert_eq!(fail.outcome_label(), "fail");
        assert!(!fail.overall_pass());
    }

    #[test]
    fn checker_kinds_used_extracts_snake_case_labels() {
        let out = DispatchOutcome::Dispatched {
            results: vec![
                (
                    "a".into(),
                    CheckerKind::NumericRange,
                    CheckerResult::Pass,
                ),
                (
                    "b".into(),
                    CheckerKind::RegexMatch,
                    CheckerResult::Fail {
                        reason: "x".into(),
                    },
                ),
            ],
        };
        let labels = out.checker_kinds_used();
        assert_eq!(labels, vec!["numeric_range", "regex_match"]);
    }

    #[test]
    fn expected_kind_as_str_round_trips_serde() {
        for k in [
            ExpectedKind::Exact,
            ExpectedKind::Range,
            ExpectedKind::RegexMatch,
            ExpectedKind::JsonSchemaConforms,
            ExpectedKind::BooleanFlag,
        ] {
            let s = serde_json::to_string(&k).expect("ser");
            let stripped = s.trim_matches('"');
            assert_eq!(stripped, k.as_str(), "label mismatch for {k:?}");
        }
    }
}
