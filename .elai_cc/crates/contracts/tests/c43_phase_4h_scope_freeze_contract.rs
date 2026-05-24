//! C-43 — Phase 4H scope-freeze ratification contract.
//!
//! Locks the wire surface for the 2 SSB variants
//! (`scope_freeze_ratified` + `scope_freeze_violated`), the
//! ScopeFreezeManifest TOML parse, and the verify_freeze semantics
//! (mtime > ratification_ts_ms ⇒ violation).
//!
//! Grounding posture: ELAI design hypothesis. The source plan cited
//! arXiv:2604.21725 (AEL — "Agent Evolving Learning for Open-Ended
//! Environments") as ground truth, but two independent WebFetches on
//! 2026-05-24 confirmed the paper is Thompson-Sampling agent learning,
//! NOT scope-freeze. The citation has been removed across every C-43
//! deliverable; see workflow/sessions/s-25120/mistakes.md for the bad-
//! cite analysis. This contract does NOT pin any paper-derived
//! invariant — only the ELAI-internal wire surface.

use std::fs;
use std::io::Write;

use elai_orchestrator::scope_freeze::{
    ratify_freeze, verify_freeze, ScopeFreezeManifest, MISSING_ITEM_SENTINEL_MS,
    SCOPE_FREEZE_FILENAME,
};
use elai_orchestrator::ssb::{
    ssb_action_type_from_str_public, ScopeFreezeRatifiedPayload, ScopeFreezeViolatedPayload,
};
use elai_orchestrator::{SsbActionType, SsbPayload, SCHEMA_VERSION_PUBLIC};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// (G1) ScopeFreezeManifest parses from a representative TOML fixture.
// ---------------------------------------------------------------------------

const SAMPLE_TOML: &str = r#"
ratification_ts_ms = 1748044800000
ratifier_id_hash = 4660
frozen_items = [
    "crates/orchestrator/src/self_mod_orchestrator.rs",
    "crates/orchestrator/src/msi.rs",
]
"#;

#[test]
fn c43_g1_manifest_parses_from_representative_toml() {
    let tmp = TempDir::new().expect("tempdir");
    fs::write(tmp.path().join(SCOPE_FREEZE_FILENAME), SAMPLE_TOML).expect("write manifest");
    let manifest = ScopeFreezeManifest::load(tmp.path()).expect("manifest parses");
    assert_eq!(manifest.ratification_ts_ms, 1_748_044_800_000);
    assert_eq!(manifest.ratifier_id_hash, 4_660);
    assert_eq!(manifest.frozen_items.len(), 2);
    assert!(manifest
        .frozen_items
        .iter()
        .any(|p| p.ends_with("self_mod_orchestrator.rs")));
}

// ---------------------------------------------------------------------------
// (G2) verify_freeze returns no violations for an unmodified frozen item.
// ---------------------------------------------------------------------------

#[test]
fn c43_g2_verify_clean_when_item_unchanged_since_future_ratification() {
    let tmp = TempDir::new().expect("tempdir");
    fs::write(tmp.path().join("frozen.rs"), b"// frozen").expect("write item");
    // Use a ratification timestamp far in the future so the item's
    // mtime is necessarily older than ratification_ts_ms.
    let future_ms: u64 = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_millis() as u64)
        + 600_000;
    let toml = format!(
        "ratification_ts_ms = {future_ms}\nratifier_id_hash = 1\nfrozen_items = [\"frozen.rs\"]\n"
    );
    fs::write(tmp.path().join(SCOPE_FREEZE_FILENAME), toml).expect("write manifest");
    let report = verify_freeze(tmp.path()).expect("verify");
    assert!(report.is_clean(), "expected clean, got {report:?}");
    assert_eq!(report.ratified_at, future_ms);
    assert_eq!(report.violations.len(), 0);
}

// ---------------------------------------------------------------------------
// (G3) verify_freeze reports a violation when an item's mtime is later
//      than ratification_ts_ms.
// ---------------------------------------------------------------------------

#[test]
fn c43_g3_verify_flags_item_modified_after_ratification() {
    let tmp = TempDir::new().expect("tempdir");
    let toml = "ratification_ts_ms = 1\nratifier_id_hash = 1\nfrozen_items = [\"frozen.rs\"]\n";
    fs::write(tmp.path().join(SCOPE_FREEZE_FILENAME), toml).expect("write manifest");
    // File written AFTER the ratification timestamp (t=1).
    let mut f = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(tmp.path().join("frozen.rs"))
        .expect("open");
    f.write_all(b"// mutated post-ratification").expect("write");
    drop(f);
    let report = verify_freeze(tmp.path()).expect("verify");
    assert_eq!(report.violations.len(), 1);
    let v = &report.violations[0];
    assert_eq!(v.item_path, "frozen.rs");
    assert!(v.modified_at_ms > 1);
    assert_ne!(v.modified_at_ms, MISSING_ITEM_SENTINEL_MS);
}

// ---------------------------------------------------------------------------
// (G4) ratify_freeze writes a manifest that round-trips through
//      ScopeFreezeManifest::load.
// ---------------------------------------------------------------------------

#[test]
fn c43_g4_ratify_round_trips_through_load() {
    let tmp = TempDir::new().expect("tempdir");
    let (manifest, hash) = ratify_freeze(
        tmp.path(),
        "maintainer",
        vec!["a.rs".into(), "b.rs".into(), "c.rs".into()],
    )
    .expect("ratify");
    assert_eq!(manifest.frozen_items.len(), 3);
    assert_ne!(hash, 0);
    let reloaded = ScopeFreezeManifest::load(tmp.path()).expect("reload");
    assert_eq!(reloaded.frozen_items, manifest.frozen_items);
    assert_eq!(reloaded.ratifier_id_hash, manifest.ratifier_id_hash);
    assert_eq!(reloaded.manifest_hash(), hash);
}

// ---------------------------------------------------------------------------
// (G5) The 2 SsbActionType variants exist, round-trip through
//      from_str + as_sql_str, and carry the expected payload shape.
// ---------------------------------------------------------------------------

#[test]
fn c43_g5_ssb_variants_round_trip() {
    // From-string parse + as_sql_str symmetry.
    let ratified = ssb_action_type_from_str_public("scope_freeze_ratified").expect("ratified");
    let violated = ssb_action_type_from_str_public("scope_freeze_violated").expect("violated");
    assert!(matches!(ratified, SsbActionType::ScopeFreezeRatified));
    assert!(matches!(violated, SsbActionType::ScopeFreezeViolated));
    assert_eq!(ratified.as_sql_str(), "scope_freeze_ratified");
    assert_eq!(violated.as_sql_str(), "scope_freeze_violated");

    // Payload discriminant mapping.
    let r_payload = SsbPayload::ScopeFreezeRatified(ScopeFreezeRatifiedPayload {
        manifest_hash: 0xCAFE_BABE_DEAD_BEEF,
        ratifier_id_hash: 7,
        item_count: 3,
        ts_ms: 1_748_044_800_000,
    });
    assert!(matches!(
        r_payload.action_type(),
        SsbActionType::ScopeFreezeRatified
    ));

    let v_payload = SsbPayload::ScopeFreezeViolated(ScopeFreezeViolatedPayload {
        item_path_hash: 42,
        frozen_since_ms: 1_748_044_800_000,
        modified_at_ms: 1_748_044_900_000,
        ts_ms: 1_748_044_900_000,
    });
    assert!(matches!(
        v_payload.action_type(),
        SsbActionType::ScopeFreezeViolated
    ));

    // SCHEMA_VERSION must reflect the C-43 closure bump.
    assert!(
        SCHEMA_VERSION_PUBLIC >= 116,
        "C-43 expects SCHEMA_VERSION >= 116, observed {SCHEMA_VERSION_PUBLIC}"
    );
}

// ---------------------------------------------------------------------------
// (G6) Bad-cite memo: this contract pins ZERO paper-derived invariants.
//      The source plan cited arXiv:2604.21725 as ground truth for the
//      scope-freeze mechanism; two independent WebFetches on 2026-05-24
//      confirmed the paper is Thompson-Sampling agent learning. See
//      workflow/sessions/s-25120/mistakes.md for the bad-cite analysis.
//      This test exists solely to make the absence of a paper-grounded
//      invariant explicit at audit time.
// ---------------------------------------------------------------------------

#[test]
fn c43_g6_no_paper_grounding_design_hypothesis() {
    // Intentional no-op assertion: the contract is documenting the
    // grounding posture, not a runtime invariant. If a future plan
    // re-introduces an arXiv: cite for scope-freeze, the
    // wiring-contract-validator + paper-ingestor must verify the cite
    // BEFORE that plan's audit can pass.
    let grounding_label = "ELAI design hypothesis — no paper grounding";
    assert_eq!(grounding_label, "ELAI design hypothesis — no paper grounding");
}
