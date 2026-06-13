//! I-48 — inline prompt literals ban contract.
//!
//! Asserts:
//!   (a) POSITIVE: all 9 extracted prompt .md files exist at their expected
//!       paths under crates/orchestrator/prompts/ and crates/cli/prompts/.
//!   (b) POSITIVE: each named const in the dispatch modules contains the
//!       `include_str!` binding (not an inline literal) for its prompt file.
//!   (c) NEGATIVE: no dispatch module contains a string literal >=200 chars
//!       that includes a role-framing marker (the ban; proves inline literals
//!       were not re-introduced).
//!   (d) SELF-TEST: a synthetic 210-char literal containing a role-framing
//!       marker IS detected by the scanner (proves the check is live, R2).
//!
//! Scan scope: 8 dispatch modules (7 orchestrator + 1 cli).
//! Threshold: 200 chars (safely above all legitimate short format strings in
//!            scope; safely below the shortest extracted prompt at 91 chars
//!            post-extraction, which is now absent anyway).
//!
//! Grounding:
//!   * context-package-I-48.md (byte-class analysis, TRICKY/CLEAN classification)
//!   * exploration/I-48-prompts.md (ban test shape, threshold recommendation,
//!     allowlist rationale, ARTIFACT_include_str_HARNESS_DEADCODE pitfall)
//!   * crates/contracts/tests/i51_model_catalog_drift_contract.rs (POSITIVE+NEGATIVE shape)
//!   * .claude/rules/wiring-enforcement.md R2 (dead-evidence path must scream)

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

/// Resolve the .elai_cc root from CARGO_MANIFEST_DIR (contracts crate).
/// contracts/ is at .elai_cc/crates/contracts/ -> parent().parent() = .elai_cc/
fn elai_cc_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .expect("crates dir")
        .parent()
        .expect(".elai_cc root")
        .to_path_buf()
}

fn orchestrator_prompts_dir() -> PathBuf {
    elai_cc_root()
        .join("crates")
        .join("orchestrator")
        .join("src")
        .join("prompts")
}

fn cli_prompts_dir() -> PathBuf {
    elai_cc_root()
        .join("crates")
        .join("cli")
        .join("src")
        .join("prompts")
}

fn orchestrator_src() -> PathBuf {
    elai_cc_root().join("crates").join("orchestrator").join("src")
}

fn cli_src() -> PathBuf {
    elai_cc_root().join("crates").join("cli").join("src")
}

// ---------------------------------------------------------------------------
// (a) POSITIVE: all 9 prompt files exist and are non-empty
// ---------------------------------------------------------------------------

#[test]
fn contract_all_prompt_files_exist() {
    let odir = orchestrator_prompts_dir();
    let cdir = cli_prompts_dir();

    let expected: &[(&str, &str)] = &[
        // (dir, filename)
        ("orchestrator", "baby_step_system.md"),
        ("orchestrator", "research_synthesizer.md"),
        ("orchestrator", "quarantine_system.md"),
        ("orchestrator", "hook_prompt_handler.md"),
        ("orchestrator", "automix_verifier.md"),
        ("orchestrator", "agent_preamble.md"),
        ("orchestrator", "simple_intro_default.md"),
        ("orchestrator", "simple_intro_with_style.md"),
        ("cli", "live_chat_system.md"),
    ];

    for (crate_name, fname) in expected {
        let path = if *crate_name == "orchestrator" {
            odir.join(fname)
        } else {
            cdir.join(fname)
        };
        assert!(
            path.exists(),
            "prompt file must exist: {}",
            path.display()
        );
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        assert!(
            !content.is_empty(),
            "prompt file must be non-empty: {}",
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// (b) POSITIVE: named const binding sites use include_str! not inline literals
// ---------------------------------------------------------------------------

#[test]
fn contract_const_bindings_use_include_str() {
    let src_dir = orchestrator_src();

    // (file, expected include_str! argument fragment)
    let bindings: &[(&str, &str)] = &[
        (
            "production_agent_rpc.rs",
            "include_str!(\"prompts/baby_step_system.md\")",
        ),
        (
            "quarantine.rs",
            "include_str!(\"prompts/quarantine_system.md\")",
        ),
        (
            "cost/automix_gate.rs",
            "include_str!(\"../prompts/automix_verifier.md\")",
        ),
        (
            "architecture/hook_prompt.rs",
            "include_str!(\"../prompts/hook_prompt_handler.md\")",
        ),
        (
            "research_pipeline.rs",
            "include_str!(\"prompts/research_synthesizer.md\")",
        ),
        (
            "prompt.rs",
            "include_str!(\"prompts/simple_intro_default.md\")",
        ),
        (
            "prompt.rs",
            "include_str!(\"prompts/simple_intro_with_style.md\")",
        ),
        (
            "agent_template.rs",
            "include_str!(\"prompts/agent_preamble.md\")",
        ),
    ];

    for (rel_path, needle) in bindings {
        let path = src_dir.join(rel_path);
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        assert!(
            src.contains(needle),
            "expected include_str! binding `{needle}` in {}",
            path.display()
        );
    }

    // cli/live_cli.rs must have the cli include_str! binding
    let live_cli = cli_src().join("live_cli.rs");
    let live_cli_src = std::fs::read_to_string(&live_cli)
        .unwrap_or_else(|e| panic!("failed to read live_cli.rs: {e}"));
    assert!(
        live_cli_src.contains("include_str!(\"prompts/live_chat_system.md\")"),
        "live_cli.rs must contain include_str! binding for live_chat_system.md"
    );
}

// ---------------------------------------------------------------------------
// (c) NEGATIVE ban: no string literal >=200 chars with a role-framing marker
// ---------------------------------------------------------------------------

/// Role-framing markers that indicate a prompt literal rather than a short
/// operational string.  The set is kept small and precise to avoid false
/// positives from legitimate source comments or error messages.
const ROLE_FRAMING_MARKERS: &[&str] = &[
    "You are ",
    "MUST include a tool",
    "MUST end with a citation",
    "Return STRICT JSON",
    "Hard rules:",
    "QUARANTINED summarizer",
    "strict-JSON decision-maker",
    "background sub-agent",
    "interactive coding assistant",
    "interactive agent that helps",
    "factual research synthesizer",
    "output verifier",
];

const INLINE_PROMPT_THRESHOLD: usize = 200;

/// Returns true if `src` contains a string literal (between `"`) that is
/// >=threshold chars long AND contains at least one role-framing marker.
///
/// The scanner is a best-effort pattern match over source bytes, not a full
/// Rust parser.  It is deliberately conservative: it scans for runs of
/// characters surrounded by `"` that are not preceded by `include_str!(`.
/// False-negative risk is low given the targeted dispatch modules are short
/// and well-structured.  False-positives are eliminated by keeping the
/// threshold high and the marker set role-specific.
///
/// Per exploration/I-48-prompts.md and context-package: no allowlist entries
/// are needed after correct extraction because include_str! call sites contain
/// no string literals matching the role-framing patterns.  The only non-prompt
/// string over ~60 chars in scope is build_verifier_user_prompt's format string
/// (~60 chars, no role-framing markers) — safely below threshold.
fn contains_banned_inline_prompt(src: &str) -> bool {
    let bytes = src.as_bytes();
    let len = bytes.len();
    let include_str_marker = b"include_str!(";
    let mut i = 0;

    while i < len {
        // Skip `include_str!(` contexts — the argument is a path, not a prompt.
        if bytes[i..].starts_with(include_str_marker) {
            // advance past the closing paren
            if let Some(close) = bytes[i..].iter().position(|&b| b == b')') {
                i += close + 1;
                continue;
            }
        }

        if bytes[i] == b'"' {
            // Find the closing quote (skip escaped quotes)
            let mut j = i + 1;
            while j < len {
                if bytes[j] == b'\\' {
                    j += 2; // skip escape sequence
                    continue;
                }
                if bytes[j] == b'"' {
                    break;
                }
                j += 1;
            }
            if j < len {
                let literal_bytes = &bytes[i + 1..j];
                if literal_bytes.len() >= INLINE_PROMPT_THRESHOLD {
                    for marker in ROLE_FRAMING_MARKERS {
                        if literal_bytes
                            .windows(marker.len())
                            .any(|w| w == marker.as_bytes())
                        {
                            return true;
                        }
                    }
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    false
}

#[test]
fn contract_no_inline_prompt_literals_in_dispatch_modules() {
    let src_dir = orchestrator_src();
    let cli_src_dir = cli_src();

    let scan_files: &[(&str, &str)] = &[
        ("orchestrator", "production_agent_rpc.rs"),
        ("orchestrator", "cost/automix_gate.rs"),
        ("orchestrator", "quarantine.rs"),
        ("orchestrator", "architecture/hook_prompt.rs"),
        ("orchestrator", "research_pipeline.rs"),
        ("orchestrator", "prompt.rs"),
        ("orchestrator", "agent_template.rs"),
        ("cli", "live_cli.rs"),
    ];

    for (crate_name, rel_path) in scan_files {
        let path = if *crate_name == "orchestrator" {
            src_dir.join(rel_path)
        } else {
            cli_src_dir.join(rel_path)
        };
        let src = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        assert!(
            !contains_banned_inline_prompt(&src),
            "BANNED inline prompt literal (>={} chars + role-framing marker) found in {}. \
             Prompts must live in prompts/*.md and be bound via include_str!.",
            INLINE_PROMPT_THRESHOLD,
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// (d) SELF-TEST: scanner fires on a synthetic >=200-char role-framing literal
//     (R2 — dead-evidence path must scream, wiring-enforcement.md)
// ---------------------------------------------------------------------------

#[test]
fn contract_scanner_self_test_detects_inline_prompt() {
    // Synthetic inline literal: 210 chars, contains "You are " role-framing marker.
    // This is NOT real production code — it is the value-sensitivity probe.
    // The string below is never reached by any production path; it exists only
    // to prove the scanner function above is live and sensitive.
    let synthetic_source = concat!(
        r#"const BAD: &str = "You are a test agent that MUST comply with all instructions. "#,
        r#"This is a synthetic inline prompt literal injected by the I-48 self-test arm. "#,
        r#"It exists only to prove the scanner fires. Do not remove this test.";"#,
    );

    // Verify the synthetic literal is actually >=200 chars (self-check the self-test)
    // The literal content is between the outer quotes in the synthetic source.
    let start = synthetic_source.find('"').expect("opening quote") + 1;
    let end = synthetic_source.rfind('"').expect("closing quote");
    let literal_len = end - start;
    assert!(
        literal_len >= INLINE_PROMPT_THRESHOLD,
        "self-test synthetic literal is only {literal_len} chars; must be >={INLINE_PROMPT_THRESHOLD}"
    );

    assert!(
        contains_banned_inline_prompt(synthetic_source),
        "scanner must detect the synthetic inline prompt literal (R2 self-test)"
    );
}

// ---------------------------------------------------------------------------
// (e) BYTE-BOUNDARY verification for TRICKY-class literals
//     (backslash-continuation — no trailing newline, no \r\n contamination)
// ---------------------------------------------------------------------------

#[test]
fn contract_tricky_literal_byte_boundaries() {
    let odir = orchestrator_prompts_dir();
    let cdir = cli_prompts_dir();

    // baby_step_system.md — starts with "You are an autonomous", ends with "edit has succeeded."
    let bss = std::fs::read(odir.join("baby_step_system.md"))
        .expect("baby_step_system.md must be readable");
    assert!(
        bss.starts_with(b"You are an autonomous"),
        "baby_step_system.md must start with 'You are an autonomous'"
    );
    assert!(
        bss.ends_with(b"edit has succeeded."),
        "baby_step_system.md must end with 'edit has succeeded.' (no trailing newline)"
    );
    assert!(
        !bss.contains(&b'\r'),
        "baby_step_system.md must contain no \\r bytes (CRLF contamination)"
    );

    // quarantine_system.md — starts with "You are a QUARANTINED", ends with "tool call."
    let qs = std::fs::read(odir.join("quarantine_system.md"))
        .expect("quarantine_system.md must be readable");
    assert!(
        qs.starts_with(b"You are a QUARANTINED"),
        "quarantine_system.md must start with 'You are a QUARANTINED'"
    );
    assert!(
        qs.ends_with(b"tool call."),
        "quarantine_system.md must end with 'tool call.' (no trailing newline)"
    );
    assert!(
        !qs.contains(&b'\r'),
        "quarantine_system.md must contain no \\r bytes"
    );

    // hook_prompt_handler.md — starts with "You are a strict-JSON", ends with "no fences."
    let hp = std::fs::read(odir.join("hook_prompt_handler.md"))
        .expect("hook_prompt_handler.md must be readable");
    assert!(
        hp.starts_with(b"You are a strict-JSON"),
        "hook_prompt_handler.md must start with 'You are a strict-JSON'"
    );
    assert!(
        hp.ends_with(b"no fences."),
        "hook_prompt_handler.md must end with 'no fences.' (no trailing newline)"
    );
    assert!(
        !hp.contains(&b'\r'),
        "hook_prompt_handler.md must contain no \\r bytes"
    );

    // live_chat_system.md — starts with "You are an interactive coding", ends with "concisely."
    let lc = std::fs::read(cdir.join("live_chat_system.md"))
        .expect("live_chat_system.md must be readable");
    assert!(
        lc.starts_with(b"You are an interactive coding"),
        "live_chat_system.md must start with 'You are an interactive coding'"
    );
    assert!(
        lc.ends_with(b"concisely."),
        "live_chat_system.md must end with 'concisely.' (no trailing newline)"
    );
    assert!(
        !lc.contains(&b'\r'),
        "live_chat_system.md must contain no \\r bytes"
    );
}
