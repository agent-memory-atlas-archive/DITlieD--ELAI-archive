// J-32 Tier-1: Front-door router for total-function lane routing.
//
// T0 = slash bypass (handled by SlashCommand::parse in the caller).
// T1 = deterministic matchers ported from cli::classifier (action-verb,
// question, code-intent, project-ref) into this module so there is ONE
// implementation of the heuristic; cli::classifier becomes a thin facade.
// T2 (LMFE sidecar classification) is NOT in this slice.

use std::env;

/// The closed set of lanes the front door can dispatch to.
///
/// Tier-1 maps the existing classifier heuristics onto Chat/Task/Research/
/// Explore/Fix. The set is intentionally minimal; T2 may expand it but the
/// function is total by construction (Chat is the safe default).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontDoorLane {
    Chat,
    Task,
    Research,
    Explore,
    Fix,
}

/// Which tier produced the decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontDoorTier {
    T0,
    T1,
}

/// A byte-offset span into the original input identifying the argument
/// fragment for the routed lane. Never a paraphrase; the lane receives the
/// user's literal bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArgSpan {
    pub start: usize,
    pub end: usize,
}

/// T0 slash-bypass payload. When the input starts with a known slash name,
/// the router returns this so the caller can delegate to SlashCommand::parse
/// unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashBypass {
    pub name: String,
    pub arg_span: Option<ArgSpan>,
}

/// The front-door verdict. Composes a lane + tier + confidence + arg spans;
/// the caller (cli::classifier facade) maps `lane` onto SlashCommand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontDoorDecision {
    pub lane: FrontDoorLane,
    pub tier: FrontDoorTier,
    pub confidence: u8,
    pub arg_spans: Vec<ArgSpan>,
    pub slash_bypass: Option<SlashBypass>,
}

// -- T1 matcher primitives (the single implementation; classifier.rs calls
// these via the facade so there is no parallel heuristic). --

/// Action verbs that indicate a task-mode intent.
const ACTION_VERBS: &[&str] = &[
    "implement",
    "fix",
    "add",
    "create",
    "modify",
    "refactor",
    "build",
    "write",
];

/// Question prefixes that indicate an inquiry.
const QUESTION_PREFIXES: &[&str] = &[
    "how",
    "what",
    "why",
    "when",
    "does",
    "is",
    "can",
    "compare",
    "explain",
];

/// Phrases indicating code-modification intent (negates pure-question routing).
const CODE_INTENT_PHRASES: &[&str] = &["make it", "change", "update the code"];

/// "fix" as a standalone action verb routes to the Fix lane; other action
/// verbs route to Task. This keeps parity with the existing classifier's
/// Task default while giving Fix its own lane for the chip/risk mapping.
const FIX_VERB: &str = "fix";

/// Check if input contains an action verb as a word-boundary match.
/// Words are delimited by whitespace; we compare lowercase tokens.
pub fn has_action_verb(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.split_whitespace().any(|word| {
        let stripped = word.trim_end_matches(|c: char| c.is_ascii_punctuation());
        ACTION_VERBS.iter().any(|verb| stripped == *verb)
    })
}

/// Check if input looks like a question (ends with '?' or starts with a
/// question word).
pub fn has_question(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.ends_with('?') {
        return true;
    }
    let lower = trimmed.to_lowercase();
    let first_word = lower.split_whitespace().next().unwrap_or_default();
    QUESTION_PREFIXES.iter().any(|prefix| first_word == *prefix)
}

/// Check if input contains code-modification intent phrases.
pub fn has_code_intent(input: &str) -> bool {
    let lower = input.to_lowercase();
    CODE_INTENT_PHRASES.iter().any(|phrase| lower.contains(phrase))
}

/// Check if input references a project entity known to the DKG.
///
/// `known_names` is a slice of project-known identifiers (file names,
/// function names, module names) extracted from the DKG index. If empty
/// (DKG unavailable), this returns `false` (graceful degradation per spec).
pub fn has_project_ref(input: &str, known_names: &[&str]) -> bool {
    if known_names.is_empty() {
        return false;
    }
    let lower = input.to_lowercase();
    known_names.iter().any(|name| {
        let name_lower = name.to_lowercase();
        lower.contains(&name_lower)
    })
}

/// Whether the input's first token is the `fix` action verb (routes to the
/// Fix lane rather than the generic Task lane).
fn leads_with_fix(input: &str) -> bool {
    let lower = input.to_lowercase();
    let first = lower.split_whitespace().next().unwrap_or_default();
    let stripped = first.trim_end_matches(|c: char| c.is_ascii_punctuation());
    stripped == FIX_VERB
}

/// The whole-input arg span (start..end byte offsets into the original
/// input). Tier-1 routes the full message as the arg; T2 will extract
/// narrower spans.
fn full_input_span(input: &str) -> ArgSpan {
    let trimmed = input.trim_start();
    let leading = input.len() - trimmed.len();
    let end = leading + trimmed.trim_end().len();
    ArgSpan { start: leading, end }
}

/// The front-door router. Stateless; the struct exists so callers hold a
/// named handle and so T2 can later carry a sidecar handle without changing
/// the call site.
#[derive(Debug, Clone)]
pub struct FrontDoorRouter;

impl FrontDoorRouter {
    /// Resolve a bare (non-slash) message to a lane decision.
    ///
    /// `known_names`: project-known identifiers from the DKG; pass `&[]` if
    /// unavailable. `known_slash_names`: the set of registered slash command
    /// names; when the input leads with one, T0 bypass is returned.
    pub fn resolve(&self, input: &str, known_names: &[&str], known_slash_names: &[&str]) -> FrontDoorDecision {
        if let Some(bypass) = detect_slash_bypass(input, known_slash_names) {
            return FrontDoorDecision {
                lane: FrontDoorLane::Chat,
                tier: FrontDoorTier::T0,
                confidence: 100,
                arg_spans: vec![],
                slash_bypass: Some(bypass),
            };
        }

        let action = has_action_verb(input);
        let question = has_question(input);
        let code_intent = has_code_intent(input);
        let project_ref = has_project_ref(input, known_names);

        let span = full_input_span(input);

        if action {
            let lane = if leads_with_fix(input) {
                FrontDoorLane::Fix
            } else {
                FrontDoorLane::Task
            };
            return FrontDoorDecision {
                lane,
                tier: FrontDoorTier::T1,
                confidence: 90,
                arg_spans: vec![span],
                slash_bypass: None,
            };
        }

        if project_ref && question {
            return FrontDoorDecision {
                lane: FrontDoorLane::Explore,
                tier: FrontDoorTier::T1,
                confidence: 80,
                arg_spans: vec![span],
                slash_bypass: None,
            };
        }

        if question && !code_intent && !project_ref {
            return FrontDoorDecision {
                lane: FrontDoorLane::Research,
                tier: FrontDoorTier::T1,
                confidence: 80,
                arg_spans: vec![span],
                slash_bypass: None,
            };
        }

        // Default: Task (parity with the existing classifier default so the
        // facade's SlashCommand mapping stays bit-identical when the flag is
        // ON).
        FrontDoorDecision {
            lane: FrontDoorLane::Task,
            tier: FrontDoorTier::T1,
            confidence: 50,
            arg_spans: vec![span],
            slash_bypass: None,
        }
    }
}

/// T0 bypass: if the input starts with `/<name>` and `name` is a registered
/// slash command, return a bypass payload so the caller delegates to
/// SlashCommand::parse unchanged.
fn detect_slash_bypass(input: &str, known_slash_names: &[&str]) -> Option<SlashBypass> {
    let trimmed = input.trim_start();
    if !trimmed.starts_with('/') {
        return None;
    }
    let rest = &trimmed[1..];
    let (name_token, after) = match rest.find(|c: char| c.is_whitespace()) {
        Some(idx) => (&rest[..idx], Some(&rest[idx..])),
        None => (rest, None),
    };
    if known_slash_names.iter().any(|n| *n == name_token) {
        let arg_span = after.map(|a| {
            let skip = a.len() - a.trim_start().len();
            let start = input.len() - trimmed.len() + 1 + name_token.len() + skip;
            let end = input.len() - a.trim_end().len();
            ArgSpan { start, end }
        });
        Some(SlashBypass {
            name: name_token.to_string(),
            arg_span,
        })
    } else {
        None
    }
}

/// Whether the front-door router path is enabled. Reads the
/// `ELAI_FRONT_DOOR_ENABLE` env var; default OFF this slice.
pub fn enabled() -> bool {
    matches!(
        env::var("ELAI_FRONT_DOOR_ENABLE").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> FrontDoorRouter {
        FrontDoorRouter
    }

    #[test]
    fn t0_slash_bypass_when_name_registered() {
        let decision = router().resolve("/fix the bug", &[], &["fix", "task"]);
        assert_eq!(decision.tier, FrontDoorTier::T0);
        assert_eq!(
            decision.slash_bypass.as_ref().unwrap().name,
            "fix"
        );
        assert!(decision.slash_bypass.as_ref().unwrap().arg_span.is_some());
    }

    #[test]
    fn t0_no_bypass_for_unregistered_slash() {
        let decision = router().resolve("/unknown thing", &[], &["fix", "task"]);
        assert_ne!(decision.tier, FrontDoorTier::T0);
        assert!(decision.slash_bypass.is_none());
    }

    #[test]
    fn t1_action_verb_routes_to_task() {
        let decision = router().resolve("implement the login page", &[], &[]);
        assert_eq!(decision.tier, FrontDoorTier::T1);
        assert_eq!(decision.lane, FrontDoorLane::Task);
    }

    #[test]
    fn t1_fix_verb_routes_to_fix_lane() {
        let decision = router().resolve("fix the broken tests", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Fix);
        assert_eq!(decision.tier, FrontDoorTier::T1);
    }

    #[test]
    fn t1_question_without_code_intent_routes_to_research() {
        let decision = router().resolve("how does the authentication work?", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Research);
    }

    #[test]
    fn t1_question_prefix_without_mark_routes_to_research() {
        let decision = router().resolve("what is the difference between X and Y", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Research);
    }

    #[test]
    fn t1_question_with_code_intent_routes_to_task() {
        let decision = router().resolve("change how does this work?", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Task);
    }

    #[test]
    fn t1_project_ref_plus_question_routes_to_explore() {
        let known = &["auth_module"];
        let decision = router().resolve("how does auth_module handle tokens?", known, &[]);
        assert_eq!(decision.lane, FrontDoorLane::Explore);
    }

    #[test]
    fn t1_project_ref_without_question_routes_to_task() {
        let known = &["auth_module"];
        let decision = router().resolve("auth_module needs cleanup", known, &[]);
        assert_eq!(decision.lane, FrontDoorLane::Task);
    }

    #[test]
    fn t1_empty_dkg_degrades_gracefully() {
        let decision = router().resolve("how does auth_module handle tokens?", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Research);
    }

    #[test]
    fn t1_bare_statement_routes_to_task() {
        let decision = router().resolve("the tests are broken", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Task);
    }

    #[test]
    fn t1_action_verb_takes_priority_over_question() {
        let decision = router().resolve("fix the broken tests?", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Fix);
    }

    #[test]
    fn t1_action_verb_with_trailing_punctuation() {
        let decision = router().resolve("fix! how does this work?", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Fix);
    }

    #[test]
    fn t1_action_verb_with_comma() {
        let decision = router().resolve("implement, test, and deploy the feature", &[], &[]);
        assert_eq!(decision.lane, FrontDoorLane::Task);
    }

    #[test]
    fn arg_spans_are_byte_offsets_into_original() {
        let input = "  implement the login page  ";
        let decision = router().resolve(input, &[], &[]);
        let span = decision.arg_spans[0];
        let fragment = &input[span.start..span.end];
        assert_eq!(fragment, "implement the login page");
    }

    #[test]
    fn enabled_defaults_off_without_env() {
        env::remove_var("ELAI_FRONT_DOOR_ENABLE");
        assert!(!enabled());
    }

    #[test]
    fn enabled_on_when_env_set() {
        env::set_var("ELAI_FRONT_DOOR_ENABLE", "1");
        assert!(enabled());
        env::remove_var("ELAI_FRONT_DOOR_ENABLE");
    }
}
