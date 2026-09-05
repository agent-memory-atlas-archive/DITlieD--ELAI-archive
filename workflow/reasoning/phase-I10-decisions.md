I-10 decisions: prefix invariance + TTL tiering + cache-hit hard guard
2026-06-12 | executor: A79-I10-EXECUTE

WHAT
cache_economics.rs: CacheEconomicsPlanner (TTL tier selector), PrefixInvarianceGuard (miss-rate halter),
max_tokens_clamp (schema-bound helper). CacheEconomicsEvent SSB variant (SCHEMA_VERSION 237, TOTAL_VARIANTS 546).
Frontend pair: CloudSpendSummary + CloudSpendInfo + CloudPanel get three new fields (ttl_tier, breakeven_reads,
guard_trips_today). Contract test: i10_cache_economics_contract.rs.

KEY DECISIONS

D-1 TTL tier policy: 1h on verified-stable prefix (bp.valid=true), 5m on volatile tails. Compose
    on runtime_client.rs existing ephemeral_1h() call at :584; do NOT re-implement. Planner in
    cache_economics.rs decides the tier; the annotation (setting cache_control on the system head
    BEFORE apply_anthropic_cache_markers) happens at call sites in anthropic.rs and anthropic_api.rs.
    RATIONALE: is_none guards in apply_anthropic_cache_markers preserve caller-set values; annotation
    before the call is the only legal injection point (GD-1 hard rule).

D-2 SSB baseline: re-read live SCHEMA_VERSION=236 / TOTAL_VARIANTS=545 at edit time (I-38 + I-43
    already shipped). I-10 bumps to 237 / 546. Pattern-2 single-variant with 3-arm Kind discriminator
    (TtlPlanned / HitRateGuardTripped / PrefixDiffDumped).

D-3 Guard signal source: guard reads BOTH bp.valid (prefix byte-stable signal) AND cache_read_tokens
    from SSB CloudSpendPayload rows. bp.valid alone is insufficient (RISK-2: 8-slot vs 10-slot drift
    in runtime_client.rs means slots 8-9 [Memory/Evidence] are invisible to bp.valid). A miss on
    bp.valid=true turn (cache_read_tokens=0 despite stable hash) triggers the guard.

D-4 max_tokens_clamp: schema-bound helper clamping max_tokens to SCHEMA_FIELDS * MAX_TOKENS_PER_FIELD
    (= schema_fields * 32u32, conservative). The 4096 default burns quota ~25x on a 3-field response.
    Named constants only, no magic numbers.

D-5 Frontend pair: CloudSpendSummary extended with ttl_tier (Option<String>), breakeven_reads (u32),
    guard_trips_today (u32). Frontend renders "--" when ttl_tier is None (no-data state). No new
    Tauri command needed; existing get_cloud_spend wire shape is extended in place (W-4 3-file change).

D-6 Byte-frozen prefix validator: assert_prefix_has_no_mutable_tokens() checks the system head text
    for timestamp patterns, UUID patterns, and mutable endpoint strings before the planner stamps
    the 1h TTL. Lives in cache_economics.rs as a pure fn (no async). Returns a Result listing the
    offending byte ranges.

D-7 Write-premium defaults (re-verified at edit per plan): 1h = 2x write cost (BREAKEVEN_READS_1H=3),
    5m = 1.25x (BREAKEVEN_READS_5M=1). These are planner defaults; live measurement would override
    via journal stats, but measurement is deferred to Tier-2 per W-10. Named constants in module.

D-8 R9 gate: CacheEconomicsEvent is default-ON gated by ELAI_CACHE_ECONOMICS_DISABLE kill-switch (not
    fully wire-dark). No allowlist row needed (R9 only applies to default-OFF flags). The kill-switch
    makes guard trips suppressible in CI where no SSB is present.

D-9 RISK-2 documentation: the 8-slot/10-slot drift in runtime_client.rs is flagged in the contract
    test comment and in the guard implementation (a NOTE comment). Out-of-scope to fix in I-10
    (guarded by scope-contract R-3); logged to gotchas.md.

EXPECTED TEST OUTCOME
- cargo check --workspace --tests: 0 errors
- cargo test -p elai-contracts (i10 contract): PASS 5 arms (schema-lock, ttl-tier-selection,
  guard-trip, stable-prefix-no-trip, max-tokens-clamp)
- cargo test -p elai-orchestrator --lib (action_type_lock 2/2: 237/546): PASS
- cargo test -p elai-contracts e45, i43 11/11, i38 6/6, i34 11/11, h2 cache contract: PASS (no regressions)
- svelte-check 0 errors
- cross_lang_diff.py over changed Rust<->TS pair: 0 drift

UNKNOWNS / DEFERRED
- Live Anthropic cache_read_tokens observed on real cloud turns (Tier-2 production verify, RunPod-PENDING)
- Write-premium re-verification via actual billing data (automated monitoring job, deferred Tier-2)
- 8-slot/10-slot drift fix in runtime_client.rs (dedicated patch, I-11 or slot-hygiene pass)

---

A83-I10-FIX decisions: wire-dark BLOCK remediation
2026-06-12 | fixer: A83-I10-FIX

BLOCK RESOLVED
R7 WIRE-DARK: CacheEconomicsPlanner/PrefixInvarianceGuard/emit_cache_economics_event had ZERO
production callers. Fixed by hooking both seams in crates/cli/src/runtime_client.rs (the only
crate that has both SSB access and cache_head_stable value -- see D-A83-1 below).

D-A83-1 HOOK LOCATION DECISION: audit report named anthropic_api.rs complete() as the F-1 seam.
    The providers crate has no elai-orchestrator dep (confirmed in crates/providers/Cargo.toml),
    so importing CacheEconomicsPlanner there would require a new dep (banned per CLAUDE.md hard rule).
    The exploration map explicitly states runtime_client.rs:584 is "the LIVE 1h-TTL annotation path"
    and says "pick the join the exploration map prescribes". runtime_client.rs is in the cli crate
    which already depends on elai-orchestrator. Both F-1 (plan_ttl -> CacheControl annotation) and
    F-2 (check_miss_rate -> emit) land at runtime_client.rs where cache_head_stable + streaming_ssb
    are co-located.

D-A83-2 GAP STATS CONSERVATIVE DEFAULT: GapStats::default() (mean_gap_ms=0, sample_count=0) is used
    at the plan_ttl call site. This gives TtlTier::Short on an unknown gap (conservative: avoids
    over-paying the 2x 1h write premium on sessions where gap history is unknown). TtlTier::Long is
    only returned when bp.valid=true AND gap_stats.suggests_long_ttl() (sample_count>0 + mean<10min).
    Per-session gap stats from SSB CloudSpendPayload rows is the Tier-2 improvement (RunPod-PENDING).

D-A83-3 GUARD ACCUMULATOR: two u64 fields (cache_guard_hits, cache_guard_misses) added to RuntimeClient
    struct (not global state; per-instance across turns). Seeded 0 in runtime_builder.rs. Incremented
    when cache_head_stable=true (bp.valid). Cumulative snapshot fed to check_miss_rate() each turn.
    Alternative (per-turn snapshot only) was rejected: single-turn miss detection is too noisy; the
    guard is designed to detect a drift pattern across turns, not a one-off API hiccup.

D-A83-4 SESSION-ID PATTERN FIX (F-3 WARN): assert_prefix_has_no_mutable_tokens() extended to detect
    "s-" + 6-10 hex chars (pattern b'0'..=b'9' | b'a'..=b'f'). The original attempt used
    b.is_ascii_hexdigit() && b.is_ascii_lowercase() which fails on digits (is_ascii_lowercase() is
    false for '0'-'9'). Fixed to an explicit range match.

D-A83-5 CONTRACT ARMS W1-W4: four new wiring arms added to i10_cache_economics_contract.rs proving
    (W1) plan_ttl tier maps to correct CacheControl label, (W2) guard fires and SSB row lands via
    emit_cache_economics_event, (W3) session-id detected by validator, (W4) Local backend guard no-op.
    W2 is the strongest proof: it drives emit_cache_economics_event directly (the same call the live
    path uses) and asserts the SSB row lands -- this is a production-path round-trip test.

TEST OUTCOME (A83-I10-FIX)
- cargo check --workspace --tests: 0 errors
- i10_cache_economics_contract: 23/23 PASS (18 existing + 4 new wiring arms + 1 new session-id arm)
- action_type_lock: 2/2 PASS (237/546 pins unchanged)
- h2_cache_stable_layout_contract: 4/4 PASS
- e45_cross_language_drift_contract: 14/14 PASS
- cache_economics unit tests (orchestrator): 21/21 PASS
- Production caller grep confirms: plan_ttl @ runtime_client.rs:590, check_miss_rate @ :871,
  emit_cache_economics_event @ :853 and :881
