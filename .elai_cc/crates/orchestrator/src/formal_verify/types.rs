//! E-53 — formal-verifier types (kind discriminator + outcome enum).
//!
//! Distinct from `verification_result::VerifierKind` (which is a
//! consensus-aggregator label) — these types name the CONCRETE
//! external toolchain dispatched, and the per-toolchain outcome
//! shape.
//!
//! GROUNDING:
//!   - C-52 `dafny_verify::DafnyVerifyResult` for the Dafny tier
//!     (paper arXiv:2604.22601 §3 tiered prompting + vacuous-guard).
//!   - L3-lock note in `verification_result.rs:33-35`: VerifierKind
//!     is the consensus-source family; the toolchain enum here lives
//!     in its own namespace and never collides with that lock.
//!   - WIRE-DARK / RUNPOD-PENDING posture per C-52 cli_shellout
//!     pattern (line 11-13): a missing binary surfaces as
//!     `RunpodPending`, NOT panic, so the orchestrator records the
//!     absence and proceeds via the infeasibility-note path.
//!
//! NO new external dep is added in this ship — z3 / kani-verifier /
//! verus / dafny / lean-copilot are ALL detected at runtime via the
//! existing shellout pattern; an absent binary returns
//! `RunpodPending`. Anti-slopsquatting per CLAUDE.md: a Cargo.toml
//! line is the only mechanism to introduce a dep, and this ship
//! adds none.

use serde::{Deserialize, Serialize};

/// Concrete formal-verifier toolchain dispatched by the pipeline.
/// Distinct from [`crate::verification_result::VerifierKind`]; the
/// latter is the consensus-aggregator family label, the former is
/// the toolchain executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerifierKind {
    /// Z3 SMT solver shellout (NEW; routed through the
    /// `verification_result::VerifierKind::Z3SmtCheck` family for
    /// consensus). Binary is RUNPOD-PENDING on local dev hosts.
    Z3Smt,
    /// Dafny verifier shellout — composes with
    /// `crate::dafny_verify::invoke_dafny_cli` (C-52). Binary is
    /// RUNPOD-PENDING locally per the C-52 mod doc.
    Dafny,
    /// Kani harness — `cargo-kani`. Binary is RUNPOD-PENDING locally.
    /// The harness is reserved for memory-safety / panic-freedom
    /// invariants on Rust code; scaffolding-only in lean E-53 ship.
    Kani,
    /// Verus annotation check — reserved for FSM / IPC / KV-cache
    /// invariants. Toolchain is RUNPOD-PENDING; scaffolding-only.
    Verus,
}

impl VerifierKind {
    /// Snake-case wire label used in SSB rows + dashboard rendering.
    /// Stable across runs; new variants append to keep wire format
    /// monotone.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Z3Smt => "z3_smt",
            Self::Dafny => "dafny",
            Self::Kani => "kani",
            Self::Verus => "verus",
        }
    }

    /// Whether the local dev host is expected to have this binary
    /// installed. All four return `false` today — none are present on
    /// the local Windows dev box per C-52 + E-53 context-package
    /// UPSTREAM-8. The host-detection call (`which`-style probe) is
    /// in `crate::formal_verify::orchestrator::probe_toolchain`.
    #[must_use]
    pub fn expected_local_install(self) -> bool {
        false
    }
}

/// All four toolchain variants in declaration order. Used by the CLI
/// `formal-verify status` subverb to enumerate the available
/// toolchains + each one's RUNPOD-PENDING status.
pub const ALL_VERIFIER_KINDS: [VerifierKind; 4] = [
    VerifierKind::Z3Smt,
    VerifierKind::Dafny,
    VerifierKind::Kani,
    VerifierKind::Verus,
];

/// Outcome of a single formal-verifier dispatch.
///
/// The four variants match the documented design space:
///   - `Verified`: spec PASSED + vacuous-guard ALSO PASSED (per C-52
///     paper §3 — raw verifier exit-zero is insufficient; functional
///     tests must also pass).
///   - `Refuted { counterexample }`: spec FAILED with a concrete
///     witness. The counterexample is privacy-preserved as a string
///     hash on the SSB row (full text out-of-band).
///   - `Inapplicable { reason }`: verifier could not apply (Z3
///     Unknown / Timeout, Kani NotApplicable, Verus opt-out via
///     env). Does NOT contribute to consensus per the L3-lock comment
///     on `verification_result::VerifierKind`.
///   - `RunpodPending`: external binary absent on local host. The
///     pipeline returns this when `which`-probe fails; the SSB row
///     fires so the dashboard can track the deferred load.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VerifierOutcome {
    /// Spec verified AND vacuous-guard passed.
    Verified,
    /// Spec falsified with concrete witness. Witness text is held
    /// out-of-band; only the discriminator + a short label live on
    /// the SSB row.
    Refuted {
        /// Short label (length-capped 256 bytes) describing the
        /// witness category — full counterexample lives in audit
        /// logs.
        counterexample_label: String,
    },
    /// Verifier could not apply to this input (Unknown / Timeout /
    /// NotApplicable). Caller MUST NOT count toward consensus.
    Inapplicable {
        /// Short reason snake-case-ish ("z3_unknown", "kani_timeout",
        /// "verus_opt_out").
        reason: String,
    },
    /// External binary absent on the host. The pipeline records this
    /// to inform downstream consumers the verifier is RUNPOD-PENDING.
    RunpodPending,
}

impl VerifierOutcome {
    /// Snake-case discriminator for SSB wire format. Stable across
    /// runs; tests + dashboard consume this.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Refuted { .. } => "refuted",
            Self::Inapplicable { .. } => "inapplicable",
            Self::RunpodPending => "runpod_pending",
        }
    }

    /// Whether the outcome counts as a consensus signal. `Verified`
    /// + `Refuted` count; `Inapplicable` + `RunpodPending` DO NOT
    /// (the L3-lock comment on `verification_result::VerifierKind`
    /// makes this rule explicit — abstain on Unknown/Timeout).
    #[must_use]
    pub fn counts_toward_consensus(&self) -> bool {
        matches!(self, Self::Verified | Self::Refuted { .. })
    }
}

/// Input request to the formal-verify pipeline. Carries the target
/// path + chosen verifier + an optional cost cap (milliseconds).
///
/// `target_path` is a string (not `PathBuf`) so the CLI parser can
/// pass it through unchanged; the pipeline's shellout layer converts
/// to `Path` at the boundary. `cost_cap_ms` is advisory — each
/// toolchain has its own paper-grounded cap (Dafny 30s per C-52
/// MAX_HEAL_ITER * per-iter budget; Z3 5s default; Kani 30s; Verus
/// 5min). The pipeline takes `min(cost_cap_ms, toolchain_default)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyRequest {
    /// Target file or directory the verifier should examine.
    pub target_path: String,
    /// Toolchain to dispatch.
    pub verifier: VerifierKind,
    /// Optional advisory wall-clock cap. None = use toolchain default.
    pub cost_cap_ms: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_kind_wire_tags_are_snake_case() {
        assert_eq!(VerifierKind::Z3Smt.as_str(), "z3_smt");
        assert_eq!(VerifierKind::Dafny.as_str(), "dafny");
        assert_eq!(VerifierKind::Kani.as_str(), "kani");
        assert_eq!(VerifierKind::Verus.as_str(), "verus");
    }

    #[test]
    fn all_verifier_kinds_has_four() {
        assert_eq!(ALL_VERIFIER_KINDS.len(), 4);
    }

    #[test]
    fn no_verifier_expected_local_install() {
        // RUNPOD-PENDING posture: all 4 binaries absent on dev host
        // per C-52 cross-check + E-53 context-package UPSTREAM-8.
        for k in ALL_VERIFIER_KINDS.iter() {
            assert!(!k.expected_local_install());
        }
    }

    #[test]
    fn outcome_discriminator_strings() {
        assert_eq!(VerifierOutcome::Verified.as_str(), "verified");
        assert_eq!(
            VerifierOutcome::Refuted {
                counterexample_label: "x".into()
            }
            .as_str(),
            "refuted"
        );
        assert_eq!(
            VerifierOutcome::Inapplicable {
                reason: "y".into()
            }
            .as_str(),
            "inapplicable"
        );
        assert_eq!(VerifierOutcome::RunpodPending.as_str(), "runpod_pending");
    }

    #[test]
    fn only_verified_and_refuted_count_toward_consensus() {
        assert!(VerifierOutcome::Verified.counts_toward_consensus());
        assert!(VerifierOutcome::Refuted {
            counterexample_label: "x".into()
        }
        .counts_toward_consensus());
        assert!(!VerifierOutcome::Inapplicable {
            reason: "y".into()
        }
        .counts_toward_consensus());
        assert!(!VerifierOutcome::RunpodPending.counts_toward_consensus());
    }

    #[test]
    fn outcome_round_trip_serde_preserves_discriminator() {
        for o in [
            VerifierOutcome::Verified,
            VerifierOutcome::Refuted {
                counterexample_label: "auth_bypass".into(),
            },
            VerifierOutcome::Inapplicable {
                reason: "z3_unknown".into(),
            },
            VerifierOutcome::RunpodPending,
        ] {
            let s = serde_json::to_string(&o).expect("serialize");
            let back: VerifierOutcome = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(o, back);
            // Discriminator label survives the round-trip.
            assert_eq!(o.as_str(), back.as_str());
        }
    }
}
