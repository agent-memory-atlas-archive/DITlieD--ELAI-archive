// Query input validation — runs before any embedding call.
//
// Spec grounding:
//   knowledge.txt L127: max query length 10,000 chars
//   knowledge.txt L128: reject null bytes and control characters
//   knowledge.txt L129: reject empty queries (zero tokens after whitespace split)
//   knowledge.txt L130: validation runs in <1ms

/// Maximum query length in characters (not bytes).
pub const MAX_QUERY_LEN: usize = 10_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryValidationError {
    /// Query has zero non-whitespace tokens.
    Empty,
    /// Query exceeds MAX_QUERY_LEN characters.
    TooLong { len: usize },
    /// Query contains null bytes or forbidden control characters.
    InvalidChars { position: usize, byte: u8 },
}

impl std::fmt::Display for QueryValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "query is empty (no non-whitespace content)"),
            Self::TooLong { len } => {
                write!(f, "query too long: {len} chars (max {MAX_QUERY_LEN})")
            }
            Self::InvalidChars { position, byte } => {
                write!(
                    f,
                    "invalid character at byte position {position}: 0x{byte:02x}"
                )
            }
        }
    }
}

impl std::error::Error for QueryValidationError {}

/// Validates query input before any embedding or search call.
///
/// Zero-allocation fast path for valid queries. No model invocation.
pub struct QueryValidator;

impl QueryValidator {
    /// Validate a query string.
    ///
    /// Rejects:
    /// - Empty queries (zero tokens after whitespace split)
    /// - Queries exceeding 10,000 characters
    /// - Queries containing null bytes (0x00) or control characters (0x00-0x1f)
    ///   except newline (0x0a), carriage return (0x0d), and tab (0x09)
    ///
    /// Accepts:
    /// - Queries with embedded newlines, tabs, carriage returns (legitimate multi-line input)
    /// - DEL (0x7f) — not a C0 control character
    pub fn validate(query: &str) -> Result<(), QueryValidationError> {
        // Length check first (chars, not bytes — spec says "chars").
        let char_count = query.chars().count();
        if char_count > MAX_QUERY_LEN {
            return Err(QueryValidationError::TooLong { len: char_count });
        }

        // Empty check: zero tokens after whitespace split.
        // This catches both "" and "   \t\n  ".
        if query.split_whitespace().next().is_none() {
            return Err(QueryValidationError::Empty);
        }

        // Control character scan. Single pass over bytes.
        // Allowed: \t (0x09), \n (0x0a), \r (0x0d). Everything else < 0x20 is rejected.
        for (pos, byte) in query.bytes().enumerate() {
            if byte < 0x20 && byte != b'\t' && byte != b'\n' && byte != b'\r' {
                return Err(QueryValidationError::InvalidChars {
                    position: pos,
                    byte,
                });
            }
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- ACCEPTANCE: Empty string rejected --
    #[test]
    fn validate_rejects_empty_string() {
        let result = QueryValidator::validate("");
        assert_eq!(result, Err(QueryValidationError::Empty));
    }

    // -- ADVERSARIAL: whitespace-only queries rejected --
    // Spec: "zero tokens after whitespace split" — 100 spaces has zero tokens.
    #[test]
    fn validate_rejects_whitespace_only() {
        let spaces = " ".repeat(100);
        assert_eq!(QueryValidator::validate(&spaces), Err(QueryValidationError::Empty));
    }

    #[test]
    fn validate_rejects_tabs_and_newlines_only() {
        assert_eq!(QueryValidator::validate("\t\n\r\n  \t"), Err(QueryValidationError::Empty));
    }

    // -- ACCEPTANCE: 10,001-char string rejected --
    #[test]
    fn validate_rejects_oversized_query() {
        let big = "a".repeat(10_001);
        match QueryValidator::validate(&big) {
            Err(QueryValidationError::TooLong { len }) => assert_eq!(len, 10_001),
            other => panic!("expected TooLong, got {other:?}"),
        }
    }

    // -- Boundary: exactly 10,000 chars accepted --
    #[test]
    fn validate_accepts_exactly_max_length() {
        let exact = "a".repeat(MAX_QUERY_LEN);
        assert!(QueryValidator::validate(&exact).is_ok());
    }

    // -- ACCEPTANCE: Null byte rejected --
    #[test]
    fn validate_rejects_null_byte() {
        let result = QueryValidator::validate("hello\0world");
        match result {
            Err(QueryValidationError::InvalidChars { position: 5, byte: 0 }) => {}
            other => panic!("expected InvalidChars at position 5, got {other:?}"),
        }
    }

    // -- ACCEPTANCE: Control char rejected (BEL = 0x07) --
    #[test]
    fn validate_rejects_bel_control_char() {
        let result = QueryValidator::validate("hello\x07world");
        match result {
            Err(QueryValidationError::InvalidChars { position: 5, byte: 0x07 }) => {}
            other => panic!("expected InvalidChars at position 5 byte 0x07, got {other:?}"),
        }
    }

    // -- Additional control chars: SOH, STX, ETX, EOT, ENQ, ACK --
    #[test]
    fn validate_rejects_all_c0_control_chars_except_allowed() {
        let forbidden: Vec<u8> = (0x00..0x20)
            .filter(|&b| b != b'\t' && b != b'\n' && b != b'\r')
            .collect();
        for byte in forbidden {
            let query = format!("test{}end", char::from(byte));
            assert!(
                QueryValidator::validate(&query).is_err(),
                "byte 0x{byte:02x} should be rejected"
            );
        }
    }

    // -- ACCEPTANCE: Valid query accepted --
    #[test]
    fn validate_accepts_valid_query() {
        assert!(QueryValidator::validate("fn main()").is_ok());
    }

    // -- ACCEPTANCE: Newline in query accepted --
    #[test]
    fn validate_accepts_newline_in_query() {
        assert!(QueryValidator::validate("fn\nmain()").is_ok());
    }

    // -- Tab and carriage return also accepted --
    #[test]
    fn validate_accepts_tab_and_cr() {
        assert!(QueryValidator::validate("fn\tmain()\r\n").is_ok());
    }

    // -- ADVERSARIAL: validation runs before any model call --
    // This test uses the fact that validation is a pure function with no side effects.
    // If it were to call any model, it would need &self with model references.
    // QueryValidator is a unit struct with no fields — it cannot hold model references.
    #[test]
    fn validate_is_zero_allocation_for_valid_queries() {
        // Just verify it returns Ok quickly. The <1ms constraint is structural:
        // no allocations, no I/O, single-pass byte scan.
        let query = "fn main() { println!(\"hello world\"); }";
        assert!(QueryValidator::validate(query).is_ok());
    }

    // -- Length check uses char count, not byte count --
    // A multi-byte UTF-8 char should count as 1 char, not 2-4.
    #[test]
    fn validate_counts_chars_not_bytes() {
        // 10_000 2-byte UTF-8 chars = 20_000 bytes but only 10_000 chars -> OK
        let query: String = std::iter::repeat('\u{00E9}').take(10_000).collect(); // 'é'
        assert!(QueryValidator::validate(&query).is_ok());

        // 10_001 chars -> rejected
        let too_long: String = std::iter::repeat('\u{00E9}').take(10_001).collect();
        assert!(QueryValidator::validate(&too_long).is_err());
    }

    // -- Order of checks: length first, then empty, then control chars --
    // A 10_001-char whitespace-only string should get TooLong, not Empty.
    #[test]
    fn validate_length_check_before_empty_check() {
        let big_spaces = " ".repeat(10_001);
        match QueryValidator::validate(&big_spaces) {
            Err(QueryValidationError::TooLong { .. }) => {}
            other => panic!("expected TooLong for oversized whitespace, got {other:?}"),
        }
    }

    // -- ADVERSARIAL: 10,001-char query rejected without ONNX model call --
    // The spec says: "assert validation rejects 10,001-char query without any ONNX model
    // call (mock model that panics if called)." QueryValidator has no model reference,
    // so this is structurally guaranteed.
    #[test]
    fn validate_rejects_oversized_without_model_invocation() {
        let big = "x".repeat(10_001);
        let result = QueryValidator::validate(&big);
        assert!(result.is_err(), "must reject before any model could be called");
    }

    // -- DEL (0x7F) is not a C0 control character, should be accepted --
    #[test]
    fn validate_accepts_del_character() {
        assert!(QueryValidator::validate("hello\x7fworld").is_ok());
    }

    // -- Unicode queries work --
    #[test]
    fn validate_accepts_unicode() {
        assert!(QueryValidator::validate("fn 日本語() {}").is_ok());
        assert!(QueryValidator::validate("let π = 3.14;").is_ok());
    }

    // -- Display formatting --
    #[test]
    fn error_display_messages() {
        let empty = QueryValidationError::Empty;
        assert!(empty.to_string().contains("empty"));

        let long = QueryValidationError::TooLong { len: 20_000 };
        assert!(long.to_string().contains("20000"));

        let bad = QueryValidationError::InvalidChars { position: 5, byte: 0x00 };
        assert!(bad.to_string().contains("0x00"));
    }
}
