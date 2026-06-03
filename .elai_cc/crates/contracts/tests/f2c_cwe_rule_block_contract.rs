//! F-2c CWE rule BLOCK contract tests (CWE-798/321/532/95).
//!
//! Each of the four new low-FP CWE rules is exercised through the REAL
//! production gate `evaluate_vibegate` (NOT a direct rule-engine call) so the
//! test proves the rule fires on the SAME path the PreToolUse write-gate uses.
//! Per the F-2c controller design decision the four rules are GrepPattern-ONLY
//! (anchored regex predicates), NOT Grep+AST Composite: the production gate
//! wires `NoOpAst` (vibegate.rs:104) whose `evaluate_query` always returns an
//! empty vec, so any AND(Grep,Ast) rule's AST child never matches and the rule
//! would be wire-dark on the live gate. All four rules mirror the 12 existing
//! F-2a grep-only security rules exactly.
//!
//! For each CWE class:
//!   * a vulnerable production snippet BLOCKs,
//!   * its patched / placeholder counterpart does NOT block,
//!   * the same vulnerable snippet in a test-context file does NOT block
//!     (file_context = "production" excludes test paths).

use elai_orchestrator::vibegate::{evaluate_vibegate, VIBESEC_CODE_VULN_CATEGORY};
use elai_orchestrator::hook_dispatcher::HookVerdictKind;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// True when `evaluate_vibegate` surfaces a code-vuln Block for `source` at
/// `file_path`, and (when `expect_rule_prefix` is non-empty) the blocking
/// rule_id carries that prefix.
fn code_vuln_blocks(source: &str, file_path: &str, expect_rule_prefix: &str) -> bool {
    evaluate_vibegate("Edit", source, file_path).iter().any(|f| {
        f.verdict == HookVerdictKind::Block
            && f.category == VIBESEC_CODE_VULN_CATEGORY
            && (expect_rule_prefix.is_empty()
                || f.rule_id.as_deref().is_some_and(|id| id.starts_with(expect_rule_prefix)))
    })
}

/// True when `evaluate_vibegate` surfaces ANY Block (code-vuln OR secret axis)
/// for `source` at `file_path`. Used for the must-NOT-block negatives so a
/// patched snippet that the secret axis would also clear is asserted clean.
fn any_block(source: &str, file_path: &str) -> bool {
    evaluate_vibegate("Edit", source, file_path)
        .iter()
        .any(|f| f.verdict == HookVerdictKind::Block)
}

// ---------------------------------------------------------------------------
// CWE-798 — hardcoded credentials (SEC-CREDS-*)
// ---------------------------------------------------------------------------

#[test]
fn cwe798_python_hardcoded_password_blocks() {
    let source = r#"def connect():
    password = "hunter2pw"
    return password"#;
    assert!(
        code_vuln_blocks(source, "app/db.py", "SEC-CREDS-"),
        "CWE-798 Python hardcoded password literal must BLOCK via SEC-CREDS-*"
    );
}

#[test]
fn cwe798_typescript_hardcoded_apikey_blocks() {
    let source = r#"const apiKey = "sk-live-9aF2k0xQ7b";"#;
    assert!(
        code_vuln_blocks(source, "src/client.ts", "SEC-CREDS-"),
        "CWE-798 TypeScript hardcoded api_key literal must BLOCK via SEC-CREDS-*"
    );
}

#[test]
fn cwe798_placeholder_does_not_block() {
    // A placeholder literal is the FP-bounding case: must NOT block.
    let source = r#"def connect():
    password = "CHANGEME"
    return password"#;
    assert!(
        !any_block(source, "app/db.py"),
        "CWE-798 placeholder 'CHANGEME' must NOT block (FP guard)"
    );
}

#[test]
fn cwe798_env_read_does_not_block() {
    // Reading from the environment is the safe form: no string-literal assign.
    let source = r#"import os
password = os.environ["DB_PASSWORD"]"#;
    assert!(
        !any_block(source, "app/db.py"),
        "CWE-798 env-read (no literal) must NOT block"
    );
}

#[test]
fn cwe798_test_context_does_not_block() {
    let source = r#"def test_login():
    password = "hunter2pw"
    assert login(password)"#;
    // file_context=production => a test path must not fire.
    assert!(
        !code_vuln_blocks(source, "tests/test_login.py", "SEC-CREDS-"),
        "CWE-798 rule must NOT fire in a test-context file"
    );
}

// ---------------------------------------------------------------------------
// CWE-321 — hardcoded cryptographic key (SEC-KEY-*)
// ---------------------------------------------------------------------------

#[test]
fn cwe321_python_literal_aes_key_blocks() {
    let source = r#"from Crypto.Cipher import AES
cipher = AES.new(b"0123456789abcdef", AES.MODE_ECB)"#;
    assert!(
        code_vuln_blocks(source, "app/crypto.py", "SEC-KEY-"),
        "CWE-321 Python literal AES key must BLOCK via SEC-KEY-*"
    );
}

#[test]
fn cwe321_typescript_literal_cipher_key_blocks() {
    let source = r#"const c = crypto.createCipheriv("aes-256-cbc", "0123456789abcdef0123456789abcdef", iv);"#;
    assert!(
        code_vuln_blocks(source, "src/enc.ts", "SEC-KEY-"),
        "CWE-321 TypeScript literal createCipheriv key must BLOCK via SEC-KEY-*"
    );
}

#[test]
fn cwe321_key_from_variable_does_not_block() {
    // Key sourced from a variable (not a literal at the sink) is the safe form.
    let source = r#"from Crypto.Cipher import AES
cipher = AES.new(key, AES.MODE_ECB)"#;
    assert!(
        !any_block(source, "app/crypto.py"),
        "CWE-321 non-literal key at the crypto sink must NOT block"
    );
}

#[test]
fn cwe321_test_context_does_not_block() {
    let source = r#"from Crypto.Cipher import AES
cipher = AES.new(b"0123456789abcdef", AES.MODE_ECB)"#;
    assert!(
        !code_vuln_blocks(source, "tests/test_crypto.py", "SEC-KEY-"),
        "CWE-321 rule must NOT fire in a test-context file"
    );
}

// ---------------------------------------------------------------------------
// CWE-532 — secrets written to logs (SEC-LOG-*)
// ---------------------------------------------------------------------------

#[test]
fn cwe532_python_secret_in_logging_blocks() {
    let source = r#"import logging
logging.info("user password=%s", password)"#;
    assert!(
        code_vuln_blocks(source, "app/auth.py", "SEC-LOG-"),
        "CWE-532 Python secret in logging.info must BLOCK via SEC-LOG-*"
    );
}

#[test]
fn cwe532_typescript_secret_in_console_blocks() {
    let source = r#"console.log("session token", token);"#;
    assert!(
        code_vuln_blocks(source, "src/session.ts", "SEC-LOG-"),
        "CWE-532 TypeScript secret in console.log must BLOCK via SEC-LOG-*"
    );
}

#[test]
fn cwe532_benign_log_does_not_block() {
    // A log line with no secret-named identifier must NOT block.
    let source = r#"import logging
logging.info("request completed in %d ms", elapsed)"#;
    assert!(
        !any_block(source, "app/auth.py"),
        "CWE-532 benign log (no secret identifier) must NOT block"
    );
}

#[test]
fn cwe532_test_context_does_not_block() {
    let source = r#"import logging
logging.info("user password=%s", password)"#;
    assert!(
        !code_vuln_blocks(source, "app/__tests__/auth_test.py", "SEC-LOG-"),
        "CWE-532 rule must NOT fire in a test-context file"
    );
}

// ---------------------------------------------------------------------------
// CWE-95 — eval / exec on a non-literal argument (SEC-EVAL-*)
// ---------------------------------------------------------------------------

#[test]
fn cwe95_python_eval_on_variable_blocks() {
    let source = r#"def run(user_input):
    return eval(user_input)"#;
    assert!(
        code_vuln_blocks(source, "app/runner.py", "SEC-EVAL-"),
        "CWE-95 Python eval(variable) must BLOCK via SEC-EVAL-*"
    );
}

#[test]
fn cwe95_typescript_new_function_blocks() {
    let source = r#"const fn = new Function(userCode);"#;
    assert!(
        code_vuln_blocks(source, "src/sandbox.ts", "SEC-EVAL-"),
        "CWE-95 TypeScript new Function(variable) must BLOCK via SEC-EVAL-*"
    );
}

#[test]
fn cwe95_eval_literal_does_not_block() {
    // eval("1+1") on a string literal is not injection — must NOT block.
    let source = r#"x = eval("1 + 1")"#;
    assert!(
        !any_block(source, "app/runner.py"),
        "CWE-95 eval on a string literal must NOT block (literal is not injection)"
    );
}

#[test]
fn cwe95_test_context_does_not_block() {
    let source = r#"def test_run():
    return eval(user_input)"#;
    assert!(
        !code_vuln_blocks(source, "tests/test_runner.py", "SEC-EVAL-"),
        "CWE-95 rule must NOT fire in a test-context file"
    );
}
