# Family K aggregate — ELAI deep feature discovery

## Aggregate metadata

- Family: **K** (sequencing, composition, proof, distribution, affordability, packaging, discipline; plus net-new K-23 workspace and K-24 billing)
- Inputs merged: DEEP-K-001 through DEEP-K-004 and `11-K-SUMMARY_INDEX.discovery.md`
- Rule: every distinct atomic record is preserved with its ID and source anchor
- Namespace: ledger-local IX/SD/AC/INF IDs are written as `K-00N/<ID>` (or `SUMMARY/<ID>`) for disambiguation
- Source-plan atoms keep their ledger IDs under that namespace
- Newer-plan precedence: explicit SUPERSEDED/replacement/status-override/refutation in a source wins; else higher K-N plan number wins **target state**; losers retained under Superseded notes
- Deep ledgers supersede the summary orientation pass where they conflict (summary is orientation; deep is original-grounded)
- Status labels (PLANNED/BUILD/WIRED/SHIPPED/PARTIAL/FIXTURE-now/BENCH-PENDING/OUT-OF-K/KILLED) are plan-recorded claims unless noted
- Generated: 2026-08-12 (aggregator pass)

## Provenance

| Ledger | Source path | Declared audit (ledger footer) | Extracted atoms |
|---|---|---|---:|
| K-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-K-001.discovery.md` | 224 source (201 E + 6 I + 15 A + 2 C) + IX/SD/AC relational | 288 |
| K-002 | `docs/thefinalglance/feature-discovery/deep/DEEP-K-002.discovery.md` | 210 source (184 E + 7 I + 16 A + 3 C) + IX/SD/AC relational | 275 |
| K-003 | `docs/thefinalglance/feature-discovery/deep/DEEP-K-003.discovery.md` | 250 source (223 E + 8 I + 16 A + 3 C) + IX/SD/AC relational | 308 |
| K-004 | `docs/thefinalglance/feature-discovery/deep/DEEP-K-004.discovery.md` | 238 source (213 E + 8 I + 15 A + 2 C) + IX/SD/AC relational | 294 |
| SUMMARY | `docs/thefinalglance/feature-discovery/11-K-SUMMARY_INDEX.discovery.md` | 622 explicit + 1 entry-local A + 8 A + 2 I ledger (633 traceable) + composition/cross-cutting orientation | 699 |
| **Extracted total** | — | — | **1864** |

## Notation

- Namespaced ID form: `K-00N/<local-id>` or `SUMMARY/<local-id>`.
- Cross-file interactions, summary deltas, and ambiguity/conflict registers keep local IDs under their ledger namespace (e.g. `K-001/IX-003`, `K-004/SD-K4-001`, `K-001/AC-001`).
- Kind tags: `[SRC]` source explicit/I/A/C atom, `[IX]` cross-source interaction, `[SD]` summary delta, `[AC]` ambiguity/conflict register, `[INF]` inference class inside source tables, `[SUM]` summary-layer atom.
- `SURFACE` marks user-facing windows, panels, commands, CLI verbs, flags, status displays, notifications (feeds dashboard DESIGN.md).
- Anchor form: `(DEEP-K-00N.discovery.md:L<line>)` points at the deep-ledger definition line; plan anchors like `K-4:L7-L12` are preserved from the source atom.

## Family map (session inventory)

| Track / wave | Sessions (K-N) | Deep ledger owners (primary) |
|---|---|---|
| Index / doctrine | K-0, K-10 doctrine | K-001 (K-0, FIX) |
| Grounding / master | KGM, KMB | K-002 |
| K-T1 PROVE | K-1, K-2 | K-001 (K-1), K-003 (K-2) |
| K-T2 WEDGE+DISTRIBUTION | K-3, K-11 | K-002 (K-3), K-004 (K-11) |
| K-T3 ADAPTIVE GATE ECONOMY | K-4, K-5 | K-004 |
| K-T4 BUYER-SIDE EVIDENCE | K-6, K-7 | K-002 (K-6), K-004 (K-7) |
| K-T5 DEPRECIATION+REFRESH | K-8, K-9, K-25 | K-001 (K-8,K-9), K-004 (K-25) |
| KAT / human evidence | K-11, K-12 | K-004 (K-11), K-003 (K-12) |
| OGT | K-13, K-14 | K-003 |
| GRC | K-15, K-16 | K-003 (K-15), K-001 (K-16) |
| CMG | K-17 | K-002 |
| VCG | K-18, K-18a, K-19 | K-002 (K-18), K-004 (K-18a,K-19) |
| LFG | K-20, K-21, K-22 | K-001 (K-20), K-003 (K-21), K-002 (K-22) |
| MPW net-new | K-23 | K-004 |
| BCC net-new | K-24 | K-003 |
| CAW | K-25 | K-004 |
| Ops campaigns / unindexed | K-26, K-27 | K-003 |
| Summary orientation | all | SUMMARY |

## User-facing surfaces

Every atom that names a window, panel, drawer, command, CLI verb, flag, status display, or notification. Full text is retained in ledger sections below; this subsection indexes them by namespaced ID for dashboard DESIGN.md feed.

Surface-tagged atom count: **591**.

### Surfaces — K-001/Cross-source interactions and lifecycle (3)

- **K-001/IX-001** [IX] (DEEP-K-001.discovery.md:L319 | SURFACE): IX | E | K-0 places K-1 and K-8 in WAVE KW-1: invoke A-1 first for kill-switch, run K-8 in parallel to stop wasted builds. | K0-E053, K0-E066; K1-E001; K8-E001 | anchor=
- **K-001/IX-002** [IX] (DEEP-K-001.discovery.md:L320 | SURFACE): IX | E | K-1 is the central claims registry; K-16 and K-20 both EXTEND `claims_policy_lint.py` rather than fork it (GD-1). | K1-E013–K1-E014; K16-E012; K20-E007 | anchor=
- **K-001/IX-014** [IX] (DEEP-K-001.discovery.md:L332 | SURFACE): IX | E | A-1 CLI `elai bench claim-scope` is both K-0’s first action and K-1’s measured-number source via INVOKE. | K0-E024, K0-E066; K1-E004, K1-E008 | anchor=

### Surfaces — K-001/Explicit ambiguity and conflict register (2)

- **K-001/AC-003** [AC] (DEEP-K-001.discovery.md:L346 | SURFACE): AC | ambiguity/surface | K-9 refresh-decision tile vs CLI-only is edit-time W-4/W-6 choice. | K9-A001; K0-A003 | anchor=
- **K-001/AC-010** [AC] (DEEP-K-001.discovery.md:L353 | SURFACE): AC | ambiguity/surface | K-20 PR re-execute line wording is illustrative, not frozen UI copy. | K20-A002 | anchor=

### Surfaces — K-001/FIX (3)

- **K-001/FIX-E001** [SRC] (DEEP-K-001.discovery.md:L34 | SURFACE): E/TEST | The file is a negative fixture for K-1 Task 5, seeded so `claims_policy_lint.py` must FAIL. | anchor=FIX-claims:L1-L1
- **K-001/FIX-E003** [SRC] (DEEP-K-001.discovery.md:L36 | SURFACE): E/FAIL | Violation class 1 is a banned superlative claim: “ELAI is the safest coding agent on the market today.” | anchor=FIX-claims:L4-L6
- **K-001/FIX-E006** [SRC] (DEEP-K-001.discovery.md:L39 | SURFACE): E/SURFACE | Required invocation surface: `claims_policy_lint.py --files <this file>` must exit nonzero. | anchor=FIX-claims:L13-L14

### Surfaces — K-001/FIX-claims deltas (1)

- **K-001/SD-FIX-001** [SD] (DEEP-K-001.discovery.md:L365 | SURFACE): SD | **ADD** the concrete fixture path, exact two violation strings, tag-list including `[MEASURED]`, and required nonzero exit of `claims_policy_lint.py --files`. Summary only says “seeded unmeasured superlative / bare external number”. | 11-K:L75-L76 | FIX-E001–FIX-E006 | anchor=11-K:L75-L76

### Surfaces — K-001/K-0 deltas (6)

- **K-001/SD-K0-001** [SD] (DEEP-K-001.discovery.md:L371 | SURFACE): SD | **ADD** full KD-1..KD-5 normative text, DONE definitions for BUILD vs doctrine sessions, and hard-rule block (blake3, unwrap ban, anti-slopsquat, allowlist). Summary compresses into K-X.* bullets. | 11-K:L28-L50, L830-L848 | K0-E013–K0-E021, K0-E061 | anchor=11-K:L28-L50, L830-L848
- **K-001/SD-K0-003** [SD] (DEEP-K-001.discovery.md:L373 | SURFACE): SD | **ADD** full KW-1..KW-3 sequencing, literal first claim-scope command, and five kill-switches with consequences. | 11-K:L33-L41, L850+ | K0-E053, K0-E056–K0-E060, K0-E066 | anchor=11-K:L33-L41, L850+
- **K-001/SD-K0-004** [SD] (DEEP-K-001.discovery.md:L374 | SURFACE): SD | **ADD** K-10 six-point doctrine including no-new-letters vs new-files-within-K distinction and canvas demotion. | 11-K:L22-L22, L35-L35 | K0-E002, K0-E054 | anchor=11-K:L22-L22, L35-L35
- **K-001/SD-K0-005** [SD] (DEEP-K-001.discovery.md:L375 | SURFACE): SD | **ADD** complete non-goals list and web landmines (43% vs 58% distinction, SpecBench PLAN-asserted, KW-6 ban). | 11-K:L41-L42, L830-L848 | K0-E055, K0-E062–K0-E064 | anchor=11-K:L41-L42, L830-L848
- **K-001/SD-K0-007** [SD] (DEEP-K-001.discovery.md:L377 | SURFACE): SD | **ADD** index-level CLI/UI inventory for sessions outside this chunk’s detail files (K-2..K-7, K-11..K-15, K-17..K-19, K-21..K-24) as index facts. | 11-K:L52+ | K0-E026–K0-E050 | anchor=11-K:L52+
- **K-001/SD-K0-008** [SD] (DEEP-K-001.discovery.md:L378 | SURFACE): SD | **FLAG** co-presence of “almost no new capability” with K-23/K-24 net-new admission. | 11-K:L31-L32 | K0-C001 | anchor=11-K:L31-L32

### Surfaces — K-001/K-1 deltas (2)

- **K-001/SD-K1-001** [SD] (DEEP-K-001.discovery.md:L384 | SURFACE): SD | **ADD** exact produce paths, lint CLI trigger over `docs/strategy/positioning/*.md`, auditor-only application, and conditional-routing edit trigger. | 11-K:L54-L76 | K1-E013–K1-E018, K1-E025 | anchor=11-K:L54-L76
- **K-001/SD-K1-002** [SD] (DEEP-K-001.discovery.md:L385 | SURFACE): SD | **ADD** full KW citation set with arXiv/OpenReview identifiers and SpecBench re-quote-at-build rule; gap 26.4%→57.8% detail. | 11-K:L58-L63 | K1-E006, K1-E026 | anchor=11-K:L58-L63

### Surfaces — K-001/K-20 deltas (3)

- **K-001/SD-K20-003** [SD] (DEEP-K-001.discovery.md:L424 | SURFACE): SD | **ADD** PR surface line “verify offline / re-execute…”, A-4/`elai wrap` packaging path, and KD-5 escalation semantics. | 11-K:L529-L543 | K20-E010–K20-E011, K20-E020 | anchor=11-K:L529-L543
- **K-001/SD-K20-004** [SD] (DEEP-K-001.discovery.md:L425 | SURFACE): SD | **ADD** explicit bans on second RRA bench and rra.rs fork; G-2 CLI triad cited. | 11-K:L540-L540 | K20-E003–K20-E004, K20-E023 | anchor=11-K:L540-L540
- **K-001/SD-K20-005** [SD] (DEEP-K-001.discovery.md:L426 | SURFACE): SD | **FLAG** market-gap wording dual of REPLAYABLE demand vs RE-EXECUTABLE stronger property. | 11-K:L525-L536 | K20-C001 | anchor=11-K:L525-L536

### Surfaces — K-001/K-8 deltas (1)

- **K-001/SD-K8-002** [SD] (DEEP-K-001.discovery.md:L396 | SURFACE): SD | **ADD** exact produce paths (`provider-absorption-pregate.md`, `K-8-absorbable-unit-audit.md`), DETERMINISTIC_CHECK rg commands, and no-Rust-contract test plan. | 11-K:L236-L244 | K8-E008–K8-E009, K8-E014, K8-E017–K8-E018 | anchor=11-K:L236-L244

### Surfaces — K-001/K-9 deltas (3)

- **K-001/SD-K9-001** [SD] (DEEP-K-001.discovery.md:L404 | SURFACE): SD | **ADD** exact CLI `elai bench model-refresh [--lane]`, ModelRefreshCadence kind list, IncumbentKept safe arm, and routing/allowlist MUST-be-0 isolation check. | 11-K:L247-L271 | K9-E010–K9-E012, K9-E016, K9-E019 | anchor=11-K:L247-L271
- **K-001/SD-K9-002** [SD] (DEEP-K-001.discovery.md:L405 | SURFACE): SD | **ADD** W-10 split (schema leg now / live RunPod later), dated refresh-report artifact, and optional refresh-decision tile. | 11-K:L265-L268 | K9-E009, K9-E013–K9-E014, K9-A001 | anchor=11-K:L265-L268
- **K-001/SD-K9-004** [SD] (DEEP-K-001.discovery.md:L407 | SURFACE): SD | **ADD** cost discipline for fixture set size and files-touched list including args_bench + optional SSB bump. | 11-K:L265-L266 | K9-E020, K9-E023 | anchor=11-K:L265-L266

### Surfaces — K-001/K0 (35)

- **K-001/K0-E007** [SRC] (DEEP-K-001.discovery.md:L56 | SURFACE): E/CAP | K-T2 WEDGE + DISTRIBUTION (A-4, K-3, A-5): A-4 is portable `.rra`/`elai wrap`; K-3 renders evidence into GitHub check-runs + PR comments; A-5 is review-time study. Delta: KD-2. | anchor=K-0:L76-L82
- **K-001/K0-E009** [SRC] (DEEP-K-001.discovery.md:L58 | SURFACE): E/CAP | K-T4 BUYER-SIDE EVIDENCE (K-6, K-7): K-6 builds `elai export compliance`; K-7 is AppSec/insurance positioning brief (pure prose). No org/SSO build. | anchor=K-0:L92-L96
- **K-001/K0-E011** [SRC] (DEEP-K-001.discovery.md:L60 | SURFACE): E/CAP | K-T6 SEQUENCING / CUT (K-10 doctrine in index): priority order, no-new-letters, activation-over-authoring, canvas demotion, kill-switches. Delta: KD-5. | anchor=K-0:L106-L110
- **K-001/K0-E014** [SRC] (DEEP-K-001.discovery.md:L63 | SURFACE): E/CFG | KD-1 NUMBER-BEFORE-CLAIM: no externally-visible claim ships without a measured journal-recomputable ELAI number OR explicit `[EXTERNAL]`/`[DESIGN-RATIONALE]` tag. Enforced by K-1 lint; binds K-1 and K-7. | anchor=K-0:L140-L144
- **K-001/K0-E015** [SRC] (DEEP-K-001.discovery.md:L64 | SURFACE): E/CFG | KD-2 MEET-THE-BUYER-IN-THE-PR: every trust signal ships a PR/CI rendering or justified W-6 absence; dashboard is deep view never only view. Binds K-3. | anchor=K-0:L145-L148
- **K-001/K0-E017** [SRC] (DEEP-K-001.discovery.md:L66 | SURFACE): E/CFG | KD-4 PROVIDER-ABSORPTION PRE_GATE: efficiency-class units re-verify provider CURRENT native surface (WebSearch) before EXECUTING; duplicating a now-native feature is kill-or-de-scope. Binds K-8 and future efficiency plans. | anchor=K-0:L154-L157
- **K-001/K0-E018** [SRC] (DEEP-K-001.discovery.md:L67 | SURFACE): E/CFG | KD-5 WINDOW-CLOCK: wedge artifacts (A-4 `.rra`, K-3 PR emitter, K-6 compliance export) are time-boxed first-mover deliverables; a slipped quarter escalates to the user. Binds A-4, K-3, K-6. | anchor=K-0:L158-L161
- **K-001/K0-E019** [SRC] (DEEP-K-001.discovery.md:L68 | SURFACE): E/TEST | DONE for BUILD sessions requires cargo check GREEN, auditor PASS on affected crates, wiring_contract_check `--run-trigger` PASS, cross_lang_diff if frontend_pair, and GD-4/KD-3 friction bound where gate defaults ON. | anchor=K-0:L163-L166
- **K-001/K0-E023** [SRC] (DEEP-K-001.discovery.md:L72 | SURFACE): E/FLOW | Section-A order: A-1 invoke → go → A-2 → A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips; A-6/A-7/A-8 parallel-safe. | anchor=K-0:L192-L193
- **K-001/K0-E024** [SRC] (DEEP-K-001.discovery.md:L73 | SURFACE): E/SURFACE | CLI surface named at index: `elai bench claim-scope [--suite][--seed][--json]` is A-1’s shipped kill-gate. | anchor=K-0:L178-L180, L584-L587
- **K-001/K0-E026** [SRC] (DEEP-K-001.discovery.md:L75 | SURFACE): E/CAP | K-2: `elai bench routing-economics`; live cost leg gated on J-28 + RunPod; net 0..+1 SSB or prefer-reuse. | anchor=K-0:L202-L206, L413-L415
- **K-001/K0-E027** [SRC] (DEEP-K-001.discovery.md:L76 | SURFACE): E/SURFACE | K-3 PR/CI evidence emitter: GitHub check-runs + PR comments; no-evidence hunk renders UNVOUCHED never silently clean. | anchor=K-0:L207-L210
- **K-001/K0-E028** [SRC] (DEEP-K-001.discovery.md:L77 | SURFACE): E/SURFACE | K-4: `elai bench gate-tax`; ceiling breach = release BLOCK; names top-3 friction contributors. | anchor=K-0:L211-L216
- **K-001/K0-E030** [SRC] (DEEP-K-001.discovery.md:L79 | SURFACE): E/SURFACE | K-6: `elai export compliance` → redacted RRA + attestation + audit extract + retention + procurement mapping; offline verify; no raw secrets. | anchor=K-0:L221-L225
- **K-001/K0-E031** [SRC] (DEEP-K-001.discovery.md:L80 | SURFACE): E/SURFACE | K-7: AppSec/insurance positioning brief; pure prose; no runtime; no SSB; user-reviewed before external use. | anchor=K-0:L226-L229
- **K-001/K0-E034** [SRC] (DEEP-K-001.discovery.md:L83 | SURFACE): E/SURFACE | K-11 extends K-3 with human-viewable `## Evidence` section in PR/comment + cockpit; risk-tiered; UNVOUCHED when no artifact. | anchor=K-0:L243-L248
- **K-001/K0-E038** [SRC] (DEEP-K-001.discovery.md:L87 | SURFACE): E/SURFACE | K-15: `elai bench reasoning-budget`; packages receipt-grounded-reasoning differentiator into K-1 dossier; prefer-reuse ConvergenceCurveMeasured. | anchor=K-0:L278-L286, L440-L442
- **K-001/K0-E041** [SRC] (DEEP-K-001.discovery.md:L90 | SURFACE): E/SURFACE | K-18: `elai bench case-reuse`; packages per-customer compounding-state differentiator; reuse arm OBEYS K-19. | anchor=K-0:L310-L317, L449-L452
- **K-001/K0-E045** [SRC] (DEEP-K-001.discovery.md:L94 | SURFACE): E/SURFACE | K-22: `doctrine_coverage.py` + coverage ledger + enforcement tags; measures rulebook enforcement; no SSB. | anchor=K-0:L353-L364, L463-L466
- **K-001/K0-E046** [SRC] (DEEP-K-001.discovery.md:L95 | SURFACE): E/CAP | K-23 NET-NEW multi-repo/workspace memory: U1..U6 slices; +WorkspaceEvent; `ELAI_WORKSPACE_ENABLE`; feature-map group 1.15; no centralized cloud store. | anchor=K-0:L366-L382, L467-L472
- **K-001/K0-E047** [SRC] (DEEP-K-001.discovery.md:L96 | SURFACE): E/SURFACE | K-23 CLI surfaces: `workspace create/add/map`, `workspace orchestrate`; canvas WORKSPACE altitude. | anchor=K-0:L370-L377
- **K-001/K0-E048** [SRC] (DEEP-K-001.discovery.md:L97 | SURFACE): E/CAP | K-24 NET-NEW credit-based seats on OpenRouter rail: L1..L6; +CreditLedgerEvent; `ELAI_BILLING_ENABLE`; balance is fold never counter; single-writer double-spend defense. | anchor=K-0:L384-L402, L473-L478
- **K-001/K0-E049** [SRC] (DEEP-K-001.discovery.md:L98 | SURFACE): E/SURFACE | K-24 CLI/UI: `billing balance`; H-9 CloudPanel billing tile needs ts-rs binding; “$20=$200” pitch gated on L6 measured SAFEDELTA. | anchor=K-0:L390-L397, L400-L402
- **K-001/K0-E053** [SRC] (DEEP-K-001.discovery.md:L102 | SURFACE): E/FLOW | KW-1: INVOKE A-1 first (kill-switch if NULL/NEGATIVE) + parallel K-8. KW-2: A-2→A-3 feed K-1; K-2 fixture; K-3 distribution; K-9 cadence. KW-3: K-4→K-5, K-6/K-7, K-13/K-14, K-15/K-16 (author K-16 before/with J GRC executors). | anchor=K-0:L490-L506
- **K-001/K0-E054** [SRC] (DEEP-K-001.discovery.md:L103 | SURFACE): E/CFG | K-10 doctrine: (1) A-1..A-4+K-3 above all else; (2) NO NEW PLAN LETTERS until those DONE (new FILES within K allowed); (3) activation-over-authoring; (4) canvas Track 2 H-10..H-16 DEMOTED behind K-3+A-5 not killed; (5) I-31 GRPO last; (6) kill-switches part of doctrine. | anchor=K-0:L509-L523
- **K-001/K0-E056** [SRC] (DEEP-K-001.discovery.md:L105 | SURFACE): E/FAIL | Kill-switch 1: A-1/A-3 NULL/NEGATIVE safety delta at bounded FP → trust wedge as HEADLINE dies; re-aim to per-customer compounding state; K-T2/K-T4 stop. | anchor=K-0:L543-L545
- **K-001/K0-E057** [SRC] (DEEP-K-001.discovery.md:L106 | SURFACE): E/FAIL | Kill-switch 2: K-4 composed tax cannot fit tolerable ceiling → aggressive adaptive-collapse; revisit K-5 weights. | anchor=K-0:L546-L547
- **K-001/K0-E058** [SRC] (DEEP-K-001.discovery.md:L107 | SURFACE): E/FAIL | Kill-switch 3: GitHub/lab ships native cryptographic replayable vendor-neutral evidence → A-4 becomes “be compatible”; pivot to compliance export + cockpit-on-top; K-3 survives. | anchor=K-0:L548-L550
- **K-001/K0-E059** [SRC] (DEEP-K-001.discovery.md:L108 | SURFACE): E/FAIL | Kill-switch 4: A-5 no review-time win → cockpit repositions as compliance artifact; K-3 remains distribution. | anchor=K-0:L551-L552
- **K-001/K0-E060** [SRC] (DEEP-K-001.discovery.md:L109 | SURFACE): E/FAIL | Kill-switch 5: 2x3090 box missing → K-9 still runs catalog+cloud comparisons; 24GB adoption deferred. | anchor=K-0:L553-L554
- **K-001/K0-E061** [SRC] (DEEP-K-001.discovery.md:L110 | SURFACE): E/LIMIT | Hard rules: blake3 only; no `.unwrap()`/`.expect()` on prod; scoped tests auditor-only; anti-slopsquatting; GD-1 compose-not-restate; main LLM never runs tests nor writes memory-bank; every external number tagged; default-OFF needs allowlist promotion plan. | anchor=K-0:L557-L564
- **K-001/K0-E063** [SRC] (DEEP-K-001.discovery.md:L112 | SURFACE): E/EVIDENCE | SpecBench “+28pp per 10x” is PLAN-asserted; quote precise table value + page anchor before external use. | anchor=K-0:L571-L572
- **K-001/K0-E066** [SRC] (DEEP-K-001.discovery.md:L115 | SURFACE): E/FLOW | Literal first command of WAVE KW-1: `elai bench claim-scope --suite default --seed 1234 --json`; if NULL/NEGATIVE STOP kill-switch 1; parallel start K-8. | anchor=K-0:L584-L587
- **K-001/K0-E067** [SRC] (DEEP-K-001.discovery.md:L116 | SURFACE): E/DATA | External problem numbers (KW set): KW-1 packages 5.2%/21.7% + recurrence; KW-2 11x vulns; KW-3 reward-hacking; KW-4 CodeAct latency/tokens; KW-5 SpecBench +28pp; all [EXTERNAL]. | anchor=K-0:L70-L71, L569-L574
- **K-001/K0-A003** [SRC] (DEEP-K-001.discovery.md:L120 | SURFACE): A/SURFACE | K-9 and K-6 frontend_pair choices are deferred to edit-time W-4/W-6. | anchor=K-0:L221-L225, L235-L240

### Surfaces — K-001/K1 (14)

- **K-001/K1-E002** [SRC] (DEEP-K-001.discovery.md:L132 | SURFACE): E/CAP | Ships two artifacts: (1) DOSSIER binding EXTERNAL KW numbers to ELAI MEASURED numbers; (2) CLAIMS POLICY doc + deterministic LINT. | anchor=K-1:L5-L10
- **K-001/K1-E004** [SRC] (DEEP-K-001.discovery.md:L134 | SURFACE): E/DEP | Pre-requisites: A-1 SHIPPED (`elai bench claim-scope`, args_bench.rs:70 / ssb.rs:9127); KW set web-verified; A-2/A-3 not required to author dossier shell. | anchor=K-1:L18-L20, L40-L45
- **K-001/K1-E006** [SRC] (DEEP-K-001.discovery.md:L136 | SURFACE): E/DATA | External numbers (each [EXTERNAL], never claimed as ELAI’s): KW-1 USENIX 5.2%/21.7%, 58%/43% recurrence; KW-2 arXiv:2507.02976 11x (135 vs 12); KW-3 reward-hacking 73.8%/46.8%; KW-4 CodeAct ~50% latency/-60% tokens; KW-5 SpecBench +28pp (exact value PLAN-asserted, quote at build). | anchor=K-1:L33-L36, L141-L146
- **K-001/K1-E008** [SRC] (DEEP-K-001.discovery.md:L138 | SURFACE): E/DEP | A-1 SHIPPED: `ClaimScopeMeasured` SSB; CLI `elai bench claim-scope`; emitter `cli::bench_claim_scope::run_inner`; fixture contract `i43_claim_scope_fixture_contract.rs`. | anchor=K-1:L40-L42
- **K-001/K1-E010** [SRC] (DEEP-K-001.discovery.md:L140 | SURFACE): E/CFG | GD-3 (no SWE-bench-Verified headline; ablation framing) and GD-5 (honest residue) are doctrine the claims policy operationalizes onto marketing surface. | anchor=K-1:L46-L47
- **K-001/K1-E011** [SRC] (DEEP-K-001.discovery.md:L141 | SURFACE): E/DATA | Author dossier (~120-180 lines): one row per problem → (external number [EXTERNAL]+KW source) paired with (ELAI measured + recompute pointer OR “PENDING — fills from A-N”). Every external number re-verified at authoring; re-quote SpecBench exact value + page anchor. | anchor=K-1:L60-L64
- **K-001/K1-E013** [SRC] (DEEP-K-001.discovery.md:L143 | SURFACE): E/CAP | Build claims-policy LINT (`.claude/tools/claims_policy_lint.py` stdlib-only OR elai-side check): scan README/positioning for number-or-superlative patterns; each must be adjacent to measured-pointer or `[EXTERNAL]`/`[DESIGN]` tag; exit nonzero on violation (fail-closed). ~80-120 LOC. | anchor=K-1:L68-L70
- **K-001/K1-E016** [SRC] (DEEP-K-001.discovery.md:L146 | SURFACE): E/DATA | Produces: `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, `docs/strategy/ELAI-CLAIMS-POLICY.md`, `.claude/tools/claims_policy_lint.py`. | anchor=K-1:L78-L81, L133-L136
- **K-001/K1-E017** [SRC] (DEEP-K-001.discovery.md:L147 | SURFACE): E/DEP | Consumes: `cli::bench_claim_scope` (INVOKE not rebuild); safedelta_runner + bench_seccodeplt; I-43 honest-wording table. | anchor=K-1:L82-L85
- **K-001/K1-E018** [SRC] (DEEP-K-001.discovery.md:L148 | SURFACE): E/SURFACE | Trigger command surface: `python .claude/tools/claims_policy_lint.py --files docs/strategy/positioning/*.md`. | anchor=K-1:L97-L99
- **K-001/K1-E020** [SRC] (DEEP-K-001.discovery.md:L150 | SURFACE): E/SURFACE | Frontend pair N/A — positioning docs + CI/audit lint, no dashboard surface (W-6 justified). | anchor=K-1:L95-L96
- **K-001/K1-E023** [SRC] (DEEP-K-001.discovery.md:L153 | SURFACE): E/TEST | W-1 done = lint runs in audit path + negative fixture FAILS, not “doc exists”. GD-2 recompute pointer on measured rows; GD-3 no SWE-bench headline in dossier. | anchor=K-1:L120-L125
- **K-001/K1-E026** [SRC] (DEEP-K-001.discovery.md:L156 | SURFACE): E/EVIDENCE | Citation anchors: KW-1 Spracklen USENIX; KW-2 arXiv:2507.02976; KW-3 OpenReview ikrQWGgxYg (gap 26.4%→57.8%); KW-4 Microsoft CodeAct BUILD 2026; KW-5 SpecBench arXiv:2605.21384. | anchor=K-1:L140-L146
- **K-001/K1-E030** [SRC] (DEEP-K-001.discovery.md:L160 | SURFACE): E/SURFACE | User-facing surfaces: dossier strategy document; claims-policy document; lint CLI; auditor positioning-copy gate; conditional-routing on README/positioning edits. | anchor=K-1:L60-L72, L95-L101

### Surfaces — K-001/K16 (5)

- **K-001/K16-E006** [SRC] (DEEP-K-001.discovery.md:L251 | SURFACE): E/CFG | RR-3 MEASURE-YOUR-OWN-BUDGET: reasoning-budget/overthinking/difficulty thresholds ELAI-MEASURED per-(model,task-type) on ELAI gold set (K-15); NEVER paper constant (ID-5); WINDOW never lever for depth-class failure (RD-4). | anchor=K-16:L18-L18
- **K-001/K16-E016** [SRC] (DEEP-K-001.discovery.md:L261 | SURFACE): E/SURFACE | Frontend pair N/A — doctrine + claims recogniser, no dashboard (W-6). | anchor=K-16:L64-L65
- **K-001/K16-E022** [SRC] (DEEP-K-001.discovery.md:L267 | SURFACE): E/DATA | Files: CREATE reasoning-receipts.md, reasoning_selfverify_negative.md, K-16-decisions; MODIFY conditional-routing; MODIFY claims_policy_lint.py. | anchor=K-16:L99-L104
- **K-001/K16-E023** [SRC] (DEEP-K-001.discovery.md:L268 | SURFACE): E/EVIDENCE | Paper grounding [EXTERNAL]: Reflexion/Tsui self-correction blind spot; ToT self-eval weak link; self-consistency + “Consensus is Not Verification”; Lanham faithfulness; overthinking 2604.10739 model-specific knee. | anchor=K-16:L106-L109
- **K-001/K16-E026** [SRC] (DEEP-K-001.discovery.md:L271 | SURFACE): E/SURFACE | User-facing/operator surfaces: doctrine rule load on reasoning-engine tasks; claims lint over positioning; no dedicated dashboard panel. | anchor=K-16:L64-L69

### Surfaces — K-001/K20 (12)

- **K-001/K20-E003** [SRC] (DEEP-K-001.discovery.md:L286 | SURFACE): E/LIMIT | Must NOT author J-14-extension cassette/pinned-env; must NOT build second RRA bench; must NOT fork rra.rs; measured re-execution-fidelity is [BENCH-PENDING]; must surface WORLD-DRIFT as fidelity downgrade (GD-5); keep honest-tier vocabulary (I-43). | anchor=K-20:L5-L5, L52-L56
- **K-001/K20-E004** [SRC] (DEEP-K-001.discovery.md:L287 | SURFACE): E/DEP | Substrate cited: G-2 RRA WIRED (`elai run --emit-rra` / `rra show` / `replay --verify`; msgpack+zstd; action-stream on cloud); J-14 PLANNED verify tiers T0/T1/T2 + `rra export-fixture`; G-1 taint at 4 ingest sites; I-45 objective seal; I-18 chain; G-9 redaction/auth scrub. | anchor=K-20:L7-L8, L15-L22
- **K-001/K20-E008** [SRC] (DEEP-K-001.discovery.md:L291 | SURFACE): E/TEST | NEGATIVE fixture `reexecutable_rra_negative.md`: (a) untagged “ELAI runs are fully reproducible”; (b) “re-execute exact live world bit-for-bit”; (c) cloud token-faithful where J-13 caps did not pass — all FAIL. | anchor=K-20:L34-L34, L68-L68
- **K-001/K20-E010** [SRC] (DEEP-K-001.discovery.md:L293 | SURFACE): E/INT | Production: auditor lint over differentiator copy; dossier row points to J-14-extension number when it lands; K-3/K-11 PR emitter links re-executable bundle; A-4 `elai wrap` ships it (KD-2 distribution). | anchor=K-20:L47-L50
- **K-001/K20-E011** [SRC] (DEEP-K-001.discovery.md:L294 | SURFACE): E/SURFACE | Frontend: differentiator surfaces on K-3/K-11 PR-evidence render (“verify offline / re-execute with `elai ...`” line) + reviewer cockpit; otherwise dossier/brief. E-23-class may justify N/A (W-6). | anchor=K-20:L57-L58
- **K-001/K20-E018** [SRC] (DEEP-K-001.discovery.md:L301 | SURFACE): E/DATA | Files: MODIFY dossier (+1 row), claims_policy_lint.py, K-7 brief (+1 line); CREATE reexecutable_rra_negative.md, K-20-decisions. | anchor=K-20:L90-L95
- **K-001/K20-E022** [SRC] (DEEP-K-001.discovery.md:L305 | SURFACE): E/EVIDENCE | Market empty-cell grounding: report 8 found no public agent stack combining re-executable properties; Bazel/OCI/CAS pinned-env shape; not an ELAI measured number. | anchor=K-20:L98-L100
- **K-001/K20-E023** [SRC] (DEEP-K-001.discovery.md:L306 | SURFACE): E/SURFACE | CLI surfaces cited (not authored here): `elai run --emit-rra`, `rra show`, `replay --verify`, planned `rra export-fixture`, A-4 `elai wrap`. | anchor=K-20:L15-L17, L50-L50
- **K-001/K20-E024** [SRC] (DEEP-K-001.discovery.md:L307 | SURFACE): E/SURFACE | User-facing surfaces in K-20 scope: dossier differentiator row; K-7 brief line; claims lint; PR “verify offline / re-execute” line via K-3/K-11; negative fixture. | anchor=K-20:L31-L35, L57-L62
- **K-001/K20-I001** [SRC] (DEEP-K-001.discovery.md:L308 | SURFACE): I/DEP | Because K-20 packages rather than implements re-execution, dashboard DESIGN.md consumers should treat re-exec as future-linked status until J-14-extension lands. | anchor=K-20:L5-L5, L57-L62
- **K-001/K20-A002** [SRC] (DEEP-K-001.discovery.md:L310 | SURFACE): A/SURFACE | Exact wording of the PR “verify offline / re-execute with `elai ...`” line is illustrative, not a frozen UI string. | anchor=K-20:L57-L58
- **K-001/K20-C001** [SRC] (DEEP-K-001.discovery.md:L311 | SURFACE): C/STATUS | Market gap language oscillates between REPLAYABLE (named market demand) and RE-EXECUTABLE (strict-stronger wedge ELAI banks); both are intentional: G-2 ships replayable; K-20 packages re-executable as stronger claim. | anchor=K-20:L5-L5, L12-L13

### Surfaces — K-001/K8 (8)

- **K-001/K8-E004** [SRC] (DEEP-K-001.discovery.md:L177 | SURFACE): E/DEP | Pre-requisites: ELAI-FEATURE-MAP.md; absorption record KS-3 report 17; WebSearch per provider native surface. Parallel-safe with K-1, K-9. | anchor=K-8:L16-L18
- **K-001/K8-E009** [SRC] (DEEP-K-001.discovery.md:L182 | SURFACE): E/CFG | Ship `.claude/rules/provider-absorption-pregate.md`: any efficiency-class unit re-verifies provider current native surface (WebSearch) before EXECUTING; duplicating a native feature is kill-or-de-scope. Add conditional-routing load line. | anchor=K-8:L55-L57, L65-L65
- **K-001/K8-E010** [SRC] (DEEP-K-001.discovery.md:L183 | SURFACE): E/TEST | NEGATIVE arm: efficiency-class task that skips provider re-check is audit-flagged; rule must fire on live efficiency-class task (JD-1). | anchor=K-8:L58-L59, L84-L88
- **K-001/K8-E011** [SRC] (DEEP-K-001.discovery.md:L184 | SURFACE): E/INT | Production reachability: conditional-routing loads rule on efficiency-class task; audit flags skipped re-check; cfg(test)/contracts never satisfies R7. | anchor=K-8:L71-L75
- **K-001/K8-E013** [SRC] (DEEP-K-001.discovery.md:L186 | SURFACE): E/SURFACE | Frontend pair N/A — harness doc + rule + feature-map upsert (W-6). Observable: feature-map tags grep-provable; rule file exists and loads; kill list user-decision artifact. | anchor=K-8:L81-L88
- **K-001/K8-E016** [SRC] (DEEP-K-001.discovery.md:L189 | SURFACE): E/TEST | W-1 done = rule loads on live efficiency-class task (JD-1), not file-exists. W-7: each unit’s provider surface re-verified at sweep (dated). | anchor=K-8:L107-L110
- **K-001/K8-E022** [SRC] (DEEP-K-001.discovery.md:L195 | SURFACE): E/SURFACE | User-facing/operator surfaces: feature-map absorption tags; kill/de-scope proposal doc for user decision; harness rule load on efficiency-class tasks; audit flag on skipped re-check. No dashboard panel. | anchor=K-8:L81-L88, L100-L105
- **K-001/K8-I001** [SRC] (DEEP-K-001.discovery.md:L196 | SURFACE): I/CFG | “Does not auto-block” means EXECUTING may still proceed after re-check documentation, while audit later flags skips — enforcement is process/audit not hard runtime refuse. | anchor=K-8:L10-L12, L58-L59, L84-L88

### Surfaces — K-001/K9 (15)

- **K-001/K9-E001** [SRC] (DEEP-K-001.discovery.md:L208 | SURFACE): E/STATUS | K-9 is PLANNED WAVE KW-2; process + bench wrapper; depends J-9 catalog; cadence cannot silently swap — user ratifies. | anchor=K-9:L1-L3
- **K-001/K9-E003** [SRC] (DEEP-K-001.discovery.md:L210 | SURFACE): E/CFG | Safety: regressing candidate is auto-KEPT-incumbent; a swap needs user ratification (GD-4/user-gate). Routing/allowlist/CLAUDE.md untouched without user sign-off. | anchor=K-9:L9-L12, L61-L62, L79-L80, L105-L106
- **K-001/K9-E007** [SRC] (DEEP-K-001.discovery.md:L214 | SURFACE): E/STATE | Prior rejections (Gemma-4, Qwen3-32B) are tier-conditional CATALOG data by J-9 (informational); routing stays untouched without sign-off. Cadence must NOT surface them as default-adopt without explicit sign-off (No-Gemma-4). | anchor=K-9:L11-L12, L38-L39, L144-L145
- **K-001/K9-E009** [SRC] (DEEP-K-001.discovery.md:L216 | SURFACE): E/CAP | Thin bench wrapper over J-9 catalog (`benchmarks/model_refresh_runner.rs` or CLI arm): run candidates vs fixtures; emit decision row. Live model-eval legs RunPod/2x3090-gated (W-10); deterministic catalog-read + decision-schema leg ships now. | anchor=K-9:L55-L58, L125-L126
- **K-001/K9-E012** [SRC] (DEEP-K-001.discovery.md:L219 | SURFACE): E/SURFACE | Production CLI: `elai bench model-refresh [--lane <lane>]` (or recurring checklist invocation). | anchor=K-9:L76-L76, L88-L88
- **K-001/K9-E013** [SRC] (DEEP-K-001.discovery.md:L220 | SURFACE): E/SURFACE | Frontend pair: refresh-decision tile (ADOPT/KEEP/RE-EVALUATE per lane) on runtime/catalog panel (decide at edit, W-4) OR W-6-justified CLI-only absence. | anchor=K-9:L84-L86, L113-L113
- **K-001/K9-E015** [SRC] (DEEP-K-001.discovery.md:L222 | SURFACE): E/LIMIT | Stub bans: no silent model swap; no routing/allowlist/CLAUDE mutation without sign-off; no re-implement J-9 catalog; no candidate win without fixture run; no `.unwrap()`/`.expect()` on prod wrapper. | anchor=K-9:L78-L83
- **K-001/K9-E016** [SRC] (DEEP-K-001.discovery.md:L223 | SURFACE): E/TEST | DETERMINISTIC_CHECK: wiring_contract_check --run-trigger; cadence doc exists; rg model-refresh in cli; rg IncumbentKept/auto-kept/ratif in runner; routing/allowlist count MUST be 0 in runner; decisions file. | anchor=K-9:L94-L102
- **K-001/K9-E017** [SRC] (DEEP-K-001.discovery.md:L224 | SURFACE): E/TEST | ACCEPTANCE: no silent swap; routing untouched; GD-1 compose J-9/E-27/J-19/E-31; HD-1 fixture-measured; cargo check GREEN; auditor PASS -p orchestrator,-p cli; SSB net 0..+1 sequential. | anchor=K-9:L104-L109
- **K-001/K9-E019** [SRC] (DEEP-K-001.discovery.md:L226 | SURFACE): E/TEST | Contract k9_model_refresh_contract.rs: catalog read; decision row fields; regress→IncumbentKept; no fixture FAIL; no routing/allowlist mutation (grep-isolation). | anchor=K-9:L117-L121
- **K-001/K9-E020** [SRC] (DEEP-K-001.discovery.md:L227 | SURFACE): E/DATA | Files: CREATE cadence doc, model_refresh_runner.rs, contract, decisions; MODIFY args_bench + bench arm; MODIFY ssb + action_type_lock IF +ModelRefreshCadence. | anchor=K-9:L123-L129
- **K-001/K9-E021** [SRC] (DEEP-K-001.discovery.md:L228 | SURFACE): E/EVIDENCE | External context [EXTERNAL/DESIGN]: KS-2 §1.9 ~24GB floor, MoE-30B-A3B, open-weight SWE-bench 70s vs frontier ~80; verdicts are fixture-measured (HD-1). | anchor=K-9:L28-L30, L131-L137
- **K-001/K9-E023** [SRC] (DEEP-K-001.discovery.md:L230 | SURFACE): E/LIMIT | Full per-lane model bench is expensive: keep fixture set small; live legs RunPod-gated; catalog-read + decision-schema is cheap always-runnable part. | anchor=K-9:L142-L143
- **K-001/K9-E025** [SRC] (DEEP-K-001.discovery.md:L232 | SURFACE): E/SURFACE | User-facing surfaces: CLI `elai bench model-refresh`; decision row ADOPT/KEEP/RE-EVALUATE; dated refresh report; optional refresh-decision tile; cadence strategy doc. | anchor=K-9:L76-L92, L84-L86
- **K-001/K9-A001** [SRC] (DEEP-K-001.discovery.md:L234 | SURFACE): A/SURFACE | Whether refresh-decision tile ships or CLI-only W-6 absence is undecided until edit. | anchor=K-9:L84-L86

### Surfaces — K-002/Cross-source interactions and lifecycle (8)

- **K-002/IX-004** [IX] (DEEP-K-002.discovery.md:L308 | SURFACE): IX | E | K-6 offline-verify path is the same `elai verify-attestation` surface K-3 embeds as an offline-verify line in PR check-runs. | K3-E004–E005; K6-E010; KGM-E011 | anchor=
- **K-002/IX-006** [IX] (DEEP-K-002.discovery.md:L310 | SURFACE): IX | E | K-6 soft-depends K-1 measured numbers; K-17/K-18/K-22 all extend the same K-1 claims_policy_lint + dossier surface. | K6-E020; K17-E005–E006, E013; K18-E009–E010; K22-E014 | anchor=
- **K-002/IX-007** [IX] (DEEP-K-002.discovery.md:L311 | SURFACE): IX | E | K-17 packages J-34 bench numbers; forbids a parallel context bench; points measured events at J-34 ContextCalibrationEvent rather than a K SSB. | K17-E008–E009, E016 | anchor=
- **K-002/IX-013** [IX] (DEEP-K-002.discovery.md:L317 | SURFACE): IX | E | KD-1/KD-2/KD-5 from the brief bind K-3 (PR render + window clock), K-6 (compliance time-box), and K-17/K-18/K-22 claim tags. | KMB-E021; K3-E001, E012; K6-E001; K17-E005; K18-E009; K22-E014 | anchor=
- **K-002/IX-016** [IX] (DEEP-K-002.discovery.md:L320 | SURFACE): IX | E | User-facing buyer surfaces split: K-3 PR/GitHub check-run + dashboard deep view; K-6 offline compliance bundle + mapping doc; K-7 brief (out of this chunk’s numbered files but sequenced by brief). | K3-E005–E006, E012; K6-E002–E004; KMB-E016–E017 | anchor=
- **K-002/IX-017** [IX] (DEEP-K-002.discovery.md:L321 | SURFACE): IX | E | Operator harness surfaces: K-17/K-18 auditor claims lint; K-18 `elai bench case-reuse`; K-22 `doctrine_coverage.py` + doctrine-enforcement rule load. | K17-E013–E015; K18-E002, E010; K22-E003, E012–E013 | anchor=
- **K-002/IX-018** [IX] (DEEP-K-002.discovery.md:L322 | SURFACE): IX | I | If K-3 frontend_pair and K-6 optional compliance tile both land, both ride dashboard-v2 but remain distinct (GitHub state vs compliance export). | K3-E012; K6-E016 | anchor=
- **K-002/IX-019** [IX] (DEEP-K-002.discovery.md:L323 | SURFACE): IX | E | Master-brief kill-switch “GitHub ships crypto evidence” pivots A-4 but keeps K-3 rendering; matches K-3’s renderer-only boundary. | KMB-E023; K3-E008 | anchor=

### Surfaces — K-002/Explicit ambiguity and conflict register (4)

- **K-002/AC-004** [AC] (DEEP-K-002.discovery.md:L333 | SURFACE): AC | A | K-3 github.svelte.ts field shape and body_composer seam stability. | K3-A001, A003 | anchor=
- **K-002/AC-005** [AC] (DEEP-K-002.discovery.md:L334 | SURFACE): AC | A | K-6 bundle directory vs archive; compliance tile yes/no; E-38 staleness. | K6-A001–A003 | anchor=
- **K-002/AC-007** [AC] (DEEP-K-002.discovery.md:L336 | SURFACE): AC | A | K-18 SSB Kind choice; panel vs new tile; lint compose with K-19/K-15/K-17. | K18-A001–A003 | anchor=
- **K-002/AC-009** [AC] (DEEP-K-002.discovery.md:L338 | SURFACE): AC | A | Map PLAN-asserted review_surface symbol names and KW-5 table page. | KGM-A001–A002 | anchor=

### Surfaces — K-002/Grounding-map deltas (1)

- **K-002/SD-KGM-001** [SD] (DEEP-K-002.discovery.md:L350 | SURFACE): SD | **ADD** full KG-1..KG-9 live anchors (I-43 CLI, github_pr file inventory, review/H-52, SAFEDELTA/QDELTA/SeCodePLT/RRA, 105 allowlist rows, args_bench sub list). | 11-K:L942-L944, L980 | KGM-E004–E013 | anchor=11-K:L942-L944, L980

### Surfaces — K-002/K-17 deltas (2)

- **K-002/SD-K17-001** [SD] (DEEP-K-002.discovery.md:L379 | SURFACE): SD | **ADD** exact folklore constants Local 0.75 / Cloud 0.85, `ELAI_CONTEXT_POLICY_DISABLE`, `ELAI_LOCAL_MAX_CONTEXT`, and 10-slot composition (Memory H-31 + Evidence H-53). | 11-K:L447-L452 | K17-E004, E010–E011 | anchor=11-K:L447-L452
- **K-002/SD-K17-002** [SD] (DEEP-K-002.discovery.md:L380 | SURFACE): SD | **ADD** zero-SSB / ContextCalibrationEvent pointer, W-2 N/A reason, and ContextStatePanel/H-9 frontend pair. | 11-K:L456-L457 | K17-E016–E017, E024 | anchor=11-K:L456-L457

### Surfaces — K-002/K-18 deltas (3)

- **K-002/SD-K18-001** [SD] (DEEP-K-002.discovery.md:L389 | SURFACE): SD | **ADD** exact CLI shape `--suite default --seed N --json`, retrieval stack BM25+dual-vector+reranker+RRF, and FIXTURE-tier-now sequencing. | 11-K:L468-L479 | K18-E002–E003, E006 | anchor=11-K:L468-L479
- **K-002/SD-K18-003** [SD] (DEEP-K-002.discovery.md:L391 | SURFACE): SD | **ADD** runner path `case_reuse_runner.rs`, ConvergencePanel/CloudPanel pair, and W-1 “must actually run” done definition. | 11-K:L485-L489 | K18-E012, E014, E019 | anchor=11-K:L485-L489
- **K-002/SD-K18-004** [SD] (DEEP-K-002.discovery.md:L392 | SURFACE): SD | **ADD** full OUT-OF-K VCG mint/wire/canvas/reuse/promotion companion scope. | 11-K:L493 | K18-E022 | anchor=11-K:L493

### Surfaces — K-002/K-22 deltas (2)

- **K-002/SD-K22-001** [SD] (DEEP-K-002.discovery.md:L399 | SURFACE): SD | **ADD** exact tool CLI `--json`, coverage triple percentages, UNANCHORED residual class, and self-tag of doctrine-enforcement.md as AUDITOR-ADVISORY. | 11-K:L585-L608 | K22-E003, E008, E025–E026 | anchor=11-K:L585-L608
- **K-002/SD-K22-005** [SD] (DEEP-K-002.discovery.md:L403 | SURFACE): SD | **ADD** ledger path alternative workflow/reasoning and fixture `doctrine_coverage_negative.md`. | 11-K:L603 | K22-E009, E023, A002 | anchor=11-K:L603

### Surfaces — K-002/K-3 deltas (3)

- **K-002/SD-K3-001** [SD] (DEEP-K-002.discovery.md:L359 | SURFACE): SD | **ADD** exact `--emit-evidence` mode, production trigger string, and optional `PrEvidenceEmitted` Kind set. | 11-K:L105-L126 | K3-E009–E010, E022 | anchor=11-K:L105-L126
- **K-002/SD-K3-003** [SD] (DEEP-K-002.discovery.md:L361 | SURFACE): SD | **ADD** W-2 allowlist-if-gated clause, cross_lang_diff requirement, and 236/545 sequential net. | 11-K:L123-L126 | K3-E020, E023, E018 | anchor=11-K:L123-L126
- **K-002/SD-K3-005** [SD] (DEEP-K-002.discovery.md:L363 | SURFACE): SD | **QUALIFY** summary UI pair: source marks check-run status on github.svelte.ts as edit-time decision, not a fixed field contract. | 11-K:L124 | K3-E012, A003 | anchor=11-K:L124

### Surfaces — K-002/K-6 deltas (1)

- **K-002/SD-K6-001** [SD] (DEEP-K-002.discovery.md:L369 | SURFACE): SD | **ADD** exact CLI flags `--out`, offline-verify composition path, and ComplianceExported Kind triad. | 11-K:L180-L205 | K6-E002, E010, E013 | anchor=11-K:L180-L205

### Surfaces — K-002/K17 (13)

- **K-002/K17-E002** [SRC] (DEEP-K-002.discovery.md:L149 | SURFACE): E/CAP | Banks differentiator: ELAI MEASURES per-(model,lane) context-degradation knee against gate verdicts and runs OPPOSITE cloud vs local context policies. | anchor=K-17:L4-L5, L12-L13, L30-L30
- **K-002/K17-E003** [SRC] (DEEP-K-002.discovery.md:L150 | SURFACE): E/CFG | Cloud regime: cache-stable prefix + late compaction; local regime: window-minimization + unload-early. | anchor=K-17:L4-L5, L30-L30
- **K-002/K17-E004** [SRC] (DEEP-K-002.discovery.md:L151 | SURFACE): E/CFG | Folklore window fractions (0.75 local / 0.85 cloud) and slot ORDER are replaced by a measured knee (after J-34). | anchor=K-17:L4-L5, L13-L13, L17-L17
- **K-002/K17-E006** [SRC] (DEEP-K-002.discovery.md:L153 | SURFACE): E/SURFACE | Claims-policy entry forces “best/optimal/never-loses context management” strings to carry measured-or-[DESIGN] qualifier + regime-split correction. | anchor=K-17:L30-L30, L32-L32
- **K-002/K17-E007** [SRC] (DEEP-K-002.discovery.md:L154 | SURFACE): E/SURFACE | K-7 brief line for AppSec/platform “how do you keep the agent on-context across a long run”. | anchor=K-17:L30-L30, L92-L92
- **K-002/K17-E009** [SRC] (DEEP-K-002.discovery.md:L156 | SURFACE): E/DEP | Number source is J-34 `elai bench context-fill` (BENCH-PENDING); measured lift waits on it. | anchor=K-17:L1-L1, L8-L8, L18-L18, L41-L41, L74-L74
- **K-002/K17-E011** [SRC] (DEEP-K-002.discovery.md:L158 | SURFACE): E/STATUS | WIRED regime policy: context_policy.rs Local=floor(window*0.75) / Cloud=floor(window*0.85), `ELAI_CONTEXT_POLICY_DISABLE`, h50_context_policy_contract.rs; effective_context_window(Local) reads `ELAI_LOCAL_MAX_CONTEXT`. | anchor=K-17:L17-L17, L97-L97
- **K-002/K17-E013** [SRC] (DEEP-K-002.discovery.md:L160 | SURFACE): E/INT | EXTEND (not fork) K-1 claims_policy_lint.py recogniser for context-management claims and context-efficiency numbers without ELAI-measured journal pointer. | anchor=K-17:L32-L32, L39-L39, L44-L44, L90-L90
- **K-002/K17-E014** [SRC] (DEEP-K-002.discovery.md:L161 | SURFACE): E/FAIL | Negative fixture fails: (a) “ELAI never loses context” untagged; (b) bare “0% degradation at full window” without measured pointer; (c) RULER/Chroma frontier knee as ELAI local-lane knee. | anchor=K-17:L33-L33, L66-L67, L86-L86
- **K-002/K17-E015** [SRC] (DEEP-K-002.discovery.md:L162 | SURFACE): E/INT | Production reachability: auditor runs claims_policy_lint.py over positioning copy; dossier row points at J-34 knee once measured; test-only never satisfies R7. | anchor=K-17:L45-L48
- **K-002/K17-E017** [SRC] (DEEP-K-002.discovery.md:L164 | SURFACE): E/SURFACE | frontend_pair: ContextStatePanel (DARK E-23 context telemetry) once mounted + H-9 cost cockpit (knee vs $/verified-task); else dossier/brief artifact; justified N/A (W-6). | anchor=K-17:L55-L56, L80-L80
- **K-002/K17-E021** [SRC] (DEEP-K-002.discovery.md:L168 | SURFACE): E/DATA | Files: dossier row; claims_policy_lint ~13 LOC; K-7 brief +1 line; `_fixtures/context_management_negative.md`; `K-17-decisions.md`. | anchor=K-17:L88-L93
- **K-002/K17-E024** [SRC] (DEEP-K-002.discovery.md:L171 | SURFACE): E/CFG | W-2 N/A: cited context_policy already carries `ELAI_CONTEXT_POLICY_DISABLE`. | anchor=K-17:L80-L80

### Surfaces — K-002/K18 (12)

- **K-002/K18-E002** [SRC] (DEEP-K-002.discovery.md:L185 | SURFACE): E/SURFACE | CLI verb: `elai bench case-reuse --suite default --seed N --json` as real production bench entrypoint. | anchor=K-18:L5-L5, L30-L30, L49-L49, L61-L61
- **K-002/K18-E003** [SRC] (DEEP-K-002.discovery.md:L186 | SURFACE): E/CAP | Recurrence-hit-rate arm (SAFE, ships now): held-out corpus from F-4-proven case rows; measures how often NEW failing task matches existing trigger_signature over BM25+dual-vector+reranker+RRF; ELAI_SEED-recomputable. | anchor=K-18:L5-L5, L30-L30, L76-L76
- **K-002/K18-E008** [SRC] (DEEP-K-002.discovery.md:L191 | SURFACE): E/CFG | Reuse-into-generation path default-OFF behind wire-dark-allowlist row + promotion plan (W-2/GD-4). | anchor=K-18:L57-L57, L84-L84
- **K-002/K18-E010** [SRC] (DEEP-K-002.discovery.md:L193 | SURFACE): E/SURFACE | Claims lint extension fails untagged “self-improving / learns from your codebase / gets better with use” and bare compounding numbers without journal pointer. | anchor=K-18:L33-L34, L41-L41, L97-L97
- **K-002/K18-E012** [SRC] (DEEP-K-002.discovery.md:L195 | SURFACE): E/CAP | Produces `benchmarks/case_reuse_runner.rs` + `case-reuse` bench sub at args_bench.rs:75; composes I-27 ConvergenceCurveMeasured + safedelta runner shape. | anchor=K-18:L38-L39, L43-L44, L94-L95
- **K-002/K18-E013** [SRC] (DEEP-K-002.discovery.md:L196 | SURFACE): E/EVENT | Prefer-reuse bench row family / ConvergenceCurveMeasured (+Kind CaseReuseArm / RecurrenceHit); net 0..+1 from LIVE ~267/577 (dated); decide at edit W-9. | anchor=K-18:L61-L62, L87-L87
- **K-002/K18-E014** [SRC] (DEEP-K-002.discovery.md:L197 | SURFACE): E/SURFACE | frontend_pair: recurrence-hit-rate + iteration/$ delta on I-27 ConvergencePanel + H-9 CloudPanel; optional VCG canvas case-altitude when shipped; decide at edit W-4 / justified N/A. | anchor=K-18:L58-L59, L85-L85
- **K-002/K18-E015** [SRC] (DEEP-K-002.discovery.md:L198 | SURFACE): E/INT | Production: real CLI bench + auditor lint; contracts never satisfy R7. | anchor=K-18:L48-L51
- **K-002/K18-E017** [SRC] (DEEP-K-002.discovery.md:L200 | SURFACE): E/DEP | Does NOT author VCG capability (mint/wire/canvas/reuse-loop); reuse-ON arm BENCH-PENDING on it. | anchor=K-18:L5-L5, L16-L16, L57-L57, L80-L80
- **K-002/K18-E021** [SRC] (DEEP-K-002.discovery.md:L204 | SURFACE): E/DATA | Files: case_reuse_runner.rs; args_bench.rs; dossier row; claims_policy_lint ~14 LOC; K-7 line; compounding_state_negative.md; K-18-decisions.md; optional SSB. | anchor=K-18:L93-L101
- **K-002/K18-E022** [SRC] (DEEP-K-002.discovery.md:L205 | SURFACE): E/DEFER | OUT-OF-K VCG J-track: mint-on-proof case node; deterministic self-wiring edges; MEMORY/CASE canvas altitude; gated retrieval reuse; episodic->skill->rule ladder. | anchor=K-18:L114-L114
- **K-002/K18-A002** [SRC] (DEEP-K-002.discovery.md:L211 | SURFACE): A/SURFACE | Whether a dedicated case-reuse tile is required vs riding existing panels is edit-time. | anchor=K-18:L58-L59

### Surfaces — K-002/K22 (12)

- **K-002/K22-E003** [SRC] (DEEP-K-002.discovery.md:L224 | SURFACE): E/SURFACE | Tool: `.claude/tools/doctrine_coverage.py` stdlib-only deterministic `--json`; extracts clause IDs from CLAUDE.md + `.claude/rules/*.md`. | anchor=K-22:L7-L7, L32-L32, L41-L41, K22:L100-L100
- **K-002/K22-E009** [SRC] (DEEP-K-002.discovery.md:L230 | SURFACE): E/SURFACE | Ledger artifact: `docs/strategy/ELAI-DOCTRINE-ENFORCEMENT-COVERAGE.md` (or workflow/reasoning/) with per-clause table + coverage % + UNENFORCED compilation BACKLOG ranked by leverage. | anchor=K-22:L33-L33, L42-L42, L101-L101
- **K-002/K22-E011** [SRC] (DEEP-K-002.discovery.md:L232 | SURFACE): E/LIMIT | Does NOT modify rulebook clauses (read+tag only); does NOT introduce runtime gate over .elai_cc; does NOT claim every clause can be compiled (Rice/halting). | anchor=K-22:L4-L4, L61-L61, L88-L88
- **K-002/K22-E012** [SRC] (DEEP-K-002.discovery.md:L233 | SURFACE): E/SURFACE | Doctrine rule: `.claude/rules/doctrine-enforcement.md` (~25-30 lines); any rulebook edit DECLARES clause enforcement tag. | anchor=K-22:L34-L34, L43-L43, L102-L102
- **K-002/K22-E014** [SRC] (DEEP-K-002.discovery.md:L235 | SURFACE): E/SURFACE | EXTEND claims_policy_lint so “fully enforced doctrine / every rule is checked / all deterministic gates” fails without measured coverage % + honest AUDITOR-ADVISORY/UNENFORCED split. | anchor=K-22:L35-L35, L44-L44, L104-L104
- **K-002/K22-E015** [SRC] (DEEP-K-002.discovery.md:L236 | SURFACE): E/FAIL | Negative: (a) fixtures “every ELAI rule is a frozen deterministic gate” untagged fails lint; (b) clause asserting checker absent from manifest is tagged UNENFORCED and coverage reflects it. | anchor=K-22:L36-L36, L69-L69, L97-L97
- **K-002/K22-E016** [SRC] (DEEP-K-002.discovery.md:L237 | SURFACE): E/INT | Production: auditor/harness runs doctrine_coverage.py; conditional-routing loads rule; lint over positioning copy; K-4 reads ledger for real vs advisory gates (KD-3). | anchor=K-22:L51-L55, L94-L94
- **K-002/K22-E018** [SRC] (DEEP-K-002.discovery.md:L239 | SURFACE): E/SURFACE | frontend_pair: N/A — tool + ledger doc + thin rule; no dashboard surface; coverage tile later optional (W-6 justified). | anchor=K-22:L64-L65, K22:L91-L92
- **K-002/K22-E020** [SRC] (DEEP-K-002.discovery.md:L241 | SURFACE): E/DEP | Checker inventory includes maintainability_check, production_caller_check, python_wire_check, traced_test_assert_check, wiring_contract_check, power_of_ten_check, claims_policy_lint, failure_taxonomy, cross_lang_diff, spec_link_check, elai_map_freshness. | anchor=K-22:L18-L18
- **K-002/K22-E023** [SRC] (DEEP-K-002.discovery.md:L244 | SURFACE): E/DATA | Files: doctrine_coverage.py; ledger md; doctrine-enforcement.md; conditional-routing ~2 LOC; claims_policy_lint ~10 LOC; doctrine_coverage_negative.md; K-22-decisions.md. | anchor=K-22:L99-L106
- **K-002/K22-E027** [SRC] (DEEP-K-002.discovery.md:L248 | SURFACE): E/EVIDENCE | EXTERNAL ContextCov ~50% prose-only is design rationale only; actual ELAI coverage % is measured by the tool (KD-1). | anchor=K-22:L4-L4, L111-L111
- **K-002/K22-I001** [SRC] (DEEP-K-002.discovery.md:L250 | SURFACE): I/OBS | Coverage % becomes a KD-1-tagged measured-ELAI number only after the tool runs over the live rulebook. | anchor=K-22:L20-L20, L33-L33

### Surfaces — K-002/K3 (16)

- **K-002/K3-E002** [SRC] (DEEP-K-002.discovery.md:L76 | SURFACE): E/CAP | Builds a RENDERER that emits ELAI per-hunk evidence into GitHub check-runs + PR comments over the already-wired github_pr adapter. | anchor=K-3:L5-L14, L54-L61
- **K-002/K3-E005** [SRC] (DEEP-K-002.discovery.md:L79 | SURFACE): E/SURFACE | One check-run summarizing gate verdicts + attestation digest + offline-verify line. | anchor=K-3:L58-L59, L95-L99
- **K-002/K3-E006** [SRC] (DEEP-K-002.discovery.md:L80 | SURFACE): E/SURFACE | Per-hunk PR comments placed only via the H-52 finding-anchor resolver; hallucinated line numbers are structurally impossible. | anchor=K-3:L10-L11, L59-L61, L89-L89, L115-L115
- **K-002/K3-E009** [SRC] (DEEP-K-002.discovery.md:L83 | SURFACE): E/SURFACE | CLI verb: `elai pr-review` evidence-emit mode; production trigger `elai pr-review <pr>` / fixture `elai pr-review <fixture-pr> --emit-evidence`. | anchor=K-3:L62-L63, L82-L84, L95-L96, L104-L104
- **K-002/K3-E010** [SRC] (DEEP-K-002.discovery.md:L84 | SURFACE): E/EVENT | Prefer-reuse github_pr / review_surface / attestation SSB rows (net 0..+1); else +`PrEvidenceEmitted` with Kind CheckRunPosted / HunkVouched / HunkUnvouched. | anchor=K-3:L63-L64, L74-L74, L96-L97
- **K-002/K3-E011** [SRC] (DEEP-K-002.discovery.md:L85 | SURFACE): E/FAIL | Negative arm: flag `--post-unvouched-as-clean` MUST be rejected/fail; closed-set render makes out-of-evidence “clean” structurally impossible (R2). | anchor=K-3:L65-L66, L99-L99, L106-L106, L131-L131
- **K-002/K3-E012** [SRC] (DEEP-K-002.discovery.md:L86 | SURFACE): E/SURFACE | frontend_pair: surface emitted check-run status on `crates/dashboard-v2/src/lib/state/github.svelte.ts` (decide at edit, W-4); dashboard = DEEP view, PR = buyer view (KD-2). | anchor=K-3:L67-L67, L91-L93, L112-L112, L123-L124
- **K-002/K3-E013** [SRC] (DEEP-K-002.discovery.md:L87 | SURFACE): E/INT | Production reachability: CLI evidence-emit mode -> github_pr adapter -> real GitHub check-run; `#[cfg(test)]`/contracts never satisfy R7. | anchor=K-3:L81-L84
- **K-002/K3-E014** [SRC] (DEEP-K-002.discovery.md:L88 | SURFACE): E/DEP | Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. | anchor=K-3:L18-L21, L75-L80
- **K-002/K3-E016** [SRC] (DEEP-K-002.discovery.md:L90 | SURFACE): E/DATA | Creates `evidence_emitter.rs` under github_pr/; modifies `pr_review_cmd.rs` (~40 LOC for `--emit-evidence`); optional SSB + github.svelte.ts; contract `k3_pr_evidence_emitter_contract.rs`; `workflow/reasoning/K-3-decisions.md`. | anchor=K-3:L134-L139
- **K-002/K3-E017** [SRC] (DEEP-K-002.discovery.md:L91 | SURFACE): E/TEST | Contract arms: full-evidence check-run + vouched comments; no-evidence UNVOUCHED; H-52 placement; every claim has journal row; `--post-unvouched-as-clean` rejected. | anchor=K-3:L128-L131
- **K-002/K3-E018** [SRC] (DEEP-K-002.discovery.md:L92 | SURFACE): E/TEST | Deterministic checks include wiring_contract_check, rg for emit-evidence mode and UNVOUCHED, broken-arm absence, cross_lang_diff on github_pr + github.svelte.ts. | anchor=K-3:L101-L109
- **K-002/K3-E022** [SRC] (DEEP-K-002.discovery.md:L96 | SURFACE): E/OBS | Expect artifact: real check-run whose every claim joins a journal row (GD-2); out-of-evidence comment structurally impossible. | anchor=K-3:L94-L99, L120-L120
- **K-002/K3-E023** [SRC] (DEEP-K-002.discovery.md:L97 | SURFACE): E/CFG | W-2: emit mode is invoked, not default-OFF; IF gated, register in wire-dark-allowlist with promotion plan. | anchor=K-3:L121-L122
- **K-002/K3-I001** [SRC] (DEEP-K-002.discovery.md:L100 | SURFACE): I/SURFACE | Tauri dashboard remains the deep operator surface while PR check-runs/comments are the meet-the-buyer surface under KD-2. | anchor=K-3:L30-L32, L91-L93
- **K-002/K3-A003** [SRC] (DEEP-K-002.discovery.md:L103 | SURFACE): A/SURFACE | Exact check-run status fields on `github.svelte.ts` decided at edit (W-4). | anchor=K-3:L67-L67, L91-L93

### Surfaces — K-002/K6 (8)

- **K-002/K6-E002** [SRC] (DEEP-K-002.discovery.md:L114 | SURFACE): E/SURFACE | CLI verb: `elai export compliance [--out <dir\ | anchor=archive>]`; production trigger `elai export compliance --out /tmp/elai-compliance-bundle`.
- **K-002/K6-E004** [SRC] (DEEP-K-002.discovery.md:L116 | SURFACE): E/SURFACE | Mapping document path: `docs/strategy/ELAI-COMPLIANCE-MAPPING.md` translating bundle fields onto procurement clauses. | anchor=K-6:L57-L61, L71-L71, L102-L102
- **K-002/K6-E010** [SRC] (DEEP-K-002.discovery.md:L122 | SURFACE): E/TEST | Bundle verifies offline on a clean machine via `elai verify-attestation` + conformance checker. | anchor=K-6:L27-L28, L62-L63, L94-L96, L110-L110
- **K-002/K6-E014** [SRC] (DEEP-K-002.discovery.md:L126 | SURFACE): E/INT | Production reachability is the real CLI export arm; test-only/contracts never satisfy R7. | anchor=K-6:L78-L80
- **K-002/K6-E016** [SRC] (DEEP-K-002.discovery.md:L128 | SURFACE): E/SURFACE | frontend_pair: N/A CLI+docs justified (W-6) OR optional compliance-export tile (decide at edit). | anchor=K-6:L87-L89, L117-L118
- **K-002/K6-A001** [SRC] (DEEP-K-002.discovery.md:L135 | SURFACE): A/DATA | Bundle format single archive vs directory layout unresolved; recommend directory + optional archive flag after reading I-44/G-8 shapes. | anchor=K-6:L55-L56, L143-L145
- **K-002/K6-A003** [SRC] (DEEP-K-002.discovery.md:L137 | SURFACE): A/SURFACE | Whether a compliance-export dashboard tile ships in v1 is edit-time choice. | anchor=K-6:L87-L89
- **K-002/K6-I001** [SRC] (DEEP-K-002.discovery.md:L138 | SURFACE): I/COMP | Mapping doc is a procurement-language surface distinct from the machine-verifiable bundle itself. | anchor=K-6:L7-L12, L57-L61

### Surfaces — K-002/KGM (14)

- **K-002/KGM-E004** [SRC] (DEEP-K-002.discovery.md:L37 | SURFACE): E/CAP | KG-1: I-43 claim-scoper is SHIPPED with `ClaimScopeMeasured` (PilotDelta/CrossLaneDivergence/ClaimScoped), CLI `elai bench claim-scope [--suite] [--seed] [--json]`, emitter `bench_claim_scope::run_inner`, fixture contract. | anchor=KGM:L27-L33
- **K-002/KGM-E006** [SRC] (DEEP-K-002.discovery.md:L39 | SURFACE): E/CAP | KG-2: `github_pr` is a FULL WIRED module (review, body_composer, comment_watcher, adapter_trait, gh_cli, semgrep_gate, auto_merge_guard, package_verify, git_assist, clock, types, tests) plus CLI `pr_review_cmd`/`github_pr_cmd`, Tauri `commands_c40.rs`, frontend `github.svelte.ts`, two contracts. | anchor=KGM:L34-L40
- **K-002/KGM-E007** [SRC] (DEEP-K-002.discovery.md:L40 | SURFACE): E/CAP | KG-3: review surface present via `review_surface.rs`, `review_cmd.rs`, G-5 contracts, H-52 anchor contract; `score_review_surface`/`review_anchor` names PLAN-asserted at function grain, re-grep at edit. | anchor=KGM:L41-L45
- **K-002/KGM-E011** [SRC] (DEEP-K-002.discovery.md:L44 | SURFACE): E/CAP | KG-7: RRA CLI present (`rra_cmd.rs`); `elai verify-attestation` referenced; precondition for A-4/K-6. | anchor=KGM:L54-L55
- **K-002/KGM-E012** [SRC] (DEEP-K-002.discovery.md:L45 | SURFACE): E/DATA | KG-8: `wire-dark-allowlist.toml` has 105 entries; source for K-4 gate inventory and K-5 policy-table input. | anchor=KGM:L56-L58
- **K-002/KGM-E013** [SRC] (DEEP-K-002.discovery.md:L46 | SURFACE): E/SURFACE | KG-9: bench sub-command dispatcher live at `args_bench.rs:75` with named subs list; K-2 and K-4 ADD subs here. | anchor=KGM:L59-L62
- **K-002/KGM-E017** [SRC] (DEEP-K-002.discovery.md:L50 | SURFACE): E/DEP | KC-4: A-4 (`elai wrap` + .rra) is PLANNED; K-3 does NOT wait on A-4 and renders own-run evidence first. | anchor=KGM:L80-L83
- **K-002/KGM-E019** [SRC] (DEEP-K-002.discovery.md:L52 | SURFACE): E/EVIDENCE | KW-2 patch-insecurity EXTERNAL: Llama-3.3-70B introduced 135 verified vulns vs 12 developer (=11x) over 20,000+ SWE-bench issues; agentic frameworks also introduce vulns; more autonomy = more vulns. | anchor=KGM:L99-L103
- **K-002/KGM-E020** [SRC] (DEEP-K-002.discovery.md:L53 | SURFACE): E/EVIDENCE | KW-3 reward-hacking EXTERNAL: 73.8% Kernel-Bench and 46.8% ALE-Bench proxy-only gains; proxy-reality gap widens 26.4%->57.8% from 10 to 100 steps. | anchor=KGM:L104-L107
- **K-002/KGM-E022** [SRC] (DEEP-K-002.discovery.md:L55 | SURFACE): E/EVIDENCE | KW-5 SpecBench EXTERNAL “+28pp per 10x code size” is PLAN-asserted at exact-number grain; dossier-build must quote precise table+page before use. | anchor=KGM:L113-L116
- **K-002/KGM-E025** [SRC] (DEEP-K-002.discovery.md:L58 | SURFACE): E/DEP | KS-2 post-research scorecard + kill-switches; KS-3 Perplexity reports on-disk path unknown; KW verifications stand on primary sources. | anchor=KGM:L130-L134
- **K-002/KGM-E028** [SRC] (DEEP-K-002.discovery.md:L61 | SURFACE): E/LIMIT | Every external figure is tagged `[EXTERNAL]` and NEVER claimed as an ELAI measured number (KD-1). | anchor=KGM:L87-L91
- **K-002/KGM-A001** [SRC] (DEEP-K-002.discovery.md:L63 | SURFACE): A/DATA | Exact `score_review_surface` / `review_anchor` symbol names remain PLAN-asserted until executor re-grep. | anchor=KGM:L44-L45
- **K-002/KGM-A002** [SRC] (DEEP-K-002.discovery.md:L64 | SURFACE): A/EVIDENCE | KW-5 exact SpecBench table value/page anchor unresolved until dossier-build. | anchor=KGM:L113-L116

### Surfaces — K-002/KMB (13)

- **K-002/KMB-E004** [SRC] (DEEP-K-002.discovery.md:L265 | SURFACE): E/COMP | Documented procurement demands: Tennessee RFI traceability/audit/third-party assessment/data-flow/retention/kill-switch; OMB M-25-22 independent evaluation + reproducible results; EU AI Act GPAI from 2026-08-02; Art-50; PLD from 2026-12-09; insurance AI sublimits. | anchor=KMB:L86-L94
- **K-002/KMB-E005** [SRC] (DEEP-K-002.discovery.md:L266 | SURFACE): E/DEP | Moat ranking: 1 distribution, 2 per-customer compounding state, 3 proprietary verification, 4 enterprise trust/compliance, 5 cost engineering (weakest). ELAI strongest at 3, building 2, weakest at 1. | anchor=KMB:L109-L113
- **K-002/KMB-E010** [SRC] (DEEP-K-002.discovery.md:L271 | SURFACE): E/FLOW | Order: A-1 -> (go) -> A-2 -> A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips; A-6..A-8 parallel-safe before K-2/K-9. | anchor=KMB:L201-L203
- **K-002/KMB-E012** [SRC] (DEEP-K-002.discovery.md:L273 | SURFACE): E/CAP | K-2 `elai bench routing-economics` hybrid number; live cost leg needs A-8. | anchor=KMB:L222-L237
- **K-002/KMB-E013** [SRC] (DEEP-K-002.discovery.md:L274 | SURFACE): E/CAP | K-3 PR/CI evidence emitter (distribution leg): check-runs + PR comments; UNVOUCHED no-evidence; closed-set from RRA; `--post-unvouched-as-clean` fails. | anchor=KMB:L239-L254
- **K-002/KMB-E014** [SRC] (DEEP-K-002.discovery.md:L275 | SURFACE): E/CAP | K-4 composed gate-tax budget with locked ceiling; breach = release BLOCK; `elai bench gate-tax`. | anchor=KMB:L256-L270
- **K-002/KMB-E016** [SRC] (DEEP-K-002.discovery.md:L277 | SURFACE): E/CAP | K-6 `elai export compliance` bundle + mapping; Art-50 designed-to-support only; no new attestation machinery. | anchor=KMB:L289-L302
- **K-002/KMB-E020** [SRC] (DEEP-K-002.discovery.md:L281 | SURFACE): E/CAP | K-10 sequencing/cut doctrine: A-1..A-4+K-3 above everything; NO NEW PLAN LETTERS until those DONE; activation-over-authoring; canvas H-10..H-16 demoted behind K-3+A-5; I-31 last. | anchor=KMB:L343-L355
- **K-002/KMB-E021** [SRC] (DEEP-K-002.discovery.md:L282 | SURFACE): E/CFG | Doctrine KD-1 number-before-claim; KD-2 meet-the-buyer-in-the-PR; KD-3 composed-friction ceiling; KD-4 provider-absorption pre-gate; KD-5 window-clock for wedge artifacts. | anchor=KMB:L365-L378
- **K-002/KMB-E023** [SRC] (DEEP-K-002.discovery.md:L284 | SURFACE): E/RECOVERY | Kill-switches: null/negative A-1/A-3 safety delta -> stop K-T2/K-T4 headline trust; K-4 tax untolerable -> aggressive collapse; GitHub ships crypto evidence -> A-4 pivots compatibility, K-3 survives; A-5 no review-time win -> cockpit compliance artifact, K-3 remains; no 2x3090 -> K-9 still runs, 24GB adoption deferred. | anchor=KMB:L397-L410
- **K-002/KMB-E026** [SRC] (DEEP-K-002.discovery.md:L287 | SURFACE): E/DET | W-7 verify-at-authoring list: live SCHEMA/TV; github_pr real state; G-5/H-52; safedelta/seccodeplt/bench_cmd; I-43 code status; wire-dark allowlist; J-28 UsageTurnEvent. | anchor=KMB:L423-L435
- **K-002/KMB-E030** [SRC] (DEEP-K-002.discovery.md:L291 | SURFACE): E/A | Open questions: K-4 ceiling VALUE user call; K-3 GitHub-only vs GitLab; K-6 archive vs directory; A-5 study n/pilot framing; whether K-10 lives only in K-0 or also `.claude/rules/`; canvas demotion confirmation. | anchor=KMB:L455-L469
- **K-002/KMB-E031** [SRC] (DEEP-K-002.discovery.md:L292 | SURFACE): E/STATUS | One-screen summary: prove, distribute, make affordable, package for buyer, install discipline; ~10 net-new sessions + 8 pointers + 5 doctrine + 5 kill-switches. | anchor=KMB:L472-L483

### Surfaces — K-002/Master-brief deltas (2)

- **K-002/SD-KMB-001** [SD] (DEEP-K-002.discovery.md:L409 | SURFACE): SD | **ADD** full strategy (a)-(d), procurement demands, absorption/moat ranking, kill-switches, non-goals, and KD-1..KD-5 text. | 11-K:L35-L45, L942-L943 | KMB-E002–E008, E021–E023 | anchor=11-K:L35-L45, L942-L943
- **K-002/SD-KMB-004** [SD] (DEEP-K-002.discovery.md:L412 | SURFACE): SD | **FLAG** stale brief anchors superseded by grounding map (I-43, SCHEMA, github_pr). | 11-K:L944 | KMB-C001–C002; SD-KGM-002 | anchor=11-K:L944

### Surfaces — K-003/Ambiguity and conflict register (chunk-local) (2)

- **K-003/AC-002** [AC] (DEEP-K-003.discovery.md:L470 | SURFACE): AC | C | Summary args_bench :75 vs source :52-93 STALE | K2-C002 | anchor=
- **K-003/AC-005** [AC] (DEEP-K-003.discovery.md:L473 | SURFACE): AC | A | K-15/K-2/K-24 frontend tile exact panel deferred to edit | K2-A003, K15-A002, K24-A002 | anchor=

### Surfaces — K-003/Cross-file interaction records (6)

- **K-003/IX-006** [IX] (DEEP-K-003.discovery.md:L370 | SURFACE): IX | K13-E011, K14-E010, K15-E009, K21-E010, K24-E021 | Multiple units EXTEND the same K-1 claims_policy_lint recogniser without forking it (OWASP cells, async-replay, reasoning-grounding, completeness, billing claims). | K-13:L102-L104; K-14:L105-L106; K-15:L34-L34; K-21:L43-L43; K-24:L114-L114 | anchor=
- **K-003/IX-009** [IX] (DEEP-K-003.discovery.md:L373 | SURFACE): IX | K12-E004, K15-E017 | K-12 extends I-11 BeliefState with critical_atoms; J-83 (OUT-OF-K via K-15) is the typed BeliefState reasoning carrier — composition surface, not dual authorship in this chunk. | K-12:L29-L29; K-15:L116-L116 | anchor=
- **K-003/IX-012** [IX] (DEEP-K-003.discovery.md:L376 | SURFACE): IX | K26-E016/E030, K27-E002/E011 | K-27 extends K-26 toward PS-3 L-track greenfield; K-26 explicitly left PS-3 OUT OF SCOPE for its window. | K-26:L147-L147; K-27:L3-L3, L16-L16 | anchor=
- **K-003/IX-014** [IX] (DEEP-K-003.discovery.md:L378 | SURFACE): IX | K12-E010, K24-E007 | Kill-switch / allowlist patterns: K-12 default-ON disable flag vs K-24 default-OFF enable flag — opposite polarity, both W-2/R9 disciplined. | K-12:L31-L31; K-24:L35-L35 | anchor=
- **K-003/IX-016** [IX] (DEEP-K-003.discovery.md:L380 | SURFACE): IX | K2-E018, K15-E011, K24-E007, K26-E030 | Shared CLI surface family: `elai bench *` and `elai billing *` / `elai usage` / `elai run` as production entrypoints that satisfy R7. | K-2:L88-L88; K-15:L50-L51; K-24:L60-L60; K-26:L70-L95 | anchor=
- **K-003/IX-018** [IX] (DEEP-K-003.discovery.md:L382 | SURFACE): IX | K26-E017, K2-E022 | K-26 seed queue includes dashboard tiles for rows K-2/K-18a emit; K-2 FORK-3 defers its own cost panel while RunnerPresent — campaign may build adjacent panels first. | K-26:L61-L63; K-2:L67-L67 | anchor=

### Surfaces — K-003/K-12 deltas (1)

- **K-003/SD-K12-003** [SD] (DEEP-K-003.discovery.md:L405 | SURFACE): SD | **FLAG** default-ON disable flag vs observe-then-enforce dual rollout (summary A-06); source confirms both without a single transition schedule. | 11-K:L319 | K12-E010, K12-A003 | anchor=11-K:L319

### Surfaces — K-003/K-13 deltas (1)

- **K-003/SD-K13-002** [SD] (DEEP-K-003.discovery.md:L412 | SURFACE): SD | **ADD** kill-switch-1 caveat for NULL/NEGATIVE safety delta on the injection cell; dossier-row-home open question. | 11-K:L327-L353 | K13-E022, K13-A001 | anchor=11-K:L327-L353

### Surfaces — K-003/K-14 deltas (1)

- **K-003/SD-K14-003** [SD] (DEEP-K-003.discovery.md:L421 | SURFACE): SD | **ADD** exact G3-G6 async-tool primitives named as differentiator factual basis (BackgroundTaskManager/AgentNotificationEmitted). | 11-K:L373-L375 | K14-E012, K14-E009 | anchor=11-K:L373-L375

### Surfaces — K-003/K-15 deltas (1)

- **K-003/SD-K15-002** [SD] (DEEP-K-003.discovery.md:L428 | SURFACE): SD | **ADD** exact CLI flags `--suite default --seed N --json` and Kind name ReasoningBudgetArm. | 11-K:L383-L412 | K15-E011, K15-E012 | anchor=11-K:L383-L412

### Surfaces — K-003/K-2 deltas (2)

- **K-003/SD-K2-004** [SD] (DEEP-K-003.discovery.md:L395 | SURFACE): SD | **ADD** FORK-1/2/3 (SSB reuse, live funding, frontend tile), RECON_NEEDED-1..4, ABORT-1..4, INTEGRATION_SMOKE hard-input requirements. | 11-K:L78-L103 | K2-E020-E022, K2-E028-E029 | anchor=11-K:L78-L103
- **K-003/SD-K2-005** [SD] (DEEP-K-003.discovery.md:L396 | SURFACE): SD | **QUALIFY** summary K-2.12 'Feed H-9 cockpit display' and K-2.14 'Leave E-39 cap supremacy unchanged': neither is a load-bearing acceptance clause in the numbered K-2 plan body (H-9 appears only as deferred panel context; E-39 is not named). | 11-K:L93-L95 | K2-E022, K2-E024 | anchor=11-K:L93-L95

### Surfaces — K-003/K-21 deltas (1)

- **K-003/SD-K21-002** [SD] (DEEP-K-003.discovery.md:L436 | SURFACE): SD | **ADD** OUT-OF-K E1 deliverables including C8-CANVAS shadow-activity anomaly nodes, E2 observed_calls DKG edges, E3 machine-derived-first audit lane. | 11-K:L579 | K21-E015 | anchor=11-K:L579

### Surfaces — K-003/K-24 deltas (1)

- **K-003/SD-K24-003** [SD] (DEEP-K-003.discovery.md:L445 | SURFACE): SD | **ADD** exact module layout billing/{...}, openrouter_backend.rs, wire-dark allowlist promotion_plan_id=K-24-promotion, LIVE 268/577 bump note. | 11-K:L666-L717 | K24-E007, K24-E019, K24-E022 | anchor=11-K:L666-L717

### Surfaces — K-003/K-26 deltas (1)

- **K-003/SD-K26-002** [SD] (DEEP-K-003.discovery.md:L453 | SURFACE): SD | **ADD** seed-queue concrete .elai_cc tasks including named dashboard panels/tiles and auth_broker writer=None seam. | 11-K:L756-L781 | K26-E017, K26-E030 | anchor=11-K:L756-L781

### Surfaces — K-003/K-27 deltas (1)

- **K-003/SD-K27-002** [SD] (DEEP-K-003.discovery.md:L462 | SURFACE): SD | **FLAG CONFLICT** source is truncated mid-sentence; summary line count 32 vs manifest logical 33 is trailing-newline artifact; campaign procedure after MOVE 0 is not recoverable from this image. | 11-K:L954 | K27-C001, K27-A001 | anchor=11-K:L954

### Surfaces — K-003/K12 (8)

- **K-003/K12-E010** [SRC] (DEEP-K-003.discovery.md:L92 | SURFACE): E/CFG | Default-ON behind ELAI_LOSSLESS_GATE_DISABLE (W-2); OBSERVE-then-enforce (GD-4: bound FP rate over >=3 real runs before BLOCK-by-default). | anchor=K-12:L31-L31, L74-L74
- **K-003/K12-E013** [SRC] (DEEP-K-003.discovery.md:L95 | SURFACE): E/SURFACE | frontend_pair N/A justified — gates are runtime fail-closed checks; verdicts flow to SSB that existing context/trust panels already read. | anchor=K-12:L51-L52
- **K-003/K12-E015** [SRC] (DEEP-K-003.discovery.md:L97 | SURFACE): E/TEST | Contract k12_losslessness_gate_contract.rs: unrecoverable>0 returns Err; atom present PASSES; atom drop FAILS; paraphrase does NOT satisfy; recover_stubs hash-mismatch still StubRecoveryFailed; ELAI_LOSSLESS_GATE_DISABLE downgrades to observe. | anchor=K-12:L89-L90
- **K-003/K12-E019** [SRC] (DEEP-K-003.discovery.md:L101 | SURFACE): E/RECOVERY | Honest residue: external side-effects stay surfaced from saga log (RV-4), never auto-reverted. | anchor=K-12:L28-L28
- **K-003/K12-E021** [SRC] (DEEP-K-003.discovery.md:L103 | SURFACE): E/DET | Gate verdict recomputable from SSB journal under ELAI_SEED (GD-2); atom gate works on action record not tokens (GD-5). | anchor=K-12:L86-L87
- **K-003/K12-A002** [SRC] (DEEP-K-003.discovery.md:L106 | SURFACE): A/CFG | R1 interaction with H-50 local window-minimization: fail-close must force-archive before minimize trim without blocking legitimate minimization — verify against live trim path. | anchor=K-12:L107-L107
- **K-003/K12-A003** [SRC] (DEEP-K-003.discovery.md:L107 | SURFACE): A/CFG | Default-ON behind disable flag AND observe-then-enforce: initial enforcement mode transition (observe vs hard BLOCK) is not a single concrete schedule beyond '>=3 real runs'. | anchor=K-12:L31-L31, L74-L74
- **K-003/K12-I001** [SRC] (DEEP-K-003.discovery.md:L108 | SURFACE): I/SURFACE | Because frontend_pair is N/A-justified and verdicts already flow via SSB, operator-visible surface is existing context/trust panels rather than a new drawer. | anchor=K-12:L51-L52

### Surfaces — K-003/K13 (10)

- **K-003/K13-E002** [SRC] (DEEP-K-003.discovery.md:L119 | SURFACE): E/SURFACE | Artifact 1 [PACKAGE/WI-5]: docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md translating ELAI trust spine onto OWASP LLM01-10 (2025) + Agentic ASI01-10 (2025), one cell per threat, tagged measured-vs-design with citation (KD-1). | anchor=K-13:L8-L11, L91-L96
- **K-003/K13-E003** [SRC] (DEEP-K-003.discovery.md:L120 | SURFACE): E/SURFACE | Artifact 2 [PROVE/WI-4]: bind prompt-injection (LLM01/LLM05/ASI01) to G-6 SAFEDELTA AgentDojo gate-ON-vs-OFF delta as K-1 dossier row tagged [BENCH-PENDING] until run lands. | anchor=K-13:L11-L14, L97-L101
- **K-003/K13-E005** [SRC] (DEEP-K-003.discovery.md:L122 | SURFACE): E/DATA | Each mapping cell = ELAI control(s) + code/plan citation + STATUS tag (WIRED-MEASURED | anchor=WIRED-DESIGN
- **K-003/K13-E009** [SRC] (DEEP-K-003.discovery.md:L126 | SURFACE): E/DEP | E-37 PROMPT INJECTION DEFENSE PARTIAL (Spotlighting + PromptArmor; output_sanitizer/classifier/egress allowlist/AgentDojo-CI absent) — WI-3 completes perimeter OUT-OF-K. | anchor=K-13:L66-L67
- **K-003/K13-E011** [SRC] (DEEP-K-003.discovery.md:L128 | SURFACE): E/INT | EXTEND K-1 claims_policy_lint recogniser so OWASP cell asserting 'covered'/'protected' without measured number AND without [DESIGN] tag FAILS. ~15-30 LOC. | anchor=K-13:L102-L104
- **K-003/K13-E013** [SRC] (DEEP-K-003.discovery.md:L130 | SURFACE): E/SURFACE | Feeds K-7 AppSec/insurance brief (buyer 'OWASP coverage?' question). | anchor=K-13:L11-L11, L75-L75
- **K-003/K13-E015** [SRC] (DEEP-K-003.discovery.md:L132 | SURFACE): E/OBS | observable_effect trigger: `python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`; expect_ssb none (E-23-class). | anchor=K-13:L135-L140
- **K-003/K13-E016** [SRC] (DEEP-K-003.discovery.md:L133 | SURFACE): E/SURFACE | frontend_pair N/A — positioning/procurement doc + CI/audit lint; no dashboard surface (W-6 justified). | anchor=K-13:L133-L134
- **K-003/K13-E020** [SRC] (DEEP-K-003.discovery.md:L137 | SURFACE): E/DATA | Files: CREATE ELAI-OWASP-CONTROL-MAPPING.md, owasp_negative.md, K-13-decisions.md; MODIFY ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 row), claims_policy_lint.py. | anchor=K-13:L177-L182
- **K-003/K13-E022** [SRC] (DEEP-K-003.discovery.md:L139 | SURFACE): E/LIMIT | Kill-switch-1 caveat from K-0 applies: NULL/NEGATIVE safety delta re-aims the wedge for the injection cell. | anchor=K-13:L201-L204

### Surfaces — K-003/K14 (8)

- **K-003/K14-E004** [SRC] (DEEP-K-003.discovery.md:L154 | SURFACE): E/CFG | TZ-1 PLACEMENT: wall-clock/elapsed context block is VOLATILE and MUST sit in dedicated 3E slot STRICTLY AFTER the last cache breakpoint (never system/rules/PhaseSpec). | anchor=K-14:L53-L55
- **K-003/K14-E008** [SRC] (DEEP-K-003.discovery.md:L158 | SURFACE): E/SURFACE | Author .claude/rules/time-determinism.md (~50-70 lines) + conditional-routing.md trigger for timestamp/elapsed/watchdog/async-tool tasks. | anchor=K-14:L94-L97
- **K-003/K14-E009** [SRC] (DEEP-K-003.discovery.md:L159 | SURFACE): E/SURFACE | Differentiator claim: 'decoupled async tools whose lifecycle lands in a deterministic, replayable, taint-tagged journal' — K-1 dossier [DESIGN] row + claims-policy entry + K-7 brief line; honest partial-built status (G3-G6 wired; model-facing loop J-T4/T5). | anchor=K-14:L98-L104
- **K-003/K14-E010** [SRC] (DEEP-K-003.discovery.md:L160 | SURFACE): E/INT | EXTEND K-1 claims_policy_lint so async tools/decoupled/replayable claim without [DESIGN] + partial-status FAILS. ~10-15 LOC. | anchor=K-14:L105-L106
- **K-003/K14-E012** [SRC] (DEEP-K-003.discovery.md:L162 | SURFACE): E/DEP | Consumes: ClockProvider/OsClock/SeededClock core/src/clock.rs:43/85/124; 3E allocator 8 slots + 2 cache breakpoints; RRA G-2; ND-1 J-66; WatchdogTimeout SSB; background_task_id + BackgroundTaskManager + BackgroundTaskStarted/Complete + AgentNotificationEmitted (H-36). | anchor=K-14:L69-L82, L118-L122
- **K-003/K14-E013** [SRC] (DEEP-K-003.discovery.md:L163 | SURFACE): E/SURFACE | frontend_pair N/A — doctrine file + positioning/claims; no dashboard (W-6). | anchor=K-14:L134-L135
- **K-003/K14-E016** [SRC] (DEEP-K-003.discovery.md:L166 | SURFACE): E/DATA | Files: CREATE time-determinism.md, async_replay_negative.md, K-14-decisions.md; MODIFY conditional-routing.md, ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 row), claims_policy_lint.py. | anchor=K-14:L173-L179
- **K-003/K14-E017** [SRC] (DEEP-K-003.discovery.md:L167 | SURFACE): E/EVIDENCE | arXiv:2510.23853 + 2601.13206 temporally-blind grounding [EXTERNAL]; never claimed as ELAI number. | anchor=K-14:L33-L33, L186-L187

### Surfaces — K-003/K15 (10)

- **K-003/K15-E002** [SRC] (DEEP-K-003.discovery.md:L184 | SURFACE): E/SURFACE | Builds `elai bench reasoning-budget` — per-(model,task-type) verified-outcome-rate vs reasoning-compute curve under SAFEDELTA gate-ON-vs-OFF (GRC-v3 router ON vs flat-budget OFF). | anchor=K-15:L5-L5, L31-L31
- **K-003/K15-E004** [SRC] (DEEP-K-003.discovery.md:L186 | SURFACE): E/SURFACE | Differentiator [PACKAGE/PROVE]: 'ELAI's reasoning engine grounds every reasoning step on an EXTERNAL receipt; the model never grades its own confidence' — K-1 dossier [DESIGN] + claims-policy + K-7 brief. | anchor=K-15:L5-L5, L33-L33
- **K-003/K15-E005** [SRC] (DEEP-K-003.discovery.md:L187 | SURFACE): E/LIMIT | Does NOT author J-83..J-86; bench is BENCH-PENDING until J track lands; NO measured number for differentiator until bench runs; done-gate is ELAI-measured on gold set NEVER paper constant; does NOT re-implement I-27/J-34/H-9/G-6. | anchor=K-15:L5-L5
- **K-003/K15-E007** [SRC] (DEEP-K-003.discovery.md:L189 | SURFACE): E/DEP | Composes I-27 ConvergenceCurveMeasured + bench_convergence (DARK); J-34 `elai bench context-fill` sibling shape; safedelta_runner + args_bench.rs:75 seam; H-9 CloudPanel. | anchor=K-15:L7-L8, L15-L21
- **K-003/K15-E011** [SRC] (DEEP-K-003.discovery.md:L193 | SURFACE): E/SURFACE | CLI: `elai bench reasoning-budget --suite default --seed N --json` (PRODUCTION bench). | anchor=K-15:L50-L51, L64-L64
- **K-003/K15-E013** [SRC] (DEEP-K-003.discovery.md:L195 | SURFACE): E/SURFACE | frontend_pair: curve + knee on I-27 ConvergencePanel + H-9 CloudPanel — reasoning-budget arm on existing convergence tile (decide at edit, W-4); justified N/A if rides those panels. | anchor=K-15:L61-L62
- **K-003/K15-E014** [SRC] (DEEP-K-003.discovery.md:L196 | SURFACE): E/OBS | y-axis = receipts/verified-OUTCOME, NOT self-reported accuracy or pass@k (GD-3); curve recomputable under ELAI_SEED (GD-2). | anchor=K-15:L55-L58, L79-L79
- **K-003/K15-E016** [SRC] (DEEP-K-003.discovery.md:L198 | SURFACE): E/DATA | Files: CREATE reasoning_budget_runner.rs, reasoning_grounding_negative.md, K-15-decisions.md; MODIFY args_bench.rs, J-34 calibration writer, dossier, claims_policy_lint.py; optional SSB Kind. | anchor=K-15:L96-L104
- **K-003/K15-E018** [SRC] (DEEP-K-003.discovery.md:L200 | SURFACE): E/LIMIT | NO LLM-judge scoring in the bench (TD-3); NO fork of I-27/J-34/K-1 lint; NO main-LLM self-run of lint. | anchor=K-15:L59-L59
- **K-003/K15-A002** [SRC] (DEEP-K-003.discovery.md:L202 | SURFACE): A/SURFACE | Exact panel tile vs ride-existing-panels decision deferred to edit under W-4. | anchor=K-15:L61-L62

### Surfaces — K-003/K2 (13)

- **K-003/K2-E003** [SRC] (DEEP-K-003.discovery.md:L38 | SURFACE): E/CAP | Objective builds `elai bench routing-economics`: fixed task corpus through two arms (cloud-only vs H-7 architect/editor hybrid), reporting % steps offloaded, $ delta, and quality delta as verified-OUTCOME rate (NOT pass@k), under ELAI_SEED and journal-recomputable (GD-2). | anchor=K-2:L7-L7
- **K-003/K2-E007** [SRC] (DEEP-K-003.discovery.md:L42 | SURFACE): E/DEP | Pre-requisites: H-7 calibrated routing + I-25 router (A-6 — re-grep cloud_mode.rs CloudModeRouter + providers escalation_policy.rs); bench dispatcher at args_bench.rs; safedelta_runner / bench_qdelta runner shape; J-28 UsageTurnEvent (SHIPPED, schema_version 272). | anchor=K-2:L10-L10, L20-L23
- **K-003/K2-E009** [SRC] (DEEP-K-003.discovery.md:L44 | SURFACE): E/STATUS | J-28 token ledger is SHIPPED (commit fe069d0; UsageTurnEvent at schema_version 272, usage_ledger + `elai usage` + SpendAccumulator); original KC-3 blocker (no normalized per-lane capture) is DISCHARGED at the capture layer. | anchor=K-2:L17-L17, L23-L23
- **K-003/K2-E010** [SRC] (DEEP-K-003.discovery.md:L45 | SURFACE): E/DEP | Remaining live-arm gate is infrastructural: live cloud lane priced off models.toml AND a local lane (RunPod/remote-sidecar, W-10) — neither funded in the K-26 window. | anchor=K-2:L17-L17
- **K-003/K2-E012** [SRC] (DEEP-K-003.discovery.md:L47 | SURFACE): E/INT | Bench dispatcher is args_bench.rs:52-93 (`if sub ==` chain); known subs include qdelta/safedelta/gate-tax/case-reuse; K-2's original `:75` anchor is STALE. | anchor=K-2:L22-L22, L71-L71
- **K-003/K2-E017** [SRC] (DEEP-K-003.discovery.md:L52 | SURFACE): E/SURFACE | CLI: dispatcher arm `if sub == "routing-economics"` in args_bench.rs:52-93 + bench_routing_economics.rs handler + register in unknown-sub error list. | anchor=K-2:L44-L46, L76-L76
- **K-003/K2-E018** [SRC] (DEEP-K-003.discovery.md:L53 | SURFACE): E/SURFACE | Production CLI verb: `elai bench routing-economics [--seed <u64>] [--json] [--force-cloud-everything]`. | anchor=K-2:L88-L88, L99-L99
- **K-003/K2-E021** [SRC] (DEEP-K-003.discovery.md:L56 | SURFACE): E/CFG | FORK-2: live lane funded this window? YES -> Measured real $; NO (K-26 default) -> fixture-arm-only + live arm 'cost unavailable' (RunnerPresent). | anchor=K-2:L66-L66
- **K-003/K2-E022** [SRC] (DEEP-K-003.discovery.md:L57 | SURFACE): E/SURFACE | FORK-3 (frontend, W-4): HF-vs-hybrid $/verified-task panel tile vs CLI-only; if still RunnerPresent defer tile (W-6 CLI-only default this window). | anchor=K-2:L67-L67, L96-L97
- **K-003/K2-E023** [SRC] (DEEP-K-003.discovery.md:L58 | SURFACE): E/TEST | Negative arm `--force-cloud-everything`: %offloaded==0 and savings delta ~0; hybrid arm >0 on same fixture. | anchor=K-2:L56-L58, L145-L145
- **K-003/K2-E028** [SRC] (DEEP-K-003.discovery.md:L63 | SURFACE): E/TEST | INTEGRATION_SMOKE: auditor drives real `elai bench routing-economics --seed 1234 --json --force-cloud-everything` then hybrid on full fixture; hybrid %offloaded strictly exceeds force-cloud; beacon rows; live cost 'cost unavailable'; same-seed byte-identical. Single-task happy-path does NOT satisfy R10. | anchor=K-2:L135-L136
- **K-003/K2-E031** [SRC] (DEEP-K-003.discovery.md:L66 | SURFACE): E/EVIDENCE | External figures HERA ~45.67% offload and CodeAct ~50%/-60% are [DESIGN]/[EXTERNAL]; ELAI delta is whatever the bench measures. | anchor=K-2:L15-L15, L156-L158
- **K-003/K2-A003** [SRC] (DEEP-K-003.discovery.md:L71 | SURFACE): A/SURFACE | FORK-3 leaves open whether an HF-vs-hybrid panel tile ships now or remains CLI-only. | anchor=K-2:L67-L67, L96-L97

### Surfaces — K-003/K21 (11)

- **K-003/K21-E006** [SRC] (DEEP-K-003.discovery.md:L221 | SURFACE): E/CFG | JC-3 RECONCILE-MISSING=FAIL + MONITOR-THE-MONITOR: observed sensitive effect with no matching journal row in bounded window = DETERMINISTIC FAIL; dropped-event counters FAIL CLOSED; observer-impaired is FIRST-CLASS; OBSERVE-ONLY first (GD-4, >=3 real runs). | anchor=K-21:L18-L18
- **K-003/K21-E007** [SRC] (DEEP-K-003.discovery.md:L222 | SURFACE): E/CFG | JC-4 HONEST-COVERAGE-SCOPE + PLATFORM-RESIDUE: honest claim is 'every COVERED effect class left a row'; host-trust ceiling (Rekor/TPM-IMA); observer LINUX-ONLY; Windows MEDIATION-ONLY; never advertise cloud observer guarantee on local lane (CL-4/GD-5). | anchor=K-21:L19-L19
- **K-003/K21-E008** [SRC] (DEEP-K-003.discovery.md:L223 | SURFACE): E/SURFACE | Author .claude/rules/journal-completeness.md (~60-75 lines) + conditional-routing trigger for journal-completeness / side-effect-mediation / kernel-observer / reconcile / shadow-stream / trust-the-journal-RRA-canvas tasks. | anchor=K-21:L41-L41
- **K-003/K21-E009** [SRC] (DEEP-K-003.discovery.md:L224 | SURFACE): E/DEP | Cross-link K-20 re-executable-RRA: re-executable is NOT complete — cassette faithfully reproduces incomplete record; trust-journal/canvas/derived-status claims carry covered-scope qualifier. | anchor=K-21:L42-L42
- **K-003/K21-E011** [SRC] (DEEP-K-003.discovery.md:L226 | SURFACE): E/TEST | NEGATIVE fixtures: (a) untagged complete/tamper-proof journal; (b) 'hash-chain proves completeness'; (c) reconcile SKIP/UNVERIFIED when observer absent; (d) Windows advertising Linux eBPF guarantee — lint fails (a)/(b), rule catches (c)/(d). | anchor=K-21:L44-L44
- **K-003/K21-E013** [SRC] (DEEP-K-003.discovery.md:L228 | SURFACE): E/SURFACE | frontend_pair N/A — doctrine + claims recogniser; E1 reconcile verdicts + C8-CANVAS shadow-activity anomaly nodes are J-track surfaces out-of-K. | anchor=K-21:L70-L71
- **K-003/K21-E015** [SRC] (DEEP-K-003.discovery.md:L230 | SURFACE): E/DEFER | OUT-OF-K E1 J sub-track: (a) MEDIATION SECCOMP_RET_USER_NOTIF; (b) kernel OBSERVER eBPF/LSM + normalization; (c) RECONCILE gate; (d) C8-CANVAS shadow-activity anomaly nodes; plus E2 runtime-observed DKG edges and E3 machine-derived-first audit lane. | anchor=K-21:L123-L123
- **K-003/K21-E016** [SRC] (DEEP-K-003.discovery.md:L231 | SURFACE): E/DATA | Files: CREATE journal-completeness.md, journal_completeness_negative.md, K-21-decisions.md; MODIFY conditional-routing.md, claims_policy_lint.py. | anchor=K-21:L105-L110
- **K-003/K21-E017** [SRC] (DEEP-K-003.discovery.md:L232 | SURFACE): E/TEST | k21 contract or Python: covered-scope-qualified passes; untagged complete/tamper-proof fails; hash-chain-proves-completeness fails; SKIP-on-missing-observer fails; Windows-eBPF-guarantee fails; empty fails-loud. | anchor=K-21:L102-L103
- **K-003/K21-A001** [SRC] (DEEP-K-003.discovery.md:L234 | SURFACE): A/CFG | Exact high-value effect-class enumeration beyond the four named categories and bounded-window duration for reconcile are left to E1 J authoring under the rule. | anchor=K-21:L17-L18, L123-L123
- **K-003/K21-I001** [SRC] (DEEP-K-003.discovery.md:L238 | SURFACE): I/DEP | Because K-20 and K-21 share claim surface, a 're-executable AND complete' claim needs BOTH tags — the plan requires coordinated lint shapes, not two forks. | anchor=K-21:L122-L122

### Surfaces — K-003/K24 (15)

- **K-003/K24-E001** [SRC] (DEEP-K-003.discovery.md:L248 | SURFACE): E/STATUS | K-24 is PLANNED (BCC wave; NET-NEW capability folded into K; multi-slice TRACK L1..L6 as ONE file; first commerce/billing surface; ~70% plumbing exists as J-28/J-49/C-5/H-9/J-81). | anchor=K-24:L3-L3
- **K-003/K24-E002** [SRC] (DEEP-K-003.discovery.md:L249 | SURFACE): E/CAP | Model: subscription price -> monthly CREDITS (provider $) -> credits spent per metered usage -> ELAI pays upstream -> ELAI controls rate limits, rollover, routing (Nous Portal / OpenRouter shape). | anchor=K-24:L5-L5
- **K-003/K24-E007** [SRC] (DEEP-K-003.discovery.md:L254 | SURFACE): E/SURFACE | L1 consumer: `elai billing balance` folds and prints; whole capability behind default-OFF ELAI_BILLING_ENABLE (allowlist type=experiment, promotion_plan_id=K-24-promotion). | anchor=K-24:L35-L35, L60-L60
- **K-003/K24-E009** [SRC] (DEEP-K-003.discovery.md:L256 | SURFACE): E/CAP | L3: OpenRouter MANAGED backend B-37 #12 hand-rolled Rust (no litellm, no new dep); returns actual cost per call booked as Settled via J-28; ELAI cost of credit-$ ~1.055x face (5.5% purchase fee). | anchor=K-24:L37-L37
- **K-003/K24-E011** [SRC] (DEEP-K-003.discovery.md:L258 | SURFACE): E/FLOW | L4: J-49 admission checks balance >= hold AND fair-share AND tier rate/concurrency; fail-closed -> queue or refuse with clear 'out of credits' surface. | anchor=K-24:L38-L38
- **K-003/K24-E012** [SRC] (DEEP-K-003.discovery.md:L259 | SURFACE): E/FLOW | L5: Payment (Stripe/Paddle) OUTSIDE ELAI (PCI scope zero); webhook subscription created/renewed/cancelled emits Granted(face+bonus) + Expired(over rollover cap); failed payment -> no Granted; chargeback -> AdjustmentManual(negative) + seat-suspension flag. | anchor=K-24:L39-L39
- **K-003/K24-E013** [SRC] (DEEP-K-003.discovery.md:L260 | SURFACE): E/CFG | Rollover cap and tier rate limits are seats.toml DATA per tier (W-2, no new flag per tier). | anchor=K-24:L38-L39
- **K-003/K24-E014** [SRC] (DEEP-K-003.discovery.md:L261 | SURFACE): E/TEST | L6: SAFEDELTA $/verified-task for ELAI-routed (local-heavy) vs raw-frontier; receipts on y-axis (GD-3); under ELAI_SEED; compose G-6 runner shape do NOT fork. | anchor=K-24:L40-L40
- **K-003/K24-E016** [SRC] (DEEP-K-003.discovery.md:L263 | SURFACE): E/SURFACE | Honest pricing wording: 'Plans include monthly credits for model and tool usage, metered at provider cost. ELAI routes work to the cheapest model that can verifiably do the job — often a local one — so your credits resolve more tasks per dollar.' NEVER 'unlimited Sonnet/Opus/GPT.' | anchor=K-24:L40-L40
- **K-003/K24-E017** [SRC] (DEEP-K-003.discovery.md:L264 | SURFACE): E/SURFACE | frontend_pair: credit balance + reserve/settle + tier rate limits on H-9 CloudPanel (or dedicated billing tile) as Svelte 5 runes + #[tauri::command] + generate_handler! + ts-rs; cross_lang_diff.py PASS (W-4). | anchor=K-24:L74-L75
- **K-003/K24-E018** [SRC] (DEEP-K-003.discovery.md:L265 | SURFACE): E/OBS | trigger: `elai billing balance` ; metered OpenRouter request ; renewal webhook fixture; expect CreditLedgerEvent rows hash-chained recomputable. | anchor=K-24:L76-L79
- **K-003/K24-E022** [SRC] (DEEP-K-003.discovery.md:L269 | SURFACE): E/DATA | Files: CREATE billing/{mod,credit_ledger,reserve_settle,refill,webhook}.rs, openrouter_backend.rs, billing_cmd, k24 contract, SAFEDELTA arm, K-24-decisions.md; MODIFY args.rs, ssb/action_type_lock, J-49 admission, J-28 booking, dashboard-v2, seats.toml, wire-dark-allowlist. | anchor=K-24:L117-L128
- **K-003/K24-E026** [SRC] (DEEP-K-003.discovery.md:L273 | SURFACE): E/DEP | GD-1 reuse list: J-28 meter, J-49 seat/quota, C-5/J-81 caps + runaway kill, H-9 $/resolved-task, J-54 free pool, J-55 modality router, B-37 backend pattern, I-51 catalog, F-4/I-18/J-69/J-70 audit journal — re-implemented NEVER; only credit ledger + managed rail + payment seam + pricing surface are new. | anchor=K-24:L5-L5, L15-L24
- **K-003/K24-E030** [SRC] (DEEP-K-003.discovery.md:L277 | SURFACE): E/CFG | W-2/R9: ELAI_BILLING_ENABLE default-OFF with wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-24-promotion, ON-path smoke) — not a terminal state. | anchor=K-24:L35-L35, L100-L100, L107-L108
- **K-003/K24-A002** [SRC] (DEEP-K-003.discovery.md:L281 | SURFACE): A/SURFACE | Exact billing tile vs ride H-9 CloudPanel decided at edit. | anchor=K-24:L74-L75

### Surfaces — K-003/K26 (17)

- **K-003/K26-E003** [SRC] (DEEP-K-003.discovery.md:L294 | SURFACE): E/CFG | DEADLINE: bmapi codex-proxy credits expire 2026-07-15; account-a ~5.4k/8k monthly bar (2026-07-10 dashboard); account-b second pool; every live-model leg burns those credits and ZERO Anthropic tokens. | anchor=K-26:L5-L5
- **K-003/K26-E004** [SRC] (DEEP-K-003.discovery.md:L295 | SURFACE): E/INT | gpt-5.5 is NATIVE ELAI provider: ProviderKind::Gpt55Proxy, detect_provider_kind("gpt-5.5"), auth BMAPI_API_KEY + BMAPI_BASE_URL (default https://bmapi.020212.xyz/v1). | anchor=K-26:L11-L11
- **K-003/K26-E006** [SRC] (DEEP-K-003.discovery.md:L297 | SURFACE): E/TEST | Auditor-authored live smoke crates/tools/tests/gpt55_proxy_live_smoke.rs (SP-3B): gated ELAI_PROVIDER_LIVE_SMOKE=1, #[ignore], drives execute_tool("Agent") -> real bmapi HTTP, asserts PromptHardeningApplied SSB row. | anchor=K-26:L13-L13
- **K-003/K26-E007** [SRC] (DEEP-K-003.discovery.md:L298 | SURFACE): E/LIMIT | CONTAMINATION GUARD: only ELAI's OWN runtime work enters ELAI journal; no TEIKOKU/GLM/harness task bridged into ssb.db. | anchor=K-26:L15-L15
- **K-003/K26-E008** [SRC] (DEEP-K-003.discovery.md:L299 | SURFACE): E/STATUS | RED-TEAM CORRECTION: `elai run` has NO model-selection seam; CliAction::Run no model field; --provider closed local | anchor=cloud
- **K-003/K26-E012** [SRC] (DEEP-K-003.discovery.md:L303 | SURFACE): E/CFG | BOX RULES: ONE cargo at a time ALWAYS -j 1; auditor sole test runner via PowerShell FOREGROUND; keys sourced per-run NEVER written to repo/logs; ONE commit per unit maintainer voice; every dogfood task REAL .elai_cc work never synthetic farm. | anchor=K-26:L22-L28
- **K-003/K26-E014** [SRC] (DEEP-K-003.discovery.md:L305 | SURFACE): E/FLOW | MOVE 1 finish J-28: cargo test -p elai-cli -j 1; memory-writer; FEATURE-MAP upsert; ONE commit of 20 J-28 files ONLY; auth_broker.rs SEPARATE tiny commit; push both. | anchor=K-26:L39-L44
- **K-003/K26-E015** [SRC] (DEEP-K-003.discovery.md:L306 | SURFACE): E/FLOW | MOVE 2 SP-3B live smoke on account-a with ELAI_PROVIDER_LIVE_SMOKE=1; exit 0 + PromptHardeningApplied row required. | anchor=K-26:L46-L51
- **K-003/K26-E016** [SRC] (DEEP-K-003.discovery.md:L307 | SURFACE): E/FLOW | MOVE 3 dogfood engine: real .elai_cc fixes through `elai run` for 5 days; FORK-5 Route A (widen B-27 validation for gpt-5.5, RECON-1 <=5 files) vs Route B (existing lane + per-call gpt-5.5 Agent subagents). | anchor=K-26:L53-L58
- **K-003/K26-E017** [SRC] (DEEP-K-003.discovery.md:L308 | SURFACE): E/DATA | Seed queue: (0 Route A only) gpt-5.5 seam; (1) usage_cmd print_json serde; (2) Dashboard panel RecurrenceHitRateMeasured; (3) Dashboard tile GateTaxMeasured; (4) startup_banner Windows tempdir race; (5) bench_convergence flake; (6) auth_broker SSB-emit writer=None at runtime_cmd.rs:395. | anchor=K-26:L59-L66
- **K-003/K26-E019** [SRC] (DEEP-K-003.discovery.md:L310 | SURFACE): E/FLOW | MOVE 4 PS-2: after >=5 dogfood tasks run `elai bench gate-tax --seed 1234 --json`; surface ceilings to user (FORK-1 user locks); --enforce-ceiling exits nonzero until locked. | anchor=K-26:L72-L79
- **K-003/K26-E020** [SRC] (DEEP-K-003.discovery.md:L311 | SURFACE): E/FLOW | MOVE 5 PS-5: after >=8-10 dogfood tasks with red>green, journal probe; FORK-2: >=10 distinct pairs -> BUILD `--suite journal`; <10 keep dogfooding, PS-5 stays RUNNER_PRESENT. | anchor=K-26:L81-L89
- **K-003/K26-E021** [SRC] (DEEP-K-003.discovery.md:L312 | SURFACE): E/SURFACE | PRODUCTION_HOOK for journal suite: crates/cli/src/bench_case_reuse.rs — existing `elai bench case-reuse --suite journal` rejection becomes real dispatch. | anchor=K-26:L86-L86
- **K-003/K26-E025** [SRC] (DEEP-K-003.discovery.md:L316 | SURFACE): E/FLOW | MOVE 8 surplus burn optional: (a) more dogfood; (b) J-28 R10 multi-turn/compaction smoke; (c) TEIKOKU codex-lane workers on L-1/L-2 (cap 2 attempts D-9; never touch ELAI journal). | anchor=K-26:L107-L110
- **K-003/K26-E026** [SRC] (DEEP-K-003.discovery.md:L317 | SURFACE): E/TEST | INTEGRATION_SMOKE: (1) after MOVE 2 SP-3B; (2) before DONE long multi-turn `elai run` with EvidenceGateEvaluated+TestOutcome+UsageTurnEvent + `elai usage` rollup + gate-tax recompute. | anchor=K-26:L112-L116
- **K-003/K26-E029** [SRC] (DEEP-K-003.discovery.md:L320 | SURFACE): E/EVIDENCE | REDTEAM 2026-07-10 non-author: ATTACK 1 SUCCEEDED PLAN-BLOCK (no elai run gpt-5.5 seam) -> FORK-5 patch; ATTACK 2 SUCCEEDED (table `actions` wrong -> `ssb`); ATTACKS 3-7 FAILED (plan survives). | anchor=K-26:L150-L157
- **K-003/K26-E030** [SRC] (DEEP-K-003.discovery.md:L321 | SURFACE): E/SURFACE | User-facing surfaces in seed queue: Dashboard panel for RecurrenceHitRateMeasured; Dashboard tile for GateTaxMeasured; CLI `elai usage`; CLI `elai bench gate-tax`; CLI `elai bench case-reuse --suite journal`; CLI `elai bench routing-economics`; CLI `elai run`. | anchor=K-26:L61-L64, L75-L75, L86-L86, L94-L95

### Surfaces — K-003/K27 (9)

- **K-003/K27-E001** [SRC] (DEEP-K-003.discovery.md:L334 | SURFACE): E/STATUS | K-27 authored 2026-07-11 (s-k26-gpt56), gate-checked + red-teamed; extends K-26 burn campaign; credit window 2 dies 2026-07-15. | anchor=K-27:L1-L7
- **K-003/K27-E002** [SRC] (DEEP-K-003.discovery.md:L335 | SURFACE): E/EVIDENCE | PROOF_SPINE feeds PS-3: builds L-1..L-9 in dependency order — blockers of L-10 `elai bench hallucination-delta` which OWNS PS-3. | anchor=K-27:L3-L3
- **K-003/K27-E003** [SRC] (DEEP-K-003.discovery.md:L336 | SURFACE): E/CFG | DEADLINE bmapi credits expire 2026-07-15; account-a ~2.3k left (dashboard credits-USED grew 5423→5712); account-b hard-429 on GET /v1/models (owner renew pending, FORK-1). | anchor=K-27:L5-L5
- **K-003/K27-E004** [SRC] (DEEP-K-003.discovery.md:L337 | SURFACE): E/INT | gpt-5.6-sol live on account-a across chat/completions 200, /v1/responses 200, codex CLI e2e 200; /v1/messages still 403; both codex config.toml pinned model+review_model=gpt-5.6-sol. | anchor=K-27:L11-L11
- **K-003/K27-E006** [SRC] (DEEP-K-003.discovery.md:L339 | SURFACE): E/SEC | Consequence: codex output is TAINTED DATA; codex never sits in controller seat; codex never executes on Windows host or /mnt/f; nothing codex produces lands without off-agent gates. | anchor=K-27:L12-L12
- **K-003/K27-E007** [SRC] (DEEP-K-003.discovery.md:L340 | SURFACE): E/CAP | PROVEN CONTAINMENT CHAIN (opus-orchestrator-mode): conductor/opus_walk/{opus_supervisor,opus_dispatch,opus_land,opus_preland_verify}.py on teikoku distro; opus_land.py --worktree (ext4), gates with cwd=<worktree>/.elai_cc, fail-closed scope-guard under .elai_cc/, frozen checkers, ff-only merge into /mnt/f + push, sha256-recorded. | anchor=K-27:L13-L13
- **K-003/K27-E013** [SRC] (DEEP-K-003.discovery.md:L346 | SURFACE): E/LIMIT | ELAI in-product routing does NOT accept gpt-5.6-* (providers/mod.rs:197 starts_with("gpt-5.5") only; fallback misroutes to Anthropic); irrelevant to codex CLI config but blocks in-product Agent-tool use of 5.6 until one-line widen ships. | anchor=K-27:L18-L18
- **K-003/K27-E017** [SRC] (DEEP-K-003.discovery.md:L350 | SURFACE): E/SEC | AI-4 CONTROLLER-PULLED PATCHES: controller extracts `git -C <worktree> diff` over wsl stdout; worker never given path/mount/command reaching Windows FS or teikoku distro. | anchor=K-27:L25-L25
- **K-003/K27-E018** [SRC] (DEEP-K-003.discovery.md:L351 | SURFACE): E/SEC | AI-5 PATCH SCREEN: deny-grep before leave quarantine for network egress libs/URLs, base64/hex >200, build.rs, proc-macro crates, .github/hooks/.gitattributes, paths escaping .elai_cc, include! absolute, unsafe blocks; ANY hit = FROZEN pending owner eyes-on (FORK-4). | anchor=K-27:L26-L26

### Surfaces — K-004/Ambiguity and conflict register (chunk-local) (1)

- **K-004/AC-006** [AC] (DEEP-K-004.discovery.md:L445 | SURFACE): AC | A | K-23 U4 before/after J-18; exact WORKSPACE altitude chrome | K23-A001-A002 | anchor=

### Surfaces — K-004/Cross-file interaction records (7)

- **K-004/IX-002** [IX] (DEEP-K-004.discovery.md:L344 | SURFACE): IX | K4-E038, K18a-E011 | K-4 4-file `gate-tax` wiring explicitly mirrors K-18a case-reuse bench CLI pattern. | K-4:L180-L183; K-18a:L76-L79 | anchor=
- **K-004/IX-005** [IX] (DEEP-K-004.discovery.md:L347 | SURFACE): IX | K11-E002/E009, K7-E003 | K-11 human evidence render + K-7 brief both sell buyer-side evidence; K-7 points at K-6 pack while K-11 extends K-3 PR surface (KD-2 chain). | K-11:L5-L6; K-7:L5-L10 | anchor=
- **K-004/IX-006** [IX] (DEEP-K-004.discovery.md:L348 | SURFACE): IX | K7-E008, K19-E008, K25-E001 | Multiple units extend/reuse K-1 claims_policy_lint (K-7 numbers; K-19 self-improving claims) without forking; K-25 is advisory-tool not lint but shares "never auto-apply" human-gate culture with K-7/K-8. | K-7:L47; K-19:L41; K-25:L12 | anchor=
- **K-004/IX-011** [IX] (DEEP-K-004.discovery.md:L353 | SURFACE): IX | K23-E025, K11-E004 | Both add dashboard frontend_pair surfaces (WORKSPACE altitude vs cockpit EvidenceArtifactSet) with cross_lang_diff. | K-23:L78-L79; K-11:L52-L53 | anchor=
- **K-004/IX-012** [IX] (DEEP-K-004.discovery.md:L354 | SURFACE): IX | K23-E007, K19-E005, K11-E008 | Flag polarity mix: K-23 default-OFF enable; K-19 mandates default-OFF for reuse-into-gen; K-11 default-ON kill-switch — all W-2/R9 disciplined. | K-23:L37; K-19:L18; K-11:L33 | anchor=
- **K-004/IX-016** [IX] (DEEP-K-004.discovery.md:L358 | SURFACE): IX | K4-E041, K18a-E017, K23-E020 | Shared R10 INTEGRATION_SMOKE discipline: real entrypoint, hard inputs, independent auditor — gate-tax, case-reuse, workspace merge. | K-4:L221-L227; K-18a:L133-L137; K-23:L41 | anchor=
- **K-004/IX-019** [IX] (DEEP-K-004.discovery.md:L361 | SURFACE): IX | K11-E019, K23-E021 | Human merge authority / worker git read-only: K-11 poll-and-surface only; K-23 member commits governed by each member's commit-cadence. | K-11:L102; K-23:L41 | anchor=

### Surfaces — K-004/K-11 deltas (1)

- **K-004/SD-K11-001** [SD] (DEEP-K-004.discovery.md:L400 | SURFACE): SD | **ADD** EvidenceArtifactRendered Kind set, LIVE ~267/577 W-9 note, exact kill-switch name, W-10 fixture vs live split. | 11-K:L273-L297 | K11-E008, K11-E011, K11-E014 | anchor=11-K:L273-L297

### Surfaces — K-004/K-18a deltas (1)

- **K-004/SD-K18a-001** [SD] (DEEP-K-004.discovery.md:L408 | SURFACE): SD | **ADD** entire unit (summary A-05 unindexed). Full inventory of observe-arm scope, fixtures, CLI, forks, aborts, verifies, red-team, deferred K-1/VCG legs. | 11-K:L23, L896 | K18a-E001-E022, K18a-C001 | anchor=11-K:L23, L896

### Surfaces — K-004/K-25 deltas (1)

- **K-004/SD-K25-001** [SD] (DEEP-K-004.discovery.md:L432 | SURFACE): SD | **ADD** AT_RISK_STALE verdict class, exact --help unsoundness declaration, elai_status_query pattern mirror, watch-set creep ban. | 11-K:L719-L741 | K25-E007, K25-E009, K25-E015, K25-E021 | anchor=11-K:L719-L741

### Surfaces — K-004/K-4 deltas (2)

- **K-004/SD-K4-002** [SD] (DEEP-K-004.discovery.md:L373 | SURFACE): SD | **CORRECT** summary K-4.11 wire-dark-allowlist 105-row gate inventory: source FIX-1 bans that registry as scope source (it is default-OFF exemptions, 135 rows). | 11-K:L142 | K4-E016, K4-C001 | anchor=11-K:L142
- **K-004/SD-K4-005** [SD] (DEEP-K-004.discovery.md:L376 | SURFACE): SD | **CORRECT** summary "release gate" as implied Rust artifact: source ATTACK-4 makes breach the CLI non-zero EXIT CODE only. | 11-K:L137, L146 | K4-E039, K4-E034 | anchor=11-K:L137, L146

### Surfaces — K-004/K11 (18)

- **K-004/K11-E002** [SRC] (DEEP-K-004.discovery.md:L163 | SURFACE): E/CAP | Extends K-3 PR/CI emitter with a human-viewable EVIDENCE-ARTIFACT section: screenshots / logs / RRA-summary / test-evidence into PR body + G-5 reviewer cockpit, risk-tiered by G-5 DSR. | anchor=K-11:L5-L6, L30-L33
- **K-004/K11-E003** [SRC] (DEEP-K-004.discovery.md:L164 | SURFACE): E/SURFACE | Markdown `## Evidence` section: RRA summary + per-test evidence links + collapsible logs + screenshot refs, posted via `post_review_comment`. | anchor=K-11:L32-L32, L54-L57
- **K-004/K11-E004** [SRC] (DEEP-K-004.discovery.md:L165 | SURFACE): E/SURFACE | Same EvidenceArtifactSet view in G-5 reviewer cockpit via Tauri read; cross_lang_diff.py PASS (frontend_pair). | anchor=K-11:L52-L53, L78-L78, L91-L91
- **K-004/K11-E005** [SRC] (DEEP-K-004.discovery.md:L166 | SURFACE): E/DATA | EvidenceArtifactSet assembler (~80 LOC) collects existing artifacts only: G-2 RRA digest, F-4 TestOutcomePayload stdout_hash rows, G-5 DSR + vouched/unvouched bands, H-52 anchored findings, visual-ui-loop screenshot refs. | anchor=K-11:L31-L31, L37-L38, L41-L42
- **K-004/K11-E008** [SRC] (DEEP-K-004.discovery.md:L169 | SURFACE): E/SURFACE | CLI flag `elai pr-review --emit-evidence` (default-ON) behind kill-switch `ELAI_PR_EVIDENCE_RENDER_DISABLE` (W-2; kill-switch not enable-flag). | anchor=K-11:L33-L33, L39-L39, L71-L71, L76-L76
- **K-004/K11-E009** [SRC] (DEEP-K-004.discovery.md:L170 | SURFACE): E/INT | Wire into K-3 emit path (~20 LOC): check-run posting calls K-11 renderer; evidence section ships with per-hunk verdicts in SAME emit. | anchor=K-11:L33-L33, L44-L45
- **K-004/K11-E010** [SRC] (DEEP-K-004.discovery.md:L171 | SURFACE): E/CAP | Module `crates/orchestrator/src/github_pr/evidence_render.rs` (~150 LOC under .elai_cc path in Files touched). | anchor=K-11:L37-L38, L89-L89
- **K-004/K11-E011** [SRC] (DEEP-K-004.discovery.md:L172 | SURFACE): E/EVENT | prefer-reuse github_pr / G-5 review_surface rows; else +1 EvidenceArtifactRendered (Pattern-2; Kind: SectionRendered / HunkVouched / HunkUnvouched); W-9 from LIVE baseline ~267/577. | anchor=K-11:L56-L56, L82-L82
- **K-004/K11-E012** [SRC] (DEEP-K-004.discovery.md:L173 | SURFACE): E/DEP | Consumes (never re-implements): post_review_comment / ReviewCommentKind; rra build/verify; review_surface; review_anchor; TestOutcomePayload; .claude/tools/visual/ screenshots. | anchor=K-11:L7-L8, L41-L42, L48-L49, L70-L70
- **K-004/K11-E014** [SRC] (DEEP-K-004.discovery.md:L175 | SURFACE): E/CFG | W-10 split: deterministic render fixture now; live check-run post network/RunPod-gated separately. | anchor=K-11:L7-L7, L55-L55, L79-L79
- **K-004/K11-E018** [SRC] (DEEP-K-004.discovery.md:L179 | SURFACE): E/CFG | KD-2: evidence renders into the PR; cockpit is DEEP view never only view. | anchor=K-11:L10-L10, L68-L68
- **K-004/K11-E019** [SRC] (DEEP-K-004.discovery.md:L180 | SURFACE): E/DEFER | DOCTRINE-TENSION recorded NOT in K-11 scope: no-mistakes PR-babysit-to-merge vs one-commit-at-end + worker-git-read-only → wedge slice is poll-CI + surface, human merges, NO autonomous force-push. | anchor=K-11:L102-L102
- **K-004/K11-E020** [SRC] (DEEP-K-004.discovery.md:L181 | SURFACE): E/DEFER | lavish overlapping-text elementFromPoint occlusion probe is a separate rider to visual-ui-loop.md, not K-11. | anchor=K-11:L103-L103
- **K-004/K11-E021** [SRC] (DEEP-K-004.discovery.md:L182 | SURFACE): E/LIMIT | GitHub check-run/PR-comment size limits: render refs/links not embedded blobs; decide against live API limit via WebSearch (KD-1). | anchor=K-11:L101-L101
- **K-004/K11-E022** [SRC] (DEEP-K-004.discovery.md:L183 | SURFACE): E/EVIDENCE | Grounding: steal-kunchen C2.2 EXTENSION(2) no-mistakes ## Testing/Evidence; C3.3 META "eng-manager not diff-reviewer" is AUTHOR/market-validation claim not ELAI number (HD-1/ID-5). | anchor=K-11:L5-L5, L96-L96
- **K-004/K11-E023** [SRC] (DEEP-K-004.discovery.md:L184 | SURFACE): E/DATA | Creates workflow/reasoning/K-11-decisions.md; modifies CLI pr-review + dashboard-v2 cockpit evidence view. | anchor=K-11:L34-L34, L88-L93
- **K-004/K11-E024** [SRC] (DEEP-K-004.discovery.md:L185 | SURFACE): E/INT | Production paths: `elai pr-review --emit-evidence` → github_pr post; K-3 check-run path → EvidenceArtifactSet → post_review_comment. | anchor=K-11:L43-L46
- **K-004/K11-A002** [SRC] (DEEP-K-004.discovery.md:L188 | SURFACE): A/SURFACE | Exact collapsible-log markdown mechanics and GitHub size cutoffs decided at authoring. | anchor=K-11:L32-L32, L101-L101

### Surfaces — K-004/K18a (9)

- **K-004/K18a-E003** [SRC] (DEEP-K-004.discovery.md:L200 | SURFACE): E/LIMIT | OUT: Task 2 reuse economics (VCG J-track unbuilt J-95..J-98); Tasks 3/4/5 dossier+claims-lint (claims_policy_lint.py and ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md DO NOT EXIST 2026-07-08 — would silently AUTHOR K-1); live local-lane (W-10). | anchor=K-18a:L17-L24
- **K-004/K18a-E008** [SRC] (DEEP-K-004.discovery.md:L205 | SURFACE): E/FLOW | RECON R-D: SSB row reuse prefer existing bench family +Kind; net 0..+1. | anchor=K-18a:L43-L45, L118-L119
- **K-004/K18a-E010** [SRC] (DEEP-K-004.discovery.md:L207 | SURFACE): E/CAP | case_reuse_runner.rs arm-1 only: query retrieval with fresh failure signature; report recurrence_hit_rate (top-k contains proven case) AND false_match_rate (distractors) per k in {1,3,5} under ELAI_SEED; ZERO prompt-assembly/reuse-into-generation imports. | anchor=K-18a:L65-L71
- **K-004/K18a-E011** [SRC] (DEEP-K-004.discovery.md:L208 | SURFACE): E/SURFACE | CLI: `elai bench case-reuse --suite fixture --seed 1234 --json` (and optionally `--suite journal` if F-2); mirror parse_bench_safedelta_args at :896; sub must appear in unknown-sub error string at :87. | anchor=K-18a:L76-L84, L155-L156
- **K-004/K18a-E012** [SRC] (DEEP-K-004.discovery.md:L209 | SURFACE): E/CFG | FORK F-1 settled: claims_policy_lint absent → PACKAGE legs OUT. FORK F-2: R-A <10 → FIXTURE-only; >=10 add journal suite. FORK F-3: no convergence → safedelta_runner shape. FORK F-4: ONNX unstaged → retrieval_mode="bm25_only"; staged → "hybrid"; never fabricate hybrid from BM25. | anchor=K-18a:L101-L110
- **K-004/K18a-E014** [SRC] (DEEP-K-004.discovery.md:L211 | SURFACE): E/TEST | Same-seed byte-identity (ELAI_SEED=1234 twice); different-seed 5678 valid-and-different (anti-hardcoding); empty corpus fails-loud (R1). | anchor=K-18a:L86-L90, L122-L124, L157-L158
- **K-004/K18a-E015** [SRC] (DEEP-K-004.discovery.md:L212 | SURFACE): E/TEST | NEGATIVE: `--match-everything` debug flag (cfg(test)-only) collapses false_match_rate to ~1.0 proving metric discriminates. | anchor=K-18a:L88-L89, L125-L126, L140-L143
- **K-004/K18a-E017** [SRC] (DEEP-K-004.discovery.md:L214 | SURFACE): E/TEST | VERIFY off-agent: wiring_contract_check --run-trigger; production_caller_check no DARK; maintainability_check; scoped cargo test + cargo check; INTEGRATION_SMOKE drives real entrypoint seeds 1234/5678 + negative arm (R10). | anchor=K-18a:L121-L137
- **K-004/K18a-I001** [SRC] (DEEP-K-004.discovery.md:L220 | SURFACE): I/DEP | K-18a is the runnable subset that later K-4 cites as the proven 4-file CLI template (case_reuse_runner + bench_case_reuse). | anchor=K-18a:L15-L16; (cross-file via K-4)

### Surfaces — K-004/K19 (8)

- **K-004/K19-E005** [SRC] (DEEP-K-004.discovery.md:L237 | SURFACE): E/CFG | SF-3 OBSERVE-FIRST + NEVER-INTO-THE-AUDITOR: reuse-into-GENERATION is OBSERVE-ONLY first (GD-4), DEFAULT-OFF with wire-dark-allowlist + promotion plan (W-2/R9), NEVER injected into auditor/critic/adversary (H-31 firewall). | anchor=K-19:L18-L18, L63-L63
- **K-004/K19-E008** [SRC] (DEEP-K-004.discovery.md:L240 | SURFACE): E/CAP | EXTEND (not fork) K-1 claims_policy_lint recogniser: "self-improving / learns from your codebase / gets better with use / gets smarter over time" FAILS unless execution-grounded-fitness correction + [DESIGN]/[BENCH-PENDING on K-18] tag; "fitness/reward by byte-hash or LLM-judged-helpful" design FAILS. | anchor=K-19:L41-L41, L49-L49, L88-L88
- **K-004/K19-E009** [SRC] (DEEP-K-004.discovery.md:L241 | SURFACE): E/SURFACE | Negative fixture `docs/thefinalglance/K-plan/_fixtures/self_improving_negative.md`: untagged "learns from your code"; fitness by retrieval/LLM-helpful; case injected into AUDITOR — lint fails (a), rule catches (b)/(c). | anchor=K-19:L42-L42, L79-L79, L105-L105
- **K-004/K19-E013** [SRC] (DEEP-K-004.discovery.md:L245 | SURFACE): E/INT | Production: conditional-routing load (JD-1); auditor runs claims_policy_lint; VCG executor + K-18 reuse arm MUST obey. | anchor=K-19:L55-L58
- **K-004/K19-E014** [SRC] (DEEP-K-004.discovery.md:L246 | SURFACE): E/SURFACE | frontend_pair N/A (W-6 justified doctrine + recogniser). | anchor=K-19:L66-L67
- **K-004/K19-E017** [SRC] (DEEP-K-004.discovery.md:L249 | SURFACE): E/DATA | Creates self-improvement-fitness.md (~70 LOC), self_improving_negative.md, K-19-decisions.md; MODIFY conditional-routing.md (~2 LOC), claims_policy_lint.py (~12 LOC). | anchor=K-19:L101-L106
- **K-004/K19-E018** [SRC] (DEEP-K-004.discovery.md:L250 | SURFACE): E/EVIDENCE | Paper grounding [EXTERNAL not ELAI numbers]: Misevolve 2509.26354 refusal collapse; DGM 2505.22954; MPBench 2606.04329; auditor-memory-firewall 2510.09905; JudgeBench. | anchor=K-19:L111-L111
- **K-004/K19-E019** [SRC] (DEEP-K-004.discovery.md:L251 | SURFACE): E/DEFER | OUT-OF-K technical companion (VCG J-track full list in K-18 OUT-OF-K): mint-on-F-4 case node, deterministic edges, MEMORY/CASE canvas altitude, gated reuse loop, episodic→skill→rule ladder (I-12→C-25→B-23 L3) — all BOUND by SF-1..SF-4. | anchor=K-19:L118-L118

### Surfaces — K-004/K23 (17)

- **K-004/K23-E006** [SRC] (DEEP-K-004.discovery.md:L269 | SURFACE): E/CAP | U1 WORKSPACE OBJECT: workspace.toml (member project_roots + relationship hints produces-for\ | anchor=consumes-from\
- **K-004/K23-E007** [SRC] (DEEP-K-004.discovery.md:L270 | SURFACE): E/CFG | Capability behind ONE default-OFF `ELAI_WORKSPACE_ENABLE` with wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-23-promotion, ON-path smoke). | anchor=K-23:L37-L37, L103-L103, L110-L110
- **K-004/K23-E010** [SRC] (DEEP-K-004.discovery.md:L273 | SURFACE): E/SEC | SOUND edges may drive gates; HEURISTIC edges surface with warning badge, NEVER asserted as call edge (R5/R6 — wrong edge worse than no edge). | anchor=K-23:L38-L38, L73-L73, L99-L99
- **K-004/K23-E011** [SRC] (DEEP-K-004.discovery.md:L274 | SURFACE): E/SURFACE | `elai workspace map` renders Sound vs Heuristic edges; fixture expects exactly ONE Sound types←api edge, ZERO false edges. | anchor=K-23:L38-L38, L83-L83
- **K-004/K23-E012** [SRC] (DEEP-K-004.discovery.md:L275 | SURFACE): E/CAP | U3 FEDERATED RETRIEVAL: `elai workspace search <id> "<q>"` fans query to EACH member pipeline in parallel; RRF-merge k=60; every hit carries per-repo provenance; NO new index/embedding store. | anchor=K-23:L39-L39
- **K-004/K23-E014** [SRC] (DEEP-K-004.discovery.md:L277 | SURFACE): E/CAP | U4 CROSS-REPO BLAST RADIUS: extend B-29 cone + J-16 gate across Sound cross_repo_edges; HEURISTIC contributes "possible consumer, unverified" advisory only. CLI `elai workspace impact <id> <repo>:<symbol>`. | anchor=K-23:L40-L40
- **K-004/K23-E016** [SRC] (DEEP-K-004.discovery.md:L279 | SURFACE): E/CAP | U5 `workspace orchestrate`: lightweight workspace FSM above per-root FSM. | anchor=K-23:L41-L41
- **K-004/K23-E021** [SRC] (DEEP-K-004.discovery.md:L284 | SURFACE): E/LIMIT | Controller roots at WORKSPACE synthetic root, never edits member code (main-llm-role); worker git read-only; orchestrate 4 tripwires at workspace grain. | anchor=K-23:L41-L41
- **K-004/K23-E024** [SRC] (DEEP-K-004.discovery.md:L287 | SURFACE): E/PRIV | Workspace belongs to a J-59 seat; cross-seat workspace sharing OUT of scope (same later opt-in as cross-seat memory). | anchor=K-23:L42-L42, L75-L75
- **K-004/K23-E025** [SRC] (DEEP-K-004.discovery.md:L288 | SURFACE): E/SURFACE | canvas WORKSPACE altitude beside J-31/J-60/J-98: nodes=member repos, edges=cross_repo_edges (colored Sound vs Heuristic), in-flight delegation lights member node, workspace-session+PRs temporal layer; re-derived byte-identically from workspace.ssb.db. | anchor=K-23:L42-L42, L78-L79
- **K-004/K23-E027** [SRC] (DEEP-K-004.discovery.md:L290 | SURFACE): E/SURFACE | CLI verbs: `elai workspace create\ | anchor=add\
- **K-004/K23-E028** [SRC] (DEEP-K-004.discovery.md:L291 | SURFACE): E/DATA | Module tree workspace/{mod,manifest,dep_graph,federated_search,cross_repo_cone,workspace_orchestrate,episodic}.rs (~1100 LOC); CLI workspace_cmd.rs; blast_radius.rs +40 LOC; dashboard WORKSPACE altitude ~120 LOC. | anchor=K-23:L120-L125
- **K-004/K23-E032** [SRC] (DEEP-K-004.discovery.md:L295 | SURFACE): E/DEFER | Until J-18 lands, U2/U4 edges Heuristic-only advisory and impact observe-only; Sound/Heuristic split authored day one for flag-flip promotion. | anchor=K-23:L137-L137
- **K-004/K23-E033** [SRC] (DEEP-K-004.discovery.md:L296 | SURFACE): E/LIMIT | v1 caps workspace at small member set (common 2-4 repo case); large-org federation later rider. | anchor=K-23:L138-L138
- **K-004/K23-E034** [SRC] (DEEP-K-004.discovery.md:L297 | SURFACE): E/LIMIT | Workspace controller must never touch a member's FSM — polls J-2/H-36 only. | anchor=K-23:L139-L139
- **K-004/K23-E036** [SRC] (DEEP-K-004.discovery.md:L299 | SURFACE): E/EVIDENCE | Grounding: multi-project-memory-analysis-2026-06-26.md §7.0..§7.12; Polygraph [EXTERNAL] SPATIAL+TEMPORAL axes; ELAI reuses listed substrate. | anchor=K-23:L10-L10, L130-L134
- **K-004/K23-A002** [SRC] (DEEP-K-004.discovery.md:L302 | SURFACE): A/SURFACE | Exact drawer/window for canvas WORKSPACE altitude decided at edit (rides J-31/J-60/J-98 pattern). | anchor=K-23:L78-L79

### Surfaces — K-004/K25 (12)

- **K-004/K25-E003** [SRC] (DEEP-K-004.discovery.md:L314 | SURFACE): E/CAP | Continuous watch: (a) dated file-based provider-surface snapshot; (b) deterministic overlap checker vs feature-map ABSORBED/AT-RISK/DURABLE tags; (c) staleness trigger on K-8's conditional-routing lane so efficiency-class task on stale snapshot must refresh before EXECUTING. | anchor=K-25:L11-L17
- **K-004/K25-E004** [SRC] (DEEP-K-004.discovery.md:L315 | SURFACE): E/DATA | Snapshot `workflow/status/provider-surface.json` schema v1: one row per watched capability {provider, capability_key, status: NATIVE\ | anchor=PARTIAL\
- **K-004/K25-E006** [SRC] (DEEP-K-004.discovery.md:L317 | SURFACE): E/SURFACE | Tool `.claude/tools/absorption_watch.py` (stdlib-only, offline, --json, --max-age-days N default 90). | anchor=K-25:L63-L66, L81-L81, L119-L119
- **K-004/K25-E008** [SRC] (DEEP-K-004.discovery.md:L319 | SURFACE): E/FAIL | Exit codes: 0 clean / 1 overlaps found / 2 config-or-parse error fail-closed / 3 snapshot stale past --max-age-days. | anchor=K-25:L66-L66, L112-L113, L119-L119
- **K-004/K25-E009** [SRC] (DEEP-K-004.discovery.md:L320 | SURFACE): E/LIMIT | Unit-to-capability join is keyword/tag-based and declared UNSOUND in --help (advisory, R5-style honesty); LLM adjudicates each punch row. | anchor=K-25:L67-L68, L150-L152
- **K-004/K25-E010** [SRC] (DEEP-K-004.discovery.md:L321 | SURFACE): E/SURFACE | Rule `.claude/rules/absorption-watch.md` (~15 lines): on efficiency-class task run absorption_watch.py FIRST; exit 3 = refresh snapshot (WebSearch, re-date) BEFORE EXECUTING; exit 1 = surface punch rows as kill/de-scope PROPOSAL. | anchor=K-25:L69-L72, L82-L82
- **K-004/K25-E013** [SRC] (DEEP-K-004.discovery.md:L324 | SURFACE): E/LIMIT | NO done-gate consumption of checker verdict (advisory cache; R8 untouched); mirror elai_status_query R8 note. | anchor=K-25:L74-L75, L94-L94, L121-L121
- **K-004/K25-E015** [SRC] (DEEP-K-004.discovery.md:L326 | SURFACE): E/DEP | Extends K-8 tags + provider-absorption-pregate.md; composes elai_status_query.py advisory-cache pattern; sibling to K-9 model-refresh axis (disjoint); G-10 is runtime exogenous watcher — share nothing. | anchor=K-25:L21-L23, L40-L49, L84-L86
- **K-004/K25-E016** [SRC] (DEEP-K-004.discovery.md:L327 | SURFACE): E/SURFACE | frontend_pair N/A — harness tool + rule + status file (W-6). | anchor=K-25:L96-L97
- **K-004/K25-E019** [SRC] (DEEP-K-004.discovery.md:L330 | SURFACE): E/DATA | Creates provider-surface.json, absorption_watch.py (~150 LOC), absorption-watch.md, K-25-decisions.md; MODIFY conditional-routing.md (~1 LOC). | anchor=K-25:L135-L140
- **K-004/K25-E021** [SRC] (DEEP-K-004.discovery.md:L332 | SURFACE): E/LIMIT | Watch-set creep: cap snapshot to capabilities that overlap an existing ELAI unit; general provider-news feed out of scope. | anchor=K-25:L155-L156
- **K-004/K25-E022** [SRC] (DEEP-K-004.discovery.md:L333 | SURFACE): E/EVIDENCE | Grounding: spec-eval 2026-07-05 §4 L2 / §5 ADD-2; K-8; KS-3 report 17 [EXTERNAL]; elai_status_query/elai-status-monitor pattern. | anchor=K-25:L142-L147

### Surfaces — K-004/K4 (18)

- **K-004/K4-E003** [SRC] (DEEP-K-004.discovery.md:L37 | SURFACE): E/CAP | Builds `elai bench gate-tax`: a per-release friction report answering what the trust spine costs on a normal coding task, against a LOCKED ceiling; ceiling breach yields non-zero CLI exit for the release path; report names top-3 friction contributors. | anchor=K-4:L7-L12, L180-L188
- **K-004/K4-E011** [SRC] (DEEP-K-004.discovery.md:L45 | SURFACE): E/DATA | DOSSIER ROW (KD-1): "composed gate-tax on coding tasks (in-band runtime set; audit tax reported separately; trading surface unmeasured)". | anchor=K-4:L36-L37
- **K-004/K4-E013** [SRC] (DEEP-K-004.discovery.md:L47 | SURFACE): E/DEFER | A `cfg(bench)`-gated whole-pipeline falsifier MAY exist LATER as one-shot validation of the cost model — compiled out of release, NEVER a runtime flag, NEVER the primary instrument. | anchor=K-4:L42-L43, L370-L372
- **K-004/K4-E014** [SRC] (DEEP-K-004.discovery.md:L48 | SURFACE): E/DEP | Prerequisites: case_reuse_runner.rs + bench_case_reuse.rs (proven runner + 4-file CLI template), args_bench.rs dispatcher, EvidenceGateEvaluated (action_type.rs:308), HookEvent (:87), TieredAnalysis.elapsed_us, gate-manifest.txt. | anchor=K-4:L48-L50, L60-L68
- **K-004/K4-E016** [SRC] (DEEP-K-004.discovery.md:L50 | SURFACE): E/STATUS | FIX-1: gate set is NOT sourced from wire-dark-allowlist.toml (that is the default-OFF EXEMPTION registry, 135 rows); in-scope is the IN-BAND RUNTIME CODING gates with RIDER-b enumerating out-of-scope. | anchor=K-4:L71-L75, L247-L247, L340-L341
- **K-004/K4-E018** [SRC] (DEEP-K-004.discovery.md:L52 | SURFACE): E/LIMIT | FIX-3: safedelta_runner.rs ON/OFF idiom is STATIC before/after split with no wall-clock number; Leg A uses journal elapsed_us OR a bench microbench, not that idiom. | anchor=K-4:L78-L79, L251-L251
- **K-004/K4-E025** [SRC] (DEEP-K-004.discovery.md:L59 | SURFACE): E/OBS | RECON_NEEDED-5: time offline checkers via Command wall-clock over K reps (median); label "observed, unbounded" unless ceiling set. | anchor=K-4:L116-L119
- **K-004/K4-E026** [SRC] (DEEP-K-004.discovery.md:L60 | SURFACE): E/FLOW | FORK-1 Leg-A per-gate-family: hook-tier → Route A (journal-native median elapsed_us); FSM EvidenceGate family → Route B (bench microharness K seeded reps median, reuse gate_self_validation.rs adapter; excluded gates NAMED as residue). | anchor=K-4:L122-L126
- **K-004/K4-E028** [SRC] (DEEP-K-004.discovery.md:L62 | SURFACE): E/DATA | GateTaxMeasured payload fields: coding_task_gate_tax_ms per named class, audit_gate_tax_ms, audit_unbounded bool, over_block_rate, coding_ceiling_ms, coding_breached bool, audit_ceiling_ms, top-3 contributor gate_ids + costs, coverage counts (n_in_scope_coding, n_trading_surface, n_offline_audit, n_dormant, n_untagged==0), n_gates, n_fires. | anchor=K-4:L190-L194
- **K-004/K4-E035** [SRC] (DEEP-K-004.discovery.md:L69 | SURFACE): E/FAIL | ABORT-5: if coding ceiling is CEILING-PENDING and breach exit is reachable from real release path, bench MUST exit non-zero (fail-CLOSED); label "release blocked pending ceiling confirmation"; WIRE-DARK only for truly unwired period. | anchor=K-4:L146-L148, L362-L363
- **K-004/K4-E038** [SRC] (DEEP-K-004.discovery.md:L72 | SURFACE): E/SURFACE | CLI surface: `elai bench gate-tax [--seed <u64>] [--json]` via 4-file wiring (args_bench.rs arm + bench_gate_tax.rs + main.rs dispatch + args.rs variant), K-18a pattern. | anchor=K-4:L180-L186, L214-L217, L241-L243
- **K-004/K4-E039** [SRC] (DEEP-K-004.discovery.md:L73 | SURFACE): E/FAIL | coding_task_gate_tax OR over_block_rate above coding ceiling → non-zero process exit (the release-gate is the CLI exit code, NOT a fictional Rust release_gate fn). | anchor=K-4:L171-L178, L200-L205, L213-L217, L360-L361
- **K-004/K4-E040** [SRC] (DEEP-K-004.discovery.md:L74 | SURFACE): E/INT | Sole producer/consumer pair: run_gate_tax → bench_gate_tax.rs::run from real `elai bench gate-tax`; #[cfg(test)]/contracts never satisfy R7; production_caller_check 0 DARK. | anchor=K-4:L213-L219, L299-L299
- **K-004/K4-E043** [SRC] (DEEP-K-004.discovery.md:L77 | SURFACE): E/LIMIT | Stub bans: no invented ceiling; no wire-dark-allowlist gate set; no summing task+audit numbers; no untagged coverage; no product bypass; no GateOverBlocked as rate; no warn-only breach; no MEASURED on empty journal; no .unwrap on prod. | anchor=K-4:L245-L253
- **K-004/K4-I001** [SRC] (DEEP-K-004.discovery.md:L83 | SURFACE): I/SURFACE | Optional friction tile on cost/audit surface is not mandated in the refined acceptance; CLI JSON + GateTaxMeasured row are the load-bearing operator surfaces. | anchor=K-4:L254-L258, L286-L300
- **K-004/K4-A002** [SRC] (DEEP-K-004.discovery.md:L85 | SURFACE): A/DATA | Whether FORK-2 reuses an existing bench row family or always creates GateTaxMeasured defaults to NEW but is re-grepped at edit. | anchor=K-4:L127-L129
- **K-004/K4-A003** [SRC] (DEEP-K-004.discovery.md:L86 | SURFACE): A/TEST | Exact N for benign corpus size, K microbench reps, and seed set for Route-B are not fixed beyond "K seeded reps" and "N known-good". | anchor=K-4:L113-L115, L122-L126, L277-L278
- **K-004/K4-C001** [SRC] (DEEP-K-004.discovery.md:L87 | SURFACE): C/CAP | Summary-layer K-4 still describes ON/OFF ablation, wire-dark 105-row inventory, whole-system over-block, and args_bench.rs:75; refined source VETOES product ON/OFF, bans wire-dark inventory, splits two numbers + RIDER-b, and uses CLI exit-code contract without a line-:75 claim. | anchor=K-4:L14-L43, L71-L82, L171-L178

### Surfaces — K-004/K5 (4)

- **K-004/K5-E003** [SRC] (DEEP-K-004.discovery.md:L99 | SURFACE): E/CFG | Always-on cheap legs: E-30 argv allowlist, F-1 compound gate, F-9 dep gate, G-1 taint. | anchor=K-5:L7-L9, L35-L35, L51-L53
- **K-004/K5-E005** [SRC] (DEEP-K-004.discovery.md:L101 | SURFACE): E/DATA | Risk-signal vector: J-16 blast-radius cone size, F-2 CWE-surface class, B-22 autonomy level, D-15 novelty, file criticality (DKG/B-29 centrality), plus C-30 complexity classes. | anchor=K-5:L9-L11, L32-L37, L44-L45
- **K-004/K5-E013** [SRC] (DEEP-K-004.discovery.md:L109 | SURFACE): E/SURFACE | frontend_pair: tier-decision rows on the audit-timeline surface (decide at edit, W-4) OR W-6-justified absence. | anchor=K-5:L86-L87
- **K-004/K5-A003** [SRC] (DEEP-K-004.discovery.md:L122 | SURFACE): A/SURFACE | Exact audit-timeline UI presentation of tier decisions is deferred (W-4 or W-6). | anchor=K-5:L86-L87

### Surfaces — K-004/K7 (5)

- **K-004/K7-E002** [SRC] (DEEP-K-004.discovery.md:L133 | SURFACE): E/SURFACE | Authors pure-prose brief `docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md` (~120-180 lines / ~150 LOC) structured as (problem-with-a-number) → (ELAI control) → (compliance-export evidence). | anchor=K-7:L5-L10, L43-L46, L52-L52, L98-L98
- **K-004/K7-E008** [SRC] (DEEP-K-004.discovery.md:L139 | SURFACE): E/CFG | Every number tagged measured-vs-external (KD-1); brief must PASS `claims_policy_lint.py --files docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md`. | anchor=K-7:L9-L10, L47-L47, L68-L71, L83-L83
- **K-004/K7-E013** [SRC] (DEEP-K-004.discovery.md:L144 | SURFACE): E/INT | Production audit/lint gate is claims_policy_lint over the brief; #[cfg(test)] never satisfies R7. | anchor=K-7:L57-L59
- **K-004/K7-E014** [SRC] (DEEP-K-004.discovery.md:L145 | SURFACE): E/SURFACE | frontend_pair N/A — positioning doc; W-6 justified. | anchor=K-7:L65-L66
- **K-004/K7-A001** [SRC] (DEEP-K-004.discovery.md:L152 | SURFACE): A/SURFACE | Exact section titles and insurance-questionnaire template mapping beyond the problem→control→evidence shape are left to authoring. | anchor=K-7:L43-L46

### Surfaces — SUMMARY/A-06 (1)

- **SUMMARY/A-06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L897 | SURFACE): A, rollout | K-12 says default-ON behind a disable flag and also “observe-then-enforce”; the original must clarify the rollout transition and initial default. | anchor=11-K:L897

### Surfaces — SUMMARY/A-08 (1)

- **SUMMARY/A-08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L899 | SURFACE): A, market claims | “Industry has no number,” moat ranking, Polygraph comparisons, OpenRouter/LiteLLM facts, and paper collapse figures are grounding claims, not ELAI feature behavior; originals/grounding must preserve citations. | anchor=11-K:L899

### Surfaces — SUMMARY/K-1 (3)

- **SUMMARY/K-1.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L63 | SURFACE): E, external datum | Preserve KW-5 SpecBench +28 percentage points, explicitly external. | anchor=11-K:L63
- **SUMMARY/K-1.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L66 | SURFACE): E, invariant | An external-facing claim may ship only with a journal-recomputable ELAI number or an explicit `[EXTERNAL]`/`[DESIGN]` tag. | anchor=11-K:L66
- **SUMMARY/K-1.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L74 | SURFACE): E, done gate | Every dossier row must contain either a measured ELAI number plus GD-2 recompute pointer, or an explicit design/external tag plus KW source. | anchor=11-K:L74

### Surfaces — SUMMARY/K-11 (9)

- **SUMMARY/K-11.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L278 | SURFACE): E, PR UI | Add a `## Evidence` section to the PR comment/check-run. | anchor=11-K:L278
- **SUMMARY/K-11.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L279 | SURFACE): E, cockpit UI | Show the same evidence in the reviewer cockpit. | anchor=11-K:L279
- **SUMMARY/K-11.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L284 | SURFACE): E, evidence | Render references to screenshots already captured by `visual-ui-loop`. | anchor=11-K:L284
- **SUMMARY/K-11.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L286 | SURFACE): E, fail-closed UI | Render a no-artifact hunk as `UNVOUCHED`, never clean or omitted. | anchor=11-K:L286
- **SUMMARY/K-11.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L289 | SURFACE): E, CLI | Add `elai pr-review --emit-evidence`. | anchor=11-K:L289
- **SUMMARY/K-11.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L291 | SURFACE): E, kill switch | Allow `ELAI_PR_EVIDENCE_RENDER_DISABLE` to disable the render. | anchor=11-K:L291
- **SUMMARY/K-11.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L293 | SURFACE): E, operations | Gate live check-run posting on network availability/permission under W-10. | anchor=11-K:L293
- **SUMMARY/K-11.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L294 | SURFACE): E, delivery boundary | Resolve the PR-auto-fix doctrine tension by shipping poll-and-surface only in the wedge slice. | anchor=11-K:L294
- **SUMMARY/K-11.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L297 | SURFACE): E, grounding limit | Treat the kunchen “engineering manager, not diff reviewer” statement as an AUTHOR/market-validation claim, not an ELAI measurement. | anchor=11-K:L297

### Surfaces — SUMMARY/K-12 (2)

- **SUMMARY/K-12.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L319 | SURFACE): E, default/rollout | Make the gate default-ON behind `ELAI_LOSSLESS_GATE_DISABLE`, while using observe-then-enforce rollout under GD-4. | anchor=11-K:L319
- **SUMMARY/K-12.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L321 | SURFACE): E, UI | Add no separate frontend; verdicts flow to existing context/trust panels. | anchor=11-K:L321

### Surfaces — SUMMARY/K-13 (2)

- **SUMMARY/K-13.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L334 | SURFACE): E, cell schema | Each cell names ELAI control(s). | anchor=11-K:L334
- **SUMMARY/K-13.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L347 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L347

### Surfaces — SUMMARY/K-14 (4)

- **SUMMARY/K-14.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L360 | SURFACE): E, TZ-1 | Put volatile time strictly after the last cache breakpoint, never in system/rules prompts. | anchor=11-K:L360
- **SUMMARY/K-14.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L377 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L377
- **SUMMARY/K-14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L817 | SURFACE):  | constrains future time/async work so volatile context remains replay-compatible while supervisor liveness stays outside replay authority. | anchor=11-K:L817
- **SUMMARY/K-14 time anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L997 | SURFACE):  | `.claude/rules/time-determinism.md`, `clock.rs`, cache breakpoints, background-task/notification primitives. | anchor=11-K:L997

### Surfaces — SUMMARY/K-15 (4)

- **SUMMARY/K-15.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L387 | SURFACE): E, CLI | Add `elai bench reasoning-budget` through the bench-subcommand seam. | anchor=11-K:L387
- **SUMMARY/K-15.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L392 | SURFACE): E, determinism | Run both arms under `ELAI_SEED`. | anchor=11-K:L392
- **SUMMARY/K-15.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L397 | SURFACE): E, prohibition | Never import the paper's 6K/7K knee as an ELAI threshold. | anchor=11-K:L397
- **SUMMARY/K-15.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L407 | SURFACE): E, UI | Reuse I-27 `ConvergencePanel` and H-9 `CloudPanel`, or explicitly justify N/A at edit time. | anchor=11-K:L407

### Surfaces — SUMMARY/K-16 (2)

- **SUMMARY/K-16.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L428 | SURFACE): E, RR-3 prohibition | Never use context-window size as the reasoning-control lever. | anchor=11-K:L428
- **SUMMARY/K-16.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L434 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L434

### Surfaces — SUMMARY/K-17 (8)

- **SUMMARY/K-17.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L444 | SURFACE): E, differentiator | Add a K-1 dossier row stating that ELAI measures a context-degradation knee per `(model, lane)` against gate verdicts. | anchor=11-K:L444
- **SUMMARY/K-17.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L446 | SURFACE): E, local policy | Describe the local regime as window-minimization plus unload-early. | anchor=11-K:L446
- **SUMMARY/K-17.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L447 | SURFACE): E, replacement | Replace folklore 0.75-local/0.85-cloud window fractions with the measured J-34 knee. | anchor=11-K:L447
- **SUMMARY/K-17.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L451 | SURFACE): E, no duplication | Add no parallel context bench; J-34 `elai bench context-fill` is the number source. | anchor=11-K:L451
- **SUMMARY/K-17.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L457 | SURFACE): E, UI | Reuse `ContextStatePanel`/H-9 or justify N/A. | anchor=11-K:L457
- **SUMMARY/K-17.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L459 | SURFACE): E, negative | Fail an untagged “ELAI never loses context” claim. | anchor=11-K:L459
- **SUMMARY/K-17.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L460 | SURFACE): E, negative | Fail an unmeasured “0% degradation at full window” claim. | anchor=11-K:L460
- **SUMMARY/K-17 context anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L999 | SURFACE):  | `context_budget.rs`, `context_policy.rs`, `ContextCalibrationEvent`, `ContextStatePanel`. | anchor=11-K:L999

### Surfaces — SUMMARY/K-18 (5)

- **SUMMARY/K-18.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L468 | SURFACE): E, CLI | Add `elai bench case-reuse` through the shared bench seam. | anchor=11-K:L468
- **SUMMARY/K-18.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L475 | SURFACE): E, determinism | Run under `ELAI_SEED`. | anchor=11-K:L475
- **SUMMARY/K-18.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L486 | SURFACE): E, state | Prefer an existing bench row/`ConvergenceCurveMeasured` with `CaseReuseArm`/`RecurrenceHit`; add at most one family. | anchor=11-K:L486
- **SUMMARY/K-18.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L487 | SURFACE): E, UI | Reuse I-27/H-9 panels or justify N/A. | anchor=11-K:L487
- **SUMMARY/K-18.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L493 | SURFACE): E, OUT-OF-K | The VCG J-track mints a typed case only on F-4 proof, self-wires deterministic edges, adds a MEMORY/CASE canvas altitude, gates retrieval-time reuse, and defines episodic-to-skill-to-rule promotion; it is technical companion scope, not K-18 implementation. | anchor=11-K:L493

### Surfaces — SUMMARY/K-19 (3)

- **SUMMARY/K-19.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L510 | SURFACE): E, SF-3 | Keep reuse default-OFF and behind the wire-dark allowlist. | anchor=11-K:L510
- **SUMMARY/K-19.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L519 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L519
- **SUMMARY/K-19.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L523 | SURFACE): E, research rationale | Preserve the summary's central-risk claim that self-graded improvement collapses, with external Misevolve/F-10 figures treated as grounding, not ELAI measurements. | anchor=11-K:L523

### Surfaces — SUMMARY/K-2 (6)

- **SUMMARY/K-2.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L82 | SURFACE): E, CLI | Add `elai bench routing-economics` through a `routing-economics` bench subcommand at the `args_bench.rs:75` seam. | anchor=11-K:L82
- **SUMMARY/K-2.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L88 | SURFACE): E, determinism | Run both arms under `ELAI_SEED`. | anchor=11-K:L88
- **SUMMARY/K-2.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L93 | SURFACE): E, display | Feed H-9 cockpit display. | anchor=11-K:L93
- **SUMMARY/K-2.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L98 | SURFACE): E, UI | Decide at edit time whether a hybrid-versus-cloud table belongs on the runtime/cost panel. | anchor=11-K:L98
- **SUMMARY/K-2.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L102 | SURFACE): E, negative | A `--force-cloud-everything` arm must collapse the savings delta, demonstrating the hybrid caused the result. | anchor=11-K:L102
- **SUMMARY/K-2.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L103 | SURFACE): E, market status | The “first production number” is a planned positioning claim; the cited HERA ~45% offload is research-grade/external, not an ELAI result. | anchor=11-K:L103

### Surfaces — SUMMARY/K-20 (1)

- **SUMMARY/K-20.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L543 | SURFACE): E, state/UI | Add no SSB family; point to future `ReproPackEvent`; ride K-3/K-11 or N/A. | anchor=11-K:L543

### Surfaces — SUMMARY/K-21 (3)

- **SUMMARY/K-21.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L554 | SURFACE): E, applicability | Bind completeness, side-effect mediation, kernel observation, reconciliation, and any trust-the-journal/RRA/canvas/derived-status work. | anchor=11-K:L554
- **SUMMARY/K-21.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L570 | SURFACE): E, JC-4 platform | Describe the observer as Linux-only and Windows as mediation-only. | anchor=11-K:L570
- **SUMMARY/K-21.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L578 | SURFACE): E, audit | Catch a Windows claim that advertises the Linux observer. | anchor=11-K:L578

### Surfaces — SUMMARY/K-22 (3)

- **SUMMARY/K-22.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L585 | SURFACE): E, tool | Add deterministic `.claude/tools/doctrine_coverage.py`. | anchor=11-K:L585
- **SUMMARY/K-22.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L604 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L604
- **SUMMARY/K-22 doctrine-coverage anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1001 | SURFACE):  | `CLAUDE.md`, `.claude/rules/*.md`, `.claude/tools/doctrine_coverage.py`, `.claude/rules/doctrine-enforcement.md`, `.claude/gate-manifest.txt`, `gate_freeze_check.py`. | anchor=11-K:L1001

### Surfaces — SUMMARY/K-23 (23)

- **SUMMARY/K-23.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L614 | SURFACE): E, capability | Add a thin cross-repository workspace layer above deep per-root ELAI substrate. | anchor=11-K:L614
- **SUMMARY/K-23.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L616 | SURFACE): E, boundary | Do not mistake Cosmos cross-project artifacts or canvas multi-project views for cross-repo code intelligence. | anchor=11-K:L616
- **SUMMARY/K-23.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L617 | SURFACE): E, U1 object | Add a workspace object and manifest. | anchor=11-K:L617
- **SUMMARY/K-23.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L619 | SURFACE): E, U1 journal | Add a global `workspace.ssb.db` workspace journal. | anchor=11-K:L619
- **SUMMARY/K-23.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L620 | SURFACE): E, U1 CLI | Add `workspace create`. | anchor=11-K:L620
- **SUMMARY/K-23.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L621 | SURFACE): E, U1 CLI | Add `workspace add`. | anchor=11-K:L621
- **SUMMARY/K-23.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L622 | SURFACE): E, U1 CLI | Add `workspace map`. | anchor=11-K:L622
- **SUMMARY/K-23.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L629 | SURFACE): E, U3 retrieval | Fan a workspace query to every member's existing retrieval pipeline. | anchor=11-K:L629
- **SUMMARY/K-23.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L635 | SURFACE): E, U5 CLI | Add `workspace orchestrate`. | anchor=11-K:L635
- **SUMMARY/K-23.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L636 | SURFACE): E, U5 FSM | Add a workspace FSM above each per-root FSM. | anchor=11-K:L636
- **SUMMARY/K-23.29** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L642 | SURFACE): E, U6 resume | Resume from the workspace journal plus each member's H-51 digest. | anchor=11-K:L642
- **SUMMARY/K-23.32** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L645 | SURFACE): E, U6 UI | Add a canvas `WORKSPACE` altitude beside J-31/J-60/J-98. | anchor=11-K:L645
- **SUMMARY/K-23.36** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L649 | SURFACE): E, reuse | Reuse `paths.rs` unchanged and compose Cosmos, RRF, B-29/J-16, J-18, orchestrator, Tier-C, J-80, J-2/H-36, Kahn, context budget, H-51/J-21/C-48/J-59, SSB/FSM, and canvas substrate. | anchor=11-K:L649
- **SUMMARY/K-23.38** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L651 | SURFACE): E, implementation | Add workspace verbs under the CLI. | anchor=11-K:L651
- **SUMMARY/K-23.41** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L654 | SURFACE): E, configuration | Gate the track with `ELAI_WORKSPACE_ENABLE` and an allowlist promotion row. | anchor=11-K:L654
- **SUMMARY/K-23.42** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L655 | SURFACE): E, fixture | On a two-repo fixture, `workspace map acme` reports exactly one Sound `types <- api` edge and zero false edges. | anchor=11-K:L655
- **SUMMARY/K-23.43** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L656 | SURFACE): E, impact proof | `workspace impact` reports the real consumer in repo B with zero false consumers. | anchor=11-K:L656
- **SUMMARY/K-23.44** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L657 | SURFACE): E, orchestration proof | `workspace orchestrate` lands two PRs and a green cross-repo contract test. | anchor=11-K:L657
- **SUMMARY/K-23.46** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L659 | SURFACE): E, UI proof | The canvas altitude re-derives byte-identically. | anchor=11-K:L659
- **SUMMARY/K-23.49** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L662 | SURFACE): E, negative | A workspace store written under a member root fails. | anchor=11-K:L662
- **SUMMARY/K-23.50** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L663 | SURFACE): E, local-first | A centralized cloud workspace store fails. | anchor=11-K:L663
- **SUMMARY/K-23 workspace:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L826 | SURFACE):  | each root retains its own deep index/FSM; the workspace layer derives Sound cross-repo edges, federates retrieval, expands impact only over Sound edges, orders delegated per-root work, journals workspace state globally, and renders a deterministic canvas altitude. | anchor=11-K:L826
- **SUMMARY/K-23 workspace anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1002 | SURFACE):  | `paths.rs:173`, `crates/orchestrator/src/workspace/`, CLI workspace verbs, `crates/orchestrator/src/blast_radius.rs`, `workspace.ssb.db`, `ELAI_WORKSPACE_ENABLE`, canvas WORKSPACE altitude. | anchor=11-K:L1002

### Surfaces — SUMMARY/K-24 (10)

- **SUMMARY/K-24.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L672 | SURFACE): E, upstream | ELAI pays upstream provider cost. | anchor=11-K:L672
- **SUMMARY/K-24.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L673 | SURFACE): E, controls | ELAI owns limits, rollover, and routing policy. | anchor=11-K:L673
- **SUMMARY/K-24.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L682 | SURFACE): E, L1 CLI | Add `billing balance`. | anchor=11-K:L682
- **SUMMARY/K-24.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L694 | SURFACE): E, L4 failure | Refuse out-of-credit work with a clear user-facing surface. | anchor=11-K:L694
- **SUMMARY/K-24.29** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L698 | SURFACE): E, L5 boundary | Keep Stripe/Paddle outside ELAI and maintain zero PCI scope. | anchor=11-K:L698
- **SUMMARY/K-24.35** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L704 | SURFACE): E, implementation | Add `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, and `elai billing` CLI. | anchor=11-K:L704
- **SUMMARY/K-24.36** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L705 | SURFACE): E, integration | Wire J-49 admission, J-28 booking, and an H-9 CloudPanel billing tile. | anchor=11-K:L705
- **SUMMARY/K-24.38** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L707 | SURFACE): E, configuration | Gate the track with `ELAI_BILLING_ENABLE` and an allowlist promotion row. | anchor=11-K:L707
- **SUMMARY/K-24.40** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L709 | SURFACE): E, proof | `billing balance` folds correctly. | anchor=11-K:L709
- **SUMMARY/K-24 billing anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1003 | SURFACE):  | `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, `seats.toml`, `ssb.rs:1508`, `action_type_lock.rs:775`, `ELAI_BILLING_ENABLE`, H-9 `CloudPanel`. | anchor=11-K:L1003

### Surfaces — SUMMARY/K-25 (8)

- **SUMMARY/K-25.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L723 | SURFACE): E, snapshot | Add `workflow/status/provider-surface.json`. | anchor=11-K:L723
- **SUMMARY/K-25.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L730 | SURFACE): E, tool | Add stdlib `.claude/tools/absorption_watch.py`. | anchor=11-K:L730
- **SUMMARY/K-25.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L731 | SURFACE): E, CLI | Support `--json` output. | anchor=11-K:L731
- **SUMMARY/K-25.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L732 | SURFACE): E, CLI | Support `--max-age-days`, default 90. | anchor=11-K:L732
- **SUMMARY/K-25.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L740 | SURFACE): E, overlap behavior | Surface exit-1 rows as kill/de-scope proposals. | anchor=11-K:L740
- **SUMMARY/K-25.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L744 | SURFACE): E, precedent | Mirror the `elai_status_query.py` Layer-0 advisory-cache boundary. | anchor=11-K:L744
- **SUMMARY/K-25.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L746 | SURFACE): E, state/UI | Add no SSB family and no frontend. | anchor=11-K:L746
- **SUMMARY/K-25 watchdog anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1004 | SURFACE):  | `workflow/status/provider-surface.json`, `.claude/tools/absorption_watch.py`, `.claude/rules/absorption-watch.md`, `elai_status_query.py`, `.claude/rules/proof-spine.md`, `proof_spine_status.py`. | anchor=11-K:L1004

### Surfaces — SUMMARY/K-26 (4)

- **SUMMARY/K-26.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L764 | SURFACE): E, dogfood | Put real `.elai_cc` fixes through `elai run` for model-agnostic accrual. | anchor=11-K:L764
- **SUMMARY/K-26.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L765 | SURFACE): E, seam gap | Record FORK-5: `elai run` lacks a GPT-5.5 seam. | anchor=11-K:L765
- **SUMMARY/K-26.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L770 | SURFACE): E, conditional build | Build `--suite journal` conditionally from the probe result. | anchor=11-K:L770
- **SUMMARY/K-26.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L775 | SURFACE): E, repaired defect | Patch the `elai run` model-seam gap found by the red team. | anchor=11-K:L775

### Surfaces — SUMMARY/K-3 (10)

- **SUMMARY/K-3.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L109 | SURFACE): E, distribution | Render ELAI evidence into GitHub check-runs and PR comments. | anchor=11-K:L109
- **SUMMARY/K-3.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L115 | SURFACE): E, render | Include an offline-verification instruction using `elai verify-attestation`. | anchor=11-K:L115
- **SUMMARY/K-3.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L117 | SURFACE): E, fail-closed UI | Render a hunk with no evidence as `UNVOUCHED`; never omit it or mark it clean. | anchor=11-K:L117
- **SUMMARY/K-3.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L118 | SURFACE): E, reuse | Implement as a renderer over the existing C-40/E-41 `github_pr` adapter and `elai pr-review`; reimplement none of the posting machinery. | anchor=11-K:L118
- **SUMMARY/K-3.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L119 | SURFACE): E, reuse | Reuse G-5 review-surface, H-52 anchors, F-4 evidence, G-8 attestation, and J-16 cone data. | anchor=11-K:L119
- **SUMMARY/K-3.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L123 | SURFACE): E, state | Prefer existing `github_pr`, review-surface, or attestation rows; add at most one event. | anchor=11-K:L123
- **SUMMARY/K-3.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L124 | SURFACE): E, UI | Pair with the existing GitHub state surface (`github.svelte.ts`), decision finalized at edit time. | anchor=11-K:L124
- **SUMMARY/K-3.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L125 | SURFACE): E, done gate | On a real fixture-repo PR, every check-run claim must join to a journal row. | anchor=11-K:L125
- **SUMMARY/K-3.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L126 | SURFACE): E, negative | A `--post-unvouched-as-clean` broken arm must fail. | anchor=11-K:L126
- **SUMMARY/K-3/K-11 GitHub anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L994 | SURFACE):  | `pr_review_cmd.rs`, `crates/orchestrator/src/github_pr/{review.rs,body_composer.rs,comment_watcher.rs,adapter_trait.rs,evidence_render.rs}`, `github.svelte.ts`, `.claude` `visual-ui-loop`. | anchor=11-K:L994

### Surfaces — SUMMARY/K-4 (4)

- **SUMMARY/K-4.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L132 | SURFACE): E, CLI | Add `elai bench gate-tax` through a `gate-tax` bench subcommand at `args_bench.rs:75`. | anchor=11-K:L132
- **SUMMARY/K-4.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L142 | SURFACE): E, reuse | Reuse G-6 ablation framing, the 105-row `wire-dark-allowlist.toml` gate inventory, and E-49 metrics. | anchor=11-K:L142
- **SUMMARY/K-4.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L144 | SURFACE): E, UI | Consider a friction tile on the cost/audit surface at edit time. | anchor=11-K:L144
- **SUMMARY/K-4/K-5/K-19 promotion anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L995 | SURFACE):  | `wire-dark-allowlist.toml`, the frozen gate manifest, `ELAI_LOSSLESS_GATE_DISABLE`, `ELAI_PR_EVIDENCE_RENDER_DISABLE`. | anchor=11-K:L995

### Surfaces — SUMMARY/K-5 (3)

- **SUMMARY/K-5.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L155 | SURFACE): E, always-on | Keep the E-30 argv allowlist always on. | anchor=11-K:L155
- **SUMMARY/K-5.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L163 | SURFACE): E, risk input | Use F-2 CWE-surface class. | anchor=11-K:L163
- **SUMMARY/K-5.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L174 | SURFACE): E, UI | Consider tier-decision rows in the audit timeline. | anchor=11-K:L174

### Surfaces — SUMMARY/K-6 (4)

- **SUMMARY/K-6.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L184 | SURFACE): E, CLI | Add `elai export compliance`. | anchor=11-K:L184
- **SUMMARY/K-6.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L198 | SURFACE): E, integration | Add a CLI arm composing existing RRA, attestation, and redaction paths plus a strategy mapping doc. | anchor=11-K:L198
- **SUMMARY/K-6.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L200 | SURFACE): E, UI | CLI-only N/A is justified; a compliance tile remains an edit-time option. | anchor=11-K:L200
- **SUMMARY/K-6.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L201 | SURFACE): E, portability | Verify the exported bundle offline on a clean machine with `elai verify-attestation` and a conformance checker. | anchor=11-K:L201

### Surfaces — SUMMARY/K-8 (2)

- **SUMMARY/K-8.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L237 | SURFACE): E, fresh research | Before executing any efficiency-class unit, verify the provider's current native surface via WebSearch. | anchor=11-K:L237
- **SUMMARY/K-8.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L244 | SURFACE): E, negative | Flag an efficiency-class task that skips the provider recheck. | anchor=11-K:L244

### Surfaces — SUMMARY/K-9 (2)

- **SUMMARY/K-9.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L265 | SURFACE): E, implementation | Add a cadence checklist and bench wrapper over J-9. | anchor=11-K:L265
- **SUMMARY/K-9.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L267 | SURFACE): E, UI | Consider a refresh-decision tile at edit time. | anchor=11-K:L267

### Surfaces — SUMMARY/K-X (6)

- **SUMMARY/K-X.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L32 | SURFACE): E, exceptions | K-23 multi-repo/workspace memory and K-24 billing/credits are the two explicitly acknowledged net-new capability exceptions. | anchor=11-K:L32
- **SUMMARY/K-X.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L34 | SURFACE): E, first gate | K first invokes the already-shipped A-1 falsifier and may trigger a kill-switch before further build work. | anchor=11-K:L34
- **SUMMARY/K-X.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L37 | SURFACE): E, distribution | A-4 supplies the portable artifact and K-3/K-11 put its machine and human render where review happens: the PR/check-run and cockpit. | anchor=11-K:L37
- **SUMMARY/K-X.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L41 | SURFACE): E, differentiators | KD-1 is number-before-claim; KD-2 is meet-the-buyer-in-the-PR; KD-3 is a composed-friction ceiling; KD-4 is provider-absorption pre-gating; KD-5 is a time-boxed wedge/window clock. | anchor=11-K:L41
- **SUMMARY/K-X.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L47 | SURFACE): E, UI discipline | Runtime-facing additions either name a frontend pair or defer the choice to edit time under W-4; CLI/doc-only work may justify N/A under W-6. | anchor=11-K:L47
- **SUMMARY/K-X.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L49 | SURFACE): E, promotion discipline | The two net-new tracks inherit run-trigger PASS, R10 integration smoke, journal-as-record, default-OFF enablement, and an allowlist promotion row. | anchor=11-K:L49

### Surfaces — SUMMARY/SUMMARY-other (8)

- **SUMMARY/WF-K-3** [SUM-WF] (11-K-SUMMARY_INDEX.discovery.md:L789 | SURFACE): workflow | K-3 turns closed-set machine evidence into PR/check-run verdicts; **K-11** adds the human evidence view and DSR prioritization. | anchor=11-K:L789
- **SUMMARY/WF-K-3-K-11** [SUM-WF] (11-K-SUMMARY_INDEX.discovery.md:L803 | SURFACE): workflow | K-3/K-11 render the same evidence set in the PR and cockpit; missing evidence becomes `UNVOUCHED`. | anchor=11-K:L803
- **SUMMARY/Claim truth:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L832 | SURFACE):  | measured ELAI number plus recompute pointer, or explicit design/external/pending status; never a bare superlative or imported research number presented as ELAI's result. | anchor=11-K:L832
- **SUMMARY/Time determinism:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L840 | SURFACE):  | volatile time comes after cache breakpoints, exact time input is journaled, wall time never decides gates/transitions/promotions, and supervisor wall-clock kill is outside replay. | anchor=11-K:L840
- **SUMMARY/Observe-first/default-off:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L842 | SURFACE):  | risky case reuse, observer enforcement, new workspace, and billing promotion follow observe-first or allowlisted activation as specified. | anchor=11-K:L842
- **SUMMARY/Local-first storage:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L844 | SURFACE):  | workspace state cannot move under an arbitrary member root or into a centralized cloud store; credit truth cannot be delegated to a provider spend store. | anchor=11-K:L844
- **SUMMARY/Honest platform/mode boundaries:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L846 | SURFACE):  | Linux observation is not advertised on Windows; disconnected repositories get no invented edges; cloud/action-faithful replay is not mislabeled token-faithful; cassettes prove an old captured world. | anchor=11-K:L846
- **SUMMARY/Verified Case Graph:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L886 | SURFACE):  | proof-minted typed nodes, deterministic edges, MEMORY/CASE canvas altitude, gated reuse, episodic-to-skill-to-rule ladder. | anchor=11-K:L886

## Ledger K-001 — all atomic records

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-K-001.discovery.md`
Extracted ID-bearing rows: **288**.

### K-001 source atoms by unit

#### Unit `FIX` — 7 atoms (E=6, I=1)

- **K-001/FIX-E001** [SRC/E] (DEEP-K-001.discovery.md:L34 | SURFACE): `E/TEST` — The file is a negative fixture for K-1 Task 5, seeded so `claims_policy_lint.py` must FAIL. — plan-anchor=`FIX-claims:L1-L1`
- **K-001/FIX-E002** [SRC/E] (DEEP-K-001.discovery.md:L35): `E/LIMIT` — The file exists only to be scanned by the lint; it is not positioning copy. — plan-anchor=`FIX-claims:L3-L4`
- **K-001/FIX-E003** [SRC/E] (DEEP-K-001.discovery.md:L36 | SURFACE): `E/FAIL` — Violation class 1 is a banned superlative claim: “ELAI is the safest coding agent on the market today.” — plan-anchor=`FIX-claims:L4-L6`
- **K-001/FIX-E004** [SRC/E] (DEEP-K-001.discovery.md:L37): `E/FAIL` — Violation class 2 is a bare untagged external number: “Independent researchers found hallucinated packages in 21.7% of open-source LLM code generations.” — plan-anchor=`FIX-claims:L4-L9`
- **K-001/FIX-E005** [SRC/E] (DEEP-K-001.discovery.md:L38): `E/DATA` — Both lines ship a claim with no `[EXTERNAL]`/`[DESIGN]`/`[MEASURED]` tag and no recompute pointer on the same line. — plan-anchor=`FIX-claims:L11-L13`
- **K-001/FIX-E006** [SRC/E] (DEEP-K-001.discovery.md:L39 | SURFACE): `E/SURFACE` — Required invocation surface: `claims_policy_lint.py --files <this file>` must exit nonzero. — plan-anchor=`FIX-claims:L13-L14`
- **K-001/FIX-I001** [SRC/I] (DEEP-K-001.discovery.md:L40): `I/TEST` — The fixture encodes exactly the two fail arms K-1 acceptance requires (unmeasured superlative + bare external number) and no third class. — plan-anchor=`FIX-claims:L1-L14`

#### Unit `K0` — 72 atoms (A=3, C=1, E=68)

- **K-001/K0-E001** [SRC/E] (DEEP-K-001.discovery.md:L50): `E/STATUS` — K-plan is the future-proof / wedge-execution side: 23 net-new sessions K-1..K-9 + K-11..K-24 across tracks K-T1..K-T6, plus KAT/OGT/GRC/CMG/VCG/LFG steals and MPW+BCC capability waves; 8 Section-A execution pointers; authored 2026-06-12 with later 2026-06-26 waves. — plan-anchor=`K-0:L1-L32`
- **K-001/K0-E002** [SRC/E] (DEEP-K-001.discovery.md:L51): `E/STATUS` — K-10 is sequencing/cut DOCTRINE folded into this index per the 2026-06-12 user ruling; it is NOT a session file. — plan-anchor=`K-0:L1-L1, L106-L110, L509-L523`
- **K-001/K0-E003** [SRC/E] (DEEP-K-001.discovery.md:L52): `E/DEP` — Source synthesis: K-MASTER-BRIEF [KS-1] is authoritative for SCOPE; live ANCHORS are superseded by the grounding map KC corrections. — plan-anchor=`K-0:L34-L46`
- **K-001/K0-E004** [SRC/E] (DEEP-K-001.discovery.md:L53): `E/STATUS` — K is unusual: it mostly PROVES, DISTRIBUTES, makes-affordable, PACKAGES, and DISCIPLINES the prior verification/evidence spine; almost no new capability except K-23/K-24. — plan-anchor=`K-0:L48-L53, L63-L74`
- **K-001/K0-E005** [SRC/E] (DEEP-K-001.discovery.md:L54): `E/DEP` — Mandatory predecessor reads before any K-N: grounding map, F/G/H/J wiring canons, MASTER-BRIEF §§1-5, this index, K-SUMMARY_INDEX, then the K-N detail file. — plan-anchor=`K-0:L55-L61`
- **K-001/K0-E006** [SRC/E] (DEEP-K-001.discovery.md:L55): `E/CAP` — K-T1 PROVE (A-1, A-2, A-3, K-1, K-2): invoke shipped A-1 falsifier first; A-2/A-3 produce measured deltas; K-1 binds EXTERNAL KW numbers to MEASURED rows + claims policy; K-2 owns first production hybrid-routing economics number. Delta: KD-1. — plan-anchor=`K-0:L65-L74`
- **K-001/K0-E007** [SRC/E] (DEEP-K-001.discovery.md:L56 | SURFACE): `E/CAP` — K-T2 WEDGE + DISTRIBUTION (A-4, K-3, A-5): A-4 is portable `.rra`/`elai wrap`; K-3 renders evidence into GitHub check-runs + PR comments; A-5 is review-time study. Delta: KD-2. — plan-anchor=`K-0:L76-L82`
- **K-001/K0-E008** [SRC/E] (DEEP-K-001.discovery.md:L57): `E/CAP` — K-T3 ADAPTIVE GATE ECONOMY (K-4, K-5): K-4 builds composed gate-tax budget with locked ceiling (breach = release BLOCK); K-5 builds risk-tiered gating (LLM never picks tier). Delta: KD-3. — plan-anchor=`K-0:L84-L90`
- **K-001/K0-E009** [SRC/E] (DEEP-K-001.discovery.md:L58 | SURFACE): `E/CAP` — K-T4 BUYER-SIDE EVIDENCE (K-6, K-7): K-6 builds `elai export compliance`; K-7 is AppSec/insurance positioning brief (pure prose). No org/SSO build. — plan-anchor=`K-0:L92-L96`
- **K-001/K0-E010** [SRC/E] (DEEP-K-001.discovery.md:L59): `E/CAP` — K-T5 DEPRECIATION + REFRESH (K-8, K-9): K-8 absorbable-unit audit + provider PRE_GATE; K-9 quarterly model-refresh cadence on J-9. Delta: KD-4. — plan-anchor=`K-0:L98-L104`
- **K-001/K0-E011** [SRC/E] (DEEP-K-001.discovery.md:L60 | SURFACE): `E/CAP` — K-T6 SEQUENCING / CUT (K-10 doctrine in index): priority order, no-new-letters, activation-over-authoring, canvas demotion, kill-switches. Delta: KD-5. — plan-anchor=`K-0:L106-L110`
- **K-001/K0-E012** [SRC/E] (DEEP-K-001.discovery.md:L61): `E/STATUS` — Status legend: PLANNED / IN PROGRESS / DONE / DEFERRED; cargo-green or file-exists is never sufficient for DONE (W-1). — plan-anchor=`K-0:L112-L118`
- **K-001/K0-E013** [SRC/E] (DEEP-K-001.discovery.md:L62): `E/CFG` — Inherits W-1..W-10, GD-1/GD-2/GD-4/GD-5, and HD-1 where efficiency/cost numbers apply. — plan-anchor=`K-0:L123-L135`
- **K-001/K0-E014** [SRC/E] (DEEP-K-001.discovery.md:L63 | SURFACE): `E/CFG` — KD-1 NUMBER-BEFORE-CLAIM: no externally-visible claim ships without a measured journal-recomputable ELAI number OR explicit `[EXTERNAL]`/`[DESIGN-RATIONALE]` tag. Enforced by K-1 lint; binds K-1 and K-7. — plan-anchor=`K-0:L140-L144`
- **K-001/K0-E015** [SRC/E] (DEEP-K-001.discovery.md:L64 | SURFACE): `E/CFG` — KD-2 MEET-THE-BUYER-IN-THE-PR: every trust signal ships a PR/CI rendering or justified W-6 absence; dashboard is deep view never only view. Binds K-3. — plan-anchor=`K-0:L145-L148`
- **K-001/K0-E016** [SRC/E] (DEEP-K-001.discovery.md:L65): `E/CFG` — KD-3 COMPOSED-FRICTION CEILING: whole-system gate tax measured per release against a locked ceiling; new default-ON gate that breaches does not ship default-ON. Binds K-4/K-5. — plan-anchor=`K-0:L149-L153`
- **K-001/K0-E017** [SRC/E] (DEEP-K-001.discovery.md:L66 | SURFACE): `E/CFG` — KD-4 PROVIDER-ABSORPTION PRE_GATE: efficiency-class units re-verify provider CURRENT native surface (WebSearch) before EXECUTING; duplicating a now-native feature is kill-or-de-scope. Binds K-8 and future efficiency plans. — plan-anchor=`K-0:L154-L157`
- **K-001/K0-E018** [SRC/E] (DEEP-K-001.discovery.md:L67 | SURFACE): `E/CFG` — KD-5 WINDOW-CLOCK: wedge artifacts (A-4 `.rra`, K-3 PR emitter, K-6 compliance export) are time-boxed first-mover deliverables; a slipped quarter escalates to the user. Binds A-4, K-3, K-6. — plan-anchor=`K-0:L158-L161`
- **K-001/K0-E019** [SRC/E] (DEEP-K-001.discovery.md:L68 | SURFACE): `E/TEST` — DONE for BUILD sessions requires cargo check GREEN, auditor PASS on affected crates, wiring_contract_check `--run-trigger` PASS, cross_lang_diff if frontend_pair, and GD-4/KD-3 friction bound where gate defaults ON. — plan-anchor=`K-0:L163-L166`
- **K-001/K0-E020** [SRC/E] (DEEP-K-001.discovery.md:L69): `E/TEST` — DONE for positioning/doctrine sessions requires deterministic artifact plus NEGATIVE arm FAIL (lint catches unmeasured superlative; absorbable-unit rule loads; uncited brief number fails review). — plan-anchor=`K-0:L167-L169`
- **K-001/K0-E021** [SRC/E] (DEEP-K-001.discovery.md:L70): `E/TEST` — DONE also requires EXTERNAL numbers carry KD-1 tags, KW re-verify at dossier build, `workflow/reasoning/K-N-decisions.md`, and memory-writer ran. — plan-anchor=`K-0:L170-L172`
- **K-001/K0-E022** [SRC/E] (DEEP-K-001.discovery.md:L71): `E/DEP` — Section-A pointers (not K-N files): A-1 I-43 claim-scope SHIPPED; A-2 F-2b SeCodePLT live; A-3 G-6 SAFEDELTA; A-4 I-40 wrap+`.rra` PLANNED; A-5 G-5 review-time study; A-6 H-7/I-25 routing; A-7 J-9 catalog; A-8 J-28/J-29 PLANNED and BLOCKS K-2 live cost leg. — plan-anchor=`K-0:L175-L193`
- **K-001/K0-E023** [SRC/E] (DEEP-K-001.discovery.md:L72 | SURFACE): `E/FLOW` — Section-A order: A-1 invoke → go → A-2 → A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips; A-6/A-7/A-8 parallel-safe. — plan-anchor=`K-0:L192-L193`
- **K-001/K0-E024** [SRC/E] (DEEP-K-001.discovery.md:L73 | SURFACE): `E/SURFACE` — CLI surface named at index: `elai bench claim-scope [--suite][--seed][--json]` is A-1’s shipped kill-gate. — plan-anchor=`K-0:L178-L180, L584-L587`
- **K-001/K0-E025** [SRC/E] (DEEP-K-001.discovery.md:L74): `E/CAP` — K-1: problem-quant dossier + claims policy; PROVABLE doc+lint; no new action_type; KD-1 enforcement; wave KW-1; effort S. — plan-anchor=`K-0:L199-L201, L411-L412`
- **K-001/K0-E026** [SRC/E] (DEEP-K-001.discovery.md:L75 | SURFACE): `E/CAP` — K-2: `elai bench routing-economics`; live cost leg gated on J-28 + RunPod; net 0..+1 SSB or prefer-reuse. — plan-anchor=`K-0:L202-L206, L413-L415`
- **K-001/K0-E027** [SRC/E] (DEEP-K-001.discovery.md:L76 | SURFACE): `E/SURFACE` — K-3 PR/CI evidence emitter: GitHub check-runs + PR comments; no-evidence hunk renders UNVOUCHED never silently clean. — plan-anchor=`K-0:L207-L210`
- **K-001/K0-E028** [SRC/E] (DEEP-K-001.discovery.md:L77 | SURFACE): `E/SURFACE` — K-4: `elai bench gate-tax`; ceiling breach = release BLOCK; names top-3 friction contributors. — plan-anchor=`K-0:L211-L216`
- **K-001/K0-E029** [SRC/E] (DEEP-K-001.discovery.md:L78): `E/CFG` — K-5: deterministic R8-frozen GateIntensity table; LLM never picks tier (TD-3); must fit under K-4 ceiling. — plan-anchor=`K-0:L217-L220`
- **K-001/K0-E030** [SRC/E] (DEEP-K-001.discovery.md:L79 | SURFACE): `E/SURFACE` — K-6: `elai export compliance` → redacted RRA + attestation + audit extract + retention + procurement mapping; offline verify; no raw secrets. — plan-anchor=`K-0:L221-L225`
- **K-001/K0-E031** [SRC/E] (DEEP-K-001.discovery.md:L80 | SURFACE): `E/SURFACE` — K-7: AppSec/insurance positioning brief; pure prose; no runtime; no SSB; user-reviewed before external use. — plan-anchor=`K-0:L226-L229`
- **K-001/K0-E032** [SRC/E] (DEEP-K-001.discovery.md:L81): `E/CAP` — K-8: absorbable-unit audit + provider pre_gate; ABSORBED/AT-RISK/DURABLE tags; kill list PROPOSAL never auto-applied; rule via conditional-routing; no SSB. — plan-anchor=`K-0:L230-L234, L426-L427`
- **K-001/K0-E033** [SRC/E] (DEEP-K-001.discovery.md:L82): `E/CAP` — K-9: quarterly model-refresh; ADOPT/KEEP/RE-EVALUATE user-ratified; cannot silently swap; lane doctrine restated. — plan-anchor=`K-0:L235-L240, L428-L429`
- **K-001/K0-E034** [SRC/E] (DEEP-K-001.discovery.md:L83 | SURFACE): `E/SURFACE` — K-11 extends K-3 with human-viewable `## Evidence` section in PR/comment + cockpit; risk-tiered; UNVOUCHED when no artifact. — plan-anchor=`K-0:L243-L248`
- **K-001/K0-E035** [SRC/E] (DEEP-K-001.discovery.md:L84): `E/CAP` — K-12: flip lossless compaction WARN→FAIL-CLOSED; handoff-faithfulness Critical-Atom-Recall gate. — plan-anchor=`K-0:L249-L255`
- **K-001/K0-E036** [SRC/E] (DEEP-K-001.discovery.md:L85): `E/DATA` — K-13: OWASP control-mapping doc + injection number bound into K-1 dossier as [BENCH-PENDING]; extend K-1 lint; no new action_type. — plan-anchor=`K-0:L258-L265, L434-L436`
- **K-001/K0-E037** [SRC/E] (DEEP-K-001.discovery.md:L86): `E/CFG` — K-14: `.claude/rules/time-determinism.md` TZ-1..TZ-4; conditional-routing load; bank replayable async-tools differentiator as K-1 [DESIGN] row. — plan-anchor=`K-0:L266-L275, L437-L439`
- **K-001/K0-E038** [SRC/E] (DEEP-K-001.discovery.md:L87 | SURFACE): `E/SURFACE` — K-15: `elai bench reasoning-budget`; packages receipt-grounded-reasoning differentiator into K-1 dossier; prefer-reuse ConvergenceCurveMeasured. — plan-anchor=`K-0:L278-L286, L440-L442`
- **K-001/K0-E039** [SRC/E] (DEEP-K-001.discovery.md:L88): `E/CFG` — K-16: `.claude/rules/reasoning-receipts.md` RR-1..RR-4; conditional-routing; extend K-1 lint; zero SSB; pre-gates J-83..J-86. — plan-anchor=`K-0:L287-L295, L443-L445`
- **K-001/K0-E040** [SRC/E] (DEEP-K-001.discovery.md:L89): `E/CAP` — K-17: bank measured regime-aware context-management differentiator; packages J-34; no new bench; KD-4 framing on absorbed caching/compaction. — plan-anchor=`K-0:L298-L307, L446-L448`
- **K-001/K0-E041** [SRC/E] (DEEP-K-001.discovery.md:L90 | SURFACE): `E/SURFACE` — K-18: `elai bench case-reuse`; packages per-customer compounding-state differentiator; reuse arm OBEYS K-19. — plan-anchor=`K-0:L310-L317, L449-L452`
- **K-001/K0-E042** [SRC/E] (DEEP-K-001.discovery.md:L91): `E/CFG` — K-19: `.claude/rules/self-improvement-fitness.md` SF-1..SF-4; pre-gates VCG J-track + K-18 reuse arm. — plan-anchor=`K-0:L318-L328, L453-L455`
- **K-001/K0-E043** [SRC/E] (DEEP-K-001.discovery.md:L92): `E/CAP` — K-20: bank re-executable-not-just-replayable RRA differentiator as THE WEDGE; [DESIGN]/[BENCH-PENDING]; packages J-14 extension; zero SSB. — plan-anchor=`K-0:L331-L339, L456-L459`
- **K-001/K0-E044** [SRC/E] (DEEP-K-001.discovery.md:L93): `E/CFG` — K-21: `.claude/rules/journal-completeness.md` JC-1..JC-4; integrity≠completeness; pre-gates E1 J sub-track. — plan-anchor=`K-0:L340-L352, L460-L462`
- **K-001/K0-E045** [SRC/E] (DEEP-K-001.discovery.md:L94 | SURFACE): `E/SURFACE` — K-22: `doctrine_coverage.py` + coverage ledger + enforcement tags; measures rulebook enforcement; no SSB. — plan-anchor=`K-0:L353-L364, L463-L466`
- **K-001/K0-E046** [SRC/E] (DEEP-K-001.discovery.md:L95 | SURFACE): `E/CAP` — K-23 NET-NEW multi-repo/workspace memory: U1..U6 slices; +WorkspaceEvent; `ELAI_WORKSPACE_ENABLE`; feature-map group 1.15; no centralized cloud store. — plan-anchor=`K-0:L366-L382, L467-L472`
- **K-001/K0-E047** [SRC/E] (DEEP-K-001.discovery.md:L96 | SURFACE): `E/SURFACE` — K-23 CLI surfaces: `workspace create/add/map`, `workspace orchestrate`; canvas WORKSPACE altitude. — plan-anchor=`K-0:L370-L377`
- **K-001/K0-E048** [SRC/E] (DEEP-K-001.discovery.md:L97 | SURFACE): `E/CAP` — K-24 NET-NEW credit-based seats on OpenRouter rail: L1..L6; +CreditLedgerEvent; `ELAI_BILLING_ENABLE`; balance is fold never counter; single-writer double-spend defense. — plan-anchor=`K-0:L384-L402, L473-L478`
- **K-001/K0-E049** [SRC/E] (DEEP-K-001.discovery.md:L98 | SURFACE): `E/SURFACE` — K-24 CLI/UI: `billing balance`; H-9 CloudPanel billing tile needs ts-rs binding; “$20=$200” pitch gated on L6 measured SAFEDELTA. — plan-anchor=`K-0:L390-L397, L400-L402`
- **K-001/K0-E050** [SRC/E] (DEEP-K-001.discovery.md:L99): `E/PRIV` — K-24 payload carries seat-id/amounts/model/lane/provider-cost-ref, NEVER card data or payment secret; Stripe/Paddle outside ELAI (PCI scope zero). — plan-anchor=`K-0:L395-L396, L473-L476`
- **K-001/K0-E051** [SRC/E] (DEEP-K-001.discovery.md:L100): `E/EVENT` — Authoring-time live baseline stamps: 236/545 (2026-06-12), ~267–268/577 (2026-06-26 derived); NEVER hard-code; re-grep at edit (W-9); bump SEQUENTIALLY with 9-tuple golden lockstep. — plan-anchor=`K-0:L408-L410, L479-L485`
- **K-001/K0-E052** [SRC/E] (DEEP-K-001.discovery.md:L101): `E/EVENT` — Per-session SSB plan: K-1/K-7/K-8/K-13/K-14/K-16/K-17/K-19/K-20/K-21/K-22 = zero new action_type; K-2..K-6/K-9/K-11/K-12/K-15/K-18 = prefer-reuse net 0..+1; K-23 +WorkspaceEvent; K-24 +CreditLedgerEvent. — plan-anchor=`K-0:L411-L478`
- **K-001/K0-E053** [SRC/E] (DEEP-K-001.discovery.md:L102 | SURFACE): `E/FLOW` — KW-1: INVOKE A-1 first (kill-switch if NULL/NEGATIVE) + parallel K-8. KW-2: A-2→A-3 feed K-1; K-2 fixture; K-3 distribution; K-9 cadence. KW-3: K-4→K-5, K-6/K-7, K-13/K-14, K-15/K-16 (author K-16 before/with J GRC executors). — plan-anchor=`K-0:L490-L506`
- **K-001/K0-E054** [SRC/E] (DEEP-K-001.discovery.md:L103 | SURFACE): `E/CFG` — K-10 doctrine: (1) A-1..A-4+K-3 above all else; (2) NO NEW PLAN LETTERS until those DONE (new FILES within K allowed); (3) activation-over-authoring; (4) canvas Track 2 H-10..H-16 DEMOTED behind K-3+A-5 not killed; (5) I-31 GRPO last; (6) kill-switches part of doctrine. — plan-anchor=`K-0:L509-L523`
- **K-001/K0-E055** [SRC/E] (DEEP-K-001.discovery.md:L104): `E/LIMIT` — Non-goals: no IDE play; no app-generator; no SWE-bench-Verified headline; no org/SSO/RBAC/multi-tenant in K; no gate-everything defaults; no local-lane expansion beyond mechanical/privacy until K-9 passes 24GB candidate; no efficiency tricks without KD-4; no over-claim of determinism beyond honest tier. — plan-anchor=`K-0:L526-L538`
- **K-001/K0-E056** [SRC/E] (DEEP-K-001.discovery.md:L105 | SURFACE): `E/FAIL` — Kill-switch 1: A-1/A-3 NULL/NEGATIVE safety delta at bounded FP → trust wedge as HEADLINE dies; re-aim to per-customer compounding state; K-T2/K-T4 stop. — plan-anchor=`K-0:L543-L545`
- **K-001/K0-E057** [SRC/E] (DEEP-K-001.discovery.md:L106 | SURFACE): `E/FAIL` — Kill-switch 2: K-4 composed tax cannot fit tolerable ceiling → aggressive adaptive-collapse; revisit K-5 weights. — plan-anchor=`K-0:L546-L547`
- **K-001/K0-E058** [SRC/E] (DEEP-K-001.discovery.md:L107 | SURFACE): `E/FAIL` — Kill-switch 3: GitHub/lab ships native cryptographic replayable vendor-neutral evidence → A-4 becomes “be compatible”; pivot to compliance export + cockpit-on-top; K-3 survives. — plan-anchor=`K-0:L548-L550`
- **K-001/K0-E059** [SRC/E] (DEEP-K-001.discovery.md:L108 | SURFACE): `E/FAIL` — Kill-switch 4: A-5 no review-time win → cockpit repositions as compliance artifact; K-3 remains distribution. — plan-anchor=`K-0:L551-L552`
- **K-001/K0-E060** [SRC/E] (DEEP-K-001.discovery.md:L109 | SURFACE): `E/FAIL` — Kill-switch 5: 2x3090 box missing → K-9 still runs catalog+cloud comparisons; 24GB adoption deferred. — plan-anchor=`K-0:L553-L554`
- **K-001/K0-E061** [SRC/E] (DEEP-K-001.discovery.md:L110 | SURFACE): `E/LIMIT` — Hard rules: blake3 only; no `.unwrap()`/`.expect()` on prod; scoped tests auditor-only; anti-slopsquatting; GD-1 compose-not-restate; main LLM never runs tests nor writes memory-bank; every external number tagged; default-OFF needs allowlist promotion plan. — plan-anchor=`K-0:L557-L564`
- **K-001/K0-E062** [SRC/E] (DEEP-K-001.discovery.md:L111): `E/EVIDENCE` — Web landmine: KW-1 bare “43%” is the stricter all-10 sub-metric; cite BOTH 58% across-10 and 43% all-10 with distinction. — plan-anchor=`K-0:L569-L570`
- **K-001/K0-E063** [SRC/E] (DEEP-K-001.discovery.md:L112 | SURFACE): `E/EVIDENCE` — SpecBench “+28pp per 10x” is PLAN-asserted; quote precise table value + page anchor before external use. — plan-anchor=`K-0:L571-L572`
- **K-001/K0-E064** [SRC/E] (DEEP-K-001.discovery.md:L113): `E/EVIDENCE` — KW-6 DORA/METR/vendor-telemetry PENDING-VERIFY; must not enter plan files as asserted numbers until K-1 re-verifies into dossier. — plan-anchor=`K-0:L573-L574`
- **K-001/K0-E065** [SRC/E] (DEEP-K-001.discovery.md:L114): `E/STATUS` — Session log empty at generation. — plan-anchor=`K-0:L577-L579`
- **K-001/K0-E066** [SRC/E] (DEEP-K-001.discovery.md:L115 | SURFACE): `E/FLOW` — Literal first command of WAVE KW-1: `elai bench claim-scope --suite default --seed 1234 --json`; if NULL/NEGATIVE STOP kill-switch 1; parallel start K-8. — plan-anchor=`K-0:L584-L587`
- **K-001/K0-E067** [SRC/E] (DEEP-K-001.discovery.md:L116 | SURFACE): `E/DATA` — External problem numbers (KW set): KW-1 packages 5.2%/21.7% + recurrence; KW-2 11x vulns; KW-3 reward-hacking; KW-4 CodeAct latency/tokens; KW-5 SpecBench +28pp; all [EXTERNAL]. — plan-anchor=`K-0:L70-L71, L569-L574`
- **K-001/K0-E068** [SRC/E] (DEEP-K-001.discovery.md:L117): `E/DEP` — K-11 EXTENDS K-3; K-12 enforces lossless RECORD + handoff faithfulness; K-15/K-16 GRC wave; K-17 CMG; K-18/K-19 VCG; K-20/K-21/K-22 LFG; technical halves OUT-OF-K as J/E extensions. — plan-anchor=`K-0:L3-L31, L242-L364`
- **K-001/K0-A001** [SRC/A] (DEEP-K-001.discovery.md:L118): `A/STATUS` — Multiple “LIVE” schema stamps (236/545 vs ~267–268/577) are dated; which stamp is current is execution-time re-grep only. — plan-anchor=`K-0:L408-L485`
- **K-001/K0-A002** [SRC/A] (DEEP-K-001.discovery.md:L119): `A/EVENT` — Many sessions leave “prefer-reuse vs +1” as decide-at-edit; concrete action_type names are provisional. — plan-anchor=`K-0:L413-L478`
- **K-001/K0-A003** [SRC/A] (DEEP-K-001.discovery.md:L120 | SURFACE): `A/SURFACE` — K-9 and K-6 frontend_pair choices are deferred to edit-time W-4/W-6. — plan-anchor=`K-0:L221-L225, L235-L240`
- **K-001/K0-C001** [SRC/C] (DEEP-K-001.discovery.md:L121): `C/STATUS` — Index claims K adds almost no new capability, then records K-23/K-24 as first net-new-capability K files; both statements are intentional and co-present. — plan-anchor=`K-0:L48-L53, L32-L32, L366-L402`

#### Unit `K1` — 34 atoms (A=3, E=30, I=1)

- **K-001/K1-E001** [SRC/E] (DEEP-K-001.discovery.md:L131): `E/STATUS` — K-1 is PLANNED WAVE KW-1; E-23-class near-zero code; doc + deterministic claims-policy lint; KD-1; dossier assembles as A-1/A-2/A-3 numbers land. — plan-anchor=`K-1:L1-L3`
- **K-001/K1-E002** [SRC/E] (DEEP-K-001.discovery.md:L132 | SURFACE): `E/CAP` — Ships two artifacts: (1) DOSSIER binding EXTERNAL KW numbers to ELAI MEASURED numbers; (2) CLAIMS POLICY doc + deterministic LINT. — plan-anchor=`K-1:L5-L10`
- **K-001/K1-E003** [SRC/E] (DEEP-K-001.discovery.md:L133): `E/LIMIT` — Must NOT run A-1/A-2/A-3 as new code (A-1 INVOKE; A-2/A-3 Section-A pointers); must NOT fabricate measured numbers (KD-1); must NOT re-implement I-43 honest-wording table (cite normative). — plan-anchor=`K-1:L10-L13`
- **K-001/K1-E004** [SRC/E] (DEEP-K-001.discovery.md:L134 | SURFACE): `E/DEP` — Pre-requisites: A-1 SHIPPED (`elai bench claim-scope`, args_bench.rs:70 / ssb.rs:9127); KW set web-verified; A-2/A-3 not required to author dossier shell. — plan-anchor=`K-1:L18-L20, L40-L45`
- **K-001/K1-E005** [SRC/E] (DEEP-K-001.discovery.md:L135): `E/CONC` — Parallel-safe with K-8 and K-9; NOT parallel with K-7 (shares claims-policy lint; K-7 consumes it). — plan-anchor=`K-1:L21-L22`
- **K-001/K1-E006** [SRC/E] (DEEP-K-001.discovery.md:L136 | SURFACE): `E/DATA` — External numbers (each [EXTERNAL], never claimed as ELAI’s): KW-1 USENIX 5.2%/21.7%, 58%/43% recurrence; KW-2 arXiv:2507.02976 11x (135 vs 12); KW-3 reward-hacking 73.8%/46.8%; KW-4 CodeAct ~50% latency/-60% tokens; KW-5 SpecBench +28pp (exact value PLAN-asserted, quote at build). — plan-anchor=`K-1:L33-L36, L141-L146`
- **K-001/K1-E007** [SRC/E] (DEEP-K-001.discovery.md:L137): `E/DATA` — KW-6 DORA/METR/vendor is PENDING-VERIFY — excluded until dossier-build re-verification. — plan-anchor=`K-1:L36-L37, L155-L155`
- **K-001/K1-E008** [SRC/E] (DEEP-K-001.discovery.md:L138 | SURFACE): `E/DEP` — A-1 SHIPPED: `ClaimScopeMeasured` SSB; CLI `elai bench claim-scope`; emitter `cli::bench_claim_scope::run_inner`; fixture contract `i43_claim_scope_fixture_contract.rs`. — plan-anchor=`K-1:L40-L42`
- **K-001/K1-E009** [SRC/E] (DEEP-K-001.discovery.md:L139): `E/DEP` — A-2 SeCodePLT substrate present; live local-model run is missing leg (W-10 RunPod). A-3 SAFEDELTA substrate present. — plan-anchor=`K-1:L43-L45`
- **K-001/K1-E010** [SRC/E] (DEEP-K-001.discovery.md:L140 | SURFACE): `E/CFG` — GD-3 (no SWE-bench-Verified headline; ablation framing) and GD-5 (honest residue) are doctrine the claims policy operationalizes onto marketing surface. — plan-anchor=`K-1:L46-L47`
- **K-001/K1-E011** [SRC/E] (DEEP-K-001.discovery.md:L141 | SURFACE): `E/DATA` — Author dossier (~120-180 lines): one row per problem → (external number [EXTERNAL]+KW source) paired with (ELAI measured + recompute pointer OR “PENDING — fills from A-N”). Every external number re-verified at authoring; re-quote SpecBench exact value + page anchor. — plan-anchor=`K-1:L60-L64`
- **K-001/K1-E012** [SRC/E] (DEEP-K-001.discovery.md:L142): `E/DATA` — Author CLAIMS POLICY doc (~40-60 lines): no externally-visible claim without measured journal-recomputable number or explicit `[EXTERNAL]`/`[DESIGN]` tag; honest-wording table normative. — plan-anchor=`K-1:L65-L67`
- **K-001/K1-E013** [SRC/E] (DEEP-K-001.discovery.md:L143 | SURFACE): `E/CAP` — Build claims-policy LINT (`.claude/tools/claims_policy_lint.py` stdlib-only OR elai-side check): scan README/positioning for number-or-superlative patterns; each must be adjacent to measured-pointer or `[EXTERNAL]`/`[DESIGN]` tag; exit nonzero on violation (fail-closed). ~80-120 LOC. — plan-anchor=`K-1:L68-L70`
- **K-001/K1-E014** [SRC/E] (DEEP-K-001.discovery.md:L144): `E/INT` — Wire lint into auditor path over positioning copy and conditional-routing trigger for “editing README/positioning”. Main LLM must NOT run the lint (auditor-applied). — plan-anchor=`K-1:L71-L72, L87-L89, L94-L94`
- **K-001/K1-E015** [SRC/E] (DEEP-K-001.discovery.md:L145): `E/TEST` — NEGATIVE arm: fixtures file with unmeasured superlative (“the safest coding agent”) and bare untagged external number — lint must FAIL both. — plan-anchor=`K-1:L73-L74, L100-L101, L108-L108`
- **K-001/K1-E016** [SRC/E] (DEEP-K-001.discovery.md:L146 | SURFACE): `E/DATA` — Produces: `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, `docs/strategy/ELAI-CLAIMS-POLICY.md`, `.claude/tools/claims_policy_lint.py`. — plan-anchor=`K-1:L78-L81, L133-L136`
- **K-001/K1-E017** [SRC/E] (DEEP-K-001.discovery.md:L147 | SURFACE): `E/DEP` — Consumes: `cli::bench_claim_scope` (INVOKE not rebuild); safedelta_runner + bench_seccodeplt; I-43 honest-wording table. — plan-anchor=`K-1:L82-L85`
- **K-001/K1-E018** [SRC/E] (DEEP-K-001.discovery.md:L148 | SURFACE): `E/SURFACE` — Trigger command surface: `python .claude/tools/claims_policy_lint.py --files docs/strategy/positioning/*.md`. — plan-anchor=`K-1:L97-L99`
- **K-001/K1-E019** [SRC/E] (DEEP-K-001.discovery.md:L149): `E/EVENT` — expect_ssb: none (E-23-class; deterministic check not journaled). No new SSB action_type. — plan-anchor=`K-1:L98-L99, L117-L118`
- **K-001/K1-E020** [SRC/E] (DEEP-K-001.discovery.md:L150 | SURFACE): `E/SURFACE` — Frontend pair N/A — positioning docs + CI/audit lint, no dashboard surface (W-6 justified). — plan-anchor=`K-1:L95-L96`
- **K-001/K1-E021** [SRC/E] (DEEP-K-001.discovery.md:L151): `E/TEST` — DETERMINISTIC_CHECK includes wiring_contract_check, dossier/policy file existence, negative fixture exit nonzero, dossier external tags via rg, K-1-decisions.md. — plan-anchor=`K-1:L103-L111`
- **K-001/K1-E022** [SRC/E] (DEEP-K-001.discovery.md:L152): `E/TEST` — ACCEPTANCE: KD-1 row shape; external numbers re-verified; lint fail-closed and auditor-applied; no new dependency; no new SSB. — plan-anchor=`K-1:L113-L118`
- **K-001/K1-E023** [SRC/E] (DEEP-K-001.discovery.md:L153 | SURFACE): `E/TEST` — W-1 done = lint runs in audit path + negative fixture FAILS, not “doc exists”. GD-2 recompute pointer on measured rows; GD-3 no SWE-bench headline in dossier. — plan-anchor=`K-1:L120-L125`
- **K-001/K1-E024** [SRC/E] (DEEP-K-001.discovery.md:L154): `E/TEST` — Contract tests: compliant pass; unmeasured superlative fail; bare untagged number fail; correctly [EXTERNAL]-tagged pass; empty/missing input fails-loud (R1). — plan-anchor=`K-1:L127-L130`
- **K-001/K1-E025** [SRC/E] (DEEP-K-001.discovery.md:L155): `E/DATA` — Files: CREATE dossier, claims policy, lint, negative fixture, K-1-decisions; MODIFY conditional-routing (~2 LOC load claims policy when editing positioning). — plan-anchor=`K-1:L132-L138`
- **K-001/K1-E026** [SRC/E] (DEEP-K-001.discovery.md:L156 | SURFACE): `E/EVIDENCE` — Citation anchors: KW-1 Spracklen USENIX; KW-2 arXiv:2507.02976; KW-3 OpenReview ikrQWGgxYg (gap 26.4%→57.8%); KW-4 Microsoft CodeAct BUILD 2026; KW-5 SpecBench arXiv:2605.21384. — plan-anchor=`K-1:L140-L146`
- **K-001/K1-E027** [SRC/E] (DEEP-K-001.discovery.md:L157): `E/A` — Open: dossier home docs/strategy/ vs workflow/reasoning/ — recommend docs/strategy/ (positioning; workflow/ gitignored). — plan-anchor=`K-1:L150-L152`
- **K-001/K1-E028** [SRC/E] (DEEP-K-001.discovery.md:L158): `E/CFG` — Lint superlative pattern set must stay NARROW (numbers + short banned-superlative list); observe-only first if over-fires (GD-4 posture for text gate). — plan-anchor=`K-1:L153-L154`
- **K-001/K1-E029** [SRC/E] (DEEP-K-001.discovery.md:L159): `E/DEP` — Next: makes KD-1 enforceable; gives K-7 its number source; pairs with A-1/A-2/A-3; dossier fills as benches land. — plan-anchor=`K-1:L157-L160`
- **K-001/K1-E030** [SRC/E] (DEEP-K-001.discovery.md:L160 | SURFACE): `E/SURFACE` — User-facing surfaces: dossier strategy document; claims-policy document; lint CLI; auditor positioning-copy gate; conditional-routing on README/positioning edits. — plan-anchor=`K-1:L60-L72, L95-L101`
- **K-001/K1-I001** [SRC/I] (DEEP-K-001.discovery.md:L161): `I/FLOW` — Measured dossier rows remain placeholders until A-1/A-2/A-3 land; shell can ship earlier with explicit PENDING markers. — plan-anchor=`K-1:L15-L17, L60-L64`
- **K-001/K1-A001** [SRC/A] (DEEP-K-001.discovery.md:L162): `A/CFG` — Exact banned-superlative lexicon and adjacency rules for “measured-pointer” are not fully specified beyond fail-closed intent. — plan-anchor=`K-1:L68-L70, L153-L154`
- **K-001/K1-A002** [SRC/A] (DEEP-K-001.discovery.md:L163): `A/DATA` — Final dossier path choice between docs/strategy/ and workflow/reasoning/ is open at authoring. — plan-anchor=`K-1:L60-L61, L150-L152`
- **K-001/K1-A003** [SRC/A] (DEEP-K-001.discovery.md:L164): `A/INT` — Whether lint lives as Python tool only or also as `elai`-side check is alternative, not fixed. — plan-anchor=`K-1:L68-L70, L127-L130`

#### Unit `K8` — 25 atoms (A=2, E=22, I=1)

- **K-001/K8-E001** [SRC/E] (DEEP-K-001.discovery.md:L174): `E/STATUS` — K-8 is PLANNED WAVE KW-1; audit + rule; KD-4; NO SSB; kill list is PROPOSAL never auto-applied. — plan-anchor=`K-8:L1-L3`
- **K-001/K8-E002** [SRC/E] (DEEP-K-001.discovery.md:L175): `E/CAP` — Two deliverables: (a) sweep remaining efficiency-class units tag ABSORBED / AT-RISK / DURABLE and produce kill/de-scope PROPOSAL; (b) ship KD-4 PRE_GATE rule. — plan-anchor=`K-8:L5-L12`
- **K-001/K8-E003** [SRC/E] (DEEP-K-001.discovery.md:L176): `E/LIMIT` — Rule does NOT auto-block; it adds a mandatory re-check step. No removal/de-scope without explicit user confirmation (house rule). — plan-anchor=`K-8:L10-L12, L77-L77, L103-L103`
- **K-001/K8-E004** [SRC/E] (DEEP-K-001.discovery.md:L177 | SURFACE): `E/DEP` — Pre-requisites: ELAI-FEATURE-MAP.md; absorption record KS-3 report 17; WebSearch per provider native surface. Parallel-safe with K-1, K-9. — plan-anchor=`K-8:L16-L18`
- **K-001/K8-E005** [SRC/E] (DEEP-K-001.discovery.md:L178): `E/FLOW` — Sweep targets: H-Track-1 H-1..H-9 residue; Track-14 I-10..I-16 residue; E-2/E-25/E-32 legs; J-24 tool-context legs. Confirm live PLANNED/PARTIAL from feature map + SUMMARY_INDEX at sweep time. — plan-anchor=`K-8:L24-L36`
- **K-001/K8-E006** [SRC/E] (DEEP-K-001.discovery.md:L179): `E/STATE` — Tag meanings: ABSORBED = kill or de-scope to instrument half; AT-RISK = re-check at build time; DURABLE = verification-coupled, keep. — plan-anchor=`K-8:L25-L28, L49-L51`
- **K-001/K8-E007** [SRC/E] (DEEP-K-001.discovery.md:L180): `E/DATA` — Upsert absorption tags into ELAI-FEATURE-MAP.md unit rows; bump map date. — plan-anchor=`K-8:L52-L52, L66-L66, L104-L104`
- **K-001/K8-E008** [SRC/E] (DEEP-K-001.discovery.md:L181): `E/DATA` — Author kill/de-scope PROPOSAL at `workflow/reasoning/K-8-absorbable-unit-audit.md` for USER CONFIRMATION; not auto-applied. — plan-anchor=`K-8:L53-L54, L64-L64`
- **K-001/K8-E009** [SRC/E] (DEEP-K-001.discovery.md:L182 | SURFACE): `E/CFG` — Ship `.claude/rules/provider-absorption-pregate.md`: any efficiency-class unit re-verifies provider current native surface (WebSearch) before EXECUTING; duplicating a native feature is kill-or-de-scope. Add conditional-routing load line. — plan-anchor=`K-8:L55-L57, L65-L65`
- **K-001/K8-E010** [SRC/E] (DEEP-K-001.discovery.md:L183 | SURFACE): `E/TEST` — NEGATIVE arm: efficiency-class task that skips provider re-check is audit-flagged; rule must fire on live efficiency-class task (JD-1). — plan-anchor=`K-8:L58-L59, L84-L88`
- **K-001/K8-E011** [SRC/E] (DEEP-K-001.discovery.md:L184 | SURFACE): `E/INT` — Production reachability: conditional-routing loads rule on efficiency-class task; audit flags skipped re-check; cfg(test)/contracts never satisfies R7. — plan-anchor=`K-8:L71-L75`
- **K-001/K8-E012** [SRC/E] (DEEP-K-001.discovery.md:L185): `E/LIMIT` — Stub bans: no auto-applied removal; no absorption tag without WebSearch evidence + date; no rule without conditional-routing load; no runtime code; no SSB. — plan-anchor=`K-8:L76-L80, L105-L105`
- **K-001/K8-E013** [SRC/E] (DEEP-K-001.discovery.md:L186 | SURFACE): `E/SURFACE` — Frontend pair N/A — harness doc + rule + feature-map upsert (W-6). Observable: feature-map tags grep-provable; rule file exists and loads; kill list user-decision artifact. — plan-anchor=`K-8:L81-L88`
- **K-001/K8-E014** [SRC/E] (DEEP-K-001.discovery.md:L187): `E/TEST` — DETERMINISTIC_CHECK: wiring_contract_check; rule file exists; audit proposal exists; rg provider-absorption-pregate in conditional-routing; rg ABSORBED\ — plan-anchor=`AT-RISK\`
- **K-001/K8-E015** [SRC/E] (DEEP-K-001.discovery.md:L188): `E/TEST` — ACCEPTANCE: KD-4 rule + conditional-routing load; every tag has WebSearch evidence + date; kill list proposal-only; feature-map upsert + date bump; no runtime/SSB. — plan-anchor=`K-8:L100-L105`
- **K-001/K8-E016** [SRC/E] (DEEP-K-001.discovery.md:L189 | SURFACE): `E/TEST` — W-1 done = rule loads on live efficiency-class task (JD-1), not file-exists. W-7: each unit’s provider surface re-verified at sweep (dated). — plan-anchor=`K-8:L107-L110`
- **K-001/K8-E017** [SRC/E] (DEEP-K-001.discovery.md:L190): `E/TEST` — No Rust contract; harness check that rule appears in conditional-routing load list. Kill list correctness is user review. — plan-anchor=`K-8:L112-L114`
- **K-001/K8-E018** [SRC/E] (DEEP-K-001.discovery.md:L191): `E/DATA` — Files: CREATE audit proposal, pregate rule, decisions; MODIFY conditional-routing (~2 LOC); MODIFY ELAI-FEATURE-MAP tags. — plan-anchor=`K-8:L116-L121`
- **K-001/K8-E019** [SRC/E] (DEEP-K-001.discovery.md:L192): `E/EVIDENCE` — Grounding: KS-3 report 17 absorption (caching/compaction/memory/subagents/tool-search/batch 2024-2026; predicted next: verification/audit, org governance, trajectory memory); KS-2 §1.2 perishable efficiency; CLAUDE.md no-removal-without-confirmation + KD-4 + anti-slopsquatting. — plan-anchor=`K-8:L123-L129`
- **K-001/K8-E020** [SRC/E] (DEEP-K-001.discovery.md:L193): `E/LIMIT` — Sweep WebSearches must be DATED and re-runnable; KD-4 point is re-check at BUILD time not once here. AT-RISK is honest default on partial overlap. Do NOT delete/de-scope any plan unit in this session. — plan-anchor=`K-8:L131-L136`
- **K-001/K8-E021** [SRC/E] (DEEP-K-001.discovery.md:L194): `E/DEP` — Pairs with K-9 (model-refresh other half of “don’t build stale”); enables K-10 activation-over-authoring preference. — plan-anchor=`K-8:L138-L141`
- **K-001/K8-E022** [SRC/E] (DEEP-K-001.discovery.md:L195 | SURFACE): `E/SURFACE` — User-facing/operator surfaces: feature-map absorption tags; kill/de-scope proposal doc for user decision; harness rule load on efficiency-class tasks; audit flag on skipped re-check. No dashboard panel. — plan-anchor=`K-8:L81-L88, L100-L105`
- **K-001/K8-I001** [SRC/I] (DEEP-K-001.discovery.md:L196 | SURFACE): `I/CFG` — “Does not auto-block” means EXECUTING may still proceed after re-check documentation, while audit later flags skips — enforcement is process/audit not hard runtime refuse. — plan-anchor=`K-8:L10-L12, L58-L59, L84-L88`
- **K-001/K8-A001** [SRC/A] (DEEP-K-001.discovery.md:L197): `A/STATE` — Exact criterion boundary between ABSORBED vs AT-RISK on partial native overlap is judgmental beyond “AT-RISK honest default”. — plan-anchor=`K-8:L134-L135`
- **K-001/K8-A002** [SRC/A] (DEEP-K-001.discovery.md:L198): `A/DATA` — Map header noted STALE 234/543 per KC-5; live unit inventory must be re-grepped at sweep. — plan-anchor=`K-8:L32-L36`

#### Unit `K9` — 29 atoms (A=3, E=25, I=1)

- **K-001/K9-E001** [SRC/E] (DEEP-K-001.discovery.md:L208 | SURFACE): `E/STATUS` — K-9 is PLANNED WAVE KW-2; process + bench wrapper; depends J-9 catalog; cadence cannot silently swap — user ratifies. — plan-anchor=`K-9:L1-L3`
- **K-001/K9-E002** [SRC/E] (DEEP-K-001.discovery.md:L209): `E/CAP` — Recurring quarterly evaluated checkbox: top catalog candidates per lane vs incumbent on ELAI’s OWN fixtures; output ADOPT/KEEP/RE-EVALUATE decision row for user ratification. — plan-anchor=`K-9:L5-L12, L50-L54`
- **K-001/K9-E003** [SRC/E] (DEEP-K-001.discovery.md:L210 | SURFACE): `E/CFG` — Safety: regressing candidate is auto-KEPT-incumbent; a swap needs user ratification (GD-4/user-gate). Routing/allowlist/CLAUDE.md untouched without user sign-off. — plan-anchor=`K-9:L9-L12, L61-L62, L79-L80, L105-L106`
- **K-001/K9-E004** [SRC/E] (DEEP-K-001.discovery.md:L211): `E/DEP` — Pre-requisites: J-9 HardwareModelCatalog + HardwareModelCatalogEvent; E-27 AWQ; J-19 vLLM; E-31 tier matrix; I-16 KV eval; 2x3090 for 24GB-class (pending). Parallel-safe with K-1/K-7/K-8. — plan-anchor=`K-9:L16-L19, L32-L40`
- **K-001/K9-E005** [SRC/E] (DEEP-K-001.discovery.md:L212): `E/DATA` — Metrics on fixtures: pass-rate, VRAM, tok/s, $/verified-task. — plan-anchor=`K-9:L7-L8, L25-L27, L52-L53`
- **K-001/K9-E006** [SRC/E] (DEEP-K-001.discovery.md:L213): `E/CFG` — Lane doctrine restated: local = mechanical/privacy/latency (H-50); 24GB-class = autonomy candidate; cloud = planning. — plan-anchor=`K-9:L23-L30, L53-L54`
- **K-001/K9-E007** [SRC/E] (DEEP-K-001.discovery.md:L214 | SURFACE): `E/STATE` — Prior rejections (Gemma-4, Qwen3-32B) are tier-conditional CATALOG data by J-9 (informational); routing stays untouched without sign-off. Cadence must NOT surface them as default-adopt without explicit sign-off (No-Gemma-4). — plan-anchor=`K-9:L11-L12, L38-L39, L144-L145`
- **K-001/K9-E008** [SRC/E] (DEEP-K-001.discovery.md:L215): `E/DATA` — Author cadence checklist `docs/strategy/ELAI-MODEL-REFRESH-CADENCE.md` (~80 lines): quarterly per lane; decision row schema; lane doctrine. — plan-anchor=`K-9:L51-L54, L124-L124`
- **K-001/K9-E009** [SRC/E] (DEEP-K-001.discovery.md:L216 | SURFACE): `E/CAP` — Thin bench wrapper over J-9 catalog (`benchmarks/model_refresh_runner.rs` or CLI arm): run candidates vs fixtures; emit decision row. Live model-eval legs RunPod/2x3090-gated (W-10); deterministic catalog-read + decision-schema leg ships now. — plan-anchor=`K-9:L55-L58, L125-L126`
- **K-001/K9-E010** [SRC/E] (DEEP-K-001.discovery.md:L217): `E/EVENT` — SSB: prefer-reuse J-9 HardwareModelCatalogEvent; else +ModelRefreshCadence Pattern-2 Kind: CandidateEvaluated / DecisionRecorded / IncumbentKept. Re-grep 236/545 at edit; sequential bump. — plan-anchor=`K-9:L59-L60, L89-L90`
- **K-001/K9-E011** [SRC/E] (DEEP-K-001.discovery.md:L218): `E/TEST` — NEGATIVE arms: candidate claiming win without fixture run FAILS; swap without ratification blocked. — plan-anchor=`K-9:L63-L63, L120-L121`
- **K-001/K9-E012** [SRC/E] (DEEP-K-001.discovery.md:L219 | SURFACE): `E/SURFACE` — Production CLI: `elai bench model-refresh [--lane <lane>]` (or recurring checklist invocation). — plan-anchor=`K-9:L76-L76, L88-L88`
- **K-001/K9-E013** [SRC/E] (DEEP-K-001.discovery.md:L220 | SURFACE): `E/SURFACE` — Frontend pair: refresh-decision tile (ADOPT/KEEP/RE-EVALUATE per lane) on runtime/catalog panel (decide at edit, W-4) OR W-6-justified CLI-only absence. — plan-anchor=`K-9:L84-L86, L113-L113`
- **K-001/K9-E014** [SRC/E] (DEEP-K-001.discovery.md:L221): `E/OBS` — First cadence run produces decision row + dated refresh-report; regressing candidate auto-KEPT-incumbent. — plan-anchor=`K-9:L91-L92`
- **K-001/K9-E015** [SRC/E] (DEEP-K-001.discovery.md:L222 | SURFACE): `E/LIMIT` — Stub bans: no silent model swap; no routing/allowlist/CLAUDE mutation without sign-off; no re-implement J-9 catalog; no candidate win without fixture run; no `.unwrap()`/`.expect()` on prod wrapper. — plan-anchor=`K-9:L78-L83`
- **K-001/K9-E016** [SRC/E] (DEEP-K-001.discovery.md:L223 | SURFACE): `E/TEST` — DETERMINISTIC_CHECK: wiring_contract_check --run-trigger; cadence doc exists; rg model-refresh in cli; rg IncumbentKept/auto-kept/ratif in runner; routing/allowlist count MUST be 0 in runner; decisions file. — plan-anchor=`K-9:L94-L102`
- **K-001/K9-E017** [SRC/E] (DEEP-K-001.discovery.md:L224 | SURFACE): `E/TEST` — ACCEPTANCE: no silent swap; routing untouched; GD-1 compose J-9/E-27/J-19/E-31; HD-1 fixture-measured; cargo check GREEN; auditor PASS -p orchestrator,-p cli; SSB net 0..+1 sequential. — plan-anchor=`K-9:L104-L109`
- **K-001/K9-E018** [SRC/E] (DEEP-K-001.discovery.md:L225): `E/TEST` — W-1 done = first cadence run produces decision row + dated report. W-5/W-10 split live eval vs schema leg. — plan-anchor=`K-9:L111-L115`
- **K-001/K9-E019** [SRC/E] (DEEP-K-001.discovery.md:L226 | SURFACE): `E/TEST` — Contract k9_model_refresh_contract.rs: catalog read; decision row fields; regress→IncumbentKept; no fixture FAIL; no routing/allowlist mutation (grep-isolation). — plan-anchor=`K-9:L117-L121`
- **K-001/K9-E020** [SRC/E] (DEEP-K-001.discovery.md:L227 | SURFACE): `E/DATA` — Files: CREATE cadence doc, model_refresh_runner.rs, contract, decisions; MODIFY args_bench + bench arm; MODIFY ssb + action_type_lock IF +ModelRefreshCadence. — plan-anchor=`K-9:L123-L129`
- **K-001/K9-E021** [SRC/E] (DEEP-K-001.discovery.md:L228 | SURFACE): `E/EVIDENCE` — External context [EXTERNAL/DESIGN]: KS-2 §1.9 ~24GB floor, MoE-30B-A3B, open-weight SWE-bench 70s vs frontier ~80; verdicts are fixture-measured (HD-1). — plan-anchor=`K-9:L28-L30, L131-L137`
- **K-001/K9-E022** [SRC/E] (DEEP-K-001.discovery.md:L229): `E/DEFER` — If 2x3090 missing: cadence still runs (catalog + cloud-lane comparisons); 24GB-class adoption deferred; lane doctrine unchanged. — plan-anchor=`K-9:L139-L141`
- **K-001/K9-E023** [SRC/E] (DEEP-K-001.discovery.md:L230 | SURFACE): `E/LIMIT` — Full per-lane model bench is expensive: keep fixture set small; live legs RunPod-gated; catalog-read + decision-schema is cheap always-runnable part. — plan-anchor=`K-9:L142-L143`
- **K-001/K9-E024** [SRC/E] (DEEP-K-001.discovery.md:L231): `E/DEP` — Closes depreciation/refresh track with K-8; keystones A-1..A-4+K-3 ship first; cadence runs alongside and quarterly thereafter. — plan-anchor=`K-9:L147-L150`
- **K-001/K9-E025** [SRC/E] (DEEP-K-001.discovery.md:L232 | SURFACE): `E/SURFACE` — User-facing surfaces: CLI `elai bench model-refresh`; decision row ADOPT/KEEP/RE-EVALUATE; dated refresh report; optional refresh-decision tile; cadence strategy doc. — plan-anchor=`K-9:L76-L92, L84-L86`
- **K-001/K9-I001** [SRC/I] (DEEP-K-001.discovery.md:L233): `I/EVENT` — Prefer-reuse path means first cadence may emit zero new action_type if J-9 rows already carry sufficient kinds. — plan-anchor=`K-9:L59-L60, L89-L90`
- **K-001/K9-A001** [SRC/A] (DEEP-K-001.discovery.md:L234 | SURFACE): `A/SURFACE` — Whether refresh-decision tile ships or CLI-only W-6 absence is undecided until edit. — plan-anchor=`K-9:L84-L86`
- **K-001/K9-A002** [SRC/A] (DEEP-K-001.discovery.md:L235): `A/EVENT` — Final choice between HardwareModelCatalogEvent reuse and +ModelRefreshCadence is decide-at-edit. — plan-anchor=`K-9:L59-L60`
- **K-001/K9-A003** [SRC/A] (DEEP-K-001.discovery.md:L236): `A/CFG` — Exact fixture set size and quarterly scheduler/cron mechanism are unspecified. — plan-anchor=`K-9:L51-L58, L142-L143`

#### Unit `K16` — 29 atoms (A=2, E=26, I=1)

- **K-001/K16-E001** [SRC/E] (DEEP-K-001.discovery.md:L246): `E/STATUS` — K-16 is PLANNED WAVE KW-3; CANT-LIE+DISCIPLINE; E-23 + doctrine class; K-8/K-14-shaped; rule file + conditional-routing; NO runtime code; NO SSB; reuses K-1 lint. — plan-anchor=`K-16:L1-L3`
- **K-001/K16-E002** [SRC/E] (DEEP-K-001.discovery.md:L247): `E/CAP` — Promotes binding pre-gate: reasoning TRACE is monitoring/UX ONLY; confidence/selection MUST come from EXTERNAL RECEIPT. Prevents silent fallback to LLM self-eval when receipts run out. — plan-anchor=`K-16:L5-L5, L12-L13`
- **K-001/K16-E003** [SRC/E] (DEEP-K-001.discovery.md:L248): `E/LIMIT` — Must NOT author J-83..J-86 capability; must NOT introduce runtime gate (auditor-applied; runtime is shipped F-4/D-13/I-4); must NOT restate those mechanics; must NOT claim a measured number. — plan-anchor=`K-16:L5-L5, L63-L63, L87-L87`
- **K-001/K16-E004** [SRC/E] (DEEP-K-001.discovery.md:L249): `E/CFG` — RR-1 TRACE-NEVER-ADMISSIBLE: CoT/trace never confidence or selection signal; confidence/abstention from external receipt (test/checker/pixel-diff/retrieval) via engine (F-4/H-39, RD-1). — plan-anchor=`K-16:L16-L16`
- **K-001/K16-E005** [SRC/E] (DEEP-K-001.discovery.md:L250): `E/CFG` — RR-2 SELECTOR HIERARCHY: deterministic RECEIPT > statistical VOTE over INDEPENDENT samples (receipt-ABSENT fallback ONLY; NEVER for factual-recall → retrieval) > LLM self-eval (BANNED). Vote never selector when receipt exists. — plan-anchor=`K-16:L17-L17`
- **K-001/K16-E006** [SRC/E] (DEEP-K-001.discovery.md:L251 | SURFACE): `E/CFG` — RR-3 MEASURE-YOUR-OWN-BUDGET: reasoning-budget/overthinking/difficulty thresholds ELAI-MEASURED per-(model,task-type) on ELAI gold set (K-15); NEVER paper constant (ID-5); WINDOW never lever for depth-class failure (RD-4). — plan-anchor=`K-16:L18-L18`
- **K-001/K16-E007** [SRC/E] (DEEP-K-001.discovery.md:L252): `E/CFG` — RR-4 COMMIT-AND-FREEZE: receipt-confirmed reasoning step committed and frozen; generator does not re-open; commit is grounding + safe-compression boundary (preserve receipts losslessly, fold prose lossily; no LLM re-summarization HD-2; ciphers banned IC-3) + overthinking defense. — plan-anchor=`K-16:L19-L19`
- **K-001/K16-E008** [SRC/E] (DEEP-K-001.discovery.md:L253): `E/DEP` — Runtime archetypes cited (BIND not restate): F-4 AntiFabricationGate (WIRED blake3 stdout_hash); D-13 adversary cross-family (DARK); I-4 select-by-receipts (DARK); H-39 abstention_score.rs (WIRED receipt-sourced bit). — plan-anchor=`K-16:L21-L25, L108-L108`
- **K-001/K16-E009** [SRC/E] (DEEP-K-001.discovery.md:L254): `E/DEP` — Pre-gates J GRC-v3 track J-83..J-86; soft-depends K-1 lint; parallel-safe with K-13/K-14/K-15; cites K-15 for RR-3 knee. — plan-anchor=`K-16:L7-L9, L26-L27`
- **K-001/K16-E010** [SRC/E] (DEEP-K-001.discovery.md:L255): `E/DATA` — Author `.claude/rules/reasoning-receipts.md` (~55-70 lines) with RR-1..RR-4; conditional-routing trigger for building/auditing reasoning-engine / GRC / reasoning-budget / candidate-selection units. — plan-anchor=`K-16:L38-L38, L45-L47`
- **K-001/K16-E011** [SRC/E] (DEEP-K-001.discovery.md:L256): `E/INT` — Package one-line cross-link: GRC-v3 differentiator claim banked by K-15 is governed by RR-1; no new dossier row (K-15 owns it). — plan-anchor=`K-16:L39-L39`
- **K-001/K16-E012** [SRC/E] (DEEP-K-001.discovery.md:L257): `E/CAP` — EXTEND K-1 claims-policy lint (NOT fork): “self-verifying reasoning / model checks its own reasoning / chain-of-thought confidence” FAILS unless receipt-not-self-eval correction; paper-sourced reasoning-budget knee FAILS (RR-3/ID-5). ~10-15 LOC; composes K-15 recogniser if present. — plan-anchor=`K-16:L40-L40, L102-L102`
- **K-001/K16-E013** [SRC/E] (DEEP-K-001.discovery.md:L258): `E/TEST` — NEGATIVE fixture `reasoning_selfverify_negative.md`: (a) untagged “ELAI’s chain-of-thought self-verifies”; (b) design using LLM grading own paths; (c) budget set to paper’s 7K knee — lint fails (a)/(c); rule/auditor catches (b). — plan-anchor=`K-16:L41-L41, L77-L77`
- **K-001/K16-E014** [SRC/E] (DEEP-K-001.discovery.md:L259): `E/INT` — Production: conditional-routing loads rule on reasoning-engine/GRC tasks (JD-1); auditor runs lint over reasoning-grounding claims; J-83..J-86 executor MUST obey; contracts never satisfy R7. — plan-anchor=`K-16:L54-L58`
- **K-001/K16-E015** [SRC/E] (DEEP-K-001.discovery.md:L260): `E/LIMIT` — Stub bans: no trace as confidence; no LLM self-eval selector; no vote-when-receipt-exists / no vote-for-recall; no paper-constant budget; no fork of K-1 lint; no main-LLM self-run of lint; no re-implement F-4/D-13/I-4; no authoring J capability here. — plan-anchor=`K-16:L59-L63`
- **K-001/K16-E016** [SRC/E] (DEEP-K-001.discovery.md:L261 | SURFACE): `E/SURFACE` — Frontend pair N/A — doctrine + claims recogniser, no dashboard (W-6). — plan-anchor=`K-16:L64-L65`
- **K-001/K16-E017** [SRC/E] (DEEP-K-001.discovery.md:L262): `E/OBS` — expect_ssb: none (E-23/doctrine; JD-1 harness evidence). Artifacts: rule loads on live task; lint PASS tagged claim / FAIL negative over-claims; auditor catches LLM-self-eval selector in design. — plan-anchor=`K-16:L66-L69`
- **K-001/K16-E018** [SRC/E] (DEEP-K-001.discovery.md:L263): `E/TEST` — DETERMINISTIC_CHECK: wiring_contract_check; rule file; rg RR-1..RR-4; rg reasoning-receipts in conditional-routing; negative fixture nonzero; strategy docs must have 0 self-verify/chain-confidence/model-grades-own matches; decisions file. — plan-anchor=`K-16:L71-L80`
- **K-001/K16-E019** [SRC/E] (DEEP-K-001.discovery.md:L264): `E/TEST` — ACCEPTANCE: four clauses + load triggers; RR-1/RR-2/RR-3 content; lint extended not forked; no runtime/SSB/dependency; J capability not authored here. — plan-anchor=`K-16:L82-L87`
- **K-001/K16-E020** [SRC/E] (DEEP-K-001.discovery.md:L265): `E/TEST` — W-1 done = conditional-routing loads rule on reasoning-engine task + lint negative FAILS. — plan-anchor=`K-16:L89-L90`
- **K-001/K16-E021** [SRC/E] (DEEP-K-001.discovery.md:L266): `E/TEST` — Lint extension tests: receipt-grounded tagged passes; self-verifies fails; model checks own fails; paper 7K knee fails; empty fails-loud. Rule test is JD-1 load + reviewed-design catch, not code assertion until J-83..J-86. — plan-anchor=`K-16:L96-L97`
- **K-001/K16-E022** [SRC/E] (DEEP-K-001.discovery.md:L267 | SURFACE): `E/DATA` — Files: CREATE reasoning-receipts.md, reasoning_selfverify_negative.md, K-16-decisions; MODIFY conditional-routing; MODIFY claims_policy_lint.py. — plan-anchor=`K-16:L99-L104`
- **K-001/K16-E023** [SRC/E] (DEEP-K-001.discovery.md:L268 | SURFACE): `E/EVIDENCE` — Paper grounding [EXTERNAL]: Reflexion/Tsui self-correction blind spot; ToT self-eval weak link; self-consistency + “Consensus is Not Verification”; Lanham faithfulness; overthinking 2604.10739 model-specific knee. — plan-anchor=`K-16:L106-L109`
- **K-001/K16-E024** [SRC/E] (DEEP-K-001.discovery.md:L269): `E/DEFER` — TECHNICAL OUT-OF-K: J-83 BeliefState carrier; J-84 2-axis effort classifier; J-85 overthinking guard; J-86 receipt-gated ladder; I-14 preserve-receipts rider — all BOUND by RR-1..RR-4. — plan-anchor=`K-16:L116-L116`
- **K-001/K16-E025** [SRC/E] (DEEP-K-001.discovery.md:L270): `E/DEP` — Coordinate K-15 and K-16 lint recognisers: one lint, two claim shapes; never two forks. Landing order either way extends the other’s recogniser. — plan-anchor=`K-16:L115-L115`
- **K-001/K16-E026** [SRC/E] (DEEP-K-001.discovery.md:L271 | SURFACE): `E/SURFACE` — User-facing/operator surfaces: doctrine rule load on reasoning-engine tasks; claims lint over positioning; no dedicated dashboard panel. — plan-anchor=`K-16:L64-L69`
- **K-001/K16-I001** [SRC/I] (DEEP-K-001.discovery.md:L272): `I/SEC` — Binding “at reasoning-step grain” means future J selectors/budget classifiers are planning-constrained even though runtime enforcement remains existing spine gates. — plan-anchor=`K-16:L5-L5, L21-L25`
- **K-001/K16-A001** [SRC/A] (DEEP-K-001.discovery.md:L273): `A/TEST` — How auditor “catches” an LLM-self-eval design (review checklist vs automated design scan) is not operationalized beyond “reviewed design”. — plan-anchor=`K-16:L41-L41, L69-L69, L97-L97`
- **K-001/K16-A002** [SRC/A] (DEEP-K-001.discovery.md:L274): `A/CFG` — Exact TOML/DSL for what counts as a “reasoning-engine unit” trigger beyond keyword list is not fully specified. — plan-anchor=`K-16:L38-L38`

#### Unit `K20` — 28 atoms (A=2, C=1, E=24, I=1)

- **K-001/K20-E001** [SRC/E] (DEEP-K-001.discovery.md:L284): `E/STATUS` — K-20 is PLANNED WAVE KW-3 on wedge path; PROVABLE+PACKAGE; E-23-class; NO new bench; NO new machinery; PACKAGES J-14 cassette + G-2 RRA; ZERO new SSB; KD-1/KD-2/KD-5. — plan-anchor=`K-20:L1-L3`
- **K-001/K20-E002** [SRC/E] (DEEP-K-001.discovery.md:L285): `E/CAP` — Banks THE WEDGE differentiator: re-executable (not just replayable) evidence — third party re-runs against recorded G-1 taint-ingested external bytes under pinned env and gets matching objective-sealed verdict (EXTERNAL faithfulness). — plan-anchor=`K-20:L5-L5, L12-L13`
- **K-001/K20-E003** [SRC/E] (DEEP-K-001.discovery.md:L286 | SURFACE): `E/LIMIT` — Must NOT author J-14-extension cassette/pinned-env; must NOT build second RRA bench; must NOT fork rra.rs; measured re-execution-fidelity is [BENCH-PENDING]; must surface WORLD-DRIFT as fidelity downgrade (GD-5); keep honest-tier vocabulary (I-43). — plan-anchor=`K-20:L5-L5, L52-L56`
- **K-001/K20-E004** [SRC/E] (DEEP-K-001.discovery.md:L287 | SURFACE): `E/DEP` — Substrate cited: G-2 RRA WIRED (`elai run --emit-rra` / `rra show` / `replay --verify`; msgpack+zstd; action-stream on cloud); J-14 PLANNED verify tiers T0/T1/T2 + `rra export-fixture`; G-1 taint at 4 ingest sites; I-45 objective seal; I-18 chain; G-9 redaction/auth scrub. — plan-anchor=`K-20:L7-L8, L15-L22`
- **K-001/K20-E005** [SRC/E] (DEEP-K-001.discovery.md:L288): `E/DATA` — Package: K-1 dossier [DESIGN]/[BENCH-PENDING] row with code basis + honest-tier note; one-line claims-policy entry; K-7 brief line answering “can a third party independently re-verify without trusting your recording”. Do NOT amend K-1/K-7 plan files (row/line adds only). — plan-anchor=`K-20:L31-L31`
- **K-001/K20-E006** [SRC/E] (DEEP-K-001.discovery.md:L289): `E/CFG` — Differentiator MUST state WORLD-DRIFT as fidelity downgrade (cassette proves OLD world; non-snapshottable effects like payments/emails stay action-faithful-only with visible note). Ships no new rule; cites GD-5 + I-43 + CL-4. — plan-anchor=`K-20:L32-L32, L105-L106`
- **K-001/K20-E007** [SRC/E] (DEEP-K-001.discovery.md:L290): `E/CAP` — EXTEND K-1 lint: “reproducible/replayable/re-executable/verifiable/auditable evidence” without tier qualifier + measured-or-[DESIGN] tag FAILS; “fully reproducible / re-runs exact live world / bit-for-bit on cloud” FAILS. Coordinate with K-15/K-17 and K-21 completeness recognisers (never fork). — plan-anchor=`K-20:L33-L33, L40-L40`
- **K-001/K20-E008** [SRC/E] (DEEP-K-001.discovery.md:L291 | SURFACE): `E/TEST` — NEGATIVE fixture `reexecutable_rra_negative.md`: (a) untagged “ELAI runs are fully reproducible”; (b) “re-execute exact live world bit-for-bit”; (c) cloud token-faithful where J-13 caps did not pass — all FAIL. — plan-anchor=`K-20:L34-L34, L68-L68`
- **K-001/K20-E009** [SRC/E] (DEEP-K-001.discovery.md:L292): `E/STATE` — Honest tier: action-faithful on cloud; token-faithful only where J-13 caps pass (I-43 table). — plan-anchor=`K-20:L5-L5, L54-L54`
- **K-001/K20-E010** [SRC/E] (DEEP-K-001.discovery.md:L293 | SURFACE): `E/INT` — Production: auditor lint over differentiator copy; dossier row points to J-14-extension number when it lands; K-3/K-11 PR emitter links re-executable bundle; A-4 `elai wrap` ships it (KD-2 distribution). — plan-anchor=`K-20:L47-L50`
- **K-001/K20-E011** [SRC/E] (DEEP-K-001.discovery.md:L294 | SURFACE): `E/SURFACE` — Frontend: differentiator surfaces on K-3/K-11 PR-evidence render (“verify offline / re-execute with `elai ...`” line) + reviewer cockpit; otherwise dossier/brief. E-23-class may justify N/A (W-6). — plan-anchor=`K-20:L57-L58`
- **K-001/K20-E012** [SRC/E] (DEEP-K-001.discovery.md:L295): `E/EVENT` — expect_ssb: none; measured number is [BENCH-PENDING] pointer to J-14-extension ReproPackEvent, never a new event. — plan-anchor=`K-20:L61-L61, L84-L84`
- **K-001/K20-E013** [SRC/E] (DEEP-K-001.discovery.md:L296): `E/OBS` — Artifacts: dossier row with [DESIGN]/[BENCH-PENDING] + code basis + honest-tier; lint PASS tagged / FAIL over-claims. Live third-party re-run RunPod/CI-gated W-10. — plan-anchor=`K-20:L60-L62`
- **K-001/K20-E014** [SRC/E] (DEEP-K-001.discovery.md:L297): `E/TEST` — DETERMINISTIC_CHECK: wiring_contract_check; rg re-executable/cassette/action-faithful/J-14 in dossier; negative fixture nonzero; strategy docs 0 matches for fully reproducible/bit-for-bit/re-execute-the-exact; decisions file. — plan-anchor=`K-20:L64-L71`
- **K-001/K20-E015** [SRC/E] (DEEP-K-001.discovery.md:L298): `E/TEST` — ACCEPTANCE: KD-1 tagged differentiator; cites G-2/J-14/G-1/I-45/I-18/G-9 reimplements none; BENCH-PENDING on J-14-ext; honest tier + world-drift; lint extend not fork; J machinery not authored; KD-5 time-boxed. — plan-anchor=`K-20:L73-L78`
- **K-001/K20-E016** [SRC/E] (DEEP-K-001.discovery.md:L299): `E/TEST` — W-1 done = lint negative FAILS + dossier row exists with tag + honest-tier note. — plan-anchor=`K-20:L80-L81`
- **K-001/K20-E017** [SRC/E] (DEEP-K-001.discovery.md:L300): `E/TEST` — Contract tests: tagged differentiator with honest-tier passes; fully reproducible fails; live-world bit-for-bit fails; token-faithful uncapped fails; empty fails-loud. — plan-anchor=`K-20:L87-L88`
- **K-001/K20-E018** [SRC/E] (DEEP-K-001.discovery.md:L301 | SURFACE): `E/DATA` — Files: MODIFY dossier (+1 row), claims_policy_lint.py, K-7 brief (+1 line); CREATE reexecutable_rra_negative.md, K-20-decisions. — plan-anchor=`K-20:L90-L95`
- **K-001/K20-E019** [SRC/E] (DEEP-K-001.discovery.md:L302): `E/DEP` — Completeness coupling: re-execution only as faithful as RECORD; incomplete record still incomplete under re-exec; K-21 orthogonal; “re-executable AND complete” needs BOTH tags. — plan-anchor=`K-20:L105-L105`
- **K-001/K20-E020** [SRC/E] (DEEP-K-001.discovery.md:L303): `E/CFG` — KD-5: THE wedge artifact is time-boxed; slipped quarter on J-14-extension is ESCALATION to user not silent re-plan. Pairs with A-4 + K-3. — plan-anchor=`K-20:L107-L107`
- **K-001/K20-E021** [SRC/E] (DEEP-K-001.discovery.md:L304): `E/DEFER` — OUT-OF-K J-14 extension: (1) content-keyed VCR cassette of G-1 bytes; (2) pin env OCI+Nix lockfile+CAS; (3) re-run under I-45 sealed objective + first-divergence locator; (4) T0/T1/T2 tiers with honest downgrade + world-drift; under G-9 scrub + I-18 chain. — plan-anchor=`K-20:L108-L108`
- **K-001/K20-E022** [SRC/E] (DEEP-K-001.discovery.md:L305 | SURFACE): `E/EVIDENCE` — Market empty-cell grounding: report 8 found no public agent stack combining re-executable properties; Bazel/OCI/CAS pinned-env shape; not an ELAI measured number. — plan-anchor=`K-20:L98-L100`
- **K-001/K20-E023** [SRC/E] (DEEP-K-001.discovery.md:L306 | SURFACE): `E/SURFACE` — CLI surfaces cited (not authored here): `elai run --emit-rra`, `rra show`, `replay --verify`, planned `rra export-fixture`, A-4 `elai wrap`. — plan-anchor=`K-20:L15-L17, L50-L50`
- **K-001/K20-E024** [SRC/E] (DEEP-K-001.discovery.md:L307 | SURFACE): `E/SURFACE` — User-facing surfaces in K-20 scope: dossier differentiator row; K-7 brief line; claims lint; PR “verify offline / re-execute” line via K-3/K-11; negative fixture. — plan-anchor=`K-20:L31-L35, L57-L62`
- **K-001/K20-I001** [SRC/I] (DEEP-K-001.discovery.md:L308 | SURFACE): `I/DEP` — Because K-20 packages rather than implements re-execution, dashboard DESIGN.md consumers should treat re-exec as future-linked status until J-14-extension lands. — plan-anchor=`K-20:L5-L5, L57-L62`
- **K-001/K20-A001** [SRC/A] (DEEP-K-001.discovery.md:L309): `A/DATA` — Exact dossier row field schema beyond required tags/code-basis/honest-tier is not prescribed. — plan-anchor=`K-20:L31-L31`
- **K-001/K20-A002** [SRC/A] (DEEP-K-001.discovery.md:L310 | SURFACE): `A/SURFACE` — Exact wording of the PR “verify offline / re-execute with `elai ...`” line is illustrative, not a frozen UI string. — plan-anchor=`K-20:L57-L58`
- **K-001/K20-C001** [SRC/C] (DEEP-K-001.discovery.md:L311 | SURFACE): `C/STATUS` — Market gap language oscillates between REPLAYABLE (named market demand) and RE-EXECUTABLE (strict-stronger wedge ELAI banks); both are intentional: G-2 ships replayable; K-20 packages re-executable as stronger claim. — plan-anchor=`K-20:L5-L5, L12-L13`

### K-001 cross-file interactions (namespaced K-001/IX-*)

- **K-001/IX-001** [IX] (DEEP-K-001.discovery.md:L319): E | K-0 places K-1 and K-8 in WAVE KW-1: invoke A-1 first for kill-switch, run K-8 in parallel to stop wasted builds. | K0-E053, K0-E066; K1-E001; K8-E001
- **K-001/IX-002** [IX] (DEEP-K-001.discovery.md:L320): E | K-1 is the central claims registry; K-16 and K-20 both EXTEND `claims_policy_lint.py` rather than fork it (GD-1). | K1-E013–K1-E014; K16-E012; K20-E007
- **K-001/IX-003** [IX] (DEEP-K-001.discovery.md:L321): E | K-1 dossier is the row host for K-20’s differentiator and for many later K PROVE rows (K-13/K-14/K-15/K-17/K-18) named by the index. | K0-E036–K0-E043; K1-E011; K20-E005
- **K-001/IX-004** [IX] (DEEP-K-001.discovery.md:L322): E | FIX-claims is the concrete negative fixture K-1 DETERMINISTIC_CHECK and acceptance require for unmeasured superlative + bare external number. | FIX-E001–FIX-E006; K1-E015, K1-E021
- **K-001/IX-005** [IX] (DEEP-K-001.discovery.md:L323): E | KD-1 (index) is operationalized by K-1 policy+lint; KD-4 by K-8 pregate rule; KD-5 time-box binds wedge path including K-20. | K0-E014, K0-E017–K0-E018; K1-E002; K8-E009; K20-E020
- **K-001/IX-006** [IX] (DEEP-K-001.discovery.md:L324): E | K-8 and K-9 form K-T5 depreciation/refresh: K-8 stops building absorbed features; K-9 stops local lane silent aging. | K0-E010; K8-E021; K9-E024
- **K-001/IX-007** [IX] (DEEP-K-001.discovery.md:L325): E | K-16 pre-gates J-83..J-86 and cites K-15 as the only legal source of measured reasoning-budget knees (RR-3). | K16-E006, K16-E009, K16-E024; K0-E038–K0-E039
- **K-001/IX-008** [IX] (DEEP-K-001.discovery.md:L326): E | K-16 and K-8/K-14 share the same pre-gate shape: `.claude/rules/*` + conditional-routing + JD-1 live load evidence + zero SSB. | K16-E001, K16-E010; K8-E009–K8-E010; K0-E037
- **K-001/IX-009** [IX] (DEEP-K-001.discovery.md:L327): E | K-20 strict-strengthens G-2 replayable RRA using J-14 cassette packaging; measured proof stays OUT-OF-K on J-14-extension. | K20-E002–K20-E004, K20-E021; K0-E043
- **K-001/IX-010** [IX] (DEEP-K-001.discovery.md:L328): E | K-20 distribution path is KD-2: A-4 portable artifact + K-3/K-11 PR render of verify/re-execute line. | K20-E010–K20-E011; K0-E007, K0-E015, K0-E027, K0-E034
- **K-001/IX-011** [IX] (DEEP-K-001.discovery.md:L329): E | K-20 and K-21 are orthogonal axes: re-executable ≠ complete journal; dual claims need dual tags. | K20-E019; K0-E044
- **K-001/IX-012** [IX] (DEEP-K-001.discovery.md:L330): E | K-9 rides A-7/J-9 catalog substrate and must not reimplement it (GD-1); optional ModelRefreshCadence is the only new event candidate in this chunk. | K9-E004, K9-E010, K9-E015; K0-E033, K0-E052
- **K-001/IX-013** [IX] (DEEP-K-001.discovery.md:L331): E | External KW numbers flow K-0 landmines → K-1 dossier rows → claims lint; KW-6 blocked until re-verify. | K0-E062–K0-E064; K1-E006–K1-E007
- **K-001/IX-014** [IX] (DEEP-K-001.discovery.md:L332): E | A-1 CLI `elai bench claim-scope` is both K-0’s first action and K-1’s measured-number source via INVOKE. | K0-E024, K0-E066; K1-E004, K1-E008
- **K-001/IX-015** [IX] (DEEP-K-001.discovery.md:L333): E | Multiple sessions in this chunk are E-23-class with zero SSB: K-1, K-8, K-16, K-20 (K-9 may +1). | K0-E052; K1-E019; K8-E012; K16-E017; K20-E012; K9-E010
- **K-001/IX-016** [IX] (DEEP-K-001.discovery.md:L334): E | Auditor-applied lint is shared enforcement plane for K-1/K-16/K-20 claims; main LLM self-run is banned. | K1-E014; K16-E015; K20-E010
- **K-001/IX-017** [IX] (DEEP-K-001.discovery.md:L335): E | User ratification gates destructive decisions: K-8 kill list; K-9 model swap; K-0 human-governance doctrine. | K8-E003; K9-E003; K0-E054
- **K-001/IX-018** [IX] (DEEP-K-001.discovery.md:L336): I | If K-1 lands before K-16/K-20, later sessions only add recogniser LOC; if reverse, K-1 still owns the tool path those sessions extend. | K1-E013; K16-E025; K20-E007
- **K-001/IX-019** [IX] (DEEP-K-001.discovery.md:L337): E | Honest-tier vocabulary (action- vs token-faithful; I-43 table) is shared across K-1 normative wording and K-20 differentiator constraints. | K1-E012; K20-E003, K20-E009
- **K-001/IX-020** [IX] (DEEP-K-001.discovery.md:L338): E | K-0 non-goal “no local-lane expansion until K-9 24GB candidate passes” couples K-9 acceptance to later autonomy ambitions. | K0-E055; K9-E006, K9-E022

### K-001 summary deltas (namespaced K-001/SD-*)

- **K-001/SD-FIX-001** [SD] (DEEP-K-001.discovery.md:L365): **ADD** the concrete fixture path, exact two violation strings, tag-list including `[MEASURED]`, and required nonzero exit of `claims_policy_lint.py --files`. Summary only says “seeded unmeasured superlative / bare external number”. | 11-K:L75-L76 | FIX-E001–FIX-E006
- **K-001/SD-K0-001** [SD] (DEEP-K-001.discovery.md:L371): **ADD** full KD-1..KD-5 normative text, DONE definitions for BUILD vs doctrine sessions, and hard-rule block (blake3, unwrap ban, anti-slopsquat, allowlist). Summary compresses into K-X.* bullets. | 11-K:L28-L50, L830-L848 | K0-E013–K0-E021, K0-E061
- **K-001/SD-K0-002** [SD] (DEEP-K-001.discovery.md:L372): **ADD** complete Section-A pointer set A-1..A-8 with statuses, gating order, and A-8 blocks K-2 live cost. Summary notes Section-A exists but does not inventory them. | 11-K:L20-L25, L50-L50 | K0-E022–K0-E023
- **K-001/SD-K0-003** [SD] (DEEP-K-001.discovery.md:L373): **ADD** full KW-1..KW-3 sequencing, literal first claim-scope command, and five kill-switches with consequences. | 11-K:L33-L41, L850+ | K0-E053, K0-E056–K0-E060, K0-E066
- **K-001/SD-K0-004** [SD] (DEEP-K-001.discovery.md:L374): **ADD** K-10 six-point doctrine including no-new-letters vs new-files-within-K distinction and canvas demotion. | 11-K:L22-L22, L35-L35 | K0-E002, K0-E054
- **K-001/SD-K0-005** [SD] (DEEP-K-001.discovery.md:L375): **ADD** complete non-goals list and web landmines (43% vs 58% distinction, SpecBench PLAN-asserted, KW-6 ban). | 11-K:L41-L42, L830-L848 | K0-E055, K0-E062–K0-E064
- **K-001/SD-K0-006** [SD] (DEEP-K-001.discovery.md:L376): **ADD** per-session SSB zero/+1 plan table and multi-date LIVE schema stamps with re-grep discipline. | 11-K:L48-L48 | K0-E051–K0-E052
- **K-001/SD-K0-007** [SD] (DEEP-K-001.discovery.md:L377): **ADD** index-level CLI/UI inventory for sessions outside this chunk’s detail files (K-2..K-7, K-11..K-15, K-17..K-19, K-21..K-24) as index facts. | 11-K:L52+ | K0-E026–K0-E050
- **K-001/SD-K0-008** [SD] (DEEP-K-001.discovery.md:L378): **FLAG** co-presence of “almost no new capability” with K-23/K-24 net-new admission. | 11-K:L31-L32 | K0-C001
- **K-001/SD-K1-001** [SD] (DEEP-K-001.discovery.md:L384): **ADD** exact produce paths, lint CLI trigger over `docs/strategy/positioning/*.md`, auditor-only application, and conditional-routing edit trigger. | 11-K:L54-L76 | K1-E013–K1-E018, K1-E025
- **K-001/SD-K1-002** [SD] (DEEP-K-001.discovery.md:L385): **ADD** full KW citation set with arXiv/OpenReview identifiers and SpecBench re-quote-at-build rule; gap 26.4%→57.8% detail. | 11-K:L58-L63 | K1-E006, K1-E026
- **K-001/SD-K1-003** [SD] (DEEP-K-001.discovery.md:L386): **ADD** A-1/A-2/A-3 substrate anchors, parallel-safety (not with K-7), dossier PENDING row mechanics, and GD-3 dossier ban. | 11-K:L64-L74 | K1-E003–K1-E010, K1-E023
- **K-001/SD-K1-004** [SD] (DEEP-K-001.discovery.md:L387): **ADD** contract-test matrix including empty-input fail-loud and optional `elai`-side check alternative. | 11-K:L75-L76 | K1-E024, K1-A003
- **K-001/SD-K1-005** [SD] (DEEP-K-001.discovery.md:L388): **ADD** open dossier-home recommendation and narrow superlative / observe-first risk. | 11-K:L71-L71 | K1-E027–K1-E028, K1-A001–K1-A002
- **K-001/SD-K1-006** [SD] (DEEP-K-001.discovery.md:L389): **QUALIFY** summary “safest agent” example: numbered fixture string is “safest coding agent on the market today”. | 11-K:L75-L75 | FIX-E003; K1-E015
- **K-001/SD-K8-001** [SD] (DEEP-K-001.discovery.md:L395): **ADD** rule does not auto-block (mandatory re-check, not hard refuse); evidence+date required per tag; map date bump. | 11-K:L223-L245 | K8-E003, K8-E007, K8-E012, K8-E020
- **K-001/SD-K8-002** [SD] (DEEP-K-001.discovery.md:L396): **ADD** exact produce paths (`provider-absorption-pregate.md`, `K-8-absorbable-unit-audit.md`), DETERMINISTIC_CHECK rg commands, and no-Rust-contract test plan. | 11-K:L236-L244 | K8-E008–K8-E009, K8-E014, K8-E017–K8-E018
- **K-001/SD-K8-003** [SD] (DEEP-K-001.discovery.md:L397): **ADD** absorption record predicted-next list and feature-map STALE header caveat. | 11-K:L225-L230 | K8-E019, K8-A002
- **K-001/SD-K8-004** [SD] (DEEP-K-001.discovery.md:L398): **ADD** pairing with K-9 and activation-over-authoring (K-10) next-step framing. | 11-K:L809-L813 | K8-E021
- **K-001/SD-K9-001** [SD] (DEEP-K-001.discovery.md:L404): **ADD** exact CLI `elai bench model-refresh [--lane]`, ModelRefreshCadence kind list, IncumbentKept safe arm, and routing/allowlist MUST-be-0 isolation check. | 11-K:L247-L271 | K9-E010–K9-E012, K9-E016, K9-E019
- **K-001/SD-K9-002** [SD] (DEEP-K-001.discovery.md:L405): **ADD** W-10 split (schema leg now / live RunPod later), dated refresh-report artifact, and optional refresh-decision tile. | 11-K:L265-L268 | K9-E009, K9-E013–K9-E014, K9-A001
- **K-001/SD-K9-003** [SD] (DEEP-K-001.discovery.md:L406): **ADD** No-Gemma-4/Qwen3-32B must not be default-adopt candidates; 2x3090-missing still runs cloud comparisons. | 11-K:L263-L263 | K9-E007, K9-E022
- **K-001/SD-K9-004** [SD] (DEEP-K-001.discovery.md:L407): **ADD** cost discipline for fixture set size and files-touched list including args_bench + optional SSB bump. | 11-K:L265-L266 | K9-E020, K9-E023
- **K-001/SD-K16-001** [SD] (DEEP-K-001.discovery.md:L413): **ADD** full RR clause prose (including HD-2/IC-3 compression boundary inside RR-4) and exact conditional-routing trigger phrase. | 11-K:L414-L431 | K16-E004–K16-E007, K16-E010
- **K-001/SD-K16-002** [SD] (DEEP-K-001.discovery.md:L414): **ADD** negative fixture path, paper 7K knee arm, strategy-doc greps for self-verify language, and recogniser coordination with K-15. | 11-K:L433-L438 | K16-E012–K16-E013, K16-E018, K16-E025
- **K-001/SD-K16-003** [SD] (DEEP-K-001.discovery.md:L415): **ADD** explicit OUT-OF-K J-83..J-86 + I-14 rider list and “no new dossier row” ownership (K-15 owns differentiator). | 11-K:L433-L435 | K16-E011, K16-E024
- **K-001/SD-K16-004** [SD] (DEEP-K-001.discovery.md:L416): **ADD** shipped archetype status tags (WIRED/DARK) and re-grep paths (task_runner AntiFabricationGate). | 11-K:L432-L432 | K16-E008
- **K-001/SD-K20-001** [SD] (DEEP-K-001.discovery.md:L422): **ADD** full OUT-OF-K J-14-extension four-part machine list and ReproPackEvent pointer (no new SSB in K). | 11-K:L540-L548 | K20-E012, K20-E021
- **K-001/SD-K20-002** [SD] (DEEP-K-001.discovery.md:L423): **ADD** claims-policy multi-verb list (reproducible/replayable/re-executable/verifiable/auditable), K-21 coordination, and non-snapshottable effects note. | 11-K:L529-L543 | K20-E006–K20-E007, K20-E019
- **K-001/SD-K20-003** [SD] (DEEP-K-001.discovery.md:L424): **ADD** PR surface line “verify offline / re-execute…”, A-4/`elai wrap` packaging path, and KD-5 escalation semantics. | 11-K:L529-L543 | K20-E010–K20-E011, K20-E020
- **K-001/SD-K20-004** [SD] (DEEP-K-001.discovery.md:L425): **ADD** explicit bans on second RRA bench and rra.rs fork; G-2 CLI triad cited. | 11-K:L540-L540 | K20-E003–K20-E004, K20-E023
- **K-001/SD-K20-005** [SD] (DEEP-K-001.discovery.md:L426): **FLAG** market-gap wording dual of REPLAYABLE demand vs RE-EXECUTABLE stronger property. | 11-K:L525-L536 | K20-C001

### K-001 ambiguity and conflict register (namespaced K-001/AC-*)

- **K-001/AC-001** [AC] (DEEP-K-001.discovery.md:L344): ambiguity/status | Live SCHEMA/TOTAL stamps in K-0 are multi-dated; execution must re-grep rather than trust any single stamp. | K0-A001
- **K-001/AC-002** [AC] (DEEP-K-001.discovery.md:L345): ambiguity/event | Prefer-reuse vs +1 action_type decisions remain open for K-9 and other BUILD sessions listed only in the index. | K0-A002; K9-A002
- **K-001/AC-003** [AC] (DEEP-K-001.discovery.md:L346): ambiguity/surface | K-9 refresh-decision tile vs CLI-only is edit-time W-4/W-6 choice. | K9-A001; K0-A003
- **K-001/AC-004** [AC] (DEEP-K-001.discovery.md:L347): ambiguity/config | K-1 lint superlative lexicon and measured-pointer adjacency rules are narrow-but-not-enumerated. | K1-A001
- **K-001/AC-005** [AC] (DEEP-K-001.discovery.md:L348): ambiguity/data | K-1 dossier final path (docs/strategy vs workflow/reasoning) open; plan recommends strategy. | K1-A002
- **K-001/AC-006** [AC] (DEEP-K-001.discovery.md:L349): ambiguity/state | K-8 ABSORBED vs AT-RISK boundary on partial provider overlap is judgment-guided. | K8-A001
- **K-001/AC-007** [AC] (DEEP-K-001.discovery.md:L350): ambiguity/test | K-16 auditor catch of LLM-self-eval design is process-level, not a machine fixture beyond lint arms. | K16-A001
- **K-001/AC-008** [AC] (DEEP-K-001.discovery.md:L351): conflict/status | K-0 “almost no new capability” coexists with K-23/K-24 net-new capability admission; do not flatten either. | K0-C001
- **K-001/AC-009** [AC] (DEEP-K-001.discovery.md:L352): conflict/status | K-20 markets REPLAYABLE demand vs RE-EXECUTABLE stronger property; both intentional. | K20-C001
- **K-001/AC-010** [AC] (DEEP-K-001.discovery.md:L353): ambiguity/surface | K-20 PR re-execute line wording is illustrative, not frozen UI copy. | K20-A002
- **K-001/AC-011** [AC] (DEEP-K-001.discovery.md:L354): dependency | K-8 feature-map inventory is marked STALE at plan time; sweep must re-grep live units. | K8-A002
- **K-001/AC-012** [AC] (DEEP-K-001.discovery.md:L355): ambiguity/config | K-9 quarterly schedule mechanism and fixture-set size unspecified. | K9-A003

## Ledger K-002 — all atomic records

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-K-002.discovery.md`
Extracted ID-bearing rows: **275**.

### K-002 source atoms by unit

#### Unit `KGM` — 32 atoms (A=2, C=1, E=28, I=1)

- **K-002/KGM-E001** [SRC/E] (DEEP-K-002.discovery.md:L34): `E/STATUS` — Map authored 2026-06-12 as W-7 source for all K detail files; supersedes master-brief anchors where they go stale. — plan-anchor=`KGM:L1-L11`
- **K-002/KGM-E002** [SRC/E] (DEEP-K-002.discovery.md:L35): `E/DET` — Canonical live baseline: `SCHEMA_VERSION=236`, `TOTAL_VARIANTS=545`; never hard-code; re-read `pub const SCHEMA_VERSION` and `TOTAL_VARIANTS` at edit (W-9). — plan-anchor=`KGM:L13-L22`
- **K-002/KGM-E003** [SRC/E] (DEEP-K-002.discovery.md:L36): `E/C` — Master brief “234/543” is DEAD bait; every K SSB delta bumps sequentially from 236/545 one at a time with 9-tuple golden lockstep. — plan-anchor=`KGM:L20-L22, L84-L84`
- **K-002/KGM-E004** [SRC/E] (DEEP-K-002.discovery.md:L37 | SURFACE): `E/CAP` — KG-1: I-43 claim-scoper is SHIPPED with `ClaimScopeMeasured` (PilotDelta/CrossLaneDivergence/ClaimScoped), CLI `elai bench claim-scope [--suite] [--seed] [--json]`, emitter `bench_claim_scope::run_inner`, fixture contract. — plan-anchor=`KGM:L27-L33`
- **K-002/KGM-E005** [SRC/E] (DEEP-K-002.discovery.md:L38): `E/FLOW` — Because KG-1 landed, A-1 is INVOKE-the-shipped-gate, not build-it (KC-1). — plan-anchor=`KGM:L33-L33, L67-L71`
- **K-002/KGM-E006** [SRC/E] (DEEP-K-002.discovery.md:L39 | SURFACE): `E/CAP` — KG-2: `github_pr` is a FULL WIRED module (review, body_composer, comment_watcher, adapter_trait, gh_cli, semgrep_gate, auto_merge_guard, package_verify, git_assist, clock, types, tests) plus CLI `pr_review_cmd`/`github_pr_cmd`, Tauri `commands_c40.rs`, frontend `github.svelte.ts`, two contracts. — plan-anchor=`KGM:L34-L40`
- **K-002/KGM-E007** [SRC/E] (DEEP-K-002.discovery.md:L40 | SURFACE): `E/CAP` — KG-3: review surface present via `review_surface.rs`, `review_cmd.rs`, G-5 contracts, H-52 anchor contract; `score_review_surface`/`review_anchor` names PLAN-asserted at function grain, re-grep at edit. — plan-anchor=`KGM:L41-L45`
- **K-002/KGM-E008** [SRC/E] (DEEP-K-002.discovery.md:L41): `E/CAP` — KG-4: SAFEDELTA substrate present (`safedelta_runner.rs`, `bench_safedelta.rs`, g6 contracts). — plan-anchor=`KGM:L46-L48`
- **K-002/KGM-E009** [SRC/E] (DEEP-K-002.discovery.md:L42): `E/CAP` — KG-5: QDELTA substrate present (`bench_qdelta.rs` + benchmarks registration); K-4 gate-tax mirrors this runner shape and shares corpus. — plan-anchor=`KGM:L49-L50`
- **K-002/KGM-E010** [SRC/E] (DEEP-K-002.discovery.md:L43): `E/CAP` — KG-6: SeCodePLT substrate present; LIVE local-model run (W-10 RunPod) remains the missing leg. — plan-anchor=`KGM:L51-L53`
- **K-002/KGM-E011** [SRC/E] (DEEP-K-002.discovery.md:L44 | SURFACE): `E/CAP` — KG-7: RRA CLI present (`rra_cmd.rs`); `elai verify-attestation` referenced; precondition for A-4/K-6. — plan-anchor=`KGM:L54-L55`
- **K-002/KGM-E012** [SRC/E] (DEEP-K-002.discovery.md:L45 | SURFACE): `E/DATA` — KG-8: `wire-dark-allowlist.toml` has 105 entries; source for K-4 gate inventory and K-5 policy-table input. — plan-anchor=`KGM:L56-L58`
- **K-002/KGM-E013** [SRC/E] (DEEP-K-002.discovery.md:L46 | SURFACE): `E/SURFACE` — KG-9: bench sub-command dispatcher live at `args_bench.rs:75` with named subs list; K-2 and K-4 ADD subs here. — plan-anchor=`KGM:L59-L62`
- **K-002/KGM-E014** [SRC/E] (DEEP-K-002.discovery.md:L47): `E/STATUS` — KC-1 accelerates K-T1: I-43 already exists so first measured number is harvested from a real run, not built. — plan-anchor=`KGM:L67-L71`
- **K-002/KGM-E015** [SRC/E] (DEEP-K-002.discovery.md:L48): `E/STATUS` — KC-2: K-3 is a RENDERER over the wired github_pr adapter + G-5/H-52; re-implements neither. — plan-anchor=`KGM:L72-L74`
- **K-002/KGM-E016** [SRC/E] (DEEP-K-002.discovery.md:L49): `E/DEP` — KC-3: A-8/J-28 is NOT shipped (no `UsageTurnEvent`); K-2 live cost-delta leg GATED on J-28; deterministic fixture leg is not. — plan-anchor=`KGM:L75-L78`
- **K-002/KGM-E017** [SRC/E] (DEEP-K-002.discovery.md:L50 | SURFACE): `E/DEP` — KC-4: A-4 (`elai wrap` + .rra) is PLANNED; K-3 does NOT wait on A-4 and renders own-run evidence first. — plan-anchor=`KGM:L80-L83`
- **K-002/KGM-E018** [SRC/E] (DEEP-K-002.discovery.md:L51): `E/EVIDENCE` — KW-1 package-hallucination EXTERNAL figures: >=5.2% commercial / 21.7% open-source; 205,474 unique fakes; 58% recur across 10 / 43% recur in ALL 10; composition 38%/13%/51%. — plan-anchor=`KGM:L92-L98`
- **K-002/KGM-E019** [SRC/E] (DEEP-K-002.discovery.md:L52 | SURFACE): `E/EVIDENCE` — KW-2 patch-insecurity EXTERNAL: Llama-3.3-70B introduced 135 verified vulns vs 12 developer (=11x) over 20,000+ SWE-bench issues; agentic frameworks also introduce vulns; more autonomy = more vulns. — plan-anchor=`KGM:L99-L103`
- **K-002/KGM-E020** [SRC/E] (DEEP-K-002.discovery.md:L53 | SURFACE): `E/EVIDENCE` — KW-3 reward-hacking EXTERNAL: 73.8% Kernel-Bench and 46.8% ALE-Bench proxy-only gains; proxy-reality gap widens 26.4%->57.8% from 10 to 100 steps. — plan-anchor=`KGM:L104-L107`
- **K-002/KGM-E021** [SRC/E] (DEEP-K-002.discovery.md:L54): `E/EVIDENCE` — KW-4 orchestration overhead EXTERNAL: CodeAct collapse cuts ~50% latency and >60% tokens; grounds K-4/K-5. — plan-anchor=`KGM:L108-L112`
- **K-002/KGM-E022** [SRC/E] (DEEP-K-002.discovery.md:L55 | SURFACE): `E/EVIDENCE` — KW-5 SpecBench EXTERNAL “+28pp per 10x code size” is PLAN-asserted at exact-number grain; dossier-build must quote precise table+page before use. — plan-anchor=`KGM:L113-L116`
- **K-002/KGM-E023** [SRC/E] (DEEP-K-002.discovery.md:L56): `E/EVIDENCE` — KW-6 PENDING-VERIFY: DORA 2026, GitClear, METR, Uplevel, CodeRabbit figures stay `[EXTERNAL, unverified]` and do NOT enter plan files as asserted numbers until K-1 re-verifies. — plan-anchor=`KGM:L117-L122`
- **K-002/KGM-E024** [SRC/E] (DEEP-K-002.discovery.md:L57): `E/DEP` — KS-1 master brief is authoritative for SCOPE; this map supersedes it on ANCHORS (KC-1..KC-5). — plan-anchor=`KGM:L127-L129`
- **K-002/KGM-E025** [SRC/E] (DEEP-K-002.discovery.md:L58 | SURFACE): `E/DEP` — KS-2 post-research scorecard + kill-switches; KS-3 Perplexity reports on-disk path unknown; KW verifications stand on primary sources. — plan-anchor=`KGM:L130-L134`
- **K-002/KGM-E026** [SRC/E] (DEEP-K-002.discovery.md:L59): `E/STATUS` — KS-4 feature-map header stamp 234/543 is STALE per KC-5; re-grep at every read. — plan-anchor=`KGM:L135-L136`
- **K-002/KGM-E027** [SRC/E] (DEEP-K-002.discovery.md:L60): `E/STATUS` — DC-K1..DC-K5 activation-vs-build corrections: A-1 ACTIVATE; K-3 BUILD renderer; K-2 live leg BLOCKED-ON-J-28; A-4 BUILD; K-4/K-5 BUILD over present substrate. — plan-anchor=`KGM:L141-L145`
- **K-002/KGM-E028** [SRC/E] (DEEP-K-002.discovery.md:L61 | SURFACE): `E/LIMIT` — Every external figure is tagged `[EXTERNAL]` and NEVER claimed as an ELAI measured number (KD-1). — plan-anchor=`KGM:L87-L91`
- **K-002/KGM-I001** [SRC/I] (DEEP-K-002.discovery.md:L62): `I/DEP` — Detail-file authors that ignore KC corrections and re-assert brief anchors will fail W-7 validation twice (authoring + wiring-contract validator). — plan-anchor=`KGM:L5-L11, L65-L66`
- **K-002/KGM-A001** [SRC/A] (DEEP-K-002.discovery.md:L63 | SURFACE): `A/DATA` — Exact `score_review_surface` / `review_anchor` symbol names remain PLAN-asserted until executor re-grep. — plan-anchor=`KGM:L44-L45`
- **K-002/KGM-A002** [SRC/A] (DEEP-K-002.discovery.md:L64 | SURFACE): `A/EVIDENCE` — KW-5 exact SpecBench table value/page anchor unresolved until dossier-build. — plan-anchor=`KGM:L113-L116`
- **K-002/KGM-C001** [SRC/C] (DEEP-K-002.discovery.md:L65): `C/STATUS` — Brief claimed I-43 PLANNED and github_pr possibly dark; map flips both to SHIPPED/WIRED (KC-1/KC-2). — plan-anchor=`KGM:L67-L74, L20-L22`

#### Unit `K3` — 29 atoms (A=3, E=25, I=1)

- **K-002/K3-E001** [SRC/E] (DEEP-K-002.discovery.md:L75): `E/STATUS` — K-3 is PLANNED WAVE KW-2 BUILD-renderer over a WIRED adapter; KD-2/KD-5; single highest-leverage distribution leg; effort ~1.5 sessions (~6-7 h). — plan-anchor=`K-3:L1-L3, L16-L17`
- **K-002/K3-E002** [SRC/E] (DEEP-K-002.discovery.md:L76 | SURFACE): `E/CAP` — Builds a RENDERER that emits ELAI per-hunk evidence into GitHub check-runs + PR comments over the already-wired github_pr adapter. — plan-anchor=`K-3:L5-L14, L54-L61`
- **K-002/K3-E003** [SRC/E] (DEEP-K-002.discovery.md:L77): `E/DATA` — Closed-set render source is the RRA evidence manifest (G-2); free-text/open-set evidence comments are banned. — plan-anchor=`K-3:L26-L29, L51-L51, L55-L57, L88-L88`
- **K-002/K3-E004** [SRC/E] (DEEP-K-002.discovery.md:L78): `E/DATA` — Per-hunk evidence kinds: F-1/F-2/F-9 gate verdicts; F-4 stdout-hash-bound test outcomes; G-5 DSR + vouched/unvouched bands; H-52 anchored findings; G-8 attestation digest + offline-verify line; J-16 blast-radius cone when available. — plan-anchor=`K-3:L9-L12, L55-L57`
- **K-002/K3-E005** [SRC/E] (DEEP-K-002.discovery.md:L79 | SURFACE): `E/SURFACE` — One check-run summarizing gate verdicts + attestation digest + offline-verify line. — plan-anchor=`K-3:L58-L59, L95-L99`
- **K-002/K3-E006** [SRC/E] (DEEP-K-002.discovery.md:L80 | SURFACE): `E/SURFACE` — Per-hunk PR comments placed only via the H-52 finding-anchor resolver; hallucinated line numbers are structurally impossible. — plan-anchor=`K-3:L10-L11, L59-L61, L89-L89, L115-L115`
- **K-002/K3-E007** [SRC/E] (DEEP-K-002.discovery.md:L81): `E/FAIL` — A hunk with NO evidence renders UNVOUCHED; never silently clean, never omitted (R1). — plan-anchor=`K-3:L12-L14, L29-L30, L60-L61, L87-L87, L113-L113`
- **K-002/K3-E008** [SRC/E] (DEEP-K-002.discovery.md:L82): `E/LIMIT` — Hard boundary: renderer only; re-implements neither github_pr adapter nor any evidence source (GD-1). — plan-anchor=`K-3:L12-L14, L61-L61, L76-L77, L86-L86, L114-L114`
- **K-002/K3-E009** [SRC/E] (DEEP-K-002.discovery.md:L83 | SURFACE): `E/SURFACE` — CLI verb: `elai pr-review` evidence-emit mode; production trigger `elai pr-review <pr>` / fixture `elai pr-review <fixture-pr> --emit-evidence`. — plan-anchor=`K-3:L62-L63, L82-L84, L95-L96, L104-L104`
- **K-002/K3-E010** [SRC/E] (DEEP-K-002.discovery.md:L84 | SURFACE): `E/EVENT` — Prefer-reuse github_pr / review_surface / attestation SSB rows (net 0..+1); else +`PrEvidenceEmitted` with Kind CheckRunPosted / HunkVouched / HunkUnvouched. — plan-anchor=`K-3:L63-L64, L74-L74, L96-L97`
- **K-002/K3-E011** [SRC/E] (DEEP-K-002.discovery.md:L85 | SURFACE): `E/FAIL` — Negative arm: flag `--post-unvouched-as-clean` MUST be rejected/fail; closed-set render makes out-of-evidence “clean” structurally impossible (R2). — plan-anchor=`K-3:L65-L66, L99-L99, L106-L106, L131-L131`
- **K-002/K3-E012** [SRC/E] (DEEP-K-002.discovery.md:L86 | SURFACE): `E/SURFACE` — frontend_pair: surface emitted check-run status on `crates/dashboard-v2/src/lib/state/github.svelte.ts` (decide at edit, W-4); dashboard = DEEP view, PR = buyer view (KD-2). — plan-anchor=`K-3:L67-L67, L91-L93, L112-L112, L123-L124`
- **K-002/K3-E013** [SRC/E] (DEEP-K-002.discovery.md:L87 | SURFACE): `E/INT` — Production reachability: CLI evidence-emit mode -> github_pr adapter -> real GitHub check-run; `#[cfg(test)]`/contracts never satisfy R7. — plan-anchor=`K-3:L81-L84`
- **K-002/K3-E014** [SRC/E] (DEEP-K-002.discovery.md:L88 | SURFACE): `E/DEP` — Consumes github_pr adapter (KG-2/KC-2), review_surface G-5, H-52 resolver, F-4, G-8, J-16 when present, RRA manifest G-2. — plan-anchor=`K-3:L18-L21, L75-L80`
- **K-002/K3-E015** [SRC/E] (DEEP-K-002.discovery.md:L89): `E/DEP` — A-4 foreign-agent evidence is optional later consumer; K-3 ships own-run evidence first and does not block on A-4. — plan-anchor=`K-3:L14-L14, L43-L43, L155-L156`
- **K-002/K3-E016** [SRC/E] (DEEP-K-002.discovery.md:L90 | SURFACE): `E/DATA` — Creates `evidence_emitter.rs` under github_pr/; modifies `pr_review_cmd.rs` (~40 LOC for `--emit-evidence`); optional SSB + github.svelte.ts; contract `k3_pr_evidence_emitter_contract.rs`; `workflow/reasoning/K-3-decisions.md`. — plan-anchor=`K-3:L134-L139`
- **K-002/K3-E017** [SRC/E] (DEEP-K-002.discovery.md:L91 | SURFACE): `E/TEST` — Contract arms: full-evidence check-run + vouched comments; no-evidence UNVOUCHED; H-52 placement; every claim has journal row; `--post-unvouched-as-clean` rejected. — plan-anchor=`K-3:L128-L131`
- **K-002/K3-E018** [SRC/E] (DEEP-K-002.discovery.md:L92 | SURFACE): `E/TEST` — Deterministic checks include wiring_contract_check, rg for emit-evidence mode and UNVOUCHED, broken-arm absence, cross_lang_diff on github_pr + github.svelte.ts. — plan-anchor=`K-3:L101-L109`
- **K-002/K3-E019** [SRC/E] (DEEP-K-002.discovery.md:L93): `E/LIMIT` — No `.unwrap()`/`.expect()` on prod render path; no open-set free-text evidence; no hallucinated line numbers. — plan-anchor=`K-3:L86-L90`
- **K-002/K3-E020** [SRC/E] (DEEP-K-002.discovery.md:L94): `E/EVENT` — SSB net 0..+1 sequential from live 236/545. — plan-anchor=`K-3:L117-L117`
- **K-002/K3-E021** [SRC/E] (DEEP-K-002.discovery.md:L95): `E/DEFER` — GitHub-only v1 recommended; GitLab demand-gated later rider. — plan-anchor=`K-3:L151-L152`
- **K-002/K3-E022** [SRC/E] (DEEP-K-002.discovery.md:L96 | SURFACE): `E/OBS` — Expect artifact: real check-run whose every claim joins a journal row (GD-2); out-of-evidence comment structurally impossible. — plan-anchor=`K-3:L94-L99, L120-L120`
- **K-002/K3-E023** [SRC/E] (DEEP-K-002.discovery.md:L97 | SURFACE): `E/CFG` — W-2: emit mode is invoked, not default-OFF; IF gated, register in wire-dark-allowlist with promotion plan. — plan-anchor=`K-3:L121-L122`
- **K-002/K3-E024** [SRC/E] (DEEP-K-002.discovery.md:L98): `E/EVIDENCE` — Market premise EXTERNAL: PR is standardized supervision (KS-3 16/18); deterministic/cryptographic/replayable PR evidence is unfilled slot. — plan-anchor=`K-3:L5-L7, L141-L145`
- **K-002/K3-E025** [SRC/E] (DEEP-K-002.discovery.md:L99): `E/STATUS` — Parallel-safe with K-2/K-4/K-6/K-9; NOT parallel with edits to github_pr internals. — plan-anchor=`K-3:L22-L22`
- **K-002/K3-I001** [SRC/I] (DEEP-K-002.discovery.md:L100 | SURFACE): `I/SURFACE` — Tauri dashboard remains the deep operator surface while PR check-runs/comments are the meet-the-buyer surface under KD-2. — plan-anchor=`K-3:L30-L32, L91-L93`
- **K-002/K3-A001** [SRC/A] (DEEP-K-002.discovery.md:L101): `A/INT` — Whether body_composer/comment_watcher seams are stable enough to render over without forking is unresolved until edit-time confirm. — plan-anchor=`K-3:L153-L154`
- **K-002/K3-A002** [SRC/A] (DEEP-K-002.discovery.md:L102): `A/EVENT` — Whether to reuse existing rows or add `PrEvidenceEmitted` is decided at edit. — plan-anchor=`K-3:L63-L64, L74-L74`
- **K-002/K3-A003** [SRC/A] (DEEP-K-002.discovery.md:L103 | SURFACE): `A/SURFACE` — Exact check-run status fields on `github.svelte.ts` decided at edit (W-4). — plan-anchor=`K-3:L67-L67, L91-L93`

#### Unit `K6` — 26 atoms (A=3, E=22, I=1)

- **K-002/K6-E001** [SRC/E] (DEEP-K-002.discovery.md:L113): `E/STATUS` — K-6 is PLANNED WAVE KW-3 BUILD-composition; KD-5; no new attestation machinery; effort ~1 session (~4-5 h). — plan-anchor=`K-6:L1-L3, L16-L16`
- **K-002/K6-E002** [SRC/E] (DEEP-K-002.discovery.md:L114 | SURFACE): `E/SURFACE` — CLI verb: `elai export compliance [--out <dir\ — plan-anchor=`archive>]`; production trigger `elai export compliance --out /tmp/elai-compliance-bundle`.`
- **K-002/K6-E003** [SRC/E] (DEEP-K-002.discovery.md:L115): `E/DATA` — One bundle composing: redacted RRA (G-9 precondition) + G-8 attestation + audit-trail extract + I-44 retention statement + mapping doc. — plan-anchor=`K-6:L7-L12, L54-L56, L122-L125`
- **K-002/K6-E004** [SRC/E] (DEEP-K-002.discovery.md:L116 | SURFACE): `E/SURFACE` — Mapping document path: `docs/strategy/ELAI-COMPLIANCE-MAPPING.md` translating bundle fields onto procurement clauses. — plan-anchor=`K-6:L57-L61, L71-L71, L102-L102`
- **K-002/K6-E005** [SRC/E] (DEEP-K-002.discovery.md:L117): `E/COMP` — Mapping covers: traceability req->code->build->release (I-18); audit-log requirements; OMB M-25-22-style independent-evaluation/reproducibility; EU AI Act Art-50 “designed to support” ONLY; PLD exposure; SOC2-adjacent control narratives. — plan-anchor=`K-6:L8-L12, L57-L61, L134-L138`
- **K-002/K6-E006** [SRC/E] (DEEP-K-002.discovery.md:L118): `E/LIMIT` — Article-50-CERTIFIED claim is KILLED per GTM record; Art-50 wording is “designed to support” only. — plan-anchor=`K-6:L12-L14, L50-L51, L84-L84, L111-L111, L103-L103`
- **K-002/K6-E007** [SRC/E] (DEEP-K-002.discovery.md:L119): `E/LIMIT` — NO org/SSO/RBAC multi-tenant build (non-goal). — plan-anchor=`K-6:L14-L14, L86-L86, L148-L149`
- **K-002/K6-E008** [SRC/E] (DEEP-K-002.discovery.md:L120): `E/LIMIT` — NO new attestation/redaction machinery (composition only; GD-1). — plan-anchor=`K-6:L3-L3, L82-L82, L109-L109`
- **K-002/K6-E009** [SRC/E] (DEEP-K-002.discovery.md:L121): `E/PRIV` — Secret-grep over the export must be EMPTY (G-9 composed). — plan-anchor=`K-6:L28-L29, L62-L63, L83-L83, L95-L96, L110-L110`
- **K-002/K6-E010** [SRC/E] (DEEP-K-002.discovery.md:L122 | SURFACE): `E/TEST` — Bundle verifies offline on a clean machine via `elai verify-attestation` + conformance checker. — plan-anchor=`K-6:L27-L28, L62-L63, L94-L96, L110-L110`
- **K-002/K6-E011** [SRC/E] (DEEP-K-002.discovery.md:L123): `E/FAIL` — Negative: bundle missing attestation digest FAILS offline verify (R2). — plan-anchor=`K-6:L63-L63, K6:L96-L96, L124-L124`
- **K-002/K6-E012** [SRC/E] (DEEP-K-002.discovery.md:L124): `E/COMP` — Every 2026 regulatory fact must be cited to E-38 posture doc `docs/strategy/ELAI-2026-POSTURE.md` or primary sources; never training memory. — plan-anchor=`K-6:L31-L32, L40-L41, L48-L49, L85-L85, L112-L112`
- **K-002/K6-E013** [SRC/E] (DEEP-K-002.discovery.md:L125): `E/EVENT` — SSB: +`ComplianceExported` (Pattern-2; Kind BundleAssembled / OfflineVerified / ClauseMapped) OR reuse G-8 rows; sequential from 236/545. — plan-anchor=`K-6:L64-L65, L72-L72, L91-L93, L113-L113`
- **K-002/K6-E014** [SRC/E] (DEEP-K-002.discovery.md:L126 | SURFACE): `E/INT` — Production reachability is the real CLI export arm; test-only/contracts never satisfy R7. — plan-anchor=`K-6:L78-L80`
- **K-002/K6-E015** [SRC/E] (DEEP-K-002.discovery.md:L127): `E/DEP` — Consumes rra_cmd, verify_attestation_cmd, G-9 redaction, I-44 zero-retention, I-18 chain, E-38 posture doc. — plan-anchor=`K-6:L17-L18, L34-L41, L73-L77`
- **K-002/K6-E016** [SRC/E] (DEEP-K-002.discovery.md:L128 | SURFACE): `E/SURFACE` — frontend_pair: N/A CLI+docs justified (W-6) OR optional compliance-export tile (decide at edit). — plan-anchor=`K-6:L87-L89, L117-L118`
- **K-002/K6-E017** [SRC/E] (DEEP-K-002.discovery.md:L129): `E/DATA` — Files: `compliance_export_cmd.rs` + arm registration; mapping md; optional SSB; contract `k6_compliance_export_contract.rs`; `K-6-decisions.md` (bundle-format decision). — plan-anchor=`K-6:L127-L132`
- **K-002/K6-E018** [SRC/E] (DEEP-K-002.discovery.md:L130): `E/TEST` — Contract arms: five-part assemble; offline verify; secret-grep empty; missing-digest fail; mapping has no “certified” and cites posture doc. — plan-anchor=`K-6:L121-L125`
- **K-002/K6-E019** [SRC/E] (DEEP-K-002.discovery.md:L131): `E/TEST` — Deterministic checks ban “certified” strings in mapping and require E-38/E-38 posture citations. — plan-anchor=`K-6:L98-L106`
- **K-002/K6-E020** [SRC/E] (DEEP-K-002.discovery.md:L132): `E/DEP` — Soft-depends K-1 numbers existing (bundle evidence references them). — plan-anchor=`K-6:L19-L20`
- **K-002/K6-E021** [SRC/E] (DEEP-K-002.discovery.md:L133): `E/STATUS` — Parallel-safe with K-2/K-3/K-4/K-9. — plan-anchor=`K-6:L19-L19`
- **K-002/K6-E022** [SRC/E] (DEEP-K-002.discovery.md:L134): `E/EVIDENCE` — Buyer is org-side AppSec/DevSecOps/GRC/platform/procurement (KS-3 report 16), not individual developer. — plan-anchor=`K-6:L5-L6, L23-L25`
- **K-002/K6-A001** [SRC/A] (DEEP-K-002.discovery.md:L135 | SURFACE): `A/DATA` — Bundle format single archive vs directory layout unresolved; recommend directory + optional archive flag after reading I-44/G-8 shapes. — plan-anchor=`K-6:L55-L56, L143-L145`
- **K-002/K6-A002** [SRC/A] (DEEP-K-002.discovery.md:L136): `A/COMP` — E-38 posture 90-day staleness check: if stale at authoring, refresh before citing; EU/PLD dates must re-confirm. — plan-anchor=`K-6:L146-L147`
- **K-002/K6-A003** [SRC/A] (DEEP-K-002.discovery.md:L137 | SURFACE): `A/SURFACE` — Whether a compliance-export dashboard tile ships in v1 is edit-time choice. — plan-anchor=`K-6:L87-L89`
- **K-002/K6-I001** [SRC/I] (DEEP-K-002.discovery.md:L138 | SURFACE): `I/COMP` — Mapping doc is a procurement-language surface distinct from the machine-verifiable bundle itself. — plan-anchor=`K-6:L7-L12, L57-L61`

#### Unit `K17` — 27 atoms (A=2, E=24, I=1)

- **K-002/K17-E001** [SRC/E] (DEEP-K-002.discovery.md:L148): `E/STATUS` — K-17 is PLANNED WAVE KW-3 PROVABLE+PACKAGE E-23-class; NO new bench; PACKAGES J-34; ZERO new SSB; effort ~0.5 session. — plan-anchor=`K-17:L1-L3, L7-L7`
- **K-002/K17-E002** [SRC/E] (DEEP-K-002.discovery.md:L149 | SURFACE): `E/CAP` — Banks differentiator: ELAI MEASURES per-(model,lane) context-degradation knee against gate verdicts and runs OPPOSITE cloud vs local context policies. — plan-anchor=`K-17:L4-L5, L12-L13, L30-L30`
- **K-002/K17-E003** [SRC/E] (DEEP-K-002.discovery.md:L150 | SURFACE): `E/CFG` — Cloud regime: cache-stable prefix + late compaction; local regime: window-minimization + unload-early. — plan-anchor=`K-17:L4-L5, L30-L30`
- **K-002/K17-E004** [SRC/E] (DEEP-K-002.discovery.md:L151 | SURFACE): `E/CFG` — Folklore window fractions (0.75 local / 0.85 cloud) and slot ORDER are replaced by a measured knee (after J-34). — plan-anchor=`K-17:L4-L5, L13-L13, L17-L17`
- **K-002/K17-E005** [SRC/E] (DEEP-K-002.discovery.md:L152): `E/DATA` — K-1 dossier row tagged `[DESIGN]`/`[BENCH-PENDING]` with context_budget.rs + context_policy.rs code basis; no fabricated number (KD-1). — plan-anchor=`K-17:L30-L30, L38-L38, L60-L60, L72-L72`
- **K-002/K17-E006** [SRC/E] (DEEP-K-002.discovery.md:L153 | SURFACE): `E/SURFACE` — Claims-policy entry forces “best/optimal/never-loses context management” strings to carry measured-or-[DESIGN] qualifier + regime-split correction. — plan-anchor=`K-17:L30-L30, L32-L32`
- **K-002/K17-E007** [SRC/E] (DEEP-K-002.discovery.md:L154 | SURFACE): `E/SURFACE` — K-7 brief line for AppSec/platform “how do you keep the agent on-context across a long run”. — plan-anchor=`K-17:L30-L30, L92-L92`
- **K-002/K17-E008** [SRC/E] (DEEP-K-002.discovery.md:L155): `E/LIMIT` — Does NOT author J-34 or IMP-2..IMP-8; does NOT build a second context bench; does NOT fork context_budget/context_policy; does NOT re-state CL-1..CL-4. — plan-anchor=`K-17:L5-L5, L49-L54, L76-L76`
- **K-002/K17-E009** [SRC/E] (DEEP-K-002.discovery.md:L156 | SURFACE): `E/DEP` — Number source is J-34 `elai bench context-fill` (BENCH-PENDING); measured lift waits on it. — plan-anchor=`K-17:L1-L1, L8-L8, L18-L18, L41-L41, L74-L74`
- **K-002/K17-E010** [SRC/E] (DEEP-K-002.discovery.md:L157): `E/STATUS` — WIRED code basis: context_budget.rs 10 slots (8 + Memory H-31 + Evidence H-53) + floors/steal + 2 cache breakpoints + 3-level adaptive reduction. — plan-anchor=`K-17:L16-L16, L97-L97`
- **K-002/K17-E011** [SRC/E] (DEEP-K-002.discovery.md:L158 | SURFACE): `E/STATUS` — WIRED regime policy: context_policy.rs Local=floor(window*0.75) / Cloud=floor(window*0.85), `ELAI_CONTEXT_POLICY_DISABLE`, h50_context_policy_contract.rs; effective_context_window(Local) reads `ELAI_LOCAL_MAX_CONTEXT`. — plan-anchor=`K-17:L17-L17, L97-L97`
- **K-002/K17-E012** [SRC/E] (DEEP-K-002.discovery.md:L159): `E/DEP` — Cites CL-1..CL-4 as binding discipline; ships NO new rule. — plan-anchor=`K-17:L20-L20, L31-L31, L43-L43`
- **K-002/K17-E013** [SRC/E] (DEEP-K-002.discovery.md:L160 | SURFACE): `E/INT` — EXTEND (not fork) K-1 claims_policy_lint.py recogniser for context-management claims and context-efficiency numbers without ELAI-measured journal pointer. — plan-anchor=`K-17:L32-L32, L39-L39, L44-L44, L90-L90`
- **K-002/K17-E014** [SRC/E] (DEEP-K-002.discovery.md:L161 | SURFACE): `E/FAIL` — Negative fixture fails: (a) “ELAI never loses context” untagged; (b) bare “0% degradation at full window” without measured pointer; (c) RULER/Chroma frontier knee as ELAI local-lane knee. — plan-anchor=`K-17:L33-L33, L66-L67, L86-L86`
- **K-002/K17-E015** [SRC/E] (DEEP-K-002.discovery.md:L162 | SURFACE): `E/INT` — Production reachability: auditor runs claims_policy_lint.py over positioning copy; dossier row points at J-34 knee once measured; test-only never satisfies R7. — plan-anchor=`K-17:L45-L48`
- **K-002/K17-E016** [SRC/E] (DEEP-K-002.discovery.md:L163): `E/EVENT` — expect_ssb: none; measured number is [BENCH-PENDING] pointer to J-34 ContextCalibrationEvent. — plan-anchor=`K-17:L58-L59, L82-L82`
- **K-002/K17-E017** [SRC/E] (DEEP-K-002.discovery.md:L164 | SURFACE): `E/SURFACE` — frontend_pair: ContextStatePanel (DARK E-23 context telemetry) once mounted + H-9 cost cockpit (knee vs $/verified-task); else dossier/brief artifact; justified N/A (W-6). — plan-anchor=`K-17:L55-L56, L80-L80`
- **K-002/K17-E018** [SRC/E] (DEEP-K-002.discovery.md:L165): `E/LIMIT` — KD-4: do NOT headline bare “we manage context/compact/cache” (provider-absorbed); headline measured knee + regime split + journal-recomputable receipt. — plan-anchor=`K-17:L12-L13, L83-L83, L102-L103`
- **K-002/K17-E019** [SRC/E] (DEEP-K-002.discovery.md:L166): `E/LIMIT` — ID-5: any knee is ELAI-measured per-(model,lane), never RULER/Chroma curve. — plan-anchor=`K-17:L5-L5, L74-L74, L98-L98`
- **K-002/K17-E020** [SRC/E] (DEEP-K-002.discovery.md:L167): `E/TEST` — Lint extension tests: tagged differentiator passes; three negative arms fail; empty input fails-loud (R1). — plan-anchor=`K-17:L85-L86`
- **K-002/K17-E021** [SRC/E] (DEEP-K-002.discovery.md:L168 | SURFACE): `E/DATA` — Files: dossier row; claims_policy_lint ~13 LOC; K-7 brief +1 line; `_fixtures/context_management_negative.md`; `K-17-decisions.md`. — plan-anchor=`K-17:L88-L93`
- **K-002/K17-E022** [SRC/E] (DEEP-K-002.discovery.md:L169): `E/DEFER` — OUT-OF-K IMP-1..IMP-8 context-activation track (J-34 build, adaptive_reduce wire, BeliefState condense, pristine injector, blast-radius+retrieval, rotation pool, directive-supervisor, vLLM) recorded not authored. — plan-anchor=`K-17:L105-L105`
- **K-002/K17-E023** [SRC/E] (DEEP-K-002.discovery.md:L170): `E/DEP` — Soft-depends K-1, K-7; sibling of K-15 (reasoning-budget PROVE+PACKAGE shape). — plan-anchor=`K-17:L8-L9, L21-L21`
- **K-002/K17-E024** [SRC/E] (DEEP-K-002.discovery.md:L171 | SURFACE): `E/CFG` — W-2 N/A: cited context_policy already carries `ELAI_CONTEXT_POLICY_DISABLE`. — plan-anchor=`K-17:L80-L80`
- **K-002/K17-I001** [SRC/I] (DEEP-K-002.discovery.md:L172): `I/OBS` — Once J-34 lands, promotion path is [BENCH-PENDING] -> measured without a second claim machinery. — plan-anchor=`K-17:L47-L47, L60-L60`
- **K-002/K17-A001** [SRC/A] (DEEP-K-002.discovery.md:L173): `A/OBS` — Exact measured knee magnitude/threshold remains unknown until J-34 runs. — plan-anchor=`K-17:L5-L5, L102-L102`
- **K-002/K17-A002** [SRC/A] (DEEP-K-002.discovery.md:L174): `A/INT` — Lint recogniser overlap with K-15 must compose as one recogniser/multiple shapes, not forks. — plan-anchor=`K-17:L104-L104`

#### Unit `K18` — 29 atoms (A=3, E=25, I=1)

- **K-002/K18-E001** [SRC/E] (DEEP-K-002.discovery.md:L184): `E/STATUS` — K-18 is PLANNED WAVE KW-3 PROVABLE+PACKAGE BUILD-instrument + dossier/brief; VCG J-track done-gate; +0..1 SSB prefer-reuse; effort ~1 session. — plan-anchor=`K-18:L1-L3, L7-L7`
- **K-002/K18-E002** [SRC/E] (DEEP-K-002.discovery.md:L185 | SURFACE): `E/SURFACE` — CLI verb: `elai bench case-reuse --suite default --seed N --json` as real production bench entrypoint. — plan-anchor=`K-18:L5-L5, L30-L30, L49-L49, L61-L61`
- **K-002/K18-E003** [SRC/E] (DEEP-K-002.discovery.md:L186 | SURFACE): `E/CAP` — Recurrence-hit-rate arm (SAFE, ships now): held-out corpus from F-4-proven case rows; measures how often NEW failing task matches existing trigger_signature over BM25+dual-vector+reranker+RRF; ELAI_SEED-recomputable. — plan-anchor=`K-18:L5-L5, L30-L30, L76-L76`
- **K-002/K18-E004** [SRC/E] (DEEP-K-002.discovery.md:L187): `E/CAP` — Reuse-ON-vs-OFF economics arm (BENCH-PENDING on VCG J-track): ON injects top-matched F-4-PROVEN case as demonstration; OFF = cold; reports iteration-count delta + $/verified-task delta per (bug-class, model). — plan-anchor=`K-18:L5-L5, L31-L31, L76-L76`
- **K-002/K18-E005** [SRC/E] (DEEP-K-002.discovery.md:L188): `E/OBS` — Y-axis is verified-OUTCOME receipts, never pass@k or self-report (GD-3); no LLM-judge scoring (TD-3). — plan-anchor=`K-18:L5-L5, L31-L31, L54-L54, L56-L56`
- **K-002/K18-E006** [SRC/E] (DEEP-K-002.discovery.md:L189): `E/FLOW` — Observe-first: recurrence-hit-rate proves graph WOULD be useful with ZERO reuse-into-generation (GD-4 Slice-1). — plan-anchor=`K-18:L5-L5, L13-L13, L30-L30, L78-L78`
- **K-002/K18-E007** [SRC/E] (DEEP-K-002.discovery.md:L190): `E/DEP` — Reuse arm OBEYS K-19 execution-grounded reuse-fitness (held-out pass); never “it was retrieved” or LLM-judged-helpful. — plan-anchor=`K-18:L5-L5, L31-L31, L55-L55, L78-L78, L112-L112`
- **K-002/K18-E008** [SRC/E] (DEEP-K-002.discovery.md:L191 | SURFACE): `E/CFG` — Reuse-into-generation path default-OFF behind wire-dark-allowlist row + promotion plan (W-2/GD-4). — plan-anchor=`K-18:L57-L57, L84-L84`
- **K-002/K18-E009** [SRC/E] (DEEP-K-002.discovery.md:L192): `E/DATA` — Differentiator: “per-customer compounding state / gets faster-cheaper on YOUR recurring bugs over time” as K-1 `[DESIGN]`/`[BENCH-PENDING]` row + claims-policy entry + K-7 brief line. — plan-anchor=`K-18:L32-L32, L40-L40`
- **K-002/K18-E010** [SRC/E] (DEEP-K-002.discovery.md:L193 | SURFACE): `E/SURFACE` — Claims lint extension fails untagged “self-improving / learns from your codebase / gets better with use” and bare compounding numbers without journal pointer. — plan-anchor=`K-18:L33-L34, L41-L41, L97-L97`
- **K-002/K18-E011** [SRC/E] (DEEP-K-002.discovery.md:L194): `E/FAIL` — Negative fixture: (a) untagged learns/improves; (b) bare “50% fewer iterations”; (c) self-graded LLM-helpful fitness design (K-19 catches (c)). — plan-anchor=`K-18:L34-L34, L71-L71, L91-L91`
- **K-002/K18-E012** [SRC/E] (DEEP-K-002.discovery.md:L195 | SURFACE): `E/CAP` — Produces `benchmarks/case_reuse_runner.rs` + `case-reuse` bench sub at args_bench.rs:75; composes I-27 ConvergenceCurveMeasured + safedelta runner shape. — plan-anchor=`K-18:L38-L39, L43-L44, L94-L95`
- **K-002/K18-E013** [SRC/E] (DEEP-K-002.discovery.md:L196 | SURFACE): `E/EVENT` — Prefer-reuse bench row family / ConvergenceCurveMeasured (+Kind CaseReuseArm / RecurrenceHit); net 0..+1 from LIVE ~267/577 (dated); decide at edit W-9. — plan-anchor=`K-18:L61-L62, L87-L87`
- **K-002/K18-E014** [SRC/E] (DEEP-K-002.discovery.md:L197 | SURFACE): `E/SURFACE` — frontend_pair: recurrence-hit-rate + iteration/$ delta on I-27 ConvergencePanel + H-9 CloudPanel; optional VCG canvas case-altitude when shipped; decide at edit W-4 / justified N/A. — plan-anchor=`K-18:L58-L59, L85-L85`
- **K-002/K18-E015** [SRC/E] (DEEP-K-002.discovery.md:L198 | SURFACE): `E/INT` — Production: real CLI bench + auditor lint; contracts never satisfy R7. — plan-anchor=`K-18:L48-L51`
- **K-002/K18-E016** [SRC/E] (DEEP-K-002.discovery.md:L199): `E/DEP` — Case-birth substrate: F-4 red->green stdout_hash mint; G-2 RRA sealed payload; failure_taxonomy.py trigger_signature; B-29 blast_radius_cone. — plan-anchor=`K-18:L17-L17, L45-L45`
- **K-002/K18-E017** [SRC/E] (DEEP-K-002.discovery.md:L200 | SURFACE): `E/DEP` — Does NOT author VCG capability (mint/wire/canvas/reuse-loop); reuse-ON arm BENCH-PENDING on it. — plan-anchor=`K-18:L5-L5, L16-L16, L57-L57, L80-L80`
- **K-002/K18-E018** [SRC/E] (DEEP-K-002.discovery.md:L201): `E/LIMIT` — No vendor/paper number as ELAI’s (ID-5); no fork of I-27/G-6/case-base/K-1 lint; no main-LLM self-run of lint. — plan-anchor=`K-18:L52-L56`
- **K-002/K18-E019** [SRC/E] (DEEP-K-002.discovery.md:L202): `E/TEST` — Done = recurrence-hit-rate + FIXTURE arms actually run and produce recomputable numbers AND lint negative fails — not “runner compiles”. — plan-anchor=`K-18:L83-L83, L76-L76`
- **K-002/K18-E020** [SRC/E] (DEEP-K-002.discovery.md:L203): `E/TEST` — Determinism test: recurrence-hit-rate byte-identical under fixed seed; lint tests include empty-input fail-loud. — plan-anchor=`K-18:L90-L91`
- **K-002/K18-E021** [SRC/E] (DEEP-K-002.discovery.md:L204 | SURFACE): `E/DATA` — Files: case_reuse_runner.rs; args_bench.rs; dossier row; claims_policy_lint ~14 LOC; K-7 line; compounding_state_negative.md; K-18-decisions.md; optional SSB. — plan-anchor=`K-18:L93-L101`
- **K-002/K18-E022** [SRC/E] (DEEP-K-002.discovery.md:L205 | SURFACE): `E/DEFER` — OUT-OF-K VCG J-track: mint-on-proof case node; deterministic self-wiring edges; MEMORY/CASE canvas altitude; gated retrieval reuse; episodic->skill->rule ladder. — plan-anchor=`K-18:L114-L114`
- **K-002/K18-E023** [SRC/E] (DEEP-K-002.discovery.md:L206): `E/EVIDENCE` — Moat ranking premise: #2 per-customer compounding state has no PROVE leg in K until K-18; industry publishes no production number for agent improving on your codebase. — plan-anchor=`K-18:L5-L5, L12-L13, L105-L105`
- **K-002/K18-E024** [SRC/E] (DEEP-K-002.discovery.md:L207): `E/DEP` — Soft-depends K-1/K-7; gated by K-19; sibling of K-15. — plan-anchor=`K-18:L8-L9`
- **K-002/K18-E025** [SRC/E] (DEEP-K-002.discovery.md:L208): `E/OBS` — Live local-lane run is RunPod/remote-sidecar-gated (W-10). — plan-anchor=`K-18:L7-L7, L61-L61, L85-L85`
- **K-002/K18-I001** [SRC/I] (DEEP-K-002.discovery.md:L209): `I/STATE` — FIXTURE tier shipping now is the only always-on measured arm until VCG enables reuse-ON. — plan-anchor=`K-18:L7-L7, L30-L31`
- **K-002/K18-A001** [SRC/A] (DEEP-K-002.discovery.md:L210): `A/EVENT` — Exact prefer-reuse Kind vs new family decided at edit against live ~267/577 baseline. — plan-anchor=`K-18:L61-L62, L87-L87`
- **K-002/K18-A002** [SRC/A] (DEEP-K-002.discovery.md:L211 | SURFACE): `A/SURFACE` — Whether a dedicated case-reuse tile is required vs riding existing panels is edit-time. — plan-anchor=`K-18:L58-L59`
- **K-002/K18-A003** [SRC/A] (DEEP-K-002.discovery.md:L212): `A/INT` — Lint recogniser overlaps K-19/K-15/K-17 and must compose, not fork. — plan-anchor=`K-18:L113-L113`

#### Unit `K22` — 31 atoms (A=2, E=28, I=1)

- **K-002/K22-E001** [SRC/E] (DEEP-K-002.discovery.md:L222): `E/STATUS` — K-22 is PLANNED WAVE KW-3 PROVABLE+DISCIPLINE audit+deterministic tool K-8-shaped; NO SSB; effort ~1 session (~4-5 h). — plan-anchor=`K-22:L1-L3, L7-L7`
- **K-002/K22-E002** [SRC/E] (DEEP-K-002.discovery.md:L223): `E/CAP` — Build per-clause ENFORCEMENT-COVERAGE LEDGER tagging each doctrine clause {FROZEN-CHECKER / AUDITOR-ADVISORY / UNENFORCED} + coverage metric. — plan-anchor=`K-22:L4-L4, L31-L32`
- **K-002/K22-E003** [SRC/E] (DEEP-K-002.discovery.md:L224 | SURFACE): `E/SURFACE` — Tool: `.claude/tools/doctrine_coverage.py` stdlib-only deterministic `--json`; extracts clause IDs from CLAUDE.md + `.claude/rules/*.md`. — plan-anchor=`K-22:L7-L7, L32-L32, L41-L41, K22:L100-L100`
- **K-002/K22-E004** [SRC/E] (DEEP-K-002.discovery.md:L225): `E/DATA` — Clause families: W/GD/HD/TD/RD/VD/RV/CL/ID/JD/KD/ND + per-file R-n/MR-n/JC-n/SF-n/RR-n/TZ-n. — plan-anchor=`K-22:L8-L8, L16-L16, L32-L32`
- **K-002/K22-E005** [SRC/E] (DEEP-K-002.discovery.md:L226): `E/FLOW` — Join clause to enforcer via gate-manifest (R8) + `.claude/tools/` inventory; tagging is name+manifest join (TD-3); LLM does not grade enforcement. — plan-anchor=`K-22:L32-L32, L62-L62`
- **K-002/K22-E006** [SRC/E] (DEEP-K-002.discovery.md:L227): `E/STATE` — FROZEN-CHECKER iff a manifest-frozen deterministic tool enforces it; AUDITOR-ADVISORY = auditor-applied no frozen check; UNENFORCED = prose only. — plan-anchor=`K-22:L4-L4, L17-L17, L32-L32`
- **K-002/K22-E007** [SRC/E] (DEEP-K-002.discovery.md:L228): `E/FAIL` — UNKNOWN/unmappable clause is UNENFORCED, never silently FROZEN-CHECKER (R1/ND-3). — plan-anchor=`K-22:L32-L32, L59-L59, L85-L85`
- **K-002/K22-E008** [SRC/E] (DEEP-K-002.discovery.md:L229): `E/OBS` — Emit coverage metric: % frozen-checker-enforced, % advisory, % unenforced; recomputable from rulebook+manifest (GD-2). — plan-anchor=`K-22:L32-L32, L67-L69, L84-L84`
- **K-002/K22-E009** [SRC/E] (DEEP-K-002.discovery.md:L230 | SURFACE): `E/SURFACE` — Ledger artifact: `docs/strategy/ELAI-DOCTRINE-ENFORCEMENT-COVERAGE.md` (or workflow/reasoning/) with per-clause table + coverage % + UNENFORCED compilation BACKLOG ranked by leverage. — plan-anchor=`K-22:L33-L33, L42-L42, L101-L101`
- **K-002/K22-E010** [SRC/E] (DEEP-K-002.discovery.md:L231): `E/SEC` — Backlog is user-confirmed PROPOSAL; never auto-build checkers (no-removal/no-silent-build). — plan-anchor=`K-22:L4-L4, L33-L33, L60-L60, L86-L86, L117-L117`
- **K-002/K22-E011** [SRC/E] (DEEP-K-002.discovery.md:L232 | SURFACE): `E/LIMIT` — Does NOT modify rulebook clauses (read+tag only); does NOT introduce runtime gate over .elai_cc; does NOT claim every clause can be compiled (Rice/halting). — plan-anchor=`K-22:L4-L4, L61-L61, L88-L88`
- **K-002/K22-E012** [SRC/E] (DEEP-K-002.discovery.md:L233 | SURFACE): `E/SURFACE` — Doctrine rule: `.claude/rules/doctrine-enforcement.md` (~25-30 lines); any rulebook edit DECLARES clause enforcement tag. — plan-anchor=`K-22:L34-L34, L43-L43, L102-L102`
- **K-002/K22-E013** [SRC/E] (DEEP-K-002.discovery.md:L234): `E/INT` — conditional-routing.md loads doctrine-enforcement.md on rulebook-editing tasks. — plan-anchor=`K-22:L34-L34, L53-L53, L78-L78, L103-L103`
- **K-002/K22-E014** [SRC/E] (DEEP-K-002.discovery.md:L235 | SURFACE): `E/SURFACE` — EXTEND claims_policy_lint so “fully enforced doctrine / every rule is checked / all deterministic gates” fails without measured coverage % + honest AUDITOR-ADVISORY/UNENFORCED split. — plan-anchor=`K-22:L35-L35, L44-L44, L104-L104`
- **K-002/K22-E015** [SRC/E] (DEEP-K-002.discovery.md:L236 | SURFACE): `E/FAIL` — Negative: (a) fixtures “every ELAI rule is a frozen deterministic gate” untagged fails lint; (b) clause asserting checker absent from manifest is tagged UNENFORCED and coverage reflects it. — plan-anchor=`K-22:L36-L36, L69-L69, L97-L97`
- **K-002/K22-E016** [SRC/E] (DEEP-K-002.discovery.md:L237 | SURFACE): `E/INT` — Production: auditor/harness runs doctrine_coverage.py; conditional-routing loads rule; lint over positioning copy; K-4 reads ledger for real vs advisory gates (KD-3). — plan-anchor=`K-22:L51-L55, L94-L94`
- **K-002/K22-E017** [SRC/E] (DEEP-K-002.discovery.md:L238): `E/EVENT` — expect_ssb: none (harness leg; JD-1 = tool runs + recomputable ledger). — plan-anchor=`K-22:L63-L63, L68-L68`
- **K-002/K22-E018** [SRC/E] (DEEP-K-002.discovery.md:L239 | SURFACE): `E/SURFACE` — frontend_pair: N/A — tool + ledger doc + thin rule; no dashboard surface; coverage tile later optional (W-6 justified). — plan-anchor=`K-22:L64-L65, K22:L91-L92`
- **K-002/K22-E019** [SRC/E] (DEEP-K-002.discovery.md:L240): `E/DEP` — Consumes gate_freeze_check.py + gate-manifest + checker inventory; composes J-70 DO-333 gate-side traceability; extends K-1 lint; mirrors K-8 audit+tool shape. — plan-anchor=`K-22:L17-L22, L45-L50`
- **K-002/K22-E020** [SRC/E] (DEEP-K-002.discovery.md:L241 | SURFACE): `E/DEP` — Checker inventory includes maintainability_check, production_caller_check, python_wire_check, traced_test_assert_check, wiring_contract_check, power_of_ten_check, claims_policy_lint, failure_taxonomy, cross_lang_diff, spec_link_check, elai_map_freshness. — plan-anchor=`K-22:L18-L18`
- **K-002/K22-E021** [SRC/E] (DEEP-K-002.discovery.md:L242): `E/TEST` — Tool tests: matching tool->FROZEN; auditor-only->ADVISORY; prose->UNENFORCED; fake self-assert->UNENFORCED; coverage deterministic; empty input fails-loud. — plan-anchor=`K-22:L96-L97`
- **K-002/K22-E022** [SRC/E] (DEEP-K-002.discovery.md:L243): `E/TEST` — W-1 done = tool RUNS and emits recomputable ledger + rule loads on live rulebook edit, not mere file existence. — plan-anchor=`K-22:L91-L91, L84-L88`
- **K-002/K22-E023** [SRC/E] (DEEP-K-002.discovery.md:L244 | SURFACE): `E/DATA` — Files: doctrine_coverage.py; ledger md; doctrine-enforcement.md; conditional-routing ~2 LOC; claims_policy_lint ~10 LOC; doctrine_coverage_negative.md; K-22-decisions.md. — plan-anchor=`K-22:L99-L106`
- **K-002/K22-E024** [SRC/E] (DEEP-K-002.discovery.md:L245): `E/LIMIT` — FROZEN-CHECKER is necessary-not-sufficient (“tool CLAIMS this clause”); deeper proof is J-70 objective->technique->evidence; state this limit in ledger. — plan-anchor=`K-22:L116-L116`
- **K-002/K22-E025** [SRC/E] (DEEP-K-002.discovery.md:L246): `E/DET` — Un-anchorable clause reported as UNANCHORED backlog row, never dropped silently (R1); stable clause id = file+clause-id or file+first-line-slug. — plan-anchor=`K-22:L115-L115`
- **K-002/K22-E026** [SRC/E] (DEEP-K-002.discovery.md:L247): `E/IDEM` — K-22 must tag its own doctrine-enforcement.md honestly as AUDITOR-ADVISORY/harness-load in first ledger run (eat-your-own-dogfood). — plan-anchor=`K-22:L118-L118`
- **K-002/K22-E027** [SRC/E] (DEEP-K-002.discovery.md:L248 | SURFACE): `E/EVIDENCE` — EXTERNAL ContextCov ~50% prose-only is design rationale only; actual ELAI coverage % is measured by the tool (KD-1). — plan-anchor=`K-22:L4-L4, L111-L111`
- **K-002/K22-E028** [SRC/E] (DEEP-K-002.discovery.md:L249): `E/DEFER` — OUT-OF-K: C9-PLANS feature-map mechanical half and J-70 gate-side matrix are separate J fills. — plan-anchor=`K-22:L119-L119`
- **K-002/K22-I001** [SRC/I] (DEEP-K-002.discovery.md:L250 | SURFACE): `I/OBS` — Coverage % becomes a KD-1-tagged measured-ELAI number only after the tool runs over the live rulebook. — plan-anchor=`K-22:L20-L20, L33-L33`
- **K-002/K22-A001** [SRC/A] (DEEP-K-002.discovery.md:L251): `A/DATA` — Exact clause-extraction heuristics for heterogeneous HARD RULES vs numbered families remain implementer-specified beyond stable-id requirement. — plan-anchor=`K-22:L115-L115`
- **K-002/K22-A002** [SRC/A] (DEEP-K-002.discovery.md:L252): `A/DATA` — Ledger path strategy vs workflow/reasoning alternative left open by “or”. — plan-anchor=`K-22:L33-L33, L76-L76`

#### Unit `KMB` — 36 atoms (A=1, C=2, E=32, I=1)

- **K-002/KMB-E001** [SRC/E] (DEEP-K-002.discovery.md:L262): `E/STATUS` — Master brief dated 2026-06-12; status APPROVED-DIRECTION, folder not yet authored at write time; purpose is single comprehensive handoff for K-plan folder authoring. — plan-anchor=`KMB:L1-L7`
- **K-002/KMB-E002** [SRC/E] (DEEP-K-002.discovery.md:L263): `E/CAP` — Strategy thesis (a)-(d): MEASURED trust number; PORTABLE open vendor-neutral evidence; CHEAP TO LIVE WITH composed friction budget; SOLD AS COMPLIANCE-READY EVIDENCE to org-side buyers. — plan-anchor=`KMB:L42-L59`
- **K-002/KMB-E003** [SRC/E] (DEEP-K-002.discovery.md:L264): `E/EVIDENCE` — Unfilled market gap: vendor-neutral cryptographic chain of custody prompt->context->permissions->tool calls->diff->tests->deploy with independent attestation; labs ship containment+auditability not proof. — plan-anchor=`KMB:L76-L82`
- **K-002/KMB-E004** [SRC/E] (DEEP-K-002.discovery.md:L265 | SURFACE): `E/COMP` — Documented procurement demands: Tennessee RFI traceability/audit/third-party assessment/data-flow/retention/kill-switch; OMB M-25-22 independent evaluation + reproducible results; EU AI Act GPAI from 2026-08-02; Art-50; PLD from 2026-12-09; insurance AI sublimits. — plan-anchor=`KMB:L86-L94`
- **K-002/KMB-E005** [SRC/E] (DEEP-K-002.discovery.md:L266 | SURFACE): `E/DEP` — Moat ranking: 1 distribution, 2 per-customer compounding state, 3 proprietary verification, 4 enterprise trust/compliance, 5 cost engineering (weakest). ELAI strongest at 3, building 2, weakest at 1. — plan-anchor=`KMB:L109-L113`
- **K-002/KMB-E006** [SRC/E] (DEEP-K-002.discovery.md:L267): `E/EVIDENCE` — Gate-tax EXTERNAL CodeAct ~-50% latency / >-60% tokens; adaptive gating beats gate-everything. — plan-anchor=`KMB:L114-L116`
- **K-002/KMB-E007** [SRC/E] (DEEP-K-002.discovery.md:L268): `E/DET` — Replay honesty: cloud byte-replay fragile; honest guarantee is forensic reproducibility; local lane can claim more (J-13). — plan-anchor=`KMB:L117-L121`
- **K-002/KMB-E008** [SRC/E] (DEEP-K-002.discovery.md:L269): `E/LIMIT` — Local autonomous floor ~24GB VRAM; ELAI 7B@12GB correct only as mechanical/privacy/latency lane (H-50). — plan-anchor=`KMB:L122-L127`
- **K-002/KMB-E009** [SRC/E] (DEEP-K-002.discovery.md:L270): `E/STATUS` — Section A execution pointers (sequence, not re-author): A-1 I-43; A-2 F-2b live; A-3 G-6; A-4 I-40 wrap; A-5 G-5 review-time study; A-6 H-7/I-25; A-7 J-9; A-8 J-28/J-29. — plan-anchor=`KMB:L187-L203`
- **K-002/KMB-E010** [SRC/E] (DEEP-K-002.discovery.md:L271 | SURFACE): `E/FLOW` — Order: A-1 -> (go) -> A-2 -> A-3; A-4 parallel after G-9 live; A-5 after K-3 or cockpit-only if K-3 slips; A-6..A-8 parallel-safe before K-2/K-9. — plan-anchor=`KMB:L201-L203`
- **K-002/KMB-E011** [SRC/E] (DEEP-K-002.discovery.md:L272): `E/CAP` — K-1 dossier + claims policy: no external claim without measured journal-recomputable number or EXTERNAL/DESIGN tag. — plan-anchor=`KMB:L208-L220`
- **K-002/KMB-E012** [SRC/E] (DEEP-K-002.discovery.md:L273 | SURFACE): `E/CAP` — K-2 `elai bench routing-economics` hybrid number; live cost leg needs A-8. — plan-anchor=`KMB:L222-L237`
- **K-002/KMB-E013** [SRC/E] (DEEP-K-002.discovery.md:L274 | SURFACE): `E/CAP` — K-3 PR/CI evidence emitter (distribution leg): check-runs + PR comments; UNVOUCHED no-evidence; closed-set from RRA; `--post-unvouched-as-clean` fails. — plan-anchor=`KMB:L239-L254`
- **K-002/KMB-E014** [SRC/E] (DEEP-K-002.discovery.md:L275 | SURFACE): `E/CAP` — K-4 composed gate-tax budget with locked ceiling; breach = release BLOCK; `elai bench gate-tax`. — plan-anchor=`KMB:L256-L270`
- **K-002/KMB-E015** [SRC/E] (DEEP-K-002.discovery.md:L276): `E/CAP` — K-5 risk-tiered GateIntensity policy table R8-frozen; LLM never picks tier (TD-3). — plan-anchor=`KMB:L272-L287`
- **K-002/KMB-E016** [SRC/E] (DEEP-K-002.discovery.md:L277 | SURFACE): `E/CAP` — K-6 `elai export compliance` bundle + mapping; Art-50 designed-to-support only; no new attestation machinery. — plan-anchor=`KMB:L289-L302`
- **K-002/KMB-E017** [SRC/E] (DEEP-K-002.discovery.md:L278): `E/CAP` — K-7 AppSec/insurance positioning brief; pure prose + K-1 numbers; no runtime/SSB. — plan-anchor=`KMB:L304-L310`
- **K-002/KMB-E018** [SRC/E] (DEEP-K-002.discovery.md:L279): `E/CAP` — K-8 absorbable-unit audit + KD-4 provider pre-gate; kill list is user-confirmed proposal. — plan-anchor=`KMB:L312-L325`
- **K-002/KMB-E019** [SRC/E] (DEEP-K-002.discovery.md:L280): `E/CAP` — K-9 quarterly model-refresh cadence; ADOPT/KEEP/RE-EVALUATE user-ratified. — plan-anchor=`KMB:L327-L341`
- **K-002/KMB-E020** [SRC/E] (DEEP-K-002.discovery.md:L281 | SURFACE): `E/CAP` — K-10 sequencing/cut doctrine: A-1..A-4+K-3 above everything; NO NEW PLAN LETTERS until those DONE; activation-over-authoring; canvas H-10..H-16 demoted behind K-3+A-5; I-31 last. — plan-anchor=`KMB:L343-L355`
- **K-002/KMB-E021** [SRC/E] (DEEP-K-002.discovery.md:L282 | SURFACE): `E/CFG` — Doctrine KD-1 number-before-claim; KD-2 meet-the-buyer-in-the-PR; KD-3 composed-friction ceiling; KD-4 provider-absorption pre-gate; KD-5 window-clock for wedge artifacts. — plan-anchor=`KMB:L365-L378`
- **K-002/KMB-E022** [SRC/E] (DEEP-K-002.discovery.md:L283): `E/LIMIT` — Non-goals: no IDE play; no app-generator; no SWE-bench-Verified headline; no org/SSO/RBAC in K; no gate-everything defaults; no local expansion until 24GB candidate passes K-9; no new efficiency without KD-4; no overclaim of determinism tiers. — plan-anchor=`KMB:L381-L394`
- **K-002/KMB-E023** [SRC/E] (DEEP-K-002.discovery.md:L284 | SURFACE): `E/RECOVERY` — Kill-switches: null/negative A-1/A-3 safety delta -> stop K-T2/K-T4 headline trust; K-4 tax untolerable -> aggressive collapse; GitHub ships crypto evidence -> A-4 pivots compatibility, K-3 survives; A-5 no review-time win -> cockpit compliance artifact, K-3 remains; no 2x3090 -> K-9 still runs, 24GB adoption deferred. — plan-anchor=`KMB:L397-L410`
- **K-002/KMB-E024** [SRC/E] (DEEP-K-002.discovery.md:L285): `E/FLOW` — Track map K-T1 PROVE; K-T2 WEDGE+DISTRIBUTION (A-4,K-3,A-5); K-T3 ADAPTIVE GATE; K-T4 BUYER-SIDE (K-6,K-7); K-T5 DEPRECIATION+REFRESH; K-T6 SEQUENCING. — plan-anchor=`KMB:L357-L363`
- **K-002/KMB-E025** [SRC/E] (DEEP-K-002.discovery.md:L286): `E/INT` — Authoring produces K-0_INDEX, K-SUMMARY_INDEX, K-1..K-N, `_grounding/K-GROUNDING-MAP.md`; mirror J-plan format. — plan-anchor=`KMB:L420-L422`
- **K-002/KMB-E026** [SRC/E] (DEEP-K-002.discovery.md:L287 | SURFACE): `E/DET` — W-7 verify-at-authoring list: live SCHEMA/TV; github_pr real state; G-5/H-52; safedelta/seccodeplt/bench_cmd; I-43 code status; wire-dark allowlist; J-28 UsageTurnEvent. — plan-anchor=`KMB:L423-L435`
- **K-002/KMB-E027** [SRC/E] (DEEP-K-002.discovery.md:L288): `E/EVENT` — SSB prefer-reuse before new variants; Pattern-2 + 9-tuple lockstep; bump from LIVE integers only. — plan-anchor=`KMB:L436-L438`
- **K-002/KMB-E028** [SRC/E] (DEEP-K-002.discovery.md:L289): `E/EVIDENCE` — Every external number in K-1/K-7 re-verified via WebSearch at authoring (house rule after bad-cite incidents). — plan-anchor=`KMB:L439-L441`
- **K-002/KMB-E029** [SRC/E] (DEEP-K-002.discovery.md:L290): `E/LIMIT` — Never amend old plan files (new files only); K-8 kill list is PROPOSAL; docs/ gitignored. — plan-anchor=`KMB:L445-L448`
- **K-002/KMB-E030** [SRC/E] (DEEP-K-002.discovery.md:L291 | SURFACE): `E/A` — Open questions: K-4 ceiling VALUE user call; K-3 GitHub-only vs GitLab; K-6 archive vs directory; A-5 study n/pilot framing; whether K-10 lives only in K-0 or also `.claude/rules/`; canvas demotion confirmation. — plan-anchor=`KMB:L455-L469`
- **K-002/KMB-E031** [SRC/E] (DEEP-K-002.discovery.md:L292 | SURFACE): `E/STATUS` — One-screen summary: prove, distribute, make affordable, package for buyer, install discipline; ~10 net-new sessions + 8 pointers + 5 doctrine + 5 kill-switches. — plan-anchor=`KMB:L472-L483`
- **K-002/KMB-E032** [SRC/E] (DEEP-K-002.discovery.md:L293): `E/EVIDENCE` — Problem ammunition list (USENIX/SWE-bench/reward-hacking/SpecBench/DORA/METR/vendor telemetry) for K dossier with cite-verify duty. — plan-anchor=`KMB:L158-L176`
- **K-002/KMB-C001** [SRC/C] (DEEP-K-002.discovery.md:L294): `C/STATUS` — Brief states I-43 “exists as PLANNED — confirm no code landed” and SCHEMA 234/543 bait; K-GROUNDING-MAP later flips I-43 SHIPPED and baseline 236/545 (KC-1/KC-5). Authors MUST prefer the map. — plan-anchor=`KMB:L426-L434 vs KGM-E002/KGM-E005`
- **K-002/KMB-C002** [SRC/C] (DEEP-K-002.discovery.md:L295): `C/STATUS` — Brief treats github_pr as “VERIFY live… rename caused stale FAIL”; map confirms FULL WIRED (KC-2). — plan-anchor=`KMB:L249-L251, L428-L429 vs KGM-E006`
- **K-002/KMB-I001** [SRC/I] (DEEP-K-002.discovery.md:L296): `I/DEP` — K-17/K-18/K-22 are later expansions beyond the brief’s original K-1..K-10 list; brief scope is the original wave framing, not the complete later K index. — plan-anchor=`KMB:L182-L185, L205-L355`
- **K-002/KMB-A001** [SRC/A] (DEEP-K-002.discovery.md:L297): `A/CFG` — Locked composed-friction ceiling value is user-set after first measurement; brief forbids inventing one. — plan-anchor=`KMB:L457-L458`

### K-002 cross-file interactions (namespaced K-002/IX-*)

- **K-002/IX-001** [IX] (DEEP-K-002.discovery.md:L305): E | K-GROUNDING-MAP is the W-7 authority that corrects K-MASTER-BRIEF anchors (SSB baseline, I-43 status, github_pr status, J-28 gate, A-4 status). | KGM-E001–E003, E005–E006, E016–E017, E024, E027; KMB-C001–C002
- **K-002/IX-002** [IX] (DEEP-K-002.discovery.md:L306): E | K-3 is the distribution renderer KC-2/KMB prescribe over KG-2 github_pr + KG-3 review/H-52 + KG-7 RRA/attestation. | K3-E002, E014; KGM-E006–E007, E011, E015; KMB-E013
- **K-002/IX-003** [IX] (DEEP-K-002.discovery.md:L307): E | K-3 does not wait on A-4; foreign-agent evidence is a later compose when wrap lands (KC-4 shared by map and K-3). | K3-E015; KGM-E017
- **K-002/IX-004** [IX] (DEEP-K-002.discovery.md:L308): E | K-6 offline-verify path is the same `elai verify-attestation` surface K-3 embeds as an offline-verify line in PR check-runs. | K3-E004–E005; K6-E010; KGM-E011
- **K-002/IX-005** [IX] (DEEP-K-002.discovery.md:L309): E | K-6 G-9 secret-scrub and redacted RRA composition is the publication privacy leg complementary to K-3’s public PR render. | K6-E003, E009; K3-E003
- **K-002/IX-006** [IX] (DEEP-K-002.discovery.md:L310): E | K-6 soft-depends K-1 measured numbers; K-17/K-18/K-22 all extend the same K-1 claims_policy_lint + dossier surface. | K6-E020; K17-E005–E006, E013; K18-E009–E010; K22-E014
- **K-002/IX-007** [IX] (DEEP-K-002.discovery.md:L311): E | K-17 packages J-34 bench numbers; forbids a parallel context bench; points measured events at J-34 ContextCalibrationEvent rather than a K SSB. | K17-E008–E009, E016
- **K-002/IX-008** [IX] (DEEP-K-002.discovery.md:L312): E | K-17 cites CL-1..CL-4 + WIRED context_budget/context_policy; K-18 cites F-4/G-2/B-29/I-27 substrate the same way (compose, never re-author). | K17-E010–E012; K18-E012, E016
- **K-002/IX-009** [IX] (DEEP-K-002.discovery.md:L313): E | K-18 reuse arm is gated by K-19 fitness discipline and VCG J-track landing; observe-first recurrence-hit ships without generation influence. | K18-E003–E008, E017
- **K-002/IX-010** [IX] (DEEP-K-002.discovery.md:L314): E | K-18 and master-brief moat #2 “per-customer compounding state” are the same open PROVE gap; K-18 is the measurement leg. | K18-E023; KMB-E005
- **K-002/IX-011** [IX] (DEEP-K-002.discovery.md:L315): E | K-22 ledger feeds K-4’s view of which gates are REAL frozen-checkers vs advisory (KD-3), composing with gate-tax. | K22-E016; KMB-E014, E021
- **K-002/IX-012** [IX] (DEEP-K-002.discovery.md:L316): E | K-22 mirrors K-8 audit+tool+proposal shape and composes J-70 gate-side DO-333 rather than forking it. | K22-E001, E019; KMB-E018
- **K-002/IX-013** [IX] (DEEP-K-002.discovery.md:L317): E | KD-1/KD-2/KD-5 from the brief bind K-3 (PR render + window clock), K-6 (compliance time-box), and K-17/K-18/K-22 claim tags. | KMB-E021; K3-E001, E012; K6-E001; K17-E005; K18-E009; K22-E014
- **K-002/IX-014** [IX] (DEEP-K-002.discovery.md:L318): E | External KW-1..KW-5 figures in the map are the verified ammunition the brief §2.4 and K-1/K-7 use; KW-6 remains pending. | KGM-E018–E023; KMB-E032, E028
- **K-002/IX-015** [IX] (DEEP-K-002.discovery.md:L319): E | SSB discipline is shared: map sets 236/545 sequential lockstep; K-3/K-6 may +1; K-17/K-22 force 0; K-18 prefer-reuse 0..+1 from a later live baseline. | KGM-E002–E003; K3-E020; K6-E013; K17-E016; K18-E013; K22-E017
- **K-002/IX-016** [IX] (DEEP-K-002.discovery.md:L320): E | User-facing buyer surfaces split: K-3 PR/GitHub check-run + dashboard deep view; K-6 offline compliance bundle + mapping doc; K-7 brief (out of this chunk’s numbered files but sequenced by brief). | K3-E005–E006, E012; K6-E002–E004; KMB-E016–E017
- **K-002/IX-017** [IX] (DEEP-K-002.discovery.md:L321): E | Operator harness surfaces: K-17/K-18 auditor claims lint; K-18 `elai bench case-reuse`; K-22 `doctrine_coverage.py` + doctrine-enforcement rule load. | K17-E013–E015; K18-E002, E010; K22-E003, E012–E013
- **K-002/IX-018** [IX] (DEEP-K-002.discovery.md:L322): I | If K-3 frontend_pair and K-6 optional compliance tile both land, both ride dashboard-v2 but remain distinct (GitHub state vs compliance export). | K3-E012; K6-E016
- **K-002/IX-019** [IX] (DEEP-K-002.discovery.md:L323): E | Master-brief kill-switch “GitHub ships crypto evidence” pivots A-4 but keeps K-3 rendering; matches K-3’s renderer-only boundary. | KMB-E023; K3-E008
- **K-002/IX-020** [IX] (DEEP-K-002.discovery.md:L324): E | Parallel-safety: K-3 // K-6 (and K-2/K-4/K-9) when files disjoint; K-17 // K-13..K-19; K-22 // K-8/K-13..K-21. | K3-E025; K6-E021; K17-E023; K22 source L9

### K-002 summary deltas (namespaced K-002/SD-*)

- **K-002/SD-KGM-001** [SD] (DEEP-K-002.discovery.md:L350): **ADD** full KG-1..KG-9 live anchors (I-43 CLI, github_pr file inventory, review/H-52, SAFEDELTA/QDELTA/SeCodePLT/RRA, 105 allowlist rows, args_bench sub list). | 11-K:L942-L944, L980 | KGM-E004–E013
- **K-002/SD-KGM-002** [SD] (DEEP-K-002.discovery.md:L351): **ADD/CORRECT** KC-1..KC-5 binding flips and DEAD 234/543 baseline. | 11-K:L41-L42, L944 | KGM-E002–E003, E014–E017, E027, C001
- **K-002/SD-KGM-003** [SD] (DEEP-K-002.discovery.md:L352): **ADD** KW-1..KW-6 exact external figures, dual 58%/43% recurrence distinction, and PENDING-VERIFY exclusion for KW-6. | 11-K:L59-L70, L980 | KGM-E018–E023, E028
- **K-002/SD-KGM-004** [SD] (DEEP-K-002.discovery.md:L353): **ADD** KS/DC design-doc corrections and map-over-brief anchor precedence. | 11-K:L942-L944 | KGM-E024–E027
- **K-002/SD-K3-001** [SD] (DEEP-K-002.discovery.md:L359): **ADD** exact `--emit-evidence` mode, production trigger string, and optional `PrEvidenceEmitted` Kind set. | 11-K:L105-L126 | K3-E009–E010, E022
- **K-002/SD-K3-002** [SD] (DEEP-K-002.discovery.md:L360): **ADD** closed-set RRA-manifest reader task split (~120 LOC) + renderer (~180-240 LOC) + evidence_emitter.rs path. | 11-K:L118-L122 | K3-E002–E003, E016
- **K-002/SD-K3-003** [SD] (DEEP-K-002.discovery.md:L361): **ADD** W-2 allowlist-if-gated clause, cross_lang_diff requirement, and 236/545 sequential net. | 11-K:L123-L126 | K3-E020, E023, E018
- **K-002/SD-K3-004** [SD] (DEEP-K-002.discovery.md:L362): **ADD** GitLab demand-gate deferral, A-4 non-blocking, parallel-safety, and open seam-stability question. | 11-K:L105-L126 | K3-E015, E021, E025, A001
- **K-002/SD-K3-005** [SD] (DEEP-K-002.discovery.md:L363): **QUALIFY** summary UI pair: source marks check-run status on github.svelte.ts as edit-time decision, not a fixed field contract. | 11-K:L124 | K3-E012, A003
- **K-002/SD-K6-001** [SD] (DEEP-K-002.discovery.md:L369): **ADD** exact CLI flags `--out`, offline-verify composition path, and ComplianceExported Kind triad. | 11-K:L180-L205 | K6-E002, E010, E013
- **K-002/SD-K6-002** [SD] (DEEP-K-002.discovery.md:L370): **ADD** E-38 posture-doc citation HARD RULE, 90-day staleness risk, and GTM-killed certified wording checks in DETERMINISTIC_CHECK. | 11-K:L194, L112 | K6-E006, E012, E019, A002
- **K-002/SD-K6-003** [SD] (DEEP-K-002.discovery.md:L371): **ADD** five-part contract arms including mapping no-certified test; soft K-1 dependency; parallel-safety. | 11-K:L180-L205 | K6-E017–E021
- **K-002/SD-K6-004** [SD] (DEEP-K-002.discovery.md:L372): **EXPAND** format ambiguity (directory recommended + optional archive) beyond summary’s “one bundle”. | 11-K:L185 | K6-A001
- **K-002/SD-K6-005** [SD] (DEEP-K-002.discovery.md:L373): **ADD** summary K-6.22 unredacted-seeded-secret arm is a summary elaboration; numbered source states secret-grep empty + missing-digest fail without that exact fixture wording. | 11-K:L205 | K6-E009, E011
- **K-002/SD-K17-001** [SD] (DEEP-K-002.discovery.md:L379): **ADD** exact folklore constants Local 0.75 / Cloud 0.85, `ELAI_CONTEXT_POLICY_DISABLE`, `ELAI_LOCAL_MAX_CONTEXT`, and 10-slot composition (Memory H-31 + Evidence H-53). | 11-K:L447-L452 | K17-E004, E010–E011
- **K-002/SD-K17-002** [SD] (DEEP-K-002.discovery.md:L380): **ADD** zero-SSB / ContextCalibrationEvent pointer, W-2 N/A reason, and ContextStatePanel/H-9 frontend pair. | 11-K:L456-L457 | K17-E016–E017, E024
- **K-002/SD-K17-003** [SD] (DEEP-K-002.discovery.md:L381): **ADD** full OUT-OF-K IMP-1..IMP-8 activation list and KD-4 anti-headline framing. | 11-K:L462 | K17-E018, E022
- **K-002/SD-K17-004** [SD] (DEEP-K-002.discovery.md:L382): **ADD** lint compose-with-K-15 requirement and empty-input fail-loud test. | 11-K:L455-L461 | K17-E013, E020, A002
- **K-002/SD-K17-005** [SD] (DEEP-K-002.discovery.md:L383): **ADD** dossier path `ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md` and fixture path `context_management_negative.md`. | 11-K:L444-L458 | K17-E021
- **K-002/SD-K18-001** [SD] (DEEP-K-002.discovery.md:L389): **ADD** exact CLI shape `--suite default --seed N --json`, retrieval stack BM25+dual-vector+reranker+RRF, and FIXTURE-tier-now sequencing. | 11-K:L468-L479 | K18-E002–E003, E006
- **K-002/SD-K18-002** [SD] (DEEP-K-002.discovery.md:L390): **ADD** wire-dark default-OFF for reuse-into-generation, K-19 SF fitness bans, and dated LIVE ~267/577 prefer-reuse note. | 11-K:L480-L489 | K18-E007–E008, E013
- **K-002/SD-K18-003** [SD] (DEEP-K-002.discovery.md:L391): **ADD** runner path `case_reuse_runner.rs`, ConvergencePanel/CloudPanel pair, and W-1 “must actually run” done definition. | 11-K:L485-L489 | K18-E012, E014, E019
- **K-002/SD-K18-004** [SD] (DEEP-K-002.discovery.md:L392): **ADD** full OUT-OF-K VCG mint/wire/canvas/reuse/promotion companion scope. | 11-K:L493 | K18-E022
- **K-002/SD-K18-005** [SD] (DEEP-K-002.discovery.md:L393): **ADD** lint/composes K-19/K-15/K-17 coordination and byte-identical seed determinism test. | 11-K:L490-L492 | K18-E010–E011, E020, A003
- **K-002/SD-K22-001** [SD] (DEEP-K-002.discovery.md:L399): **ADD** exact tool CLI `--json`, coverage triple percentages, UNANCHORED residual class, and self-tag of doctrine-enforcement.md as AUDITOR-ADVISORY. | 11-K:L585-L608 | K22-E003, E008, E025–E026
- **K-002/SD-K22-002** [SD] (DEEP-K-002.discovery.md:L400): **ADD** necessary-not-sufficient FROZEN-CHECKER limit (J-70 deeper proof) and checker inventory enumeration. | 11-K:L589, L601 | K22-E020, E024
- **K-002/SD-K22-003** [SD] (DEEP-K-002.discovery.md:L401): **ADD** K-4 consumer link, conditional-routing load, and W-1 JD-1 run-not-exist done gate. | 11-K:L602-L606 | K22-E013, E016, E022
- **K-002/SD-K22-004** [SD] (DEEP-K-002.discovery.md:L402): **ADD** OUT-OF-K C9-PLANS / J-70 companion split and ContextCov EXTERNAL rationale caveat. | 11-K:L608, L822 | K22-E027–E028
- **K-002/SD-K22-005** [SD] (DEEP-K-002.discovery.md:L403): **ADD** ledger path alternative workflow/reasoning and fixture `doctrine_coverage_negative.md`. | 11-K:L603 | K22-E009, E023, A002
- **K-002/SD-KMB-001** [SD] (DEEP-K-002.discovery.md:L409): **ADD** full strategy (a)-(d), procurement demands, absorption/moat ranking, kill-switches, non-goals, and KD-1..KD-5 text. | 11-K:L35-L45, L942-L943 | KMB-E002–E008, E021–E023
- **K-002/SD-KMB-002** [SD] (DEEP-K-002.discovery.md:L410): **ADD** Section A pointer order/gating and original K-1..K-10 deliverable contracts including K-10 no-new-letters. | 11-K:L35, L942 | KMB-E009–E020, E024
- **K-002/SD-KMB-003** [SD] (DEEP-K-002.discovery.md:L411): **ADD** authoring instructions 6.1-6.8, open questions §7, and one-screen summary. | 11-K:L943 | KMB-E025–E031
- **K-002/SD-KMB-004** [SD] (DEEP-K-002.discovery.md:L412): **FLAG** stale brief anchors superseded by grounding map (I-43, SCHEMA, github_pr). | 11-K:L944 | KMB-C001–C002; SD-KGM-002
- **K-002/SD-KMB-005** [SD] (DEEP-K-002.discovery.md:L413): **QUALIFY** brief as original-wave framing; K-17/K-18/K-22 are later letter expansions not specified in the brief’s K-1..K-10 list. | 11-K:L943 | KMB-I001

### K-002 ambiguity and conflict register (namespaced K-002/AC-*)

- **K-002/AC-001** [AC] (DEEP-K-002.discovery.md:L330): C | Master brief SSB 234/543 and I-43 PLANNED vs grounding map 236/545 and I-43 SHIPPED. | KMB-C001; KGM-E002, E005
- **K-002/AC-002** [AC] (DEEP-K-002.discovery.md:L331): C | Master brief github_pr verify-at-authoring caution vs map FULL WIRED. | KMB-C002; KGM-E006, E015
- **K-002/AC-003** [AC] (DEEP-K-002.discovery.md:L332): A | K-3 `PrEvidenceEmitted` vs reuse existing rows. | K3-A002
- **K-002/AC-004** [AC] (DEEP-K-002.discovery.md:L333): A | K-3 github.svelte.ts field shape and body_composer seam stability. | K3-A001, A003
- **K-002/AC-005** [AC] (DEEP-K-002.discovery.md:L334): A | K-6 bundle directory vs archive; compliance tile yes/no; E-38 staleness. | K6-A001–A003
- **K-002/AC-006** [AC] (DEEP-K-002.discovery.md:L335): A | K-17 measured knee magnitude pending J-34; lint compose with K-15. | K17-A001–A002
- **K-002/AC-007** [AC] (DEEP-K-002.discovery.md:L336): A | K-18 SSB Kind choice; panel vs new tile; lint compose with K-19/K-15/K-17. | K18-A001–A003
- **K-002/AC-008** [AC] (DEEP-K-002.discovery.md:L337): A | K-22 clause extraction heuristics; ledger path strategy vs workflow. | K22-A001–A002
- **K-002/AC-009** [AC] (DEEP-K-002.discovery.md:L338): A | Map PLAN-asserted review_surface symbol names and KW-5 table page. | KGM-A001–A002
- **K-002/AC-010** [AC] (DEEP-K-002.discovery.md:L339): A | Brief open questions: K-4 ceiling value; GitLab scope; K-10 rules-file placement; A-5 pilot n. | KMB-E030, A001
- **K-002/AC-011** [AC] (DEEP-K-002.discovery.md:L340): C | Brief original K-1..K-10 scope vs later K-17/K-18/K-22 files in this chunk (later expansions). | KMB-I001

## Ledger K-003 — all atomic records

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-K-003.discovery.md`
Extracted ID-bearing rows: **308**.

### K-003 source atoms by unit

#### Unit `K2` — 38 atoms (A=3, C=2, E=32, I=1)

- **K-003/K2-E001** [SRC/E] (DEEP-K-003.discovery.md:L36): `E/STATUS` — K-2 is PLANNED, wargame-refined 2026-07-11 (K-26 MOVE 6 refine-before-execute); fixture leg ships now; LIVE cost leg is RunPod/remote-sidecar-gated (W-10), no longer J-28-blocked. — plan-anchor=`K-2:L3-L5`
- **K-003/K2-E002** [SRC/E] (DEEP-K-003.discovery.md:L37): `E/EVIDENCE` — PROOF_SPINE feeds PS-4 (routing economics $/verified-task); the K-2 runner is the buildable spine unit per proof-spine ruling 2026-07-05 and K-26 MOVE 6. — plan-anchor=`K-2:L3-L3`
- **K-003/K2-E003** [SRC/E] (DEEP-K-003.discovery.md:L38 | SURFACE): `E/CAP` — Objective builds `elai bench routing-economics`: fixed task corpus through two arms (cloud-only vs H-7 architect/editor hybrid), reporting % steps offloaded, $ delta, and quality delta as verified-OUTCOME rate (NOT pass@k), under ELAI_SEED and journal-recomputable (GD-2). — plan-anchor=`K-2:L7-L7`
- **K-003/K2-E004** [SRC/E] (DEEP-K-003.discovery.md:L39): `E/LIMIT` — Does NOT fork I-27's verified-outcome-vs-compute harness (compose runner shape, KS-1); HERA/CodeAct figures are [DESIGN], never claimed as ELAI's (HD-1). — plan-anchor=`K-2:L7-L7, L25-L25`
- **K-003/K2-E005** [SRC/E] (DEEP-K-003.discovery.md:L40): `E/FAIL` — LIVE arm emits a real $ number only when a live-priced cloud-vs-hybrid run exists; else honest 'cost unavailable — live lane not funded', never a fabricated cost (R1). — plan-anchor=`K-2:L3-L7, L52-L54`
- **K-003/K2-E006** [SRC/E] (DEEP-K-003.discovery.md:L41): `E/STATUS` — Estimated effort ~1-1.5 sessions (~5-6 h). Deterministic FIXTURE tier in-process now; LIVE arm network-gated. — plan-anchor=`K-2:L9-L9`
- **K-003/K2-E007** [SRC/E] (DEEP-K-003.discovery.md:L42 | SURFACE): `E/DEP` — Pre-requisites: H-7 calibrated routing + I-25 router (A-6 — re-grep cloud_mode.rs CloudModeRouter + providers escalation_policy.rs); bench dispatcher at args_bench.rs; safedelta_runner / bench_qdelta runner shape; J-28 UsageTurnEvent (SHIPPED, schema_version 272). — plan-anchor=`K-2:L10-L10, L20-L23`
- **K-003/K2-E008** [SRC/E] (DEEP-K-003.discovery.md:L43): `E/CONC` — Parallel-safe with K-3, K-4, K-6, K-9 (disjoint files). — plan-anchor=`K-2:L11-L11`
- **K-003/K2-E009** [SRC/E] (DEEP-K-003.discovery.md:L44 | SURFACE): `E/STATUS` — J-28 token ledger is SHIPPED (commit fe069d0; UsageTurnEvent at schema_version 272, usage_ledger + `elai usage` + SpendAccumulator); original KC-3 blocker (no normalized per-lane capture) is DISCHARGED at the capture layer. — plan-anchor=`K-2:L17-L17, L23-L23`
- **K-003/K2-E010** [SRC/E] (DEEP-K-003.discovery.md:L45 | SURFACE): `E/DEP` — Remaining live-arm gate is infrastructural: live cloud lane priced off models.toml AND a local lane (RunPod/remote-sidecar, W-10) — neither funded in the K-26 window. — plan-anchor=`K-2:L17-L17`
- **K-003/K2-E011** [SRC/E] (DEEP-K-003.discovery.md:L46): `E/INT` — CloudModeRouter at cloud_mode.rs:265 `struct CloudModeRouter`, :284 `pub fn route(...)` uses RuntimeRoutingPolicy::evaluate; VERIFIED live 2026-07-11. — plan-anchor=`K-2:L20-L20`
- **K-003/K2-E012** [SRC/E] (DEEP-K-003.discovery.md:L47 | SURFACE): `E/INT` — Bench dispatcher is args_bench.rs:52-93 (`if sub ==` chain); known subs include qdelta/safedelta/gate-tax/case-reuse; K-2's original `:75` anchor is STALE. — plan-anchor=`K-2:L22-L22, L71-L71`
- **K-003/K2-E013** [SRC/E] (DEEP-K-003.discovery.md:L48): `E/EVENT` — SSB current max cited: RecurrenceHitRateMeasured=270, GateTaxMeasured=271, UsageTurnEvent=272; K-2's new beacon bumps to 273 (RECON_NEEDED-1 re-grep). — plan-anchor=`K-2:L24-L24, L48-L49`
- **K-003/K2-E014** [SRC/E] (DEEP-K-003.discovery.md:L49): `E/DATA` — Task corpus: N=4-6 representative coding tasks, each with deterministic verified-outcome oracle; content-keyed fixture (no network, byte-reproducible); mirror SAFEDELTA/SeCodePLT oracle shape. — plan-anchor=`K-2:L36-L38`
- **K-003/K2-E015** [SRC/E] (DEEP-K-003.discovery.md:L50): `E/CAP` — Runner at benchmarks/routing_economics_runner.rs (~250-320 LOC): two arms (force-cloud-everything vs H-7 hybrid via CloudModeRouter::route); per task records lane-per-step + tokens + cost + verified-outcome; aggregates %offloaded, $/verified-task, quality delta. — plan-anchor=`K-2:L40-L40`
- **K-003/K2-E016** [SRC/E] (DEEP-K-003.discovery.md:L51): `E/STATE` — RunnerStatus{Measured — plan-anchor=`RunnerPresent} split: dry/unfunded live lane reports RunnerPresent, never a fabricated number.`
- **K-003/K2-E017** [SRC/E] (DEEP-K-003.discovery.md:L52 | SURFACE): `E/SURFACE` — CLI: dispatcher arm `if sub == "routing-economics"` in args_bench.rs:52-93 + bench_routing_economics.rs handler + register in unknown-sub error list. — plan-anchor=`K-2:L44-L46, L76-L76`
- **K-003/K2-E018** [SRC/E] (DEEP-K-003.discovery.md:L53 | SURFACE): `E/SURFACE` — Production CLI verb: `elai bench routing-economics [--seed <u64>] [--json] [--force-cloud-everything]`. — plan-anchor=`K-2:L88-L88, L99-L99`
- **K-003/K2-E019** [SRC/E] (DEEP-K-003.discovery.md:L54): `E/EVENT` — SSB beacon +RoutingEconomicsMeasured flat action_type (mirror GateTaxMeasured schema 271: as_sql_str "routing_economics_measured"); emit via emit_beacon_and_rra so number is journal-recomputable (GD-2), NOT just blake3-pinned json. — plan-anchor=`K-2:L48-L49, L117-L117`
- **K-003/K2-E020** [SRC/E] (DEEP-K-003.discovery.md:L55): `E/CFG` — FORK-1: reuse existing bench/RouteDecisionEvent family vs +1 RoutingEconomicsMeasured; prefer reuse, never overload unrelated family. — plan-anchor=`K-2:L65-L65`
- **K-003/K2-E021** [SRC/E] (DEEP-K-003.discovery.md:L56 | SURFACE): `E/CFG` — FORK-2: live lane funded this window? YES -> Measured real $; NO (K-26 default) -> fixture-arm-only + live arm 'cost unavailable' (RunnerPresent). — plan-anchor=`K-2:L66-L66`
- **K-003/K2-E022** [SRC/E] (DEEP-K-003.discovery.md:L57 | SURFACE): `E/SURFACE` — FORK-3 (frontend, W-4): HF-vs-hybrid $/verified-task panel tile vs CLI-only; if still RunnerPresent defer tile (W-6 CLI-only default this window). — plan-anchor=`K-2:L67-L67, L96-L97`
- **K-003/K2-E023** [SRC/E] (DEEP-K-003.discovery.md:L58 | SURFACE): `E/TEST` — Negative arm `--force-cloud-everything`: %offloaded==0 and savings delta ~0; hybrid arm >0 on same fixture. — plan-anchor=`K-2:L56-L58, L145-L145`
- **K-003/K2-E024** [SRC/E] (DEEP-K-003.discovery.md:L59): `E/LIMIT` — stub_ban: no $ on live leg without funded lane; no fork of I-27; no claim of HERA/CodeAct as ELAI; no pass@k; no .unwrap()/.expect() on prod runner. — plan-anchor=`K-2:L90-L95`
- **K-003/K2-E025** [SRC/E] (DEEP-K-003.discovery.md:L60): `E/OBS` — expect_ssb: RoutingEconomicsMeasured row (or reused family) recomputable under seed via emit_beacon_and_rra; expect_artifact: one report per arm with %offloaded + $/verified-task + quality delta. — plan-anchor=`K-2:L98-L101`
- **K-003/K2-E026** [SRC/E] (DEEP-K-003.discovery.md:L61): `E/TEST` — Contract tests k2_routing_economics_contract.rs (~90 LOC): distinct %-offloaded; recompute from SSB; verified-outcome metric; live honesty 'cost unavailable'; force-cloud collapse. — plan-anchor=`K-2:L144-L145`
- **K-003/K2-E027** [SRC/E] (DEEP-K-003.discovery.md:L62): `E/INT` — PRODUCTION_HOOK: args_bench.rs:52-93 -> parse_bench_routing_economics_args -> CliAction -> bench_routing_economics.rs -> routing_economics_runner::run -> emit_beacon_and_rra; #[cfg(test)] never satisfies R7. — plan-anchor=`K-2:L75-L76, L88-L89`
- **K-003/K2-E028** [SRC/E] (DEEP-K-003.discovery.md:L63 | SURFACE): `E/TEST` — INTEGRATION_SMOKE: auditor drives real `elai bench routing-economics --seed 1234 --json --force-cloud-everything` then hybrid on full fixture; hybrid %offloaded strictly exceeds force-cloud; beacon rows; live cost 'cost unavailable'; same-seed byte-identical. Single-task happy-path does NOT satisfy R10. — plan-anchor=`K-2:L135-L136`
- **K-003/K2-E029** [SRC/E] (DEEP-K-003.discovery.md:L64): `E/FAIL` — ABORT-1 schema change outside flat-beacon; ABORT-2 router cannot express force-cloud vs hybrid; ABORT-3 fixture oracle needs network; ABORT-4 cargo OOM twice. — plan-anchor=`K-2:L138-L142`
- **K-003/K2-E030** [SRC/E] (DEEP-K-003.discovery.md:L65): `E/DATA` — Files: CREATE routing_economics_runner.rs, bench_routing_economics.rs, k2 contract, K-2-decisions.md; MODIFY args_bench.rs + optional SSB action_type/payloads/lock. — plan-anchor=`K-2:L147-L153`
- **K-003/K2-E031** [SRC/E] (DEEP-K-003.discovery.md:L66 | SURFACE): `E/EVIDENCE` — External figures HERA ~45.67% offload and CodeAct ~50%/-60% are [DESIGN]/[EXTERNAL]; ELAI delta is whatever the bench measures. — plan-anchor=`K-2:L15-L15, L156-L158`
- **K-003/K2-E032** [SRC/E] (DEEP-K-003.discovery.md:L67): `E/LIMIT` — Fixture number framed as PILOT/plumbing (KD-1), never "the" routing-savings number; live $ is the actual first-mover differentiator and stays RunnerPresent until W-10. — plan-anchor=`K-2:L161-L163`
- **K-003/K2-I001** [SRC/I] (DEEP-K-003.discovery.md:L68): `I/DEP` — Because J-28 is SHIPPED, any residual 'block live $ on J-28' reading is obsolete; the controlling gate is W-10 live-endpoint presence. — plan-anchor=`K-2:L17-L17, L52-L54, L115-L116`
- **K-003/K2-A001** [SRC/A] (DEEP-K-003.discovery.md:L69): `A/CFG` — Exact corpus size within N=4-6, oracle identity per task, and worst-case hold multipliers are settled at edit, not fixed in the plan. — plan-anchor=`K-2:L36-L38, L161-L162`
- **K-003/K2-A002** [SRC/A] (DEEP-K-003.discovery.md:L70): `A/EVENT` — FORK-1 leaves open whether RoutingEconomicsMeasured is net +1 or a reuse of an existing family. — plan-anchor=`K-2:L65-L65, L127-L127`
- **K-003/K2-A003** [SRC/A] (DEEP-K-003.discovery.md:L71 | SURFACE): `A/SURFACE` — FORK-3 leaves open whether an HF-vs-hybrid panel tile ships now or remains CLI-only. — plan-anchor=`K-2:L67-L67, L96-L97`
- **K-003/K2-C001** [SRC/C] (DEEP-K-003.discovery.md:L72): `C/DEP` — K-26 MOVE 6 correction: live-arm gate is W-10 infra, not J-28; summary-layer K-2.11 still states 'Block the live dollar leg on J-28' and is superseded by this source. — plan-anchor=`K-2:L3-L5, L17-L17, L52-L54`
- **K-003/K2-C002** [SRC/C] (DEEP-K-003.discovery.md:L73): `C/INT` — Summary cites args_bench.rs:75 seam; this source states the dispatcher chain is :52-93 and explicitly marks original :75 as STALE. — plan-anchor=`K-2:L22-L22, L44-L44, L71-L71`

#### Unit `K12` — 26 atoms (A=3, E=22, I=1)

- **K-003/K12-E001** [SRC/E] (DEEP-K-003.discovery.md:L83): `E/STATUS` — K-12 is PLANNED (WAVE KW-3; S-M; BUILD gate; EXTENSION of H-49/I-11/G-2/SessionDAG) — two deterministic gates over BUILT code. — plan-anchor=`K-12:L3-L3`
- **K-003/K12-E002** [SRC/E] (DEEP-K-003.discovery.md:L84): `E/CAP` — Two-layer context model: cryptographically lossless RECORD (append-only SSB; trim→stub→recover_stubs with blake3) under a lossy-but-reversible VIEW. — plan-anchor=`K-12:L5-L5`
- **K-003/K12-E003** [SRC/E] (DEEP-K-003.discovery.md:L85): `E/FAIL` — R1: check_lossless_compaction (compact.rs:545, H-49 RV-2) currently WARNs when unrecoverable_count>0; K-12 flips FAIL-CLOSED — refuse trim (or force SSB capture-hook first) when unrecoverable_count>0. — plan-anchor=`K-12:L5-L5, L17-L17, L28-L28`
- **K-003/K12-E004** [SRC/E] (DEEP-K-003.discovery.md:L86): `E/CAP` — R2: HANDOFF-FAITHFULNESS GATE — extend I-11 BeliefState with critical_atoms{kind, value, must_preserve}; after handoff / I-5 pristine-rebuild / trim, assert EVERY must_preserve atom present by exact string/symbol match against SSB; missing = FAIL. — plan-anchor=`K-12:L5-L5, L29-L31`
- **K-003/K12-E005** [SRC/E] (DEEP-K-003.discovery.md:L87): `E/LIMIT` — NO generative summarization (HD-2); NO ciphers (IC-3); exact-presence-check only — NEVER semantic similarity. — plan-anchor=`K-12:L5-L5, L47-L48`
- **K-003/K12-E006** [SRC/E] (DEEP-K-003.discovery.md:L88): `E/DEP` — GD-1: do NOT re-implement recover_stubs, BeliefState, or RRA verify — COMPOSE them. — plan-anchor=`K-12:L5-L5, L50-L50`
- **K-003/K12-E007** [SRC/E] (DEEP-K-003.discovery.md:L89): `E/STATUS` — Existing: session_dag.rs:1539 recover_stubs WIRED; compact.rs:545 WARN-only PARTIAL; belief_state.rs BUILT (map said PLANNED); pristine_path_filter.rs BUILT; G-2 verify_rra WIRED. — plan-anchor=`K-12:L15-L20`
- **K-003/K12-E008** [SRC/E] (DEEP-K-003.discovery.md:L90): `E/DATA` — CriticalAtom schema: Vec<CriticalAtom{kind: AtomKind, value: String, must_preserve: bool}> on BeliefState; orchestrator declares them (TD-3 — LLM never sets must_preserve). — plan-anchor=`K-12:L29-L29, L49-L49`
- **K-003/K12-E009** [SRC/E] (DEEP-K-003.discovery.md:L91): `E/INT` — Wire R1 into live compaction path; R2 into escalation_waterfall.rs:239 + pristine-rebuild seam. — plan-anchor=`K-12:L31-L31, L43-L44`
- **K-003/K12-E010** [SRC/E] (DEEP-K-003.discovery.md:L92 | SURFACE): `E/CFG` — Default-ON behind ELAI_LOSSLESS_GATE_DISABLE (W-2); OBSERVE-then-enforce (GD-4: bound FP rate over >=3 real runs before BLOCK-by-default). — plan-anchor=`K-12:L31-L31, L74-L74`
- **K-003/K12-E011** [SRC/E] (DEEP-K-003.discovery.md:L93): `E/CAP` — Produces: compact.rs fail-closed; belief_state.rs CriticalAtom; handoff_faithfulness.rs Critical-Atom-Recall gate. — plan-anchor=`K-12:L35-L38`
- **K-003/K12-E012** [SRC/E] (DEEP-K-003.discovery.md:L94): `E/EVENT` — expect_ssb: +1 HandoffFaithfulnessGated (Pattern-2; Kind: AtomVerified / AtomMissing / CompactionRefused) OR reuse BeliefStateEvent +Kind; baseline+0..1 from LIVE ~267/577 (re-grep W-9). — plan-anchor=`K-12:L55-L55`
- **K-003/K12-E013** [SRC/E] (DEEP-K-003.discovery.md:L95 | SURFACE): `E/SURFACE` — frontend_pair N/A justified — gates are runtime fail-closed checks; verdicts flow to SSB that existing context/trust panels already read. — plan-anchor=`K-12:L51-L52`
- **K-003/K12-E014** [SRC/E] (DEEP-K-003.discovery.md:L96): `E/OBS` — observable_effect: compaction that would drop un-archived message refused (Err not warn); NEGATIVE: handoff dropping bug-location atom MUST FAIL. — plan-anchor=`K-12:L53-L56`
- **K-003/K12-E015** [SRC/E] (DEEP-K-003.discovery.md:L97 | SURFACE): `E/TEST` — Contract k12_losslessness_gate_contract.rs: unrecoverable>0 returns Err; atom present PASSES; atom drop FAILS; paraphrase does NOT satisfy; recover_stubs hash-mismatch still StubRecoveryFailed; ELAI_LOSSLESS_GATE_DISABLE downgrades to observe. — plan-anchor=`K-12:L89-L90`
- **K-003/K12-E016** [SRC/E] (DEEP-K-003.discovery.md:L98): `E/SEC` — LLM never sets must_preserve and never adjudicates the gate (TD-3). — plan-anchor=`K-12:L72-L72`
- **K-003/K12-E017** [SRC/E] (DEEP-K-003.discovery.md:L99): `E/LIMIT` — NO re-implementation of recover_stubs/BeliefState/verify_rra; NO .unwrap on prod; blake3 not sha2. — plan-anchor=`K-12:L50-L50`
- **K-003/K12-E018** [SRC/E] (DEEP-K-003.discovery.md:L100): `E/STATUS` — MAP FIX at upsert: I-5/I-11/H-50 PLANNED→PARTIAL/BUILT (code + contracts exist). — plan-anchor=`K-12:L108-L108`
- **K-003/K12-E019** [SRC/E] (DEEP-K-003.discovery.md:L101 | SURFACE): `E/RECOVERY` — Honest residue: external side-effects stay surfaced from saga log (RV-4), never auto-reverted. — plan-anchor=`K-12:L28-L28`
- **K-003/K12-E020** [SRC/E] (DEEP-K-003.discovery.md:L102): `E/INT` — Production reachability: live_cli.rs → check_lossless_compaction; escalation_waterfall.rs:239 → handoff_faithfulness; contracts never satisfy R7. — plan-anchor=`K-12:L42-L45`
- **K-003/K12-E021** [SRC/E] (DEEP-K-003.discovery.md:L103 | SURFACE): `E/DET` — Gate verdict recomputable from SSB journal under ELAI_SEED (GD-2); atom gate works on action record not tokens (GD-5). — plan-anchor=`K-12:L86-L87`
- **K-003/K12-E022** [SRC/E] (DEEP-K-003.discovery.md:L104): `E/DATA` — Files: MODIFY compact.rs, belief_state.rs, escalation_waterfall.rs + live_cli; CREATE handoff_faithfulness.rs, k12 contract, K-12-decisions.md. — plan-anchor=`K-12:L92-L98`
- **K-003/K12-A001** [SRC/A] (DEEP-K-003.discovery.md:L105): `A/DATA` — Exact atom EXTRACTION source set (task/objective + DKG symbols + I-5 contrastive constraint + failing-test anchor) decided at authoring; OBSERVE-first for false-FAILs. — plan-anchor=`K-12:L106-L106`
- **K-003/K12-A002** [SRC/A] (DEEP-K-003.discovery.md:L106 | SURFACE): `A/CFG` — R1 interaction with H-50 local window-minimization: fail-close must force-archive before minimize trim without blocking legitimate minimization — verify against live trim path. — plan-anchor=`K-12:L107-L107`
- **K-003/K12-A003** [SRC/A] (DEEP-K-003.discovery.md:L107 | SURFACE): `A/CFG` — Default-ON behind disable flag AND observe-then-enforce: initial enforcement mode transition (observe vs hard BLOCK) is not a single concrete schedule beyond '>=3 real runs'. — plan-anchor=`K-12:L31-L31, L74-L74`
- **K-003/K12-I001** [SRC/I] (DEEP-K-003.discovery.md:L108 | SURFACE): `I/SURFACE` — Because frontend_pair is N/A-justified and verdicts already flow via SSB, operator-visible surface is existing context/trust panels rather than a new drawer. — plan-anchor=`K-12:L51-L52`

#### Unit `K13` — 24 atoms (A=1, E=22, I=1)

- **K-003/K13-E001** [SRC/E] (DEEP-K-003.discovery.md:L118): `E/STATUS` — K-13 is PLANNED (WAVE KW-3; E-23-class PACKAGE + PROVE; NO runtime code, NO SSB); reuses K-1 claims-policy lint. — plan-anchor=`K-13:L3-L3`
- **K-003/K13-E002** [SRC/E] (DEEP-K-003.discovery.md:L119 | SURFACE): `E/SURFACE` — Artifact 1 [PACKAGE/WI-5]: docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md translating ELAI trust spine onto OWASP LLM01-10 (2025) + Agentic ASI01-10 (2025), one cell per threat, tagged measured-vs-design with citation (KD-1). — plan-anchor=`K-13:L8-L11, L91-L96`
- **K-003/K13-E003** [SRC/E] (DEEP-K-003.discovery.md:L120 | SURFACE): `E/SURFACE` — Artifact 2 [PROVE/WI-4]: bind prompt-injection (LLM01/LLM05/ASI01) to G-6 SAFEDELTA AgentDojo gate-ON-vs-OFF delta as K-1 dossier row tagged [BENCH-PENDING] until run lands. — plan-anchor=`K-13:L11-L14, L97-L101`
- **K-003/K13-E004** [SRC/E] (DEEP-K-003.discovery.md:L121): `E/LIMIT` — Does NOT author injection HARDENING (WI-1/WI-2/WI-3), config-ingress admission (WI-6), run G-6 itself, fabricate injection number, or claim 'OWASP certified'/'fully covered'. — plan-anchor=`K-13:L14-L21`
- **K-003/K13-E005** [SRC/E] (DEEP-K-003.discovery.md:L122 | SURFACE): `E/DATA` — Each mapping cell = ELAI control(s) + code/plan citation + STATUS tag (WIRED-MEASURED — plan-anchor=`WIRED-DESIGN`
- **K-003/K13-E006** [SRC/E] (DEEP-K-003.discovery.md:L123): `E/LIMIT` — N/A boundary: model inversion/extraction/theft (LLM02-tail, old LLM10) N/A for single-user local; re-open under multi-seat via J-59 ('sequence J-59 BEFORE multi-seat'). — plan-anchor=`K-13:L58-L61, L94-L95`
- **K-003/K13-E007** [SRC/E] (DEEP-K-003.discovery.md:L124): `E/EVIDENCE` — Standout covered axes recorded (not new work): ASI04 MCP poisoning = G-4+E-24+C-42; ASI09 human-agent trust = F-4+H-53+G-5+K-3/K-11; ASI10 rogue agents = R8+H-43/H-44+L1 HardVeto; LLM03 supply chain = F-9+E-43+I-18+G-8; LLM09 misinformation = grounding spine. — plan-anchor=`K-13:L52-L56`
- **K-003/K13-E008** [SRC/E] (DEEP-K-003.discovery.md:L125): `E/DEP` — G-1 TAINT WIRED (LLM01/ASI01 data-plane); residual binding gap substring-not-transformation-surviving is WI-1 (OUT-OF-K). — plan-anchor=`K-13:L64-L65`
- **K-003/K13-E009** [SRC/E] (DEEP-K-003.discovery.md:L126 | SURFACE): `E/DEP` — E-37 PROMPT INJECTION DEFENSE PARTIAL (Spotlighting + PromptArmor; output_sanitizer/classifier/egress allowlist/AgentDojo-CI absent) — WI-3 completes perimeter OUT-OF-K. — plan-anchor=`K-13:L66-L67`
- **K-003/K13-E010** [SRC/E] (DEEP-K-003.discovery.md:L127): `E/DEP` — G-6 SAFEDELTA BENCH-PENDING (SusVibes/BaxBench/SeCodePLT/AgentDojo, gated by F-2b, 0 measured rows) — AgentDojo arm is injection-number source. — plan-anchor=`K-13:L72-L73`
- **K-003/K13-E011** [SRC/E] (DEEP-K-003.discovery.md:L128 | SURFACE): `E/INT` — EXTEND K-1 claims_policy_lint recogniser so OWASP cell asserting 'covered'/'protected' without measured number AND without [DESIGN] tag FAILS. ~15-30 LOC. — plan-anchor=`K-13:L102-L104`
- **K-003/K13-E012** [SRC/E] (DEEP-K-003.discovery.md:L129): `E/DEP` — COMPOSES/feeds K-6 (OWASP mapping becomes procurement clause) WITHOUT modifying K-6 plan file; pointer + feature-map cross-link. — plan-anchor=`K-13:L105-L107`
- **K-003/K13-E013** [SRC/E] (DEEP-K-003.discovery.md:L130 | SURFACE): `E/SURFACE` — Feeds K-7 AppSec/insurance brief (buyer 'OWASP coverage?' question). — plan-anchor=`K-13:L11-L11, L75-L75`
- **K-003/K13-E014** [SRC/E] (DEEP-K-003.discovery.md:L131): `E/TEST` — NEGATIVE arm fixtures: (a) 'fully covered' with no number/tag FAILS; (b) 'OWASP certified' superlative FAILS. — plan-anchor=`K-13:L108-L109, L146-L147`
- **K-003/K13-E015** [SRC/E] (DEEP-K-003.discovery.md:L132 | SURFACE): `E/OBS` — observable_effect trigger: `python .claude/tools/claims_policy_lint.py --files docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`; expect_ssb none (E-23-class). — plan-anchor=`K-13:L135-L140`
- **K-003/K13-E016** [SRC/E] (DEEP-K-003.discovery.md:L133 | SURFACE): `E/SURFACE` — frontend_pair N/A — positioning/procurement doc + CI/audit lint; no dashboard surface (W-6 justified). — plan-anchor=`K-13:L133-L134`
- **K-003/K13-E017** [SRC/E] (DEEP-K-003.discovery.md:L134): `E/EVIDENCE` — Re-verify OWASP LLM-2025 + Agentic-2025 at authoring from genai.owasp.org; NEVER cite stale v1.1 owasp.org landing list. — plan-anchor=`K-13:L27-L29, L159-L159`
- **K-003/K13-E018** [SRC/E] (DEEP-K-003.discovery.md:L135): `E/DATA` — LLM01..LLM10 and ASI01..ASI10 all 20 threats must have rows (rg check). — plan-anchor=`K-13:L148-L148, L155-L155`
- **K-003/K13-E019** [SRC/E] (DEEP-K-003.discovery.md:L136): `E/DEFER` — OUT-OF-K technical companions: WI-1 transformation-surviving taint; WI-2 cross-process taint; WI-3 E-37 perimeter completion; WI-6 config-ingress admission (E-43/E-24 extension). — plan-anchor=`K-13:L205-L211`
- **K-003/K13-E020** [SRC/E] (DEEP-K-003.discovery.md:L137 | SURFACE): `E/DATA` — Files: CREATE ELAI-OWASP-CONTROL-MAPPING.md, owasp_negative.md, K-13-decisions.md; MODIFY ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 row), claims_policy_lint.py. — plan-anchor=`K-13:L177-L182`
- **K-003/K13-E021** [SRC/E] (DEEP-K-003.discovery.md:L138): `E/TEST` — k13_owasp_claims_contract.rs or Python: tagged cell passes; untagged fully-covered fails; OWASP certified fails; [DESIGN] passes; empty fails-loud; [BENCH-PENDING] does not count as fabricated measured number. — plan-anchor=`K-13:L170-L175`
- **K-003/K13-E022** [SRC/E] (DEEP-K-003.discovery.md:L139 | SURFACE): `E/LIMIT` — Kill-switch-1 caveat from K-0 applies: NULL/NEGATIVE safety delta re-aims the wedge for the injection cell. — plan-anchor=`K-13:L201-L204`
- **K-003/K13-A001** [SRC/A] (DEEP-K-003.discovery.md:L140): `A/DATA` — Dossier-row home: row IN K-1 dossier vs K-13 OWASP section — recommend row in K-1 dossier (single number ledger); decide at authoring. — plan-anchor=`K-13:L198-L200`
- **K-003/K13-I001** [SRC/I] (DEEP-K-003.discovery.md:L141): `I/INT` — Because conditional-routing already loads claims policy when docs/strategy/ copy is edited (K-1), K-13's lint extension rides that existing harness path. — plan-anchor=`K-13:L123-L124`

#### Unit `K14` — 23 atoms (A=1, E=21, I=1)

- **K-003/K14-E001** [SRC/E] (DEEP-K-003.discovery.md:L151): `E/STATUS` — K-14 is PLANNED (WAVE KW-3; E-23 + doctrine class, K-8-shaped); a .claude/rules/ doctrine + claims-policy entry + dossier/brief rows; NO runtime code, NO SSB. — plan-anchor=`K-14:L3-L3`
- **K-003/K14-E002** [SRC/E] (DEEP-K-003.discovery.md:L152): `E/CAP` — Two K slivers: (1) DISCIPLINE — determinism invariant making wall-clock awareness coexist with byte-identical replay + cache spine; (2) PACKAGE/PROVE — bank IMP-9 replayable-async-tools differentiator as tagged claim. — plan-anchor=`K-14:L5-L21`
- **K-003/K14-E003** [SRC/E] (DEEP-K-003.discovery.md:L153): `E/LIMIT` — Does NOT author T-1..T-5 (timestamp/agent-time/H-51/async-loop/watchdog); does NOT claim MEASURED number for differentiator; does NOT introduce wall-clock into any deterministic gate (ND-1). — plan-anchor=`K-14:L16-L21`
- **K-003/K14-E004** [SRC/E] (DEEP-K-003.discovery.md:L154 | SURFACE): `E/CFG` — TZ-1 PLACEMENT: wall-clock/elapsed context block is VOLATILE and MUST sit in dedicated 3E slot STRICTLY AFTER the last cache breakpoint (never system/rules/PhaseSpec). — plan-anchor=`K-14:L53-L55`
- **K-003/K14-E005** [SRC/E] (DEEP-K-003.discovery.md:L155): `E/CFG` — TZ-2 SOURCE+JOURNAL: human-facing stamp from REAL OsClock (not SeededClock) AND recorded INTO SSB action-stream as 'what the model saw'; gate/journal timestamps keep ClockProvider/SeededClock. — plan-anchor=`K-14:L56-L59`
- **K-003/K14-E006** [SRC/E] (DEEP-K-003.discovery.md:L156): `E/CFG` — TZ-3 NEVER-A-GATE-INPUT: no rule predicate, FSM transition, or promotion gate may branch on time block or wall-clock (ND-1, J-66); judgment context only. — plan-anchor=`K-14:L60-L62`
- **K-003/K14-E007** [SRC/E] (DEEP-K-003.discovery.md:L157): `E/CFG` — TZ-4 LIVENESS-OFF-THE-REPLAY-SET: wall-clock watchdog kill is SUPERVISOR-PLANE outside seeded replay set — outcome RECORDED not gate-evaluated; PRIMARY bound is step/no-material-progress; wall-clock liveness defaults AUTONOMOUS-only (MASTER 412). — plan-anchor=`K-14:L63-L67`
- **K-003/K14-E008** [SRC/E] (DEEP-K-003.discovery.md:L158 | SURFACE): `E/SURFACE` — Author .claude/rules/time-determinism.md (~50-70 lines) + conditional-routing.md trigger for timestamp/elapsed/watchdog/async-tool tasks. — plan-anchor=`K-14:L94-L97`
- **K-003/K14-E009** [SRC/E] (DEEP-K-003.discovery.md:L159 | SURFACE): `E/SURFACE` — Differentiator claim: 'decoupled async tools whose lifecycle lands in a deterministic, replayable, taint-tagged journal' — K-1 dossier [DESIGN] row + claims-policy entry + K-7 brief line; honest partial-built status (G3-G6 wired; model-facing loop J-T4/T5). — plan-anchor=`K-14:L98-L104`
- **K-003/K14-E010** [SRC/E] (DEEP-K-003.discovery.md:L160 | SURFACE): `E/INT` — EXTEND K-1 claims_policy_lint so async tools/decoupled/replayable claim without [DESIGN] + partial-status FAILS. ~10-15 LOC. — plan-anchor=`K-14:L105-L106`
- **K-003/K14-E011** [SRC/E] (DEEP-K-003.discovery.md:L161): `E/TEST` — NEGATIVE arm: (a) 'fully deterministic async tools' untagged fails lint; (b) time block in system prompt / wall-clock in gate caught by rule (auditor-applied). — plan-anchor=`K-14:L107-L109`
- **K-003/K14-E012** [SRC/E] (DEEP-K-003.discovery.md:L162 | SURFACE): `E/DEP` — Consumes: ClockProvider/OsClock/SeededClock core/src/clock.rs:43/85/124; 3E allocator 8 slots + 2 cache breakpoints; RRA G-2; ND-1 J-66; WatchdogTimeout SSB; background_task_id + BackgroundTaskManager + BackgroundTaskStarted/Complete + AgentNotificationEmitted (H-36). — plan-anchor=`K-14:L69-L82, L118-L122`
- **K-003/K14-E013** [SRC/E] (DEEP-K-003.discovery.md:L163 | SURFACE): `E/SURFACE` — frontend_pair N/A — doctrine file + positioning/claims; no dashboard (W-6). — plan-anchor=`K-14:L134-L135`
- **K-003/K14-E014** [SRC/E] (DEEP-K-003.discovery.md:L164): `E/OBS` — expect_ssb: none (E-23/doctrine); expect_artifact: rule fires via conditional-routing; lint PASSES tagged claim and FAILS untagged over-claim. — plan-anchor=`K-14:L136-L139`
- **K-003/K14-E015** [SRC/E] (DEEP-K-003.discovery.md:L165): `E/DEFER` — OUT-OF-K technical companions T-1..T-5 (ConversationMessage.created_at slot; agent-time; H-51 resume header; model-facing async-tool loop; watchdog promotion) all BOUND by TZ-1..TZ-4. — plan-anchor=`K-14:L198-L205`
- **K-003/K14-E016** [SRC/E] (DEEP-K-003.discovery.md:L166 | SURFACE): `E/DATA` — Files: CREATE time-determinism.md, async_replay_negative.md, K-14-decisions.md; MODIFY conditional-routing.md, ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md (+1 row), claims_policy_lint.py. — plan-anchor=`K-14:L173-L179`
- **K-003/K14-E017** [SRC/E] (DEEP-K-003.discovery.md:L167 | SURFACE): `E/EVIDENCE` — arXiv:2510.23853 + 2601.13206 temporally-blind grounding [EXTERNAL]; never claimed as ELAI number. — plan-anchor=`K-14:L33-L33, L186-L187`
- **K-003/K14-E018** [SRC/E] (DEEP-K-003.discovery.md:L168): `E/TEST` — k14_async_replay_claims_contract or Python: [DESIGN] passes; untagged fully-deterministic fails; 'guaranteed replay' superlative fails; empty fails-loud. — plan-anchor=`K-14:L166-L171`
- **K-003/K14-A001** [SRC/A] (DEEP-K-003.discovery.md:L169): `A/DEFER` — Open user decisions D-2 (tz granularity), D-3 (watchdog default), D-4 (auto-background threshold) belong to J authoring pass, not K-14. — plan-anchor=`K-14:L204-L205`
- **K-003/K14-E019** [SRC/E] (DEEP-K-003.discovery.md:L170): `E/DEP` — Soft-depends K-1 (lint + dossier) and K-7 (brief); parallel-safe with K-8, K-9, K-13 (disjoint files). — plan-anchor=`K-14:L25-L30`
- **K-003/K14-E020** [SRC/E] (DEEP-K-003.discovery.md:L171): `E/TEST` — DETERMINISTIC_CHECK requires time-determinism.md exists, TZ-1..TZ-4 present, conditional-routing trigger wired, untagged over-claim lint exit nonzero, and zero 'fully deterministic async'/'guaranteed replay' in docs/strategy. — plan-anchor=`K-14:L141-L149`
- **K-003/K14-E021** [SRC/E] (DEEP-K-003.discovery.md:L172): `E/LIMIT` — Scope discipline: ships RULE + CLAIM only; temptation to author T-1 timestamp slot must be resisted — that is the technical J track. — plan-anchor=`K-14:L192-L194`
- **K-003/K14-I001** [SRC/I] (DEEP-K-003.discovery.md:L173): `I/DEP` — Because differentiator rests on G3-G6 already-built journal primitives, the claim is architectural honesty about partial exposure, not a greenfield build. — plan-anchor=`K-14:L77-L79, L195-L197`

#### Unit `K15` — 24 atoms (A=2, E=21, I=1)

- **K-003/K15-E001** [SRC/E] (DEEP-K-003.discovery.md:L183): `E/STATUS` — K-15 is PLANNED (WAVE KW-3; PROVABLE+PACKAGE; BUILD-instrument composing I-27/J-34 + dossier/brief rows); GRC-v3 done-gate; BENCH-PENDING on J GRC-v3 track; +0..1 SSB prefer-reuse. — plan-anchor=`K-15:L3-L3`
- **K-003/K15-E002** [SRC/E] (DEEP-K-003.discovery.md:L184 | SURFACE): `E/SURFACE` — Builds `elai bench reasoning-budget` — per-(model,task-type) verified-outcome-rate vs reasoning-compute curve under SAFEDELTA gate-ON-vs-OFF (GRC-v3 router ON vs flat-budget OFF). — plan-anchor=`K-15:L5-L5, L31-L31`
- **K-003/K15-E003** [SRC/E] (DEEP-K-003.discovery.md:L185): `E/CAP` — Finds per-(model,task-type) PEAK/KNEE that J-84 classifier and J-85 overthinking-guard ceiling track; local Qwen-7B overthinks EARLIEST/LOWEST — never transfer paper 6K/7K (ID-5). — plan-anchor=`K-15:L5-L5, L55-L56`
- **K-003/K15-E004** [SRC/E] (DEEP-K-003.discovery.md:L186 | SURFACE): `E/SURFACE` — Differentiator [PACKAGE/PROVE]: 'ELAI's reasoning engine grounds every reasoning step on an EXTERNAL receipt; the model never grades its own confidence' — K-1 dossier [DESIGN] + claims-policy + K-7 brief. — plan-anchor=`K-15:L5-L5, L33-L33`
- **K-003/K15-E005** [SRC/E] (DEEP-K-003.discovery.md:L187 | SURFACE): `E/LIMIT` — Does NOT author J-83..J-86; bench is BENCH-PENDING until J track lands; NO measured number for differentiator until bench runs; done-gate is ELAI-measured on gold set NEVER paper constant; does NOT re-implement I-27/J-34/H-9/G-6. — plan-anchor=`K-15:L5-L5`
- **K-003/K15-E006** [SRC/E] (DEEP-K-003.discovery.md:L188): `E/CFG` — Deterministic FIXTURE tier ships now; ON arm BENCH-PENDING on J-83/J-85; live local-lane run RunPod/remote-sidecar-gated (W-10). — plan-anchor=`K-15:L6-L6, L31-L31`
- **K-003/K15-E007** [SRC/E] (DEEP-K-003.discovery.md:L189 | SURFACE): `E/DEP` — Composes I-27 ConvergenceCurveMeasured + bench_convergence (DARK); J-34 `elai bench context-fill` sibling shape; safedelta_runner + args_bench.rs:75 seam; H-9 CloudPanel. — plan-anchor=`K-15:L7-L8, L15-L21`
- **K-003/K15-E008** [SRC/E] (DEEP-K-003.discovery.md:L190): `E/DATA` — Measured knee written to ModelCapabilityProfile-adjacent calibration artifact J-34 already defines (reuse), keyed by model identity, G-10-drift-invalidated. — plan-anchor=`K-15:L32-L32, L115-L115`
- **K-003/K15-E009** [SRC/E] (DEEP-K-003.discovery.md:L191): `E/INT` — EXTEND K-1 lint: 'reasoning grounds itself / self-verifying reasoning' without [DESIGN]+receipt-not-self-eval FAILS; reasoning-budget number without ELAI-measured journal pointer FAILS. — plan-anchor=`K-15:L34-L34`
- **K-003/K15-E010** [SRC/E] (DEEP-K-003.discovery.md:L192): `E/TEST` — NEGATIVE fixtures: (a) untagged 'ELAI's reasoning self-verifies'; (b) bare 'GRC-v3 lifts accuracy 20%' no pointer; (c) paper's 7K knee as ELAI's — all FAIL. — plan-anchor=`K-15:L35-L35`
- **K-003/K15-E011** [SRC/E] (DEEP-K-003.discovery.md:L193 | SURFACE): `E/SURFACE` — CLI: `elai bench reasoning-budget --suite default --seed N --json` (PRODUCTION bench). — plan-anchor=`K-15:L50-L51, L64-L64`
- **K-003/K15-E012** [SRC/E] (DEEP-K-003.discovery.md:L194): `E/EVENT` — prefer-reuse I-27 ConvergenceCurveMeasured (+Kind: ReasoningBudgetArm) — net 0..+1 from LIVE 268/577, decide at edit (W-9). — plan-anchor=`K-15:L65-L65`
- **K-003/K15-E013** [SRC/E] (DEEP-K-003.discovery.md:L195 | SURFACE): `E/SURFACE` — frontend_pair: curve + knee on I-27 ConvergencePanel + H-9 CloudPanel — reasoning-budget arm on existing convergence tile (decide at edit, W-4); justified N/A if rides those panels. — plan-anchor=`K-15:L61-L62`
- **K-003/K15-E014** [SRC/E] (DEEP-K-003.discovery.md:L196 | SURFACE): `E/OBS` — y-axis = receipts/verified-OUTCOME, NOT self-reported accuracy or pass@k (GD-3); curve recomputable under ELAI_SEED (GD-2). — plan-anchor=`K-15:L55-L58, L79-L79`
- **K-003/K15-E015** [SRC/E] (DEEP-K-003.discovery.md:L197): `E/TEST` — k15 contract or Python: [DESIGN]/[BENCH-PENDING] passes; untagged self-verifies fails; bare +20% fails; paper-7K fails; empty fails-loud; FIXTURE arm byte-identical under fixed seed. — plan-anchor=`K-15:L93-L94`
- **K-003/K15-E016** [SRC/E] (DEEP-K-003.discovery.md:L198 | SURFACE): `E/DATA` — Files: CREATE reasoning_budget_runner.rs, reasoning_grounding_negative.md, K-15-decisions.md; MODIFY args_bench.rs, J-34 calibration writer, dossier, claims_policy_lint.py; optional SSB Kind. — plan-anchor=`K-15:L96-L104`
- **K-003/K15-E017** [SRC/E] (DEEP-K-003.discovery.md:L199): `E/DEFER` — OUT-OF-K: J-83 typed BeliefState carrier KEYSTONE; J-84 2-axis classifier; J-85 overthinking guard; J-86 receipt-gated ladder; I-14 preserve-receipts rider. — plan-anchor=`K-15:L116-L116`
- **K-003/K15-E018** [SRC/E] (DEEP-K-003.discovery.md:L200 | SURFACE): `E/LIMIT` — NO LLM-judge scoring in the bench (TD-3); NO fork of I-27/J-34/K-1 lint; NO main-LLM self-run of lint. — plan-anchor=`K-15:L59-L59`
- **K-003/K15-A001** [SRC/A] (DEEP-K-003.discovery.md:L201): `A/EVENT` — Whether a new SSB Kind is added to ConvergenceCurveMeasured or a separate family is decided at edit (W-9). — plan-anchor=`K-15:L65-L65, L104-L104`
- **K-003/K15-A002** [SRC/A] (DEEP-K-003.discovery.md:L202 | SURFACE): `A/SURFACE` — Exact panel tile vs ride-existing-panels decision deferred to edit under W-4. — plan-anchor=`K-15:L61-L62`
- **K-003/K15-E019** [SRC/E] (DEEP-K-003.discovery.md:L203): `E/DEP` — Soft-depends K-1 (lint + dossier), K-7 (brief); bench-blocks on J GRC-v3 track; parallel-safe with K-13, K-14, K-16. — plan-anchor=`K-15:L8-L9`
- **K-003/K15-E020** [SRC/E] (DEEP-K-003.discovery.md:L204): `E/INT` — J-84/J-85 read the measured-knee artifact as the per-(model,task-type) ceiling (PRODUCTION reasoning loop, the calibration consumer). — plan-anchor=`K-15:L53-L53`
- **K-003/K15-E021** [SRC/E] (DEEP-K-003.discovery.md:L205): `E/TEST` — DETERMINISTIC_CHECK: reasoning-budget arm in args_bench.rs; verified.outcome/receipt/knee/peak in runner; zero paper 7000/7K/R1.32B transfers; negative lint fixture fails. — plan-anchor=`K-15:L69-L75`
- **K-003/K15-I001** [SRC/I] (DEEP-K-003.discovery.md:L206): `I/DEP` — Because ON arm cannot produce real delta until J-83+J-85 land, any brief implying measured lift before that is a KD-1 violation the plan explicitly forbids. — plan-anchor=`K-15:L112-L114`

#### Unit `K21` — 23 atoms (A=1, E=21, I=1)

- **K-003/K21-E001** [SRC/E] (DEEP-K-003.discovery.md:L216): `E/STATUS` — K-21 is PLANNED (WAVE KW-3; CANT-WRECK+DISCIPLINE; E-23 + doctrine class, K-8/K-14/K-16/K-19-shaped); .claude/rules/ doctrine + conditional-routing; NO runtime code, NO SSB. — plan-anchor=`K-21:L3-L3`
- **K-003/K21-E002** [SRC/E] (DEEP-K-003.discovery.md:L217): `E/CAP` — Promote journal COMPLETENESS invariant: an append-only log cannot prove it is COMPLETE; every covered side-effect must be MEDIATED or independently OBSERVED; observed effect with no journal row is hard FAIL. — plan-anchor=`K-21:L5-L5`
- **K-003/K21-E003** [SRC/E] (DEEP-K-003.discovery.md:L218): `E/LIMIT` — Does NOT author E1 capability (mediation/eBPF/reconcile = J); does NOT introduce runtime gate; does NOT re-state wiring-enforcement R1/R2 (binds it at completeness grain); does NOT claim measured completeness number. — plan-anchor=`K-21:L5-L5`
- **K-003/K21-E004** [SRC/E] (DEEP-K-003.discovery.md:L219): `E/CFG` — JC-1 COMPLETENESS-IS-NOT-INTEGRITY: hash-chain (I-18/J-69) proves no row ALTERED; cannot prove every side-effect PRODUCED a row; integrity and completeness are ORTHOGONAL. — plan-anchor=`K-21:L16-L16`
- **K-003/K21-E005** [SRC/E] (DEEP-K-003.discovery.md:L220): `E/CFG` — JC-2 MEDIATE-OR-OBSERVE: declared high-value effect classes (write-opens, network egress, child-process spawn/exec, privilege changes) must be MEDIATED (SECCOMP_RET_USER_NOTIF through Tier-C, EXTENDING G-3) or independently OBSERVED (eBPF/LSM); normalization layer MANDATORY. — plan-anchor=`K-21:L17-L17`
- **K-003/K21-E006** [SRC/E] (DEEP-K-003.discovery.md:L221 | SURFACE): `E/CFG` — JC-3 RECONCILE-MISSING=FAIL + MONITOR-THE-MONITOR: observed sensitive effect with no matching journal row in bounded window = DETERMINISTIC FAIL; dropped-event counters FAIL CLOSED; observer-impaired is FIRST-CLASS; OBSERVE-ONLY first (GD-4, >=3 real runs). — plan-anchor=`K-21:L18-L18`
- **K-003/K21-E007** [SRC/E] (DEEP-K-003.discovery.md:L222 | SURFACE): `E/CFG` — JC-4 HONEST-COVERAGE-SCOPE + PLATFORM-RESIDUE: honest claim is 'every COVERED effect class left a row'; host-trust ceiling (Rekor/TPM-IMA); observer LINUX-ONLY; Windows MEDIATION-ONLY; never advertise cloud observer guarantee on local lane (CL-4/GD-5). — plan-anchor=`K-21:L19-L19`
- **K-003/K21-E008** [SRC/E] (DEEP-K-003.discovery.md:L223 | SURFACE): `E/SURFACE` — Author .claude/rules/journal-completeness.md (~60-75 lines) + conditional-routing trigger for journal-completeness / side-effect-mediation / kernel-observer / reconcile / shadow-stream / trust-the-journal-RRA-canvas tasks. — plan-anchor=`K-21:L41-L41`
- **K-003/K21-E009** [SRC/E] (DEEP-K-003.discovery.md:L224 | SURFACE): `E/DEP` — Cross-link K-20 re-executable-RRA: re-executable is NOT complete — cassette faithfully reproduces incomplete record; trust-journal/canvas/derived-status claims carry covered-scope qualifier. — plan-anchor=`K-21:L42-L42`
- **K-003/K21-E010** [SRC/E] (DEEP-K-003.discovery.md:L225): `E/INT` — EXTEND K-1 lint: 'complete/full/tamper-proof/fully-auditable/nothing-escapes/total provenance' FAILS without covered-effect-set + platform + host-trust-ceiling; 'journal/RRA captures everything' FAILS without mediated-or-observed-and-reconciled basis; composes K-20 recogniser. — plan-anchor=`K-21:L43-L43`
- **K-003/K21-E011** [SRC/E] (DEEP-K-003.discovery.md:L226 | SURFACE): `E/TEST` — NEGATIVE fixtures: (a) untagged complete/tamper-proof journal; (b) 'hash-chain proves completeness'; (c) reconcile SKIP/UNVERIFIED when observer absent; (d) Windows advertising Linux eBPF guarantee — lint fails (a)/(b), rule catches (c)/(d). — plan-anchor=`K-21:L44-L44`
- **K-003/K21-E012** [SRC/E] (DEEP-K-003.discovery.md:L227): `E/DEP` — Cites G-3 BLAST-RADIUS WIRED (mediation extends); Tier-C seccomp WIRED core (SECCOMP_RET_USER_NOTIF host); SSB single-writer WAL WIRED; I-18/J-69 PLANNED (integrity axis); G-2 RRA WIRED; wiring-enforcement R1/R2. — plan-anchor=`K-21:L22-L28, L54-L56`
- **K-003/K21-E013** [SRC/E] (DEEP-K-003.discovery.md:L228 | SURFACE): `E/SURFACE` — frontend_pair N/A — doctrine + claims recogniser; E1 reconcile verdicts + C8-CANVAS shadow-activity anomaly nodes are J-track surfaces out-of-K. — plan-anchor=`K-21:L70-L71`
- **K-003/K21-E014** [SRC/E] (DEEP-K-003.discovery.md:L229): `E/OBS` — expect_ssb: none (E-23); expect_artifact: rule fires via conditional-routing; lint PASSES covered-scope claim and FAILS over-claims. — plan-anchor=`K-21:L72-L75`
- **K-003/K21-E015** [SRC/E] (DEEP-K-003.discovery.md:L230 | SURFACE): `E/DEFER` — OUT-OF-K E1 J sub-track: (a) MEDIATION SECCOMP_RET_USER_NOTIF; (b) kernel OBSERVER eBPF/LSM + normalization; (c) RECONCILE gate; (d) C8-CANVAS shadow-activity anomaly nodes; plus E2 runtime-observed DKG edges and E3 machine-derived-first audit lane. — plan-anchor=`K-21:L123-L123`
- **K-003/K21-E016** [SRC/E] (DEEP-K-003.discovery.md:L231 | SURFACE): `E/DATA` — Files: CREATE journal-completeness.md, journal_completeness_negative.md, K-21-decisions.md; MODIFY conditional-routing.md, claims_policy_lint.py. — plan-anchor=`K-21:L105-L110`
- **K-003/K21-E017** [SRC/E] (DEEP-K-003.discovery.md:L232 | SURFACE): `E/TEST` — k21 contract or Python: covered-scope-qualified passes; untagged complete/tamper-proof fails; hash-chain-proves-completeness fails; SKIP-on-missing-observer fails; Windows-eBPF-guarantee fails; empty fails-loud. — plan-anchor=`K-21:L102-L103`
- **K-003/K21-E018** [SRC/E] (DEEP-K-003.discovery.md:L233): `E/SEC` — LLM never adjudicates observed-effect <-> journal-row match (TD-3, JC-3). — plan-anchor=`K-21:L18-L18, L68-L68`
- **K-003/K21-A001** [SRC/A] (DEEP-K-003.discovery.md:L234 | SURFACE): `A/CFG` — Exact high-value effect-class enumeration beyond the four named categories and bounded-window duration for reconcile are left to E1 J authoring under the rule. — plan-anchor=`K-21:L17-L18, L123-L123`
- **K-003/K21-E019** [SRC/E] (DEEP-K-003.discovery.md:L235): `E/DEP` — Pre-gates the E1 J sub-track; completeness underwrites K-20's re-executable claim; soft-depends K-1 lint. — plan-anchor=`K-21:L8-L9`
- **K-003/K21-E020** [SRC/E] (DEEP-K-003.discovery.md:L236): `E/LIMIT` — Scope discipline: ships RULE + claim-governance only; temptation to author E1 mediation/eBPF/reconcile must be resisted (technical J sub-track). — plan-anchor=`K-21:L119-L119`
- **K-003/K21-E021** [SRC/E] (DEEP-K-003.discovery.md:L237): `E/TEST` — DETERMINISTIC_CHECK: journal-completeness.md exists; JC-1..JC-4 present; conditional-routing trigger wired; negative fixture lint nonzero; zero unscoped complete/tamper-proof claims in docs/strategy. — plan-anchor=`K-21:L77-L85`
- **K-003/K21-I001** [SRC/I] (DEEP-K-003.discovery.md:L238 | SURFACE): `I/DEP` — Because K-20 and K-21 share claim surface, a 're-executable AND complete' claim needs BOTH tags — the plan requires coordinated lint shapes, not two forks. — plan-anchor=`K-21:L122-L122`

#### Unit `K24` — 35 atoms (A=2, E=32, I=1)

- **K-003/K24-E001** [SRC/E] (DEEP-K-003.discovery.md:L248 | SURFACE): `E/STATUS` — K-24 is PLANNED (BCC wave; NET-NEW capability folded into K; multi-slice TRACK L1..L6 as ONE file; first commerce/billing surface; ~70% plumbing exists as J-28/J-49/C-5/H-9/J-81). — plan-anchor=`K-24:L3-L3`
- **K-003/K24-E002** [SRC/E] (DEEP-K-003.discovery.md:L249 | SURFACE): `E/CAP` — Model: subscription price -> monthly CREDITS (provider $) -> credits spent per metered usage -> ELAI pays upstream -> ELAI controls rate limits, rollover, routing (Nous Portal / OpenRouter shape). — plan-anchor=`K-24:L5-L5`
- **K-003/K24-E003** [SRC/E] (DEEP-K-003.discovery.md:L250): `E/LIMIT` — SAFE: no 'unlimited' an agent loop can detonate; spend metered and capped per request. — plan-anchor=`K-24:L5-L5`
- **K-003/K24-E004** [SRC/E] (DEEP-K-003.discovery.md:L251): `E/EVIDENCE` — Differentiator is RESOLVES MORE TASKS PER DOLLAR (local routing + verification + caching); claim MUST be SAFEDELTA-measured before said (L6 done-gate for any '$20=$200' line). — plan-anchor=`K-24:L5-L5, L40-L40`
- **K-003/K24-E005** [SRC/E] (DEEP-K-003.discovery.md:L252): `E/DATA` — Financial source of truth is ELAI's SSB credit journal — NEVER OpenRouter's or LiteLLM's spend store (GD-2; split-brain billing risk). Provider number is INPUT booked as Settled, reconciled nightly. — plan-anchor=`K-24:L5-L5, L66-L68`
- **K-003/K24-E006** [SRC/E] (DEEP-K-003.discovery.md:L253): `E/EVENT` — L1: +CreditLedgerEvent Pattern-2; kinds Granted — plan-anchor=`Reserved`
- **K-003/K24-E007** [SRC/E] (DEEP-K-003.discovery.md:L254 | SURFACE): `E/SURFACE` — L1 consumer: `elai billing balance` folds and prints; whole capability behind default-OFF ELAI_BILLING_ENABLE (allowlist type=experiment, promotion_plan_id=K-24-promotion). — plan-anchor=`K-24:L35-L35, L60-L60`
- **K-003/K24-E008** [SRC/E] (DEEP-K-003.discovery.md:L255): `E/FLOW` — L2 8-step loop: ESTIMATE (tokens x rate from J-28 SpendAccumulator + models.toml) -> reserve CONSERVATIVE hold -> emit Reserved -> CALL provider (J-29 LiveUsageGauge mid-stream; J-81 aborts runaway) -> READ actual -> emit Settled(actual), release (hold-actual) same-tick -> plan bonus at GRANT time -> balance=0 mid-task: halt at SAFE boundary with resumable handoff, NEVER hard kill mid-edit. — plan-anchor=`K-24:L36-L36`
- **K-003/K24-E009** [SRC/E] (DEEP-K-003.discovery.md:L256 | SURFACE): `E/CAP` — L3: OpenRouter MANAGED backend B-37 #12 hand-rolled Rust (no litellm, no new dep); returns actual cost per call booked as Settled via J-28; ELAI cost of credit-$ ~1.055x face (5.5% purchase fee). — plan-anchor=`K-24:L37-L37`
- **K-003/K24-E010** [SRC/E] (DEEP-K-003.discovery.md:L257): `E/LIMIT` — I-51: OpenRouter is PROVIDER/BILLING RAIL NOT dynamic model registry; catalog stays curated in models.toml. — plan-anchor=`K-24:L37-L37, L69-L69`
- **K-003/K24-E011** [SRC/E] (DEEP-K-003.discovery.md:L258 | SURFACE): `E/FLOW` — L4: J-49 admission checks balance >= hold AND fair-share AND tier rate/concurrency; fail-closed -> queue or refuse with clear 'out of credits' surface. — plan-anchor=`K-24:L38-L38`
- **K-003/K24-E012** [SRC/E] (DEEP-K-003.discovery.md:L259 | SURFACE): `E/FLOW` — L5: Payment (Stripe/Paddle) OUTSIDE ELAI (PCI scope zero); webhook subscription created/renewed/cancelled emits Granted(face+bonus) + Expired(over rollover cap); failed payment -> no Granted; chargeback -> AdjustmentManual(negative) + seat-suspension flag. — plan-anchor=`K-24:L39-L39`
- **K-003/K24-E013** [SRC/E] (DEEP-K-003.discovery.md:L260 | SURFACE): `E/CFG` — Rollover cap and tier rate limits are seats.toml DATA per tier (W-2, no new flag per tier). — plan-anchor=`K-24:L38-L39`
- **K-003/K24-E014** [SRC/E] (DEEP-K-003.discovery.md:L261 | SURFACE): `E/TEST` — L6: SAFEDELTA $/verified-task for ELAI-routed (local-heavy) vs raw-frontier; receipts on y-axis (GD-3); under ELAI_SEED; compose G-6 runner shape do NOT fork. — plan-anchor=`K-24:L40-L40`
- **K-003/K24-E015** [SRC/E] (DEEP-K-003.discovery.md:L262): `E/LIMIT` — Margin reality: OpenRouter 5.5% fee + 10% plan bonus = ~15% eaten on frontier portion; economics need large local/free-lane fraction; GATE launch on measured local-lane resolve-rate. — plan-anchor=`K-24:L40-L40, L140-L140`
- **K-003/K24-E016** [SRC/E] (DEEP-K-003.discovery.md:L263 | SURFACE): `E/SURFACE` — Honest pricing wording: 'Plans include monthly credits for model and tool usage, metered at provider cost. ELAI routes work to the cheapest model that can verifiably do the job — often a local one — so your credits resolve more tasks per dollar.' NEVER 'unlimited Sonnet/Opus/GPT.' — plan-anchor=`K-24:L40-L40`
- **K-003/K24-E017** [SRC/E] (DEEP-K-003.discovery.md:L264 | SURFACE): `E/SURFACE` — frontend_pair: credit balance + reserve/settle + tier rate limits on H-9 CloudPanel (or dedicated billing tile) as Svelte 5 runes + #[tauri::command] + generate_handler! + ts-rs; cross_lang_diff.py PASS (W-4). — plan-anchor=`K-24:L74-L75`
- **K-003/K24-E018** [SRC/E] (DEEP-K-003.discovery.md:L265 | SURFACE): `E/OBS` — trigger: `elai billing balance` ; metered OpenRouter request ; renewal webhook fixture; expect CreditLedgerEvent rows hash-chained recomputable. — plan-anchor=`K-24:L76-L79`
- **K-003/K24-E019** [SRC/E] (DEEP-K-003.discovery.md:L266): `E/EVENT` — SSB net +1 (CreditLedgerEvent), sequential bump from LIVE 268/577 (re-grep W-9). — plan-anchor=`K-24:L102-L102`
- **K-003/K24-E020** [SRC/E] (DEEP-K-003.discovery.md:L267): `E/PRIV` — NO card data or payment secret in harness or journal (PCI scope zero; webhook -> Granted only). — plan-anchor=`K-24:L71-L71, L100-L100`
- **K-003/K24-E021** [SRC/E] (DEEP-K-003.discovery.md:L268): `E/TEST` — k24_credit_billing_contract.rs (~200 LOC): fold identity; reserve/settle same-tick; double-spend single-writer; out-of-credits refusal; renewal webhook; fold-not-counter; Settled-not-provider-truth; '$20=$200' without L6 fails K-1 lint. — plan-anchor=`K-24:L113-L114`
- **K-003/K24-E022** [SRC/E] (DEEP-K-003.discovery.md:L269 | SURFACE): `E/DATA` — Files: CREATE billing/{mod,credit_ledger,reserve_settle,refill,webhook}.rs, openrouter_backend.rs, billing_cmd, k24 contract, SAFEDELTA arm, K-24-decisions.md; MODIFY args.rs, ssb/action_type_lock, J-49 admission, J-28 booking, dashboard-v2, seats.toml, wire-dark-allowlist. — plan-anchor=`K-24:L117-L128`
- **K-003/K24-E023** [SRC/E] (DEEP-K-003.discovery.md:L270): `E/EVIDENCE` — OpenRouter [EXTERNAL]: passthrough pricing, 5.5% purchase fee, BYOK 5%, $0.80 min; LiteLLM budget-against-own-DB is why it can NEVER be the ledger. — plan-anchor=`K-24:L132-L133`
- **K-003/K24-E024** [SRC/E] (DEEP-K-003.discovery.md:L271): `E/STATUS` — PRIOR-CAUTION: 2026-06-02 market work advised dropping bare cost framing for TRUST; efficiency pitch compatible ONLY as 'proven more value per dollar, verified'. — plan-anchor=`K-24:L13-L13`
- **K-003/K24-E025** [SRC/E] (DEEP-K-003.discovery.md:L272): `E/DEP` — GTM hypothesis: team/agency seat @ $39-49/seat may monetize BEFORE consumer tiers — author seats.toml for pooled-team tier from day one (not verified launch fact). — plan-anchor=`K-24:L142-L142`
- **K-003/K24-E026** [SRC/E] (DEEP-K-003.discovery.md:L273 | SURFACE): `E/DEP` — GD-1 reuse list: J-28 meter, J-49 seat/quota, C-5/J-81 caps + runaway kill, H-9 $/resolved-task, J-54 free pool, J-55 modality router, B-37 backend pattern, I-51 catalog, F-4/I-18/J-69/J-70 audit journal — re-implemented NEVER; only credit ledger + managed rail + payment seam + pricing surface are new. — plan-anchor=`K-24:L5-L5, L15-L24`
- **K-003/K24-E027** [SRC/E] (DEEP-K-003.discovery.md:L274): `E/FAIL` — NEGATIVE: mutable-counter balance FAILS; OpenRouter spend number used AS the ledger FAILS (GD-2); settle that does NOT match metered actual FAILS; '$20=$200' with no L6 SAFEDELTA FAILS K-1 lint. — plan-anchor=`K-24:L79-L79`
- **K-003/K24-E028** [SRC/E] (DEEP-K-003.discovery.md:L275): `E/DEP` — Single-vendor risk mitigated by B-37 direct-Anthropic/OpenAI fallback backend (bedrock/azure direct already exist) + local lane as always-on floor. — plan-anchor=`K-24:L37-L37, L141-L141`
- **K-003/K24-E029** [SRC/E] (DEEP-K-003.discovery.md:L276): `E/RECOVERY` — Balance hits 0 mid-task: J-81 + J-49 halt at a SAFE boundary with a resumable handoff, NEVER a hard kill mid-edit. — plan-anchor=`K-24:L36-L36, L72-L72`
- **K-003/K24-E030** [SRC/E] (DEEP-K-003.discovery.md:L277 | SURFACE): `E/CFG` — W-2/R9: ELAI_BILLING_ENABLE default-OFF with wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-24-promotion, ON-path smoke) — not a terminal state. — plan-anchor=`K-24:L35-L35, L100-L100, L107-L108`
- **K-003/K24-E031** [SRC/E] (DEEP-K-003.discovery.md:L278): `E/TEST` — Double-spend contract: two concurrent Reserved against the last credit — exactly one admitted via single-writer serialization. — plan-anchor=`K-24:L95-L96, L114-L114`
- **K-003/K24-E032** [SRC/E] (DEEP-K-003.discovery.md:L279): `E/DATA` — CreditLedgerEvent payload carries only seat-id, amounts, model/lane, provider-cost reference — NEVER card data or payment secret. — plan-anchor=`K-24:L45-L45`
- **K-003/K24-A001** [SRC/A] (DEEP-K-003.discovery.md:L280): `A/CFG` — Worst-case pre-flight hold multiplier per model decided at execution from real model rates. — plan-anchor=`K-24:L139-L139`
- **K-003/K24-A002** [SRC/A] (DEEP-K-003.discovery.md:L281 | SURFACE): `A/SURFACE` — Exact billing tile vs ride H-9 CloudPanel decided at edit. — plan-anchor=`K-24:L74-L75`
- **K-003/K24-I001** [SRC/I] (DEEP-K-003.discovery.md:L282): `I/DEP` — Because margin depends on J-55 routing quality, a poor local resolve-rate makes margin negative even if the ledger is correct — L6 is both product proof and business launch gate. — plan-anchor=`K-24:L140-L140`

#### Unit `K26` — 33 atoms (A=2, E=30, I=1)

- **K-003/K26-E001** [SRC/E] (DEEP-K-003.discovery.md:L292): `E/STATUS` — K-26 is a hard 5-day codex-credit burn campaign: proof-spine usage accrual + K-2 build; authored 2026-07-10, gate-checked + red-teamed; Not yet executed at plan time. — plan-anchor=`K-26:L1-L7`
- **K-003/K26-E002** [SRC/E] (DEEP-K-003.discovery.md:L293): `E/EVIDENCE` — PROOF_SPINE feeds PS-2 (gate-tax ceilings from real journals), PS-4 (K-2 routing-economics; J-28 live leg finished), PS-5 (TestOutcome red>green pairs toward case-reuse journal suite); discharges J-28 deferred R10 multi-turn smoke. — plan-anchor=`K-26:L3-L3`
- **K-003/K26-E003** [SRC/E] (DEEP-K-003.discovery.md:L294 | SURFACE): `E/CFG` — DEADLINE: bmapi codex-proxy credits expire 2026-07-15; account-a ~5.4k/8k monthly bar (2026-07-10 dashboard); account-b second pool; every live-model leg burns those credits and ZERO Anthropic tokens. — plan-anchor=`K-26:L5-L5`
- **K-003/K26-E004** [SRC/E] (DEEP-K-003.discovery.md:L295 | SURFACE): `E/INT` — gpt-5.5 is NATIVE ELAI provider: ProviderKind::Gpt55Proxy, detect_provider_kind("gpt-5.5"), auth BMAPI_API_KEY + BMAPI_BASE_URL (default https://bmapi.020212.xyz/v1). — plan-anchor=`K-26:L11-L11`
- **K-003/K26-E005** [SRC/E] (DEEP-K-003.discovery.md:L296): `E/SEC` — TWO codex accounts under /root/.codex-accounts/{account-a,account-b}; per-account key in auth.json (root 0600, NEVER copied into any repo); rotation = swap BMAPI_API_KEY. — plan-anchor=`K-26:L12-L12`
- **K-003/K26-E006** [SRC/E] (DEEP-K-003.discovery.md:L297 | SURFACE): `E/TEST` — Auditor-authored live smoke crates/tools/tests/gpt55_proxy_live_smoke.rs (SP-3B): gated ELAI_PROVIDER_LIVE_SMOKE=1, #[ignore], drives execute_tool("Agent") -> real bmapi HTTP, asserts PromptHardeningApplied SSB row. — plan-anchor=`K-26:L13-L13`
- **K-003/K26-E007** [SRC/E] (DEEP-K-003.discovery.md:L298 | SURFACE): `E/LIMIT` — CONTAMINATION GUARD: only ELAI's OWN runtime work enters ELAI journal; no TEIKOKU/GLM/harness task bridged into ssb.db. — plan-anchor=`K-26:L15-L15`
- **K-003/K26-E008** [SRC/E] (DEEP-K-003.discovery.md:L299 | SURFACE): `E/STATUS` — RED-TEAM CORRECTION: `elai run` has NO model-selection seam; CliAction::Run no model field; --provider closed local — plan-anchor=`cloud`
- **K-003/K26-E009** [SRC/E] (DEEP-K-003.discovery.md:L300): `E/INT` — ONLY wired gpt-5.5 production entry is Agent-tool handler per-call model arg (tools/lib.rs:4389). PS-2/PS-5 accrual is MODEL-AGNOSTIC; codex credits only decide who PAYS. — plan-anchor=`K-26:L17-L17`
- **K-003/K26-E010** [SRC/E] (DEEP-K-003.discovery.md:L301): `E/DATA` — auth.json field name OPENAI_API_KEY (export VALUE as BMAPI_API_KEY — names differ). No gpt-5.5 row in J-28 pricing catalog — VERIFY-6 fixture-only labeling load-bearing. — plan-anchor=`K-26:L18-L19`
- **K-003/K26-E011** [SRC/E] (DEEP-K-003.discovery.md:L302): `E/DATA` — SSB journal table is `ssb` (schema.rs:3294), not `actions`; action_type values evidence_gate_evaluated / test_outcome / usage_turn_event are real. — plan-anchor=`K-26:L20-L20`
- **K-003/K26-E012** [SRC/E] (DEEP-K-003.discovery.md:L303 | SURFACE): `E/CFG` — BOX RULES: ONE cargo at a time ALWAYS -j 1; auditor sole test runner via PowerShell FOREGROUND; keys sourced per-run NEVER written to repo/logs; ONE commit per unit maintainer voice; every dogfood task REAL .elai_cc work never synthetic farm. — plan-anchor=`K-26:L22-L28`
- **K-003/K26-E013** [SRC/E] (DEEP-K-003.discovery.md:L304): `E/FLOW` — MOVE 0 preflight: box uncontended; read account-a key; settle RECON-1..2; probe both accounts 1-token; write answers to K-26-decisions.md. — plan-anchor=`K-26:L30-L37`
- **K-003/K26-E014** [SRC/E] (DEEP-K-003.discovery.md:L305 | SURFACE): `E/FLOW` — MOVE 1 finish J-28: cargo test -p elai-cli -j 1; memory-writer; FEATURE-MAP upsert; ONE commit of 20 J-28 files ONLY; auth_broker.rs SEPARATE tiny commit; push both. — plan-anchor=`K-26:L39-L44`
- **K-003/K26-E015** [SRC/E] (DEEP-K-003.discovery.md:L306 | SURFACE): `E/FLOW` — MOVE 2 SP-3B live smoke on account-a with ELAI_PROVIDER_LIVE_SMOKE=1; exit 0 + PromptHardeningApplied row required. — plan-anchor=`K-26:L46-L51`
- **K-003/K26-E016** [SRC/E] (DEEP-K-003.discovery.md:L307 | SURFACE): `E/FLOW` — MOVE 3 dogfood engine: real .elai_cc fixes through `elai run` for 5 days; FORK-5 Route A (widen B-27 validation for gpt-5.5, RECON-1 <=5 files) vs Route B (existing lane + per-call gpt-5.5 Agent subagents). — plan-anchor=`K-26:L53-L58`
- **K-003/K26-E017** [SRC/E] (DEEP-K-003.discovery.md:L308 | SURFACE): `E/DATA` — Seed queue: (0 Route A only) gpt-5.5 seam; (1) usage_cmd print_json serde; (2) Dashboard panel RecurrenceHitRateMeasured; (3) Dashboard tile GateTaxMeasured; (4) startup_banner Windows tempdir race; (5) bench_convergence flake; (6) auth_broker SSB-emit writer=None at runtime_cmd.rs:395. — plan-anchor=`K-26:L59-L66`
- **K-003/K26-E018** [SRC/E] (DEEP-K-003.discovery.md:L309): `E/OBS` — VERIFY after every task: sqlite3 ssb.db COUNT of evidence_gate_evaluated/test_outcome/usage_turn_event must monotonically increase. — plan-anchor=`K-26:L70-L70`
- **K-003/K26-E019** [SRC/E] (DEEP-K-003.discovery.md:L310 | SURFACE): `E/FLOW` — MOVE 4 PS-2: after >=5 dogfood tasks run `elai bench gate-tax --seed 1234 --json`; surface ceilings to user (FORK-1 user locks); --enforce-ceiling exits nonzero until locked. — plan-anchor=`K-26:L72-L79`
- **K-003/K26-E020** [SRC/E] (DEEP-K-003.discovery.md:L311 | SURFACE): `E/FLOW` — MOVE 5 PS-5: after >=8-10 dogfood tasks with red>green, journal probe; FORK-2: >=10 distinct pairs -> BUILD `--suite journal`; <10 keep dogfooding, PS-5 stays RUNNER_PRESENT. — plan-anchor=`K-26:L81-L89`
- **K-003/K26-E021** [SRC/E] (DEEP-K-003.discovery.md:L312 | SURFACE): `E/SURFACE` — PRODUCTION_HOOK for journal suite: crates/cli/src/bench_case_reuse.rs — existing `elai bench case-reuse --suite journal` rejection becomes real dispatch. — plan-anchor=`K-26:L86-L86`
- **K-003/K26-E022** [SRC/E] (DEEP-K-003.discovery.md:L313): `E/FLOW` — MOVE 6 K-2 build: plan_gate_check fails on legacy plan; 9-step plan-improvement; re-gate PASS; execute orchestrate FSM; PRODUCTION_HOOK args_bench.rs routing-economics arm. — plan-anchor=`K-26:L91-L97`
- **K-003/K26-E023** [SRC/E] (DEEP-K-003.discovery.md:L314): `E/LIMIT` — PS-4 HONESTY CAVEAT (VERIFY-6): bmapi credit runs are free — any $/verified-task off gpt-5.5 labeled fixture/plumbing, never headline cloud-economics number. — plan-anchor=`K-26:L98-L98`
- **K-003/K26-E024** [SRC/E] (DEEP-K-003.discovery.md:L315): `E/FLOW` — MOVE 7 account rotation FORK-3: 2x 429 or <500 credits remaining -> swap to account-b; log swap in K-26-decisions.md. — plan-anchor=`K-26:L100-L104`
- **K-003/K26-E025** [SRC/E] (DEEP-K-003.discovery.md:L316 | SURFACE): `E/FLOW` — MOVE 8 surplus burn optional: (a) more dogfood; (b) J-28 R10 multi-turn/compaction smoke; (c) TEIKOKU codex-lane workers on L-1/L-2 (cap 2 attempts D-9; never touch ELAI journal). — plan-anchor=`K-26:L107-L110`
- **K-003/K26-E026** [SRC/E] (DEEP-K-003.discovery.md:L317 | SURFACE): `E/TEST` — INTEGRATION_SMOKE: (1) after MOVE 2 SP-3B; (2) before DONE long multi-turn `elai run` with EvidenceGateEvaluated+TestOutcome+UsageTurnEvent + `elai usage` rollup + gate-tax recompute. — plan-anchor=`K-26:L112-L116`
- **K-003/K26-E027** [SRC/E] (DEEP-K-003.discovery.md:L318): `E/FAIL` — ABORT-1 both accounts dead; ABORT-2 journal rows not landing; ABORT-3 cargo OOM twice; ABORT-4 TEIKOKU bridge temptation; ABORT-5 schema bump outside MOVE 5/6 beacon path. — plan-anchor=`K-26:L118-L124`
- **K-003/K26-E028** [SRC/E] (DEEP-K-003.discovery.md:L319): `E/LIMIT` — OUT OF SCOPE: PS-1 safedelta (RunPod); PS-3 L-10 (needs L-1..L-9); any harness (.claude/) fix. — plan-anchor=`K-26:L144-L148`
- **K-003/K26-E029** [SRC/E] (DEEP-K-003.discovery.md:L320 | SURFACE): `E/EVIDENCE` — REDTEAM 2026-07-10 non-author: ATTACK 1 SUCCEEDED PLAN-BLOCK (no elai run gpt-5.5 seam) -> FORK-5 patch; ATTACK 2 SUCCEEDED (table `actions` wrong -> `ssb`); ATTACKS 3-7 FAILED (plan survives). — plan-anchor=`K-26:L150-L157`
- **K-003/K26-E030** [SRC/E] (DEEP-K-003.discovery.md:L321 | SURFACE): `E/SURFACE` — User-facing surfaces in seed queue: Dashboard panel for RecurrenceHitRateMeasured; Dashboard tile for GateTaxMeasured; CLI `elai usage`; CLI `elai bench gate-tax`; CLI `elai bench case-reuse --suite journal`; CLI `elai bench routing-economics`; CLI `elai run`. — plan-anchor=`K-26:L61-L64, L75-L75, L86-L86, L94-L95`
- **K-003/K26-A001** [SRC/A] (DEEP-K-003.discovery.md:L322): `A/CFG` — FORK-5 Route A vs B settled by RECON-1 blast radius at MOVE 0; not fixed in the plan text alone. — plan-anchor=`K-26:L56-L58, L132-L132`
- **K-003/K26-A002** [SRC/A] (DEEP-K-003.discovery.md:L323): `A/STATUS` — Plan status 'Not yet executed' is point-in-time at authoring; actual campaign completion requires external evidence beyond this file. — plan-anchor=`K-26:L7-L7`
- **K-003/K26-I001** [SRC/I] (DEEP-K-003.discovery.md:L324): `I/DEP` — Because PS-2/PS-5 are model-agnostic, campaign success does not require gpt-5.5 for number accrual — only for credit burn path. — plan-anchor=`K-26:L17-L17, L55-L55`

#### Unit `K27` — 24 atoms (A=1, C=1, E=22)

- **K-003/K27-E001** [SRC/E] (DEEP-K-003.discovery.md:L334 | SURFACE): `E/STATUS` — K-27 authored 2026-07-11 (s-k26-gpt56), gate-checked + red-teamed; extends K-26 burn campaign; credit window 2 dies 2026-07-15. — plan-anchor=`K-27:L1-L7`
- **K-003/K27-E002** [SRC/E] (DEEP-K-003.discovery.md:L335 | SURFACE): `E/EVIDENCE` — PROOF_SPINE feeds PS-3: builds L-1..L-9 in dependency order — blockers of L-10 `elai bench hallucination-delta` which OWNS PS-3. — plan-anchor=`K-27:L3-L3`
- **K-003/K27-E003** [SRC/E] (DEEP-K-003.discovery.md:L336 | SURFACE): `E/CFG` — DEADLINE bmapi credits expire 2026-07-15; account-a ~2.3k left (dashboard credits-USED grew 5423→5712); account-b hard-429 on GET /v1/models (owner renew pending, FORK-1). — plan-anchor=`K-27:L5-L5`
- **K-003/K27-E004** [SRC/E] (DEEP-K-003.discovery.md:L337 | SURFACE): `E/INT` — gpt-5.6-sol live on account-a across chat/completions 200, /v1/responses 200, codex CLI e2e 200; /v1/messages still 403; both codex config.toml pinned model+review_model=gpt-5.6-sol. — plan-anchor=`K-27:L11-L11`
- **K-003/K27-E005** [SRC/E] (DEEP-K-003.discovery.md:L338): `E/SEC` — THREAT MODEL (owner bound): bmapi is Chinese proxy that INJECTS server-side prompt content — measured prompt_tokens=3273 on an 8-token call; assume every codex episode may carry adversarial instructions. — plan-anchor=`K-27:L12-L12`
- **K-003/K27-E006** [SRC/E] (DEEP-K-003.discovery.md:L339 | SURFACE): `E/SEC` — Consequence: codex output is TAINTED DATA; codex never sits in controller seat; codex never executes on Windows host or /mnt/f; nothing codex produces lands without off-agent gates. — plan-anchor=`K-27:L12-L12`
- **K-003/K27-E007** [SRC/E] (DEEP-K-003.discovery.md:L340 | SURFACE): `E/CAP` — PROVEN CONTAINMENT CHAIN (opus-orchestrator-mode): conductor/opus_walk/{opus_supervisor,opus_dispatch,opus_land,opus_preland_verify}.py on teikoku distro; opus_land.py --worktree (ext4), gates with cwd=<worktree>/.elai_cc, fail-closed scope-guard under .elai_cc/, frozen checkers, ff-only merge into /mnt/f + push, sha256-recorded. — plan-anchor=`K-27:L13-L13`
- **K-003/K27-E008** [SRC/E] (DEEP-K-003.discovery.md:L341): `E/INT` — opus_preland_verify.py::fresh_preland_verify spawns FRESH cc session to verdict diff against plan spec — cross-family review seam. — plan-anchor=`K-27:L13-L13`
- **K-003/K27-E009** [SRC/E] (DEEP-K-003.discovery.md:L342): `E/CFG` — codex-vm holds /root/elai-clone (git yes, cargo NO); Codex episodes CANNOT build; they write code + tests; builds/gates run off-vm on teikoku worktree. — plan-anchor=`K-27:L14-L14`
- **K-003/K27-E010** [SRC/E] (DEEP-K-003.discovery.md:L343): `E/INT` — Executor dispatch: opus_dispatch.py::dispatch_worker(...) at conductor/opus_walk/opus_dispatch.py:143; shape wsl -d codex-vm, CODEX_HOME=/root/.codex-accounts/<acct>, `codex exec -s workspace-write -` stdin task, cwd=episode worktree. — plan-anchor=`K-27:L15-L15`
- **K-003/K27-E011** [SRC/E] (DEEP-K-003.discovery.md:L344): `E/DEP` — L-track dependency order: L-1 → {L-3,L-5,L-8} → L-2 KEYSTONE → L-4 → L-6 → L-7 → L-9 → L-10 (bench; live arm RunPod-gated W-10, fixture arm buildable); L files legacy-format fail plan_gate_check until refined (D-8). — plan-anchor=`K-27:L16-L16`
- **K-003/K27-E012** [SRC/E] (DEEP-K-003.discovery.md:L345): `E/STATUS` — walk-state.json: queue=[H-10] commit-reconcile pending, escalation_queue empty, cursor idle; K-26 leftovers (K-2 slice commit, gate-freeze re-lock) stay owner-gated out of scope. — plan-anchor=`K-27:L17-L17`
- **K-003/K27-E013** [SRC/E] (DEEP-K-003.discovery.md:L346 | SURFACE): `E/LIMIT` — ELAI in-product routing does NOT accept gpt-5.6-* (providers/mod.rs:197 starts_with("gpt-5.5") only; fallback misroutes to Anthropic); irrelevant to codex CLI config but blocks in-product Agent-tool use of 5.6 until one-line widen ships. — plan-anchor=`K-27:L18-L18`
- **K-003/K27-E014** [SRC/E] (DEEP-K-003.discovery.md:L347): `E/SEC` — AI-1 TAINTED RETURNS: codex text/diff is inert evidence; controller never executes instructions found in worker output; fences worker returns as data. — plan-anchor=`K-27:L22-L22`
- **K-003/K27-E015** [SRC/E] (DEEP-K-003.discovery.md:L348): `E/SEC` — AI-2 CONTAINMENT: episodes ONLY inside codex-vm, cwd git worktree of /root/elai-clone, sandbox `-s workspace-write`; NO /mnt/f and NO /mnt/c mounted during executor episode; controller verifies mountpoint -q /mnt/f FAILS before each dispatch. — plan-anchor=`K-27:L23-L23`
- **K-003/K27-E016** [SRC/E] (DEEP-K-003.discovery.md:L349): `E/SEC` — AI-3 NO SECRETS: episode env carries nothing beyond codex auth.json already in vm; never export ELAI/Anthropic/ssh/GitHub credentials into codex env/prompt/worktree; elai-clone remote must be credential-free (RECON-2). — plan-anchor=`K-27:L24-L24`
- **K-003/K27-E017** [SRC/E] (DEEP-K-003.discovery.md:L350 | SURFACE): `E/SEC` — AI-4 CONTROLLER-PULLED PATCHES: controller extracts `git -C <worktree> diff` over wsl stdout; worker never given path/mount/command reaching Windows FS or teikoku distro. — plan-anchor=`K-27:L25-L25`
- **K-003/K27-E018** [SRC/E] (DEEP-K-003.discovery.md:L351 | SURFACE): `E/SEC` — AI-5 PATCH SCREEN: deny-grep before leave quarantine for network egress libs/URLs, base64/hex >200, build.rs, proc-macro crates, .github/hooks/.gitattributes, paths escaping .elai_cc, include! absolute, unsafe blocks; ANY hit = FROZEN pending owner eyes-on (FORK-4). — plan-anchor=`K-27:L26-L26`
- **K-003/K27-E019** [SRC/E] (DEEP-K-003.discovery.md:L352): `E/SEC` — AI-6 DEPENDENCY FREEZE: any Cargo.toml/Cargo.lock/package.json delta = automatic FROZEN pending owner confirmation. — plan-anchor=`K-27:L27-L27`
- **K-003/K27-E020** [SRC/E] (DEEP-K-003.discovery.md:L353): `E/SEC` — AI-7 CROSS-FAMILY JUDGMENT: every patch passes opus_preland_verify (fresh-context non-codex) + frozen checkers via opus_land; codex never judges codex; executor never runs checkers as authority (R8). — plan-anchor=`K-27:L28-L28`
- **K-003/K27-E021** [SRC/E] (DEEP-K-003.discovery.md:L354): `E/SEC` — AI-8 LANDING MONOPOLY: opus_land.py is ONLY path to /mnt/f; Codex never runs git against real repo; controller never hand-merges. — plan-anchor=`K-27:L29-L29`
- **K-003/K27-E022** [SRC/E] (DEEP-K-003.discovery.md:L355): `E/FLOW` — MOVE 0 preflight (no credits except one 1-token probe per account): cargo/rustc empty; gate_freeze_check clean (plan-standard.md drift owner-gated); probe account-a/b; settle RECON-1..4; reconcile walk-state H-10 — file TRUNCATES mid-sentence here. — plan-anchor=`K-27:L31-L33`
- **K-003/K27-C001** [SRC/C] (DEEP-K-003.discovery.md:L356): `C/STATUS` — Source file is truncated mid-sentence at L33 ('reconcile walk-state H-10 (its'); subsequent MOVEs, acceptance, and end marker are ABSENT from the on-disk image. Any claim of complete K-27 campaign procedure beyond AI-1..AI-8 + MOVE 0 start is unsupported by this source. — plan-anchor=`K-27:L31-L33`
- **K-003/K27-A001** [SRC/A] (DEEP-K-003.discovery.md:L357): `A/STATUS` — FORK-1 (account-b renew), FORK-4 (patch freeze eyes-on), and RECON-1..4 details are named but not fully specified in the surviving text. — plan-anchor=`K-27:L5-L5, L26-L26, L31-L33`

### K-003 cross-file interactions (namespaced K-003/IX-*)

- **K-003/IX-001** [IX] (DEEP-K-003.discovery.md:L365): K2-E001/E009/E010, K26-E022 | K-26 MOVE 6 owns refine-before-execute and build of K-2; K-2's 2026-07-11 wargame refine is that campaign's product. | K-2:L3-L5; K-26:L91-L97
- **K-003/IX-002** [IX] (DEEP-K-003.discovery.md:L366): K2-E009, K26-E002/E014 | J-28 UsageTurnEvent schema 272 is both K-2's live-leg cost capture input and K-26 MOVE 1 commit target. | K-2:L17-L23; K-26:L39-L44
- **K-003/IX-003** [IX] (DEEP-K-003.discovery.md:L367): K2-E005/E016, K26-E023 | Both require honest RunnerPresent / fixture labeling when live priced lanes or free-credit runs cannot fund a real cloud $ number. | K-2:L52-L54; K-26:L98-L98
- **K-003/IX-004** [IX] (DEEP-K-003.discovery.md:L368): K15-E002/E007, K2-E004 | K-15 composes I-27 convergence harness; K-2 forbids forking I-27 and instead mirrors safedelta/gate_tax shape — sibling benches, different axes. | K-15:L5-L7; K-2:L7-L7, L25-L25
- **K-003/IX-005** [IX] (DEEP-K-003.discovery.md:L369): K13-E003/E010, K15-E005/E006 | Both use [BENCH-PENDING] pattern: K-13 injection number waits on G-6 AgentDojo; K-15 ON arm waits on J-83/J-85. | K-13:L97-L101; K-15:L5-L6
- **K-003/IX-006** [IX] (DEEP-K-003.discovery.md:L370): K13-E011, K14-E010, K15-E009, K21-E010, K24-E021 | Multiple units EXTEND the same K-1 claims_policy_lint recogniser without forking it (OWASP cells, async-replay, reasoning-grounding, completeness, billing claims). | K-13:L102-L104; K-14:L105-L106; K-15:L34-L34; K-21:L43-L43; K-24:L114-L114
- **K-003/IX-007** [IX] (DEEP-K-003.discovery.md:L371): K14-E009, K15-E004, K13-E003 | Differentiator/dossier rows all land in K-1 ELAI-PROBLEM-QUANTIFICATION-DOSSIER with honest tags and feed K-7 brief. | K-14:L98-L104; K-15:L33-L33; K-13:L97-L101
- **K-003/IX-008** [IX] (DEEP-K-003.discovery.md:L372): K21-E009, K21-I001 | K-21 JC-1 governs K-20 re-executable claims: re-executable ≠ complete; coordinated lint tags required. | K-21:L42-L42, L122-L122
- **K-003/IX-009** [IX] (DEEP-K-003.discovery.md:L373): K12-E004, K15-E017 | K-12 extends I-11 BeliefState with critical_atoms; J-83 (OUT-OF-K via K-15) is the typed BeliefState reasoning carrier — composition surface, not dual authorship in this chunk. | K-12:L29-L29; K-15:L116-L116
- **K-003/IX-010** [IX] (DEEP-K-003.discovery.md:L374): K24-E006/E008, K2-E009 | K-24 L2/L3 book Settled amounts against J-28 UsageTurnEvent/SpendAccumulator that K-2 also consumes for live economics. | K-24:L35-L37; K-2:L23-L23
- **K-003/IX-011** [IX] (DEEP-K-003.discovery.md:L375): K24-E014, K15-E002 | Both demand SAFEDELTA/receipts-on-y-axis measurement discipline for efficiency claims (L6 vs reasoning-budget). | K-24:L40-L40; K-15:L5-L5
- **K-003/IX-012** [IX] (DEEP-K-003.discovery.md:L376): K26-E016/E030, K27-E002/E011 | K-27 extends K-26 toward PS-3 L-track greenfield; K-26 explicitly left PS-3 OUT OF SCOPE for its window. | K-26:L147-L147; K-27:L3-L3, L16-L16
- **K-003/IX-013** [IX] (DEEP-K-003.discovery.md:L377): K26-E005, K27-E003/E005 | Both campaigns use the same bmapi/codex account topology and treat proxy injection as a live threat; K-27 hardens with AI-1..AI-8 containment. | K-26:L11-L12; K-27:L12-L29
- **K-003/IX-014** [IX] (DEEP-K-003.discovery.md:L378): K12-E010, K24-E007 | Kill-switch / allowlist patterns: K-12 default-ON disable flag vs K-24 default-OFF enable flag — opposite polarity, both W-2/R9 disciplined. | K-12:L31-L31; K-24:L35-L35
- **K-003/IX-015** [IX] (DEEP-K-003.discovery.md:L379): K14-E004..E007, K21-E004..E007 | Sibling four-clause doctrine pattern (TZ-* vs JC-*) with conditional-routing pre-gates and K-1 lint extensions. | K-14:L52-L67; K-21:L15-L19
- **K-003/IX-016** [IX] (DEEP-K-003.discovery.md:L380): K2-E018, K15-E011, K24-E007, K26-E030 | Shared CLI surface family: `elai bench *` and `elai billing *` / `elai usage` / `elai run` as production entrypoints that satisfy R7. | K-2:L88-L88; K-15:L50-L51; K-24:L60-L60; K-26:L70-L95
- **K-003/IX-017** [IX] (DEEP-K-003.discovery.md:L381): K13-E012/E013, K14-E009 | Buyer packaging chain: K-13 OWASP mapping feeds K-6/K-7; K-14 differentiator feeds K-7; both avoid amending K-6/K-7 plan files. | K-13:L105-L107; K-14:L103-L104
- **K-003/IX-018** [IX] (DEEP-K-003.discovery.md:L382): K26-E017, K2-E022 | K-26 seed queue includes dashboard tiles for rows K-2/K-18a emit; K-2 FORK-3 defers its own cost panel while RunnerPresent — campaign may build adjacent panels first. | K-26:L61-L63; K-2:L67-L67

### K-003 summary deltas (namespaced K-003/SD-*)

- **K-003/SD-K2-001** [SD] (DEEP-K-003.discovery.md:L392): **CORRECT** summary K-2.11 'Block the live dollar leg on J-28': source states J-28 SHIPPED and live gate is W-10 RunPod/remote-sidecar. | 11-K:L92 | K2-E001, K2-E009, K2-E010, K2-C001
- **K-003/SD-K2-002** [SD] (DEEP-K-003.discovery.md:L393): **CORRECT** summary K-2.01/K-2 seam `args_bench.rs:75`: source says dispatcher chain is :52-93 and marks :75 STALE. | 11-K:L82 | K2-E012, K2-C002
- **K-003/SD-K2-003** [SD] (DEEP-K-003.discovery.md:L394): **ADD** RunnerStatus{Measured | RunnerPresent} split, explicit 'cost unavailable — live lane not funded' string, and emit_beacon_and_rra (not blake3-json-only). | 11-K:L78-L103 | K2-E016, K2-E005, K2-E019
- **K-003/SD-K2-004** [SD] (DEEP-K-003.discovery.md:L395): **ADD** FORK-1/2/3 (SSB reuse, live funding, frontend tile), RECON_NEEDED-1..4, ABORT-1..4, INTEGRATION_SMOKE hard-input requirements. | 11-K:L78-L103 | K2-E020-E022, K2-E028-E029
- **K-003/SD-K2-005** [SD] (DEEP-K-003.discovery.md:L396): **QUALIFY** summary K-2.12 'Feed H-9 cockpit display' and K-2.14 'Leave E-39 cap supremacy unchanged': neither is a load-bearing acceptance clause in the numbered K-2 plan body (H-9 appears only as deferred panel context; E-39 is not named). | 11-K:L93-L95 | K2-E022, K2-E024
- **K-003/SD-K2-006** [SD] (DEEP-K-003.discovery.md:L397): **ADD** corpus N=4-6, schema bump target 273, contract file path, and PS-4 spine identity. | 11-K:L78-L103 | K2-E002, K2-E013-E014, K2-E026
- **K-003/SD-K12-001** [SD] (DEEP-K-003.discovery.md:L403): **ADD** force SSB capture-hook-before-refuse path, RV-4 external side-effect residue, and exact Kind set AtomVerified/AtomMissing/CompactionRefused. | 11-K:L299-L325 | K12-E003, K12-E012, K12-E019
- **K-003/SD-K12-002** [SD] (DEEP-K-003.discovery.md:L404): **ADD** open risks: atom extraction source set, H-50 minimize interaction, MAP FIX I-5/I-11/H-50 PLANNED→BUILT. | 11-K:L299-L325 | K12-E018, K12-A001-A002
- **K-003/SD-K12-003** [SD] (DEEP-K-003.discovery.md:L405): **FLAG** default-ON disable flag vs observe-then-enforce dual rollout (summary A-06); source confirms both without a single transition schedule. | 11-K:L319 | K12-E010, K12-A003
- **K-003/SD-K13-001** [SD] (DEEP-K-003.discovery.md:L411): **ADD** standout-covered axis roster (ASI04/09/10, LLM03/09) and full STATUS enum including WIRED-MEASURED/WIRED-DESIGN. | 11-K:L327-L353 | K13-E005, K13-E007
- **K-003/SD-K13-002** [SD] (DEEP-K-003.discovery.md:L412): **ADD** kill-switch-1 caveat for NULL/NEGATIVE safety delta on the injection cell; dossier-row-home open question. | 11-K:L327-L353 | K13-E022, K13-A001
- **K-003/SD-K13-003** [SD] (DEEP-K-003.discovery.md:L413): **ADD** exact OWASP list re-verify rule (genai.owasp.org only; stale v1.1 ban) and fixture path owasp_negative.md. | 11-K:L327-L353 | K13-E017, K13-E014
- **K-003/SD-K14-001** [SD] (DEEP-K-003.discovery.md:L419): **ADD** full TZ clause text with 3E slot/OutputReserve placement detail, OsClock vs SeededClock split, and MASTER 412 autonomous-only wall-clock liveness. | 11-K:L355-L381 | K14-E004-E007
- **K-003/SD-K14-002** [SD] (DEEP-K-003.discovery.md:L420): **ADD** OUT-OF-K T-1..T-5 companion list with D-2/D-3/D-4 user decisions deferred to J pass. | 11-K:L381 | K14-E015, K14-A001
- **K-003/SD-K14-003** [SD] (DEEP-K-003.discovery.md:L421): **ADD** exact G3-G6 async-tool primitives named as differentiator factual basis (BackgroundTaskManager/AgentNotificationEmitted). | 11-K:L373-L375 | K14-E012, K14-E009
- **K-003/SD-K15-001** [SD] (DEEP-K-003.discovery.md:L427): **ADD** G-10-drift invalidation of knee artifact, ModelCapabilityProfile-adjacent store reuse of J-34, and Qwen-7B earliest-overthink note. | 11-K:L383-L412 | K15-E003, K15-E008
- **K-003/SD-K15-002** [SD] (DEEP-K-003.discovery.md:L428): **ADD** exact CLI flags `--suite default --seed N --json` and Kind name ReasoningBudgetArm. | 11-K:L383-L412 | K15-E011, K15-E012
- **K-003/SD-K15-003** [SD] (DEEP-K-003.discovery.md:L429): **ADD** OUT-OF-K J-83..J-86 + I-14 rider enumeration and honest residue T1..T5 pointer. | 11-K:L383-L412 | K15-E017
- **K-003/SD-K21-001** [SD] (DEEP-K-003.discovery.md:L435): **ADD** declared high-value effect classes (write-opens, egress, spawn/exec, privilege changes) and mandatory normalization layer for schema-mismatch FPs. | 11-K:L549-L579 | K21-E005
- **K-003/SD-K21-002** [SD] (DEEP-K-003.discovery.md:L436): **ADD** OUT-OF-K E1 deliverables including C8-CANVAS shadow-activity anomaly nodes, E2 observed_calls DKG edges, E3 machine-derived-first audit lane. | 11-K:L579 | K21-E015
- **K-003/SD-K21-003** [SD] (DEEP-K-003.discovery.md:L437): **ADD** K-20 coordination rule (re-executable AND complete needs both tags) and host-trust Rekor/TPM-IMA rider citation. | 11-K:L549-L579 | K21-E007, K21-E009, K21-I001
- **K-003/SD-K24-001** [SD] (DEEP-K-003.discovery.md:L443): **ADD** full 8-step reserve/settle loop text, same-tick hold release, and mid-edit safe-boundary halt (never hard kill). | 11-K:L666-L717 | K24-E008
- **K-003/SD-K24-002** [SD] (DEEP-K-003.discovery.md:L444): **ADD** honest pricing copy string, ~15% margin eat math (5.5% fee + 10% bonus), and launch-gate on local-lane resolve-rate. | 11-K:L666-L717 | K24-E015-E016
- **K-003/SD-K24-003** [SD] (DEEP-K-003.discovery.md:L445): **ADD** exact module layout billing/{...}, openrouter_backend.rs, wire-dark allowlist promotion_plan_id=K-24-promotion, LIVE 268/577 bump note. | 11-K:L666-L717 | K24-E007, K24-E019, K24-E022
- **K-003/SD-K24-004** [SD] (DEEP-K-003.discovery.md:L446): **ADD** PRIOR-CAUTION trust-not-cheap positioning and GTM $39-49 team-seat hypothesis as non-verified. | 11-K:L716-L717 | K24-E024-E025
- **K-003/SD-K26-001** [SD] (DEEP-K-003.discovery.md:L452): **ADD** complete MOVE 0..8 procedure, BOX RULES, FORK-1..5, ABORT-1..5, VERIFY-1..7, and INTEGRATION_SMOKE pair — summary had preamble-only ~22 atoms. | 11-K:L756-L781 | K26-E012-E029
- **K-003/SD-K26-002** [SD] (DEEP-K-003.discovery.md:L453): **ADD** seed-queue concrete .elai_cc tasks including named dashboard panels/tiles and auth_broker writer=None seam. | 11-K:L756-L781 | K26-E017, K26-E030
- **K-003/SD-K26-003** [SD] (DEEP-K-003.discovery.md:L454): **ADD** red-team ATTACK 1..7 outcomes, table-name `ssb` correction, and B-27 Anthropic-only validation ground truth. | 11-K:L756-L781 | K26-E008-E011, K26-E029
- **K-003/SD-K26-004** [SD] (DEEP-K-003.discovery.md:L455): **CORRECT** any reading that K-26 is only 'historical deadline + high-level moves': original is a full operations plan with production hooks and honesty caveats. | 11-K:L756-L781 | K26-E001, K26-E022-E023
- **K-003/SD-K27-001** [SD] (DEEP-K-003.discovery.md:L461): **ADD** entire unit (summary A-04: unindexed on-disk). Inventory all surviving atoms: PS-3 L-track mission, gpt-5.6-sol paths, threat model, AI-1..AI-8, opus_walk containment, L-order, MOVE 0 start. | 11-K:L23, L893-L894, L954 | K27-E001-E022
- **K-003/SD-K27-002** [SD] (DEEP-K-003.discovery.md:L462): **FLAG CONFLICT** source is truncated mid-sentence; summary line count 32 vs manifest logical 33 is trailing-newline artifact; campaign procedure after MOVE 0 is not recoverable from this image. | 11-K:L954 | K27-C001, K27-A001
- **K-003/SD-K27-003** [SD] (DEEP-K-003.discovery.md:L463): **ADD** relationship to K-26: extends burn campaign; K-26 leftovers (K-2 commit, gate-freeze re-lock) owner-gated out of K-27 scope. | 11-K:L756-L781 | K27-E001, K27-E012

### K-003 ambiguity and conflict register (namespaced K-003/AC-*)

- **K-003/AC-001** [AC] (DEEP-K-003.discovery.md:L469): C | Summary live-$ blocked on J-28 vs source W-10 after J-28 shipped | K2-C001
- **K-003/AC-002** [AC] (DEEP-K-003.discovery.md:L470): C | Summary args_bench :75 vs source :52-93 STALE | K2-C002
- **K-003/AC-003** [AC] (DEEP-K-003.discovery.md:L471): A | K-12 observe-then-enforce vs default-ON transition schedule | K12-A003
- **K-003/AC-004** [AC] (DEEP-K-003.discovery.md:L472): A | K-13 dossier row home (K-1 vs sibling section) | K13-A001
- **K-003/AC-005** [AC] (DEEP-K-003.discovery.md:L473): A | K-15/K-2/K-24 frontend tile exact panel deferred to edit | K2-A003, K15-A002, K24-A002
- **K-003/AC-006** [AC] (DEEP-K-003.discovery.md:L474): A | K-15/K-2 SSB +1 vs reuse FORK undecided until edit | K2-A002, K15-A001
- **K-003/AC-007** [AC] (DEEP-K-003.discovery.md:L475): C | K-27 file truncated; incomplete campaign procedure | K27-C001
- **K-003/AC-008** [AC] (DEEP-K-003.discovery.md:L476): A | K-26/K-27 campaign execution status is historical ops; not self-updating product state | K26-A002, K27-A001

## Ledger K-004 — all atomic records

Source: `docs/thefinalglance/feature-discovery/deep/DEEP-K-004.discovery.md`
Extracted ID-bearing rows: **294**.

### K-004 source atoms by unit

#### Unit `K4` — 53 atoms (A=3, C=1, E=48, I=1)

- **K-004/K4-E001** [SRC/E] (DEEP-K-004.discovery.md:L35): `E/STATUS` — K-4 is PLANNED WAVE KW-3 BUILD instrument + doctrine, KEYSTONE of K-T3, KD-3 owner; refined 2026-07-08 v2 after wargame blocks, three premise fixes, six red-team patches, and a user-locked scope decision with three riders. — plan-anchor=`K-4:L1-L3`
- **K-004/K4-E002** [SRC/E] (DEEP-K-004.discovery.md:L36): `E/STATUS` — PROOF_SPINE feeds PS-2 as owner of the composed gate-tax spine number (proof-spine-2026-07-05.md line 16-17). — plan-anchor=`K-4:L5-L5`
- **K-004/K4-E003** [SRC/E] (DEEP-K-004.discovery.md:L37 | SURFACE): `E/CAP` — Builds `elai bench gate-tax`: a per-release friction report answering what the trust spine costs on a normal coding task, against a LOCKED ceiling; ceiling breach yields non-zero CLI exit for the release path; report names top-3 friction contributors. — plan-anchor=`K-4:L7-L12, L180-L188`
- **K-004/K4-E004** [SRC/E] (DEEP-K-004.discovery.md:L38): `E/DATA` — TWO labeled numbers are reported and NEVER summed: `coding_task_gate_tax` (per-task in-band coding gates, ceiling-bound headline) and `audit_gate_tax` (per-audit offline checkers, own ceiling or explicit "unbounded, observed" tag). — plan-anchor=`K-4:L14-L24, L180-L188, L287-L289`
- **K-004/K4-E005** [SRC/E] (DEEP-K-004.discovery.md:L39): `E/CFG` — `coding_task_gate_tax` = frequency-weighted per-gate unit costs of the IN-BAND runtime coding set: FSM EvidenceGate family (11 gates) + T1/T2/T3 hooks + anti-fabrication + compliance pre-gate; telemetry-visible via EvidenceGateEvaluated / HookEvent / TieredAnalysis rows; journal-recomputable (GD-2). — plan-anchor=`K-4:L17-L20, L65-L67`
- **K-004/K4-E006** [SRC/E] (DEEP-K-004.discovery.md:L40): `E/CFG` — `audit_gate_tax` times offline auditor checkers once per changeset: `production_caller_check.py`, `maintainability_check.py`, `wiring_contract_check.py`, `plan_gate_check.py`, `taste_check.py`, by invocation wall-clock. — plan-anchor=`K-4:L21-L24, L94-L95, L180-L181`
- **K-004/K4-E007** [SRC/E] (DEEP-K-004.discovery.md:L41): `E/OBS` — `over_block_rate` (per-task) is a NEW benign-corpus sweep: N known-good coding diffs through the in-scope gate set, blocked/total per gate + composed; NOT the GateOverBlocked meta-gate artifact. — plan-anchor=`K-4:L25-L26, L80-L82, L200-L205`
- **K-004/K4-E008** [SRC/E] (DEEP-K-004.discovery.md:L42): `E/OBS` — RIDER-b COVERAGE TABLE: every run tags EVERY gate-manifest.txt-complement gate as `in-scope-coding` \ — plan-anchor=``trading-surface` \`
- **K-004/K4-E009** [SRC/E] (DEEP-K-004.discovery.md:L43): `E/DEFER` — RIDER-c FUTURE ARM: identical runner re-runs later with a trading corpus to produce `trading_task_gate_tax`; finance/trading gates are not dropped, they are a future arm. — plan-anchor=`K-4:L30-L31, L376-L377`
- **K-004/K4-E010** [SRC/E] (DEEP-K-004.discovery.md:L44): `E/LIMIT` — RESIDUE (GD-5): composed per-task number is a freq-weighted sum of parts and UNDERESTIMATES interaction effects; over-block covers the largest interaction class; coverage table names uncovered gates; remainder is NAMED residue. — plan-anchor=`K-4:L33-L35, L370-L372`
- **K-004/K4-E011** [SRC/E] (DEEP-K-004.discovery.md:L45 | SURFACE): `E/DATA` — DOSSIER ROW (KD-1): "composed gate-tax on coding tasks (in-band runtime set; audit tax reported separately; trading surface unmeasured)". — plan-anchor=`K-4:L36-L37`
- **K-004/K4-E012** [SRC/E] (DEEP-K-004.discovery.md:L46): `E/LIMIT` — VETO: Option-2 whole-pipeline global gate-bypass ON/OFF is REJECTED — shipped runtime switch disabling the gate stack violates R8/W-2; end-to-end LLM wall-clock is provider-latency-variance-dominated so ON/OFF delta of sub-ms gates drowns in noise. — plan-anchor=`K-4:L39-L43`
- **K-004/K4-E013** [SRC/E] (DEEP-K-004.discovery.md:L47 | SURFACE): `E/DEFER` — A `cfg(bench)`-gated whole-pipeline falsifier MAY exist LATER as one-shot validation of the cost model — compiled out of release, NEVER a runtime flag, NEVER the primary instrument. — plan-anchor=`K-4:L42-L43, L370-L372`
- **K-004/K4-E014** [SRC/E] (DEEP-K-004.discovery.md:L48 | SURFACE): `E/DEP` — Prerequisites: case_reuse_runner.rs + bench_case_reuse.rs (proven runner + 4-file CLI template), args_bench.rs dispatcher, EvidenceGateEvaluated (action_type.rs:308), HookEvent (:87), TieredAnalysis.elapsed_us, gate-manifest.txt. — plan-anchor=`K-4:L48-L50, L60-L68`
- **K-004/K4-E015** [SRC/E] (DEEP-K-004.discovery.md:L49): `E/DEP` — Parallel-safe with K-2, K-3, K-6, K-9; K-5 DEPENDS on K-4 (the ceiling it must fit under). — plan-anchor=`K-4:L51-L51, L376-L377`
- **K-004/K4-E016** [SRC/E] (DEEP-K-004.discovery.md:L50 | SURFACE): `E/STATUS` — FIX-1: gate set is NOT sourced from wire-dark-allowlist.toml (that is the default-OFF EXEMPTION registry, 135 rows); in-scope is the IN-BAND RUNTIME CODING gates with RIDER-b enumerating out-of-scope. — plan-anchor=`K-4:L71-L75, L247-L247, L340-L341`
- **K-004/K4-E017** [SRC/E] (DEEP-K-004.discovery.md:L51): `E/LIMIT` — FIX-2: NO single gates-ON/OFF product switch; Leg A measures each gate's unit cost in ISOLATION (OFF = gate not run in the runner), never a product-level bypass. — plan-anchor=`K-4:L76-L77, L136-L138`
- **K-004/K4-E018** [SRC/E] (DEEP-K-004.discovery.md:L52 | SURFACE): `E/LIMIT` — FIX-3: safedelta_runner.rs ON/OFF idiom is STATIC before/after split with no wall-clock number; Leg A uses journal elapsed_us OR a bench microbench, not that idiom. — plan-anchor=`K-4:L78-L79, L251-L251`
- **K-004/K4-E019** [SRC/E] (DEEP-K-004.discovery.md:L53): `E/LIMIT` — FIX-4: GateOverBlocked covers 8 finance gates × ONE fixture each and CANNOT produce a corpus-scale over-block rate for coding gates; over-block is a NEW benign-corpus sweep. — plan-anchor=`K-4:L80-L82, L251-L251, L358-L359`
- **K-004/K4-E020** [SRC/E] (DEEP-K-004.discovery.md:L54): `E/FLOW` — RECON_NEEDED-1: enumerate distinct gate_ids from EvidenceGate family + hooks via SsbViewer; EMPTY dry journal → named fixture gate set + mark RUNNER_PRESENT-not-MEASURED (never fabricate live composition). — plan-anchor=`K-4:L97-L101, L139-L140`
- **K-004/K4-E021** [SRC/E] (DEEP-K-004.discovery.md:L55): `E/FLOW` — RECON_NEEDED-1b: tag every gate-manifest.txt gate by grepping SsbActionType emissions; untagged = runner FAIL (RIDER-b). — plan-anchor=`K-4:L102-L105`
- **K-004/K4-E022** [SRC/E] (DEEP-K-004.discovery.md:L56): `E/DATA` — RECON_NEEDED-2: classify EvidenceGateEvaluated.ts_ms as POINT-IN-TIME not duration; only genuine duration is TieredAnalysisPayload.elapsed_us. — plan-anchor=`K-4:L106-L109, L62-L64, L355-L357`
- **K-004/K4-E023** [SRC/E] (DEEP-K-004.discovery.md:L57): `E/CFG` — RECON_NEEDED-3: if no first-class task_class field, key by template_id and NAME it the task-class proxy (GD-5 residue). — plan-anchor=`K-4:L110-L112`
- **K-004/K4-E024** [SRC/E] (DEEP-K-004.discovery.md:L58): `E/TEST` — RECON_NEEDED-4: author/find N known-good coding diffs that MUST NOT block for the over-block sweep; distinct from recurrence fixture. — plan-anchor=`K-4:L113-L115`
- **K-004/K4-E025** [SRC/E] (DEEP-K-004.discovery.md:L59 | SURFACE): `E/OBS` — RECON_NEEDED-5: time offline checkers via Command wall-clock over K reps (median); label "observed, unbounded" unless ceiling set. — plan-anchor=`K-4:L116-L119`
- **K-004/K4-E026** [SRC/E] (DEEP-K-004.discovery.md:L60 | SURFACE): `E/FLOW` — FORK-1 Leg-A per-gate-family: hook-tier → Route A (journal-native median elapsed_us); FSM EvidenceGate family → Route B (bench microharness K seeded reps median, reuse gate_self_validation.rs adapter; excluded gates NAMED as residue). — plan-anchor=`K-4:L122-L126`
- **K-004/K4-E027** [SRC/E] (DEEP-K-004.discovery.md:L61): `E/EVENT` — FORK-2 default NEW flat `GateTaxMeasured` action_type (mirror RecurrenceHitRateMeasured): flat scalars, no free-text, SCHEMA_VERSION auto-derived, golden action_type_lock regenerated SAME commit; signed .rra like K-18a. — plan-anchor=`K-4:L127-L129, L190-L198`
- **K-004/K4-E028** [SRC/E] (DEEP-K-004.discovery.md:L62 | SURFACE): `E/DATA` — GateTaxMeasured payload fields: coding_task_gate_tax_ms per named class, audit_gate_tax_ms, audit_unbounded bool, over_block_rate, coding_ceiling_ms, coding_breached bool, audit_ceiling_ms, top-3 contributor gate_ids + costs, coverage counts (n_in_scope_coding, n_trading_surface, n_offline_audit, n_dormant, n_untagged==0), n_gates, n_fires. — plan-anchor=`K-4:L190-L194`
- **K-004/K4-E029** [SRC/E] (DEEP-K-004.discovery.md:L63): `E/CFG` — FORK-3 ceiling values: TRIGGER = first clean run producing two numbers + over-block rate; coding ceiling VALUE is USER-SET (never invented); audit ceiling user-set OR "unbounded, observed"; until confirmed consts carry `// CEILING-PENDING[K-4]` and exit fail-CLOSED-blocks (ABORT-5). — plan-anchor=`K-4:L130-L134, L172-L178, L297-L298`
- **K-004/K4-E030** [SRC/E] (DEEP-K-004.discovery.md:L64): `E/FAIL` — ABORT-1: any product-level runtime switch disabling gate stack → STOP + handoff. — plan-anchor=`K-4:L136-L138`
- **K-004/K4-E031** [SRC/E] (DEEP-K-004.discovery.md:L65): `E/FAIL` — ABORT-2: RECON-1 empty AND no fixture gate set → STOP; plumbing is RUNNER_PRESENT never MEASURED. — plan-anchor=`K-4:L139-L140`
- **K-004/K4-E032** [SRC/E] (DEEP-K-004.discovery.md:L66): `E/FAIL` — ABORT-2b: any untaggable manifest gate → FAIL; headline may NOT claim whole coding trust-spine if uncovered. — plan-anchor=`K-4:L141-L142`
- **K-004/K4-E033** [SRC/E] (DEEP-K-004.discovery.md:L67): `E/FAIL` — ABORT-3: inventing a ceiling to make a test pass → STOP. — plan-anchor=`K-4:L143-L143`
- **K-004/K4-E034** [SRC/E] (DEEP-K-004.discovery.md:L68): `E/FAIL` — ABORT-4: GateTaxMeasured needing free-text/nested fields → STOP + handoff (schema decision). — plan-anchor=`K-4:L144-L145`
- **K-004/K4-E035** [SRC/E] (DEEP-K-004.discovery.md:L69 | SURFACE): `E/FAIL` — ABORT-5: if coding ceiling is CEILING-PENDING and breach exit is reachable from real release path, bench MUST exit non-zero (fail-CLOSED); label "release blocked pending ceiling confirmation"; WIRE-DARK only for truly unwired period. — plan-anchor=`K-4:L146-L148, L362-L363`
- **K-004/K4-E036** [SRC/E] (DEEP-K-004.discovery.md:L70): `E/CAP` — `pub fn run_gate_tax(seed: u64) -> Result<GateTaxReport, GateTaxError>` in benchmarks/gate_tax_runner.rs composing case_reuse_runner shape; ~330-420 LOC; no .unwrap()/.expect() on prod path. — plan-anchor=`K-4:L160-L163`
- **K-004/K4-E037** [SRC/E] (DEEP-K-004.discovery.md:L71): `E/DET` — Same seed twice → byte-identical GateTaxReport (GD-2). — plan-anchor=`K-4:L164-L166, L291-L291`
- **K-004/K4-E038** [SRC/E] (DEEP-K-004.discovery.md:L72 | SURFACE): `E/SURFACE` — CLI surface: `elai bench gate-tax [--seed <u64>] [--json]` via 4-file wiring (args_bench.rs arm + bench_gate_tax.rs + main.rs dispatch + args.rs variant), K-18a pattern. — plan-anchor=`K-4:L180-L186, L214-L217, L241-L243`
- **K-004/K4-E039** [SRC/E] (DEEP-K-004.discovery.md:L73 | SURFACE): `E/FAIL` — coding_task_gate_tax OR over_block_rate above coding ceiling → non-zero process exit (the release-gate is the CLI exit code, NOT a fictional Rust release_gate fn). — plan-anchor=`K-4:L171-L178, L200-L205, L213-L217, L360-L361`
- **K-004/K4-E040** [SRC/E] (DEEP-K-004.discovery.md:L74 | SURFACE): `E/INT` — Sole producer/consumer pair: run_gate_tax → bench_gate_tax.rs::run from real `elai bench gate-tax`; #[cfg(test)]/contracts never satisfy R7; production_caller_check 0 DARK. — plan-anchor=`K-4:L213-L219, L299-L299`
- **K-004/K4-E041** [SRC/E] (DEEP-K-004.discovery.md:L75): `E/TEST` — INTEGRATION_SMOKE R10 three cases off-agent: (1) HAPPY GateTaxMeasured with both numbers + full coverage + GD-2 recompute; (2) OVER-CEILING spammy fixture non-zero exit; (3) DRY-JOURNAL empty journal non-zero exit and no MEASURED-labeled row. — plan-anchor=`K-4:L221-L227, L364-L365`
- **K-004/K4-E042** [SRC/E] (DEEP-K-004.discovery.md:L76): `E/TEST` — Contract k4_gate_tax_contract.rs: Leg B compose from fixture journal; telemetry-derived gate set; two distinct payload fields; untagged coverage FAIL; spammy ceiling trip; top-3 named; same-seed identity; empty-journal FAIL-loud/RUNNER_PRESENT. — plan-anchor=`K-4:L316-L322`
- **K-004/K4-E043** [SRC/E] (DEEP-K-004.discovery.md:L77 | SURFACE): `E/LIMIT` — Stub bans: no invented ceiling; no wire-dark-allowlist gate set; no summing task+audit numbers; no untagged coverage; no product bypass; no GateOverBlocked as rate; no warn-only breach; no MEASURED on empty journal; no .unwrap on prod. — plan-anchor=`K-4:L245-L253`
- **K-004/K4-E044** [SRC/E] (DEEP-K-004.discovery.md:L78): `E/DATA` — Files: CREATE gate_tax_runner.rs + bench_gate_tax.rs + k4_gate_tax_contract.rs + K-4-decisions.md; MODIFY args_bench/args/main/bench_cmd; optional SSB action_type/payloads/schema + golden. — plan-anchor=`K-4:L324-L332`
- **K-004/K4-E045** [SRC/E] (DEEP-K-004.discovery.md:L79): `E/OBS` — ADV-1 dry journal silent composed_tax=0 is worst fail-open; must FAIL-loud/RUNNER_PRESENT. ADV-2 Route-B needs K>=N median reps. ADV-3 ceilings FIXTURE-provisional until real-journal. ADV-4 benign corpus distinct from positive. ADV-5 coverage-table silent-narrowing injection. — plan-anchor=`K-4:L273-L284`
- **K-004/K4-E046** [SRC/E] (DEEP-K-004.discovery.md:L80): `E/EVIDENCE` — Grounding: KW-4 CodeAct ~50% latency/>60% tokens [EXTERNAL/DESIGN]; KS-3 report 16/17; VERIFIED 2026-07-08 substrate inventory including FIX-1/4 and ATTACK-4 findings. — plan-anchor=`K-4:L334-L341`
- **K-004/K4-E047** [SRC/E] (DEEP-K-004.discovery.md:L81): `E/STATUS` — Red-team plan-standard: OVERALL PLAN-BLOCK on v1; all 6 successful attacks patched in v2 (scope, ts_ms, over-block, release_gate, CEILING-PENDING, R10 dry-journal). — plan-anchor=`K-4:L344-L365`
- **K-004/K4-E048** [SRC/E] (DEEP-K-004.discovery.md:L82): `E/DEFER` — Live/network-gated gates (model legs) excluded from deterministic in-band set; W-10 gated add later. — plan-anchor=`K-4:L373-L373`
- **K-004/K4-I001** [SRC/I] (DEEP-K-004.discovery.md:L83 | SURFACE): `I/SURFACE` — Optional friction tile on cost/audit surface is not mandated in the refined acceptance; CLI JSON + GateTaxMeasured row are the load-bearing operator surfaces. — plan-anchor=`K-4:L254-L258, L286-L300`
- **K-004/K4-A001** [SRC/A] (DEEP-K-004.discovery.md:L84): `A/CFG` — Exact coding_task and audit ceiling numeric values remain user-set after first real run; audit may stay unbounded. — plan-anchor=`K-4:L130-L134, L367-L369`
- **K-004/K4-A002** [SRC/A] (DEEP-K-004.discovery.md:L85 | SURFACE): `A/DATA` — Whether FORK-2 reuses an existing bench row family or always creates GateTaxMeasured defaults to NEW but is re-grepped at edit. — plan-anchor=`K-4:L127-L129`
- **K-004/K4-A003** [SRC/A] (DEEP-K-004.discovery.md:L86 | SURFACE): `A/TEST` — Exact N for benign corpus size, K microbench reps, and seed set for Route-B are not fixed beyond "K seeded reps" and "N known-good". — plan-anchor=`K-4:L113-L115, L122-L126, L277-L278`
- **K-004/K4-C001** [SRC/C] (DEEP-K-004.discovery.md:L87 | SURFACE): `C/CAP` — Summary-layer K-4 still describes ON/OFF ablation, wire-dark 105-row inventory, whole-system over-block, and args_bench.rs:75; refined source VETOES product ON/OFF, bans wire-dark inventory, splits two numbers + RIDER-b, and uses CLI exit-code contract without a line-:75 claim. — plan-anchor=`K-4:L14-L43, L71-L82, L171-L178`

#### Unit `K5` — 26 atoms (A=3, E=22, I=1)

- **K-004/K5-E001** [SRC/E] (DEEP-K-004.discovery.md:L97): `E/STATUS` — K-5 is PLANNED WAVE KW-3 BUILD policy + dispatch; KD-3/R8; depends on K-4 (ceiling it must fit under). — plan-anchor=`K-5:L1-L3, L16-L19`
- **K-004/K5-E002** [SRC/E] (DEEP-K-004.discovery.md:L98): `E/CAP` — Builds a deterministic R8-FROZEN GateIntensity policy table mapping risk-signal vector → which verification legs fire; collapse where safe, gate where expensive. — plan-anchor=`K-5:L4-L13, L50-L55`
- **K-004/K5-E003** [SRC/E] (DEEP-K-004.discovery.md:L99 | SURFACE): `E/CFG` — Always-on cheap legs: E-30 argv allowlist, F-1 compound gate, F-9 dep gate, G-1 taint. — plan-anchor=`K-5:L7-L9, L35-L35, L51-L53`
- **K-004/K5-E004** [SRC/E] (DEEP-K-004.discovery.md:L100): `E/CFG` — Expensive legs fire on risk triggers: I-20 mutation, H-26 differential, VLM tier-2, E-53 formal (Verus/Dafny). — plan-anchor=`K-5:L9-L11, L36-L36, L51-L53`
- **K-004/K5-E005** [SRC/E] (DEEP-K-004.discovery.md:L101 | SURFACE): `E/DATA` — Risk-signal vector: J-16 blast-radius cone size, F-2 CWE-surface class, B-22 autonomy level, D-15 novelty, file criticality (DKG/B-29 centrality), plus C-30 complexity classes. — plan-anchor=`K-5:L9-L11, L32-L37, L44-L45`
- **K-004/K5-E006** [SRC/E] (DEEP-K-004.discovery.md:L102): `E/SEC` — LLM NEVER picks the tier (TD-3); an LLM-proposed tier is ignored. — plan-anchor=`K-5:L11-L12, L40-L40, L61-L62, L81-L81`
- **K-004/K5-E007** [SRC/E] (DEEP-K-004.discovery.md:L103): `E/SEC` — Policy table hash is in frozen gate-manifest; a run that edited its own table is BLOCKED by gate_freeze_check.py (R8). — plan-anchor=`K-5:L11-L12, L56-L57, L99-L102, L108-L109`
- **K-004/K5-E008** [SRC/E] (DEEP-K-004.discovery.md:L104): `E/TEST` — Measured composed tax of the policy must fit under K-4's locked ceiling on >=3 real runs (GD-4/KD-3). — plan-anchor=`K-5:L12-L13, L60-L61, L107-L107`
- **K-004/K5-E009** [SRC/E] (DEEP-K-004.discovery.md:L105): `E/CAP` — Module `crates/orchestrator/src/gate_intensity.rs` (~240 LOC): table + deterministic selector/dispatcher. — plan-anchor=`K-5:L51-L55, L67-L68, L127-L127`
- **K-004/K5-E010** [SRC/E] (DEEP-K-004.discovery.md:L106): `E/DATA` — Policy data file `data/gate_intensity/policy_table.toml` (~60 LOC), R8-frozen. — plan-anchor=`K-5:L128-L128`
- **K-004/K5-E011** [SRC/E] (DEEP-K-004.discovery.md:L107): `E/EVENT` — SSB +GateIntensitySelected (Pattern-2; Kind: TierAssigned / ExpensiveLegFired / CollapsedToCheap) OR reuse F-1 compound-gate rows; re-grep 236/545 at edit; sequential bump. — plan-anchor=`K-5:L58-L59, L69-L69, L91-L92`
- **K-004/K5-E012** [SRC/E] (DEEP-K-004.discovery.md:L108): `E/INT` — Production reachability: auditor's per-edit gate-dispatch path consults `gate_intensity::select`; test-only never R7. — plan-anchor=`K-5:L77-L79, L113-L114`
- **K-004/K5-E013** [SRC/E] (DEEP-K-004.discovery.md:L109 | SURFACE): `E/SURFACE` — frontend_pair: tier-decision rows on the audit-timeline surface (decide at edit, W-4) OR W-6-justified absence. — plan-anchor=`K-5:L86-L87`
- **K-004/K5-E014** [SRC/E] (DEEP-K-004.discovery.md:L110): `E/FLOW` — Observable: trivial rename → zero expensive-leg rows (always-on only); high-cone auth-touching edit → full stack. — plan-anchor=`K-5:L88-L94, L120-L122`
- **K-004/K5-E015** [SRC/E] (DEEP-K-004.discovery.md:L111): `E/TEST` — Contract k5_gate_intensity_contract.rs: rename cheap-only; auth full stack; deterministic same-signals→same-tier; frozen hash; composed tax under fixture K-4 ceiling (over ceiling FAILS). — plan-anchor=`K-5:L119-L124`
- **K-004/K5-E016** [SRC/E] (DEEP-K-004.discovery.md:L112): `E/LIMIT` — Stub bans: no LLM-chosen tier; no unfrozen table; no re-implementation of risk signals or gate legs; no gate-everything default; no .unwrap on prod selector. — plan-anchor=`K-5:L80-L85`
- **K-004/K5-E017** [SRC/E] (DEEP-K-004.discovery.md:L113): `E/CFG` — Trigger thresholds (cone size, novelty, criticality) start OBSERVE-ONLY (log would-be tier without enforcing) until composed tax measured. — plan-anchor=`K-5:L142-L143`
- **K-004/K5-E018** [SRC/E] (DEEP-K-004.discovery.md:L114): `E/RECOVERY` — If a risk signal is unavailable/PARTIAL/dark, policy DEGRADES to a SAFER (more gates) tier, never a cheaper one (fail-safe). — plan-anchor=`K-5:L144-L145`
- **K-004/K5-E019** [SRC/E] (DEEP-K-004.discovery.md:L115): `E/FLOW` — Frozen-table workflow: authored ONCE out-of-band; a plan run that needs to change it escalates to the user (R8), never self-edits. — plan-anchor=`K-5:L146-L147`
- **K-004/K5-E020** [SRC/E] (DEEP-K-004.discovery.md:L116): `E/DEP` — Composes B-22/C-30/J-16/F-2/D-15/B-29 signals and F-1/F-9/E-30/G-1/I-20/H-26/E-53 legs; re-grep live symbols W-7; KG-8 inventory context. — plan-anchor=`K-5:L31-L39, L70-L76`
- **K-004/K5-E021** [SRC/E] (DEEP-K-004.discovery.md:L117): `E/DATA` — Creates workflow/reasoning/K-5-decisions.md; modifies auditor gate-dispatch (~30 LOC) and gate-manifest (~1 LOC hash). — plan-anchor=`K-5:L63-L63, L126-L133`
- **K-004/K5-E022** [SRC/E] (DEEP-K-004.discovery.md:L118): `E/EVIDENCE` — Grounding: KS-3 report 17 adaptive gating; KW-4 collapse-where-safe [EXTERNAL/DESIGN]; CLAUDE.md TD-3, R8, GD-4/KD-3. — plan-anchor=`K-5:L135-L139`
- **K-004/K5-I001** [SRC/I] (DEEP-K-004.discovery.md:L119): `I/INT` — "VLM tier-2" is treated as an expensive leg entrypoint peer of I-20/H-26/E-53 without a separate unit id in this plan. — plan-anchor=`K-5:L9-L11, L36-L36`
- **K-004/K5-A001** [SRC/A] (DEEP-K-004.discovery.md:L120): `A/CFG` — Exact numeric thresholds for cone/novelty/criticality and the observe-only → enforce transition schedule are unspecified beyond observe-first. — plan-anchor=`K-5:L142-L143`
- **K-004/K5-A002** [SRC/A] (DEEP-K-004.discovery.md:L121): `A/EVENT` — GateIntensitySelected NEW row versus F-1 reuse is deferred to edit-time re-grep. — plan-anchor=`K-5:L58-L59`
- **K-004/K5-A003** [SRC/A] (DEEP-K-004.discovery.md:L122 | SURFACE): `A/SURFACE` — Exact audit-timeline UI presentation of tier decisions is deferred (W-4 or W-6). — plan-anchor=`K-5:L86-L87`

#### Unit `K7` — 21 atoms (A=1, E=19, I=1)

- **K-004/K7-E001** [SRC/E] (DEEP-K-004.discovery.md:L132): `E/STATUS` — K-7 is PLANNED WAVE KW-3 positioning-only E-23-class; NO runtime code, NO SSB; depends on K-1 numbers. — plan-anchor=`K-7:L1-L3, L61-L61`
- **K-004/K7-E002** [SRC/E] (DEEP-K-004.discovery.md:L133 | SURFACE): `E/SURFACE` — Authors pure-prose brief `docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md` (~120-180 lines / ~150 LOC) structured as (problem-with-a-number) → (ELAI control) → (compliance-export evidence). — plan-anchor=`K-7:L5-L10, L43-L46, L52-L52, L98-L98`
- **K-004/K7-E003** [SRC/E] (DEEP-K-004.discovery.md:L134): `E/CAP` — Positions evidence pack (K-6 compliance export + trust spine) into AppSec/SCA budgets and insurance underwriting questionnaires. — plan-anchor=`K-7:L5-L10, L20-L25`
- **K-004/K7-E004** [SRC/E] (DEEP-K-004.discovery.md:L135): `E/EVIDENCE` — F-9 hallucinated-dependency gate vs KW-1 USENIX numbers (5.2%/21.7%, 58%/43% recurrence) [EXTERNAL]. — plan-anchor=`K-7:L7-L8, L23-L24, L102-L102`
- **K-004/K7-E005** [SRC/E] (DEEP-K-004.discovery.md:L136): `E/EVIDENCE` — F-2 CWE gates vs KW-2 11x-vulns / CWE-95/502 finding [EXTERNAL]. — plan-anchor=`K-7:L7-L8, L24-L24, L103-L103`
- **K-004/K7-E006** [SRC/E] (DEEP-K-004.discovery.md:L137): `E/EVIDENCE` — Audit trail + attestation + rollback answer insurance questionnaire controls/logs/rollback/independent-evidence questions. — plan-anchor=`K-7:L8-L9, L24-L25, L44-L45`
- **K-004/K7-E007** [SRC/E] (DEEP-K-004.discovery.md:L138): `E/DEP` — Soft-depends K-1 (numbers + claims-policy lint) + K-6 (the pack sold); F-9/F-2 controls re-confirmed live; E-23 precedent. — plan-anchor=`K-7:L13-L16, L27-L32, L54-L56`
- **K-004/K7-E008** [SRC/E] (DEEP-K-004.discovery.md:L139 | SURFACE): `E/CFG` — Every number tagged measured-vs-external (KD-1); brief must PASS `claims_policy_lint.py --files docs/strategy/ELAI-APPSEC-INSURANCE-BRIEF.md`. — plan-anchor=`K-7:L9-L10, L47-L47, L68-L71, L83-L83`
- **K-004/K7-E009** [SRC/E] (DEEP-K-004.discovery.md:L140): `E/LIMIT` — NO Article-50-certified claim (GTM-killed; "designed to support" only); `rg` for Article.50.certified\ — plan-anchor=`certified compliant MUST be 0.`
- **K-004/K7-E010** [SRC/E] (DEEP-K-004.discovery.md:L141): `E/LIMIT` — NO enterprise-first framing contradicting GTM stance (agencies-first @ $39-49/seat; kill Article-50 hook). — plan-anchor=`K-7:L38-L39, L64-L64, L84-L84, L105-L105`
- **K-004/K7-E011** [SRC/E] (DEEP-K-004.discovery.md:L142): `E/SEC` — User-reviewed before any external use. — plan-anchor=`K-7:L10-L10, L85-L85`
- **K-004/K7-E012** [SRC/E] (DEEP-K-004.discovery.md:L143): `E/TEST` — NEGATIVE arm shared with K-1: seeded untagged number FAILS the lint. — plan-anchor=`K-7:L70-L71, L94-L95`
- **K-004/K7-E013** [SRC/E] (DEEP-K-004.discovery.md:L144 | SURFACE): `E/INT` — Production audit/lint gate is claims_policy_lint over the brief; #[cfg(test)] never satisfies R7. — plan-anchor=`K-7:L57-L59`
- **K-004/K7-E014** [SRC/E] (DEEP-K-004.discovery.md:L145 | SURFACE): `E/SURFACE` — frontend_pair N/A — positioning doc; W-6 justified. — plan-anchor=`K-7:L65-L66`
- **K-004/K7-E015** [SRC/E] (DEEP-K-004.discovery.md:L146): `E/DATA` — Creates workflow/reasoning/K-7-decisions.md (~20 LOC). — plan-anchor=`K-7:L48-L48, L99-L99`
- **K-004/K7-E016** [SRC/E] (DEEP-K-004.discovery.md:L147): `E/PRIV` — If brief is ever externalized, apply feedback_external_prompt_anonymity (neutral vocab, no codenames/paths); internal brief keeps real names. — plan-anchor=`K-7:L40-L40, L109-L110`
- **K-004/K7-E017** [SRC/E] (DEEP-K-004.discovery.md:L148): `E/DEP` — Number freshness: external numbers must match K-1 dossier re-verified KW set; if dossier re-verification changes a value, brief inherits it (single source of truth). — plan-anchor=`K-7:L111-L112`
- **K-004/K7-E018** [SRC/E] (DEEP-K-004.discovery.md:L149): `E/LIMIT` — No runtime code, no SSB, no new dependency (E-23-class). — plan-anchor=`K-7:L86-L86`
- **K-004/K7-E019** [SRC/E] (DEEP-K-004.discovery.md:L150): `E/EVIDENCE` — KS-3 report 16: org-side buyer, existing budget lines, insurer AI sublimits [EXTERNAL]. — plan-anchor=`K-7:L5-L5, L104-L104`
- **K-004/K7-I001** [SRC/I] (DEEP-K-004.discovery.md:L151): `I/DEP` — Summary cites G-8/G-9/I-44 as control citations; numbered K-7 body names K-6 export + F-9/F-2 + K-1 more prominently than those three IDs, which appear only via K-6 composition. — plan-anchor=`K-7:L5-L15, L54-L56`
- **K-004/K7-A001** [SRC/A] (DEEP-K-004.discovery.md:L152 | SURFACE): `A/SURFACE` — Exact section titles and insurance-questionnaire template mapping beyond the problem→control→evidence shape are left to authoring. — plan-anchor=`K-7:L43-L46`

#### Unit `K11` — 27 atoms (A=2, E=24, I=1)

- **K-004/K11-E001** [SRC/E] (DEEP-K-004.discovery.md:L162): `E/STATUS` — K-11 is PLANNED WAVE KW-2 M BUILD-renderer; EXTENSION of K-3/E-41/G-5/G-2; author NO new evidence machinery. — plan-anchor=`K-11:L1-L5`
- **K-004/K11-E002** [SRC/E] (DEEP-K-004.discovery.md:L163 | SURFACE): `E/CAP` — Extends K-3 PR/CI emitter with a human-viewable EVIDENCE-ARTIFACT section: screenshots / logs / RRA-summary / test-evidence into PR body + G-5 reviewer cockpit, risk-tiered by G-5 DSR. — plan-anchor=`K-11:L5-L6, L30-L33`
- **K-004/K11-E003** [SRC/E] (DEEP-K-004.discovery.md:L164 | SURFACE): `E/SURFACE` — Markdown `## Evidence` section: RRA summary + per-test evidence links + collapsible logs + screenshot refs, posted via `post_review_comment`. — plan-anchor=`K-11:L32-L32, L54-L57`
- **K-004/K11-E004** [SRC/E] (DEEP-K-004.discovery.md:L165 | SURFACE): `E/SURFACE` — Same EvidenceArtifactSet view in G-5 reviewer cockpit via Tauri read; cross_lang_diff.py PASS (frontend_pair). — plan-anchor=`K-11:L52-L53, L78-L78, L91-L91`
- **K-004/K11-E005** [SRC/E] (DEEP-K-004.discovery.md:L166 | SURFACE): `E/DATA` — EvidenceArtifactSet assembler (~80 LOC) collects existing artifacts only: G-2 RRA digest, F-4 TestOutcomePayload stdout_hash rows, G-5 DSR + vouched/unvouched bands, H-52 anchored findings, visual-ui-loop screenshot refs. — plan-anchor=`K-11:L31-L31, L37-L38, L41-L42`
- **K-004/K11-E006** [SRC/E] (DEEP-K-004.discovery.md:L167): `E/FAIL` — Hunk with NO backing artifact renders UNVOUCHED entry, never omitted, never silently clean (extends K-3 invariant). — plan-anchor=`K-11:L13-L13, L31-L31, L50-L50, L57-L57`
- **K-004/K11-E007** [SRC/E] (DEEP-K-004.discovery.md:L168): `E/FLOW` — Risk-tier: high-attention G-5 DSR hunks expand; low-risk collapse. — plan-anchor=`K-11:L32-L32, L86-L86`
- **K-004/K11-E008** [SRC/E] (DEEP-K-004.discovery.md:L169 | SURFACE): `E/SURFACE` — CLI flag `elai pr-review --emit-evidence` (default-ON) behind kill-switch `ELAI_PR_EVIDENCE_RENDER_DISABLE` (W-2; kill-switch not enable-flag). — plan-anchor=`K-11:L33-L33, L39-L39, L71-L71, L76-L76`
- **K-004/K11-E009** [SRC/E] (DEEP-K-004.discovery.md:L170 | SURFACE): `E/INT` — Wire into K-3 emit path (~20 LOC): check-run posting calls K-11 renderer; evidence section ships with per-hunk verdicts in SAME emit. — plan-anchor=`K-11:L33-L33, L44-L45`
- **K-004/K11-E010** [SRC/E] (DEEP-K-004.discovery.md:L171 | SURFACE): `E/CAP` — Module `crates/orchestrator/src/github_pr/evidence_render.rs` (~150 LOC under .elai_cc path in Files touched). — plan-anchor=`K-11:L37-L38, L89-L89`
- **K-004/K11-E011** [SRC/E] (DEEP-K-004.discovery.md:L172 | SURFACE): `E/EVENT` — prefer-reuse github_pr / G-5 review_surface rows; else +1 EvidenceArtifactRendered (Pattern-2; Kind: SectionRendered / HunkVouched / HunkUnvouched); W-9 from LIVE baseline ~267/577. — plan-anchor=`K-11:L56-L56, L82-L82`
- **K-004/K11-E012** [SRC/E] (DEEP-K-004.discovery.md:L173 | SURFACE): `E/DEP` — Consumes (never re-implements): post_review_comment / ReviewCommentKind; rra build/verify; review_surface; review_anchor; TestOutcomePayload; .claude/tools/visual/ screenshots. — plan-anchor=`K-11:L7-L8, L41-L42, L48-L49, L70-L70`
- **K-004/K11-E013** [SRC/E] (DEEP-K-004.discovery.md:L174): `E/LIMIT` — NO new capture; NO git mutation; NO lavish author-brand design bundle; NO .unwrap on prod; blake3-only; worker-git-read-only. — plan-anchor=`K-11:L31-L32, L47-L51`
- **K-004/K11-E014** [SRC/E] (DEEP-K-004.discovery.md:L175 | SURFACE): `E/CFG` — W-10 split: deterministic render fixture now; live check-run post network/RunPod-gated separately. — plan-anchor=`K-11:L7-L7, L55-L55, L79-L79`
- **K-004/K11-E015** [SRC/E] (DEEP-K-004.discovery.md:L176): `E/TEST` — Contract k11_evidence_render_contract.rs: assemble RRA+F-4+G-5+H-52+screenshots; UNVOUCHED path; calls post_review_comment; DSR collapse/expand; no git mutation. — plan-anchor=`K-11:L85-L86, L92-L92`
- **K-004/K11-E016** [SRC/E] (DEEP-K-004.discovery.md:L177): `E/LIMIT` — Cloud reproducibility claim stays action-stream (GD-5), never "byte-identical tokens". — plan-anchor=`K-11:L13-L13, L83-L83`
- **K-004/K11-E017** [SRC/E] (DEEP-K-004.discovery.md:L178): `E/CFG` — KD-1: every rendered number carries measured/journal source or [EXTERNAL]/[DESIGN] tag. — plan-anchor=`K-11:L69-L69`
- **K-004/K11-E018** [SRC/E] (DEEP-K-004.discovery.md:L179 | SURFACE): `E/CFG` — KD-2: evidence renders into the PR; cockpit is DEEP view never only view. — plan-anchor=`K-11:L10-L10, L68-L68`
- **K-004/K11-E019** [SRC/E] (DEEP-K-004.discovery.md:L180 | SURFACE): `E/DEFER` — DOCTRINE-TENSION recorded NOT in K-11 scope: no-mistakes PR-babysit-to-merge vs one-commit-at-end + worker-git-read-only → wedge slice is poll-CI + surface, human merges, NO autonomous force-push. — plan-anchor=`K-11:L102-L102`
- **K-004/K11-E020** [SRC/E] (DEEP-K-004.discovery.md:L181 | SURFACE): `E/DEFER` — lavish overlapping-text elementFromPoint occlusion probe is a separate rider to visual-ui-loop.md, not K-11. — plan-anchor=`K-11:L103-L103`
- **K-004/K11-E021** [SRC/E] (DEEP-K-004.discovery.md:L182 | SURFACE): `E/LIMIT` — GitHub check-run/PR-comment size limits: render refs/links not embedded blobs; decide against live API limit via WebSearch (KD-1). — plan-anchor=`K-11:L101-L101`
- **K-004/K11-E022** [SRC/E] (DEEP-K-004.discovery.md:L183 | SURFACE): `E/EVIDENCE` — Grounding: steal-kunchen C2.2 EXTENSION(2) no-mistakes ## Testing/Evidence; C3.3 META "eng-manager not diff-reviewer" is AUTHOR/market-validation claim not ELAI number (HD-1/ID-5). — plan-anchor=`K-11:L5-L5, L96-L96`
- **K-004/K11-E023** [SRC/E] (DEEP-K-004.discovery.md:L184 | SURFACE): `E/DATA` — Creates workflow/reasoning/K-11-decisions.md; modifies CLI pr-review + dashboard-v2 cockpit evidence view. — plan-anchor=`K-11:L34-L34, L88-L93`
- **K-004/K11-E024** [SRC/E] (DEEP-K-004.discovery.md:L185 | SURFACE): `E/INT` — Production paths: `elai pr-review --emit-evidence` → github_pr post; K-3 check-run path → EvidenceArtifactSet → post_review_comment. — plan-anchor=`K-11:L43-L46`
- **K-004/K11-I001** [SRC/I] (DEEP-K-004.discovery.md:L186): `I/DEP` — H-52 review_anchor is PARTIAL/uncommitted in existing-state inventory; K-11 still lists it as a consumed anchor for rendered findings. — plan-anchor=`K-11:L21-L21, L41-L42`
- **K-004/K11-A001** [SRC/A] (DEEP-K-004.discovery.md:L187): `A/EVENT` — Exact prefer-reuse versus +1 EvidenceArtifactRendered decision depends on live baseline re-grep. — plan-anchor=`K-11:L56-L56, L82-L82`
- **K-004/K11-A002** [SRC/A] (DEEP-K-004.discovery.md:L188 | SURFACE): `A/SURFACE` — Exact collapsible-log markdown mechanics and GitHub size cutoffs decided at authoring. — plan-anchor=`K-11:L32-L32, L101-L101`

#### Unit `K18a` — 26 atoms (A=2, C=1, E=22, I=1)

- **K-004/K18a-E001** [SRC/E] (DEEP-K-004.discovery.md:L198): `E/STATUS` — K-18a is a NEW numbered file (house rule: K-18 is NOT amended), authored 2026-07-08, PLANNED; feeds PS-5 with recurrence-hit-rate observe number + runner for later reuse delta. — plan-anchor=`K-18a:L1-L4`
- **K-004/K18a-E002** [SRC/E] (DEEP-K-004.discovery.md:L199): `E/CAP` — Executes only K-18 Task 1 (recurrence-hit-rate OBSERVE arm + FIXTURE tier) + Task 6 (decisions doc); ZERO reuse-into-generation; does NOT produce compounding iteration/$ delta (Task 2). — plan-anchor=`K-18a:L6-L12, L65-L71`
- **K-004/K18a-E003** [SRC/E] (DEEP-K-004.discovery.md:L200 | SURFACE): `E/LIMIT` — OUT: Task 2 reuse economics (VCG J-track unbuilt J-95..J-98); Tasks 3/4/5 dossier+claims-lint (claims_policy_lint.py and ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md DO NOT EXIST 2026-07-08 — would silently AUTHOR K-1); live local-lane (W-10). — plan-anchor=`K-18a:L17-L24`
- **K-004/K18a-E004** [SRC/E] (DEEP-K-004.discovery.md:L201): `E/DEP` — Sequencing honesty PSR-1: arm-1 builds IN I-27/G-6 runner shape N-1 later binds (compose-not-fork). PS-6 wire-dark rate can bank as auditor rider, not a unit of this plan. — plan-anchor=`K-18a:L26-L30`
- **K-004/K18a-E005** [SRC/E] (DEEP-K-004.discovery.md:L202): `E/FLOW` — RECON R-A: count distinct F-4-proven red→green pairs in newest ssb.db; threshold >=10 for journal suite. — plan-anchor=`K-18a:L33-L36, L103-L105`
- **K-004/K18a-E006** [SRC/E] (DEEP-K-004.discovery.md:L203): `E/FLOW` — RECON R-B: locate I-27 ConvergenceCurveMeasured/bench_convergence compose target (args_bench:1027 parse exists but no convergence_runner.rs listed). — plan-anchor=`K-18a:L37-L39, L106-L107`
- **K-004/K18a-E007** [SRC/E] (DEEP-K-004.discovery.md:L204): `E/FLOW` — RECON R-C: ONNX staged vs BM25-only; honest retrieval_mode tag. — plan-anchor=`K-18a:L40-L42, L108-L110`
- **K-004/K18a-E008** [SRC/E] (DEEP-K-004.discovery.md:L205 | SURFACE): `E/FLOW` — RECON R-D: SSB row reuse prefer existing bench family +Kind; net 0..+1. — plan-anchor=`K-18a:L43-L45, L118-L119`
- **K-004/K18a-E009** [SRC/E] (DEEP-K-004.discovery.md:L206): `E/DATA` — Fixture corpus: benchmarks/case_reuse_fixtures/ with >=20 synthetic (proven-case, fresh-recurrence) pairs across >=4 failure_taxonomy classes + >=20 NON-recurrent distractors; signatures from real failure_taxonomy classifier never hand-typed. — plan-anchor=`K-18a:L56-L63`
- **K-004/K18a-E010** [SRC/E] (DEEP-K-004.discovery.md:L207 | SURFACE): `E/CAP` — case_reuse_runner.rs arm-1 only: query retrieval with fresh failure signature; report recurrence_hit_rate (top-k contains proven case) AND false_match_rate (distractors) per k in {1,3,5} under ELAI_SEED; ZERO prompt-assembly/reuse-into-generation imports. — plan-anchor=`K-18a:L65-L71`
- **K-004/K18a-E011** [SRC/E] (DEEP-K-004.discovery.md:L208 | SURFACE): `E/SURFACE` — CLI: `elai bench case-reuse --suite fixture --seed 1234 --json` (and optionally `--suite journal` if F-2); mirror parse_bench_safedelta_args at :896; sub must appear in unknown-sub error string at :87. — plan-anchor=`K-18a:L76-L84, L155-L156`
- **K-004/K18a-E012** [SRC/E] (DEEP-K-004.discovery.md:L209 | SURFACE): `E/CFG` — FORK F-1 settled: claims_policy_lint absent → PACKAGE legs OUT. FORK F-2: R-A <10 → FIXTURE-only; >=10 add journal suite. FORK F-3: no convergence → safedelta_runner shape. FORK F-4: ONNX unstaged → retrieval_mode="bm25_only"; staged → "hybrid"; never fabricate hybrid from BM25. — plan-anchor=`K-18a:L101-L110`
- **K-004/K18a-E013** [SRC/E] (DEEP-K-004.discovery.md:L210): `E/FAIL` — ABORT-1 gate-manifest frozen edit; ABORT-2 any reuse-into-generation; ABORT-3 same integration failed 3 times; ABORT-4 SSB schema bump with no reusable row. — plan-anchor=`K-18a:L112-L119`
- **K-004/K18a-E014** [SRC/E] (DEEP-K-004.discovery.md:L211 | SURFACE): `E/TEST` — Same-seed byte-identity (ELAI_SEED=1234 twice); different-seed 5678 valid-and-different (anti-hardcoding); empty corpus fails-loud (R1). — plan-anchor=`K-18a:L86-L90, L122-L124, L157-L158`
- **K-004/K18a-E015** [SRC/E] (DEEP-K-004.discovery.md:L212 | SURFACE): `E/TEST` — NEGATIVE: `--match-everything` debug flag (cfg(test)-only) collapses false_match_rate to ~1.0 proving metric discriminates. — plan-anchor=`K-18a:L88-L89, L125-L126, L140-L143`
- **K-004/K18a-E016** [SRC/E] (DEEP-K-004.discovery.md:L213): `E/DATA` — K-18a-decisions.md must carry fixture-scoped KD-1 tag: "recurrence_hit_rate is a FIXTURE number; not the compounding moat number; reuse delta BENCH-PENDING on VCG" + deferred-legs ledger. — plan-anchor=`K-18a:L95-L97, L160-L161`
- **K-004/K18a-E017** [SRC/E] (DEEP-K-004.discovery.md:L214 | SURFACE): `E/TEST` — VERIFY off-agent: wiring_contract_check --run-trigger; production_caller_check no DARK; maintainability_check; scoped cargo test + cargo check; INTEGRATION_SMOKE drives real entrypoint seeds 1234/5678 + negative arm (R10). — plan-anchor=`K-18a:L121-L137`
- **K-004/K18a-E018** [SRC/E] (DEEP-K-004.discovery.md:L215): `E/LIMIT` — Red-team Attack C failed: fixture hit-rate cannot be marketed as compounding moat (KD-1 tag + no dossier + K-18 stub_ban). Attack D failed: sneak reuse blocked by ABORT-2. — plan-anchor=`K-18a:L147-L152`
- **K-004/K18a-E019** [SRC/E] (DEEP-K-004.discovery.md:L216): `E/DATA` — Files: CREATE case_reuse_runner.rs, case_reuse_fixtures/, K-18a-decisions.md, contract/determinism tests; MODIFY args_bench.rs; SSB only if R-D forces via ABORT-4 handoff. — plan-anchor=`K-18a:L170-L176`
- **K-004/K18a-E020** [SRC/E] (DEEP-K-004.discovery.md:L217): `E/FLOW` — Roles: executor builds Moves 2-4 + red-first tests; AUDITOR runs every VERIFY/smoke/checker (main model never runs tests); memory-writer after PASS. — plan-anchor=`K-18a:L165-L168`
- **K-004/K18a-E021** [SRC/E] (DEEP-K-004.discovery.md:L218): `E/DET` — Hit-rate/false-match recompute from journal (GD-2); entropy-quarantine via ClockProvider/EntropySource if flakes. — plan-anchor=`K-18a:L122-L124, L92-L93`
- **K-004/K18a-E022** [SRC/E] (DEEP-K-004.discovery.md:L219): `E/DEFER` — When K-1 executes it picks K-18 Tasks 3-5 as recorded follow-ons; Task 2 remains VCG-pending. — plan-anchor=`K-18a:L22-L23, L160-L161`
- **K-004/K18a-I001** [SRC/I] (DEEP-K-004.discovery.md:L220 | SURFACE): `I/DEP` — K-18a is the runnable subset that later K-4 cites as the proven 4-file CLI template (case_reuse_runner + bench_case_reuse). — plan-anchor=`K-18a:L15-L16; (cross-file via K-4)`
- **K-004/K18a-A001** [SRC/A] (DEEP-K-004.discovery.md:L221): `A/EVENT` — Exact SSB row family/Kind chosen only after R-D; may be ConvergenceCurveMeasured extension or new. — plan-anchor=`K-18a:L43-L45`
- **K-004/K18a-A002** [SRC/A] (DEEP-K-004.discovery.md:L222): `A/STATUS` — Relationship to full K-18 is "execute SAFE scope now"; full K-18 economics remain outside this file's deliverables. — plan-anchor=`K-18a:L1-L12`
- **K-004/K18a-C001** [SRC/C] (DEEP-K-004.discovery.md:L223): `C/STATUS` — Summary discovery A-05 states K-18a exists but is not described; this deep chunk invents the full inventory the summary omitted. — plan-anchor=`K-18a:L1-L178`

#### Unit `K19` — 22 atoms (A=1, E=20, I=1)

- **K-004/K19-E001** [SRC/E] (DEEP-K-004.discovery.md:L233): `E/STATUS` — K-19 is PLANNED WAVE KW-3 CANT-WRECK+DISCIPLINE E-23 class; doctrine file + conditional-routing + K-1 lint extension; NO runtime code, NO SSB. — plan-anchor=`K-19:L1-L3, L88-L89`
- **K-004/K19-E002** [SRC/E] (DEEP-K-004.discovery.md:L234): `E/CAP` — Ships `.claude/rules/self-improvement-fitness.md` with four-clause invariant SF-1..SF-4 as HARD RULE pre-gate for self-improvement / case-graph / archive-promotion / playbook-curation / memory-reuse / DGM / fitness-scoring tasks. — plan-anchor=`K-19:L4-L5, L15-L20, L39-L39, L47-L48`
- **K-004/K19-E003** [SRC/E] (DEEP-K-004.discovery.md:L235): `E/CFG` — SF-1 EXECUTION-GROUNDED-FITNESS-ONLY: fitness is NEVER byte-hash (forbids LIVE GAP-1 derived_alpha=blake3(config_bytes)) and NEVER LLM-judged "this helped"; IS off-agent gate-PROVEN pass on HELD-OUT recurrence (D-6/H-47/H-43). "Retrieved/used/count went up" is NOT fitness. — plan-anchor=`K-19:L16-L16, K-19:L61-L62`
- **K-004/K19-E004** [SRC/E] (DEEP-K-004.discovery.md:L236): `E/CFG` — SF-2 STRONGER-THAN-GENERATOR + ABSTENTION-AWARE: different pretraining family (D-13/VD-4); credits abstention (RD-1, abstain bit from D-16/H-39 never re-derived); trap detection BEHAVIORAL with MEASURED recall on frozen contrastive corpus (H-44), NEVER keyword-grep (forbids LIVE GAP-2). — plan-anchor=`K-19:L17-L17, L61-L61`
- **K-004/K19-E005** [SRC/E] (DEEP-K-004.discovery.md:L237 | SURFACE): `E/CFG` — SF-3 OBSERVE-FIRST + NEVER-INTO-THE-AUDITOR: reuse-into-GENERATION is OBSERVE-ONLY first (GD-4), DEFAULT-OFF with wire-dark-allowlist + promotion plan (W-2/R9), NEVER injected into auditor/critic/adversary (H-31 firewall). — plan-anchor=`K-19:L18-L18, L63-L63`
- **K-004/K19-E006** [SRC/E] (DEEP-K-004.discovery.md:L238): `E/CFG` — SF-4 STALENESS + SUPERSESSION + REPLAY-SAFE: re-derive against CURRENT commit+AST (J-21) before reuse; H-32 28d TTL; G-10 drift invalidation; superseded HARD-FILTERED never narrated never DELETED (RV); poisoning write-time + retrieval-auth + write-quarantine (E-5/MINJA); per-namespace isolation (J-59). — plan-anchor=`K-19:L19-L19, L64-L64`
- **K-004/K19-E007** [SRC/E] (DEEP-K-004.discovery.md:L239): `E/INT` — conditional-routing.md loads the rule on self-improvement / case-graph / archive-promotion / playbook-curation / memory-reuse / DGM tasks. — plan-anchor=`K-19:L20-L20, L39-L39, L48-L48, L56-L56`
- **K-004/K19-E008** [SRC/E] (DEEP-K-004.discovery.md:L240 | SURFACE): `E/CAP` — EXTEND (not fork) K-1 claims_policy_lint recogniser: "self-improving / learns from your codebase / gets better with use / gets smarter over time" FAILS unless execution-grounded-fitness correction + [DESIGN]/[BENCH-PENDING on K-18] tag; "fitness/reward by byte-hash or LLM-judged-helpful" design FAILS. — plan-anchor=`K-19:L41-L41, L49-L49, L88-L88`
- **K-004/K19-E009** [SRC/E] (DEEP-K-004.discovery.md:L241 | SURFACE): `E/SURFACE` — Negative fixture `docs/thefinalglance/K-plan/_fixtures/self_improving_negative.md`: untagged "learns from your code"; fitness by retrieval/LLM-helpful; case injected into AUDITOR — lint fails (a), rule catches (b)/(c). — plan-anchor=`K-19:L42-L42, L79-L79, L105-L105`
- **K-004/K19-E010** [SRC/E] (DEEP-K-004.discovery.md:L242): `E/LIMIT` — Does NOT author VCG capability (J track); does NOT introduce a runtime gate (auditor-applied); does NOT restate VD-1..VD-5 or H-43..H-48 mechanics (BINDS them); does NOT claim a measured number (proof is K-18 bench). — plan-anchor=`K-19:L5-L5, L88-L89, L115-L116`
- **K-004/K19-E011** [SRC/E] (DEEP-K-004.discovery.md:L243): `E/DEP` — Cites runtime archetypes H-43..H-48, D-6, D-13, H-39, H-31, J-21, H-32, G-10, E-5, J-59; LIVE anti-patterns at darwin_admit_loop.rs GAP-1 and trap_tasks.rs GAP-2. — plan-anchor=`K-19:L7-L8, L22-L26, L50-L52`
- **K-004/K19-E012** [SRC/E] (DEEP-K-004.discovery.md:L244): `E/DEP` — Soft-depends K-1 lint; pre-gates VCG J-track; defines reuse-fitness K-18's reuse arm consumes; sibling shape to K-8/K-14/K-16. — plan-anchor=`K-19:L8-L9, L27-L28`
- **K-004/K19-E013** [SRC/E] (DEEP-K-004.discovery.md:L245 | SURFACE): `E/INT` — Production: conditional-routing load (JD-1); auditor runs claims_policy_lint; VCG executor + K-18 reuse arm MUST obey. — plan-anchor=`K-19:L55-L58`
- **K-004/K19-E014** [SRC/E] (DEEP-K-004.discovery.md:L246 | SURFACE): `E/SURFACE` — frontend_pair N/A (W-6 justified doctrine + recogniser). — plan-anchor=`K-19:L66-L67`
- **K-004/K19-E015** [SRC/E] (DEEP-K-004.discovery.md:L247): `E/EVENT` — expect_ssb: none (E-23/doctrine; JD-1 harness evidence). — plan-anchor=`K-19:L70-L70`
- **K-004/K19-E016** [SRC/E] (DEEP-K-004.discovery.md:L248): `E/TEST` — Lint tests: tagged pass; untagged learns-from-code fail; byte-hash design fail; LLM-helpful/retrieval-count fail; auditor-injection fail; empty input fails-loud. — plan-anchor=`K-19:L98-L99`
- **K-004/K19-E017** [SRC/E] (DEEP-K-004.discovery.md:L249 | SURFACE): `E/DATA` — Creates self-improvement-fitness.md (~70 LOC), self_improving_negative.md, K-19-decisions.md; MODIFY conditional-routing.md (~2 LOC), claims_policy_lint.py (~12 LOC). — plan-anchor=`K-19:L101-L106`
- **K-004/K19-E018** [SRC/E] (DEEP-K-004.discovery.md:L250 | SURFACE): `E/EVIDENCE` — Paper grounding [EXTERNAL not ELAI numbers]: Misevolve 2509.26354 refusal collapse; DGM 2505.22954; MPBench 2606.04329; auditor-memory-firewall 2510.09905; JudgeBench. — plan-anchor=`K-19:L111-L111`
- **K-004/K19-E019** [SRC/E] (DEEP-K-004.discovery.md:L251 | SURFACE): `E/DEFER` — OUT-OF-K technical companion (VCG J-track full list in K-18 OUT-OF-K): mint-on-F-4 case node, deterministic edges, MEMORY/CASE canvas altitude, gated reuse loop, episodic→skill→rule ladder (I-12→C-25→B-23 L3) — all BOUND by SF-1..SF-4. — plan-anchor=`K-19:L118-L118`
- **K-004/K19-E020** [SRC/E] (DEEP-K-004.discovery.md:L252): `E/LIMIT` — K-1/K-18 recogniser overlap: compose one recogniser two claim shapes, never two forks; whoever lands first extends the other. — plan-anchor=`K-19:L117-L117`
- **K-004/K19-I001** [SRC/I] (DEEP-K-004.discovery.md:L253): `I/DEP` — One-line cross-link so K-18 compounding-state differentiator is governed by SF-1 without a new dossier row (K-18 owns the row). — plan-anchor=`K-19:L40-L40`
- **K-004/K19-A001** [SRC/A] (DEEP-K-004.discovery.md:L254): `A/TEST` — SF-clause runtime enforcement until VCG exists is auditor-applied reviewed-design catch, not a code assertion beyond lint/load. — plan-anchor=`K-19:L99-L99`

#### Unit `K23` — 39 atoms (A=2, E=36, I=1)

- **K-004/K23-E001** [SRC/E] (DEEP-K-004.discovery.md:L264): `E/STATUS` — K-23 is PLANNED MPW wave NET-NEW capability folded into K; multi-slice TRACK as ONE file U1..U6; first cross-REPO axis; single-root spine otherwise untouched. — plan-anchor=`K-23:L1-L3`
- **K-004/K23-E002** [SRC/E] (DEEP-K-004.discovery.md:L265): `E/CAP` — Thin cross-repo LAYER: cheap DEPENDENCY graph for planning + LIVE SUBAGENT DELEGATION for code; ~80% composition of existing machinery; must NOT break per-root SSB as authoritative record (GD-2). — plan-anchor=`K-23:L5-L5`
- **K-004/K23-E003** [SRC/E] (DEEP-K-004.discovery.md:L266): `E/LIMIT` — Cross-repo layer is DERIVED re-computable projection, NEVER second primary store, NEVER centralized cloud store. — plan-anchor=`K-23:L5-L5, L70-L71, L98-L98`
- **K-004/K23-E004** [SRC/E] (DEEP-K-004.discovery.md:L267): `E/STATUS` — Existing stores are pure functions of one project_root (paths.rs:173, BM25_PROJECT_ID "default"); only cross-project mechanism today is Cosmos ~/.elai/cosmos/ (B-42/C-16). — plan-anchor=`K-23:L5-L5, L16-L17`
- **K-004/K23-E005** [SRC/E] (DEEP-K-004.discovery.md:L268): `E/CFG` — Two-mode design: CONNECTED (shared symbols/imports = Sound edges) vs DISCONNECTED (federation only, empty edges); default federation; promote to edges ONLY where sound cross-repo reference proven. — plan-anchor=`K-23:L13-L13`
- **K-004/K23-E006** [SRC/E] (DEEP-K-004.discovery.md:L269 | SURFACE): `E/CAP` — U1 WORKSPACE OBJECT: workspace.toml (member project_roots + relationship hints produces-for\ — plan-anchor=`consumes-from\`
- **K-004/K23-E007** [SRC/E] (DEEP-K-004.discovery.md:L270 | SURFACE): `E/CFG` — Capability behind ONE default-OFF `ELAI_WORKSPACE_ENABLE` with wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-23-promotion, ON-path smoke). — plan-anchor=`K-23:L37-L37, L103-L103, L110-L110`
- **K-004/K23-E008** [SRC/E] (DEEP-K-004.discovery.md:L271): `E/CAP` — U2 CROSS_REPO_DEP_GRAPH: derived cross_repo_edges keyed by blake3(concat member DKG index-manifest hashes); re-derives when any member DKG changes. — plan-anchor=`K-23:L38-L38`
- **K-004/K23-E009** [SRC/E] (DEEP-K-004.discovery.md:L272): `E/DATA` — Edge fields: {from_repo,to_repo,kind:PackageDep\ — plan-anchor=`ApiDep\`
- **K-004/K23-E010** [SRC/E] (DEEP-K-004.discovery.md:L273 | SURFACE): `E/SEC` — SOUND edges may drive gates; HEURISTIC edges surface with warning badge, NEVER asserted as call edge (R5/R6 — wrong edge worse than no edge). — plan-anchor=`K-23:L38-L38, L73-L73, L99-L99`
- **K-004/K23-E011** [SRC/E] (DEEP-K-004.discovery.md:L274 | SURFACE): `E/SURFACE` — `elai workspace map` renders Sound vs Heuristic edges; fixture expects exactly ONE Sound types←api edge, ZERO false edges. — plan-anchor=`K-23:L38-L38, L83-L83`
- **K-004/K23-E012** [SRC/E] (DEEP-K-004.discovery.md:L275 | SURFACE): `E/CAP` — U3 FEDERATED RETRIEVAL: `elai workspace search <id> "<q>"` fans query to EACH member pipeline in parallel; RRF-merge k=60; every hit carries per-repo provenance; NO new index/embedding store. — plan-anchor=`K-23:L39-L39`
- **K-004/K23-E013** [SRC/E] (DEEP-K-004.discovery.md:L276): `E/DATA` — 3E RepoMap/CodeContext slots carry multi-repo provenance, each line labelled with its repo. — plan-anchor=`K-23:L39-L39`
- **K-004/K23-E014** [SRC/E] (DEEP-K-004.discovery.md:L277 | SURFACE): `E/CAP` — U4 CROSS-REPO BLAST RADIUS: extend B-29 cone + J-16 gate across Sound cross_repo_edges; HEURISTIC contributes "possible consumer, unverified" advisory only. CLI `elai workspace impact <id> <repo>:<symbol>`. — plan-anchor=`K-23:L40-L40`
- **K-004/K23-E015** [SRC/E] (DEEP-K-004.discovery.md:L278): `E/DET` — U4 is deterministic; LLM never adjudicates (TD-3); observe-only first (GD-4); precision-first (false consumer worse than miss). — plan-anchor=`K-23:L40-L40`
- **K-004/K23-E016** [SRC/E] (DEEP-K-004.discovery.md:L279 | SURFACE): `E/CAP` — U5 `workspace orchestrate`: lightweight workspace FSM above per-root FSM. — plan-anchor=`K-23:L41-L41`
- **K-004/K23-E017** [SRC/E] (DEEP-K-004.discovery.md:L280): `E/STATE` — FSM ring: WORKSPACE_PLANNING → WORKSPACE_DELEGATING → WORKSPACE_MERGING → WORKSPACE_COMPLETE. — plan-anchor=`K-23:L41-L41, L52-L52`
- **K-004/K23-E018** [SRC/E] (DEEP-K-004.discovery.md:L281): `E/FLOW` — PLANNING: derive/refresh edges, touched-member set, Kahn-order over Sound edges (phase-3G); cycle = tripwire. — plan-anchor=`K-23:L41-L41`
- **K-004/K23-E019** [SRC/E] (DEEP-K-004.discovery.md:L282): `E/FLOW` — DELEGATING: per member Tier-C sub-agent into J-80 worktree running that member's OWN FSM; emits to MEMBER's SSB; poll via J-2 task_status + H-36; cap = orchestrate parallel_cap (3); independents parallel, dependents serialize. — plan-anchor=`K-23:L41-L41`
- **K-004/K23-E020** [SRC/E] (DEEP-K-004.discovery.md:L283): `E/FLOW` — MERGING: collect member PR/branch; R10 INDEPENDENT cross-repo integration smoke on HARD input; missing integration evidence = FAIL. — plan-anchor=`K-23:L41-L41, L101-L101`
- **K-004/K23-E021** [SRC/E] (DEEP-K-004.discovery.md:L284 | SURFACE): `E/LIMIT` — Controller roots at WORKSPACE synthetic root, never edits member code (main-llm-role); worker git read-only; orchestrate 4 tripwires at workspace grain. — plan-anchor=`K-23:L41-L41`
- **K-004/K23-E022** [SRC/E] (DEEP-K-004.discovery.md:L285): `E/CAP` — U6 episodic resume: workspace.ssb.db + each member H-51 handoff digest reconstructs cross-repo narrative WITHOUT copying member state. — plan-anchor=`K-23:L42-L42`
- **K-004/K23-E023** [SRC/E] (DEEP-K-004.discovery.md:L286): `E/SEC` — Provenance: every cross-repo fact/edge carries source_repo; J-21 extended with source_repo + C-48 TrustTier; low-trust member cannot silently poison high-trust without J-21 validation against B's commit+AST + provenance badge. — plan-anchor=`K-23:L42-L42`
- **K-004/K23-E024** [SRC/E] (DEEP-K-004.discovery.md:L287 | SURFACE): `E/PRIV` — Workspace belongs to a J-59 seat; cross-seat workspace sharing OUT of scope (same later opt-in as cross-seat memory). — plan-anchor=`K-23:L42-L42, L75-L75`
- **K-004/K23-E025** [SRC/E] (DEEP-K-004.discovery.md:L288 | SURFACE): `E/SURFACE` — canvas WORKSPACE altitude beside J-31/J-60/J-98: nodes=member repos, edges=cross_repo_edges (colored Sound vs Heuristic), in-flight delegation lights member node, workspace-session+PRs temporal layer; re-derived byte-identically from workspace.ssb.db. — plan-anchor=`K-23:L42-L42, L78-L79`
- **K-004/K23-E026** [SRC/E] (DEEP-K-004.discovery.md:L289): `E/EVENT` — +WorkspaceEvent Pattern-2; WorkspaceEventKind = Created\ — plan-anchor=`MemberAdded\`
- **K-004/K23-E027** [SRC/E] (DEEP-K-004.discovery.md:L290 | SURFACE): `E/SURFACE` — CLI verbs: `elai workspace create\ — plan-anchor=`add\`
- **K-004/K23-E028** [SRC/E] (DEEP-K-004.discovery.md:L291 | SURFACE): `E/DATA` — Module tree workspace/{mod,manifest,dep_graph,federated_search,cross_repo_cone,workspace_orchestrate,episodic}.rs (~1100 LOC); CLI workspace_cmd.rs; blast_radius.rs +40 LOC; dashboard WORKSPACE altitude ~120 LOC. — plan-anchor=`K-23:L120-L125`
- **K-004/K23-E029** [SRC/E] (DEEP-K-004.discovery.md:L292): `E/TEST` — Contract k23_workspace_memory_contract.rs: Sound edge fixture; disconnected empty edges; federated search repo-B top-k; impact real consumer zero false; Heuristic not in asserted radius; orchestrate Kahn+Tier-C+R10 fail on missing smoke; store path global; GD-2 re-derive; source_repo poisoning firewall. — plan-anchor=`K-23:L116-L117`
- **K-004/K23-E030** [SRC/E] (DEEP-K-004.discovery.md:L293): `E/LIMIT` — Stub bans: no second primary store; no cloud store; no store under member root; no Heuristic-driven gate; no fabricated disconnected edge; no cross-seat share v1; no re-implement per-root stack; blake3 only; no .unwrap. — plan-anchor=`K-23:L69-L77`
- **K-004/K23-E031** [SRC/E] (DEEP-K-004.discovery.md:L294): `E/CFG` — SSB net +1 WorkspaceEvent sequential from LIVE 268/577 (re-grep W-9). — plan-anchor=`K-23:L105-L105`
- **K-004/K23-E032** [SRC/E] (DEEP-K-004.discovery.md:L295 | SURFACE): `E/DEFER` — Until J-18 lands, U2/U4 edges Heuristic-only advisory and impact observe-only; Sound/Heuristic split authored day one for flag-flip promotion. — plan-anchor=`K-23:L137-L137`
- **K-004/K23-E033** [SRC/E] (DEEP-K-004.discovery.md:L296 | SURFACE): `E/LIMIT` — v1 caps workspace at small member set (common 2-4 repo case); large-org federation later rider. — plan-anchor=`K-23:L138-L138`
- **K-004/K23-E034** [SRC/E] (DEEP-K-004.discovery.md:L297 | SURFACE): `E/LIMIT` — Workspace controller must never touch a member's FSM — polls J-2/H-36 only. — plan-anchor=`K-23:L139-L139`
- **K-004/K23-E035** [SRC/E] (DEEP-K-004.discovery.md:L298): `E/DEFER` — Polygraph org-shared team memory OUT until J-59 isolation extended to opt-in sharing (separate later decision, not prerequisite). — plan-anchor=`K-23:L13-L13, L143-L143`
- **K-004/K23-E036** [SRC/E] (DEEP-K-004.discovery.md:L299 | SURFACE): `E/EVIDENCE` — Grounding: multi-project-memory-analysis-2026-06-26.md §7.0..§7.12; Polygraph [EXTERNAL] SPATIAL+TEMPORAL axes; ELAI reuses listed substrate. — plan-anchor=`K-23:L10-L10, L130-L134`
- **K-004/K23-I001** [SRC/I] (DEEP-K-004.discovery.md:L300): `I/DEP` — Parallel-safe with K-24 and K-1..K-22 doc/bench/doctrine; NOT parallel with orchestrate internals (U5) or B-29/J-16 cone (U4). — plan-anchor=`K-23:L9-L9`
- **K-004/K23-A001** [SRC/A] (DEEP-K-004.discovery.md:L301): `A/FLOW` — Whether U4 ships before J-18 (Heuristic-advisory) or waits is an execution-time decision. — plan-anchor=`K-23:L137-L137`
- **K-004/K23-A002** [SRC/A] (DEEP-K-004.discovery.md:L302 | SURFACE): `A/SURFACE` — Exact drawer/window for canvas WORKSPACE altitude decided at edit (rides J-31/J-60/J-98 pattern). — plan-anchor=`K-23:L78-L79`

#### Unit `K25` — 24 atoms (A=1, E=22, I=1)

- **K-004/K25-E001** [SRC/E] (DEEP-K-004.discovery.md:L312): `E/STATUS` — K-25 is PLANNED CAW wave; extends K-8 never re-authors it; KD-4 made continuous; NO SSB; advisory cache boundary — never a done-gate; kill/de-scope output is PROPOSAL never auto-applied. — plan-anchor=`K-25:L1-L4, L91-L94`
- **K-004/K25-E002** [SRC/E] (DEEP-K-004.discovery.md:L313): `E/STATUS` — PROOF_SPINE: none (defensive depreciation discipline; spine-exempt; user-acked 2026-07-05). — plan-anchor=`K-25:L5-L6`
- **K-004/K25-E003** [SRC/E] (DEEP-K-004.discovery.md:L314 | SURFACE): `E/CAP` — Continuous watch: (a) dated file-based provider-surface snapshot; (b) deterministic overlap checker vs feature-map ABSORBED/AT-RISK/DURABLE tags; (c) staleness trigger on K-8's conditional-routing lane so efficiency-class task on stale snapshot must refresh before EXECUTING. — plan-anchor=`K-25:L11-L17`
- **K-004/K25-E004** [SRC/E] (DEEP-K-004.discovery.md:L315 | SURFACE): `E/DATA` — Snapshot `workflow/status/provider-surface.json` schema v1: one row per watched capability {provider, capability_key, status: NATIVE\ — plan-anchor=`PARTIAL\`
- **K-004/K25-E005** [SRC/E] (DEEP-K-004.discovery.md:L316): `E/FAIL` — Row without both source_url and verified_date is invalid — fail-closed parse exit 2. — plan-anchor=`K-25:L61-L62, L73-L74, L93-L93`
- **K-004/K25-E006** [SRC/E] (DEEP-K-004.discovery.md:L317 | SURFACE): `E/SURFACE` — Tool `.claude/tools/absorption_watch.py` (stdlib-only, offline, --json, --max-age-days N default 90). — plan-anchor=`K-25:L63-L66, L81-L81, L119-L119`
- **K-004/K25-E007** [SRC/E] (DEEP-K-004.discovery.md:L318): `E/OBS` — Verdicts: NATIVE_OVERLAP (PLANNED/PARTIAL unit × NATIVE capability), AT_RISK_STALE (AT-RISK tag older than max-age), CLEAN. — plan-anchor=`K-25:L64-L66`
- **K-004/K25-E008** [SRC/E] (DEEP-K-004.discovery.md:L319 | SURFACE): `E/FAIL` — Exit codes: 0 clean / 1 overlaps found / 2 config-or-parse error fail-closed / 3 snapshot stale past --max-age-days. — plan-anchor=`K-25:L66-L66, L112-L113, L119-L119`
- **K-004/K25-E009** [SRC/E] (DEEP-K-004.discovery.md:L320 | SURFACE): `E/LIMIT` — Unit-to-capability join is keyword/tag-based and declared UNSOUND in --help (advisory, R5-style honesty); LLM adjudicates each punch row. — plan-anchor=`K-25:L67-L68, L150-L152`
- **K-004/K25-E010** [SRC/E] (DEEP-K-004.discovery.md:L321 | SURFACE): `E/SURFACE` — Rule `.claude/rules/absorption-watch.md` (~15 lines): on efficiency-class task run absorption_watch.py FIRST; exit 3 = refresh snapshot (WebSearch, re-date) BEFORE EXECUTING; exit 1 = surface punch rows as kill/de-scope PROPOSAL. — plan-anchor=`K-25:L69-L72, L82-L82`
- **K-004/K25-E011** [SRC/E] (DEEP-K-004.discovery.md:L322): `E/INT` — Conditional-routing load line (same trigger as K-8's rule); production harness rule-load is the R7 path. — plan-anchor=`K-25:L72-L72, L88-L89, L120-L120`
- **K-004/K25-E012** [SRC/E] (DEEP-K-004.discovery.md:L323): `E/LIMIT` — NO auto-applied kill/de-scope (user confirms — house rule no-removal-without-confirmation). — plan-anchor=`K-25:L72-L72, L92-L92, L122-L122`
- **K-004/K25-E013** [SRC/E] (DEEP-K-004.discovery.md:L324 | SURFACE): `E/LIMIT` — NO done-gate consumption of checker verdict (advisory cache; R8 untouched); mirror elai_status_query R8 note. — plan-anchor=`K-25:L74-L75, L94-L94, L121-L121`
- **K-004/K25-E014** [SRC/E] (DEEP-K-004.discovery.md:L325): `E/LIMIT` — NO network calls in the checker (stdlib offline); WebSearch refresh is the LLM's job. — plan-anchor=`K-25:L37-L38, L95-L95`
- **K-004/K25-E015** [SRC/E] (DEEP-K-004.discovery.md:L326 | SURFACE): `E/DEP` — Extends K-8 tags + provider-absorption-pregate.md; composes elai_status_query.py advisory-cache pattern; sibling to K-9 model-refresh axis (disjoint); G-10 is runtime exogenous watcher — share nothing. — plan-anchor=`K-25:L21-L23, L40-L49, L84-L86`
- **K-004/K25-E016** [SRC/E] (DEEP-K-004.discovery.md:L327 | SURFACE): `E/SURFACE` — frontend_pair N/A — harness tool + rule + status file (W-6). — plan-anchor=`K-25:L96-L97`
- **K-004/K25-E017** [SRC/E] (DEEP-K-004.discovery.md:L328): `E/EVENT` — expect_ssb: none (harness leg; JD-1 evidence). — plan-anchor=`K-25:L100-L100`
- **K-004/K25-E018** [SRC/E] (DEEP-K-004.discovery.md:L329): `E/TEST` — Fixture snapshot trio (clean / overlap / stale / malformed-row) asserts four exit codes; no Rust contract. — plan-anchor=`K-25:L131-L133`
- **K-004/K25-E019** [SRC/E] (DEEP-K-004.discovery.md:L330 | SURFACE): `E/DATA` — Creates provider-surface.json, absorption_watch.py (~150 LOC), absorption-watch.md, K-25-decisions.md; MODIFY conditional-routing.md (~1 LOC). — plan-anchor=`K-25:L135-L140`
- **K-004/K25-E020** [SRC/E] (DEEP-K-004.discovery.md:L331): `E/FLOW` — W-1 done = rule fires on live efficiency-class task and exit code steers session (JD-1), not mere file existence. — plan-anchor=`K-25:L126-L127`
- **K-004/K25-E021** [SRC/E] (DEEP-K-004.discovery.md:L332 | SURFACE): `E/LIMIT` — Watch-set creep: cap snapshot to capabilities that overlap an existing ELAI unit; general provider-news feed out of scope. — plan-anchor=`K-25:L155-L156`
- **K-004/K25-E022** [SRC/E] (DEEP-K-004.discovery.md:L333 | SURFACE): `E/EVIDENCE` — Grounding: spec-eval 2026-07-05 §4 L2 / §5 ADD-2; K-8; KS-3 report 17 [EXTERNAL]; elai_status_query/elai-status-monitor pattern. — plan-anchor=`K-25:L142-L147`
- **K-004/K25-I001** [SRC/I] (DEEP-K-004.discovery.md:L334): `I/DEP` — K-9 cadence run is a natural snapshot-refresh trigger but files remain disjoint. — plan-anchor=`K-25:L46-L47`
- **K-004/K25-A001** [SRC/A] (DEEP-K-004.discovery.md:L335): `A/CFG` — Exact initial capability watch-set beyond "KS-3 absorbed set + AT-RISK tags" is filled at first snapshot authoring. — plan-anchor=`K-25:L59-L62`

### K-004 cross-file interactions (namespaced K-004/IX-*)

- **K-004/IX-001** [IX] (DEEP-K-004.discovery.md:L343): K4-E015, K5-E001/E008 | K-5 depends on K-4's locked coding ceiling; policy must fit under it on >=3 real runs. | K-4:L51; K-5:L3, L60-L61
- **K-004/IX-002** [IX] (DEEP-K-004.discovery.md:L344): K4-E038, K18a-E011 | K-4 4-file `gate-tax` wiring explicitly mirrors K-18a case-reuse bench CLI pattern. | K-4:L180-L183; K-18a:L76-L79
- **K-004/IX-003** [IX] (DEEP-K-004.discovery.md:L345): K4-E014, K18a-E010 | K-4 composes case_reuse_runner shape proven by K-18a arm-1. | K-4:L48-L49, L160-L161; K-18a:L65-L71
- **K-004/IX-004** [IX] (DEEP-K-004.discovery.md:L346): K4-E027, K18a-E008 | GateTaxMeasured "like K-18a" signed .rra / prefer-reuse SSB discipline shared with case-reuse beacon path. | K-4:L190-L194; K-18a:L43-L45
- **K-004/IX-005** [IX] (DEEP-K-004.discovery.md:L347): K11-E002/E009, K7-E003 | K-11 human evidence render + K-7 brief both sell buyer-side evidence; K-7 points at K-6 pack while K-11 extends K-3 PR surface (KD-2 chain). | K-11:L5-L6; K-7:L5-L10
- **K-004/IX-006** [IX] (DEEP-K-004.discovery.md:L348): K7-E008, K19-E008, K25-E001 | Multiple units extend/reuse K-1 claims_policy_lint (K-7 numbers; K-19 self-improving claims) without forking; K-25 is advisory-tool not lint but shares "never auto-apply" human-gate culture with K-7/K-8. | K-7:L47; K-19:L41; K-25:L12
- **K-004/IX-007** [IX] (DEEP-K-004.discovery.md:L349): K19-E012, K18a-E002/E016 | K-19 defines reuse-fitness that K-18/K-18a reuse arm must obey; K-18a ships observe-only with explicit KD-1 fixture tag pending VCG+K-19 activation. | K-19:L8-L9; K-18a:L11-L12, L95-L97
- **K-004/IX-008** [IX] (DEEP-K-004.discovery.md:L350): K19-E019, K18a-E003 | VCG J-track capability is OUT of both K-19 and K-18a; both pre-gate/measure around it. | K-19:L118; K-18a:L17-L18
- **K-004/IX-009** [IX] (DEEP-K-004.discovery.md:L351): K25-E003/E015, K5-E007 | K-25 extends K-8 pre-gate lane; K-5 freezes policy hash in same gate-manifest R8 machinery family. | K-25:L40-L43; K-5:L56-L57
- **K-004/IX-010** [IX] (DEEP-K-004.discovery.md:L352): K23-E019, K4-E012 | K-23 U5 reuses Tier-C/J-80 isolation; K-4 VETOES product-level gate-bypass — both protect freeze/isolation invariants of the spine. | K-23:L41; K-4:L39-L43
- **K-004/IX-011** [IX] (DEEP-K-004.discovery.md:L353): K23-E025, K11-E004 | Both add dashboard frontend_pair surfaces (WORKSPACE altitude vs cockpit EvidenceArtifactSet) with cross_lang_diff. | K-23:L78-L79; K-11:L52-L53
- **K-004/IX-012** [IX] (DEEP-K-004.discovery.md:L354): K23-E007, K19-E005, K11-E008 | Flag polarity mix: K-23 default-OFF enable; K-19 mandates default-OFF for reuse-into-gen; K-11 default-ON kill-switch — all W-2/R9 disciplined. | K-23:L37; K-19:L18; K-11:L33
- **K-004/IX-013** [IX] (DEEP-K-004.discovery.md:L355): K4-E008, K5-E003/E004 | K-4 RIDER-b coverage tags every manifest gate; K-5 routes always-on vs expensive legs within that inventory. | K-4:L27-L29; K-5:L7-L11
- **K-004/IX-014** [IX] (DEEP-K-004.discovery.md:L356): K7-E007, K11-E012 | Buyer packaging chain: K-7 sells K-6; K-11 renders G-2 RRA human summary that K-6 also redacts into compliance bundles. | K-7:L14-L15; K-11:L19-L19
- **K-004/IX-015** [IX] (DEEP-K-004.discovery.md:L357): K25-I001, K25-E010 | K-9 model-refresh is natural snapshot-refresh trigger for K-25 without shared files. | K-25:L46-L47, L69-L71
- **K-004/IX-016** [IX] (DEEP-K-004.discovery.md:L358): K4-E041, K18a-E017, K23-E020 | Shared R10 INTEGRATION_SMOKE discipline: real entrypoint, hard inputs, independent auditor — gate-tax, case-reuse, workspace merge. | K-4:L221-L227; K-18a:L133-L137; K-23:L41
- **K-004/IX-017** [IX] (DEEP-K-004.discovery.md:L359): K19-E003/E004, K18a-E015 | SF-1/SF-2 ban self-graded fitness; K-18a false_match_rate + match-everything arm is the observe-arm metric discipline that prevents useless-green hit-rate. | K-19:L16-L17; K-18a:L140-L143
- **K-004/IX-018** [IX] (DEEP-K-004.discovery.md:L360): K23-E003/E026, K4-E037 | GD-2 journal-as-record: workspace.ssb.db + GateTaxMeasured recompute both require journal-recomputable numbers/projections. | K-23:L5, L98; K-4:L291
- **K-004/IX-019** [IX] (DEEP-K-004.discovery.md:L361): K11-E019, K23-E021 | Human merge authority / worker git read-only: K-11 poll-and-surface only; K-23 member commits governed by each member's commit-cadence. | K-11:L102; K-23:L41
- **K-004/IX-020** [IX] (DEEP-K-004.discovery.md:L362): K5-E018, K25-E013 | Fail-safe vs fail-open: missing risk signal → more gates (K-5); missing snapshot schema → exit 2 never silent CLEAN (K-25). | K-5:L144-L145; K-25:L73-L74

### K-004 summary deltas (namespaced K-004/SD-*)

- **K-004/SD-K4-001** [SD] (DEEP-K-004.discovery.md:L372): **CORRECT** summary K-4.05/K-4.16 ON/OFF SAFEDELTA ablation and gates-OFF denominator: refined source VETOES product-level ON/OFF and uses per-gate isolation + freq-weighted model. | 11-K:L134-L136, L145 | K4-E012, K4-E017, K4-C001
- **K-004/SD-K4-002** [SD] (DEEP-K-004.discovery.md:L373): **CORRECT** summary K-4.11 wire-dark-allowlist 105-row gate inventory: source FIX-1 bans that registry as scope source (it is default-OFF exemptions, 135 rows). | 11-K:L142 | K4-E016, K4-C001
- **K-004/SD-K4-003** [SD] (DEEP-K-004.discovery.md:L374): **CORRECT** summary single "composed whole-system" number: source reports TWO labeled numbers never summed (coding_task vs audit) + over_block_rate. | 11-K:L133-L135 | K4-E004-E007
- **K-004/SD-K4-004** [SD] (DEEP-K-004.discovery.md:L375): **CORRECT** summary K-4.01 `args_bench.rs:75`: refined K-4 does not pin gate-tax to line 75; it specifies 4-file K-18a-pattern wiring. | 11-K:L132 | K4-E038, K4-C001
- **K-004/SD-K4-005** [SD] (DEEP-K-004.discovery.md:L376): **CORRECT** summary "release gate" as implied Rust artifact: source ATTACK-4 makes breach the CLI non-zero EXIT CODE only. | 11-K:L137, L146 | K4-E039, K4-E034
- **K-004/SD-K4-006** [SD] (DEEP-K-004.discovery.md:L377): **ADD** RIDER-b coverage table, RIDER-c trading future arm, CEILING-PENDING fail-closed, FORK-1 per-family routes, GateTaxMeasured flat payload, three R10 smoke cases, full ABORT/ADV set. | 11-K:L128-L148 | K4-E008-E009, K4-E020-E045
- **K-004/SD-K4-007** [SD] (DEEP-K-004.discovery.md:L378): **ADD** FIX-4 benign-corpus over-block (not GateOverBlocked), ts_ms point-in-time, dossier-row KD-1 wording, PS-2 proof-spine ownership. | 11-K:L128-L148 | K4-E002, K4-E007, K4-E011, K4-E019, K4-E022
- **K-004/SD-K5-001** [SD] (DEEP-K-004.discovery.md:L384): **ADD** policy_table.toml path, gate_intensity.rs module, observe-only threshold calibration, fail-safe degrade on missing signals, frozen-table out-of-band workflow. | 11-K:L150-L178 | K5-E009-E010, K5-E017-E019
- **K-004/SD-K5-002** [SD] (DEEP-K-004.discovery.md:L385): **ADD** exact Kind set TierAssigned/ExpensiveLegFired/CollapsedToCheap and re-grep 236/545 note. | 11-K:L173 | K5-E011
- **K-004/SD-K5-003** [SD] (DEEP-K-004.discovery.md:L386): **ADD** C-30 complexity classes and B-29 centrality as named signal sources beyond the summary risk list. | 11-K:L162-L166 | K5-E005
- **K-004/SD-K7-001** [SD] (DEEP-K-004.discovery.md:L392): **ADD** exact brief path ELAI-APPSEC-INSURANCE-BRIEF.md, GTM agencies-first / Article-50 ban, externalization anonymity rule, number-freshness inheritance from K-1 dossier. | 11-K:L207-L221 | K7-E002, K7-E009-E010, K7-E016-E017
- **K-004/SD-K7-002** [SD] (DEEP-K-004.discovery.md:L393): **ADD** deterministic checks (lint pass + Article-50 rg count 0) and W-1 = lint pass not file-exists. | 11-K:L207-L221 | K7-E008, K7-E012
- **K-004/SD-K7-003** [SD] (DEEP-K-004.discovery.md:L394): **QUALIFY** summary G-8/G-9/I-44 control reuse: numbered body centers K-6 export + F-9/F-2 + K-1. | 11-K:L216 | K7-I001
- **K-004/SD-K11-001** [SD] (DEEP-K-004.discovery.md:L400): **ADD** EvidenceArtifactRendered Kind set, LIVE ~267/577 W-9 note, exact kill-switch name, W-10 fixture vs live split. | 11-K:L273-L297 | K11-E008, K11-E011, K11-E014
- **K-004/SD-K11-002** [SD] (DEEP-K-004.discovery.md:L401): **ADD** doctrine-tension poll-CI-only wedge, lavish occlusion deferral, screenshot ref-not-blob size policy. | 11-K:L295-L297 | K11-E019-E021
- **K-004/SD-K11-003** [SD] (DEEP-K-004.discovery.md:L402): **ADD** H-52 PARTIAL status and contract test matrix including no-git-mutation. | 11-K:L283-L286 | K11-I001, K11-E015
- **K-004/SD-K18a-001** [SD] (DEEP-K-004.discovery.md:L408): **ADD** entire unit (summary A-05 unindexed). Full inventory of observe-arm scope, fixtures, CLI, forks, aborts, verifies, red-team, deferred K-1/VCG legs. | 11-K:L23, L896 | K18a-E001-E022, K18a-C001
- **K-004/SD-K18a-002** [SD] (DEEP-K-004.discovery.md:L409): **ADD** relationship to K-18: executes SAFE Task-1+6 only; not an amendment of K-18; feeds PS-5. | 11-K:L464-L493 | K18a-E001-E003
- **K-004/SD-K18a-003** [SD] (DEEP-K-004.discovery.md:L410): **ADD** metric pair recurrence_hit_rate + false_match_rate @k={1,3,5}, retrieval_mode honesty, match-everything negative. | 11-K:L464-L493 | K18a-E010, K18a-E012, K18a-E015
- **K-004/SD-K19-001** [SD] (DEEP-K-004.discovery.md:L416): **ADD** full SF-1..SF-4 clause text with GAP-1/GAP-2 live sites, exact lint claim phrases, negative fixture path, OUT-OF-K VCG ladder. | 11-K:L495-L524 | K19-E003-E009, K19-E019
- **K-004/SD-K19-002** [SD] (DEEP-K-004.discovery.md:L417): **ADD** K-18 recogniser composition rule and [DESIGN]/[BENCH-PENDING on K-18] required tag shape. | 11-K:L518 | K19-E008, K19-E020
- **K-004/SD-K19-003** [SD] (DEEP-K-004.discovery.md:L418): **ADD** paper citation list and JD-1 harness evidence pattern (no SSB). | 11-K:L524 | K19-E015, K19-E018
- **K-004/SD-K23-001** [SD] (DEEP-K-004.discovery.md:L424): **ADD** exact Edge field schema, relationship hints produces-for/consumes-from/peer, WorkspaceEventKind full enum, parallel_cap=3, journal-content-minimization. | 11-K:L610-L664 | K23-E006, K23-E009, K23-E019, K23-E026
- **K-004/SD-K23-002** [SD] (DEEP-K-004.discovery.md:L425): **ADD** open risks: J-18 Heuristic-only until Sound, member-count cap, two-FSM controller hazard. | 11-K:L610-L664 | K23-E032-E034
- **K-004/SD-K23-003** [SD] (DEEP-K-004.discovery.md:L426): **ADD** module path list, LIVE 268/577 bump, promotion_plan_id=K-23-promotion, contract path. | 11-K:L646-L650 | K23-E007, K23-E028-E029, K23-E031
- **K-004/SD-K25-001** [SD] (DEEP-K-004.discovery.md:L432): **ADD** AT_RISK_STALE verdict class, exact --help unsoundness declaration, elai_status_query pattern mirror, watch-set creep ban. | 11-K:L719-L741 | K25-E007, K25-E009, K25-E015, K25-E021
- **K-004/SD-K25-002** [SD] (DEEP-K-004.discovery.md:L433): **ADD** W-1 = live rule-fire + exit-code steering (not file exists), fixture trio, spine-exempt PROOF_SPINE none. | 11-K:L719-L741 | K25-E002, K25-E018, K25-E020
- **K-004/SD-K25-003** [SD] (DEEP-K-004.discovery.md:L434): **ADD** K-9 as natural refresh trigger and G-10 symmetry note (share nothing). | 11-K:L719-L741 | K25-E015, K25-I001

### K-004 ambiguity and conflict register (namespaced K-004/AC-*)

- **K-004/AC-001** [AC] (DEEP-K-004.discovery.md:L440): C | Summary K-4 ON/OFF + wire-dark inventory + single whole-system number vs refined two-number + VETO + RIDER-b design | K4-C001
- **K-004/AC-002** [AC] (DEEP-K-004.discovery.md:L441): C | Summary K-18a absent (A-05) vs full on-disk plan inventory in this chunk | K18a-C001
- **K-004/AC-003** [AC] (DEEP-K-004.discovery.md:L442): A | K-4 ceiling numeric values and audit unbounded option | K4-A001
- **K-004/AC-004** [AC] (DEEP-K-004.discovery.md:L443): A | K-4/K-5/K-11/K-18a SSB +1 vs reuse FORK undecided until edit | K4-A002, K5-A002, K11-A001, K18a-A001
- **K-004/AC-005** [AC] (DEEP-K-004.discovery.md:L444): A | K-5 trigger thresholds observe-only → enforce schedule | K5-A001
- **K-004/AC-006** [AC] (DEEP-K-004.discovery.md:L445): A | K-23 U4 before/after J-18; exact WORKSPACE altitude chrome | K23-A001-A002
- **K-004/AC-007** [AC] (DEEP-K-004.discovery.md:L446): A | K-7 insurance section template detail | K7-A001
- **K-004/AC-008** [AC] (DEEP-K-004.discovery.md:L447): A | K-25 first-fill watch-set concrete keys | K25-A001

## Summary-layer record (11-K-SUMMARY_INDEX)

Source: `docs/thefinalglance/feature-discovery/11-K-SUMMARY_INDEX.discovery.md`
Extracted summary atoms: **699**.

Summary is orientation. Where a deep ledger C/SD/AC record corrects it, the deep record wins target state and the summary wording remains here as the losing description.

### Summary — preamble (4)

- **SUMMARY/E** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L11):  — explicit in `K-SUMMARY_INDEX.md`; wording may be atomized but no capability is added.
- **SUMMARY/I** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L12):  — inference needed to describe a composition or consequence; never treated as committed behavior.
- **SUMMARY/A** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L13):  — ambiguity, stale index metadata, or a missing/deferred source requiring deeper inspection.
- **SUMMARY/Status vocabulary** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L14):  preserves the summary's own distinctions: WIRED/SHIPPED, BUILD/PLANNED, DESIGN, PARTIAL, FIXTURE-now, BENCH-PENDING, RunPod/network-gated, default-OFF/observe-first, OUT-OF-K, KILLED, and user-gated.

### Summary — K-X (20)

- **SUMMARY/K-X.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L31): [E, purpose] K is primarily a sequencing, composition, proof, distribution, affordability, packaging, and discipline layer over the verification/evidence spine, not a conventional capability letter.
- **SUMMARY/K-X.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L32 | SURFACE): [E, exceptions] K-23 multi-repo/workspace memory and K-24 billing/credits are the two explicitly acknowledged net-new capability exceptions.
- **SUMMARY/K-X.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L33): [E, product thesis] The product is the measured number: external positioning waits on falsification, measurement, portable evidence, and PR distribution rather than shipping unsupported claims.
- **SUMMARY/K-X.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L34 | SURFACE): [E, first gate] K first invokes the already-shipped A-1 falsifier and may trigger a kill-switch before further build work.
- **SUMMARY/K-X.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L35): [E, sequencing] A-1..A-4 plus K-3 gate the rest of the corpus under the K-10 no-new-letters rule.
- **SUMMARY/K-X.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L36): [E, proof] A-2/A-3 with K-1/K-2 produce the missing headline measurements.
- **SUMMARY/K-X.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L37 | SURFACE): [E, distribution] A-4 supplies the portable artifact and K-3/K-11 put its machine and human render where review happens: the PR/check-run and cockpit.
- **SUMMARY/K-X.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L38): [E, affordability] K-4 measures whole-system gate friction; K-5 must choose a deterministic risk tier that fits beneath K-4's locked ceiling.
- **SUMMARY/K-X.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L39): [E, buyer packaging] K-6 turns existing evidence into an offline compliance bundle and K-7 turns governed measurements and controls into buyer-facing AppSec/insurance prose.
- **SUMMARY/K-X.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L40): [E, depreciation] K-8 performs the initial absorbability audit; K-25 makes it a staleness-gated standing watchdog; K-9 separately refreshes local/model-lane choices quarterly.
- **SUMMARY/K-X.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L41 | SURFACE): [E, differentiators] KD-1 is number-before-claim; KD-2 is meet-the-buyer-in-the-PR; KD-3 is a composed-friction ceiling; KD-4 is provider-absorption pre-gating; KD-5 is a time-boxed wedge/window clock.
- **SUMMARY/K-X.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L42): [E, truth labels] Every external figure KW-1..KW-6 must be tagged `[EXTERNAL]`; every shipped-number claim must be ELAI-measured.
- **SUMMARY/K-X.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L43): [E, composition] Entries must name their reused substrate and explicit do-not-reimplement boundaries under GD-1.
- **SUMMARY/K-X.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L44): [E, record] Measured runtime outputs are expected to be journal-recomputable under GD-2.
- **SUMMARY/K-X.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L45): [E, done gates] Runtime legs use a W-1 real run-trigger observable-effect gate; positioning/doctrine legs use E-23/JD-1 artifact-and-loading gates.
- **SUMMARY/K-X.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L46): [E, negative proof] Each entry defines an R2 negative or broken arm so success is causally attributable and fail-closed behavior is tested.
- **SUMMARY/K-X.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L47 | SURFACE): [E, UI discipline] Runtime-facing additions either name a frontend pair or defer the choice to edit time under W-4; CLI/doc-only work may justify N/A under W-6.
- **SUMMARY/K-X.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L48): [E, SSB discipline] Most K units add no SSB family or prefer reuse with at most one event; K-23 and K-24 inherit the explicit sequential +1 event bump.
- **SUMMARY/K-X.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L49 | SURFACE): [E, promotion discipline] The two net-new tracks inherit run-trigger PASS, R10 integration smoke, journal-as-record, default-OFF enablement, and an allowlist promotion row.
- **SUMMARY/K-X.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L50): [E, scope boundary] The eight Section-A items A-1..A-8 are pre-existing plan files that K sequences; they are not K sessions and are not summarized here.

### Summary — K-1 (20)

- **SUMMARY/K-1.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L58): [E, document] Create a problem-quantification dossier that binds external problem figures to ELAI-measured results as those measurements land.
- **SUMMARY/K-1.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L59): [E, external datum] Preserve KW-1 USENIX hallucinated-package figures: 5.2%/21.7% and 58%/43% recurrence, explicitly external.
- **SUMMARY/K-1.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L60): [E, external datum] Preserve KW-2's 11x-vulnerabilities figure and citation `arXiv:2507.02976`, explicitly external.
- **SUMMARY/K-1.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L61): [E, external datum] Preserve KW-3 reward-hacking figures 73.8%/46.8%, explicitly external.
- **SUMMARY/K-1.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L62): [E, external datum] Preserve KW-4 CodeAct figures approximately 50%/-60%, explicitly external.
- **SUMMARY/K-1.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L63 | SURFACE): [E, external datum] Preserve KW-5 SpecBench +28 percentage points, explicitly external.
- **SUMMARY/K-1.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L64): [E, measured join] Bind those figures to the A-1 kill-gate verdict, A-2 SeCodePLT result, and A-3 SAFEDELTA delta when available.
- **SUMMARY/K-1.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L65): [E, policy] Create a claims-policy document.
- **SUMMARY/K-1.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L66 | SURFACE): [E, invariant] An external-facing claim may ship only with a journal-recomputable ELAI number or an explicit `[EXTERNAL]`/`[DESIGN]` tag.
- **SUMMARY/K-1.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L67): [E, tooling] Run a claims-policy lint over README and positioning copy.
- **SUMMARY/K-1.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L68): [E, wording] Treat the I-43 trace-authentication-versus-deterministic-replay wording table per lane as normative.
- **SUMMARY/K-1.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L69): [E, dependency] Invoke I-43's A-1 verdict; do not rebuild it.
- **SUMMARY/K-1.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L70): [E, exclusion] KW-6 DORA/METR/vendor material remains PENDING-VERIFY and excluded until dossier-build reverification.
- **SUMMARY/K-1.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L71): [E, integration] Place the dossier under `docs/strategy/` or `workflow/reasoning/`.
- **SUMMARY/K-1.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L72): [E, integration] Place the lint under `.claude/tools/` or an `elai`-side positioning/README check.
- **SUMMARY/K-1.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L73): [E, state] Add no SSB event family.
- **SUMMARY/K-1.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L74 | SURFACE): [E, done gate] Every dossier row must contain either a measured ELAI number plus GD-2 recompute pointer, or an explicit design/external tag plus KW source.
- **SUMMARY/K-1.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L75): [E, negative] A seeded unmeasured superlative such as “the safest agent” must fail lint.
- **SUMMARY/K-1.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L76): [E, negative] A bare external number without an explicit tag must fail lint.
- **SUMMARY/K-1/K-6/K-7/K-13/K-17/K-20:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L991):  `docs/strategy/`, `workflow/reasoning/`, `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, `docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`, `rra_cmd.rs`.

### Summary — K-2 (24)

- **SUMMARY/K-2.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L82 | SURFACE): [E, CLI] Add `elai bench routing-economics` through a `routing-economics` bench subcommand at the `args_bench.rs:75` seam.
- **SUMMARY/K-2.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L83): [E, corpus] Run against a fixed task corpus.
- **SUMMARY/K-2.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L84): [E, ablation] Compare cloud-only with the H-7 architect/editor hybrid.
- **SUMMARY/K-2.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L85): [E, metric] Report the percentage of steps offloaded.
- **SUMMARY/K-2.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L86): [E, metric] Report dollar delta.
- **SUMMARY/K-2.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L87): [E, metric] Report quality delta as verified-OUTCOME rate, never pass@k.
- **SUMMARY/K-2.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L88 | SURFACE): [E, determinism] Run both arms under `ELAI_SEED`.
- **SUMMARY/K-2.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L89): [E, provenance] Make results GD-2 recomputable from journal evidence.
- **SUMMARY/K-2.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L90): [E, reuse] Mirror the SAFEDELTA/QDELTA runner shape rather than inventing a new harness.
- **SUMMARY/K-2.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L91): [E, reuse] Consume H-7 routing and I-25 router substrate.
- **SUMMARY/K-2.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L92): [E, dependency] Block the live dollar leg on J-28 normalized per-lane cost capture.
- **SUMMARY/K-2.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L93 | SURFACE): [E, display] Feed H-9 cockpit display.
- **SUMMARY/K-2.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L94): [E, reuse] Reuse I-27's verified-outcome-versus-compute harness; do not fork it.
- **SUMMARY/K-2.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L95): [E, constraint] Leave E-39 cap supremacy unchanged.
- **SUMMARY/K-2.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L96): [E, runner] Add a runner mirroring `benchmarks/safedelta_runner.rs`.
- **SUMMARY/K-2.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L97): [E, state] Prefer an existing bench/`RouteDecisionEvent` row; otherwise add at most `RoutingEconomicsMeasured`.
- **SUMMARY/K-2.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L98 | SURFACE): [E, UI] Decide at edit time whether a hybrid-versus-cloud table belongs on the runtime/cost panel.
- **SUMMARY/K-2.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L99): [E, artifact] Emit one artifact per arm with dollars per verified task.
- **SUMMARY/K-2.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L100): [E, delivery] Ship the deterministic FIXTURE tier immediately.
- **SUMMARY/K-2.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L101): [E, fail-closed] Refuse to emit a live dollar number until J-28 exists; never fabricate cost.
- **SUMMARY/K-2.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L102 | SURFACE): [E, negative] A `--force-cloud-everything` arm must collapse the savings delta, demonstrating the hybrid caused the result.
- **SUMMARY/K-2.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L103 | SURFACE): [E, market status] The “first production number” is a planned positioning claim; the cited HERA ~45% offload is research-grade/external, not an ELAI result.
- **SUMMARY/K-2/K-4/K-15/K-18:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L992):  `args_bench.rs:75`, `benchmarks/safedelta_runner.rs`, `benchmarks/reasoning_budget_runner.rs`, `failure_taxonomy.py`, plus existing convergence/context-fill runners.
- **SUMMARY/K-2 routing anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L993):  `cloud_mode.rs`, `escalation_policy.rs`.

### Summary — K-3 (19)

- **SUMMARY/K-3.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L109 | SURFACE): [E, distribution] Render ELAI evidence into GitHub check-runs and PR comments.
- **SUMMARY/K-3.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L110): [E, render] Include F-1/F-2/F-9 gate verdicts per hunk.
- **SUMMARY/K-3.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L111): [E, render] Include F-4 stdout-hash-bound test outcomes.
- **SUMMARY/K-3.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L112): [E, render] Include G-5 DSR and vouched/unvouched bands.
- **SUMMARY/K-3.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L113): [E, render] Place H-52 anchored findings using its resolver so fabricated line numbers are structurally impossible.
- **SUMMARY/K-3.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L114): [E, render] Include the G-8 attestation digest.
- **SUMMARY/K-3.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L115 | SURFACE): [E, render] Include an offline-verification instruction using `elai verify-attestation`.
- **SUMMARY/K-3.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L116): [E, render] Include the J-16 blast-radius cone summary when available.
- **SUMMARY/K-3.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L117 | SURFACE): [E, fail-closed UI] Render a hunk with no evidence as `UNVOUCHED`; never omit it or mark it clean.
- **SUMMARY/K-3.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L118 | SURFACE): [E, reuse] Implement as a renderer over the existing C-40/E-41 `github_pr` adapter and `elai pr-review`; reimplement none of the posting machinery.
- **SUMMARY/K-3.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L119 | SURFACE): [E, reuse] Reuse G-5 review-surface, H-52 anchors, F-4 evidence, G-8 attestation, and J-16 cone data.
- **SUMMARY/K-3.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L120): [E, data boundary] Read the RRA evidence manifest as a closed set.
- **SUMMARY/K-3.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L121): [E, structural security] Make out-of-evidence comments structurally impossible by rendering only from the closed manifest.
- **SUMMARY/K-3.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L122): [E, integration] Add the renderer beneath `crates/orchestrator/src/github_pr/` on the body-composer/comment path.
- **SUMMARY/K-3.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L123 | SURFACE): [E, state] Prefer existing `github_pr`, review-surface, or attestation rows; add at most one event.
- **SUMMARY/K-3.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L124 | SURFACE): [E, UI] Pair with the existing GitHub state surface (`github.svelte.ts`), decision finalized at edit time.
- **SUMMARY/K-3.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L125 | SURFACE): [E, done gate] On a real fixture-repo PR, every check-run claim must join to a journal row.
- **SUMMARY/K-3.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L126 | SURFACE): [E, negative] A `--post-unvouched-as-clean` broken arm must fail.
- **SUMMARY/K-3/K-11 GitHub anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L994 | SURFACE):  `pr_review_cmd.rs`, `crates/orchestrator/src/github_pr/{review.rs,body_composer.rs,comment_watcher.rs,adapter_trait.rs,evidence_render.rs}`, `github.svelte.ts`, `.claude` `visual-ui-loop`.

### Summary — K-4 (18)

- **SUMMARY/K-4.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L132 | SURFACE): [E, CLI] Add `elai bench gate-tax` through a `gate-tax` bench subcommand at `args_bench.rs:75`.
- **SUMMARY/K-4.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L133): [E, cadence] Produce a per-release friction report.
- **SUMMARY/K-4.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L134): [E, metric] Measure end-to-end added wall-clock per task class.
- **SUMMARY/K-4.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L135): [E, metric] Measure composed whole-system over-block rate over a fixed corpus.
- **SUMMARY/K-4.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L136): [E, ablation] Compare gates ON versus gates OFF using SAFEDELTA-style framing.
- **SUMMARY/K-4.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L137): [E, interpretation] Treat friction as QDELTA's denominator twin.
- **SUMMARY/K-4.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L138): [E, configuration] Lock a ceiling to a real user-selected value established from the first ON/OFF measurement; never invent it.
- **SUMMARY/K-4.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L139): [E, release gate] Block a release when the composed ceiling is breached.
- **SUMMARY/K-4.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L140): [E, observability] Name the top three friction contributors.
- **SUMMARY/K-4.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L141): [E, reuse] Live beside and share I-38 QDELTA's runner shape and corpus.
- **SUMMARY/K-4.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L142 | SURFACE): [E, reuse] Reuse G-6 ablation framing, the 105-row `wire-dark-allowlist.toml` gate inventory, and E-49 metrics.
- **SUMMARY/K-4.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L143): [E, state] Add at most `GateTaxMeasured`, or extend I-38 rows.
- **SUMMARY/K-4.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L144 | SURFACE): [E, UI] Consider a friction tile on the cost/audit surface at edit time.
- **SUMMARY/K-4.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L145): [E, done gate] Make the composed number journal-recomputable.
- **SUMMARY/K-4.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L146): [E, negative] A deliberately spammy fixture gate must trip the ceiling and fail the release gate.
- **SUMMARY/K-4.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L147): [E, denominator] A gates-OFF run establishes the ON/OFF denominator.
- **SUMMARY/K-4.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L148): [E, fail-loud] A zero ceiling with any default-ON gate must fail loudly, forcing a real configured value.
- **SUMMARY/K-4/K-5/K-19 promotion anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L995 | SURFACE):  `wire-dark-allowlist.toml`, the frozen gate manifest, `ELAI_LOSSLESS_GATE_DISABLE`, `ELAI_PR_EVIDENCE_RENDER_DISABLE`.

### Summary — K-5 (25)

- **SUMMARY/K-5.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L154): [E, policy] Define a deterministic R8-frozen `GateIntensity` policy table.
- **SUMMARY/K-5.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L155 | SURFACE): [E, always-on] Keep the E-30 argv allowlist always on.
- **SUMMARY/K-5.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L156): [E, always-on] Keep F-1/F-9 secret/dependency checks always on.
- **SUMMARY/K-5.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L157): [E, always-on] Keep G-1 taint checks always on.
- **SUMMARY/K-5.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L158): [E, expensive gates] Trigger I-20 mutation testing only through risk policy.
- **SUMMARY/K-5.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L159): [E, expensive gates] Trigger H-26 differential testing only through risk policy.
- **SUMMARY/K-5.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L160): [E, expensive gates] Trigger VLM tier 2 only through risk policy.
- **SUMMARY/K-5.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L161): [E, expensive gates] Trigger E-53 formal verification only through risk policy.
- **SUMMARY/K-5.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L162): [E, risk input] Use J-16 blast-radius cone size.
- **SUMMARY/K-5.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L163 | SURFACE): [E, risk input] Use F-2 CWE-surface class.
- **SUMMARY/K-5.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L164): [E, risk input] Use B-22 autonomy level.
- **SUMMARY/K-5.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L165): [E, risk input] Use D-15 novelty.
- **SUMMARY/K-5.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L166): [E, risk input] Use file criticality.
- **SUMMARY/K-5.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L167): [E, authority] Never let the LLM select a tier; an LLM-proposed tier is ignored.
- **SUMMARY/K-5.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L168): [E, economy] Collapse safely for low-risk mechanical edits by using fewer turns and only cheap gates.
- **SUMMARY/K-5.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L169): [E, safety] Use the full expensive stack at high-risk decision points.
- **SUMMARY/K-5.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L170): [E, integration] Implement a compositional policy-table-plus-selector dispatcher.
- **SUMMARY/K-5.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L171): [E, integrity] Put the policy-table hash into the frozen gate manifest.
- **SUMMARY/K-5.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L172): [E, self-protection] Prevent a run from weakening or editing its own gating policy.
- **SUMMARY/K-5.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L173): [E, state] Add at most `GateIntensitySelected`, otherwise reuse F-1 rows.
- **SUMMARY/K-5.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L174 | SURFACE): [E, UI] Consider tier-decision rows in the audit timeline.
- **SUMMARY/K-5.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L175): [E, expected behavior] A trivial rename emits only always-on gate rows and zero expensive-leg rows.
- **SUMMARY/K-5.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L176): [E, expected behavior] A high-cone authentication-touching edit fires the full stack.
- **SUMMARY/K-5.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L177): [E, proof] Demonstrate measured composed tax below K-4's ceiling on at least three real runs.
- **SUMMARY/K-5.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L178): [E, negative] Block an attempted in-run edit to the frozen policy table.

### Summary — K-6 (22)

- **SUMMARY/K-6.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L184 | SURFACE): [E, CLI] Add `elai export compliance`.
- **SUMMARY/K-6.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L185): [E, bundle] Emit one bundle, not separate uncoordinated artifacts.
- **SUMMARY/K-6.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L186): [E, bundle] Include a G-9-redacted G-2 RRA.
- **SUMMARY/K-6.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L187): [E, bundle] Include the G-8 attestation.
- **SUMMARY/K-6.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L188): [E, bundle] Include an audit-trail extract.
- **SUMMARY/K-6.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L189): [E, bundle] Include a retention statement that names the I-44 mode.
- **SUMMARY/K-6.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L190): [E, mapping] Include a mapping document that links bundle fields to procurement clauses.
- **SUMMARY/K-6.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L191): [E, mapping] Map requirements through code, build, and release traceability.
- **SUMMARY/K-6.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L192): [E, mapping] Map audit-log requirements.
- **SUMMARY/K-6.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L193): [E, mapping] Map OMB M-25-22-style independent-evaluation and reproducibility rights.
- **SUMMARY/K-6.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L194): [E, wording] Describe EU AI Act Article 50 only as “designed to support”; never claim Article-50 certification.
- **SUMMARY/K-6.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L195): [E, mapping] Address PLD exposure.
- **SUMMARY/K-6.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L196): [E, mapping] Provide SOC2-adjacent control narratives.
- **SUMMARY/K-6.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L197): [E, reuse] Compose G-2/G-8/G-9/I-44/I-18; add no new attestation machinery.
- **SUMMARY/K-6.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L198 | SURFACE): [E, integration] Add a CLI arm composing existing RRA, attestation, and redaction paths plus a strategy mapping doc.
- **SUMMARY/K-6.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L199): [E, state] Add at most `ComplianceExported`, otherwise reuse G-8 rows.
- **SUMMARY/K-6.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L200 | SURFACE): [E, UI] CLI-only N/A is justified; a compliance tile remains an edit-time option.
- **SUMMARY/K-6.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L201 | SURFACE): [E, portability] Verify the exported bundle offline on a clean machine with `elai verify-attestation` and a conformance checker.
- **SUMMARY/K-6.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L202): [E, privacy test] A raw-secret grep over the export must be empty.
- **SUMMARY/K-6.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L203): [E, traceability] Every procurement clause in the mapping document must cite a bundle field.
- **SUMMARY/K-6.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L204): [E, negative] A bundle without its attestation digest fails offline verification.
- **SUMMARY/K-6.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L205): [E, negative] An unredacted RRA containing a seeded secret fails the secret-grep gate.

### Summary — K-7 (11)

- **SUMMARY/K-7.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L211): [E, document] Create a pure-prose AppSec/insurance positioning brief under `docs/strategy/`.
- **SUMMARY/K-7.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L212): [E, buyer] Position the evidence pack within AppSec/SCA budgets.
- **SUMMARY/K-7.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L213): [E, evidence] Link the F-9 hallucinated-dependency gate to governed KW-1 USENIX figures.
- **SUMMARY/K-7.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L214): [E, evidence] Link F-2 CWE gates to the governed KW-2 11x-vulnerability figure.
- **SUMMARY/K-7.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L215): [E, insurance] Answer insurance questions about controls, logs, rollback, and independent evidence.
- **SUMMARY/K-7.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L216): [E, dependency] Source every number from K-1's dossier.
- **SUMMARY/K-7.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L217): [E, reuse] Cite G-8/G-9/I-44 controls; add no runtime behavior.
- **SUMMARY/K-7.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L218): [E, state] Add no SSB event.
- **SUMMARY/K-7.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L219): [E, governance] Tag every number as measured or external.
- **SUMMARY/K-7.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L220): [E, human gate] Require user review before external use.
- **SUMMARY/K-7.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L221): [E, negative] Fail K-1 lint for any number lacking a measured-or-external tag.

### Summary — K-8 (19)

- **SUMMARY/K-8.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L227): [E, audit] Sweep remaining PLANNED/PARTIAL efficiency units in H-Track-1 H-1..H-9 residue.
- **SUMMARY/K-8.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L228): [E, audit] Sweep Track-14 I-10..I-16 residue.
- **SUMMARY/K-8.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L229): [E, audit] Sweep E-2/E-25/E-32 residual legs.
- **SUMMARY/K-8.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L230): [E, audit] Sweep J-24 tool-context residual legs.
- **SUMMARY/K-8.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L231): [E, classification] Tag provider-native units `ABSORBED` and kill or de-scope them to their instrument half.
- **SUMMARY/K-8.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L232): [E, classification] Tag units requiring a build-time provider recheck `AT-RISK`.
- **SUMMARY/K-8.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L233): [E, classification] Tag verification-coupled units worth keeping `DURABLE`.
- **SUMMARY/K-8.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L234): [E, artifact] Produce a kill/de-scope decision list.
- **SUMMARY/K-8.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L235): [E, authority] Require explicit user confirmation before any removal; never auto-apply the list.
- **SUMMARY/K-8.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L236): [E, doctrine] Add a KD-4 planning pre-gate rule.
- **SUMMARY/K-8.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L237 | SURFACE): [E, fresh research] Before executing any efficiency-class unit, verify the provider's current native surface via WebSearch.
- **SUMMARY/K-8.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L238): [E, integration] Upsert absorbability tags into `ELAI-FEATURE-MAP.md`.
- **SUMMARY/K-8.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L239): [E, integration] Add a conditional-routing `.claude/rules/` file for efficiency-class tasks.
- **SUMMARY/K-8.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L240): [E, integration] Store the kill list as a `workflow/reasoning/` user-decision artifact.
- **SUMMARY/K-8.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L241): [E, state] Add no SSB event.
- **SUMMARY/K-8.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L242): [E, done gate] Make feature-map tags grep-provable.
- **SUMMARY/K-8.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L243): [E, done gate] Demonstrate the rule loading in a live efficiency-class task.
- **SUMMARY/K-8.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L244 | SURFACE): [E, negative] Flag an efficiency-class task that skips the provider recheck.
- **SUMMARY/K-8.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L245): [E, violation] Treat an automatically applied removal as a house-rule violation.

### Summary — K-9 (21)

- **SUMMARY/K-9.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L251): [E, cadence] Add a recurring quarterly evaluated checkpoint.
- **SUMMARY/K-9.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L252): [E, selection] Compare top J-9 catalog candidates per lane with the incumbent.
- **SUMMARY/K-9.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L253): [E, metric] Measure fixture pass rate.
- **SUMMARY/K-9.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L254): [E, metric] Measure VRAM use.
- **SUMMARY/K-9.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L255): [E, metric] Measure tokens per second.
- **SUMMARY/K-9.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L256): [E, metric] Measure dollars per verified task.
- **SUMMARY/K-9.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L257): [E, lanes] Evaluate through J-9 catalog, E-27 AWQ, and J-19 vLLM lanes.
- **SUMMARY/K-9.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L258): [E, decision] Emit an `ADOPT`, `KEEP`, or `RE-EVALUATE` decision row.
- **SUMMARY/K-9.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L259): [E, human gate] Require user ratification of that row.
- **SUMMARY/K-9.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L260): [E, doctrine] Keep the local lane scoped to mechanical/privacy/latency work.
- **SUMMARY/K-9.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L261): [E, doctrine] Treat 24GB-class hardware as the autonomy-candidate tier, pending the 2x3090 box.
- **SUMMARY/K-9.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L262): [E, doctrine] Keep cloud as the planning/architect lane.
- **SUMMARY/K-9.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L263): [E, prior decisions] Treat Gemma-4 and Qwen3-32B rejections as tier-conditional catalog data, not permanent global bans.
- **SUMMARY/K-9.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L264): [E, routing integrity] Do not change routing without user sign-off.
- **SUMMARY/K-9.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L265 | SURFACE): [E, implementation] Add a cadence checklist and bench wrapper over J-9.
- **SUMMARY/K-9.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L266): [E, state] Prefer J-9 rows; otherwise add at most `ModelRefreshCadence`.
- **SUMMARY/K-9.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L267 | SURFACE): [E, UI] Consider a refresh-decision tile at edit time.
- **SUMMARY/K-9.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L268): [E, artifact] Emit a dated refresh report on the first run.
- **SUMMARY/K-9.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L269): [E, safe fallback] Automatically keep the incumbent when a candidate regresses fixtures.
- **SUMMARY/K-9.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L270): [E, negative] Reject a claimed win without a fixture run.
- **SUMMARY/K-9.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L271): [E, negative] Block a silent model swap without ratification.

### Summary — K-11 (21)

- **SUMMARY/K-11.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L277): [E, render] Extend K-3's per-hunk verdict renderer with human-viewable evidence.
- **SUMMARY/K-11.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L278 | SURFACE): [E, PR UI] Add a `## Evidence` section to the PR comment/check-run.
- **SUMMARY/K-11.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L279 | SURFACE): [E, cockpit UI] Show the same evidence in the reviewer cockpit.
- **SUMMARY/K-11.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L280): [E, evidence] Render the G-2 RRA digest.
- **SUMMARY/K-11.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L281): [E, evidence] Render F-4 stdout-hash test outcomes and their links.
- **SUMMARY/K-11.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L282): [E, evidence] Render G-5 DSR vouched/unvouched bands.
- **SUMMARY/K-11.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L283): [E, evidence] Render H-52 anchored findings.
- **SUMMARY/K-11.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L284 | SURFACE): [E, evidence] Render references to screenshots already captured by `visual-ui-loop`.
- **SUMMARY/K-11.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L285): [E, prioritization] Risk-tier the view using G-5 DSR so reviewers inspect by tier.
- **SUMMARY/K-11.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L286 | SURFACE): [E, fail-closed UI] Render a no-artifact hunk as `UNVOUCHED`, never clean or omitted.
- **SUMMARY/K-11.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L287): [E, reuse] Use K-3/E-41 `github_pr`, `ReviewCommentKind`, G-5, G-2, F-4, H-52, and existing screenshots; add no evidence machinery.
- **SUMMARY/K-11.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L288): [E, implementation] Add `crates/orchestrator/src/github_pr/evidence_render.rs`.
- **SUMMARY/K-11.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L289 | SURFACE): [E, CLI] Add `elai pr-review --emit-evidence`.
- **SUMMARY/K-11.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L290): [E, default] Make evidence emission default-ON.
- **SUMMARY/K-11.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L291 | SURFACE): [E, kill switch] Allow `ELAI_PR_EVIDENCE_RENDER_DISABLE` to disable the render.
- **SUMMARY/K-11.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L292): [E, state] Prefer existing SSB rows; add at most `EvidenceArtifactRendered`.
- **SUMMARY/K-11.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L293 | SURFACE): [E, operations] Gate live check-run posting on network availability/permission under W-10.
- **SUMMARY/K-11.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L294 | SURFACE): [E, delivery boundary] Resolve the PR-auto-fix doctrine tension by shipping poll-and-surface only in the wedge slice.
- **SUMMARY/K-11.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L295): [E, human authority] Keep merge human-controlled.
- **SUMMARY/K-11.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L296): [E, git safety] Do not autonomously force-push.
- **SUMMARY/K-11.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L297 | SURFACE): [E, grounding limit] Treat the kunchen “engineering manager, not diff reviewer” statement as an AUTHOR/market-validation claim, not an ELAI measurement.

### Summary — K-12 (24)

- **SUMMARY/K-12.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L303): [E, compaction gate] Change `check_lossless_compaction` from WARN to fail-closed when `unrecoverable_count > 0`.
- **SUMMARY/K-12.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L304): [E, archival invariant] Call a compacted record lossless only if archival ran first.
- **SUMMARY/K-12.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L305): [E, missing evidence] Treat missing R1 archival/wiring evidence as FAIL.
- **SUMMARY/K-12.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L306): [E, schema] Add `critical_atoms { kind, value, must_preserve }` to I-11 `BeliefState`.
- **SUMMARY/K-12.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L307): [E, handoff gate] After any model/lane handoff, assert every must-preserve atom's exact string/symbol is present in the SSB record.
- **SUMMARY/K-12.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L308): [E, pristine rebuild] Run the same assertion after I-5 pristine rebuild.
- **SUMMARY/K-12.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L309): [E, trimming] Run the same assertion after trimming/compaction.
- **SUMMARY/K-12.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L310): [E, fail-closed] Fail the handoff/trim when any must-preserve atom is missing.
- **SUMMARY/K-12.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L311): [E, determinism] Implement Critical-Atom-Recall as an exact deterministic check.
- **SUMMARY/K-12.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L312): [E, prohibition] Never use semantic similarity to satisfy the gate.
- **SUMMARY/K-12.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L313): [E, prohibition] Never use a generative model or ciphered comparison to adjudicate the gate.
- **SUMMARY/K-12.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L314): [E, reuse] Reuse H-49 RV-2, I-11, SessionDAG recover stubs, I-5, and G-2 verification; do not reimplement them.
- **SUMMARY/K-12.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L315): [E, status correction] Correct stale feature-map labels: I-5/I-11/H-50 are BUILT.
- **SUMMARY/K-12.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L316): [E, integration] Wire fail-close in `compact.rs`.
- **SUMMARY/K-12.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L317): [E, integration] Add the CriticalAtom schema in `belief_state.rs` and a `handoff_faithfulness.rs` gate.
- **SUMMARY/K-12.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L318): [E, integration] Wire the gate at `escalation_waterfall.rs:239` and the live compaction path.
- **SUMMARY/K-12.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L319 | SURFACE): [E, default/rollout] Make the gate default-ON behind `ELAI_LOSSLESS_GATE_DISABLE`, while using observe-then-enforce rollout under GD-4.
- **SUMMARY/K-12.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L320): [E, state] Prefer `BeliefStateEvent` plus a Kind; add at most `HandoffFaithfulnessGated`.
- **SUMMARY/K-12.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L321 | SURFACE): [E, UI] Add no separate frontend; verdicts flow to existing context/trust panels.
- **SUMMARY/K-12.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L322): [E, test] A compaction with any unrecoverable item must return `Err`, not warn.
- **SUMMARY/K-12.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L323): [E, integration smoke] A HARD multi-turn local-to-cloud handoff that drops an atom must fail R10.
- **SUMMARY/K-12.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L324): [E, negative] A paraphrase does not satisfy exact preservation.
- **SUMMARY/K-12.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L325): [E, authority] The LLM never sets `must_preserve` and never adjudicates the result.
- **SUMMARY/K-12 context anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L996):  `compact.rs:545`, `belief_state.rs`, `escalation_waterfall.rs:239`, `session_dag.rs:1539`, `pristine_path_filter.rs`, `handoff_faithfulness.rs`.

### Summary — K-13 (23)

- **SUMMARY/K-13.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L331): [E, document] Create `docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`.
- **SUMMARY/K-13.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L332): [E, coverage] Include one row for each OWASP LLM01-LLM10 (2025).
- **SUMMARY/K-13.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L333): [E, coverage] Include one row for each OWASP ASI01-ASI10 (2025).
- **SUMMARY/K-13.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L334 | SURFACE): [E, cell schema] Each cell names ELAI control(s).
- **SUMMARY/K-13.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L335): [E, cell schema] Each cell includes a code/plan citation.
- **SUMMARY/K-13.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L336): [E, cell schema] Each cell carries exactly an honest status such as `WIRED-MEASURED`, `WIRED-DESIGN`, `PARTIAL`, `PLANNED`, or `N-A`.
- **SUMMARY/K-13.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L337): [E, claims] Each cell has an ELAI-measured number or an explicit `[DESIGN]`/`[EXTERNAL]` tag.
- **SUMMARY/K-13.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L338): [E, boundary] Treat model inversion/extraction as N/A only for the current single-user-local boundary.
- **SUMMARY/K-13.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L339): [E, lifecycle] Reopen that N/A boundary for multi-seat under J-59.
- **SUMMARY/K-13.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L340): [E, proof] Bind prompt-injection controls LLM01/LLM05/ASI01 to the G-6 SAFEDELTA AgentDojo gate-ON-versus-OFF delta.
- **SUMMARY/K-13.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L341): [E, honest status] Mark that injection row `[BENCH-PENDING]` until A-2/A-3/AgentDojo runs land.
- **SUMMARY/K-13.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L342): [E, reuse] Extend K-1 dossier/lint; do not fork them.
- **SUMMARY/K-13.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L343): [E, first-use fact] This is the repository's first OWASP-string mapping according to the summary's authoring grep.
- **SUMMARY/K-13.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L344): [E, buyer integration] Cross-link the mapping into K-6 as a procurement clause without amending/rebuilding K-6.
- **SUMMARY/K-13.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L345): [E, buyer integration] Let K-7 answer the buyer's “OWASP coverage?” question from this mapping.
- **SUMMARY/K-13.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L346): [E, reused controls] Map existing controls G-1/F-9/F-2/F-1/F-3/G-9/I-1/I-50/E-30/Tier-C/F-4/H-53/G-5/E-24/G-4/C-42/E-43/B-22 rather than restating them.
- **SUMMARY/K-13.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L347 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-13.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L348): [E, done gate] The K-1 lint must pass only when every mapping cell is tagged.
- **SUMMARY/K-13.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L349): [E, negative] A “fully covered” cell without a number/tag fails.
- **SUMMARY/K-13.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L350): [E, negative] An “OWASP certified” superlative fails.
- **SUMMARY/K-13.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L351): [E, prohibition] Fabricating the pending injection number is banned.
- **SUMMARY/K-13.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L352): [E, OUT-OF-K] WI-1 transformation-surviving taint, WI-2 cross-process taint, and WI-3 E-37 perimeter completion belong to an E-37/G-1 technical extension, not K.
- **SUMMARY/K-13.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L353): [E, OUT-OF-K] WI-6 config-ingress admission belongs to an E-43/E-24 technical extension, not K.

### Summary — K-14 (25)

- **SUMMARY/K-14.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L359): [E, doctrine] Add `.claude/rules/time-determinism.md`.
- **SUMMARY/K-14.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L360 | SURFACE): [E, TZ-1] Put volatile time strictly after the last cache breakpoint, never in system/rules prompts.
- **SUMMARY/K-14.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L361): [E, TZ-1 rationale] Preserve cache/replay behavior by obeying the R-1 mitigation.
- **SUMMARY/K-14.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L362): [E, TZ-2] Source time through `OsClock`.
- **SUMMARY/K-14.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L363): [E, TZ-2] Journal the exact time bytes supplied as model input.
- **SUMMARY/K-14.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L364): [E, TZ-2 replay] Replay those bytes rather than recomputing time.
- **SUMMARY/K-14.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L365): [E, TZ-2 determinism] Keep `SeededClock` gate/journal byte identity.
- **SUMMARY/K-14.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L366): [E, TZ-3] Never use wall time as a gate input.
- **SUMMARY/K-14.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L367): [E, TZ-3] Never use wall time as a transition or promotion input.
- **SUMMARY/K-14.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L368): [E, TZ-4] Keep wall-clock kill on the supervisor liveness plane outside the replay set.
- **SUMMARY/K-14.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L369): [E, TZ-4] Record the wall-clock kill but do not evaluate it as replay evidence.
- **SUMMARY/K-14.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L370): [E, TZ-4] Use step/no-progress as the primary liveness criterion.
- **SUMMARY/K-14.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L371): [E, TZ-4] Permit wall-clock kill only in AUTONOMOUS mode.
- **SUMMARY/K-14.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L372): [E, routing] Load the doctrine through conditional routing for time, timestamp, watchdog, and async-tool work.
- **SUMMARY/K-14.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L373): [E, differentiator] Add IMP-9 to the K-1 dossier: decoupled async tools whose lifecycle lands in a deterministic, replayable, taint-tagged journal.
- **SUMMARY/K-14.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L374): [E, honest status] Tag IMP-9 `[DESIGN]`, with G3-G6 wired but the model-facing loop J-future.
- **SUMMARY/K-14.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L375): [E, measurement] Attach no measured number to IMP-9 until one exists.
- **SUMMARY/K-14.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L376): [E, reuse] Extend K-1/K-7/K-8 doctrine and existing clock/RRA/cache/async primitives; do not fork.
- **SUMMARY/K-14.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L377 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-14.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L378): [E, done gate] A live relevant task must load the rule; K-1 lint must pass the tagged claim.
- **SUMMARY/K-14.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L379): [E, negative] An untagged “fully deterministic async tools” claim fails.
- **SUMMARY/K-14.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L380): [E, audit] Catch a time block in a system prompt or wall-clock in a gate at doctrine audit.
- **SUMMARY/K-14.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L381): [E, OUT-OF-K] T-1 timestamp slot, T-2 agent-time, T-3 H-51 resume header, T-4 async-tool loop, and T-5 watchdog promotion belong to a future J-family time track bound by TZ-1..TZ-4.
- **SUMMARY/K-14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L817 | SURFACE):  constrains future time/async work so volatile context remains replay-compatible while supervisor liveness stays outside replay authority.
- **SUMMARY/K-14 time anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L997 | SURFACE):  `.claude/rules/time-determinism.md`, `clock.rs`, cache breakpoints, background-task/notification primitives.

### Summary — K-15 (27)

- **SUMMARY/K-15.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L387 | SURFACE): [E, CLI] Add `elai bench reasoning-budget` through the bench-subcommand seam.
- **SUMMARY/K-15.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L388): [E, curve] Measure verified-outcome rate versus reasoning compute per `(model, task-type)`.
- **SUMMARY/K-15.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L389): [E, calibration] Derive a measured peak and knee from the curve.
- **SUMMARY/K-15.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L390): [E, sibling] Use the J-34 empirical context-calibration shape, with reasoning budget rather than context fill on the x-axis.
- **SUMMARY/K-15.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L391): [E, ablation] Compare GRC-v3 router ON with flat-budget OFF.
- **SUMMARY/K-15.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L392 | SURFACE): [E, determinism] Run both arms under `ELAI_SEED`.
- **SUMMARY/K-15.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L393): [E, outcome] Put receipt-backed verified outcomes on the y-axis; never use pass@k or self-grading.
- **SUMMARY/K-15.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L394): [E, provenance] Make the curve GD-2 recomputable.
- **SUMMARY/K-15.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L395): [E, consumer] Supply the measured knee to J-84/J-85's overthinking ceiling.
- **SUMMARY/K-15.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L396): [E, granularity] Store the knee per model and task type.
- **SUMMARY/K-15.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L397 | SURFACE): [E, prohibition] Never import the paper's 6K/7K knee as an ELAI threshold.
- **SUMMARY/K-15.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L398): [E, delivery] Ship the deterministic FIXTURE tier immediately.
- **SUMMARY/K-15.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L399): [E, deferred] Keep the ON arm BENCH-PENDING until J-83/J-85 exist.
- **SUMMARY/K-15.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L400): [E, operations] Keep the live local-lane run RunPod-gated.
- **SUMMARY/K-15.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L401): [E, differentiator] Add “receipt-grounded reasoning” to the K-1 dossier and K-7 brief.
- **SUMMARY/K-15.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L402): [E, meaning] Define the differentiator as confidence from an external receipt at every step, never from the chain self-grading.
- **SUMMARY/K-15.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L403): [E, truth label] Mark the differentiator `[DESIGN]`/`[BENCH-PENDING]`.
- **SUMMARY/K-15.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L404): [E, reuse] Reuse I-27 convergence, J-34 calibration storage, G-6 SAFEDELTA, H-9, and K-1/K-7; do not fork their harnesses.
- **SUMMARY/K-15.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L405): [E, implementation] Add `benchmarks/reasoning_budget_runner.rs`.
- **SUMMARY/K-15.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L406): [E, state] Prefer `ConvergenceCurveMeasured` with a `ReasoningBudgetArm` kind; add at most one new family.
- **SUMMARY/K-15.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L407 | SURFACE): [E, UI] Reuse I-27 `ConvergencePanel` and H-9 `CloudPanel`, or explicitly justify N/A at edit time.
- **SUMMARY/K-15.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L408): [E, proof] The FIXTURE arm must run and emit a recomputable curve.
- **SUMMARY/K-15.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L409): [E, pending proof] The ON arm must eventually show verified-outcome lift at/near the knee versus OFF; this remains BENCH-PENDING.
- **SUMMARY/K-15.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L410): [E, negative] Reject an untagged “reasoning self-verifies” claim.
- **SUMMARY/K-15.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L411): [E, negative] Reject a bare unmeasured “GRC-v3 +20%” claim.
- **SUMMARY/K-15.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L412): [E, negative] Reject a paper-derived 7K knee represented as ELAI's measured knee.
- **SUMMARY/K-15/K-16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L818):  pair a measured reasoning-budget knee with a doctrine that admits receipts, independent votes only as fallback, and never self-evaluation.

### Summary — K-16 (22)

- **SUMMARY/K-16.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L418): [E, doctrine] Add `.claude/rules/reasoning-receipts.md`.
- **SUMMARY/K-16.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L419): [E, RR-1] Treat a reasoning trace as monitoring/UX only.
- **SUMMARY/K-16.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L420): [E, RR-1] Never admit a reasoning trace as the confidence or selection signal.
- **SUMMARY/K-16.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L421): [E, RR-1] Require an external receipt for admissible confidence.
- **SUMMARY/K-16.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L422): [E, RR-2] Prefer a deterministic receipt over all other selector evidence.
- **SUMMARY/K-16.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L423): [E, RR-2 fallback] If no receipt exists, allow a statistical vote only over independent samples.
- **SUMMARY/K-16.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L424): [E, RR-2 boundary] Never use the vote fallback for factual recall; route factual recall to retrieval.
- **SUMMARY/K-16.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L425): [E, RR-2 prohibition] Ban LLM self-evaluation as a selector.
- **SUMMARY/K-16.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L426): [E, RR-3] Measure a reasoning-budget/overthinking knee per model and task type with K-15.
- **SUMMARY/K-16.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L427): [E, RR-3 prohibition] Never import a paper constant as the knee.
- **SUMMARY/K-16.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L428 | SURFACE): [E, RR-3 prohibition] Never use context-window size as the reasoning-control lever.
- **SUMMARY/K-16.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L429): [E, RR-4] Commit and freeze receipt-confirmed state.
- **SUMMARY/K-16.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L430): [E, routing] Load the rule for reasoning-engine, GRC, reasoning-budget, and candidate-selection work.
- **SUMMARY/K-16.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L431): [E, enforcement scope] Keep this rule auditor-applied; add no new runtime gate.
- **SUMMARY/K-16.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L432): [E, runtime basis] Rely on shipped F-4/D-13/I-4/H-39 runtime archetypes rather than restating them.
- **SUMMARY/K-16.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L433): [E, claims] Extend K-1 lint for receipt-grounded wording and K-15's claim.
- **SUMMARY/K-16.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L434 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-16.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L435): [E, done gate] A task building J-83..J-86 must load the rule through conditional routing.
- **SUMMARY/K-16.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L436): [E, audit] Catch an LLM-self-evaluating candidate selector in reviewed design.
- **SUMMARY/K-16.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L437): [E, negative] Fail an untagged “the model checks its own reasoning” claim.
- **SUMMARY/K-16.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L438): [E, negative] Fail a paper knee represented as ELAI's own threshold.
- **SUMMARY/K-16/K-19/K-21 doctrine anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L998):  `.claude/rules/reasoning-receipts.md`, `.claude/rules/self-improvement-fitness.md`, `.claude/rules/journal-completeness.md`, and conditional-routing triggers.

### Summary — K-17 (21)

- **SUMMARY/K-17.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L444 | SURFACE): [E, differentiator] Add a K-1 dossier row stating that ELAI measures a context-degradation knee per `(model, lane)` against gate verdicts.
- **SUMMARY/K-17.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L445): [E, cloud policy] Describe the cloud regime as cache-stable-prefix plus late compaction.
- **SUMMARY/K-17.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L446 | SURFACE): [E, local policy] Describe the local regime as window-minimization plus unload-early.
- **SUMMARY/K-17.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L447 | SURFACE): [E, replacement] Replace folklore 0.75-local/0.85-cloud window fractions with the measured J-34 knee.
- **SUMMARY/K-17.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L448): [E, replacement] Replace fixed slot-order claims with measured calibration.
- **SUMMARY/K-17.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L449): [E, truth label] Mark the row `[DESIGN]` for the wired 10-slot allocator/regime split.
- **SUMMARY/K-17.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L450): [E, truth label] Mark measured lift `[BENCH-PENDING]` on J-34.
- **SUMMARY/K-17.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L451 | SURFACE): [E, no duplication] Add no parallel context bench; J-34 `elai bench context-fill` is the number source.
- **SUMMARY/K-17.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L452): [E, status] Treat `context_budget.rs` 10-slot allocation and `context_policy.rs` regime split as WIRED/code-grounded.
- **SUMMARY/K-17.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L453): [E, depreciation framing] Treat generic caching/compaction as provider-absorbed; claim the durable verification-coupled measurement, not the bare feature.
- **SUMMARY/K-17.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L454): [E, doctrine reuse] Cite CL-1..CL-4; do not re-author them.
- **SUMMARY/K-17.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L455): [E, claims] Extend K-1 lint and K-7 brief.
- **SUMMARY/K-17.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L456): [E, state] Add no SSB family; point to J-34 `ContextCalibrationEvent`.
- **SUMMARY/K-17.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L457 | SURFACE): [E, UI] Reuse `ContextStatePanel`/H-9 or justify N/A.
- **SUMMARY/K-17.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L458): [E, done gate] The dossier row must exist with both honest tags and its code basis.
- **SUMMARY/K-17.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L459 | SURFACE): [E, negative] Fail an untagged “ELAI never loses context” claim.
- **SUMMARY/K-17.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L460 | SURFACE): [E, negative] Fail an unmeasured “0% degradation at full window” claim.
- **SUMMARY/K-17.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L461): [E, negative] Fail an external RULER/Chroma knee represented as ELAI's knee.
- **SUMMARY/K-17.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L462): [E, OUT-OF-K] IMP-1 build J-34; IMP-2 wire `adaptive_reduce`; IMP-3 wire I-11 escalation condense; IMP-4 wire I-5 pristine injection plus J-78 access spec; IMP-5 wire J-16 plus J-38 retrieval; IMP-6 wire J-54 pool plus H-7 routing; IMP-7 wire I-1 supervisor plane; and IMP-8 wire J-19 vLLM belong to a J context-activation track bound by CL-1..CL-4.
- **SUMMARY/K-17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L819):  packages J-34's per-model/lane context knee and opposite cloud/local regimes; it adds no duplicate bench.
- **SUMMARY/K-17 context anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L999 | SURFACE):  `context_budget.rs`, `context_policy.rs`, `ContextCalibrationEvent`, `ContextStatePanel`.

### Summary — K-18 (27)

- **SUMMARY/K-18.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L468 | SURFACE): [E, CLI] Add `elai bench case-reuse` through the shared bench seam.
- **SUMMARY/K-18.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L469): [E, granularity] Measure per `(bug-class, model)`.
- **SUMMARY/K-18.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L470): [E, metric] Measure iteration delta.
- **SUMMARY/K-18.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L471): [E, metric] Measure dollar delta.
- **SUMMARY/K-18.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L472): [E, metric] Measure recurrence-hit rate.
- **SUMMARY/K-18.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L473): [E, ablation] Compare case-reuse ON against cold/OFF.
- **SUMMARY/K-18.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L474): [E, ON behavior] Inject an F-4-PROVEN case node as a demonstration in the ON arm.
- **SUMMARY/K-18.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L475 | SURFACE): [E, determinism] Run under `ELAI_SEED`.
- **SUMMARY/K-18.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L476): [E, provenance] Make results GD-2 recomputable.
- **SUMMARY/K-18.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L477): [E, outcome] Use external receipts/verified outcomes rather than pass@k or self-grading.
- **SUMMARY/K-18.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L478): [E, safe slice] Ship recurrence-hit observation first with zero reuse into generation.
- **SUMMARY/K-18.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L479): [E, pending] Keep reuse-ON economics BENCH-PENDING on the VCG J-track.
- **SUMMARY/K-18.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L480): [E, discipline dependency] Require K-19 execution-grounded fitness before activating reuse.
- **SUMMARY/K-18.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L481): [E, differentiator] Add “gets faster/cheaper on YOUR recurring bugs over time” to K-1/K-7.
- **SUMMARY/K-18.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L482): [E, truth label] Mark that differentiator `[DESIGN]`/`[BENCH-PENDING]`.
- **SUMMARY/K-18.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L483): [E, market status] Treat “no production number exists” and moat #2 ranking as research/positioning context, not a measured result.
- **SUMMARY/K-18.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L484): [E, reuse] Reuse I-27/G-6/H-9/F-4/G-2/failure taxonomy/B-29 and case-memory ingredients; do not fork their runners.
- **SUMMARY/K-18.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L485): [E, implementation] Add a runner mirroring convergence/SAFEDELTA.
- **SUMMARY/K-18.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L486 | SURFACE): [E, state] Prefer an existing bench row/`ConvergenceCurveMeasured` with `CaseReuseArm`/`RecurrenceHit`; add at most one family.
- **SUMMARY/K-18.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L487 | SURFACE): [E, UI] Reuse I-27/H-9 panels or justify N/A.
- **SUMMARY/K-18.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L488): [E, done gate] The recurrence-hit and fixture arms run now and emit recomputable numbers.
- **SUMMARY/K-18.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L489): [E, pending proof] The reuse-ON arm must eventually reduce iteration/dollar cost versus cold.
- **SUMMARY/K-18.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L490): [E, negative] Fail untagged “learns from your code” or “improves with use” claims.
- **SUMMARY/K-18.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L491): [E, negative] Fail a bare unmeasured “50% fewer iterations” claim.
- **SUMMARY/K-18.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L492): [E, negative] Fail byte-hash, LLM-helpfulness, or retrieval-count reuse-fitness designs.
- **SUMMARY/K-18.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L493 | SURFACE): [E, OUT-OF-K] The VCG J-track mints a typed case only on F-4 proof, self-wires deterministic edges, adds a MEMORY/CASE canvas altitude, gates retrieval-time reuse, and defines episodic-to-skill-to-rule promotion; it is technical companion scope, not K-18 implementation.
- **SUMMARY/K-18/K-19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L820):  pair case-reuse economics with execution-grounded, stronger-than-generator, abstention-aware fitness and an auditor firewall.

### Summary — K-19 (25)

- **SUMMARY/K-19.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L499): [E, doctrine] Add `.claude/rules/self-improvement-fitness.md`.
- **SUMMARY/K-19.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L500): [E, applicability] Bind self-improvement, case-graph, archive-promotion, playbook-curation, memory-reuse, and DGM work.
- **SUMMARY/K-19.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L501): [E, SF-1] Admit execution-grounded fitness only.
- **SUMMARY/K-19.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L502): [E, SF-1 prohibition] Never use a byte hash as fitness; replace live GAP-1 `derived_alpha=blake3(config_bytes)`.
- **SUMMARY/K-19.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L503): [E, SF-1 prohibition] Never use LLM-judged helpfulness.
- **SUMMARY/K-19.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L504): [E, SF-1 prohibition] Never treat “it was retrieved” as fitness.
- **SUMMARY/K-19.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L505): [E, SF-1 positive] Require a gate-PROVEN pass on a held-out recurrence.
- **SUMMARY/K-19.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L506): [E, SF-2] Require a stronger-than-generator filter under D-13.
- **SUMMARY/K-19.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L507): [E, SF-2] Make fitness abstention-aware under RD-1.
- **SUMMARY/K-19.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L508): [E, SF-2] Detect traps behaviorally with measured recall, replacing live GAP-2 keyword grep.
- **SUMMARY/K-19.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L509): [E, SF-3] Roll out observe-first under GD-4.
- **SUMMARY/K-19.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L510 | SURFACE): [E, SF-3] Keep reuse default-OFF and behind the wire-dark allowlist.
- **SUMMARY/K-19.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L511): [E, SF-3 firewall] Never inject reused cases into the auditor.
- **SUMMARY/K-19.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L512): [E, SF-4] Re-derive a case against the current commit and AST.
- **SUMMARY/K-19.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L513): [E, SF-4] Hard-filter superseded cases; do not merely narrate them as outdated.
- **SUMMARY/K-19.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L514): [E, SF-4] Never delete historical cases under reversibility doctrine.
- **SUMMARY/K-19.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L515): [E, SF-4] Quarantine poisoning under E-5/J-59.
- **SUMMARY/K-19.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L516): [E, routing] Load the rule through conditional routing for relevant work.
- **SUMMARY/K-19.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L517): [E, enforcement scope] Keep this auditor-applied; add no new runtime gate.
- **SUMMARY/K-19.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L518): [E, claims] Extend K-1 lint for self-improving/learns-from-code/improves-with-use language.
- **SUMMARY/K-19.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L519 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-19.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L520): [E, done gate] A VCG or K-18 reuse task must load the rule.
- **SUMMARY/K-19.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L521): [E, audit] Catch a byte-hash/LLM-helpful fitness design.
- **SUMMARY/K-19.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L522): [E, audit] Catch reuse into the auditor.
- **SUMMARY/K-19.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L523 | SURFACE): [E, research rationale] Preserve the summary's central-risk claim that self-graded improvement collapses, with external Misevolve/F-10 figures treated as grounding, not ELAI measurements.

### Summary — K-20 (21)

- **SUMMARY/K-20.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L529): [E, differentiator] Add “re-executable, not just replayable, evidence” as the wedge in K-1/K-7.
- **SUMMARY/K-20.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L530): [E, third-party property] A third party reruns against the recorded G-1 taint-ingested external bytes.
- **SUMMARY/K-20.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L531): [E, cassette] Store those bytes in a content-keyed VCR cassette.
- **SUMMARY/K-20.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L532): [E, environment] Pin execution with an OCI digest, lockfile, and CAS fixtures.
- **SUMMARY/K-20.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L533): [E, verdict] Require a matching I-45 objective-sealed verdict.
- **SUMMARY/K-20.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L534): [E, distinction] Describe the property as external faithfulness absent from replay-and-trust or screenshot evidence.
- **SUMMARY/K-20.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L535): [E, truth label] Mark the dossier row `[DESIGN]`/`[BENCH-PENDING]`.
- **SUMMARY/K-20.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L536): [E, honest tier] Distinguish action-faithful cloud replay from token-faithful replay.
- **SUMMARY/K-20.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L537): [E, token boundary] Claim token faithfulness only on lanes where J-13 capability checks pass.
- **SUMMARY/K-20.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L538): [E, drift] Treat world drift as a fidelity downgrade; a cassette proves the old world, not the live world.
- **SUMMARY/K-20.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L539): [E, no duplication] Add no new bench; J-14's extension owns re-execution proof.
- **SUMMARY/K-20.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L540): [E, reuse] Compose G-2, J-14, G-1, I-45, I-18, G-9, A-4, K-3/K-11, and other K differentiator rows.
- **SUMMARY/K-20.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L541): [E, orthogonality] Keep re-executability distinct from K-21 completeness.
- **SUMMARY/K-20.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L542): [E, integration] Add the tagged row to `docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, extend K-1 lint, and add a K-7 line.
- **SUMMARY/K-20.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L543 | SURFACE): [E, state/UI] Add no SSB family; point to future `ReproPackEvent`; ride K-3/K-11 or N/A.
- **SUMMARY/K-20.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L544): [E, negative] Fail an untagged “fully reproducible” claim.
- **SUMMARY/K-20.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L545): [E, negative] Fail “bit-for-bit re-execute the live world” because of world drift.
- **SUMMARY/K-20.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L546): [E, negative] Fail a token-faithful claim on an uncapped lane.
- **SUMMARY/K-20.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L547): [E, OUT-OF-K] Technical re-execution is a J-14 extension and is not authored in K.
- **SUMMARY/K-20/K-21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L821):  keep three properties separate: journal integrity, journal completeness, and re-executable external faithfulness.
- **SUMMARY/K-20 RRA anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1000):  `rra.rs`, future `ReproPackEvent`, J-14 cassette/`rra export-fixture` paths.

### Summary — K-21 (27)

- **SUMMARY/K-21.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L553): [E, doctrine] Add `.claude/rules/journal-completeness.md`.
- **SUMMARY/K-21.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L554 | SURFACE): [E, applicability] Bind completeness, side-effect mediation, kernel observation, reconciliation, and any trust-the-journal/RRA/canvas/derived-status work.
- **SUMMARY/K-21.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L555): [E, JC-1] Distinguish integrity from completeness.
- **SUMMARY/K-21.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L556): [E, JC-1] A hash chain proves a row was not altered; it does not prove every side effect wrote a row.
- **SUMMARY/K-21.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L557): [E, JC-1] An append-only log cannot determine its own omissions.
- **SUMMARY/K-21.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L558): [E, JC-2] Declare the high-value side-effect set that must be mediated or independently observed.
- **SUMMARY/K-21.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L559): [E, JC-2 mediation] Permit `SECCOMP_RET_USER_NOTIF` atomic commit extending G-3/Tier-C.
- **SUMMARY/K-21.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L560): [E, JC-2 observation] Alternatively use an independent eBPF/LSM observer.
- **SUMMARY/K-21.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L561): [E, JC-2 normalization] Require a normalization layer for observer events.
- **SUMMARY/K-21.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L562): [E, JC-2 prohibition] Never assume an effect was journaled.
- **SUMMARY/K-21.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L563): [E, JC-3] Treat reconciliation missingness as FAIL.
- **SUMMARY/K-21.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L564): [E, JC-3 monitor] Monitor the monitor with dropped-event counters.
- **SUMMARY/K-21.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L565): [E, JC-3 fail-close] Fail closed on dropped observer events.
- **SUMMARY/K-21.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L566): [E, JC-3 state] Treat observer impairment as first-class state/evidence.
- **SUMMARY/K-21.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L567): [E, JC-3 determinism] Make reconciliation deterministic; the LLM does not adjudicate.
- **SUMMARY/K-21.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L568): [E, JC-3 rollout] Use observe-first rollout.
- **SUMMARY/K-21.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L569): [E, JC-4 scope] State the honestly covered scope.
- **SUMMARY/K-21.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L570 | SURFACE): [E, JC-4 platform] Describe the observer as Linux-only and Windows as mediation-only.
- **SUMMARY/K-21.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L571): [E, JC-4 trust ceiling] State that host trust requires a Rekor/TPM-IMA rider on I-18/G-8.
- **SUMMARY/K-21.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L572): [E, routing] Load the doctrine conditionally on relevant tasks.
- **SUMMARY/K-21.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L573): [E, enforcement scope] Keep this rule auditor-applied; add no runtime gate or SSB family.
- **SUMMARY/K-21.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L574): [E, claims] Extend K-1 lint for scoped completeness language.
- **SUMMARY/K-21.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L575): [E, done gate] A live observer/completeness task must load the rule.
- **SUMMARY/K-21.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L576): [E, negative] Fail “complete/tamper-proof journal” when untagged or when it conflates integrity with completeness.
- **SUMMARY/K-21.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L577): [E, audit] Catch SKIP-on-missing-observer reconciliation.
- **SUMMARY/K-21.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L578 | SURFACE): [E, audit] Catch a Windows claim that advertises the Linux observer.
- **SUMMARY/K-21.27** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L579): [E, OUT-OF-K] The E1 technical track owns mediation extension, eBPF/LSM observation, reconciliation, and monitor-the-monitor runtime machinery.

### Summary — K-22 (26)

- **SUMMARY/K-22.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L585 | SURFACE): [E, tool] Add deterministic `.claude/tools/doctrine_coverage.py`.
- **SUMMARY/K-22.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L586): [E, input] Extract every doctrine clause from `CLAUDE.md`.
- **SUMMARY/K-22.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L587): [E, input] Extract every doctrine clause from `.claude/rules/*.md`.
- **SUMMARY/K-22.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L588): [E, join] Join clauses to enforcers through `.claude/gate-manifest.txt` and the `.claude/tools/` checker inventory.
- **SUMMARY/K-22.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L589): [E, classification] Tag genuinely frozen enforcement `FROZEN-CHECKER`.
- **SUMMARY/K-22.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L590): [E, classification] Tag human/auditor-only enforcement `AUDITOR-ADVISORY`.
- **SUMMARY/K-22.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L591): [E, classification] Tag missing/unmappable enforcement `UNENFORCED`.
- **SUMMARY/K-22.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L592): [E, metric] Emit the percentage of clauses enforced by frozen checkers.
- **SUMMARY/K-22.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L593): [E, provenance] Make the coverage percentage recomputable from the rulebook and manifest.
- **SUMMARY/K-22.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L594): [E, backlog] Emit a compilation backlog of unenforced clauses.
- **SUMMARY/K-22.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L595): [E, authority] Treat backlog items as user-confirmed proposals; never auto-build them.
- **SUMMARY/K-22.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L596): [E, doctrine] Add `.claude/rules/doctrine-enforcement.md`.
- **SUMMARY/K-22.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L597): [E, edit contract] Require every future rulebook edit to declare each clause's enforcement tag.
- **SUMMARY/K-22.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L598): [E, determinism] Never let the LLM grade enforcement.
- **SUMMARY/K-22.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L599): [E, fail-closed classification] An unmappable clause is `UNENFORCED`, never silently enforced.
- **SUMMARY/K-22.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L600): [E, threat] Measure the one-level-up residue where workers are gated but the rulebook controlling gates is itself ungated.
- **SUMMARY/K-22.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L601): [E, reuse] Consume `gate_freeze_check.py`, the gate manifest, checker inventory, and J-70 traceability; do not fork them.
- **SUMMARY/K-22.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L602): [E, consumer] Let K-4 consult this ledger to distinguish real frozen gates from advisory doctrine.
- **SUMMARY/K-22.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L603): [E, integration] Emit a `docs/strategy/` ledger and add a conditional-routing trigger plus K-1 lint extension.
- **SUMMARY/K-22.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L604 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-22.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L605): [E, done gate] Run the tool and emit the per-clause ledger, coverage percentage, and backlog.
- **SUMMARY/K-22.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L606): [E, done gate] Load doctrine-enforcement rule on a live rulebook edit.
- **SUMMARY/K-22.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L607): [E, negative] A clause claiming a checker absent from the manifest is `UNENFORCED`.
- **SUMMARY/K-22.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L608): [E, negative] Fail an untagged “fully enforced doctrine/every rule is a frozen gate” claim.
- **SUMMARY/K-22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L822):  measures how much of ELAI's own doctrine is actually compiled into frozen enforcement.
- **SUMMARY/K-22 doctrine-coverage anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1001 | SURFACE):  `CLAUDE.md`, `.claude/rules/*.md`, `.claude/tools/doctrine_coverage.py`, `.claude/rules/doctrine-enforcement.md`, `.claude/gate-manifest.txt`, `gate_freeze_check.py`.

### Summary — K-23 (53)

- **SUMMARY/K-23.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L614 | SURFACE): [E, capability] Add a thin cross-repository workspace layer above deep per-root ELAI substrate.
- **SUMMARY/K-23.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L615): [E, gap] Preserve existing per-root stores unchanged; current stores remain functions of one `project_root` with `BM25_PROJECT_ID="default"`.
- **SUMMARY/K-23.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L616 | SURFACE): [E, boundary] Do not mistake Cosmos cross-project artifacts or canvas multi-project views for cross-repo code intelligence.
- **SUMMARY/K-23.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L617 | SURFACE): [E, U1 object] Add a workspace object and manifest.
- **SUMMARY/K-23.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L618): [E, U1 storage] Store workspaces globally at `~/.elai/workspaces/<id>/`, beside Cosmos and never inside a member repo.
- **SUMMARY/K-23.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L619 | SURFACE): [E, U1 journal] Add a global `workspace.ssb.db` workspace journal.
- **SUMMARY/K-23.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L620 | SURFACE): [E, U1 CLI] Add `workspace create`.
- **SUMMARY/K-23.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L621 | SURFACE): [E, U1 CLI] Add `workspace add`.
- **SUMMARY/K-23.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L622 | SURFACE): [E, U1 CLI] Add `workspace map`.
- **SUMMARY/K-23.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L623): [E, U2 projection] Derive a `cross_repo_edges` projection.
- **SUMMARY/K-23.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L624): [E, U2 cache] Key the derived projection by a BLAKE3 hash of member DKG manifests and cascade invalidation on change.
- **SUMMARY/K-23.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L625): [E, U2 edge class] Classify LSP/SCIP-derived J-18 edges as `Sound`.
- **SUMMARY/K-23.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L626): [E, U2 edge class] Classify name/import-derived edges as `Heuristic`.
- **SUMMARY/K-23.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L627): [E, correctness] Permit Sound edges to drive gates.
- **SUMMARY/K-23.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L628): [E, correctness] Permit Heuristic edges to advise only; they cannot drive impact gates.
- **SUMMARY/K-23.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L629 | SURFACE): [E, U3 retrieval] Fan a workspace query to every member's existing retrieval pipeline.
- **SUMMARY/K-23.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L630): [E, U3 merge] Merge member results with reciprocal-rank fusion at `k=60`.
- **SUMMARY/K-23.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L631): [E, U3 provenance] Preserve per-repository provenance on every result.
- **SUMMARY/K-23.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L632): [E, U3 reuse] Build no centralized/new index.
- **SUMMARY/K-23.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L633): [E, U4 impact] Extend the B-29/J-16 blast-radius cone across repositories.
- **SUMMARY/K-23.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L634): [E, U4 precision] Traverse Sound edges only for cross-repo gating.
- **SUMMARY/K-23.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L635 | SURFACE): [E, U5 CLI] Add `workspace orchestrate`.
- **SUMMARY/K-23.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L636 | SURFACE): [E, U5 FSM] Add a workspace FSM above each per-root FSM.
- **SUMMARY/K-23.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L637): [E, U5 ordering] Kahn-order member work from Sound dependencies.
- **SUMMARY/K-23.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L638): [E, U5 delegation] Dispatch Tier-C subagents into J-80 worktrees.
- **SUMMARY/K-23.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L639): [E, U5 isolation] Each delegated member runs its own existing FSM.
- **SUMMARY/K-23.27** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L640): [E, U5 lifecycle] Add a `WORKSPACE_MERGING` phase/state.
- **SUMMARY/K-23.28** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L641): [E, U5 verification] Run a cross-repository contract integration smoke under R10.
- **SUMMARY/K-23.29** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L642 | SURFACE): [E, U6 resume] Resume from the workspace journal plus each member's H-51 digest.
- **SUMMARY/K-23.30** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L643): [E, U6 provenance] Attach `source_repo` provenance.
- **SUMMARY/K-23.31** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L644): [E, U6 trust] Extend J-21 poisoning/trust evaluation per repository.
- **SUMMARY/K-23.32** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L645 | SURFACE): [E, U6 UI] Add a canvas `WORKSPACE` altitude beside J-31/J-60/J-98.
- **SUMMARY/K-23.33** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L646): [E, mode] In `CONNECTED` mode, expose real Sound edges.
- **SUMMARY/K-23.34** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L647): [E, mode] In `DISCONNECTED` mode, provide federation only with an empty edge set.
- **SUMMARY/K-23.35** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L648): [E, honesty] Never fabricate a relationship between disconnected repositories.
- **SUMMARY/K-23.36** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L649 | SURFACE): [E, reuse] Reuse `paths.rs` unchanged and compose Cosmos, RRF, B-29/J-16, J-18, orchestrator, Tier-C, J-80, J-2/H-36, Kahn, context budget, H-51/J-21/C-48/J-59, SSB/FSM, and canvas substrate.
- **SUMMARY/K-23.37** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L650): [E, implementation] Add `crates/orchestrator/src/workspace/`.
- **SUMMARY/K-23.38** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L651 | SURFACE): [E, implementation] Add workspace verbs under the CLI.
- **SUMMARY/K-23.39** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L652): [E, implementation] Extend `crates/orchestrator/src/blast_radius.rs`.
- **SUMMARY/K-23.40** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L653): [E, state] Add one Pattern-2 `WorkspaceEvent` family, subject to W-9 live-count confirmation.
- **SUMMARY/K-23.41** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L654 | SURFACE): [E, configuration] Gate the track with `ELAI_WORKSPACE_ENABLE` and an allowlist promotion row.
- **SUMMARY/K-23.42** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L655 | SURFACE): [E, fixture] On a two-repo fixture, `workspace map acme` reports exactly one Sound `types <- api` edge and zero false edges.
- **SUMMARY/K-23.43** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L656 | SURFACE): [E, impact proof] `workspace impact` reports the real consumer in repo B with zero false consumers.
- **SUMMARY/K-23.44** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L657 | SURFACE): [E, orchestration proof] `workspace orchestrate` lands two PRs and a green cross-repo contract test.
- **SUMMARY/K-23.45** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L658): [E, record proof] WorkspaceEvent rows in `workspace.ssb.db` are GD-2 recomputable.
- **SUMMARY/K-23.46** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L659 | SURFACE): [E, UI proof] The canvas altitude re-derives byte-identically.
- **SUMMARY/K-23.47** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L660): [E, negative] A fabricated edge between disconnected members fails.
- **SUMMARY/K-23.48** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L661): [E, negative] A Heuristic edge driving the impact gate fails.
- **SUMMARY/K-23.49** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L662 | SURFACE): [E, negative] A workspace store written under a member root fails.
- **SUMMARY/K-23.50** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L663 | SURFACE): [E, local-first] A centralized cloud workspace store fails.
- **SUMMARY/K-23.51** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L664): [E, deferred] Cross-seat/org-shared team memory is out until a separate per-seat opt-in decision under J-49/J-59; it is not a prerequisite.
- **SUMMARY/K-23 workspace:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L826 | SURFACE):  each root retains its own deep index/FSM; the workspace layer derives Sound cross-repo edges, federates retrieval, expands impact only over Sound edges, orders delegated per-root work, journals workspace state globally, and renders a deterministic canvas altitude.
- **SUMMARY/K-23 workspace anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1002 | SURFACE):  `paths.rs:173`, `crates/orchestrator/src/workspace/`, CLI workspace verbs, `crates/orchestrator/src/blast_radius.rs`, `workspace.ssb.db`, `ELAI_WORKSPACE_ENABLE`, canvas WORKSPACE altitude.

### Summary — K-24 (50)

- **SUMMARY/K-24.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L670): [E, business model] Convert a subscription into monthly credits denominated in real provider dollars.
- **SUMMARY/K-24.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L671): [E, metering] Spend credits per metered provider usage.
- **SUMMARY/K-24.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L672 | SURFACE): [E, upstream] ELAI pays upstream provider cost.
- **SUMMARY/K-24.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L673 | SURFACE): [E, controls] ELAI owns limits, rollover, and routing policy.
- **SUMMARY/K-24.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L674): [E, safety] Cap every request; never market or implement an unlimited agent loop.
- **SUMMARY/K-24.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L675): [E, value thesis] Position the advantage as more resolved tasks per dollar through local routing, verification, and caching—not a provider deal.
- **SUMMARY/K-24.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L676): [E, margin] Treat the J-54 local/free pool, made correct by J-55, as the margin engine.
- **SUMMARY/K-24.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L677): [E, L1 event] Add `CreditLedgerEvent` as an SSB sub-journal event family.
- **SUMMARY/K-24.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L678): [E, L1 fold] Compute balance by folding `Granted - Settled - open Reserved - Expired`.
- **SUMMARY/K-24.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L679): [E, L1 prohibition] Never store balance as a mutable counter.
- **SUMMARY/K-24.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L680): [E, L1 integrity] Hash-chain the ledger using I-18/J-69.
- **SUMMARY/K-24.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L681): [E, L1 concurrency] Use SSB's single writer as double-spend defense.
- **SUMMARY/K-24.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L682 | SURFACE): [E, L1 CLI] Add `billing balance`.
- **SUMMARY/K-24.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L683): [E, L2 reservation] Estimate request cost before dispatch.
- **SUMMARY/K-24.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L684): [E, L2 reservation] Emit a `Reserved` hold before the provider call.
- **SUMMARY/K-24.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L685): [E, L2 metering] Execute with J-29 live metering.
- **SUMMARY/K-24.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L686): [E, L2 runaway safety] Enforce the J-81/E-39 runaway cap during the call.
- **SUMMARY/K-24.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L687): [E, L2 settlement] Settle from returned actual cost.
- **SUMMARY/K-24.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L688): [E, L2 release] Release the unused reservation in the same tick.
- **SUMMARY/K-24.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L689): [E, L3 backend] Add a managed OpenRouter backend as B-37 backend #12.
- **SUMMARY/K-24.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L690): [E, L3 dependency] Implement the backend directly/hand-rolled per J-54; add no new dependency.
- **SUMMARY/K-24.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L691): [E, L3 booking] Capture actual cost through J-28.
- **SUMMARY/K-24.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L692): [E, L4 admission] Gate J-49 seat admission on credit balance.
- **SUMMARY/K-24.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L693): [E, L4 limits] Apply tier rate limits.
- **SUMMARY/K-24.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L694 | SURFACE): [E, L4 failure] Refuse out-of-credit work with a clear user-facing surface.
- **SUMMARY/K-24.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L695): [E, L5 refill] Refill credits on subscription renewal.
- **SUMMARY/K-24.27** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L696): [E, L5 rollover] Apply rollover rules defined as data in `seats.toml`.
- **SUMMARY/K-24.28** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L697): [E, L5 webhook] Translate payment webhooks into `Granted` and `Expired` events.
- **SUMMARY/K-24.29** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L698 | SURFACE): [E, L5 boundary] Keep Stripe/Paddle outside ELAI and maintain zero PCI scope.
- **SUMMARY/K-24.30** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L699): [E, L6 measurement] Run SAFEDELTA for dollars per resolved task.
- **SUMMARY/K-24.31** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L700): [E, L6 claims gate] Gate any “$20=$200” style claim on the ELAI-measured result.
- **SUMMARY/K-24.32** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L701): [E, L6 business gate] Gate launch on measured local-lane resolve rate because frontier-margin cost remains material.
- **SUMMARY/K-24.33** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L702): [E, registry boundary] Use OpenRouter as transport+meter, never as the dynamic model registry; preserve I-51's registry rejection.
- **SUMMARY/K-24.34** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L703): [E, fallback] Preserve B-37 direct-provider fallback.
- **SUMMARY/K-24.35** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L704 | SURFACE): [E, implementation] Add `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, and `elai billing` CLI.
- **SUMMARY/K-24.36** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L705 | SURFACE): [E, integration] Wire J-49 admission, J-28 booking, and an H-9 CloudPanel billing tile.
- **SUMMARY/K-24.37** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L706): [E, event kinds] Support `Granted`, `Reserved`, `Settled`, `Refunded`, `Expired`, and `AdjustmentManual` kinds.
- **SUMMARY/K-24.38** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L707 | SURFACE): [E, configuration] Gate the track with `ELAI_BILLING_ENABLE` and an allowlist promotion row.
- **SUMMARY/K-24.39** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L708): [E, proof] A live OpenRouter call books `Settled == returned actual cost`.
- **SUMMARY/K-24.40** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L709 | SURFACE): [E, proof] `billing balance` folds correctly.
- **SUMMARY/K-24.41** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L710): [E, proof] A renewal webhook emits `Granted(face+bonus)` plus `Expired(over-cap)`.
- **SUMMARY/K-24.42** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L711): [E, negative] A mutable-counter balance fails.
- **SUMMARY/K-24.43** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L712): [E, split-brain] Using OpenRouter/LiteLLM spend storage as ELAI's ledger fails.
- **SUMMARY/K-24.44** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L713): [E, negative] A settlement different from metered actual cost fails.
- **SUMMARY/K-24.45** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L714): [E, negative] An unsupported “$20=$200” claim fails K-1 lint.
- **SUMMARY/K-24.46** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L715): [E, copy constraint] Pricing copy never says “unlimited.”
- **SUMMARY/K-24.47** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L716): [E, positioning] Sell as “proven more value per dollar, verified,” never merely “cheap.”
- **SUMMARY/K-24.48** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L717): [E, GTM hypothesis] Treat a $39-49/team-or-agency seat as a plausible first revenue path from GTM memory, not a verified launch fact.
- **SUMMARY/K-24 billing:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L827):  seat admission reserves a hash-chained credit balance, meters the real provider call under runaway caps, settles actual cost, and folds balance from events; business claims remain blocked until SAFEDELTA measures dollars per resolved task.
- **SUMMARY/K-24 billing anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1003 | SURFACE):  `crates/orchestrator/src/billing/`, `openrouter_backend.rs`, `seats.toml`, `ssb.rs:1508`, `action_type_lock.rs:775`, `ELAI_BILLING_ENABLE`, H-9 `CloudPanel`.

### Summary — K-25 (32)

- **SUMMARY/K-25.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L723 | SURFACE): [E, snapshot] Add `workflow/status/provider-surface.json`.
- **SUMMARY/K-25.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L724): [E, schema] Each snapshot row contains `provider`.
- **SUMMARY/K-25.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L725): [E, schema] Each snapshot row contains `capability_key`.
- **SUMMARY/K-25.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L726): [E, schema] Each snapshot row has status `NATIVE`, `PARTIAL`, or `ABSENT`.
- **SUMMARY/K-25.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L727): [E, schema] Each snapshot row contains `source_url`.
- **SUMMARY/K-25.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L728): [E, schema] Each snapshot row contains `verified_date`.
- **SUMMARY/K-25.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L729): [E, parse contract] A row without a source or date fails parsing.
- **SUMMARY/K-25.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L730 | SURFACE): [E, tool] Add stdlib `.claude/tools/absorption_watch.py`.
- **SUMMARY/K-25.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L731 | SURFACE): [E, CLI] Support `--json` output.
- **SUMMARY/K-25.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L732 | SURFACE): [E, CLI] Support `--max-age-days`, default 90.
- **SUMMARY/K-25.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L733): [E, join] Join the provider snapshot to K-8 `ABSORBED`/`AT-RISK`/`DURABLE` feature-map tags.
- **SUMMARY/K-25.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L734): [E, exit code] Exit 0 for clean/no overlap.
- **SUMMARY/K-25.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L735): [E, exit code] Exit 1 for `NATIVE_OVERLAP`, naming the overlapping rows.
- **SUMMARY/K-25.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L736): [E, exit code] Exit 2 for parse errors and fail closed.
- **SUMMARY/K-25.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L737): [E, exit code] Exit 3 for a stale snapshot.
- **SUMMARY/K-25.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L738): [E, doctrine] Add `.claude/rules/absorption-watch.md` on K-8's conditional-routing lane.
- **SUMMARY/K-25.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L739): [E, stale behavior] On exit 3, refresh via WebSearch and re-date sources before executing the efficiency task.
- **SUMMARY/K-25.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L740 | SURFACE): [E, overlap behavior] Surface exit-1 rows as kill/de-scope proposals.
- **SUMMARY/K-25.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L741): [E, authority] Never auto-apply those proposals.
- **SUMMARY/K-25.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L742): [E, epistemic boundary] Declare the keyword join unsound and advisory/LLM-adjudicated.
- **SUMMARY/K-25.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L743): [E, done-gate boundary] Never let the watchdog verdict discharge a runtime done gate or audit PASS.
- **SUMMARY/K-25.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L744 | SURFACE): [E, precedent] Mirror the `elai_status_query.py` Layer-0 advisory-cache boundary.
- **SUMMARY/K-25.23** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L745): [E, sibling] Let K-9's cadence serve as a natural provider-snapshot refresh trigger without merging their axes.
- **SUMMARY/K-25.24** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L746 | SURFACE): [E, state/UI] Add no SSB family and no frontend.
- **SUMMARY/K-25.25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L747): [E, proof-spine status] Mark the unit spine-exempt; it contributes no PROOF_SPINE number.
- **SUMMARY/K-25.26** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L748): [E, fixture] One NATIVE snapshot row overlapping one planned unit returns exit 1 and names the pair.
- **SUMMARY/K-25.27** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L749): [E, fixture] A stale snapshot returns exit 3 and never silently reports clean.
- **SUMMARY/K-25.28** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L750): [E, fixture] A malformed row returns exit 2.
- **SUMMARY/K-25.29** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L751): [E, integration test] A live efficiency task loads the rule and follows the exit code.
- **SUMMARY/K-25.30** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L752): [E, violation] Citing the advisory checker as a done-gate/audit PASS is a violation.
- **SUMMARY/K-25.31** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L753): [E, violation] Auto-applying de-scope remains a house-rule violation.
- **SUMMARY/K-25 watchdog anchors:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L1004 | SURFACE):  `workflow/status/provider-surface.json`, `.claude/tools/absorption_watch.py`, `.claude/rules/absorption-watch.md`, `elai_status_query.py`, `.claude/rules/proof-spine.md`, `proof_spine_status.py`.

### Summary — K-26 (22)

- **SUMMARY/K-26.01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L759): [E, campaign] Define a hard-deadlined five-day operations/build campaign to spend two expiring `bmapi`/`codex-proxy` credit pools.
- **SUMMARY/K-26.02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L760): [E, historical deadline] Record the stated credit expiry as 2026-07-15; this is historical/stale operational context, not a continuing product behavior.
- **SUMMARY/K-26.03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L761): [E, proof target] Feed proof-spine objectives PS-2, PS-4, and PS-5.
- **SUMMARY/K-26.04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L762): [E, work item] Finish and commit J-28.
- **SUMMARY/K-26.05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L763): [E, work item] Run the SP-3B GPT-5.5 live smoke.
- **SUMMARY/K-26.06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L764 | SURFACE): [E, dogfood] Put real `.elai_cc` fixes through `elai run` for model-agnostic accrual.
- **SUMMARY/K-26.07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L765 | SURFACE): [E, seam gap] Record FORK-5: `elai run` lacks a GPT-5.5 seam.
- **SUMMARY/K-26.08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L766): [E, route A] Option A widens B-27's Anthropic-only validation as dogfood task zero.
- **SUMMARY/K-26.09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L767): [E, route B] Option B uses an existing lane plus per-call GPT-5.5 Agent-tool subagents.
- **SUMMARY/K-26.10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L768): [E, work item] Lock the K-4 gate-tax ceiling for PS-2.
- **SUMMARY/K-26.11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L769): [E, work item] Run the PS-5 journal probe.
- **SUMMARY/K-26.12** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L770 | SURFACE): [E, conditional build] Build `--suite journal` conditionally from the probe result.
- **SUMMARY/K-26.13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L771): [E, work item] Refine-before-execute and build K-2 for PS-4.
- **SUMMARY/K-26.14** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L772): [E, operations] Rotate from account A to account B, then burn surplus credits.
- **SUMMARY/K-26.15** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L773): [E, plan validation] Record `plan_gate` PASS 10/10.
- **SUMMARY/K-26.16** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L774): [E, red team] Record a non-author red team with PLAN-BLOCK v1.
- **SUMMARY/K-26.17** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L775 | SURFACE): [E, repaired defect] Patch the `elai run` model-seam gap found by the red team.
- **SUMMARY/K-26.18** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L776): [E, repaired defect] Patch the wrong VERIFY-query table name found by the red team.
- **SUMMARY/K-26.19** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L777): [E, scope exclusion] Leave PS-1 RunPod-gated and out of the campaign.
- **SUMMARY/K-26.20** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L778): [E, scope exclusion] Leave PS-3 in L-track and out of the campaign.
- **SUMMARY/K-26.21** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L779): [E, scope exclusion] Leave unrelated harness fixes out.
- **SUMMARY/K-26.22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L780): [A, summary drift] The summary title still says “all 24 sessions” and the body ends at K-25 even though this K-26 preamble was appended; the original is mandatory for completion/status.

### Summary — 1. Falsify, measure, govern the claim, then distribute it (5)

- **SUMMARY/WF-A-1** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L786):  A-1 runs first and can kill the wedge before K builds around it.
- **SUMMARY/WF-A-2-A-3** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L787):  A-2/A-3 , **K-2**, **K-4**, **K-15**, **K-18**, and the J-owned proof legs produce measured results; fixture and pending/live tiers remain visibly distinct.
- **SUMMARY/WF-K-1** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L788):  K-1 is the central truth registry and lint: every external number, design statement, and measured claim receives an honest status and recompute/source pointer.
- **SUMMARY/WF-K-3** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L789 | SURFACE):  K-3 turns closed-set machine evidence into PR/check-run verdicts; **K-11** adds the human evidence view and DSR prioritization.
- **SUMMARY/WF-K-6** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L790):  K-6 , **K-7**, **K-13**, **K-14**, **K-15**, **K-17**, **K-18**, and **K-20** reuse K-1 rather than inventing separate claim-governance systems.

### Summary — 2. Make verification affordable without letting the model weaken it (3)

- **SUMMARY/WF-K-4** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L794):  K-4 measures composed wall-clock and over-block cost across the whole gate stack and locks the acceptable ceiling from real data.
- **SUMMARY/WF-K-22** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L795):  K-22 tells K-4 which doctrine clauses have real frozen enforcement versus advisory-only enforcement.
- **SUMMARY/WF-K-5** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L796):  K-5 chooses deterministic intensity using risk facts, keeps cheap protections always on, and escalates expensive gates only on fixed triggers.

### Summary — 3. Package one evidence spine for code review and procurement (4)

- **SUMMARY/WF-K-3-K-11** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L803 | SURFACE):  K-3/K-11 render the same evidence set in the PR and cockpit; missing evidence becomes `UNVOUCHED`.
- **SUMMARY/WF-K-6-804** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L804):  K-6 composes the redacted RRA, attestation, audit extract, retention mode, and clause mapping into one offline-verifiable bundle.
- **SUMMARY/WF-K-13** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L805):  K-13 cross-links OWASP controls into that procurement view without claiming certification.
- **SUMMARY/WF-K-7** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L806):  K-7 uses only K-1-governed numbers and is human-reviewed before external use.

### Summary — 4. Keep provider/model choices fresh without automated destructive decisions (4)

- **SUMMARY/WF-K-8** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L810):  K-8 classifies efficiency work as absorbed, at-risk, or durable and produces only a proposal for removal.
- **SUMMARY/WF-K-25** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L811):  K-25 persists a dated provider-native snapshot, fails stale/malformed state with distinct exit codes, and requires refresh before execution.
- **SUMMARY/WF-K-25-812** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L812):  K-25 remains advisory: its keyword join cannot satisfy a done gate and cannot remove work.
- **SUMMARY/WF-K-9** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L813):  K-9 evaluates the separate model-lane axis quarterly; regressions keep the incumbent and adoption still needs user ratification.

### Summary — 6. Add the two net-new K capability tracks without bypassing the spine (17)

- **SUMMARY/Claim truth:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L832 | SURFACE):  measured ELAI number plus recompute pointer, or explicit design/external/pending status; never a bare superlative or imported research number presented as ELAI's result.
- **SUMMARY/Closed-set evidence:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L833):  render only evidence present in the RRA/manifest; no artifact means `UNVOUCHED`, not clean.
- **SUMMARY/Fail-closed missingness:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L834):  missing cost capture, archival, critical atom, observer event, enforcer mapping, attestation digest, or snapshot schema cannot silently pass.
- **SUMMARY/Deterministic authority:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L835):  the LLM does not choose gate intensity, set must-preserve atoms, adjudicate critical-atom recall, select from self-evaluation, grade doctrine enforcement, or reconcile observer completeness.
- **SUMMARY/Frozen judgment:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L836):  a run cannot mutate its own gate-intensity table; rule/checker claims are grounded in the frozen gate manifest.
- **SUMMARY/External receipts:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L837):  reasoning traces and self-reports are UX only; execution/test/retrieval receipts decide confidence and selection.
- **SUMMARY/Integrity is not completeness:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L838):  I-18/J-69 hash chaining protects recorded rows, while K-21 separately demands mediation/observation and reconciliation for omissions.
- **SUMMARY/Exact handoff fidelity:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L839):  must-preserve atoms are exact-string/symbol obligations; paraphrase is failure.
- **SUMMARY/Time determinism:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L840 | SURFACE):  volatile time comes after cache breakpoints, exact time input is journaled, wall time never decides gates/transitions/promotions, and supervisor wall-clock kill is outside replay.
- **SUMMARY/Human governance:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L841):  removal, model adoption/routing change, unenforced-clause compilation, and cross-seat sharing need explicit user decisions.
- **SUMMARY/Observe-first/default-off:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L842 | SURFACE):  risky case reuse, observer enforcement, new workspace, and billing promotion follow observe-first or allowlisted activation as specified.
- **SUMMARY/No duplicate substrate:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L843):  K renderers, benches, doctrine, exports, and differentiator rows extend named F/G/H/I/J/K components rather than fork them.
- **SUMMARY/Local-first storage:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L844 | SURFACE):  workspace state cannot move under an arbitrary member root or into a centralized cloud store; credit truth cannot be delegated to a provider spend store.
- **SUMMARY/Privacy/security:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L845):  compliance export must be redacted; payment processors remain outside ELAI/PCI scope; reused cases never enter the auditor; repository provenance is retained.
- **SUMMARY/Honest platform/mode boundaries:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L846 | SURFACE):  Linux observation is not advertised on Windows; disconnected repositories get no invented edges; cloud/action-faithful replay is not mislabeled token-faithful; cassettes prove an old captured world.
- **SUMMARY/Cost safety:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L847):  billing requests are reserved and capped, settlements equal actual metering, and “unlimited” is forbidden.
- **SUMMARY/Advisory-cache boundary:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L848):  provider overlap detection can prompt a refresh/proposal but can never discharge objective verification.

### Summary — Explicit technical companions that are not K deliverables (7)

- **SUMMARY/OWASP:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L882):  WI-1 transformation-surviving taint binding, WI-2 cross-process taint, WI-3 E-37 perimeter completion, WI-6 config-ingress admission.
- **SUMMARY/Time:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L883):  T-1 timestamp slot, T-2 agent time, T-3 resume header, T-4 async-tool model loop, T-5 watchdog promotion.
- **SUMMARY/GRC-v3:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L884):  J-83..J-86 technical reasoning-engine track.
- **SUMMARY/Context activation:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L885):  IMP-1..IMP-8 as enumerated under K-17.
- **SUMMARY/Verified Case Graph:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L886 | SURFACE):  proof-minted typed nodes, deterministic edges, MEMORY/CASE canvas altitude, gated reuse, episodic-to-skill-to-rule ladder.
- **SUMMARY/Lacuna E1:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L887):  G-3/Tier-C mediation extension, eBPF/LSM observer, reconciliation, and monitor-the-monitor.
- **SUMMARY/Other lacuna fills:** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L888):  E2 runtime-observed DKG edges; E3 machine-derived-first audit lane; L1 objective-correctness spec bundle plus cross-family jury; L8 plan-DAG producer/consumer type completeness; L3 J-14 re-execution; L4 usage-weighted reachability; L5 cross-language/dynamic blast cone; L6 orchestrator-FSM dispatch ordering; L7 cant-lie X mutation floor; and unspecified Tier-2/Tier-3/THIN fills.

### Summary — A-01 (1)

- **SUMMARY/A-01** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L892): [A, stale title] The title says “all 24 sessions” and “6 tracks K-T1..K-T6,” but the detailed body has 24 normal entries, headings only for K-T1..K-T5, plus a later K-26 preamble. K-T6's mapping is not recoverable from this summary.

### Summary — A-02 (1)

- **SUMMARY/A-02** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L893): [A, K-10] K-10 is described only as a sequencing/no-new-letters doctrine in `K-0_INDEX.md`; there is no `K-10_*.md` file in the current K directory.

### Summary — A-03 (1)

- **SUMMARY/A-03** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L894): [A, K-26] K-26's campaign has detailed preamble bullets but no normal track/lever/integration/observable-effect section, and its July 2026 credit deadline is historical.

### Summary — A-04 (1)

- **SUMMARY/A-04** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L895): [A, K-27] `K-27_gpt56_sol_isolated_executor_lane.md` exists on disk but is not mentioned in the 600-line summary. Its filename alone is insufficient to inventory behavior.

### Summary — A-05 (1)

- **SUMMARY/A-05** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L896): [A, K-18a] `K-18a_ps5_observe_arm_testrun.md` exists on disk but is not mentioned in the summary. It may relate to PS-5/K-18, but that relationship is not asserted here.

### Summary — A-06 (1)

- **SUMMARY/A-06** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L897 | SURFACE): [A, rollout] K-12 says default-ON behind a disable flag and also “observe-then-enforce”; the original must clarify the rollout transition and initial default.

### Summary — A-07 (1)

- **SUMMARY/A-07** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L898): [A, event counts] “LIVE 268/577” and several net `0..+1` estimates are authoring snapshots with W-9 re-grep instructions, not stable present-day schema counts.

### Summary — A-08 (1)

- **SUMMARY/A-08** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L899 | SURFACE): [A, market claims] “Industry has no number,” moat ranking, Polygraph comparisons, OpenRouter/LiteLLM facts, and paper collapse figures are grounding claims, not ELAI feature behavior; originals/grounding must preserve citations.

### Summary — A-09 (1)

- **SUMMARY/A-09** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L900): [I, completed-system composition] Once K-23 and K-24 are both built, workspace-orchestrated calls would presumably pass through the ordinary seat/meter controls because each member runs its own FSM, but the K summary never explicitly specifies workspace-billing interaction. Do not encode this presumed coupling without original/cross-plan confirmation.

### Summary — A-10 (1)

- **SUMMARY/A-10** [SUM] (11-K-SUMMARY_INDEX.discovery.md:L901): [I, event unification] Later N may canonicalize K event/measurement families, but K itself only states preferred reuse/net-at-most-one. This discovery preserves K's intent and does not preempt N's final schema.

## Superseded notes (newer-plan precedence)

Losing descriptions are retained here. Winner is the deep original or an explicit correction marker.

### SUP-001 — from `K-001/AC-011`

- Ledger anchor: `(DEEP-K-001.discovery.md:L354)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: dependency | K-8 feature-map inventory is marked STALE at plan time; sweep must re-grep live units. | K8-A002

### SUP-002 — from `K-002/KMB-C002`

- Ledger anchor: `(DEEP-K-002.discovery.md:L295)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: Brief treats github_pr as “VERIFY live… rename caused stale FAIL”; map confirms FULL WIRED (KC-2).

### SUP-003 — from `K-002/AC-005`

- Ledger anchor: `(DEEP-K-002.discovery.md:L334)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: A | K-6 bundle directory vs archive; compliance tile yes/no; E-38 staleness. | K6-A001–A003

### SUP-004 — from `K-002/SD-KGM-002`

- Ledger anchor: `(DEEP-K-002.discovery.md:L351)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **ADD/CORRECT** KC-1..KC-5 binding flips and DEAD 234/543 baseline. | 11-K:L41-L42, L944 | KGM-E002–E003, E014–E017, E027, C001

### SUP-005 — from `K-003/K2-C001`

- Ledger anchor: `(DEEP-K-003.discovery.md:L72)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: K-26 MOVE 6 correction: live-arm gate is W-10 infra, not J-28; summary-layer K-2.11 still states 'Block the live dollar leg on J-28' and is superseded by this source.

### SUP-006 — from `K-003/K2-C002`

- Ledger anchor: `(DEEP-K-003.discovery.md:L73)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: Summary cites args_bench.rs:75 seam; this source states the dispatcher chain is :52-93 and explicitly marks original :75 as STALE.

### SUP-007 — from `K-003/SD-K2-001`

- Ledger anchor: `(DEEP-K-003.discovery.md:L392)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary K-2.11 'Block the live dollar leg on J-28': source states J-28 SHIPPED and live gate is W-10 RunPod/remote-sidecar. | 11-K:L92 | K2-E001, K2-E009, K2-E010, K2-C001

### SUP-008 — from `K-003/SD-K2-002`

- Ledger anchor: `(DEEP-K-003.discovery.md:L393)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary K-2.01/K-2 seam `args_bench.rs:75`: source says dispatcher chain is :52-93 and marks :75 STALE. | 11-K:L82 | K2-E012, K2-C002

### SUP-009 — from `K-003/SD-K26-004`

- Ledger anchor: `(DEEP-K-003.discovery.md:L455)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** any reading that K-26 is only 'historical deadline + high-level moves': original is a full operations plan with production hooks and honesty caveats. | 11-K:L756-L781 | K26-E001, K26-E022-E023

### SUP-010 — from `K-003/AC-001`

- Ledger anchor: `(DEEP-K-003.discovery.md:L469)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: C | Summary live-$ blocked on J-28 vs source W-10 after J-28 shipped | K2-C001

### SUP-011 — from `K-003/AC-002`

- Ledger anchor: `(DEEP-K-003.discovery.md:L470)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: C | Summary args_bench :75 vs source :52-93 STALE | K2-C002

### SUP-012 — from `K-004/K4-C001`

- Ledger anchor: `(DEEP-K-004.discovery.md:L87)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: Summary-layer K-4 still describes ON/OFF ablation, wire-dark 105-row inventory, whole-system over-block, and args_bench.rs:75; refined source VETOES product ON/OFF, bans wire-dark inventory, splits two numbers + RIDER-b, and uses CLI exit-code contract without a line-:75 claim.

### SUP-013 — from `K-004/K18a-C001`

- Ledger anchor: `(DEEP-K-004.discovery.md:L223)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: Summary discovery A-05 states K-18a exists but is not described; this deep chunk invents the full inventory the summary omitted.

### SUP-014 — from `K-004/SD-K4-001`

- Ledger anchor: `(DEEP-K-004.discovery.md:L372)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary K-4.05/K-4.16 ON/OFF SAFEDELTA ablation and gates-OFF denominator: refined source VETOES product-level ON/OFF and uses per-gate isolation + freq-weighted model. | 11-K:L134-L136, L145 | K4-E012, K4-E017, K4-C001

### SUP-015 — from `K-004/SD-K4-002`

- Ledger anchor: `(DEEP-K-004.discovery.md:L373)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary K-4.11 wire-dark-allowlist 105-row gate inventory: source FIX-1 bans that registry as scope source (it is default-OFF exemptions, 135 rows). | 11-K:L142 | K4-E016, K4-C001

### SUP-016 — from `K-004/SD-K4-003`

- Ledger anchor: `(DEEP-K-004.discovery.md:L374)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary single "composed whole-system" number: source reports TWO labeled numbers never summed (coding_task vs audit) + over_block_rate. | 11-K:L133-L135 | K4-E004-E007

### SUP-017 — from `K-004/SD-K4-004`

- Ledger anchor: `(DEEP-K-004.discovery.md:L375)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary K-4.01 `args_bench.rs:75`: refined K-4 does not pin gate-tax to line 75; it specifies 4-file K-18a-pattern wiring. | 11-K:L132 | K4-E038, K4-C001

### SUP-018 — from `K-004/SD-K4-005`

- Ledger anchor: `(DEEP-K-004.discovery.md:L376)`
- Winner (target state): deep source (delta qualifies/corrects summary)
- Losing / corrective text: **CORRECT** summary "release gate" as implied Rust artifact: source ATTACK-4 makes breach the CLI non-zero EXIT CODE only. | 11-K:L137, L146 | K4-E039, K4-E034

### SUP-019 — from `K-004/AC-001`

- Ledger anchor: `(DEEP-K-004.discovery.md:L440)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: C | Summary K-4 ON/OFF + wire-dark inventory + single whole-system number vs refined two-number + VETO + RIDER-b design | K4-C001

### SUP-020 — from `K-004/AC-002`

- Ledger anchor: `(DEEP-K-004.discovery.md:L441)`
- Winner (target state): deep ledger (newer plan detail)
- Losing / corrective text: C | Summary K-18a absent (A-05) vs full on-disk plan inventory in this chunk | K18a-C001

### SUP-021 — from `K-001/K0-C001`

- Ledger anchor: `(DEEP-K-001.discovery.md:L121)`
- Winner (target state): deep C-atom (corrective) over summary/stale index
- Losing / corrective text: Index claims K adds almost no new capability, then records K-23/K-24 as first net-new-capability K files; both statements are intentional and co-present.

### SUP-022 — from `K-001/K20-C001`

- Ledger anchor: `(DEEP-K-001.discovery.md:L311)`
- Winner (target state): deep C-atom (corrective) over summary/stale index
- Losing / corrective text: Market gap language oscillates between REPLAYABLE (named market demand) and RE-EXECUTABLE (strict-stronger wedge ELAI banks); both are intentional: G-2 ships replayable; K-20 packages re-executable as stronger claim.

### SUP-023 — from `K-002/KGM-C001`

- Ledger anchor: `(DEEP-K-002.discovery.md:L65)`
- Winner (target state): deep C-atom (corrective) over summary/stale index
- Losing / corrective text: Brief claimed I-43 PLANNED and github_pr possibly dark; map flips both to SHIPPED/WIRED (KC-1/KC-2).

### SUP-024 — from `K-002/KMB-C001`

- Ledger anchor: `(DEEP-K-002.discovery.md:L294)`
- Winner (target state): deep C-atom (corrective) over summary/stale index
- Losing / corrective text: Brief states I-43 “exists as PLANNED — confirm no code landed” and SCHEMA 234/543 bait; K-GROUNDING-MAP later flips I-43 SHIPPED and baseline 236/545 (KC-1/KC-5). Authors MUST prefer the map.

### SUP-025 — from `K-003/K27-C001`

- Ledger anchor: `(DEEP-K-003.discovery.md:L356)`
- Winner (target state): deep C-atom (corrective) over summary/stale index
- Losing / corrective text: Source file is truncated mid-sentence at L33 ('reconcile walk-state H-10 (its'); subsequent MOVEs, acceptance, and end marker are ABSENT from the on-disk image. Any claim of complete K-27 campaign procedure beyond AI-1..AI-8 + MOVE 0 start is unsupported by this source.

## Closing audit block

### Ledgers merged

- Deep ledgers: **4/4** (DEEP-K-001, DEEP-K-002, DEEP-K-003, DEEP-K-004)
- Summary record: **1/1** (11-K-SUMMARY_INDEX.discovery.md)
- Total inputs: **5/5**

### Per-ledger atom counts

| Ledger | Source (SRC) | Interactions (IX) | Ambiguity/Conflict (AC) | Summary deltas (SD) | Other | Ledger total (ID-bearing) |
|---|---:|---:|---:|---:|---:|---:|
| K-001 | 224 | 20 | 12 | 32 | 0 | 288 |
| K-002 | 210 | 20 | 11 | 34 | 0 | 275 |
| K-003 | 250 | 18 | 8 | 32 | 0 | 308 |
| K-004 | 238 | 20 | 8 | 28 | 0 | 294 |
| SUMMARY | 699 orientation atoms | — | — | — | — | 699 |
| Superseded notes | — | — | — | — | — | 25 |

### Deep-ledger source-atom breakdown (traceable by unit prefix)

**K-001 source (224):** FIX=7, K0=72, K1=34, K16=29, K20=28, K8=25, K9=29

**K-002 source (210):** K17=27, K18=29, K22=31, K3=29, K6=26, KGM=32, KMB=36

**K-003 source (250):** K12=26, K13=24, K14=23, K15=24, K2=38, K21=23, K24=35, K26=33, K27=24

**K-004 source (238):** K11=27, K18a=26, K19=22, K23=39, K25=24, K4=53, K5=26, K7=21

- Deep source atoms subtotal: **922**
- Deep interaction atoms: **78**
- Deep AC atoms: **39**
- Deep summary-delta atoms: **126**
- Summary-layer atoms: **699**
- Superseded notes: **25**

### Aggregate distinct record count: **1864**

### Superseded notes count: **25**

### Zero-omission statement

ZERO-OMISSION: All four deep ledgers and the family summary record were merged.
Every extracted table row with a conforming atomic ID from each deep ledger is present in this aggregate under a ledger namespace.
Declared footer source atoms (for traceability): K-001=224, K-002=210, K-003=250, K-004=238 (sum 922 source).
Extracted source SRC rows: K-001=224, K-002=210, K-003=250, K-004=238 (sum 922).
Cross-file interactions, summary deltas, and ambiguity/conflict registers are namespaced by ledger (e.g. K-001/IX-003).
Losing target-state descriptions are retained under Superseded notes rather than deleted.
No deep-ledger atom was compressed into prose-only form.

ZERO-DUPLICATE: Plan-local IDs remain unique within their ledger namespace.
Interactions and summary deltas cite source atoms and keep separate IDs.
Summary-layer atoms are orientation records and may restate deep content; deep records remain authoritative for implement.

### Output path

`docs/thefinalglance/feature-discovery/aggregates/K-AGGREGATE.md`

