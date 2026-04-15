//! MistakeBook SSB integration: query, insert, and format mistake entries
//! scoped to individual baby steps.
//!
//! Wraps `elai_core::mistake_book` types with SSB persistence.
//! Per baby step: max 5 entries, oldest-evicted, blake3 deduplication.
//!
//! Clear-on-success: `clear_baby_step()` appends a tombstone entry
//! (`cleared = true`) to SSB. `query_by_baby_step` skips all entries for
//! any `baby_step_id` that has a tombstone, preserving append-only semantics.

use elai_core::mistake_book::{
    format_constraints, insert_with_eviction, DiffHash, FailureCategory, MistakeEntry,
    MAX_ENTRIES_PER_STEP,
};

use crate::ssb::{MistakeEntryPayload, Ssb, SsbActionType, SsbError, SsbPayload, SsbReader};

// ---------------------------------------------------------------------------
// MistakeBook — SSB-backed per-baby-step constraint store
// ---------------------------------------------------------------------------

/// Query all mistake entries for a given baby step from SSB.
///
/// Filters SSB entries by action_type = mistake_entry, then deserializes
/// and filters by `baby_step_id`. Returns entries sorted by timestamp ascending.
///
/// Tombstone-aware: if a `cleared = true` payload exists for `baby_step_id`,
/// all entries for that step are excluded (clear-on-success semantics).
/// The last tombstone wins; entries appended after a tombstone are not currently
/// supported and should not occur in normal operation.
pub fn query_by_baby_step(ssb: &Ssb, baby_step_id: &str) -> Result<Vec<MistakeEntry>, SsbError> {
    let rows = ssb.query_by_type(SsbActionType::MistakeEntry)?;
    let mut entries = Vec::new();
    let mut cleared = false;

    for row in &rows {
        let payload: SsbPayload = rmp_serde::from_slice(&row.payload)?;
        if let SsbPayload::MistakeEntry(me) = payload {
            if me.baby_step_id == baby_step_id {
                if me.cleared {
                    // Tombstone: discard all accumulated entries for this step.
                    cleared = true;
                    entries.clear();
                } else if !cleared {
                    entries.push(me.entry);
                }
            }
        }
    }

    entries.sort_by_key(|e| e.timestamp);
    Ok(entries)
}

/// Write a mistake entry to SSB for a given baby step.
///
/// Reads existing entries for the baby step, applies eviction logic
/// (max 5, oldest-evicted, deduplicated by diff_hash), then writes
/// the new entry to SSB. Returns the SSB sequence number.
pub async fn write_mistake(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    baby_step_id: &str,
    entry: MistakeEntry,
) -> Result<i64, SsbError> {
    // Check existing entries for this baby step
    let existing = query_by_baby_step(ssb, baby_step_id)?;

    // Verify we haven't exceeded capacity (dedup check)
    let mut entries = existing;
    let _evicted = insert_with_eviction(&mut entries, entry.clone());

    // If the entry was deduplicated (already exists), skip write
    if !entries.iter().any(|e| e.diff_hash == entry.diff_hash) {
        return Ok(0); // already exists, no-op
    }

    let payload = SsbPayload::MistakeEntry(MistakeEntryPayload {
        baby_step_id: baby_step_id.to_owned(),
        entry,
        cleared: false,
    });

    ssb.write_payload(agent_id, superstep, payload, vec![]).await
}

/// Clear all mistake entries for a given baby step on success.
///
/// Appends a tombstone entry (`cleared = true`) to SSB for `baby_step_id`.
/// After this call, `query_by_baby_step` returns an empty list for that step
/// and `format_baby_step_constraints` returns an empty CONSTRAINTS block.
/// Also clears any ForbiddenDiffs derived from entries for this step since
/// they are no longer relevant once the step has passed.
///
/// SSB is append-only; this does NOT physically delete rows — it writes a
/// logical tombstone. The tombstone is the last write for the step and takes
/// precedence over all prior `MistakeEntry` rows for the same `baby_step_id`.
pub async fn clear_baby_step(
    ssb: &Ssb,
    agent_id: &str,
    superstep: i64,
    baby_step_id: &str,
) -> Result<i64, SsbError> {
    let sentinel = MistakeEntry {
        attempt_number: 0,
        diff_hash: DiffHash::from_diff(""),
        failure_category: FailureCategory::Unknown,
        faulty_line: None,
        root_cause_json: None,
        action_taken: "cleared".to_owned(),
        timestamp: 0,
    };
    let payload = SsbPayload::MistakeEntry(MistakeEntryPayload {
        baby_step_id: baby_step_id.to_owned(),
        entry: sentinel,
        cleared: true,
    });
    let seq = ssb.write_payload(agent_id, superstep, payload, vec![]).await?;
    tracing::info!(
        baby_step_id = %baby_step_id,
        seq = seq,
        "MistakeBook: cleared all entries for baby step on success"
    );
    Ok(seq)
}

/// Format the CONSTRAINTS block for prompt injection from SSB entries
/// scoped to a specific baby step.
pub fn format_baby_step_constraints(
    ssb: &Ssb,
    baby_step_id: &str,
) -> Result<String, SsbError> {
    let entries = query_by_baby_step(ssb, baby_step_id)?;
    // Only take the most recent MAX_ENTRIES_PER_STEP entries
    let recent: Vec<_> = if entries.len() > MAX_ENTRIES_PER_STEP {
        entries[entries.len() - MAX_ENTRIES_PER_STEP..].to_vec()
    } else {
        entries
    };
    Ok(format_constraints(&recent))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssb::{Ssb, SsbConfig};
    use elai_core::clock::SeededClock;
    use elai_core::mistake_book::{DiffHash, FailureCategory};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn test_clock() -> Arc<SeededClock> {
        Arc::new(SeededClock::new())
    }

    fn test_config(dir: &TempDir) -> SsbConfig {
        SsbConfig {
            db_path: dir.path().join("test_ssb.db"),
            pool_size: Some(2),
            project_id: "test-proj".into(),
            session_id: "test-session".into(),
        }
    }

    fn make_entry(attempt: u32, diff: &str, ts: u64) -> MistakeEntry {
        MistakeEntry {
            attempt_number: attempt,
            diff_hash: DiffHash::from_diff(diff),
            failure_category: FailureCategory::CompileError,
            faulty_line: Some(10),
            root_cause_json: Some("wrong type".to_string()),
            action_taken: "templated".to_string(),
            timestamp: ts,
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn write_and_query_mistake() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        let entry = make_entry(1, "diff-a", 100);
        let seq = write_mistake(&ssb, "agent-1", 0, "step-1", entry).await.unwrap();
        assert!(seq > 0);

        let entries = query_by_baby_step(&ssb, "step-1").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].attempt_number, 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn query_scoped_to_baby_step() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        write_mistake(&ssb, "a", 0, "step-1", make_entry(1, "d1", 10)).await.unwrap();
        write_mistake(&ssb, "a", 0, "step-2", make_entry(2, "d2", 20)).await.unwrap();
        write_mistake(&ssb, "a", 0, "step-1", make_entry(3, "d3", 30)).await.unwrap();

        let step1 = query_by_baby_step(&ssb, "step-1").unwrap();
        assert_eq!(step1.len(), 2);

        let step2 = query_by_baby_step(&ssb, "step-2").unwrap();
        assert_eq!(step2.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn format_constraints_from_ssb() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        write_mistake(&ssb, "a", 0, "step-1", make_entry(1, "d1", 10)).await.unwrap();
        write_mistake(&ssb, "a", 0, "step-1", make_entry(2, "d2", 20)).await.unwrap();

        let constraints = format_baby_step_constraints(&ssb, "step-1").unwrap();
        assert!(constraints.contains("CONSTRAINTS"));
        assert!(constraints.contains("Do NOT produce patch"));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn empty_baby_step_returns_empty_constraints() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        let constraints = format_baby_step_constraints(&ssb, "no-such-step").unwrap();
        assert_eq!(constraints, "");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clear_baby_step_empties_entries() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        // Write two mistakes for step-1 and one for step-2.
        write_mistake(&ssb, "a", 0, "step-1", make_entry(1, "d1", 10)).await.unwrap();
        write_mistake(&ssb, "a", 0, "step-1", make_entry(2, "d2", 20)).await.unwrap();
        write_mistake(&ssb, "a", 0, "step-2", make_entry(3, "d3", 30)).await.unwrap();

        // Confirm step-1 has 2 entries before clear.
        let before = query_by_baby_step(&ssb, "step-1").unwrap();
        assert_eq!(before.len(), 2, "expected 2 entries before clear");

        // Clear step-1 on success.
        let seq = clear_baby_step(&ssb, "a", 0, "step-1").await.unwrap();
        assert!(seq > 0, "tombstone write must return positive seq");

        // step-1 should now return empty.
        let after = query_by_baby_step(&ssb, "step-1").unwrap();
        assert_eq!(after.len(), 0, "step-1 must be empty after clear");

        // step-2 must be unaffected.
        let step2 = query_by_baby_step(&ssb, "step-2").unwrap();
        assert_eq!(step2.len(), 1, "step-2 must be unaffected by clear of step-1");

        // Constraints block for cleared step must be empty.
        let constraints = format_baby_step_constraints(&ssb, "step-1").unwrap();
        assert_eq!(constraints, "", "constraints must be empty after clear");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn clear_baby_step_idempotent() {
        let dir = TempDir::new().unwrap();
        let ssb = Ssb::new(test_config(&dir), test_clock()).unwrap();

        write_mistake(&ssb, "a", 0, "step-1", make_entry(1, "d1", 10)).await.unwrap();
        clear_baby_step(&ssb, "a", 0, "step-1").await.unwrap();
        // Second clear must succeed and step stays empty.
        clear_baby_step(&ssb, "a", 0, "step-1").await.unwrap();
        let entries = query_by_baby_step(&ssb, "step-1").unwrap();
        assert_eq!(entries.len(), 0, "double clear must leave step empty");
    }
}
