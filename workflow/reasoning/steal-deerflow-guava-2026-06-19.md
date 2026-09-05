STEAL ANALYSIS: DeerFlow 2.0 (repo) + Guava (paper)
Date: 2026-06-19
Author: main LLM, in-band (paper-to-plan phase 4)
Sources:
- DeerFlow 2.0 repo: F:\.ELAI_workflow\docs\.source\deer-flow-main ; exploration: workflow/reasoning/deer-flow-exploration.md (opus, 224K tok)
- Guava paper: arXiv:2606.18363 "An Effective and Universal Harness for Embodied Manipulation" (Liu et al., 2026-06-16, cs.RO, preprint); interface map: workflow/papers/guava-interface.md (sonnet)
Verdicted against: docs/thefinalglance/ELAI-FEATURE-MAP.md (mtime 2026-06-18, FRESH) + its SECTION 4 REJECTED/ALREADY-TRIED LEDGER.

HEADLINE
DeerFlow 2.0 is a Claude-Code-style super-agent harness (lead agent + ~18-hook middleware stack + progressively-loaded skills + isolated sub-agent executor + LLM-driven memory; LangChain v1, MIT). ELAI is at or beyond it on every axis the exploration surfaced, and the feature map already pre-concluded this exact comparison class: the 2026-06-14 superpowers+context-rot ledger entry (FEATURE-MAP lines 1804-1843) closed the Claude-Code-harness comparison as "CONFIRMATION, zero net-new... highest-leverage = finish/commit H-50/H-51, not a new plan." DeerFlow reproduces that verdict. The Guava ROBOTICS paper, counterintuitively, yields the only net-new steals -- all in the local-lane TRAINING-DATA / MODEL-SELECTION corner, because that is the one area where a manipulation-distillation paper has hard data and ELAI's plans are happy-path-only.

CORRECTION SURFACED (grounding catch)
The deer-flow exploration (and the stale memory note project_efficiency_deepdive) frame S1 as fixing "ELAI's documented zero-cache-markers cost bug." That premise is STALE. FEATURE-MAP lines 938-940: PROMPT CACHING (CLOUD) [WIRED] apply_anthropic_cache_markers is LIVE on the production cloud path; the H-1 "split-brain / zero markers" belief was REFUTED and re-stamped at anthropic_api.rs:219. The only real remaining cache gap is the LOCAL lane (sidecar PrefixKVCache bypassed on the live streaming path, runtime_client.rs:547), already owned by the H-50 optional leg + J-19 vLLM automatic-prefix-caching. Do not author a cloud-cache steal.

DEER-FLOW STEAL VERDICTS (S1-S9 from the exploration)
S1 static-prefix + dynamic-context-as-<system-reminder> for prefix-cache reuse
  -> ALREADY-SHIPPED. Cloud markers WIRED (H-1/H-2 PARTIAL), CONTEXT BUDGET ALLOCATOR 8-slot WIRED with 2 xxhash cache breakpoints + per-model-static cache-safe layout (HD-2), PREFIX INVARIANCE + CACHE-HIT GUARD KEYSTONE (I-10). The <system-reminder> idiom == ELAI's stable-slot-ordering-with-volatile-last. Local KV reuse gap = H-50/J-19. Net new: 0.
S2 progressive metadata-only skill loading + deferred MCP tool_search
  -> ALREADY-SHIPPED/PARTIAL. TOOL GATING + LAZY MCP [WIRED] (C-35); TOOL SEARCH TOOL [PARTIAL] 77K->8.7K (H-4); MCP DEFER-LOADING [PARTIAL] (H-34); E-43 skill systems (metadata frontmatter + on-demand). Net new: 0.
S3 two-layer runtime loop detection (toolcall-hash + per-tool frequency, warn-then-hard-stop)
  -> MOSTLY ALREADY-SHIPPED. AGENT SYSTEM anti-spiral blake3 3-identical-hash stop [WIRED] (phase-3C) = the toolcall-hash layer; ANTI-SPIRAL PRIMITIVES 4 StagnationPattern variants fire unconditionally in drive_task (C-10); TEMPORAL TOOL-SEQUENCE ENFORCEMENT over tool-call history (I-35). The only conceivable sliver is the graduated warn-THEN-hard-stop on per-tool-type FREQUENCY as a distinct second tier; covered in spirit by Spinning/DiminishingReturns. Net new: 0 (at most a one-line rider on C-10/I-35, not worth a unit).
S4 summarization with skill-rescue (protect recently-loaded skills/tools from compaction)
  -> SPLIT: the summarization CORE is ALREADY-REJECTED -- LLM re-summarization compaction on the cache prefix is banned (HD-2, busts cache; deterministic masking only). The skill/tool-RESCUE idea is ALREADY-PLANNED: J-25 tool-pair boundary guard (un-orphan before ANY trim boundary) + section-aware budgeted read; H-6 masking drops trailing observations, never rewrites prefix. Net new: 0. (Mirrors the superpowers ledger verdict verbatim.)
S5 isolated sub-agent executor (cooperative-cancel-at-stream-boundary + token-attribution-to-dispatch-step)
  -> ALREADY-SHIPPED, STRONGER. TIER-C PROCESS ISOLATION [WIRED] = real OS processes + postcard IPC + seccomp (vs DeerFlow asyncio context isolation); streaming early-abort (H-3); token attribution = J-28 UsageTurnEvent + NormalizedUsage lane tag. Net new: 0.
S6 LLM-driven memory (confidence/dedupe/correction + reinforcement weighting)
  -> ALREADY-SHIPPED/PARTIAL, STRONGER. USER MEMORY SUBSTRATE Mem0 ADD/UPDATE/DELETE/NOOP dispatcher (E-5) == correction+dedupe; reinforcement weighting == J-21 activation-decay; plus poisoning defense (MINJA/J-21), bi-temporal supersession (H-31), auditor-objectivity firewall (zero memory to auditor/critic). DeerFlow has none of the hardening. Net new: 0.
S7 deep-research SKILL methodology (4-phase + diversity matrix + synthesis gate + temporal-precision table)
  -> ALREADY-SHIPPED (prompt-level). deep-research skill + researcher oracle-mode floors (>=5 search / >=10 fetch / >=3 subq) + source-tiers + EVIDENCE-RANKED SEARCH LADDER (I-4). The ONE marginal prose rider worth noting: a temporal-precision / date-anchoring table in the researcher prompt (aligns with B-42 code-freshness doctrine). Net new: 0 (optional one-line prose rider on .claude/agents/researcher.md, NOT a plan file).
S8 CLARIFY->PLAN->ACT clarification gate as run-halting middleware
  -> ALREADY-SHIPPED. IDEA-FRAMER + CRITIC LOOP FsmSubstate::Framing (B-40) + the CLARIFY chip on low confidence (FEATURE-MAP ~1154) + InteractiveGate/AutonomousGate + idea-intake skill. Net new: 0.
S9 dangling-tool-call recovery + safety-finish-reason suppression (multi-provider robustness)
  -> POSSIBLE TINY EXTENSION. Not explicitly named in the map. A defensive robustness patch on the provider/runtime_client layer (B-37 11-backend trait + I-49 protocol shapes): tolerate a truncated/dangling tool_call in a provider response and suppress a spurious safety finish_reason rather than crash. This is a hardening rider, not a feature. Verdict: EXTENSION(B-37/I-49) candidate, LOW value, only if provider-robustness bugs are observed live.
DEER-FLOW NET: 0 new plans. 1 low-value defensive rider candidate (S9). 1 optional prose rider (S7 temporal table). NOT worth stealing: the LangGraph/LangChain substrate, FastAPI gateway, Next.js UI, Docker/k8s sandbox provisioner, extra search providers, and DeerFlow's weaker self-discipline TodoMiddleware -- ELAI is far ahead on verification (off-agent auditor, wiring-enforcement, production-caller gate, freeze-the-gate); do NOT regress toward DeerFlow's in-boundary "mark complete when done" prompt.

GUAVA STEAL VERDICTS (robotics surface N/A; transfer via harness-design + small-model-distillation lens)
Domain-N/A (do not steal): grasp/align/move semantic tools, SAM3 segmentation, 6-DoF grasp planner, RGB-camera observations, sim-to-real, gripper control. ELAI has no robot.
G-confirm-1 iterative perception-reasoning-action loop > one-shot (Fig 2 left; 75.6 vs 62.7)
  -> CONFIRMATION of GRC observe/hypothesize/plan/act/verify + FSM + BSP. External grounding for the loop doctrine. Build: none.
G-confirm-2 semantic action abstractions > low-level (Fig 2 mid; App B)
  -> CONFIRMATION. ELAI named-skill/registered-tool dispatch + BASH INTERCEPTOR refuse-bare-grep->registered-tool (I-46) is exactly "semantic over raw." Build: none.
G-confirm-3 multimodal observations (Fig 2 right)
  -> CONFIRMATION (structured code-graph/DKG + textual plan/memory). Guava's RGB leg is robotics-only. Build: none.
*** G-NEW-1 (KEYSTONE) recovery-trajectory injection in the SFT set ***
  Guava: 743/1934 = 38.4% of training trajectories are failed-THEN-recovered (perturbation injected into a success trace, generation continued from the broken state, same filtering). SFT on success-only underperforms badly on long-horizon (Finding 3: shell-game SFT 6.7%; place-all-red SFT 0.0%). VERDICT: EXTENSION(I-30 VERIFIER-RANKED SFT EXHAUST). I-30 today = "winning paths from sealed RRAs -> SFT JSONL" = happy-path only. Guava is hard evidence that the exhaust must also mine failed-then-recovered execution traces (RRA runs that erred then self-corrected to a verified outcome) at a meaningful fraction, or the distilled local model is brittle on long-horizon orchestration. NOT a conflict with PRISTINE CONTEXT INJECTOR (I-5): I-5 excludes failed-sibling tracebacks at INFERENCE-context time; G-NEW-1 includes recovery traces at TRAINING-data time. Different stage. This is the one genuinely new, grounded, actionable steal.
G-NEW-2 (rider) targeted-RL-on-hard-tasks-only budget policy
  Guava: GRPO applied ONLY to 2 hard long-horizon tasks (each rollout = multimodal infer + sim step, too expensive full-suite); SFT already mastered easy tasks. RL lift only where it matters (place-all-red 0->93.3%). VERDICT: RIDER on I-31 GRPO FLYWHEEL (which is DEFERRED-LAST behind H-43/H-44/H-28+I-24, IC-7 SFT-before-GRPO). A design note on HOW to spend the RL budget when I-31 eventually fires: gate GRPO on a per-task bottleneck score, not the whole suite. Record in I-31; do not author now (I-31 is not active).
G-NEW-3 (rider/measurement) local-model tool-call-reliability floor
  Guava Table 3, identical harness: Gemini-3.1-Pro 89.8, Claude-Sonnet-4.6 88.9, GPT-5.4 70.2, Qwen3.5-2B 2.2 (hallucinates tool calls, cannot hold the structured format). Capability floor ~2-4B for tool-call adherence. ELAI's local lane = Qwen2.5-Coder-7B (primary) + 3B (alternate, NF4); the 3B sits ON the floor for tool-heavy roles. ELAI structurally MITIGATES the 2B failure mode the others can't: XGrammar tool-call mask FORCES valid JSON tool calls (sidecar LogitsProcessor) -- but HD-3 bans constrained decoding on REASONING/coder/planner roles (arXiv:2408.02442), so the grammar gate is correctly scoped to tool-call format only. VERDICT: CONFIRMATION of the 7B>3B>2B house ordering + the XGrammar-for-tool-format choice; optional measurement RIDER on J-34/B-10 ModelCapabilityProfile -- add a per-(local-model) tool-call-adherence bench (gate role-assignment: don't hand a tool-heavy role to a model below a measured adherence floor), analogous to J-34's context-fill knee. Grounded, but a small bench, not a feature.
G-confirm-4 small distilled model beats frontier under a good harness (4B 75.6 > GPT-5.4 70.2)
  -> CONFIRMATION of the ELAI local-lane / "smarter-orchestration-not-model" thesis (memory project_efficiency_trust_engineering). Strong external citation for the sidecar bet + SFT-on-rollouts ladder (I-30/I-31). Build: none.
G-confirm-5 SFT-before-GRPO is mandatory (two-stage; GRPO on RL-SFT init, not random)
  -> CONFIRMATION of IC-7 (already house doctrine; ledger 1465/1519-1520). Cite Guava as the empirical anchor. Build: none.
G-confirm-6 harness > end-to-end VLA (data-efficient <2K traj, modular, inspectable; VLA only wins on dexterity)
  -> CONFIRMATION of ELAI's harness-over-monolith stance. Build: none.

NET-NEW SHORTLIST (what is actually authorable)
1. [KEYSTONE, NEW] EXTENSION(I-30): mine failed-then-recovered execution traces into the SFT exhaust at a target fraction (~30-40%), not winning-paths-only. Grounded: Guava 2606.18363 sec 3.2 + App A.4 (38.4%), Finding 3. Touches: VERIFIER-RANKED SFT EXHAUST (I-30), RRA sealing, SAFEDELTA framing. One plan unit, or an amendment block on the I-30 file per the "new files only" house rule (-> a new J-5x file extending I-30).
2. [RIDER, record-not-author] I-31 GRPO budget = targeted-on-bottleneck-tasks. Grounded: Guava Finding 3. Record in I-31; I-31 is deferred-last, not active.
3. [RIDER/SMALL BENCH] J-34/B-10 local-model tool-call-adherence floor + role-assignment gate. Grounded: Guava Table 3. Optional.
4. [PROSE RIDER, not a plan] researcher.md temporal-precision/date-anchoring table (S7). Optional.
5. [DEFENSIVE RIDER, only if observed] B-37/I-49 dangling-tool-call recovery + safety-finish-reason suppression (S9). Low value.
Plus: Guava is strong EXTERNAL GROUNDING to CITE on IC-7, the local-lane thesis, and harness-over-VLA -- valuable as evidence even where it triggers no build.

RECOMMENDATION
Author exactly ONE new plan file: the I-30 recovery-trajectory-injection extension (item 1, the keystone), as a new numbered file in the J family extending I-30 (house rule: never amend old plan files in place). Record items 2-3 as ledger riders on I-31 and J-34. Skip 4-5 unless asked. Do NOT author any deer-flow cloud-cache / context-mgmt / memory / clarify steal -- all already-shipped or ledger-rejected; the correct deer-flow action is the standing one the superpowers ledger already names: finish/commit H-50/H-51, no new plan.

STATUS: assessment delivered; user chose "author keystone + the J-34 bench rider."
OUTCOME (2026-06-19): authored 2 NEW plan files (house rule: never amend old plan files in place):
- docs/thefinalglance/J-plan/J-50_recovery_trajectory_sft_exhaust.md (item 1 KEYSTONE; EXTENSION of I-30; reuse SftExhaustEvent +1 Kind RecoveryExtracted; depends I-30)
- docs/thefinalglance/J-plan/J-51_toolcall_adherence_floor_role_gate.md (item 3 bench rider; BUILD-on-shipped B-10/B-37/J-34; +ToolCallAdherenceMeasured)
Both = WAVE GUAVA-1, vertical-slice (producer+consumer+smoke) per wiring-enforcement PRE_GATE; SSB baseline 268/577 COMPUTED, re-grep at edit (W-9).
Index + map updates: J-0_INDEX.md (title 49->51 sessions, sessions table rows, constants block, GUAVA WAVE, Date + Source synthesis) ; J-SUMMARY_INDEX.md (title + 2 entries under GUAVA INTAKE) ; ELAI-FEATURE-MAP.md (J-50/J-51 PLANNED entries, I-31 targeted-RL rider, Section-4 deer-flow+Guava verdict block, header date bump).
Item 2 (targeted-RL): recorded as an I-31 ledger rider (not authored — I-31 deferred-LAST). Items 4-5 (S7 prose rider, S9 defensive rider): not authored per recommendation.
Deer-flow: 0 new plans (all S1-S9 already-shipped or ledger-rejected; correct standing action = finish/commit H-50/H-51). No commit (docs/ + workflow/ gitignored).
