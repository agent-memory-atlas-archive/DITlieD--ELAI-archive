// J-12 H5: task-start top-1 skill retrieval (BM25). Pure, LLM-free, I/O-free.
// Caller (drive_task) builds the SkillDoc corpus from F-6 SkillStore after
// tier filtering (S2+ retrievable, S1 floor excluded). This module never
// imports SkillStore / SkillTrustTier so the retriever stays pure and
// unit-testable in isolation. SkillRetrieved SSB + prompt injection are
// later residuals; this call keeps the pure retriever production-reachable
// (D-030).

/// One BM25 corpus document (caller builds from F-6 SkillStore after tier filter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillDoc {
    pub id: String,
    /// Retrieval text (use LearnedSkill.description).
    pub text: String,
}

/// Top-1 retrieval result when score >= min_score.
#[derive(Debug, Clone, PartialEq)]
pub struct SkillRetrieval {
    pub id: String,
    pub score: f64,
    pub text: String,
}

pub const BM25_K1: f64 = 1.5;
pub const BM25_B: f64 = 0.75;
/// Default min score floor. Scores below this are discarded (no hit).
pub const DEFAULT_MIN_SCORE: f64 = 0.0;

/// Lowercase + split on non-alphanumeric, drop empty tokens.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// BM25 top-1 retrieval over `docs` for `query`.
/// - Empty docs OR empty query (trim) => None
/// - Tokenize: lowercase, split on non-alphanumeric, drop empty tokens
/// - Smoothed IDF: ln(1 + (N - df + 0.5) / (df + 0.5)) with N = corpus size
/// - BM25 with K1=1.5 B=0.75; avgdl over corpus (empty corpus already returned)
/// - HARD CAP top-1 only (never return multi-hit). On score ties, prefer lower
///   `id` (byte order) for determinism.
/// - Return None if best score < min_score
/// - Never panics; pure; no I/O; no LLM
pub fn retrieve_top1_skill(
    query: &str,
    docs: &[SkillDoc],
    min_score: f64,
) -> Option<SkillRetrieval> {
    if docs.is_empty() {
        return None;
    }
    let q_tokens = tokenize(query);
    if q_tokens.is_empty() {
        return None;
    }

    let n = docs.len() as f64;
    let doc_tokens: Vec<Vec<String>> = docs.iter().map(|d| tokenize(&d.text)).collect();
    let avgdl: f64 = {
        let total: usize = doc_tokens.iter().map(|t| t.len()).sum();
        if total == 0 {
            return None;
        }
        total as f64 / n
    };

    // Document frequency per query term (df = docs containing the term).
    let mut df: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for q in &q_tokens {
        let count = doc_tokens
            .iter()
            .filter(|dt| dt.iter().any(|t| t == q))
            .count();
        df.insert(q.as_str(), count);
    }

    // Score each doc; track top-1 with lower-id tiebreak.
    let mut best: Option<(usize, f64)> = None;
    for (i, dt) in doc_tokens.iter().enumerate() {
        let dl = dt.len() as f64;
        let mut score = 0.0_f64;
        // Term frequency in this doc.
        let mut tf: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
        for t in dt {
            *tf.entry(t.as_str()).or_insert(0) += 1;
        }
        for q in &q_tokens {
            let f = *tf.get(q.as_str()).unwrap_or(&0) as f64;
            if f == 0.0 {
                continue;
            }
            let d = *df.get(q.as_str()).unwrap_or(&0) as f64;
            let idf = ((n - d + 0.5) / (d + 0.5)).ln_1p();
            let denom = f + BM25_K1 * (1.0 - BM25_B + BM25_B * (dl / avgdl));
            score += idf * (f * (BM25_K1 + 1.0)) / denom;
        }
        match best {
            None => best = Some((i, score)),
            Some((bi, bs)) => {
                // Higher score wins; on tie, lower id (byte order) wins.
                let id_i = docs[i].id.as_bytes();
                let id_bi = docs[bi].id.as_bytes();
                let lower_id_is_i = id_i < id_bi;
                if score > bs || (score == bs && lower_id_is_i) {
                    best = Some((i, score));
                }
            }
        }
    }

    let (idx, score) = best?;
    if score < min_score {
        return None;
    }
    let doc = &docs[idx];
    Some(SkillRetrieval {
        id: doc.id.clone(),
        score,
        text: doc.text.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str, text: &str) -> SkillDoc {
        SkillDoc {
            id: id.to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn empty_docs_none() {
        assert!(retrieve_top1_skill("anything", &[], DEFAULT_MIN_SCORE).is_none());
    }

    #[test]
    fn empty_query_none() {
        let docs = vec![doc("a", "some text here")];
        assert!(retrieve_top1_skill("", &docs, DEFAULT_MIN_SCORE).is_none());
        assert!(retrieve_top1_skill("   ", &docs, DEFAULT_MIN_SCORE).is_none());
    }

    #[test]
    fn single_exact_match_returns_id_with_positive_score() {
        let docs = vec![doc("a", "rust async runtime tokio")];
        let r = retrieve_top1_skill("tokio", &docs, DEFAULT_MIN_SCORE).unwrap();
        assert_eq!(r.id, "a");
        assert!(r.score > 0.0);
    }

    #[test]
    fn top1_hard_cap_prefers_higher_score() {
        let docs = vec![
            doc("a", "rust async runtime tokio"),
            doc("b", "python django web framework"),
        ];
        let r = retrieve_top1_skill("tokio", &docs, DEFAULT_MIN_SCORE).unwrap();
        assert_eq!(r.id, "a");
    }

    #[test]
    fn below_min_score_none() {
        let docs = vec![doc("a", "rust async runtime tokio")];
        assert!(retrieve_top1_skill("tokio", &docs, 1000.0).is_none());
    }

    #[test]
    fn tie_breaks_by_lower_id() {
        let docs = vec![
            doc("z", "rust async runtime tokio"),
            doc("a", "rust async runtime tokio"),
        ];
        let r = retrieve_top1_skill("tokio", &docs, DEFAULT_MIN_SCORE).unwrap();
        assert_eq!(r.id, "a");
    }

    #[test]
    fn case_insensitive_tokenize() {
        let docs = vec![doc("a", "Rust ASYNC Runtime TOKIO")];
        let r = retrieve_top1_skill("tokio", &docs, DEFAULT_MIN_SCORE).unwrap();
        assert_eq!(r.id, "a");
        assert!(r.score > 0.0);
    }
}
