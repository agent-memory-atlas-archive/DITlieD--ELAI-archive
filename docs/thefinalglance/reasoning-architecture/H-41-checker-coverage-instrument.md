# Checker-Coverage Instrument (H-41)

- **ELAI ids:** H-41 / RG-41 / RD-3 / RS-1..RS-3
- **Spec status (as designed):** PARTIAL (feature-map 1.2); original plan PLANNED Track-6 instrument over D-10/D-11
- **Crate / module (spec):** `crates/orchestrator/src/checker_coverage.rs` (`mode_a_coverage`, `classify_complexity_taxon`); emit at `agent_pool.rs:2030`; `ssb.rs::SsbActionType::CheckerGapDetected`
- **Purpose of this file:** enough architecture to rebuild the live Mode-A coverage roll-up and ranked checker-gap backlog. Not a status ledger. Does **not** add the sound-checker-first preference (already shipped).

## 1. What it is

The **live checker-coverage + checker-gap instrument** measures two production signals D-10/D-11 do not: (a) `mode_a_fraction = #Constructive / #(Factual|Computational claims)` this session, plus Mode-B / refusal / deferred fractions; (b) a ranked **checker-gap backlog** of claim-classes that fell to `FalsifiabilityCrossFamily` or `ConformalRefusal` **because no constructive checker matched**. Each gap row is classified by the Xu Table-3 complexity taxonomy. NP-complete / combinatorial / undecidable taxa are HIGH-priority: **route to a sound solver or refuse, never LLM-grade** (`H-plan/H-41_checker_coverage_gap_instrument.md:5-16`, `:59-76`, `H-REASONING-GROUNDING-MAP.md:103-106`).

## 2. Why it exists

D-10 **already** routes a Factual/Computational claim “constructive when possible, cross-family when not, refusal otherwise” (`grounding_mode.rs:157`, Rule 5) and firewalls Auditor/Meta → `ConservativeFallback` (Rule 1). The preference is SHIPPED (RS-1). D-21 is an **offline** F1 fixture benchmark, not live coverage (RS-3). Absent (RG-41): the measurement that turns “prefer a sound checker” into a managed program. Kambhampati 2402.08115: LLM self/peer verification **collapses** on reasoning/planning; sound **external** verifiers work. Xu 2401.11817 Table 3 names classes that **force** LLM hallucination (NP-complete SAT/SubsetSum; combinatorial Ω(2^n); undecidable first-order entailment). If this organ is missing, “where do we need a new checker?” is heuristic. If it is faked (a dashboard that restates Rule 5 without journal roll-up), Mode-B fallbacks are invisible. Honest residue (GD-5): coverage measures **REACH** to a sound checker, not that the checker is correct (checker bugs are H-26/H-27) (`H-plan/H-41_checker_coverage_gap_instrument.md:7-16`, `:26-36`, `:78-81`).

## 3. How it works (runtime)

1. **Who calls it.** Live claim-grading turn: `agent_pool` → `select_grounding_mode` (`agent_pool.rs:2030`). Backlog read: `elai checker-gaps` (or dashboard) (`H-plan/H-41_checker_coverage_gap_instrument.md:64-76`, `:92-95`).
2. **Preference consumed, not restated.** H-41 does **not** re-implement `select_grounding_mode` or `checker_registry` (GD-1 P0) (`:97-98`).
3. **Coverage roll-up (read-only, no new `action_type`).** Over shipped grounding-mode decisions + `CheckerResultEmitted` rows for the session: `mode_a_fraction = #Constructive / #(Factual|Computational claims)`, plus Mode-B / refusal / deferred fractions. Journal-recomputable under `ELAI_SEED` (GD-2) (`:59-62`, `H-REASONING-GROUNDING-MAP.md:147-149`).
4. **Gap emit (off-agent).** At the `:2030` call site, when a Factual/Computational claim routes to `FalsifiabilityCrossFamily` or `ConformalRefusal` **because no D-11 constructive checker matched**, emit `CheckerGapDetected` `{claim_class, complexity_taxon, fell_to_mode}`. The **agent does not author** this row (`H-plan/H-41_checker_coverage_gap_instrument.md:64-68`).
5. **Covered path.** A Factual claim **with** a matching checker routes `Constructive` and emits **no** gap row (`:105-108`).
6. **Xu classifier (deterministic).** Map claim → `complexity_taxon ∈ {DecidableHasChecker, DecidableNoChecker, NpCompleteOrCombinatorial, Undecidable, Unknown}`. NP-complete/combinatorial and Undecidable = HIGH: must solver-or-refuse, **never** LLM-grade (Xu Corollary 1). A gap on those is a **missing solver**, not merely a missing checker (`:70-73`). Misclassification risk: surface `Unknown` rather than guessing (`:170-171`).
7. **Ranked backlog.** Aggregate `CheckerGapDetected` rows ranked by `frequency × taxon-priority` — the answer to “which sound checker buys the most Mode-A coverage next?” (`:75-76`).
8. **Instrument, not a gate.** Does **not** BLOCK a Mode-B fallback. A follow-on gate that forces refusal on uncovered NP-complete claims composes D-16 **after** numbers justify it (GD-4) (`:174-175`).
9. **Near-empty registry.** `mode_a_fraction ~ 0` and the backlog is the whole claim distribution — the **correct** honest signal (“invest in checkers”), not a bug (`:172-173`).
10. **R2.** `--ignore-checker-gaps` **must** make `mode_a_fraction` mis-report. An instrument that stays right when broken is a FAIL (`:99`, `:110`).
11. **Fail-closed.** NP-complete / undecidable claim routed to an LLM grade **as if verified** = FAIL (RD-3) (`:97`). blake3; no unwrap in prod.

The model may generate the claim text. It may **not** choose Mode A/B, classify taxon, or suppress a gap row.

## 4. Types, enums, objects

Copy from `H-plan/H-41_checker_coverage_gap_instrument.md:39-41`, `:59-73`. Do not invent fields.

```
GroundingMode {                         // D-10 SHIPPED; REUSE
  Constructive,                         // Mode A
  FalsifiabilityCrossFamily,            // Mode B
  ConformalRefusal,
  OperationalDeferred,
  ConservativeFallback,                 // Auditor/Meta firewall
}

fn select_grounding_mode(claim, ctx) -> GroundingMode   // :110; call site agent_pool.rs:2030

fn mode_a_coverage(session_rows) -> ModeACoverage
  // mode_a_fraction = #Constructive / #(Factual|Computational claims)
  // plus Mode-B / refusal / deferred fractions

ComplexityTaxon {
  DecidableHasChecker,
  DecidableNoChecker,
  NpCompleteOrCombinatorial,            // HIGH; never-LLM-grade
  Undecidable,                          // HIGH; never-LLM-grade
  Unknown,                              // prefer over guessing
}

fn classify_complexity_taxon(claim) -> ComplexityTaxon

CheckerGapDetected {                    // SsbActionType; Pattern-2
  claim_class,
  complexity_taxon,
  fell_to_mode,
}

CheckerResult                           // D-11; REUSE
```

Denominator is **Factual|Computational claims**, not all claims (`:60-61`). How other claim types enter the ratio: **UNCERTAIN** (§16).

## 5. Control flow / state machine

**Owner:** orchestrator instrument around the **shipped** router. No new routing FSM.

```
claim-grading turn
  -> select_grounding_mode(claim, ctx)          // D-10
  -> if Factual|Computational:
       if Constructive: no gap row
       if fell to Mode B / ConformalRefusal
          AND no D-11 match:
            classify_complexity_taxon
            emit CheckerGapDetected
  -> session roll-up mode_a_coverage from journal
  -> elai checker-gaps ranks frequency × taxon-priority
```

**Forced:** NP-complete / undecidable never presented as LLM-verified (RD-3). H-41 itself still **does not block** Mode-B (instrument-first) (`:174-175` vs stub_ban `:97` — see §16).

## 6. Grounding, receipts, gates

**Receipts:** shipped grounding-mode decision rows + `CheckerResultEmitted`. Coverage is a **roll-up**, not a new score (`H-REASONING-GROUNDING-MAP.md:147-149`).

**Never allowed as coverage / “verified”:** LLM self/peer critique; Mode-B vote treated as Mode A; restating Rule 5 without journal counts; D-21 offline F1 as a substitute for live fraction (`H-plan/H-41_checker_coverage_gap_instrument.md:26-36`, `:44-45`).

**Selector hierarchy (consumed, D-10):** constructive checker first; else cross-family; else refusal. H-41 measures that hierarchy’s **hit rate**.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Mode-A fraction | `#Constructive / #(Factual\|Computational)` | per session, journal-recomputable |
| gap row | one per unmatched Factual/Computational fallback | per routing call |
| taxon HIGH | `NpCompleteOrCombinatorial`, `Undecidable` | never-LLM-grade flag |
| new variants | `+CheckerGapDetected` baseline+1 off `220/497`; ratio has **no** variant | SSB |
| empty registry | fraction ~0 is honest | not a bug |

## 8. Roles and neighbors

**Consumes:** `select_grounding_mode` (RS-1), `CheckerResult` / `CheckerResultEmitter` (RS-2), `agent_pool.rs:2030`, D-21 as **distinct** offline bench (RS-3). **Produces:** `mode_a_coverage`, `classify_complexity_taxon`, `CheckerGapDetected`. **Feeds:** CONSTRUCTIVE-CHECKER REGISTRY investment queue (`ELAI-FEATURE-MAP.md:629-631`).

**Must not fork:** a second router; LLM-grading NP-complete claims; treating this instrument as a hard gate on day one.

**Parallel-safe with:** H-39, H-40, H-42 (`H-plan/H-41_checker_coverage_gap_instrument.md:22`). Downstream: investing in D-11 checkers against this backlog is the high-leverage verification work (`:178-180`).

**Dashboard:** Mode-A ratio + ranked backlog in the trust cockpit (`:101-103`).

## 9. Config / flags / defaults

- Default-ON: coverage roll-up + gap signal with no env override; kill-switch only (`:123`).
- Kill-switch identifier: **UNCERTAIN**.
- Feature-map does not name an `ELAI_*` enable for H-41 (`ELAI-FEATURE-MAP.md:629-631`).
- Follow-on refusal gate on uncovered NP-complete: **not** this organ; compose D-16 after GD-4 (`H-plan/H-41_checker_coverage_gap_instrument.md:174-175`).
- Decision doc: `workflow/reasoning/H-41-decisions.md` (`:78-81`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `checker_coverage.rs::mode_a_coverage` | `H-plan/H-41_checker_coverage_gap_instrument.md:85` |
| `classify_complexity_taxon` | `:86` |
| `SsbActionType::CheckerGapDetected` | `:87` |
| consume `select_grounding_mode` | `:89` |
| consume `CheckerResult` | `:90` |
| emit at `agent_pool.rs:2030` | `:91`, `:154` |
| `elai checker-gaps` CLI | `:75`, `:156` |
| trust-cockpit Svelte pair | `:156` |
| `h41_checker_coverage_contract.rs` | `:146` |
| fixtures `fixtures/checker_gap/` | `:150` |
| `ssb.rs` + `action_type_lock.rs:624` + e45 | `:155` |

## 11. Journal / SSB events

| Event | Payload | Notes |
|---|---|---|
| `CheckerGapDetected` | `{claim_class, complexity_taxon, fell_to_mode}` | Pattern-2; off-agent; feature-map SCHEMA 219 (`ELAI-FEATURE-MAP.md:631`) — re-read live integers |
| Mode-A ratio | **none** (roll-up of shipped rows) | GD-1 |

Covered Constructive path: **no** gap row (`H-plan/H-41_checker_coverage_gap_instrument.md:107-108`).

## 12. Tests and observables

Contract (`:146-150`):

| Id | Assert |
|---|---|
| (a) | covered Factual → Constructive, no gap row |
| (b) | uncovered → `CheckerGapDetected` with right `claim_class` |
| (c) | NP-complete → taxon `NpCompleteOrCombinatorial` + HIGH |
| (d) | `mode_a_fraction` matches hand-computed fixture |
| (e) | `--ignore-checker-gaps` mis-reports (R2) |
| (f) | `ELAI_SEED` twice → identical ratio + backlog **order** |

**Observable effect** (`:104-110`): real task with (a) matching D-11 checker and (b) no matcher; (a) Constructive + no gap; (b) gap row; `elai checker-gaps` ranks (b); NP-complete no-solver flagged HIGH.

W-1: `--run-trigger` PASS. `cargo-green != DONE` (`:122`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Sound-checker-first **preference is consumed**, not rewritten.
- Live `mode_a_fraction` is a journal roll-up of Constructive over Factual|Computational claims.
- Unmatched Factual/Computational fallback emits off-agent `CheckerGapDetected` `{claim_class, complexity_taxon, fell_to_mode}`.
- Xu taxa include HIGH never-LLM-grade for NP-complete/combinatorial and Undecidable; `Unknown` rather than a guessed hard class.
- Backlog ranked by frequency × taxon-priority.
- Instrument does not silently equate Mode B with verified.
- `--ignore-checker-gaps` mis-reports coverage (R2).
- Coverage documented as **reach**, not checker-correctness.

A second `select_grounding_mode`, or an offline confusion-matrix (D-21) labeled “live coverage,” is a cousin.

## 14. Non-goals / banned designs

- Adding the A>B preference (already RS-1) (`H-plan/H-41_checker_coverage_gap_instrument.md:7-8`, `H-SUMMARY_INDEX.md:678`).
- Re-implementing D-10/D-11 (GD-1 P0).
- LLM-grading NP-complete / undecidable as verified (RD-3 FAIL).
- New `action_type` for the coverage **ratio**.
- Blocking Mode-B as this organ’s ship gate (`:174-175`).
- Claiming the checker is correct because coverage is high (GD-5) (`:80-81`).

## 15. Source citations

Primary: `docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md` (full file). Grounding: `H-REASONING-GROUNDING-MAP.md:29-40`, `:79-84`, `:103-106`, `:129-132`, `:147-149`. Doctrine: `H-plan/H-0_INDEX.md:344-347`. Index: `H-plan/H-SUMMARY_INDEX.md:672-684`. Overview: `ELAI-SPEC-OVERVIEW.md:248-249`. Feature map: `ELAI-FEATURE-MAP.md:581-584` (D-10), `:585-589` (D-11), `:629-631`. Papers: Kambhampati 2402.08115; Xu 2401.11817 Table 3.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Classifier mapping from claim text/type to taxon is heuristic; calibration fixture required (`H-plan/H-41_checker_coverage_gap_instrument.md:170-171`).
- Denominator: only Factual|Computational, or all routed claims (`:60-61`).
- Stub_ban “never LLM-grade” vs instrument-not-a-gate (does Mode-B on NP-complete still **happen**, only flagged HIGH?) (`:97` vs `:174-175`). Rebuild: **measure + HIGH flag** this organ; do not invent a hard block.
- Kill-switch name.
- SCHEMA live integer vs feature-map 219.
- `fell_to_mode` exact `GroundingMode` subset when OperationalDeferred vs ConservativeFallback (`:65-67` names FalsifiabilityCrossFamily or ConformalRefusal only).
