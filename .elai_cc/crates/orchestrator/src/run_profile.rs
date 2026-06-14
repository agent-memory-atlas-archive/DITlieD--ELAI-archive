//! I-42 — critique-only operating profile.
//!
//! ELAI's default operating profile is `Orchestrate`: spawned executor/fixer
//! agents write code autonomously. The `CritiqueOnly` profile is the
//! human-as-verifier counterpart — the OPERATOR writes the code, spawned
//! agents are dispatched ONLY for critique (the I-41 LensSet at design time,
//! the shipped auditor + off-agent spine at ship time), and NO executor/fixer
//! agent ever touches disk.
//!
//! The structural guarantee lives at the single executor-install chokepoint in
//! [`crate::runtime_setup::apply_task_runner_wiring`]: when the resolved
//! profile is `CritiqueOnly`, the production `baby_step_executor` is forced to
//! `None` BEFORE it is assigned to `TaskRunnerConfig::baby_step_executor`. The
//! runner falls back to the non-disk-mutating synthetic path, and because the
//! fixer loop (`fix_and_rerun_until_green`) derives from the SAME
//! `baby_step_executor`, that ONE gate covers BOTH the executor and the fixer.
//! The refusal is fail-closed by construction: there is nothing to spawn.
//!
//! `RunProfile` is ORTHOGONAL to [`crate::autonomous_gate::GateMode`]
//! (autonomy of the gate vs whether executor agents are spawned at all). It
//! mirrors `GateMode`'s flag-plumbing pattern but is a sibling enum — the two
//! are never overloaded (I-42 stub_ban).

use serde::{Deserialize, Serialize};

/// Default profile label parsed by [`RunProfile::from_cli_args`].
pub const RUN_PROFILE_ORCHESTRATE: &str = "orchestrate";
/// Critique-only profile label parsed by [`RunProfile::from_cli_args`].
pub const RUN_PROFILE_CRITIQUE_ONLY: &str = "critique-only";

/// The run operating profile selected at dispatch time via `--profile`.
///
/// Mirrors the [`crate::autonomous_gate::GateMode`] `Copy` enum idiom: a small
/// `Copy` enum with `Display`, `from_str`, and `from_cli_args(Option<&str>)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunProfile {
    /// Default. Spawned executor/fixer agents write code autonomously.
    Orchestrate,
    /// Human-as-verifier. The operator writes the code; the executor/fixer
    /// spawn is structurally refused at the install chokepoint. Agents are
    /// dispatched only for critique (design-time lenses, ship-time spine).
    CritiqueOnly,
}

impl Default for RunProfile {
    fn default() -> Self {
        RunProfile::Orchestrate
    }
}

impl std::fmt::Display for RunProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunProfile::Orchestrate => f.write_str(RUN_PROFILE_ORCHESTRATE),
            RunProfile::CritiqueOnly => f.write_str(RUN_PROFILE_CRITIQUE_ONLY),
        }
    }
}

impl RunProfile {
    /// Parse a user-supplied profile string. Lower-cased and trimmed first so
    /// `--profile CRITIQUE-ONLY ` and `--profile critique-only` are equivalent.
    /// Returns `None` on an unrecognised label (the caller validates / rejects).
    #[must_use]
    pub fn from_str(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            RUN_PROFILE_ORCHESTRATE => Some(RunProfile::Orchestrate),
            RUN_PROFILE_CRITIQUE_ONLY => Some(RunProfile::CritiqueOnly),
            _ => None,
        }
    }

    /// Resolve the run profile from a parsed `--profile <value>` CLI flag.
    /// Returns [`RunProfile::Orchestrate`] when `raw` is `None` (the flag was
    /// omitted) or when the value is unrecognised, preserving the default
    /// autonomous-orchestrate posture at the production wiring point. Unknown
    /// values are rejected earlier at parse time in
    /// `crate::args::parse_run_subcommand_args`, so an unrecognised value here
    /// can only arise from a non-CLI caller and safely defaults.
    #[must_use]
    pub fn from_cli_args(raw: Option<&str>) -> Self {
        match raw.and_then(Self::from_str) {
            Some(profile) => profile,
            None => RunProfile::Orchestrate,
        }
    }

    /// Structural spawn gate. `true` when this profile REFUSES the
    /// executor/fixer agent spawn — i.e. the production `baby_step_executor`
    /// must be forced to `None` at the install chokepoint so no disk-mutating
    /// agent is ever installed. Consulted by
    /// [`crate::runtime_setup::apply_task_runner_wiring`].
    #[must_use]
    pub fn refuses_executor_spawn(self) -> bool {
        matches!(self, RunProfile::CritiqueOnly)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_orchestrate() {
        assert_eq!(RunProfile::default(), RunProfile::Orchestrate);
    }

    #[test]
    fn from_cli_args_none_defaults_orchestrate() {
        assert_eq!(RunProfile::from_cli_args(None), RunProfile::Orchestrate);
    }

    #[test]
    fn from_cli_args_parses_both_labels() {
        assert_eq!(
            RunProfile::from_cli_args(Some("orchestrate")),
            RunProfile::Orchestrate
        );
        assert_eq!(
            RunProfile::from_cli_args(Some("critique-only")),
            RunProfile::CritiqueOnly
        );
        // Case/whitespace insensitive.
        assert_eq!(
            RunProfile::from_cli_args(Some("  CRITIQUE-ONLY ")),
            RunProfile::CritiqueOnly
        );
    }

    #[test]
    fn unrecognised_defaults_orchestrate() {
        assert_eq!(RunProfile::from_str("bogus"), None);
        assert_eq!(RunProfile::from_cli_args(Some("bogus")), RunProfile::Orchestrate);
    }

    #[test]
    fn only_critique_only_refuses_spawn() {
        assert!(RunProfile::CritiqueOnly.refuses_executor_spawn());
        assert!(!RunProfile::Orchestrate.refuses_executor_spawn());
    }

    #[test]
    fn display_round_trips_labels() {
        assert_eq!(RunProfile::Orchestrate.to_string(), "orchestrate");
        assert_eq!(RunProfile::CritiqueOnly.to_string(), "critique-only");
    }
}
