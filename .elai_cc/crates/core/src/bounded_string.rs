//! Token-bounded string newtype for context telescope fields.
//!
//! `BoundedString<N>` wraps a `String` and rejects inputs exceeding N tokens
//! at construction time. Token counting is pluggable via the `TokenCounter` trait.

use std::fmt;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Token counting
// ---------------------------------------------------------------------------

/// Trait for counting tokens in a text string.
///
/// Implementations may use whitespace splitting, char/4 heuristics,
/// or actual tokenizer vocabularies.
pub trait TokenCounter {
    fn count(&self, text: &str) -> usize;
}

/// Simple whitespace-split token counter.
///
/// Counts tokens as whitespace-delimited words. Empty strings yield 0 tokens.
pub struct WordTokenCounter;

impl TokenCounter for WordTokenCounter {
    fn count(&self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }
        text.split_whitespace().count()
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Returned when a string exceeds the token limit N.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooLong {
    pub actual: usize,
    pub limit: usize,
}

impl fmt::Display for TooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "string has {} tokens, exceeds limit of {}",
            self.actual, self.limit
        )
    }
}

impl std::error::Error for TooLong {}

// ---------------------------------------------------------------------------
// BoundedString<N>
// ---------------------------------------------------------------------------

/// A string guaranteed to contain at most N tokens (as measured at construction).
///
/// Serializes as a plain string. Deserialization trusts stored data and does
/// not re-check the token count (avoids needing a TokenCounter at deser time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BoundedString<const N: usize>(String);

impl<const N: usize> BoundedString<N> {
    /// Create a new bounded string, checking that `s` has at most N tokens.
    pub fn new(s: &str, counter: &dyn TokenCounter) -> Result<Self, TooLong> {
        let count = counter.count(s);
        if count > N {
            return Err(TooLong {
                actual: count,
                limit: N,
            });
        }
        Ok(Self(s.to_string()))
    }

    /// Returns the maximum token count for this bounded string type.
    pub const fn max_tokens() -> usize {
        N
    }

    /// Returns the inner string as a slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> fmt::Display for BoundedString<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<const N: usize> AsRef<str> for BoundedString<N> {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> std::ops::Deref for BoundedString<N> {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

/// Deserialize without token check — trusts stored data.
impl<'de, const N: usize> Deserialize<'de> for BoundedString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_within_limit() {
        let c = WordTokenCounter;
        assert!(BoundedString::<3>::new("a b c", &c).is_ok());
    }

    #[test]
    fn new_rejects_over_limit() {
        let c = WordTokenCounter;
        let err = BoundedString::<2>::new("a b c", &c).unwrap_err();
        assert_eq!(err.actual, 3);
        assert_eq!(err.limit, 2);
    }

    #[test]
    fn empty_always_ok() {
        let c = WordTokenCounter;
        assert!(BoundedString::<1>::new("", &c).is_ok());
    }

    #[test]
    fn display_shows_inner() {
        let c = WordTokenCounter;
        let bs = BoundedString::<5>::new("hello", &c).unwrap();
        assert_eq!(format!("{bs}"), "hello");
    }

    #[test]
    fn deref_to_str() {
        let c = WordTokenCounter;
        let bs = BoundedString::<5>::new("hello world", &c).unwrap();
        assert_eq!(bs.len(), 11); // str::len via Deref
    }
}
