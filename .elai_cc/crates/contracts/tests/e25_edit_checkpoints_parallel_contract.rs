//! s-22600 Tranche G G.2 -- edit_checkpoints parallel-writer race
//! contract (audit WARN-2 followup).
//!
//! Pins:
//!   - Two concurrent `snapshot_pre_edit` calls against the same
//!     session both succeed AND land in distinct on-disk directories.
//!   - The `EditCheckpoint.nanos` tie-break is unique per writer at
//!     OS-clock resolution.
//!
//! Pre-G.2 the seq counter could lose monotonicity when two writers
//! both observed `max_seen = N` -- both would compute `seq = N+1` and
//! both create `{N+1:06}/...`, breaking rewind discoverability. The
//! `_{nanos:09}` suffix on the seq dir name eliminates that race.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use elai_orchestrator::architecture::edit_checkpoints::{snapshot_pre_edit, EditCheckpoint};

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fresh_test_root() -> PathBuf {
    let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    let mut root = std::env::temp_dir();
    root.push(format!(
        "elai-s22600-trancheG-G2-parallel-{}-{}-{}",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    ));
    fs::create_dir_all(&root).expect("create test root");
    root
}

#[test]
fn parallel_writers_land_in_distinct_directories() {
    let root = std::sync::Arc::new(fresh_test_root());
    let path = std::sync::Arc::new(root.join("file.txt"));
    let mut handles = Vec::new();
    for i in 0..8 {
        let p = std::sync::Arc::clone(&path);
        let r = std::sync::Arc::clone(&root);
        handles.push(std::thread::spawn(move || {
            let body = format!("parallel-v{i}");
            snapshot_pre_edit(&r, "s-parallel-contract", &p, body.as_bytes())
                .expect("parallel snapshot ok")
        }));
    }
    let snaps: Vec<EditCheckpoint> = handles
        .into_iter()
        .map(|h| h.join().expect("thread join"))
        .collect();
    // `(seq, nanos)` pair must be unique across all writers; this is
    // the invariant that guarantees rewind discoverability.
    let pairs: std::collections::HashSet<(u64, u64)> =
        snaps.iter().map(|c| (c.seq, c.nanos)).collect();
    assert_eq!(
        pairs.len(),
        snaps.len(),
        "duplicate (seq, nanos) pair across parallel writers: {snaps:?}"
    );
    // Each writer wrote a distinct on-disk file -- if two snapshots
    // collided on the same `{seq}/{prefix}/{encoded}` path one would
    // have silently overwritten the other.
    let on_disk: std::collections::HashSet<_> =
        snaps.iter().map(|c| c.snapshot_path.clone()).collect();
    assert_eq!(
        on_disk.len(),
        snaps.len(),
        "duplicate on-disk snapshot_path across parallel writers"
    );
    let _ = fs::remove_dir_all(&*root);
}
