// MVP audit 2026-04-18 §4.10: `restore_output_formatting` is no longer an
// identity stub. It reparses Rust/Python source with tree-sitter and emits
// a deterministic indent-by-depth AST pretty-print; unsupported languages,
// empty input, and parse failures all fall back to the input unchanged so
// callers never break.
//
// This test file lives next to `context_budget_contract.rs` and covers the
// same symbol from the outside — the updated assertion at line 511 of
// `context_budget_contract.rs` is the identity-to-determinism rewrite; the
// checks here add the broader contract surface (determinism across 3 calls,
// unsupported-language identity, empty-input safety, no-panic on garbage).

use elai_orchestrator::context_budget::restore_output_formatting;

// ---------------------------------------------------------------------------
// Contract 1: Rust input produces deterministic output — byte-for-byte
// identical across three calls.
// ---------------------------------------------------------------------------

#[test]
fn contract_rust_pretty_print_is_deterministic_across_three_calls() {
    let code = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let x = add(1, 2);
    println!("{}", x);
}
"#;
    let a = restore_output_formatting(code, "rust");
    let b = restore_output_formatting(code, "rust");
    let c = restore_output_formatting(code, "rust");
    assert_eq!(a, b, "first/second call must match byte-for-byte");
    assert_eq!(b, c, "second/third call must match byte-for-byte");
    assert!(!a.is_empty(), "rust output must be non-empty");
    assert_ne!(a, code, "pretty-print must transform the input for a supported language");
}

// ---------------------------------------------------------------------------
// Contract 2: the "rs" shorthand is accepted (file-extension-style arg) —
// matches the surface of `reclaim_formatting` which accepts "py" for
// Python.
// ---------------------------------------------------------------------------

#[test]
fn contract_rust_extension_shorthand_accepted() {
    let code = "fn f() -> u32 { 42 }";
    let via_full = restore_output_formatting(code, "rust");
    let via_ext = restore_output_formatting(code, "rs");
    assert_eq!(via_full, via_ext, "'rs' must resolve to Rust grammar");
    assert_ne!(via_full, code);
}

// ---------------------------------------------------------------------------
// Contract 3: Python also produces deterministic, transforming output.
// ---------------------------------------------------------------------------

#[test]
fn contract_python_pretty_print_deterministic() {
    let code = "def greet(name):\n    return f'hi {name}'\n";
    let a = restore_output_formatting(code, "python");
    let b = restore_output_formatting(code, "python");
    assert_eq!(a, b);
    assert_ne!(a, code);
    // 'py' shorthand also resolves.
    let via_ext = restore_output_formatting(code, "py");
    assert_eq!(a, via_ext);
}

// ---------------------------------------------------------------------------
// Contract 4: unsupported language → input returned unchanged.
// Covers: the empty string, unknown identifiers, and languages that ELAI
// intentionally does NOT wire here yet (Go, TypeScript, JS). The identity
// fallback is the load-bearing backward-compat guarantee.
// ---------------------------------------------------------------------------

#[test]
fn contract_unsupported_language_returns_identity() {
    let code = "package main\nfunc main() {}";
    assert_eq!(restore_output_formatting(code, "go"), code);
    assert_eq!(restore_output_formatting(code, "typescript"), code);
    assert_eq!(restore_output_formatting(code, "javascript"), code);
    assert_eq!(restore_output_formatting(code, "c++"), code);
    assert_eq!(restore_output_formatting(code, ""), code);
    assert_eq!(restore_output_formatting(code, "totally-unknown"), code);
}

// ---------------------------------------------------------------------------
// Contract 5: empty input never panics and always returns "".
// ---------------------------------------------------------------------------

#[test]
fn contract_empty_input_is_safe() {
    assert_eq!(restore_output_formatting("", "rust"), "");
    assert_eq!(restore_output_formatting("", "python"), "");
    assert_eq!(restore_output_formatting("", ""), "");
    assert_eq!(restore_output_formatting("", "go"), "");
}

// ---------------------------------------------------------------------------
// Contract 6: malformed Rust falls back to identity (has_error => no
// pretty-print). Guards against returning a half-AST on broken input.
// ---------------------------------------------------------------------------

#[test]
fn contract_malformed_rust_falls_back_to_identity() {
    // Unbalanced brace — tree-sitter's error recovery will mark the root
    // as having errors; we must return the original text untouched so the
    // caller (output postprocessor) never ships a corrupt pretty-print.
    let malformed = "fn broken( {";
    let out = restore_output_formatting(malformed, "rust");
    assert_eq!(out, malformed);
}

// ---------------------------------------------------------------------------
// Contract 7: no panic on arbitrary UTF-8 content, even when marked as a
// supported language. The parser may reject it; we must degrade gracefully.
// ---------------------------------------------------------------------------

#[test]
fn contract_arbitrary_utf8_does_not_panic() {
    let inputs = [
        "こんにちは",
        "\u{1F600}\u{1F680}",
        "// comment only\n",
        "   ",
        "\n\n\n",
    ];
    for input in &inputs {
        // Each of these is either valid-ish or invalid for Rust; neither
        // branch may panic. The exact output is unspecified — we only
        // require termination.
        let _ = restore_output_formatting(input, "rust");
        let _ = restore_output_formatting(input, "python");
        let _ = restore_output_formatting(input, "go");
    }
}
