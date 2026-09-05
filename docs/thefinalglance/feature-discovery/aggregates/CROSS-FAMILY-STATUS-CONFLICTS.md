# Cross-family status conflicts — ELAI deep feature discovery

## Document contract

- **Purpose:** One register of every case where two or more sources or families disagree about a feature’s **status** or **shape** (shipped vs planned, wired vs stub/wire-dark, kept vs vetoed/rejected, differing tier assignments, differing implementations of the same feature).
- **Inputs (read-only):** all 17 family aggregates under `docs/thefinalglance/feature-discovery/aggregates/` — `PHASE`, `B`..`Q`.
- **Precedence rule (NEWER-PLAN PRECEDENCE):**
  1. An explicit `SUPERSEDED` / replacement / `CORRECT` / `REJECTED` / `SHIPPED` measurement / `VETO` / `VALID REFUTED` marker in a source beats everything.
  2. Else later **letter family** wins (B oldest → Q newest).
  3. Within a family, higher plan number is newer.
  4. Deep ledgers supersede summary orientation where they conflict (every family contract states this).
  5. **Losing claims stay recorded** as superseded notes; never deleted.
- **Scope of a conflict record:** status/shape disagreement only. Pure inventory wording deltas that do not change ship/wire/tier/disposition are left in per-family SUP sections (see inventory appendix).
- **Generated:** 2026-08-12 (cross-family status-conflicts pass).

### Notation

| Field | Meaning |
|---|---|
| **CF-NNN** | Cross-family conflict id |
| **Feature** | Named capability / unit under dispute |
| **Claims** | Conflicting readings, each with atom ID + aggregate anchor |
| **RESOLUTION** | Winner under NEWER-PLAN PRECEDENCE, or `UNRESOLVED` |
| **Loser retained** | Where the losing description still lives |

---

## 1. Seed conflicts (verified in aggregates)

These were pre-named as load-bearing seeds. Each was re-checked against the family aggregate before registration.

### CF-001 — Q-14 E-1 novelty canary: SHIPPED vs planned-only

- **Feature:** Q-14 sub-unit E-1 (conjecture novelty canary / fail-closed instrument).
- **Claim A (deep, measurement banner):** E-1 **SHIPPED 2026-07-18** — canary + fail-closed `instrument_strength=FAILED` + `--self-test-novelty-canary` in `conjecture_novelty_canary.py` consumed by `conjecture_triage.py`; adversarial-auditor READY. E-2/E-4/E-3 remain PLANNED.
  - Atoms: `Q-001/Q14-E002`, `Q-001/Q14-E003`, `Q-001/SD-Q14-001`
  - Aggregate: `Q-AGGREGATE.md` SUP-001; `Q-14:L4-L5`; DEEP-Q-001:L278-L279
- **Claim B (summary orientation):** E-1 presented as build-on-shipped / planned inventory only (no SHIPPED banner).
  - Atoms: `SUMMARY/Q14-E1-*`, `SUMMARY/DEP-Q-14`
  - Aggregate: `17-Q:L493-L513, L692`
- **Claim C (same-file header):** whole-file header `STATUS PLANNED` coexists with E-1 SHIPPED banner.
  - Atoms: `Q-001/Q14-E001`, `Q-001/Q14-C001`, `Q-001/AC-001`
  - Aggregate: `Q-14:L3-L5`
- **RESOLUTION:** **RESOLVED — deep SHIPPED wins.** Explicit measurement SHIPPED banner beats summary planned inventory. Per-sub-unit banners control; file header is coarse and must not erase E-1 SHIPPED.
- **Loser retained:** summary planned inventory; header STATUS word (Q-AGGREGATE SUP-001, SUP-005).

### CF-002 — Q-15 S-1 GEPA policy arm: REJECTED-by-measurement vs live selectable

- **Feature:** Q-15 S-1 (GEPA pareto_per_task as third Q-12 selector policy).
- **Claim A (deep measurement):** S-1 **REJECTED-by-measurement 2026-07-17** (B-SYN-2; P2 mean delta 0.1015625 vs locked 0.25 floor; CI clears only CI>0 leg). Closes bench lane; research-lane variant must NOT be authored; do not reopen design to chase a passing number.
  - Atoms: `Q-001/Q15-E001`, `Q-001/Q15-C001`, `Q-001/SD-Q15-001`
  - Aggregate: `Q-AGGREGATE.md` SUP-002; `Q-15:L3`; DEEP-Q-001:L312
- **Claim B (summary orientation):** inventories S-1 as a live selectable policy arm without rejection banner.
  - Atoms: `SUMMARY/Q15-S1-01..Q15-S1-11`, `SUMMARY/DEP-Q-15`
  - Aggregate: `17-Q:L545-L560, L692-L693`
- **RESOLUTION:** **RESOLVED — REJECTED-by-measurement is target ship disposition.** Full S-1 design retained as non-ship history only.
- **Loser retained:** summary live-arm inventory (Q-AGGREGATE SUP-002). Remaining live Q-15 units: S-2, S-3, S-4, S-6 (`Q-001/Q15-E002`).

### CF-003 — K-4 gate-tax design: refined v2 VETO of ON/OFF ablation + wire-dark inventory

- **Feature:** K-4 composed gate-tax budget / measurement design.
- **Claim A (summary orientation):** product ON/OFF SAFEDELTA ablation, gates-OFF denominator, 105-row `wire-dark-allowlist.toml` as gate inventory, single whole-system over-block number, pin `args_bench.rs:75`.
  - Atoms: `SUMMARY/K-4.05`, `SUMMARY/K-4.07`, `SUMMARY/K-4.11`, `SUMMARY/K-4.16`, `SUMMARY/K-4.01`
  - Aggregate: `11-K:L132-L147`
- **Claim B (refined K-4 deep source):** **VETO** product-level gates ON/OFF (violates R8/W-2; LLM wall-clock drowns sub-ms gate deltas). **FIX-1 bans wire-dark-allowlist as scope source** (default-OFF exemptions; 135 rows, not 105). Two labeled numbers (coding_task vs audit) + over_block_rate, never summed. Per-gate isolation measurement. CLI exit-code contract; no line-:75 pin; 4-file K-18a-pattern wiring.
  - Atoms: `K-004/K4-E012`, `K-004/K4-E016`, `K-004/K4-E017`, `K-004/K4-E018`, `K-004/K4-C001`, `K-004/SD-K4-001..004`, `K-004/AC-001`
  - Aggregate: `K-AGGREGATE.md` SUP-012..SUP-019; `K-4:L14-L43, L71-L82, L171-L178`
- **RESOLUTION:** **RESOLVED — refined K-4 VETO/FIX design is target state.** Summary ON/OFF + wire-dark inventory + single-number framing is superseded.
- **Loser retained:** summary K-4.01/05/07/11/16 inventory (K-AGGREGATE SUP-012..019).

### CF-004 — P-family Tier-1 vs Tier-2 corrections (named units)

Each row is deep REVISED scope / SD-CORRECT over summary (or index) peer-requirement flattening. **Family P is newer than B..O**, so deep P wins for P-unit shape. Within P, numbered detail + REVISED scope win over summary inventory.

| Sub-id | Feature | Losing claim (summary / opening / index) | Winning claim (deep REVISED) | Aggregate anchors |
|---|---|---|---|---|
| **CF-004a** | **P-15** verified case-graph | Summary lists narrative/episodic tiers, embedding retrieval, and bBoN as if Tier-1; wargame EXPECTED_OBSERVATION asserts bBoN ranks | Tier-1 = gated retrieve/distill/fuse + reuse-hit + default-off; **bBoN + narrative/episodic + embedding → Tier-2** | P-003/SD-P15-001, P15-C001; P-AGG SUP-028, SUP-035; `16-P:L507-L521`; `P-15:L20, L51` |
| **CF-004b** | **P-20** context/memory lifecycle | Premise/summary list Dreaming promotion gates + per-provider cache-control as peer Tier-1 | Tier-1 = pre-compaction flush + on-demand versioned skill/rules index only; **Dreaming + cache mapping → Tier-2** | P-003/SD-P20-001, P20-C001; P-AGG SUP-029, SUP-036; `16-P:L841-L848`; `P-20:L6, L18-L20` |
| **CF-004c** | **P-22** control-plane/channel hardening | Summary treats nine-step channel-binding, pairing codes, dmScope, signed device pairing, unknown-event withhold, idempotency keys, trusted-policy hooks as peer Tier-1 | Tier-1 = **fenced child returns + depth-scoped tool-strip only**; remaining channel/pairing surface → Tier-2 | P-003/SD-P22-001, P22-C001; P-AGG SUP-031, SUP-037; `16-P:L914-L939`; `P-22:L6-L21, L42-L45` |
| **CF-004d** | **P-12** desktop/GUI operator lane | P-0 index + summary treat OmniParser Tier-3 pixel fallback as P-12 ship package | Tier-1 = cua sandbox + frame-bind + arm/lease + recording; **OmniParser Tier-3 deferred Tier-2 with P-13** | P-001/P12-C001, SD-P12-001; P-AGG SUP-002, SUP-009; `P-0:L42-L44`; `P-12:L20` |
| **CF-004e** | **P-13** perception provenance + SoM | Summary “Add no CLI”; empty-AXTree acceptance expects Tier-3 pixel fire once | Acceptance uses `elai perception scaffold` trigger while PRODUCTION_HOOK extends gateway in place; **Tier-3 pixel/YOLO deferred Tier-2 with P-12** | P-001/P13-C001, P13-C002, SD-P13-001/002; P-AGG SUP-003, SUP-004, SUP-010, SUP-011; `P-13:L20, L46, L51, L71-L72` |
| **CF-004f** | **P-18** durable TaskSpec exec | Index/summary list retry ladders, watchdog, preflight, due-time clamp, resume checkpoint as peer requirements | Tier-1 = revision-counter + sticky-cancel + drift-binding only; **retry/watchdog/due-time clamp → Tier-2/RECON**; same-tick commitment ban still active in stub_ban | P-001/P18-C001, SD-P18-001/003; P-AGG SUP-006, SUP-007, SUP-012; `P-18:L20, L42`; `P-0:L96-L100` |
| **CF-004g** | **P-21** gate-credibility | Summary lists skipBackfill, structural redaction, posture labels as peer Tier-1 | REVISED scope places skipBackfill / structural redaction / posture labels in **Tier-2**, but WIRING_CONTRACT / stub_ban / observable_effect / PRODUCTION_HOOK still require some of those now — **internal inconsistency** (see residual RA-001) | P-001/P21-C001, SD-P21-001; P-AGG SUP-008, SUP-013; `P-21:L18-L19, L29-L30, L37-L41, L47, L70-L71` |

- **RESOLUTION (004a–004f):** **RESOLVED — deep REVISED Tier-1 scope wins.** Summary peer-requirement flattening is superseded note only.
- **RESOLUTION (004g):** partial — Tier-2 *intent* of REVISED scope is the newer design statement, but same-file wiring clauses contradict it → see **RA-001 UNRESOLVED**.

Additional P Tier corrections of the same class (not in the seed shortlist, same rule, recorded for completeness):

| Sub-id | Feature | Winner Tier-1 shape | Loser (summary/opening) | P-AGG SUP |
|---|---|---|---|---|
| CF-004h | P-1 fleet health | Smoke + DepRotCheck + N-7 diagnosis card; apply-button/G-10/drift canary Tier-2 | Opening lists DriftCanary + one-click auto-repair as core | SUP-014, SUP-021, SUP-022 |
| CF-004i | P-2 inter-tool ports | CLI registry/wire/SSB only | O-4 canvas edge + third-party M-5 mount as core → Tier-2 | SUP-015, SUP-023 |
| CF-004j | P-6 privacy lattice | O-3 vault-read only | Dual O-3 AND E-20 enforcement; E-20 → Tier-2 | SUP-016, SUP-024 |
| CF-004k | P-7 operating profile | Manual asymmetry-guarded change | Automatic N-3 learning loop as core → Tier-2 | SUP-017, SUP-025 |
| CF-004l | P-14 explore-then-crystallize | E-14 browser Playwright only | Playwright + PyAutoGUI peer emitters; desktop → Tier-2 | SUP-018, SUP-026 |
| CF-004m | P-19 config/ops lifecycle | Fail-closed config + doctor detect/repair/lint | VACUUM backups, restart continuation, post-upgrade probes, compaction split as peer → Tier-2 | SUP-020, SUP-027 |
| CF-004n | P-3 persona export | Tier-1 without persona regen | Full import+regenerate as Tier-1; regen → Tier-2 | SUP-034 |
| CF-004o | P-8 multi-seat | Two-seat kernel Tier-1 | “2-10 scale” as immediate | SUP-001 |
| CF-004p | P-16 install catalog | “hidden-until-verified” registry release gate → Tier-2 | Summary admission family phrasing | SUP-005 |

### CF-005 — J-80 worktree pool: wargame doc-drift (callers, deps, H-49)

- **Feature:** J-80 worktree pool lifecycle (extension of I-2).
- **Doc-drift #1 (callers):**
  - Claim A: plan body / SUMMARY place three pool callers in `bsp/engine.rs`.
  - Claim B (recon): callers are **actually in `task_runner.rs`** (e.g. `task_runner.rs:5676` pool.spawn in per-node phase loop; PRODUCTION_HOOK refined to `branch_workspace.rs:327` + task_runner).
  - Atoms: `J-010/J80-C001`, `J-010/J80-E014`, `J-010/J80-E015`
  - Aggregate: `J-80:L134`; DEEP-J-010:L397-L379
- **Doc-drift #2 (dependency):**
  - Claim A: ACCEPTANCE cites non-existent `sysinfo` dep.
  - Claim B: design uses `fd-lock` (`Cargo.toml fd-lock="4"`); no new dep.
  - Atoms: `J-010/J80-C002`
  - Aggregate: `J-80:L135`
- **Doc-drift #3 (H-49 status):**
  - Claim A: H-49 flagged PARTIAL.
  - Claim B: CAS/RV-1 legs **SHIPPED** with production caller (`timetravel_cmd.rs`).
  - Atoms: `J-010/J80-C003`, `J-010/SD-J80-004`
  - Aggregate: `J-80:L136`; `10-J:L847-L855`
- **Related status:** I-2 MAP says PLANNED is STALE → apply MAP FIX I-2→**PARTIAL** (`J-010/J80-E003`).
- **RESOLUTION:** **RESOLVED — recon/wargame corrections win target wiring facts.** Callers = task_runner (not bsp/engine prose); dep = fd-lock; H-49 CAS/RV-1 treated SHIPPED for consumer purposes; I-2 → PARTIAL on map.
- **Loser retained:** plan/SUMMARY bsp/engine caller wording; sysinfo acceptance line; H-49 PARTIAL flag as stale map text.

### CF-006 — PHASE-007 dep-graph and payload-type conflicts

#### CF-006a — 1I dep-graph missing 1G→1I edge

- **Feature:** Phase 1I knowledge degradation depends on Phase 1G.
- **Claim A:** implementation-guide dep graph omits 1G→1I.
- **Claim B:** 1I requires adding the missing edge before implement (compile-failure if ignored).
- **Atoms:** `PHASE-007/AC-001`, `P1I-C001`, `P1I-E002`, `PHASE-007/SD-P1I-001`
- **Aggregate:** PHASE-AGGREGATE PHASE-007 AC register; `01-phase:L147`
- **RESOLUTION:** **RESOLVED — ACTION-REQUIRED dep-graph fix is target state** (1I plan correction wins over incomplete graph).

#### CF-006b — 3B SsbPayload “15 variants” vs 17 action_types

- **Feature:** Phase 3B SSB typed payload inventory.
- **Claim A:** prose “15 variants”.
- **Claim B:** 17 action_types in CHECK schema.
- **Atoms:** `PHASE-007/AC-007`, `P3B-C001`, `P3B-A001`, `PHASE-007/SD-P3B-007`
- **Aggregate:** `01-phase:L250`
- **RESOLUTION:** **RESOLVED for counting authority — 17 action_types CHECK schema controls.** “15 variants” wording retained as superseded prose.

#### CF-006c — 4D research_* SSB types absent from 3B 17-type CHECK

- **Feature:** Phase 4D research progress journal types (`research_step`, `research_decompose_level`, `web_research_step`, related).
- **Claim A:** 4D journals research_* action types.
- **Claim B:** those types are **not registered** in Phase 3B’s 17-type CHECK; consumers cannot journal them without schema extension.
- **Atoms:** `PHASE-007/AC-009`, `PHASE-007/IX-017`, `P4D-C001`, `P4D-A001`, `PHASE-007/SD-P4D-006`
- **Aggregate:** PHASE-007 IX/AC; 4D vs 3B anchors in deep ledger
- **RESOLUTION:** **RESOLVED as known schema gap:** target state requires a **schema extension** before 4D research_* rows are journalable; 3B 17-type CHECK remains authoritative until extended. Not a silent “4D wins naming” override.
- **Related:** CF-006d — 2D design methods `web_search`/`grep`/`api_registry` vs SourceMatch-only type until 4F (`PHASE-007/AC-004`, `P2D-C001`) → **RESOLVED:** SourceMatch-only until 4F is binding interim type shape.

### CF-007 — Phase 4J empire_* five SSB variants vs singular `empire_event`

- **Feature:** Empire integration SSB `action_type` granularity.
- **Claim A (PRE-RESOLVED DECISION 3):** five variants — `empire_connected`, `empire_disconnected`, `empire_event_received`, `empire_capability_invoked`, `empire_repair_proposed` (**not** single `empire_event`).
  - Atom: `P4J-E004`
  - Aggregate: `4J:L30-L37`
- **Claim B (task text still stale):** 4J.4 / 4J.8 TASK lines still write `action_type='empire_event'` singular.
  - Atom: `P4J-C001`
  - Aggregate: `4J:L356-L358, L596`
- **Related shape conflict:** DETERMINISTIC_CHECK still says `cargo test -p elai_empire` while DECISION 2 redirects to `elai-orchestrator --features empire` (`P4J-C002`; `4J:L21-L28, L313-L314, L618`).
- **RESOLUTION:** **RESOLVED — PRE-RESOLVED DECISION wins.** Target event set is the five `empire_*` variants. Singular `empire_event` task wording is superseded prose. Package/test target is orchestrator `--features empire`, not `elai_empire`.
- **Loser retained:** TASK singular action_type lines; `elai_empire` test package name.

---

## 2. Additional status/shape conflicts (from aggregate conflict registers)

These are not in the seed shortlist but are true multi-source status or shape disagreements. Precedence applied the same way.

### CF-008 — Q-10 B-CREATIVITY-0 ladder: VALID REFUTED vs open adoption

- **Feature:** Q-10 research-conditioned creativity ladder (B0v3).
- **Claim A:** VERDICT 2026-07-14 **VALID REFUTED**; no B1/skill/runtime/adoption.
  - Atoms: `Q-001/Q0-E010`, `Q-002/Q10-E001`, `SUMMARY/Q10-90..Q10-93`
- **Claim B:** same index page still carries next-step Tier-0 authorization language; any pre-verdict text describing Tier-2/3 adoption as open work.
  - Atoms: `Q-001/Q0-E028`, `Q-001/Q0-I001`, `Q-001/AC-003`
- **RESOLUTION:** **RESOLVED — VALID REFUTED halt is binding.** Next-step Tier-0 authorization is historical superseded note (Q-AGGREGATE SUP-003, SUP-006).

### CF-009 — Q-6 OOM/I-O injection tier

- **Feature:** Q-6 gate-kill mutation set.
- **Claim A (deep):** Tier-1 = file corruption only; OOM/I-O is **Tier-2 deferred**.
  - Atoms: `Q-002/Q6-E005`, `Q6-E010`, `Q6-C001`
- **Claim B (summary):** lists OOM-injection as first-class mutation without Tier-2 qualification.
  - Atom: `SUMMARY/Q6-12` (`17-Q:L189`)
- **RESOLUTION:** **RESOLVED — Tier-1 = file corruption; OOM/I-O Tier-2** (Q-AGGREGATE SUP-004).

### CF-010 — B-31 resumable indexing: DONE vs PLANNED

- **Feature:** B-31 batched indexing / IndexLock / auto-resume.
- **Claim A:** later B-0 reconciliation marks B-31 **DONE** (resumable batched indexing, lock, startup auto-resume, production CLI/dashboard lifecycle, five SSB payloads, wire-dark `ELAI_DISABLE_INDEXING`).
  - Atoms: `B0-025`, B-AGGREGATE SUP-002, SUP-004, SUP-009
- **Claim B:** older plan/summary PLANNED description of intended behavior as open extension work.
- **RESOLUTION:** **RESOLVED — DONE wins** (explicit status reconciliation in B-0).

### CF-011 — B-38 ambiguity-gate: SHIPPED vs PLANNED

- **Feature:** B-38 three ambiguity-gate SSB types (schema 42→43).
- **Claim A:** explicitly **SHIPPED** (status inferred from schema comments/enum variants; no standalone memory block).
  - Atom: `B0-028`; B-AGG SUP-003
- **Claim B:** earlier PLANNED marker.
- **RESOLUTION:** **RESOLVED — SHIPPED wins.**

### CF-012 — B-20 failure categories: twelve vs nine

- **Feature:** B-20 dispatcher failure category set.
- **Claim A (2026-05-23 rebaseline + C-33):** twelve categories canonical; coexistence until B-18B.
  - Atoms: `B20-001`, B-AGG SUP-005, SUP-010
- **Claim B:** original objective one dispatcher and nine failures.
- **RESOLUTION:** **RESOLVED — twelve is canonical.**

### CF-013 — B-39 REFINE_CAP: 3 vs 2

- **Feature:** B-39 chat reflect refine cap.
- **Claim A (later C-33 reconciliation):** implementation shipped `REFINE_CAP=3`; contract asserts 3.
  - Atom: `B39-037`; B-AGG SUP-028
- **Claim B:** most task/test/acceptance text caps refinement at 2.
- **RESOLUTION:** **RESOLVED — 3 is current canonical value.** Cap-2 retained as historical.

### CF-014 — B-12 dashboard shell IA: single Workspace vs Tier-0 five-icon model

- **Feature:** B-12 dashboard primary shell / icon rail.
- **Claim A (V2 rebaseline + July closure):** one canonical Workspace; Tier-0 Shell retired; active three-button 48px IconRail + four-question-tab IA.
  - Atoms: `B12-002`, `B12-007`, `B12-023`, `B12-028`; B-AGG SUP-017..SUP-023
- **Claim B:** five 48px sidebar icons, `Ctrl+Shift+W` Tier-0 toggle, `primary_shell` preference.
- **RESOLUTION:** **RESOLVED — single Workspace / retired Tier-0 is target.** Five-icon and Tier-0 toggle are deferred/superseded.

### CF-015 — D-28 adversarial farm: shipping-pass live vs header DEFERRED

- **Feature:** D-28 adversarial farm host path.
- **Claim A:** shipping-pass — `AdversarialFarmRunCompleted`; local CLI/SSB legs claimed live; training/deploy deferred.
  - D-AGG SUP-004
- **Claim B:** header Status:DEFERRED and two-event design (`AdversarialCorpusGenerated` / `PpmFinetuneCompleted`).
- **RESOLUTION:** **RESOLVED for host legs — shipping-pass replacement beats header DEFERRED** for those legs; training/deploy remain deferred.

### CF-016 — D-29 hierarchical BAI: mixed shipped host vs clean DEFERRED

- **Feature:** D-29 hierarchical BAI.
- **Claim A:** Path A host Rust dispatch claimed shipped; Gibbs/RunPod research closure still DEFERRED.
  - D-AGG SUP-005
- **Claim B:** summary/D-0 clean DEFERRED only.
- **RESOLUTION:** **RESOLVED as mixed status** — host path live-qualified; research closure remains DEFERRED. Not full DONE.

### CF-017 — D-30 calibration bootstrap: partial wired vs blanket DEFERRED

- **Feature:** D-30 calibration bootstrap.
- **Claim A:** Rust dispatch claimed wired/live; corpus/data/D22 legs DEFERRED.
  - D-AGG SUP-006
- **Claim B:** summary blanket DEFERRED.
- **RESOLUTION:** **RESOLVED as partial** — host path annotations beat summary-only DEFERRED.

### CF-018 — F-10 self-mod leg: intentional wire-dark vs all-three-loop-legs done

- **Feature:** F-10 self-mod / evolve loop leg.
- **Claim A (deep as-built):** self-mod leg intentionally **wire-dark** behind darwinian-evolve allowlist until F-11.
  - F-AGG SUP-019
- **Claim B (summary):** acceptance all-three-loop-legs block claim implies full trio live.
- **RESOLUTION:** **RESOLVED — wire-dark until F-11 is target disposition** for self-mod leg.

### CF-019 — G-1 / G-4 / G-5 / G-7 header PLANNED vs “SHIPPED reality” anchors

- **Feature:** G-plan units with dated live corrections while header status stays PLANNED.
- **Claim A:** header `STATUS PLANNED`.
- **Claim B:** later dated corrections name concrete live modules/emitters/schema ordinals as shipped ground truth; files never declare overall DONE.
- **Atoms (examples):** `G-001/DG001-G1-C004`, `G4-C005`, `G5-C007`, `G7-C010`
- **RESOLUTION:** **RESOLVED for reading discipline — treat dated shipped anchors as ground-truth for named seams; do not promote whole unit to DONE without explicit DONE transition.** Header PLANNED retained as initiative-level status. Losing “unit is fully DONE” inference is not authorized.

### CF-020 — G-5 dependency: G-2 hard prereq vs SSB-only path shipped without RRA

- **Feature:** G-5 DSR / review surface dependency on G-2 RRA.
- **Claim A:** header/objective still call G-2 a hard forward dependency.
- **Claim B:** dated correction + done-gate: SSB-only scoring shipped; RRA optional.
- **Atoms:** `G-001/DG001-G5-C006`
- **RESOLUTION:** **RESOLVED — SSB-only path is sufficient for G-5 DONE framing; RRA optional.** Hard G-2 prereq prose superseded for that gate.

### CF-021 — G-2 recover_from_checkpoint stub vs “no production caller” myth

- **Feature:** G-2 BSP / RRA recover path production reachability.
- **Claim A (refuted myth):** BSP has no production caller.
- **Claim B:** `task_runner.rs:5018` is asserted non-test production caller; `recover_from_checkpoint` remains a stub for resume-from-RRA (optional follow-on).
- **Atoms:** `G-002/DG002-G2-011`
- **RESOLUTION:** **RESOLVED — do not revive “no production caller” claim.** Resume-from-RRA stays optional/deferred; base BSP caller exists.

### CF-022 — I-2 / belief / RRA “planned-only” memories vs PARTIAL/SHIPPED facts

- **Feature:** I-plan map status for worktree pool, RRA, E-25 checkpoint, I-11 belief_state.
- **Claim A (stale memory/map):** PLANNED / absent.
- **Claim B (later corrections):** I-2 barrier half BUILT → PARTIAL (via J-80 MAP FIX); shipped RRA and E-25 checkpoint facts supersede older “absent/planned-only” and reclassify I-45/I-52 as extension/activation; GS-1 corrects I-11 from PLANNED to PARTIAL (belief_state.rs + condensed receipt path live; J-83 adds carrier unification beyond).
- **Atoms:** `I-002/I002-GM-032`; `J-002/DJ002-G-004`; `J-010/J80-E003`
- **RESOLUTION:** **RESOLVED — later PARTIAL/SHIPPED facts win over stale planned-only memory.** Map must be read with MAP FIX / grounding-map corrections applied.

### CF-023 — J-20 non-stationary TS supersedes D-5 / D-29 / H-48 plain-TS

- **Feature:** Thompson-sampling promotion mechanism.
- **Claim A (older plans):** plain TS slices in D-5 / D-29 / H-48 remain the design.
- **Claim B (J-20 NEW file, never-amend rule):** J-20 **SUPERSEDES** plain-TS slices of D-5/D-29/H-48 with discounted/sliding-window non-stationary TS + change-point reset + security-invariant promotion gate. D-29 hierarchical BAI promotion called DARK for the plain-TS mechanism J-20 replaces.
- **Atoms:** `J-010/J20-E002`, `J20-E013`, `J-010/IX-006`, `J-007/J0-U020`; J-AGG SUPERSEDED-NOTE block
- **RESOLUTION:** **RESOLVED — J-20 is target TS mechanism** (later family J + explicit SUPERSEDING NEW file). Older plain-TS slices retained as superseded mechanism notes, not deleted plans.

### CF-024 — J-52 default-on daemon vs E-52 dark enable

- **Feature:** J-52 per-edit enforcement hooks vs consumed E-52 probe.
- **Claim A:** J-52 new per-edit daemon is default-on behind a disable.
- **Claim B:** consumed E-52 probe is dark behind an unallowlisted enable with real subprocess execution deferred.
- **Atoms:** `J-002/DJ002-CON-008` (J-AGG SUPERSEDED-NOTE)
- **RESOLUTION:** **UNRESOLVED at mechanism-composition level** — plan does not state whether J-52 supersedes, removes, or bypasses the old E-52 enable. See **RA-002**.

### CF-025 — M-1 license surface: CLI+banner vs SettingsWindow

- **Feature:** M-1 entitlement / license status UI.
- **Claim A (deep):** target surfaces are `elai license status` + ExpiredGrace status banner + B-43 install-end activation; numbered M-1 does **not** name SettingsWindow.
  - Atoms: `M-001/M1-C001`, `M-001/SD-M1-007`
- **Claim B (summary):** license-status line paired with SettingsWindow (W-4 deferred layout).
  - Atoms: `SUMMARY/M1-E043`, `SUMMARY/M1-E044`
- **RESOLUTION:** **RESOLVED — deep CLI+banner target wins** (M-AGG SUP-001). SettingsWindow pairing retained as orientation-only speculation.

### CF-026 — M enterprise rider numbering: M-7+ vs residual M-6+

- **Feature:** Enterprise rider plan numbering after M-6 reassignment to evidence-format open standard (2026-07-05).
- **Claim A:** M-7+ is the renumbered rider track.
- **Claim B:** summary body L141 still says M-6+.
- **RESOLUTION:** **RESOLVED — M-7+** (M-AGG SUP-002).

### CF-027 — N program inventory: 16 units vs “all 15 sessions”

- **Feature:** N-plan unit count / external track N-13..N-16.
- **Claim A (deep N-0 CORRECT):** inventory is N-1..N-16 (**16 units**); external track appended.
- **Claim B (summary header):** “All 15 sessions”; open-ambiguity treats 15 vs 16 as unresolved.
- **RESOLUTION:** **RESOLVED — 16 units** (N-AGG SUP-001).

### CF-028 — N-6 provenance scales: six vs five

- **Feature:** N-6 provenance scale count.
- **Claim A (deep):** six independent provenance scales.
- **Claim B (summary):** “five provenance scales” premise.
- **RESOLUTION:** **RESOLVED — six** (N-AGG SUP-002).

### CF-029 — N-5 router set: E-39 in/out of ten routers

- **Feature:** N-5 policy arbitration router inventory.
- **Claim A (deep):** numbered source does **not** list E-39 among ten routers; only L-6 vs J-73 is trigger example.
- **Claim B (summary):** “J-84 escalation versus E-39 cost-block” as first-class router conflict.
- **RESOLUTION:** **RESOLVED — E-39 is not in the ten-router list** (N-AGG SUP-006). Summary example retained as superseded/qualified.

### CF-030 — O-4 canvas editing tier

- **Feature:** O-4 workflow canvas edit capability.
- **Claim A (deep original):** Tier 1 = read-only inspection; TaskSpec write-back + provenance drill-down deferred Tier 2.
- **Claim B (summary reading risk):** node editing as immediate ship-now feature.
- **RESOLUTION:** **RESOLVED — Tier 1 read-only** (O-AGG SUP/SD-MC003).

### CF-031 — O-2 stage/sink vocabulary vs Tier-1 runtime

- **Feature:** O-2 TaskSpec stage/sink production wiring.
- **Claim A:** schema names eight stages and four sinks.
- **Claim B:** Tier 1 explicitly names only Fetch/Classify/Gate/Digest and DashboardInbox; SsbEvent, SpawnResearchTree, RunPlanUnit, missed-run wiring are Tier 2.
- **RESOLUTION:** **RESOLVED — schema availability ≠ production wiring; Tier-1 subset is ship scope** (O-AGG SUP/SD-MC004). Same pattern: O-1 topic kinds schema-four / Tier-1 steal-scan+monitor only (SUP/SD-MC005).

### CF-032 — L-map substrate status labels (DARK / WIRED / PLANNED)

- **Feature:** Cross-family substrate flags as L grounding map sees them (not a single product unit).
- **Verified L-map claims (grounding inventory, not re-litigation of each unit’s own plan):**
  - DARK: G-7 conformal (`ELAI_CONFORMAL_REFUSAL_ENABLE`), D-16 UniCR, D-13 adversary (`ELAI_ADVERSARY_ENABLE`), I-4 search ladder (`ELAI_SEARCH_LADDER_ENABLE`), E-43 skill system (`ELAI_SKILL_SYSTEM_ENABLE`), D-11 checker (`ELAI_CHECKER_ENABLE`).
  - WIRED: B-39 reflect pipeline; B-29 symbol impact API; I-40 wrap + RRA verify.
  - PLANNED: J-65 power-of-ten; J-63 local-lane claudeception; J-102 config-ingress.
- **Atoms:** `L-001/LCGG-E007..E021`, `L-001/LOQF-E005..E025`, `L-001/LCGG-E039`
- **RESOLUTION:** **RESOLVED as L-map orientation at discovery time** — these labels are L’s composition map of other families. When a later family explicitly SHIPS or REJECTS a unit (e.g. Q-15 S-1, J-20 supersession), that later explicit marker overrides L’s snapshot for that unit’s disposition. L-map DARK means “activate under own kill-switch; do not rebuild,” not “absent.”

### CF-033 — PHASE TokenUsage field set (design 4 vs shim 2)

- **Feature:** Phase-0 / early TokenUsage schema.
- **Claim A:** design block 4 fields including `total_tokens` and `estimated_cost_usd`.
- **Claim B:** 0.2c shim 2 fields.
- **Atoms:** `P0-C001`, `PHASE-001/AC-001`
- **RESOLUTION:** **UNRESOLVED at field-set finalization** until implement recon chooses one schema and migrates the other. See **RA-003**.

### CF-034 — PHASE SPLADE deferral home: 1G vs 1H

- **Feature:** SPLADE deferral ownership label.
- **Claim A:** 1G (source).
- **Claim B:** 1H (MASTER summary).
- **Atoms:** `P1C-C002`, `PHASE-001/AC-006`
- **RESOLUTION:** **RESOLVED for implement pointer — prefer phase source 1G** over MASTER summary home when they conflict (MASTER is orientation). If later 1H file explicitly owns the work, that higher phase number would win; not observed as an explicit SUPERSEDED marker here.

### CF-035 — PHASE 4I media models: media-validator Mode A/B vs architecture-overview

- **Feature:** Media model inventory authority.
- **Claim A:** architecture-overview media models (stale LLaVA note etc.).
- **Claim B:** phase-4I binds **media-validator Mode A/B** as authority; flags overview stale.
- **Atoms:** `P4I-C001`, `PHASE-004/AC-010`, `PHASE-004/SD-4I-006`
- **RESOLUTION:** **RESOLVED — media-validator is authority.**

### CF-036 — PHASE 4C ruff path: subprocess+breaker vs library-binding note

- **Feature:** 4C Python lint integration.
- **Claim A (adversarial note):** “ruff should use library binding.”
- **Claim B (decided):** subprocess + circuit breaker when binding unavailable is authoritative.
- **Atoms:** `P4C-C001`, `PHASE-004/AC-008`
- **RESOLUTION:** **RESOLVED — subprocess+breaker wins.**

### CF-037 — PHASE 1F prerequisite: Phase-0-independent vs 1C dependency graph

- **Feature:** 1F dependency declaration.
- **Claim A:** original prerequisite “Phase 0 independent of 1A-1E.”
- **Claim B:** corrected to Phase **1C** branch dependency.
- **Atoms:** `P1F-C001`, `PHASE-005/AC-001`
- **RESOLUTION:** **RESOLVED — 1C dependency is target.**

### CF-038 — H-49 CAS / RV-1: PARTIAL map flag vs SHIPPED production caller

- **Feature:** H-49 time-travel / CAS restore legs (also feeds CF-005 #3).
- **Claim A:** some map/plan text flags H-49 PARTIAL.
- **Claim B (J-80 recon):** CAS/RV-1 legs SHIPPED with production caller `timetravel_cmd.rs`.
- **RESOLUTION:** **RESOLVED — treat CAS/RV-1 as SHIPPED for consumers**; broader H-49 unit may still carry PARTIAL for other legs until H aggregate states full DONE. Loser: blanket PARTIAL without leg split.

### CF-039 — I-11 belief_state: PLANNED vs PARTIAL

- **Feature:** I-11 BeliefState condensed receipt path.
- **Claim A:** older PLANNED.
- **Claim B (GS-1 / J GRC grounding map):** PARTIAL — `belief_state.rs`, `BeliefState::new`, `emit_condensed_receipt`, `BeliefStateEventKind::Condensed`, live contract; J-83 adds further carrier unification.
- **Atoms:** `J-002/DJ002-G-004`
- **RESOLUTION:** **RESOLVED — PARTIAL** for the live condense seam.

### CF-040 — D-1 / D-8 archive hash formula (two-part vs three-part)

- **Feature:** Archive content-hash formula for staged eval / mutation proposal chain.
- **Claim A (D-1):** two-part (body+parent).
- **Claim B (D-8):** three-part (+`mutation_proposal_text`).
- **Atoms:** D-AGG SUP-002; `D-002/D1-E16`, `D-002/D8-E03`, `D-002/X-C01`
- **RESOLUTION:** **UNRESOLVED** — neither fully supersedes; both retained. See **RA-004**.

### CF-041 — D-7 canary block rule (non-target-only vs any-family)

- **Feature:** D-7 canary family block predicate.
- **Claim A:** objective non-target-only.
- **Claim B:** implementation ANY-family below threshold.
- **Atoms:** D-AGG SUP-008
- **RESOLUTION:** **UNRESOLVED** — both retained for implement reconciliation. See **RA-005**.

### CF-042 — D-2 “different family” PPM vs D-13 cross-family predicate

- **Feature:** Whether Qwen Math PPM counts as cross-family adversary for D-13.
- **Claim A (D-2 objective wording):** PPM as “different model family.”
- **Claim B (D-13 + D-2 body):** Qwen Coder + Qwen Math are same family; PPM is heterogeneous specialization/base, **not** a valid D-13 cross-family verifier.
- **Atoms:** `D-003/D003-ISSUE-C001`, `D-003/D003-DELTA-002`
- **RESOLUTION:** **RESOLVED — D-13 same-family rejection wins.** D-2 PPM is not D-13 adversary.

### CF-043 — D-13 ModelFamily predicate wording vs required tests

- **Feature:** `cross_family_check` enum-variant vs higher-level family grouping.
- **Claim A:** prose says different enum variants suffice.
- **Claim B:** required test says `Qwen25Coder` + `Qwen25Math` must still compare same-family/false → needs unstated higher-level family grouping.
- **Atoms:** `D-003/D003-ISSUE-C004`
- **RESOLUTION:** **UNRESOLVED on implementation predicate shape** — target *behavior* is same-family reject for those two; exact enum design open. See **RA-006**.

---

## 3. Residual ambiguity (precedence cannot close)

| ID | Feature | Why precedence fails | Anchors (both sides) |
|---|---|---|---|
| **RA-001** | **P-21** gate-credibility Tier boundary | Same numbered plan: REVISED scope says skipBackfill / structural redaction / posture labels are Tier-2, while WIRING_CONTRACT, stub_ban, observable_effect smoke, and PRODUCTION_HOOK require them in this changeset. No SUPERSEDED marker chooses one half of the file. | Winner-intent: `P-001/SD-P21-001` Tier-2 deferral. Conflicting same-file: `P-001/P21-C001` wiring/stub/smoke/hook. Aggregate: P-AGG SUP-008, SUP-013; `P-21:L18-L71`. **UNRESOLVED** |
| **RA-002** | **J-52 vs E-52** enable semantics | J-52 default-on daemon vs E-52 dark enable; plan does not say supersede / remove / bypass. No explicit SUPERSEDING of E-52 enable by J-52. | `J-002/DJ002-CON-008`; J-52:L16,L22,L30,L69,L96. **UNRESOLVED** |
| **RA-003** | **TokenUsage** field set | Design 4 fields vs 0.2c shim 2 fields; no later marker names the final schema. | `P0-C001`, `PHASE-001/AC-001`. **UNRESOLVED** |
| **RA-004** | **D-1 vs D-8** archive hash formula | Two-part vs three-part; D-AGG SUP-002 explicitly keeps conflict open. | `D-002/D1-E16`, `D-002/D8-E03`, `D-002/X-C01`. **UNRESOLVED** |
| **RA-005** | **D-7** canary block rule | Non-target-only objective vs ANY-family implementation; neither deleted. | D-AGG SUP-008. **UNRESOLVED** |
| **RA-006** | **D-13** family-grouping predicate | Behavior target clear (Coder+Math same-family); enum/grouping implementation unspecified. | `D-003/D003-ISSUE-C004`. **UNRESOLVED** |
| **RA-007** | **P-12** frame hash algorithm | Frame-binding text says sha256 of image bytes; hard rules elsewhere require blake3. Same unit, no SUPERSEDED. | `P-001/P12-A003`; `P-12:L19, L32, L44, L73`. **UNRESOLVED** (algorithm, not tier) |
| **RA-008** | **N-8** five views vs Kind enum completeness | Target substrate = five views; Kind list names four and omits explicit J-70 Traceability — enum completeness open (N-AGG SUP-008). | `N-002/N8-E003`, `N8-A002`, `N0-A001`. **UNRESOLVED** (schema completeness) |
| **RA-009** | **Q-11 vs Q-14** novelty canary ownership | Summary/integration assigns novelty canary / `--self-test-canary` to Q-11 path; Q-14 later says Q-11 hardens only runtime `novelty.rs` and adds `--self-test-novelty-canary` on harness. Chronology/recon required. | `SUMMARY/AMB-15` (`17-Q:L740`); Q-14 E-1 SHIPPED atoms. **UNRESOLVED** (overlap/supersession of instrument home) |
| **RA-010** | **G unit DONE** promotion | Multiple G units have shipped seam anchors while header stays PLANNED; no file declares overall DONE. Precedence resolves *seam* ground truth (CF-019) but cannot invent unit-level DONE. | `G1-C004`, `G4-C005`, `G5-C007`, `G7-C010`. **UNRESOLVED** at initiative DONE word only |

---

## 4. Inventory appendix — per-family superseded-note counts

These are **within-family** SUP/PREC tallies already authored inside each aggregate. This cross-family file does **not** re-list every wording delta; it lifts status/shape disagreements into CF-* records above.

| Family | Aggregate file | Superseded / precedence notes (footer) | Covered in this register |
|---|---|---:|---|
| PHASE | `PHASE-AGGREGATE.md` | 22 (12 SUP + 10 PREC) | CF-006, CF-007, CF-033..037, RA-003 |
| B | `B-AGGREGATE.md` | 101 | CF-010..014 |
| C | `C-AGGREGATE.md` | 62 | (shape deltas stay in family SUP; no extra status seed) |
| D | `D-AGGREGATE.md` | 18 | CF-015..017, CF-040..043, RA-004..006 |
| E | `E-AGGREGATE.md` | 70 | (family-internal; L-map cites E-43 DARK in CF-032) |
| F | `F-AGGREGATE.md` | 20 | CF-018 |
| G | `G-AGGREGATE.md` | 27 | CF-019..021, RA-010 |
| H | `H-AGGREGATE.md` | 40 | CF-038 (via J-80 recon) |
| I | `I-AGGREGATE.md` | 30 | CF-022, CF-039 |
| J | `J-AGGREGATE.md` | 28 | CF-005, CF-023, CF-024, RA-002 |
| K | `K-AGGREGATE.md` | 25 | CF-003 |
| L | `L-AGGREGATE.md` | 9 | CF-032 |
| M | `M-AGGREGATE.md` | 4 | CF-025, CF-026 |
| N | `N-AGGREGATE.md` | 9 | CF-027..029, RA-008 |
| O | `O-AGGREGATE.md` | 10 | CF-030, CF-031 |
| P | `P-AGGREGATE.md` | 37 | CF-004a..p, RA-001, RA-007 |
| Q | `Q-AGGREGATE.md` | 6 | CF-001, CF-002, CF-008, CF-009, RA-009 |
| **Total family-local SUP/PREC** | — | **518** | — |

---

## 5. Closing audit block

| Metric | Value |
|---|---|
| **Families covered** | **17/17** (PHASE, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q) |
| **Conflict records (CF-*)** | **43** top-level (CF-001..CF-043), with CF-004 expanded into **16** tier sub-records (004a..004p) and CF-006 into **4** phase sub-records (006a..006d) |
| **Countable conflict units** | **43 + 15 sub-rows beyond 004a/006a already counted in parents** → treat as **58** distinct status/shape disagreement units if sub-rows counted separately; **43** if only top-level CF ids |
| **Resolved (top-level CF with clear winner)** | **37** (CF-001..003, 004-as-block for a–f/h–p, 005..023, 025..032, 034..039, 042) |
| **Partially resolved / mixed** | **3** (CF-004g→RA-001; CF-016 mixed; CF-017 partial; CF-019 seam-only) — counted in resolved-with-caveat rather than UNRESOLVED where a target *reading discipline* exists |
| **UNRESOLVED residual (RA-*)** | **10** (RA-001..RA-010) |
| **Strict resolved vs unresolved (status units)** | Using discrete units including CF-004a–p and CF-006a–d as separate: **resolved ≈ 55**, **unresolved = 10** (RA set); CF-004g and CF-024/033/040/041/043 fold into RA |
| **Seed conflicts verified** | **7/7** seed groups present: Q-14 E-1, Q-15 S-1, K-4 VETO, P tier list (15/20/22/12/13/18/21), J-80 doc-drift, PHASE-007 dep-graph/payload, 4J empire_* vs empire_event |
| **Write surface** | Only this file; inputs read-only; no git operations |
| **Zero-deletion rule** | All losing claims kept as superseded notes with aggregate anchors |

### Confidence

- **Verified:** seed conflicts and SUP/AC anchors cited above against aggregate text on 2026-08-12.
- **High confidence:** resolution outcomes that rest on explicit SUPERSEDED / VETO / SHIPPED / REJECTED / REVISED-scope markers inside deep ledgers.
- **Uncertain:** whether some family-internal C-class atoms outside the status vocabulary (pure naming, pure path remaps) should be promoted into CF-* later; they remain in family SUP inventories (518 total) and are not re-hosted here unless they change ship/wire/tier/disposition.

### How to use this register

1. When a plan and a summary disagree on status, look up the unit here first.
2. Apply the RESOLUTION line; do not re-derive from summary orientation alone.
3. If the unit is in the residual table, stop and recon before implement — do not paper over with letter precedence alone.
4. For wording-only deltas, open the family aggregate SUP section (counts above).
