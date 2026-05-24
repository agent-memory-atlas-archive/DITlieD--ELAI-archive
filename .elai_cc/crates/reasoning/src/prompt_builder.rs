// Prompt builder: inject Project Intelligence convention blocks into the
// SYSTEM prompt section.
//
// Spec phase-3L Step 4 / M10:
//   reachable_from: "prompt construction in reasoning/src/prompt_builder.rs
//   (CONVENTIONS block)"
//
// This module is intentionally tiny — the heavy lifting (extracting
// conventions, picking anti-patterns, fitting the 200-token budget) is
// owned by `elai_knowledge::project_intelligence` so the reasoning crate
// does not couple to the convention extractor.
//
// Anti-rules guarded:
//   - NO new dependencies; reasoning already depends on elai_core. To
//     consume the PI hook surface we route through plain `String` outputs
//     of `ProjectIntelligenceHook::render_block`, which the caller (CLI
//     dispatcher, orchestrator) produces and hands to us. This keeps the
//     reasoning crate cycle-free.
//   - 200-token cap upheld by truncation at a char-boundary (matches the
//     `TelescopeContext::estimated_tokens` rule chars/4 ≈ tokens).
//   - Idempotent: building with the same inputs twice yields byte-equal
//     output.

/// Token-budget cap for the CONVENTIONS block (spec Step 4).
/// Matches `elai_knowledge::PI_T1_TOKEN_BUDGET` to avoid double-truncation
/// when both consumers stack.
pub const CONVENTIONS_PROMPT_BUDGET_TOKENS: usize = 200;

/// Marker that callers (auditor, orchestrator) can grep on to confirm the
/// SYSTEM section saw the CONVENTIONS injection.
pub const CONVENTIONS_BLOCK_HEADER: &str = "CONVENTIONS";

/// Build a SYSTEM-section prompt by injecting a pre-rendered CONVENTIONS
/// block (typically produced via
/// `elai_knowledge::PiT1Hook::render_block(profile)`) on top of an
/// existing base prompt.
///
/// Idempotent: if `base_prompt` already contains the CONVENTIONS header
/// the block is not re-appended.
///
/// Returns the merged prompt, truncated to `CONVENTIONS_PROMPT_BUDGET_TOKENS
/// * 4` chars for the injected block ONLY (the base_prompt is preserved
/// verbatim).
pub fn build_with_conventions(conventions_block: Option<&str>, base_prompt: &str) -> String {
    let Some(block) = conventions_block else {
        return base_prompt.to_owned();
    };
    if base_prompt.contains(CONVENTIONS_BLOCK_HEADER) {
        // Idempotency: do not re-inject.
        return base_prompt.to_owned();
    }
    let trimmed = truncate_to_budget(block, CONVENTIONS_PROMPT_BUDGET_TOKENS * 4);
    let mut out = String::with_capacity(trimmed.len() + base_prompt.len() + 2);
    out.push_str(&trimmed);
    if !trimmed.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(base_prompt);
    out
}

fn truncate_to_budget(block: &str, max_chars: usize) -> String {
    if block.len() <= max_chars {
        return block.to_owned();
    }
    let mut boundary = max_chars;
    while boundary > 0 && !block.is_char_boundary(boundary) {
        boundary -= 1;
    }
    let mut trimmed = block[..boundary].to_owned();
    if let Some(last_nl) = trimmed.rfind('\n') {
        trimmed.truncate(last_nl + 1);
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_without_conventions_is_passthrough() {
        let out = build_with_conventions(None, "BASE PROMPT");
        assert_eq!(out, "BASE PROMPT");
    }

    #[test]
    fn build_with_conventions_prepends_block() {
        let block = "CONVENTIONS\n  fn:snake_case (strong)\n";
        let out = build_with_conventions(Some(block), "BASE\n");
        assert!(out.starts_with("CONVENTIONS\n"));
        assert!(out.contains("BASE"));
    }

    #[test]
    fn build_with_conventions_is_idempotent() {
        let block = "CONVENTIONS\n  fn:snake_case (strong)\n";
        let base = "CONVENTIONS\n  fn:snake_case (strong)\nBASE\n";
        // Idempotency guard: don't double-inject when base already carries
        // a CONVENTIONS header.
        let out = build_with_conventions(Some(block), base);
        assert_eq!(out, base);
    }

    #[test]
    fn build_with_conventions_respects_char_budget() {
        // 1000 char "CONVENTIONS" block, capped to budget.
        let big = format!("CONVENTIONS\n{}\n", "x".repeat(10_000));
        let out = build_with_conventions(Some(&big), "BASE");
        let injected_len = out.find("BASE").unwrap_or(out.len());
        assert!(
            injected_len <= CONVENTIONS_PROMPT_BUDGET_TOKENS * 4 + 1,
            "injected block length {injected_len} exceeded budget"
        );
    }

    #[test]
    fn build_with_conventions_handles_empty_base() {
        let block = "CONVENTIONS\n  fn:snake_case (strong)\n";
        let out = build_with_conventions(Some(block), "");
        assert!(out.starts_with("CONVENTIONS\n"));
    }
}
