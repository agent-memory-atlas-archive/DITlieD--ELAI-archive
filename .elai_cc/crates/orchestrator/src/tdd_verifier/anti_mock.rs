//! E-13 — Anti-mock heuristic scanner for test diffs.
//!
//! Pattern-based scanner that flags inserted-test code containing
//! mocking primitives without a balancing integration-test marker.
//! The scanner is a heuristic (NOT a static analysis), so the
//! reported `AntiMockHit` rows are advisory — the caller decides
//! whether to block, warn, or accept.
//!
//! The plan's larger AntiMockHook surface (PostToolUse dispatcher
//! integration + cross-impl mutation-survival cross-check) was cut
//! to the heuristic scan per the re-check report's scope-lean lean.
//! The hook wiring lives upstream in the orchestrator's existing
//! PostToolUse hook chain; this module ships the pure-function
//! scanner so the auditor can call it without spinning up the full
//! hook dispatcher.
//!
//! Patterns (Rust-focused; other languages are PERMITTED but not
//! flagged by this scanner):
//!   * `#[mockall]` attribute
//!   * `#[mockall::automock]` attribute
//!   * `Mock<` generic instantiation
//!   * `MockAll::` path use
//!   * `mock!(` macro invocation
//!   * `mockall::mock!`
//!   * `MockHook::new(` constructor
//!
//! The scanner does NOT load any external regex dependency on first
//! call — patterns are simple literal substring matches compiled
//! once at module-init via `LazyLock<Vec<&str>>`.

use std::sync::LazyLock;

/// Closed list of mocking-primitive substrings the scanner flags
/// when inserted in test code. Append-only; new patterns land via
/// plan-level updates with an explicit decisions-doc entry.
static MOCK_PATTERNS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    vec![
        "#[mockall]",
        "#[mockall::automock]",
        "Mock<",
        "MockAll::",
        "mock!(",
        "mockall::mock!",
        "MockHook::new(",
    ]
});

/// One mocking-primitive hit detected by the scanner.
///
/// `pattern` is the literal pattern that matched.
/// `line_no` is the 1-based line number where the match occurred.
/// `line_text` is the offending line verbatim (trimmed to 240 chars
/// to bound payload size).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiMockHit {
    pub pattern: &'static str,
    pub line_no: usize,
    pub line_text: String,
}

const MAX_LINE_LEN: usize = 240;

/// Scan `diff_or_source` for mocking primitives.
///
/// Returns a non-empty `Vec<AntiMockHit>` when any
/// [`MOCK_PATTERNS`] entry occurs on a non-blank line of the input;
/// empty otherwise. The scanner does NOT inspect git-diff structure
/// (added/removed/context); callers MUST pre-filter to inserted
/// lines if that distinction matters.
///
/// Bounded cost: O(lines * patterns); patterns are 7 literals so the
/// per-line cost is constant. Lines longer than [`MAX_LINE_LEN`] are
/// truncated in the reported `line_text`.
#[must_use]
pub fn anti_mock_scan(diff_or_source: &str) -> Vec<AntiMockHit> {
    let mut hits = Vec::new();
    for (idx, raw_line) in diff_or_source.lines().enumerate() {
        let line = raw_line.trim_end();
        if line.is_empty() {
            continue;
        }
        for pattern in MOCK_PATTERNS.iter() {
            if line.contains(pattern) {
                let truncated = if line.len() > MAX_LINE_LEN {
                    let mut s = line[..MAX_LINE_LEN].to_string();
                    s.push_str("...");
                    s
                } else {
                    line.to_string()
                };
                hits.push(AntiMockHit {
                    pattern,
                    line_no: idx + 1,
                    line_text: truncated,
                });
            }
        }
    }
    hits
}

/// True when the input contains zero mock primitives. Convenience
/// wrapper used by the agent driver when no per-hit detail is
/// needed.
#[must_use]
pub fn is_mock_free(diff_or_source: &str) -> bool {
    anti_mock_scan(diff_or_source).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_no_hits() {
        assert!(anti_mock_scan("").is_empty());
        assert!(anti_mock_scan("   \n\n\t\n").is_empty());
    }

    #[test]
    fn clean_test_code_passes() {
        let src = "
            #[test]
            fn adds() {
                assert_eq!(2 + 2, 4);
            }
        ";
        assert!(is_mock_free(src));
    }

    #[test]
    fn mockall_attribute_flagged() {
        let src = "
            #[mockall::automock]
            pub trait MyTrait {}
        ";
        let hits = anti_mock_scan(src);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern, "#[mockall::automock]");
    }

    #[test]
    fn mock_macro_invocation_flagged() {
        let src = "mock!(MyMock);";
        let hits = anti_mock_scan(src);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].pattern, "mock!(");
    }

    #[test]
    fn long_line_is_truncated_in_report() {
        let long = "x".repeat(400);
        let src = format!("mock!({long});");
        let hits = anti_mock_scan(&src);
        assert_eq!(hits.len(), 1);
        assert!(hits[0].line_text.ends_with("..."));
        assert!(hits[0].line_text.len() <= MAX_LINE_LEN + 3);
    }

    #[test]
    fn multiple_hits_in_one_input() {
        let src = "
            #[mockall]
            struct A;
            let m: Mock<u32> = mock!(thing);
        ";
        let hits = anti_mock_scan(src);
        assert!(hits.len() >= 3);
    }
}
