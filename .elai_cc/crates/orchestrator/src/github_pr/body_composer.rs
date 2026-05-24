//! E-41: `PrBodyComposer` — pure-stdlib placeholder substitution for PR
//! body templates.
//!
//! Replaces Jinja-style `{{key}}` markers with values from a key/value
//! context map. No external template engine — keeps the orchestrator
//! crate free of `tera` / `handlebars` / `liquid` dependencies (anti-deps
//! HARD RULE in CLAUDE.md, sibling C-40 precedent).
//!
//! The composer enforces the maintainer-voice rules from
//! `.claude/rules/commit-cadence.md` only through the template the caller
//! provides — we substitute literally, not strip or rewrite. Callers MUST
//! supply a maintainer-voice template; the cite-discipline lives at the
//! template-author layer, not here.
//!
//! Production caller: [`crate::github_pr::body_composer_cli::compose_body_from_pr`]
//! invoked by the `pr review compose-body` CLI subverb (closes the
//! WIRE-DARK gap pattern — every E-41 module has at least one CLI-level
//! caller that exercises it in production).

use std::collections::BTreeMap;

use super::error::GithubPrError;

/// Maximum supported template length (256 KB). Templates larger than this
/// almost always indicate a runaway template generator and would balloon
/// SSB payloads even after hashing — so we fail fast.
const MAX_TEMPLATE_BYTES: usize = 256 * 1024;

/// Maximum number of placeholder substitutions in one render pass.
/// Defensive cap against pathological templates (e.g. `{{a}}{{a}}{{a}}...`).
const MAX_SUBSTITUTIONS: usize = 10_000;

/// Render context for [`compose_pr_body`]. A deterministic ordered map
/// (`BTreeMap`) so identical inputs yield byte-identical output —
/// required for the `pr_review_composed` SSB row to be deterministic
/// across runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrContext {
    /// Placeholder name (without braces) -> literal replacement text.
    /// Keys MUST match `[A-Za-z0-9_]+`; non-matching keys are rejected by
    /// [`compose_pr_body`].
    pub values: BTreeMap<String, String>,
}

impl PrContext {
    /// Construct an empty context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Insert a single key/value pair. Returns `self` for builder use.
    #[must_use]
    pub fn with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.values.insert(key.into(), value.into());
        self
    }
}

/// Render `template` against `ctx` by replacing every `{{key}}` marker
/// with the matching value from `ctx.values`. Unknown placeholders are
/// left literal (so reviewers can see "this key was never substituted"
/// rather than silently dropping content).
///
/// Substitution rules:
/// - Keys must match `[A-Za-z0-9_]+` (alphanumeric + underscore). Other
///   shapes are passed through as literal text.
/// - Outer braces use double `{{` and `}}` (Jinja-compatible). Single
///   `{` / `}` characters are literal.
/// - The same `{{key}}` may appear any number of times; each occurrence
///   is replaced independently.
/// - Up to [`MAX_SUBSTITUTIONS`] replacements per call; beyond that the
///   call fails with `GithubPrError::InvalidRequest`.
///
/// # Errors
/// Returns [`GithubPrError::InvalidRequest`] when the template exceeds
/// [`MAX_TEMPLATE_BYTES`] or the substitution count exceeds
/// [`MAX_SUBSTITUTIONS`].
pub fn compose_pr_body(template: &str, ctx: &PrContext) -> Result<String, GithubPrError> {
    if template.len() > MAX_TEMPLATE_BYTES {
        return Err(GithubPrError::InvalidRequest {
            reason: format!(
                "template length {} bytes exceeds cap of {} bytes",
                template.len(),
                MAX_TEMPLATE_BYTES
            ),
        });
    }
    let mut out = String::with_capacity(template.len());
    let bytes = template.as_bytes();
    let mut i: usize = 0;
    let mut substitutions: usize = 0;
    while i < bytes.len() {
        // Look for the next `{{` opener.
        if i + 1 < bytes.len() && bytes[i] == b'{' && bytes[i + 1] == b'{' {
            // Scan for matching `}}`.
            let start_key = i + 2;
            let mut end_key = start_key;
            while end_key + 1 < bytes.len()
                && !(bytes[end_key] == b'}' && bytes[end_key + 1] == b'}')
            {
                end_key += 1;
            }
            if end_key + 1 < bytes.len()
                && bytes[end_key] == b'}'
                && bytes[end_key + 1] == b'}'
            {
                let key_slice = &template[start_key..end_key];
                if is_valid_key(key_slice) {
                    if let Some(replacement) = ctx.values.get(key_slice) {
                        out.push_str(replacement);
                        substitutions += 1;
                        if substitutions > MAX_SUBSTITUTIONS {
                            return Err(GithubPrError::InvalidRequest {
                                reason: format!(
                                    "template exceeded substitution cap of {MAX_SUBSTITUTIONS}"
                                ),
                            });
                        }
                        i = end_key + 2;
                        continue;
                    }
                }
                // Unknown / malformed key — emit the original literal
                // including the braces so reviewers can see the marker.
                out.push_str(&template[i..end_key + 2]);
                i = end_key + 2;
                continue;
            }
            // Unterminated `{{ ...` — copy the rest literally and break.
            out.push_str(&template[i..]);
            break;
        }
        // Copy one UTF-8 char to the output. `template.is_char_boundary`
        // ensures we never split a multi-byte sequence.
        let mut next = i + 1;
        while next < bytes.len() && !template.is_char_boundary(next) {
            next += 1;
        }
        out.push_str(&template[i..next]);
        i = next;
    }
    Ok(out)
}

/// Returns true when `key` matches `[A-Za-z0-9_]+` and is non-empty.
fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substitutes_known_key() {
        let ctx = PrContext::new().with("plan", "E-41").with("session", "s-25120");
        let out = compose_pr_body("plan={{plan}} session={{session}}", &ctx).expect("compose ok");
        assert_eq!(out, "plan=E-41 session=s-25120");
    }

    #[test]
    fn leaves_unknown_key_literal() {
        let ctx = PrContext::new().with("plan", "E-41");
        let out = compose_pr_body("plan={{plan}} extra={{unknown}}", &ctx).expect("compose ok");
        assert_eq!(out, "plan=E-41 extra={{unknown}}");
    }

    #[test]
    fn rejects_malformed_key_with_spaces() {
        let ctx = PrContext::new().with("plan id", "should-not-match");
        // Key with a space is invalid -> literal.
        let out =
            compose_pr_body("{{plan id}}", &ctx).expect("compose ok (invalid key passes through)");
        assert_eq!(out, "{{plan id}}");
    }

    #[test]
    fn cap_enforced_on_substitutions() {
        let mut ctx = PrContext::new();
        ctx.values.insert("k".into(), "X".into());
        // 10_001 occurrences -> over cap.
        let template = "{{k}}".repeat(MAX_SUBSTITUTIONS + 1);
        let err = compose_pr_body(&template, &ctx).expect_err("over cap");
        match err {
            GithubPrError::InvalidRequest { reason } => {
                assert!(reason.contains("substitution cap"));
            }
            other => panic!("expected InvalidRequest, got {other:?}"),
        }
    }

    #[test]
    fn single_braces_are_literal() {
        let ctx = PrContext::new().with("k", "V");
        let out = compose_pr_body("{ {k} } -- {{k}}", &ctx).expect("compose ok");
        assert_eq!(out, "{ {k} } -- V");
    }

    #[test]
    fn cap_enforced_on_template_size() {
        let ctx = PrContext::new();
        let oversize = "x".repeat(MAX_TEMPLATE_BYTES + 1);
        let err = compose_pr_body(&oversize, &ctx).expect_err("over cap");
        match err {
            GithubPrError::InvalidRequest { reason } => {
                assert!(reason.contains("template length"));
            }
            other => panic!("expected InvalidRequest, got {other:?}"),
        }
    }
}
