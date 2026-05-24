//! C-41 — Skill manifest schema (locally authored).
//!
//! Grounding caveat (per `workflow/papers/scientific-workflow-interface.md`):
//! arXiv:2604.21910 describes Skills as "plain markdown documents with
//! definition tables, synonym mappings, and decision logic" (Section 4) and
//! does NOT publish a formal manifest schema. The [`SkillManifest`] struct
//! defined here is LOCALLY AUTHORED, inspired by the markdown-skills convention
//! the paper observes. Do NOT claim paper-grounded for the schema itself; the
//! effect-size number (44% -> 83.3% full-match intent accuracy on Claude Opus
//! 4.6, S0 vs S3, Section 5.1 Table 2) IS paper-grounded and is the headline
//! motivation for landing the registry, but the wire-level fields here are
//! ours.
//!
//! Frontmatter convention extension: the existing
//! `parse_skill_frontmatter` (crates/cli/src/commands.rs L3863) recognises
//! `name:` and `description:`. C-41 adds five OPTIONAL keys consumed by the
//! registry — `triggers:` (comma-separated phrase list), `version:` (semver
//! string), `inputs:`, `outputs:`, `scope:`. Older skills lacking these
//! fields parse successfully with defaults applied (`triggers`/`inputs`/
//! `outputs` default to empty vectors, `version` defaults to "0.0.0",
//! `scope` defaults to empty string). The registry must therefore tolerate
//! the absent-field case.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::experience::PromptFragment;

/// Origin tag describing where a skill manifest was discovered.
///
/// Mirrors the existing `crates/cli/src/commands.rs::SkillOrigin` enum but
/// kept as a separate type here so the orchestrator crate does not pull the
/// cli crate as a dependency (one-way dep direction preserved). The
/// `LegacyCommandsDir` variant tracks markdown skills under a `commands/`
/// directory (pre-skills-tree layout); `SkillsDir` is the canonical
/// `skills/<name>/SKILL.md` shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillOrigin {
    /// Canonical layout: `<root>/<skill_name>/SKILL.md`.
    SkillsDir,
    /// Legacy layout: `<root>/<skill_name>.md` or
    /// `<root>/<skill_name>/SKILL.md` under a `commands/` parent.
    LegacyCommandsDir,
}

impl SkillOrigin {
    /// Stable wire-protocol string for SSB payloads + JSON serialization.
    /// Lowercase snake_case so the value matches established SSB tag style.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SkillsDir => "skills_dir",
            Self::LegacyCommandsDir => "legacy_commands_dir",
        }
    }
}

/// Loaded skill metadata. Constructed by
/// [`crate::skill_registry::discovery::find_skills`] from a SKILL.md file
/// and any extended frontmatter keys.
///
/// All collection fields default to empty when absent from the source
/// frontmatter so older skills authored against the pre-C-41 frontmatter
/// convention continue to load. The scoring function in
/// [`crate::skill_registry::scoring::score_skill_for_context`] gracefully
/// returns 0.0 when `triggers` is empty (no false positives on legacy
/// skills).
// C-45: dropped `Eq` from the derive — `learned_credit_score: Option<f32>`
// is not `Eq` (NaN-payloads + IEEE-754 quirk). `PartialEq` retained for
// existing assertions; codebase audit (grep) confirms no consumer requires
// `Eq` on the struct directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkillManifest {
    /// Canonical skill name (typically the directory leaf name; overridable
    /// via the `name:` frontmatter key).
    pub name: String,
    /// Semver string. Defaults to "0.0.0" when absent. The locally-authored
    /// 3-int parse in [`crate::skill_registry::scoring`] tolerates malformed
    /// values by treating them as (0, 0, 0) — never panics on bad input.
    pub version: String,
    /// Trigger phrases used by the scoring function. Empty list = skill is
    /// discoverable by exact-name lookup but never auto-selected by context
    /// scoring (intentional fall-through path for legacy skills).
    pub triggers: Vec<String>,
    /// Input field names expected by the skill. Informational metadata for
    /// the dashboard / orchestrator routing surface; not enforced at
    /// dispatch time by the registry itself.
    pub inputs: Vec<String>,
    /// Output field names produced by the skill. Same informational role
    /// as `inputs`.
    pub outputs: Vec<String>,
    /// Scope tag (e.g. "execution", "planning", "research"). Empty string
    /// when absent. Used by the ranker as a soft match boost when the
    /// invocation context carries a matching scope hint.
    pub scope: String,
    /// Origin tag for the discovery root that surfaced this manifest.
    pub origin: SkillOrigin,
    /// Absolute path to the SKILL.md file backing this manifest. The
    /// registry returns this path so callers can re-read the prose
    /// scaffold without re-walking the filesystem.
    pub file_path: PathBuf,
    /// C-45: contrastive credit-attribution score in `[-1.0, +1.0]`
    /// (CANTANTE Section 3.2, arXiv:2605.13295). Populated by the
    /// pilot harness (`crate::credit_attribution::harness`) and read
    /// by future routing-weight surfaces. `None` means "no
    /// measurement yet" (the C-45 pilot run-pod-pending step has not
    /// populated this field for this skill). Defaults to `None` so
    /// legacy SkillManifests parsed before C-45 round-trip unchanged.
    /// Write-through via [`Self::update_credit_score`] enforces the
    /// paper-anchored clamp.
    #[serde(default)]
    pub learned_credit_score: Option<f32>,
    /// C-49: heuristic distilled prompt fragments accumulated for this
    /// skill across successful traces. Empty by default; the experience
    /// module's [`crate::experience::distill_fragments`] is the
    /// authoritative producer, and
    /// [`Self::set_distilled_fragments`] is the single write-through
    /// path so older manifests parsed before C-49 round-trip unchanged
    /// (serde default = empty vec).
    /// See `workflow/papers/iterative-distillation-interface.md` for
    /// the grounding caveats (heuristic, not RL; per-skill not
    /// cross-model; buffer task-dependent).
    #[serde(default)]
    pub distilled_fragments: Vec<PromptFragment>,
}

impl SkillManifest {
    /// C-49 — write-through setter for [`Self::distilled_fragments`].
    /// Keeping the field mutation behind a method matches the C-45
    /// `update_credit_score` precedent and gives future invariant
    /// guards (e.g. a `max_emit` cap) a single chokepoint.
    pub fn set_distilled_fragments(&mut self, fragments: Vec<PromptFragment>) {
        self.distilled_fragments = fragments;
    }

    /// C-45 — write-through setter that clamps the supplied score into
    /// `[-1.0, +1.0]` (CANTANTE Section 3.2 paper-anchored range).
    /// Non-finite inputs clamp to zero so a noisy harness cannot smuggle
    /// `NaN` / `Inf` through the registry surface.
    ///
    /// This is the single mutation point for `learned_credit_score`;
    /// callers MUST go through this method rather than writing the
    /// field directly so the clamp invariant cannot drift.
    pub fn update_credit_score(&mut self, score: f32) {
        let clamped = if score.is_finite() {
            score.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        self.learned_credit_score = Some(clamped);
    }

    /// Parse a `semver`-shaped string into (major, minor, patch). Returns
    /// `(0, 0, 0)` on malformed input — never panics.
    ///
    /// Locally hand-rolled to avoid adding the `semver` crate as a new
    /// dependency per the anti-slopsquatting rule (CLAUDE.md). The 3-int
    /// triple is sufficient for the registry's "newer wins" tie-breaker;
    /// pre-release and build-metadata segments are intentionally ignored.
    #[must_use]
    pub fn parsed_version(&self) -> (u32, u32, u32) {
        let mut iter = self.version.split('.');
        let major = iter
            .next()
            .and_then(|segment| segment.parse::<u32>().ok())
            .unwrap_or(0);
        let minor = iter
            .next()
            .and_then(|segment| segment.parse::<u32>().ok())
            .unwrap_or(0);
        let patch = iter
            .next()
            .and_then(|segment| {
                // Strip pre-release / build suffix if present so e.g.
                // "1.2.3-rc.4" parses as (1, 2, 3).
                let prefix = segment
                    .split(['-', '+'])
                    .next()
                    .unwrap_or(segment);
                prefix.parse::<u32>().ok()
            })
            .unwrap_or(0);
        (major, minor, patch)
    }
}

/// Extended frontmatter view consumed by [`SkillManifest`] construction.
///
/// The existing `parse_skill_frontmatter` in `crates/cli/src/commands.rs`
/// returns `(Option<String>, Option<String>)` for (name, description) only.
/// C-41 introduces a richer parser (`parse_skill_frontmatter_extended`)
/// that returns this struct so the discovery walker can populate
/// `SkillManifest` without re-reading the file. Kept as a separate type so
/// the cli-side parser can stay backwards-compatible.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontmatterFields {
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub triggers: Vec<String>,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub scope: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsed_version_handles_well_formed() {
        let manifest = SkillManifest {
            name: "x".into(),
            version: "1.2.3".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        assert_eq!(manifest.parsed_version(), (1, 2, 3));
    }

    #[test]
    fn parsed_version_handles_pre_release_suffix() {
        let manifest = SkillManifest {
            name: "x".into(),
            version: "2.0.0-rc.1".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        assert_eq!(manifest.parsed_version(), (2, 0, 0));
    }

    #[test]
    fn parsed_version_handles_garbage_without_panic() {
        let manifest = SkillManifest {
            name: "x".into(),
            version: "not-a-version".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        assert_eq!(manifest.parsed_version(), (0, 0, 0));
    }

    #[test]
    fn skill_origin_as_str_stable() {
        assert_eq!(SkillOrigin::SkillsDir.as_str(), "skills_dir");
        assert_eq!(SkillOrigin::LegacyCommandsDir.as_str(), "legacy_commands_dir");
    }

    // C-45 — learned_credit_score field defaults + clamp.
    #[test]
    fn learned_credit_score_defaults_to_none() {
        let m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        assert_eq!(m.learned_credit_score, None);
    }

    #[test]
    fn update_credit_score_clamps_high() {
        let mut m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        m.update_credit_score(2.5);
        assert_eq!(m.learned_credit_score, Some(1.0));
    }

    #[test]
    fn update_credit_score_clamps_low() {
        let mut m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        m.update_credit_score(-7.0);
        assert_eq!(m.learned_credit_score, Some(-1.0));
    }

    #[test]
    fn update_credit_score_nan_becomes_zero() {
        let mut m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        m.update_credit_score(f32::NAN);
        assert_eq!(m.learned_credit_score, Some(0.0));
    }

    // C-49 — distilled_fragments default + write-through setter.
    #[test]
    fn distilled_fragments_default_is_empty() {
        let m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        assert!(m.distilled_fragments.is_empty());
    }

    #[test]
    fn set_distilled_fragments_round_trip() {
        let mut m = SkillManifest {
            name: "x".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("x"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        };
        let frag = PromptFragment::new("decompose first".into(), 4, 0.9, 1000);
        m.set_distilled_fragments(vec![frag.clone()]);
        assert_eq!(m.distilled_fragments.len(), 1);
        assert_eq!(m.distilled_fragments[0].text, "decompose first");
    }
}
