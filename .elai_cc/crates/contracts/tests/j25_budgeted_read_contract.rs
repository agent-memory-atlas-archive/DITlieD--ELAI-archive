//! J-25 T2 — section-aware budgeted read contract (source-wiring gate).
//!
//! Verifies the structural claims from the J-25 WIRING_CONTRACT for the T2
//! slice (`read_budgeted_section_aware` loader inside handoff_digest.rs):
//!
//! D1a: `handoff_digest.rs` exports `pub fn read_budgeted_section_aware`,
//!      `pub struct BudgetedSection`, `pub struct BudgetedReadResult`.
//! D1b: `lib.rs` re-exports all three from `handoff_digest` (module reachable
//!      from the crate root as `elai_orchestrator::...`).
//! D2:  repl.rs H-51 TieredLoad inject arm calls
//!      `read_budgeted_section_aware` (R6/R7 production-reachability leg).
//! D3:  skeleton-always-survives — `budget_tokens=0` still returns every
//!      section header (unit-logic via the public API).
//! D4:  body fills in section order with the 0.95 safety factor — the first
//!      section's body is filled before the second, and `loaded_tokens`
//!      respects `floor(budget * 0.95) + skeleton` (unit-logic).
//! D5:  overflow sets `truncated=true` and `resume_hint = Some(...)` naming a
//!      1-based line number (unit-logic).
//!
//! The NEGATIVE leg (loader never called from production => DARK) is locked
//! by D2 (slice C) together with the auditor's `production_caller_check.py`
//! run over the full T2 changeset. This slice (B) locks the structural
//! exports + the loader's deterministic behavior; slice C locks the wire.

use std::fs;
use std::path::PathBuf;

fn crates_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/contracts has a parent")
        .to_path_buf()
}

fn read_src(rel: &str) -> String {
    let path = crates_dir().join(rel);
    fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

// ---------------------------------------------------------------------------
// D1a: handoff_digest.rs exports the fn + structs
// ---------------------------------------------------------------------------

/// D1a: handoff_digest.rs exports `pub fn read_budgeted_section_aware`,
/// `pub struct BudgetedSection`, `pub struct BudgetedReadResult`.
#[test]
fn d1a_j25_budgeted_read_symbols_exist() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        src.contains("pub fn read_budgeted_section_aware"),
        "handoff_digest.rs must export pub fn read_budgeted_section_aware"
    );
    assert!(
        src.contains("pub struct BudgetedSection"),
        "handoff_digest.rs must export pub struct BudgetedSection"
    );
    assert!(
        src.contains("pub struct BudgetedReadResult"),
        "handoff_digest.rs must export pub struct BudgetedReadResult"
    );
}

// ---------------------------------------------------------------------------
// D1b: lib.rs re-exports all three from handoff_digest
// ---------------------------------------------------------------------------

/// D1b: lib.rs re-exports `read_budgeted_section_aware`, `BudgetedSection`,
/// `BudgetedReadResult` from the `handoff_digest` module so the contract
/// test can reach them as `elai_orchestrator::...`.
#[test]
fn d1b_j25_budgeted_read_reexported_from_lib() {
    let src = read_src("orchestrator/src/lib.rs");
    let block_start = src.find("pub use handoff_digest::{");
    assert!(
        block_start.is_some(),
        "lib.rs must have a handoff_digest pub use block"
    );
    let block = &src[block_start.unwrap()..];
    let block_end = block.find("};").expect("pub use block terminates");
    let block = &block[..block_end];
    assert!(
        block.contains("read_budgeted_section_aware"),
        "lib.rs must re-export read_budgeted_section_aware"
    );
    assert!(
        block.contains("BudgetedSection"),
        "lib.rs must re-export BudgetedSection"
    );
    assert!(
        block.contains("BudgetedReadResult"),
        "lib.rs must re-export BudgetedReadResult"
    );
}

// ---------------------------------------------------------------------------
// D2: repl.rs tiered-load arm calls read_budgeted_section_aware (R6/R7 wire)
// ---------------------------------------------------------------------------

/// D2: repl.rs H-51 TieredLoad inject arm calls
/// `read_budgeted_section_aware` inside the
/// `else if let Ok(digest_text) = std::fs::read_to_string(&handle.digest_path)`
/// branch. This is the R6/R7 production-reachability leg: the loader must have
/// a non-test production caller, and this is the smallest justified seam.
#[test]
fn d2_repl_tiered_load_calls_read_budgeted_section_aware() {
    let src = read_src("cli/src/repl.rs");
    let arm_start = src.find("else if let Ok(digest_text) = std::fs::read_to_string(&handle.digest_path)");
    assert!(
        arm_start.is_some(),
        "repl.rs must contain the TieredLoad inject arm \
         (`else if let Ok(digest_text) = std::fs::read_to_string(&handle.digest_path)`)"
    );
    let body = &src[arm_start.unwrap()..];
    // The arm ends at the next closing brace at the same indent as the `else if`.
    // For a robust check, scan to the matching `}` that closes the `else if` arm.
    // We just assert the call appears between the arm start and the next
    // `cli.system_prompt.push(preamble);` line (the arm's body terminator).
    let arm_end = body
        .find("cli.system_prompt.push(preamble);")
        .map(|i| i + "cli.system_prompt.push(preamble);".len())
        .unwrap_or(body.len());
    let arm_body = &body[..arm_end];
    assert!(
        arm_body.contains("read_budgeted_section_aware"),
        "repl.rs TieredLoad inject arm must call read_budgeted_section_aware; \
         arm body was: {arm_body}"
    );
}

// ---------------------------------------------------------------------------
// D3: skeleton-always-survives — budget_tokens=0 still returns every header
// ---------------------------------------------------------------------------

/// D3: with `budget_tokens=0`, every section header in the digest appears in
/// `result.sections` (the skeleton always survives). No body content is
/// required; only the headers must be present, in order.
#[test]
fn d3_j25_budgeted_read_skeleton_always_survives() {
    use elai_orchestrator::read_budgeted_section_aware;

    let digest = "\
SUMMARY
this is the summary body line one
this is the summary body line two
DECISIONS
decision one: do X
decision two: do Y
FILES TOUCHED
file_a.rs
file_b.rs
HOW TO PICK UP
step one
step two
";
    let result = read_budgeted_section_aware(digest, 0);
    let headers: Vec<&str> = result.sections.iter().map(|s| s.header.as_str()).collect();
    assert_eq!(
        headers,
        vec!["SUMMARY", "DECISIONS", "FILES TOUCHED", "HOW TO PICK UP"],
        "budget_tokens=0 must still return every section header in order"
    );
    // With zero budget, bodies must be empty (skeleton only).
    for s in &result.sections {
        assert!(
            s.body.is_empty(),
            "budget_tokens=0 must produce empty bodies (header={}); got {:?}",
            s.header,
            s.body
        );
    }
    assert!(
        !result.truncated,
        "budget_tokens=0 with no body content emitted must not set truncated"
    );
}

// ---------------------------------------------------------------------------
// D4: body fills in section order with 0.95 safety factor
// ---------------------------------------------------------------------------

/// D4: with a budget that fits the first section's body but not the second,
/// the first section's body is filled and the second is truncated. The total
/// `loaded_tokens` respects `floor(budget * 0.95) + skeleton_tokens`.
#[test]
fn d4_j25_budgeted_read_body_fills_in_section_order() {
    use elai_orchestrator::read_budgeted_section_aware;

    // Section 1 body: 4 lines of 40 chars each = ~160 chars = ~40 tokens.
    // Section 2 body: 4 lines of 40 chars each = ~160 chars = ~40 tokens.
    // Skeleton: 2 headers of ~10 chars each + newlines = ~24 chars = ~6 tokens.
    // budget_tokens = 50 => body budget = floor(50 * 0.95) = 47 tokens = 188 chars.
    // Section 1 body (160 chars + 4 newlines = 164 chars) fits within 188.
    // Section 2 body does NOT fit (164 > 188-164=24 remaining).
    let long_line = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 40 chars
    let digest = format!(
        "SUMMARY\n{long_line}\n{long_line}\n{long_line}\n{long_line}\nDECISIONS\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n"
    );
    let budget: u32 = 50;
    let result = read_budgeted_section_aware(&digest, budget);

    // Section 1 body must be fully filled (all 4 lines).
    let s1 = &result.sections[0];
    assert_eq!(s1.header, "SUMMARY");
    assert_eq!(
        s1.body.lines().count(),
        4,
        "section 1 body must be fully filled (4 lines), got: {:?}",
        s1.body
    );

    // Section 2 body must be truncated (fewer than 4 lines).
    let s2 = &result.sections[1];
    assert_eq!(s2.header, "DECISIONS");
    assert!(
        s2.body.lines().count() < 4,
        "section 2 body must be truncated (fewer than 4 lines), got: {:?}",
        s2.body
    );

    // loaded_tokens must respect floor(budget * 0.95) + skeleton_tokens.
    // body_budget_tokens = floor(50 * 0.95) = 47.
    // skeleton_tokens = headers + newlines assembled length / 4.
    // The assembled text = headers + bodies; loaded_tokens = assembled_len / 4.
    // The body portion consumed <= body_budget_tokens = 47.
    // So loaded_tokens <= 47 + skeleton_tokens.
    let body_budget_tokens = ((budget as f64) * 0.95).floor() as u32;
    let skeleton_chars: usize = result
        .sections
        .iter()
        .map(|s| s.header.len() + 1 + 1) // header + \n + \n separator
        .sum();
    let skeleton_tokens = skeleton_chars / 4;
    assert!(
        result.loaded_tokens <= body_budget_tokens + skeleton_tokens as u32,
        "loaded_tokens ({}) must respect floor(budget*0.95) ({}) + skeleton_tokens ({})",
        result.loaded_tokens,
        body_budget_tokens,
        skeleton_tokens
    );
}

// ---------------------------------------------------------------------------
// D5: overflow sets truncated=true and resume_hint=Some("line N")
// ---------------------------------------------------------------------------

/// D5: with a digest larger than the budget, `truncated == true` and
/// `resume_hint` is `Some(...)` naming a 1-based line number.
#[test]
fn d5_j25_budgeted_read_overflow_sets_truncated_and_resume_hint() {
    use elai_orchestrator::read_budgeted_section_aware;

    let long_line = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"; // 64 chars
    let digest = format!(
        "SUMMARY\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n{long_line}\n"
    );
    // budget = 10 tokens => body budget = floor(10 * 0.95) = 9 tokens = 36 chars.
    // The first body line (64 chars) does NOT fit in 36 chars => truncated.
    let result = read_budgeted_section_aware(&digest, 10);
    assert!(
        result.truncated,
        "overflow must set truncated=true; got truncated={}",
        result.truncated
    );
    let hint = result
        .resume_hint
        .as_ref()
        .expect("overflow must set resume_hint to Some(...)");
    assert!(
        hint.starts_with("line "),
        "resume_hint must name a 1-based line number (\"line N\"), got: {:?}",
        hint
    );
    // The line number must be parseable as a usize >= 1.
    let n: usize = hint["line ".len()..]
        .parse()
        .unwrap_or_else(|e| panic!("resume_hint line number must parse as usize: {hint:?} ({e})"));
    assert!(n >= 1, "resume_hint line number must be >= 1, got {n}");
    assert!(n <= digest.lines().count(), "resume_hint line number ({n}) must be within the digest's line count ({})", digest.lines().count());
}
