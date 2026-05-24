//! E-14 — TOML journey loader.
//!
//! Loads journey definitions from `.elai_cc/journeys/*.toml` (or any
//! path the caller supplies). A journey is a small DSL:
//!
//! ```toml
//! name = "signup_smoke"
//! url = "http://localhost:5173/signup"
//!
//! [[steps]]
//! kind = "navigate"
//! target = "http://localhost:5173/signup"
//!
//! [[steps]]
//! kind = "fill"
//! target = "input[name='email']"
//! value = "elai@example.test"
//!
//! [[assertions]]
//! label = "loads-signup"
//! kind = "contains_text"
//! needle = "Sign up"
//! ```
//!
//! Steps the runner does NOT yet execute in the scope-lean cut (e.g.
//! `fill`, `click`) are validated for shape only; future work attaches
//! semantics. `assertions` round-trip through the same `Assertion`
//! struct the runtime consumes so there's a single source of truth.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::assertion::{Assertion, AssertionKind};
use super::E2eError;

/// Discriminator for what a journey step does. Open variants kept narrow
/// in the scope-lean cut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JourneyStepKind {
    /// Navigate the browser to `target` (a URL).
    Navigate,
    /// Click the selector at `target`.
    Click,
    /// Fill the input at `target` with `value`.
    Fill,
    /// Wait for selector `target` to appear (no semantic in scope-lean,
    /// runner records the intent for future enrichment).
    WaitFor,
}

impl JourneyStepKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Navigate => "navigate",
            Self::Click => "click",
            Self::Fill => "fill",
            Self::WaitFor => "wait_for",
        }
    }
}

/// One step of a journey. `target` and `value` semantics depend on `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyStep {
    pub kind: JourneyStepKind,
    pub target: String,
    #[serde(default)]
    pub value: Option<String>,
}

/// TOML-on-disk shape for the assertions array. Mirrors `Assertion` but
/// flattens `kind` + payload into the same table so authoring is one block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TomlAssertion {
    ContainsText { label: String, needle: String },
    HasSelector { label: String, selector: String },
    StatusOk { label: String },
}

impl From<TomlAssertion> for Assertion {
    fn from(t: TomlAssertion) -> Self {
        match t {
            TomlAssertion::ContainsText { label, needle } => Assertion {
                label,
                kind: AssertionKind::ContainsText { needle },
            },
            TomlAssertion::HasSelector { label, selector } => Assertion {
                label,
                kind: AssertionKind::HasSelector { selector },
            },
            TomlAssertion::StatusOk { label } => Assertion {
                label,
                kind: AssertionKind::StatusOk,
            },
        }
    }
}

/// On-disk journey definition. `assertions` are parsed via the
/// `TomlAssertion` intermediate so the public `Journey` struct holds the
/// canonical runtime `Assertion`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct TomlJourney {
    name: String,
    url: String,
    #[serde(default)]
    steps: Vec<JourneyStep>,
    #[serde(default)]
    assertions: Vec<TomlAssertion>,
}

/// Runtime journey definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Journey {
    pub name: String,
    pub url: String,
    pub steps: Vec<JourneyStep>,
    pub assertions: Vec<Assertion>,
}

/// Load a journey TOML from disk. Returns `MalformedOutput` on parse
/// failure so the caller can surface a parse-error badge separately
/// from an I/O failure.
pub fn load_journey(path: &Path) -> Result<Journey, E2eError> {
    let raw = fs::read_to_string(path)?;
    let toml_j: TomlJourney = toml::from_str(&raw).map_err(|e| E2eError::MalformedOutput {
        tool: "journey",
        detail: format!("toml parse failed for {}: {e}", path.display()),
    })?;
    Ok(Journey {
        name: toml_j.name,
        url: toml_j.url,
        steps: toml_j.steps,
        assertions: toml_j.assertions.into_iter().map(Into::into).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(dir: &TempDir, name: &str, body: &str) -> std::path::PathBuf {
        let p = dir.path().join(name);
        std::fs::write(&p, body).expect("write tmp");
        p
    }

    #[test]
    fn load_minimal_journey() {
        let dir = TempDir::new().expect("tempdir");
        let body = r#"
name = "smoke"
url = "http://localhost:5173/"

[[steps]]
kind = "navigate"
target = "http://localhost:5173/"

[[assertions]]
kind = "status_ok"
label = "loaded"
"#;
        let p = write(&dir, "smoke.toml", body);
        let j = load_journey(&p).expect("load");
        assert_eq!(j.name, "smoke");
        assert_eq!(j.steps.len(), 1);
        assert_eq!(j.steps[0].kind, JourneyStepKind::Navigate);
        assert_eq!(j.assertions.len(), 1);
        assert_eq!(j.assertions[0].label, "loaded");
        assert!(matches!(
            j.assertions[0].kind,
            AssertionKind::StatusOk
        ));
    }

    #[test]
    fn load_journey_with_fill_and_contains() {
        let dir = TempDir::new().expect("tempdir");
        let body = r#"
name = "signup"
url = "http://localhost:5173/signup"

[[steps]]
kind = "fill"
target = "input[name='email']"
value = "u@e.test"

[[assertions]]
kind = "contains_text"
label = "shows-form"
needle = "Sign up"
"#;
        let p = write(&dir, "signup.toml", body);
        let j = load_journey(&p).expect("load");
        assert_eq!(j.steps[0].kind, JourneyStepKind::Fill);
        assert_eq!(j.steps[0].value.as_deref(), Some("u@e.test"));
        match &j.assertions[0].kind {
            AssertionKind::ContainsText { needle } => assert_eq!(needle, "Sign up"),
            _ => panic!("expected contains_text"),
        }
    }

    #[test]
    fn malformed_toml_returns_malformed_output() {
        let dir = TempDir::new().expect("tempdir");
        let p = write(&dir, "bad.toml", "this is = not [valid toml");
        match load_journey(&p) {
            Err(E2eError::MalformedOutput { tool, detail }) => {
                assert_eq!(tool, "journey");
                assert!(detail.contains("toml parse failed"));
            }
            other => panic!("expected MalformedOutput, got {other:?}"),
        }
    }

    #[test]
    fn missing_file_returns_io_error() {
        let p = std::path::PathBuf::from("/does/not/exist/journey.toml");
        match load_journey(&p) {
            Err(E2eError::Io(_)) => {}
            other => panic!("expected Io, got {other:?}"),
        }
    }
}
