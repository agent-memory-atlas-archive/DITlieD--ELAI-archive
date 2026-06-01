//! C-42b (Tranche M, s-22600) -- deterministic 200-tuple corpus generator
//! for the `plugins_macros` fuzz harness.
//!
//! Emits a line-delimited tuple corpus at
//! `crates/plugins_macros/tests/corpus/mcp_authz_200_tuples.txt`. Each
//! line is `tool_name|authz_decision|expected_emit_count` (pipe-delimited
//! so the parser is whitespace-insensitive and tool names can carry
//! dots / underscores without quoting).
//!
//! Emit-count semantics (per Tranche D D7 dual-emit decision, see
//! `workflow/sessions/s-22600/exploration/trancheD-integration.md`):
//!   * `Allow` -> 2 (gateway authz emit + composed tool-gate emit)
//!   * `Deny`  -> 2 (gateway authz emit + composed deny emit; D7 dual-emit
//!                   guarantees both layers fire even on rejection)
//!   * `Ask`   -> 1 (HITL pending; only the gateway authz emit fires --
//!                   no composed verdict yet)
//!
//! Determinism: `rand` 0.9 (anti-stale-training compliant -- never 0.8)
//! with a hard-coded seed (`CORPUS_SEED = 42`) so the same 200 lines
//! drop out of every run. Reproducibility lets the contract test at
//! `crates/contracts/tests/m_authz_corpus_loop_contract.rs` parse the
//! file and assert per-tuple emit-count expectations without re-running
//! the generator inside the test.
//!
//! API surface (CLAUDE.md HARD RULE anti-stale-training): the snippet
//! uses the `rand` 0.9 names (`random_range`, `random_bool`) -- NOT the
//! deprecated `gen_*` 0.8 spellings.
//!
//! BUILD / RUN:
//!   cargo run --example corpusgen -p elai_plugins_macros
//!
//! The output file path is workspace-relative; run from the workspace
//! root (`F:/.ELAI_workflow/.elai_cc`).

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Output path, workspace-relative.
const CORPUS_PATH: &str =
    "crates/plugins_macros/tests/corpus/mcp_authz_200_tuples.txt";

/// Hard-coded RNG seed so every run produces the same 200 lines.
const CORPUS_SEED: u64 = 42;

/// Total tuple count.
const TUPLE_COUNT: usize = 200;

/// Realistic MCP tool names sampled at random. Lifted from the
/// dispatcher surface enumerated at
/// `crates/plugins/src/mcp_bridge.rs` plus a handful of common system
/// tools (Read / Write / Edit / Glob / Grep / Bash) that appear on
/// every dispatch path.
const TOOL_NAMES: &[&str] = &[
    "Read",
    "Write",
    "Edit",
    "Glob",
    "Grep",
    "Bash",
    "mcp__local-rag__mcp_search_context",
    "mcp__local-rag__mcp_memorize",
    "mcp__local-rag__mcp_log_change",
    "mcp__local-rag__mcp_ingest_file",
    "mcp__local-rag__mcp_think",
    "mcp__local-rag__mcp_fsm_get_state",
    "mcp__local-rag__mcp_fsm_transition",
    "mcp__pod-lifecycle__pod_create",
    "mcp__pod-lifecycle__pod_stop",
    "mcp__pod-lifecycle__pod_list",
    "mcp__notion__authenticate",
    "WebFetch",
    "WebSearch",
    "NotebookEdit",
];

/// Per-decision expected emit count -- see module doc.
fn expected_emit_count(decision: &str) -> u8 {
    match decision {
        "Allow" => 2,
        "Deny" => 2,
        "Ask" => 1,
        // Any value outside the three-arm enum is a generator bug; bail.
        // .expect() acceptable here per CLAUDE.md "bootloader init where
        // panic = won't start, you'll see it immediately" exception.
        other => panic!("corpusgen: unknown authz_decision variant `{}`", other),
    }
}

fn main() {
    let path = PathBuf::from(CORPUS_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("corpusgen: failed to create corpus parent dir");
    }

    let mut rng = StdRng::seed_from_u64(CORPUS_SEED);
    let mut out = fs::File::create(&path).expect("corpusgen: failed to open corpus output file");

    for _ in 0..TUPLE_COUNT {
        let tool_idx = rng.random_range(0..TOOL_NAMES.len());
        let tool = TOOL_NAMES[tool_idx];

        // 50/50 split between Allow/Deny + a ~20% Ask sprinkle to
        // exercise the HITL branch. random_bool first to bias the
        // Allow/Deny split, then random_range for the Ask carve-out.
        let decision = if rng.random_bool(0.20) {
            "Ask"
        } else if rng.random_bool(0.50) {
            "Allow"
        } else {
            "Deny"
        };

        let emit_count = expected_emit_count(decision);
        writeln!(out, "{}|{}|{}", tool, decision, emit_count)
            .expect("corpusgen: failed to write tuple line");
    }

    out.flush().expect("corpusgen: failed to flush corpus file");
    eprintln!(
        "corpusgen: wrote {} tuples (seed={}) to {}",
        TUPLE_COUNT, CORPUS_SEED, CORPUS_PATH
    );
}
