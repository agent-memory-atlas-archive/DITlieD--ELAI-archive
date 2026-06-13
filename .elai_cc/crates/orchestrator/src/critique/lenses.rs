// cross-lang-skip: tauri-svelte
//! I-41 — spec-time critique lenses (L1-L8).
//!
//! Advisory-only: lens findings are surfaced to humans and emitted as
//! [`CritiqueLensFinding`] SSB rows (advisory=true). They NEVER influence
//! any accept/block verdict. The isolation is compiler-enforced: this module
//! does NOT import `idea_framing::CriticFinding`, `idea_framing::Severity`,
//! or any gate-plane type from `directive.rs`.
//!
//! # Kill-switch
//! Set `ELAI_CRITIQUE_LENSES_ENABLE=1` to enable. Default-OFF (R9).
//! Registered in `wire-dark-allowlist.toml` (I-41 row).
//!
//! # Grounding
//! Source design: workflow/research/grok-elai-trust/ELAI-CRITIQUE-LENSES.md.
//! Noise evidence: MSR-2026 60.2% 0-30% signal band; NAACL Mar-2025 all-8
//! defenses bypassed >50% ASR -- advisory-only, never gate-verdict.
//! Compose-not-restate (GD-1): L3 names ambiguities, AmbiguityScorer gates
//! on numbers; L6 names drift, cross_lang_diff.py gates on it.

use std::collections::HashSet;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::ssb::{
    CritiqueLensFindingKind, CritiqueLensFindingPayload, CritiqueLensFindingTag,
    Ssb, SsbActionType, SsbPayload, SsbWriteRequest,
};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Agent-id tag for SSB rows emitted by the lens runner.
const LENS_AGENT_ID: &str = "critique:lens-runner";

/// Maximum findings per lens per pass (anti-noise cap, per plan).
const MAX_FINDINGS_PER_LENS: usize = 8;

/// Kill-switch env var.  Default-OFF.
const ENABLE_ENV: &str = "ELAI_CRITIQUE_LENSES_ENABLE";

// ---------------------------------------------------------------------------
// Lens prompt text (include_str! from src/critique/prompts/)
// ---------------------------------------------------------------------------

const L1_PROMPT: &str = include_str!("prompts/L1.md");
const L2_PROMPT: &str = include_str!("prompts/L2.md");
const L3_PROMPT: &str = include_str!("prompts/L3.md");
const L4_PROMPT: &str = include_str!("prompts/L4.md");
const L5_PROMPT: &str = include_str!("prompts/L5.md");
const L6_PROMPT: &str = include_str!("prompts/L6.md");
const L7_PROMPT: &str = include_str!("prompts/L7.md");
const L8_PROMPT: &str = include_str!("prompts/L8.md");

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Which critique lens produced a finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LensId {
    L1AssumptionExcavator,
    L2ImpliedCompleteness,
    L3AmbiguityMapper,
    L4GapAnalyzer,
    L5PreImplArchitect,
    L6ApiContractValidator,
    L7DocumentationValidator,
    L8AnxietyReader,
}

impl LensId {
    /// Human-readable label used in SSB rows and CLI output.
    pub fn label(self) -> &'static str {
        match self {
            Self::L1AssumptionExcavator => "L1-assumption-excavator",
            Self::L2ImpliedCompleteness => "L2-implied-completeness",
            Self::L3AmbiguityMapper => "L3-ambiguity-mapper",
            Self::L4GapAnalyzer => "L4-gap-analyzer",
            Self::L5PreImplArchitect => "L5-pre-impl-architect",
            Self::L6ApiContractValidator => "L6-api-contract-validator",
            Self::L7DocumentationValidator => "L7-documentation-validator",
            Self::L8AnxietyReader => "L8-anxiety-reader",
        }
    }

    /// Ordered default set for HEAVY risk: all 8 lenses.
    pub fn all() -> &'static [LensId] {
        &[
            LensId::L1AssumptionExcavator,
            LensId::L2ImpliedCompleteness,
            LensId::L3AmbiguityMapper,
            LensId::L4GapAnalyzer,
            LensId::L5PreImplArchitect,
            LensId::L6ApiContractValidator,
            LensId::L7DocumentationValidator,
            LensId::L8AnxietyReader,
        ]
    }

    /// Default set for SIMPLE risk: L1, L4, L8 (low-noise trio per plan rationale).
    pub fn simple_set() -> &'static [LensId] {
        &[
            LensId::L1AssumptionExcavator,
            LensId::L4GapAnalyzer,
            LensId::L8AnxietyReader,
        ]
    }

    fn prompt_template(self) -> &'static str {
        match self {
            Self::L1AssumptionExcavator => L1_PROMPT,
            Self::L2ImpliedCompleteness => L2_PROMPT,
            Self::L3AmbiguityMapper => L3_PROMPT,
            Self::L4GapAnalyzer => L4_PROMPT,
            Self::L5PreImplArchitect => L5_PROMPT,
            Self::L6ApiContractValidator => L6_PROMPT,
            Self::L7DocumentationValidator => L7_PROMPT,
            Self::L8AnxietyReader => L8_PROMPT,
        }
    }
}

/// Risk level controls which lenses run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 2-3 lenses (L1, L4, L8).
    Simple,
    /// Full 8-lens pass.
    Heavy,
}

/// Advisory severity.  Distinct from `idea_framing::Severity` by design.
/// No `Into<idea_framing::Severity>` impl exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LensSeverity {
    /// Surface as a blocking candidate to the human; never auto-blocks.
    Flag,
    /// Warrants human review; non-urgent.
    Concern,
    /// Informational.
    Note,
}

impl LensSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Flag => "FLAG",
            Self::Concern => "CONCERN",
            Self::Note => "NOTE",
        }
    }
}

/// A single critique finding from one lens pass.
///
/// The `location` field is REQUIRED non-empty. Any raw finding with an empty
/// location is dropped via [`drop_if_no_anchor`] before surfacing, and a
/// `DroppedNoAnchor` SSB row is emitted instead.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LensFinding {
    pub lens: LensId,
    pub severity: LensSeverity,
    /// Concrete anchor (spec heading / quoted phrase / file:line / symbol).
    /// Non-empty enforced by [`drop_if_no_anchor`].
    pub location: String,
    pub claim: String,
    pub why_it_matters: String,
    pub suggested_resolution: String,
    /// 0.0 – 1.0 model-reported confidence.
    pub confidence: f32,
}

/// Error type for [`LensSet::run`].
#[derive(Debug, thiserror::Error)]
pub enum LensCritiqueError {
    #[error("runtime required for LensSet but not provided (runtime=None)")]
    NoRuntime,
    #[error("lens {lens} LLM call failed: {source}")]
    LlmFailed {
        lens: &'static str,
        #[source]
        source: elai_providers::runtime::types::RuntimeAdapterError,
    },
    #[error("lens {lens} response JSON parse failed: {msg}")]
    ParseFailed { lens: &'static str, msg: String },
}

// ---------------------------------------------------------------------------
// LensSet
// ---------------------------------------------------------------------------

/// The configured set of critique lenses for one spec-time run.
pub struct LensSet {
    lenses: Vec<LensId>,
    risk_level: RiskLevel,
}

impl LensSet {
    /// Create with explicit lens list.
    pub fn new(lenses: Vec<LensId>, risk_level: RiskLevel) -> Self {
        Self { lenses, risk_level }
    }

    /// Create from a risk level using the default lens selection.
    pub fn from_risk(risk_level: RiskLevel) -> Self {
        let lenses = match risk_level {
            RiskLevel::Simple => LensId::simple_set().to_vec(),
            RiskLevel::Heavy => LensId::all().to_vec(),
        };
        Self { lenses, risk_level }
    }

    /// Run all configured lenses over `spec_text`.
    ///
    /// When `runtime` is `None` or `ELAI_CRITIQUE_LENSES_ENABLE` is not set,
    /// returns an empty `Vec<LensFinding>` (advisory-absent is never a block).
    ///
    /// Emits [`SsbPayload::CritiqueLensFinding`] rows for each finding and for
    /// each lens run. Anchorless raw findings produce a `DroppedNoAnchor` row
    /// and are excluded from the returned `Vec`.
    pub async fn run(
        &self,
        spec_text: &str,
        runtime: Option<&dyn elai_providers::runtime::types::RuntimeAdapter>,
        ssb: Option<&Arc<Ssb>>,
        session_id: &str,
    ) -> Result<Vec<LensFinding>, LensCritiqueError> {
        // Kill-switch check.
        if std::env::var(ENABLE_ENV).as_deref() != Ok("1") {
            return Ok(Vec::new());
        }

        let rt = match runtime {
            Some(r) => r,
            None => return Err(LensCritiqueError::NoRuntime),
        };

        let mut all_findings: Vec<LensFinding> = Vec::new();
        let mut dedup_keys: HashSet<(String, String)> = HashSet::new();

        for &lens_id in &self.lenses {
            let findings = self
                .run_one_lens(lens_id, spec_text, rt, ssb, session_id)
                .await;

            match findings {
                Ok(raw_findings) => {
                    for raw in raw_findings {
                        match drop_if_no_anchor(raw) {
                            Some(f) => {
                                let key =
                                    (f.location.clone(), f.claim.clone());
                                if dedup_keys.insert(key) {
                                    emit_advisory_finding(&f, ssb, session_id).await;
                                    all_findings.push(f);
                                }
                            }
                            None => {
                                // DroppedNoAnchor SSB row already emitted inside
                                // drop_if_no_anchor's caller — emit it here.
                                emit_dropped_no_anchor(lens_id, ssb, session_id).await;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        lens = lens_id.label(),
                        err = %e,
                        "lens run failed; skipping (advisory-absent, not a block)"
                    );
                }
            }

            // Emit LensRun beacon per lens (even on failure — advisory telemetry).
            emit_lens_run(lens_id, ssb, session_id).await;
        }

        Ok(all_findings)
    }

    async fn run_one_lens(
        &self,
        lens_id: LensId,
        spec_text: &str,
        runtime: &dyn elai_providers::runtime::types::RuntimeAdapter,
        _ssb: Option<&Arc<Ssb>>,
        _session_id: &str,
    ) -> Result<Vec<LensFinding>, LensCritiqueError> {
        use elai_providers::runtime::types::{CompletionConfig, Message, MessageRole};

        let prompt_text = lens_id
            .prompt_template()
            .replace("{{spec_text}}", spec_text);

        let messages = vec![Message {
            role: MessageRole::User,
            content: prompt_text,
        }];

        let config = CompletionConfig {
            max_tokens: 2048,
            temperature: 0.0,
            ..Default::default()
        };

        let response = runtime
            .complete(messages, config)
            .await
            .map_err(|e| LensCritiqueError::LlmFailed {
                lens: lens_id.label(),
                source: e,
            })?;

        parse_findings_from_response(lens_id, &response.text)
    }
}

// ---------------------------------------------------------------------------
// Anchor enforcement
// ---------------------------------------------------------------------------

/// Returns `Some(finding)` when `location` is non-empty; `None` when the
/// finding must be dropped (caller emits a `DroppedNoAnchor` SSB row).
pub fn drop_if_no_anchor(raw: LensFinding) -> Option<LensFinding> {
    if raw.location.trim().is_empty() {
        None
    } else {
        Some(raw)
    }
}

// ---------------------------------------------------------------------------
// Dedup: by (location, claim) across lenses
// ---------------------------------------------------------------------------

/// Remove duplicate findings keyed by `(location, claim)`.  Order-preserving,
/// first occurrence wins.
pub fn dedup_findings(findings: Vec<LensFinding>) -> Vec<LensFinding> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    findings
        .into_iter()
        .filter(|f| seen.insert((f.location.clone(), f.claim.clone())))
        .collect()
}

// ---------------------------------------------------------------------------
// JSON parsing helpers
// ---------------------------------------------------------------------------

fn parse_findings_from_response(
    lens_id: LensId,
    text: &str,
) -> Result<Vec<LensFinding>, LensCritiqueError> {
    // Extract the first JSON array from the response (the model may wrap it
    // in a fenced code block or prose).
    let json_str = extract_json_array(text).ok_or_else(|| LensCritiqueError::ParseFailed {
        lens: lens_id.label(),
        msg: "no JSON array found in response".into(),
    })?;

    let raw: Vec<serde_json::Value> =
        serde_json::from_str(&json_str).map_err(|e| LensCritiqueError::ParseFailed {
            lens: lens_id.label(),
            msg: e.to_string(),
        })?;

    let mut findings = Vec::new();
    for item in raw.into_iter().take(MAX_FINDINGS_PER_LENS) {
        if let Some(f) = parse_one_finding(lens_id, &item) {
            findings.push(f);
        }
    }
    Ok(findings)
}

fn extract_json_array(text: &str) -> Option<String> {
    // Try fenced block first.
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim().to_owned());
        }
    }
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            let candidate = after[..end].trim();
            if candidate.starts_with('[') {
                return Some(candidate.to_owned());
            }
        }
    }
    // Fallback: find the first '[' ... ']' pair.
    let start = text.find('[')?;
    // Find the matching closing bracket.
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut end = start;
    for (i, &b) in bytes[start..].iter().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    end = start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    if end > start {
        Some(text[start..=end].to_owned())
    } else {
        None
    }
}

fn parse_one_finding(lens_id: LensId, v: &serde_json::Value) -> Option<LensFinding> {
    let location = v.get("location")?.as_str()?.to_owned();
    let claim = v.get("claim")?.as_str()?.to_owned();
    let why_it_matters = v
        .get("why_it_matters")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_owned();
    let suggested_resolution = v
        .get("suggested_resolution")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_owned();
    let confidence = v
        .get("confidence")
        .and_then(|x| x.as_f64())
        .unwrap_or(0.5) as f32;

    let severity_str = v
        .get("severity")
        .and_then(|x| x.as_str())
        .unwrap_or("NOTE");
    let severity = match severity_str {
        "FLAG" | "BLOCKER-CANDIDATE" => LensSeverity::Flag,
        "CONCERN" => LensSeverity::Concern,
        _ => LensSeverity::Note,
    };

    Some(LensFinding {
        lens: lens_id,
        severity,
        location,
        claim,
        why_it_matters,
        suggested_resolution,
        confidence,
    })
}

// ---------------------------------------------------------------------------
// SSB write helper (local to this module — Ssb::write takes SsbWriteRequest)
// ---------------------------------------------------------------------------

async fn write_ssb_row(
    writer: &Ssb,
    action_type: SsbActionType,
    agent_id: &str,
    payload: SsbPayload,
) {
    match rmp_serde::to_vec_named(&payload) {
        Ok(bytes) => {
            let req = SsbWriteRequest {
                action_type,
                agent_id: agent_id.to_owned(),
                superstep: 0,
                payload: bytes,
                files: Vec::new(),
            };
            let _ = writer.write(req).await;
        }
        Err(e) => {
            warn!(err = %e, "critique: failed to serialize SSB payload; advisory row dropped");
        }
    }
}

// ---------------------------------------------------------------------------
// SSB emission helpers
// ---------------------------------------------------------------------------

async fn emit_advisory_finding(
    finding: &LensFinding,
    ssb: Option<&Arc<Ssb>>,
    session_id: &str,
) {
    let Some(writer) = ssb else { return };
    let payload = SsbPayload::CritiqueLensFinding(CritiqueLensFindingPayload {
        kind: CritiqueLensFindingKind::AdvisoryFinding {
            lens: finding.lens.label().to_owned(),
            severity: finding.severity.as_str().to_owned(),
            location: finding.location.clone(),
            claim: finding.claim.clone(),
            confidence: finding.confidence,
        },
        advisory: true,
        session_id: session_id.to_owned(),
    });
    write_ssb_row(
        writer,
        SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::AdvisoryFinding),
        LENS_AGENT_ID,
        payload,
    )
    .await;
}

async fn emit_dropped_no_anchor(
    lens_id: LensId,
    ssb: Option<&Arc<Ssb>>,
    session_id: &str,
) {
    let Some(writer) = ssb else { return };
    let payload = SsbPayload::CritiqueLensFinding(CritiqueLensFindingPayload {
        kind: CritiqueLensFindingKind::DroppedNoAnchor {
            lens: lens_id.label().to_owned(),
        },
        advisory: true,
        session_id: session_id.to_owned(),
    });
    write_ssb_row(
        writer,
        SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::DroppedNoAnchor),
        LENS_AGENT_ID,
        payload,
    )
    .await;
}

async fn emit_lens_run(
    lens_id: LensId,
    ssb: Option<&Arc<Ssb>>,
    session_id: &str,
) {
    let Some(writer) = ssb else { return };
    let payload = SsbPayload::CritiqueLensFinding(CritiqueLensFindingPayload {
        kind: CritiqueLensFindingKind::LensRun {
            lens: lens_id.label().to_owned(),
        },
        advisory: true,
        session_id: session_id.to_owned(),
    });
    write_ssb_row(
        writer,
        SsbActionType::CritiqueLensFinding(CritiqueLensFindingTag::LensRun),
        LENS_AGENT_ID,
        payload,
    )
    .await;
}
