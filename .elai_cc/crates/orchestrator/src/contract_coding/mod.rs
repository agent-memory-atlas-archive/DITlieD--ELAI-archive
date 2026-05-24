//! C-50 — Contract-Coding STRUCTURAL contract test scaffolding.
//!
//! Paper grounding: arXiv:2604.13100 (Lin, Zhao, Shi 2026 —
//! "Contract-Coding: Towards Repo-Level Generation via Structured
//! Symbolic Paradigm"). Verified via arXiv abs page on 2026-05-24.
//! Interface map: `workflow/papers/contract-coding-interface.md`.
//!
//! GROUNDING CAVEATS (mandatory — DO NOT misframe):
//!   - The paper's "contract" is a STRUCTURAL artifact, kernel
//!     `K = <N, Sigma, Delta>` where `N` is the module namespace,
//!     `Sigma` is the type-signature set, and `Delta` is the state
//!     space (Section 4.2). The paper uses NO Hoare triples, NO
//!     temporal-logic assertions, and NO Eiffel/JML-style behavioral
//!     pre/postconditions. Any C-50 surface that frames itself as
//!     "pre/postcondition emitter" is miscited; this module ships
//!     STRUCTURAL contracts only.
//!   - The paper's headline "47%" is the Roguelike subset of the
//!     Greenfield-5 benchmark (Table 1, Section 5.2). Contract-Coding
//!     TIES Trae at 47%; Gemini Studio BEATS Contract-Coding at 63%;
//!     OpenHands trails at 30%. The differentiator the paper argues
//!     is qualitative — failures are "strictly confined to Local
//!     Logic Errors, not architectural collapse" (Section 5.2). Do
//!     NOT cite "47% Greenfield-5 improvement" without the Roguelike
//!     subset + structural-integrity qualifier.
//!   - Mutation primitives are deliberately {Add, Update} only. The
//!     paper excludes Delete to avoid kernel inconsistency (Section
//!     4.3); this module enforces the same {Add, Update} closure on
//!     [`ContractMutation`].
//!   - The full Generator -> Discriminator RL training pipeline
//!     (paper Algorithm 1) is RUNPOD-PENDING. This module ships the
//!     STRUCTURAL contract surface + heuristic discriminator + test-
//!     file scaffolder only.
//!
//! Module layout:
//!   - [`spec`]          — `StructuralContract { N, Sigma, Delta }`,
//!                         `TypeSignature`, `StateConstraint`.
//!   - [`emitter`]       — `emit_contract_test` produces a Rust
//!                         test-file template scaffolding contract
//!                         assertions from a `StructuralContract`.
//!   - [`discriminator`] — `audit_against_contract` returns an
//!                         `AuditResult` (heuristic; the RL trained
//!                         Discriminator Agent is RUNPOD-PENDING).
//!   - [`mutation`]      — `ContractMutation { Add, Update }`,
//!                         paper-grounded {Add, Update} closure.
//!   - [`scaffolder`]    — `scaffold_contract_test_from_signature`
//!                         entry point for the test-file scaffolder.

pub mod discriminator;
pub mod emitter;
pub mod mutation;
pub mod scaffolder;
pub mod spec;

pub use discriminator::{audit_against_contract, AuditResult, ContractViolation, ViolationKind};
pub use emitter::{emit_contract_test, EmitError};
pub use mutation::ContractMutation;
pub use scaffolder::{scaffold_contract_test_from_signature, ScaffolderError};
pub use spec::{StateConstraint, StructuralContract, TypeSignature};
