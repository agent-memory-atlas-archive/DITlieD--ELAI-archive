//! Memory-taint digest rendering for the optional persistent memory path.
//!
//! Kept separate from the handoff author so the parent remains below the
//! maintainability line cap. This helper preserves the LEG-C behavior: the
//! section is enabled only by the truthy persistence flag, store misses are
//! rendered as `untrusted`, and fact IDs are rendered in caller-provided
//! (already deduplicated) order.

use std::path::Path;

/// Extract and BTreeSet-deduplicate memory fact IDs from the session stream.
pub(super) fn extract_fact_ids(entries: &[crate::ssb::SsbEntry]) -> Vec<String> {
    use std::collections::BTreeSet;
    use crate::ssb::{SsbActionType, SsbPayload};

    let mut ids = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in entries {
        if !matches!(entry.action_type, SsbActionType::MemoryFactExtracted | SsbActionType::MemoryFactSuperseded) {
            continue;
        }
        let Ok(payload) = rmp_serde::from_slice::<SsbPayload>(&entry.payload) else { continue; };
        let id = match payload {
            SsbPayload::MemoryFactExtracted(payload) => payload.fact_id,
            SsbPayload::MemoryFactSuperseded(payload) => payload.new_fact_id,
            _ => String::new(),
        };
        let id = id.trim().to_owned();
        if !id.is_empty() && seen.insert(id.clone()) {
            ids.push(id);
        }
    }
    ids
}

/// Build the optional MEMORY TAINT section body.
///
/// `None` means the feature flag is disabled; `Some` contains the exact body
/// that the typed digest renderer should associate with `MemoryTaint`.
pub(super) fn section_body(memory_fact_ids: &[String], cwd: &Path) -> Option<String> {
    let enabled = std::env::var("ELAI_MEMORY_TAINT_PERSIST_ENABLE")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false);
    if !enabled {
        return None;
    }

    let mut body = String::new();
    let store = elai_memory::FactStore::open(cwd.join("memory_facts.sqlite")).ok();
    for fact_id in memory_fact_ids {
        let label = store
            .as_ref()
            .and_then(|store| store.g1_taint(fact_id).ok().flatten())
            .unwrap_or_else(|| "untrusted".to_string());
        body.push_str(&format!("- {fact_id}: {label}\n"));
    }
    if body.is_empty() {
        body.push_str("(no memory facts written this session)\n");
    }
    Some(body)
}
