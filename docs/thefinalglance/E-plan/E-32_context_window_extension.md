# E-32: Context window — 32K native lift + Found-in-the-Middle calibration + DCA deep-mode opt-in
Objective: raise ELAI's local context ceiling from 25K to 32K (free — Qwen2.5-Coder-7B is natively trained there; no YaRN, no quality loss); add training-free Found-in-the-Middle (FITM) attention calibration to the sidecar (+15pp RAG recall per arxiv:2406.16008); ship an opt-in "deep mode" that swaps the sidecar to Qwen2.5-7B-Instruct-1M with Dual Chunk Attention (DCA, arxiv:2501.15383) at 64K cap on the 24GB remote; and EXPLICITLY REJECT YaRN on the default path (15-20% quality drop at factor=4 per vllm issue 18728). Low-risk single-line config bump + recall calibration + opt-in alt-model lane.
Estimated effort: 1-2 sessions, ~6-8 h
Pre-requisites: B-37 SHIPPED (RuntimeAdapter); phase-3K RAG (knowledge engine in `crates/knowledge`); B-31 / B-31.5 (resumable indexing + scale bench — PLANNED but no hard block); E-31 (per-VRAM tier matrix — sequence after so deep-mode's 24GB gating uses the same tier file).
Parallel-safe with: E-31 (same model lane, different axis), E-33, E-34, E-35.
Source report: workflow/improvement-sweep-2026-05-19/reports/32-context-window-extension.md

## Context

ELAI's current `ELAI_LOCAL_MAX_CONTEXT=20000` (CLAUDE.md "Remote sidecar" block) sits below the Qwen2.5-Coder-7B native window of 32,768 tokens (huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct model card: "native context length 32,768"). Raising the ceiling to 32K is FREE — no YaRN, no DCA, no fine-tune — the model was trained there. The 25K hard-ceiling in CLAUDE.md was set empirically based on `memory:project_qwen3b_baseline` (25K 100% pass) and is now stricter than necessary; the 7B's published native window is 32K and KV cache at NF4 only grows ~700MB from 25K to 32K (linear-in-length per KV Cache Memory Calculation guide; ~921MB at 25K → ~1.2GB at 32K on Qwen 3B; 7B is ~2x but still well within the 24GB remote envelope). Pushing past 32K requires YaRN (degrades short prompts 15-20% on Qwen3-32B per vllm issue 18728; F1 cliff at 40-50% of advertised window per arxiv:2601.15300) OR DCA via the Qwen2.5-7B-Instruct-1M variant (drop-in, no short-seq impact, near-perfect 1M passkey per arxiv:2501.15383, 96.7% activation memory reduction with chunked prefill).

Lost-in-the-Middle is a structural attention problem (arxiv:2307.03172 TACL 2024: 30%+ drop when answer moves from position 1 to 10 in a 20-doc context; arxiv:2601.15300: Qwen2.5-7B F1 0.55→0.30 at 40-50% of 128K window). Found-in-the-Middle (Hsieh et al. arxiv:2406.16008, ACL 2024) gives +15pp RAG accuracy via training-free positional-prior subtraction with one dummy-doc forward pass — directly applicable to ELAI's RAG path. Self-Route (EMNLP 2024 arxiv:2407.16833) shows 63% of queries get identical answers from RAG and long-context; routing saves 39-65% cost. ELAI's stateless-per-call + knowledge engine + baby-step decomposition is empirically aligned with the long-context-vs-RAG literature: long context wins only on global summarization and tasks where retrieval shreds semantic units; RAG + good chunking continues to win on cost for everything else.

Cited grounding: huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct (32K native), arxiv:2501.15383 Qwen2.5-1M (DCA, 7B-1M variant, 120GB for 1M / 64K feasible on 24GB), arxiv:2406.16008 Found-in-the-Middle (training-free, +15pp), arxiv:2307.03172 Lost in the Middle TACL 2024, arxiv:2601.15300 (Qwen2.5-7B F1 cliff 40-50% of 128K), arxiv:2309.00071 YaRN (rejected for default), arxiv:2404.06654 RULER (only half of "32K" models actually pass 32K), arxiv:2406.06025 RepoQA (7B class struggles past 32K on cross-function reasoning), arxiv:2407.16833 Self-Route (39-65% cost savings).

## Existing state (B/C/D coverage)

- CLAUDE.md "Remote sidecar" block sets `ELAI_LOCAL_MAX_CONTEXT=20000` (line stating "ELAI_LOCAL_MAX_CONTEXT=25000 is the hard ceiling..."). E-32 raises the documented hard ceiling to 32000 with 25000 as the conservative default — CLAUDE.md edit is one line.
- `crates/cli/src/runtime_client.rs` lines 319-326 (verified by grep): `ELAI_LOCAL_MAX_CONTEXT` env override with 131_072 default. E-32 does NOT change the code default; only the env-var doc + the CLAUDE.md ceiling.
- B-37 (SHIPPED): RuntimeAdapter trait + PerRoleRuntime — E-32 adds a "deep_mode" variant slot for the 1M model lane without trait change (uses existing Sidecar variant with a different model path).
- B-31 / B-31.5 (PLANNED, knowledge engine): E-32's FITM calibration runs on the sidecar inference path, NOT on the knowledge-engine retrieval path; B-31 is parallel-safe.
- C-? — no C-plan entry on context window or FITM (grep on C-SUMMARY_INDEX.md returns no matches for "context window", "Found-in-the-Middle", "FITM", "DCA").
- D-? — no D-plan entry overlap.
- Memory:project_vram_tier_plan: deep-mode (Qwen2.5-7B-Instruct-1M at 64K) ships ONLY for the 24GB tier (remote 3090); 12GB local stays on standard 7B at 32K. E-31's tier-matrix file is the canonical source of which tier gets which model.

## Reads (mandatory before coding)

- workflow/improvement-sweep-2026-05-19/reports/32-context-window-extension.md (full)
- F:\.ELAI_workflow\CLAUDE.md "Remote sidecar (canonical)" block (the env-var line)
- F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py around model.generate + logits-processor wiring (grep for `LogitsProcessor`, `ETCLogitsProcessor`, `DoLaSteeringProcessor`)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\runtime_client.rs:319-326 (ELAI_LOCAL_MAX_CONTEXT env override site)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs:1685-1876 (REPL run_turn — second client of ELAI_LOCAL_MAX_CONTEXT)
- F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\state.rs (third client of the env var)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs (RuntimeAdapter::Sidecar; complete/stream_step entry points)
- F:\.ELAI_workflow\workflow\remote-setup\REMOTE-SIDECAR.md (the canonical sidecar boot doc; needs the new env-var update)
- workflow/improvement-sweep-2026-05-19/E-plan/E-31-tier-matrix.md (after E-31; deep-mode row goes here)
- arxiv 2406.16008 (Found-in-the-Middle) PDF — for the calibration formula
- arxiv 2501.15383 (Qwen2.5-1M / DCA) PDF — for the chunked-prefill setup

## Tasks

1. Raise the documented hard ceiling in CLAUDE.md "Remote sidecar (canonical)" block from 25000 to 32000 (~1 LOC). The line currently reads "ELAI_LOCAL_MAX_CONTEXT=25000 is the hard ceiling (KV cache ~921 MB at 25K; total ~2.85 GB). Use 20000 in production." → change to "ELAI_LOCAL_MAX_CONTEXT=32000 is the hard ceiling (native Qwen2.5-Coder-7B window per HF model card; KV ~1.2 GB at 32K on 7B NF4; total ~3.1 GB). Use 25000-28000 in production for headroom." Cite huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct.

2. Update `workflow/remote-setup/REMOTE-SIDECAR.md` to set the documented env-var to 25000 (production) / 32000 (max) (~2 LOC). Today it says 20000 (production); the ceiling lift carries through. SessionDag handoff threshold (85% of max per memory:project_search_architecture) recomputed: 85% of 32000 = 27200 tokens.

3. Implement Found-in-the-Middle calibration as a sidecar LogitsProcessor (~140 LOC NEW in `python/elai_inference/server.py` + ~60 LOC test). New class `FoundInTheMiddleCalibrator(LogitsProcessor)`: on requests with prompt_len > 8K AND `ELAI_FITM_ENABLED=1`, run one extra forward pass with the dummy-doc baseline (the same prompt with the target-position block replaced by a generic dummy), subtract the resulting positional prior from the real logits. Configurable dummy via `ELAI_FITM_DUMMY_TEXT` env (default: 50-token Lorem-ipsum block). Cost: 1 extra forward per long request (per source report cost note). Gate via env var; default OFF until task 11 internal RAG-eval confirms ≥+5pp uplift. Cite arxiv:2406.16008. Add SSB row `fitm_calibration_applied` with `{prompt_len, dummy_len, applied_ms}`.

4. Add per-request `fitm` field to the sidecar Anthropic-protocol request body (~10 LOC). Defaults to env. Lets the dashboard / CLI override on hot tasks.

5. Author opt-in "deep mode" sidecar variant config (~30 LOC python + ~40 LOC doc). When boot env `ELAI_DEEP_MODE=1` is set AND the box has ≥24GB VRAM, load `Qwen/Qwen2.5-7B-Instruct-1M` instead of the abliterated 7B. Chunked-prefill enabled per arxiv:2501.15383 §3.2 (96.7% activation memory reduction; required at 64K to fit on 24GB). Hard cap context at 64000 in deep mode. Refuse to boot deep mode on <24GB boxes — fall back to standard mode + log a warning. Cite arxiv:2501.15383 + memory:project_vram_tier_plan. Add SSB row `deep_mode_activated` with `{model_id, ctx_cap, vram_gb}`.

6. Add `RuntimeOrchestrator` SessionDag-handoff hook for deep-mode escalation (~50 LOC in `crates/providers/src/runtime/escalation_policy.rs` from E-31). When SessionDag handoff fires at 85% of 32K (27200 tokens) AND the task is "cross-file synthesis" (heuristic: >3 files referenced AND no clear retrieval-localised answer), prefer deep-mode escalation over cloud escalation IF a 24GB box is reachable. Cite arxiv:2407.16833 Self-Route + memory:feedback_autonomous_mode. New SSB row `deep_mode_escalation_chosen` with `{reason, files_referenced}`.

7. EXPLICITLY REJECT YaRN in `python/elai_inference/server.py` (~10 LOC). Add a boot-time refusal: if `ELAI_USE_YARN=1` is set AND model is Qwen2.5-Coder family, raise `SidecarBootError("YaRN rejected per E-32: 15-20% quality drop on short prompts (vllm issue 18728); F1 cliff at 40-50% of 128K window (arxiv:2601.15300). Use deep mode (DCA via Qwen2.5-7B-Instruct-1M) for >32K instead.")`. Loud refusal > silent degradation.

8. Add a small RAG-eval harness at `.elai_cc/benches/context_window/run_fitm_eval.py` (~180 LOC). Pulls 20-50 ELAI knowledge-engine retrieval traces from `workflow/sessions/`, runs each with FITM on and off, records recall@3 / recall@5. Output: `benches/context_window/results-{date}.json`. Runs against the local sidecar (no cloud cost). Used to gate FITM default-on per task 3.

9. Update SessionDag handoff threshold logic to use 32000 not 25000 as the basis (~15 LOC in the orchestrator — grep for `session_dag_handoff` or `handoff_threshold`). 85% of 32K = 27200; today's 85% of 25K = 21250. The basis variable should read from the sidecar's reported `effective_max_context`, not be hard-coded.

10. Defer A-MEM-style Zettelkasten note graph (sweep recommendation #5, arxiv:2502.12110 NeurIPS 2025) to a follow-up E-N. Beats RAG on multi-hop cross-session continuity per Letta benchmark; integration surface (knowledge engine + symbol graph crossover) is large and overlaps with B-31 / B-32. Document the deferral and the open question (Q5 in source report: A-MEM vs ELAI's existing knowledge engine + symbol graph — overlap or complementary?).

11. Author `workflow/reasoning/E-32-decisions.md` (~100 LOC): (a) why 32K not YaRN-128K (the 15-20% drop + F1 cliff numbers); (b) FITM rollout gate (default-on AFTER internal RAG-eval shows ≥+5pp recall@3 uplift on ELAI's actual workload — not the paper's NaturalQuestions number); (c) deep-mode opt-in gate (only on 24GB+ box; only on cross-file synthesis after SessionDag handoff); (d) explicit OUT-OF-SCOPE: YaRN (rejected), StreamingLLM (wrong abstraction per arxiv:2309.17453), Ring Attention (single-GPU collapses to vanilla per arxiv:2310.01889), Infini-attention (not ported to Qwen2.5), 1M context (deferred — needs 120GB).

## WIRING_CONTRACT

produces_symbols:
  - python/elai_inference/server.py::FoundInTheMiddleCalibrator (new LogitsProcessor class)
  - python/elai_inference/server.py::activate_deep_mode (new fn)
  - python/elai_inference/server.py::refuse_yarn_at_boot (new fn)
  - crates/providers/src/runtime/escalation_policy.rs::deep_mode_escalation_hook (extends E-31's EscalationPolicy)
  - SsbActionType::FitmCalibrationApplied (new)
  - SsbActionType::DeepModeActivated (new)
  - SsbActionType::DeepModeEscalationChosen (new)
  - SsbActionType::YarnRefused (new)
  - benches/context_window/run_fitm_eval.py (eval scaffold)
consumes_symbols:
  - python/elai_inference/server.py::model.generate LogitsProcessor chain (existing DoLa/ETC/steering wiring; FITM inserts AFTER existing processors per CRANE-style ordering)
  - crates/providers/src/runtime/sidecar.rs::complete + ::stream_step (existing; new `fitm` request field passthrough)
  - crates/cli/src/runtime_client.rs:319-326 (existing ELAI_LOCAL_MAX_CONTEXT site; no code delta — ceiling raise is doc-only via CLAUDE.md)
  - crates/orchestrator/src/grounding_client.rs (SessionDag handoff threshold computation)
reachable_from:
  - sidecar boot -> refuse_yarn_at_boot -> SidecarBootError (if ELAI_USE_YARN=1)
  - sidecar boot -> activate_deep_mode (if ELAI_DEEP_MODE=1 AND ≥24GB) -> deep_mode_activated SSB
  - any request with prompt_len > 8K -> FoundInTheMiddleCalibrator -> fitm_calibration_applied SSB
  - SessionDag handoff at 85% of effective_max_context -> deep_mode_escalation_hook -> deep_mode_escalation_chosen SSB
stub_ban:
  - no YaRN code path remains reachable on default sidecar
  - no hard-coded 25000 or 20000 in handoff-threshold logic (must read effective_max_context from sidecar)
  - no FITM dummy doc longer than 100 tokens (cost ceiling)
  - no deep-mode boot on <24GB box (refuse, fall back, do not crash)
  - no .unwrap() in FITM Python class boundaries (must raise typed errors)
  - blake3 for any cache key (FITM dummy-text cache by hash); NEVER blake2/sha2
frontend_pair: crates/dashboard-v2/src/lib/components/RuntimePanel.svelte (extends with current effective_max_context + deep-mode status row)
observable_effect:
  trigger: "Open dashboard, start a session that hits 27200 tokens of cross-file context (4+ files), with ELAI_FITM_ENABLED=1 and a 24GB remote available"
  expect_ssb: "fitm_calibration_applied on the long requests; deep_mode_escalation_chosen at 85% threshold; deep_mode_activated on the receiving sidecar"
  expect_artifact: "ContextWindow telemetry in MonitorWindow shows 32K effective ceiling; deep-mode handoff event visible in AuditTimelineWindow"

## DETERMINISTIC_CHECK

```bash
# 1. CLAUDE.md ceiling raised to 32000
grep "ELAI_LOCAL_MAX_CONTEXT=32000\|hard ceiling" F:/.ELAI_workflow/CLAUDE.md
# Expected: 32000 string present

# 2. REMOTE-SIDECAR.md production env raised
grep "ELAI_LOCAL_MAX_CONTEXT" F:/.ELAI_workflow/workflow/remote-setup/REMOTE-SIDECAR.md
# Expected: 25000 or 32000 (not 20000) in the production env line

# 3. FITM calibrator class exists
rg -n "FoundInTheMiddleCalibrator|fitm_calibration_applied|ELAI_FITM_ENABLED" .elai_cc/python/elai_inference/server.py
# Expected: >= 4 hits

# 4. Deep-mode activation present
rg -n "activate_deep_mode|ELAI_DEEP_MODE|Qwen2.5-7B-Instruct-1M" .elai_cc/python/elai_inference/server.py
# Expected: >= 4 hits

# 5. YaRN refusal in place
rg -n "refuse_yarn_at_boot|ELAI_USE_YARN" .elai_cc/python/elai_inference/server.py
# Expected: >= 2 hits

# 6. New SSB variants
rg "FitmCalibrationApplied|DeepModeActivated|DeepModeEscalationChosen|YarnRefused" .elai_cc/crates/orchestrator/src/ssb.rs
# Expected: 4 enum arms

# 7. FITM eval scaffold exists
test -f .elai_cc/benches/context_window/run_fitm_eval.py

# 8. SessionDag handoff threshold reads effective_max_context dynamically
rg -n "effective_max_context|handoff_threshold" .elai_cc/crates/orchestrator/src/grounding_client.rs

# 9. End-to-end smoke: long-context request emits FITM SSB
elai /repl "explain the relationship between TaskRunner, SsbWriter, and ContextSafetyGuard across all 3 files in detail"  # designed to exceed 8K prompt
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT count(*) FROM ssb_events WHERE action_type='fitm_calibration_applied';"
# Expected: >= 1 row
```

## ACCEPTANCE

- [ ] CLAUDE.md hard ceiling raised from 25000 to 32000 with grounded citation (HF model card)
- [ ] REMOTE-SIDECAR.md production env-var updated to 25000-28000 range
- [ ] `FoundInTheMiddleCalibrator` LogitsProcessor inserted AFTER existing DoLa/ETC/steering processors (preserves CLAUDE.md anti-disturbance to the LogitsProcessor chain)
- [ ] `fitm_calibration_applied` SSB row emitted for each long-context request when FITM enabled
- [ ] Deep-mode opt-in boots Qwen2.5-7B-Instruct-1M with DCA + chunked prefill at 64K cap on 24GB+ box (refuses on <24GB)
- [ ] `deep_mode_activated` SSB row emitted at sidecar boot when deep mode active
- [ ] `deep_mode_escalation_chosen` SSB row emitted on SessionDag handoff for cross-file synthesis tasks
- [ ] YaRN explicitly refused at boot with `SidecarBootError` when `ELAI_USE_YARN=1` set (loud refusal, no silent fallback)
- [ ] `yarn_refused` SSB row emitted on the refusal path
- [ ] SessionDag handoff threshold reads `effective_max_context` from sidecar (no hard-coded 25000/32000)
- [ ] FITM eval scaffold present; running it is a follow-up
- [ ] Decision doc at `workflow/reasoning/E-32-decisions.md` captures rejection rationale + open questions Q1-Q5
- [ ] LogitsProcessor chain unchanged on requests < 8K prompt (no FITM overhead on short reqs)
- [ ] CLAUDE.md anti-disturbance: no `.unwrap()` introduced; blake3 used for any new cache key

## Tests to add

- `.elai_cc/python/tests/test_fitm_calibrator.py`: assert FITM is a no-op on prompts ≤ 8K; assert one extra forward pass occurs on > 8K; assert logits delta is bounded.
- `.elai_cc/python/tests/test_yarn_refusal.py`: set ELAI_USE_YARN=1 + Qwen2.5-Coder config → assert SidecarBootError raised with cited message.
- `.elai_cc/python/tests/test_deep_mode_boot.py`: mock VRAM at 24GB → assert deep-mode activates; mock at 12GB → assert refusal + fallback to standard.
- `.elai_cc/crates/contracts/tests/fitm_ssb_contract.rs`: SSB row schema validity for fitm_calibration_applied.
- `.elai_cc/crates/contracts/tests/deep_mode_ssb_contract.rs`: SSB row schema validity for deep_mode_activated + deep_mode_escalation_chosen.
- `.elai_cc/crates/orchestrator/tests/handoff_threshold_dynamic.rs`: assert handoff threshold reads from sidecar effective_max_context, not a constant.

## Files touched

- MODIFY: `F:\.ELAI_workflow\CLAUDE.md` (~3 LOC: one line for the ceiling + one line citing E-32-decisions.md)
- MODIFY: `F:\.ELAI_workflow\workflow\remote-setup\REMOTE-SIDECAR.md` (~3 LOC: env-var update)
- MODIFY: `.elai_cc/python/elai_inference/server.py` (+~220 LOC: FITM class + deep-mode boot + YaRN refusal + per-request fitm field)
- MODIFY: `.elai_cc/crates/providers/src/runtime/escalation_policy.rs` (~50 LOC delta: deep_mode_escalation_hook)
- MODIFY: `.elai_cc/crates/providers/src/runtime/sidecar.rs` (~20 LOC: pass-through `fitm` request field)
- MODIFY: `.elai_cc/crates/orchestrator/src/grounding_client.rs` (~25 LOC: handoff threshold reads effective_max_context)
- MODIFY: `.elai_cc/crates/orchestrator/src/ssb.rs` (~25 LOC: 4 new SsbActionType variants)
- MODIFY: `.elai_cc/crates/dashboard-v2/src/lib/components/RuntimePanel.svelte` (~30 LOC: effective_max_context + deep-mode rows)
- CREATE: `.elai_cc/benches/context_window/run_fitm_eval.py` (~180 LOC)
- CREATE: `workflow/reasoning/E-32-decisions.md` (~100 LOC)

## Citations / Grounding

- huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct (cited in source) — "native context length 32,768"
- arXiv:2501.15383 (cited in source) — Qwen2.5-1M technical report: DCA, 7B-1M variant, near-perfect 1M passkey, 96.7% activation memory reduction with chunked prefill
- arXiv:2406.16008 (cited in source) — Found-in-the-Middle: training-free calibration, +15pp RAG accuracy, Recall@3 20.5→68.3
- arXiv:2307.03172 (cited in source) — Lost in the Middle TACL 2024: 30%+ accuracy drop on middle position
- arXiv:2601.15300 (cited in source) — Intelligence degradation: Qwen2.5-7B F1 0.55→0.30 at 40-50% of 128K
- arXiv:2309.00071 (cited in source) — YaRN: standard for Qwen but REJECTED for ELAI default
- vllm issue 18728 (cited in source) — YaRN-static 15-20% quality drop on Qwen3-32B at factor=4
- arXiv:2404.06654 (cited in source) — RULER: only half of "32K"-claimed models actually pass 32K
- arXiv:2406.06025 (cited in source) — RepoQA: 7B-class struggles past 32K on cross-function reasoning
- arXiv:2407.16833 (cited in source) — RAG-vs-LC + Self-Route: 63% query overlap; 39-65% cost savings
- arXiv:2407.01100 (cited in source) — PINE ICLR 2025: mechanistic position-bias removal (deferred — needs inference modification)
- arXiv:2502.12110 (cited in source) — A-MEM NeurIPS 2025 (DEFERRED follow-up E-N)
- arXiv:2309.17453 (cited in source) — StreamingLLM (REJECTED — wrong abstraction)
- arXiv:2310.01889 (cited in source) — Ring Attention (REJECTED — single-GPU collapses)
- arXiv:2404.07143 (cited in source) — Infini-attention (REJECTED — not ported to Qwen2.5)
- arXiv:2412.15204 (cited in source) — LongBench v2: 7B-class long-context understanding genuinely poor regardless of window
- memory:project_qwen3b_baseline, memory:project_gpu_context (local grounding for VRAM math)

## Open questions / risks

- KV-cache footprint at 32K on Qwen2.5-Coder-7B-NF4 on the 12GB local box: should be ~1.2GB (linear extrapolation from 25K ~921MB), but FA2 attention buffers may push total close to local VRAM ceiling. Measure before promoting 32K default-on for local; remote 24GB has ample headroom. Open question Q3 in source report.
- FITM uplift on ELAI's workload is NOT the paper's NaturalQuestions number (+15pp); needs internal eval (task 8) confirming ≥+5pp recall@3 before default-on. Source report Q2.
- Deep-mode boot time: Qwen2.5-7B-Instruct-1M with chunked-prefill at 64K is non-trivial to set up; first-boot may be slow. Profile and document.
- SessionDag handoff threshold change from hard-coded to dynamic-from-sidecar must keep backward-compat for tests that mock the sidecar. Add a sane fallback (25000) only when sidecar is unavailable.
- YaRN refusal is a HARD break: if any existing user has `ELAI_USE_YARN=1` in their env (unlikely; not documented), they'll get a SidecarBootError. Acceptable — loud > silent — and the message tells them to use deep mode instead.
- A-MEM atomic-note graph (deferred) overlaps with knowledge engine + symbol graph (B-31 / B-32). Open question Q5 in source report — spike before committing.
- Deep-mode escalation vs cloud escalation tradeoff: when SessionDag handoff fires, route to deep mode (free, local 24GB) OR cloud Sonnet (~$3/MTok in, ~$15/MTok out)? Source report Q4. Default policy ("prefer local deep mode IF 24GB reachable") is the safe ship; A/B once telemetry exists.

## Next

- E-32.B (deferred): A-MEM Zettelkasten atomic-note graph integration with existing knowledge engine + symbol graph. Cite arxiv:2502.12110.
- E-32.C (deferred): PINE (arxiv:2407.01100 ICLR 2025) — mechanistic position-bias removal — only adopt if FITM uplift is materially smaller than paper claims on ELAI workload.
- B-31 / B-31.5 (knowledge engine): the FITM calibration changes per-request behavior; ensure B-31's resumable indexing path doesn't double-apply FITM.
- C-13 (IPC dispatcher production swap): consumes new SSB variants (fitm_calibration_applied, deep_mode_*) — sequence C-13 after E-32.
- E-31 (per-VRAM tier matrix): E-32 ADDS the deep-mode row (24GB tier alt-model lane) to E-31's tier-matrix file.
- B-37 RuntimeOrchestrator (SHIPPED): the deep_mode_escalation_hook is the second consumer of E-31's EscalationPolicy module; sequence E-32 after E-31 to inherit the module.
