//! J-25 T1 typed digest-section template — impl + helpers + dedup validator.
//!
//! Split out of `handoff_digest.rs` (MR-1: parent file crossed the 1000-line
//! cap). The `pub enum DigestSection` definition stays in the parent module so
//! the J-25 WIRING_CONTRACT D1a source-grep (`pub enum DigestSection` in
//! `handoff_digest.rs`) keeps passing; this submodule holds the impl block and
//! the free functions the contract test D1b/D3/D4/D5/D6 exercise through the
//! `pub use digest_section::{...}` re-export in the parent.
//!
//! Deterministic; NO LLM, NO I/O, NO network.

use super::{CHARS_PER_TOKEN, MAX_DIGEST_TOKENS, truncate_lines_to_char_budget};
use super::DigestSection;

impl DigestSection {
    /// The ALL-CAPS header line written to the digest file.  Compatible with
    /// [`super::is_section_header`] so [`super::read_budgeted_section_aware`]
    /// keeps parsing sections emitted by the typed template.
    pub fn header(&self) -> &'static str {
        match self {
            Self::Intent => "INTENT",
            Self::NextAction => "NEXT ACTION",
            Self::Directives => "DIRECTIVES",
            Self::TaskTree => "TASK TREE",
            Self::CurrentWork => "CURRENT WORK",
            Self::Files => "FILES TOUCHED",
            Self::Discoveries => "DISCOVERIES",
            Self::Errors => "ERRORS",
            Self::LiveResources => "LIVE RESOURCES",
            Self::Decisions => "DECISIONS",
            Self::Notes => "NOTES",
            Self::MemoryTaint => "MEMORY TAINT",
        }
    }

    /// Per-section token-budget CAP (token units, plan T1).
    pub fn budget_tokens(&self) -> usize {
        match self {
            Self::Intent => 500,
            Self::NextAction => 1000,
            Self::Directives => 800,
            Self::TaskTree => 1000,
            Self::CurrentWork => 2000,
            Self::Files => 1500,
            Self::Discoveries => 2000,
            Self::Errors => 1500,
            Self::LiveResources => 1000,
            Self::Decisions => 3000,
            Self::Notes => 800,
            Self::MemoryTaint => 800,
        }
    }

    /// Per-section char-budget CAP (token cap * [`CHARS_PER_TOKEN`]).
    pub fn budget_chars(&self) -> usize {
        self.budget_tokens().saturating_mul(CHARS_PER_TOKEN)
    }

    /// Render this section as `HEADER\n` + body, truncating the body to
    /// [`DigestSection::budget_chars`] chars on a line boundary.
    pub fn render(&self, body: &str) -> String {
        let cap = self.budget_chars();
        let truncated = truncate_lines_to_char_budget(body, cap);
        format!("{}\n{}", self.header(), truncated)
    }
}

/// Render a sequence of typed sections to a single markdown body String,
/// truncating each section to its per-section char cap and the total to
/// [`MAX_DIGEST_TOKENS`] chars.  Sections are separated by blank lines.
///
/// Deterministic; NO LLM, NO I/O.  The output is plain markdown with
/// ALL-CAPS headers, compatible with [`super::read_budgeted_section_aware`].
pub fn render_digest_sections(sections: &[(DigestSection, String)]) -> String {
    let mut out = String::new();
    let mut total_chars: usize = 0;
    let total_cap = MAX_DIGEST_TOKENS.saturating_mul(CHARS_PER_TOKEN);

    for (section, body) in sections {
        let header = section.header();
        let separator = if out.is_empty() { 0 } else { 2 };
        let overhead = separator + header.len() + 1;
        let remaining = total_cap.saturating_sub(total_chars);
        if remaining < overhead {
            break;
        }
        let body_cap = remaining - overhead;
        let section_cap = section.budget_chars().min(body_cap);
        let truncated_body = truncate_lines_to_char_budget(body, section_cap);
        if !out.is_empty() {
            out.push_str("\n\n");
        }
        out.push_str(header);
        out.push('\n');
        out.push_str(&truncated_body);
        total_chars = total_chars.saturating_add(overhead + truncated_body.len());
        if total_chars >= total_cap {
            break;
        }
    }

    out
}

/// Extract discovery titles from a `DISCOVERIES` body.  A discovery entry is
/// a markdown bullet `- <title>: <rest>` or `- <title>`; the title is the text
/// up to the first `:` (trimmed) or the whole bullet text when no colon is
/// present.  Deterministic; NO LLM, NO I/O.
pub fn extract_discovery_titles(discoveries_body: &str) -> Vec<String> {
    let mut titles: Vec<String> = Vec::new();
    for line in discoveries_body.lines() {
        let trimmed = line.trim_start();
        let rest = match trimmed.strip_prefix("- ") {
            Some(r) => r,
            None => continue,
        };
        let title = match rest.split_once(':') {
            Some((t, _)) => t.trim(),
            None => rest.trim(),
        };
        if !title.is_empty() {
            titles.push(title.to_owned());
        }
    }
    titles
}

/// Verdict from [`validate_discoveries_dedup`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DedupVerdict {
    /// Titles in `current_titles` that already appear in `prior_titles`,
    /// in the order encountered.
    pub duplicates: Vec<String>,
    /// `true` iff at least one duplicate was found.
    pub has_duplicates: bool,
}

/// Deterministic discoveries-dedup validator.  Flags discovery titles in
/// `current_titles` that already appear in `prior_titles` (case-sensitive,
/// trimmed).  NO LLM, NO I/O, NO network.
pub fn validate_discoveries_dedup(
    current_titles: &[String],
    prior_titles: &std::collections::HashSet<String>,
) -> DedupVerdict {
    let mut duplicates: Vec<String> = Vec::new();
    for title in current_titles {
        let trimmed = title.trim();
        if !trimmed.is_empty() && prior_titles.contains(trimmed) {
            duplicates.push(trimmed.to_owned());
        }
    }
    let has_duplicates = !duplicates.is_empty();
    DedupVerdict {
        duplicates,
        has_duplicates,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digest_section_header_and_budget() {
        assert_eq!(DigestSection::Intent.header(), "INTENT");
        assert_eq!(DigestSection::Intent.budget_tokens(), 500);
        assert_eq!(DigestSection::NextAction.header(), "NEXT ACTION");
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

    #[test]
    fn test_digest_section_render_truncates_to_cap() {
        let long_body = "x".repeat(DigestSection::Intent.budget_chars() + 100);
        let rendered = DigestSection::Intent.render(&long_body);
        assert!(rendered.starts_with("INTENT\n"));
        let body_part = &rendered["INTENT\n".len()..];
        assert!(body_part.len() <= DigestSection::Intent.budget_chars());
    }

    #[test]
    fn test_render_digest_sections_emits_headers() {
        let sections = vec![
            (DigestSection::Intent, "session abc".to_string()),
            (DigestSection::Decisions, "- decided X\n".to_string()),
        ];
        let out = render_digest_sections(&sections);
        assert!(out.contains("INTENT\nsession abc"));
        assert!(out.contains("DECISIONS\n- decided X"));
    }

    #[test]
    fn test_render_digest_sections_total_under_max() {
        let big = "y".repeat(10_000);
        let sections = vec![
            (DigestSection::Decisions, big.clone()),
            (DigestSection::Files, big.clone()),
            (DigestSection::CurrentWork, big),
        ];
        let out = render_digest_sections(&sections);
        assert!(out.len() <= MAX_DIGEST_TOKENS * CHARS_PER_TOKEN);
    }

    #[test]
    fn test_extract_discovery_titles() {
        let body = "- First finding: blah\n- Second finding\n- Third: x\n";
        let titles = extract_discovery_titles(body);
        assert_eq!(titles, vec!["First finding", "Second finding", "Third"]);
    }

    #[test]
    fn test_validate_discoveries_dedup_flags_duplicates() {
        let mut prior = std::collections::HashSet::new();
        prior.insert("known finding".to_string());
        let current = vec!["known finding".to_string(), "new finding".to_string()];
        let verdict = validate_discoveries_dedup(&current, &prior);
        assert!(verdict.has_duplicates);
        assert_eq!(verdict.duplicates, vec!["known finding"]);
    }

    #[test]
    fn test_validate_discoveries_dedup_no_duplicates() {
        let prior = std::collections::HashSet::new();
        let current = vec!["unique finding".to_string()];
        let verdict = validate_discoveries_dedup(&current, &prior);
        assert!(!verdict.has_duplicates);
        assert!(verdict.duplicates.is_empty());
    }
}
