# M-plan — Summary Index of all 6 sessions (single track COMMERCIAL SHIPPING / MULTI-SEAT — the closing letter after B..L)
M-6 appended 2026-07-05 (evidence-format open standard, from the spec-eval run
workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md ADD-1, on explicit user direction "make the
new plan files"; the ENTERPRISE RIDER's informal "M-6+" slot renumbered to M-7+ in M-0_INDEX).
Generated: 2026-07-02 (one wave, main LLM in-band; idea-to-plan via the paper-to-plan PHASE 4-5
protocol, no paper; gap analysis grep-proven at workflow/reasoning/steal-multiseat-gap-2026-07-02.md;
user explicitly authorized the new letter this session — K-10 logged deviation on AUTHORING only,
M EXECUTION stays behind the K-10 order per M-0 GATE M-0). Mirrors H/I/J/K/L-SUMMARY format.
Doctrine + MD-1..MD-5 + Section-A pointers MA-1..MA-7 + the ENTERPRISE RIDER: M-0_INDEX.md.
LIVE 268/577 (K stamp 2026-06-26), re-grep at edit (W-9). Net new SSB across M: +2..+4.

## Premise
B..L finished = a sellable SINGLE seat; M makes it sellable TO A COMPANY. The corpus deliberately
deferred the org layer (K-0:531 "No org/SSO/RBAC/multi-tenant build in K"); M is its minimal,
offline-first, no-org-SaaS form. ~60% composition (K-24 rail, J-49/J-59 per-seat, B-43 installer,
K-6/K-7 evidence, G-8/G-9, K-1 lint); net-new = exactly the commerce machinery no letter owns
(entitlement, org pack, fleet rollup, updater, legal wrapper). Delta family: MD-1 never-brick
(commerce fail-OPEN to Free tier; safety gates untouched — the one recorded R1 inversion), MD-2
offline-verifiable commerce (G-8 pattern everywhere; license service never a runtime dep), MD-3
minimal first-party cloud (ONE mint endpoint; policy + evidence travel by file/git; SSO/SCIM
demand-gated rider), MD-4 seat-privacy default (J-59 isolation; rollups = numbers + hashes, never
transcripts), MD-5 claims discipline on money (K-1 lint over all pricing/EULA copy; value claims
gated on K-24's measured number).

### M-1 — seat entitlement + purchase rail [M KEYSTONE]
- Track M | lever SELLABLE | grounding STRONG (gaps G1/G2 grep-proven) | nature BUILD | effort M-L | WAVE M1
- Build: signed offline-verifiable Entitlement{org_id, tier, seats_max, features, expiry, sig}
  (G-8 signing core REUSED); `elai license activate|status|deactivate`; 14-day grace ->
  DegradedToFree (MD-1: never bricks, safety gates untouched); MoR payment webhook (outside ELAI,
  PCI zero — K-24 decision inherited) -> the ONE first-party mint endpoint (mint/renew/revoke
  only, MD-3) -> seat activation; tier() read gates K-24 credit grants + M-2/M-3 Team features
  (data-driven via M-5d tiers.toml, TD-3). Seat-count = honest client-side best-effort; over-use
  surfaces in M-3 (recorded limit, GD-5).
- COMPOSES-WITH: G-8 signing, K-24 grant path + seats.toml, ClockProvider (all expiry math),
  B-43 (activation offered at install-end). Restates none.
- Integration: crates/orchestrator/src/commerce/entitlement.rs; +EntitlementEvent (Pattern-2:
  Activated/Renewed/ExpiredGrace/DegradedToFree/Revoked/RefusedForged). frontend_pair: license
  status line in SettingsWindow (decide at edit, W-4).
- observable_effect: fixture Team entitlement activates OFFLINE -> Activated + Team unlock;
  SeededClock expiry -> ExpiredGrace -> DegradedToFree AND a Free task still runs green AND every
  safety gate still fires (MD-1 negative arm); forged signature REFUSED (R2); over-activation
  RECORDED not absorbed.

### M-2 — org policy pack + distribution + drift detection
- Track M | lever CANT-WRECK (org grain) | grounding STRONG (gap G3; 3F/R8/E-39 shipped) | nature BUILD-compose | effort M | WAVE M2 | depends M-1
- Build: ONE signed file-distributable OrgPolicyPack (org rules -> org-immune tier, gate-manifest
  pins, E-39 spend caps, B-37 allowlist, G-9 config, M-4 channel pin) applied at boot; a seat can
  ADD stricter checks, never weaken (monotone-tighten, the R8 property lifted to org grain);
  drift DETECTED + surfaced (never auto-corrected pre-GD-4); pack = DATA never executable (L-13
  FS-2 at org grain); no pack = byte-identical today's behavior (W-2 structural).
- COMPOSES-WITH: 3F lifecycle + mutable-tier, gate_freeze_check/manifest (R8 — pack PINS it),
  E-39, K-5 policy-table hash, G-9, G-8 signing, M-1 tier. Restates none.
- Integration: commerce/org_policy.rs + `elai org policy init|sign|verify|show`; +OrgPolicyEvent
  (Pattern-2: Applied/Drift/RefusedForged/WeakenAttemptBlocked).
- observable_effect: fixture pack boots Applied + a task blocked by the ORG cap naming the pack
  clause; tampered pack REFUSED; local cap-raise -> WeakenAttemptBlocked; no-pack seat
  byte-identical (regression arm); stale pack -> Drift at boot.

### M-3 — fleet evidence rollup (`elai org report`)
- Track M | lever AUDITABLE (org grain) | grounding STRONG (gap G4) | nature BUILD-compose | effort M | WAVE M3 | depends M-1, M-2, MA-3 (J-59), MA-6 (K-6)
- Build: pull-model aggregation — seats `elai export compliance --org` (K-6 bundle + seat_id/
  org_id/pack_hash/drift fields, serde-default); admin `elai org report <dir> --verify` -> ONE
  org bundle: per-seat table + rollups (verified-outcome rate, K-4 gate-tax, K-1 deltas, H-9
  $/resolved-task, K-3 unvouched counts, M-2 drift, seat-roster honesty vs seats_max) — every
  number recomputable from constituent journals (GD-2 at fleet grain); zero telemetry, zero raw
  content (MD-4); broken-chain seats REFUSED + NAMED; defines ZERO new metrics (KD-1).
- COMPOSES-WITH: K-6 (input shape extended, not forked), G-2/G-8 verify, G-9 (precondition +
  aggregate self-test), K-1/K-4/H-9/K-3, M-1/M-2. Restates none.
- Integration: commerce/org_report.rs; SSB prefer-reuse (+OrgReportEvent only if nothing fits, W-9).
- observable_effect: 3 fixture exports (clean/drift/broken) -> one bundle; 2 verify PASS, broken
  seat RefusedSeat-named (R2); rollup numbers recompute byte-identically; seeded secret in a
  constituent FAILS the build; transcript-byte grep of the bundle EMPTY (MD-4 arm).

### M-4 — signed release channel + auto-updater + support bundle
- Track M | lever REACH + CANT-WRECK | grounding STRONG (gaps G5/G6; B-43:15 defers the updater by name) | nature BUILD | effort M | WAVE M2 | depends MA-4, M-2, G-9
- Build: signed release manifests (per-artifact blake3, B-43 refuse-on-mismatch core REUSED;
  revocation-list distribution = M-1's key-compromise lever), stable/beta channels + org pin
  (M-2 clause) + deterministic staged rollout (blake3(seat_id)%100 — journal-explainable, GD-2),
  `elai update check|apply|rollback|channel` (check default-ON non-blocking; apply user-confirmed
  or org-scheduled — never silent, W-2 clean); rollback keeps last-good binary (J-3 discipline on
  binaries); `elai support bundle` = LOCAL G-9-redacted diagnostic (versions, tier, hashes,
  doctor, redacted journal tail), printed summary, NEVER auto-sent (MD-4). v1 whole-artifact;
  delta updates a later rider.
- COMPOSES-WITH: B-43 HostInstaller + manifest pattern (sibling, shared verify code), B-45, M-2,
  G-9, doctor, G-8. Restates none.
- Integration: crates/cli/src/update.rs + support bundle arm; +ReleaseChannelEvent (Pattern-2) or
  prefer-reuse (W-9).
- observable_effect: corrupted-artifact fixture REFUSED before any file lands (R2); clean apply +
  rollback restores prior binary (hash match); org stable-pin hides beta manifests; support
  bundle on a seeded-secret fixture session passes the secret-grep EMPTY (G-9 arm).

### M-5 — commercial packaging + verifier OSS + trust continuity
- Track M | lever SELLABLE + AUDITABLE | grounding STRONG (gap G7; GTM decisions on file, executed not re-litigated) | nature E-23-class + ONE extraction | effort S-M | WAVE M1 (a,d) / M3 (b,c)
- Build: (a) standalone `elai-verify` OSS crate/binary wrapping the SAME verify paths (G-8
  attestation + G-2 rra + K-6 conformance + M-3 org bundle) with ZERO engine deps — the
  extraction is a compile-time boundary proof; engine stays closed (GTM decision executed);
  (b) EULA + license-split + DPA drafts (counsel-review banner; mint service sees org_id +
  payment metadata ONLY); (c) trust-continuity statement — escrow-on-lapse or convert-to-OSS-on-
  abandonment (user picks; both drafted), key-custody note, honest solo support posture (GD-5;
  no fake SLA); (d) tiers.toml — Free/Pro/Team as DATA, the ONE pricing-truth source M-1/K-24
  read; GTM $39-49 seat framing tagged [EXTERNAL-GTM] until K-24's measured number (MD-5).
  Article-50-certified hook stays KILLED (LOCK-3).
- COMPOSES-WITH: G-8/G-2/K-6/M-3 verify legs, K-1 dossier + claims lint (+pricing recognizer),
  K-7 brief, M-1. Restates none.
- Integration: elai-verify crate (published) + docs/strategy/commercial/ + tiers.toml + lint
  extension. NO SSB.
- observable_effect: on a CLEAN engine-free machine the standalone binary verifies fixture
  attestation/RRA/org-bundle and catches a one-byte tamper (R2); crate compiles with zero
  engine-crate deps; a seeded pricing superlative FAILS the lint (MD-5 arm); `elai license
  status` renders from tiers.toml.

### M-6 — evidence format as a versioned open standard [the schema, not just the verifier; the format-moat play]
- Track M | lever REACH + AUDITABLE | grounding STRONG (spec-eval 2026-07-05 ADD-1: a solo author cannot
  win a plugin-ecosystem war but can own the FORMAT agent evidence is exchanged in; SLSA/in-toto precedent
  [training, unsearched — re-verify at execution]) | nature BUILD-compose + docs | effort M | WAVE M3 |
  depends M-5a (elai-verify extracted), G-2/G-8/K-6 emit paths stable
- Build: publish "EEF v1" (name at execution) — the versioned, machine-readable spec of the RRA bundle,
  attestation predicate, compliance bundle, and verdict/receipt row shapes, so third-party tools can EMIT
  ELAI-verifiable evidence and elai-verify becomes the reference verifier of a FORMAT. Slices: (a) schemas
  DERIVED from the engine's serialization types (one source of schema truth — never hand-forked); (b)
  prose spec + conformance fixtures + `elai-verify conform <artifact>` (an M-5 binary extension) with
  fail-closed semantics NORMATIVE (missing evidence = FAIL is a conformance requirement, R1); (c)
  append-only schema evolution behind a golden-file lock joining the frozen gate-manifest (R8, E-45
  pattern); (d) publication beside elai-verify + a worked third-party-emitter example. "Open standard"
  wording is KD-1-gated on >=1 EXTERNAL emitter (until then: "published format"); the K-1 lint gains an
  adoption-claim recognizer. PROOF_SPINE: none directly (distribution amplifier; spine-exempt, user-acked).
- COMPOSES-WITH: M-5a (the binary), G-2/G-8/K-6 (the emit paths described, never forked), E-45 golden
  lock, K-1 lint, K-3/K-11 (the rendered shapes), gate-manifest R8. Engine stays closed (GTM record).
- observable_effect: a conformant fixture PASSes + a one-byte-tampered/missing-field fixture FAILS
  `elai-verify conform` on an engine-free machine (R2); the derived schemas byte-match a regeneration
  from the engine types (drift arm); a schema field removal trips the golden lock (R8); a seeded
  "industry standard" phrase FAILS the K-1 lint until the adoption row exists; the third-party example
  emits a row conform accepts. NO SSB (standalone verifier + docs).

## Cross-cutting (what makes M different from B..L)
Every prior letter made the seat TRUSTWORTHY; M makes it PURCHASABLE, POLICEABLE, and AUDITABLE at
org grain without becoming an org SaaS: one mint endpoint, everything else offline-verifiable
files (MD-2/MD-3). Commerce is the one deliberately fail-OPEN layer (MD-1) — the trust product
never bricks a seat — while every safety gate stays fail-closed. The org buyer gets the M-3
answer to K-7's questions computed from seat journals, not vendor dashboards (GD-2 at fleet
grain). SSO/SAML/SCIM/RBAC + hosted console = demand-gated M-6+ rider (K-0:531 + GTM
"agencies-not-enterprise" upheld). Sequencing: M is authored as the CLOSING letter and executes
last — GATE M-0 holds all M sessions behind K's A-1..A-4 + K-3 + K-24 L1..L4, because selling
before the measured number exists is exactly the claim discipline (KD-1/MD-5) forbids.

End of M-SUMMARY_INDEX.
