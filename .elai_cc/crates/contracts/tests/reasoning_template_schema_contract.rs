// D-1 contract: ReasoningTemplate typed schema + Blake3 content-hash +
// archive lifecycle.
//
// Verifies:
// 1. SCHEMA_VERSION_PUBLIC >= 64 (D-1 atomic bump).
// 2. All 4 new SsbActionType variants round-trip via as_sql_str /
//    ssb_action_type_from_str_public:
//      - ArchiveEntryAdded
//      - ArchiveEntryRetired
//      - TemplatePromoted
//      - TemplateCandidateRejected
// 3. Each payload struct serializes via msgpack (rmp_serde) AND JSON
//    (serde_json) without loss — round-trip equality.
// 4. SQL strings are snake_case and match the wire labels.
// 5. ReasoningTemplate closed-enum variant counts (Domain, PatternKind,
//    VerifierKind, ArchiveStatus, RetirementReason) match the spec-declared
//    constants. Adding a variant without bumping the const breaks
//    compilation (see _domain_check etc. in reasoning_template.rs).
// 6. ArchiveChain.append + verify_chain end-to-end on tmpdir.

use elai_orchestrator::archive_chain::{ArchiveChain, ArchiveEntry};
use elai_orchestrator::reasoning_template::{
    compute_content_hash, ArchiveStatus, Domain, PatternKind, Predicate, ReasoningTemplate,
    TemplateBody, TemplateId, VerifierKind, ARCHIVE_STATUS_VARIANT_COUNT, DOMAIN_VARIANT_COUNT,
    PATTERN_KIND_VARIANT_COUNT, RETIREMENT_REASON_VARIANT_COUNT, VERIFIER_KIND_VARIANT_COUNT,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ArchiveEntryAddedPayload, ArchiveEntryRetiredPayload,
    SsbActionType, SsbPayload, TemplateCandidateRejectedPayload, TemplatePromotedPayload,
    SCHEMA_VERSION_PUBLIC,
};
use tempfile::TempDir;

/// Floor for D-1: schema must include the 4 new variants + bump to 64.
const D1_SCHEMA_FLOOR: i64 = 64;

#[test]
fn schema_version_includes_d1_floor() {
    assert!(
        SCHEMA_VERSION_PUBLIC >= D1_SCHEMA_FLOOR,
        "schema must include D-1 SSB variants (got {SCHEMA_VERSION_PUBLIC}, floor {D1_SCHEMA_FLOOR})"
    );
}

fn roundtrip_action_type(at: SsbActionType, expected_sql: &str) {
    let s = at.as_sql_str();
    assert_eq!(
        s, expected_sql,
        "as_sql_str mismatch for {at:?}: got {s}, expected {expected_sql}"
    );
    let decoded = ssb_action_type_from_str_public(s)
        .unwrap_or_else(|e| panic!("ssb_action_type_from_str_public({s:?}) failed: {e:?}"));
    assert_eq!(decoded, at, "round-trip failed for {s}");
}

#[test]
fn archive_entry_added_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::ArchiveEntryAdded, "archive_entry_added");
}

#[test]
fn archive_entry_retired_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::ArchiveEntryRetired, "archive_entry_retired");
}

#[test]
fn template_promoted_action_type_roundtrip() {
    roundtrip_action_type(SsbActionType::TemplatePromoted, "template_promoted");
}

#[test]
fn template_candidate_rejected_action_type_roundtrip() {
    roundtrip_action_type(
        SsbActionType::TemplateCandidateRejected,
        "template_candidate_rejected",
    );
}

#[test]
fn archive_entry_added_payload_roundtrip() {
    let p = ArchiveEntryAddedPayload {
        template_id: "11111111111111111111111111111111".to_string(),
        content_hash: "aa".repeat(32),
        parent_hash: Some("bb".repeat(32)),
        domain: "code".to_string(),
        pattern_kind: "decomposition".to_string(),
        archive_status: "frozen".to_string(),
    };
    let payload = SsbPayload::ArchiveEntryAdded(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::ArchiveEntryAdded);

    // msgpack round-trip.
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack ser");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack de");
    assert_eq!(decoded, payload);

    // JSON round-trip.
    let json = serde_json::to_string(&payload).expect("json ser");
    let decoded: SsbPayload = serde_json::from_str(&json).expect("json de");
    assert_eq!(decoded, payload);
}

#[test]
fn archive_entry_retired_payload_roundtrip() {
    let p = ArchiveEntryRetiredPayload {
        template_id: "22222222222222222222222222222222".to_string(),
        retired_at_ms: 1_700_000_000_000,
        retirement_reason: "superseded".to_string(),
    };
    let payload = SsbPayload::ArchiveEntryRetired(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::ArchiveEntryRetired);
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack ser");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack de");
    assert_eq!(decoded, payload);
}

#[test]
fn template_promoted_payload_roundtrip() {
    let p = TemplatePromotedPayload {
        template_id: "33333333333333333333333333333333".to_string(),
        stage: "stage_200".to_string(),
        pass_rate: 0.987,
    };
    let payload = SsbPayload::TemplatePromoted(p.clone());
    assert_eq!(payload.action_type(), SsbActionType::TemplatePromoted);
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack ser");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack de");
    assert_eq!(decoded, payload);
}

#[test]
fn template_candidate_rejected_payload_roundtrip() {
    let p = TemplateCandidateRejectedPayload {
        template_id: "44444444444444444444444444444444".to_string(),
        rejection_reason: "below_floor".to_string(),
    };
    let payload = SsbPayload::TemplateCandidateRejected(p.clone());
    assert_eq!(
        payload.action_type(),
        SsbActionType::TemplateCandidateRejected
    );
    let bytes = rmp_serde::to_vec_named(&payload).expect("msgpack ser");
    let decoded: SsbPayload = rmp_serde::from_slice(&bytes).expect("msgpack de");
    assert_eq!(decoded, payload);
}

#[test]
fn closed_enum_variant_counts_match_spec_constants() {
    // Spec: PatternKind / VerifierKind / ArchiveStatus / RetirementReason
    // each have exactly 4 variants and are L3-frozen per
    // docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md §4.
    // Domain was extended atomic-with-D-7 (additive) to 6 variants — the
    // four original variants stay stable; Planning + ToolUse joined to
    // support the cross-domain transfer canary's 5 orthogonal task
    // families. See workflow/reasoning/D-7-decisions.md.
    assert_eq!(DOMAIN_VARIANT_COUNT, 6);
    assert_eq!(PATTERN_KIND_VARIANT_COUNT, 4);
    assert_eq!(VERIFIER_KIND_VARIANT_COUNT, 4);
    assert_eq!(ARCHIVE_STATUS_VARIANT_COUNT, 4);
    assert_eq!(RETIREMENT_REASON_VARIANT_COUNT, 4);
}

#[test]
fn archive_chain_append_then_verify_chain_end_to_end() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("contract_archive.jsonl");
    let mut chain = ArchiveChain::new(path.clone());

    let body = TemplateBody::try_new("contract test body").expect("body fits");
    let content_hash = compute_content_hash(body.as_str(), None);
    let template = ReasoningTemplate {
        id: TemplateId::new_v4(),
        domain: Domain::Math,
        pattern_kind: PatternKind::Verification,
        template_body: body,
        eligibility_predicate: Predicate::Always,
        verifier_required: VerifierKind::Prm,
        content_hash,
        created_at_ms: 0,
        parent_id: None,
        archive_status: ArchiveStatus::Frozen,
    };
    let entry = ArchiveEntry::new_root(template.clone());
    chain.append(entry).expect("append ok");

    // verify_chain on a clean append returns Ok.
    chain.verify_chain().expect("verify_chain ok");

    // Reload from disk + verify again (round-trip).
    let reloaded = ArchiveChain::load(path).expect("reload");
    assert_eq!(reloaded.len(), 1);
    reloaded.verify_chain().expect("verify reloaded");

    // build_registry yields the Frozen entry.
    let registry = reloaded.build_registry();
    assert_eq!(registry.len(), 1);
    assert!(registry.get(&template.id).is_some());
}

#[test]
fn status_transition_forbids_frozen_outbound() {
    assert!(!ArchiveStatus::Frozen.can_transition_to(ArchiveStatus::Promoted));
    assert!(!ArchiveStatus::Frozen.can_transition_to(ArchiveStatus::Retired));
    assert!(!ArchiveStatus::Frozen.can_transition_to(ArchiveStatus::Candidate));
}

#[test]
fn status_transition_forbids_retired_outbound() {
    assert!(!ArchiveStatus::Retired.can_transition_to(ArchiveStatus::Promoted));
    assert!(!ArchiveStatus::Retired.can_transition_to(ArchiveStatus::Candidate));
    assert!(!ArchiveStatus::Retired.can_transition_to(ArchiveStatus::Frozen));
}
