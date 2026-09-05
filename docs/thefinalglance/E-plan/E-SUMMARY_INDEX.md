# E-plan — Summary Index of All 55 Sessions
Generated: 2026-05-19 (synthesized from full read of E-1..E-55 + sweep README + per-batch agent reports)
Source files: 56 (E-0_INDEX + 55 E-N files), ~12,000 LOC of spec content
Purpose: condensed per-session punch list for fast navigation; each entry preserves the report mapping, status, pre-requisites, key deliverables (types/files), wiring spirit, and the observable-effect smoke trigger. Mirrors C-SUMMARY_INDEX.md format.

## 1. Premise & Reading Order

- Sweep ran 2026-05-19: 55 grounded reports, ~770 web fetches, ~75 PDFs cached, every claim arxiv- or URL-cited; conjecture explicitly labeled "Ungrounded:".
- Each E-N session ships one cohesive recommendation from one report, augments or extends one or more existing B/C/D surfaces, and is proven with an observable_effect smoke (CLI command → SSB row → artifact).
- 55 in-scope sessions; 1 fully DEFERRED (E-47 LoRA); 1 POSITIONING-only (E-23 developer discourse — README/AGENTS.md/UX-copy).
- Estimated effort per session: ~6–12 h. Total estimated effort: ~330–660 h across the full sweep.
- Files are loosely grouped by wave (10 sweep waves + 1 ELAI-internal wave 11); most are parallel-safe within a wave.

## 2. Wave Structure (from README + Wave 11 ELAI-specific)

- Wave 1 — UX + memory: E-1, E-2, E-3, E-4, E-5
- Wave 2 — Orchestration quality + critique: E-6, E-7, E-8, E-9, E-10
- Wave 3 — Idea-to-product: E-11, E-12, E-13, E-14, E-15
- Wave 4 — Validation + retrieval + routing: E-16, E-17, E-18, E-19, E-20
- Wave 5 — Peer landscape + Claude Code internals: E-21, E-22, E-23, E-24, E-25
- Wave 6 — Inference + kernels + observability + incidents: E-26, E-27, E-28, E-29, E-30
- Wave 7 — Model + UI + Rust ecosystem + structured output: E-31, E-32, E-33, E-34, E-35
- Wave 8 — Vision RAG + security + strategy + economics + graphs: E-36, E-37, E-38, E-39, E-40
- Wave 9 — CI/CD + grammars + skills + durability + drift: E-41, E-42, E-43, E-44, E-45
- Wave 10 — Planning + fine-tune + deep research + metrics + SaaS: E-46, E-47, E-48, E-49, E-50
- Wave 11 — Edge + LSP + formal verification + eval + RL: E-51, E-52, E-53, E-54, E-55

## 3. Per-Session Index

### E-1 — Infinite-Context Chat Illusion
- Closes: 01-infinite-context-chat-illusion
- Pre-reqs: E-5 (memory_facts substrate), B-37 SHIPPED
- Parallel-safe with: most of Wave 1+; conflicts with E-5 schema only
- Adds A-MEM Zettelkasten + MemGPT Core/Archival + sleep-time digest layer atop E-5's memory_facts table; reads voice_style facts for unified preamble; gates promotion behind BeliefShift/CTIM-Rover risk per Open questions
- Files: crates/memory/ (new); +memory_promoted / memory_evicted SSB
- Smoke: 5-turn session → digest emitted; turn 6 cites Archival fact via [memory:K] tag

### E-2 — Local LLM Speed Tricks
- Closes: 02-local-llm-speed-tricks
- Pre-reqs: dashboard-v2 streaming surface (B-14 DONE)
- Parallel-safe with: E-3, E-4, all of Wave 2+
- ONE-DAY SHIP: frame-paced SSE rendering (smoothStream-style) in CommandWindow + skeleton bubble + cursor sentinel; SECOND SHIP: prompt-lookup decoding in sidecar (HF prompt_lookup_num_tokens=10, preserves DoLa/ETC LogitsProcessor chain)
- Files: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte; python/elai_inference/server.py; +stream_smoothed SSB
- Smoke: chat reply renders at steady 40–60 chars/sec; sidecar throughput 2–4× on code-edit prompts (input-grounded)

### E-3 — Model Handoff UX
- Closes: 03-model-handoff-ux
- Pre-reqs: B-37 SHIPPED (PerRoleRuntime)
- Parallel-safe with: E-2, E-4
- Show ROLE, hide MODEL by default (Devin/Replit pattern); drawer-level model exposure for power users; reads E-5's voice_style fact for preamble; CUDA-graph warm pool documented but Hopper-deferred
- Files: dashboard-v2 SettingsWindow + ResponseCardPayload; +role_shown_model_hidden SSB
- Smoke: chat shows "Designer" role badge, no model name; toggle in Settings reveals model

### E-4 — Chat UX Critique
- Closes: 04-chat-ux-critique
- Pre-reqs: E-5 expertise facts
- Parallel-safe with: E-3
- Sycophancy detection + persona-drift alert + repair-resistance counter; 14 CommandWindow UX rules ported from report; calibrates terseness via E-5 fact_type='expertise'
- Files: crates/dashboard-v2/src/lib/intent/; +sycophancy_detected / persona_drift_alert SSB
- Smoke: user assertion "you're wrong" → no flip if confidence high; persona-drift across 10 turns → alert chip in StatusBar

### E-5 — User Modeling & Memory
- Closes: 05-user-modeling-and-memory
- Pre-reqs: none (foundational; E-1/E-3/E-4 depend on it)
- Parallel-safe with: E-2 only
- Mem0 ADD/UPDATE/DELETE/NOOP dispatcher + Zep bi-temporal scaffolding + MINJA poisoning defense; trust-score + temporal decay; SQLite memory_facts table; new crates/memory/ Rust crate
- Files: crates/memory/ (new ~600 LOC); +memory_fact_added/updated/deleted/noop SSB
- Smoke: "I prefer terse responses" → memory_fact_added fact_type='voice_style'; later turn reads it; adversarial inject blocked by MINJA gate

### E-6 — Multi-Agent Failure Modes
- Closes: 06-multi-agent-failure-modes
- Pre-reqs: D-13 Adversary (PLANNED), iso-budget harness (new in E-6)
- Parallel-safe with: E-7, E-8, E-9, E-10
- MAST 14-mode classifier + cross-family Adversary mandatory (Self-Correction Bench 64.5%) + iso-budget role audit (delete agent roles without >1σ lift); supersedes/extends D-13 same-family blind-spot
- Files: crates/orchestrator/src/mast.rs (new); +mast_mode_classified / iso_budget_run / adversary_disagreement SSB
- Smoke: synthetic same-family executor+auditor pair → adversary_disagreement on adversarial trace; iso-budget run prunes 1 role without delta loss

### E-7 — Darwinian Self-Improvement
- Closes: 07-darwinian-self-improvement
- Pre-reqs: D-5 BAI, D-8 archive, D-9 L3 frozen (all PLANNED)
- Parallel-safe with: E-6, E-8
- Two-Gate guardrail (bounded-capacity, arxiv:2510.04399) + MAP-Elites archive + EST quarantine + Ed25519 chain + undeferred trap fixtures; closes Sakana DGM reward-hacking incident gap
- Files: crates/orchestrator/src/two_gate.rs + map_elites.rs; +two_gate_blocked / map_elites_promoted / est_quarantined SSB
- Smoke: synthetic reward-hacking variant → two_gate_blocked + quarantined; clean variant promotes

### E-8 — Self-Evolving Prompts
- Closes: 08-self-evolving-prompts
- Pre-reqs: D-1 ReasoningTemplate (PLANNED); cloud Sonnet route available
- Parallel-safe with: E-6, E-7
- GEPA (ICLR 2026 Oral) admission via SHADOW lifecycle, cloud-Sonnet optimizer + local Qwen target (small LMs cannot self-optimize per arxiv:2405.10276); offline Python optimizer pipeline ships blake3-hashed templates to ELAI's archive
- Files: workflow/optimizer/ (Python, offline); +prompt_optimizer_proposed/shadow/promoted SSB
- Smoke: 24h offline run produces ≥1 GEPA-promoted template; live cloud /task uses it

### E-9 — Agent Eval Benchmarks
- Closes: 09-agent-eval-benchmarks
- Pre-reqs: C-51 Aider polyglot (PLANNED)
- Parallel-safe with: E-6, E-7, E-49
- Stop reporting SWE-bench Verified (87.6→23.1% cliff, 24.4% inflated); adopt SWE-Live + Pro + tau2-bench + private held-out; paired-bootstrap CI; harness-vs-model attribution tuple
- Files: crates/cli/src/benchmark.rs (extend); +bench_swelive_run / bench_tau2_run / harness_attribution_split SSB
- Smoke: `elai benchmark --suite swelive` writes paired-bootstrap CI + attribution tuple to phase_*.toml

### E-10 — Critic of ELAI Design
- Closes: 10-critic-of-elai-design
- Pre-reqs: depends on E-6 iso-budget harness
- Parallel-safe with: E-7, E-8
- 14 hostile-critic punches → 5 concrete FSM/role ablations + 5 conceded strengths preserved; uses E-6 iso-budget delta test as the deletion gate
- Files: workflow/reasoning/ablations/ (docs + harness); +ablation_pruned_role SSB
- Smoke: ablation harness on 1 role yields measured Δ<1σ → ablation_pruned_role row; PR ablates the role from agent_pool

### E-11 — Idea-to-SaaS Prior Art
- Closes: 11-idea-to-saas-prior-art (extends B-40 SHIPPED)
- Pre-reqs: B-40, B-41 SHIPPED
- Parallel-safe with: E-12, E-50
- Scope-clarity score on IdeaCritic + ProductionGrade auditor mode (B-41 mode extension) + PackageVerifier (slopsquatting 19.7%) + per-level critic + ANTI_SPIRAL_HARD_ABORT + Aider unified-diff
- Files: crates/orchestrator/src/idea_framing.rs (extend); +scope_score_below_threshold / slopsquat_block / production_grade_block SSB
- Smoke: idea "build me a calculator" → scope_score above threshold; vague idea → block; npm package "react-icns" → slopsquat_block

### E-12 — Production Fullstack Codegen
- Closes: 12-production-fullstack-codegen
- Pre-reqs: E-11, B-30 PLANNED, B-37 SHIPPED
- Parallel-safe with: E-50
- 7 framework-convention manifests + FrameworkAdapter trait + 15-item production_checklist runners (gitleaks / OWASP semgrep / RLS probe / axe-lighthouse / OTel-seed / Biome+Ruff+Clippy-deny) + assertion-only-diff BLOCK
- Files: crates/orchestrator/src/production_grade/; +production_check_failed (per checker) / rls_probe_missing / biome_deny_violation SSB
- Smoke: greenfield SaaS pipeline → 15/15 checks pass before COMMITTING; deliberate RLS-missing variant → rls_probe_missing block

### E-13 — TDD Verification Agents
- Closes: 13-tdd-verification-agents (augments C-28/C-29)
- Pre-reqs: C-28 mutation, C-29 IntentPair (PLANNED)
- Parallel-safe with: E-12, E-14
- Property-gate + targeted cargo-mutants survival cache + anti-over-mock PostToolUse + mutant-hunter agent + Fuzz4All CONCEPT-level gate + ≥1-non-LLM-verifier requirement; targeted, not workspace-wide
- Files: crates/orchestrator/src/mutation_runner.rs (extend); +mutant_survived / anti_over_mock_block / fuzz4all_finding SSB
- Smoke: PR with over-mocked test → anti_over_mock_block; clean PR survives targeted mutant run

### E-14 — Browser E2E Agents
- Closes: 14-browser-e2e-agents (extends B-24 SHIPPED + B-42 SHIPPED)
- Pre-reqs: B-24, B-42 SHIPPED; E-15 sandbox profile for npx playwright
- Parallel-safe with: E-12, E-15
- AXTree-first Playwright snapshot-to-disk + axe-core gate + constrained-decoding VLM Tier 2 + hybrid SSIM-first diff + asymmetric veto + DeployPreviewWorker (Cosmos) + multi-context SaaS-journey + Cloudflare/Turnstile detect-and-skip-never-bypass
- Files: crates/orchestrator/src/browser_e2e/; +axtree_snapshot / axe_violation / deploy_preview_ready SSB
- Smoke: greenfield app build → AXTree snapshot diff committed; deploy preview ready; intentional a11y bug → axe_violation block

### E-15 — Sandboxed Exec
- Closes: 15-sandboxed-exec (extends B-18 + C-11 PLANNED)
- Pre-reqs: B-18 / C-11 PLANNED
- Parallel-safe with: E-14 (consumer), E-22 (sandbox-pool sibling)
- Firecracker hot snapshot-restore pool + arg-checked seccomp socket() (AF_INET/INET6 only) + NO_NEW_PRIVS→CAPBSET_DROP→seccomp ordering + chroot codepath REMOVED + gVisor KVM-less fallback + veth+iptables researcher net allowlist + fork-from-snapshot per baby step + AppContainer Windows trio MUST
- Files: crates/orchestrator/src/sandbox/ (extend); +firecracker_snapshot_restored / seccomp_arg_blocked SSB
- Smoke: novel baby step → fork-from-snapshot 28ms; CVE-2025-31133 PoC blocked

### E-16 — VLM Image Validation
- Closes: 16-vlm-image-validation (DELTA on B-24 SHIPPED)
- Pre-reqs: B-24 SHIPPED
- Parallel-safe with: E-14
- AXTree + anti-sycophancy + RCE convergence add-ons WITHOUT rewriting tier1/feedback_loop; intra-modal sibling of D-13 cross-family Adversary
- Files: crates/orchestrator/src/media_validator/ (extend); +rce_convergence_step / intra_modal_disagreement SSB
- Smoke: deliberate-fail screenshot → intra_modal_disagreement; honest VLM verdict preserved

### E-17 — RAG Retrieval Quality ★ THE BIG ONE
- Closes: 17-rag-retrieval-quality (closes B-4 degraded-runtime gap)
- Pre-reqs: B-1 SHIPPED, B-4 SHIPPED
- Parallel-safe with: most of E-plan
- HIGHEST-LEVERAGE SINGLE FIX in entire sweep: stage GraphCodeBERT/Granite/cross-encoder ONNX models at .elai_cc/models/onnx/; activate code_vec query path (currently ELAI_CODE_VEC=1 opt-in); add Anthropic Contextual Retrieval + Jina-rerank-v3; Found-in-the-Middle +15pp; fixes DEEP MRR < STANDARD MRR monotonicity violation
- Files: scripts/setup_onnx.sh (new); knowledge engine config; +onnx_model_staged / rerank_active / fitm_calibrated SSB
- Smoke: blake3-verified ONNX models on disk → DEEP MRR > STANDARD MRR in phase_3K_rag.toml

### E-18 — Tool-Use Reliability
- Closes: 18-tool-use-reliability (sequences after E-35)
- Pre-reqs: E-35 XGrammar substrate, B-7 SHIPPED via B-7b
- Parallel-safe with: E-17, E-19, E-20
- XGrammar Tier 2 constraint enforcement + SEARCH/REPLACE non-git fences + Hermes tool envelope + path-allowlist mask + reliability per-tool counter; E-35 supplies the substrate, E-18 wires the tool-call call sites
- Files: crates/tools/src/grammar.rs (extend), python/elai_inference/server.py (XGrammar mask); +xgrammar_constraint_enforced / search_replace_fenced SSB
- Smoke: tool call with malformed JSON → XGrammar reshapes to valid; SEARCH/REPLACE patch lands on first try

### E-19 — Reasoning Quality
- Closes: 19-reasoning-quality (extends B-39 SHIPPED)
- Pre-reqs: B-39 SHIPPED; D-2 PPM (PLANNED, future)
- Parallel-safe with: E-18, E-20
- L2 Micro-Reflection gating behind L1-confidence check (self-certainty per arxiv:2502.18581); PRM-lite scoring for L4 AB-MCTS (stopgap before D-2 ships); Typed-CoT JSON triples via E-18 XGrammar mask
- Files: crates/orchestrator/src/reasoning/ (extend), crates/orchestrator/src/chat_reflect.rs (extend); +prm_lite_score / l2_confidence_gate SSB
- Smoke: easy task → L2 skipped due to L1 high-confidence; hard task → L2 fires; AB-MCTS branches scored by PRM-lite

### E-20 — Local/Cloud Routing
- Closes: 20-local-cloud-routing (extends B-27 PLANNED + B-37 SHIPPED + C-5 PLANNED)
- Pre-reqs: B-37 SHIPPED; B-27 PLANNED (E-20 augments)
- Parallel-safe with: E-17, E-18, E-19
- xRouter / RouterArena / C3PO routing policies + data-leakage rules (E-37 sister) + privacy-tier-aware path mask (sharing path-allowlist primitive with E-18); orthogonal to D-10 mode routing (where to run vs what to verify)
- Files: crates/orchestrator/src/routing.rs (extend); +route_decision / data_leakage_blocked SSB
- Smoke: prompt mentions API key → routed local; routed cloud requires redaction check first

### E-21 — Peer Systems / LLM-as-Worker
- Closes: 21-peer-systems-llm-as-worker
- Pre-reqs: B-22 SHIPPED; AGENTLESS evidence pull
- Parallel-safe with: E-22, E-23
- 7 peer-grounded steals: PageRank repo-map, SBFL on failing tests, critic-rerank, ACI command-shape audit, repo-as-state-store, multi-provider routing exposure via B-37 PerRoleRuntime, SWE-bench-Verified subcommand
- Files: crates/orchestrator/src/sbfl.rs (new); +sbfl_localized / ace_audit_run SSB
- Smoke: failing test set → SBFL emits suspiciousness ranking; agent self-selects the top-1 file

### E-22 — GitHub Trending May 2026
- Closes: 22-github-trending-may2026
- Pre-reqs: B-18 PLANNED, C-11 PLANNED
- Parallel-safe with: E-21, E-43 (coordinates on SKILL.md export)
- SKILL.md export shim + microsandbox/libkrun MicroVm sandbox boot (Linux; Windows falls back to Process) + deferred MCP schema loading flag + Streamable HTTP transport audit + elai.map vs claude-context benchmark + 12-factor audit
- Files: scripts/export_skills.py (new); crates/orchestrator/src/sandbox/ (extend); +skill_exported / microsandbox_pool_size / twelve_factor_audit SSB
- Smoke: `elai skill export` produces signed claude.skills.io payload; sandbox pool boots in <100ms

### E-23 — Developer Discourse 2026 [POSITIONING]
- Closes: 23-developer-discourse-2026
- Pre-reqs: depends on at least one of E-21/E-24/E-25 landing first
- Parallel-safe with: all other E-Ns (no shared code surface)
- POSITIONING-only — no new code surface. ContextStatePanel + TrustTelemetryPanel + productivity-baseline (not "speedup proven" per grounded-by-default) + CommandWindow first-run rewrite in maintainer voice + README / AGENTS.md citation hygiene + pricing-transparency badge + prompt-injection check status surface
- Files: README.md, .claude/AGENTS.md, dashboard-v2 panels; no new SSB
- Smoke: first-run dashboard shows the new copy; AGENTS.md publishes

### E-24 — Agent Protocols / MCP
- Closes: 24-agent-protocols-mcp (extends B-3 batch-ful-d MCP bridge)
- Pre-reqs: B-3 SHIPPED (MCP bridge)
- Parallel-safe with: E-25, E-30, E-37
- blake3 schema-hash pinning + Trail-of-Bits line-jumping sanitization + per-tool token-budget enforcement + Streamable HTTP migration (replaces SSE since 2025-03) + LLM-decision pre-MCP-call guard + explicit gateway-rejection guard + 6 new contract tests
- Files: crates/plugins/src/mcp_bridge.rs (extend); +mcp_tool_blake3_pinned / line_jumping_sanitized / streamable_http_active SSB
- Smoke: external MCP server with mutated tool description → blake3 mismatch blocks call; Streamable HTTP transport active

### E-25 — Claude Code Architecture
- Closes: 25-claude-code-architecture (extends Phase 3H session handoff)
- Pre-reqs: Phase 3H session handoff in code
- Parallel-safe with: E-24 (consumer of prompt_decision primitive), E-44
- 6-tier compaction ladder above existing SESSION_HANDOFF + deferred MCP dashboard surface + `!\`cmd\`` skill templating + live reload + shared `prompt_decision` LLM-decision hook primitive (consumed by E-24 Task 5) + opusplan-style substate runtime overlay on B-37 PerRoleRuntime + session forking + pre-edit checkpoints + 7 anti-pattern guards
- Files: crates/orchestrator/src/compaction.rs (extend), crates/dashboard-v2/src/lib/drawers/CompactionDrawer.svelte (new); +compaction_tier_N / opusplan_substate / prompt_decision_evaluated SSB
- Smoke: context approaches 85% → tier-1 compaction fires; subsequent tiers escalate; full session reconstructs from any tier

### E-26 — Inference Engines 2026
- Closes: 26-inference-engines-2026 (augments B-37 SHIPPED)
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: E-27, E-28 (all edit python/server.py — sequence)
- SGLang RuntimeAdapter slot + XGrammar LogitsProcessor + Ollama deprecation; preserves DoLa/ETC LogitsProcessor chain; vLLM V1 logits-processor breakage flagged Open Q
- Files: crates/providers/src/runtime/sglang.rs (new), python/elai_inference/server.py (extend); +sglang_health_changed / ollama_deprecated SSB
- Smoke: SGLang adapter health probe green; ollama backend → warn-and-skip on construction

### E-27 — Quantization 2026 ★ HIGH-ROI
- Closes: 27-quantization-2026
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: E-26, E-28 (sequence python/server.py edits)
- Switch serving NF4 → AWQ+Marlin (same HumanEval 51.8%, 4.4× throughput); FP8 KV cache (Ada+ native / Ampere sw-sim, 2× context window free); NF4 reserved for QLoRA training only
- Files: python/elai_inference/server.py (extend), workflow/quantize/ (new); +awq_loaded / fp8_kv_active / nf4_qlora_only SSB
- Smoke: sidecar boots with AWQ+Marlin; throughput measured ≥4× over NF4 baseline at same HumanEval

### E-28 — Attention Kernels 2026
- Closes: 28-attention-kernels-2026
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: E-26, E-27 (sequence python/server.py)
- FA2 mandatory enforced + flash-attn 2.8.3 bump + StreamingLLM sinks + YaRN eval-gated + FA3/FA4 BLOCKED on Ampere; SDPA path explicit refuse-to-boot on 7B without FA2
- Files: python/elai_inference/server.py (extend); +fa2_pinned / streamingllm_sink_active SSB
- Smoke: sidecar boot without FA2 → refuse; FA2 active → fa2_pinned row at startup

### E-29 — Agentic Observability
- Closes: 29-agentic-observability (extends SSB / Phase 3B)
- Pre-reqs: Phase 3B SSB shipped
- Parallel-safe with: E-30 (consumer of new SSB types)
- OTel GenAI semconv 1.40.0 (Apr 2026) columns on SSB + ssb-otel-export crate + Transcript/Failure-modes/Spend/Query drawers + ssb_resume_from replay + MAST 14-mode classifier (sister of E-6)
- Files: crates/ssb_otel_export/ (new), crates/dashboard-v2 drawers; +otel_span_emitted SSB
- Smoke: live session emits GenAI-conformant OTLP spans to local collector; drawer replays a past session

### E-30 — Public Agent Incidents ★ SAFETY FLOOR
- Closes: 30-public-agent-incidents
- Pre-reqs: B-19 hook unification (PLANNED)
- Parallel-safe with: E-24, E-29
- Non-LLM PreToolUse argv-regex destructive-verb allowlist + CVE-fix trust-dialog ordering + MCP exact-version+blake3 pinning + auditor-lineage collision guard + content-source trust labels + red-team contract 15/15; closes Replit "11× ALL CAPS ignored" + Cursor Plan-Mode bypass + Claude Code rm -rf shipped twice
- Files: crates/plugins/src/pretool_argv.rs (new); +destructive_verb_denied / mcp_pin_violation / auditor_lineage_collision SSB
- Smoke: `rm -rf ~/` argv → destructive_verb_denied regardless of LLM verdict

### E-31 — Coding Model Leaderboard
- Closes: 31-coding-model-leaderboard (strategic positioning)
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: E-32, E-34
- Strategic per-VRAM tier matrix + EscalationPolicy module + Qwen3 FA2-incompat flagged + MoE bet documented as ELAI's riskiest strategic bet + non-DoLa/non-steering MoE-compatible execution path
- Files: crates/orchestrator/src/escalation_policy.rs (new), CLAUDE.md tier-table refresh; +tier_model_swap SSB
- Smoke: per-VRAM tier resolution returns correct model+backend tuple for 12/24/48GB

### E-32 — Context Window Extension
- Closes: 32-context-window-extension
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: E-17, E-31
- Raise ELAI_LOCAL_MAX_CONTEXT 25K→32K native on Qwen2.5-Coder-7B (free, model trained there); Found-in-the-Middle calibration (arxiv:2406.16008) +15pp RAG recall free; skip YaRN/StreamingLLM/RingAttention (15-20% quality cost)
- Files: CLAUDE.md sidecar env, python/elai_inference/server.py (max_context bump); +context_lifted_32k / fitm_recall_lift_measured SSB
- Smoke: 32K-context prompt fits; FITM-calibrated RAG recall lifts on the 30-query corpus

### E-33 — Dashboard UX Agentic
- Closes: 33-dashboard-ux-agentic (UX delta on B-26 DONE)
- Pre-reqs: B-26 DONE
- Parallel-safe with: all non-dashboard E-Ns
- 5 new Svelte components (Activity ribbon Kiro-style, skeleton screens, …) + Wong palette + reduced-motion preference + footer mode; pure frontend-pair, no backend overlap; respects existing DESIGN.md outlined-only chrome
- Files: crates/dashboard-v2/src/lib/windows/* (extend); +activity_ribbon_frame / skeleton_screen_shown SSB
- Smoke: dashboard load → skeleton bubbles render; Activity ribbon scrolls smoothly at 60fps with reduced-motion respected

### E-34 — Rust AI Ecosystem 2026
- Closes: 34-rust-ai-ecosystem-2026 (amends CLAUDE.md anti-stale)
- Pre-reqs: none
- Parallel-safe with: all other E-Ns
- CLAUDE.md anti-stale rule amendment (ort 2.0.0-rc.12 IS production per ELAI's own Cargo.toml:50 — rule was wrong); rmcp 1.7.0 / genai / insta / proptest adoption; outdated-tech-audit.md appendix; B-PLAN-SUMMARY.md correction BEFORE B-16 runs to prevent breakage
- Files: CLAUDE.md (rule amendment), outdated-tech-audit.md (appendix), B-PLAN-SUMMARY.md (correction); +ort_rc_amended / rmcp_adopted SSB
- Smoke: B-16 enforcement run after E-34 lands does NOT regress on ort dep

### E-35 — Structured Output 2026 ★ SUBSTRATE
- Closes: 35-structured-output-2026 (substrate E-18/E-19/E-20 consume)
- Pre-reqs: none
- Parallel-safe with: all non-sidecar E-Ns
- XGrammar substrate (sidecar grammar-mask layer) + INSTRUCTOR mode + CodeAct (+20% accuracy / -30% turns) + SEARCH/REPLACE non-git fences + qwen_tools parser; lands BEFORE E-18 so tool-call wiring collapses to thin calls
- Files: crates/providers/src/decoding.rs (extend), python/elai_inference/server.py (XGrammar mask); +xgrammar_substrate_loaded / codeact_step SSB
- Smoke: structured-output task with JSON schema → zero retries; CodeAct mode produces measurable -30% turn count

### E-36 — Document Vision RAG
- Closes: 36-document-vision-rag (8th GroundingMethod)
- Pre-reqs: B-35 ArtifactCheck, B-42 CosmosCheck, Phase 2B grounding stubs
- Parallel-safe with: most of E-plan
- New GroundingMethod::VisualArtifactCheck (8th method alongside FILE/SOURCE/TEST/WEB/API/API_REGISTRY/Artifact/Cosmos); ColQwen2.5 ViDoRe v2 0.597 + PaddleOCR-VL 0.9B; routes design.pdf / api-spec.pdf claims to vision verifier
- Files: crates/reasoning/src/grounding.rs (extend), workflow/vision_rag/ (new); +visual_artifact_check SSB
- Smoke: PDF claim → VLM extracts + verifies; non-PDF claim falls through to existing methods

### E-37 — Prompt Injection Defense
- Closes: 37-prompt-injection-defense (extends E-24 + visibility filter)
- Pre-reqs: E-24 MCP hardening
- Parallel-safe with: E-39, E-50
- Spotlighting envelope on ingest + blake3 (NOT SHA-256 per CLAUDE.md anti-stale) pin on MCP tool descriptions + PromptArmor off-model classifier + egress allowlist + markdown-image strip; closes data-plane prompt-injection gap (visibility filter + MCP deny-list defend control plane only)
- Files: crates/plugins/src/prompt_injection.rs (new); +spotlighting_envelope / promptarmor_verdict / egress_blocked SSB
- Smoke: hostile-injected PDF page → PromptArmor classifier verdict logged; egress to non-allowlist URL blocked

### E-38 — AI Roadmap 2026 Signals
- Closes: 38-ai-roadmap-2026-signals (compliance posture)
- Pre-reqs: phase-3B SSB
- Parallel-safe with: most of E-plan
- EU AI Act May 2026 enforcement audit-trail liveness gate (Article 13) + MoE strategic bet doc + RAG-first posture through 2027; composes with E-40 symbol-graph audit logs and E-39 LayeredCapManualOverride SSB row
- Files: workflow/compliance/eu_ai_act.md, crates/orchestrator/src/compliance_gate.rs (new); +eu_ai_act_audit SSB
- Smoke: PLANNING → TESTING_FIRST transition emits Article-13-conformant audit row

### E-39 — Cost Token Economics ★ HARD CEILING
- Closes: 39-cost-token-economics (extends B-27 PLANNED + C-5 PLANNED)
- Pre-reqs: B-27 PLANNED, C-5 PLANNED
- Parallel-safe with: E-20 (consumer)
- Hard ceiling enforcement BLOCKS not warns; one-shot retry max then escalate; per-tool spend gates; pre-execution difficulty estimator before cloud route; closes $47K wallet-loop / 2.3M retry storm incidents; LayeredCapManualOverride for user-driven override
- Files: crates/orchestrator/src/cost_gate.rs (extend); +hard_ceiling_blocked / retry_budget_exhausted SSB
- Smoke: per-task cap=$0.05 + warn=80% → hard_ceiling_blocked at $0.05; retry budget exhausted → escalate not retry

### E-40 — Symbol Graph Tools
- Closes: 40-symbol-graph-tools (extends B-29 + C-24 PLANNED)
- Pre-reqs: B-29 PLANNED, C-24 PLANNED
- Parallel-safe with: E-52
- SCIP + tree-sitter hybrid (already in elai.map); Stack Graphs archived Sep 2025 (skip); FocalAD arxiv:2506.11419 misattributed flagged (third bad-cite pattern after B-40/B-42 — feedback_paper_citation_verify); SymbolGraphDSL query layer
- Files: crates/knowledge/src/symbol_dsl.rs (new); +symbol_graph_dsl_query SSB
- Smoke: `elai symbol dsl "callers(SsbActionType)"` returns deterministic ranked list

### E-41 — CI/CD PR Review
- Closes: 41-ci-cd-pr-review (extends C-40 PLANNED GitHub Adapter)
- Pre-reqs: C-40 PLANNED
- Parallel-safe with: E-45 (paired PR-time gate)
- 4-leg defense (in-flow / PR / deploy / observe) + heterogeneous PR reviewer (different family from author) + Semgrep AI ruleset + dependency-verify-as-tool-call + per-baby-step commits + `--no-ff` merge + git-trailer-only AI attribution per .claude/rules/commit-cadence.md
- Files: crates/github_adapter/ (extend C-40); +pr_review_dispatched SSB
- Smoke: PR opened → cross-family reviewer comment lands; Semgrep AI rules surface +ssrf finding; ELAI fixes before merge

### E-42 — Tree-Sitter Grammars 2026
- Closes: 42-tree-sitter-grammars-2026 (extends B-30 PLANNED)
- Pre-reqs: B-30 PLANNED
- Parallel-safe with: most of E-plan
- Pin tree-sitter 0.26.x + ABI 15 probe + TIER-A (15 langs production) / TIER-B (cfg-gated experimental) split + Svelte 5 runes ERROR-rate telemetry + injection.scm queries; closes "Svelte 5 runes ungrammared" gap
- Files: crates/knowledge/src/grammars.rs (extend), Cargo.toml pins; +tier_b_grammar_failed SSB
- Smoke: Svelte 5 runes file → ERROR-rate < 30%; falls back to line-window chunker on TIER-B fail

### E-43 — Agent Skill Systems
- Closes: 43-agent-skill-systems (extends/coordinates E-22)
- Pre-reqs: B-19 PLANNED, B-20 PLANNED
- Parallel-safe with: E-22 (coordinates SKILL.md export)
- agentskills.io v1 frontmatter compliance + per-skill eval/ directories + skill invocation telemetry (blake3) + publish targets (claude/codex/gemini) + mcp-scan import gate + new .claude/rules/skill-import-policy.md; closes ToxicSkills + ClawHavoc incidents
- Files: .claude/skills/* (frontmatter add), scripts/skill_publish.sh (new); +skill_imported / skill_invocation_logged SSB
- Smoke: `elai skill publish` → signed payload accepted by claude.skills.io; unsigned skill import blocked by mcp-scan

### E-44 — Checkpointing Durability
- Closes: 44-checkpointing-durability (extends B-9 SHIPPED)
- Pre-reqs: B-9 SHIPPED
- Parallel-safe with: E-25 (compaction sibling), E-45 (snapshot-test sibling)
- Idempotency-keyed tool wrappers + SAGA compensation log + SessionDag Continue-As-New rollover + madsim DST harness + lazy checkpoint migrator for SCHEMA_VERSION 40→45; closes ACRFence semantic rollback attack on Claude Code
- Files: crates/orchestrator/src/idempotency.rs (new), crates/orchestrator/src/saga.rs (new); +idempotency_replay / saga_compensated / continue_as_new SSB
- Smoke: tool-call retry returns idempotent result (same key, same result); SAGA compensation on partial failure

### E-45 — Cross-Language Drift
- Closes: 45-cross-language-drift (extends B-3 ts-rs base)
- Pre-reqs: B-3 SHIPPED (ts-rs)
- Parallel-safe with: E-41 (paired PR-time gate)
- ts-rs v12 snapshot test + schemars on sidecar boundary + action_type golden-file lock (Protobuf-style append-only) + extended .claude/tools/cross_lang_diff.py --strict mode + prek pre-commit hook; new SSB action types MUST flow through golden file at land-time
- Files: crates/contracts/tests/ts_rs_snapshot.rs (new), crates/contracts/tests/action_type_golden.rs (new); +action_type_golden_drift SSB
- Smoke: rename Rust SsbActionType variant → cross_lang_diff.py exit 1; golden-file lock blocks until both sides update

### E-46 — Planning Decomposition
- Closes: 46-planning-decomposition (extends B-22b 4G.9 + B-40 SHIPPED)
- Pre-reqs: B-22 SHIPPED, B-40 SHIPPED
- Parallel-safe with: E-48, E-49
- Dual-ledger + AGENTLESS leaf shortcut + Reflexion buffer + goal-drift Jaccard+cosine + entropy-stop + PRM scalar + REPLAN_PHASE / LEDGER_RESET cascade; widens 4G.9 canary
- Files: crates/orchestrator/src/decomposition.rs (extend); +goal_drift_canary_fired / replan_phase / ledger_reset SSB
- Smoke: synthetic goal-drift trajectory → canary fires + REPLAN_PHASE; clean trajectory completes

### E-47 — LoRA Finetuning [DEFERRED]
- Closes: 47-lora-finetuning
- Pre-reqs: RunPod for retraining; D-30 PLANNED
- Parallel-safe with: ALL (RESEARCH-GRADE)
- DEFERRED per report itself (catastrophic forgetting 5-19pp). SPEC.md-only Python subpackage; no .py files shipped; CI lint enforces DEFERRED_PER_E47_REPORT ship-gate token. Unsloth+QLoRA+PiSSA+DoRA ladder documented but not built
- Files: workflow/research/lora/SPEC.md ONLY; no SSB
- Smoke: ship-gate token present → CI lint passes; attempt to add .py without unlock → CI fail

### E-48 — Web Research Orchestrator
- Closes: 48-web-research-orchestrator (extends B-7 SHIPPED + C-48 SafeHarbor PLANNED)
- Pre-reqs: B-7 SHIPPED, B-39 SHIPPED
- Parallel-safe with: E-46
- Editable plan + Brave/SearXNG/Mojeek tiered fan-out + crawl4ai/Jina extractor + cross-encoder rerank + FlashResearch pruner + conformal triple-threshold + ContradictionNode + 4-stage CitationVerifier (GhostCite hardening) + sub-agent attribution lock
- Files: crates/orchestrator/src/web_research_orchestrator.rs (extend); +citation_verified / contradiction_surfaced SSB
- Smoke: research query → ≥80% citation_verified pass rate; contradiction across sources surfaces in dashboard

### E-49 — Coding Agent Metrics
- Closes: 49-coding-agent-metrics (extends B-23 + C-50 + C-51 PLANNED)
- Pre-reqs: C-50, C-51 PLANNED
- Parallel-safe with: E-9, E-46
- STABLE-Pass@1 (n=5) + $/CTS + METR 50% horizon + harness-vs-model attribution tuple + private held-out eval + 30-day revert canary + prompt-injection ASR + eval-awareness + context-rot + Cover@τ + STOP-Verified CI lint
- Files: crates/cli/src/metrics.rs (extend); +stable_pass_at_1 / harness_attribution_tuple / stop_verified SSB
- Smoke: bench run emits 7-tuple per task: (pass_rate, cost_per_token, horizon_min, harness_attr, ASR, eval_aware, cover_tau)

### E-50 — SaaS Architecture Patterns
- Closes: 50-saas-architecture-patterns (extends E-11 + E-12)
- Pre-reqs: E-11, E-12 (consumer)
- Parallel-safe with: most of E-plan
- 4 archetype enum (B2C/B2B/API-first/AI-app) + mandatory RLS BabyStep emitter + 6-check production-readiness gate + scoped-token enforcer + OTel/SLO/runbook scaffolds + MoR-vs-Stripe-direct decision fork + 4 archetype seed TOMLs; Designer consumes the catalog
- Files: workflow/archetypes/*.toml (new), crates/orchestrator/src/archetype.rs (new); +archetype_selected / rls_baby_step_emitted SSB
- Smoke: "build me a B2C SaaS" → archetype_selected=B2C; RLS baby step emitted in pipeline

### E-51 — Edge Inference + MLX
- Closes: 51-edge-inference-and-mlx (extends B-37 SHIPPED)
- Pre-reqs: B-37 SHIPPED
- Parallel-safe with: most of E-plan
- Two new RuntimeBackend variants: Mlx (Apple Silicon fast-path, M5 Max measured 158 tok/s on Qwen 1.7B) + LlamaCpp (universal GGUF fallback); both as `/v1/messages` adapters; NPU HARD-DEFERRED; Tauri-webview + transformers.js + WebGPU consideration documented
- Files: crates/providers/src/runtime/mlx.rs (new), crates/providers/src/runtime/llamacpp.rs (new); +mlx_loaded / llamacpp_loaded SSB
- Smoke: macOS M-series host → MLX adapter resolves; any host with GGUF → llama.cpp adapter resolves

### E-52 — LSP Integration
- Closes: 52-lsp-integration (extends B-19 PLANNED + B-29 PLANNED)
- Pre-reqs: B-19 PLANNED, B-29 PLANNED
- Parallel-safe with: E-40
- LSP as VERIFIER not retriever (correcting the cited "LSP doesn't help" finding which was retriever-only); rust-analyzer headless CLI (`analysis-stats`, `scip`, `lsif`, `ssr`, `diagnostics`); multilspy multi-LSP client; jdtls/RA RAM hogs documented; diagnostic-probe pre-AUDITING gate via PostToolUse hook
- Files: crates/plugins/src/lsp_verifier.rs (new); +lsp_diagnostic_probed SSB
- Smoke: PostToolUse Write/Edit → rust-analyzer diagnostics run; deliberate type-error → diagnostic surfaces before AUDITING

### E-53 — Formal Verification Codegen
- Closes: 53-formal-verification-codegen (extends D-3 PLANNED + C-52 PLANNED)
- Pre-reqs: D-3 PLANNED
- Parallel-safe with: E-52, E-54
- Narrow Z3 SMT in D-3 verifier consensus, gated by CSQ Critical; Kani for Rust hot paths in AUDITING; AutoVerus-style Verus annotation for ELAI's FSM/IPC invariants (CI-gating COMMITTING, not per-audit); Dafny as verification-aware IR for safety-critical Python. HARD AVOID: Lean theorem-prover loops as routine audit gate (TPU-day latency)
- Files: crates/orchestrator/src/verifier/{z3,kani,verus,dafny}.rs (new); +z3_consensus / kani_proved / autoverus_proved SSB
- Smoke: CSQ Critical financial-data function → Z3 consensus required for promotion; Lean-loop attempt → CI lint rejection

### E-54 — Eval Corpus Design
- Closes: 54-eval-corpus-design (REPLACES D-6 9-gram, EXTENDS D-23)
- Pre-reqs: D-6 PLANNED (E-54 replaces its 9-gram filter)
- Parallel-safe with: E-49
- D-6 9-gram → 13-gram (proven bypassable per arxiv:2311.04850 — same as GPT-3); per-family directory shard + _index/{family}.jsonl to avoid NTFS perf cliffs at 5K tasks; matched-pair (eval vs deploy) sycophancy probes for D-23 (Sonnet 4.5 verbalizes "I'm being evaluated" >80%); held-out test-triplet ORM
- Files: .elai_cc/held-out/ structure refactor, scripts/eval_corpus_audit.py (new); +ngram13_filtered / eval_aware_flag / per_family_shard_loaded SSB
- Smoke: contamination scan with 13-gram catches CodeForces sample D-6 9-gram misses; 5K-task fixture loads in <1s

### E-55 — Agentic RL and Reward
- Closes: 55-agentic-rl-and-reward (extends D-5 PLANNED, D-28 DEFERRED)
- Pre-reqs: D-5 PLANNED (E-55 layers SFT-on-rollouts above it)
- Parallel-safe with: E-46, E-49
- Don't fine-tune with RL yet — D-5 BAI Thompson sampling is already soft contextual RL (logarithmic regret). DPO HARD AVOID (3-7pp OOD drop per Apple 2025). When escalating, use KTO+RLVR; SFT-on-rollouts (SWE-Gym pattern, +12-14pp from 491 traces) likely beats any RL fine-tune. 3 unlock conditions before any RL ladder
- Files: workflow/research/rl_ladder/SPEC.md (DEFERRED spec); workflow/training/sft_on_rollouts/ (NEW, practical next rung); +sft_on_rollouts_promoted SSB
- Smoke: 491-trace SFT run produces ≥+5pp on private held-out; full RL ladder remains DEFERRED until 3 conditions satisfied

---

## 4. E-N → B / C / D Closure Map (Augments / Extends / Replaces)

| E-N | Augments (B DONE)        | Extends (C PLANNED) | Extends (D PLANNED)           |
|-----|--------------------------|---------------------|-------------------------------|
| E-1 | —                        | —                   | D-1 ReasoningTemplate (consumer) |
| E-2 | B-14                     | —                   | —                             |
| E-3 | B-37                     | —                   | —                             |
| E-4 | —                        | C-7                 | —                             |
| E-5 | —                        | —                   | — (foundational)              |
| E-6 | —                        | —                   | D-13 (supersede)              |
| E-7 | —                        | —                   | D-5, D-8, D-9                 |
| E-8 | —                        | —                   | D-1                           |
| E-9 | —                        | C-51                | —                             |
| E-10 | —                       | —                   | —                             |
| E-11 | B-40, B-41              | C-21, C-31, C-32    | —                             |
| E-12 | B-30 (extends), B-37    | C-31, C-32          | —                             |
| E-13 | —                       | C-28, C-29          | D-3                           |
| E-14 | B-24, B-42              | C-7, C-16           | —                             |
| E-15 | —                       | C-11 (extends B-18) | —                             |
| E-16 | B-24                    | —                   | —                             |
| E-17 | B-1, B-4                | C-8, C-17           | —                             |
| E-18 | B-7                     | C-13                | —                             |
| E-19 | B-39                    | —                   | D-2 (stopgap), D-3            |
| E-20 | B-37                    | C-5                 | D-10                          |
| E-21 | B-22                    | —                   | —                             |
| E-22 | —                       | C-11                | —                             |
| E-23 | (POSITIONING)           | —                   | —                             |
| E-24 | B-3 (MCP bridge)        | C-35, C-39, C-42    | —                             |
| E-25 | Phase-3H handoff        | —                   | —                             |
| E-26 | B-37                    | —                   | —                             |
| E-27 | B-37                    | —                   | —                             |
| E-28 | B-37                    | —                   | —                             |
| E-29 | Phase-3B SSB            | —                   | —                             |
| E-30 | —                       | C-44                | D-9                           |
| E-31 | B-37                    | —                   | —                             |
| E-32 | B-37                    | —                   | —                             |
| E-33 | B-26                    | C-6, C-7            | —                             |
| E-34 | (amends CLAUDE.md)      | —                   | —                             |
| E-35 | (substrate for E-18+)   | —                   | —                             |
| E-36 | B-35, B-42              | —                   | —                             |
| E-37 | —                       | C-42, C-48          | —                             |
| E-38 | Phase-3B SSB            | —                   | —                             |
| E-39 | —                       | C-5                 | —                             |
| E-40 | —                       | C-24                | —                             |
| E-41 | —                       | C-40                | —                             |
| E-42 | —                       | —                   | (extends B-30 PLANNED)        |
| E-43 | —                       | —                   | (extends B-19, B-20 PLANNED)  |
| E-44 | B-9                     | —                   | —                             |
| E-45 | B-3 (ts-rs)             | —                   | —                             |
| E-46 | B-22, B-40              | —                   | —                             |
| E-47 | (DEFERRED)              | —                   | D-30 (PLANNED, DEFERRED)      |
| E-48 | B-7, B-39               | C-48                | —                             |
| E-49 | —                       | C-50, C-51          | —                             |
| E-50 | (consumes E-11, E-12)   | —                   | —                             |
| E-51 | B-37                    | —                   | —                             |
| E-52 | —                       | —                   | (extends B-19, B-29 PLANNED)  |
| E-53 | —                       | C-52                | D-3                           |
| E-54 | —                       | —                   | D-6 (replaces 9-gram), D-23   |
| E-55 | —                       | —                   | D-5, D-28 (DEFERRED)          |

## 5. Cross-Cutting Hard Rules Across Sessions

- blake3 only; SHA-256 explicitly rewritten (E-37 amended report's SHA-256 → blake3)
- No .unwrap()/.expect() in production (CLAUDE.md inheritance)
- FA2 mandatory (E-28 enforces sidecar-refuses-to-boot without FA2 on 7B)
- model.generate() + LogitsProcessor only; E-26 SGLang adapter preserves this
- Anti-slopsquatting: E-11 PackageVerifier blocks; E-15 evicts state post-install
- Anti-stale: E-34 amends CLAUDE.md "ort stable not -rc" rule (was wrong)
- Anti-sycophancy: E-4 detection; report-flagged Open questions surfaced in every E-N
- No new external deps without confirmation; E-22/E-26/E-27/E-33/E-37/E-43/E-51 each require explicit ok
- Tier C sandbox mandatory for CLI subprocesses; E-15 implements, E-22/E-41 consume
- E-47 DEFERRED with hard ship-gate token; E-55 RL ladder DEFERRED with 3 unlock conditions; E-53 Lean HARD AVOID
- E-23 is POSITIONING-only (no new code surface — README/AGENTS.md/UX-copy only)

## 6. Total Effort and Coverage Summary

- Total session slots: 55
- In-scope (executable now, CPU/cloud): 53 (E-1..E-46, E-48..E-55 minus deferred items)
- DEFERRED entirely: 1 (E-47)
- POSITIONING-only (no code): 1 (E-23)
- Files authored: 57 (E-0_INDEX + this summary + 55 E-N files, all 2026-05-19)
- Estimated effort per implementation session: ~6–12 h
- Total estimated effort: ~330–660 h (excludes deferred RunPod compute)
- Verification source: 55 grounded reports + ~770 web fetches + ~75 cached PDFs; every E-N's WIRING_CONTRACT grounded against elai.map / B / C / D summaries.

## 7. SCHEMA_VERSION Bump Sites

See E-0_INDEX.md "SCHEMA_VERSION BUMP SITES" section for the full per-E-N list. The ship order determines final cumulative SCHEMA_VERSION on top of the post-C/D 47+ baseline.

## 8. Wave-Wise Dependency Sketch (recommended ship sequence)

Wave A (foundational substrate, NO prereq, run in parallel where shown):
  E-5 (memory) — needed by E-1, E-3, E-4
  E-17 (RAG ONNX stage) ★ ONE-DAY FIX
  E-2 (frame-paced SSE) ★ ONE-DAY FIX
  E-34 (CLAUDE.md amend) — un-block B-16
  E-35 (XGrammar substrate) — needed by E-18 / E-19

Wave B (orchestration safety):
  E-30 (PreToolUse argv allowlist) ★ safety floor
  E-15 (Firecracker sandbox)
  E-37 (data-plane prompt injection)
  E-24 (MCP hardening) — needed by E-37 land path

Wave C (serving + inference):
  E-27 (AWQ+Marlin) ★ HIGH-ROI — sequence with E-26, E-28
  E-26 (SGLang adapter)
  E-28 (FA2 mandatory)
  E-32 (32K context lift)
  E-51 (MLX + llama.cpp adapters)

Wave D (production-grade SaaS):
  E-11 (idea → SaaS)
  E-12 (production fullstack checklist)
  E-50 (SaaS archetype catalog)
  E-14 (browser E2E)

Wave E (eval + measurement):
  E-9 (SWE-Live + Pro + tau2)
  E-13 (TDD + targeted mutants)
  E-49 (STABLE-Pass@1 + $/CTS)
  E-54 (D-6 9-gram → 13-gram + per-family shards)

Wave F (reasoning + cross-family):
  E-6 (Adversary + iso-budget)
  E-19 (L2 confidence gate + PRM-lite)
  E-8 (GEPA offline)
  E-7 (DGM Two-Gate)

Wave G (durability + drift):
  E-44 (idempotency + SAGA)
  E-45 (cross-language drift snapshots)
  E-25 (6-tier compaction)

Wave H (everything else, parallel-safe with previous waves):
  E-3, E-4, E-10, E-16, E-18, E-20, E-21, E-22, E-23, E-29, E-31, E-33, E-36, E-38, E-39, E-40, E-41, E-42, E-43, E-46, E-48, E-52, E-53

Wave I (deferred / research):
  E-47 (LoRA) — DEFERRED entirely
  E-55 (RL ladder) — DEFERRED; SFT-on-rollouts as practical next rung

---

End of E-SUMMARY_INDEX.md.
