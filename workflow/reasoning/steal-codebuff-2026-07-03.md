# steal-codebuff — 2026-07-03

Source: C:\Users\user\Downloads\codebuff-main (local clone; Apache-2.0; Bun+TS monorepo). Map read: CODEBASE-MAP.md (user-supplied, generated 2026-07-03). Source files read this session: evals/buffbench/{pick-commits.ts,judge.ts,lessons-extractor.ts,runners/claude.ts}, agents/context-pruner.ts, agents/base2/{base2.ts,base2-free-glm.ts}, common/src/types/gravity-index.ts (grep). ELAI side: ELAI-FEATURE-MAP.md (header + 1.3/1.9/1.11-1.14 + Section 2/3/4 ledger), L-SUMMARY_INDEX.md, L-0_INDEX.md, PLAN-FOLDER-TEMPLATE.md. Run shape: paper-to-plan, 0 agents (repo local + map supplied; user-directed direct reads).

What codebuff is: an open-source multi-agent coding CLI (specialized agents: file-picker/planner/editor/reviewer/researcher; OpenRouter any-model; agent store; SDK). Freebuff = the ad-supported free tier on open models (DeepSeek/Kimi/GLM/MiMo/MiniMax). Repo-claimed eval: 61% vs Claude Code 53% on 175+ tasks via their own buffbench harness — REPO-CLAIMED, unverified, tier-2; design-rationale only, never citable as an ELAI number (HD-1/ID-5).

META-VERDICT: codebuff is ~80% CONFIRMATION of ELAI's harness>model thesis (cheap models + orchestration beating a frontier single-agent CLI is exactly the RLM-wave finding, Section 4 RLM verdict block). ELAI's per-mechanism coverage is equal or stronger on every orchestration axis codebuff has. The two genuinely stealable assets are both in evals/: the commit-mined task supply and the competitor-arm runner shape — the measurement machinery, not the agent machinery.

## Verdicts (checked against the Section 4 REJECTED/ALREADY-TRIED ledger before every NEW/EXTENSION)

S1 commit-derived eval task synthesis — NEW as EXTENSION(J-56/E-54/G-6/I-38). evals/buffbench/pick-commits.ts + eval-task-generator.ts + setup-test-repo.ts: scan a real repo's git history, basic filter, LLM screen for HARD commits (explicit reject list: deps/formatting/docs/merges), task prompt derived from the commit, parent tree = workspace, the REAL diff = ground truth. ELAI's living corpus (J-56) ingests CAUGHT FAILURES only; E-54 has the contamination filter + per-family shards but no task-supply generator; nothing in any letter mines commit history for eval rows. Contamination-resistant by construction on private repos. ELAI port: deterministic filter legs first, LLM screen ADVISORY-tier only (TD-3), rows enter the J-56 corpus via `elai eval ingest`. -> L-20 leg A.

S2 competitor-arm paired runner — NEW as EXTENSION(over the WIRED agent-CLI bridges + I-38/E-49; sibling of C-51/C-54/C-59 which target public suites). evals/buffbench/runners/{claude,codex,opencode}.ts: spawn the vendor CLI (`claude -p --output-format stream-json --dangerously-skip-permissions`), parse stream-json, capture steps + cost, score all arms on the SAME tasks. ELAI ALREADY SHIPS the spawn+parse mechanism as the agent-CLI bridges (cli_common.rs:330 spawn_isolated_with_event, claude_code.rs hand-parsed stream-json, WIRED, Tier-C, broader than goose) — buffbench proves the missing composition: point the bridges at the bench as ARMS, pair with the ELAI arm under an identical envelope, I-38 paired floor (>=5 seeds), E-49 $/CTS per arm. GD-3 BINDS: the output is an internal paired delta under a published reproducible envelope, never a leaderboard-rank claim (SWE-bench-headline DROPPED, ledger); any public use of the number gates on KD-1. ELAI's port drops --dangerously-skip-permissions semantics for competitor arms only inside the disposable bench workspace (J-80 pool), never for the ELAI arm. -> L-20 leg B.

S3 cache-expiry-aware prune timing — ALREADY-PLANNED(L-17). agents/context-pruner.ts:378-408 prunes not only on overflow but when the prompt cache will MISS anyway (gap > 5min TTL): compaction is free exactly when the cache is cold. That is L-17's Recompute(ColdGapTtl) cause, verbatim concept. Codebuff = convergent external prior art for L-17; recorded as a rider (strengthens the L-T3 case), no new unit.

S4 deterministic context-pruner mechanics — CONFIRMATION + riders on I-14/K-12/L-16. context-pruner.ts is an "agent" with ZERO LLM calls: a fully deterministic handleSteps fold (80/20 head-tail truncation, tool-call -> one-line summaries, think-tag strip, spawn-output blacklist by agent role, ASYMMETRIC budgets user 50K >> assistant/tool 20K, mid-turn vs turn-boundary prune with a synthetic continuation message). Aligns with TD-3/IC-2/HD-2. ELAI's L-16 fold-view is architecturally stronger (lossless store / lossy view; codebuff's set_messages DESTROYS raw history). Riders worth folding into I-14/L-16 strategies at edit: the asymmetric user-over-assistant budget split; the mid-turn continuation-prompt shape.

S5 per-(role,model) agent prompt variants — ALREADY-SHIPPED(I-36 DARK, default-ON kill-switch). agents/base2/* one file per model/tier + reviewer per model; but base2-free-glm.ts shows they are thin parametric wrappers (createBase2('free',{model})), i.e. weaker than I-36's compiled per-(role,backend) store + transfer-gap ledger. Confirmation.

S6 best-of-n editor/thinker — ALREADY-SHIPPED(I-4 evidence-ranked search ladder, DARK->promotion). Ledger: MCTS-first REJECTED, ladder stands; batch-racing REJECTED (IC-1). Confirmation.

S7 tool-call stream parsing for weak models — ALREADY-PLANNED(J-53 tool-input-repair; E-35/E-18 XGrammar; I-36 minimal schemas; J-55 constrained-call). agent-runtime tool-stream-parser.ts + partial-json is the prompt-format leg; ELAI's grammar-masked leg is strictly stronger on the local lane. Confirmation.

S8 weak-lane mandatory verification fan-out — NEW as EXTENSION(J-51/J-55/C-32/H-7/B-41). agents/base2/base2.ts free mode: per-model reviewer selection (FREEBUFF_REVIEWER_AGENT_ID_BY_MODEL data map), review MANDATORY ("You must spawn a <reviewer> ... in parallel with typechecking or testing") + parallel typecheck/full-typecheck/test fan-out mid-turn; default mode makes review skippable. The principle — verification intensity scales INVERSELY with generator capability, as a data-driven policy — has no ELAI unit: H-7 routes WHO, J-55 routes the ACTION SURFACE, J-51 gates ROLE ELIGIBILITY, C-32 has a fixed retry loop; none scales gen-time verification intensity per model profile. -> L-21.

S9 lessons-extractor — EXTENSION-rider folded into L-20 leg C. evals/buffbench/lessons-extractor.ts: per failed task, an LLM extracts {whatWentWrong, whatShouldHaveBeenDone} pairs from the trace + judge output. ELAI analogs: J-10 insight backprop (audit reports), I-12 curator, VCG J-96/J-97. The bench-context variant (mine lessons from failed BENCH arms) is a small observe-only leg: advisory rows journaled for J-10/I-12 consumption; H-31 firewall (never into the auditor); K-19 SF-1 execution-grounded fitness binds any promotion.

S10 LLM judge scoring 0-10 vs ground truth — REJECT as a gate; ADVISORY-tier only (TD-3 "LLM is T3-advisory never the load-bearing gate"; Section 4 "LLM-judge-on-a-gate" REJECTED rider from the Five-Pillars run). L-20's deterministic legs (build/test/wiring-check + I-38 paired stats) are the gate; a judge row may be journaled advisory.

S11 set_messages / add_message (agent rewrites its own history) — REJECT. Violates L-T3 CC-1 (raw transcript immutable) + GD-2 journal-as-record. Codebuff's pruner destroying raw history is the exact failure L-16 exists to prevent.

S12 agent store / publish + "agents are the new MCP" — REJECT. ToxicSkills landscape + skill-import-policy (E-43); ELAI's safe subset is L-12 (skill front-door, skillspector-gated) + L-13 (rules-as-DATA). A code-carrying agent marketplace is the #1 injection surface.

S13 handleSteps user-programmable generator agents (QuickJS-WASM sandboxed TS) — REJECT for the user surface (FS-2 DATA-not-code). The deterministic-control need is covered: adaptive_tool journaled named-ops, J-15 conductor exit-code script-steps, BSP supersteps. Sandboxing user code well (their quickjs choice) is noted as prior art for J-63's Tier-C posture only.

S14 OpenRouter any-model routing — ALREADY-SHIPPED(B-37 11 backends)/PLANNED(J-30 BYOK, J-54 free pool, K-24 rail). Dynamic model registry ALREADY-REJECTED (I-51, reaffirmed twice). Confirmation.

S15 ad-supported free tier + streaks — REJECT (GTM verified-not-cheap; K-24 metered credits; no ads surface, no gamification).

S16 gravity_index (their hosted repo-index/search tool) — ALREADY-SHIPPED (the whole 1.1 retrieval stack; PageRank repo map WIRED). Confirmation.

S17 OpenTUI React terminal UI — REJECT (Track-20 D1 precedent: TUI = product call; ELAI has REPL + dashboard-v2).

S18 the 61%>53% cheap-model result itself — CONFIRMATION of the harness>model thesis; repo-claimed, never cited as evidence.

## Outcome
L-20 (commit-mined eval supply + competitor-arm paired harness + lessons leg) + L-21 (weak-lane verification-intensity policy) authored under L-T4; indexes + feature map upserted. Riders recorded on L-17 (ColdGapTtl prior art), I-14/L-16 (pruner mechanics), J-63 (quickjs sandbox prior art). No new deps, no new letters, net 0..+2 SSB prefer-reuse.
