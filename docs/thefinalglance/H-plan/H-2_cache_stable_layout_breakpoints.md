# H-2: Cache-stable layout + breakpoints + 1h TTL — place real markers on a verified-byte-stable cloud prefix

STATUS: PLANNED (WAVE 1; in-process; BUILD — the existing breakpoint mechanism is computed-but-inert + Local-only)

Objective: ELAI's deterministic context-budget allocator already SPECIFIES 2 cache breakpoints (after PhaseSpec,
after CodeContext; context_budget.rs:1191-1200) and computes a blake3 byte-prefix stability flag bp.valid
(:1507) — the single most ELAI-specific cache win, because the allocator + ELAI_SEED can GUARANTEE a byte-stable
prefix turn-to-turn that a heuristic agent structurally cannot. But the whole mechanism is DOUBLY INERT (C3):
bp.valid is consumed ONLY by an eprintln! telemetry line (runtime_client.rs:402-403), update_breakpoints is fed
EMPTY slices for slots 2-7 (runtime_client.rs:379-384, so BP1's hash is degenerate), and it is only reachable
under BudgetPolicy::Local (ApiDefault/Cloud never call it). H-2 feeds real slot bytes, CONSUMES bp.valid to
PLACE cache_control markers on the CLOUD path, adds a SECOND tail breakpoint (20-block lookback), opts into 1h
TTL via "ttl":"1h" (NO beta header — CORRECTED), keys the min cacheable prefix on the exact model id, and adds
the HD-2 byte-prefix debug_assert so a cache-bust becomes a test failure, not a silent bill.

Estimated effort: ~1-2 sessions (~6-10 h). Pure in-process; no network at gate time (W-10 Tier-1 whole).
Pre-requisites: H-1 (the unified marker-application path) MUST land first — H-2 places markers; H-1 makes the
  RuntimeAdapter path capable of carrying them. context_budget.rs breakpoint mechanism SHIPPED (inert).
Parallel-safe with: H-3/H-5/H-8 (disjoint files). NOT parallel-safe with H-1 (shared marker path; sequence H-1 -> H-2).
Source: ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.2 (LEVER L2) + §15 (caching mechanics ledger);
  _grounding/H-GROUNDING-MAP.md CORRECTION C3 + the H-2 symbol table + verify:caching-api. Doctrine: H-0_INDEX.md.

## Context

"Don't Break the Cache" (arXiv:2601.06007 [WEAK]): naive full-context caching can PARADOXICALLY INCREASE latency
because dynamic tool results trigger cache writes for content never reused; the fix is layout — volatile content
AFTER the last breakpoint, exclude tool results from the cached region (HD-2). The cache key is a 100% byte-
identical prefix; any change before a breakpoint busts it and everything after. The 20-block lookback window means
a growing conversation that appends >20 blocks past a breakpoint silently misses unless a SECOND tail breakpoint is
placed. ELAI's edge (where it WINS, not ties): the allocator + ELAI_SEED can prove prefix-stability deterministically
and ASSERT on it; Claude Code / Cursor rely on incidental ordering and silently lose the cache on reorderings
(Cursor shipped a 0%-cache Auto-mode bug, Mar 2026). H-2 turns that latent guarantee into placed markers + a hard assert.

## Existing state (B/C/D/E/F/G coverage)
- ContextBudget::update_breakpoints (orchestrator/src/context_budget.rs:1481) — WIRE-DARK. Computes blake3-prefix
  hashes + sets bp.valid; the only production consumer of bp.valid is the eprintln! at runtime_client.rs:402-403.
  Never places a marker. Single prod caller runtime_client.rs:376 (Local branch); rest test.
- CacheBreakpoint (context_budget.rs:1113) — 2 breakpoints at :1191-1200 ([0]=PhaseSpec, [1]=CodeContext; test pins :2541-2542).
- bp.valid (context_budget.rs:1507) — the blake3 byte-prefix stability flag; inert (eprintln only).
- runtime_client.rs:376-385 — feeds slots 0 (system_bytes) + 1 (rules_bytes) real content, slots 2-7 EMPTY (Vec::new() :375);
  ApiDefault (:414) / Cloud (:418) never call update_breakpoints; only cache_control sites here are tool_executor.rs:231/238 (None).
- CumulativeTokenTracker (context_budget.rs:774) — SHIPPED (the live cache_read_tokens counter; grounding_client.rs:905-906).
- H-1 unified marker path (apply_anthropic_cache_markers + the RuntimeAdapter stamp) — the placement target H-2 drives.

## Reads (mandatory before coding)
- _grounding/H-GROUNDING-MAP.md CORRECTION C3 + the H-2 symbol table + verify:caching-api (the CORRECTED 1h-TTL +
  per-model-id min-prefix + 20-block + invalidation-order rows).
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.2 (L2) + §15 (caching mechanics) + §16/E4.
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\context_budget.rs:1100-1210,1470-1520 (CacheBreakpoint + update_breakpoints + bp.valid)
- F:\.ELAI_workflow\.elai_cc\crates\cli\src\runtime_client.rs:320-413 (the Local branch, the empty slices, the eprintln consumer)
- CLAUDE.md HARD RULES + HD-2 (cache-preserving) + HD-1 (measured-not-multiplied).

## Tasks (WAVE 1 — build on the cloud path)

1. **Feed real per-slot bytes to update_breakpoints on the cloud path (~70 LOC).** Replace the empty slices
   (runtime_client.rs:379-384) with the real bytes of slots 2-7, and call update_breakpoints from the ApiDefault/Cloud
   branches (today only Local calls it). BP1's hash is no longer degenerate.

2. **Consume bp.valid to PLACE a marker only on a verified-stable prefix (~60 LOC).** When bp.valid is true for a
   breakpoint, place a cache_control marker there via the H-1 unified path; when false, place NO marker (a verified-
   unstable prefix is never cached — that is the whole point). Respect the 4-breakpoint cap (exceeding -> 400).

3. **Add a SECOND tail breakpoint (~40 LOC).** Place a breakpoint near the conversation tail so a tool loop that
   appends >20 blocks past the stable-prefix breakpoint still hits (the 20-block lookback). Use 2 of the 4 breakpoints
   (stable-prefix + tail), reserve 2.

4. **Opt into 1h TTL on long sessions (~20 LOC).** Set "ttl":"1h" on the cache_control marker for sessions expected
   to exceed 5 minutes (the 5-min default expires mid-batch; 1h write is 2x but free hits refresh it). NO beta header
   (extended-cache-ttl-2025-04-11 is REMOVED — CORRECTED; "ttl":"1h" is GA). Breakeven note: 1h breaks even on the
   SECOND read (2x write), 5-min on the FIRST.

5. **Add the HD-2 byte-prefix debug_assert (~20 LOC).** Assert the cached byte-prefix is identical turn-to-turn; a
   nondeterministic byte (timestamp, random id, reordered map) in the prefix is then a TEST FAILURE, not a silent
   bill. Volatile content (tool results, SSB rows, RAG chunks) lives strictly AFTER the last breakpoint (invalidation
   order tools->system->messages).

6. **Key min cacheable prefix on the exact model id (~15 LOC).** 1024 for Opus 4.8 / Sonnet 4.6; 4096 for Opus
   4.7/4.6/4.5 + Haiku 4.5; 2048 for Haiku 3.5. Read the live model id; never assume "Opus=4096".

7. **Decision doc workflow/reasoning/H-2-decisions.md (~100 LOC).** Records: the empty-slice + Local-only inertness;
   the bp.valid -> marker placement rule; NO 1h beta header; the per-model-id min-prefix; the tail-breakpoint rationale;
   the debug_assert-as-cache-bust-guard.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/context_budget.rs (cloud-path update_breakpoints reach + bp.valid -> marker-placement decision)
  - crates/cli/src/runtime_client.rs (real slot-byte feed; the tail breakpoint; the HD-2 byte-prefix debug_assert; the cloud-branch update_breakpoints call)
  - crates/providers/src/providers/anthropic.rs OR runtime/anthropic_api.rs (the "ttl":"1h" opt-in + per-model-id min-prefix on the marker)
consumes_symbols:
  - crates/orchestrator/src/context_budget.rs::update_breakpoints
  - crates/orchestrator/src/context_budget.rs::CacheBreakpoint
  - crates/orchestrator/src/context_budget.rs (bp.valid)
  - crates/orchestrator/src/context_budget.rs::CumulativeTokenTracker
  - the H-1 unified marker application path (apply_anthropic_cache_markers / RuntimeAdapter stamp)
reachable_from:
  - crates/cli/src/runtime_client.rs (BudgetPolicy::ApiDefault/Cloud branch) -> update_breakpoints (real slots) -> bp.valid -> marker placed -> the outgoing cloud request  [PRODUCTION cloud path]
stub_ban:
  - NO marker placed on a prefix where bp.valid is false (a verified-unstable prefix is never cached — fail-closed)
  - NO extended-cache-ttl-2025-04-11 beta header (REMOVED; use "ttl":"1h")
  - NO min-prefix tier assumption (key on exact model id)
  - NO volatile content (tool results / SSB rows) placed BEFORE the last breakpoint (HD-2)
  - NO empty-slice feed to update_breakpoints on the cloud path
  - blake3 only (bp.valid hash); no .unwrap()/.expect() in prod
frontend_pair:
  N/A own pair — the breakpoint-stability + hit-rate are surfaced by H-9's cockpit (cross-ref). Justified.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a real multi-turn cloud-mode task (>=3 turns) with a
    stable PhaseSpec/CodeContext prefix, NO env override>"
  expect_ssb: "the H-9 cache path reports a RISING cache-read fraction across turns (the byte-stable prefix hits);
    bp.valid is true and consumed to place a marker (recorded in the journal/telemetry)"
  expect_artifact: "the cached byte-prefix is byte-identical turn-to-turn (the HD-2 debug_assert holds); the cloud
    request carries a cache_control marker on the verified-stable prefix + a tail breakpoint; a deliberate prefix
    perturbation FAILS the debug_assert (the cache-bust guard fires)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-2_cache_stable_layout_breakpoints.md --run-trigger
rg "update_breakpoints" .elai_cc/crates/cli/src/runtime_client.rs            # now called on the cloud branch, not just Local
rg "bp.valid|cache_control" .elai_cc/crates/cli/src/runtime_client.rs        # bp.valid now drives marker placement
rg "extended-cache-ttl" .elai_cc/crates/providers .elai_cc/crates/orchestrator  # MUST be 0 (no 1h beta header)
rg "ttl.*1h|\"1h\"" .elai_cc/crates/providers/src/providers/anthropic.rs     # 1h TTL via the marker
rg "debug_assert" .elai_cc/crates/cli/src/runtime_client.rs                  # the byte-prefix stability assert exists
test -f workflow/reasoning/H-2-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (markers placed on a verified-stable prefix; the
  debug_assert holds on a stable run + FAILS on a perturbed run; the hit-rate rises). cargo-green != DONE.
- [ ] W-2 default-ON: cache-stable layout active with no env override.
- [ ] W-3 production entrypoint: update_breakpoints fires from the live cloud branch (ApiDefault/Cloud), not Local-only, not test.
- [ ] HD-2: a prefix perturbation FAILS the byte-prefix debug_assert (the cache-bust is a test failure).
- [ ] HD-1: any reported saving is the MEASURED hit-rate delta (via H-9), never the literature 41-80% quoted as ELAI's.
- [ ] No 1h beta header sent; min-prefix keyed per-model-id; <=4 breakpoints.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator -p elai-cli -p elai-providers.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE 1; bp.valid (consumed) drives a placed marker (W-8) on the live cloud path (W-3).
- W-2: active with no override; only a kill-switch disables.
- W-3: the ApiDefault/Cloud branch (not Local-only, not test).
- W-4: N/A own pair (surfaced by H-9); justified.
- W-5: a runnable multi-turn cloud task -> a real wire artifact (markers) + a real assert + a real hit-rate; in-process.
- W-6: dashboard E2E is H-9's; H-2 N/A justified.
- W-7: update_breakpoints / bp.valid / CacheBreakpoint / the empty-slice site grep-confirmed 2026-06-03 (C3).
- W-8: the real slot-byte feed (produced) is consumed by update_breakpoints; bp.valid (consumed) drives the placed marker (produced).
- W-9: NO new action_type (reuses the caching path).
- W-10: Tier-1 in-process; no network at gate time.
- HD-2: the byte-prefix debug_assert makes a cache-bust a test failure; masking/volatile content stays after the last breakpoint.

## Tests to add
- crates/contracts/tests/h2_cache_stable_layout_contract.rs (~170 LOC): (a) update_breakpoints fed real slot bytes
  (not empty) yields a non-degenerate BP1 hash; (b) bp.valid true -> a marker is placed, bp.valid false -> NO marker;
  (c) a stable prefix is byte-identical across two builds (the debug_assert holds), a perturbed prefix FAILS it;
  (d) a tail breakpoint is placed when the conversation exceeds 20 blocks past the stable-prefix breakpoint;
  (e) "ttl":"1h" is set on long sessions and NO extended-cache-ttl beta header is emitted; (f) min-prefix keyed per model id.

## Files touched
- MODIFY: crates/cli/src/runtime_client.rs (~120 LOC: real slot bytes + cloud-branch update_breakpoints + tail breakpoint + debug_assert)
- MODIFY: crates/orchestrator/src/context_budget.rs (~50 LOC: bp.valid -> marker-placement decision exposed)
- MODIFY: crates/providers/src/providers/anthropic.rs (~25 LOC: "ttl":"1h" opt-in + per-model-id min-prefix on the marker)
- CREATE: crates/contracts/tests/h2_cache_stable_layout_contract.rs (~170 LOC)
- CREATE: workflow/reasoning/H-2-decisions.md (~100 LOC)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.2 (L2; arXiv:2601.06007 [WEAK]; the ELAI allocator/ELAI_SEED edge).
- Anthropic caching docs (verify:caching-api, 2026-06-03): 1h TTL GA via "ttl":"1h" (NO beta header — the gate was
  REMOVED); min-prefix per-model-id (Opus 4.8=1024); 4 breakpoints; 20-block lookback; tools->system->messages invalidation.
- ELAI ground truth (C3, grep-confirmed 2026-06-03): update_breakpoints context_budget.rs:1481; 2 bps :1191-1200;
  bp.valid :1507 (eprintln-only consumer runtime_client.rs:402-403); empty slices runtime_client.rs:379-384; Local-only.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; HD-1/HD-2.

## Open questions / risks
- TOP risk: any nondeterministic byte in the prefix (a timestamp, a reordered map) silently kills the cache; the
  debug_assert is the guard, but it must cover the WHOLE cached prefix, not a sample. The contract test asserts byte-identity.
- The 20-block window means a single tail breakpoint is insufficient for very long tool loops; budget the 2 reserved breakpoints.
- 1h TTL costs 2x on write; only worth it for sessions >5 min — the opt-in must be conditioned on expected session length, not blanket.

## Next
H-9 (cost cockpit) reads H-2's hit-rate as a live dashboard tile + cache-expiry timer. H-5 (batch) pairs the 1h TTL
H-2 introduces for shared-context batch hit rates. H-6 (masking) relies on HD-2 — it drops trailing observations so
the H-2 byte-stable prefix survives, where LLM summarization would bust it.

End of H-2.
