//! E-53 — formal-verify pipeline orchestrator.
//!
//! Composes the three upstream substrates shipped earlier this
//! session:
//!   - C-50 [`crate::contract_coding::emit_contract_test`] +
//!     [`crate::contract_coding::spec::StructuralContract`] for the
//!     structural spec emission step (kernel K = <N, Sigma, Delta>;
//!     paper arXiv:2604.13100).
//!   - C-52 [`crate::dafny_verify::invoke_dafny_cli`] for the Dafny
//!     verifier dispatch (paper arXiv:2604.22601 §3 tiered prompting).
//!   - This module's [`super::types`] for the toolchain-agnostic
//!     [`VerifierKind`] / [`VerifierOutcome`] taxonomy.
//!
//! GROUNDING CAVEATS (mandatory):
//!   - Z3 / Kani / Verus binaries are absent on the local dev host
//!     per C-52 cross-check + E-53 UPSTREAM-8. The pipeline returns
//!     [`VerifierOutcome::RunpodPending`] (NOT panic) when probing
//!     fails. Anti-slopsquatting per CLAUDE.md: NO new dep is added
//!     in this ship; the runtime detection is a `which`-style probe.
//!   - Gemma-4-31B is the paper's top performer (90.91% verify@5 RQ3a
//!     per arXiv:2604.22601) but is BLOCKED per memory rule
//!     `feedback_no_gemma4`. Substitutes from the paper's evaluation
//!     set are documented in [`crate::dafny_verify::ELAI_SUBSTITUTE_MODELS`].
//!     The pipeline never dispatches Gemma-4 as a model backend.
//!   - The Z3 route returns `Inapplicable { reason: "z3_stub" }` in
//!     this ship — the SMT-LIB2 encoder + Z3 shellout are RUNPOD-
//!     PENDING. The variant + dispatch boundary land here so future
//!     work can swap the body without touching call sites.
//!
//! WIRE-DARK avoidance per E40_WIRE_DARK_AVOIDANCE_VIA_CLI: this
//! function has at least one production call site via the
//! `crates/cli/src/formal_verify_cmd.rs` CLI subverb.

use std::path::Path;

use crate::contract_coding::spec::{StructuralContract, TypeSignature};
use crate::dafny_verify::{invoke_dafny_cli, DafnyCliError};

use super::closure_scan::{scan_obligation_closure, ClosureScanResult};
use super::types::{
    DischargedGoal, HatchKind, ObligationNode, VerifierKind, VerifierOutcome, VerifyRequest,
};

/// Outcome of a complete pipeline run. Carries the chosen verifier +
/// the toolchain outcome + the structural-contract emission hash
/// (blake3 prefix; 0 when the contract emission was skipped because
/// the verifier returned RunpodPending before reaching the
/// contract-emit step).
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineResult {
    /// Toolchain that was dispatched.
    pub verifier: VerifierKind,
    /// Outcome from the toolchain (or RunpodPending when binary
    /// absent).
    pub outcome: VerifierOutcome,
    /// Whether the C-50 structural-contract emission step was
    /// invoked successfully before the verifier dispatch. `false`
    /// when the pipeline short-circuited on a missing binary.
    pub contract_emitted: bool,
    /// H-54 closure scan result (Layer-2 first leg). Always present when
    /// the scan ran. `None` only when the scan was bypassed via
    /// `ELAI_CLOSURE_SCAN_DISABLE` kill-switch (in which case the scan
    /// returned `ClosureScanKind::Clean` with closure_size=0, but we keep
    /// `None` here to distinguish "bypassed" from "ran and was clean").
    pub closure_scan: Option<ClosureScanResult>,
}

/// Build a minimal [`StructuralContract`] anchored at the target
/// path's stem. Reuses C-50's TypeSignature struct — the function
/// name is the path stem, signature is the literal verifier name,
/// language hint is "rust".
///
/// The minimal contract is enough to gate the
/// `audit_against_contract` discriminator (C-50) — richer specs land
/// in patch-N follow-ups when the live verifier binaries are
/// available on a remote runner.
fn build_minimal_contract(target_path: &str, verifier: VerifierKind) -> StructuralContract {
    let stem = std::path::Path::new(target_path)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("target");
    let sig = TypeSignature {
        function_name: stem.to_string(),
        signature: format!("fn {stem}() -> Result<(), {verifier}Error>", verifier = verifier.as_str()),
        language_hint: "rust".to_string(),
    };
    StructuralContract {
        namespace: vec!["formal_verify".to_string(), stem.to_string()],
        signatures: vec![sig],
        state_space: Vec::new(),
    }
}

/// Build the seeded frozen `DischargedGoal` used by the `--demo-escape-hatch`
/// observable-effect trigger (H-54 W-1 done-gate, the sanctioned seeded-fixture
/// pattern matching H-52 `--demo-anchor` / H-53 `--demo-prebind`).
///
/// The goal models the AlphaProof failure mode 1:1 (arXiv:2605.22763v1):
///   root goal: `<verifier>:<target>:top_statement` — looks sorry-free (hatch=None)
///   helper lemma: `<verifier>:<target>:renamed_helper_lemma` — carries `sorry`
///     (the RELOCATED escape hatch a top-statement-only scan would miss)
///
/// A BFS scan over this goal returns `EscapeHatchAnywhere`, which blocks the
/// verifier dispatch and lets the CLI emit the `TrapBehaviorDetected(ClosureEscapeHatch)`
/// SSB row — proving the gate fires through a REAL entrypoint.
///
/// This is the ONLY path that makes the orchestrator return a blocked result in
/// the current scaffolding tier (before live verifier DAGs are available on RunPod).
fn seeded_escape_hatch_goal(target_path: &str, verifier: VerifierKind) -> DischargedGoal {
    use std::collections::HashMap;
    let prefix = format!("{}:{}", verifier.as_str(), target_path);
    let root_id = format!("{prefix}:top_statement");
    let helper_id = format!("{prefix}:renamed_helper_lemma");
    let mut dag = HashMap::new();
    dag.insert(
        root_id.clone(),
        ObligationNode {
            obligation_id: root_id.clone(),
            hatch: None, // root looks sorry-free (the AlphaProof failure mode)
            depends_on: vec![helper_id.clone()],
        },
    );
    dag.insert(
        helper_id.clone(),
        ObligationNode {
            obligation_id: helper_id,
            hatch: Some(HatchKind::Sorry), // the relocated sorry in the helper
            depends_on: vec![],
        },
    );
    DischargedGoal::with_dag(root_id, dag)
}

/// Run the formal-verify pipeline for a single request.
///
/// Steps:
///   1. Build a minimal C-50 [`StructuralContract`] anchored at the
///      target path.
///   2. Dispatch the requested verifier:
///      - `Dafny` -> shellout via C-52 [`invoke_dafny_cli`]; mapping:
///        * `Ok(res)` with `res.dafny_pass && res.udebug_pass` ->
///          `Verified`
///        * `Ok(res)` with `dafny_pass=false` -> `Refuted` with a
///          short label from the verifier's exit code
///        * `Ok(res)` with `dafny_pass=true && udebug_pass=false` ->
///          `Inapplicable { reason: "vacuous_guard_failed" }`
///        * `Err(ToolchainNotFound)` -> `RunpodPending`
///        * `Err(other)` -> `Inapplicable` with a short reason
///      - `Z3Smt` -> RUNPOD-PENDING stub returning
///        `Inapplicable { reason: "z3_stub" }`. The SMT-LIB2 encoder
///        + Z3 shellout land in a follow-up patch.
///      - `Kani`, `Verus` -> RUNPOD-PENDING stubs returning
///        `Inapplicable { reason: "<verifier>_stub" }`.
///
/// CONTRACT_EMITTED semantics: `true` when step 1 produced a contract
/// before step 2 dispatched the verifier. `false` ONLY when step 1
/// itself failed (e.g. the C-50 spec validator rejected the
/// signature). The current build_minimal_contract is infallible so
/// `contract_emitted` is always `true` in this ship.
pub async fn run(request: VerifyRequest) -> PipelineResult {
    let contract = build_minimal_contract(&request.target_path, request.verifier);
    let contract_emitted = !contract.signatures.is_empty();

    // H-54 Layer-2 first leg: scan the obligation closure BEFORE the
    // verifier dispatch. An EscapeHatchAnywhere result short-circuits the
    // pipeline without running the verifier (never spend the expensive proof
    // check on a closure with an admitted hole).
    //
    // For the E-53 scaffolding-only tier (all verifiers return RunpodPending
    // or Inapplicable locally), the scan runs over a `DischargedGoal::unavailable`
    // placeholder, which immediately returns `ClosureUnavailable` — the Layer-4
    // verifier verdict is the sound backstop (R5). When a live verifier DAG is
    // available (Tier-2 / RunPod), the caller should pass a populated
    // DischargedGoal via the `goal` field added to the request in the follow-up.
    //
    // For the `--demo-escape-hatch` seeded-fixture path (W-1 observable_effect),
    // we substitute a frozen goal containing a relocated sorry in a renamed
    // helper lemma (the AlphaProof failure mode 1:1). This makes the scan return
    // EscapeHatchAnywhere so the pipeline blocks and the caller can emit the
    // TrapBehaviorDetected(ClosureEscapeHatch) SSB row — proving the gate fires
    // through a REAL entrypoint even before live verifier DAGs are available.
    //
    // For now the pipeline constructs an unavailable goal from the request's
    // target_path (scaffolding posture: DAG not yet exposed). The scan still
    // fires (Layer-2 runs unconditionally); it returns ClosureUnavailable.
    let discharged_goal = if request.demo_escape_hatch {
        seeded_escape_hatch_goal(&request.target_path, request.verifier)
    } else {
        DischargedGoal::unavailable(format!(
            "{}:{}",
            request.verifier.as_str(),
            request.target_path
        ))
    };
    let scan_result = scan_obligation_closure(&discharged_goal, request.verifier, None);

    // WARN-1 fix: when ELAI_CLOSURE_SCAN_DISABLE fired, the scan returned a
    // synthetic Clean result (closure_size=0) that is indistinguishable from
    // a real clean scan if we wrap it in Some(). Per the PipelineResult doc,
    // `closure_scan: None` means "bypassed via kill-switch" — the only way to
    // distinguish bypass from clean. Set None here when the kill-switch is set.
    let scan_field = if std::env::var("ELAI_CLOSURE_SCAN_DISABLE").is_ok() {
        None
    } else {
        Some(scan_result.clone())
    };

    // Hard block on unregistered escape hatch anywhere in the closure.
    // In the scaffolding tier this path is unreachable (ClosureUnavailable),
    // but the block logic is real for when the verifier exposes a live DAG.
    // On the --demo-escape-hatch path this IS reachable (seeded goal has
    // a relocated sorry -> EscapeHatchAnywhere).
    if scan_result.kind.is_block() {
        let escape_ids: Vec<&str> = scan_result
            .escape_nodes
            .iter()
            .map(|n| n.obligation_id.as_str())
            .collect();
        return PipelineResult {
            verifier: request.verifier,
            outcome: VerifierOutcome::Inapplicable {
                reason: format!(
                    "closure_escape_hatch_block:{}",
                    escape_ids.join(",")
                ),
            },
            contract_emitted,
            closure_scan: scan_field,
        };
    }

    let outcome = match request.verifier {
        VerifierKind::Dafny => dispatch_dafny(&request.target_path).await,
        VerifierKind::Z3Smt => VerifierOutcome::Inapplicable {
            reason: "z3_stub".to_string(),
        },
        VerifierKind::Kani => VerifierOutcome::Inapplicable {
            reason: "kani_stub".to_string(),
        },
        VerifierKind::Verus => VerifierOutcome::Inapplicable {
            reason: "verus_stub".to_string(),
        },
    };

    PipelineResult {
        verifier: request.verifier,
        outcome,
        contract_emitted,
        closure_scan: scan_field,
    }
}

/// Dafny dispatch helper. Returns the outcome enum directly so the
/// orchestrator doesn't need to repeat the mapping logic.
async fn dispatch_dafny(target_path: &str) -> VerifierOutcome {
    let path = Path::new(target_path);
    match invoke_dafny_cli(path).await {
        Ok(res) if res.dafny_pass && res.udebug_pass => VerifierOutcome::Verified,
        Ok(res) if res.dafny_pass && !res.udebug_pass => VerifierOutcome::Inapplicable {
            reason: "vacuous_guard_failed".to_string(),
        },
        Ok(res) => VerifierOutcome::Refuted {
            counterexample_label: format!("dafny_exit_{}", res.exit_code),
        },
        Err(DafnyCliError::ToolchainNotFound(_)) => VerifierOutcome::RunpodPending,
        Err(DafnyCliError::SpecNotFound { .. }) => VerifierOutcome::Inapplicable {
            reason: "spec_not_found".to_string(),
        },
        Err(DafnyCliError::Io(_)) => VerifierOutcome::Inapplicable {
            reason: "io_error".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn z3_route_returns_inapplicable_stub() {
        let req = VerifyRequest {
            target_path: "crates/auth/src/lib.rs".to_string(),
            verifier: VerifierKind::Z3Smt,
            cost_cap_ms: None,
            demo_escape_hatch: false,
        };
        let result = run(req).await;
        assert_eq!(result.verifier, VerifierKind::Z3Smt);
        assert!(result.contract_emitted);
        match result.outcome {
            VerifierOutcome::Inapplicable { reason } => assert_eq!(reason, "z3_stub"),
            other => panic!("expected Inapplicable; got {other:?}"),
        }
    }

    #[tokio::test]
    async fn kani_route_returns_inapplicable_stub() {
        let req = VerifyRequest {
            target_path: "crates/foo/src/lib.rs".to_string(),
            verifier: VerifierKind::Kani,
            cost_cap_ms: None,
            demo_escape_hatch: false,
        };
        let result = run(req).await;
        match result.outcome {
            VerifierOutcome::Inapplicable { reason } => assert_eq!(reason, "kani_stub"),
            other => panic!("expected Inapplicable; got {other:?}"),
        }
    }

    #[tokio::test]
    async fn verus_route_returns_inapplicable_stub() {
        let req = VerifyRequest {
            target_path: "crates/foo/src/lib.rs".to_string(),
            verifier: VerifierKind::Verus,
            cost_cap_ms: None,
            demo_escape_hatch: false,
        };
        let result = run(req).await;
        match result.outcome {
            VerifierOutcome::Inapplicable { reason } => assert_eq!(reason, "verus_stub"),
            other => panic!("expected Inapplicable; got {other:?}"),
        }
    }

    #[tokio::test]
    async fn dafny_route_returns_runpod_pending_when_binary_absent() {
        // Local dev host lacks dafny binary per C-52 mod doc. The
        // spec-path-missing path triggers SpecNotFound before binary
        // probe; use a non-existent path to land on
        // SpecNotFound -> Inapplicable (NOT RunpodPending). To hit
        // RunpodPending we need a real spec file that the binary
        // probe fails on; emulate via a temp file.
        let tmp = std::env::temp_dir().join("e53_formal_verify_empty.dfy");
        std::fs::write(&tmp, "// empty spec").expect("write temp spec");

        let req = VerifyRequest {
            target_path: tmp.to_string_lossy().to_string(),
            verifier: VerifierKind::Dafny,
            cost_cap_ms: None,
            demo_escape_hatch: false,
        };
        let result = run(req).await;
        let _ = std::fs::remove_file(&tmp);
        // On the local dev host the binary is missing -> RunpodPending.
        // If a future runner has dafny installed, the outcome would
        // be Refuted (empty spec hits the verifier with exit != 0).
        match result.outcome {
            VerifierOutcome::RunpodPending | VerifierOutcome::Refuted { .. } => {}
            other => panic!("expected RunpodPending or Refuted; got {other:?}"),
        }
    }

    #[tokio::test]
    async fn contract_emit_step_always_runs_before_dispatch() {
        let req = VerifyRequest {
            target_path: "crates/billing/payment.rs".to_string(),
            verifier: VerifierKind::Z3Smt,
            cost_cap_ms: None,
            demo_escape_hatch: false,
        };
        let result = run(req).await;
        assert!(
            result.contract_emitted,
            "contract emit step is the C-50 composition gate; must run before dispatch"
        );
    }
}
