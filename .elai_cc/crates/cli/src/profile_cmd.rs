//! P-7 `elai profile` production handler.
use std::sync::Arc;

use elai_core::{clock::OsClock, paths::sessions_dir_path};
use elai_orchestrator::{
    operating_profile::{self, OperatingProfile, OwnRunFPEvidence},
    ssb::{Ssb, SsbConfig},
    trial_ledger::TrialLedger,
};
use thiserror::Error;

const PROJECT_ID: &str = "elai-profile";
const SESSION_ID: &str = "elai-profile-session";

#[derive(Debug, Error)]
pub(crate) enum ProfileCommandError {
    #[error("current directory error: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("SSB error: {0}")]
    Ssb(String),
    #[error("trial ledger error: {0}")]
    Ledger(#[from] elai_orchestrator::trial_ledger::TrialLedgerError),
    #[error("profile error: {0}")]
    Profile(#[from] operating_profile::ProfileError),
    #[error("invalid value: {0}")]
    Value(#[from] std::num::ParseFloatError),
    #[error("evidence JSON error: {0}")]
    Evidence(#[from] serde_json::Error),
}

fn parse_evidence(raw: &str, domain: &str) -> Result<OwnRunFPEvidence, ProfileCommandError> {
    let value: serde_json::Value = serde_json::from_str(raw)?;
    let object = value.as_object().ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue("evidence must be a JSON object".to_owned())))?;
    let string = |key: &str| object.get(key).and_then(serde_json::Value::as_str).map(ToOwned::to_owned).ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue(format!("evidence missing string field {key}"))));
    let number = |key: &str| object.get(key).and_then(serde_json::Value::as_f64).ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue(format!("evidence missing numeric field {key}"))));
    let periods = object.get("t_periods").and_then(serde_json::Value::as_u64).ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue("evidence missing integer field t_periods".to_owned())))?;
    let seed = object.get("seed").and_then(serde_json::Value::as_u64).ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue("evidence missing integer field seed".to_owned())))?;
    Ok(OwnRunFPEvidence { data_window: string("data_window")?, sr_observed: number("sr_observed")?, t_periods: periods, skew: number("skew")?, excess_kurt: number("excess_kurt")?, sharpe_std: number("sharpe_std")?, sr_benchmark: number("sr_benchmark")?, pbo: object.get("pbo").and_then(serde_json::Value::as_f64), seed })
}

fn selected_profile(domain: Option<&str>) -> OperatingProfile {
    let name = domain.unwrap_or("connected");
    // The amended Tier-1 fork exposes the three profiles as data. Their
    // future sibling composition remains journaled intent, not wiring.
    let mut profile = match name {
        "toy" => OperatingProfile::toy(),
        "team" => OperatingProfile::team(),
        "connected" | "default" => OperatingProfile::connected(),
        _ => OperatingProfile::connected(),
    };
    if name != "default" {
        if let Some(values) = profile.per_domain.remove("default") {
            profile.per_domain.insert(name.to_owned(), values);
        }
    }
    profile
}

fn open_runtime() -> Result<(Arc<Ssb>, TrialLedger), ProfileCommandError> {
    let project_root = std::env::current_dir()?;
    let sessions_dir = sessions_dir_path(&project_root);
    std::fs::create_dir_all(&sessions_dir)?;
    let db_path = sessions_dir.join("ssb.db");
    let config = SsbConfig {
        db_path: db_path.clone(),
        pool_size: None,
        project_id: PROJECT_ID.to_owned(),
        session_id: SESSION_ID.to_owned(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    let ssb = Arc::new(Ssb::new(config, clock).map_err(|e| ProfileCommandError::Ssb(e.to_string()))?);
    let ledger = TrialLedger::open(&db_path, SESSION_ID)?;
    Ok((ssb, ledger))
}

/// Dispatch the live `elai profile` command.
pub(crate) async fn run(
    subcommand: String,
    domain: Option<String>,
    field: Option<String>,
    value: Option<String>,
    evidence: Option<String>,
) -> Result<(), ProfileCommandError> {
    let domain_name = domain.as_deref().unwrap_or("default");
    let profile = selected_profile(Some(domain_name));
    if subcommand == "show" {
        if let Some(values) = profile.per_domain.get(domain_name) {
            println!("profile domain={domain_name} autonomy={} gate_strictness={} explanation_verbosity={}", values.autonomy, values.gate_strictness, values.explanation_verbosity);
        }
        return Ok(());
    }
    let field_name = field.as_deref().ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::UnknownField("missing --field".to_owned())))?;
    let new_value = value.as_deref().ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue("missing --value".to_owned())))?.parse::<f64>()?;
    let (ssb, ledger) = open_runtime()?;
    let next = match subcommand.as_str() {
        "tighten" => operating_profile::tighten(&profile, domain_name, field_name, new_value, &ssb).await?,
        "loosen" => {
            let parsed = evidence.map(|raw| parse_evidence(&raw, domain_name)).transpose()?;
            operating_profile::loosen(&profile, domain_name, field_name, new_value, parsed, &ledger, &ssb).await?
        }
        "revert" => operating_profile::revert(&profile, domain_name, field_name, new_value, &ssb).await?,
        other => return Err(ProfileCommandError::Profile(operating_profile::ProfileError::InvalidValue(format!("unknown subcommand: {other}")))),
    };
    let values = next.per_domain.get(domain_name).ok_or_else(|| ProfileCommandError::Profile(operating_profile::ProfileError::MissingDomain(domain_name.to_owned())))?;
    println!("profile {subcommand} applied: domain={domain_name} field={field_name} value={}", match field_name {
        "autonomy" => values.autonomy,
        "gate_strictness" => values.gate_strictness,
        "explanation_verbosity" => values.explanation_verbosity,
        _ => return Err(ProfileCommandError::Profile(operating_profile::ProfileError::UnknownField(field_name.to_owned()))),
    });
    Ok(())
}
