# Multi-seat / company-shipping gap analysis — 2026-07-02
Session: idea-to-plan (paper-to-plan PHASE 4 analog, no paper). Question from the user: "if all
plans B..L finish, is ELAI enough to charge for a seat? are there gaps around shipping to
companies with multiseat?" Sources: full read of phase-index-MASTER + B..L SUMMARY_INDEX files
(prior turn), targeted greps over docs/thefinalglance (bash grep — the tree is gitignored, rg
skips it), ELAI-FEATURE-MAP.md group headers + 1.16 + SECTION 2, K-0_INDEX org-layer non-goal
lines, GTM memory (project_gtm_playbook_2026 / project_focus_rescope_2026).

## Verdict on the question
Capability-wise, B..L finished = a sellable SINGLE seat (core loop + trust spine + K packaging +
L-14 one-shot front door). Commercially, the corpus CANNOT sell that seat to a company: nothing
issues/validates/meters a seat entitlement, nothing distributes org policy, nothing aggregates
evidence across seats for the org-side buyer K-7 targets, and the auto-updater B-43 names is
unauthored. Gap confirmed -> new closing letter M (user explicitly authorized a new letter this
message; K-10 logged-deviation pattern per L-0_INDEX header precedent).

## What EXISTS (M composes, never restates — GD-1)
- K-24 BCC billing/credits: credit ledger as SSB sub-journal, reserve/settle, OpenRouter rail,
  payment webhook OUTSIDE ELAI (PCI zero), seats.toml, team/agency $39-49/seat framing. This is
  METERING, not seat LICENSING — nothing binds a paying org to an installed seat.
- J-49 multi-seat admission (local-serving gateway, per-user quota) + J-59 per-seat memory +
  trust isolation (the data-governance slice). K-0:382: cross-seat/org-shared memory OUT until a
  separate per-seat opt-in decision.
- K-23 MPW multi-repo workspace (single-user, cross-REPO — a different axis, its own file says so).
- B-43 one-line install (HostInstaller) + B-45 dashboard installer (deferred) + "separate plan
  auto-updater (delta updates + signature pinning)" — NAMED in B-43:15, never authored.
- K-6 compliance export / K-7 appsec+insurance brief / K-13 OWASP mapping / K-1 claims lint —
  buyer-side evidence, all PER-SEAT/PER-MACHINE artifacts.
- K-3/K-11 PR evidence emitter — per-repo distribution leg.
- K-0_INDEX:531 (verbatim intent): "No org/SSO/RBAC/multi-tenant build in K (artifacts speak
  compliance first; org layer only on ..." — the deferral is deliberate and recorded.
- E-50 SaaS archetypes/RLS/multi-tenancy = for apps ELAI GENERATES, not for ELAI itself (checked;
  not a coverage overlap).

## The GAPS (grep-confirmed zero coverage anywhere in B..L)
G1 Seat entitlement: no license issuance, activation, verification (online or offline), seat
   count, tier gating, expiry/grace, revocation. Zero hits for license-key/entitlement machinery.
G2 Purchase->entitlement loop: K-24 stops at "payment webhook outside ELAI"; nothing turns a
   payment into a verifiable seat artifact on the buyer's machine.
G3 Org policy distribution: mutable=0 rules + gate-manifest freeze are per-install; no signed org
   policy pack, no cross-seat enforcement, no drift detection.
G4 Fleet evidence rollup: RRA/attestation/dossier/gate-tax are per-machine; the org buyer (K-7's
   AppSec/GRC persona) has no `org report` aggregating seats.
G5 Release channel: signed releases/auto-update/channel pin/staged rollout/rollback — unauthored
   (B-43 defers it by name).
G6 Support path: no redacted diagnostic bundle (G-9 substrate exists, no consumer).
G7 Commercial packaging: EULA/DPA/pricing copy under the K-1 claims lint; the GTM decision
   "open-source ONLY the verifier" has no executing session anywhere in B..L.
G8 (recorded, NOT planned) Enterprise identity: SSO/SAML/SCIM/RBAC — stays demand-gated per
   K-0:531 + GTM "agencies NOT enterprise". M records the rider, builds nothing.

## Outcome
Authored M-plan (docs/thefinalglance/M-plan/): M-0_INDEX + M-1..M-5 + M-SUMMARY_INDEX; feature-map
upserted (group 1.18 + M ledger line). M-1 seat entitlement + purchase rail [KEYSTONE], M-2 org
policy pack, M-3 fleet evidence rollup, M-4 release channel + updater + support bundle, M-5
commercial packaging + verifier OSS + trust continuity. All M execution gated AFTER K's A-1..A-4 +
K-3 + K-24 L1..L4 (K-10 order upheld; M is the closing letter, not a queue-jumper).
