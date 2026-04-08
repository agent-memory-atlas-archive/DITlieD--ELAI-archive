//! BSP superstep determinism tests — TDD red phase (3I.5).
//!
//! Verifies: (a) BspCheckpoint serialization is deterministic given identical
//! inputs, (b) UntrackedValue channels are structurally excluded from
//! checkpoints, (c) EphemeralValue channels clear correctly at superstep
//! boundaries, (d) Topic entries are ordered by SSB sequence number,
//! (e) two-phase snapshot-then-clear boundary for accumulating vs
//! non-accumulating Topics.

use std::collections::BTreeMap;

use elai_orchestrator::bsp::{
    BspCheckpoint, ChannelId, ChannelRegistry, ChannelState, ChannelValue, EphemeralValue,
    InMemoryChannelRegistry, LastValue, NodeStatus, Topic, UntrackedValue,
};

const REPETITIONS: usize = 100;

#[test]
fn bsp_determinism() {
    let first_bytes = build_checkpoint_bytes();
    for i in 1..REPETITIONS {
        let bytes = build_checkpoint_bytes();
        assert_eq!(
            first_bytes, bytes,
            "checkpoint serialization diverged on iteration {i}"
        );
    }
}

fn build_checkpoint_bytes() -> Vec<u8> {
    let mut reg = InMemoryChannelRegistry::new();

    let lv: ChannelId<LastValue<u32>> = ChannelId::new("position");
    reg.write(&lv, 42u32, 1).expect("write LastValue");

    let topic: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("events", false);
    reg.write_topic(&topic, 10u32, 1).expect("write topic 1");
    reg.write_topic(&topic, 20u32, 2).expect("write topic 2");

    let eph: ChannelId<EphemeralValue<u32>> = ChannelId::new("signal");
    reg.write_ephemeral(&eph, 99u32, 1).expect("write ephemeral");

    let snap = reg.snapshot_for_scheduling();
    let channel_state: BTreeMap<String, ChannelState> = snap
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    let mut node_statuses = BTreeMap::new();
    node_statuses.insert("node_a".to_owned(), NodeStatus::Complete);
    node_statuses.insert("node_b".to_owned(), NodeStatus::Pending);

    let checkpoint = BspCheckpoint {
        superstep: 7,
        channel_state,
        node_statuses,
        created_at_secs: 1_700_000_000,
    };

    rmp_serde::to_vec_named(&checkpoint).expect("serialize checkpoint")
}

#[test]
fn untracked_never_checkpointed() {
    let mut reg = InMemoryChannelRegistry::new();

    let ut: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("cache");
    reg.write_untracked(&ut, vec![1, 2, 3])
        .expect("write untracked");

    let lv: ChannelId<LastValue<u32>> = ChannelId::new("visible");
    reg.write(&lv, 1u32, 1).expect("write LastValue");

    let snap = reg.snapshot_for_scheduling();

    assert!(
        snap.get("cache").is_none(),
        "UntrackedValue channel must not appear in snapshot"
    );
    assert!(
        snap.get("visible").is_some(),
        "LastValue channel must appear in snapshot"
    );

    // Structural check: ChannelState has exactly 3 variants (no UntrackedValue).
    // If this match compiles, the invariant holds structurally.
    let state = snap.get("visible").expect("visible channel in snapshot");
    match state {
        ChannelState::LastValue(_) | ChannelState::Topic(_) | ChannelState::Ephemeral(_) => {}
    }
}

#[test]
fn ephemeral_clears() {
    let mut reg = InMemoryChannelRegistry::new();
    let eph: ChannelId<EphemeralValue<u32>> = ChannelId::new("pulse");

    reg.write_ephemeral(&eph, 42u32, 1)
        .expect("write ephemeral");
    let val = reg.read_ephemeral(&eph).expect("read ephemeral");
    assert_eq!(val, ChannelValue::Present(42u32), "value must be present after write");

    // Simulate superstep boundary: reset write counts, then clear.
    reg.reset_ephemeral_write_counts();
    let cleared = reg.clear_ephemeral();
    assert!(
        cleared.contains(&"pulse".to_owned()),
        "pulse must be in cleared list"
    );
    let val_after = reg.read_ephemeral(&eph).expect("read after clear");
    assert_eq!(
        val_after,
        ChannelValue::Missing,
        "ephemeral must be Missing after clear with zero writes"
    );

    // Write again after clear — value survives until next clear cycle.
    reg.write_ephemeral(&eph, 99u32, 2)
        .expect("write ephemeral again");
    let val_rewrite = reg.read_ephemeral(&eph).expect("read after rewrite");
    assert_eq!(
        val_rewrite,
        ChannelValue::Present(99u32),
        "value must be present after second write"
    );

    // Second full clear cycle: reset write counts, clear again.
    reg.reset_ephemeral_write_counts();
    let cleared_2 = reg.clear_ephemeral();
    assert!(
        cleared_2.contains(&"pulse".to_owned()),
        "pulse must be cleared in second cycle"
    );
    let val_cycle2 = reg.read_ephemeral(&eph).expect("read after second clear");
    assert_eq!(
        val_cycle2,
        ChannelValue::Missing,
        "ephemeral must be Missing after second clear cycle"
    );
}

#[test]
fn topic_ordering() {
    let first_entries = build_topic_entries();

    // Assert ascending SSB seq order: seq=10 (100u32), seq=20 (200u32), seq=30 (300u32).
    assert_eq!(first_entries.len(), 3, "expected 3 topic entries");
    let first: u32 = rmp_serde::from_slice(&first_entries[0]).expect("deserialize entry[0]");
    let second: u32 = rmp_serde::from_slice(&first_entries[1]).expect("deserialize entry[1]");
    let third: u32 = rmp_serde::from_slice(&first_entries[2]).expect("deserialize entry[2]");
    assert_eq!(first, 100, "first entry must be value written at seq=10");
    assert_eq!(second, 200, "second entry must be value written at seq=20");
    assert_eq!(third, 300, "third entry must be value written at seq=30");

    // Consistency: identical output across REPETITIONS iterations.
    for i in 1..REPETITIONS {
        let entries = build_topic_entries();
        assert_eq!(
            first_entries, entries,
            "topic entry order diverged on iteration {i}"
        );
    }
}

fn build_topic_entries() -> Vec<Vec<u8>> {
    let mut reg = InMemoryChannelRegistry::new();
    let topic: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("log", false);

    // Write out-of-order SSB seq values.
    reg.write_topic(&topic, 300u32, 30).expect("write seq=30");
    reg.write_topic(&topic, 100u32, 10).expect("write seq=10");
    reg.write_topic(&topic, 200u32, 20).expect("write seq=20");

    let snap = reg.snapshot_for_scheduling();
    let state = snap.get("log").expect("topic must be in snapshot");

    match state {
        ChannelState::Topic(entries) => entries.clone(),
        other => panic!("expected Topic variant, got {other:?}"),
    }
}

#[test]
fn superstep_boundary_two_phase_reset() {
    let mut reg = InMemoryChannelRegistry::new();

    let non_accum: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("transient", false);
    let accum: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("persistent", true);

    reg.write_topic(&non_accum, 1u32, 1)
        .expect("write non-accum");
    reg.write_topic(&accum, 2u32, 1).expect("write accum");

    // Phase 1: snapshot (both present).
    let snap = reg.snapshot_for_scheduling();
    assert!(
        snap.get("transient").is_some(),
        "non-accum must be in snapshot before clear"
    );
    assert!(
        snap.get("persistent").is_some(),
        "accum must be in snapshot before clear"
    );

    // Phase 2: clear non-accumulating topics.
    reg.clear_non_accumulating_topics();

    let val_non = reg.read_topic::<u32>(&non_accum).expect("read non-accum after clear");
    assert_eq!(
        val_non,
        ChannelValue::Missing,
        "non-accumulating Topic must be cleared"
    );

    let val_acc = reg.read_topic::<u32>(&accum).expect("read accum after clear");
    assert!(
        matches!(val_acc, ChannelValue::Present(_)),
        "accumulating Topic must survive clear"
    );
}
