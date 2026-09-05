# J-52: return-time + per-edit enforcement hooks — SubagentStop evidence-receipt gate, per-edit LSP daemon block-feedback, path-keyed dynamic rule injection (three legs extending the WIRED hook system)

STATUS: PLANNED (WAVE LAZYCODEX-REINTAKE-1; Tier-1 all three legs now; nature EXTENSION over the WIRED hook_dispatcher + SubagentToolExecutor + diagnostic_probe + context_budget Rules slot)

- Track J (lazycodex v4.11.1 re-intake) | lever PROVABLE (can't-lie / structural enforcement) + SMARTER-ORCHESTRATION | grounding STRONG (lazycodex/OmO v4.11.1 real-source read = docs/.source/lazycodex-main-INDEX.md; ELAI substrate grounded via elai.map 2026-06-19, executor re-greps per W-7) | nature EXTENSION | effort M-L | WAVE LAZYCODEX-REINTAKE-1 | parallel-safe with J-50/J-51 (disjoint files) | composes wiring-enforcement R6/R7 + agent-dispatch-preamble R-8 + production-caller-gate D-030 + E-52 + B-10/B-37
- Premise: a 2026-06-19 RE-INTAKE of lazycodex/OmO. The repo was mined once before (STEAL-LIST.md 2026-06-10) but at v0.2.2 — a thin skill-pack read from a SUMMARY at MEDIUM confidence — which captured only F1 (Stop-hook continuation, DEFERRED) and F2 (skills = E-43). This checkout is v4.11.1, ~20x bigger, read from real src/*.ts at HIGH confidence. The deep ENFORCEMENT mechanisms below were not present / not seen in the v0.2.2 pass. Three of them are genuine net-new EXTENSIONS of ELAI's WIRED hook system, which today has PreToolUse/PostToolUse/Stop/PreCompact but NO SubagentStop axis and where "agents never see hooks" (FEATURE-MAP HOOK SYSTEM, hook_dispatcher.rs). All three turn an existing PROMPT-level discipline into a MECHANICAL, fail-closed, off-agent gate — the exact "the fix has to be STRUCTURAL, an LLM cannot be instructed out of its shortest-path bias" thesis of production-caller-gate.md.

## Steal/build (3 legs)

LEG 1 — SubagentStop EVIDENCE-RECEIPT GATE [S1, the strongest]. Source: components/lazycodex-executor-verify/src/codex-hook.ts (runSubagentStopHook). A SubagentStop hook BLOCKS a sub-executor from finishing unless its last line is exactly `EVIDENCE_RECORDED: <path>` AND that path resolves to a NON-EMPTY REAL file (realpath-canonical, symlink-rejected, traversal-guarded both pre- and post-realpath) inside the evidence dir; else increment a persisted atomic attempt counter (MAX 3) and return {decision:"block", reason:<nag directive>}; capped nag, then give up. The proof is a FILESYSTEM ARTIFACT the gate reads, never the agent's stdout. ELAI port: add a `subagent_stop` enforcement axis at the sub-agent RETURN seam (crates/tools/src/lib.rs::SubagentToolExecutor, where the worker's "### Final response" is assembled, ~lib.rs:4025/4214) that fail-closed blocks acceptance of an executor worker's DONE return unless it carries an evidence receipt pointing at a real non-empty off-agent artifact (path-hardened identically); on miss, a bounded-nag re-dispatch (persisted counter, MAX_EVIDENCE_NAGS=3), then ESCALATE to the auditor / a handoff entry (R-4), never crash. This makes agent-dispatch-preamble R-8 / D-030 a MECHANICAL return-time tripwire instead of a prompt plea — catching wire-dark/no-evidence AT the executor return, one full auditor cycle EARLIER.

LEG 2 — PER-EDIT LSP DAEMON BLOCK-FEEDBACK [S2]. Source: components/lsp/src/codex-hook.ts (runLspPostToolUseHook). PostToolUse on edit-like tools runs REAL diagnostics through a long-lived REUSED daemon (MAX_CONCURRENT_DIAGNOSTICS=4) and on any real diagnostic returns {decision:"block", reason:<formatted diagnostics>} inline; the subtle correctness is the transient-vs-permanent distinction (a daemon-UNREACHABLE result is transient and NOT recorded — else it silences the lane all session; an "no LSP server for this extension" result IS recorded so it stops retrying), plus output bounded (8000 normal / 1200 under context pressure). ELAI port: extend E-52 diagnostic_probe.rs from its pre-AUDITING CoreValidationTier BATCH probe (called at task_runner.rs:14728) into a PER-EDIT PostToolUse block-feed riding the existing ruff PostToolUse seam (hook_dispatcher.rs::dispatch_post_tool_use:1971 + crates/plugins/src/post_tool_use.rs), backed by a reused daemon (launch rust-analyzer/tsc/svelte-check once, reuse across edits — diagnostic_probe.rs already maps language->tool at :124/137), with the same transient(not-recorded)/permanent(recorded) discipline and bounded output. The format mirror of comment-checker (post-edit lint block-feed) is the SAME shape ELAI already ships for ruff — leg 2 adds the LSP toolchain to it.

LEG 3 — PATH-KEYED DYNAMIC RULE INJECTION [S3]. Source: components/rules (PostToolUse dynamic-target-fingerprints.ts). Editing src/auth/* mechanically pulls in src/auth's LOCAL rules: fingerprint the edited path, inject only rules whose target fingerprint changed and that are not already in the transcript (transcript-dedup), priority-sourced. ELAI port: make conditional-routing.md MECHANICAL — a PostToolUse path-fingerprint injector (same dispatch_post_tool_use:1971 seam) that resolves the matching local rule for the edited path and injects it into the CONTEXT BUDGET ALLOCATOR Rules slot (context_budget.rs SlotWeights), transcript-deduped (never re-inject a rule already resident — the Rules slot is re-filled each turn, so dedup against current slot contents), priority-ordered sources (.claude/rules first). SubagentToolExecutor.active_rules (lib.rs:4025 vicinity) is the rule-carrying field the injection targets for a worker. Removes reliance on the main LLM remembering to load the right rule when the edited path triggers conditional-routing.

HARD boundaries (negative space, all three legs): the AUDITOR stays the off-agent authority — these are cheap EARLY tripwires, NOT replacements for production_caller_check.py / python_wire_check.py / the wiring gates (leg 1 must not let a worker's self-emitted receipt count as the audit verdict — the receipt only proves an artifact EXISTS, the auditor still re-runs the real checkers off-agent, R5). NO hard-kill: leg 1 caps at MAX_EVIDENCE_NAGS then ESCALATES (handoff R-4), it never crashes the run. NO second LSP toolchain or a bypass of E-52's existing language->tool map (leg 2 reuses it; a daemon-unreachable result is NEVER recorded). NO re-injecting a resident rule (leg 3 dedups against the live Rules slot; double-injection is a token-waste bug). NO new ELAI_*_ENABLE — every leg is default-ON behind a kill-switch (W-2/R-f). The receipt path-hardening MUST reject symlinks + traversal exactly as the source does (a receipt pointing outside the evidence dir or through a symlink is a BLOCK, not a pass).

## Existing ground truth (grounded via elai.map 2026-06-19; PLAN-asserted line numbers, executor re-greps per W-7)

- HOOK SYSTEM (T1/T2/T3) WIRED — crates/orchestrator/src/hook_dispatcher.rs intercepts IPC at PreToolUse/PostToolUse/Stop/PreCompact; HookInvocation::for_task_level(action_type, step_id) (hook_dispatcher.rs:352) is the dispatch shape; dispatch_post_tool_use (hook_dispatcher.rs:1971); with_pre_compact_token_state (:435). NO `subagent_stop` action_type exists today (grep-confirmed) — leg 1 is the net-new axis.
- SubagentToolExecutor WIRED — crates/tools/src/lib.rs:4214 (impl ToolExecutor::execute at :4025 assembles the worker "### Final response"; carries active_rules + an enforcement field; allowed_tools_for_subagent :3758, allowed_paths_for_subagent :3854). This is leg 1's return seam AND leg 3's active_rules target; do NOT re-implement the sub-agent executor.
- E-52 LSP-AS-VERIFIER PARTIAL — crates/orchestrator/src/diagnostic_probe.rs (DiagnosticProbeRequest; tool_for_language rust->.. / typescript->tsc / svelte->svelte-check at :124/137; rank_worst :379; notify_diagnostic_probe called from task_runner.rs:14728 at CoreValidationTier — the BATCH pre-AUDITING leg). ELAI_LSP_DIAGNOSTIC_PROBE_ENABLE unallowlisted, real subprocess deferred (FEATURE-MAP). Leg 2 extends this to per-edit + daemon; do NOT re-implement the language->tool map.
- existing ruff PostToolUse WIRED — crates/plugins/src/post_tool_use.rs (RuffOutcome cache, has_warned_once :185, RUFF timeout 5s :483) + hook_dispatcher.rs::run_ruff_post_tool_use:890. The PROVEN PostToolUse block-feed shape legs 2+3 ride; do NOT re-implement the PostToolUse dispatch.
- CONTEXT BUDGET ALLOCATOR WIRED — crates/orchestrator/src/context_budget.rs (8 fixed slots incl Rules; SlotWeights; update_breakpoints :1535; detect_model_capability_profile :705). Leg 3 feeds the Rules slot; do NOT re-define SlotWeights.
- VIBEGATE WIRED — crates/orchestrator/src/vibegate.rs:68 (evaluate_query; DefaultRuleEngine :112) — the PreToolUse write-gate precedent leg 1's fail-closed posture mirrors; do NOT route leg 1 through VibeGate (different lifecycle event).
- conditional-routing.md (the INSTRUCTION-level rule-loading doctrine leg 3 makes mechanical); agent-dispatch-preamble.md R-8 + production-caller-gate.md D-030 (the prompt-plea version of leg 1's gate).

## REVISED scope (W-10 tiered)

Tier-1 (SHIP NOW): all three legs at runtime in-process. Leg 1 evidence gate on the executor subagent return + bounded nag + escalate; leg 2 per-edit LSP daemon block-feed reusing E-52's toolchain; leg 3 path-keyed rule injection into the Rules slot. Each is in-process at gate time (W-10), no network.
Tier-2 (DEFER): a persistent cross-session LSP daemon pool (Tier-1 reuses within a session only); receipt-content schema validation beyond existence+non-empty (Tier-1 checks the artifact is a real non-empty file, not its internal shape — that is the auditor's job); a learned per-path rule-relevance score (Tier-1 uses the static path->rule fingerprint map). All recorded as riders.

## Integration

- NEW: crates/orchestrator/src/subagent_evidence_gate.rs (~140 LOC: EvidenceReceipt parse from the worker return, path-hardening (canonicalize + symlink-reject + traversal-guard pre/post realpath), persisted atomic nag counter MAX_EVIDENCE_NAGS, the block/escalate decision). NEW: crates/orchestrator/src/lsp_daemon_client.rs (~120 LOC: reused-daemon spawn+client over E-52's language->tool map, transient/permanent classification). NEW: crates/orchestrator/src/path_rule_inject.rs (~110 LOC: edited-path fingerprint, .claude/rules priority resolver, Rules-slot dedup).
- MODIFY: crates/tools/src/lib.rs (SubagentToolExecutor return path — call the evidence gate before accepting the worker DONE; ~30 LOC); crates/orchestrator/src/hook_dispatcher.rs (register the `subagent_stop` dispatch + wire legs 2+3 into dispatch_post_tool_use:1971; ~50 LOC); crates/orchestrator/src/diagnostic_probe.rs (expose a daemon-mode entry reusing tool_for_language; ~25 LOC); crates/orchestrator/src/context_budget.rs (accept a path-keyed injected rule into the Rules slot with dedup; ~20 LOC).
- SSB: +SubagentEvidenceGated (Pattern-2; Kind: EvidencePresent / EvidenceMissingBlocked / NagExhaustedEscalated); +EditDiagnosticGated (Pattern-2; Kind: DiagnosticBlocked / DaemonTransient / ExtensionUnsupported); +PathRuleInjected (Pattern-2; Kind: RuleInjected / AlreadyResident / NoMatch). baseline+3; SCHEMA_VERSION COMPUTED (auto-follows SSB_ACTION_TYPES.len(), ssb.rs); full 9-tuple lockstep per action type; re-grep SCHEMA_VERSION + TOTAL_VARIANTS at edit (W-9). NOTE: if the action_type-lock review prefers one event with a wider Kind, the three may collapse to a single HookEnforcementEvent (6-7 arm Kind) — executor's call at lock time, but the THREE decisions must each remain journal-distinct (R1).
- frontend_pair: a HookEnforcementPanel tile on the existing Monitor/Hook panel family — counts evidence-gate blocks, per-edit diagnostic blocks, and rule injections per session (the operator's "how often did structural enforcement fire" surface); #[tauri::command] + ts-rs pair; cross_lang_diff.py PASS.

## WIRING_CONTRACT

produces_symbols:
  - crates/orchestrator/src/subagent_evidence_gate.rs::SubagentEvidenceGate (receipt parse + path-hardening + bounded-nag + block/escalate) [LEG 1]
  - the `subagent_stop` enforcement axis in hook_dispatcher.rs + the SubagentToolExecutor return-path call (crates/tools/src/lib.rs) [LEG 1]
  - crates/orchestrator/src/lsp_daemon_client.rs::LspDaemonClient (reused-daemon diagnostics over E-52's language->tool map; transient/permanent classification) [LEG 2]
  - crates/orchestrator/src/path_rule_inject.rs::PathRuleInjector (edited-path fingerprint -> Rules-slot injection, deduped) [LEG 3]
  - SubagentEvidenceGated / EditDiagnosticGated / PathRuleInjected SSB action_types

consumes_symbols:
  - crates/tools/src/lib.rs::SubagentToolExecutor (the worker-return seam + active_rules field — extend, do NOT re-implement) — GD-1
  - crates/orchestrator/src/hook_dispatcher.rs::dispatch_post_tool_use (the PostToolUse dispatch legs 2+3 ride) + run_ruff_post_tool_use (the proven block-feed shape) — GD-1
  - crates/orchestrator/src/diagnostic_probe.rs (E-52 language->tool map + DiagnosticProbeRequest — daemon-mode reuses it, never a 2nd toolchain) — GD-1
  - crates/orchestrator/src/context_budget.rs SlotWeights Rules slot (leg 3 injects here; dedup against live slot contents) — GD-1
  - conditional-routing.md (the doctrine leg 3 mechanizes); agent-dispatch-preamble R-8 / production-caller-gate D-030 (the prompt-plea leg 1 mechanizes) — GD-1

reachable_from:
  - the SubagentToolExecutor worker-return path on a real sub-agent dispatch (leg 1 EvidenceMissingBlocked / NagExhaustedEscalated)  [PRODUCTION sub-agent return seam]
  - hook_dispatcher.rs::dispatch_post_tool_use on a real edit-like tool call (leg 2 DiagnosticBlocked, leg 3 RuleInjected)  [PRODUCTION PostToolUse seam]
  - the three SSB rows + the HookEnforcementPanel tile are the off-agent artifacts; a #[cfg(test)] caller NEVER satisfies this (R7)

stub_ban:
  - NO accepting a worker DONE on the agent's stdout claim alone — leg 1 requires a real NON-EMPTY artifact the gate reads (R4: measure the objective, not the proxy)
  - NO treating leg-1's receipt as the audit verdict — the auditor still re-runs production_caller_check.py / python_wire_check.py off-agent (R5/R8); the receipt only proves an artifact exists
  - NO symlink / path-traversal escape on the receipt path (canonicalize + reject symlink + guard pre/post realpath, exactly as the source)
  - NO hard-kill on a sub-floor / no-evidence worker — bounded MAX_EVIDENCE_NAGS then ESCALATE (handoff R-4), never crash (W-2 graceful)
  - NO recording a daemon-UNREACHABLE LSP result (leg 2 transient — recording it silences the lane all session); only an unsupported-extension result is recorded
  - NO second LSP toolchain / NO bypass of E-52's language->tool map (leg 2 reuses it)
  - NO re-injecting a rule already resident in the Rules slot (leg 3 dedups); NO unbounded rule injection (priority-sourced, path-matched only)
  - NO new ELAI_*_ENABLE — each leg default-ON behind ELAI_SUBAGENT_EVIDENCE_GATE_DISABLE / ELAI_EDIT_DIAGNOSTIC_GATE_DISABLE / ELAI_PATH_RULE_INJECT_DISABLE (W-2/R-f)
  - NO .unwrap()/.expect() on prod paths; NO blake2/sha2 (blake3 for the receipt-path / rule fingerprint); NO silent fallback (a disabled gate logs that it is disabled, not a no-op)

frontend_pair:
  HookEnforcementPanel tile on the Monitor/Hook panel family, fed by the three SSB action_types (evidence blocks / diagnostic blocks / rule injections per session); #[tauri::command] + ts-rs + cross_lang_diff.py PASS. Justified: structural-enforcement frequency is the operator's "is the harness actually catching shortcut-bias" instrument (the believe-the-number surface).

observable_effect:
  trigger: "a real sub-agent dispatch where the executor returns DONE with no evidence receipt (leg 1); a real edit-like tool call that introduces a diagnostic (leg 2); an edit under a path with a matching local rule (leg 3)"
  expect_ssb: "SubagentEvidenceGated::EvidenceMissingBlocked then (after MAX nags) NagExhaustedEscalated carrying the worker id + attempt count; EditDiagnosticGated::DiagnosticBlocked carrying the file + diagnostic count; PathRuleInjected::RuleInjected carrying the edited path + the injected rule id — all three byte-recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the worker re-dispatched with the nag directive until it records a real non-empty artifact (leg 1); the diagnostic block fed inline before the next tool call (leg 2); the rule resident in the Rules slot on the next turn (leg 3); the HookEnforcementPanel tile rendering the counts. NEGATIVE that must FAIL: a worker DONE accepted on stdout alone with no artifact (leg 1 must BLOCK); a receipt path through a symlink / outside the evidence dir accepted (must BLOCK); a daemon-unreachable LSP result RECORDED as unsupported (leg 2 must keep it transient); a rule injected twice into the Rules slot (leg 3 dedup must prevent); any of the three gates a silent no-op when its DISABLE flag is set (must log the disable)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-52_returntime_perEdit_enforcement_hooks.md --run-trigger
rg "subagent_stop" .elai_cc/crates/orchestrator/src/hook_dispatcher.rs        # leg-1 axis present
rg "canonicalize|symlink|traversal" .elai_cc/crates/orchestrator/src/subagent_evidence_gate.rs   # path-hardening present
rg "MAX_EVIDENCE_NAGS" .elai_cc/crates/orchestrator/src/subagent_evidence_gate.rs                # bounded nag present
rg "SubagentEvidenceGated|EditDiagnosticGated|PathRuleInjected" .elai_cc/crates/orchestrator/src/ssb.rs   # 3 action types
rg -c "\.unwrap\(\)|\.expect\(|blake2|sha2" .elai_cc/crates/orchestrator/src/subagent_evidence_gate.rs    # MUST be 0
rg -c "ELAI_.*_ENABLE" .elai_cc/crates/orchestrator/src/subagent_evidence_gate.rs                 # MUST be 0 (kill-switch only)
test -f workflow/reasoning/J-52-decisions.md
```

## ACCEPTANCE
- [ ] leg 1: a worker DONE with no real non-empty evidence artifact is BLOCKED at the SubagentToolExecutor return; bounded MAX_EVIDENCE_NAGS then ESCALATE; a symlink/traversal receipt path is rejected; the auditor still re-runs the real checkers off-agent (receipt != verdict).
- [ ] leg 2: a per-edit diagnostic is block-fed inline reusing E-52's language->tool map via a reused daemon; a daemon-unreachable result stays transient (not recorded); output bounded under context pressure.
- [ ] leg 3: the matching local rule is injected into the Rules slot on the edited path, deduped against live slot contents, priority-sourced.
- [ ] each gate default-ON behind its ELAI_*_DISABLE kill-switch (W-2/R-f); a disabled gate logs the disable, never a silent no-op.
- [ ] frontend_pair: HookEnforcementPanel tile + #[tauri::command] + ts-rs; cross_lang_diff.py PASS.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator + -p elai-tools only.
- [ ] No new dependency (rust-analyzer/tsc/svelte-check are existing E-52 subprocesses; the daemon reuses them).
- [ ] SSB +3 (or the collapsed single event); SCHEMA_VERSION + TOTAL_VARIANTS re-grepped at edit, 9-tuple lockstep.

## WIRING DONE-GATE (clause by clause)
- W-1 missing evidence = FAIL: the three SSB rows are required; a gate that fired but logged nothing FAILS. W-2 kill-switch-not-enable: three ELAI_*_DISABLE, default-ON. W-3 no opt-in on the gate: the gates run unconditionally (only the DISABLE opts out, and it logs). W-4 measure-the-objective: leg 1 reads a real artifact, not stdout. W-5 sound-reference: leg-1 reachability is the real SubagentToolExecutor return; leg-2/3 the real PostToolUse dispatch. W-6: HookEnforcementPanel tile. W-7: every anchor re-grepped (PLAN-asserted line numbers above). W-8 gate frozen: J-52 does not edit production_caller_check.py / the wiring checkers (it composes them). W-9: SCHEMA re-grep at edit. W-10 integration: an independent integration smoke drives a real sub-agent dispatch + a real edit through the live entrypoint (not a unit shim). GD-1: composes SubagentToolExecutor/diagnostic_probe/context_budget/hook_dispatcher, restates none. GD-2: rows recomputable under ELAI_SEED. GD-4: the three gates default-ON -> FP-bound the block rate (a gate that blocks a legitimately-complete worker is a measured FP). GD-5: honest residue below.

## Tests to add
- crates/contracts/tests/j52_subagent_evidence_gate_contract.rs (~90 LOC): (a) DONE-with-real-artifact -> EvidencePresent pass; (b) DONE-no-artifact -> EvidenceMissingBlocked + re-dispatch; (c) MAX_EVIDENCE_NAGS exhausted -> NagExhaustedEscalated (handoff, not crash); (d) symlink receipt path -> BLOCK; (e) traversal-outside-evidence-dir receipt -> BLOCK; (f) receipt != audit verdict (auditor checker still runs).
- crates/contracts/tests/j52_edit_diagnostic_gate_contract.rs (~70 LOC): (a) diagnostic on edit -> DiagnosticBlocked block-feed; (b) daemon-unreachable -> DaemonTransient NOT recorded; (c) unsupported extension -> ExtensionUnsupported recorded once; (d) output bounded under the context-pressure flag.
- crates/contracts/tests/j52_path_rule_inject_contract.rs (~60 LOC): (a) edit under matched path -> RuleInjected into Rules slot; (b) rule already resident -> AlreadyResident (no double-inject); (c) no matching rule -> NoMatch; (d) priority source ordering.

## Files touched
- CREATE: crates/orchestrator/src/subagent_evidence_gate.rs (~140 LOC); crates/orchestrator/src/lsp_daemon_client.rs (~120 LOC); crates/orchestrator/src/path_rule_inject.rs (~110 LOC)
- MODIFY: crates/tools/src/lib.rs (~30); crates/orchestrator/src/hook_dispatcher.rs (~50); crates/orchestrator/src/diagnostic_probe.rs (~25); crates/orchestrator/src/context_budget.rs (~20); crates/orchestrator/src/ssb.rs (+3 action types, lockstep); crates/orchestrator/tests/action_type_lock.rs (+3); the Monitor panel family (HookEnforcementPanel tile) + its #[tauri::command] + ts-rs binding
- CREATE: crates/contracts/tests/j52_{subagent_evidence_gate,edit_diagnostic_gate,path_rule_inject}_contract.rs; workflow/reasoning/J-52-decisions.md

## Citations / Grounding
- lazycodex/OmO v4.11.1 real source (docs/.source/lazycodex-main-INDEX.md, opus-read 2026-06-19, MEDIUM-HIGH reliability, MIT): components/lazycodex-executor-verify (leg 1 codex-hook.ts runSubagentStopHook + the realpath/symlink/traversal hardening + MAX_ATTEMPTS=3 nag), components/lsp (leg 2 runLspPostToolUseHook + daemon reuse + transient/permanent distinction), components/rules (leg 3 dynamic-target-fingerprints + transcript-dedup). Steal verdict + prior-intake delta: workflow/reasoning/steal-lazycodex-reintake-2026-06-19.md.
- prior intake correction (CARRIED): STEAL-LIST.md (2026-06-10) analyzed lazycodex at v0.2.2 from a SUMMARY (MEDIUM); it captured only F1/F2. This re-intake reads v4.11.1 source (HIGH). F1 Stop-hook continuation stays DEFERRED (harness-level; PreCompact handoff stands); legs 1-3 are the NET-NEW enforcement mechanisms that v0.2.2 did not carry.
- ELAI ground truth (elai.map 2026-06-19, PLAN-asserted, re-grep W-7): hook_dispatcher.rs (for_task_level:352, dispatch_post_tool_use:1971, run_ruff_post_tool_use:890, NO subagent_stop), SubagentToolExecutor (tools/lib.rs:4214/4025 + active_rules), diagnostic_probe.rs (E-52, tool_for_language:124/137, notify @ task_runner.rs:14728 CoreValidationTier), post_tool_use.rs (ruff seam), context_budget.rs (Rules slot, SlotWeights), vibegate.rs:68.
- doctrine: production-caller-gate.md D-030 ("the fix has to be STRUCTURAL and off-agent" — leg 1 is the return-time mechanization), wiring-enforcement.md R4/R5/R6/R7/R8 (off-agent, sound-reference, frozen-gate), agent-dispatch-preamble.md R-8, conditional-routing.md (leg 3 mechanizes), E-52 (leg 2 extends).
- SSB: +3 (or collapsed +1); SCHEMA_VERSION COMPUTED -> baseline 268 / TOTAL_VARIANTS 577 as of LIVE re-grep 2026-06-16 (J-0_INDEX constants); NEVER hard-code, re-grep BOTH at edit (W-9/R-c).
- CLAUDE.md: blake3 only, no .unwrap in prod, scoped -p tests, W-2 kill-switch-not-enable, no new deps, ID-5 no fabricated stats.

## Open questions / risks
- The SubagentToolExecutor return seam (tools/lib.rs:4025/4214) is PLAN-asserted; if the worker-return is assembled elsewhere the executor re-greps for the AgentResult/"### Final response" assembly point (W-7) and wires the gate there — a material relocation is a handoff fork (R-4), not a silent move.
- The reused LSP daemon lifecycle (one per session) is in-process; a daemon crash mid-session must degrade to E-52's batch probe, never block edits — decide-at-executing whether the daemon failure path falls back to the existing CoreValidationTier probe.
- SSB count: 3 distinct events vs 1 collapsed event is an action_type-lock-review call; the THREE decisions must stay journal-distinct either way (R1). Recorded.
- leg 1 must not become a self-verification loop the worker games (it writes any non-empty file to pass): mitigated because the receipt only unblocks the RETURN — the auditor's off-agent checker is still the verdict (R5/R8); the gate is the cheap tripwire, the auditor the authority.

## Riders (recorded, not in-scope)
- Cross-session persistent LSP daemon pool (Tier-2; Tier-1 is per-session reuse). 
- Receipt-content schema validation beyond existence+non-empty (Tier-2; the auditor owns content).
- Learned per-path rule-relevance scoring (Tier-2; Tier-1 is the static fingerprint map).
- POST-COMPACT RULE RE-INJECTION (lazycodex rules/runPostCompactRecovery) = a rider on H-50/H-51 (largely covered by the CONTEXT BUDGET ALLOCATOR re-filling the Rules slot each turn); recorded, not a J-52 leg.
- Runnable end-of-plan aggregate-artifact validator (lazycodex quality-gate.ts) = a rider on the auditor; recorded.

End of J-52.
