//! B-41 Slice 3i — audit artifact persistence layout.
//!
//! Locks the on-disk format produced by `auditor_mode::persist_verdict`:
//!   1. Directory structure is `workflow/sessions/{sid}/audits/{mode}/{node_id}.md`.
//!   2. The Markdown file is deterministic (re-rendering same verdict yields
//!      byte-identical output — no timestamps, no random ordering).
//!   3. Every verdict field surfaces in the Markdown so an offline reader
//!      (e.g. the dashboard's AuditTimelineWindow) can reconstruct the
//!      verdict without re-fetching from SSB.

use elai_orchestrator::auditor_mode::{
    persist_verdict, render_verdict_markdown, AuditorMode, Finding, LevelAuditVerdict, Severity,
    TierNodeId, Verdict, AUDITS_DIR_SEGMENT,
};
use elai_orchestrator::research_tree::{BabyStepId, ConceptId, PhaseId, StepId};

fn sample_block_verdict(mode: AuditorMode, tier: TierNodeId) -> LevelAuditVerdict {
    LevelAuditVerdict {
        mode,
        tier_node_id: tier,
        findings: vec![
            Finding {
                category: "C-LAZY-CODE".into(),
                severity: Severity::Block,
                message: "unwrap in production".into(),
                source: Some("foo.rs:42".into()),
            },
            Finding {
                category: "C-STALE-API".into(),
                severity: Severity::Warn,
                message: "uses log crate (should be tracing)".into(),
                source: None,
            },
        ],
        verdict: Verdict::Block,
        respawned_once: false,
        cascade_to_parent: false,
    }
}

#[test]
fn persist_writes_to_workflow_sessions_audits_mode_node_id_md() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let v = sample_block_verdict(
        AuditorMode::BabyStepDetail,
        TierNodeId::BabyStep(BabyStepId(101)),
    );
    let path = persist_verdict(tmp.path(), &v).expect("persist ok");
    let expected = tmp
        .path()
        .join(AUDITS_DIR_SEGMENT)
        .join("babystep_detail")
        .join("101.md");
    assert_eq!(path, expected, "audit artifact path layout");
    assert!(expected.exists());
}

#[test]
fn persist_directory_partitions_by_mode() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let modes = [
        (AuditorMode::Concepts, TierNodeId::Concept(ConceptId(1)), "concepts"),
        (AuditorMode::Phases, TierNodeId::Phase(PhaseId(2)), "phases"),
        (AuditorMode::Steps, TierNodeId::Step(StepId(3)), "steps"),
        (AuditorMode::BabySteps, TierNodeId::BabyStep(BabyStepId(4)), "babysteps"),
        (
            AuditorMode::BabyStepDetail,
            TierNodeId::BabyStep(BabyStepId(5)),
            "babystep_detail",
        ),
        (AuditorMode::Plan, TierNodeId::Concept(ConceptId(6)), "plan"),
    ];
    for (mode, tier, dir_label) in modes {
        let v = LevelAuditVerdict::allow(mode, tier);
        let path = persist_verdict(tmp.path(), &v).expect("persist ok");
        assert!(
            path.to_string_lossy().contains(&format!("audits\\{dir_label}\\"))
                || path.to_string_lossy().contains(&format!("audits/{dir_label}/")),
            "mode {dir_label} must have its own subdirectory, got {}",
            path.display()
        );
    }
}

#[test]
fn render_markdown_is_deterministic_byte_identical() {
    let v = sample_block_verdict(AuditorMode::Phases, TierNodeId::Phase(PhaseId(7)));
    let a = render_verdict_markdown(&v);
    let b = render_verdict_markdown(&v);
    assert_eq!(a, b, "two calls produce identical bytes");
    let c = render_verdict_markdown(&v);
    assert_eq!(a, c, "third call still identical");
}

#[test]
fn render_markdown_carries_all_verdict_metadata() {
    let v = sample_block_verdict(AuditorMode::Steps, TierNodeId::Step(StepId(42)));
    let md = render_verdict_markdown(&v);
    assert!(md.contains("mode: steps"));
    assert!(md.contains("tier: step (id=42)"));
    assert!(md.contains("verdict: block"));
    assert!(md.contains("respawned_once: false"));
    assert!(md.contains("cascade_to_parent: false"));
    assert!(md.contains("findings_count: 2"));
    assert!(md.contains("[BLOCK] C-LAZY-CODE"));
    assert!(md.contains("[WARN] C-STALE-API"));
    assert!(md.contains("source: foo.rs:42"));
}

#[test]
fn cascade_state_surfaces_in_markdown() {
    let mut v = sample_block_verdict(
        AuditorMode::BabyStepDetail,
        TierNodeId::BabyStep(BabyStepId(11)),
    );
    v.respawned_once = true;
    v.cascade_to_parent = true;
    let md = render_verdict_markdown(&v);
    assert!(md.contains("respawned_once: true"));
    assert!(md.contains("cascade_to_parent: true"));
}
