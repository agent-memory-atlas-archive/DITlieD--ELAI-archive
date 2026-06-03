//! F-2a vibesec seed integrity guard.
//!
//! Provides [`verify_vibesec_seed_integrity`] which queries the rules DB for
//! all mutable=0 security seed rows (source=SEED, domain=security) and checks
//! their blake3 digest against the canonical digest embedded at compile time.
//!
//! If the DB rows have been tampered with after seeding, the digest will not
//! match and the function returns `Err(IntegrityError::DigestMismatch)`.
//!
//! The canonical digest is derived from the 12 rule_id + predicate_json pairs
//! defined in crates/rules/seed/security/*.toml (F-2a security ruleset).
//! Any change to a predicate requires updating the matching constant below and
//! re-validating with the tests in this module.
//!
//! String encoding note: the constants below must be byte-identical to what
//! the TOML loader stores in the database. The TOML files use single-quoted
//! (literal) strings, so the stored value is the verbatim TOML literal content.
//! Rust raw strings (`r#"..."#`) preserve the same byte sequence without any
//! additional escape processing, making them the correct representation here.

use rusqlite::Connection;
use tracing::warn;

// ---------------------------------------------------------------------------
// Canonical predicate_json strings (byte-identical to seed/security/*.toml)
//
// TOML single-quoted strings have no escape processing: every byte is literal.
// Rust raw strings r#"..."# have the same property. The values below are
// transcribed directly from the predicate_json fields in the TOML files.
// ---------------------------------------------------------------------------

// SEC-AUTHZ-001 (missing_authz.toml)
const PRED_AUTHZ_001: &str = r#"{"type":"grep_pattern","pattern":"@(app|router)\\.(get|post|put|delete|patch)\\s*\\(","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-CREDS-001 (hardcoded_creds.toml) — CWE-798
const PRED_CREDS_001: &str = r#"{"type":"grep_pattern","pattern":"(?i)(password|passwd|secret|api_key|apikey|token)\\s*=\\s*\"[^\"\\n]{2,}[0-9][^\"\\n]*\"","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-CREDS-002 (hardcoded_creds.toml) — CWE-798
const PRED_CREDS_002: &str = r#"{"type":"grep_pattern","pattern":"(?i)(password|passwd|secret|api_key|apikey|token)\\s*=\\s*\"[^\"\\n]{2,}[0-9][^\"\\n]*\"","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-EVAL-001 (eval_injection.toml) — CWE-95
const PRED_EVAL_001: &str = r#"{"type":"grep_pattern","pattern":"(eval|exec)\\s*\\(\\s*[A-Za-z_]","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-EVAL-002 (eval_injection.toml) — CWE-95
const PRED_EVAL_002: &str = r#"{"type":"grep_pattern","pattern":"(eval\\s*\\(\\s*[A-Za-z_]|new\\s+Function\\s*\\(\\s*[A-Za-z_])","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-KEY-001 (hardcoded_key.toml) — CWE-321
const PRED_KEY_001: &str = r#"{"type":"grep_pattern","pattern":"(AES|DES|ARC4)\\.new\\s*\\(\\s*b?\"[^\"\\n]{8,}\"|Fernet\\s*\\(\\s*b?\"[^\"\\n]{8,}\"","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-KEY-002 (hardcoded_key.toml) — CWE-321
const PRED_KEY_002: &str = r#"{"type":"grep_pattern","pattern":"createCipheriv\\s*\\(\\s*\"[^\"\\n]+\"\\s*,\\s*\"[0-9a-fA-F]{16,}\"","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-LOG-001 (secrets_in_logs.toml) — CWE-532
const PRED_LOG_001: &str = r#"{"type":"grep_pattern","pattern":"(logging\\.(info|debug|warning|error|critical)|logger\\.[a-z]+|print)\\s*\\([^)\\n]*(password|passwd|secret|token|api_key|apikey)","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-LOG-002 (secrets_in_logs.toml) — CWE-532
const PRED_LOG_002: &str = r#"{"type":"grep_pattern","pattern":"console\\.(log|info|error|warn|debug)\\s*\\([^)\\n]*(password|passwd|secret|token|api_key|apikey)","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-AUTHZ-002 (missing_authz.toml)
const PRED_AUTHZ_002: &str = r#"{"type":"grep_pattern","pattern":"(app|router)\\.(get|post|put|delete|patch)\\s*\\(\\s*\"[^\"]+\"\\s*,\\s*async\\s*\\(req","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-SQLI-001 (sqli.toml)
const PRED_SQLI_001: &str = r#"{"type":"grep_pattern","pattern":"(execute|executemany|raw)\\s*\\(\\s*f\"(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE)","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-SQLI-002 (sqli.toml)
const PRED_SQLI_002: &str = r#"{"type":"grep_pattern","pattern":"\"(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE)[^\"]*\"\\.format\\s*\\(|\"(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE)[^\"]*\"\\s*%","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-SQLI-003 (sqli.toml)
const PRED_SQLI_003: &str = r#"{"type":"grep_pattern","pattern":"(query|execute|raw)\\s*\\(\\s*`\\s*(SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE)[^`]*\\$\\{","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-WCRYPTO-001 (weak_crypto.toml)
const PRED_WCRYPTO_001: &str = r#"{"type":"grep_pattern","pattern":"hashlib\\.(md5|sha1)\\s*\\(","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-WCRYPTO-002 (weak_crypto.toml)
const PRED_WCRYPTO_002: &str = r#"{"type":"grep_pattern","pattern":"Crypto\\.Cipher\\.DES|from Crypto\\.Cipher import DES|ARC4|from Crypto\\.Cipher import ARC4","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-WCRYPTO-003 (weak_crypto.toml)
const PRED_WCRYPTO_003: &str = r#"{"type":"grep_pattern","pattern":"createHash\\s*\\(\\s*\"(md5|sha1)\"\\s*\\)","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-WCRYPTO-004 (weak_crypto.toml)
const PRED_WCRYPTO_004: &str = r#"{"type":"grep_pattern","pattern":"createCipheriv\\s*\\(\\s*\"(des|des-cbc|des-ecb|rc4)\"","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-XSS-001 (xss.toml)
const PRED_XSS_001: &str = r#"{"type":"grep_pattern","pattern":"mark_safe\\s*\\(|Markup\\s*\\(|jinja2\\.Markup\\s*\\(","scope":null,"context_lines":2,"language":"python","file_context":"production"}"#;

// SEC-XSS-002 (xss.toml)
const PRED_XSS_002: &str = r#"{"type":"grep_pattern","pattern":"\\.innerHTML\\s*=|\\.outerHTML\\s*=|insertAdjacentHTML\\s*\\(","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

// SEC-XSS-003 (xss.toml)
const PRED_XSS_003: &str = r#"{"type":"grep_pattern","pattern":"dangerouslySetInnerHTML\\s*=\\s*\\{\\s*\\{\\s*__html","scope":null,"context_lines":2,"language":"typescript","file_context":"production"}"#;

/// Canonical rows ordered by rule_id ASC — must mirror seed/security/*.toml exactly.
///
/// `pub` so the F-2c predicate-shape contract (crates/contracts) can assert the
/// new CWE-798/321/532/95 rows are GrepPattern-only and registered-language.
pub const CANONICAL_ROWS: &[(&str, &str)] = &[
    ("SEC-AUTHZ-001", PRED_AUTHZ_001),
    ("SEC-AUTHZ-002", PRED_AUTHZ_002),
    ("SEC-CREDS-001", PRED_CREDS_001),
    ("SEC-CREDS-002", PRED_CREDS_002),
    ("SEC-EVAL-001", PRED_EVAL_001),
    ("SEC-EVAL-002", PRED_EVAL_002),
    ("SEC-KEY-001", PRED_KEY_001),
    ("SEC-KEY-002", PRED_KEY_002),
    ("SEC-LOG-001", PRED_LOG_001),
    ("SEC-LOG-002", PRED_LOG_002),
    ("SEC-SQLI-001", PRED_SQLI_001),
    ("SEC-SQLI-002", PRED_SQLI_002),
    ("SEC-SQLI-003", PRED_SQLI_003),
    ("SEC-WCRYPTO-001", PRED_WCRYPTO_001),
    ("SEC-WCRYPTO-002", PRED_WCRYPTO_002),
    ("SEC-WCRYPTO-003", PRED_WCRYPTO_003),
    ("SEC-WCRYPTO-004", PRED_WCRYPTO_004),
    ("SEC-XSS-001", PRED_XSS_001),
    ("SEC-XSS-002", PRED_XSS_002),
    ("SEC-XSS-003", PRED_XSS_003),
];

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors from the vibesec seed integrity check.
#[derive(Debug, thiserror::Error)]
pub enum IntegrityError {
    #[error("sqlite error querying security seed rows: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error(
        "vibesec seed digest mismatch: expected {expected}, got {actual}; \
         security ruleset may have been tampered with after seeding"
    )]
    DigestMismatch { expected: String, actual: String },

    #[error(
        "vibesec seed row count mismatch: expected {expected} rows, found {actual}"
    )]
    RowCountMismatch { expected: usize, actual: usize },
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the canonical blake3 hex digest for the embedded security seed rows.
///
/// This is the "ground truth" digest derived from the F-2a TOML definitions.
/// Use this to validate a fresh DB or to update the expected digest after a
/// deliberate ruleset change.
pub fn canonical_hex_digest() -> String {
    let payload = build_payload(CANONICAL_ROWS.iter().map(|&(id, pred)| (id, pred)));
    hex_digest(payload.as_bytes())
}

/// Verify that the mutable=0 security seed rows in `conn` have not been
/// tampered with since seeding.
///
/// Queries `rules` for all rows where `domain = 'security'` AND `mutable = 0`
/// AND `source = 'SEED'`, sorted by `rule_id` ASC. Serialises
/// `rule_id + "|" + predicate` for each row (newline-separated), computes
/// blake3, and compares to the compile-time canonical digest.
///
/// Returns `Ok(())` if the digest matches, or `Err(IntegrityError::*)` if the
/// row count or digest does not match.
pub fn verify_vibesec_seed_integrity(conn: &Connection) -> Result<(), IntegrityError> {
    let mut stmt = conn.prepare(
        "SELECT rule_id, predicate \
         FROM rules \
         WHERE domain = 'security' AND mutable = 0 AND source = 'SEED' \
         ORDER BY rule_id ASC",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
        .collect::<Result<_, _>>()?;

    let expected_count = CANONICAL_ROWS.len();
    if rows.len() != expected_count {
        warn!(
            expected = expected_count,
            actual = rows.len(),
            "vibesec seed row count mismatch"
        );
        return Err(IntegrityError::RowCountMismatch {
            expected: expected_count,
            actual: rows.len(),
        });
    }

    let actual_payload = build_payload(rows.iter().map(|(id, pred)| (id.as_str(), pred.as_str())));
    let actual_digest = hex_digest(actual_payload.as_bytes());
    let expected_digest = canonical_hex_digest();

    if actual_digest != expected_digest {
        warn!(
            expected = %expected_digest,
            actual = %actual_digest,
            "vibesec seed integrity digest mismatch"
        );
        return Err(IntegrityError::DigestMismatch {
            expected: expected_digest,
            actual: actual_digest,
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Build the stable serialization payload: `"rule_id|predicate_json\n"` per row.
/// Caller must guarantee the rows are already sorted by rule_id ASC.
fn build_payload<'a>(rows: impl Iterator<Item = (&'a str, &'a str)>) -> String {
    let mut out = String::new();
    for (rule_id, predicate) in rows {
        out.push_str(rule_id);
        out.push('|');
        out.push_str(predicate);
        out.push('\n');
    }
    out
}

/// Compute lowercase hex blake3 digest of `data`.
fn hex_digest(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_digest_is_stable() {
        let d1 = canonical_hex_digest();
        let d2 = canonical_hex_digest();
        assert_eq!(d1, d2, "canonical digest must be deterministic");
        assert_eq!(d1.len(), 64, "blake3 hex digest must be 64 chars");
    }

    #[test]
    fn canonical_row_count_matches_expected() {
        assert_eq!(
            CANONICAL_ROWS.len(),
            20,
            "20 security seed rules: 12 F-2a + 8 F-2c (CWE-798/321/532/95 × py+ts)"
        );
    }

    #[test]
    fn canonical_rows_sorted_by_rule_id() {
        let ids: Vec<&str> = CANONICAL_ROWS.iter().map(|&(id, _)| id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "CANONICAL_ROWS must be sorted by rule_id ASC");
    }

    #[test]
    fn build_payload_is_deterministic() {
        let rows = [("A", "pred_a"), ("B", "pred_b")];
        let p1 = build_payload(rows.iter().map(|&(id, pred)| (id, pred)));
        let p2 = build_payload(rows.iter().map(|&(id, pred)| (id, pred)));
        assert_eq!(p1, p2);
        assert_eq!(p1, "A|pred_a\nB|pred_b\n");
    }

    #[test]
    fn verify_passes_with_correct_rows() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE rules (
                rule_id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                mutable INTEGER NOT NULL DEFAULT 1,
                source TEXT NOT NULL,
                predicate TEXT NOT NULL
            );",
        )
        .expect("create table");

        for &(rule_id, predicate) in CANONICAL_ROWS {
            conn.execute(
                "INSERT INTO rules (rule_id, domain, mutable, source, predicate) \
                 VALUES (?1, 'security', 0, 'SEED', ?2)",
                rusqlite::params![rule_id, predicate],
            )
            .expect("insert");
        }

        verify_vibesec_seed_integrity(&conn).expect("integrity check must pass");
    }

    #[test]
    fn verify_fails_on_tampered_predicate() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE rules (
                rule_id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                mutable INTEGER NOT NULL DEFAULT 1,
                source TEXT NOT NULL,
                predicate TEXT NOT NULL
            );",
        )
        .expect("create table");

        for &(rule_id, predicate) in CANONICAL_ROWS {
            let p = if rule_id == "SEC-SQLI-001" {
                "tampered_predicate_json"
            } else {
                predicate
            };
            conn.execute(
                "INSERT INTO rules (rule_id, domain, mutable, source, predicate) \
                 VALUES (?1, 'security', 0, 'SEED', ?2)",
                rusqlite::params![rule_id, p],
            )
            .expect("insert");
        }

        let err = verify_vibesec_seed_integrity(&conn)
            .expect_err("tampered predicate must cause mismatch");
        assert!(
            matches!(err, IntegrityError::DigestMismatch { .. }),
            "expected DigestMismatch, got: {err}"
        );
    }

    #[test]
    fn verify_fails_on_row_count_mismatch() {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            "CREATE TABLE rules (
                rule_id TEXT PRIMARY KEY,
                domain TEXT NOT NULL,
                mutable INTEGER NOT NULL DEFAULT 1,
                source TEXT NOT NULL,
                predicate TEXT NOT NULL
            );",
        )
        .expect("create table");

        // Insert only the first row — count mismatch.
        let &(rule_id, predicate) = CANONICAL_ROWS.first().expect("at least one row");
        conn.execute(
            "INSERT INTO rules (rule_id, domain, mutable, source, predicate) \
             VALUES (?1, 'security', 0, 'SEED', ?2)",
            rusqlite::params![rule_id, predicate],
        )
        .expect("insert");

        let err = verify_vibesec_seed_integrity(&conn)
            .expect_err("row count mismatch must be an error");
        assert!(
            matches!(err, IntegrityError::RowCountMismatch { .. }),
            "expected RowCountMismatch, got: {err}"
        );
    }
}
