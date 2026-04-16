// Contract tests for elai-web-search public API surface.
// Validates struct fields, error variants, and pure function contracts.

use elai_web_search::{
    SearchError, SearchResult, strip_html, parse_domain,
    DEFAULT_SEARCH_TIMEOUT_SECS, HEALTH_CHECK_TIMEOUT_SECS,
    DEFAULT_MAX_RESULTS, DEFAULT_RATE_LIMIT_INTERVAL_MS,
};

// --- SearchResult field contract ---

#[test]
fn search_result_has_required_fields() {
    let result = SearchResult {
        title: "title".to_owned(),
        url: "https://example.com".to_owned(),
        snippet: "snippet".to_owned(),
        engine: "duckduckgo".to_owned(),
        score: 1.0,
        parsed_domain: "example.com".to_owned(),
    };
    assert_eq!(result.title, "title");
    assert_eq!(result.url, "https://example.com");
    assert_eq!(result.snippet, "snippet");
    assert_eq!(result.engine, "duckduckgo");
    assert_eq!(result.score, 1.0);
    assert_eq!(result.parsed_domain, "example.com");
}

#[test]
fn search_result_serializes_to_json() {
    let result = SearchResult {
        title: "t".to_owned(),
        url: "u".to_owned(),
        snippet: "s".to_owned(),
        engine: "e".to_owned(),
        score: 0.5,
        parsed_domain: "d".to_owned(),
    };
    let json = serde_json::to_string(&result).expect("serialize");
    assert!(json.contains("\"title\":\"t\""));
    assert!(json.contains("\"score\":0.5"));
}

// --- SearchError variant contract ---

#[test]
fn search_error_rate_limited_display() {
    let err = SearchError::RateLimited;
    assert_eq!(format!("{err}"), "Rate limit exceeded");
}

#[test]
fn search_error_service_unavailable_display() {
    let err = SearchError::ServiceUnavailable { status: 503 };
    assert!(format!("{err}").contains("503"));
}

// --- strip_html contract ---

#[test]
fn strip_html_removes_tags() {
    assert_eq!(strip_html("<b>bold</b>"), "bold");
    assert_eq!(strip_html("no tags"), "no tags");
    assert_eq!(strip_html(""), "");
}

// --- parse_domain contract ---

#[test]
fn parse_domain_extracts_host() {
    assert_eq!(parse_domain("https://docs.rust-lang.org/std/"), "docs.rust-lang.org");
    assert_eq!(parse_domain("http://example.com/path"), "example.com");
    assert_eq!(parse_domain(""), "");
}

// --- Named constants contract ---

#[test]
fn named_constants_have_expected_values() {
    assert_eq!(DEFAULT_SEARCH_TIMEOUT_SECS, 10);
    assert_eq!(HEALTH_CHECK_TIMEOUT_SECS, 2);
    assert_eq!(DEFAULT_MAX_RESULTS, 20);
    assert_eq!(DEFAULT_RATE_LIMIT_INTERVAL_MS, 1000);
}

// --- SearxClient construction contract ---

#[test]
fn searx_client_constructs_with_defaults() {
    use elai_core::clock::OsClock;
    use elai_web_search::client::SearxClient;
    use std::sync::Arc;
    use std::time::Duration;

    let clock = Arc::new(OsClock);
    let client = SearxClient::new("http://localhost:8888", clock).expect("construct");
    assert_eq!(client.base_url(), "http://localhost:8888");
    assert_eq!(client.timeout(), Duration::from_secs(10));
    assert_eq!(client.max_results(), 20);
}
