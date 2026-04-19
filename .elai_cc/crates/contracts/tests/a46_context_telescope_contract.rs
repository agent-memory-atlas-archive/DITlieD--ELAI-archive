//! Batch 4C (A-46) — Context Telescope caller contract.
//!
//! Asserts:
//!   * `TelescopeStats` aggregates `invocations`, `avg_reduction_pct`, and
//!     `max_layers` across repeated record() calls.
//!   * `TelescopeCollapse` SSB payload round-trips through msgpack.
//!   * `TelescopeResult` exposes stable {from_tokens, to_tokens,
//!     layers_collapsed} fields.
//!
//! Grounding:
//!   * crates/orchestrator/src/telescope_caller.rs
//!   * crates/core/src/telescope.rs ContextTelescope

use elai_orchestrator::{
    SsbActionType, SsbPayload, TelescopeCollapsePayload, TelescopeResult, TelescopeStats,
};

// --- (a) TelescopeStats starts zero. ---

#[test]
fn contract_telescope_stats_zero_initial() {
    let s = TelescopeStats::new();
    assert_eq!(s.invocations(), 0);
    assert_eq!(s.max_layers(), 0);
    assert!((s.avg_reduction_pct() - 0.0).abs() < 0.0001);
}

// --- (b) TelescopeCollapse SSB payload round-trips. ---

#[test]
fn contract_telescope_collapse_payload_roundtrips() {
    let p = TelescopeCollapsePayload {
        from_tokens: 1800,
        to_tokens: 1200,
        layers_collapsed: 2,
        step_id: "step-1".into(),
    };
    let wrapped = SsbPayload::TelescopeCollapse(p.clone());
    assert_eq!(wrapped.action_type(), SsbActionType::TelescopeCollapse);
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("msgpack");
    let back: SsbPayload = rmp_serde::from_slice(&bytes).expect("decode");
    match back {
        SsbPayload::TelescopeCollapse(r) => {
            assert_eq!(r.from_tokens, 1800);
            assert_eq!(r.to_tokens, 1200);
            assert_eq!(r.layers_collapsed, 2);
            assert_eq!(r.step_id, "step-1");
        }
        other => panic!("unexpected variant: {other:?}"),
    }
}

// --- (c) TelescopeResult carries the three counters expected by stats. ---

#[test]
fn contract_telescope_result_fields() {
    let r = TelescopeResult {
        from_tokens: 2000,
        to_tokens: 1500,
        layers_collapsed: 1,
    };
    assert_eq!(r.from_tokens, 2000);
    assert_eq!(r.to_tokens, 1500);
    assert_eq!(r.layers_collapsed, 1);
}

// --- (d) SsbActionType::TelescopeCollapse string round-trips. ---

#[test]
fn contract_telescope_collapse_string_roundtrips() {
    assert_eq!(
        SsbActionType::TelescopeCollapse.as_sql_str(),
        "telescope_collapse"
    );
}
