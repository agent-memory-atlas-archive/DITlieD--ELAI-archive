//! C-47: Heuristic complexity score H in [0,1] per CAST paper Section 3.2.
//!
//! The paper uses a Gemini-Pro judge over base-model rollouts to assign
//! per-trajectory hardness. We can not run an external judge at the bridge
//! seam, so C-47 ships a cheap, deterministic, stdlib-only heuristic that
//! looks at the call-shape JSON string and returns a value clamped to
//! `[COMPLEXITY_FLOOR, COMPLEXITY_CEIL]`. The heuristic is intentionally
//! simple — it is a signal-stand-in, not a faithful re-implementation of
//! the paper's rubric. The contract test asserts the invariant
//! `0.0 <= H <= 1.0`.
//!
//! Inputs:
//!   * `call_shape: &str` — serialised JSON of the tool-call arguments
//!
//! Outputs:
//!   * `f32` in `[COMPLEXITY_FLOOR, COMPLEXITY_CEIL]`
//!
//! Heuristic factors (all unit-bounded contributions, summed then
//! clamped):
//!   * argument-key count    — more keys = higher complexity (max ~0.3)
//!   * nesting depth         — deeper JSON = higher complexity (max ~0.3)
//!   * special-char ratio    — high punctuation density (e.g. regex
//!                              patterns, file paths) = higher complexity
//!                              (max ~0.4)
//!
//! Named constants per CLAUDE.md anti-stale: no magic numbers in the
//! formula.

/// Minimum complexity score (used when the call-shape is empty/null).
pub const COMPLEXITY_FLOOR: f32 = 0.0;
/// Maximum complexity score (paper rubric is `H in [0,1]`).
pub const COMPLEXITY_CEIL: f32 = 1.0;

const KEY_COUNT_SATURATION: f32 = 12.0;
const KEY_COUNT_WEIGHT: f32 = 0.30;
const DEPTH_SATURATION: f32 = 5.0;
const DEPTH_WEIGHT: f32 = 0.30;
const SPECIAL_CHAR_WEIGHT: f32 = 0.40;

/// Return the heuristic complexity score H in [0,1] for a serialized
/// call-shape JSON string. Empty / unparsable input returns
/// [`COMPLEXITY_FLOOR`].
pub fn complexity_score(call_shape_json: &str) -> f32 {
    let trimmed = call_shape_json.trim();
    if trimmed.is_empty() {
        return COMPLEXITY_FLOOR;
    }

    let key_factor = key_count_factor(trimmed);
    let depth_factor = depth_factor(trimmed);
    let special_factor = special_char_factor(trimmed);

    let raw = key_factor * KEY_COUNT_WEIGHT
        + depth_factor * DEPTH_WEIGHT
        + special_factor * SPECIAL_CHAR_WEIGHT;

    raw.clamp(COMPLEXITY_FLOOR, COMPLEXITY_CEIL)
}

/// Saturate the argument-key count into a `[0,1]` factor.
fn key_count_factor(s: &str) -> f32 {
    // Approximate JSON-object key count via colon density at outer
    // braces. This is intentionally cheap — we already serialize the
    // call shape downstream, so a precise serde walk is overkill for
    // a heuristic.
    let colons = s.chars().filter(|c| *c == ':').count() as f32;
    (colons / KEY_COUNT_SATURATION).min(1.0)
}

/// Maximum brace-nesting depth normalised to `[0,1]`.
fn depth_factor(s: &str) -> f32 {
    let mut depth: i32 = 0;
    let mut max_depth: i32 = 0;
    for ch in s.chars() {
        match ch {
            '{' | '[' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            '}' | ']' => {
                depth -= 1;
                if depth < 0 {
                    // Malformed input — treat as zero-depth.
                    depth = 0;
                }
            }
            _ => {}
        }
    }
    (max_depth as f32 / DEPTH_SATURATION).min(1.0)
}

/// Punctuation density (excluding the JSON structural characters
/// `{`, `}`, `[`, `]`, `:`, `,`, `"`). High densities flag regex
/// patterns, shell commands, file paths, etc.
fn special_char_factor(s: &str) -> f32 {
    if s.is_empty() {
        return 0.0;
    }
    let total = s.chars().count() as f32;
    let special = s
        .chars()
        .filter(|c| {
            c.is_ascii_punctuation()
                && !matches!(c, '{' | '}' | '[' | ']' | ':' | ',' | '"' | '\'')
        })
        .count() as f32;
    (special / total).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_floor() {
        assert_eq!(complexity_score(""), COMPLEXITY_FLOOR);
    }

    #[test]
    fn complexity_stays_in_unit_range() {
        let samples = [
            r#"{}"#,
            r#"{"k":"v"}"#,
            r#"{"a":1,"b":{"c":[1,2,3]}}"#,
            r#"{"pattern":"^[a-z_][a-z0-9_]*$\\d+\\.\\d+"}"#,
        ];
        for s in samples {
            let h = complexity_score(s);
            assert!(
                (COMPLEXITY_FLOOR..=COMPLEXITY_CEIL).contains(&h),
                "H out of range for {s:?}: {h}"
            );
        }
    }

    #[test]
    fn deeper_nesting_is_more_complex() {
        let shallow = complexity_score(r#"{"a":1}"#);
        let deep = complexity_score(r#"{"a":{"b":{"c":{"d":{"e":1}}}}}"#);
        assert!(deep >= shallow);
    }

    #[test]
    fn malformed_input_does_not_panic() {
        // Unbalanced braces — must clamp depth at zero without overflow.
        let _ = complexity_score("}]}}}}}}}");
        let _ = complexity_score("{{{{{{{{{{");
    }
}
