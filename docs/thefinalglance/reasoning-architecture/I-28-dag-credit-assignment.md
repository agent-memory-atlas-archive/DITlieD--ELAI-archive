# DAG credit assignment (I-28)

- **ELAI ids:** I-28 / ID-4 / ID-5 / FORK-1..FORK-4 / Slice 1–3
- **Spec status (as designed):** PLAN **PLANNED / NOT-BUILT** in the feature map (`ELAI-FEATURE-MAP.md:1845-1847`, `:3438`). The **authoritative plan** is the ratified BUILD-WITH-SEAM file (`I-plan/I-28_dag_credit_assignment.md`). Live code exists **default-OFF** (`ELAI_DAG_CREDIT_ENABLE`, `dag_credit.rs`, `dispatch_dag_credit`) — implementation truth, not a map flip.
- **Crate / module (spec):** `crates/orchestrator/src/dag_credit.rs` (`DagCreditAssigner`, Slice 2 `DpoPairExtractor`); production `task_runner.rs::drive_task` → `dispatch_search_ladder` → `dispatch_dag_credit` **before** `maybe_run_escalation_waterfall`.
- **Purpose of this file:** enough architecture to rebuild branch-grain credit + gated DPO capture. Not a status ledger.

## 1. What it is

**DAG credit assignment** is orchestrator-owned, **deterministic** credit on I-4’s sealed `SearchLadderOutcome`. **Slice 1** writes **branch-grain** credit from **winner score − sibling score** (clamped) as `DagCreditAssigned` `Kind=CreditWalk`. **Slice 2** captures **pruned** verdicts into **DPO pairs** only from **sealed RRAs**. **Slice 3** (retrieval-chunk demotion) stays **deferred** until every branch has `chunk_id` provenance. **Shapley/SHARP is declined** (`I-plan/I-28_dag_credit_assignment.md:1-25,52-57`). `I-SUMMARY_INDEX.md:512-525` still describes “walk backward from COMPLETE” and “highest-L2-scoring VETOED sibling” — that DPO negative is **superseded by FORK-1** (progress_score on pruned rows, **never** an L2 score on a veto) (`:91`).

## 2. Why it exists

I-4 pays for a winner and ranked/pruned siblings. Without credit, that compute never trains anything honest. If the organ is faked (LLM-authored credit, L2 on HardVeto, unsealed RRA as DPO, Shapley over a missing tree), ID-4 and ID-5 die (`:52-57,110-115`). H-46 clade scoring and I-30 SFT **consume** this; they must not re-derive credit (`:13-14,43-45`).

## 3. How it works (runtime)

1. **Who calls it.** `drive_task` **after** `dispatch_search_ladder` returns `Option<SearchLadderOutcome>` and **before** I-7 waterfall (`:20,27-29,47-48`). Do **not** reconstruct the outcome by reading SSB (`:29`). The model does not call `DagCreditAssigner`.
2. **Input.** I-4 `SearchLadderOutcome` (winner + ranked siblings; Slice 2 also `pruned`). I-17 `VerdictRow` **read-only**. Never mutate HardVeto score / `VETO_SCORE` (`:12,43,97`).
3. **Gate (deterministic).** `ELAI_DAG_CREDIT_ENABLE` default-OFF. Unset → **zero** credit rows (`:20,50,100`). Also requires ladder enable for the producer (`:49`).
4. **Slice 1 credit (deterministic).** For each **non-winner ranked sibling**: `credit = (winner.score - sibling.score).clamp(0.0, 1.0)`. One `CreditWalk` row per such sibling: winner score, sibling score, ranked position, delta (`:52-53,65`). No winner → empty (`dag_credit.rs:19-21`).
5. **Slice 2 DPO (gated, after FORK-1).** Negative key = **LEAN**: extend `SearchLadderOutcome.pruned`, select by preserved **`progress_score`** (not L2). Unsealed RRA → **zero pairs**. Sealed winner + selected pruned sibling → one `DpoPairExtracted` with verifiable provenance (`:21,66,91`). JSONL writer pairs with I-30’s first reader **or** stays wire-dark `promotion_plan_id=I-30` (`:21,45,93`).
6. **Slice 3.** **No** `ChunkDemoted` until per-branch `chunk_id` exists and the unblock decision is recorded (`:24,67,82,99`). Do not invent a synthetic chunk id (`:88`).
7. **Commit.** `DagCreditAssigned` action type (flat + payload-kind). Append after `ScopeCeilingAdvisory` in the registry; **derive** SCHEMA from registry length — do not hardcode a future number (`:30-31`). Plan-time count: 606 entries, epoch −309, derived **297** (`:31,72`).
8. **Fail-closed.** LLM credit rejected. Vetoed branch with L2 rejected without mutating `VerdictRow`. Env leak (credit while flag unset) is ABORT (`:100,110-114`).
9. **The model may generate** none of the credit. It may **not** supply a score that bypasses the sibling-delta rule (`:110-111`).

## 4. Types, enums, objects

Copy from `I-plan/I-28_dag_credit_assignment.md:35-38,52-53,65` and live structs. Do not invent.

```
DagCreditAssigner::run(outcome: &SearchLadderOutcome) -> Vec<DagCreditAssignedPayload>
  // Slice 1; no LLM, no Shapley

DpoPairExtractor::run(outcome, &DpoProvenance) -> Vec<DagCreditAssignedPayload>
  // Slice 2; live fill of FORK-1 (a)

DpoProvenance { rra_sealed: bool }

fn dag_credit_enabled() -> bool
  // truthy: 1|true|yes|on; else false
ELAI_DAG_CREDIT_ENABLE   // default-OFF

DagCreditAssignedKind {
  CreditWalk,          // Slice 1
  DpoPairExtracted,    // Slice 2
  ChunkDemoted,        // Slice 3 RESERVED — do not emit until unblock
}

DagCreditAssignedPayload {
  kind: DagCreditAssignedKind,
  branch_id: String,
  credit_score: f64,         // clamp [0,1] for CreditWalk
  winner_score: f64,
  sibling_score: f64,        // live DPO sets 0.0 (veto; not L2)
  ranked_position: usize,
  ts_ms: i64,
}

credit_score (Slice 1) = (winner.verdict.score - sibling.verdict.score).clamp(0.0, 1.0)

// FORK-1 DPO-negative key (choose LEAN now)
(a) LEAN: SearchLadderOutcome.pruned + max preserved progress_score
(b) also gate id on pruned SSB row
(c) first-vetoed-by-generation-order
// NEVER L2 on a vetoed branch (ID-4)
```

**I-SUMMARY / FEATURE-MAP cousin (do not rebuild):** “winner vs highest-L2-scoring VETOED sibling” (`I-SUMMARY_INDEX.md:515-516`, `ELAI-FEATURE-MAP.md:1845-1846`). FORK-1 **forbids** that L2 (`:91`).

**Live DPO selection:** `pruned.iter().max_by(progress_score then reverse branch_id)` (`dag_credit.rs:62-68`). Option (a).

## 5. Control flow / state machine

**Owner:** orchestrator `dispatch_dag_credit`. Generator owns none of the scores.

```
dispatch_search_ladder -> Option<SearchLadderOutcome>
  → if !ELAI_DAG_CREDIT_ENABLE: return (zero rows)
  → DagCreditAssigner::run  → CreditWalk × (ranked.len - 1)
  → [Slice 2] if FORK-1 landed: DpoPairExtractor (sealed RRA only)
  → emit_dag_credit_to_ssb
  → maybe_run_escalation_waterfall   // I-7 AFTER credit
```

**Forced:**

- Node-grain credit waits for **v3 PUCT + provenance** (FORK-2) (`:25,92`).
- Slice 3 blocked on chunk provenance owner (FORK-4: pristine-context builder **vs** retrieval layer — **undecided**) (`:94`).
- Frontend credit-heat **deferred** (`:61-62`).
- Live two-gate: also skips if `checker_runtime` unplumbed (`task_runner.rs:14192-14198`) — fill, not in the plan body.

**Live DPO wire:** production calls `DpoProvenance { rra_sealed: false }` (“seal hook currently unsealed”) (`task_runner.rs:14204-14208`) → **zero** DPO rows in prod until a seal hook exists. Rebuild: **sealed=true** is the Slice 2 contract; hardcoding false is a **temporary** seam, not the organ.

## 6. Grounding, receipts, gates

Credit is a **function of I-17 survivor scores** already on the outcome. It is **not** a new verifier.

**Never allowed as credit / DPO evidence:** LLM-authored score (`:52,110`); L2 on HardVeto (`:12,54,113`); unsealed RRA (`:55,112`); Shapley/SHARP (`:56,98`); synthetic `chunk_id` (`:88`); fabricated measurements (`:59`).

Selector for Slice 2 negative (FORK-1): **progress_score on PrunedBranch**, which I-4 preserves **before** prune (`I-04` `PrunedBranch.progress_score`). Veto score stays **0.0**.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| credit range | clamp **[0, 1]** | per sibling row |
| CreditWalk count | one per ranked non-winner sibling | per outcome |
| DPO | **one** pair per sealed outcome (live extractor) | Slice 2 |
| SCHEMA floor (plan-time) | **297** derived; re-read registry (`:31,78`) | lock test |
| flags | both ladder + credit ENABLE for smoke (`:49,117`) | prod |

No token budget here (I-26). No Shapley exponential enumeration (`:25`).

## 8. Roles and neighbors

Role-agnostic host pass. I-30 consumes DPO JSONL (`:14,45`). IS-4 archive + H-46 clade scoring **downstream** — do not reimplement (`:13,44`). I-4 is **blocking** (`:9`). N-9 later unifies attribution engines including I-28 (`ELAI-FEATURE-MAP.md:3295`) — cite, do not merge.

**Must not fork:** a second credit formula inside I-30; reading SSB to rebuild the outcome (`:29`).

## 9. Config / flags / defaults

- **`ELAI_DAG_CREDIT_ENABLE`** default-OFF (`:20,50`).
- Requires `ELAI_SEARCH_LADDER_ENABLE=1` for the producer (`:49`).
- Wire-dark allowlist row **required**: `promotion_plan_id=I-28-promotion` (`:20,108`). **Live grep of `wire-dark-allowlist.toml` found no I-28 row** — treat as a **spec requirement**, not as optional.
- Slice 2 writer: `promotion_plan_id=I-30` if unpaired (`:93`).
- No dashboard in this unit (`:61-62`).
- Plan-refresh unit must **not** edit `ssb.rs` / `task_runner.rs` (`:32,122-123`) — that bound is for the **plan author**, not the rebuild harness.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `dag_credit.rs::DagCreditAssigner` | `:36` |
| `DagCreditAssigned` + `CreditWalk` | `:37` |
| `DpoPairExtracted` + dataset | `:38` |
| `dispatch_search_ladder` → `Option<SearchLadderOutcome>` | `:29` |
| `dispatch_dag_credit` in `drive_task` | `:20,28,47` |
| `i28_dag_credit_contract.rs` | `:20,105,117` |
| live `emit_dag_credit_to_ssb` | `dag_credit.rs:82` |
| live Kind enum | `ssb/payloads.rs:21945-21962` |
| evidence | `conductor/opus_walk/evidence/I-28/seam-design-proposal.md` §5–§7; `seam-design-ratification.md` (`:70-71`) |

## 11. Journal / SSB events

**`DagCreditAssigned`** (flat action type + payload kind) (`:30`). Kinds: `CreditWalk` (Slice 1), `DpoPairExtracted` (Slice 2), `ChunkDemoted` (reserved, **no emit**) (`ssb/payloads.rs:21945-21949`).

sql_str live: `dag_credit_assigned` (contract). Re-derive SCHEMA from registry length (`:31`). Contract floor **297** (`i28_dag_credit_contract.rs:23`).

## 12. Tests and observables

Named (`:64-67,77-117`):

| Id | Assert |
|---|---|
| Slice 1 seeded | one CreditWalk per ranked sibling; fields = winner score, sibling score, position, delta |
| formula | e.g. 0.9−0.4 → 0.5 clamp (live `credit_formula_unit`) |
| flag OFF | zero DagCreditAssigned rows |
| flag ON + ladder ON | CreditWalk rows + schema floor + sql_str round-trip |
| Slice 2 sealed | one DpoPairExtracted; unsealed → zero |
| FORK-1 | highest `progress_score` among pruned (live `dpo_selects_highest_progress_among_pruned`) |
| Slice 3 | zero ChunkDemoted until provenance decision |
| REDTEAM | LLM credit rejected; unsealed RRA zero pairs; L2-on-veto rejected; env leak zero rows; no synthetic chunk_id |

Off-agent VERIFY list is in the plan (`:103-108`) — auditor-owned; this architecture file does not claim those runs.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Host `DagCreditAssigner` computes **winner_score − sibling_score**, clamp [0,1], **no LLM**.
- Production path: ladder outcome **Option** fed forward; **not** SSB round-trip reconstruction.
- `CreditWalk` rows default-OFF behind `ELAI_DAG_CREDIT_ENABLE`.
- HardVeto / `VETO_SCORE` **untouched**. No L2 on vetoed branches.
- Slice 2 DPO negatives from **pruned progress_score** (FORK-1 LEAN) and **sealed RRA only**.
- Shapley/SHARP **absent**. Slice 3 **absent** without `chunk_id`.
- SCHEMA derived from registry; floor documented, not invented.

An LLM judge of “which branch taught us more,” or SUMMARY’s L2-veto DPO pair, is a **cousin**.

## 14. Non-goals / banned designs

- Shapley / SHARP (`:4,25,56,98`).
- Slice 3 without provenance (`:24,57,99`).
- LLM credit (`:52`).
- Mutating frozen `VerdictRow` (`:97`).
- Unsealed RRA DPO (`:55`).
- Node-grain credit before v3 PUCT (`:25,92`).
- Dashboard heat in this unit (`:61-62`).
- Hardcoding a future SCHEMA_VERSION (`:31`).
- Credit rows with the env flag unset (`:100`).

## 15. Source citations

Primary: `docs/thefinalglance/I-plan/I-28_dag_credit_assignment.md` (full file). Index (older DPO wording): `I-plan/I-SUMMARY_INDEX.md:512-525`. Feature map (stale PLANNED + L2-veto): `ELAI-FEATURE-MAP.md:1845-1847`. Doctrine ID-4: `I-plan/I-0_INDEX.md:235-239`. Live: `dag_credit.rs`, `task_runner.rs:12667,14177-14227`, `ssb/payloads.rs:21945-21962`, `crates/contracts/tests/i28_dag_credit_contract.rs`. Evidence: `conductor/opus_walk/evidence/I-28/seam-design-{proposal,ratification}.md` as cited in the plan.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- FEATURE-MAP “NOT-BUILT [D-026 GPU]” vs live `dag_credit.rs` — map stale; do not invent a new status beyond PARTIAL/DARK gated.
- Required I-28 allowlist row **missing** from live `wire-dark-allowlist.toml` (plan `:20,108`).
- Production `rra_sealed: false` hardcoded (`task_runner.rs:14204-14208`) vs Slice 2 sealed contract.
- Whether `DpoPairExtracted` is its **own** action type (plan `:38`) or a **Kind** on `DagCreditAssigned` (live). Rebuild the **Kind-on-DagCreditAssigned** live shape unless a later plan splits them.
- `ranked_position` uses enumerate on full `ranked` (includes winner at 0) so first sibling is 1 (`dag_credit.rs:24-33`) — plan says “ranked position” without the indexing convention.
- FORK-1 (b)(c) not implemented; only (a).
- FORK-3 JSONL path not in live `dag_credit.rs`.
- FORK-4 provenance owner undecided (`:94`).
- `ts_ms: 0` in `DagCreditAssigner::run` (`dag_credit.rs:34`) vs real clock.
- Live `checker_runtime` second gate (`task_runner.rs:14192`) not in the plan.
- SCHEMA 297 was plan-time; live `SCHEMA_VERSION_PUBLIC >= 297` floor only (`i28_dag_credit_contract.rs:23`).
