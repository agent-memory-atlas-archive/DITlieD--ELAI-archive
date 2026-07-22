//! J-35 intent-control layer: typed intent spec + structurally-forced
//! premise-check + outcome/reversibility route + journaled assumptions.
//!
//! Tier-1 OBSERVE-ONLY: emits IntentControlEvent SSB rows but does NOT
//! block decomposition. Default-OFF behind ELAI_INTENT_CONTROL_ENABLE.
//!
//! The layer composes shipped substrates (B-40 IdeaFramer, D-16 conformal,
//! B-22 GateMode, Phase-3A RiskLevel, the SSB outcome journal) and restates
//! none of them. It NEVER emits an "aim-verified" verdict; it surfaces
//! aim-adjacent signals (premise flags, assumptions, route) for human or
//! journal-sourced disposal, never gating aim.

use std::path::Path;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use elai_core::clock::ClockProvider;

use crate::autonomous_gate::GateMode;
use crate::gate::RiskLevel;
use crate::ssb::{
    IntentControlEventPayload, IntentControlKind, Ssb, SsbActionType, SsbPayload, SsbReader,
    SsbWriteRequest,
};

const INTENT_CONTROL_AGENT_ID: &str = "intent-control";

/// Reversibility class for the route decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReversibilityClass {
    Reversible,
    HighCost,
    Irreversible,
}

impl ReversibilityClass {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Reversible => "reversible",
            Self::HighCost => "high_cost",
            Self::Irreversible => "irreversible",
        }
    }
}

/// Route decision over ambiguity + premise-flag + reversibility + GateMode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteDecision {
    Ask,
    ProceedUnderAssumptions,
    Refuse,
    Escalate,
}

impl RouteDecision {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Ask => "ask",
            Self::ProceedUnderAssumptions => "proceed_under_assumptions",
            Self::Refuse => "refuse",
            Self::Escalate => "escalate",
        }
    }
}

/// A premise extracted from the task message, checked against grounding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PremiseCheck {
    /// The premise text extracted from the request.
    pub premise: String,
    /// Whether the premise was flagged as false or unsupported.
    pub flagged: bool,
    /// Evidence for the flag (contradiction text or "unsupported").
    pub evidence: String,
}

/// The typed intent spec assembled from the raw task message.
/// Reuses the shape of BabyStepSpec/IntentPair/WiringContract concepts
/// but stays lightweight at Tier-1 (the spec is SHOWN, not hidden).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentSpec {
    /// Stable identifier (blake3 prefix of the task text, hex).
    pub spec_id: String,
    /// The reconstructed goal (the user's literal goal, shown not hidden).
    pub goal: String,
    /// Extracted constraints.
    pub constraints: Vec<String>,
    /// Hidden assumptions detected in the request.
    pub hidden_assumptions: Vec<String>,
    /// Declared non-goals.
    pub non_goals: Vec<String>,
    /// Premise-check results (format-independent).
    pub premise_checks: Vec<PremiseCheck>,
}

impl IntentSpec {
    /// Count of flagged premises.
    pub fn premise_flag_count(&self) -> u32 {
        self.premise_checks.iter().filter(|p| p.flagged).count() as u32
    }

    /// Count of hidden assumptions.
    pub fn assumption_count(&self) -> u32 {
        self.hidden_assumptions.len() as u32
    }
}

/// Deterministic premise extraction: scans the task text for premise
/// patterns ("the X is Y", "foo.rs has a sync loop", etc.) and returns
/// them for checking. Format-independent: does NOT condition on the
/// requested output format.
fn extract_premises(task_text: &str) -> Vec<String> {
    let mut premises = Vec::new();
    let lower = task_text.to_lowercase();
    let trigger_phrases = [
        "the sync ",
        "the async ",
        "the synchronous ",
        "the asynchronous ",
        "is sync",
        "is async",
        "already ",
        "currently ",
    ];
    for phrase in &trigger_phrases {
        if lower.contains(phrase) {
            let idx = lower.find(phrase).unwrap_or(0);
            let end = task_text.len().min(idx + 120);
            if let Some(snippet) = task_text.get(idx..end) {
                let snippet = snippet.trim();
                if !snippet.is_empty() {
                    premises.push(snippet.to_owned());
                }
            }
        }
    }
    premises
}

const FILE_READ_CAP: usize = 64 * 1024;

const ASYNC_MARKERS: &[&str] = &["async fn", ".await", "tokio::"];

fn extract_referenced_paths(text: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let extensions = [".rs", ".toml", ".py", ".ts", ".svelte"];
    let mut tokens = text.split_whitespace();
    while let Some(tok) = tokens.next() {
        let trimmed = tok.trim_matches(|c: char| !c.is_alphanumeric() && c != '.' && c != '/' && c != '-' && c != '_');
        for ext in &extensions {
            if trimmed.ends_with(ext) {
                paths.push(trimmed.to_owned());
                break;
            }
        }
    }
    paths
}

fn read_capped(path: &Path) -> Option<String> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    let bytes = std::fs::read(path).ok()?;
    let cap = FILE_READ_CAP.min(bytes.len());
    String::from_utf8_lossy(&bytes[..cap]).to_string().into()
}

fn check_premise(premise: &str, task_text: &str, project_root: Option<&Path>) -> PremiseCheck {
    let lower_premise = premise.to_lowercase();
    let premise_claims_sync =
        (lower_premise.contains("sync") || lower_premise.contains("synchronous"))
            && !lower_premise.contains("async")
            && !lower_premise.contains("asynchronous");

    let referenced = extract_referenced_paths(task_text);

    for rel in &referenced {
        let resolved = match project_root {
            Some(root) => root.join(rel),
            None => Path::new(rel).to_path_buf(),
        };
        if !resolved.exists() {
            return PremiseCheck {
                premise: premise.to_owned(),
                flagged: true,
                evidence: format!("file missing: {}", resolved.display()),
            };
        }
        if let Some(contents) = read_capped(&resolved) {
            let lower_contents = contents.to_lowercase();
            if premise_claims_sync {
                for marker in ASYNC_MARKERS {
                    if lower_contents.contains(marker) {
                        return PremiseCheck {
                            premise: premise.to_owned(),
                            flagged: true,
                            evidence: format!(
                                "DKG/file contradiction: {} contains {}",
                                resolved.display(),
                                marker
                            ),
                        };
                    }
                }
            }
        }
    }

    PremiseCheck {
        premise: premise.to_owned(),
        flagged: false,
        evidence: String::new(),
    }
}

/// Determine the route decision from the spec + gate mode + risk level.
/// ASK only when a question is expected to improve execution AND the
/// action is costly/irreversible. Never on every ambiguity.
fn decide_route(
    spec: &IntentSpec,
    gate_mode: GateMode,
    risk: RiskLevel,
) -> RouteDecision {
    let flag_count = spec.premise_flag_count();
    let has_assumptions = spec.assumption_count() > 0;
    match (gate_mode, risk) {
        (GateMode::Interactive, RiskLevel::High) if flag_count > 0 => RouteDecision::Ask,
        (GateMode::Interactive, RiskLevel::High) if has_assumptions => RouteDecision::Ask,
        (GateMode::Autonomous, RiskLevel::High) => RouteDecision::Escalate,
        (GateMode::Autonomous, RiskLevel::Medium) if flag_count > 0 => RouteDecision::ProceedUnderAssumptions,
        (GateMode::Autonomous, _) => RouteDecision::ProceedUnderAssumptions,
        (GateMode::Interactive, _) => RouteDecision::ProceedUnderAssumptions,
    }
}

/// Classify reversibility from the task text. At Tier-1 this is a
/// keyword heuristic; the real classification composes the shipped
/// RiskLevel + TreeAutoAdvancePolicy in Tier-2.
fn classify_reversibility(task_text: &str) -> ReversibilityClass {
    let lower = task_text.to_lowercase();
    if lower.contains("delete") || lower.contains("drop") || lower.contains("force push") {
        ReversibilityClass::Irreversible
    } else if lower.contains("deploy") || lower.contains("publish") || lower.contains("commit") {
        ReversibilityClass::HighCost
    } else {
        ReversibilityClass::Reversible
    }
}

/// Compute the autonomous default from the SSB outcome journal (revealed
/// behaviour), NOT a population/majority prior. At Tier-1 with an empty
/// journal, falls back to the minimally-presumptive reversible action.
fn journal_sourced_default(
    ssb: Option<&Arc<Ssb>>,
    spec: &IntentSpec,
) -> RouteDecision {
    if let Some(writer) = ssb {
        if let Ok(rows) = writer.query_by_type(SsbActionType::IntentControlEvent(
            IntentControlKind::RouteDecided,
        )) {
            let proceed_count = rows
                .iter()
                .filter_map(|r| {
                    let decoded: Result<SsbPayload, _> = rmp_serde::from_slice(&r.payload);
                    decoded.ok()
                })
                .filter_map(|p| match p {
                    SsbPayload::IntentControlEvent(p) => Some(p.route_decision),
                    _ => None,
                })
                .filter(|d| d == "proceed_under_assumptions")
                .count();
            let ask_count = rows
                .iter()
                .filter_map(|r| {
                    let decoded: Result<SsbPayload, _> = rmp_serde::from_slice(&r.payload);
                    decoded.ok()
                })
                .filter_map(|p| match p {
                    SsbPayload::IntentControlEvent(p) => Some(p.route_decision),
                    _ => None,
                })
                .filter(|d| d == "ask")
                .count();
            if proceed_count + ask_count > 0 {
                return if proceed_count >= ask_count {
                    RouteDecision::ProceedUnderAssumptions
                } else {
                    RouteDecision::Ask
                };
            }
        }
    }
    let _ = spec;
    RouteDecision::ProceedUnderAssumptions
}

/// Build a stable spec_id from the task text (blake3 prefix, hex).
fn spec_id_from_task(task_text: &str) -> String {
    let hash = blake3::hash(task_text.as_bytes());
    let prefix: [u8; 8] = hash.as_bytes()[..8].try_into().unwrap_or([0u8; 8]);
    u64::from_le_bytes(prefix).to_string()
}

/// Emit an IntentControlEvent SSB row.
async fn emit_event(
    ssb: &Arc<Ssb>,
    kind: IntentControlKind,
    spec_id: &str,
    premise_flag_count: u32,
    route_decision: &str,
    assumption_count: u32,
    reversibility_class: &str,
    gate_mode: &str,
    details_json: serde_json::Value,
    ts_ms: i64,
) {
    let payload = SsbPayload::IntentControlEvent(IntentControlEventPayload {
        kind,
        spec_id: spec_id.to_owned(),
        premise_flag_count,
        route_decision: route_decision.to_owned(),
        assumption_count,
        reversibility_class: reversibility_class.to_owned(),
        gate_mode: gate_mode.to_owned(),
        details_json,
        ts_ms,
    });
    let action_type = SsbActionType::IntentControlEvent(kind);
    let bytes = rmp_serde::to_vec_named(&payload).unwrap_or_default();
    let req = SsbWriteRequest {
        action_type,
        agent_id: INTENT_CONTROL_AGENT_ID.to_owned(),
        superstep: 0,
        payload: bytes,
        files: Vec::new(),
    };
    if let Err(e) = ssb.write(req).await {
        warn!(error = %e, "intent_control: SSB write failed (non-fatal)");
    }
}

/// Top-level entry: run the intent-control layer on a task message.
/// OBSERVE-ONLY at Tier-1: emits IntentControlEvent rows but does NOT
/// block decomposition. Default-OFF behind ELAI_INTENT_CONTROL_ENABLE.
///
/// `ssb` is the live SSB writer. `clock` is the clock provider for
/// timestamps. `gate_mode` and `risk` come from the resolved config.
pub async fn run_intent_control(
    task_text: &str,
    ssb: &Arc<Ssb>,
    clock: Arc<dyn ClockProvider>,
    gate_mode: GateMode,
    risk: RiskLevel,
    project_root: Option<&Path>,
) {
    let trimmed = task_text.trim();
    if trimmed.is_empty() {
        return;
    }

    let ts_ms = Ssb::now_ms_via_clock(&clock);
    let spec_id = spec_id_from_task(trimmed);

    let premises = extract_premises(trimmed);
    let premise_checks: Vec<PremiseCheck> = premises
        .iter()
        .map(|p| check_premise(p, trimmed, project_root))
        .collect();

    let spec = IntentSpec {
        spec_id: spec_id.clone(),
        goal: trimmed.to_owned(),
        constraints: Vec::new(),
        hidden_assumptions: premise_checks
            .iter()
            .filter(|p| !p.flagged)
            .map(|p| p.premise.clone())
            .collect(),
        non_goals: Vec::new(),
        premise_checks,
    };

    let flag_count = spec.premise_flag_count();
    let assumption_count = spec.assumption_count();
    let reversibility = classify_reversibility(trimmed);

    let route = if flag_count > 0 {
        decide_route(&spec, gate_mode, risk)
    } else {
        journal_sourced_default(Some(ssb), &spec)
    };

    let gate_mode_str = match gate_mode {
        GateMode::Interactive => "interactive",
        GateMode::Autonomous => "autonomous",
    };

    emit_event(
        ssb,
        IntentControlKind::SpecReconstructed,
        &spec_id,
        flag_count,
        route.as_str(),
        assumption_count,
        reversibility.as_str(),
        gate_mode_str,
        serde_json::json!({
            "goal": spec.goal,
            "constraints": spec.constraints,
            "hidden_assumptions": spec.hidden_assumptions,
            "non_goals": spec.non_goals,
        }),
        ts_ms,
    )
    .await;

    for check in &spec.premise_checks {
        if check.flagged {
            emit_event(
                ssb,
                IntentControlKind::PremiseFlagged,
                &spec_id,
                flag_count,
                route.as_str(),
                assumption_count,
                reversibility.as_str(),
                gate_mode_str,
                serde_json::json!({
                    "premise": check.premise,
                    "evidence": check.evidence,
                }),
                ts_ms,
            )
            .await;
        }
    }

    emit_event(
        ssb,
        IntentControlKind::RouteDecided,
        &spec_id,
        flag_count,
        route.as_str(),
        assumption_count,
        reversibility.as_str(),
        gate_mode_str,
        serde_json::json!({
            "route": route.as_str(),
            "reversibility": reversibility.as_str(),
            "gate_mode": gate_mode_str,
        }),
        ts_ms,
    )
    .await;

    for assumption in &spec.hidden_assumptions {
        emit_event(
            ssb,
            IntentControlKind::AssumptionLogged,
            &spec_id,
            flag_count,
            route.as_str(),
            assumption_count,
            reversibility.as_str(),
            gate_mode_str,
            serde_json::json!({
                "assumption": assumption,
            }),
            ts_ms,
        )
        .await;
    }

    info!(
        spec_id = %spec_id,
        flag_count,
        assumption_count,
        route = route.as_str(),
        reversibility = reversibility.as_str(),
        "intent_control: observe-only pass complete"
    );
}

/// Check whether the intent-control layer is enabled.
/// Default-OFF: returns false unless ELAI_INTENT_CONTROL_ENABLE=1.
pub fn intent_control_enabled() -> bool {
    std::env::var("ELAI_INTENT_CONTROL_ENABLE")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Production gate for J-35: default-OFF. `drive_task` calls this; contracts
/// must call THIS SAME fn so deleting the guard makes the OFF-path test red.
pub async fn maybe_run_intent_control(
    task_text: &str,
    ssb: &Arc<Ssb>,
    clock: Arc<dyn ClockProvider>,
    gate_mode: GateMode,
    risk: RiskLevel,
    project_root: Option<&Path>,
) {
    if !intent_control_enabled() {
        return;
    }
    run_intent_control(task_text, ssb, clock, gate_mode, risk, project_root).await;
}
