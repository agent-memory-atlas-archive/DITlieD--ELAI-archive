//! E-3 (SCHEMA_VERSION 190) -- role-prompt builder + VoiceAnchor.
//!
//! Per-role prompt registry that wraps a base system prompt with a unified
//! voice anchor, calibration hints, sycophancy band, and forbidden-opener
//! list loaded from `.elai_cc/config/voice.toml`. Consumed by
//! [`crate::runtime_orchestrator::RuntimeOrchestrator::try_complete`] BEFORE
//! the adapter dispatch so every backend (sidecar / cloud / CLI) sees the
//! same preamble.
//!
//! Wire-dark default: load_voice_anchor_from_toml resolves a per-role
//! section if present, falling back to `[anchor.default]` /
//! `[calibration.default]` / `[sycophancy.default]` / `[forbidden.default]`
//! when the role section is absent. The builder is pure (no I/O) so unit
//! tests can hand-roll a VoiceAnchor and exercise the wrapping logic
//! without touching disk.
//!
//! See:
//!   - docs/thefinalglance/E-plan/E-3_model_handoff_ux.md (Task 1, Task 10)
//!   - workflow/sessions/s-night-2026-05-30/impl-tasks/TASK-E03-001.md
//!   - workflow/sessions/s-21636/exploration/tranche4-E3-dashboard.md (PATTERN-2 collapse)

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// Default canonical location for the voice.toml file relative to the
/// `.elai_cc/` workspace root. Co-located with `source-tiers.toml` for
/// taxonomy consistency.
pub const DEFAULT_VOICE_TOML_PATH: &str = "config/voice.toml";

/// Per-role tone snapshot loaded from `voice.toml`. Owned, plain data so
/// callers can clone cheaply and pass it through the orchestrator without
/// holding a reference to the on-disk file.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceAnchor {
    /// One-sentence voice anchor (e.g. "ground every claim, refuse to hedge
    /// without evidence"). Rendered as the FIRST line of the wrapped
    /// preamble so the model sees it before the task-specific system text.
    pub anchor: String,
    /// 3-5 calibration hints (e.g. "prefix unsearched claims with
    /// 'From training (unsearched):'"). Rendered as a bulleted list.
    pub calibration: Vec<String>,
    /// Sycophancy guard band in `[0.0, 1.0]`. Higher = stricter refusal
    /// of affirming openers. Currently advisory (consumed by the style
    /// normalizer in sub-phase 4B); kept here so the anchor + band ship
    /// in one struct.
    pub sycophancy_band: f32,
    /// 3-5 banned opener phrases (e.g. "I apologize", "Sure thing!").
    /// Rendered as a "Never open with:" list inside the wrapped preamble.
    pub forbidden: Vec<String>,
}

impl VoiceAnchor {
    /// Construct an empty anchor (no rendering effect). Used by callers
    /// that want to bypass the role-prompt wrap while still satisfying
    /// the `&VoiceAnchor` parameter type.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            anchor: String::new(),
            calibration: Vec::new(),
            sycophancy_band: 0.0,
            forbidden: Vec::new(),
        }
    }

    /// True when the anchor would render no preamble text. Lets the
    /// orchestrator short-circuit the wrap when the role has no anchor
    /// configured (e.g. an experimental role added without a toml entry).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.anchor.trim().is_empty()
            && self.calibration.is_empty()
            && self.forbidden.is_empty()
    }
}

/// Errors surfaced by [`load_voice_anchor_from_toml`].
#[derive(Debug, Error)]
pub enum RolePromptError {
    /// The toml file could not be read from disk.
    #[error("voice.toml not readable at {path}: {source}")]
    ReadFailed {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    /// The toml file is syntactically invalid.
    #[error("voice.toml parse error at {path}: {source}")]
    ParseFailed {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    /// Neither the requested role section nor the `default` fallback
    /// section was found in the toml file. Surfaces the role for log
    /// hygiene so the operator can spot taxonomy drift.
    #[error("no anchor section for role '{role}' and no [anchor.default] fallback in {path}")]
    NoSectionForRole { role: String, path: PathBuf },
    /// The sycophancy_band value was outside the `[0.0, 1.0]` range.
    #[error("sycophancy_band {band} for role '{role}' is outside [0.0, 1.0]")]
    SycophancyOutOfRange { role: String, band: f32 },
}

/// Internal toml schema: `[anchor.<role>]` -> `{ anchor }`,
/// `[calibration.<role>]` -> `{ hints }`,
/// `[sycophancy.<role>]` -> `{ band }`,
/// `[forbidden.<role>]` -> `{ phrases }`.
#[derive(Debug, Deserialize)]
struct VoiceTomlFile {
    #[serde(default)]
    anchor: std::collections::BTreeMap<String, AnchorSection>,
    #[serde(default)]
    calibration: std::collections::BTreeMap<String, CalibrationSection>,
    #[serde(default)]
    sycophancy: std::collections::BTreeMap<String, SycophancySection>,
    #[serde(default)]
    forbidden: std::collections::BTreeMap<String, ForbiddenSection>,
}

#[derive(Debug, Deserialize)]
struct AnchorSection {
    #[serde(default)]
    anchor: String,
}

#[derive(Debug, Deserialize)]
struct CalibrationSection {
    #[serde(default)]
    hints: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SycophancySection {
    #[serde(default)]
    band: f32,
}

#[derive(Debug, Deserialize)]
struct ForbiddenSection {
    #[serde(default)]
    phrases: Vec<String>,
}

/// Resolve the canonical voice.toml path. Looks in two anchored locations
/// in order:
///   1. `<workspace_root>/.elai_cc/config/voice.toml` -- production layout
///      when CARGO_MANIFEST_DIR points inside `crates/orchestrator/`.
///   2. `<workspace_root>/config/voice.toml` -- when the .elai_cc/ root
///      itself is the workspace.
///
/// Returns the FIRST existing path. Falls back to option 1 if neither
/// exists so the caller error path surfaces the canonical location.
#[must_use]
pub fn default_voice_toml_path() -> PathBuf {
    // CARGO_MANIFEST_DIR is the crate dir at compile time. Walk up to
    // .elai_cc/ root then into config/.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_owned());
    let from_crate = Path::new(&manifest_dir)
        .ancestors()
        .find(|p| p.join(".elai_cc").is_dir())
        .map(|p| p.join(".elai_cc").join(DEFAULT_VOICE_TOML_PATH));
    if let Some(p) = from_crate {
        if p.is_file() {
            return p;
        }
    }
    // Fall back to <crate>/../../config/voice.toml (when running from
    // inside .elai_cc/).
    let from_workspace = Path::new(&manifest_dir)
        .ancestors()
        .find(|p| p.join("config").is_dir() && p.join("crates").is_dir())
        .map(|p| p.join(DEFAULT_VOICE_TOML_PATH));
    if let Some(p) = from_workspace {
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from(DEFAULT_VOICE_TOML_PATH)
}

/// Load a [`VoiceAnchor`] for `role` from the toml file at `path`.
///
/// Resolution: if the role-specific section exists, prefer it; otherwise
/// fall back to the `default` section. If neither exists, return
/// [`RolePromptError::NoSectionForRole`].
///
/// The four sections (anchor / calibration / sycophancy / forbidden) are
/// resolved independently -- a role may have a specific `[anchor.coder]`
/// but inherit `[calibration.default]`. This matches the precedent set by
/// `.elai_cc/config/source-tiers.toml` where per-source overrides layer
/// on top of a default block.
pub fn load_voice_anchor_from_toml(role: &str, path: &Path) -> Result<VoiceAnchor, RolePromptError> {
    let text = fs::read_to_string(path).map_err(|source| RolePromptError::ReadFailed {
        path: path.to_path_buf(),
        source,
    })?;
    let parsed: VoiceTomlFile = toml::from_str(&text).map_err(|source| RolePromptError::ParseFailed {
        path: path.to_path_buf(),
        source,
    })?;

    let role_key = role.to_owned();
    let default_key = "default".to_owned();

    let anchor_section = parsed
        .anchor
        .get(&role_key)
        .or_else(|| parsed.anchor.get(&default_key));
    let calibration_section = parsed
        .calibration
        .get(&role_key)
        .or_else(|| parsed.calibration.get(&default_key));
    let sycophancy_section = parsed
        .sycophancy
        .get(&role_key)
        .or_else(|| parsed.sycophancy.get(&default_key));
    let forbidden_section = parsed
        .forbidden
        .get(&role_key)
        .or_else(|| parsed.forbidden.get(&default_key));

    let anchor_text = match anchor_section {
        Some(s) => s.anchor.clone(),
        None => {
            return Err(RolePromptError::NoSectionForRole {
                role: role_key,
                path: path.to_path_buf(),
            });
        }
    };

    let calibration = calibration_section
        .map(|s| s.hints.clone())
        .unwrap_or_default();
    let sycophancy_band = sycophancy_section.map(|s| s.band).unwrap_or(0.0_f32);
    let forbidden = forbidden_section
        .map(|s| s.phrases.clone())
        .unwrap_or_default();

    if !(0.0..=1.0).contains(&sycophancy_band) {
        return Err(RolePromptError::SycophancyOutOfRange {
            role: role_key,
            band: sycophancy_band,
        });
    }

    Ok(VoiceAnchor {
        anchor: anchor_text,
        calibration,
        sycophancy_band,
        forbidden,
    })
}

/// Build the unified role-aware system prompt by concatenating the voice
/// anchor + calibration hints + forbidden-opener list ABOVE the base
/// system text.
///
/// Layout (each section separated by a blank line):
///
/// ```text
/// Role anchor: <anchor>
///
/// Calibration:
///   - <hint 1>
///   - <hint 2>
///
/// Never open with:
///   - "<phrase 1>"
///   - "<phrase 2>"
///
/// <base_system>
/// ```
///
/// If [`VoiceAnchor::is_empty`] returns true, the base system text is
/// returned unchanged (preserves the wire-dark default behaviour). If
/// `base_system` is empty, the preamble is returned without trailing
/// blank-line padding.
#[must_use]
pub fn build_role_prompt(role: &str, base_system: &str, anchor: &VoiceAnchor) -> String {
    if anchor.is_empty() {
        return base_system.to_owned();
    }
    let mut out = String::with_capacity(base_system.len() + 512);
    if !anchor.anchor.trim().is_empty() {
        out.push_str(&format!("Role anchor ({}): {}\n", role, anchor.anchor.trim()));
    }
    if !anchor.calibration.is_empty() {
        out.push('\n');
        out.push_str("Calibration:\n");
        for hint in &anchor.calibration {
            let trimmed = hint.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push_str("  - ");
            out.push_str(trimmed);
            out.push('\n');
        }
    }
    if !anchor.forbidden.is_empty() {
        out.push('\n');
        out.push_str("Never open with:\n");
        for phrase in &anchor.forbidden {
            let trimmed = phrase.trim();
            if trimmed.is_empty() {
                continue;
            }
            out.push_str("  - \"");
            out.push_str(trimmed);
            out.push_str("\"\n");
        }
    }
    if !base_system.is_empty() {
        out.push('\n');
        out.push_str(base_system);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_toml(body: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create tempfile");
        f.write_all(body.as_bytes()).expect("write tempfile body");
        f.flush().expect("flush tempfile");
        f
    }

    #[test]
    fn build_role_prompt_with_empty_anchor_returns_base_unchanged() {
        let anchor = VoiceAnchor::empty();
        let out = build_role_prompt("coder", "BASE SYS", &anchor);
        assert_eq!(out, "BASE SYS");
    }

    #[test]
    fn build_role_prompt_concatenates_anchor_then_calibration_then_forbidden_then_base() {
        let anchor = VoiceAnchor {
            anchor: "ground every claim".to_owned(),
            calibration: vec!["cite source".to_owned(), "no hedging".to_owned()],
            sycophancy_band: 0.5,
            forbidden: vec!["Sure thing!".to_owned()],
        };
        let out = build_role_prompt("coder", "BASE SYS", &anchor);
        // Anchor first, base last.
        let anchor_pos = out.find("Role anchor (coder): ground every claim").expect("anchor present");
        let cal_pos = out.find("Calibration:").expect("calibration block present");
        let forb_pos = out.find("Never open with:").expect("forbidden block present");
        let base_pos = out.find("BASE SYS").expect("base present");
        assert!(anchor_pos < cal_pos);
        assert!(cal_pos < forb_pos);
        assert!(forb_pos < base_pos);
        assert!(out.contains("- cite source"));
        assert!(out.contains("- \"Sure thing!\""));
    }

    #[test]
    fn load_voice_anchor_resolves_per_role_then_default() {
        let toml = r#"
[anchor.default]
anchor = "default voice"

[anchor.coder]
anchor = "coder voice"

[calibration.default]
hints = ["d-hint"]

[sycophancy.default]
band = 0.4

[forbidden.default]
phrases = ["I apologize"]
"#;
        let tmp = write_temp_toml(toml);
        let coder = load_voice_anchor_from_toml("coder", tmp.path()).expect("coder load");
        // Per-role anchor wins over default.
        assert_eq!(coder.anchor, "coder voice");
        // Calibration / sycophancy / forbidden fall through to default.
        assert_eq!(coder.calibration, vec!["d-hint".to_owned()]);
        assert!((coder.sycophancy_band - 0.4).abs() < 1e-6);
        assert_eq!(coder.forbidden, vec!["I apologize".to_owned()]);

        let planner = load_voice_anchor_from_toml("planner", tmp.path()).expect("planner load");
        // Planner falls through to [anchor.default].
        assert_eq!(planner.anchor, "default voice");
    }

    #[test]
    fn load_voice_anchor_rejects_out_of_range_band() {
        let toml = r#"
[anchor.coder]
anchor = "x"

[sycophancy.coder]
band = 1.5
"#;
        let tmp = write_temp_toml(toml);
        let err = load_voice_anchor_from_toml("coder", tmp.path()).expect_err("should reject");
        match err {
            RolePromptError::SycophancyOutOfRange { role, band } => {
                assert_eq!(role, "coder");
                assert!((band - 1.5).abs() < 1e-6);
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    fn load_voice_anchor_missing_section_returns_error() {
        let toml = r#"
[calibration.other]
hints = ["x"]
"#;
        let tmp = write_temp_toml(toml);
        let err = load_voice_anchor_from_toml("coder", tmp.path()).expect_err("should fail");
        match err {
            RolePromptError::NoSectionForRole { role, .. } => {
                assert_eq!(role, "coder");
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }
}
