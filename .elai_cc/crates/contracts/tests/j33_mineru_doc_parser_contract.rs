//! J-33 Tier-1 contract — MinerU doc-parser client wiring.
//!
//! Pins the env-gate semantics of
//! [`elai_knowledge::doc_parser_client`] so the flag-OFF default and the
//! flag-ON-no-backend failure mode stay stable across future refactors.
//!
//! Grounding:
//!   * crates/knowledge/src/doc_parser_client.rs (DOC_PARSER_ENABLE_ENV,
//!     doc_parser_enabled_from_env, DocParserClient::from_env / from_env_or_disable)
//!   * crates/knowledge/src/artifacts/indexer.rs (index_single_file pdf/docx branch)
//!   * crates/orchestrator/src/indexing_service.rs (is_binary_file pdf/docx bypass)
//!   * .elai_cc/wire-dark-allowlist.toml ELAI_DOC_PARSER_ENABLE row

use std::sync::Mutex;
use std::time::Duration;

use elai_knowledge::chunking::pdf_structured::ParserBackend;
use elai_knowledge::doc_parser_client::{
    doc_parser_enabled_from_env, DocParserClient, DocParserError,
    DOC_PARSER_ENABLE_ENV, DOC_PARSER_BASE_URL_ENV,
};

// Env-mutating tests share global env vars; serialize them so `cargo test`
// (without --test-threads=1) does not flake. Tests that do not touch env
// (doc_parser_dead_port_returns_service_unavailable_no_hang) skip the lock.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Helper: clear both env vars for a deterministic flag-OFF baseline.
/// Caller must hold ENV_LOCK.
fn clear_doc_parser_env() {
    std::env::remove_var(DOC_PARSER_ENABLE_ENV);
    std::env::remove_var(DOC_PARSER_BASE_URL_ENV);
}

// --- (a) Flag OFF => doc_parser_enabled_from_env is false. ---

#[test]
fn doc_parser_flag_off_returns_false() {
    let _g = ENV_LOCK.lock().unwrap();
    clear_doc_parser_env();
    assert!(
        !doc_parser_enabled_from_env(),
        "flag must be OFF when ELAI_DOC_PARSER_ENABLE is unset"
    );
}

// --- (b) Flag OFF => DocParserClient::from_env returns None. ---

#[test]
fn doc_parser_from_env_none_when_flag_off() {
    let _g = ENV_LOCK.lock().unwrap();
    clear_doc_parser_env();
    assert!(
        DocParserClient::from_env().is_none(),
        "from_env must return None (no-op path) when the flag is OFF"
    );
}

// --- (c) Flag ON + base_url unset => from_env_or_disable errors BaseUrlMissing. ---

#[test]
fn doc_parser_flag_on_without_base_url_errors() {
    let _g = ENV_LOCK.lock().unwrap();
    clear_doc_parser_env();
    std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
    let r = DocParserClient::from_env_or_disable();
    clear_doc_parser_env();
    assert!(
        matches!(r, Err(DocParserError::BaseUrlMissing)),
        "flag ON + no base_url must surface BaseUrlMissing, not a silent fallthrough"
    );
}

// --- (d) DocParserClient against a dead port => ServiceUnavailable, no hang. ---

#[test]
fn doc_parser_dead_port_returns_service_unavailable_no_hang() {
    let _g = ENV_LOCK.lock().unwrap();
    clear_doc_parser_env();
    // 127.0.0.1:1 is a dead port on every CI host; the 200ms timeout
    // guarantees the test cannot hang even if the connect blocks.
    let client = DocParserClient::new(
        "http://127.0.0.1:1".into(),
        ParserBackend::MinerU,
        false,
    )
    .with_timeout(Duration::from_millis(200));

    let tmp = tempfile::NamedTempFile::new().expect("tmp");
    let r = client.parse(tmp.path());
    clear_doc_parser_env();
    drop(_g);
    assert!(
        matches!(r, Err(DocParserError::ServiceUnavailable { .. })),
        "dead port must surface ServiceUnavailable, not hang or return Ok"
    );
}

// --- (e) Flag ON + base_url set => from_env constructs a MinerU client. ---

#[test]
fn doc_parser_flag_on_with_base_url_constructs_client() {
    let _g = ENV_LOCK.lock().unwrap();
    clear_doc_parser_env();
    std::env::set_var(DOC_PARSER_ENABLE_ENV, "1");
    std::env::set_var(DOC_PARSER_BASE_URL_ENV, "http://127.0.0.1:8760");
    let r = DocParserClient::from_env_or_disable();
    clear_doc_parser_env();
    let c = r.expect("client must construct when flag ON + base_url set");
    assert_eq!(c.backend(), ParserBackend::MinerU);
    assert_eq!(c.base_url(), "http://127.0.0.1:8760");
}
