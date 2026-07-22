// J-40 T3 contract tests: ELAI_STUB_SEARCH env opt-out retirement.
// Exercises the SAME production fn wired at
//   repl_command.rs /research handler (select_research_search_backend call)
//   research_pipeline.rs (select_research_search_backend definition)
// No reimplemented backend selection in the test body.
//
// Falsification procedure (auditor-run, NOT executor self-run):
//   RED:  restore `let use_stub = std::env::var("ELAI_STUB_SEARCH")...` in
//         repl_command.rs or inside select_research_search_backend.
//         Re-run this contract. test_env_opt_out_is_dead FAILS (static-source
//         assert finds ELAI_STUB_SEARCH in a std::env::var call) -> RED.
//   GREEN: delete the restored arm. Re-run. Passes -> GREEN.
//
// UNCOVERED MUTATION: PATH B (construction-failure fallback) still routes
// to StubSearchBackend. SearxSearchBackend::new succeeds even with an
// unreachable URL (builds runtime+client, does not connect), so the Err
// arm cannot be driven without mocking Clock/runtime. PATH B runtime is
// not gated this slice; the static-source assert documents the exemption.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn elai_cc_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(std::path::Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn read_source(rel: &str) -> String {
    let path = elai_cc_root().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

#[test]
fn env_opt_out_is_dead() {
    let pipeline_src = read_source("crates/orchestrator/src/research_pipeline.rs");
    let repl_src = read_source("crates/cli/src/repl_command.rs");

    let pipeline_stub_hits: Vec<&str> = pipeline_src
        .lines()
        .filter(|l| l.contains("ELAI_STUB_SEARCH") && l.contains("std::env::var"))
        .collect();
    assert!(
        pipeline_stub_hits.is_empty(),
        "research_pipeline.rs still reads ELAI_STUB_SEARCH via std::env::var: {:?}",
        pipeline_stub_hits
    );

    let repl_stub_hits: Vec<&str> = repl_src
        .lines()
        .filter(|l| l.contains("ELAI_STUB_SEARCH") && l.contains("std::env::var"))
        .collect();
    assert!(
        repl_stub_hits.is_empty(),
        "repl_command.rs still reads ELAI_STUB_SEARCH via std::env::var: {:?}",
        repl_stub_hits
    );
}

#[test]
fn construction_failure_fallback_is_exempt() {
    let src = read_source("crates/orchestrator/src/research_pipeline.rs");
    assert!(
        src.contains("Err(e)") && src.contains("StubSearchBackend::new()"),
        "select_research_search_backend must retain the Err -> StubSearchBackend::new() \
         construction-failure fallback arm (stub_ban-exempted safety net)"
    );
}

#[test]
fn select_fn_is_re_exported_and_called() {
    let pipeline_src = read_source("crates/orchestrator/src/research_pipeline.rs");
    let lib_src = read_source("crates/orchestrator/src/lib.rs");
    let repl_src = read_source("crates/cli/src/repl_command.rs");

    assert!(
        pipeline_src.contains("pub fn select_research_search_backend"),
        "select_research_search_backend is not defined in research_pipeline.rs"
    );
    assert!(
        lib_src.contains("select_research_search_backend"),
        "select_research_search_backend is not re-exported from lib.rs"
    );
    assert!(
        repl_src.contains("select_research_search_backend"),
        "repl_command.rs does not call select_research_search_backend"
    );
}
