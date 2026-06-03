//! F-5 slice A3 — `QuarantineBroadcastQueue` drop-oldest bounding contract.
//!
//! Pushing 300 records into a capacity-256 queue must keep the 256 NEWEST
//! records (drop-oldest), drop 44, and surface that drop count exactly once
//! on drain. This is the W-2 "no silent queue drop" guarantee: the opposite
//! of a tokio mpsc bounded channel (which drops-newest / blocks the sender).

use elai_memory::{
    QuarantineBroadcastQueue, QuarantineRecord, QUARANTINE_QUEUE_CAPACITY,
};

fn rec(n: u64) -> QuarantineRecord {
    QuarantineRecord {
        fact_id: format!("fact-{n:04}"),
        staleness_ratio: 1.5 + (n as f64) * 0.001,
        tick_ms: 1_700_000_000_000 + n,
    }
}

#[test]
fn capacity_constant_is_256() {
    assert_eq!(QUARANTINE_QUEUE_CAPACITY, 256);
}

#[test]
fn push_300_into_cap_256_drops_oldest_44() {
    let queue = QuarantineBroadcastQueue::new();
    let total: u64 = 300;
    for n in 0..total {
        queue.push(rec(n));
    }

    // 300 pushed, 256 capacity -> 44 evicted, 256 retained.
    assert_eq!(queue.len(), QUARANTINE_QUEUE_CAPACITY);
    assert_eq!(queue.dropped_count(), 44);

    let (drained, dropped) = queue.drain();
    assert_eq!(dropped, 44, "exactly 44 records dropped");
    assert_eq!(
        drained.len(),
        QUARANTINE_QUEUE_CAPACITY,
        "drain returns the full retained capacity"
    );

    // Drop-OLDEST: records 0..=43 evicted, 44..=299 survive, in FIFO order.
    let first = &drained[0];
    let last = &drained[drained.len() - 1];
    assert_eq!(
        first.fact_id, "fact-0044",
        "oldest surviving record is #44 (0..=43 were dropped-oldest)"
    );
    assert_eq!(
        last.fact_id, "fact-0299",
        "newest record always survives drop-oldest"
    );

    // Full survivor identity check: indices 44..300 in order.
    for (i, r) in drained.iter().enumerate() {
        let expected = (i as u64) + 44;
        assert_eq!(
            r.fact_id,
            format!("fact-{expected:04}"),
            "survivor at index {i} should be fact-{expected:04}"
        );
    }
}

#[test]
fn drain_resets_dropped_count() {
    let queue = QuarantineBroadcastQueue::new();
    for n in 0..(QUARANTINE_QUEUE_CAPACITY as u64 + 10) {
        queue.push(rec(n));
    }
    let (_first_batch, dropped_first) = queue.drain();
    assert_eq!(dropped_first, 10);

    // After drain, the drop counter is reset; a clean push round drops none.
    assert_eq!(queue.dropped_count(), 0);
    assert!(queue.is_empty());
    queue.push(rec(9000));
    let (second_batch, dropped_second) = queue.drain();
    assert_eq!(second_batch.len(), 1);
    assert_eq!(dropped_second, 0, "second drain reports only its own drops");
}

#[test]
fn under_capacity_drops_nothing() {
    let queue = QuarantineBroadcastQueue::new();
    for n in 0..100 {
        queue.push(rec(n));
    }
    assert_eq!(queue.len(), 100);
    assert_eq!(queue.dropped_count(), 0);
    let (drained, dropped) = queue.drain();
    assert_eq!(drained.len(), 100);
    assert_eq!(dropped, 0);
    assert!(queue.is_empty());
}
