//! J-25 T1 slice C — typed DigestSection contract (source-wiring gate).
//!
//! Verifies the structural + behavioral claims from the J-25 WIRING_CONTRACT
//! for the T1 typed-section template shipped in handoff_digest.rs:
//!
//! D1a: `handoff_digest.rs` exports `pub enum DigestSection`.
//! D1b: `lib.rs` re-exports `DigestSection` and `validate_discoveries_dedup`
//!      from the `handoff_digest` module.
//! D2:  production path — `build_digest_body` (called by `author_topic_digest`
//!      from live_cli.rs:3161) constructs `DigestSection` instances; asserted
//!      via source-grep of the production caller chain.
//! D3:  per-section budget caps match the J-25 plan (Intent=500, NextAction=1000,
//!      Directives=800, TaskTree=1000, CurrentWork=2000, Files=1500,
//!      Discoveries=2000, Errors=1500, LiveResources=1000, Decisions=3000,
//!      Notes=800) via `DigestSection::budget_tokens`.
//! D4:  `render_digest_sections` truncates over-budget bodies and the total
//!      stays under `MAX_DIGEST_TOKENS * CHARS_PER_TOKEN` (unit-logic).
//! D5:  `validate_discoveries_dedup` flags duplicate titles vs a prior set;
//!      clean when all titles are unique.
//! D6:  rendered headers are parseable by `read_budgeted_section_aware`
//!      (round-trip: render -> budgeted read keeps headers).
//!
//! The NEGATIVE leg (template never called from production => DARK) is locked
//! by D2 (slice C) together with the auditor's `production_caller_check.py`
//! run over the full T1 changeset. This slice locks the structural exports +
//! the deterministic behavior of the typed template + dedup validator.

use std::collections::HashSet;
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
// D1a: handoff_digest.rs exports pub enum DigestSection
// ---------------------------------------------------------------------------

/// D1a: handoff_digest.rs contains `pub enum DigestSection`.
#[test]
fn d1a_j25_digest_section_enum_exists() {
    let src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        src.contains("pub enum DigestSection"),
        "handoff_digest.rs must export pub enum DigestSection"
    );
}

// ---------------------------------------------------------------------------
// D1b: lib.rs re-exports DigestSection and validate_discoveries_dedup
// ---------------------------------------------------------------------------

/// D1b: lib.rs `pub use handoff_digest::{...}` block lists both
/// `DigestSection` and `validate_discoveries_dedup` so the contract test can
/// reach them as `elai_orchestrator::...`.
#[test]
fn d1b_j25_digest_section_reexported_from_lib() {
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
        block.contains("DigestSection"),
        "lib.rs must re-export DigestSection"
    );
    assert!(
        block.contains("validate_discoveries_dedup"),
        "lib.rs must re-export validate_discoveries_dedup"
    );
}

// ---------------------------------------------------------------------------
// D2: production path constructs DigestSection (source-grep)
// ---------------------------------------------------------------------------

/// D2: the production path `author_topic_digest` (called from
/// live_cli.rs:3161) calls `build_digest_body`, which constructs
/// `DigestSection` instances. Asserted via source-grep of the construction
/// site inside `build_digest_body` plus the live_cli.rs caller line.
#[test]
fn d2_production_path_constructs_digest_section() {
    // 1. live_cli.rs reaches the H-51 author path (production entrypoint).
    // T5 wrapped the direct call: `author_from_pending` delegates to
    // `author_topic_digest` (ABORT-1). Either symbol in the Drop path proves
    // the production author entrypoint is wired from live_cli.
    let cli_src = read_src("cli/src/live_cli.rs");
    assert!(
        cli_src.contains("author_topic_digest") || cli_src.contains("author_from_pending"),
        "live_cli.rs must call author_topic_digest or its T5 wrapper author_from_pending (production entrypoint)"
    );

    // 2. handoff_digest.rs: author_topic_digest calls build_digest_body.
    let digest_src = read_src("orchestrator/src/handoff_digest.rs");
    assert!(
        digest_src.contains("fn build_digest_body"),
        "handoff_digest.rs must define build_digest_body"
    );
    assert!(
        digest_src.contains("build_digest_body("),
        "author_topic_digest must call build_digest_body"
    );

    // 3. build_digest_body constructs DigestSection variants.
    let body_start = digest_src
        .find("fn build_digest_body")
        .expect("build_digest_body fn present");
    let body_end = digest_src[body_start..]
        .find("\n}\n")
        .map(|i| body_start + i + 3)
        .unwrap_or(digest_src.len());
    let body = &digest_src[body_start..body_end];
    assert!(
        body.contains("DigestSection::"),
        "build_digest_body must construct DigestSection variants; body was: {body}"
    );
}

// ---------------------------------------------------------------------------
// D3: per-section budget caps match the J-25 plan
// ---------------------------------------------------------------------------

/// D3: `DigestSection::budget_tokens` returns the plan-mandated caps for every
/// variant: Intent=500, NextAction=1000, Directives=800, TaskTree=1000,
/// CurrentWork=2000, Files=1500, Discoveries=2000, Errors=1500,
/// LiveResources=1000, Decisions=3000, Notes=800.
#[test]
fn d3_j25_digest_section_budget_caps_match_plan() {
    use elai_orchestrator::DigestSection;

    assert_eq!(DigestSection::Intent.budget_tokens(), 500);
    assert_eq!(DigestSection::NextAction.budget_tokens(), 1000);
    assert_eq!(DigestSection::Directives.budget_tokens(), 800);
    assert_eq!(DigestSection::TaskTree.budget_tokens(), 1000);
    assert_eq!(DigestSection::CurrentWork.budget_tokens(), 2000);
    assert_eq!(DigestSection::Files.budget_tokens(), 1500);
    assert_eq!(DigestSection::Discoveries.budget_tokens(), 2000);
    assert_eq!(DigestSection::Errors.budget_tokens(), 1500);
    assert_eq!(DigestSection::LiveResources.budget_tokens(), 1000);
    assert_eq!(DigestSection::Decisions.budget_tokens(), 3000);
    assert_eq!(DigestSection::Notes.budget_tokens(), 800);
}

// ---------------------------------------------------------------------------
// D4: render_digest_sections truncates over-budget body; total under cap
// ---------------------------------------------------------------------------

/// D4a: a single section with a body exceeding its per-section char cap is
/// truncated by `render_digest_sections` to at most `budget_chars()` chars
/// of body text (header excluded).
#[test]
fn d4a_j25_render_digest_sections_truncates_over_budget_body() {
    use elai_orchestrator::{DigestSection, render_digest_sections};

    let cap_chars = DigestSection::Intent.budget_chars();
    let over_body = "x".repeat(cap_chars + 500);
    let sections = vec![(DigestSection::Intent, over_body)];
    let out = render_digest_sections(&sections);
    assert!(
        out.starts_with("INTENT\n"),
        "rendered output must start with the INTENT header"
    );
    let body = &out["INTENT\n".len()..];
    assert!(
        body.len() <= cap_chars,
        "body len ({}) must be <= Intent.budget_chars() ({})",
        body.len(),
        cap_chars
    );
}

/// D4b: the total rendered length stays under
/// `MAX_DIGEST_TOKENS * CHARS_PER_TOKEN` even when multiple sections emit
/// bodies far over their individual caps.
#[test]
fn d4b_j25_render_digest_sections_total_under_max() {
    use elai_orchestrator::{DigestSection, render_digest_sections};

    // MAX_DIGEST_TOKENS = 4000, CHARS_PER_TOKEN = 4 => total cap = 16000 chars.
    // We cannot import the private consts, but the contract is documented in
    // handoff_digest.rs and we assert the documented 16000 ceiling.
    const MAX_DIGEST_TOKENS: usize = 4_000;
    const CHARS_PER_TOKEN: usize = 4;
    let total_cap = MAX_DIGEST_TOKENS * CHARS_PER_TOKEN;

    let huge = "y".repeat(20_000);
    let sections = vec![
        (DigestSection::Decisions, huge.clone()),
        (DigestSection::Files, huge.clone()),
        (DigestSection::CurrentWork, huge.clone()),
        (DigestSection::Discoveries, huge),
    ];
    let out = render_digest_sections(&sections);
    assert!(
        out.len() <= total_cap,
        "total rendered len ({}) must be <= MAX_DIGEST_TOKENS*CHARS_PER_TOKEN ({})",
        out.len(),
        total_cap
    );
}

// ---------------------------------------------------------------------------
// D5: validate_discoveries_dedup flags duplicates; clean when unique
// ---------------------------------------------------------------------------

/// D5a: `validate_discoveries_dedup` flags titles in `current_titles` that
/// already appear in `prior_titles` and reports `has_duplicates == true`.
#[test]
fn d5a_j25_validate_discoveries_dedup_flags_duplicates() {
    use elai_orchestrator::validate_discoveries_dedup;

    let mut prior: HashSet<String> = HashSet::new();
    prior.insert("known finding".to_string());
    prior.insert("prior art".to_string());
    let current = vec![
        "known finding".to_string(),
        "new finding".to_string(),
        "prior art".to_string(),
    ];
    let verdict = validate_discoveries_dedup(&current, &prior);
    assert!(
        verdict.has_duplicates,
        "has_duplicates must be true when prior overlaps current"
    );
    assert_eq!(
        verdict.duplicates,
        vec!["known finding", "prior art"],
        "duplicates must list the overlapping titles in encounter order"
    );
}

/// D5b: `validate_discoveries_dedup` reports `has_duplicates == false` and an
/// empty duplicates vec when every current title is unique vs the prior set.
#[test]
fn d5b_j25_validate_discoveries_dedup_clean_when_unique() {
    use elai_orchestrator::validate_discoveries_dedup;

    let mut prior: HashSet<String> = HashSet::new();
    prior.insert("old A".to_string());
    let current = vec![
        "fresh finding".to_string(),
        "another fresh finding".to_string(),
    ];
    let verdict = validate_discoveries_dedup(&current, &prior);
    assert!(
        !verdict.has_duplicates,
        "has_duplicates must be false when no current title is in prior"
    );
    assert!(
        verdict.duplicates.is_empty(),
        "duplicates must be empty when no overlap"
    );
}

// ---------------------------------------------------------------------------
// D6: rendered headers parseable by read_budgeted_section_aware (round-trip)
// ---------------------------------------------------------------------------

/// D6: `render_digest_sections` emits ALL-CAPS headers that
/// `read_budgeted_section_aware` recognizes as section headers, so a
/// round-trip (render -> budgeted read) preserves every header in order.
#[test]
fn d6_j25_render_round_trips_through_budgeted_read() {
    use elai_orchestrator::{DigestSection, read_budgeted_section_aware, render_digest_sections};

    let sections = vec![
        (DigestSection::Intent, "session abc for topic demo".to_string()),
        (DigestSection::NextAction, "run elai /continue".to_string()),
        (DigestSection::Decisions, "- decide X\n- decide Y\n".to_string()),
        (DigestSection::Files, "- src/a.rs\n- src/b.rs\n".to_string()),
        (DigestSection::Notes, "follow up on Z\n".to_string()),
    ];
    let rendered = render_digest_sections(&sections);

    // Budget large enough to fit everything (no truncation).
    let result = read_budgeted_section_aware(&rendered, 5_000);

    let headers: Vec<&str> = result.sections.iter().map(|s| s.header.as_str()).collect();
    assert_eq!(
        headers,
        vec!["INTENT", "NEXT ACTION", "DECISIONS", "FILES TOUCHED", "NOTES"],
        "round-trip must preserve every section header in order"
    );

    // Bodies must be non-empty for each populated section.
    for s in &result.sections {
        assert!(
            !s.body.is_empty(),
            "round-trip body must be non-empty for header={}; got {:?}",
            s.header,
            s.body
        );
    }

    // No truncation under a generous budget.
    assert!(
        !result.truncated,
        "round-trip must not truncate under a generous budget"
    );
}
