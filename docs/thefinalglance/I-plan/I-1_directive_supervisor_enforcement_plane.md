# I-1: directive supervisor enforcement plane — move side-effect execution out of the generator's reach

- Track 12 — FAULT-ISOLATION + ENFORCEMENT-PLANE | lever CANT-WRECK | grounding STRONG | nature BUILD | effort L | WAVE 2 | binds ID-2
- Premise: ELAI's PreToolUse hooks run in-process with the generator they police; the 2026 incident record
  shows an agent disabling its own sandbox is structurally possible when enforcement shares an address space
  (infrastructure dossier §4.1). There is no Directive type and no process-separated supervisor (IN: hooks
  in-process; IS-19 doctrine assumes the boundary without enforcing it structurally). The generator must
  produce pure Intent->Directive structs only; a process-separated Supervisor must be the sole executor of
  side effects, gate-checking before every action and holding all credentials and egress paths.

## Steal/build
Erlang/Jido actor discipline: the generator is a pure function over intent — it emits Directive structs and
cannot call any sink directly. The Supervisor is a separate process (OS process boundary, not just a thread);
it receives Directives, runs every gate (F-1 VibeGate, E-30 argv allowlist, G-3 blast-radius) as a
pre-execution check, and executes the side effect only on gate PASS. Saga-style compensation (E-44) undoes
partial state on any crash. The invariant "a generator-authored fn cannot satisfy a gate or reach the egress
path" is verifiable and gets a contract test. Declined: a pure async-task split (insufficient — shared heap,
same fault domain); a capability-passing scheme without OS isolation (too easy to leak the capability handle
back to the generator).

## Existing ground truth
- IS-19: wiring tooling live (wiring_contract_check.py, gate_freeze_check.py); existing hooks run in-process —
  the gap this session closes.
- IS-11: capability_flow.rs taint_text:234; crates/safety/src/provenance.rs:131 — safety crate exists as
  composition seam.
- Phase 4A IPC process seam EXISTS — reuse as the supervisor/generator channel; do not re-invent.
- F-1 VibeGate, E-30 argv allowlist, G-3 blast-radius: SHIPPED gate primitives; become supervisor-plane gates
  (PLAN-asserted; executor re-greps per W-7).
- E-44 saga compensation: SHIPPED (PLAN-asserted; executor re-greps per W-7).
- No Directive type, no supervisor boundary: IN (hooks run in-process confirmed by IS-19 doctrine framing).
- task_runner dispatch + hook_dispatcher seams: composition points (PLAN-asserted; executor re-greps per W-7).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- Directive struct (pure value type: tool_name, args, intent_id, initiator) in crates/orchestrator/src/directive.rs
- Supervisor process abstraction + in-process stub sufficient for test (OS-separated process = Tier-2 gate)
- Gate pre-check chain in Supervisor (F-1, E-30, G-3 called before execution)
- Saga compensation hook on Supervisor crash (calls E-44 rollback)
- Contract test asserting generator cannot call any sink directly (the invariant)
- DirectiveExecuted SSB variant (CompensationFired arm)

DEFER (Tier-2 / gated):
- Full OS process separation (separate pid, IPC channel): separate gate, network/process-launch gated
- Credential vault in the Supervisor (key management surface): separate session
- Cloud sandbox adapter using the same trait: optional, never a dep (IC-9 posture)

## Integration
- CREATE: crates/orchestrator/src/directive.rs (~80 LOC: Directive type + Supervisor trait + in-process impl)
- MODIFY: crates/orchestrator/src/task_runner.rs — dispatch path composes Supervisor (PLAN-asserted seam;
  executor re-greps per W-7)
- MODIFY: crates/orchestrator/src/hook_dispatcher.rs — delegate gate checks to Supervisor pre-check (PLAN-asserted;
  executor re-greps per W-7)
- CREATE: crates/contracts/tests/i1_directive_supervisor_contract.rs (~60 LOC)
- CREATE: workflow/reasoning/I-1-decisions.md
- frontend_pair: supervisor verdict rows rendered in an existing drawer (PLAN-asserted; executor confirms
  which drawer at EXECUTING — supervisor DirectiveExecuted rows visible without a new panel).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/directive.rs::Directive (the pure value type)
  - crates/orchestrator/src/directive.rs::Supervisor (trait + in-process impl)
  - DirectiveExecuted (new SSB action_type variant; CompensationFired arm) — baseline+1 from LIVE 221/532
    as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - crates/contracts/tests/i1_directive_supervisor_contract.rs
consumes_symbols:
  - Phase 4A IPC seam — reused as the generator/supervisor channel; do NOT re-invent the process seam
  - F-1 VibeGate (why: becomes a supervisor pre-check gate; compose, do not re-implement gate logic)
  - E-30 argv allowlist (why: same — supervisor pre-check)
  - G-3 blast-radius (why: same)
  - E-44 saga compensation (why: called by Supervisor on crash for state restore)
  - crates/safety/src/provenance.rs::Provenance (IS-11: safety crate composition seam)
reachable_from:
  - crates/orchestrator/src/task_runner.rs (tool dispatch path) -> Supervisor::execute_directive  [PRODUCTION task dispatch]
  - hook_dispatcher.rs gate pre-check path -> Supervisor::run_gates  [PRODUCTION hook chain]
  (a #[cfg(test)] / contracts/ caller NEVER satisfies this — R7)
stub_ban:
  - NO direct sink calls from any generator code path (the invariant this session enforces)
  - NO .unwrap()/.expect() on production paths
  - NO blake2/sha2 (blake3 only per CLAUDE.md)
  - NO fabricated measurements or invented file:line numbers
  - NO OS process separation claimed as shipped in Tier-1 (the in-process stub is Tier-1; OS pid split is Tier-2)
  - NO re-implementation of F-1/E-30/G-3 gate logic (compose the shipped primitives)
frontend_pair:
  DirectiveExecuted rows rendered in an existing SSB-backed drawer (no new dashboard component required at
  Tier-1; full supervisor verdict panel is Tier-2). Justified: the SSB row is the observable artifact;
  the drawer consuming it is composing IS-1 append-only infrastructure.
observable_effect:
  trigger: "elai run --task <fixture-task> -- <injected OOM/panic mid-tool>"
  expect_ssb: "DirectiveExecuted(CompensationFired) row in ssb.db within the run; DirectiveExecuted(Succeeded)
    row on a clean run; a generator-authored direct-sink call attempt produces DirectiveExecuted(Refused) with
    the full directive logged"
  expect_artifact: "pre-tool state byte-identical after crash (journal-verified from checkpoint); the negative
    arm — a generator fn calling a sink directly bypassing the Supervisor — is structurally impossible (the
    contract test asserts no public sink-reaching path exists from any generator module); missing SSB row = FAIL (R1)"

## Grounding
- IS-19: wiring tooling + in-process hooks gap confirmed live. IS-11: safety crate composition seam.
  IN (no Directive type, no supervisor boundary) confirmed by IS-19 framing.
- Infrastructure dossier §4.1 (the incident record + in-process enforcement structural weakness).
  Trust-review A3 (the enforcement-plane move rationale). Consolidated draft Part I §supervisor section.
  All arXiv IDs carried from these sources: WebSearch-confirm before headline (map citation caveat).
- SSB: DirectiveExecuted +1 variant — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 /
  action_type_lock.rs:601); re-read at edit (W-9).

FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule).

## COMPOSES-WITH
- Phase 4A IPC: the process seam — reuse as supervisor/generator channel. Do NOT re-invent.
- F-1 VibeGate, E-30 argv allowlist, G-3 blast-radius: SHIPPED gates moved to supervisor pre-check plane.
  Do NOT re-implement gate logic.
- E-44 saga compensation: SHIPPED rollback — compose on crash. Do NOT re-implement compensation.
- IS-11 safety crate (capability_flow.rs, provenance.rs): composition seam. Do NOT re-implement taint logic.
- I-2 (CoW workspace isolation): the workspace isolation boundary this session's Supervisor will enforce.
- I-35 (temporal tool-sequence rules) and I-31 (GRPO reward endpoint) live on the Supervisor side;
  I-1 is their precondition.

End of I-1.
