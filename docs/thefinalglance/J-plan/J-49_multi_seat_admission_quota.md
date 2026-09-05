# J-49: multi-seat admission + per-seat quota / fair-share / spend isolation (the documented "revisit only if multi-tenant" trigger firing; raw tokens + USD, no synthetic credit unit)

STATUS: PLANNED (WAVE LSW-2; effort M-L; EXTENSION+BUILD — activates the dormant UsageTurnEvent.identity field; depends J-48)

Objective: ELAI is single-user today — every usage row carries an identity field defaulting to
"local" (J-28 UsageTurnEvent) and per-user multi-tenant accounting was REJECTED in the 2026-06-12
goose ledger WITH an explicit revisit trigger ("revisit only if multi-tenant"). The user made
multi-seat an explicit goal on 2026-06-18, which IS that trigger. This session ships the multi-seat
leg: a SeatRegistry (data-driven — present = multi-seat active, absent = today's single "local"
seat, so NO new ELAI_*_ENABLE graveyard, W-2), a per-seat admission pre-filter in FRONT of J-48's
LocalServingRouter::decide (fair-share + per-seat concurrency + per-seat spend ceiling reusing
CloudModeRouter's daily/monthly cap machinery), and per-seat aggregation keyed off the existing
identity field. HARD BOUNDARY (re-proposing a rejection is a P0 — this stays inside the revisit's
scope): NO synthetic credit / billing display unit (that line stays REJECTED — raw tokens + USD
only); NO OpenRouter-style dynamic registry (I-51 static models.toml canonical); the fair-share
algorithm is a simple weighted token-bucket, NOT a new scheduler. GD-1 do-NOT-re-implement: the
serving decision is J-48; the local/cloud spend caps are CloudModeRouter (C-5); the usage capture +
identity field + repricing is J-28; the read-side window is J-29.

Estimated effort: ~1-2 sessions (~7 h). In-process at gate time (the admission pre-filter + the
token-bucket + per-seat aggregation are pure logic over recorded rows); the live multi-seat smoke
(two concurrent identities contending on a shared local tier) is remote-sidecar/RunPod-gated (W-10).
Pre-requisites (re-grep at edit, W-7): J-48 LocalServingRouter SHIPPED (this sits in front of it);
J-28 UsageTurnEvent + identity field + SpendAccumulator (repriced off models.toml) SHIPPED;
CloudModeRouter spend caps cloud_mode.rs:309/683-691 SHIPPED; J-29 UsageWindow SHIPPED (the per-seat
read-side rides it).
Parallel-safe with: none in this wave (depends J-48; touches the same try_complete pre-step seam).
Source: workflow/reasoning/steal-local-serving-router-2026-06-18.md (the REJECTED-ledger revisit
analysis + the gateway-owns-fairness boundary). Doctrine: J-0_INDEX.md.

## Context
Why this matters: the deep-research verdict (Source) places multi-tenant fairness, per-user quota,
and isolation squarely in the GATEWAY layer, NOT in a GPU scheduler — exactly where ELAI's spend
caps + usage ledger already live. The well-understood failure mode the gateway must prevent is one
tenant flooding the shared queue and starving the others; the standard, boring fix is a weighted
token-bucket / fair-queue at admission, which is what this ships (the Source explicitly warns
against anything fancier at this scale). The ledger discipline is the load-bearing constraint here:
the goose verdict (ELAI-FEATURE-MAP line ~1710) REJECTED per-user multi-tenant accounting but kept
identity as a dormant FIELD and named the precise revisit condition; J-49 is that revisit, so it
ACTIVATES the dormant field and reuses the spend machinery rather than inventing accounting, and it
leaves the separately-rejected "abstract credit/billing display unit" rejected (raw tokens + USD).

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- UsageTurnEvent [J-28] uniform per-turn row carrying an identity field default "local" — the
  dormant multi-seat hook this activates (ELAI-FEATURE-MAP line ~1711).
- SpendAccumulator + daily/monthly USD caps [WIRED, C-5] cloud_mode.rs:309 (route_with_spend) /
  683-691 (repriced off models.toml by J-28). Per-seat spend ceiling reuses this, keyed by identity.
- J-48 LocalServingRouter::decide [PLANNED, this wave's prerequisite] — J-49's admission pre-filter
  runs in FRONT of it (seat-fair-share first, then tier-serving decision).
- J-29 UsageWindow [PLANNED] per-model/task/session/lane read-side — extended with a per-seat
  breakdown (the read-side rides J-29, no new Tauri surface invented here).
- REJECTED ledger (ELAI-FEATURE-MAP line ~1700/1710): "abstract credit/billing display unit over
  tokens: REJECTED ... revisit only if multi-tenant"; "per-user multi-tenant accounting: REJECTED
  (single-user; identity stays a FIELD ... default 'local')". J-49 = the conditional revisit; the
  credit-unit rejection STANDS.

## Reads (mandatory before coding)
1. workflow/reasoning/steal-local-serving-router-2026-06-18.md (the revisit analysis + boundary).
2. docs/thefinalglance/J-plan/J-48_local_serving_tier_router.md (the decide() contract this fronts).
3. J-0_INDEX.md WIRING DOCTRINE + DONE DEFINITION + this file's row + the REJECTED-ledger note.
4. crates/orchestrator/src/cloud_mode.rs:300-396 + 683-691 (spend caps + repricing to reuse per-seat).
5. The J-28 UsageTurnEvent struct + identity field (the dormant field to activate).
6. CLAUDE.md hard rules: no synthetic-credit unit (ledger); no .unwrap on prod; named constants
   (token-bucket rate/burst + per-seat caps are NAMED consts); blake3 only.

## Tasks (WAVE LSW-2 — multi-seat admission)
1. (~80 LOC) SeatRegistry in crates/orchestrator/src/ (seat_registry.rs): a data-driven map of
   seat-id -> {weight, per-seat concurrency cap, per-seat daily/monthly USD cap}. ABSENT registry =
   one implicit "local" seat with today's behavior (so multi-seat activates by DATA, not by an
   enable-flag — W-2). Source = a seats.toml (mirrors I-51 models.toml shape), inventory-validated.
2. (~70 LOC) A weighted token-bucket fair-share admission pre-filter, run in
   RuntimeOrchestrator::try_complete BEFORE J-48's decide(): identity over cap or bucket-empty =>
   AdmitQueued/Deny for THAT seat only (never starves others). Per-seat spend ceiling reuses
   CloudModeRouter's cap check, keyed by identity (NOT a new cap engine).
3. (~40 LOC) Per-seat aggregation: roll up the existing UsageTurnEvent rows by identity (pure read
   over recorded rows — DSR/G-5, derived not recalled). Extend J-29 UsageWindow with the per-seat
   breakdown; no new dashboard command.
4. (~25 LOC) Emit a SeatAdmissionEvent OR prefer-reuse the J-48 decision row with the identity
   discriminator (decide at edit — net 0..+1 SSB; the row must carry seat-id + admit/deny + reason).
5. (~20 LOC) workflow/reasoning/J-49-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/seat_registry.rs::SeatRegistry (+ the weighted token-bucket admitter)
  - runtime_orchestrator.rs try_complete per-seat admission pre-filter (in front of J-48 decide)
  - J-29 UsageWindow per-seat breakdown (read-side extension)
consumes_symbols:
  - J-48 local_serving.rs::LocalServingRouter::decide (the serving decision this fronts; GD-1)
  - cloud_mode.rs spend-cap check + SpendAccumulator (per-seat ceiling; NOT a new cap engine)
  - J-28 UsageTurnEvent.identity (the dormant field activated; NOT a new accounting schema)
  - J-29 UsageWindow (the read surface; NOT a new dashboard command)
reachable_from:
  - runtime_orchestrator.rs::try_complete (PRODUCTION — admission runs before every completion)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO synthetic credit / abstract billing display unit (ledger REJECTION stands — raw tokens + USD)
  - NO OpenRouter-style dynamic model registry (I-51 static models.toml canonical — ALREADY-REJECTED)
  - NO new cap engine (reuse CloudModeRouter's daily/monthly check keyed by identity)
  - NO MLFQ / fancy scheduler (weighted token-bucket fair-queue only — Source)
  - NO new ELAI_*_ENABLE (multi-seat activates by the presence of seats.toml, W-2)
  - NO key/PII in the SSB payload (seat-id is an opaque handle; payload never carries seat secrets)
  - NO .unwrap()/.expect() on the admission prod path
frontend_pair:
  N/A with justification — the per-seat view is a breakdown of J-29's already-shipped UsageWindow
  surface; no new Tauri command. Justified.
observable_effect:
  trigger: "two concurrent identities driving completions through try_complete on a shared local
    tier, one over its per-seat cap (RunPod/remote-sidecar fixture for the live contention case; an
    in-process unit fixture drives the admitter + per-seat rollup for the gate-time leg)"
  expect_ssb: "a SeatAdmissionEvent (or J-48 decision row + identity) showing the over-cap seat
    AdmitQueued/Deny while the under-cap seat serves — with seat-id + reason"
  expect_artifact: "the per-seat UsageWindow rollup reflects both identities' token+USD totals;
    NEGATIVE case that must FAIL — one seat's flood causing another seat's request to be denied
    (fair-share violated), or any synthetic credit unit appearing in the rollup, is a contract failure"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/seat_registry.rs   # WIRED, not DARK
rg "token_bucket|fair" crates/orchestrator/src/seat_registry.rs        # the fair-queue exists
rg -c "credit|billing_unit|ELAI_MULTI_SEAT_ENABLE" crates/orchestrator/src/seat_registry.rs  # MUST be 0 (ledger + W-2)
rg -c "unwrap\(\)|expect\(" crates/orchestrator/src/seat_registry.rs   # MUST be 0 (prod path)
test -f workflow/reasoning/J-49-decisions.md
```

## ACCEPTANCE
- [ ] LEDGER: no synthetic credit unit; identity-field activated, not a new accounting schema
      (the rejection's exact boundary honored).
- [ ] W-2: multi-seat activates by seats.toml presence, NOT a new ELAI_*_ENABLE (rg count 0).
- [ ] FAIR-SHARE: one seat's flood never denies another seat (the negative test fails on violation).
- [ ] R7/D-030: the admitter is reachable from try_complete (production), not a test.
- [ ] GD-1: J-48 decide / CloudModeRouter caps / J-28 identity / J-29 window consumed, not restated.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator.
- [ ] No new dependency (seats.toml parsed with the existing toml dep).

## WIRING DONE-GATE (clause by clause)
- W-1: real entrypoint = try_complete admission; artifact = the SeatAdmissionEvent row + per-seat rollup.
- W-2: no enable-flag graveyard — data-driven activation by seats.toml. mapped.
- W-3: the revisit analysis + ledger citation reproduced in-repo (the steal doc). mapped.
- W-5/W-10: live two-seat contention smoke is RunPod/remote-sidecar-gated; the admitter + rollup
  run in-process at gate time. tier-split declared.
- W-6: N/A (per-seat view is a J-29 breakdown, no new surface). justified.
- W-7: every anchor re-greps at edit.
- GD-1: composition list above. GD-4: per-seat caps default permissive; the deny path activates
  only after its FP/friction is observed across >=3 real multi-seat sessions. GD-5: honest residue
  — fair-share weights are unmeasured until a real two-seat workload exists; ship equal weights.

## Tests to add
- crates/contracts/tests/j49_multi_seat_contract.rs (~90 LOC): (a) absent seats.toml => single
  "local" seat, behavior identical to pre-J-49; (b) two seats, equal weight, fair-share holds under
  a one-seat flood; (c) a seat over its per-seat USD cap is denied while others serve; (d) per-seat
  rollup sums the right identity's rows; (e) NO synthetic credit unit in any output; (f) corrupt
  seats.toml degrades to the single-seat default, never a panic.

## Files touched
- CREATE: crates/orchestrator/src/seat_registry.rs (~150 LOC)
- MODIFY: crates/orchestrator/src/runtime_orchestrator.rs (~50 LOC: admission pre-filter)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: mod seat_registry)
- MODIFY: the J-29 UsageWindow read-side (~30 LOC: per-seat breakdown) — re-grep its path at edit
- CREATE: crates/contracts/tests/j49_multi_seat_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-49-decisions.md (~25 LOC)

## Citations / Grounding
- Source: workflow/reasoning/steal-local-serving-router-2026-06-18.md (gateway-owns-fairness
  boundary; the REJECTED-ledger revisit analysis with the exact ELAI-FEATURE-MAP line cites).
- ELAI ground truth: the file:line anchors in Existing state, re-greppable per W-7.
- CLAUDE.md hard rules invoked: ledger-rejection-respect (no credit unit); W-2 (no enable-flag);
  DSR/G-5 (derived per-seat rollup, never recalled); no-.unwrap-in-prod; named-constants.

## Open questions / risks
- Whether to reuse the J-48 decision row + identity vs a dedicated SeatAdmissionEvent — decide at
  edit (prefer-reuse; net 0..+1 SSB).
- Fair-share weights are policy, not measured — ship equal weights, expose in seats.toml, tune only
  on a real multi-seat workload (GD-5).
- Seat identity provenance (where the seat-id comes from on the local/CLI path vs a future
  multi-client path) — the J-2 elai-as-MCP-server control plane is the likely client seam; J-49
  takes identity as an input and does not invent the auth (defer the auth source to J-2 / a future
  plan, named here so it is not silently assumed).

## Next
Closes the multi-seat half of the local-serving story by activating the dormant identity field
under the documented revisit trigger. Pairs with J-48 (the serving decision it fronts) and J-2 (the
external-client control plane that would supply real seat identities). The "abstract credit unit"
rejection remains in force.

End of J-49.
