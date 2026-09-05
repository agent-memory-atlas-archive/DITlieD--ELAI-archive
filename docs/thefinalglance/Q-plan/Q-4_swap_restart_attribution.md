# Q-4: never iterate on a swapped model — a mid-unit model substitution restarts the unit fresh and journals a restart-tax row

STATUS: PLANNED (WAVE 2; EXTENSION(4G SUBSTITUTE_MODEL + 4M fallback chains + J free-API rotation); SMALL — doctrine rule + a small enforcement + one accounting row)

- Track: ORCHESTRATION-ECONOMICS | lever: iteration-state-is-model-conditioned | source: steal-fable5-orchestration STEAL 4 (loop 01, exit code 2 "reroute") | grounding: STRONG (autonomous_gate.rs read 2026-07-10; the gap is confirmed in code) | nature: EXTENSION | effort: S-M | WAVE 2
- PROOF_SPINE: guards PS-4/PS-5 attribution (a mis-attributed model outcome corrupts BOTH the routing-economics and the case-reuse benches; not a direct feeder — an integrity guard on the numbers Q-2/Q-3/K-2/K-18 produce).

- Premise (steal-fable5-orchestration STEAL 4): when the runtime swaps the model mid-unit (a fallback chain, a rotation, a substitution), the loop must NOT continue iterating the new model on the OLD model's partial state. Iteration state (partial diffs, self-consistency with earlier turns, implicit style commitments) is model-CONDITIONED; a swapped model inheriting it produces neither model's behavior and POISONS attribution — which model earned/lost trust-ledger credit (F-6, I-25 training rows) becomes unanswerable. ELAI's ledger-trained router and per-skill trust tiers REQUIRE clean per-model attribution, so this is not cosmetic.
- GROUNDED gap (autonomous_gate.rs, read 2026-07-10): `AutonomyDecision::SubstituteModel{from,to}` fires on `max_restarts_exhausted`; `substitute_model()` swaps `active_model` and emits a `ModelSwap` SSB row. Nothing at the gate layer discards model-conditioned iteration state or records a restart tax — the swap just repoints `active_model`. Whether the CONSUMER (task_runner) restarts the unit fresh or continues in-place is NOT established at the gate; that is the exact RECON below. If restart-fresh already holds in the consumer, Q-4 collapses to the doctrine sentence + the accounting row; if not, it adds the restart enforcement.

## Steal/build
On `AutonomyDecision::SubstituteModel`, the unit RESTARTS from unit-start under the new model, carrying the I-11 condensed BeliefState as the ONLY inherited context (never the raw partial diffs / turn history), and a `ModelSwapRestart{from, to, unit_id, discarded_turns}` row is journaled so PS-4 cost accounting sees the restart TAX honestly (a swap is not free — the discarded work is a real cost the routing bench must count).
EXCLUDED: carrying raw iteration state (partial diffs, prior-turn self-consistency) across the swap (the whole point — that state is the poison); a swap that leaves the restart UNJOURNALED (then the routing bench under-counts the swap's true cost and PS-4 lies); treating this as a new SUBSTITUTE mechanism (it is a CONSTRAINT on the existing 4G one, not a second path).

## Existing ground truth (autonomous_gate.rs, read 2026-07-10 — STRONG)
- `AutonomyDecision::SubstituteModel { from: String, to: String }` (autonomous_gate.rs ~L1032) — emits a `ModelSwap` SSB row (comment L1029-1031), fires from the decide() ladder step 4 on `ctx.max_restarts_exhausted` (~L1293-1315).
- `AutonomousGate::substitute_model()` (~L1435) swaps `active_model` to the next fallback and returns `(from,to)`; `active_model()` (~L1428) exposes the head. VRAM-guarded via `ModelPool::can_fit_llm` before the swap.
- `EscalationTrigger::SubstituteModelFailed` (~L597) fires when the fallback chain is exhausted or the target won't fit VRAM.
- 4M per-role fallback chains + J free-API rotation — the OTHER swap sources the same restart doctrine binds (a rotation is a swap).
- I-11 BeliefState (<=2K) — the ONLY context carried across the restart.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the restart-on-swap enforcement in the task_runner consumer of `AutonomyDecision::SubstituteModel` (discard model-conditioned iteration state, re-enter the unit at unit-start with the BeliefState handoff) + the `ModelSwapRestart` accounting row + the doctrine line in Q-0_INDEX.
- DEFER (Tier-2): threading the same restart constraint through the J-rotation and 4M-fallback swap paths if they are separate consumer sites (land once RECON maps them).

## Integration
- crates/orchestrator: the task_runner (or the AutonomyDecision consumer) handling of `SubstituteModel` — the restart-fresh branch + the state-discard + the `ModelSwapRestart` emit.
- frontend_pair: N/A with justification — the restart surfaces as the `ModelSwapRestart` SSB row + the existing ModelSwap trail; no new panel. Justified.

## MOVES (wargame)
M1 in the `AutonomyDecision::SubstituteModel` consumer, on swap DISCARD the unit's model-conditioned iteration state and re-enter the unit at unit-start under `to`, carrying ONLY the I-11 BeliefState.
EXPECTED_OBSERVATION: after a swap, the new model's first turn sees the BeliefState handoff and NOT the old model's partial diffs/turn history; the unit's turn counter resets.
ON_FAIL: the new model inherits the old partial diffs -> attribution poison -> discard the iteration state at the swap boundary; assert the new model's turn-0 context excludes the prior turns.
M2 journal a `ModelSwapRestart{from, to, unit_id, discarded_turns}` row at the restart so the swap's discarded work is a counted cost.
EXPECTED_OBSERVATION: every `ModelSwap` that restarts a unit is accompanied by exactly one `ModelSwapRestart` row with discarded_turns>0; the K-2/PS-4 accounting reads it.
ON_FAIL: a restart lands with no accounting row -> the routing bench under-counts swap cost -> emit `ModelSwapRestart` at the restart boundary; assert the pairing with `ModelSwap`.
M3 add the doctrine line to Q-0_INDEX (and cross-link 4G/4M): "a model substitution restarts the unit fresh; iteration state is model-conditioned and never inherited across a swap."
EXPECTED_OBSERVATION: Q-0_INDEX carries the doctrine line; the swap-restart binds every swap source (4G fallback, 4M chain, J rotation) named there.
ON_FAIL: the doctrine covers only 4G -> J-rotation swaps still inherit state -> name all three swap sources in the doctrine; RECON maps their consumer sites.

## FORK
FORK: if RECON-1 shows the task_runner consumer ALREADY restarts the unit fresh on `SubstituteModel` (discards iteration state) -> Q-4 collapses to M2 (the `ModelSwapRestart` accounting row) + M3 (the doctrine line); the enforcement (M1) is already satisfied and is recorded as ALREADY in the decision doc. If it continues in-place -> ship M1 in full.

## RECON_NEEDED
RECON_NEEDED-1 (the load-bearing one): the task_runner / conductor handling of `AutonomyDecision::SubstituteModel` — does it restart the unit fresh or continue iterating? -> settle via `grep -rn "SubstituteModel\|AutonomyDecision\|active_model" .elai_cc/crates/orchestrator/src/task_runner.rs` + codegraph_callers on `SubstituteModel` BEFORE any edit. This decides whether Q-4 is a full unit or a doctrine+row.
RECON_NEEDED-2: whether 4M fallback + J rotation swaps go through the same `SubstituteModel` consumer or separate sites -> settle via `grep -rn "fallback\|rotation\|swap\|substitute" .elai_cc/crates/providers/src .elai_cc/crates/orchestrator/src`.
RECON_NEEDED-3: the I-11 BeliefState handoff API for the restart context -> settle via `grep -rn "BeliefState\|handoff\|condense" .elai_cc/crates`.

## ABORT
ABORT: if restarting the unit fresh on every swap would create an infinite swap-restart loop (swap -> restart -> exhaust -> swap ...) -> STOP + flag; the restart must compose with the existing `SubstituteModelFailed` escalation (chain-exhausted -> Escalate, not loop). The restart is bounded by the SAME fallback-chain length the gate already enforces; a restart that outlives the chain is the vetoed design.

## WIRING_CONTRACT
produces_symbols:
  - the restart-fresh branch in the `AutonomyDecision::SubstituteModel` consumer (discard iteration state + BeliefState-only re-entry)
  - `ModelSwapRestart` SSB row (baseline+1; re-read at edit W-9)
  - the swap-restart doctrine line in Q-0_INDEX (binds 4G/4M/J swap sources)
consumes_symbols:
  - 4G `AutonomyDecision::SubstituteModel` + `substitute_model()` + the `ModelSwap` row (autonomous_gate.rs); 4M fallback chains; J rotation; I-11 BeliefState; the `SubstituteModelFailed` escalation (the bound)
reachable_from:
  - a real max_restarts_exhausted swap in an autonomous run restarts the unit under the new model [PRODUCTION autonomous gate -> task_runner]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO inheriting raw iteration state (partial diffs / turn history) across a swap
  - NO unjournaled restart (a `ModelSwapRestart` row is mandatory — else PS-4 under-counts)
  - NO second substitute mechanism (constrain the existing 4G path, do not add one)
  - NO unbounded swap-restart loop (bounded by the fallback chain / `SubstituteModelFailed`)
observable_effect:
  trigger: "drive an autonomous unit to max_restarts_exhausted so the gate returns SubstituteModel, on a live run"
  expect_ssb: "a ModelSwap row followed by a ModelSwapRestart{from,to,discarded_turns>0}; the new model's turn-0 context excludes the prior model's turns"
  expect_artifact: "the unit re-run under the new model with only the BeliefState carried; NEGATIVE: the new model iterating on the old model's partial diffs, or a restart with no ModelSwapRestart row, must FAIL"

PRODUCTION_HOOK: producer the restart-fresh branch + `ModelSwapRestart` emit -> consumer the task_runner site that handles `AutonomyDecision::SubstituteModel` (RECON-1), wired same-changeset; the `ModelSwapRestart` row (paired with the existing `ModelSwap`) is the off-agent artifact.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-4_swap_restart_attribution.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a real swap emitted a `ModelSwap`+`ModelSwapRestart` pair off-agent, the new model's first turn context excluded the prior model's turns, and the restart was bounded by the fallback chain (no loop).

INTEGRATION_SMOKE: an independent agent drives a real autonomous unit through a genuine model substitution (exhaust max_restarts on a HARD input so the fallback fires), asserting the unit restarts fresh under the new model with only the BeliefState, a `ModelSwapRestart` lands, and a chain-exhausted case escalates via `SubstituteModelFailed` rather than looping — the real gate->task_runner->swap path, not a mocked decision (R10).

## REDTEAM
REDTEAM: attack that FAILED — "corrupt the trust ledger by swapping mid-unit so a good model gets blamed for a bad model's partial diff": FAILS because M1 discards the model-conditioned state at the swap boundary and M2's `ModelSwapRestart` row makes the swap + its discarded turns explicit, so F-6/I-25 attribute each turn to the model that actually produced it. Attack that SUCCEEDED and its patch — "hide the true cost of thrashing by swapping repeatedly without recording the wasted work": patched by the mandatory `ModelSwapRestart{discarded_turns}` row that feeds PS-4/K-2 so the routing bench counts the restart tax; a swap that under-counts its cost cannot pass the accounting recompute.

## Grounding
- steal-fable5-orchestration-2026-07-10.md STEAL 4 (loop 01, exit-code-2 reroute) — the model-conditioned-state argument + the attribution-integrity rationale.
- autonomous_gate.rs read 2026-07-10: `SubstituteModel{from,to}` (~L1032), `substitute_model()` (~L1435), `ModelSwap` row (L1029-31), the decide() ladder step-4 swap-on-max_restarts_exhausted (~L1293-1315), `SubstituteModelFailed` (~L597). The consumer restart behavior is the RECON-1 unknown.
- 4M fallback, J rotation, I-11 BeliefState, F-6/I-25 attribution — feature-map-confirmed, executor re-greps (W-7).

## COMPOSES-WITH
4G AutonomousGate `SubstituteModel`/`substitute_model()`/`ModelSwap`/`SubstituteModelFailed`, 4M fallback chains, J rotation, I-11 BeliefState, F-6 trust ledger + I-25 router (the attribution consumers this protects). Re-implements NONE (GD-1). Guards the per-model attribution that Q-3's same-executor invariant and the PS-4/PS-5 benches all depend on.

End of Q-4.
