//! C-29 — Testing-Trust Tier T-2 (IntentPair formalization).
//!
//! This module defines the IntentPair surface that grounds each BabyStep
//! against a small number of natural-language / concrete-assertion pairs
//! before the PLANNING -> TESTING_FIRST evidence gate is allowed to pass.
//!
//! Field naming follows the C-29 plan body and the TiCoder shape
//! (arXiv 2404.10100): natural_language_intent / concrete_test_assertion /
//! baby_step_ref. Bounded-string lengths (120/120/80 chars) match
//! testing-trust-design.txt §Layer 1 L555-558.
//!
//! Anti-staleness (per CLAUDE.md): thiserror for IntentError; no `Box<dyn
//! Error>`. No `.unwrap()` / `.expect()` outside `#[cfg(test)]`. char_count
//! is measured via `String::chars().count()` so multi-byte UTF-8 sequences
//! count as one character (see workflow/reasoning/C-29-decisions.md CSQ).
//!
//! IntentPairSet is the per-step *state* held on `BabyStepSpec`.
//! IntentConfirmation is the *SSB-row evidence record* emitted when the
//! IntentPairsGate auto-confirms or the dashboard's Confirm button fires.
//! Both shapes are needed; mirror the C-1 GroundingSweepResult /
//! GroundingSweepEvidence pair pattern. See context-package-C29 DRIFT-C29-8.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::research_tree::BabyStepId;

// ---------------------------------------------------------------------------
// IntentError — thiserror-backed error type
// ---------------------------------------------------------------------------

/// Errors produced by the IntentPair constructor + IntentPairSet::confirm.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
pub enum IntentError {
    #[error(
        "natural_language_intent exceeds {limit} chars (got {actual}): the intent must \
         fit within the spec budget"
    )]
    IntentTooLong { actual: usize, limit: usize },
    #[error(
        "concrete_test_assertion exceeds {limit} chars (got {actual}): the assertion \
         must fit within the spec budget"
    )]
    AssertionTooLong { actual: usize, limit: usize },
    #[error(
        "baby_step_ref exceeds {limit} chars (got {actual}): the ref must fit within \
         the spec budget"
    )]
    RefTooLong { actual: usize, limit: usize },
    #[error("natural_language_intent must be non-empty")]
    IntentEmpty,
    #[error("concrete_test_assertion must be non-empty")]
    AssertionEmpty,
    #[error(
        "cannot confirm: only {actual} pair(s) populated, need at least {min_pairs}"
    )]
    NotEnoughPairs { actual: usize, min_pairs: usize },
}

// ---------------------------------------------------------------------------
// Char bounds (spec L555-558)
// ---------------------------------------------------------------------------

/// Maximum char-count for `IntentPair.natural_language_intent`. Mirrors
/// testing-trust-design.txt §Layer 1 L555 "input_description (120 chars)".
pub const INTENT_MAX_CHARS: usize = 120;
/// Maximum char-count for `IntentPair.concrete_test_assertion`. Mirrors
/// testing-trust-design.txt §Layer 1 L556 "expected_output (120 chars)".
pub const ASSERTION_MAX_CHARS: usize = 120;
/// Maximum char-count for `IntentPair.baby_step_ref`. Mirrors
/// testing-trust-design.txt §Layer 1 L557 "rationale (80 chars)".
pub const REF_MAX_CHARS: usize = 80;

// ---------------------------------------------------------------------------
// IntentPair
// ---------------------------------------------------------------------------

/// A single IntentPair grounding the BabyStep in an NL/assertion pair.
///
/// Field naming preserves the C-29 plan body (TiCoder shape) per
/// DRIFT-C29-7 — `natural_language_intent` makes the Layer-2 grounding role
/// explicit; `concrete_test_assertion` is the string the Auditor uses to
/// validate Property soundness in T-3+; `baby_step_ref` is the foreign-key
/// edge to BabyStepId (canonical join key for SSB rows).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentPair {
    /// Natural-language statement of what the BabyStep should achieve.
    /// 120-char bound enforced at constructor via `IntentPair::new`.
    pub natural_language_intent: String,
    /// Concrete assertion (test pseudo-code or expected-output sketch).
    /// 120-char bound enforced at constructor.
    pub concrete_test_assertion: String,
    /// Free-form reference back to the owning baby step. By convention the
    /// dashboard writes `"bs-<step_id>"` here; the constructor enforces only
    /// the 80-char length budget.
    pub baby_step_ref: String,
}

impl IntentPair {
    /// Constructs a new IntentPair after validating char bounds.
    ///
    /// Rejects empty `natural_language_intent` / `concrete_test_assertion`
    /// outright (DRIFT-C29-7 + C-29-decisions.md CSQ). `baby_step_ref` may
    /// be empty when the dashboard auto-fills it post-construction; the
    /// 80-char ceiling still applies.
    pub fn new(
        natural_language_intent: impl Into<String>,
        concrete_test_assertion: impl Into<String>,
        baby_step_ref: impl Into<String>,
    ) -> Result<Self, IntentError> {
        let intent = natural_language_intent.into();
        let assertion = concrete_test_assertion.into();
        let r = baby_step_ref.into();

        let intent_chars = intent.chars().count();
        if intent.is_empty() {
            return Err(IntentError::IntentEmpty);
        }
        if intent_chars > INTENT_MAX_CHARS {
            return Err(IntentError::IntentTooLong {
                actual: intent_chars,
                limit: INTENT_MAX_CHARS,
            });
        }

        let assertion_chars = assertion.chars().count();
        if assertion.is_empty() {
            return Err(IntentError::AssertionEmpty);
        }
        if assertion_chars > ASSERTION_MAX_CHARS {
            return Err(IntentError::AssertionTooLong {
                actual: assertion_chars,
                limit: ASSERTION_MAX_CHARS,
            });
        }

        let ref_chars = r.chars().count();
        if ref_chars > REF_MAX_CHARS {
            return Err(IntentError::RefTooLong {
                actual: ref_chars,
                limit: REF_MAX_CHARS,
            });
        }

        Ok(Self {
            natural_language_intent: intent,
            concrete_test_assertion: assertion,
            baby_step_ref: r,
        })
    }
}

// ---------------------------------------------------------------------------
// IntentPairSet (per-step state)
// ---------------------------------------------------------------------------

/// Per-BabyStep collection of IntentPairs + confirmation status.
///
/// Default = empty unconfirmed set so `#[serde(default)]` on
/// `BabyStepSpec.intent_pairs` deserializes existing JSON without a schema
/// bump. `confirm()` requires at least `min_pairs` populated entries.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct IntentPairSet {
    /// Populated pairs. Order is insertion order; no de-duplication.
    pub pairs: Vec<IntentPair>,
    /// True once `confirm()` succeeded.
    pub confirmed: bool,
    /// UTC timestamp captured by `confirm()`, in milliseconds since
    /// `UNIX_EPOCH`. `None` when the set has never been confirmed.
    pub confirmed_at_ms: Option<i64>,
}

/// Wall-clock helper used by `IntentPairSet::confirm`. Returns ms since
/// UNIX_EPOCH, falling back to `0` if the system clock is before the
/// epoch (clock skew). Pure helper — no panic.
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

impl IntentPairSet {
    /// Construct an empty unconfirmed set. Equivalent to `Default::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of populated pairs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// Whether the set has zero pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Whether `confirm()` has been called and succeeded.
    #[must_use]
    pub fn is_confirmed(&self) -> bool {
        self.confirmed
    }

    /// Append a single pair. Returns the new total count.
    pub fn push(&mut self, pair: IntentPair) -> usize {
        self.pairs.push(pair);
        self.pairs.len()
    }

    /// Confirm the set. Requires `self.len() >= min_pairs`. Stamps the
    /// current UTC time on success. Subsequent calls are idempotent: the
    /// confirmation timestamp is preserved and `Ok(())` returned without
    /// rewriting the field.
    pub fn confirm(&mut self, min_pairs: usize) -> Result<(), IntentError> {
        if self.confirmed {
            return Ok(());
        }
        if self.pairs.len() < min_pairs {
            return Err(IntentError::NotEnoughPairs {
                actual: self.pairs.len(),
                min_pairs,
            });
        }
        self.confirmed = true;
        self.confirmed_at_ms = Some(now_ms());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ConfirmedBy + IntentConfirmation (SSB-row evidence)
// ---------------------------------------------------------------------------

/// Who confirmed the IntentPairSet. `User` = the dashboard's Confirm button;
/// `AutonomousGate` = the autonomous gate's auto-confirm path when pairs
/// >= min_pairs in autonomous mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmedBy {
    User,
    AutonomousGate,
}

impl ConfirmedBy {
    /// SQL/SSB-friendly snake_case string used by the SSB payload.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::AutonomousGate => "autonomous_gate",
        }
    }
}

/// The evidence-record (SSB-row payload) emitted when the gate auto-confirms
/// or the user clicks Confirm. Kept distinct from `IntentPairSet` so the
/// per-step state and the SSB-row witness can evolve independently.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentConfirmation {
    pub step_id: BabyStepId,
    pub pair_count: usize,
    pub confirmed_by: ConfirmedBy,
    /// Wall-clock ms since `UNIX_EPOCH` at confirmation time.
    pub timestamp_ms: i64,
}

// ---------------------------------------------------------------------------
// IntentConfig — [testing.intent] elai_cc.toml block
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Auto-confirm helper (autonomous gate bridge)
// ---------------------------------------------------------------------------

/// C-29 autonomous-gate bridge: decide whether the supplied
/// `IntentPairSet` qualifies for auto-confirmation under the given
/// `min_pairs` threshold. Pure function — no SSB side-effect, no clock
/// access. The autonomous gate evaluates this and, on `Some(_)`, emits
/// an `IntentConfirmed` SSB row via its existing writer with
/// `confirmed_by="autonomous_gate"`. On `None`, the gate auto-rejects with
/// `reason=intent_pairs_below_threshold` and refuses to advance (DRIFT-C29-3:
/// auto-confirm does NOT bypass the pair-count threshold).
#[must_use]
pub fn evaluate_autonomous_intent_confirm(
    set: &IntentPairSet,
    min_pairs: usize,
) -> Option<usize> {
    if min_pairs == 0 || set.len() >= min_pairs {
        Some(set.len())
    } else {
        None
    }
}

/// `[testing.intent]` block in `elai_cc.toml`. Defaults to `min_pairs = 3`
/// per testing-trust-design.txt L567 ("require SSB action INTENT_CONFIRMED
/// with at least 3 pairs per baby step"). Set `min_pairs = 0` to disable
/// the gate entirely (developer ergonomics, not the default).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct IntentConfig {
    /// Required IntentPair count for the PLANNING -> TESTING_FIRST gate.
    pub min_pairs: usize,
}

impl Default for IntentConfig {
    fn default() -> Self {
        Self { min_pairs: 3 }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn s(n: usize) -> String {
        "x".repeat(n)
    }

    #[test]
    fn intent_pair_rejects_empty_intent() {
        let err = IntentPair::new("", "non-empty assertion", "bs-1").unwrap_err();
        assert!(matches!(err, IntentError::IntentEmpty));
    }

    #[test]
    fn intent_pair_rejects_empty_assertion() {
        let err = IntentPair::new("non-empty intent", "", "bs-1").unwrap_err();
        assert!(matches!(err, IntentError::AssertionEmpty));
    }

    #[test]
    fn intent_pair_rejects_oversized_intent() {
        let err =
            IntentPair::new(s(INTENT_MAX_CHARS + 1), "assertion", "bs-1").unwrap_err();
        match err {
            IntentError::IntentTooLong { actual, limit } => {
                assert_eq!(actual, INTENT_MAX_CHARS + 1);
                assert_eq!(limit, INTENT_MAX_CHARS);
            }
            other => panic!("expected IntentTooLong, got {other:?}"),
        }
    }

    #[test]
    fn intent_pair_rejects_oversized_assertion() {
        let err =
            IntentPair::new("intent", s(ASSERTION_MAX_CHARS + 1), "bs-1").unwrap_err();
        match err {
            IntentError::AssertionTooLong { actual, limit } => {
                assert_eq!(actual, ASSERTION_MAX_CHARS + 1);
                assert_eq!(limit, ASSERTION_MAX_CHARS);
            }
            other => panic!("expected AssertionTooLong, got {other:?}"),
        }
    }

    #[test]
    fn intent_pair_rejects_oversized_ref() {
        let err = IntentPair::new("intent", "assertion", s(REF_MAX_CHARS + 1)).unwrap_err();
        match err {
            IntentError::RefTooLong { actual, limit } => {
                assert_eq!(actual, REF_MAX_CHARS + 1);
                assert_eq!(limit, REF_MAX_CHARS);
            }
            other => panic!("expected RefTooLong, got {other:?}"),
        }
    }

    /// CSQ (C-29-decisions.md): char-count semantics — multi-byte UTF-8
    /// counts per `char`, not per byte. 119 four-byte emoji must pass.
    #[test]
    fn intent_pair_char_bound_is_chars_not_bytes() {
        let nineteen_emoji: String = "\u{1F600}".repeat(119);
        let pair = IntentPair::new(nineteen_emoji.clone(), "ok", "bs-1")
            .expect("119 emoji must pass char-budget");
        assert_eq!(pair.natural_language_intent.chars().count(), 119);
        assert!(pair.natural_language_intent.len() > INTENT_MAX_CHARS);
    }

    #[test]
    fn intent_pair_set_default_is_empty_unconfirmed() {
        let set = IntentPairSet::default();
        assert!(set.is_empty());
        assert!(!set.is_confirmed());
        assert_eq!(set.len(), 0);
        assert!(set.confirmed_at_ms.is_none());
    }

    #[test]
    fn intent_pair_set_confirm_below_threshold_fails() {
        let mut set = IntentPairSet::new();
        set.push(IntentPair::new("a", "b", "bs-1").unwrap());
        set.push(IntentPair::new("c", "d", "bs-1").unwrap());
        let err = set.confirm(3).unwrap_err();
        match err {
            IntentError::NotEnoughPairs { actual, min_pairs } => {
                assert_eq!(actual, 2);
                assert_eq!(min_pairs, 3);
            }
            other => panic!("expected NotEnoughPairs, got {other:?}"),
        }
        assert!(!set.is_confirmed());
        assert!(set.confirmed_at_ms.is_none());
    }

    #[test]
    fn intent_pair_set_confirm_at_threshold_succeeds() {
        let mut set = IntentPairSet::new();
        set.push(IntentPair::new("a", "b", "bs-1").unwrap());
        set.push(IntentPair::new("c", "d", "bs-1").unwrap());
        set.push(IntentPair::new("e", "f", "bs-1").unwrap());
        set.confirm(3).expect("confirm at threshold");
        assert!(set.is_confirmed());
        assert!(set.confirmed_at_ms.is_some());
    }

    #[test]
    fn intent_pair_set_confirm_idempotent() {
        let mut set = IntentPairSet::new();
        for _ in 0..3 {
            set.push(IntentPair::new("a", "b", "bs-1").unwrap());
        }
        set.confirm(3).expect("first confirm");
        let first_ts = set.confirmed_at_ms;
        set.confirm(3).expect("idempotent confirm");
        assert_eq!(set.confirmed_at_ms, first_ts);
    }

    #[test]
    fn confirmed_by_as_str_round_trip() {
        assert_eq!(ConfirmedBy::User.as_str(), "user");
        assert_eq!(ConfirmedBy::AutonomousGate.as_str(), "autonomous_gate");
    }

    #[test]
    fn intent_config_default_min_pairs_is_3() {
        let cfg = IntentConfig::default();
        assert_eq!(cfg.min_pairs, 3);
    }
}
