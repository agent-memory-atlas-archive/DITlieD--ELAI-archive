//! `MemoryFact` row + `FactType` + `Scope` enums. Mirrors the
//! `memory_facts` SQL table layout one-for-one (16 columns, bi-temporal
//! `valid_from` / `valid_to` / `ingested_at` triple per Zep §2.2.3).
//!
//! Stable wire-form snake_case tags on `FactType` / `Scope` are the
//! contract carried by JSON `value` columns + Tauri command payloads; do
//! not rename without bumping `SCHEMA_VERSION`.

use serde::{Deserialize, Serialize};

use crate::error::MemoryError;
use crate::trust::{Provenance, TrustTier};

/// Discriminator for the kind of fact stored. Mirrors the canonical list
/// from the E-5 report ("What to model about the user") plus `mistake`
/// (C-21 MistakeBook bridge). Each variant carries its own
/// [`crate::decay::default_decay_rate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactType {
    /// Voice / tone / formatting preferences. Per-user global by default
    /// (E-3 voice anchor reads these).
    VoiceStyle,
    /// Dreyfus-stage expertise per language / framework / domain (E-4
    /// read path).
    Expertise,
    /// User's risk tolerance for destructive / cloud / high-spend actions.
    RiskTol,
    /// First-person opinion the user has expressed. HIGH-protection class
    /// per the report — `opinion` rows require trust_score >= 0.7 at
    /// retrieval (see [`crate::retrieval`]).
    Opinion,
    /// Tool / library / pattern preference (e.g. "prefer tracing over
    /// log crate").
    Preference,
    /// Per-project domain knowledge (project conventions, naming, layout).
    DomainKnowledge,
    /// Session productivity rhythm (preferred working hours, max session
    /// length).
    SessionPattern,
    /// MistakeBook bridge: a recurring mistake / footgun the user wants
    /// surfaced (C-21 maps `rules.db` ACTIVE rows here).
    Mistake,
}

impl FactType {
    /// Stable snake_case wire tag — single source of truth for SQL writes
    /// and JSON exchange.
    pub fn as_str(&self) -> &'static str {
        match self {
            FactType::VoiceStyle => "voice_style",
            FactType::Expertise => "expertise",
            FactType::RiskTol => "risk_tol",
            FactType::Opinion => "opinion",
            FactType::Preference => "preference",
            FactType::DomainKnowledge => "domain_knowledge",
            FactType::SessionPattern => "session_pattern",
            FactType::Mistake => "mistake",
        }
    }

    /// Parse a snake_case wire tag back into a [`FactType`].
    pub fn from_str(s: &str) -> Result<Self, MemoryError> {
        match s {
            "voice_style" => Ok(FactType::VoiceStyle),
            "expertise" => Ok(FactType::Expertise),
            "risk_tol" => Ok(FactType::RiskTol),
            "opinion" => Ok(FactType::Opinion),
            "preference" => Ok(FactType::Preference),
            "domain_knowledge" => Ok(FactType::DomainKnowledge),
            "session_pattern" => Ok(FactType::SessionPattern),
            "mistake" => Ok(FactType::Mistake),
            other => Err(MemoryError::Invalid(format!(
                "unknown fact_type tag: {other}"
            ))),
        }
    }

    /// Retrieval trust-score threshold per the E-5 report Task 7.
    /// `opinion` is the HIGH-protection class.
    pub fn retrieval_threshold(&self) -> f32 {
        match self {
            FactType::Opinion => 0.7,
            FactType::Preference => 0.5,
            FactType::Mistake => 0.4,
            FactType::VoiceStyle
            | FactType::Expertise
            | FactType::RiskTol
            | FactType::DomainKnowledge
            | FactType::SessionPattern => 0.5,
        }
    }
}

/// Per-project vs. per-user-global scope. Per the E-5 report + BeliefShift
/// (arXiv:2603.23848), promotion from `project` to `global` is GATED, not
/// eager — the gate logic itself is reserved for the dispatcher; this enum
/// is the persistent shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    /// Per-project fact (default for newly extracted rows).
    Project,
    /// Global, per-user fact (cross-project — promoted by the gate or by
    /// explicit user command).
    Global,
}

impl Scope {
    /// Stable snake_case wire tag.
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Project => "project",
            Scope::Global => "global",
        }
    }

    /// Parse a snake_case wire tag back into a [`Scope`].
    pub fn from_str(s: &str) -> Result<Self, MemoryError> {
        match s {
            "project" => Ok(Scope::Project),
            "global" => Ok(Scope::Global),
            other => Err(MemoryError::Invalid(format!(
                "unknown scope tag: {other}"
            ))),
        }
    }
}

/// One row of the `memory_facts` table — the canonical bi-temporal episodic
/// fact shape collapsed into a flat Rust struct.
///
/// Timestamps are milliseconds since UNIX epoch (sourced from
/// [`elai_core::ClockProvider::now_ms`]). `valid_to == None` marks a row
/// as currently-valid; `superseded_by` points to the replacement
/// `fact_id` when an UPDATE replaces this row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryFact {
    /// Stable identifier — UUIDv4 hex by convention (see
    /// [`MemoryFact::new_id`]).
    pub fact_id: String,
    /// User identifier (single-user installs hardcode `"default"`).
    pub user_id: String,
    /// Project identifier (`None` for `scope == Global`).
    pub project_id: Option<String>,
    /// `project` or `global`.
    pub scope: Scope,
    /// One of the eight [`FactType`] variants.
    pub fact_type: FactType,
    /// Stable key (e.g. `"prefers_tracing_over_log"`). Together with
    /// `fact_type` this is the duplicate-detection key for the Mem0
    /// dispatcher (ADD vs UPDATE).
    pub key: String,
    /// Arbitrary serde_json value carrying the fact's payload. Stored as
    /// JSON text in sqlite.
    pub value: serde_json::Value,
    /// [0.0, 1.0] — extractor / classifier-emitted confidence.
    pub confidence: f32,
    /// [0.0, 1.0] — monotone-decayed trust score (see [`crate::decay`]).
    pub trust_score: f32,
    /// `now()` at row creation. Never changes after insert.
    pub ingested_at_ms: u128,
    /// Start of the bi-temporal validity interval (`valid_from`).
    pub valid_from_ms: u128,
    /// End of the bi-temporal validity interval. `None` while the row is
    /// active.
    pub valid_to_ms: Option<u128>,
    /// Decay rate per day (units of trust_score / day). See
    /// [`crate::decay::default_decay_rate`] for fact-type defaults.
    pub decay_rate_per_day: f32,
    /// `now()` at the most recent observation (read or extract). Updated
    /// on retrieval — feeds the decay curve.
    pub last_observed_at_ms: u128,
    /// Non-empty list of evidence pointers (SessionDag node IDs / message
    /// IDs / artifact paths). Empty list = row excluded from retrieval
    /// per arXiv:2512.16962 MemoryGraft defense.
    pub evidence_pointers: Vec<String>,
    /// Optional `fact_id` of the row that supersedes this one.
    pub superseded_by: Option<String>,
    /// H-32 VALIDITY axis — last wall-clock ms when the cited DKG symbol was
    /// confirmed to still resolve. Distinct from `last_observed_at_ms`
    /// (RECENCY). `None` on Tier-B (Preference-class) facts; `None` on a
    /// Tier-A fact that has never been validated (ages from `valid_from_ms`
    /// for TTL purposes). Updated by `get_relevant_facts` on a valid
    /// re-check (stamps the TTL-reset). Never deleted.
    pub last_validated_at_ms: Option<u128>,
    /// C-48 SafeHarbor trust tier. T3 = most pessimistic default (migration assigns T3
    /// to all pre-C-48 rows). Safety-critical queries exclude T2/T3.
    pub trust_tier: TrustTier,
    /// C-48 SafeHarbor provenance. Non-optional post-migration.
    pub provenance: Provenance,
}

impl MemoryFact {
    /// Generate a fresh UUIDv4 hex string for `fact_id`. Single source of
    /// truth so call sites cannot drift to ad-hoc IDs.
    pub fn new_id() -> String {
        uuid::Uuid::new_v4().to_string() // entropy-allow: opaque primary-key row id, never byte-compared by replay (matches orchestrator audit_id/request_id precedent)
    }

    /// Validate invariants before persistence. Cheap — pure CPU.
    pub fn validate(&self) -> Result<(), MemoryError> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err(MemoryError::Invalid(format!(
                "confidence out of [0,1]: {}",
                self.confidence
            )));
        }
        if !(0.0..=1.0).contains(&self.trust_score) {
            return Err(MemoryError::Invalid(format!(
                "trust_score out of [0,1]: {}",
                self.trust_score
            )));
        }
        if self.decay_rate_per_day < 0.0 {
            return Err(MemoryError::Invalid(format!(
                "decay_rate_per_day negative: {}",
                self.decay_rate_per_day
            )));
        }
        if self.key.is_empty() {
            return Err(MemoryError::Invalid("key is empty".into()));
        }
        if self.user_id.is_empty() {
            return Err(MemoryError::Invalid("user_id is empty".into()));
        }
        if matches!(self.scope, Scope::Project) && self.project_id.is_none() {
            return Err(MemoryError::Invalid(
                "scope=project requires project_id".into(),
            ));
        }
        // Provenance enforcement (E-5 Task 10): no empty evidence
        // pointers on insert. Empty = excluded at retrieval anyway, but
        // catching at insert is the stronger contract.
        if self.evidence_pointers.is_empty() {
            return Err(MemoryError::Invalid(
                "evidence_pointers must be non-empty (provenance enforcement)".into(),
            ));
        }
        if let Some(end) = self.valid_to_ms {
            if end < self.valid_from_ms {
                return Err(MemoryError::Invalid(
                    "valid_to_ms < valid_from_ms".into(),
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod cold_wiring_tests {
    //! Cold-wiring (s-81344): deterministic unit coverage for the previously
    //! untested fact.rs codecs + validator. All inputs are fixed literals (no
    //! wall clock / rng) per the entropy-quarantine rule.
    use super::*;

    fn make_valid_fact() -> MemoryFact {
        MemoryFact {
            fact_id: "fact-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: Some("proj-1".to_string()),
            scope: Scope::Project,
            fact_type: FactType::Preference,
            key: "prefers_tracing".to_string(),
            value: serde_json::json!({ "v": true }),
            confidence: 0.8,
            trust_score: 0.7,
            ingested_at_ms: 1_000,
            valid_from_ms: 1_000,
            valid_to_ms: None,
            decay_rate_per_day: 0.05,
            last_observed_at_ms: 1_000,
            evidence_pointers: vec!["ev-1".to_string()],
            superseded_by: None,
            trust_tier: TrustTier::T0UserDirect,
            provenance: Provenance::user_direct("cli"),
            last_validated_at_ms: None,
        }
    }

    #[test]
    fn fact_type_codec_roundtrips_all_variants() {
        let pairs = [
            (FactType::VoiceStyle, "voice_style"),
            (FactType::Expertise, "expertise"),
            (FactType::RiskTol, "risk_tol"),
            (FactType::Opinion, "opinion"),
            (FactType::Preference, "preference"),
            (FactType::DomainKnowledge, "domain_knowledge"),
            (FactType::SessionPattern, "session_pattern"),
            (FactType::Mistake, "mistake"),
        ];
        for (ft, tag) in pairs {
            assert_eq!(ft.as_str(), tag);
            assert_eq!(FactType::from_str(tag).unwrap(), ft);
        }
        assert!(FactType::from_str("not_a_tag").is_err());
    }

    #[test]
    fn fact_type_retrieval_thresholds() {
        // opinion is the HIGH-protection class (0.7); others per E-5 Task 7.
        assert!((FactType::Opinion.retrieval_threshold() - 0.7).abs() < f32::EPSILON);
        assert!((FactType::Preference.retrieval_threshold() - 0.5).abs() < f32::EPSILON);
        assert!((FactType::Mistake.retrieval_threshold() - 0.4).abs() < f32::EPSILON);
        assert!((FactType::Expertise.retrieval_threshold() - 0.5).abs() < f32::EPSILON);
        assert!((FactType::VoiceStyle.retrieval_threshold() - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn scope_codec_roundtrips() {
        assert_eq!(Scope::Project.as_str(), "project");
        assert_eq!(Scope::Global.as_str(), "global");
        assert_eq!(Scope::from_str("project").unwrap(), Scope::Project);
        assert_eq!(Scope::from_str("global").unwrap(), Scope::Global);
        assert!(Scope::from_str("nope").is_err());
    }

    #[test]
    fn validate_accepts_well_formed_fact() {
        assert!(make_valid_fact().validate().is_ok());
    }

    #[test]
    fn validate_rejects_confidence_out_of_range() {
        let mut f = make_valid_fact();
        f.confidence = 1.5;
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_trust_score_out_of_range() {
        let mut f = make_valid_fact();
        f.trust_score = -0.1;
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_negative_decay_rate() {
        let mut f = make_valid_fact();
        f.decay_rate_per_day = -1.0;
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_key_and_user_id() {
        let mut f = make_valid_fact();
        f.key = String::new();
        assert!(f.validate().is_err());
        let mut f = make_valid_fact();
        f.user_id = String::new();
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_project_scope_without_project_id() {
        let mut f = make_valid_fact();
        f.scope = Scope::Project;
        f.project_id = None;
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_evidence_pointers() {
        let mut f = make_valid_fact();
        f.evidence_pointers.clear();
        assert!(f.validate().is_err());
    }

    #[test]
    fn validate_rejects_inverted_validity_interval() {
        let mut f = make_valid_fact();
        f.valid_from_ms = 5_000;
        f.valid_to_ms = Some(1_000);
        assert!(f.validate().is_err());
    }
}
