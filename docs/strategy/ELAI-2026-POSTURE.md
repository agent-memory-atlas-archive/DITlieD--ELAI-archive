ELAI 2026 strategic posture spec

This document is the canonical reference the ELAI codebase consults when an architectural decision invokes "the 2026 ecosystem says X." Every claim below cites a primary source (WebSearch-verified or fetched directly). Ungrounded conjecture is forbidden -- a missing citation is a defect, log via the posture-doc-stale check at `crates/orchestrator/src/roadmap_signals.rs::check_posture_doc_age`.

E-38 lands the producer-side substrate (Pattern-2 SSB `AiRoadmapSignalEvent`, EU AI Act compliance pre-gate, MCP transport-version log, MRCR-unverified backend probe, posture-doc-stale signal, dashboard `RoadmapPanel`). This document is the prose half of that landing -- the bets, hedges, signals, and per-lab snapshots ELAI's runtime invariants encode.

Cross-references: `workflow/reasoning/E-38-decisions.md`; `docs/strategy/MoE-on-Ampere-tier.md`; `CLAUDE.md` E-38 posture reference HARD RULE.


SECTION 1 — Five top-level signals shaping 2026-2027 posture

(1) MoE is the de-facto frontier architecture across Mixtral, Gemini 3, GPT-5.5, DeepSeek V4, Qwen 3.6, Llama 4. ELAI's dense-only Qwen2.5-Coder bet (per `memory:project_qwen3b_baseline`) remains correct for the 7-12 GB VRAM tier, but the runtime layer MUST NOT hard-code dense assumptions. B-37 RuntimeAdapter (SHIPPED, commit fd5c2e5, 11 backends + PerRoleRuntime + RuntimeOrchestrator + RuntimePanel) is the architectural hedge. MoE-on-Ampere via expert-offload is a documented next-tier deliverable -- see `docs/strategy/MoE-on-Ampere-tier.md` for the design space + arXiv citations.

Source: callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms (verified May 2026).

(2) FlashAttention-2 is legacy 2027+. FA-3 is Hopper-only (H100/H200). FA-4 targets Hopper+Blackwell via CuTeDSL. Ampere (RTX 3090, the user's actual hardware) stays on FA-2 indefinitely and loses FA-3/FA-4 performance gains. ELAI's `ELAI_FORCE_SDPA=1` fallback (per `memory:project_qwen3b_baseline`, server.py:1935-1948) is the correct insurance. The contract test `crates/contracts/tests/e38_anti_pattern_contracts.rs::no_hardcoded_flash_attention_2_assumption` enforces "never hard-code FA-2 anywhere" as a regression guard.

Source: tridao.me/publications/flash3/flash3.pdf (FA-3 paper; verified May 2026).

(3) The "1M-token-lie": Claude Opus 4.7 advertises a 1M-token context window but scores 32.2% MRCR v2 at full length; Llama 4 Scout 10M scores 15.6%. RAG-first stays correct for ELAI through 2027. The mistake_book / cosmos_worker / RAG layers MUST NOT collapse into "just put it in context" even if context budgets grow. The MRCR-unverified producer at `crates/orchestrator/src/roadmap_signals.rs::build_mrcr_unverified_payload` is the runtime guard: any backend declaring >= 100K context tokens without a measured `mrcr_v2_score` emits a `mrcr_unverified` SSB row at init.

Sources: medium.com/@administrator_54541/the-1-million-token-lie (verified May 2026); digitalapplied.com/blog/ai-context-window-comparison-2026-1m-to-10m-tokens (verified May 2026); arXiv:2409.12640 (MRCR v2 multi-needle long-context benchmark; arxiv.org/abs/2409.12640 verified May 2026).

(4) MCP is now a Linux Foundation standard (donated Dec 2025). 2026 roadmap: stateless transport, Tasks primitive, MCP Registry Q4 2026, SAML/OIDC/PKCE Q2 2026. ELAI is on the right protocol; the contract test `crates/contracts/tests/e38_anti_pattern_contracts.rs::no_bespoke_agent_protocol_parallel_to_mcp` enforces "no LangGraph/CrewAI/AutoGen/AG2 parallel protocols". The `crates/orchestrator/src/dispatch/mcp.rs::McpClientDispatch::new` emits `mcp_transport_version_logged` SSB rows at process boot so drift between current transport (Streamable HTTP per spec 2025-11-25) and the roadmap target is observable in the `RoadmapPanel` dashboard tile.

Source: modelcontextprotocol.io/development/roadmap (verified May 2026).

(5) Regulatory: EU AI Act high-risk Annex III obligations POSTPONED from Aug 2 2026 to Dec 2 2027 per Digital Omnibus May 7 2026. GPAI obligations live Aug 2 2025; transparency Aug 2 2026. US Dec 2025 EO federal-preempts state AI laws. China chip exports case-by-case Jan 2026 with 25% tariff. Determinism + audit-trail design (rules-as-data + FSM + evidence packs) is the right hedge for both regimes -- the EU AI Act compliance pre-gate at `crates/orchestrator/src/compliance.rs::pre_gate_planning_to_testing_first` fires before every PLANNING/FEASIBILITY_PROBE -> TESTING_FIRST transition and asserts art. 12 (record-keeping) + art. 13 (transparency) preconditions.

Sources: artificialintelligenceact.eu/implementation-timeline (verified May 2026); paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws (verified May 2026); bisi.org.uk/reports/trump-reverses-us-ai-chip-export-policy-to-china (verified May 2026).


SECTION 2 — Seven ELAI bets that could go wrong (with hedges)

Bet 1 -- Dense-only Qwen2.5-Coder on the local tier. Risk: LOW for 7-12 GB VRAM today; MEDIUM if MoE becomes the only frontier path 2027+. Hedge: B-37 RuntimeAdapter (SHIPPED) is arch-blind; `docs/strategy/MoE-on-Ampere-tier.md` documents the expert-offload path for a future B-N/E-N.

Bet 2 -- FA-2 on Ampere. Risk: LOW; Ampere is on FA-2 indefinitely. Hedge: `ELAI_FORCE_SDPA=1` env-var keeps the SDPA fallback live; contract test bans hard FA-2 imports outside try/except.

Bet 3 -- RAG-first through 2027 (vs "1M effective context just works"). Risk: LOW under current MRCR v2 numbers (Opus 4.7 32.2% at 1M; Scout 15.6% at 10M). Hedge: `mrcr_unverified` SSB row at backend init.

Bet 4 -- MCP over bespoke protocols. Risk: LOW; LF standard + 2026 roadmap is well-publicized. Hedge: contract test bans LangGraph/CrewAI/AutoGen/AG2 parallel protocols; `mcp_transport_version_logged` row makes drift observable.

Bet 5 -- Sidecar-on-single-3090 as canonical. Risk: MEDIUM under AWS Trainium3+Cerebras disaggregated inference (Q3 2026). Hedge: B-37 RuntimeAdapter cloud backends (Sonnet 4.6 PROVEN per `memory:project_b37_shipped`); second 3090 incoming per `memory:project_vram_tier_plan`.

Bet 6 -- Determinism-first + rules-as-data. Risk: LOW; under EU AI Act high-risk this becomes a competitive REQUIREMENT, not just a hedge. The EU AI Act compliance pre-gate (E-38) operationalizes this bet.

Bet 7 -- No-MoE-local stance. Risk: MEDIUM-HIGH if 2x3090 = 48 GB makes Mixtral 8x22B or DeepSeek V4-Flash viable. Hedge: `docs/strategy/MoE-on-Ampere-tier.md` keeps the design space documented; Task 6 of E-38 plan is the placeholder for the future B-N/E-N that ships an actual expert-offload backend.


SECTION 3 — Per-lab signal summary (2026 snapshot)

Anthropic -- Claude Opus 4.7 (Apr 16 2026): SWE-bench Verified 87.6%; Pro 64.3%. MRCR v2 at 1M = 32.2%. Sonnet 4.6 OSWorld 72.5% per workos.com 2026 comparison; Sonnet 4.6 used as ELAI's cloud baseline per B-37 SHIPPED. Source: anthropic.com/news/claude-opus-4-7 (verified May 2026).

OpenAI -- GPT-5.5 trails Opus on MRCR v2 (74% vs 93% on the 8-needle variant per llm-stats.com). Operator shut down Aug 31 2025 (workos.com comparison + paper-hastings policy alert); the contract test `crates/contracts/tests/e38_anti_pattern_contracts.rs::no_operator_screenshot_polling_loop` bans the screenshot-loop pattern.

Google DeepMind -- Gemini 3.1 + Flash-Lite. Flash-Lite scores 60.1% MRCR v2 (llm-stats.com verified May 2026). DeepMind 1M context window claim still leads MRCR v2 leaderboards on the 128K+ band.

Meta -- Llama 4 Scout (10M context, 15.6% MRCR v2 per the "1-million-token-lie" analysis). Llama 4 is MoE.

xAI -- Grok 4 generation; agentic competition lane.

DeepSeek -- V4-Pro 1.6T params, 1M ctx; trails GPT-5.4/Gemini 3.1 by 3-6 months. V4-Flash distilled is a candidate ELAI cloud-mode option pending license + EU/US legal posture verification. Source: cnbc.com/2026/04/24/deepseek-v4-llm-preview-open-source-ai-competition-china (verified May 2026).

Mistral -- Devstral 2 123B; SWE-bench Verified 72.2%. Candidate for 2x3090 NF4 evaluation pending second-3090 arrival. Source: venturebeat.com/ai/mistral-launches-powerful-devstral-2-coding-model-including-open-source (verified May 2026).


SECTION 4 — Hardware roadmap (2026 snapshot)

Nvidia -- Rubin Q2 2026; Rubin Ultra 2027. Source: tomshardware.com/pc-components/gpus/nvidia-announces-rubin-gpus (verified May 2026).
AMD -- MI355X / MI450 Helios 2H 2026. Source: videocardz.com/newz/amd-launches-instinct-mi350-series (verified May 2026).
AWS -- Trainium3 + Cerebras WSE-3 disaggregated inference Q3 2026. Source: cerebras.ai/press-release/awscollaboration (verified May 2026).
Apple -- M5 Pro / Max MBP Mar 11 2026; Mac Studio M5 Ultra delayed to Oct 2026+. Source: machinelearning.apple.com/research/exploring-llms-mlx-m5 (verified May 2026).
ELAI user hardware: 1x RTX 3090 24 GB (today); 2x RTX 3090 incoming per `memory:project_vram_tier_plan`. The VRAM tier plan ships per-tier configs at 8/12/16/24/48 GB; the 2x3090 tier opens the door to Mixtral 8x22B / DeepSeek V4-Flash / Devstral 2 NF4.


SECTION 5 — Regulatory landscape

EU AI Act -- GPAI obligations live Aug 2 2025. Transparency obligations Aug 2 2026. High-risk Annex III obligations POSTPONED Aug 2 2026 -> Dec 2 2027 per Digital Omnibus May 7 2026. E-38 compliance pre-gate operationalizes art. 12 (record-keeping) + art. 13 (transparency). Source: artificialintelligenceact.eu/implementation-timeline (verified May 2026).

US -- Dec 2025 executive order federal-preempts state AI laws. Source: paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws (verified May 2026).

China -- chip exports case-by-case Jan 13 2026 + 25% tariff. H200/MI325X exports under case-by-case review. Source: bisi.org.uk/reports/trump-reverses-us-ai-chip-export-policy-to-china (verified May 2026).

EU Code of Practice (Aug 2 2025+) -- supplements GPAI transparency. Verify whether the Code imposes anything beyond transparency on ELAI-class self-hosted-sidecar use cases by reading Commission GPAI guidelines (digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers) BEFORE declaring the compliance gate complete.


SECTION 6 — ELAI's architectural insurance

(a) B-37 RuntimeAdapter (SHIPPED commit fd5c2e5) -- arch-blind trait with 5 methods (backend / capabilities / health_check / complete / stream); 11 backends; PerRoleRuntime + RuntimeOrchestrator + RuntimePanel. Hedge for Bets 1, 2, 5, 7. Contract test `e38_anti_pattern_contracts.rs::runtime_adapter_trait_stays_arch_agnostic` enforces the arch-blind invariant.

(b) MCP bridge (`crates/plugins/src/mcp_bridge.rs::DefaultMcpBridge`) -- canonical agent protocol. Hedge for Bet 4. `mcp_transport_version_logged` SSB beacon at boot.

(c) FSM + rules-as-data (`crates/orchestrator/src/fsm.rs` + `crates/rules/`) -- deterministic transitions + evidence packs at every phase boundary. Hedge for Bet 6 + EU AI Act compliance.

(d) Spec-grounded (paper-ingestor / source-grounding rules in CLAUDE.md) -- every external claim has a citation. The grounding sweep step in CLAUDE.md `Plan improvement (MANDATORY PROTOCOL)` enforces this at plan-time.

(e) RAG + cosmos_worker (B-42 SHIPPED per `memory:project_b42_shipped`) -- layered memory keeps RAG-first viable as context budgets grow. Hedge for Bet 3.


SECTION 7 — Refresh discipline

This document MUST be re-validated every 90 days. The `posture_doc_stale` SSB row at `crates/orchestrator/src/roadmap_signals.rs::check_posture_doc_age` fires when the mtime crosses the 90-day window; the `RoadmapPanel` dashboard tile surfaces the warning. The 90-day window is aggressive but appropriate given 2026 pace; relax to 180 days only if maintenance proves heavy AND log every relaxation in `workflow/reasoning/E-38-decisions.md`.

When you refresh: re-run WebSearch on every Source: line above. Flag any 404s, any moved URLs, any superseded papers. Update the per-lab snapshot in Section 3 + the hardware roadmap in Section 4. Re-author Section 1 only when one of the 5 top-level signals has materially shifted (e.g. MoE becomes the only frontier path, or EU AI Act Annex III is re-timetabled).


END ELAI-2026-POSTURE.md
