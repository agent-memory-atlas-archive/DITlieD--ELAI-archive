# Hallucination policy gate + abstention (L-7)

- **ELAI ids:** L-7 / L-T1 / HG-4 / WAVE 3 / CANT-WRECK
- **Spec status (as designed):** PLANNED (`L-plan/L-7_hallucination_policy_gate_abstention.md:3`). No live `claim/policy_gate.rs` / `HallucinationGateAction` / `ELAI_HALLUCINATION_GATE_DISABLE` at architecture time.
- **Crate / module (spec):** `crates/orchestrator/src/claim/policy_gate.rs`; production callers are the AutonomousGate transition seam **and** the edit/action seam (`hallucination_gate_action`) (`L-plan/L-7_hallucination_policy_gate_abstention.md:35-41`).
- **Purpose of this file:** enough architecture to rebuild the deterministic ClaimRisk→action gate. Not a status ledger.

## 1. What it is

The **hallucination policy gate** is the CGG decision surface. Predictors and L-6 produce a `ClaimRisk` tier; L-7 maps it to a **GateAction**: Low → Allow; Medium → Caveat / require citation; High → **ForceEffort** (L-6) before the claim may act; Critical → **BLOCK** the edit/action **or ABSTAIN**. HG-4: unverifiable high-risk never silent-allow; missing evidence = high risk (wiring R1 on claims). It **composes** B-22 AutonomousGate `EvidenceCheck`, D-16 conformal refusal, I-8 Aletheia abort (H-39 credited), D-17 honest-refusal SLA. The verdict is an SSB row (G-2 RRA-reproducible). The LLM never sets or overrides the action (TD-3) (`L-plan/L-7_hallucination_policy_gate_abstention.md:5`, `L-plan/L-SUMMARY_INDEX.md:53-58`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:109-111`).

## 2. Why it exists

A risk score with no action is telemetry (`L-plan/L-7_hallucination_policy_gate_abstention.md:13`). If this organ is missing, High/Critical can still edit. If it is faked (prompt “be careful”, or the model picks Allow), HG-4 is gone. If it BLOCKs by default before the false-positive rate is bounded, users disable it — the gate-killer (`:97`, GD-4, K-T3).

H-39 credits honest abstain (`abstain=0`) and punishes confident-wrong. I-8 is how the agent **acknowledges infeasible** without eating a confident-wrong penalty (`:5,13,18`).

L-15 S2: High/Critical **enter repair** (L-6) rather than abstain-first; S5 abstains only when the receipt gate fails and budget is gone (`L-plan/L-15_repair_controller_and_evidence_first_reorder.md:51-69`). That matches High→ForceEffort and Critical→Block/Abstain **after** bounded effort, not instead of it.

## 3. How it works (runtime)

1. **Who calls it.** AutonomousGate `EvidenceCheck` per transition (B-22) **and** the edit/action seam. Not contracts-only (R7) (`:40-41`).
2. **Input.** L-2 `ClaimRisk` (post L-6 re-score when ForceEffort ran). D-16 `tau` for the abstain boundary. **No LLM.**
3. **Map (deterministic)** (`:29`):

| RiskTier | GateAction |
|---|---|
| Low | `Allow` |
| Medium | `Caveat` (attach caveat / require citation) |
| High | `ForceEffort` → L-6; claim **may not act** until re-score |
| Critical | `Block` **or** `Abstain` |
| missing evidence | treat as high risk (HG-4 / R1) |

4. **Wire (compose, GD-1)** (`:30`):
   - Critical → AutonomousGate `EvidenceCheck` **FAIL** (transition cannot advance).
   - `Abstain` → I-8 `acknowledge_infeasible`, credited by H-39.
   - `Caveat` → annotate the output (citation required).
   - D-16 `RefusalDecision` / `tau` compose the abstain boundary — do not re-Platt inside `policy_gate.rs`.
5. **Observe vs enforce (GD-4, W-2).** Kill-switch `ELAI_HALLUCINATION_GATE_DISABLE`: observe-and-record, **do not block** (`:31,53`). Without DISABLE, still **observe-only** until FP/over-block is bounded on **≥3 real runs**, then BLOCK-by-default (`:5,31,66`). Promotion waits on L-10 measured FP (`:98`).
6. **Journal.** GateAction as SSB: prefer-reuse `RefusalDecision` / EvidenceCheck rows, else `+HallucinationGateAction` Kind `{Allowed, Caveated, ForcedEffort, Blocked, Abstained}` (`:51`, `L-plan/L-0_INDEX.md:136`). Joins G-2 RRA (`:13`).
7. **D-17.** Session `refusal_rate` SLA still runs (`<5%` confabulating / `>50%` misconfigured) (`D-17-honest-refusal-sla.md` §1). L-7 does not reimplement the SLA; it **feeds** `RefusalDecision` counts. L-24 rider B later adds typed `{tried, missing, route}` unblock conditions on these refusals (`L-plan/L-SUMMARY_INDEX.md:257`) — **not** WAVE-3 L-7 scope.
8. **Fail-closed.** Unverifiable High → ForceEffort or Abstain, never Allow (`:44`). LLM cannot write `GateAction` (`:45`). DISABLE does not delete the row — it only skips enforce (`:53`).
9. **The model may generate** claim text already typed/scored. It may **not** pick the action, clear EvidenceCheck, or un-DISABLE the gate.

## 4. Types, enums, objects

Copy from `L-plan/L-7_hallucination_policy_gate_abstention.md:29`.

```
enum GateAction {
  Allow,
  Caveat,          // annotate / require citation
  ForceEffort,     // hand to L-6; no act yet
  Block,           // edit/action cannot proceed
  Abstain,         // I-8 + H-39 credit
}

fn hallucination_gate_action(risk: &ClaimRisk, ctx) -> GateAction
```

**Consumed, not redefined:**

- D-16 `RefusalDecision`, `tau` (`D-16-conformal-refusal.md` §1–3).
- B-22 AutonomousGate `EvidenceCheck` / `AutoAdvanceEvaluator`.
- I-8 `acknowledge_infeasible`.
- H-39 `AbstentionScored` (`correct=+1 / abstain=0 / confident-wrong=-t/(1-t)`).
- D-17 SLA snapshot (neighbor).
- G-2 `rra.rs`.
- L-2 `ClaimRisk`; L-6 `route_targeted_effort`.

**Critical Block vs Abstain:** both legal (`:29`). Which one when is **UNCERTAIN** (§16) — L-15 S5 points leftover Critical at Abstain after repair budget; destructive-path invented package in the observable is BLOCK (`:51`).

## 5. Control flow / state machine

**Owner:** orchestrator gate at transition + edit. Deterministic over tier.

```
ClaimRisk (after L-6 if any)
  DISABLE? → journal would-be GateAction; do not fail EvidenceCheck; still never "silent" in the log
  else if observe-window (FP unbound)? → same journal-only
  else enforce:
    Low → Allow
    Medium → Caveat
    High → ForceEffort → L-6 → re-score → re-enter map
    Critical → EvidenceCheck FAIL; Block or Abstain (I-8)
```

**Forced:** High cannot act before L-6 (`:29`). Parallel-safe with L-8 (`:9`). Depends L-2 and L-6 (`:9`).

K-4 gate-tax ceiling is a **cross-link**, not a K-4 amend (`:98`).

## 6. Grounding, receipts, gates

The gate **does not** re-verify. It consumes L-2 (which already required HG-2 for High/Critical) and L-6’s attempt. Receipt of the **decision** is the SSB/RRA row.

**Never allowed as an action chooser:** LLM override, D-16 τ written by the agent, DISABLE as a silent Allow without a row, Medium Allow without caveat, High Allow.

**HG-2 reminder:** a single predictor never produced High/Critical (L-2). L-7 must not re-derive a Critical from one signal.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| policy map | ~50 LOC | `policy_gate.rs` |
| AutonomousGate wire | ~40 LOC | transition seam |
| journal | ~30 LOC | same file |
| file | ~120 LOC | (`:87`) |
| contract | ~85 LOC | (`:84`) |
| GD-4 | ≥3 real runs before BLOCK-by-default | whole CGG |
| D-17 band | <5% / >50% | session, neighbor |
| SCHEMA | reuse RefusalDecision/EvidenceCheck or +1 HallucinationGateAction | W-9 |
| kill-switch | `ELAI_HALLUCINATION_GATE_DISABLE` | W-2 |

Caveat fatigue: Medium boundary tuned against L-10 reviewer-reaching (`:99`). Not a constant here.

## 8. Roles and neighbors

Applies at **autonomous transitions** and **edit/action**. Role-agnostic otherwise. Auditor owns F-4; this gate owns whether the **claim** may act.

**Consumes:** L-2, L-6, D-16, I-8, D-17, B-22, H-39, G-2 (`:38-39`).

**Produces:** `GateAction` + journal row; EvidenceCheck fail on Critical; I-8 abort on Abstain.

**Must not fork:** a prompt policy; a second τ; LLM Allow.

**Frontend:** D-27 refusal/mode panels or justified N/A (`:48-49`).

## 9. Config / flags / defaults

- `ELAI_HALLUCINATION_GATE_DISABLE` — observe, do not block (`:31,59`).
- Default: observe-then-enforce after FP bound (GD-4), not silent BLOCK on day one (`:66`).
- D-16 remains DARK behind `ELAI_CONFORMAL_REFUSAL_ENABLE`; L-7 **composes** it when present (DC-2: activate, don’t rebuild).
- Decision doc: `workflow/reasoning/L-7-decisions.md` (Block vs Abstain; Medium caveat text). **Absent** at architecture time.
- blake3; no `.unwrap()` (`:47`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/claim/policy_gate.rs` | map + journal + observe/enforce (`:36`) |
| `hallucination_gate_action` | production name (`:41`) |
| AutonomousGate EvidenceCheck | B-22 seam (`:88`) |
| D-16 `conformal_calibration.rs` | tau / RefusalDecision |
| I-8 `acknowledge_infeasible` | Abstain |
| G-2 `rra.rs` | replay |
| `l7_policy_gate_contract.rs` | tests (a)–(g) (`:84`) |

Live: **no** `policy_gate.rs`.

## 11. Journal / SSB events

Plan (`:51`, `L-plan/L-0_INDEX.md:136`):

```
// prefer-reuse:
RefusalDecision          // D-16
AutonomousGate EvidenceCheck

// else:
HallucinationGateAction  // Pattern-2
  Kind: Allowed | Caveated | ForcedEffort | Blocked | Abstained
```

Net 0..+1. GD-2: verdict recomputable from ClaimRisk + map under `ELAI_SEED` (`:80`). DISABLE still **emits** the would-be Kind (`:53`).

## 12. Tests and observables

Named (`:84`):

| Id | Assert |
|---|---|
| (a) | Low→Allow, Medium→Caveat, High→ForceEffort, Critical→Block/Abstain |
| (b) | unverifiable High never silent Allow |
| (c) | LLM cannot override GateAction |
| (d) | DISABLE: record, do not block |
| (e) | Critical FAILs AutonomousGate EvidenceCheck |
| (f) | Abstain credited via H-39 (not penalised) |
| (g) | verdict RRA-reproducible |

**observable_effect** (`:50-53`, `L-plan/L-SUMMARY_INDEX.md:58`): invented package on a destructive path → BLOCK or Abstain, journaled, RRA-replayable. Medium ships with caveat. Negative: unverifiable High never silent-allow; DISABLE observes without blocking.

**W-1:** live Critical FAILs EvidenceCheck and the verdict is RRA-reproducible (`:73`). **R10:** hard multi-turn Critical mid-run, not a toy path (`:81`).

Adversarial: LLM Allow; BLOCK before 3-run FP bound; DISABLE as no-row; High acting before L-6.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- Closed `GateAction` `{Allow, Caveat, ForceEffort, Block, Abstain}` with the §3 table.
- High cannot act before L-6 ForceEffort. Critical fails EvidenceCheck or Abstains via I-8.
- Unverifiable High/Critical never Allow (HG-4).
- LLM has no input to the action (TD-3).
- `ELAI_HALLUCINATION_GATE_DISABLE` observes without blocking; rows still emit.
- BLOCK-by-default only after FP bounded on ≥3 runs (GD-4).
- D-16/B-22/I-8/H-39/G-2 composed, not reimplemented.
- Verdict is an SSB/RRA row.
- Tests (a)–(g). Production seams are AutonomousGate + edit/action.

A system prompt “don’t hallucinate”, or D-16 τ used as the only gate without ClaimRisk, is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-implementing D-16 / AutonomousGate / I-8 (GD-1 P0) (`:47`).
- LLM GateAction (TD-3).
- Silent Allow of unverifiable High (HG-4).
- BLOCK-by-default before ≥3-run FP bound (GD-4).
- Scoring/fusing inside this file (L-2) or running retrieval (L-6).
- Paper ε / UniCR numbers as ELAI FP bounds (ID-5).
- L-24 UnblockCondition fields (later rider).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-7_hallucination_policy_gate_abstention.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:53-58`. Doctrine: `L-plan/L-0_INDEX.md:53,99-100,136`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:31-36,70-71,109-111`. L-15 S2/S5: `L-plan/L-15_repair_controller_and_evidence_first_reorder.md:51-69`. Neighbors: `D-16-conformal-refusal.md`, `D-17-honest-refusal-sla.md`, `H-39-abstention-aware-scoring.md`. Paper mechanism via L-0; no L-7-specific AUROC.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Critical → Block vs Abstain choice (`:29` vs L-15 S5 vs destructive-path observable).
- Exact Caveat artifact (banner vs required `[N]` vs both).
- How D-16 τ and ClaimRisk Critical interact (AND vs L-7-only vs D-16-only).
- Observe-window storage (where the ≥3-run FP ledger lives).
- K-4 gate-tax numeric ceiling (cross-link only).
- Prefer-reuse vs new `HallucinationGateAction` (decide at edit).
- Frontend D-27 vs N/A.
- Whether Medium Caveat is user-visible when DISABLE is set.
- B-22 EvidenceCheck API shape (re-grep at edit).
