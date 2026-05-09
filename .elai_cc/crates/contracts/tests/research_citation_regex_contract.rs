// Track A.3 contract test: citation-counting regex \[\d+\] in research_pipeline.rs.
//
// GREEN STATE (pre-patch): A.3 was implemented in session s-1928380. The
// has_numeric_citation() fn and NUMERIC_CITATION_RE LazyLock already exist at
// research_pipeline.rs:28-34. This file is the missing contract test that was
// called for in the Track A prompt.
//
// has_numeric_citation is private (not pub). Per the task constraints, the
// smallest surface change to make it testable from a contract test is a
// #[cfg(test)] pub(crate) re-export in research_pipeline.rs:
//
//   #[cfg(test)]
//   pub use has_numeric_citation as has_numeric_citation_for_test;
//
// OR we can call through the public phase_f_synthesize path with crafted
// inputs and observe the fallback / non-fallback branch.
//
// Preferred approach: add a thin #[cfg(test)] wrapper module in
// research_pipeline.rs. Document that as the only surface change.
// If the wrapper is not yet added, this test references
// `elai_orchestrator::research_pipeline::has_numeric_citation_for_test` which
// will fail to compile (RED STATE, intentional).
//
// Surface change required in production code (research_pipeline.rs):
//   // At the bottom of the file, inside #[cfg(test)]:
//   #[cfg(test)]
//   pub mod test_helpers {
//       pub use super::has_numeric_citation;
//   }
// That is the smallest possible change: one pub use inside a cfg(test) block.

use elai_orchestrator::research_pipeline::test_helpers::has_numeric_citation;

// ---------------------------------------------------------------------------
// A.3.a — pattern matches exactly [N] and rejects non-numeric brackets.
// ---------------------------------------------------------------------------

#[test]
fn citation_regex_rejects_non_numeric_brackets() {
    assert!(
        !has_numeric_citation("[some text]"),
        "'[some text]' must not count as a citation — only [N] patterns qualify",
    );
}

#[test]
fn citation_regex_counts_single_numeric_marker() {
    assert!(
        has_numeric_citation("[3]"),
        "'[3]' must be recognised as a citation marker",
    );
}

#[test]
fn citation_regex_counts_multiple_numeric_markers() {
    // [3] and [42] both present — at least one match is sufficient.
    assert!(
        has_numeric_citation("[3] [42]"),
        "'[3] [42]' must be recognised as containing citation markers",
    );
}

#[test]
fn citation_regex_rejects_bare_text() {
    assert!(
        !has_numeric_citation("no brackets"),
        "plain text with no brackets must not match",
    );
}

#[test]
fn citation_regex_rejects_qwen_instruction_tokens() {
    // Qwen 3B emits [INST] and [INFO] — these must NOT be treated as citations.
    assert!(
        !has_numeric_citation("[INST] do something"),
        "[INST] must not be accepted as a citation marker",
    );
    assert!(
        !has_numeric_citation("[INFO]"),
        "[INFO] must not be accepted as a citation marker",
    );
}

#[test]
fn citation_regex_rejects_empty_brackets() {
    assert!(
        !has_numeric_citation("[]"),
        "empty brackets must not match \\[\\d+\\]",
    );
}

#[test]
fn citation_regex_accepts_two_digit_markers() {
    // [12] is a valid citation reference in research prose.
    assert!(
        has_numeric_citation("[12]"),
        "[12] must be recognised as a numeric citation marker",
    );
}
