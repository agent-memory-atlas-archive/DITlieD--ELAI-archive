//! H-30: staleness / intent-drift INSTRUMENT runner (the measure-before-trust
//! keystone for the durable-user-memory write/read path).
//!
//! Drives the SHIPPED bi-temporal memory substrate
//! ([`elai_memory::FactStore`] + [`elai_memory::audit_conflicts`] +
//! [`elai_memory::get_relevant_facts`]) over a labeled supersession fixture and
//! computes TWO grounded rates plus a negative-control arm. NO model, NO
//! sandbox, NO live inference, NO live store — fully deterministic and offline.
//! The eval opens an ISOLATED in-memory `FactStore` per case (never the live
//! store, never `/remember`), so it INSTRUMENTS the wire-dark
//! `get_relevant_facts` read-path WITHOUT activating it for production (H-32
//! wires the live prompt path later).
//!
//! # The two grounded rates (STALE arXiv:2605.06527 metric definitions)
//!
//! Per fixture case `(X superseded by X')`:
//!   * **SR (State Resolution = RECOGNITION)**: did the supersession path
//!     RECOGNIZE that X is superseded? i.e. after driving
//!     [`elai_memory::audit_conflicts`] the stale row is closed
//!     (`valid_to_ms.is_some()`). STALE: "directly tests whether the model
//!     recognizes that a prior belief is no longer valid."
//!   * **IPA (Implicit Policy Adaptation = APPLICATION)**: did RETRIEVAL
//!     actually FILTER X and serve X'? i.e. [`elai_memory::get_relevant_facts`]
//!     returns X' and NEVER returns X. STALE: "whether the updated state is
//!     integrated into realistic downstream behavior."
//!
//! INVARIANT (a contract test): `sr_rate >= ipa_rate` always. Application
//! cannot exceed recognition — a fact the store never recognized as stale
//! cannot be filtered on the application side. STALE Finding 1 (verbatim):
//! "Recognition does not imply application." If `ipa_rate > sr_rate` the
//! fixture is misconfigured.
//!
//! # CITATION DISCIPLINE (this project has a history of fabricated cites)
//!
//! - The recognition-vs-application gap (best-in-class still leaks: SR 76.0% /
//!   IPA 39.0%) is **Qwen3.5-27B Type-I-specific**, NOT a general-population
//!   average (STALE Table; Gemini-3.1-pro is 92.0% / 71.0%). We never present a
//!   literature figure as ELAI's own number — ELAI's SR/IPA are COMPUTED
//!   per-run from the fixture outcomes (HD-1 / GD-2).
//! - Bi-temporal soft-close semantics (a superseded fact's `t_invalid` is set
//!   to the `t_valid` of the superseding fact; non-destructive, never deleted)
//!   are grounded in Zep/Graphiti arXiv:2501.13956 §2.2.3. We do NOT attach a
//!   token-percentage to the hard-filter-not-narrate design: the "+68% token
//!   narration trap" number is FABRICATED (verified absent from 2501.13956) and
//!   is never cited here.
//! - BeliefShift arXiv:2603.23848 grounds the belief-revision CONCEPT
//!   (present X, then superseding X', measure the system uses X' not X) only;
//!   this single-turn labeled probe is NOT a replica of BeliefShift's
//!   longitudinal multi-session vector benchmark.
//!
//! # The negative control (R2: the dead-evidence path must SCREAM)
//!
//! The `--broken-retrieval` arm replaces the structural hard-filter with a
//! retriever that READS superseded rows too (via the read-only
//! [`FactStore::list_including_closed`] accessor — no write, no DELETE, no
//! UPDATE). With the filter broken, the recognized-but-stale row LEAKS into the
//! returned set: `stale_fact_reached_prompt > 0` and `ipa_rate` DROPS below
//! `sr_rate`. R2 DOCTRINE: a `--broken-retrieval` run that STILL reports
//! `stale_fact_reached_prompt == 0` is itself an INSTRUMENT FAILURE — an
//! instrument that cannot detect a broken retriever is useless. The runner
//! flags `instrument_failed = true` in that case so callers fail closed.
//!
//! # Wiring discipline
//!
//! - blake3 for the corpus content hash (anti-stale rule).
//! - thiserror for errors; tracing for diagnostics.
//! - serde for the embedded corpus + the report.
//! - No `.unwrap()` / `.expect()` outside `#[cfg(test)]`. No silent swallows.
//! - The corpus is embedded at compile time via `include_str!` so a shipped
//!   binary is filesystem-independent (mirrors `seccodeplt_runner`).
//! - NO `SystemTime` — `now_ms` is a deterministic seed-derived monotonic
//!   counter so the read-bump in `get_relevant_facts` cannot drift the report.

use blake3::Hasher;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use elai_memory::{
    audit_conflicts, get_relevant_facts, FactStore, FactType, MemoryError, MemoryFact, Provenance,
    RetrievalFilter, Scope, TrustTier,
};

use crate::determinism::ELAI_SEED_ENV;

// ---------------------------------------------------------------------------
// Embedded corpus
// ---------------------------------------------------------------------------

/// The labeled supersession fixture corpus, embedded at compile time. One JSON
/// array of [`StalenessCase`] objects. Filesystem-independent.
const CORPUS_JSON: &str = include_str!("staleness_fixtures/corpus.json");

/// Suite label recorded in the report + the SSB beacon.
pub const SUITE_LABEL: &str = "staleness_supersession_fixture";

/// Single-user install id used for every fixture row (the fixtures store WORK
/// FACTS only — never a user persona; NOT-A-PERSONAL-AGENT).
const FIXTURE_USER_ID: &str = "default";

/// Project id used for every fixture row.
const FIXTURE_PROJECT_ID: &str = "h30-fixture";

/// Retrieval top-K. Each case has at most one fresh row + (broken arm) one
/// stale row, so 8 leaves generous headroom while staying bounded.
const RETRIEVAL_K: usize = 8;

/// Base timestamp (ms) for the first fixture write. A fixed literal so the run
/// is byte-reproducible — NEVER `SystemTime::now()`.
const FIXTURE_EPOCH_MS: u128 = 1_700_000_000_000;

/// Monotonic step (ms) applied between successive store operations so
/// `valid_from < valid_to` and `last_observed` ordering is deterministic.
const FIXTURE_STEP_MS: u128 = 1_000;

/// Fixed trust score for every fixture row — above the `Preference` retrieval
/// floor (0.5) so a row is never dropped for the WRONG reason (low trust).
const FIXTURE_TRUST_SCORE: f32 = 0.9;

/// Fixed confidence for every fixture row.
const FIXTURE_CONFIDENCE: f32 = 0.9;

/// Decay rate per day for fixture rows (irrelevant to the eval; the eval never
/// runs a decay tick, but the field is required).
const FIXTURE_DECAY_RATE: f32 = 0.0;

// ---------------------------------------------------------------------------
// Corpus case
// ---------------------------------------------------------------------------

/// One labeled supersession case. States a stale fact `X` (key + stale_value),
/// a superseding fact `X'` (same key, fresh_value), the per-row trust tiers
/// (the tier ordering drives whether the tier adjudicator RECOGNIZES the
/// supersession), and `expect_recognized` (the label: should the adjudicator
/// close the stale row?).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StalenessCase {
    /// Stable case id (e.g. `"stale-pref-test-runner-001"`).
    pub id: String,
    /// Fact type wire tag (e.g. `"preference"`).
    pub fact_type: String,
    /// Shared fact key (e.g. `"test_runner"`).
    pub key: String,
    /// The stale value `X` (e.g. `"cargo nextest"`).
    pub stale_value: String,
    /// The fresh value `X'` (e.g. `"cargo test"`).
    pub fresh_value: String,
    /// Trust-tier wire tag for the stale row (e.g. `"T3"`).
    pub stale_tier: String,
    /// Trust-tier wire tag for the fresh row (e.g. `"T0"`).
    pub fresh_tier: String,
    /// Non-empty evidence pointers for the stale row (provenance enforcement).
    pub stale_evidence: Vec<String>,
    /// Non-empty evidence pointers for the fresh row.
    pub fresh_evidence: Vec<String>,
    /// The label: should the tier-driven adjudicator RECOGNIZE this conflict
    /// and soft-close the stale row? `false` is the honest SR<1.0 case (the
    /// fresh row is not strictly higher tier, so both rows survive).
    pub expect_recognized: bool,
    /// Free-text rationale (documentation only; not scored).
    #[serde(default)]
    pub note: String,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum StalenessEvalError {
    /// The embedded corpus JSON failed to deserialise (build-time authoring bug
    /// surfaced at run time).
    #[error("staleness corpus malformed json: {0}")]
    MalformedCorpus(String),
    /// The corpus is empty (would make every rate degenerate / NaN).
    #[error("staleness corpus is empty")]
    EmptyCorpus,
    /// A fixture case carried an unparseable fact_type / tier wire tag.
    #[error("staleness fixture case '{id}' invalid: {detail}")]
    InvalidCase { id: String, detail: String },
    /// `ELAI_SEED` was set but not parseable as a u64.
    #[error("staleness invalid ELAI_SEED '{value}': {detail}")]
    InvalidSeed { value: String, detail: String },
    /// The isolated fixture store could not be opened / driven. Bubbled from
    /// the memory substrate.
    #[error("staleness fixture store error: {0}")]
    Store(#[from] MemoryError),
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

/// Aggregate report — one per [`StalenessEvalRunner::run`] invocation.
///
/// All rates are in `[0.0, 1.0]` and COMPUTED from the fixture outcomes, never
/// literals. `sr_rate >= ipa_rate` always holds (the contract asserts it).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StalenessEvalReport {
    /// Suite label ([`SUITE_LABEL`]).
    pub suite: String,
    /// Number of fixture cases evaluated.
    pub n_cases: u32,
    /// State Resolution (recognition) rate: cases the supersession path
    /// recognized as stale (stale row closed) / n_cases.
    pub sr_rate: f64,
    /// Implicit Policy Adaptation (application) rate: cases retrieval served
    /// the fresh value AND never served the stale value / n_cases.
    pub ipa_rate: f64,
    /// `sr_rate - ipa_rate` — the recognition-vs-application gap (the STALE
    /// failure mode: recognized-but-still-served).
    pub recognition_application_gap: f64,
    /// Count of returned rows across ALL retrievals that were superseded /
    /// closed (`valid_to_ms.is_some()` OR `superseded_by.is_some()`). 0 on the
    /// clean path by construction; > 0 when the retriever is broken.
    pub stale_fact_reached_prompt: u64,
    /// Total retrieval calls across the run (one per case).
    pub total_retrievals: u64,
    /// `true` when this run drove the negative-control (broken) retriever.
    pub broken_arm: bool,
    /// R2 self-test: `true` IFF this was a broken-arm run that STILL reported
    /// `stale_fact_reached_prompt == 0` — i.e. the instrument failed to scream.
    /// Always `false` on the clean arm.
    pub instrument_failed: bool,
    /// Deterministic RNG seed used to assemble the run (`ELAI_SEED`, default 0).
    pub seed: u64,
    /// blake3 hex of the canonical corpus bytes (provenance / staleness pin).
    pub corpus_blake3: String,
}

// ---------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------

/// H-30 staleness instrument runner. Construction parses + validates the
/// embedded corpus once; [`StalenessEvalRunner::run`] drives the SHIPPED memory
/// substrate over each case in an isolated in-memory store and computes the
/// report. Deterministic: same corpus + same `broken_arm` => byte-identical
/// report.
#[derive(Debug)]
pub struct StalenessEvalRunner {
    cases: Vec<StalenessCase>,
    corpus_blake3: String,
    seed: u64,
    broken_arm: bool,
}

impl StalenessEvalRunner {
    /// Construct from the embedded corpus, reading `ELAI_SEED` (default 0).
    /// `broken_arm` selects the negative-control retriever.
    pub fn from_embedded(broken_arm: bool) -> Result<Self, StalenessEvalError> {
        let seed = read_seed()?;
        Self::from_json_with_seed(CORPUS_JSON, seed, broken_arm)
    }

    /// Construct from explicit corpus JSON bytes + an explicit seed. Used by the
    /// unit + contract tests; production uses [`Self::from_embedded`].
    pub fn from_json_with_seed(
        json: &str,
        seed: u64,
        broken_arm: bool,
    ) -> Result<Self, StalenessEvalError> {
        let cases: Vec<StalenessCase> = serde_json::from_str(json)
            .map_err(|e| StalenessEvalError::MalformedCorpus(e.to_string()))?;
        if cases.is_empty() {
            return Err(StalenessEvalError::EmptyCorpus);
        }
        // Validate each case's wire tags up front so run() cannot half-drive a
        // malformed fixture.
        for c in &cases {
            FactType::from_str(&c.fact_type).map_err(|e| StalenessEvalError::InvalidCase {
                id: c.id.clone(),
                detail: format!("fact_type '{}': {e}", c.fact_type),
            })?;
            TrustTier::from_str(&c.stale_tier).map_err(|e| StalenessEvalError::InvalidCase {
                id: c.id.clone(),
                detail: format!("stale_tier '{}': {e}", c.stale_tier),
            })?;
            TrustTier::from_str(&c.fresh_tier).map_err(|e| StalenessEvalError::InvalidCase {
                id: c.id.clone(),
                detail: format!("fresh_tier '{}': {e}", c.fresh_tier),
            })?;
            if c.stale_evidence.is_empty() || c.fresh_evidence.is_empty() {
                return Err(StalenessEvalError::InvalidCase {
                    id: c.id.clone(),
                    detail: "evidence pointers must be non-empty (provenance enforcement)".into(),
                });
            }
        }
        let mut hasher = Hasher::new();
        hasher.update(json.as_bytes());
        let corpus_blake3 = hasher.finalize().to_hex().to_string();
        Ok(Self {
            cases,
            corpus_blake3,
            seed,
            broken_arm,
        })
    }

    /// Number of fixture cases.
    #[must_use]
    pub fn case_count(&self) -> usize {
        self.cases.len()
    }

    /// The recorded seed.
    #[must_use]
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Whether this runner drives the broken (negative-control) retriever.
    #[must_use]
    pub fn is_broken_arm(&self) -> bool {
        self.broken_arm
    }

    /// blake3 hex of the canonical corpus bytes.
    #[must_use]
    pub fn corpus_blake3(&self) -> &str {
        &self.corpus_blake3
    }

    /// Borrow the parsed cases (for calibration assertions).
    #[must_use]
    pub fn cases(&self) -> &[StalenessCase] {
        &self.cases
    }

    /// Build the stale (X) [`MemoryFact`] for a case at a given write time.
    fn build_stale_fact(case: &StalenessCase, now_ms: u128) -> Result<MemoryFact, StalenessEvalError> {
        let fact_type = FactType::from_str(&case.fact_type).map_err(map_case_err(case))?;
        let tier = TrustTier::from_str(&case.stale_tier).map_err(map_case_err(case))?;
        Ok(MemoryFact {
            fact_id: format!("{}-stale", case.id),
            user_id: FIXTURE_USER_ID.to_string(),
            project_id: Some(FIXTURE_PROJECT_ID.to_string()),
            scope: Scope::Project,
            fact_type,
            key: case.key.clone(),
            value: serde_json::Value::String(case.stale_value.clone()),
            confidence: FIXTURE_CONFIDENCE,
            trust_score: FIXTURE_TRUST_SCORE,
            ingested_at_ms: now_ms,
            valid_from_ms: now_ms,
            valid_to_ms: None,
            decay_rate_per_day: FIXTURE_DECAY_RATE,
            last_observed_at_ms: now_ms,
            evidence_pointers: case.stale_evidence.clone(),
            superseded_by: None,
            trust_tier: tier,
            provenance: Provenance {
                source: "h30_fixture_stale".into(),
                tier,
            },
            last_validated_at_ms: None,
        })
    }

    /// Build the fresh (X') [`MemoryFact`] for a case at a given write time.
    fn build_fresh_fact(case: &StalenessCase, now_ms: u128) -> Result<MemoryFact, StalenessEvalError> {
        let fact_type = FactType::from_str(&case.fact_type).map_err(map_case_err(case))?;
        let tier = TrustTier::from_str(&case.fresh_tier).map_err(map_case_err(case))?;
        Ok(MemoryFact {
            fact_id: format!("{}-fresh", case.id),
            user_id: FIXTURE_USER_ID.to_string(),
            project_id: Some(FIXTURE_PROJECT_ID.to_string()),
            scope: Scope::Project,
            fact_type,
            key: case.key.clone(),
            value: serde_json::Value::String(case.fresh_value.clone()),
            confidence: FIXTURE_CONFIDENCE,
            trust_score: FIXTURE_TRUST_SCORE,
            ingested_at_ms: now_ms,
            valid_from_ms: now_ms,
            valid_to_ms: None,
            decay_rate_per_day: FIXTURE_DECAY_RATE,
            last_observed_at_ms: now_ms,
            evidence_pointers: case.fresh_evidence.clone(),
            superseded_by: None,
            trust_tier: tier,
            provenance: Provenance {
                source: "h30_fixture_fresh".into(),
                tier,
            },
            last_validated_at_ms: None,
        })
    }

    /// Drive ONE fixture case in an isolated in-memory store and return its
    /// per-case outcome. Pure of wall-clock / RNG: `now_ms` is the
    /// caller-threaded deterministic counter.
    fn run_case(&self, case: &StalenessCase, t0: u128) -> Result<CaseOutcome, StalenessEvalError> {
        // Isolated store per case — never the live store, never /remember.
        let mut store = FactStore::open_in_memory()?;

        // 1. Insert the stale fact X.
        let stale = Self::build_stale_fact(case, t0)?;
        store.insert(&stale)?;

        // 2. Insert the contradicting fresh fact X' (same key, later valid_from)
        //    so audit_conflicts sees a (older, newer) pair on the shared key.
        let t_fresh = t0 + FIXTURE_STEP_MS;
        let fresh = Self::build_fresh_fact(case, t_fresh)?;
        store.insert(&fresh)?;

        // 3. Drive the SHIPPED tier-driven adjudicator (REUSE, GD-1). With
        //    auto_supersede_by_tier=true the older row is soft-closed IFF the
        //    newer row's tier is strictly higher. This is the RECOGNITION path.
        let fact_type = FactType::from_str(&case.fact_type).map_err(map_case_err(case))?;
        let t_audit = t0 + 2 * FIXTURE_STEP_MS;
        let _actions = audit_conflicts(&mut store, FIXTURE_USER_ID, fact_type, true, t_audit)?;

        // SR (recognition): did the store close the stale row?
        let stale_row = store.get(&stale.fact_id)?;
        let recognized = stale_row
            .as_ref()
            .map(|f| f.valid_to_ms.is_some() || f.superseded_by.is_some())
            .unwrap_or(false);

        // 4. RETRIEVAL — the read-path being instrumented. Clean arm uses the
        //    shipped hard-filtered get_relevant_facts; broken arm bypasses the
        //    structural filter via the read-only include-closed accessor.
        let filter = RetrievalFilter {
            project_id: Some(FIXTURE_PROJECT_ID.to_string()),
            fact_type: Some(fact_type),
            trust_override: None,
            safety_critical: false,
            citation_validator: None,
        };
        let t_retrieve = t0 + 3 * FIXTURE_STEP_MS;
        let returned: Vec<MemoryFact> = if self.broken_arm {
            broken_retrieve(&store, FIXTURE_USER_ID, &filter, RETRIEVAL_K)?
        } else {
            get_relevant_facts(&store, FIXTURE_USER_ID, &filter, RETRIEVAL_K, t_retrieve)
                .map(|(facts, _)| facts)?
        };

        // IPA (application): retrieval served the fresh value AND never the
        // stale value for this key.
        let served_fresh = returned
            .iter()
            .any(|f| f.key == case.key && f.value == fresh.value);
        let served_stale = returned
            .iter()
            .any(|f| f.key == case.key && f.value == stale.value);
        let applied = served_fresh && !served_stale;

        // stale_fact_reached_prompt: count superseded/closed rows that leaked
        // into the returned set (the regression the metric counts).
        let stale_reached = returned
            .iter()
            .filter(|f| f.valid_to_ms.is_some() || f.superseded_by.is_some())
            .count();
        let stale_reached_u64 = u64::try_from(stale_reached).unwrap_or(u64::MAX);

        debug!(
            case = %case.id,
            recognized,
            applied,
            served_fresh,
            served_stale,
            stale_reached,
            broken_arm = self.broken_arm,
            "staleness: case scored",
        );
        if self.broken_arm && stale_reached == 0 {
            // Diagnostic only; the aggregate R2 self-test decides instrument_failed.
            warn!(case = %case.id, "staleness: broken arm produced no stale leak for this case");
        }

        Ok(CaseOutcome {
            recognized,
            applied,
            stale_reached: stale_reached_u64,
        })
    }

    /// Run the eval over every fixture case and compute the report.
    ///
    /// Deterministic: same corpus + same `broken_arm` => byte-identical report.
    /// The SR/IPA rates are computed from the per-case outcomes, never literals.
    pub fn run(&self) -> Result<StalenessEvalReport, StalenessEvalError> {
        let n = self.cases.len();
        info!(
            n_cases = n,
            broken_arm = self.broken_arm,
            "staleness: starting supersession-fixture run",
        );

        let mut recognized_total: u64 = 0;
        let mut applied_total: u64 = 0;
        let mut stale_reached_total: u64 = 0;

        for (idx, case) in self.cases.iter().enumerate() {
            // Deterministic, case-disjoint base time so stores never interleave
            // timestamps across cases.
            let idx_u128 = u128::try_from(idx).unwrap_or(0);
            let t0 = FIXTURE_EPOCH_MS + idx_u128 * 16 * FIXTURE_STEP_MS;
            let outcome = self.run_case(case, t0)?;
            if outcome.recognized {
                recognized_total += 1;
            }
            if outcome.applied {
                applied_total += 1;
            }
            stale_reached_total = stale_reached_total.saturating_add(outcome.stale_reached);
        }

        let n_u32 = u32::try_from(n).unwrap_or(u32::MAX);
        let n_f = f64::from(n_u32);
        // n >= 1 (non-empty corpus), so no /0.
        let sr_rate = (recognized_total as f64) / n_f;
        let ipa_rate = (applied_total as f64) / n_f;
        let recognition_application_gap = sr_rate - ipa_rate;

        // R2 self-test: a broken-arm run that fails to leak ANY stale row is
        // an instrument failure (the dead-evidence path did NOT scream).
        let instrument_failed = self.broken_arm && stale_reached_total == 0;

        let report = StalenessEvalReport {
            suite: SUITE_LABEL.to_string(),
            n_cases: n_u32,
            sr_rate,
            ipa_rate,
            recognition_application_gap,
            stale_fact_reached_prompt: stale_reached_total,
            total_retrievals: n as u64,
            broken_arm: self.broken_arm,
            instrument_failed,
            seed: self.seed,
            corpus_blake3: self.corpus_blake3.clone(),
        };

        info!(
            sr_rate = report.sr_rate,
            ipa_rate = report.ipa_rate,
            gap = report.recognition_application_gap,
            stale_fact_reached_prompt = report.stale_fact_reached_prompt,
            broken_arm = report.broken_arm,
            instrument_failed = report.instrument_failed,
            "staleness: run complete",
        );
        if report.instrument_failed {
            warn!("staleness: BROKEN-ARM RUN REPORTED ZERO STALE LEAK -- the instrument failed to scream (R2 dead-evidence-path failure)");
        }
        Ok(report)
    }
}

// ---------------------------------------------------------------------------
// Per-case outcome
// ---------------------------------------------------------------------------

/// One fixture case's measured outcome.
#[derive(Debug, Clone, Copy)]
struct CaseOutcome {
    /// SR: the supersession path recognized + closed the stale row.
    recognized: bool,
    /// IPA: retrieval served the fresh value and never the stale value.
    applied: bool,
    /// Superseded/closed rows that leaked into the returned set for this case.
    stale_reached: u64,
}

// ---------------------------------------------------------------------------
// The broken (negative-control) retriever — R2
// ---------------------------------------------------------------------------

/// The DELIBERATELY-BROKEN retriever: it mirrors the shape of
/// [`get_relevant_facts`] (project + fact-type scoping, top-K) BUT reads from
/// the read-only include-closed accessor [`FactStore::list_including_closed`]
/// instead of the hard-filtered `list_current`. So superseded / closed rows are
/// NOT structurally excluded — a recognized-stale row LEAKS into the returned
/// set. This makes `stale_fact_reached_prompt > 0`, proving the metric is not
/// vacuously 0 (R2). NO write, NO DELETE, NO UPDATE — read-only.
fn broken_retrieve(
    store: &FactStore,
    user_id: &str,
    filter: &RetrievalFilter,
    k: usize,
) -> Result<Vec<MemoryFact>, StalenessEvalError> {
    let candidate_limit = k.saturating_mul(4).max(64);
    let mut rows = store.list_including_closed(user_id, filter.fact_type, candidate_limit)?;
    // Apply the SAME scope + trust + provenance filters the real retriever
    // applies, so the ONLY difference is the missing valid_to_ms hard-filter
    // (a faithful negative control — it breaks exactly one property).
    rows.retain(|f| match (f.scope, &filter.project_id) {
        (Scope::Global, _) => true,
        (Scope::Project, None) => true,
        (Scope::Project, Some(want)) => f.project_id.as_deref() == Some(want.as_str()),
    });
    rows.retain(|f| {
        let threshold = filter
            .trust_override
            .unwrap_or_else(|| f.fact_type.retrieval_threshold());
        f.trust_score >= threshold
    });
    rows.retain(|f| !f.evidence_pointers.is_empty());
    rows.sort_by(|a, b| b.last_observed_at_ms.cmp(&a.last_observed_at_ms));
    rows.truncate(k);
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Map a `MemoryError` from a per-case wire-tag parse into the typed
/// `InvalidCase` error, capturing the offending case id.
fn map_case_err(case: &StalenessCase) -> impl Fn(MemoryError) -> StalenessEvalError + '_ {
    move |e| StalenessEvalError::InvalidCase {
        id: case.id.clone(),
        detail: e.to_string(),
    }
}

/// Read `ELAI_SEED` (default 0). Errors only when the var is set but not a
/// valid u64 — an empty / unset var is the default-0 path.
fn read_seed() -> Result<u64, StalenessEvalError> {
    match std::env::var(ELAI_SEED_ENV) {
        Ok(raw) if !raw.trim().is_empty() => {
            raw.trim()
                .parse::<u64>()
                .map_err(|e| StalenessEvalError::InvalidSeed {
                    value: raw.clone(),
                    detail: e.to_string(),
                })
        }
        _ => Ok(0u64),
    }
}

// ---------------------------------------------------------------------------
// Tests + calibration
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_runner() -> StalenessEvalRunner {
        StalenessEvalRunner::from_json_with_seed(CORPUS_JSON, 0, false)
            .expect("embedded corpus parses (clean)")
    }

    fn broken_runner() -> StalenessEvalRunner {
        StalenessEvalRunner::from_json_with_seed(CORPUS_JSON, 0, true)
            .expect("embedded corpus parses (broken)")
    }

    #[test]
    fn corpus_has_cases_and_required_fields() {
        let r = clean_runner();
        assert!(r.case_count() >= 2, "corpus must have >=2 cases");
        for c in r.cases() {
            assert!(!c.id.is_empty(), "id required");
            assert!(!c.key.is_empty(), "key required");
            assert!(!c.stale_value.is_empty(), "stale_value required");
            assert!(!c.fresh_value.is_empty(), "fresh_value required");
            assert_ne!(c.stale_value, c.fresh_value, "stale and fresh must differ for {}", c.id);
            assert!(!c.stale_evidence.is_empty(), "stale_evidence required for {}", c.id);
            assert!(!c.fresh_evidence.is_empty(), "fresh_evidence required for {}", c.id);
            // wire tags must parse
            FactType::from_str(&c.fact_type).expect("fact_type parses");
            TrustTier::from_str(&c.stale_tier).expect("stale_tier parses");
            TrustTier::from_str(&c.fresh_tier).expect("fresh_tier parses");
        }
    }

    #[test]
    fn case_ids_are_unique() {
        let r = clean_runner();
        let mut ids: Vec<&str> = r.cases().iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "case ids must be unique");
    }

    #[test]
    fn corpus_has_at_least_one_recognized_and_one_unrecognized_case() {
        // Non-degeneracy: SR must be measurable as a fraction in (0,1). The
        // unrecognized case keeps SR < 1.0 (honest); the recognized cases keep
        // SR > 0.0.
        let r = clean_runner();
        let recognized = r.cases().iter().filter(|c| c.expect_recognized).count();
        let unrecognized = r.cases().iter().filter(|c| !c.expect_recognized).count();
        assert!(recognized >= 1, "need >=1 expect_recognized case");
        assert!(unrecognized >= 1, "need >=1 unrecognized case (SR<1.0, honest)");
    }

    #[test]
    fn clean_arm_stale_reached_is_zero_and_sr_ge_ipa() {
        let r = clean_runner();
        let rep = r.run().expect("clean run");
        assert!(!rep.broken_arm, "clean arm");
        assert!(!rep.instrument_failed, "clean arm never instrument_failed");
        assert_eq!(
            rep.stale_fact_reached_prompt, 0,
            "clean path: the list_current hard-filter must keep stale rows out of the prompt"
        );
        // The grounded invariant: application cannot exceed recognition.
        assert!(
            rep.sr_rate >= rep.ipa_rate,
            "SR ({}) must be >= IPA ({}) -- application cannot exceed recognition",
            rep.sr_rate,
            rep.ipa_rate
        );
        // Rates are sane fractions.
        assert!((0.0..=1.0).contains(&rep.sr_rate), "sr_rate in [0,1]");
        assert!((0.0..=1.0).contains(&rep.ipa_rate), "ipa_rate in [0,1]");
        // Non-degenerate: at least one case recognized + applied.
        assert!(rep.sr_rate > 0.0, "sr_rate must be > 0 (>=1 recognized case)");
        assert!(rep.ipa_rate > 0.0, "ipa_rate must be > 0 (>=1 applied case)");
        // The unrecognized case keeps SR strictly below 1.0.
        assert!(rep.sr_rate < 1.0, "sr_rate must be < 1.0 (the honest unrecognized case)");
    }

    #[test]
    fn broken_arm_screams_stale_reached_gt_zero_and_ipa_drops() {
        let clean = clean_runner().run().expect("clean run");
        let broken = broken_runner().run().expect("broken run");
        assert!(broken.broken_arm, "broken arm flag set");
        assert!(
            broken.stale_fact_reached_prompt > 0,
            "broken retriever MUST leak >=1 superseded row (R2: the instrument screams)"
        );
        // The broken arm must NOT pass the R2 self-test as a failure.
        assert!(
            !broken.instrument_failed,
            "broken arm leaked rows, so instrument_failed must stay false"
        );
        // Application drops below the clean arm because the stale row is served.
        assert!(
            broken.ipa_rate <= clean.ipa_rate,
            "broken IPA ({}) must be <= clean IPA ({})",
            broken.ipa_rate,
            clean.ipa_rate
        );
        // SR is unchanged by a broken RETRIEVER (recognition is write-side).
        assert!(
            (broken.sr_rate - clean.sr_rate).abs() < 1e-12,
            "SR is recognition-side and must be identical across arms"
        );
    }

    #[test]
    fn r2_self_test_a_broken_arm_that_reports_zero_is_an_instrument_failure() {
        // Drive the broken arm over a corpus that contains ONLY an unrecognized
        // case (no stale row is ever closed, so nothing to leak). The broken
        // retriever then reports stale_reached==0, and that MUST be flagged as
        // an instrument failure (the dead-evidence path must scream).
        let only_unrecognized = r#"[
          {
            "id": "only-unrec-001",
            "fact_type": "preference",
            "key": "k",
            "stale_value": "a",
            "fresh_value": "b",
            "stale_tier": "T0",
            "fresh_tier": "T0",
            "stale_evidence": ["e1"],
            "fresh_evidence": ["e2"],
            "expect_recognized": false
          }
        ]"#;
        let runner = StalenessEvalRunner::from_json_with_seed(only_unrecognized, 0, true)
            .expect("parse");
        let rep = runner.run().expect("run");
        assert_eq!(
            rep.stale_fact_reached_prompt, 0,
            "no row was recognized-stale, so even the broken retriever leaks nothing"
        );
        assert!(
            rep.instrument_failed,
            "a broken-arm run reporting stale_reached==0 MUST be flagged instrument_failed (R2)"
        );
    }

    #[test]
    fn run_is_deterministic_byte_identical() {
        let r = clean_runner();
        let a = r.run().expect("run a");
        let b = r.run().expect("run b");
        assert_eq!(a, b, "two clean runs over the same corpus must be byte-identical");
        let rb = broken_runner();
        let c = rb.run().expect("run c");
        let d = rb.run().expect("run d");
        assert_eq!(c, d, "two broken runs over the same corpus must be byte-identical");
    }

    #[test]
    fn seed_round_trips_into_report() {
        let r = StalenessEvalRunner::from_json_with_seed(CORPUS_JSON, 1234, false).expect("parse");
        let rep = r.run().expect("run");
        assert_eq!(rep.seed, 1234, "seed round-trips into the report");
        assert_eq!(rep.suite, SUITE_LABEL);
    }

    #[test]
    fn corpus_blake3_is_64_hex_chars() {
        let r = clean_runner();
        assert_eq!(r.corpus_blake3().len(), 64, "blake3 hex digest is 64 chars");
        assert!(r.corpus_blake3().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn empty_corpus_is_rejected() {
        match StalenessEvalRunner::from_json_with_seed("[]", 0, false) {
            Err(StalenessEvalError::EmptyCorpus) => {}
            other => panic!("expected EmptyCorpus, got {other:?}"),
        }
    }

    #[test]
    fn malformed_corpus_is_rejected() {
        match StalenessEvalRunner::from_json_with_seed("{not an array", 0, false) {
            Err(StalenessEvalError::MalformedCorpus(_)) => {}
            other => panic!("expected MalformedCorpus, got {other:?}"),
        }
    }

    #[test]
    fn invalid_tier_tag_is_rejected() {
        let bad = r#"[
          {"id":"x","fact_type":"preference","key":"k","stale_value":"a","fresh_value":"b",
           "stale_tier":"T9","fresh_tier":"T0","stale_evidence":["e"],"fresh_evidence":["e2"],
           "expect_recognized":true}
        ]"#;
        match StalenessEvalRunner::from_json_with_seed(bad, 0, false) {
            Err(StalenessEvalError::InvalidCase { .. }) => {}
            other => panic!("expected InvalidCase, got {other:?}"),
        }
    }
}
