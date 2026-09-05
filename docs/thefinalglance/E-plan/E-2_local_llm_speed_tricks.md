# E-2: Local LLM speed tricks (perceived + actual)
Objective: ship the three highest-impact-per-cost local-inference speedups identified in the report — dashboard frame-paced rendering (smoothStream-style buffer), skeleton bubble + cursor sentinel, and prompt-lookup n-gram speculative decoding on the sidecar — plus HF StaticCache + prefix-KV reuse for the immutable system prefix. Keeps the LogitsProcessor chain (DoLa, ETC, steering) intact; does NOT migrate off `model.generate`.
Estimated effort: 2 sessions (~6-10 h)
Pre-requisites: B-14 (live token streaming chat, DONE) + B-37 (multi-runtime adapter, DONE). Sidecar Python tree at `.elai_cc/python/elai_inference/server.py` must be reachable for the prompt-lookup + StaticCache changes.
Parallel-safe with: E-1 (orchestrator + reasoning + memory crate; this E-N is dashboard + python/sidecar surface), E-3 (handoff UX touches CommandWindow but distinct payloads), E-4 (chat UX) — but coordinate the CommandWindow.svelte edits with E-4 since both modify the same file.
Source report: workflow/improvement-sweep-2026-05-19/reports/02-local-llm-speed-tricks.md

## Context

The user's symptom — "letters come up slowly" — is almost certainly NOT a tokens/sec problem. Current measured output on the remote 3090 with `huihui-ai/Qwen2.5-Coder-3B-Instruct-abliterated` NF4+SDPA (per memory:project_qwen3b_baseline.md) clears human read rate (~5 words/sec). The report Section "Perceived-latency UX" decomposes the actual cause into three: (1) SSE-chunk burstiness from the sidecar (chunks arrive every 50-200ms in clumps), (2) per-char DOM updates causing layout thrash on every token, (3) no "thinking" cue between submit and TTFT. Doherty threshold (0.1s instantaneous, 1s flow-preserving, 10s disengage) is the canonical anchor; TTFT <200ms feels snappy, >1.5s feels broken regardless of subsequent throughput.

The report ranks fixes by ELAI-impact-per-integration-cost (Section "Best techniques to adopt"). Top three: (1) dashboard frame-paced render via RAF-buffer drain at ~40-60 chars/sec — Vercel AI SDK ships `smoothStream` for exactly this; (2) skeleton bubble + cursor sentinel — Bilgihan et al. (ResearchGate 326858669) show structural placeholders rate higher than spinners on perceived speed AND ease of navigation; (3) prompt-lookup n-gram speculative decoding (apoorvumang/prompt-lookup-decoding) — 2x-4x on input-grounded code tasks, native in HF transformers via `prompt_lookup_num_tokens`, KEEPS LogitsProcessor wiring intact (the DoLa/ETC chain runs on the verifier pass). Plus HF `StaticCache` + manual prefix-KV reuse for the immutable system prefix, which delivers a multi-x TTFT cut on agent loops where the same 2-4KB system block is re-prefilled every turn (per vLLM APC measurements).

What the report explicitly rejects on integration grounds: switching to vLLM/SGLang/TensorRT-LLM (forfeits LogitsProcessor chain), Skeleton-of-Thought (breaks single-stream UX + DoLa contrasts), MoE swaps (CLAUDE.md hard rule), and EAGLE-3 on the 12GB tier (deferred to 32B/48GB hardware).

## Existing state (B/C/D coverage)

- B-14 (SHIPPED): `stream_chat(message)` Tauri command at `crates/dashboard-v2/src-tauri/src/lib.rs` + frontend SSE via `@tauri-apps/api/event listen`; `CommandWindow.svelte` + `CommandPalette.svelte` host the bubble. This is the file the frame-pacing buffer goes in front of.
- B-28 (PLANNED — Focus Tunnel E-1..E-7): existing internal `E-1..E-7` step numbering for FIM/DIFF/STRUCTURED mode selection + speculative file routing. NOTE: that sub-numbering is internal to B-28 and is unrelated to this improvement-sweep E-plan. No conflict; just a naming overlap.
- B-37 (SHIPPED): `RuntimeAdapter` trait + Sidecar backend at `crates/providers/src/runtime/sidecar.rs:240-306`. The python sidecar prompt-lookup change is transparent to RuntimeAdapter; no Rust delta there.
- Sidecar reality (per memory:project_dola_tq_findings + memory:project_qwen3b_baseline): DoLa is HARMFUL on 7B (-15% code quality, 4.7x slower) and disabled by default via `--no-dola`; ETC LogitsProcessor + steering are real and live in `python/elai_inference/server.py`. Prompt-lookup runs the LogitsProcessor chain on the VERIFIER pass only — draft tokens bypass DoLa/ETC, which on the 7B is desirable (DoLa already disabled).
- C-? — no overlapping C-plan entry on rendering pacing or speculative decoding (Grep on C-SUMMARY_INDEX.md returns no matches for "smoothStream", "prompt-lookup", "speculative", "frame-pac", "skeleton").
- D-? — no overlapping D-plan entry.
- This E-N is NET-NEW surface (no prior B/C/D work) plus a MODIFY of `python/elai_inference/server.py` model.generate kwargs and `CommandWindow.svelte` SSE consumer.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\02-local-llm-speed-tricks.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\python\elai_inference\server.py` lines 1-100 (header + imports), lines around `model.generate(` call site (find via grep), and the LogitsProcessor wiring section (`DoLaSteeringProcessor`, `ETCLogitsProcessor`, `GGSignalCollector`, `_KVQuantCache` per elai.map lines 22002-22010)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\CommandWindow.svelte:1-200` (current SSE consumer + bubble layout)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\api\stub.ts:1430-1530` (current `stream_chat` stub — actual Tauri command lives in `src-tauri/src/lib.rs`)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src-tauri\src\lib.rs` (find `stream_chat` Tauri command + SSE emit path)
- `F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\sidecar.rs:240-306` (Rust-side stream_step + Other-error mapping — no edits here, just reference)
- `F:\.ELAI_workflow\.elai_cc\crates\cli\src\live_cli.rs:1685-1876` (REPL run_turn — second-tier client of the same sidecar; ELAI_LOCAL_MAX_CONTEXT context window)
- HuggingFace docs (online, do not load into context): `huggingface.co/docs/transformers/v4.50.0/en/generation_strategies` (prompt_lookup_num_tokens), `huggingface.co/docs/transformers/v4.50.0/en/kv_cache#static-cache` (StaticCache + torch.compile interaction)
- apoorvumang/prompt-lookup-decoding README (online; cited by report)
- Vercel AI SDK 4.1 smoothStream source (online; cited by report)

## Tasks

1. **Add a frame-paced render queue between SSE event source and message bubble (~250 LOC in `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte` + new `src/lib/streaming/frame_pacer.ts`).** New module `frame_pacer.ts` exposes a `FramePacer` class with `enqueue(chunk: string)` and a `requestAnimationFrame`-driven drain at a configurable cps (chars-per-second) target. Default 60 cps for code, 100 cps for plain text — auto-detect via the existing markdown-fence heuristic. Falls back to instant-drain on a `flush()` call (used at stream-end so the user is never left waiting for a buffer to empty). Toggle exposed in SettingsWindow as `streaming.frame_pace.enabled` (default true) and `streaming.frame_pace.cps` (40-200 range).

2. **Add SkeletonBubble + CursorSentinel components (~180 LOC, Svelte 5 runes).** `SkeletonBubble.svelte` renders a pre-allocated placeholder with shimmer animation the instant the user submits — pre-allocates expected bubble height via a configurable px estimate (default 240px; user can override). `CursorSentinel.svelte` renders a blinking caret (CSS-only, no JS interval). Both wired into CommandWindow.svelte: skeleton on `submit`, cursor on `streaming=true`, both removed on `stream_complete`.

3. **Add prompt-lookup decoding to `python/elai_inference/server.py` (~30 LOC delta).** Locate every `model.generate(...)` call site; add `prompt_lookup_num_tokens=int(os.getenv('ELAI_PROMPT_LOOKUP_NUM_TOKENS', '10'))` kwarg when the env var is set (default off so existing baselines are not perturbed silently). Gate per-request via a new `prompt_lookup` field on the Anthropic-protocol request body (defaults to env var). Log per-request accept-rate via a new line in the structured log: `prompt_lookup_acceptance: <float>`. Per report Open Question 2: only promote default-on if pooled accept-rate >= 30% over one week.

4. **Add HF StaticCache + manual prefix-KV reuse for the immutable system prefix (~120 LOC in `python/elai_inference/server.py`).** Pre-compute KV for the immutable system prefix once at boot (or when sysprompt changes; key by blake3 of the prefix string). On each request: slice the cached KV into a fresh `StaticCache`, append per-request suffix KV via the normal generate path. Determinism test (Task 9 below) is mandatory because KV bookkeeping off-by-one breaks coherence silently. Gate via env `ELAI_PREFIX_KV_REUSE=1`; default off until determinism test passes for 3 consecutive runs.

5. **Surface per-request streaming-timing telemetry in SSB (~90 LOC).** Add a new SsbActionType `streaming_timing_recorded` carrying `{request_id, ttft_ms, mean_inter_chunk_ms, p50_inter_chunk_ms, p99_inter_chunk_ms, total_ms, tokens_emitted}`. Sidecar emits this via its existing `/notify` -> CLI / dashboard path on `stream_step` finalization. The dashboard MonitorWindow surfaces a small histogram. Without telemetry, we cannot answer report Open Question 1 ("p50 inter-chunk delta on remote 3090 right now") — block default-on of frame-pacing on this data.

6. **Add a "Cancel button on the bubble" affordance (~40 LOC delta in CommandWindow.svelte).** Hook to the existing `stream_chat` cancel path (already shipped per B-14 task 4 "Stop button cancels via cancel_stream"). The report Section 7 calls out: "Cancel button on the bubble. Even unused, its presence reduces felt latency because the user knows they can escape." This is a low-cost reinforcement of B-14, not a new behavior.

7. **Add an explicit "thinking" cue between submit and TTFT (~50 LOC).** When `submit -> stream_start` elapsed time exceeds 200ms (configurable), surface a `<small class="thinking-indicator">elai is reading the project...</small>` line under the skeleton bubble. Disappears on first token. Per Doherty threshold and nngroup research cited in report Section "Perceived-latency UX" — fills the dead-zone with a continuity signal.

8. **Soft-commit partial structures (~80 LOC delta in CommandWindow.svelte markdown rendering).** When a code-fence opening token (` ``` `) arrives, eagerly render the empty code block frame so the user sees structural intent before content lands. Use a simple state machine that tracks fence open/close + buffered partial markdown to prevent flicker on incomplete structures. Per report Section "Stable streaming UI" + smashingmagazine.com/2026/05/designing-stable-interfaces-streaming-content.

9. **Determinism + integrity test harness for prefix-KV reuse (~150 LOC, Python).** New script `python/elai_inference/test_prefix_kv_determinism.py`: runs the same prompt three times with `ELAI_PREFIX_KV_REUSE=1`, compares output token-by-token against the same three runs with `ELAI_PREFIX_KV_REUSE=0`. Asserts byte-equivalent outputs (greedy + temp=0). If not byte-equivalent, the test FAILS and prefix-KV reuse stays env-gated off.

10. **Telemetry-driven A/B harness for frame-pacing (~120 LOC, Svelte settings + log capture).** Per report Open Question 5: ship the frame-pacing toggle behind a setting (default off vs default on, randomized per session), log self-reported "snappy/slow" vote. Persist vote in SSB via a new `ux_pacing_vote_recorded` SsbActionType. N=5 sessions of self-rating is enough to confirm direction.

11. **Write decision doc at `workflow/reasoning/E-2-decisions.md`** covering: (a) frame-pacing default-on threshold (recommend: enable default-on AFTER p50 inter-chunk telemetry confirms >100ms burstiness; otherwise leave opt-in), (b) prompt-lookup-num-tokens value (start 10 per report; adapt via telemetry), (c) prefix-KV reuse rollout gate (must pass determinism test 3x consecutive before default-on), (d) explicit OUT-OF-SCOPE list (EAGLE-3, vLLM migration, Skeleton-of-Thought, MoE swap — all deferred / rejected per CLAUDE.md).

## WIRING_CONTRACT

produces_symbols:
  - crates/dashboard-v2/src/lib/streaming/frame_pacer.ts::FramePacer (enqueue, drain via RAF, flush)
  - crates/dashboard-v2/src/lib/components/SkeletonBubble.svelte
  - crates/dashboard-v2/src/lib/components/CursorSentinel.svelte
  - SsbActionType::{StreamingTimingRecorded, UxPacingVoteRecorded}
  - python/elai_inference/server.py prompt_lookup_num_tokens kwarg surface + prefix-KV reuse path
  - python/elai_inference/test_prefix_kv_determinism.py
consumes_symbols:
  - crates/dashboard-v2/src/lib/windows/CommandWindow.svelte (SSE consumer wraps FramePacer)
  - crates/dashboard-v2/src-tauri/src/lib.rs::stream_chat (emits streaming_timing_recorded at stream-end)
  - crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte:23-58 (new streaming.* settings rows)
reachable_from:
  - User submits a chat turn in dashboard CommandWindow -> SSE event stream -> FramePacer drain -> bubble
  - REPL `elai.bat repl` -> live_cli.rs:1876 runtime.run_turn -> sidecar prompt_lookup_num_tokens path
stub_ban:
  - "TODO: pace later" literal absent
  - "// no pacing" literal absent
  - vLLM/SGLang import statements absent from python/elai_inference (report anti-pattern)
  - any new model.generate(...) call site WITHOUT going through the existing LogitsProcessor chain
frontend_pair: crates/dashboard-v2/src/lib/windows/CommandWindow.svelte + new src/lib/components/SkeletonBubble.svelte + CursorSentinel.svelte
observable_effect:
  trigger: "Open dashboard, submit a 1500-token code generation prompt with frame-pacing enabled at 60 cps"
  expect_ssb: "streaming_timing_recorded within 3s of stream-end with p50_inter_chunk_ms, ttft_ms, tokens_emitted populated; ux_pacing_vote_recorded after the user clicks snappy/slow"
  expect_artifact: "SkeletonBubble visible from submit until first token (with CursorSentinel blinking after first token); rendered char rate close to 60 cps regardless of underlying SSE chunk timing; cancel button visible throughout"

## DETERMINISTIC_CHECK

```bash
# 1. FramePacer module exists and is imported by CommandWindow
rg "from '\\\$lib/streaming/frame_pacer'" .elai_cc/crates/dashboard-v2/src/lib/windows/CommandWindow.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/streaming/frame_pacer.ts

# 2. SkeletonBubble + CursorSentinel exist and are imported
test -f .elai_cc/crates/dashboard-v2/src/lib/components/SkeletonBubble.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/components/CursorSentinel.svelte

# 3. Sidecar prompt-lookup + prefix-KV reuse env hooks present
rg "prompt_lookup_num_tokens" .elai_cc/python/elai_inference/server.py
rg "ELAI_PREFIX_KV_REUSE" .elai_cc/python/elai_inference/server.py
rg "StaticCache" .elai_cc/python/elai_inference/server.py

# 4. New SsbActionType variants
rg "streaming_timing_recorded|ux_pacing_vote_recorded" .elai_cc/crates/orchestrator/src/ssb.rs

# 5. Determinism harness for prefix-KV reuse exists and passes byte-equivalence
test -f .elai_cc/python/elai_inference/test_prefix_kv_determinism.py
cd .elai_cc/python && python -m elai_inference.test_prefix_kv_determinism
# Expected: assertion byte-equivalent across 3 runs

# 6. End-to-end smoke: sustained 200-token generation, observe per-chunk timing histogram
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT json_extract(payload,'$.p50_inter_chunk_ms'), json_extract(payload,'$.ttft_ms') FROM ssb_events WHERE action_type='streaming_timing_recorded' ORDER BY id DESC LIMIT 5;"
```

## ACCEPTANCE

- [ ] FramePacer drains at the configured cps regardless of upstream SSE chunk burstiness (verifiable via test asserting drain ~= cps target +/-10%)
- [ ] SkeletonBubble appears on submit and is removed on first token (no flicker)
- [ ] CursorSentinel blinks throughout streaming and disappears on stream_complete
- [ ] Sidecar `prompt_lookup_num_tokens=10` path produces byte-equivalent output to the baseline at temp=0 (lossless guarantee per report anti-pattern list)
- [ ] Prefix-KV reuse path passes the determinism harness 3x consecutive before being default-on (and remains env-gated otherwise)
- [ ] `streaming_timing_recorded` SSB row emitted for every stream-end with all 6 timing fields populated
- [ ] Cancel button visible throughout streaming and cancels via existing B-14 cancel_stream path
- [ ] DoLa/ETC/steering LogitsProcessor chain remains attached to the verifier pass (rg confirms no `if prompt_lookup: skip_processors` branch added)
- [ ] No new dependency added (per CLAUDE.md "no new dependencies without confirmation"); FramePacer is stdlib + RAF only
- [ ] `.unwrap()` count in non-test source unchanged or decreased

## Tests to add

- `crates/dashboard-v2/src/lib/streaming/frame_pacer.test.ts`: assert drain rate ~= configured cps +/-10% across 3 chunk-burst patterns (uniform, bursty, single-shot).
- `crates/dashboard-v2/src/lib/components/SkeletonBubble.test.ts`: assert mount-on-submit and removal-on-first-token.
- `python/elai_inference/test_prefix_kv_determinism.py`: byte-equivalent output across 3 runs with KV-reuse on/off (Task 9).
- `python/elai_inference/test_prompt_lookup_lossless.py`: temp=0 + greedy, prompt_lookup_num_tokens=10 vs 0, byte-equivalent output across 5 prompts (a code edit, refactor, summarize, scaffold, free-form).
- `crates/contracts/tests/streaming_timing_recorded_contract.rs`: assert SSB row schema validity for streaming_timing_recorded payload.
- `crates/contracts/tests/ux_pacing_vote_recorded_contract.rs`: assert vote-record SSB shape.
- `crates/dashboard-v2/src/lib/windows/CommandWindow.test.ts`: e2e mock SSE stream with bursty chunks, assert FramePacer is engaged and rendered char rate stays near configured cps.

## Files touched

- CREATE: `crates/dashboard-v2/src/lib/streaming/frame_pacer.ts` (~180 LOC, FramePacer class + types)
- CREATE: `crates/dashboard-v2/src/lib/components/SkeletonBubble.svelte` (~90 LOC)
- CREATE: `crates/dashboard-v2/src/lib/components/CursorSentinel.svelte` (~40 LOC)
- CREATE: `python/elai_inference/test_prefix_kv_determinism.py` (~150 LOC)
- CREATE: `python/elai_inference/test_prompt_lookup_lossless.py` (~120 LOC)
- CREATE: `workflow/reasoning/E-2-decisions.md` (~100 LOC)
- MODIFY: `crates/dashboard-v2/src/lib/windows/CommandWindow.svelte:1-200` (~120 LOC delta: import + wire FramePacer + skeleton + cursor + thinking-indicator + soft-commit code fences)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (~70 LOC delta: streaming_timing_recorded emit on stream-end)
- MODIFY: `crates/dashboard-v2/src/lib/windows/SettingsWindow.svelte` (~50 LOC delta: new streaming.* settings rows + frame-pacing A/B vote button)
- MODIFY: `python/elai_inference/server.py` (~150 LOC delta: prompt_lookup_num_tokens kwarg + StaticCache prefix-KV path + acceptance-rate logging)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~30 LOC: two new SsbActionType variants + as_sql_str entries)

## Citations / Grounding

- arXiv:2503.01840 — EAGLE-3 (4.1-6.5x at temp 0; EXPLICITLY DEFERRED to 32B/48GB tier in this E-N per report Section "Best techniques to adopt #7" — call out so future readers don't re-litigate)
- arXiv:2401.10774 — Medusa (2.2-3.6x; rejected for this E-N because draft tokens bypass DoLa contrast — per report Table)
- arXiv:2406.16858 — EAGLE-2 (background; not adopted here)
- arXiv:2402.02057 — Lookahead Decoding (background; rejected because it would replace the generate loop and complicate LogitsProcessor placement)
- arXiv:2311.08252 — REST retrieval-based speculative decoding (alternative; deferred — prompt-lookup has lower integration cost and matches our input-grounded code-task profile)
- github.com/apoorvumang/prompt-lookup-decoding — Prompt Lookup Decoding (PRIMARY citation for Task 3; 2x-4x on input-grounded tasks)
- arXiv:2402.02750 — KIVI 2-bit KV quant (deferred to E-? once context >25K is needed; not in this E-N)
- arXiv:2504.19874 — TurboQuant (already on 32B roadmap; not adopted here)
- HuggingFace transformers v4.50.0 generation_strategies.md (prompt_lookup_num_tokens native support)
- HuggingFace transformers v4.50.0 kv_cache.md (StaticCache + torch.compile interaction)
- vercel.com/blog/ai-sdk-4-1 — smoothStream client-side stream transform (PRIMARY citation for Task 1)
- ResearchGate 326858669 — Bilgihan et al. on skeleton screens (PRIMARY citation for Task 2)
- nngroup / Doherty threshold (PRIMARY citation for Task 7 thinking-indicator threshold of 200ms)
- smashingmagazine.com/2026/05/designing-stable-interfaces-streaming-content (PRIMARY citation for Task 8 soft-commit)
- redis.io/blog/streaming-llm-responses (TTFT vs throughput perception)

## Open questions / risks

- We have NO p50 inter-chunk timing on the remote 3090 today (per report Open Question 1). Frame-pacing benefit is theoretical until Task 5 telemetry lands. Mitigation: ship frame-pacing toggle off-by-default until 1 week of telemetry confirms p50 > 100ms.
- Prompt-lookup acceptance rate depends on ELAI's actual workload mix (report Open Question 2). Promotion to default-on gated on >=30% pooled accept-rate over one week.
- HF StaticCache + flash_attention_2 + `torch.compile(reduce-overhead)` may not fuse on Qwen2.5 in our py3.12/flash-attn 2.7.4 env (report Open Question 3 cites GitHub issue 30055). Mitigation: prefix-KV reuse is env-gated; determinism harness must pass before default-on.
- KIVI 2-bit KV quant is NOT in this E-N (report Open Question 4 has no measurement yet); revisit when context >25K is needed.
- Sidecar boot is HARDWARE-HEAVY (CLAUDE.md hard rule). All Python sidecar changes must be tested against the remote 3090, not a second local instance.
- Ungrounded conjecture flagged in report: that perceived-latency self-report votes will track the academic 40-60% perceived-faster claim. Sample N=5 minimum per Task 10; do not over-claim on smaller samples.

## Next

E-3 (model-handoff UX) reuses the FramePacer + SkeletonBubble for the cross-model role-handoff display. E-4 (chat-ux critique) shares CommandWindow.svelte — coordinate timeline so this E-N lands first (E-4 builds on the skeleton bubble surface for its "per-claim confidence chips" and "doom-loop detector" UI hooks). KIVI 2-bit KV quant + EAGLE-3 land in a later E-N once we have measured Open Question 1+4 data.
