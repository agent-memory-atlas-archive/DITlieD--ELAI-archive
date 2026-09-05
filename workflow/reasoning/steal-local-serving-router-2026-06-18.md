# Steal analysis — local multi-model serving router + multi-seat admission (2026-06-18)

Source: the user's deep-research report (anonymized "build-or-adopt a custom GPU scheduler"
prompt; report archived as the user's Downloads/deep-research-report (24).md, verdict transcribed
here in-repo per W-3 — no summary-only port, the verdict + numbers are reproduced below).
Origin: a discussion of multi-provider gateways, GPU scheduling and admission control.
Private collaboration details omitted; the ELAI design conclusions follow.


## The research verdict (grounded, cited in-report)

ADOPT, DO NOT BUILD a token-level GPU scheduler. For a 2x24GB box (and up): mature engines
(vLLM, SGLang) already own continuous batching, chunked prefill, KV/paged cache, preemption/
recompute, tensor parallel, KV quantization, sleep/offload. A hand-rolled scheduler + MLFQ
reinvents exactly the mechanisms the engine maintainers were still fixing in 2025-2026 (vLLM
added scheduler_reserve_full_isl against over-admission/KV-thrash; SGLang priority-scheduling
still had prefill/decode-disaggregation bugs in 2026), so a custom build's likely yield is
operational complexity, not advantage. TGI entered maintenance mode (repo archived 2026-03-21,
recommends vLLM/SGLang). Dynamo positions ABOVE the engine and says for a single model on a
single GPU the engine alone suffices.

What IS legitimately external/bespoke (the only justified custom code):
- an OUTER router/gateway: route-by-model/tier, max-concurrency per tier, timeout, metrics,
  cloud-fallback (spill) when the big tier is cold/busy/over-capacity.
- a DEAD-SIMPLE warm/cold whole-model policy: pin the small tiers (3B/7B); treat the big tier
  (32B) as EXCLUSIVE mode (it needs both GPUs via TP=2, so when it enters: wait, hibernate the
  small tier, or spill to cloud).
- COARSE admission only: FIFO with at most 2-3 priority classes, NEVER an MLFQ that re-orders
  by estimated tokens (it competes with the engine's own scheduler and usually loses).
- Everything fine (batching, chunking, preemption, KV budgeting) stays in the engine.

Clean boundary the report endorses: GATEWAY owns auth / authz / metering-billing / per-key-team
limits / model-tier routing / cloud fallback / retries / health / observability / coarse
admission; ENGINE owns continuous batching / prefill-decode scheduler / KV-prefix cache /
preemption / TP-PP / KV-quant / fine VRAM use.

Concrete numbers carried (directional, scope-tagged per HD-1, NOT measured on our box):
- KV @ 32K single seq, FP16: 7B ~= 1.75 GiB, 32B ~= 8.0 GiB; FP8 ~= half.
- 4-bit weights floor: 7.6B ~= 3.8 GB, 32B ~= 16 GB decimal; real serving envelope much larger
  (Dynamo rule-of-thumb 14-16 GB for a 7B deploy).
- 32B-4bit @ 32K on ONE 24GB = ~16 GB weights + ~8 GB KV = ~24 GB before overhead -> bad
  territory; needs TP=2 across both GPUs, which consumes both and kills keeping 3B+7B warm.
- Model swap on a shared GPU ~30-120 s per a vLLM RFC (Feb 2026); good case load logs ~10-18 s,
  bad case (unsharded checkpoint / bad IO) ~570 s. Cold-start is dominated by checkpoint format,
  NVMe vs slow storage, TP double-read, warm-up/CUDA-graph capture.
- dual-GPU WITHOUT NVLink: TP raises sync overhead; NCCL can fall back to host memory; the 32B
  tier can become a both-GPU sink with visibly worse TTFT.

## ELAI ground truth (codegraph + feature-map grounded 2026-06-17..18)

What is ALREADY shipped/planned (J-48/J-49 COMPOSE-WITH, never restate — GD-1):
- CloudModeRouter [WIRED, B-27/C-5] cloud_mode.rs:265 struct / :284 route / :309 route_with_spend
  — local-vs-cloud DECISION + daily/monthly USD spend caps. NOT a scheduler, no concurrency, no
  warm/cold, no busy-aware spill.
- RuntimeOrchestrator [WIRED, B-37] runtime_orchestrator.rs:127 / for_role:185 / try_complete:235
  — per-role backend pick + fallback chain + health + BackendCostAccrued. One backend per role,
  sequential. No per-tier concurrency cap, no admission, no whole-model lifecycle.
- VramTier [WIRED] core/src/vram_tier.rs:6 (8/12/16/24/48); providers classify_vram_tier:145 —
  detects what config FITS. Not allocation/scheduling of N concurrent models.
- J-19 vLLM-V1 backend [PLANNED] — the ENGINE adoption (vLLM #12 on the B-37 trait, AWQ-Marlin,
  APC, batch-invariant). SGLang was REJECTED in J-19's own verdict (CVE-2026-7304 dill.loads RCE)
  — so the engine target is vLLM, not SGLang. E-26 has only a partial SGLang adapter.
- J-9 hardware model catalog [PLANNED] — per-lane per-quant VRAM-fit data off the probe stack.
- CASCADING ESCALATION [WIRED, I-7] local->cloud waterfall — J-48 spill rides this, not a new path.
- CALIBRATED ROUTING [PARTIAL, H-7/E-31] RouteDecisionEvent (3-arm) — J-48 prefer-reuses these rows.
- LOCAL/CLOUD ROUTING POLICY [DARK, E-20] xRouter-derived, no callers — different concern
  (cost/privacy routing), NOT model-serving lifecycle; do not fold into it.
- UsageTurnEvent [J-28] carries an identity field default "local" — the dormant multi-seat hook.
- SpendAccumulator / cloud spend caps cloud_mode.rs:683-691 (repriced off models.toml by J-28) —
  J-49 per-seat spend isolation composes with this.
- BSP SUPERSTEPS [WIRED, phase-3D] "JoinSet spawn_parallel + VRAM scheduling" — this is
  macro-orchestration agent parallelism, NOT multi-model serving. DO NOT CONFUSE / DO NOT extend it.

## REJECTED-ledger check (paper-to-plan phase 4; re-proposing a rejection is a P0)

ELAI-FEATURE-MAP.md line ~1710 (goose+reference steal verdicts, 2026-06-12):
  "per-user multi-tenant accounting: REJECTED (single-user; identity stays a FIELD in
   UsageTurnEvent, default 'local')."
  "abstract credit/billing display unit over tokens: REJECTED ... revisit only if multi-tenant."

This is a CONDITIONAL rejection with an explicit revisit trigger ("revisit only if
multi-tenant"). The user has now made multi-seat an explicit goal (2026-06-18), which IS the
documented trigger. So J-49 is NOT a blind re-proposal of a rejected item — it is the
revisit firing, and it ACTIVATES the dormant UsageTurnEvent.identity field rather than inventing
new accounting. The "abstract credit/billing display unit" stays REJECTED — J-49 does per-seat
quota/fair-share/spend-isolation in raw tokens + USD, no synthetic credit unit.

## Verdicts

- ENGINE adoption (vLLM/SGLang): ALREADY-PLANNED (J-19). No new file. J-48 sits ABOVE it.
- per-lane VRAM-fit catalog: ALREADY-PLANNED (J-9). No new file. J-48 consumes its data.
- OUTER local-serving router (per-tier concurrency cap + warm/cold whole-model lifecycle +
  busy/cold cloud-spill + coarse FIFO/2-3-class admission): NEW -> J-48. The report's "minimal
  justified custom code", with the hard boundary "engine owns fine scheduling, J-48 owns coarse
  policy only" baked in as a stub_ban.
- MULTI-SEAT admission + per-seat quota/fair-share + spend isolation: NEW (conditional-revisit
  fired) -> J-49. Activates UsageTurnEvent.identity; raw tokens+USD only; data-driven (a seat
  registry present = multi-seat active, absent = today's single "local" seat, so no W-2
  enable-flag graveyard).
- custom MLFQ token scheduler / synthetic credit billing unit / OpenRouter dynamic registry /
  SGLang as the chosen engine: REJECTED / ALREADY-REJECTED (ledgered, not re-litigated).

Outcome: J-48 + J-49 authored 2026-06-18 (LSW-1/LSW-2). Index + summary + feature-map updated.
