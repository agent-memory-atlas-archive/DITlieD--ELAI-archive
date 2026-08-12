//! Per-user process operating profile (P-7 Tier-1 manual fork).
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::deflation_gate::{evaluate_deflation, DeflationInput, GateVerdictKind};
use crate::ssb::{OperatingProfileChangePayload, ProfileChangeKind, Ssb, SsbPayload};
use crate::trial_ledger::TrialLedger;

#[derive(Debug, Clone, PartialEq)]
pub struct OperatingProfile {
    pub per_domain: BTreeMap<String, DomainProfile>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomainProfile {
    pub autonomy: f64,
    pub gate_strictness: f64,
    pub explanation_verbosity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OwnRunFPEvidence {
    pub data_window: String,
    pub sr_observed: f64,
    pub t_periods: u64,
    pub skew: f64,
    pub excess_kurt: f64,
    pub sharpe_std: f64,
    pub sr_benchmark: f64,
    pub pbo: Option<f64>,
    pub seed: u64,
}

#[derive(Debug)]
pub enum ProfileError {
    MissingDomain(String),
    UnknownField(String),
    InvalidValue(String),
    NotMonotone { field: String, old: f64, new: f64 },
    EvidenceRequired,
    EvidenceRejected(String),
    Ledger(String),
    Journal(String),
}
impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDomain(d) => write!(f, "profile domain not found: {d}"),
            Self::UnknownField(x) => write!(f, "unknown profile field: {x}"),
            Self::InvalidValue(x) => write!(f, "invalid profile value: {x}"),
            Self::NotMonotone { field, old, new } => write!(f, "{field} is not monotone-tightened ({old} -> {new})"),
            Self::EvidenceRequired => write!(f, "loosen requires measured own-run false-positive evidence"),
            Self::EvidenceRejected(x) => write!(f, "loosen refused by deflation gate: {x}"),
            Self::Ledger(x) => write!(f, "trial ledger error: {x}"),
            Self::Journal(x) => write!(f, "profile journal error: {x}"),
        }
    }
}
impl std::error::Error for ProfileError {}

impl OperatingProfile {
    fn with(domain: &str, values: DomainProfile) -> Self {
        let mut per_domain = BTreeMap::new();
        per_domain.insert(domain.to_owned(), values);
        Self { per_domain }
    }
    pub fn toy() -> Self { Self::with("default", DomainProfile { autonomy: 0.9, gate_strictness: 0.2, explanation_verbosity: 0.2 }) }
    pub fn connected() -> Self { Self::with("default", DomainProfile { autonomy: 0.5, gate_strictness: 0.5, explanation_verbosity: 0.5 }) }
    pub fn team() -> Self { Self::with("default", DomainProfile { autonomy: 0.2, gate_strictness: 0.9, explanation_verbosity: 0.9 }) }
}

/// Stable Tier-1 composition intent for the unbuilt sibling profile units.
/// This is descriptive data only; it deliberately performs no sibling wiring.
fn sibling_intent(profile_name: &str) -> Option<&'static str> {
    match profile_name {
        "toy" => Some("intent:toy->P-5"),
        "connected" => Some("intent:connected->P-2"),
        "team" => Some("intent:team->M+P-8"),
        _ => None,
    }
}

fn field_value(d: &DomainProfile, field: &str) -> Result<f64, ProfileError> {
    match field { "autonomy" => Ok(d.autonomy), "gate_strictness" => Ok(d.gate_strictness), "explanation_verbosity" => Ok(d.explanation_verbosity), x => Err(ProfileError::UnknownField(x.to_owned())) }
}
fn set_field(d: &mut DomainProfile, field: &str, value: f64) -> Result<(), ProfileError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) { return Err(ProfileError::InvalidValue(value.to_string())); }
    match field { "autonomy" => d.autonomy=value, "gate_strictness" => d.gate_strictness=value, "explanation_verbosity" => d.explanation_verbosity=value, x => return Err(ProfileError::UnknownField(x.to_owned())) }
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct ProfileObservation {
    domain: String,
    field: String,
    old: f64,
    new: f64,
}

#[derive(Debug, Clone, Copy)]
enum ObservationDirection { Tighten, Loosen, Revert }

/// GD-4 observe-only phase: validate a proposed profile change without mutation or I/O.
fn observe_only(profile: &OperatingProfile, domain: &str, field: &str, new_value: f64, direction: ObservationDirection) -> Result<ProfileObservation, ProfileError> {
    let old = field_value(profile.per_domain.get(domain).ok_or_else(|| ProfileError::MissingDomain(domain.to_owned()))?, field)?;
    if !new_value.is_finite() || !(0.0..=1.0).contains(&new_value) {
        return Err(ProfileError::InvalidValue(new_value.to_string()));
    }
    let valid_direction = match direction {
        ObservationDirection::Tighten => match field { "gate_strictness" => new_value >= old, "autonomy" | "explanation_verbosity" => new_value <= old, _ => false },
        ObservationDirection::Loosen => match field { "gate_strictness" => new_value < old, "autonomy" | "explanation_verbosity" => new_value > old, _ => false },
        ObservationDirection::Revert => true,
    };
    if !valid_direction {
        return Err(ProfileError::NotMonotone { field: field.to_owned(), old, new: new_value });
    }
    Ok(ProfileObservation { domain: domain.to_owned(), field: field.to_owned(), old, new: new_value })
}

async fn journal(ssb: &Arc<Ssb>, domain: &str, field: &str, old: f64, new: f64, evidence_ref: Option<String>, kind: ProfileChangeKind) -> Result<(), ProfileError> {
    ssb.write_payload("operating_profile", 0, SsbPayload::OperatingProfileChange(OperatingProfileChangePayload { domain: domain.to_owned(), field: field.to_owned(), old_value: old.to_string(), new_value: new.to_string(), evidence_ref, change_kind: kind }), Vec::new()).await.map(|_| ()).map_err(|e| ProfileError::Journal(e.to_string()))
}

pub async fn tighten(profile: &OperatingProfile, domain: &str, field: &str, new_value: f64, ssb: &Arc<Ssb>) -> Result<OperatingProfile, ProfileError> {
    let observation = observe_only(profile, domain, field, new_value, ObservationDirection::Tighten)?;
    journal(ssb, &observation.domain, &observation.field, observation.old, observation.new, sibling_intent(domain).map(str::to_owned), ProfileChangeKind::Tighten).await?;
    let mut next = profile.clone();
    set_field(next.per_domain.get_mut(domain).ok_or_else(|| ProfileError::MissingDomain(domain.to_owned()))?, field, new_value)?;
    Ok(next)
}

pub async fn loosen(profile: &OperatingProfile, domain: &str, field: &str, new_value: f64, evidence: Option<OwnRunFPEvidence>, ledger: &TrialLedger, ssb: &Arc<Ssb>) -> Result<OperatingProfile, ProfileError> {
    let evidence = evidence.ok_or(ProfileError::EvidenceRequired)?;
    let input = DeflationInput { objective: format!("process_fp_rate_{domain}"), data_window: evidence.data_window.clone(), sr_observed: evidence.sr_observed, t_periods: evidence.t_periods, skew: evidence.skew, excess_kurt: evidence.excess_kurt, sharpe_std: evidence.sharpe_std, sr_benchmark: evidence.sr_benchmark, pbo: evidence.pbo, seed: evidence.seed };
    let verdict = evaluate_deflation(ledger, ssb, input).await.map_err(|e| ProfileError::Ledger(e.to_string()))?;
    if verdict.kind != GateVerdictKind::Allow { return Err(ProfileError::EvidenceRejected(verdict.reason)); }
    let observation = observe_only(profile, domain, field, new_value, ObservationDirection::Loosen)?;
    journal(ssb, &observation.domain, &observation.field, observation.old, observation.new, Some(evidence.data_window), ProfileChangeKind::Loosen).await?;
    let mut next = profile.clone();
    set_field(next.per_domain.get_mut(domain).ok_or_else(|| ProfileError::MissingDomain(domain.to_owned()))?, field, new_value)?;
    Ok(next)
}

pub async fn revert(profile: &OperatingProfile, domain: &str, field: &str, old_value: f64, ssb: &Arc<Ssb>) -> Result<OperatingProfile, ProfileError> {
    let observation = observe_only(profile, domain, field, old_value, ObservationDirection::Revert)?;
    journal(ssb, &observation.domain, &observation.field, observation.old, observation.new, None, ProfileChangeKind::Revert).await?;
    let mut next = profile.clone();
    set_field(next.per_domain.get_mut(domain).ok_or_else(|| ProfileError::MissingDomain(domain.to_owned()))?, field, old_value)?;
    Ok(next)
}


#[cfg(test)]
mod tests {
    use super::*;
    use elai_core::clock::SeededClock;
    use elai_core::ClockProvider;
    use std::sync::Arc;

    fn test_ssb(dir: &tempfile::TempDir) -> Arc<Ssb> {
        let config = crate::ssb::SsbConfig {
            db_path: dir.path().join("profile.db"),
            pool_size: Some(2),
            project_id: "profile-tests".to_owned(),
            session_id: "profile-tests".to_owned(),
        };
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        Arc::new(Ssb::new(config, clock).expect("test SSB should initialize"))
    }

    #[test]
    fn observe_only_does_not_mutate_profile() {
        let profile = OperatingProfile::connected();
        let before = profile.clone();
        let observation = observe_only(&profile, "default", "autonomy", 0.4, ObservationDirection::Tighten)
            .expect("valid observation");
        assert_eq!(observation.old, 0.5);
        assert_eq!(observation.new, 0.4);
        assert_eq!(profile, before);
    }

    #[tokio::test]
    async fn loosen_without_evidence_is_refused() {
        let dir = tempfile::tempdir().expect("test directory");
        let ssb = test_ssb(&dir);
        let ledger = TrialLedger::open(&dir.path().join("ledger.db"), "profile-tests")
            .expect("test ledger should initialize");
        let result = loosen(&OperatingProfile::connected(), "default", "autonomy", 0.7, None, &ledger, &ssb).await;
        assert!(matches!(result, Err(ProfileError::EvidenceRequired)));
    }

    #[tokio::test]
    async fn wrong_direction_loosen_is_rejected_before_gate_call() {
        let dir = tempfile::tempdir().expect("test directory");
        let ssb = test_ssb(&dir);
        let ledger = TrialLedger::open(&dir.path().join("ledger.db"), "profile-tests")
            .expect("test ledger should initialize");
        let evidence = OwnRunFPEvidence {
            data_window: "unit-test".to_owned(), sr_observed: 1.0, t_periods: 100,
            skew: 0.0, excess_kurt: 0.0, sharpe_std: 1.0, sr_benchmark: 0.0,
            pbo: Some(0.0), seed: 7,
        };
        let result = loosen(&OperatingProfile::connected(), "default", "autonomy", 0.4, Some(evidence), &ledger, &ssb).await;
        assert!(matches!(result, Err(ProfileError::NotMonotone { .. })));
    }

    #[tokio::test]
    async fn tighten_rejects_non_finite_value_without_journaling() {
        let dir = tempfile::tempdir().expect("test directory");
        let ssb = test_ssb(&dir);
        let profile = OperatingProfile::connected();
        let result = tighten(&profile, "default", "autonomy", f64::NEG_INFINITY, &ssb).await;
        assert!(matches!(result, Err(ProfileError::InvalidValue(_))));

        let conn = ssb.read_pool.get().await.expect("pool connection");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM ssb WHERE action_type = ?1",
                ["operating_profile_change"],
                |row| row.get(0),
            )
            .expect("profile journal count");
        assert_eq!(count, 0, "invalid tighten must not emit a profile journal row");
    }

    #[tokio::test]
    async fn loosen_allow_promotes_profile_and_journals_change() {
        let dir = tempfile::tempdir().expect("test directory");
        let ssb = test_ssb(&dir);
        let clock: Arc<dyn ClockProvider> = Arc::new(SeededClock::new());
        let ledger = TrialLedger::open(&dir.path().join("ledger.db"), "profile-tests")
            .expect("test ledger should initialize");
        let data_window = "allow-fixture-window";
        let objective = "process_fp_rate_default";
        ledger
            .register_trial(&ssb, &clock, objective, data_window, b"deterministic-fixture", 7)
            .await
            .expect("fixture trial should register");
        let evidence = OwnRunFPEvidence {
            data_window: data_window.to_owned(), sr_observed: 1.0, t_periods: 100,
            skew: 0.0, excess_kurt: 0.0, sharpe_std: 0.1, sr_benchmark: 0.0,
            pbo: Some(0.0), seed: 7,
        };

        let promoted = loosen(
            &OperatingProfile::connected(), "default", "autonomy", 0.7,
            Some(evidence), &ledger, &ssb,
        ).await.expect("deterministic Allow fixture should promote loosened profile");
        assert_eq!(promoted.per_domain["default"].autonomy, 0.7);

        let conn = ssb.read_pool.get().await.expect("pool connection");
        let (count, payload): (i64, Vec<u8>) = conn
            .query_row(
                "SELECT COUNT(*), payload FROM ssb WHERE action_type = ?1",
                ["operating_profile_change"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("profile journal row");
        assert_eq!(count, 1, "successful loosen emits one profile journal row");
        let payload: SsbPayload = rmp_serde::from_slice(&payload)
            .expect("profile journal payload should deserialize");
        assert!(matches!(
            payload,
            SsbPayload::OperatingProfileChange(OperatingProfileChangePayload {
                domain, field, old_value, new_value, evidence_ref,
                change_kind: ProfileChangeKind::Loosen,
            }) if domain == "default" && field == "autonomy"
                && old_value == "0.5" && new_value == "0.7"
                && evidence_ref.as_deref() == Some(data_window)
        ));
    }

    #[tokio::test]
    async fn tighten_succeeds_without_evidence_and_revert_restores_prior_field() {
        let dir = tempfile::tempdir().expect("test directory");
        let ssb = test_ssb(&dir);
        let profile = OperatingProfile::connected();
        let tightened = tighten(&profile, "default", "autonomy", 0.4, &ssb).await
            .expect("tighten should succeed without evidence");
        assert_eq!(tightened.per_domain["default"].autonomy, 0.4);
        let reverted = revert(&tightened, "default", "autonomy", 0.5, &ssb).await
            .expect("revert should succeed");
        assert_eq!(reverted.per_domain["default"].autonomy, 0.5);
    }
}
