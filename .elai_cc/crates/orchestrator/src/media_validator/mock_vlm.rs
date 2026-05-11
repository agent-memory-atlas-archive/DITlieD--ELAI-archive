//! B-24 Phase 4I (Group C1): MockVlmBackend — deterministic Tier-2 stub.
//!
//! Phase 0 of the B-24 rollout SHIPS this backend in production CI: real
//! Tier 2 (Qwen3-VL / InternVL3.5) requires >= 24 GB GPU which is not
//! available in CI. The mock returns a fixed VlmValidationOutput so the
//! veto-logic exhaustive matrix in 4I.5 can be tested without a GPU.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::types::{Severity, VlmValidationOutput};

/// Backend abstraction used by Tier2Validator. Provides a single
/// `validate_advisory` operation. Real backends (Ollama, SGLang) implement
/// this in Group C5; the Mock is the only impl shipped in C1.
pub trait VlmBackend: Send + Sync {
    /// Returns the structured-output advisory. Mock returns the same
    /// `deterministic_output` every call. Real backends post the artifact
    /// + prompt to the local VLM server and parse the JSON response
    /// against `VlmValidationOutput`'s implicit schema.
    fn validate_advisory(&self, artifact: &std::path::Path) -> VlmValidationOutput;

    /// Identifier surfaced via `Tier2Result.model_used`. Mock returns
    /// `"mock-vlm"`; real backends return e.g. `"qwen3-vl:30b-a3b-instruct"`.
    fn model_id(&self) -> &str;
}

/// Default deterministic output returned by `MockVlmBackend`. Values were
/// picked to satisfy a permissive `QualitySpec` (overall_quality > 0.5,
/// confidence > 0.5, no issues) so the happy-path veto-logic test in C2
/// flows through `Verdict::Pass`.
fn default_mock_output() -> VlmValidationOutput {
    VlmValidationOutput {
        overall_quality: 0.85,
        issues: Vec::new(),
        confidence: 0.9,
        reasoning: "mock VLM advisory (Phase 0 / no GPU)".into(),
    }
}

/// Deterministic stand-in for the real VLM. Optionally accepts a fixture
/// path so future contract tests can pin a specific advisory profile (e.g.
/// "always emits a Major issue") without subclassing.
///
/// Audit BLOCK-2 fix (decision 40): added a shared `AtomicUsize` call
/// counter so adversarial tests (e.g. `a2_oversized`) can assert Tier2
/// was NEVER invoked. The counter is `Arc<AtomicUsize>` so `.clone()` is
/// cheap and observers can read it without taking ownership.
#[derive(Debug, Clone)]
pub struct MockVlmBackend {
    pub fixture: Option<PathBuf>,
    pub deterministic_output: VlmValidationOutput,
    pub call_count: Arc<AtomicUsize>,
}

impl MockVlmBackend {
    /// Default constructor — returns the always-pass advisory.
    pub fn new() -> Self {
        Self {
            fixture: None,
            deterministic_output: default_mock_output(),
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Pinned-output constructor — useful in adversarial tests that want
    /// to drive specific veto-logic branches without spinning up a VLM.
    pub fn with_output(output: VlmValidationOutput) -> Self {
        Self {
            fixture: None,
            deterministic_output: output,
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Convenience for the "Critical issue triggers HumanReview" branch.
    pub fn always_critical() -> Self {
        Self {
            fixture: None,
            deterministic_output: VlmValidationOutput {
                overall_quality: 0.4,
                issues: vec![super::types::Issue {
                    severity: Severity::Critical,
                    category: "fabrication".into(),
                    description: "mock-critical".into(),
                    location: None,
                }],
                confidence: 0.95,
                reasoning: "mock critical issue".into(),
            },
            call_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Returns the number of times `validate_advisory` has been invoked
    /// on any clone of this backend. Audit BLOCK-2 fix (decision 40):
    /// adversarial tests assert `==0` for the early-rejection branches
    /// (size_guard, path_guard, Tier1 fail).
    pub fn invocation_count(&self) -> usize {
        self.call_count.load(Ordering::SeqCst)
    }
}

impl Default for MockVlmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl VlmBackend for MockVlmBackend {
    fn validate_advisory(&self, _artifact: &std::path::Path) -> VlmValidationOutput {
        self.call_count.fetch_add(1, Ordering::SeqCst);
        self.deterministic_output.clone()
    }

    fn model_id(&self) -> &str {
        "mock-vlm"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn mock_returns_deterministic_output() {
        let backend = MockVlmBackend::new();
        let a = backend.validate_advisory(Path::new("a.png"));
        let b = backend.validate_advisory(Path::new("b.png"));
        assert_eq!(a, b);
        assert!(a.issues.is_empty());
        assert_eq!(a.overall_quality, 0.85);
    }

    #[test]
    fn always_critical_emits_critical_issue() {
        let backend = MockVlmBackend::always_critical();
        let out = backend.validate_advisory(Path::new("x.png"));
        assert_eq!(out.issues.len(), 1);
        assert_eq!(out.issues[0].severity, Severity::Critical);
    }
}
