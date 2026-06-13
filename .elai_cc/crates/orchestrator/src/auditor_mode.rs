// B-41: Per-Level Auditor Modes
//
// Six auditor modes — Concepts, Phases, Steps, BabySteps, BabyStepDetail, Plan
// — replace the single-cycle Plan-only auditor with tier-scoped audit
// dispatch. Each mode owns a tier-scoped REVIEW prompt (see
// .claude/agents/auditor.md MODE DISPATCH section) and a tier-scoped GRC
// budget (see reasoning/src/config.rs:agent_grc_config_for_role).
//
// Plan mode is retained on AgentRole::Auditor for back-compat — existing
// callers that spawn `Auditor` see no behaviour change. The 5 new variants
// (AuditorConcepts/AuditorPhases/AuditorSteps/AuditorBabySteps/
// AuditorBabyStepDetail) gate the per-tier dispatch in
// pipeline_tree_navigator (Slice 3).
//
// BLOCK policy: on a tier-N BLOCK verdict, respawn the level-N decomposer
// ONCE with the findings as a `rescope_from_audit` hint. Second BLOCK
// cascades via emit_integration_bubble_up (sibling category
// AUDITOR_BUBBLE_UP_CATEGORY). One-shot respawn is monotonic
// (respawned_once moves false -> true, never back) — this matches plan §3
// task 4 stub_ban: re-auditing the same output produces the same verdict by
// construction.
//
// Design hypothesis (per workflow/reasoning/B-41-decisions.md DECISION-PAPER):
// per-level scoping improves recall AND precision for tiered verification.
// Closest empirical citation: arXiv:2404.03602 (Kamoi COLM 2024) — prompt
// scope changes recall by ~17-37pp without consistent direction. arXiv:2406.01297
// (Kamoi TACL 2024) does NOT directly claim this; per-level scoping remains
// a design hypothesis until B-41 ships and verdict telemetry is collected.

use serde::{Deserialize, Serialize};

use elai_ipc_types::AgentRole;

use crate::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};

// ---------------------------------------------------------------------------
// AuditorMode — six variants
// ---------------------------------------------------------------------------

/// Per-level auditor mode. Drives both the AgentRole spawned by the
/// orchestrator and the REVIEW prompt loaded into the auditor agent.
///
/// Mapping to AgentRole is via [`AuditorMode::map_to_agent_role`]. The Plan
/// variant maps to the pre-existing [`AgentRole::Auditor`] (no behaviour
/// change for legacy callers); modes 1-4.5 map to the 5 new B-41 variants.
///
/// `Visual` is reserved for B-43 (visual audit on rendered dashboard) and
/// only compiles under the `visual_audit` feature flag. Without the flag the
/// variant is absent from the enum, so no match arm obligation propagates to
/// downstream code until B-43 ships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditorMode {
    /// Tier-1 concept-level audit (market fit, IP boundaries, scope,
    /// contradictions vs FramedIdea, grounding against cosmos briefing).
    Concepts,
    /// Tier-2 phase-level audit (ordering, contract sufficiency, dependency
    /// cycles).
    Phases,
    /// Tier-3 step-level audit (granularity, test coverage of step
    /// boundaries).
    Steps,
    /// Tier-4 babystep-batch audit (BabyStepSpec 8-block completeness:
    /// GROUNDING / PRE_GATE / TASK / TDD / DETERMINISTIC_CHECK / ACCEPTANCE
    /// / INTEGRATION / EXIT_CRITERIA).
    BabySteps,
    /// Tier-4.5 line-level pass over babystep batch outputs (lazy code,
    /// unwrap in production, hardcoded secrets, stale APIs). Cascades to
    /// BabyStep tier on second BLOCK (see DECISION-CASCADE in
    /// B-41-decisions.md).
    BabyStepDetail,
    /// Plan-mode — single-cycle integration pass at end of plan. Back-compat
    /// route for pre-B-41 callers.
    Plan,
    /// E-12 (E-11 prereq, fulfilled in E-12) — plan-scope production-grade
    /// auditor that runs the 15-item production checklist (secrets scan,
    /// OWASP semgrep, RLS probe, dep audit, OTel/Sentry presence,
    /// lighthouse, biome/ruff/clippy-deny, etc.) against the emitted
    /// FilePlan + selected FrameworkProfile. Dispatches the
    /// [`AgentRole::AuditorProductionGrade`] spawn. See
    /// `crate::production_checklist::*` + `crate::framework_profile`.
    ///
    /// Wire-dark posture: 15 runner modules ship the full surface
    /// (real subprocess wires, real timeout, real ToolchainMissing
    /// branch) but the 15 external CLIs are not installed by default on
    /// the local dev host. The runner returns
    /// `production_checklist::RunResult::ToolchainMissing` instead of a
    /// stub or panic; live invocation is RUNPOD-PENDING per
    /// `workflow/reasoning/E-12-tier-b-decisions.md`.
    ProductionGrade,
    /// Reserved for B-43. Compiles only under `visual_audit`.
    #[cfg(feature = "visual_audit")]
    Visual,
}

impl AuditorMode {
    /// Map an AuditorMode to its concrete AgentRole. Plan mode keeps the
    /// pre-existing [`AgentRole::Auditor`] for back-compat; the 5 per-level
    /// modes map to the B-41-added unit variants.
    pub fn map_to_agent_role(self) -> AgentRole {
        match self {
            Self::Concepts => AgentRole::AuditorConcepts,
            Self::Phases => AgentRole::AuditorPhases,
            Self::Steps => AgentRole::AuditorSteps,
            Self::BabySteps => AgentRole::AuditorBabySteps,
            Self::BabyStepDetail => AgentRole::AuditorBabyStepDetail,
            Self::Plan => AgentRole::Auditor,
            Self::ProductionGrade => AgentRole::AuditorProductionGrade,
            #[cfg(feature = "visual_audit")]
            Self::Visual => AgentRole::Auditor, // B-43: replace with AgentRole::AuditorVisual when added.
        }
    }

    /// Canonical snake_case label — used in SSB payloads + dashboard badges.
    /// Mirrors `bsp::engine::agent_role_str` for the auditor family.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Concepts => "concepts",
            Self::Phases => "phases",
            Self::Steps => "steps",
            Self::BabySteps => "babysteps",
            Self::BabyStepDetail => "babystep_detail",
            Self::Plan => "plan",
            Self::ProductionGrade => "production_grade",
            #[cfg(feature = "visual_audit")]
            Self::Visual => "visual",
        }
    }
}

// ---------------------------------------------------------------------------
// TierNodeId — typed sum over the 4 decomposition ID newtypes
// ---------------------------------------------------------------------------

/// Strongly-typed identifier for the tier node under audit. Carries the
/// underlying newtype (ConceptId / PhaseId / StepId / BabyStepId) so the
/// mode/id pair cannot be silently mismatched at the call site.
///
/// BabyStepDetail does NOT have its own variant — that mode audits a batch of
/// BabyStep outputs and is keyed on the same BabyStepId as the parent
/// BabyStep tier. See B-41-decisions.md DECISION-CASCADE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "tier", content = "id", rename_all = "snake_case")]
pub enum TierNodeId {
    Concept(ConceptId),
    Phase(PhaseId),
    Step(StepId),
    BabyStep(BabyStepId),
}

impl TierNodeId {
    /// Returns the underlying u64 id. Useful for SSB payload columns where
    /// the tier is encoded as a separate string column.
    pub fn raw_id(self) -> u64 {
        match self {
            Self::Concept(ConceptId(v)) => v,
            Self::Phase(PhaseId(v)) => v,
            Self::Step(StepId(v)) => v,
            Self::BabyStep(BabyStepId(v)) => v,
        }
    }

    /// Tier label ("concept" | "phase" | "step" | "baby_step").
    pub fn tier_str(self) -> &'static str {
        match self {
            Self::Concept(_) => "concept",
            Self::Phase(_) => "phase",
            Self::Step(_) => "step",
            Self::BabyStep(_) => "baby_step",
        }
    }
}

// ---------------------------------------------------------------------------
// Verdict + Severity + Finding
// ---------------------------------------------------------------------------

/// Allow / Block verdict from a per-level audit pass.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Allow,
    Block,
}

/// Severity of an individual audit finding. BLOCK findings drive Verdict::Block
/// at the verdict-aggregation step; WARN/INFO are advisory only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Block,
    Warn,
    Info,
}

/// A single audit observation. `category` carries a C-* tag (e.g.
/// "C-MARKET-FIT" for Concepts, "C-LAZY-CODE" for BabyStepDetail) or a
/// custom string; downstream consumers (rule-cluster loader, dashboard)
/// group by this field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub category: String,
    pub severity: Severity,
    pub message: String,
    /// Optional pointer to the offending span (file:line, node id, etc).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

// ---------------------------------------------------------------------------
// LevelAuditVerdict
// ---------------------------------------------------------------------------

/// Outcome of a single per-level audit pass. Persisted to SSB via
/// `per_level_audit_verdict` (see ssb.rs SCHEMA_VERSION 45 bump).
///
/// `respawned_once` is the monotonic single-respawn guard (false -> true,
/// never back); second BLOCK at the same tier sets `cascade_to_parent=true`
/// and triggers the existing emit_integration_bubble_up path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelAuditVerdict {
    pub mode: AuditorMode,
    pub tier_node_id: TierNodeId,
    pub findings: Vec<Finding>,
    pub verdict: Verdict,
    pub respawned_once: bool,
    pub cascade_to_parent: bool,
}

impl LevelAuditVerdict {
    /// Convenience constructor for an Allow verdict with no findings.
    pub fn allow(mode: AuditorMode, tier_node_id: TierNodeId) -> Self {
        Self {
            mode,
            tier_node_id,
            findings: Vec::new(),
            verdict: Verdict::Allow,
            respawned_once: false,
            cascade_to_parent: false,
        }
    }

    /// Aggregate verdict from a finding list — any BLOCK finding yields
    /// Verdict::Block, otherwise Allow.
    pub fn from_findings(
        mode: AuditorMode,
        tier_node_id: TierNodeId,
        findings: Vec<Finding>,
    ) -> Self {
        let verdict = if findings.iter().any(|f| f.severity == Severity::Block) {
            Verdict::Block
        } else {
            Verdict::Allow
        };
        Self {
            mode,
            tier_node_id,
            findings,
            verdict,
            respawned_once: false,
            cascade_to_parent: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuditorModeError {
    /// AuditorMode is set to BabyStepDetail but the cascade target tier is
    /// not BabyStep — should be impossible by construction (BabyStepDetail
    /// only cascades to BabyStep). Surfaced as a programmer error.
    #[error("BabyStepDetail cascade target must be BabyStep tier, got {0}")]
    InvalidCascadeTarget(&'static str),
    /// Attempted to respawn a tier-N decomposer when `respawned_once` is
    /// already true. Violates the monotonic single-respawn guard.
    #[error("double respawn forbidden at tier {0} node {1}")]
    DoubleRespawn(&'static str, u64),
    /// Persistence failure (filesystem write or SSB queue).
    #[error("verdict persistence failed: {0}")]
    Persistence(String),
}

// ---------------------------------------------------------------------------
// Constants — cascade category + artifact paths
// ---------------------------------------------------------------------------

/// SSB integration_bubble_up category constant for auditor-driven cascades.
/// Sibling of INTEGRATION_BUBBLE_UP_CATEGORY in task_runner.rs. Keeping a
/// distinct constant lets downstream filters separate auditor-cascade rows
/// from test-failure cascades without parsing the SSB payload.
pub const AUDITOR_BUBBLE_UP_CATEGORY: &str = "auditor_bubble_up";

/// SSB action_type strings emitted by B-41 verdict persistence (lockstep with
/// ssb.rs SsbActionType variants + as_sql_str + from_str + payload structs).
pub const SSB_AT_PER_LEVEL_AUDIT_STARTED: &str = "per_level_audit_started";
pub const SSB_AT_PER_LEVEL_AUDIT_VERDICT: &str = "per_level_audit_verdict";
pub const SSB_AT_PER_LEVEL_AUDIT_CASCADE: &str = "per_level_audit_cascade";

/// Auditor artifact persistence path template (rendered per session).
/// Verdict markdown is written to
/// `workflow/sessions/{sid}/audits/{mode}/{node_id}.md`.
pub const AUDITS_DIR_SEGMENT: &str = "audits";

// ---------------------------------------------------------------------------
// Persistence helper — verdict markdown
// ---------------------------------------------------------------------------

/// Render a LevelAuditVerdict as a Markdown artifact (deterministic format:
/// no timestamps, no random ordering — `findings` are written in input order).
///
/// Consumed by `persist_verdict` (writes to workflow/sessions/.../audits/) and
/// by dashboard-v2 AuditTimelineWindow for offline display.
pub fn render_verdict_markdown(v: &LevelAuditVerdict) -> String {
    let mut out = String::new();
    out.push_str("# Per-Level Audit Verdict\n\n");
    out.push_str(&format!("mode: {}\n", v.mode.as_str()));
    out.push_str(&format!(
        "tier: {} (id={})\n",
        v.tier_node_id.tier_str(),
        v.tier_node_id.raw_id()
    ));
    out.push_str(&format!(
        "verdict: {}\n",
        match v.verdict {
            Verdict::Allow => "allow",
            Verdict::Block => "block",
        }
    ));
    out.push_str(&format!("respawned_once: {}\n", v.respawned_once));
    out.push_str(&format!("cascade_to_parent: {}\n", v.cascade_to_parent));
    out.push_str(&format!("findings_count: {}\n\n", v.findings.len()));
    if !v.findings.is_empty() {
        out.push_str("## Findings\n\n");
        for f in &v.findings {
            let sev = match f.severity {
                Severity::Block => "BLOCK",
                Severity::Warn => "WARN",
                Severity::Info => "INFO",
            };
            out.push_str(&format!("- [{}] {}: {}", sev, f.category, f.message));
            if let Some(src) = &f.source {
                out.push_str(&format!(" (source: {})", src));
            }
            out.push('\n');
        }
    }
    out
}

/// Persist a verdict markdown to
/// `{session_root}/audits/{mode}/{node_id}.md`. Creates parent directories
/// as needed. Returns the written path.
///
/// Pure-IO helper kept in this module (rather than task_runner) so unit tests
/// can verify the directory layout without booting an SSB.
pub fn persist_verdict(
    session_root: &std::path::Path,
    v: &LevelAuditVerdict,
) -> Result<std::path::PathBuf, AuditorModeError> {
    let dir = session_root
        .join(AUDITS_DIR_SEGMENT)
        .join(v.mode.as_str());
    std::fs::create_dir_all(&dir)
        .map_err(|e| AuditorModeError::Persistence(format!("mkdir {}: {}", dir.display(), e)))?;
    let path = dir.join(format!("{}.md", v.tier_node_id.raw_id()));
    let body = render_verdict_markdown(v);
    // Atomic write: tmp + sync_all + rename via shared helper (I-3 Phase-3C hardening).
    // Converts the prior direct std::fs::write (non-atomic) to the full pattern.
    crate::util::atomic_io::atomic_write_sync(&path, body.as_bytes())
        .map_err(|e| AuditorModeError::Persistence(format!("write {}: {}", path.display(), e)))?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// Respawn-once guard
// ---------------------------------------------------------------------------

/// Monotonic guard: returns `Ok(updated)` with `respawned_once=true` if the
/// verdict has not been respawned yet. Returns `Err(DoubleRespawn)` if it
/// already has — caller must cascade instead.
///
/// Pure function; no SSB or agent_pool side effects. The caller is
/// responsible for actually spawning the decomposer and persisting the
/// updated verdict.
pub fn mark_respawned(v: LevelAuditVerdict) -> Result<LevelAuditVerdict, AuditorModeError> {
    if v.respawned_once {
        return Err(AuditorModeError::DoubleRespawn(
            v.tier_node_id.tier_str(),
            v.tier_node_id.raw_id(),
        ));
    }
    Ok(LevelAuditVerdict {
        respawned_once: true,
        ..v
    })
}

/// Mark a verdict for cascade — sets `cascade_to_parent=true`. Idempotent.
/// Caller must invoke emit_integration_bubble_up with AUDITOR_BUBBLE_UP_CATEGORY
/// after persisting the updated verdict.
pub fn mark_cascade(v: LevelAuditVerdict) -> LevelAuditVerdict {
    LevelAuditVerdict {
        cascade_to_parent: true,
        ..v
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auditor_mode_maps_each_variant_to_distinct_agent_role() {
        // 5 per-level modes each map to a distinct AgentRole; Plan maps back
        // to the pre-existing Auditor for back-compat.
        let pairs = [
            (AuditorMode::Concepts, AgentRole::AuditorConcepts),
            (AuditorMode::Phases, AgentRole::AuditorPhases),
            (AuditorMode::Steps, AgentRole::AuditorSteps),
            (AuditorMode::BabySteps, AgentRole::AuditorBabySteps),
            (
                AuditorMode::BabyStepDetail,
                AgentRole::AuditorBabyStepDetail,
            ),
            (AuditorMode::Plan, AgentRole::Auditor),
            (
                AuditorMode::ProductionGrade,
                AgentRole::AuditorProductionGrade,
            ),
        ];
        for (mode, expected) in pairs {
            assert_eq!(mode.map_to_agent_role(), expected, "mode={:?}", mode);
        }
    }

    #[test]
    fn auditor_mode_str_labels_are_stable() {
        assert_eq!(AuditorMode::Concepts.as_str(), "concepts");
        assert_eq!(AuditorMode::Phases.as_str(), "phases");
        assert_eq!(AuditorMode::Steps.as_str(), "steps");
        assert_eq!(AuditorMode::BabySteps.as_str(), "babysteps");
        assert_eq!(AuditorMode::BabyStepDetail.as_str(), "babystep_detail");
        assert_eq!(AuditorMode::Plan.as_str(), "plan");
        assert_eq!(AuditorMode::ProductionGrade.as_str(), "production_grade");
    }

    #[test]
    fn tier_node_id_raw_id_extracts_underlying_value() {
        assert_eq!(TierNodeId::Concept(ConceptId(7)).raw_id(), 7);
        assert_eq!(TierNodeId::Phase(PhaseId(11)).raw_id(), 11);
        assert_eq!(TierNodeId::Step(StepId(13)).raw_id(), 13);
        assert_eq!(TierNodeId::BabyStep(BabyStepId(17)).raw_id(), 17);
    }

    #[test]
    fn from_findings_block_when_any_block_finding() {
        let findings = vec![
            Finding {
                category: "C-INFO".into(),
                severity: Severity::Info,
                message: "ok".into(),
                source: None,
            },
            Finding {
                category: "C-LAZY-CODE".into(),
                severity: Severity::Block,
                message: "unwrap in production".into(),
                source: Some("foo.rs:42".into()),
            },
        ];
        let v = LevelAuditVerdict::from_findings(
            AuditorMode::BabyStepDetail,
            TierNodeId::BabyStep(BabyStepId(1)),
            findings,
        );
        assert_eq!(v.verdict, Verdict::Block);
        assert!(!v.respawned_once);
        assert!(!v.cascade_to_parent);
    }

    #[test]
    fn from_findings_allow_when_no_block() {
        let findings = vec![Finding {
            category: "C-INFO".into(),
            severity: Severity::Warn,
            message: "minor".into(),
            source: None,
        }];
        let v = LevelAuditVerdict::from_findings(
            AuditorMode::Concepts,
            TierNodeId::Concept(ConceptId(1)),
            findings,
        );
        assert_eq!(v.verdict, Verdict::Allow);
    }

    #[test]
    fn mark_respawned_is_monotonic() {
        let v = LevelAuditVerdict::allow(AuditorMode::Steps, TierNodeId::Step(StepId(5)));
        let v1 = mark_respawned(v).expect("first respawn ok");
        assert!(v1.respawned_once);
        let err = mark_respawned(v1).expect_err("second respawn must fail");
        match err {
            AuditorModeError::DoubleRespawn(tier, id) => {
                assert_eq!(tier, "step");
                assert_eq!(id, 5);
            }
            other => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn render_verdict_markdown_is_deterministic() {
        let v = LevelAuditVerdict {
            mode: AuditorMode::Phases,
            tier_node_id: TierNodeId::Phase(PhaseId(3)),
            findings: vec![Finding {
                category: "C-DEP-CYCLE".into(),
                severity: Severity::Block,
                message: "phase 3 depends on phase 5 which depends on phase 3".into(),
                source: Some("plan.md:84".into()),
            }],
            verdict: Verdict::Block,
            respawned_once: false,
            cascade_to_parent: false,
        };
        let a = render_verdict_markdown(&v);
        let b = render_verdict_markdown(&v);
        assert_eq!(a, b);
        assert!(a.contains("mode: phases"));
        assert!(a.contains("tier: phase (id=3)"));
        assert!(a.contains("verdict: block"));
        assert!(a.contains("[BLOCK] C-DEP-CYCLE"));
    }

    #[test]
    fn persist_verdict_writes_expected_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let v = LevelAuditVerdict::allow(
            AuditorMode::BabySteps,
            TierNodeId::BabyStep(BabyStepId(42)),
        );
        let path = persist_verdict(tmp.path(), &v).expect("persist ok");
        let expected = tmp.path().join("audits").join("babysteps").join("42.md");
        assert_eq!(path, expected);
        assert!(expected.exists());
    }
}
