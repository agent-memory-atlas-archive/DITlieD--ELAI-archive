# ELAI Context Lifecycle: Regime-Aware In-Session Management + Token-Efficient Cross-Session Handoff — Design

Status: DESIGN (grounded, pre-decomposition). Authored 2026-06-05.
Scope: two coupled halves that share the window/offload machinery — (1) make in-session context
management REGIME-AWARE (the local 25-32K-window / no-cross-request-cache lane needs the OPPOSITE
policy from the cloud 200K / caching-live lane), and (2) make cross-session continuation token-cheap
and semi-automatic (a grounded topic-digest + auto-detect + tiered load). Backs a new H-plan Track 9
(H-50 in-session, H-51 cross-session).

Grounding (strongest first):
- Code read this session by three subagents (file:line throughout). The load-bearing facts:
  (a) the auto-compact threshold is a FLAT 100K, never derived from the model window — so it can
      never fire for a 32K local model; (b) the 85% handoff gate reads its window from
      max_tokens_for_model = 64K-for-any-non-"opus" and ignores ELAI_LOCAL_MAX_CONTEXT, firing at
      ~54K, above the local window; (c) even when it fires, recover_stubs re-inflates the stubs to
      full bytes so net live trim ~0; (d) local prompt caching is a no-op in production because the
      sidecar's own PrefixKVCache is wired only into the non-streaming path while the live local
      client sends stream:true.
- Validated prior art (WebSearch 2026-06-05): Letta sleep-time compute, Mem0, A-MEM, Anthropic
  context engineering, vLLM automatic prefix caching vs HF transformers. Citations at end.
- The user's own Claude-Code handoff rule (now live in ELAI at .claude/rules/handoff-protocol.md)
  and the prior ELAI plans this composes with (H-1, H-2, H-6, H-35, H-49).

This document does not change code. It is a blueprint.

---

## 1. Goal

1. A local model (window 25-32K, no cross-request cache) must stay under its window GRACEFULLY
   (unload old/bulky context, keep a small working set, reload on demand) — not die at a hard
   `context_window_blocked` error, which is what happens today.
2. A cloud model (200K, caching live) must keep a BIG, byte-stable, cached prefix and only compact
   near its real window — the opposite policy. The system must pick the policy by the live backend.
3. Cross-session continuation must cost ~1-3K tokens (a grounded topic-digest), not ~20-27K (the
   packed trimmed transcript ELAI reloads today), and must be SEMI-automatic (auto-DETECT + one-tap
   confirm), never blind auto-continue.

The unifying realization: **cloud and local want opposite context policies, and ELAI runs one
cloud-shaped policy for both.** Everything below follows from that.

---

## 2. The grounded finding (what's actually true today)

### 2.1 Local models are effectively unprotected in-session

- `DEFAULT_AUTO_COMPACTION_INPUT_TOKENS_THRESHOLD = 100_000` is a FLAT constant
  (`orchestrator/src/conversation.rs:22`), set once from one env var, never derived from the model
  window. `maybe_auto_compact` (`conversation.rs:789`) compares cumulative input tokens against it.
  A 32K-window model overflows long before cumulative input reaches 100K → **the auto-compact never
  fires for local.**
- The 85% handoff gate (`cli/src/live_cli.rs:1238`) computes `model_window * 0.85`, but
  `model_window` comes from `max_tokens_for_model` (`cli/src/cli_consts.rs:20-26`), which returns
  **64_000 for any model whose name does not contain "opus"** and **ignores `ELAI_LOCAL_MAX_CONTEXT`
  entirely**. A Qwen local model therefore gets a ~54.4K trigger — above its 32K window. It fires
  too late.
- The window used by the gate (64K-non-opus) and the window used by budget accounting
  (`BudgetPolicy::Local` from `ELAI_LOCAL_MAX_CONTEXT`, `runtime_client.rs:338`) are sourced
  independently and **disagree**.
- Even when the gate fires, the SSB-present production path calls `recover_stubs`
  (`session_dag.rs:1551`) which **re-inflates every stubbed tool-result back to full bytes** before
  the model call → net live-context trim ≈ 0.
- `adaptive_reduce` has zero production callers; `compaction_ladder` (default-ON) runs only in the
  autonomous `task_runner` PreCompact branch over a summary string (`task_runner.rs:9604`), NOT the
  interactive REPL message history. The only live input shrinkage in a local turn is ShortCoder
  code-text compression (~15-30%, code blocks only, `runtime_client.rs:360`).
- The practical local backstop is a HARD ERROR: provider preflight reject
  (`providers/src/providers/mod.rs:257` → `context_window_blocked`), not graceful compaction.

Verdict: a long local session does not gracefully offload — it eventually errors.

### 2.2 Caching: live on cloud, a no-op on local

- CLOUD is fully wired. `apply_anthropic_cache_markers` (`providers/src/providers/anthropic.rs:1094`)
  stamps up to 4 ephemeral breakpoints (tools/system/tail) gated by an exact-model-id min-prefix
  table, applied at all three dispatch sites including `runtime/anthropic_api.rs:219`. The H-1
  "RuntimeAdapter ships cache_control:None" split-brain is REFUTED: `None` is set at construction
  (`anthropic_api.rs:137`) then re-stamped at `complete:219` before the wire. 1h TTL
  (`ephemeral_1h`, `providers/src/types.rs:93`) is applied to the system head only when a blake3
  byte-stability check proves the prefix is unchanged (`runtime_client.rs:509-535`); a guard
  (`PROMPT_CACHE_PREFIX_BOUNDARY=3`, `session_dag.rs:20/934`) keeps trim from busting the prefix.
- LOCAL caching is dark in production. The sidecar runs HuggingFace `transformers` `model.generate`
  (NOT vLLM) and hand-rolled its own `PrefixKVCache` (`python/elai_inference/server.py:1819`, LRU of
  `DynamicCache`, on by default, and it DOES honor incoming Anthropic `cache_control`). BUT that
  cache is wired only into the non-streaming `generate` path; `generate_streaming`
  (`server.py:2320`) never leases/admits and hardcodes streaming `cache_* = 0` (`server.py:3816,
  4732`). The production local client sends `stream:true` (`runtime_client.rs:547`), so it hits the
  streaming endpoint and **bypasses the prefix cache entirely.** Web-confirmed: HF transformers has
  no native cross-request KV reuse; vLLM does (automatic prefix caching).

### 2.3 The two regimes want opposite policies

| | Cloud (200K, caching live) | Local (25-32K, no cross-request cache in prod) |
|---|---|---|
| Goal | keep a big, byte-stable, cached prefix | minimize the prefix |
| Why | caching amortizes a 1.25x write over many 0.1x reads | nothing to amortize, and no room |
| Compaction | only near 200K; never bust the prefix | aggressive UNLOAD at ~70-75% of the small window |
| Offloaded content | keep in prefix (cache pays) | keep UNLOADED; reload one item on demand |
| ELAI today | roughly fits | does not fit: thresholds (100K/64K) above the window; its one cache is stream-bypassed |

---

## 3. Gaps

G1. No single model-window source of truth; the auto-compact (flat 100K) and the handoff gate
    (64K-non-opus) both ignore the real local window. The window is not even consistently defined.
G2. No in-session UNLOAD for the interactive local REPL: nothing shrinks the re-sent context before
    the window overflows; `recover_stubs` re-inflates unconditionally; `adaptive_reduce` is dead.
G3. Local prompt caching is a no-op (the sidecar cache is bypassed on the streaming path), so the
    only local lever is window-minimization — which §G2 doesn't deliver either.
G4. Cross-session resume reloads the packed trimmed transcript (~20-27K on local), not a small
    grounded digest, and is fully manual (no auto-detect).

---

## 4. Design principles (grounded)

P1. ONE window truth, derived thresholds. Compute the model window once (from the live backend:
    `ELAI_LOCAL_MAX_CONTEXT` for local, the cloud ceiling for cloud) and feed BOTH the auto-compact
    and the handoff gate from it. Trigger trim at a fraction of the REAL window, never a flat
    constant and never a wrong default. (Anthropic context-engineering: the policy must match the
    window.)
P2. Regime-opposite policy, selected by backend. Cloud keeps a big cache-stable prefix and compacts
    late; local minimizes the prefix and unloads early. The policy is chosen by `BackendMode`, not
    hardcoded. (vLLM/transformers reality + Anthropic caching economics.)
P3. Unload stays unloaded (local). On local, offloaded content stays offloaded; reload is selective
    and on-demand, never an unconditional re-inflate that defeats the trim. The masked tokens remain
    re-fetchable from SSB (lossless view, like H-6 deterministic masking + the undo RV-2 rule).
P4. Cache honesty. Never advertise a cache hit a path can't deliver. The cache-read fraction is
    MEASURED from the journal PER REGIME (cloud > 0; local = 0 today until §7.4). (GD-2/GD-5.)
P5. Cross-session = small grounded digest, lazy detail. Offload the SEMANTIC digest, reload it (not
    the transcript); the raw transcript stays on disk, pulled on demand. (Mem0 lazy salient-fact
    retrieval; Letta sleep-time consolidation; Anthropic structured note-taking.)
P6. Detect automatically, continue on confirm. The friction the user wants removed is "remember to
    write it + find it + choose to continue"; the risk to avoid is blind auto-continue. So make only
    DETECTION automatic.

---

## 5. Architecture — in-session (H-50)

### 5.1 One model-window source of truth (fixes G1)
Define `effective_context_window(backend)` once: local → `ELAI_LOCAL_MAX_CONTEXT` (or the
`BudgetPolicy::Local` effective ceiling); cloud → the cloud ceiling; api → the model's real window.
Replace the flat `100_000` auto-compact threshold and the `max_tokens_for_model` 64K-non-opus gate
with `trim_at = floor(effective_window * TRIM_FRACTION)` (local ~0.70-0.75, cloud ~0.85). Both the
`conversation.rs` auto-compact and the `live_cli.rs` handoff gate read this one value. This single
change is the highest-leverage fix — it makes the existing trim machinery actually fire for local.

### 5.2 Regime-aware policy (fixes G2/G3)
A `ContextPolicy` chosen by `BackendMode`:
- LOCAL policy: at `trim_at`, run the cheapest-first ladder IN the interactive turn — drop dead
  trailing observations (H-6 deterministic masking) → mask/stub bulky tool-results → narrative-
  summarize the oldest turns (compact_session) — and DO NOT auto-recover the stubs (P3). Keep a
  small working set (recent turns verbatim + the masked tail re-fetchable). `recover_stubs` becomes
  SELECTIVE: called only for a specific stub the model explicitly asks to re-read, not for the whole
  set. This is the "unload/load" system the user is asking about; ELAI already has the offload (SSB
  stubs) and the reload (recover_stubs) — H-50 stops the unconditional reload and triggers the
  unload at the right (window-derived) threshold.
- CLOUD policy: keep the big cache-stable prefix; only compact near the real 200K; the existing
  PROMPT_CACHE_PREFIX_BOUNDARY guard already prevents busting the prefix — preserve it. No change to
  the live caching path beyond ensuring §6 doesn't inject before the breakpoint.

### 5.3 Reuse, don't reinvent
This is mostly WIRING: `adaptive_reduce` + `compaction_ladder` already exist (the ladder is even
default-ON, just wired to the wrong path); `recover_stubs` already does selective recovery by
`stub.ssb_seq`; H-6 is the deterministic-masking activation plan. H-50 = window-truth + route the
ladder/masking into the interactive local turn + make recovery selective + keep the cloud path as-is.

### 5.4 Optional local free-lunch — unbypass the sidecar cache
The sidecar's `PrefixKVCache` is a real cross-request cache that already honors `cache_control`, but
the streaming path skips it. Two ways to claim it: (a) teach `generate_streaming` to `lease()` the
longest cached prefix and `admit()` after generation (mirroring the non-streaming path), or
(b) route the local client to the non-streaming endpoint for turns above a size threshold. Either
turns local caching from a no-op into a ~free-lunch on repeated prefixes (the vLLM-APC benefit,
hand-rolled). This is a Tier-2/optional item — the load-bearing local lever is still §5.2
window-minimization.

---

## 6. Architecture — cross-session (H-51)

The three layers from the prior handoff analysis, now sitting on top of the regime-aware base:

6.1 SLEEP-TIME AUTHOR (capture). At session end / idle, a sleep-agent pass (Letta sleep-time
    compute) authors/updates a TOPIC-keyed digest from the SSB journal (off-agent ground truth) via
    `compact_session`: SUMMARY + DECISIONS (from SSB intervention rows) + FILES (from FileWrite
    rows) + the FSM/plan cursor + open follow-ups. ~1-3K tokens. This is the user's
    `handoff-protocol.md` rule, but auto-generated and grounded in the journal rather than the LLM's
    recollection.
6.2 AUTO-DETECT + ONE-TAP CONFIRM. A fresh REPL checks for {open topic-handoff + non-IDLE FSM +
    active DAG tip} for the cwd/project and surfaces one line ("prior session on <topic> at <state>,
    N open follow-ups — resume? [y/N]"). Detection automatic; continuation confirmed (P6). If
    ambiguous, list the open topics.
6.3 TIERED LOAD. On confirm, load ONLY the ~1-3K digest by default; the full trimmed transcript +
    tool-result stubs stay in SSB and are lazy-pulled via the SELECTIVE recover_stubs from §5.2 only
    when the model asks. Default resume cost on local drops ~20-27K → ~1-3K — the whole point.
6.4 Cloud-cache safety: the digest injects AFTER the cache breakpoint (or as part of the stable
    prefix), so on cloud it never busts the cached prefix (HD-2).

---

## 7. Two approaches considered (per the two-approaches rule)

APPROACH A (CHOSEN): regime-aware policy in ELAI's own context manager (§5-6). Keeps the existing
  HF-transformers sidecar, fixes the window-truth + unload + digest in Rust, optionally unbypasses
  the sidecar's own cache. + Works on the current stack; no infra change; composes H-1/H-2/H-6/H-35.
  - More code than swapping the engine.
APPROACH B (ALTERNATIVE): move local inference to vLLM to get automatic prefix caching for free
  (the ~free-lunch, 2-4x throughput on repeated prefixes). + Eliminates the hand-rolled
  PrefixKVCache + the streaming-bypass bug; gets battle-tested APC. - A real infra change (vLLM on
  the remote box, VRAM/compat re-validation per the GPU-context memory), and it does NOT solve the
  window-minimization problem (a 32K window is still 32K) — caching reduces COMPUTE, not the token
  count the small model must attend to. So even with vLLM, §5.1/§5.2 are still required.
  Verdict: ship A; keep B as a future inference-layer option behind the same `ContextPolicy`
  abstraction. Caching reduces cost-per-token; window-minimization reduces tokens — local needs
  both, but the tokens fix is primary.

---

## 8. Surfaces + SSB

CLI: `elai context status` (show the live window, fill %, regime, trim threshold, what's
unloaded/recoverable); the existing `/compact` stays manual. H-51 adds the resume offer + the
existing `/continue` re-pointed to the tiered (digest-first) load.
Dashboard: a context-budget gauge per regime + an "unloaded N items (reloadable)" indicator; the
H-51 resume offer surfaced in CommandWindow.
SSB (W-9, baseline+N from the LIVE const 224/501 as of 2026-06-05, re-read at edit):
- H-50: REUSE the existing `ContextReduction` payload for unload events (GD-1, no new variant);
  optional +1 `WindowPolicyApplied` (Pattern-2) only if a distinct off-agent regime-decision row is
  needed for the gate. Prefer reuse.
- H-51: +1 `HandoffLifecycleEvent` (Pattern-2; Kind: DigestWritten / ResumeOffered / ResumeConfirmed
  / TieredLoad). baseline+1.

---

## 9. Wiring-enforcement

- W-2/R9: the regime policy is default-ON (kill-switch `ELAI_CONTEXT_POLICY_DISABLE`); the sidecar
  cache-unbypass (§5.4) ships behind a kill-switch and is registered if it stays off.
- R4 off-agent artifact: the done-gate asserts a `ContextReduction` row landed AND the model call
  after a local trim carried FEWER input tokens than before (measured from the journal, not the
  agent's claim). For H-51, the `HandoffLifecycleEvent` rows + the measured ~1-3K resume cost.
- R1/R2 missing=FAIL: a local session driven past `trim_at` with NO ContextReduction row = FAIL; a
  "caching on" claim on a path whose journal shows cache_read=0 = FAIL (CL-4).
- GD-2: the cache-read fraction and the resume token cost are recomputable from the journal under
  ELAI_SEED, per regime.

---

## 10. Reconciliation with existing plans

- H-1/H-2 (cloud caching): confirmed live; H-50 leaves the cloud path intact and only ensures the
  H-51 digest injects after the breakpoint. (Note H-1's split-brain premise is refuted — the cloud
  gap is verification, not wiring.)
- H-6 (deterministic masking + recover_stubs + get_relevant_facts): H-50 is where H-6's masking
  actually runs in the interactive local turn at a window-derived threshold; shared work.
- H-35 (/rewind) + H-49 (undo-tree): the SessionDag + recover_stubs + the lossless-view rule (RV-2)
  are shared substrate; H-51's tiered load reuses the same selective recovery.
- .claude/rules/handoff-protocol.md (the user's live rule): H-51 wires it into the runtime
  (auto-author from the journal + auto-detect), turning a prose rule into an executed, grounded one.

---

## 11. Risks

- Over-aggressive local unload drops context the model still needs: mitigate with the cheapest-first
  ladder (drop dead-trailing → mask → summarize) and keep everything re-fetchable (P3); bound the
  false-drop rate on >=3 runs (GD-4).
- Digest staleness / drift: if files changed since the digest, flag + validate-before-use (compose
  H-30 staleness instrument + H-32 validate-citation).
- Grounding: digest DECISIONS come from SSB intervention rows, FILES from FileWrite rows — never the
  LLM's recollection; fail-closed if the journal row is gone.
- Auditor firewall: no digest/user-memory injection into auditor/adversary roles (Personalization-
  Trap, per H-31).
- The §5.4 streaming cache change must not alter outputs (prefix caching is output-preserving by
  construction; assert byte-identical generations with/without the cache in a fixture).

## 12. Measurement (believe-the-number, HD-1)

One reproducible tuple per regime, from the journal under ELAI_SEED: local → {max input tokens held
in-window over a long task, # unload events, # selective reloads, errors avoided}; cloud →
{cache-read fraction, $/resolved-task}; cross-session → {resume token cost (target ~1-3K), digest
recall on a LongMemEval-style probe}. Numbers are ELAI's own, never a paper figure quoted as ours.

## 13. Prior-art citations (verified 2026-06-05)
- Letta sleep-time compute (offline consolidation; same accuracy 11K vs 20K): letta.com/blog/sleep-time-compute, github.com/letta-ai/sleep-time-compute
- Mem0 (lazy salient-fact load; >90% token savings, +26% on LOCOMO): arxiv.org/abs/2504.19413
- A-MEM (Zettelkasten dynamic linking; NeurIPS 2025): arxiv.org/abs/2502.12110
- Anthropic, effective context engineering (compaction + note-taking + sub-agents, Sep 2025):
  anthropic.com/engineering/effective-context-engineering-for-ai-agents ; effective-harnesses-for-long-running-agents
- vLLM automatic prefix caching vs HF transformers (no native cross-request KV reuse):
  docs.vllm.ai/en/stable/design/prefix_caching/
- Benchmarks: MemoryAgentBench (ICLR 2026), LongMemEval, LoCoMo.

## 14. Open questions
- TRIM_FRACTION values per regime (local 0.70 vs 0.75) — tune on measured runs.
- §5.4: teach streaming to lease/admit vs route local to non-stream — decide on a spike.
- Whether H-50 needs the `WindowPolicyApplied` SSB variant or ContextReduction suffices (prefer reuse).
- Digest grain: per-topic only, or per-topic + per-plan-phase.
