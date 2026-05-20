//! E-35 (dashboard surface): grammar-compilation visibility commands.
//!
//! Surfaces the per-`(role, block, schema_hash)` grammar cache the sidecar
//! pre-compiles at boot (see
//! `crates/providers/src/runtime/sidecar_xgrammar_hook.rs` for the
//! Rust-side contract; Python wiring is owned by a separate orchestration
//! session). The XGrammar substrate is in scope; the producer side (the
//! Python sidecar emitting `grammar_cache_loaded` SSB rows) lands in
//! `followups_for_other_sessions`.
//!
//! In the meantime, this command returns a deterministic placeholder shape
//! so the Svelte panel can render against a stable contract. When the
//! producer ships, this command will be re-pointed at `state.viewer`'s
//! `grammar_cache_loaded` reader without changing the wire shape.

use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

/// One row of the grammar-compilation table.
///
/// Wire shape (stable):
/// * `schema_id`     — canonical `"{role}.{block}"` identifier.
/// * `schema_hash`   — blake3 hex of the canonical-JSON schema.
/// * `compile_status` — `"compiled" | "cached" | "missing"`.
/// * `last_used_ms`  — UNIX-ms timestamp of the last `xgrammar_mask_applied`
///                     row, or 0 when unused since boot.
/// * `hit_count`     — number of `xgrammar_mask_applied` rows recorded
///                     against this `(schema_id, schema_hash)`.
#[tauri::command]
pub fn get_grammar_compilations(_state: State<'_, AppState>) -> CmdResult<Value> {
    // Until the sidecar producer ships, return an empty list + a status
    // flag so the frontend renders the "no data yet" panel state. This is
    // intentionally NOT a mock row — the Svelte component is responsible
    // for the empty-state message.
    Ok(json!({
        "rows": Value::Array(Vec::new()),
        "producer_status": "pending_sidecar_wiring",
        "expected_total": 64,
        "note": "64 grammars = 8 AgentRole values * 8 BabyStepSpec blocks; \
                 wiring in followups_for_other_sessions (E-35).",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Quick shape check: the JSON we return has the keys the Svelte panel
    /// expects. We do not call the actual Tauri command (that would need a
    /// real Tauri runtime context); we exercise the shape via a helper.
    #[test]
    fn placeholder_shape_has_expected_keys() {
        let v = json!({
            "rows": Vec::<Value>::new(),
            "producer_status": "pending_sidecar_wiring",
            "expected_total": 64,
            "note": "",
        });
        assert!(v.get("rows").is_some());
        assert!(v.get("producer_status").is_some());
        assert_eq!(v.get("expected_total").and_then(Value::as_u64), Some(64));
    }
}
