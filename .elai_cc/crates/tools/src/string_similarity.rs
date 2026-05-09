//! B-20A Task 6 — hand-rolled Levenshtein + similarity ratio.
//!
//! Decision C (workflow/reasoning/B-20A-decisions.md): no new dependency.
//! `strsim`, `levenshtein`, and `similar` are all explicitly rejected;
//! the project's Anti-Slopsquatting + "no new deps without confirmation"
//! rules in CLAUDE.md mandate hand-rolling this primitive.
//!
//! Implementation: classic O(n*m) DP with a 2D `Vec<Vec<usize>>` matrix
//! (no `HashMap`, no `unsafe`, no allocations beyond the matrix itself).
//! Operates on Unicode chars (collected once into `Vec<char>`) so the
//! ratio behaves intuitively for non-ASCII source code (CJK identifiers,
//! emoji in test fixtures).
//!
//! Threshold contract: callers should emit a hint only when
//! `similarity_ratio(...) > 0.5`. The dispatcher's `dispatch_edit`
//! enforces this on the `EditMatchTier::NotMatched` arm.

/// Compute Levenshtein edit distance between `a` and `b`.
///
/// Returns the minimum number of single-character insertions, deletions,
/// or substitutions required to transform `a` into `b`. O(n*m) time,
/// O(n*m) space. Both empty strings ⇒ 0. Either empty ⇒ length of the
/// other.
pub fn levenshtein(a: &str, b: &str) -> usize {
    if a == b {
        return 0;
    }
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n == 0 {
        return m;
    }
    if m == 0 {
        return n;
    }
    // dp[i][j] = distance between a[..i] and b[..j].
    let mut dp: Vec<Vec<usize>> = vec![vec![0; m + 1]; n + 1];
    for i in 0..=n {
        dp[i][0] = i;
    }
    for j in 0..=m {
        dp[0][j] = j;
    }
    for i in 1..=n {
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            let deletion = dp[i - 1][j] + 1;
            let insertion = dp[i][j - 1] + 1;
            let substitution = dp[i - 1][j - 1] + cost;
            let mut best = deletion;
            if insertion < best {
                best = insertion;
            }
            if substitution < best {
                best = substitution;
            }
            dp[i][j] = best;
        }
    }
    dp[n][m]
}

/// Similarity ratio in `[0.0, 1.0]`. Defined as `1 - distance / max_len`.
/// Both empty strings ⇒ `1.0` (vacuous match). Either empty + the other
/// non-empty ⇒ `0.0`.
pub fn similarity_ratio(a: &str, b: &str) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let dist = levenshtein(a, b);
    // NARROWING-OK: usize->f32 precision loss above ~16M chars; acceptable for similarity ratio display.
    let max_len = a.chars().count().max(b.chars().count()) as f32;
    if max_len == 0.0 {
        return 1.0;
    }
    // NARROWING-OK: usize->f32; levenshtein distance fits display precision.
    1.0 - (dist as f32 / max_len)
}

/// Threshold above which a similarity hint is worth emitting. Below this,
/// the nearest line is too far from `old_text` to help the agent.
pub const SIMILARITY_HINT_THRESHOLD: f32 = 0.5;

/// Find the line in `content` with the highest `similarity_ratio` against
/// `query`. Returns `None` when the file is empty or the best ratio falls
/// at-or-below `SIMILARITY_HINT_THRESHOLD`. Line numbers are 1-based to
/// match editor convention.
pub fn best_line_match(content: &str, query: &str) -> Option<(String, usize, f32)> {
    if content.is_empty() || query.is_empty() {
        return None;
    }
    // Compare against full lines; a line-level granularity matches the
    // "did you mean line 84-95?" UX described in the B-20 plan body.
    let mut best: Option<(String, usize, f32)> = None;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let ratio = similarity_ratio(query, trimmed);
        match &best {
            None => {
                best = Some((trimmed.to_string(), idx + 1, ratio));
            }
            Some((_, _, prev)) if ratio > *prev => {
                best = Some((trimmed.to_string(), idx + 1, ratio));
            }
            _ => {}
        }
    }
    match best {
        Some((text, line_no, ratio)) if ratio > SIMILARITY_HINT_THRESHOLD => {
            Some((text, line_no, ratio))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_canonical_examples() {
        assert_eq!(levenshtein("kitten", "sitting"), 3);
        assert_eq!(levenshtein("flaw", "lawn"), 2);
        assert_eq!(levenshtein("", ""), 0);
        assert_eq!(levenshtein("abc", ""), 3);
        assert_eq!(levenshtein("", "abc"), 3);
        assert_eq!(levenshtein("same", "same"), 0);
    }

    #[test]
    fn ratio_bounded_zero_to_one() {
        assert!((similarity_ratio("hi", "hi") - 1.0).abs() < f32::EPSILON);
        let r = similarity_ratio("hello", "world");
        assert!((0.0..=1.0).contains(&r));
    }

    #[test]
    fn best_line_match_picks_closest() {
        let content = "fn alpha() {}\nfn beta(x: u32) -> u32 { x + 1 }\nfn gamma() {}\n";
        let query = "fn beta(x: u32) -> u64 { x + 2 }";
        let (text, line_no, ratio) =
            best_line_match(content, query).expect("must hit threshold");
        assert!(text.contains("beta"));
        assert_eq!(line_no, 2);
        assert!(ratio > SIMILARITY_HINT_THRESHOLD);
    }

    #[test]
    fn best_line_match_below_threshold_returns_none() {
        let content = "alpha\nbeta\ngamma\n";
        let query = "qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq";
        assert!(best_line_match(content, query).is_none());
    }
}
