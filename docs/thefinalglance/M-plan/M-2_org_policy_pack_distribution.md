# M-2 — Org policy pack + distribution + drift detection
Track M | lever CANT-WRECK (org grain) | grounding STRONG (gap G3; 3F/R8/E-39 substrate SHIPPED) |
nature BUILD-compose | effort M | WAVE M2 | depends M-1 (Team tier), rules-engine 3F lifecycle,
gate-manifest freeze (R8) live.

## Objective
One signed, file-distributable OrgPolicyPack every seat applies at boot and cannot weaken: org
rules (mutable=0-equivalent tier), gate-manifest pins, spend caps, model/backend allowlist,
redaction config, update-channel pin. The org buys ONE control surface instead of policing N
laptops by hand. No cloud, no console — the pack travels by git/file (MD-3).

## Premise / grounding
- Gap G3: mutable=0 rules, gate_freeze_check + .claude/gate-manifest.txt, E-39 caps, B-37
  allowlists, G-9 redaction ALL exist — per-install, hand-configured, with no cross-seat
  distribution or drift story. The org layer was an explicit K non-goal (K-0:531); M-2 is its
  minimal file-based form, NOT an org SaaS (ENTERPRISE RIDER stands).
- The freeze discipline is the precedent: R8 already proves "a run cannot edit its own gate"; M-2
  lifts the same property one level — "a seat cannot edit its org's gate".

## LOCKs
- LOCK-1 the pack is DATA: policy.toml + rule rows + manifest pin hashes + caps + allowlist +
  redaction config + channel pin, signed with the org key (G-8 core again). NEVER executable
  content (the L-13 FS-2 doctrine at org grain).
- LOCK-2 precedence: org pack > local config, and org rules land in an immune tier a local edit
  cannot demote (exactly the mutable=0 semantics, org-keyed). A seat may ADD stricter checks,
  never weaken (monotone-tighten only).
- LOCK-3 absence of a pack = today's behavior, byte-for-byte. Single-seat users never see M-2
  (W-2 satisfied structurally; no env gate).
- LOCK-4 drift is DETECTED + surfaced, not auto-corrected: boot compares applied-pack hash vs the
  org-pinned hash -> OrgPolicyEvent(Drift) + boot warning + M-3 rollup visibility. Auto-remediate
  is a later rider (needs GD-4 FP bounding first).

## Tasks (vertical slices)
1. Pack format + signer/verifier + `elai org policy init|sign|verify|show` (admin authoring side);
   crates/orchestrator/src/commerce/org_policy.rs; +OrgPolicyEvent (Pattern-2: Applied/Drift/
   RefusedForged/WeakenAttemptBlocked).
2. Boot apply seam: pack discovery (repo .elai/org-policy.signed, then ~/.elai/org/), verify,
   apply-in-order (rules rows via the 3F loader into the org-immune tier; caps into E-39; pins
   compared against the local frozen set; redaction config into G-9; channel pin read by M-4).
   Emits Applied with the pack hash.
3. Weaken-attempt gate: a local rule edit / cap raise / allowlist widen that contradicts the pack
   is REFUSED with WeakenAttemptBlocked naming the pack clause (deterministic compare, TD-3).
4. Drift check: per-boot + per-`org report` (M-3 consumes Drift rows per seat).

## COMPOSES-WITH (do NOT reimplement)
3F rules lifecycle + mutable-tier semantics; gate_freeze_check.py / gate-manifest (R8 — the pack
PINS it, never replaces it); E-39 cost gate (caps consumer); K-5 GateIntensity policy table (the
pack may pin its hash); G-9 redaction; G-8 signing core; M-1 tier() (pack apply = Team feature).

## WIRING_CONTRACT
produces: commerce/org_policy.rs (pack type, verify/apply, weaken gate), `elai org policy *` CLI,
  OrgPolicyEvent SSB (+1).
consumes: 3F loader, gate-manifest hashes, E-39 caps, G-9 config, M-1 tier, G-8 signing.
reachable_from: boot path (pack discovery) + `elai org policy *` CLI.
trigger: place fixtures/org-policy.signed (spend cap 0.05, one org GREP rule, stable channel pin)
  in a fixture repo; boot; run a fixture task that would exceed the cap.
expect_ssb: OrgPolicyEvent(Applied) with pack hash; the task blocked by the ORG cap (E-39 row
  names the org clause, not local config); the org GREP rule fires in SHADOW.
observable_effect (done-gate): the fixture boots with the pack applied + org cap enforced; a
  tampered pack byte is REFUSED (RefusedForged, R2); a local attempt to raise the org cap ->
  WeakenAttemptBlocked; a seat with NO pack behaves byte-identically to pre-M-2 (regression arm);
  a stale applied-pack vs pinned-hash fixture -> Drift row at boot.

## Risks (top 3)
R1 org-rule FP friction at fleet scale (the CodeGate death, GD-4): org rules enter SHADOW first,
  promote per-org on the same fpr<5% discipline — the 3F lifecycle already encodes this; the pack
  just carries lifecycle state.
R2 pack-key compromise = org-wide policy injection. Same key posture as M-1 R1; pack verify also
  pins the SIGNER identity per org (a foreign org's validly-signed pack is refused).
R3 precedence bugs (org vs local vs defaults) are silent-failure-prone: the apply order is one
  deterministic function with a contract test enumerating all three layers (R2 broken-arm test).
