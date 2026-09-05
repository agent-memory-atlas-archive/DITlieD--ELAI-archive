# M-plan — INDEX (COMMERCIAL SHIPPING / MULTI-SEAT: the closing letter)
Generated: 2026-07-02 (one wave, main LLM in-band; idea-to-plan via paper-to-plan PHASE 4-5;
gap analysis at workflow/reasoning/steal-multiseat-gap-2026-07-02.md). 6 sessions M-1..M-6 +
7 Section-A execution pointers (MA-1..MA-7). M-6 appended 2026-07-05 (evidence-format open
standard; spec-eval ADD-1, user-directed; the rider's informal "M-6+" slot renumbered to M-7+). Doctrine inherits F-0 W-1..W-10 + G-0 GD-1..GD-5 +
K-0 KD-1..KD-5 and ADDS MD-1..MD-5.

USER OVERRIDE (recorded honestly, L-0_INDEX precedent): K-10 defers new plan LETTERS until
A-1..A-4 + K-3 are DONE. The user explicitly authorized this letter 2026-07-02 ("write down a new
plan letter... to be the closing off after B to L is done"). Logged deviation on AUTHORING only —
M EXECUTION stays gated behind the K-10 order (see WAVES). M is authored last and runs last.

## Premise
B..L finished = a sellable SINGLE seat. M makes the seat SELLABLE TO A COMPANY. Every prior letter
verified the agent's work; M packages the verified seat into something an org can buy, deploy,
police, and audit: a seat entitlement a payment can mint and a machine can verify offline (M-1),
an org policy pack every seat obeys and cannot weaken (M-2), a fleet evidence rollup that answers
the org-side buyer K-7 targets (M-3), a signed release channel + updater + support path (M-4), and
the commercial/legal wrapper incl. the GTM open-source-verifier decision executed (M-5). ~60%
composition of shipped/planned substrate (K-24 billing rail, J-49/J-59 per-seat, B-43 installer,
K-6/K-7 evidence pack, G-8/G-9 signing/redaction, K-1 claims lint); the net-new is exactly the
commerce machinery no prior letter owns (grep-proof in the gap file).

## Delta family (MD-1..MD-5)
MD-1 NEVER-BRICK: entitlement failure/expiry degrades to the FREE tier — never blocks local work,
  never touches safety gates. Commerce is fail-OPEN by design (the one deliberate, recorded
  inversion of R1 — a trust product must not rug-pull its own seats); safety gates stay
  fail-closed exactly as before. The free tier is the base state, so W-2 is satisfied without an
  env gate.
MD-2 OFFLINE-VERIFIABLE COMMERCE: every entitlement / policy pack / fleet bundle verifies OFFLINE
  (the G-8 local-Merkle/signature pattern). The license service is issue/renew/revoke ONLY —
  never a runtime dependency; a seat that never phones home keeps working (MD-1 floor).
MD-3 MINIMAL FIRST-PARTY CLOUD: exactly one tiny endpoint (mint/renew/revoke). Org policy and
  fleet evidence stay file/git-distributable. NO SSO/SAML/SCIM/RBAC/org-console build — the
  K-0:531 non-goal + GTM "agencies NOT enterprise" decision stand; see ENTERPRISE RIDER.
MD-4 SEAT-PRIVACY DEFAULT: per-seat journal/memory isolated (J-59); org sharing opt-in per
  artifact class (K-0:382 decision upheld); rollups aggregate NUMBERS + HASHES, never raw
  transcripts (G-9 is a precondition of every export).
MD-5 CLAIMS DISCIPLINE ON MONEY (KD-1 extended): no pricing/value/tier copy ships outside the
  K-1 claims lint; "$20=$200"-class claims stay gated on K-24's measured $/resolved-task number.

## Section-A execution pointers (EXISTING files M only sequences — never re-authors, GD-1)
MA-1  K-24 BCC billing/credits — the metering rail M-1's entitlement gates (L1..L4 before M-1 live leg).
MA-2  J-49 multi-seat local-serving admission — per-user quota on a shared box; M-1 tier feeds it.
MA-3  J-59 per-seat memory + trust isolation — the data-governance slice; gates M-3's rollup.
MA-4  B-43 one-line install + B-45 dashboard installer — M-4 builds the updater they defer.
MA-5  K-3/K-11 PR evidence emitter — the per-repo distribution leg M-3 aggregates above.
MA-6  K-6/K-7/K-13 compliance export + briefs — the per-seat artifacts M-3 rolls up, M-5 wraps.
MA-7  K-23 multi-repo workspace — orthogonal axis; noted to prevent scope confusion.

## Sessions
M-1  seat entitlement + purchase rail [KEYSTONE]           — WAVE M1
M-2  org policy pack + distribution + drift detection      — WAVE M2
M-3  fleet evidence rollup (`elai org report`)             — WAVE M3 | depends M-1, MA-3, MA-6
M-4  signed release channel + auto-updater + support bundle — WAVE M2 | depends MA-4
M-5  commercial packaging + verifier OSS + trust continuity — WAVE M1(a,d)/M3(b,c)
M-6  evidence format as a versioned open standard (EEF v1)  — WAVE M3 | depends M-5a

## Waves + the K-10 gate
GATE M-0: no M session EXECUTES until K's A-1..A-4 + K-3 are DONE and K-24 slices L1..L4 are live
(the number exists, the distribution leg exists, the metering rail exists — selling before those
is selling an unmeasured claim, KD-1). Then:
  WAVE M1: M-1; M-5a (verifier OSS extraction) + M-5d (tiers.toml).
  WAVE M2: M-2; M-4.
  WAVE M3: M-3; M-5b (EULA/DPA/pricing under lint) + M-5c (continuity statement); M-6 (evidence
  format standard — after M-5a extraction + stable emit paths).

## ENTERPRISE RIDER (recorded, NOT planned)
SSO/SAML/SCIM/RBAC + hosted org console = a FUTURE M-7+ (renumbered 2026-07-05; M-6 = the
evidence-format standard), DEMAND-GATED: authored only when >=3
paying orgs ask, or a signed deal is blocked on it. Until then the K-0:531 non-goal stands and the
M-2 file-distributed policy pack is the org story. Re-proposing an org SaaS before the gate fires
is a P0-grade miss (feature-map Section 4 discipline).

## SSB budget (W-9)
LIVE baseline 268/577 (K-plan stamp 2026-06-26, ssb.rs:1508 / action_type_lock.rs:775 — re-grep at
edit). M net +2..+4: +EntitlementEvent (M-1, Pattern-2), +OrgPolicyEvent (M-2, Pattern-2),
optional +OrgReportEvent (M-3, prefer-reuse a K-6/G-8 row first), optional +ReleaseChannelEvent
(M-4, prefer-reuse). M-5 = zero SSB (E-23-class + one crate extraction).

## Grounding (W-7)
Live anchors below carried from the K/L 2026-06-26..27 stamps + the 2026-07-02 gap-file greps;
_grounding/M-GROUNDING-MAP.md is generated at the FIRST M execution session (before any code) —
every anchor re-grepped then. Gap-existence claims are grep-proven in the gap file; external
market/pricing figures are [EXTERNAL] and live only in M-5's lint-governed copy.
