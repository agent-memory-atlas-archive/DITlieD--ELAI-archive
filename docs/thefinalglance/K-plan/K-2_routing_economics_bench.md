# K-2: routing-economics bench — the industry's first production hybrid-routing $ number (cloud-only vs architect/editor hybrid), under seed, recomputable

PROOF_SPINE: feeds PS-4 (routing economics $/verified-task; the K-2 runner is the buildable spine unit — proof-spine ruling 2026-07-05, K-26 MOVE 6). The deterministic fixture leg ships the runner + recomputable number now; the LIVE $ arm stays usage-gated (RunPod/remote-sidecar W-10 unfunded this window) exactly like PS-2/PS-5 (RunnerPresent is an honest verdict, a fabricated cost is not).

STATUS: PLANNED, wargame-refined 2026-07-11 (K-26 MOVE 6 refine-before-execute). Fixture leg ships now; LIVE cost leg no longer J-28-blocked (J-28 SHIPPED — see GROUNDING) but remains RunPod/remote-sidecar-gated (W-10).

Objective: the research found NO production number for hybrid local/cloud routing savings (only research-grade HERA ~45% offload — KS-2 §1.9); ELAI can own the first. This session BUILDS `elai bench routing-economics`: a fixed task corpus run through two arms (cloud-only vs the H-7 architect/editor hybrid), reporting % steps offloaded, $ delta, and quality delta (verified-OUTCOME rate, NOT pass@k), all under ELAI_SEED and recomputable from the journal (GD-2). Hard boundaries: it does NOT fork I-27's verified-outcome-vs-compute harness (COMPOSE its runner shape, KS-1); the external HERA/CodeAct figures are [DESIGN], never claimed as ELAI's (HD-1); and the LIVE arm emits a real $ number only when a live-priced cloud-vs-hybrid run exists (RunPod/remote-sidecar), else it emits an honest 'cost unavailable — live lane not funded', never a fabricated cost (R1).

Estimated effort: ~1-1.5 sessions (~5-6 h). Deterministic FIXTURE tier in-process now; the LIVE arm is network-gated (remote sidecar / RunPod, W-10).
Pre-requisites: H-7 calibrated routing + I-25 router (A-6 — re-grep cloud_mode.rs CloudModeRouter + providers escalation_policy.rs live, W-7; anchors below). The bench sub-command dispatcher at args_bench.rs (KG-9 — RECON_NEEDED-2, drifted off :75). The safedelta_runner / bench_qdelta runner shape to mirror (KG-4/KG-5). J-28 UsageTurnEvent (SHIPPED, schema_version 272) for the normalized per-lane cost capture the live leg consumes.
Parallel-safe with: K-3, K-4, K-6, K-9 (disjoint files).
Source: K-MASTER-BRIEF §3.2 K-2 [KS-1]; KS-2 §1.9; grounding map KG-5/KG-9/KC-3, HD-1. Doctrine: K-0_INDEX.md.

## Context
Why this matters: cost engineering is the weakest moat (KS-3 report 17) but operationally required, and a MEASURED hybrid-routing number is a first-mover differentiator the industry lacks. Mechanism: a corpus of fixed coding tasks, each run twice (force-cloud vs hybrid), with the journal recording per-step lane + token + cost + verified-outcome; the bench aggregates per-arm $/verified-task. External figures HERA ~45.67% offload and CodeAct ~50%/-60% (KW-4) are [DESIGN] context; the ELAI delta is whatever the bench measures (HD-1).

The 2026-07-11 correction (K-26 MOVE 6): J-28's token ledger is SHIPPED (commit fe069d0; UsageTurnEvent at schema_version 272, usage_ledger + `elai usage` + SpendAccumulator). So the K-2 live leg's ORIGINAL blocker (KC-3, "no normalized per-lane capture exists") is DISCHARGED at the capture layer. The remaining live-arm gate is purely infrastructural: a real cloud-vs-hybrid RUN needs a live cloud lane priced off models.toml AND a local lane (RunPod/remote-sidecar, W-10) — neither funded in the K-26 window. So the runner + the FIXTURE arm (synthetic token/cost rows, deterministic) + the SSB beacon ship now; the LIVE arm is present but returns 'cost unavailable — live lane not funded' until W-10 lands (the honest RunnerPresent-equivalent).

## Existing state (A/B..J coverage) — re-grounded 2026-07-11
- A-6 H-7 routing + I-25 router: cloud_mode.rs:265 `struct CloudModeRouter`, :284 `pub fn route(...)` (uses providers::runtime::RuntimeRoutingPolicy::evaluate; E-31 tier matrix). VERIFIED live 2026-07-11.
- KG-5 I-38 QDELTA runner (bench_qdelta.rs, parse_bench_qdelta_args @ args_bench.rs:1199) + KG-4 safedelta (parse_bench_safedelta_args @ args_bench.rs:902 + benchmarks/safedelta_runner.rs) — the runner shapes to mirror. VERIFIED live.
- KG-9 bench dispatcher args_bench.rs:52-93 (the `if sub == "X" { return parse_bench_X_args(...) }` chain; known subs incl qdelta/safedelta/gate-tax/case-reuse). K-2 adds `routing-economics`, mirroring the gate-tax arm (K-4 precedent @ :64). VERIFIED live (K-2's original ":75" anchor is STALE).
- KC-3 J-28 token ledger — SHIPPED (was PLANNED when K-2 was authored). UsageTurnEvent present in ssb/action_type.rs + payloads.rs + schema.rs at schema_version 272. The normalized-usage capture now EXISTS; the live-arm gate is now W-10 (infra), not J-28.
- SSB current max: RecurrenceHitRateMeasured=270, GateTaxMeasured=271, UsageTurnEvent=272 (flat action_types, one sql_str, no Kind). K-2's new beacon bumps to 273 (RECON_NEEDED-1: re-grep the max at edit; the tree moves).
- I-27 convergence-curve (verified-outcome-vs-compute) — a sibling; COMPOSE the runner shape, do NOT fork.

## Reads (mandatory before coding)
1. _grounding/K-GROUNDING-MAP.md (KG-5/KG-9/KC-3, HD-1).
2. crates/cli/src/bench_qdelta.rs + benchmarks/safedelta_runner.rs (runner shape) + args_bench.rs:52-93 (dispatcher arm shape) + the gate-tax arm (bench_gate_tax.rs + benchmarks/gate_tax_runner.rs) as the freshest flat-beacon precedent.
3. cloud_mode.rs:265-300 CloudModeRouter::route + providers escalation_policy.rs RuntimeRoutingPolicy (A-6 — the routed substrate; re-grep at edit).
4. crates/orchestrator/src/usage_ledger.rs + the UsageTurnEvent payload (J-28 — the normalized per-lane cost capture the live leg consumes; confirm the field names at edit).
5. ELAI_SEED threading + the RRA recompute path (GD-2) for the seeded recomputable requirement (mirror emit_beacon_and_rra from bench_case_reuse / bench_gate_tax).

## Tasks (WAVE KW-2 — the bench). Each task is a MOVE with its observation + failure counter.

1. Define the fixed task corpus: a small set (N=4-6) of representative coding tasks, each with a deterministic verified-outcome oracle; store as a content-keyed fixture (no network, byte-reproducible). Mirror the SAFEDELTA/SeCodePLT oracle shape; do NOT invent a weak proxy.
EXPECTED_OBSERVATION: a fixture module compiles; a unit test asserts the corpus is non-empty, each task has an oracle, and two independent loads are byte-identical (GD-2).
ON_FAIL: corpus loads non-deterministically (HashMap iteration order) -> cause: unordered collection in the fixture -> counter-move: back the corpus with an ordered Vec/BTreeMap and re-assert byte-identity.

2. Build the runner benchmarks/routing_economics_runner.rs (mirror safedelta_runner + gate_tax_runner shape): two arms (force-cloud-everything vs H-7 hybrid via CloudModeRouter::route), per task records lane-per-step + token counts + cost + verified-outcome; aggregates per-arm %offloaded, $/verified-task, quality delta. Pure `compose_*` aggregation fn (deterministic given inputs, like compose_gate_tax) + a RunnerStatus{Measured|RunnerPresent} split so a dry/unfunded live lane reports RunnerPresent, never a fabricated number. ~250-320 LOC. No .unwrap()/.expect() outside #[cfg(test)].
EXPECTED_OBSERVATION: `cargo check -p elai-orchestrator` GREEN; a unit test drives both arms over the fixture and asserts distinct %-offloaded + a RunnerPresent verdict on the unfunded live lane.
ON_FAIL: the hybrid arm offloads 0% (router always picks cloud) -> cause: CloudModeRouter::route called with a cloud-forcing policy/context in BOTH arms -> counter-move: assert the two arms pass DISTINCT RoutingPolicy/context and add a test that the hybrid arm's offload count > 0 on the fixture.

3. Add the `routing-economics` bench sub: the dispatcher arm at args_bench.rs:52-93 (`if sub == "routing-economics" { return parse_bench_routing_economics_args(&rest[1..]); }`, mirror the gate-tax arm @ :64) + a bench_routing_economics.rs CLI handler (mirror bench_gate_tax.rs) + register the sub in the unknown-sub error list. ~60 LOC.
EXPECTED_OBSERVATION: `rg -n "routing-economics" crates/cli/src/args_bench.rs` returns the arm + the error-list entry; `elai bench routing-economics --seed 1234 --json` parses and dispatches (no "unknown bench sub-command").
ON_FAIL: the sub parses but the handler is never reached (arm added after the catch-all) -> cause: arm inserted below the `_ =>` unknown branch -> counter-move: place the arm within the ordered `if sub ==` chain before the error return (grep the line ordering).

4. SSB beacon: add +RoutingEconomicsMeasured flat action_type (mirror GateTaxMeasured @ schema_version 271 exactly: as_sql_str "routing_economics_measured", the from-str arm, the golden action-type lock, a payload struct with the arm/offload/cost fields). Bump to schema_version 273 (RECON_NEEDED-1: re-grep the current max at edit). Emit via emit_beacon_and_rra so the number is journal-recomputable (GD-2), NOT just a blake3-pinned json.
EXPECTED_OBSERVATION: `cargo test -p elai-orchestrator action_type_lock` GREEN after the golden lock regen; a RoutingEconomicsMeasured row lands in ssb.db keyed to the run.
ON_FAIL: the golden action-type lock test fails -> cause: the lock snapshot not regenerated after the +1 -> counter-move: regenerate the golden lock (the sanctioned snapshot-update path) and re-run; confirm the diff is ONLY the new row.

5. GATE the LIVE cost leg on a funded live lane: the runner emits a real $ number ONLY when a live-priced cloud lane + a local lane are present (W-10); absent it, the live arm emits 'cost unavailable — live lane not funded (W-10)' (R1, never a fabricated cost). The fixture arm (synthetic token/cost, deterministic) ships unconditionally and IS the shipped number this window.
EXPECTED_OBSERVATION: with no live endpoint configured, `elai bench routing-economics --json` runner_status=RunnerPresent and the live-arm cost field reads 'cost unavailable', while the fixture arm reports a real recomputable %offloaded + synthetic $/verified-task.
ON_FAIL: the runner emits a $ number off the unfunded lane -> cause: the RunnerPresent gate checks the wrong precondition (e.g. J-28 presence, which is now always true) -> counter-move: gate on live-endpoint presence (the W-10 config), not J-28 presence; add a test asserting 'cost unavailable' when no endpoint is set.

6. NEGATIVE arm `--force-cloud-everything`: shows the savings delta COLLAPSE (the hybrid is the cause of the offload/savings).
EXPECTED_OBSERVATION: a test asserts %offloaded==0 and the savings delta ~0 under --force-cloud-everything, and >0 under the hybrid arm on the same fixture.
ON_FAIL: the negative arm still shows savings -> cause: the flag doesn't actually force the cloud lane in the router call -> counter-move: assert the flag threads into the RoutingPolicy the router sees; test the router decision directly.

7. workflow/reasoning/K-2-decisions.md (the decision log + the per-anchor re-grep results at edit).
EXPECTED_OBSERVATION: `test -f workflow/reasoning/K-2-decisions.md` returns 0.
ON_FAIL: file absent at audit -> cause: skipped -> counter-move: write it (the decision doc is a soft-gate the auditor checks).

## FORKS
FORK-1 (Task 4, SSB): reuse an existing bench/RouteDecisionEvent row family vs +1 RoutingEconomicsMeasured. TRIGGER: at edit, re-grep for an existing row family whose fields already carry {arm, offloaded, cost, verified_outcome}. If one exists and is not overloaded -> reuse (net SSB +0). Else -> +1 flat beacon at schema 273 (the grounded default; the flat-beacon precedent is fresh — GateTaxMeasured 271). J-19 precedent: decide at edit, prefer reuse, never overload an unrelated family.
FORK-2 (Task 5, live arm): live lane funded this window? TRIGGER: is a live cloud endpoint + a local lane (W-10 RunPod/remote-sidecar) configured and reachable at build? YES -> emit the real $ (Measured). NO (the K-26 window default) -> ship fixture-arm-only + the live arm returns 'cost unavailable' (RunnerPresent). Do NOT fabricate a live number to avoid RunnerPresent.
FORK-3 (frontend, W-4): ship an HF-vs-hybrid $/verified-task panel tile now vs CLI-only-for-now. TRIGGER: is the number Measured (FORK-2 YES)? If still RunnerPresent, a panel would render a placeholder — defer the tile (W-6-justified CLI-only) and add it when the live number lands. Grounded default this window: CLI-only.

## RECON_NEEDED (settle at edit, exact checks)
RECON_NEEDED-1: current SSB max schema_version + next flat action_type slot. CHECK: `rg -n "SCHEMA_VERSION 27[0-9]|schema_version = 27[0-9]" crates/orchestrator/src/ssb/` + read the as_sql_str/from-str match tail. (2026-07-11: max=272 UsageTurnEvent; K-2 -> 273. Re-verify; the tree moves.)
RECON_NEEDED-2: the exact dispatcher insertion line + the unknown-sub error list. CHECK: `rg -n "if sub == |unknown .bench. sub-command" crates/cli/src/args_bench.rs` (2026-07-11: chain @ 52-93, error list @ :93). Insert the arm inside the chain, add the sub to the error list.
RECON_NEEDED-3: CloudModeRouter::route signature + how a "force cloud" vs "hybrid" decision is expressed (which RoutingPolicy/context field). CHECK: read cloud_mode.rs:265-300 + providers escalation_policy.rs RuntimeRoutingPolicy at edit; the two arms differ ONLY in this input, never a re-implemented router.
RECON_NEEDED-4: J-28 UsageTurnEvent payload field names for the normalized per-lane cost (local input_tokens semantics vs cloud uncached-suffix). CHECK: read usage_ledger.rs + the UsageTurnEvent payload struct; the live arm consumes these fields, never re-derives cost.

## PRODUCTION_HOOK
seam: crates/cli/src/args_bench.rs:52-93 — the `routing-economics` dispatcher arm -> parse_bench_routing_economics_args -> CliAction -> bench_routing_economics.rs handler -> routing_economics_runner::run -> emit_beacon_and_rra. This is the REAL production entrypoint `elai bench routing-economics`; a #[cfg(test)]/contracts caller NEVER satisfies reachability (R7). The runner's new pub fns are wired into this arm IN THE SAME changeset (walking skeleton: producer+consumer+smoke together).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/benchmarks/routing_economics_runner.rs (the two-arm runner + compose fn + RunnerStatus split)
  - crates/cli/src/bench_routing_economics.rs + the `routing-economics` arm at args_bench.rs:52-93
  - +RoutingEconomicsMeasured SSB flat action_type (schema 273) — or a reused bench/RouteDecisionEvent family (FORK-1, decide at edit)
consumes_symbols:
  - cloud_mode::CloudModeRouter::route + providers::runtime::RuntimeRoutingPolicy (A-6 — the routed substrate; re-greps, never re-implements)
  - benchmarks::safedelta_runner / gate_tax_runner / bench_qdelta (the runner + flat-beacon SHAPE — compose, do not fork I-27)
  - usage_ledger UsageTurnEvent / normalized per-lane cost (J-28, SHIPPED schema 272 — the honest cost capture the LIVE leg consumes)
reachable_from:
  - `elai bench routing-economics [--seed <u64>] [--json] [--force-cloud-everything]` [PRODUCTION CLI]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO $ number on the live leg without a funded live lane (W-10; fabricated cost = R1 violation)
  - NO fork of I-27's verified-outcome harness (compose the runner shape)
  - NO claim of the HERA/CodeAct external figures as ELAI's (HD-1; tag [DESIGN])
  - NO pass@k as the quality metric (verified-OUTCOME rate only)
  - NO .unwrap()/.expect() on the prod runner path
frontend_pair:
  HF-vs-hybrid $/verified-task table on a runtime/cost panel (FORK-3, decide at edit) OR W-6-justified CLI-only-for-now absence (the grounded default while RunnerPresent).
observable_effect:
  trigger: "elai bench routing-economics --seed 1234 --json"
  expect_ssb: "RoutingEconomicsMeasured row (or the reused bench family row) recomputable from the journal under the seed (GD-2), via emit_beacon_and_rra"
  expect_artifact: "one report per arm with %offloaded + $/verified-task + quality delta; the --force-cloud-everything arm shows the delta COLLAPSE; the live arm WITHOUT a funded lane emits 'cost unavailable' rather than a fabricated number"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-2_routing_economics_bench.md --run-trigger
python .claude/tools/production_caller_check.py --files crates/orchestrator/src/benchmarks/routing_economics_runner.rs,crates/cli/src/bench_routing_economics.rs --json   # 0 DARK
rg -n "routing-economics" crates/cli/src/args_bench.rs        # the dispatcher arm + error-list entry exist
rg -n "verified.outcome|verified_outcome" crates/orchestrator/src/benchmarks/routing_economics_runner.rs  # NOT pass@k
rg -c "pass@k|passatk" crates/orchestrator/src/benchmarks/routing_economics_runner.rs   # MUST be 0
rg -n "cost unavailable|RunnerPresent|UsageTurnEvent" crates/orchestrator/src/benchmarks/routing_economics_runner.rs  # the live-lane honest gate is present
test -f workflow/reasoning/K-2-decisions.md
```

## ADVERSARIAL
- If the two arms accidentally share the same RoutingPolicy/context, both offload identically and the headline "%offloaded delta" is a fake 0 that still LOOKS like a measured number. Test: assert hybrid %offloaded strictly > force-cloud %offloaded on the fixture (Task 2 ON_FAIL).
- If the RunnerPresent gate keys on J-28 presence (now always true) instead of live-lane presence, the runner emits a fabricated $ off synthetic tokens and mislabels it Measured. Test: with no live endpoint, runner_status==RunnerPresent AND the live cost field == 'cost unavailable' (Task 5 ON_FAIL).
- If emit_beacon_and_rra is skipped and only a blake3-pinned json is written, the number is not journal-recomputable (GD-2 violation, the K-18a lesson). Test: the RoutingEconomicsMeasured row is queryable from ssb.db and recomputes the report byte-identically under the seed.
- If the golden action-type lock isn't regenerated, the +1 silently fails the lock test at audit. Test: action_type_lock GREEN with a diff of exactly one new row.

## ACCEPTANCE
- [ ] GD-2: every emitted number recomputable from the journal under ELAI_SEED (the SSB beacon row, not just a json).
- [ ] HD-1: the ELAI delta is measured per config; HERA/CodeAct figures tagged [DESIGN].
- [ ] R1 (live honesty): the live $ leg is gated on a funded lane; without it, the runner emits 'cost unavailable', not a number (RunnerPresent).
- [ ] GD-1: composes H-7/I-25/I-38/gate-tax-shape; re-implements none; does not fork I-27.
- [ ] OFF-AGENT PROOF (R4): a RoutingEconomicsMeasured SSB row lands from a real `elai bench routing-economics` run (the agent did not author the row); production_caller_check 0 DARK on the two new files.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli.
- [ ] No new dependency. SSB: net 0..+1, sequential bump from live 272 (FORK-1).

## VERIFY (all off-agent, auditor-run)
VERIFY-1: the RoutingEconomicsMeasured SSB row in ssb.db (sqlite3 count delta from a real CLI run) — the agent authored neither the row nor the assertion.
VERIFY-2: production_caller_check.py 0 DARK + gate_freeze_check OK + maintainability_check 0 BLOCK on the changeset (frozen checkers, auditor-run, never executor-self-run as authority).
VERIFY-3: the INTEGRATION_SMOKE below (real CLI verb on a HARD input) landed its rows + a byte-identical same-seed re-run (GD-2).
VERIFY-4: `cargo test -p elai-orchestrator action_type_lock` GREEN (golden lock regen, diff = one row).

## INTEGRATION_SMOKE
After Tasks 2-4 land (the runner + arm + beacon) AND before DONE: an INDEPENDENT auditor drives the REAL entrypoint `elai bench routing-economics --seed 1234 --json --force-cloud-everything` and then `--seed 1234 --json` (hybrid), on the full fixture corpus (the HARD input, not one toy task), asserting: (a) both arms produce reports; (b) the hybrid %offloaded strictly exceeds the force-cloud arm; (c) a RoutingEconomicsMeasured row per arm landed in ssb.db; (d) the live cost field reads 'cost unavailable' (unfunded lane) NOT a number; (e) a same-seed re-run is byte-identical (GD-2). A single-task happy-path run does NOT satisfy this (R10).

## ABORT (stop and hand off, never improvise)
ABORT-1: the SSB +1 needs a schema change OUTSIDE the flat-beacon pattern (a migration, a new table) — walled-off change, own session (K-26 ABORT-5), not this build.
ABORT-2: CloudModeRouter::route's real signature can't express a "force cloud" vs "hybrid" decision via an input field (it's hardcoded) — that is an A-6 seam defect, not a K-2 task; STOP + handoff, do NOT re-implement the router inside the bench.
ABORT-3: the fixture corpus cannot get a DETERMINISTIC verified-outcome oracle without a live run (the oracle needs network) — then the fixture arm itself is dishonest; STOP + surface (the whole bench's quality metric depends on a real oracle).
ABORT-4: cargo OOM twice on the serial `-j 1` protocol — stop builds, surface (box contention, K-26 BOX RULES).

## Tests to add
- crates/contracts/tests/k2_routing_economics_contract.rs (~90 LOC): (a) two arms produce distinct %-offloaded; (b) the report is recomputable from the seeded journal (the SSB row, GD-2); (c) verified-outcome is the quality metric (no pass@k symbol); (d) the live arm WITHOUT a funded lane yields 'cost unavailable' not a number (R1 degradation case); (e) --force-cloud-everything collapses the delta (R2 negative arm). Each test function carries a marker; the (b) recompute + (d) honesty cases are the load-bearing ones.

## Files touched
- CREATE: crates/orchestrator/src/benchmarks/routing_economics_runner.rs (~300 LOC)
- CREATE: crates/cli/src/bench_routing_economics.rs (~80 LOC)
- MODIFY: crates/cli/src/args_bench.rs (~30 LOC: the `routing-economics` arm in the 52-93 chain + the error-list entry)
- MODIFY: crates/orchestrator/src/ssb/action_type.rs (+RoutingEconomicsMeasured flat, schema 273) + payloads.rs (the payload struct) + tests/action_type_lock.rs (golden regen) — IF +1 (FORK-1)
- CREATE: crates/contracts/tests/k2_routing_economics_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/K-2-decisions.md (~30 LOC)

## Citations / Grounding
- KS-2 §1.9: hybrid-routing production evidence is THIN (HERA ~45.67% offload research-grade only).
- KW-4 CodeAct ~50%/-60% [EXTERNAL/DESIGN]. KW (HERA) [DESIGN].
- ELAI ground truth (re-grounded 2026-07-11): cloud_mode.rs:265/284 (CloudModeRouter::route + RuntimeRoutingPolicy), args_bench.rs:52-93 (bench dispatcher, gate-tax arm @ :64 precedent), ssb/action_type.rs (RecurrenceHitRateMeasured 270 / GateTaxMeasured 271 / UsageTurnEvent 272 flat-beacon precedents), usage_ledger.rs (J-28 SHIPPED, the normalized cost capture).
- CLAUDE.md: HD-1 (measured-per-config), GD-2 (recomputable), R1/R4/R7/R10 (wiring-enforcement), no-cargo-test-unless-permitted (auditor only).

## Open questions / risks
- Quality oracle: verified-outcome requires a real oracle per task — reuse the SAFEDELTA/SeCodePLT oracle shape; a live-run oracle would make the fixture arm dishonest (ABORT-3). Keep the corpus small + frame the fixture number as a PILOT/plumbing number (KD-1), never "the" routing-savings number.
- The live $ number is the actual first-mover differentiator and it stays RunnerPresent until W-10 (RunPod/remote-sidecar) funds a real cloud-vs-hybrid run. This build ships the RUNNER + the honest fixture leg; the live number is a later, infra-gated MOVE.

## REDTEAM (author-drafted 2026-07-11; the AUTHORITATIVE independent adversarial-auditor mode=plan pass runs as the MOVE 6b pre-build plan-audit per D-6 — the author is not the final red-teamer, plan-standard point 7)
- ATTACK A (survives as a guard, not a hole): "the runner will emit a fabricated $ because J-28 now exists so the old KC-3 gate passes." PATCH already in the plan: Task 5 + Task-5 ON_FAIL + the ADVERSARIAL block re-gate the RunnerPresent decision on LIVE-LANE presence (W-10 config), NOT J-28 presence — the stale gate is explicitly the failure mode called out.
- ATTACK B (survives): "%offloaded delta is a fake 0 that looks measured." PATCH: Task 2 ON_FAIL + INTEGRATION_SMOKE (b) assert hybrid strictly > force-cloud; a 0-delta is a test failure, not a silent pass.
- ATTACK C (survives): "the number is a blake3 json, not journal-recomputable." PATCH: emit_beacon_and_rra is mandated (Task 4) + VERIFY-1/VERIFY-3 assert the SSB row recomputes byte-identically (GD-2, the K-18a lesson).
- ATTACK D (open until edit — RECON, not a hole): "the router can't express force-cloud vs hybrid via an input, so the two arms are secretly identical." Not settleable from the plan; RECON_NEEDED-3 names the exact read (cloud_mode.rs:265-300 + escalation_policy.rs) and ABORT-2 is the escape if the router is hardcoded — the landmine has a tripline.
- ATTACK E (failed against the plan): "K-2 forks I-27's harness." The WIRING_CONTRACT consumes_symbols + stub_ban + GD-1 acceptance explicitly compose-not-fork; the runner mirrors safedelta/gate_tax shape, a distinct concern from I-27's convergence curve.

## Next
This produces ELAI's first hybrid-routing economics RUNNER + an honest recomputable fixture number (PS-4 spine unit) and feeds K-1's dossier + K-4's gate-tax context. The LIVE $ differentiator lands when W-10 (RunPod/remote-sidecar) funds a real cloud-vs-hybrid run — a later infra-gated MOVE, not this build.

End of K-2.
