C-42 Tranche B-6 -- proc-macro deferral + plain-fn shippable-now decisions

Date: 2026-05-31
Session: s-22300
Parent: workflow/reasoning/B6-quad-tranche-decisions.md

DECISION: ship plain-fn runtime guard NOW; defer proc-macro to C-42b.

CONTEXT
=======

C-42 plan demands a declarative `#[authz(states="...", actors="...")]`
attribute that registers tool handlers + emits per-call SSB rows. The
exploration confirmed that the workspace has ZERO `proc-macro = true`
crates today (grep "proc-macro = true" elai.map = 0 hits). Adding the
declarative attribute would require:
  - NEW `crates/plugins_macros/{Cargo.toml, src/lib.rs}` with
    `[lib] proc-macro = true`
  - NEW workspace member registration in `.elai_cc/Cargo.toml`
  - NEW deps: proc-macro2 + syn + quote (and likely darling for
    attribute-arg parsing)
  - Cargo.lock churn affecting every downstream crate

Per CLAUDE.md HARD RULE "no new dependencies without confirmation",
introducing 3-4 new proc-macro infra deps + the first proc-macro crate in
the workspace is a non-trivial gate.

THE SAME OUTCOME IS ACHIEVABLE WITHOUT THE ATTRIBUTE
=====================================================

The exploration documented the key insight:
  > The runtime emit_mcp_authz_check guard at compose.rs:84 wrapper
  > achieves the SAME outcome (per-call SSB emission gated by policy)
  > without any of the proc-macro infra. Plan-spec demands the attribute
  > for declarative compile-time tool-registration ergonomics -- that's a
  > developer-UX win, NOT a security/correctness win. The runtime checker
  > is already pure + deterministic (checker.rs:1-9); adding a proc-macro
  > layer multiplies the surface without changing the threat model.

The runtime guard:
  - emits one `McpAuthzEvent` SSB row per composed verdict (Allow /
    DeniedByAuthz / DeniedByToolGate) via the new
    `compose_with_tool_gate_emit` wrapper
  - threads the SSB emitter through an `Option<&dyn McpAuthzSsbEmitter>`
    parameter (zero-overhead when None, matches existing
    `CaseBaseSsbEmitter` / `ToolGateSsbEmitter` patterns)
  - keeps `check_authz` purity intact (defense-in-depth boundary per
    checker.rs:1-9 doc-comment)
  - closes the MASTER-INDEX line 194 SHIPPED-WIRED gap (per-call SSB
    rows missing)

DEFER CRITERIA FOR C-42b
========================

When a second proc-macro need surfaces in the workspace (e.g. C-XX
wants a similar attribute on memory-trust handlers, or B-YY wants a
declarative SSB-emit attribute), the cost of creating `plugins_macros`
amortizes across both. At that point:

  - create `crates/plugins_macros/` with proc-macro2 + syn + quote
  - implement `#[authz(states="...", actors="...")]` that emits a
    runtime-registration call PLUS the SSB-emit wrapper at expansion time
  - migrate `compose_with_tool_gate_emit` callers to use the attribute
    on the handler functions themselves
  - the SSB substrate (McpAuthzEvent) stays unchanged -- no further
    SCHEMA bump needed

PRE-CONDITIONS FOR C-42b
========================

Documented dependencies for the deferred work:
  - proc-macro2 (verify via `cargo info proc-macro2` for current version)
  - syn 2.x (workspace-wide pin needed)
  - quote 1.x
  - darling 0.20+ for attribute-arg parsing (if attribute carries
    `states="..."` / `actors="..."` lists)

SHIPPABLE-NOW DELIVERABLES
==========================

This tranche delivers:
  - `crates/plugins/src/authz/compose.rs::McpAuthzSsbEmitter` (NEW trait)
  - `crates/plugins/src/authz/compose.rs::compose_with_tool_gate_emit`
    (NEW emit-aware wrapper)
  - `crates/orchestrator/src/ssb.rs::SsbActionType::McpAuthzEvent` +
    3-arm `McpAuthzEventKind` (Pattern-2 46th application)
  - `crates/orchestrator/src/ssb.rs::ELAI_C42_MCP_AUTHZ_EMIT_ENABLE_ENV`
    (wire-dark default OFF)
  - `crates/orchestrator/src/ssb.rs::MCP_FUZZ_CORPUS_DIR` constant
  - `.elai_cc/data/mcp_fuzz_corpus/.gitkeep` (corpus dir sentinel)
  - `crates/contracts/tests/b6_quad_ssb_contract.rs` round-trip coverage

DEFERRED TO C-42b
==================

  - `#[authz(...)]` declarative attribute (proc-macro)
  - `crates/plugins_macros/` (first proc-macro crate)
  - 200+ tuple deterministic corpus generator (build.rs OR corpusgen bin)
  - Populated `.elai_cc/data/mcp_fuzz_corpus/tuple_*.json` files
  - Caller migration from `compose_with_tool_gate_emit` to the attribute

PARENT TRANCHE
==============

See `workflow/reasoning/B6-quad-tranche-decisions.md` for the
coordinated SCHEMA_VERSION bump + Pattern-2 collapse decisions across
C-42 / C-47 / B-39 / C-50.
