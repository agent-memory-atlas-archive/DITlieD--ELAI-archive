//! E-53 — formal verification codegen pipeline (lean ship).
//!
//! Composes C-52 [`crate::dafny_verify`] (Dafny verifier shellout)
//! and C-50 [`crate::contract_coding`] (structural contracts) into
//! an end-to-end pipeline routed by the [`crate::csq`] classifier:
//! a path classified `CounterfactualRisky` dispatches a formal
//! verifier (Dafny / Z3 / Kani / Verus); other classifications
//! skip dispatch.
//!
//! PAPER GROUNDING (3 inline GROUNDED citations, per cluster cite-
//! discipline):
//!   - arXiv:2604.22601 NL2VC (Dafny tier framing) — re-used via C-52
//!     [`crate::dafny_verify`] dispatch only; no new claim added in
//!     this module.
//!   - arXiv:2501.03124 PRMBench (Deception Resistance + Circular
//!     Logic as PRM-failure modes) — frames why heterogeneous
//!     consensus across `VerifierKind` source families is the
//!     structural defense in `verification_result.rs:7-12`.
//!   - AlphaProof Nature 2025 (Lean theorem-prover ~500 TPU-days /
//!     problem, prohibitive cost) — grounds the Lean HARD-AVOID
//!     posture; no Lean dispatch path lives in this pipeline.
//!
//! Other arXiv IDs the E-53 plan cited (DafnyPro variance 2604.05292
//! vs 2601.05385, AutoVerus, AlphaVerus, Vericoding, Re:Form, Dafny
//! POPL'25, Seed-Prover, DeepSeek-Prover-V2, Lean Copilot,
//! Theorem-YC-S25 marketing) are NOT inline-cited; deferred to
//! workflow/sessions/s-25120/completion-drive/mistakes.md cite-
//! incident #11 (E-53-INGEST-pending) per cluster cite-discipline.
//!
//! GEMMA-4 BLOCKED rule (C-52 substrate): the pipeline NEVER routes
//! to Gemma-4 as a model backend. Substitutes documented in
//! [`crate::dafny_verify::ELAI_SUBSTITUTE_MODELS`] are
//! Codestral-22B-v0.1 / GPT-OSS-120B / Qwen 3 Coder 30B.
//!
//! NO new external dep is added in this ship. All four toolchains
//! (Z3, Dafny, Kani, Verus) are detected at runtime via shellout +
//! return `VerifierOutcome::RunpodPending` when the binary is
//! absent.
//!
//! Module layout:
//!   - [`types`]        — [`VerifierKind`] (toolchain enum, distinct
//!                        from `verification_result::VerifierKind`
//!                        which is the consensus-source family),
//!                        [`VerifierOutcome`] (4 variants),
//!                        [`VerifyRequest`].
//!   - [`orchestrator`] — [`run`] entry; composes C-50 contract
//!                        emission + verifier dispatch + outcome
//!                        mapping.

pub mod closure_scan;
pub mod orchestrator;
pub mod types;

pub use closure_scan::{scan_obligation_closure, ClosureScanKind, ClosureScanResult, EscapeNode};
pub use orchestrator::{run, PipelineResult};
pub use types::{
    DischargedGoal, HatchKind, ObligationNode, VerifierKind, VerifierOutcome, VerifyRequest,
    ALL_VERIFIER_KINDS,
};
