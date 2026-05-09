// 4D.14: WebContentIndex — web-content A-RAG with sentence-level indexing.
//
// Provides keyword search, semantic search (L2-normalized cosine via dot product),
// and chunk_read (progressive disclosure) over ingested web content.

use rustc_hash::FxHashMap;

use elai_core::source_tier::SourceTier;

use crate::arag::{ContextTracker, RetrievalResult};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Target chunk size in tokens (spec: ~500 tokens).
const CHUNK_SIZE_TOKENS: usize = 500;

/// Minimum sentence length in characters to avoid trivial splits.
const MIN_SENTENCE_LEN: usize = 10;

/// Oversampling factor for semantic search: retrieve N*top_k sentences,
/// then group by chunk and max-pool scores.
const OVERSAMPLE_FACTOR: usize = 3;

// ---------------------------------------------------------------------------
// WebChunk
// ---------------------------------------------------------------------------

/// A chunk of web content with sentence-level decomposition.
#[derive(Debug, Clone)]
pub struct WebChunk {
    pub chunk_id: String,
    pub source_url: String,
    pub source_tier: SourceTier,
    pub text: String,
    sentences: Vec<String>,
    pub word_count: usize,
}

impl WebChunk {
    pub fn new(chunk_id: String, source_url: String, source_tier: SourceTier, text: String) -> Self {
        let sentences = split_sentences(&text);
        let word_count = text.split_whitespace().count();
        Self {
            chunk_id,
            source_url,
            source_tier,
            text,
            sentences,
            word_count,
        }
    }

    /// Returns the sentences extracted from this chunk's text.
    pub fn sentences(&self) -> &[String] {
        &self.sentences
    }
}

// ---------------------------------------------------------------------------
// WebAragConfig
// ---------------------------------------------------------------------------

/// Configuration for web content A-RAG indexing.
#[derive(Debug, Clone)]
pub struct WebAragConfig {
    pub top_k: usize,
    pub chunk_size_tokens: usize,
}

impl Default for WebAragConfig {
    fn default() -> Self {
        Self {
            top_k: 5,
            chunk_size_tokens: CHUNK_SIZE_TOKENS,
        }
    }
}

// ---------------------------------------------------------------------------
// WebContentIndex
// ---------------------------------------------------------------------------

/// In-memory index for web content with sentence-level retrieval.
#[derive(Debug)]
pub struct WebContentIndex {
    chunks: FxHashMap<String, WebChunk>,
    /// All sentences across all chunks, in ingestion order.
    sentences: Vec<String>,
    /// Maps each sentence index to its parent chunk_id.
    sentence_to_chunk: Vec<String>,
    /// Optional embeddings, one per sentence. Set externally after ingestion.
    embeddings: Option<Vec<Vec<f32>>>,
}

impl WebContentIndex {
    pub fn new(_config: WebAragConfig) -> Self {
        Self {
            chunks: FxHashMap::default(),
            sentences: Vec::new(),
            sentence_to_chunk: Vec::new(),
            embeddings: None,
        }
    }

    /// Ingest a web chunk, decomposing it into sentences for indexing.
    pub fn ingest(&mut self, chunk: WebChunk) {
        let chunk_id = chunk.chunk_id.clone();
        for sentence in &chunk.sentences {
            self.sentences.push(sentence.clone());
            self.sentence_to_chunk.push(chunk_id.clone());
        }
        self.chunks.insert(chunk_id, chunk);
    }

    /// Set sentence embeddings (one per sentence, in ingestion order).
    /// Returns `false` and logs a warning if the embedding count does not
    /// match the sentence count — partial results will occur in semantic_search.
    pub fn set_sentence_embeddings(&mut self, embeddings: Vec<Vec<f32>>) -> bool {
        let matched = embeddings.len() == self.sentences.len();
        if !matched {
            tracing::warn!(
                embeddings = embeddings.len(),
                sentences = self.sentences.len(),
                "embedding count mismatch — semantic search will produce incomplete results"
            );
        }
        self.embeddings = Some(embeddings);
        matched
    }

    /// Keyword search: score each chunk by length-normalized keyword frequency.
    /// Formula: score = count(kw, text) * len(kw) / sqrt(word_count)
    pub fn keyword_search(&self, query: &str, top_k: usize) -> Vec<RetrievalResult> {
        let keywords: Vec<&str> = query.split_whitespace().collect();
        if keywords.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(String, f64)> = Vec::new();

        for chunk in self.chunks.values() {
            let mut total_score = 0.0_f64;
            let words: Vec<&str> = chunk.text.split_whitespace().collect();
            let wc = words.len();
            if wc == 0 {
                continue;
            }
            let sqrt_wc = (wc as f64).sqrt();

            for kw in &keywords {
                let kw_len = kw.len();
                let count = words
                    .iter()
                    .filter(|w| {
                        let trimmed = w.trim_end_matches(|c: char| c.is_ascii_punctuation());
                        trimmed.eq_ignore_ascii_case(kw)
                    })
                    .count();
                if count > 0 {
                    total_score += (count as f64) * (kw_len as f64) / sqrt_wc;
                }
            }

            if total_score > 0.0 {
                scored.push((chunk.chunk_id.clone(), total_score));
            }
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);

        scored
            .into_iter()
            .map(|(chunk_id, score)| RetrievalResult {
                chunk_id,
                snippet: String::new(),
                score,
                tool_used: "keyword_search".to_string(),
            })
            .collect()
    }

    /// Semantic search: L2-normalize embeddings, dot product, 3x oversampling,
    /// group by chunk, max-pool scores.
    pub fn semantic_search(
        &self,
        _query: &str,
        query_embedding: &[f32],
        top_k: usize,
    ) -> Vec<RetrievalResult> {
        let embeddings = match &self.embeddings {
            Some(e) => e,
            None => return self.keyword_search(_query, top_k),
        };

        if embeddings.is_empty() || self.sentences.is_empty() {
            return Vec::new();
        }

        let q_norm = l2_normalize(query_embedding);

        // Score each sentence.
        let mut sentence_scores: Vec<(usize, f64)> = embeddings
            .iter()
            .enumerate()
            .map(|(i, emb)| {
                let e_norm = l2_normalize(emb);
                let dot: f64 = q_norm
                    .iter()
                    .zip(e_norm.iter())
                    .map(|(a, b)| (*a as f64) * (*b as f64))
                    .sum();
                (i, dot)
            })
            .collect();

        // Sort descending by score.
        sentence_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // 3x oversampling: take top_k * OVERSAMPLE_FACTOR sentences.
        let oversample_count = top_k * OVERSAMPLE_FACTOR;
        sentence_scores.truncate(oversample_count);

        // Group by chunk, max-pool scores.
        let mut chunk_max: FxHashMap<&str, f64> = FxHashMap::default();
        for (idx, score) in &sentence_scores {
            if *idx < self.sentence_to_chunk.len() {
                let cid = &self.sentence_to_chunk[*idx];
                let entry = chunk_max.entry(cid.as_str()).or_insert(0.0);
                if *score > *entry {
                    *entry = *score;
                }
            }
        }

        // Sort chunks by max score, take top_k.
        let mut chunk_results: Vec<(&str, f64)> = chunk_max.into_iter().collect();
        chunk_results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        chunk_results.truncate(top_k);

        chunk_results
            .into_iter()
            .map(|(chunk_id, score)| RetrievalResult {
                chunk_id: chunk_id.to_string(),
                snippet: String::new(),
                score,
                tool_used: "semantic_search".to_string(),
            })
            .collect()
    }

    /// Progressive disclosure: return full text for requested chunk_ids.
    /// Marks chunks as read in the ContextTracker.
    /// Unknown chunk_ids are silently skipped (return empty for unknowns).
    pub fn chunk_read(
        &self,
        chunk_ids: Vec<String>,
        tracker: &mut ContextTracker,
    ) -> Vec<RetrievalResult> {
        let mut results = Vec::new();
        for chunk_id in chunk_ids {
            if let Some(chunk) = self.chunks.get(&chunk_id) {
                tracker.mark_read(chunk_id.clone());
                results.push(RetrievalResult {
                    chunk_id,
                    snippet: chunk.text.clone(),
                    score: 1.0,
                    tool_used: "chunk_read".to_string(),
                });
            }
            // Unknown chunk_ids: silently skip (spec: return empty result).
        }
        results
    }
}

// ---------------------------------------------------------------------------
// Sentence splitting with URL guard
// ---------------------------------------------------------------------------

/// Split text into sentences at [.!?\n]+ boundaries, with URL guard.
/// Uses char_indices for UTF-8 safety. Skips split points inside http(s):// URLs.
fn split_sentences(text: &str) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let mut sentences = Vec::new();
    let mut start = 0;

    // Find URL spans to protect from splitting.
    let url_spans = find_url_spans(text);

    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let (byte_pos, ch) = chars[i];

        if ch == '.' || ch == '!' || ch == '?' || ch == '\n' {
            // Check if this split point is inside a URL span.
            if is_inside_url(byte_pos, &url_spans) {
                i += 1;
                continue;
            }

            // Consume consecutive sentence-ending chars.
            let mut end_byte = byte_pos + ch.len_utf8();
            let mut j = i + 1;
            while j < len {
                let (_, next_ch) = chars[j];
                if next_ch == '.' || next_ch == '!' || next_ch == '?' || next_ch == '\n' || next_ch == ' ' {
                    end_byte = chars[j].0 + next_ch.len_utf8();
                    j += 1;
                } else {
                    break;
                }
            }

            let sentence = text[start..end_byte].trim();
            if sentence.len() >= MIN_SENTENCE_LEN {
                sentences.push(sentence.to_string());
            }
            start = end_byte;
            i = j;
        } else {
            i += 1;
        }
    }

    // Remaining text.
    if start < text.len() {
        let remainder = text[start..].trim();
        if remainder.len() >= MIN_SENTENCE_LEN {
            sentences.push(remainder.to_string());
        }
    }

    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim().to_string());
    }

    sentences
}

/// Find byte ranges of http(s):// URLs in text.
fn find_url_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        // Look for "http://" or "https://".
        if i + 7 <= len && &bytes[i..i + 7] == b"http://" {
            let url_start = i;
            i += 7;
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            spans.push((url_start, i));
        } else if i + 8 <= len && &bytes[i..i + 8] == b"https://" {
            let url_start = i;
            i += 8;
            while i < len && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            spans.push((url_start, i));
        } else {
            i += 1;
        }
    }

    spans
}

/// Check if a byte position falls inside any URL span.
fn is_inside_url(byte_pos: usize, url_spans: &[(usize, usize)]) -> bool {
    url_spans.iter().any(|(start, end)| byte_pos >= *start && byte_pos < *end)
}

/// L2-normalize a vector. Returns zero vector if magnitude is zero.
fn l2_normalize(v: &[f32]) -> Vec<f32> {
    let mag: f64 = v.iter().map(|x| (*x as f64) * (*x as f64)).sum();
    let mag = mag.sqrt();
    if mag < 1e-12 {
        return vec![0.0; v.len()];
    }
    // NARROWING-OK: intentional f64->f32 downcast; f64 used only for intermediate precision, output is f32 embedding.
    v.iter().map(|x| (*x as f64 / mag) as f32).collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sentence_split_basic() {
        let text = "Hello world. This is a test sentence. Another one here!";
        let sents = split_sentences(text);
        assert_eq!(sents.len(), 3);
    }

    #[test]
    fn sentence_split_url_no_break() {
        let text = "See https://example.com/foo.bar.baz for details. End sentence here.";
        let sents = split_sentences(text);
        assert!(
            sents[0].contains("https://example.com/foo.bar.baz"),
            "URL should not be split: {:?}",
            sents
        );
    }

    #[test]
    fn sentence_split_min_length_filter() {
        let text = "Hi. This is a longer sentence that passes the filter.";
        let sents = split_sentences(text);
        // "Hi." is < MIN_SENTENCE_LEN, should be filtered.
        assert!(sents.iter().all(|s| s.len() >= MIN_SENTENCE_LEN));
    }

    #[test]
    fn l2_normalize_basic() {
        let v = vec![3.0_f32, 4.0];
        let n = l2_normalize(&v);
        let mag: f64 = n.iter().map(|x| (*x as f64) * (*x as f64)).sum();
        assert!((mag.sqrt() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn l2_normalize_zero() {
        let v = vec![0.0_f32, 0.0];
        let n = l2_normalize(&v);
        assert!(n.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn keyword_score_multi_keyword() {
        let mut index = WebContentIndex::new(WebAragConfig::default());
        let chunk = WebChunk::new(
            "c1".to_string(),
            "https://example.com".to_string(),
            SourceTier::T2Strong,
            "Rust is great and Rust is fast for systems programming in Rust.".to_string(),
        );
        index.ingest(chunk);

        let results = index.keyword_search("rust systems", 5);
        assert_eq!(results.len(), 1);
        // "rust" appears 3 times (4 chars), "systems" appears 1 time (7 chars)
        // 12 words total (split_whitespace includes "Rust." as one token)
        // score = (3*4 + 1*7) / sqrt(12) = 19 / 3.464... ≈ 5.485
        let expected = 19.0_f64 / (12.0_f64).sqrt();
        assert!(
            (results[0].score - expected).abs() < 0.1,
            "got {}, expected {}",
            results[0].score,
            expected
        );
    }

    #[test]
    fn ingest_updates_sentences_and_mapping() {
        let mut index = WebContentIndex::new(WebAragConfig::default());
        let chunk = WebChunk::new(
            "c1".to_string(),
            "https://example.com".to_string(),
            SourceTier::T3Weak,
            "First sentence here. Second sentence here.".to_string(),
        );
        let sentence_count = chunk.sentences().len();
        index.ingest(chunk);

        assert_eq!(index.sentences.len(), sentence_count);
        assert!(index.sentence_to_chunk.iter().all(|c| c == "c1"));
    }
}
