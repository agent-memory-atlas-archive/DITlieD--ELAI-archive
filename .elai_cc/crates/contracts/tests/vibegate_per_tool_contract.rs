//! F-1 VibeGate per-tool contract tests.
//!
//! For each of the four mutating tools (Edit / Write / NotebookEdit / Bash),
//! asserts that evaluate_vibegate:
//!   1. BLOCKs a payload carrying a known SQLi string (vibesec_code_vuln).
//!   2. BLOCKs a payload carrying the canonical AWS access-key sentinel
//!      (AKIAIOSFODNN7EXAMPLE — vibesec_secret).
//!   3. Passes a clean payload with no block findings.
//!
//! evaluate_vibegate is a pure synchronous function — no DB or runtime needed.

use elai_orchestrator::hook_dispatcher::HookVerdictKind;
use elai_orchestrator::vibegate::{
    evaluate_vibegate, VIBESEC_CODE_VULN_CATEGORY, VIBESEC_SECRET_CATEGORY,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Python SQLi via f-string — must trigger SEC-SQLI-001 (CWE-89).
const SQLI_SOURCE: &str =
    r#"result = cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")"#;

/// AWS access-key canonical test credential — must trigger the secret axis.
const AWS_KEY_SOURCE: &str = r#"let key = "AKIAIOSFODNN7EXAMPLEKEY1";"#;

/// Clean Rust source — must pass both axes.
const CLEAN_SOURCE: &str = "fn add(a: i32, b: i32) -> i32 { a + b }";

fn assert_code_vuln_block(tool: &str, source: &str, file: &str) {
    let findings = evaluate_vibegate(tool, source, file);
    assert!(
        findings
            .iter()
            .any(|f| f.verdict == HookVerdictKind::Block
                && f.category == VIBESEC_CODE_VULN_CATEGORY),
        "{tool}: SQLi source must produce a vibesec_code_vuln Block finding; got {findings:?}"
    );
}

fn assert_secret_block(tool: &str, source: &str, file: &str) {
    let findings = evaluate_vibegate(tool, source, file);
    assert!(
        findings
            .iter()
            .any(|f| f.verdict == HookVerdictKind::Block
                && f.category == VIBESEC_SECRET_CATEGORY),
        "{tool}: AWS key source must produce a vibesec_secret Block finding; got {findings:?}"
    );
}

fn assert_clean_pass(tool: &str, source: &str, file: &str) {
    let findings = evaluate_vibegate(tool, source, file);
    let blocks: Vec<_> = findings
        .iter()
        .filter(|f| f.verdict == HookVerdictKind::Block)
        .collect();
    assert!(
        blocks.is_empty(),
        "{tool}: clean source must produce no Block findings; got {blocks:?}"
    );
}

// ---------------------------------------------------------------------------
// Edit tool
// ---------------------------------------------------------------------------

#[test]
fn contract_edit_tool_blocks_sqli() {
    assert_code_vuln_block("Edit", SQLI_SOURCE, "app/db.py");
}

#[test]
fn contract_edit_tool_blocks_aws_key() {
    assert_secret_block("Edit", AWS_KEY_SOURCE, "src/config.rs");
}

#[test]
fn contract_edit_tool_passes_clean_source() {
    assert_clean_pass("Edit", CLEAN_SOURCE, "src/math.rs");
}

// ---------------------------------------------------------------------------
// Write tool
// ---------------------------------------------------------------------------

#[test]
fn contract_write_tool_blocks_sqli() {
    assert_code_vuln_block("Write", SQLI_SOURCE, "app/repository.py");
}

#[test]
fn contract_write_tool_blocks_aws_key() {
    assert_secret_block("Write", AWS_KEY_SOURCE, "src/credentials.rs");
}

#[test]
fn contract_write_tool_passes_clean_source() {
    assert_clean_pass("Write", CLEAN_SOURCE, "src/lib.rs");
}

// ---------------------------------------------------------------------------
// NotebookEdit tool
// ---------------------------------------------------------------------------

#[test]
fn contract_notebook_edit_blocks_sqli() {
    assert_code_vuln_block("NotebookEdit", SQLI_SOURCE, "notebooks/analysis.py");
}

#[test]
fn contract_notebook_edit_blocks_aws_key() {
    assert_secret_block("NotebookEdit", AWS_KEY_SOURCE, "notebooks/config.py");
}

#[test]
fn contract_notebook_edit_passes_clean_source() {
    assert_clean_pass("NotebookEdit", CLEAN_SOURCE, "notebooks/helpers.py");
}

// ---------------------------------------------------------------------------
// Bash tool
// ---------------------------------------------------------------------------

#[test]
fn contract_bash_blocks_sqli() {
    // Bash can carry inline Python — the gate evaluates source text regardless.
    assert_code_vuln_block("Bash", SQLI_SOURCE, "scripts/migrate.py");
}

#[test]
fn contract_bash_blocks_aws_key() {
    assert_secret_block("Bash", AWS_KEY_SOURCE, "scripts/setup.sh");
}

#[test]
fn contract_bash_passes_clean_source() {
    assert_clean_pass("Bash", CLEAN_SOURCE, "scripts/build.sh");
}

// ---------------------------------------------------------------------------
// Source-blake3 helper: prefix length is 16 hex chars
// ---------------------------------------------------------------------------

#[test]
fn contract_source_blake3_prefix_is_16_chars() {
    use elai_orchestrator::vibegate::source_blake3_prefix;
    let prefix = source_blake3_prefix(SQLI_SOURCE);
    assert_eq!(
        prefix.len(),
        16,
        "source_blake3_prefix must return exactly 16 hex chars"
    );
    assert!(
        prefix.chars().all(|c| c.is_ascii_hexdigit()),
        "source_blake3_prefix must be lowercase hex"
    );
}
