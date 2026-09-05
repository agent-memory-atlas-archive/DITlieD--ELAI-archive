# Q-2: quorum wake-gate — cheap cross-family voters decide whether the expensive lane wakes on a scheduled tick

STATUS: PLANNED (WAVE 2; EXTENSION(O-1 daemon + 4G AutonomousGate + I-25 router + J free-API pool); BUILD-on-shipped — the wake-triage stage; depends O-2 scheduled ticks)

- Track: ORCHESTRATION-ECONOMICS | lever: cross-family-blind-quorum-gates-the-wake | source: steal-fable5-orchestration STEAL 2 (loop 06) | grounding: MODERATE (index-derived, PLAN-asserted) | nature: EXTENSION | effort: M | WAVE 2
- PROOF_SPINE: feeds PS-4 (routing economics $/verified-task — the quiet-tick cost, ~voter-cost vs waking the flagship, is a DIRECT term; the gate makes a scheduled ELAI cheaper without touching correctness).

- Premise (steal-fable5-orchestration STEAL 2): on a scheduled/autonomous tick, N cheap CROSS-FAMILY models independently vote "actionable?" (blind to each other's answers); the expensive conductor wakes ONLY on a majority yes. A quiet tick costs ~voter-cost (near-zero on the J free-API pool) instead of a full flagship wake. The uninstall trigger is MEASURED: when the dispatch ledger shows the wake-precision (wakes-ending-in-action / wakes) is high enough that the filter is redundant, the gate retires ITSELF. The cross-family blindness is the D-13 insight applied to ATTENTION: a single cheap triager makes one family's blind spot the system's; three cross-family voters do not.

## Steal/build
A wake-triage stage in front of the O-1 daemon / any scheduled O-2 TaskSpec that would dispatch the expensive lane: J free-API rotation supplies N cross-family voters, each gets the tick's condensed context and returns a boolean, votes are journaled (`WakeQuorumVoted{tick_id, voter_family, vote}`), and the conductor dispatch is CONDITIONAL on quorum (`WakeGated{tick_id, quorum, woke}`). Voters are advisory-never-the-gate (TD-3): they gate COST, never correctness — a false "not actionable" only DELAYS to the next tick, it never suppresses a verified result. Wake-precision is a recomputable journal metric; the install/uninstall thresholds are declared up front in the decision doc.
EXCLUDED: a single cheap triager (loop 01 shape — cheaper but one family's blind spot becomes the system's; the whole steal is that blindness must be cross-family); voters that can VETO an in-flight verified result (they gate the WAKE, not the output — a wake-gate that can kill a correct answer is a correctness gate wearing a cost-gate mask); same-family voters (D-13 requires cross-family).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-1 research daemon (Cosmos generalization) + O-2 scheduled TaskSpec ticks — the WAKE sites this gates (docs/thefinalglance/O-plan/O-1,O-2).
- 4G `AutonomousGate` (crates/orchestrator/src/autonomous_gate.rs) — advances on deterministic evidence but still RUNS the evaluator model per tick; the wake-gate sits BEFORE that model call.
- I-25 ledger-trained router — routes per-task AFTER wake; the wake-gate is the missing BEFORE-wake stage.
- J free-API rotation pool (cross-family, cost ~0) — the voter supply; D-13 cross-family adversary is the same cross-family principle applied to verification.
- SSB measured-number row pattern (GateTaxMeasured/RecurrenceHitRateMeasured) — the idiom for the recomputable wake-precision metric.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the wake-triage stage on the O-2 scheduled-tick path; N=3 cross-family voters from the J pool; `WakeQuorumVoted` + `WakeGated` journaling; conductor dispatch gated on majority; the wake-precision recompute from the journal.
- DEFER (Tier-2): the SELF-UNINSTALL (auto-retire when measured precision crosses the declared threshold) — land after the journal has enough ticks to measure precision honestly.

## Integration
- crates/orchestrator: the wake-triage stage in the scheduled-tick dispatch path (before the AutonomousGate evaluator call); the voter fan-out over the J free-API pool; the quorum decision + the two SSB emits.
- frontend_pair: N/A with justification — the wake decision surfaces as SSB rows + the existing dispatch-ledger view; no new panel. Justified.

## MOVES (wargame)
M1 add the wake-triage stage: fan out the tick's condensed context to N=3 cross-family voters (J pool), collect blind boolean votes, journal one `WakeQuorumVoted` per voter.
EXPECTED_OBSERVATION: a scheduled tick emits exactly N `WakeQuorumVoted` rows with distinct voter_family values; the voters do not see each other's answers (independent calls).
ON_FAIL: voters share context / are same-family -> the blindness/cross-family invariant breaks -> enforce N distinct families from the J pool + independent dispatch; add the same-family-refusal test.
M2 gate the expensive dispatch on majority: wake only on quorum yes, emit `WakeGated{quorum, woke}`; a no-quorum tick emits `WakeGated{woke:false}` and re-enters the next tick unchanged.
EXPECTED_OBSERVATION: a majority-yes tick wakes the conductor exactly once; a majority-no tick does NOT wake it and leaves only the vote + gate rows; a verified result already in flight is never suppressed.
ON_FAIL: a no-quorum tick still wakes the flagship -> the gate is not load-bearing -> make dispatch strictly conditional on the quorum row; assert the no-wake case.
M3 recompute wake-precision (wakes-ending-in-action / wakes) from the journal; expose it as a measured number with the install/uninstall thresholds recorded.
EXPECTED_OBSERVATION: `elai bench wake-gate` (or the K-2 harness) recomputes wake-precision from `WakeGated` + downstream action rows deterministically; the number is journal-recomputable, not a stored scalar.
ON_FAIL: precision is a stored/asserted scalar -> not recomputable (GD-2) -> derive it from the journal each run; add the recompute test.

## FORK
FORK: if the J free-API pool cannot guarantee N DISTINCT families available at a given tick -> gate on the families that ARE available (min 2 distinct) and journal `WakeQuorumVoted{degraded:true}`; if fewer than 2 distinct families are available, FAIL-OPEN to waking (a cost optimization must never block a wake it cannot cheaply evaluate) and journal the fail-open. Do NOT fall back to a single voter (defeats the cross-family point).

## RECON_NEEDED
RECON_NEEDED-1: the scheduled-tick dispatch seam where the expensive lane is invoked (where the gate inserts) -> settle via `grep -rn "dispatch\|conductor\|wake\|evaluator" .elai_cc/crates/orchestrator/src/autonomous_gate.rs` + the O-1 daemon tick loop.
RECON_NEEDED-2: the J free-API pool API (how to request N cross-family completions cheaply) -> settle via `grep -rn "free.api\|rotation\|provider.pool\|family" .elai_cc/crates/providers/src`.
RECON_NEEDED-3: what counts as "action" downstream of a wake (for the precision denominator) -> settle via the dispatch-ledger/SSB action rows the router already emits; the denominator must be an existing off-agent row, not a new judgment.

## ABORT
ABORT: if implementing the gate would give voters any power to SUPPRESS or VETO an in-flight verified result (not just delay a wake) -> STOP + handoff. The wake-gate gates COST only; a stage that can kill a correct answer is a correctness gate and is the vetoed design (TD-3 / R4).

## WIRING_CONTRACT
produces_symbols:
  - the wake-triage stage on the scheduled-tick path + the cross-family voter fan-out
  - `WakeQuorumVoted` + `WakeGated` SSB rows (baseline+2; re-read at edit W-9)
  - the journal-recomputable wake-precision metric
consumes_symbols:
  - O-1 daemon + O-2 scheduled ticks (the wake sites); 4G AutonomousGate (the evaluator the gate precedes); J free-API pool (voters); I-25 router (post-wake); SSB measured-number idiom; D-13 cross-family principle
reachable_from:
  - a scheduled O-2 tick runs the wake-triage before the expensive dispatch [PRODUCTION scheduler tick]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO single-voter / same-family quorum (cross-family only, min 2 distinct)
  - NO voter veto over an in-flight verified result (gate the wake, never the output)
  - NO stored wake-precision scalar (journal-recomputable only, GD-2)
  - NO silent fail-closed on an empty voter pool (fail-OPEN to waking + journal it)
observable_effect:
  trigger: "advance a scheduled O-2 tick whose context is not actionable, then one whose context is actionable"
  expect_ssb: "N WakeQuorumVoted (distinct families) + one WakeGated per tick; woke=false on the quiet tick, woke=true on the actionable one, dispatched exactly once"
  expect_artifact: "a journal-recomputable wake-precision number; NEGATIVE: a quiet tick that still wakes the flagship, or a voter suppressing an in-flight result, must FAIL"

PRODUCTION_HOOK: producer the wake-triage stage -> consumer the scheduled-tick dispatch seam that invokes the expensive lane (RECON-1), wired same-changeset so a real tick routes through the gate; the `WakeGated` row is the off-agent artifact.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-2_quorum_wake_gate.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = N distinct-family `WakeQuorumVoted` rows + a `WakeGated` row landed off-agent from a real scheduled tick, the quiet tick did NOT dispatch the expensive lane, the actionable tick did (once), and wake-precision recomputes from the journal.

INTEGRATION_SMOKE: an independent agent runs several real scheduled ticks with a mix of actionable/non-actionable contexts against the live daemon path, asserting the expensive lane wakes ONLY on quorum, the quiet-tick cost is ~voter-cost (measured), and an empty-pool tick fails OPEN — a multi-tick run exercising quiet + wake + degraded-pool paths, not one happy wake (R10).

## REDTEAM
REDTEAM: attack that FAILED — "starve the system by having voters always say not-actionable so real work never wakes": FAILS because voters are advisory (TD-3) and a false no only DELAYS to the next tick; the standing-goal (Q-1) reachability predicates and the dispatch-ledger precision metric would surface a systematically-starving gate as low wake-precision, and the empty/degraded-pool path fails OPEN. Attack that SUCCEEDED and its patch — "smuggle a correctness decision into the wake-gate by letting a voter mark an in-flight result unactionable": patched by the ABORT + stub_ban that scope voters to the WAKE only; the gate physically cannot see or suppress an in-flight verified output.

## Grounding
- steal-fable5-orchestration-2026-07-10.md STEAL 2 (loop 06) — the cross-family quorum, the quiet-tick economics, the measured self-uninstall.
- O-1/O-2 scheduler (the wake sites), 4G AutonomousGate (autonomous_gate.rs, the evaluator the gate precedes), J free-API pool, I-25 router, D-13 cross-family adversary — feature-map-confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
O-1 daemon + O-2 scheduled ticks, 4G AutonomousGate, J free-API rotation pool, I-25 router, SSB measured-number rows, the D-13 cross-family principle. Re-implements NONE (GD-1). Pairs with Q-1 (a starving wake-gate surfaces as a violated standing goal) and Q-3 (the advisor is the OTHER cheap-model-in-the-loop economics lever).

End of Q-2.
