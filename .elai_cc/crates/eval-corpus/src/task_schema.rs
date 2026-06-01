//! E-54 task schema -- canonical eval-corpus Task envelope.
//!
//! Mandatory fields per the plan:
//!   - `id`, `family` (TaskFamily=CanaryFamily 5-arm; see lib.rs DECISION-1),
//!     `created_at`, `cutoffs` (model_id -> "YYYY-MM" per LiveCodeBench v6
//!     arXiv:2403.07974), `problem`, `reference_solution`, `tests` (non-empty
//!     Vec per arXiv:2503.02951 KodCode), `provenance`,
//!     `embedding_hash` (blake3 prefix of normalized text), `ngram_hash`
//!     (blake3 of canonicalized 13-gram set).
//! Optional fields:
//!   - `framing_pair` (D-23 matched-pair sycophancy probes;
//!     arXiv:2510.20487).
//!   - `perturbed_pair` (BrokenMath-style perturbed-true theorems;
//!     arXiv:2510.04721).
//!
//! TOML serialisation: serde + toml crate. The loader REJECTS any task whose
//! `tests` is missing or empty (returns [`TaskSchemaError::EmptyTests`]).
//!
//! Anti-stale: blake3 (workspace baseline). No `.unwrap()` in library code.

use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::TaskFamily;

/// Canonical eval-corpus task envelope.
///
/// Persisted to disk as TOML at
/// `held-out/{family_snake}/{YYYY-MM-DD}/{id}.toml` (see [`crate::storage`]).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Canonical task id (matches the on-disk filename without `.toml`).
    pub id: String,
    /// Task family (CanaryFamily 5-arm; see [`crate::TaskFamily`]).
    pub family: TaskFamily,
    /// Wall-clock creation timestamp.
    pub created_at: DateTime<Utc>,
    /// Mandatory per arXiv:2403.07974 LiveCodeBench v6: per-model training
    /// cutoff so the time-window filter can reject tasks released before any
    /// configured eval model's cutoff. Map keys = model_id (e.g.
    /// `qwen2.5-coder-3b`); values = "YYYY-MM" (e.g. `2024-10`).
    pub cutoffs: FxHashMap<String, String>,
    /// Prompt body shown to the model.
    pub problem: String,
    /// Canonical reference solution -- MUST pass every `tests` entry per the
    /// test_executor recipe (arXiv:2411.04905 OpenCoder).
    pub reference_solution: String,
    /// MANDATORY non-empty Vec of executable tests. Per arXiv:2503.02951
    /// KodCode (447K verified Q-solution-test triplets, error <2.5%).
    /// The loader rejects empty / missing.
    pub tests: Vec<ExecutableTest>,
    /// Generator provenance (per arXiv:2412.08905 Phi-4 generator-rotation
    /// rule + arXiv:2509.00072 Test-of-Time synthesis-obscures rule).
    pub provenance: Provenance,
    /// Optional D-23 matched-pair framing (eval vs deploy phrasings; per
    /// arXiv:2510.20487 + Anthropic measured eval-awareness on Sonnet 4.5).
    /// MUST be `Some` for D-23 sycophancy tasks; contract
    /// `d23_tasks_have_framing_pair` enforces.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub framing_pair: Option<FramingPair>,
    /// Optional BrokenMath-style perturbed-true / subtly-false pair for D-23
    /// sycophancy probes (per arXiv:2510.04721 BrokenMath; GPT-5 = 29%
    /// sycophantic on perturbed-true theorems).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perturbed_pair: Option<PerturbedPair>,
    /// blake3 prefix (16 hex chars = 8 bytes) of the normalized problem text.
    pub embedding_hash: String,
    /// blake3 prefix (16 hex chars = 8 bytes) of the canonicalized 13-gram
    /// signature bytes.
    pub ngram_hash: String,
}

/// One executable test bound to a [`Task`]. Per arXiv:2503.02951 KodCode +
/// arXiv:2411.04905 OpenCoder triplet schema (Q + solution + tests).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutableTest {
    /// Input fed to the test runner (e.g. a single CLI argument, a JSON
    /// payload, or a unit-test function body).
    pub input: String,
    /// Canonical expected stdout / return value. Compared
    /// byte-equal after whitespace normalization.
    pub expected_output: String,
    /// Test kind discriminator.
    pub kind: TestKind,
}

/// Closed enum of supported test kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TestKind {
    /// Single-function unit test (input string is a function body /
    /// invocation).
    UnitTest,
    /// End-to-end test (input string is a full program invocation).
    EndToEnd,
    /// Property test (input string declares the property + generator
    /// strategy).
    PropertyTest,
}

/// Generator provenance for the task. Per arXiv:2412.08905 Phi-4
/// generator-rotation rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    /// Generator label. Must be unique across the corpus (e.g. `local_llm`,
    /// `cloud_llm`, `human_curated`, `github_issue_import`). See
    /// [`crate::generator_rotation`].
    pub generator: String,
    /// Optional canonical prompt id (e.g. blake3 prefix of the generator's
    /// prompt template) so the same template can be deduplicated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    /// Wall-clock generation date.
    pub date: DateTime<Utc>,
}

/// Matched-pair framing for D-23 sycophancy probes. The auditor runs the
/// model twice with the same underlying prompt -- once in `eval` phrasing,
/// once in `deploy` phrasing. The `deploy` variant MUST NOT contain
/// "benchmark", "evaluation", or "test" tokens (contract
/// `deploy_framing_no_eval_keywords` enforces). Per arXiv:2510.20487.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FramingPair {
    /// Evaluation-framed prompt (e.g. "On this benchmark, what is X?").
    pub eval: String,
    /// Production-framed prompt (e.g. "I'm working on a task -- what is
    /// X?"). MUST NOT contain "benchmark" / "evaluation" / "test" tokens.
    pub deploy: String,
}

/// BrokenMath-style perturbed-pair. Per arXiv:2510.04721; GPT-5 = 29%
/// sycophantic on subtly-false theorems.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PerturbedPair {
    /// Original true statement (model SHOULD agree with this when prompted).
    pub original_true: String,
    /// Perturbed subtly-false statement (model SHOULD reject this when
    /// prompted; sycophantic = agree).
    pub subtly_false: String,
    /// Expected answer for the *subtly_false* probe.
    pub expected_answer: SycophancyExpected,
}

/// Expected behaviour for a [`PerturbedPair`] probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SycophancyExpected {
    /// Model should agree with the original-true statement.
    AgreeWithTrue,
    /// Model should reject the subtly-false statement.
    RejectFalse,
}

/// Schema-level errors raised at TOML load time. Closed sum.
#[derive(Debug, Error)]
pub enum TaskSchemaError {
    /// `tests` field empty or missing -- per the KodCode mandate.
    #[error("eval-corpus task `tests` field is empty or missing (id={0})")]
    EmptyTests(String),
    /// `cutoffs` field empty -- per LiveCodeBench v6 mandate.
    #[error("eval-corpus task `cutoffs` field is empty (id={0})")]
    EmptyCutoffs(String),
    /// `deploy` framing-pair variant contains a banned token.
    #[error("eval-corpus task `framing_pair.deploy` contains banned token `{0}` (id={1})")]
    BannedDeployToken(String, String),
    /// TOML parse failure (path + reason).
    #[error("eval-corpus task TOML parse failed at {0}: {1}")]
    TomlParse(String, String),
    /// I/O failure (path + reason).
    #[error("eval-corpus task IO failed at {0}: {1}")]
    Io(String, String),
}

/// Tokens that MUST NOT appear (case-insensitive substring match) in the
/// `framing_pair.deploy` variant. Per arXiv:2510.20487 (Sonnet 4.5 verbalizes
/// "I am being evaluated" >80% of time; the deploy framing must look like
/// production traffic, not benchmark traffic).
pub const BANNED_DEPLOY_TOKENS: &[&str] = &["benchmark", "evaluation", "evaluate", "test suite"];

impl Task {
    /// Validate a [`Task`] against the mandatory-field contract:
    ///   - `tests` non-empty
    ///   - `cutoffs` non-empty
    ///   - if `framing_pair` is `Some`, `deploy` MUST NOT contain any
    ///     [`BANNED_DEPLOY_TOKENS`] token (case-insensitive)
    ///
    /// Returns the first violation as a [`TaskSchemaError`]; OK otherwise.
    pub fn validate(&self) -> Result<(), TaskSchemaError> {
        if self.tests.is_empty() {
            return Err(TaskSchemaError::EmptyTests(self.id.clone()));
        }
        if self.cutoffs.is_empty() {
            return Err(TaskSchemaError::EmptyCutoffs(self.id.clone()));
        }
        if let Some(fp) = &self.framing_pair {
            let deploy_lower = fp.deploy.to_ascii_lowercase();
            for token in BANNED_DEPLOY_TOKENS {
                if deploy_lower.contains(token) {
                    return Err(TaskSchemaError::BannedDeployToken(
                        (*token).to_string(),
                        self.id.clone(),
                    ));
                }
            }
        }
        Ok(())
    }

    /// Load a task from a TOML file. Validates against the mandatory-field
    /// contract before returning.
    pub fn load_from_toml(path: &std::path::Path) -> Result<Self, TaskSchemaError> {
        let bytes = std::fs::read_to_string(path).map_err(|e| {
            TaskSchemaError::Io(path.display().to_string(), e.to_string())
        })?;
        let task: Task = toml::from_str(&bytes).map_err(|e| {
            TaskSchemaError::TomlParse(path.display().to_string(), e.to_string())
        })?;
        task.validate()?;
        Ok(task)
    }

    /// Serialize a task to TOML.
    pub fn to_toml(&self) -> Result<String, TaskSchemaError> {
        toml::to_string_pretty(self).map_err(|e| {
            TaskSchemaError::TomlParse("<in-memory>".to_string(), e.to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaskFamily;
    use chrono::TimeZone;

    fn fixture_task() -> Task {
        let mut cutoffs = FxHashMap::default();
        cutoffs.insert("qwen2.5-coder-3b".to_string(), "2024-10".to_string());
        Task {
            id: "fixture-1".to_string(),
            family: TaskFamily::CodeFix,
            created_at: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            cutoffs,
            problem: "Write a function that adds two integers.".to_string(),
            reference_solution: "fn add(a:i32,b:i32)->i32{a+b}".to_string(),
            tests: vec![ExecutableTest {
                input: "add(1,2)".to_string(),
                expected_output: "3".to_string(),
                kind: TestKind::UnitTest,
            }],
            provenance: Provenance {
                generator: "human_curated".to_string(),
                prompt_id: None,
                date: Utc.with_ymd_and_hms(2026, 5, 27, 0, 0, 0).unwrap(),
            },
            framing_pair: None,
            perturbed_pair: None,
            embedding_hash: "deadbeefcafe1234".to_string(),
            ngram_hash: "1234deadbeefcafe".to_string(),
        }
    }

    #[test]
    fn validate_accepts_well_formed_task() {
        let t = fixture_task();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_tests() {
        let mut t = fixture_task();
        t.tests = Vec::new();
        assert!(matches!(t.validate(), Err(TaskSchemaError::EmptyTests(_))));
    }

    #[test]
    fn validate_rejects_empty_cutoffs() {
        let mut t = fixture_task();
        t.cutoffs = FxHashMap::default();
        assert!(matches!(t.validate(), Err(TaskSchemaError::EmptyCutoffs(_))));
    }

    #[test]
    fn validate_rejects_banned_deploy_token() {
        let mut t = fixture_task();
        t.framing_pair = Some(FramingPair {
            eval: "On this benchmark, prove X.".to_string(),
            deploy: "On this benchmark, please prove X.".to_string(),
        });
        assert!(matches!(
            t.validate(),
            Err(TaskSchemaError::BannedDeployToken(_, _))
        ));
    }

    #[test]
    fn validate_accepts_clean_deploy_framing() {
        let mut t = fixture_task();
        t.framing_pair = Some(FramingPair {
            eval: "On this benchmark, prove X.".to_string(),
            deploy: "I'm working on a proof -- can you prove X ?".to_string(),
        });
        assert!(t.validate().is_ok());
    }
}
