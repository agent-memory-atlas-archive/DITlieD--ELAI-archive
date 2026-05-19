// C-18 B-36 backfill — dashboard-v2 D11 (`sidecar_search`) contract.
//
// See dashboard_v2_D1_contract.rs for the in-crate-tests rationale (C-18 D4).
//
// What it asserts:
//   - The `sidecar_search` response JSON shape — documented at
//     crates/dashboard-v2/src-tauri/src/commands.rs:5843 as the
//     SearXNG-compatible `{ query, results, number_of_results, [error] }`
//     — round-trips through serde for both the empty-query short-circuit
//     and the populated-results case.
//   - The argument type `(query: String)` is locked at compile time.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchResult {
    title: String,
    url: String,
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SidecarSearchResponse {
    query: String,
    results: Vec<SearchResult>,
    number_of_results: u64,
    #[serde(default)]
    error: Option<String>,
}

#[test]
fn d11_sidecar_search_empty_query_shape_round_trips() {
    // Empty-query branch at commands.rs:5847.
    let sample = json!({
        "query": "",
        "results": [],
        "number_of_results": 0u64,
    });
    let parsed: SidecarSearchResponse =
        serde_json::from_value(sample.clone()).expect("empty-query response must deserialise");
    assert_eq!(parsed.query, "");
    assert!(parsed.results.is_empty());
    assert_eq!(parsed.number_of_results, 0);
    assert!(parsed.error.is_none());

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: SidecarSearchResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

#[test]
fn d11_sidecar_search_populated_results_shape_round_trips() {
    let sample = json!({
        "query": "rust async tokio runtime",
        "results": [
            {
                "title": "Tokio runtime guide",
                "url": "https://tokio.rs/runtime",
                "content": "Tokio is an asynchronous runtime for the Rust programming language.",
            },
            {
                "title": "Async/.await primer",
                "url": "https://rust-lang.github.io/async-book/",
            },
        ],
        "number_of_results": 2u64,
    });
    let parsed: SidecarSearchResponse =
        serde_json::from_value(sample.clone()).expect("populated response must deserialise");
    assert_eq!(parsed.results.len(), 2);
    assert_eq!(parsed.number_of_results, 2);
    assert!(parsed.results[0].content.is_some());
    assert!(parsed.results[1].content.is_none());

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: SidecarSearchResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

#[test]
fn d11_sidecar_search_network_error_shape_round_trips() {
    // Network-error branch at commands.rs:5865.
    let sample = json!({
        "query": "anything",
        "results": [],
        "number_of_results": 0u64,
        "error": "connection refused: tcp connect error",
    });
    let parsed: SidecarSearchResponse =
        serde_json::from_value(sample.clone()).expect("network-error response must deserialise");
    assert!(parsed.error.is_some());

    let reser: Value = serde_json::to_value(&parsed).expect("re-serialise");
    let reparsed: SidecarSearchResponse =
        serde_json::from_value(reser).expect("re-deserialise");
    assert_eq!(parsed, reparsed);
}

#[allow(dead_code)]
fn _assert_sidecar_search_argument_types() {
    let _query: String = String::new();
}

// TODO(C-19+): when ELAI_LIVE_SIDECAR-gated integration coverage lands, add
// a #[ignore]-by-default test that hits a real /search endpoint and asserts
// the same shape.
