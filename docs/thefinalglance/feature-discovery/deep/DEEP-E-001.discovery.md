# DEEP-E-001 — exhaustive original-source discovery

## Scope, method, and accounting

- **Processed chunk:** `DEEP-E-001`, family E, exactly the nine manifest originals below. No E original outside this chunk was read. `05-E-SUMMARY_INDEX.discovery.md` was consulted only as an orientation/comparison baseline.
- **Source semantics:** these files specify designed work. They do not prove implementation. `SHIPPED`/`DONE`, `PLANNED`, `DEFERRED`, `POSITIONING`, `CONDITIONAL`, and `IN-FLIGHT` are retained as source claims.
- **Anchor syntax:** `E-19:47-65` means exact logical lines 47–65 of `E-19_reasoning_quality.md`; `IDX:147-157` means `E-0_INDEX.md`.
- **Classification:** all records in the per-file ledgers are **EXPLICIT**. Cross-file compositions are separately marked **INFERRED**. Unclear source statements are in **Ambiguities**; incompatible statements are in **Conflicts**.
- **Verified input accounting:** **9/9 files**, **2,134/2,134 logical lines**, **239,903/239,903 bytes**, all **9/9 SHA-256 values matched**.
- **Atomic-ID accounting:** `354` total unique records: `281` explicit-source atoms, `8` inferred interactions, `23` ambiguities, `7` conflicts, and `35` summary-delta records.
- **Output SHA-256:** `4da2635f557d09ebac844759e0712f7ddfc493483a267b54d75539d4cdfd6a6b` (normalized self-hash: replace the 64-hex value in this field with 64 ASCII zeroes, preserve all other bytes, then SHA-256 the file).

## Exact processed manifest and verification

Manifest header reproduced: family **E**; **9 files**, **2,134 logical lines**, **239,903 bytes**; target status **within 1,800–2,800**.

| Chunk | Class | Lines | Bytes | Manifest SHA-256 = observed SHA-256 | Exact absolute path |
|---|---|---:|---:|---|---|
| DEEP-E-001 | numbered/index original | 270 | 18,289 | `8fad1c0a31a6ff9c8ffa828c6e2805fd836cc4f416e0c0fc458d432401f525f1` | `docs/thefinalglance/E-plan/E-0_INDEX.md` |
| DEEP-E-001 | numbered/index original | 184 | 22,340 | `f6a35d5f9efd8d8ed61751cd54e65d32b2f2e9f1661cda21f4a7cb7e41d07bfe` | `docs/thefinalglance/E-plan/E-2_local_llm_speed_tricks.md` |
| DEEP-E-001 | numbered/index original | 198 | 19,422 | `3f46328b0820ccafb999c4ae74df2f2cf1f5eb489f50376ae8831b567b590b22` | `docs/thefinalglance/E-plan/E-6_multi_agent_failure_modes.md` |
| DEEP-E-001 | numbered/index original | 229 | 23,553 | `3dce448358fc23accc53938f8ec1b8e8091220d6cc230bb9399a965192983db9` | `docs/thefinalglance/E-plan/E-7_darwinian_self_improvement.md` |
| DEEP-E-001 | numbered/index original | 256 | 29,214 | `6a7c915beeb5a5e17de77256a6f71f8b9aad65d1ede8673b0c386a35e7d5ee59` | `docs/thefinalglance/E-plan/E-19_reasoning_quality.md` |
| DEEP-E-001 | numbered/index original | 233 | 29,080 | `1c34c64c706133e751d8cea6ba917b5ffa844e5689c1b52c4a8a9724cca5e2f5` | `docs/thefinalglance/E-plan/E-36_document_vision_rag.md` |
| DEEP-E-001 | numbered/index original | 237 | 29,480 | `054f001ed639b0b9ad76d6347139a7ec241b2d3c62c2e0eba3585f61b6e6d970` | `docs/thefinalglance/E-plan/E-38_ai_roadmap_2026_signals.md` |
| DEEP-E-001 | numbered/index original | 202 | 23,056 | `94bff276a560a86888f0f9e89aa65803162d8ffd8ec34fdcb31161e6e0f30ecf` | `docs/thefinalglance/E-plan/E-42_tree_sitter_grammars_2026.md` |
| DEEP-E-001 | numbered/index original | 325 | 45,469 | `254ac17c31ab7de92783328a0ae3381be96f170a512906a51f1005e6560c5489` | `docs/thefinalglance/E-plan/E-55_agentic_rl_and_reward.md` |
| **Total** |  | **2,134** | **239,903** | **9/9 match** |  |

## Per-file exhaustive ledgers

### E-0_INDEX.md — program/index ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E0-001 | IDX:1-15 | Defines the E plan as a 55-session, 55-report post-B/C/D improvement sweep. Each E-N is a self-contained implementation prompt, maps 1:1 to a grounded report, cites existing symbols through its `WIRING_CONTRACT`, and labels its relation as DELTA, EXTENSION, or REPLACEMENT; predecessor summaries and the sweep README are mandatory pre-reading before execution. |
| DEEP-E001-E0-002 | IDX:17-24 | Status lifecycle is `PLANNED` (written/not started), `DEFERRED`, `POSITIONING`, `IN PROGRESS`, or `DONE` (smoke + benchmark verified). All 55 entries were `PLANNED` at generation; the empty session log contains no evidence that any E item shipped. |
| DEEP-E001-E0-003 | IDX:30-54 | Top-20 priority order is operational, not merely topical: E-17, E-6, E-27, E-2, E-30, E-19, E-5, E-15, E-9, E-12, E-24, E-21, E-8, E-31, E-7, E-37, E-39, E-3, E-43, E-32; E-55/E-52/E-53/E-51/E-54 follow. “Ship-it-now beats strategic” within this list. |
| DEEP-E001-E0-004 | IDX:57-135 | The 55 entries are grouped into eleven waves: UX/memory; orchestration/critique; idea-to-product; validation/retrieval/routing; peers/Claude internals; inference/observability/incidents; model/UI/Rust/structured output; vision/security/strategy/economics/graphs; CI/grammars/skills/durability/drift; planning/fine-tune/research/metrics/SaaS; edge/LSP/formal/eval/RL. |

The following catalogue atoms preserve every indexed E-N capability and its exact E-0 schema signal. “—” means the index declares no bump.

| Atomic ID | E-N / wave / priority | Indexed capability | Indexed schema signal | Anchor |
|---|---|---|---|---|
| DEEP-E001-E0-005 | E-1 / W1 | Infinite-context illusion: Letta+Mem0+Zep+HippoRAG and sleep-time digest. | A-MEM Zettelkasten + MemGPT Core/Archival rows; `memory_promoted`, `memory_evicted`. | IDX:61,182 |
| DEEP-E001-E0-006 | E-2 / W1 / P4 | Local speed: SSE frame pacing and prompt-lookup decoding. | `stream_smoothed`. | IDX:36,62,183 |
| DEEP-E001-E0-007 | E-3 / W1 / P18 | Model handoff UX: show role, hide model; CUDA-graph warm pool. | `role_shown_model_hidden`. | IDX:50,63,184 |
| DEEP-E001-E0-008 | E-4 / W1 | Chat UX critique: sycophancy, persona drift, repair resistance, 14 UX rules. | `sycophancy_detected`, `persona_drift_alert`. | IDX:64,185 |
| DEEP-E001-E0-009 | E-5 / W1 / P7 | User modelling/memory: Mem0+Zep collapsed to SQLite; MINJA defense. | `memory_fact_added`, `memory_fact_updated`, `memory_fact_deleted`, `memory_fact_noop`. | IDX:39,65,186 |
| DEEP-E001-E0-010 | E-6 / W2 / P2 | MAST 14-mode multi-agent defense and iso-budget role audit; mandatory cross-family Adversary. | `mast_mode_classified`, `iso_budget_run`, `adversary_disagreement`. | IDX:34,68,187 |
| DEEP-E001-E0-011 | E-7 / W2 / P15 | Darwinian self-improvement hardened because DGM reward-hacked; Two-Gate required. | `two_gate_blocked`, `map_elites_promoted`, `est_quarantined`. | IDX:47,69,188 |
| DEEP-E001-E0-012 | E-8 / W2 / P13 | GEPA self-evolving prompts; cloud-Sonnet optimizer, local target. | `prompt_optimizer_proposed`, `prompt_optimizer_shadow`, `prompt_optimizer_promoted`. | IDX:45,70,189 |
| DEEP-E001-E0-013 | E-9 / W2 / P9 | Replace SWE-bench Verified headline with Live, Pro, tau2 and private evaluation; attribute scaffold vs model. | `bench_swelive_run`, `bench_tau2_run`, `harness_attribution_split`. | IDX:41,71,190 |
| DEEP-E001-E0-014 | E-10 / W2 | Hostile ELAI critic with 14 punches and five conceded strengths; empirically collapse roles. | `ablation_pruned_role`. | IDX:72,191 |
| DEEP-E001-E0-015 | E-11 / W3 | Idea-to-SaaS prior art; scope/product framing and Verified→Pro cliff. | `scope_score_below_threshold`, `slopsquat_block`, `production_grade_block`. | IDX:75,192 |
| DEEP-E001-E0-016 | E-12 / W3 / P10 | Production full-stack generation with day-one auth, RLS, observability, accessibility and headers. | `production_check_failed` (per checker), `rls_probe_missing`, `biome_deny_violation`. | IDX:42,76,193 |
| DEEP-E001-E0-017 | E-13 / W3 | TDD verification agents, PGS, mutation-test auditing and anti-over-mocking. | `mutant_survived`, `anti_over_mock_block`, `fuzz4all_finding`. | IDX:77,194 |
| DEEP-E001-E0-018 | E-14 / W3 | Browser E2E with AXTree+axe-core and deploy-preview worker. | `axtree_snapshot`, `axe_violation`, `deploy_preview_ready`. | IDX:78,195 |
| DEEP-E001-E0-019 | E-15 / W3 / P8 | Sandboxed execution via Firecracker/runsc/microsandbox; reject chroot. | `firecracker_snapshot_restored`, `seccomp_arg_blocked`. | IDX:40,79,196 |
| DEEP-E001-E0-020 | E-16 / W4 | VLM validation with AXTree-YAML, LPIPS convergence and intra-modal checks. | `rce_convergence_step`, `intra_modal_disagreement`. | IDX:82,197 |
| DEEP-E001-E0-021 | E-17 / W4 / P1 | Stage ONNX; true 3-ranker RRF, contextual retrieval, Jina reranking and FITM. | `onnx_model_staged`, `rerank_active`, `fitm_calibrated`. | IDX:32-33,83,198 |
| DEEP-E001-E0-022 | E-18 / W4 | Tool reliability via XGrammar Tier 2 and non-git SEARCH/REPLACE fences. | `xgrammar_constraint_enforced`, `search_replace_fenced`. | IDX:84,199 |
| DEEP-E001-E0-023 | E-19 / W4 / P6 | Reasoning: L2 is weakest; L4 AB-MCTS needs PRM-lite. | `prm_lite_score`, `l2_confidence_gate`. | IDX:38,85,200 |
| DEEP-E001-E0-024 | E-20 / W4 | Local/cloud routing with xRouter/RouterArena/C3PO and data-leakage rules. | `route_decision`, `data_leakage_blocked`. | IDX:86,201 |
| DEEP-E001-E0-025 | E-21 / W5 / P12 | Peer-worker design: AGENTLESS-style less decomposition; delete roles lacking 1σ lift. | `sbfl_localized`, `ace_audit_run`. | IDX:44,89,202 |
| DEEP-E001-E0-026 | E-22 / W5 | GitHub signals: export cross-vendor `SKILL.md`; microsandbox/libkrun and 12-factor audit. | `skill_exported`, `microsandbox_pool_size`, `twelve_factor_audit`. | IDX:90,203 |
| DEEP-E001-E0-027 | E-23 / W5 | 2026 developer-discourse positioning, not code. | — (`POSITIONING`; README/AGENTS/UX only). | IDX:91,204 |
| DEEP-E001-E0-028 | E-24 / W5 / P11 | External-boundary-only MCP; Streamable HTTP; schema/hash/sanitization hardening. | `mcp_tool_blake3_pinned`, `line_jumping_sanitized`, `streamable_http_active`. | IDX:43,92,205 |
| DEEP-E001-E0-029 | E-25 / W5 | Claude Code architecture: deterministic infrastructure, six-tier compaction, OpusPlan/prompt decisions. | `compaction_tier_N`, `opusplan_substate`, `prompt_decision_evaluated`. | IDX:93,206 |
| DEEP-E001-E0-030 | E-26 / W6 | Hybrid inference: retain sidecar, add SGLang adapter, deprecate Ollama. | `sglang_health_changed`, `ollama_deprecated`. | IDX:96,207 |
| DEEP-E001-E0-031 | E-27 / W6 / P3 | Serving quantization: AWQ+Marlin; FP8 KV; NF4 only for QLoRA. | `awq_loaded`, `fp8_kv_active`, `nf4_qlora_only`. | IDX:35,97,208 |
| DEEP-E001-E0-032 | E-28 / W6 | FA2 on Ampere; FA3/4 Hopper-only; StreamingLLM sink. | `fa2_pinned`, `streamingllm_sink_active`. | IDX:98,209 |
| DEEP-E001-E0-033 | E-29 / W6 | Agent observability via OTel GenAI semantics and SSB-as-trace review. | `otel_span_emitted`. | IDX:99,210 |
| DEEP-E001-E0-034 | E-30 / W6 / P5 | Public-incident safety floor: non-LLM destructive-argv denial and lineage controls. | `destructive_verb_denied`, `mcp_pin_violation`, `auditor_lineage_collision`. | IDX:37,100,211 |
| DEEP-E001-E0-035 | E-31 / W7 / P14 | Coding-model tier strategy; Qwen2.5-Coder lane has no successor/Qwen3 FA2 breakage. | `tier_model_swap`. | IDX:46,103,212 |
| DEEP-E001-E0-036 | E-32 / W7 / P20 | Raise Qwen2.5-Coder-7B native context 25K→32K and calibrate FITM. | `context_lifted_32k`, `fitm_recall_lift_measured`. | IDX:52,104,213 |
| DEEP-E001-E0-037 | E-33 / W7 | Agentic dashboard activity ribbon and skeleton screens. | `activity_ribbon_frame`, `skeleton_screen_shown`. | IDX:105,214 |
| DEEP-E001-E0-038 | E-34 / W7 | Rust AI ecosystem correction: `ort` 2.0.0-rc.12 is production; adopt `rmcp`. | `ort_rc_amended`, `rmcp_adopted`. | IDX:106,215 |
| DEEP-E001-E0-039 | E-35 / W7 | Structured output substrate: XGrammar Tier 2 and CodeAct. | `xgrammar_substrate_loaded`, `codeact_step`. | IDX:107,216 |
| DEEP-E001-E0-040 | E-36 / W8 | Vision document RAG: ColQwen/ColModernVBERT, PaddleOCR-VL, `VisualArtifactCheck`. | `visual_artifact_check`. | IDX:110,217 |
| DEEP-E001-E0-041 | E-37 / W8 / P16 | Data-plane prompt-injection defense: spotlighting, PromptArmor and egress blocking. | `spotlighting_envelope`, `promptarmor_verdict`, `egress_blocked`. | IDX:48,111,218 |
| DEEP-E001-E0-042 | E-38 / W8 | Strategic signals: MoE, RAG-first and EU AI Act posture. | `eu_ai_act_audit`. | IDX:112,219 |
| DEEP-E001-E0-043 | E-39 / W8 / P17 | Token economics: hard spend ceiling and retry budget. | `hard_ceiling_blocked`, `retry_budget_exhausted`. | IDX:49,113,220 |
| DEEP-E001-E0-044 | E-40 / W8 | SCIP+tree-sitter symbol graph and DSL query; correct FocalAD attribution. | `symbol_graph_dsl_query`. | IDX:114,221 |
| DEEP-E001-E0-045 | E-41 / W9 | Four-leg CI/PR review with cross-family reviewer. | `pr_review_dispatched`. | IDX:117,222 |
| DEEP-E001-E0-046 | E-42 / W9 | Pin tree-sitter 0.26.x/ABI 15 and split stable versus experimental grammars. | `tier_b_grammar_failed`. | IDX:118,223 |
| DEEP-E001-E0-047 | E-43 / W9 / P19 | Cross-vendor skill system with signed import/export. | `skill_imported`, `skill_invocation_logged`. | IDX:51,119,224 |
| DEEP-E001-E0-048 | E-44 / W9 | Durable checkpointing: idempotent replay, saga compensation, Continue-As-New. | `idempotency_replay`, `saga_compensated`, `continue_as_new`. | IDX:120,225 |
| DEEP-E001-E0-049 | E-45 / W9 | Cross-language drift prevention and action-type golden lock. | `action_type_golden_drift`. | IDX:121,226 |
| DEEP-E001-E0-050 | E-46 / W10 | Planning decomposition with goal-drift canary, replanning and ledger reset. | `goal_drift_canary_fired`, `replan_phase`, `ledger_reset`. | IDX:124,227 |
| DEEP-E001-E0-051 | E-47 / W10 | LoRA work is research/deferred because of catastrophic forgetting; Unsloth ladder only after gate. | — (`DEFERRED`; ship-gate token on attempt). | IDX:125,228 |
| DEEP-E001-E0-052 | E-48 / W10 | Web-research orchestration with citation verification and contradiction surfacing. | `citation_verified`, `contradiction_surfaced`. | IDX:126,229 |
| DEEP-E001-E0-053 | E-49 / W10 | Coding-agent metrics: STABLE-Pass@1, $/CTS, attribution and verified stop. | `stable_pass_at_1`, `harness_attribution_tuple`, `stop_verified`. | IDX:127,230 |
| DEEP-E001-E0-054 | E-50 / W10 | Opinionated SaaS template with four archetypes and RLS baby steps. | `archetype_selected`, `rls_baby_step_emitted`. | IDX:128,231 |
| DEEP-E001-E0-055 | E-51 / W11 | Edge inference via MLX and llama.cpp `RuntimeAdapter`s. | `mlx_loaded`, `llamacpp_loaded`. | IDX:131,232 |
| DEEP-E001-E0-056 | E-52 / W11 | LSP as verifier, not retriever; pre-AUDITING diagnostics. | `lsp_diagnostic_probed`. | IDX:132,233 |
| DEEP-E001-E0-057 | E-53 / W11 | Narrow Z3/Kani/AutoVerus formal verification; routine Lean hard-avoid. | `z3_consensus`, `kani_proved`, `autoverus_proved`. | IDX:133,234 |
| DEEP-E001-E0-058 | E-54 / W11 | Eval corpus: replace broken 9-gram with 13-gram, per-family shards and eval-awareness. | `ngram13_filtered`, `eval_aware_flag`, `per_family_shard_loaded`. | IDX:134,235 |
| DEEP-E001-E0-059 | E-55 / W11 | BAI is soft RL; defer weight RL and make SFT-on-rollouts the next rung. | `sft_on_rollouts_promoted`. | IDX:53-54,135,236 |
| DEEP-E001-E0-060 | IDX:141-145 | Closure model: roughly half augment shipped B work, roughly one-third extend planned C/D work, and E-11/E-12/E-22/E-31/E-33/E-36/E-38 contain named net-new surfaces. Work is not a single sequential track and must run against its own prerequisites. |
| DEEP-E001-E0-061 | IDX:147-157 | Whole-plan user outcome: per-VRAM serving hardening (AWQ/FP8/FA2/MLX/llama.cpp), genuine RAG reranking, production-SaaS gates, safe memory illusion, mandatory cross-family critique, enforced cost caps, LSP verification, mutation/intent/MR checks, narrow formal verification, and deferred RL with SFT first. |
| DEEP-E001-E0-062 | IDX:163 | Global content-hash rule: use `blake3`; never blake2 or sha2 for non-RFC uses. |
| DEEP-E001-E0-063 | IDX:164 | Production Rust forbids `.unwrap()`/`.expect()` except tests and main entrypoint. |
| DEEP-E001-E0-064 | IDX:165 | After C-13, production must not use `InProcessToolDispatcher`. |
| DEEP-E001-E0-065 | IDX:166 | FA2 is mandatory; SDPA has an approximately 4K ceiling. |
| DEEP-E001-E0-066 | IDX:167 | Generation must use `model.generate()` plus `LogitsProcessor`; manual token loops are forbidden. |
| DEEP-E001-E0-067 | IDX:168 | Before adding a package, verify name, version, and last-published date (anti-slopsquatting). |
| DEEP-E001-E0-068 | IDX:169 | Apply anti-stale-training rules; E-34 amends the stale `ort` rule. |
| DEEP-E001-E0-069 | IDX:170 | Every report-flagged risk must be surfaced under its E-N Open Questions (anti-sycophancy). |
| DEEP-E001-E0-070 | IDX:171 | No new external dependency without explicit user confirmation; the index names E-22/E-26/E-27/E-33/E-37/E-43/E-51 as approval sites. |
| DEEP-E001-E0-071 | IDX:172 | Tier-C isolation is mandatory for CLI subprocesses; E-15 supplies it and E-41 consumes it. |
| DEEP-E001-E0-072 | IDX:173 | Sidecar boot requires the `HARDWARE-HEAVY` flag and is remote-only. |
| DEEP-E001-E0-073 | IDX:174 | Only one change runs at a time, and the auditor is the sole test runner. |
| DEEP-E001-E0-074 | IDX:177-180,238 | Current schema baseline is 47 at sweep generation; each shipping E-N bumps only its declared scope and final schema version depends on ship order. |
| DEEP-E001-E0-075 | IDX:244-248 | Cross-plan dependency surface is broad and explicit: named B phases are augmented and named C/D phases extended; per-E exact relations live in each original’s Existing State block. |
| DEEP-E001-E0-076 | IDX:251-254 | Session log starts empty. Each shipped item must record the FSM chain `context-compiler → scope-planner-elai → parallel explorers+synthesizer → spec-grounder → prompt-engineer → impact-mapper → EXECUTING → auditor → fixer(if BLOCK) → wiring-contract-validator → memory-writer → COMMITTING`. |
| DEEP-E001-E0-077 | IDX:257-268 | Default starting order is E-17, E-2, E-30, E-27, E-15; later work follows wave dependencies or a current pain point, and every E-N prerequisite must be respected. |

### E-2_local_llm_speed_tricks.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E2-001 | E-2:1-6 | **PLANNED, 2 sessions/6–10h.** Extend B-14 streaming and B-37 sidecar without leaving `model.generate()` or breaking DoLa/ETC/steering. Coordinates `CommandWindow.svelte` edits with E-4. |
| DEEP-E001-E2-002 | E-2:8-14 | Diagnosed latency has three surfaces: 50–200ms bursty SSE chunks, per-character DOM layout thrash, and no submit→TTFT cue. Targets <200ms “snappy” TTFT; explicitly rejects vLLM/SGLang/TensorRT-LLM migration, Skeleton-of-Thought, MoE swaps, and 12GB-tier EAGLE-3. |
| DEEP-E001-E2-003 | E-2:18-24 | Existing path is B-14 `stream_chat`→Tauri event→CommandWindow/Palette plus B-37 `RuntimeAdapter`; prompt lookup is transparent to Rust. DoLa is harmful/disabled by default on 7B, while ETC/steering remain live. This plan calls its dashboard+sidecar delta net-new. |
| DEEP-E001-E2-004 | E-2:41 | `FramePacer` exposes `enqueue(chunk)` and RAF-driven drain plus instant `flush()` at stream end. Auto-detected defaults are 60 cps for code and 100 cps for prose; settings are `streaming.frame_pace.enabled` (stated default `true`) and `streaming.frame_pace.cps` (40–200). |
| DEEP-E001-E2-005 | E-2:43 | `SkeletonBubble.svelte` appears immediately on submit, shimmers, preallocates configurable height (240px default), and disappears on first token/`stream_complete` as specified by wiring/acceptance. |
| DEEP-E001-E2-006 | E-2:43 | `CursorSentinel.svelte` is a CSS-only blinking caret visible while `streaming=true`; no JavaScript interval; removed on `stream_complete`. |
| DEEP-E001-E2-007 | E-2:45 | Sidecar prompt lookup adds `prompt_lookup_num_tokens`, default value 10 when enabled, request field `prompt_lookup`, env `ELAI_PROMPT_LOOKUP_NUM_TOKENS`, and structured `prompt_lookup_acceptance`. Existing baselines remain default-off; promote only after pooled acceptance ≥30% for one week. |
| DEEP-E001-E2-008 | E-2:47 | Prefix-KV reuse precomputes immutable system-prefix KV, keys it by blake3, clones/slices into a fresh HF `StaticCache`, invalidates on prompt change, and is gated by `ELAI_PREFIX_KV_REUSE=1`; default-off until three consecutive deterministic passes. |
| DEEP-E001-E2-009 | E-2:49 | `StreamingTimingRecorded` payload is `{request_id,ttft_ms,mean_inter_chunk_ms,p50_inter_chunk_ms,p99_inter_chunk_ms,total_ms,tokens_emitted}` per stream end. It flows through `/notify` and renders a MonitorWindow histogram; frame-pacing default-on is blocked pending the data. |
| DEEP-E001-E2-010 | E-2:51 | Bubble cancel control reuses B-14 `cancel_stream`; its presence is a perceived-latency affordance, not a new cancellation backend. |
| DEEP-E001-E2-011 | E-2:53 | A configurable 200ms submit→stream-start threshold shows “elai is reading the project...” below the skeleton and removes it on first token. |
| DEEP-E001-E2-012 | E-2:55 | Partial-Markdown state machine soft-commits an empty code-block frame when an opening fence arrives, tracks open/close state, and buffers incomplete markup to avoid flicker. |
| DEEP-E001-E2-013 | E-2:57 | `test_prefix_kv_determinism.py` runs reuse on/off three times at greedy temperature 0 and requires token/byte-equivalent output; mismatch fails and keeps reuse off. |
| DEEP-E001-E2-014 | E-2:59 | Frame-pacing A/B is randomized per session between default-off/default-on, logs a `snappy/slow` self-rating as `UxPacingVoteRecorded`, and uses N=5 sessions as the directional gate. |
| DEEP-E001-E2-015 | E-2:61 | `E-2-decisions.md` records frame-pacing threshold (p50 chunk gap >100ms), prompt lookup starting value 10, three-pass KV gate, and explicit deferral/rejection of EAGLE-3, vLLM, Skeleton-of-Thought and MoE. |
| DEEP-E001-E2-016 | E-2:63-88 | Produced surfaces are `FramePacer`, Skeleton/Cursor components, `StreamingTimingRecorded`, `UxPacingVoteRecorded`, prompt-lookup/prefix-KV paths and determinism test. They consume CommandWindow, `stream_chat`, SettingsWindow and are reachable from dashboard SSE and CLI REPL sidecar calls. Smoke expects skeleton→cursor, ~60cps pacing, cancel visibility, timing row and vote row. |
| DEEP-E001-E2-017 | E-2:79-84 | Negative contract: no pacing TODO/no-pacing literals, no vLLM/SGLang imports, and no new `model.generate` call bypassing the existing LogitsProcessor chain. |
| DEEP-E001-E2-018 | E-2:90-116 | Deterministic checks require imports/files, both sidecar env hooks and `StaticCache`, both SSB variants, passing KV byte-equivalence, and queryable p50/TTFT rows in SQLite after a 200-token stream. |
| DEEP-E001-E2-019 | E-2:118-130 | Done gates: pacing within ±10%; no skeleton flicker; cursor lifetime correct; prompt lookup lossless at temp 0; prefix reuse passes 3×; every stream end has all timing fields; cancel path works; LogitsProcessors stay attached; no new dependency; production unwrap count does not increase. |
| DEEP-E001-E2-020 | E-2:131-139 | Test inventory: three burst-pattern pacing test; skeleton lifecycle; KV equivalence; five-prompt prompt-lookup equivalence; timing/vote SSB contracts; bursty mock-SSE CommandWindow integration. |
| DEEP-E001-E2-021 | E-2:141-153 | File lifecycle is explicit: create five implementation/tests plus decision doc; modify CommandWindow, Tauri stream command, SettingsWindow, sidecar server and SSB enum. No provider-adapter modification is planned. |
| DEEP-E001-E2-022 | E-2:173-180 | Failure/risk gates: no current 3090 p50 data; prompt lookup workload acceptance unknown; StaticCache+FA2+compile compatibility uncertain; KIVI excluded; sidecar tests are remote/HARDWARE-HEAVY; do not claim perceived-speed evidence below N=5. |
| DEEP-E001-E2-023 | E-2:182-184 | Dependency/next: E-3 reuses pacing/skeleton; E-4 must land after/coordinate on CommandWindow; KIVI and EAGLE-3 wait for measured data and higher hardware. |

### E-6_multi_agent_failure_modes.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E6-001 | E-6:1-6 | **PLANNED, 2–3 sessions/10–12h.** Extends B-41/D-13/anti-spiral/MistakeBook; can land a minimal cloud-Sonnet Adversary before D-13 or extend D-13 afterward. |
| DEEP-E001-E6-002 | E-6:8-20 | Defends the eight roles against 14 MAST modes in three categories, same-model blindness, echo/conformity and persona drift; measures whether eight roles beat an equal-token single agent. |
| DEEP-E001-E6-003 | E-6:14-20 | Existing-state deltas: tag B-41 BLOCK/PASS verdicts; mandate cross-family auditing; count semantic-step FM-1.3 repetitions; replace free-text inter-agent payloads; provenance-gate MistakeBook; add iso-budget attribution; re-anchor role prompts. |
| DEEP-E001-E6-004 | E-6:43 | `TypedAgentEnvelope` has variants `TaskHandoff`, `FindingReport`, `GroundingCitation`, `ConsensusVote`, `AdversaryVerdict`, `MistakeBookEntry`; fields `sender_role`, `receiver_role`, optional `mast_tag`, blake3 `[u8;32] content_hash`, `ts_ms`; wire format `envelope_v1`. Debug refuses raw payloads; release logs/stat-counts them. |
| DEEP-E001-E6-005 | E-6:45 | `MastMode` models all 14 modes (`FM-1.1 DisobeySpec` through `FM-3.3 IncorrectVerification`), supports serde/`as_sql_str`, and BLOCK verdicts require a MAST tag. |
| DEEP-E001-E6-006 | E-6:47 | `FindingReason::* -> MastMode` is deterministic for BLOCK/CASCADE and emits `AuditMastTagged {verdict_id,mast_mode,confidence_band}`. |
| DEEP-E001-E6-007 | E-6:49 | `route_cross_family` requires adversary family ≠ executor family. With one local family it may use cloud Sonnet only under `ELAI_ALLOW_CLOUD_ADVERSARY=1`; otherwise it refuses auditing and emits `CrossFamilyUnavailable`. `adversary_verdict_required` is true for code/architecture/security claims. |
| DEEP-E001-E6-008 | E-6:51 | `run_ablation` compares the capped eight-role pipeline against one Executor with appended “Wait” under identical thinking-token budget B, records per-role tokens/pass rate/MAST counts, emits `IsoBudgetMeasured`, and is exposed as `elai bench iso-budget --task … --budget …`. |
| DEEP-E001-E6-009 | E-6:53 | `PersonaAnchor::tick` reinjects role prompt, grounding instruction and last successful finding every configurable N turns (default 20), or early when fastembed cosine <0.55; it emits `PersonaReanchored`. Threshold is unconfirmed/calibration-required. |
| DEEP-E001-E6-010 | E-6:55 | MistakeBook provenance is `{origin_role,origin_run_id,originating_mast_mode,content_hash,timestamp_ms}`. Uncorroborated Auditor entries become `PROVISIONAL`, cannot influence the next context pack, and wait for a later cross-family pass. |
| DEEP-E001-E6-011 | E-6:57 | TDD contract checks the ten common `AgentConnection` write sites use envelopes and reject raw strings. |
| DEEP-E001-E6-012 | E-6:59 | FSM modifies `AUDITING -> COMPLETE`: it is blocked unless the verdict has `MastMode::None` **or** an attached `AdversaryVerdict {cross_family_confirmed:true}` (see conflict ledger against stronger acceptance wording). |
| DEEP-E001-E6-013 | E-6:61 | `MastFailurePanel.svelte` renders all 14 modes in three columns live from `AuditMastTagged`; `get_mast_breakdown` returns session-grouped `Vec<MastModeCount>`. |
| DEEP-E001-E6-014 | E-6:63 | `elai mast report --session <id>` deterministically summarizes 14-mode counts per session and per role from SSB. |
| DEEP-E001-E6-015 | E-6:65-98 | Wiring exports envelope/mode/adversary/ablation/persona/provenance APIs and four events. Entry points are Auditing, new Adversary substate, CLI and Tauri. Smoke expects `IsoBudgetMeasured` within 60s and identical CLI/dashboard two-row ablation data. |
| DEEP-E001-E6-016 | E-6:90-93 | Stub bans: no fixed false `cross_family_confirmed`, no hand-set `MastMode::None` for production BLOCKs, and no production default envelope. |
| DEEP-E001-E6-017 | E-6:100-131 | Deterministic proof checks raw-string absence, MAST coverage of BLOCK paths, cross-family transition guard, JSON ablation keys, N=20 re-anchor, provisional-context exclusion and blake3-only envelope hashing. |
| DEEP-E001-E6-018 | E-6:133-145 | Done gates include envelope-only debug payloads/release counter, MAST tag on every BLOCK with ≥12/14 modes exercised, mandatory confirmed adversary for required phases, ≥1σ pipeline lift or `insufficient_lift=true`, logged re-anchor cosine, provisional exclusion, stable CLI text, live dashboard, blake3/no unwrap. |
| DEEP-E001-E6-019 | E-6:146-155 | Test matrix covers envelope round trips, 14-mode serde, same-family rejection, budget ±5%, turn/cosine anchors, provenance exclusion, 3-run stable CLI output, and dashboard rendering of all modes. |
| DEEP-E001-E6-020 | E-6:157-172 | File lifecycle creates seven modules/surfaces and modifies SSB/FSM/auditor/gateway/pool/MistakeBook/Tauri. Four SSB variants—not the index’s three generic names—are declared here. |
| DEEP-E001-E6-021 | E-6:185-191 | Risks/fallbacks: cosine 0.55 needs 50+ sessions; cloud cross-family costs/latency and explicit opt-in; deterministic MAST mapping can mis-tag but exposes confidence/override; provider tokenizers skew iso-budget; typed envelopes do not eliminate agents reading one another. |
| DEEP-E001-E6-022 | E-6:193-198 | E-6 feeds E-9 attribution, enables E-10’s empirical role-collapse decision, subsumes/merges D-13 according to landing order, and leaves session-length caps for future evidence. |

### E-7_darwinian_self_improvement.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E7-001 | E-7:1-6 | **PLANNED, 3 sessions/~12h.** Refits D-1..D-9 and moves only D-28’s evaluation fixtures into Wave 1; co-lands or amends D work depending on ship order. |
| DEEP-E001-E7-002 | E-7:10-23 | Deltas replace fitness-proportional parents, validate/augment PRM, diversify verifiers, rebuild drift tripwires, replace vanilla TS, strengthen contamination/transfer checks, sign lineage, extend L3 locks and undefer 50 traps. |
| DEEP-E001-E7-003 | E-7:45 | `BehaviourDescriptor=(latency_tier,token_class,failure_mode)` where latency is Sub10s/Sub60s/Over60s, tokens Lt500/Lt2k/Lt8k/Over8k and failure is optional E-6 `MastMode`; `MapElitesArchive` keeps one best elite per cell and emits `ArchiveCellElevated`. |
| DEEP-E001-E7-004 | E-7:47 | `two_gate_promote` permits promotion iff held-out margin ≥ τ **and** reachable policy capacity ≤ L3-locked `K_m`; rejection reasons are `below_margin`/`capacity_exceeded` in `TwoGateBlocked`. |
| DEEP-E001-E7-005 | E-7:49 | BAI becomes `DiscountedTs` plus fixed-budget `SuccessiveRejects`; default per-session discount is `exp(-1/0.3)≈0.0357`, arms retain a breakpoint window, and configuration tracks non-stationarity. |
| DEEP-E001-E7-006 | E-7:15-16,51 | D-2 must pass PRMBench Step-Consistency and Confidence-Invariance ≥0.7. EST applies N=5 semantic-equivalent rewrites (whitespace, rename, cloud-Sonnet paraphrase), requires PPM score change ≤ε=0.05, emits `EstInvarianceMeasured`, and blocks variance violations before consensus. |
| DEEP-E001-E7-007 | E-7:16 | D-3’s second verifier must be from an independent base-model family; verifier-diversity has a lower bound and its set is L3-frozen. |
| DEEP-E001-E7-008 | E-7:17,53 | Tripwire retains KL/Δρ/IQR/DDM, adds Page-Hinkley/ADWIN/EST and auto-reverts on ≥2 concurrent trips. In-loop perplexity is removed; optional perplexity is feature-gated to a frozen, loop-inaccessible external LM. |
| DEEP-E001-E7-009 | E-7:19,55 | Held-out contamination pipeline applies 9-gram, then fastembed cosine ≥0.85 block, temporal cutoff, and per-task random 16-character GUID canary; recognition retires the task and emits `ContamFilterRejected`. |
| DEEP-E001-E7-010 | E-7:20,57 | Transfer regression rejects when **any** non-target family breaches -20% relative or -5% absolute; adversarially select the weakest three families from the last three generations and use Successive Rejects over family score. Emits `TransferRegressionBlocked`. |
| DEEP-E001-E7-011 | E-7:23,59 | Fifty hand-crafted trap TOMLs cover green-block loops, Lego flips, marker removal, fake logs, verifier capture and metric overflow. Trap weight is 10× normal; `TrapTripped` is FATAL and has no promotion path. |
| DEEP-E001-E7-012 | E-7:21,61 | Every archive link stores `{self_blake3,parent_blake3,parent_lock_list_blake3,verifier_set_blake3,ed25519_sig}` and is signed by an Auditor in a separate Tier-C process. `elai archive verify` walks signatures and emits `ArchiveSigVerified`/`ArchiveSigBroken`. |
| DEEP-E001-E7-013 | E-7:22,63 | L3 lock list adds SSB/process-supervision logs, trusted-output sandbox configuration, verifier-set definitions and external perplexity model path; lock-list modification must be blocked absent auditor sign-off. |
| DEEP-E001-E7-014 | E-7:65-67 | MutationArchivePanel becomes a latency×token×failure heatmap with Two-Gate/EST/trap evidence. CLI adds `archive map-elites stats` and extends `archive verify`. |
| DEEP-E001-E7-015 | E-7:69-110 | Wiring exports archive/gate/BAI/EST/tripwire/filter/transfer/trap/crypto APIs and eight SSB variants. Auditing runs EST, Promoting runs Two-Gate, and self-mod proposals resolve to cell elevation/rejection. Five-generation smoke requires cell table, decisions, zero trap promotions and ≥1 verified signature/generation. |
| DEEP-E001-E7-016 | E-7:101-105 | Stub bans remove unconditional Two-Gate success, production default EST, in-loop perplexity, and old `fitness_proportional|softmax_parent` selection. |
| DEEP-E001-E7-017 | E-7:112-146 | Deterministic checks enforce blake3, gated perplexity, verified `ed25519-dalek`, queryable cells, capacity rejection, caught fake-log trap, ≥50 fixtures and a real EST variance flag. |
| DEEP-E001-E7-018 | E-7:148-160 | Done gates: ≥6/12 cells after five generations; both Two-Gate rejection paths; configurable discounted TS+SR; gameable EST block; Page-Hinkley/ADWIN; contamination/canaries; weakest-three transfer threshold; 50 fatal traps; tamper-detecting signed chain; full L3 locks; no unwrap/blake2/sha2/async-trait. |
| DEEP-E001-E7-019 | E-7:162-173 | Tests cover cell placement, capacity reason, synthetic regret, EST, perplexity-off, paraphrase contamination, fatal traps, tampered chain, lock edits and SSB serialization. |
| DEEP-E001-E7-020 | E-7:175-192 | File lifecycle creates archive/gate/BAI/EST/filter/canary/trap/crypto/lock modules and 50 fixtures; modifies tripwire, SSB, FSM, Cargo, CLI and dashboard. New dependency `ed25519-dalek` requires package verification. |
| DEEP-E001-E7-021 | E-7:214-218 | Unconfirmed parameters: descriptor axes must keep Shannon cell entropy >1.0 nats after 20 generations; cloud paraphrase calibration may differ; default `K_m=50` is a deployment hypothesis. |
| DEEP-E001-E7-022 | E-7:219-222 | Finite traps can miss exploits; -20%/-5% is a design hypothesis; crate version is unspecified and must be verified. |
| DEEP-E001-E7-023 | E-7:220,228 | **DEFERRED:** D-28 adversarial retraining remains blocked until D-5 has ≥20 promotion cycles/user authority; E-7 ships evaluation traps only. |
| DEEP-E001-E7-024 | E-7:224-229 | E-7 feeds E-9, supersedes/amends D-1..D-9, and strengthens D-8 lineage with second-party signature+lock hash while acknowledging that lineage is not goodness. |

### E-19_reasoning_quality.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E19-001 | E-19:1-6 | **PLANNED, 2–3 sessions/10–14h.** Extends shipped B-39/B-41, planned B-27, and uses auditor PRM-lite until planned D-2 PPM supersedes it. |
| DEEP-E001-E19-002 | E-19:10-20 | Four danger zones are unconditional/confirmatory 7B L2, intrinsic-only L4 value, breadth/depth pruning, and same-pipeline L6 verification. Six-layer prevalence and paper measurements are rationale, not implementation results. |
| DEEP-E001-E19-003 | E-19:24-31 | Reuses B-39 N=4 sampling, B-41 auditor, B-27 hybrid path, MistakeBook/micro-reflection and InterrogationTree; E-19 inserts a pre-L2 gate and composes local diversity with D-13 cross-family review. |
| DEEP-E001-E19-004 | E-19:47,51 | `ConfidenceGate` computes self-certainty/logit margin from N=4 L1 candidates. Route table: score ≥0.80 skips L2+L3; `[0.40,0.80)` runs L2; `<0.40` goes directly to L3. Emits score/skip/direct-route rows. |
| DEEP-E001-E19-005 | E-19:49 | L2 prompt becomes `ReSampleWithPrior`: MistakeBook forbidden-diff prior, never “reflect on your mistake”; hard `L2_MAX_TOKENS=240`, with `L2TokenCapHit` when bound. |
| DEEP-E001-E19-006 | E-19:53 | L4 `AuditorVerifierScore` is deterministic pass/fail+`rule_applied` and combines with intrinsic signals at weights auditor=.6, mean_logprob=.2, cosine=.2; each node emits `L4PrmLiteScore`; D-2 later swaps in PPM. |
| DEEP-E001-E19-007 | E-19:55 | LToT expansion keeps `N_MAINLINE=3` high-utility branches plus `N_LATERAL=2` low-utility syntactic-pass branches for one additional ply, emitting `L4LateralKept`. |
| DEEP-E001-E19-008 | E-19:57 | Typed-CoT lite emits XGrammar-constrained JSON `{rule_applied,types_in,types_out}` triples to wiring-contract validation and logs `L2TypedCotEmitted`; cited math lift must be described only as rationale for wiring quality. |
| DEEP-E001-E19-009 | E-19:59 | L6 promotion requires agreement between fresh-RNG-seed L1 and a rotated rules.db L3 MCQ set; pass/disagree have separate SSB events. |
| DEEP-E001-E19-010 | E-19:61 | `DifficultyEstimator` uses FailureTaxonomy×historical retries: easy `(depth=2,N=2)`, medium `(3,3)`, hard `(3,N=5+lateral)`; logs `L4TtcBudgetChosen`. |
| DEEP-E001-E19-011 | E-19:63 | Failure classes `implicit-statistical-learning` and `pattern-with-exceptions` are no-CoT: L1→L3, never L2; emits `L2SkippedNoCotClass`. |
| DEEP-E001-E19-012 | E-19:65 | Calibration corpus is 200 production traces at `.elai_cc/eval/reasoning/`; tune 0.80 by ROC Youden-J and persist `[reasoning] l2_threshold` plus `calibrated_at`. |
| DEEP-E001-E19-013 | E-19:67-110 | Wiring exports nine reasoning APIs and eleven events; consumes chat-reflect, micro-reflection, MistakeBook, InterrogationTree, auditor and taxonomy; reachable from task/role reasoning. Trivial-task smoke expects L2 skip, easy budget and calibration artifact/UI fields. |
| DEEP-E001-E19-014 | E-19:100-105 | Negative contract forbids unconditional L2, intrinsic-only L4 scoring, L6 promotion without both diversity checks, old reflection wording and production unwrap. |
| DEEP-E001-E19-015 | E-19:112-155 | Deterministic proof searches all gates, token cap, direct route, PRM-lite, LToT, typed CoT, L6 diversity, TTC and calibration, then verifies trivial-task SSB output. |
| DEEP-E001-E19-016 | E-19:157-171 | Done gates preserve exact thresholds/weights/branch counts/schema/routes, require D-2 upgrade documentation and prevent “always reflect” regression. |
| DEEP-E001-E19-017 | E-19:173-183 | Nine tests cover score boundary, cap, direct-route absence of L2, weighted node score, lateral lifetime, JSON schema, seed disagreement, difficulty mapping and event round trips. |
| DEEP-E001-E19-018 | E-19:185-200 | File lifecycle creates self-certainty/typed-CoT/TTC/PRM-lite/LToT/L6 modules, two eval files and decision doc; modifies actual `crates/reasoning/src/chat_reflect.rs`, micro-reflection, task runner, SSB and ReasoningDrawer. |
| DEEP-E001-E19-019 | E-19:218-224 | **CONDITIONAL prior art only:** SE-Agent is per-task trajectory search, not DGM/self-mod; its reported scores are rationale and README-only 80% must not be cited. Population is discarded after each task. |
| DEEP-E001-E19-020 | E-19:226-232 | Missing operator is segment recombination across scored branches, but importing SE-Agent’s LLM-judge reward is forbidden. It is cloud/hard-only, never local default, and waits for off-agent fitness/GAP-1 after main E-19. |
| DEEP-E001-E19-021 | E-19:234-237 | Conditional `recombine_segments` makes ≤2 candidates from ≥2 parents, reuses PRM-lite/off-agent scoring, keeps only strict parent-beating results, emits `l4_recombination_applied`, and must beat BestOfN on ≥30 hard tasks or be dropped. |
| DEEP-E001-E19-022 | E-19:239-247 | Risks/fallbacks: calibrate 0.80 or retire L2 if wrong→right rate stays negligible; PRM-lite is syntactic only; lateral count may fall to 1; L6 diversity config defaults on HEAVY/off FOCUSED if latency fails; NF4 accuracy remains unaddressed; difficulty heuristic may gain historical cosine. |
| DEEP-E001-E19-023 | E-19:249-256 | Next gates: D-2 replaces PRM-lite; D-13 adds cross-family axis; D-3 aggregates scores/verdicts; retire L2 if flip rate <2%; autonomous full-task waits for ≥90% L6 diversity pass on holdout; recombination remains conditional. |

### E-36_document_vision_rag.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E36-001 | E-36:1-6 | **PLANNED, 3 sessions/10–14h.** Adds an opt-in vision side to B-42 Cosmos/B-35 ArtifactCheck/B-37 runtimes and requires E-17 RRF; C-7 SSB substrate must land first. |
| DEEP-E001-E36-002 | E-36:8-15 | Text RAG remains primary for grep-able content. Vision retrieval is additive only for diagrams/scans/screens/slides; VLM captions are retrieval evidence, never authoritative grounding. |
| DEEP-E001-E36-003 | E-36:18-23 | Extends—not replaces—Cosmos. Adds `doc_parser` backends and `vision_verifier`; expands E-17 fusion from three to four lists; adds five vision SSB types; schema pin is deliberately TBD at execution. |
| DEEP-E001-E36-004 | E-36:43 | `elai doctor knowledge-models --vision` probes presence, tokenizer hash and size. Default `[knowledge.vision_retriever] model="colmodernvbert"`; alternate ColQwen2.5-v0.2; models live under `models_dir_path()`. |
| DEEP-E001-E36-005 | E-36:45 | `VisionChunk={source_blake3,page_idx,multi_vec:Vec<[f16;128]>,pooled,vlm_caption,parser_confidence}`; pool ~1,030 vectors to ~100 (≤128 contract), store in project SQLite `vision_chunks` keyed by source blake3. The cited pooling algorithm must be verified first. |
| DEEP-E001-E36-006 | E-36:47 | `PdfStructuredChunker` uses PaddleOCR-VL via `doc_parser` and returns markdown, HTML tables, figure crops, page confidences and `fallback_recommended`. Markdown goes to text chunking, figures to image chunking; any page confidence <0.85 forces image indexing. |
| DEEP-E001-E36-007 | E-36:49 | Cosmos gains `Diagram`, `ArchImage`, `Screenshot`, `ScannedDoc` schemas. Diagram stale-checks source **or** render; the other three stale-check source-file blake3. `ParserBackend` includes Paddle/MinerU/Dolphin plus three cloud parsers. |
| DEEP-E001-E36-008 | E-36:51 | Visual-spatial claims dispatch to `GroundingMethod::VisualArtifactCheck`: top-k image retrieval then Qwen2.5-VL structured `{verdict:Confirm|Refute|Indeterminate,evidence_pages,reliability:VlmCaption|OcrParse|DiagramSource}`. Consumers must downweight VLM-caption reliability. |
| DEEP-E001-E36-009 | E-36:53 | `RrfFusion4` uses default weights `{bm25:1.0,code_vec:1.0,text_vec:0.8,vision_vec:0.6}`, emits `rrf_vision_component_weight`, and omits the vision list when disabled. |
| DEEP-E001-E36-010 | E-36:55 | `[knowledge.vision_rag] enabled=false` is a hard shipping default. Disabled ingest stores caption+hash for visually rich inputs but skips multi-vectors; enabled stores the index. `VisionRagPolicyDecision` records enabled/budget/actual storage. |
| DEEP-E001-E36-011 | E-36:57 | Cloud parser fallback threshold defaults .7 and is eligible only outside `--mode local`; local refuses and logs `DocParserCloudFallbackBlockedByMode`; cloud cost consumes task spend ceiling. |
| DEEP-E001-E36-012 | E-36:59 | Separate 50-query gold set evaluates ColModernVBERT vs ColQwen at nDCG@5/Recall@5/MRR. At least one backend must reach nDCG@5≥.50 before any policy default-enables vision. |
| DEEP-E001-E36-013 | E-36:61 | CosmosDrawer extension or `VisionRAGDrawer` shows index state, storage, parse confidence and latest verdict/reliability; Tauri exposes `get_vision_artifact_status` and `get_vision_artifact_check_recent`. |
| DEEP-E001-E36-014 | E-36:63-107 | Wiring exports chunkers/schemas/artifact types/parser enum/verdict/reliability/RRF4, doctor probe and five events: `VisionChunkEmitted`, `VisionFallbackTriggered`, `VisualArtifactCheckVerdict`, `VisionRagPolicyDecision`, `DocParserCloudFallbackBlockedByMode`. Entry points are CLI, query pipeline, Tauri and verification dispatch. |
| DEEP-E001-E36-015 | E-36:94-103 | Negative contract: no unwrap, non-blake3 file hash, caption-only authority, full 1,030-vector storage, auto-enable, MarkItDown for image PDFs, Nougat, or local-mode cloud fallback. |
| DEEP-E001-E36-016 | E-36:104-107 | Smoke verifies an auth-diagram redirect claim and requires chunk/check/RRF rows, blake3-keyed database row, nonempty pages on determinate verdicts and a four-stage query trace. |
| DEEP-E001-E36-017 | E-36:109-155 | Deterministic checks cover hashes, enum+dispatch, four artifact types, parser registration, pooling cap, default-off config, five SSB variants, RRF4, two eval files and staged-model doctor output. |
| DEEP-E001-E36-018 | E-36:157-169 | Done gates require staged hash/size, working Paddle confidence, ≤25KB/page pooled storage, correct stale policy, routed reliability-aware verdict, opt-in RRF participation, mode/threshold fallback, .50 corpus nDCG and blake3-only code. |
| DEEP-E001-E36-019 | E-36:171-179 | Tests cover visual-vs-file dispatch, pooling recall within ±2 points, low/high parser confidence, LOW/MEDIUM/HIGH reliability mapping, three-run stable four-list ordering, diagram-render staleness and corpus nDCG gate. |
| DEEP-E001-E36-020 | E-36:181-202 | File lifecycle creates two chunkers, verifier, optional/new drawer, eval corpus/driver, five contracts and decision doc; modifies Cosmos, pipeline, verification, runtime, SSB, doctor, Tauri and config docs. |
| DEEP-E001-E36-021 | E-36:221-224 | Pooling citation is unverified; if absent, use measured max/mean pooling. Target budget is 25KB/page and suggested project default 500MB; emit over-budget evidence. |
| DEEP-E001-E36-022 | E-36:225 | Local VRAM cannot hold coder+VLM concurrently; captioning is ingest-time batch or remote billable sidecar that must be stopped after use. |
| DEEP-E001-E36-023 | E-36:226-229 | Do not default-enable without local-corpus .50 nDCG; all downstream gates must inspect reliability; cloud multimodal reranking is deferred; C-7 blocks entry if not shipped. |
| DEEP-E001-E36-024 | E-36:231-233 | Next: compose B-35+E-17 for text/vision retrieval; defer cloud reranking, reasoning-tier ColBERT and vision-chunk PageRank; coordinate cloud parse spend with E-39. |

### E-38_ai_roadmap_2026_signals.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E38-001 | E-38:1-6 | **PLANNED, strategic/lightweight, 1–2 sessions/6–8h.** B-37 is shipped hedge; D-1..D-7 are planned compliance substrate; no new FSM state. |
| DEEP-E001-E38-002 | E-38:10-12 | MoE is treated as the frontier trend; dense Qwen remains valid at 7–12GB but no adapter may encode dense-only assumptions. Future MoE-on-Ampere expert offload is documented, not implemented. |
| DEEP-E001-E38-003 | E-38:14 | Ampere stays on FA2 while FA3/4 target newer hardware; `ELAI_FORCE_SDPA=1` remains insurance and code must not hard-code FA2. |
| DEEP-E001-E38-004 | E-38:16 | RAG/mistake_book/Cosmos remain load-bearing through 2027; advertised huge context must not replace layered retrieval without effective-context evidence. |
| DEEP-E001-E38-005 | E-38:18 | MCP is the selected protocol; bridge must watch stateless transport, Tasks, Registry and SAML/OIDC/PKCE roadmap changes and compose with E-24. |
| DEEP-E001-E38-006 | E-38:20-24 | Regulatory hedge is deterministic rules/FSM/evidence/replay. EU high-risk timing is stated postponed; compliance gate targets Articles 12 record-keeping and 13 transparency at `PLANNING→TESTING_FIRST`. |
| DEEP-E001-E38-007 | E-38:54 | `docs/strategy/ELAI-2026-POSTURE.md` is a ~350-line cited spec covering five signals, seven risky bets, labs, hardware, regulation and architectural insurance; no ungrounded roadmap claim. |
| DEEP-E001-E38-008 | E-38:56 | `pre_gate_planning_to_testing_first` asserts `ClockProviderIntact`, `DeterministicSeed`, `TemplateHashChain`, `AuditTrailLive`; failure blocks transition and logs assertion+remediation, success logs verification. |
| DEEP-E001-E38-009 | E-38:58 | CI contract forbids hard FA2 imports/assertions outside try/except fallback in Python inference code and preserves SDPA fallback. |
| DEEP-E001-E38-010 | E-38:60 | Any backend declaring ≥100K tokens must provide measured `mrcr_v2_score≥0.50` or literal `vendor-claim-only`; the latter emits `MrcrUnverified` at init. Canonical MRCR citation must be looked up, not fabricated. |
| DEEP-E001-E38-011 | E-38:62 | AST contract keeps `RuntimeAdapter` architecture-blind (no dense/MoE-named methods) and adds an E-38 rationale comment. |
| DEEP-E001-E38-012 | E-38:64 | `MoE-on-Ampere-tier.md` documents expert CPU/disk offload for named future models/hardware; citations must be verified. It is preparation only. |
| DEEP-E001-E38-013 | E-38:66 | MCP bridge TODO/watch is runtime no-op except `McpTransportVersionLogged {current,target}` at initialization. |
| DEEP-E001-E38-014 | E-38:68 | Three anti-pattern contracts forbid non-MCP/Anthropic bespoke agent protocols, screenshot+click agent loops, and FSM transitions without an SSB row. |
| DEEP-E001-E38-015 | E-38:70 | CLAUDE hard rule points to the posture doc; 2026 lab/timeline claims must cite it or fresh WebFetch, never training memory. |
| DEEP-E001-E38-016 | E-38:72 | Session protocol checks posture-doc mtime and warns/logs `PostureDocStale` after 90 days; reminder, not runtime hook. |
| DEEP-E001-E38-017 | E-38:74-109 | Wiring exports compliance verdict/assertion and five SSB types (`EuAiActComplianceBlocked`, `Verified`, `McpTransportVersionLogged`, `MrcrUnverified`, `PostureDocStale`); consumes runtime, ClockProvider, template archive, SSB and MCP. Synthetic missing-clock smoke must leave FSM in PLANNING with doc-linked error. |
| DEEP-E001-E38-018 | E-38:96-105 | Negative contract repeats no unwrap, hard FA2, architecture-named adapter APIs, ≥100K unmeasured context, bespoke protocol, screenshot loop, unjournaled FSM transition or uncited posture claim. No frontend surface. |
| DEEP-E001-E38-019 | E-38:111-153 | Deterministic checks require both strategy docs, gate+FSM call, architecture blindness, context/MRCR review, all five events, three guard tests, liveness test and CLAUDE cross-reference. |
| DEEP-E001-E38-020 | E-38:155-167 | Done gates require cited docs, all four pre-gate assertions/hints, six classes of contract behavior, bridge version log, hard-rule/stale reminder and all five SSB variants. |
| DEEP-E001-E38-021 | E-38:169-179 | Nine tests cover four failures+pass, FA grep, four MRCR fixtures, adapter AST, protocol/screenshot guards, five transition rows, MCP log and stale mtime. |
| DEEP-E001-E38-022 | E-38:181-202 | File lifecycle creates two docs, canonical `orchestrator/src/compliance.rs`, nine contracts and decision doc; modifies FSM, runtime comment, bridge, SSB, config, CLAUDE and session protocol. |
| DEEP-E001-E38-023 | E-38:224-226 | Fallback when D-1 is absent: run only ClockProvider+audit liveness, mark hash assertion `Skipped`, emit `eu_ai_act_compliance_partial`, then re-tighten after D-1. |
| DEEP-E001-E38-024 | E-38:227-233 | Open gates: verify MRCR source and GPAI guidance; avoid single-GPU lock-in; surface DeepSeek legal status; preserve MoE path; verify dependencies; 90-day cadence may relax to 180 only with logging. |
| DEEP-E001-E38-025 | E-38:235-237 | Downstream posture informs E-39/E-24/E-37/E-40 and future MoE/Devstral/cloud backends; stale-doc warning must precede major roadmap shifts. |

### E-42_tree_sitter_grammars_2026.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E42-001 | E-42:1-6 | **PLANNED, 3 sessions/10–12h.** Refines planned B-30 and stabilizes the grammar substrate beneath in-flight B-29; changes `crates/knowledge/src/dkg/`, not chunk-level logic. |
| DEEP-E001-E42-002 | E-42:10-16 | Core target is 0.26.x/ABI 15. Grammars are maturity-tiered; composite parsing is one-level outer+injected reparse; Svelte 5 support is explicitly best-effort; Python `tree-sitter-language-pack` is forbidden in Rust. |
| DEEP-E001-E42-003 | E-42:43 | Cargo exact-pin instruction is `tree-sitter="=0.26"` in every importing crate, documented in `grammars.md`, with `cargo tree` verification against transitive bumps. |
| DEEP-E001-E42-004 | E-42:45 | `assert_abi_compatibility` calls the 0.26 language-version API for every loaded grammar, accepts versions **14 or 15**, fails fast with actionable error otherwise, and emits per-grammar `GrammarAbiProbeResult`. |
| DEEP-E001-E42-005 | E-42:47 | TIER-A registration claims “canonical 15” but enumerates rust, python, typescript, tsx, javascript, go, java, c, c-sharp, ruby, php, bash, html, css, json, yaml, toml, markdown; individual canonical crates are verified before addition; Markdown is CommonMark+GFM, never MDX. |
| DEEP-E001-E42-006 | E-42:49 | `experimental_grammars` gates kotlin, scala, swift, svelte, vue and optional C++; each load emits `GrammarTierBLoaded` with “may emit ERROR nodes.” Kotlin must be the `fwcd/tree-sitter-kotlin` grammar, not bindings. |
| DEEP-E001-E42-007 | E-42:51 | Svelte precheck detects `$state/$derived/$effect/$props`; tree walk counts ERROR+MISSING. With runes and ERROR rate >5%, downgrade that file to raw-text chunks and warn; emit `SvelteRunesParseHealth {file,error_node_count,missing_node_count,runes_detected}`. |
| DEEP-E001-E42-008 | E-42:53 | HTML/Svelte/Vue `injections.scm` maps script body→TypeScript and style→CSS and explicitly stops at one injection level. |
| DEEP-E001-E42-009 | E-42:55 | Experimental Astro split parses `---` frontmatter as TS and body as HTML/JS injections; RSC directives are ordinary TS syntax and have a TIER-A clean-parse regression test. |
| DEEP-E001-E42-010 | E-42:57 | Existing minified guard is average line length >500; add hard >2MB raw-text fallback and `TreeSitterSkippedLargeFile`; no parser rows for such input. |
| DEEP-E001-E42-011 | E-42:59 | Webview/WASM grammar path is guarded as unsupported because only Wasmtime is fully supported; predicted 1.5–2× slowdown is acceptable offline, not real-time; attempted load emits `tree_sitter_wasm_unsupported`. |
| DEEP-E001-E42-012 | E-42:61 | Regenerate `elai.map` with SCIP indices; previously supported languages must retain row count within ±5% and no SCIP_DEF/SCIP_REF regression. |
| DEEP-E001-E42-013 | E-42:63-95 | Wiring exports ABI types/functions, TIER-A/B registration and Svelte health plus four declared SSB enum variants. `DkgEngine::build_graph` is entry; CLI indexing flows through it. Smoke covers rs/py/ts/runes-Svelte/>2MB JSON and writes grammar-version audit. |
| DEEP-E001-E42-014 | E-42:76-80 | Dependency matrix names 17 TIER-A crates (TS crate also supplies TSX) and five TIER-B crates; mapper is a post-change consumer. |
| DEEP-E001-E42-015 | E-42:84-91 | Negative contract forbids language-pack, wrong core range, unwrap around ABI, ungated TIER-B, claims of full Svelte 5 support and MDX advertising. GrammarHealthPanel is optional/B-26-gated. |
| DEEP-E001-E42-016 | E-42:97-136 | Deterministic checks require exact pin, ABI definition+call, registration functions/cfg, no meta-dep/MDX, four SSB types, fwcd pin and a one-row-per-grammar audit artifact. |
| DEEP-E001-E42-017 | E-42:138-149 | Done gates require builds with/without feature, all ABI 14/15 rows, >5% raw fallback, >2MB skip, comprehensive `grammars.md`, mapper ±5%, no meta-dep/unwrap and an audit table with name/crate/version/ABI/tier/fork URL. |
| DEEP-E001-E42-018 | E-42:151-161 | Tests cover valid/invalid ABI, four runes/no false-positive, >5% fallback, 3MB skip, HTML injection, MDX non-crash without support, tier counts and kotlin fork pin. |
| DEEP-E001-E42-019 | E-42:163-174 | File lifecycle creates ABI/Svelte modules, `grammars.md`, shared injection query, audit and decision docs; modifies DKG, Cargo and SSB; dashboard panel remains optional. |
| DEEP-E001-E42-020 | E-42:190-198 | Risks: actual pins may report ABI14; Kotlin remains fragile; Svelte detection is not a parser fix; Vue must not use the Vue-2-only ikatyang fork; Astro remains experimental; mapper ±5 may need per-language tuning; tree-sitter never replaces SCIP semantics. |
| DEEP-E001-E42-021 | E-42:200-202 | E-42 precedes B-30 extraction. Oxc is separate; Astro/MDX/glimmer/nunjucks/liquid grammars are explicitly deferred for 12+ months. |

### E-55_agentic_rl_and_reward.md — complete ledger

| Atomic ID | Anchor | Explicit feature / behavior / state |
|---|---|---|
| DEEP-E001-E55-001 | E-55:1-6 | **PLANNED, 3 sessions/10–12h for non-weight rungs.** Extends planned D-2/D-3/D-5/D-13/D-23 and keeps D-28 deferred. Actual SFT is follow-up; weight-level DPO/PPO/GRPO/KTO is hard-deferred. |
| DEEP-E001-E55-002 | E-55:8-10 | Ordered posture is RLAIF > GEPA > PRM+ORM > MONA > SFT at ≥200 GREEN sessions > KTO+RLVR only after unlock >> DPO never. D-5 template-arm Thompson sampling is already soft contextual RL. |
| DEEP-E001-E55-003 | E-55:14-22 | D-5 receives fused observations/new arms; D-2 supplies PRM; D-3/E-53 supply ORM verifier consensus; D-13/B-37 supply cloud route; D-23/E-54 supply sycophancy; E-8 may own GEPA; no existing RLAIF/GEPA/MONA/SFT pipeline is assumed. |
| DEEP-E001-E55-004 | E-55:54 | `RlaifJudge::rate_trajectory` uses `PerRoleRuntime::cloud_sonnet`, samples 1/10 via reservoir, rates five Likert dimensions plus rationale/latency, and calibrates quarterly against 20 human-graded tasks. Correlation <.6 downgrades it to advisory. |
| DEEP-E001-E55-005 | E-55:56 | GEPA takes bottom quartile from K=100 PPM-scored rollouts, asks Sonnet for refinements, retains mean-reward×semantic-diversity Pareto arms, and evicts only when the arm’s 95th-percentile reward stays below leader median after ≥30 trials. Emits evolved/evicted. |
| DEEP-E001-E55-006 | E-55:58 | `FusedReward` combines PRM steps with `OrmSignal::{BuildGreen,TestsPass,ProbeSemantic,VerifierConsensus}` at configurable default .4/.6; alternative hierarchical mode filters bottom 50% by PRM before ORM. Feeds D-5. |
| DEEP-E001-E55-007 | E-55:60-65 | `CompositeReward=base-format-length-sycophancy-repetition`: .1/format violation; .05 if length >2× or <.3× median; .2 matched-pair sycophancy; .05 repeated consecutive 50-token sequence. Myopic >.8 plus RLAIF step-validity <.5 emits `RewardHackingSuspected`. |
| DEEP-E001-E55-008 | E-55:67 | `RolloutCollector` writes only GREEN (PASS, sycophancy-clean, no hacking flag) JSONL with prompt/trajectory/PRM/ORM/RLAIF/reward/diff/verdict. At 200 it emits `SftTrainingThresholdReached` and opens a ticket; no fine-tune occurs in E-55. |
| DEEP-E001-E55-009 | E-55:69 | Nightly canary uses 20 human-verified high-reward/low-quality cases and must flag ≥16/20; not a per-audit gate. |
| DEEP-E001-E55-010 | E-55:71 | D-5 Beta observation defaults to binary `composite_reward>.5`; optional Jeffreys/fractional update. New GEPA arms start Beta(1,1); every update emits `BaiArmObservationRecorded`. |
| DEEP-E001-E55-011 | E-55:73-77 | `UnlockStatus` requires all three: ≥3 stable-arm/no-tripwire but below-target sessions; ORM coverage ≥80%; and 2×3090 or rented 24GB headroom. Only then emit `RlWeightUpdateUnlocked`+ticket; until then DPO/PPO/GRPO/KTO are blocked. Registered conditional stub is `E-55.B7`. |
| DEEP-E001-E55-012 | E-55:79 | Contract rejects files/modules matching DPO/PPO/GRPO/DAPO/KTO until a recorded unlock; DPO remains specially prohibited due to data shape/OOD evidence. |
| DEEP-E001-E55-013 | E-55:81 | Four live panels expose fusion, judge/calibration, GEPA Pareto frontier and the three deferred-RL conditions; four matching Tauri commands query them. |
| DEEP-E001-E55-014 | E-55:83-88 | Decision doc records DPO rejection, SFT-first, three triggers, quarterly cadence, .4/.6 rationale and five measured TODO thresholds: RLAIF correlation, fusion ≥3pp, GEPA ≥5pp, SFT ≥5pp, and unknown BAI horizon. |
| DEEP-E001-E55-015 | E-55:90-119 | Wiring exports judge/evolver/fusion/composite/collector/unlock APIs, nine event names and four commands; consumes planned D surfaces, PerRoleRuntime, E-54 embeddings, E-53 verifier, blake3/serde; persists dated JSONL/index and stub registry. |
| DEEP-E001-E55-016 | E-55:120-135 | Negative contract: no unwrap/todo; no weight-update modules; ≥200 collection gate; 1/10 judge cap; <.6 advisory downgrade; GEPA diversity and ≥30 trials; default .4/.6; mandatory MONA; ≥80% canary; blake3; no direct Anthropic SDK. |
| DEEP-E001-E55-017 | E-55:136-139 | 100-rollout smoke expects ~10 judge rows, ≥1 evolved/0 evicted arm, 100 fusion+BAI rows, hacking evidence if applicable, fused posterior, and zero unlock rows. |
| DEEP-E001-E55-018 | E-55:141-216 | Deterministic checks cover five rating fields/sample cap, GEPA+30 trials, four ORM variants/.4-.6, four penalties/MONA, 200 threshold, three unlock fields, absent RL modules/direct SDK, blake3/no unwrap, stub entry, four panels and eight cargo smoke tests. |
| DEEP-E001-E55-019 | E-55:218-234 | Done gates require the routing/calibration/evolution/fusion/collector/posterior/unlock/canary/UI contracts, no RL modules, blake3/no unwrap, green workspace and verified citations while keeping all planned/deferred dependencies additive. |
| DEEP-E001-E55-020 | E-55:236-254 | Sixteen enumerated tests cover route/sample/calibration, three GEPA properties, two fusion properties, two MONA properties, two collector properties, unlock, BAI update and three contracts (module absence/canary/no direct SDK). |
| DEEP-E001-E55-021 | E-55:256-276 | File lifecycle creates six orchestrator modules, 17 stated test files, four panels, decision/training docs; modifies planned D surfaces, auditor, Tauri and session stub ledger. Missing D-5 may receive only a minimal integration stub. |
| DEEP-E001-E55-022 | E-55:309 | Open Q1: compare Sonnet vs PRM rank on ≥3 runs; <.6 means RLAIF adds signal, >.85 means skip it. |
| DEEP-E001-E55-023 | E-55:310 | Open Q2: fusion must beat PPM-only by ≥3pp on 50 held-out tasks or be removed. |
| DEEP-E001-E55-024 | E-55:311 | Open Q3: 100-rollout GEPA pilot costs ~$20–50 and must change the converged arm and win ≥5pp, otherwise template space is saturated. |
| DEEP-E001-E55-025 | E-55:312 | Open Q4: E-55.1 SFT on 200 GREEN sessions must lift local-Qwen held-out pass rate ≥5pp; otherwise training is not the bottleneck. |
| DEEP-E001-E55-026 | E-55:313 | Open Q5: no known BAI saturation curve; log arm stability/saturation per session to `E-55-bai-convergence-trace.jsonl`. |
| DEEP-E001-E55-027 | E-55:314-317 | Operational risks: default 100 judge calls/session then `RlaifQuotaExceeded`; quarantine GEPA arms flagged by MONA; allow per-task-family fusion overrides; E-55.1 needs new deps, 2×3090 and user confirmation. |
| DEEP-E001-E55-028 | E-55:318-321 | Canary may initially have only five seeds (4/5 threshold); cloud model must be exact-pinned/recalibrated; lazy stub registration can block without session; every cited arXiv ID must be externally verified before execution. |
| DEEP-E001-E55-029 | E-55:323-325 | Next ladder is evidence first: E-53→ORM, E-54→sycophancy penalty, possible E-52 probe, E-55.1 after all SFT gates, E-55.2 after fusion evidence, D-28 after BAI saturation, future KTO+RLVR/maybe DAPO, never DPO. |

## Explicit cross-source interactions

These records are explicit in at least one processed original; they do not invent ordering beyond cited text.

| Atomic ID | Anchor(s) | Interaction |
|---|---|---|
| DEEP-E001-INT-001 | IDX:141-145; E-2:1-5 | E-2 is an augmentation of shipped B-14/B-37 and shares CommandWindow with E-4; it must coordinate that file while leaving RuntimeAdapter unchanged. |
| DEEP-E001-INT-002 | E-2:182-184 | E-3 consumes FramePacer/SkeletonBubble; E-4 consumes the same bubble for later confidence/doom-loop UI, so E-2 should land first. |
| DEEP-E001-INT-003 | E-6:4-5,193-198; E-7:4-5 | E-6 and E-7 are parallel-safe, but E-7’s MAP-Elites `failure_mode` consumes E-6 `MastMode`; E-6 also feeds E-9/E-10 and merges with D-13 based on landing order. |
| DEEP-E001-INT-004 | E-7:14-23,224-229 | E-7 is a coordinated delta across D-1..D-9; it can supersede unshipped planned surfaces or amend shipped ones, while D-28 retraining remains separate. |
| DEEP-E001-INT-005 | E-19:27-31,251-255 | E-19’s PRM-lite is replaced by D-2, its L6 local diversity composes with D-13 cross-family review, and D-3 aggregates both. |
| DEEP-E001-INT-006 | E-19:57; IDX:84,107 | Typed-CoT depends on E-18’s XGrammar primitive, which in turn depends on the E-35 structured-output substrate. |
| DEEP-E001-INT-007 | E-19:220-237 | Conditional SE-Agent recombination is sequenced after committed E-19 and off-agent GAP-1 fitness; it is not part of DGM/E-7 self-modification. |
| DEEP-E001-INT-008 | E-36:18-23,229-233 | E-36 consumes C-7 Cosmos events, B-35 ArtifactCheck registry, B-37 roles and E-17 RRF; C-7 can block entry, and E-39 must govern cloud parse spend. |
| DEEP-E001-INT-009 | E-36:51-57 | Visual grounding combines image retrieval with a runtime VLM, but reliability tier controls downstream auditor/scope-planner weight and local mode can forbid cloud parsing. |
| DEEP-E001-INT-010 | E-38:28-35,224-237 | B-37 is the architecture hedge; C-15/C-27 and D-1..D-7 are compliance prerequisites. E-24 consumes MCP drift awareness; E-39/E-40 contribute adjacent cost/audit posture but are not declared function arguments. |
| DEEP-E001-INT-011 | E-42:18-25,200-202 | E-42 stabilizes grammar pins/health first; B-30 then builds extractors and B-29/codebase_mapper consume the set without replacing SCIP semantic analysis. |
| DEEP-E001-INT-012 | E-55:14-22,323-325 | E-55 fuses D-2 PRM, D-3/E-53 verifier outcome, D-23/E-54 sycophancy, D-13/B-37 cloud and possibly E-8 GEPA into D-5’s arm update; D-28 stays locked. |
| DEEP-E001-INT-013 | IDX:147-157,163-174 | All processed plans inherit blake3/no-unwrap/generate+processor/dependency-approval/Tier-C/hardware/auditor-only rules even when the individual file repeats only a subset. |

## Inferred compositions (not directly specified as one transaction)

| Atomic ID | Basis | Inference |
|---|---|---|
| DEEP-E001-INF-001 | E-2:41-59 | The user-visible stream state machine is likely `submit → skeleton → optional thinking cue → first token removes skeleton → paced text+cursor+cancel → stream_complete flushes/removes cursor → optional vote`; the parts are explicit, but no single source line declares this complete ordering. |
| DEEP-E001-INF-002 | E-6:43-63 | A guarded audit likely sends a typed finding, deterministically tags MAST, routes a different-family verdict, gates COMPLETE, then journals/dashboard-reports it; exact transaction/rollback semantics are not stated. |
| DEEP-E001-INF-003 | E-7:45-63 | A self-mod candidate likely flows MAP cell → EST/contamination/transfer/traps → Two-Gate → signed archive, but the complete ordering among tripwires is not explicitly defined. |
| DEEP-E001-INF-004 | E-19:47-65 | Reasoning route precedence is inferred as no-CoT class override, then confidence route, then difficulty-conditioned L4/L6; collision precedence between task class and confidence is not specified. |
| DEEP-E001-INF-005 | E-36:45-59 | Mixed PDF ingest likely parses text/tables, crops figures, conditionally indexes image multi-vectors, then fuses four retrieval lists and verifies visual claims; atomic steps are explicit but transaction/partial-failure recovery is absent. |
| DEEP-E001-INF-006 | E-38:56-68 | Compliance is intended as a pre-transition fail-closed aggregate over four assertions, but whether checks short-circuit or collect every failure is not stated. |
| DEEP-E001-INF-007 | E-42:45-61 | Startup registration likely probes ABI before indexing and per-file health then chooses parser/raw fallback; precise initialization failure isolation is not specified. |
| DEEP-E001-INF-008 | E-55:54-81 | Learning loop likely samples RLAIF, computes PRM/ORM+MONA, updates BAI, evolves/quarantines arms and collects only green outputs; exact ordering and atomicity across these writes is not defined. |

## Ambiguities and unresolved source choices

| Atomic ID | Anchor(s) | Ambiguity |
|---|---|---|
| DEEP-E001-AMB-001 | E-2:41,59,175 | Frame pacing is called default-true, randomized default-on/off, and off-by-default until telemetry. Shipping precedence is not resolved. |
| DEEP-E001-AMB-002 | E-2:43,88,121-122 | Skeleton removal is variously `stream_complete` and first token; observable wording says visible until first token, so intended removal likely first token but source task sentence groups both components together. |
| DEEP-E001-AMB-003 | E-2:45 | Env semantics say add kwarg “when env var is set,” then request defaults to env, while default-off and numeric default 10 coexist; behavior when request=true and env absent is not precisely specified. |
| DEEP-E001-AMB-004 | E-6:45,59 | `MastMode` is introduced as 14 real variants but FSM refers to `MastMode::None`, while other text models absence as `Option<MastMode>`; whether `None` is an enum variant or option value is unclear. |
| DEEP-E001-AMB-005 | E-6:49,188 | Cross-family cloud route both “refuses auditing” without env and is described as user opt-out per session; no explicit per-session override schema is given. |
| DEEP-E001-AMB-006 | E-7:47,218 | τ comes from per-cell held-out data, but `K_m=50` is only an unconfirmed default and no capacity metric/unit is defined. |
| DEEP-E001-AMB-007 | E-7:57 | `max(relative_delta=-20%, absolute_delta=-5%)` is syntactically ambiguous as a rejection formula; prose clarifies either threshold on any family, but inclusive boundary behavior is unstated. |
| DEEP-E001-AMB-008 | E-19:47,65,241 | The source calls .80 an initial constant and also an acceptance threshold tuned by ROC; it does not specify whether a tuned value replaces the compile-time constant or config overrides it. |
| DEEP-E001-AMB-009 | E-19:61,63 | No-CoT class and difficulty routes both consume FailureTaxonomy; precedence and whether direct L3 later reaches L4 are unstated. |
| DEEP-E001-AMB-010 | E-36:2,14,19,51 | “8th GroundingMethod” counting is unstable: FILE/SOURCE/TEST/WEB/API/API_REGISTRY + ArtifactCheck + CosmosCheck already names eight before VisualArtifactCheck; `VerificationMethod` versus `GroundingMethod` naming also shifts. |
| DEEP-E001-AMB-011 | E-36:45,223 | Pool-to-~100 algorithm is contingent on verifying a cited paper; fallback max/mean method and recall selection rule are not chosen. |
| DEEP-E001-AMB-012 | E-36:55,224 | Storage budget records a 500MB suggested default, but task schema names only budget/actual and does not specify eviction/rejection when exceeded. |
| DEEP-E001-AMB-013 | E-38:56,226 | Full four-assertion gate is required, yet pre-D-1 fallback permits `Skipped` and partial compliance; the deadline/automatic re-tightening mechanism is unspecified. |
| DEEP-E001-AMB-014 | E-38:60 | TOML field is described as numeric and also string `vendor-claim-only`; exact tagged schema/parser behavior is absent. |
| DEEP-E001-AMB-015 | E-42:43 | “0.26.x with patch-floor” conflicts semantically with exact `=0.26`, which generally denotes 0.26.0 rather than any 0.26 patch; intended pin is unclear. |
| DEEP-E001-AMB-016 | E-42:47,78,160 | “15 TIER-A languages” coexists with 18 named language labels, 17 grammar crates and a test count ≥15; authoritative count is unresolved. |
| DEEP-E001-AMB-017 | E-42:49 | TIER-B says 2–5/5 languages and “optionally cpp”; source does not decide whether C++ ships in the initial feature. |
| DEEP-E001-AMB-018 | E-42:59,72-75 | `tree_sitter_wasm_unsupported` is required by Task 9 but absent from the four-event `produces_symbols`/SSB registration lists. |
| DEEP-E001-AMB-019 | E-55:21,56 | E-55 says consume E-8 GEPA “if exists/shipped” without deciding ownership or duplicate-module migration. |
| DEEP-E001-AMB-020 | E-55:71 | Binary threshold is default while Jeffreys/fractional success is optional; config name and statistical update formula are not defined. |
| DEEP-E001-AMB-021 | E-55:73-79,123-124 | Contract blocks KTO until unlock, while final ladder permits KTO+RLVR after unlock; DPO is described both in a common “until unlock” list and as never allowed. DPO’s permanent ban is stronger, but test behavior after unlock is not specified. |
| DEEP-E001-AMB-022 | E-55:318 | Required 20-case/≥16 canary can initially ship with 5/≥4, without a deadline or done-gate exception mechanism. |
| DEEP-E001-AMB-023 | E-55:319 | Main text pins Sonnet 4.6, while risk example names `claude-sonnet-4-7-1m` and parenthetically references an Opus ID; exact judge model is unresolved. |

## Direct conflicts

| Atomic ID | Anchor(s) | Conflict and preservation rule |
|---|---|---|
| DEEP-E001-CON-001 | E-6:59 vs E-6:137 | Task 9 allows `MastMode::None OR cross_family_confirmed`; acceptance requires a confirmed cross-family verdict whenever `adversary_verdict_required` is true. Preserve the acceptance rule as stronger, but implementation prompt must resolve the `MastMode::None` bypass. |
| DEEP-E001-CON-002 | E-7:51 vs E-7:153 | Task defines EST violation as score variance **above** ε; acceptance parenthetical says a gameable candidate is “PRM score invariant” and blocks it, which reverses the stated detector. Do not silently choose; expected synthetic fixture must clarify whether invariance or non-invariance signals gaming. |
| DEEP-E001-CON-003 | E-19:59 vs E-19:181 | Task requires fresh-seed L1 plus rotated L3 agreement; the named test description says same-seed agrees/different-seed disagrees and omits rotated L3, so it does not fully prove the acceptance contract. |
| DEEP-E001-CON-004 | E-36:47 vs E-36:57 | Page confidence <.85 forces image indexing; cloud fallback threshold defaults .7. Pages in [.7,.85) get vision indexing without cloud parse, which may be intentional but the two “fallback” labels can be mistaken as one rule. Preserve as two distinct gates. |
| DEEP-E001-CON-005 | E-42:12,47 | Context maturity table names 18 TIER-A languages; objective/task repeatedly call the production set 15. Count cannot simultaneously be exact. Preserve names and flag count for decision. |
| DEEP-E001-CON-006 | E-55:69,229 vs E-55:318 | Acceptance requires 20 trajectories and ≥80%; risk authorizes initial five. A five-case launch cannot satisfy the stated done gate. |
| DEEP-E001-CON-007 | E-55:3,67,225 vs E-55:285 | Implementation gate is ≥200 GREEN rollouts, while SWE-Gym rationale is 491; the source correctly treats 491 as evidence, not ELAI threshold. Any 491 execution gate belongs only to the summary and conflicts with the original. |

## Summary additions and corrections

Baseline anchors refer to `docs/thefinalglance/feature-discovery/05-E-SUMMARY_INDEX.discovery.md` (`SUM`). Every delta below is grounded in this chunk’s originals; it is not a new design proposal.

| Atomic ID | Summary anchor | Addition/correction from originals |
|---|---|---|
| DEEP-E001-DEL-001 | SUM:3-11,1119-1124 | **Addition:** E-0 gives exact all-PLANNED-at-generation semantics, empty session log, full 55-item schema signal map, hard rules, lifecycle and default start order; summary delegated those details. |
| DEEP-E001-DEL-002 | SUM:1142-1154 | **Correction:** summary’s later wave sequence differs from E-0’s explicit immediate order E-17→E-2→E-30→E-27→E-15. Treat each as its source’s recommendation, not one canonical order. |
| DEEP-E001-DEL-003 | SUM:31-44 | **Addition E-2:** StaticCache/prefix-KV reuse, timing schema/histogram, cancel affordance, 200ms thinking cue, partial-fence state machine, randomized self-rating, settings/default gates, decision doc and complete tests were omitted. |
| DEEP-E001-DEL-004 | SUM:35,43 | **Correction E-2:** original defaults are 60 cps code/100 cps prose with 40–200 setting; summary’s 40–60 cps is not the complete configured behavior. |
| DEEP-E001-DEL-005 | SUM:44 | **Correction E-2:** 2–4× is report rationale for input-grounded tasks; original done gates require losslessness/acceptance telemetry, not an ELAI 2–4× throughput acceptance result. |
| DEEP-E001-DEL-006 | SUM:42 | **Correction E-2:** E-0 indexes `stream_smoothed`; the original produces `StreamingTimingRecorded` and `UxPacingVoteRecorded`. No original E-2 symbol named `stream_smoothed` is specified. |
| DEEP-E001-DEL-007 | SUM:98-112,1321 | **Addition E-6:** exact envelope variants/fields, debug/release handling, deterministic MAST mapping, cross-family env refusal, N=20/.55 persona anchor, provisional MistakeBook, FSM guard, CLI/dashboard, events and full gates resolve most queued gaps. |
| DEEP-E001-DEL-008 | SUM:105,112 | **Correction E-6:** original does not implement role deletion or require the smoke to prune a role; it reports ≥1σ lift or `insufficient_lift=true`. Role deletion belongs to E-10/index-level policy. |
| DEEP-E001-DEL-009 | SUM:108-110 | **Correction E-6:** generic summary events `mast_mode_classified/iso_budget_run/adversary_disagreement` come from E-0/summary; original declares `AuditMastTagged`, `IsoBudgetMeasured`, `PersonaReanchored`, `CrossFamilyUnavailable`. |
| DEEP-E001-DEL-010 | SUM:114-129,1322 | **Addition E-7:** axes/classes, τ∧K_m formula, D-2 .7 gates, independent verifier, DiscountedTS/SR, ε=.05 EST, tripwire auto-revert, .85+temporal+GUID contamination, -20/-5 transfer, 50×10 trap rules, signed-link schema and L3 lock contents. |
| DEEP-E001-DEL-011 | SUM:124-127 | **Correction E-7:** summary filenames/events (`two_gate.rs`, `map_elites.rs`, `map_elites_promoted`, `est_quarantined`) differ from original `promotion_gate.rs`, `template_archive.rs`, `ArchiveCellElevated`, `EstInvarianceMeasured`. |
| DEEP-E001-DEL-012 | SUM:122 | **Correction E-7:** only the 50-trap evaluation surface is undeferred; D-28’s retraining loop stays deferred until ≥20 promotion cycles. |
| DEEP-E001-DEL-013 | SUM:333-349,1333 | **Addition E-19:** exact .80/.40 router, 240-token re-sample prior, .6/.2/.2 scoring, 3+2 lateral policy, triple fields, fresh-seed+rotated-L3 rule, difficulty map, no-CoT classes, 200-trace ROC calibration and conditional recombination. |
| DEEP-E001-DEL-014 | SUM:348 | **Correction E-19:** “hard task runs L2” is wrong for original routing: confidence <.40 routes directly to L3; only medium confidence [.40,.80) runs L2. |
| DEEP-E001-DEL-015 | SUM:343-346 | **Correction E-19:** actual modified path is `crates/reasoning/src/chat_reflect.rs`, and original events are granular (`L2SkippedHighConfidence`, `L2SelfCertaintyScore`, `L4PrmLiteScore`, etc.), not `l2_confidence_gate`/`prm_lite_score` symbols. |
| DEEP-E001-DEL-016 | SUM:644-665,1347 | **Addition E-36:** default/alternate models, doctor probe, `VisionChunk`/PDF schemas, four artifact variants/stale rules, reliability enum, RRF4 weights, default-off ingest behavior, .7 cloud gate, .50/50-query benchmark, UI/commands and failure rules. |
| DEEP-E001-DEL-017 | SUM:657 | **Correction E-36:** original default is ColModernVBERT; ColQwen2.5-v0.2 is alternate, not the sole selected retriever. |
| DEEP-E001-DEL-018 | SUM:659-662 | **Correction E-36:** original routes by `claim_class:visual_spatial`, not PDF filename; code lives chiefly in `verification.rs`/knowledge/runtime and eval paths, not the summary’s `grounding.rs` + `workflow/vision_rag/`. |
| DEEP-E001-DEL-019 | SUM:663 | **Correction E-36:** E-0 has `visual_artifact_check`; original event is `VisualArtifactCheckVerdict` plus four other vision events. |
| DEEP-E001-DEL-020 | SUM:648-656 | **Correction E-36:** summary enumerates eight pre-existing methods while calling VisualArtifactCheck eighth. Original has the same count ambiguity; ordinal must be resolved before schema documentation. |
| DEEP-E001-DEL-021 | SUM:686-698,1349 | **Addition E-38:** exact four compliance assertions, fail/partial behavior, posture/MoE docs, FA/MRCR/architecture contracts, MCP version watch, protocol/screenshot/liveness guards, stale-doc rule, five events and full tests. |
| DEEP-E001-DEL-022 | SUM:693-694 | **Correction E-38:** direct composition with E-40 audit logs and E-39 `LayeredCapManualOverride` is not present in the original wiring contract; only downstream coordination/adjacency is stated. |
| DEEP-E001-DEL-023 | SUM:695-697 | **Correction E-38:** original artifacts are `docs/strategy/ELAI-2026-POSTURE.md`, `MoE-on-Ampere-tier.md`, and canonical `crates/orchestrator/src/compliance.rs`, not `workflow/compliance/eu_ai_act.md` or `compliance_gate.rs`. |
| DEEP-E001-DEL-024 | SUM:690,697-698 | **Correction E-38:** original gate covers Article 12+13 and emits `EuAiActComplianceBlocked/Verified` (and possible partial), not one generic `eu_ai_act_audit` row. |
| DEEP-E001-DEL-025 | SUM:753-768,1353 | **Addition E-42:** full named tier lists/forks, ABI {14,15}, exact >5% fallback, injection targets, Astro/RSC, >2MB/minified guard, WASM guard, mapper proof, audit artifact and tests. |
| DEEP-E001-DEL-026 | SUM:758 | **Correction E-42:** startup accepts ABI 14 **or** 15, not ABI 15 only. |
| DEEP-E001-DEL-027 | SUM:759 | **Correction E-42:** original’s “15” conflicts with its own 18 names/17 crates; summary cannot resolve the count and should not present 15 as exhaustive. |
| DEEP-E001-DEL-028 | SUM:763,767-768 | **Correction E-42:** original does not promise to close Svelte-5 grammar support; it calls it best-effort, falls back at >5% (not 30%), and uses raw-text chunking (not specifically line-window). |
| DEEP-E001-DEL-029 | SUM:764,766 | **Correction E-42:** original changes `dkg/mod.rs` and creates `grammars.md`; it emits `GrammarTierBLoaded`, not the summary/index `tier_b_grammar_failed` event. |
| DEEP-E001-DEL-030 | SUM:1024-1041,1366 | **Addition E-55:** committed zero-weight-update rungs are RLAIF, GEPA, PRM+ORM, MONA, BAI update and rollout collection; exact three unlocks, data schema, thresholds, dashboards, guards and open-question gates are all specified in the original. |
| DEEP-E001-DEL-031 | SUM:1033,1038,1040 | **Correction E-55:** actual SFT training is **not active** in E-55; only collector+ticket ships. Threshold is 200 GREEN rollouts and ≥5pp follow-up evidence, while 491 traces/12–14pp are SWE-Gym rationale. |
| DEEP-E001-DEL-032 | SUM:1039 | **Correction E-55:** index says `sft_on_rollouts_promoted`; original emits `SftTrainingThresholdReached` at collection threshold and reserves training for E-55.1. |
| DEEP-E001-DEL-033 | SUM:1036,1230,1366 | **Resolution E-55:** the three formerly unnamed unlocks are (1) three saturated-but-below-target BAI sessions with no tripwire, (2) ORM coverage ≥80%, (3) 2×3090 or rented 24GB headroom. |
| DEEP-E001-DEL-034 | SUM:1031-1032 | **Clarification E-55:** KTO+RLVR is a possible post-unlock first weight rung; DPO is permanently hard-avoided, not merely delayed. |
| DEEP-E001-DEL-035 | SUM:1047-1067 | **Addition:** original interactions add concrete dependency/fallback detail (E-6→E-7 MastMode, E-17/C-7→E-36, D-2/D-13→E-19, D/E-53/E-54→E-55) beyond the summary’s high-level compositions. |

## Coverage, omission, and duplicate audit

| Audit | Result |
|---|---|
| Manifest boundary | PASS — only the nine DEEP-E-001 originals were read; the permitted summary discovery file was orientation/comparison only. |
| Full-read coverage | PASS — every logical line in each original was emitted/read: 270+184+198+229+256+233+237+202+325 = 2,134. |
| Integrity | PASS — all nine individual SHA-256 hashes and line/byte counts equal the manifest; aggregate is 2,134 lines/239,903 bytes. |
| Required dimensions | PASS — ledgers cover capabilities/actions/surfaces; workflows/states; schema/data/lifecycle; config/defaults/flags; integrations/events; permissions/security/privacy/compliance; failure/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. |
| Explicit vs non-explicit | PASS — per-file records are EXPLICIT; inferred compositions, ambiguities and conflicts are isolated and labeled. |
| Atomic-ID uniqueness | PASS — 354 occurrences, 354 unique IDs; no duplicate IDs. |
| Per-prefix sequence gaps | PASS — numeric suffixes are contiguous from 001 to the recorded maximum within all 14 prefixes. |
| Semantic duplicate audit | PASS — repeated index/original concepts are retained once per source ledger because provenance differs; comparison records cross-reference them and are excluded from explicit-feature deduplication. No explicit atom is duplicated within a per-file ledger. |
| Summary delta coverage | PASS — every relevant summary section/gap for E-0/E-2/E-6/E-7/E-19/E-36/E-38/E-42/E-55 was compared; additions, corrections and resolved gaps are recorded above. |

## Final accounting

- Originals: **9 files / 2,134 lines / 239,903 bytes**.
- Orientation baseline: `05-E-SUMMARY_INDEX.discovery.md` only; not counted as an original.
- Atomic records: **354 total** = **281 explicit + 8 inferred interactions + 23 ambiguities + 7 conflicts + 35 summary deltas**.
- Output normalized SHA-256: see the self-hash field in Scope above.
