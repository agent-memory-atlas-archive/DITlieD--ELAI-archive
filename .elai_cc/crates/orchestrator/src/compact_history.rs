//! J-76 C7 — intermediate-numerics extraction helpers for `_compact_history`.
//!
//! Extracted from `compact` so that file stays under the 1000-line
//! maintainability ceiling. The numerics extraction is pure regex-style token
//! scanning (no I/O, no LLM); `compact::summarize_messages` calls
//! [`extract_intermediate_numerics_from_messages`] when
//! `CompactionConfig::preserve_intermediate_numerics` is true.

use crate::session::{ContentBlock, ConversationMessage};

/// Maximum number of intermediate numerics preserved in a compact summary.
const MAX_INTERMEDIATE_NUMERICS: usize = 32;

/// Extract intermediate numerics from text for compact-summary preservation.
///
/// Keeps integers, decimals, percents, hex sizes like `0x..`, and simple
/// metric forms (e.g. `92.7 t/s`, `exit=1`). Dedup preserving first-seen
/// order. Cap list length at [`MAX_INTERMEDIATE_NUMERICS`].
#[must_use]
pub fn extract_intermediate_numerics(text: &str) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut push_if_new = |token: String| {
        if !seen.contains(&token) {
            seen.push(token);
        }
    };

    let tokens: Vec<&str> = text.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let cleaned = tokens[i]
            .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '%' && c != '=' && c != '/' && c != 'x');
        if cleaned.is_empty() {
            i += 1;
            continue;
        }
        if let Some(hex) = cleaned.strip_prefix("0x").and_then(|rest| {
            rest.chars().all(|c| c.is_ascii_hexdigit()).then_some(rest)
        }) {
            if !hex.is_empty() {
                push_if_new(format!("0x{hex}"));
            }
            i += 1;
            continue;
        }
        if let Some(pct) = cleaned.strip_suffix('%') {
            if is_number(pct) {
                push_if_new(format!("{pct}%"));
                i += 1;
                continue;
            }
        }
        if is_number(cleaned) {
            // Look ahead: if the next token is a unit (alphabetic/slash),
            // combine into a metric like "92.7 t/s".
            if i + 1 < tokens.len() {
                let next = tokens[i + 1]
                    .trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '/');
                if is_unit(next) {
                    push_if_new(format!("{cleaned} {next}"));
                    i += 2;
                    continue;
                }
            }
            push_if_new(cleaned.to_string());
            i += 1;
            continue;
        }
        if let Some((lhs, rhs)) = cleaned.split_once('=') {
            if is_number(rhs) {
                push_if_new(format!("{lhs}={rhs}"));
                i += 1;
                continue;
            }
        }
        if let Some((num, unit)) = split_metric(cleaned) {
            if is_number(num) && !unit.is_empty() && unit.chars().all(|c| c.is_ascii_alphabetic() || c == '/') {
                push_if_new(format!("{num} {unit}"));
                i += 1;
                continue;
            }
        }
        i += 1;
    }

    seen.truncate(MAX_INTERMEDIATE_NUMERICS);
    seen
}

fn is_number(s: &str) -> bool {
    let s = s.trim_start_matches('+').trim_start_matches('-');
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_digit() || c == '.')
        && s.chars().any(|c| c.is_ascii_digit())
}

fn is_unit(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphabetic() || c == '/')
        && s.chars().any(|c| c.is_ascii_alphabetic())
}

fn split_metric(s: &str) -> Option<(&str, &str)> {
    let mut split = 0usize;
    for (i, c) in s.char_indices() {
        if i == 0 {
            continue;
        }
        if !(c.is_ascii_digit() || c == '.' || c == '+' || c == '-') {
            split = i;
            break;
        }
    }
    if split == 0 {
        return None;
    }
    let (num, unit) = s.split_at(split);
    if is_number(num) {
        Some((num, unit))
    } else {
        None
    }
}

pub fn extract_intermediate_numerics_from_messages(messages: &[ConversationMessage]) -> Vec<String> {
    let mut all: Vec<String> = Vec::new();
    for message in messages {
        for block in &message.blocks {
            let text = match block {
                ContentBlock::Text { text } => text.as_str(),
                ContentBlock::ToolUse { input, .. } => input.as_str(),
                ContentBlock::ToolResult { output, .. } => output.as_str(),
            };
            for numeric in extract_intermediate_numerics(text) {
                if !all.contains(&numeric) {
                    all.push(numeric);
                }
            }
        }
    }
    all.truncate(MAX_INTERMEDIATE_NUMERICS);
    all
}

#[cfg(test)]
mod tests {
    use super::extract_intermediate_numerics;

    #[test]
    fn extract_intermediate_numerics_finds_decimals_percents_and_metrics() {
        let nums = extract_intermediate_numerics(
            "throughput was 92.7 t/s, coverage 100%, exit=1, hex 0xff, plain 42",
        );
        assert!(nums.contains(&"92.7 t/s".to_string()));
        assert!(nums.contains(&"100%".to_string()));
        assert!(nums.contains(&"exit=1".to_string()));
        assert!(nums.contains(&"0xff".to_string()));
        assert!(nums.contains(&"42".to_string()));
    }

    #[test]
    fn extract_intermediate_numerics_dedups_first_seen() {
        let nums = extract_intermediate_numerics("42 42 42");
        assert_eq!(nums, vec!["42".to_string()]);
    }
}
