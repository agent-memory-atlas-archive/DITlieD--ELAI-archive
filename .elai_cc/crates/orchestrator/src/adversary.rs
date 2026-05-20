//! D-13 — Adversary role + cross-family heterogeneous-verifier check.
//!
//! This module is the *generator* side of the adversarial-verification
//! pass. For a worker [`crate::claim::Claim`] it produces a structured
//! [`CounterEvidenceSchema`] via an implementor of the [`Adversary`]
//! trait. The consensus call-site
//! [`crate::agent_pool::run_adversarial_pass`] then re-runs the
//! verifier pool on `(claim, counter_evidence)` and emits two SSB rows:
//! `AdversaryGenerated` + `ConsensusReevaluatedWithCounter`.
//!
//! ## Grounding
//!
//! - arxiv:2403.07974 (T1) — *Adversarial verification* taxonomy: the
//!   adversary's job is to generate the *strongest plausible
//!   counter-claim*, not to "disagree by default". A verifier that
//!   cannot withstand a competent adversary on a different model
//!   family is not robust to deployment-time adversarial input.
//! - arxiv:2504.16760 (T1) — *Lightweight Latent Verifiers*: verifier
//!   transfer fails between base LLMs of the same family;
//!   heterogeneous-family verification is the documented defense.
//! - arxiv:2507.02778 (T1) — *Self-Correction Bench*: same-model
//!   self-critique has a 64.5% blind-spot rate. Cross-family critique
//!   is the auxiliary signal that breaks the blind spot.
//!
//! ## Hard constraints (per the plan)
//!
//! 1. The adversary's model family MUST differ from the worker's. The
//!    pure predicate [`heterogeneous_verifier_check`] enforces this for
//!    the *verifier pool* in the consensus pass; the trait contract
//!    [`Adversary::generate_counter`] is responsible for emitting from
//!    a different family at call time. Same-family adversary pairings
//!    are a stub-ban.
//! 2. The counter-evidence is STRUCTURED ([`CounterEvidenceSchema`]),
//!    not narrative. Narrative critique degrades small-model attention
//!    (CTIM-Rover REALM 2025, T1) and frustrates auditor inspection.
//! 3. Errors do not silently fall back to "no adversary" — they bubble
//!    via [`AdversaryError`] so the caller can decide between Mode E
//!    refusal and a retry.
//!
//! Native async-fn-in-trait per Anti-Stale-Training; `async-trait` is
//! rejected. The object-safe wrapper pattern mirrors
//! [`crate::agent_pool::AdapterErased`] from D-3.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::claim::{Claim, EvidenceRef};

// ---------------------------------------------------------------------------
// ModelFamily — closed enum of verifier model families
// ---------------------------------------------------------------------------

/// Closed enum of model families used by the heterogeneous-verifier
/// check. `Other(String)` is intentionally provided as an escape hatch
/// for future families not yet first-class in ELAI's deployment — same
/// shape as `crate::adversary::ModelFamily` in the D-13 plan, but the
/// closed variants are the only ones used by
/// [`heterogeneous_verifier_check`]'s built-in cross-family logic.
///
/// Two families are "different" iff their snake-case wire labels differ.
/// `Other(s1)` vs `Other(s2)` is "different" iff `s1 != s2`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelFamily {
    /// Anthropic Claude family (Sonnet / Opus / Haiku across versions).
    Anthropic,
    /// OpenAI GPT family (GPT-4 / 4o / 4.5 / 5 across versions).
    OpenAi,
    /// Meta Llama family (Llama 2 / 3 / 4 across sizes).
    Meta,
    /// Mistral / Mixtral family.
    Mistral,
    /// DeepSeek family (V2 / V3 / Coder across sizes).
    DeepSeek,
    /// Local on-device model (Qwen2.5-Coder-7B in ELAI's deployment).
    /// Treated as a single family regardless of the specific local
    /// checkpoint — local-vs-local pairings are same-family by design.
    Local,
    /// Escape hatch for families not yet first-class. Pairs of
    /// `Other(a)` and `Other(b)` are heterogeneous iff `a != b`.
    Other(String),
}

impl ModelFamily {
    /// Stable wire label used in SSB rows + dashboard rendering.
    /// `Other(s)` borrows `s`; other variants return a 'static literal.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Anthropic => "anthropic",
            Self::OpenAi => "openai",
            Self::Meta => "meta",
            Self::Mistral => "mistral",
            Self::DeepSeek => "deepseek",
            Self::Local => "local",
            Self::Other(s) => s.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// AdversaryConfig
// ---------------------------------------------------------------------------

/// Configuration for a single [`Adversary::generate_counter`] call. The
/// `adversary_model_family` field is the family the adversary MUST
/// emit from — the trait contract is responsible for refusing if the
/// underlying runtime does not match.
///
/// `severity_threshold` is the floor below which a counter-evidence
/// payload is treated as "not strong enough to invalidate" (the
/// consensus pass keeps the original verdict). It lives on the config
/// rather than the schema so per-mode tuning is config-driven.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdversaryConfig {
    pub adversary_model_family: ModelFamily,
    /// Max retries when the adversary's first counter-generation fails
    /// schema validation. `0` = strict (one shot, no retry). `> 0` =
    /// best-effort with bounded attempts.
    pub max_counter_attempts: u32,
    /// Minimum `severity` an emitted `CounterEvidenceSchema` must clear
    /// for the consensus pass to treat the claim as invalidated. Range
    /// `[0.0, 1.0]`; `0.0` accepts any counter, `1.0` accepts none.
    pub severity_threshold: f64,
}

impl AdversaryConfig {
    /// Default: cloud Sonnet as the cross-family adversary, single
    /// attempt, severity floor `0.5` (a "more-likely-than-not"
    /// counter-evidence threshold). Matches the D-13 plan's "no
    /// hedging — strongest plausible counter or `ClaimRobust`"
    /// contract.
    #[must_use]
    pub fn default_for_local_worker() -> Self {
        Self {
            adversary_model_family: ModelFamily::Anthropic,
            max_counter_attempts: 1,
            severity_threshold: 0.5,
        }
    }
}

// ---------------------------------------------------------------------------
// CounterEvidenceSchema
// ---------------------------------------------------------------------------

/// Structured counter-evidence produced by an [`Adversary`]. Carries
/// content-addressable evidence refs (so the auditor can inspect the
/// concrete sources the adversary cited) and a `severity` scalar on
/// `[0.0, 1.0]` indicating how strongly the counter-evidence
/// invalidates the original claim.
///
/// `severity == 0.0` is shorthand for "no counter-evidence found —
/// claim appears robust"; the adversary is contractually required to
/// emit a `severity == 0.0` payload rather than fail when it cannot
/// produce a counter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CounterEvidenceSchema {
    /// Stable per-counter identifier. Producers typically derive this
    /// from a blake3 of the counter body so dedupe is content-aware.
    pub counter_id: String,
    /// `claim_id` of the [`Claim`] this counter is responding to.
    pub original_claim_id: String,
    /// Bounded plaintext counter-claim. Adversary contract: this is
    /// the strongest plausible counter, not a hedge. Producers must
    /// truncate to a deployment-specific bound before calling
    /// `generate_counter` returns — no truncation here.
    pub counter_text: String,
    /// Concrete evidence references backing the counter. Empty `Vec`
    /// when `severity == 0.0` (no counter found); otherwise must be
    /// non-empty per the adversary contract.
    pub counter_evidence: Vec<EvidenceRef>,
    /// `[0.0, 1.0]` invalidation severity. `0.0` = claim robust; `1.0`
    /// = claim definitively falsified.
    pub severity: f64,
}

impl CounterEvidenceSchema {
    /// True iff this counter clears the supplied severity floor. Used
    /// by [`crate::agent_pool::run_adversarial_pass`] to decide
    /// whether the re-evaluated verdict should override the original.
    #[must_use]
    pub fn invalidates_claim(&self, severity_floor: f64) -> bool {
        self.severity >= severity_floor
    }
}

// ---------------------------------------------------------------------------
// AdversaryError
// ---------------------------------------------------------------------------

/// Errors emitted by an [`Adversary`] implementation. Per the plan,
/// adversary errors NEVER silently fall back to "no counter" — the
/// consensus call-site treats them as a hard signal that the
/// FalsifiabilityCrossFamily mode is unavailable for this claim.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AdversaryError {
    /// The adversary's runtime is not on a different family from the
    /// worker. Mode B refuses to dispatch — caller falls to Mode E.
    #[error("same_family_pairing: worker={worker} adversary={adversary}")]
    SameFamilyPairing {
        worker: String,
        adversary: String,
    },
    /// The adversary's runtime returned a payload that did not parse
    /// into [`CounterEvidenceSchema`]. The pass is rejected — fallback
    /// to a freeform text mode is a stub-ban per the D-13 plan.
    #[error("schema_parse_failure: {detail}")]
    SchemaParseFailure { detail: String },
    /// The adversary's runtime adapter was unreachable / errored.
    /// Caller decides retry vs. Mode E.
    #[error("runtime_unavailable: {detail}")]
    RuntimeUnavailable { detail: String },
    /// All `max_counter_attempts` retries were exhausted without a
    /// valid schema parse. Distinct from `SchemaParseFailure` so the
    /// caller can branch on retry-budget exhaustion.
    #[error("attempts_exhausted: {attempts}")]
    AttemptsExhausted { attempts: u32 },
}

// ---------------------------------------------------------------------------
// Adversary trait — native async fn in trait
// ---------------------------------------------------------------------------

/// Native async-fn-in-trait per Anti-Stale-Training (no `async-trait`
/// crate). For object-safe dispatch use the [`DynAdversary`] wrapper +
/// [`AdversaryErased`] blanket impl below, mirroring the D-3
/// [`crate::agent_pool::AdapterErased`] pattern.
///
/// Contract:
/// 1. The adversary MUST emit from a model family different from the
///    worker that produced `claim`. Same-family pairings return
///    [`AdversaryError::SameFamilyPairing`].
/// 2. On no-counter-found, return `Ok(schema)` with `severity == 0.0`
///    and `counter_evidence == vec![]` — NOT an error.
/// 3. On schema-parse failure exceeding `config.max_counter_attempts`,
///    return [`AdversaryError::AttemptsExhausted`].
pub trait Adversary: Send + Sync {
    fn generate_counter(
        &self,
        claim: &Claim,
        config: &AdversaryConfig,
    ) -> impl std::future::Future<Output = Result<CounterEvidenceSchema, AdversaryError>> + Send;
}

/// Object-safe wrapper trait that bridges [`Adversary`] (native async
/// fn in trait, not dyn-compatible) into a boxed-future shape callable
/// through `&dyn`. Implementors should NOT implement this trait
/// directly — implement [`Adversary`] and use the blanket
/// [`AdversaryErased`] wrapper.
pub trait DynAdversary: Send + Sync {
    fn generate_counter_dyn<'a>(
        &'a self,
        claim: &'a Claim,
        config: &'a AdversaryConfig,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<CounterEvidenceSchema, AdversaryError>>
                + Send
                + 'a,
        >,
    >;
}

/// Blanket wrapper that converts an [`Adversary`] (with native
/// async-fn-in-trait) into a [`DynAdversary`] (object-safe via boxed
/// future). Use `AdversaryErased::new(adversary)` to pass an adversary
/// across an `&dyn` boundary (e.g. into
/// [`crate::agent_pool::run_adversarial_pass`]).
pub struct AdversaryErased<A: Adversary> {
    inner: A,
}

impl<A: Adversary> AdversaryErased<A> {
    pub fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl<A: Adversary + 'static> DynAdversary for AdversaryErased<A> {
    fn generate_counter_dyn<'a>(
        &'a self,
        claim: &'a Claim,
        config: &'a AdversaryConfig,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = Result<CounterEvidenceSchema, AdversaryError>>
                + Send
                + 'a,
        >,
    > {
        Box::pin(self.inner.generate_counter(claim, config))
    }
}

// ---------------------------------------------------------------------------
// MockAdversary — deterministic test double
// ---------------------------------------------------------------------------

/// Programmable [`Adversary`] used in unit + integration tests. Returns
/// a fixed [`CounterEvidenceSchema`] for any incoming claim. The
/// `model_family` field is echoed into the schema so the same-family
/// guard in [`crate::agent_pool::run_adversarial_pass`] can be
/// exercised deterministically.
///
/// Construct via [`MockAdversary::with_counter`] for the "counter
/// found" path or [`MockAdversary::robust`] for the "no counter — claim
/// robust" path.
pub struct MockAdversary {
    counter: CounterEvidenceSchema,
    family: ModelFamily,
}

impl MockAdversary {
    /// Build a mock that returns a fixed counter-evidence schema.
    /// `family` is the model family the mock claims to emit from
    /// (passed through to the consensus pass for the
    /// heterogeneous-verifier check).
    #[must_use]
    pub fn with_counter(family: ModelFamily, counter: CounterEvidenceSchema) -> Self {
        Self { counter, family }
    }

    /// Build a mock that returns the "no counter — claim robust"
    /// payload (`severity == 0.0`, empty evidence). Convenience
    /// constructor for the Scenario-A integration test path.
    #[must_use]
    pub fn robust(family: ModelFamily, original_claim_id: impl Into<String>) -> Self {
        Self {
            family,
            counter: CounterEvidenceSchema {
                counter_id: "mock-robust".into(),
                original_claim_id: original_claim_id.into(),
                counter_text: String::new(),
                counter_evidence: Vec::new(),
                severity: 0.0,
            },
        }
    }

    /// Family this mock claims to emit from — consulted by callers
    /// that need to wire a (verifier_kind, model_family) entry for the
    /// heterogeneous-verifier check.
    #[must_use]
    pub fn family(&self) -> &ModelFamily {
        &self.family
    }
}

impl Adversary for MockAdversary {
    async fn generate_counter(
        &self,
        _claim: &Claim,
        _config: &AdversaryConfig,
    ) -> Result<CounterEvidenceSchema, AdversaryError> {
        Ok(self.counter.clone())
    }
}

// ---------------------------------------------------------------------------
// heterogeneous_verifier_check — pure predicate
// ---------------------------------------------------------------------------

/// Confirms the supplied verifier pool spans at least two distinct
/// [`ModelFamily`] values. Returns `true` iff the union of families
/// across `verifiers` has cardinality `>= 2`.
///
/// Empty input returns `false` — an empty pool cannot be heterogeneous.
/// One-element input returns `false` — a single family is by definition
/// homogeneous.
///
/// This is the central guard for the FalsifiabilityCrossFamily mode.
/// Per arxiv:2504.16760 (T1), verifier transfer fails between same-
/// family base LLMs; a "consensus" of five Qwen verifiers is not
/// stronger than one. The check is pure, deterministic, and allocates
/// only a small `Vec` for de-duplication.
#[must_use]
pub fn heterogeneous_verifier_check(
    _claim: &Claim,
    verifiers: &[(crate::verification_result::VerifierKind, ModelFamily)],
) -> bool {
    if verifiers.len() < 2 {
        return false;
    }
    let mut seen: Vec<&ModelFamily> = Vec::with_capacity(verifiers.len());
    for (_kind, fam) in verifiers {
        if !seen.iter().any(|s| *s == fam) {
            seen.push(fam);
            if seen.len() >= 2 {
                return true;
            }
        }
    }
    false
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::{ClaimType, EvidenceKind, EvidenceRef};
    use crate::verification_result::VerifierKind;

    fn mk_claim(id: &str) -> Claim {
        Claim {
            claim_id: id.into(),
            text: "synthetic claim".into(),
            claim_type: ClaimType::Factual,
            evidence: vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "src/main.rs:1-1",
                b"fn x() {}",
            )],
            ts_ms: 1_700_000_000_000,
        }
    }

    #[test]
    fn model_family_as_str_round_trip() {
        let all = [
            ModelFamily::Anthropic,
            ModelFamily::OpenAi,
            ModelFamily::Meta,
            ModelFamily::Mistral,
            ModelFamily::DeepSeek,
            ModelFamily::Local,
            ModelFamily::Other("custom-x".into()),
        ];
        for fam in &all {
            assert!(!fam.as_str().is_empty(), "label must be non-empty: {fam:?}");
        }
        assert_eq!(ModelFamily::Other("custom-x".into()).as_str(), "custom-x");
    }

    #[test]
    fn heterogeneous_check_two_families_true() {
        let claim = mk_claim("c-1");
        let pool = [
            (VerifierKind::Ppm, ModelFamily::Anthropic),
            (VerifierKind::Intrinsic, ModelFamily::Local),
            (VerifierKind::Deterministic, ModelFamily::Anthropic),
        ];
        assert!(heterogeneous_verifier_check(&claim, &pool));
    }

    #[test]
    fn heterogeneous_check_single_family_false() {
        let claim = mk_claim("c-2");
        let pool = [
            (VerifierKind::Ppm, ModelFamily::Local),
            (VerifierKind::Intrinsic, ModelFamily::Local),
            (VerifierKind::Deterministic, ModelFamily::Local),
        ];
        assert!(!heterogeneous_verifier_check(&claim, &pool));
    }

    #[test]
    fn heterogeneous_check_empty_false() {
        let claim = mk_claim("c-3");
        let pool: [(VerifierKind, ModelFamily); 0] = [];
        assert!(!heterogeneous_verifier_check(&claim, &pool));
    }

    #[test]
    fn heterogeneous_check_one_verifier_false() {
        let claim = mk_claim("c-4");
        let pool = [(VerifierKind::Ppm, ModelFamily::Anthropic)];
        assert!(!heterogeneous_verifier_check(&claim, &pool));
    }

    #[test]
    fn counter_evidence_invalidates_above_floor() {
        let c = CounterEvidenceSchema {
            counter_id: "k".into(),
            original_claim_id: "c".into(),
            counter_text: "Rust does not require GC".into(),
            counter_evidence: vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "src/lib.rs:1-1",
                b"// no gc",
            )],
            severity: 0.8,
        };
        assert!(c.invalidates_claim(0.5));
        assert!(!c.invalidates_claim(0.9));
    }

    #[test]
    fn counter_evidence_round_trip() {
        let c = CounterEvidenceSchema {
            counter_id: "k1".into(),
            original_claim_id: "c1".into(),
            counter_text: "counter text".into(),
            counter_evidence: vec![],
            severity: 0.0,
        };
        let s = serde_json::to_string(&c).expect("serialize");
        let back: CounterEvidenceSchema =
            serde_json::from_str(&s).expect("deserialize");
        assert_eq!(c, back);
    }

    #[tokio::test]
    async fn mock_adversary_returns_configured_counter() {
        let counter = CounterEvidenceSchema {
            counter_id: "k1".into(),
            original_claim_id: "c-1".into(),
            counter_text: "Rust borrow checker is the alternative to GC".into(),
            counter_evidence: vec![EvidenceRef::from_content(
                EvidenceKind::Corpus,
                "ref:rust-book",
                b"ownership",
            )],
            severity: 0.9,
        };
        let mock = MockAdversary::with_counter(ModelFamily::Anthropic, counter.clone());
        let cfg = AdversaryConfig::default_for_local_worker();
        let claim = mk_claim("c-1");
        let out = mock
            .generate_counter(&claim, &cfg)
            .await
            .expect("mock returns Ok");
        assert_eq!(out, counter);
        assert_eq!(mock.family(), &ModelFamily::Anthropic);
    }

    #[tokio::test]
    async fn mock_adversary_robust_path() {
        let mock = MockAdversary::robust(ModelFamily::Anthropic, "c-2");
        let cfg = AdversaryConfig::default_for_local_worker();
        let claim = mk_claim("c-2");
        let out = mock
            .generate_counter(&claim, &cfg)
            .await
            .expect("mock returns Ok");
        assert_eq!(out.severity, 0.0);
        assert!(out.counter_evidence.is_empty());
        assert!(!out.invalidates_claim(cfg.severity_threshold));
    }

    #[test]
    fn default_config_is_anthropic() {
        let cfg = AdversaryConfig::default_for_local_worker();
        assert_eq!(cfg.adversary_model_family, ModelFamily::Anthropic);
        assert_eq!(cfg.max_counter_attempts, 1);
        assert!((cfg.severity_threshold - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn adversary_error_display() {
        let e = AdversaryError::SameFamilyPairing {
            worker: "local".into(),
            adversary: "local".into(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("same_family_pairing"));
    }
}
