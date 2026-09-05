# H-1: Prompt caching on the production path — close the RuntimeAdapter split-brain + activate the dark hit-rate monitor

STATUS: PLANNED (WAVE 1; in-process; ACTIVATION not build; the measured-number foundation with H-2 + H-9)

Objective: ELAI's main coding-agent loop ALREADY prompt-caches (CORRECTION C1, grep-confirmed 2026-06-03):
ProductionAgentRpcServer::handle_infer_request (production_agent_rpc.rs:807) -> AnthropicClient::send_message
(anthropic.rs:440) -> apply_anthropic_cache_markers (anthropic.rs:1026, called unconditionally at :448 send +
:502 stream) stamps CacheControl::ephemeral() on the last system block + last tool def. So H-1 is NOT "add
caching to a markerless path". Two real gaps remain: (a) a SPLIT-BRAIN — the parallel AnthropicApiRuntime::
complete (RuntimeAdapter) path (anthropic_api.rs:101) hardcodes cache_control:None (:137/:163) and ships ZERO
markers for its secondary production callers (hook_prompt.rs:242 architecture hooks, focus_tunnel/orchestrator.rs:
218/280, rlaif_judge) despite supports_caching:true (:189, a false-advertised capability); (b) NO live monitor
verifies caching actually hits — cache_check.rs (measure_cache_hit_rate :32) is wire-dark (no production caller,
gated behind default-OFF ELAI_CRITIC_OF_DESIGN_ENABLE=1, allowlist:39). H-1 unifies marker logic onto the
RuntimeAdapter path and lights up the live hit-rate measurement so "we cache" becomes "we MEASURE the hit rate."

Estimated effort: ~1 session (~4-6 h). Pure in-process; no network/subprocess/RunPod at gate time (W-10 Tier-1 whole).
Pre-requisites: apply_anthropic_cache_markers SHIPPED (anthropic.rs:1026, grep-confirmed); the inference
  entrypoint chain SHIPPED (ipc_accept.rs:485 -> handle_infer_request -> send_message). H-1 wires, does not build.
Parallel-safe with: H-4 (tool catalog / beta header — disjoint concern in the same file; coordinate on anthropic.rs
  edits), H-5/H-8 (disjoint). NOT parallel-safe with H-2 (both touch the caching marker path) — H-1 first, then H-2.
Source: workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.1 (LEVER L1) + §16 ERRATA;
  docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md CORRECTION C1 + the H-1 symbol table. Doctrine: H-0_INDEX.md.

## Context

In CLOUD mode the only real input-cost lever is prompt caching (read = 0.1x = 90% off; ELAI-EFFICIENCY §10.0).
Anthropic bills input and output separately and output is 5x input, so caching discounts ONLY the repeated input
prefix — which is exactly what the coding-agent loop already does via send_message. The efficiency-direction doc's
prior framing ("the production cloud path ships zero cache markers") was an OVERCLAIM; grounding corrected it (C1).
The honest remaining work is (1) the split-brain: the RuntimeAdapter::complete path is a SECOND live production
path (its callers are hooks, focus-tunnel sub-calls, the rlaif judge) that pays full $3/MTok on a prefix it could
pay $0.30 on; and (2) the dark measurement: there is no live consumer of measure_cache_hit_rate, so ELAI cannot
state its own hit rate — which is the believe-the-number control H-9 needs. HD-1 binds the number: it is the
MEASURED hit rate on ELAI's own runs, recomputable from the journal, never a literature figure.

## Existing state (B/C/D/E/F/G coverage)
- apply_anthropic_cache_markers (providers/src/providers/anthropic.rs:1026) — SHIPPED. Stamps CacheControl::
  ephemeral() on last system block + last tool def; called unconditionally from send_message (:448) + stream_message
  (:502). The load-bearing caching path. B-27 grounding comment anthropic.rs:1020-1025 records it was added because
  the prior state was a 0% hit rate. H-1 REUSES it (GD-1; do not re-implement).
- AnthropicApiRuntime::build_request (providers/src/runtime/anthropic_api.rs:101) — WIRE-DARK for caching. Hardcodes
  cache_control:None (:137/:163), never calls apply_anthropic_cache_markers, declares supports_caching:true (:189).
  The split-brain to close.
- ProductionAgentRpcServer::handle_infer_request (orchestrator/src/production_agent_rpc.rs:807) — SHIPPED, the
  coding-agent inference entrypoint (calls send_message :841). Reached via ipc_accept.rs:485, built runtime_builder.rs:737
  + dashboard lib.rs:1340/1431.
- measure_cache_hit_rate (orchestrator/src/cache_check.rs:32) — WIRE-DARK. verdict_code=0 when hit_rate<=0.05 (:41);
  no production caller (only def + cwe_classifier.rs:510 include_str + e10 contract test); default-OFF + allowlisted.
- CumulativeTokenTracker.cache_read_tokens (orchestrator/src/context_budget.rs:777) — SHIPPED. grounding_client.rs:905-906
  records usage.cache_read_input_tokens (non-test). The live cache-read counter H-1's monitor reads.
- The 16 cache_control:None sites in build_tool_catalog (production_agent_rpc.rs:292..:752) — INERT-BY-DESIGN
  (apply_anthropic_cache_markers re-stamps the last tool def downstream). NOT a hazard; H-1 must NOT "fix" them.

## Reads (mandatory before coding)
- docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md CORRECTION C1 + C2 + the H-1 symbol table + the
  WEB VERIFICATION verify:caching-api rows (min-prefix is per-MODEL-ID; pricing).
- workflow/reasoning/ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.1 (L1) + §10.0 (input vs output) + §16/E4.
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\providers\anthropic.rs:440-520,1020-1041 (send + the marker stamper)
- F:\.ELAI_workflow\.elai_cc\crates\providers\src\runtime\anthropic_api.rs:101-189 (build_request, the None split-brain + supports_caching)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\cache_check.rs (full — measure_cache_hit_rate + the verdict thresholds)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\production_agent_rpc.rs:807-845 (handle_infer_request -> send_message)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; HD-1 (measured-not-multiplied), HD-2 (cache-preserving).

## Tasks (WAVE 1 — in-process, activation)

1. **Unify marker logic onto the RuntimeAdapter path (~50 LOC in anthropic_api.rs).** Make AnthropicApiRuntime::
   build_request stamp ephemeral cache_control on the stable prefix (last system block + last tool def) via the
   SAME mechanism as apply_anthropic_cache_markers (call it, or factor the stamper into a shared helper both paths
   call). Remove the unconditional cache_control:None at :137/:163. Honor supports_caching (:189) — a path that
   advertises the capability must act on it. Key the min cacheable prefix on the EXACT model id (Opus 4.8 = 1024,
   NOT 4096 — per-model-id, not a tier). No .unwrap on the hot path.

2. **Activate the live hit-rate monitor on a production path (~40 LOC).** Add a non-test production caller of
   measure_cache_hit_rate that reads the live cache_read_input_tokens / input_tokens from CumulativeTokenTracker
   (context_budget.rs:777) per inference and computes the hit rate. Promote it off the default-OFF
   ELAI_CRITIC_OF_DESIGN_ENABLE gate (W-2: the monitor runs unconditionally; only a kill-switch disables it).
   The hit rate becomes an off-agent artifact.

3. **Surface the hit rate as an off-agent artifact (~30 LOC).** PREFER REUSE (minimise SSB bumps): emit the
   measured hit rate on the existing CloudSpend/token path that H-9 reads (hand the number to H-9's cockpit), OR
   if no existing row carries it, add a single advisory cache_efficiency_sampled row (Pattern-2, baseline+1). The
   number must be recomputable from the journal under ELAI_SEED (GD-2).

4. **Decision doc workflow/reasoning/H-1-decisions.md (~90 LOC).** Records: (a) the split-brain finding (main loop
   caches, RuntimeAdapter does not) and why both paths must stamp; (b) min-prefix keyed per-model-id; (c) the 16
   build_tool_catalog None sites are inert-by-design and intentionally untouched; (d) the measured hit rate is the
   only honest target (HD-1), not the literature 41-80% (which is [WEAK], cross-provider pooled).

## WIRING_CONTRACT
produces_symbols:
  - crates/providers/src/runtime/anthropic_api.rs (RuntimeAdapter cache-marker application — closes the split-brain)
  - crates/orchestrator/src/cache_check.rs (a live production caller of measure_cache_hit_rate)
  - (hit-rate emit) reuse of the H-9 CloudSpend/token path OR crates/orchestrator/src/ssb.rs::SsbActionType::CacheEfficiencySampled (only if reuse impossible)
consumes_symbols:
  - crates/providers/src/providers/anthropic.rs::apply_anthropic_cache_markers (the shipped stamper — REUSE)
  - crates/providers/src/runtime/anthropic_api.rs::AnthropicApiRuntime::build_request (the cache_control:None split-brain)
  - crates/orchestrator/src/cache_check.rs::measure_cache_hit_rate
  - crates/orchestrator/src/context_budget.rs::CumulativeTokenTracker (cache_read_tokens)
  - crates/orchestrator/src/production_agent_rpc.rs::handle_infer_request
reachable_from:
  - crates/orchestrator/src/ipc_accept.rs:485 -> handle_infer_request -> send_message -> apply_anthropic_cache_markers  [PRODUCTION inference path, already caching]
  - crates/orchestrator/src/architecture/hook_prompt.rs:242 + focus_tunnel/orchestrator.rs:218/280 -> AnthropicApiRuntime::complete  [the RuntimeAdapter path H-1 newly marks]
  - the live measure_cache_hit_rate caller -> the off-agent hit-rate artifact  [the activated monitor]
stub_ban:
  - NO re-implementation of apply_anthropic_cache_markers (GD-1 P0 — call/factor the shipped stamper)
  - NO "fix" of the 16 inert build_tool_catalog cache_control:None sites (they are re-stamped downstream)
  - NO default-off ENABLE flag as the monitor's terminal state (W-2 kill-switch only; default ON)
  - NO min-prefix tier assumption ("Opus=4096" is WRONG for Opus 4.8); key on the exact model id
  - NO silent fail-open: a path that cannot stamp a marker logs it, never advertises supports_caching falsely
  - blake3 only; no .unwrap()/.expect() in prod
frontend_pair:
  N/A for H-1 in isolation — the hit rate is surfaced by H-9's cost cockpit (cross-ref H-9). Justification: H-1
  is the producer of the measured hit rate; H-9 owns its dashboard surface + the W-6 dashboard E2E for it.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a real two-turn cloud-mode task that routes through BOTH
    handle_infer_request AND a RuntimeAdapter caller (e.g. an architecture hook), with NO env override>"
  expect_ssb: "the live hit-rate monitor records a non-zero cache-read fraction recomputable from the journal
    (via the H-9 CloudSpend/token path or the cache_efficiency_sampled row); cache_read_input_tokens > 0 on turn 2"
  expect_artifact: "turn 2's RuntimeAdapter-path request carries cache_control ephemeral markers on the stable
    prefix (the off-agent wire artifact, captured by the gate harness), where turn 1 + the pre-H-1 baseline carried
    none; the main-loop path is unaffected (still caches)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-1_prompt_caching_production_path.md --run-trigger
rg "apply_anthropic_cache_markers|cache_control" .elai_cc/crates/providers/src/runtime/anthropic_api.rs   # marker now applied (no bare None on the stable prefix)
rg "measure_cache_hit_rate" .elai_cc/crates/orchestrator/src --glob '!*test*' --glob '!*contract*'        # now has a non-test production caller
rg -c "ELAI_CRITIC_OF_DESIGN_ENABLE" .elai_cc/crates/orchestrator/src/cache_check.rs                       # monitor no longer gated default-OFF for the hit-rate path
rg -c "blake2|sha2" .elai_cc/crates/providers/src/runtime/anthropic_api.rs                                  # MUST be 0
test -f workflow/reasoning/H-1-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: wiring_contract_check.py --run-trigger PASS (produces exist, consumes resolve, reachable_from
  reached by BFS, stub_ban clean, observable_effect RAN + the RuntimeAdapter request carried markers + the monitor
  reported a non-zero hit rate). cargo-green != DONE.
- [ ] W-2 default-ON: the markers stamp + the monitor runs with NO env override; only a kill-switch disables.
- [ ] W-3 production entrypoint: the new marker application fires from a live RuntimeAdapter caller (hook/focus-tunnel),
  not a #[cfg(test)] caller; the monitor fires from the live inference path.
- [ ] GD-1 compose: apply_anthropic_cache_markers is CONSUMED, not re-implemented (grep shows one stamper).
- [ ] GD-2 reproducible: the measured hit rate re-computes byte-identically from the journal under ELAI_SEED.
- [ ] HD-1: the reported number is the MEASURED hit rate, labelled as such; no literature figure quoted as ELAI's.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-providers -p elai-orchestrator.
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: wiring_contract_check --run-trigger is ACCEPTANCE item 1; the produced marker application is consumed on the
  live RuntimeAdapter path (W-8) reached by hook_prompt/focus_tunnel (W-3).
- W-2: the markers + monitor run with no override; only a kill-switch disables (the hit-rate path is off the default-OFF gate).
- W-3: hook_prompt.rs:242 / focus_tunnel/orchestrator.rs:218/280 (RuntimeAdapter) + ipc_accept.rs:485 (main loop); not test-only.
- W-4: N/A own pair (surfaced by H-9); justified.
- W-5: observable_effect is a runnable cloud task -> a real wire artifact + a real journal number; pure in-process.
- W-6: the dashboard E2E for the hit rate is H-9's (cross-ref); H-1 N/A justified.
- W-7: apply_anthropic_cache_markers / anthropic_api.rs / cache_check.rs / handle_infer_request grep-confirmed 2026-06-03 (C1).
- W-8: the marker application (produced) is called on the live RuntimeAdapter path; the monitor caller (produced) consumes measure_cache_hit_rate.
- W-9: prefer reuse (no bump); only if unavoidable +cache_efficiency_sampled baseline+1 (then also bump e45_cross_language_drift_contract.rs:294).
- W-10: entirely Tier-1 in-process (marker stamp + hit-rate read); no network/subprocess/RunPod at gate time.
- GD-1/GD-2/HD-1: compose (reuse the stamper); the number is journal-reproducible + measured-not-borrowed.

## Tests to add
- crates/contracts/tests/h1_caching_production_contract.rs (~140 LOC): (a) build_request output carries an ephemeral
  cache_control on the last system block + last tool def for a RuntimeAdapter request (was None pre-H-1); (b) the
  min-prefix decision is keyed per-model-id (Opus 4.8 -> 1024, Opus 4.6 -> 4096) not a tier; (c) the hit-rate monitor
  computes a non-zero fraction from a fixture with cache_read_input_tokens > 0; (d) the monitor runs with the
  default-OFF critic gate UNSET (the hit-rate path is ungated).
- crates/contracts/tests/fixtures/caching/ (a two-turn fixture exercising both the main loop and a RuntimeAdapter caller).

## Files touched
- MODIFY: crates/providers/src/runtime/anthropic_api.rs (~50 LOC: stamp markers on the stable prefix; honor supports_caching; per-model-id min-prefix)
- MODIFY: crates/orchestrator/src/cache_check.rs (~40 LOC: a live production caller + ungate the hit-rate path)
- MODIFY: crates/orchestrator/src/production_agent_rpc.rs OR grounding_client.rs (~30 LOC: emit/route the measured hit rate to the H-9 path)
- CREATE: crates/contracts/tests/h1_caching_production_contract.rs (~140 LOC)
- CREATE: workflow/reasoning/H-1-decisions.md (~90 LOC)
- (conditional) MODIFY: crates/orchestrator/src/ssb.rs + action_type_lock.rs + e45 contract (only if a new advisory row is unavoidable)

## Citations / Grounding
- ELAI-EFFICIENCY-TRUST-DIRECTION-2026-06-03.md §10.1 (L1, ACTIVATE caching on the production path) + §16/E4
  (the 41-80% figure is [WEAK], cross-provider pooled — the only honest target is the measured hit rate).
- Anthropic prompt caching docs (live 2026-06-03, verify:caching-api): read = 0.1x; min cacheable prefix is
  per-MODEL-ID (Opus 4.8 = 1024, Opus 4.7/4.6/4.5 = 4096); ephemeral is the only cache type.
- ELAI ground truth (C1, grep-confirmed 2026-06-03): apply_anthropic_cache_markers anthropic.rs:1026 (called :448/:502);
  AnthropicApiRuntime::build_request anthropic_api.rs:101 (cache_control:None :137/:163, supports_caching :189);
  measure_cache_hit_rate cache_check.rs:32 (verdict_code=0 when hit_rate<=0.05 :41, no prod caller).
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect() in prod; HD-1/HD-2.

## Open questions / risks
- The RuntimeAdapter::complete path's call FREQUENCY relative to the main loop is unmeasured (hooks/focus-tunnel/rlaif
  may be cold). The split-brain is a correctness fix regardless, but the $ impact of closing it is bounded by that
  frequency — measure it via H-9 before claiming a cost delta (HD-1, GD-5).
- Min-prefix per-model-id means a model swap changes the threshold; the decision must read the live model id, not a const.
- UNVERIFIED until H-9 measures: ELAI's own hit rate has no baseline. Do not claim a number before H-9 reports it.

## Next
H-2 (cache-stable layout) builds on H-1's unified marker path: it makes the marker placement OPTIMAL (byte-stable
prefix verified by bp.valid, tail breakpoint, 1h TTL). H-9 (cost cockpit) is the instrument that turns H-1's
measured hit rate into a dashboard tile + the believe-the-number control paired with G-6 SAFEDELTA.

End of H-1.
