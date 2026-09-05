# E-38: 2026 AI roadmap signals — strategic-posture spec + EU AI Act compliance gate + MoE/FA-3/effective-context hedges
Objective: Encode the 2026 AI ecosystem signals into ELAI's strategic posture and runtime invariants. Add a compliance pre-gate at PLANNING → TESTING_FIRST transition that asserts EU AI Act determinism + audit-trail requirements are intact. Mark dense-only and FA-2-only assumptions as "current-tier, hedged" with documented escape hatches via the B-37 RuntimeAdapter. Lock in RAG-first-through-2027 framing against the "1M-token-lie" so layered memory (mistake_book / cosmos_worker / RAG) stays load-bearing. This E-N is STRATEGIC + lightweight wiring — most weight is in docs + a small set of compliance assertions.
Estimated effort: 1-2 sessions, ~6-8 h (smallest of this assigned set)
Pre-requisites: B-37 SHIPPED (PerRoleRuntime + RuntimeAdapter — the architectural hedge; per memory:project_b37_shipped commit fd5c2e5), D-1..D-7 PLANNED (testing-trust + determinism infrastructure for EU AI Act compliance gate)
Parallel-safe with: all other E-Ns (E-38 is mostly documentation + a compliance assertion; no overlapping symbol surface), E-36 (different concern), E-37 (different concern), E-39 (different concern but referenced in cloud cost framing), E-40 (different concern)
Source report: workflow/improvement-sweep-2026-05-19/reports/38-ai-roadmap-2026-signals.md

## Context

The report is strategic, not implementation-heavy. Five top-level signals shape ELAI's 2026-2027 posture:

(1) MoE is the de-facto frontier architecture across Mixtral, Gemini 3, GPT-5.5, DeepSeek V4, Qwen 3.6, Llama 4. ELAI's dense-only Qwen2.5-Coder bet is fine for the 7-12 GB VRAM tier — but ELAI MUST NOT hard-code dense assumptions into runtime/adapter layers. B-37 RuntimeAdapter (SHIPPED, commit fd5c2e5, 11 backends, PerRoleRuntime, RuntimeOrchestrator) is the right hedge; this E-N ensures MoE-on-Ampere (expert offload to CPU/disk) is a documented next-tier deliverable in the VRAM-tier plan (memory:project_vram_tier_plan). Source: callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms.

(2) FlashAttention-2 is legacy; FA-3 is Hopper-only (H100/H200); FA-4 targets Hopper+Blackwell via CuTeDSL. Ampere (RTX 3090, the user's actual hardware per memory:project_qwen3b_baseline) stays on FA-2 indefinitely and loses FA-3/4 performance gains. ELAI's `ELAI_FORCE_SDPA=1` fallback (per memory:project_qwen3b_baseline) is correct insurance. This E-N codifies "never hard-code FA-2 anywhere" as a runtime invariant. Source: tridao.me/publications/flash3/flash3.pdf.

(3) The "1M-token-lie": Claude Opus 4.7 advertises 1M tokens but scores 32.2% MRCR v2 at full length; Llama 4 Scout 10M scores 15.6%. RAG-first stays correct for ELAI through 2027. The mistake_book / cosmos_worker / RAG layers MUST NOT collapse into "just put it in context" even if context budgets grow. Source: medium.com/@administrator_54541/the-1-million-token-lie + digitalapplied.com/blog/ai-context-window-comparison-2026-1m-to-10m-tokens.

(4) MCP is now a Linux Foundation standard (donated Dec 2025). 2026 roadmap: stateless transport, Tasks primitive, MCP Registry Q4, SAML/OIDC/PKCE Q2. ELAI is on the right protocol — track the transport-v2 change carefully (composes with E-24 Streamable HTTP migration). Source: modelcontextprotocol.io/development/roadmap.

(5) Regulatory: EU AI Act high-risk Annex III obligations POSTPONED from Aug 2 2026 to Dec 2 2027 per Digital Omnibus May 7 2026 (artificialintelligenceact.eu/implementation-timeline). GPAI obligations live Aug 2 2025; transparency Aug 2 2026. US Dec 2025 EO federal-preempts state AI laws. China export controls case-by-case Jan 2026. Determinism + audit-trail design (rules-as-data + FSM + evidence packs) is the right hedge for both regimes — encoded as a compliance gate at PLANNING → TESTING_FIRST in this E-N.

The strategic implication for ELAI is what the report's "ELAI bets that could go wrong" section names directly: 7 named bets, of which Bet 6 (determinism-first + rules-as-data) is the strongest (LOW likelihood of losing — under EU AI Act high-risk this becomes a competitive REQUIREMENT, not just a hedge). Bet 1 (dense-only Qwen) and Bet 2 (FA-2 on Ampere) are HEDGED via B-37 RuntimeAdapter. Bet 5 (sidecar-on-single-3090 as canonical) is MEDIUM risk under AWS Trainium3+Cerebras disaggregated inference (Q3 2026) — also hedged via B-37 (cloud backends).

The E-N is structured as: (a) a documented Strategic Posture Spec at `docs/strategy/ELAI-2026-POSTURE.md` (~400 lines, the report's signals translated into ELAI-specific assertions); (b) 4 runtime invariants enforced via small contract tests; (c) a PLANNING → TESTING_FIRST compliance pre-gate that asserts EU AI Act art. 12-13 audit-trail prereqs are intact (determinism: ClockProvider injection per C-15, deterministic seeds, evidence packs from FSM transitions, replay-from-SSB capability).

## Existing state (B/C/D coverage)

- B-37 SHIPPED (Multi-Runtime Backend Abstraction, commit fd5c2e5; 11 backends + RuntimeAdapter + PerRoleRuntime + RuntimeOrchestrator + RuntimePanel; per memory:project_b37_shipped): the architectural hedge. NO CHANGE. E-38 documents B-37 as the hedge against Bets 1, 2, 5, 7.
- D-1..D-7 PLANNED (testing-trust substrate: ReasoningTemplate schema + Blake3 hashing + archive lifecycle + cross-domain canary): per D-SUMMARY_INDEX. EXTENSION: E-38's compliance pre-gate USES the D-1..D-7 surface (template archive, content_hash chain, archive verify). Without D-1..D-7 the compliance gate has no substrate to assert — sequencing matters.
- C-15 PLANNED (ClockProvider injection): per C-SUMMARY_INDEX line 194 + memory cite. EXTENSION: compliance pre-gate asserts ClockProvider injection is intact at the orchestrator boundary; bare SystemTime::now / Instant::now usage = compliance failure.
- C-27 PLANNED (T-1 determinism gate): per C-SUMMARY_INDEX. EXTENSION: E-38 compliance gate composes with C-27's determinism floor.
- B-27 PLANNED (Cloud Mode — D-1..D-4): per B-PLAN-SUMMARY line 523. RELATED: cloud-mode evidence packs are part of the audit-trail surface E-38 asserts.
- B-29 PLANNED (Symbol Impact API): per B-PLAN-SUMMARY line 571. ORTHOGONAL — referenced only because the compliance pre-gate runs after impact_radius computation in the PLANNING phase.
- Existing FSM transitions (PLANNING, TESTING_FIRST, EXECUTING, AUDITING, MEMORIZING, COMPLETE per CLAUDE.md): E-38 adds the pre-gate at the PLANNING → TESTING_FIRST edge. No new FSM states.
- `ELAI_FORCE_SDPA=1` (per memory:project_qwen3b_baseline) — already in place as the FA-2/FA-3 hedge. NO CHANGE. E-38 codifies "no hard-coded FA-2" as a contract test.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\38-ai-roadmap-2026-signals.md` (full — 127 lines)
- `F:\.ELAI_workflow\CLAUDE.md` (anti-stale-training section; FSM gate-check section; remote sidecar block)
- `F:\.ELAI_workflow\docs\thefinalglance\B-plan\B-PLAN-SUMMARY.md` lines 762-815 (B-37 SHIPPED detail)
- `F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-SUMMARY_INDEX.md` lines 22-66 (D-1..D-7 testing-trust substrate)
- `F:\.ELAI_workflow\docs\thefinalglance\C-plan\C-SUMMARY_INDEX.md` lines 194-200 (C-15 ClockProvider injection)
- artificialintelligenceact.eu/implementation-timeline — WebFetch (EU AI Act implementation timeline + Digital Omnibus May 7 2026 postponement)
- paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws — WebFetch (US Dec 2025 EO)
- modelcontextprotocol.io/development/roadmap — WebFetch (MCP 2026 roadmap)
- tridao.me/publications/flash3/flash3.pdf — WebFetch (FA-3 Hopper-only + FA-4 Hopper+Blackwell)
- medium.com/@administrator_54541/the-1-million-token-lie — WebFetch (MRCR v2 numbers)
- callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms — WebFetch
- digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers — WebFetch (Commission GPAI guidelines)

## Tasks

1. Author `docs/strategy/ELAI-2026-POSTURE.md` (~350 lines). Sections: (a) the 5 top-level signals (MoE / FA-3 / 1M-token-lie / MCP / regulatory) with citations; (b) the 7 bets-that-could-go-wrong from the report mapped to ELAI's existing hedges; (c) per-lab signal summary (Anthropic / OpenAI / Google DeepMind / Meta / xAI / DeepSeek / Mistral); (d) hardware roadmap (Nvidia Rubin Q2 2026 / Rubin Ultra 2027; AMD MI355X / MI450 2H 2026; AWS Trainium3+Cerebras Q3 2026; Apple M5 Pro/Max Mar 2026; Mac Studio M5 Ultra delayed to Oct 2026+); (e) regulatory landscape (EU AI Act Annex III postponed to Dec 2 2027; US Dec 2025 EO federal preempts state; China chip exports case-by-case Jan 2026); (f) ELAI's architectural insurance — RuntimeAdapter / MCP / FSM+rules-as-data / spec-grounded / RAG+cosmos_worker. The document is the spec the auditor consults when an architectural decision invokes "the 2026 ecosystem says X." The doc is grounded — every claim has a citation; no ungrounded conjecture.

2. EU AI Act compliance pre-gate at PLANNING → TESTING_FIRST. New module `crates/safety/src/eu_ai_act_compliance.rs` (~180 LOC) or co-located with E-37's safety crate if that lands first; otherwise standalone module under `crates/orchestrator/src/compliance.rs`. Pre-gate runs before any FSM transition from PLANNING to TESTING_FIRST and asserts: (a) ClockProvider injection intact at orchestrator boundary (no bare `SystemTime::now` / `Instant::now` in the planning crate per C-15); (b) deterministic-seed propagation present in the test runner (per C-27 T-1 determinism); (c) D-1 template content_hash chain verifiable for the current plan's reasoning templates; (d) SSB row count emitted in PLANNING phase ≥ minimum threshold (audit-trail liveness check). On failure, BLOCK transition + emit `eu_ai_act_compliance_blocked` SSB row with the failing assertion + remediation hint. Reference: artificialintelligenceact.eu/implementation-timeline + EU AI Act art. 12 (record-keeping) + art. 13 (transparency).

3. Runtime invariant — "no hard-coded FA-2." Contract test `crates/contracts/tests/no_hardcoded_flash_attention_2_contract.rs` (~60 LOC). Grep the Python sidecar source tree at `python/elai_inference/` for `import flash_attn_2` / `flash_attention_2 ==` / `assert.*flash_attn` patterns. Fail if any hard reference exists outside a try/except fallback. The test runs in CI on every PR touching `python/elai_inference/`. Reference: tridao.me/publications/flash3/flash3.pdf — FA-2 is legacy 2027+; ELAI_FORCE_SDPA=1 must remain a viable fallback.

4. Runtime invariant — "no advertised-context-without-MRCR-verification." Contract test `crates/contracts/tests/context_advertised_vs_effective_contract.rs` (~80 LOC). When `[runtime] declared_max_context_tokens` is set in `elai_cc.toml` ≥ 100K for any backend, the test requires a matching `[runtime] mrcr_v2_score` field with a measured value ≥ 0.50 (or the backend declares `[runtime] mrcr_v2_score = "vendor-claim-only"` which logs a `mrcr_unverified` SSB row at backend init). Closes Bet 3 (RAG-first / 1M-token-lie) — ELAI does not architect a backend for "10M effective context" without measurement. Reference: medium.com/@administrator_54541/the-1-million-token-lie + arxiv MRCR v2 paper (look up canonical arXiv ID via WebSearch — report does not give one; do not fabricate).

5. Runtime invariant — "no MoE-or-dense lock-in in adapter layers." Contract test `crates/contracts/tests/runtime_adapter_arch_agnostic_contract.rs` (~70 LOC). Asserts that the `RuntimeAdapter` trait at `crates/orchestrator/src/runtime_orchestrator.rs` (B-37 shipped) has no methods that name an architecture (no `is_dense()` / `dense_only_path()` / `requires_dense_kv_cache()` / etc.). The trait stays architecture-blind. Add a comment to the trait pointing to E-38 / this rule. Reference: callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms + B-37 SHIPPED architecture.

6. Documented MoE-on-Ampere VRAM-tier extension. Add `docs/strategy/MoE-on-Ampere-tier.md` (~150 LOC) describing the expert-offload-to-CPU/disk path for MoE models (Mixtral 8x22B, DeepSeek V4-Flash distilled, future Qwen-MoE) on the 12-24 GB Ampere tier. Cite published expert-offload approaches — VERIFY arxiv IDs via WebSearch before citing; do not fabricate. The doc is preparation for a future E-N or B-N that ships the actual expert-offload backend; this E-N just documents the design space. Coordinate with memory:project_vram_tier_plan.

7. MCP transport-v2 watch hook. Add a TODO marker in `crates/plugins/src/mcp_bridge.rs` referencing the MCP roadmap (modelcontextprotocol.io/development/roadmap) and the planned stateless transport + Tasks primitive + MCP Registry Q4 2026 + SAML/OIDC/PKCE Q2. The marker is a no-op runtime — it's a documentation pointer for the next E-N that addresses transport-v2 migration. Composes with E-24 Task 4 (Streamable HTTP migration). Emit `mcp_transport_version_logged` SSB row at bridge init with the current transport version + the roadmap target version, so logs show drift.

8. Anti-pattern guard tests (3 contract tests). Each codifies one of the report's anti-patterns: (a) `crates/contracts/tests/no_bespoke_agent_protocol_parallel_to_mcp_contract.rs` — assert no protocol code in `crates/plugins/` other than MCP and Anthropic protocol; grep for keywords like `LangGraph`, `CrewAI`, `AutoGen` and fail if any non-comment usage. (b) `crates/contracts/tests/no_operator_screenshot_loop_pattern_contract.rs` — assert no agent code constructs a screenshot-loop computer-use surface (per OpenAI Operator shutdown Aug 2025); grep for screenshot + click + agent-loop patterns. (c) `crates/contracts/tests/audit_trail_liveness_contract.rs` — assert SSB writes happen on every FSM transition; the FSM transition handler MUST emit an SSB row or the test fails (EU AI Act art. 12 record-keeping requirement).

9. Strategic-posture cross-reference in `CLAUDE.md`. Add ONE line under "Hard rules" pointing to `docs/strategy/ELAI-2026-POSTURE.md` as the canonical reference for architectural-direction decisions. Existing CLAUDE.md HARD RULE: "From training (unsearched):" prefix discipline — extend the rule's example list to include "any claim about a 2026 lab roadmap or AI Act timeline MUST cite the posture doc or WebFetch directly; never from training memory."

10. Strategic-posture review checkpoint at session start. Add a one-line note to `.claude/rules/session-protocol.md` (if it exists; verify via Glob) that says "at session start, check `docs/strategy/ELAI-2026-POSTURE.md` last-modified date. If >90 days old, surface a `posture_doc_stale` warning to the user and recommend a refresh pass." This is enforcement-by-reminder, not a hook — keeps the posture doc from becoming stale and ungrounded.

## WIRING_CONTRACT

produces_symbols:
  - orchestrator::compliance::pre_gate_planning_to_testing_first(plan_ctx: &PlanContext) -> ComplianceVerdict (new)
  - orchestrator::compliance::ComplianceVerdict { Pass, Block(assertion: ComplianceAssertion, hint: String) } (new)
  - orchestrator::compliance::ComplianceAssertion { ClockProviderIntact, DeterministicSeed, TemplateHashChain, AuditTrailLive } (new)
  - SsbActionType::EuAiActComplianceBlocked (new)
  - SsbActionType::EuAiActComplianceVerified (new)
  - SsbActionType::McpTransportVersionLogged (new)
  - SsbActionType::MrcrUnverified (new)
  - SsbActionType::PostureDocStale (new — warns at session start when doc >90 days old)
consumes_symbols:
  - orchestrator::runtime_orchestrator::RuntimeAdapter (B-37 SHIPPED; the trait stays architecture-blind per Task 5)
  - orchestrator::runtime_orchestrator::PerRoleRuntime (B-37 SHIPPED)
  - core::clock::ClockProvider (C-15 PLANNED; compliance gate asserts presence)
  - reasoning::template::TemplateArchive (D-1 PLANNED; compliance gate asserts content_hash chain)
  - orchestrator::ssb::SsbActionType (FSM transition emission audit-trail check)
  - plugins::mcp_bridge::McpBridge (transport-version watch hook)
reachable_from:
  - crates/orchestrator/src/fsm.rs (PLANNING → TESTING_FIRST transition handler invokes compliance pre-gate)
  - crates/plugins/src/mcp_bridge.rs::initialize (transport-version log emission)
  - CI workflows (.elai_cc/.github/workflows/) — the contract tests above run on every PR
stub_ban:
  - no .unwrap() in compliance pre-gate path (CLAUDE.md HARD RULE)
  - no hard-coded `flash_attn_2` import outside try/except fallback (Task 3 invariant)
  - no `is_dense_arch()` / `requires_dense` method on RuntimeAdapter trait (Task 5 invariant)
  - no `declared_max_context_tokens` ≥ 100K without `mrcr_v2_score` field (Task 4 invariant)
  - no bespoke agent protocol parallel to MCP (Task 8a guard)
  - no screenshot-loop computer-use surface (Task 8b guard)
  - no FSM transition without SSB row (Task 8c guard)
  - no posture-doc claim without citation (Task 1 discipline)
frontend_pair: null (this E-N has no dashboard surface — strategic doc + compliance gate + contract tests only)
observable_effect:
  trigger: attempt FSM transition PLANNING → TESTING_FIRST with ClockProvider missing (synthetic fixture)
  expect_ssb: "eu_ai_act_compliance_blocked with assertion=ClockProviderIntact + remediation hint"
  expect_artifact: "transition refused; FSM stays in PLANNING; user-facing error references docs/strategy/ELAI-2026-POSTURE.md compliance section"

## DETERMINISTIC_CHECK

```bash
# 1. Strategic posture doc exists + recent
test -f docs/strategy/ELAI-2026-POSTURE.md
test -f docs/strategy/MoE-on-Ampere-tier.md
# Expected: both exist
# Recent: last-modified within 90 days (posture_doc_stale gate)

# 2. EU AI Act compliance pre-gate present
rg "pre_gate_planning_to_testing_first|EuAiActComplianceBlocked" .elai_cc/crates --type rust
# Expected: ≥2 hits (impl + dispatch in fsm.rs)

# 3. No hard-coded FA-2 in Python sidecar
rg "import flash_attn_2|flash_attention_2 ==" .elai_cc/python/elai_inference/ -g '!*test*' -g '!*fallback*'
# Expected: zero hits OR all hits inside try/except (manual review)

# 4. RuntimeAdapter trait is architecture-blind
rg "is_dense|requires_dense|dense_only|moe_only" .elai_cc/crates/orchestrator/src/runtime_orchestrator.rs
# Expected: zero hits

# 5. Advertised context ≥ 100K requires MRCR score field
rg "declared_max_context_tokens.*\d{6,}" .elai_cc/elai_cc.toml
# Manual verify: any backend with ≥ 100K declared has matching mrcr_v2_score field

# 6. 5 new SSB action variants
rg "EuAiActComplianceBlocked|EuAiActComplianceVerified|McpTransportVersionLogged|MrcrUnverified|PostureDocStale" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 5 hits

# 7. Three anti-pattern contract tests exist
test -f .elai_cc/crates/contracts/tests/no_bespoke_agent_protocol_parallel_to_mcp_contract.rs
test -f .elai_cc/crates/contracts/tests/no_operator_screenshot_loop_pattern_contract.rs
test -f .elai_cc/crates/contracts/tests/audit_trail_liveness_contract.rs
# Expected: all 3 exist

# 8. FSM transitions emit SSB rows (audit-trail liveness gate)
cargo test -p contracts --test audit_trail_liveness_contract
# Expected: PASS

# 9. CLAUDE.md references posture doc
rg "ELAI-2026-POSTURE.md" .elai_cc/CLAUDE.md F:/.ELAI_workflow/CLAUDE.md
# Expected: ≥1 hit
```

## ACCEPTANCE

- [ ] `docs/strategy/ELAI-2026-POSTURE.md` authored with ~350 lines covering 5 signals + 7 bets + per-lab + hardware + regulatory + insurance sections; every claim cited
- [ ] `docs/strategy/MoE-on-Ampere-tier.md` authored with ~150 lines documenting expert-offload design space
- [ ] EU AI Act compliance pre-gate runs at PLANNING → TESTING_FIRST; blocks on any of 4 failing assertions with hint; SSB row emitted
- [ ] Contract test `no_hardcoded_flash_attention_2_contract` passes — no hard FA-2 imports outside fallback
- [ ] Contract test `context_advertised_vs_effective_contract` passes — any backend ≥100K declared has matching MRCR score field
- [ ] Contract test `runtime_adapter_arch_agnostic_contract` passes — RuntimeAdapter trait has no architecture-named methods
- [ ] 3 anti-pattern contract tests pass — no parallel agent protocol, no screenshot-loop, audit-trail emission per FSM transition
- [ ] MCP transport-version log emitted at bridge init showing current vs roadmap-target
- [ ] CLAUDE.md hard rule added referencing posture doc as canonical reference for 2026-roadmap claims
- [ ] `.claude/rules/session-protocol.md` posture-doc-stale check added (>90 days warning)
- [ ] All 5 new SSB action variants present in ssb.rs

## Tests to add

- `crates/contracts/tests/eu_ai_act_compliance_pre_gate_contract.rs`: 4 failing fixtures × 1 passing fixture; assert correct ComplianceAssertion variant fires
- `crates/contracts/tests/no_hardcoded_flash_attention_2_contract.rs`: grep Python sidecar; pass if no bare imports outside try/except
- `crates/contracts/tests/context_advertised_vs_effective_contract.rs`: toml fixtures × 4 (valid declare+MRCR / declare-only fail / vendor-claim-only logs row / no declare fine)
- `crates/contracts/tests/runtime_adapter_arch_agnostic_contract.rs`: parse `crates/orchestrator/src/runtime_orchestrator.rs` AST; assert no method names contain `dense` / `moe` / arch-keyword set
- `crates/contracts/tests/no_bespoke_agent_protocol_parallel_to_mcp_contract.rs`: grep `crates/plugins/`; assert no LangGraph / CrewAI / AutoGen / AG2 references
- `crates/contracts/tests/no_operator_screenshot_loop_pattern_contract.rs`: grep agent code; assert no `screenshot + click + loop` pattern
- `crates/contracts/tests/audit_trail_liveness_contract.rs`: drive 5 FSM transitions; assert SSB row emitted on each (EU AI Act art. 12)
- `crates/contracts/tests/mcp_transport_version_logged_contract.rs`: bridge init emits row with current + roadmap version
- `crates/contracts/tests/posture_doc_stale_warning_contract.rs`: synthetic posture doc with mtime > 90 days → `posture_doc_stale` row fires at session start

## Files touched

- CREATE: `docs/strategy/ELAI-2026-POSTURE.md` (~350 LOC)
- CREATE: `docs/strategy/MoE-on-Ampere-tier.md` (~150 LOC)
- CREATE: `crates/orchestrator/src/compliance.rs` (~180 LOC; SHIPPED — canonical compliance pre-gate lives at orchestrator/compliance not the standalone safety crate)
- MODIFY: `crates/orchestrator/src/fsm.rs` (+~30 LOC for PLANNING → TESTING_FIRST hook call)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (+~10 LOC comment block referencing E-38 / arch-blind invariant)
- MODIFY: `crates/plugins/src/mcp_bridge.rs` (+~20 LOC for transport-version log at init)
- MODIFY: `crates/orchestrator/src/ssb.rs` (+5 SsbActionType variants)
- MODIFY: `elai_cc.toml` schema docs (`[runtime] declared_max_context_tokens`, `[runtime] mrcr_v2_score`)
- MODIFY: `CLAUDE.md` (+1 line under "Hard rules" referencing posture doc)
- MODIFY: `.claude/rules/session-protocol.md` (+2 lines for posture-doc-stale check)
- CREATE: `crates/contracts/tests/eu_ai_act_compliance_pre_gate_contract.rs`
- CREATE: `crates/contracts/tests/no_hardcoded_flash_attention_2_contract.rs`
- CREATE: `crates/contracts/tests/context_advertised_vs_effective_contract.rs`
- CREATE: `crates/contracts/tests/runtime_adapter_arch_agnostic_contract.rs`
- CREATE: `crates/contracts/tests/no_bespoke_agent_protocol_parallel_to_mcp_contract.rs`
- CREATE: `crates/contracts/tests/no_operator_screenshot_loop_pattern_contract.rs`
- CREATE: `crates/contracts/tests/audit_trail_liveness_contract.rs`
- CREATE: `crates/contracts/tests/mcp_transport_version_logged_contract.rs`
- CREATE: `crates/contracts/tests/posture_doc_stale_warning_contract.rs`
- CREATE: `workflow/reasoning/E-38-decisions.md`

## Citations / Grounding

- anthropic.com/news/claude-opus-4-7 (cited in source) — Opus 4.7 Apr 16 2026; SWE-bench Verified 87.6%; Pro 64.3%
- artificialintelligenceact.eu/implementation-timeline (cited in source) — EU AI Act GPAI Aug 2 2025; transparency Aug 2 2026; high-risk POSTPONED to Dec 2 2027 per May 7 2026 Digital Omnibus
- modelcontextprotocol.io/development/roadmap (cited in source) — Linux Foundation governance; stateless transport, Tasks, MCP Registry Q4, SAML/OIDC/PKCE Q2
- tridao.me/publications/flash3/flash3.pdf (cited in source) — FA-3 Hopper-only; FA-4 Hopper+Blackwell CuTeDSL; Ampere stuck on FA-2
- callsphere.ai/blog/mixture-of-experts-architecture-why-moe-dominates-2026-llms (cited in source) — MoE dominant 2026
- medium.com/@administrator_54541/the-1-million-token-lie (cited in source) — Opus 4.7 32.2% MRCR v2 at 1M, Scout 15.6% at 10M
- digitalapplied.com/blog/ai-context-window-comparison-2026-1m-to-10m-tokens (cited in source) — long-context comparison
- paulhastings.com/insights/client-alerts/president-trump-signs-executive-order-challenging-state-ai-laws (cited in source) — US Dec 2025 EO federal preempts state AI laws
- bisi.org.uk/reports/trump-reverses-us-ai-chip-export-policy-to-china (cited in source) — H200/MI325X exports Jan 13 2026 + 25% tariff; case-by-case
- tomshardware.com/pc-components/gpus/nvidia-announces-rubin-gpus (cited in source) — Nvidia Rubin Q2 2026; Rubin Ultra 2027
- videocardz.com/newz/amd-launches-instinct-mi350-series (cited in source) — AMD MI355X / MI450 Helios 2H 2026
- cerebras.ai/press-release/awscollaboration (cited in source) — AWS Trainium3 + Cerebras WSE-3 disaggregated Q3 2026
- machinelearning.apple.com/research/exploring-llms-mlx-m5 (cited in source) — Apple M5 Pro/Max MBP Mar 11 2026; Mac Studio Oct 2026+
- cnbc.com/2026/04/24/deepseek-v4-llm-preview-open-source-ai-competition-china (cited in source) — DeepSeek V4-Pro 1.6T params 1M ctx; trails GPT-5.4/Gemini 3.1 by 3-6 months
- venturebeat.com/ai/mistral-launches-powerful-devstral-2-coding-model-including-open-source (cited in source) — Devstral 2 123B; SWE-bench Verified 72.2%
- workos.com/blog/anthropics-computer-use-versus-openais-computer-using-agent-cua (cited in source) — Claude Sonnet 4.6 OSWorld 72.5%; Operator shut Aug 31 2025
- softmaxdata.com/blog/definitive-guide-to-agentic-frameworks-in-2026-langgraph-crewai-ag2-openai-and-more (cited in source) — LangGraph passes CrewAI in stars early 2026

## Open questions / risks

- D-1..D-7 sequencing: compliance pre-gate (Task 2) asserts D-1 template hash chain. If D-1..D-7 are not yet shipped when E-38 enters EXECUTING, scope Task 2 to assert ONLY (a) ClockProvider intact + (b) audit-trail liveness; mark TemplateHashChain assertion as `Skipped(reason="D-1 not yet shipped")` and emit `eu_ai_act_compliance_partial` SSB row. Re-tighten when D-1 lands.
- MRCR v2 paper canonical arXiv ID (Task 4): the report cites "MRCR v2" without an arXiv. WebSearch before citing in the posture doc. If no canonical paper exists yet, cite the medium.com analysis + the original Anthropic MRCR blog and mark the field as "vendor-claim-only when no peer-reviewed baseline."
- Open Question 5 from report: EU Code of Practice (Aug 2 2025+) — does it impose anything beyond GPAI transparency on ELAI-class self-hosted-sidecar use cases? Read Commission GPAI guidelines (digital-strategy.ec.europa.eu/en/policies/guidelines-gpai-providers) before declaring the compliance gate complete. If the Code of Practice adds requirements, append them to the pre-gate assertions.
- Bet 5 (sidecar-on-single-3090): user has a second 3090 incoming per memory:project_vram_tier_plan. Posture doc Task 1 must NOT lock the doc into single-GPU posture; explicitly call out the 2x3090 tier + Devstral 2 NF4 viability question (report Open Question 2) as a near-term measurement.
- DeepSeek V4-Flash licensing + EU/US legal posture (report's "Implication: V4-Flash is a candidate ELAI cloud-mode option … Verify license + EU/US legal posture before wiring"): if E-N E-39 (cost economics) wires DeepSeek V4-Flash as a cloud backend, E-38 posture doc MUST surface the legal-posture question. Coordinate.
- Bet 7 (No-MoE local stance, MEDIUM-HIGH risk): the posture doc should NOT close the door on MoE-on-Ampere; Task 6 explicitly documents the expert-offload path as a future deliverable. If the user's second 3090 arrives and 2x3090 = 48 GB makes Mixtral 8x22B viable, that path becomes near-term.
- Anti-slopsquatting on any new crates added (Task 2 may need a TOML parser if not already in deps): VERIFY before adding. The compliance gate is likely implementable with existing serde_toml.
- Posture doc maintenance burden (Task 10 stale-check): 90-day refresh is aggressive but appropriate given 2026 pace. If maintenance proves heavy, relax to 180 days but log every relaxation.

## Next

E-38 is the strategic spine for the entire E-N wave. Downstream coordinations: (a) E-39 (cost economics) — DeepSeek V4-Flash + Cerebras free-tier + Groq free-tier additions must check the posture doc's per-lab signals first; (b) E-24 + E-37 (MCP hardening + prompt-injection defense) — composes with the MCP transport-v2 watch hook + EU AI Act audit-trail gate; (c) E-40 (symbol graph) — composes with the audit-trail liveness contract; (d) future E-N candidates: MoE-on-Ampere expert-offload backend (Task 6 design space); Devstral 2 NF4 on 2x3090 benchmark E-N (gated on user's second 3090); per-lab cloud-fallback wiring (Gemini 3 Flash / GPT-5.4 mini / DeepSeek V4-Flash / Cerebras-free / Groq-free as B-37 backends). Posture-doc-stale gate ensures the doc gets refreshed before any major roadmap claim shifts.
