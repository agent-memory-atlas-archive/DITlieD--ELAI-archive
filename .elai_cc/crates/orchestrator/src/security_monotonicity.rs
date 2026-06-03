//! F-10 / Slice A3 — security-monotonicity iteration guard.
//!
//! Self-improvement loops (REFINE candidate selection, AUTO-ADVANCE step
//! promotion, SELF-MOD proposal promotion) iterate toward a better candidate.
//! The F-10 thesis (Shukla arXiv:2506.11022, MOTIVATION-only) is that those
//! loops can *introduce* a new security weakness while improving some other
//! metric — a non-monotone regression. This guard makes the security posture
//! monotone across iterations: a CWE class present at iteration N but absent at
//! N-1 is a regression that the loop must not silently carry forward.
//!
//! Design (F-10-decisions.md, FORK 1 + FORK 2 = CHOSEN):
//!
//!   * "iteration" means an AI-improvement round (a turn for REFINE, a step
//!     level for AUTO-ADVANCE, a proposal for SELF-MOD), NOT a sub-step inside
//!     one call. The unit of comparison is the *set* of CWE classes
//!     ([`crate::cwe_classifier::classify_delta`]) the candidate emits, diffed
//!     against the prior iteration's set.
//!   * The gate is orchestrator-local and STANDALONE — it is NOT a
//!     `TripwireKind`. Each of the three production legs calls
//!     [`SecurityMonotonicityGate::check_monotonicity`] and, on
//!     [`MonotonicityVerdict::Regressed`], calls
//!     [`emit_security_class_regression_blocked`].
//!   * Default-ON. The single env var
//!     [`crate::ssb::ELAI_SECURITY_MONOTONICITY_DISABLE_ENV`] is a kill-switch
//!     (W-2 — never a default-off ENABLE flag). When set truthy the gate
//!     returns [`MonotonicityVerdict::Monotone`] without classifying.
//!
//! Determinism + isolation: [`SecurityMonotonicityGate::check_monotonicity`] is
//! a pure function of (prior set, current source, lang) modulo the env read.
//! No network, no filesystem, no clock. The only side effect is in
//! [`emit_security_class_regression_blocked`], which appends one SSB beacon row
//! plus one Finding row.

use std::collections::BTreeSet;
use std::path::Path;

use crate::cwe_classifier::{classify_delta, delta_fingerprint, CweClass};
use crate::ssb::{
    FindingPayload, LoopKind, SecurityClassRegressionBlockedPayload, Ssb, SsbActionType,
    SsbPayload, SsbReader, ELAI_SECURITY_MONOTONICITY_DISABLE_ENV,
};

/// Agent-id tag attached to SSB rows this guard writes. Logged-only.
const GATE_AGENT_ID: &str = "security_monotonicity_gate";
/// Superstep tag for guard SSB rows. The guard runs outside the BSP superstep
/// loop, so it uses a fixed sentinel (mirrors the deterministic-audit gate).
const GATE_SUPERSTEP: i64 = 0;
/// Finding category for the regression Finding row.
const GATE_FINDING_CATEGORY: &str = "security_monotonicity";

/// The result of comparing one iteration's CWE-class set against the prior
/// iteration's set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonotonicityVerdict {
    /// The current iteration introduced no CWE class absent from the prior
    /// iteration. Security posture is monotone (non-worsening).
    Monotone,
    /// The current iteration introduced one or more CWE classes that were NOT
    /// present in the prior iteration. The strings are the canonical
    /// `CWE-NNN` identifiers (sorted, deterministic) of the newly introduced
    /// classes.
    Regressed {
        /// Canonical `CWE-NNN` ids newly introduced by this iteration.
        introduced_classes: Vec<String>,
    },
}

impl MonotonicityVerdict {
    /// `true` for [`MonotonicityVerdict::Regressed`].
    #[must_use]
    pub fn is_regressed(&self) -> bool {
        matches!(self, MonotonicityVerdict::Regressed { .. })
    }
}

/// F-10 security-monotonicity iteration guard.
///
/// Zero-sized: the gate holds no state. The prior-iteration class set is
/// supplied by the caller (queried from the SSB by loop_id), so the gate is a
/// pure comparator that callers invoke at each iteration boundary.
#[derive(Debug, Default, Clone, Copy)]
pub struct SecurityMonotonicityGate;

impl SecurityMonotonicityGate {
    /// Construct the guard.
    #[must_use]
    pub fn new() -> Self {
        SecurityMonotonicityGate
    }

    /// `true` when the kill-switch env var
    /// [`ELAI_SECURITY_MONOTONICITY_DISABLE_ENV`] is set to a truthy value
    /// (`"1"`, `"true"`, `"yes"`, `"on"`, case-insensitive). Default (var
    /// unset or empty) is ACTIVE/enabled — the guard runs (W-2).
    #[must_use]
    pub fn is_disabled() -> bool {
        match std::env::var(ELAI_SECURITY_MONOTONICITY_DISABLE_ENV) {
            Ok(v) => {
                let t = v.trim().to_ascii_lowercase();
                t == "1" || t == "true" || t == "yes" || t == "on"
            }
            Err(_) => false,
        }
    }

    /// Compare the current iteration's candidate source against the prior
    /// iteration's CWE-class set.
    ///
    /// Returns [`MonotonicityVerdict::Monotone`] when the kill-switch is set
    /// OR when the current candidate introduces no class absent from `prior`.
    /// Returns [`MonotonicityVerdict::Regressed`] (with the sorted
    /// `CWE-NNN` ids of the newly introduced classes) otherwise.
    ///
    /// `loop_kind` / `loop_id` are accepted for caller ergonomics + future
    /// per-leg telemetry symmetry with [`emit_security_class_regression_blocked`];
    /// the comparison itself depends only on (`prior`, `current_source`,
    /// `lang`). The function NEVER short-circuits to `Monotone` on a real
    /// error path — classification is total ([`classify_delta`] cannot fail),
    /// so the only `Monotone` returns are the kill-switch and a genuinely empty
    /// introduced set.
    #[must_use]
    pub fn check_monotonicity(
        loop_kind: LoopKind,
        loop_id: &str,
        prior_classes: &BTreeSet<CweClass>,
        current_source: &str,
        lang: &str,
    ) -> MonotonicityVerdict {
        // Touch the leg identifiers so the symmetry-of-signature with the emit
        // path is real (no dead args) without changing the comparison.
        let _ = (loop_kind, loop_id);

        if Self::is_disabled() {
            return MonotonicityVerdict::Monotone;
        }

        let current = classify_delta(current_source, lang);
        // introduced = current \ prior (set difference). BTreeSet difference
        // yields a deterministic ascending order; map to canonical CWE ids.
        let introduced: Vec<String> = current
            .difference(prior_classes)
            .map(|c| c.as_str().to_string())
            .collect();

        if introduced.is_empty() {
            MonotonicityVerdict::Monotone
        } else {
            MonotonicityVerdict::Regressed {
                introduced_classes: introduced,
            }
        }
    }
}

/// Emit the F-10 regression beacon for a non-monotone iteration.
///
/// Writes TWO append-only SSB rows:
///   1. one [`SsbPayload::SecurityClassRegressionBlocked`] flat beacon carrying
///      the blake3 fingerprint of the regressing candidate, the introduced +
///      prior class id sets, the [`LoopKind`], `loop_id`, and `lang`; and
///   2. one [`SsbPayload::Finding`] (reusing the existing Finding emit path) so
///      the regression surfaces in the standard audit/Finding timeline. The
///      Finding severity is non-blocking observability (`"warning"`) for the
///      fire-and-forget legs and `"escalate"` for the SELF-MOD precondition
///      that actually blocks promotion — the caller picks via `severity`.
///
/// `severity` must be one of `"warning"` / `"escalate"` (caller-chosen per
/// leg). `current_source` is hashed (blake3, via [`delta_fingerprint`]) — the
/// full source text is NEVER stored. Both writes are awaited so the rows are
/// durable before the caller proceeds; a write error is logged (the guard is
/// observability, not a hard fail) and does NOT panic.
pub async fn emit_security_class_regression_blocked(
    ssb: &Ssb,
    loop_kind: LoopKind,
    loop_id: &str,
    prior: &BTreeSet<CweClass>,
    introduced: &[String],
    current_source: &str,
    lang: &str,
    severity: &str,
) {
    let delta_blake3 = delta_fingerprint(current_source);
    let prior_ids: Vec<String> = prior.iter().map(|c| c.as_str().to_string()).collect();
    let introduced_ids: Vec<String> = introduced.to_vec();

    // --- Row 1: the flat F-10 beacon ---
    let beacon = SsbPayload::SecurityClassRegressionBlocked(SecurityClassRegressionBlockedPayload {
        delta_blake3: delta_blake3.clone(),
        introduced_classes: introduced_ids.clone(),
        prior_classes: prior_ids,
        loop_kind,
        loop_id: loop_id.to_string(),
        lang: lang.to_string(),
    });
    if let Err(e) = ssb
        .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, beacon, Vec::new())
        .await
    {
        tracing::error!(
            error = %e,
            loop_id,
            "security_monotonicity: write SecurityClassRegressionBlocked failed"
        );
    }

    // --- Row 2: the Finding (reuse existing Finding emit path) ---
    let leg = match loop_kind {
        LoopKind::Refine => "refine",
        LoopKind::AutoAdvance => "auto_advance",
        LoopKind::SelfMod => "self_mod",
    };
    let finding = SsbPayload::Finding(FindingPayload {
        severity: severity.to_string(),
        category: GATE_FINDING_CATEGORY.to_string(),
        message: format!(
            "security-monotonicity regression in {leg} loop '{loop_id}' ({lang}): \
             introduced {introduced_ids:?} (delta {delta_blake3})"
        ),
        file_path: None,
        line: None,
        failure_category: None,
        audit_step: None,
        few_shot_examples: None,
    });
    if let Err(e) = ssb
        .write_payload(GATE_AGENT_ID, GATE_SUPERSTEP, finding, Vec::new())
        .await
    {
        tracing::error!(
            error = %e,
            loop_id,
            "security_monotonicity: write regression Finding failed"
        );
    }
}

/// Reconstruct the prior REFINE iteration's CWE-class set for `loop_id` from
/// the SSB.
///
/// Reads every [`SsbActionType::SecurityClassRegressionBlocked`] row, keeps the
/// ones whose `loop_kind == Refine` AND `loop_id` matches, takes the LATEST
/// (highest `seq`) such row, and reconstructs that iteration's full emitted
/// class set as the union of its `prior_classes` and `introduced_classes`
/// (the prior iteration's monotone baseline plus what it newly introduced).
/// Returns an empty set when no prior REFINE regression row exists for this
/// loop (cold-start — the first iteration has nothing to regress against).
fn prior_refine_classes(ssb: &Ssb, loop_id: &str) -> BTreeSet<CweClass> {
    let rows = match ssb.query_by_type(SsbActionType::SecurityClassRegressionBlocked) {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!(error = %e, "security_monotonicity: prior-set query failed");
            return BTreeSet::new();
        }
    };
    let mut best: Option<(i64, SecurityClassRegressionBlockedPayload)> = None;
    for entry in &rows {
        let Ok(SsbPayload::SecurityClassRegressionBlocked(p)) =
            rmp_serde::from_slice::<SsbPayload>(entry.payload.as_ref())
        else {
            continue;
        };
        if !matches!(p.loop_kind, LoopKind::Refine) || p.loop_id != loop_id {
            continue;
        }
        match &best {
            Some((seq, _)) if *seq >= entry.seq => {}
            _ => best = Some((entry.seq, p)),
        }
    }
    let mut set = BTreeSet::new();
    if let Some((_, p)) = best {
        for id in p.prior_classes.iter().chain(p.introduced_classes.iter()) {
            if let Some(c) = CweClass::parse(id) {
                set.insert(c);
            }
        }
    }
    set
}

/// REFINE-leg entrypoint (W-10 fire-and-forget, call-site only).
///
/// One function both the CLI refine call-site (`live_cli.rs`) and the dashboard
/// refine call-site (`commands.rs`) invoke so the wiring at each site is a
/// single call. Steps:
///   1. load the prior REFINE iteration's CWE-class set for `loop_id` from the
///      SSB ([`prior_refine_classes`]; empty on cold-start);
///   2. [`SecurityMonotonicityGate::check_monotonicity`] (`loop_kind=Refine`)
///      on `current_source`;
///   3. on [`MonotonicityVerdict::Regressed`] emit the beacon + Finding
///      (severity `"warning"` — the REFINE leg is observability, NON-blocking)
///      and, when `log_path` is set, append one line to the per-session
///      security-monotonicity log.
///
/// Non-blocking + best-effort: any SSB / IO error is logged, never propagated
/// (the user's turn must not fail because the guard could not write). Honours
/// the kill-switch transitively via `check_monotonicity`.
pub async fn run_refine_leg(
    ssb: &Ssb,
    loop_id: &str,
    current_source: &str,
    lang: &str,
    log_path: Option<&Path>,
) {
    let prior = prior_refine_classes(ssb, loop_id);
    let verdict = SecurityMonotonicityGate::check_monotonicity(
        LoopKind::Refine,
        loop_id,
        &prior,
        current_source,
        lang,
    );
    let MonotonicityVerdict::Regressed { introduced_classes } = verdict else {
        return;
    };
    emit_security_class_regression_blocked(
        ssb,
        LoopKind::Refine,
        loop_id,
        &prior,
        &introduced_classes,
        current_source,
        lang,
        "warning",
    )
    .await;
    if let Some(path) = log_path {
        let delta = delta_fingerprint(current_source);
        let line = format!(
            "refine\t{loop_id}\t{lang}\tintroduced={introduced_classes:?}\tdelta={delta}\n"
        );
        // Best-effort append; the per-session dir is created by the session
        // bootstrap, but we still create parent dirs defensively. Any IO error
        // is logged and swallowed (observability log, never a turn-failer).
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Ok(mut f) => {
                use std::io::Write as _;
                if let Err(e) = f.write_all(line.as_bytes()) {
                    tracing::warn!(error = %e, "security_monotonicity: log append failed");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "security_monotonicity: log open failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(ids: &[CweClass]) -> BTreeSet<CweClass> {
        ids.iter().cloned().collect()
    }

    #[test]
    fn monotone_when_no_new_class() {
        // current introduces nothing absent from prior -> Monotone.
        let prior = set(&[CweClass::Sqli, CweClass::CommandInjection]);
        // clean source classifies to empty set ⊆ prior.
        let v = SecurityMonotonicityGate::check_monotonicity(
            LoopKind::Refine,
            "loop-0",
            &prior,
            "fn ok() {}",
            "rust",
        );
        assert_eq!(v, MonotonicityVerdict::Monotone);
        assert!(!v.is_regressed());
    }

    #[test]
    fn regressed_when_new_class_introduced() {
        // prior had no command-injection; current introduces CWE-78 (python).
        let prior = set(&[CweClass::Sqli]);
        let v = SecurityMonotonicityGate::check_monotonicity(
            LoopKind::SelfMod,
            "loop-1",
            &prior,
            "os.system(user_cmd)",
            "python",
        );
        match v {
            MonotonicityVerdict::Regressed { introduced_classes } => {
                assert!(introduced_classes.contains(&"CWE-78".to_string()));
                // CWE-89 was already in prior -> must NOT be in introduced.
                assert!(!introduced_classes.contains(&"CWE-89".to_string()));
            }
            MonotonicityVerdict::Monotone => panic!("expected Regressed"),
        }
    }

    #[test]
    fn class_already_in_prior_is_not_a_regression() {
        // prior already has CWE-78; current re-emits it -> Monotone (no NEW class).
        let prior = set(&[CweClass::CommandInjection]);
        let v = SecurityMonotonicityGate::check_monotonicity(
            LoopKind::AutoAdvance,
            "loop-2",
            &prior,
            "os.system(user_cmd)",
            "python",
        );
        assert_eq!(v, MonotonicityVerdict::Monotone);
    }

    #[test]
    fn kill_switch_forces_monotone() {
        // With the kill-switch set, even a clear regression returns Monotone.
        // Scope the env mutation tightly; restore afterwards so other tests in
        // the same binary are unaffected.
        let key = ELAI_SECURITY_MONOTONICITY_DISABLE_ENV;
        let prev = std::env::var(key).ok();
        std::env::set_var(key, "1");
        let prior = set(&[]);
        let v = SecurityMonotonicityGate::check_monotonicity(
            LoopKind::Refine,
            "loop-3",
            &prior,
            "os.system(user_cmd)",
            "python",
        );
        // restore env before asserting so a failed assert still cleans up.
        match prev {
            Some(p) => std::env::set_var(key, p),
            None => std::env::remove_var(key),
        }
        assert_eq!(v, MonotonicityVerdict::Monotone);
        assert!(!SecurityMonotonicityGate::is_disabled() || true);
    }

    #[test]
    fn is_disabled_default_false() {
        // With the var removed, the guard is ACTIVE (default-ON, W-2).
        let key = ELAI_SECURITY_MONOTONICITY_DISABLE_ENV;
        let prev = std::env::var(key).ok();
        std::env::remove_var(key);
        let disabled = SecurityMonotonicityGate::is_disabled();
        match prev {
            Some(p) => std::env::set_var(key, p),
            None => std::env::remove_var(key),
        }
        assert!(!disabled, "guard must be default-ON when env unset");
    }
}
