//! F-12 contract (killer-risk 2 — FP / over-eager auto-close guard):
//!
//! No `soft_close` may fire below
//! `SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD`. `SurfaceToUser` is the
//! SAFE default for: below-threshold Contradiction, Neutral, Entailment,
//! and any judge `Err`. A judge failure NEVER auto-closes — it degrades
//! to the existing surface behaviour (fail-SAFE).

use elai_memory::semantic_contradiction::{
    adjudicate_semantic, collect_cross_key_candidates, NliJudge, NliLabel, NliVerdict,
    SemanticAction, SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
};
use elai_memory::{FactStore, FactType, MemoryError, MemoryFact, Provenance, Scope, TrustTier};

struct ScriptedJudge {
    result: Result<NliVerdict, ()>,
}

impl NliJudge for ScriptedJudge {
    fn classify(&self, _premise: &str, _hypothesis: &str) -> Result<NliVerdict, MemoryError> {
        match &self.result {
            Ok(v) => Ok(NliVerdict {
                label: v.label,
                confidence: v.confidence,
            }),
            Err(()) => Err(MemoryError::Invalid("scripted judge failure".into())),
        }
    }
}

fn fact(id: &str, key: &str, value: serde_json::Value, t: u128) -> MemoryFact {
    MemoryFact {
        fact_id: id.into(),
        user_id: "default".into(),
        project_id: Some("p1".into()),
        scope: Scope::Project,
        fact_type: FactType::Preference,
        key: key.into(),
        value,
        confidence: 0.9,
        trust_score: 0.9,
        ingested_at_ms: t,
        valid_from_ms: t,
        valid_to_ms: None,
        decay_rate_per_day: 0.001,
        last_observed_at_ms: t,
        evidence_pointers: vec!["sess".into(), "msg".into()],
        superseded_by: None,
        trust_tier: TrustTier::T1CodebaseDerived,
        provenance: Provenance::user_direct("test"),
    }
}

fn seed_pair(store: &mut FactStore) -> u128 {
    let t0 = 1_700_000_000_000u128;
    store
        .insert(&fact(
            "older",
            "prefers_db",
            serde_json::json!({"subject": "postgres"}),
            t0,
        ))
        .expect("ins older");
    store
        .insert(&fact(
            "newer",
            "migrated_off",
            serde_json::json!({"subject": "sqlite"}),
            t0 + 5_000,
        ))
        .expect("ins newer");
    t0
}

fn run(store: &mut FactStore, judge: &dyn NliJudge, t0: u128) -> Vec<SemanticAction> {
    let candidates =
        collect_cross_key_candidates(store, "default", FactType::Preference, Some(Scope::Project))
            .expect("collect");
    adjudicate_semantic(
        store,
        judge,
        &candidates,
        SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD,
        t0 + 10_000,
    )
    .expect("adjudicate")
    .into_iter()
    .map(|o| o.action)
    .collect()
}

fn assert_no_close(store: &FactStore) {
    assert_eq!(
        store.get("older").expect("get").expect("exists").valid_to_ms,
        None,
        "older row must NOT be soft-closed"
    );
    assert_eq!(
        store.get("newer").expect("get").expect("exists").valid_to_ms,
        None,
        "newer row must NOT be soft-closed"
    );
}

#[test]
fn below_threshold_contradiction_surfaces_not_closes() {
    let mut store = FactStore::open_in_memory().expect("open");
    let t0 = seed_pair(&mut store);
    // Contradiction label but confidence just under the threshold.
    let judge = ScriptedJudge {
        result: Ok(NliVerdict {
            label: NliLabel::Contradiction,
            confidence: SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD - 0.05,
        }),
    };
    let actions = run(&mut store, &judge, t0);
    assert!(
        actions
            .iter()
            .all(|a| matches!(a, SemanticAction::SurfaceToUser)),
        "below-threshold contradiction must SurfaceToUser, got {actions:?}"
    );
    assert_no_close(&store);
}

#[test]
fn entailment_never_auto_closes() {
    let mut store = FactStore::open_in_memory().expect("open");
    let t0 = seed_pair(&mut store);
    let judge = ScriptedJudge {
        result: Ok(NliVerdict {
            label: NliLabel::Entailment,
            confidence: 0.99,
        }),
    };
    let actions = run(&mut store, &judge, t0);
    assert!(
        actions
            .iter()
            .all(|a| matches!(a, SemanticAction::SurfaceToUser)),
        "entailment must never auto-close"
    );
    assert_no_close(&store);
}

#[test]
fn judge_error_degrades_to_surface_to_user() {
    let mut store = FactStore::open_in_memory().expect("open");
    let t0 = seed_pair(&mut store);
    // A judge that errors must NOT auto-close — fail-SAFE to SurfaceToUser.
    let judge = ScriptedJudge { result: Err(()) };
    let actions = run(&mut store, &judge, t0);
    assert!(
        actions
            .iter()
            .all(|a| matches!(a, SemanticAction::SurfaceToUser)),
        "a judge Err must degrade to SurfaceToUser, never auto-close"
    );
    assert_no_close(&store);
}

#[test]
fn threshold_constant_is_conservative() {
    // The auto-close threshold is conservative-by-default to bound the
    // false-positive hazard (killer-risk 2). Assert it is high.
    assert!(
        SEMANTIC_CONTRADICTION_CONFIDENCE_THRESHOLD >= 0.80,
        "auto-close threshold must be conservative (>= 0.80)"
    );
}
