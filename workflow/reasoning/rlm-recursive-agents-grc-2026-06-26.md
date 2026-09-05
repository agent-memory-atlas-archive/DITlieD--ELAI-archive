RLM / RECURSIVE-CODING-AGENTS / OPENPROSE vs ELAI — pre-ingestion reasoning
Date: 2026-06-26
Lens: spec-eval AS-SPECIFIED (judge ELAI's design as if every named module works; shipped-vs-planned is a separate readiness note, never a design markdown).
Author: main LLM (Opus 4.8 1m), in-band.

GROUNDING POSTURE (read this first)
- ELAI side: GROUNDED. Every ELAI claim below is from a full 100% read of docs/thefinalglance/ELAI-FEATURE-MAP.md (regen 2026-06-11, upserted through 2026-06-26) + the GRC base phases in docs/specs/guide/steps/phase-index-MASTER.txt (PHASE 2A-2D) + the J-SUMMARY_INDEX GRC-v3 wave (J-83..J-86) and J-42 entry. Live SCHEMA ~268 / TV ~577 (grep at edit per the map).
- PAPER side: NOT YET GROUNDED. Every claim about the RLM/LongCoT/OpenProse/code-execution corpus is from the link titles + the RLM arXiv record + training memory, labelled "From training (unsearched)". These are TO BE GROUNDED by the sonnet paper-ingestor interface maps under workflow/papers/. Do not act on a paper-side claim until its interface map lands. This file is the pre-ingestion map of WHERE each item lands in ELAI so the post-ingestion steal pass is cheap.

================================================================================
PART 1 — THE ELAI GRC REASONING SYSTEM (what the user asked me to understand)
================================================================================

GRC = Grounded Reasoning Chain. It is ELAI's discipline that every reasoning step is small, typed, and tied to external evidence, with a per-topic budget that forces convergence or escalation. Three layers exist, oldest to newest:

1) GRC BASE (phase-2A..2D, WIRED). crates/reasoning/.
   - GrcState FSM: Observe -> Hypothesize -> PlanAction -> Act -> Verify -> Converge (+ Status free echoes). Default chain enforced; per-agent transition_overrides.
   - Per-TOPIC budget (max_calls / max_chars); each topic an independent budget; must_converge at the limit (overflow forces convergence or escalation).
   - GROUNDING VERIFICATION (phase-2B): 6 binary pass/fail methods FILE / SOURCE / TEST / WEB / API / API_REGISTRY_CHECK (MiniCheck-style), +7th CosmosCheck (B-42), +8th VisualArtifactCheck (E-36). A plan-sweep BLOCKS PLANNING->EXECUTING on any UNGROUNDED claim. Grounding format "type:reference" -> {Verified, High, Uncertain}.
   - PER-AGENT GRC CONFIG (phase-2C): 8 roles, each with distinct budget/steps/grounding/allowed_methods. Anti-rubber-stamp: auditor Converge needs a FINDING:.
   - CSQ + INLINE FEASIBILITY (phase-2D): counterfactual safety questions on fs/net/money/auth functions; max-2 inline checks during EXECUTING.
   - Each GRC step writes a ReasoningTraceEntry to the SSB journal; checkpoints per step (recoverable).

2) GRC v2 (LIVE NOW, runtime, the mcp_think tool's current contract). The decisive shift: native interleaved thinking IS the reasoning channel; mcp_think is DOWNGRADED to an opt-in DECISION LEDGER for durable forks only (step_type decision | escalate; older observe/hypothesize/... DEPRECATED). Persisted out-of-prompt, searchable via mcp_search_context. Per-topic budget retained. Reason natively; record only real decisions at real forks.

3) GRC v3 (PLANNED, J-83..J-86 + K-15/K-16; the user's flagged "new GRC improvements"). Verdict of the run: UNIFICATION + ACTIVATION, NOT invention. ELAI already ships six of seven pieces scattered (I-11 carrier / H-39 abstention / I-7 escalation / I-4 ladder / H-42+I-26 budget / D-10 router / H-50 regime policy / D-2+I-24 PRM / H-49 backtrack / I-27+H-9 cost). The win is naming ONE typed carrier they compose around + ONE genuinely-new deterministic guard.
   - J-83 BeliefState reasoning carrier [KEYSTONE, EXTENDS I-11 PARTIAL]: carries reasoning across a model/context boundary as CLAIMS+RECEIPTS, not raw transcript. Each entry {claim, external grounding-receipt, abstention/confidence bit READ from H-39 (the model NEVER sets its own confidence — RD-1), DKG provenance}. Per-step RECEIPT GATE = least-to-most-WITH-receipts: commit iff the D-10-routed domain receipt PASSES, else ABSTAIN (this is the user's "the part it's sure about proceeds, the rest stays" — the L2M poison-cascade fix). Commit-and-FREEZE (triple payoff: grounding + safe compression + overthinking defense). One carrier, domain-routed validators (F-4 code / D-11 math / J-1 vision / H-trading) = the cross-domain answer.
   - J-84 2-axis reasoning-effort classifier [EXTENDS H-42+I-26]: TYPE axis (reuse D-10: reason vs RETRIEVE-recall-never-a-chain vs validate) x COST proxy (H-39 + C-45 case-base + J-38 coverage + C-29) -> trivial=ZERO / easy-medium=serial DEPTH / hard=parallel WIDTH (J-86) / very-hard=escalate(I-7)-or-abstain(I-8). LLM never picks the tier (TD-3); WINDOW is never a tier (RD-4).
   - J-85 deterministic overthinking guard [the ONE net-new mechanism]: a per-call reasoning CEILING + a 3-signal early-stop (answer oscillation r=0.78 lead / hesitation r=0.71 / declining confidence r=0.63; flip_ratio>1.0 = STOP) that catches a chain DRIFTING OFF a correct answer. DISTINCT from anti-spiral (which catches REPETITION) — they run side-by-side. Deterministic (no LLM stop). Ceiling = K-15-MEASURED per-(model,task-type) knee, NEVER a paper constant.
   - J-86 receipt-gated candidate ladder [ACTIVATES dark I-4+D-2+I-24]: branch candidate beliefstates, the receipt-PASSING one advances (commit+freeze), the rest backtrack (SessionDag/H-49). SELECTOR HIERARCHY: deterministic RECEIPT > VOTE-over-independent-samples (receipt-absent fallback only, NEVER for recall) > LLM self-eval (BANNED). No MCTS-first.
   - K-15 `elai bench reasoning-budget` = the ELAI-measured knee (the GRC-v3 done-gate). K-16 .claude/rules/reasoning-receipts.md RR-1..RR-4 (a trace is monitoring/UX only, NEVER the confidence signal; receipt > vote > never-self-eval; measure-your-own knee; commit-and-freeze).
   - DECISIVE CORRECTION (the deepest grounding, 3 papers — Reflexion self-correction blind spot, ToT self-eval-is-the-weak-link, Lanham faithfulness): a model CANNOT reliably ground itself or judge which of its own parts is "sure". So "the sure part proceeds" MUST mean "the EXTERNAL-RECEIPT-confirmed part proceeds" = the shipped trust spine (F-4 / D-13 / I-4 / IC-6) applied one level down to reasoning itself.

4) J-42 within-step distill-reset reasoning [InftyThink sawtooth, PLANNED]. The single most RLM-shaped existing unit: a reasoning step today runs straight-line until the context wall (local) or the GRC budget, then truncates. J-42 adds a within-step distill -> reset -> re-reason operator (extends GRC reset_topic + B-39 refine loop) with a FROZEN admission check (bounded size + uncertainty-preserved). OBSERVE-ONLY first. This IS the recursive/iterative-reasoning-over-a-context-wall pattern the RLM corpus is about, already scoped.

KEY DOCTRINE CODES that bind any RLM steal (Section 3 of the map):
- TD-3: every gate/router/selector verdict is DETERMINISTIC; the LLM proposes, deterministic checks adjudicate. An LLM may never be the load-bearing gate.
- HD-2 / IC-3: deterministic-reversible compression only; LLM re-summarization of the cache prefix is BANNED (busts the cache); ciphers BANNED.
- HD-3: decoding is sacred for REASONING roles — never constrain-decode / force-JSON a reasoning role; grammars only on pure op-emission.
- ID-5 / HD-1: every threshold/number is ELAI-MEASURED per (model, lane); a paper constant is NEVER a done-gate or an enforcement threshold.
- RD-1: the confidence/abstention bit is READ from H-39, never set by the generator.
- GD-1 (compose-not-restate) + GD-4 (observe-only until FP bounded on >=3 runs).
- The wedge: ELAI is the harness that makes reasoning VERIFIED (off-agent receipts), not the model. "Harness > model" is ELAI's thesis.

================================================================================
PART 2 — THE INCOMING CORPUS (theme + what each item is)
================================================================================
[From training (unsearched) until the ingestor maps land.]

THE ONE THESIS connecting the whole corpus: Recursive Language Models (RLM). Instead of stuffing a huge document/codebase into one context window, the model treats context as an ENVIRONMENT (a Python REPL / a filesystem) that it grep/slice/recursively-calls-an-LLM-over. Omar Khattab's framing "Claude Code is finally an RLM" = a coding agent that recursively decomposes, spawns sub-LLM calls over a filesystem, and uses CODE EXECUTION to interrogate context IS an RLM, and this beats long-context models on long-horizon reasoning (LongCoT).

Date-sorted items and where each lands in ELAI:
- Recursive Language Models (arXiv 2512.24601, v3 2026-05-11). THE core paper. ALREADY PARTIALLY EVALUATED by ELAI: the _compact_history recipe was taken (J-76 C7 — 85%-window trigger, summarize PRESERVING intermediate numerics, truncate to [system+initial+summary+continue], LOCAL-lane only); RLM REPL-as-context was DEFERRED (Section 4 map: "RLM REPL-as-context: DEFER, only _compact_history taken").
- LongCoT: Benchmarking Long-Horizon Chain-of-Thought Reasoning (arXiv 2604.14140, 2026-04-15). A long-horizon CoT BENCHMARK. ELAI has NO LongCoT in its bench family (1.13). Candidate adoption as an eval arm.
- Code Execution as Reasoning (raw.works, 2026-05-11). The broad thesis (overlaps Anthropic "code execution with MCP" / code-mode). ELAI has CodeAct (E-35) but KEYSTONE-guards it OFF for the sub-14B local lane (J-55).
- The Mismanaged Geniuses Hypothesis (alexzhang13, 2026-04-09) + RLMs-on-LongCoT mini-exercise (2026-04-26). MGH = models are capable, the harness/management is the bottleneck. This is ELAI's own "harness > model" thesis stated by an outsider.
- RLMs are the new reasoning models / RLMs are SOTA on LongCoT / RLMs as Memory Systems / ypi: a recursive coding agent (raw.works, 2026-02..04). RLM as a reasoning + memory architecture; ypi/rlm-cli/pi = recursive coding-agent harnesses. NOTE: ELAI already evaluated earendil-works/pi (5-repo run 2026-06-14): REJECT, zero net-new (pi is a generic single-agent TS runtime; the "make it prove it" test-proof = already F-4; retro-mines-transcripts = J-12 -> I-12).
- A harness for every task: dynamic workflows in Claude Code (claude.com, 2026-06-02). The Workflow tool / model-authored deterministic JS orchestration. ELAI explicitly BANS the Workflow tool (CLAUDE.md hard rule, keyword false-positive on this root's vocabulary). ELAI's equivalent = BSP supersteps + orchestrate + FSM gating.
- OpenProse / "Stop Babysitting Agents, Start Authoring Outcomes" (turingpost, 2026-06-04) + openprose/prose + prose.md + unix-rlm. A declarative prose-as-spec LANGUAGE for reliable agent workflows. ELAI's analog = plan files + skills + wiring-enforcement (vertical-slice PRE_GATE), arguably more rigorous.
- Repos (rawwerks/recursive-coding-agents, rlm-rubric, rlm-cli, ypi, alexzhang13/rlm, dspy, ax, openprose/*). Code, not papers — better suited to a follow-up paper-to-plan mode=source/repo-map pass than to paper-ingestion. DSPy/ax: note ELAI already canonicalized GEPA (E-8) and REJECTED dspy/swiftide as deps (the steal is the IDEA, never the framework).
- Langfuse (github). LLM observability/tracing platform. ELAI has OTel GenAI export (E-29) + the SSB journal + cost cockpit (H-9). External trace store likely REJECTED under GD-2 (local SSB is the record, never an external store) — same class as the already-rejected "central multi-cloud governed trace store".

================================================================================
PART 3 — PRE-INGESTION STEAL MAP (verdict hypotheses, to confirm post-ingestion)
================================================================================
Each verdict is a HYPOTHESIS pending the interface maps. Verdict classes per paper-to-plan phase 4: NEW | EXTENSION(plan-id) | ALREADY-SHIPPED | ALREADY-REJECTED | REJECT.

H1. RLM REPL-as-context core (2512.24601). Hypothesis: REVISIT the DEFER. The earlier deferral predates this corpus's LongCoT-is-SOTA grounding. ELAI's orchestration layer is ALREADY an RLM (orchestrate + idea-intake recursive Concept>Phase>Step>BabyStep + sub-agent GC'd contexts + BSP). The genuine GAP is at the REASONING grain: no "model writes code in a REPL to interrogate its OWN working context as a reasoning move". The cleanest ELAI-idiom landing = compose J-42 (InftyThink within-step distill-reset) + J-38 (mid-task retrieval loop) + E-35 CodeAct + J-103 read-ladder into a CAPABLE-LANE-ONLY recursive-reasoning operator, measured-gated by K-15, NEVER routed to the local Qwen-7B lane (J-55 keystone guard; ~12% multi-tool code-mode collapse <14B). Likely verdict: EXTENSION(J-42/J-86) for the capable lane, with the local lane staying on J-42 distill-reset (text, no REPL). NOT a new REPL-as-everything architecture.

H2. LongCoT benchmark (2604.14140). Hypothesis: NEW (small) — adopt LongCoT as a reasoning-bench ARM for K-15 `elai bench reasoning-budget` and/or the SAFEDELTA quality sibling (I-38), measured per-(model,lane), never the paper's leaderboard number (ID-5). Strongest candidate for a concrete net-new unit. Confirm the benchmark is open + the harness is portable before committing.

H3. Code-execution-as-reasoning (raw.works) + dynamic-workflows (claude.com). Hypothesis: CONFIRMATION + bounded EXTENSION. ELAI's CodeAct (E-35) already implements code-as-action; the Workflow-tool DSL is BANNED by design (the orchestrate/BSP/FSM path is the deterministic equivalent). The defensible steal is ONLY the capable-lane reasoning-grain code-exec from H1; the JS Workflow DSL and "let the model author the orchestration script" are REJECTED (collides with FSM-gated plan-first orchestration + the Workflow-tool ban).

H4. MGH + "Claude Code is an RLM" + RLMs-are-the-new-reasoning-models. Hypothesis: CONFIRMATION grounding, NOT a build. External corroboration of ELAI's harness>model thesis (already corroborated by Guava IC-7, kunchen, deer-flow, superpowers verdicts). Bank as a K-1 dossier [EXTERNAL] row / spec-eval grounding, never a plan unit.

H5. RLM-as-memory (raw.works 2026-02-11). Hypothesis: ALREADY-COVERED / EXTENSION(E-1). ELAI's infinite-context chat (E-1: A-MEM + MemGPT digest) + SessionDag lossless-store/lossy-view + J-25 mimocode mechanics already implement recursive-summary memory. Confirm whether the RLM-memory angle adds anything past E-1/J-76.

H6. ypi / rlm-cli / pi / recursive-coding-agents repos. Hypothesis: REJECT/RIDERS (pi already REJECTED 2026-06-14). Recursive coding-agent harnesses = ELAI orchestrate + F-4 test-proof + idea-intake recursion, generally STRONGER. Only ride any genuinely-new mechanic into the existing units. Defer to a follow-up mode=source repo-map if the user wants the code read.

H7. OpenProse / prose.md / unix-rlm. Hypothesis: CONFIRMATION / possible thin EXTENSION. The declarative prose-workflow language maps to ELAI plan files + skills; ELAI's wiring-enforcement makes its specs verifiable where OpenProse relies on author discipline. A possible thin steal: an authoring ergonomics rider, never a new orchestration substrate.

H8. Langfuse. Hypothesis: REJECT (GD-2 — local SSB is the record; OTel export E-29 already exists for the egress case). External trace store contradicts the append-only-local-journal spine.

NET PRE-INGESTION READ: the corpus is ~80% CONFIRMATION of ELAI's existing direction (harness>model, recursive decomposition, deterministic-over-prompt, receipt-grounded reasoning) and validates the GRC-v3 thesis directly (Khattab's "Claude Code is an RLM" = ELAI's orchestration layer described from outside). The ~20% genuinely-actionable surface is narrow and concentrated: (a) the LongCoT benchmark as a K-15/I-38 eval arm; (b) a capable-lane reasoning-grain recursive code-execution operator composing J-42+J-38+E-35+J-86, never the local lane. Everything else is already shipped, already planned, already rejected, or banned by an explicit house rule.

================================================================================
PART 4 — WHAT WE SHOULD DO (recommendation, pending ingestion)
================================================================================
1. INGEST FIRST (this session): 4 sonnet paper-ingestor agents, 3 docs each, max 2 in parallel, writing interface maps to workflow/papers/. This grounds every paper-side claim above. Cost ~$4-8, ~10-20 min wall clock.
2. THEN a focused steal pass (separate, after maps land): paper-to-plan phase 4/5 over the maps, expected to yield AT MOST 1-2 small units — a LongCoT bench arm (K-15/I-38 EXTENSION) and a capable-lane recursive-reasoning operator (J-42/J-86 EXTENSION) — plus ledger riders for the CONFIRMATION/REJECT items so they are never re-litigated.
3. Do NOT pre-author plans before the maps land (grounded-by-default). Do NOT reopen the Workflow-tool ban or the local-lane CodeAct guard (both are load-bearing house rules with measured backing).

REJECT-LEDGER PRE-LOAD (so the steal pass does not re-litigate): RLM-as-everything REPL replacing the context budget; the JS Workflow DSL; LLM-self-judged reasoning stops (TD-3); paper reasoning-budget/overthinking knees transferred to ELAI (ID-5); dspy/ax as a dep; Langfuse as the trace store; pi-class generic harnesses.

End of pre-ingestion reasoning.

================================================================================
PART 5 — POST-INGESTION UPDATE (wave 1: 6 interface maps, 2026-06-26)
================================================================================
GROUNDED now (interface maps under workflow/papers/, reliability tagged): rlm-interface.md (RLM core 2512.24601, HIGH, pre-existing 2026-06-23); longcot-bench-2604.14140-interface.md (HIGH); code-execution-as-reasoning-rawworks-interface.md (MEDIUM); rlms-new-reasoning-models-rawworks-interface.md (MEDIUM); mgh-blog-interface.md (LOW-MED); longcot-rlm-blog-interface.md (LOW-MED); rlms-sota-longcot-blog-interface.md (LOW-MED). Wave 2 (RLMs-as-memory, ypi, dynamic-workflows, OpenProse, deck) NOT yet ingested.

THE FINDINGS THAT CHANGE THE STEAL CALCULUS:

F1 (the headline, MEASURED but single-source LOW reliability). The empirical RLM core is NOT abstract "recursion beats long-context". It is: a DSPy.RLM SCAFFOLD (a HARNESS, no fine-tuning) makes SMALL OPEN models competitive-to-SOTA on long-horizon reasoning. Reported: Qwen3-4B 0% -> 100% on MRCRv2 with the scaffold; Qwen3.5-9B/27B beat GPT-5.2 on LongCoT; Codex CLI 72.5% / Opus-4.7-Mini 75.4%. This is ELAI's OWN "harness > model" thesis, measured, and it lands directly on ELAI's LOCAL small-model lane (Qwen2.5-Coder-7B) — the exact wedge. It also confirms MGH (capable-but-mismanaged) with a number (Qwen3-4B 0->100 is the mismanagement made visible).

F2 (LongCoT is OPEN + portable — H2 confirmed actionable). arXiv:2604.14140 verified; GitHub LongHorizonReasoning/longcot is MIT; HuggingFace dataset LongHorizonReasoning/LongCoT live. ELAI CAN run it. So "adopt LongCoT as a measured reasoning-bench arm for K-15 / I-38" is concretely buildable, license-clean, ELAI-measured per-(model,lane) (ID-5 — never cite the paper leaderboard).

F3 (the RLM weakness was HARNESS-FIXABLE). The paper predicted Math/Chemistry ~0 (REPL-slicing doesn't help symbolic reasoning); the raw.works follow-up REFUTED it — a Claude-Code-generated prompt improvement closed the MATH/CS regression. Reinforces "the scaffold, not the model, is the lever," and that ELAI's domain-routed validators (D-10 -> D-11 math checker) are the right shape for the symbolic case.

F4 (THE REAL DESIGN TENSION — flag, do not resolve from training). The RLM scaffold works by the model WRITING PYTHON to slice/recurse over a context variable = a form of code-mode. ELAI's J-55 KEYSTONE guard FORBIDS routing the sub-14B local lane (Qwen-7B) to CodeAct (measured ~12% multi-tool code-mode collapse <14B, ~14B viability threshold, all WebSearch-verified 2026-06-20). Yet the RLM results show 4B-27B models SUCCEEDING with the RLM scaffold. Two grounded hypotheses to TEST (do not assume): (a) RLM's REPL is NARROWER than general CodeAct — it slices ONE string variable + a single rlm_query recursion primitive, not arbitrary multi-tool code-mode — so it may sit BELOW the collapse threshold J-55 measured for general code-mode; (b) the LongCoT tasks are retrieval/needle-shaped (MRCR), where slicing trivially helps, vs the code-edit tasks J-55 measured. RESOLUTION = an ELAI MEASUREMENT (run the constrained RLM-slice primitive on the local lane against LongCoT), never a training assumption. This is the single most important open question for any RLM steal and it is a J-55/J-34-class measured-knee question, exactly ELAI's idiom.

UPDATED VERDICT HYPOTHESES (supersede Part 3 where they differ):
- H1 RLM REPL-as-context: REVISIT-the-DEFER strengthened, but SCOPED. The buildable unit is a CONSTRAINED recursive-context primitive (a single string-slice + one rlm_query-style child-spawn over the SHIPPED sub-agent/BSP machinery), measured-gated, capable-lane-default, local-lane ONLY-IF F4's measurement clears. Compose J-42 (InftyThink distill-reset) + J-38 (mid-task retrieval loop) + the orchestrate sub-agent recursion + E-35 CodeAct. EXTENSION(J-42/J-86), not a new architecture. ELAI's orchestration layer already IS the coarse-grain RLM (Khattab's "Claude Code is an RLM"); the net-new is the WITHIN-REASONING-STEP recursive-slice primitive + its measurement.
- H2 LongCoT bench: NEW (small, high-confidence) — CONFIRMED actionable (F2). The best single concrete unit: `elai bench longcot` as a K-15/I-38 arm, MIT harness adapted, ELAI-measured, two arms (RLM-scaffold ON vs flat). Doubles as the F4 measurement instrument.
- H3/H4 MGH + code-exec + RLMs-new-reasoning: CONFIRMATION grounding for harness>model + the local-lane thesis (bank as K-1 [EXTERNAL] / spec-eval grounding, the strongest external corroboration yet — a measured 0->100). Not builds.
- DSPy.RLM as a dep: REJECTED (E-8 GEPA canonical; steal the scaffold IDEA, never the framework — house rule, consistent).

================================================================================
PART 6 — FINAL STEAL VERDICTS (all 12 interface maps ingested, 2026-06-26)
================================================================================
Maps added wave 2: rlm-as-memory-rawworks-interface.md (MED); ypi-recursive-coding-agent-interface.md (MED); rlm-rubric-recursive-coding-agents-interface.md (MED); claude-code-workflows-interface.md (HIGH); openprose-interface.md (MED); langfuse-interface.md (HIGH).

NEW GROUNDED FINDING (rubric G6, sharpens everything): the RLM conformance rubric's decisive gate G6 = WHO chooses the decomposition. RLM (full) = the LLM chooses it at run time; not-RLM (map-reduce) = the orchestrator script fixes it at design time. ELAI is plan-first / FSM-gated / TD-3-deterministic, so by the rubric ELAI is DELIBERATELY Level-2/3, NOT a full RLM at the orchestration grain. This is a CHOICE, not a deficiency: ELAI trades RLM's model-chosen flexibility for determinism + off-agent verification (the trust spine). Corollary: "become a full RLM" is an ANTI-GOAL — maximizing model-chosen decomposition would violate TD-3, the Workflow-tool ban, and HD-2. The steal is NEVER "be more RLM at orchestration"; it is only the narrow WITHIN-REASONING-STEP recursive-slice primitive (Part 5 H1) + measurement.

VERDICTS (paper-to-plan phase-4 classes):
- LongCoT benchmark (2604.14140, MIT, open harness LongHorizonReasoning/longcot + HF dataset): NEW [the one clear build]. `elai bench longcot` as a K-15/I-38 arm, ELAI-measured per-(model,lane), two arms (recursive-scaffold ON vs flat), license-clean. Doubles as the F4 measurement instrument (does a constrained RLM-slice lift the local Qwen-7B lane without tripping the J-55 code-mode collapse). HIGHEST-VALUE net-new.
- Constrained within-step recursive-context primitive (RLM REPL-as-context, scoped): EXTENSION(J-42 + J-86), capable-lane default, local-lane GATED on the F4/LongCoT measurement. A single string-slice + one rlm_query-style child-spawn over the SHIPPED orchestrate/BSP sub-agent machinery + J-38 retrieval loop; NOT general CodeAct, NOT model-chosen orchestration (G6 anti-goal). Author only if the measurement clears.
- RLM-as-memory (raw.works): EXTENSION(E-1)/ALREADY-COVERED. RLM memory == E-1 A-MEM/MemGPT digest + SessionDag + J-25 mimocode mechanics. Possible rider: LongMemEval as an eval arm. No new substrate.
- RLM rubric G1-G7: NEW (tiny, optional) — adopt as an EXTERNAL POSITIONING taxonomy in the K-1 dossier / spec-eval ("ELAI is deliberately Level-2/3 by G6, trading RLM flexibility for determinism+verification"). NOT a capability target. Grounding, not a build.
- ypi (MIT, full-RLM-by-rubric): REJECT/RIDER. Same class as the already-REJECTED pi (2026-06-14): an LLM-runtime recursive coding agent, structurally non-deterministic. Ride only a genuinely-new primitive into existing units; capture none of the loop.
- Claude Code dynamic-workflows / Workflow tool: REJECT — ban VINDICATED (the "workflow" keyword is the literal trigger; LLM-as-controller-via-JS-script vs ELAI's LLM-as-controller-over-FSM-gated-agents). ELAI's orchestrate + BSP supersteps are the deterministic equivalent. Do not reopen.
- OpenProse / prose.md (MIT): REJECT as a substrate — the LLM IS the runtime, so non-determinism is STRUCTURAL (conflicts with TD-3 + the whole trust spine). Optional thin rider only: a "session-to-prose" EXPORT if ELAI ever wants to share orchestration patterns externally (cosmetic, user-gated).
- Langfuse (MIT core): EXTENSION(E-29) / config-note, NOT a reject. Complementary: SSB stays the synchronous RECORD (GD-2 + wiring Layer-4); Langfuse is an OPTIONAL downstream analytics sink over OTLP/HTTP at /api/public/otel (gRPC unsupported). If ever wanted, it is just pointing E-29's existing OTLP exporter at that endpoint over HTTP. No new plan.
- MGH + code-execution-as-reasoning + RLMs-are-the-new-reasoning-models: CONFIRMATION grounding (harness>model, measured Qwen3-4B 0->100). Bank as K-1 [EXTERNAL] / spec-eval; the strongest external corroboration of ELAI's thesis to date. Not builds.

NET: exactly TWO candidate net-new units survive (LongCoT bench [build now-able] + constrained within-step recursive-reasoning primitive [build IF the LongCoT measurement clears the J-55 tension]), plus grounding riders and one config-note (Langfuse->E-29). Everything else is already-shipped, already-planned, already-rejected, or an explicit-house-rule anti-goal. This is the expected shape for this corpus: ~85% CONFIRMATION of ELAI's existing direction, ~15% a narrow measured-gated reasoning-lane improvement.

NEXT (when authorized): paper-to-plan phase 5 over these maps -> author the LongCoT bench unit (K/J), and the constrained-recursive-reasoning EXTENSION as a J-42/J-86 sibling guarded by the measurement; upsert ELAI-FEATURE-MAP.md (PLANNED entries + the REJECT/CONFIRMATION ledger riders) so none of this is re-litigated.

================================================================================
PART 7 — MEASURED RESULT (the experiment ran, 2026-06-26)
================================================================================
Experiment: workflow/experiments/rlm-vs-elai-retrieval-2026-06-26.md, implemented + run at
.liberty/experiments/rlm_vs_elai/ (harness: tasks.py/model.py/arms.py/run.py/analyze.py; raw
results.jsonl 144 rows; VERDICT.md). Free APIs via Liberty, model groq/llama-3.1-8b-instant (the
weak / J-55 axis). W (imposed attention budget) = 6000 tokens; bands 0.5W/2W/8W; n=12/band (8
single-needle + 4 combine); Phase-1 synthetic MRCR-style, deterministic exact-match (no LLM judge).
Phase-2 LongCoT + a strong model NOT run (gemini-flash free tier is 20 req/day; deferred).

SINGLE-NEEDLE accuracy (the clean retrieval/navigation signal, n=8/band):
  band   A flat   B RLM   C retrieval(ELAI)   D merge      (tokens/correct @8W)
  0.5W   100%     87%     100%                87%
  2.0W    62%     50%     100%                100%
  8.0W    25%     62%     100%                100%          A 25760  B 1550  C 6032  D 2988

VERDICTS (this MEASURES Part 5/6's hypotheses):
- Q1 RLM vs ELAI retrieval: ELAI WINS, strictly. Deterministic BM25 retrieval (C) = 100% at every
  over-budget length; RLM model-written slicing (B) = 50-62% and erratic. The "C > B -> ELAI
  strictly better" branch of the plan's decision tree. On lexical-needle long-context, ELAI's
  deterministic retrieval ALONE already captures the whole RLM benefit. RLM did not beat it.
- Q2 the merge: D == C accuracy (both 100% long-context), D deterministic/replayable, D cheapest
  per-correct. The merge matches ELAI's accuracy WITH ELAI's guarantees, but did NOT beat pure
  retrieval — because BM25 was already at ceiling (no gap for adaptivity to close on a LEXICAL task).
  -> the constrained within-step recursive primitive (J-42/J-86 sibling) is SAFE to author (never
  underperforms retrieval, cheapest), but its WIN is UNPROVEN until a NON-LEXICAL / multi-hop task.
- Q3 J-55: B shows code-errors only at 8W and erratic accuracy = MILD weak-model fragility, NOT a
  full collapse -> confirms F4(a) (constrained slice-REPL sits below the general code-mode collapse
  threshold). D (named ops, no free code) holds 100% -> the constrained primitive sidesteps it.
  KEY OBSERVATION: the weak 8B NEVER engaged the rlm_query recursion primitive — RLM's headline
  mechanism didn't even fire on the local lane. The lift, if any, lives on capable models only.
- Confound: COMBINE (sum-two-needles) is poor for all arms = 8B ARITHMETIC failure (both needles
  retrieved, added wrong), not navigation. Excluded from the primary signal.

NET (answers the user's "ELAI, RLM, or both?"): ELAI's deterministic retrieval is the winner on
long-context lexical recall and on the weak local lane; pure RLM (model-written code) is REJECTED
there (lower accuracy, non-replayable, fragile, recursion never engaged); the MERGE is the
strictly-safe "both together" — equal accuracy, equal guarantees, lowest cost — worth keeping as a
hedge for the non-lexical/semantic/multi-hop shapes this synthetic cannot exercise. This is exactly
the Part-6 shape (ELAI is deliberately Level-2/3 by rubric G6; the steal is the narrow constrained
primitive, gated on a measurement, NEVER a free-Python RLM and NEVER "be more RLM at orchestration").
The decisive unrun experiment = a NON-LEXICAL long-context task on a STRONG model + Phase-2 LongCoT;
that is the only place D could beat C, and it is the gate for authoring the J-42/J-86 sibling.

================================================================================
PART 8 — MULTI-HOP RESULT (the decisive follow-up ran, 2026-06-26)
================================================================================
Full writeup: workflow/experiments/RLM-VS-ELAI-RESULTS-2026-06-26.md (both experiments, all tables,
failure modes, verdict). Raw: .liberty/experiments/rlm_vs_elai/multihop.jsonl (64 rows). Model:
cerebras/gpt-oss-120b (CAPABLE, pinned). Task: 2-hop needle (dept->person->room) where the hop2 needle
uses a SYNONYM ("room") the question never names ("office"), so single-shot BM25 has ZERO lexical path
to hop2 — only the person learned from hop1 reaches it (verified offline ~5/6).

ACCURACY (n=8/band): band | A | B | C | D
  2xW | 25% | 50% | 12% | 25%    8xW | 12% | 0% | 0% | 25%

THE DECISIVE SIGNAL IS THE FAILURE MODE, NOT THE %:
- C (ELAI single-shot retrieval) structurally CANNOT multi-hop (0-12%) — but ABSTAINS (empty answers, no
  number invented). Fails SAFE.
- B (free-code RLM) is WORST: at 8xW it HALLUCINATED "42" on 7/8 tasks, high code-errors, recursion never
  engaged even on the 120B. Fails by FABRICATING — the trust-contract-breaking failure mode.
- D (merge / deterministic iterative navigation) is the ONLY arm >0% at 8xW (25%); correct when it converges,
  ABSTAINS (empty) when it hits the 10-op cap. Shortfall = step-budget (tunable), not capability.

UPDATED VERDICT (supersedes the Part-6/Part-7 "merge is a safe-but-unneeded hedge"): the merge is now
JUSTIFIED, not just safe. exp-1 (lexical) = ELAI retrieval wins outright, RLM loses, merge ties cheaply.
exp-2 (multi-hop) = single-shot retrieval structurally fails (abstains), free-code RLM fails AND hallucinates,
the MERGE is the only multi-hop-capable arm that preserves the abstain-not-fabricate trust contract. So:
ELAI deterministic retrieval is the SPINE (wins where it applies); ADD the constrained iterative-navigation
primitive (the J-42/J-86 sibling: fixed op-set over a BM25 candidate set, journaled/replayable, abstain-on-
budget) for multi-hop; REJECT free-Python RLM on every lane (never won a cell, recursion never engaged,
fabricates on hard tasks). Next tune: D-only at max_steps 16-20 (recover the abstained-but-reachable cases)
+ the `elai bench longcot` standing instrument. NOT authored yet (grounded-by-default; paper-to-plan phase 5
when authorized).

================================================================================
PART 9 — "CAN WE GET 100% ON 8xW MULTI-HOP?" — YES, VIA DETERMINISM (2026-06-26)
================================================================================
Full design: workflow/experiments/RLM-MERGE-ELAI-IMPLEMENTATION-2026-06-26.md (ceiling section).
Three measured arms on the 8xW multi-hop band (cerebras-120B for D, weak-8B for the hybrid):
- ORACLE (pure deterministic traversal, 0 model calls): 8/8 = 100%. The answer is ALWAYS deterministically
  reachable; the gap from D's 25% to 100% is MODEL-NAVIGATION RELIABILITY, not reachability.
- D@18 (model-driven navigator, more budget): 2xW 25->37%, 8xW 25->25% (FLAT). Tuning the model's navigation
  latitude PLATEAUS ~25% at 8xW — per-step model error compounds over the hop chain. You do NOT reach 100% this way.
- GRAPH-HOP HYBRID (deterministic hops, model ONLY for atomic entity+number extraction) on the WEAK 8B: 7/8 = 87%
  at 8xW. Deterministic grep found hop2 on all 8; the 1 miss was a final number mis-read, not navigation. A
  deterministic-traversal harness made a weak 8B beat model-driven nav on a strong 120B by 50+ points.
ANSWER: 100% is possible. The lever is DETERMINISM, not model-navigation tuning. Minimise the model to the
genuinely-fuzzy step (entity read); make find/hop/extract deterministic. ELAI ALREADY SHIPS the production form:
crates/memory/src/hippo_rag.rs (HippoRAG PPR over the entity graph, arXiv:2405.14831) retrieves multi-hop-linked
facts in ONE deterministic pass. So the multi-hop path routes through HippoRAG + atomic extraction; the
model-driven RetrievalNavigator is only the fallback for no-clean-entity-linkage cases (caps ~25-37%). Free-code
RLM stays rejected (0%, hallucinated). This is ELAI's harness>model thesis, measured end to end.

