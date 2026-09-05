# J-36 decisions — frozen-head prefix reuse (Local ContextPolicy arm)

## SCOPE (Tier-1 shipped slices 1-5)

J-36 ships prefix-KV cache reuse for the Local regime across 5 slices:

- Slice 1 (streaming lease): lease PrefixKVCache on the streaming path,
  surface cache_* fields in SSE events.
- Slice 2 (FH-4 eligibility gate): gate PrefixKV eligibility through
  ModelFeatures and cache breakpoints.  `prefix_kv_eligible` on
  ModelFeatures; Qwen3.5-4B profile sets it true.
- Slice 3 (SSB registration): register LocalPrefixCacheEvent SSB action
  type and emit CacheIneligible from task dispatch.
- Slice 4 (cache_* threading): thread sidecar cache_* into
  CompletionResponse and emit HeadCacheHit/Miss/TailReprefill from
  try_complete.
- Slice 5 (frozen-head Local arm, this slice): amend H-50
  ContextPolicy::Local with the frozen-head invariant; add
  FROZEN_HEAD_SEQ_BOUNDARY, honors_frozen_head,
  frozen_head_system_prefix_len; wire into compact_session + trim_context;
  contract test + this decisions doc.

## H-50 frozen-head invariant

The frozen head (sys+rules+anchor head, messages with seq <=
FROZEN_HEAD_SEQ_BOUNDARY=3) is never trimmed, reordered, or unloaded.
Only the task tail is minimized.  Both regimes honor the invariant:

- Local: newly honors it (this slice).  The Local arm previously said
  "no cross-request KV cache benefit"; J-36 activates local prefix-KV
  reuse when eligible.  Local still minimizes the TAIL only.
- Cloud: already kept the cache-stable prefix intact (CLOUD_TRIM_FRACTION
  = 0.85 = SESSION_HANDOFF_THRESHOLD).

FROZEN_HEAD_SEQ_BOUNDARY = 3 = PROMPT_CACHE_PREFIX_BOUNDARY (alias-equal
by construction; no second boundary number).

compact_session preserves leading non-summary System messages via
frozen_head_system_prefix_len: the frozen head is copied into the rebuilt
message vector before the continuation summary.  When frozen_len==0
(existing compact summary at index 0, or no leading System), behavior
matches the prior code exactly.

trim_context Pass-1 skip uses both boundaries (defensive; equal by
construction): `msg.seq <= PROMPT_CACHE_PREFIX_BOUNDARY ||
msg.seq <= FROZEN_HEAD_SEQ_BOUNDARY`.

## Eligibility predicate + conservative default

`prefix_kv_eligible` (context_budget.rs ModelFeatures) gates whether the
Local regime attempts prefix-KV reuse.  Conservative default: TQ-on or
TQ-unknown => CacheIneligible (TurboQuant may perturb the KV cache
breakpoints).  The eligibility check runs at task dispatch
(task_runner.rs) and emits a LocalPrefixCacheEvent(CacheIneligible) when
ineligible.

## TQ co-existence (Tier-2 DEFERRED)

The interaction between TurboQuant and prefix-KV cache reuse needs
empirical validation on the live sidecar.  The conservative default
(TQ-on/unknown => CacheIneligible) is safe but may be overly restrictive.
Tier-2 work: measure TQ+prefix-KV on Qwen3.5-4B, relax the gate if the
KV breakpoints are stable under TQ.  Stays DEFERRED with H-50/H-51 as
already written.

## Dashboard prefill-savings tile (Tier-2 DEFERRED)

A dashboard tile showing prefill-token savings from prefix-KV reuse
(HeadCacheHit count + saved tokens) is deferred.  The SSB events are
already emitted (LocalPrefixCacheEvent with HeadCacheHit/Miss/
TailReprefill/CacheIneligible); the dashboard rendering is Tier-2.

## Rider seams (not blockers)

- J-9 catalog column: a per-model `prefix_kv_eligible` column in the
  HardwareModelCatalog would make the eligibility declarative rather than
  hardcoded in ModelFeatures profiles.  Rider seam, not a blocker for
  J-36.
- J-34 curve tail-budget: the calibrated_trim_fraction (J-34) and the
  frozen-head invariant compose naturally (the trim fraction applies to
  the tail, the head is protected).  Rider seam, not a blocker.

## W-1 residual: live two-turn HeadCacheHit

The structural preconditions for prefix-KV reuse are shipped (frozen-head
invariant, eligibility gate, SSB events, cache_* threading).  Verifying
that the sidecar actually reuses the prefix KV across turns requires a
live two-turn HeadCacheHit on a real sidecar run (W-1 done-gate).  This
remains unmet for full done-gate discharge.  The offline contract
(j36_prefix_head_reuse_contract.rs) locks the structural wiring only.

## Model + API constraints

Free glm-5-2 only; no BMAPI/codex for this slice.
