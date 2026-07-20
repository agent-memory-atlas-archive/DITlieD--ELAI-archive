//! J-25 T3 — tool-pair boundary guard contract (source-wiring gate).
//!
//! Verifies the four structural claims from the J-25 WIRING_CONTRACT for the
//! T3 slice (FORK-1 ROUTE A: tool_use/tool_result un-orphan only):
//!
//! D1: `adjust_boundary_for_invariants` exists as a pure fn in
//!     crates/orchestrator/src/context_boundary.rs and is declared in
//!     lib.rs (module reachable from the crate root).
//! D2: the fn has a non-test production caller in
//!     session_dag.rs::trim_context (R6/R7 production reachability).
//! D3: the fn body contains no `.unwrap()` / `.expect()` on prod paths
//!     (DETERMINISTIC_CHECK: `rg -c "unwrap\(\)" context_boundary.rs`
//!     MUST be 0).
//! D4: the guard drops a ToolResult candidate whose matching ToolUse
//!     stays in-window (the Anthropic-invariant guard). This is the
//!     REDTEAM ATTACK-1 positive leg: with the guard ENABLED, the
//!     orphaning candidate is removed.
//!
//! The NEGATIVE leg (guard DISABLED produces the malformed request) is
//! not exercisable at the source-grep level — it requires running
//! `trim_context` with the guard bypassed, which is not exposed in
//! production. The unit test in context_boundary.rs::tests covers the
//! red-then-green behavioral assertion; this contract test locks the
//! structural wiring.

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
// D1: adjust_boundary_for_invariants exists + module declared in lib.rs
// ---------------------------------------------------------------------------

/// D1a: the context_boundary module file exists and exports the fn.
#[test]
fn d1a_context_boundary_module_exists() {
    let src = read_src("orchestrator/src/context_boundary.rs");
    assert!(
        src.contains("pub fn adjust_boundary_for_invariants"),
        "context_boundary.rs must export adjust_boundary_for_invariants"
    );
}

/// D1b: lib.rs declares the module.
#[test]
fn d1b_context_boundary_declared_in_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    assert!(
        src.contains("pub mod context_boundary;"),
        "lib.rs must declare `pub mod context_boundary;`"
    );
}

// ---------------------------------------------------------------------------
// D2: non-test production caller in session_dag.rs::trim_context
// ---------------------------------------------------------------------------

/// D2: session_dag.rs::trim_context calls the guard. The call must be
/// inside the trim_context fn body (not in a test module) and must
/// reference the fully-qualified path or the crate-local alias.
#[test]
fn d2_trim_context_calls_guard() {
    let src = read_src("orchestrator/src/session_dag.rs");
    assert!(
        src.contains("adjust_boundary_for_invariants"),
        "session_dag.rs must call adjust_boundary_for_invariants"
    );
    // The call must appear in the trim_context body, not only in tests.
    // Find the fn and check the call is inside it.
    let trim_start = src.find("pub fn trim_context");
    assert!(
        trim_start.is_some(),
        "session_dag.rs must define pub fn trim_context"
    );
    let trim_start = trim_start.unwrap();
    let trim_body = &src[trim_start..];
    assert!(
        trim_body.contains("adjust_boundary_for_invariants"),
        "adjust_boundary_for_invariants must be called inside trim_context body"
    );
}

// ---------------------------------------------------------------------------
// D3: zero unwrap/expect in context_boundary.rs prod paths
// ---------------------------------------------------------------------------

/// D3: the guard module must not contain `.unwrap()` or `.expect(` on
/// prod paths. Test-module uses are allowed (inside #[cfg(test)]).
#[test]
fn d3_no_unwrap_expect_in_prod_paths() {
    let src = read_src("orchestrator/src/context_boundary.rs");
    // Strip the #[cfg(test)] module before counting — test-only unwraps
    // are allowed.
    let prod_src = match src.find("#[cfg(test)]") {
        Some(idx) => &src[..idx],
        None => src.as_str(),
    };
    assert!(
        !prod_src.contains(".unwrap()"),
        "context_boundary.rs prod path must not contain .unwrap()"
    );
    assert!(
        !prod_src.contains(".expect("),
        "context_boundary.rs prod path must not contain .expect("
    );
}

// ---------------------------------------------------------------------------
// D4: the guard drops orphaning candidates (REDTEAM ATTACK-1 positive)
// ---------------------------------------------------------------------------

/// D4: the fn body must implement the I1 invariant (drop a ToolResult
/// candidate whose matching ToolUse stays in-window). We assert the
/// invariant by exercising the fn directly via the orchestrator crate's
/// public API. This is the REDTEAM ATTACK-1 positive leg: guard ENABLED
/// => orphaning candidate removed.
#[test]
fn d4_guard_drops_orphaning_tool_result_candidate() {
    use elai_orchestrator::context_boundary::adjust_boundary_for_invariants;
    use elai_orchestrator::session_dag::{
        DagMessageRole, TrimmedMessage, PROMPT_CACHE_PREFIX_BOUNDARY,
    };
    use elai_orchestrator::ContentBlock;

    // Build a fixture: an Assistant message carrying a ToolUse, followed
    // by a Tool message carrying the matching ToolResult. The ToolResult
    // is the only stub candidate. The ToolUse stays in-window.
    let assistant = TrimmedMessage {
        seq: PROMPT_CACHE_PREFIX_BOUNDARY + 1,
        role: DagMessageRole::Assistant,
        content: vec![],
        is_stubbed: false,
        stub_ssb_seq: Some(100),
        blocks: Some(vec![ContentBlock::ToolUse {
            id: "tu-1".to_owned(),
            name: "ls".to_owned(),
            input: "{}".to_owned(),
        }]),
    };
    let tool = TrimmedMessage {
        seq: PROMPT_CACHE_PREFIX_BOUNDARY + 2,
        role: DagMessageRole::Tool,
        content: vec![],
        is_stubbed: false,
        stub_ssb_seq: Some(200),
        blocks: Some(vec![ContentBlock::ToolResult {
            tool_use_id: "tu-1".to_owned(),
            tool_name: "ls".to_owned(),
            output: "ok".to_owned(),
            is_error: false,
        }]),
    };
    // Prefix messages to respect the PROMPT_CACHE_PREFIX_BOUNDARY floor.
    let mut messages: Vec<TrimmedMessage> = (1..=PROMPT_CACHE_PREFIX_BOUNDARY)
        .map(|seq| TrimmedMessage {
            seq,
            role: DagMessageRole::User,
            content: vec![],
            is_stubbed: false,
            stub_ssb_seq: None,
            blocks: None,
        })
        .collect();
    messages.push(assistant);
    messages.push(tool);

    let tool_idx = messages.len() - 1;
    let candidates = vec![(tool_idx, 500usize)];

    let out = adjust_boundary_for_invariants(&messages, &candidates);
    assert!(
        out.is_empty(),
        "D4 REDTEAM ATTACK-1: guard ENABLED must drop the orphaning ToolResult candidate; got {:?}",
        out
    );
}

/// D5: the guard KEEPS a pair-safe candidate set (both sides of a
/// tool pair stubbed together). This is the symmetric positive: the
/// guard does not over-prune pair-safe stubbing plans.
#[test]
fn d5_guard_keeps_pair_safe_candidates() {
    use elai_orchestrator::context_boundary::adjust_boundary_for_invariants;
    use elai_orchestrator::session_dag::{
        DagMessageRole, TrimmedMessage, PROMPT_CACHE_PREFIX_BOUNDARY,
    };
    use elai_orchestrator::ContentBlock;

    let assistant = TrimmedMessage {
        seq: PROMPT_CACHE_PREFIX_BOUNDARY + 1,
        role: DagMessageRole::Assistant,
        content: vec![],
        is_stubbed: false,
        stub_ssb_seq: Some(100),
        blocks: Some(vec![ContentBlock::ToolUse {
            id: "tu-1".to_owned(),
            name: "ls".to_owned(),
            input: "{}".to_owned(),
        }]),
    };
    let tool = TrimmedMessage {
        seq: PROMPT_CACHE_PREFIX_BOUNDARY + 2,
        role: DagMessageRole::Tool,
        content: vec![],
        is_stubbed: false,
        stub_ssb_seq: Some(200),
        blocks: Some(vec![ContentBlock::ToolResult {
            tool_use_id: "tu-1".to_owned(),
            tool_name: "ls".to_owned(),
            output: "ok".to_owned(),
            is_error: false,
        }]),
    };
    let mut messages: Vec<TrimmedMessage> = (1..=PROMPT_CACHE_PREFIX_BOUNDARY)
        .map(|seq| TrimmedMessage {
            seq,
            role: DagMessageRole::User,
            content: vec![],
            is_stubbed: false,
            stub_ssb_seq: None,
            blocks: None,
        })
        .collect();
    messages.push(assistant);
    messages.push(tool);

    let assistant_idx = messages.len() - 2;
    let tool_idx = messages.len() - 1;
    let candidates = vec![(assistant_idx, 500usize), (tool_idx, 500usize)];

    let out = adjust_boundary_for_invariants(&messages, &candidates);
    assert_eq!(
        out.len(),
        2,
        "D5: pair-safe candidates (both sides stubbed) must survive; got {:?}",
        out
    );
}
