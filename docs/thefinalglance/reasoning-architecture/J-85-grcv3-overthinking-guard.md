# GRC-v3 deterministic overthinking guard (J-85)

- **ELAI ids:** J-85 / GA-3 / OG-1..OG-5 / WAVE GRC-2 / IMPL 23/28/29/30/31
- **Spec status (as designed):** PLANNED — the **ONE genuinely-new** component of GRC-v3 (everything else is unification/activation). Grep-confirmed absent: `oscillation` hits only `anti_spiral_extended.rs` + telemetry, a **different** detector (`J-plan/J-85_grcv3_overthinking_guard.md:1-5`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:70-74`, `ELAI-FEATURE-MAP.md:668-680`).
- **Crate / module (spec):** `crates/orchestrator/src/overthinking_guard.rs` (new); frozen hesitation word-list under gate-manifest R8; sits at GRC Verify beside `anti_spiral_extended.rs`; ceiling hosted by I-26; freeze composed with J-83.
- **Purpose of this file:** enough architecture to rebuild the per-call ceiling + 3-signal drift-off-correct early-stop. Not a status ledger.

## 1. What it is

The **overthinking guard** is a deterministic per-call **reasoning-token ceiling** plus a **3-signal early-stop monitor** that catches a chain **drifting off a previously correct answer**. Accuracy is non-monotonic in reasoning-token budget: it rises, peaks, then falls via a “negative flip”. The three paper-correlated signals (external, WHY only) are answer **oscillation** (r=0.78, the **lead**), **hesitation markers** `"wait"/"but"/"actually"` (r=0.71), and **declining confidence** (r=0.63); `flip_ratio>1.0` is the regime. The guard is **DISTINCT from anti-spiral**: anti-spiral catches **REPETITION**; this organ catches **DRIFT**. Both run side by side. The knee is **K-15-measured per (model, task-type)**, never the paper’s 6K/7K. No LLM decides when to stop (`J-plan/J-85_grcv3_overthinking_guard.md:5,13-19`, `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:70-74,159-161`).

GRC-v3 = unification + activation of existing pieces **plus this one net-new organ**.

## 2. Why it exists

A run can keep generating past its useful peak and **abandon a previously-correct intermediate answer**. That is invisible to anti-spiral (which only sees repetition: same hash, same error class, A-B-A-B strings, diminishing error counts — `C-10-anti-spiral-lateral-thinking.md:10-12,23-27`) and to J-84 (which **sizes** the budget up front and does not watch during the call) (`J-plan/J-85_grcv3_overthinking_guard.md:11`). If this organ is missing, receipt-confirmed J-83 entries can be second-guessed away (IMPL 31’s overthinking-defense leg is empty). If it is faked (LLM “stop now”, or a copied 7K token cap, or merging it into anti-spiral’s `StagnationPattern::Oscillation`), small models overthink **earlier** than the paper’s R1-32B and the wrong stop fires (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:94-96,162-165`).

Honest residue (R-3): `flip_ratio` is **lagging**; the guard acts on the **lead** (rising oscillation) and accepts some false-positive early stops (paper precision 76.3%). Tune on ELAI data (K-15), never on the paper threshold (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:159-161`). R-4: local Qwen2.5-Coder-7B overthinks earliest/lowest; its self-eval is noise; this guard matters **most** on that lane (`J-plan/J-85_grcv3_overthinking_guard.md:118`).

## 3. How it works (runtime)

1. **Who calls it.** GRC Verify / `task_runner` reasoning loop, **per call**, **beside** anti-spiral. J-83 commit path for freeze. Not a `#[cfg(test)]` caller (R7) (`J-plan/J-85_grcv3_overthinking_guard.md:56-59`).
2. **Input.** Running generation (intermediate-answer region), frozen hesitation word-list, H-39 confidence-bit trajectory over steps, J-84 tier, K-15 knee artifact, J-83 committed entries, failed-receipt retry count. **No LLM judge** (`J-plan/J-85_grcv3_overthinking_guard.md:11,61`).
3. **OG-1 per-call ceiling (deterministic).** Hard reasoning-token cap from the J-84 tier, **bounded by** the K-15 per-(model, task-type) knee. Host: I-26 `BudgetController`. Hitting the ceiling → **stop + commit last receipt-confirmed state**. **Never** hard-code 6K/7K / R1-32B / s1-32B (`J-plan/J-85_grcv3_overthinking_guard.md:15,40,83`). `rg` of `7000|7K|5000|paper.*knee|R1.32B.*threshold` in `overthinking_guard.rs` MUST be 0 (`J-plan/J-85_grcv3_overthinking_guard.md:76`).
4. **OG-2 3-signal monitor (deterministic, at Verify):**
   - **Oscillation (lead):** count intermediate-answer **flips**. Extraction must be cheap and deterministic (structured field or regex over the running answer region). An LLM re-read is a **material fork / TD-3 violation — STOP + handoff** (`J-plan/J-85_grcv3_overthinking_guard.md:16,119`).
   - **Hesitation:** scan against a **FROZEN** word-list (`wait` / `but` / `actually` named in the plan). Gate-manifest R8; not LLM-generated (`J-plan/J-85_grcv3_overthinking_guard.md:16,63`).
   - **Declining confidence:** H-39 bit trajectory over steps (READ, never re-derived) (`J-plan/J-85_grcv3_overthinking_guard.md:16,25`).
   - **`flip_ratio>1.0` = STOP** (negative flips exceed positive — the **regime**, lagging vs oscillation lead) (`J-plan/J-85_grcv3_overthinking_guard.md:16`).
5. **Act on the lead.** Stop when rising oscillation fires, do not wait for `flip_ratio` if the lead is already up. Accept false-positive early stops; observe-only first if false-stop rate is a concern (R-3) (`J-plan/J-85_grcv3_overthinking_guard.md:117`).
6. **On stop.** Commit the **last receipt-confirmed** J-83 state, **not** the drifted continuation (`J-plan/J-85_grcv3_overthinking_guard.md:67-69`).
7. **OG-3 distinct-from-anti-spiral.** Run **beside** `anti_spiral_extended.rs`. Do **not** merge. Both can fire. Record **which** fired. Negative test: repetition-only → anti-spiral **only**; drift-only → guard **only** (`J-plan/J-85_grcv3_overthinking_guard.md:17,41,69`). C-10’s `StagnationPattern::Oscillation` is last-4-entries **alternating two strings** (`anti_spiral_extended.rs:46,243-246`) — **error/output hash period-2**, not answer-drift. Do not reuse that enum as OG-2.
8. **OG-4 freeze composition.** On J-83 receipt-confirmed commit, freeze the entry; post-commit generator rewrite dropped+logged (IMPL 31). The guard **enforces** that freeze against oscillation (`J-plan/J-85_grcv3_overthinking_guard.md:18,42`).
9. **OG-5 modality-change.** After **~3** failed **receipted** retries, signal `MODALITY_CHANGE` to J-84/J-86 (retrieve / I-7 escalate / I-8 abstain). **Never** “reason more” (IMPL 23). The guard **signals**; J-84/J-86 **act** (`J-plan/J-85_grcv3_overthinking_guard.md:19,43`).
10. **Commit journal.** `+OverthinkingGuardFired` Pattern-2 (`Kind: CeilingHit / OscillationStopped / HesitationStopped / ConfidenceDeclined`) **OR** reuse `ReasoningBudgetScaled` (+Kind). Net 0..+1, decide at edit (W-9) (`J-plan/J-85_grcv3_overthinking_guard.md:68`).
11. **Fail-closed.** LLM “keep going / stop now” is ignored (TD-3). Paper knee literals banned. Hesitation list must be the frozen fixture, not regenerated (`J-plan/J-85_grcv3_overthinking_guard.md:100`).

The model may generate the chain. It may **not** decide the stop, edit the word-list, raise the ceiling, or unfreeze a committed entry.

## 4. Types, enums, objects

Copy from `J-plan/J-85_grcv3_overthinking_guard.md:13-19,46-48,68`. Do not invent fields.

```
OverthinkingStopReason {
  CeilingHit,
  OscillationStopped,      // lead signal
  HesitationStopped,
  ConfidenceDeclined,
}

// Plan SSB Kind names match the four reasons above.

fn flip_ratio(...) -> f64
  // STOP when > 1.0  (lagging regime indicator)

HesitationWordList   // FROZEN fixture; plan names "wait" / "but" / "actually"
                     // gate-manifest R8; not LLM-generated

fn monitor_overthinking(running_generation, h39_trajectory, knee) -> Option<OverthinkingStopReason>
  // deterministic; no LLM

fn per_call_ceiling(j84_tier, k15_knee_artifact) -> TokenCeiling
  // per-(model, task-type); never a paper constant

MODALITY_CHANGE      // after ~3 failed receipted retries
                     // targets: retrieve | I-7 escalate | I-8 abstain
                     // guard signals; J-84/J-86 act
```

**Consumed, not redefined:** J-83 freeze; I-26 `BudgetController`; H-39 trajectory; C-10 `anti_spiral_extended.rs` (beside, not merged); K-15 knee artifact.

**Do not reuse as this organ:** `StagnationPattern { Spinning, Oscillation, NoDrift, DiminishingReturns }` (`C-10-anti-spiral-lateral-thinking.md:49`). That `Oscillation` is repetition.

Paper correlations r=0.78 / 0.71 / 0.63 and 76.3% precision are **[EXTERNAL]** — journal **why**, never as ELAI numeric thresholds (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:85-96`).

## 5. Control flow / state machine

**Owner of the during-call loop:** orchestrator monitor at GRC Verify. J-84 owns **up-front** tier. J-83 owns **commit**. Anti-spiral owns **repetition**. K-15 owns **knee measurement**. I-26 hosts the token envelope.

```
per reasoning call:
  ceiling = per_call_ceiling(J-84 tier, K-15 knee[model, task_type])
  anti_spiral.detect(history)                 // repetition; DISTINCT
  guard.monitor(generation, H-39 traj)        // drift; DISTINCT
     │
     ├─ tokens >= ceiling            → STOP CeilingHit
     ├─ rising oscillation (lead)    → STOP OscillationStopped   // do not wait for flip_ratio
     ├─ hesitation density (frozen list)
     ├─ declining H-39 trajectory
     └─ flip_ratio > 1.0             → STOP (regime; lagging)
     │
     ▼ on any stop:
        commit LAST J-83 receipt-confirmed entry
        drop drifted continuation
        journal which detector fired (anti-spiral vs guard vs both)
     │
     ▼ if ~3 failed receipted retries:
        signal MODALITY_CHANGE  (not more reasoning)
     │
     ▼ J-83 commit:
        freeze entry; rewrite dropped (OG-4)
```

**Forced:** per-lane knees (do not share one knee across local Qwen-7B and a 32B cloud model) (`J-plan/J-85_grcv3_overthinking_guard.md:118`). Safety guard is **default-ON** behind `*_DISABLE` + allowlist (not an experiment) (`J-plan/J-85_grcv3_overthinking_guard.md:92`).

**Parallel with J-84:** classifier sets budget **up front**; guard stops **during**. Disjoint seams (`J-plan/J-85_grcv3_overthinking_guard.md:8`).

## 6. Grounding, receipts, gates

Stop commits the last **receipt-confirmed** state. A trace that “looks done” is **not** a stop signal (RR-1 / IC-6). Confidence trajectory is H-39 **receipt-read**, not generator self-grade.

**Never allowed as a stop signal:** LLM “I should stop”; paper 6K/7K; anti-spiral repetition pattern reused as drift; window fill; majority vote.

Selector hierarchy unchanged: external receipt > vote-over-independent-samples (J-86, receipt-absent only) > LLM self-eval banned.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| Per-call ceiling | K-15 knee per (model, task-type), bounded by J-84 tier | per call |
| `flip_ratio` stop | `> 1.0` | per call (lagging) |
| Paper r / precision | r=0.78 / 0.71 / 0.63; 76.3% @ 80% recall | **EXTERNAL WHY only** — not ELAI knobs |
| Paper knees (banned as code) | R1-32B ~7K, s1-32B ~5K; Qwen-7B overthinks earlier | do not transfer (`J-plan/J-85_grcv3_overthinking_guard.md:11`) |
| Failed receipted retries | **~3** → `MODALITY_CHANGE` | per claim (`J-plan/J-85_grcv3_overthinking_guard.md:19`) |
| SSB | +OverthinkingGuardFired **or** reuse ReasoningBudgetScaled; net 0..+1 from 268/577 snapshot | W-9 |
| Size | ~160 LOC guard + frozen fixture | plan |
| Flag | default-ON `*_DISABLE` + allowlist | safety, not experiment |

**Do not confuse with:** H-42/J-84 **up-front** budget; 2A `must_converge` at `remaining_calls<=1` (format loop, not drift); C-10 3-identical-hash stop (repetition); I-26 85% search collapse (sibling spend, not answer oscillation).

## 8. Roles and neighbors

Role-agnostic safety guard. Matters most on the **local small-model** lane (R-4).

**Consumes:** J-83 freeze; J-84 tier; I-26 ceiling host; H-39 trajectory; anti-spiral (beside); K-15 artifact; I-7/I-8 modality targets.

**Produces:** stop reason; committed last-confirmed state; `MODALITY_CHANGE` signal; optional `OverthinkingGuardFired` row.

**Must not fork:** a second anti-spiral; an LLM stop-token policy; one global token cap for all models.

**Depends:** J-83. **Parallel-safe with:** J-84, K-15.

## 9. Config / flags / defaults

- Default-ON behind `*_DISABLE` + allowlist row (W-2). Exact env name **UNCERTAIN** (§16).
- Hesitation list is a **frozen fixture** in the gate-manifest R8 set. Editing it is a frozen-gate change, out-of-band (`J-plan/J-85_grcv3_overthinking_guard.md:48,63`).
- Frontend: chosen-stop-reason on existing GRC/reasoning panel + I-27 ConvergencePanel. Decide tile at edit (W-4); justified N/A allowed (`J-plan/J-85_grcv3_overthinking_guard.md:64-65`).
- Decisions doc: `workflow/reasoning/J-85-decisions.md` (distinct-from-anti-spiral + measure-your-own-knee) (`J-plan/J-85_grcv3_overthinking_guard.md:43`).
- Observe-only first **if** false-stop rate is a concern (R-3); the plan still marks the organ default-ON as a safety guard — composition of those two clauses is **UNCERTAIN** (§16).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/overthinking_guard.rs` | CREATE ~160 LOC |
| frozen hesitation-marker fixture | CREATE; gate-manifest R8 |
| GRC Verify / `task_runner` | MODIFY ~25 LOC: monitor + ceiling **beside** anti-spiral |
| `anti_spiral_extended.rs` | READ; do not merge (`anti_spiral_extended.rs` Oscillation ≠ OG-2) |
| `routing/budget_controller.rs` | OG-1 host |
| `abstention_score.rs` | OG-2 trajectory |
| J-83 `belief_state.rs` freeze | OG-4 |
| `ssb.rs` + `action_type_lock.rs` + golden | iff +OverthinkingGuardFired |
| `crates/contracts/tests/j85_overthinking_guard_contract.rs` | CREATE ~70 LOC |
| `workflow/reasoning/J-85-decisions.md` | CREATE ~25 LOC |

## 11. Journal / SSB events

Plan (`J-plan/J-85_grcv3_overthinking_guard.md:68`):

```
OverthinkingGuardFired {     // Pattern-2, if added
  kind: CeilingHit | OscillationStopped | HesitationStopped | ConfidenceDeclined
}
```

**OR** reuse `ReasoningBudgetScaled` (+Kind). Decide at edit. Stop decision **recomputable from the journal** (GD-2) (`J-plan/J-85_grcv3_overthinking_guard.md:97`). Record which of {anti-spiral, guard} fired (OG-3). Do not rewind schema to 268/577.

## 12. Tests and observables

Named contract `j85_overthinking_guard_contract.rs` (`J-plan/J-85_grcv3_overthinking_guard.md:99-100`):

| Id | Assert |
|---|---|
| (a) | `flip_ratio>1.0` (rising oscillation) **STOPS** |
| (b) | per-call ceiling stops at configured **measured** knee; commits last confirmed state |
| (c) | **DISTINCT** from anti-spiral (repetition-only vs drift-only) |
| (d) | J-83 committed entry frozen against post-commit rewrite |
| (e) | 3 failed receipted retries → `MODALITY_CHANGE`, not more-reasoning |
| (f) | LLM “keep going / stop now” **ignored** |
| (g) | hesitation word-list is the **frozen fixture**, not regenerated |

**W-1 smoke:** drive a task tuned to overthink past its peak; guard stops at/near the measured knee; receipt-confirmed state committed, not the drifted one. Negative: detectors distinct; LLM-proposed stop never the gate (`J-plan/J-85_grcv3_overthinking_guard.md:66-69,91`).

**DETERMINISTIC_CHECK:** 3 signals + ceiling present; LLM-stop count 0; paper-knee literals 0; `anti_spiral` referenced as DISTINCT; production_caller 0 DARK; `J-85-decisions.md` exists (`J-plan/J-85_grcv3_overthinking_guard.md:71-79`).

Adversarial: merging OG-2 into `StagnationPattern::Oscillation`; transferring 7K to Qwen-7B.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- It is the **one net-new** GRC-v3 mechanism (not a rename of anti-spiral or H-42).
- Hard per-call reasoning ceiling from a **measured per-(model, task-type) knee**, never a paper 6K/7K.
- Three deterministic signals: intermediate-answer oscillation (lead), frozen hesitation-list scan, H-39 declining trajectory; `flip_ratio>1.0` is the lagging regime.
- Acts on the lead; commits last **receipt-confirmed** state, not the drifted tail.
- Anti-spiral (repetition) and this guard (drift) are **two detectors**; both can fire; negative tests prove exclusivity.
- J-83 freeze is enforced against post-commit oscillation.
- ~3 failed receipted retries signal modality change, not more chain.
- No LLM stop. Frozen word-list. Production caller at Verify.

A “max_tokens=7000” cap, a prompt “don’t overthink”, or C-10’s period-2 hash Oscillation is a cousin, not this organ.

## 14. Non-goals / banned designs

- Merging with anti-spiral (GA-3) (`J-plan/J-85_grcv3_overthinking_guard.md:62`).
- LLM deciding when to stop (TD-3).
- Paper knee hard-coded (ID-5); the paper **forbids** transferring R1-32B ~7K / s1-32B ~5K (`J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md:94-96`).
- Sharing one knee across model lanes (R-4).
- LLM re-read to extract “intermediate answers” (TD-3 fork) (`J-plan/J-85_grcv3_overthinking_guard.md:119`).
- Regenerating the hesitation list at runtime (R8).
- More reasoning after ~3 failed receipted retries (IMPL 23).
- Using C-10 `StagnationPattern::Oscillation` as OG-2.
- Making this organ an experiment default-OFF (it is a safety guard, W-2).

## 15. Source citations

Primary: `docs/thefinalglance/J-plan/J-85_grcv3_overthinking_guard.md` (full file). Grounding map: `J-plan/_grounding/J-GRC-V3-GROUNDING-MAP.md` GA-3, R-3/R-4. Index: `J-plan/J-SUMMARY_INDEX.md:1885-1899`. Feature map: `ELAI-FEATURE-MAP.md:668-680`. Neighbor: `reasoning-architecture/C-10-anti-spiral-lateral-thinking.md:10-12`. Live: `anti_spiral_extended.rs:46,112,195,243-246`; `trust_telemetry.rs:25,191`. Design: `workflow/reasoning/reasoning-engine-grc-v3-2026-06-25.md` §6 + IMPLICATION 28–31 (as cited by the J-85 plan).

arXiv **[EXTERNAL, WHY]:** overthinking 2604.10739 (`J-plan/J-85_grcv3_overthinking_guard.md:112`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Intermediate-answer extraction method (structured field vs regex). Plan forbids LLM re-read; does not pick the cheap method (`J-plan/J-85_grcv3_overthinking_guard.md:119`).
- Numeric hesitation **density** threshold (list is named; fire-on-any vs count-per-N-tokens is unspecified).
- Numeric oscillation-flip count before the lead fires (paper r is not an ELAI count).
- How `flip_ratio>1.0` combines with “act on the lead” when oscillation is up but ratio is still ≤1.0 (plan says act on lead; tests still key (a) on `flip_ratio>1.0`).
- Exact `~3` retries: 3 vs ≥3; what “receipted” means (failed validator vs missing receipt).
- Env name for `*_DISABLE`.
- SSB new type vs reuse `ReasoningBudgetScaled`.
- Whether default-ON (W-2) and R-3 observe-only-if-false-stops-matter can both hold at first ship.
- Full frozen word-list beyond the three named tokens.
- K-15 artifact path/schema this ceiling reads.
