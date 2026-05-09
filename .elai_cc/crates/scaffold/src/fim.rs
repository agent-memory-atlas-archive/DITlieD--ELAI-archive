//! Fill-In-the-Middle (FIM) section discovery + Qwen prompt construction.
//!
//! Templates ship with HTML-comment markers around the section body that
//! a model is expected to fill:
//!
//! ```text
//! <!-- FIM_PREFIX: Hero section skeleton -->
//! <section data-section="hero">
//!   <!-- FIM_FILL_START -->
//!   <h1>{{project_name}}</h1>
//!   <p>{{description}}</p>
//!   <!-- FIM_FILL_END -->
//! </section>
//! <!-- FIM_SUFFIX: Hero section skeleton -->
//! ```
//!
//! Vite-React (.jsx) and Astro (.astro) wrap the same markers in JSX
//! block comments (`{/* FIM_FILL_START */}`); Vite-Svelte (.svelte) uses
//! `<!--` HTML comments inside the markup section. The discovery routine
//! treats anything matching the literal token `FIM_FILL_START` as the
//! opener, so all three comment dialects are accepted with one parser.
//!
//! The Qwen2.5-Coder model exposes FIM via three special tokens that wrap
//! the prompt body. We assemble the prompt here and let the sidecar's
//! `/v1/completions` endpoint forward it to the engine. The sidecar does
//! not currently understand a `stop` parameter, so we strip any leaked
//! `<|fim_pad|>` / `<|endoftext|>` artifacts post-hoc.

use thiserror::Error;

/// Sentinel token marking the start of the section to fill. Matched as a
/// literal substring; the surrounding comment syntax does not matter.
pub const FIM_TOKEN_FILL_START: &str = "FIM_FILL_START";

/// Sentinel token marking the end of the section to fill.
pub const FIM_TOKEN_FILL_END: &str = "FIM_FILL_END";

/// Qwen2.5-Coder FIM special tokens. Verified against the upstream
/// tokeniser; ordering is `<prefix><body_before><suffix><body_after><middle>`.
pub const QWEN_FIM_PREFIX: &str = "<|fim_prefix|>";
pub const QWEN_FIM_SUFFIX: &str = "<|fim_suffix|>";
pub const QWEN_FIM_MIDDLE: &str = "<|fim_middle|>";
pub const QWEN_FIM_PAD: &str = "<|fim_pad|>";

/// EOS token Qwen models emit at end of generation. Stripped post-hoc
/// because the sidecar `/v1/completions` endpoint does not honour a
/// `stop` parameter today.
pub const QWEN_END_OF_TEXT: &str = "<|endoftext|>";

/// Discovered FIM region. `before` includes everything up to and
/// including the line that contains `FIM_FILL_START`; `after` includes
/// the line that contains `FIM_FILL_END` and everything after; the
/// `current_middle` is whatever was between them at parse time (the
/// template's placeholder content).
#[derive(Debug, Clone)]
pub struct FimSplice {
    pub before: String,
    pub current_middle: String,
    pub after: String,
}

/// Errors returned by [`find_fim_section`] when the markers are absent
/// or malformed. The Tauri command surface maps these to user-visible
/// strings; programmatic callers should match on the variant.
#[derive(Debug, Error)]
pub enum FimError {
    #[error("FIM start marker (FIM_FILL_START) not found")]
    MissingStartMarker,
    #[error("FIM end marker (FIM_FILL_END) not found after start")]
    MissingEndMarker,
    #[error("FIM markers out of order — end appears before start")]
    MarkersOutOfOrder,
}

/// Locate the fill region. Returns the surrounding text on either side
/// (markers included) and the middle (markers excluded).
///
/// Algorithm: substring search for the literal sentinel tokens, then
/// extend each match to the end of its containing line so the comment
/// trailer (` -->` or ` */}`) stays attached to the marker block. The
/// middle therefore never contains the marker line itself, which keeps
/// `splice_fim_section` round-trip-safe.
pub fn find_fim_section(content: &str) -> Result<FimSplice, FimError> {
    let start_byte = content
        .find(FIM_TOKEN_FILL_START)
        .ok_or(FimError::MissingStartMarker)?;
    let end_byte = content
        .find(FIM_TOKEN_FILL_END)
        .ok_or(FimError::MissingEndMarker)?;
    if end_byte <= start_byte {
        return Err(FimError::MarkersOutOfOrder);
    }

    // Extend the start match to include the rest of its line + trailing newline
    // (the marker comment closer).
    let start_line_end = content[start_byte..]
        .find('\n')
        .map(|n| start_byte + n + 1)
        .unwrap_or(content.len());

    // Walk the end match back to the start of its line so the marker
    // comment opener (`<!--` / `{/*`) stays attached to the suffix.
    let end_line_start = content[..end_byte].rfind('\n').map(|n| n + 1).unwrap_or(0);

    let before = content[..start_line_end].to_owned();
    let middle = content[start_line_end..end_line_start].to_owned();
    let after = content[end_line_start..].to_owned();

    Ok(FimSplice {
        before,
        current_middle: middle,
        after,
    })
}

/// Build the merged file contents with `fill` substituted for the
/// placeholder middle. The prefix and suffix (markers included) are
/// preserved byte-for-byte.
pub fn splice_fim_section(splice: &FimSplice, fill: &str) -> String {
    let mut out =
        String::with_capacity(splice.before.len() + fill.len() + splice.after.len() + 2);
    out.push_str(&splice.before);
    out.push_str(fill);
    if !fill.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&splice.after);
    out
}

/// Construct the Qwen FIM prompt body. The `hint` is appended after the
/// `<|fim_middle|>` token so the model receives a natural-language
/// directive in addition to the surrounding code.
pub fn build_qwen_fim_prompt(splice: &FimSplice, hint: &str) -> String {
    let trimmed_hint = hint.trim();
    let mut prompt = String::with_capacity(
        QWEN_FIM_PREFIX.len()
            + splice.before.len()
            + QWEN_FIM_SUFFIX.len()
            + splice.after.len()
            + QWEN_FIM_MIDDLE.len()
            + trimmed_hint.len()
            + 16,
    );
    prompt.push_str(QWEN_FIM_PREFIX);
    prompt.push_str(&splice.before);
    prompt.push_str(QWEN_FIM_SUFFIX);
    prompt.push_str(&splice.after);
    prompt.push_str(QWEN_FIM_MIDDLE);
    if !trimmed_hint.is_empty() {
        prompt.push_str("\n<!-- ");
        prompt.push_str(trimmed_hint);
        prompt.push_str(" -->\n");
    }
    prompt
}

/// Build the Approach B (instruction-framing) prompt used as fallback
/// when the special-token route returns garbage. Output: a plain
/// instruction with explicit BEFORE / AFTER / HINT blocks.
pub fn build_instruction_fim_prompt(splice: &FimSplice, hint: &str) -> String {
    format!(
        "Replace the section between the markers. Output ONLY the replacement text — no commentary, no markers, no code fences.\n\nBEFORE:\n{}\nAFTER:\n{}\nHINT: {}\n\nREPLACEMENT:\n",
        splice.before,
        splice.after,
        hint.trim()
    )
}

/// Strip Qwen FIM artefacts that may leak when the sidecar does not
/// honour a `stop` parameter. Removes the literal token strings
/// `<|fim_pad|>`, `<|endoftext|>`, and any stray `<|fim_prefix|>` /
/// `<|fim_suffix|>` / `<|fim_middle|>` echoes the model may emit.
///
/// Audit fix (s-69820): the previous implementation also dropped any
/// line whose trimmed body started with `<!--` or `{/*`, which silently
/// destroyed legitimate generated HTML comments (a11y notes, IE
/// conditional blocks, doc strings). Token-only stripping preserves
/// every byte that does not literally contain a Qwen FIM sentinel.
pub fn strip_fim_artifacts(text: &str) -> String {
    let mut cleaned = text.to_owned();
    for token in [
        QWEN_FIM_PAD,
        QWEN_END_OF_TEXT,
        QWEN_FIM_PREFIX,
        QWEN_FIM_SUFFIX,
        QWEN_FIM_MIDDLE,
    ] {
        cleaned = cleaned.replace(token, "");
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_replaces_middle() {
        let body = "head\n<!-- FIM_FILL_START -->\nold\n<!-- FIM_FILL_END -->\ntail\n";
        let splice = find_fim_section(body).unwrap();
        let merged = splice_fim_section(&splice, "fresh");
        assert!(merged.contains("head"));
        assert!(merged.contains("tail"));
        assert!(merged.contains("fresh"));
        assert!(!merged.contains("old"));
    }
}
