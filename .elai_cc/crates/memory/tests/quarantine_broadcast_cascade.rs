//! F-5 slice A3 — `collect_quarantined` mock-clock cascade contract.
//!
//! A fact aged past `STALENESS_QUARANTINE_THRESHOLD` (1.5 half-lives) appears
//! in `collect_quarantined`; a freshly-observed fact does not. Uses a virtual
//! clock (no wall-clock dependence) so the test is deterministic.

use elai_memory::fact::{FactType, MemoryFact, Scope};
use elai_memory::store::FactStore;
use elai_memory::trust::{Provenance, TrustTier};
use elai_memory::{collect_quarantined, STALENESS_QUARANTINE_THRESHOLD};

const MS_PER_DAY: u128 = 86_400_000;
/// Fixed virtual epoch (2023-11-14T22:13:20Z) — any stable base works.
const BASE_MS: u128 = 1_700_000_000_000;

fn make_fact(
    fact_id: &str,
    decay_rate_per_day: f32,
    last_observed_at_ms: u128,
) -> MemoryFact {
    MemoryFact {
        fact_id: fact_id.to_string(),
        user_id: "default".into(),
        project_id: Some("p1".into()),
        scope: Scope::Project,
        fact_type: FactType::Mistake,
        key: "k".into(),
        value: serde_json::json!({ "v": 1 }),
        confidence: 0.9,
        trust_score: 0.9,
        ingested_at_ms: last_observed_at_ms,
        valid_from_ms: last_observed_at_ms,
        valid_to_ms: None,
        decay_rate_per_day,
        last_observed_at_ms,
        evidence_pointers: vec!["s".into(), "m".into()],
        superseded_by: None,
        trust_tier: TrustTier::T1CodebaseDerived,
        provenance: Provenance::codebase_derived("test"),
        last_validated_at_ms: None,
    }
}

#[test]
fn aged_fact_past_threshold_is_quarantined() {
    let store = FactStore::open_in_memory().expect("open in-memory store");

    // Mistake decay_rate = 0.005/day -> half_life = ln(2)/0.005 ~ 138.6 days.
    // Threshold 1.5 half-lives ~ 207.9 days. A fact last observed 250 days
    // ago is safely past it; a fact observed "now" is not.
    let rate = 0.005_f32;
    let now = BASE_MS + 250 * MS_PER_DAY;

    let stale = make_fact("stale-1", rate, BASE_MS); // 250 days old
    let fresh = make_fact("fresh-1", rate, now); // observed at `now`
    store.insert(&stale).expect("insert stale fact");
    store.insert(&fresh).expect("insert fresh fact");

    let quarantined = collect_quarantined(&store, now as u64)
        .expect("collect_quarantined succeeds");

    assert_eq!(
        quarantined.len(),
        1,
        "exactly the aged fact should be quarantined, got {quarantined:?}"
    );
    let rec = &quarantined[0];
    assert_eq!(rec.fact_id, "stale-1");
    assert_eq!(rec.tick_ms, now as u64, "tick_ms records the broadcast time");
    assert!(
        rec.staleness_ratio >= STALENESS_QUARANTINE_THRESHOLD,
        "ratio {} must be >= threshold {}",
        rec.staleness_ratio,
        STALENESS_QUARANTINE_THRESHOLD
    );
    // 250 days / 138.6-day half-life ~ 1.80 half-lives.
    assert!(
        (rec.staleness_ratio - 1.80).abs() < 0.05,
        "expected ~1.80 half-lives, got {}",
        rec.staleness_ratio
    );
}

#[test]
fn immortal_fact_never_quarantines() {
    let store = FactStore::open_in_memory().expect("open in-memory store");
    // decay_rate = 0 -> infinite half-life -> ratio 0, never crosses.
    let immortal = make_fact("immortal-1", 0.0, BASE_MS);
    store.insert(&immortal).expect("insert immortal fact");

    // Even an absurd age (10 years) must not quarantine a zero-rate fact.
    let now = BASE_MS + 3650 * MS_PER_DAY;
    let quarantined = collect_quarantined(&store, now as u64)
        .expect("collect_quarantined succeeds");
    assert!(
        quarantined.is_empty(),
        "zero-decay-rate fact must never be quarantined, got {quarantined:?}"
    );
}

#[test]
fn future_dated_fact_is_not_quarantined() {
    let store = FactStore::open_in_memory().expect("open in-memory store");
    // last_observed is in the FUTURE relative to `now` -> ratio 0.
    let future = make_fact("future-1", 0.005, BASE_MS + 10 * MS_PER_DAY);
    store.insert(&future).expect("insert future-dated fact");

    let quarantined = collect_quarantined(&store, BASE_MS as u64)
        .expect("collect_quarantined succeeds");
    assert!(
        quarantined.is_empty(),
        "future-dated fact must not be quarantined, got {quarantined:?}"
    );
}
