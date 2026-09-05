# P-7: the per-user operating profile — adapt PROCESS not just content; asymmetric, measured, never sycophantic

STATUS: PLANNED (WAVE 3; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — rides the anti-selection-bias promotion machinery)

- Track: PERSONAL-INCARNATION | lever: measured-asymmetric-process-adaptation | source finding: GAP-7 (direct answer to "how does ELAI adapt to every user, not only memory/code/workflows") | grounding: MODERATE (index-derived, PLAN-asserted) | nature: composition on N-3/H-17/H-18 | effort: M-L | WAVE 3 | depends N-3 (anti-selection-bias promotion) landed

- Premise: memory adapts CONTENT (E-5 voice/expertise facts, E-4 terseness). NOTHING adapts PROCESS: how much autonomy, how much gate strictness, how much explanation, per DOMAIN, per user. P-7 adds a per-user OperatingProfile learned from journaled outcomes and promoted through the corpus's own anti-selection-bias machinery. The three user archetypes the owner named — solo toy, connected small projects, big team — become three DEFAULT profiles on ONE engine, not three products: toy = P-5 ceremony floor; connected = P-2 world bus on; team = M org pack + P-8.

## Steal/build
An `OperatingProfile { per_domain: { autonomy, gate_strictness, explanation_verbosity } }` learned from journaled outcomes and promoted through H-17 trial ledger + H-18 deflated significance (exactly what N-3 generalizes) — adaptation is a MEASURED promotion, never a vibe. The load-bearing invariant is ASYMMETRY, mirroring M-2 monotone-tighten: a profile may TIGHTEN gates freely, but may LOOSEN ONLY against measured false-positive evidence from THAT user's own runs. This is the structural defense against adaptation-as-sycophancy (a system that relaxes verification because the user seems annoyed is Goodharting its own trust spine — the same failure class D-14/N-15 police at answer grain, now policed at PROCESS grain). Every profile change is observe-first (GD-4), journaled, and revertible (H-49 applies to configuration too).
EXCLUDED: a loosening driven by anything but measured own-run FP evidence (A-4 adaptation-as-sycophancy — the attack surface this unit exists to close); an unmeasured/vibe promotion (rides H-17/H-18 significance — never a raw count); a profile change that isn't observe-first + journaled + revertible; adapting CONTENT (that is E-5's job — P-7 is PROCESS only, disjoint grain).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- N-3 is anti-selection-bias promotion; H-17 is the trial ledger; H-18 is deflated significance — a profile change is a promotion through this exact machinery.
- M-2 is monotone-tighten (gates tighten freely, loosen only on evidence) — P-7's asymmetry is M-2 at process grain.
- GD-4 is observe-first for any change defaulting on; H-49 is reversibility — every profile change is both.
- E-5/E-4 adapt CONTENT (voice, terseness) — P-7 is disjoint (PROCESS: autonomy, strictness, verbosity).
- D-14/N-15 police anti-affirmation/anti-sycophancy at answer grain — P-7 extends the posture to process.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the OperatingProfile type + the three default profiles (toy/connected/team) + the ASYMMETRY enforcement (tighten free, loosen only on measured own-run FP evidence via H-17/H-18) + the journaled `operating_profile_change` SSB row + observe-first + revert.
- DEFER (Tier-2): the automatic per-domain learning loop from journaled outcomes (the full N-3 promotion pipeline pointed at process metrics — lands once N-3 is confirmed live and its FP-rate falsifier is measured) + the per-domain granularity beyond the three defaults.

## Integration
- crates/orchestrator: the OperatingProfile type + the asymmetry-guarded change function (routes loosening through H-17/H-18) + the SSB emit + the observe-first/revert plumbing.
- CLI: `elai profile show|tighten|loosen <domain>` subcommand is the Tier-1 production caller (loosen refuses without evidence).
- learning (Tier-2): the N-3 promotion pipeline reads journaled process outcomes + promotes a profile change.

## MOVES (wargame)
M1 add the OperatingProfile type + the three default profiles + a change function where TIGHTEN applies directly and LOOSEN routes through H-17/H-18 significance on the user's own-run FP evidence.
EXPECTED_OBSERVATION: `elai profile tighten <domain>` applies + journals; `elai profile loosen <domain>` WITHOUT sufficient measured own-run FP evidence is REFUSED; with it, promotes through H-18 significance.
ON_FAIL: a loosen applies on a raw count or on no evidence -> the asymmetry (A-4 defense) is broken -> gate loosen on the H-18 deflated-significance verdict, add the no-evidence-refusal test.
M2 make every profile change observe-first (GD-4) + journaled + revertible (H-49).
EXPECTED_OBSERVATION: a profile change emits `operating_profile_change {domain, field, old, new, evidence_ref}`; a revert restores the prior value; the change runs observe-only first where it defaults a gate on.
ON_FAIL: a change isn't revertible or skips observe-first -> route it through the H-49 inverse op + GD-4 observe-first, assert the journal + revert.
M3 wire the three defaults to the sibling units: toy -> P-5 ceremony floor, connected -> P-2 world bus on, team -> M org pack + P-8.
EXPECTED_OBSERVATION: selecting the toy default sets the P-5 toy ceremony tier; selecting team enables the M org pack posture; the mapping is journaled.
ON_FAIL: a default doesn't compose with its sibling unit -> the "one engine, three defaults" invariant breaks -> map each default to the sibling's declared profile, not a fork.

## FORK
FORK: if N-3's promotion pipeline is NOT yet live/confirmed (P-7 depends on N-3 landed) -> ship Tier-1 with the asymmetry-guarded MANUAL change function (tighten free, loosen refused without own-run FP evidence supplied explicitly) and route the AUTOMATIC learning loop to Tier-2 behind N-3's confirmation; record the N-3-dependency seam in the decision doc. The asymmetry defense (A-4) is fully realized manually; automation is the convenience, not the safety.

## RECON_NEEDED
RECON_NEEDED: N-3's promotion API + whether H-17/H-18 can score a PROCESS metric (FP rate) as readily as a content metric -> settle via reading N-3 anti-selection-bias promotion + H-17 trial ledger + H-18 deflated significance + `grep -rn "N-3\|H-17\|H-18\|promotion\|significance\|trial.ledger" .elai_cc/crates`.
RECON_NEEDED: M-2's monotone-tighten enforcement point (to mirror it at process grain) -> settle via `grep -rn "M-2\|monotone\|tighten\|loosen" .elai_cc/crates`.
RECON_NEEDED: where E-5 content adaptation lives, to keep P-7 process-only + disjoint -> settle via reading E-5 memory adaptation.

## ABORT
ABORT: if the loosen path can be satisfied WITHOUT measured own-run FP evidence (e.g. by a user assertion, a global default, or another user's data) -> stop and flag; that is the adaptation-as-sycophancy attack surface (A-4) the whole unit exists to close. Loosening is earned by that user's own measured false-positives, or it does not happen.

## WIRING_CONTRACT
produces_symbols:
  - the OperatingProfile type + the three default profiles + the asymmetry-guarded change function (loosen via H-17/H-18)
  - the `elai profile show|tighten|loosen` CLI subcommand + the journaled `operating_profile_change` SSB row + observe-first/revert
consumes_symbols:
  - N-3 anti-selection-bias promotion, H-17 trial ledger, H-18 deflated significance, M-2 monotone-tighten, GD-4 observe-first, H-49 reversibility, E-5 (disjoint grain), P-5/P-2/M/P-8 (the three defaults' targets)
reachable_from:
  - the `elai profile tighten|loosen` CLI subcommand on a live run  [PRODUCTION CLI]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO loosen without measured own-run FP evidence (A-4)
  - NO unmeasured/vibe promotion (rides H-17/H-18)
  - NO profile change that isn't observe-first + journaled + revertible
  - NO content adaptation (E-5's job; P-7 is process-only)
observable_effect:
  trigger: "attempt `elai profile loosen <domain>` with no FP evidence, then with measured own-run FP evidence; and `elai profile tighten <domain>`"
  expect_ssb: "a tighten applies + journals operating_profile_change; a loosen with no evidence is REFUSED (no change row); a loosen with H-18-significant own-run FP evidence promotes + journals with evidence_ref"
  expect_artifact: "a revert restores the prior value; NEGATIVE: a loosen that applies on no/raw-count evidence, or a change that isn't revertible, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai profile` subcommand dispatch calls the asymmetry-guarded change function (loosen routes through H-17/H-18) + the SSB emit — the executor names the exact promotion-scoring fn at RECON and wires it same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-7_per_user_operating_profile.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a loosen was REFUSED off-agent without measured own-run FP evidence and PROMOTED through H-18 significance with it, a tighten applied freely, and every change was journaled + revertible.

INTEGRATION_SMOKE: an independent agent tightens a domain (applies), attempts a loosen with fabricated/insufficient evidence (must refuse), supplies measured own-run FP evidence and loosens (promotes through H-18), reverts, and asserts the asymmetry held throughout + the three defaults each mapped to their sibling unit's posture — a multi-op run firing the refuse + promote + revert paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "talk ELAI into lowering a security gate by acting frustrated across a session": FAILS because loosening is gated on measured own-run FP evidence through H-18, not on user affect; frustration produces no FP evidence, so the loosen refuses. This is the A-4 attack, and the asymmetry rule is its complete answer. Attack that SUCCEEDED and its patch — "loosen using another user's or a global FP rate": patched by scoping the FP evidence to THAT user's own runs only (M1 stub_ban), so cross-user or global evidence never satisfies a per-user loosen.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-7 + A-4 (adaptation-as-sycophancy; monotone-loosen-only-on-evidence is the named patch) + A-2 (falsifier: profile-promotion FP rate).
- feature-map: N-3 anti-selection-bias promotion, H-17 trial ledger, H-18 deflated significance, M-2 monotone-tighten, GD-4 observe-first, H-49 reversibility, E-5/E-4 content adaptation, D-14/N-15 anti-sycophancy.

## COMPOSES-WITH
N-3/H-17/H-18 (the promotion machinery, EXTEND to process metrics), M-2 (mirror asymmetry), GD-4/H-49, E-5 (disjoint content grain), P-5/P-2/M/P-8 (the three defaults' targets). Re-implements NONE (GD-1). P-5's ceremony tier is the toy default's target; P-8 is the team default's target.

End of P-7.
