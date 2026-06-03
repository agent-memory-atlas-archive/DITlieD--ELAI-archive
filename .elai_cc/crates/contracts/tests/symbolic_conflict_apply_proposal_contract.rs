//! F-8 / Slice A3 — symbolic-conflict compose-path contract.
//!
//! Validates the WARN-then-escalate compose unit
//! [`elai_orchestrator::screen_config_variant_for_conflicts`] — the reusable
//! screening step the self-mod promotion seam
//! (`external_benches::promote_after_bench_completed`, the F-10 darwinian-evolve
//! seam) and the `elai self-mod apply --proposal-fixture` CLI entrypoint both
//! call. The seam itself is `#[cfg(feature = "darwinian-evolve")]`; the
//! contracts crate links `elai-orchestrator` WITHOUT that feature, so this
//! contract exercises the exact public compose unit the seam delegates to (the
//! same `intent_from_config_variant` -> `detect_conflicts` -> emit path),
//! which is unconditional.
//!
//! POLICY under test: WARN-then-escalate, NEVER hard fail-closed.
//!   1. A contradictory locked-key `ConfigVariant` driven through the compose
//!      unit is FLAGGED (returns `true` = held pending) and emits at least one
//!      `SymbolicConflictDetected` SSB beacon (the held-pending record). The
//!      candidate is NOT applied and NOT hard-rejected — no enum widening, no
//!      `Blocked*` decision; the beacon IS the hold.
//!   2. A clean `ConfigVariant` proceeds: the compose unit returns `false`
//!      (clean) and emits ZERO beacons.
//!
//! Ssb is constructed INSIDE the tokio runtime (gotchas:110 — `Ssb::new`
//! spawns a write-loop; building it outside a reactor panics at runtime even
//! though cargo-green + pure-fn tests would hide it). The `#[tokio::test]`
//! attribute provides the reactor.

use std::sync::Arc;

use elai_core::clock::OsClock;
use elai_orchestrator::screen_config_variant_for_conflicts;
use elai_orchestrator::self_mod_layer1a::PipelineMode;
use elai_orchestrator::self_mod_types::{ConfigVariant, VariantStatus};
use elai_orchestrator::ssb::{Ssb, SsbActionType, SsbConfig, SsbReader};
use elai_orchestrator::tripwire::TripwireKind;
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn make_ssb(tmp: &TempDir, project: &str, session: &str) -> Arc<Ssb> {
    let cfg = SsbConfig {
        db_path: tmp.path().join("ssb.db"),
        pool_size: Some(2),
        project_id: project.into(),
        session_id: session.into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    Arc::new(Ssb::new(cfg, clock).expect("ssb new inside runtime"))
}

/// Build a `ConfigVariant` whose `config_blob` is a flat JSON object of the
/// given `(key, value)` pairs — the same shape `intent_from_config_variant`
/// decodes.
fn variant(variant_id: &str, pairs: &[(&str, serde_json::Value)]) -> ConfigVariant {
    let map: serde_json::Map<String, serde_json::Value> = pairs
        .iter()
        .map(|(k, v)| ((*k).to_string(), v.clone()))
        .collect();
    let config_blob =
        serde_json::to_vec(&serde_json::Value::Object(map)).expect("encode config_blob");
    ConfigVariant {
        variant_id: variant_id.to_string(),
        parent_id: None,
        score: 0.0,
        children_count: 0,
        config_blob,
        created_at: 0,
        pipeline_outcomes: Vec::new(),
        pipeline_mode: PipelineMode::Surgical,
        status: VariantStatus::Active,
    }
}

fn conflict_row_count(ssb: &Ssb) -> usize {
    ssb.query_by_type(SsbActionType::SymbolicConflictDetected)
        .expect("query SymbolicConflictDetected rows")
        .len()
}

// ---------------------------------------------------------------------------
// Contract 1: contradictory locked-key variant is held pending + emits beacon
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn contradictory_locked_key_variant_is_held_pending() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f8-a3", "scf-held").await;

    // `score_weights` is a real `self_mod_safety::LOCKED_KEYS` entry. A variant
    // that mutates it is the canonical Contradictory conflict.
    let cv = variant("cfg-contradictory", &[("score_weights", serde_json::json!("tampered"))]);

    let held = screen_config_variant_for_conflicts(&ssb, &cv, &[], "AdmittingVariant").await;

    assert!(
        held,
        "a contradictory locked-key variant MUST be flagged + held pending (WARN-then-escalate)"
    );

    let rows = ssb
        .query_by_type(SsbActionType::SymbolicConflictDetected)
        .expect("query SymbolicConflictDetected rows");
    assert!(
        !rows.is_empty(),
        "the held-pending record is the SymbolicConflictDetected beacon; at least one must exist"
    );

    // The beacon must carry the flagged candidate id (the held record is
    // correlatable back to the variant the operator must review).
    let decoded: elai_orchestrator::ssb::SsbPayload =
        rmp_serde::from_slice(&rows[0].payload).expect("decode beacon");
    match decoded {
        elai_orchestrator::ssb::SsbPayload::SymbolicConflictDetected(p) => {
            assert_eq!(
                p.candidate_id, "cfg-contradictory",
                "beacon candidate_id must correlate to the flagged variant"
            );
            assert_eq!(
                p.key, "score_weights",
                "beacon must name the contradicted locked key"
            );
            assert_eq!(
                p.fsm_substate, "AdmittingVariant",
                "beacon must record the FSM sub-state at flag time"
            );
        }
        other => panic!("expected SymbolicConflictDetected beacon, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Contract 2: a locked-key mutation during an active tripwire ALSO holds
// (the compose path screens against the live tripwire set, not just locks)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn locked_mutation_during_tripwire_is_held_with_causal_row() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f8-a3", "scf-causal").await;

    let cv = variant("cfg-causal", &[("auditor_config", serde_json::json!("x"))]);

    // One active tripwire ⇒ quiescence violated ⇒ the locked mutation raises a
    // CausalViolation in ADDITION to the Contradictory row.
    let held =
        screen_config_variant_for_conflicts(&ssb, &cv, &[TripwireKind::Ddm], "AdmittingVariant")
            .await;

    assert!(held, "locked mutation during an active tripwire must be held pending");

    let rows = ssb
        .query_by_type(SsbActionType::SymbolicConflictDetected)
        .expect("query rows");
    // At least two beacons: Contradictory + CausalViolation.
    assert!(
        rows.len() >= 2,
        "expected >=2 beacons (contradictory + causal_violation), got {}",
        rows.len()
    );
    let mut kinds: Vec<String> = rows
        .iter()
        .filter_map(|r| {
            match rmp_serde::from_slice::<elai_orchestrator::ssb::SsbPayload>(&r.payload) {
                Ok(elai_orchestrator::ssb::SsbPayload::SymbolicConflictDetected(p)) => Some(p.kind),
                _ => None,
            }
        })
        .collect();
    kinds.sort();
    assert!(
        kinds.iter().any(|k| k == "contradictory"),
        "a contradictory beacon must be present: {kinds:?}"
    );
    assert!(
        kinds.iter().any(|k| k == "causal_violation"),
        "a causal_violation beacon must be present when a tripwire is active: {kinds:?}"
    );
}

// ---------------------------------------------------------------------------
// Contract 3: a clean variant proceeds — no hold, no beacon
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn clean_variant_proceeds_with_no_beacon() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f8-a3", "scf-clean").await;

    // `freshness_half_life` is NOT a locked key — a clean variant.
    let cv = variant("cfg-clean", &[("freshness_half_life", serde_json::json!(7.0))]);

    let held = screen_config_variant_for_conflicts(&ssb, &cv, &[], "AdmittingVariant").await;

    assert!(!held, "a clean variant must proceed (not held)");
    assert_eq!(
        conflict_row_count(&ssb),
        0,
        "a clean variant must emit ZERO SymbolicConflictDetected beacons"
    );
}

// ---------------------------------------------------------------------------
// Contract 4: an empty-blob variant is a no-op (never conflicts, never holds)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn empty_variant_is_noop() {
    let tmp = TempDir::new().expect("tempdir");
    let ssb = make_ssb(&tmp, "f8-a3", "scf-empty").await;

    let cv = variant("cfg-empty", &[]);
    let held = screen_config_variant_for_conflicts(&ssb, &cv, &[], "AdmittingVariant").await;

    assert!(!held, "an empty-blob variant has no addressable keys; never held");
    assert_eq!(conflict_row_count(&ssb), 0, "empty variant emits no beacons");
}
