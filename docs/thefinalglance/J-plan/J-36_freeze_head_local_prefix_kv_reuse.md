# J-36: freeze-the-head local prefix-KV reuse — keep the system+rules+anchor prefix hot and KV-cached on the STREAMING path, minimize only the task tail, per-model/quant eligible

STATUS: PLANNED (WAVE post-CFW; Tier-1 SHIP NOW = streaming-path prefix lease on a frozen head
boundary + per-model/quant eligibility guard + journaled prefill artifact; Tier-2 DEFER = TurboQuant
co-existence + a dashboard prefill-savings tile. BUILD-on-shipped: the PrefixKVCache machinery already
exists, it is just bypassed on the live local path.)

- Track J | lever LOCAL-COG-THROUGHPUT + CANT-DERAIL(rot-avoidance) | grounding STRONG (sidecar code
  verified 2026-06-15: server.py:1819 PrefixKVCache, :1959 init, :2480 "lease NON-STREAMING path only",
  :2486 chunk-gate, :1970 TQ-disable, :2487-2493 marker_boundary seam, :2483-2484 cache token counters)
  | nature ACTIVATION-of-built-substrate | effort M | depends H-50 (regime-aware ContextPolicy, soft)
  + I-15 (semantic/blake3 fast-path, soft) + J-9 (per-model eligibility column, soft) + J-34 (measured
  tail budget, soft) | composes H-42 (serial-depth budget — the constraint that becomes binding once
  this lands)

## Premise

The resident-cog thesis (the local sidecar loads weights ONCE — server.py from_pretrained — and stays a
persistent NF4 process, so per-call cost is prefill + decode with NO weight reload) only pays off if the
STABLE HEAD of every prompt — system prompt + rules + anchor pane (the immutable task-spec slot, I-6) —
is not re-prefilled every single step. The sidecar already HAS the mechanism to avoid that: PrefixKVCache
(server.py:1819) is an LRU of DynamicCache objects keyed by input-id prefix, with a `lease()` that returns
a reusable KV prefix. But three code-verified facts make it a no-op on the live local lane today:

1. The lease is wired on the NON-STREAMING path ONLY (server.py:2480 comment, verbatim: "C-A: Prefix KV
   cache lease (non-streaming path only)"). The interactive local lane streams, so it never leases.
2. Even on the non-streaming path the lease is gated to `input_token_count <= PREFILL_CHUNK_SIZE`
   (server.py:2486) — it only reuses a prefix when the WHOLE input fits one prefill chunk, so a long head
   plus any task tail falls straight to `_chunked_prefill` (2507) with zero prefix reuse.
3. It is disabled entirely under TurboQuant (init_prefix_kv_cache, server.py:1970-1974: "turboquant_bits
   > 0; DynamicCache incompatible with _KVQuantCache").

The result: every local step re-prefills the full ~8K head from cold. This unit makes the head a FROZEN,
KV-CACHED, STABLE prefix on the streaming path so fixed-head prefill drops toward ~0, leaving only the
small per-step task TAIL to prefill. It is the explicit reconciliation of the tension the passes named:
H-50's local "window-minimization / unload-early" policy FIGHTS prefix reuse (unload => cold prefix =>
re-prefill). The resolution is FREEZE THE HEAD, MINIMIZE THE TAIL — H-50's minimization operates only on
the tail; the head segment is never trimmed, reordered, or unloaded, so the two stop fighting and the
local lane is fastest AND most reliable at once (short tail = less rot, near-free head prefill, max
per-step accuracy p).

Source reasoning: workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md PASS 2.5
("freeze the head, minimize the tail", the open fork promoted here) + PASS 3 (the resident-cog model).

## Design doctrine (the hard correction the passes forced)

This is NOT "make the model smaller / smarter" — it is a deterministic-harness throughput lever on a
FIXED model (the harness-engineering spine, PASS 4). And it must be MODEL-AGNOSTIC, because ELAI no
longer assumes one canonical model: J-9's HardwareModelCatalog + the HW-14 policy shift mean the box may
run Qwen2.5-Coder 3B/7B/32B, or a tier-conditional Gemma/Qwen3 on a bigger box. Naive prefix-KV reuse is
correct ONLY for a standard causal-LM with full attention where the head occupies fixed absolute
positions 0..N; it is NOT automatically safe for sliding-window / non-standard-attention architectures,
and it is incompatible with the _KVQuantCache used under TurboQuant. So the unit's correctness rests on a
CONSERVATIVE per-model/per-quant ELIGIBILITY GUARD with a fail-safe default: unknown architecture or TQ
active => DO NOT cache, take the cold prefill (slower but correct). "Works with any local model" is
delivered by the guard's safe fallback, never by assuming the cache is universally valid.

## Steal/build (the frozen-head reuse, FH-1..FH-5)

FH-1. FROZEN HEAD BOUNDARY. Define the head segment = system prompt + rules block + I-6 anchor pane,
marked as a stable cache breakpoint via the EXISTING `marker_boundary` mechanism (server.py:2487-2493 —
already used to pass an Anthropic `cache_control: ephemeral` boundary through; reuse it for the local
head, do NOT invent a second marker system). The head is byte-stable across steps within a session, so
its token-id prefix hashes identically (compose I-15 blake3/xxhash fast-path for the key). The head is
NEVER trimmed/reordered/unloaded by H-50's local policy (this is the new invariant H-50 must honor).

FH-2. STREAMING-PATH LEASE. Extend the prefix-KV lease to the streaming generate path (re-grep the
streaming handler at edit; server.py:2480 marks today's lease non-streaming-only). On a head cache hit,
the streaming prefill processes only inputs from the head boundary forward (the tail delta), reusing the
cropped leased DynamicCache exactly as the non-streaming path already does (server.py:2516-2529 crop-to-
`_prefix_len - 1` invariant — REUSE that proven cropping, do not re-derive it).

FH-3. HEAD-BOUNDARY LEASE, NOT WHOLE-INPUT GATE. Generalize the `input_token_count <= PREFILL_CHUNK_SIZE`
gate (server.py:2486) so the lease can reuse the HEAD prefix even when total input exceeds one chunk: the
lease keys on the head boundary, the tail beyond it goes through `_chunked_prefill` as new tokens. Without
this, a long frozen head never benefits (the exact limitation today).

FH-4. PER-MODEL / PER-QUANT ELIGIBILITY GUARD (the model-agnostic floor). A deterministic predicate
`prefix_kv_eligible(model_profile, quant) -> bool` consuming the shipped ModelCapabilityProfile
(context_budget.rs HW-6) + the runtime quant flag: eligible ONLY when (a) TurboQuant OFF (the
_KVQuantCache incompatibility is already the reason init returns None, server.py:1970), AND (b) the model
is a known standard-causal full-attention architecture. UNKNOWN / sliding-window / TQ-on => ineligible =>
cold prefill, logged with reason (never silent). Compose J-9: the eligibility fact is a natural column on
J-9's ModelCatalogEntry / model_catalog.toml (rider on J-9, recorded below) — but J-36 ships its own
conservative default so it does NOT block on J-9.

FH-5. TAIL BUDGET FROM MEASURED CURVES, NOT FOLKLORE. The freed head capacity sets how big the tail may
grow before rot bites; size the tail budget from J-34's measured per-model fill/positional degradation
curves where available, else a conservative default. Compose J-34; do not hardcode a window fraction.

## Existing ground truth (sidecar code-verified 2026-06-15; re-grep at edit, W-7)

- server.py:1819 `class PrefixKVCache` — LRU of DynamicCache keyed by input-id prefix; `.lease()` at
  :1891-region, `.admit/store` at :1909. SHIPPED.
- server.py:1959 `init_prefix_kv_cache(max_entries, turboquant_bits)` — returns None when turboquant_bits
  > 0 (:1970-1974) or max_entries == 0 (:1977). The TQ incompatibility is real and is FH-4's (a) reason.
- server.py:2480-2503 — the lease block, commented "non-streaming path only"; chunk-gated at :2486;
  marker_boundary passed through at :2487-2493; `cache_read_tokens` / `cache_creation_tokens` /
  `prefix_cache_hit` counters at :2483-2484/2496 (the off-agent done-gate artifact, FH already emits them).
- server.py:2507-2529 — `_chunked_prefill` for input > PREFILL_CHUNK_SIZE + the proven crop-to-
  `_prefix_len - 1` RoPE/attention-mask correctness fix (FH-2 REUSES this; the cropping is load-bearing).
- The Rust local streaming caller (Pass 2.5 cited runtime_client.rs:547 — that exact path was NOT found
  at crates/providers/src/runtime_client.rs this session; re-grep the live streaming client at edit, W-7;
  the load-bearing fact — streaming does not lease — is confirmed sidecar-side regardless).
- H-50 regime-aware ContextPolicy (PLANNED, H-plan Track 9) — J-36 adds the FROZEN-HEAD invariant to its
  local policy. I-6 anchor pane / I-15 semantic cache / I-11 BeliefState (PLANNED). J-9 catalog / J-34
  curves (PLANNED). H-42 serial-depth budget (PLANNED) — the constraint that becomes binding AFTER J-36.
- SSB: LIVE SCHEMA 268 / SSB_ACTION_TYPES 577 (J-35's freshly-verified baseline 2026-06-14;
  TOTAL_VARIANTS = ALL_VARIANTS.len() auto-follows; re-grep BOTH ssb.rs and action_type_lock.rs at edit,
  W-9 — sibling unshipped J files each add +1, so the live baseline at edit may be higher).

## Reads (mandatory before coding)

1. workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md PASS 2.5 + PASS 3 (the thesis).
2. .elai_cc/python/elai_inference/server.py:1804-1985 (PrefixKVCache + init) and :2460-2560 (the
   request prefill/lease/chunked-prefill region + the streaming handler — locate it by grep, W-7).
3. .elai_cc/crates/orchestrator/src/context_budget.rs (ModelCapabilityProfile — HW-6, the FH-4 input).
4. docs/thefinalglance/H-plan/H-50_* (regime-aware ContextPolicy — the local policy J-36 amends).
5. docs/thefinalglance/J-plan/J-9_hardware_detection_model_catalog.md (the eligibility-column rider seam).
6. docs/thefinalglance/J-plan/J-34_empirical_context_calibration.md (the tail-budget curves).

## Tasks (Tier-1)

1. Frozen-head boundary + key (~60 LOC, sidecar): mark the head segment with the existing
   `marker_boundary`; key the leased prefix on the head-boundary token-id hash (blake3, compose I-15).
2. Streaming-path lease (~80 LOC, sidecar): extend the lease + the proven crop-and-feed (server.py:2516-
   2529) to the streaming generate handler so a head cache hit prefills only the tail delta.
3. Head-boundary lease generalization (~30 LOC): allow the lease when total input > PREFILL_CHUNK_SIZE by
   keying on the head boundary (tail beyond it = new tokens via _chunked_prefill).
4. Per-model/quant eligibility guard (~50 LOC, Rust + sidecar flag): `prefix_kv_eligible(profile, quant)`
   consuming ModelCapabilityProfile; TQ-on / unknown-arch / sliding-window => ineligible => cold prefill,
   reason logged. Conservative default so unknown models are SAFE (correct, slower), never wrong.
5. SSB (~6-part): +LocalPrefixCacheEvent (Pattern-2 single-variant; Kind: HeadCacheHit / HeadCacheMiss /
   TailReprefill / CacheIneligible), baseline+1, re-read at edit (W-9). Emit on the streaming prefill
   path carrying head_prefill_tokens (cache_read_tokens), tail_prefill_tokens, eligibility reason.
6. Decision doc: workflow/reasoning/J-36-decisions.md (the H-50 freeze-head invariant, the eligibility
   predicate + conservative default, the TQ co-existence defer, the J-9/J-34 rider seams).

## WIRING_CONTRACT

produces_symbols:
  - sidecar: streaming-path prefix-KV lease + frozen-head boundary keying (server.py region)
  - crates: prefix_kv_eligible(model_profile, quant) eligibility predicate
  - +LocalPrefixCacheEvent SSB (baseline+1)
  - [Tier-2, deferred] prefill-savings dashboard tile
consumes_symbols:
  - server.py::PrefixKVCache (lease/admit), init_prefix_kv_cache, the crop-to-_prefix_len-1 invariant,
    the marker_boundary mechanism (re-implement NONE — extend the call sites only, GD-1)
  - context_budget.rs::ModelCapabilityProfile (HW-6 — the FH-4 eligibility input)
  - H-50 ContextPolicy local arm (the FROZEN-HEAD invariant is added here)
  - I-15 blake3/xxhash cache-key fast-path; J-34 measured tail budget; J-9 catalog (rider column)
reachable_from:
  - the local STREAMING generate request path (the live interactive lane) [PRODUCTION; re-grep at edit]
  - LocalPrefixCacheEvent emitted on that path per step
stub_ban:
  - NO re-implementation of PrefixKVCache / lease / crop logic / init (extend the shipped sidecar, GD-1)
  - NO second marker/breakpoint system (reuse marker_boundary)
  - NO unconditional prefix reuse: the FH-4 guard is mandatory; TQ-on/unknown-arch MUST fall to cold
    prefill (a cache hit reported on an ineligible model is a FAIL)
  - NO head trim/reorder/unload by the H-50 local policy (the frozen-head invariant)
  - NO .unwrap()/.expect() on prod paths; blake3 only; no new deps
frontend_pair:
  prefill-savings tile (Tier-2, DEFERRED — Tier-1 surface is the journaled LocalPrefixCacheEvent rows).
observable_effect:
  trigger: "two consecutive local streaming turns in one session sharing an identical head (system +
    rules + anchor), TQ OFF, on a standard-causal model (Qwen2.5-Coder fixture)"
  expect_ssb: "turn 2 emits LocalPrefixCacheEvent{HeadCacheHit} with head_prefill_tokens ~= 0
    (cache_read_tokens == head length) and tail_prefill_tokens == the small task delta, recomputable
    from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "measured streaming prefill latency/token-count for the head segment on turn 2 is a
    fraction of turn 1 (cold). NEGATIVE (R2): with TurboQuant ON, OR a model whose profile is unknown/
    sliding-window, the SAME trigger emits CacheIneligible with a reason and a FULL cold prefill — a
    HeadCacheHit reported under TQ-on or an ineligible arch FAILS the contract test; a head segment that
    was trimmed/reordered between turns (H-50 violating the frozen-head invariant) yields HeadCacheMiss,
    asserted to NOT happen for a stable head."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-36_freeze_head_local_prefix_kv_reuse.md --run-trigger
rg -n "non-streaming path only" .elai_cc/python/elai_inference/server.py   # the bypass this unit closes (should change/extend)
rg -n "marker_boundary" .elai_cc/python/elai_inference/server.py            # frozen-head seam reused, not re-invented
rg -n "prefix_kv_eligible" .elai_cc/crates/orchestrator/src                 # eligibility guard wired
rg -c "turboquant_bits|_KVQuantCache" .elai_cc/python/elai_inference/server.py  # TQ incompatibility still respected
rg -c "\.unwrap\(\)|\.expect\(" .elai_cc/crates/orchestrator/src/context_budget.rs  # new code: 0 outside tests
test -f workflow/reasoning/J-36-decisions.md
```

## ACCEPTANCE
- [ ] Streaming-path head cache hit lands a LocalPrefixCacheEvent{HeadCacheHit} with head_prefill_tokens
      ~= 0 on turn 2 of a shared-head session (off-agent journal artifact, not stdout).
- [ ] FH-4 guard: TurboQuant ON => CacheIneligible + cold prefill (never a hit); unknown/sliding-window
      arch => CacheIneligible + cold prefill (conservative default proven by fixture).
- [ ] Head-boundary lease works when total input > PREFILL_CHUNK_SIZE (long head benefits).
- [ ] H-50 frozen-head invariant honored: the local window-minimization policy trims only the tail; the
      head segment is byte-stable across turns (contract-asserted).
- [ ] No re-implementation of PrefixKVCache/lease/crop (grep-proven consume-only).
- [ ] SSB 6-part discipline complete; baseline re-read at edit (LIVE 268/577 as of 2026-06-14).
- [ ] cargo check --workspace GREEN; sidecar import/smoke GREEN; auditor PASS on -p elai-orchestrator
      + the sidecar prefill fixture (scoped).
- [ ] No new dependency; blake3 only.
- [ ] Tier-2 (TQ co-existence + dashboard tile) registered as a deferred checkbox in the decisions doc.

## WIRING DONE-GATE (clause by clause)
- W-1: done = LocalPrefixCacheEvent{HeadCacheHit} off the REAL streaming path + the eligibility-fallback
  fixtures passing, not build-green.
- W-2: Tier-1 ships default-ON on the streaming path WHEN eligible (the guard is the gate, not a dark
  flag); ineligible models fall back loudly. No ELAI_*_ENABLE flag needed (behavior is a perf path with a
  correctness guard); if one is added it carries a wire-dark-allowlist row (R9).
- W-5/W-10: standard-causal Qwen fixture is the live trigger; the multi-architecture zoo (Gemma/Qwen3/
  sliding-window) is honest untested residue covered by the conservative-ineligible default (see risks).
- W-7: server.py anchors + the Rust streaming caller re-grepped at edit (line numbers WILL drift).
- W-9: SSB baseline re-read at edit.
- GD-1: consumes the shipped PrefixKVCache machinery; re-implements none.
- GD-2: prefill token counts + eligibility decision recompute byte-identically from the journal under
  ELAI_SEED on a fixed fixture.

## Tests to add
- crates/contracts/tests/j36_prefix_head_reuse_contract.rs (~150 LOC):
  (a) shared-head two-turn fixture => turn 2 HeadCacheHit, head_prefill_tokens ~= 0;
  (b) TurboQuant-on fixture => CacheIneligible + cold prefill, no hit;
  (c) unknown/sliding-window profile => CacheIneligible (conservative default);
  (d) total input > PREFILL_CHUNK_SIZE with a long head => head still leased (FH-3);
  (e) head mutated between turns => HeadCacheMiss (invariant-violation visible, never a wrong-KV hit);
  (f) crop correctness: a leased partial prefix feeds the proven crop-to-_prefix_len-1 path (no RoPE
      mis-rotation / mask underflow — reuse, not re-derive).
- sidecar: a prefill-token assertion fixture proving turn-2 head prefill << turn-1 (latency-independent).

## Files touched
- MODIFY: .elai_cc/python/elai_inference/server.py (streaming-path lease + head-boundary keying; ~120 LOC)
- CREATE/MODIFY: .elai_cc/crates/orchestrator/src/context_budget.rs or a new prefix_eligibility module
  (prefix_kv_eligible predicate; ~50 LOC)
- MODIFY: ssb.rs + action_type_lock.rs et al (6-part, +1 variant)
- MODIFY: the H-50 local ContextPolicy arm (frozen-head invariant; re-grep H-50's landed module at edit)
- CREATE: .elai_cc/crates/contracts/tests/j36_prefix_head_reuse_contract.rs (~150 LOC)
- CREATE: workflow/reasoning/J-36-decisions.md
- [Tier-2, deferred] dashboard prefill-savings tile + cross_lang_diff pass

## Citations / Grounding
- ELAI ground truth: server.py:1819/1959/2480/2486/2507/2516 (sidecar greps 2026-06-15); LIVE 268/577.
- Thesis: workflow/reasoning/elai-local-reasoning-and-context-2026-06-14.md PASS 2.5 / PASS 3.
- Composes: H-50 (ContextPolicy), I-6 (anchor pane), I-15 (semantic cache), J-9 (catalog eligibility
  column rider), J-34 (measured tail budget), H-42 (serial-depth budget becomes binding after this).
- Model-agnostic correctness: prefix-KV reuse is sound for standard causal-LM full-attention at fixed
  absolute positions; sliding-window/non-standard attention is NOT assumed sound — guarded, never claimed.
  (This architecture-class caveat is design-reasoning, to be confirmed per-model against the live arch
  before any model is marked eligible — never asserted from training memory.)

## Open questions / risks
- The multi-architecture zoo (Gemma sliding-window, Qwen3, MoE) is untested; the FH-4 conservative-
  ineligible default keeps it CORRECT (cold prefill) until a per-arch fixture proves eligibility. Eligible
  is opt-in-per-arch, never default-on for unknown models.
- TurboQuant co-existence (a prefix cache compatible with _KVQuantCache) is genuinely harder and is
  DEFERRED to Tier-2 — under TQ the unit is a documented no-op (cold prefill), not a regression.
- J-9 column ownership: J-36 ships its own eligibility predicate; folding the per-model flag into J-9's
  model_catalog.toml is a rider on J-9, not a blocker for J-36.
- The Rust streaming-client anchor (runtime_client.rs:547 in the source reasoning) was not located at that
  path this session — re-grep the live streaming caller at edit (the sidecar-side bypass fact is confirmed
  regardless).

## Next
Frees the local lane from re-prefilling the frozen head every step, so the binding constraint shifts from
window size to the SERIAL DEPTH of the decomposition (H-42) — i.e. J-36 is the precondition that makes
"maximize parallel baby-steps, minimize the serial chain" the real throughput lever. The J-9 eligibility
column + TQ co-existence are the natural follow-ons.

End of J-36.
