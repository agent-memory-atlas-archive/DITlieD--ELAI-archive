# DEEP-I-002 — deep feature discovery

## Scope and evidence protocol

This artifact deep-reads exactly the seven files assigned to `DEEP-I-002`. The auxiliary grounding map is treated as first-class evidence, not as metadata. `09-I-SUMMARY_INDEX.discovery.md` was consulted only to orient the comparison in **Summary additions and corrections**; none of its statements are counted as source atoms. No source assigned to another deep chunk was opened.

Evidence labels:

- **[E] Explicit** — stated directly by a processed source.
- **[I] Inferred** — a conservative composition consequence, never promoted to an explicit requirement.
- **[A] Ambiguous/open** — the source leaves a name, threshold, shipped state, or implementation decision unresolved.
- **[C] Correction/conflict** — an explicit correction, contradiction, or binding replacement of another claim.

An atomic ID occurs exactly once. Repeated source statements are consolidated into the first atom that captures the behavior and cited at every applicable line range; a later statement is given a new atom only when it adds a distinct constraint, state, field, gate, or acceptance condition.

## Exact processed manifest and integrity verification

Manifest anchor: `DEEP-SOURCE-MANIFEST.md:793-805`.

| Ordinal | Class | Exact source | Manifest lines | Actual lines | Manifest bytes | Actual bytes | Manifest/actual SHA-256 | Result |
|---:|---|---|---:|---:|---:|---:|---|---|
| 1 | auxiliary/grounding/fixture | `docs/thefinalglance/I-plan/_grounding/I-STEAL-GROUNDING-MAP.md` | 96 | 96 | 7,098 | 7,098 | `852ff3b2dddf6faa1417d85fb4ca32c807ad9864455ac44d09fd4fe8c6ac1048` | exact |
| 2 | numbered/index original | `docs/thefinalglance/I-plan/I-0_INDEX.md` | 603 | 603 | 55,315 | 55,315 | `43b35d0a46695824b1ace99b26d123ee341f6e3e2482f2ce788455b234e25093` | exact |
| 3 | numbered/index original | `docs/thefinalglance/I-plan/I-5_pristine_context_injector.md` | 115 | 115 | 8,303 | 8,303 | `6c1f799dc789166ae317b010c90bde3b638790da3d7226377db51e973551f669` | exact |
| 4 | numbered/index original | `docs/thefinalglance/I-plan/I-10_prefix_invariance_ttl_tiering.md` | 101 | 101 | 9,012 | 9,012 | `ea92c63822a8f99dddd92229ee9ce6e0740bdf38c9d412ea80c42cbbdb10d3b4` | exact |
| 5 | numbered/index original | `docs/thefinalglance/I-plan/I-29_diffusion_draft_lane.md` | 109 | 109 | 9,693 | 9,693 | `ccd141807a2f4a2e9e06a14cca0f1b9a98fc354ad45607d557a14184eb81906e` | exact |
| 6 | numbered/index original | `docs/thefinalglance/I-plan/I-46_bash_interceptor_redirect.md` | 70 | 70 | 4,699 | 4,699 | `76f28dd16ae1a1afd57adb4b4a510dd2340c2bc1ff0ded8024743781ee75c892` | exact |
| 7 | numbered/index original | `docs/thefinalglance/I-plan/I-54_structural_ast_edit_tool.md` | 75 | 75 | 5,130 | 5,130 | `2b0994903bad66b9191a261e10b043ca44ab5f10894e16f969fd757894f8562d` | exact |
| **Total** |  | **7 files** | **1,169** | **1,169** | **99,250** | **99,250** |  | **exact** |

All logical-line, byte, and SHA-256 values were recomputed from the files before extraction. There is no manifest drift.

## Per-file exhaustive ledgers

### 1. `_grounding/I-STEAL-GROUNDING-MAP.md`

Source fully read: 96/96 lines. Role: Track-20 grounding, corrections, and execution-time read rules.

- **I002-GM-001 [E][provenance]** The map covers Track 20 (`I-45..I-54`), was grepped on 2026-06-10 in session `s-iplan-track20-2026-06-10`, and derives from `docs/.source/STEAL-LIST.md`. Anchor: lines 1-4.
- **I002-GM-002 [E][evidence]** The map says `elai.map` staleness was unknown, so every ELAI anchor was confirmed directly by glob/grep in live source; detail files cite `SL-n` IDs instead of restating facts. Anchor: lines 4-6.
- **I002-GM-003 [E][schema][lifecycle]** Live constants were `SCHEMA_VERSION = 221` at `ssb.rs:2981` and `TOTAL_VARIANTS = 532` at `action_type_lock.rs:601`; both must be re-read at edit and never hard-coded. Anchor: lines 12-15.
- **I002-GM-004 [E][capability][status]** `crates/tools/src/bash_validation.rs` exists, while redirect/rewrite absence is a STEAL-LIST assertion that I-46 must re-grep before building. Anchor: lines 17-19.
- **I002-GM-005 [E][dependency]** I-46's mandatory gajae references are `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchor: lines 19-21.
- **I002-GM-006 [E][integration][status]** `crates/providers/src/oauth.rs` exists and is Anthropic/Claude-plan OAuth; the source reports no other subscription-auth broker under `crates/providers/`. Anchor: lines 23-25.
- **I002-GM-007 [E][dependency]** The corresponding gajae auth references are `packages/ai/src/auth-broker/`, `auth-gateway/`, and `auth-storage.ts`. Anchor: lines 25-26.
- **I002-GM-008 [E][data][status]** Model capability/pricing data is scattered across `providers/pricing.rs`, two distinct `vram_tier.rs` modules, and `pricing_rollup.rs`; no single capability catalog exists. Anchor: lines 27-30.
- **I002-GM-009 [E][dependency]** The catalog reference port names `api-registry.ts`, `models.json`, and `model-manager.ts`. Anchor: lines 29-30.
- **I002-GM-010 [E][C][status]** `rra.rs`, `attestation.rs`, and `rra_cmd.rs` are shipped, correcting the stale G-plan memory that `rra.rs` was absent; I-45 is therefore an extension, not a forward dependency. Anchor: lines 32-35.
- **I002-GM-011 [E][dependency]** F-4 `stdout_hash` is the adjacent precedent for I-45. Anchor: lines 34-35.
- **I002-GM-012 [E][status]** E-25 edit checkpoints, its CLI, and both named checkpoint contracts are shipped and grep-confirmed. Anchor: lines 37-40.
- **I002-GM-013 [E][C][scope]** H-49 is planned rather than shipped; I-52 activates the shipped E-25 store as an agent tool instead of creating a second snapshot store. Anchor: lines 39-42, 72-74.
- **I002-GM-014 [E][integration][fallback]** If H-49 ships first, I-52 becomes a tool surface over H-49's store and should reuse its `EditHistoryEvent` row. Anchor: lines 40-42.
- **I002-GM-015 [E][parser][status]** The knowledge crate already pins tree-sitter 0.26 and grammars for Rust, Python, Go, TypeScript, Java, C#, Ruby, JavaScript, and C. Anchor: lines 44-46.
- **I002-GM-016 [E][C][dependency]** The rejection of an `ast-grep` dependency remains binding; I-54 must build only on the shipped grammars. Anchor: lines 45-46, 78-79.
- **I002-GM-017 [E][status]** No Bedrock or Azure implementation exists in `crates/`; the grep is accepted as high-confidence and same-day. Anchor: lines 48-49.
- **I002-GM-018 [E][integration]** Existing provider reach comprises the ten named runtime adapters, local serving, and `oauth.rs`. Anchor: lines 49-51.
- **I002-GM-019 [E][scope]** I-49 adds direct clients only for SigV4+EventStream, Azure Responses, and the OpenAI-as-Anthropic shim seam—protocol shapes a CLI bridge cannot supply. Anchor: lines 51-53.
- **I002-GM-020 [E][status]** No recipe runner exists in `crates/tools/src`; `just`/`makefile`/`recipe` hits were only incidental. Anchor: lines 55-57.
- **I002-GM-021 [E][dependency]** I-53's gajae references are the recipe directory, `archive-reader.ts`, and `json-tree.ts`. Anchor: lines 56-57.
- **I002-GM-022 [E][evidence][constraint]** Gajae behavior is medium confidence because file existence was verified but semantics came from repository summaries. Anchor: lines 59-60.
- **I002-GM-023 [E][compliance][done-gate]** Executors of I-46, I-49, I-50, I-51, and I-53 must read their named gajae sources before execution and rescope if summaries differ; summary-only scope is a W-3 violation. Anchor: lines 60-62.
- **I002-GM-024 [E][evidence][constraint]** Roach-pi/lazycodex semantics are likewise medium confidence. Anchor: lines 64-65.
- **I002-GM-025 [E][compliance]** I-45 and I-47 must read the summary-cited roach-pi extension sources before execution. Anchor: lines 64-67.
- **I002-GM-026 [E][C][architecture]** SLC-2 requires a unified capability catalog to wrap—not delete—the measured 8/12/16/24/48GB VRAM-tier plan; simplification/removal needs explicit user approval. Anchor: lines 75-77.
- **I002-GM-027 [E][schema]** Track 20 plans at most six new variants, from I-45/I-46/I-47/I-50/I-52/I-54; I-48/I-49/I-51/I-53 reuse existing rows. Anchor: lines 82-86.
- **I002-GM-028 [E][schema][done-gate]** Variant bumps must follow I-0 lockstep discipline. Anchor: lines 84-86.
- **I002-GM-029 [E][status]** No web verification was required at map time. Anchor: lines 89-92.
- **I002-GM-030 [E][deferred][security]** I-49 defers current AWS/Azure protocol verification to authoring time; the executor must web-search current docs and anti-slopsquat-check candidate `aws-*` crates before porting. Anchor: lines 91-94.

Grounding-map ambiguity/conflict register:

- **I002-GM-031 [A][status]** Several negative findings are source-asserted rather than independently re-grepped by this map (`redirect` absence, non-Anthropic broker absence, and no capability catalog); the relevant executor re-check is part of the requirement. Anchors: lines 17-30.
- **I002-GM-032 [C][status]** The shipped RRA and E-25 checkpoint facts supersede older “absent/planned-only” memories and change I-45/I-52 from greenfield builds to extension/activation. Anchors: lines 32-42, 72-74.

### 2. `I-0_INDEX.md`

Source fully read: 603/603 lines. Role: corpus doctrine, complete 54-session catalog, schemas, sequencing, execution rules, and status ledger.

#### Corpus, scope, and reading contract

- **I002-IDX-001 [E][scope]** I-plan contains 54 sessions across Tracks 12-20 with no reserved I track. Anchor: line 1.
- **I002-IDX-002 [E][provenance]** Tracks 12-18 were authored from the named consolidated/review/dossier corpus on 2026-06-10. Anchors: lines 3, 19-37.
- **I002-IDX-003 [E][scope][non-goal]** Track 19 adds only design-phase critique, critique-only operation, cheap Phase-0 falsifiers, and zero-retention export; token-efficiency restatements and the already-covered verifier wedge are excluded. Anchor: lines 4-11.
- **I002-IDX-004 [E][non-goal]** GTM/willingness-to-pay and SOC 2 are deliberately not engineering sessions. Anchor: lines 10-11.
- **I002-IDX-005 [E][scope]** Track 20 derives from the STEAL-LIST Tier-1/Tier-2 residue; Tier-3 overlaps are reference material, not duplicated sessions. Anchor: lines 12-17.
- **I002-IDX-006 [E][C][non-goal]** A wholesale gajae provider-layer port is rejected because B-37 plus CLI bridges/OAuth/local serving is broader. Anchors: lines 15-17, 151-165.
- **I002-IDX-007 [E][evidence]** The main grounding map carries `IS-1..IS-19`, `IN-1..IN-14`, `IC-1..IC-18`, and live 221/532 constants; its anchors were re-grepped because `elai.map` was stale. Anchor: lines 43-47.
- **I002-IDX-008 [E][evidence]** The Track-20 map carries `SL-1..SL-11` and `SLC-1..SLC-3`, including shipped-RRA, shipped-checkpoint, wrap-not-delete, activation, and ast-grep-ban corrections. Anchor: lines 48-51.
- **I002-IDX-009 [E][dependency]** I composes with B through H and restates none; track numbering continues H's 1-11 while I session numbering restarts at I-1. Anchor: lines 53-56.
- **I002-IDX-010 [E][workflow]** Every I-session execution must read, in order, its grounding slice; F wiring canon and R1-R9; G deltas; applicable H families; I-0; I summary; then its detail file. Anchor: lines 58-68.
- **I002-IDX-011 [E][done-gate]** Every L/XL compressed detail file must be expanded to full form before execution and every anchor re-grepped. Anchor: lines 66-68.
- **I002-IDX-012 [E][dependency]** Track-specific mandatory reads are declared for Tracks 13/16, 14, 15, 17, 18, and 20. Anchor: lines 69-74.
- **I002-IDX-013 [E][compliance]** Track-20 ports must read named competitor sources; scope based only on repository summaries violates W-3. Anchor: lines 72-74.

#### Track-level behavior

- **I002-IDX-014 [E][architecture]** Track 12 separates pure `Intent -> Directive` reasoning from a sole side-effect supervisor, isolates parallel workspaces, deterministically merges, and hardens SSB/checkpoints. Anchor: lines 78-85.
- **I002-IDX-015 [E][security]** Track 12's supervisor is the structural boundary all later tracks rely on; gates run before side effects and the generator cannot reach gate code. Anchor: lines 79-85.
- **I002-IDX-016 [E][architecture]** Track 13 demotes the LLM to stateless hypothesis generation, gives the orchestrator state-tree ownership, prunes failed branches from context, and ranks by receipts. Anchor: lines 87-92.
- **I002-IDX-017 [E][rollout]** Track 13's MCTS capability must ship as best-of-N, then tournament, then tree; I-5 measures the vividness trap before claiming it. Anchor: lines 89-92.
- **I002-IDX-018 [E][architecture]** Track 14 supplies byte-frozen prefixes, TTL economics, max-token clamping, belief-state handoffs, deterministic compression, per-config measurement, and a receipt for every middleware transform. Anchor: lines 94-100.
- **I002-IDX-019 [E][architecture]** Track 15 composes shipped verification into an in-loop tri-layer veto, bounded mutation, independent contracts, progress rewards, unified calibration, hash chains, and scorer canaries. Anchor: lines 102-109.
- **I002-IDX-020 [E][architecture]** Track 16 trains routing from journaled verifier outcomes, scales budgets by serial depth, stops exploration on marginal verified-pass-per-dollar collapse, instruments the convergence curve, and harvests DPO pairs. Anchor: lines 111-116.
- **I002-IDX-021 [E][rollout][security]** Track 17 performs rejection-sampling SFT before GRPO; GRPO is last and blocked by guards, with all learned weights treated as configuration promotions. Anchor: lines 118-123.
- **I002-IDX-022 [E][architecture]** Track 18 replaces prompt-shaped controls with nine named deterministic mechanisms, including schema roles, decision governance, pinned grammars, sequence rules, compiled prompts, EARS requirements, QDELTA, repo-derived state, and an RRA wrapper/spec. Anchor: lines 125-134.
- **I002-IDX-023 [E][constraint]** Every Track-19 LLM critique surface is advisory and never load-bearing; only the off-agent spine decides DONE. Anchor: lines 136-149.
- **I002-IDX-024 [E][architecture]** Track 20 is primarily extension/activation over shipped code; only I-49 and I-54 are builds and both are binding-read reference ports. Anchor: lines 151-165.

#### Corpus-wide declines and status semantics

- **I002-IDX-025 [E][non-goal]** Declined: voice-first input, Shapley/SHARP before I-28-era RL infrastructure, proactive DTMC without a trace corpus, full TTD-DR, local-lane OPRO optimization, and routine Lean gating. Anchor: lines 167-171.
- **I002-IDX-026 [E][non-goal]** Track 20 declines the wholesale provider layer, redundant rate-limit/backoff, niche clients absent demand, hindsight memory, BM25 tool search, generator-side review, overlapping media tools, agent cron/monitor/jobs, receipt-only returns, duplicate conflict detection, duplicate experiment isolation, TUI port, a standalone lineage session, LSP-as-verifier duplication, and Stop-hook continuation. Anchor: lines 172-182.
- **I002-IDX-027 [E][status]** Valid session states are `PLANNED`, `IN PROGRESS`, `DONE`, and `DEFERRED`. Anchor: lines 184-185.
- **I002-IDX-028 [E][done-gate]** `DONE` means live `wiring_contract_check.py --run-trigger PASS`; cargo-green alone is insufficient. Anchor: lines 184-187.

#### Inherited wiring, determinism, and evidence rules

- **I002-IDX-029 [E][done-gate]** W-1 defines the done gate as the run-trigger wiring check. Anchor: lines 192-193.
- **I002-IDX-030 [E][flags][rollout]** W-2 bans terminal default-OFF env gates; I-29/I-31 may use existing allowlisted kill-switch lanes, while new gates ship default-on with `ELAI_*_DISABLE`. Anchor: lines 194-195.
- **I002-IDX-031 [E][traceability]** W-3 requires every plan claim to map to a grounding-map/source ID. Anchor: line 196.
- **I002-IDX-032 [E][UI][done-gate]** W-4 requires `frontend_pair` and `cross_lang_diff` PASS for dashboard signals. Anchor: line 197.
- **I002-IDX-033 [E][observability]** W-5 requires a live end-to-end `observable_effect`. Anchor: line 198.
- **I002-IDX-034 [E][UI][done-gate]** W-6 makes the dashboard surface part of done unless its absence is justified. Anchor: line 199.
- **I002-IDX-035 [E][evidence]** W-7 requires execution-time and audit-time re-grep of every file/line anchor. Anchor: line 200.
- **I002-IDX-036 [E][scope]** W-8 makes one session one shippable unit with one done gate. Anchor: line 201.
- **I002-IDX-037 [E][schema]** W-9 requires live re-reading of `SCHEMA_VERSION` and `TOTAL_VARIANTS` for every baseline-plus-N claim. Anchor: line 202.
- **I002-IDX-038 [E][rollout]** W-10 ships deterministic in-process legs now and separately gates network/GPU/subprocess legs with deterministic fixtures exercising the real path. Anchors: lines 203-204.
- **I002-IDX-039 [E][architecture]** GD-1 makes reimplementation of a shipped consumed symbol a P0; sessions compose rather than restate. Anchors: lines 206-208.
- **I002-IDX-040 [E][determinism]** GD-2 requires every emitted number to recompute from SSB under `ELAI_SEED`. Anchor: line 208.
- **I002-IDX-041 [E][rollout]** GD-4 forbids default-on before false-positive/over-block rates are bounded on at least three real runs. Anchors: lines 208-209.
- **I002-IDX-042 [E][truthfulness]** GD-5 forbids guarantees on fail-open, RunPod-pending, or cloud-opaque paths. Anchors: lines 209-210.
- **I002-IDX-043 [E][measurement]** HD-1 requires efficiency numbers measured per configuration, never multiplied. Anchors: lines 212-213.
- **I002-IDX-044 [E][cache]** HD-2 makes any byte-stable-prefix cache bust a test failure. Anchors: lines 213-214.
- **I002-IDX-045 [E][generation]** HD-3 says output-token wins are 5x and reasoning roles are never force-JSON encoded. Anchor: line 214.
- **I002-IDX-046 [E][security]** HD-4 makes model-driving gates auditor-driven and off-agent. Anchors: lines 214-215.
- **I002-IDX-047 [E][evidence]** HD-5 distinguishes valid/rendered/claimed artifacts from verified artifacts. Anchor: line 215.
- **I002-IDX-048 [E][dependency]** TD, VD, RD, RV, and CL rule families bind the named I sessions by concern. Anchors: lines 216-219.
- **I002-IDX-049 [E][state][security]** ID-1 gives canonical state to the orchestrator, hard-excludes failed siblings/raw tracebacks/dead hypotheses, permits only a compressed contrastive constraint, and requires measuring the effect. Anchors: lines 221-227.
- **I002-IDX-050 [E][concurrency][security]** ID-2 forbids shared mutable workspaces for branches, permits merges only at deterministic barriers, and blocks root escape. Anchors: lines 228-230.
- **I002-IDX-051 [E][lifecycle][observability]** ID-3 requires a reversible SSB receipt for every context-to-wire transform and exact replay of model-visible bytes. Anchors: lines 231-234.
- **I002-IDX-052 [E][selection][security]** ID-4 restricts selection/pruning/promotion to off-agent evidence; neural scores may order survivors but never revive a vetoed branch, and the SSB records why the winner won. Anchors: lines 235-239.
- **I002-IDX-053 [E][measurement]** ID-5 forbids paper constants as gates, acceptance criteria, or headlines before local engine/corpus measurement; paper values remain `[DESIGN]/[WEAK]` hypotheses. Anchors: lines 240-244.
- **I002-IDX-054 [E][lifecycle][security]** ID-6 treats every learned/tuned artifact as a config promotion requiring the named ledger gates and an evaluator outside the loop's write boundary. Anchors: lines 245-249.
- **I002-IDX-055 [E][done-gate]** COMPLETE requires workspace cargo check, auditor PASS, live wiring-contract PASS, applicable cross-language PASS, shipped/justified dashboard, applicable GD-4 bound, and delegated `memory-writer`. Anchors: lines 251-258.

#### Exhaustive 54-session capability catalog

- **I002-IDX-056 [E][I-1][data]** I-1 defines pure `Intent -> Directive` transitions. Anchors: lines 265-267.
- **I002-IDX-057 [E][I-1][security]** Its supervisor alone executes side effects and does so only after gates; generator reachability to gate code is forbidden. Anchors: lines 265-267.
- **I002-IDX-058 [E][I-1][test]** A contract test proves the “no gate satisfiable from inside the boundary” invariant. Anchors: lines 266-267.
- **I002-IDX-059 [E][I-2][platform]** I-2 uses Linux overlayfs and pooled warm worktrees on Windows. Anchors: lines 268-269.
- **I002-IDX-060 [E][I-2][concurrency]** Branch workspaces are isolated and merge deterministically at a barrier. Anchors: lines 268-269.
- **I002-IDX-061 [E][I-2][extension]** A cloud-sandbox adapter is optional. Anchor: line 269.
- **I002-IDX-062 [E][I-2][dependency]** I-2 composes the BSP, E-15, C-11, and C-23 substrates in Wave 2. Anchor: line 268.
- **I002-IDX-063 [E][I-3][performance]** I-3 hardens SSB pragmas and measures actual insert throughput. Anchors: lines 270-271.
- **I002-IDX-064 [E][I-3][durability]** Checkpoints use `.tmp`/`fsync`/`rename` atomic writes. Anchors: lines 270-271.
- **I002-IDX-065 [E][I-3][flags]** `io_uring` is Linux-gated rather than a universal path. Anchor: line 271.
- **I002-IDX-066 [E][I-3][nature]** I-3 is a small Wave-1 extension of SSB and H-44 durability. Anchor: line 270.
- **I002-IDX-067 [E][I-4][workflow]** I-4 progresses from receipt-ranked best-of-N to tournament to PUCT with virtual loss. Anchors: lines 273-275.
- **I002-IDX-068 [E][I-4][dependency]** It depends on I-2 isolation and I-17 evaluation. Anchor: line 275.
- **I002-IDX-069 [E][I-4][status]** I-4 is the Track-13 keystone and blocks I-22, I-28, and I-30. Anchor: line 274.
- **I002-IDX-070 [E][I-4][integration]** It composes SessionDag/H-49, D-3, and the supporting receipt stack rather than delegating selection to an LLM. Anchors: lines 274-275.
- **I002-IDX-071 [E][I-5][context]** I-5 hard-excludes failed siblings and raw tracebacks and retains a contrastive constraint. Anchors: lines 276-277.
- **I002-IDX-072 [E][I-5][measurement]** It A/B measures the vividness tax and is the ID-1 instrument. Anchor: line 277.
- **I002-IDX-073 [E][I-5][dependency]** It composes H-6, H-50, and the E-19 probe in Wave 2. Anchor: line 276.
- **I002-IDX-074 [E][I-6][context]** I-6 adds a privileged final context slot containing TaskSpec and active constraints. Anchors: lines 278-279.
- **I002-IDX-075 [E][I-6][workflow]** The slot appears after the last cache breakpoint and requires an acknowledgement tag. Anchor: line 279.
- **I002-IDX-076 [E][I-6][dependency]** It composes the existing ContextSlot substrate in Wave 2. Anchor: line 278.
- **I002-IDX-077 [E][I-7][routing]** I-7 uses a local-first cascade and escalates only after condensation through I-11. Anchors: lines 280-281.
- **I002-IDX-078 [E][I-7][constraint]** The Batch API is confined to offline lanes. Anchor: line 281.
- **I002-IDX-079 [E][I-7][dependency]** I-7 depends on I-11 and composes H-5/H-7 in Wave 4. Anchors: lines 280-281.
- **I002-IDX-080 [E][I-7][C]** Its cascade embodies IC-1's rejection of interactive batch racing. Anchor: line 281.
- **I002-IDX-081 [E][I-8][action]** I-8 adds an `acknowledge_infeasible` tool. Anchors: lines 282-283.
- **I002-IDX-082 [E][I-8][state]** It detects dead ends through AST hashing. Anchor: line 283.
- **I002-IDX-083 [E][I-8][workflow]** It applies an empirical halting bound and credits abstention under RD-1. Anchor: line 283.
- **I002-IDX-084 [E][I-8][dependency]** It composes D-16/D-17, H-39, and E-35 in Wave 1. Anchor: line 282.
- **I002-IDX-085 [E][I-9][measurement]** I-9 measures `alpha_t` with injected corruption and recovery rate. Anchors: lines 284-285.
- **I002-IDX-086 [E][I-9][control]** It actively anneals when `alpha > 1`. Anchor: line 285.
- **I002-IDX-087 [E][I-9][constraint]** Local baselines precede any external threshold under ID-5. Anchor: line 285.
- **I002-IDX-088 [E][I-10][cache]** I-10 freezes prefixes, including tool-array handling, and sets explicit one-hour/five-minute TTL tiers. Anchors: lines 287-290.
- **I002-IDX-089 [E][I-10][planning]** A cache-economics planner uses journaled inter-call-gap statistics. Anchors: lines 289-290.
- **I002-IDX-090 [E][I-10][failure]** Unexplained misses over 10% halt execution and dump a blake3 prefix diff. Anchor: line 290.
- **I002-IDX-091 [E][I-10][dependency]** This Track-14 keystone builds on H-1/H-2/H-4/H-9 and E-39. Anchor: line 288.
- **I002-IDX-092 [E][I-11][schema]** I-11 defines a typed, versioned `BeliefState` for cross-model and cross-tier handoffs. Anchors: lines 291-293.
- **I002-IDX-093 [E][I-11][measurement]** It periodically compares full-history and BeliefState reruns and journals the delta. Anchors: lines 292-293.
- **I002-IDX-094 [E][I-11][observability]** BeliefState handoffs are ID-3 receipt-bearing middleware transforms. Anchor: line 293.
- **I002-IDX-095 [E][I-11][dependency]** It composes H-7, H-51, DKG, and I-7 in Wave 2. Anchor: line 291.
- **I002-IDX-096 [E][I-12][lifecycle]** I-12 promotes playbook content only after at least `k` SSB-verified outcomes and measured lift. Anchors: lines 294-296.
- **I002-IDX-097 [E][I-12][lifecycle]** The same ledger can demote entries. Anchor: line 296.
- **I002-IDX-098 [E][I-12][validation]** DSPy-style validation runs against the C-28 suite. Anchor: line 296.
- **I002-IDX-099 [E][I-12][surface]** Curated content exports to `CLAUDE.md`, `AGENTS.md`, and skills. Anchor: line 296.
- **I002-IDX-100 [E][I-12][security]** Learned playbook entries are governed by ID-6 promotion discipline. Anchor: line 296.
- **I002-IDX-101 [E][I-13][security]** I-13 performs a deterministic scan at memory-write time. Anchors: lines 297-299.
- **I002-IDX-102 [E][I-13][schema]** It splits `to_llm_content` from `to_ui_dict`. Anchor: line 298.
- **I002-IDX-103 [E][I-13][capability]** It expands `TraceQuery`. Anchor: line 298.
- **I002-IDX-104 [E][I-13][dependency][A]** YARA is an optional new-dependency decision; regex/Aho-Corasick is the first implementation. Anchor: line 299.
- **I002-IDX-105 [E][I-14][workflow]** I-14 applies LSP slicing and tree-sitter AST folding before any learned compression. Anchors: lines 300-302.
- **I002-IDX-106 [E][I-14][deferred]** LLMLingua-2 is a gated Tier-2 path. Anchor: line 301.
- **I002-IDX-107 [E][I-14][non-goal]** N-gram ciphers are banned. Anchor: line 302.
- **I002-IDX-108 [E][I-14][lifecycle]** Every compression is journaled and reversible under ID-3. Anchor: line 302.
- **I002-IDX-109 [E][I-15][cache]** I-15 caches durable intermediate DKG/research summaries. Anchors: lines 303-304.
- **I002-IDX-110 [E][I-15][performance]** An exact-hash O(1) lookup precedes vector search. Anchor: line 304.
- **I002-IDX-111 [E][I-16][evaluation]** I-16 evaluates sink-protected SnapKV/ThinK and Int4 KV on the sidecar. Anchors: lines 305-307.
- **I002-IDX-112 [E][I-16][flags]** Evaluation is RunPod-gated. Anchor: line 306.
- **I002-IDX-113 [E][I-16][C][constraint]** MoE-sidecar retargeting is an evaluation, never an assumed swap. Anchors: lines 306-307.
- **I002-IDX-114 [E][I-16][measurement]** ID-5 requires local evidence before adoption claims. Anchor: line 307.
- **I002-IDX-115 [E][I-17][workflow]** I-17 applies a supreme Layer-1 hard veto, a calibrated Layer-2 score to survivors, and Layer-3 adversarial escalation on disagreement. Anchors: lines 309-312.
- **I002-IDX-116 [E][I-17][failure]** It uses an empirical 15-second halting bound. Anchor: line 311.
- **I002-IDX-117 [E][I-17][recovery]** Flakes enter quarantine with ledger evidence. Anchors: lines 311-312.
- **I002-IDX-118 [E][I-17][deferred]** Cross-language verifier packs are a Tier-2 seam. Anchor: line 312.
- **I002-IDX-119 [E][I-17][dependency]** I-17 is blocking for I-4. Anchor: line 310.
- **I002-IDX-120 [E][I-18][integrity]** I-18 adds a per-row `chain_prev_hash` and a five-layer stack hash. Anchors: lines 313-315.
- **I002-IDX-121 [E][I-18][integration]** Rekor transparency publishing is opt-in. Anchor: line 314.
- **I002-IDX-122 [E][I-18][compliance]** It emits an AI-BOM/AISVS AC.8 artifact. Anchors: lines 314-315.
- **I002-IDX-123 [E][I-18][fallback]** Offline blake3-Merkle sealing is the default. Anchor: line 315.
- **I002-IDX-124 [E][I-19][security]** I-19 blocks source edits that lack a symbol-level test delta at PreToolUse. Anchors: lines 316-318.
- **I002-IDX-125 [E][I-19][validation]** It performs a syntax-only dry run. Anchor: line 317.
- **I002-IDX-126 [E][I-19][exception]** Refactors may use a coverage-delta exemption. Anchors: lines 317-318.
- **I002-IDX-127 [E][I-19][rollout]** False-positive rate must be bounded before default-on. Anchor: line 318.
- **I002-IDX-128 [E][I-20][selection]** I-20 selects mutants inside the diff cone using reachability. Anchors: lines 319-322.
- **I002-IDX-129 [E][I-20][budget]** Mutation is time-boxed and sampled with a confidence interval and early stop. Anchors: lines 320-321.
- **I002-IDX-130 [E][I-20][cache]** Results are cached by `(fn-hash, test-hash)`. Anchor: line 321.
- **I002-IDX-131 [E][I-20][recovery]** Surviving mutants become structured regeneration Directives. Anchors: lines 321-322.
- **I002-IDX-132 [E][I-20][integration]** PBT relation adapters sit behind the D-3 trait. Anchor: line 322.
- **I002-IDX-133 [E][I-20][status]** I-20 is the Track-15 keystone and a Wave-3 extension. Anchor: line 319.
- **I002-IDX-134 [E][I-21][security]** I-21 gives generator and evaluator zero shared context. Anchors: lines 323-325.
- **I002-IDX-135 [E][I-21][workflow]** Sprint contracts require structured approval. Anchor: line 324.
- **I002-IDX-136 [E][I-21][retry]** Trace reinjection retries are bounded. Anchors: lines 324-325.
- **I002-IDX-137 [E][I-21][fallback]** A third loop escalates to a cross-family adversary. Anchor: line 325.
- **I002-IDX-138 [E][I-22][scoring]** I-22 scores delta progress. Anchors: lines 326-327.
- **I002-IDX-139 [E][I-22][failure]** It truncates at the first error and decays reward on no-progress loops. Anchor: line 327.
- **I002-IDX-140 [E][I-22][dependency]** I-22 depends on I-4 and follows ID-4. Anchors: lines 326-327.
- **I002-IDX-141 [E][I-23][architecture]** I-23 unifies calibration in one implementation. Anchors: lines 328-329.
- **I002-IDX-142 [E][I-23][integration]** Refusal, `claim_score`, and search prior consume that implementation. Anchor: line 329.
- **I002-IDX-143 [E][I-23][workflow]** Calibrated uncertainty can fork execution. Anchor: line 329.
- **I002-IDX-144 [E][I-24][security]** I-24 injects AST-preserving obfuscation canaries during scoring. Anchors: lines 330-332.
- **I002-IDX-145 [E][I-24][failure]** It detects style bias and neutralizes the scorer's weight. Anchors: lines 331-332.
- **I002-IDX-146 [E][I-24][dependency]** It composes H-44's detector rather than restating it. Anchor: line 332.
- **I002-IDX-147 [E][I-25][learning]** I-25 trains a difficulty/router classifier on ELAI's own `(task-features, lane, cost, verifier outcome)` evidence. Anchors: lines 334-337.
- **I002-IDX-148 [E][I-25][routing]** Execution-cost profiling pre-routes known-expensive task shapes. Anchors: lines 336-337.
- **I002-IDX-149 [E][I-25][security]** Learned routing is governed by ID-5 local baselines and ID-6 promotion discipline. Anchor: line 337.
- **I002-IDX-150 [E][I-26][budget]** I-26 scales thinking budgets with serial depth. Anchors: lines 338-340.
- **I002-IDX-151 [E][I-26][failure]** It applies an 85% collapse threshold and a plateau early stop. Anchor: line 339.
- **I002-IDX-152 [E][I-26][integration]** Cloud lanes map budgets to provider effort parameters. Anchors: lines 339-340.
- **I002-IDX-153 [E][I-26][compatibility]** Opus 4.7+ rejecting manual `budget_tokens` is a binding provider constraint. Anchor: line 340.
- **I002-IDX-154 [E][I-27][measurement]** I-27 plots verified-outcome rate against compute. Anchors: lines 341-343.
- **I002-IDX-155 [E][I-27][UI]** It exposes a Pareto cost/accuracy cockpit tile. Anchor: line 342.
- **I002-IDX-156 [E][I-27][constraint]** Anytime monotonicity is measured, never asserted. Anchors: lines 342-343.
- **I002-IDX-157 [E][I-28][credit]** I-28 assigns backward causal credit from COMPLETE. Anchors: lines 344-346.
- **I002-IDX-158 [E][I-28][dataset]** It extracts a DPO pair from a winner and the highest-scoring failed branch. Anchors: lines 345-346.
- **I002-IDX-159 [E][I-28][retrieval]** It demotes retrieval chunks associated with failure. Anchor: line 346.
- **I002-IDX-160 [E][I-28][non-goal]** Shapley attribution is declined as research-grade; I-28 depends on I-4. Anchors: lines 344-346.
- **I002-IDX-161 [E][I-29][generation]** I-29 makes a Mercury-class dLLM an untrusted draft generator behind the full veto stack. Anchors: lines 347-349.
- **I002-IDX-162 [E][I-29][concurrency]** CRDT semantics merge multi-agent drafts. Anchors: lines 348-349.
- **I002-IDX-163 [E][I-29][permissions]** New dependency and network access require user approval. Anchor: line 349.
- **I002-IDX-164 [E][I-29][C][scope]** Full TTD-DR is re-scoped to the narrow draft lane; I-29 is L effort and must be full-form-expanded. Anchors: lines 347-349, 167-171.
- **I002-IDX-165 [E][I-30][dataset]** I-30 extracts winning paths from sealed RRAs into SFT JSONL. Anchors: lines 351-354.
- **I002-IDX-166 [E][I-30][dataset]** It pairs fluent-but-vetoed outputs as negatives. Anchors: lines 353-354.
- **I002-IDX-167 [E][I-30][rollout]** Rejection-sampling LoRA SFT ships before GRPO. Anchor: line 354.
- **I002-IDX-168 [E][I-30][security]** Promotion uses ID-6 ledger gates and depends on I-4. Anchors: lines 352-354.
- **I002-IDX-169 [E][I-31][learning]** I-31 performs GRPO from a typed endpoint serving execution-grounded rewards. Anchors: lines 355-358.
- **I002-IDX-170 [E][I-31][security]** Blocking prerequisites include I-24 canaries, H-28 reward-tampering defense, and H-17/H-18 ledger controls. Anchors: lines 355-358.
- **I002-IDX-171 [E][I-31][constraint]** KL penalty is mandatory and the evaluator remains outside the write boundary. Anchors: lines 356-358.
- **I002-IDX-172 [E][I-31][flags]** The capability stays inside the existing `darwinian-evolve` kill-switched lane. Anchor: line 358.
- **I002-IDX-173 [E][I-31][status]** I-31 is explicitly last. Anchor: line 355.
- **I002-IDX-174 [E][I-32][roles]** I-32 removes personas from factual/code roles. Anchors: lines 360-363.
- **I002-IDX-175 [E][I-32][roles]** It retains personas for alignment-dependent auditor-style and anti-spiral roles. Anchors: lines 362-363.
- **I002-IDX-176 [E][I-32][schema]** Roles become schema-first contracts. Anchor: line 363.
- **I002-IDX-177 [E][I-32][measurement]** Each role receives a measured A/B. Anchor: line 363.
- **I002-IDX-178 [E][I-33][governance]** I-33 externalizes candidates for decision-type LLM calls with CEFL. Anchors: lines 364-366.
- **I002-IDX-179 [E][I-33][validation]** I6Q validates rationale quality. Anchor: line 365.
- **I002-IDX-180 [E][I-33][integrity]** Commit-reveal protects entropy integrity on stochastic gate choices. Anchors: lines 365-366.
- **I002-IDX-181 [E][I-33][observability]** CDL, DIU, and FSR metrics are ledgered. Anchor: line 366.
- **I002-IDX-182 [E][I-34][generation]** I-34 applies CRANE thinking-token grammar augmentation on local lanes. Anchors: lines 367-369.
- **I002-IDX-183 [E][I-34][security]** Grammars are blake3-pinned control-plane artifacts. Anchor: line 368.
- **I002-IDX-184 [E][I-34][validation]** Outputs undergo post-parse schema revalidation. Anchors: lines 368-369.
- **I002-IDX-185 [E][I-34][failure]** The revalidation backstops silently noncompliant constrained-decoding output. Anchor: line 369.
- **I002-IDX-186 [E][I-35][workflow]** I-35 defines trigger/predicate/enforcement rules over tool-call history. Anchors: lines 370-372.
- **I002-IDX-187 [E][I-35][security]** Enforcement runs at the I-1 supervisor plane. Anchor: line 371.
- **I002-IDX-188 [E][I-35][capability]** It supports deterministic “X-before-Y” sequence classes that are CDA-immune. Anchors: lines 371-372.
- **I002-IDX-189 [E][I-35][deferred]** Pro2Guard DTMC monitoring is Tier 2 and deferred pending evidence. Anchor: line 372.
- **I002-IDX-190 [E][I-36][artifact]** I-36 compiles per-backend prompt artifacts. Anchors: lines 373-375.
- **I002-IDX-191 [E][I-36][measurement]** It measures transfer gaps from the ledger. Anchor: line 374.
- **I002-IDX-192 [E][I-36][constraint]** Deliberative lanes receive no explicit chain-of-thought instruction under extended thinking. Anchors: lines 374-375.
- **I002-IDX-193 [E][I-36][compatibility]** Backend temperature constraints remain binding; outputs follow ID-3 receipts and ID-6 promotion discipline. Anchor: line 375.
- **I002-IDX-194 [E][I-37][schema]** I-37 creates EARS-typed requirements with content hashes. Anchors: lines 376-378.
- **I002-IDX-195 [E][I-37][traceability]** Each requirement binds tests and mutation coverage. Anchors: lines 377-378.
- **I002-IDX-196 [E][I-37][UI]** The cockpit exposes per-requirement verified/unverified state. Anchors: lines 377-378.
- **I002-IDX-197 [E][I-37][integration]** Spec-kit import interop is included. Anchor: line 378.
- **I002-IDX-198 [E][I-38][metric]** I-38 defines QDELTA as defective diffs reaching review per 100 tasks. Anchors: lines 379-381.
- **I002-IDX-199 [E][I-38][experiment]** Gates-on and gates-off arms are compared with bootstrap confidence intervals. Anchors: lines 380-381.
- **I002-IDX-200 [E][I-38][statistical-gate]** The floor requires paired seeds of at least five, SEM, pass^k, and dollars/task. Anchor: line 381.
- **I002-IDX-201 [E][I-38][compliance]** Contamination discipline surfaces the D-20 probe; ID-5 binds the metric. Anchor: line 381.
- **I002-IDX-202 [E][I-39][status]** `elai plan status` derives plan state from the journal and wiring fragments. Anchors: lines 382-384.
- **I002-IDX-203 [E][I-39][non-goal]** Hand-maintained tallies are demoted to build artifacts. Anchor: line 383.
- **I002-IDX-204 [E][I-39][compatibility-test]** One migration-replay test runs an old `ssb.db` through current code to detect RRA rot. Anchors: lines 383-384.
- **I002-IDX-205 [E][I-40][artifact]** I-40 publishes `.rra` as an open specification. Anchors: lines 385-387.
- **I002-IDX-206 [E][I-40][integration]** A minimal proxy/MCP wrapper adds taint, write gates, and RRA evidence around external agents. Anchors: lines 386-387.
- **I002-IDX-207 [E][I-40][product]** The wrapper is the wedge/standard tier; the orchestrator remains premium. Anchor: line 387.
- **I002-IDX-208 [E][I-40][status]** I-40 is an L-effort, separately gated Wave-5 build. Anchors: lines 385-387, 530-533.
- **I002-IDX-209 [E][I-41][capability]** I-41 runs eight advisory critique lenses over prose/specification. Anchors: lines 389-392.
- **I002-IDX-210 [E][I-41][evidence]** Findings without anchors are dropped. Anchor: line 391.
- **I002-IDX-211 [E][I-41][security]** `advisory=true`; critique never yields accept/block. Anchors: lines 391-392.
- **I002-IDX-212 [E][I-41][surface]** It integrates with the B-40 idea-critic loop and `elai critique`. Anchor: line 392.
- **I002-IDX-213 [E][I-42][schema]** I-42 adds `RunProfile { Orchestrate, CritiqueOnly }`. Anchors: lines 393-395.
- **I002-IDX-214 [E][I-42][permissions]** CritiqueOnly structurally disables executor/fixer spawning while the operator writes. Anchors: lines 394-395.
- **I002-IDX-215 [E][I-42][invariant]** The profile must produce zero executor-attributed `file_write` rows. Anchor: line 395.
- **I002-IDX-216 [E][I-42][architecture]** Both profiles use one engine and one verification spine; I-42 depends on I-41. Anchors: lines 393-395.
- **I002-IDX-217 [E][I-43][experiment]** I-43 runs a reduced-N gates-on/off pilot. Anchors: lines 396-398.
- **I002-IDX-218 [E][I-43][failure]** The pilot detects the block-everything trap. Anchor: line 397.
- **I002-IDX-219 [E][I-43][truthfulness]** A cross-lane scoper limits reproducibility claims. Anchors: lines 397-398.
- **I002-IDX-220 [E][I-43][rollout]** A go/no-go memo precedes the G-6 XL build; fixture runs ship now and the live leg is gated. Anchor: line 398.
- **I002-IDX-221 [E][I-44][schema]** I-44 defines `RetentionPolicy { Full, ZeroAfterVerify }`. Anchors: lines 399-401.
- **I002-IDX-222 [E][I-44][privacy]** ZeroAfterVerify purges raw inputs after a sealed verdict but retains redacted evidence. Anchors: lines 400-401.
- **I002-IDX-223 [E][I-44][recovery-test]** `replay --verify` must still pass after purge. Anchor: line 401.
- **I002-IDX-224 [E][I-44][flags]** Zero retention is opt-in and the default remains unchanged under GD-4; I-44 depends on G-9. Anchors: lines 399-401.
- **I002-IDX-225 [E][I-45][integrity]** I-45 stamps `blake3(canonical objective)` into every sealed PASS. Anchors: lines 403-406.
- **I002-IDX-226 [E][I-45][lifecycle]** Mutating the goal after PASS makes the receipt stale on every read path. Anchors: lines 405-406.
- **I002-IDX-227 [E][I-45][failure]** Stale receipts hard-fail. Anchor: line 406.
- **I002-IDX-228 [E][I-45][status]** I-45 is an extension over shipped RRA/attestation substrate. Anchors: lines 404-406.
- **I002-IDX-229 [E][I-46][action]** I-46 refuses and redirects bare grep/cat/find to registered tools. Anchors: lines 407-409.
- **I002-IDX-230 [E][I-46][fixup]** It uses a deterministic fixup table and bounded allowed prefixes. Anchors: lines 408-409.
- **I002-IDX-231 [E][I-46][security]** Redirect substitutes are never auto-executed. Anchor: line 409.
- **I002-IDX-232 [E][I-46][invariant]** Interception narrows and never widens existing validation. Anchor: line 409.
- **I002-IDX-233 [E][I-47][security]** I-47 tags auditor inputs as untrusted and adds an anti-self-attestation frame. Anchors: lines 410-412.
- **I002-IDX-234 [E][I-47][dispatch]** A frozen per-role worker-discipline block is injected at dispatch. Anchors: lines 411-412.
- **I002-IDX-235 [E][I-47][cache]** The injected block is byte-stable under HD-2. Anchor: line 412.
- **I002-IDX-236 [E][I-48][artifact]** I-48 moves every dispatch prompt to reviewed `crates/*/prompts/*.md` files loaded by `include_str!`. Anchors: lines 413-415.
- **I002-IDX-237 [E][I-48][compatibility]** Extraction must preserve prompt bytes. Anchors: lines 414-415.
- **I002-IDX-238 [E][I-48][test]** A contract test bans inline prompt literals, enabling diffable and pinnable prompts. Anchor: line 415.
- **I002-IDX-239 [E][I-49][integration]** I-49 adds direct Bedrock SigV4+EventStream, Azure Responses, and OpenAI-to-Anthropic shim protocol lanes. Anchors: lines 416-418.
- **I002-IDX-240 [E][I-49][rollout]** Protocol logic and replay fixtures ship now; live calls are separate checkboxes. Anchors: lines 417-418.
- **I002-IDX-241 [E][I-49][non-goal]** It is the only provider steal; a wholesale port remains rejected. Anchor: line 418.
- **I002-IDX-242 [E][I-49][status]** I-49 is an L-effort Wave-4 build and therefore requires full-form expansion and source rereads. Anchors: lines 416-418, 66-68.
- **I002-IDX-243 [E][I-50][architecture]** I-50 defines an `AuthProvider` trait and broker generalizing Anthropic-only `oauth.rs`. Anchors: lines 419-421.
- **I002-IDX-244 [E][I-50][compatibility]** Existing Anthropic OAuth flow must remain byte-compatible. Anchors: lines 420-421.
- **I002-IDX-245 [E][I-50][privacy]** Token material is never journaled. Anchor: line 421.
- **I002-IDX-246 [E][I-50][dependency]** I-50 composes B-37 and I-49 in Wave 4. Anchors: lines 419-421.
- **I002-IDX-247 [E][I-51][data]** I-51 establishes one `models.toml` containing per-model context, pricing, thinking, and VRAM-tier data. Anchors: lines 422-424.
- **I002-IDX-248 [E][I-51][compatibility]** Existing modules become views while their APIs are preserved. Anchors: lines 423-424.
- **I002-IDX-249 [E][I-51][C][constraint]** The catalog consolidates duplicated names but wraps and never deletes measured VRAM-tier logic. Anchor: line 424.
- **I002-IDX-250 [E][I-51][integration]** E-39/E-31 and later routing consume the catalog. Anchors: lines 422-424.
- **I002-IDX-251 [E][I-52][tool]** I-52 exposes save, rewind, and diff over the shipped E-25 checkpoint store. Anchors: lines 425-427.
- **I002-IDX-252 [E][I-52][scope]** Checkpoints are session-scoped. Anchor: line 426.
- **I002-IDX-253 [E][I-52][failure]** Rewind refuses divergent state. Anchor: line 426.
- **I002-IDX-254 [E][I-52][C][integration]** If H-49 ships, I-52 coordinates with/reuses that store and never creates a second one. Anchors: lines 426-427.
- **I002-IDX-255 [E][I-53][tool]** I-53 discovers and gates Justfile, Makefile, and cargo-alias recipes. Anchors: lines 428-430.
- **I002-IDX-256 [E][I-53][tool]** It adds a read-only archive reader. Anchor: line 429.
- **I002-IDX-257 [E][I-53][tool]** It adds a capped JSON-tree explorer. Anchors: lines 429-430.
- **I002-IDX-258 [E][I-53][non-goal]** Cron, monitor, and background-job tools are declined because the supervisor/orchestrator owns scheduling. Anchor: line 430.
- **I002-IDX-259 [E][I-53][dependency]** Recipe execution composes bash validation, E-30, I-46, and the tool catalog. Anchors: lines 428-430.
- **I002-IDX-260 [E][I-54][tool]** I-54 performs tree-sitter-query structural search, preview, and apply using in-tree grammars. Anchors: lines 431-433.
- **I002-IDX-261 [E][I-54][workflow]** Preview must precede apply. Anchor: line 432.
- **I002-IDX-262 [E][I-54][security]** Apply traverses the same edit gates as ordinary Edit. Anchor: line 433.
- **I002-IDX-263 [E][I-54][dependency]** Any ast-grep dependency is banned. Anchor: line 433.
- **I002-IDX-264 [E][I-54][scope]** E-52 remains the separate semantic-edit sibling. Anchors: lines 431-433.

#### Complete action/event schema ledger

The baseline is explicitly provisional: 221/532 on 2026-06-10, re-read at edit; ship-order bumps are cumulative and all six lockstep sites plus the cross-language drift contract/golden lock must reconcile. Anchors: lines 435-443, 502-503.

- **I002-IDX-265 [E][schema][I-1]** Add `DirectiveExecuted { Executed, Discarded, CompensationFired }`, baseline +1. Anchor: line 444.
- **I002-IDX-266 [E][schema][I-2]** Add `BranchWorkspaceEvent { Spawned, Merged, ConflictResolved, EscapeBlocked }`, baseline +1. Anchor: line 445.
- **I002-IDX-267 [E][schema][I-3]** Add no action type; reuse checkpoint/memory-monitoring rows and put throughput in metrics TOML. Anchor: line 446.
- **I002-IDX-268 [E][schema][I-4]** Add `SearchNodeEvent { Expanded, Pruned, Promoted, BudgetCollapsed }`, baseline +1. Anchor: line 447.
- **I002-IDX-269 [E][schema][I-5]** Add `ContextPruned { SiblingExcluded, TracebackContrasted, VividnessProbe }`, baseline +1. Anchor: line 448.
- **I002-IDX-270 [E][schema][I-6]** Add no action type; reuse `ContextReduction`, because anchor presence is a prompt fact. Anchor: line 449.
- **I002-IDX-271 [E][schema][I-7]** Add `EscalationCascade { LocalWin, Condensed, Escalated, BatchRouted }`, baseline +1. Anchor: line 450.
- **I002-IDX-272 [E][schema][I-8]** Add `TrajectoryAborted { Voluntary, DeadEndHash, HaltingBound }`, baseline +1. Anchor: line 451.
- **I002-IDX-273 [E][schema][I-9]** Add flat beacon `ErrorAmplificationMeasured`, baseline +1. Anchor: line 452.
- **I002-IDX-274 [E][schema][I-10]** Add `CacheEconomicsEvent { TtlPlanned, HitRateGuardTripped, PrefixDiffDumped }`, baseline +1. Anchor: line 453.
- **I002-IDX-275 [E][schema][I-11]** Add `BeliefStateEvent { Condensed, Handoff, HonestyDelta }`, baseline +1. Anchor: line 454.
- **I002-IDX-276 [E][schema][I-12]** Add `PlaybookEvent { Promoted, Demoted, ValidationRun, Exported }`, baseline +1. Anchor: line 455.
- **I002-IDX-277 [E][schema][I-13]** Add `MemoryGuardEvent { PoisonBlocked, DualSerialized, TraceQueryExpanded }`, baseline +1. Anchor: line 456.
- **I002-IDX-278 [E][schema][I-14]** Add ID-3 receipt `ContextTransformReceipt { LspSlice, AstFold, Tier2Compressed }`, baseline +1. Anchor: line 457.
- **I002-IDX-279 [A][schema][I-15]** Prefer reusing `OracleCacheHit/Miss` with a `semantic_cache` discriminator; add `SemanticCacheEvent` only if the existing payload lacks source. Decide at edit. Anchors: lines 458-459.
- **I002-IDX-280 [E][schema][I-16]** Add flat beacon `KvCacheTuned`, baseline +1. Anchor: line 460.
- **I002-IDX-281 [E][schema][I-17]** Add `TriLayerVerdict { HardVeto, ScoredSurvivor, DisagreementEscalated, FlakeQuarantined }`, baseline +1. Anchor: line 461.
- **I002-IDX-282 [E][schema][I-18]** Add `RowChainSealed { ChainAdvanced, RekorPushed, AibomEmitted, ChainBroken }`, baseline +1. Anchor: line 462.
- **I002-IDX-283 [E][schema][I-19]** Add `TddPreflightBlocked { MissingTestDelta, SyntaxDryRunFailed, RefactorExempted }`, baseline +1. Anchor: line 463.
- **I002-IDX-284 [E][schema][I-20]** Add `MutationBudgetedRun { CiCleared, CiFailed, SurvivorDirective, CacheHit }`, baseline +1. Anchor: line 464.
- **I002-IDX-285 [E][schema][I-21]** Add `SprintContractEvent { Proposed, Approved, Rejected, RalphReinjected, AdversaryEscalated }`, baseline +1. Anchor: line 465.
- **I002-IDX-286 [E][schema][I-22]** Add `ProgressRewardScored { Delta, FirstErrorTruncated, DecayApplied }`, baseline +1. Anchor: line 466.
- **I002-IDX-287 [E][schema][I-23]** Add no action type; reuse `RefusalDecision` and `CalibrationUpdate`. Anchor: line 467.
- **I002-IDX-288 [E][schema][I-24]** Add `PrmCanaryEvent { CanaryInjected, StyleBiasDetected, ScorerNeutralized }`, baseline +1. Anchor: line 468.
- **I002-IDX-289 [E][schema][I-25]** Add no action type; emit the existing E-31 `RouteDecisionEvent`. Anchor: line 469.
- **I002-IDX-290 [E][schema][I-26]** Add no action type; reuse `ReasoningBudgetScaled` and extend its payload only if necessary. Anchor: line 470.
- **I002-IDX-291 [E][schema][I-27]** Add flat beacon `ConvergenceCurveMeasured`, baseline +1. Anchor: line 471.
- **I002-IDX-292 [E][schema][I-28]** Add `DagCreditAssigned { CreditWalk, DpoPairExtracted, ChunkDemoted }`, baseline +1. Anchor: line 472.
- **I002-IDX-293 [E][schema][I-29]** Add `DiffusionDraftEvent { DraftAccepted, DraftVetoed, CrdtMerged }`, baseline +1. Anchor: line 473.
- **I002-IDX-294 [E][schema][I-30]** Add `SftExhaustEvent { PathExtracted, NegativePaired, DatasetSealed }`, baseline +1. Anchor: line 474.
- **I002-IDX-295 [E][schema][I-31]** Add `GrpoRewardComputed { RewardServed, PromotionBlocked, PromotionLedgered }`, baseline +1. Anchor: line 475.
- **I002-IDX-296 [E][schema][I-32]** Add `RoleContractApplied { PersonaStripped, PersonaRetained, AbDelta }`, baseline +1. Anchor: line 476.
- **I002-IDX-297 [E][schema][I-33]** Add `GovernancePrimitiveFired { CeflExternalized, I6qRejected, CommitRevealSealed }`, baseline +1. Anchor: line 477.
- **I002-IDX-298 [E][schema][I-34]** Add `GrammarGateEvent { CranePassed, PinViolation, PostParseRejected }`, baseline +1. Anchor: line 478.
- **I002-IDX-299 [E][schema][I-35]** Add `ToolSequenceVerdict { Allowed, OrderViolation, HaltEnforced }`, baseline +1. Anchor: line 479.
- **I002-IDX-300 [E][schema][I-36]** Add `PromptCompiled { Compiled, TransferGapMeasured, Rolledback }`, baseline +1. Anchor: line 480.
- **I002-IDX-301 [E][schema][I-37]** Add `RequirementVerified { Satisfied, Unverified, Regressed }`, baseline +1. Anchor: line 481.
- **I002-IDX-302 [E][schema][I-38]** Add flat beacon `QdeltaMeasured`, baseline +1. Anchor: line 482.
- **I002-IDX-303 [E][schema][I-39]** Add flat beacon `PlanStateDerived`, baseline +1; migration replay reuses existing rows. Anchor: line 483.
- **I002-IDX-304 [E][schema][I-40]** Add no action type for the spec; wrapper reuses taint-flow and run-artifact-emitted rows. Anchor: line 484.
- **I002-IDX-305 [E][schema][I-41]** Add advisory `CritiqueLensFinding { AdvisoryFinding, LensRun, DroppedNoAnchor }`, baseline +1. Anchor: line 485.
- **I002-IDX-306 [E][schema][I-42]** Add no action type; reuse `phase_transition` and prove CritiqueOnly from absent executor `file_write` rows. Anchors: lines 486-487.
- **I002-IDX-307 [E][schema][I-43]** Add `ClaimScopeMeasured { PilotDelta, CrossLaneDivergence, ClaimScoped }`, baseline +1. Anchor: line 488.
- **I002-IDX-308 [E][schema][I-44]** Add `RetentionPurged { RawInputsPurged, EvidenceRetained, PurgeVerified }`, baseline +1. Anchor: line 489.
- **I002-IDX-309 [E][schema][I-45]** Add `ObjectiveSealed { Sealed, MutationDetected, ReceiptStale }`, baseline +1. Anchor: line 490.
- **I002-IDX-310 [E][schema][I-46]** Add `BashRedirected { Redirected, FixedUp, PrefixBlocked }`, baseline +1. Anchor: line 491.
- **I002-IDX-311 [E][schema][I-47]** Add `PromptHardeningApplied { InputsTagged, RulesInjected, SelfAttestationStripped }`, baseline +1. Anchor: line 492.
- **I002-IDX-312 [E][schema][I-48]** Add no action type; inspect existing journaled request bytes and later reuse I-36 `PromptCompiled` provenance. Anchors: lines 493-494.
- **I002-IDX-313 [E][schema][I-49]** Add no action type; reuse route/runtime rows with new backend IDs. Anchor: line 495.
- **I002-IDX-314 [E][schema][I-50]** Add `AuthBrokerEvent { TokenAcquired, Refreshed, ProviderLinked, RefreshFailed }`, baseline +1. Anchor: line 496.
- **I002-IDX-315 [E][schema][I-51]** Add no action type; existing cost/route rows carry catalog-derived values. Anchor: line 497.
- **I002-IDX-316 [A][schema][I-52]** Add `CheckpointToolEvent { Saved, Rewound, Diffed, RefusedDiverged }`, baseline +1, unless H-49 ships first, in which case reuse `EditHistoryEvent`. Anchors: lines 498-499.
- **I002-IDX-317 [E][schema][I-53]** Add no action type; all tools use existing `tool_call` rows. Anchor: line 500.
- **I002-IDX-318 [E][schema][I-54]** Add `StructuralEditApplied { Matched, Previewed, Applied, RefusedStale, ParseFailed }`, baseline +1. Anchor: line 501.

#### Sequencing, implementation constraints, and carried landmines

- **I002-IDX-319 [E][sequencing]** Wave 1 is I-3/I-8/I-10/I-15/I-23/I-34/I-38/I-39/I-43/I-45/I-51: cheap, trust-reinforcing, instrument-first, and search-independent. Anchors: lines 505-513.
- **I002-IDX-320 [E][sequencing]** I-43 falsifies before the G-6/I-38 XL measurement build; I-45 is the cheapest live-RRA trust steal; I-51 feeds later routing. Anchors: lines 510-513.
- **I002-IDX-321 [E][sequencing]** Wave 2 is the isolation/context/governance/dispatch spine: I-1/I-2/I-5/I-6/I-11/I-13/I-14/I-32/I-33/I-35/I-41/I-46/I-47/I-48/I-53. Anchors: lines 514-519.
- **I002-IDX-322 [E][sequencing]** Wave 3 builds receipt-ranked verification and gated edit tools; I-4 cannot ship before I-17 and I-20, I-42 depends on I-41, I-44 depends on G-9, and I-52/I-54 reuse edit gates. Anchors: lines 520-525.
- **I002-IDX-323 [E][sequencing]** Wave 4 contains search, routing, measurement, and provider/auth reach; I-49/I-50 wait until router/catalog substrate exists. Anchors: lines 526-529.
- **I002-IDX-324 [E][sequencing]** Wave 5 contains separately gated frontier/flywheel/wedge work; distillation consumes Wave-4 trajectories, GRPO is last, and the wrapper waits for end-to-end evidence. Anchors: lines 530-533.
- **I002-IDX-325 [E][implementation]** Non-RFC hashing uses blake3 only; production code forbids `.unwrap()`/`.expect()` except tests/main/const-init. Anchors: lines 535-539.
- **I002-IDX-326 [E][implementation]** Production uses `tracing`, `thiserror`/`anyhow`, `tokio::sync::Mutex` in async, native async traits, and `FxHashMap`/`BTreeMap` over standard `HashMap`. Anchors: lines 538-540.
- **I002-IDX-327 [E][determinism]** Time and entropy come through `ClockProvider`/`EntropySource`; raw time/entropy is quarantined. Anchor: line 540.
- **I002-IDX-328 [E][test]** Scoped per-crate tests are for development; final validation is `cargo check --workspace`, with full suites run by the auditor. Anchors: lines 540-542.
- **I002-IDX-329 [E][permissions][security]** Every dependency must pass registry/repository/last-publish/license checks and receive user approval; known candidates are YARA, dLLM client, and RL stack. Anchors: lines 541-543.
- **I002-IDX-330 [E][compatibility]** Version-specific APIs require fresh web verification. Anchors: lines 543-544.
- **I002-IDX-331 [E][permissions]** Removal/simplification requires user confirmation; changes land one at a time. Anchor: line 544.
- **I002-IDX-332 [E][artifact]** Every session gets `workflow/reasoning/I-N-decisions.md`; commit cadence is one commit per shipped session, with maintainer voice, no AI trailers, and only when asked. Anchors: lines 544-546.
- **I002-IDX-333 [E][cache][compatibility]** Anthropic's default TTL changed from one hour to five minutes on 2026-03-06; TTL must always be explicit and long-TTL entries precede shorter ones. Anchors: lines 549-552.
- **I002-IDX-334 [E][rate-limit]** TPM reserves input plus `max_tokens`; schema-bound clamping prevents a 4096-token reservation for tiny scores. Anchors: lines 553-554.
- **I002-IDX-335 [E][concurrency][rate-limit]** A nominal 60 RPM limit is enforced sub-second, so parallel branch submissions must jitter/spread. Anchor: line 555.
- **I002-IDX-336 [E][economics][A]** Five-minute writes default to 1.25x/breakeven one read and one-hour writes to 2x/breakeven about three reads, but pricing must be re-verified at edit. Anchors: lines 556-557.
- **I002-IDX-337 [E][constraint]** Batch work takes roughly 10-45 minutes with 24-hour SLA, cancellation does not refund queued work, and batch never runs interactively/search-loop. Anchors: lines 558-559.
- **I002-IDX-338 [E][compatibility]** Opus 4.7+ rejects manual `budget_tokens`; use effort and automatic interleaved thinking. Anchors: lines 560-561.
- **I002-IDX-339 [E][validation]** Constrained-decoding engines can silently violate hard schemas and lose coverage on complex schemas; post-parse revalidation is mandatory. Anchors: lines 562-564.
- **I002-IDX-340 [E][security]** Grammars are pinned control-plane artifacts; third-party grammars and untrusted enum-bearing schemas are rejected because constrained decoding is an attack surface. Anchors: lines 565-567.
- **I002-IDX-341 [E][non-goal]** Cipher/abbreviation substitution in code prompts is banned because it breaks BPE priors. Anchor: line 568.
- **I002-IDX-342 [E][determinism][truthfulness]** Temperature zero is not deterministic; reproducibility claims remain action-stream-scoped. Anchors: lines 569-570.
- **I002-IDX-343 [E][metric][non-goal]** SWE-bench Verified is rejected as a headline; QDELTA is ablation-framed and never a leaderboard rank. Anchors: lines 571-572.
- **I002-IDX-344 [E][routing]** OPRO-style optimizers run on cloud models, though their target can be local. Anchors: lines 573-574.
- **I002-IDX-345 [E][roles]** Personas are harmful for factual/code accuracy and retained only for alignment-dependent work. Anchors: lines 575-576.
- **I002-IDX-346 [E][product]** Evidence/cockpit work targets review-time bottlenecks, not raw generation speed. Anchors: lines 577-578.
- **I002-IDX-347 [E][C][governance]** All `IC-1..IC-18` corrections are binding; highlighted replacements include no batch racing, deterministic compression first, cipher ban, TTL primary, measured tool-array layout, laddered search, SFT-before-GRPO, evaluation-only MoE retargeting, trust-spine reuse, and corrected persona/grammar posture. Anchors: lines 580-588.
- **I002-IDX-348 [E][status]** The session log was empty at generation; the index does not claim any I session DONE. Anchors: lines 590-593.
- **I002-IDX-349 [E][next-step]** The prescribed first command checks I-39, then I-10 follows as the economics keystone. Anchors: lines 595-601.

I-0 ambiguity/conflict register:

- **I002-IDX-350 [A][status]** “Shipped” predecessors in detail prose are plan-time assertions unless re-grepped under W-7; the index itself does not establish their current 2026 status. Anchors: lines 43-74, 200.
- **I002-IDX-351 [A][schema]** I-15 and I-52 retain conditional action-row decisions; the advertised net `~+41` is approximate and depends on I-52 reuse. Anchors: lines 458-459, 498-503.
- **I002-IDX-352 [C][scope]** I-29 narrows full TTD-DR to an untrusted draft lane; I-49 narrows the provider steal to three otherwise-unreachable protocols; I-54 takes only the structural-edit idea and retains the ast-grep ban. Anchors: lines 167-182, 416-433.
- **I002-IDX-353 [I][interaction]** Because W-7/W-9 force live rechecks and the session log is empty, all baseline and shipped-substrate claims are implementation inputs, not proof that the corresponding features currently ship. Anchors: lines 184-204, 438-441, 590-593.

### 3. `I-5_pristine_context_injector.md`

Source fully read: 115/115 lines. Metadata: Track 13, `TRUST-STEERING`, strong grounding, build, M effort, Wave 2, binds ID-1.

- **I002-I05-001 [E][premise]** Failed-sibling tracebacks/raw errors are hypothesized to consume context and bias the next step; the claim is instrumented rather than imported from literature. Anchors: lines 5-9.
- **I002-I05-002 [E][workflow]** `PristinePathFilter::build_branch_context` rebuilds branch context from the parent chain at expansion time. Anchors: lines 12-17, 61-63.
- **I002-I05-003 [E][context]** Failed sibling payloads are hard-excluded. Anchors: lines 13-18, 39-42.
- **I002-I05-004 [E][context]** Raw tracebacks are hard-excluded. Anchors: lines 13-18, 73-75.
- **I002-I05-005 [E][schema]** The only permitted negative signal is a structured contrastive constraint of at most 30 tokens, exemplified as attempt/line/type/avoid. Anchors: lines 14-17, 73-75.
- **I002-I05-006 [E][measurement]** A sampled same-decision A/B compares pristine rebuild against accumulated context via the E-19 reasoning-quality probe. Anchors: lines 5-9, 16-17, 43-44.
- **I002-I05-007 [E][observability]** The A/B outcome delta is ledgered in a `VividnessProbe` row and must recompute under `ELAI_SEED`. Anchors: lines 16-17, 43-44, 87-89.
- **I002-I05-008 [E][non-goal]** Carrying raw tracebacks is explicitly the anti-pattern. Anchor: line 18.
- **I002-I05-009 [E][non-goal]** Truncation is rejected because it preserves a smaller attention sink; context must be rebuilt. Anchors: lines 18-20.
- **I002-I05-010 [E][architecture]** I-5 extends H-6 turn masking and never creates a parallel masking/turn-drop layer. Anchors: lines 20-21, 28-29, 54-56, 76-77.
- **I002-I05-011 [E][constraint]** No numeric vividness improvement may be claimed before local A/B measurement. Anchors: lines 21-22, 78-79, 100-101.
- **I002-I05-012 [E][integration]** The rebuild hooks `context_budget.rs` assembly while preserving `update_breakpoints` byte-prefix stability and using `ContextSlot`. Anchors: lines 24-27, 53-56, 64-66.
- **I002-I05-013 [E][dependency][A]** H-6 and H-50 are plan-asserted Wave-2 predecessors and must be re-grepped before execution. Anchors: lines 28-31, 97-99.
- **I002-I05-014 [E][dependency][A]** E-19 is a plan-asserted existing probe that I-5 calls rather than rebuilds; it must be re-grepped. Anchors: lines 32-33, 68, 99.
- **I002-I05-015 [E][schema][status]** `ContextPruned` is described as net-new and confirmed absent as a named action type at authoring. Anchors: lines 34-35.
- **I002-I05-016 [E][scope]** Tier 1 creates `crates/orchestrator/src/pristine_path_filter.rs` and logs byte deltas from exclusion/compression. Anchors: lines 37-40, 52-53.
- **I002-I05-017 [E][schema]** Register `ContextPruned` with `SiblingExcluded`, `TracebackContrasted`, and `VividnessProbe`; baseline +1. Anchors: lines 41-44, 61-63, 105-106.
- **I002-I05-018 [E][sampling]** Only a sampled fraction of expansions runs both context arms. Anchors: lines 43-44.
- **I002-I05-019 [E][integration]** The production caller is I-4 `SearchLadder::run_best_of_n -> PristinePathFilter::build_branch_context`. Anchors: lines 45, 69-72.
- **I002-I05-020 [E][artifact]** Tier 1 requires `workflow/reasoning/I-5-decisions.md`. Anchor: line 46.
- **I002-I05-021 [E][deferred][UI]** The ReasoningDrawer vividness-tax tile is Tier 2, wire-dark until at least three A/B runs show a consistent delta and the GD-4 bound is measured. Anchors: lines 47-49, 57-58, 81-83.
- **I002-I05-022 [E][deferred]** Per-task-class contrastive-token-budget auto-tuning is deferred until after measurement. Anchor: line 50.
- **I002-I05-023 [E][integration]** H-50 supplies regime-aware assembly; I-5 shares that path instead of forking it. Anchors: lines 30-31, 67, 110.
- **I002-I05-024 [E][reachability]** Contract-test-only callers do not satisfy production reachability R7. Anchor: line 72.
- **I002-I05-025 [E][implementation]** Production paths forbid `.unwrap()`/`.expect()`, blake2/sha2, and fabricated measurements. Anchor: line 80.
- **I002-I05-026 [E][UI][done-gate]** If the dashboard tile ships in the H-15 era, `cross_lang_diff.py PASS` is mandatory; absence is W-6-justified for v1. Anchors: lines 81-83.
- **I002-I05-027 [E][trigger]** The live observable uses the retry-branch fixture with search ladder after I-4 v1 ships. Anchors: lines 84-86.
- **I002-I05-028 [E][observability]** The SSB must contain `SiblingExcluded` for the failed payload and `TracebackContrasted` for the compressed constraint. Anchors: lines 87-89.
- **I002-I05-029 [E][acceptance]** The outbound retry payload must contain zero bytes of the prior 5K-line traceback and must contain the <=30-token constraint phrase. Anchors: lines 90-92.
- **I002-I05-030 [E][broken-arm]** `--carry-tracebacks` must re-inflate the payload to at least 5K tokens and fail. Anchors: lines 91-92.
- **I002-I05-031 [E][grounding]** ID-1 is the stateless-generator/trajectory-pruning basis and requires instrument-first treatment. Anchors: lines 94-96.
- **I002-I05-032 [E][grounding]** IS-6 identifies `ContextSlot` and `update_breakpoints`; CL-1 binds regime awareness. Anchors: lines 97, 102.
- **I002-I05-033 [E][evidence]** Any arXiv identifier placed in the decision document must be web-verified before becoming a headline. Anchors: lines 103-104.
- **I002-I05-034 [E][schema][A]** The 221/532 baseline is dated provenance and must be re-read under W-9 at edit. Anchors: lines 105-106.
- **I002-I05-035 [E][dependency]** I-5 is a Wave-2 building block that I-4 consumes in Wave 4. Anchors: lines 108-113.

I-5 ambiguity/inference register:

- **I002-I05-036 [A][threshold]** `<=30` tokens is a hard fixture/contract ceiling, but optimal per-task-class budgets remain deferred rather than product constants. Anchors: lines 14-17, 47-50.
- **I002-I05-037 [A][UI]** The exact live UI timing depends on H-15-era surfaces and measured stability; v1 explicitly permits no tile. Anchors: lines 47-49, 81-83.
- **I002-I05-038 [I][privacy]** Hard-excluding sibling tracebacks also limits accidental replay of sensitive raw error content, but the source frames this as context quality, not a privacy guarantee. Anchors: lines 13-18, 90-92.

### 4. `I-10_prefix_invariance_ttl_tiering.md`

Source fully read: 101/101 lines. Metadata: Track 14 keystone, `CHEAPER`, strong grounding, build-on-shipped, M effort, Wave 1.

- **I002-I10-001 [E][premise]** Timestamps, mutable endpoint text, and provider-default TTLs can silently invalidate cloud prompt prefixes and force retransfers. Anchors: lines 4-7.
- **I002-I10-002 [E][cache]** Prefix bytes are frozen end-to-end. Anchors: lines 6-7, 12-13.
- **I002-I10-003 [E][constraint]** Cached prefix regions reject timestamps, UUIDs, and mutable endpoint strings. Anchors: lines 12-13, 38, 68-70.
- **I002-I10-004 [E][tools]** Dynamic tool restrictions use `tool_choice`/`allowed_tools` payload attributes, never prompt-text mutation. Anchors: lines 12-15, 38.
- **I002-I10-005 [E][measurement][A]** Inline-frozen versus H-4-deferred tool arrays are chosen per catalog size from measured cache-hit-by-token cost; neither layout is preselected. Anchors: lines 14-15, 71, 95.
- **I002-I10-006 [E][cache]** Frozen-prefix breakpoints use an explicit one-hour TTL. Anchors: lines 16-18.
- **I002-I10-007 [E][cache]** Volatile tails use an explicit five-minute TTL. Anchors: lines 16-18.
- **I002-I10-008 [E][ordering]** Long-TTL entries must precede shorter-TTL entries in a request. Anchors: lines 17-18.
- **I002-I10-009 [E][economics][A]** Planner defaults use one-hour 2x/~3-read breakeven and five-minute 1.25x/one-read breakeven, but pricing must be re-verified and never treated as fixed. Anchors: lines 19-20, 88-89.
- **I002-I10-010 [E][planning]** `CacheEconomicsPlanner` reads journaled inter-call gaps and selects a TTL tier per call class with a write-premium breakeven check. Anchors: lines 16-20, 36-37, 53-55.
- **I002-I10-011 [E][rate-limit]** `max_tokens_clamp` bounds every cached scoring call to its output schema. Anchors: lines 21-22, 36-37, 56.
- **I002-I10-012 [E][failure]** `PrefixInvarianceGuard` halts on unexplained misses over 10% for identical task classes. Anchors: lines 23-24, 36-37, 55.
- **I002-I10-013 [E][diagnostics]** Guard trip emits a blake3 prefix diff naming the mutation and leaking byte range. Anchors: lines 23-24, 36-37, 55.
- **I002-I10-014 [E][scheduling]** Burst-amortized scheduling is the primary TTL-preservation strategy. Anchors: lines 23-26.
- **I002-I10-015 [E][fallback]** Heartbeat traffic survives only as a last resort when burst scheduling cannot apply. Anchors: lines 24-26.
- **I002-I10-016 [E][dependency][C]** I-10 adds only TTL/economics/guard functionality if H-1/H-2 already ship and must re-grep the open gap before coding. Anchors: lines 9-11, 28-33, 98-99.
- **I002-I10-017 [E][status][A]** At authoring, CacheControl plumbing existed, Anthropic marker sites still had `cache_control: None`, and the planner/selector/guard were absent; each is plan-asserted and rechecked under W-7. Anchors: lines 28-33.
- **I002-I10-018 [E][scope]** Tier 1 creates `crates/orchestrator/src/cache_economics.rs` with planner, guard, and clamp helper. Anchors: lines 35-37, 47-49.
- **I002-I10-019 [E][validation]** Tier 1 validates frozen prefixes at prompt assembly and rejects dynamic text. Anchor: line 38.
- **I002-I10-020 [E][integration]** Every `apply_anthropic_cache_markers` site receives TTL annotation; the shipped marker implementation is never reimplemented. Anchors: lines 39, 61-62, 72.
- **I002-I10-021 [E][schema]** Add `CacheEconomicsEvent { TtlPlanned, HitRateGuardTripped, PrefixDiffDumped }`, baseline +1. Anchors: lines 40, 53-57, 90.
- **I002-I10-022 [E][test]** A deterministic seeded two-turn fixture proves turn-2 cache read and an injected-timestamp guard trip. Anchor: line 41.
- **I002-I10-023 [E][deferred]** Default-on planner rollout to all task classes waits for at least three real-run false-positive/over-block measurements. Anchors: lines 43-45.
- **I002-I10-024 [E][deferred]** Automated pricing-breakpoint reverification is a monitoring job, not a gate. Anchor: line 45.
- **I002-I10-025 [E][integration]** Marker types, context breakpoints, Anthropic cache-control paths, and `build_tool_catalog` are the production seams. Anchors: lines 47-50, 58-66.
- **I002-I10-026 [E][UI]** TTL/economics fields extend the H-9 cost tile in the CostDrawer/CachePanel family. Anchors: lines 50, 77-79.
- **I002-I10-027 [E][reachability]** Production reachability is Anthropic cache-control to `plan_ttl()` and context assembly to `check_miss_rate()`; tests alone do not satisfy R7. Anchors: lines 64-67.
- **I002-I10-028 [E][constraint]** TTL may never be left to provider default. Anchor: line 70.
- **I002-I10-029 [E][architecture]** Neither `apply_anthropic_cache_markers` nor `update_breakpoints` may be duplicated. Anchors: lines 71-73.
- **I002-I10-030 [E][implementation]** Production forbids `.unwrap()`/`.expect()`, blake2/sha2, and fabricated miss/cost measurements; blake3 is mandatory. Anchors: lines 74-76.
- **I002-I10-031 [E][measurement]** Cost/miss measurements are per configuration and never multiplied. Anchor: line 76.
- **I002-I10-032 [E][UI][done-gate]** The user-visible planner decision/guard signal makes a frontend pair mandatory. Anchors: lines 77-79.
- **I002-I10-033 [E][trigger]** The observable is a two-turn CLI run emitting SSB. Anchors: lines 80-82.
- **I002-I10-034 [E][schema]** `TtlPlanned` includes `ttl_seconds: 3600|300` and `breakeven_reads`. Anchor: line 82.
- **I002-I10-035 [E][acceptance]** Turn 2 must journal `cache_read_input_tokens > 0` over the frozen prefix. Anchor: line 83.
- **I002-I10-036 [E][acceptance]** An injected timestamp must trip the guard within one session and identify the leaking mutation/byte range. Anchors: lines 82-83.
- **I002-I10-037 [E][broken-arm]** Disabling the guard must fail because the injected mutation no longer trips. Anchor: line 83.
- **I002-I10-038 [E][integration]** H-9's wire-dark hit-rate monitor becomes planner data, and E-39 consumes economics signals. Anchors: lines 92-97.
- **I002-I10-039 [E][done-gate]** Full-form expansion is mandatory before execution, including live re-grep of IS-5 and the IC-5 choice. Anchors: lines 98-99.

I-10 ambiguity/conflict register:

- **I002-I10-040 [A][layout]** Inline versus deferred tool catalogs remains a runtime measurement decision, not a source-level constant. Anchors: lines 14-15, 71, 95.
- **I002-I10-041 [A][UI]** `CostDrawer` versus `CachePanel` is plan-asserted naming and must be re-grepped. Anchor: line 50.
- **I002-I10-042 [C][strategy]** The plan corrects heartbeat-primary to burst/TTL-primary and corrects any assumption that H-4 deferral must be overridden. Anchors: lines 14-15, 23-26, 87.

### 5. `I-29_diffusion_draft_lane.md`

Source fully read: 109/109 lines. Metadata: Track 16, `GENERATIVE`, moderate grounding, build, L effort, Wave 5; dependency and network tiers require user approval.

- **I002-I29-001 [E][generation]** A Mercury-class diffusion LLM is a fast, explicitly untrusted draft generator with no privileged status. Anchors: lines 1-8.
- **I002-I29-002 [E][security]** Every diffusion output traverses the identical I-17/I-20 verification/mutation stack used by autoregressive output. Anchors: lines 4-6, 19-21.
- **I002-I29-003 [E][non-goal][C]** The full TTD-DR denoising/research loop is declined; the narrow steal is draft speed, not autonomy. Anchors: lines 7-9, 28-30, 79.
- **I002-I29-004 [E][permissions]** No new dependency or network code may land before user approval. Anchors: lines 8-9, 49-53.
- **I002-I29-005 [E][done-gate]** L effort requires full-form expansion before execution. Anchor: line 11.
- **I002-I29-006 [E][integration]** The dLLM client adds a `RuntimeAdapter` variant and must not fork the trait. Anchors: lines 13-15, 57, 65, 102-104.
- **I002-I29-007 [E][security]** Before naming a client crate, executor must check npm/PyPI/cargo, repository URL, last-published date, and license, then obtain user confirmation. Anchors: lines 14-16, 49-51, 80.
- **I002-I29-008 [E][workflow]** The orchestrator assigns a subtask to the diffusion lane. Anchors: lines 16-17.
- **I002-I29-009 [E][workflow]** The dLLM creates a one-pass or few-step parallel-fill draft. Anchors: lines 17-18.
- **I002-I29-010 [E][workflow]** Multiple sub-agent patches flow through `draft_merge.rs`; the merged result then enters I-17 and surviving output enters I-20. Anchors: lines 18-21.
- **I002-I29-011 [E][concurrency]** Parallel agents append patch operations to a shared CRDT log. Anchors: lines 21-23.
- **I002-I29-012 [E][determinism]** Merge is deterministic and commutative. Anchors: lines 21-23.
- **I002-I29-013 [E][conflict-resolution]** Character/line conflicts use last-write-wins on a content hash, never LLM judgment. Anchors: lines 22-24, 83.
- **I002-I29-014 [E][validation]** C-32 MiniCritic runs at every semantic merge checkpoint. Anchors: lines 23-24, 42-43, 69.
- **I002-I29-015 [E][failure]** A compile-failing diffusion draft produces the same hard-veto/pruning outcome as an AR draft. Anchors: lines 19-20.
- **I002-I29-016 [E][integrity]** Each `DraftVetoed` row must reference a `TriLayerVerdict(HardVeto)` row; diffusion has no shortcut gate. Anchors: lines 24-26, 45-46, 78, 90-91.
- **I002-I29-017 [E][rollout]** Default-on is forbidden until a corruption rate is bounded over at least three real runs. Anchors: lines 26-28, 38, 52, 81.
- **I002-I29-018 [E][metric]** Corruption rate is defined as the fraction of drafts reaching Layer-1 hard veto that are genuinely bad rather than merely slow. Anchors: lines 26-27.
- **I002-I29-019 [E][flags]** Until promotion, the lane is an allowlisted experiment in `wire-dark-allowlist.toml`, following `darwinian-evolve`. Anchors: lines 27-28, 44, 58, 97.
- **I002-I29-020 [E][dependency][A]** B-37, C-32, and E-48 are plan-asserted seams that must be re-grepped. Anchors: lines 32-35.
- **I002-I29-021 [E][dependency]** I-17 is a blocking prerequisite for live gate parity. Anchor: line 36.
- **I002-I29-022 [E][status]** At authoring, neither dLLM client nor merge engine was production-reachable. Anchor: line 37.
- **I002-I29-023 [E][scope]** Tier 1 creates an approximately 100-LOC, in-process, dependency-free `DraftMerger` skeleton with patch log, content-hash resolution, and MiniCritic call sites. Anchors: lines 40-43, 55-56, 63-64.
- **I002-I29-024 [E][config]** Tier 1 adds an allowlist row containing `type=experiment`, owner, expiry date, `promotion_plan=I-29`, and smoke-test path. Anchor: line 44.
- **I002-I29-025 [E][test]** Tier 1 adds `i29_diffusion_draft_lane_contract.rs` with a seeded veto-parity fixture. Anchors: lines 45-46.
- **I002-I29-026 [E][artifact]** Tier 1 adds `workflow/reasoning/I-29-decisions.md`. Anchor: line 47.
- **I002-I29-027 [E][deferred]** Tier 2 defers the approved dLLM `DiffusionAdapter` and live endpoint wiring. Anchors: lines 49-52, 57.
- **I002-I29-028 [E][deferred]** Default-on promotion and a parallel-volume test remain gated on live client/evidence. Anchors: lines 52-53.
- **I002-I29-029 [E][UI]** Generation cards display a draft-lane badge with `DraftAccepted`, `DraftVetoed`, and `CrdtMerged`; `cross_lang_diff.py` covers Tauri-to-Svelte. Anchors: lines 60, 85-86.
- **I002-I29-030 [E][schema]** Produce `DiffusionDraftEvent { DraftAccepted, DraftVetoed, CrdtMerged }`, baseline +1. Anchors: lines 85-86, 99.
- **I002-I29-031 [E][integration]** Consumed symbols are B-37 `RuntimeAdapter`, C-32 MiniCritic, I-17 tri-layer veto, and I-20 budgeted mutation, each reused rather than rebuilt. Anchors: lines 67-71, 102-107.
- **I002-I29-032 [E][reachability]** The eventual production chain is search ladder/task runner -> `DiffusionAdapter` -> `DraftMerger` -> I-17. Anchor: line 74.
- **I002-I29-033 [E][reachability]** Direct Tier-1 contract calls do not satisfy R7; only the gated Tier-2 chain establishes production reachability. Anchor: line 75.
- **I002-I29-034 [E][constraint]** No placeholder/TODO for the declined TTD-DR loop may appear. Anchors: lines 77-80.
- **I002-I29-035 [E][implementation]** Production forbids `.unwrap()`/`.expect()` and blake2/sha2. Anchor: line 82.
- **I002-I29-036 [E][trigger]** Tier 1 uses the wiring-contract fixture harness; gated Tier 2 uses `elai run --diffusion-lane`. Anchors: lines 88-90.
- **I002-I29-037 [E][acceptance]** A two-agent patch fixture emits `CrdtMerged`, and a clean draft emits `DraftAccepted`. Anchor: line 90.
- **I002-I29-038 [E][determinism-test]** Two seeded runs of parallel patches must produce byte-identical output and the same content hash. Anchor: line 91.
- **I002-I29-039 [E][regression]** With the diffusion lane off, every existing generation path remains untouched. Anchor: line 91.
- **I002-I29-040 [E][negative-test]** Gate bypass is structurally impossible; the contract asserts the `DraftVetoed -> HardVeto` join. Anchor: line 91.
- **I002-I29-041 [E][evidence][A]** The speed claim is moderate/[DESIGN]/[WEAK] until Tier 2 lands and ELAI measures it; SSB planning is strong. Anchors: lines 93-100.
- **I002-I29-042 [E][evidence]** Any Mercury-class paper citation must be web-verified before headlining; no paper is asserted for the hand-rolled CRDT. Anchor: line 100.

I-29 ambiguity/conflict register:

- **I002-I29-043 [A][implementation]** The detail file resolves the summary's “Automerge-class versus hand-rolled” choice toward an in-process hand-rolled Tier-1 skeleton with no external dependency. Anchors: lines 42-43, 98-100.
- **I002-I29-044 [C][status]** Contrary to a broad reading of “adds a diffusion LLM,” only the deterministic merge skeleton/test/config ship in Tier 1; the actual client and network lane are explicitly deferred pending approval. Anchors: lines 40-53.
- **I002-I29-045 [A][threshold]** The source says default-on needs `>=3` real runs, while its stub-ban phrase “`<=3` runs are insufficient” could imply strictly more than three; implementation should preserve the clearer `>=3` rule or resolve the wording. Anchors: lines 26-28, 38, 52, 81.
- **I002-I29-046 [I][failure]** A missing referenced hard-veto row necessarily makes gate-parity evidence invalid, even though the source expresses this through the contract join rather than a named runtime recovery. Anchors: lines 24-26, 90-91.

### 6. `I-46_bash_interceptor_redirect.md`

Source fully read: 70/70 lines. Metadata: Track 20, `CHEAPER + CANT-WRECK`, strong seam/medium gajae semantics, extension, S-M effort, Wave 2.

- **I002-I46-001 [E][premise]** Existing validation allows/denies, while interception can recover dead tool calls, reduce raw-output context floods, and journal substitutions. Anchors: lines 6-10.
- **I002-I46-002 [E][compliance]** Before execution, read gajae `bash-interceptor.ts`, `bash-command-fixup.ts`, and `bash-allowed-prefixes.ts`. Anchors: lines 12-14, 59-61.
- **I002-I46-003 [E][action]** The interceptor parses/classifies each bash command before existing bash validation. Anchors: lines 15-16, 41-45.
- **I002-I46-004 [E][redirect]** Bare grep/find/cat/head/tail covered by registered search/read tools are refused with redirect. Anchors: lines 15-16.
- **I002-I46-005 [E][response]** Redirect response includes the exact registered substitute tool call. Anchor: line 16.
- **I002-I46-006 [E][fixup]** A small frozen deterministic table repairs known-broken command shapes such as flag order and missing quoting. Anchors: lines 17-18.
- **I002-I46-007 [E][constraint]** Fixup is allowed only when semantics-preserving. Anchor: line 18.
- **I002-I46-008 [E][allowlist]** A bounded frozen allowed-prefix table identifies commands that pass unchanged. Anchors: lines 18-19, 31-33.
- **I002-I46-009 [E][non-goal]** The interceptor performs no LLM-driven rewrite. Anchors: lines 20-22, 48.
- **I002-I46-010 [E][security]** It never auto-executes a proposed substitute; the agent must reissue it so the supervisor/audit trail remains intact. Anchors: lines 20-22, 48.
- **I002-I46-011 [E][invariant]** Interception may narrow, never widen, existing validation. Anchors: lines 21-22.
- **I002-I46-012 [E][status][A]** `bash_validation.rs` is shipped, while redirect absence is STEAL-LIST-asserted and must be re-grepped. Anchors: lines 24-26.
- **I002-I46-013 [E][architecture]** E-30 argv rules, planned I-35 sequence rules, and I-46 substitution are disjoint enforcement dimensions. Anchors: lines 27-29.
- **I002-I46-014 [E][scope]** Ship now: interceptor, fixup table, prefix table, and read/search-family redirects in the existing validation path. Anchors: lines 31-33.
- **I002-I46-015 [E][deferred]** Recipe-runner redirects wait for I-53. Anchor: line 34.
- **I002-I46-016 [E][deferred]** Per-project custom prefix configuration is deferred. Anchor: line 34.
- **I002-I46-017 [E][integration]** Extend `crates/tools/src/bash_validation.rs` and bash dispatch; do not fork validation. Anchors: lines 36-38.
- **I002-I46-018 [E][integrity]** Fixup/prefix tables are R8-frozen repo artifacts and a run cannot edit its own rewrite rules. Anchors: lines 37-38.
- **I002-I46-019 [E][UI]** Redirect rows appear in the existing tool-call drawer; no component is added. Anchor: line 39.
- **I002-I46-020 [E][symbols]** Produce `tools::bash_interceptor`, frozen tables, and `BashRedirected` SSB rows. Anchors: lines 41-43.
- **I002-I46-021 [E][integration]** Consume the existing validation verdict path, tool registry, and unchanged downstream E-30 argv allowlist. Anchors: lines 44-45.
- **I002-I46-022 [E][reachability]** Every production agent bash call traverses the interceptor; tests alone do not satisfy R7. Anchors: lines 46-47.
- **I002-I46-023 [E][security]** No bypass flag may skip interception. Anchors: lines 48-49.
- **I002-I46-024 [E][parsing]** Regex-only “looks like grep” detection is forbidden; argv parsing is required. Anchors: lines 49-50.
- **I002-I46-025 [E][implementation]** Production paths forbid `.unwrap()`. Anchor: line 50.
- **I002-I46-026 [E][trigger]** The live trigger is an agent bash call such as `grep -rn pattern crates/`. Anchors: lines 51-53.
- **I002-I46-027 [E][observability]** That call emits `BashRedirected(Redirected)` naming its substitute. Anchors: lines 52-54.
- **I002-I46-028 [E][observability]** A known-fixup fixture emits `BashRedirected(FixedUp)`. Anchors: lines 53-54.
- **I002-I46-029 [E][observability]** A novel non-allowlisted prefix emits `BashRedirected(PrefixBlocked)`. Anchor: line 54.
- **I002-I46-030 [E][acceptance]** Redirected commands create no process in the journal window. Anchors: lines 55-57.
- **I002-I46-031 [E][negative-test]** An allowed-prefix command executes unchanged with zero redirect rows. Anchors: lines 55-57.
- **I002-I46-032 [E][broken-arm]** A redirect row plus a spawned grep in the same window fails. Anchor: line 57.
- **I002-I46-033 [E][schema]** Add `BashRedirected { Redirected, FixedUp, PrefixBlocked }`, baseline +1, and re-read live constants. Anchors: lines 64-68.
- **I002-I46-034 [E][interaction]** H-4 discovers substitute tools; I-53 supplies future recipe targets; I-35 remains a separate sequence layer. Anchors: lines 64-66.

I-46 ambiguity/inference register:

- **I002-I46-035 [A][config]** The initial allowed-prefix/fixup table contents are not enumerated; only the read/search redirect family and the requirement that tables be bounded/frozen are explicit. Anchors: lines 15-19, 31-34.
- **I002-I46-036 [A][evidence]** Redirect absence and gajae behavioral parity remain medium-confidence until the mandatory source read/re-grep. Anchors: lines 3-4, 13-14, 24-26.
- **I002-I46-037 [I][failure]** A blocked prefix returns a journaled refusal rather than falling through, inferred from `PrefixBlocked` plus the no-bypass/narrowing invariants. Anchors: lines 21-22, 48-54.

### 7. `I-54_structural_ast_edit_tool.md`

Source fully read: 75/75 lines. Metadata: Track 20, `TOOLS-VERIFIED`, strong tree-sitter/medium gajae semantics, build, M effort, Wave 3.

- **I002-I54-001 [E][premise]** I-54 steals structural-search/edit ergonomics but builds on shipped tree-sitter 0.26 and nine in-tree grammars. Anchors: lines 1-12.
- **I002-I54-002 [E][compliance]** Before execution, read gajae `ast-edit.ts` and `ast-grep.ts` for interface behavior only. Anchors: lines 14-16.
- **I002-I54-003 [E][parser]** The engine uses tree-sitter query syntax and introduces no new pattern DSL. Anchors: lines 9-12.
- **I002-I54-004 [E][rewrite]** Rewrites use capture templates. Anchors: lines 10-12.
- **I002-I54-005 [E][tool]** Search maps a tree-sitter query to matches with file, line, and capture bindings. Anchors: lines 17-18.
- **I002-I54-006 [E][tool]** Preview maps query plus rewrite template to a would-be diff and writes nothing. Anchors: lines 18-19.
- **I002-I54-007 [E][tool]** Apply executes only the previewed edit set. Anchors: lines 18-20.
- **I002-I54-008 [E][atomicity]** Apply is atomic per file. Anchors: lines 18-20.
- **I002-I54-009 [E][staleness]** Apply refuses if any target file changed since preview. Anchor: line 19.
- **I002-I54-010 [E][security]** Structural apply traverses the identical PreToolUse edit gates used by ordinary Edit. Anchors: lines 20-21, 38-40, 47-48.
- **I002-I54-011 [E][dependency][C]** Any dependency named `ast-grep` is a review-blocking violation. Anchors: lines 22-24, 51-53.
- **I002-I54-012 [E][scope]** A single call does not perform cross-language rewrite; queries are per language. Anchors: lines 22-23.
- **I002-I54-013 [E][scope]** The tool is syntax-only; semantic/type-aware renaming remains E-52 LSP territory. Anchors: lines 23-24.
- **I002-I54-014 [E][workflow]** Apply without a preceding preview row is forbidden. Anchors: lines 23-24, 51-52.
- **I002-I54-015 [E][parser][status]** The nine grammars are Rust, Python, Go, TypeScript, Java, C#, Ruby, JavaScript, and C. Anchors: lines 26-29.
- **I002-I54-016 [E][architecture]** Tools consume a public knowledge-crate parser/grammar API; no other crate may pin a second tree-sitter version. Anchors: lines 27-29, 38-40, 52-53.
- **I002-I54-017 [E][interaction]** Harness-side `impact_radius.py`/`elai_graph` are blast-radius siblings; preview is the runtime-side equivalent surface. Anchors: lines 30-31.
- **I002-I54-018 [E][scope]** Tier 1 ships catalog-registered, gated search/preview/apply for Rust, Python, and TypeScript. Anchors: lines 33-35.
- **I002-I54-019 [E][deferred]** The remaining six grammars are enabled only on demand. Anchor: line 36.
- **I002-I54-020 [E][deferred]** A rewrite-template library is deferred. Anchor: line 36.
- **I002-I54-021 [E][integration]** Create `crates/tools/src/structural_edit.rs`, register it in the tool catalog, and route it through the edit-gate path. Anchors: lines 38-40.
- **I002-I54-022 [E][UI]** Existing tool-call drawer rows show preview and apply, including the operator-visible diff; no new component is added. Anchors: lines 41-42.
- **I002-I54-023 [E][symbols]** Produce `tools::structural_edit`, the staleness guard, and `StructuralEditApplied` rows. Anchors: lines 44-46.
- **I002-I54-024 [E][integration]** Consume the single knowledge parser version, F-1 PreToolUse edit gates, and the tool registry. Anchors: lines 47-48.
- **I002-I54-025 [E][reachability]** Production reachability is a live agent invoking search -> preview -> apply on a real crate; contracts alone do not satisfy R7. Anchors: lines 49-50.
- **I002-I54-026 [E][failure]** Parse failure returns typed `ParseFailed`; regex/string-replace fallback is forbidden. Anchors: lines 51-53.
- **I002-I54-027 [E][security]** Edit-gate bypass is forbidden. Anchor: line 53.
- **I002-I54-028 [E][implementation]** Production paths forbid `.unwrap()`/`.expect()`. Anchor: line 53.
- **I002-I54-029 [E][trigger]** The live fixture structurally renames a function and its call sites across a fixture crate. Anchors: lines 54-55.
- **I002-I54-030 [E][observability]** `Matched` records match count, `Previewed` records the diff hash, and `Applied` records per-file results. Anchors: lines 56-57.
- **I002-I54-031 [E][failure]** A mid-flight external file change emits `RefusedStale`. Anchor: line 57.
- **I002-I54-032 [E][acceptance]** The edited fixture must compile. Anchors: lines 58-59.
- **I002-I54-033 [E][acceptance]** Post-apply bytes may touch only previewed spans and must equal the preview diff. Anchors: lines 58-60.
- **I002-I54-034 [E][negative-test]** A syntactically broken rewrite template fails during preview with `ParseFailed` and writes zero bytes. Anchors: lines 59-60.
- **I002-I54-035 [E][broken-arm]** Apply without a preview row fails under R1. Anchors: lines 60-61.
- **I002-I54-036 [E][schema]** Add `StructuralEditApplied { Matched, Previewed, Applied, RefusedStale, ParseFailed }`, baseline +1, with live W-9 reread. Anchors: lines 68-73.
- **I002-I54-037 [E][interaction]** Survivor Directives from I-20 may use structural edit as the fix vehicle; E-52 and impact mapping remain disjoint siblings. Anchors: lines 68-71.

I-54 ambiguity/inference register:

- **I002-I54-038 [A][claim]** The premise calls the ergonomics “rename-safe,” but the same file explicitly limits the tool to syntax-only edits and excludes semantic rename; safety means preview/gate/staleness integrity, not type-aware rename correctness. Anchors: lines 6-12, 22-24.
- **I002-I54-039 [A][API]** The required knowledge-crate public parser API is named architecturally but no exact symbol is specified. Anchors: lines 27-29, 38-40.
- **I002-I54-040 [I][idempotency]** Reapplying an already-consumed preview should be refused or produce no unpreviewed changes; this follows from preview-bound bytes/staleness but no explicit preview-consumption policy is named. Anchors: lines 18-24, 51-61.

## Cross-source interactions

### Explicit interaction ledger

- **I002-X-001 [E][context/search]** I-5 is a Wave-2 production dependency of I-4: each best-of-N expansion calls the pristine-context builder, while tests alone do not establish reachability. Sources: I-0 lines 273-277; I-5 lines 69-72, 108-113.
- **I002-X-002 [E][context/cache]** I-5 must reuse H-6/H-50 context machinery and simultaneously preserve the `update_breakpoints` prefix invariant that I-10 guards. Sources: I-5 lines 24-33, 64-68; I-10 lines 58-66.
- **I002-X-003 [E][measurement]** E-19 is the I-5 same-choice A/B surface; H-9's hit monitor is I-10 planner input; both convert a design claim into journal-derived evidence before rollout. Sources: I-5 lines 32-44; I-10 lines 92-97.
- **I002-X-004 [E][search/security]** I-29 can enter production only through search/task dispatch and must route merged drafts through I-17 then I-20 without a special verdict path. Sources: I-0 lines 309-322, 347-349; I-29 lines 67-75.
- **I002-X-005 [E][rollout]** I-29 uses the existing `darwinian-evolve` allowlist/kill-switch precedent, satisfying the index's W-2 exception for an experimental lane while default-on waits for GD-4 evidence. Sources: I-0 lines 194-195; I-29 lines 26-28, 44, 97.
- **I002-X-006 [E][tool-security]** I-46 runs before bash validation, then leaves E-30 argv checks downstream; I-35 sequence enforcement is a separate later layer. Sources: I-0 lines 370-372, 407-409; I-46 lines 24-29, 41-45, 64-66.
- **I002-X-007 [E][tool-discovery]** I-46 obtains substitute targets from the registered tool surface/H-4, and I-53 later supplies recipe targets; I-46 does not invent or execute the target. Sources: I-46 lines 41-45, 64-66; I-0 lines 428-430.
- **I002-X-008 [E][edit-security]** I-54 search is read-only, preview is write-free, and apply reuses F-1 PreToolUse edit gates; I-20 Directives may select structural edit as their fix vehicle. Sources: I-54 lines 17-24, 44-50, 68-71.
- **I002-X-009 [E][parser-ownership]** The knowledge crate is the single tree-sitter version/grammar owner; the tools crate consumes a public API and may not add an ast-grep or second tree-sitter dependency. Sources: grounding map lines 44-46, 78-79; I-54 lines 26-29, 38-53.
- **I002-X-010 [E][schema]** All five detailed sessions add one Pattern-2 action variant, so actual shipping order must reconcile the shared 221/532 baseline rather than independently adding “baseline +1.” Sources: I-0 lines 438-453, 473, 491, 501-503.
- **I002-X-011 [E][evidence]** Grounding-map “shipped/absent” facts and detail-file plan assertions never waive W-7/W-9 live rechecks. Sources: grounding map lines 5-6, 12-21; I-0 lines 192-204.
- **I002-X-012 [E][permissions]** I-29's client is blocked on dependency approval; I-54 explicitly avoids a dependency; I-46 depends only on shipped tables/validation. This produces three distinct dependency postures despite all being ports/builds. Sources: I-0 lines 538-544; I-29 lines 49-53; I-46 lines 31-38; I-54 lines 22-29.

### Inferred composition consequences

- **I002-X-013 [I][failure-propagation]** A broken I-5 prefix-preservation implementation can surface as an I-10 prefix-diff/guard trip; the shared `update_breakpoints` seam makes this a plausible cross-feature diagnostic, though no direct event join is specified. Sources: I-5 lines 24-27, 64-66; I-10 lines 60-66, 80-83.
- **I002-X-014 [I][determinism]** I-29's content-hash merge and I-54's preview diff hash both bind later mutation to earlier bytes, but they are separate mechanisms and share no stated hash schema. Sources: I-29 lines 21-24, 88-91; I-54 lines 18-20, 54-60.
- **I002-X-015 [I][observability]** Because I-46 refuses shell grep and returns a registered search call, subsequent evidence should appear under the substitute tool's normal rows in addition to `BashRedirected`; the exact correlation field is unspecified. Sources: I-46 lines 15-22, 41-57.

### Consolidated ambiguity/conflict ledger

- **I002-X-016 [A][status]** The corpus was authored against 2026-06-10 live state; no statement in this chunk proves the same modules/constants still hold at implementation time. W-7/W-9 explicitly demand reread.
- **I002-X-017 [A][rollout]** I-29 says both “at least three runs” and “three or fewer are insufficient”; the threshold wording conflicts and needs resolution before promotion.
- **I002-X-018 [A][UI]** I-5's tile is deferred/wire-dark, I-10's exact cost-panel component name is plan-asserted, and I-46/I-54 reuse an existing drawer; only the interaction contract is stable, not every current frontend symbol.
- **I002-X-019 [A][idempotency]** The corpus is explicit about deterministic replay, CRDT merge, atomic file application, and stale preview rejection, but it does not define replay deduplication keys or single-consumption semantics for I-29/I-54 operations.
- **I002-X-020 [C][summary-scope]** High-level phrases such as “adds a diffusion LLM” and “adds a vividness tile” overstate Tier-1 delivery unless read with the detail files: both live dLLM access and the I-5 tile are gated/deferred.

## Summary additions and corrections

Comparison target: `09-I-SUMMARY_INDEX.discovery.md`, used only for orientation. “Addition” means the deep sources make an explicit behavior that the oriented summary entry did not record; “correction” means the deep source narrows, resolves, or contradicts the summary wording.

- **I002-SD-001 [addition][scope]** The exact DEEP-I-002 subset is now integrity-verified and fully read: one Track-20 grounding map, I-0, and I-5/I-10/I-29/I-46/I-54—1,169 lines/99,250 bytes. The summary explicitly left deep sources uninspected.
- **I002-SD-002 [addition][evidence]** The auxiliary map is direct evidence for shipped/absent seams, mandatory competitor-source reads, SLC corrections, and the dated live constants; it is not merely an inventory row. Deep anchors: grounding map lines 1-94.
- **I002-SD-003 [addition][workflow]** I-0 specifies a seven-step predecessor-reading order and mandatory full-form expansion for L/XL sessions. Deep anchors: I-0 lines 58-74.
- **I002-SD-004 [addition][status]** I-0's session log is empty, and DONE has a seven-part gate including delegated memory writing; this prevents treating planned features as current implementation status. Deep anchors: I-0 lines 184-187, 251-258, 590-593.
- **I002-SD-005 [addition][flags]** W-2's terminal-gate rule is more exact: new I gates ship default-on behind disable flags, while I-29/I-31 reuse existing allowlisted kill-switch lanes. Deep anchors: I-0 lines 192-204.
- **I002-SD-006 [addition][evidence]** W-7/W-9 require both execution and audit re-greps plus live schema recounts, qualifying every dated “shipped” and 221/532 claim. Deep anchors: I-0 lines 200-204, 438-441.
- **I002-SD-007 [addition][lifecycle]** ID-3 requires reversible receipts for every context-to-wire transform and exact reconstruction of what the model saw. Deep anchors: I-0 lines 231-234.
- **I002-SD-008 [addition][schema]** I-0 supplies a complete 54-session action-type ledger, including exact Kind enums, reuse decisions, and two conditional decisions (I-15/I-52). Deep anchors: I-0 lines 443-503.
- **I002-SD-009 [addition][sequencing]** I-0 records five exact waves and additional dependency order, including I-4 behind I-17/I-20 and provider/auth work behind router/catalog substrate. Deep anchors: I-0 lines 505-533.
- **I002-SD-010 [addition][implementation]** The deep index adds production coding constraints, auditor/test boundaries, dependency approval, no-removal permission, decision-doc, and commit-cadence rules. Deep anchors: I-0 lines 535-546.
- **I002-SD-011 [addition][compatibility]** The index adds explicit sub-second RPM jittering, cache-entry order, constrained-decoding attack posture, action-stream-scoped reproducibility, and review-time—not generation-time—product focus. Deep anchors: I-0 lines 549-578.
- **I002-SD-012 [correction][I-5 UI]** The summary says I-5 adds a ReasoningDrawer tile; the detail file makes it Tier 2, wire-dark, and W-6-justifiably absent in v1 until at least three consistent A/B runs and GD-4 evidence. Deep anchors: I-5 lines 47-50, 57-58, 81-83.
- **I002-SD-013 [addition][I-5 scope]** Tier 1's exact deliverables are `pristine_path_filter.rs`, the three-Kind `ContextPruned` row, sampled dual-arm probe wiring, I-4 integration, and an I-5 decision document. Deep anchors: I-5 lines 37-46.
- **I002-SD-014 [addition][I-5 sampling]** The A/B runs only on a sampled fraction of branch expansions; it is not a mandatory dual generation on every branch. Deep anchors: I-5 lines 43-44.
- **I002-SD-015 [addition][I-5 non-goals]** Truncation is rejected separately from raw-trace carrying, and a new masking layer is forbidden. Deep anchors: I-5 lines 18-21, 73-80.
- **I002-SD-016 [addition][I-5 status]** H-6/H-50/E-19 are plan-asserted/re-grep dependencies rather than current-state proof in this file. Deep anchors: I-5 lines 24-35, 94-106.
- **I002-SD-017 [addition][I-10 rate-limit]** I-10 includes `max_tokens_clamp` for schema-bound scoring calls, a material Tier-1 capability absent from the oriented I-10 summary ledger. Deep anchors: I-10 lines 21-22, 36-37, 53-57.
- **I002-SD-018 [addition][I-10 ordering]** Long-TTL cache entries must precede short-TTL entries in each request. Deep anchors: I-10 lines 16-18.
- **I002-SD-019 [addition][I-10 economics]** The planner's dated write-premium/breakeven defaults and mandatory pricing reverification are explicit. Deep anchors: I-10 lines 19-20, 85-90.
- **I002-SD-020 [addition][I-10 rollout]** Global default-on waits for three real runs, and automated price reverification is deferred as monitoring rather than a gate. Deep anchors: I-10 lines 43-45.
- **I002-SD-021 [addition][I-10 observability]** `TtlPlanned` carries `ttl_seconds` and `breakeven_reads`; the acceptance fixture requires `cache_read_input_tokens > 0` on turn 2. Deep anchors: I-10 lines 80-83.
- **I002-SD-022 [addition][I-10 status]** Cache plumbing existed but Anthropic markers remained `None`, while the planner/selector/guard were absent at authoring and remain W-7 recheck facts. Deep anchors: I-10 lines 28-33.
- **I002-SD-023 [correction][I-29 scope]** The summary's broad “adds a diffusion LLM” is not Tier-1 delivery: only the in-process merge skeleton, allowlist row, contract, and decision doc ship now; client/network/default-on/volume test are deferred and approval-gated. Deep anchors: I-29 lines 40-53.
- **I002-SD-024 [correction][I-29 implementation]** The summary leaves Automerge versus hand-rolled merge open; the detail file selects a roughly 100-LOC dependency-free in-process skeleton with deterministic content-hash resolution. Deep anchors: I-29 lines 42-43, 55-65, 98-100.
- **I002-SD-025 [addition][I-29 config]** The allowlist row has required owner, expiry, promotion plan, and smoke-test fields. Deep anchor: I-29 line 44.
- **I002-SD-026 [addition][I-29 concurrency]** Parallel patches append to a shared CRDT log and conflicts resolve at character/line level with content-hash last-write-wins, never an LLM. Deep anchors: I-29 lines 21-24, 77-83.
- **I002-SD-027 [addition][I-29 reachability]** Tier-1 contract calls do not satisfy R7; production reachability exists only after the Tier-2 adapter path is live. Deep anchors: I-29 lines 73-75.
- **I002-SD-028 [conflict][I-29 threshold]** `>=3` real runs and “<=3 insufficient” are inconsistent threshold wordings and require resolution. Deep anchors: I-29 lines 26-28, 38, 52, 81.
- **I002-SD-029 [addition][I-29 event]** The full event enum/badge includes `DraftAccepted`, which the oriented summary's observability bullet omitted while naming only veto and merge. Deep anchors: I-29 lines 85-90, 99.
- **I002-SD-030 [addition][I-46 compliance]** Execution must first read three named gajae source files; medium-confidence summary semantics are not enough. Deep anchors: I-46 lines 12-14, 59-61.
- **I002-SD-031 [addition][I-46 parsing]** Fixups must be semantics-preserving; argv parsing is mandatory; regex-only classification and bypass flags are banned. Deep anchors: I-46 lines 17-18, 48-50.
- **I002-SD-032 [addition][I-46 deferred]** Recipe redirects and per-project prefix configuration are explicitly deferred. Deep anchor: I-46 line 34.
- **I002-SD-033 [addition][I-46 events]** Detail acceptance requires `FixedUp` and `PrefixBlocked` rows as well as `Redirected`, plus no-process and allowed-prefix negative arms. Deep anchors: I-46 lines 51-57, 67-68.
- **I002-SD-034 [addition][I-46 integrity]** Fixup/prefix tables are R8-frozen and a run may not alter its own redirect policy. Deep anchors: I-46 lines 36-38.
- **I002-SD-035 [addition][I-54 compliance]** Execution must read the two named gajae AST tools for interface semantics while retaining a different engine. Deep anchors: I-54 lines 14-16.
- **I002-SD-036 [addition][I-54 schema]** Search results include file/line/capture bindings and rewrites are capture-template based. Deep anchors: I-54 lines 9-12, 17-19.
- **I002-SD-037 [addition][I-54 scope]** Cross-language rewrites in one call are forbidden; each query is per-language. Deep anchors: I-54 lines 22-24.
- **I002-SD-038 [addition][I-54 deferred]** Six remaining named grammars and a rewrite-template library are deferred. Deep anchors: I-54 lines 26-36.
- **I002-SD-039 [addition][I-54 events]** The complete event enum adds `RefusedStale` and `ParseFailed` to Matched/Previewed/Applied. Deep anchors: I-54 lines 54-57, 68-73.
- **I002-SD-040 [addition][I-54 failure]** Parse failure is typed and may never fall back to regex/string replacement; tools may not pin a second tree-sitter version. Deep anchors: I-54 lines 26-29, 51-53.
- **I002-SD-041 [ambiguity][I-54 claim]** “Rename-safe” is limited by the explicit syntax-only/non-semantic scope; no type-aware rename guarantee is earned. Deep anchors: I-54 lines 6-12, 22-24.
- **I002-SD-042 [addition][grounding status]** The map confirms shipped RRA and checkpoint substrate, correcting stale absence/build assumptions and making I-45 an extension and I-52 an activation. Deep anchors: grounding map lines 32-42, 72-74.
- **I002-SD-043 [addition][grounding gaps]** The map records Anthropic-only OAuth, scattered catalog data with duplicate VRAM-tier module names, no Bedrock/Azure, and no recipe runner. Deep anchors: grounding map lines 23-30, 48-57.
- **I002-SD-044 [addition][source-read gate]** Named Track-20 executors have binding direct-source-read obligations for both gajae and roach-pi/lazycodex references. Deep anchors: grounding map lines 59-67.
- **I002-SD-045 [addition][schema]** Track 20's maximum is six new variants; four sessions reuse existing rows. Deep anchors: grounding map lines 81-86.
- **I002-SD-046 [addition][I-49 deferred verification]** AWS/Azure protocol and candidate-crate verification is deliberately deferred to I-49 authoring time. Deep anchors: grounding map lines 89-94.

## Zero-omission and duplicate audit

| Processed source | Lines read | Primary atomic IDs | Section/line coverage audit |
|---|---:|---:|---|
| `_grounding/I-STEAL-GROUNDING-MAP.md` | 96/96 | 32 | provenance 1-6; SL findings 8-67; corrections 69-79; constants 81-86; web/defer 88-94; terminator 96 |
| `I-0_INDEX.md` | 603/603 | 353 | provenance/reads 1-74; track rationale/declines 76-182; status/doctrine 184-258; all 54 sessions 260-433; all 54 schema decisions 435-503; sequencing 505-533; hard rules/landmines/corrections 535-588; log/next step 590-601; terminator 603 |
| `I-5_pristine_context_injector.md` | 115/115 | 38 | metadata/premise 1-9; build/non-goals 11-22; ground truth 24-35; tiering 37-50; integration/wiring/observable 52-92; grounding/composition 94-113; terminator 115 |
| `I-10_prefix_invariance_ttl_tiering.md` | 101/101 | 42 | metadata/premise 1-7; build/non-goals 9-26; ground truth 28-33; tiering 35-45; integration/wiring/observable 47-83; grounding/composition 85-99; terminator 101 |
| `I-29_diffusion_draft_lane.md` | 109/109 | 46 | metadata/premise/full-form 1-11; build 13-30; ground truth 32-38; tiering 40-53; integration/wiring/observable 55-91; grounding/composition 93-107; terminator 109 |
| `I-46_bash_interceptor_redirect.md` | 70/70 | 37 | metadata/premise 1-10; build 12-22; ground truth 24-29; tiering 31-34; integration/wiring/observable 36-57; grounding/composition 59-68; terminator 70 |
| `I-54_structural_ast_edit_tool.md` | 75/75 | 40 | metadata/premise 1-12; build 14-24; ground truth 26-31; tiering 33-36; integration/wiring/observable 38-61; grounding/composition 63-73; terminator 75 |
| **Total** | **1,169/1,169** | **588** | **all source sections accounted** |

Audit results:

- Manifest membership: **7/7 exact**, with **0 extra original source files** read.
- Manifest integrity: **7/7 SHA-256**, **7/7 line counts**, **7/7 byte counts**, and both totals match.
- Primary source-feature atoms: **588**.
- Secondary analytic atoms: **20 interactions + 46 summary deltas = 66**.
- Total unique atomic IDs: **654**.
- Duplicate atomic IDs: **0**.
- ID sequence gaps: **0**; every namespace is contiguous from `001` through its recorded maximum.
- Semantic duplicate policy: repeated wording inside a source was consolidated into one primary atom with all relevant anchors. Cross-source interaction and summary-delta atoms are separately typed analytical relationships, not second claims that the underlying feature is new.
- Non-feature lines: headings, separators, blank lines, and `End of ...` terminators were read and coverage-accounted but correctly produce no feature atom.
- Explicit/inferred/ambiguous/conflicting separation: labels are inline on every primary/interaction atom; each detailed file and the index/grounding map also has a dedicated ambiguity or conflict register where needed.

Artifact metrics after finalization:

- Files processed: **7**.
- Source lines processed: **1,169**.
- Source bytes verified: **99,250**.
- Atomic IDs: **654** (**588 primary**, **66 analytic**).
- Output logical lines: **819**.
- Output bytes: **110854**.
- Canonical output SHA-256: `62585440b3b781a2baebf7d3d8ea7e7f25cbc0f2311aa9f4aa59bb3586aee201`.

The canonical output digest is SHA-256 of this final file after normalizing only the 64 hexadecimal characters in the preceding digest field to 64 ASCII zeroes. This avoids an impossible byte-self-reference while making the in-band digest reproducible; the byte-exact final-file SHA-256 is reported in the completion handoff.
