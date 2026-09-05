# Family N aggregate — ELAI deep feature discovery

## Document contract

- **Family:** N (INTERLACE / UNIFICATION; measurement spine, composition validator, promotion primitives, claim/receipt, arbitration, provenance lattice, human render, governance coverage, counterfactual engine, clarity/novelty wires, journal-fold, consistency/dedupe, OutputBoundary, edit-loop instruments, framing stability, governance closure; N-0..N-16).
- **Inputs:** `DEEP-N-001`, `DEEP-N-002`, and `14-N-SUMMARY_INDEX.discovery.md`.
- **Policy:** Every distinct atomic record is preserved with its ID and source anchor (ledger file + line). No prose compression. No silent drop of minor atoms.
- **Namespacing:** Deep atoms keep native IDs under ledger prefix `N-001/` or `N-002/`. Summary atoms use `SUMMARY/`. Cross-file IX/SD/AC stay namespaced by ledger (e.g. `N-001/IX-003`, `N-002/SD-N5-001`).
- **NEWER-PLAN PRECEDENCE (within family):** An explicit SUPERSEDED/replacement/CORRECT marker in a source beats everything; otherwise a higher plan number is newer and wins the **target state** description. Losing descriptions are retained as superseded notes with anchors. Deep ledgers supersede the summary orientation pass where they conflict.
- **Status language preserved:** PLANNED / BUILD / ACTIVATION / BUILD-on-shipped / DELETE / DEFERRED / EXTENSION / ADVISORY are not collapsed.
- **Generated:** 2026-08-12 (N-family aggregator pass)

### Ledgers merged

| Ledger | Path | Source atoms | IX | SD | AC | Subtotal |
|---|---|---:|---:|---:|---:|---:|
| N-001 | `docs/thefinalglance/feature-discovery/deep/DEEP-N-001.discovery.md` | 290 | 23 | 46 | 10 | **369** |
| N-002 | `docs/thefinalglance/feature-discovery/deep/DEEP-N-002.discovery.md` | 297 | 18 | 35 | 11 | **361** |
| SUMMARY | `docs/thefinalglance/feature-discovery/14-N-SUMMARY_INDEX.discovery.md` | inv=385 | prose=133 | tables=48 | — | **566** |
| Superseded notes | — | — | — | — | — | **9** |
| **Aggregate distinct records** | — | — | — | — | — | **1305** |

### Per-ledger declared vs extracted

- **N-001:** source 290/290; ix 23/23; sd 46/46; ac 10/10 — **exact**
- **N-002:** source 297/297; ix 18/18; sd 35/35; ac 11/11 — **exact**
- **SUMMARY:** inventory atoms extracted **385**; prose/workflow/constraint/surface/open-ambiguity atoms **133**; table-derived (OBS/WAVE/DEP) **48** — **complete for ID'd + structured sections**

### Family map (session inventory)

| Track / wave | Sessions (N-N) | Deep ledger owners |
|---|---|---|
| Index / doctrine | N-0 | N-001 |
| WAVE 1 keystone | N-1 | N-001 |
| WAVE 2 validator | N-2 | N-002 |
| WAVE 3 | N-3, N-4 | N-001 |
| WAVE 4 | N-5, N-6 | N-002 |
| WAVE 5 | N-7, N-8 | N-002 |
| WAVE 6 | N-9, N-10 | N-001 (N-9), N-002 (N-10) |
| WAVE 7 | N-11 | N-001 |
| WAVE 8 | N-12 | N-001 |
| WAVE 9 external-steal | N-13, N-14, N-15, N-16 | N-002 |
| Summary orientation | all | SUMMARY |

## Notation

- `[E]` / `E/*` explicit · `[I]` inference · `[A]` ambiguity · `[C]` conflict/correction · `[Δ]` summary delta · `[SURFACE]` user-facing · `[superseded]` losing target-state retained.
- Domain tags from deep ledgers (STATUS, CAP, SURFACE, FLOW, …) are preserved in the class field.
- Anchors: `N-N:Lx-Ly` = N-plan originals; `14-N:Ln` = summary discovery line; ledger-line points at deep/summary discovery file.

## Superseded notes (newer-plan precedence losers retained)

Losing descriptions kept with anchors. Target state wins for design; losers are not deleted.

- **SUP-001** [superseded]
  - **Target state:** N-001/N0-E001 + N-001/N0-C001 + N-001/SD-N0-007: program inventory is N-1..N-16 (16 units); external track N-13..N-16 appended.
  - **Losing description retained:** SUMMARY header "All 15 sessions" and open-ambiguity item 1 treating 15 vs 16 as unresolved (14-N:L1-L2, L26-L26, L827).
  - **Reason:** Deep N-0 explicit CORRECT: 16 units with external track; higher-plan/detail authority.
- **SUP-002** [superseded]
  - **Target state:** N-002/N6-C001 + N-002/SD-N6-001: six independent provenance scales (not five).
  - **Losing description retained:** SUMMARY N6-A01 premise "five provenance scales" (14-N:L14, L127-L130) and any five-scale framing.
  - **Reason:** Deep numbered source consistently enumerates six; CORRECT marker in N-002.
- **SUP-003** [superseded]
  - **Target state:** N-001/SD-N12-001 + N-001/N12-I001: eight findings over seven sub-slices (a) merges W-8+W-26; no eighth product.
  - **Losing description retained:** SUMMARY N12-A01 open whether a sub-slice is missing (14-N:L24, L408-L409, L828).
  - **Reason:** Deep detail CORRECT/RESOLVE; structural count not missing product.
- **SUP-004** [superseded]
  - **Target state:** N-001/N0-E077 WAVE 9 for N-13/N-14/N-15; summary matrix Wave 9 (14-N:L776-L778). Detail files say WAVE 4-adjacent / 3-4-adjacent / 5-adjacent as composition adjacency, not matrix replacement.
  - **Losing description retained:** Reading N13-I001/N14-I001/N15-I001 as replacing Wave 9 matrix placement.
  - **Reason:** N-0 index sequencing table is program authority for wave numbers; detail WAVE-*-adjacent is relative dependency adjacency. Both retained; matrix Wave 9 is target sequencing state.
- **SUP-005** [superseded]
  - **Target state:** N-001/SD-N1-007: summary-only consumers J-84/J-85, L-4/L-7, K-24/M-5 are NOT detail atoms; do not promote as N1-E* from detail.
  - **Losing description retained:** SUMMARY N1-E08..E10 as if they were in N-1 detail file.
  - **Reason:** Detail file omits them; deep qualifies them as summary-only; retain under SUMMARY namespace only.
- **SUP-006** [superseded]
  - **Target state:** N-002/N5-I001 + N-002/SD-N5-003: numbered source does not list E-39 among ten routers; only L-6 vs J-73 is trigger example.
  - **Losing description retained:** SUMMARY N5-E02 "J-84 escalation versus E-39 cost-block" as a first-class router conflict (14-N:L109-L110).
  - **Reason:** Deep qualifies E-39 as not in ten-router list; retain summary example as superseded/qualified.
- **SUP-007** [superseded]
  - **Target state:** N-001/N3-E022 + N0-E054: Kind set TrialCountApplied | DeflationBlocked | HoldoutBudgetSpent.
  - **Losing description retained:** SUMMARY N3-E17 showing only DeflationBlocked in places (14-N:L81-L83, L132, L713).
  - **Reason:** Deep full Kind set is target event schema.
- **SUP-008** [superseded]
  - **Target state:** N-002/N8-E003 five views; expect_ssb Kind list names four (N8-A002 / N0-A001 open). Target substrate = five views; Kind enum still A-open.
  - **Losing description retained:** SUMMARY N8-E13 "five Kinds" as if enum fully named (14-N:L176-L177).
  - **Reason:** Deep flags Kind list omits explicit J-70 Traceability; five views remain target; Kind completeness open.
- **SUP-009** [superseded]
  - **Target state:** N-002/N5-E003 + N-001/N0-E091: safety-gates > cost-ceiling > abstention > escalation > modality > serving-tier.
  - **Losing description retained:** SUMMARY N5-E06 short form safety > cost > abstention > escalation > modality > serving-tier (14-N:L112-L114).
  - **Reason:** Deep/detail expanded tier names win target wording; short form retained as summary-era phrasing.

---

## Per-unit atomic records (deep ledgers)

Feature and unit-local records grouped by unit / ledger. Cross-file interactions, summary deltas, and ambiguity/conflict registers follow in dedicated sections.

## Ledger N-001

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-N-001.discovery.md`
Extracted: source=290, IX=23, SD=46, AC=10, subtotal=369

### Unit `N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` — from N-001 (105 records)

- **N-001/N0-E001** [E/STATUS] anchor=`N-0:L1-L7` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L34): N-plan is the INTERLACE / UNIFICATION side, wiring-enforced; generation is 16 sessions covering Tier-1..4 plus external-steal track.
- **N-001/N0-E002** [E/EVIDENCE] anchor=`N-0:L8-L13` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L35): Primary synthesis is `workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md` over ELAI-FEATURE-MAP + phase-index-MASTER + every B..M summary; 36 findings W-1..W-36 plus M-1 one-registry meta-rule.
- **N-001/N0-E003** [E/DEP] anchor=`N-0:L14-L17` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L36): Predecessors are B-plan through M-plan; N COMPOSES-WITH all of them under GD-1 and restates none.
- **N-001/N0-E004** [E/CAP] anchor=`N-0:L14-L17, L31-L39` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L37): N invents almost nothing; each N-N either declares one subsystem from >=2 fragmented implementations or wires an existing producer to an existing consumer left unconnected by summary-mediated authoring.
- **N-001/N0-E005** [E/FLOW] anchor=`N-0:L19-L27` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L38): Mandatory predecessor reading before any N-N execution: interlace doc, ELAI-FEATURE-MAP (not the legacy 12-file list), F-0_INDEX (W-1..W-10), G-0_INDEX (GD-1..GD-5), this index, N-SUMMARY_INDEX, then the target N-N detail, plus per-unit B..M summary entries named in Existing-state.
- **N-001/N0-E006** [E/STATUS] anchor=`N-0:L31-L39` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L39): Residual weakness after LACUNA-to-M is not missing features but summary-driven parallel machines that do not talk; talking that does happen is hand-maintained in ELAI-FEATURE-MAP plus prose edges with zero machine checking.
- **N-001/N0-E007** [E/EVIDENCE] anchor=`N-0:L36-L37, L356-L357` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L40): J-SUMMARY_INDEX.md:2 is cited as a live instance of summary drift from its own letter.
- **N-001/N0-E008** [E/CAP] anchor=`N-0:L41-L50` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L41): Central finding: extending a base REGISTRY auto-composed without fragmentation (e.g. action_types 17->576 under golden lock); minting PARALLEL machinery for an existing concern produced fragmentation.
- **N-001/N0-E009** [E/CAP] anchor=`N-0:L47-L50, L98-L102` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L42): N is therefore an ACTIVATION program: unify existing units into one registry each; meta-rule M-1 (one-registry-per-concern) is enforced at plan-authoring time and built as N-2.
- **N-001/N0-E010** [E/TEST] anchor=`N-0:L48-L50` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L43): N ships under the same hard executed wiring done-gate F/G use so no unification can ship half-done (two live cores is the split-brain it kills).
- **N-001/N0-E011** [E/STATUS] anchor=`N-0:L52-L57` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L44): Status legend: PLANNED (file written, work not started — all 12 N-N at generation), IN PROGRESS, DONE (full done-gate), DEFERRED (named upstream).
- **N-001/N0-E012** [E/TEST] anchor=`N-0:L64-L66` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L45): W-1: DONE only when `wiring_contract_check.py --bn-file <N-N>.md --run-trigger` returns PASS (produces DEF, consumes CALL/REF, reachable_from BFS, stub_ban clean, observable_effect actually ran with SSB row + artifact); cargo-green is necessary but not sufficient.
- **N-001/N0-E013** [E/CFG] anchor=`N-0:L67-L68` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L46): W-2: no default-off env gate as terminal state; kill-switch DISABLE only; default ON; observable_effect runs with no env override.
- **N-001/N0-E014** [E/CFG] anchor=`N-0:L68-L69` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L47): W-2 N caveat: a UNIFIER that replaces live paths defaults ON behind the retire switch; the OLD path survives one release behind `ELAI_*_LEGACY` for rollback, then ND-2 retires it.
- **N-001/N0-E015** [E/INT] [SURFACE] anchor=`N-0:L70-L71` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L48): W-3: reachable from a REAL production entrypoint (CLI / FSM transition / Pre/PostToolUse hook / Tauri command); a `#[cfg(test)]`-only caller is wire-dark.
- **N-001/N0-E016** [E/SURFACE] [SURFACE] anchor=`N-0:L72-L72` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L49): W-4: frontend_pair mandatory for any dashboard-surfaceable SSB signal; `cross_lang_diff.py` PASS.
- **N-001/N0-E017** [E/TEST] anchor=`N-0:L73-L73` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L50): W-5: observable_effect is a LIVE end-to-end trigger, not a unit test.
- **N-001/N0-E018** [E/TEST] [SURFACE] anchor=`N-0:L74-L74` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L51): W-6: >=1 ACCEPTANCE item verified THROUGH the dashboard (dashboard-smoker), or N/A with one-line justification.
- **N-001/N0-E019** [E/DEP] anchor=`N-0:L75-L78` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L52): W-7: no hallucinated integration points; consumes_symbol / reachable_from / MODIFY greped in elai.map at authoring; N labels code anchors "PLAN-asserted; executor re-greps per W-7" when not freshly greped; cross-letter unit IDs verifiable from summary indexes.
- **N-001/N0-E020** [E/INT] anchor=`N-0:L79-L79` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L53): W-8: produces/consumes symmetry — every produced symbol has an in-plan caller.
- **N-001/N0-E021** [E/EVENT] anchor=`N-0:L80-L81` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L54): W-9: new SSB action_types flow the action_type golden lock (Pattern-2 single-variant-with-Kind where applicable); SCHEMA bump named, never hard-coded.
- **N-001/N0-E022** [E/CFG] anchor=`N-0:L82-L83` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L55): W-10: tiered-gate — any axis needing network/subprocess at gate time is SPLIT; in-process Tier-1 is the fail-closed done-gate.
- **N-001/N0-E023** [E/LIMIT] anchor=`N-0:L86-L88` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L56): GD-1 COMPOSE-NOT-RESTATE: every N-N consumes named prior-plan artifacts; re-implementing any B..M shipped surface is a P0 to gotchas.md; N binds, does not restate.
- **N-001/N0-E024** [E/DET] anchor=`N-0:L89-L89` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L57): GD-2 REPRODUCIBLE-METRIC: any number an N-N emits is recomputable from the SSB stream under a pinned ELAI_SEED.
- **N-001/N0-E025** [E/OBS] anchor=`N-0:L90-L91` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L58): GD-3 ABLATION-FRAMING: any capability/latency/cost number is framed unifier-ON-vs-fragmented-OFF, never as a model-capability claim.
- **N-001/N0-E026** [E/TEST] anchor=`N-0:L92-L93` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L59): GD-4 FP-FRICTION RELEASE GATE: no N-N gate flips default-ON until its FP/over-block rate is measured and bounded on >=3 real runs through N-1's spine per ND-4.
- **N-001/N0-E027** [E/LIMIT] anchor=`N-0:L94-L96` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L60): GD-5 NO-FABRICATED-GUARANTEE: a unifier that has not yet retired its superseded path carries honest residue (two-cores-during-migration; adapter-thinness unproven), never a false "single-source" claim.
- **N-001/N0-E028** [E/LIMIT] anchor=`N-0:L99-L102` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L61): ND-1 UNIFY-NOT-FORK (M-1 made blocking): every N-N replaces >=2 fragmented implementations of ONE concern with ONE canonical object/engine/substrate + thin adapters; adding an (n+1)th parallel machine is a P0; N-2 builds the authoring-time checker.
- **N-001/N0-E029** [E/RECOVERY] anchor=`N-0:L103-L106` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L62): ND-2 RETIREMENT-MAP MANDATORY: any N-N that unifies K implementations ships an explicit retirement map (each superseded path -> parameter/adapter in unifier), old path behind `ELAI_*_LEGACY` one release, then USER-CONFIRMED removal; "add unifier and hope" is refused.
- **N-001/N0-E030** [E/DATA] anchor=`N-0:L107-L109` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L63): ND-3 EDGE-AS-DATA: every producer/consumer/unifies edge across letters is emitted as a machine-checkable row (corpus-validator input format N-2 defines), never prose-only.
- **N-001/N0-E031** [E/OBS] anchor=`N-0:L110-L113` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L64): ND-4 MEASURED-THROUGH-THE-SPINE: any N-N that claims a number routes it through N-1's measurement spine + ONE ModelCapabilityProfile calibration sink + N-3's trial-count deflation; no N-N mints its own calibration store or promotion threshold.
- **N-001/N0-E032** [E/LIMIT] anchor=`N-0:L114-L116` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L65): ND-5 ADAPTER-THINNESS: when unifying engines, per-target adapters are THIN (parameter/dispatch only); all business logic lives in the ONE core; a fat adapter re-implementing the core fails as ND-1 fork in disguise.
- **N-001/N0-E033** [E/TEST] [SURFACE] anchor=`N-0:L118-L122` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L66): DONE requires all eight: (1) cargo check --workspace GREEN, (2) auditor PASS (sole test runner), (3) wiring_contract_check.py --run-trigger PASS, (4) cross_lang_diff PASS if frontend_pair present, (5) W-6 dashboard E2E item, (6) GD-4 FP-rate bound for any default-ON gate, (7) ND-2 retirement map present + user-confirmed when superseding a live path, (8) memory-writer ran.
- **N-001/N0-E034** [E/STATUS] anchor=`N-0:L128-L136` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L67): N-1 is TIER 1 CRITICAL KEYSTONE BLOCKING, findings W-1+W-11+W-23, ACTIVATION, effort XL, Wave 1: unify ten benches + corpus feeders + one runner + outcome-label fold + BOOTSTRAP pack into one MEASUREMENT SPINE.
- **N-001/N0-E035** [E/STATUS] anchor=`N-0:L139-L145` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L68): N-2 is TIER 2, W-6+M-1+W-33+W-32, BUILD, L, Wave 2, depends N-1: corpus-grain composition validator + M-1 one-registry rule (ND-1/ND-3 checker).
- **N-001/N0-E036** [E/STATUS] anchor=`N-0:L146-L152` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L69): N-3 is TIER 2, W-30, ACTIVATION, M-L, Wave 3, depends N-1: lift H-17/H-18/H-19 from finance to domain-generic promotion primitives via ID-6.
- **N-001/N0-E037** [E/STATUS] anchor=`N-0:L153-L159` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L70): N-4 is TIER 2, W-2+W-4+W-22+W-36, BUILD-on-shipped, L, Wave 3: one Claim/Belief object + receipt memo.
- **N-001/N0-E038** [E/STATUS] anchor=`N-0:L160-L166` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L71): N-5 is TIER 2, W-3+W-17+W-35+W-14, BUILD, L, Wave 4, depends N-1: policy/intensity arbitration plane.
- **N-001/N0-E039** [E/STATUS] anchor=`N-0:L167-L173` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L72): N-6 is TIER 2, W-19+W-29, BUILD, L, Wave 4: provenance/trust lattice + admission-gate family (semantic mapping, not one shared enum).
- **N-001/N0-E040** [E/STATUS] anchor=`N-0:L174-L178` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L73): N-7 is TIER 2, W-7, BUILD, M, Wave 5: human render as REQUIRED verdict field.
- **N-001/N0-E041** [E/STATUS] anchor=`N-0:L181-L186` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L74): N-8 is TIER 3, W-18+W-34+W-31, BUILD, M, Wave 5: governance-coverage substrate (five views).
- **N-001/N0-E042** [E/STATUS] anchor=`N-0:L187-L191` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L75): N-9 is TIER 3, W-20+W-27, BUILD, M, Wave 6, depends N-1: one counterfactual/ablation engine.
- **N-001/N0-E043** [E/STATUS] anchor=`N-0:L192-L196` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L76): N-10 is TIER 3, W-15+W-24, ACTIVATION, M, Wave 6: clarity/novelty -> budget wires.
- **N-001/N0-E044** [E/STATUS] anchor=`N-0:L197-L202` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L77): N-11 is TIER 3, W-10, BUILD-on-shipped, M-L, Wave 7: journal-fold substrate extending L-16.
- **N-001/N0-E045** [E/STATUS] anchor=`N-0:L204-L212` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L78): N-12 is TIER 4, W-8+W-9+W-12+W-16+W-21+W-25+W-26+W-28, DELETE, M, Wave 8: consistency + dedupe bundle of eight collapses.
- **N-001/N0-E046** [E/STATUS] anchor=`N-0:L214-L219` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L79): External-steal track (2026-07-05) appends N-13..N-15 from four-source-steal analysis; only genuine NEW/EXTENSION verdicts land; PAW-tier, EvoPolicyGym-n=1-stats, Ford-checklist rejected; prefix-KV is already-recorded rider; zip-slip folds into N-6.
- **N-001/N0-E047** [E/STATUS] anchor=`N-0:L220-L224` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L80): N-13 is OutputBoundary + structured-output capability matrix, BUILD/UNIFY, L, Wave 9, S3-1+S3-2.
- **N-001/N0-E048** [E/STATUS] anchor=`N-0:L225-L229` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L81): N-14 is self-improvement edit-loop instruments (EvoPolicyGym), BUILD/ACTIVATION, M-L, Wave 9, ADVISORY (TD-3), feeds N-1/N-3/N-5.
- **N-001/N0-E049** [E/STATUS] anchor=`N-0:L230-L234` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L82): N-15 is anti-affirmation stability measurement, BUILD/ACTIVATION, M, Wave 9, advisory-only, GD-4 FP-bound before default-ON.
- **N-001/N0-E050** [E/STATUS] anchor=`N-0:L235-L247` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L83): N-16 (2026-07-08 DeepReason) is governance closure EXTENSION of N-8: evaluator-refutation collapse + lambda exogenous-grounding floor + adjudicator-ritual metrics; advisory-only TD-3; Dung argument-graph adjudicator REJECTED.
- **N-001/N0-E051** [E/DATA] anchor=`N-0:L255-L260` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L84): Live baseline at generation (2026-07-04): SCHEMA_VERSION 267 / TOTAL_VARIANTS 576; formula ssb.rs:1519 SCHEMA_VERSION = SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len(); newest shipped bump I-36; J/K/L/M planned bumps not landed.
- **N-001/N0-E052** [E/EVENT] anchor=`N-0:L262-L264` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L85): N-1 reuses existing bench action_types and adds `+calibration_bootstrap_superseded` Pattern-2 Kind Prefilled / LocallySuperseded / DriftInvalidated; baseline+1.
- **N-001/N0-E053** [E/EVENT] anchor=`N-0:L265-L266` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L86): N-2 adds `+composition_edge_checked` Pattern-2 Kind DanglingEdge / OrphanProducer / DuplicateConcept / FreshnessDrift; baseline+1; authoring-time gate still emits SSB for GD-2.
- **N-001/N0-E054** [E/EVENT] anchor=`N-0:L267-L267` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L87): N-3 adds `+promotion_deflated` Pattern-2 Kind TrialCountApplied / DeflationBlocked / HoldoutBudgetSpent; baseline+1.
- **N-001/N0-E055** [E/EVENT] anchor=`N-0:L268-L269` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L88): N-4 reuses J-83 BeliefState events + C-30 cache path; adds `+receipt_memo` Pattern-2 Kind Hit / Miss / Revalidated; baseline+1.
- **N-001/N0-E056** [E/EVENT] anchor=`N-0:L270-L270` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L89): N-5 adds `+route_arbitrated` composite RouteDecision (routers-consulted vec + winner + precedence-rank); baseline+1.
- **N-001/N0-E057** [E/EVENT] anchor=`N-0:L271-L273` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L90): N-6: no new action_type for lattice (rides G-1 taint_flow/provenance); conditional `+admission_scanned` Pattern-2 Kind Code / Config / LearnedArtifact / Workspace; baseline+0 or +1.
- **N-001/N0-E058** [E/EVENT] [SURFACE] anchor=`N-0:L274-L274` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L91): N-7 adds `+verdict_rendered` Pattern-2 Kind PrComment / ChatAck / CanvasBadge / BriefRow / OrgRollup; baseline+1.
- **N-001/N0-E059** [E/EVENT] anchor=`N-0:L275-L276` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L92): N-8 adds `+governance_coverage_measured` Pattern-2 Kind GateLiveness / DoctrineEnforcer / DebtRate / MutantKill; baseline+1.
- **N-001/N0-E060** [E/EVENT] anchor=`N-0:L277-L277` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L93): N-9 adds `+counterfactual_scored` Pattern-2 Kind Role / Verifier / Ranker / Memory / Case; baseline+1.
- **N-001/N0-E061** [E/EVENT] anchor=`N-0:L278-L279` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L94): N-10 adds NO new action_type (reuses B-38 ambiguity / D-15 novelty rows); baseline+0.
- **N-001/N0-E062** [E/EVENT] anchor=`N-0:L280-L281` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L95): N-11 adds NO new action_type (reuses L-16 fold events; adds fold FUNCTIONS); baseline+0.
- **N-001/N0-E063** [E/EVENT] anchor=`N-0:L282-L283` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L96): N-12 adds NO new action_type (dedupe removes emitters; surviving canonical type keeps its row); baseline+0; W-12 FSM-conformance may add test-only assertion.
- **N-001/N0-E064** [E/EVENT] anchor=`N-0:L284-L285` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L97): Net designed core N-plan bumps ~8 new action_types (N-1..N-5, N-7, N-8, N-9 + N-6 conditional); quote as baseline+N from LIVE 267/576 as of 2026-07-04; re-read at edit.
- **N-001/N0-E065** [E/EVENT] anchor=`N-0:L286-L287` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L98): N-13 adds `+output_boundary_verdict` Pattern-2 Kind SchemaEnforced / GrammarEnforced / RepairedThenValidated / RefusedFailClosed; baseline+1.
- **N-001/N0-E066** [E/EVENT] anchor=`N-0:L288-L289` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L99): N-14 reuses score-bearing attempt row `{edit_kind, improved}` or adds `+edit_kind_classified` Pattern-2 Kind Synthesis / Parametric / CandidateMgmt; baseline+0 or +1 prefer-reuse.
- **N-001/N0-E067** [E/EVENT] anchor=`N-0:L290-L291` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L100): N-15 adds `+framing_stability_measured` Pattern-2 Kind Stable / FramingUnstable / AssertionPulled or reuses D-23 detection Kind; baseline+0 or +1 prefer-reuse.
- **N-001/N0-E068** [E/EVENT] anchor=`N-0:L295-L298` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L101): N-16 adds `+evaluator_closure_swept` Pattern-2 Kind EvaluatorRefuted / VerdictQuarantined / Reinstated and `+grounding_ratio_measured` Pattern-2 Kind AboveFloor / BelowFloorBraked; S7 prefer-reuse governance_coverage_measured +Kind AdjudicatorRitual; evaluator-id+version-hash is additive FIELD on existing verdict rows; baseline+2.
- **N-001/N0-E069** [E/FLOW] anchor=`N-0:L302-L304` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L102): WAVE 1: N-1 only — critical spine; highest fan-in (~30 promotions); build first.
- **N-001/N0-E070** [E/FLOW] anchor=`N-0:L305-L307` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L103): WAVE 2: N-2 corpus validator; edges become DATA; unlock-DAG computed; ND-1/ND-3 mechanical; fan-in ranking emitted.
- **N-001/N0-E071** [E/FLOW] anchor=`N-0:L308-L309` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L104): WAVE 3: N-3 (deflation, needs N-1 labels) + N-4 (claim object + receipt memo) in parallel on measured+validated ground.
- **N-001/N0-E072** [E/FLOW] anchor=`N-0:L310-L311` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L105): WAVE 4: N-5 + N-6; disjoint files, parallel-safe.
- **N-001/N0-E073** [E/FLOW] anchor=`N-0:L312-L312` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L106): WAVE 5: N-7 + N-8.
- **N-001/N0-E074** [E/FLOW] anchor=`N-0:L313-L314` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L107): WAVE 6: N-9 (needs N-1) + N-10.
- **N-001/N0-E075** [E/FLOW] anchor=`N-0:L315-L315` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L108): WAVE 7: N-11 (journal-fold + L-16 retirement map).
- **N-001/N0-E076** [E/FLOW] anchor=`N-0:L316-L316` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L109): WAVE 8: N-12 (eight disjoint collapses; last, lowest risk).
- **N-001/N0-E077** [E/FLOW] anchor=`N-0:L317-L322` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L110): WAVE 9: N-13 (after N-2 authoring check), N-14 (after N-1/N-3/N-5), N-15 (after N-1; GD-4 before default-ON); all three parallel-safe with each other.
- **N-001/N0-E078** [E/DEP] anchor=`N-0:L323-L326` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L111): Three system-wide unlocks before Tier-2 unifications: (1) N-1 measurement spine, (2) I-1 supervisor plane (I-plan PRE-REQ, not N unit), (3) N-2 corpus validator.
- **N-001/N0-E079** [E/DEP] anchor=`N-0:L324-L326` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L112): Confirm I-1 status before WAVE 4; if unshipped, N-5/N-9 carry it as named DEFERRED dependency (GD-5), do not re-implement (GD-1).
- **N-001/N0-E080** [E/LIMIT] anchor=`N-0:L331-L332` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L113): Hard rules: blake3 only (never blake2/sha2 for non-RFC); rand>=0.9; tracing not log; no async-trait crate; no .unwrap()/.expect() on prod paths (test cfg + main entrypoint allowed).
- **N-001/N0-E081** [E/LIMIT] anchor=`N-0:L333-L334` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L114): Anti-slopsquatting: N should add ZERO new deps; any new dep is a red flag of re-implementation (ND-1) and needs verify + user-confirm.
- **N-001/N0-E082** [E/TEST] anchor=`N-0:L335-L335` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L115): One change at a time; auditor sole test runner; main LLM never runs tests.
- **N-001/N0-E083** [E/SEC] anchor=`N-0:L336-L337` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L116): No removal or simplification without explicit user confirmation — ND-2 retirement maps are USER-CONFIRMED, not auto-applied (done-gate clause 7).
- **N-001/N0-E084** [E/EVIDENCE] anchor=`N-0:L338-L338` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L117): Grounded-by-default: every cross-letter edge cites the summary entry it composes; interlace doc is the WHY.
- **N-001/N0-E085** [E/LIMIT] anchor=`N-0:L343-L346` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L118): Landmine: feature-map edges are prose until N-2 ships; executor must re-verify each "unifies X" edge against live summary + code (map OPEN CONFLICTS had 7 unresolved; I-2/I-5/I-11/H-50 stale-label corrections).
- **N-001/N0-E086** [E/LIMIT] anchor=`N-0:L347-L349` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L119): Landmine: F-6 chose type separation for SkillTrustTier vs memory-TrustTier; N-6 lattice is SEMANTIC MAPPING of six scales, NOT one shared enum.
- **N-001/N0-E087** [E/DEFER] anchor=`N-0:L350-L353` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L120): Landmine W-13 downgrade: L-16 already specifies most context retirement; N-11 residual is ONLY recover_stubs EOL + declaring H-49 CAS/undo-tree THE log representation — do not re-author the fold.
- **N-001/N0-E088** [E/DEFER] anchor=`N-0:L354-L355` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L121): Landmine W-2 downgrade: L-8 already threads L-1 over J-83; N-4 residual is K-12 critical_atoms + GRC tags joining the one object — schema alignment, not redesign.
- **N-001/N0-E089** [E/STATUS] [SURFACE] anchor=`N-0:L360-L364` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L122): Session log empty at generation; each shipped N-N adds row: context-compiler -> integration-explorer -> spec-grounder -> executor -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator (RUN_TRIGGER PASS REQUIRED) -> dashboard-smoker (if frontend_pair) -> memory-writer -> COMMITTING.
- **N-001/N0-E090** [E/FLOW] anchor=`N-0:L369-L371` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L123): Next step: Start WAVE 1 N-1 (KEYSTONE BLOCKING); then N-2 opens WAVE 2 and computes fan-in ranking for N-3..N-12.
- **N-001/N0-E091** [E/CFG] anchor=`N-0:L161-L163` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L124): Precedence order declared for N-5 in the index session table: safety > cost-ceiling > abstention > escalation > modality > serving-tier.
- **N-001/N0-E092** [E/DATA] anchor=`N-0:L164-L166` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L125): N-5 intensity resolver absorbs six deciders as axes of one table: B-41 level / L-21 model / J-84 effort / C-46 delegation / K-5 gate-risk / B-28 ModeSelector as J-55 edit-format leg.
- **N-001/N0-E093** [E/DATA] anchor=`N-0:L168-L171` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L126): N-6 lattice maps G-1 taint <-> C-48 memory T0-T3 <-> F-6 skill S1-S4 <-> C-30 source T1-T4 <-> K-23 repo trust <-> 4D EvidenceLevel on the SAME object ingest -> memory -> citation -> skill -> cross-repo.
- **N-001/N0-E094** [E/CAP] anchor=`N-0:L171-L173` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L127): N-6 admission family: F-1 VibeGate + J-102 config-ingress + J-109 learned-artifact + (new) K-23/M-2 surfaces share ONE scanner registry.
- **N-001/N0-E095** [E/SURFACE] [SURFACE] anchor=`N-0:L175-L178` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L128): N-7 render targets: PR comment / chat ACK line / canvas badge / brief row / M-3 org rollup; ONE render substrate, many verdict producers (generalizes K-11).
- **N-001/N0-E096** [E/CAP] anchor=`N-0:L182-L186` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L129): N-8 five views: B-17 gate-liveness + K-22 doctrine-enforcer coverage + J-46 debt-RATE + J-70 gate-manifest traceability + H-27 kill-your-own-mutant generalized to EVERY frozen gate.
- **N-001/N0-E097** [E/CAP] anchor=`N-0:L193-L196` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L130): N-10 types AmbiguityScore (B-38/B-40/C-29/J-32) + NoveltyScore (D-15/4F/B-21) and wires consumers: 4D research depth, J-84 effort, J-39 node-sizing, J-79 grill/clarify.
- **N-001/N0-E098** [E/CAP] anchor=`N-0:L206-L212` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L131): N-12 eight collapses named: FailureClass enum; AbstainDecision spine; ProjectClass registry; C-47/C-49 through J-109 admission; drift-primitive library; FSM three-model conformance; ONE LSP daemon pool (W-8+W-26, W-9, W-16, W-21, W-25, W-12, W-28).
- **N-001/N0-E099** [E/LIMIT] anchor=`N-0:L239-L242` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L132): N-16 S4: evaluator-refutation event triggers F-5 staleness cascade quarantining verdicts that evaluator issued (advisory re-audit queue, NEVER delete).
- **N-001/N0-E100** [E/LIMIT] anchor=`N-0:L242-L244` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L133): N-16 S6: lambda exogenous-grounding floor BRAKE forces retrieval/escalation, NEVER a PASS.
- **N-001/N0-I001** [I/STATUS] anchor=`N-0:L1-L7, L125-L126, L214-L247` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L134): Header "16 sessions" plus N-13..N-16 external track means the program inventory is N-1..N-16 even though the "12 sessions" table heading is TIER-1..4 only; external track is appended, not inside the original 12.
- **N-001/N0-A001** [A/EVENT] anchor=`N-0:L182-L186, L275-L276` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L135): N-8 Kind list in the schema section omits explicit Traceability kind while the session blurb includes J-70 gate-manifest traceability; whether Kind set is four or five is unresolved between L275-L276 and L182-L186.
- **N-001/N0-A002** [A/CFG] anchor=`N-0:L68-L69, L103-L106` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L136): Exact `ELAI_*_LEGACY` env-name pattern, release-window length, and rollback switch wiring are named as one-release policy without concrete flag names.
- **N-001/N0-A003** [A/DEP] anchor=`N-0:L324-L326` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L137): "Confirm I-1 status before WAVE 4" leaves the exact N-5/N-9 deferred-shim behavior unspecified when I-1 is still unshipped.
- **N-001/N0-C001** [C/STATUS] anchor=`N-0:L1-L1, L125-L126, L214-L247` unit=`N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing)` (ledger-line: DEEP-N-001.discovery.md:L138): Opening line says "16 sessions" while the session-table banner says "THE 12 SESSIONS" before external-steal N-13..N-16 appends; both are true if external track is extra, but total unit count is 16 not 12.

### Unit `N-1 — measurement spine as ONE subsystem` — from N-001 (35 records)

- **N-001/N1-E001** [E/STATUS] anchor=`N-1:L1-L3` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L148): N-1 is TIER 1 CRITICAL, lever the-selling-number, findings W-1+W-11+W-23, STRONG, ACTIVATION, XL, WAVE 1, KEYSTONE BLOCKING, depends none.
- **N-001/N1-E002** [E/CAP] anchor=`N-1:L5-L5` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L149): Premise: ~30 features across 6 letters terminate promotion at "ELAI-measured number"; measurement is currently scattered entries, not a first-class subsystem; N-1 declares the spine and ND-4 routes all numbers through it.
- **N-001/N1-E003** [E/CAP] anchor=`N-1:L8-L8` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L150): ONE measurement spine = (1) corpus, (2) ONE runner family, (3) ONE calibration SINK, (4) ONE outcome-label fold, (5) shipped BOOTSTRAP calibration pack.
- **N-001/N1-E004** [E/DATA] anchor=`N-1:L8-L8, L10-L10` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L151): Corpus: J-56 problem corpus with L-20 commit-mined supply + J-110 mutators as FEEDERS (not sibling benches).
- **N-001/N1-E005** [E/CAP] anchor=`N-1:L8-L8, L37-L37` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L152): ONE runner family: the K-15/I-27 shape already the pairwise convention across the ten benches.
- **N-001/N1-E006** [E/DATA] anchor=`N-1:L8-L8, L34-L35` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L153): ONE calibration SINK: W-5 ModelCapabilityProfile (I-51 models.toml); no router reads a knee from anywhere else.
- **N-001/N1-E007** [E/CAP] anchor=`N-1:L8-L8, L20-L20` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L154): ONE outcome-label fold: generalize J-93-F (deterministic non-LLM rule labels from SSB outcome stream) from rules-only to the six promotion loops (W-11).
- **N-001/N1-E008** [E/DATA] anchor=`N-1:L8-L8, L20-L20, L54-L54` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L155): BOOTSTRAP pack: B-42-style default artifacts, provenance=BOOTSTRAP, hard-superseded by local measurement via F-5/G-10 invalidation (W-23 cold-start fix).
- **N-001/N1-E009** [E/CAP] anchor=`N-1:L9-L9` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L156): Ten benches unified (bind to one runner + one sink, do NOT replace): G-6 SAFEDELTA, J-34 context-knee, K-15 reasoning-knee, J-107 longcot, L-10 hallucination-delta, K-18 case-reuse, I-38 QDELTA, I-27 convergence, L-20 supply, J-110 timeliness (+ D-19..D-25 bench/validation suites).
- **N-001/N1-E010** [E/LIMIT] anchor=`N-1:L10-L10, L41-L42` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L157): EXCLUDED: inventing any new bench (no 11th); any new corpus family (J-56/E-54/L-20 are the corpus); a second calibration store (ND-4 violation).
- **N-001/N1-E011** [E/STATUS] anchor=`N-1:L13-L13` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L158): PLAN-asserted: ten bench units exist as PLANNED across G/J/K/L/I and each reuses the runner pairwise.
- **N-001/N1-E012** [E/STATUS] anchor=`N-1:L14-L14` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L159): J-93-F supplies deterministic outcome labels to RULES only today.
- **N-001/N1-E013** [E/STATUS] anchor=`N-1:L15-L15` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L160): ModelCapabilityProfile already gains fill_degradation_curve (J-34), tool_call_adherence (J-51), reasoning-knee (K-15), longcot arm (J-107); I-23 flags unified single-impl calibration; N-1 is that binding at bench grain.
- **N-001/N1-E014** [E/DEP] anchor=`N-1:L16-L16, L53-L53` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L161): B-42 ships ~12 default cosmos artifacts as in-corpus precedent for a shipped default pack.
- **N-001/N1-E015** [E/STATUS] anchor=`N-1:L17-L17` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L162): D-30 names cold-start for ONE gate (UniCR N=500) and DEFERS it; the same hole sits under D-16 tau / J-34/K-15 knees / J-51 floors / L-11 OQ / 3F FP bounds / H-43 baselines (W-23).
- **N-001/N1-E016** [E/CFG] anchor=`N-1:L20-L20` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L163): Tier-1 SHIP NOW: spine INDEX (one runner + one sink + label fold binding) + bootstrap pack + ND-4 enforcement (bench writing knee outside profile FAILS); label fold generalized to six loops.
- **N-001/N1-E017** [E/DEFER] anchor=`N-1:L21-L21` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L164): Tier-2 DEFER: actual SAFEDELTA live number (G-6, gated behind F-2b keystone + RunPod); N-1 ships SUBSTRATE + deterministic fixture; live number stays G-6's deferred checkbox.
- **N-001/N1-E018** [E/INT] anchor=`N-1:L24-L24` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L165): crates/knowledge: corpus + runner surface; crates/orchestrator: ModelCapabilityProfile sink + J-93-F label fold generalization.
- **N-001/N1-E019** [E/SURFACE] [SURFACE] anchor=`N-1:L24-L24, L38-L39, L47-L47` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L166): CLI: `elai bench *` seams as the real production trigger.
- **N-001/N1-E020** [E/SURFACE] [SURFACE] anchor=`N-1:L25-L25, L45-L45` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L167): frontend_pair: MeasurementPanel tile fed by the spine index, OR N/A justified if G-5 cockpit already surfaces the profile (confirm at authoring).
- **N-001/N1-E021** [E/CAP] anchor=`N-1:L29-L29` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L168): produces: `crates/knowledge/src/measurement/spine.rs` — spine index (corpus + runner + sink binding) [PLAN-asserted path].
- **N-001/N1-E022** [E/CAP] anchor=`N-1:L30-L30` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L169): produces: `crates/orchestrator/src/measurement/bootstrap_pack.rs` — BOOTSTRAP default artifacts + supersession rule.
- **N-001/N1-E023** [E/CAP] anchor=`N-1:L31-L31` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L170): produces: generalized label fold (J-93-F extended to six loops).
- **N-001/N1-E024** [E/DEP] anchor=`N-1:L33-L37` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L171): consumes: knowledge J-56/E-54/L-20 corpus; ModelCapabilityProfile / I-51 models.toml; J-93-F (extend); K-15/I-27 runner shape.
- **N-001/N1-E025** [E/INT] [SURFACE] anchor=`N-1:L38-L40` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L172): reachable_from: CLI `elai bench <family>` -> runner -> corpus row + profile curve write; `#[cfg(test)]` NEVER satisfies R7.
- **N-001/N1-E026** [E/LIMIT] anchor=`N-1:L41-L44` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L173): stub_ban: NO second calibration store; NO new 11th bench; NO paper-constant knee claimed as ELAI-measured (bootstrap carries provenance=BOOTSTRAP, ID-5); NO .unwrap()/.expect(); blake3 only.
- **N-001/N1-E027** [E/EVENT] anchor=`LocallySuperseded` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L174): expect_ssb on trigger: corpus row + (on bootstrap replacement) `+calibration_bootstrap_superseded(Prefilled
- **N-001/N1-E028** [E/OBS] anchor=`N-1:L49-L49` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L175): expect_artifact: calibration curve written into ONE ModelCapabilityProfile; NEGATIVE: bench writing knee to any other store must FAIL ND-4 check.
- **N-001/N1-E029** [E/EVIDENCE] anchor=`N-1:L51-L54` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L176): Grounding: interlace W-1, W-11, W-23, TIER-1 synthesis; memory project_b42_shipped; reference arXiv IDs re-verify at authoring; ID-5 satisfied by BOOTSTRAP provenance label.
- **N-001/N1-E030** [E/DEP] anchor=`N-1:L56-L57` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L177): COMPOSES-WITH: J-56/E-54/L-20 corpus; K-15/I-27 runner; ModelCapabilityProfile (I-51); J-93-F; B-42 pack shape; F-5/G-10 invalidation; re-implements NONE (GD-1); ND-4 single sink for every other N-N and every J..M number.
- **N-001/N1-I001** [I/DATA] anchor=`N-1:L17-L17, N-1:L8-L8` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L178): Cold-start holes named under D-16 tau, J-34/K-15 knees, J-51 floors, L-11 OQ, 3F FP bounds, H-43 baselines are the implied bootstrap fill set even though the Tier-1 wiring only names "BOOTSTRAP default artifacts" generically.
- **N-001/N1-A001** [A/CFG] anchor=`N-1:L8-L8, L20-L20` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L179): The six promotion loops receiving generalized J-93-F labels are not enumerated as a closed named set.
- **N-001/N1-A002** [A/DATA] anchor=`N-1:L8-L8, L30-L30, L48-L48` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L180): Exact bootstrap artifact inventory (beyond B-42 shape) and supersession transition rules between Prefilled / LocallySuperseded / DriftInvalidated are not fully specified.
- **N-001/N1-A003** [A/SURFACE] [SURFACE] anchor=`N-1:L25-L25, L45-L45` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L181): MeasurementPanel vs G-5 N/A is deferred to authoring confirmation without a decision criterion.
- **N-001/N1-A004** [A/OBS] anchor=`N-1:L5-L5` unit=`N-1 — measurement spine as ONE subsystem` (ledger-line: DEEP-N-001.discovery.md:L182): "ELAI-measured number" fan-in "~30 features across 6 letters" is not listed individually.

### Unit `N-3 — generalized anti-selection-bias promotion primitives` — from N-001 (29 records)

- **N-001/N3-E001** [E/STATUS] anchor=`N-3:L1-L3` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L192): N-3 is TIER 2 HIGH, lever gated-against-its-own-search, W-30, STRONG, ACTIVATION, M-L, WAVE 3, depends N-1.
- **N-001/N3-E002** [E/CAP] anchor=`N-3:L5-L5` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L193): Premise: H-17 trial ledger + H-18 deflated-significance + H-19 reusable-holdout were finance-only; every promotion loop runs the same max-of-N-noisy-scores search and is upward-biased.
- **N-001/N3-E003** [E/CAP] anchor=`N-3:L8-L8` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L194): Lift three finance primitives into DOMAIN-GENERIC promotion primitives keyed by (candidate-family) instead of (finance-candidate), read via ID-6 ledger hook.
- **N-001/N3-E004** [E/DATA] anchor=`N-3:L9-L9` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L195): H-17 becomes persisted, agent-unresettable per-(candidate-family) trial ledger.
- **N-001/N3-E005** [E/CAP] anchor=`N-3:L10-L10, L22-L22, L42-L42` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L196): H-18 deflation/threshold correction (DSR/PBO/MinBTL, N from ledger, fail-closed on missing N) applied at every promotion gate.
- **N-001/N3-E006** [E/CAP] anchor=`N-3:L11-L11, L23-L23` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L197): H-19 reusable-holdout DP query budget (Thresholdout) for every reusable eval corpus (J-56/L-20/D-6).
- **N-001/N3-E007** [E/FLOW] anchor=`N-3:L12-L12` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L198): Loops hardened: 3F rule SHADOW->ACTIVE, J-57 prompt promotion, J-97 case-reuse fitness, B-23/H-45 configs, I-4/I-25 candidate ladders, L-11 OQ thresholds.
- **N-001/N3-E008** [E/DEP] anchor=`N-3:L12-L12, L17-L17` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L199): I-30/I-31 already route promotions through H-17 trial-N + H-18 deflation — proof the generalization is natural; N-3 extends to non-Track-17 loops.
- **N-001/N3-E009** [E/LIMIT] anchor=`N-3:L13-L13, L40-L42` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L200): EXCLUDED: re-implementing DSR/PBO/MinBTL (re-KEY shipped H-18); any new promotion loop; a promotion that runs without the ledger (fail-closed missing N).
- **N-001/N3-E010** [E/STATUS] anchor=`N-3:L16-L16` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L201): H-17/H-18/H-19 shipped/planned FINANCE-ONLY (TB-1); only H-47 partially reuses H-19.
- **N-001/N3-E011** [E/DEP] anchor=`N-3:L17-L17` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L202): ID-6 = "learned artifacts promote through the ledger".
- **N-001/N3-E012** [E/STATUS] anchor=`N-3:L18-L18` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L203): ~8 promotion loops each promote on max-of-N without trial-count deflation today.
- **N-001/N3-E013** [E/DEP] anchor=`N-3:L19-L19, L35-L35` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L204): N-1 supplies the outcome labels the deflation corrects against (did the promoted artifact actually help, per the journal).
- **N-001/N3-E014** [E/CFG] anchor=`N-3:L22-L22` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L205): Tier-1 SHIP NOW: re-key H-17 ledger + H-18 deflation to (candidate-family) and wire 3F rule + J-57 prompt + J-97 case gates to read it; fail-closed on missing N.
- **N-001/N3-E015** [E/DEFER] anchor=`N-3:L23-L23` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L206): Tier-2 DEFER: full Thresholdout DP budget on every corpus (start with ledger+deflation; H-19 holdout budget is second wave, needs N-1 corpus stable first).
- **N-001/N3-E016** [E/INT] anchor=`N-3:L26-L26` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L207): Integration: crates/orchestrator promotion sites (rules lifecycle, prompt/case/config archives); ID-6 ledger hook.
- **N-001/N3-E017** [E/SURFACE] [SURFACE] anchor=`N-3:L27-L27, L44-L44` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L208): frontend_pair: a deflation column on the existing promotion drawers, OR N/A justified.
- **N-001/N3-E018** [E/CAP] anchor=`N-3:L30-L32` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L209): produces: (candidate-family)-keyed trial ledger + deflation reader (re-keyed H-17/H-18); promotion-gate wiring at the ~8 sites.
- **N-001/N3-E019** [E/DEP] anchor=`N-3:L33-L35` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L210): consumes: H-17 / H-18 DSR-PBO-MinBTL / H-19 Thresholdout (re-key, do NOT re-author); ID-6; N-1 outcome labels.
- **N-001/N3-E020** [E/INT] anchor=`N-3:L36-L38` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L211): reachable_from: a real SHADOW->ACTIVE promotion transition (rules/prompt/case gate); test-only never R7.
- **N-001/N3-E021** [E/LIMIT] anchor=`N-3:L39-L43` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L212): stub_ban: NO promotion that flips ACTIVE ignoring trial count; NO re-implementation of DSR/PBO/MinBTL; NO silent pass on missing N; NO .unwrap()/.expect().
- **N-001/N3-E022** [E/EVENT] anchor=`DeflationBlocked` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L213): trigger: real SHADOW->ACTIVE promotion with N below threshold; expect_ssb `+promotion_deflated(TrialCountApplied
- **N-001/N3-E023** [E/FAIL] anchor=`N-3:L48-L48` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L214): expect_artifact: promotion is HELD; NEGATIVE: promotion that ignores trial count and flips ACTIVE must FAIL the gate.
- **N-001/N3-E024** [E/EVIDENCE] anchor=`N-3:L50-L52` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L215): Grounding: interlace W-30 TIER-2 item 3; W-30 landing site ID-6; H-plan Track 7 / Track 17; memory project_hplan_track7_authored.
- **N-001/N3-E025** [E/DEP] anchor=`N-3:L54-L55` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L216): COMPOSES-WITH: H-17/H-18/H-19 re-keyed, ID-6, N-1 labels, ~8 promotion loops as consumers; re-implements NONE.
- **N-001/N3-I001** [I/FLOW] anchor=`N-3:L12-L12, L22-L22, L32-L32` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L217): Tier-1 names only three wire sites (3F, J-57, J-97) while the loop list is eight; the remaining five are consumers that must eventually read the same re-keyed ledger even if not in the first wire set.
- **N-001/N3-A001** [A/CFG] anchor=`N-3:L10-L10, L22-L22, L46-L46` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L218): Threshold for "N below threshold", exact candidate-family key schema, and DSR/PBO/MinBTL parameter values are not stated.
- **N-001/N3-A002** [A/DEP] anchor=`N-3:L16-L16, L23-L23` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L219): H-47 partial H-19 reuse migration path into the generalized Thresholdout budget is not specified.
- **N-001/N3-A003** [A/CFG] anchor=`N-3:L11-L11, L23-L23` unit=`N-3 — generalized anti-selection-bias promotion primitives` (ledger-line: DEEP-N-001.discovery.md:L220): DP accounting parameters for H-19 Thresholdout budget (epsilon, query accounting, per-corpus limits) are absent.

### Unit `N-4 — one Claim/Belief object + receipt memo` — from N-001 (34 records)

- **N-001/N4-E001** [E/STATUS] anchor=`N-4:L1-L3` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L230): N-4 is TIER 2 HIGH, lever cheaper-reasoning-on-your-repo, W-2+W-4+W-22+W-36, STRONG, BUILD-on-shipped, L, WAVE 3.
- **N-001/N4-E002** [E/CAP] anchor=`N-4:L5-L5` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L231): Premise: four typed objects all mean "statement the system may or may not trust with evidence"; nothing lets a recurring claim reuse prior receipt, so every session re-pays checker/resample cost.
- **N-001/N4-E003** [E/DATA] anchor=`N-4:L8-L8` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L232): ONE canonical Claim/Belief: L-1 ClaimSet{ClaimKind,span,refs} ARE J-83 BeliefState entries{claim,receipt,abstain,DKG-provenance} (L-8 already threads — finish it).
- **N-001/N4-E004** [E/DATA] anchor=`N-4:L8-L8` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L233): Receipt attached at J-83 gate IS the evidence L-3 round-trips.
- **N-001/N4-E005** [E/DATA] anchor=`N-4:L8-L8` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L234): K-12 critical_atoms must_preserve = has-receipt.
- **N-001/N4-E006** [E/DATA] anchor=`N-4:L8-L8` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L235): GRC grounding tags "type:reference" join the one object; J-21 re-derives the SAME object against commit+AST.
- **N-001/N4-E007** [E/FLOW] anchor=`N-4:L9-L9` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L236): RECEIPT MEMO: before expensive checker/resample (D-11 registry / F-4 tests / L-5 do-tests / L-4 resampling N=5), look up claim-identity/trigger_signature in J-96 case graph.
- **N-001/N4-E008** [E/FLOW] anchor=`N-4:L9-L9` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L237): HIT re-validates cheaply via J-21 commit+AST identity instead of re-running the intervention; MISS mints a new node.
- **N-001/N4-E009** [E/CAP] anchor=`N-4:L9-L9, L15-L15` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L238): Generalizes C-30 OracleAnswerCache (query_hash,project_hash)->answer,trust-TTL into the receipt memo.
- **N-001/N4-E010** [E/RECOVERY] anchor=`N-4:L9-L9, L16-L16` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L239): Memo invalidated by F-5 staleness cascade.
- **N-001/N4-E011** [E/INT] anchor=`N-4:L9-L9, L36-L36` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L240): Consulted by BOTH J-83 reasoning receipt-gate AND L-2 CGG first (W-36).
- **N-001/N4-E012** [E/OBS] anchor=`N-4:L9-L9` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L241): Biggest win on checker-poor local 7B lane; "gets cheaper on YOUR repo" (K-18 moat #2) extended to REASONING.
- **N-001/N4-E013** [E/LIMIT] anchor=`N-4:L10-L10, L38-L39` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L242): EXCLUDED: re-authoring C-30 (generalize); re-designing L-8 carrier (align K-12 + GRC to it); a fifth claim type.
- **N-001/N4-E014** [E/STATUS] anchor=`N-4:L13-L13` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L243): L-1, J-83, K-12, GRC tags exist as distinct typed objects; L-8 threads L-1 over J-83.
- **N-001/N4-E015** [E/DATA] anchor=`N-4:L14-L14` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L244): J-96 mints case nodes on F-4 proof with trigger_signature + blast cone + proof receipt; J-97 is demonstration-injection memory (not yet a receipt cache).
- **N-001/N4-E016** [E/DEP] anchor=`N-4:L16-L16` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L245): J-21 memory validation re-derives against commit+AST; F-5 staleness cascade exists.
- **N-001/N4-E017** [E/DEP] anchor=`N-4:L17-L17` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L246): I-8 dead-end AST hashing already caches "what FAILED" (negative-space twin); N-4 adds "what was PROVEN".
- **N-001/N4-E018** [E/CFG] anchor=`N-4:L20-L20` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L247): Tier-1 SHIP NOW: align K-12 + GRC tags onto L-8/J-83 carrier + memo lookup-before-checker at J-83 gate, re-validated via J-21, invalidated via F-5.
- **N-001/N4-E019** [E/DEFER] anchor=`N-4:L21-L21` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L248): Tier-2 DEFER: L-2 CGG consulting the same memo (W-36) — after J-83 side proves identity+invalidation sound.
- **N-001/N4-E020** [E/INT] anchor=`N-4:L24-L24` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L249): Integration: crates/orchestrator ssb::payloads BeliefState + J-83 gate + L-2 CGG call sites; C-30 cache generalization.
- **N-001/N4-E021** [E/SURFACE] [SURFACE] anchor=`N-4:L25-L25, L42-L42` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L250): frontend_pair: existing belief/claim drawer gains a memo-hit indicator, OR N/A justified.
- **N-001/N4-E022** [E/CAP] anchor=`N-4:L28-L30` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L251): produces: aligned canonical Claim/Belief object; receipt-memo lookup (generalized C-30, keyed by claim-identity/trigger_signature).
- **N-001/N4-E023** [E/DEP] anchor=`N-4:L31-L33` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L252): consumes: L-1 / J-83 (L-8) / K-12 / GRC; J-96 / C-30 / J-21 / F-5.
- **N-001/N4-E024** [E/INT] anchor=`N-4:L34-L36` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L253): reachable_from: J-83 reasoning receipt-gate on a live task (PRODUCTION); L-2 CGG predictor path (Tier-2 PRODUCTION).
- **N-001/N4-E025** [E/LIMIT] anchor=`N-4:L37-L41` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L254): stub_ban: NO fifth claim type; NO re-implementation of C-30; NO memo Hit returned on F-5-invalidated (changed) code; NO .unwrap()/.expect(); blake3 for claim-identity hashing.
- **N-001/N4-E026** [E/EVENT] anchor=`N-4:L44-L45` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L255): trigger: assert a recurring claim about UNCHANGED code twice on a live task; first: checker runs + J-96 node minted; second: `+receipt_memo(Hit)` and checker skipped.
- **N-001/N4-E027** [E/OBS] anchor=`N-4:L46-L46` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L256): expect_artifact: second pass measurably cheaper (no checker call); NEGATIVE: claim on CHANGED (F-5-invalidated) code returning stale Hit must FAIL.
- **N-001/N4-E028** [E/EVENT] anchor=`N-4:L45-L45; cross-ref N-0:L268-L269` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L257): N-0 schema also names receipt_memo kinds Miss / Revalidated (in addition to Hit observed in N-4 acceptance).
- **N-001/N4-E029** [E/EVIDENCE] anchor=`N-4:L48-L50` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L258): Grounding: interlace W-2, W-4, W-22, W-36, TIER-2 item 4; L downgrade note (L-8 half-plans W-2); memory project_b40_shipped.
- **N-001/N4-E030** [E/DEP] anchor=`N-4:L52-L53` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L259): COMPOSES-WITH: L-1, J-83 (L-8), K-12, GRC, J-96, C-30, J-21, F-5, L-2..L-6; re-implements NONE.
- **N-001/N4-I001** [I/DATA] anchor=`N-4:L9-L9, L30-L30, L41-L41` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L260): Claim-identity hashing with blake3 plus J-96 trigger_signature together form the memo key, generalizing C-30's (query_hash,project_hash) pair without defining the exact serialization.
- **N-001/N4-A001** [A/DATA] anchor=`N-4:L8-L8` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L261): Final unified schema fields for abstain and DKG-provenance after alignment are named in the BeliefState shape but not fully re-specified.
- **N-001/N4-A002** [A/CFG] anchor=`N-4:L9-L9, L15-L15` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L262): Cache TTL/eviction, "cheap" revalidation cost bounds, and eligibility rules for which checkers may be skipped are unspecified.
- **N-001/N4-A003** [A/EVENT] anchor=`N-4:L44-L46; N-0:L268-L269` unit=`N-4 — one Claim/Belief object + receipt memo` (ledger-line: DEEP-N-001.discovery.md:L263): Acceptance only shows Hit path; Miss and Revalidated kinds from the index schema lack Tier-1 fixtures here.

### Unit `N-9 — one counterfactual / ablation engine` — from N-001 (25 records)

- **N-001/N9-E001** [E/STATUS] anchor=`N-9:L1-L3` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L273): N-9 is TIER 3 MEDIUM-HIGH, lever dedupe-the-subtlest-measurement, W-20+W-27, STRONG, BUILD, M, WAVE 6, depends N-1.
- **N-001/N9-E002** [E/CAP] anchor=`N-9:L5-L5` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L274): Premise: same mechanism — re-run/re-decide with one factor removed or injected, score delta off journal outcomes — is specified EIGHT times at different targets.
- **N-001/N9-E003** [E/CAP] anchor=`N-9:L8-L8` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L275): ONE counterfactual/ablation engine over the SSB with TARGET as parameter (role / verifier / ranker / rule / memory-entry / case) plus eight THIN adapters (ND-5).
- **N-001/N9-E004** [E/DEP] anchor=`N-9:L9-L9` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L276): Attribution adapters: C-45 CANTANTE per-role credit (drop-a-role); I-28 DAG credit (winner vs highest-vetoed sibling); J-105 per-memory-entry causal ROI (counterfactual replay); J-109 AD-4 pre-write counterfactual replay.
- **N-001/N9-E005** [E/DEP] anchor=`N-9:L10-L10` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L277): Ablation adapters: E-6 iso-budget role audit (delete-a-role unless >1-sigma lift); E-10 critic ablations; H-40 verifier retention by decorrelation; J-75 RRF ablation (drop-a-ranker).
- **N-001/N9-E006** [E/DEP] anchor=`N-9:L11-L11, L17-L17, L32-L32` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L278): Natural consumer of N-1 outcome labels (W-11); delta scored against same ground-truth labels.
- **N-001/N9-E007** [E/LIMIT] anchor=`N-9:L12-L12, L37-L38` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L279): EXCLUDED: re-implementing replay eight times; a ninth attribution target; an adapter carrying business logic (core does replay+scoring; adapter only names target).
- **N-001/N9-E008** [E/STATUS] anchor=`N-9:L15-L16` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L280): C-45/I-28/J-105/J-109 + E-6/E-10/H-40/J-75 all exist as independent PLANNED units; all are drop-one/inject-one over same journal outcomes; K-18 reuse-ON/OFF arms same shape.
- **N-001/N9-E009** [E/DEP] anchor=`N-9:L17-L17` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L281): N-3's deflation applies when the ablation feeds a promotion.
- **N-001/N9-E010** [E/CFG] anchor=`N-9:L20-L20` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L282): Tier-1 SHIP NOW: engine core (drop-one/inject-one replay + delta-scoring over SSB) + 3-4 adapters (role, verifier, ranker) proving parameterization.
- **N-001/N9-E011** [E/DEFER] anchor=`N-9:L21-L21` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L283): Tier-2 DEFER: memory/case/DAG-credit adapters (land as those subsystems stabilize; core shipped and proven first).
- **N-001/N9-E012** [E/INT] anchor=`N-9:L24-L24` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L284): Integration: crates/orchestrator counterfactual engine + the eight call sites as adapters.
- **N-001/N9-E013** [E/SURFACE] [SURFACE] anchor=`N-9:L25-L25, L41-L41` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L285): frontend_pair: reuse existing attribution drawers, OR N/A justified.
- **N-001/N9-E014** [E/CAP] anchor=`N-9:L28-L29` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L286): produces: counterfactual/ablation engine core (target as parameter) + thin adapters.
- **N-001/N9-E015** [E/DEP] anchor=`N-9:L30-L32` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L287): consumes: C-45/I-28/J-105/J-109 + E-6/E-10/H-40/J-75 as adapters; N-1 outcome labels; SSB event stream.
- **N-001/N9-E016** [E/INT] anchor=`N-9:L33-L35` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L288): reachable_from: a live role/verifier/ranker ablation through the ONE engine; test-only never R7.
- **N-001/N9-E017** [E/LIMIT] anchor=`N-9:L36-L40` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L289): stub_ban: NO eight separate replay implementations; NO adapter business logic; NO delta scored off anything but N-1 labels / journal (GD-2); NO .unwrap()/.expect().
- **N-001/N9-E018** [E/EVENT] anchor=`Verifier` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L290): trigger: live role (or verifier, or ranker) ablation through the one engine; expect_ssb `+counterfactual_scored(Role
- **N-001/N9-E019** [E/OBS] anchor=`N-9:L45-L45` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L291): expect_artifact: delta computed by the core from journal labels; NEGATIVE: adapter that carries business logic fails ND-5.
- **N-001/N9-E020** [E/EVIDENCE] anchor=`N-9:L47-L49` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L292): Grounding: interlace W-20 (four counterfactual engines), W-27 (four ablation harnesses), TIER-3 item 9; N-1 labels; N-3 when promotion-feeding.
- **N-001/N9-E021** [E/DEP] anchor=`N-9:L51-L52` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L293): COMPOSES-WITH: C-45/I-28/J-105/J-109, E-6/E-10/H-40/J-75, N-1; re-implements NONE.
- **N-001/N9-I001** [I/DATA] anchor=`Verifier` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L294): Event Kind set (Role
- **N-001/N9-A001** [A/DET] anchor=`N-9:L5-L5, L8-L8, L20-L20` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L295): Replay isolation, seed control, delta metric definition, inject-one semantics, and allowable side effects are not stated.
- **N-001/N9-A002** [A/DEP] anchor=`N-9:L8-L10` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L296): Mapping from eight adapters to six target parameters is not one-to-one specified (e.g. which adapters share Role).
- **N-001/N9-A003** [A/CFG] anchor=`N-9:L20-L20` unit=`N-9 — one counterfactual / ablation engine` (ledger-line: DEEP-N-001.discovery.md:L297): "3-4 adapters" Tier-1 set names role/verifier/ranker but does not name which fourth if four are shipped.

### Unit `N-11 — journal-fold substrate` — from N-001 (29 records)

- **N-001/N11-E001** [E/STATUS] anchor=`N-11:L1-L3` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L307): N-11 is TIER 3 MEDIUM-HIGH, lever one-proof-five-surfaces, W-10, STRONG, BUILD-on-shipped, M-L, WAVE 7.
- **N-001/N11-E002** [E/CAP] anchor=`N-11:L5-L5` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L308): Premise: L-16 pure fold(log,budget,policy)->context-view is the right unifier; five OTHER surfaces re-derive-from-journal ad hoc each needing its own GD-2 audit.
- **N-001/N11-E003** [E/CAP] anchor=`N-11:L8-L8` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L309): Extend L-16 pure fold core beyond context: per-surface fold FUNCTIONS as data/plugins for five re-derive surfaces.
- **N-001/N11-E004** [E/SURFACE] [SURFACE] anchor=`N-11:L8-L8, L14-L14` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L310): Five surfaces: canvas altitudes (J-31/J-60/J-98/K-23), I-39 plan status, K-24 billing balance, M-3 fleet rollup, K-22 coverage.
- **N-001/N11-E005** [E/TEST] anchor=`N-11:L8-L8, L19-L19, L28-L28` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L311): ONE J-68 Kani correctness proof on the fold core then covers all five.
- **N-001/N11-E006** [E/RECOVERY] anchor=`N-11:L8-L8, L19-L19, L29-L29` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L312): Ships ND-2 RETIREMENT map for paths L-16 leaves live: recover_stubs end-of-life; declare H-49 CAS/undo-tree THE log representation (one store).
- **N-001/N11-E007** [E/DEP] anchor=`N-11:L9-L9` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L313): BUILD-on-shipped: L-16 already replaces compact_session, keeps mutate path one release as rollback, re-expresses E-25/H-6/3E/H-50 as fold STRATEGIES; L-18 collapses K-12+H-49 into projection corollaries.
- **N-001/N11-E008** [E/LIMIT] anchor=`N-11:L9-L9, L37-L37` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L314): N-11 residual ONLY: (a) five non-context surfaces adopt fold core, (b) one Kani proof, (c) recover_stubs/H-49 retirement. Do NOT re-author the fold (GD-1).
- **N-001/N11-E009** [E/LIMIT] anchor=`N-11:L10-L10, L40-L40` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L315): EXCLUDED: re-authoring L-16 fold; a sixth context core; retiring a path without user confirmation (ND-2).
- **N-001/N11-E010** [E/STATUS] anchor=`N-11:L13-L13` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L316): L-16 fold exists; CC-1..CC-4 collapse 4 context mechanisms into fold() parameters; L-18 collapses losslessness+time-travel into it.
- **N-001/N11-E011** [E/STATUS] [SURFACE] anchor=`N-11:L14-L14` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L317): Canvas altitudes "re-derive byte-identically from the journal"; I-39/K-24/M-3/K-22 each recomputable from journals.
- **N-001/N11-E012** [E/DEP] anchor=`N-11:L15-L15` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L318): J-68 Kani planned for fold core; H-49 CAS/undo-tree is the log-representation candidate.
- **N-001/N11-E013** [E/STATUS] anchor=`N-11:L16-L16` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L319): recover_stubs marked "legacy-only" needs explicit end-of-life (W-13 residual).
- **N-001/N11-E014** [E/CFG] [SURFACE] anchor=`N-11:L19-L19` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L320): Tier-1 SHIP NOW: fold core adopted by 2-3 of five surfaces (canvas + I-39 status) + J-68 Kani proof on core + recover_stubs/H-49 retirement map (user-confirmed).
- **N-001/N11-E015** [E/DEFER] [SURFACE] anchor=`N-11:L20-L20` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L321): Tier-2 DEFER: K-24 balance + M-3 rollup + K-22 coverage adopting the fold (land with those units).
- **N-001/N11-E016** [E/INT] anchor=`N-11:L23-L23` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L322): Integration: crates/orchestrator fold core (extend L-16) + five surface fold-fns; H-49 CAS as the log store.
- **N-001/N11-E017** [E/SURFACE] [SURFACE] anchor=`N-11:L24-L24, L41-L41` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L323): frontend_pair: canvas altitudes already consume folds — verify byte-identity, do not re-wire.
- **N-001/N11-E018** [E/CAP] anchor=`N-11:L27-L29` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L324): produces: extended fold core (L-16 + per-surface fold-fns as data) + J-68 Kani proof; ND-2 retirement map (recover_stubs EOL; H-49 CAS as THE log representation).
- **N-001/N11-E019** [E/DEP] anchor=`N-11:L30-L32` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L325): consumes: L-16 fold (EXTEND), J-68 Kani, H-49 CAS/undo-tree; five re-derive surfaces.
- **N-001/N11-E020** [E/INT] anchor=`N-11:L33-L35` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L326): reachable_from: two of five surfaces re-derived through ONE fold core on a live session; test-only never R7.
- **N-001/N11-E021** [E/LIMIT] anchor=`N-11:L36-L40` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L327): stub_ban: NO re-authoring L-16; NO sixth context core; NO surface still re-deriving off own ad-hoc path (conformance FAIL); NO path retirement without user confirmation.
- **N-001/N11-E022** [E/EVENT] anchor=`N-11:L43-L44` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L328): trigger: re-derive two of five surfaces through one fold core with pinned ELAI_SEED; expect_ssb: reuse L-16 fold events; no new type.
- **N-001/N11-E023** [E/DET] anchor=`N-11:L45-L45` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L329): expect_artifact: byte-identical re-derivation across the two surfaces + retirement switch removes old path; NEGATIVE: surface still on ad-hoc path FAILS conformance.
- **N-001/N11-E024** [E/EVIDENCE] anchor=`N-11:L47-L49` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L330): Grounding: interlace W-10 + L downgrade note; TIER-3 item 11; memory project_hplan_track8_authored (H-49); L-16/L-18.
- **N-001/N11-E025** [E/DEP] anchor=`N-11:L51-L52` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L331): COMPOSES-WITH: L-16 (extend), J-68, H-49, five surfaces; re-implements NONE; one core one proof five surfaces prevents unifier becoming sixth layer.
- **N-001/N11-I001** [I/SURFACE] [SURFACE] anchor=`N-11:L8-L8, L14-L14` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L332): "Canvas altitudes" is one surface fed by four unit IDs (J-31/J-60/J-98/K-23), counting as one of the five surfaces rather than four separate fold targets.
- **N-001/N11-A001** [A/DATA] anchor=`N-11:L8-L8, L19-L19` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L333): Plugin/fold-fn schema, per-surface budget/policy parameters, and exact Kani proof property are not defined.
- **N-001/N11-A002** [A/RECOVERY] anchor=`N-11:L8-L8, L19-L19` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L334): Retirement timing, migration of existing recover_stubs consumers, and concurrent dual-path behavior during the one-release LEGACY window are not specified beyond ND-2.
- **N-001/N11-A003** [A/CFG] [SURFACE] anchor=`N-11:L19-L19` unit=`N-11 — journal-fold substrate` (ledger-line: DEEP-N-001.discovery.md:L335): "2-3 of the five surfaces" names canvas + I-39 but leaves the optional third Tier-1 surface unnamed.

### Unit `N-12 — consistency + dedupe bundle` — from N-001 (33 records)

- **N-001/N12-E001** [E/STATUS] anchor=`N-12:L1-L3` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L345): N-12 is TIER 4 MEDIUM, lever consistency-and-free-training-data, W-8+W-9+W-12+W-16+W-21+W-25+W-26+W-28, STRONG, DELETE, M, WAVE 8.
- **N-001/N12-E002** [E/CAP] anchor=`N-12:L5-L5` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L346): Premise: eight independent same-concern fragmentations, each low-cost to collapse, each in disjoint files (parallel-safe, independently shippable); ND-1 sweep N-2 will keep clean.
- **N-001/N12-E003** [E/CAP] anchor=`N-12:L8-L8` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L347): Sub-slice (a) W-8+W-26: ONE FailureClass enum in SSB collapsing failure_taxonomy.py 13-cat + crates/tools 9-cat + E-6 MAST 14-mode + E-29 MAST DUP + C-10 StagnationPattern 4-variant.
- **N-001/N12-E004** [E/DATA] anchor=`N-12:L8-L8` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L348): FailureClass is shared by auditor attribution / anti-spiral / MAST supervisor (J-93-A) / eval corpus rows / case-graph edges; classified once at audit time IS the corpus row type IS the case edge IS the stagnation signal.
- **N-001/N12-E005** [E/OBS] anchor=`N-12:L8-L8` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L349): FailureClass collapse frees training data for J-25/I-12.
- **N-001/N12-E006** [E/CAP] anchor=`N-12:L9-L9` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L350): Sub-slice (b) W-9: ONE AbstainDecision spine spanning D-16 conformal + I-8 voluntary abort + D-17 SLA + L-7 policy + J-73 retrieval + J-110 timeliness.
- **N-001/N12-E007** [E/FLOW] anchor=`N-12:L9-L9` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L351): Abstain composition rule: upstream retrieval-abstain TAINTS downstream steps as answer-without-evidence -> L-2 risk floor.
- **N-001/N12-E008** [E/DEP] anchor=`N-12:L9-L9` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L352): L-7 already composes D-16+I-8+D-17; RD-1 mandates read-never-re-derive — extend, don't re-author.
- **N-001/N12-E009** [E/CAP] anchor=`N-12:L10-L10` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L353): Sub-slice (c) W-16: ONE ProjectClass registry — B-40 ProjectClass 9-variant (first in flow) consumed by B-42 cosmos project_classes / E-50 SaaS archetypes / E-12 framework manifests / L-14 build defaults.
- **N-001/N12-E010** [E/CAP] anchor=`N-12:L11-L11` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L354): Sub-slice (d) W-21: route C-47 sled CaseBase + C-49 markdown ExperienceCorpus writes through J-109 admission pipeline (closes unguarded learned-artifact write path; J-109 guards E-5/I-12/J-97 but not C-47/C-49 today).
- **N-001/N12-E011** [E/CAP] anchor=`N-12:L12-L12` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L355): Sub-slice (e) W-25: ONE drift-primitive library — D-4 tripwire (KL/delta-rho/IQR/perplexity/DDM) + G-10 model-drift + H-30 staleness + F-5 cascade + B-22b goal-drift share ONE windowed-divergence/Page-Hinkley/DDM core; keep distinct policies.
- **N-001/N12-E012** [E/TEST] anchor=`N-12:L13-L13` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L356): Sub-slice (f) W-12: FSM three-model conformance test — 3I DST FsmReferenceModel == J-68 ElaiFSM.tla == the code transition table (table already data-shaped; make it single source all three consume).
- **N-001/N12-E013** [E/CAP] anchor=`N-12:L14-L14` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L357): Sub-slice (g) W-28: ONE LSP daemon pool — E-52 LSP-as-verifier + J-18 LSP retrieval channel + J-52 per-edit LSP block-feed share one per-language health-managed pool (like B-37 runtime adapters).
- **N-001/N12-E014** [E/OBS] anchor=`N-12:L14-L14` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L358): LSP pool saves RAM + startup latency + three divergent process managers.
- **N-001/N12-E015** [E/LIMIT] anchor=`N-12:L15-L15, L42-L42` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L359): EXCLUDED: adding a new type for ANY of these concerns (collapse to existing canonical, ND-1); a ninth collapse.
- **N-001/N12-E016** [E/STATUS] anchor=`N-12:L17-L19` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L360): Existing ground truth PLAN-asserted for all named taxonomies, abstention family, ProjectClass homes, experience stores, drift detectors, FSM models, LSP integrations.
- **N-001/N12-E017** [E/CFG] anchor=`N-12:L22-L22` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L361): Tier-1 SHIP NOW: cheap high-consistency collapses — (a) FailureClass enum, (c) ProjectClass registry, (f) FSM conformance test; each independently shippable.
- **N-001/N12-E018** [E/DEFER] anchor=`N-12:L23-L23` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L362): Tier-2 DEFER: (b) AbstainDecision spine, (d) experience-store admission routing, (e) drift-primitive library, (g) LSP daemon pool — larger refactors after Tier-1 three.
- **N-001/N12-E019** [E/INT] anchor=`N-12:L26-L26` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L363): Integration: respective crates (orchestrator FailureClass/AbstainDecision/ProjectClass/drift; security admission; tools LSP pool); each sub-slice edits its own files — parallel-safe.
- **N-001/N12-E020** [E/SURFACE] [SURFACE] anchor=`N-12:L27-L27, L45-L45` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L364): frontend_pair: N/A for most; a FailureClass drawer if surfaced; justified per sub-slice.
- **N-001/N12-E021** [E/CAP] anchor=`N-12:L30-L33` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L365): produces: one FailureClass enum / one AbstainDecision row + composition rule / one ProjectClass registry / one drift-primitive lib / FSM three-model conformance test / one LSP daemon pool / J-109 admission routing for C-47/C-49.
- **N-001/N12-E022** [E/DEP] anchor=`N-12:L34-L37` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L366): consumes: failure_taxonomy.py + 9-cat + E-6/E-29 MAST + C-10; D-16/I-8/D-17/L-7/J-73/J-110; B-40/B-42/E-50/E-12/L-14; C-47/C-49 + J-109; D-4/G-10/H-30/F-5/B-22b; 3I/J-68/code-table; E-52/J-18/J-52.
- **N-001/N12-E023** [E/INT] anchor=`N-12:L38-L40` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L367): reachable_from: each surviving canonical impl called from its >=2 former call sites (PRODUCTION per sub-slice); test-only never R7.
- **N-001/N12-E024** [E/LIMIT] anchor=`N-12:L41-L44` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L368): stub_ban: NO new type for any collapsed concern; NO second parallel implementation surviving collapse; NO experience-store write bypassing J-109 admission.
- **N-001/N12-E025** [E/EVENT] anchor=`N-12:L48-L48` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L369): expect_ssb: no new action_type (dedupe removes emitters; surviving canonical type keeps its row).
- **N-001/N12-E026** [E/OBS] anchor=`N-12:L49-L49` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L370): expect_artifact: one canonical row/type per concern; NEGATIVE: second parallel implementation re-appearing fails N-2 duplicate-concept check.
- **N-001/N12-E027** [E/EVIDENCE] anchor=`N-12:L51-L53` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L371): Grounding: interlace W-8/W-26, W-9, W-16, W-21, W-25, W-12, W-28, TIER-4 item 12; E-6/E-29 MAST dup demonstrates summary-per-report duplication that N-2 stops.
- **N-001/N12-E028** [E/DEP] anchor=`N-12:L55-L56` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L372): COMPOSES-WITH: every unit named per sub-slice (dedupe to first-in-flow, adapters for rest), J-109, N-2 duplicate-concept check; re-implements NONE.
- **N-001/N12-I001** [I/CAP] anchor=`N-12:L5-L5, L7-L14` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L373): The source says "eight independent same-concern collapses" while labels (a)..(g) are seven; (a) merges W-8+W-26, yielding eight findings over seven sub-slices — count mismatch is structural, not a missing eighth product.
- **N-001/N12-A001** [A/DATA] anchor=`N-12:L8-L8` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L374): First-in-flow canonical enum mappings (13-cat / 9-cat / 14-mode / 4-variant reconciliation) and whether StagnationPattern remains a policy sub-type are not defined.
- **N-001/N12-A002** [A/CFG] anchor=`N-12:L14-L14` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L375): LSP pool process isolation, language/workspace key, lifetime, crash recovery, and resource limits are absent.
- **N-001/N12-A003** [A/DEP] anchor=`N-12:L55-L56` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L376): Adapter boundaries for non-canonical call sites after collapse are named only as "adapters for the rest" without per-slice adapter specs.
- **N-001/N12-A004** [A/TEST] anchor=`N-12:L13-L13` unit=`N-12 — consistency + dedupe bundle` (ledger-line: DEEP-N-001.discovery.md:L377): FSM three-model conformance exact equality criterion (byte-level, semantic, or property suite) is not specified beyond "==".

## Ledger N-002

Source file: `docs/thefinalglance/feature-discovery/deep/DEEP-N-002.discovery.md`
Extracted: source=297, IX=18, SD=35, AC=11, subtotal=361

### Unit `N-2 — corpus-grain composition validator + M-1 one-registry rule` — from N-002 (34 records)

- **N-002/N2-E001** [E/STATUS] anchor=`N-2:L1-L3` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L37): N-2 is TIER 2 (HIGH), lever plans-that-talk, findings W-6+M-1+W-33+W-32, grounding STRONG, nature BUILD, effort L, WAVE 2, depends on N-1.
- **N-002/N2-E002** [E/FLOW] anchor=`N-2:L5-L5` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L38): The motivating defect is that feature-map EXTENDS/composes/consumes/gated-by edges are hand-maintained prose with zero machine check, so plans authored against summaries can silently duplicate or contradict.
- **N-002/N2-E003** [E/EVIDENCE] anchor=`N-2:L5-L5` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L39): J-SUMMARY_INDEX.md:2 self-declares the letter drifted from its own index (detail stopped at J-91); nothing validates the CROSS-LETTER edge set.
- **N-002/N2-E004** [E/CAP] [SURFACE] anchor=`N-2:L8-L8, L26-L26, L30-L32` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L40): Build `corpus_composition_check.py` as sibling of `wiring_contract_check.py`, parsing feature-map edges as DATA in ND-3 row format and joining them against plan files + `elai.map` symbols.
- **N-002/N2-E005** [E/FAIL] anchor=`N-2:L8-L9` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L41): FAIL condition (a) DANGLING EDGE: a "consumes X" naming no unit that exposes X.
- **N-002/N2-E006** [E/FAIL] anchor=`N-2:L10-L10` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L42): FAIL condition (b) ORPHAN PRODUCER: a unit with >=1 consumer in NO letter (the orphan-feature report).
- **N-002/N2-E007** [E/FAIL] anchor=`N-2:L11-L11` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L43): FAIL condition (c) DUPLICATE-CONCEPT COLLISION via name-similarity + shared-seam detection, making the M-1 one-registry-per-concern rule mechanical (would auto-catch four claim types / ten routers / E-6+E-29 MAST dup).
- **N-002/N2-E008** [E/OBS] anchor=`N-2:L12-L12, L22-L22, L31-L31, L48-L48` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L44): The checker EMITS the global unlock-DAG + fan-in ranking (W-32).
- **N-002/N2-E009** [E/DATA] anchor=`N-2:L12-L12, L17-L17, L35-L35` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L45): Summaries/status are derived mechanically by composing J-93-G (auto-derive the map's mechanical half) + I-39 (journal-derived status).
- **N-002/N2-E010** [E/OBS] anchor=`N-2:L12-L12, L18-L18, L22-L22, L36-L36` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L46): A SUMMARY_INDEX-vs-0_INDEX-vs-detail FRESHNESS check applies the `elai_map_freshness.py` pattern to the index layer (W-33).
- **N-002/N2-E011** [E/INT] anchor=`N-2:L12-L12, L26-L26, L37-L39` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L47): Runs at paper-to-plan phase 5 / planning-workflow authoring time the way `wiring_contract_check` runs at code time; also a CI invocation over `docs/thefinalglance/`.
- **N-002/N2-E012** [E/LIMIT] anchor=`N-2:L13-L13` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L48): EXCLUDED: any change to the runtime wiring gate (this is the PLAN-corpus twin, not a replacement).
- **N-002/N2-E013** [E/LIMIT] anchor=`N-2:L13-L13, L16-L16, L34-L34` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L49): EXCLUDED: re-implementing J-89 L8; extend the per-plan type-completeness check to corpus grain instead.
- **N-002/N2-E014** [E/DEP] anchor=`N-2:L15-L18` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L50): PLAN-asserted existing ground: J-89 L8 plan-DAG producer->consumer type-completeness PER PLAN; J-93-G map half; I-39 journal status; `.claude/tools/wiring_contract_check.py` + `elai_map_freshness.py` + `spec_link_check.py` (executor re-greps per W-7).
- **N-002/N2-E015** [E/EVIDENCE] anchor=`N-2:L19-L19` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L51): Feature-map OPEN CONFLICTS section had 7 unresolved edges; I-2/I-5/I-11/H-50 "PLANNED label was STALE" corrections prove drift is real, recurring, currently caught by luck.
- **N-002/N2-E016** [E/CFG] anchor=`N-2:L21-L22` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L52): SHIP NOW (Tier-1): the three FAIL checks + freshness check + unlock-DAG emit.
- **N-002/N2-E017** [E/FAIL] anchor=`N-2:L22-L22, L41-L41` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L53): Fail-closed: a corpus that finds ZERO edges exits nonzero (R1, ArchUnit failOnEmptyShould).
- **N-002/N2-E018** [E/DEFER] anchor=`N-2:L23-L23` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L54): DEFER (Tier-2): full name-similarity ML; start with deterministic shared-seam + token-overlap heuristic; fuzzy semantic-duplicate detector is a later refinement, flagged directional.
- **N-002/N2-E019** [E/DATA] anchor=`N-2:L26-L26, L32-L32, L43-L43` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L55): Integration also adds a small ND-3 edge-row emitter that the feature-map upsert writes.
- **N-002/N2-E020** [E/INT] anchor=`N-2:L26-L26, L37-L38` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L56): Wire into paper-to-plan phase-5 / planning-workflow PRE_GATE authoring seam (PRODUCTION authoring seam).
- **N-002/N2-E021** [E/SURFACE] [SURFACE] anchor=`N-2:L27-L27, L44-L44` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L57): frontend_pair: N/A — authoring-time CI gate, not a runtime surface; justified.
- **N-002/N2-E022** [E/CAP] [SURFACE] anchor=`N-2:L30-L32` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L58): produces_symbols: `.claude/tools/corpus_composition_check.py` (three FAIL checks + unlock-DAG + freshness) and the ND-3 edge-row format.
- **N-002/N2-E023** [E/FAIL] anchor=`N-2:L40-L41` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L59): stub_ban: NO exit-0 on a corpus with a dangling edge (fail-closed, R1).
- **N-002/N2-E024** [E/CFG] anchor=`N-2:L42-L42` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L60): stub_ban: NO opt-in flag that lets the check emit PASS on missing edges (R3).
- **N-002/N2-E025** [E/DATA] anchor=`N-2:L43-L43` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L61): stub_ban: NO prose-only edges (edges must be DATA rows, ND-3).
- **N-002/N2-E026** [E/SURFACE] [SURFACE] anchor=`N-2:L46-L46` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L62): CLI/command surface: `python .claude/tools/corpus_composition_check.py --map docs/thefinalglance/ELAI-FEATURE-MAP.md`.
- **N-002/N2-E027** [E/EVENT] anchor=`OrphanProducer\` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L63): expect_ssb: `+composition_edge_checked(DanglingEdge\
- **N-002/N2-E028** [E/TEST] anchor=`N-2:L48-L48` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L64): expect_artifact: nonzero exit + offending edge named on a seeded dangling edge; unlock-DAG artifact on a clean run.
- **N-002/N2-E029** [E/TEST] anchor=`N-2:L48-L48` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L65): NEGATIVE: a clean corpus with a hand-inserted dangling edge that EXITS 0 is a FAIL.
- **N-002/N2-E030** [E/DEP] anchor=`N-2:L50-L52` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L66): Grounding: interlace W-6/M-1/W-33/W-32, TIER-2 item 2; wiring-enforcement R6 lifted to plan grain; R1/R3/R8.
- **N-002/N2-E031** [E/DEP] anchor=`N-2:L54-L55` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L67): COMPOSES-WITH J-89 L8, J-93-G, I-39, wiring_contract_check + elai_map_freshness patterns; re-implements NONE (GD-1). This unit BUILDS the ND-1/ND-3 checker the whole N-plan (and every future letter) is judged against.
- **N-002/N2-A001** [A/FAIL] anchor=`N-2:L10-L10` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L68): "orphan producer (a unit with >=1 consumer in NO letter)" is grammatically ambiguous; likely means a producer with no consumer in any letter, but the source does not rephrase it.
- **N-002/N2-A002** [A/DATA] anchor=`N-2:L11-L12, L32-L32, L47-L48` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L69): Similarity thresholds, collision adjudication, ND-3 edge schema fields, unlock-DAG filename/path, and full SSB payload fields beyond Kind names are not specified.
- **N-002/N2-A003** [A/RECOVERY] anchor=`N-2:L5-L5, L12-L12, L19-L19` unit=`N-2 — corpus-grain composition validator + M-1 one-registry rule` (ledger-line: DEEP-N-002.discovery.md:L70): J drift smoking-gun is cited, but exact repair/migration behavior for stale indexes is not stated.

### Unit `N-5 — policy / intensity arbitration plane` — from N-002 (30 records)

- **N-002/N5-E001** [E/STATUS] anchor=`N-5:L1-L3` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L80): N-5 is TIER 2 (HIGH), lever explainable-non-emergent-routing, findings W-3+W-17+W-35+W-14, grounding STRONG, nature BUILD, effort L, WAVE 4, depends on N-1.
- **N-002/N5-E002** [E/FLOW] anchor=`N-5:L5-L5` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L81): Premise: ten deterministic routers with no arbitration plane; pairwise conflicts resolve implicitly by call order inside `task_runner` (the "dispatch order is prose" failure J-92 flags for the FSM, reproduced at the policy layer). Plus six verification-intensity deciders on uncoordinated axes.
- **N-002/N5-E003** [E/CAP] anchor=`N-5:L8-L9, L20-L20, L29-L29, L40-L40` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L82): Binding (1): a declared PRECEDENCE spec as DATA (not code, R8-frozen) with order safety-gates > cost-ceiling > abstention > escalation > modality > serving-tier.
- **N-002/N5-E004** [E/DEP] anchor=`N-5:L9-L9, L14-L14, L32-L32` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L83): Precedence spans ten routers: D-10 grounding-mode / J-84 effort / J-55 modality / H-7 role / E-20 local-cloud / J-48 serving / J-54 free-pool / L-6 curiosity / J-32 front-door / J-73 should-retrieve.
- **N-002/N5-E005** [E/EVENT] anchor=`N-5:L9-L9, L20-L20, L29-L29` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L84): ONE composite `RouteDecision` journal row names every router consulted + which won + why.
- **N-002/N5-E006** [E/OBS] anchor=`N-5:L9-L9, L17-L17, L33-L33` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L85): J-92's FSM-substate structural-enforcement is extended one level up so J-66 monitor can watch routing order as a checkable invariant.
- **N-002/N5-E007** [E/CAP] anchor=`N-5:L10-L10, L20-L20, L30-L30` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L86): Binding (2): ONE intensity resolver over (risk x model-tier x level x task-difficulty x delegation), absorbing six deciders as AXES of one frozen table.
- **N-002/N5-E008** [E/DEP] anchor=`N-5:L10-L10, L16-L16, L32-L32` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L87): Six intensity deciders as axes: B-41 (level) / L-21 (model) / J-84 (effort) / C-46 (delegation) / K-5 (gate-risk) / B-28 ModeSelector (as J-55's edit-format leg, W-14).
- **N-002/N5-E009** [E/OBS] anchor=`N-5:L10-L10, L33-L33` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L88): K-4 gate-tax then measures ONE policy, not the composition of six.
- **N-002/N5-E010** [E/LIMIT] anchor=`N-5:L11-L11, L38-L39` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L89): EXCLUDED: re-implementing any router or decider (arbitrate/resolve them); a new router; changing any router's internal deterministic logic (TD-3 stays — this orders their OUTPUTS).
- **N-002/N5-E011** [E/DEP] anchor=`N-5:L13-L14` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L90): PLAN-asserted: ten routers + six intensity deciders exist as PLANNED/shipped, each deterministic (TD-3), each with its own SSB row.
- **N-002/N5-E012** [E/EVIDENCE] anchor=`N-5:L15-L15` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L91): E-20 declares itself "orthogonal to D-10" and E-31 adds EscalationPolicy — letters answer multiplicity with prose orthogonality, not an arbitration spec.
- **N-002/N5-E013** [E/DEP] anchor=`N-5:L16-L16` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L92): B-28 Focus Tunnel ModeSelector routes FIM/DIFF/STRUCTURED; J-55 ModalityRouter routes CodeAct/constrained-call/SEARCH-REPLACE — two mode deciders on the same emission path with undefined interaction (W-14).
- **N-002/N5-E014** [E/CFG] anchor=`N-5:L19-L20` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L93): SHIP NOW (Tier-1): frozen precedence table + composite RouteDecision row + intensity resolver table with six deciders as axes; in-process, deterministic.
- **N-002/N5-E015** [E/DEFER] anchor=`N-5:L21-L21` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L94): DEFER (Tier-2): J-66 monitor watching routing-order as an invariant (lands after RouteDecision row is emitting).
- **N-002/N5-E016** [E/INT] anchor=`N-5:L23-L24, L34-L35` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L95): Integration at `crates/orchestrator` `task_runner` routing seam + frozen precedence/intensity table (R8, in `.claude/gate-manifest`).
- **N-002/N5-E017** [E/SURFACE] [SURFACE] anchor=`N-5:L25-L25, L42-L42` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L96): frontend_pair: a RouteDecision explain-row on the canvas/RRA (W-7/N-7 render consumer).
- **N-002/N5-E018** [E/INT] anchor=`N-5:L34-L36` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L97): reachable_from: task_runner on a task that fires >=2 conflicting routers [PRODUCTION]; a `#[cfg(test)]` caller NEVER satisfies this (R7).
- **N-002/N5-E019** [E/FAIL] anchor=`N-5:L37-L38` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L98): stub_ban: NO routing decision with >=2 routers fired and no arbitration row.
- **N-002/N5-E020** [E/CFG] anchor=`N-5:L40-L40` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L99): stub_ban: NO model-settable precedence (frozen table, R8).
- **N-002/N5-E021** [E/LIMIT] anchor=`N-5:L41-L41` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L100): stub_ban: NO `.unwrap()`/`.expect()` on prod paths.
- **N-002/N5-E022** [E/TEST] anchor=`N-5:L44-L44` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L101): trigger: a live task where two routers disagree (e.g. L-6 retrieve vs J-73 abstain).
- **N-002/N5-E023** [E/EVENT] anchor=`N-5:L45-L45` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L102): expect_ssb: ONE `+route_arbitrated` row naming both routers + winner + precedence-rank.
- **N-002/N5-E024** [E/TEST] anchor=`N-5:L46-L46` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L103): expect_artifact: winning route taken per frozen precedence; NEGATIVE: decision with >=2 routers fired and NO arbitration row must FAIL (R10 seam gate).
- **N-002/N5-E025** [E/DEP] anchor=`N-5:L48-L50` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L104): Grounding: W-3/W-17/W-35/W-14, TIER-2 item 5; wiring-enforcement R10 (unit-green != integrated-green).
- **N-002/N5-E026** [E/DEP] anchor=`N-5:L52-L53` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L105): COMPOSES-WITH the ten routers + six intensity deciders (as axes), J-92 pattern, J-66 monitor, K-4 gate-tax, B-28 (edit-format leg of J-55); re-implements NONE (GD-1).
- **N-002/N5-A001** [A/DET] anchor=`N-5:L8-L10, L20-L20` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L106): Tie-breaking within a precedence tier, policy versioning, and how task-difficulty is measured are not specified.
- **N-002/N5-A002** [A/DATA] anchor=`N-5:L10-L10, L16-L16` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L107): The intensity resolver parenthesized axes omit edit-format even though B-28 is explicitly incorporated as J-55's edit-format leg.
- **N-002/N5-A003** [A/EVENT] anchor=`N-5:L9-L9, L45-L45` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L108): Exact RouteDecision field schema beyond "routers consulted + winner + why" and full `+route_arbitrated` payload beyond names/rank are not specified.
- **N-002/N5-I001** [I/DEP] anchor=`N-5:L9-L9, L44-L44` unit=`N-5 — policy / intensity arbitration plane` (ledger-line: DEEP-N-002.discovery.md:L109): Summary-level conflict examples (J-84 vs E-39, J-48 vs J-54) are not re-listed in the numbered source's router enumeration; only L-6 vs J-73 appears as the observable trigger example. E-39 is not in the ten-router list.

### Unit `N-6 — provenance / trust lattice + admission-gate family` — from N-002 (26 records)

- **N-002/N6-E001** [E/STATUS] anchor=`N-6:L1-L3` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L119): N-6 is TIER 2 (HIGH), lever security-composes-only-if-the-lattice-does, findings W-19+W-29, grounding STRONG, nature BUILD, effort L, WAVE 4.
- **N-002/N6-E002** [E/SEC] anchor=`N-6:L5-L5` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L120): Premise: six independent "how much do we trust this byte's origin" scales, only ONE pairwise wire between them; a T2 memory row, G-1 tainted string, and T2 source citation are three unrelated T2s; security composes only if the lattice does.
- **N-002/N6-E003** [E/CAP] anchor=`N-6:L8-L8, L19-L19, L28-L28` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L121): Declare ONE provenance lattice: a total SEMANTIC MAPPING between the six scales — NOT one shared enum.
- **N-002/N6-E004** [E/SEC] anchor=`N-6:L8-L8, L10-L10, L14-L14, L23-L23, L37-L37` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L122): F-6 DELIBERATELY made SkillTrustTier distinct from memory-TrustTier with an auditor-enforced type barrier to avoid namespace confusion; map S2 into T0-T3 terms rather than merging enums.
- **N-002/N6-E005** [E/FLOW] anchor=`N-6:L8-L8, L13-L13, L31-L31` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L123): Mapping is carried on the SAME object across: ingest(G-1 taint, 4 sites) -> memory(C-48 T0-T3) -> citation(H-53) -> oracle-TTL(C-30 T1-T4) -> skill-autonomy(F-6 S1-S4) -> cross-repo(K-23) -> research(4D EvidenceLevel).
- **N-002/N6-E006** [E/CAP] anchor=`N-6:L9-L9, L19-L19, L29-L29` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L124): Declare the ADMISSION family: F-1 VibeGate (code writes) + J-102 config-ingress (.md instruction surface) + J-109 learned-artifact (memory/playbook/case) + new K-23 workspace-manifest / M-2 org-pack surfaces share ONE scanner registry (scanner legs as shared DATA, surface as parameter).
- **N-002/N6-E007** [E/DEP] anchor=`N-6:L9-L9, L15-L15, L32-L32` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L125): They already share legs (J-102 reuses F-1's secret-grep; J-109 indexes off J-102's manifest); M-2 monotone-tighten + J-102 quarantine are the same verdict shape.
- **N-002/N6-E008** [E/LIMIT] anchor=`N-6:L10-L10, L37-L40` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L126): EXCLUDED: collapsing F-6's enum into C-48's (map, don't merge — type barrier stays); re-implementing any scanner; a seventh trust scale.
- **N-002/N6-E009** [E/DEP] anchor=`N-6:L12-L14` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L127): PLAN-asserted six scales exist independently; only J-93-C (taint-survives-memory) wires a pair.
- **N-002/N6-E010** [E/EVIDENCE] anchor=`N-6:L16-L16` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L128): E-37/E-30/E-24/G-1/F-3/J-101/J-102 assemble the injection perimeter across five letters but pass no single provenance TYPE.
- **N-002/N6-E011** [E/CFG] anchor=`N-6:L18-L19` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L129): SHIP NOW (Tier-1): lattice MAPPING table + carrying the label across ingest->memory->citation chain + shared scanner registry for the three EXISTING admission gates.
- **N-002/N6-E012** [E/DEFER] anchor=`N-6:L20-L20` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L130): DEFER (Tier-2): K-23/M-2 new admission surfaces plugging into the registry (they land with their own units; N-6 makes the registry they plug into).
- **N-002/N6-E013** [E/INT] anchor=`N-6:L22-L23` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L131): Integration: `crates/security` provenance/taint + admission gate call sites; a lattice mapping table (frozen). F-6 type barrier preserved.
- **N-002/N6-E014** [E/SURFACE] [SURFACE] anchor=`N-6:L24-L24, L41-L41` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L132): frontend_pair: a provenance badge on cited/memory surfaces (W-7/N-7 render consumer), OR N/A justified.
- **N-002/N6-E015** [E/INT] anchor=`N-6:L33-L35` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L133): reachable_from: real ingest -> memory-write -> citation chain [PRODUCTION data-plane]; a write to any admission surface [PRODUCTION admission].
- **N-002/N6-E016** [E/SEC] anchor=`N-6:L38-L38` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L134): stub_ban: NO taint lost at the memory/citation/skill boundary.
- **N-002/N6-E017** [E/LIMIT] anchor=`N-6:L39-L40` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L135): stub_ban: NO re-implementation of a scanner leg (share via the registry); NO seventh trust scale.
- **N-002/N6-E018** [E/TEST] anchor=`N-6:L43-L43` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L136): trigger: ingest of a web-tainted string -> memory-write -> citation.
- **N-002/N6-E019** [E/EVENT] anchor=`N-6:L44-L44` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L137): expect_ssb: taint/trust label MAPS coherently end-to-end (rides existing G-1/provenance rows; `+admission_scanned` if the shared registry emits).
- **N-002/N6-E020** [E/TEST] anchor=`N-6:L45-L45` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L138): expect_artifact: trust label at citation is the lattice-mapped image of the ingest label; NEGATIVE: tainted string that loses taint at memory/citation/skill boundary must FAIL.
- **N-002/N6-E021** [E/DEP] anchor=`N-6:L47-L49` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L139): Grounding: W-19 + F-6 map-don't-merge refinement, W-29 admission family, TIER-2 item 6; CLAUDE.md security posture + fail-closed admission.
- **N-002/N6-E022** [E/DEP] anchor=`N-6:L51-L52` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L140): COMPOSES-WITH G-1/C-48/F-6/C-30/K-23/4D, F-1/J-102/J-109, J-93-C; re-implements NONE (GD-1).
- **N-002/N6-A001** [A/DATA] anchor=`N-6:L8-L8` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L141): Exact lattice mapping cells (what S2 means in T0-T3 terms, what taint label a T2 source implies, incomparable meanings) are not tabulated.
- **N-002/N6-A002** [A/STATE] anchor=`N-6:L9-L9, L20-L20` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L142): Whether K-23/M-2 are gates or only surfaces, and when Tier-2 plugs them in, remains deferred without concrete schema.
- **N-002/N6-A003** [A/EVENT] anchor=`N-6:L44-L44` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L143): Optional `+admission_scanned` is conditional; no dedicated lattice-mapping SSB variant is named.
- **N-002/N6-C001** [C/STATUS] anchor=`N-6:L5-L5, L8-L8, L13-L13` unit=`N-6 — provenance / trust lattice + admission-gate family` (ledger-line: DEEP-N-002.discovery.md:L144): Premise counts "six independent" scales and enumerates six consistently; any summary that says "five provenance scales" is corrected by this source.

### Unit `N-7 — human render as a REQUIRED verdict field` — from N-002 (24 records)

- **N-002/N7-E001** [E/STATUS] anchor=`N-7:L1-L3` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L154): N-7 is TIER 2 (HIGH), lever a-non-author-can-see-the-value, finding W-7, grounding MODERATE, nature BUILD, effort M, WAVE 5.
- **N-002/N7-E002** [E/FLOW] anchor=`N-7:L5-L5` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L155): Premise: engine is ~200 units deep; human-visible trust surfaces are few and late; evidence is machine-shaped by default (RRA/SSB/hashes); only K-11 converts to human form and only for PRs.
- **N-002/N7-E003** [E/CAP] anchor=`N-7:L8-L8` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L156): Make "human render" a REQUIRED field of every gate/verdict family the way +SSB-row is required today.
- **N-002/N7-E004** [E/SURFACE] [SURFACE] anchor=`N-7:L8-L8, L27-L27` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L157): Every verdict class declares its K-11-style render target: PR comment / chat ACK line / canvas badge / brief row / M-3 org rollup.
- **N-002/N7-E005** [E/CAP] anchor=`N-7:L8-L8, L27-L27` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L158): Build ONE render substrate `render(verdict, target)` dispatcher with many verdict producers plugging in, instead of K-11 being a one-off for G-5.
- **N-002/N7-E006** [E/DEP] anchor=`N-7:L8-L8, L28-L28, L37-L37` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L159): N-2 authoring gate makes `render_field` a REQUIRED contract field: a new verdict class with no declared render fails the corpus check.
- **N-002/N7-E007** [E/DEP] anchor=`N-7:L9-L9, L31-L31` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L160): Verdict families that gain a render: J-83 receipts, L-7 hallucination verdicts, N-1 measurement numbers, N-5 RouteDecision, N-6 provenance, N-8 governance-coverage.
- **N-002/N7-E008** [E/LIMIT] [SURFACE] anchor=`N-7:L10-L10, L36-L38` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L161): EXCLUDED: re-authoring K-11 per surface; a new dashboard framework (compose existing Svelte dashboard-v2 + K-7 brief + M-3 rollup); inventing a verdict (N-7 renders existing verdicts).
- **N-002/N7-E009** [E/DEP] [SURFACE] anchor=`N-7:L12-L13` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L162): PLAN-asserted: K-11 PR-evidence render, K-7 buyer brief, G-5 reviewer cockpit, M-3 fleet rollup, canvas altitudes exist; KD-2 + K-20 wedge concede distribution gap.
- **N-002/N7-E010** [E/DEP] [SURFACE] anchor=`N-7:L14-L15` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L163): Verdicts emit as SSB rows + RRA bundles today; only K-11 humanizes them, PR-only. dashboard-v2 (Svelte 5 + Tauri v2) is the frontend; drawers are the current render surface.
- **N-002/N7-E011** [E/CFG] anchor=`N-7:L17-L18` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L164): SHIP NOW (Tier-1): `render(verdict, target)` substrate + `render_field` as required contract field (wired to N-2) + rendering 2-3 verdict families (J-83 receipts + L-7 + N-1 numbers) at their declared targets.
- **N-002/N7-E012** [E/DEFER] [SURFACE] anchor=`N-7:L19-L19` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L165): DEFER (Tier-2): M-3 org-rollup render + full canvas-badge coverage (land as org/canvas units mature).
- **N-002/N7-E013** [E/INT] [SURFACE] anchor=`N-7:L21-L22` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L166): Integration: `crates/orchestrator` verdict emission + render substrate; dashboard-v2 + K-11/K-7 as render targets.
- **N-002/N7-E014** [E/SURFACE] [SURFACE] anchor=`N-7:L23-L23, L39-L39` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L167): frontend_pair: MANDATORY — this IS the frontend unit; `cross_lang_diff` over the Tauri command + Svelte render targets must PASS.
- **N-002/N7-E015** [E/CAP] [SURFACE] anchor=`N-7:L26-L28` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L168): produces_symbols: render substrate (dispatcher + target adapters PR/chat/canvas/brief/rollup) + `render_field` as required WIRING_CONTRACT field enforced by N-2.
- **N-002/N7-E016** [E/INT] [SURFACE] anchor=`N-7:L32-L34` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L169): reachable_from: live verdict-emitting gate -> render at declared target [PRODUCTION]; dashboard/Tauri render path [PRODUCTION frontend].
- **N-002/N7-E017** [E/TEST] anchor=`N-7:L41-L41` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L170): trigger: a live verdict-emitting gate (e.g. an L-7 hallucination verdict on a real task).
- **N-002/N7-E018** [E/EVENT] [SURFACE] anchor=`ChatAck\` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L171): expect_ssb: `+verdict_rendered(PrComment\
- **N-002/N7-E019** [E/TEST] [SURFACE] anchor=`N-7:L43-L43` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L172): expect_artifact: human-readable render lands at declared target (visible in dashboard-smoker); NEGATIVE: new verdict class with no declared render fails N-2 authoring check.
- **N-002/N7-E020** [E/DEP] [SURFACE] anchor=`N-7:L45-L47` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L173): Grounding: W-7, TIER-2 item 7, KD-2/K-20; dashboardRules.md + visual-ui-loop.md; K-11 as existing render template.
- **N-002/N7-E021** [E/DEP] [SURFACE] anchor=`N-7:L49-L50` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L174): COMPOSES-WITH K-11, K-7, G-5, M-3, dashboard-v2, the verdict producers; re-implements NONE (GD-1).
- **N-002/N7-A001** [A/SURFACE] [SURFACE] anchor=`N-7:L9-L9, L18-L18` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L175): Mapping from each existing verdict class to one or multiple render targets is not enumerated beyond the Tier-1 2-3 family list.
- **N-002/N7-A002** [A/FAIL] anchor=`N-7:L8-L8, L41-L43` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L176): Accessibility, localization, acknowledgement semantics, and delivery-failure behavior for renders are absent.
- **N-002/N7-A003** [A/DEFER] [SURFACE] anchor=`N-7:L19-L19` unit=`N-7 — human render as a REQUIRED verdict field` (ledger-line: DEEP-N-002.discovery.md:L177): Full canvas-badge coverage and M-3 rollup render timing relative to org/canvas unit maturity is open.

### Unit `N-8 — governance-coverage substrate` — from N-002 (29 records)

- **N-002/N8-E001** [E/STATUS] anchor=`N-8:L1-L3` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L187): N-8 is TIER 3 (MEDIUM-HIGH), lever a-number-no-competitor-can-utter, findings W-18+W-34+W-31, grounding STRONG, nature BUILD, effort M, WAVE 5.
- **N-002/N8-E002** [E/FLOW] anchor=`N-8:L5-L5` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L188): Premise: five instruments of "is our own governance real," none citing more than one of the others; frozen prevents tampering but does NOT prove the gate catches what it claims.
- **N-002/N8-E003** [E/CAP] anchor=`N-8:L8-L8, L33-L34` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L189): ONE governance-coverage substrate with FIVE views.
- **N-002/N8-E004** [E/OBS] anchor=`N-8:L9-L9` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L190): View B-17 gate-liveness ("does this gate actually FIRE") — runtime-gate grain.
- **N-002/N8-E005** [E/OBS] anchor=`N-8:L10-L10` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L191): View K-22 doctrine-enforcer coverage ("does this rule have an ENFORCER") — authored-doctrine grain.
- **N-002/N8-E006** [E/OBS] anchor=`N-8:L11-L11` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L192): View J-46 doctrine-debt-RATE ("is adoption keeping pace").
- **N-002/N8-E007** [E/OBS] anchor=`N-8:L12-L12` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L193): View J-70 gate-manifest traceability ("does every frozen gate declare objective/evidence").
- **N-002/N8-E008** [E/OBS] anchor=`N-8:L13-L13` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L194): View H-27 kill-your-own-mutant-corpus generalized from finance ("does the enforcer actually CATCH" — prove each gate kills 100% of a frozen mutant corpus of its OWN logic + satisfies declared MRs before ACTIVE).
- **N-002/N8-E009** [E/CFG] anchor=`N-8:L14-L14, L25-L25, L35-L35, L44-L44` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L195): Make kill-your-own-mutant-corpus a gate-manifest LIFECYCLE requirement: a new gate cannot enter `.claude/gate-manifest.txt` without its mutant corpus + measured kill rate.
- **N-002/N8-E010** [E/DEP] anchor=`N-8:L14-L14, L57-L57` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L196): N-2 corpus validator (plan edges) is a sixth grain of the same "what governance is decorative" question.
- **N-002/N8-E011** [E/OBS] anchor=`N-8:L15-L15, L25-L25, L34-L34` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L197): Headline number: "N% of ELAI's rules are machine-enforced, each enforcer mutation-proven, drift-rate X/quarter" (K-1-dossier grade).
- **N-002/N8-E012** [E/LIMIT] anchor=`N-8:L16-L16, L43-L45` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L198): EXCLUDED: re-implementing any of the five (view them); a new gate framework; claiming the number before it is MEASURED through N-1 (ND-4).
- **N-002/N8-E013** [E/DEP] anchor=`N-8:L18-L22` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L199): PLAN-asserted: B-17/K-22/J-46/J-70 exist; H-27 proves each FINANCE gate kills a frozen mutant corpus; ~40 other deterministic gates self-validate only by authorship care + R8 freezing; `.claude/gate-manifest.txt` + `gate_freeze_check.py` are the frozen-gate registry.
- **N-002/N8-E014** [E/CFG] anchor=`N-8:L24-L25` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L200): SHIP NOW (Tier-1): substrate joining B-17 + K-22 + J-70 into one coverage view + headline number (measured through N-1) + kill-your-own-mutant-corpus as gate-manifest lifecycle requirement for NEW gates.
- **N-002/N8-E015** [E/DEFER] anchor=`N-8:L26-L26` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L201): DEFER (Tier-2): back-filling mutant corpora for all ~40 existing gates (lifecycle rule stops the bleed; back-fill is a rolling task).
- **N-002/N8-E016** [E/INT] anchor=`N-8:L28-L29` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L202): Integration: `.claude/tools/` governance-coverage checker + gate-manifest lifecycle hook; `crates/orchestrator` for the runtime-gate-liveness leg.
- **N-002/N8-E017** [E/SURFACE] [SURFACE] anchor=`N-8:L30-L30, L46-L46` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L203): frontend_pair: a governance tile (the marketable number) on G-5/M-3.
- **N-002/N8-E018** [E/INT] anchor=`N-8:L39-L41` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L204): reachable_from: CI/authoring invocation of coverage checker [PRODUCTION]; gate-manifest lifecycle at gate-freeze time [PRODUCTION].
- **N-002/N8-E019** [E/FAIL] anchor=`N-8:L44-L44` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L205): stub_ban: NO new gate entering the frozen set without a mutant corpus + kill rate.
- **N-002/N8-E020** [E/OBS] anchor=`N-8:L45-L45` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L206): stub_ban: NO headline number not recomputable from the journal (GD-2) / not measured through N-1 (ND-4).
- **N-002/N8-E021** [E/SURFACE] [SURFACE] anchor=`N-8:L48-L48` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L207): CLI/command surface: `python <the governance-coverage checker> over the rulebook + gate-manifest`.
- **N-002/N8-E022** [E/EVENT] anchor=`DoctrineEnforcer\` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L208): expect_ssb: `+governance_coverage_measured(GateLiveness\
- **N-002/N8-E023** [E/TEST] anchor=`N-8:L50-L50` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L209): expect_artifact: coverage % + a NEW gate lacking a mutant corpus is REFUSED entry to the frozen set; NEGATIVE: a decorative rule (clause with no enforcer) reported as covered must FAIL.
- **N-002/N8-E024** [E/DEP] anchor=`N-8:L52-L54` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L210): Grounding: W-18/W-34/W-31, TIER-3 item 8; H-plan Track 7 (H-27); wiring-enforcement R8.
- **N-002/N8-E025** [E/DEP] anchor=`N-8:L56-L57` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L211): COMPOSES-WITH B-17, K-22, J-46, J-70, H-27 (generalized), gate-manifest, N-1, N-2; re-implements NONE (GD-1).
- **N-002/N8-A001** [A/DATA] anchor=`N-8:L15-L15, L25-L25` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L212): Formula for the composite marketable number, kill-rate threshold, and quarter/window definition are unspecified.
- **N-002/N8-A002** [A/EVENT] anchor=`N-8:L9-L13, L49-L49` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L213): expect_ssb Kind list names four of five views (GateLiveness, DoctrineEnforcer, DebtRate, MutantKill) and omits an explicit J-70 traceability Kind.
- **N-002/N8-A003** [A/INT] anchor=`N-8:L48-L48` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L214): Exact governance-coverage checker filename/path is placeholder (`python <the governance-coverage checker>`).
- **N-002/N8-C001** [C/CFG] anchor=`N-8:L8-L13, L24-L25` unit=`N-8 — governance-coverage substrate` (ledger-line: DEEP-N-002.discovery.md:L215): Tier-1 "joining B-17 + K-22 + J-70" lists three instruments while the substrate is defined as five views including J-46 and H-27; H-27 is present as lifecycle requirement, J-46's Tier-1 join status is less explicit.

### Unit `N-10 — clarity / novelty -> budget wires` — from N-002 (24 records)

- **N-002/N10-E001** [E/STATUS] anchor=`N-10:L1-L3` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L225): N-10 is TIER 3 (MEDIUM-HIGH), lever active-resource-allocation, findings W-15+W-24, grounding STRONG, nature ACTIVATION, effort M, WAVE 6.
- **N-002/N10-E002** [E/FLOW] anchor=`N-10:L5-L5` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L226): Premise: four passive clarity detectors + two novelty detectors exist, but NOTHING feeds measured ambiguity/novelty of THIS task into research/reasoning budget; they only route to consent surface (grill offer) or human queue.
- **N-002/N10-E003** [E/DATA] anchor=`N-10:L9-L9, L30-L30, L33-L33` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L227): Type AmbiguityScore (shared type) from B-38 continuous ambiguity (4 dims + floors + streak) + B-40 critic findings + C-29 intent pairs + J-32 front-door confidence.
- **N-002/N10-E004** [E/DATA] anchor=`N-10:L10-L10, L30-L30, L33-L33` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L228): Type NoveltyScore (shared type, joined) from D-15 NoveltyClass (labels CLAIMS) + Phase-4F/B-21 novel-step detection (labels PLAN STEPS).
- **N-002/N10-E005** [E/DEP] anchor=`N-10:L11-L11, L31-L34` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L229): Consumers: research-tree depth (4D Simple/Moderate/Complex breadth/depth) + reasoning-effort tier (J-84) + decomposition node-sizing (J-39 bound-and-escalate) + grill/clarify offer (J-79).
- **N-002/N10-E006** [E/FLOW] [SURFACE] anchor=`N-10:L11-L11` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L230): High-ambiguity OR off-map task -> deeper research + more clarify + smaller node bound + earlier escalation, not just a chip. Ambiguity = "we don't understand the ask"; novelty = "the ask is off our map" — both buy depth.
- **N-002/N10-E007** [E/LIMIT] anchor=`N-10:L12-L12, L39-L40` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L231): EXCLUDED: re-detecting (wire existing detectors); a new detector; a non-deterministic depth choice (score->budget mapping is deterministic, TD-3).
- **N-002/N10-E008** [E/DEP] anchor=`N-10:L14-L17` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L232): PLAN-asserted: B-38/B-40/C-29/J-32 feed ONLY J-79 grill offer today (W-15); D-15 and 4F/B-21 neither feed budgets (W-24); 4D/J-84/J-39 pick depth from OTHER inputs.
- **N-002/N10-E009** [E/DEP] anchor=`N-10:L18-L18, L50-L50` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L233): N-5 intensity resolver is the natural home for the effort-tier consumer (the two compose).
- **N-002/N10-E010** [E/CFG] anchor=`N-10:L20-L21` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L234): SHIP NOW (Tier-1): AmbiguityScore + NoveltyScore shared types + wiring them to 2-3 consumers (4D depth + J-84 effort via N-5 + J-79 offer). Deterministic score->budget mapping.
- **N-002/N10-E011** [E/DEFER] anchor=`N-10:L22-L22` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L235): DEFER (Tier-2): J-39 node-sizing consumer (lands with the decomposition unit).
- **N-002/N10-E012** [E/INT] anchor=`N-10:L24-L25` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L236): Integration: `crates/orchestrator` research-tree + effort-tier (via N-5) + node-admission + grill-offer seams.
- **N-002/N10-E013** [E/SURFACE] [SURFACE] anchor=`N-10:L26-L26, L42-L42` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L237): frontend_pair: N/A (internal resource allocation), unless the depth choice is surfaced on the canvas; justified.
- **N-002/N10-E014** [E/INT] anchor=`N-10:L35-L37` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L238): reachable_from: live task where research-tree / effort tier is chosen [PRODUCTION]; `#[cfg(test)]` NEVER satisfies (R7).
- **N-002/N10-E015** [E/FAIL] anchor=`N-10:L41-L41` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L239): stub_ban: NO high-ambiguity task getting baseline depth.
- **N-002/N10-E016** [E/TEST] anchor=`N-10:L44-L44` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L240): trigger: a deliberately vague OR off-map task vs a clear baseline task.
- **N-002/N10-E017** [E/EVENT] anchor=`N-10:L45-L45` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L241): expect_ssb: reuse the shipped ambiguity (B-38) / novelty (D-15) rows; no new type.
- **N-002/N10-E018** [E/TEST] anchor=`N-10:L46-L46` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L242): expect_artifact: research depth + effort tier are HIGHER for the vague/novel task, reading the same scores; NEGATIVE: high-ambiguity task that gets baseline depth must FAIL.
- **N-002/N10-E019** [E/DEP] anchor=`N-10:L48-L50` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L243): Grounding: W-15/W-24, TIER-3 item 10; NOTE W-15 extension (J-39 fourth customer); N-5 for effort-tier consumer.
- **N-002/N10-E020** [E/DEP] anchor=`N-10:L52-L53` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L244): COMPOSES-WITH B-38/B-40/C-29/J-32, D-15/4F/B-21, 4D/J-84/J-39/J-79, N-5; re-implements NONE (GD-1).
- **N-002/N10-A001** [A/CFG] anchor=`N-10:L9-L11, L21-L21` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L245): Score ranges, normalization/join rule, cutoffs, and exact budget increments are unspecified.
- **N-002/N10-A002** [A/FLOW] anchor=`N-10:L11-L11` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L246): "more clarify" does not say whether stronger offer, mandatory query, or repeated grill.
- **N-002/N10-A003** [A/DEFER] anchor=`N-10:L11-L11, L21-L22` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L247): J-39 is both listed as a consumer and deferred to Tier-2; interim behavior when only 2-3 consumers ship is not specified beyond the Tier-1 list.
- **N-002/N10-C001** [C/EVENT] anchor=`N-10:L30-L30, L45-L45` unit=`N-10 — clarity / novelty -> budget wires` (ledger-line: DEEP-N-002.discovery.md:L248): expect_ssb says "no new type" while produces_symbols declares AmbiguityScore + NoveltyScore shared types; likely means no new SSB action type rather than no shared types.

### Unit `N-13 — OutputBoundary + structured-output capability matrix` — from N-002 (37 records)

- **N-002/N13-E001** [E/STATUS] anchor=`N-13:L1-L3` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L258): N-13 STATUS PLANNED; WAVE 4-adjacent; Tier-2 HIGH; nature BUILD/UNIFY — the one fragmentation N-1..N-12 authoring missed.
- **N-002/N13-E002** [E/CAP] anchor=`N-13:L5-L5, L13-L13` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L259): Declare ONE per-type OutputBoundary registry that every machine-actionable model output crosses, unifying >=8 scattered constrained-output implementations into one canonical object PLUS a structured-output capability matrix (fields on the ONE catalog) + a strongest-available-enforcement resolver.
- **N-002/N13-E003** [E/DEP] [SURFACE] anchor=`N-13:L5-L5, L16-L16, L40-L40` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L260): Unified machines (parameterized, never re-implemented): C-26 DecodingMode + per-role defaults + retries; E-35 XGrammar + INSTRUCTOR; I-34 CRANE + grammar pinning + post-parse re-validation + D-16 routing; J-53 tool-input repair; J-51 adherence floor; H-13 canvas per-mode grammars + sanitizeAction; B-24 VLM constrained JSON; L-1 typed ClaimSet; I-33 CEFL decision schemas.
- **N-002/N13-E004** [E/LIMIT] anchor=`N-13:L5-L5, L27-L27, L47-L47` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L261): Hard boundary: does not re-implement any grammar/parser/validator — BINDS them behind one registry + one resolver (GD-1/ND-1).
- **N-002/N13-E005** [E/CFG] anchor=`N-13:L7-L7` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L262): Estimated effort ~1-2 sessions (~L); Tier-1 in-process (boundary verdict at emission time, no network); capability resolver is pure DATA + small dispatcher.
- **N-002/N13-E006** [E/DEP] anchor=`N-13:L8-L9` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L263): Pre-requisites re-greped at edit (W-7): the >=8 machines above plus B-37 RuntimeCapabilities, I-51 models.toml. Parallel-safe with N-14, N-15.
- **N-002/N13-E007** [E/EVIDENCE] anchor=`N-13:L10-L13, L91-L94` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L264): Source: four-source-steal-2026-07-05.md Artifact 3 (S3-1 + S3-2); gaps-doc P1+P2; doctrine N-0_INDEX.md. Design line kept: "Schema validation after generation catches bad structure. Constrained generation prevents bad structure." plus I-34 post-parse re-validation distrusts the constrainer.
- **N-002/N13-E008** [E/DATA] [SURFACE] anchor=`N-13:L13-L13` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L265): Registry keyed by machine-actionable output TYPE: ToolDirective, ClaimSet, PlanSkeleton/BabyStepSpec intake, AuditFinding, MemoryCandidate, canvas ops, VLM verdicts, decision cards.
- **N-002/N13-E009** [E/DATA] anchor=`N-13:L13-L13, L27-L27` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L266): Each registry row = {schema/grammar ref (blake3-PINNED per I-34), parser fn ref, deterministic repair-policy ref (frozen table, TD-3), retry budget, fail-closed action = D-16 refusal or typed error, journaled invalid-output-rate sink = N-1 spine, enforcement_mode resolved per backend}.
- **N-002/N13-E010** [E/DATA] anchor=`N-13:L17-L17` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L267): Capability catalog is scattered: B-37 RuntimeCapabilities, I-51 models.toml, J-51 ModelCapabilityProfile exist as ONE catalog/profile but none enumerates structured-output FIELDS.
- **N-002/N13-E011** [E/CFG] anchor=`N-13:L17-L17, L28-L28` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L268): Add capability FIELDS: supports_json_schema / tool_call_schema / regex_grammar / cfg_grammar / logit_bias / streaming_validation / strict_mode on B-37 / I-51 (do NOT mint a second catalog).
- **N-002/N13-E012** [E/SEC] anchor=`N-13:L18-L18, L49-L50` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L269): Binding security: I-34 grammar pinning (grammars = blake3-pinned control-plane artifacts; enum-bearing schemas from untrusted sources REJECTED); H-13 reject-never-silently-repair; J-53/I-46 repair-is-a-FROZEN-deterministic-lookup-table (never an LLM in the gate, TD-3).
- **N-002/N13-E013** [E/CAP] anchor=`N-13:L27-L29` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L270): Tasks: declare OutputBoundary registry type (~120 LOC); add capability fields (~40 LOC); strongest-available-enforcement resolver (~50 LOC pure dispatch).
- **N-002/N13-E014** [E/DET] anchor=`N-13:L29-L29, L48-L48` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L271): Enforcement precedence: provider strict schema > native tool schema > local CFG > regex > parse-validate-retry-fail-closed. Model NEVER picks enforcement_mode (TD-3); resolver reads capability fields only.
- **N-002/N13-E015** [E/INT] anchor=`N-13:L30-L30, L43-L44` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L272): Wire boundary at emission seams: tool-directive parse, claim extraction (L-1), decision card (I-33), audit finding — each emits an OutputBoundary verdict row. Producer + in-plan consumer + smoke in the same unit.
- **N-002/N13-E016** [E/EVENT] anchor=`N-13:L31-L31, L38-L38` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L273): Emit `+output_boundary_verdict` to SSB; per-(type, backend) invalid-rate becomes an N-1 measured number.
- **N-002/N13-E017** [E/DATA] anchor=`N-13:L32-L32, L89-L89` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L274): Decision doc: `workflow/reasoning/N-13-decisions.md`.
- **N-002/N13-E018** [E/CAP] anchor=`N-13:L35-L37` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L275): produces_symbols: `crates/orchestrator/src/output_boundary.rs::OutputBoundary` and `::resolve_enforcement_mode`.
- **N-002/N13-E019** [E/EVENT] anchor=`N-13:L38-L38` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L276): SSB `+output_boundary_verdict` Pattern-2 single-variant; Kind: SchemaEnforced / GrammarEnforced / RepairedThenValidated / RefusedFailClosed.
- **N-002/N13-E020** [E/INT] anchor=`N-13:L43-L45` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L277): reachable_from: tool-directive parse, L-1 claim-extraction, I-33 decision-card, audit-finding emit [PRODUCTION]; `#[cfg(test)]`/contracts NEVER satisfies (R7).
- **N-002/N13-E021** [E/LIMIT] anchor=`N-13:L46-L52` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L278): stub_ban: NO new grammar/parser/validator engine (P0 ND-1); NO model-settable enforcement_mode; NO silent repair; NO unpinned/untrusted enum-bearing schema; NO second bespoke validation path for a registered type (N-2 duplicate-concept); NO `.unwrap()`/`.expect()`; blake3 only for schema pin.
- **N-002/N13-E022** [E/SURFACE] [SURFACE] anchor=`N-13:L53-L54` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L279): frontend_pair: a boundary-verdict badge on existing claim/decision/tool drawers reading `+output_boundary_verdict` (`cross_lang_diff` PASS), OR N/A justified if G-5 cockpit already surfaces the invalid-rate tile.
- **N-002/N13-E023** [E/TEST] anchor=`N-13:L56-L56` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L280): trigger: emit a machine-actionable output through the real orchestrator path with a backend whose capability row lacks strict schema.
- **N-002/N13-E024** [E/OBS] anchor=`N-13:L58-L58` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L281): expect_artifact: per-(type, backend) invalid-rate lands in N-1 spine; NEGATIVE: new machine-actionable type WITHOUT registry row fails N-2; second bespoke path for registered type fails N-2 duplicate-concept.
- **N-002/N13-E025** [E/TEST] anchor=`fn.*parse.*grammar"` MUST be 0; `rg "output_boundary_verdict"` in ssb.rs; N-13-decisions.md exists.` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L282): DETERMINISTIC_CHECK: wiring_contract_check --run-trigger; `rg "struct OutputBoundary"`; `rg -c "fn.*validate\
- **N-002/N13-E026** [E/TEST] anchor=`N-13:L69-L75` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L283): ACCEPTANCE: ND-1 collapse >=8 to ONE registry (9th parallel refused); ND-4 invalid-rate through N-1; I-34 pinning; TD-3 resolver-chosen mode; W-2 default ON behind `ELAI_OUTPUT_BOUNDARY_LEGACY` (old paths survive one release then ND-2 retires with user confirmation); cargo check GREEN; auditor PASS; no new dependency.
- **N-002/N13-E027** [E/CFG] anchor=`N-13:L74-L74, L78-L78` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L284): W-2 kill-switch: `ELAI_OUTPUT_BOUNDARY_LEGACY`, default ON for the new path.
- **N-002/N13-E028** [E/EVENT] anchor=`N-13:L78-L78` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L285): W-9: `+output_boundary_verdict` via action_type golden lock; SCHEMA bump baseline+1 from LIVE 267/576 as of 2026-07-04 (re-read ssb.rs:1519 + action_type_lock.rs at edit).
- **N-002/N13-E029** [E/TEST] anchor=`N-13:L80-L81` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L286): Contract tests `crates/contracts/tests/n13_output_boundary_contract.rs` (~90 LOC): (a) strongest enforcement resolve; (b) lack strict schema falls to repair-then-validate then RefusedFailClosed; (c) untrusted enum schema REJECTED; (d) enforcement_mode cannot be set from model output; (e) corrupt/oversized -> RefusedFailClosed never silent pass.
- **N-002/N13-E030** [E/DATA] anchor=`N-13:L83-L89` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L287): Files: CREATE output_boundary.rs (~210 LOC); MODIFY ssb.rs (~10 LOC); MODIFY ONE catalog (~40 LOC); MODIFY four emission seams (~40 LOC); CREATE contract test + decisions doc.
- **N-002/N13-E031** [E/EVIDENCE] anchor=`N-13:L91-L93` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L288): Gaps-doc paper-side claims are [EXTERNAL-UNVERIFIED]; fragmentation finding HIGH confidence from corpus read subject to J-detail-body sampling caveat.
- **N-002/N13-A001** [A/DATA] anchor=`N-13:L97-L98` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L289): Exact enum of machine-actionable output types is open; start with ToolDirective/ClaimSet/DecisionCard/AuditFinding, extend as N-2 flags new ones.
- **N-002/N13-A002** [A/CFG] anchor=`N-13:L98-L98` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L290): Whether B-37 RuntimeCapabilities or I-51 models.toml owns the capability fields (do NOT split across both).
- **N-002/N13-A003** [A/DEFER] anchor=`N-13:L99-L99` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L291): ND-2 retirement of the 8 scattered paths is USER-CONFIRMED (done-gate clause 7), not auto-applied.
- **N-002/N13-A004** [A/SEC] anchor=`N-13:L18-L18, L27-L27, L49-L49` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L292): "reject-never-silently-repair" (H-13) coexists with registry repair-policy (frozen deterministic table); the permitted distinction is only partly specified.
- **N-002/N13-A005** [A/CFG] anchor=`N-13:L27-L27, L58-L58` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L293): Registry storage format, schema language, concrete retry budgets, invalid-rate window, and exact refusal payload are unspecified.
- **N-002/N13-I001** [I/STATUS] anchor=`N-13:L3-L3` unit=`N-13 — OutputBoundary + structured-output capability matrix` (ledger-line: DEEP-N-002.discovery.md:L294): Summary matrix places N-13 in Wave 9; the numbered source says WAVE 4-adjacent / Wave 9 is only the summary's sequencing table.

### Unit `N-14 — self-improvement edit-loop instruments` — from N-002 (36 records)

- **N-002/N14-E001** [E/STATUS] anchor=`N-14:L1-L3` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L304): N-14 STATUS PLANNED; WAVE 3/4-adjacent; Tier-2/3; nature BUILD + ACTIVATION — feeds N-1 spine / N-3 promotion / N-5 intensity; all ADVISORY per TD-3.
- **N-002/N14-E002** [E/CAP] anchor=`N-14:L5-L5` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L305): Four instruments from EvoPolicyGym (arXiv:2607.02440): (1) stripped-AST EditKind classifier {synthesis-edit vs parametric-edit}; (2) synthesis-vs-tuning TASK-DEMAND as routing/intensity axis; (3) two-stage hidden SELECT-then-REPORT promotion rule; (4) improvement-event-timing fields on the convergence curve.
- **N-002/N14-E003** [E/LIMIT] anchor=`N-14:L5-L5, L57-L57` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L306): Hard boundary: EditKind is a deterministic diagnostic, NEVER a gate (paper admits AST-topology lens is conservative); ADVISORY axis only.
- **N-002/N14-E004** [E/CFG] anchor=`N-14:L7-L7` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L307): Effort ~1-2 sessions (~M-L); Tier-1 in-process (tree-sitter AST hashing + journal reads; no network). Four legs share one seam (score-bearing attempt row) and ship as one vertical slice.
- **N-002/N14-E005** [E/DEP] anchor=`N-14:L8-L9` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L308): Pre-requisites: verdict/attempt SSB rows, C-10 StagnationPattern + B-22b persona rotation, I-4, I-22, J-10, I-27 ConvergenceCurveMeasured, H-45/H-47, D-6 held-out, L-21, H-7, J-84, N-1, N-3, N-5; tree-sitter already in crates/knowledge. Parallel-safe with N-13, N-15.
- **N-002/N14-E006** [E/EVIDENCE] anchor=`N-14:L10-L13, L102-L103` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L309): Source: four-source-steal Artifact 2 (S2-1/S2-2/S2-3/S2-5); paper WebSearch-verified 2026-07-05. Paper headline: leaders convert synthesis edits at 41-48% hit rate; weak models churn structure at 3-10% while competitive on tuning 25-38%.
- **N-002/N14-E007** [E/DATA] anchor=`N-14:L13-L13, L30-L30` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L310): SYNTHESIS edit: previously-unseen policy-bundle AST TOPOLOGY after numeric constants are stripped. PARAMETRIC edit: changes the bundle while preserving stripped topology.
- **N-002/N14-E008** [E/CAP] anchor=`N-14:L30-L30, L40-L40` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L311): EditKind classifier (~90 LOC): blake3 of changed file-set's AST topology with numeric constants + identifiers normalized (tree-sitter; do NOT add a parser dep). unchanged topology + changed bundle => ParametricEdit; new topology => SynthesisEdit; rollbacks/retests are CandidateMgmt events, not edit kinds.
- **N-002/N14-E009** [E/DET] anchor=`N-14:L30-L30, L60-L60, L81-L81` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L312): Classification is DETERMINISTIC + byte-stable under ELAI_SEED. NEGATIVE: whitespace/rename-only diff must NOT flip topology.
- **N-002/N14-E010** [E/DATA] anchor=`N-14:L30-L30` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L313): Join {edit_kind, improved: bool} onto every score-bearing attempt row.
- **N-002/N14-E011** [E/FLOW] anchor=`N-14:L31-L31` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L314): Wire EditKind as ADVISORY evidence into: C-10/B-22b persona choice (structure-churn-without-traction -> Architect/Simplifier; wrong-structure-tuning -> Contrarian/rethink, replacing blind rotation); I-4 receipts; I-22 progress shaping (decay faster on repeated no-hit synthesis churn); J-10 insight backprop (label which edit KIND produced the win).
- **N-002/N14-E012** [E/CFG] anchor=`N-14:L32-L32, L58-L58` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L315): Synthesis-vs-tuning TASK-DEMAND axis: classify upcoming step's edit intent and feed into N-5 intensity resolver + H-7 architect/editor split + L-21 intensity + J-84 effort: synthesis-demand -> architect lane + full verification stack; tuning-demand -> editor lane eligible + parameter-sweep-shaped retries. ELAI-MEASURED thresholds only (ID-5); paper Table 4/8 grounds demand but never imports constants.
- **N-002/N14-E013** [E/FLOW] anchor=`N-14:L33-L33` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L316): Two-stage SELECT-then-REPORT (~30 LOC on H-45/H-47): selection on hidden validation slice (never agent-visible, budget-bound via H-19/N-3); reported fitness from DISJOINT held-out slice; ties resolved deterministically. Composes D-6 per-family tagging.
- **N-002/N14-E014** [E/DATA] anchor=`N-14:L34-L34` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L317): Improvement-event-timing fields on I-27: per improvement event {budget-consumed-at-jump, jump-magnitude, plateau-length}. Makes budget-to-first-improvement a per-config number.
- **N-002/N14-E015** [E/EVENT] anchor=`N-14:L35-L35, L44-L44` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L318): Emit join via one SSB field on existing attempt row (prefer-reuse) OR `+edit_kind_classified` Pattern-2 if needed; Kind: Synthesis / Parametric / CandidateMgmt. Route labels through N-1 fold (ND-4).
- **N-002/N14-E016** [E/DATA] anchor=`N-14:L36-L36, L100-L100` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L319): Decision doc: `workflow/reasoning/N-14-decisions.md`.
- **N-002/N14-E017** [E/CAP] anchor=`N-14:L39-L44` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L320): produces_symbols: `edit_kind.rs::classify_edit_kind`, `::classify_task_demand`, two-stage SELECT-then-REPORT rule, improvement-event-timing fields.
- **N-002/N14-E018** [E/INT] anchor=`N-14:L52-L55` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L321): reachable_from: score-bearing attempt/verdict emission in agent loop [PRODUCTION]; routing decision seam in task_runner [PRODUCTION]; tests NEVER satisfy R7.
- **N-002/N14-E019** [E/LIMIT] anchor=`N-14:L56-L61` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L322): stub_ban: NO EditKind as BLOCKER; NO imported paper constants; NO re-implementation of C-10/I-27/promotion; NO topology flip on whitespace/rename-only; NO unwrap; blake3 only for topology hash.
- **N-002/N14-E020** [E/SURFACE] [SURFACE] anchor=`N-14:L62-L63` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L323): frontend_pair: an edit-kind chip on existing anti-spiral / search-receipt drawer, OR N/A justified (internal advisory) if no surface consumes it yet.
- **N-002/N14-E021** [E/TEST] anchor=`N-14:L65-L65` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L324): trigger: real agent-loop attempt applying (a) constant-only edit then (b) restructuring edit, seeded under ELAI_SEED.
- **N-002/N14-E022** [E/TEST] anchor=`N-14:L66-L66` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L325): expect_ssb: (a) Parametric, (b) Synthesis; repeated no-hit synthesis churn shifts persona to Architect/Simplifier (not blind rotation).
- **N-002/N14-E023** [E/TEST] anchor=`N-14:L67-L67` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L326): expect_artifact: timing fields on I-27; SELECT-then-REPORT reports disjoint held-out fitness; NEGATIVE: whitespace/rename flip to Synthesis FAILS; promotion reporting SELECTION-slice number FAILS.
- **N-002/N14-E024** [E/TEST] anchor=`reject\` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L327): DETERMINISTIC_CHECK: wiring_contract_check; rg classify_edit_kind; rg -c "block\
- **N-002/N14-E025** [E/CFG] anchor=`N-14:L84-L84` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L328): ACCEPTANCE includes W-2 default ON behind `ELAI_EDIT_KIND_LEGACY` (blind persona rotation survives one release as fallback).
- **N-002/N14-E026** [E/EVENT] anchor=`N-14:L88-L88` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L329): W-9 SCHEMA baseline+0..1 from LIVE 267/576 as of 2026-07-04, re-read at edit (prefer reuse of attempt field).
- **N-002/N14-E027** [E/TEST] anchor=`N-14:L90-L91` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L330): Contract tests n14_edit_kind_contract.rs: constant-only->Parametric; restructure->Synthesis; whitespace/rename unchanged; persona shift; SELECT-then-REPORT disjoint; malformed/unparseable degrades to CandidateMgmt never false Synthesis.
- **N-002/N14-E028** [E/DATA] anchor=`N-14:L93-L100` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L331): Files: CREATE edit_kind.rs (~140 LOC); MODIFY anti-spiral persona seam, I-27, H-45/H-47, N-5/H-7/L-21/J-84, ssb.rs; CREATE contract test + decisions.
- **N-002/N14-E029** [E/EVIDENCE] anchor=`N-14:L102-L103` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L332): ADVERSARIAL caveats: n=1 per cell (no seeds/CIs — ELAI must not import the practice; runs its own I-38 floor); AST-topology conservative (ADVISORY only); memorization confound irrelevant (ELAI classifies its own edits).
- **N-002/N14-E030** [E/DEP] anchor=`N-14:L113-L113` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L333): Sequences behind EvoPolicyGym bench candidate (S2-4, recorded as K-2 bench note).
- **N-002/N14-C001** [C/EVIDENCE] anchor=`N-14:L13-L13, L32-L32, L58-L58, L102-L103` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L334): Paper conversion-rate figures (41-48% / 3-10% / 25-38%) are context only under ID-5; never become ELAI thresholds.
- **N-002/N14-A001** [A/DATA] anchor=`N-14:L108-L108` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L335): Which attempt/verdict SSB row already carries a per-attempt payload for the join (prefer reuse vs +edit_kind_classified) is open.
- **N-002/N14-A002** [A/CFG] anchor=`N-14:L109-L109` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L336): Whether task-demand classification runs per-step or is cached per plan node is open.
- **N-002/N14-A003** [A/DET] anchor=`N-14:L110-L110` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L337): Whether string literals normalize in the AST topology set (source of false Parametric if mechanism is encoded in a literal) is open.
- **N-002/N14-A004** [A/CFG] anchor=`N-14:L30-L32, L91-L91` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L338): Supported languages, tree-sitter fallback for parse errors, churn window, and persona-switch thresholds are unspecified.
- **N-002/N14-I001** [I/STATUS] anchor=`N-14:L3-L3` unit=`N-14 — self-improvement edit-loop instruments` (ledger-line: DEEP-N-002.discovery.md:L339): Summary places N-14 in Wave 9; numbered source says WAVE 3/4-adjacent.

### Unit `N-15 — anti-affirmation stability measurement` — from N-002 (30 records)

- **N-002/N15-E001** [E/STATUS] [SURFACE] anchor=`N-15:L1-L3` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L349): N-15 STATUS PLANNED; WAVE 5-adjacent; Tier-3; nature BUILD/ACTIVATION — feeds D-23 sycophancy bench + N-1 spine; residue structural sycophancy defenses do not cover.
- **N-002/N15-E002** [E/CAP] [SURFACE] anchor=`N-15:L5-L5` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L350): (1) framing/order-invariance extended from decision-type calls (I-33) to ADVISORY panels: present shuffled option/section order across N advisory samples; flip under presentation-only permutation => log framing-unstable and zero weight for the run. (2) two new D-23 fixture classes: option-order inversion (HBR framing) + assertion-pull (Stanford "I think X").
- **N-002/N15-E003** [E/LIMIT] anchor=`N-15:L5-L5, L49-L49` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L351): Hard boundary: advisory-only lanes; no decision gate, promotion, or merge consumes this as a blocker; it zeroes an ADVISORY weight and produces a MEASURED number.
- **N-002/N15-E004** [E/CFG] anchor=`N-15:L7-L7` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L352): Effort ~1 session (~M); Tier-1 in-process for framing permutation; fixtures are DATA feeding D-23's existing runner.
- **N-002/N15-E005** [E/DEP] anchor=`N-15:L8-L9` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L353): Pre-requisites: I-33 CEFL, I-24 scorer-neutralization, J-45, B-41, L-11, I-41, D-23 (>=70% detection / <=15% FP), E-54 matched-pair, D-14 inverted-framing, E-4 chat affirmation-drift, N-1. Parallel-safe with N-13, N-14.
- **N-002/N15-E006** [E/EVIDENCE] anchor=`N-15:L10-L10, L94-L95` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L354): Source Artifact 4 S4-2/S4-3; papers arXiv:2510.01395 Stanford (models affirm 50% more than humans, N=1604) grounds assertion-pull; HBR/NYU framing-sensitivity grounds option-order.
- **N-002/N15-E007** [E/SEC] [SURFACE] anchor=`N-15:L13-L13` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L355): Two-halves honesty (GD-5): inside a black-box model sycophancy cannot be annulled from the harness; at system boundary ELAI achieves structural irrelevance (TD-3; I-17 L1 hard-veto; K-16 RR-2; I-33 candidate set; H-31; D-14; E-4). Residue is advisory prose a human reads directly.
- **N-002/N15-E008** [E/DEP] [SURFACE] anchor=`N-15:L16-L18` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L356): Decision-grain framing defense I-33 covers DECISION-type calls, not advisory panels. I-24 pattern is applied to framing instead of style. Uncovered advisory lanes: B-41, J-45, L-11, I-41.
- **N-002/N15-E009** [E/CAP] [SURFACE] anchor=`N-15:L28-L28, L52-L52` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L357): Advisory framing-invariance check (~70 LOC): when advisory panel emits N samples, shuffle option/section order; if VERDICT flips, log framing-unstable and ZERO its weight for the run. GD-4: measure FP rate on >=3 real runs before default-ON.
- **N-002/N15-E010** [E/DATA] anchor=`N-15:L29-L29` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L358): Two D-23 fixture classes (~40 LOC DATA + oracle-by-construction): (a) option-order inversion — same facts, reordered options, decision must NOT flip; (b) assertion-pull — "I think X" prefix on false X, measure adoption WITH and WITHOUT D-14 probe. Feed D-23 detection/FP gates + E-54; do NOT re-implement D-23 runner.
- **N-002/N15-E011** [E/OBS] [SURFACE] anchor=`N-15:L30-L30` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L359): Route framing-instability rate + two fixture detection numbers through N-1 spine (ND-4); surface framing-stability number as N-8 governance-coverage view input ("advisories are framing-stable at rate X").
- **N-002/N15-E012** [E/EVENT] anchor=`N-15:L31-L31, L38-L38` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L360): Emit `+framing_stability_measured` Pattern-2 Kind: Stable / FramingUnstable / AssertionPulled OR reuse D-23 detection row Kind (prefer-reuse).
- **N-002/N15-E013** [E/DATA] anchor=`N-15:L32-L32, L92-L92` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L361): Decision doc: `workflow/reasoning/N-15-decisions.md` (two-halves honesty, advisory-only boundary, D-14 probe-cost/coverage under GD-4).
- **N-002/N15-E014** [E/CAP] anchor=`N-15:L35-L38` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L362): produces_symbols: `framing_invariance.rs::check_framing_stability`; two fixture classes in crates/knowledge or D-23 corpus; SSB framing row.
- **N-002/N15-E015** [E/INT] [SURFACE] anchor=`N-15:L44-L47` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L363): reachable_from: advisory-panel emission (B-41 / J-45) [PRODUCTION]; D-23 `elai bench sycophancy` runner seam [PRODUCTION]; tests NEVER satisfy R7.
- **N-002/N15-E016** [E/SURFACE] [SURFACE] anchor=`N-15:L47-L47, L58-L58` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L364): CLI/bench surface: `elai bench sycophancy` with the two new fixture classes.
- **N-002/N15-E017** [E/LIMIT] anchor=`N-15:L48-L53` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L365): stub_ban: NO decision/gate/promotion consuming framing-instability as BLOCKER; NO imported paper number as done-gate threshold (50%-more-affirmation is context); NO re-implementation of D-23/I-33/D-14; NO default-ON before FP bound on >=3 runs; NO unwrap.
- **N-002/N15-E018** [E/SURFACE] [SURFACE] anchor=`N-15:L54-L55` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L366): frontend_pair: framing-stability tile on G-5 cockpit / N-8 governance view reading `+framing_stability_measured`, OR N/A if sycophancy-bench tile already surfaces it.
- **N-002/N15-E019** [E/TEST] [SURFACE] anchor=`N-15:L57-L58` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L367): trigger: run advisory panel on presentation-order-sensitive input, seeded; then `elai bench sycophancy` with two fixture classes.
- **N-002/N15-E020** [E/TEST] anchor=`N-15:L58-L59` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L368): expect: flip emits FramingUnstable + weight zeroed; assertion-pull emits adoption-rate rows with/without D-14; rates land in N-1 and feed D-23's >=70%/<=15% gates; NEGATIVE: presentation-stable advisory logged unstable FAILS FP; default-ON with no FP bound FAILS GD-4.
- **N-002/N15-E021** [E/TEST] anchor=`block\` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L369): DETERMINISTIC_CHECK: wiring_contract_check; rg check_framing_stability; rg -c "gate\
- **N-002/N15-E022** [E/CFG] anchor=`N-15:L76-L76` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L370): ACCEPTANCE W-2: default ON behind `ELAI_FRAMING_INVARIANCE_LEGACY`.
- **N-002/N15-E023** [E/EVENT] anchor=`N-15:L80-L80` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L371): W-9 SCHEMA baseline+0..1 from LIVE 267/576 as of 2026-07-04.
- **N-002/N15-E024** [E/TEST] anchor=`N-15:L82-L83` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L372): Contract tests n15_framing_invariance_contract.rs: flip->FramingUnstable+zero weight; stable->Stable no FP; option-order flip DETECTED; assertion-pull higher adoption WITHOUT D-14 than WITH; single-sample advisory degrades to Stable-unknown never silent false FramingUnstable.
- **N-002/N15-E025** [E/DATA] anchor=`N-15:L85-L92` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L373): Files: CREATE framing_invariance.rs (~90 LOC); CREATE/MODIFY D-23 corpus; MODIFY B-41/J-45/L-11/I-41 seams; MODIFY ssb.rs; MODIFY N-8 governance view; CREATE contract test + decisions.
- **N-002/N15-E026** [E/EVIDENCE] anchor=`N-15:L94-L95` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L374): HBR/NYU underlying piece is [EXTERNAL] until fetched — fixture is ELAI-measured regardless. Anthropic CoT-unfaithfulness ids are K-16 RR-1 grounding, not consumed here.
- **N-002/N15-A001** [A/CFG] anchor=`N-15:L28-L28, L101-L101` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L375): Sample count N, permutation scheme, verdict-equivalence definition, stability threshold, and weight scope beyond "for the run" are open.
- **N-002/N15-A002** [A/CFG] anchor=`N-15:L100-L100` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L376): D-14 probe doubles model-call cost; coverage policy (which claims get probed) is the FP/cost knob under GD-4 — default coverage decided at authoring from measured FP/cost curve.
- **N-002/N15-A003** [A/SURFACE] [SURFACE] anchor=`N-15:L102-L102` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L377): Whether framing-instability should also emit a human-render note (N-7); default zero-weight + spine number, add render only if N-7 has shipped.
- **N-002/N15-I001** [I/STATUS] anchor=`N-15:L3-L3` unit=`N-15 — anti-affirmation stability measurement` (ledger-line: DEEP-N-002.discovery.md:L378): Summary places N-15 in Wave 9; numbered source says WAVE 5-adjacent.

### Unit `N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` — from N-002 (27 records)

- **N-002/N16-E001** [E/STATUS] anchor=`N-16:L1-L3` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L388): N-16 is EXTERNAL-STEAL (DeepReason 2026-07-08), lever is-our-own-adjudication-still-real, verdicts S4+S6+S7, grounding STRONG (steal-deepreason; ELAI anchors PLAN-asserted re-grep W-7), nature EXTENSION(N-8)+BUILD, effort M, WAVE 9.
- **N-002/N16-E002** [E/EVIDENCE] anchor=`N-16:L5-L5` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L389): PROOF_SPINE (PSR-2): feeds PS-2 (composed gate-tax vs locked ceiling, K-4) transitively: S7 measures fraction of shipped units whose declared VERIFY legs never ran; S4 keeps gate verdicts from standing on a since-refuted evaluator. N-16 is the honesty check ON the number PS-2 sells.
- **N-002/N16-E003** [E/FLOW] anchor=`N-16:L7-L7` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L390): Premise: GENERATOR capture covered by anti-spiral + L-23; ADJUDICATOR capture — criticism ritualizes while statuses stay formally correct, and verdicts keep standing after the standard is refuted — is residue N-8's five views do NOT catch.
- **N-002/N16-E004** [E/CAP] anchor=`N-16:L7-L7` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L391): N-16 adds three missing views over the SAME "what part of our own governance is decorative" question N-8 asks: sixth/seventh/eighth views over one join, not a new framework (ND-1). All three are ADVISORY alarms that force retrieval/re-audit/escalation, never PASS signals (TD-3, HG-4); measured through N-1 (ND-4); ELAI-measured thresholds only (ID-5).
- **N-002/N16-E005** [E/CAP] anchor=`N-16:L11-L11, L25-L25, L34-L34` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L392): S4 EVALUATOR-REFUTATION => VERDICT-COLLAPSE: every SSB verdict row carries evaluator-id + version-hash; a registered evaluator-refutation event triggers F-5-style staleness cascade over exactly the verdicts that evaluator issued: QUARANTINE into advisory re-audit queue, NEVER delete (append-only).
- **N-002/N16-E006** [E/EVENT] anchor=`N-16:L11-L11, L19-L19` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L393): Refutation event sources: failed H-27 mutant corpus; I-24/N-15 bias detection; frozen-tool bug fix.
- **N-002/N16-E007** [E/LIMIT] anchor=`N-16:L11-L11, L14-L14, L44-L44` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L394): Takes only the valuable consequence of Dung reinstatement WITHOUT the argument-graph adjudicator (REJECTED, steal §4.3).
- **N-002/N16-E008** [E/OBS] anchor=`N-16:L12-L12` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L395): S6 LAMBDA EXOGENOUS-GROUNDING FLOOR: windowed fraction of accepted claims/verdicts whose support bottoms out in an EXOGENOUS anchor (program check, off-agent artifact, retrieval evidence, human ruling) vs a model-judged rubric.
- **N-002/N16-E009** [E/FAIL] [SURFACE] anchor=`N-16:L12-L12, L35-L35` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L396): lambda below a floor forces retrieval/research/escalation (a BRAKE), and is NEVER a pass signal; detects STALLED/closed-loop dynamics, not wrong-but-stable consensus. One SSB-derived tile + one threshold; journal-recomputable.
- **N-002/N16-E010** [E/OBS] anchor=`N-16:L13-L13, L36-L36` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L397): S7 ADJUDICATOR-RITUAL METRICS: entropy over auditor finding-CLASSES per window (new surfaces vs re-litigating same nodes) + fraction of shipped units whose declared VERIFY legs never ran (criticism debt) + rate at which gates THEMSELVES get challenged/amended (validity-node-attack-rate ~= 0 translation). Feeds N-8's marketable governance number.
- **N-002/N16-E011** [E/LIMIT] anchor=`N-16:L14-L14` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L398): EXCLUDED: Dung grounded-extension adjudicator + reinstatement as general mechanism; any of the three ACTING as a gate that emits PASS; claiming a governance number before MEASURED through N-1.
- **N-002/N16-E012** [E/DEP] anchor=`N-16:L16-L22` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L399): PLAN-asserted: N-8 five views [PLANNED]; F-5 staleness cascade [WIRED-class]; H-27 + I-24/N-15; SSB verdict rows; gate-manifest + gate_freeze_check.py; N-1; D-17 refusal SLA; R4 off-agent evidence.
- **N-002/N16-E013** [E/CFG] [SURFACE] anchor=`N-16:L24-L25` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L400): SHIP NOW (Tier-1, in-process): evaluator-id+version-hash field + S4 quarantine cascade; S6 lambda tile + floor + brake (advisory); S7 finding-class entropy + VERIFY-leg-ran rate. All observe-only (GD-4) until FP bound measured on >=3 runs.
- **N-002/N16-E014** [E/DEFER] anchor=`N-16:L26-L26` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L401): DEFER (Tier-2): back-filling version-hashes onto historical verdict rows; gate-amend-rate leg of S7 (needs >=1 quarter of gate-manifest history).
- **N-002/N16-E015** [E/INT] anchor=`N-16:L28-L29` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L402): Integration: crates/orchestrator (verdict-row field + F-5 cascade hook + lambda/ritual derivations) + `.claude/tools` governance-coverage checker gains three views; N-1 spine sink.
- **N-002/N16-E016** [E/SURFACE] [SURFACE] anchor=`N-16:L30-L30, L47-L47` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L403): frontend_pair: three numbers on the N-8 / G-5 / M-3 governance tile — lambda gauge + quarantine-queue count + ritual-entropy band.
- **N-002/N16-E017** [E/INT] anchor=`N-16:L39-L42` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L404): reachable_from: registered evaluator-refutation event > F-5 verdict-collapse sweep [PRODUCTION]; CI/authoring governance-coverage checker > S6/S7 derivations [PRODUCTION]; tests NEVER satisfy R7.
- **N-002/N16-E018** [E/LIMIT] anchor=`N-16:L43-L46` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L405): stub_ban: NO Dung adjudicator; NO deletion of quarantined verdict; NO lambda/ritual number emitting PASS; NO number claimed before N-1 measurement; NO re-implementation of N-8/F-5/H-27/I-24; NO unwrap; blake3 not sha2.
- **N-002/N16-E019** [E/TEST] anchor=`N-16:L49-L49` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L406): trigger: register an evaluator-refutation event then run governance-coverage checker over rulebook + verdict stream.
- **N-002/N16-E020** [E/EVENT] anchor=`N-16:L50-L50` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L407): expect_ssb: `+evaluator_closure_swept` Pattern-2 Kind EvaluatorRefuted/VerdictQuarantined/Reinstated + `+grounding_ratio_measured` Kind AboveFloor/BelowFloorBraked; S7 prefer-reuse `governance_coverage_measured` (+Kind AdjudicatorRitual); baseline+2 from LIVE 267/576 as of 2026-07-04.
- **N-002/N16-E021** [E/TEST] anchor=`N-16:L51-L51` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L408): expect_artifact: refuted evaluator's past verdicts in advisory re-audit queue (NOT deleted); lambda under floor fires retrieval/escalation brake; shipped unit whose VERIFY leg never ran counted as criticism debt. NEGATIVE: silent delete FAILS; lambda/ritual as PASS FAILS; number without N-1 FAILS.
- **N-002/N16-E022** [E/DEP] anchor=`N-16:L53-L56` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L409): Grounding: steal-deepreason S4/S6/S7 + DeepReason spec §11 [EXTERNAL/DESIGN-RATIONALE, ID-5]; ELAI anchors re-grepped at edit; TD-3/ID-5/R4/R8/GD-2/ND-1/ND-4.
- **N-002/N16-E023** [E/DEP] anchor=`N-16:L58-L59` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L410): COMPOSES-WITH N-8 (EXTENDED), F-5, H-27 + I-24 + N-15, gate-manifest, N-1, D-17, R4; re-implements NONE.
- **N-002/N16-A001** [A/CFG] anchor=`N-16:L12-L13, L25-L25` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L411): Window length, lambda numerator/denominator edge cases, floor value, brake duration, re-audit release criteria, evaluator identity migration, and entropy formula are unspecified.
- **N-002/N16-A002** [A/EVENT] anchor=`N-16:L11-L11` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L412): "frozen-tool bug fix" event registration path and version-hash algorithm are unspecified.
- **N-002/N16-A003** [A/STATUS] anchor=`N-16:L3-L3, L16-L22` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L413): "PLAN-asserted, re-grep W-7" explicitly lowers confidence in local anchor verification even though steal grounding is STRONG.
- **N-002/N16-A004** [A/EVENT] anchor=`N-16:L50-L50` unit=`N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual)` (ledger-line: DEEP-N-002.discovery.md:L414): Kind `Reinstated` is named in expect_ssb without defining when/how a quarantined verdict is reinstated.

---

## Cross-source interactions (namespaced)

### N-001 interactions

- **N-001/IX-001** [E] (DEEP-N-001.discovery.md:L385): N-1 is the Wave-1 keystone: every later number claim and GD-4 FP gate must measure through its spine and ModelCapabilityProfile sink (ND-4). | evidence=N0-E031, N0-E069, N1-E001–N1-E006, N1-E030
- **N-001/IX-002** [E] (DEEP-N-001.discovery.md:L386): N-3 depends on N-1 outcome labels as ground truth for deflation and is Wave 3 after the spine. | evidence=N0-E036, N0-E071, N3-E001, N3-E013
- **N-001/IX-003** [E] (DEEP-N-001.discovery.md:L387): N-9 depends on N-1 labels for delta scoring and applies N-3 deflation when ablation feeds promotion. | evidence=N0-E042, N9-E001, N9-E006, N9-E009
- **N-001/IX-004** [E] (DEEP-N-001.discovery.md:L388): N-4 and N-3 co-occupy Wave 3 on measured ground; N-4 is independent of N-1 in its own metadata but program ND-4 still routes any claimed cost/cheaper number through the spine. | evidence=N0-E071, N4-E001, N4-E012, N0-E031
- **N-001/IX-005** [E] (DEEP-N-001.discovery.md:L389): N-11 Wave 7 extends L-16 rather than inventing a fold; N-0 landmine forbids re-authoring the fold and narrows residual to recover_stubs + H-49 store. | evidence=N0-E044, N0-E087, N11-E007–N11-E008
- **N-001/IX-006** [E] (DEEP-N-001.discovery.md:L390): N-12 Wave 8 is the ND-1 sweep whose ongoing anti-refork enforcement is N-2's duplicate-concept check (N-2 is out of this chunk but named as consumer/keeper). | evidence=N0-E045, N0-E076, N12-E002, N12-E026, N12-E028
- **N-001/IX-007** [E] (DEEP-N-001.discovery.md:L391): W-1..W-10 + GD-1..GD-5 + ND-1..ND-5 + eight-clause DONE definition apply to every unit in this chunk. | evidence=N0-E012–N0-E033
- **N-001/IX-008** [E] (DEEP-N-001.discovery.md:L392): Action-type registry: N-1 +calibration_bootstrap_superseded; N-3 +promotion_deflated; N-4 +receipt_memo; N-9 +counterfactual_scored; N-11 and N-12 add none. | evidence=N0-E052, N0-E054–N0-E055, N0-E060, N0-E062–N0-E063, N1-E027, N3-E022, N4-E026, N9-E018, N11-E022, N12-E025
- **N-001/IX-009** [E] [SURFACE] (DEEP-N-001.discovery.md:L393): Production reachability invariant R7 is repeated per unit: CLI bench (N-1), real SHADOW->ACTIVE (N-3), live J-83 gate (N-4), live ablation (N-9), live fold re-derive (N-11), multi-call-site surviving impls (N-12). | evidence=N1-E025, N3-E020, N4-E024, N9-E016, N11-E020, N12-E023
- **N-001/IX-010** [E] [SURFACE] (DEEP-N-001.discovery.md:L394): User-facing surfaces in this chunk: MeasurementPanel or G-5 (N-1); deflation column on promotion drawers (N-3); memo-hit on belief/claim drawer (N-4); attribution drawers reuse (N-9); canvas altitude fold verification (N-11); optional FailureClass drawer (N-12); CLI `elai bench <family>` (N-1). | evidence=N1-E019–N1-E020, N3-E017, N4-E021, N9-E013, N11-E017, N12-E020
- **N-001/IX-011** [E] (DEEP-N-001.discovery.md:L395): Retirement/user-confirm: W-2 LEGACY one-release + ND-2 maps bind N-11 recover_stubs/H-49 retirement and any unifier that supersedes live paths. | evidence=N0-E014, N0-E029, N0-E083, N11-E006, N11-E021
- **N-001/IX-012** [E] (DEEP-N-001.discovery.md:L396): GD-2 determinism: N-1 fixture, N-9 journal-label deltas, N-11 pinned-seed byte-identical folds all recompute from SSB/journal under seed. | evidence=N0-E024, N1-E017, N9-E017, N11-E022–N11-E023
- **N-001/IX-013** [E] (DEEP-N-001.discovery.md:L397): Fail-closed family: ND-4 foreign knee store (N-1); missing N / ignore-trial ACTIVE (N-3); F-5-stale memo Hit (N-4); fat adapter (N-9); ad-hoc fold path (N-11); parallel reappearance / admission bypass (N-12). | evidence=N1-E028, N3-E021–N3-E023, N4-E025–N4-E027, N9-E017–N9-E019, N11-E021–N11-E023, N12-E024–N12-E026
- **N-001/IX-014** [E] (DEEP-N-001.discovery.md:L398): N-4 memo composes C-30 cache shape, J-96 case nodes, J-21 identity, and F-5 invalidation; I-8 is the negative-space twin already caching failures. | evidence=N4-E007–N4-E010, N4-E015–N4-E017
- **N-001/IX-015** [E] (DEEP-N-001.discovery.md:L399): N-3 re-keys H-17/H-18/H-19 and hardens eight promotion consumers; I-30/I-31 are the Track-17 precedent. | evidence=N3-E003–N3-E008, N3-E014
- **N-001/IX-016** [E] (DEEP-N-001.discovery.md:L400): N-9 unifies four attribution + four ablation engines into one core; Tier-1 proves with role/verifier/ranker only. | evidence=N9-E003–N9-E005, N9-E010–N9-E011
- **N-001/IX-017** [E] (DEEP-N-001.discovery.md:L401): N-12(a) FailureClass feeds auditor, anti-spiral, MAST, corpus rows, and case edges as one classification event — training-data free for J-25/I-12. | evidence=N12-E003–N12-E005
- **N-001/IX-018** [E] (DEEP-N-001.discovery.md:L402): N-12(b) abstain taint composition raises L-2 risk floor on answer-without-evidence after upstream retrieval abstain. | evidence=N12-E006–N12-E007
- **N-001/IX-019** [E] (DEEP-N-001.discovery.md:L403): N-12(d) closes the C-47/C-49 write path that J-109 already guards for E-5/I-12/J-97. | evidence=N12-E010
- **N-001/IX-020** [E] (DEEP-N-001.discovery.md:L404): Blake3-only hashing is global (N-0 hard rule) and restated on N-1 and N-4 claim-identity paths. | evidence=N0-E080, N1-E026, N4-E025
- **N-001/IX-021** [E] (DEEP-N-001.discovery.md:L405): Schema baseline 267/576 must be re-read at edit; no unit may hard-code the bump. | evidence=N0-E051, N0-E064, N0-E021
- **N-001/IX-022** [I] (DEEP-N-001.discovery.md:L406): N-4's "measurably cheaper" second pass is the kind of number ND-4 would require to route through N-1 if claimed as an ELAI-measured GTM metric, even though N-4's acceptance is a local cost observation. | evidence=N4-E012, N4-E027, N0-E031
- **N-001/IX-023** [E] (DEEP-N-001.discovery.md:L407): External-steal and N-13..N-16 are defined in N-0 but their detail files are outside DEEP-N-001; only index-level atoms are in scope here. | evidence=N0-E046–N0-E050, N0-E065–N0-E068, N0-E077

### N-002 interactions

- **N-002/IX-001** [E] (DEEP-N-002.discovery.md:L422): N-2 is the plan-corpus ND-1/ND-3 enforcer that later N units extend: N-7 makes `render_field` required under N-2; N-13 makes OutputBoundary registry rows and no-second-validation-path fail under N-2. | evidence=N2-E004–N2-E007, N2-E031; N7-E006, N7-E019; N13-E021, N13-E024
- **N-002/IX-002** [E] (DEEP-N-002.discovery.md:L423): N-7 render consumers include N-5 RouteDecision, N-6 provenance, N-8 governance-coverage, N-1 numbers, J-83, L-7; N-5/N-6 explicitly name themselves as N-7 render consumers. | evidence=N7-E007; N5-E017; N6-E014
- **N-002/IX-003** [E] (DEEP-N-002.discovery.md:L424): N-5 intensity resolver is the effort-tier home for N-10 scores and the task-demand axis consumer for N-14 synthesis-vs-tuning demand. | evidence=N5-E007–N5-E009; N10-E009–N10-E010; N14-E012
- **N-002/IX-004** [E] [SURFACE] (DEEP-N-002.discovery.md:L425): N-8 is the five-view governance substrate; N-16 EXTENDS it with S4/S6/S7 views rather than forking a new framework; N-15 feeds framing-stability into N-8's governance number; N-2 is named as a sixth grain of decorative-governance. | evidence=N8-E003–N8-E010; N16-E001, N16-E004; N15-E011
- **N-002/IX-005** [E] (DEEP-N-002.discovery.md:L426): N-1 measurement spine (outside this chunk but referenced) sinks N-8 headline numbers, N-13 invalid rates, N-14 labels, N-15 framing/adoption rates, and N-16 lambda/ritual numbers (ND-4). | evidence=N8-E011, N8-E020; N13-E016; N14-E015; N15-E011; N16-E004, N16-E009
- **N-002/IX-006** [E] (DEEP-N-002.discovery.md:L427): Routing/resource loop: N-10 joins ambiguity/novelty → consumers; N-14 adds task-demand axis; routers fire; N-5 arbitrates with frozen precedence and `RouteDecision`/`+route_arbitrated`; N-16 may brake on low exogenous grounding without creating a pass. | evidence=N10-E003–N10-E006; N14-E012; N5-E003–N5-E005, N5-E023; N16-E009
- **N-002/IX-007** [E] (DEEP-N-002.discovery.md:L428): Provenance/output path: N-6 maps trust labels ingest→memory→citation→…; N-13 OutputBoundary types include ClaimSet/decision cards at emission; N-7 requires human render for resulting verdicts. | evidence=N6-E005; N13-E008, N13-E015; N7-E003–N7-E004
- **N-002/IX-008** [E] (DEEP-N-002.discovery.md:L429): Governance lifecycle: N-7 requires render declaration; N-8 requires mutant corpus + kill rate for frozen entry; N-16 adds evaluator-id/version-hash and refutation→F-5 quarantine; extended coverage reports lambda/entropy/criticism debt. | evidence=N7-E003; N8-E009, N8-E019; N16-E005–N16-E010
- **N-002/IX-009** [E] (DEEP-N-002.discovery.md:L430): N-15 bias detection is an explicit S4 evaluator-refutation event source for N-16. | evidence=N15-E009; N16-E006
- **N-002/IX-010** [E] (DEEP-N-002.discovery.md:L431): N-13/N-14/N-15 declare mutual parallel-safety (disjoint files/seams). | evidence=N13-E006; N14-E005; N15-E005
- **N-002/IX-011** [E] [SURFACE] (DEEP-N-002.discovery.md:L432): Authoring-time surfaces: N-2 CLI composition check + N-8/N-16 governance-coverage checker; runtime/bench surfaces: N-15 `elai bench sycophancy`, N-5 task_runner arbitration, N-13 emission seams, N-14 agent-loop join. | evidence=N2-E026; N8-E021; N16-E015; N15-E016; N5-E016; N13-E015; N14-E018
- **N-002/IX-012** [E] [SURFACE] (DEEP-N-002.discovery.md:L433): UI composition on shared tiles: N-8 governance tile on G-5/M-3 receives N-16 lambda gauge / quarantine count / ritual-entropy band and N-15 framing-stability input; N-5/N-6/N-7/N-13 add canvas/RRA/drawer badges. | evidence=N8-E017; N16-E016; N15-E018; N5-E017; N6-E014; N7-E014; N13-E022
- **N-002/IX-013** [E] (DEEP-N-002.discovery.md:L434): Kill-switches with default-ON new path: `ELAI_OUTPUT_BOUNDARY_LEGACY` (N-13), `ELAI_EDIT_KIND_LEGACY` (N-14), `ELAI_FRAMING_INVARIANCE_LEGACY` (N-15). | evidence=N13-E027; N14-E025; N15-E022
- **N-002/IX-014** [E] (DEEP-N-002.discovery.md:L435): Shared security posture: blake3-only hashing (N-13 schema pin, N-14 topology, N-16 ban sha2); no `.unwrap()`/`.expect()` on prod paths across N-5/N-13/N-14/N-15/N-16; TD-3 advisory-or-frozen-data (model never sets precedence/enforcement). | evidence=N5-E020–N5-E021; N13-E012–N13-E014, N13-E021; N14-E003, N14-E019; N15-E003; N16-E018
- **N-002/IX-015** [E] (DEEP-N-002.discovery.md:L436): SSB schema pressure: N-13 baseline+1 (`+output_boundary_verdict`); N-14 prefer +0..1; N-15 +0..1; N-16 baseline+2 (`+evaluator_closure_swept` + `+grounding_ratio_measured`); all re-read live 267/576 baseline at edit. | evidence=N13-E028; N14-E026; N15-E023; N16-E020
- **N-002/IX-016** [E] (DEEP-N-002.discovery.md:L437): N-6 Tier-1 ships registry for three EXISTING admission gates; zip-slip/admission family expansion and K-23/M-2 plug-in are deferred (summary NX-E09 fold note is disposition-level, not a numbered-source ship requirement for this chunk alone). | evidence=N6-E006, N6-E011–N6-E012
- **N-002/IX-017** [I] (DEEP-N-002.discovery.md:L438): ClaimSet is both an N-13 boundary type and an N-4/J-83 claim carrier (N-4 outside this chunk); temporal order ClaimSet→boundary→claim object is not stated in these sources. | evidence=N13-E008, N13-E015
- **N-002/IX-018** [E] (DEEP-N-002.discovery.md:L439): Fail-closed authoring R1 (N-2 zero-edge exit, dangling exit-0) composes with R10 seam gate (N-5 missing arbitration row) and N-13/N-6 boundary/taint negatives. | evidence=N2-E017, N2-E029; N5-E019, N5-E024; N13-E024; N6-E016, N6-E020

---

## Ambiguity and conflict registers (namespaced)

### N-001 AC register

- **N-001/AC-001** [conflict] (DEEP-N-001.discovery.md:L413): N-0 "16 sessions" vs "THE 12 SESSIONS" banner vs external N-13..N-16 append; total units are 16. | refs=N0-C001, N0-I001
- **N-001/AC-002** [ambiguity] (DEEP-N-001.discovery.md:L414): N-8 Kind list (four named in schema block) vs five governance views including J-70 traceability. | refs=N0-A001
- **N-001/AC-003** [ambiguity] (DEEP-N-001.discovery.md:L415): LEGACY flag naming, release duration, and rollback switch wiring under W-2/ND-2. | refs=N0-A002
- **N-001/AC-004** [dependency] (DEEP-N-001.discovery.md:L416): I-1 unshipped behavior for N-5/N-9 deferred shims is named but unspecified. | refs=N0-A003
- **N-001/AC-005** [ambiguity] (DEEP-N-001.discovery.md:L417): N-1 six promotion loops, bootstrap inventory, MeasurementPanel decision criterion, ~30 fan-in list. | refs=N1-A001–N1-A004
- **N-001/AC-006** [ambiguity] (DEEP-N-001.discovery.md:L418): N-3 N-threshold, candidate-family key, DP parameters, H-47 migration. | refs=N3-A001–N3-A003
- **N-001/AC-007** [ambiguity] (DEEP-N-001.discovery.md:L419): N-4 unified abstain/DKG schema, TTL/eligibility, Miss/Revalidated fixtures. | refs=N4-A001–N4-A003
- **N-001/AC-008** [ambiguity] (DEEP-N-001.discovery.md:L420): N-9 replay isolation/seed/delta metric; adapter-to-target mapping; optional fourth Tier-1 adapter; Kind Rule omission. | refs=N9-I001, N9-A001–N9-A003
- **N-001/AC-009** [ambiguity] (DEEP-N-001.discovery.md:L421): N-11 fold plugin schema, Kani property, retirement migration, optional third Tier-1 surface. | refs=N11-A001–N11-A003
- **N-001/AC-010** [ambiguity/count] (DEEP-N-001.discovery.md:L422): N-12 "eight collapses" vs seven labeled (a)..(g); FailureClass mapping; LSP pool ops; adapter specs; FSM equality criterion. | refs=N12-I001, N12-A001–N12-A004

### N-002 AC register

- **N-002/AC-001** [ambiguity] (DEEP-N-002.discovery.md:L445): N-2 orphan-producer wording and ND-3/schema/similarity thresholds remain open. | refs=N2-A001–N2-A003
- **N-002/AC-002** [ambiguity] (DEEP-N-002.discovery.md:L446): N-5 tie-breaks, task-difficulty measure, and intensity-tuple omission of edit-format. | refs=N5-A001–N5-A003
- **N-002/AC-003** [correction] (DEEP-N-002.discovery.md:L447): N-6 is six scales, not five; Tier-2 K-23/M-2 plug-in schema open. | refs=N6-C001, N6-A001–N6-A003
- **N-002/AC-004** [ambiguity] (DEEP-N-002.discovery.md:L448): N-7 per-class render target map and delivery-failure behavior unspecified. | refs=N7-A001–N7-A003
- **N-002/AC-005** [conflict/qualify] (DEEP-N-002.discovery.md:L449): N-8 five views vs Tier-1 join of three instruments; SSB Kind list omits explicit J-70 Kind; checker path placeholder. | refs=N8-C001, N8-A001–N8-A003
- **N-002/AC-006** [conflict/qualify] (DEEP-N-002.discovery.md:L450): N-10 produces shared score types but expect_ssb says "no new type"; J-39 listed and deferred. | refs=N10-C001, N10-A001–N10-A003
- **N-002/AC-007** [ambiguity] (DEEP-N-002.discovery.md:L451): N-13 type enum, catalog owner, repair-vs-reject distinction, retry budgets, user-confirmed retirement. | refs=N13-A001–N13-A005
- **N-002/AC-008** [evidence/conflict] (DEEP-N-002.discovery.md:L452): N-14 paper rates are context only; open attempt-row reuse, literal normalization, parse fallback. | refs=N14-C001, N14-A001–N14-A004
- **N-002/AC-009** [ambiguity] (DEEP-N-002.discovery.md:L453): N-15 sample count N, probe coverage cost knob, optional N-7 render note. | refs=N15-A001–N15-A003
- **N-002/AC-010** [ambiguity] (DEEP-N-002.discovery.md:L454): N-16 window/floor/entropy formulas, Reinstated semantics, version-hash algorithm, W-7 re-grep confidence. | refs=N16-A001–N16-A004
- **N-002/AC-011** [status drift] (DEEP-N-002.discovery.md:L455): Summary Wave-9 placement for N-13/N-14/N-15 vs numbered WAVE 4-adjacent / 3-4-adjacent / 5-adjacent labels. | refs=N13-I001; N14-I001; N15-I001

---

## Summary deltas (namespaced)

Relational records: deep vs `14-N-SUMMARY_INDEX.discovery.md`. Not double-counted as features.

### N-001 summary deltas

- **N-001/SD-N0-001** [Δ] section=N-0 (DEEP-N-001.discovery.md:L432): **ADD** full W-1..W-10 inheritance text including N-specific W-2 LEGACY caveat and production entrypoint list. | summary_anchor=14-N:L41, L830 | source_atoms=N0-E012–N0-E022
- **N-001/SD-N0-002** [Δ] section=N-0 (DEEP-N-001.discovery.md:L433): **ADD** full GD-1..GD-5 and ND-1..ND-5 blocking rules with retirement map, edge-as-data, spine measurement, and adapter-thinness. | summary_anchor=14-N:L41, L830 | source_atoms=N0-E023–N0-E032
- **N-001/SD-N0-003** [Δ] section=N-0 (DEEP-N-001.discovery.md:L434): **ADD** eight-clause DONE definition and session-log shipping pipeline. | summary_anchor=14-N:L830 | source_atoms=N0-E033, N0-E089
- **N-001/SD-N0-004** [Δ] section=N-0 (DEEP-N-001.discovery.md:L435): **ADD** live SCHEMA 267/576 baseline, per-unit action_type bumps (including N-11/N-12 +0), and re-read-at-edit rule. | summary_anchor=14-N:L708-L726 | source_atoms=N0-E051–N0-E068
- **N-001/SD-N0-005** [Δ] section=N-0 (DEEP-N-001.discovery.md:L436): **ADD** complete WAVE 1..9 unlock-DAG, three system-wide unlocks, and I-1 pre-req for WAVE 4. | summary_anchor=14-N:L760-L780 | source_atoms=N0-E069–N0-E079
- **N-001/SD-N0-006** [Δ] section=N-0 (DEEP-N-001.discovery.md:L437): **ADD** hard rules (blake3, no new deps, user-confirmed removal) and carried landmines (prose edges, F-6 type barrier, L-16/L-8 downgrades, J drift). | summary_anchor=14-N:L830-L838 | source_atoms=N0-E080–N0-E088
- **N-001/SD-N0-007** [Δ] section=N-0 (DEEP-N-001.discovery.md:L438): **CORRECT** summary open ambiguity "15 sessions vs 16": N-0 explicitly has 16 units with external track N-13..N-16 and DeepReason N-16. | summary_anchor=14-N:L26-L26, L827 | source_atoms=N0-E001, N0-C001, N0-I001
- **N-001/SD-N0-008** [Δ] section=N-0 (DEEP-N-001.discovery.md:L439): **ADD** index-level N-5..N-8/N-10/N-13..N-16 structure atoms that the summary already partially covers but without ND/schema/sequencing authority. | summary_anchor=14-N:L169-L580 | source_atoms=N0-E038–N0-E050, N0-E091–N0-E100
- **N-001/SD-N1-001** [Δ] section=N-1 (DEEP-N-001.discovery.md:L445): **ADD** exact produces paths `measurement/spine.rs` and `bootstrap_pack.rs`, PLAN-asserted. | summary_anchor=14-N:L45-L78 | source_atoms=N1-E021–N1-E023
- **N-001/SD-N1-002** [Δ] section=N-1 (DEEP-N-001.discovery.md:L446): **ADD** full supersession Kind set Prefilled / LocallySuperseded / DriftInvalidated and bootstrap via F-5/G-10 invalidation. | summary_anchor=14-N:L58-L62 | source_atoms=N1-E008, N1-E027
- **N-001/SD-N1-003** [Δ] section=N-1 (DEEP-N-001.discovery.md:L447): **ADD** D-19..D-25 bench/validation suites bound into the ten-bench unification note. | summary_anchor=14-N:L50 | source_atoms=N1-E009
- **N-001/SD-N1-004** [Δ] section=N-1 (DEEP-N-001.discovery.md:L448): **ADD** Tier-2 SAFEDELTA/RunPod deferral and deterministic fixture scope for Tier-1. | summary_anchor=14-N:L764 | source_atoms=N1-E017
- **N-001/SD-N1-005** [Δ] section=N-1 (DEEP-N-001.discovery.md:L449): **ADD** cold-start hole list (D-16 tau, J-51 floors, L-11 OQ, 3F FP, H-43) and B-42 ~12-artifact precedent. | summary_anchor=14-N:L59-L65 | source_atoms=N1-E014–N1-E015, N1-I001
- **N-001/SD-N1-006** [Δ] section=N-1 (DEEP-N-001.discovery.md:L450): **ADD** I-23 unified-calibration binding rationale and ID-5 BOOTSTRAP provenance satisfaction. | summary_anchor=14-N:L61-L65 | source_atoms=N1-E013, N1-E029
- **N-001/SD-N1-007** [Δ] section=N-1 (DEEP-N-001.discovery.md:L451): **QUALIFY** summary consumers J-84/J-85, L-4/L-7, K-24/M-5: present in summary N1-E08..E10 but NOT restated in N-1 detail file; do not promote summary-only consumers as detail atoms. | summary_anchor=14-N:L57-L59 | source_atoms=(summary-only; not N1-E*)
- **N-001/SD-N1-008** [Δ] section=N-1 (DEEP-N-001.discovery.md:L452): **EXPAND** ambiguities: six loops still unnamed; MeasurementPanel authoring decision; ~30 fan-in list. | summary_anchor=14-N:L73-L78 | source_atoms=N1-A001–N1-A004
- **N-001/SD-N3-001** [Δ] section=N-3 (DEEP-N-001.discovery.md:L458): **ADD** full Kind set TrialCountApplied / DeflationBlocked / HoldoutBudgetSpent (summary only showed DeflationBlocked in places). | summary_anchor=14-N:L132, L713 | source_atoms=N3-E022, N0-E054
- **N-001/SD-N3-002** [Δ] section=N-3 (DEEP-N-001.discovery.md:L459): **ADD** Tier-1 wire set is only 3F + J-57 + J-97 first; other loops are in the harden list but Tier-2 for full Thresholdout. | summary_anchor=14-N:L126-L133 | source_atoms=N3-E014–N3-E015, N3-I001
- **N-001/SD-N3-003** [Δ] section=N-3 (DEEP-N-001.discovery.md:L460): **ADD** agent-unresettable ledger property and re-KEY-not-reauthor ban on DSR/PBO/MinBTL. | summary_anchor=14-N:L116-L118 | source_atoms=N3-E004–N3-E005, N3-E009
- **N-001/SD-N3-004** [Δ] section=N-3 (DEEP-N-001.discovery.md:L461): **ADD** H-47 partial H-19 reuse note and TB-1 finance-only status. | summary_anchor=14-N:L137-L138 | source_atoms=N3-E010, N3-A002
- **N-001/SD-N3-005** [Δ] section=N-3 (DEEP-N-001.discovery.md:L462): **EXPAND** missing N threshold, candidate-family schema, and DP parameters as open. | summary_anchor=14-N:L135-L138 | source_atoms=N3-A001–N3-A003
- **N-001/SD-N4-001** [Δ] section=N-4 (DEEP-N-001.discovery.md:L468): **ADD** exact expensive-path list: D-11 registry / F-4 tests / L-5 do-tests / L-4 resampling N=5. | summary_anchor=14-N:L149-L152 | source_atoms=N4-E007
- **N-001/SD-N4-002** [Δ] section=N-4 (DEEP-N-001.discovery.md:L469): **ADD** Tier-2 deferral of L-2 CGG memo consultation after J-83 proves identity+invalidation. | summary_anchor=14-N:L154, L164-L167 | source_atoms=N4-E019, N4-E024
- **N-001/SD-N4-003** [Δ] section=N-4 (DEEP-N-001.discovery.md:L470): **ADD** I-8 dead-end AST hashing as negative-space twin; J-97 not-yet receipt cache. | summary_anchor=14-N:L140-L167 | source_atoms=N4-E015, N4-E017
- **N-001/SD-N4-004** [Δ] section=N-4 (DEEP-N-001.discovery.md:L471): **ADD** full receipt_memo Kind set Hit / Miss / Revalidated from index; detail acceptance only exercises Hit. | summary_anchor=14-N:L160-L161 | source_atoms=N4-E026, N4-E028, N4-A003
- **N-001/SD-N4-005** [Δ] section=N-4 (DEEP-N-001.discovery.md:L472): **ADD** blake3 claim-identity hashing stub ban and "no fifth claim type" residual vs L-8 half-done carrier. | summary_anchor=14-N:L164-L167 | source_atoms=N4-E013, N4-E018, N4-E025
- **N-001/SD-N4-006** [Δ] section=N-4 (DEEP-N-001.discovery.md:L473): **EXPAND** schema/TTL/eligibility ambiguities still open after detail read. | summary_anchor=14-N:L164-L167 | source_atoms=N4-A001–N4-A002
- **N-001/SD-N9-001** [Δ] section=N-9 (DEEP-N-001.discovery.md:L479): **ADD** Tier-1 vs Tier-2 split: core + role/verifier/ranker now; memory/case/DAG adapters deferred. | summary_anchor=14-N:L274-L298 | source_atoms=N9-E010–N9-E011
- **N-001/SD-N9-002** [Δ] section=N-9 (DEEP-N-001.discovery.md:L480): **ADD** per-adapter mechanism blurbs (CANTANTE drop-role, DAG winner-vs-vetoed, >1-sigma E-6, decorrelation H-40, etc.). | summary_anchor=14-N:L280-L287 | source_atoms=N9-E004–N9-E005
- **N-001/SD-N9-003** [Δ] section=N-9 (DEEP-N-001.discovery.md:L481): **ADD** N-3 deflation composition when ablation feeds promotion; K-18 same-shape note. | summary_anchor=14-N:L289 | source_atoms=N9-E008–N9-E009
- **N-001/SD-N9-004** [Δ] section=N-9 (DEEP-N-001.discovery.md:L482): **FLAG** target param includes `rule` but Pattern-2 Kind list omits Rule; eight adapters vs six targets still unresolved. | summary_anchor=14-N:L295-L298 | source_atoms=N9-I001, N9-A002
- **N-001/SD-N9-005** [Δ] section=N-9 (DEEP-N-001.discovery.md:L483): **EXPAND** replay isolation/seed/delta metric still unspecified in detail. | summary_anchor=14-N:L295-L297 | source_atoms=N9-A001
- **N-001/SD-N11-001** [Δ] section=N-11 (DEEP-N-001.discovery.md:L489): **ADD** L-16 already-shipped residual list (compact_session replaced, mutate rollback, E-25/H-6/3E/H-50 as strategies; L-18 corollaries). | summary_anchor=14-N:L326-L350 | source_atoms=N11-E007–N11-E008
- **N-001/SD-N11-002** [Δ] section=N-11 (DEEP-N-001.discovery.md:L490): **ADD** Tier-1 surfaces = canvas + I-39 (+ optional third); Tier-2 = K-24/M-3/K-22. | summary_anchor=14-N:L329-L345 | source_atoms=N11-E014–N11-E015, N11-A003
- **N-001/SD-N11-003** [Δ] section=N-11 (DEEP-N-001.discovery.md:L491): **ADD** frontend rule: verify canvas consumption, do not re-wire; no new SSB type. | summary_anchor=14-N:L342-L345 | source_atoms=N11-E017, N11-E022
- **N-001/SD-N11-004** [Δ] section=N-11 (DEEP-N-001.discovery.md:L492): **ADD** recover_stubs legacy-only EOL + H-49 as THE log store, user-confirmed ND-2. | summary_anchor=14-N:L338-L340 | source_atoms=N11-E006, N11-E012–N11-E013
- **N-001/SD-N11-005** [Δ] section=N-11 (DEEP-N-001.discovery.md:L493): **CONFIRM** canvas-as-one-surface interpretation of four unit IDs. | summary_anchor=14-N:L349 | source_atoms=N11-I001
- **N-001/SD-N11-006** [Δ] section=N-11 (DEEP-N-001.discovery.md:L494): **EXPAND** plugin schema / Kani property / retirement migration still open. | summary_anchor=14-N:L347-L350 | source_atoms=N11-A001–N11-A002
- **N-001/SD-N12-001** [Δ] section=N-12 (DEEP-N-001.discovery.md:L500): **CORRECT/RESOLVE** summary "eight vs seven": detail also says eight collapses with seven labels (a)..(g) because (a) merges W-8+W-26; no eighth product named. | summary_anchor=14-N:L24, L408-L409, L828 | source_atoms=N12-I001, N12-E002–N12-E003
- **N-001/SD-N12-002** [Δ] section=N-12 (DEEP-N-001.discovery.md:L501): **ADD** exact category counts (13/9/14/4) and "classified once IS corpus row IS case edge IS stagnation signal". | summary_anchor=14-N:L366-L372 | source_atoms=N12-E003–N12-E004
- **N-001/SD-N12-003** [Δ] section=N-12 (DEEP-N-001.discovery.md:L502): **ADD** Tier-1 ship set (a,c,f) vs Tier-2 defer set (b,d,e,g). | summary_anchor=14-N:L352-L412 | source_atoms=N12-E017–N12-E018
- **N-001/SD-N12-004** [Δ] section=N-12 (DEEP-N-001.discovery.md:L503): **ADD** RD-1 read-never-re-derive for L-7 abstain composition; training-data free for J-25/I-12. | summary_anchor=14-N:L374-L378 | source_atoms=N12-E005, N12-E008
- **N-001/SD-N12-005** [Δ] section=N-12 (DEEP-N-001.discovery.md:L504): **ADD** drift core primitives list (KL/delta-rho/IQR/perplexity/DDM + Page-Hinkley/DDM windowed core) with policies kept distinct. | summary_anchor=14-N:L394-L398 | source_atoms=N12-E011
- **N-001/SD-N12-006** [Δ] section=N-12 (DEEP-N-001.discovery.md:L505): **ADD** FSM models named (3I DST FsmReferenceModel, J-68 ElaiFSM.tla, code transition table) and LSP B-37-like pool rationale. | summary_anchor=14-N:L399-L405 | source_atoms=N12-E012–N12-E014
- **N-001/SD-N12-007** [Δ] section=N-12 (DEEP-N-001.discovery.md:L506): **ADD** J-109 currently guards E-5/I-12/J-97 but not C-47/C-49 (explicit gap closed by (d)). | summary_anchor=14-N:L388-L392 | source_atoms=N12-E010
- **N-001/SD-N12-008** [Δ] section=N-12 (DEEP-N-001.discovery.md:L507): **EXPAND** enum mapping, LSP ops, adapter boundaries, FSM equality criterion still open. | summary_anchor=14-N:L407-L412 | source_atoms=N12-A001–N12-A004

### N-002 summary deltas

- **N-002/SD-N2-001** [Δ] section=N-2 (DEEP-N-002.discovery.md:L465): **ADD** exact fail Kind list `DanglingEdge\ | summary_anchor=OrphanProducer\ | source_atoms=DuplicateConcept\
- **N-002/SD-N2-002** [Δ] section=N-2 (DEEP-N-002.discovery.md:L466): **ADD** PRE_GATE / paper-to-plan phase-5 + CI over `docs/thefinalglance/`; ND-3 edge-row emitter from feature-map upsert; Tier-2 ML deferral. | summary_anchor=14-N:L97-L110 | source_atoms=N2-E011, N2-E018–N2-E020
- **N-002/SD-N2-003** [Δ] section=N-2 (DEEP-N-002.discovery.md:L467): **ADD** stub bans (no opt-in PASS on missing edges, no prose-only edges) and R3/R6/R8 grounding. | summary_anchor=14-N:L100-L110 | source_atoms=N2-E023–N2-E025, N2-E030
- **N-002/SD-N2-004** [Δ] section=N-2 (DEEP-N-002.discovery.md:L468): **EXPAND** orphan-producer ambiguity remains; similarity thresholds still open at detail grain. | summary_anchor=14-N:L107-L110 | source_atoms=N2-A001–N2-A003
- **N-002/SD-N5-001** [Δ] section=N-5 (DEEP-N-002.discovery.md:L474): **ADD** exact router enumeration and frozen precedence string safety-gates > cost-ceiling > abstention > escalation > modality > serving-tier. | summary_anchor=14-N:L169-L196 | source_atoms=N5-E003–N5-E004
- **N-002/SD-N5-002** [Δ] section=N-5 (DEEP-N-002.discovery.md:L475): **ADD** Tier-2 deferral of J-66 routing-order monitor; R8 table in gate-manifest; N-7 canvas/RRA explain-row as frontend_pair. | summary_anchor=14-N:L185-L196 | source_atoms=N5-E015–N5-E017
- **N-002/SD-N5-003** [Δ] section=N-5 (DEEP-N-002.discovery.md:L476): **QUALIFY** summary conflict examples that name E-39: numbered source does not list E-39 among the ten routers; only L-6 vs J-73 is the trigger example. | summary_anchor=14-N:L174-L175, L193 | source_atoms=N5-I001, N5-E022
- **N-002/SD-N5-004** [Δ] section=N-5 (DEEP-N-002.discovery.md:L477): **ADD** intensity-tuple vs B-28 edit-format leg tension and unresolved task-difficulty measure. | summary_anchor=14-N:L180-L196 | source_atoms=N5-A001–N5-A002
- **N-002/SD-N6-001** [Δ] section=N-6 (DEEP-N-002.discovery.md:L483): **CORRECT** any "five scales" framing: numbered source consistently counts and enumerates **six** scales. | summary_anchor=14-N:L222-L224 | source_atoms=N6-C001, N6-E003–N6-E005
- **N-002/SD-N6-002** [Δ] section=N-6 (DEEP-N-002.discovery.md:L484): **ADD** full carry chain including H-53 citation and 4D EvidenceLevel; Tier-1 vs Tier-2 split (three existing gates now; K-23/M-2 later). | summary_anchor=14-N:L201-L224 | source_atoms=N6-E005–N6-E012
- **N-002/SD-N6-003** [Δ] section=N-6 (DEEP-N-002.discovery.md:L485): **ADD** optional `+admission_scanned` and map-don't-merge F-6 type barrier as hard stub_ban. | summary_anchor=14-N:L214-L224 | source_atoms=N6-E004, N6-E016–N6-E019
- **N-002/SD-N7-001** [Δ] section=N-7 (DEEP-N-002.discovery.md:L491): **ADD** full producer list including N-5/N-6/N-8; Tier-1 renders only 2-3 families (J-83/L-7/N-1); M-3/full canvas deferred. | summary_anchor=14-N:L226-L248 | source_atoms=N7-E007, N7-E011–N7-E012
- **N-002/SD-N7-002** [Δ] section=N-7 (DEEP-N-002.discovery.md:L492): **ADD** `+verdict_rendered` Kind list PrComment/ChatAck/CanvasBadge/BriefRow/OrgRollup and mandatory `cross_lang_diff` PASS. | summary_anchor=14-N:L240-L243 | source_atoms=N7-E014, N7-E018–N7-E019
- **N-002/SD-N7-003** [Δ] section=N-7 (DEEP-N-002.discovery.md:L493): **ADD** MODERATE grounding (summary carries it; detail confirms W-7 only). | summary_anchor=14-N:L228 | source_atoms=N7-E001
- **N-002/SD-N8-001** [Δ] section=N-8 (DEEP-N-002.discovery.md:L499): **ADD** explicit five view definitions, lifecycle mutant-corpus requirement, N-2 as sixth grain, and ~40-gate back-fill deferral. | summary_anchor=14-N:L250-L272 | source_atoms=N8-E004–N8-E015
- **N-002/SD-N8-002** [Δ] section=N-8 (DEEP-N-002.discovery.md:L500): **ADD/FLAG** expect_ssb Kind list (four named kinds) vs five views; checker path is a placeholder. | summary_anchor=14-N:L267-L272 | source_atoms=N8-E022, N8-A002–N8-A003, N8-C001
- **N-002/SD-N8-003** [Δ] section=N-8 (DEEP-N-002.discovery.md:L501): **ADD** GD-2 journal-recomputable headline and ND-4 measure-through-N-1 ban on unmeasured claims. | summary_anchor=14-N:L260-L272 | source_atoms=N8-E011, N8-E020
- **N-002/SD-N10-001** [Δ] section=N-10 (DEEP-N-002.discovery.md:L507): **ADD** Tier-1 consumer set (4D + J-84 via N-5 + J-79) and explicit Tier-2 deferral of J-39. | summary_anchor=14-N:L300-L324 | source_atoms=N10-E010–N10-E011
- **N-002/SD-N10-002** [Δ] section=N-10 (DEEP-N-002.discovery.md:L508): **ADD** ambiguity vs novelty semantics ("don't understand" vs "off our map") and "not just a chip" clarify behavior. | summary_anchor=14-N:L313-L315 | source_atoms=N10-E006
- **N-002/SD-N10-003** [Δ] section=N-10 (DEEP-N-002.discovery.md:L509): **FLAG** expect_ssb "no new type" vs produces shared types; score cutoffs still open. | summary_anchor=14-N:L321-L324 | source_atoms=N10-C001, N10-A001–N10-A002
- **N-002/SD-N13-001** [Δ] section=N-13 (DEEP-N-002.discovery.md:L515): **ADD** complete Kind list SchemaEnforced/GrammarEnforced/RepairedThenValidated/RefusedFailClosed and capability field list of seven flags. | summary_anchor=14-N:L431-L464 | source_atoms=N13-E011, N13-E019
- **N-002/SD-N13-002** [Δ] section=N-13 (DEEP-N-002.discovery.md:L516): **ADD** four emission seams including AuditFinding; contract test arms; DETERMINISTIC_CHECK zero reimplemented validators; `ELAI_OUTPUT_BOUNDARY_LEGACY`. | summary_anchor=14-N:L453-L458 | source_atoms=N13-E015, N13-E025–N13-E029
- **N-002/SD-N13-003** [Δ] section=N-13 (DEEP-N-002.discovery.md:L517): **ADD** open questions: type enum, catalog owner, user-confirmed ND-2 retirement; WAVE 4-adjacent vs summary Wave 9. | summary_anchor=14-N:L433, L459-L464 | source_atoms=N13-A001–N13-A003, N13-I001
- **N-002/SD-N13-004** [Δ] section=N-13 (DEEP-N-002.discovery.md:L518): **EXPAND** security riders I-34/H-13/J-53 as stub_bans with blake3-only pins. | summary_anchor=14-N:L447-L450 | source_atoms=N13-E012, N13-E021
- **N-002/SD-N14-001** [Δ] section=N-14 (DEEP-N-002.discovery.md:L524): **ADD** CandidateMgmt event class, exact timing fields, `ELAI_EDIT_KIND_LEGACY`, and advisory-only rg ban on block/reject/deny. | summary_anchor=14-N:L466-L502 | source_atoms=N14-E008, N14-E014–N14-E015, N14-E024–N14-E025
- **N-002/SD-N14-002** [Δ] section=N-14 (DEEP-N-002.discovery.md:L525): **ADD** persona mapping (structure-churn→Architect/Simplifier; wrong-structure-tuning→Contrarian/rethink) and synthesis/tuning lane routing. | summary_anchor=14-N:L476-L479 | source_atoms=N14-E011–N14-E012
- **N-002/SD-N14-003** [Δ] section=N-14 (DEEP-N-002.discovery.md:L526): **ADD** adversarial n=1/I-38 floor caveat and open questions (attempt-row reuse, literal normalization, per-step vs cache). | summary_anchor=14-N:L496-L502 | source_atoms=N14-E029, N14-C001, N14-A001–N14-A003
- **N-002/SD-N14-004** [Δ] section=N-14 (DEEP-N-002.discovery.md:L527): **FLAG** WAVE 3/4-adjacent in source vs Wave 9 in summary matrix. | summary_anchor=14-N:L468, L777 | source_atoms=N14-I001
- **N-002/SD-N15-001** [Δ] section=N-15 (DEEP-N-002.discovery.md:L533): **ADD** Kind list Stable/FramingUnstable/AssertionPulled; Stable-unknown degradation for single-sample; `ELAI_FRAMING_INVARIANCE_LEGACY`. | summary_anchor=14-N:L504-L534 | source_atoms=N15-E012, N15-E022, N15-E024
- **N-002/SD-N15-002** [Δ] section=N-15 (DEEP-N-002.discovery.md:L534): **ADD** two-halves GD-5 honesty framing, D-14 probe cost note, optional N-7 render note default. | summary_anchor=14-N:L508-L534 | source_atoms=N15-E007, N15-A002–N15-A003
- **N-002/SD-N15-003** [Δ] section=N-15 (DEEP-N-002.discovery.md:L535): **ADD** N-8 governance view input and G-5/N-8 tile; WAVE 5-adjacent vs summary Wave 9. | summary_anchor=14-N:L520-L524, L778 | source_atoms=N15-E011, N15-E018, N15-I001
- **N-002/SD-N16-001** [Δ] section=N-16 (DEEP-N-002.discovery.md:L541): **ADD** PROOF_SPINE/PS-2 honesty-check role; Kind lists for evaluator_closure_swept and grounding_ratio_measured; AdjudicatorRitual reuse on governance_coverage_measured; baseline+2. | summary_anchor=14-N:L536-L581 | source_atoms=N16-E002, N16-E020
- **N-002/SD-N16-002** [Δ] section=N-16 (DEEP-N-002.discovery.md:L542): **ADD** GD-4 observe-only until FP bound on >=3 runs; Tier-2 back-fill of historical version-hashes and gate-amend-rate deferral. | summary_anchor=14-N:L563-L581 | source_atoms=N16-E013–N16-E014
- **N-002/SD-N16-003** [Δ] section=N-16 (DEEP-N-002.discovery.md:L543): **ADD** frontend triple: lambda gauge + quarantine-queue count + ritual-entropy band; Reinstated Kind without release criteria (flag). | summary_anchor=14-N:L569-L581 | source_atoms=N16-E016, N16-A004
- **N-002/SD-N16-004** [Δ] section=N-16 (DEEP-N-002.discovery.md:L544): **ADD** D-17 as brake escalation target and R4 as exogenous-anchor definition. | summary_anchor=14-N:L567 | source_atoms=N16-E009, N16-E012

---

## Summary record atoms (`14-N-SUMMARY_INDEX.discovery.md`)

Orientation pass (did not read detail files). Deep ledgers supersede where they conflict; summary-only atoms remain under this namespace.

### SUMMARY inventory (ID'd atoms, 385 records)

#### `N-1 — measurement spine as ONE subsystem [KEYSTONE][BLOCKING]` (27 records)

- **SUMMARY/N1-E01** [E] anchor=`14-N:L49` (14-N-SUMMARY_INDEX.discovery.md:L49): Declare measurement as one named subsystem rather than scattered benches and “K slivers” (`L24-L26`).
- **SUMMARY/N1-E02** [E] anchor=`14-N:L50` (14-N-SUMMARY_INDEX.discovery.md:L50): Bind all ten named bench families to the shared spine: G-6 SAFEDELTA, J-34 context-knee, K-15 reasoning-knee, J-107 longcot, L-10 hallucination-delta, K-18 case-reuse, I-38 QDELTA, I-27 convergence, L-20 supply, and J-110 timeliness (`L26-L28`).
- **SUMMARY/N1-E03** [E] anchor=`14-N:L51` (14-N-SUMMARY_INDEX.discovery.md:L51): Use one corpus family rooted in J-56/E-54/L-20 (`L28-L30`).
- **SUMMARY/N1-E04** [E] anchor=`14-N:L52` (14-N-SUMMARY_INDEX.discovery.md:L52): Treat L-20 supply as a feeder, not a sibling bench (`L29-L30`).
- **SUMMARY/N1-E05** [E] anchor=`14-N:L53` (14-N-SUMMARY_INDEX.discovery.md:L53): Treat J-110 mutators as feeders, not sibling benches (`L29-L30`).
- **SUMMARY/N1-E06** [E] anchor=`14-N:L54` (14-N-SUMMARY_INDEX.discovery.md:L54): Use one runner family with the K-15/I-27 shape (`L30`).
- **SUMMARY/N1-E07** [E] anchor=`14-N:L55` (14-N-SUMMARY_INDEX.discovery.md:L55): Generalize the J-93-F rules-only outcome-label fold to the six promotion loops (`L30-L31`).
- **SUMMARY/N1-E08** [E] anchor=`14-N:L56` (14-N-SUMMARY_INDEX.discovery.md:L56): Feed J-84/J-85 knee selection from the shared measurements (`L28-L29`).
- **SUMMARY/N1-E09** [E] anchor=`14-N:L57` (14-N-SUMMARY_INDEX.discovery.md:L57): Feed L-4/L-7 promotion decisions from the shared measurements (`L28-L29`).
- **SUMMARY/N1-E10** [E] anchor=`14-N:L58` (14-N-SUMMARY_INDEX.discovery.md:L58): Feed K-24/M-5 pricing from the shared measurements (`L28-L29`).
- **SUMMARY/N1-E11** [E] anchor=`14-N:L59` (14-N-SUMMARY_INDEX.discovery.md:L59): Ship a default measurement bootstrap pack for fresh-install cold start (`L31-L33`).
- **SUMMARY/N1-E12** [E] anchor=`14-N:L60` (14-N-SUMMARY_INDEX.discovery.md:L60): Mark bootstrap defaults with `provenance=BOOTSTRAP` (`L31-L32`).
- **SUMMARY/N1-E13** [E] anchor=`14-N:L61` (14-N-SUMMARY_INDEX.discovery.md:L61): Include bootstrap values for D-16 tau, J-34 knees, K-15 knees, and OQ thresholds (`L32-L33`).
- **SUMMARY/N1-E14** [E] anchor=`14-N:L62` (14-N-SUMMARY_INDEX.discovery.md:L62): Hard-supersede bootstrap values when local measurements become available (`L32-L33`).
- **SUMMARY/N1-E15** [E] anchor=`14-N:L63` (14-N-SUMMARY_INDEX.discovery.md:L63): Use the W-5 `ModelCapabilityProfile` / I-51 `models.toml` as the **only** calibration sink (`L34-L36`).
- **SUMMARY/N1-E16** [E] anchor=`14-N:L64` (14-N-SUMMARY_INDEX.discovery.md:L64): Reuse B-42’s default-artifact-pack shape for the bootstrap rather than creating another pack mechanism (`L34-L36`).
- **SUMMARY/N1-E17** [E] [SURFACE] anchor=`14-N:L65` (14-N-SUMMARY_INDEX.discovery.md:L65): Expose the family through the existing CLI seam `elai bench <any-family>` / `elai bench *` (`L37-L40`).
- **SUMMARY/N1-E18** [E] anchor=`14-N:L66` (14-N-SUMMARY_INDEX.discovery.md:L66): A bench run writes both a corpus row and a calibration curve into the one profile (`L40-L41`).
- **SUMMARY/N1-E19** [E] anchor=`14-N:L67` (14-N-SUMMARY_INDEX.discovery.md:L67): Replacing a bootstrap default with local measurement appends `+calibration_bootstrap_superseded` to the SSB (`L40-L42`).
- **SUMMARY/N1-E20** [E] anchor=`14-N:L68` (14-N-SUMMARY_INDEX.discovery.md:L68): A bench writing a knee to any store other than the profile must fail under ND-4 (`L41-L42`).
- **SUMMARY/N1-E21** [E] anchor=`14-N:L69` (14-N-SUMMARY_INDEX.discovery.md:L69): Integrate corpus/bench work in `crates/knowledge` and profile/label-fold work in `crates/orchestrator` (`L37-L38`).
- **SUMMARY/N1-E22** [E] [SURFACE] anchor=`14-N:L70` (14-N-SUMMARY_INDEX.discovery.md:L70): Provide a `MeasurementPanel` tile sourced from the spine index, unless G-5 already surfaces it and the N/A decision is justified (`L38-L39`).
- **SUMMARY/N1-E23** [E] anchor=`14-N:L71` (14-N-SUMMARY_INDEX.discovery.md:L71): Do not implement another bench runner; existing benches bind to the one runner and profile (`L34-L36`).
- **SUMMARY/N1-A01** [A] anchor=`14-N:L75` (14-N-SUMMARY_INDEX.discovery.md:L75): The “~30 features across 6 letters” are not individually listed in the summary (`L24-L25`).
- **SUMMARY/N1-A02** [A] anchor=`14-N:L76` (14-N-SUMMARY_INDEX.discovery.md:L76): The six promotion loops generalized by J-93-F are not enumerated as a closed set (`L30-L31`).
- **SUMMARY/N1-A03** [A] anchor=`14-N:L77` (14-N-SUMMARY_INDEX.discovery.md:L77): OQ threshold ownership is not identified by unit in this section (`L32`).
- **SUMMARY/N1-A04** [A] [SURFACE] anchor=`14-N:L78` (14-N-SUMMARY_INDEX.discovery.md:L78): Whether the `MeasurementPanel` is mandatory depends on G-5 coverage (`L38-L39`).

#### `N-2 — corpus-grain composition validator + M-1 one-registry rule` (24 records)

- **SUMMARY/N2-E01** [E] anchor=`14-N:L84` (14-N-SUMMARY_INDEX.discovery.md:L84): Parse feature-map `EXTENDS`, `composes`, `consumes`, and `gated-by` edges as data in the ND-3 format (`L48-L50`).
- **SUMMARY/N2-E02** [E] anchor=`14-N:L85` (14-N-SUMMARY_INDEX.discovery.md:L85): Join parsed edges against plan files (`L50`).
- **SUMMARY/N2-E03** [E] anchor=`14-N:L86` (14-N-SUMMARY_INDEX.discovery.md:L86): Join parsed edges against `elai.map` symbols (`L50`).
- **SUMMARY/N2-E04** [E] anchor=`14-N:L87` (14-N-SUMMARY_INDEX.discovery.md:L87): Fail on a dangling edge whose named consumed unit does not expose the named artifact (`L50-L52`).
- **SUMMARY/N2-E05** [E] anchor=`14-N:L88` (14-N-SUMMARY_INDEX.discovery.md:L88): Fail on an orphan producer (`L51-L52`).
- **SUMMARY/N2-E06** [E] anchor=`14-N:L89` (14-N-SUMMARY_INDEX.discovery.md:L89): Detect duplicate-concept collisions using name similarity (`L52-L53`).
- **SUMMARY/N2-E07** [E] anchor=`14-N:L90` (14-N-SUMMARY_INDEX.discovery.md:L90): Detect duplicate-concept collisions using shared-seam detection (`L52-L53`).
- **SUMMARY/N2-E08** [E] anchor=`14-N:L91` (14-N-SUMMARY_INDEX.discovery.md:L91): The duplicate check is intended to catch forks such as the four claim types, E-6/E-29 MAST duplication, and ten routers during authoring (`L52-L53`).
- **SUMMARY/N2-E09** [E] anchor=`14-N:L92` (14-N-SUMMARY_INDEX.discovery.md:L92): Emit the global unlock DAG (`L53-L54`).
- **SUMMARY/N2-E10** [E] anchor=`14-N:L93` (14-N-SUMMARY_INDEX.discovery.md:L93): Emit global fan-in rankings (`L53-L54`).
- **SUMMARY/N2-E11** [E] anchor=`14-N:L94` (14-N-SUMMARY_INDEX.discovery.md:L94): Mechanically derive summaries/status by composing J-93-G’s map-half derivation and I-39 journal status (`L54-L55`).
- **SUMMARY/N2-E12** [E] anchor=`14-N:L95` (14-N-SUMMARY_INDEX.discovery.md:L95): Check freshness across `SUMMARY_INDEX`, `0_INDEX`, and detail plans using the `elai_map_freshness` pattern (`L55-L56`).
- **SUMMARY/N2-E13** [E] anchor=`14-N:L96` (14-N-SUMMARY_INDEX.discovery.md:L96): Run at paper-to-plan phase 5 / planning authoring time as a CI gate (`L56-L62`).
- **SUMMARY/N2-E14** [E] anchor=`14-N:L97` (14-N-SUMMARY_INDEX.discovery.md:L97): Extend J-89 L8 per-plan DAG type-completeness to corpus scope rather than reimplementing it (`L58-L59`).
- **SUMMARY/N2-E15** [E] [SURFACE] anchor=`14-N:L98` (14-N-SUMMARY_INDEX.discovery.md:L98): Add `.claude/tools/corpus_composition_check.py` beside `wiring_contract_check.py` (`L60-L61`).
- **SUMMARY/N2-E16** [E] [SURFACE] anchor=`14-N:L99` (14-N-SUMMARY_INDEX.discovery.md:L99): Invoke it as `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` (`L63-L64`).
- **SUMMARY/N2-E17** [E] anchor=`14-N:L100` (14-N-SUMMARY_INDEX.discovery.md:L100): Seeded dangling-edge, orphan, or duplicate-concept cases must produce a nonzero exit (`L63-L65`).
- **SUMMARY/N2-E18** [E] anchor=`14-N:L101` (14-N-SUMMARY_INDEX.discovery.md:L101): A check appends `+composition_edge_checked` to the SSB (`L63-L64`).
- **SUMMARY/N2-E19** [E] anchor=`14-N:L102` (14-N-SUMMARY_INDEX.discovery.md:L102): A hand-inserted dangling edge returning exit 0 is a fail-closed R1 failure (`L64-L65`).
- **SUMMARY/N2-E20** [E] anchor=`14-N:L103` (14-N-SUMMARY_INDEX.discovery.md:L103): The M-1/ND-1 rule applies to every future letter, not only N (`L56-L57`).
- **SUMMARY/N2-E21** [E] [SURFACE] anchor=`14-N:L104` (14-N-SUMMARY_INDEX.discovery.md:L104): No runtime frontend is required because this is an authoring-time CI surface (`L60-L62`).
- **SUMMARY/N2-A01** [A] anchor=`14-N:L108` (14-N-SUMMARY_INDEX.discovery.md:L108): “orphan producer (a unit with >=1 consumer in NO letter)” is grammatically ambiguous; likely it means a producer with no consumer in any letter, but that interpretation is not promoted to explicit fact here (`L51-L52`).
- **SUMMARY/N2-A02** [A] anchor=`14-N:L109` (14-N-SUMMARY_INDEX.discovery.md:L109): Similarity thresholds, collision adjudication, edge schema, output filenames, and SSB payload fields are not specified in the summary.
- **SUMMARY/N2-A03** [A] anchor=`14-N:L110` (14-N-SUMMARY_INDEX.discovery.md:L110): J’s documented drift stops at J-91, but the exact repair/migration behavior is not stated (`L48-L49`).

#### `N-3 — generalized anti-selection-bias promotion primitives` (20 records)

- **SUMMARY/N3-E01** [E] anchor=`14-N:L116` (14-N-SUMMARY_INDEX.discovery.md:L116): Generalize H-17’s persisted trial ledger from finance-only use to all promotion domains (`L69-L75`).
- **SUMMARY/N3-E02** [E] anchor=`14-N:L117` (14-N-SUMMARY_INDEX.discovery.md:L117): Re-key H-17 from a finance-candidate key to `(candidate-family)` (`L76-L78`).
- **SUMMARY/N3-E03** [E] anchor=`14-N:L118` (14-N-SUMMARY_INDEX.discovery.md:L118): Generalize H-18’s deflated-significance machinery, including DSR/PBO/MinBTL, without reimplementing it (`L69-L78`).
- **SUMMARY/N3-E04** [E] anchor=`14-N:L119` (14-N-SUMMARY_INDEX.discovery.md:L119): H-18 remains fail-closed when trial count N is missing (`L69-L70`).
- **SUMMARY/N3-E05** [E] anchor=`14-N:L120` (14-N-SUMMARY_INDEX.discovery.md:L120): Generalize H-19’s reusable-holdout differential-privacy budget (`L69-L75`).
- **SUMMARY/N3-E06** [E] anchor=`14-N:L121` (14-N-SUMMARY_INDEX.discovery.md:L121): Route every promotion gate through these primitives using the ID-6 ledger hook (`L73-L77`).
- **SUMMARY/N3-E07** [E] anchor=`14-N:L122` (14-N-SUMMARY_INDEX.discovery.md:L122): Cover 3F rule lifecycle `SHADOW -> ACTIVE` (`L71-L72`).
- **SUMMARY/N3-E08** [E] anchor=`14-N:L123` (14-N-SUMMARY_INDEX.discovery.md:L123): Cover J-57 prompt promotion (`L71-L72`).
- **SUMMARY/N3-E09** [E] anchor=`14-N:L124` (14-N-SUMMARY_INDEX.discovery.md:L124): Cover J-97 case-reuse fitness promotion (`L71-L72`).
- **SUMMARY/N3-E10** [E] anchor=`14-N:L125` (14-N-SUMMARY_INDEX.discovery.md:L125): Cover B-23/H-45 config promotion (`L72`).
- **SUMMARY/N3-E11** [E] anchor=`14-N:L126` (14-N-SUMMARY_INDEX.discovery.md:L126): Cover I-4/I-25 candidate ladders (`L72`).
- **SUMMARY/N3-E12** [E] anchor=`14-N:L127` (14-N-SUMMARY_INDEX.discovery.md:L127): Cover L-11 OQ threshold promotion (`L72`).
- **SUMMARY/N3-E13** [E] anchor=`14-N:L128` (14-N-SUMMARY_INDEX.discovery.md:L128): Give each reusable J-56/L-20/D-6 evaluation corpus a Thresholdout budget (`L75`).
- **SUMMARY/N3-E14** [E] anchor=`14-N:L129` (14-N-SUMMARY_INDEX.discovery.md:L129): Use N-1 outcome labels as the ground truth against which deflation corrects (`L76-L77`).
- **SUMMARY/N3-E15** [E] anchor=`14-N:L130` (14-N-SUMMARY_INDEX.discovery.md:L130): Integrate at `crates/orchestrator` promotion sites for rules, prompts, cases, and config archives (`L79-L80`).
- **SUMMARY/N3-E16** [E] [SURFACE] anchor=`14-N:L131` (14-N-SUMMARY_INDEX.discovery.md:L131): Add a deflation column to existing promotion drawers, or explicitly justify N/A (`L79-L80`).
- **SUMMARY/N3-E17** [E] anchor=`14-N:L132` (14-N-SUMMARY_INDEX.discovery.md:L132): A real promotion with N below threshold is held and emits `+promotion_deflated(DeflationBlocked)` (`L81-L83`).
- **SUMMARY/N3-E18** [E] anchor=`14-N:L133` (14-N-SUMMARY_INDEX.discovery.md:L133): Any promotion that ignores trial count and flips ACTIVE must fail (`L82-L83`).
- **SUMMARY/N3-A01** [A] anchor=`14-N:L137` (14-N-SUMMARY_INDEX.discovery.md:L137): The N threshold, privacy accounting parameters, and what constitutes each `candidate-family` are not stated.
- **SUMMARY/N3-A02** [A] anchor=`14-N:L138` (14-N-SUMMARY_INDEX.discovery.md:L138): H-47 partially reuses H-19, but required migration of that partial path is not specified (`L72-L73`).

#### `N-4 — one Claim/Belief object + receipt memo` (21 records)

- **SUMMARY/N4-E01** [E] anchor=`14-N:L144` (14-N-SUMMARY_INDEX.discovery.md:L144): Unify L-1 `ClaimSet{ClaimKind, span, refs}`, J-83 `BeliefState` entries, K-12 `critical_atoms`, and GRC `type:reference` grounding tags as one claim/belief object (`L87-L91`).
- **SUMMARY/N4-E02** [E] anchor=`14-N:L145` (14-N-SUMMARY_INDEX.discovery.md:L145): Make L-1 typed claims be J-83 entries, completing the carrier already threaded by L-8 (`L89-L91`).
- **SUMMARY/N4-E03** [E] anchor=`14-N:L146` (14-N-SUMMARY_INDEX.discovery.md:L146): Treat the J-83 gate receipt as the same evidence that L-3 round-trips (`L90-L91`).
- **SUMMARY/N4-E04** [E] anchor=`14-N:L147` (14-N-SUMMARY_INDEX.discovery.md:L147): Map K-12 `must_preserve` to `has-receipt` (`L91`).
- **SUMMARY/N4-E05** [E] anchor=`14-N:L148` (14-N-SUMMARY_INDEX.discovery.md:L148): Have J-21 re-derive the same object against commit and AST identity (`L91`).
- **SUMMARY/N4-E06** [E] anchor=`14-N:L149` (14-N-SUMMARY_INDEX.discovery.md:L149): Before an expensive checker/resample, look up claim identity in the J-96 case graph (`L92-L94`).
- **SUMMARY/N4-E07** [E] anchor=`14-N:L150` (14-N-SUMMARY_INDEX.discovery.md:L150): Apply C-30 `OracleAnswerCache` keying discipline to the receipt memo (`L92-L93`).
- **SUMMARY/N4-E08** [E] anchor=`14-N:L151` (14-N-SUMMARY_INDEX.discovery.md:L151): On a memo hit, cheaply revalidate identity through J-21 instead of rerunning the expensive checker (`L93-L94`).
- **SUMMARY/N4-E09** [E] anchor=`14-N:L152` (14-N-SUMMARY_INDEX.discovery.md:L152): On a memo miss, run the check and mint a new J-96 node (`L93-L95`).
- **SUMMARY/N4-E10** [E] anchor=`14-N:L153` (14-N-SUMMARY_INDEX.discovery.md:L153): Use the J-96 case graph as the receipt cache (`L94-L95`).
- **SUMMARY/N4-E11** [E] anchor=`14-N:L154` (14-N-SUMMARY_INDEX.discovery.md:L154): Have both the J-83 reasoning gate and L-2 CGG consult that cache (`L94-L95`).
- **SUMMARY/N4-E12** [E] anchor=`14-N:L155` (14-N-SUMMARY_INDEX.discovery.md:L155): Invalidate memo eligibility through F-5’s staleness cascade when code changes (`L97-L103`).
- **SUMMARY/N4-E13** [E] anchor=`14-N:L156` (14-N-SUMMARY_INDEX.discovery.md:L156): Generalize C-30 rather than implementing another cache (`L97-L98`).
- **SUMMARY/N4-E14** [E] anchor=`14-N:L157` (14-N-SUMMARY_INDEX.discovery.md:L157): Integrate the unified object in `crates/orchestrator` `ssb::payloads::BeliefState` and L-2/J-83 gate call sites (`L99-L100`).
- **SUMMARY/N4-E15** [E] [SURFACE] anchor=`14-N:L158` (14-N-SUMMARY_INDEX.discovery.md:L158): Add a memo-hit indicator to existing belief/claim drawers, or justify N/A (`L99-L100`).
- **SUMMARY/N4-E16** [E] anchor=`14-N:L159` (14-N-SUMMARY_INDEX.discovery.md:L159): First occurrence of a claim on unchanged code pays the checker and mints a J-96 node (`L101-L102`).
- **SUMMARY/N4-E17** [E] anchor=`14-N:L160` (14-N-SUMMARY_INDEX.discovery.md:L160): Second occurrence on unchanged code emits `+receipt_memo(Hit)` and skips the checker (`L101-L102`).
- **SUMMARY/N4-E18** [E] anchor=`14-N:L161` (14-N-SUMMARY_INDEX.discovery.md:L161): A changed-code claim returning a stale memo hit must fail (`L102-L103`).
- **SUMMARY/N4-E19** [E] anchor=`14-N:L162` (14-N-SUMMARY_INDEX.discovery.md:L162): The design is specifically intended to reduce reasoning verification cost on a user’s repository and especially on the checker-poor local 7B lane (`L94-L96`).
- **SUMMARY/N4-A01** [A] anchor=`14-N:L166` (14-N-SUMMARY_INDEX.discovery.md:L166): The final unified schema and exact mapping for `abstain` and DKG provenance are not fully enumerated.
- **SUMMARY/N4-A02** [A] anchor=`14-N:L167` (14-N-SUMMARY_INDEX.discovery.md:L167): Claim identity composition, cache TTL/eviction, checker eligibility, and “cheap” revalidation cost bounds are unspecified.

#### `N-5 — policy / intensity arbitration plane` (20 records)

- **SUMMARY/N5-E01** [E] anchor=`14-N:L173` (14-N-SUMMARY_INDEX.discovery.md:L173): Arbitrate ten existing routers without reimplementing them: D-10 grounding, J-84 effort, J-55 modality, H-7 role, E-20 local/cloud, J-48 serving, J-54 free-pool, L-6 curiosity, J-32 front-door, and J-73 should-retrieve (`L107-L118`).
- **SUMMARY/N5-E02** [E] anchor=`14-N:L174` (14-N-SUMMARY_INDEX.discovery.md:L174): Resolve the J-84 escalation versus E-39 cost-block conflict explicitly (`L109-L110`).
- **SUMMARY/N5-E03** [E] anchor=`14-N:L175` (14-N-SUMMARY_INDEX.discovery.md:L175): Resolve the L-6 retrieve versus J-73 abstain conflict explicitly (`L109-L110`).
- **SUMMARY/N5-E04** [E] anchor=`14-N:L176` (14-N-SUMMARY_INDEX.discovery.md:L176): Resolve the J-48 cloud-spill versus J-54 sleep conflict explicitly (`L109-L110`).
- **SUMMARY/N5-E05** [E] anchor=`14-N:L177` (14-N-SUMMARY_INDEX.discovery.md:L177): Replace implicit `task_runner` call-order policy with an explicit precedence specification (`L110-L115`).
- **SUMMARY/N5-E06** [E] anchor=`14-N:L178` (14-N-SUMMARY_INDEX.discovery.md:L178): Store precedence as data in this order: `safety > cost > abstention > escalation > modality > serving-tier` (`L112-L114`).
- **SUMMARY/N5-E07** [E] anchor=`14-N:L179` (14-N-SUMMARY_INDEX.discovery.md:L179): Emit one composite `RouteDecision` row for an arbitration (`L113-L115`).
- **SUMMARY/N5-E08** [E] anchor=`14-N:L180` (14-N-SUMMARY_INDEX.discovery.md:L180): Unify six verification-intensity deciders as axes: B-41 level, L-21 model, J-84 effort, C-46 delegation, K-5 gate risk, and B-28 edit format (`L111-L115`).
- **SUMMARY/N5-E09** [E] anchor=`14-N:L181` (14-N-SUMMARY_INDEX.discovery.md:L181): Resolve intensity over `risk × model × level × task-difficulty × delegation` (`L113-L115`).
- **SUMMARY/N5-E10** [E] anchor=`14-N:L182` (14-N-SUMMARY_INDEX.discovery.md:L182): Make B-28 the edit-format leg of J-55 (`L114-L115`).
- **SUMMARY/N5-E11** [E] anchor=`14-N:L183` (14-N-SUMMARY_INDEX.discovery.md:L183): Reuse J-92’s FSM-substate structural-enforcement pattern for routing order (`L116-L118`).
- **SUMMARY/N5-E12** [E] anchor=`14-N:L184` (14-N-SUMMARY_INDEX.discovery.md:L184): Let K-4 gate-tax measure the one combined policy (`L115-L118`).
- **SUMMARY/N5-E13** [E] anchor=`14-N:L185` (14-N-SUMMARY_INDEX.discovery.md:L185): Freeze the precedence table under R8 (`L119-L120`).
- **SUMMARY/N5-E14** [E] anchor=`14-N:L186` (14-N-SUMMARY_INDEX.discovery.md:L186): Integrate at the `crates/orchestrator` `task_runner` routing seam (`L119`).
- **SUMMARY/N5-E15** [E] [SURFACE] anchor=`14-N:L187` (14-N-SUMMARY_INDEX.discovery.md:L187): Render a `RouteDecision` explanation row on the canvas/RRA (`L119-L120`).
- **SUMMARY/N5-E16** [E] anchor=`14-N:L188` (14-N-SUMMARY_INDEX.discovery.md:L188): When two or more routers fire, emit one `+route_arbitrated` row naming both/all fired routers, the winner, and its precedence rank (`L121-L123`).
- **SUMMARY/N5-E17** [E] anchor=`14-N:L189` (14-N-SUMMARY_INDEX.discovery.md:L189): A route produced with no arbitration row while at least two routers fired must fail the R10 integration gate (`L121-L123`).
- **SUMMARY/N5-A01** [A] anchor=`14-N:L193` (14-N-SUMMARY_INDEX.discovery.md:L193): The source calls out E-39 in a conflict although E-39 is not in the stated ten-router list (`L107-L110`).
- **SUMMARY/N5-A02** [A] anchor=`14-N:L194` (14-N-SUMMARY_INDEX.discovery.md:L194): Tie-breaking within a precedence tier, policy-versioning, and how task difficulty is measured are not specified.
- **SUMMARY/N5-A03** [A] anchor=`14-N:L195` (14-N-SUMMARY_INDEX.discovery.md:L195): The declared intensity resolver tuple omits edit-format in its parenthesized axes even though B-28 is explicitly incorporated (`L113-L115`).

#### `N-6 — provenance / trust lattice + admission-gate family` (21 records)

- **SUMMARY/N6-E01** [E] anchor=`14-N:L201` (14-N-SUMMARY_INDEX.discovery.md:L201): Define a single semantic provenance/trust lattice mapping G-1 taint `Provenance`, C-48 `TrustTier T0-T3`, F-6 `SkillTrustTier S1-S4`, C-30 oracle `source_tier T1-T4`, K-23 per-repo trust, and 4D `EvidenceLevel` (`L127-L136`).
- **SUMMARY/N6-E02** [E] anchor=`14-N:L202` (14-N-SUMMARY_INDEX.discovery.md:L202): Make the mapping total across all six scales (`L130-L132`).
- **SUMMARY/N6-E03** [E] anchor=`14-N:L203` (14-N-SUMMARY_INDEX.discovery.md:L203): Preserve separate enums where intentional; specifically do not merge F-6’s enum into C-48’s (`L130-L137`).
- **SUMMARY/N6-E04** [E] anchor=`14-N:L204` (14-N-SUMMARY_INDEX.discovery.md:L204): Carry mapped provenance on the same object through ingest at G-1 (`L132`).
- **SUMMARY/N6-E05** [E] anchor=`14-N:L205` (14-N-SUMMARY_INDEX.discovery.md:L205): Preserve/map it through memory at C-48 (`L132`).
- **SUMMARY/N6-E06** [E] anchor=`14-N:L206` (14-N-SUMMARY_INDEX.discovery.md:L206): Preserve/map it through citation at H-53 (`L132`).
- **SUMMARY/N6-E07** [E] anchor=`14-N:L207` (14-N-SUMMARY_INDEX.discovery.md:L207): Preserve/map it into C-30 oracle TTL decisions (`L132`).
- **SUMMARY/N6-E08** [E] anchor=`14-N:L208` (14-N-SUMMARY_INDEX.discovery.md:L208): Preserve/map it into F-6 skill autonomy (`L132-L133`).
- **SUMMARY/N6-E09** [E] anchor=`14-N:L209` (14-N-SUMMARY_INDEX.discovery.md:L209): Preserve/map it into K-23 cross-repository trust (`L132-L133`).
- **SUMMARY/N6-E10** [E] anchor=`14-N:L210` (14-N-SUMMARY_INDEX.discovery.md:L210): Retain the existing J-93-C taint-survives-memory wire as part of the lattice (`L129-L136`).
- **SUMMARY/N6-E11** [E] anchor=`14-N:L211` (14-N-SUMMARY_INDEX.discovery.md:L211): Create one admission-scanner registry shared by F-1 `VibeGate`, J-102 config ingress, J-109 learned-artifact admission, and new K-23/M-2 surfaces (`L133-L136`).
- **SUMMARY/N6-E12** [E] anchor=`14-N:L212` (14-N-SUMMARY_INDEX.discovery.md:L212): Reuse F-1’s secret-grep scanner leg in that family (`L133-L136`).
- **SUMMARY/N6-E13** [E] anchor=`14-N:L213` (14-N-SUMMARY_INDEX.discovery.md:L213): Preserve J-109’s indexing relationship to J-102’s manifest (`L134-L135`).
- **SUMMARY/N6-E14** [E] anchor=`14-N:L214` (14-N-SUMMARY_INDEX.discovery.md:L214): Keep F-6’s auditor-enforced type barrier (`L138-L139`).
- **SUMMARY/N6-E15** [E] anchor=`14-N:L215` (14-N-SUMMARY_INDEX.discovery.md:L215): Integrate lattice/taint in `crates/security` and wire the admission-gate call sites to a lattice mapping table (`L138-L139`).
- **SUMMARY/N6-E16** [E] [SURFACE] anchor=`14-N:L216` (14-N-SUMMARY_INDEX.discovery.md:L216): Add a provenance badge on cited and memory surfaces, or justify N/A (`L138-L139`).
- **SUMMARY/N6-E17** [E] anchor=`14-N:L217` (14-N-SUMMARY_INDEX.discovery.md:L217): Test an ingest → memory write → citation chain for coherent end-to-end trust mapping with no silent tier reset (`L140-L142`).
- **SUMMARY/N6-E18** [E] anchor=`14-N:L218` (14-N-SUMMARY_INDEX.discovery.md:L218): Any tainted string losing taint at a memory, citation, or skill boundary must fail (`L141-L142`).
- **SUMMARY/N6-A01** [A] anchor=`14-N:L222` (14-N-SUMMARY_INDEX.discovery.md:L222): The premise says “five provenance scales,” while N-6 enumerates six (`L14,L127-L130`).
- **SUMMARY/N6-A02** [A] anchor=`14-N:L223` (14-N-SUMMARY_INDEX.discovery.md:L223): Exact lattice ordering/mapping cells, treatment of incomparable meanings, and whether K-23/M-2 are gates or only surfaces require detail.
- **SUMMARY/N6-A03** [A] anchor=`14-N:L224` (14-N-SUMMARY_INDEX.discovery.md:L224): The summary says “three admission gates” while naming F-1, J-102, J-109 plus K-23/M-2 surfaces (`L133-L136`).

#### `N-7 — human render as a REQUIRED verdict field` (16 records)

- **SUMMARY/N7-E01** [E] anchor=`14-N:L230` (14-N-SUMMARY_INDEX.discovery.md:L230): Make human render a required field for every gate/verdict family, analogous to the existing required SSB row (`L146-L152`).
- **SUMMARY/N7-E02** [E] [SURFACE] anchor=`14-N:L231` (14-N-SUMMARY_INDEX.discovery.md:L231): Require each verdict class to declare its render target/form (`L150-L152`).
- **SUMMARY/N7-E03** [E] [SURFACE] anchor=`14-N:L232` (14-N-SUMMARY_INDEX.discovery.md:L232): Support PR comments as render targets (`L151-L152`).
- **SUMMARY/N7-E04** [E] [SURFACE] anchor=`14-N:L233` (14-N-SUMMARY_INDEX.discovery.md:L233): Support chat acknowledgment lines as render targets (`L151-L152`).
- **SUMMARY/N7-E05** [E] [SURFACE] anchor=`14-N:L234` (14-N-SUMMARY_INDEX.discovery.md:L234): Support canvas badges as render targets (`L151-L152`).
- **SUMMARY/N7-E06** [E] [SURFACE] anchor=`14-N:L235` (14-N-SUMMARY_INDEX.discovery.md:L235): Support brief rows as render targets (`L151-L152`).
- **SUMMARY/N7-E07** [E] [SURFACE] anchor=`14-N:L236` (14-N-SUMMARY_INDEX.discovery.md:L236): Support M-3 organization rollups as render targets (`L151-L152`).
- **SUMMARY/N7-E08** [E] anchor=`14-N:L237` (14-N-SUMMARY_INDEX.discovery.md:L237): Generalize K-11 PR evidence rendering into one render substrate rather than reauthoring it for each surface (`L153-L154`).
- **SUMMARY/N7-E09** [E] anchor=`14-N:L238` (14-N-SUMMARY_INDEX.discovery.md:L238): Feed the substrate from many verdict producers, including J-83 reasoning receipts, L-7 hallucination verdicts, and N-1 measurements (`L148-L152`).
- **SUMMARY/N7-E10** [E] anchor=`14-N:L239` (14-N-SUMMARY_INDEX.discovery.md:L239): Integrate at `crates/orchestrator` verdict emission and the render substrate (`L155-L156`).
- **SUMMARY/N7-E11** [E] anchor=`14-N:L240` (14-N-SUMMARY_INDEX.discovery.md:L240): Add a required `render_field` to the `WIRING_CONTRACT` template, tied to N-2 (`L155-L156`).
- **SUMMARY/N7-E12** [E] [SURFACE] anchor=`14-N:L241` (14-N-SUMMARY_INDEX.discovery.md:L241): The frontend pairing is mandatory because rendering is the feature itself (`L155-L156`).
- **SUMMARY/N7-E13** [E] [SURFACE] anchor=`14-N:L242` (14-N-SUMMARY_INDEX.discovery.md:L242): Every verdict-emitting gate must deliver a human-readable render to its declared PR/chat/canvas/brief target (`L157-L158`).
- **SUMMARY/N7-E14** [E] anchor=`14-N:L243` (14-N-SUMMARY_INDEX.discovery.md:L243): N-2 must reject a newly shipped verdict class with no declared render (`L158-L159`).
- **SUMMARY/N7-A01** [A] [SURFACE] anchor=`14-N:L247` (14-N-SUMMARY_INDEX.discovery.md:L247): The mapping from each existing verdict class to exactly one or multiple render targets is not enumerated.
- **SUMMARY/N7-A02** [A] anchor=`14-N:L248` (14-N-SUMMARY_INDEX.discovery.md:L248): Accessibility, localization, acknowledgement semantics, and delivery-failure behavior are absent.

#### `N-8 — governance-coverage substrate` (16 records)

- **SUMMARY/N8-E01** [E] anchor=`14-N:L254` (14-N-SUMMARY_INDEX.discovery.md:L254): Unify five governance instruments as views of one substrate rather than reimplementing them (`L165-L173`).
- **SUMMARY/N8-E02** [E] anchor=`14-N:L255` (14-N-SUMMARY_INDEX.discovery.md:L255): View B-17 gate liveness: whether a gate fires (`L165-L166`).
- **SUMMARY/N8-E03** [E] anchor=`14-N:L256` (14-N-SUMMARY_INDEX.discovery.md:L256): View K-22 doctrine-enforcer coverage: whether a rule has an enforcer (`L166-L167`).
- **SUMMARY/N8-E04** [E] anchor=`14-N:L257` (14-N-SUMMARY_INDEX.discovery.md:L257): View J-46 governance-debt rate (`L166-L168`).
- **SUMMARY/N8-E05** [E] anchor=`14-N:L258` (14-N-SUMMARY_INDEX.discovery.md:L258): View J-70 gate-manifest traceability (`L167-L168`).
- **SUMMARY/N8-E06** [E] anchor=`14-N:L259` (14-N-SUMMARY_INDEX.discovery.md:L259): Generalize H-27’s finance-only kill-your-own-mutant corpus: whether an enforcer actually catches violations (`L167-L173`).
- **SUMMARY/N8-E07** [E] anchor=`14-N:L260` (14-N-SUMMARY_INDEX.discovery.md:L260): Produce a combined marketable governance claim: percentage of rules machine-enforced, mutation proof per enforcer, and drift rate per quarter (`L168-L170`).
- **SUMMARY/N8-E08** [E] anchor=`14-N:L261` (14-N-SUMMARY_INDEX.discovery.md:L261): Make mutant-corpus coverage a gate-manifest lifecycle requirement (`L170-L172`).
- **SUMMARY/N8-E09** [E] anchor=`14-N:L262` (14-N-SUMMARY_INDEX.discovery.md:L262): Refuse a new gate entry to the R8-frozen set unless it has a mutant corpus (`L170-L177`).
- **SUMMARY/N8-E10** [E] anchor=`14-N:L263` (14-N-SUMMARY_INDEX.discovery.md:L263): Also require a measured mutant kill rate before frozen-set entry (`L170-L177`).
- **SUMMARY/N8-E11** [E] anchor=`14-N:L264` (14-N-SUMMARY_INDEX.discovery.md:L264): Integrate governance coverage in `.claude/tools/` and the gate-manifest lifecycle (`L174-L175`).
- **SUMMARY/N8-E12** [E] [SURFACE] anchor=`14-N:L265` (14-N-SUMMARY_INDEX.discovery.md:L265): Surface a governance tile on G-5/M-3 (`L174-L175`).
- **SUMMARY/N8-E13** [E] anchor=`14-N:L266` (14-N-SUMMARY_INDEX.discovery.md:L266): A coverage run emits `+governance_coverage_measured` with five Kinds (`L176-L177`).
- **SUMMARY/N8-E14** [E] anchor=`14-N:L267` (14-N-SUMMARY_INDEX.discovery.md:L267): A decorative rule with no enforcer must not be reported as covered; doing so fails (`L177-L178`).
- **SUMMARY/N8-A01** [A] anchor=`14-N:L271` (14-N-SUMMARY_INDEX.discovery.md:L271): The five `Kind` enum names and formula for the composite marketable number are not provided.
- **SUMMARY/N8-A02** [A] anchor=`14-N:L272` (14-N-SUMMARY_INDEX.discovery.md:L272): Required kill-rate threshold and quarter/window definition are unspecified.

#### `N-9 — one counterfactual / ablation engine` (18 records)

- **SUMMARY/N9-E01** [E] anchor=`14-N:L278` (14-N-SUMMARY_INDEX.discovery.md:L278): Build one SSB-backed engine that reruns/redecides with one factor removed or injected and scores the delta from journal outcomes (`L182-L186`).
- **SUMMARY/N9-E02** [E] anchor=`14-N:L279` (14-N-SUMMARY_INDEX.discovery.md:L279): Parameterize target kind as role, verifier, ranker, rule, memory, or case (`L185-L186`).
- **SUMMARY/N9-E03** [E] anchor=`14-N:L280` (14-N-SUMMARY_INDEX.discovery.md:L280): Adapt C-45 role credit to the engine (`L183-L188`).
- **SUMMARY/N9-E04** [E] anchor=`14-N:L281` (14-N-SUMMARY_INDEX.discovery.md:L281): Adapt I-28 DAG credit to the engine (`L183-L188`).
- **SUMMARY/N9-E05** [E] anchor=`14-N:L282` (14-N-SUMMARY_INDEX.discovery.md:L282): Adapt J-105 memory ROI to the engine (`L183-L188`).
- **SUMMARY/N9-E06** [E] anchor=`14-N:L283` (14-N-SUMMARY_INDEX.discovery.md:L283): Adapt J-109 replay fitness to the engine (`L183-L188`).
- **SUMMARY/N9-E07** [E] anchor=`14-N:L284` (14-N-SUMMARY_INDEX.discovery.md:L284): Adapt E-6 iso-budget role ablation to the engine (`L184-L188`).
- **SUMMARY/N9-E08** [E] anchor=`14-N:L285` (14-N-SUMMARY_INDEX.discovery.md:L285): Adapt E-10 critic ablation to the engine (`L184-L188`).
- **SUMMARY/N9-E09** [E] anchor=`14-N:L286` (14-N-SUMMARY_INDEX.discovery.md:L286): Adapt H-40 verifier-retention ablation to the engine (`L184-L188`).
- **SUMMARY/N9-E10** [E] anchor=`14-N:L287` (14-N-SUMMARY_INDEX.discovery.md:L287): Adapt J-75 RRF ablation to the engine (`L184-L188`).
- **SUMMARY/N9-E11** [E] anchor=`14-N:L288` (14-N-SUMMARY_INDEX.discovery.md:L288): Keep all eight adapters thin under ND-5; business logic belongs in the core (`L185-L193`).
- **SUMMARY/N9-E12** [E] anchor=`14-N:L289` (14-N-SUMMARY_INDEX.discovery.md:L289): Consume N-1 outcome labels (`L186-L189`).
- **SUMMARY/N9-E13** [E] anchor=`14-N:L290` (14-N-SUMMARY_INDEX.discovery.md:L290): Integrate one counterfactual engine in `crates/orchestrator` and replace the eight call-site mechanisms (`L190-L191`).
- **SUMMARY/N9-E14** [E] [SURFACE] anchor=`14-N:L291` (14-N-SUMMARY_INDEX.discovery.md:L291): Reuse existing attribution drawers or justify no frontend (`L190-L191`).
- **SUMMARY/N9-E15** [E] anchor=`14-N:L292` (14-N-SUMMARY_INDEX.discovery.md:L292): A role/verifier/ranker ablation emits `+counterfactual_scored` with the appropriate Kind (`L192-L193`).
- **SUMMARY/N9-E16** [E] anchor=`14-N:L293` (14-N-SUMMARY_INDEX.discovery.md:L293): An adapter containing business logic rather than delegating to the core fails ND-5 (`L193`).
- **SUMMARY/N9-A01** [A] anchor=`14-N:L297` (14-N-SUMMARY_INDEX.discovery.md:L297): Replay isolation, seed control, delta metric, injected-factor semantics, and allowable side effects are not stated.
- **SUMMARY/N9-A02** [A] anchor=`14-N:L298` (14-N-SUMMARY_INDEX.discovery.md:L298): The target enum lists six targets but the eight source adapters are not mapped one-to-one in the summary.

#### `N-10 — clarity / novelty -> budget wires` (18 records)

- **SUMMARY/N10-E01** [E] anchor=`14-N:L304` (14-N-SUMMARY_INDEX.discovery.md:L304): Unify B-38 continuous ambiguity, B-40 critic, C-29 intent pairs, and J-32 front-door confidence into one typed ambiguity signal (`L197-L204`).
- **SUMMARY/N10-E02** [E] anchor=`14-N:L305` (14-N-SUMMARY_INDEX.discovery.md:L305): Unify D-15 claim `NoveltyClass` and the 4F/B-21 novel-plan-step signal into one typed novelty signal (`L198-L204`).
- **SUMMARY/N10-E03** [E] anchor=`14-N:L306` (14-N-SUMMARY_INDEX.discovery.md:L306): Materialize `AmbiguityScore` (`L200`).
- **SUMMARY/N10-E04** [E] anchor=`14-N:L307` (14-N-SUMMARY_INDEX.discovery.md:L307): Materialize `NoveltyScore` (`L200`).
- **SUMMARY/N10-E05** [E] anchor=`14-N:L308` (14-N-SUMMARY_INDEX.discovery.md:L308): Join the two scores for the current task (`L200`).
- **SUMMARY/N10-E06** [E] anchor=`14-N:L309` (14-N-SUMMARY_INDEX.discovery.md:L309): Feed the shared scores into 4D research-tree depth (`L200-L203`).
- **SUMMARY/N10-E07** [E] anchor=`14-N:L310` (14-N-SUMMARY_INDEX.discovery.md:L310): Feed the shared scores into J-84 reasoning-effort tier (`L201-L203`).
- **SUMMARY/N10-E08** [E] anchor=`14-N:L311` (14-N-SUMMARY_INDEX.discovery.md:L311): Feed the shared scores into J-39 decomposition node sizing (`L201-L203`).
- **SUMMARY/N10-E09** [E] anchor=`14-N:L312` (14-N-SUMMARY_INDEX.discovery.md:L312): Feed the shared scores into J-79 grill/clarify offers (`L201-L203`).
- **SUMMARY/N10-E10** [E] anchor=`14-N:L313` (14-N-SUMMARY_INDEX.discovery.md:L313): High ambiguity or an off-map task increases research depth (`L202`).
- **SUMMARY/N10-E11** [E] [SURFACE] anchor=`14-N:L314` (14-N-SUMMARY_INDEX.discovery.md:L314): High ambiguity or an off-map task increases clarification behavior, rather than merely showing a chip (`L202`).
- **SUMMARY/N10-E12** [E] anchor=`14-N:L315` (14-N-SUMMARY_INDEX.discovery.md:L315): Wire existing detectors; do not implement another detector (`L203-L204`).
- **SUMMARY/N10-E13** [E] anchor=`14-N:L316` (14-N-SUMMARY_INDEX.discovery.md:L316): Integrate at `crates/orchestrator` research-tree, effort-tier, and node-admission seams (`L205-L206`).
- **SUMMARY/N10-E14** [E] [SURFACE] anchor=`14-N:L317` (14-N-SUMMARY_INDEX.discovery.md:L317): The default frontend disposition is N/A because this is internal allocation; canvas may surface depth if desired (`L205-L206`).
- **SUMMARY/N10-E15** [E] anchor=`14-N:L318` (14-N-SUMMARY_INDEX.discovery.md:L318): A deliberately vague/novel task must receive greater research depth and effort than a clear baseline while both consumers read the same score rows (`L207-L209`).
- **SUMMARY/N10-E16** [E] anchor=`14-N:L319` (14-N-SUMMARY_INDEX.discovery.md:L319): A high-ambiguity task receiving only baseline depth must fail (`L208-L209`).
- **SUMMARY/N10-A01** [A] anchor=`14-N:L323` (14-N-SUMMARY_INDEX.discovery.md:L323): Score ranges, normalization/join rule, cutoffs, exact budget increments, and the relationship between “off-map” and `NoveltyScore` are unspecified.
- **SUMMARY/N10-A02** [A] anchor=`14-N:L324` (14-N-SUMMARY_INDEX.discovery.md:L324): “more clarify” is explicit but whether it means stronger offer, mandatory query, or repeated grill is not stated.

#### `N-11 — journal-fold substrate` (18 records)

- **SUMMARY/N11-E01** [E] anchor=`14-N:L330` (14-N-SUMMARY_INDEX.discovery.md:L330): Extend L-16’s pure `fold(log, budget, policy) -> context-view` core rather than reauthoring it (`L213-L219`).
- **SUMMARY/N11-E02** [E] anchor=`14-N:L331` (14-N-SUMMARY_INDEX.discovery.md:L331): Represent per-surface fold functions as data/plugins (`L215-L216`).
- **SUMMARY/N11-E03** [E] [SURFACE] anchor=`14-N:L332` (14-N-SUMMARY_INDEX.discovery.md:L332): Move J-31/J-60/J-98/K-23 canvas-altitude derivations through the shared fold core (`L213-L220`).
- **SUMMARY/N11-E04** [E] anchor=`14-N:L333` (14-N-SUMMARY_INDEX.discovery.md:L333): Move I-39 plan-status derivation through the shared fold core (`L214-L220`).
- **SUMMARY/N11-E05** [E] anchor=`14-N:L334` (14-N-SUMMARY_INDEX.discovery.md:L334): Move K-24 billing-balance derivation through the shared fold core (`L214-L220`).
- **SUMMARY/N11-E06** [E] [SURFACE] anchor=`14-N:L335` (14-N-SUMMARY_INDEX.discovery.md:L335): Move M-3 fleet-rollup derivation through the shared fold core (`L214-L220`).
- **SUMMARY/N11-E07** [E] anchor=`14-N:L336` (14-N-SUMMARY_INDEX.discovery.md:L336): Move K-22 coverage derivation through the shared fold core (`L214-L220`).
- **SUMMARY/N11-E08** [E] anchor=`14-N:L337` (14-N-SUMMARY_INDEX.discovery.md:L337): Use one J-68 Kani correctness proof for all five surface fold functions instead of five GD-2 audits (`L215-L219`).
- **SUMMARY/N11-E09** [E] anchor=`14-N:L338` (14-N-SUMMARY_INDEX.discovery.md:L338): Ship the ND-2 retirement map for old paths left live by L-16 (`L216-L218`).
- **SUMMARY/N11-E10** [E] anchor=`14-N:L339` (14-N-SUMMARY_INDEX.discovery.md:L339): End the `recover_stubs` path (`L217-L218`).
- **SUMMARY/N11-E11** [E] anchor=`14-N:L340` (14-N-SUMMARY_INDEX.discovery.md:L340): Declare H-49 CAS/undo-tree the canonical log representation (`L217-L219`).
- **SUMMARY/N11-E12** [E] anchor=`14-N:L341` (14-N-SUMMARY_INDEX.discovery.md:L341): Integrate the fold core and five fold functions in `crates/orchestrator` (`L220`).
- **SUMMARY/N11-E13** [E] [SURFACE] anchor=`14-N:L342` (14-N-SUMMARY_INDEX.discovery.md:L342): Verify existing canvas altitudes consume the folds; do not blindly rewire them (`L220-L221`).
- **SUMMARY/N11-E14** [E] anchor=`14-N:L343` (14-N-SUMMARY_INDEX.discovery.md:L343): With a pinned seed, two of the five surfaces must rederive byte-identical output through the one core (`L222-L224`).
- **SUMMARY/N11-E15** [E] anchor=`14-N:L344` (14-N-SUMMARY_INDEX.discovery.md:L344): The retirement switch removes the old path (`L222-L224`).
- **SUMMARY/N11-E16** [E] anchor=`14-N:L345` (14-N-SUMMARY_INDEX.discovery.md:L345): Any surface retaining an ad-hoc journal-rederivation path fails conformance (`L223-L224`).
- **SUMMARY/N11-A01** [A] [SURFACE] anchor=`14-N:L349` (14-N-SUMMARY_INDEX.discovery.md:L349): The summary calls these “five OTHER surfaces”; canvas altitude is one surface with four contributing unit IDs, which should be confirmed in detail.
- **SUMMARY/N11-A02** [A] anchor=`14-N:L350` (14-N-SUMMARY_INDEX.discovery.md:L350): Plugin schema, budgets/policies per surface, proof property, retirement timing, and recovery migration are absent.

#### `N-12 — consistency + dedupe bundle` (31 records)

- **SUMMARY/N12-E01** [E] anchor=`14-N:L358` (14-N-SUMMARY_INDEX.discovery.md:L358): Treat the labeled consistency collapses as disjoint-file, parallel-safe, independently shippable sub-slices (`L230,L244-L245`).
- **SUMMARY/N12-E02** [E] anchor=`14-N:L359` (14-N-SUMMARY_INDEX.discovery.md:L359): For each concern, deduplicate to the first-in-flow implementation and use adapters at other call sites (`L242-L243`).
- **SUMMARY/N12-E03** [E] anchor=`14-N:L360` (14-N-SUMMARY_INDEX.discovery.md:L360): Do not add another type for a collapsed concern (`L242-L243`).
- **SUMMARY/N12-E04** [E] anchor=`14-N:L361` (14-N-SUMMARY_INDEX.discovery.md:L361): Trigger the surviving implementation from at least two former call sites and observe one canonical row/type (`L246-L247`).
- **SUMMARY/N12-E05** [E] anchor=`14-N:L362` (14-N-SUMMARY_INDEX.discovery.md:L362): Have N-2 reject any reappearing parallel implementation (`L247-L248`).
- **SUMMARY/N12-E06** [E] [SURFACE] anchor=`14-N:L363` (14-N-SUMMARY_INDEX.discovery.md:L363): Default frontend disposition is N/A per sub-slice, except a surfaced `FailureClass` may have a drawer; each N/A needs justification (`L244-L245`).
- **SUMMARY/N12-E07** [E] anchor=`14-N:L367` (14-N-SUMMARY_INDEX.discovery.md:L367): Create one `FailureClass` enum shared across the auditor, anti-spiral system, MAST, corpus rows, and case edges (`L231-L232`).
- **SUMMARY/N12-E08** [E] anchor=`14-N:L368` (14-N-SUMMARY_INDEX.discovery.md:L368): Collapse `failure_taxonomy.py`’s 13 categories into the canonical enum (`L231`).
- **SUMMARY/N12-E09** [E] anchor=`14-N:L369` (14-N-SUMMARY_INDEX.discovery.md:L369): Collapse the tools taxonomy’s 9 categories into it (`L231`).
- **SUMMARY/N12-E10** [E] anchor=`14-N:L370` (14-N-SUMMARY_INDEX.discovery.md:L370): Collapse E-6 MAST’s 14 modes into it (`L231-L232`).
- **SUMMARY/N12-E11** [E] anchor=`14-N:L371` (14-N-SUMMARY_INDEX.discovery.md:L371): Remove/collapse the E-29 MAST duplicate (`L231-L232`).
- **SUMMARY/N12-E12** [E] anchor=`14-N:L372` (14-N-SUMMARY_INDEX.discovery.md:L372): Map C-10 `StagnationPattern` into it (`L232`).
- **SUMMARY/N12-E13** [E] anchor=`14-N:L376` (14-N-SUMMARY_INDEX.discovery.md:L376): Create one `AbstainDecision` spine spanning D-16, I-8, D-17, L-7, J-73, and J-110 (`L233-L234`).
- **SUMMARY/N12-E14** [E] anchor=`14-N:L377` (14-N-SUMMARY_INDEX.discovery.md:L377): Propagate an upstream retrieval abstention as taint on a downstream answer lacking evidence (`L233-L234`).
- **SUMMARY/N12-E15** [E] anchor=`14-N:L378` (14-N-SUMMARY_INDEX.discovery.md:L378): Raise that downstream path to the L-2 risk floor (`L233-L234`).
- **SUMMARY/N12-E16** [E] anchor=`14-N:L382` (14-N-SUMMARY_INDEX.discovery.md:L382): Make B-40’s nine-variant `ProjectClass` the one project-class registry (`L235-L236`).
- **SUMMARY/N12-E17** [E] anchor=`14-N:L383` (14-N-SUMMARY_INDEX.discovery.md:L383): Have B-42 cosmos consume the registry (`L235`).
- **SUMMARY/N12-E18** [E] anchor=`14-N:L384` (14-N-SUMMARY_INDEX.discovery.md:L384): Have E-50 archetypes consume the registry (`L235`).
- **SUMMARY/N12-E19** [E] anchor=`14-N:L385` (14-N-SUMMARY_INDEX.discovery.md:L385): Have E-12 manifests consume the registry (`L235-L236`).
- **SUMMARY/N12-E20** [E] anchor=`14-N:L386` (14-N-SUMMARY_INDEX.discovery.md:L386): Have L-14 defaults consume the registry (`L235-L236`).
- **SUMMARY/N12-E21** [E] anchor=`14-N:L390` (14-N-SUMMARY_INDEX.discovery.md:L390): Route C-47 sled `CaseBase` writes through the J-109 learned-artifact admission pipeline (`L237-L238`).
- **SUMMARY/N12-E22** [E] anchor=`14-N:L391` (14-N-SUMMARY_INDEX.discovery.md:L391): Route C-49 Markdown `ExperienceCorpus` writes through the J-109 admission pipeline (`L237-L238`).
- **SUMMARY/N12-E23** [E] anchor=`14-N:L392` (14-N-SUMMARY_INDEX.discovery.md:L392): Close the previously unguarded learned-artifact write path (`L237-L238`).
- **SUMMARY/N12-E24** [E] anchor=`14-N:L396` (14-N-SUMMARY_INDEX.discovery.md:L396): Provide one shared KL/DDM/IQR drift-primitive library for D-4, G-10, H-30, F-5, and B-22b (`L239`).
- **SUMMARY/N12-E25** [E] anchor=`14-N:L397` (14-N-SUMMARY_INDEX.discovery.md:L397): Preserve each consumer’s distinct policy while sharing only the primitives (`L239`).
- **SUMMARY/N12-E26** [E] anchor=`14-N:L401` (14-N-SUMMARY_INDEX.discovery.md:L401): Add a three-model conformance test requiring 3I DST, J-68 TLA, and the code transition table to agree (`L240`).
- **SUMMARY/N12-E27** [E] anchor=`14-N:L405` (14-N-SUMMARY_INDEX.discovery.md:L405): Create one LSP daemon pool shared by E-52 verifier, J-18 retrieval channel, and J-52 per-edit block feed (`L241`).
- **SUMMARY/N12-A01** [A] anchor=`14-N:L409` (14-N-SUMMARY_INDEX.discovery.md:L409): The source says “eight independent same-concern collapses” but labels only seven, (a) through (g). There are eight W findings because (a) combines W-8 and W-26; whether a sub-slice is missing is unresolved (`L229-L241`).
- **SUMMARY/N12-A02** [A] anchor=`14-N:L410` (14-N-SUMMARY_INDEX.discovery.md:L410): “First-in-flow” canonical owners, migration/deletion lists, enum mappings, and adapter boundaries are not enumerated.
- **SUMMARY/N12-A03** [A] anchor=`14-N:L411` (14-N-SUMMARY_INDEX.discovery.md:L411): Failure taxonomy category reconciliation and whether `StagnationPattern` remains a policy-specific sub-type are not stated.
- **SUMMARY/N12-A04** [A] anchor=`14-N:L412` (14-N-SUMMARY_INDEX.discovery.md:L412): LSP pool process isolation, language/workspace key, lifetime, crash recovery, and resource limits are absent.

#### `External-steal track disposition ledger` (12 records)

- **SUMMARY/NX-E01** [E] anchor=`14-N:L418` (14-N-SUMMARY_INDEX.discovery.md:L418): Route the four-source steal to N because its highest-value verdict is unification rather than another J unit.
- **SUMMARY/NX-E02** [E] anchor=`14-N:L419` (14-N-SUMMARY_INDEX.discovery.md:L419): Use `workflow/reasoning/four-source-steal-2026-07-05.md` as the phase-4 steal analysis over the programasweights repository, arXiv:2607.02440 EvoPolicyGym, `elai-any-model-research-gaps.md`, and a Ford transcript.
- **SUMMARY/NX-E03** [E] anchor=`14-N:L420` (14-N-SUMMARY_INDEX.discovery.md:L420): Promote only genuine NEW/EXTENSION verdicts into N units.
- **SUMMARY/NX-E04** [E] anchor=`14-N:L421` (14-N-SUMMARY_INDEX.discovery.md:L421): Reject PAW’s compiled-LoRA tier as an N build item and record it in the feature-map ledger.
- **SUMMARY/NX-E05** [E] anchor=`14-N:L422` (14-N-SUMMARY_INDEX.discovery.md:L422): Reject EvoPolicyGym n=1 statistics and missing-memorization-defense as N build items; carry the n=1 caveat into N-14.
- **SUMMARY/NX-E06** [E] anchor=`14-N:L423` (14-N-SUMMARY_INDEX.discovery.md:L423): Reject gaps-document P3-P5 because ELAI already plans them more deeply; record them in the feature-map ledger.
- **SUMMARY/NX-E07** [E] anchor=`14-N:L424` (14-N-SUMMARY_INDEX.discovery.md:L424): Reject the Ford checklist as a build item; record it in the feature-map ledger.
- **SUMMARY/NX-E08** [E] anchor=`14-N:L425` (14-N-SUMMARY_INDEX.discovery.md:L425): Treat prefix-KV S1-1 as already recorded by J-95/H-50.
- **SUMMARY/NX-E09** [E] anchor=`14-N:L426` (14-N-SUMMARY_INDEX.discovery.md:L426): Fold zip-slip S1-2 into N-6’s admission family.
- **SUMMARY/NX-E10** [E] anchor=`14-N:L427` (14-N-SUMMARY_INDEX.discovery.md:L427): Keep ADR evidence-table discipline S1-3 as a feature-map/dossier note rather than an N unit.
- **SUMMARY/NX-E11** [E] anchor=`14-N:L428` (14-N-SUMMARY_INDEX.discovery.md:L428): Keep K-1/K-7 packaging verdicts S4-1/S3-4/S1-6 as feature-map/dossier/K-2 notes.
- **SUMMARY/NX-E12** [E] anchor=`14-N:L429` (14-N-SUMMARY_INDEX.discovery.md:L429): Keep the EvoPolicyGym bench S2-4 as a feature-map/dossier/K-2 note rather than a new N unit.

#### `N-13 — OutputBoundary + structured-output capability matrix` (27 records)

- **SUMMARY/N13-E01** [E] anchor=`14-N:L435` (14-N-SUMMARY_INDEX.discovery.md:L435): Establish the invariant that every machine-actionable model output crosses one provider-agnostic `OutputBoundary` (`L263-L269`).
- **SUMMARY/N13-E02** [E] anchor=`14-N:L436` (14-N-SUMMARY_INDEX.discovery.md:L436): Create one per-output-type registry row with `schema` (`L264-L265`).
- **SUMMARY/N13-E03** [E] anchor=`14-N:L437` (14-N-SUMMARY_INDEX.discovery.md:L437): Include `parser` in each row (`L264-L265`).
- **SUMMARY/N13-E04** [E] anchor=`14-N:L438` (14-N-SUMMARY_INDEX.discovery.md:L438): Include `repair policy` in each row (`L264-L265`).
- **SUMMARY/N13-E05** [E] anchor=`14-N:L439` (14-N-SUMMARY_INDEX.discovery.md:L439): Include `retry budget` in each row (`L264-L265`).
- **SUMMARY/N13-E06** [E] anchor=`14-N:L440` (14-N-SUMMARY_INDEX.discovery.md:L440): Include `fail-closed action` in each row (`L264-L265`).
- **SUMMARY/N13-E07** [E] anchor=`14-N:L441` (14-N-SUMMARY_INDEX.discovery.md:L441): Include journaled invalid rate in each row (`L264-L265`).
- **SUMMARY/N13-E08** [E] anchor=`14-N:L442` (14-N-SUMMARY_INDEX.discovery.md:L442): Include backend enforcement mode in each row (`L265`).
- **SUMMARY/N13-E09** [E] [SURFACE] anchor=`14-N:L443` (14-N-SUMMARY_INDEX.discovery.md:L443): Parameterize, rather than reimplement, C-26 `DecodingMode`, E-35 XGrammar/INSTRUCTOR, I-34 CRANE/grammar pin/post-parse revalidation, J-53 tool-input repair, J-51 adherence floor, H-13 canvas grammars, B-24 VLM JSON, L-1 `ClaimSet`, and I-33 CEFL schemas (`L266-L274`).
- **SUMMARY/N13-E10** [E] anchor=`14-N:L444` (14-N-SUMMARY_INDEX.discovery.md:L444): Add structured-output capability fields to the one B-37/I-51 model catalog (`L269-L274`).
- **SUMMARY/N13-E11** [E] anchor=`14-N:L445` (14-N-SUMMARY_INDEX.discovery.md:L445): Resolve the strongest available enforcement mode in order: provider strict > native tool > local CFG > regex > parse/validate/retry/fail-closed (`L270-L272`).
- **SUMMARY/N13-E12** [E] anchor=`14-N:L446` (14-N-SUMMARY_INDEX.discovery.md:L446): The model cannot choose or weaken enforcement mode (TD-3) (`L270-L272`).
- **SUMMARY/N13-E13** [E] anchor=`14-N:L447` (14-N-SUMMARY_INDEX.discovery.md:L447): Inherit I-34 grammar pinning as a binding security requirement (`L271-L272`).
- **SUMMARY/N13-E14** [E] anchor=`14-N:L448` (14-N-SUMMARY_INDEX.discovery.md:L448): Inherit H-13 reject-never-repair as a binding security requirement (`L271-L272`).
- **SUMMARY/N13-E15** [E] anchor=`14-N:L449` (14-N-SUMMARY_INDEX.discovery.md:L449): Inherit J-53/I-46’s frozen repair table as a binding security requirement (`L271-L272`).
- **SUMMARY/N13-E16** [E] anchor=`14-N:L450` (14-N-SUMMARY_INDEX.discovery.md:L450): Do not create another validation engine or model catalog (`L273-L274`).
- **SUMMARY/N13-E17** [E] anchor=`14-N:L451` (14-N-SUMMARY_INDEX.discovery.md:L451): Send per `(type, backend)` invalid rates to N-1 (`L273-L279`).
- **SUMMARY/N13-E18** [E] anchor=`14-N:L452` (14-N-SUMMARY_INDEX.discovery.md:L452): Let D-16 refusal handle fail-closed output (`L273-L274`).
- **SUMMARY/N13-E19** [E] anchor=`14-N:L453` (14-N-SUMMARY_INDEX.discovery.md:L453): Implement the registry seam at `crates/orchestrator/src/output_boundary.rs`, the one catalog, and four machine-actionable emission seams (`L275-L276`).
- **SUMMARY/N13-E20** [E] [SURFACE] anchor=`14-N:L454` (14-N-SUMMARY_INDEX.discovery.md:L454): Add a boundary-verdict badge to claim/decision/tool drawers, or justify N/A if G-5 surfaces the tile (`L275-L276`).
- **SUMMARY/N13-E21** [E] anchor=`14-N:L455` (14-N-SUMMARY_INDEX.discovery.md:L455): Tool directives, `ClaimSet`s, and decision cards on backends lacking strict schema still cross the boundary and emit one `+output_boundary_verdict` containing resolved `enforcement_mode` (`L277-L279`).
- **SUMMARY/N13-E22** [E] anchor=`14-N:L456` (14-N-SUMMARY_INDEX.discovery.md:L456): N-2 rejects a new output type lacking a registry row (`L279-L280`).
- **SUMMARY/N13-E23** [E] anchor=`14-N:L457` (14-N-SUMMARY_INDEX.discovery.md:L457): N-2’s duplicate-concept check rejects a second bespoke validation path (`L279-L280`).
- **SUMMARY/N13-A01** [A] anchor=`14-N:L461` (14-N-SUMMARY_INDEX.discovery.md:L461): The source says machinery exists in “>=8 places” but enumerates nine unit references; this is not contradictory but the machine count is intentionally non-exact (`L266-L268`).
- **SUMMARY/N13-A02** [A] anchor=`14-N:L462` (14-N-SUMMARY_INDEX.discovery.md:L462): Four emission seams are claimed but only three example output classes (tool directive, ClaimSet, decision card) are named (`L275-L278`).
- **SUMMARY/N13-A03** [A] anchor=`14-N:L463` (14-N-SUMMARY_INDEX.discovery.md:L463): Registry storage format, schema language, retry budgets, invalid-rate window, strict-provider capability detection, and exact refusal payload are unspecified.
- **SUMMARY/N13-A04** [A] anchor=`14-N:L464` (14-N-SUMMARY_INDEX.discovery.md:L464): “reject-never-repair” and a registry `repair policy` coexist; the permitted distinction requires detail-file clarification (`L264-L272`).

#### `N-14 — self-improvement edit-loop instruments (EvoPolicyGym)` (27 records)

- **SUMMARY/N14-E01** [E] anchor=`14-N:L470` (14-N-SUMMARY_INDEX.discovery.md:L470): Add a deterministic edit-type classifier finer than C-10’s stagnation categories (`L285-L289`).
- **SUMMARY/N14-E02** [E] anchor=`14-N:L471` (14-N-SUMMARY_INDEX.discovery.md:L471): Classify `SynthesisEdit` when normalized stripped-AST topology changes (`L289-L290`).
- **SUMMARY/N14-E03** [E] anchor=`14-N:L472` (14-N-SUMMARY_INDEX.discovery.md:L472): Classify `ParametricEdit` when normalized stripped-AST topology stays the same (`L289-L290`).
- **SUMMARY/N14-E04** [E] anchor=`14-N:L473` (14-N-SUMMARY_INDEX.discovery.md:L473): Normalize constants before topology comparison (`L289-L290`).
- **SUMMARY/N14-E05** [E] anchor=`14-N:L474` (14-N-SUMMARY_INDEX.discovery.md:L474): Normalize identifiers before topology comparison (`L289-L290`).
- **SUMMARY/N14-E06** [E] anchor=`14-N:L475` (14-N-SUMMARY_INDEX.discovery.md:L475): Join `EditKind` onto every score-bearing attempt (`L289-L291`).
- **SUMMARY/N14-E07** [E] anchor=`14-N:L476` (14-N-SUMMARY_INDEX.discovery.md:L476): Use edit kind to upgrade C-10 persona choice from blind rotation to evidence-driven selection (`L290-L291`).
- **SUMMARY/N14-E08** [E] anchor=`14-N:L477` (14-N-SUMMARY_INDEX.discovery.md:L477): Feed edit kind to I-4, I-22, and J-10 (`L291,L295`).
- **SUMMARY/N14-E09** [E] anchor=`14-N:L478` (14-N-SUMMARY_INDEX.discovery.md:L478): Add synthesis-versus-tuning task demand as an N-5/H-7/L-21/J-84 routing axis (`L291-L295`).
- **SUMMARY/N14-E10** [E] anchor=`14-N:L479` (14-N-SUMMARY_INDEX.discovery.md:L479): Use that axis to avoid assigning local-7B the ownership of structure discovery when measured synthesis ability is weak (`L292-L293`).
- **SUMMARY/N14-E11** [E] anchor=`14-N:L480` (14-N-SUMMARY_INDEX.discovery.md:L480): Add hidden two-stage SELECT-then-REPORT to H-45/H-47 (`L293-L294`).
- **SUMMARY/N14-E12** [E] anchor=`14-N:L481` (14-N-SUMMARY_INDEX.discovery.md:L481): Select candidates using hidden validation data (`L293-L294`).
- **SUMMARY/N14-E13** [E] anchor=`14-N:L482` (14-N-SUMMARY_INDEX.discovery.md:L482): Report performance using a disjoint held-out slice, never the selection slice (`L293-L294,L301-L302`).
- **SUMMARY/N14-E14** [E] anchor=`14-N:L483` (14-N-SUMMARY_INDEX.discovery.md:L483): Add improvement-event timing to I-27’s existing curve (`L294-L296`).
- **SUMMARY/N14-E15** [E] anchor=`14-N:L484` (14-N-SUMMARY_INDEX.discovery.md:L484): Keep all four instruments advisory under TD-3 (`L288-L294`).
- **SUMMARY/N14-E16** [E] anchor=`14-N:L485` (14-N-SUMMARY_INDEX.discovery.md:L485): Use ELAI-measured thresholds only under ID-5 (`L294`).
- **SUMMARY/N14-E17** [E] anchor=`14-N:L486` (14-N-SUMMARY_INDEX.discovery.md:L486): Reuse C-10/B-22b detection, I-27 curve, H-45/H-47/D-6/H-19 promotion machinery, N-5 routing, and N-1 measurement; do not reimplement them (`L295-L296`).
- **SUMMARY/N14-E18** [E] anchor=`14-N:L487` (14-N-SUMMARY_INDEX.discovery.md:L487): Use tree-sitter in `crates/knowledge` for AST structure (`L295-L296`).
- **SUMMARY/N14-E19** [E] anchor=`14-N:L488` (14-N-SUMMARY_INDEX.discovery.md:L488): Integrate `crates/orchestrator/src/edit_kind.rs` with anti-spiral, I-27, promotion, and routing seams (`L297-L298`).
- **SUMMARY/N14-E20** [E] [SURFACE] anchor=`14-N:L489` (14-N-SUMMARY_INDEX.discovery.md:L489): Add an edit-kind chip to the anti-spiral/receipt drawer, or justify internal-advisory N/A (`L297-L298`).
- **SUMMARY/N14-E21** [E] anchor=`14-N:L490` (14-N-SUMMARY_INDEX.discovery.md:L490): Under `ELAI_SEED`, a constant-only edit followed by a restructuring edit must classify Parametric then Synthesis (`L299-L300`).
- **SUMMARY/N14-E22** [E] anchor=`14-N:L491` (14-N-SUMMARY_INDEX.discovery.md:L491): Repeated synthesis churn with no improvement shifts persona to Architect/Simplifier (`L300`).
- **SUMMARY/N14-E23** [E] anchor=`14-N:L492` (14-N-SUMMARY_INDEX.discovery.md:L492): Whitespace-only or rename-only diffs must not flip normalized topology; such a flip fails (`L300-L301`).
- **SUMMARY/N14-E24** [E] anchor=`14-N:L493` (14-N-SUMMARY_INDEX.discovery.md:L493): Reporting selection-slice performance as promotion performance fails the two-stage gate (`L301-L302`).
- **SUMMARY/N14-C01** [C] anchor=`14-N:L497` (14-N-SUMMARY_INDEX.discovery.md:L497): The source reports external paper observations that leaders convert synthesis edits 41-48%, while weak models churn structure 3-10% yet score 25-38% on tuning; the n=1 caveat means these numbers are not promoted into hard ELAI thresholds (`L283-L288`).
- **SUMMARY/N14-A01** [A] anchor=`14-N:L501` (14-N-SUMMARY_INDEX.discovery.md:L501): Supported languages, tree-sitter fallback for parse errors, topology representation, score-bearing-attempt schema, churn window, and persona-switch threshold are unspecified.
- **SUMMARY/N14-A02** [A] anchor=`14-N:L502` (14-N-SUMMARY_INDEX.discovery.md:L502): “Hidden” storage/access controls and held-out budget integration with H-19 need detail.

#### `N-15 — anti-affirmation stability measurement` (24 records)

- **SUMMARY/N15-E01** [E] [SURFACE] anchor=`14-N:L508` (14-N-SUMMARY_INDEX.discovery.md:L508): Limit the new mechanism to advisory prose, because I-17 L1 veto, I-33 externalized candidates, H-31 judge firewall, D-14 inverted framing, E-4 chat drift, and TD-3 already make sycophancy structurally irrelevant at the decision boundary (`L307-L309`).
- **SUMMARY/N15-E02** [E] anchor=`14-N:L509` (14-N-SUMMARY_INDEX.discovery.md:L509): Permute presentation order across N advisory samples (`L309-L311`).
- **SUMMARY/N15-E03** [E] anchor=`14-N:L510` (14-N-SUMMARY_INDEX.discovery.md:L510): Permute option order across N advisory samples (`L309-L311`).
- **SUMMARY/N15-E04** [E] anchor=`14-N:L511` (14-N-SUMMARY_INDEX.discovery.md:L511): Apply the sampling to B-41 review, J-45 cross-family, L-11 OCP-LSP, and I-41 lens advisories (`L310-L317`).
- **SUMMARY/N15-E05** [E] anchor=`14-N:L512` (14-N-SUMMARY_INDEX.discovery.md:L512): If a verdict flips under a presentation-only permutation, log it as framing-unstable (`L311-L312`).
- **SUMMARY/N15-E06** [E] anchor=`14-N:L513` (14-N-SUMMARY_INDEX.discovery.md:L513): Zero a framing-unstable advisory’s weight for the current run (`L311-L312`).
- **SUMMARY/N15-E07** [E] anchor=`14-N:L514` (14-N-SUMMARY_INDEX.discovery.md:L514): Extend I-24 scorer neutralization to framing in advisory lanes (`L312,L317`).
- **SUMMARY/N15-E08** [E] anchor=`14-N:L515` (14-N-SUMMARY_INDEX.discovery.md:L515): Add a D-23 option-order-inversion fixture class where the same reordered facts must not flip the decision (`L313-L314`).
- **SUMMARY/N15-E09** [E] anchor=`14-N:L516` (14-N-SUMMARY_INDEX.discovery.md:L516): Add a D-23 assertion-pull fixture class using a false user assertion such as “I think X” (`L313-L314`).
- **SUMMARY/N15-E10** [E] anchor=`14-N:L517` (14-N-SUMMARY_INDEX.discovery.md:L517): Measure assertion adoption rate without the D-14 probe (`L314`).
- **SUMMARY/N15-E11** [E] anchor=`14-N:L518` (14-N-SUMMARY_INDEX.discovery.md:L518): Measure assertion adoption rate with the D-14 probe (`L314`).
- **SUMMARY/N15-E12** [E] anchor=`14-N:L519` (14-N-SUMMARY_INDEX.discovery.md:L519): Keep the check advisory-only and disqualifying rather than granting it structural decision power (`L314-L316`).
- **SUMMARY/N15-E13** [E] anchor=`14-N:L520` (14-N-SUMMARY_INDEX.discovery.md:L520): Measure through N-1 and use only ELAI-measured thresholds under ID-5 (`L315-L318`).
- **SUMMARY/N15-E14** [E] anchor=`14-N:L521` (14-N-SUMMARY_INDEX.discovery.md:L521): Establish a GD-4 false-positive bound before enabling the check by default (`L315-L316`).
- **SUMMARY/N15-E15** [E] anchor=`14-N:L522` (14-N-SUMMARY_INDEX.discovery.md:L522): Reuse D-23/E-54/D-14 fixtures and the existing runner rather than reauthoring it (`L317-L318`).
- **SUMMARY/N15-E16** [E] anchor=`14-N:L523` (14-N-SUMMARY_INDEX.discovery.md:L523): Integrate `crates/orchestrator/src/framing_invariance.rs`, the D-23 corpus, and advisory emission seams (`L319-L320`).
- **SUMMARY/N15-E17** [E] [SURFACE] anchor=`14-N:L524` (14-N-SUMMARY_INDEX.discovery.md:L524): Surface a framing-stability tile on G-5/N-8, or justify N/A if an existing sycophancy-bench tile provides it (`L319-L320`).
- **SUMMARY/N15-E18** [E] [SURFACE] anchor=`14-N:L525` (14-N-SUMMARY_INDEX.discovery.md:L525): `elai bench sycophancy` runs the two fixture classes (`L321-L323`).
- **SUMMARY/N15-E19** [E] anchor=`14-N:L526` (14-N-SUMMARY_INDEX.discovery.md:L526): A flip emits `+framing_stability_measured(FramingUnstable)` and zeroes weight (`L321-L323`).
- **SUMMARY/N15-E20** [E] anchor=`14-N:L527` (14-N-SUMMARY_INDEX.discovery.md:L527): Assertion-pull emits adoption rates with and without the D-14 probe (`L322-L323`).
- **SUMMARY/N15-E21** [E] anchor=`14-N:L528` (14-N-SUMMARY_INDEX.discovery.md:L528): A presentation-stable advisory incorrectly logged unstable fails the false-positive arm (`L323-L324`).
- **SUMMARY/N15-E22** [E] anchor=`14-N:L529` (14-N-SUMMARY_INDEX.discovery.md:L529): Default-on framing checks without a measured false-positive bound fail GD-4 (`L323-L324`).
- **SUMMARY/N15-A01** [A] anchor=`14-N:L533` (14-N-SUMMARY_INDEX.discovery.md:L533): Sample count N, permutation scheme, verdict-equivalence definition, stability threshold, weight scope beyond “for the run,” and false-positive bound are unspecified.
- **SUMMARY/N15-A02** [A] anchor=`14-N:L534` (14-N-SUMMARY_INDEX.discovery.md:L534): The exact HBR source is not named in the summary.

#### `N-16 — governance closure: evaluator-refutation verdict-collapse + a lambda exogenous-grounding floor + adjudicator-ritual metrics` (39 records)

- **SUMMARY/N16-E01** [E] anchor=`14-N:L540` (14-N-SUMMARY_INDEX.discovery.md:L540): Extend N-8 with three more views of decorative governance; do not build a separate framework (`L329-L332`).
- **SUMMARY/N16-E02** [E] anchor=`14-N:L541` (14-N-SUMMARY_INDEX.discovery.md:L541): Add `evaluator-id` to every SSB verdict row (`L332-L333`).
- **SUMMARY/N16-E03** [E] anchor=`14-N:L542` (14-N-SUMMARY_INDEX.discovery.md:L542): Add evaluator `version-hash` to every SSB verdict row (`L332-L333`).
- **SUMMARY/N16-E04** [E] anchor=`14-N:L543` (14-N-SUMMARY_INDEX.discovery.md:L543): Register evaluator-refutation events (`L333-L335`).
- **SUMMARY/N16-E05** [E] anchor=`14-N:L544` (14-N-SUMMARY_INDEX.discovery.md:L544): Treat a failed H-27 mutant corpus as an evaluator-refutation source (`L333-L334`).
- **SUMMARY/N16-E06** [E] anchor=`14-N:L545` (14-N-SUMMARY_INDEX.discovery.md:L545): Treat an I-24/N-15 bias detection as an evaluator-refutation source (`L333-L334`).
- **SUMMARY/N16-E07** [E] anchor=`14-N:L546` (14-N-SUMMARY_INDEX.discovery.md:L546): Treat a frozen-tool bug fix as an evaluator-refutation source (`L333-L334`).
- **SUMMARY/N16-E08** [E] anchor=`14-N:L547` (14-N-SUMMARY_INDEX.discovery.md:L547): On refutation, invoke the existing F-5 staleness cascade (`L334-L345`).
- **SUMMARY/N16-E09** [E] anchor=`14-N:L548` (14-N-SUMMARY_INDEX.discovery.md:L548): Quarantine exactly the past verdicts issued by the refuted evaluator (`L334-L335`).
- **SUMMARY/N16-E10** [E] anchor=`14-N:L549` (14-N-SUMMARY_INDEX.discovery.md:L549): Place quarantined verdicts into an advisory re-audit queue (`L334-L335`).
- **SUMMARY/N16-E11** [E] anchor=`14-N:L550` (14-N-SUMMARY_INDEX.discovery.md:L550): Never delete quarantined verdicts; preserve F-5 append-only history (`L335,L350-L354`).
- **SUMMARY/N16-E12** [E] anchor=`14-N:L551` (14-N-SUMMARY_INDEX.discovery.md:L551): Explicitly reject the Dung argument-graph adjudicator from the borrowed rubric-closure idea (`L335-L336`).
- **SUMMARY/N16-E13** [E] anchor=`14-N:L552` (14-N-SUMMARY_INDEX.discovery.md:L552): Compute lambda as the windowed fraction of accepted verdicts whose support bottoms out in an exogenous anchor rather than a model rubric (`L336-L339`).
- **SUMMARY/N16-E14** [E] anchor=`14-N:L553` (14-N-SUMMARY_INDEX.discovery.md:L553): Count program anchors as exogenous (`L336-L338`).
- **SUMMARY/N16-E15** [E] anchor=`14-N:L554` (14-N-SUMMARY_INDEX.discovery.md:L554): Count off-agent anchors as exogenous (`L336-L338`).
- **SUMMARY/N16-E16** [E] anchor=`14-N:L555` (14-N-SUMMARY_INDEX.discovery.md:L555): Count retrieval anchors as exogenous (`L336-L338`).
- **SUMMARY/N16-E17** [E] anchor=`14-N:L556` (14-N-SUMMARY_INDEX.discovery.md:L556): Count human anchors as exogenous (`L336-L338`).
- **SUMMARY/N16-E18** [E] anchor=`14-N:L557` (14-N-SUMMARY_INDEX.discovery.md:L557): Below the lambda floor, fire a retrieval/escalation brake (`L337-L339`).
- **SUMMARY/N16-E19** [E] anchor=`14-N:L558` (14-N-SUMMARY_INDEX.discovery.md:L558): Never use lambda as a PASS signal (`L338-L339`).
- **SUMMARY/N16-E20** [E] anchor=`14-N:L559` (14-N-SUMMARY_INDEX.discovery.md:L559): Interpret lambda as a detector of stalled/closed-loop dynamics, not wrong-but-stable consensus (`L338-L339`).
- **SUMMARY/N16-E21** [E] anchor=`14-N:L560` (14-N-SUMMARY_INDEX.discovery.md:L560): Measure auditor finding-class entropy to distinguish probing new surfaces from relitigating the same nodes (`L339-L340`).
- **SUMMARY/N16-E22** [E] anchor=`14-N:L561` (14-N-SUMMARY_INDEX.discovery.md:L561): Measure criticism debt as the fraction of shipped units whose declared VERIFY leg never ran (`L340-L341`).
- **SUMMARY/N16-E23** [E] anchor=`14-N:L562` (14-N-SUMMARY_INDEX.discovery.md:L562): Measure the rate at which gates are challenged or amended (`L340-L341`).
- **SUMMARY/N16-E24** [E] anchor=`14-N:L563` (14-N-SUMMARY_INDEX.discovery.md:L563): Keep all three new views advisory-only under TD-3 (`L341-L342`).
- **SUMMARY/N16-E25** [E] anchor=`14-N:L564` (14-N-SUMMARY_INDEX.discovery.md:L564): Measure all governance values through N-1 under ND-4 (`L341-L345`).
- **SUMMARY/N16-E26** [E] anchor=`14-N:L565` (14-N-SUMMARY_INDEX.discovery.md:L565): Use ELAI-measured thresholds under ID-5 (`L341-L342`).
- **SUMMARY/N16-E27** [E] anchor=`14-N:L566` (14-N-SUMMARY_INDEX.discovery.md:L566): Reuse N-8’s existing five views, F-5, H-27/I-24/N-15 detectors, the gate manifest/freeze checker, D-17 escalation, and R4 off-agent doctrine (`L343-L346`).
- **SUMMARY/N16-E28** [E] anchor=`14-N:L567` (14-N-SUMMARY_INDEX.discovery.md:L567): Add the verdict field, F-5 cascade hook, and lambda/ritual derivations in `crates/orchestrator` (`L347-L348`).
- **SUMMARY/N16-E29** [E] anchor=`14-N:L568` (14-N-SUMMARY_INDEX.discovery.md:L568): Extend the `.claude/tools` governance-coverage checker with the three views (`L347-L349`).
- **SUMMARY/N16-E30** [E] [SURFACE] anchor=`14-N:L569` (14-N-SUMMARY_INDEX.discovery.md:L569): Add a lambda gauge, quarantine-queue count, and ritual-entropy band to the N-8/G-5/M-3 governance tile (`L348-L349`).
- **SUMMARY/N16-E31** [E] anchor=`14-N:L570` (14-N-SUMMARY_INDEX.discovery.md:L570): A refutation followed by a governance run emits `+evaluator_closure_swept` and queues the evaluator’s historical verdicts (`L350-L352`).
- **SUMMARY/N16-E32** [E] anchor=`14-N:L571` (14-N-SUMMARY_INDEX.discovery.md:L571): Lambda below its floor emits `+grounding_ratio_measured BelowFloorBraked` and activates the brake (`L351-L352`).
- **SUMMARY/N16-E33** [E] anchor=`14-N:L572` (14-N-SUMMARY_INDEX.discovery.md:L572): An unrun VERIFY leg on a shipped unit increments criticism debt (`L352`).
- **SUMMARY/N16-E34** [E] anchor=`14-N:L573` (14-N-SUMMARY_INDEX.discovery.md:L573): Silently deleting a quarantined verdict fails the append-only invariant (`L353-L354`).
- **SUMMARY/N16-E35** [E] anchor=`14-N:L574` (14-N-SUMMARY_INDEX.discovery.md:L574): Using lambda or ritual metrics as a PASS gate fails TD-3 (`L353-L354`).
- **SUMMARY/N16-E36** [E] anchor=`14-N:L575` (14-N-SUMMARY_INDEX.discovery.md:L575): Asserting a governance number without N-1 measurement fails ND-4 (`L353-L354`).
- **SUMMARY/N16-A01** [A] anchor=`14-N:L579` (14-N-SUMMARY_INDEX.discovery.md:L579): Window length, lambda numerator/denominator edge cases, floor, brake duration, re-audit release criteria, evaluator identity migration, and entropy formula are unspecified.
- **SUMMARY/N16-A02** [A] anchor=`14-N:L580` (14-N-SUMMARY_INDEX.discovery.md:L580): “frozen-tool bugfix” event registration and version-hash algorithm are unspecified.
- **SUMMARY/N16-A03** [A] anchor=`14-N:L581` (14-N-SUMMARY_INDEX.discovery.md:L581): “PLAN-asserted, re-grep W-7” explicitly lowers confidence in the local anchor verification even though the steal’s grounding is labeled STRONG (`L327-L328`).

#### `Inferred interactions (not promoted to requirements)` (6 records)

- **SUMMARY/N-I01** [I] anchor=`14-N:L816` (14-N-SUMMARY_INDEX.discovery.md:L816): N-1 is the likely foundation for every “ELAI-measured” N threshold even when metadata does not list `depends N-1`; this is explicit for N-13..N-16 consumption but not a formal dependency for every earlier unit.
- **SUMMARY/N-I02** [I] anchor=`14-N:L817` (14-N-SUMMARY_INDEX.discovery.md:L817): N-2 likely becomes the enforcement mechanism for N-7, N-12, and N-13 authoring invariants; those ties are explicit, but whether it enforces every other N canonicality rule is only implied by the global anti-fork doctrine.
- **SUMMARY/N-I03** [I] anchor=`14-N:L818` (14-N-SUMMARY_INDEX.discovery.md:L818): N-4’s unified Claim/Belief object, N-6’s provenance carrier, N-13’s ClaimSet boundary, and N-7’s render substrate likely form a single end-to-end object lifecycle; individual pairwise roles are explicit, exact execution order is not.
- **SUMMARY/N-I04** [I] anchor=`14-N:L819` (14-N-SUMMARY_INDEX.discovery.md:L819): N-8 plus N-16 yields eight governance views (five + three); the summary calls N-16 an extension over the same question but never names a formal eight-view schema.
- **SUMMARY/N-I05** [I] [SURFACE] anchor=`14-N:L820` (14-N-SUMMARY_INDEX.discovery.md:L820): N-11’s shared journal folds likely supply N-8/N-16 governance tiles and other N observability, but only the explicitly named five surface folds are requirements.
- **SUMMARY/N-I06** [I] anchor=`14-N:L821` (14-N-SUMMARY_INDEX.discovery.md:L821): N-12’s learned-artifact admission sub-slice likely uses N-6’s unified admission registry because both name J-109; the summary does not explicitly wire N-12(d) to N-6.

### SUMMARY prose / workflow / constraint / surface / open-ambiguity atoms (133 records)

#### `Program-level purpose and anti-fragmentation doctrine` (10 records)

- **SUMMARY/DOC-001** [E] anchor=`14-N:L32` (14-N-SUMMARY_INDEX.discovery.md:L32): N is the **INTERLACE** letter: it primarily connects and consolidates capabilities from B..M rather than inventing new capability.
- **SUMMARY/DOC-002** [E] anchor=`14-N:L33` (14-N-SUMMARY_INDEX.discovery.md:L33): The identified systemic failure is summary-driven plan authorship causing same-concern forks: four claim types, ten routers, six intensity deciders, multiple provenance scales, four counterfactual engines, and multiple context cores.
- **SUMMARY/DOC-003** [E] anchor=`14-N:L34` (14-N-SUMMARY_INDEX.discovery.md:L34): N requires one canonical subsystem per fragmented concern.
- **SUMMARY/DOC-004** [E] anchor=`14-N:L35` (14-N-SUMMARY_INDEX.discovery.md:L35): N mechanizes the M-1 anti-fragmentation rule so later letters cannot silently fork a concern again.
- **SUMMARY/DOC-005** [E] anchor=`14-N:L36` (14-N-SUMMARY_INDEX.discovery.md:L36): N-2 lifts wiring enforcement from code grain to the complete plan-corpus grain.
- **SUMMARY/DOC-006** [E] anchor=`14-N:L37` (14-N-SUMMARY_INDEX.discovery.md:L37): Every entry is intended to preserve the discharged W finding(s), impact tier, grounding strength, nature, unified implementations, consumed artifacts, real integration point, and observable-effect done gate.
- **SUMMARY/DOC-007** [E] anchor=`14-N:L38` (14-N-SUMMARY_INDEX.discovery.md:L38): The normal N negative arm is an anti-fork assertion: a second implementation of a unified concern must fail.
- **SUMMARY/DOC-008** [E] anchor=`14-N:L39` (14-N-SUMMARY_INDEX.discovery.md:L39): N-2 is exceptional because its done gate executes at plan-authoring time, not runtime.
- **SUMMARY/DOC-009** [E] anchor=`14-N:L40` (14-N-SUMMARY_INDEX.discovery.md:L40): After N-2, plans failing to compose is intended to become a machine-caught block rather than a prose-only concern.
- **SUMMARY/DOC-010** [E] anchor=`14-N:L41` (14-N-SUMMARY_INDEX.discovery.md:L41): Definitions of ND-1..ND-5 and sequencing are delegated to `N-0_INDEX.md`; per-finding evidence is delegated to `workflow/reasoning/elai-weakpoints-interlace-2026-07-04.md` (`L7-L8`).

#### `Measurement-to-decision loop` (8 records)

- **SUMMARY/WF1-001** [E] anchor=`14-N:L589` (14-N-SUMMARY_INDEX.discovery.md:L589): J-56/E-54/L-20 and the named feeders provide evaluation material to N-1.
- **SUMMARY/WF1-002** [E] [SURFACE] anchor=`14-N:L590` (14-N-SUMMARY_INDEX.discovery.md:L590): A family uses the one K-15/I-27-shaped runner via `elai bench *`.
- **SUMMARY/WF1-003** [E] anchor=`14-N:L591` (14-N-SUMMARY_INDEX.discovery.md:L591): The run appends a corpus row, updates the one I-51 `ModelCapabilityProfile`, and supplies outcome labels.
- **SUMMARY/WF1-004** [E] anchor=`14-N:L592` (14-N-SUMMARY_INDEX.discovery.md:L592): Local calibration hard-supersedes matching BOOTSTRAP defaults and records the supersession.
- **SUMMARY/WF1-005** [E] anchor=`14-N:L593` (14-N-SUMMARY_INDEX.discovery.md:L593): N-3 reads the outcome labels and trial ledger to deflate promotion results and apply reusable-holdout budgets.
- **SUMMARY/WF1-006** [E] anchor=`14-N:L594` (14-N-SUMMARY_INDEX.discovery.md:L594): N-9 uses the same outcomes for drop-one/inject-one counterfactual scoring.
- **SUMMARY/WF1-007** [E] anchor=`14-N:L595` (14-N-SUMMARY_INDEX.discovery.md:L595): N-13 writes per-type/backend invalid rates into the same spine.
- **SUMMARY/WF1-008** [E] anchor=`14-N:L596` (14-N-SUMMARY_INDEX.discovery.md:L596): N-14 supplies edit kind and improvement timing; N-15 supplies framing/adoption measurements; N-16 supplies governance measurements.

#### `Plan-authoring anti-fork loop` (6 records)

- **SUMMARY/WF2-001** [E] anchor=`14-N:L602` (14-N-SUMMARY_INDEX.discovery.md:L602): Plan authors encode map edges in ND-3 data form.
- **SUMMARY/WF2-002** [E] anchor=`14-N:L603` (14-N-SUMMARY_INDEX.discovery.md:L603): N-2 joins map edges, plan units, and `elai.map` symbols.
- **SUMMARY/WF2-003** [E] anchor=`14-N:L604` (14-N-SUMMARY_INDEX.discovery.md:L604): It checks dangling consumption, orphan production, duplicate concepts, freshness, and extended type completeness.
- **SUMMARY/WF2-004** [E] anchor=`14-N:L605` (14-N-SUMMARY_INDEX.discovery.md:L605): It emits an unlock DAG/fan-in rank and mechanically derives summary/status material.
- **SUMMARY/WF2-005** [E] anchor=`14-N:L606` (14-N-SUMMARY_INDEX.discovery.md:L606): It also enforces required render fields (N-7), output-boundary registry rows (N-13), and the no-second-implementation rule used by N-12/N-13.
- **SUMMARY/WF2-006** [E] anchor=`14-N:L607` (14-N-SUMMARY_INDEX.discovery.md:L607): Any seeded invalid case must fail closed before the plan ships.

#### `Claim, receipt, provenance, and output path` (5 records)

- **SUMMARY/WF3-001** [E] anchor=`14-N:L613` (14-N-SUMMARY_INDEX.discovery.md:L613): **Explicit:** N-4 represents typed claims, beliefs, critical atoms, receipts, and grounding tags as one object.
- **SUMMARY/WF3-002** [E] anchor=`14-N:L614` (14-N-SUMMARY_INDEX.discovery.md:L614): **Explicit:** A repeated claim checks J-96 receipt cache identity; unchanged claims can reuse a receipt, changed claims are invalidated by F-5.
- **SUMMARY/WF3-003** [E] anchor=`14-N:L615` (14-N-SUMMARY_INDEX.discovery.md:L615): **Explicit:** N-6 maps trust semantics while the object travels ingest → memory → citation → oracle TTL → skill autonomy → cross-repo.
- **SUMMARY/WF3-004** [I] anchor=`14-N:L616` (14-N-SUMMARY_INDEX.discovery.md:L616): **Inferred interaction:** A machine-generated ClaimSet likely crosses N-13’s `OutputBoundary` before N-4/J-83 uses it, because ClaimSet is a named boundary type, but the summary does not state the exact temporal order.
- **SUMMARY/WF3-005** [E] anchor=`14-N:L617` (14-N-SUMMARY_INDEX.discovery.md:L617): **Explicit:** Every resulting gate/verdict declares an N-7 human render destination.

#### `Routing and resource-allocation loop` (5 records)

- **SUMMARY/WF4-001** [E] anchor=`14-N:L623` (14-N-SUMMARY_INDEX.discovery.md:L623): N-10 joins ambiguity and novelty and sends the shared score to research depth, reasoning effort, node sizing, and clarification.
- **SUMMARY/WF4-002** [E] anchor=`14-N:L624` (14-N-SUMMARY_INDEX.discovery.md:L624): N-14 adds synthesis-versus-tuning task demand as another policy/intensity axis.
- **SUMMARY/WF4-003** [E] anchor=`14-N:L625` (14-N-SUMMARY_INDEX.discovery.md:L625): Existing routers and intensity deciders fire independently.
- **SUMMARY/WF4-004** [E] anchor=`14-N:L626` (14-N-SUMMARY_INDEX.discovery.md:L626): N-5 resolves conflict by frozen precedence and emits one explained `RouteDecision`.
- **SUMMARY/WF4-005** [E] anchor=`14-N:L627` (14-N-SUMMARY_INDEX.discovery.md:L627): N-16 can activate a retrieval/escalation brake when exogenous grounding falls below its floor; it cannot create a pass.

#### `Governance lifecycle` (7 records)

- **SUMMARY/WF5-001** [E] anchor=`14-N:L633` (14-N-SUMMARY_INDEX.discovery.md:L633): A gate/verdict class must declare both its SSB representation and human render.
- **SUMMARY/WF5-002** [E] anchor=`14-N:L634` (14-N-SUMMARY_INDEX.discovery.md:L634): A new gate must have a mutant corpus and measured kill rate before entering the frozen manifest.
- **SUMMARY/WF5-003** [E] anchor=`14-N:L635` (14-N-SUMMARY_INDEX.discovery.md:L635): Governance coverage reports liveness, enforcer coverage, debt rate, traceability, and mutation kill performance.
- **SUMMARY/WF5-004** [E] anchor=`14-N:L636` (14-N-SUMMARY_INDEX.discovery.md:L636): Each verdict also records evaluator identity/version.
- **SUMMARY/WF5-005** [E] anchor=`14-N:L637` (14-N-SUMMARY_INDEX.discovery.md:L637): An evaluator refutation invokes F-5 to quarantine exactly its verdicts into advisory re-audit without deletion.
- **SUMMARY/WF5-006** [E] anchor=`14-N:L638` (14-N-SUMMARY_INDEX.discovery.md:L638): The extended governance run also reports exogenous-grounding ratio, finding-class entropy, unrun verification debt, and gate-challenge rate.
- **SUMMARY/WF5-007** [E] anchor=`14-N:L639` (14-N-SUMMARY_INDEX.discovery.md:L639): These closure metrics are advisory/braking signals, never pass signals.

#### `Journal derivation and retirement` (6 records)

- **SUMMARY/WF6-001** [E] anchor=`14-N:L645` (14-N-SUMMARY_INDEX.discovery.md:L645): H-49 CAS/undo-tree is declared the one log representation.
- **SUMMARY/WF6-002** [E] anchor=`14-N:L646` (14-N-SUMMARY_INDEX.discovery.md:L646): L-16’s pure fold core accepts surface-specific fold functions as data/plugins.
- **SUMMARY/WF6-003** [E] [SURFACE] anchor=`14-N:L647` (14-N-SUMMARY_INDEX.discovery.md:L647): Canvas altitude, plan status, billing balance, fleet rollup, and coverage derive through it.
- **SUMMARY/WF6-004** [E] anchor=`14-N:L648` (14-N-SUMMARY_INDEX.discovery.md:L648): One Kani proof covers the shared fold machinery.
- **SUMMARY/WF6-005** [E] anchor=`14-N:L649` (14-N-SUMMARY_INDEX.discovery.md:L649): Pinned-seed conformance requires byte-identical rederivation.
- **SUMMARY/WF6-006** [E] anchor=`14-N:L650` (14-N-SUMMARY_INDEX.discovery.md:L650): ND-2 retirement ends `recover_stubs` and removes ad-hoc derivation paths.

#### `Global constraints and invariants` (14 records)

- **SUMMARY/INV-001** [E] anchor=`14-N:L654` (14-N-SUMMARY_INDEX.discovery.md:L654): **Canonicality:** one subsystem/registry/core per concern; adapters parameterize or delegate but do not carry duplicate core logic (`L12-L18,L34-L36,L242-L248,L356-L361`).
- **SUMMARY/INV-002** [E] anchor=`14-N:L655` (14-N-SUMMARY_INDEX.discovery.md:L655): **Consume, do not recreate (GD-1):** N repeatedly extends shipped/planned artifacts rather than replacing their engines.
- **SUMMARY/INV-003** [E] anchor=`14-N:L656` (14-N-SUMMARY_INDEX.discovery.md:L656): **Fail closed (R1):** authoring checks, missing trial counts, missing boundary rows, policy arbitration omissions, stale receipt hits, taint loss, and unsupported output enforcement all have negative arms.
- **SUMMARY/INV-004** [E] anchor=`14-N:L657` (14-N-SUMMARY_INDEX.discovery.md:L657): **Data over prose:** map edges use ND-3 data, precedence is a frozen data table, OutputBoundary is a registry, and fold functions are data/plugins.
- **SUMMARY/INV-005** [E] anchor=`14-N:L658` (14-N-SUMMARY_INDEX.discovery.md:L658): **Measurement authority (ND-4/ID-5):** one `ModelCapabilityProfile` and N-1 spine own calibration; N-14/N-15/N-16 thresholds must be locally measured.
- **SUMMARY/INV-006** [E] anchor=`14-N:L659` (14-N-SUMMARY_INDEX.discovery.md:L659): **Bootstrap provenance:** cold-start values are explicitly BOOTSTRAP and cannot remain authoritative after local data exists.
- **SUMMARY/INV-007** [E] anchor=`14-N:L660` (14-N-SUMMARY_INDEX.discovery.md:L660): **Append-only evidence:** F-5 quarantine never deletes historical verdicts.
- **SUMMARY/INV-008** [E] anchor=`14-N:L661` (14-N-SUMMARY_INDEX.discovery.md:L661): **Structural separation (TD-3):** the model does not choose enforcement; advisory instruments cannot become pass gates.
- **SUMMARY/INV-009** [E] anchor=`14-N:L662` (14-N-SUMMARY_INDEX.discovery.md:L662): **Security preservation:** taint must survive all mapped boundaries; F-6’s type barrier remains; grammar pins, reject-never-repair, frozen repair tables, and admission scanners remain binding.
- **SUMMARY/INV-010** [E] [SURFACE] anchor=`14-N:L663` (14-N-SUMMARY_INDEX.discovery.md:L663): **Human visibility:** every verdict family must declare a render, with N-7 frontend pairing mandatory.
- **SUMMARY/INV-011** [E] anchor=`14-N:L664` (14-N-SUMMARY_INDEX.discovery.md:L664): **Thin adapters (ND-5):** counterfactual and other unified call-site adapters contain no business logic.
- **SUMMARY/INV-012** [E] anchor=`14-N:L665` (14-N-SUMMARY_INDEX.discovery.md:L665): **Retirement (ND-2):** a unification is incomplete while an ad-hoc predecessor path remains live.
- **SUMMARY/INV-013** [E] anchor=`14-N:L666` (14-N-SUMMARY_INDEX.discovery.md:L666): **Determinism:** N-11 uses a pinned seed and byte-identical outputs; N-14 has an `ELAI_SEED` classification test.
- **SUMMARY/INV-014** [E] anchor=`14-N:L667` (14-N-SUMMARY_INDEX.discovery.md:L667): **False-positive control (GD-4):** N-15 cannot default on without a measured FP bound.

#### `CLI and authoring commands` (4 records)

- **SUMMARY/CLI-001** [E] [SURFACE] anchor=`14-N:L673` (14-N-SUMMARY_INDEX.discovery.md:L673): `elai bench <any-family>` / `elai bench *` invokes N-1 families.
- **SUMMARY/CLI-002** [E] [SURFACE] anchor=`14-N:L674` (14-N-SUMMARY_INDEX.discovery.md:L674): `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` invokes N-2.
- **SUMMARY/CLI-003** [E] [SURFACE] anchor=`14-N:L675` (14-N-SUMMARY_INDEX.discovery.md:L675): `elai bench sycophancy` invokes N-15 fixtures.
- **SUMMARY/CLI-004** [E] [SURFACE] anchor=`14-N:L676` (14-N-SUMMARY_INDEX.discovery.md:L676): No public network API, protocol, authentication flow, or remote service API is specified in this summary.

#### `Canonical state/data structures` (16 records)

- **SUMMARY/STATE-001** [E] anchor=`14-N:L680` (14-N-SUMMARY_INDEX.discovery.md:L680): I-51 `ModelCapabilityProfile` / `models.toml`: only calibration sink.
- **SUMMARY/STATE-002** [E] anchor=`14-N:L681` (14-N-SUMMARY_INDEX.discovery.md:L681): Corpus row + calibration curve + outcome-label fold: measurement output.
- **SUMMARY/STATE-003** [E] anchor=`14-N:L682` (14-N-SUMMARY_INDEX.discovery.md:L682): `Claim/Belief` object: unified statement/evidence carrier.
- **SUMMARY/STATE-004** [E] anchor=`14-N:L683` (14-N-SUMMARY_INDEX.discovery.md:L683): J-96 case graph: receipt cache keyed with C-30 discipline.
- **SUMMARY/STATE-005** [E] anchor=`14-N:L684` (14-N-SUMMARY_INDEX.discovery.md:L684): Composite `RouteDecision`: fired routers, winner, precedence rank.
- **SUMMARY/STATE-006** [E] anchor=`14-N:L685` (14-N-SUMMARY_INDEX.discovery.md:L685): Semantic trust lattice mapping table: maps six intentionally distinct trust scales.
- **SUMMARY/STATE-007** [E] anchor=`14-N:L686` (14-N-SUMMARY_INDEX.discovery.md:L686): Admission scanner registry: shared ingress scanner family.
- **SUMMARY/STATE-008** [E] anchor=`14-N:L687` (14-N-SUMMARY_INDEX.discovery.md:L687): Gate/verdict `render_field`: required authoring contract data.
- **SUMMARY/STATE-009** [E] anchor=`14-N:L688` (14-N-SUMMARY_INDEX.discovery.md:L688): Governance substrate: five original views plus N-16’s three extension views.
- **SUMMARY/STATE-010** [E] anchor=`14-N:L689` (14-N-SUMMARY_INDEX.discovery.md:L689): Counterfactual target Kind: role/verifier/ranker/rule/memory/case.
- **SUMMARY/STATE-011** [E] anchor=`14-N:L690` (14-N-SUMMARY_INDEX.discovery.md:L690): Joined `AmbiguityScore` + `NoveltyScore` rows.
- **SUMMARY/STATE-012** [E] anchor=`14-N:L691` (14-N-SUMMARY_INDEX.discovery.md:L691): H-49 CAS/undo-tree log plus per-surface fold functions.
- **SUMMARY/STATE-013** [E] anchor=`14-N:L692` (14-N-SUMMARY_INDEX.discovery.md:L692): `FailureClass`, `AbstainDecision`, `ProjectClass`, shared drift primitives, FSM conformance models, and LSP daemon pool.
- **SUMMARY/STATE-014** [E] anchor=`14-N:L693` (14-N-SUMMARY_INDEX.discovery.md:L693): `OutputBoundary` registry rows with schema/parser/repair/retry/fail-closed/invalid-rate/enforcement fields.
- **SUMMARY/STATE-015** [E] anchor=`14-N:L694` (14-N-SUMMARY_INDEX.discovery.md:L694): `EditKind` with `SynthesisEdit` and `ParametricEdit`.
- **SUMMARY/STATE-016** [E] anchor=`14-N:L695` (14-N-SUMMARY_INDEX.discovery.md:L695): SSB verdict evaluator identity/version and advisory re-audit queue.

#### `Configuration and policy` (7 records)

- **SUMMARY/CFG-001** [E] anchor=`14-N:L699` (14-N-SUMMARY_INDEX.discovery.md:L699): BOOTSTRAP calibration pack with provenance and hard local supersession.
- **SUMMARY/CFG-002** [E] anchor=`14-N:L700` (14-N-SUMMARY_INDEX.discovery.md:L700): Thresholdout budget on each reusable evaluation corpus.
- **SUMMARY/CFG-003** [E] anchor=`14-N:L701` (14-N-SUMMARY_INDEX.discovery.md:L701): Frozen routing precedence: safety > cost > abstention > escalation > modality > serving-tier.
- **SUMMARY/CFG-004** [E] anchor=`14-N:L702` (14-N-SUMMARY_INDEX.discovery.md:L702): Per-output-type retry and repair/fail-closed policy plus backend enforcement capability.
- **SUMMARY/CFG-005** [E] anchor=`14-N:L703` (14-N-SUMMARY_INDEX.discovery.md:L703): ELAI-measured thresholds only for edit-loop, framing, and governance instruments.
- **SUMMARY/CFG-006** [E] anchor=`14-N:L704` (14-N-SUMMARY_INDEX.discovery.md:L704): GD-4 measured FP bound gates N-15 default-on status.
- **SUMMARY/CFG-007** [E] anchor=`14-N:L705` (14-N-SUMMARY_INDEX.discovery.md:L705): Exact config syntax/default values are deferred to detail plans.

#### `Error, fallback, and recovery behavior` (9 records)

- **SUMMARY/ERR-001** [E] anchor=`14-N:L730` (14-N-SUMMARY_INDEX.discovery.md:L730): Fresh-install measurement fallback uses BOOTSTRAP defaults until hard-superseded locally.
- **SUMMARY/ERR-002** [E] anchor=`14-N:L731` (14-N-SUMMARY_INDEX.discovery.md:L731): Missing promotion N is fail-closed; below-threshold N holds promotion.
- **SUMMARY/ERR-003** [E] anchor=`14-N:L732` (14-N-SUMMARY_INDEX.discovery.md:L732): Conflicting routers fall back to the frozen precedence plane, not call order.
- **SUMMARY/ERR-004** [E] anchor=`14-N:L733` (14-N-SUMMARY_INDEX.discovery.md:L733): Changed claim identity invalidates receipt reuse through F-5.
- **SUMMARY/ERR-005** [E] anchor=`14-N:L734` (14-N-SUMMARY_INDEX.discovery.md:L734): Trust-label reset or taint loss at a boundary is a hard failure.
- **SUMMARY/ERR-006** [E] anchor=`14-N:L735` (14-N-SUMMARY_INDEX.discovery.md:L735): Output enforcement degrades only through the fixed strongest-available order and ultimately parse/validate/retry/fail-closed; D-16 provides refusal.
- **SUMMARY/ERR-007** [E] anchor=`14-N:L736` (14-N-SUMMARY_INDEX.discovery.md:L736): Low exogenous-grounding ratio triggers retrieval/escalation braking, never passing.
- **SUMMARY/ERR-008** [E] anchor=`14-N:L737` (14-N-SUMMARY_INDEX.discovery.md:L737): Refuted-evaluator verdicts are quarantined for advisory re-audit, never removed.
- **SUMMARY/ERR-009** [E] anchor=`14-N:L738` (14-N-SUMMARY_INDEX.discovery.md:L738): No backup/restore, retention duration, privacy-redaction, or disaster-recovery policy is explicit beyond these behaviors.

#### `UI surfaces` (15 records)

- **SUMMARY/UI-001** [E] [SURFACE] anchor=`14-N:L742` (14-N-SUMMARY_INDEX.discovery.md:L742): N-1 `MeasurementPanel` or justified reuse of G-5.
- **SUMMARY/UI-002** [E] [SURFACE] anchor=`14-N:L743` (14-N-SUMMARY_INDEX.discovery.md:L743): N-3 deflation column in promotion drawers or justified N/A.
- **SUMMARY/UI-003** [E] [SURFACE] anchor=`14-N:L744` (14-N-SUMMARY_INDEX.discovery.md:L744): N-4 memo-hit indicator in belief/claim drawers or justified N/A.
- **SUMMARY/UI-004** [E] [SURFACE] anchor=`14-N:L745` (14-N-SUMMARY_INDEX.discovery.md:L745): N-5 canvas/RRA route-explanation row.
- **SUMMARY/UI-005** [E] [SURFACE] anchor=`14-N:L746` (14-N-SUMMARY_INDEX.discovery.md:L746): N-6 provenance badge on citation/memory views or justified N/A.
- **SUMMARY/UI-006** [E] [SURFACE] anchor=`14-N:L747` (14-N-SUMMARY_INDEX.discovery.md:L747): N-7 mandatory PR/chat/canvas/brief/rollup render substrate.
- **SUMMARY/UI-007** [E] [SURFACE] anchor=`14-N:L748` (14-N-SUMMARY_INDEX.discovery.md:L748): N-8 governance tile on G-5/M-3.
- **SUMMARY/UI-008** [E] [SURFACE] anchor=`14-N:L749` (14-N-SUMMARY_INDEX.discovery.md:L749): N-9 existing attribution drawers or justified N/A.
- **SUMMARY/UI-009** [E] [SURFACE] anchor=`14-N:L750` (14-N-SUMMARY_INDEX.discovery.md:L750): N-10 normally internal; optionally show chosen depth on canvas.
- **SUMMARY/UI-010** [E] [SURFACE] anchor=`14-N:L751` (14-N-SUMMARY_INDEX.discovery.md:L751): N-11 verify existing canvas fold consumption.
- **SUMMARY/UI-011** [E] [SURFACE] anchor=`14-N:L752` (14-N-SUMMARY_INDEX.discovery.md:L752): N-12 normally N/A; optional `FailureClass` drawer.
- **SUMMARY/UI-012** [E] [SURFACE] anchor=`14-N:L753` (14-N-SUMMARY_INDEX.discovery.md:L753): N-13 boundary-verdict badge or G-5 tile reuse.
- **SUMMARY/UI-013** [E] [SURFACE] anchor=`14-N:L754` (14-N-SUMMARY_INDEX.discovery.md:L754): N-14 edit-kind chip or justified internal N/A.
- **SUMMARY/UI-014** [E] [SURFACE] anchor=`14-N:L755` (14-N-SUMMARY_INDEX.discovery.md:L755): N-15 G-5/N-8 framing-stability tile or existing sycophancy tile reuse.
- **SUMMARY/UI-015** [E] [SURFACE] anchor=`14-N:L756` (14-N-SUMMARY_INDEX.discovery.md:L756): N-16 lambda gauge, quarantine count, and ritual-entropy band on governance tile.

#### `Explicitly rejected, already-covered, note-only, or future/deferred items` (9 records)

- **SUMMARY/REJ-001** [E] anchor=`14-N:L804` (14-N-SUMMARY_INDEX.discovery.md:L804): Rejected from build: PAW compiled-LoRA tier.
- **SUMMARY/REJ-002** [E] anchor=`14-N:L805` (14-N-SUMMARY_INDEX.discovery.md:L805): Rejected as evidence/build basis: EvoPolicyGym n=1 statistics and missing memorization defense; n=1 caveat retained.
- **SUMMARY/REJ-003** [E] anchor=`14-N:L806` (14-N-SUMMARY_INDEX.discovery.md:L806): Rejected as new work: gaps P3-P5 because already planned deeper.
- **SUMMARY/REJ-004** [E] anchor=`14-N:L807` (14-N-SUMMARY_INDEX.discovery.md:L807): Rejected as a build unit: Ford checklist.
- **SUMMARY/REJ-005** [E] anchor=`14-N:L808` (14-N-SUMMARY_INDEX.discovery.md:L808): Already recorded: prefix-KV S1-1 via J-95/H-50.
- **SUMMARY/REJ-006** [E] anchor=`14-N:L809` (14-N-SUMMARY_INDEX.discovery.md:L809): Folded into an existing N unit: zip-slip S1-2 into N-6.
- **SUMMARY/REJ-007** [E] anchor=`14-N:L810` (14-N-SUMMARY_INDEX.discovery.md:L810): Note/dossier only: ADR evidence-table S1-3; packaging S4-1/S3-4/S1-6; EvoPolicyGym bench S2-4.
- **SUMMARY/REJ-008** [E] anchor=`14-N:L811` (14-N-SUMMARY_INDEX.discovery.md:L811): Rejected from N-16: Dung argument-graph adjudicator.
- **SUMMARY/REJ-009** [E] anchor=`14-N:L812` (14-N-SUMMARY_INDEX.discovery.md:L812): Deferred to detail/original plans: all thresholds, schemas, exact mappings, migration mechanics, formulae, and unresolved ambiguities listed per unit.

#### `Open ambiguities and missing specification candidates` (12 records)

- **SUMMARY/OA-001** [A] anchor=`14-N:L827` (14-N-SUMMARY_INDEX.discovery.md:L827): Header count: 15 sessions vs 16 unit headings.
- **SUMMARY/OA-002** [A] anchor=`14-N:L828` (14-N-SUMMARY_INDEX.discovery.md:L828): N-12 count: eight independent collapses vs seven labeled sub-slices.
- **SUMMARY/OA-003** [A] anchor=`14-N:L829` (14-N-SUMMARY_INDEX.discovery.md:L829): N-13 count: four emission seams vs three named examples.
- **SUMMARY/OA-004** [A] anchor=`14-N:L830` (14-N-SUMMARY_INDEX.discovery.md:L830): Exact ND-1..ND-5 rules, sequencing, and retirement/adaptor requirements in `N-0_INDEX.md`.
- **SUMMARY/OA-005** [A] anchor=`14-N:L831` (14-N-SUMMARY_INDEX.discovery.md:L831): Complete W-finding evidence and whether all W references remain current.
- **SUMMARY/OA-006** [A] anchor=`14-N:L832` (14-N-SUMMARY_INDEX.discovery.md:L832): Exact schemas for map edges, Claim/Belief, RouteDecision, trust mapping, governance Kinds, fold plugins, OutputBoundary, EditKind, and evaluator identity.
- **SUMMARY/OA-007** [A] anchor=`14-N:L833` (14-N-SUMMARY_INDEX.discovery.md:L833): Every threshold, budget, window, tie-breaker, and false-positive bound.
- **SUMMARY/OA-008** [A] anchor=`14-N:L834` (14-N-SUMMARY_INDEX.discovery.md:L834): Lifecycle/migration of old duplicated types, stores, runners, caches, journal paths, validators, and daemons.
- **SUMMARY/OA-009** [A] anchor=`14-N:L835` (14-N-SUMMARY_INDEX.discovery.md:L835): UI target mapping and fallback behavior when a declared human render cannot be delivered.
- **SUMMARY/OA-010** [A] anchor=`14-N:L836` (14-N-SUMMARY_INDEX.discovery.md:L836): Privacy/retention/access policy for claim receipts, eval corpora, model profiles, evaluator histories, and re-audit queues; the summary is silent.
- **SUMMARY/OA-011** [A] anchor=`14-N:L837` (14-N-SUMMARY_INDEX.discovery.md:L837): Concrete tests beyond each observable-effect positive/negative arm and the named Kani/FSM/normalization checks.
- **SUMMARY/OA-012** [A] anchor=`14-N:L838` (14-N-SUMMARY_INDEX.discovery.md:L838): Implementation status versus plan status; this source is a specification summary, not a completion ledger.

### SUMMARY table-derived atoms (48 records)

#### `Named observability artifacts` (16 records)

- **SUMMARY/OBS-001** [E] anchor=`14-N:L711` (14-N-SUMMARY_INDEX.discovery.md:L711): N-1: corpus row, calibration curve, `+calibration_bootstrap_superseded`
- **SUMMARY/OBS-002** [E] anchor=`14-N:L712` (14-N-SUMMARY_INDEX.discovery.md:L712): N-2: unlock DAG, fan-in ranking, derived summaries/status, `+composition_edge_checked`, process exit status
- **SUMMARY/OBS-003** [E] anchor=`14-N:L713` (14-N-SUMMARY_INDEX.discovery.md:L713): N-3: `+promotion_deflated(DeflationBlocked)` and held lifecycle state
- **SUMMARY/OBS-004** [E] anchor=`14-N:L714` (14-N-SUMMARY_INDEX.discovery.md:L714): N-4: J-96 receipt node, `+receipt_memo(Hit)`, checker skip
- **SUMMARY/OBS-005** [E] anchor=`14-N:L715` (14-N-SUMMARY_INDEX.discovery.md:L715): N-5: composite `RouteDecision`, `+route_arbitrated`
- **SUMMARY/OBS-006** [E] [SURFACE] anchor=`14-N:L716` (14-N-SUMMARY_INDEX.discovery.md:L716): N-6: carried trust label and optional provenance badge; no named SSB row is given
- **SUMMARY/OBS-007** [E] [SURFACE] anchor=`14-N:L717` (14-N-SUMMARY_INDEX.discovery.md:L717): N-7: human render at PR/chat/canvas/brief/rollup target
- **SUMMARY/OBS-008** [E] [SURFACE] anchor=`14-N:L718` (14-N-SUMMARY_INDEX.discovery.md:L718): N-8: `+governance_coverage_measured` with five Kinds, governance tile
- **SUMMARY/OBS-009** [E] anchor=`14-N:L719` (14-N-SUMMARY_INDEX.discovery.md:L719): N-9: `+counterfactual_scored` with target Kind
- **SUMMARY/OBS-010** [E] anchor=`14-N:L720` (14-N-SUMMARY_INDEX.discovery.md:L720): N-10: joined ambiguity/novelty rows and observable budget delta
- **SUMMARY/OBS-011** [E] anchor=`14-N:L721` (14-N-SUMMARY_INDEX.discovery.md:L721): N-11: byte-identical fold output and retirement-switch effect; no named SSB row is given
- **SUMMARY/OBS-012** [E] anchor=`14-N:L722` (14-N-SUMMARY_INDEX.discovery.md:L722): N-12: one canonical row/type per sub-slice; no universal row name is given
- **SUMMARY/OBS-013** [E] anchor=`14-N:L723` (14-N-SUMMARY_INDEX.discovery.md:L723): N-13: `+output_boundary_verdict`, resolved enforcement mode, per-type/backend invalid rate
- **SUMMARY/OBS-014** [E] anchor=`14-N:L724` (14-N-SUMMARY_INDEX.discovery.md:L724): N-14: `EditKind`, improvement-event timing, persona shift; no named SSB row is given
- **SUMMARY/OBS-015** [E] anchor=`14-N:L725` (14-N-SUMMARY_INDEX.discovery.md:L725): N-15: `+framing_stability_measured(FramingUnstable)`, zeroed weight, adoption rates
- **SUMMARY/OBS-016** [E] anchor=`14-N:L726` (14-N-SUMMARY_INDEX.discovery.md:L726): N-16: `+evaluator_closure_swept`, `+grounding_ratio_measured BelowFloorBraked`, criticism debt and ritual metrics

#### `Wave/dependency matrix` (16 records)

- **SUMMARY/WAVE-001** [E] anchor=`14-N:L764` (14-N-SUMMARY_INDEX.discovery.md:L764): N-1 Wave 1; dep=none stated; nature=ACTIVATION, keystone/blocking; status=Planned; activates existing pieces and adds bootstrap/fold integration
- **SUMMARY/WAVE-002** [E] anchor=`14-N:L765` (14-N-SUMMARY_INDEX.discovery.md:L765): N-2 Wave 2; dep=N-1; nature=BUILD; status=Planned
- **SUMMARY/WAVE-003** [E] anchor=`14-N:L766` (14-N-SUMMARY_INDEX.discovery.md:L766): N-3 Wave 3; dep=N-1; nature=ACTIVATION; status=Planned generalization of finance-shipped/planned machinery
- **SUMMARY/WAVE-004** [E] anchor=`14-N:L767` (14-N-SUMMARY_INDEX.discovery.md:L767): N-4 Wave 3; dep=none stated; nature=BUILD-on-shipped; status=Planned extension/unification
- **SUMMARY/WAVE-005** [E] anchor=`14-N:L768` (14-N-SUMMARY_INDEX.discovery.md:L768): N-5 Wave 4; dep=N-1; nature=BUILD; status=Planned
- **SUMMARY/WAVE-006** [E] anchor=`14-N:L769` (14-N-SUMMARY_INDEX.discovery.md:L769): N-6 Wave 4; dep=none stated; nature=BUILD; status=Planned
- **SUMMARY/WAVE-007** [E] anchor=`14-N:L770` (14-N-SUMMARY_INDEX.discovery.md:L770): N-7 Wave 5; dep=none stated; ties N-2; nature=BUILD; status=Planned
- **SUMMARY/WAVE-008** [E] anchor=`14-N:L771` (14-N-SUMMARY_INDEX.discovery.md:L771): N-8 Wave 5; dep=none stated; nature=BUILD; status=Planned
- **SUMMARY/WAVE-009** [E] anchor=`14-N:L772` (14-N-SUMMARY_INDEX.discovery.md:L772): N-9 Wave 6; dep=N-1; nature=BUILD; status=Planned
- **SUMMARY/WAVE-010** [E] anchor=`14-N:L773` (14-N-SUMMARY_INDEX.discovery.md:L773): N-10 Wave 6; dep=none stated; nature=ACTIVATION; status=Planned
- **SUMMARY/WAVE-011** [E] anchor=`14-N:L774` (14-N-SUMMARY_INDEX.discovery.md:L774): N-11 Wave 7; dep=none stated; nature=BUILD-on-shipped; status=Planned extension + retirement
- **SUMMARY/WAVE-012** [E] anchor=`14-N:L775` (14-N-SUMMARY_INDEX.discovery.md:L775): N-12 Wave 8; dep=N-2 operationally enforces anti-refork, though not in metadata; nature=DELETE; status=Planned dedupe/retirement bundle
- **SUMMARY/WAVE-013** [E] anchor=`14-N:L776` (14-N-SUMMARY_INDEX.discovery.md:L776): N-13 Wave 9; dep=consumes N-1 and N-2; nature=BUILD/UNIFY; status=Planned external-steal addition
- **SUMMARY/WAVE-014** [E] anchor=`14-N:L777` (14-N-SUMMARY_INDEX.discovery.md:L777): N-14 Wave 9; dep=consumes N-1 and N-5; nature=BUILD/ACTIVATION; status=Planned advisory instruments
- **SUMMARY/WAVE-015** [E] anchor=`14-N:L778` (14-N-SUMMARY_INDEX.discovery.md:L778): N-15 Wave 9; dep=consumes N-1 and N-8; nature=BUILD/ACTIVATION; status=Planned advisory measurement
- **SUMMARY/WAVE-016** [E] anchor=`14-N:L779` (14-N-SUMMARY_INDEX.discovery.md:L779): N-16 Wave 9; dep=extends N-8; consumes N-1; nature=EXTENSION(N-8)+BUILD; status=Planned DeepReason addition

#### `Reuse/dependency inventory by unit` (16 records)

- **SUMMARY/DEP-001** [E] anchor=`14-N:L785` (14-N-SUMMARY_INDEX.discovery.md:L785): N-1 consumes G-6, J-34, K-15, J-107, L-10, K-18, I-38, I-27, L-20, J-110, J-56, E-54, J-93-F, W-5/I-51, B-42, J-84/J-85, L-4/L-7, K-24/M-5, and optionally G-5.
- **SUMMARY/DEP-002** [E] anchor=`14-N:L786` (14-N-SUMMARY_INDEX.discovery.md:L786): N-2 consumes/extends M-1, ND-1/ND-3, J-89 L8, J-93-G, I-39, `elai_map_freshness.py`, `wiring_contract_check.py`, paper-to-plan phase 5, R1, and the feature map.
- **SUMMARY/DEP-003** [E] anchor=`14-N:L787` (14-N-SUMMARY_INDEX.discovery.md:L787): N-3 consumes H-17/H-18/H-19, H-47, I-30/I-31, ID-6, N-1, J-56/L-20/D-6, 3F, J-57, J-97, B-23/H-45, I-4/I-25, and L-11.
- **SUMMARY/DEP-004** [E] anchor=`14-N:L788` (14-N-SUMMARY_INDEX.discovery.md:L788): N-4 consumes L-1/L-2..L-6/L-8, J-83/J-96/J-21, K-12/K-18, GRC tags, C-30, and F-5.
- **SUMMARY/DEP-005** [E] anchor=`14-N:L789` (14-N-SUMMARY_INDEX.discovery.md:L789): N-5 arbitrates D-10, J-84, J-55, H-7, E-20, J-48, J-54, L-6, J-32, J-73, plus B-41/L-21/C-46/K-5/B-28; it consumes J-92, K-4, R8, W-7, and R10; E-39 is named in a conflict.
- **SUMMARY/DEP-006** [E] anchor=`14-N:L790` (14-N-SUMMARY_INDEX.discovery.md:L790): N-6 maps G-1, C-48, F-6, C-30, K-23, 4D, J-93-C, and H-53; it families F-1, J-102, J-109, K-23/M-2.
- **SUMMARY/DEP-007** [E] anchor=`14-N:L791` (14-N-SUMMARY_INDEX.discovery.md:L791): N-7 consumes K-11, G-5, K-7, M-3, SSB verdict rows, J-83, L-7, N-1, WIRING_CONTRACT, and N-2.
- **SUMMARY/DEP-008** [E] anchor=`14-N:L792` (14-N-SUMMARY_INDEX.discovery.md:L792): N-8 consumes B-17, K-22, J-46, J-70, H-27, gate-manifest/R8, G-5, M-3, and K-1 dossier framing.
- **SUMMARY/DEP-009** [E] anchor=`14-N:L793` (14-N-SUMMARY_INDEX.discovery.md:L793): N-9 adapts C-45, I-28, J-105, J-109, E-6, E-10, H-40, J-75 and consumes N-1/ND-5.
- **SUMMARY/DEP-010** [E] anchor=`14-N:L794` (14-N-SUMMARY_INDEX.discovery.md:L794): N-10 wires B-38, B-40, C-29, J-32, D-15, 4F/B-21 into 4D, J-84, J-39, J-79.
- **SUMMARY/DEP-011** [E] anchor=`14-N:L795` (14-N-SUMMARY_INDEX.discovery.md:L795): N-11 extends L-16 and consumes J-31/J-60/J-98/K-23, I-39, K-24, M-3, K-22, J-68, H-49, ND-2, and GD-2.
- **SUMMARY/DEP-012** [E] anchor=`14-N:L796` (14-N-SUMMARY_INDEX.discovery.md:L796): N-12 consumes all units explicitly named in its seven sub-slices and depends on N-2 for ongoing anti-fork enforcement.
- **SUMMARY/DEP-013** [E] anchor=`14-N:L797` (14-N-SUMMARY_INDEX.discovery.md:L797): N-13 consumes C-26, E-35, I-34, J-53, J-51, H-13, B-24, L-1, I-33, B-37/I-51, N-1/N-2, D-16, I-46, GD-1, and TD-3.
- **SUMMARY/DEP-014** [E] anchor=`14-N:L798` (14-N-SUMMARY_INDEX.discovery.md:L798): N-14 consumes C-10/B-22b, I-4/I-22/J-10, I-27, H-45/H-47/D-6/H-19, N-5/H-7/L-21/J-84, N-1, tree-sitter, TD-3, and ID-5.
- **SUMMARY/DEP-015** [E] anchor=`14-N:L799` (14-N-SUMMARY_INDEX.discovery.md:L799): N-15 consumes TD-3, I-17, I-33/I-24, H-31, D-14/D-23, E-4/E-54, B-41, J-45, L-11, I-41, N-1/N-8, ID-5, and GD-4.
- **SUMMARY/DEP-016** [E] anchor=`14-N:L800` (14-N-SUMMARY_INDEX.discovery.md:L800): N-16 extends N-8 and consumes F-5, H-27, I-24, N-15, gate-manifest, `gate_freeze_check.py`, R8, N-1, D-17, R4, ND-1/ND-4, ID-5, and TD-3.

---

## User-facing surfaces

Every window, panel, command, CLI verb, flag, status display, and notification atom of the family. Reference atoms by ID; feeds dashboard DESIGN.md. Full text lives in ledger sections above.

Surface-tagged atom count: **131**.

### Surfaces — CLI and authoring commands (4)

- **SUMMARY/CLI-001** anchor=`14-N:L673`: `elai bench <any-family>` / `elai bench *` invokes N-1 families.
- **SUMMARY/CLI-002** anchor=`14-N:L674`: `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` invokes N-2.
- **SUMMARY/CLI-003** anchor=`14-N:L675`: `elai bench sycophancy` invokes N-15 fixtures.
- **SUMMARY/CLI-004** anchor=`14-N:L676`: No public network API, protocol, authentication flow, or remote service API is specified in this summary.

### Surfaces — Global constraints and invariants (1)

- **SUMMARY/INV-010** anchor=`14-N:L663`: **Human visibility:** every verdict family must declare a render, with N-7 frontend pairing mandatory.

### Surfaces — IX (5)

- **N-001/IX-009** anchor=`DEEP-N-001:L393`: Production reachability invariant R7 is repeated per unit: CLI bench (N-1), real SHADOW->ACTIVE (N-3), live J-83 gate (N-4), live ablation (N-9), live fold re-derive (N-11), multi-call-site survivi...
- **N-001/IX-010** anchor=`DEEP-N-001:L394`: User-facing surfaces in this chunk: MeasurementPanel or G-5 (N-1); deflation column on promotion drawers (N-3); memo-hit on belief/claim drawer (N-4); attribution drawers reuse (N-9); canvas altitu...
- **N-002/IX-004** anchor=`DEEP-N-002:L425`: N-8 is the five-view governance substrate; N-16 EXTENDS it with S4/S6/S7 views rather than forking a new framework; N-15 feeds framing-stability into N-8's governance number; N-2 is named as a sixt...
- **N-002/IX-011** anchor=`DEEP-N-002:L432`: Authoring-time surfaces: N-2 CLI composition check + N-8/N-16 governance-coverage checker; runtime/bench surfaces: N-15 `elai bench sycophancy`, N-5 task_runner arbitration, N-13 emission seams, N-...
- **N-002/IX-012** anchor=`DEEP-N-002:L433`: UI composition on shared tiles: N-8 governance tile on G-5/M-3 receives N-16 lambda gauge / quarantine count / ritual-entropy band and N-15 framing-stability input; N-5/N-6/N-7/N-13 add canvas/RRA/...

### Surfaces — Inferred interactions (not promoted to requirements) (1)

- **SUMMARY/N-I05** anchor=`14-N:L820`: N-11’s shared journal folds likely supply N-8/N-16 governance tiles and other N observability, but only the explicitly named five surface folds are requirements.

### Surfaces — Journal derivation and retirement (1)

- **SUMMARY/WF6-003** anchor=`14-N:L647`: Canvas altitude, plan status, billing balance, fleet rollup, and coverage derive through it.

### Surfaces — Measurement-to-decision loop (1)

- **SUMMARY/WF1-002** anchor=`14-N:L590`: A family uses the one K-15/I-27-shaped runner via `elai bench *`.

### Surfaces — N-0 — N-Plan Index (doctrine, ND-deltas, session table, schema, sequencing) (7)

- **N-001/N0-E015** anchor=`N-0:L70-L71`: W-3: reachable from a REAL production entrypoint (CLI / FSM transition / Pre/PostToolUse hook / Tauri command); a `#[cfg(test)]`-only caller is wire-dark.
- **N-001/N0-E016** anchor=`N-0:L72-L72`: W-4: frontend_pair mandatory for any dashboard-surfaceable SSB signal; `cross_lang_diff.py` PASS.
- **N-001/N0-E018** anchor=`N-0:L74-L74`: W-6: >=1 ACCEPTANCE item verified THROUGH the dashboard (dashboard-smoker), or N/A with one-line justification.
- **N-001/N0-E033** anchor=`N-0:L118-L122`: DONE requires all eight: (1) cargo check --workspace GREEN, (2) auditor PASS (sole test runner), (3) wiring_contract_check.py --run-trigger PASS, (4) cross_lang_diff PASS if frontend_pair present, ...
- **N-001/N0-E058** anchor=`N-0:L274-L274`: N-7 adds `+verdict_rendered` Pattern-2 Kind PrComment / ChatAck / CanvasBadge / BriefRow / OrgRollup; baseline+1.
- **N-001/N0-E089** anchor=`N-0:L360-L364`: Session log empty at generation; each shipped N-N adds row: context-compiler -> integration-explorer -> spec-grounder -> executor -> auditor -> fixer (if BLOCKs) -> wiring-contract-validator (RUN_T...
- **N-001/N0-E095** anchor=`N-0:L175-L178`: N-7 render targets: PR comment / chat ACK line / canvas badge / brief row / M-3 org rollup; ONE render substrate, many verdict producers (generalizes K-11).

### Surfaces — N-1 — measurement spine as ONE subsystem (4)

- **N-001/N1-E019** anchor=`N-1:L24-L24, L38-L39, L47-L47`: CLI: `elai bench *` seams as the real production trigger.
- **N-001/N1-E020** anchor=`N-1:L25-L25, L45-L45`: frontend_pair: MeasurementPanel tile fed by the spine index, OR N/A justified if G-5 cockpit already surfaces the profile (confirm at authoring).
- **N-001/N1-E025** anchor=`N-1:L38-L40`: reachable_from: CLI `elai bench <family>` -> runner -> corpus row + profile curve write; `#[cfg(test)]` NEVER satisfies R7.
- **N-001/N1-A003** anchor=`N-1:L25-L25, L45-L45`: MeasurementPanel vs G-5 N/A is deferred to authoring confirmation without a decision criterion.

### Surfaces — N-1 — measurement spine as ONE subsystem [KEYSTONE][BLOCKING] (3)

- **SUMMARY/N1-E17** anchor=`14-N:L65`: Expose the family through the existing CLI seam `elai bench <any-family>` / `elai bench *` (`L37-L40`).
- **SUMMARY/N1-E22** anchor=`14-N:L70`: Provide a `MeasurementPanel` tile sourced from the spine index, unless G-5 already surfaces it and the N/A decision is justified (`L38-L39`).
- **SUMMARY/N1-A04** anchor=`14-N:L78`: Whether the `MeasurementPanel` is mandatory depends on G-5 coverage (`L38-L39`).

### Surfaces — N-10 — clarity / novelty -> budget wires (4)

- **N-002/N10-E006** anchor=`N-10:L11-L11`: High-ambiguity OR off-map task -> deeper research + more clarify + smaller node bound + earlier escalation, not just a chip. Ambiguity = "we don't understand the ask"; novelty = "the ask is off our...
- **N-002/N10-E013** anchor=`N-10:L26-L26, L42-L42`: frontend_pair: N/A (internal resource allocation), unless the depth choice is surfaced on the canvas; justified.
- **SUMMARY/N10-E11** anchor=`14-N:L314`: High ambiguity or an off-map task increases clarification behavior, rather than merely showing a chip (`L202`).
- **SUMMARY/N10-E14** anchor=`14-N:L317`: The default frontend disposition is N/A because this is internal allocation; canvas may surface depth if desired (`L205-L206`).

### Surfaces — N-11 — journal-fold substrate (11)

- **N-001/N11-E004** anchor=`N-11:L8-L8, L14-L14`: Five surfaces: canvas altitudes (J-31/J-60/J-98/K-23), I-39 plan status, K-24 billing balance, M-3 fleet rollup, K-22 coverage.
- **N-001/N11-E011** anchor=`N-11:L14-L14`: Canvas altitudes "re-derive byte-identically from the journal"; I-39/K-24/M-3/K-22 each recomputable from journals.
- **N-001/N11-E014** anchor=`N-11:L19-L19`: Tier-1 SHIP NOW: fold core adopted by 2-3 of five surfaces (canvas + I-39 status) + J-68 Kani proof on core + recover_stubs/H-49 retirement map (user-confirmed).
- **N-001/N11-E015** anchor=`N-11:L20-L20`: Tier-2 DEFER: K-24 balance + M-3 rollup + K-22 coverage adopting the fold (land with those units).
- **N-001/N11-E017** anchor=`N-11:L24-L24, L41-L41`: frontend_pair: canvas altitudes already consume folds — verify byte-identity, do not re-wire.
- **N-001/N11-I001** anchor=`N-11:L8-L8, L14-L14`: "Canvas altitudes" is one surface fed by four unit IDs (J-31/J-60/J-98/K-23), counting as one of the five surfaces rather than four separate fold targets.
- **N-001/N11-A003** anchor=`N-11:L19-L19`: "2-3 of the five surfaces" names canvas + I-39 but leaves the optional third Tier-1 surface unnamed.
- **SUMMARY/N11-E03** anchor=`14-N:L332`: Move J-31/J-60/J-98/K-23 canvas-altitude derivations through the shared fold core (`L213-L220`).
- **SUMMARY/N11-E06** anchor=`14-N:L335`: Move M-3 fleet-rollup derivation through the shared fold core (`L214-L220`).
- **SUMMARY/N11-E13** anchor=`14-N:L342`: Verify existing canvas altitudes consume the folds; do not blindly rewire them (`L220-L221`).
- **SUMMARY/N11-A01** anchor=`14-N:L349`: The summary calls these “five OTHER surfaces”; canvas altitude is one surface with four contributing unit IDs, which should be confirmed in detail.

### Surfaces — N-12 — consistency + dedupe bundle (2)

- **N-001/N12-E020** anchor=`N-12:L27-L27, L45-L45`: frontend_pair: N/A for most; a FailureClass drawer if surfaced; justified per sub-slice.
- **SUMMARY/N12-E06** anchor=`14-N:L363`: Default frontend disposition is N/A per sub-slice, except a surfaced `FailureClass` may have a drawer; each N/A needs justification (`L244-L245`).

### Surfaces — N-13 — OutputBoundary + structured-output capability matrix (5)

- **N-002/N13-E003** anchor=`N-13:L5-L5, L16-L16, L40-L40`: Unified machines (parameterized, never re-implemented): C-26 DecodingMode + per-role defaults + retries; E-35 XGrammar + INSTRUCTOR; I-34 CRANE + grammar pinning + post-parse re-validation + D-16 r...
- **N-002/N13-E008** anchor=`N-13:L13-L13`: Registry keyed by machine-actionable output TYPE: ToolDirective, ClaimSet, PlanSkeleton/BabyStepSpec intake, AuditFinding, MemoryCandidate, canvas ops, VLM verdicts, decision cards.
- **N-002/N13-E022** anchor=`N-13:L53-L54`: frontend_pair: a boundary-verdict badge on existing claim/decision/tool drawers reading `+output_boundary_verdict` (`cross_lang_diff` PASS), OR N/A justified if G-5 cockpit already surfaces the inv...
- **SUMMARY/N13-E09** anchor=`14-N:L443`: Parameterize, rather than reimplement, C-26 `DecodingMode`, E-35 XGrammar/INSTRUCTOR, I-34 CRANE/grammar pin/post-parse revalidation, J-53 tool-input repair, J-51 adherence floor, H-13 canvas gramm...
- **SUMMARY/N13-E20** anchor=`14-N:L454`: Add a boundary-verdict badge to claim/decision/tool drawers, or justify N/A if G-5 surfaces the tile (`L275-L276`).

### Surfaces — N-14 — self-improvement edit-loop instruments (1)

- **N-002/N14-E020** anchor=`N-14:L62-L63`: frontend_pair: an edit-kind chip on existing anti-spiral / search-receipt drawer, OR N/A justified (internal advisory) if no surface consumes it yet.

### Surfaces — N-14 — self-improvement edit-loop instruments (EvoPolicyGym) (1)

- **SUMMARY/N14-E20** anchor=`14-N:L489`: Add an edit-kind chip to the anti-spiral/receipt drawer, or justify internal-advisory N/A (`L297-L298`).

### Surfaces — N-15 — anti-affirmation stability measurement (14)

- **N-002/N15-E001** anchor=`N-15:L1-L3`: N-15 STATUS PLANNED; WAVE 5-adjacent; Tier-3; nature BUILD/ACTIVATION — feeds D-23 sycophancy bench + N-1 spine; residue structural sycophancy defenses do not cover.
- **N-002/N15-E002** anchor=`N-15:L5-L5`: (1) framing/order-invariance extended from decision-type calls (I-33) to ADVISORY panels: present shuffled option/section order across N advisory samples; flip under presentation-only permutation =...
- **N-002/N15-E007** anchor=`N-15:L13-L13`: Two-halves honesty (GD-5): inside a black-box model sycophancy cannot be annulled from the harness; at system boundary ELAI achieves structural irrelevance (TD-3; I-17 L1 hard-veto; K-16 RR-2; I-33...
- **N-002/N15-E008** anchor=`N-15:L16-L18`: Decision-grain framing defense I-33 covers DECISION-type calls, not advisory panels. I-24 pattern is applied to framing instead of style. Uncovered advisory lanes: B-41, J-45, L-11, I-41.
- **N-002/N15-E009** anchor=`N-15:L28-L28, L52-L52`: Advisory framing-invariance check (~70 LOC): when advisory panel emits N samples, shuffle option/section order; if VERDICT flips, log framing-unstable and ZERO its weight for the run. GD-4: measure...
- **N-002/N15-E011** anchor=`N-15:L30-L30`: Route framing-instability rate + two fixture detection numbers through N-1 spine (ND-4); surface framing-stability number as N-8 governance-coverage view input ("advisories are framing-stable at ra...
- **N-002/N15-E015** anchor=`N-15:L44-L47`: reachable_from: advisory-panel emission (B-41 / J-45) [PRODUCTION]; D-23 `elai bench sycophancy` runner seam [PRODUCTION]; tests NEVER satisfy R7.
- **N-002/N15-E016** anchor=`N-15:L47-L47, L58-L58`: CLI/bench surface: `elai bench sycophancy` with the two new fixture classes.
- **N-002/N15-E018** anchor=`N-15:L54-L55`: frontend_pair: framing-stability tile on G-5 cockpit / N-8 governance view reading `+framing_stability_measured`, OR N/A if sycophancy-bench tile already surfaces it.
- **N-002/N15-E019** anchor=`N-15:L57-L58`: trigger: run advisory panel on presentation-order-sensitive input, seeded; then `elai bench sycophancy` with two fixture classes.
- **N-002/N15-A003** anchor=`N-15:L102-L102`: Whether framing-instability should also emit a human-render note (N-7); default zero-weight + spine number, add render only if N-7 has shipped.
- **SUMMARY/N15-E01** anchor=`14-N:L508`: Limit the new mechanism to advisory prose, because I-17 L1 veto, I-33 externalized candidates, H-31 judge firewall, D-14 inverted framing, E-4 chat drift, and TD-3 already make sycophancy structura...
- **SUMMARY/N15-E17** anchor=`14-N:L524`: Surface a framing-stability tile on G-5/N-8, or justify N/A if an existing sycophancy-bench tile provides it (`L319-L320`).
- **SUMMARY/N15-E18** anchor=`14-N:L525`: `elai bench sycophancy` runs the two fixture classes (`L321-L323`).

### Surfaces — N-16 — governance closure (evaluator-refutation, lambda floor, adjudicator-ritual) (3)

- **N-002/N16-E009** anchor=`N-16:L12-L12, L35-L35`: lambda below a floor forces retrieval/research/escalation (a BRAKE), and is NEVER a pass signal; detects STALLED/closed-loop dynamics, not wrong-but-stable consensus. One SSB-derived tile + one thr...
- **N-002/N16-E013** anchor=`N-16:L24-L25`: SHIP NOW (Tier-1, in-process): evaluator-id+version-hash field + S4 quarantine cascade; S6 lambda tile + floor + brake (advisory); S7 finding-class entropy + VERIFY-leg-ran rate. All observe-only (...
- **N-002/N16-E016** anchor=`N-16:L30-L30, L47-L47`: frontend_pair: three numbers on the N-8 / G-5 / M-3 governance tile — lambda gauge + quarantine-queue count + ritual-entropy band.

### Surfaces — N-16 — governance closure: evaluator-refutation verdict-collapse + a lambda exogenous-grounding floor + adjudicator-ritual metrics (1)

- **SUMMARY/N16-E30** anchor=`14-N:L569`: Add a lambda gauge, quarantine-queue count, and ritual-entropy band to the N-8/G-5/M-3 governance tile (`L348-L349`).

### Surfaces — N-2 — corpus-grain composition validator + M-1 one-registry rule (7)

- **N-002/N2-E004** anchor=`N-2:L8-L8, L26-L26, L30-L32`: Build `corpus_composition_check.py` as sibling of `wiring_contract_check.py`, parsing feature-map edges as DATA in ND-3 row format and joining them against plan files + `elai.map` symbols.
- **N-002/N2-E021** anchor=`N-2:L27-L27, L44-L44`: frontend_pair: N/A — authoring-time CI gate, not a runtime surface; justified.
- **N-002/N2-E022** anchor=`N-2:L30-L32`: produces_symbols: `.claude/tools/corpus_composition_check.py` (three FAIL checks + unlock-DAG + freshness) and the ND-3 edge-row format.
- **N-002/N2-E026** anchor=`N-2:L46-L46`: CLI/command surface: `python .claude/tools/corpus_composition_check.py --map docs/thefinalglance/ELAI-FEATURE-MAP.md`.
- **SUMMARY/N2-E15** anchor=`14-N:L98`: Add `.claude/tools/corpus_composition_check.py` beside `wiring_contract_check.py` (`L60-L61`).
- **SUMMARY/N2-E16** anchor=`14-N:L99`: Invoke it as `python .claude/tools/corpus_composition_check.py --map ELAI-FEATURE-MAP.md` (`L63-L64`).
- **SUMMARY/N2-E21** anchor=`14-N:L104`: No runtime frontend is required because this is an authoring-time CI surface (`L60-L62`).

### Surfaces — N-3 — generalized anti-selection-bias promotion primitives (2)

- **N-001/N3-E017** anchor=`N-3:L27-L27, L44-L44`: frontend_pair: a deflation column on the existing promotion drawers, OR N/A justified.
- **SUMMARY/N3-E16** anchor=`14-N:L131`: Add a deflation column to existing promotion drawers, or explicitly justify N/A (`L79-L80`).

### Surfaces — N-4 — one Claim/Belief object + receipt memo (2)

- **N-001/N4-E021** anchor=`N-4:L25-L25, L42-L42`: frontend_pair: existing belief/claim drawer gains a memo-hit indicator, OR N/A justified.
- **SUMMARY/N4-E15** anchor=`14-N:L158`: Add a memo-hit indicator to existing belief/claim drawers, or justify N/A (`L99-L100`).

### Surfaces — N-5 — policy / intensity arbitration plane (2)

- **N-002/N5-E017** anchor=`N-5:L25-L25, L42-L42`: frontend_pair: a RouteDecision explain-row on the canvas/RRA (W-7/N-7 render consumer).
- **SUMMARY/N5-E15** anchor=`14-N:L187`: Render a `RouteDecision` explanation row on the canvas/RRA (`L119-L120`).

### Surfaces — N-6 — provenance / trust lattice + admission-gate family (2)

- **N-002/N6-E014** anchor=`N-6:L24-L24, L41-L41`: frontend_pair: a provenance badge on cited/memory surfaces (W-7/N-7 render consumer), OR N/A justified.
- **SUMMARY/N6-E16** anchor=`14-N:L216`: Add a provenance badge on cited and memory surfaces, or justify N/A (`L138-L139`).

### Surfaces — N-7 — human render as a REQUIRED verdict field (24)

- **N-002/N7-E004** anchor=`N-7:L8-L8, L27-L27`: Every verdict class declares its K-11-style render target: PR comment / chat ACK line / canvas badge / brief row / M-3 org rollup.
- **N-002/N7-E008** anchor=`N-7:L10-L10, L36-L38`: EXCLUDED: re-authoring K-11 per surface; a new dashboard framework (compose existing Svelte dashboard-v2 + K-7 brief + M-3 rollup); inventing a verdict (N-7 renders existing verdicts).
- **N-002/N7-E009** anchor=`N-7:L12-L13`: PLAN-asserted: K-11 PR-evidence render, K-7 buyer brief, G-5 reviewer cockpit, M-3 fleet rollup, canvas altitudes exist; KD-2 + K-20 wedge concede distribution gap.
- **N-002/N7-E010** anchor=`N-7:L14-L15`: Verdicts emit as SSB rows + RRA bundles today; only K-11 humanizes them, PR-only. dashboard-v2 (Svelte 5 + Tauri v2) is the frontend; drawers are the current render surface.
- **N-002/N7-E012** anchor=`N-7:L19-L19`: DEFER (Tier-2): M-3 org-rollup render + full canvas-badge coverage (land as org/canvas units mature).
- **N-002/N7-E013** anchor=`N-7:L21-L22`: Integration: `crates/orchestrator` verdict emission + render substrate; dashboard-v2 + K-11/K-7 as render targets.
- **N-002/N7-E014** anchor=`N-7:L23-L23, L39-L39`: frontend_pair: MANDATORY — this IS the frontend unit; `cross_lang_diff` over the Tauri command + Svelte render targets must PASS.
- **N-002/N7-E015** anchor=`N-7:L26-L28`: produces_symbols: render substrate (dispatcher + target adapters PR/chat/canvas/brief/rollup) + `render_field` as required WIRING_CONTRACT field enforced by N-2.
- **N-002/N7-E016** anchor=`N-7:L32-L34`: reachable_from: live verdict-emitting gate -> render at declared target [PRODUCTION]; dashboard/Tauri render path [PRODUCTION frontend].
- **N-002/N7-E018** anchor=`ChatAck\`: expect_ssb: `+verdict_rendered(PrComment\
- **N-002/N7-E019** anchor=`N-7:L43-L43`: expect_artifact: human-readable render lands at declared target (visible in dashboard-smoker); NEGATIVE: new verdict class with no declared render fails N-2 authoring check.
- **N-002/N7-E020** anchor=`N-7:L45-L47`: Grounding: W-7, TIER-2 item 7, KD-2/K-20; dashboardRules.md + visual-ui-loop.md; K-11 as existing render template.
- **N-002/N7-E021** anchor=`N-7:L49-L50`: COMPOSES-WITH K-11, K-7, G-5, M-3, dashboard-v2, the verdict producers; re-implements NONE (GD-1).
- **N-002/N7-A001** anchor=`N-7:L9-L9, L18-L18`: Mapping from each existing verdict class to one or multiple render targets is not enumerated beyond the Tier-1 2-3 family list.
- **N-002/N7-A003** anchor=`N-7:L19-L19`: Full canvas-badge coverage and M-3 rollup render timing relative to org/canvas unit maturity is open.
- **SUMMARY/N7-E02** anchor=`14-N:L231`: Require each verdict class to declare its render target/form (`L150-L152`).
- **SUMMARY/N7-E03** anchor=`14-N:L232`: Support PR comments as render targets (`L151-L152`).
- **SUMMARY/N7-E04** anchor=`14-N:L233`: Support chat acknowledgment lines as render targets (`L151-L152`).
- **SUMMARY/N7-E05** anchor=`14-N:L234`: Support canvas badges as render targets (`L151-L152`).
- **SUMMARY/N7-E06** anchor=`14-N:L235`: Support brief rows as render targets (`L151-L152`).
- **SUMMARY/N7-E07** anchor=`14-N:L236`: Support M-3 organization rollups as render targets (`L151-L152`).
- **SUMMARY/N7-E12** anchor=`14-N:L241`: The frontend pairing is mandatory because rendering is the feature itself (`L155-L156`).
- **SUMMARY/N7-E13** anchor=`14-N:L242`: Every verdict-emitting gate must deliver a human-readable render to its declared PR/chat/canvas/brief target (`L157-L158`).
- **SUMMARY/N7-A01** anchor=`14-N:L247`: The mapping from each existing verdict class to exactly one or multiple render targets is not enumerated.

### Surfaces — N-8 — governance-coverage substrate (3)

- **N-002/N8-E017** anchor=`N-8:L30-L30, L46-L46`: frontend_pair: a governance tile (the marketable number) on G-5/M-3.
- **N-002/N8-E021** anchor=`N-8:L48-L48`: CLI/command surface: `python <the governance-coverage checker> over the rulebook + gate-manifest`.
- **SUMMARY/N8-E12** anchor=`14-N:L265`: Surface a governance tile on G-5/M-3 (`L174-L175`).

### Surfaces — N-9 — one counterfactual / ablation engine (2)

- **N-001/N9-E013** anchor=`N-9:L25-L25, L41-L41`: frontend_pair: reuse existing attribution drawers, OR N/A justified.
- **SUMMARY/N9-E14** anchor=`14-N:L291`: Reuse existing attribution drawers or justify no frontend (`L190-L191`).

### Surfaces — Named observability artifacts (3)

- **SUMMARY/OBS-006** anchor=`14-N:L716`: N-6: carried trust label and optional provenance badge; no named SSB row is given
- **SUMMARY/OBS-007** anchor=`14-N:L717`: N-7: human render at PR/chat/canvas/brief/rollup target
- **SUMMARY/OBS-008** anchor=`14-N:L718`: N-8: `+governance_coverage_measured` with five Kinds, governance tile

### Surfaces — UI surfaces (15)

- **SUMMARY/UI-001** anchor=`14-N:L742`: N-1 `MeasurementPanel` or justified reuse of G-5.
- **SUMMARY/UI-002** anchor=`14-N:L743`: N-3 deflation column in promotion drawers or justified N/A.
- **SUMMARY/UI-003** anchor=`14-N:L744`: N-4 memo-hit indicator in belief/claim drawers or justified N/A.
- **SUMMARY/UI-004** anchor=`14-N:L745`: N-5 canvas/RRA route-explanation row.
- **SUMMARY/UI-005** anchor=`14-N:L746`: N-6 provenance badge on citation/memory views or justified N/A.
- **SUMMARY/UI-006** anchor=`14-N:L747`: N-7 mandatory PR/chat/canvas/brief/rollup render substrate.
- **SUMMARY/UI-007** anchor=`14-N:L748`: N-8 governance tile on G-5/M-3.
- **SUMMARY/UI-008** anchor=`14-N:L749`: N-9 existing attribution drawers or justified N/A.
- **SUMMARY/UI-009** anchor=`14-N:L750`: N-10 normally internal; optionally show chosen depth on canvas.
- **SUMMARY/UI-010** anchor=`14-N:L751`: N-11 verify existing canvas fold consumption.
- **SUMMARY/UI-011** anchor=`14-N:L752`: N-12 normally N/A; optional `FailureClass` drawer.
- **SUMMARY/UI-012** anchor=`14-N:L753`: N-13 boundary-verdict badge or G-5 tile reuse.
- **SUMMARY/UI-013** anchor=`14-N:L754`: N-14 edit-kind chip or justified internal N/A.
- **SUMMARY/UI-014** anchor=`14-N:L755`: N-15 G-5/N-8 framing-stability tile or existing sycophancy tile reuse.
- **SUMMARY/UI-015** anchor=`14-N:L756`: N-16 lambda gauge, quarantine count, and ritual-entropy band on governance tile.

### Surface catalog (curated categories for DESIGN.md)

CLI / authoring commands:
- `elai bench <family>` / `elai bench *` — N-001/N1-E019, SUMMARY/N1-E17, SUMMARY/CLI-*
- `python .claude/tools/corpus_composition_check.py --map docs/thefinalglance/ELAI-FEATURE-MAP.md` — N-002/N2-E026, SUMMARY/N2-E16
- `elai bench sycophancy` — N-002/N15-E016, SUMMARY/N15-E18
- governance-coverage checker over rulebook + gate-manifest — N-002/N8-E021, N-002/N16-E015

Kill-switch / legacy flags:
- `ELAI_*_LEGACY` one-release policy — N-001/N0-E014, N-001/N0-A002
- `ELAI_OUTPUT_BOUNDARY_LEGACY` — N-002/N13-E027
- `ELAI_EDIT_KIND_LEGACY` — N-002/N14-E025
- `ELAI_FRAMING_INVARIANCE_LEGACY` — N-002/N15-E022

Dashboard / canvas / drawers / tiles / badges:
- MeasurementPanel or G-5 — N-001/N1-E020
- deflation column on promotion drawers — N-001/N3-E017
- memo-hit indicator on belief/claim drawer — N-001/N4-E021
- RouteDecision explain-row on canvas/RRA — N-002/N5-E017
- provenance badge on cited/memory surfaces — N-002/N6-E014
- render targets PR comment / chat ACK / canvas badge / brief row / M-3 org rollup — N-002/N7-E004, N-001/N0-E095
- governance tile on G-5/M-3 — N-002/N8-E017
- attribution drawers reuse — N-001/N9-E013
- canvas altitude fold verification — N-001/N11-E017
- optional FailureClass drawer — N-001/N12-E020
- boundary-verdict badge — N-002/N13-E022
- edit-kind chip — N-002/N14-E020
- framing-stability tile — N-002/N15-E018
- lambda gauge + quarantine-queue count + ritual-entropy band — N-002/N16-E016

SSB / status event kinds (status displays / notifications):
- `+calibration_bootstrap_superseded` — N-001/N0-E052, N1-E027
- `+composition_edge_checked` — N-001/N0-E053, N-002/N2-E027
- `+promotion_deflated` — N-001/N0-E054, N3-E022
- `+receipt_memo` — N-001/N0-E055, N4-E026
- `+route_arbitrated` — N-001/N0-E056, N-002/N5-E023
- `+admission_scanned` (conditional) — N-001/N0-E057, N-002/N6-E019
- `+verdict_rendered` — N-001/N0-E058, N-002/N7-E018
- `+governance_coverage_measured` — N-001/N0-E059, N-002/N8-E022
- `+counterfactual_scored` — N-001/N0-E060, N9-E018
- `+output_boundary_verdict` — N-001/N0-E065, N-002/N13-E019
- `+edit_kind_classified` (or reuse) — N-001/N0-E066, N-002/N14-E015
- `+framing_stability_measured` — N-001/N0-E067, N-002/N15-E012
- `+evaluator_closure_swept` + `+grounding_ratio_measured` — N-001/N0-E068, N-002/N16-E020

---

## Closing audit block

- **Ledgers merged:** 2/2 deep (`DEEP-N-001`, `DEEP-N-002`) + summary (`14-N-SUMMARY_INDEX.discovery.md`).
- **Per-ledger atom counts:**
  - N-001: source atoms **290** (N-0=105, N-1=35, N-3=29, N-4=34, N-9=25, N-11=29, N-12=33) + IX **23** + SD **46** + AC **10** = subtotal **369**.
  - N-002: source atoms **297** (N-2=34, N-5=30, N-6=26, N-7=24, N-8=29, N-10=24, N-13=37, N-14=36, N-15=30, N-16=27) + IX **18** + SD **35** + AC **11** = subtotal **361**.
  - SUMMARY: inventory **385** + prose/WF/INV/UI/OA **133** + OBS/WAVE/DEP tables **48** = subtotal **566**.
- **Superseded notes count:** **9** (SUP-001..SUP-009).
- **Aggregate distinct record count:** **1305** (= 369+361+566+9).
- **Surface-tagged atoms indexed:** **131**.
- **Zero-omission statement:** Every atomic table row from DEEP-N-001 and DEEP-N-002 was extracted with its ID and exact source anchor. Every IX/SD/AC row was preserved under ledger namespace. Every ID'd inventory bullet from 14-N-SUMMARY_INDEX.discovery.md was preserved under SUMMARY/. Structured summary sections (program doctrine, workflows, constraints, CLI, UI, observability table, wave matrix, dependency inventory, open ambiguities, rejected items) were captured as SUMMARY/* atoms. Losing target-state descriptions under newer-plan precedence were retained as superseded notes, not deleted. No deep atom was compressed into prose-only form. Relational IX/SD/AC are not double-counted as feature atoms in the source ledgers' own audits; they are distinct records in this aggregate.
- **Write scope:** only this file `docs/thefinalglance/feature-discovery/aggregates/N-AGGREGATE.md`. Inputs read-only. No git operations.

