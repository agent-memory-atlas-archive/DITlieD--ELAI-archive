//! E-37 U1 — output sanitizer (OWASP LLM Top 10 LLM01, LLM06 sensitive-data
//! disclosure + LLM09 supply-chain-via-markdown-image). Pure, stdlib-only,
//! no regex crate, no LLM classifier. Two concerns handled separately:
//!
//!   * [`strip_markdown_images`] — removes `![alt](url)` and `<img src="...">`
//!     markdown whose host is NOT on the caller-supplied allowlist. Closes the
//!     indirect-prompt-injection / image-pixel-exfil channel where a tool
//!     result embeds a markdown image pointing at an attacker host (the host
//!     sees the request + Referer and learns the agent state, or the rendered
//!     image carries instructions the model follows).
//!   * [`classify_output`] — flags impersonation / script-injection patterns
//!     the model might mistake for system framing: `<script`, `<assistant>`,
//!     `<system>`, `<user>` role tags. Returns [`OutputVerdict::Flagged`]
//!     carrying the matched pattern strings so the orchestrator can emit a
//!     `OutputClassifierFlagged` SSB row and downstream gates can react.
//!
//! Grounding: OWASP LLM Top 10 2025 LLM01:2025 Prompt Injection + LLM06:2025
//! Sensitive Information Disclosure (markdown-image exfil channel is the
//! CometJacking / EchoLeak shape, CVE-2025-32711 + CVE-2025-59536 family).
//! Hand-rolled host extractor (no `url` crate, no `regex` crate) keeps
//! `elai-safety` dependency-free, matching the E-30 / G-3 blast_radius
//! discipline.

/// Host extractor: from a URL string, take the bytes after `://` up to the
/// next `/` `?` `#` or end-of-string, then ASCII-lowercase. Returns `None`
/// when there is no `://` separator (so a bare path or fragment does not
/// produce a false host). No IDN / punycode handling — the gate is
/// allowlist-membership on the literal host string the model would emit.
fn extract_host(url: &str) -> Option<String> {
    let scheme_end = url.find("://")?;
    let rest = &url[scheme_end + 3..];
    let end = rest
        .find(|c: char| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    if end == 0 {
        return None;
    }
    let host = rest[..end].to_ascii_lowercase();
    Some(host)
}

/// Result of stripping non-allowlisted markdown images from a tool output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizedOutput {
    /// The output with non-allowlisted `![alt](url)` and `<img src="...">`
    /// removed (replaced with empty string).
    pub cleaned: String,
    /// Number of markdown-image occurrences stripped.
    pub stripped_count: u32,
    /// Lowercased hosts that were stripped (one per stripped image, in the
    /// order encountered). Empty when nothing was stripped.
    pub stripped_hosts: Vec<String>,
}

/// Verdict from [`classify_output`]. `Clean` when no impersonation / script
/// pattern was found; `Flagged` otherwise, carrying the matched pattern
/// strings (e.g. `"<script"`, `"<assistant>"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputVerdict {
    Clean,
    Flagged { patterns: Vec<String> },
}

/// Strip `![alt](url)` and `<img src="...">` markdown whose host is NOT on
/// `allowlist`. The allowlist is matched against the lowercased host. An
/// image whose URL has no parseable host (no `://`) is preserved — the
/// strip targets the network-exfil channel, not local anchors.
///
/// The parser is deliberately line-naive: it scans the input once, finds
/// `![` ... `]` ... `(` ... `)` spans and `<img` ... `>` spans, and removes
/// them when the extracted host is non-empty and not in the allowlist.
/// Malformed spans (no closing `)` etc.) are left in place rather than
/// guessed at — fail-open on parse ambiguity, fail-closed on the host
/// decision itself.
pub fn strip_markdown_images(output: &str, allowlist: &[String]) -> SanitizedOutput {
    let mut stripped_hosts: Vec<String> = Vec::new();
    let mut stripped_count: u32 = 0;
    let mut cleaned = String::with_capacity(output.len());
    let bytes = output.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Try `![alt](url)` form.
        if bytes[i] == b'!' && i + 1 < bytes.len() && bytes[i + 1] == b'[' {
            if let Some((span_end, url)) = parse_md_image(&output[i..]) {
                let host = extract_host(&url);
                let strip = match &host {
                    Some(h) => !allowlist.iter().any(|a| a == h),
                    None => false,
                };
                if strip {
                    stripped_count += 1;
                    if let Some(h) = host {
                        stripped_hosts.push(h);
                    }
                    i += span_end;
                    continue;
                }
            }
        }
        // Try `<img ... src="..." ...>` form (case-insensitive tag).
        if bytes[i] == b'<' && i + 4 <= bytes.len() {
            let tag = &output[i..i + 4];
            if tag.eq_ignore_ascii_case("<img") {
                if let Some((span_end, url)) = parse_html_img(&output[i..]) {
                    let host = extract_host(&url);
                    let strip = match &host {
                        Some(h) => !allowlist.iter().any(|a| a == h),
                        None => false,
                    };
                    if strip {
                        stripped_count += 1;
                        if let Some(h) = host {
                            stripped_hosts.push(h);
                        }
                        i += span_end;
                        continue;
                    }
                }
            }
        }
        cleaned.push(bytes[i] as char);
        i += 1;
    }
    SanitizedOutput {
        cleaned,
        stripped_count,
        stripped_hosts,
    }
}

/// Parse `![alt](url)` starting at the head of `s`. Returns
/// `(bytes_consumed, url)` on a complete match. `bytes_consumed` is the
/// number of bytes from the start of `s` that form the image span.
fn parse_md_image(s: &str) -> Option<(usize, String)> {
    let b = s.as_bytes();
    // s[0..2] == "!["
    // find closing ']'
    let mut depth = 1;
    let mut j = 2;
    while j < b.len() {
        match b[j] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            b'\\' => j += 1,
            _ => {}
        }
        j += 1;
    }
    if depth != 0 || j + 1 >= b.len() || b[j + 1] != b'(' {
        return None;
    }
    // find closing ')'
    let url_start = j + 2;
    let mut k = url_start;
    while k < b.len() && b[k] != b')' {
        if b[k] == b'\\' {
            k += 1;
        }
        k += 1;
    }
    if k >= b.len() {
        return None;
    }
    let url = s[url_start..k].to_string();
    Some((k + 1, url))
}

/// Parse `<img ... src="url" ...>` starting at the head of `s`. Returns
/// `(bytes_consumed, url)`. Recognizes single or double quotes around the
/// URL. Returns `None` when no `src` attribute is present or the tag is
/// unclosed.
fn parse_html_img(s: &str) -> Option<(usize, String)> {
    let b = s.as_bytes();
    let mut k = 4; // past "<img"
    let mut src_url: Option<String> = None;
    while k < b.len() {
        // end of tag
        if b[k] == b'>' {
            let url = src_url?;
            return Some((k + 1, url));
        }
        // attribute boundary: skip whitespace then read attr name
        if b[k].is_ascii_whitespace() {
            while k < b.len() && b[k].is_ascii_whitespace() {
                k += 1;
            }
            continue;
        }
        // read attribute name up to '=' or whitespace or '>'
        let name_start = k;
        while k < b.len()
            && b[k] != b'='
            && b[k] != b'>'
            && !b[k].is_ascii_whitespace()
        {
            k += 1;
        }
        let name = &s[name_start..k];
        // skip whitespace before '='
        while k < b.len() && b[k].is_ascii_whitespace() {
            k += 1;
        }
        if k < b.len() && b[k] == b'=' {
            k += 1;
            while k < b.len() && b[k].is_ascii_whitespace() {
                k += 1;
            }
            if k >= b.len() {
                return None;
            }
            let quote = b[k];
            if quote == b'"' || quote == b'\'' {
                k += 1;
                let val_start = k;
                while k < b.len() && b[k] != quote {
                    k += 1;
                }
                if k >= b.len() {
                    return None;
                }
                let val = s[val_start..k].to_string();
                k += 1;
                if name.eq_ignore_ascii_case("src") {
                    src_url = Some(val);
                }
            } else {
                // unquoted value
                let val_start = k;
                while k < b.len()
                    && b[k] != b'>'
                    && !b[k].is_ascii_whitespace()
                {
                    k += 1;
                }
                let val = s[val_start..k].to_string();
                if name.eq_ignore_ascii_case("src") {
                    src_url = Some(val);
                }
            }
        }
    }
    None
}

/// Classify a (already image-stripped) output for impersonation / script
/// patterns. Returns [`OutputVerdict::Flagged`] carrying the matched
/// pattern strings when any of `<script`, `<assistant>`, `<system>`,
/// `<user>` is present (case-insensitive). The role-tag check is
/// case-sensitive on the angle-bracket form because the model-instruction
/// injection channel depends on the literal tag the harness wraps messages
/// in; `<System>` would not be mistaken for a system frame by a well-behaved
/// harness, so we do not flag it (avoid noisy false positives on prose that
/// mentions "the System").
pub fn classify_output(output: &str) -> OutputVerdict {
    let mut patterns: Vec<String> = Vec::new();
    let lower = output.to_ascii_lowercase();
    if lower.contains("<script") {
        patterns.push("<script".to_string());
    }
    if output.contains("<assistant>") {
        patterns.push("<assistant>".to_string());
    }
    if output.contains("<system>") {
        patterns.push("<system>".to_string());
    }
    if output.contains("<user>") {
        patterns.push("<user>".to_string());
    }
    if patterns.is_empty() {
        OutputVerdict::Clean
    } else {
        OutputVerdict::Flagged { patterns }
    }
}
