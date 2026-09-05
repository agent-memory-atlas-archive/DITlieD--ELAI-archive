H-28-WIRE-1 — wire the reward-tampering gate into the darwin admit loop

authored 2026-07-07 by the opus-walk controller (planning-time judgment; the free glm worker executes).
class: ACTIVATION (verified). the guard EXISTS + is CLI-proven + is in the frozen gate-manifest; the
production seam EXISTS; they are simply not wired together. this is NOT greenfield (contrast H-29, whose
live-trading-resume FSM edge does not exist yet — do NOT seed H-29).
scope: .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs only (+ one wire-dark-allowlist row, one
unit test, one smoke). single crate. blast radius verified low (impact_radius 0 at-risk; check_reward_tampering
has ZERO production callers today outside the CLI diagnostic).

PROOF_SPINE: feeds PS-6 (wire-dark rate) — converts a built-but-CLI-only capstone gate (H-28) into a wired
production caller; secondary: proves the free-worker IMPLEMENT leg of the opus walk (the one unproven thing).

================================================================================
GROUNDING LEDGER (verified from live source 2026-07-07; cite, do not re-derive)
================================================================================
GROUNDED: the guard fn exists and already emits the off-agent artifact:
  .elai_cc/crates/orchestrator/src/reward_tampering.rs:303
  pub async fn check_reward_tampering(ssb:&Arc<Ssb>, _db_path:&Path, expected_harness_blake3:&str,
    observed_harness_blake3:&str, prior_reward:f64, new_reward:f64, band:f64, run_id:&str, seed:u64)
    -> Result<RewardTamperingOutcome, RewardTamperingError>
  it calls classify_reward_tampering(...) then emit_outcome(ssb,...) which writes ONE
  SsbPayload::RewardTampering row (reward_tampering.rs:282 "the off-agent done-gate artifact, R4").
  DEFAULT_PLAUSIBLE_REWARD_BAND is a pub const in the same module (used by crates/cli/src/reward_tamper_cmd.rs:34).
GROUNDED: the seam exists and an Ssb is in scope there:
  .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs ~1354-1358 — GuardOutcome::Admitted arm calls
  archive.insert_variant(&variant).await ("the FIRST production caller of the DGM archive admit path").
  the same arm already calls emit_admit_row(ssb, ...) and emit_clade_selected_row(ssb, ...), so the loop's
  ssb handle (struct field ssb_arc: &Arc<crate::ssb::Ssb>, darwin_admit_loop.rs:141) is available at the seam.
GROUNDED: the loop tracks a prior/new fitness pair — last_admitted_fitness (prior) and the current
  candidate's execution-grounded fitness (new) — plus seed and a frozen corpus (FrozenCorpus, run_one_generation).
GROUNDED: the gate is CLI-proven today via crates/cli/src/reward_tamper_cmd.rs:180 and frozen in
  .claude/gate-manifest.txt:39,150 — so wiring it does not change the gate's math, only adds a production caller.

================================================================================
THE UNIT (vertical slice: producer + production consumer + smoke, one changeset)
================================================================================
Move 1 — add a default-OFF reward-tampering interception in the darwin_admit_loop Admitted arm, BEFORE
insert_variant, that calls check_reward_tampering with the loop's own signals and SKIPS admission (emits a
non-admit/reject beacon, no insert_variant) on a Tampering verdict.

PARAM MAPPING (planning-time decisions — do NOT re-litigate; these remove the judgment calls that made I-33 fail):
  ssb                    -> the loop's existing ssb handle (the one passed to emit_admit_row).
  prior_reward           -> the parent/last-admitted candidate fitness (last_admitted_fitness).
  new_reward             -> THIS candidate's execution-grounded fitness (the score already computed for the
                            Admitted decision — the same value the guard chain used).
  band                   -> reward_tampering::DEFAULT_PLAUSIBLE_REWARD_BAND (do not invent a literal).
  expected_harness_blake3 / observed_harness_blake3 -> the frozen corpus blake3; expected == observed in THIS
                            loop because the harness is not swapped per-candidate — the ACTIVE leg here is the
                            reward-band leg (prior vs new reward outside the plausible band). (The harness-swap
                            leg stays inert-but-present; that is correct — it fires only if the corpus blake3 drifts.)
  run_id                 -> the generation/run id already threaded into the loop's SSB rows.
  seed                   -> the loop's seed (already in scope; used by emit_admit_row).
FLAG: gate the whole interception behind a NEW default-OFF env flag ELAI_H28_REWARD_GATE_ENABLE (R9). When
  OFF (default) the loop behaves EXACTLY as today (no behavior change ships enabled). When ON, a Tampering
  verdict blocks that candidate's insert_variant and records a reject.

RECON_NEEDED: the EXACT field names for last_admitted_fitness, the current candidate's execution-grounded
fitness value, the run_id, and the frozen-corpus blake3 accessor — settle by S1/S2 reading darwin_admit_loop.rs
around the Admitted arm + run_one_generation (grep -nE "last_admitted_fitness|execution_grounded_fitness|FrozenCorpus|run_id|corpus_blake3" darwin_admit_loop.rs). The SEMANTIC mapping above is fixed; only the identifiers are looked up. If the loop does NOT expose a usable prior/new reward pair at the seam, that is a material fork (ABORT below), not an improvisation.

EXPECTED_OBSERVATION: with ELAI_H28_REWARD_GATE_ENABLE=1, driving run_one_generation on a candidate whose
new_reward is implausibly above prior_reward (outside DEFAULT_PLAUSIBLE_REWARD_BAND) produces a
SsbPayload::RewardTampering row with a Tampering verdict AND no Admitted beacon for that candidate; with the
flag OFF (default), that same candidate admits exactly as on main and NO RewardTampering row is emitted.
ON_FAIL: build error at the call site (borrow/async/type) -> CAUSE the mapped identifiers were wrong ->
COUNTER-MOVE re-read the Admitted arm, match the exact fitness field the guard chain already used for this
candidate, thread &Arc<Ssb> by reference as emit_admit_row does. A RewardTampering row that never appears under
the ON path -> CAUSE the interception is placed after insert_variant or behind the wrong branch -> COUNTER-MOVE
move the call to the top of the Admitted arm, before select_parent_with_cmp/insert_variant.
FORKS: none (single deterministic seam, one call site).
ABORT: if S1/S2 finds the loop exposes NO prior/new reward pair at the seam (the fitness values are not both in
scope in the Admitted arm), STOP + handoff — this would require plumbing reward signals through the loop
signature, which is a larger change than an activation slice and needs an owner call; do NOT invent a reward
pair to satisfy the signature (that is the reward-tampering-gate's own failure mode).
VERIFY (off-agent artifacts, R4; run by the auditor/controller, never self-run-as-authority by the executor):
  - python /mnt/f/.ELAI_workflow/.claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/darwin_admit_loop.rs  (the new call site makes check_reward_tampering WIRED, not DARK)
  - cargo test -p elai-orchestrator darwin_admit reward_tamper  (the unit test asserting the ON/OFF behavior above passes)
  - the RewardTampering SSB row is the off-agent artifact: the smoke asserts it lands under the ON path.
INTEGRATION_SMOKE: drive run_one_generation (the REAL loop entrypoint, not a direct call to check_reward_tampering)
with ELAI_H28_REWARD_GATE_ENABLE=1 on a hard candidate set including one reward-implausible candidate; assert
(a) the reward-implausible candidate is NOT admitted, (b) a RewardTampering row with Tampering verdict is in the
SSB for that candidate, (c) with the flag unset the same set admits as on main. This is the R10 seam proof —
the gate firing inside the loop, not in isolation.

Move 2 — register the flag per R9 (no default-OFF graveyard).
WIRING_CONTRACT:
  produces: ELAI_H28_REWARD_GATE_ENABLE gating the darwin_admit_loop reward-tampering interception.
  consumes: the loop's Admitted arm reads the flag; a row in .elai_cc/wire-dark-allowlist.toml carries
    type=kill_switch, owner=maintainer, expiry_date=2026-10-01, promotion_plan_id=H-28-WIRE-1, and names the smoke.
  reachable_from: run_one_generation (the self-mod promotion loop) at runtime.
  observable_effect: flag ON -> a reward-implausible candidate is rejected + a RewardTampering row lands;
    flag OFF -> loop unchanged.
  stub_ban: no TODO, no placeholder reward values, no "wire later"; the call site + flag + allowlist row + unit
    test + smoke ship together or the unit is not done.
EXPECTED_OBSERVATION: python /mnt/f/.ELAI_workflow/.claude/tools/gate_freeze_check.py stays green (the allowlist
  is not a frozen-gate file) and grep finds the ELAI_H28_REWARD_GATE_ENABLE row in wire-dark-allowlist.toml.
ON_FAIL: the flag defaults ON or has no allowlist row -> CAUSE R9 violation -> COUNTER-MOVE default it OFF and
  add the allowlist row with all five fields.

================================================================================
REDTEAM (attacks against THIS plan)
================================================================================
REDTEAM attack-1 (PATCHED): "the worker satisfies check_reward_tampering's signature by passing 0.0/0.0 or a
  constant reward pair, making the gate inert while looking wired." PATCH: the INTEGRATION_SMOKE requires a
  reward-IMPLAUSIBLE candidate to be REJECTED under the ON flag — a constant/zero pair cannot produce a
  Tampering verdict, so this smoke fails closed on a faked mapping. The PARAM MAPPING pins new_reward to the
  candidate's real execution-grounded fitness, not a literal.
REDTEAM attack-2 (SURVIVED): "wiring changes loop behavior and breaks existing darwin tests." Defence: the flag
  is default-OFF; every existing test runs the OFF path = today's behavior exactly. The ON path is exercised
  only by the new smoke/unit test. Holds.
REDTEAM attack-3 (SURVIVED via ABORT): "the loop has no real prior/new reward at the seam, so this is secretly
  greenfield plumbing." Defence: the ABORT condition catches exactly this — if S1/S2 cannot find the pair in
  scope, the worker STOPS and hands off rather than inventing signals. The grounding ledger's read found
  last_admitted_fitness + the candidate fitness in the Admitted arm, so the expected outcome is activation, but
  the tripline is armed.

================================================================================
EXECUTABLE-BLIND SELF-CHECK
================================================================================
a mid-tier glm executor can run this: the guard fn + seam + param semantics + flag + off-agent artifact are all
named with file:line; the only lookups left are mechanical identifier names (RECON_NEEDED, a single grep); every
fork is either "none" or an ABORT with a concrete tripline; VERIFY names off-agent artifacts (SSB row + checker
exit-0 + cargo test), never the executor's stdout.
