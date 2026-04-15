// A-47: Bare-message fallback classifier for routing unslashed user input.
// Uses str-based pattern matching (no regex crate in cli/).
// Hot path — all bare messages pass through this.

use crate::commands::SlashCommand;

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
    "how", "what", "why", "when", "does", "is", "can", "compare", "explain",
];

/// Phrases indicating code-modification intent (negates pure-question routing).
const CODE_INTENT_PHRASES: &[&str] = &[
    "make it",
    "change",
    "update the code",
];

/// Check if input contains an action verb as a word boundary match.
/// Words are delimited by whitespace; we compare lowercase tokens.
fn has_action_verb(input: &str) -> bool {
    let lower = input.to_lowercase();
    lower.split_whitespace().any(|word| {
        let stripped = word.trim_end_matches(|c: char| c.is_ascii_punctuation());
        ACTION_VERBS.iter().any(|verb| stripped == *verb)
    })
}

/// Check if input looks like a question (ends with '?' or starts with a question word).
fn has_question(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.ends_with('?') {
        return true;
    }
    let lower = trimmed.to_lowercase();
    let first_word = lower.split_whitespace().next().unwrap_or_default();
    QUESTION_PREFIXES.iter().any(|prefix| first_word == *prefix)
}

/// Check if input contains code-modification intent phrases.
fn has_code_intent(input: &str) -> bool {
    let lower = input.to_lowercase();
    CODE_INTENT_PHRASES.iter().any(|phrase| lower.contains(phrase))
}

/// Check if input references a project entity known to the DKG.
///
/// `known_names` is a slice of project-known identifiers (file names, function names,
/// module names) extracted from the DKG index. If empty (DKG unavailable), this
/// returns `false` — graceful degradation per spec.
fn has_project_ref(input: &str, known_names: &[&str]) -> bool {
    if known_names.is_empty() {
        return false;
    }
    let lower = input.to_lowercase();
    known_names.iter().any(|name| {
        let name_lower = name.to_lowercase();
        lower.contains(&name_lower)
    })
}

/// Classify a bare (non-slash-prefixed) message into a routed SlashCommand.
///
/// Route table (priority order, per pipeline-modes-design.txt §1):
/// 1. `HAS_ACTION_VERB` → `/task`
/// 2. `HAS_PROJECT_REF && HAS_QUESTION` → `/explore`
/// 3. `HAS_QUESTION && !HAS_CODE_INTENT && !HAS_PROJECT_REF` → `/research`
/// 4. default → `/task`
///
/// `known_names`: project-known identifiers from DKG. Pass `&[]` if DKG unavailable.
pub fn classify_bare_message(input: &str, known_names: &[&str]) -> SlashCommand {
    let action = has_action_verb(input);
    let question = has_question(input);
    let code_intent = has_code_intent(input);
    let project_ref = has_project_ref(input, known_names);

    if action {
        return SlashCommand::Task {
            description: Some(input.to_string()),
            surgical: false,
            new_project: false,
        };
    }

    if project_ref && question {
        return SlashCommand::Explore {
            target: Some(input.to_string()),
        };
    }

    if question && !code_intent && !project_ref {
        return SlashCommand::Research {
            question: Some(input.to_string()),
        };
    }

    // Default: treat as task.
    SlashCommand::Task {
        description: Some(input.to_string()),
        surgical: false,
        new_project: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_verb_routes_to_task() {
        let result = classify_bare_message("implement the login page", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn question_without_code_intent_routes_to_research() {
        let result = classify_bare_message("how does the authentication work?", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn question_prefix_without_mark_routes_to_research() {
        let result = classify_bare_message("what is the difference between X and Y", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn question_with_code_intent_routes_to_task() {
        let result = classify_bare_message("change how does this work?", &[]);
        // "change" is a code_intent phrase, so even with a question, it falls to default task
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn project_ref_plus_question_routes_to_explore() {
        let known = &["auth_module"];
        let result = classify_bare_message("how does auth_module handle tokens?", known);
        assert!(matches!(result, SlashCommand::Explore { .. }));
    }

    #[test]
    fn project_ref_without_question_routes_to_task() {
        let known = &["auth_module"];
        let result = classify_bare_message("auth_module needs cleanup", known);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn empty_dkg_degrades_gracefully() {
        // With no known names, project_ref is always false
        let result = classify_bare_message("how does auth_module handle tokens?", &[]);
        assert!(matches!(result, SlashCommand::Research { .. }));
    }

    #[test]
    fn bare_statement_routes_to_task() {
        let result = classify_bare_message("the tests are broken", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_takes_priority_over_question() {
        // "fix" is action verb, "?" makes it a question too, but action wins
        let result = classify_bare_message("fix the broken tests?", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_with_trailing_punctuation() {
        // "fix!" should still match after stripping punctuation
        let result = classify_bare_message("fix! how does this work?", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }

    #[test]
    fn action_verb_with_comma() {
        let result = classify_bare_message("implement, test, and deploy the feature", &[]);
        assert!(matches!(result, SlashCommand::Task { .. }));
    }
}
