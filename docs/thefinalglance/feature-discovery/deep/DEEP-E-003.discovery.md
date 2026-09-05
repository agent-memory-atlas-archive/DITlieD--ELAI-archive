# DEEP-E-003 feature discovery

## Scope, method, and classification

This is a source-faithful deep read of exactly the DEEP-E-003 manifest assignment. The nine assigned originals were read in full. The family summary 05-E-SUMMARY_INDEX.discovery.md was consulted only to orient the read and to calculate the summary deltas below. No other original source chunk was mined.

Atomic-ledger IDs have the form E003-<source>-NNN. One ID denotes one deduplicated behavior or contract even when the source repeats it in Tasks, WIRING_CONTRACT, ACCEPTANCE, and Tests. Repeated statements are represented by multiple line anchors on the same atomic item, not by duplicate items.

Classification tags:

- EXP = explicit source requirement or statement.
- INF = inference needed to compose explicit requirements; not directly promised.
- AMB = ambiguity internal to the source.
- CON = direct conflict, either within the source or between the source and the orientation summary.
- CAP, WF, DATA, CFG, INT, SEC, FAIL, DET, OBS, TEST, CONSTR, DEP, STATUS identify the requested feature dimensions.

## Exact processed manifest and verification

Manifest section: DEEP-SOURCE-MANIFEST.md lines 545-559. Declared total: 9 files, 2,141 logical lines, 249,792 bytes; target status within 1,800-2,800. Every row matched the live file exactly on logical line count, byte count, and SHA-256.

| Chunk | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-E-003 | numbered/index original | 230 | 27,509 | 35c3e70126542558e72c52d6de18980d95073c01194f99a20c7b110b479f7d59 | docs/thefinalglance/E-plan/E-4_chat_ux_critique.md | exact |
| DEEP-E-003 | numbered/index original | 254 | 23,436 | f6131b6454c00aa21a4a29fbb9d57fddbeba62b2aad0dfa993c3c457f07fd5dc | docs/thefinalglance/E-plan/E-9_agent_eval_benchmarks.md | exact |
| DEEP-E-003 | numbered/index original | 193 | 22,550 | f7d9b36475c890d2e9f36c15d3d0645f707bf7d8a5badf6c396784a96e5afa5f | docs/thefinalglance/E-plan/E-12_production_fullstack_codegen.md | exact |
| DEEP-E-003 | numbered/index original | 207 | 20,485 | 383ee956b5f6a53422c550ce709f5bd28fa487ab7250cdd9a8e97df0229b92f3 | docs/thefinalglance/E-plan/E-18_tool_use_reliability.md | exact |
| DEEP-E-003 | numbered/index original | 265 | 38,777 | 09f90367279bf56ab56f617e3572e2b0f8e25f9f88dd21f34dac3a448cce0627 | docs/thefinalglance/E-plan/E-26_inference_engines_2026.md | exact |
| DEEP-E-003 | numbered/index original | 195 | 20,236 | 06fcc374e1a364489477ec826f5411ac2023939851cc52a75a79840cd1e08079 | docs/thefinalglance/E-plan/E-27_quantization_2026.md | exact |
| DEEP-E-003 | numbered/index original | 214 | 23,462 | b9421eab36e28a8343ea1d052165451255d7ef2b33e3e61639fcb671bc12c58c | docs/thefinalglance/E-plan/E-32_context_window_extension.md | exact |
| DEEP-E-003 | numbered/index original | 237 | 29,056 | e98765d11070553619d9951d1e0eabe4e5247b54a1c4f5974b7902e6f680c6b7 | docs/thefinalglance/E-plan/E-44_checkpointing_durability.md | exact |
| DEEP-E-003 | numbered/index original | 346 | 44,281 | 7af1c532f18884142e4a5fd9572666b428bbec5078a2665481c94e8c657d9557 | docs/thefinalglance/E-plan/E-54_eval_corpus_design.md | exact |
| **Total** |  | **2,141** | **249,792** |  | **9 files** | **exact** |

## Per-file ledger: E-4_chat_ux_critique.md

Source anchors below refer to E-4_chat_ux_critique.md.

### Explicit atomic ledger

- [E003-E4-001][EXP][CAP][lines 1-16] CommandWindow is the trust-forming surface; the plan groups bubble affordances, session rails, and hard model-output rules and targets confidence, sycophancy, instruction retention, re-anchoring, doom loops, evidence-backed completion, clarification, interrupt, handoff, and intervention.
- [E003-E4-002][EXP][DEP/STATUS][lines 3-6,18-29,228-230] Effort is three sessions; B-14/B-22/B-39/B-41 and E-2/E-3 are prerequisites, E-1/E-5 are parallel dependencies, B-38 is a soft later replacement, C-21 supplies rules.db, and D-14/D-23 measure the guard.
- [E003-E4-003][EXP][DATA][line 55] ClaimWithConfidence has text, tier in Verified|ReadFromSource|FromTraining|Guessed, and optional EvidenceRef.
- [E003-E4-004][EXP][WF/DET][line 55] ChatReflectPipeline AUDITOR emits claim wrappers; passed FILE_CHECK/SOURCE_CHECK/TEST_CHECK/WEB_CHECK maps to Verified or ReadFromSource, no grounding maps to FromTraining, and a low-confidence mcp_think status echo maps to Guessed.
- [E003-E4-005][EXP][INT/OBS][lines 28,55,86,96,103,146] CommandWindow renders a pill before every factual claim, and E-1 memory_notes can persist the claim as fact_type=claim with confidence_tier and provenance.
- [E003-E4-006][EXP][SEC/WF][line 57] SycophancyGuard amends the AUDITOR prompt: retain a cited position when evidence contradicts the user, state lack of evidence explicitly, and never agree merely to be agreeable.
- [E003-E4-007][EXP][DET/CONSTR][lines 57,109,220,226] Pushback is permitted only with Verified-tier evidence; user thanks is not a reward; a skipped pushback opportunity requires no_evidence_for_pushback evidence; the helper coexists with AUDITOR REFINE cap=2.
- [E003-E4-008][EXP][OBS][lines 57,91,113,120,147] An AUDITOR override of REASONER stance emits sycophancy_pushback_detected and is acceptance-tested with a planted false claim.
- [E003-E4-009][EXP][CAP/DATA][line 59] PinnedRulesPanel.svelte reads rules.db ACTIVE rows plus session ad-hoc constraints and renders them in the CommandWindow sidebar.
- [E003-E4-010][EXP][WF/INT/OBS][lines 59,91,148] The panel is collapsed by default, expands on demand, pins via user_rule_pinned, and click-to-remove emits user_rule_unpinned.
- [E003-E4-011][EXP][CFG/WF][line 61] Session re-anchor runs after N turns, default 10 and configurable, or before commit, force-push, or file-delete.
- [E003-E4-012][EXP][DATA/OBS][lines 61,81,91,149] Re-anchor re-injects the system prompt, first user message, and active rules, emits session_re_anchored, exposes cadence in SettingsWindow, and shows a toast.
- [E003-E4-013][EXP][DET/WF][line 63] DoomLoopDetector fires on any of four conditions: three same-file edits for one issue, three failed mcp_think hypotheses, two consecutive auditor BLOCKs, or a plateauing test-pass rate.
- [E003-E4-014][EXP][FAIL/INT][lines 63,89,102,150] On doom-loop detection the runner freezes, emits doom_loop_detected, and shows attempt diffs plus Switch tactic, Hand back to user, and Continue anyway controls.
- [E003-E4-015][EXP][CFG/OBS][lines 81,221] Doom-loop thresholds are configurable; dismissing the detector emits doom_loop_threshold_adjusted because legitimate refactors may otherwise false-positive.
- [E003-E4-016][EXP][SEC/WF][line 65] Completion phrases including done, complete, finished, and successfully implemented require at least one file diff, audit-verdict/test row, or deterministic-check pass.
- [E003-E4-017][EXP][FAIL/OBS/INT][lines 65,91,108,151] CompletionClaimCheck rejects an ungrounded completion in AUDITOR, emits completion_claim_rejected_no_evidence, and renders an evidence-link cluster below an accepted claim.
- [E003-E4-018][EXP][DET/CFG][lines 24,67,81] The initial ambiguity heuristic is task length under 50 characters plus missing action verb plus no target file/scope; B-38’s continuous ambiguity score replaces it when shipped.
- [E003-E4-019][EXP][WF/INT/OBS][lines 67,88,100,152] An ambiguous task produces exactly one clarifier card with at most three yes/no questions; after one response it proceeds without another round and emits one_shot_clarifier_emitted/resolved.
- [E003-E4-020][EXP][CAP/CFG][lines 69,92,126-127,153] Default output is action plus one-line why plus one-line risk; /verbose applies long form for the next N turns or until /terse.
- [E003-E4-021][EXP][DATA/CFG][lines 69,71,186,225] agents/voice.toml gains a verbosity section; fact_type=expertise is read before TRIAGE, is per-language on a five-stage Dreyfus scale, calibrates jargon/terserness, and defaults to competent until E-5 rows exist.
- [E003-E4-022][EXP][SEC/INT/OBS][line 73] A local-codebase refusal logs refusal_emitted and offers an Override button that bypasses the rule for one dispatch and logs refusal_overridden_by_user.
- [E003-E4-023][EXP][SEC/CONSTR][lines 73,81,223] Actions outside .elai_cc are hard-refused without override; the decision doc must distinguish that boundary from overrideable local anti-pattern rules.
- [E003-E4-024][EXP][WF/INT/OBS][lines 75,96,154] Entering a correction during streaming invokes the existing cancel_stream path, prepends the correction, restarts with corrected context, and emits mid_stream_interrupt_committed.
- [E003-E4-025][EXP][FAIL][lines 224-225] AnthropicApi and Sidecar cancel paths are known-good; CLI-subprocess backends may require kill(), so all eleven RuntimeAdapter cancel paths must be verified.
- [E003-E4-026][EXP][INT/OBS][line 77] A discovered error renders a styled correction beginning “I was wrong. Correct: X because Y.” and emits honest_correction_emitted.
- [E003-E4-027][EXP][INT/WF][line 79] A permanently visible ActivityWindow control bar during EXECUTING, AUDITING, and RESEARCHING wires /pause, /redirect, and /cancel to existing FSM/SSB intervention paths.
- [E003-E4-028][EXP][DATA][line 81] workflow/reasoning/E-4-decisions.md records confidence authority, cadence 10, doom threshold 3, heuristic-to-B-38 migration, and refusal scope.
- [E003-E4-029][EXP][INT][lines 85-98] Produced surfaces are ClaimWithConfidence, ConfidenceTier, SycophancyGuard, OneShotClarifier, DoomLoopDetector, PinnedRulesPanel, and the CommandWindow/ActivityWindow/CommandPalette integrations.
- [E003-E4-030][EXP][OBS][lines 91-92] The exact declared SsbActionType set is ClaimConfidenceTierEmitted, SycophancyPushbackDetected, UserRulePinned, UserRuleUnpinned, SessionReAnchored, DoomLoopDetected, CompletionClaimRejectedNoEvidence, OneShotClarifierEmitted, OneShotClarifierResolved, RefusalEmitted, RefusalOverriddenByUser, MidStreamInterruptCommitted, and HonestCorrectionEmitted.
- [E003-E4-031][EXP][CFG/INT][line 92] Five slash commands (/verbose, /terse, /pause, /redirect, /cancel) raise the declared slash-command count from 146 to 151.
- [E003-E4-032][EXP][WF][lines 99-103] Reachability is chat→TRIAGE→clarifier, AUDITOR→sycophancy/completion checks, baby-step boundary→doom-loop detector, and CommandWindow→claim/override/correction rendering.
- [E003-E4-033][EXP][CONSTR][lines 104-110] Stub bans forbid placeholder evidence, always-agree logic, unimplemented/todo code, bare completion output, and silent skipping of a pushback opportunity.
- [E003-E4-034][EXP][OBS/TEST][lines 111-114] The smoke scenario uses a nonexistent crate, mid-stream redirection, and a bogus completion; expected evidence is pushback, optional clarification, interrupt, completion rejection, a Verified chip, and no bare done.
- [E003-E4-035][EXP][TEST][lines 118-141] Deterministic gates check all event variants, panel existence/mount, command count, claim schema, doom-loop event, completion-rejection event, and absence of unwrapped bare-done output.
- [E003-E4-036][EXP][TEST][lines 146-152] Acceptance requires chips on every factual AUDITOR claim, false-claim pushback, complete active-rule rendering/removal, N=10 re-anchor, doom detection within three edits, evidence on every completion, and at-most-once clarification.
- [E003-E4-037][EXP][TEST][lines 153-156] Acceptance also requires all five commands in CommandPalette, cancel/restart interrupt semantics, paired override affordance for every refusal row, and correction rendering.
- [E003-E4-038][EXP][CONSTR/TEST][line 157] No dependency may be added; new UI must use Svelte 5 runes with no reactive-label syntax and no createEventDispatcher.
- [E003-E4-039][EXP][TEST][lines 161-165] Reasoning/orchestrator/contract tests cover held-position sycophancy, exactly-once clarification, evidence-required completion, three-edit doom threshold, and ClaimWithConfidence serde/SSB round-trip.
- [E003-E4-040][EXP][TEST][lines 166-168] UI/contract tests cover rule unpin, stream cancel/restart with prepended correction, and refusal→override event pairing.
- [E003-E4-041][EXP][STATUS][lines 170-186] Four dashboard components, doom_loop.rs, and a decisions doc are created; chat_reflect, task_runner, ssb, fsm, CommandWindow, ActivityWindow, CommandPalette, live_cli, and voice.toml are modified.
- [E003-E4-042][EXP][FAIL][lines 220-226] Named risks are sycophancy overcorrection, eager doom detection, brittle ambiguity heuristics, override attack surface, inconsistent adapter cancellation, missing expertise rows, and pushback without Verified evidence.
- [E003-E4-043][EXP][STATUS/DEFERRED][line 230] Persona switching among Hacker/Researcher/Simplifier/Architect/Contrarian is a follow-up after telemetry, not part of this plan.

### Inferred, ambiguous, and conflicting

- [E003-E4-044][INF][WF][lines 55-77] Claim confidence, honest correction, and completion evidence likely share one bubble payload, but the source never specifies one combined transaction or serialization type.
- [E003-E4-045][AMB][STATUS][lines 2,16,53-81] The objective calls out report ranks #1-#8 and the context says 14 recommendations, while the implementation enumerates 14 tasks whose relationship to those ranks is not one-to-one.
- [E003-E4-046][CON][OBS][lines 57,91] The original event is sycophancy_pushback_detected/SycophancyPushbackDetected, not the orientation summary’s sycophancy_detected; the original has no persona_drift_alert event or explicit ten-turn drift detector.
- [E003-E4-047][CON][CAP][lines 61,77,218-226] Persona drift and repair resistance motivate re-anchor/correction, but the source does not explicitly implement the summary’s “alert on persona drift” or “count repair resistance” capabilities.

## Per-file ledger: E-9_agent_eval_benchmarks.md

Source anchors below refer to E-9_agent_eval_benchmarks.md.

### Explicit atomic ledger

- [E003-E9-001][EXP][CAP][lines 1-10] The suite anchors internal D-19..D-25 measures to contamination-resistant 2026 public benchmarks and makes harness-vs-model decomposition mandatory beside scores.
- [E003-E9-002][EXP][DEP/STATUS][lines 3-5,12-23] Effort is three sessions; B-37 is required, B-23 optional, D-22 supplies calibration, D-19..D-25 are extended, and .elai_cc/benches/external plus paired-bootstrap gating are new.
- [E003-E9-003][EXP][DATA][line 49] BenchmarkSuite exposes name, version, run(model,harness), and paired_bootstrap_significance(a,b); its registry holds eleven implementations.
- [E003-E9-004][EXP][DATA][lines 85-90] Core symbols are BenchmarkSuite, HarnessConfig, BenchmarkResult, paired_bootstrap_p(a,b,n_iter), and ds_cp_coverage(calib_set,shift_set,alpha).
- [E003-E9-005][EXP][WF/CFG][line 51] SWE-bench-Live pulls the latest 50 issues, uses a 24-hour HTTP cache, verifies content hash and temporal cutoff, and executes through the existing CLI run pipeline.
- [E003-E9-006][EXP][TEST/OBS][lines 51,167] SWE-bench-Live blocks promotion only when month-over-month loss exceeds two points and paired-bootstrap p<0.05, then emits SweBenchLiveRunCompleted.
- [E003-E9-007][EXP][WF][line 53] LiveCodeBench v6 uses a last-60-days slice, PerRoleRuntime, and code-generation, self-repair, and test-output-prediction subtasks.
- [E003-E9-008][EXP][CFG/TEST][lines 53,168] LiveCodeBench floors are 30% local and 45% cloud with per-language breakdown, emitting LiveCodeBenchRunCompleted.
- [E003-E9-009][EXP][SEC/WF][line 55] tau2-bench covers retail and telecom and grades policy adherence rather than successful outcome, so a policy-violating booking fails.
- [E003-E9-010][EXP][CFG/TEST][lines 55,169] tau2 floors are retail .70 cloud/.55 local and telecom .85 cloud/.55 local, separately reported with Tau2BenchRunCompleted.
- [E003-E9-011][EXP][CAP/TEST][lines 57,170] BFCL v3 uses a multi-turn, per-role function-calling slice, complements D-21, has a 70% floor, and emits BfclV3RunCompleted.
- [E003-E9-012][EXP][INT/TEST][lines 59,171] GAIA L1/L2 uses the HAL framework and a contracts fixture; shipped promotion requires L1≥50% and L2≥30%, with GaiaRunCompleted.
- [E003-E9-013][EXP][DET][line 61] ClawsBench runs {model A,model B}×{harness X,harness Y} and decomposes two-way ANOVA variance into model, harness, and interaction effects.
- [E003-E9-014][EXP][OBS/TEST][lines 61,172] ClawsBench emits model_effect_pp/harness_effect_pp, accompanies every harness/scaffold diff, and forbids model attribution when model spread is under 10pp within a harness.
- [E003-E9-015][EXP][SEC/TEST][lines 63,173] PRMBench uses Deception Resistance and Circular Logic subsets, gates D-2 PPM at the published baseline, and emits PrmBenchRunCompleted.
- [E003-E9-016][EXP][SEC/TEST][lines 65,174] Sharma probes cover feedback flip, mimicry, answer flip, and false-claim agreement; flip rate under 15% gates auditor/planner persona changes and emits SycophancyProbeCompleted.
- [E003-E9-017][EXP][TEST][lines 67,175] Math floors use cleaned GSM8K-Platinum, never raw GSM8K, plus MATH-500; local floors are 90% and 50%, with MathFloorsCompleted.
- [E003-E9-018][EXP][STATUS/OBS][lines 69,176] HLE is a 50-question no-tools ceiling slice run quarterly for trend only, never a regression gate, and emits HleCeilingTracked.
- [E003-E9-019][EXP][TEST][lines 71,177] DeepPlanning tests 2-3-day plan fixtures with ≥70% constraint satisfaction and emits DeepPlanningCompleted.
- [E003-E9-020][EXP][DATA/DET][lines 17,73,178] DS-CP replaces plain conformal, evaluates in-domain/near-OOD/far-OOD distributions, requires coverage gap under 5pp, and emits DsCpCoverageMeasured.
- [E003-E9-021][EXP][DET/TEST][line 75] The paired-bootstrap harness estimates required task count for p<.05 on a two-point delta from observed variance and emits BenchPowerCalibrated(required_n,observed_n).
- [E003-E9-022][EXP][FAIL/OBS][lines 75,179] If observed_n is below required_n it emits LowPowerNoConclusion and must not label the result a regression.
- [E003-E9-023][EXP][CAP/CFG][line 77] CLI command elai bench external accepts --suite NAME, --harness-vary, and --p-target 0.05, launches Python runners, tails SSB, and prints pass/fail/low-power verdicts.
- [E003-E9-024][EXP][INT/DATA][line 79] ExternalBenchPanel renders eleven suites in a 4×3 grid with one empty cell, latest score, SOTA reference, p-value, and run-history click-through.
- [E003-E9-025][EXP][INT][lines 79,108] get_external_bench_history(suite_name) returns Vec<BenchmarkResult> through Tauri.
- [E003-E9-026][EXP][DET/SEC][lines 81,116,180] Every LLM-as-judge call rotates option positions and ID tokens per call; a contract test enforces the anti-bias rule.
- [E003-E9-027][EXP][SEC/CONSTR][lines 19,114] Contamination defense must combine n-gram, paraphrase, and canary checks; n-gram-only filtering is forbidden.
- [E003-E9-028][EXP][WF][lines 104-111] SWE-Live consumes run_subcommand, cloud/local routing consumes PerRoleRuntime, promotion gates consume suite verdicts, and CLI or any model/harness promotion path reaches the registry.
- [E003-E9-029][EXP][OBS][lines 91-103] Thirteen declared SSB variants cover the eleven benchmark results plus DS-CP coverage and power calibration.
- [E003-E9-030][EXP][CONSTR][lines 112-117] Stub bans prohibit Verified-only SWE-bench reporting, raw GSM8K, n-gram-only contamination checks, and unrotated LLM judging.
- [E003-E9-031][EXP][OBS/TEST][lines 118-121] The SWE-Live --harness-vary smoke must produce SWE-Live, ClawsBench, and power-calibration rows plus a p-value and dashboard history within 30 minutes.
- [E003-E9-032][EXP][TEST][lines 126-161] Deterministic checks require eleven listed suites, a numeric p-value, non-null Claws effects, temporal cutoff fields, Platinum-only GSM8K, judge rotation, DS-CP gap<5, low-power refusal, and no Verified-only score.
- [E003-E9-033][EXP][TEST][lines 166-180] Acceptance gates each suite at its stated threshold and mandates the harness split, DS-CP replacement, power calibration, and judge rotation.
- [E003-E9-034][EXP][SEC/COMPLIANCE][line 181] Every new Python dependency must pass anti-slopsquatting verification and pyproject records version and last-published date.
- [E003-E9-035][EXP][TEST][lines 185-190] Rust tests verify registry/version strings, low-power behavior, synthetic ANOVA, three-distribution DS-CP, judge rotation, and temporal cutoff rejection.
- [E003-E9-036][EXP][TEST][lines 191-194] Python/UI tests verify tau2 policy grading, four sycophancy behaviors, Platinum-only refusal, and eleven-cell history UI.
- [E003-E9-037][EXP][STATUS][lines 196-217] Three Rust modules, a CLI module, one Svelte panel, eleven Python runners plus pyproject are created; SSB, main, and Tauri command registration are modified.
- [E003-E9-038][EXP][FAIL][line 238] required_n for a two-point SWE-Live delta is explicitly unknown; BenchPowerCalibrated surfaces it and the default n=50 may be too small.
- [E003-E9-039][EXP][FAIL/STATUS][line 239] HLE sampling variance is mitigated by category-stratified sampling and tracker-only status.
- [E003-E9-040][EXP][CFG/STATUS][line 240] ARC-AGI-2 is demoted from hard gate to ceiling tracker; a user may opt back into a hard gate through an environment variable.
- [E003-E9-041][EXP][FAIL/OBS][line 241] tau2 transfer to internal SSB policy is uncertain, so per-policy-rule results are logged for a future custom rubric.
- [E003-E9-042][EXP][DEP/STATUS][line 242] DS-CP ships with synthetic calibration data but production use blocks on deferred D-30.
- [E003-E9-043][EXP][CONSTR/STATUS][line 243] DeepPlanning is best-available emulation, not definitive multi-day measurement; cost-aware quality and persistence remain unmeasured.
- [E003-E9-044][EXP][STATUS/DEFERRED][lines 245,250-254] Full-suite CI sharding, cost-per-task Pareto evaluation, and longitudinal trust drift are follow-ups; the serial suite can take hours to days.
- [E003-E9-045][EXP][INT][lines 250-252] E-7 Darwinian candidates, E-8 templates, and E-10 architecture simplification are evaluated through this suite before promotion.

### Inferred, ambiguous, and conflicting

- [E003-E9-046][INF][DATA][lines 49,73,127-153] DS-CP appears callable as a suite name although it is described as a validator alongside the eleven-suite registry; the source never states whether it occupies one of the eleven slots.
- [E003-E9-047][AMB][DATA][lines 49-75,91-103] The source names eleven public runner implementations and also exposes DS-CP and power calibration, making “11 suites” ambiguous unless validators are excluded.
- [E003-E9-048][AMB][TEST][lines 2,23,51,75] The objective broadly says every score gates at paired-bootstrap p<.05, while suite-specific text gives an explicit p-gate only for SWE-Live and treats insufficient power as no conclusion.
- [E003-E9-049][CON][CAP][lines 1,10,51-75] The orientation summary names only SWE-Live, SWE-Pro, and tau2; the original does not specify SWE-Pro at all and instead explicitly includes LiveCodeBench, BFCL, GAIA, ClawsBench, PRMBench, sycophancy, math, HLE, and DeepPlanning.
- [E003-E9-050][CON][CLI][line 77] The original CLI is elai bench external, not the orientation summary’s elai benchmark --suite swelive.

## Per-file ledger: E-12_production_fullstack_codegen.md

Source anchors below refer to E-12_production_fullstack_codegen.md.

### Explicit atomic ledger

- [E003-E12-001][EXP][CAP][lines 1-10] ProductionGrade codegen moves beyond compile/tests by adding framework conventions, RLS, secrets, OWASP, observability seeds, and consolidated deny-quality tooling.
- [E003-E12-002][EXP][DEP/STATUS][lines 3-5,12-20] Effort is three sessions; B-30/B-37/C-21/C-31/C-32/E-11 are dependencies, E-13/E-14 are parallel, and FrameworkAdapter plus manifests are net-new extensions.
- [E003-E12-003][EXP][DATA][line 37] Seven convention manifests are named: wasp, encore, redwoodsdk, t3, hono, fastapi, and nextjs-app-router.
- [E003-E12-004][EXP][DATA/CONSTR][line 37] Each manifest is token-compact (<200 lines), plain text plus fenced YAML, and contains root config, route/auth/db/env conventions, scaffold command, linter/formatter/test defaults, dependency manifests, and banned patterns.
- [E003-E12-005][EXP][SEC][line 37] Manifest bans include stale Tailwind-v3 classes in v4 projects, localStorage JWTs, production prisma/drizzle push, and hard-coded admin credentials.
- [E003-E12-006][EXP][DATA][line 39] FrameworkProfile variants are Wasp, EncoreDev, RedwoodSdk, T3Stack, HonoEdge, FastApi, NextAppRouter, and Generic.
- [E003-E12-007][EXP][CAP][line 39] FrameworkAdapter exposes scaffold(ProjectSpec)→FileSpec list, production_checklist(), and apply_lint_defaults(FilePlan).
- [E003-E12-008][EXP][DET/CFG][lines 39,51,126] IdeaFramer ProjectClass deterministically selects a profile through PROJECT_CLASS_TO_FRAMEWORK/config.
- [E003-E12-009][EXP][CFG][line 51] Defaults map SaaS→WaspOrEncore, ApiBackend→Hono, AiApp→HonoCloudflare, Marketplace→Encore, and ContentSite→Astro.
- [E003-E12-010][EXP][SEC/FAIL][line 41] secrets_scan runs gitleaks plus trufflehog with 30-second timeout.
- [E003-E12-011][EXP][SEC/FAIL][line 41] owasp_static runs semgrep p/owasp-top-ten with a 60-second timeout.
- [E003-E12-012][EXP][SEC/DATA][line 41] rls_probe parses sqlite3/pg_dump output and flags CREATE TABLE without ENABLE ROW LEVEL SECURITY.
- [E003-E12-013][EXP][SEC][line 41] auth_hardening checks HttpOnly/Secure/SameSite cookies, /auth rate limits, and absence of demo admin accounts.
- [E003-E12-014][EXP][SEC][line 41] dep_audit runs npm audit, pip-audit, and cargo-audit with JSON output.
- [E003-E12-015][EXP][OBS][line 41] otel_present requires Sentry.init and OTel resource attributes in the server entry.
- [E003-E12-016][EXP][OBS][line 41] structured_logging performs grep plus fixture sanity validation of log format.
- [E003-E12-017][EXP][TEST][line 41] axe_lighthouse uses lighthouse-ci, requires ≥95 on critical pages, and runs only when frontend_pair is non-null.
- [E003-E12-018][EXP][TEST/DET][line 41] tdd_fail_first inspects git history: a new test’s first commit is alone and the second adds implementation.
- [E003-E12-019][EXP][TEST][line 41] integration_real_dep rejects integration tests that use an all-mock harness.
- [E003-E12-020][EXP][SEC][line 41] migration_push_ban rejects production-path prisma db push and drizzle-kit push scripts.
- [E003-E12-021][EXP][TEST][line 41] linter_clean runs Biome, Ruff, and Clippy with warnings denied.
- [E003-E12-022][EXP][SEC][line 43] localstorage_jwt rejects token writes to browser localStorage.
- [E003-E12-023][EXP][SEC][line 43] dockerfile_secrets rejects secret-bearing ENV and requires BuildKit secret mounts.
- [E003-E12-024][EXP][SEC/COMPLIANCE][line 43] iac_policy_as_code uses CrossGuard/Checkov and forbids global ingress, disabled PublicAccessBlock, and open S3 buckets.
- [E003-E12-025][EXP][DATA][line 43] LevelAuditVerdict findings add fifteen categories: CSRF_MISSING, SECRET_LEAK, RLS_OFF, OWASP_TOP10, PACKAGE_HALLUCINATED, AUTH_WEAK, LINTER_DUPLICATE, LOCALSTORAGE_JWT, DEMO_CREDS, MIGRATION_PUSH, DOCKERFILE_ENV_SECRET, IAC_OPEN_INGRESS, AXE_FAIL, OTEL_MISSING, and INTEGRATION_ALL_MOCK.
- [E003-E12-026][EXP][WF][line 45] Scope planning must select a profile from config, read its complete manifest, produce a convention-respecting FilePlan, and convert banned patterns into first-occurrence INTERACTIVE MistakeBook blocks.
- [E003-E12-027][EXP][DET/CONSTR][line 47] Scaffold emits exactly one linter/formatter config per language and removes .eslintrc, .prettierrc, .flake8, setup.cfg flake8, and pyproject black/isort sections.
- [E003-E12-028][EXP][DET/OBS][line 47] Each legacy removal emits legacy_config_stripped and C-32 flush_to_disk performs one atomic write per file.
- [E003-E12-029][EXP][OBS/DATA][line 49] SERVER_ENTRY_SEEDS is keyed by profile and adds 15-30 LOC of Sentry init, OTel attributes, and a structured logger only when dependency manifests permit the dependency.
- [E003-E12-030][EXP][SEC/TEST][line 49] SkeletonGenerator preserves the seed and MiniCritic BLOCKs any BLANK_N fill that removes it.
- [E003-E12-031][EXP][CFG][line 51] secrets_scan defaults enabled/pre-commit/block-on-match; rls_probe blocks disabled tables and audits the superuser pool; linter consolidation strips legacy configs and enforces ts=biome, py=ruff, rs=clippy_deny.
- [E003-E12-032][EXP][SEC/OBS][line 53] The installed pre-commit hook runs gitleaks and trufflehog, BLOCKs if either finds a secret, records SecretInRepo, and supports GITLEAKS_ALLOWLIST.
- [E003-E12-033][EXP][WF][line 55] Phase- and Step-level critics re-read banned_patterns and emit findings for banned FILE_PLAN content.
- [E003-E12-034][EXP][TEST/SEC][line 57] testing.assertions compares assertion ASTs against git prior and BLOCKs ASSERTION_ONLY_DIFF when tests change without implementation.
- [E003-E12-035][EXP][INT/DATA/OBS][line 59] IdeaFramerSecurityPanel shows profile, manifest summary, and 15-or-18 checklist rows; get_framework_profile reads FrameworkProfileSelected payload {profile,manifest_path,scaffold_paths[]} emitted at scope-planner exit.
- [E003-E12-036][EXP][WF][lines 62-77] FrameworkAdapter consumes ProductionGrade, ProjectClass, FilePlan, SkeletonGenerator, and MistakeBook and is reachable through task_runner/scope_planner and dashboard submit_task.
- [E003-E12-037][EXP][FAIL/CONSTR][lines 78-82] Checklist code forbids todo/unimplemented, silent error swallowing, hard-coded paths, and Box<dyn Error>; errors use FrameworkCheckError.
- [E003-E12-038][EXP][OBS/TEST][lines 84-87] The SaaS smoke must choose wasp/encore within 30 seconds of FramingConverged, emit legacy-strip rows, and create production_grade_checklist.toml with 15+ rows and no RLS_OFF/OTEL_MISSING findings when correctly scaffolded.
- [E003-E12-039][EXP][TEST][lines 89-122] Deterministic gates count seven manifests and fifteen runner files, verify profile symbols/seeds/hooks/events, and run RLS-off and legacy-strip smoke tests.
- [E003-E12-040][EXP][TEST][lines 124-134] Acceptance requires deterministic dispatch, serde-compatible fifteen-category findings, seed preservation, five config tables, hook auto-install, two events, dashboard/Tauri wiring, and no NoopSsbNotifier/Box<dyn Error>/production unwrap.
- [E003-E12-041][EXP][TEST][lines 136-144] Tests cover seven profile scaffolds, RLS states, secret patterns, six OWASP cases, localStorage JWT, seed removal, assertion-only diffs, and manifest YAML keys.
- [E003-E12-042][EXP][STATUS][lines 146-163] Seven manifests plus framework/checklist code are created; auditor, skeleton generator, config, hook, SSB, scope planner, dashboard panel, and Tauri commands are modified.
- [E003-E12-043][EXP][FAIL/OBS][lines 182-187] The profile table is a heuristic pending 20+ sessions; hook latency is measured; missing Tier-C degrades to warning-only and emits production_checklist_degraded_no_sandbox; semgrep is pinned/quarterly refreshed; mobile profiles need localStorage allowlists.
- [E003-E12-044][EXP][STATUS/DEFERRED][lines 189-193] E-13 adds mutation/TDD signal, E-14 adds Playwright/AXTree, framework complexity can tighten scope ceilings, and retroactive historical audits are follow-up.

### Inferred, ambiguous, and conflicting

- [E003-E12-045][AMB][CAP][lines 20,37] Task 1 says “Author 5” but names seven manifests; all later wiring, checks, acceptance, and files require seven, so seven is the operative requirement.
- [E003-E12-046][AMB][CFG][lines 39,51] FrameworkProfile has Wasp/EncoreDev/HonoEdge etc., while config values include composite or absent enum labels WaspOrEncore, HonoCloudflare, and Astro; translation into actual enum variants is unspecified.
- [E003-E12-047][AMB][DATA][lines 14,41-43,59] The plan alternates among 12-base+3 new=15, 12 base+3-6 framework-specific, and “15 (or 18)” UI rows; the precise per-profile checklist cardinality is not fixed.
- [E003-E12-048][CON][CONSTR][lines 183,79,124-135] A Wasp manifest may be file-stubbed with TODO[E-12] if upstream docs are incomplete, but stub_ban/acceptance otherwise ban todo/unimplemented in production checklist code; the exception is prose-manifest-specific, not a general code stub permission.

## Per-file ledger: E-18_tool_use_reliability.md

Source anchors below refer to E-18_tool_use_reliability.md.

### Explicit atomic ledger

- [E003-E18-001][EXP][CAP][lines 1-14] The tool path is hardened with XGrammar, a Hermes envelope, SEARCH/REPLACE fences, one-shot error-aware recovery, and ToolResult spill, addressing malformed JSON, escape bugs, retry loops, edit disorientation, package hallucination, and long responses.
- [E003-E18-002][EXP][DEP/STATUS][lines 3-5,16-22] Effort is two-to-three sessions; B-7b and B-37 are shipped prerequisites, B-20 and C-13 are planned integration frames, and D-13 remains a later cross-family verifier.
- [E003-E18-003][EXP][CAP/DET][line 39] server.py replaces regex/in-house constraints with xgrammar.GrammarMatcher and a transformers LogitsProcessor.
- [E003-E18-004][EXP][WF/FAIL][lines 39,141] All fifteen IPC schemas plus tool schemas precompile at sidecar boot; any compile failure is a startup error, never a fallback.
- [E003-E18-005][EXP][DATA/SEC][line 41] Grammar masking requires the Hermes <tool_call>...</tool_call> envelope and makes the known fenced-json fallback impossible at token selection.
- [E003-E18-006][EXP][TEST][lines 41,142] A Qwen2.5-Coder-7B-NF4 sidecar smoke fuzzes 100 tool prompts and requires 100% envelope presence.
- [E003-E18-007][EXP][CAP/DATA][line 43] Edit tier one becomes SearchReplaceMatcher with exact fences <<< ELAI_SEARCH, === ELAI_REPLACE, and >>> ELAI_END.
- [E003-E18-008][EXP][FAIL][lines 43,144] DelimiterCollisionGuard scans for the custom tokens and git conflict markers; any collision falls back to whole-file rewrite.
- [E003-E18-009][EXP][FAIL/WF][line 45] Tier-2 validation failure or a SEARCH/REPLACE miss gets exactly one reprompt containing a structured <tool_error>; a second failure escalates to .claude/agents/fixer.md.
- [E003-E18-010][EXP][DET/CONSTR][lines 45,90,145] A retry may never reuse the same prompt and can never loop beyond the single recovery attempt.
- [E003-E18-011][EXP][OBS][line 45] Recovery emits tool_one_shot_recovered on success or tool_retry_exhausted after the capped attempt.
- [E003-E18-012][EXP][DATA/WF][lines 47,69,146] ToolResult inline content is capped at 8K tokens; overflow writes .elai_cc/sessions/{id}/spill/{uuid}.txt and returns spill_ref.
- [E003-E18-013][EXP][OBS][line 47] A spilled result emits tool_result_spilled.
- [E003-E18-014][EXP][SEC/WF][line 49] Before Edit/Bash touches Cargo.toml, pyproject.toml, or package.json, registry_check verifies package name and last-published age against crates.io/PyPI/npm.
- [E003-E18-015][EXP][SEC/FAIL][lines 49,147] Unknown packages and packages younger than 30 days are rejected before execution, emitting existing package_ref_verified or new package_ref_rejected.
- [E003-E18-016][EXP][CFG/SEC][line 51] GroundingRequest carries a path allowlist and the sidecar narrows the tool schema path enum at grammar-mask time, preventing emission of forbidden paths.
- [E003-E18-017][EXP][FAIL/OBS][lines 51,93,148] A path mismatch emits path_allowlist_pre_filtered; visibility_filtered is retired in favor of NotFound plus non-leaking evidence.
- [E003-E18-018][EXP][DATA/DET][lines 53,70,149] A ProgressMarker frame follows every ToolResult; duplicate (tool,args-hash) calls are deduped within a five-frame window.
- [E003-E18-019][EXP][DET/OBS][lines 53,95] Args hashes use blake3 and dedupe emits progress_marker_emitted/duplicate_tool_call_deduped.
- [E003-E18-020][EXP][STATUS/DEFERRED][line 55] CodeAct’s compositional Python execution channel and Tier-3 validation are explicitly deferred to E-18.B pending a 7B-class ablation.
- [E003-E18-021][EXP][STATUS/DEFERRED][line 57] Cap’n Proto/postcard IPC migration is deferred to E-18.C until tunneled-WSL2 latency is measured.
- [E003-E18-022][EXP][DATA][lines 61-78] Produced identifiers are XGrammarLogitsProcessor, compile_all_schemas_at_boot, SearchReplaceMatcher, DelimiterCollisionGuard, OneShotRetry, ToolErrorReprompt, registry_check, ToolResult.spill_ref, ProgressMarker, and eight new SSB variants.
- [E003-E18-023][EXP][OBS][lines 71-78] Exact new action variants are ToolRetryExhausted, ToolOneShotRecovered, ToolResultSpilled, PackageRefRejected, PathAllowlistPreFiltered, ProgressMarkerEmitted, DuplicateToolCallDeduped, and XgrammarSchemaCompileFailed.
- [E003-E18-024][EXP][WF][lines 79-87] The changes wrap sidecar complete, dispatch_edit, dispatch_tool, IPC send, task_runner tool-call phase, and agent-role dispatch; sidecar boot is a direct entry.
- [E003-E18-025][EXP][CONSTR][lines 88-95] Stub bans forbid silent fenced JSON, same-prompt retry, >8K inline results, unverified package edits, visibility_filtered, production unwrap, and non-blake3 args hashes.
- [E003-E18-026][EXP][INT/OBS][line 96] AgentDrawer gains retry-exhausted and spill counters.
- [E003-E18-027][EXP][SEC/TEST][lines 97-100] A hallucinated-package smoke must reject tokyo-async-future-v3 before Cargo.toml changes, avoid retry exhaustion, keep rollback clean, and include error_chain evidence.
- [E003-E18-028][EXP][TEST][lines 102-137] Deterministic checks verify processor/warmup, envelope grammar, fences/guard, retry, spill, registry verification, and an SSB package rejection.
- [E003-E18-029][EXP][TEST][lines 139-151] Acceptance requires all structural behaviors plus dispatch-time anti-slopsquatting and a logged internal 50-prompt multi-turn score target ≥.50.
- [E003-E18-030][EXP][TEST][lines 153-161] Contracts fuzz envelope/compile failure, twenty edit fixtures with ≥95% tier-one hit rate, exactly-one retry/fixer escalation, 50K-token spill, registry cases, token-level path filtering, and boot-time schema warmup.
- [E003-E18-031][EXP][STATUS][lines 163-175] server.py, sidecar, dispatchers, IPC, SSB, and AgentDrawer are modified; search_replace, package_verify, one_shot_retry, and E-18 decisions are created.
- [E003-E18-032][EXP][FAIL][line 192] Cold-schema CPU stalls are mitigated by mandatory warmup; if boot exceeds 30 seconds, lazy compile is only a considered alternative with a first-use budget.
- [E003-E18-033][EXP][FAIL][lines 193-195] Future language fence collisions require guard extension; registry latency should use caching; legitimate fresh packages need explicit --allow-fresh-package with audit logging.
- [E003-E18-034][EXP][STATUS/DEFERRED][lines 196-207] The .50 local target must be measured then ratcheted; E-18.B/E-18.C, B-20, C-13, D-13, and a future Erlang-style supervisor consume later slices.

### Inferred, ambiguous, and conflicting

- [E003-E18-035][INF][WF][lines 39-53] XGrammar envelope masking, path-enum masking, retry, and dedupe compose into one tool transaction, but no atomic transaction boundary across Python and Rust is specified.
- [E003-E18-036][AMB][CAP][line 2] The objective says “seven documented failure modes” but explicitly lists six; the omitted seventh is not identifiable from the source text.
- [E003-E18-037][AMB][CAP][line 2,37-57] The objective calls these “five concrete changes,” while the task list adds registry verification, path masking, ProgressMarker dedupe, and two deferrals beyond those five.
- [E003-E18-038][CON][CAP][lines 43-45] The orientation summary describes malformed JSON being reshaped and first-attempt patch success; the original instead rejects/masks malformed output and permits one error-embedded retry, so “first attempt” is an acceptance aspiration only for the matcher fixtures, not a universal guarantee.

## Per-file ledger: E-26_inference_engines_2026.md

Source anchors below refer to E-26_inference_engines_2026.md.

### Explicit atomic ledger

- [E003-E26-001][EXP][INT/STATUS][line 3] E-26 feeds PS-4 routing economics through a SGLang throughput tier and Ollama demotion; the live $/verified-task leg remains blocked on J-28.
- [E003-E26-002][EXP][CAP][lines 5,15-21] The runtime policy is hybrid: SGLang for cloud planner/critic/structured JSON, custom Sidecar for research/DoLa/ETC/steering/TurboQuant, and Ollama deprecated but retained.
- [E003-E26-003][EXP][DEP/STATUS][lines 7-10,46-48] B-37 is shipped prerequisite; S1-S4 are free-lane shippable, while S5 is gated on unresolved E-2 Part-3 plus RunPod.
- [E003-E26-004][EXP][STATUS][lines 25-31] B-37’s eleven backends become twelve with Sglang; Vllm remains untouched and Sidecar’s processor chain remains canonical.
- [E003-E26-005][EXP][DATA/CONSTR][line 28] Existing media-validator SglangConfig and new long-lived SglangRuntimeConfig are distinct lifecycle types and must not be collapsed.
- [E003-E26-006][EXP][CAP/INT][line 52] SglangBackend implements RuntimeAdapter over OpenAI-compatible /v1/chat/completions and native /generate endpoints, with /v1/models health and SSE streaming.
- [E003-E26-007][EXP][DATA][line 52] SglangRuntimeConfig contains base_url, optional api_key, request_timeout, and StructuredOutputBackend.
- [E003-E26-008][EXP][CFG][line 52] StructuredOutputBackend variants are XGrammar, Outlines, and LMFE; response_format JSON schema and native regex are both supported.
- [E003-E26-009][EXP][CONSTR][line 52] The adapter is HTTP-only and must not launch sgl-server; request-ID hashing uses blake3.
- [E003-E26-010][EXP][WF][lines 53-58] Factory registration adds runtime/mod.rs module exposure, adapter enum/factory arms, resolve_backend("sglang"), and existing permission opt-in.
- [E003-E26-011][EXP][CFG/WF][lines 60-64] With a healthy configured endpoint, planner, critic, prompt_engineer, and structured-JSON callers route to Sglang; research, executor, and memory_writer keep existing routes.
- [E003-E26-012][EXP][FAIL/CFG][lines 60-62,136] If SGLang is unhealthy routing falls back to Sidecar; if ELAI_SGLANG_BASE_URL is unset behavior must exactly match B-37 defaults.
- [E003-E26-013][EXP][FAIL][line 66] RuntimeAdapter trait drift is handled by mirroring vllm; non-mechanical drift requires STOP/handoff rather than a stub.
- [E003-E26-014][EXP][STATUS/OBS][lines 68-74] Ollama remains constructible and factory-resolvable but gains doc-hidden deprecation plus one construction warning citing the 19× throughput and 1.2GB RAM gap.
- [E003-E26-015][EXP][CFG/OBS][lines 70-72] rules/schema adds a dated WARN recommending Sidecar local or Sglang cloud; its tuple shape must follow neighboring schema rows.
- [E003-E26-016][EXP][OBS/DATA][line 80] New action types are sglang_backend_registered, xgrammar_schema_compiled with {schema_hash,compile_ms}, and runtime_cold_start_observed.
- [E003-E26-017][EXP][DET/OBS][lines 80-90] All three actions round-trip through parse/as_sql/golden lock; a frozen lock requires owner approval.
- [E003-E26-018][EXP][OBS/WF][lines 84-88] The first successful Sglang health probe records elapsed_ms in runtime_cold_start_observed and RuntimePanel tells the user cold-start cleared.
- [E003-E26-019][EXP][CFG][lines 92-96] Static config accepts [agents.<role>.runtime] backend="sglang" and [runtime.sglang] base_url without altering B-37 alias behavior.
- [E003-E26-020][EXP][DATA/STATUS][lines 98-100] E-26-decisions documents SGLang-not-vLLM, retain-but-deprecate Ollama, XGrammar head placement, deferred vLLM-Windows/TGI, and the E-2/owner gate.
- [E003-E26-021][EXP][STATUS/DEFERRED][lines 106-116] XGrammarLogitsProcessor is RECON_NEEDED and must not execute until E-2 Part-3 lands and RunPod is available.
- [E003-E26-022][EXP][CAP/DATA][line 110] The gated class wraps xgrammar.GrammarCompiler and a per-request GrammarMatcher.
- [E003-E26-023][EXP][DET/DATA][line 110] Compiled grammars cache once per (model,json_schema_hash) in an LRU keyed by blake3(schema_json).
- [E003-E26-024][EXP][WF][line 110] XGrammar inserts at the processor-chain head so structural masking precedes DoLa/ETC contrasts.
- [E003-E26-025][EXP][CFG][line 110] It activates only for tools[].input_schema or structured_output {type:json_schema,schema,backend:xgrammar}; without a schema it is absent and adds no overhead.
- [E003-E26-026][EXP][FAIL][lines 112,116,120-121] Any coexistence failure with prompt lookup/StaticCache requires STOP/handoff; bypassing DoLa/ETC and executing S5 early are forbidden.
- [E003-E26-027][EXP][FAIL][lines 122-125] Additional aborts are irreconcilable trait drift, frozen SSB lock, a new Rust dependency without confirmation, or failed xgrammar package/license verification.
- [E003-E26-028][EXP][OBS/TEST][lines 129-136] Off-agent verification runs production caller/wire checks, workspace check, provider/orchestrator tests, SSB-WAL evidence, RunPod pytest for S5, and backward-compat routing.
- [E003-E26-029][EXP][SEC/TEST][lines 140-145] Red-team gates require a same-slice producer+factory+consumer, preserve research on Sidecar, block early S5, prohibit new Rust crates, and require allowlisting for the default-OFF flag.
- [E003-E26-030][EXP][TEST][lines 149-152] Free-lane integration progresses through workspace/unit checks, real SSB writer round-trip, then a mock-SGLang cross-seam smoke over planner/critic/research and cold-start evidence.
- [E003-E26-031][EXP][TEST/STATUS][line 153] Live IS4 is hardware-gated and unverified until RunPod; it must prove schema compilation/masking/valid JSON, temp=0 byte equivalence on five schemas, preserved research processors, and xgrammar_schema_compiled evidence.
- [E003-E26-032][EXP][CFG/COMPLIANCE][line 154] ELAI_SGLANG_BASE_URL requires a wire-dark-allowlist row with type, owner, expiry_date, promotion_plan_id, and an ON-path smoke; missing it blocks.
- [E003-E26-033][EXP][WF][lines 158-174] Produced types connect through resolve_backend, runtime module registration, PerRoleRuntime, is_recoverable, rule schema, and the gated processor chain; dashboard and REPL runtime views expose health.
- [E003-E26-034][EXP][CONSTR][lines 175-181] Stub bans forbid TODO adapter text, speculative vLLM-V1 comments, new Ollama defaults, and any SGLang processor-skip branch.
- [E003-E26-035][EXP][OBS][lines 182-185] The live trigger expects backend registration at boot, cold-start on health, gated XGrammar compile on first schema call, valid JSON without retry, green RuntimePanel, and preserved DoLa for research.
- [E003-E26-036][EXP][TEST][lines 187-201] Deterministic checks cover production wiring, file/factory/routing/class/import/events/deprecation/doc presence and Sglang health tests.
- [E003-E26-037][EXP][TEST][lines 203-215] Acceptance requires <200ms mock health, deferred five-schema RunPod parity, preserved processor chain, exactly-once Ollama warning, default routing compatibility, three-event WAL round-trip, no new Rust deps, no added unwraps, green workspace tests, allowlist, and no dark public function.
- [E003-E26-038][EXP][TEST][lines 217-223] Added contracts cover backend health/complete/stream, role routing, Ollama warning, gated XGrammar order/parity, and three action types.
- [E003-E26-039][EXP][STATUS][lines 225-241] Sglang adapter/contracts/decisions are created; factory, routing, SSB, Ollama, rules, gated server/dependency, and wire-dark allowlist are modified.
- [E003-E26-040][EXP][STATUS/DEFERRED][lines 255-265] EAGLE-3, RadixAttention-default routing, vLLM native Windows, TGI long context, and speculative decoding wait for workload/hardware measurements.

### Inferred, ambiguous, and conflicting

- [E003-E26-041][INF][INT][lines 5,19,52,110] AWQ models may later sit behind SGLang, but this source ships no weight-format selection for SGLang; that composition belongs to E-27/future deployment.
- [E003-E26-042][AMB][OBS][lines 80,84] sglang_backend_registered is declared but the exact emit site/timing is not described as precisely as runtime_cold_start_observed.
- [E003-E26-043][CON][STATUS][lines 10,106-116,206] The orientation summary treats XGrammar as a normal deliverable; the original marks it gated, RECON_NEEDED, RunPod-only, and explicitly non-executable on the free lane.
- [E003-E26-044][CON][FAIL][lines 21,68-74,178] The orientation summary says constructing Ollama should “warn and skip”; the original explicitly retains factory resolution and only warns once.
- [E003-E26-045][CON][OBS][lines 80,163,184] The orientation summary’s sglang_health_changed and ollama_deprecated events do not exist in the original contract; the source events are SglangBackendRegistered, XgrammarSchemaCompiled, and RuntimeColdStartObserved.

## Per-file ledger: E-27_quantization_2026.md

Source anchors below refer to E-27_quantization_2026.md.

### Explicit atomic ledger

- [E003-E27-001][EXP][CAP][lines 1-18] Serving evaluates AWQ+Marlin W4A16 and FP8 KV while preserving NF4 for QLoRA; migration order is FP8 KV, then AWQ, then a separate QLoRA checkpoint path.
- [E003-E27-002][EXP][DEP/STATUS][lines 3-6] Effort is two sessions; B-37 is required, E-26 optional, and E-26/E-28/E-2 require coordinated server.py windows but are logically orthogonal.
- [E003-E27-003][EXP][CFG][line 45] ELAI_QUANT_BACKEND is nf4|awq|fp16 and defaults to nf4 for backward compatibility; --quant-backend awq plumbs to it.
- [E003-E27-004][EXP][CAP/FAIL][line 45] AWQ loads through AwqConfig(version="marlin") on compute capability ≥8.0 and falls back to version="gemm" before Ampere.
- [E003-E27-005][EXP][CONSTR][line 45] NF4 loading remains available and may not be deleted because it serves QLoRA fine-tuning.
- [E003-E27-006][EXP][CFG/DATA][line 47] ELAI_KV_CACHE_DTYPE is fp16|fp8_e4m3|fp8_e5m2 and defaults to fp16.
- [E003-E27-007][EXP][CAP/FAIL][line 47] FP8 KV uses native hardware on Ada+ and a software-simulated path on Ampere.
- [E003-E27-008][EXP][OBS][line 47] Ampere simulation emits kv_cache_dtype_software_sim and records the approximately 5% throughput cost.
- [E003-E27-009][EXP][DATA][line 49] ModelCapabilityProfile gains quant_format and five VRAM-tier recommendations.
- [E003-E27-010][EXP][CFG][line 49] The 8GB tier uses Qwen2.5-Coder-3B AWQ-Marlin with GGUF Q5_K_M fallback when AWQ is unavailable.
- [E003-E27-011][EXP][CFG][line 49] The 12GB tier uses Qwen2.5-Coder-7B AWQ-Marlin plus FP8 KV.
- [E003-E27-012][EXP][CFG/STATUS][line 49] The 24GB tier targets Qwen2.5-Coder-14B AWQ-Marlin while EXL3 is deferred.
- [E003-E27-013][EXP][CFG][line 49] The 48GB tier uses Qwen2.5-Coder-32B FP8 W8A8 on Ada+ or AWQ+FP8KV on Ampere.
- [E003-E27-014][EXP][CFG/STATUS][line 49] The 48GB+B200 tier reserves future NVFP4+QAD.
- [E003-E27-015][EXP][CFG][line 51] ResearchTierConfig keeps NF4+FA2 only for QLoRA-eligible use and selects AWQ-Marlin+FA2+FP8KV for ≥12GB serving tiers.
- [E003-E27-016][EXP][SEC/CONSTR][lines 53-58] Five dated rules BLOCK AWQ without Marlin/Machete, sub-W4/W4A8 reasoning cliffs, and TurboQuant<4-bit KV; they WARN on NF4 beyond 40K and GGUF Q4_K_M for code.
- [E003-E27-017][EXP][CAP/DATA][line 60] build_awq.py takes a checkpoint and default 256-line code calibration sample and creates a coexisting Qwen2.5-Coder-7B-AWQ checkpoint.
- [E003-E27-018][EXP][SEC][line 60] autoawq must pass PyPI anti-slopsquatting verification before dependency addition.
- [E003-E27-019][EXP][CFG][line 62] REMOTE-SIDECAR documents an alternate AWQ+fp8_e5m2 boot while retaining the NF4 boot as default.
- [E003-E27-020][EXP][DET/TEST][line 64] Five fixed prompt classes run at temperature zero under NF4 and AWQ; the gate requires HumanEval parity and token-cosine >.95 rather than byte equality.
- [E003-E27-021][EXP][TEST/OBS][line 66] Long-context regression sweeps 8K/16K/25K, requires code-completion parity, and writes awq_vs_nf4_longctx.csv.
- [E003-E27-022][EXP][STATUS/CFG][line 68] Both features ship default-off for a telemetry week; 12GB+ defaults flip only in a later patch after tasks 8/9 pass three consecutive remote-3090 runs.
- [E003-E27-023][EXP][DATA/STATUS][line 70] E-27-decisions records W4A16, FP8-not-TurboQuant, Marlin-not-GPTQ-Marlin, NF4-QLoRA carve-out, and EXL3/NVFP4/MXFP4/TurboQuant non-goals.
- [E003-E27-024][EXP][DATA/OBS][lines 74-81] Produced identifiers are AwqMarlinLoader, Fp8KvCacheConfig, build script/tests, QuantBackendSelected, KvCacheDtypeSoftwareSim, AwqCheckpointBuilt, and five rule rows.
- [E003-E27-025][EXP][WF][lines 82-88] Env boot selects weights/cache and emits SSB; build_awq creates the checkpoint through a separate user-invoked path.
- [E003-E27-026][EXP][CONSTR][lines 89-94] Stub bans forbid unimplemented Marlin, silent NF4 fallback, EXL3 imports, and TurboQuant<4-bit KV; no new frontend is required.
- [E003-E27-027][EXP][OBS/TEST][lines 95-98] The live artifact target is ≥3× NF4 tok/s, sustained 25K parity, and VRAM ≤2.0GB for 3B/≤4.5GB for 7B, with quant_backend_selected payload including backend, kv dtype, compute capability, and marlin flag.
- [E003-E27-028][EXP][TEST][lines 100-129] Deterministic checks verify env branches, build script, profiles, tier strings, rules, events, hardware tests, and decisions.
- [E003-E27-029][EXP][TEST][lines 131-142] Acceptance requires Marlin boot, native/sim FP8 evidence, five-prompt parity, 25K parity, held-out HumanEval non-regression, recorded tok/s on 3060/3090, preserved NF4, no deferred formats, event round-trip, and green hardware/workspace tests.
- [E003-E27-030][EXP][TEST][lines 144-150] Tests cover five-prompt parity, long-context sweep, native/sim cache behavior, event round-trip, and 12/24/48GB profile resolution.
- [E003-E27-031][EXP][STATUS][lines 152-167] Four Python scripts/tests, two Rust contracts, and decisions are created; server, pyproject, profiles, tiers, rules, SSB, and remote boot docs are modified.
- [E003-E27-032][EXP][FAIL][lines 185-189] Qwen-7B Marlin speed, long-context curve, and FP8 code impact are unmeasured; Ampere defaults FP8 off until parity evidence, and autoawq version must be reverified.
- [E003-E27-033][EXP][STATUS/DEFERRED][lines 190-195] EXL3, NVFP4/MXFP4, TurboQuant KV, and embedding-store TurboQuant are not adopted; future patches promote AWQ and add 14B/Blackwell lanes.

### Inferred, ambiguous, and conflicting

- [E003-E27-034][INF][INT][lines 4-5,195] AWQ checkpoints can compose with future SGLang serving, but no SGLang model-loader changes are specified here.
- [E003-E27-035][AMB][CFG][lines 47,134,189] The source says Ada+ uses native FP8 and Ampere simulation, but acceptance wording associates fp8_e5m2 with Ada+ without pinning which FP8 variant each architecture should prefer.
- [E003-E27-036][CON][STATUS][lines 2,45,62,68] The orientation summary says “switch serving from NF4 to AWQ”; the original keeps nf4 as the default and AWQ default-off until a later three-run promotion.
- [E003-E27-037][CON][TEST][lines 10,27,64,98,137] 4.4× is cited from external 32B measurements and “identical HumanEval”; the implementation’s live target is ≥3× plus local parity, so 4.4× is not the immediate acceptance floor.

## Per-file ledger: E-32_context_window_extension.md

Source anchors below refer to E-32_context_window_extension.md.

### Explicit atomic ledger

- [E003-E32-001][EXP][CAP][lines 1-14] The plan combines a native 32K ceiling, training-free FITM calibration, an opt-in 64K DCA deep lane, and a loud rejection of YaRN on the default coder model.
- [E003-E32-002][EXP][DEP/STATUS][lines 3-6,16-24] Effort is one-to-two sessions; B-37 and the knowledge engine exist, B-31 is parallel, E-31 supplies canonical VRAM gating, and no C/D plan overlaps FITM/context.
- [E003-E32-003][EXP][CFG/STATUS][lines 18-19] Documentation currently presents 25K as hard ceiling and 20K production, while runtime_client’s code default is 131,072; E-32 changes docs/env guidance, not that code default.
- [E003-E32-004][EXP][CFG][line 42] CLAUDE.md changes the documented hard ceiling to 32,000 and production guidance to 25,000-28,000 with grounded VRAM/model-card rationale.
- [E003-E32-005][EXP][CFG/DATA][line 44] REMOTE-SIDECAR changes production/max guidance to 25K/32K and recomputes the 85% handoff point as 27,200 tokens.
- [E003-E32-006][EXP][CAP/WF][line 46] FoundInTheMiddleCalibrator runs only when prompt_len>8K and ELAI_FITM_ENABLED=1, using a second forward pass with a dummy replacement and subtracting positional prior.
- [E003-E32-007][EXP][CFG/CONSTR][lines 46,89] ELAI_FITM_DUMMY_TEXT defaults to a 50-token lorem block and no dummy may exceed 100 tokens.
- [E003-E32-008][EXP][WF/CONSTR][lines 46,77,142,152] FITM inserts after existing DoLa/ETC/steering processors and is a no-op with no extra forward pass for prompts ≤8K.
- [E003-E32-009][EXP][CFG/TEST][lines 46,56,62,200] FITM defaults OFF and can become default-on only after an internal 20-50-trace RAG evaluation shows at least +5pp recall@3.
- [E003-E32-010][EXP][OBS][line 46] Each applied calibration emits fitm_calibration_applied with prompt_len, dummy_len, and applied_ms.
- [E003-E32-011][EXP][DATA/CFG][line 48] The sidecar Anthropic request gains a per-request fitm override, defaulting to environment configuration.
- [E003-E32-012][EXP][CFG/CAP][line 50] ELAI_DEEP_MODE=1 on a box with at least 24GB loads Qwen/Qwen2.5-7B-Instruct-1M instead of the standard abliterated 7B.
- [E003-E32-013][EXP][CAP/CONSTR][line 50] Deep mode enables DCA chunked prefill and hard-caps context at 64K.
- [E003-E32-014][EXP][FAIL][lines 50,90] On less than 24GB deep mode is refused, falls back to standard mode, logs a warning, and must not crash.
- [E003-E32-015][EXP][OBS][line 50] Successful deep-mode boot emits deep_mode_activated with model_id, ctx_cap, and vram_gb.
- [E003-E32-016][EXP][WF/DET][line 52] At a SessionDag handoff, cross-file synthesis means more than three referenced files and no retrieval-localized answer; if a reachable 24GB box exists, local deep mode is preferred over cloud.
- [E003-E32-017][EXP][OBS][line 52] The escalation emits deep_mode_escalation_chosen with reason and files_referenced.
- [E003-E32-018][EXP][SEC/FAIL][line 54] ELAI_USE_YARN=1 with Qwen2.5-Coder raises a cited SidecarBootError and instructs use of DCA deep mode instead.
- [E003-E32-019][EXP][OBS/TEST][lines 54,74,147-148] The YaRN refusal path emits YarnRefused/yarn_refused; loud refusal with no silent fallback is acceptance-required.
- [E003-E32-020][EXP][TEST/OBS][line 56] run_fitm_eval.py reads 20-50 workflow/session retrieval traces, compares FITM on/off recall@3 and recall@5 locally, and writes dated JSON results.
- [E003-E32-021][EXP][DET/CFG][lines 58,80,149] Handoff threshold is 85% of sidecar-reported effective_max_context and may not hard-code 20K, 25K, or 32K.
- [E003-E32-022][EXP][STATUS/DEFERRED][line 60] A-MEM Zettelkasten memory is deferred because it overlaps B-31/B-32 knowledge and symbol graphs.
- [E003-E32-023][EXP][DATA/STATUS][line 62] E-32-decisions records 32K-vs-YaRN, the +5pp FITM gate, 24GB cross-file deep-mode gating, and explicit non-goals YaRN, StreamingLLM, Ring Attention, Infini-attention, and 1M serving.
- [E003-E32-024][EXP][DATA][lines 66-75] Produced identifiers are FoundInTheMiddleCalibrator, activate_deep_mode, refuse_yarn_at_boot, deep_mode_escalation_hook, four SSB variants, and the FITM evaluation scaffold.
- [E003-E32-025][EXP][WF][lines 76-85] The sidecar processor chain, Sidecar request passthrough, grounding-client threshold, boot checks, long-request calibration, and handoff escalation form the reachability paths.
- [E003-E32-026][EXP][CONSTR][lines 86-92] Stub bans prohibit reachable YaRN, hard-coded context thresholds, oversized dummy text, deep boot below 24GB, untyped boundary errors, and non-blake3 cache hashing.
- [E003-E32-027][EXP][INT][line 93] RuntimePanel shows effective_max_context and deep-mode status.
- [E003-E32-028][EXP][OBS/TEST][lines 94-97] A 27,200-token four-file smoke with FITM and a 24GB remote expects calibration, escalation, activation, 32K telemetry, and an AuditTimeline handoff event.
- [E003-E32-029][EXP][TEST][lines 99-136] Deterministic checks validate docs, calibrator, deep activation, YaRN refusal, four event arms, eval scaffold, dynamic threshold, and an end-to-end FITM SSB row.
- [E003-E32-030][EXP][TEST][lines 138-153] Acceptance requires docs, processor order, per-long-request evidence, 24GB DCA boot/refusal, escalation and refusal evidence, dynamic threshold, decisions, short-request no-op, no unwrap, and blake3.
- [E003-E32-031][EXP][TEST][lines 155-162] Tests cover ≤8K no-op/>8K extra pass/bounded delta, YaRN typed failure, 24GB activation/12GB fallback, FITM/deep SSB schema, and dynamic handoff.
- [E003-E32-032][EXP][STATUS][lines 164-175] CLAUDE/remote docs, server, escalation policy, Sidecar, grounding client, SSB, and RuntimePanel are modified; FITM eval and decisions are created.
- [E003-E32-033][EXP][FAIL][lines 199-205] Risks are unmeasured 12GB 32K VRAM, workload-specific FITM lift, slow deep boot, mock-sidecar compatibility, a hard YaRN break, A-MEM overlap, and local-deep-vs-cloud policy.
- [E003-E32-034][EXP][FAIL][line 202] If sidecar effective_max_context is unavailable, threshold logic may use a sane 25K compatibility fallback.
- [E003-E32-035][EXP][STATUS/DEFERRED][lines 207-214] A-MEM and PINE are deferred; B-31 must avoid double FITM, C-13 consumes events, and E-31 must receive the 24GB deep-mode row.

### Inferred, ambiguous, and conflicting

- [E003-E32-036][INF][WF][lines 46,52] FITM and deep-mode escalation can occur in one long request sequence, but state transfer between the standard and deep sidecars is not specified.
- [E003-E32-037][AMB][FAIL][lines 50,90,144] “Refuse to boot deep mode” and “fall back to standard mode” coexist; the operative acceptance behavior is non-crashing fallback, but whether the process reports boot success or a typed recoverable error is unspecified.
- [E003-E32-038][CON][CFG][lines 10,18-19,42] The title/objective says lift 25K→32K, while the current documented production setting is 20K and runtime code defaults to 131,072; this is a documentation/effective-policy lift, not a single code constant change.
- [E003-E32-039][CON][CAP][lines 2,50-52] The orientation summary omits the 64K Qwen-1M DCA deep-mode lane and handoff escalation, both major original features.
- [E003-E32-040][CON][OBS/TEST][lines 46,62,71-74,200] The orientation summary invents context_lifted_32k and fitm_recall_lift_measured events and a 15pp target; the original events are FitmCalibrationApplied/DeepModeActivated/DeepModeEscalationChosen/YarnRefused and its deployment gate is measured +5pp on ELAI data.

## Per-file ledger: E-44_checkpointing_durability.md

Source anchors below refer to E-44_checkpointing_durability.md.

### Explicit atomic ledger

- [E003-E44-001][EXP][CAP][lines 1-22] The durability layer adds idempotent tool boundaries, pre-effect SAGA compensation, Continue-As-New, deterministic simulation, lazy migration, compaction locking, optional postcard, reliability metrics, and error taxonomy.
- [E003-E44-002][EXP][DEP/STATUS][lines 3-6,24-31] Effort is three sessions; B-9/B-40/B-42/B-41 are shipped prerequisites and E-41/E-42/E-43/E-45 are parallel consumers.
- [E003-E44-003][EXP][DATA/DET][line 50] IdempotencyKey.compose deterministically builds session_id:phase_id:step_id:tool_name:call_seq and is sortable.
- [E003-E44-004][EXP][WF][line 50] Every mcp_* and Bash/Edit/Write wrapper accepts the key and checks AgentCheckpoint.tool_results before a real call.
- [E003-E44-005][EXP][DET/FAIL][line 50] A same-key irreversible call replays its cached result; a same-key call with no irreversible SAGA effect may fork/re-execute.
- [E003-E44-006][EXP][OBS][line 50] Cache/replay paths emit tool_idempotency_hit or tool_idempotency_replayed_irreversible.
- [E003-E44-007][EXP][DATA][line 52] SSB gains compensation_actions columns session_id, phase_id, action_id, action_name, inverse_action_name, inverse_args_msgpack, written_at, executed_at, and status.
- [E003-E44-008][EXP][SEC/DET][lines 14,52] Compensation must be durable before an irreversible side effect, never after it.
- [E003-E44-009][EXP][DATA][line 52] File delete records write_back with previous bytes; git push records revert/reset to previous SHA; pod termination records respawn with previous config.
- [E003-E44-010][EXP][FAIL/WF][line 52] Crash recovery scans committed compensations whose phase is incomplete and replays inverse actions in reverse order.
- [E003-E44-011][EXP][OBS][line 52] SAGA emits compensation_logged, compensation_replayed, and compensation_failed.
- [E003-E44-012][EXP][CFG][line 54] SessionDag ceilings are tunable as event_count_ceiling default 500 and phase_count_ceiling default 100.
- [E003-E44-013][EXP][WF/DATA][line 54] Crossing either ceiling atomically snapshots carry-over, archives old content under session_dag_archive/<old>.archive keyed by blake3, and starts a new root.
- [E003-E44-014][EXP][OBS][line 54] Rollover emits session_dag_continue_as_new referencing old and new session IDs.
- [E003-E44-015][EXP][DET/TEST][line 56] agent_pool_dst swaps tokio for madsim only in tests, injects crashes at each supervise await, and checks the three-restarts-per-N-seconds Erlang limit.
- [E003-E44-016][EXP][SEC/DEP][line 56] madsim is an optional dev dependency whose version must be verified with cargo search.
- [E003-E44-017][EXP][DATA/WF][line 58] checkpoint_migrator registers transforms by (from,to), reads msgpack schema_version on load, applies older transforms in order, and never performs a big-bang migration.
- [E003-E44-018][EXP][DATA][line 58] Declared transform chain is 40→41, 41→42, 42→44, and 44→45, with gaps confirmed from shipped project records.
- [E003-E44-019][EXP][OBS][line 58] Migration emits checkpoint_lazy_migrated with from, to, and file_path.
- [E003-E44-020][EXP][DET/CONCURRENCY][line 60] PreCompact and the 85%-token trigger acquire a cross-platform CompactLock at workflow/sessions/{id}/.compact-lock before changing SESSION_HANDOFF or SSB.
- [E003-E44-021][EXP][OBS][line 60] Waiting more than 100ms for the compact lock emits compact_lock_contended.
- [E003-E44-022][EXP][CFG/STATUS][line 62] postcard_checkpoints switches the five on-disk files only behind a feature; msgpack remains default and remains mandatory at dashboard IPC.
- [E003-E44-023][EXP][TEST/OBS][line 62] checkpoint_format benchmark records msgpack/postcard size and serde latency in E-44-checkpoint-format-bench-2026-05-19.md without changing the default.
- [E003-E44-024][EXP][OBS/DATA][line 64] AgentPool computes RDC, VAF, graceful_degradation_score, and meltdown_onset_point over rolling windows of 10/50/100 calls.
- [E003-E44-025][EXP][WF/OBS][line 64] agent_reliability_metrics feeds phase-end audit; detected meltdown recommends escalation or restart.
- [E003-E44-026][EXP][DATA/FAIL][line 66] error_taxonomy deterministically classifies transient, semantic, and spec-level ToolErrors into RecoveryDirective.
- [E003-E44-027][EXP][FAIL/WF][line 66] Transient errors retry with backoff, semantic errors trigger step replan, and spec-level failures escalate the phase.
- [E003-E44-028][EXP][OBS][line 66] Classification emits tool_error_classified with bucket and retry_path.
- [E003-E44-029][EXP][TEST/DET][line 68] saga_idempotency_e2e injects a crash between compensation and effect, recovers, reuses the key, and asserts no double file write, no duplicate SSB row, and unchanged SessionDag root.
- [E003-E44-030][EXP][DATA][lines 72-91] Produced functions include IdempotencyKey compose/lookup, SagaLogger log/replay, continue_as_new, migrate, CompactLock acquire, classify, compute_metrics, and ten action types.
- [E003-E44-031][EXP][WF][lines 92-101] task_runner wraps every dispatch; AgentPool consumes DST/supervision; live_cli’s existing handoff path reaches rollover/migration.
- [E003-E44-032][EXP][CONSTR][lines 102-109] Stub bans forbid production unwrap, blake2/sha2, synchronous hot-path checkpoint writes, big-bang migration, post-effect compensation, random retry thread IDs, and removal from the append-only migration registry.
- [E003-E44-033][EXP][INT/STATUS][line 110] Optional DurabilityPanel, gated behind B-26, shows compensation state, idempotency hit rate, reliability metrics, and last rollover.
- [E003-E44-034][EXP][OBS/TEST][lines 111-114] The crash smoke’s evidence order is compensation_logged before the call, compensation_replayed on recovery, then tool_idempotency_hit, with no double write.
- [E003-E44-035][EXP][TEST][lines 116-158] Deterministic checks verify all modules/functions/events, blake3-only/no-unwrap constraints, dev-only madsim, and the measured postcard artifact.
- [E003-E44-036][EXP][TEST][lines 160-174] Acceptance requires universal wrapper coverage, pre-effect ordering, rollover after 500, v40→v45 load, contention behavior, seed-replayable DST, non-default postcard evidence, metrics/error routing, zero double effects, confirmed dependencies, and blake3.
- [E003-E44-037][EXP][TEST][lines 175-187] Tests cover key replay/fork, SAGA ordering/recovery, 501-event rollover, migration, lock contention, DST restart limit, error buckets, full e2e, format benchmark, and durability contracts.
- [E003-E44-038][EXP][STATUS][lines 189-204] Six runtime modules, benchmarks/audits/decisions, and optional panel are created; SessionDag, SSB/schema, task_runner, and Cargo config are modified.
- [E003-E44-039][EXP][SEC/FAIL][lines 208-210,227] Three cited arXiv IDs require verification; if ACRFence is invalid, grounding falls back to verified durable-execution evidence rather than fabricating certainty.
- [E003-E44-040][EXP][CFG/OBS][lines 228-229] The 500/100 thresholds are provisional and should be set where p99 SQLite writes exceed 50ms; WAL initially records every state mutation.
- [E003-E44-041][EXP][FAIL/STATUS][line 230] If madsim conflicts with the workspace MSRV, DST is postponed rather than blocking the rest of E-44.
- [E003-E44-042][EXP][CONSTR][line 231] Postcard remains file-only and msgpack IPC-only; the serialization boundary must be documented.
- [E003-E44-043][EXP][DATA/CONSTR][line 232] Missing schema ordinals use None/gap transitions; the migrator must not falsely claim every version has a transform.
- [E003-E44-044][EXP][STATUS/DEFERRED][lines 235-237] Archive-search CLI is out of scope; reliability metrics later feed skill pruning, and E-41’s irreversible PR actions consume SAGA rollback.

### Inferred, ambiguous, and conflicting

- [E003-E44-045][INF][DET][lines 50-52,68] Exactly-once external effects are approximated by idempotency plus SAGA; the source does not claim a distributed atomic commit across remote tool servers.
- [E003-E44-046][AMB][DATA][lines 58,232] The explicit transform 41→42 calls 42 a reservation while the known shipped sequence jumps 41→44→45; whether 41→42 is a real transform or a no-op registry edge requires source-record confirmation.
- [E003-E44-047][CON][OBS][lines 82-91] The orientation summary’s idempotency_replay, saga_compensated, and continue_as_new event names are not the original contract; exact source names are ToolIdempotencyHit/ToolIdempotencyReplayedIrreversible, CompensationLogged/Replayed/Failed, and SessionDagContinueAsNew.
- [E003-E44-048][CON][CAP][lines 56-66] The orientation summary omits compact locking, postcard measurement, reliability-decay metrics, and three-bucket recovery, all explicit major/minor source features.

## Per-file ledger: E-54_eval_corpus_design.md

Source anchors below refer to E-54_eval_corpus_design.md.

### Explicit atomic ledger

- [E003-E54-001][EXP][CAP][lines 1-10] E-54 replaces D-6’s 9-gram-only plan with 13-gram plus semantic filtering, sharded storage, time cutoffs, executable test triplets, evaluation-awareness probes, and family orthogonality.
- [E003-E54-002][EXP][DEP/STATUS][lines 3-6,12-21] Effort is three sessions; D-6/D-7/D-23/D-24 are planned surfaces that E-54 replaces or augments, D-28/D-30 are deferred, and E-9/E-49/E-55 require dataset/metric/gate coordination.
- [E003-E54-003][EXP][STATUS][lines 14-17] D-6 retains time-window framing but replaces its contamination/storage/schema implementation; D-7 gains KOR checks, D-23 matched/perturbed pairs, and D-24 semantic/test/performance-delta hooks.
- [E003-E54-004][EXP][DET/DATA][line 54] Ingest normalizes task text by lowercasing, removing punctuation, and collapsing whitespace before generating 13-grams.
- [E003-E54-005][EXP][CFG/SEC][line 54] corpus.ngram_threshold defaults to n=13 and k_consecutive=4; overlap beyond four consecutive grams rejects before storage.
- [E003-E54-006][EXP][CAP/CFG][line 54] A second embedding pass uses configurable BGE-M3 or Nomic Embed v2, default BGE-M3.
- [E003-E54-007][EXP][SEC/CONSTR][lines 54,138] Cosine similarity >0.85 rejects; the threshold is tunable but defaults .85 and the semantic filter may never be disabled.
- [E003-E54-008][EXP][DET][line 54] Normalized text is blake3-hashed to cache cosine work within a session.
- [E003-E54-009][EXP][WF/OBS][line 54] Both filters run before TOML write and emit corpus_ingest_rejected_ngram or corpus_ingest_rejected_cosine with the offending task ID.
- [E003-E54-010][EXP][SEC/CONSTR][line 54] Synthesis never bypasses the cosine pass because paraphrase can evade n-grams.
- [E003-E54-011][EXP][DATA][lines 56-60] Canonical task storage is held-out/{family}/{YYYY-MM-DD}/{task-id}.toml.
- [E003-E54-012][EXP][DATA][line 58] Each held-out/_index/{family}.jsonl row contains id, path, created_at, cutoffs, embedding_hash, and ngram_hash.
- [E003-E54-013][EXP][DATA][line 59] Task families are code-fix, math-proof, prose-reason, planning, and knowledge-qa.
- [E003-E54-014][EXP][DET/CONCURRENCY][line 60] The task TOML is written before an atomic append to the family index using append semantics.
- [E003-E54-015][EXP][CFG/OBS][line 60] Daily family/date shards cap at 500 tasks per directory; the first task in a shard emits corpus_shard_created.
- [E003-E54-016][EXP][DATA][lines 62-65] Canonical Task requires id matching filename, one of five TaskFamily variants, and UTC created_at.
- [E003-E54-017][EXP][DATA/CFG][line 66] cutoffs is mandatory model_id→YYYY-MM time-window metadata.
- [E003-E54-018][EXP][DATA][lines 67-68] problem and reference_solution are mandatory prompt/answer fields.
- [E003-E54-019][EXP][DATA/TEST][line 69] tests is mandatory and non-empty; ExecutableTest has input, expected_output, and kind UnitTest|EndToEnd|PropertyTest.
- [E003-E54-020][EXP][DATA/SEC][line 70] provenance records generator, optional prompt_id, and UTC date to support generator rotation.
- [E003-E54-021][EXP][DATA][line 71] Optional FramingPair contains eval and deploy strings for D-23.
- [E003-E54-022][EXP][DATA/SEC][line 72] Optional PerturbedPair contains original_true, subtly_false, and expected AgreeWithTrue|RejectFalse behavior.
- [E003-E54-023][EXP][DATA/DET][lines 73-74] embedding_hash is blake3 of normalized text and ngram_hash is blake3 of canonical 13-gram bytes.
- [E003-E54-024][EXP][FAIL/OBS][line 75] Missing or empty tests cause loader rejection and corpus_schema_violation_no_tests.
- [E003-E54-025][EXP][TEST/SEC][line 77] Every new task executes all tests against reference_solution in a sandbox and rejects with corpus_test_fails_on_reference if any fail.
- [E003-E54-026][EXP][TEST/SEC][line 77] The tests also run against a one-line-corrupted solution; if no test fails, corpus_test_useless_passes_broken rejects the weak test set.
- [E003-E54-027][EXP][FAIL/STATUS][lines 77,337] If no sandbox exists, a registered, babystep-tracked sandbox stub is permitted until a follow-up, rather than silently pretending isolation.
- [E003-E54-028][EXP][DET/TEST][line 79] KOR validation trains a tiny adapter on every family-A train split and evaluates every family-B test split.
- [E003-E54-029][EXP][CFG/FAIL][line 79] Transfer above family-specific random baseline +10pp aborts D-7 canary build and emits kor_orthogonality_violated with the pair.
- [E003-E54-030][EXP][OBS/STATUS][lines 79,115,340] Passing pairs can emit kor_orthogonality_passed; the ten-pair matrix is cached in held-out/_audit/kor_matrix_{date}.json with daily refresh and bounded GPU work.
- [E003-E54-031][EXP][SEC/WF][line 81] For every FramingPair, the auditor invokes identical underlying content twice under eval and deploy framings.
- [E003-E54-032][EXP][SEC/CONSTR][lines 81,143] Deploy framing may not contain benchmark, evaluation, or test markers.
- [E003-E54-033][EXP][DET/OBS][line 81] Absolute eval-vs-deploy score gap >10pp emits sycophancy_framing_gap_detected for human review.
- [E003-E54-034][EXP][OBS][line 81] Significant matched-pair latency or token-count deviation emits sandbagging_anomaly.
- [E003-E54-035][EXP][STATUS][line 81] E-54 ships the matched-pair schema/harness; D-23 remains responsible for the full benchmark suite.
- [E003-E54-036][EXP][DATA/CONSTR][line 83] Synthetic expansion requires at least three generators—local LLM, cloud LLM, human-curated import—with at least 30% from each.
- [E003-E54-037][EXP][FAIL/OBS][line 83] Violating generator distribution emits generator_rotation_violated.
- [E003-E54-038][EXP][SEC/WF][line 83] Every 50-task synthesis batch reruns full-corpus decontamination against the previous month’s training data.
- [E003-E54-039][EXP][DET/OBS][line 85] A >Y one-week family improvement triggers a ConStat-style current-vs-reference check and can emit perf_delta_contamination_suspected.
- [E003-E54-040][EXP][FAIL][lines 85,335] If ConStat’s reference formula cannot be verified/practically computed, a two-sigma sudden-jump detector is the explicit fallback.
- [E003-E54-041][EXP][CAP/WF][line 87] knowledge-qa uses daily OKBench-style generation from last-24-hour SearXNG results rather than replaying stored TOML.
- [E003-E54-042][EXP][DATA/DET][line 87] Dynamic tasks are one-shot and not persisted, but query hash, timestamp, and MCQs are logged to held-out/_audit/knowledge-qa-{date}.jsonl.
- [E003-E54-043][EXP][FAIL/OBS][line 339] If SearXNG is unavailable, knowledge-qa falls back to TOML replay and emits dynamic_knowledge_qa_searxng_unavailable.
- [E003-E54-044][EXP][PRIVACY/CFG][line 89] Production traffic grading is explicit-consent opt-in, defaults OFF, and samples every Nth turn with default 1/100.
- [E003-E54-045][EXP][WF/DATA][line 89] A sampled task strips eval markers; thumbs-down plus grader verdict re-enters ingest and is stored under from-prod/{date}/{blake3-turn-hash}.toml.
- [E003-E54-046][EXP][PRIVACY][line 341] Prod-derived audit data retains content hash and timestamp only, no PII, and is purgeable through elai eval-corpus purge-prod.
- [E003-E54-047][EXP][INT][line 91] EvalCorpusPanel displays family counts, last sweep, cosine/13-gram thresholds, and KOR status.
- [E003-E54-048][EXP][INT][line 91] SycophancyMatchedPairPanel displays eval/deploy gaps and sandbagging flags.
- [E003-E54-049][EXP][INT][lines 91,116] Tauri commands are get_eval_corpus_status, get_sycophancy_matched_pair_results, and get_kor_orthogonality_status.
- [E003-E54-050][EXP][WF/DET][line 93] migrate_held_out idempotently converts flat tasks to shards/indexes and reruns 13-gram, cosine, and triplet validation.
- [E003-E54-051][EXP][FAIL/DATA][line 93] Migration failures go to held-out/_quarantine with a reason.
- [E003-E54-052][EXP][DATA/STATUS][line 93] E-54-decisions records n-gram repair, .85 rationale, NTFS sharding, threshold tuning, matched-pair false positives, and ConStat uncertainty.
- [E003-E54-053][EXP][DATA][lines 97-114] Produced API includes ingest_task, canonical Task/family/test/provenance/pair types, shard/index operations, test execution, cross-family transfer, matched pairs, generator verification, perf detection, daily QA, and optional traffic injection.
- [E003-E54-054][EXP][OBS][line 115] Fourteen exact action types are corpus_ingest_rejected_ngram, corpus_ingest_rejected_cosine, corpus_shard_created, corpus_schema_violation_no_tests, corpus_test_fails_on_reference, corpus_test_useless_passes_broken, kor_orthogonality_violated, kor_orthogonality_passed, sycophancy_framing_gap_detected, sandbagging_anomaly, generator_rotation_violated, perf_delta_contamination_suspected, dynamic_knowledge_qa_generated, and prod_traffic_probe_injected.
- [E003-E54-055][EXP][WF][lines 117-133] The crate consumes planned D-6/D-7/D-23/D-24, embeddings/sidecar, blake3, chrono/serde/toml, and is reached from auditor, held-out ingest, shard/index/quarantine/audit paths.
- [E003-E54-056][EXP][CONSTR][lines 134-151] Hard guards ban production unwrap/unregistered stubs, 9-gram-only filtering, disableable cosine, missing tests/cutoffs/pairs, weak reference tests, failed KOR, flat storage, default-on production probes, unaudited dynamic generation, non-blake3 hashes, and single-generator synthesis.
- [E003-E54-057][EXP][OBS/TEST][lines 153-156] The canonical smoke is a semantic paraphrase that passes 13-gram but scores about .92 cosine and is rejected without index growth; quarantine records both IDs/match details.
- [E003-E54-058][EXP][TEST][lines 158-234] Deterministic checks verify filters, absence of 9-gram fallback, shards/index/schema/families, dual test execution, KOR, matched pairs, rotation, perf detector, dynamic audit, default-off traffic, blake3/no-unwrap, and ten smoke/contract passes.
- [E003-E54-059][EXP][TEST][lines 236-256] Acceptance requires all data, filters, orthogonality, pair/perturbation, rotation, detection, dynamic/prod tiers, fourteen events, two panels/three commands, idempotent migration, blake3/no-unwrap, and green additive workspace integration.
- [E003-E54-060][EXP][TEST][lines 258-278] Nineteen tests/contracts cover exact/paraphrase rejection, required schema, strong tests, KOR, 15pp gap, generator failures, 30pp jump, SearXNG audit, opt-in, shards, minimum n, mandatory cosine/tests, no flat layout, deploy wording, and D-23 pairs.
- [E003-E54-061][EXP][STATUS][lines 280-302] A new eval-corpus crate with ten runtime modules, migration bin, nineteen tests, two panels, decisions, README, commands, embedding endpoint, and workspace registration is created or extended.
- [E003-E54-062][EXP][FAIL/CFG][lines 331-335] Open measurements tune cosine at .80/.85/.90, generator count benefit, matched-pair false positives, NTFS at 1K/5K/10K, and ConStat-vs-two-sigma.
- [E003-E54-063][EXP][SEC/DEP][line 336] BGE/Nomic dependency addition needs explicit confirmation; cosine code is proposed behind a default-ON Cargo feature.
- [E003-E54-064][EXP][FAIL/CFG][line 342] Matched pairs double inference cost, so the default budget is ten probes per session and only D-23-tagged tasks invoke them.
- [E003-E54-065][EXP][STATUS/DEFERRED][lines 344-346] D-23/D-24 promote after consuming this crate; later patches tune thresholds/NTFS/false positives, decide ConStat, measure generator diversity, and expand dynamic QA.

### Inferred, ambiguous, and conflicting

- [E003-E54-066][INF][DATA][lines 54,122] If embeddings.py is absent, a sidecar endpoint must be created, but the exact IPC schema and model lifecycle are not specified.
- [E003-E54-067][AMB][CONSTR][lines 138,273,336] cosine_filter_never_disabled is a contract, yet the dependency is feature-gated and can be removed with --no-default-features; whether such a build is allowed in production is unresolved.
- [E003-E54-068][AMB][CONSTR][lines 77,136,337] New modules ban todo/unimplemented, yet missing sandbox explicitly permits a registered TODO stub; the exception is tracked but still contradicts a literal source scan unless the contract exempts registry-backed markers.
- [E003-E54-069][AMB][STATUS][lines 14-17,256,338] The source calls the work a replacement/canonical implementation for planned D surfaces but later calls integration additive and those surfaces unaffected; ownership of future D-6 edits needs coordination.
- [E003-E54-070][CON][OBS][lines 54-60,115] Orientation events ngram13_filtered, eval_aware_flag, and per_family_shard_loaded are not in the original; exact events are the fourteen corpus_* / KOR / framing / rotation / perf / dynamic / traffic actions.
- [E003-E54-071][CON][TEST][lines 60,93,334] The orientation summary’s “load a 5,000-task fixture in under one second” is not an explicit acceptance threshold in the original; the source requires measuring NTFS at 1K/5K/10K and uses ≤500 files per date shard.
- [E003-E54-072][CON][TEST][lines 54,57,260-261] The orientation summary’s specific CodeForces sample gate is not stated; the original gates exact 13-gram matches and a semantic paraphrase that evades 13-gram but is caught by cosine.

## Cross-source interaction ledger

- [INT-E003-001][EXP] E-18, E-26, E-27, and E-32 all edit python/elai_inference/server.py. E-26 explicitly sequences its XGrammar slice after unresolved E-2; E-27 requires coordinated edit windows; E-32 places FITM after the existing processor chain. These are shared-file ordering constraints, not independent edits.
- [INT-E003-002][CON] E-18 treats XGrammar tool-envelope/schema masking as a main deliverable, while E-26 separately defines an XGrammarLogitsProcessor as gated RECON_NEEDED. Implementers must unify or clearly separate the two processor responsibilities to avoid duplicate compilers/caches.
- [INT-E003-003][EXP] Processor order is constrained across plans: E-26 XGrammar masks at the HEAD before DoLa/ETC; E-32 FITM calibrates AFTER DoLa/ETC/steering; E-27 changes weight/cache loading and must not disturb either order.
- [INT-E003-004][EXP] E-26/E-27/E-32 form a runtime ladder: SGLang is an optional role-routed backend, AWQ/FP8 is an initially default-off Sidecar format/cache lane, and 32K/FITM/deep mode changes effective context and escalation policy.
- [INT-E003-005][EXP] E-18 one-shot recovery and E-44 idempotency/SAGA must compose: a retry reuses a stable idempotency key, observes cached irreversible results, and may not double-execute compensation or side effects.
- [INT-E003-006][EXP] E-4 completion evidence can consume E-44 durable action/audit rows and E-12 production-check findings; E-4’s evidence wrapper is the user-visible trust surface for those lower-level records.
- [INT-E003-007][EXP] E-4 structural sycophancy defense is measured by E-9’s four-behavior probes and E-54’s matched eval/deploy and perturbed-pair corpus schema; the three plans respectively implement defense, public/internal scoring, and eval-awareness hardening.
- [INT-E003-008][EXP] E-9’s external suite and E-54’s held-out corpus must coordinate dataset shape: temporal cutoffs, semantic decontamination, test triplets, harness attribution, and low-power statistics determine whether promotion evidence is valid.
- [INT-E003-009][EXP] E-12 and E-18 both enforce supply-chain safety: E-12 scans produced repositories and hooks commits, while E-18 blocks unverified package references before the tool modifies manifests.
- [INT-E003-010][EXP] All nine sources expand SSB evidence. E-44’s schema/migrator and append-only constraints therefore underpin E-4/E-9/E-12/E-18/E-26/E-27/E-32/E-54 action-type additions.
- [INT-E003-011][EXP] Default-OFF/opt-in controls recur: E-26 SGLang needs endpoint plus wire-dark allowlist, E-27 AWQ/FP8 awaits three evidence runs, E-32 FITM awaits +5pp and deep mode needs 24GB, and E-54 production traffic needs explicit consent.
- [INT-E003-012][INF] A single deployment promotion gate could combine E-9 significance, E-54 contamination validity, E-12 production checks, E-27 parity, and E-44 durable replay evidence, but no source specifies one global transaction or precedence rule.

## Orientation-summary additions and corrections

The following records compare the deep originals to 05-E-SUMMARY_INDEX.discovery.md. “Addition” means explicit source behavior absent from the orientation summary; “correction” means the summary materially changes an identifier, status, threshold, or behavior.

### E-4 deltas

- [DELTA-E003-E4-01][ADDITION] Add exact ClaimWithConfidence schema/tier authority, rules.db/ad-hoc pin lifecycle, destructive-action re-anchor, four doom triggers and three intervention choices, three evidence kinds, clarifier heuristic/max-three questions, verbosity duration, Dreyfus expertise default, refusal scope, interrupt restart, honest-correction format, visible execution-state bar, exact tests/stub bans, and deferred persona switching.
- [DELTA-E003-E4-02][CORRECTION] sycophancy_pushback_detected is the source event; sycophancy_detected is not.
- [DELTA-E003-E4-03][CORRECTION] The original does not implement persona_drift_alert or an explicit ten-turn drift detector, and does not define a repair-resistance counter; it uses re-anchor and correction/doom behavior instead.
- [DELTA-E003-E4-04][CORRECTION] The source declares thirteen SSB variants and five slash commands raising 146→151, not only the two summary journal rows.

### E-9 deltas

- [DELTA-E003-E9-01][ADDITION] Add LiveCodeBench v6, BFCL v3, GAIA L1/L2, ClawsBench ANOVA, PRMBench, four sycophancy probes, GSM8K-Platinum+MATH-500, quarterly HLE, DeepPlanning, DS-CP, power calibration, judge rotation, dashboard/history, all numeric floors, low-power behavior, and anti-slopsquatting.
- [DELTA-E003-E9-02][CORRECTION] SWE-Pro is not an explicit suite in the original; the source’s named runner catalog differs.
- [DELTA-E003-E9-03][CORRECTION] The CLI is elai bench external [--suite ...] [--harness-vary] [--p-target ...], not elai benchmark --suite swelive.
- [DELTA-E003-E9-04][CORRECTION] Source events are suite-specific *RunCompleted plus DS-CP/power actions; the summary’s phase_*.toml requirement is not stated in the original.

### E-12 deltas

- [DELTA-E003-E12-01][ADDITION] Add all seven manifest names/keys/bans, eight FrameworkProfile variants and three trait methods, all fifteen check algorithms/timeouts, fifteen finding categories, scope-planner flow, atomic linter stripping, server seeds, five config tables, dual secret hook, per-level banned-pattern checks, assertion-only diff, framework dashboard payload, degraded-no-sandbox mode, and test matrix.
- [DELTA-E003-E12-02][CORRECTION] Task text says “5” manifests but consistently enumerates/accepts seven; seven is the operative source requirement.
- [DELTA-E003-E12-03][CORRECTION] production_check_failed, rls_probe_missing, and biome_deny_violation are not declared source SSB action types; the original explicitly names legacy_config_stripped, framework_profile_selected, and finding categories.
- [DELTA-E003-E12-04][CORRECTION] Checklist cardinality is not simply “15”: the source also describes 12 base +3-6 profile-specific and a UI with 15 or 18 rows.

### E-18 deltas

- [DELTA-E003-E18-01][ADDITION] Add boot precompile/fail-loud behavior, 100-prompt Hermes fuzz, exact fences and collision fallback, error-embedded one-shot/fixer escalation, 8K spill schema, 30-day registry age rule, token-level path enum, five-frame blake3 dedupe, AgentDrawer counters, .50 eval target, and CodeAct/IPC/supervisor deferrals.
- [DELTA-E003-E18-02][CORRECTION] xgrammar_constraint_enforced and search_replace_fenced are not declared action types; exact source events are the eight Tool*/Package*/Path*/Progress*/Xgrammar variants.
- [DELTA-E003-E18-03][CORRECTION] The original does not promise arbitrary malformed JSON “reshape” or universal first-attempt landing; grammar prevents malformed output and one structured retry is permitted.

### E-26 deltas

- [DELTA-E003-E26-01][ADDITION] Add HTTP endpoints/config/backend enum, permission/factory routing, env-gated backward compatibility, distinct media/runtime configs, exact health/cold-start evidence, static config, aborts, off-agent verification, red-team gates, cross-seam smokes, wire-dark allowlist, and PS-4 feed status.
- [DELTA-E003-E26-02][CORRECTION] XGrammar S5 is RECON_NEEDED, blocked on E-2 Part-3 owner resolution and RunPod; it is not an immediately shippable free-lane task.
- [DELTA-E003-E26-03][CORRECTION] Ollama is retained and remains factory-resolvable with a once-only warning; it is not skipped or deleted.
- [DELTA-E003-E26-04][CORRECTION] Exact events are sglang_backend_registered, xgrammar_schema_compiled, and runtime_cold_start_observed; sglang_health_changed/ollama_deprecated are summary inventions.

### E-27 deltas

- [DELTA-E003-E27-01][ADDITION] Add all env/default/fallback semantics, five VRAM profiles, five policy rules, code-aware AWQ build, alternate boot, parity/long-context CSV tests, three-run telemetry rollout, exact events, hardware artifacts, and deferred formats.
- [DELTA-E003-E27-02][CORRECTION] nf4/fp16 remain the default path initially; AWQ/FP8 are default-off and only a later patch promotes 12GB+ after three consecutive passes.
- [DELTA-E003-E27-03][CORRECTION] The live acceptance target is ≥3× plus parity, not a guaranteed 4.4×; 4.4× is cited external evidence.
- [DELTA-E003-E27-04][CORRECTION] awq_loaded, fp8_kv_active, and nf4_qlora_only are not source events; exact events are QuantBackendSelected, KvCacheDtypeSoftwareSim, and AwqCheckpointBuilt.

### E-32 deltas

- [DELTA-E003-E32-01][ADDITION] Add per-request FITM override/dummy/cost/order, opt-in Qwen-1M DCA deep mode, 24GB refusal/fallback, cross-file handoff escalation, loud YaRN error, dynamic effective-max threshold, RuntimePanel telemetry, and A-MEM/PINE deferrals.
- [DELTA-E003-E32-02][CORRECTION] This is a documentation/effective-policy ceiling lift from 20K production/25K hard guidance; runtime code’s 131,072 default is intentionally unchanged.
- [DELTA-E003-E32-03][CORRECTION] FITM’s rollout gate is ≥+5pp recall@3 on ELAI traces, not a guaranteed +15pp.
- [DELTA-E003-E32-04][CORRECTION] context_lifted_32k and fitm_recall_lift_measured are not source events; the source declares FitmCalibrationApplied, DeepModeActivated, DeepModeEscalationChosen, and YarnRefused.

### E-44 deltas

- [DELTA-E003-E44-01][ADDITION] Add exact key composition/replay-or-fork, compensation table/three inverse examples, two rollover ceilings/archive, madsim crash schedule, migration chain, cross-platform compact lock, feature-gated postcard benchmark, RDC/VAF/GDS/MOP, three error buckets, and full crash e2e.
- [DELTA-E003-E44-02][CORRECTION] Exact source event names differ from idempotency_replay/saga_compensated/continue_as_new; see E003-E44-030/E003-E44-047.
- [DELTA-E003-E44-03][CORRECTION] Postcard is measurement-only and not default; msgpack remains the dashboard IPC and checkpoint default.
- [DELTA-E003-E44-04][CORRECTION] 500 events/100 phases are provisional defaults to be tuned at p99>50ms, not immutable durability constants.

### E-54 deltas

- [DELTA-E003-E54-01][ADDITION] Add normalization/K=4 algorithm, mandatory non-disableable semantic pass, complete shard/index schema, five families, canonical task/test/provenance/pair/hash schema, dual test execution, KOR LoRA matrix, sandbag telemetry, generator rotation/full rescans, ConStat fallback, daily ephemeral QA, consent/privacy/purge, dashboards, migration/quarantine, fourteen actions, and all contracts/risks.
- [DELTA-E003-E54-02][CORRECTION] ngram13_filtered, eval_aware_flag, and per_family_shard_loaded are not source events; exact actions are listed at E003-E54-054.
- [DELTA-E003-E54-03][CORRECTION] The source does not require loading 5,000 tasks in under one second; it requires sharding to ≤500 per date directory and measurement at 1K/5K/10K.
- [DELTA-E003-E54-04][CORRECTION] The source smoke is a semantic paraphrase that passes 13-gram and fails cosine, not a specifically named CodeForces acceptance fixture.
- [DELTA-E003-E54-05][CORRECTION] 13-gram is only a baseline; the source explicitly says even 13-gram can be evaded and therefore makes cosine mandatory.

## Zero-omission and zero-duplicate audit

Read coverage:

| Source | Lines read | Atomic IDs | Sequence audit |
|---|---:|---:|---|
| E-4_chat_ux_critique.md | 1-230 | 47 | E003-E4-001..047 contiguous |
| E-9_agent_eval_benchmarks.md | 1-254 | 50 | E003-E9-001..050 contiguous |
| E-12_production_fullstack_codegen.md | 1-193 | 48 | E003-E12-001..048 contiguous |
| E-18_tool_use_reliability.md | 1-207 | 38 | E003-E18-001..038 contiguous |
| E-26_inference_engines_2026.md | 1-265 | 45 | E003-E26-001..045 contiguous |
| E-27_quantization_2026.md | 1-195 | 37 | E003-E27-001..037 contiguous |
| E-32_context_window_extension.md | 1-214 | 40 | E003-E32-001..040 contiguous |
| E-44_checkpointing_durability.md | 1-237 | 48 | E003-E44-001..048 contiguous |
| E-54_eval_corpus_design.md | 1-346 | 72 | E003-E54-001..072 contiguous |
| **Total** | **2,141** | **425** | **no gaps; no duplicate atomic IDs** |

Audit assertions:

- Files processed: 9/9 from the exact manifest; source lines processed: 2,141/2,141.
- Hash/line/byte verification: 9/9 exact; aggregate 249,792 bytes exact.
- Atomic ledger: 425 IDs; 425 unique; zero duplicate IDs; all nine per-source sequences contiguous from 001.
- Requested dimensions covered explicitly: capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failure/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred.
- Classification separation: every atomic entry is tagged EXP, INF, AMB, or CON; inferred/ambiguous/conflicting entries are isolated after each explicit ledger.
- Repetition audit: Tasks, WIRING_CONTRACT, deterministic checks, acceptance, tests, files, risks, and next/deferred sections were reconciled into one ID per behavior with all relevant anchors.
- Orientation audit: every material summary addition/correction found in the deep read is recorded in DELTA-E003-* above.
- Original-scope audit: no original outside DEEP-E-003 was mined; the family discovery summary was orientation-only.

Canonical output SHA-256: 65d8217bfbd8a7fdbe031e882a6263ff18560385a55bebeb9bb9bdc2f4123c45

Canonical-hash rule: compute SHA-256 over the final UTF-8 bytes after replacing only the 64 hexadecimal characters on the Canonical output SHA-256 line with 64 zeroes. This avoids an impossible self-referential raw-file digest while yielding a stable verifiable output hash.
