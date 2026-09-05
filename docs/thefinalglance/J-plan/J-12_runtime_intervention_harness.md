# J-12: runtime intervention harness — the Life-Harness 4-layer port (deterministic agent-loop interventions, LLM-free at runtime)

- Track J (determinism-steal extensions) | lever CANT-DERAIL | grounding STRONG (paper + source ingest) / EXTERNAL (figures) | nature BUILD | effort L | WAVE 3 | depends none (ideation-side curator leg cites I-12, not gated by it)

- Premise: Life-Harness (arXiv:2605.22166, MIT, source at docs/.source/life-harness) shows a frozen-model agent improved across 116/126 model-env settings by adapting the RUNTIME INTERFACE only — four deterministic intervention layers, zero LLM in the loop at eval time. The mechanism survived an 18-backbone transfer because the structure is environment-side. ELAI has every seam the four layers need (rules engine, hook dispatcher, skill library, anti-spiral) but no unified intervention taxonomy and no failure-to-intervention authoring pipeline. The 88.5% mean relative improvement and per-layer ablations (-61.7% Airline without the action gate, -86.5% ALFWorld without trajectory regulation) are EXTERNAL figures [DESIGN], never claimed; the done-gate produces ELAI-measured deltas (HD-1/ID-5).

## Steal/build
Port the four layers onto EXISTING ELAI seams (compose, never re-implement):
1. H2 ACTION-REALIZATION GATE — deterministic pre-execution check on tool calls: protocol violations (missing args, type errors, invalid tool names, state-machine violations like cancel-on-non-pending) return a model-visible Block(message) so the agent self-corrects; ambiguous-but-possibly-correct actions are NEVER blocked (paper hard constraint). Lands as a new rule predicate family in crates/rules (lifecycle PROPOSED->SHADOW->ACTIVE applies; JD-3/GD-4 observe-only first) evaluated from the EXISTING PreToolUse dispatch chain.
2. H3 TOOL-DESCRIPTION EMBEDDING — constraint text injected into TOOL DESCRIPTIONS, not the system prompt: tool defs are re-sent every request so constraints hold constant attention across turns (the source's key delivery insight). Lands as a decoration pass where tool schemas are serialized for dispatch.
3. H4 POST-EXECUTION ANNOTATORS — per-tool result annotators appended to the serialized tool result (read-only, return Option<String>, never raise); distinct from C-10 stagnation detection which stays untouched (C-10 watches the trajectory; H4 annotates individual results).
4. H5 SKILL RETRIEVAL — task-start top-1 retrieval from a skill library with the source's precision-guard + min-score pipeline (BM25 K1=1.5 B=0.75, smoothed IDF, top-1 HARD CAP against context contamination). Composes F-6 learned-skill library (SkillTrustTier gates what is retrievable); the OFFLINE failure-classification pipeline (priority order: action-realization > contract-mismatch > trajectory-degeneration > residual-reasoning, then propose + REGRESSION-CHECK) is authored as the I-12 playbook-curator intake shape — J-12 ships the classifier taxonomy + regression-check harness, I-12 stays the promotion owner.
EXCLUDED: the paper's per-benchmark harness content (TauBench/AgentBench rules — env-specific, not portable); any LLM in the runtime loop; any blocking default before GD-4 FP bounds.

## Existing ground truth (grep-confirmed 2026-06-11)
- rules engine: crates/rules/src/lifecycle.rs:1 (PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED), schema.rs:39 status enum; predicates AST/GREP/SCHEMA/METRIC/COMPOSITE (phase-3F WIRED).
- hook dispatch: crates/orchestrator/src/runtime_setup.rs:120 build_production_hook_dispatcher; production callers cli/src/task_cmd.rs:431 + run_subcommand.rs:376 + dashboard src-tauri lib.rs:1761. PreToolUse chain already carries F-1 vibegate + F-9 dep gate + E-30 argv allowlist — H2 joins this chain, never forks it.
- anti-spiral: C-10 StagnationPattern 4 variants + personas fire unconditionally in drive_task (task_runner.rs:7143-7330 PLAN-asserted from feature map; executor re-greps per W-7).
- skill library: F-6 SkillTrustTier S1-S4 + skill_permits_autonomous in production (feature-map WIRED; executor re-greps).
- interface maps (ground truth for the port, W-3): workflow/papers/life-harness-paper-interface.md (formalism, invariants, ablations) + workflow/papers/life-harness-source-interface.md (verbatim rule/annotator/skill shapes, BM25 params, call order).

## Integration
- new crates/orchestrator/src/intervention/{mod,action_gate,tool_decor,annotate,skill_retrieval,classify}.rs (~700-900 LOC total); rules rows live in the EXISTING crates/rules SQLite store under a new category (no schema fork).
- call order per turn mirrors the source: tool-decor already applied -> agent proposes -> H2 check -> execute -> H4 annotate -> agent sees result. H5 fires once at task start; budget formula b_t = B - t - 1 passed to H4.
- +HarnessInterventionEvent SSB (Pattern-2 single-variant; Kind: ContractInjected / ActionGateBlocked / ResultAnnotated / SkillRetrieved). baseline+1 from LIVE 229/540 (ssb.rs:3066 / action_type_lock.rs:662, grep 2026-06-11); NEVER hard-code, re-read at edit (W-9). Full 6-part variant discipline.
- frontend_pair: intervention rows in an existing drawer (decide at edit; W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/intervention/action_gate.rs (H2 gate: check(tool_call, rules) -> Exec | Block(msg))
- crates/orchestrator/src/intervention/tool_decor.rs (H3 description decoration pass)
- crates/orchestrator/src/intervention/annotate.rs (H4 annotator registry, Option<String> contract)
- crates/orchestrator/src/intervention/skill_retrieval.rs (H5 BM25 top-1 + precision guard + min_score)
- crates/orchestrator/src/intervention/classify.rs (offline failure-priority classifier + regression-check harness)
consumes_symbols:
- crates/rules/src/lifecycle.rs::RuleLifecycle (intervention rules ride the existing lifecycle; SHADOW before ACTIVE)
- crates/orchestrator/src/runtime_setup.rs::build_production_hook_dispatcher (H2 joins the existing PreToolUse chain)
- F-6 skill library trust tiers (H5 retrieves only permitted skills)
- C-10 stagnation primitives (H4 composes alongside; NOT re-implemented)
reachable_from:
- crates/cli/src/task_cmd.rs:431 -> build_production_hook_dispatcher -> PreToolUse chain  [PRODUCTION task dispatch]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM call anywhere in the runtime intervention path (paper invariant; TD-3 analog)
- NO blocking verdict by default before the GD-4 FP bound (>=3 real runs); ships SHADOW
- NO re-implementation of C-10 stagnation detection or F-1/F-9/E-30 gates
- NO >1 retrieved skill (top-1 hard cap is a contract assert)
- NO blocking of ambiguous-but-possibly-correct actions (block list is deterministic protocol violations only)
- NO .unwrap()/.expect() on prod paths; blake3 only where hashed
frontend_pair: intervention rows in an existing drawer (decide at edit; W-4) or W-6-justified absence.
observable_effect:
- trigger: "elai run on a fixture task whose tool call violates a SHADOW-promoted-to-ACTIVE protocol rule"
- expect_ssb: "HarnessInterventionEvent{Kind: ActionGateBlocked} with the rule id + model-visible message; SkillRetrieved row at task start when the library has a >=min_score match"
- expect_artifact: "the blocked call never reaches the tool dispatcher (journal ordering provable); the agent's NEXT turn carries the Block message. NEGATIVE (R2): with the rule in SHADOW, the same call EXECUTES and lands an observe-only row; a registry with zero rules makes the gate a provable no-op (never a silent crash); the --no-intervention arm re-makes the protocol error."

## Grounding
- arXiv:2605.22166 (WebSearch-verified at triage 2026-06-11; agent S2 + ingest p2p-ingest-1/2). Ablations + invariants: workflow/papers/life-harness-paper-interface.md. Verbatim shapes + BM25 params + call order: workflow/papers/life-harness-source-interface.md (MEDIUM reliability, reviewed source).
- steal ledger: workflow/reasoning/steal-determinism-repos-2026-06-11.md Tier-1 item 1.
- LIVE baseline 229/540 grep 2026-06-11 (ssb.rs:3066 / action_type_lock.rs:662).

## COMPOSES-WITH
rules engine (phase-3F), hook system (phase-4C, F-1/F-9/E-30 chain), F-6 skill library, C-10 anti-spiral, I-12 playbook curator (offline pipeline intake shape), E-46 planning extras. Restates none.

## Open questions / risks
- which ELAI envs get seed intervention rules first: tool-protocol class (file ops, git read-only, FSM transitions) is the deterministic-rule-governed scope the paper requires; open-ended reasoning tasks are OUT of scope (paper warns explicitly).
- H3 decoration must not bust the byte-stable cache prefix (HD-2): tool defs sit before the cache breakpoints — decoration must be byte-stable per session. decide placement at edit.
- BM25 here vs crates/knowledge retrieval: the source's self-contained scorer is ~60 LOC and avoids coupling the hot intervention path to the knowledge stack; keep self-contained, revisit if H5 grows.

## Next
Closes the "no unified intervention taxonomy" gap; gives I-12 its intake classifier and regression-check; pairs with J-15's D-11 solver-feedback amendment (both are deterministic self-correction loops).

End of J-12.
