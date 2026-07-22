// J-41 T1 — ExtractiveGroundingPass: deterministic three-tier extractive
// fingerprint that flags zero-span claims (answer sentences with no
// extractive support from any evidence span). OBSERVE-ONLY in T1.

use crate::held_out::{normalize_text, ngram_filter, jaccard_overlap};

const STOPWORDS: &[&str] = &[
    "that", "this", "with", "from", "have", "been",
    "will", "would", "could", "which", "there", "their",
    "they", "them", "then", "than", "these", "those",
];

#[derive(Debug, Clone)]
pub struct ZeroSpanClaim {
    pub claim_text: String,
    pub claim_index: usize,
}

#[derive(Debug, Clone)]
pub struct ExtractiveGroundingResult {
    pub total_claims: usize,
    pub grounded_claims: usize,
    pub zero_span_claims: Vec<ZeroSpanClaim>,
    pub tier1_exact_matches: usize,
    pub tier2_fuzzy_matches: usize,
    pub tier3_keyword_matches: usize,
}

pub fn run_extractive_grounding(
    answer_text: &str,
    evidence_spans: &[&str],
) -> ExtractiveGroundingResult {
    let claims = split_sentences(answer_text);
    let total_claims = claims.len();

    let normalized_evidence: Vec<String> = evidence_spans
        .iter()
        .map(|s| normalize_text(s))
        .collect();

    let evidence_5grams: Vec<Vec<u64>> = evidence_spans
        .iter()
        .map(|s| ngram_filter(s, 5))
        .collect();

    let mut zero_span_claims = Vec::new();
    let mut tier1_exact_matches = 0;
    let mut tier2_fuzzy_matches = 0;
    let mut tier3_keyword_matches = 0;

    for (idx, claim) in claims.iter().enumerate() {
        let normalized_claim = normalize_text(claim);
        let tokens: Vec<&str> = normalized_claim.split_whitespace().collect();

        if tokens.is_empty() {
            zero_span_claims.push(ZeroSpanClaim {
                claim_text: claim.to_owned(),
                claim_index: idx,
            });
            continue;
        }

        let mut grounded = false;

        // Tier 1: exact 8-gram substring match.
        if tokens.len() >= 8 {
            for window in tokens.windows(8) {
                let window_str = window.join(" ");
                if normalized_evidence.iter().any(|e| e.contains(&window_str)) {
                    tier1_exact_matches += 1;
                    grounded = true;
                    break;
                }
            }
        }

        // Tier 2: fuzzy 5-gram jaccard overlap.
        if !grounded {
            let claim_5grams = ngram_filter(claim, 5);
            if !claim_5grams.is_empty() {
                for ev_sig in &evidence_5grams {
                    if !ev_sig.is_empty() && jaccard_overlap(&claim_5grams, ev_sig) >= 0.8 {
                        tier2_fuzzy_matches += 1;
                        grounded = true;
                        break;
                    }
                }
            }
        }

        // Tier 3: keyword proxy (>= 50% content words in any evidence span).
        if !grounded {
            let content_words: Vec<&str> = tokens
                .iter()
                .filter(|t| t.len() >= 4 && !STOPWORDS.contains(t))
                .copied()
                .collect();

            if !content_words.is_empty() {
                for ev in &normalized_evidence {
                    let ev_tokens: std::collections::HashSet<&str> =
                        ev.split_whitespace().collect();
                    let matched = content_words
                        .iter()
                        .filter(|w| ev_tokens.contains(*w))
                        .count();
                    if matched * 2 >= content_words.len() {
                        tier3_keyword_matches += 1;
                        grounded = true;
                        break;
                    }
                }
            }
        }

        if !grounded {
            zero_span_claims.push(ZeroSpanClaim {
                claim_text: claim.to_owned(),
                claim_index: idx,
            });
        }
    }

    let grounded_claims = total_claims - zero_span_claims.len();

    ExtractiveGroundingResult {
        total_claims,
        grounded_claims,
        zero_span_claims,
        tier1_exact_matches,
        tier2_fuzzy_matches,
        tier3_keyword_matches,
    }
}

fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);
        if ch == '.' || ch == '!' || ch == '?' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_owned());
            }
            current.clear();
        }
    }

    let leftover = current.trim();
    if !leftover.is_empty() {
        sentences.push(leftover.to_owned());
    }

    sentences
}
