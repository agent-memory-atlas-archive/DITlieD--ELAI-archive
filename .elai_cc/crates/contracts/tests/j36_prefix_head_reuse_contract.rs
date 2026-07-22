//! J-36 slice-5 — frozen-head prefix reuse contract (source-wiring gate).
//!
//! Verifies the structural claims from the J-36 S5 slice for the frozen-head
//! ContextPolicy Local arm:
//!
//! D1: `FROZEN_HEAD_SEQ_BOUNDARY` +
//!     `frozen_head_system_prefix_len` exist in context_policy.rs AND are
//!     re-exported from lib.rs.
//! D2: `compact_session` body references `frozen_head_system_prefix_len`
//!     (PRODUCTION_HOOK for the compact path).
//! D3: `trim_context` body references `FROZEN_HEAD_SEQ_BOUNDARY`
//!     (PRODUCTION_HOOK for the trim path).
//! D4: `FROZEN_HEAD_SEQ_BOUNDARY` source text contains `= 3` AND
//!     `PROMPT_CACHE_PREFIX_BOUNDARY` source contains `= 3` (no second
//!     number).
//! D5: ContextPolicy::Local doc/module text mentions frozen-head / frozen
//!     head (invariant documented).
//! D6: shipped J-36 substrate still present (source-grep):
//!     - `prefix_kv_eligible` in context_budget.rs
//!     - `emit_local_prefix_cache_event` in prefix_cache_ssb.rs
//!     - `LocalPrefixCacheKind` / `HeadCacheHit` in payloads or
//!       prefix_cache_ssb
//!     - `cache_read_input_tokens` on CompletionResponse in providers runtime
//!       types
//!
//! W-1 live HeadCacheHit on a real sidecar remains a residual done-gate
//! (not offline-assertable here).

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// D1: symbols exist in context_policy.rs + re-exported from lib.rs
// ---------------------------------------------------------------------------

#[test]
fn d1a_frozen_head_symbols_exist_in_context_policy() {
    let src = read_src("orchestrator/src/context_policy.rs");
    assert!(
        src.contains("pub const FROZEN_HEAD_SEQ_BOUNDARY"),
        "context_policy.rs must export pub const FROZEN_HEAD_SEQ_BOUNDARY"
    );
    assert!(
        src.contains("pub fn frozen_head_system_prefix_len"),
        "context_policy.rs must export pub fn frozen_head_system_prefix_len"
    );
}

#[test]
fn d1b_frozen_head_symbols_reexported_from_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    let block_start = src.find("pub use context_policy::{");
    assert!(block_start.is_some(), "lib.rs must have a context_policy pub use block");
    let block = &src[block_start.unwrap()..];
    let block_end = block.find("};").expect("pub use block terminates");
    let block = &block[..block_end];
    assert!(
        block.contains("FROZEN_HEAD_SEQ_BOUNDARY"),
        "lib.rs must re-export FROZEN_HEAD_SEQ_BOUNDARY"
    );
    assert!(
        block.contains("frozen_head_system_prefix_len"),
        "lib.rs must re-export frozen_head_system_prefix_len"
    );
}

// ---------------------------------------------------------------------------
// D2: compact_session references frozen_head_system_prefix_len
// ---------------------------------------------------------------------------

#[test]
fn d2_compact_session_calls_frozen_head_helper() {
    let src = read_src("orchestrator/src/compact.rs");
    let fn_start = src.find("pub fn compact_session");
    assert!(fn_start.is_some(), "compact.rs must define compact_session");
    let body = &src[fn_start.unwrap()..];
    let next_fn = body[1..].find("\nfn ").map(|i| i + 1).unwrap_or(body.len());
    let body = &body[..next_fn];
    assert!(
        body.contains("frozen_head_system_prefix_len"),
        "compact_session must call frozen_head_system_prefix_len inside its body"
    );
}

// ---------------------------------------------------------------------------
// D3: trim_context references FROZEN_HEAD_SEQ_BOUNDARY
// ---------------------------------------------------------------------------

#[test]
fn d3_trim_context_references_frozen_head_boundary() {
    let src = read_src("orchestrator/src/session_dag.rs");
    let fn_start = src.find("pub fn trim_context");
    assert!(fn_start.is_some(), "session_dag.rs must define trim_context");
    let body = &src[fn_start.unwrap()..];
    let next_fn = body[1..].find("\n    pub fn ").map(|i| i + 1).unwrap_or(body.len());
    let body = &body[..next_fn];
    assert!(
        body.contains("FROZEN_HEAD_SEQ_BOUNDARY"),
        "trim_context must reference FROZEN_HEAD_SEQ_BOUNDARY inside its body"
    );
}

// ---------------------------------------------------------------------------
// D4: both boundaries equal 3 (no second number)
// ---------------------------------------------------------------------------

#[test]
fn d4_both_boundaries_equal_three() {
    let cp_src = read_src("orchestrator/src/context_policy.rs");
    let cp_line = cp_src
        .lines()
        .find(|l| l.contains("FROZEN_HEAD_SEQ_BOUNDARY") && l.contains("= 3"))
        .unwrap_or_else(|| {
            panic!("FROZEN_HEAD_SEQ_BOUNDARY must be defined with `= 3`")
        });
    assert!(
        cp_line.contains("pub const FROZEN_HEAD_SEQ_BOUNDARY: u32 = 3;"),
        "FROZEN_HEAD_SEQ_BOUNDARY must be `pub const FROZEN_HEAD_SEQ_BOUNDARY: u32 = 3;`"
    );

    let sd_src = read_src("orchestrator/src/session_dag.rs");
    let sd_line = sd_src
        .lines()
        .find(|l| l.contains("PROMPT_CACHE_PREFIX_BOUNDARY") && l.contains("= 3"))
        .unwrap_or_else(|| {
            panic!("PROMPT_CACHE_PREFIX_BOUNDARY must be defined with `= 3`")
        });
    assert!(
        sd_line.contains("pub const PROMPT_CACHE_PREFIX_BOUNDARY: u32 = 3;"),
        "PROMPT_CACHE_PREFIX_BOUNDARY must be `pub const PROMPT_CACHE_PREFIX_BOUNDARY: u32 = 3;`"
    );
}

// ---------------------------------------------------------------------------
// D5: ContextPolicy::Local doc mentions frozen-head invariant
// ---------------------------------------------------------------------------

#[test]
fn d5_local_doc_mentions_frozen_head() {
    let src = read_src("orchestrator/src/context_policy.rs");
    // The module header or the Local enum doc must mention "frozen head" or
    // "frozen-head".
    assert!(
        src.contains("frozen head") || src.contains("frozen-head"),
        "context_policy.rs must document the frozen-head invariant"
    );
    // The Local enum variant doc must mention frozen head.
    let local_start = src.find("Local,").unwrap_or_else(|| {
        panic!("ContextPolicy::Local variant must exist")
    });
    let local_doc = &src[..local_start];
    assert!(
        local_doc.contains("frozen head") || local_doc.contains("frozen-head"),
        "ContextPolicy::Local doc must mention frozen-head invariant"
    );
}

// ---------------------------------------------------------------------------
// D6: shipped J-36 substrate still present
// ---------------------------------------------------------------------------

#[test]
fn d6a_prefix_kv_eligible_in_context_budget() {
    let src = read_src("orchestrator/src/context_budget.rs");
    assert!(
        src.contains("prefix_kv_eligible"),
        "context_budget.rs must still define prefix_kv_eligible"
    );
}

#[test]
fn d6b_emit_local_prefix_cache_event_in_prefix_cache_ssb() {
    let src = read_src("orchestrator/src/prefix_cache_ssb.rs");
    assert!(
        src.contains("emit_local_prefix_cache_event"),
        "prefix_cache_ssb.rs must still define emit_local_prefix_cache_event"
    );
}

#[test]
fn d6c_head_cache_hit_kind_present() {
    let ssb_src = read_src("orchestrator/src/ssb.rs");
    assert!(
        ssb_src.contains("HeadCacheHit"),
        "ssb.rs must still reference HeadCacheHit"
    );
    let prefix_src = read_src("orchestrator/src/prefix_cache_ssb.rs");
    assert!(
        prefix_src.contains("HeadCacheHit"),
        "prefix_cache_ssb.rs must still reference HeadCacheHit"
    );
}

#[test]
fn d6d_cache_read_input_tokens_on_completion_response() {
    let src = read_src("providers/src/runtime/types.rs");
    assert!(
        src.contains("cache_read_input_tokens"),
        "providers/src/runtime/types.rs must still define cache_read_input_tokens"
    );
}
