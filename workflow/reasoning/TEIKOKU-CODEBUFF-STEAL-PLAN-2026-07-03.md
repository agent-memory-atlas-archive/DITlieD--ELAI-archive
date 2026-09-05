TEIKOKU-CODEBUFF-STEAL-PLAN

problem: codebuff/freebuff (C:\Users\user\Downloads\codebuff-main) claims 61% vs 53% Claude Code on its own evals using cheap open models coordinated by a multi-agent role split. Option A from the provider-analysis session: do NOT call codebuff as a provider (orchestrator-inside-orchestrator, paid SDK / ad-gated CLI, hands off our gate spine). Instead PORT the design worth taking into TEIKOKU's existing AOR roles + model pools. This file is the grounded steal analysis + phased plan. Authored 2026-07-03 by the main LLM (controller), read-only exploration of the downloaded repo, no code written yet.

STATUS: PLANNED, not started. Grounded against the codebuff repo (read this session) + the TEIKOKU handoffs (teikoku-build.md AOR entry, teikoku-native-dashboard.md, teikoku-gate-redesign.md). Implementation lands in the WSL distro ~/teikoku, one commit at the end per commit-cadence, SIGNAL/generality/wiring gates apply.

================================================================================
0. WHAT CODEBUFF ACTUALLY IS (grounded, with file cites)
================================================================================
The product is a single orchestrator agent "Buffy" (base2) that spawns specialized single-role subagents in parallel, sequenced by dependency, with per-model variants of each role and a routing table that picks the right variant for the selected model. Verified by reading:

- agents/base2/base2.ts (the createBase2 orchestrator factory). Modes default|free|lite|max|fast. Spawnable roster (buildArray): file-picker/file-picker-max, code-searcher, researcher-web, researcher-docs, basher, thinker, opus-agent, gpt-5-agent, editor/editor-multi-prompt, code-reviewer (+ per-model free reviewer), context-pruner, tmux-cli, browser-use. The systemPrompt encodes the WHOLE method: "gather context (file-pickers+code-searchers+researchers in PARALLEL) before editing -> read files -> optionally ask_user -> think -> editor implements -> code-reviewer + bashers (typecheck/test) in parallel -> fix -> concise summary".
- agents/editor/editor.ts (createCodeEditor). EDITOR_MODEL_BY_VARIANT maps a variant name to a concrete model id: gpt-5->openai/gpt-5.1, opus->anthropic/claude-opus-4.8, glm->z-ai/glm-5.1, kimi->moonshotai/kimi-k2.6, deepseek->deepseek/deepseek-v4-pro, minimax->minimax/minimax-m2.7. KEY FINDING: EDITOR_VARIANTS_WITH_THINK_TAGS = new Set(['opus']) -- only Opus gets <think>-tag scaffolding in its instructions; other variants "either have native reasoning (deepseek) or are non-reasoning models where the extra prose just bloats the prompt without helping." The cheap-model prompt is DIFFERENT from the frontier prompt, deliberately.
- agents/reviewer/code-reviewer.ts (createReviewer(model)) + 10 per-model reviewer files (code-reviewer-{kimi,glm,deepseek,deepseek-flash,minimax,minimax-m3,mimo,mimo-pro,lite,opus}.ts). Same review prompt, different model. "Nit Pick Nick": no tools, brief, advocate-for-the-user, requirements-coverage check, <think> before verdict.
- agents/thinker/thinker.ts (planner "Theo"): model claude-opus-4.8, no tools, sees full history, <think> deeply then writes a brief plan. Best-of-N variant at agents/thinker/best-of-n/.
- agents/context-pruner.ts (31KB): auto-spawned INLINE every step by base2's handleSteps generator with a per-model cap (getBase2ContextPrunerMaxContextLength: MiniMax 200k / Kimi 250k / others 400k) + cacheExpiryMs 30min. The user never manages context.
- common/src/constants/free-agents.ts + freebuff-models.ts: THE ROUTING TABLE. FREEBUFF_ROOT_AGENT_ID_BY_MODEL maps each model id -> a base2-free-<model> root; FREEBUFF_REVIEWER_AGENT_ID_BY_MODEL maps each model id -> its code-reviewer-<model>. FREE_COST_MODE allowlists only specific agent+model combos (abuse guard). Free providerOptions carry data_collection:'deny' (privacy posture baked in).
- evals/buffbench/ (BuffBench): the measured-number harness. README + judge.ts + run-buffbench.ts + runners/{claude,codebuff}.ts + eval-*.json task sets (manifold/plane/saleor + *-hard). Method: reconstruct real git commits from OSS repos; run N agents in parallel on the same task; 3 parallel GPT-5 judges take the MEDIAN; score completion / code-quality / overall each 0-10 (JudgingResultSchema). This is where 61% vs 53% comes from.

================================================================================
1. MAP: codebuff role -> TEIKOKU equivalent (already-have vs genuinely-new)
================================================================================
TEIKOKU today (from teikoku-build.md AOR entry, commit 95b27d0): 3 roles ORIENTER (fuses context-compiler/integration-explorer/scope-planner/synthesizer/impact-mapper/spec-grounder/prompt-engineer) + EXECUTOR + AUDITOR (sonnet, diff+verdict); orchestrator owns memory; teikoku-orient builds a Session Orientation Brief; model_pool.py pools = claude / devin-free (kimi/glm/swe via bridge :8810) / free-api (litellm :4000) / codex (gpt-5.5); conductor wave loop + gates + spawn caps + auto-migration (P4-D).

codebuff base2 orchestrator "Buffy"      -> TEIKOKU conductor + chat_orchestrator.   ALREADY HAVE.
codebuff file-picker/code-searcher/rsrch -> TEIKOKU ORIENTER (already fuses these).   ALREADY HAVE.
codebuff thinker (planner)               -> ORIENTER does scope-plan/synthesis.       PARTIAL (no dedicated think step).
codebuff editor                          -> TEIKOKU EXECUTOR.                          ALREADY HAVE (one prompt).
codebuff code-reviewer                   -> TEIKOKU AUDITOR.                            ALREADY HAVE (one prompt).
codebuff context-pruner                  -> TEIKOKU auto-migration (P4-D) / H-50/51.   PARTIAL (migrate not prune-every-step).
codebuff opus/gpt-5 escalation agents    -> TEIKOKU pools (claude/codex).              HAVE pools, no in-loop escalation.
codebuff BuffBench eval harness          -> NOTHING. TEIKOKU has no measured number.   MISSING (highest value).

The clean read: TEIKOKU already has the ORCHESTRATOR and the ROLE SPLIT and the CHEAP POOLS. codebuff does not teach us those. What codebuff teaches that TEIKOKU lacks is (a) per-model role tuning + a routing table, (b) a reproducible eval that produces a believable number, (c) two mechanisms (best-of-N, context-prune-every-step). Everything else is confirmation, not invention.

================================================================================
2. THE RANKED STEAL LIST (grounded; value / effort each)
================================================================================
S1 [KEYSTONE] BuffBench-style eval harness -> TEIKOKU "git-commit reconstruction" bench.
  WHY: the market/GTM memory says over and over ELAI needs "a real measured number before release" (project_market_positioning_2026, project_gtm_playbook_2026 "believe-the-number"). codebuff HAS one and it is portable: pick real commits from an OSS repo, strip them, have the agent reconstruct, judge diff-vs-ground-truth with 3 median LLM judges on completion/quality/overall. This is the SAFEDELTA-shaped harness TEIKOKU keeps deferring.
  GROUNDING: evals/buffbench/{run-buffbench.ts,judge.ts,agent-runner.ts,gen-evals.ts}, JudgingResultSchema (completion/codeQuality/overall 0-10), runners/{claude,codebuff}.ts (head-to-head).
  TEIKOKU SHAPE: a conductor bench mode that drives the SAME base-cc entrypoint per pool (claude vs free-api vs devin) on a task set, judges with a claude or codex judge (median-of-3), emits a per-pool score to the SIGNAL dashboard. Reuses the pack-aware conductor + project_resolver; the "agent" under test is a TEIKOKU pool, not codebuff. VALUE: very high. EFFORT: medium-high (the judge + task-gen + isolated-repo runner).

S2 [HIGH] Per-model role tuning + a routing table.
  WHY: TEIKOKU runs ONE AOR prompt set across every pool. codebuff's grounded finding is that cheap non-reasoning models need a DIFFERENT prompt than Opus (EDITOR_VARIANTS_WITH_THINK_TAGS = opus-only; drop <think> scaffolding for kimi/glm/minimax because it bloats without helping). Porting: EXECUTOR + AUDITOR get a per-pool prompt variant selected by a routing map keyed on the active model, exactly like FREEBUFF_REVIEWER_AGENT_ID_BY_MODEL. TEIKOKU already has model_pool.py to hang this off.
  GROUNDING: agents/editor/editor.ts (EDITOR_MODEL_BY_VARIANT + think-tag set), common/.../free-agents.ts (the *_BY_MODEL maps).
  TEIKOKU SHAPE: model_pool.py gains role_prompt_variant(model) -> the ORIENTER/EXECUTOR/AUDITOR prompt profile; a reasoning-model set decides <think> scaffolding on/off. VALUE: high (directly targets cheap-model quality, the whole point). EFFORT: low-medium.

S3 [HIGH] The explicit parallel context-gather-then-act pipeline as the free-lane default.
  WHY: codebuff's method (spawn MANY file-pickers/searchers/researchers in PARALLEL first, read, THEN one editor, THEN reviewer+bashers in parallel) is the thing their eval credits for beating single-model CC. TEIKOKU's AOR already sequences ORIENTER->EXECUTOR->AUDITOR but the "fan out context-gatherers in parallel before any edit, cap by dependency" discipline is worth encoding as the free-lane orchestration profile (cheap models especially need pre-gathered context because they have weaker recall).
  GROUNDING: agents/base2/base2.ts systemPrompt "Spawning agents guidelines" + the response <example>.
  TEIKOKU SHAPE: a chat_orchestrator profile for the free/devin pools that front-loads parallel ORIENTER fan-out (respecting the existing depth=2/fanout=4 caps + $10 ceiling) before EXECUTOR. VALUE: high. EFFORT: low (prompt/profile, machinery exists).

S4 [MED] Best-of-N for the hard-problem path.
  WHY: max mode runs editor-multi-prompt + thinker-best-of-n (generate N implementations/plans, a selector picks best). This is a known quality lever for hard tasks and composes with TEIKOKU's existing spawn/fanout.
  GROUNDING: agents/editor/best-of-n/{editor-multi-prompt,best-of-n-selector2}.ts, agents/thinker/best-of-n/.
  TEIKOKU SHAPE: an opt-in "max" objective flag that fans EXECUTOR to N candidates + a selector pass (claude or codex as selector). Gate: only on the paid/claude lane or when the user opts in (cost). VALUE: medium. EFFORT: medium.

S5 [MED] Context-prune-every-step with a per-model cap.
  WHY: TEIKOKU has auto-MIGRATION at a threshold (P4-D) but not codebuff's continuous inline prune with a per-model window cap (200k/250k/400k). For a long cheap-model session the continuous prune keeps quality up without a hard migrate. Overlaps H-50/H-51 (context-lifecycle) already planned; this is a concrete mechanism for those.
  GROUNDING: agents/context-pruner.ts + base2.ts getBase2ContextPrunerMaxContextLength + handleStepsFree200k generator.
  TEIKOKU SHAPE: fold into the H-50 regime-aware context policy as the LOCAL/cheap-pool prune step; per-model cap sourced from model_pool windows (already there). VALUE: medium. EFFORT: medium (but mostly already scoped in H-50/51).

S6 [LOW] Privacy posture baked into the pool config (data_collection:'deny' on free providers).
  WHY: codebuff sets data_collection:'deny' in providerOptions for free lanes. TEIKOKU's litellm proxy could carry the equivalent per-pool flag; cheap free providers are the ones most likely to train on data. Small, aligns with the trust spine.
  GROUNDING: agents/base2/base2.ts defaultProviderOptions.
  VALUE: low-medium (trust posture). EFFORT: trivial (proxy config).

================================================================================
3. WHAT NOT TO STEAL (attack-the-design, grounded)
================================================================================
- The codebuff tool protocol (<codebuff_tool_call> / str_replace JSON envelopes, agents/editor/editor.ts). TEIKOKU workers are base-cc and use base-cc's native tools. Porting a second tool protocol is pure liability. SKIP.
- The transport (OpenRouter / Bedrock / Fireworks routing). TEIKOKU has its own litellm proxy :4000 + devin bridge :8810. Keep TEIKOKU's transport; only the ROUTING-TABLE IDEA transfers, not the endpoints.
- ads / credits / waiting-room / gravity_index (the ad-driven service recommender). Not applicable; SKIP entirely.
- The SDK-as-provider path (rejected in the prior session): paid CODEBUFF_API_KEY + their backend + orchestrator-inside-orchestrator. This plan is Option A precisely to avoid it.
- Model IDs verbatim (gpt-5.1, claude-opus-4.8, minimax-m2.7 etc.) -- these are codebuff's account's models; TEIKOKU maps to its OWN pool ids (kimi-k2-7/glm-5-2/swe-1-6 on the bridge, the litellm tiers). Steal the STRUCTURE (a table), not the values.

================================================================================
4. PHASED PLAN (TEIKOKU conventions: pack-aware, generality-clean, wiring-enforced, SIGNAL, one commit at end)
================================================================================
Sequencing rationale: S1 (the bench) FIRST, because without a measured number every later "did it help" claim is ungrounded -- the whole point of stealing from codebuff is that they MEASURED. Then S2/S3 (the cheap-model quality levers the bench can score), then S4/S5 (heavier, optional), S6 (trivial, fold in anywhere).

PHASE C1 - THE BENCH (S1). Build conductor/bench/ : task set from real OSS commits (gen-evals analog, pack-aware via project_resolver), an isolated-repo runner that drives the REAL base-cc entrypoint per pool, a median-of-3 judge (claude or codex judge, JudgingResultSchema shape: completion/quality/overall). Emit per-pool scores to a SIGNAL bench surface. DONE-CRITERION: one head-to-head run (claude pool vs free-api pool vs devin pool) on a >=5-task set produces a reproducible per-pool overall score, judged off the produced diff, shown in the dashboard. This is the number ELAI keeps saying it needs. WIRING: the runner drives the real POST /chat/turn or headless entrypoint (not a shim); the score row is a real artifact the agent did not author (R4). GENERALITY: task set is pack-parameterized, runs on a non-ELAI demo repo.

PHASE C2 - PER-MODEL ROLE TUNING (S2) + FREE-LANE PARALLEL-GATHER PROFILE (S3). model_pool.py gains role_prompt_variant(model); EXECUTOR/AUDITOR/ORIENTER prompts get a cheap-model variant (drop <think> scaffolding for the non-reasoning set, front-load parallel context gather for the free/devin profile). DONE-CRITERION: the C1 bench shows a measurable score delta for the free/devin pools with the tuned profile vs the flat profile (the bench PROVES the steal helped, or shows it didn't -- either is a grounded result). WIRING: variant selection flows model_pool -> chat_orchestrator -> the worker prompt (grep-provable chain).

PHASE C3 - OPTIONAL LEVERS (S4 best-of-N, S5 continuous prune, S6 privacy flag). Each gated + opt-in; S5 folds into the H-50/H-51 context-lifecycle plan rather than a fresh mechanism. DONE-CRITERION per lever: bench score delta measured; cost delta reported honestly (best-of-N is N-times the tokens). Ship only the levers the bench says earn their cost.

================================================================================
5. RISKS / OPEN QUESTIONS / DONE-CRITERIA
================================================================================
RISK-1: the judge is an LLM -> judge bias. codebuff mitigates with 3 parallel judges + median. Port that; additionally consider a deterministic leg (does the reconstructed diff make the repo's own tests pass) so the number is not judge-only. This matches TEIKOKU's off-agent-artifact doctrine (wiring R4).
RISK-2: "61% vs 53%" is codebuff's OWN eval on tasks THEY picked -> not neutral. Do NOT quote their number as ours. Our bench measures OUR pools on OUR task set; the only honest claim is our own reproducible delta.
RISK-3: cheap-model role tuning can regress as models change (kimi-k2.6 today, k2.7 tomorrow). The routing table must be data-driven (re-run the bench per model bump), not a frozen guess.
OPEN-Q1 (owner): judge model -- claude (costs plan tokens) vs codex gpt-5.5 (account-a pool, 4248 credits) vs a free model (cheap but weaker judge)? Lean: codex gpt-5.5 for the judge (strong, off-plan-budget), free model as a deterministic-test backstop.
OPEN-Q2 (owner): scope of first task set -- reuse codebuff's eval-*.json commit sets (manifold/plane/saleor, already in evals/buffbench/) as a starting corpus, or generate fresh from an ELAI-adjacent repo? Lean: start from their JSON sets to get a number fast, then add TEIKOKU-relevant tasks.
OPEN-Q3: does S2/S3 belong inside the existing AOR prompts or a new free-lane orchestration profile? Lean: a profile selected by pool, so the frontier path is untouched.

OVERALL DONE-CRITERION: TEIKOKU can, on demand, produce a reproducible per-pool coding-quality number on a git-reconstruction bench, and the cheap-pool prompt tuning is chosen BY that number rather than by vibes. That -- not calling codebuff -- is the whole value of this exercise.

================================================================================
6. GROUNDING LEDGER (what was read this session, all in C:\Users\user\Downloads\codebuff-main)
================================================================================
VERIFIED (read in full or head):
  README.md (freebuff FAQ: ads, models, limited mode), freebuff/SPEC.md (compile-time IS_FREEBUFF strip), freebuff/e2e/tests/ads-behavior.e2e.test.ts.
  agents/base2/base2.ts (orchestrator factory, full systemPrompt + spawn roster + handleSteps), agents/editor/editor.ts (variant->model map + think-tag finding), agents/reviewer/code-reviewer.ts (review prompt), agents/thinker/thinker.ts (planner), agents/ file tree (all roles + best-of-n + per-model variants).
  common/src/constants/freebuff-models.ts + free-agents.ts (the routing tables + free-cost allowlist + session limits).
  sdk/README.md + sdk/src/client.ts + env.ts (paid API key + backend + BYOK env vars -- grounds the "SDK is not free" NOT-STEAL).
  evals/buffbench/README.md + judge.ts (JudgingResultSchema, 3-median-GPT-5 judge) + the eval-*.json task sets + runners/{claude,codebuff}.ts (head-to-head).
TEIKOKU side (from handoffs, this session): teikoku-build.md (AOR 3-role model, model_pool pools, conductor/gates/migration, P4-D), teikoku-native-dashboard.md (SIGNAL law, provider lanes), teikoku-gate-redesign.md (gate spine).
UNVERIFIED / deferred: buffbench's actual run numbers (README asset image not read); whether the SDK BYOK path zeroes codebuff credits (billing semantics, a their-pricing question, not in-repo); the exact ELAI/TEIKOKU H-50/H-51 context-lifecycle overlap detail (cross-ref only).

NOTE: this plan is Option A (port the design). It supersedes nothing; TEIKOKU implementation happens in the WSL distro ~/teikoku when the owner says go. No code written yet.
