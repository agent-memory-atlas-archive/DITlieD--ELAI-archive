# M-5 — Commercial packaging + verifier OSS + trust continuity
Track M | lever SELLABLE + AUDITABLE | grounding STRONG (gap G7; GTM decision on file:
project_gtm_playbook_2026 "open-source ONLY the verifier, engine stays closed, escrow for
bus-factor") | nature E-23-class docs + ONE crate extraction | effort S-M | WAVE M1 (a,d) /
WAVE M3 (b,c) | depends K-1 claims lint live; (a) needs G-2/G-8 verify paths stable.

## Objective
Execute the commercial decisions that already exist as strategy but have NO executing session in
B..L: publish the standalone OSS verifier (the distribution + anti-rug-pull leg), write the legal
wrapper (EULA/DPA/pricing) under the K-1 claims lint, publish the continuity/escrow statement,
and pin the seat-tier matrix as DATA that M-1 reads.

## Slices
(a) VERIFIER OSS EXTRACTION [WAVE M1 — the one build]. Extract a standalone `elai-verify`
    crate/binary: verify-attestation (G-8) + `rra verify` action-stream re-check (G-2) + the K-6
    bundle conformance check + M-3 org-bundle verify. Zero engine deps (the extraction IS the
    proof the verify path has no engine entanglement — a compile-time boundary test). License
    MIT-or-Apache (user picks at execution); repo published; the engine repo stays closed. Every
    evidence artifact ELAI emits becomes third-party-checkable without buying anything — the
    GTM wedge's credibility leg.
(b) LEGAL WRAPPER [WAVE M3]. EULA (engine) + license-split statement (engine EULA vs verifier
    OSS) + DPA template (what a seat processes locally, what the mint service sees: org_id +
    payment metadata ONLY — grounded in M-1 LOCK-2) + refund/termination language for the seat
    model. Drafts for counsel review — authoring, not legal advice; flagged for the user.
(c) TRUST CONTINUITY STATEMENT [WAVE M3]. The solo-maintainer answer (focus-rescope contradiction
    #2): source-escrow-on-lapse commitment (or a license-converts-to-OSS-on-abandonment clause —
    user decision at execution, both drafted), key-ceremony/custody note (M-1/M-2/M-4 keys), and
    the support-expectation page (what a seat gets; honest solo-operation posture, GD-5 — no
    fake SLA).
(d) TIER MATRIX AS DATA [WAVE M1]. tiers.toml: Free / Pro / Team rows -> feature keys M-1's
    tier() gates + K-24's credit grants read. Pricing NUMBERS live here once and nowhere else;
    every rendered surface reads this file. Seed framing per GTM memory (team/agency $39-49
    class) tagged [EXTERNAL-GTM] until K-24's measured $/resolved-task lands (MD-5).

## LOCKs
- LOCK-1 MD-5/KD-1: ALL copy in (b)/(d) passes the K-1 claims lint (extended with a pricing
  recognizer): no unmeasured superlative, no "$20=$200"-class claim before K-24's measured
  number, every external figure tagged.
- LOCK-2 the verifier is extraction, NOT reimplementation (GD-1): the crate re-exports the same
  verify code paths the engine uses; a fork of verify logic = drift = the exact failure the
  attestation exists to prevent. One source of verify truth.
- LOCK-3 the Article-50-certified hook stays KILLED (GTM record; K-6's "designed to support"
  wording is the ceiling).

## COMPOSES-WITH (do NOT reimplement)
G-8 attestation verify, G-2 rra verify, K-6 conformance, M-3 org verify (the four legs the OSS
binary wraps); K-1 dossier + claims lint; K-7 brief (gains the verifier-OSS line); M-1 (tiers.toml
consumer); GTM memory decisions (executed, not re-litigated).

## WIRING_CONTRACT
produces: elai-verify crate/binary (published), EULA/DPA/continuity drafts under docs/strategy/
  commercial/, tiers.toml, claims-lint pricing recognizer extension.
consumes: G-8/G-2/K-6/M-3 verify paths, K-1 lint, GTM decision records.
reachable_from: `elai-verify` standalone binary (real entrypoint, engine-free); tiers.toml read
  by `elai license status`.
trigger: on a CLEAN machine (no engine): `elai-verify attestation fixtures/run.attestation &&
  elai-verify rra fixtures/run.rra`
expect_ssb: none (standalone binary journals nothing; E-23-class legs have no SSB).
observable_effect (done-gate): the standalone binary verifies fixture attestation + RRA + org
  bundle on an engine-free machine (PASS) and catches a one-byte tamper (FAIL, R2); the crate
  compiles with ZERO engine-crate deps (the boundary test); a seeded pricing superlative in the
  tier copy FAILS the lint (MD-5 arm); `elai license status` renders tier names/features from
  tiers.toml (fixture, M-1 join).

## Risks (top 3)
R1 verifier extraction reveals engine entanglement in verify paths: that is a FINDING, not a
  blocker — the refactor to a clean boundary is in-scope for (a) and improves the engine.
R2 legal drafts treated as final: every (b)/(c) artifact carries a counsel-review-required
  banner; the user ships nothing external without review (house rule: user reviews before any
  external use, same as K-7).
R3 tiers.toml drift vs marketing copy: the lint's pricing recognizer greps rendered copy against
  tiers.toml values — one source of pricing truth, mechanically enforced.
