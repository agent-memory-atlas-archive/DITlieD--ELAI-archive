// J-40 T3 R7 contract: ELAI_STUB_SEARCH env opt-out retirement.
// Behavioural falsification, not source-text grep.
//
// Contract calls `select_research_search_backend` (same fn production uses,
// wired at repl_command.rs /research handler). No reimplemented backend
// selection in the test body; no source greps.
//
// FALSIFICATION (auditor-run, NOT executor self-run):
//   RED:  restore the opt-out via CONST inside
//         `select_research_search_backend` in research_pipeline.rs:
//           const STUB_KEY: &str = "ELAI_STUB_SEARCH";
//           if std::env::var(STUB_KEY).ok().as_deref() == Some("1") {
//               return Box::new(StubSearchBackend::new());
//           }
//         The NEW behavioural test MUST go RED: StubSearchBackend::search
//         returns Ok(vec![]) for an unknown query, so the assert
//         `result.is_err()` fails. A hypothetical OLD grep test for
//         `line.contains("ELAI_STUB_SEARCH") && line.contains("std::env::var")`
//         would stay GREEN against this CONST mutation because the substring
//         pair is split across two lines (const + if). That difference is
//         the point of R7.
//   GREEN: delete the restored arm. Re-run. Passes -> GREEN.
//
// UNCOVERED MUTATION: PATH B (construction-failure fallback) still routes
// to StubSearchBackend. SearxSearchBackend::new succeeds even with an
// unreachable URL (builds runtime+client, does not connect), so the Err
// arm cannot be driven without mocking Clock/runtime. PATH B runtime is
// not gated this slice.

use std::sync::Mutex;

use elai_orchestrator::select_research_search_backend;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

struct EnvRestoreGuard {
    key: &'static str,
    prior: Option<std::ffi::OsString>,
}

impl Drop for EnvRestoreGuard {
    fn drop(&mut self) {
        match &self.prior {
            Some(val) => std::env::set_var(self.key, val),
            None => std::env::remove_var(self.key),
        }
    }
}

#[test]
fn env_opt_out_does_not_select_stub() {
    let _lock = ENV_MUTEX
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let prior = std::env::var_os("ELAI_STUB_SEARCH");
    std::env::set_var("ELAI_STUB_SEARCH", "1");
    let _guard = EnvRestoreGuard {
        key: "ELAI_STUB_SEARCH",
        prior,
    };

    let backend = select_research_search_backend("http://127.0.0.1:1/");
    let result = backend.search("r7-probe", 1);

    assert!(
        result.is_err(),
        "ELAI_STUB_SEARCH=1 must NOT select StubSearchBackend; \
         StubSearchBackend::search returns Ok(vec![]) for unknown queries, \
         got Ok(_) which means the retired opt-out is alive again"
    );
}
