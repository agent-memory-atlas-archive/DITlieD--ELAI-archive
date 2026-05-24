//! E-14 — Pure assertion runner.
//!
//! Given a [`PageSnapshot`] and a slice of [`Assertion`]s, produce a
//! deterministic [`AssertionReport`] with one [`AssertionOutcome`] per
//! input assertion. No subprocess, no I/O, no clock — easy to unit-test
//! deterministically.
//!
//! Assertion semantics are intentionally minimal in the scope-lean cut:
//!
//!   * `ContainsText { needle }` — `html.contains(needle)`. Surrogate
//!     for the AX-tree role/name lookup the full plan calls for; the
//!     full lookup is deferred to the VLM tier (RUNPOD-PENDING).
//!   * `HasSelector { selector }` — string-contains match against the
//!     selector string serialized into the HTML. Cheap and good enough
//!     for the wire-dark contract; a real DOM parser is deferred.
//!   * `StatusOk` — passes iff the snapshot has a non-empty HTML body
//!     (proxy for "page actually loaded"). Mirrors the GitHub
//!     url-health-check-action contract surface.
//!
//! All assertion failures are non-fatal at this layer — the caller
//! decides whether a failed assertion downgrades the verdict or stops
//! the journey.

use serde::{Deserialize, Serialize};

use super::snapshot::PageSnapshot;

/// Kind of assertion. New kinds go at the end so existing match sites
/// stay exhaustive without forcing a downstream re-fan-out.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssertionKind {
    /// Substring search over the rendered HTML.
    ContainsText { needle: String },
    /// Substring search for a selector string (e.g. `button[type='submit']`)
    /// — coarse proxy for a real DOM selector in the scope-lean cut.
    HasSelector { selector: String },
    /// Page-loaded sentinel: HTML body non-empty.
    StatusOk,
}

/// One declarative assertion bundled with an operator-facing label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assertion {
    /// Short human-readable label, e.g. "shows-signup-form". Stable
    /// across runs so the dashboard can attribute pass/fail counts to
    /// a specific named assertion.
    pub label: String,
    /// The kind discriminating the assertion semantics.
    pub kind: AssertionKind,
}

/// Per-assertion outcome. `Pass` carries no payload; `Fail` carries an
/// operator-facing message identifying which input failed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum AssertionOutcome {
    Pass {
        label: String,
    },
    Fail {
        label: String,
        message: String,
    },
}

impl AssertionOutcome {
    pub fn label(&self) -> &str {
        match self {
            Self::Pass { label } | Self::Fail { label, .. } => label,
        }
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }
}

/// Aggregate report. `outcomes` length always equals the input slice
/// length so callers can pair input -> outcome by index without
/// allocating a HashMap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssertionReport {
    pub outcomes: Vec<AssertionOutcome>,
    /// Number of outcomes that are `Pass`. Cached so dashboards don't
    /// have to iterate.
    pub passed: usize,
    /// Number of outcomes that are `Fail`. Cached so dashboards don't
    /// have to iterate.
    pub failed: usize,
}

impl AssertionReport {
    pub fn all_pass(&self) -> bool {
        self.failed == 0
    }
}

/// Run every assertion in `asserts` against `snapshot`. Pure, deterministic.
pub fn run_assertions(snapshot: &PageSnapshot, asserts: &[Assertion]) -> AssertionReport {
    let mut outcomes: Vec<AssertionOutcome> = Vec::with_capacity(asserts.len());
    let mut passed = 0usize;
    let mut failed = 0usize;
    for a in asserts {
        let outcome = match &a.kind {
            AssertionKind::ContainsText { needle } => {
                if snapshot.html.contains(needle) {
                    AssertionOutcome::Pass {
                        label: a.label.clone(),
                    }
                } else {
                    AssertionOutcome::Fail {
                        label: a.label.clone(),
                        message: format!("html does not contain `{needle}`"),
                    }
                }
            }
            AssertionKind::HasSelector { selector } => {
                if snapshot.html.contains(selector) {
                    AssertionOutcome::Pass {
                        label: a.label.clone(),
                    }
                } else {
                    AssertionOutcome::Fail {
                        label: a.label.clone(),
                        message: format!("no selector `{selector}` in rendered html"),
                    }
                }
            }
            AssertionKind::StatusOk => {
                if !snapshot.html.is_empty() {
                    AssertionOutcome::Pass {
                        label: a.label.clone(),
                    }
                } else {
                    AssertionOutcome::Fail {
                        label: a.label.clone(),
                        message: "page body empty".into(),
                    }
                }
            }
        };
        if outcome.is_pass() {
            passed += 1;
        } else {
            failed += 1;
        }
        outcomes.push(outcome);
    }
    AssertionReport {
        outcomes,
        passed,
        failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(html: &str) -> PageSnapshot {
        PageSnapshot {
            url: "http://test.local/".into(),
            html: html.into(),
            ax_tree: None,
            screenshot_path: None,
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn contains_text_passes_when_substring_present() {
        let s = snap("<html><body>Welcome to ELAI</body></html>");
        let asserts = vec![Assertion {
            label: "welcome".into(),
            kind: AssertionKind::ContainsText {
                needle: "Welcome to ELAI".into(),
            },
        }];
        let r = run_assertions(&s, &asserts);
        assert!(r.all_pass());
        assert_eq!(r.passed, 1);
        assert_eq!(r.failed, 0);
    }

    #[test]
    fn contains_text_fails_when_substring_absent() {
        let s = snap("<html><body>Hello</body></html>");
        let asserts = vec![Assertion {
            label: "missing-text".into(),
            kind: AssertionKind::ContainsText {
                needle: "Goodbye".into(),
            },
        }];
        let r = run_assertions(&s, &asserts);
        assert!(!r.all_pass());
        assert_eq!(r.failed, 1);
        match &r.outcomes[0] {
            AssertionOutcome::Fail { message, .. } => assert!(message.contains("Goodbye")),
            _ => panic!("expected fail"),
        }
    }

    #[test]
    fn has_selector_substring_match() {
        let s = snap("<html><body><button type='submit'>Go</button></body></html>");
        let asserts = vec![Assertion {
            label: "submit-button".into(),
            kind: AssertionKind::HasSelector {
                selector: "button type='submit'".into(),
            },
        }];
        let r = run_assertions(&s, &asserts);
        assert!(r.all_pass());
    }

    #[test]
    fn status_ok_fails_on_empty_body() {
        let s = snap("");
        let asserts = vec![Assertion {
            label: "loaded".into(),
            kind: AssertionKind::StatusOk,
        }];
        let r = run_assertions(&s, &asserts);
        assert_eq!(r.failed, 1);
    }

    #[test]
    fn mixed_pass_fail_counts_split() {
        let s = snap("<html><body>Hi</body></html>");
        let asserts = vec![
            Assertion {
                label: "loaded".into(),
                kind: AssertionKind::StatusOk,
            },
            Assertion {
                label: "absent".into(),
                kind: AssertionKind::ContainsText {
                    needle: "FOO".into(),
                },
            },
        ];
        let r = run_assertions(&s, &asserts);
        assert_eq!(r.passed, 1);
        assert_eq!(r.failed, 1);
        assert_eq!(r.outcomes.len(), 2);
    }
}
