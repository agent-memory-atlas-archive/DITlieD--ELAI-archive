# H-28: Current-RF reward-tampering gate + harness-in-the-frozen-set (T-12) — self-mod safety capstone

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER F capstone; S-effort BUILD reusing blake3 tripwire + WAL + rules engine)

Objective: ELAI's blake3 tripwire catches RF TAMPERING (the self-mod agent edits the eval harness -> hash mismatch)
but NOT RF-INPUT tampering (the agent leaves the harness intact but the post-self-mod score jumps implausibly — it
gamed the inputs/environment the evaluator reads). H-28 puts the evaluator, metric harness, gates, thresholds AND the
H-17 trial ledger ALL in the gate-freeze LOCKED set the self-mod agent cannot read or mutate (Everitt current-RF), and
adds a COMPOSITE reward-tampering gate with a persisted reward_tampering_counter that: HARD-BLOCKS on an eval-harness
blake3 mismatch (= RF tampering — reuse the shipped tripwire), and FLAGS/BLOCKS an implausible post-self-mod score
delta when the eval hash is INTACT (= RF-INPUT tampering — the new leg). A deterministic 3a BLOCK can NEVER be
overridden by a probabilistic 3b green (TD-3). Structural self-mod (FSM states, agent roles, gate logic) stays human
Layer-3 ONLY (TD-5 — proving a self-mod is SAFE is undecidable, Rice/Schmidhuber). Off-agent artifact: the eval-harness
hash + the score-delta history.

Estimated effort: ~1 session (S, ~4-6 h — reuses blake3 + WAL + rules engine; ~80% primitives exist per the design).
Pre-requisites: the DGM self-mod substrate SHIPPED (config_archive.rs:274 select_parent, promotion_gate.rs:109
  two_gate_promote, darwin_admit_loop.rs:298 the live driver, is_security_sensitive self_mod_layer1a.rs:383,
  AllowedConfigKey :203, LockedKeysAuditor + blake3 ArchiveVerifyResult ssb.rs:30840); H-17 (the ledger to freeze);
  H-27 (which validates this gate's own logic).
Parallel-safe with: H-27 (verify-the-verifiers). Composes the SHIPPED two_gate_promote + LockedKeysAuditor.
Source: design T-12 + PART 1 (D3/D4) + PART 0 (TB-3) + PART 5 (residue); _grounding/H-TRUST-GROUNDING-MAP.md TS-7 + NN-12 + TD-2/3/5. Doctrine: H-0_INDEX.md.

## Context

TB-3 SELF-EVALUATOR CAPTURE: a self-MODIFYING optimizer that can read or mutate its own evaluator/threshold/trial-
ledger will route itself to PASS (DGM empirically faked test results + disabled detectors; Everitt reward tampering).
ELAI's blake3 tripwire (ArchiveVerifyResult ssb.rs:30840) catches harness EDITS (hash mismatch); the gap is RF-INPUT
tampering — the harness hash is intact but the score jumps because the agent gamed the inputs/environment. H-28 (a)
puts the evaluator + harness + gates + thresholds + the H-17 ledger in the LOCKED frozen set (the highest-leverage
single safety move — D3, Everitt current-RF: the reward function is private and uninfluenceable), and (b) adds the
reward-tampering COMPOSITE: HARD-BLOCK on hash mismatch (reuse the tripwire) + FLAG/BLOCK an implausible score delta
with the hash intact (the new RF-INPUT leg). A deterministic 3a BLOCK can never be overridden by a probabilistic 3b
green (TD-3 — the order is load-bearing). Structural self-mod stays human Layer-3 (TD-5 — undecidable).

## Existing state (B/C/D/E/F/G coverage)
- DGM self-mod substrate (config_archive.rs:274 select_parent + insert_variant; promotion_gate.rs:109 two_gate_promote;
  darwin_admit_loop.rs:298 the LIVE two_gate_promote driver; is_security_sensitive self_mod_layer1a.rs:383;
  AllowedConfigKey closed enum :203) — SHIPPED (B-23/F-11). The promotion path the reward-tamper gate sits in front of.
- blake3 tripwire / ArchiveVerifyResult (ssb.rs:30840; D-8 archive verify) — SHIPPED. The RF-tampering (hash mismatch)
  leg H-28 reuses (GD-1; do NOT re-implement the tripwire).
- LockedKeysAuditor + LOCKED_KEYS_TOUCH (locked_keys_observable_smoke.rs:52/80/95; locked_keys_integration.rs) — SHIPPED
  (D-9). The frozen-key machinery; H-28 adds the harness + thresholds + the H-17 ledger to the LOCKED set.
- gate_freeze_check.py + .claude/gate-manifest.txt (R8) — SHIPPED. The freeze mechanism (TD-2).
- a reward_tampering_counter + the RF-INPUT-tamper flag (score jump with hash intact) — ABSENT (NN-12). The BUILD leg.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-7, NN-12, TD-2, TD-3, TD-5, the H-28 symbol table.
- design T-12 + PART 1 (D3/D4) + PART 0 (TB-3) + PART 5; H-17 (the ledger to freeze), H-27 (which validates this gate).
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\promotion_gate.rs:100-230 (two_gate_promote + the descriptor)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\darwin_admit_loop.rs:280-310 (the live two_gate_promote driver)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_layer1a.rs:200-400 (AllowedConfigKey :203; is_security_sensitive :383)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\tests\locked_keys_observable_smoke.rs:40-100 (LockedKeysAuditor observable pattern)
- CLAUDE.md HARD RULES: blake3 (the eval-harness hash), no .unwrap()/.expect(); TD-2/3/5.

## Tasks (WAVE 3 — self-mod safety capstone)

1. **Freeze the evaluator + harness + thresholds + the H-17 ledger (TD-2, ~30 LOC + manifest).** Add the eval harness,
   the metric code, the gate thresholds (the H-18 DSR/H-22 alpha/H-21 ULP bounds), AND the H-17 trial ledger to the
   gate-manifest LOCKED set + the AllowedConfigKey/LockedKeysAuditor frozen surface. The self-mod agent cannot read or
   mutate them (Everitt current-RF). This is the design's highest-leverage single safety move.

2. **RF-tampering leg: eval-harness blake3 mismatch HARD-BLOCK (~30 LOC).** Reuse the SHIPPED blake3 tripwire
   (ArchiveVerifyResult): on a self-mod admission, recompute the eval-harness blake3 and HARD-BLOCK on mismatch (=
   the agent edited the harness). This is leg 3a (deterministic).

3. **RF-INPUT-tampering leg: implausible score delta with hash INTACT (~50 LOC, the new piece).** When the eval-hash
   is intact but the post-self-mod score jumps beyond a plausible band, FLAG/BLOCK and route to human Layer-3 (= the
   agent gamed the inputs the evaluator reads). The plausible band is a LOCKED constant. This is the gap the shipped
   tripwire does not catch.

4. **Ordering invariant: 3a BLOCK is never overridden by 3b green (~20 LOC).** A deterministic 3a (hash mismatch)
   BLOCK can NEVER be overridden by a probabilistic 3b (score-plausibility) green. Encode the precedence so no path
   promotes on a hash mismatch (TD-3 — the order is load-bearing).

5. **Persisted reward_tampering_counter + off-agent artifact (~30 LOC).** Emit `reward_tampering` (Pattern-2 with
   Kind: HarnessHashMismatch / ImplausibleScoreDelta / StructuralSelfModRefused; baseline+1 from LIVE 207/484, re-read
   + bump e45:294) carrying the eval-harness hash + the score-delta history. The counter is append-only,
   agent-unresettable (TD-1 discipline). Structural self-mod (FSM/roles/gate logic) emits StructuralSelfModRefused +
   routes to human Layer-3 (TD-5 — undecidable, never auto-approved).

6. **Decision doc workflow/reasoning/H-28-decisions.md (~90 LOC).** Records: (a) the frozen LOCKED set (D3, the
   highest-leverage move); (b) RF vs RF-INPUT tampering (the shipped tripwire catches the former; the new leg the
   latter); (c) the 3a-never-overridden-by-3b ordering (TD-3); (d) structural self-mod stays human Layer-3 (TD-5,
   Rice/Schmidhuber — proving a self-mod SAFE is undecidable).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/reward_tampering.rs::check_reward_tampering (the COMPOSITE: 3a hash + 3b score-delta + ordering)
  - crates/orchestrator/src/reward_tampering.rs::reward_tampering_counter (persisted, append-only)
  - crates/rules/seed/meta/reward_tampering.toml (the score-plausibility band + the LOCKED-set declaration, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::RewardTampering (Pattern-2 3-Kind; baseline+1)
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::ArchiveVerifyResult (the blake3 tripwire — REUSE for leg 3a, GD-1)
  - crates/orchestrator/src/promotion_gate.rs::two_gate_promote + darwin_admit_loop.rs (the self-mod promotion the gate fronts)
  - crates/orchestrator/src/self_mod_layer1a.rs::{is_security_sensitive, AllowedConfigKey} + LockedKeysAuditor (the frozen-key machinery — REUSE)
  - .claude/tools/gate_freeze_check.py + .claude/gate-manifest.txt (freeze the harness + thresholds + the H-17 ledger — TD-2)
reachable_from:
  - the DGM self-mod admission path (darwin_admit_loop.rs:298 two_gate_promote -> check_reward_tampering before insert_variant) [PRODUCTION hook]
  - check_reward_tampering -> the blake3 + score-delta checks -> RewardTampering SSB row [off-agent artifact]
stub_ban:
  - NO self-mod admission without the reward-tampering check (missing = BLOCK — W-1/R1)
  - NO 3a (hash-mismatch) BLOCK overridden by a 3b (score) green (the ordering is load-bearing — TD-3)
  - NO structural self-mod auto-approval (FSM/roles/gate logic -> human Layer-3 only — TD-5)
  - NO agent read/mutate access to the frozen LOCKED set (harness + thresholds + ledger — TD-2)
  - NO re-implementation of the blake3 tripwire or two_gate_promote (GD-1 P0); blake3; no .unwrap()/.expect() in prod
frontend_pair:
  the eval-harness hash + the score-delta history + the reward_tampering_counter in the self-mod cockpit. cross_lang_diff PASS. W-6: a blocked self-mod visible in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <run a self-mod admission where (a) a variant edits the eval harness, and (b) a separate variant leaves the hash intact but jumps the score implausibly, NO env override>"
  expect_ssb: "a RewardTampering(HarnessHashMismatch) row (HARD-BLOCK) for (a) and a RewardTampering(ImplausibleScoreDelta) row (flag/block + Layer-3 route) for (b), recomputable from the journal (GD-2)"
  expect_artifact: "the harness-editing variant is HARD-BLOCKED on hash mismatch; the score-jump-with-hash-intact variant is FLAGGED/BLOCKED for Layer-3; a clean config variant promotes; a structural self-mod is refused to human Layer-3"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-28_reward_tampering_harness_frozen.md --run-trigger
rg "check_reward_tampering|two_gate_promote|ArchiveVerifyResult" .elai_cc/crates/orchestrator/src --glob '!*test*'   # the gate fronts the self-mod promotion + reuses the tripwire
rg -n "reward_tampering|score.*delta|locked" .elai_cc/crates/rules/seed/meta/reward_tampering.toml   # the band + LOCKED-set declaration
rg -c "blake2|sha2" .elai_cc/crates/orchestrator/src/reward_tampering.rs                              # MUST be 0
python .claude/tools/gate_freeze_check.py                                                             # harness + thresholds + H-17 ledger frozen
test -f workflow/reasoning/H-28-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a harness-edit variant HARD-BLOCKED; a score-jump-hash-intact variant flagged for Layer-3). cargo-green != DONE.
- [ ] W-2 default-ON: the reward-tampering check runs on every self-mod admission with no env override; kill-switch only.
- [ ] W-3 production entrypoint: check_reward_tampering fires from darwin_admit_loop two_gate_promote, not a #[cfg(test)] caller.
- [ ] TD-2: the harness + thresholds + the H-17 ledger are in the frozen LOCKED set. TD-3: 3a never overridden by 3b.
- [ ] TD-5: structural self-mod -> human Layer-3 only (never auto-approved).
- [ ] GD-1: blake3 tripwire + two_gate_promote + LockedKeysAuditor REUSED. GD-2: the hash + score-delta re-compute under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; check_reward_tampering (produced) consumed on the self-mod admission (W-8) reached by darwin_admit_loop (W-3).
- W-2: no override; kill-switch only. W-3: live self-mod admission path; not test-only.
- W-4: frontend_pair = self-mod cockpit; cross_lang_diff PASS. W-5: runnable self-mod admission -> real rows; Tier-1 in-process.
- W-6: blocked self-mod visible in dashboard. W-7: ArchiveVerifyResult ssb.rs:30840 + two_gate_promote promotion_gate.rs:109 + darwin_admit_loop.rs:298 + LockedKeysAuditor grep-confirmed (TS-7); RF-INPUT-tamper leg absent (NN-12).
- W-8: check_reward_tampering (produced) consumed on admission; the tripwire + two_gate_promote reused.
- W-9: +RewardTampering Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (hash + score-delta). TD-2/3/5: frozen, ordered, human-Layer-3-for-structural.

## Tests to add
- crates/contracts/tests/h28_reward_tampering_contract.rs (~160 LOC): (a) harness-edit variant -> HarnessHashMismatch
  HARD-BLOCK; (b) score-jump-with-hash-intact -> ImplausibleScoreDelta flag/block + Layer-3 route; (c) a clean config
  variant -> promotes; (d) a 3a BLOCK is NOT overridden by a 3b green (the ordering invariant); (e) a structural
  self-mod -> StructuralSelfModRefused + Layer-3; (f) the harness/thresholds/ledger are frozen (gate_freeze_check
  blocks a self-edit); (g) the reward_tampering_counter is append-only (no agent reset).
- crates/orchestrator/tests/fixtures/reward_tampering/ (a harness-edit variant + a score-jump variant + a clean one + a structural one).

## Files touched
- CREATE: crates/orchestrator/src/reward_tampering.rs (~120 LOC: check_reward_tampering + the counter + the ordering invariant)
- CREATE: crates/rules/seed/meta/reward_tampering.toml (the score-plausibility band + the LOCKED-set declaration, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+RewardTampering Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: crates/orchestrator/src/darwin_admit_loop.rs (~20 LOC: call check_reward_tampering before insert_variant)
- MODIFY: .claude/gate-manifest.txt (add the eval harness + thresholds + the H-17 trial ledger to the LOCKED set)
- CREATE: crates/contracts/tests/h28_reward_tampering_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-28-decisions.md (~90 LOC)

## Citations / Grounding
- design T-12 + D3/D4 + TB-3 + PART 5; Everitt et al. reward tampering (arXiv:1908.04734, current-RF principle);
  Denison et al. Sycophancy-to-Subterfuge; the DGM faked-test incident; Rice / Schmidhuber undecidability (structural
  self-mod SAFE is undecidable -> human Layer-3). Verify arXiv:1908.04734 before citing the current-RF formulation.
- ELAI ground truth (TS-7/NN-12, grep-confirmed 2026-06-04): ArchiveVerifyResult ssb.rs:30840; two_gate_promote
  promotion_gate.rs:109 + darwin_admit_loop.rs:298; is_security_sensitive self_mod_layer1a.rs:383; AllowedConfigKey
  :203; LockedKeysAuditor; the RF-INPUT-tamper flag (score jump with hash intact) ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-2/3/5.

## Open questions / risks
- The "implausible score delta" band is the hard calibration: too tight blocks a legitimate improvement, too loose
  misses gaming. It is a LOCKED constant set conservatively + surfaced to the operator; an implausible delta FLAGS to
  Layer-3 (human decides) rather than silently auto-blocking a real gain — fail-to-human, not fail-to-silent-reject.
- Freezing the H-17 ledger is essential (TB-3): if the agent can reset N, the DSR-at-N gate (H-18) is gamed at the
  source. H-28 + H-17's no-reset discipline together close this; verify the ledger is in the LOCKED set (the single most important check).
- This gate guards self-mod but is itself code that could be self-modified — it is in the frozen LOCKED set (it cannot
  modify itself) and H-27 validates its own logic. The combination (frozen + self-validated) is the meta-layer's spine.

## Next
H-29 (LOCKED fail-closed drift state machine) is CLUSTER G — the deployment-trust gate on the resume/keep-live FSM
edge. H-27 + H-28 complete the meta-layer (verify-the-verifiers + safe self-modification), the capstone that makes
the autonomous trading-strategy optimizer trustworthy under search (TB-1/TB-2/TB-3 all closed deterministically).

End of H-28.
