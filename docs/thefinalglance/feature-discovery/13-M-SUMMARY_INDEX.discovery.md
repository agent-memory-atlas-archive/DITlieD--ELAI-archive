# ELAI feature discovery — M commercial shipping / multi-seat

## Scope and coverage

- Primary source read completely: docs/thefinalglance/M-plan/M-SUMMARY_INDEX.md, 146/146 lines.
- Orientation only: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md. It is not used below to add M behavior absent from the M summary.
- Session coverage: 6/6 named sessions, M-1 through M-6.
- Doctrine coverage: MD-1 through MD-5.
- Evidence recorded: build behavior, commands and user surfaces, data/state/lifecycle, composition points, security/privacy/compliance, failure and fallback rules, observability/event shapes, acceptance tests, dependencies, sequencing, status, and deferred work.
- Original-plan policy: original M plan files were existence-checked and inventoried, but not read. The explicit inventory below is therefore summary-bounded; deeper-source candidates are listed for later agents.
- Classification rule: the numbered tables contain only statements explicit in the summary. Inferences and unresolved choices are isolated in their own sections.
- Source-anchor shorthand: M:Lx-Ly means lines x through y of M-SUMMARY_INDEX.md.

## Coverage map

| Unit | Summary lines | Grounding / nature / effort | Wave | Explicit dependency gate |
|---|---:|---|---|---|
| M-wide premise/doctrine | 1-23, 135-144 | Closing commercial-shipping and multi-seat letter; approximately 60% composition | Closing letter; executes last | M-0 gate behind K A-1..A-4, K-3, and K-24 L1..L4 |
| M-1 — seat entitlement + purchase rail | 25-42 | STRONG; BUILD; M-L; M KEYSTONE | M1 | Upstream compositions only |
| M-2 — org policy pack + distribution + drift detection | 44-57 | STRONG; BUILD-compose; M | M2 | M-1 |
| M-3 — fleet evidence rollup | 59-72 | STRONG; BUILD-compose; M | M3 | M-1, M-2, MA-3/J-59, MA-6/K-6 |
| M-4 — signed release channel + auto-updater + support bundle | 74-90 | STRONG; BUILD; M | M2 | MA-4, M-2, G-9 |
| M-5 — commercial packaging + verifier OSS + trust continuity | 92-110 | STRONG; E-23-class plus one extraction; S-M | M1 for a/d; M3 for b/c | Upstream verify, claim, and brief legs |
| M-6 — evidence format as a versioned open standard | 112-133 | STRONG with precedent recheck caveat; BUILD-compose plus docs; M | M3 | M-5a and stable G-2/G-8/K-6 emit paths |

## Classification legend

- CAP: product capability or boundary.
- ACTION/CLI/UI: action or user-facing surface.
- DATA/STATE/LIFE/CFG: data shape, state, lifecycle, or configuration.
- FLOW/INT: workflow or integration.
- SAFE/PRIV/COMP/CLAIM: safety, privacy, compliance, or claims discipline.
- OBS: journal/event/diagnostic observability.
- FAIL/FALLBACK: refusal, degradation, or fallback.
- TEST/OPS: specified observable proof or operational behavior.
- STATUS/DEFER: plan state, sequencing, or explicitly deferred work.

## Explicit atomic inventory

### M-wide product boundary, doctrine, and sequencing

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| MX-E001 | CAP | M is a single-track commercial-shipping / multi-seat letter closing B through L. | M:L1 |
| MX-E002 | STATUS | The index covers six M sessions. | M:L1 |
| MX-E003 | STATUS | M-6 was appended on 2026-07-05 from spec-eval ADD-1 following explicit user direction. | M:L2-L4 |
| MX-E004 | STATUS | The enterprise rider’s informal M-6+ slot was renumbered M-7+ in M-0_INDEX. | M:L2-L4 |
| MX-E005 | STATUS | The original M wave was generated 2026-07-02 with the main LLM in-band via paper-to-plan phases 4-5 and no paper. | M:L5-L6 |
| MX-E006 | STATUS | The multi-seat gap analysis was grep-proven. | M:L5-L7 |
| MX-E007 | STATUS | K-10’s logged deviation applies to authoring only; M execution remains behind the K-10 order and M-0 gate. | M:L7-L8 |
| MX-E008 | STATUS | M-0_INDEX owns doctrine MD-1..MD-5, Section-A pointers MA-1..MA-7, and the enterprise rider. | M:L9 |
| MX-E009 | STATUS | The recorded live stamp is 268/577 from K on 2026-06-26, with a W-9 re-grep required at edit. | M:L10 |
| MX-E010 | OBS | M expects only +2 through +4 net-new SSB event families/rows across the letter. | M:L10 |
| MX-E011 | CAP | B through L are treated as a sellable single-seat product; M makes it sellable to a company. | M:L13-L14 |
| MX-E012 | CAP | M is deliberately a minimal, offline-first organization layer. | M:L14-L15 |
| MX-E013 | CAP | M explicitly does not become an organization SaaS. | M:L14-L15 |
| MX-E014 | INT | M is approximately 60% composition of K-24, J-49/J-59, B-43, K-6/K-7, G-8/G-9, and K-1. | M:L15-L16 |
| MX-E015 | CAP | Net-new commerce machinery includes entitlement. | M:L16-L17 |
| MX-E016 | CAP | Net-new commerce machinery includes an organization policy pack. | M:L16-L17 |
| MX-E017 | CAP | Net-new commerce machinery includes a fleet rollup. | M:L16-L17 |
| MX-E018 | CAP | Net-new commerce machinery includes an updater. | M:L16-L17 |
| MX-E019 | CAP | Net-new commerce machinery includes a legal wrapper. | M:L16-L17 |
| MX-E020 | FALLBACK | MD-1 is the never-brick rule: commerce fails open to the Free tier. | M:L17-L18 |
| MX-E021 | SAFE | MD-1 does not weaken or disable safety gates. | M:L17-L19 |
| MX-E022 | STATUS | MD-1 is the one recorded inversion of the R1 fail-closed rule. | M:L17-L19 |
| MX-E023 | CAP | MD-2 requires commerce to be offline-verifiable, reusing the G-8 pattern. | M:L19 |
| MX-E024 | SAFE | The license service must never be a runtime dependency. | M:L19 |
| MX-E025 | CAP | MD-3 permits exactly one first-party cloud endpoint, the mint endpoint. | M:L19-L20 |
| MX-E026 | FLOW | Organization policy and evidence travel by file or git rather than a hosted control plane. | M:L20-L21 |
| MX-E027 | DEFER | SSO and SCIM are demand-gated rather than part of the minimal first-party cloud. | M:L20-L21 |
| MX-E028 | PRIV | MD-4 defaults to per-seat isolation by composing J-59. | M:L21-L22 |
| MX-E029 | PRIV | Fleet rollups contain numbers and hashes. | M:L21-L22 |
| MX-E030 | PRIV | Fleet rollups never contain transcripts. | M:L21-L22 |
| MX-E031 | CLAIM | MD-5 applies K-1 claims lint to all pricing and EULA copy. | M:L22-L23 |
| MX-E032 | CLAIM | Monetary value claims are blocked until K-24 supplies a measured number. | M:L22-L23 |
| MX-E033 | CAP | At organization grain, the finished product is purchasable, policeable, and auditable without being an org SaaS. | M:L135-L138 |
| MX-E034 | FLOW | Apart from the one mint endpoint, organization commerce, policy, and evidence use offline-verifiable files. | M:L137-L138 |
| MX-E035 | FALLBACK | Commerce is the sole deliberately fail-open layer and degrades a seat rather than bricking it. | M:L138-L139 |
| MX-E036 | SAFE | Every safety gate remains fail-closed even when commerce fails open. | M:L138-L139 |
| MX-E037 | AUDIT | Organization answers are computed from seat journals, not vendor dashboards. | M:L139-L141 |
| MX-E038 | DEFER | SSO, SAML, SCIM, RBAC, and a hosted console are demand-gated enterprise-rider capabilities. | M:L141-L142 |
| MX-E039 | CLAIM | The demand-gated rider preserves the recorded agencies-not-enterprise go-to-market posture. | M:L141-L142 |
| MX-E040 | STATUS | M is authored as the closing letter and executes last. | M:L142-L143 |
| MX-E041 | STATUS | GATE M-0 holds every M session behind K A-1..A-4, K-3, and K-24 L1..L4. | M:L143-L144 |
| MX-E042 | CLAIM | Selling before K-24’s measured number exists is forbidden by KD-1/MD-5 claim discipline. | M:L143-L144 |

### M-1 — seat entitlement + purchase rail

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M1-E001 | STATUS | M-1 is the M keystone, lever SELLABLE, STRONG-grounded by gaps G1/G2, BUILD, effort M-L, wave M1. | M:L25-L26 |
| M1-E002 | DATA | Entitlements are signed. | M:L27-L28 |
| M1-E003 | SAFE | Entitlement validity is verifiable offline. | M:L27-L28 |
| M1-E004 | DATA | Entitlement carries org_id. | M:L27 |
| M1-E005 | DATA | Entitlement carries tier. | M:L27 |
| M1-E006 | DATA | Entitlement carries seats_max. | M:L27 |
| M1-E007 | DATA | Entitlement carries features. | M:L27 |
| M1-E008 | DATA | Entitlement carries expiry. | M:L27 |
| M1-E009 | DATA | Entitlement carries sig. | M:L27 |
| M1-E010 | INT | Entitlement signing reuses the G-8 signing core. | M:L27-L28 |
| M1-E011 | CLI | The CLI exposes 'elai license activate'. | M:L28 |
| M1-E012 | CLI | The CLI exposes 'elai license status'. | M:L28 |
| M1-E013 | CLI | The CLI exposes 'elai license deactivate'. | M:L28 |
| M1-E014 | LIFE | An entitlement has a 14-day grace period. | M:L28-L29 |
| M1-E015 | LIFE | Expiration after grace transitions the seat to DegradedToFree. | M:L28-L29 |
| M1-E016 | FALLBACK | Entitlement expiry never bricks the seat; it retains Free-tier operation. | M:L29 |
| M1-E017 | SAFE | Entitlement degradation leaves safety gates untouched. | M:L29 |
| M1-E018 | FLOW | A merchant-of-record payment webhook sits outside ELAI. | M:L29-L30 |
| M1-E019 | COMP | Keeping the payment webhook outside ELAI gives ELAI zero PCI handling under the inherited K-24 decision. | M:L29-L30 |
| M1-E020 | FLOW | Payment fulfillment flows from the MoR webhook to the one first-party mint endpoint and then to seat activation. | M:L29-L31 |
| M1-E021 | ACTION | The mint endpoint can mint an entitlement. | M:L30-L31 |
| M1-E022 | ACTION | The mint endpoint can renew an entitlement. | M:L30-L31 |
| M1-E023 | ACTION | The mint endpoint can revoke an entitlement. | M:L30-L31 |
| M1-E024 | CAP | The mint endpoint is limited to mint, renew, and revoke. | M:L30-L31 |
| M1-E025 | CFG | A tier() read determines commercial feature access. | M:L31 |
| M1-E026 | INT | tier() gates K-24 credit grants. | M:L31 |
| M1-E027 | INT | tier() gates M-2 Team features. | M:L31-L32 |
| M1-E028 | INT | tier() gates M-3 Team features. | M:L31-L32 |
| M1-E029 | CFG | Tier behavior is data-driven through M-5d tiers.toml under TD-3. | M:L32 |
| M1-E030 | LIFE | Seat counting is an honest client-side best-effort mechanism. | M:L32-L33 |
| M1-E031 | OBS | Seat over-use is surfaced by M-3. | M:L32-L33 |
| M1-E032 | FAIL | Over-activation is recorded rather than silently absorbed. | M:L32-L33, L41-L42 |
| M1-E033 | INT | ClockProvider owns all entitlement-expiry math. | M:L34-L35 |
| M1-E034 | UI | B-43 offers activation at the end of installation. | M:L34-L35 |
| M1-E035 | INT | The planned integration module is crates/orchestrator/src/commerce/entitlement.rs. | M:L36 |
| M1-E036 | OBS | M-1 adds an EntitlementEvent family. | M:L36-L37 |
| M1-E037 | OBS | EntitlementEvent has Activated. | M:L36-L37 |
| M1-E038 | OBS | EntitlementEvent has Renewed. | M:L36-L37 |
| M1-E039 | OBS | EntitlementEvent has ExpiredGrace. | M:L36-L37 |
| M1-E040 | OBS | EntitlementEvent has DegradedToFree. | M:L36-L38 |
| M1-E041 | OBS | EntitlementEvent has Revoked. | M:L36-L38 |
| M1-E042 | OBS | EntitlementEvent has RefusedForged. | M:L36-L38 |
| M1-E043 | UI | A license-status line is paired with SettingsWindow. | M:L37-L38 |
| M1-E044 | STATUS | Exact SettingsWindow presentation is deferred to edit time under W-4. | M:L37-L38 |
| M1-E045 | TEST | A Team entitlement fixture must activate offline. | M:L39 |
| M1-E046 | TEST | Offline activation must emit Activated. | M:L39 |
| M1-E047 | TEST | Offline activation must unlock Team features. | M:L39 |
| M1-E048 | TEST | SeededClock expiry must produce ExpiredGrace followed by DegradedToFree. | M:L40 |
| M1-E049 | TEST | After degradation, a Free task must still complete green. | M:L40-L41 |
| M1-E050 | TEST | After degradation, every safety gate must still fire. | M:L40-L41 |
| M1-E051 | TEST | A forged signature must be refused under R2. | M:L41 |
| M1-E052 | TEST | An over-activation fixture must produce a recorded signal. | M:L41-L42 |

### M-2 — org policy pack + distribution + drift detection

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M2-E001 | STATUS | M-2 has lever CANT-WRECK at organization grain, STRONG grounding from gap G3 and shipped 3F/R8/E-39, BUILD-compose, effort M, wave M2. | M:L44-L45 |
| M2-E002 | STATUS | M-2 depends on M-1. | M:L45 |
| M2-E003 | DATA | The organization policy is one signed OrgPolicyPack. | M:L46 |
| M2-E004 | FLOW | OrgPolicyPack is distributable as a file. | M:L46 |
| M2-E005 | CFG | The pack maps organization rules into an org-immune tier. | M:L46-L47 |
| M2-E006 | CFG | The pack pins the gate manifest. | M:L46-L47 |
| M2-E007 | CFG | The pack carries E-39 spend caps. | M:L47 |
| M2-E008 | CFG | The pack carries the B-37 allowlist. | M:L47 |
| M2-E009 | CFG | The pack carries G-9 configuration. | M:L47 |
| M2-E010 | CFG | The pack carries the M-4 release-channel pin. | M:L47 |
| M2-E011 | LIFE | OrgPolicyPack is applied at boot. | M:L47 |
| M2-E012 | SAFE | A seat may add checks stricter than the organization pack. | M:L47-L48 |
| M2-E013 | SAFE | A seat may never weaken organization checks. | M:L48 |
| M2-E014 | SAFE | The organization merge is monotone-tighten, lifting R8 to organization grain. | M:L48 |
| M2-E015 | OBS | Policy drift is detected. | M:L48-L49 |
| M2-E016 | UI | Policy drift is surfaced. | M:L48-L49 |
| M2-E017 | FAIL | Policy drift is not auto-corrected before GD-4. | M:L49 |
| M2-E018 | SAFE | OrgPolicyPack is data, never executable content. | M:L49-L50 |
| M2-E019 | FALLBACK | With no organization pack, behavior is byte-identical to current seat behavior. | M:L50 |
| M2-E020 | INT | M-2 composes the 3F lifecycle and mutable tier. | M:L51 |
| M2-E021 | INT | M-2 composes gate_freeze_check and the gate manifest, which the pack pins. | M:L51-L52 |
| M2-E022 | INT | M-2 composes E-39, K-5 policy-table hash, G-9, G-8 signing, and M-1 tier. | M:L51-L52 |
| M2-E023 | INT | The planned integration module is commerce/org_policy.rs. | M:L53 |
| M2-E024 | CLI | The CLI exposes 'elai org policy init'. | M:L53 |
| M2-E025 | CLI | The CLI exposes 'elai org policy sign'. | M:L53 |
| M2-E026 | CLI | The CLI exposes 'elai org policy verify'. | M:L53 |
| M2-E027 | CLI | The CLI exposes 'elai org policy show'. | M:L53 |
| M2-E028 | OBS | M-2 adds an OrgPolicyEvent family. | M:L53-L54 |
| M2-E029 | OBS | OrgPolicyEvent has Applied. | M:L53-L54 |
| M2-E030 | OBS | OrgPolicyEvent has Drift. | M:L53-L54 |
| M2-E031 | OBS | OrgPolicyEvent has RefusedForged. | M:L53-L54 |
| M2-E032 | OBS | OrgPolicyEvent has WeakenAttemptBlocked. | M:L53-L54 |
| M2-E033 | TEST | A fixture pack must boot into Applied. | M:L55 |
| M2-E034 | TEST | The applied organization cap must block a task and name the responsible pack clause. | M:L55-L56 |
| M2-E035 | TEST | A tampered pack must be refused. | M:L56 |
| M2-E036 | TEST | A local cap raise must produce WeakenAttemptBlocked. | M:L56 |
| M2-E037 | TEST | A no-pack seat must be byte-identical to baseline. | M:L56-L57 |
| M2-E038 | TEST | A stale pack must produce Drift at boot. | M:L57 |

### M-3 — fleet evidence rollup

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M3-E001 | STATUS | M-3 has lever AUDITABLE at organization grain, STRONG grounding from gap G4, BUILD-compose, effort M, wave M3. | M:L59-L60 |
| M3-E002 | STATUS | M-3 depends on M-1 and M-2. | M:L60 |
| M3-E003 | STATUS | M-3 also depends on MA-3/J-59 and MA-6/K-6. | M:L60 |
| M3-E004 | FLOW | Fleet aggregation uses a pull model. | M:L61 |
| M3-E005 | CLI | Each seat exports with 'elai export compliance --org'. | M:L61 |
| M3-E006 | INT | The seat export extends the K-6 bundle rather than forking it. | M:L61-L62, L67 |
| M3-E007 | DATA | The organization export adds seat_id. | M:L61-L62 |
| M3-E008 | DATA | The organization export adds org_id. | M:L61-L62 |
| M3-E009 | DATA | The organization export adds pack_hash. | M:L61-L62 |
| M3-E010 | DATA | The organization export adds drift. | M:L61-L62 |
| M3-E011 | DATA | The added export fields use serde defaults for compatibility. | M:L62 |
| M3-E012 | CLI | An admin aggregates with 'elai org report <dir> --verify'. | M:L62 |
| M3-E013 | DATA | One organization bundle is produced per report invocation. | M:L62-L63 |
| M3-E014 | DATA | The organization bundle contains a per-seat table. | M:L63 |
| M3-E015 | DATA | The bundle rolls up verified-outcome rate. | M:L63 |
| M3-E016 | DATA | The bundle rolls up K-4 gate tax. | M:L63-L64 |
| M3-E017 | DATA | The bundle rolls up K-1 deltas. | M:L63-L64 |
| M3-E018 | DATA | The bundle rolls up H-9 dollars per resolved task. | M:L63-L64 |
| M3-E019 | DATA | The bundle rolls up K-3 unvouched counts. | M:L64 |
| M3-E020 | DATA | The bundle rolls up M-2 policy drift. | M:L64 |
| M3-E021 | DATA | The bundle reports seat-roster honesty against entitlement seats_max. | M:L64 |
| M3-E022 | AUDIT | Every fleet number is recomputable from constituent seat journals. | M:L64-L65 |
| M3-E023 | PRIV | Fleet reporting sends zero telemetry. | M:L65-L66 |
| M3-E024 | PRIV | Fleet reporting includes zero raw content. | M:L65-L66 |
| M3-E025 | FAIL | A seat with a broken evidence chain is refused. | M:L66 |
| M3-E026 | OBS | A refused broken-chain seat is named. | M:L66 |
| M3-E027 | CLAIM | M-3 defines zero new metrics; it only aggregates existing metrics. | M:L66 |
| M3-E028 | INT | M-3 composes G-2/G-8 verification. | M:L67 |
| M3-E029 | SAFE | G-9 is both a precondition and aggregate self-test for the report. | M:L67-L68 |
| M3-E030 | INT | M-3 composes K-1, K-4, H-9, K-3, M-1, and M-2. | M:L67-L68 |
| M3-E031 | INT | The planned integration module is commerce/org_report.rs. | M:L69 |
| M3-E032 | OBS | Existing SSB events should be reused for organization reporting when possible. | M:L69 |
| M3-E033 | OBS | OrgReportEvent may be added only if no existing event fits. | M:L69 |
| M3-E034 | TEST | Acceptance uses three fixture exports: clean, drift, and broken. | M:L70 |
| M3-E035 | TEST | The three fixtures must aggregate into one bundle. | M:L70 |
| M3-E036 | TEST | The clean and drift seats must verify PASS. | M:L70-L71 |
| M3-E037 | TEST | The broken seat must be refused and named as RefusedSeat under R2. | M:L70-L71 |
| M3-E038 | TEST | Rollup numbers must recompute byte-identically. | M:L71 |
| M3-E039 | TEST | A seeded secret in any constituent must fail the build. | M:L71-L72 |
| M3-E040 | TEST | A byte-level transcript grep over the generated bundle must be empty. | M:L72 |

### M-4 — signed release channel + auto-updater + support bundle

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M4-E001 | STATUS | M-4 has lever REACH plus CANT-WRECK, STRONG grounding from gaps G5/G6 and B-43’s named updater deferral, BUILD, effort M, wave M2. | M:L74-L75 |
| M4-E002 | STATUS | M-4 depends on MA-4, M-2, and G-9. | M:L75 |
| M4-E003 | DATA | Release manifests are signed. | M:L76 |
| M4-E004 | DATA | Each release artifact carries a blake3 digest. | M:L76 |
| M4-E005 | INT | Artifact verification reuses B-43’s refuse-on-mismatch core. | M:L76-L77 |
| M4-E006 | SAFE | Revocation-list distribution provides M-1’s key-compromise lever. | M:L77 |
| M4-E007 | CFG | The updater supports a stable channel. | M:L77 |
| M4-E008 | CFG | The updater supports a beta channel. | M:L77 |
| M4-E009 | CFG | An M-2 organization-policy clause can pin the release channel. | M:L77-L78 |
| M4-E010 | FLOW | Staged rollout is deterministic. | M:L78 |
| M4-E011 | FLOW | Rollout assignment is blake3(seat_id) modulo 100. | M:L78 |
| M4-E012 | OBS | A staged-rollout assignment must be explainable from the journal. | M:L78 |
| M4-E013 | CLI | The CLI exposes 'elai update check'. | M:L79 |
| M4-E014 | CLI | The CLI exposes 'elai update apply'. | M:L79 |
| M4-E015 | CLI | The CLI exposes 'elai update rollback'. | M:L79 |
| M4-E016 | CLI | The CLI exposes 'elai update channel'. | M:L79 |
| M4-E017 | OPS | Update checking is default-on. | M:L79 |
| M4-E018 | OPS | Update checking is non-blocking. | M:L79 |
| M4-E019 | SAFE | An update apply requires user confirmation unless organization-scheduled. | M:L79-L80 |
| M4-E020 | OPS | An organization policy may schedule update application. | M:L79-L80 |
| M4-E021 | SAFE | Update application is never silent. | M:L80 |
| M4-E022 | LIFE | Rollback retains the last-good binary. | M:L80-L81 |
| M4-E023 | CLI | The CLI exposes 'elai support bundle'. | M:L81 |
| M4-E024 | PRIV | A support bundle is assembled locally. | M:L81 |
| M4-E025 | SAFE | Support-bundle diagnostics are G-9-redacted. | M:L81 |
| M4-E026 | DATA | A support bundle includes versions. | M:L81 |
| M4-E027 | DATA | A support bundle includes tier. | M:L81 |
| M4-E028 | DATA | A support bundle includes hashes. | M:L81-L82 |
| M4-E029 | DATA | A support bundle includes doctor output. | M:L82 |
| M4-E030 | DATA | A support bundle includes a redacted journal tail. | M:L82 |
| M4-E031 | UI | Support-bundle creation prints a summary. | M:L82 |
| M4-E032 | PRIV | A support bundle is never automatically sent. | M:L82 |
| M4-E033 | STATUS | Version 1 updates whole artifacts. | M:L82-L83 |
| M4-E034 | DEFER | Delta updates are deferred to a later rider. | M:L83 |
| M4-E035 | INT | M-4 is a sibling of B-43 HostInstaller and shares its manifest-verification code. | M:L84 |
| M4-E036 | INT | M-4 also composes B-45, M-2, G-9, doctor, and G-8. | M:L84-L85 |
| M4-E037 | INT | The planned CLI integration module is crates/cli/src/update.rs plus a support-bundle arm. | M:L86 |
| M4-E038 | OBS | M-4 may add a Pattern-2 ReleaseChannelEvent. | M:L86-L87 |
| M4-E039 | OBS | Existing event types should be preferred over ReleaseChannelEvent when they fit. | M:L86-L87 |
| M4-E040 | TEST | A corrupted artifact must be refused before any file lands. | M:L88 |
| M4-E041 | TEST | A clean artifact must apply successfully. | M:L88 |
| M4-E042 | TEST | Rollback after a clean apply must restore the prior binary with a matching hash. | M:L88-L89 |
| M4-E043 | TEST | An organization stable-channel pin must hide beta manifests. | M:L89 |
| M4-E044 | TEST | A support bundle built from a seeded-secret fixture session must have an empty secret grep. | M:L89-L90 |

### M-5 — commercial packaging + verifier OSS + trust continuity

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M5-E001 | STATUS | M-5 has levers SELLABLE and AUDITABLE, STRONG grounding from gap G7, E-23-class nature plus one extraction, and effort S-M. | M:L92-L93 |
| M5-E002 | STATUS | M-5 slices a and d are wave M1; slices b and c are wave M3. | M:L93 |
| M5-E003 | CAP | M-5 publishes a standalone open-source elai-verify crate and binary. | M:L94-L95 |
| M5-E004 | INT | elai-verify wraps the same G-8 attestation verification path. | M:L94-L95 |
| M5-E005 | INT | elai-verify wraps the same G-2 RRA verification path. | M:L94-L95 |
| M5-E006 | INT | elai-verify wraps the same K-6 conformance verification path. | M:L94-L95 |
| M5-E007 | INT | elai-verify wraps the same M-3 organization-bundle verification path. | M:L95 |
| M5-E008 | SAFE | elai-verify has zero engine dependencies. | M:L95-L96 |
| M5-E009 | TEST | Extracting the verifier forms a compile-time proof of the engine boundary. | M:L95-L96 |
| M5-E010 | CAP | The ELAI engine remains closed-source. | M:L96 |
| M5-E011 | COMP | Commercial packaging includes an EULA draft. | M:L97 |
| M5-E012 | COMP | Commercial packaging includes a license-split draft. | M:L97 |
| M5-E013 | COMP | Commercial packaging includes a DPA draft. | M:L97 |
| M5-E014 | COMP | Each legal draft carries a counsel-review banner. | M:L97 |
| M5-E015 | PRIV | The mint service sees org_id. | M:L97-L98 |
| M5-E016 | PRIV | The mint service sees payment metadata. | M:L97-L98 |
| M5-E017 | PRIV | The mint service sees only org_id and payment metadata. | M:L97-L98 |
| M5-E018 | COMP | The trust-continuity statement offers escrow on lapse. | M:L98 |
| M5-E019 | COMP | The trust-continuity statement alternatively offers conversion to OSS on abandonment. | M:L98-L99 |
| M5-E020 | STATUS | Both continuity alternatives are drafted and the user selects between them. | M:L98-L99 |
| M5-E021 | COMP | Commercial documentation includes a key-custody note. | M:L99 |
| M5-E022 | CLAIM | Commercial documentation states an honest solo-support posture. | M:L99-L100 |
| M5-E023 | CLAIM | Commercial documentation must not advertise a fake SLA. | M:L99-L100 |
| M5-E024 | CFG | tiers.toml defines Free as data. | M:L100 |
| M5-E025 | CFG | tiers.toml defines Pro as data. | M:L100 |
| M5-E026 | CFG | tiers.toml defines Team as data. | M:L100 |
| M5-E027 | CFG | tiers.toml is the one pricing source of truth. | M:L100-L101 |
| M5-E028 | INT | M-1 reads tiers.toml. | M:L100-L101 |
| M5-E029 | INT | K-24 reads tiers.toml. | M:L100-L101 |
| M5-E030 | CLAIM | The $39-49-per-seat go-to-market framing is tagged [EXTERNAL-GTM]. | M:L101 |
| M5-E031 | CLAIM | That price framing remains externally qualified until K-24 supplies a measured number. | M:L101 |
| M5-E032 | DEFER | The Article-50-certified hook remains killed under LOCK-3. | M:L102 |
| M5-E033 | INT | M-5 composes G-8/G-2/K-6/M-3 verification, K-1 dossier and claims lint, K-7 brief, and M-1. | M:L103-L104 |
| M5-E034 | CLAIM | K-1 claims lint gains a pricing recognizer. | M:L103-L104 |
| M5-E035 | INT | The published integration artifact is the elai-verify crate. | M:L105 |
| M5-E036 | INT | Legal and commercial content lives under docs/strategy/commercial/. | M:L105 |
| M5-E037 | INT | Integration includes tiers.toml. | M:L105 |
| M5-E038 | INT | Integration includes a claims-lint extension. | M:L105-L106 |
| M5-E039 | OBS | M-5 adds no SSB event. | M:L106 |
| M5-E040 | TEST | On a clean, engine-free machine, elai-verify must verify a fixture attestation. | M:L107-L108 |
| M5-E041 | TEST | On a clean, engine-free machine, elai-verify must verify a fixture RRA. | M:L107-L108 |
| M5-E042 | TEST | On a clean, engine-free machine, elai-verify must verify a fixture organization bundle. | M:L107-L108 |
| M5-E043 | TEST | elai-verify must catch a one-byte artifact tamper under R2. | M:L108 |
| M5-E044 | TEST | The crate must compile with zero engine-crate dependencies. | M:L108-L109 |
| M5-E045 | TEST | A seeded pricing superlative must fail claims lint under MD-5. | M:L109 |
| M5-E046 | TEST | 'elai license status' must render from tiers.toml. | M:L109-L110 |

### M-6 — evidence format as a versioned open standard

| Atom | Kind | Explicit feature or behavior | Anchor |
|---|---|---|---|
| M6-E001 | STATUS | M-6 has levers REACH and AUDITABLE, STRONG grounding from spec-eval ADD-1, BUILD-compose plus docs, effort M, wave M3. | M:L112-L116 |
| M6-E002 | STATUS | SLSA/in-toto precedent is training-memory only and must be re-verified at execution. | M:L113-L115 |
| M6-E003 | STATUS | M-6 depends on M-5a extraction and stable G-2/G-8/K-6 emit paths. | M:L115-L116 |
| M6-E004 | CAP | M-6 publishes a versioned machine-readable evidence format provisionally called EEF v1. | M:L117 |
| M6-E005 | STATUS | The final EEF name is selected at execution. | M:L117 |
| M6-E006 | DATA | The format specifies the RRA bundle shape. | M:L117-L118 |
| M6-E007 | DATA | The format specifies the attestation predicate shape. | M:L117-L118 |
| M6-E008 | DATA | The format specifies the compliance-bundle shape. | M:L118 |
| M6-E009 | DATA | The format specifies verdict-row shapes. | M:L118 |
| M6-E010 | DATA | The format specifies receipt-row shapes. | M:L118 |
| M6-E011 | INT | Third-party tools can emit evidence that ELAI can verify. | M:L118-L119 |
| M6-E012 | CAP | elai-verify becomes the reference verifier for the format. | M:L119 |
| M6-E013 | DATA | Schemas are derived from engine serialization types. | M:L119-L120 |
| M6-E014 | SAFE | Engine serialization types are the single source of schema truth. | M:L120 |
| M6-E015 | SAFE | Schema definitions must never be maintained as a hand-written fork. | M:L120 |
| M6-E016 | COMP | Publication includes a prose format specification. | M:L121 |
| M6-E017 | TEST | Publication includes conformance fixtures. | M:L121 |
| M6-E018 | CLI | elai-verify adds 'conform <artifact>'. | M:L121 |
| M6-E019 | SAFE | Format conformance is fail-closed. | M:L121-L122 |
| M6-E020 | FAIL | Missing required evidence is normatively a conformance FAIL. | M:L122 |
| M6-E021 | LIFE | Schema evolution is append-only. | M:L123 |
| M6-E022 | SAFE | Schema evolution is protected by a golden-file lock. | M:L123-L124 |
| M6-E023 | SAFE | The schema golden-file lock joins the frozen gate manifest under R8/E-45. | M:L123-L124 |
| M6-E024 | OPS | The evidence format is published beside elai-verify. | M:L124 |
| M6-E025 | OPS | Publication includes a worked third-party-emitter example. | M:L124 |
| M6-E026 | CLAIM | The phrase 'open standard' is KD-1-gated on at least one external emitter. | M:L124-L125 |
| M6-E027 | CLAIM | Before one external emitter exists, the permitted phrase is 'published format'. | M:L125 |
| M6-E028 | CLAIM | K-1 claims lint gains an adoption-claim recognizer. | M:L125-L126 |
| M6-E029 | STATUS | M-6 has no direct PROOF_SPINE contribution. | M:L126 |
| M6-E030 | STATUS | M-6 is recorded as a distribution amplifier with a user-acknowledged spine exemption. | M:L126 |
| M6-E031 | INT | M-6 extends M-5a’s verifier binary. | M:L127 |
| M6-E032 | INT | M-6 describes, but never forks, the G-2/G-8/K-6 emit paths. | M:L127-L128 |
| M6-E033 | INT | M-6 composes E-45’s golden lock, K-1 lint, K-3/K-11 rendered shapes, and gate-manifest R8. | M:L127-L128 |
| M6-E034 | CAP | The engine stays closed while the evidence format and verifier are published. | M:L128 |
| M6-E035 | TEST | A conformant fixture must pass 'elai-verify conform' on an engine-free machine. | M:L129-L130 |
| M6-E036 | TEST | A one-byte-tampered fixture must fail 'elai-verify conform'. | M:L129-L130 |
| M6-E037 | TEST | A missing-field fixture must fail 'elai-verify conform'. | M:L129-L130 |
| M6-E038 | TEST | Derived schemas must byte-match regeneration from engine types. | M:L130-L131 |
| M6-E039 | TEST | Removing a schema field must trip the R8 golden lock. | M:L131 |
| M6-E040 | TEST | A seeded 'industry standard' phrase must fail K-1 lint while no adoption row exists. | M:L131-L132 |
| M6-E041 | TEST | The worked third-party example must emit a row accepted by conform. | M:L132-L133 |
| M6-E042 | OBS | M-6 adds no SSB event because it consists of the standalone verifier and documentation. | M:L133 |

## End-to-end workflows and interactions

### Purchase, activation, expiry, and continued safe use

1. An external merchant of record handles payment and calls ELAI’s sole first-party mint endpoint. The endpoint only mints, renews, or revokes an entitlement. [M1-E018..M1-E024]
2. A signed entitlement is delivered to a seat and verified offline; installation may offer activation at its end. [M1-E002..M1-E010, M1-E034]
3. tier() reads the data-defined tier and gates K-24 credit grants plus M-2/M-3 Team functionality. [M1-E025..M1-E029]
4. All expiry math uses ClockProvider. After a 14-day grace, the lifecycle emits ExpiredGrace and DegradedToFree. [M1-E014..M1-E017, M1-E033, M1-E039..M1-E040]
5. A degraded seat can still run Free work and every safety gate remains active. Forgery is refused, while excess activation is honestly recorded for M-3 rather than bricking the seat. [M1-E030..M1-E032, M1-E049..M1-E052]

### Policy creation, boot application, tightening, and drift

1. An administrator initializes, signs, verifies, and shows one file-distributable OrgPolicyPack. [M2-E003..M2-E004, M2-E024..M2-E027]
2. At boot, the pack supplies rules, manifest pins, caps, allowlists, redaction configuration, and release-channel pinning. [M2-E005..M2-E011]
3. A seat may tighten these controls but cannot weaken them; a raise/weaken attempt becomes WeakenAttemptBlocked. [M2-E012..M2-E014, M2-E032, M2-E036]
4. Tampering is refused. Staleness creates surfaced Drift but is not auto-corrected. No pack preserves byte-identical baseline behavior. [M2-E015..M2-E019, M2-E030..M2-E031, M2-E035, M2-E037..M2-E038]

### Private fleet evidence aggregation

1. Each seat produces an extended K-6 compliance export with seat and policy identifiers but no raw content. [M3-E005..M3-E011, M3-E023..M3-E024]
2. An administrator pulls exports into a directory and runs the verifying organization-report command. [M3-E004, M3-E012]
3. The command verifies evidence chains, refuses and names broken seats, and builds one per-seat table plus existing-metric rollups. [M3-E013..M3-E027]
4. Every number can be rederived from constituent journals, with secrets and transcript bytes excluded. [M3-E022, M3-E029, M3-E038..M3-E040]

### Safe update and local support

1. A seat non-blockingly checks signed stable/beta manifests by default. An organization pin may filter the visible channel, while blake3(seat_id)%100 deterministically assigns staged rollout. [M4-E003..M4-E018]
2. Application requires explicit user confirmation or an organization schedule and is never silent. Artifact mismatch is refused before disk write. [M4-E019..M4-E021, M4-E040]
3. The previous known-good binary is retained so rollback can restore its exact hash. [M4-E022, M4-E041..M4-E042]
4. A user can assemble a local, redacted support bundle, inspect its printed summary, and decide whether to transmit it; ELAI never auto-sends it. [M4-E023..M4-E032]

### Independent verification and format adoption

1. The same engine verification paths are extracted into an open-source, engine-independent elai-verify boundary while the engine remains closed. [M5-E003..M5-E010]
2. On a clean machine, the binary verifies attestations, RRAs, conformance artifacts, and fleet bundles and rejects one-byte tampering. [M5-E040..M5-E044]
3. M-6 derives evidence schemas from engine serialization types and publishes them with prose, fixtures, the conform command, and a third-party example. [M6-E004..M6-E025]
4. External emitters can target the format. 'Open standard' remains a prohibited adoption claim until at least one external emitter exists. [M6-E011..M6-E012, M6-E026..M6-E028]
5. Schema changes are append-only and golden-locked; field removal, missing evidence, or tampering fails closed. [M6-E019..M6-E023, M6-E035..M6-E040]

## Cross-feature invariants

1. Commerce availability and safety have deliberately different failure semantics: commerce degrades to Free; safety gates never degrade. [MX-E020..MX-E024, MX-E035..MX-E036]
2. Runtime work never depends on a licensing service. Once obtained, entitlement proof is local and offline-verifiable. [MX-E023..MX-E025, M1-E002..M1-E003]
3. The only first-party commercial cloud surface is the mint endpoint, limited to mint/renew/revoke. Policy and evidence move through files or git. [MX-E025..MX-E026, M1-E020..M1-E024]
4. Organization controls are monotonic: a seat can tighten but cannot weaken a signed organization pack. [M2-E012..M2-E014]
5. Organization pack content is inert data, never executable code. [M2-E018]
6. Absence of an organization pack must preserve byte-identical single-seat behavior. [M2-E019, M2-E037]
7. Fleet reporting is pull-based, locally recomputable, and contains neither telemetry nor raw content/transcripts. [M3-E004, M3-E022..M3-E024, M3-E040]
8. Broken or forged trust chains are refused and named where applicable. [M1-E042, M1-E051, M2-E031, M2-E035, M3-E025..M3-E026, M3-E037]
9. Update integrity is checked before any artifact lands, and a last-good binary remains rollbackable. [M4-E022, M4-E040..M4-E042]
10. Support material is local, redacted, summarized to the user, and never auto-sent. [M4-E024..M4-E032]
11. tiers.toml is data and the single pricing truth consumed by entitlement and billing. [M1-E029, M5-E024..M5-E029]
12. Monetary, pricing, adoption, SLA, and certification language remains evidence-gated rather than promotional by default. [MX-E031..MX-E032, M5-E022..M5-E023, M5-E030..M5-E032, M6-E026..M6-E028]
13. The external verifier reuses the same verification paths but has zero engine dependencies; publishing it does not open the engine. [M5-E003..M5-E010]
14. Evidence schemas derive from engine types, evolve append-only, and fail closed on missing evidence. [M6-E013..M6-E023]
15. SSB additions are intentionally bounded: EntitlementEvent and OrgPolicyEvent are explicit; M-3/M-4 prefer reuse; M-5/M-6 add none. [MX-E010, M1-E036..M1-E042, M2-E028..M2-E032, M3-E032..M3-E033, M4-E038..M4-E039, M5-E039, M6-E042]

## Surfaces and artifacts

| Surface or artifact | Behavior exposed | Atoms |
|---|---|---|
| License CLI | activate, status, deactivate | M1-E011..M1-E013 |
| SettingsWindow | license-status line; exact presentation decided at edit | M1-E043..M1-E044 |
| Org-policy CLI | init, sign, verify, show | M2-E024..M2-E027 |
| Fleet CLI | seat export and admin report with verification | M3-E005, M3-E012 |
| Update CLI | check, apply, rollback, channel | M4-E013..M4-E016 |
| Support CLI | local redacted support bundle with printed summary | M4-E023..M4-E032 |
| elai-verify | standalone verification of attestation, RRA, K-6 conformance, and org bundle | M5-E003..M5-E009 |
| elai-verify conform | fail-closed evidence-format conformance | M6-E018..M6-E020 |
| Entitlement | signed org/tier/seat/features/expiry record | M1-E002..M1-E009 |
| OrgPolicyPack | signed inert file with organization controls and pins | M2-E003..M2-E018 |
| Organization bundle | per-seat table plus recomputable rollups | M3-E013..M3-E027 |
| Release manifest | signed, per-artifact blake3, stable/beta metadata | M4-E003..M4-E009 |
| Support bundle | versions, tier, hashes, doctor, redacted journal tail | M4-E024..M4-E032 |
| Legal/commercial docs | EULA, license split, DPA, continuity alternatives, custody and support posture | M5-E011..M5-E023 |
| tiers.toml | Free/Pro/Team and pricing source of truth | M5-E024..M5-E029 |
| Published evidence format | derived schemas, prose spec, fixtures, example emitter | M6-E004..M6-E025 |

## Dependencies, composition, status, and deferred work

### Execution ordering

- All M execution is held behind GATE M-0. The named prerequisites are K A-1..A-4, K-3, and K-24 L1..L4. [MX-E007, MX-E040..MX-E042]
- Within M, M-1 precedes M-2; M-3 requires M-1/M-2 plus J-59/K-6 pointers; M-4 requires M-2/G-9 plus MA-4; M-6 requires M-5a and stable upstream emit paths. [M2-E002, M3-E002..M3-E003, M4-E002, M6-E003]
- Wave grouping: M1 contains entitlement plus tiers/verifier extraction; M2 contains policy and updater; M3 contains fleet rollup, legal/continuity work, and evidence-format publication. [coverage map; M5-E002]

### Reused capability dependencies

| Source IDs named by the summary | M usage |
|---|---|
| 3F | policy lifecycle and mutable tier |
| B-37 | organization allowlist |
| B-43 | installer-end activation and shared artifact mismatch verification |
| B-45 | updater composition |
| E-23 | commercial-document class/pattern |
| E-39 | spend caps |
| E-45 | schema/golden-lock evolution pattern |
| G-2 | RRA verification and evidence format |
| G-8 | signing, attestation verification, and offline verification pattern |
| G-9 | redaction, report precondition/self-test, support-bundle safety |
| H-9 | dollars-per-resolved-task rollup |
| J-3 | last-good binary rollback discipline |
| J-49 | pre-existing per-seat admission/quota composition |
| J-59 / MA-3 | per-seat memory/trust isolation |
| K-0:531 | recorded exclusion of org/SSO/RBAC/multi-tenant build |
| K-1 | dossier/claims lint, pricing recognizer, adoption-claim recognizer, K-1 deltas |
| K-3 | unvouched counts and rendered evidence shapes |
| K-4 | gate-tax metric |
| K-5 | policy-table hash |
| K-6 / MA-6 | compliance export and conformance path |
| K-7 | buyer/positioning questions and commercial brief |
| K-10 | authoring/execution order constraint |
| K-11 | rendered evidence shapes |
| K-24 | payment architecture, credit grants, pricing measurement, seats data |
| L-13 | inert-data / non-executable organization policy rule |
| ClockProvider / SeededClock | deterministic expiry lifecycle |
| gate_freeze_check / gate manifest | pinned organization policy and schema evolution |
| doctor | support diagnostics |
| R1, R2, R8 | failure semantics, tamper refusal, frozen/golden-lock properties |
| GD-2, GD-4, GD-5, KD-1, LOCK-3, TD-3, W-2, W-4, W-9 | explainability, drift timing, honesty, claims, killed scope, data-driven tiers, clean behavior, UI decision, event reuse/re-grep |

### Explicitly deferred, killed, conditional, or undecided

| Item | State | Anchor |
|---|---|---|
| SSO, SAML, SCIM, RBAC, hosted console | Demand-gated enterprise rider, not minimal M | M:L20-L21, L141-L142 |
| Enterprise rider numbering | Header says former M-6+ became M-7+; cross-cutting text still says M-6+ | M:L2-L4 vs M:L141 |
| Automatic drift correction | Deferred until at least GD-4 | M:L49 |
| Exact license-status presentation | Decide at edit under W-4 | M:L37-L38 |
| OrgReportEvent | Add only if no existing SSB event fits | M:L69 |
| ReleaseChannelEvent | Optional; prefer an existing event if it fits | M:L86-L87 |
| Delta updates | Later rider; v1 is whole-artifact | M:L82-L83 |
| Trust-continuity choice | User must choose escrow-on-lapse or convert-to-OSS-on-abandonment; both drafted | M:L98-L99 |
| Article-50-certified hook | Killed under LOCK-3 | M:L102 |
| Final EEF name | Chosen at execution | M:L117 |
| 'Open standard' claim | Deferred until at least one external emitter exists | M:L124-L126 |
| SLSA/in-toto precedent grounding | Must be re-verified at execution | M:L113-L115 |
| Direct proof-spine contribution from M-6 | None; explicitly exempted | M:L126 |

## Explicit failure, fallback, and negative behavior matrix

| Trigger | Required result | Safety / user-visible consequence | Anchor |
|---|---|---|---|
| Entitlement passes expiry and grace | ExpiredGrace then DegradedToFree | Free task remains usable; safety gates still fire | M:L28-L29, L40-L41 |
| Forged entitlement | RefusedForged / refusal | No Team unlock | M:L36-L42 |
| Excess seat activation | Record it | Surface through fleet report; do not silently absorb or brick | M:L32-L33, L41-L42 |
| Forged/tampered org pack | RefusedForged / refusal | Pack is not applied | M:L53-L56 |
| Local weakening or cap raise | WeakenAttemptBlocked | Organization floor remains effective | M:L47-L48, L53-L56 |
| Stale org pack | Drift | Surface at boot; do not auto-correct yet | M:L48-L49, L57 |
| No org pack | Baseline behavior | Byte-identical single-seat behavior | M:L50, L56-L57 |
| Broken seat evidence chain | Refuse and name seat | Exclude it from verified seats; no anonymous absorption | M:L65-L66, L70-L71 |
| Secret in fleet constituent | Fail report build | Prevent secret-bearing organization bundle | M:L71-L72 |
| Transcript bytes in fleet bundle | Test failure | Fleet output must remain content-free | M:L72 |
| Corrupt update artifact | Refuse before file lands | Preserve current binary/files | M:L88 |
| Failed/new binary rollback request | Restore last-good hash | Deterministic recovery | M:L80-L81, L88-L89 |
| Seeded secret in support session | Redact; grep stays empty | Bundle remains local and is never auto-sent | M:L81-L82, L89-L90 |
| One-byte verifier artifact tamper | Verification failure | Works without engine installation | M:L107-L109 |
| Pricing superlative without evidence | Claims-lint failure | Blocks unsupported pricing claim | M:L101, L109 |
| Missing conformance evidence | FAIL | Normative fail-closed format behavior | M:L121-L123, L129-L130 |
| Evidence artifact one-byte tamper | conform failure | Engine-free independent detection | M:L129-L130 |
| Schema field removal | Golden-lock failure | Append-only format evolution preserved | M:L123-L124, L131 |
| 'Industry standard' without adoption row | Claims-lint failure | Only 'published format' is allowed | M:L124-L126, L131-L132 |

## Observability and evidence inventory

| Event/evidence family | Explicit rows or contents | SSB status |
|---|---|---|
| EntitlementEvent | Activated, Renewed, ExpiredGrace, DegradedToFree, Revoked, RefusedForged | New Pattern-2 family |
| OrgPolicyEvent | Applied, Drift, RefusedForged, WeakenAttemptBlocked | New Pattern-2 family |
| OrgReportEvent | No variants specified | Prefer reuse; conditional new family only if nothing fits |
| ReleaseChannelEvent | No variants specified | Optional; prefer reuse |
| M-5 verifier/package | Verification outcomes exist, but summary says NO SSB | No new SSB |
| M-6 format/docs | Conformance verdicts/receipts are specified as format shapes, but summary says NO SSB for this work | No new SSB |
| Fleet bundle | Seat table; verified-outcome, gate-tax, K-1 delta, cost, unvouched, drift, roster-honesty rollups | Recomputed from constituent journals |
| Support bundle | Versions, tier, hashes, doctor, redacted journal tail | Local diagnostic; never auto-sent |

The summary’s +2..+4 SSB estimate is consistent with two mandatory families plus two conditional/reuse-first families; this consistency statement is an inference, not an additional requirement.

## Acceptance and test coverage index

| Capability | Positive arms | Negative / adversarial arms | Determinism / isolation arms |
|---|---|---|---|
| M-1 | Offline Team activation and unlock | Forged signature refusal; over-activation recorded | Seeded expiry chain; Free task remains green; all safety gates fire |
| M-2 | Pack Applied; org cap blocks and names clause | Tampered pack refused; local weakening blocked; stale pack drift | No-pack seat byte-identical |
| M-3 | Clean and drift exports pass into one bundle | Broken chain refused/named; seeded secret fails; transcript grep empty | Rollups recompute byte-identically |
| M-4 | Clean apply succeeds; support bundle generated | Corrupt artifact refused before disk; beta hidden by stable pin; secret grep empty | Rollback binary hash equals prior binary |
| M-5 | Engine-free verification of attestation/RRA/org bundle; license status renders tier data | One-byte tamper caught; pricing superlative linted | Crate compiles with zero engine dependencies |
| M-6 | Conformant fixture and third-party example pass conform | Tamper, missing field, field removal, and unsupported adoption claim fail | Derived schema byte-matches regeneration on engine-free verifier path |

## Inferred behavior — not explicit requirements

These are useful composition readings, but later compilation must not present them as specified M behavior without checking originals.

1. The +2..+4 SSB estimate likely means EntitlementEvent and OrgPolicyEvent are mandatory while OrgReportEvent and ReleaseChannelEvent account for the optional upper bound. This follows from M:L10, L36-L37, L53-L54, L69, and L86-L87.
2. 'DegradedToFree' likely removes paid/Team capabilities because tier() gates them, but the exact feature-by-feature removal list is not stated.
3. A revoked entitlement likely changes tier availability, but the summary names Revoked without stating the resulting seat state or grace behavior.
4. 'deactivate' likely removes the local activation, but its local-state semantics and whether it emits Revoked or another event are not stated.
5. The M-3 command’s '--verify' likely controls mandatory chain verification, but behavior without the flag is not described.
6. The roster-honesty rollup likely consumes M-1’s recorded over-activation signal; the exact reconciliation algorithm is not specified.
7. An organization-scheduled apply likely comes from OrgPolicyPack, but schedule fields are not enumerated among the listed pack clauses.
8. The revocation list likely covers entitlement and/or release keys, but its file format, refresh transport, and offline staleness semantics are absent.
9. 'published format' likely includes a distribution location and version negotiation, but neither registry nor URL is specified.
10. Legal-document generation may be documentation-only rather than runtime behavior; no command surface is stated.

## Ambiguities and missing decisions

1. Enterprise-rider numbering conflicts inside the summary: M:L2-L4 says M-6+ was renumbered M-7+, but M:L141 still calls it M-6+. Treat M-7+ as the newer header correction until M-0 is inspected.
2. The entitlement signature algorithm, canonical byte representation, key rotation, revocation-list structure, and local entitlement storage location are absent.
3. Grace-period start conditions, renewal timing, clock-skew policy, offline expiry messaging, and deactivation semantics are absent.
4. 'Seat-count best-effort' does not define seat identity issuance, duplicate-device rules, reconciliation cadence, or privacy-preserving roster representation.
5. The SettingsWindow license-status layout and states are deliberately left to edit-time W-4.
6. OrgPolicyPack serialization, schema version, signer/key custody, precedence ordering, file discovery path, git workflow, and stale threshold are absent.
7. 'Drift' does not specify which comparisons trigger it, how pack_hash relates to effective configuration, or the user remediation flow.
8. M-3 does not specify organization-bundle serialization, output path, command behavior without '--verify', partial-output handling after refusals, or whether a broken seat is omitted versus included as refused.
9. The summary names '2 verify PASS' for clean/drift fixtures; it does not say whether Drift affects an overall organization verdict.
10. M-4 does not define manifest schema/version, download transport, polling interval, failure backoff, channel-switch authorization, install atomicity, platform support, or retention count beyond one last-good binary.
11. ReleaseChannelEvent and OrgReportEvent are unresolved reuse decisions; their event variants are absent.
12. Support-bundle file format, destination, size/tail limit, exact redaction policy, encryption, retention, and user transmission flow are absent.
13. The selected trust-continuity model is unresolved between escrow-on-lapse and convert-to-OSS-on-abandonment.
14. Legal drafts explicitly require counsel review; they are not represented as legally approved final documents.
15. Exact tier entitlements, pricing, billing interval, taxes, currency, and K-24 credit-grant values are absent.
16. 'K-6 conformance' versus M-6 format conformance naming may represent different layers; originals must disambiguate.
17. The final name, media type, version-negotiation rules, canonical encoding, extension namespaces, unknown-field handling, and compatibility window for EEF v1 are absent.
18. 'Append-only schema evolution' does not state whether fields can be deprecated, made optional, or moved into a new major version.
19. The adoption row’s schema, authority, and verification process are absent.
20. M-6 precedent is explicitly unsearched and requires execution-time verification.

## Deeper-source manifest

### M originals discovered on disk but intentionally not read

| Plan role | Full path | Why inspect next |
|---|---|---|
| Doctrine, MA-1..MA-7 pointers, gate, enterprise rider | docs/thefinalglance/M-plan/M-0_INDEX.md | Resolve doctrine details, prerequisites, rider numbering, source map |
| M-1 original | docs/thefinalglance/M-plan/M-1_seat_entitlement_and_purchase_rail.md | Entitlement schema/lifecycle, keys, CLI and seat accounting details |
| M-2 original | docs/thefinalglance/M-plan/M-2_org_policy_pack_distribution.md | Pack schema, precedence, distribution, drift and event details |
| M-3 original | docs/thefinalglance/M-plan/M-3_fleet_evidence_rollup.md | Bundle schema, verifier behavior, metrics fields, partial failure |
| M-4 original | docs/thefinalglance/M-plan/M-4_signed_release_channel_updater_support.md | Manifest/update protocol, rollback, support-bundle format |
| M-5 original | docs/thefinalglance/M-plan/M-5_commercial_packaging_verifier_oss.md | Verifier crate boundary, legal artifacts, tier truth, continuity choice |
| M-6 original | docs/thefinalglance/M-plan/M-6_evidence_schema_open_standard.md | Schema generation, conformance contract, evolution and publication |

All 7/7 expected original M files named by the six-session summary plus M-0_INDEX were found.

### Explicit research/provenance sources found on disk but not read

| Source | Full path | Summary anchor |
|---|---|---|
| M-6 spec-eval ADD-1 | workflow/reasoning/elai-spec-eval-ambition-2026-07-05.md | M:L2-L4, L113-L115 |
| Multi-seat gap analysis | workflow/reasoning/steal-multiseat-gap-2026-07-02.md | M:L5-L7 |

### Upstream source candidates explicitly referenced by ID

Later agents should inspect the exact originals, not only the summary indexes. Candidate files confirmed by filename inventory include:

- docs/thefinalglance/B-plan/B-37_multi_runtime_backend.md
- docs/thefinalglance/B-plan/B-43_one_line_install.md
- B-45 is referenced by M-4 but no B-45 file appeared in the B-plan top-level filename inventory; this is a missing-source candidate to resolve through B-0/B summary or another directory.
- docs/thefinalglance/E-plan/E-23_developer_discourse_2026.md
- docs/thefinalglance/E-plan/E-39_cost_token_economics.md
- docs/thefinalglance/E-plan/E-45_cross_language_drift.md
- docs/thefinalglance/G-plan/G-2_reproducible_run_artifact.md
- docs/thefinalglance/G-plan/G-8_intoto_attestation_article50.md
- docs/thefinalglance/G-plan/G-9_journal_redaction_ttl.md
- docs/thefinalglance/H-plan/H-9_cost_cockpit_dashboard.md
- docs/thefinalglance/J-plan/J-3_harness_undo_file_backups.md
- docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md
- docs/thefinalglance/J-plan/J-59_per_seat_memory_trust_isolation.md
- docs/thefinalglance/K-plan/K-0_INDEX.md
- docs/thefinalglance/K-plan/K-1_problem_quantification_dossier_claims_policy.md
- docs/thefinalglance/K-plan/K-3_pr_ci_evidence_emitter.md
- docs/thefinalglance/K-plan/K-4_composed_gate_tax_budget.md
- docs/thefinalglance/K-plan/K-5_risk_tiered_gating_policy.md
- docs/thefinalglance/K-plan/K-6_compliance_export.md
- docs/thefinalglance/K-plan/K-7_appsec_insurance_positioning_brief.md
- K-10 is referenced as an order/deviation record, but no K-10 file appeared in the K-plan top-level filename inventory; this is a missing-source candidate to resolve through K-0/K summary or workflow records.
- docs/thefinalglance/K-plan/K-11_human_viewable_evidence_render.md
- docs/thefinalglance/K-plan/K-24_credit_billing_monetization.md
- docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md
- Phase 3F, ClockProvider/SeededClock, gate_freeze_check, and gate-manifest R8 should be traced through docs/specs/guide/steps/phase-index-MASTER.txt and its original phase plans.
- Symbolic controls or acceptance rules R1, R2, R8, GD-2, GD-4, GD-5, KD-1, LOCK-3, TD-3, W-2, W-4, and W-9 need their definitions from M-0 and referenced letter indexes.

### Missing or unresolved deeper-source candidates

1. MA-1 through MA-7 are mentioned but only MA-3, MA-4, and MA-6 are used by name in the summary; their exact targets and the unused MA pointers require M-0 inspection.
2. The enterprise rider’s authoritative identifier needs M-0 verification because M-6+ and M-7+ both appear.
3. B-45 and K-10 lack matching top-level plan filenames in their respective directory inventories.
4. K A-1..A-4 and K-24 L1..L4 are prerequisite substeps, not standalone paths in this summary; resolve within K originals.
5. seats.toml is named as a K-24 composition artifact at M:L34, while tiers.toml is M-5’s pricing truth; originals must clarify their distinct roles.
6. 'Pattern-2', frontend_pair, org-immune tier, and the named rule families need canonical definitions from M-0/upstream sources.
7. Implementation destinations named in the summary are planned targets, not confirmed source files: crates/orchestrator/src/commerce/entitlement.rs, commerce/org_policy.rs, commerce/org_report.rs, crates/cli/src/update.rs, docs/strategy/commercial/, tiers.toml, and the elai-verify crate.

## Quantitative inventory

- Primary summary coverage: 146/146 lines.
- Named M sessions: 6/6.
- M original-plan files found for later inspection: 7/7 expected, counting M-0 plus M-1..M-6.
- Explicit atomic records in the numbered inventory: 304 (M-wide 42; M-1 52; M-2 38; M-3 40; M-4 44; M-5 46; M-6 42).
- Explicit cross-feature invariants: 15.
- Explicit failure/fallback matrix rows: 18.
- Acceptance-capability groups: 6.
- Separated non-normative inferences: 10.
- Ambiguities/missing decisions: 20.
- Explicit research/provenance files found: 2/2.

No implementation status beyond the summary’s own planning metadata is inferred. In particular, BUILD, shipped upstream references, planned integration paths, and acceptance effects do not prove that M itself is implemented.
