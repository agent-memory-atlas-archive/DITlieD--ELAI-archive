//! Risk-signal driven GateIntensity selector.
//!
//! K-5: a deterministic, pure-CPU mapping from a small risk-signal vector to
//! an audit-leg tier (Light / Standard / Core). The selector is observe-only;
//! it does not suppress legs. The policy is driven by an embedded TOML table
//! (`data/gate_intensity/policy_table.toml`) plus a hard-coded degrade-safe
//! fallback that maps every signal to Core on parse failure.

use std::sync::OnceLock;

use serde::Deserialize;

pub use crate::deterministic_auditor::CheckKind;

/// Policy version shipped with this selector.
pub const POLICY_VERSION: &str = "K-5.0";

/// Embedded policy table. The runtime path uses `toml` (already a workspace
/// dependency); the const fallback below guarantees degrade-safety if the
/// embedded file cannot be parsed.
const POLICY_TOML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../data/gate_intensity/policy_table.toml"
));

// ---------------------------------------------------------------------------
// Tier and signal enums
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tier {
    Light,
    Standard,
    #[default]
    Core,
}

impl Tier {
    pub fn tier_name(&self) -> &'static str {
        match self {
            Tier::Light => "light",
            Tier::Standard => "standard",
            Tier::Core => "core",
        }
    }

    fn index(&self) -> u8 {
        match self {
            Tier::Light => 0,
            Tier::Standard => 1,
            Tier::Core => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RiskLevelSignal {
    Low,
    Medium,
    High,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComplexityClass {
    Small,
    Medium,
    Large,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BlastRadiusCone {
    SingleCrate,
    MultiCrate,
    CrossCutting,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CweClass {
    None,
    Other,
    Auth,
    Crypto,
    Injection,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NoveltySignal {
    Established,
    Recent,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CentralitySignal {
    Leaf,
    Internal,
    Core,
    #[default]
    Unknown,
}

// ---------------------------------------------------------------------------
// Signal vector + selection
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct RiskSignalVector {
    pub risk_level: RiskLevelSignal,
    pub complexity_class: ComplexityClass,
    pub blast_radius: BlastRadiusCone,
    pub cwe_class: CweClass,
    pub novelty: NoveltySignal,
    pub centrality: CentralitySignal,
}

#[derive(Debug, Clone)]
pub struct GateIntensitySelection {
    pub tier: Tier,
    pub always_on: Vec<CheckKind>,
    pub risk_triggered: Vec<CheckKind>,
    pub selected: Vec<CheckKind>,
    pub policy_version: &'static str,
}

// ---------------------------------------------------------------------------
// Policy table (TOML-backed)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
struct PolicyTable {
    policy_version: String,
    tier: TierPolicy,
}

#[derive(Debug, Clone, Deserialize)]
struct TierPolicy {
    light: TierEntry,
    standard: TierEntry,
    core: TierEntry,
}

#[derive(Debug, Clone, Deserialize)]
struct TierEntry {
    always_on: Vec<String>,
    risk_triggered: Vec<String>,
}

static POLICY: OnceLock<PolicyTable> = OnceLock::new();

fn policy() -> &'static PolicyTable {
    POLICY.get_or_init(|| {
        toml::de::from_str(POLICY_TOML).unwrap_or_else(|_| PolicyTable {
            policy_version: POLICY_VERSION.to_owned(),
            tier: TierPolicy {
                light: TierEntry {
                    always_on: vec!["Build".to_owned(), "LazyCode".to_owned()],
                    risk_triggered: vec!["FileSizeCensus".to_owned()],
                },
                standard: TierEntry {
                    always_on: vec![
                        "Build".to_owned(),
                        "FileSizeCensus".to_owned(),
                        "LazyCode".to_owned(),
                        "Completeness".to_owned(),
                    ],
                    risk_triggered: vec![
                        "VersionGate".to_owned(),
                        "ImportCrossRef".to_owned(),
                        "ModuleCoherence".to_owned(),
                    ],
                },
                core: TierEntry {
                    always_on: vec![
                        "Build".to_owned(),
                        "VersionGate".to_owned(),
                        "FileSizeCensus".to_owned(),
                        "ImportCrossRef".to_owned(),
                        "LazyCode".to_owned(),
                        "Completeness".to_owned(),
                        "ModuleCoherence".to_owned(),
                        "Mutation".to_owned(),
                        "Metamorphic".to_owned(),
                    ],
                    risk_triggered: vec![],
                },
            },
        })
    })
}

fn parse_check_kind(s: &str) -> Option<CheckKind> {
    match s {
        "Build" => Some(CheckKind::Build),
        "VersionGate" => Some(CheckKind::VersionGate),
        "FileSizeCensus" => Some(CheckKind::FileSizeCensus),
        "ImportCrossRef" => Some(CheckKind::ImportCrossRef),
        "LazyCode" => Some(CheckKind::LazyCode),
        "Completeness" => Some(CheckKind::Completeness),
        "ModuleCoherence" => Some(CheckKind::ModuleCoherence),
        "Mutation" => Some(CheckKind::Mutation),
        "Metamorphic" => Some(CheckKind::Metamorphic),
        _ => None,
    }
}

fn tier_entry_for(tier: Tier) -> &'static TierEntry {
    let p = policy();
    match tier {
        Tier::Light => &p.tier.light,
        Tier::Standard => &p.tier.standard,
        Tier::Core => &p.tier.core,
    }
}

fn parse_check_list(names: &[String]) -> Vec<CheckKind> {
    names.iter().filter_map(|s| parse_check_kind(s)).collect()
}

fn dedup_preserve_order(input: &[CheckKind]) -> Vec<CheckKind> {
    let mut out = Vec::with_capacity(input.len());
    for k in input {
        if !out.contains(k) {
            out.push(*k);
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Mapping from signals to tier
// ---------------------------------------------------------------------------

fn tier_from_risk_level(s: RiskLevelSignal) -> Tier {
    match s {
        RiskLevelSignal::Low => Tier::Light,
        RiskLevelSignal::Medium => Tier::Standard,
        RiskLevelSignal::High | RiskLevelSignal::Unknown => Tier::Core,
    }
}

fn tier_from_complexity(s: ComplexityClass) -> Tier {
    match s {
        ComplexityClass::Small => Tier::Light,
        ComplexityClass::Medium => Tier::Standard,
        ComplexityClass::Large | ComplexityClass::Unknown => Tier::Core,
    }
}

fn tier_from_blast_radius(s: BlastRadiusCone) -> Tier {
    match s {
        BlastRadiusCone::SingleCrate => Tier::Light,
        BlastRadiusCone::MultiCrate => Tier::Standard,
        BlastRadiusCone::CrossCutting | BlastRadiusCone::Unknown => Tier::Core,
    }
}

fn tier_from_cwe_class(s: CweClass) -> Tier {
    match s {
        CweClass::None => Tier::Light,
        CweClass::Other => Tier::Standard,
        CweClass::Auth | CweClass::Crypto | CweClass::Injection | CweClass::Unknown => Tier::Core,
    }
}

fn tier_from_novelty(s: NoveltySignal) -> Tier {
    match s {
        NoveltySignal::Established => Tier::Light,
        NoveltySignal::Recent => Tier::Standard,
        NoveltySignal::Unknown => Tier::Core,
    }
}

fn tier_from_centrality(s: CentralitySignal) -> Tier {
    match s {
        CentralitySignal::Leaf => Tier::Light,
        CentralitySignal::Internal => Tier::Standard,
        CentralitySignal::Core | CentralitySignal::Unknown => Tier::Core,
    }
}

fn max_tier(a: Tier, b: Tier) -> Tier {
    if b.index() >= a.index() {
        b
    } else {
        a
    }
}

// ---------------------------------------------------------------------------
// Public selector
// ---------------------------------------------------------------------------

/// Select the audit-leg tier for a risk-signal vector. Degrade-safe: any
/// `Unknown` signal maps to Core, so missing information never reduces the leg
/// set. The returned selection is observe-only; callers must not use it to
/// suppress checks run by the deterministic auditor.
pub fn select(signals: &RiskSignalVector) -> GateIntensitySelection {
    let mut tier = Tier::Light;
    tier = max_tier(tier, tier_from_risk_level(signals.risk_level));
    tier = max_tier(tier, tier_from_complexity(signals.complexity_class));
    tier = max_tier(tier, tier_from_blast_radius(signals.blast_radius));
    tier = max_tier(tier, tier_from_cwe_class(signals.cwe_class));
    tier = max_tier(tier, tier_from_novelty(signals.novelty));
    tier = max_tier(tier, tier_from_centrality(signals.centrality));

    let entry = tier_entry_for(tier);
    let always_on = parse_check_list(&entry.always_on);
    let risk_triggered = parse_check_list(&entry.risk_triggered);

    let mut selected = always_on.clone();
    selected.extend_from_slice(&risk_triggered);
    let selected = dedup_preserve_order(&selected);

    GateIntensitySelection {
        tier,
        always_on,
        risk_triggered,
        selected,
        policy_version: POLICY_VERSION,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_signals_degrade_to_core() {
        let signals = RiskSignalVector::default();
        let sel = select(&signals);
        assert_eq!(sel.tier, Tier::Core);
        assert!(sel.selected.contains(&CheckKind::Mutation));
        assert!(sel.selected.contains(&CheckKind::Metamorphic));
    }

    #[test]
    fn low_signals_select_light() {
        let signals = RiskSignalVector {
            risk_level: RiskLevelSignal::Low,
            complexity_class: ComplexityClass::Small,
            blast_radius: BlastRadiusCone::SingleCrate,
            cwe_class: CweClass::None,
            novelty: NoveltySignal::Established,
            centrality: CentralitySignal::Leaf,
        };
        let sel = select(&signals);
        assert_eq!(sel.tier, Tier::Light);
        assert!(sel.selected.contains(&CheckKind::Build));
        assert!(sel.selected.contains(&CheckKind::LazyCode));
    }
}
