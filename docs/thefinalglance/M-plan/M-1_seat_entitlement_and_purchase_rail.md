# M-1 — Seat entitlement + purchase rail [M KEYSTONE]
Track M | lever SELLABLE | grounding STRONG (gap G1/G2, grep-proven; K-24 substrate PLANNED) |
nature BUILD | effort M-L | WAVE M1 | depends: K-24 slices L1..L4 live (MA-1), G-8 attestation
signing shape shipped.

## Objective
Turn a payment into a verifiable seat. A signed, OFFLINE-verifiable Entitlement artifact that a
payment mints, a machine activates, and every tier-gated feature reads — with expiry, grace, and
degrade-to-free semantics that can never brick a working seat (MD-1).

## Premise / grounding
- Gap G1/G2 (steal-multiseat-gap-2026-07-02.md): zero entitlement machinery anywhere in B..L.
  K-24 is METERING (credits spent per usage) — it deliberately leaves "payment webhook OUTSIDE
  ELAI" and never binds a paying org to an installed seat.
- The signing/verification pattern EXISTS: G-8 emits blake3-Merkle/ed25519-signed attestations
  verified offline by `elai verify-attestation`. The Entitlement is the SAME shape pointed at
  commerce (GD-1: reuse the signing/verify core, do not re-implement).
- Buyer per GTM memory: team/agency seats, $39-49/seat class. Tiers are DATA (M-5d tiers.toml).

## LOCKs (do not re-litigate at execution)
- LOCK-1 MD-1: expiry/verification-failure => DegradedToFree + everything Free-tier keeps working
  + safety gates untouched. No kill-the-binary path exists in the code, period.
- LOCK-2 MD-2/MD-3: the license service does mint/renew/revoke ONLY. No telemetry, no runtime
  callback, no feature flags served remotely. A seat offline for a year still verifies its
  entitlement locally (expiry logic is local, from the signed artifact + ClockProvider).
- LOCK-3 payment provider: Merchant-of-Record (Paddle/Lemon-Squeezy-class) OUTSIDE ELAI, PCI
  scope zero — the K-24 decision inherited verbatim. Provider choice is a user decision at
  execution (E-50's MoR-vs-Stripe fork applies to ELAI itself here).
- LOCK-4 Entitlement is org-granular with seat slots: Entitlement{org_id, tier, seats_max,
  features[], issued_at_ms, expires_at_ms, grace_days=14, sig}; a seat ACTIVATION binds
  {seat_id = blake3(machine-scoped salt), org_id} locally. Seat-count enforcement is HONEST
  best-effort client-side (an offline product cannot hard-enforce concurrency; over-use shows in
  the M-3 rollup instead — recorded limit, GD-5).

## Tasks (vertical slices — producer + consumer + smoke each, PRE_GATE)
1. Entitlement type + signer/verifier: crates/orchestrator/src/commerce/entitlement.rs reusing
   the G-8 signing core; `elai license activate <file|key> | status | deactivate`; local store
   under ~/.elai/entitlement/. +EntitlementEvent (Pattern-2: Activated/Renewed/ExpiredGrace/
   DegradedToFree/Revoked/RefusedForged).
2. Tier gating seam: one `entitlement::tier()` read consumed by (a) K-24 credit-grant path (Team
   credits only mint against a live Team entitlement), (b) M-2 org-pack apply (Team feature),
   (c) M-3 `org report` (Team feature). Free tier = everything shipped through L that is not
   explicitly Team-tier in tiers.toml (M-5d). The gate READS data; the LLM never adjudicates tier
   (TD-3).
3. Mint service (the ONE first-party cloud piece, MD-3): a minimal endpoint — MoR webhook in,
   signed Entitlement out (mint/renew/revoke); keys offline-managed; every mint appended to an
   operator-side ledger file (auditable, GD-2-shaped). Deployment is an ops runbook, not a crate
   in the workspace.
4. Renewal/expiry loop: opportunistic re-check when online (never blocking); expiry -> 14-day
   grace (ExpiredGrace rows + status banner) -> DegradedToFree. Revocation applies on next
   opportunistic check only (offline seats keep grace semantics — honest, GD-5).

## COMPOSES-WITH (do NOT reimplement)
G-8 attestation signing/verify core; K-24 credit ledger + seats.toml (the rail this gates);
ClockProvider (all expiry math — no SystemTime); K-1 claims lint (any UI copy about tiers);
B-43 installer (activation offered at install-end, one line).

## WIRING_CONTRACT
produces: commerce/entitlement.rs (Entitlement, activate/status/deactivate, tier()),
  EntitlementEvent SSB (+1), mint-service runbook + operator ledger format.
consumes: G-8 signing core, K-24 grant path, tiers.toml (M-5d), ClockProvider.
reachable_from: `elai license *` CLI (real entrypoint); tier() read on the K-24 grant path.
trigger: `elai license activate fixtures/team-entitlement.signed && elai license status`
expect_ssb: EntitlementEvent(Activated) with org_id + tier=Team; a K-24 fixture grant succeeds
  ONLY with the live Team entitlement.
observable_effect (done-gate): fixture Team entitlement activates OFFLINE (network disabled) ->
  Activated row + Team features unlock; clock-forwarded expiry fixture (SeededClock) ->
  ExpiredGrace then DegradedToFree rows AND a Free-tier task still runs green AND every safety
  gate still fires (MD-1 negative arm); a forged-signature entitlement is REFUSED
  (RefusedForged, R2); seat over-activation beyond seats_max is RECORDED (visible in M-3), not
  silently absorbed.

## Risks (top 3)
R1 key management: a leaked mint key = infinite seats. Mitigation: offline key ceremony + revoke
  list shipped in release manifests (M-4) — recorded, not solved, GD-5.
R2 fail-open abuse: Free-tier floor means piracy of Team features is a licensing matter, not a
  technical wall. Deliberate (MD-1); the paid value concentrates in org pack/rollup which are
  org-workflow features, low piracy value.
R3 clock tampering: local expiry uses ClockProvider/OsClock; a wound-back clock extends grace.
  Bounded by opportunistic online re-check; recorded residue (GD-5).
