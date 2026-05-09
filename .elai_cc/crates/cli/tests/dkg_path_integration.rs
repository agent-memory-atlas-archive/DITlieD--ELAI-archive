//! B-1 integration test (DEFERRED — manual verification).
//!
//! Spawning a full elai REPL on a fixture project is too heavy for
//! `cargo test` (model loading, sidecar handshake, ONNX load). Document
//! the manual reproduction steps and ship an `#[ignore]` stub so the
//! contract surface stays visible to readers and the auditor.
//!
//! ## Manual reproduction
//!
//! 1. `cd D:\.ELAI_workflow\.elai_cc`
//! 2. Pre-build a DKG via:
//!    `cargo run --bin elai -- index` (or any path that triggers
//!    `IndexingService::run_full_index`), and confirm
//!    `D:\.ELAI_workflow\.elai_cc\.elai_cc\indexes\dkg.db` exists with
//!    a non-zero `SELECT COUNT(*) FROM dkg_nodes`.
//! 3. `cargo run --bin elai -- repl "list rust files"` in the same
//!    project root.
//! 4. Confirm REPL output references real file paths from the DKG (NOT
//!    an empty PageRank result).
//! 5. Confirm logs contain `core::paths::dkg_db_path` (or the canonical
//!    `.elai_cc/indexes/dkg.db` path) and never `.elai/dkg.db`.
//!
//! ## Why this is not run automatically
//!
//! - Requires the abliterated Qwen2.5-Coder model on disk.
//! - Requires the Python sidecar to be reachable (or fall back to
//!   sidecar-disabled mode, which would defeat the test).
//! - Requires a populated DKG, which itself requires a long
//!   `IndexingService::run_full_index` over the fixture project.
//!
//! The orchestrator-level path contract in
//! `crates/orchestrator/tests/indexing_service_path_contract.rs` and
//! the workspace regression scan in
//! `crates/contracts/tests/path_unification.rs` cover the static
//! guarantees end-to-end. This file remains a documentation hook.

#[test]
#[ignore = "manual: requires sidecar + model + populated DKG; see file docblock"]
fn cli_repl_query_uses_canonical_dkg_db_path_manual_only() {
    // Intentionally empty — runner is a smoke harness; see docblock for
    // step-by-step manual verification.
}
