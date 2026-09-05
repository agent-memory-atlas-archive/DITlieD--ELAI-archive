# I-10: prefix invariance + ttl tiering + cache-hit hard guard

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever CHEAPER | grounding STRONG | nature BUILD-on-shipped | effort M | WAVE 1 | [KEYSTONE Track 14]
- Premise: ELAI's deep local validation loops silently bust provider cache prefixes through timestamp injection,
  mutable endpoint strings, and default TTL reliance — costing full prompt retransfer on every turn.
  The fix is mechanical: byte-frozen prefixes, explicit 1h/5m TTL tiering chosen by a journal-statistics planner,
  and a hard guard that halts and dumps a blake3 prefix diff on unexplained miss-rate >10%.

## Steal/build
Build on H-1 cache markers + H-2 breakpoints (IS-5/IS-6 — if H-1/H-2 shipped first, I-10 adds the
TTL/economics/guard layer only; re-grep at edit to confirm IS-5 gap status).
Byte-frozen prefix discipline: no timestamps/UUIDs/mutable endpoint strings in the cached prefix region;
dynamic tool restriction via tool_choice/allowed_tools payload attributes, NEVER prompt-text mutation.
Tool-array freeze question (inline vs H-4 deferral): decided per catalog size by MEASURED (cache-hit x token)
cost from the journal — IC-5 binding correction, not the source doc's "override H-4" assumption.
TTL tiering: 1h on the frozen prefix breakpoint, 5m on volatile tails; chosen by a cache-economics planner
reading inter-call gap stats from the journal (landmine #1: 2026-03-06 default TTL flipped 1h->5m; always
set TTL explicitly). Long-TTL entries precede shorter-TTL entries in the request (landmine #1 sequencing).
Write premiums from landmine #4: 1h = 2x base (breakeven ~3 reads); 5m = 1.25x (breakeven 1 read); the
planner uses these as defaults and re-verifies pricing at edit, never treats them as fixed targets.
TPM accounting: schema-bound max_tokens clamping on every cached scoring call (landmine #2: un-clamped
max_tokens=4096 burns quota 25x vs a 3-field score response).
Hard guard: unexplained miss-rate >10% on identical task classes HALTS and dumps a blake3 prefix diff
naming the leaking byte range (IC-4: burst-amortised scheduling; heartbeat-primary DEMOTED to last-resort).
EXCLUDED alternatives: heartbeat-primary as the main TTL mechanism (demoted per IC-4; survives only as
last-resort fallback when burst scheduling cannot apply).

## Existing ground truth
- IS-5: providers/src/types.rs:58-130 CacheControl + ephemeral() + "at most 4 markers" :61; anthropic_api.rs:137/163 still cache_control:None — H-1 gap OPEN at grep time (PLAN-asserted open; executor re-greps per W-7)
- IS-6: context_budget.rs update_breakpoints:1507 (blake3 prefix stability), ContextSlot enum :1079 — 9 slots; bp.valid + adaptive_reduce:1551
- IN-13: NO cache-economics planner / TTL-tier selector / cache-hit hard-guard (cache_check.rs exists as a wire-dark monitor; the guard + planner are absent) — I-10 BUILD on H-1/H-2
- IS-14: production_agent_rpc.rs build_tool_catalog:657 — the tool-array surface that must stay prefix-stable (PLAN-asserted; executor re-greps per W-7)
- H-1/H-2/H-4/H-9: the cache substrate this session composes (PLAN-asserted plans; executor re-greps per W-7)

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- cache_economics.rs: planner (reads inter-call gap stats from the SSB journal, outputs TTL tier per call class + write-premium breakeven check), hard guard (miss-rate monitor, halt + blake3 prefix-diff on >10% unexplained miss), and a schema-bound max_tokens clamp helper
- byte-frozen prefix validation at the prompt assembly seam: reject timestamps/UUIDs/mutable strings, restrict tool dynamics to payload attributes not prompt text
- TTL annotation on every apply_anthropic_cache_markers site (composes H-1/H-2 — never re-implements these)
- CacheEconomicsEvent SSB row (Kind: TtlPlanned / HitRateGuardTripped / PrefixDiffDumped) — baseline+1 from LIVE 221/532
- deterministic fixture harness: seeded two-turn task confirming cache-read on turn-2 + guard trip on injected timestamp

DEFER (Tier-2 / gated):
- live production rollout of the planner to all task classes (GD-4: FP/over-block measured on >=3 real runs before default-ON)
- automated re-verification of write-premium pricing breakpoints (a monitoring job, not a gate)

## Integration
New module: crates/orchestrator/src/cache_economics.rs (~180 LOC: CacheEconomicsPlanner, PrefixInvarianceGuard, max_tokens_clamp)
Seams: providers/src/types.rs marker sites (IS-5) + context_budget.rs update_breakpoints (IS-6) + anthropic_api.rs cache_control path (IS-5 gap closure)
frontend_pair: TTL/economics fields added to the H-9 cost tile (CostDrawer or CachePanel — PLAN-asserted surface name; executor re-greps per W-7)

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/cache_economics.rs::CacheEconomicsPlanner (journal-driven TTL tier selector)
  - crates/orchestrator/src/cache_economics.rs::PrefixInvarianceGuard (miss-rate monitor + blake3 prefix diff)
  - crates/orchestrator/src/cache_economics.rs::max_tokens_clamp (schema-bound per-call helper)
  - +CacheEconomicsEvent (Kind: TtlPlanned/HitRateGuardTripped/PrefixDiffDumped) in ssb.rs / action_type_lock.rs
consumes_symbols:
  - providers/src/types.rs::CacheControl + ephemeral() (IS-5: the marker type I-10 extends with TTL fields) — GD-1
  - context_budget.rs::update_breakpoints (IS-6: the blake3 prefix-stability path I-10 must not violate) — GD-1
  - apply_anthropic_cache_markers (H-1 shipped surface — never re-implement; TTL annotation is the only addition) — GD-1
  - update_breakpoints (H-2 shipped surface — never re-implement) — GD-1
  - production_agent_rpc.rs::build_tool_catalog (IS-14: the tool-array surface kept prefix-stable) — GD-1
reachable_from:
  - providers/src/anthropic_api.rs cache_control path -> CacheEconomicsPlanner::plan_ttl()  [PRODUCTION cloud inference path]
  - context_budget.rs::update_breakpoints -> PrefixInvarianceGuard::check_miss_rate()  [PRODUCTION context assembly]
  - (test callers in contracts/ or #[cfg(test)] do NOT satisfy R7 production reachability)
stub_ban:
  - NO timestamps/UUIDs/mutable endpoint strings in the cached prefix region
  - NO TTL left to provider default (always set explicitly — landmine #1)
  - NO "override H-4 tool-array deferral" hardcoded assumption (measure per IC-5; decide at runtime)
  - NO apply_anthropic_cache_markers re-implementation (H-1 shipped — compose only)
  - NO update_breakpoints re-implementation (H-2 shipped — compose only)
  - NO .unwrap()/.expect() on production paths
  - NO blake2/sha2 (blake3 only per CLAUDE.md hard rule)
  - NO fabricated miss-rate or cost measurements (HD-1: measured per config, never multiplied)
frontend_pair:
  TTL/economics fields on the H-9 cost tile (CostDrawer/CachePanel family) — frontend_pair required because
  the planner decision and guard trips are user-visible economics signals (W-4/W-6).
observable_effect:
  trigger: "cargo run -p elai-cli -- run --task 'write a hello-world function' --turns 2 --emit-ssb"
  expect_ssb: "CacheEconomicsEvent{kind: TtlPlanned, ttl_seconds: 3600|300, breakeven_reads: ...} on turn-1 assembly; on a seeded injected-timestamp fixture: HitRateGuardTripped + PrefixDiffDumped naming the byte-range offset"
  expect_artifact: "turn-2 SSB row shows cache_read_input_tokens > 0 covering the frozen prefix (journal-verified); the injected-timestamp fixture trips the guard within ONE session and names the leaking mutation; a no-guard broken arm (guard disabled) does NOT trip and FAILS the harness check (R2)"

## Grounding
- IS-5 (providers/src/types.rs:58-130 cache plumbing; anthropic_api.rs gap OPEN), IS-6 (context_budget.rs breakpoints), IN-13 (planner absent confirmed)
- IC-4 (heartbeat demoted, TTL tiering primary — ADECO critique §4-3), IC-5 (tool-array decision by measurement, not assumption — consolidated correction)
- Landmines #1 (2026-03-06 TTL flip, set explicitly), #2 (TPM = input + max_tokens, clamp), #4 (write premiums: 1h=2x, 5m=1.25x) — from I-0 CARRIED WEB LANDMINES
- Anthropic caching docs (vendor, rate-limit dossier — WebSearch-confirm write-premium figures at edit)
- SSB: +CacheEconomicsEvent (Kind: TtlPlanned/HitRateGuardTripped/PrefixDiffDumped) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)

## COMPOSES-WITH
H-1 apply_anthropic_cache_markers (shipped — never re-implement; add TTL annotation only)
H-2 update_breakpoints / bp.valid (shipped — never re-implement; compose the miss-rate guard on top)
H-4 tool-search deferral (keep; resolve per-catalog by measurement per IC-5)
H-9 cache hit-rate instrument (the wire-dark monitor becomes the data source for the planner)
E-39 cost gate (the planner feeds economics signals into the existing cost gate)
FULL-FORM EXPANSION REQUIRED before EXECUTING (I-0 reading order rule — effort M at boundary of needing full
context; executor must re-grep IS-5 open-gap status and IC-5 resolution before coding).

End of I-10.
