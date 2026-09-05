# I-35: temporal tool-sequence enforcement — ordered history rules at the supervisor plane

- Track 18 — DETERMINISTIC-OVER-PROMPT HARDENING | lever CANT-WRECK | grounding STRONG | nature BUILD | effort M | WAVE 2 | depends I-1

- Premise: ELAI already enforces per-state tool allow/deny (FSM masks), per-call argument shape (E-30
  argv), and per-task tool-set commitment (F-3). The fourth dimension — ordering over history ("tests
  before push", "taint-check before write after untrusted ingest") — is absent (IN-12). This is the
  dimension that catches multi-step attack sequences; per-call gates cannot. CDA-immunity comes from
  action-level enforcement at the supervisor (token-level attacks cannot forge a tool-call history row).

## Steal/build
AgentSpec/Agent-C-shaped temporal rules evaluated at the I-1 supervisor dispatch plane:
  trigger: a tool class fires (e.g., class=git_push, class=file_write)
  predicate: a deterministic query over the ordered tool-call HISTORY of the current session
    examples: "at least one test_run row with verdict=PASS exists since the last src_edit",
              "no write-class call after an untrusted_ingest without an intervening taint_check PASS",
              "git_push requires a prior auditor_PASS row in the session"
  enforcement: halt (refuse the call + emit ToolSequenceVerdict(OrderViolation)) /
               require-confirmation (gate advisory) / substitute (redirect to a safer tool class)

Rules are expressed as a TOML rule set that is R8-frozen: the running system may NOT edit its own sequence
rules (a run that can rewrite its evaluator always writes itself to PASS — R8). The rule set hash is in
the gate-freeze manifest. Replay of a session re-derives identical verdicts from the same history (GD-2).

Deterministic: the predicate evaluates a concrete ordered list of SSB/journal rows, not a probabilistic
reachability estimate. CDA-immune by construction: the attacker controls token content, not the
tool-call history rows that are the predicate's input (action-level, not token-level).

Pro2Guard-style probabilistic DTMC reachability monitoring is DEFERRED (Tier-2): it needs a trace corpus
to estimate transition probabilities and cannot ship without one. The deterministic layer ships now.

## Existing ground truth
- IN-12 (grounding map): NO temporal tool-sequence rule layer — FSM masks + E-30 argv + F-3 commitment exist; ordering-over-history enforcement absent — confirms BUILD
- PLAN-asserted: Phase 3A FSM tool masks (per-state allow list); executor re-greps per W-7
- PLAN-asserted: E-30 argv allowlist (per-call argument shape); executor re-greps per W-7
- PLAN-asserted: F-3 committed tool sets (per-task scope commitment); executor re-greps per W-7
- IS-11: taint/provenance substrate (capability_flow.rs taint_text:234, safety/src/provenance.rs:131) — the predicate's richest input (GD-1)
- IS-14: production_agent_rpc.rs build_tool_catalog:657 — the tool-array surface I-35 gates
- I-1 dependency: supervisor plane must exist before I-35 wires into it (depends I-1 per WAVE 2 ordering)

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/tool_sequence.rs: SequenceRule (trigger/predicate/enforcement triad), SequenceRuleSet (TOML-deserialized, R8-frozen), evaluate_sequence_rule() called at supervisor dispatch
- TOML rule file for the safety core (git_push/file_write/untrusted_ingest classes), R8-frozen
- ToolSequenceVerdict SSB rows (Allowed / OrderViolation / HaltEnforced)
- Wire into I-1 supervisor dispatch: post-gate, pre-execution check
- Fixture: compliant sequence (-> Allowed), git-push with no auditor-PASS row (-> OrderViolation+halt), file-write after untrusted-ingest without taint-check (-> OrderViolation+halt)
- Sequence-verdict rows in gate panel (frontend_pair)
- Rule hash in gate-freeze manifest (gate_freeze_check.py covers it — R8)
- decision doc: workflow/reasoning/I-35-decisions.md

DEFER (Tier-2 / gated):
- Pro2Guard DTMC probabilistic reachability monitoring (arXiv:2508.00500 — needs a trace corpus; named explicitly as Tier-2 DEFERRED)
- Cross-session sequence rules (rules that span across session boundaries — session-scoped first)

## Integration
- new crates/orchestrator/src/tool_sequence.rs (~180 LOC): SequenceRule, SequenceRuleSet, evaluate_sequence_rule()
- MODIFY I-1 supervisor dispatch: call evaluate_sequence_rule() after existing per-call gates, before execution
- new: rules/tool_sequence_core.toml (the R8-frozen safety core rule file)
- seam to IS-11 taint: predicates query taint rows from capability_flow.rs (GD-1 compose)
- gate-freeze manifest: add tool_sequence_core.toml hash
- frontend_pair: sequence-verdict rows in the existing gate panel (SettingsWindow / MonitorWindow); Svelte chip + Tauri read; cross_lang_diff.py PASS (W-4)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/tool_sequence.rs::SequenceRule (trigger/predicate/enforcement record)
  - crates/orchestrator/src/tool_sequence.rs::SequenceRuleSet (the frozen rule registry)
  - crates/orchestrator/src/tool_sequence.rs::evaluate_sequence_rule() (the supervisor gate call)
  - rules/tool_sequence_core.toml (the R8-frozen safety-core rule set)
  - ssb action_type::ToolSequenceVerdict (Kind: Allowed / OrderViolation / HaltEnforced) — baseline+1 from LIVE 221/532 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
consumes_symbols:
  - I-1 supervisor dispatch (the evaluation hook point; depends I-1 — WAVE 2; GD-1)
  - Phase 3A FSM tool masks (per-state allow — I-35 adds ORDER dimension; GD-1)
  - E-30 argv allowlist (per-call shape — I-35 is the history dimension; GD-1)
  - F-3 committed tool sets (per-task scope; GD-1)
  - IS-11 capability_flow.rs taint_text:234 (taint rows as predicate input; GD-1)
reachable_from:
  - I-1 supervisor::dispatch_tool -> tool_sequence::evaluate_sequence_rule -> ToolSequenceVerdict SSB row  [PRODUCTION supervisor surface]
  - test callers in crates/contracts/tests/ do NOT satisfy R7
stub_ban:
  - NO rule set that can be edited by the running session (R8: the rule hash is in the frozen manifest; a session that modifies tool_sequence_core.toml FAILS gate_freeze_check.py)
  - NO probabilistic reachability in the Tier-1 path (Pro2Guard DTMC is Tier-2 DEFERRED — it needs a trace corpus)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  sequence-verdict rows in the gate panel (MonitorWindow) — Svelte chip + Tauri read; cross_lang_diff.py PASS required (W-4).
observable_effect:
  trigger: "fixture agent session attempting: (a) compliant order, (b) git-push with no auditor-PASS row, (c) file-write after untrusted-ingest without taint-check — run via `elai run --sequence-fixture --seed 42`"
  expect_ssb: "ToolSequenceVerdict rows: Allowed for (a), OrderViolation+HaltEnforced for (b) and (c), naming the unsatisfied predicate in the payload — recomputable (GD-2); replay of the session re-derives identical verdicts"
  expect_artifact: "the rules file hash is in the frozen manifest (gate_freeze_check.py passes); a run that edits tool_sequence_core.toml is BLOCKED (R8 — gate_freeze_check.py exits nonzero); sequence-verdict rows render in the gate panel (cross_lang_diff.py PASS)"

## Grounding
- IN-12 (grounding map): temporal sequence enforcement absent — confirms BUILD not ACTIVATION
- NL-vs-deterministic dossier: AgentSpec/Agent-C trigger/predicate/enforcement pattern
- arXiv: AgentSpec 2503.18666, Agent-C 2512.23738 — WebSearch-confirm before headline
- arXiv: Pro2Guard 2508.00500 (DTMC Tier-2 DEFERRED; named explicitly)
- IS-11 taint substrate (grounding map): the predicate's richest data source
- I-1 dependency (I-0 WAVE 2 ordering): supervisor plane is a hard prerequisite
- R8 (wiring-enforcement.md): the freeze rule that the rule-set design must satisfy
- SSB: +ToolSequenceVerdict (Kind: Allowed/OrderViolation/HaltEnforced) — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9)

## COMPOSES-WITH
- I-1 supervisor dispatch — the evaluation hook; do NOT re-implement the supervisor logic
- Phase 3A FSM tool masks — the per-state dimension; I-35 adds ORDER, not per-state control
- E-30 argv allowlist — the per-call shape dimension; I-35 is orthogonal (history, not shape)
- F-3 committed tool sets — the per-task scope dimension; I-35 adds ORDER as the fourth dimension
- IS-11 capability_flow.rs — the taint data source; compose, do NOT re-implement taint tracking

End of I-35.
