//! Contract: ChannelRegistry trait exposes typed read/write methods plus
//! snapshot_for_scheduling, clear_ephemeral, force_clear_untracked,
//! deep_size_untracked. InMemoryChannelRegistry implements all methods.
//! ChannelState has exactly 3 variants (LastValue, Topic, Ephemeral).
//! UntrackedValue is never serialized to checkpoint.

use elai_orchestrator::bsp::{
    ChannelError, ChannelId, ChannelRegistry, ChannelSnapshot, ChannelState, ChannelValue,
    EphemeralValue, InMemoryChannelRegistry, LastValue, Topic, UntrackedValue,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn registry() -> InMemoryChannelRegistry {
    InMemoryChannelRegistry::new()
}

// ---------------------------------------------------------------------------
// Contract: ChannelRegistry trait surface
// ---------------------------------------------------------------------------

#[test]
fn contract_read_missing_returns_missing() {
    let reg = registry();
    let id: ChannelId<LastValue<String>> = ChannelId::new("absent");
    assert!(matches!(reg.read(&id).unwrap(), ChannelValue::Missing));
}

#[test]
fn contract_write_then_read_returns_present() {
    let mut reg = registry();
    let id: ChannelId<LastValue<i64>> = ChannelId::new("counter");
    reg.write(&id, 42i64, 1).unwrap();
    match reg.read(&id).unwrap() {
        ChannelValue::Present(v) => assert_eq!(v, 42),
        ChannelValue::Missing => panic!("expected Present"),
    }
}

#[test]
fn contract_snapshot_returns_frozen_view() {
    let mut reg = registry();
    let id: ChannelId<LastValue<u32>> = ChannelId::new("snap_test");
    reg.write(&id, 10u32, 1).unwrap();
    let snap: ChannelSnapshot = reg.snapshot_for_scheduling();
    assert!(snap.get("snap_test").is_some());
    reg.write(&id, 20u32, 2).unwrap();
    let state = snap.get("snap_test").unwrap();
    match state {
        ChannelState::LastValue(bytes) => {
            let v: u32 = rmp_serde::from_slice(bytes).unwrap();
            assert_eq!(v, 10);
        }
        _ => panic!("expected LastValue variant"),
    }
}

#[test]
fn contract_clear_ephemeral_returns_cleared_names() {
    let mut reg = registry();
    let id: ChannelId<EphemeralValue<String>> = ChannelId::new("signal");
    reg.write_ephemeral(&id, "ping".to_string(), 1).unwrap();
    reg.reset_ephemeral_write_counts();
    let cleared = reg.clear_ephemeral();
    assert_eq!(cleared.len(), 1);
    assert_eq!(cleared[0], "signal");
    assert!(matches!(reg.read_ephemeral(&id).unwrap(), ChannelValue::Missing));
}

#[test]
fn contract_force_clear_untracked_returns_bool() {
    let mut reg = registry();
    let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("cache");
    reg.write_untracked(&id, vec![1u8, 2, 3]).unwrap();
    let erased = ChannelId::<()>::new("cache");
    assert!(reg.force_clear_untracked(&erased));
    assert!(!reg.force_clear_untracked(&erased));
}

#[test]
fn contract_deep_size_untracked_returns_heap_size() {
    let mut reg = registry();
    let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("big");
    let payload = vec![0u8; 1_000_000];
    reg.write_untracked(&id, payload).unwrap();
    let erased = ChannelId::<()>::new("big");
    let size = reg.deep_size_untracked(&erased);
    assert!(size >= 1_000_000, "deep_size was {size}, expected >= 1_000_000");
}

// ---------------------------------------------------------------------------
// Contract: ChannelState serialization excludes UntrackedValue
// ---------------------------------------------------------------------------

#[test]
fn contract_channel_state_has_three_variants() {
    let lv = ChannelState::LastValue(vec![1, 2, 3]);
    let tp = ChannelState::Topic(vec![vec![4, 5]]);
    let ep = ChannelState::Ephemeral(Some(vec![6]));

    for state in [&lv, &tp, &ep] {
        let bytes = rmp_serde::to_vec_named(state).unwrap();
        let decoded: ChannelState = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(state, &decoded);
    }
}

#[test]
fn contract_untracked_value_excluded_from_snapshot() {
    let mut reg = registry();
    let lv_id: ChannelId<LastValue<u32>> = ChannelId::new("visible");
    let ut_id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("invisible");
    reg.write(&lv_id, 99u32, 1).unwrap();
    reg.write_untracked(&ut_id, vec![0u8; 100]).unwrap();

    let snap = reg.snapshot_for_scheduling();
    assert!(snap.get("visible").is_some());
    assert!(snap.get("invisible").is_none(), "UntrackedValue must not appear in snapshot");
}

// ---------------------------------------------------------------------------
// Adversarial: SSB seq ordering for LastValue
// ---------------------------------------------------------------------------

#[test]
fn adversarial_last_value_uses_ssb_seq_not_insertion_order() {
    let mut reg = registry();
    let id: ChannelId<LastValue<String>> = ChannelId::new("contested");
    reg.write(&id, "agent_a".to_string(), 5).unwrap();
    reg.write(&id, "agent_b".to_string(), 3).unwrap();
    match reg.read(&id).unwrap() {
        ChannelValue::Present(v) => assert_eq!(v, "agent_a"),
        ChannelValue::Missing => panic!("expected Present"),
    }
}

// ---------------------------------------------------------------------------
// Adversarial: Topic serialization determinism
// ---------------------------------------------------------------------------

#[test]
fn adversarial_topic_round_trip_identical() {
    let mut reg = registry();
    let id: ChannelId<Topic<String>> = ChannelId::new("log");
    reg.write_topic(&id, "first".to_string(), 1).unwrap();
    reg.write_topic(&id, "second".to_string(), 2).unwrap();
    reg.write_topic(&id, "third".to_string(), 3).unwrap();

    let snap1 = reg.snapshot_for_scheduling();
    let state1 = snap1.get("log").unwrap();
    let bytes1 = rmp_serde::to_vec_named(state1).unwrap();

    let decoded: ChannelState = rmp_serde::from_slice(&bytes1).unwrap();
    let bytes2 = rmp_serde::to_vec_named(&decoded).unwrap();
    assert_eq!(bytes1, bytes2, "Topic serialization must be deterministic");
}

// ---------------------------------------------------------------------------
// Adversarial: EphemeralValue cleared when write_count == 0
// ---------------------------------------------------------------------------

#[test]
fn adversarial_ephemeral_cleared_when_no_writes_in_superstep() {
    let mut reg = registry();
    let id: ChannelId<EphemeralValue<String>> = ChannelId::new("heartbeat");
    reg.write_ephemeral(&id, "alive".to_string(), 1).unwrap();
    match reg.read_ephemeral(&id).unwrap() {
        ChannelValue::Present(v) => assert_eq!(v, "alive"),
        ChannelValue::Missing => panic!("expected Present after write"),
    }
    reg.reset_ephemeral_write_counts();
    let cleared = reg.clear_ephemeral();
    assert_eq!(cleared.len(), 1);
    assert!(matches!(reg.read_ephemeral(&id).unwrap(), ChannelValue::Missing));
}

// ---------------------------------------------------------------------------
// Adversarial: DeepSizeOf returns heap size, not stack size
// ---------------------------------------------------------------------------

#[test]
fn adversarial_deep_size_measures_heap_not_stack() {
    let mut reg = registry();
    let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("heap_test");
    let payload = vec![0u8; 15_000_000];
    reg.write_untracked(&id, payload).unwrap();
    let erased = ChannelId::<()>::new("heap_test");
    let size = reg.deep_size_untracked(&erased);
    assert!(
        size > 15_000_000,
        "deep_size returned {size}, expected > 15_000_000 (not stack-only 24 bytes)"
    );
}

// ---------------------------------------------------------------------------
// Contract: ChannelError variants
// ---------------------------------------------------------------------------

#[test]
fn contract_channel_error_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ChannelError>();
}

// ---------------------------------------------------------------------------
// Contract: ChannelId type safety
// ---------------------------------------------------------------------------

#[test]
fn contract_channel_id_is_clone_eq_hash_debug() {
    let id1: ChannelId<LastValue<u32>> = ChannelId::new("test");
    let id2 = id1.clone();
    assert_eq!(id1, id2);
    let mut map = std::collections::HashMap::new();
    map.insert(id1, 42);
    assert_eq!(map[&id2], 42);
    let dbg = format!("{:?}", id2);
    assert!(dbg.contains("test"));
}

// ---------------------------------------------------------------------------
// Contract: Topic accumulate modes
// ---------------------------------------------------------------------------

#[test]
fn contract_topic_accumulate_true_preserves_across_supersteps() {
    let mut reg = registry();
    let id: ChannelId<Topic<String>> = ChannelId::new_with_accumulate("log", true);
    reg.write_topic(&id, "step1".to_string(), 1).unwrap();
    reg.clear_non_accumulating_topics();
    reg.write_topic(&id, "step2".to_string(), 2).unwrap();
    let snap = reg.snapshot_for_scheduling();
    match snap.get("log").unwrap() {
        ChannelState::Topic(entries) => assert_eq!(entries.len(), 2),
        _ => panic!("expected Topic variant"),
    }
}

#[test]
fn contract_topic_accumulate_false_clears_at_boundary() {
    let mut reg = registry();
    let id: ChannelId<Topic<String>> = ChannelId::new_with_accumulate("events", false);
    reg.write_topic(&id, "old_event".to_string(), 1).unwrap();
    reg.clear_non_accumulating_topics();
    reg.write_topic(&id, "new_event".to_string(), 2).unwrap();
    let snap = reg.snapshot_for_scheduling();
    match snap.get("events").unwrap() {
        ChannelState::Topic(entries) => {
            assert_eq!(entries.len(), 1);
            let v: String = rmp_serde::from_slice(&entries[0]).unwrap();
            assert_eq!(v, "new_event");
        }
        _ => panic!("expected Topic variant"),
    }
}

// ---------------------------------------------------------------------------
// Contract: Topic accumulate conflict detection
// ---------------------------------------------------------------------------

#[test]
fn contract_topic_accumulate_conflict_returns_error() {
    let mut reg = registry();
    let id_true: ChannelId<Topic<String>> = ChannelId::new_with_accumulate("mixed", true);
    let id_false: ChannelId<Topic<String>> = ChannelId::new_with_accumulate("mixed", false);
    reg.write_topic(&id_true, "first".to_string(), 1).unwrap();
    let err = reg.write_topic(&id_false, "second".to_string(), 2).unwrap_err();
    assert!(matches!(err, ChannelError::AccumulateConflict { .. }));
}
