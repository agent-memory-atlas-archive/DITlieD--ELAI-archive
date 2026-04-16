// Contract tests for elai-web-search public API surface.
// Validates struct fields, error variants, and pure function contracts.

use elai_web_search::{
    SearchError, SearchResult, WebSearchError, strip_html, parse_domain,
    DEFAULT_SEARCH_TIMEOUT_SECS, HEALTH_CHECK_TIMEOUT_SECS,
    DEFAULT_MAX_RESULTS, DEFAULT_RATE_LIMIT_INTERVAL_MS,
};
use elai_web_search::source_quality::{
    SourceQualityTable,
    SCORE_TIER1, SCORE_TIER2, SCORE_TIER3, SCORE_TIER4, SCORE_UNKNOWN,
};
use elai_web_search::extractor::{BasicExtractor, ContentExtractor, ExtractedContent, PageType};
use elai_web_search::fetcher::{FetchedPage, FETCH_TIMEOUT_SECS, MAX_BODY_BYTES, MAX_REDIRECTS, USER_AGENT, RATE_LIMIT_INTERVAL_MS};

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

// --- 4D.12 contract tests ---

// WebSearchError variant display contracts.
#[test]
fn web_search_error_robots_denied_display() {
    let err = WebSearchError::RobotsDenied {
        url: "https://example.com/secret".to_owned(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("Robots.txt"));
    assert!(msg.contains("example.com/secret"));
}

#[test]
fn web_search_error_body_too_large_display() {
    let err = WebSearchError::BodyTooLarge {
        size: 3_000_000,
        limit: MAX_BODY_BYTES,
    };
    let msg = format!("{err}");
    assert!(msg.contains("3000000"));
    assert!(msg.contains(&MAX_BODY_BYTES.to_string()));
}

#[test]
fn web_search_error_invalid_url_display() {
    let err = WebSearchError::InvalidUrl {
        url: "bad".to_owned(),
        reason: "no scheme".to_owned(),
    };
    assert!(format!("{err}").contains("bad"));
}

// FetchedPage field contract.
#[test]
fn fetched_page_has_required_fields() {
    let page = FetchedPage {
        url: "https://example.com/final".to_owned(),
        status: 200,
        content_type: "text/html".to_owned(),
        html: "<p>test</p>".to_owned(),
        fetch_time_ms: 100,
    };
    assert_eq!(page.url, "https://example.com/final");
    assert_eq!(page.status, 200);
    assert_eq!(page.content_type, "text/html");
    assert_eq!(page.fetch_time_ms, 100);
}

#[test]
fn fetched_page_serializes_to_json() {
    let page = FetchedPage {
        url: "u".to_owned(),
        status: 200,
        content_type: "ct".to_owned(),
        html: "h".to_owned(),
        fetch_time_ms: 1,
    };
    let json = serde_json::to_string(&page).expect("serialize");
    assert!(json.contains("\"status\":200"));
    assert!(json.contains("\"url\":\"u\""));
}

// ExtractedContent field contract.
#[test]
fn extracted_content_has_required_fields() {
    let content = ExtractedContent {
        title: "Title".to_owned(),
        text: "Some text here".to_owned(),
        word_count: 3,
        page_type: PageType::Article,
        extracted_at: 1000,
    };
    assert_eq!(content.title, "Title");
    assert_eq!(content.word_count, 3);
    assert_eq!(content.page_type, PageType::Article);
    assert_eq!(content.extracted_at, 1000);
}

// PageType enum contract.
#[test]
fn page_type_enum_variants() {
    assert_ne!(PageType::Article, PageType::Empty);
    let json = serde_json::to_string(&PageType::Article).expect("serialize");
    assert!(json.contains("Article"));
}

// ContentExtractor trait contract — BasicExtractor implements it.
#[test]
fn basic_extractor_implements_content_extractor() {
    let extractor = BasicExtractor::new(500);
    let html = "<html><head><title>Test</title></head><body><p>hello world</p></body></html>";
    let result: Result<ExtractedContent, WebSearchError> =
        extractor.extract(html, "https://example.com");
    let content = result.expect("extract");
    assert_eq!(content.title, "Test");
    assert_eq!(content.page_type, PageType::Article);
    assert!(content.word_count > 0);
}

// Named constants contract for 4D.12.
#[test]
fn fetch_constants_have_expected_values() {
    assert_eq!(FETCH_TIMEOUT_SECS, 15);
    assert_eq!(MAX_REDIRECTS, 5);
    assert_eq!(MAX_BODY_BYTES, 2 * 1024 * 1024);
    assert_eq!(RATE_LIMIT_INTERVAL_MS, 500);
    assert!(!USER_AGENT.is_empty());
}

// WebFetcher construction contract.
#[tokio::test]
async fn web_fetcher_constructs() {
    use elai_core::clock::SeededClock;
    use elai_web_search::fetcher::WebFetcher;
    use std::sync::Arc;
    let clock = Arc::new(SeededClock::new());
    let fetcher = WebFetcher::new(clock);
    assert!(fetcher.is_ok());
}

// --- 4D.13 SourceQualityTable contract tests ---

fn contract_test_config() -> elai_core::source_tier::SourceTierConfig {
    let toml = r#"
[tier1]
domains = ["arxiv.org", "doi.org", "ietf.org"]
patterns = ['\.gov$', '\.edu$']

[tier2]
domains = ["docs.rs", "crates.io", "stackoverflow.com"]
min_so_score = 20

[tier4_blocklist]
domains = ["w3schools.com", "medium.com"]
patterns = []
"#;
    elai_core::source_tier::parse_source_tier_config(toml).expect("contract test config")
}

#[test]
fn source_quality_lookup_t1_domain() {
    let table = SourceQualityTable::new_from_config(contract_test_config());
    assert_eq!(
        table.lookup("arxiv.org"),
        elai_core::source_tier::SourceTier::T1Authoritative,
    );
}

#[test]
fn source_quality_lookup_t4_domain() {
    let table = SourceQualityTable::new_from_config(contract_test_config());
    assert_eq!(
        table.lookup("w3schools.com"),
        elai_core::source_tier::SourceTier::T4Rejected,
    );
}

#[test]
fn source_quality_lookup_unknown_defaults_t3weak() {
    let table = SourceQualityTable::new_from_config(contract_test_config());
    assert_eq!(
        table.lookup("some-random-site.com"),
        elai_core::source_tier::SourceTier::T3Weak,
    );
}

#[test]
fn source_quality_score_consistency() {
    let table = SourceQualityTable::new_from_config(contract_test_config());
    assert_eq!(table.score("arxiv.org"), SCORE_TIER1);
    assert_eq!(table.score("docs.rs"), SCORE_TIER2);
    assert_eq!(table.score("some-random.com"), SCORE_TIER3);
    assert_eq!(table.score("w3schools.com"), SCORE_TIER4);
}

#[test]
fn source_quality_subdomain_glob_gov() {
    let table = SourceQualityTable::new_from_config(contract_test_config());
    assert_eq!(
        table.lookup("cdc.gov"),
        elai_core::source_tier::SourceTier::T1Authoritative,
    );
    assert_eq!(
        table.lookup("nih.gov"),
        elai_core::source_tier::SourceTier::T1Authoritative,
    );
}

#[test]
fn source_quality_new_from_config_roundtrip() {
    let config = contract_test_config();
    let table = SourceQualityTable::new_from_config(config);
    // T1 exact
    assert_eq!(table.score("doi.org"), SCORE_TIER1);
    // T2 exact
    assert_eq!(table.score("crates.io"), SCORE_TIER2);
    // T4 exact
    assert_eq!(table.score("medium.com"), SCORE_TIER4);
    // Unknown -> T3Weak
    assert_eq!(table.score("nope.org"), SCORE_TIER3);
}

#[test]
fn source_quality_named_constants_values() {
    assert_eq!(SCORE_TIER1, 1.0);
    assert_eq!(SCORE_TIER2, 0.8);
    assert_eq!(SCORE_TIER3, 0.6);
    assert_eq!(SCORE_TIER4, 0.4);
    assert_eq!(SCORE_UNKNOWN, 0.2);
}
