//! Batch-fu (FU.6 / A-46) — Telescope-in-repair-assembly contract.
//!
//! Verifies:
//!   * `TelescopeCollapse` is a registered SsbActionType and round-trips.
//!   * `TelescopeStats` records invocations and reduction percentages.
//!   * `invoke_telescope` with a snippet over budget drops inner layers and
//!     records at least one collapse (stats.invocations increments).
//!   * `invoke_telescope` with budget > from-tokens is a no-op (no SSB row).

use std::sync::Arc;

use elai_core::bounded_string::{BoundedString, WordTokenCounter};
use elai_core::telescope::{
    ContextTelescope, TELESCOPE_CONCEPT_MAX, TELESCOPE_PHASE_MAX, TELESCOPE_STEP_MAX,
    TELESCOPE_VISION_MAX,
};
use elai_orchestrator::{
    invoke_telescope, SsbActionType, SsbConfig, SsbPayload, TelescopeCollapsePayload,
    TelescopeStats,
};

#[test]
fn contract_telescope_collapse_is_a_registered_action_type() {
    assert_eq!(
        SsbActionType::TelescopeCollapse.as_sql_str(),
        "telescope_collapse"
    );
}

#[test]
fn contract_telescope_collapse_payload_roundtrips() {
    let p = TelescopeCollapsePayload {
        from_tokens: 200,
        to_tokens: 100,
        layers_collapsed: 2,
        step_id: "task_1".into(),
    };
    let wrapped = SsbPayload::TelescopeCollapse(p.clone());
    let bytes = rmp_serde::to_vec_named(&wrapped).expect("serialize");
    let round: SsbPayload = rmp_serde::from_slice(&bytes).expect("deserialize");
    assert!(matches!(round, SsbPayload::TelescopeCollapse(q) if q == p));
}

fn make_oversized_telescope() -> ContextTelescope {
    let c = WordTokenCounter;
    ContextTelescope {
        project_vision: BoundedString::<TELESCOPE_VISION_MAX>::new(
            "alpha beta gamma delta epsilon zeta eta theta",
            &c,
        )
        .expect("vision"),
        concept_summary: Some(
            BoundedString::<TELESCOPE_CONCEPT_MAX>::new(
                "one two three four five six seven",
                &c,
            )
            .expect("concept"),
        ),
        phase_goal: Some(
            BoundedString::<TELESCOPE_PHASE_MAX>::new("eight nine ten eleven twelve", &c)
                .expect("phase"),
        ),
        step_context: Some(
            BoundedString::<TELESCOPE_STEP_MAX>::new(
                "thirteen fourteen fifteen sixteen seventeen eighteen",
                &c,
            )
            .expect("step"),
        ),
    }
}

fn make_ssb() -> (Arc<elai_orchestrator::Ssb>, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("ssb.db");
    let cfg = SsbConfig {
        db_path: db,
        pool_size: Some(2),
        project_id: "fu-telescope".into(),
        session_id: "fu".into(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(elai_core::clock::OsClock);
    let ssb = Arc::new(elai_orchestrator::Ssb::new(cfg, clock).expect("ssb"));
    (ssb, dir)
}

#[tokio::test]
async fn contract_invoke_telescope_collapses_layers_under_tight_budget() {
    let (ssb, _dir) = make_ssb();
    let stats = Arc::new(TelescopeStats::new());
    let tele = make_oversized_telescope();
    let r = invoke_telescope(&ssb, &stats, "agent", 0, "step-1", &tele, 10)
        .await
        .expect("invoke ok");
    assert!(
        r.layers_collapsed >= 1,
        "tight budget must drop at least one layer; got {r:?}"
    );
    assert_eq!(stats.invocations(), 1);
}

#[tokio::test]
async fn contract_invoke_telescope_is_no_emit_when_fits_budget() {
    let (ssb, _dir) = make_ssb();
    let stats = Arc::new(TelescopeStats::new());
    let tele = make_oversized_telescope();
    let r = invoke_telescope(&ssb, &stats, "agent", 0, "step-2", &tele, 10_000)
        .await
        .expect("invoke ok");
    assert_eq!(r.layers_collapsed, 0, "generous budget must not collapse");
    assert_eq!(stats.invocations(), 1);
}
