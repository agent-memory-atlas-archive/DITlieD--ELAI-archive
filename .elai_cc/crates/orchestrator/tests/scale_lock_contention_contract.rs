//! B-31.5 Tier C contract — cross-process lock contention.
//!
//! Reads `metrics/phase_b31_5_tier_c.toml` and asserts:
//!   * `two_instance_race.instance_b_exit_ms < 1000` — instance B must exit
//!     quickly when contention is detected (the IndexLock::try_acquire fast
//!     path observes the body PID + emits LockContested).
//!   * `stale_pid_reclaim.reclaim_wall_secs < 5` — reclamation of a stale
//!     lock body (dead PID) must be near-instant.

use std::path::PathBuf;

fn metric_path(name: &str) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set under cargo test");
    PathBuf::from(manifest_dir)
        .parent()
        .expect("orchestrator parent")
        .parent()
        .expect("workspace root")
        .join("metrics")
        .join(name)
}

#[test]
fn scale_tier_c_two_instance_race_fast_and_stale_reclaim_fast() {
    let path = metric_path("phase_b31_5_tier_c.toml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    let value: toml::Value = toml::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse {}: {}", path.display(), e));

    // two_instance_race section.
    let race = value
        .get("two_instance_race")
        .and_then(|v| v.as_table())
        .expect("[two_instance_race] missing or not a table");
    let exit_ms = race
        .get("instance_b_exit_ms")
        .and_then(|v| v.as_integer())
        .expect("two_instance_race.instance_b_exit_ms missing");
    assert!(
        exit_ms < 1000,
        "two_instance_race.instance_b_exit_ms must be < 1000ms (fast-fail on contention), got {exit_ms}"
    );
    let msg_match = race
        .get("instance_b_exit_message_match")
        .and_then(|v| v.as_bool())
        .expect("two_instance_race.instance_b_exit_message_match missing");
    assert!(
        msg_match,
        "two_instance_race.instance_b_exit_message_match must be true — IndexLock contention path must surface a human-readable message"
    );

    // stale_pid_reclaim section.
    let reclaim = value
        .get("stale_pid_reclaim")
        .and_then(|v| v.as_table())
        .expect("[stale_pid_reclaim] missing or not a table");
    let reclaim_wall = reclaim
        .get("reclaim_wall_secs")
        .and_then(|v| v.as_integer())
        .expect("stale_pid_reclaim.reclaim_wall_secs missing");
    assert!(
        reclaim_wall < 5,
        "stale_pid_reclaim.reclaim_wall_secs must be < 5s (reclaim is a single unlink + retry), got {reclaim_wall}"
    );
    let orphan = reclaim
        .get("orphan_detected")
        .and_then(|v| v.as_bool())
        .expect("stale_pid_reclaim.orphan_detected missing");
    assert!(orphan, "orphan_detected must be true after the stale-lock reclamation pass");
}
