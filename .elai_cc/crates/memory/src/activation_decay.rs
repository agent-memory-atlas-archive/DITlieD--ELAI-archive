//! J-21 Step 4 — activation-decay scorer (TTL demoted to max-age guardrail).
//!
//! EXTENDS H-31/H-32/E-5 (and J-21 Steps 1-3) with the fourth grounded gap:
//! **fixed TTL forgets rare-but-important invariants**. A 28-day calendar
//! expiry drops a low-frequency high-value project convention that was
//! revalidated last month but not read since. The fix demotes fixed TTL
//! from the primary forgetting policy to a maximum-age GUARDRAIL and adds
//! an activation/heat score (usage + confidence + revalidation) so a
//! revalidated/used fact lives longer and a rare-but-important invariant
//! is not dropped by a calendar.
//!
//! # Architecture
//!
//! - [`score_activation`] — pure function computing an [`ActivationScore`]
//!   in `[0.0, 1.0]` from three signals: recency-of-use
//!   (`last_observed_at_ms`), recency-of-revalidation
//!   (`last_validated_at_ms`), and extractor `confidence`. A recently
//!   used/validated high-confidence fact scores ~1.0; a stale, never
//!   validated, low-confidence fact scores ~0.0.
//! - [`collect_activation_expired`] — the new expiry collector that
//!   REPLACES fixed-TTL-as-sole-policy. A Tier-A fact is expired only if
//!   EITHER (a) it exceeds the [`ActivationDecayConfig::max_age_guardrail_ms`]
//!   hard cap (the demoted TTL, now a guardrail) OR (b) it is past the
//!   baseline TTL AND its activation score fell below
//!   [`ActivationDecayConfig::retention_threshold`]. A past-TTL but
//!   high-activation invariant is RETAINED — the delta vs the old
//!   `collect_unused_ttl_expired`.
//! - [`get_authorized_facts_with_activation_decay`] — the composed
//!   retrieval entry point: chains J-21 Step 3's
//!   [`crate::raw_evidence::get_authorized_facts_with_escalation`] and
//!   re-ranks the authorized slice by activation score (descending) so the
//!   prompt receives high-activation facts first.
//! - [`activation_decay_tick`] — the maintenance function that reads
//!   current facts, collects activation-expired IDs, and soft-closes them.
//!   The orchestrator decay scheduler calls this instead of the old
//!   TTL-only collector when the DARK capture+retrieval is un-darked.
//!
//! # Grounding
//!
//! - GEM arXiv:2605.26252 — "correctness is a property of the state
//!   trajectory, not of individual records". The activation score is a
//!   state-trajectory signal (usage + revalidation history), not a
//!   per-record calendar stamp.
//! - H-30 staleness instrument — the `stale_fact_reached_prompt` ratio is
//!   the measure-before-trust keystone this activation-decay tunes against:
//!   retaining high-activation facts past TTL must not raise the stale-fact
//!   ratio (revalidation recency is a component of the score, so a retained
//!   fact is a recently-revalidated one).
//!
//! # Relationship to H-32 / J-21 Steps 1-3
//!
//! This module does NOT amend [`crate::citation_validity`],
//! [`crate::retrieval_authorization`], or [`crate::raw_evidence`]. It
//! composes them: [`get_authorized_facts_with_activation_decay`] calls
//! Step 3's [`crate::raw_evidence::get_authorized_facts_with_escalation`]
//! and re-ranks the result. [`collect_activation_expired`] is an additive
//! alternative to [`crate::citation_validity::collect_unused_ttl_expired`];
//! both remain available (the old collector is the fixed-TTL path, this is
//! the activation-decay path). The H-31 auditor-objectivity firewall is
//! maintained — activation scores are metadata for retrieval ranking, never
//! injected into auditor/critic/adversary contexts.

use crate::citation_validity::{classify_fact_tier, Tier, UNUSED_TTL_MS};
use crate::error::MemoryError;
use crate::fact::MemoryFact;
use crate::raw_evidence::{
    get_authorized_facts_with_escalation, RawEvidenceEscalation,
    RawEvidenceStore,
};
use crate::retrieval::RetrievalFilter;
use crate::retrieval_authorization::{AuthorizationExclusion, QuarantineStore};
use crate::semantic_validation::{SemanticValidator, ValidationScope};
use crate::citation_validity::CitationValidator;
use crate::store::FactStore;

/// Recency-of-use half-life: 14 days. A fact read within the last 14 days
/// retains >= 0.5 of its recency-of-use component. Chosen as half the
/// baseline TTL (28 days) so a fact read at the TTL boundary still carries
/// a meaningful use signal.
pub const USE_HALF_LIFE_MS: u128 = 14 * 24 * 60 * 60 * 1_000;

/// Recency-of-revalidation half-life: 30 days. A fact semantically
/// revalidated within the last 30 days retains >= 0.5 of its
/// recency-of-validation component. Chosen just past the 28-day TTL so a
/// fact revalidated at the TTL boundary (the H-32 reset) carries a strong
/// validation signal.
pub const VALIDATION_HALF_LIFE_MS: u128 = 30 * 24 * 60 * 60 * 1_000;

/// Default max-age guardrail: 180 days (~6.4x the 28-day baseline TTL).
/// Past this, a Tier-A fact is expired REGARDLESS of activation score — the
/// guardrail ensures dead facts eventually go. This is the demoted TTL:
/// no longer the sole forgetting policy, but the hard cap.
pub const DEFAULT_MAX_AGE_GUARDRAIL_MS: u128 = 180 * 24 * 60 * 60 * 1_000;

/// Default activation retention threshold: 0.3. A Tier-A fact past the
/// baseline TTL but with `heat >= 0.3` is RETAINED; below 0.3 it is
/// expired. A fact with no recent use AND no recent validation scores
/// `~0.2 * confidence` (the confidence-only floor), so a low-confidence
/// stale fact (0.18 at confidence 0.9) is expired while a recently
/// revalidated fact (>= 0.4 from validation alone) is retained.
pub const DEFAULT_ACTIVATION_RETENTION_THRESHOLD: f32 = 0.3;

/// Weight on the recency-of-use component of the activation score.
const W_USE: f32 = 0.4;
/// Weight on the recency-of-revalidation component.
const W_VALIDATION: f32 = 0.4;
/// Weight on the extractor confidence component.
const W_CONFIDENCE: f32 = 0.2;

// ---------------------------------------------------------------------------
// ActivationScore
// ---------------------------------------------------------------------------

/// The activation/heat score for a single fact, in `[0.0, 1.0]`. Composed
/// from three signals: recency-of-use, recency-of-revalidation, and
/// extractor confidence. A higher heat score means the fact is more
/// "active" and should be retained longer / ranked higher in retrieval.
#[derive(Debug, Clone, PartialEq)]
pub struct ActivationScore {
    /// Recency-of-use component in `[0.0, 1.0]` — `exp(-elapsed / use_half_life)`.
    pub recency_of_use: f32,
    /// Recency-of-revalidation component in `[0.0, 1.0]` —
    /// `exp(-elapsed / validation_half_life)`. `0.0` when the fact has
    /// never been validated (`last_validated_at_ms == None`).
    pub recency_of_validation: f32,
    /// Extractor confidence in `[0.0, 1.0]` (copied from the fact).
    pub confidence: f32,
    /// The composite heat score in `[0.0, 1.0]`:
    /// `W_USE * recency_of_use + W_VALIDATION * recency_of_validation +
    ///  W_CONFIDENCE * confidence`.
    pub heat: f32,
}

/// Configuration for the activation-decay expiry policy.
#[derive(Debug, Clone)]
pub struct ActivationDecayConfig {
    /// The max-age guardrail in ms. A Tier-A fact whose TTL reference point
    /// is older than this is expired regardless of activation score.
    /// Defaults to [`DEFAULT_MAX_AGE_GUARDRAIL_MS`] (180 days).
    pub max_age_guardrail_ms: u128,
    /// The activation retention threshold. A Tier-A fact past the baseline
    /// [`UNUSED_TTL_MS`] but with `heat >= this` is RETAINED. Defaults to
    /// [`DEFAULT_ACTIVATION_RETENTION_THRESHOLD`] (0.3).
    pub retention_threshold: f32,
    /// Recency-of-use half-life in ms. Defaults to [`USE_HALF_LIFE_MS`].
    pub use_half_life_ms: u128,
    /// Recency-of-revalidation half-life in ms. Defaults to
    /// [`VALIDATION_HALF_LIFE_MS`].
    pub validation_half_life_ms: u128,
}

impl Default for ActivationDecayConfig {
    fn default() -> Self {
        Self {
            max_age_guardrail_ms: DEFAULT_MAX_AGE_GUARDRAIL_MS,
            retention_threshold: DEFAULT_ACTIVATION_RETENTION_THRESHOLD,
            use_half_life_ms: USE_HALF_LIFE_MS,
            validation_half_life_ms: VALIDATION_HALF_LIFE_MS,
        }
    }
}

/// Pure exponential decay: `exp(-elapsed_ms / half_life_ms)`, clamped to
/// `[0.0, 1.0]`. Returns `1.0` when `elapsed` is zero (or future-dated),
/// `0.0` in the limit of large elapsed. Returns `0.0` when `half_life` is
/// zero (no half-life = no signal).
fn recency_component(elapsed_ms: u128, half_life_ms: u128) -> f32 {
    if half_life_ms == 0 {
        return 0.0;
    }
    let ratio = (elapsed_ms as f64) / (half_life_ms as f64);
    let factor = (-ratio).exp();
    factor.clamp(0.0, 1.0) as f32
}

/// Compute the activation/heat score for `fact` at `now_ms`.
///
/// The score is a pure function of the fact's `last_observed_at_ms`,
/// `last_validated_at_ms`, `valid_from_ms`, and `confidence` — no store
/// access, no I/O. Deterministic and unit-testable.
///
/// # Components
/// - `recency_of_use`: `exp(-(now - last_observed_at) / use_half_life)`.
/// - `recency_of_validation`: `exp(-(now - last_validated_at) /
///   validation_half_life)`, or `0.0` when never validated.
/// - `confidence`: the fact's extractor confidence.
/// - `heat`: weighted sum (`0.4 * use + 0.4 * validation + 0.2 * confidence`),
///   clamped to `[0.0, 1.0]`.
pub fn score_activation(fact: &MemoryFact, now_ms: u128, config: &ActivationDecayConfig) -> ActivationScore {
    let use_elapsed = now_ms.saturating_sub(fact.last_observed_at_ms);
    let recency_of_use = recency_component(use_elapsed, config.use_half_life_ms);

    let recency_of_validation = match fact.last_validated_at_ms {
        Some(v) => {
            let val_elapsed = now_ms.saturating_sub(v);
            recency_component(val_elapsed, config.validation_half_life_ms)
        }
        None => 0.0,
    };

    let confidence = fact.confidence.clamp(0.0, 1.0);

    let heat = (W_USE * recency_of_use
        + W_VALIDATION * recency_of_validation
        + W_CONFIDENCE * confidence)
        .clamp(0.0, 1.0);

    ActivationScore {
        recency_of_use,
        recency_of_validation,
        confidence,
        heat,
    }
}

// ---------------------------------------------------------------------------
// collect_activation_expired — TTL demoted to max-age guardrail
// ---------------------------------------------------------------------------

/// The activation-decay expiry collector. REPLACES
/// [`crate::citation_validity::collect_unused_ttl_expired`] as the primary
/// forgetting policy: fixed TTL is demoted to a max-age guardrail.
///
/// A Tier-A (`DomainKnowledge` / `Mistake`) fact with `valid_to_ms == None`
/// is expired iff EITHER:
/// 1. Its TTL reference point (`max(last_validated_at_ms, valid_from_ms)`)
///    is older than `config.max_age_guardrail_ms` — the hard guardrail
///    (dead facts eventually go regardless of activation), OR
/// 2. Its TTL reference point is older than [`UNUSED_TTL_MS`] (the baseline
///    28-day TTL) AND its [`score_activation`] `heat` is below
///    `config.retention_threshold` — activation-decay drove it out.
///
/// A past-TTL but high-activation invariant is RETAINED (not expired) —
/// the delta vs the old fixed-TTL collector. This is the
/// "rare-but-important invariant not dropped by a calendar" property.
///
/// Tier-B facts and already-closed facts are excluded (matching the old
/// collector's scope). The caller is responsible for `FactStore::soft_close`
/// on each returned ID (soft-close, never DELETE per stub_ban).
pub fn collect_activation_expired(
    facts: &[MemoryFact],
    now_ms: u128,
    config: &ActivationDecayConfig,
) -> Vec<String> {
    facts
        .iter()
        .filter(|f| classify_fact_tier(f.fact_type) == Tier::A)
        .filter(|f| f.valid_to_ms.is_none())
        .filter(|f| {
            let ttl_ref = f
                .last_validated_at_ms
                .unwrap_or(f.valid_from_ms)
                .max(f.valid_from_ms);
            let age = now_ms.saturating_sub(ttl_ref);

            // (1) Max-age guardrail: expired regardless of activation.
            if age > config.max_age_guardrail_ms {
                return true;
            }
            // (2) Past baseline TTL: expired only if activation below threshold.
            if age > UNUSED_TTL_MS {
                let score = score_activation(f, now_ms, config);
                return score.heat < config.retention_threshold;
            }
            // Within TTL: not expired by this collector.
            false
        })
        .map(|f| f.fact_id.clone())
        .collect()
}

// ---------------------------------------------------------------------------
// activation_decay_tick — maintenance (collect + soft-close)
// ---------------------------------------------------------------------------

/// Aggregate stats emitted by [`activation_decay_tick`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ActivationDecayStats {
    /// Number of currently-valid Tier-A rows examined.
    pub rows_examined: usize,
    /// Number of rows soft-closed by this tick.
    pub rows_expired: usize,
    /// Number of past-TTL rows RETAINED because their activation score
    /// met the retention threshold (the delta vs fixed-TTL).
    pub rows_retained_past_ttl: usize,
}

/// Maintenance pass: read all currently-valid facts for `user_id`,
/// collect activation-expired IDs, and soft-close them. The orchestrator
/// decay scheduler calls this instead of the old TTL-only collector when
/// the DARK capture+retrieval is un-darked.
///
/// Uses [`FactStore::list_current`] (per-user, full `MemoryFact` rows) so
/// the activation scorer has access to `last_validated_at_ms`,
/// `valid_from_ms`, and `confidence` — the lightweight `iter_current`
/// projection lacks those columns. The `limit` is set high enough to cover
/// any realistic per-user fact count.
///
/// `soft_close` on an already-closed fact returns `NotFound` which is
/// treated as benign (a concurrent path closed it first) — logged, not
/// propagated.
pub fn activation_decay_tick(
    store: &mut FactStore,
    user_id: &str,
    now_ms: u128,
    config: &ActivationDecayConfig,
) -> Result<ActivationDecayStats, MemoryError> {
    // Generous limit — a per-user fact store is not expected to exceed this.
    let limit = 100_000usize;
    let rows = store.list_current(user_id, None, None, limit)?;
    let rows_examined = rows
        .iter()
        .filter(|f| classify_fact_tier(f.fact_type) == Tier::A)
        .count();

    // Count past-TTL retained for observability (the delta signal).
    let rows_retained_past_ttl = rows
        .iter()
        .filter(|f| classify_fact_tier(f.fact_type) == Tier::A)
        .filter(|f| {
            let ttl_ref = f
                .last_validated_at_ms
                .unwrap_or(f.valid_from_ms)
                .max(f.valid_from_ms);
            let age = now_ms.saturating_sub(ttl_ref);
            if age <= UNUSED_TTL_MS || age > config.max_age_guardrail_ms {
                false
            } else {
                let score = score_activation(f, now_ms, config);
                score.heat >= config.retention_threshold
            }
        })
        .count();

    let expired_ids = collect_activation_expired(&rows, now_ms, config);
    let mut rows_expired = 0usize;
    for id in &expired_ids {
        match store.soft_close(id, now_ms) {
            Ok(()) => rows_expired += 1,
            Err(MemoryError::NotFound(_)) => {
                tracing::debug!(
                    target = "memory.activation_decay",
                    fact_id = %id,
                    "soft_close NotFound (concurrent close) — benign"
                );
            }
            Err(e) => {
                tracing::warn!(
                    target = "memory.activation_decay",
                    fact_id = %id,
                    err = %e,
                    "soft_close failed (non-fatal)"
                );
            }
        }
    }

    Ok(ActivationDecayStats {
        rows_examined,
        rows_expired,
        rows_retained_past_ttl,
    })
}

// ---------------------------------------------------------------------------
// get_authorized_facts_with_activation_decay — composed retrieval
// ---------------------------------------------------------------------------

/// The result of [`get_authorized_facts_with_activation_decay`]: the
/// activation-re-ranked authorized facts, their activation scores (parallel
/// vec), the Step 2 exclusions, and the Step 3 raw-evidence escalation.
#[derive(Debug, Clone)]
pub struct ActivationRetrievalResult {
    /// Authorized facts re-ranked by activation score (descending).
    pub facts: Vec<MemoryFact>,
    /// Activation scores parallel to `facts`.
    pub scores: Vec<ActivationScore>,
    /// J-21 Step 2 authorization exclusions (stale/quarantined/unknown).
    pub exclusions: Vec<AuthorizationExclusion>,
    /// J-21 Step 3 raw-evidence escalation (fired when fact layer insufficient).
    pub escalation: RawEvidenceEscalation,
}

/// The composed retrieval entry point: J-21 Step 3's
/// [`get_authorized_facts_with_escalation`] + activation-decay re-ranking.
///
/// Stage 1: calls Step 3 to get authorized facts + exclusions + escalation.
/// Stage 2: scores each authorized fact's activation.
/// Stage 3: re-ranks the authorized slice by activation heat (descending) so
/// the prompt receives high-activation (recently-used / recently-revalidated
/// / high-confidence) facts first.
///
/// This is the public API the orchestrator retrieval path calls when the
/// DARK capture+retrieval is un-darked. The H-31 auditor-objectivity
/// firewall is maintained — activation scores are retrieval-ranking
/// metadata, never injected into auditor/critic/adversary contexts.
#[allow(clippy::too_many_arguments)] // J-21 facade: threads Step 3 escalation + activation re-ranking stages.
pub fn get_authorized_facts_with_activation_decay(
    fact_store: &FactStore,
    raw_evidence_store: &RawEvidenceStore,
    namespace: &str,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
    existence_validator: &dyn CitationValidator,
    semantic_validator: &dyn SemanticValidator,
    scope: &ValidationScope,
    quarantine: Option<&mut QuarantineStore>,
    activation_config: &ActivationDecayConfig,
) -> Result<ActivationRetrievalResult, MemoryError> {
    // Stage 1: J-21 Step 3 authorized retrieval + sufficiency-gated escalation.
    let (authorized, exclusions, escalation) = get_authorized_facts_with_escalation(
        fact_store,
        raw_evidence_store,
        namespace,
        user_id,
        filter,
        k,
        now_ms,
        existence_validator,
        semantic_validator,
        scope,
        quarantine,
    )?;

    // Stage 2: score each authorized fact's activation.
    let mut scored: Vec<(MemoryFact, ActivationScore)> = authorized
        .into_iter()
        .map(|f| {
            let s = score_activation(&f, now_ms, activation_config);
            (f, s)
        })
        .collect();

    // Stage 3: re-rank by activation heat (descending). Stable sort keeps
    // the prior last_observed_at_ms order for ties.
    scored.sort_by(|a, b| {
        b.1.heat
            .partial_cmp(&a.1.heat)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let facts: Vec<MemoryFact> = scored.iter().map(|(f, _)| f.clone()).collect();
    let scores: Vec<ActivationScore> = scored.into_iter().map(|(_, s)| s).collect();

    Ok(ActivationRetrievalResult {
        facts,
        scores,
        exclusions,
        escalation,
    })
}

/// Crate-internal convenience wrapper around
/// [`get_authorized_facts_with_activation_decay`] with fail-closed null
/// validators and an empty raw-evidence store. Used by in-crate tests and
/// the future orchestrator wiring shim when the semantic index is
/// unavailable. This wrapper gives
/// [`get_authorized_facts_with_activation_decay`] a same-file non-test
/// caller (the production orchestrator path is currently DARK per the
/// feature map).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)] // thin fail-closed wrapper forwarding to the facade.
fn authorized_retrieval_with_activation_fail_closed(
    fact_store: &FactStore,
    raw_evidence_store: &RawEvidenceStore,
    namespace: &str,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
    now_ms: u128,
    activation_config: &ActivationDecayConfig,
) -> Result<ActivationRetrievalResult, MemoryError> {
    let null_citation = crate::citation_validity::NullValidator;
    let null_semantic = crate::retrieval_authorization::NullSemanticValidator;
    let scope = ValidationScope::at_head("", "");
    get_authorized_facts_with_activation_decay(
        fact_store,
        raw_evidence_store,
        namespace,
        user_id,
        filter,
        k,
        now_ms,
        &null_citation,
        &null_semantic,
        &scope,
        None,
        activation_config,
    )
}

/// Crate-internal convenience wrapper around [`activation_decay_tick`] with
/// the default config. Gives [`activation_decay_tick`] a same-file non-test
/// caller (the production decay-scheduler path is currently DARK per the
/// feature map).
#[allow(dead_code)]
fn activation_decay_tick_fail_closed(
    store: &mut FactStore,
    user_id: &str,
    now_ms: u128,
) -> Result<ActivationDecayStats, MemoryError> {
    activation_decay_tick(store, user_id, now_ms, &ActivationDecayConfig::default())
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fact::{FactType, MemoryFact, Scope};
    use crate::raw_evidence::RawEvidenceStore;
    use crate::trust::{Provenance, TrustTier};

    const T: u128 = 1_700_000_000_000;
    const DKG_PTR: &str = "dkg:validate_jwt@crates/auth/src/lib.rs:42";
    const DAY_MS: u128 = 24 * 60 * 60 * 1_000;

    fn make_tier_a_fact(fact_id: &str, evidence: Vec<String>) -> MemoryFact {
        MemoryFact {
            fact_id: fact_id.into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: Some("proj-1".into()),
            scope: Scope::Project,
            fact_type: FactType::DomainKnowledge,
            key: "no_unwrap_in_prod".into(),
            value: serde_json::json!({ "raw": "no unwrap in prod" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
            evidence_pointers: evidence,
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    fn make_tier_b_fact() -> MemoryFact {
        MemoryFact {
            fact_id: "fact-pref".into(),
            user_id: "u1".into(),
            seat_id: "local".into(),
            project_id: None,
            scope: Scope::Global,
            fact_type: FactType::Preference,
            key: "pref_key".into(),
            value: serde_json::json!({ "raw": "prefers tracing" }),
            confidence: 0.9,
            trust_score: 0.8,
            ingested_at_ms: T,
            valid_from_ms: T,
            valid_to_ms: None,
            decay_rate_per_day: 0.01,
            last_observed_at_ms: T,
            evidence_pointers: vec!["dkg:foo@bar.rs:1".into()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("test"),
            last_validated_at_ms: None,
        }
    }

    // --- score_activation ---

    #[test]
    fn score_is_high_for_recently_used_validated_high_confidence() {
        let mut fact = make_tier_a_fact("f1", vec![DKG_PTR.into()]);
        fact.last_observed_at_ms = T;
        fact.last_validated_at_ms = Some(T);
        fact.confidence = 1.0;
        let config = ActivationDecayConfig::default();
        let s = score_activation(&fact, T, &config);
        assert!((s.recency_of_use - 1.0).abs() < 1e-6, "recent use -> 1.0");
        assert!(
            (s.recency_of_validation - 1.0).abs() < 1e-6,
            "recent validation -> 1.0"
        );
        assert!((s.heat - 1.0).abs() < 1e-6, "all max -> heat 1.0");
    }

    #[test]
    fn score_confidence_only_floor_when_stale_and_unvalidated() {
        let mut fact = make_tier_a_fact("f2", vec![DKG_PTR.into()]);
        // Long elapsed: 1 year since observed, never validated.
        fact.last_observed_at_ms = T;
        fact.last_validated_at_ms = None;
        fact.confidence = 0.9;
        let now = T + 365 * DAY_MS;
        let config = ActivationDecayConfig::default();
        let s = score_activation(&fact, now, &config);
        assert!(s.recency_of_use < 0.01, "year-old use -> ~0");
        assert_eq!(s.recency_of_validation, 0.0, "never validated -> 0");
        // heat = 0.4*~0 + 0.4*0 + 0.2*0.9 = 0.18
        assert!(
            (s.heat - 0.18).abs() < 1e-3,
            "confidence-only floor = 0.2 * confidence, got {}",
            s.heat
        );
    }

    #[test]
    fn score_validation_recency_dominates_for_recently_revalidated() {
        let mut fact = make_tier_a_fact("f3", vec![DKG_PTR.into()]);
        // Not read recently (1 year ago) BUT revalidated recently (1 day ago).
        fact.last_observed_at_ms = T;
        fact.last_validated_at_ms = Some(T + 365 * DAY_MS - DAY_MS);
        fact.confidence = 0.5;
        let now = T + 365 * DAY_MS;
        let config = ActivationDecayConfig::default();
        let s = score_activation(&fact, now, &config);
        assert!(s.recency_of_use < 0.01, "year-old use -> ~0");
        assert!(
            s.recency_of_validation > 0.9,
            "1-day-old validation -> ~1, got {}",
            s.recency_of_validation
        );
        // heat = 0.4*~0 + 0.4*exp(-1/30) + 0.2*0.5 ≈ 0.487
        assert!(
            (s.heat - 0.487).abs() < 1e-2,
            "validation-dominated heat ~0.487, got {}",
            s.heat
        );
    }

    #[test]
    fn score_clamps_to_unit_interval() {
        let mut fact = make_tier_a_fact("f4", vec![DKG_PTR.into()]);
        fact.confidence = 1.0;
        fact.last_observed_at_ms = T;
        fact.last_validated_at_ms = Some(T);
        let config = ActivationDecayConfig::default();
        let s = score_activation(&fact, T, &config);
        assert!((0.0..=1.0).contains(&s.heat));
    }

    // --- collect_activation_expired ---

    #[test]
    fn past_ttl_high_activation_invariant_is_retained() {
        let mut fact = make_tier_a_fact("invariant", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        // Validated at T+100d, now is T+129d → ttl_ref age = 29 days (past
        // the 28-day TTL). But validation recency (30-day half-life) is
        // still 0.381, keeping heat >= 0.3 → RETAINED.
        fact.last_validated_at_ms = Some(T + 100 * DAY_MS);
        fact.last_observed_at_ms = T + 100 * DAY_MS;
        let now = T + 129 * DAY_MS;
        let config = ActivationDecayConfig::default();
        let expired = collect_activation_expired(&[fact.clone()], now, &config);
        assert!(
            expired.is_empty(),
            "past-TTL high-activation invariant must be RETAINED, got expired {:?}",
            expired
        );
    }

    #[test]
    fn past_ttl_low_activation_fact_is_expired() {
        let mut fact = make_tier_a_fact("stale_low", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        fact.last_validated_at_ms = None;
        fact.last_observed_at_ms = T;
        fact.confidence = 0.5;
        let now = T + 40 * DAY_MS; // past 28-day TTL, no recent use/validation
        let config = ActivationDecayConfig::default();
        let expired = collect_activation_expired(&[fact.clone()], now, &config);
        assert_eq!(expired, vec![fact.fact_id]);
    }

    #[test]
    fn max_age_guardrail_expires_regardless_of_activation() {
        let mut fact = make_tier_a_fact("ancient", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        // Never validated → ttl_ref = valid_from_ms = T. Read recently
        // (yesterday) so recency_of_use is high → high activation heat.
        // BUT age from ttl_ref = 200 days > 180-day guardrail → expired
        // regardless of activation.
        fact.last_validated_at_ms = None;
        fact.last_observed_at_ms = T + 200 * DAY_MS - DAY_MS;
        fact.confidence = 1.0;
        let now = T + 200 * DAY_MS;
        let config = ActivationDecayConfig::default();
        // Verify the fact indeed has high activation (the guardrail must
        // override it, not pass because activation is low).
        let score = score_activation(&fact, now, &config);
        assert!(
            score.heat >= 0.3,
            "test precondition: fact must have high activation (heat={}), guardrail overrides it",
            score.heat
        );
        let expired = collect_activation_expired(&[fact.clone()], now, &config);
        assert_eq!(
            expired,
            vec![fact.fact_id.clone()],
            "max-age guardrail must expire even high-activation facts"
        );
    }

    #[test]
    fn within_ttl_fact_is_not_expired() {
        let mut fact = make_tier_a_fact("fresh", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        fact.last_validated_at_ms = None;
        fact.last_observed_at_ms = T;
        let now = T + 10 * DAY_MS; // well within 28-day TTL
        let config = ActivationDecayConfig::default();
        let expired = collect_activation_expired(&[fact], now, &config);
        assert!(expired.is_empty());
    }

    #[test]
    fn tier_b_fact_is_not_expired_by_activation_collector() {
        let mut fact = make_tier_b_fact();
        fact.valid_from_ms = T;
        fact.last_validated_at_ms = None;
        let now = T + 200 * DAY_MS; // past guardrail
        let config = ActivationDecayConfig::default();
        let expired = collect_activation_expired(&[fact], now, &config);
        assert!(
            expired.is_empty(),
            "Tier-B facts are not subject to the Tier-A activation expiry"
        );
    }

    #[test]
    fn closed_fact_is_not_expired_by_activation_collector() {
        let mut fact = make_tier_a_fact("closed", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        fact.valid_to_ms = Some(T + 1000); // already closed
        let now = T + 200 * DAY_MS;
        let config = ActivationDecayConfig::default();
        let expired = collect_activation_expired(&[fact], now, &config);
        assert!(expired.is_empty());
    }

    // --- activation_decay_tick ---

    #[test]
    fn tick_soft_closes_low_activation_past_ttl_facts() {
        let mut store = FactStore::open_in_memory().expect("open store");
        let mut fact = make_tier_a_fact("tick-stale", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        fact.last_validated_at_ms = None;
        fact.last_observed_at_ms = T;
        fact.confidence = 0.5;
        store.insert(&fact).expect("insert");

        let now = T + 40 * DAY_MS;
        let stats = activation_decay_tick(&mut store, "u1", now, &ActivationDecayConfig::default())
            .expect("tick");
        assert_eq!(stats.rows_expired, 1);
        assert_eq!(stats.rows_retained_past_ttl, 0);
        // The fact should now be soft-closed (valid_to_ms set).
        let reloaded = store.get(&fact.fact_id).expect("get").expect("present");
        assert!(reloaded.valid_to_ms.is_some(), "fact was soft-closed");
    }

    #[test]
    fn tick_retains_high_activation_past_ttl_facts() {
        let mut store = FactStore::open_in_memory().expect("open store");
        let mut fact = make_tier_a_fact("tick-invariant", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        // Validated at T+100d, now = T+129d → past 28-day TTL but high
        // activation (validation recency 0.381, heat ~0.383 >= 0.3).
        fact.last_validated_at_ms = Some(T + 100 * DAY_MS);
        fact.last_observed_at_ms = T + 100 * DAY_MS;
        store.insert(&fact).expect("insert");

        let now = T + 129 * DAY_MS;
        let stats = activation_decay_tick(&mut store, "u1", now, &ActivationDecayConfig::default())
            .expect("tick");
        assert_eq!(stats.rows_expired, 0, "high-activation fact retained");
        assert_eq!(stats.rows_retained_past_ttl, 1);
        let reloaded = store.get(&fact.fact_id).expect("get").expect("present");
        assert!(reloaded.valid_to_ms.is_none(), "fact was NOT soft-closed");
    }

    // --- composed retrieval (fail-closed wrapper wiring) ---

    #[test]
    fn fail_closed_retrieval_ranks_by_activation() {
        let store = FactStore::open_in_memory().expect("open store");
        let raw_store = RawEvidenceStore::new();
        // Tier-B fact (short-circuits to authorized) — recently observed.
        let mut pref = make_tier_b_fact();
        pref.last_observed_at_ms = T + 1000;
        store.insert(&pref).expect("insert pref");

        let filter = RetrievalFilter::default();
        let result = authorized_retrieval_with_activation_fail_closed(
            &store,
            &raw_store,
            "ns1",
            "u1",
            &filter,
            10,
            T + 2000,
            &ActivationDecayConfig::default(),
        )
        .expect("fail-closed retrieval");

        assert_eq!(result.facts.len(), 1);
        assert_eq!(result.scores.len(), 1);
        // Tier-B recently observed -> high recency_of_use.
        assert!(
            result.scores[0].recency_of_use > 0.9,
            "recently-observed Tier-B should have high recency_of_use"
        );
    }

    #[test]
    fn fail_closed_decay_tick_runs() {
        let mut store = FactStore::open_in_memory().expect("open store");
        let mut fact = make_tier_a_fact("fc-tick", vec![DKG_PTR.into()]);
        fact.valid_from_ms = T;
        fact.last_validated_at_ms = None;
        fact.last_observed_at_ms = T;
        fact.confidence = 0.4;
        store.insert(&fact).expect("insert");

        let now = T + 40 * DAY_MS;
        let stats = activation_decay_tick_fail_closed(&mut store, "u1", now).expect("tick");
        assert_eq!(stats.rows_expired, 1);
    }
}
