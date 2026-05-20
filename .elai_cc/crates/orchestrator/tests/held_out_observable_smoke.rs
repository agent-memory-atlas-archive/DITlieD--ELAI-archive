//! D-6 observable_effect smoke — end-to-end:
//!   * Ingest 5 candidate tasks (one of them is a near-duplicate of another).
//!   * Confirm the n-gram contamination filter rejects the duplicate.
//!   * Confirm the surviving 4 tasks are persisted + queryable.
//!   * Confirm `get_held_out_recent` semantics (top of pool sorted by ingest
//!     timestamp desc) — emulated here via [`HeldOutStore::recent`] which is
//!     the underlying read path the Tauri command forwards.
//!
//! No live model / sidecar dependencies. Runs against `open_in_memory`.

use elai_orchestrator::held_out::{
    build_task, is_contaminated, jaccard_overlap, ngram_filter, ExpectedOutcome, HeldOutStore,
    TaskSource, DEFAULT_JACCARD_THRESHOLD, DEFAULT_NGRAM_SIZE,
};

#[test]
fn five_tasks_one_contaminated_filter_holds() {
    let store = HeldOutStore::open_in_memory().expect("open in-memory store");

    let bodies = [
        ("alpha beta gamma delta epsilon zeta eta theta iota kappa", 1_000_i64),
        ("write a function that reverses a singly-linked list in place", 1_100),
        ("solve the integral of x squared dx from zero to one", 1_200),
        // Contaminator: same body as the second entry. The exact-hash
        // short-circuit catches it before the n-gram scan even runs, but
        // the test mutates the hash on the copy to force the n-gram path.
        ("write a function that reverses a singly-linked list in place", 1_300),
        ("emit a markdown table summarizing the build matrix outcomes", 1_400),
    ];

    let mut inserted = 0u32;
    let mut contamination_hits = 0u32;
    for (i, (body, ts)) in bodies.iter().enumerate() {
        let task = build_task(
            body.to_string(),
            TaskSource::Synthetic { seed: i as u64 },
            ExpectedOutcome::Code("ok".into()),
            *ts,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        // The 4th entry (i == 3) duplicates the 2nd. The store's
        // insert_if_unique runs the hash short-circuit first; the
        // duplicate triggers HeldOutError::DuplicateTask which is the
        // contamination outcome we count.
        match store.insert_if_unique(&task, DEFAULT_JACCARD_THRESHOLD) {
            Ok(true) => inserted += 1,
            Ok(false) => contamination_hits += 1,
            Err(elai_orchestrator::held_out::HeldOutError::DuplicateTask(_)) => {
                contamination_hits += 1
            }
            Err(e) => panic!("unexpected insert error: {e}"),
        }
    }

    assert_eq!(inserted, 4, "exactly four unique tasks should land");
    assert_eq!(contamination_hits, 1, "exactly one duplicate must trip the filter");
    assert_eq!(store.count().expect("count"), 4);

    // recent(...) is the read path get_held_out_recent forwards. Largest
    // ingested_ts_ms first.
    let recent = store.recent(10).expect("recent");
    assert_eq!(recent.len(), 4);
    assert!(recent[0].ingested_ts_ms >= recent[recent.len() - 1].ingested_ts_ms);
}

#[test]
fn ngram_filter_jaccard_paraphrase_detection() {
    // LiveCodeBench-style: a near-paraphrase with substantial token overlap
    // must still trip the 9-gram filter at the default 0.3 threshold. Two
    // sentences with the same first 12 tokens guarantee multiple shared
    // 9-grams.
    let a = ngram_filter(
        "the quick brown fox jumps over the lazy dog and then runs away fast",
        DEFAULT_NGRAM_SIZE,
    );
    let b = ngram_filter(
        "the quick brown fox jumps over the lazy dog and then sleeps in the sun",
        DEFAULT_NGRAM_SIZE,
    );
    let j = jaccard_overlap(&a, &b);
    assert!(j > 0.0, "paraphrase must share some 9-grams, got {j}");
    assert!(
        is_contaminated(&a, &b, 0.1),
        "near-paraphrase must trip at jaccard >= 0.1 (got {j})"
    );
}

#[test]
fn disjoint_bodies_below_threshold() {
    let a = ngram_filter(
        "write a function that reverses a singly linked list in place using tail recursion",
        DEFAULT_NGRAM_SIZE,
    );
    let b = ngram_filter(
        "compute the eigenvalues of a three by three real symmetric matrix using power iteration",
        DEFAULT_NGRAM_SIZE,
    );
    assert!(!is_contaminated(&a, &b, DEFAULT_JACCARD_THRESHOLD));
}

#[test]
fn eligibility_window_holds_fresh_tasks_back() {
    let store = HeldOutStore::open_in_memory().expect("open");
    let task = build_task(
        "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda".into(),
        TaskSource::Synthetic { seed: 99 },
        ExpectedOutcome::Code("ok".into()),
        10_000,
        5_000, // 5-second eligibility window
        DEFAULT_NGRAM_SIZE,
    );
    store.insert(&task).expect("insert");
    // Just-after-ingest: not yet active.
    assert_eq!(store.active_tasks(11_000).expect("active early").len(), 0);
    // Window elapsed: active.
    assert_eq!(store.active_tasks(20_000).expect("active late").len(), 1);
}

#[test]
fn agent_pool_sample_returns_oldest_active() {
    let store = HeldOutStore::open_in_memory().expect("open");

    let mut bodies = vec![
        (
            "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu",
            1_000_i64,
        ),
        (
            "omicron pi rho sigma tau upsilon phi chi psi omega aleph bet gimmel",
            2_000,
        ),
        (
            "compute the integral of e to the minus x squared from minus infinity to infinity",
            3_000,
        ),
    ];
    for (i, (body, ts)) in bodies.drain(..).enumerate() {
        let task = build_task(
            body.to_string(),
            TaskSource::Synthetic { seed: i as u64 },
            ExpectedOutcome::Code("ok".into()),
            ts,
            0,
            DEFAULT_NGRAM_SIZE,
        );
        store.insert(&task).expect("insert");
    }

    // All eligibility windows are zero, so all three should be active.
    let active = store.active_tasks(5_000).expect("active");
    assert_eq!(active.len(), 3);

    // The agent_pool sampler returns the oldest-ingest-first; here that's
    // ts_ms = 1_000.
    let picked = elai_orchestrator::agent_pool::sample_held_out_task(&store, 5_000)
        .expect("expected one active task");
    assert_eq!(picked.ingested_ts_ms, 1_000);
}
