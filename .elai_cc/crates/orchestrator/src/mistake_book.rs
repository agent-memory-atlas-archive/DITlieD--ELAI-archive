//! MistakeBook SSB integration: query, insert, and format mistake entries
//! scoped to individual baby steps.
//!
//! Wraps `elai_core::mistake_book` types with SSB persistence.
//! Per baby step: max 5 entries, oldest-evicted, blake3 deduplication.

use elai_core::mistake_book::{
    format_constraints, insert_with_eviction, MistakeEntry, MAX_ENTRIES_PER_STEP,
};

use crate::ssb::{MistakeEntryPayload, Ssb, SsbActionType, SsbError, SsbPayload, SsbReader};

// ---------------------------------------------------------------------------
// MistakeBook — SSB-backed per-baby-step constraint store
// ---------------------------------------------------------------------------

/// Query all mistake entries for a given baby step from SSB.
///
/// Filters SSB entries by action_type = mistake_entry, then deserializes
/// and filters by `baby_step_id`. Returns entries sorted by timestamp ascending.
pub fn query_by_baby_step(ssb: &Ssb, baby_step_id: &str) -> Result<Vec<MistakeEntry>, SsbError> {
    let rows = ssb.query_by_type(SsbActionType::MistakeEntry)?;
    let mut entries = Vec::new();

    for row in &rows {
        let payload: SsbPayload = rmp_serde::from_slice(&row.payload)?;
        if let SsbPayload::MistakeEntry(me) = payload {
            if me.baby_step_id == baby_step_id {
                entries.push(me.entry);
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
    });

    ssb.write_payload(agent_id, superstep, payload, vec![]).await
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
}
