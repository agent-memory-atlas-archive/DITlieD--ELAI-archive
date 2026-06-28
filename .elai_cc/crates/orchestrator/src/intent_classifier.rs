//! B-14 STEP 3: Intent classification for dashboard chat routing.
//!
//! Per spec `B-14_live_token_streaming_chat.md` STEP 3, every user input
//! is classified before routing into one of 7 categories:
//!   TASK / FIX / EXPLORE / CONFIG / INTERVENE / STEER / ABORT
//!
//! Classification is deterministic (slash commands + keywords) with an
//! `Ambiguous` fallback for natural-language inputs that need LLM
//! disambiguation. The actual LLM fallback call is deferred to the
//! dashboard layer (which has sidecar access); this module only exposes
//! the deterministic tier + the category enum so the Tauri
//! `classify_intent` command and the frontend TS classifier share one
//! canonical category set.
//!
//! The 7 categories match the spec's `DETERMINISTIC_CHECK`:
//!   `rg "TASK|FIX|EXPLORE|CONFIG|INTERVENE|STEER|ABORT" \
//!       crates/orchestrator/src --glob 'classifier*'`
//!
//! Production caller: `crates/dashboard-v2/src-tauri/src/commands.rs::
//! classify_intent` invokes [`classify_intent`] so the orchestrator
//! symbol is reachable from the registered Tauri command (lib.rs:807).

use serde::{Deserialize, Serialize};

/// The 7 spec intent categories plus an `Ambiguous` fallback for inputs
/// that need LLM disambiguation (spec FLOW 4 disambiguation card).
///
/// Wire label is `SCREAMING_SNAKE_CASE` to match the spec category names
/// exactly (`TASK`, `FIX`, `EXPLORE`, `CONFIG`, `INTERVENE`, `STEER`,
/// `ABORT`, `AMBIGUOUS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Intent {
    /// User wants to start/implement a new task.
    Task,
    /// User wants to debug/fix an issue.
    Fix,
    /// User wants to explore/ask about the codebase.
    Explore,
    /// User wants to change settings/configuration.
    Config,
    /// User is responding to an intervention card (approve/reject/redirect).
    Intervene,
    /// User wants to steer/adjust model behavior.
    Steer,
    /// User wants to stop/cancel/abort inflight work.
    Abort,
    /// Ambiguous natural language; dashboard should show the
    /// disambiguation card (spec FLOW 4) or fall back to LLM
    /// classification via the sidecar.
    Ambiguous,
}

impl Intent {
    /// SCREAMING_SNAKE_CASE wire label matching the spec category names.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Task => "TASK",
            Self::Fix => "FIX",
            Self::Explore => "EXPLORE",
            Self::Config => "CONFIG",
            Self::Intervene => "INTERVENE",
            Self::Steer => "STEER",
            Self::Abort => "ABORT",
            Self::Ambiguous => "AMBIGUOUS",
        }
    }
}

impl std::fmt::Display for Intent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Classify user input into one of the 7 spec intent categories.
///
/// Deterministic two-tier:
///   1. Slash command prefix -> direct map (`/task` `/fix` `/explore`
///      `/config` `/intervene` `/steer` `/abort` plus aliases
///      `/status`->EXPLORE, `/continue`->TASK, `/cancel`/`/stop`->ABORT,
///      `/research`/`/ask`->EXPLORE).
///   2. Keyword patterns (first match wins; order matters -- ABORT and
///      INTERVENE precede FIX so "cancel the fix" routes to ABORT, and
///      "approve the patch" routes to INTERVENE not TASK).
///   3. No match -> [`Intent::Ambiguous`] (LLM fallback marker).
///
/// This is the deterministic floor; the dashboard layer may invoke the
/// sidecar LLM for `Ambiguous` results per spec FLOW 4.
#[must_use]
pub fn classify_intent(text: &str) -> Intent {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Intent::Ambiguous;
    }
    let lower = trimmed.to_ascii_lowercase();
    let first_word = lower.split_whitespace().next().unwrap_or("");

    // Tier 1: slash commands.
    if let Some(intent) = slash_command_intent(first_word) {
        return intent;
    }

    // Tier 2: keyword patterns (first match wins).
    if is_abort(&lower) {
        return Intent::Abort;
    }
    if is_intervene(&lower) {
        return Intent::Intervene;
    }
    if is_fix(&lower) {
        return Intent::Fix;
    }
    if is_steer(&lower) {
        return Intent::Steer;
    }
    if is_config(&lower) {
        return Intent::Config;
    }
    if is_explore(&lower) {
        return Intent::Explore;
    }
    if is_task(&lower) {
        return Intent::Task;
    }

    Intent::Ambiguous
}

/// Whether the deterministic classifier was confident enough to skip the
/// LLM fallback. Returns `true` for all categories except `Ambiguous`.
#[must_use]
pub fn is_deterministic(intent: Intent) -> bool {
    !matches!(intent, Intent::Ambiguous)
}

fn slash_command_intent(word: &str) -> Option<Intent> {
    const MAP: &[(&str, Intent)] = &[
        ("/task", Intent::Task),
        ("/fix", Intent::Fix),
        ("/explore", Intent::Explore),
        ("/config", Intent::Config),
        ("/intervene", Intent::Intervene),
        ("/steer", Intent::Steer),
        ("/abort", Intent::Abort),
        ("/cancel", Intent::Abort),
        ("/stop", Intent::Abort),
        ("/halt", Intent::Abort),
        ("/status", Intent::Explore),
        ("/continue", Intent::Task),
        ("/research", Intent::Explore),
        ("/ask", Intent::Explore),
    ];
    MAP.iter().find(|(cmd, _)| *cmd == word).map(|(_, i)| *i)
}

fn is_abort(s: &str) -> bool {
    const VERBS: &[&str] = &["stop ", "cancel ", "abort ", "halt ", "kill "];
    VERBS.iter().any(|v| s.starts_with(v)) || s == "stop" || s == "cancel" || s == "abort"
}

fn is_intervene(s: &str) -> bool {
    // Requires BOTH an intervention verb AND a target reference so
    // "approve" alone (could be a task noun) does not false-positive.
    const VERBS: &[&str] = &["accept", "decline", "approve", "reject", "redirect"];
    const TARGETS: &[&str] = &[
        " this", " it", " change", " patch", " diff", " proposal", " intervention",
    ];
    let has_verb = VERBS.iter().any(|v| s.contains(v));
    let has_target = TARGETS.iter().any(|t| s.contains(t));
    has_verb && has_target
}

fn is_fix(s: &str) -> bool {
    const KEYS: &[&str] = &[
        "fix ", "repair ", "resolve ", "error", "failing", "failure", "broken", "crash",
        "bug", "stack trace", "panic", "exception",
    ];
    KEYS.iter().any(|k| s.contains(k))
}

fn is_steer(s: &str) -> bool {
    const KEYS: &[&str] = &["steer", "bias", "weight"];
    if KEYS.iter().any(|k| s.contains(k)) {
        return true;
    }
    // "adjust/update/tune ... vector" (tolerates an intervening word, e.g.
    // "adjust the vector"), so a steering-vector instruction is not missed.
    s.contains("vector") && (s.contains("adjust") || s.contains("update") || s.contains("tune"))
}

fn is_config(s: &str) -> bool {
    const KEYS: &[&str] = &[
        "switch to",
        "change model",
        "change mode",
        "set model",
        "use model",
        "configure",
        "settings",
    ];
    KEYS.iter().any(|k| s.contains(k))
}

fn is_explore(s: &str) -> bool {
    s.starts_with("what ")
        || s.starts_with("who ")
        || s.starts_with("where ")
        || s.starts_with("how ")
        || s.starts_with("why ")
        || s.contains(" explain ")
        || s.contains(" describe ")
        || s.contains("show me ")
        || s.contains('?')
}

fn is_task(s: &str) -> bool {
    const VERBS: &[&str] = &[
        "implement", "add ", "create ", "build ", "refactor", "update ", "remove ",
        "delete ", "rename ", "test ", "run ", "deploy ", "ship ", "write ",
    ];
    VERBS.iter().any(|v| s.starts_with(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_commands_route_directly() {
        assert_eq!(classify_intent("/task hello"), Intent::Task);
        assert_eq!(classify_intent("/fix the bug"), Intent::Fix);
        assert_eq!(classify_intent("/explore the code"), Intent::Explore);
        assert_eq!(classify_intent("/config settings"), Intent::Config);
        assert_eq!(classify_intent("/intervene approve"), Intent::Intervene);
        assert_eq!(classify_intent("/steer the model"), Intent::Steer);
        assert_eq!(classify_intent("/abort"), Intent::Abort);
    }

    #[test]
    fn slash_aliases_map_correctly() {
        assert_eq!(classify_intent("/cancel"), Intent::Abort);
        assert_eq!(classify_intent("/stop"), Intent::Abort);
        assert_eq!(classify_intent("/status"), Intent::Explore);
        assert_eq!(classify_intent("/continue"), Intent::Task);
        assert_eq!(classify_intent("/research papers"), Intent::Explore);
        assert_eq!(classify_intent("/ask a question"), Intent::Explore);
    }

    #[test]
    fn keyword_abort_precedes_fix() {
        // "cancel the fix" should route to ABORT not FIX.
        assert_eq!(classify_intent("cancel the fix"), Intent::Abort);
        assert_eq!(classify_intent("stop the build"), Intent::Abort);
        assert_eq!(classify_intent("halt everything"), Intent::Abort);
    }

    #[test]
    fn keyword_intervene_requires_target() {
        assert_eq!(classify_intent("approve this change"), Intent::Intervene);
        assert_eq!(classify_intent("reject the patch"), Intent::Intervene);
        // "approve" alone is ambiguous (could be a task noun).
        assert_eq!(classify_intent("approve"), Intent::Ambiguous);
    }

    #[test]
    fn keyword_fix_detects_errors() {
        assert_eq!(classify_intent("fix the crash"), Intent::Fix);
        assert_eq!(classify_intent("there is an error"), Intent::Fix);
        assert_eq!(classify_intent("the build is broken"), Intent::Fix);
    }

    #[test]
    fn keyword_explore_detects_questions() {
        assert_eq!(classify_intent("what does this do?"), Intent::Explore);
        assert_eq!(classify_intent("how does it work"), Intent::Explore);
        assert_eq!(classify_intent("show me the code"), Intent::Explore);
    }

    #[test]
    fn keyword_config_detects_settings() {
        assert_eq!(classify_intent("switch to cloud mode"), Intent::Config);
        assert_eq!(classify_intent("change model to sonnet"), Intent::Config);
    }

    #[test]
    fn keyword_steer_detects_bias() {
        assert_eq!(classify_intent("steer toward concise output"), Intent::Steer);
        assert_eq!(classify_intent("adjust the vector"), Intent::Steer);
    }

    #[test]
    fn keyword_task_detects_verbs() {
        assert_eq!(classify_intent("implement the feature"), Intent::Task);
        assert_eq!(classify_intent("add a new test"), Intent::Task);
        assert_eq!(classify_intent("build the module"), Intent::Task);
    }

    #[test]
    fn ambiguous_fallback_for_short_or_unknown() {
        assert_eq!(classify_intent(""), Intent::Ambiguous);
        assert_eq!(classify_intent("hello"), Intent::Ambiguous);
        assert_eq!(classify_intent("xyz"), Intent::Ambiguous);
    }

    #[test]
    fn all_seven_categories_present_in_as_str() {
        // Spec DETERMINISTIC_CHECK: all 7 categories must be present.
        let categories = [
            Intent::Task,
            Intent::Fix,
            Intent::Explore,
            Intent::Config,
            Intent::Intervene,
            Intent::Steer,
            Intent::Abort,
        ];
        for c in categories {
            assert!(!c.as_str().is_empty());
            assert_ne!(c.as_str(), "AMBIGUOUS");
        }
    }

    #[test]
    fn is_deterministic_true_for_all_except_ambiguous() {
        assert!(is_deterministic(Intent::Task));
        assert!(!is_deterministic(Intent::Ambiguous));
    }

    #[test]
    fn serde_round_trip_preserves_category() {
        for c in [
            Intent::Task,
            Intent::Fix,
            Intent::Explore,
            Intent::Config,
            Intent::Intervene,
            Intent::Steer,
            Intent::Abort,
            Intent::Ambiguous,
        ] {
            let s = c.as_str();
            let parsed: Intent =
                serde_json::from_value(serde_json::Value::String(s.to_string()))
                    .expect("round-trip");
            assert_eq!(c, parsed);
        }
    }

    #[test]
    fn display_matches_as_str() {
        assert_eq!(format!("{}", Intent::Task), "TASK");
        assert_eq!(format!("{}", Intent::Abort), "ABORT");
    }
}
