//! C-45 — write-through helper feeding a credit score into a C-41
//! `SkillManifest`.
//!
//! The C-41 SkillRegistry owns the canonical metadata surface for
//! routing weights. C-45 extends `SkillManifest` with an optional
//! `learned_credit_score: Option<f32>` field. This helper provides a
//! single point of mutation so the harness, the CLI, and the future
//! dashboard refresh path cannot drift on the clamp semantics.
//!
//! Clamp policy: identical to [`super::types::CreditScore::clamped`]
//! (the paper-anchored `[-1.0, +1.0]` range). Non-finite values clamp
//! to zero.

use super::types::CreditScore;
use crate::skill_registry::SkillManifest;

/// Write a credit score onto a `SkillManifest`'s `learned_credit_score`
/// field, applying the same `[-1.0, +1.0]` clamp used everywhere else
/// in the module.
///
/// Safe to call repeatedly; later calls overwrite earlier values
/// (the manifest holds the latest measurement, not a history).
pub fn apply_credit_to_manifest(manifest: &mut SkillManifest, raw_score: f32) {
    let clamped = CreditScore::clamped(raw_score);
    manifest.update_credit_score(clamped.value());
}

/// C-46 — cold-start-safe read helper for the selective-delegation
/// path.
///
/// Copies a [`super::types::CreditReport`]'s `HashMap` of per-role
/// credits into a `BTreeMap<AgentRole, f32>` so the consumer
/// (`routing::SelectiveDelegator`) gets deterministic iteration order
/// (CLAUDE.md anti-stale-training BTreeMap-for-iteration rule).
///
/// Returns `None` when:
///   - `enabled = false` (the caller's gate signal from
///     `[agent.credit_attribution].enabled` — production write path
///     is OFF by default).
///   - `report.is_none()` (no harness has run in this session yet).
///
/// `None` means "treat every role as cold-start" — `SelectiveDelegator`
/// is documented to handle this case via [`super::super::routing::selective::COLD_START_CREDIT`].
#[must_use]
pub fn current_credit_weights(
    enabled: bool,
    report: Option<&super::types::CreditReport>,
) -> Option<std::collections::BTreeMap<elai_ipc_types::AgentRole, f32>> {
    if !enabled {
        return None;
    }
    let report = report?;
    let mut out: std::collections::BTreeMap<elai_ipc_types::AgentRole, f32> =
        std::collections::BTreeMap::new();
    for (role, score) in &report.per_role {
        out.insert(*role, *score);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::skill_registry::{SkillManifest, SkillOrigin};

    fn fixture_manifest() -> SkillManifest {
        SkillManifest {
            name: "test-skill".into(),
            version: "0.0.0".into(),
            triggers: vec![],
            inputs: vec![],
            outputs: vec![],
            scope: String::new(),
            origin: SkillOrigin::SkillsDir,
            file_path: PathBuf::from("/tmp/test"),
            learned_credit_score: None,
            distilled_fragments: Vec::new(),
        }
    }

    #[test]
    fn apply_writes_in_band() {
        let mut m = fixture_manifest();
        apply_credit_to_manifest(&mut m, 0.42);
        assert_eq!(m.learned_credit_score, Some(0.42));
    }

    #[test]
    fn apply_clamps_above_one() {
        let mut m = fixture_manifest();
        apply_credit_to_manifest(&mut m, 5.0);
        assert_eq!(m.learned_credit_score, Some(1.0));
    }

    #[test]
    fn apply_clamps_below_minus_one() {
        let mut m = fixture_manifest();
        apply_credit_to_manifest(&mut m, -3.0);
        assert_eq!(m.learned_credit_score, Some(-1.0));
    }

    #[test]
    fn apply_nan_clamps_to_zero() {
        let mut m = fixture_manifest();
        apply_credit_to_manifest(&mut m, f32::NAN);
        assert_eq!(m.learned_credit_score, Some(0.0));
    }

    #[test]
    fn current_credit_weights_returns_none_when_disabled() {
        let report = super::super::CreditReport::new(0);
        let out = super::current_credit_weights(false, Some(&report));
        assert!(out.is_none());
    }

    #[test]
    fn current_credit_weights_returns_none_when_no_report() {
        let out = super::current_credit_weights(true, None);
        assert!(out.is_none());
    }

    #[test]
    fn current_credit_weights_copies_into_btree() {
        use elai_ipc_types::AgentRole;
        let mut report = super::super::CreditReport::new(0);
        report.record(AgentRole::Executor, super::super::CreditScore::clamped(0.42));
        report.record(AgentRole::Auditor, super::super::CreditScore::clamped(-0.3));
        let out =
            super::current_credit_weights(true, Some(&report)).expect("enabled + report");
        assert_eq!(out.get(&AgentRole::Executor).copied(), Some(0.42));
        assert_eq!(out.get(&AgentRole::Auditor).copied(), Some(-0.3));
        assert_eq!(out.len(), 2);
    }
}
