// J-40 T1 contract tests: QueryAwareExtractor + QueryAwareBasicExtractor.
// Exercises the SAME production symbols wired at
//   task_runner.rs build_orchestrator (QueryAwareBasicExtractor::new + with_query_aware_extractor)
//   web_research_orchestrator.rs fetch_and_extract (extract_query_aware call)
// No reimplemented scoring branch in the test body.

use elai_web_search::extractor::{
    ContentExtractor, ExtractedContent, PageType, QueryAwareBasicExtractor, QueryAwareExtractor,
    SPAN_BUDGET_SENTENCES,
};

const TEST_HTML: &str = concat!(
    "<html><head><title>Test</title></head><body>",
    "<p>Rust is a systems programming language. ",
    "It focuses on memory safety without garbage collection. ",
    "The borrow checker ensures reference validity. ",
    "Rust performance is comparable to C++. ",
    "Cargo is the package manager. ",
    "Macros enable metaprogramming.</p>",
    "</body></html>"
);

#[test]
fn query_aware_extractor_compresses_to_relevant_spans() {
    let extractor = QueryAwareBasicExtractor::new(1000);
    let result: ExtractedContent = extractor
        .extract_query_aware(TEST_HTML, "https://example.com", "Rust memory safety borrow checker")
        .expect("extract_query_aware ok");

    assert_eq!(result.page_type, PageType::Article);
    assert!(
        result.text.contains("memory safety"),
        "compressed text should contain 'memory safety', got: {}",
        result.text
    );
    assert!(
        result.text.contains("borrow checker"),
        "compressed text should contain 'borrow checker', got: {}",
        result.text
    );
    assert!(
        !result.text.contains("Cargo is the package manager"),
        "irrelevant sentence should be filtered out, got: {}",
        result.text
    );

    let full: ExtractedContent = extractor
        .extract(TEST_HTML, "https://example.com")
        .expect("extract ok");
    assert!(
        result.word_count < full.word_count,
        "query-aware word_count {} should be < full word_count {}",
        result.word_count,
        full.word_count
    );
}

#[test]
fn query_aware_extractor_empty_page_returns_empty() {
    let extractor = QueryAwareBasicExtractor::new(1000);
    let result = extractor
        .extract_query_aware("", "https://example.com", "query")
        .expect("extract_query_aware ok on empty");

    assert_eq!(result.page_type, PageType::Empty);
    assert_eq!(result.word_count, 0);
}

#[test]
fn query_aware_extractor_no_query_overlap_returns_first_n() {
    let html = concat!(
        "<html><head><title>Cooking</title></head><body>",
        "<p>Baking bread requires flour and yeast. ",
        "The oven should be preheated to 200 degrees. ",
        "Knead the dough for ten minutes. ",
        "Let it rise in a warm place. ",
        "Bake for thirty minutes until golden.</p>",
        "</body></html>"
    );
    let extractor = QueryAwareBasicExtractor::new(1000);
    let result = extractor
        .extract_query_aware(html, "https://example.com", "quantum physics string theory")
        .expect("extract_query_aware ok");

    assert_eq!(result.page_type, PageType::Article);
    assert!(
        !result.text.is_empty(),
        "fallback should return first N sentences, got empty text"
    );
    assert!(
        result.word_count <= SPAN_BUDGET_SENTENCES * 20,
        "word_count {} should be <= {} (rough upper bound for {} sentences)",
        result.word_count,
        SPAN_BUDGET_SENTENCES * 20,
        SPAN_BUDGET_SENTENCES
    );
}
