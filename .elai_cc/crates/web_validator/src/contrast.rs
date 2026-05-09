//! WEB.3 — Phase B / Stream B3 — WCAG 2.1 contrast math.
//!
//! Pure Rust, no new dependencies. Implements the public-domain WCAG 2.1
//! relative-luminance formula:
//!
//!   Linearise sRGB channel C in [0, 1]:
//!     C_lin = C/12.92                    if C <= 0.03928
//!           = ((C + 0.055)/1.055)^2.4    otherwise
//!
//!   Relative luminance L = 0.2126*R_lin + 0.7152*G_lin + 0.0722*B_lin
//!
//!   Contrast ratio = (L_lighter + 0.05) / (L_darker + 0.05)
//!
//!   AA pass: ratio >= 4.5 (normal text), 3.0 (large text)
//!   AAA pass: ratio >= 7.0 (normal text), 4.5 (large text)
//!
//! Reference: WCAG 2.1 SC 1.4.3 / 1.4.6 — https://www.w3.org/TR/WCAG21/

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Named constants — WCAG thresholds.
// ---------------------------------------------------------------------------

/// AA threshold for normal-size text (WCAG 1.4.3).
pub const WCAG_AA_NORMAL: f64 = 4.5;
/// AA threshold for large text (>= 18pt or >= 14pt bold).
pub const WCAG_AA_LARGE: f64 = 3.0;
/// AAA threshold for normal-size text (WCAG 1.4.6).
pub const WCAG_AAA_NORMAL: f64 = 7.0;
/// AAA threshold for large text.
pub const WCAG_AAA_LARGE: f64 = 4.5;

const SRGB_LINEAR_THRESHOLD: f64 = 0.03928;
const SRGB_LINEAR_DIVISOR: f64 = 12.92;
const SRGB_GAMMA_OFFSET: f64 = 0.055;
const SRGB_GAMMA_DIVISOR: f64 = 1.055;
const SRGB_GAMMA_EXPONENT: f64 = 2.4;

const LUMINANCE_R_COEFF: f64 = 0.2126;
const LUMINANCE_G_COEFF: f64 = 0.7152;
const LUMINANCE_B_COEFF: f64 = 0.0722;

const CONTRAST_OFFSET: f64 = 0.05;

// ---------------------------------------------------------------------------
// Public types.
// ---------------------------------------------------------------------------

/// Whether the text being evaluated is large per WCAG (>=18pt or >=14pt bold).
/// Large text is held to a more lenient threshold than normal text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextSize {
    Normal,
    Large,
}

/// One contrast-failure finding from `evaluate_contrast_in_css`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContrastFinding {
    pub fg_hex: String,
    pub bg_hex: String,
    pub ratio: f64,
    pub text_size: TextSize,
    /// Optional source location pointer (line, column) within the CSS body.
    pub line: Option<u32>,
    /// Optional source file path; populated when callers know it.
    pub source: Option<PathBuf>,
}

// ---------------------------------------------------------------------------
// Core math.
// ---------------------------------------------------------------------------

/// sRGB → linear: convert one normalised channel `c` ∈ [0,1] to its linear
/// equivalent.
pub fn srgb_to_linear(c: f64) -> f64 {
    let c = c.clamp(0.0, 1.0);
    if c <= SRGB_LINEAR_THRESHOLD {
        c / SRGB_LINEAR_DIVISOR
    } else {
        ((c + SRGB_GAMMA_OFFSET) / SRGB_GAMMA_DIVISOR).powf(SRGB_GAMMA_EXPONENT)
    }
}

/// Relative luminance of an sRGB triple in [0,1]. Returns a value in [0,1].
pub fn relative_luminance(r: f64, g: f64, b: f64) -> f64 {
    LUMINANCE_R_COEFF * srgb_to_linear(r)
        + LUMINANCE_G_COEFF * srgb_to_linear(g)
        + LUMINANCE_B_COEFF * srgb_to_linear(b)
}

/// WCAG 2.1 contrast ratio between two sRGB triples in [0,1].
/// Always returns a value >= 1.0 and is symmetric in its arguments.
pub fn contrast_ratio(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    let la = relative_luminance(a.0, a.1, a.2);
    let lb = relative_luminance(b.0, b.1, b.2);
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    (lighter + CONTRAST_OFFSET) / (darker + CONTRAST_OFFSET)
}

/// Returns true iff `ratio` meets the WCAG AA threshold for the given text size.
pub fn wcag_aa_pass(ratio: f64, size: TextSize) -> bool {
    match size {
        TextSize::Normal => ratio >= WCAG_AA_NORMAL,
        TextSize::Large => ratio >= WCAG_AA_LARGE,
    }
}

/// Returns true iff `ratio` meets the WCAG AAA threshold for the given text size.
pub fn wcag_aaa_pass(ratio: f64, size: TextSize) -> bool {
    match size {
        TextSize::Normal => ratio >= WCAG_AAA_NORMAL,
        TextSize::Large => ratio >= WCAG_AAA_LARGE,
    }
}

// ---------------------------------------------------------------------------
// CSS hex-color extractor.
// ---------------------------------------------------------------------------

/// Parse a `#rrggbb` / `#rgb` / `#rrggbbaa` hex literal into an sRGB triple
/// in [0,1]. Alpha is ignored. Returns None if the literal is malformed.
pub fn parse_hex(hex: &str) -> Option<(f64, f64, f64)> {
    let s = hex.trim().trim_start_matches('#');
    let (r, g, b) = match s.len() {
        3 => {
            let r = u8::from_str_radix(&format!("{}{}", &s[0..1], &s[0..1]), 16).ok()?;
            let g = u8::from_str_radix(&format!("{}{}", &s[1..2], &s[1..2]), 16).ok()?;
            let b = u8::from_str_radix(&format!("{}{}", &s[2..3], &s[2..3]), 16).ok()?;
            (r, g, b)
        }
        6 | 8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            (r, g, b)
        }
        _ => return None,
    };
    Some((f64::from(r) / 255.0, f64::from(g) / 255.0, f64::from(b) / 255.0))
}

/// Scan a CSS body for `color: #...` paired with `background: #...` /
/// `background-color: #...` and emit a `ContrastFinding` per AA-failing
/// pair (normal text).
///
/// Approximation: walks each `{ … }` rule block, extracts the first hex
/// `color` and the first hex `background[-color]` it finds inside the
/// block, then evaluates the pair if both are present. This catches the
/// common offender that Phase A's WEB-025 regex aimed for without
/// requiring a full CSS cascade resolver. Order-of-declarations
/// independent (covers both `color: x; background: y;` and the reverse).
pub fn evaluate_contrast_in_css(css: &str) -> Vec<ContrastFinding> {
    let mut findings = Vec::new();
    let bytes = css.as_bytes();
    let mut depth: i32 = 0;
    let mut block_start: Option<usize> = None;
    let mut block_start_line: u32 = 1;
    let mut current_line: u32 = 1;

    let visit_block = |start: usize, end: usize, line_no: u32, out: &mut Vec<ContrastFinding>| {
        let block = &css[start..end];
        let bg_hex = find_decl_hex(block, &["background-color", "background"]);
        let fg_hex = find_decl_hex(block, &["color"]);
        if let (Some(bg), Some(fg)) = (bg_hex, fg_hex) {
            if let (Some(bg_rgb), Some(fg_rgb)) = (parse_hex(&bg), parse_hex(&fg)) {
                let ratio = contrast_ratio(fg_rgb, bg_rgb);
                if !wcag_aa_pass(ratio, TextSize::Normal) {
                    out.push(ContrastFinding {
                        fg_hex: fg,
                        bg_hex: bg,
                        ratio,
                        text_size: TextSize::Normal,
                        line: Some(line_no),
                        source: None,
                    });
                }
            }
        }
    };

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'\n' {
            current_line = current_line.saturating_add(1);
        }
        if b == b'{' {
            if depth == 0 {
                block_start = Some(i + 1);
                block_start_line = current_line;
            }
            depth += 1;
        } else if b == b'}' {
            depth -= 1;
            if depth == 0 {
                if let Some(start) = block_start.take() {
                    visit_block(start, i, block_start_line, &mut findings);
                }
            }
        }
    }

    findings
}

/// Find the value of the first declaration whose property name matches one
/// of `keys` (case-insensitive). Returns the raw hex literal (incl. `#`)
/// when the value is a hex color; None otherwise.
fn find_decl_hex(block: &str, keys: &[&str]) -> Option<String> {
    let lower = block.to_ascii_lowercase();
    let mut best: Option<(usize, String)> = None;
    for key in keys {
        let pat = format!("{key}:");
        let mut search_from = 0usize;
        while let Some(rel) = lower[search_from..].find(&pat) {
            let abs = search_from + rel;
            // Reject "background-color" matching "background" prefix scan: the
            // char immediately preceding the property name (or start) must be
            // non-alphanumeric / non-dash so we don't capture "x-color:".
            let ok_left = abs == 0
                || matches!(
                    bytes_at(&lower, abs - 1),
                    Some(b';' | b'{' | b' ' | b'\t' | b'\n' | b'\r')
                );
            if ok_left {
                let value_start = abs + pat.len();
                if let Some(hex) = first_hex_token(&block[value_start..]) {
                    let take = match best {
                        Some((existing, _)) => abs < existing,
                        None => true,
                    };
                    if take {
                        best = Some((abs, hex));
                    }
                    break;
                }
            }
            search_from = abs + pat.len();
        }
    }
    best.map(|(_, h)| h)
}

fn bytes_at(s: &str, idx: usize) -> Option<u8> {
    s.as_bytes().get(idx).copied()
}

/// Pull the first `#…` hex literal out of the value text. Stops at the
/// first non-hex character after `#`. Returns None if no `#` precedes a
/// `;` or `}` boundary.
fn first_hex_token(value_text: &str) -> Option<String> {
    let trimmed = value_text.trim_start();
    let term = trimmed.find(|c: char| c == ';' || c == '}').unwrap_or(trimmed.len());
    let segment = &trimmed[..term];
    let hash = segment.find('#')?;
    let tail = &segment[hash..];
    let mut end = 1usize;
    for ch in tail[1..].chars() {
        if ch.is_ascii_hexdigit() {
            end += 1;
            if end > 9 {
                break;
            }
        } else {
            break;
        }
    }
    let hex = &tail[..end];
    if hex.len() < 4 {
        None
    } else {
        Some(hex.to_owned())
    }
}

// ---------------------------------------------------------------------------
// Tests.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srgb_to_linear_at_endpoints() {
        assert!((srgb_to_linear(0.0)).abs() < 1e-9);
        assert!((srgb_to_linear(1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn relative_luminance_known_values() {
        assert!((relative_luminance(1.0, 1.0, 1.0) - 1.0).abs() < 1e-9);
        assert!(relative_luminance(0.0, 0.0, 0.0).abs() < 1e-9);
    }

    #[test]
    fn contrast_ratio_white_on_black_is_21_to_1() {
        let r = contrast_ratio((1.0, 1.0, 1.0), (0.0, 0.0, 0.0));
        assert!((r - 21.0).abs() < 1e-6);
    }

    #[test]
    fn contrast_ratio_is_symmetric() {
        let a = contrast_ratio((0.1, 0.2, 0.3), (0.9, 0.8, 0.7));
        let b = contrast_ratio((0.9, 0.8, 0.7), (0.1, 0.2, 0.3));
        assert!((a - b).abs() < 1e-9);
    }

    #[test]
    fn wcag_thresholds_at_boundary() {
        assert!(wcag_aa_pass(WCAG_AA_NORMAL, TextSize::Normal));
        assert!(!wcag_aa_pass(WCAG_AA_NORMAL - 0.01, TextSize::Normal));
        assert!(wcag_aa_pass(WCAG_AA_LARGE, TextSize::Large));
        assert!(wcag_aaa_pass(WCAG_AAA_NORMAL, TextSize::Normal));
    }

    #[test]
    fn parse_hex_three_six_eight_digit() {
        let three = parse_hex("#fff").expect("three-digit hex");
        let six = parse_hex("#ffffff").expect("six-digit hex");
        let eight = parse_hex("#ffffff80").expect("eight-digit hex");
        for c in [three, six, eight] {
            assert!((c.0 - 1.0).abs() < 1e-9);
            assert!((c.1 - 1.0).abs() < 1e-9);
            assert!((c.2 - 1.0).abs() < 1e-9);
        }
    }

    #[test]
    fn parse_hex_rejects_bad_input() {
        assert!(parse_hex("").is_none());
        assert!(parse_hex("#zzzz").is_none());
        assert!(parse_hex("#12345").is_none());
    }

    #[test]
    fn evaluate_contrast_in_css_flags_low_contrast_only() {
        let css = "
.bad { background: #ffffff; color: #eeeeee; }
.ok  { background: #ffffff; color: #000000; }
";
        let findings = evaluate_contrast_in_css(css);
        assert_eq!(findings.len(), 1, "only the low-contrast block should fire; got {findings:?}");
        assert!(findings[0].ratio < WCAG_AA_NORMAL);
    }

    #[test]
    fn evaluate_contrast_in_css_handles_short_hex() {
        let css = ".x { background: #fff; color: #eee; }";
        let findings = evaluate_contrast_in_css(css);
        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn evaluate_contrast_in_css_no_color_is_no_finding() {
        let css = ".x { background: #ffffff; }";
        let findings = evaluate_contrast_in_css(css);
        assert!(findings.is_empty());
    }
}
