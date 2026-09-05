# J-48: local multi-model serving tier router — coarse admission + warm/cold whole-model lifecycle + busy/cold cloud-spill (ADOPT-the-engine, never build a token scheduler)

STATUS: PLANNED (WAVE LSW-1; effort M; BUILD — the outer policy layer ONLY, the engine owns all fine scheduling)

Objective: ELAI can detect VRAM tiers (vram_tier.rs:6) and pick a backend per role
(runtime_orchestrator.rs:185), and CloudModeRouter (cloud_mode.rs:284) decides local-vs-cloud with
spend caps — but nothing decides, when several local models share one or two GPUs, WHICH tier is
warm, how many concurrent requests a tier may admit, or whether to spill to cloud because the big
tier is cold/busy. This session ships that OUTER policy layer: a LocalServingRouter that owns
(1) route-by-tier with a per-tier max-concurrency cap, (2) a dead-simple warm/cold whole-model
lifecycle (pin small tiers; big tier = EXCLUSIVE mode), (3) busy/cold cloud-spill that rides the
SHIPPED I-7 cascade, (4) coarse FIFO admission with at most 3 priority classes. HARD BOUNDARY it
must NOT cross (this is the entire grounded point — Source verdict): NO continuous batching, NO
chunked prefill, NO KV/paged-cache management, NO token-level preemption, NO MLFQ that re-orders by
estimated tokens. Those belong to the engine (J-19 vLLM). GD-1 do-NOT-re-implement: the engine
adoption is J-19; the VRAM-fit catalog is J-9; the local/cloud decision + spend caps are
CloudModeRouter (B-27/C-5); the local->cloud waterfall is I-7. J-48 consumes all of them.

Estimated effort: ~1 session (~5 h). In-process at gate time for the policy + admission logic;
the live multi-model serving smoke is remote-sidecar/RunPod-gated (W-10 tier split) because it
needs >=2 resident models on real GPUs.
Pre-requisites (re-grep at edit, W-7): CloudModeRouter cloud_mode.rs:265/284/309 SHIPPED;
RuntimeOrchestrator runtime_orchestrator.rs:127/185/235 SHIPPED; VramTier core/src/vram_tier.rs:6 +
classify_vram_tier providers/src/vram_tier.rs:145 SHIPPED; CASCADING ESCALATION (I-7) WIRED;
J-19 (vLLM backend) and J-9 (hardware catalog) PLANNED — J-48 degrades gracefully if either is not
yet shipped (it routes over whatever backends RuntimeOrchestrator exposes today).
Parallel-safe with: J-49 (shares the LocalServingRouter seam but J-49 only adds a per-seat
admission pre-filter in front of it — author the J-48 contract first so J-49 threads it).
Source: workflow/reasoning/steal-local-serving-router-2026-06-18.md (the ADOPT-not-build verdict +
the COMPOSES-WITH ground truth). Doctrine: J-0_INDEX.md.

## Context
Why this matters: the deep-research verdict (Source) is unambiguous — on a 2x24GB box (and at
larger local scales) a custom GPU scheduler + MLFQ reinvents what vLLM/SGLang already do and tend
to do better; the maintainers were still fixing over-admission/KV-thrash (vLLM
scheduler_reserve_full_isl) and priority-scheduling bugs (SGLang prefill/decode disaggregation) in
2025-2026. The ONLY justified custom code is the coarse outer layer. The physical driver of the
warm/cold policy: a 32B-4bit @ 32K needs ~16 GB weights + ~8 GB KV (FP16) = ~24 GB before
overhead, so it does not co-reside with a warm 3B+7B on a single 24GB GPU and on 2x24GB it wants
TP=2 across BOTH GPUs (consuming them), which is why the big tier is modeled as EXCLUSIVE, not
co-resident (Source, numbers directional/HD-1-tagged, NOT measured on our box). Mechanism: the
router is a stateless decision over (requested tier, live tier-residency state, per-tier inflight
count, spend snapshot) returning one of {ServeLocal(tier), Wait, Hibernate(small)+ServeLocal(big),
SpillToCloud} — pure and testable, exactly like CloudModeRouter::route.

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- CloudModeRouter [WIRED] cloud_mode.rs:265 (struct) / :284 (route) / :309 (route_with_spend with
  daily/monthly USD caps). Decides local-vs-cloud only; no concurrency/warm/cold/busy awareness.
- RuntimeOrchestrator [WIRED] runtime_orchestrator.rs:127 / for_role:185 (per-role backend +
  fallback chain) / try_complete:235. One backend per role; no admission, no per-tier cap.
- VramTier [WIRED] core/src/vram_tier.rs:6; classify_vram_tier providers/src/vram_tier.rs:145 —
  tier DETECTION (fit), not multi-model scheduling.
- CASCADING ESCALATION [WIRED, I-7] local->cloud waterfall (EscalationCascade) composing
  CloudModeRouter — the spill path J-48 reuses; J-48 adds the local-busy/cold TRIGGER, not a new
  waterfall.
- CALIBRATED ROUTING [PARTIAL, H-7/E-31] RouteDecisionEvent (3-arm CloudRouteChosen/
  TierModelSwapped/LocalRouteKept), wire-dark unless ELAI_ROUTE_DECISION_SSB_ENABLE. J-48
  prefer-reuses these rows for its decision audit.
- J-19 vLLM-V1 backend [PLANNED] the engine (vLLM #12 on the B-37 trait, APC, AWQ-Marlin). The
  thing that owns fine scheduling. SGLang REJECTED in J-19 (CVE-2026-7304). J-48 routes ABOVE it.
- J-9 hardware model catalog [PLANNED] per-lane per-quant VRAM-fit data — J-48's source of "does
  tier T fit alongside what is currently warm".
- BSP SUPERSTEPS [WIRED, phase-3D] "JoinSet spawn_parallel + VRAM scheduling" — agent-parallelism
  macro-orchestration, NOT model serving. DO NOT extend or confuse with J-48 (stub_ban).
- LOCAL/CLOUD ROUTING POLICY [DARK, E-20] xRouter-derived cost/privacy routing, no callers —
  different concern; J-48 does not fold into it.

## Reads (mandatory before coding)
1. workflow/reasoning/steal-local-serving-router-2026-06-18.md (the verdict + boundary + numbers).
2. J-0_INDEX.md THE WIRING DOCTRINE + DONE DEFINITION + this file's row.
3. crates/orchestrator/src/cloud_mode.rs:200-396 (the route/route_with_spend pattern to MIRROR —
   stateless, returns data not live providers).
4. crates/orchestrator/src/runtime_orchestrator.rs:120-340 (for_role / try_complete; the seam
   J-48's decision feeds).
5. crates/core/src/vram_tier.rs + crates/providers/src/vram_tier.rs:135-160 (tier fit).
6. The I-7 EscalationCascade source (the spill waterfall J-48 triggers).
7. CLAUDE.md hard rules that bite: no .unwrap()/.expect() on prod paths; named constants (the
   per-tier concurrency caps + warm-set size are NAMED consts, never magic numbers); blake3 only.

## Tasks (WAVE LSW-1 — local serving policy)
1. (~110 LOC) Add LocalServingRouter in crates/orchestrator/src/ (new module
   local_serving.rs). A stateless decide() over inputs (requested AgentRole/tier, a
   TierResidency snapshot {warm tiers, per-tier inflight count}, the SpendSnapshotInput already
   used by CloudModeRouter) returning a typed enum LocalServingDecision { ServeLocal{tier},
   AdmitQueued{tier, class}, HibernateThenServe{evict: tier, serve: tier}, SpillToCloud{reason} }.
   Per-tier max-concurrency caps + the warm-set max are NAMED constants. Mirror CloudModeRouter's
   pure-decision shape. HOW NOT: do not touch tokens, batches, or KV — decide() never sees a
   prompt body, only counts and tier ids.
2. (~70 LOC) TierResidency tracker: a small RwLock<BTreeMap<VramTier, TierState>> on the
   RuntimeOrchestrator (warm/cold + inflight counter, bumped on dispatch, decremented on
   completion). EXCLUSIVE-mode invariant: the big tier warm => small tiers forced cold (encode in
   the type, not an ad-hoc check — maintainability dimension 5/2).
3. (~60 LOC) Wire decide() as a PRE-STEP inside RuntimeOrchestrator::try_complete (the production
   seam): before the fallback chain, call LocalServingRouter; on SpillToCloud, hand off to the
   SHIPPED I-7 cascade / CloudModeRouter rather than the local adapter; on AdmitQueued, a coarse
   FIFO with <=3 priority classes (NamedClass enum, not an int knob). This is the production caller
   (D-030): the new pub decide() is reachable from try_complete, not a test.
4. (~30 LOC) Emit the decision audit: prefer-reuse RouteDecisionEvent (E-31, 3-arm) via the
   existing wire-dark gate; only add a LocalServingDecisionEvent variant if the 3 arms genuinely
   cannot carry {tier, decision, reason} (decide at edit — net 0..+1 SSB).
5. (~20 LOC) workflow/reasoning/J-48-decisions.md (what/decision/expected-test/unknowns).

## WARGAME (plan-standard; refine-before-execute 2026-07-22)
MOVE 1 LocalServingRouter::decide pure policy (task 1).
EXPECTED_OBSERVATION: decide() returns ServeLocal when the requested tier is warm and under the named per-tier concurrency cap; returns AdmitQueued at the cap; returns HibernateThenServe when the big tier is requested while a small tier is warm (EXCLUSIVE); returns SpillToCloud with a non-empty reason when the big tier is cold/busy and a cloud spend cap is available; never panics on corrupt negative inflight (safe Wait).
ON_FAIL: decide() emits SpillToCloud with empty reason OR admits past the named cap -> counter-move: pin reason as required field on SpillToCloud + assert named-const caps in j48_local_serving_contract.rs; rewrite until both arms go red when the production branch is deleted (falsification).
MOVE 2 TierResidency on RuntimeOrchestrator (task 2).
EXPECTED_OBSERVATION: RwLock residency tracks warm/cold + inflight; bump on dispatch / decrement on completion; EXCLUSIVE-mode type invariant forces small tiers cold when big tier is warm.
ON_FAIL: small tier stays warm while big is warm -> counter-move: encode EXCLUSIVE in the type/state transition, not an ad-hoc if; contract asserts the invariant.
MOVE 3 try_complete pre-step production hook (task 3) — PRODUCTION_HOOK: runtime_orchestrator.rs::RuntimeOrchestrator::try_complete.
EXPECTED_OBSERVATION: try_complete calls LocalServingRouter::decide before the local fallback chain; SpillToCloud hands off to the shipped I-7 cascade / CloudModeRouter; AdmitQueued uses coarse FIFO with <=3 NamedClass values; production_caller_check reports WIRED for decide().
ON_FAIL: decide() exists but try_complete never calls it (WIRE-DARK) -> counter-move: wire the pre-step; falsify by deleting the call site and watching production_caller_check + the contract go RED.
MOVE 4 decision audit row (task 4).
EXPECTED_OBSERVATION: a RouteDecisionEvent (prefer-reuse E-31) or, only if the 3 arms cannot carry {tier, decision, reason}, one new LocalServingDecisionEvent variant; spill rows carry non-empty reason. No new ELAI_*_ENABLE (ships ON; reuses E-31 gate).
ON_FAIL: new default-OFF flag added -> counter-move: delete it; reuse E-31 gate only.
MOVE 5 decisions doc (task 5).
EXPECTED_OBSERVATION: workflow/reasoning/J-48-decisions.md exists with what/decision/expected-test/unknowns.
ON_FAIL: missing file -> author it before land.
FORK: J-19 vLLM backend not yet shipped at edit -> route over whatever backends RuntimeOrchestrator exposes today (graceful degrade; do NOT invent a vLLM-only path). Trigger = J-19 symbols absent in providers.
FORK: RouteDecisionEvent 3-arm cannot carry {tier, decision, reason} -> add LocalServingDecisionEvent (+1 SSB); else prefer-reuse. Trigger = edit-time payload shape check.
FORKS: none beyond the two above.
RECON_NEEDED: re-grep CloudModeRouter::route / route_with_spend, RuntimeOrchestrator::try_complete, VramTier, I-7 EscalationCascade, and RouteDecisionEvent at edit (W-7); settle whether E-31 arms can carry the audit payload before adding an SSB variant.
RECON_NEEDED: measure cold-start on the live local lane before tightening warm-set/cap constants; ship permissive named defaults until measured (GD-5).
ABORT: any continuous-batching / chunked-prefill / paged-KV / token-preemption / MLFQ code in local_serving.rs (engine owns those — J-19); any re-implementation of CloudModeRouter spend caps or I-7 waterfall; any magic-number concurrency caps; any SpillToCloud with empty reason; any decide() reachable only from #[cfg(test)].
VERIFY: DETERMINISTIC_CHECK below + crates/contracts/tests/j48_local_serving_contract.rs; pass artifacts = production_caller_check WIRED for decide(), wiring_contract_check exit 0, cargo check -p orchestrator green, contract falsification RED-then-GREEN when the try_complete call site is deleted then restored (off-agent, R4 / D-174).
REDTEAM (attacks at authoring): (a) "build a mini-scheduler" — killed by stub_ban + rg count 0 on batch/KV/preempt tokens. (b) "decide() unit-tested but never called from try_complete" — killed by MOVE 3 PRODUCTION_HOOK + production_caller_check + falsify-the-wiring. (c) "spill with empty reason passes the gate" — killed by typed non-empty reason + contract (d). Surviving residue (GD-5): warm/cold numbers are directional until measured on box.
INTEGRATION_SMOKE: in-process drive of RuntimeOrchestrator::try_complete with big-tier marked busy/cold asserts SpillToCloud decision row (RouteDecisionEvent or LocalServingDecisionEvent) with non-empty reason and cloud handoff; live >=2-resident-model smoke remains RunPod/remote-sidecar tier-split (W-10) and does not block the in-process gate leg.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/local_serving.rs::LocalServingRouter (+ decide, LocalServingDecision)
  - crates/orchestrator/src/runtime_orchestrator.rs TierResidency tracker + try_complete pre-step
consumes_symbols:
  - cloud_mode.rs::CloudModeRouter / SpendSnapshotInput (local-vs-cloud + spend caps; GD-1 — NOT re-implemented)
  - runtime_orchestrator.rs::RuntimeOrchestrator::try_complete (the production seam J-48 hooks)
  - core/src/vram_tier.rs::VramTier + providers classify_vram_tier (tier fit)
  - the I-7 EscalationCascade (the spill path J-48 triggers, never re-builds)
  - RouteDecisionEvent (E-31) for the decision audit (prefer-reuse)
reachable_from:
  - runtime_orchestrator.rs::try_complete (PRODUCTION — every per-role completion routes through it)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO continuous batching / chunked prefill / KV or paged-cache management / token-level preemption
    (the engine owns these — J-19; this is the entire ADOPT-not-build boundary)
  - NO MLFQ re-ordering by estimated token count (coarse FIFO + <=3 named classes only)
  - NO re-implementation of CloudModeRouter's local-vs-cloud decision or its spend caps
  - NO extension of the BSP phase-3D "VRAM scheduling" (that is agent-parallelism, not serving)
  - NO magic numbers for concurrency caps / warm-set size (NAMED consts)
  - NO .unwrap()/.expect() on the decide() prod path; NO silent spill (every SpillToCloud carries a reason)
frontend_pair:
  N/A with justification — the live in-process serving gauge + a "tier warm/cold + spilled" surface
  is J-29's read-side territory (UsageWindow); J-48 emits the audit row J-29 already consumes. No
  new Tauri command. Justified.
observable_effect:
  trigger: "a real per-role completion driven through RuntimeOrchestrator::try_complete with the big
    tier marked busy/cold (RunPod/remote-sidecar fixture for the >=2-resident-model case; an
    in-process unit fixture drives decide() + the try_complete pre-step for the gate-time leg)"
  expect_ssb: "a RouteDecisionEvent (or LocalServingDecisionEvent) row with decision=SpillToCloud
    and a non-empty reason when the local big tier is cold/busy and a cloud cap is available"
  expect_artifact: "the completion served from cloud (RoutedProvider::Cloud) when local spilled;
    NEGATIVE case that must FAIL — a SpillToCloud emitted with an empty reason, or a local serve
    that exceeded the per-tier concurrency cap, is a contract failure"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-48_local_serving_tier_router.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/local_serving.rs   # WIRED, not DARK
rg "fn decide" crates/orchestrator/src/local_serving.rs            # the pure decision exists
rg -c "continuous_batch|chunked_prefill|paged_cache|kv_cache|preempt" crates/orchestrator/src/local_serving.rs  # MUST be 0
rg -c "unwrap\(\)|expect\(" crates/orchestrator/src/local_serving.rs  # MUST be 0 (prod path)
test -f workflow/reasoning/J-48-decisions.md
```

## ACCEPTANCE
- [ ] W-1: done = a real try_complete run produced the off-agent decision row, not build-green.
- [ ] W-2: ships ON — no new ELAI_*_ENABLE; reuses the existing RouteDecisionEvent env gate only.
- [ ] R7/D-030: decide() reachable from try_complete (production), proven by production_caller_check.
- [ ] HARD BOUNDARY: zero token/batch/KV/preempt code in local_serving.rs (the rg count is 0).
- [ ] GD-1: CloudModeRouter / I-7 cascade / J-19 engine / J-9 catalog consumed, not restated.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator (+ providers if tier touched).
- [ ] No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: real entrypoint = try_complete; artifact = the decision SSB row. mapped.
- W-2: no default-OFF graveyard — ships ON, reuses E-31 gate. mapped.
- W-3: source verdict reproduced in-repo (the steal doc), not a Downloads-only port. mapped.
- W-5/W-10: live >=2-resident-model smoke is RunPod/remote-sidecar-gated; the decide()+pre-step
  leg runs in-process at gate time. tier-split declared.
- W-6: N/A (no new dashboard surface; J-29 owns the read-side). justified.
- W-7: every anchor above re-greps at edit.
- GD-1: composition list above. GD-4: the spill decision defaults ADVISORY-then-active only after
  its FP/friction is observed (the cap/warm-set start permissive); no gate blocks a call before
  that. GD-5: honest residue — the warm/cold numbers are directional (Source/HD-1), the executor
  measures real cold-start on the box before tightening caps.

## Tests to add
- crates/contracts/tests/j48_local_serving_contract.rs (~90 LOC): (a) decide() returns ServeLocal
  when the tier is warm and under cap; (b) returns AdmitQueued at the cap; (c) returns
  HibernateThenServe for the big tier when a small tier is warm (EXCLUSIVE invariant);
  (d) returns SpillToCloud with a non-empty reason when big tier cold/busy AND a cloud cap exists;
  (e) NEVER spills when air_gapped (defers to CloudModeRouter's AirGappedConflict); (f) corrupt
  input (negative inflight count) degrades to a safe Wait, never a panic.

## Files touched
- CREATE: crates/orchestrator/src/local_serving.rs (~180 LOC)
- MODIFY: crates/orchestrator/src/runtime_orchestrator.rs (~90 LOC: TierResidency + try_complete pre-step)
- MODIFY: crates/orchestrator/src/lib.rs (~2 LOC: mod local_serving)
- CREATE: crates/contracts/tests/j48_local_serving_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/J-48-decisions.md (~25 LOC)

## Citations / Grounding
- Source verdict + numbers: workflow/reasoning/steal-local-serving-router-2026-06-18.md (vLLM
  scheduler_reserve_full_isl over-admission guard; TGI maintenance mode 2026-03-21; Dynamo
  "engine alone suffices for one model on one GPU"; model-swap ~30-120 s vLLM RFC Feb 2026; KV @
  32K 7B~1.75 / 32B~8.0 GiB FP16 — all directional/HD-1-tagged, NOT measured on our box).
- ELAI ground truth: the file:line anchors in Existing state, re-greppable per W-7.
- CLAUDE.md hard rules invoked: no-.unwrap-in-prod; named-constants; blake3-only; one-change.

## Open questions / risks
- Whether the 3-arm RouteDecisionEvent can carry the local serving decision or needs a new variant
  — decide at edit (prefer-reuse; net 0..+1 SSB).
- EXCLUSIVE-mode eviction policy when two big-tier requests queue — start with strict FIFO + a
  single big-tier slot; revisit only if measured contention warrants (GD-4, no premature gate).
- Cold-start numbers are not measured on the 2x24GB box — the executor measures before setting the
  warm-set/cap constants; the plan ships permissive defaults (GD-5 honest residue).

## Next
Closes the "ELAI has routing-as-decision but not routing-as-serving-policy" gap without building a
scheduler. Pairs with J-19 (the engine underneath) and J-9 (the fit catalog feeding it). After
this, J-49 adds the per-seat admission pre-filter in front of decide() for multi-seat.

End of J-48.
