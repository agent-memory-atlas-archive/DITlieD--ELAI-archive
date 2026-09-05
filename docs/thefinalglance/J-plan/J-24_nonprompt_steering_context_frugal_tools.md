# J-24: non-prompt steering activation + context-frugal tools — steer behavior and carry tools without burning context

- Track J (pillar-SOTA recheck extensions) | lever EFFICIENT / STEERING | grounding STRONG (sidecar /v1/steering endpoints live; H-4/H-34 substrate) | nature BUILD (activate dark substrate) | effort M | WAVE 3 | depends none

- Premise: this unit answers the user's two direct questions (2026-06-12) as a real plan unit. (Q1) "ELAI still uses prompts to steer generation — could we steer the model behavior/personality some other way?" YES, and it is partly built: the sidecar exposes /v1/steering/* endpoints + ActAdd activation steering (phase-3J), currently DARK (--no-steering on boot). Activation steering shapes behavior with ZERO context tokens (a vector added to the residual stream, not text in the window). (Q2) "Tools/MCP — is there any way to not have them consume context? Could the infra handle that?" PARTLY: H-4 Tool Search Tool (77K->8.7K tool-def), H-34 MCP defer-loading, E-22 lazy MCP schema already plan the infra lever (load tool schemas on demand, RAG-over-tools). The fundamental floor: the model needs SOME schema in context to CALL a tool; the win is lazy/deferred, not zero. J-24 activates the steering leg (the genuinely-dark capability) and threads the deterministic context-frugal pieces.

- HONEST FRAMING (the user's "are ELAI rules really minimal and measured?"): MINIMAL yes — rules are DATA evaluated in Rust, cost ZERO context tokens, fire 100% (vs a skill which costs tokens every load AND is probabilistic). MEASURED not yet — SAFEDELTA + the efficiency benchmarks are unrun (G-6 BENCH-PENDING). J-24 does not claim a measured win; it activates the steering lever and threads the size-pre-assessment so the eventual benchmark has something to measure.

## Steal/build
1. ACTIVATION-STEERING ACTIVATION (the genuinely-dark capability): un-dark the sidecar /v1/steering path — activation steering (ActAdd-style: add a learned/contrast vector to the residual stream at generation) as a behavior/personality lever that costs ZERO context tokens. Build the vector-extraction + apply path (the endpoints exist; the steering vectors + the apply-on-generate need wiring), behind a flag, and MEASURE its effect vs an equivalent role-prompt (the believe-the-number framing: does a steering vector achieve the same behavior shift as N tokens of role prompt, at 0 token cost). This is the concrete answer to "steer some other way".
2. CONTEXT-FRUGAL TOOLS thread (activate H-4/H-34/E-22): assert the Tool Search Tool + MCP defer-loading path on the production OAuth lane (H-4 is PARTIAL: advanced-tool-use beta, 77K->8.7K tool-def cut) + the STDIO defer-loading (H-34 PARTIAL: McpToolDeferred 212). The infra lever loads tool schemas on demand instead of all upfront. J-24 does not rebuild H-4/H-34; it wires the activation + measures the tool-def token delta.
3. DETERMINISTIC OBSERVATION-SIZE PRE-ASSESSMENT (the clean P4 change, BACM): before committing an observation to the context, query its token size; if observed > slot_remaining x alpha, trigger compaction BEFORE the insert (3-mode NONE/SELECTIVE/ALL). This is the DETERMINISTIC part of BACM (a budget check, not a neural rewrite) — compatible with the <50ms discipline (a size check, not compression) and with IC-3 (no cipher/LLM-rewrite). The NEURAL SWE-Pruner skimmer is EXCLUDED here (IC-3 tension; recorded DEFERRED/opt-in-measured in the ledger).
EXCLUDED: claiming a measured steering or token win before the benchmark (G-6 BENCH-PENDING; HD-1); the SWE-Pruner neural line-skimmer (IC-3 deterministic-only tension — deferred, measured trial first); finetune-based steering (E-47/E-55 deferred); force-JSON/constrained-decode on reasoning roles (HD-3 banned; steering is a residual-stream vector, not a decode constraint); rebuilding H-4/H-34/E-22 (activate, do not re-author).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- PYTHON SIDECAR [WIRED] FastAPI /v1/messages + /v1/steering/* ; model.generate() + LogitsProcessor chain (DoLa, ETC, prompt-lookup, XGrammar mask); ActAdd steering named in phase-3J; --no-steering on the live boot (feature map 1.9; CLAUDE.md remote-sidecar boot line) — the steering path J-24 un-darks.
- TOOL SEARCH TOOL [PARTIAL] advanced-tool-use-2025-11-20 beta on OAuth, 77K->8.7K tool-def (H-4).
- MCP DEFER-LOADING [PARTIAL] Tool-Search emulation for STDIO servers, McpToolDeferred (212) + McpSchemaBudgetMeasured (213) (H-34).
- TOOL GATING + LAZY MCP [WIRED] dynamic gating, ELAI_TOOL_GATE_OFF (C-35); MICROSANDBOX/TRENDING lazy MCP schema (E-22).
- CONTEXT BUDGET ALLOCATOR [WIRED] 8 fixed slots, floors+steal, 3-level adaptive reduction <50ms (phase-3E) — the slot-remaining the size-pre-assessment checks against.
- DETERMINISTIC OBSERVATION MASKING [PARTIAL] adaptive_reduce + recover_stubs (H-6) — the compaction the size-pre-assessment triggers.

## Integration
- python/elai_inference/server.py + crates/providers: the steering-vector extraction + apply-on-generate path (~150 LOC: vector store + residual-stream add at generate, behind a flag; the /v1/steering endpoints exist) + a steering-vs-role-prompt measurement harness.
- crates/orchestrator/src/context_budget.rs: the observation-size pre-assessment (~60 LOC: query size, compare to slot_remaining x alpha, trigger H-6 compaction before insert; NONE/SELECTIVE/ALL).
- the H-4/H-34 activation is wiring + measurement (the tool-def token delta), not new code.
- +BehaviorSteeringEvent SSB (Pattern-2 single-variant; Kind: SteeringVectorApplied / SteeringEffectMeasured / ObservationSizePreassessed / ToolSchemaDeferred). baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit (W-9). PREFER-REUSE: ToolSchemaDeferred may reuse H-34 McpToolDeferred (212); decide at edit (net 0..+1).
- frontend_pair: a steering-effect + token-budget surface on the existing runtime/context panel (decide at edit; W-4) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- python/elai_inference/server.py steering-vector apply-on-generate (the dark /v1/steering path, activated)
- crates/providers steering-vector extraction + the steering-vs-role-prompt measurement harness
- crates/orchestrator/src/context_budget.rs observation-size pre-assessment (deterministic BACM NONE/SELECTIVE/ALL)
consumes_symbols:
- the sidecar /v1/steering/* endpoints (exist; the apply path is wired, the endpoints NOT reauthored)
- H-4 Tool Search Tool / H-34 MCP defer-loading (activated + measured; NOT reauthored)
- H-6 adaptive_reduce / context_budget slots (the compaction the size check triggers; NOT reauthored)
reachable_from:
- a real `elai run --local` with steering on (inference dispatcher -> sidecar /v1/steering apply at generate)  [PRODUCTION local lane, remote-GPU-gated W-10]
- a real `elai` run on the OAuth lane with Tool Search on (the tool-def cut)
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO claiming a measured steering or token win before the benchmark runs (G-6 BENCH-PENDING; HD-1 measured-per-config)
- NO SWE-Pruner neural skimmer in the context path (IC-3; the size-pre-assessment is the deterministic part only)
- NO force-JSON / constrained-decode framed as "steering" (HD-3; steering is a residual-stream vector)
- NO new default-ON steering / tool-defer until the effect + FP/friction is bounded on >=3 real runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
- NO zero-token-tools CLAIM (the floor is real: some schema must be in context; the win is lazy/deferred, stated honestly)
frontend_pair: steering-effect + token-budget on the runtime/context panel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "two `elai run --local` runs, one with a behavior-steering vector applied (e.g. a terseness/anti-sycophancy vector) and zero role-prompt tokens, one with the equivalent role-prompt and no steering — same task, same seed (remote-GPU-gated W-10)"
- expect_ssb: "SteeringVectorApplied on the steered run; SteeringEffectMeasured comparing the two runs' behavior delta at their respective context-token costs; ObservationSizePreassessed on a grep observation that exceeds the slot, triggering compaction before insert; ToolSchemaDeferred on a lazily-loaded MCP tool"
- expect_artifact: "the steered run achieves the behavior shift at ZERO context tokens vs N tokens for the role-prompt run (the measured answer to 'steer some other way'); the Tool Search lane shows the 77K->~8.7K tool-def delta. NEGATIVE (R2): steering off -> the behavior reverts (the vector is the cause, observable); an observation that fits the slot does NOT trigger compaction (no false pre-assessment); a deferred tool that is actually NEEDED still loads on demand (defer != drop); no zero-token-tools claim is emitted."

## Grounding
- sidecar /v1/steering + ActAdd (feature map 1.9 PYTHON SIDECAR [WIRED]; phase-3J ActAdd steering; CLAUDE.md --no-steering live boot) — the dark capability activated. ActAdd is the activation-steering method (residual-stream contrast vectors); the apply path is the build.
- H-4 Tool Search (feature map 1.9 PARTIAL, 77K->8.7K), H-34 MCP defer-loading (PARTIAL, McpToolDeferred 212), E-22 lazy MCP, C-35 tool gating — the context-frugal-tools substrate activated.
- BACM / ContextBudget arXiv:2604.01664 (VERIFIED via verify-p4: NONE/SELECTIVE/ALL observation loading, 30B/8K beats 235B/128K) — the deterministic size-pre-assessment. SWE-Pruner arXiv:2601.16746 (VERIFIED neural skimmer) EXCLUDED here (IC-3; ledger DEFERRED).
- G-6 SAFEDELTA BENCH-PENDING (the measured-win is gated on it; J-24 activates, does not claim). live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
phase-3J sidecar steering (un-darks it), H-4/H-34/E-22/C-35 (activates the context-frugal-tools lever), H-6 + context_budget 3E (the size-pre-assessment triggers the existing compaction), I-32 structural role scoping (steering is the non-prompt complement to dropping narrative personas from executor/researcher), G-6 SAFEDELTA (the benchmark that eventually measures the steering/token win). Restates none.

## Open questions / risks
- steering-vector provenance: a behavior vector is a learned artifact — it must promote through the ledger like any other (ID-6) and not become an un-audited backdoor on model behavior; extract it deterministically (contrast pairs) and version it.
- steering stability: activation steering can degrade coherence if the vector magnitude is too high (the DoLa-class risk) — measure coherence alongside the behavior shift; default conservative, GD-4-bounded.
- the honest tools answer: do NOT oversell — the model needs some schema in context to call a tool; J-24's win is lazy/deferred loading + RAG-over-tools, measured as a token delta, not a zero claim. State the floor in the capability echo.

## Next
Activation steering is the concrete answer to "steer behavior without prompts" — it makes ELAI's enforcement-in-harness / steering-minimal-in-prompt split MORE true (a behavior lever at 0 token cost). The size-pre-assessment + tool-defer feed the eventual SAFEDELTA/efficiency measurement that turns "minimal" into "minimal AND measured".

End of J-24.
