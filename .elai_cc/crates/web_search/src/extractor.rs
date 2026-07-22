use tracing::debug;

use crate::WebSearchError;

/// Page classification after extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PageType {
    /// Page has meaningful text content.
    Article,
    /// Page has no extractable text (JS-only, empty body, etc.).
    Empty,
}

/// Extracted content from a web page.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractedContent {
    pub title: String,
    pub text: String,
    pub word_count: usize,
    pub page_type: PageType,
    pub extracted_at: u64,
}

/// Trait for content extraction strategies. Object-safe.
/// Signature locked for 4D.17 WebResearchOrchestrator consumption.
pub trait ContentExtractor: Send + Sync {
    fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent, WebSearchError>;
}


/// Trait for query-aware content extraction. Object-safe sibling trait.
/// The extractor receives the research query and returns a compressed
/// ExtractedContent containing only the most query-relevant verbatim spans.
pub trait QueryAwareExtractor: Send + Sync {
    fn extract_query_aware(
        &self,
        html: &str,
        url: &str,
        query: &str,
    ) -> Result<ExtractedContent, WebSearchError>;
}

/// Maximum number of verbatim sentences kept by the query-aware extractor.
pub const SPAN_BUDGET_SENTENCES: usize = 5;

/// Basic content extractor: strips script/style/nav/header/footer/head elements,
/// extracts text from remaining HTML, collapses whitespace.
pub struct BasicExtractor {
    /// Epoch ms provider — stored at construction, used for extracted_at.
    extracted_at_ms: u64,
}

impl BasicExtractor {
    pub fn new(extracted_at_ms: u64) -> Self {
        Self { extracted_at_ms }
    }
}

/// Tags to remove entirely (content + tag).
const STRIP_TAGS: &[&str] = &["script", "style", "nav", "header", "footer", "noscript", "head"];

/// Case-insensitive ASCII search for a pattern in a string.
/// Returns the byte offset of the first occurrence, or None.
/// Safe: only compares ASCII bytes, no locale-dependent case folding.
fn find_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    let needle_bytes = needle.as_bytes();
    let haystack_bytes = haystack.as_bytes();
    if needle_bytes.len() > haystack_bytes.len() {
        return None;
    }
    let limit = haystack_bytes.len() - needle_bytes.len() + 1;
    'outer: for i in 0..limit {
        for (j, &nb) in needle_bytes.iter().enumerate() {
            if haystack_bytes[i + j].to_ascii_lowercase() != nb.to_ascii_lowercase() {
                continue 'outer;
            }
        }
        return Some(i);
    }
    None
}

/// Like find_ascii_ci but searches starting from a byte offset.
fn find_ascii_ci_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    find_ascii_ci(&haystack[from..], needle).map(|pos| pos + from)
}

/// Remove content between opening and closing tags for the given tag names.
/// Uses ASCII case-insensitive search — safe for all Unicode content since
/// HTML tag names are always ASCII.
fn strip_elements(html: &str, tags: &[&str]) -> String {
    let mut result = html.to_owned();
    for tag in tags {
        let open_pattern = format!("<{}", tag);
        let close_pattern = format!("</{}>", tag);
        loop {
            let start = find_ascii_ci(&result, &open_pattern);
            let Some(start_idx) = start else { break };
            let end = find_ascii_ci_from(&result, &close_pattern, start_idx);
            match end {
                Some(end_idx) => {
                    let remove_end = end_idx + close_pattern.len();
                    result.replace_range(start_idx..remove_end, "");
                }
                None => {
                    // Unclosed tag — remove from open tag to end.
                    result.truncate(start_idx);
                    break;
                }
            }
        }
    }
    result
}

/// Extract visible text from HTML by stripping all tags and collapsing whitespace.
fn extract_visible_text(html: &str) -> String {
    let stripped = crate::strip_html(html);
    let mut result = String::with_capacity(stripped.len());
    let mut last_was_space = true;
    for ch in stripped.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }
    result.trim().to_owned()
}

/// Extract title from HTML <title> tag using ASCII case-insensitive search.
/// Safe for Unicode content — tag names are ASCII, content is extracted by
/// byte offsets from the original string (no lowercase remapping).
fn extract_title(html: &str) -> String {
    let start = find_ascii_ci(html, "<title");
    let Some(start_idx) = start else {
        return String::new();
    };
    // Find end of opening tag '>' after "<title".
    let tag_close = html[start_idx..].find('>');
    let Some(tag_close_offset) = tag_close else {
        return String::new();
    };
    let content_start = start_idx + tag_close_offset + 1;
    let close = find_ascii_ci_from(html, "</title>", content_start);
    let Some(close_idx) = close else {
        return String::new();
    };
    let title = &html[content_start..close_idx];
    crate::strip_html(title).trim().to_owned()
}

impl ContentExtractor for BasicExtractor {
    fn extract(&self, html: &str, _url: &str) -> Result<ExtractedContent, WebSearchError> {
        // Step 1: Remove non-content elements.
        let cleaned = strip_elements(html, STRIP_TAGS);
        // Step 2: Extract title from original (before head is stripped).
        let title = extract_title(html);
        // Step 3: Extract visible text.
        let text = extract_visible_text(&cleaned);
        // Step 4: Count words.
        let word_count = if text.is_empty() {
            0
        } else {
            text.split_whitespace().count()
        };
        // Step 5: Classify page type.
        let page_type = if word_count == 0 {
            PageType::Empty
        } else {
            PageType::Article
        };

        debug!(
            title_len = title.len(),
            word_count,
            page_type = ?page_type,
            "content extraction complete"
        );

        Ok(ExtractedContent {
            title,
            text,
            word_count,
            page_type,
            extracted_at: self.extracted_at_ms,
        })
    }
}


/// Split text into sentences on ., !, ? followed by whitespace or end-of-string.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let ch = chars[i];
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let next = chars.get(i + 1).copied();
            if next.is_none() || next.is_some_and(|c| c.is_whitespace()) {
                let trimmed = current.trim().to_owned();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
        i += 1;
    }
    let trimmed = current.trim().to_owned();
    if !trimmed.is_empty() {
        sentences.push(trimmed);
    }
    sentences
}

/// Query-aware extractor: implements both ContentExtractor (delegating to
/// BasicExtractor for the no-query path) and QueryAwareExtractor (scoring
/// per-sentence keyword overlap with the query, keeping top N verbatim spans).
pub struct QueryAwareBasicExtractor {
    inner: BasicExtractor,
}

impl QueryAwareBasicExtractor {
    pub fn new(extracted_at_ms: u64) -> Self {
        Self {
            inner: BasicExtractor::new(extracted_at_ms),
        }
    }
}

impl ContentExtractor for QueryAwareBasicExtractor {
    fn extract(&self, html: &str, url: &str) -> Result<ExtractedContent, WebSearchError> {
        self.inner.extract(html, url)
    }
}

impl QueryAwareExtractor for QueryAwareBasicExtractor {
    fn extract_query_aware(
        &self,
        html: &str,
        url: &str,
        query: &str,
    ) -> Result<ExtractedContent, WebSearchError> {
        let extracted = self.inner.extract(html, url)?;
        if extracted.page_type == PageType::Empty {
            return Ok(extracted);
        }

        let sentences = split_sentences(&extracted.text);
        if sentences.is_empty() {
            return Ok(extracted);
        }

        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|t| t.to_ascii_lowercase())
            .collect();

        let mut scored: Vec<(f64, &str)> = sentences
            .iter()
            .map(|s| {
                let sentence_lower = s.to_ascii_lowercase();
                let sentence_char_len = s.chars().count().max(1) as f64;
                let mut score = 0.0_f64;
                for term in &query_terms {
                    let term_len = term.chars().count() as f64;
                    let mut count = 0_usize;
                    let mut search_from = 0_usize;
                    while let Some(pos) = sentence_lower[search_from..].find(term.as_str()) {
                        count += 1;
                        search_from += pos + term.len();
                        if search_from >= sentence_lower.len() {
                            break;
                        }
                    }
                    score += (count as f64) * term_len;
                }
                score /= sentence_char_len.sqrt();
                (score, s.as_str())
            })
            .collect();

        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

        let has_scores = scored.iter().any(|(s, _)| *s > 0.0);
        let selected: Vec<&str> = if has_scores {
            scored
                .into_iter()
                .filter(|(s, _)| *s > 0.0)
                .take(SPAN_BUDGET_SENTENCES)
                .map(|(_, s)| s)
                .collect()
        } else {
            sentences.iter().take(SPAN_BUDGET_SENTENCES).map(|s| s.as_str()).collect()
        };

        let compressed_text = selected.join(" ... ");
        let word_count = compressed_text.split_whitespace().count();

        Ok(ExtractedContent {
            title: extracted.title,
            text: compressed_text,
            word_count,
            page_type: extracted.page_type,
            extracted_at: extracted.extracted_at,
        })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ascii_ci() {
        assert_eq!(find_ascii_ci("Hello World", "hello"), Some(0));
        assert_eq!(find_ascii_ci("Hello World", "WORLD"), Some(6));
        assert_eq!(find_ascii_ci("Hello", "xyz"), None);
    }

    #[test]
    fn test_find_ascii_ci_unicode_safe() {
        // Turkish İ (U+0130) — 2 bytes in UTF-8. Case folding to ASCII 'i'
        // would only match if we were using Unicode folding (which we don't).
        let html = "İstanbul <TITLE>Test</TITLE>";
        let pos = find_ascii_ci(html, "<title");
        assert!(pos.is_some());
        // The offset should correctly point past the İ character.
        let idx = pos.unwrap();
        assert_eq!(&html[idx..idx + 6], "<TITLE");
    }

    #[test]
    fn test_strip_elements_removes_script() {
        let html = "<div>hello</div><script>alert('x')</script><p>world</p>";
        let result = strip_elements(html, STRIP_TAGS);
        assert!(!result.contains("alert"));
        assert!(result.contains("hello"));
        assert!(result.contains("world"));
    }

    #[test]
    fn test_strip_elements_removes_style_and_nav() {
        let html = "<style>body{}</style><nav><a>menu</a></nav><p>content</p>";
        let result = strip_elements(html, STRIP_TAGS);
        assert!(!result.contains("body{}"));
        assert!(!result.contains("menu"));
        assert!(result.contains("content"));
    }

    #[test]
    fn test_strip_elements_case_insensitive() {
        let html = "<SCRIPT>alert('x')</SCRIPT><p>safe</p>";
        let result = strip_elements(html, STRIP_TAGS);
        assert!(!result.contains("alert"));
        assert!(result.contains("safe"));
    }

    #[test]
    fn test_extract_visible_text_collapses_whitespace() {
        let html = "<p>hello   world</p>  <p>foo</p>";
        let text = extract_visible_text(html);
        assert_eq!(text, "hello world foo");
    }

    #[test]
    fn test_extract_title() {
        let html = "<html><head><title>My Page</title></head><body></body></html>";
        assert_eq!(extract_title(html), "My Page");
    }

    #[test]
    fn test_extract_title_case_insensitive() {
        let html = "<html><head><TITLE>Upper</TITLE></head></html>";
        assert_eq!(extract_title(html), "Upper");
    }

    #[test]
    fn test_extract_title_missing() {
        let html = "<html><body>no title</body></html>";
        assert_eq!(extract_title(html), "");
    }

    #[test]
    fn test_extract_title_unicode_content() {
        // Title contains non-ASCII chars — byte offsets must be correct.
        let html = "<html><head><title>Café résumé</title></head></html>";
        assert_eq!(extract_title(html), "Café résumé");
    }

    #[test]
    fn test_basic_extractor_article() {
        let extractor = BasicExtractor::new(1000);
        let html = "<html><head><title>Test</title></head><body><p>Some content here.</p></body></html>";
        let result = extractor.extract(html, "https://example.com").unwrap();
        assert_eq!(result.title, "Test");
        assert_eq!(result.page_type, PageType::Article);
        assert!(result.word_count > 0);
        assert_eq!(result.extracted_at, 1000);
    }

    // Adversarial: JS-only page -> PageType::Empty
    #[test]
    fn test_js_only_page_empty() {
        let extractor = BasicExtractor::new(1000);
        let html = "<html><head><title>App</title></head><body><script>renderApp()</script><noscript>Enable JS</noscript></body></html>";
        let result = extractor.extract(html, "https://example.com").unwrap();
        assert_eq!(result.page_type, PageType::Empty);
        assert_eq!(result.word_count, 0);
    }

    // Adversarial: Non-UTF8 handled via String::from_utf8_lossy upstream (fetcher),
    // but extractor should handle replacement chars gracefully.
    #[test]
    fn test_replacement_chars_in_html() {
        let extractor = BasicExtractor::new(1000);
        let html = "<p>caf\u{FFFD} au lait</p>";
        let result = extractor.extract(html, "https://example.com").unwrap();
        assert_eq!(result.page_type, PageType::Article);
        assert!(result.text.contains("caf"));
    }

    #[test]
    fn test_empty_html() {
        let extractor = BasicExtractor::new(1000);
        let result = extractor.extract("", "https://example.com").unwrap();
        assert_eq!(result.page_type, PageType::Empty);
        assert_eq!(result.word_count, 0);
    }

    // Object-safety compile-time check.
    const _: Option<Box<dyn ContentExtractor>> = None;
    const _: Option<Box<dyn QueryAwareExtractor>> = None;
}
