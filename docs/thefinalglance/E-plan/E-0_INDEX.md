# E-Plan Index — improvement-sweep-2026-05-19 (55 sessions)
Date: 2026-05-19
Source: workflow/improvement-sweep-2026-05-19/ (55 grounded reports, ~770 web fetches, ~75 papers cached)
Predecessors: B-plan (B-1..B-42), C-plan (C-1..C-59), D-plan (D-1..D-30).
Predecessor reading (mandatory before any E-N execution):
  workflow/improvement-sweep-2026-05-19/README.md (sweep overview, top-20 cross-cut priorities)
  docs/thefinalglance/summary/DETAILED_SUMMARY_01_overview_and_architecture.md (current state)
  docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md (every B-N status)
  docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md (every C-N status)
  docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md (every D-N status)
  This file (E-0_INDEX.md), then workflow/improvement-sweep-2026-05-19/E-plan/E-SUMMARY_INDEX.md.

How to use this index: each E-N is a self-contained prompt for one implementation session, mapped 1:1 to one sweep report. Every E-N closes a specific recommendation from a grounded report and was authored after greping B/C/D summaries + elai.map so the WIRING_CONTRACT cites real ELAI symbols, not invented ones. Each E-N's "Existing state (B/C/D coverage)" block names whether it is a DELTA, EXTENSION, or REPLACEMENT versus prior work.

CRITICAL: E-plan is GROUNDED-AGAINST-EXISTING, not net-new. Most reports are improvements to features already shipped (B-N DONE) or already planned (C-N / D-N PLANNED). Read the "Existing state" block of each E-N before executing — many E-N tasks collapse to a 1-line delta once an existing surface is identified.

## Status legend
- PLANNED — file written, work not started
- DEFERRED — research-grade / data-dependent / explicitly post-MVP (E-47, parts of E-55, parts of E-53)
- POSITIONING — non-code surface (E-23 is README/AGENTS.md/UX-copy only)
- IN PROGRESS — currently being shipped in an active session
- DONE — shipped and verified by smoke + benchmark

All 55 E-N files are PLANNED at generation. Status mutates per session log additions below.

================================================================================
TOP-20 CROSS-CUT PRIORITIES (READ THESE FIRST — from README §Top-20)
================================================================================

These are the highest-leverage E-Ns; ship-it-now beats strategic in this list.

  1. E-17  Stage the ONNX models. Knowledge engine silently degrades to BM25-only.
           ONE-DAY FIX, biggest leverage in the entire sweep.
  2. E-6   Cross-family Adversary is mandatory (Self-Correction Bench 64.5% blind-spot).
  3. E-27  Switch NF4→AWQ+Marlin for serving (keep NF4 for QLoRA only). 4.4× throughput.
  4. E-2   Frame-paced SSE rendering. One-day ship; fixes "letters come slowly" UX.
  5. E-30  Non-LLM PreToolUse argv-regex allowlist for destructive verbs.
  6. E-19  L2 Micro-Reflection is ELAI's weakest reasoning layer on a 7B.
  7. E-5   Memory illusion: Letta + Mem0 + Zep collapsed into SQLite (MINJA defense).
  8. E-15  Firecracker snapshot 28ms (or runsc/microsandbox), NOT chroot.
  9. E-9   Stop reporting SWE-bench Verified. Use Live + Pro + tau2 + private.
 10. E-12  Production-grade SaaS = day-1 auth + RLS + observability + a11y + headers.
 11. E-24  MCP partially adopted (external boundary only). Streamable HTTP since 2025-03.
 12. E-21  AGENTLESS-style less-decomposition wins. Delete agent roles without 1σ lift.
 13. E-8   GEPA (ICLR 2026 Oral) > MIPROv2 by +10pp. Cloud-Sonnet optimizer, local target.
 14. E-31  Qwen2.5-Coder lane has no successor; Qwen3 broke FA2. Strategic bet at risk.
 15. E-7   DGM itself reward-hacked. Two-Gate guardrail mandatory.
 16. E-37  Data-plane prompt-injection unguarded.
 17. E-39  Cost defense: ceiling enforcement BLOCKS not warns.
 18. E-3   Show ROLE, hide MODEL. Heterogeneous MoA loses to Self-MoA by 6.6%.
 19. E-43  Skills.md is now cross-vendor open standard.
 20. E-32  Raise 25K→32K native on Qwen2.5-Coder-7B. Free quality.
 (21+ continue: E-55 RL deferral, E-52 LSP as verifier, E-53 Z3+Kani+AutoVerus,
  E-51 MLX + llama.cpp via B-37, E-54 D-6 9-gram broken before deployment.)

================================================================================
WAVE MAP (50 reports across 10 waves + Wave 11 +5 = 55 E-Ns)
================================================================================

Wave 1 — UX + memory (chat experience pain)
  E-1   Infinite-Context Chat Illusion (Letta + Mem0 + Zep + HippoRAG; sleep-time digest)
  E-2   Local LLM Speed Tricks (SSE frame-pacing, prompt-lookup decoding)
  E-3   Model Handoff UX (show-role-hide-model; CUDA-graph warm pool)
  E-4   Chat UX Critique (sycophancy / persona-drift / repair-resistance + 14 UX rules)
  E-5   User Modeling & Memory (Mem0 + Zep collapsed to SQLite; MINJA defense)

Wave 2 — Orchestration quality + critique
  E-6   Multi-Agent Failure Modes (MAST 14 modes; iso-budget role audit)
  E-7   Darwinian Self-Improvement (DGM reward-hacked; Two-Gate guardrail)
  E-8   Self-Evolving Prompts (GEPA > MIPROv2; cloud-Sonnet optimizer)
  E-9   Agent Eval Benchmarks (SWE-Live + Pro + tau2; ClawsBench scaffold-vs-model)
  E-10  Critic of ELAI Design (14 hostile punches; 5 conceded strengths)

Wave 3 — Idea-to-product
  E-11  Idea-to-SaaS Prior Art (v0/Bolt/Lovable/Devin; Verified→Pro 87→23% cliff)
  E-12  Production Fullstack Codegen (BetterAuth + Drizzle + OTel + axe-core stack)
  E-13  TDD Verification Agents (PGS +13.4%; cargo-mutants AUDITING gate)
  E-14  Browser E2E Agents (AXTree+axe-core Tier 1; DeployPreviewWorker)
  E-15  Sandboxed Exec (Firecracker snapshot 28ms; chroot dead post-CVE)

Wave 4 — Validation + retrieval + routing
  E-16  VLM Image Validation (AXTree-YAML Tier 1; LPIPS-delta convergence)
  E-17  RAG Retrieval Quality (stage ONNX models; Contextual Retrieval; Jina-rerank-v3)
  E-18  Tool-Use Reliability (XGrammar Tier 2; SEARCH/REPLACE non-git fences)
  E-19  Reasoning Quality (L2 weakest layer; L4 AB-MCTS needs PRM-lite)
  E-20  Local/Cloud Routing (xRouter / RouterArena / C3PO; data-leakage rules)

Wave 5 — Peer landscape + Claude Code internals
  E-21  Peer Systems / LLM-as-Worker (Devin 3/20; AGENTLESS literature; Rust ecosystem)
  E-22  GitHub Trending May 2026 (SKILL.md cross-vendor; microsandbox/libkrun)
  E-23  Developer Discourse 2026 (METR -19%; SO trust 70→29% — POSITIONING)
  E-24  Agent Protocols / MCP (Adopt external-only; Streamable HTTP since 2025-03)
  E-25  Claude Code Architecture (98.4% deterministic infra; 6-tier compaction)

Wave 6 — Inference + kernels + observability + incidents
  E-26  Inference Engines 2026 (Hybrid: keep sidecar + SGLang adapter; drop Ollama)
  E-27  Quantization 2026 (Switch to AWQ+Marlin; FP8 KV = 2× context)
  E-28  Attention Kernels 2026 (Keep FA2 on Ampere; FA3/4 Hopper-only)
  E-29  Agentic Observability (OTel GenAI semconv 1.40.0; SSB-as-trace review)
  E-30  Public Agent Incidents (Replit 11× ALL CAPS ignored; 8-pattern taxonomy)

Wave 7 — Model + UI + Rust + structured output
  E-31  Coding Model Leaderboard (Qwen2.5-Coder lane has no successor)
  E-32  Context Window Extension (Raise to 32K native; F-in-M calibration +15pp)
  E-33  Dashboard UX Agentic (Activity ribbon; skeleton screens 20-30%)
  E-34  Rust AI Ecosystem 2026 (ort 2.0.0-rc.12 IS production — rule amendment)
  E-35  Structured Output 2026 (XGrammar tier-2; CodeAct +20%; SEARCH/REPLACE)

Wave 8 — Vision RAG + security + strategy + economics + graphs
  E-36  Document Vision RAG (ColQwen2.5; PaddleOCR-VL; VisualArtifactCheck GRC method)
  E-37  Prompt Injection Defense (Spotlighting + PromptArmor off-model classifier)
  E-38  AI Roadmap 2026 Signals (MoE dominates; RAG-first right; EU AI Act May)
  E-39  Cost Token Economics ($47K wallet-loop; ceiling BLOCKS not warns)
  E-40  Symbol Graph Tools (SCIP + tree-sitter hybrid; FocalAD cite misattributed)

Wave 9 — CI/CD + grammars + skills + durability + drift
  E-41  CI/CD PR Review (4-leg defense; Coderabbit/Greptile FP rates)
  E-42  Tree-Sitter Grammars 2026 (Pin 0.26.x ABI 15; TIER-A vs TIER-B)
  E-43  Agent Skill Systems (Publish 3 skills externally; signed install)
  E-44  Checkpointing Durability (ACRFence semantic rollback; idempotency-keyed)
  E-45  Cross-Language Drift (ts-rs snapshot tests; schemars+typify)

Wave 10 — Planning + fine-tune + deep research + metrics + SaaS
  E-46  Planning Decomposition (AGENTLESS-style; goal-drift Jaccard canary)
  E-47  LoRA Finetuning — DEFERRED (catastrophic forgetting 5-19pp; Unsloth ladder)
  E-48  Web Research Orchestrator (GhostCite / CITEVERIFIER; contradiction-surfacing)
  E-49  Coding Agent Metrics (STABLE-Pass@1 + $/CTS; HAL ICLR 2026)
  E-50  SaaS Architecture Patterns (Opinionated SaaS template; 4-archetype table)

Wave 11 — Edge + LSP + formal verification + eval + RL
  E-51  Edge Inference + MLX (MLX + llama.cpp as B-37 RuntimeAdapters)
  E-52  LSP Integration (LSP as VERIFIER not retriever; rust-analyzer headless)
  E-53  Formal Verification Codegen (Narrow Z3 + Kani + AutoVerus; Lean HARD AVOID)
  E-54  Eval Corpus Design (D-6 9-gram broken; per-family shards; matched-pair probes)
  E-55  Agentic RL and Reward — RL deferred (BAI is already soft RL; SFT-on-rollouts)

================================================================================
GATE FOR E-PLAN (all 55 E-Ns)
================================================================================

The E-plan is the post-D research-backlog layer. It is NOT sequenced as a single track — each E-N runs against whichever B/C/D phases it depends on, and many are parallel-safe with the entire C/D plan. The closure pattern is:

- Roughly half the E-Ns are AUGMENTATIONS of B-N SHIPPED features (E-1..E-5, E-13..E-16, E-21, E-23..E-26, E-29, E-44, E-48, E-51 — read each E-N's "Existing state" block for the specific B-N citation).
- Roughly a third EXTEND a planned C-N or D-N (E-6..E-10, E-17..E-20, E-27..E-28, E-30, E-37, E-39..E-43, E-45..E-50, E-52..E-55).
- A handful are NET-NEW surfaces (E-11..E-12 production-grade tier, E-22 SKILL.md export, E-31 strategic positioning, E-33 UX delta, E-36 VisualArtifactCheck, E-38 EU AI Act posture).

After all 55 E-Ns the user should see:
- Per-VRAM tier serving stack hardened (E-26, E-27, E-28, E-31, E-51 — AWQ+Marlin, FP8 KV, FA2 mandatory, MLX adapter, llama.cpp adapter).
- RAG quality lifted from BM25-only-at-runtime to true 3-ranker RRF + reranker + ONNX staged (E-17).
- Production-grade SaaS audit gates live (E-11, E-12, E-50 — RLS, OWASP, axe-lighthouse, gitleaks, OTel-seed).
- Memory illusion working without small-model collapse (E-1, E-5; BeliefShift + CTIM-Rover risks gated by E-49).
- Cross-family Adversary mandatory; same-model self-critique removed (E-6, E-9, E-13).
- Cost defense enforced (E-39 hard ceiling; E-20 routing; E-25 spend-aware compaction).
- LSP as verifier; rust-analyzer diagnostics pre-AUDITING (E-52).
- Mutation testing + IntentPair + MR active (E-13 + C-28 + C-29 stack).
- Formal verification narrow + gated (E-53; Lean explicitly rejected as routine gate).
- RL fine-tuning deferred behind 3 unlock conditions; SFT-on-rollouts as next rung (E-55).

================================================================================
HARD RULES INHERITED FROM CLAUDE.md (apply to every E-N)
================================================================================

- blake3 only (never blake2 / sha2 for non-RFC uses).
- No .unwrap()/.expect() in production paths (test cfg + main entrypoint allowed).
- No InProcessToolDispatcher in non-test production after C-13.
- FA2 mandatory; SDPA ~4K ceiling (E-28 enforces).
- model.generate() + LogitsProcessor only; never manual token loops.
- Anti-slopsquatting: verify package name + version + last-published date BEFORE adding (E-11 builds the PackageVerifier).
- Anti-stale-training: see CLAUDE.md anti-stale block. E-34 amends the ort-stable-not-rc line.
- Anti-sycophancy: report-flagged risks surfaced under each E-N's Open questions.
- No new external deps without user confirmation (E-22 microsandbox, E-26 SGLang, E-27 AWQ, E-33 npm cytoscape additions, E-37 PromptArmor, E-43 mcp-scan, E-51 mlx/llama.cpp — each requires explicit ok).
- Tier C isolation mandatory for CLI subprocesses (E-15 implements; E-41 GitHub adapter consumes).
- HARDWARE-HEAVY flag before sidecar boot; remote-only.
- One change at a time; auditor is the sole test runner.

================================================================================
SCHEMA_VERSION BUMP SITES (cumulative on top of B-42's 44 / B-41's 45 / C-27's 47)
================================================================================

Current shipped at sweep generation: 47 (B-41/B-42 + C-27).
E-plan bumps (each E-N's SSB variants):
  E-1   +A-MEM Zettelkasten + MemGPT Core/Archival rows (memory_promoted, memory_evicted)
  E-2   +stream_smoothed (frame-pacing flag)
  E-3   +role_shown_model_hidden (UX)
  E-4   +sycophancy_detected, persona_drift_alert
  E-5   +memory_fact_added/updated/deleted/noop (Mem0 dispatcher)
  E-6   +mast_mode_classified, iso_budget_run, adversary_disagreement (extends D-13)
  E-7   +two_gate_blocked, map_elites_promoted, est_quarantined (DGM hardening)
  E-8   +prompt_optimizer_proposed/shadow/promoted (GEPA)
  E-9   +bench_swelive_run, bench_tau2_run, harness_attribution_split
  E-10  +ablation_pruned_role (iso-budget delete)
  E-11  +scope_score_below_threshold, slopsquat_block, production_grade_block
  E-12  +production_check_failed (per checker), rls_probe_missing, biome_deny_violation
  E-13  +mutant_survived, anti_over_mock_block, fuzz4all_finding
  E-14  +axtree_snapshot, axe_violation, deploy_preview_ready
  E-15  +firecracker_snapshot_restored, seccomp_arg_blocked
  E-16  +rce_convergence_step, intra_modal_disagreement
  E-17  +onnx_model_staged, rerank_active, fitm_calibrated
  E-18  +xgrammar_constraint_enforced, search_replace_fenced
  E-19  +prm_lite_score, l2_confidence_gate
  E-20  +route_decision (xRouter), data_leakage_blocked
  E-21  +sbfl_localized, ace_audit_run
  E-22  +skill_exported, microsandbox_pool_size, twelve_factor_audit
  E-23  none (POSITIONING; README + AGENTS.md only)
  E-24  +mcp_tool_blake3_pinned, line_jumping_sanitized, streamable_http_active
  E-25  +compaction_tier_N, opusplan_substate, prompt_decision_evaluated
  E-26  +sglang_health_changed, ollama_deprecated
  E-27  +awq_loaded, fp8_kv_active, nf4_qlora_only
  E-28  +fa2_pinned, streamingllm_sink_active
  E-29  +otel_span_emitted (GenAI semconv columns)
  E-30  +destructive_verb_denied, mcp_pin_violation, auditor_lineage_collision
  E-31  +tier_model_swap (per-VRAM tier matrix)
  E-32  +context_lifted_32k, fitm_recall_lift_measured
  E-33  +activity_ribbon_frame, skeleton_screen_shown
  E-34  +ort_rc_amended, rmcp_adopted
  E-35  +xgrammar_substrate_loaded, codeact_step
  E-36  +visual_artifact_check (8th GroundingMethod)
  E-37  +spotlighting_envelope, promptarmor_verdict, egress_blocked
  E-38  +eu_ai_act_audit (Article 13 liveness)
  E-39  +hard_ceiling_blocked, retry_budget_exhausted
  E-40  +symbol_graph_dsl_query
  E-41  +pr_review_dispatched (cross-family reviewer)
  E-42  +tier_b_grammar_failed (cfg-gated experimental)
  E-43  +skill_imported (mcp-scan), skill_invocation_logged
  E-44  +idempotency_replay, saga_compensated, continue_as_new
  E-45  +action_type_golden_drift (action_type lock)
  E-46  +goal_drift_canary_fired, replan_phase, ledger_reset
  E-47  none (DEFERRED — emits ship-gate token if attempted)
  E-48  +citation_verified, contradiction_surfaced
  E-49  +stable_pass_at_1, harness_attribution_tuple, stop_verified
  E-50  +archetype_selected, rls_baby_step_emitted
  E-51  +mlx_loaded, llamacpp_loaded
  E-52  +lsp_diagnostic_probed (pre-AUDITING)
  E-53  +z3_consensus, kani_proved, autoverus_proved
  E-54  +ngram13_filtered, eval_aware_flag, per_family_shard_loaded
  E-55  +sft_on_rollouts_promoted (deferred RL ladder)

The order in which E-Ns ship determines the final SCHEMA_VERSION; each E-N's WIRING_CONTRACT block names its own bump scope.

================================================================================
CROSS-PLAN LINK (E → B / C / D)
================================================================================

See workflow/improvement-sweep-2026-05-19/E-plan/E-SUMMARY_INDEX.md §6 for the full per-E-N closure map. High-level:

- B-shipped phases AUGMENTED by E-plan: B-1, B-2, B-3, B-4, B-7, B-9, B-13, B-14, B-22, B-24, B-26, B-29 (PLANNED→E-40), B-36, B-37, B-39, B-40, B-41, B-42.
- C-planned phases EXTENDED by E-plan: C-5, C-7, C-8, C-11, C-12, C-13, C-15, C-16, C-17, C-21, C-22, C-23, C-24, C-25, C-26, C-27, C-28, C-29, C-30, C-31, C-32, C-33, C-35, C-39, C-40, C-41, C-42, C-44, C-48, C-50, C-51, C-52.
- D-planned phases EXTENDED by E-plan: D-1, D-2, D-3, D-4, D-5, D-6, D-7, D-9, D-10, D-11, D-12, D-13, D-14, D-15, D-16, D-17, D-18, D-19, D-20, D-21, D-22, D-23, D-25, D-28, D-30.

================================================================================
SESSION LOG (populated as E-Ns ship)
================================================================================

(Empty at generation. Each shipped E-N adds a row per its FSM cycle: context-compiler -> scope-planner-elai -> parallel explorers + synthesizer -> spec-grounder -> prompt-engineer -> impact-mapper -> EXECUTING -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator -> memory-writer -> COMMITTING. Mirror B-0_INDEX session log structure.)

================================================================================
NEXT STEP
================================================================================

Choose a starting E-N from the top-20 priorities above. Recommended ship order if no other constraints apply:

1. E-17 (RAG stage ONNX — ONE-DAY FIX, biggest single lift)
2. E-2  (frame-paced SSE — ONE-DAY UX fix)
3. E-30 (non-LLM PreToolUse argv allowlist — safety floor)
4. E-27 (AWQ+Marlin serving — 4.4× throughput)
5. E-15 (Firecracker — sandbox hard floor; chroot dead)

Then proceed by wave dependency, or pull whichever E-N closes a current pain point. Each E-N file has its own Pre-requisites line; respect those before opening it as a session.

End of E-0_INDEX.md.
