# E-3: Seamless multi-model chat handoff
Objective: make the local-sidecar -> cloud-Sonnet -> local-sidecar swap feel invisible to the user. Ship a unified system-prompt preamble shared across every PerRoleRuntime entry, surface ROLE not MODEL in the chat UI, pre-digest rolling chat context via sleep-time compute so each model load consumes a constant-size payload, and pre-warm the next-likely runtime adapter on FSM transition. Persona-vector steering on the local Qwen is included as an optional gate.
Estimated effort: 2 sessions (~6-10 h)
Pre-requisites: B-37 (Multi-Runtime Backend Abstraction, SHIPPED — supplies `RuntimeAdapter` + `PerRoleRuntime`) + E-1 (memory_notes + CoreBlock — supplies the digest substrate). C-15 (ClockProvider stable) optional but preferred for the FSM-transition timing telemetry.
Parallel-safe with: E-2 (sidecar speed; this E-N touches role-prompt + runtime swap surface, not generate kwargs), E-4 (chat UX; coordinate CommandWindow.svelte edits — this E-N adds a ROLE badge, E-4 adds confidence chips, both in the bubble header).
Source report: workflow/improvement-sweep-2026-05-19/reports/03-model-handoff-ux.md

## Context

ELAI currently runs 11 backends via the `RuntimeAdapter` trait (B-37 SHIPPED, commit fd5c2e5): Sidecar + AnthropicApi + 7 CLI ports (ClaudeCode, CodexCli, OpenCode, Hermes, GeminiCli, KiroCli, CopilotCli) + 2 native HTTP (Ollama, Vllm). PerRoleRuntime maps each agent role to a backend (designer/auditor/researcher -> AnthropicApi; executor/memory_writer/context_compiler/prompt_engineer -> Sidecar). Live cloud Sonnet 4.6 is proven end-to-end. But the user-observed pain is that the moment the FSM transitions and a different backend gets called, voice shifts, persona resets, and the user sees the model "load and unload many times."

The report's three load-bearing findings: (1) unify voice at the SYSTEM-PROMPT layer — a single ELAI-voice preamble prepended to every role prompt is the lowest-cost, highest-leverage intervention because system prompts are the primary mechanism for persistent application behavior (Tetrate guide; arXiv:2507.03254 codified prompting); (2) treat the model swap as INVISIBLE by default and EXPOSABLE on demand — Cursor/Aider/Continue.dev expose to developers, Devin/Replit hide and pay a transparency cost (compelframework.org case study). ELAI's user is a developer, default toward exposure but as a drawer not a primary surface; (3) the "loads and unloads many times" perception is mostly CUDA-graph capture, not weight load — graph capture is tens-of-seconds to minutes, weights are 1-2s via RDMA (arXiv:2604.06664 Foundry). Pre-warming the next-likely runtime and keeping captured graphs in a small rotating pool (vLLM warm-pool pattern) is the real fix.

Cross-model KV cache reuse is explicitly OUT OF SCOPE: arXiv:2411.02820 DroidSpeak only works for variants of the same backbone (Qwen-Coder vs Claude = NO). The realistic substitute is sleep-time compute (arXiv:2504.13171): pre-digest rolling context into a dense intermediate any model can ingest, refreshed during idle turns. E-1 already lands the digest substrate (CoreBlock + memory_notes). This E-N extends that into a unified `HandoffPayload` that gets compressed via LLMLingua-style approach (arXiv:2310.05736) before being injected into a different model's context.

Persona vectors on the local Qwen (arXiv:2507.21509 Anthropic, arXiv:2602.15669 PERSONA) are inference-time activation steering for the LOCAL model only (no cloud activation access). The local half is where the largest voice variance lives, so this is a contained, optional gate.

## Existing state (B/C/D coverage)

- B-37 (SHIPPED commit fd5c2e5): RuntimeAdapter trait + 11 backends + PerRoleRuntime + RuntimeOrchestrator + RuntimePanel + 4 SSB action_types (runtime_fallback, runtime_health_changed, cli_subprocess_spawned, cli_subprocess_killed). This E-N EXTENDS B-37 with role-prompt unification and the predicted-next-role pre-warm hint.
- B-22 (SHIPPED): Autonomous mode FSM substates + AutoAdvanceEvaluator. The predicted-next-role hint reuses the substate-transition surface.
- B-39 (SHIPPED): ChatReflectPipeline; per-stage KV reset via separate /v1/messages calls. The unified system preamble is injected per-stage (TRIAGE / REASONER / AUDITOR / SYNTHESIZE) — no architectural change to the per-stage isolation.
- B-9 (SHIPPED): SessionDag handoff. This E-N's `HandoffPayload` reuses the SessionDag snapshot annotation surface to carry the compressed digest between model switches.
- E-1 (THIS PLAN): supplies `CoreBlock` (1.5K-token user-model card + active goal + style) — this E-N treats CoreBlock as the canonical anchor for the unified preamble.
- C-21 (PLANNED) / C-48 (PLANNED): MistakeBook + TrustTier substrate. Voice-coherence telemetry uses C-48 TrustTier surface to tag the unified-preamble preamble lines.
- D-? — no D-plan overlap on handoff UX. D-14 (sycophancy probe) is orthogonal but the unified preamble may include a calibration-language clause; flag for D-14 future alignment.
- This E-N is an EXTENSION of B-37 (new role-prompt builder + new SsbActionType variants) and a NET-NEW surface for the predictive pre-warm + persona-vector path.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\03-model-handoff-ux.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs` (Sidecar RuntimeAdapter impl)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\ollama.rs` (alt local backend; same trait)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\vllm.rs` (vLLM backend)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\hermes.rs:250` + `kiro_cli.rs:231` (sample of CLI-subprocess backends — touch the same fallback path)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\runtime_orchestrator.rs` (RuntimeOrchestrator + PerRoleRuntime — the role-prompt insertion point lives here)
- `F:\.ELAI_workflow\.elai_cc\crates\agents\src\` (look for `role_prompt.rs` or equivalent; if absent, create new file `role_prompt.rs` per Task 1)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\handoff.rs` (existing handoff module per git status — Task 6 extends with LLMLingua-compression pass)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\components\RuntimePanel.svelte` (existing live-runtime UI; this E-N adds a "current role" badge layer)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte` (the bubble header gets a ROLE badge)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py` (persona-vector hook — locate the decode loop + LogitsProcessor chain; reuse the steering-engine pattern already present)
- arXiv:2507.21509 Persona Vectors (Anthropic)
- arXiv:2411.02820 DroidSpeak (read to confirm KV-reuse is intentionally NOT attempted)
- arXiv:2504.13171 Sleep-time Compute (read with E-1; same paper)
- arXiv:2310.05736 LLMLingua (compression for handoff payloads)
- aider.chat/2024/09/26/architect.html (architect/editor split — used as mental model)

## Tasks

1. **Create `crates/agents/src/role_prompt.rs` (~250 LOC, new file).** Public function `build_role_prompt(role: AgentRole, voice: &VoiceAnchor, task_specific: &str) -> String`. The `VoiceAnchor` struct carries: tone, formality, forbidden-phrasings, confidence-calibration clauses, sycophancy-guardrail clause, terse-by-default clause. Each role's prompt = `<voice anchor (~80-120 tokens)> + <role-specific instructions>`. The voice anchor is sourced from a single TOML at `.elai_cc/agents/voice.toml`. Mirror Aider's architect/editor diff-format-spec sharing — same intervention, applied across all 11 RuntimeAdapter backends uniformly. Per Tetrate "system vs user prompts" + arXiv:2507.03254 codified prompting.

2. **Wire `build_role_prompt` into `crates/orchestrator/src/runtime_orchestrator.rs` (~80 LOC delta).** Every dispatch through `RuntimeOrchestrator::send` calls `build_role_prompt(self.current_role, &self.voice_anchor, &task)` BEFORE handing to the `RuntimeAdapter::stream_step`. No RuntimeAdapter change required — the unified preamble is upstream of the trait boundary. Emit a `unified_preamble_applied` SSB row per dispatch with `{role, runtime_backend, preamble_blake3}` for audit.

3. **Surface ROLE not MODEL in the chat bubble (~80 LOC in `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte`).** Each chat bubble gets a small badge: "executor running", "auditor reviewing", "researcher gathering" — based on the `current_role` SSB signal. Model name (`sidecar:qwen2.5-coder-3b` vs `cloud:sonnet-4.6`) is HIDDEN by default; click the role badge to expand a drawer that shows the underlying backend. Per report Table "How shipped products handle multi-model UX" — Cursor/Aider expose, Devin/Replit hide; ELAI's user is a developer so expose-via-drawer (not primary surface).

4. **Add predicted-next-role hint to FSM dispatcher (~150 LOC across `crates/orchestrator/src/fsm.rs` + `runtime_orchestrator.rs`).** When FSM enters state X, dispatch a `predicted_next_role` hint to RuntimeOrchestrator based on the canonical transition table (e.g. AUDITING -> usually MEMORIZING or EXECUTING). RuntimeOrchestrator uses the hint to pre-warm the predicted backend (asynchronously, non-blocking). On 12GB single-GPU = no-op (cannot hold 2 models). On 24GB+ = pre-load weights via `RuntimeAdapter::warm()` (new trait method, default no-op). Emit `runtime_prewarm_started` / `runtime_prewarm_complete` SSB rows.

5. **Add `RuntimeAdapter::warm(&self) -> impl Future<Output = Result<(), RuntimeAdapterError>>` (~100 LOC, default impl + 2 specialized impls).** Default impl: no-op + returns immediately. Sidecar impl: send a 1-token warmup request to capture the CUDA graph (per arXiv:2604.06664 Foundry — graph capture dominates cold start). AnthropicApi impl: send a 1-token request to warm the connection pool. All other backends keep default no-op for now. Per CLAUDE.md trait-default principle and ELAI's anti-stale rule on async traits (native async fn, no async-trait crate).

6. **Extend `crates/cli/src/handoff.rs` with LLMLingua-style compression (~180 LOC).** New function `compress_handoff_payload(payload: &HandoffPayload, target_tokens: usize) -> CompressedPayload`. Uses a token-frequency + cross-encoder rerank approach (existing reranker from Phase 1G via `crates/knowledge/`); does NOT require running an LLM (per CLAUDE.md "no LLM in the validation hot path" pattern). Target ratio: 5-20x per report. Emit `handoff_payload_compressed` SSB row with `{original_tokens, compressed_tokens, compression_ratio}`. Used when handing context from a big cloud model -> small local model (or vice versa) across PerRoleRuntime boundaries.

7. **Persona-vector steering hook on the LOCAL sidecar (~200 LOC in `python/elai_inference/server.py`).** Optional, env-gated via `ELAI_PERSONA_VECTOR=1`. Pre-compute an "ELAI assistant" steering vector for Qwen2.5-Coder via contrastive activations on a fixture set of (in-voice example, neutral example) pairs at boot. Apply via the existing steering-engine surface during decode (the steering processor is already present per elai.map line 22617; reuse the same processor type). Per arXiv:2507.21509 Anthropic + arXiv:2602.15669 PERSONA. NOTE: report Open Question 2 — steering vectors are model-specific; need to extract + test on the actual 3B/7B abliterated variants before claiming Anthropic's effect sizes apply.

8. **Optional style-normalizer post-pass (~120 LOC in `crates/cli/src/live_cli.rs` rendering layer).** Env-gated via `ELAI_STYLE_NORMALIZE=1`, default off. Thin pass that strips agent-specific tells ("BLOCK:" prefix on auditor outputs, JSON-mode artifacts, "Verified:" leader) before display. Only triggered when source model != voice anchor model. Per arXiv:2602.15013 (round-trip TST) + arXiv:2507.18294 (StyleAdaptedLM). Watch latency cost — add only if A/B testing shows real coherence win.

9. **Verify `enable_prefix_caching=True` on the vLLM backend (~10 LOC in `crates/providers/src/runtime/vllm.rs:189`).** This is auto-prefix-caching for the LOCAL model's own immutable system block when running vLLM; it's NOT cross-model KV reuse (which the report explicitly says is out of scope). One-line check + telemetry assertion.

10. **Author the canonical `voice.toml` (~80 lines).** Path: `.elai_cc/agents/voice.toml`. Sections: `[anchor]` (tone, formality, lowercase 'i', no em-dashes, no robotic prefixes, contractions drop apostrophes per maintainer voice rules — already in `.claude/rules/commit-cadence.md`). `[confidence_calibration]` (the "Verified:" / "High confidence:" / "Uncertain:" prefixes from CLAUDE.md). `[sycophancy_guard]` (one-line: "Hold position with citation when you have evidence; admit when you don't"). `[forbidden_phrases]` (list of robotic prefixes to suppress: `[\\\"feat:\\\", \\\"chore:\\\", \\\"Phase N:\\\"]`).

11. **Write decision doc at `workflow/reasoning/E-3-decisions.md`** covering: (a) does the unified preamble go in role_prompt.rs (new) or extend an existing module (recommend: new file, single source of truth); (b) persona-vector default state (recommend: env-gated off until Open Question 2 measurement); (c) style-normalizer default state (recommend: off, opt-in per-session); (d) predictive pre-warm scope (12GB = noop / 24GB = enabled / 48GB = aggressive — match memory:project_vram_tier_plan.md tiers).

## WIRING_CONTRACT

produces_symbols:
  - crates::agents::role_prompt::build_role_prompt(role, voice, task) -> String
  - crates::agents::role_prompt::VoiceAnchor + load_from_toml
  - crates::providers::runtime::RuntimeAdapter::warm() trait method (default no-op + 2 specialized impls)
  - crates::orchestrator::runtime_orchestrator::predict_next_role(fsm_state) -> AgentRole
  - SsbActionType::{UnifiedPreambleApplied, RuntimePrewarmStarted, RuntimePrewarmComplete, HandoffPayloadCompressed, PersonaVectorApplied, StyleNormalizerApplied}
  - python/elai_inference persona_vector decode hook (env-gated)
consumes_symbols:
  - crates/orchestrator/src/runtime_orchestrator.rs::send (wraps build_role_prompt before dispatch)
  - crates/cli/src/handoff.rs (compress_handoff_payload integration)
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (renders ROLE badge from current_role SSB stream)
  - crates/providers/src/runtime/sidecar.rs + vllm.rs (warm impls)
reachable_from:
  - crates/cli/src/main.rs::main -> any dispatch path that goes through RuntimeOrchestrator::send
  - crates/dashboard-v2/src-tauri/src/lib.rs::run -> stream_chat command -> same path
  - FSM state-transition handler in task_runner.rs -> emits predicted_next_role hint
stub_ban:
  - "TODO: build voice anchor later" literal absent
  - "// hardcoded sonnet" literal absent
  - any role-prompt build path that DOES NOT call build_role_prompt
  - DroidSpeak-style cross-architecture KV reuse code absent (per report anti-pattern)
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (role badge layer) + crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (drawer expansion)
observable_effect:
  trigger: "elai.bat repl, run a /task that forces FSM cycle EXECUTING -> AUDITING -> MEMORIZING (e.g. a 2-baby-step plan)"
  expect_ssb: "unified_preamble_applied row per dispatch (>=3 in this cycle); runtime_prewarm_started immediately on each state transition; voice_anchor blake3 stable across all three dispatches"
  expect_artifact: "chat bubbles render ROLE badges (executor / auditor / memory-writer), model name hidden until drawer click; voice (lowercase 'i', no em-dashes, terse) reads consistent across all three bubbles regardless of underlying backend"

## DETERMINISTIC_CHECK

```bash
# 1. role_prompt module exists and is wired
rg "use elai_agents::role_prompt::build_role_prompt" .elai_cc/crates/orchestrator/src/runtime_orchestrator.rs
test -f .elai_cc/crates/agents/src/role_prompt.rs

# 2. VoiceAnchor canonical TOML exists with required sections
test -f .elai_cc/agents/voice.toml
rg "\\[anchor\\]|\\[confidence_calibration\\]|\\[sycophancy_guard\\]|\\[forbidden_phrases\\]" .elai_cc/agents/voice.toml

# 3. New SsbActionType variants present
rg "unified_preamble_applied|runtime_prewarm_started|runtime_prewarm_complete|handoff_payload_compressed|persona_vector_applied|style_normalizer_applied" .elai_cc/crates/orchestrator/src/ssb.rs

# 4. RuntimeAdapter::warm trait method exists with default impl
rg "fn warm" .elai_cc/crates/providers/src/runtime/

# 5. End-to-end smoke
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT COUNT(DISTINCT json_extract(payload, '$.preamble_blake3')) FROM ssb_events WHERE action_type='unified_preamble_applied' AND ts > strftime('%s','now','-10 minutes');"
# Expected: 1 (single voice anchor blake3 across all dispatches in the cycle)

# 6. DroidSpeak-style cross-arch KV reuse is NOT introduced
rg "cross_model_kv|cross_arch_kv|droidspeak" .elai_cc/crates .elai_cc/python
# Expected: zero hits
```

## ACCEPTANCE

- [ ] Single voice anchor file at `.elai_cc/agents/voice.toml` is the only source of voice; grep confirms no inline persona strings in agent role prompts
- [ ] `unified_preamble_applied` SSB row fires per RuntimeOrchestrator::send call with a stable `preamble_blake3` across same voice anchor
- [ ] CommandWindow.svelte chat bubbles render ROLE badges; model name hidden by default; drawer click reveals backend
- [ ] `RuntimeAdapter::warm()` trait method added with default no-op + Sidecar + AnthropicApi specialized impls
- [ ] `runtime_prewarm_started` / `_complete` SSB rows fire on FSM transitions on 24GB+ tier; noop on 12GB tier per memory:project_vram_tier_plan.md
- [ ] LLMLingua-style compression target reached: handoff payloads compressed >=5x (report Table "Latency masking / handoff")
- [ ] Persona-vector hook is env-gated; default off; when on, an A/B coherence test shows >=10% reduction in voice-drift metric (Open Question 3)
- [ ] Style-normalizer is env-gated; default off; never strips auditor BLOCK signal in agent-to-agent communication (only enabled for user-facing CommandWindow stream)
- [ ] No DroidSpeak-style cross-architecture KV reuse code introduced (anti-pattern per report)
- [ ] No mid-stream model swap path introduced (per report "Hidden model swaps mid-response" anti-pattern)

## Tests to add

- `crates/agents/tests/role_prompt_voice_anchor.rs`: load voice.toml, build prompt for each AgentRole, assert anchor blake3 stable across all 8 roles.
- `crates/orchestrator/tests/predicted_next_role_table.rs`: assert FSM state-transition table matches the documented next-role hint table.
- `crates/contracts/tests/runtime_adapter_warm_contract.rs`: assert all 11 RuntimeAdapter impls compile with the new warm() method (default no-op for 9, specialized for 2).
- `crates/contracts/tests/unified_preamble_applied_contract.rs`: assert SSB row shape + preamble_blake3 stability across cycle.
- `crates/cli/tests/handoff_compression_ratio.rs`: assert compress_handoff_payload achieves >=5x on a fixture payload.
- `python/elai_inference/test_persona_vector_extraction.py`: assert persona vector is extracted at boot when env is set + does not crash decode.
- `crates/dashboard-v2/src/lib/windows/CommandWindow.test.ts`: assert ROLE badge rendered, model name hidden by default, drawer expands on click.

## Files touched

- CREATE: `crates/agents/src/role_prompt.rs` (~250 LOC)
- CREATE: `.elai_cc/agents/voice.toml` (~80 lines)
- CREATE: `workflow/reasoning/E-3-decisions.md` (~120 LOC)
- MODIFY: `crates/agents/src/lib.rs` (~5 LOC: export role_prompt)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (~80 LOC: wrap dispatch with build_role_prompt + predict_next_role)
- MODIFY: `crates/orchestrator/src/fsm.rs` (~60 LOC: emit predicted_next_role hint on transition)
- MODIFY: `crates/providers/src/runtime/mod.rs` (~40 LOC: add warm() default impl on trait)
- MODIFY: `crates/providers/src/runtime/sidecar.rs:240` (~40 LOC: specialized warm impl)
- MODIFY: `crates/providers/src/runtime/anthropic_api.rs` (~30 LOC: specialized warm impl)
- MODIFY: `crates/providers/src/runtime/vllm.rs:189` (~10 LOC: enable_prefix_caching assertion)
- MODIFY: `crates/cli/src/handoff.rs` (~180 LOC: compress_handoff_payload + tests)
- MODIFY: `crates/cli/src/live_cli.rs` (~120 LOC: style-normalizer post-pass behind env gate)
- MODIFY: `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` (~80 LOC: ROLE badge + drawer toggle)
- MODIFY: `crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~50 LOC: drawer expansion view)
- MODIFY: `python/elai_inference/server.py` (~200 LOC: persona-vector extraction + decode hook behind env gate)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~40 LOC: 6 new SsbActionType variants)

## Citations / Grounding

- arXiv:2507.21509 — Persona Vectors (Anthropic) — PRIMARY for Task 7
- arXiv:2602.15669 — PERSONA framework (PERSONA-BASE + PERSONA-ALGEBRA) — supplements Task 7
- arXiv:2508.04826 — Persistent Instability in LLM Personality (AAAI 2026) — grounds anti-pattern: "Aggressive prompt-only persona pinning"
- arXiv:2507.03254 — Codified prompting — grounds Task 1 unified system-prompt preamble
- arXiv:2602.15013 — Text Style Transfer via PEFT + round-trip translation — Task 8 style normalizer
- arXiv:2507.18294 — StyleAdaptedLM — supplements Task 8
- arXiv:2411.02820 — DroidSpeak (Microsoft) — CITED IN ANTI-PATTERN to confirm cross-arch KV reuse is out of scope
- arXiv:2504.13171 — Sleep-time Compute — Task 6 + E-1 cross-reference
- arXiv:2310.05736 — LLMLingua — Task 6 compression
- arXiv:2604.06664 — Foundry CUDA-graph templates — Task 4 + 5 pre-warm rationale (graph capture dominates cold start)
- arXiv:2512.01357 — Tangram (warm-pool affinity) — supplements Task 4
- arXiv:2502.00674 — Rethinking Mixture-of-Agents (Self-MoA) — grounds anti-pattern: "Heterogeneous MoA as a default"
- arXiv:2509.23537 — Beyond the Strongest LLM (planner-limited) — grounds When-NOT-to-swap
- aider.chat/2024/09/26/architect.html — Architect/editor split — Task 1 mental model
- cursor.com/blog/2-0 — Cursor Composer model UX — Task 3 expose-via-drawer reference
- code.claude.com/docs/en/sub-agents — Claude Code subagent isolation — analog for ELAI FSM agent isolation
- compelframework.org/articles/casestudy-devin-and-replit — Trust loss from opacity — grounds Task 3 expose-not-hide

## Open questions / risks

- We do NOT have ELAI's current per-FSM-transition cold-start cost broken down by weight load / CUDA graph capture / prefix re-encode (report Open Question 1). Pre-warm benefit is theoretical until Task 4 lands telemetry. Mitigation: ship pre-warm tier-gated; only enable on 24GB+ where the cost is recoverable.
- Persona-vector steering on local Qwen abliterated 3B/7B has NO empirical validation that Anthropic's effect sizes apply (Open Question 2). Mitigation: env-gated; A/B coherence test required before promotion.
- A/B voice-coherence harness (Open Question 3) requires both human eval and LLM-judge — non-trivial to set up. Defer to E-4 dashboard (which already plans UX-rating SSB action types) or to a follow-up E-N.
- Sleep-time digest threshold on small models (Open Question 4) — at what conversation length does the digest start to pay off on Qwen2.5-Coder-3B? Co-investigate with E-1 telemetry.
- Self-MoA (Open Question 5) — does sampling the local model N times beat swap-to-cloud for ELAI's audit/executor workload at controlled cost? Not addressed in this E-N; flag for D-? follow-up.
- Ungrounded conjecture flagged in report: cross-model KV reuse via DroidSpeak hopes — EXPLICITLY rejected as wasted engineering. Code paths that would enable it are barred by stub_ban above.

## Next

E-1 supplies CoreBlock + memory_notes that this E-N consumes as the voice + digest anchor. E-4 (chat UX critique) layers per-claim confidence chips on top of the ROLE badge surface this E-N introduces — coordinate the bubble-header schema. A future E-N can take the voice.toml + role_prompt.rs and seed a tiny LoRA "voice adapter" for the local model (per StyleAdaptedLM arXiv:2507.18294) if A/B testing shows persistent residual voice drift on the local half. D-14 sycophancy probe consumes the unified-preamble sycophancy_guard clause as one of its evaluation signals.
