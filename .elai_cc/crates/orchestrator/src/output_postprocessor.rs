// Output postprocessor: deterministic enforcement of tone/style rules.
//
// Four stages applied to every model text response:
//   1. Emoji stripper  — remove Unicode emoji ranges, preserve technical symbols
//   2. Sycophancy filter — strip leading affirmation patterns
//   3. Length enforcer  — truncate at sentence boundary if over limit
//   4. Output formatting restoration — tree-sitter pretty-print (A-42-T2 stub)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::context_budget::restore_output_formatting;
use crate::role_contracts::OutputChannelClass;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Per-tier output limits and toggle flags. Loaded from `[output]` in
/// elai_cc.toml or constructed programmatically.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Strip emoji codepoints from model output (default: true).
    #[serde(default = "default_true")]
    pub strip_emoji: bool,

    /// Strip sycophantic leading phrases (default: true).
    #[serde(default = "default_true")]
    pub strip_sycophancy: bool,

    /// Hard token-approximation ceiling (character count / 4).
    /// 0 = disabled (no truncation).
    #[serde(default)]
    pub max_response_tokens: usize,

    /// Apply tree-sitter output formatting restoration (default: false).
    /// A-42-T2: stub — tree-sitter not wired yet (dep cycle risk with knowledge crate).
    #[serde(default)]
    pub restore_formatting: bool,

    /// Language hint for output formatting restoration (e.g. "rust", "python").
    /// Used by stage 4 when restore_formatting is true.
    #[serde(default)]
    pub restore_formatting_language: String,

    /// J-106: which output channel this runtime's text belongs to. Serde default
    /// is `Reasoning` — FAIL-CLOSED: a config file or constructor that does not
    /// set it gets a gate that cannot fire, so a reasoning role is never
    /// constrained by omission (HD-3).
    #[serde(default = "default_channel_class")]
    pub channel_class: OutputChannelClass,
}

fn default_true() -> bool {
    true
}

fn default_channel_class() -> OutputChannelClass {
    OutputChannelClass::Reasoning
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            strip_emoji: true,
            strip_sycophancy: true,
            max_response_tokens: 0,
            restore_formatting: false,
            restore_formatting_language: String::new(),
            channel_class: OutputChannelClass::Reasoning,
        }
    }
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Approximate characters-per-token ratio for length enforcement.
const CHARS_PER_TOKEN: usize = 4;

/// Marker appended when truncation occurs.
const TRUNCATION_MARKER: &str = "[response truncated]";

// ---------------------------------------------------------------------------
// Sycophancy patterns (compiled once)
// ---------------------------------------------------------------------------

/// Leading affirmation phrases to strip, case-insensitive.
const SYCOPHANCY_PATTERNS: &[&str] = &[
    r"(?i)^sure[!,.]?\s*",
    r"(?i)^of course[!,.]?\s*",
    r"(?i)^great question[!,.]?\s*",
    r"(?i)^absolutely[!,.]?\s*",
    r"(?i)^that'?s a great (idea|question)[!,.]?\s*",
    r"(?i)^happy to help[!,.]?\s*",
    r"(?i)^certainly[!,.]?\s*",
    r"(?i)^no problem[!,.]?\s*",
    r"(?i)^glad you asked[!,.]?\s*",
    r"(?i)^good question[!,.]?\s*",
    r"(?i)^great idea[!,.]?\s*",
    r"(?i)^i'?d be happy to\s*",
    r"(?i)^i'?d love to help[!,.]?\s*",
];

fn sycophancy_regexes() -> &'static [Regex] {
    static CACHE: OnceLock<Vec<Regex>> = OnceLock::new();
    CACHE.get_or_init(|| {
        SYCOPHANCY_PATTERNS
            .iter()
            .map(|pat| Regex::new(pat).expect("sycophancy regex is valid"))
            .collect()
    })
}

// ---------------------------------------------------------------------------
// Emoji regex (compiled once)
// ---------------------------------------------------------------------------

fn emoji_regex() -> &'static Regex {
    static CACHE: OnceLock<Regex> = OnceLock::new();
    CACHE.get_or_init(|| {
        // Unicode emoji ranges — intentionally excludes:
        //   - Arrows (U+2190-U+21FF)
        //   - Math operators (U+2200-U+22FF)
        //   - Box drawing / block elements (U+2500-U+259F)
        //   - Braille (U+2800-U+28FF)
        let pattern = concat!(
            "[",
            "\u{1F600}-\u{1F64F}",  // Emoticons
            "\u{1F300}-\u{1F5FF}",  // Misc symbols & pictographs
            "\u{1F680}-\u{1F6FF}",  // Transport & map symbols
            "\u{1F1E0}-\u{1F1FF}",  // Flags
            "\u{1F900}-\u{1F9FF}",  // Supplemental symbols
            "\u{1FA00}-\u{1FA6F}",  // Chess symbols
            "\u{1FA70}-\u{1FAFF}",  // Symbols extended-A
            "\u{2600}-\u{26FF}",    // Misc symbols (sun, cloud, etc.)
            "\u{2700}-\u{27BF}",    // Dingbats
            "\u{FE00}-\u{FE0F}",    // Variation selectors
            "\u{200D}",             // Zero-width joiner (emoji sequences)
            "\u{20E3}",             // Combining enclosing keycap
            "\u{E0020}-\u{E007F}",  // Tags (flag sequences)
            "]",
        );
        Regex::new(pattern).expect("emoji regex is valid")
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Apply all enabled postprocessing stages to `response`.
///
/// Order: emoji strip -> sycophancy filter -> length enforcer.
/// Each stage is independently toggleable via `config`.
pub fn process(response: &str, config: &OutputConfig) -> String {
    if response.is_empty() {
        return String::new();
    }

    let mut text = response.to_owned();

    // Stage 1: emoji strip
    if config.strip_emoji {
        text = strip_emoji(&text);
    }

    // Stage 2: sycophancy filter
    if config.strip_sycophancy {
        text = strip_sycophancy(&text);
    }

    // Stage 3: length enforcer
    if config.max_response_tokens > 0 {
        text = enforce_length(&text, config.max_response_tokens);
    }

    // Stage 4: output formatting restoration (A-42-T2, stub).
    if config.restore_formatting {
        let lang = if config.restore_formatting_language.is_empty() {
            "rust"
        } else {
            &config.restore_formatting_language
        };
        text = restore_output_formatting(&text, lang);
    }

    // Stage 5 (J-106): output meta-commentary gate. Runs ONLY for op-emission
    // channels AND only when the promotion flag is on. Both guards are
    // fail-closed: a Reasoning class or an unset flag returns the text
    // byte-identical, so HD-3 ("reasoning roles are never constrained") holds
    // structurally, not by convention. The banned-phrase table, bounded retry
    // and work-log grammar are J-106's later EXECUTE tasks and fill this stage;
    // WAVE-1 ships the guarded seam.
    if config.channel_class == OutputChannelClass::OpEmission
        && std::env::var("ELAI_OUTPUT_METACOMMENTARY_GATE_ENABLE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false)
    {
        text = apply_metacommentary_gate(&text);
    }

    text
}

// ---------------------------------------------------------------------------
// Stage implementations
// ---------------------------------------------------------------------------

/// J-106 stage-5 body. WAVE-1 is observe-only: it returns the text unchanged and
/// exists so the guarded call site is real production wiring. The frozen
/// banned-phrase table and the READ/FOUND/DECISION/ACTION/RESULT work-log grammar
/// land in J-106's later EXECUTE tasks and replace this body.
fn apply_metacommentary_gate(text: &str) -> String {
    text.to_owned()
}

fn strip_emoji(text: &str) -> String {
    let re = emoji_regex();
    let result = re.replace_all(text, "");
    // Collapse runs of multiple spaces left by removed emoji.
    collapse_spaces(&result)
}

fn strip_sycophancy(text: &str) -> String {
    let mut result = text.to_owned();
    for re in sycophancy_regexes() {
        if re.is_match(&result) {
            result = re.replace(&result, "").to_string();
            // Only strip the first matching pattern, then stop.
            break;
        }
    }
    // Trim leading whitespace left after stripping.
    let trimmed = result.trim_start();
    if trimmed.len() != result.len() {
        return trimmed.to_owned();
    }
    result
}

fn enforce_length(text: &str, max_tokens: usize) -> String {
    let max_chars = max_tokens.saturating_mul(CHARS_PER_TOKEN);
    if text.len() <= max_chars {
        return text.to_owned();
    }

    // Find the last sentence boundary before max_chars.
    let truncated = &text[..max_chars];
    let last_sentence_end = truncated
        .rfind(". ")
        .or_else(|| truncated.rfind(".\n"))
        .or_else(|| truncated.rfind('?'))
        .or_else(|| truncated.rfind('!'))
        .map(|pos| pos + 1);

    let cut_point = match last_sentence_end {
        Some(pos) if pos > max_chars / 2 => pos,
        // No good sentence boundary in the second half — hard cut at max.
        _ => max_chars,
    };

    let mut result = text[..cut_point].trim_end().to_owned();
    result.push(' ');
    result.push_str(TRUNCATION_MARKER);
    result
}

fn collapse_spaces(text: &str) -> String {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r" {2,}").expect("space regex is valid"));
    re.replace_all(text, " ").into_owned()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn default_on() -> OutputConfig {
        OutputConfig {
            strip_emoji: true,
            strip_sycophancy: true,
            max_response_tokens: 0,
            restore_formatting: false,
            restore_formatting_language: String::new(),
            channel_class: OutputChannelClass::Reasoning,
        }
    }

    // -- Emoji stripping ---------------------------------------------------

    #[test]
    fn emoji_stripped_from_text() {
        let input = "Hello \u{1F600} world \u{2764}";
        let cfg = default_on();
        let out = process(input, &cfg);
        assert!(!out.contains('\u{1F600}'));
        assert!(out.contains("Hello"));
        assert!(out.contains("world"));
    }

    #[test]
    fn arrows_and_math_preserved() {
        let input = "a \u{2192} b, x \u{2264} y";
        let cfg = default_on();
        let out = process(input, &cfg);
        assert!(out.contains('\u{2192}'), "arrow should be preserved");
        assert!(out.contains('\u{2264}'), "less-than-or-equal should be preserved");
    }

    #[test]
    fn emoji_disabled_preserves_emoji() {
        let input = "Hello \u{1F600}";
        let cfg = OutputConfig {
            strip_emoji: false,
            ..default_on()
        };
        let out = process(input, &cfg);
        assert!(out.contains('\u{1F600}'));
    }

    // -- Sycophancy filter -------------------------------------------------

    #[test]
    fn strips_sure_prefix() {
        let out = process("Sure! Here is the answer.", &default_on());
        assert_eq!(out, "Here is the answer.");
    }

    #[test]
    fn strips_of_course_prefix() {
        let out = process("Of course, I can help.", &default_on());
        assert_eq!(out, "I can help.");
    }

    #[test]
    fn strips_great_question_prefix() {
        let out = process("Great question! The reason is...", &default_on());
        assert_eq!(out, "The reason is...");
    }

    #[test]
    fn strips_absolutely_prefix() {
        let out = process("Absolutely! Let me explain.", &default_on());
        assert_eq!(out, "Let me explain.");
    }

    #[test]
    fn strips_happy_to_help_prefix() {
        let out = process("Happy to help! Here goes.", &default_on());
        assert_eq!(out, "Here goes.");
    }

    #[test]
    fn strips_certainly_prefix() {
        let out = process("Certainly, the code is:", &default_on());
        assert_eq!(out, "the code is:");
    }

    #[test]
    fn no_strip_mid_sentence() {
        let input = "I am sure this works.";
        let out = process(input, &default_on());
        assert_eq!(out, input);
    }

    #[test]
    fn sycophancy_disabled_preserves() {
        let cfg = OutputConfig {
            strip_sycophancy: false,
            ..default_on()
        };
        let out = process("Sure! Hello.", &cfg);
        assert!(out.starts_with("Sure"));
    }

    // -- Length enforcer ----------------------------------------------------

    #[test]
    fn short_response_unchanged() {
        let cfg = OutputConfig {
            max_response_tokens: 100,
            ..default_on()
        };
        let out = process("Short text.", &cfg);
        assert_eq!(out, "Short text.");
    }

    #[test]
    fn long_response_truncated_at_sentence() {
        let sentence = "This is a sentence. ";
        let long_text = sentence.repeat(50); // ~1000 chars
        let cfg = OutputConfig {
            max_response_tokens: 50, // 50 * 4 = 200 chars limit
            ..default_on()
        };
        let out = process(&long_text, &cfg);
        assert!(out.ends_with(TRUNCATION_MARKER));
        assert!(out.len() < long_text.len());
    }

    #[test]
    fn zero_max_tokens_no_truncation() {
        let long_text = "word ".repeat(500);
        let cfg = OutputConfig {
            max_response_tokens: 0,
            ..default_on()
        };
        let out = process(&long_text, &cfg);
        assert_eq!(out.trim(), long_text.trim());
    }

    // -- Empty / edge cases ------------------------------------------------

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(process("", &default_on()), "");
    }

    #[test]
    fn whitespace_only_after_sycophancy_strip() {
        let out = process("Sure! ", &default_on());
        assert_eq!(out, "");
    }

    // -- All stages combined -----------------------------------------------

    #[test]
    fn all_stages_combined() {
        let input = "Sure! \u{1F600} Here is the result.";
        let cfg = default_on();
        let out = process(input, &cfg);
        assert!(!out.contains('\u{1F600}'));
        assert!(!out.starts_with("Sure"));
        assert!(out.contains("Here is the result."));
    }

    // -- Config serde roundtrip -------------------------------------------

    #[test]
    fn config_serde_roundtrip() {
        let cfg = OutputConfig {
            strip_emoji: false,
            strip_sycophancy: true,
            max_response_tokens: 512,
            restore_formatting: false,
            restore_formatting_language: String::new(),
            channel_class: OutputChannelClass::Reasoning,
        };
        let json = serde_json::to_string(&cfg).expect("serialize");
        let parsed: OutputConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.strip_emoji, false);
        assert_eq!(parsed.strip_sycophancy, true);
        assert_eq!(parsed.max_response_tokens, 512);
    }

    #[test]
    fn config_defaults_from_empty_json() {
        let parsed: OutputConfig = serde_json::from_str("{}").expect("deserialize");
        assert!(parsed.strip_emoji);
        assert!(parsed.strip_sycophancy);
        assert_eq!(parsed.max_response_tokens, 0);
    }

    // -- Collapse spaces --------------------------------------------------

    #[test]
    fn double_spaces_collapsed() {
        let out = collapse_spaces("hello  world   foo");
        assert_eq!(out, "hello world foo");
    }
}
