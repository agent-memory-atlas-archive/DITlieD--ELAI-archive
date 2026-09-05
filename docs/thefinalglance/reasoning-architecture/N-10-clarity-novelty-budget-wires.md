# Clarity / novelty → budget wires (N-10)

- **ELAI ids:** N-10 / TIER 3 / W-15+W-24 / WAVE 6 / ACTIVATION
- **Spec status (as designed):** PLANNED. Pure composition: type existing detectors once, declare consumers (`N-plan/N-10_clarity_novelty_budget_wires.md:1-12`, `ELAI-FEATURE-MAP.md:3299-3306`, `N-plan/N-0_INDEX.md:192-196,278-279`).
- **Crate / module (spec):** `crates/orchestrator` research-tree (4D) + effort-tier (J-84 via N-5) + node-admission (J-39, Tier-2) + grill-offer (J-79) seams
- **Purpose of this file:** enough architecture to rebuild the score→budget mapping that turns passive clarity/novelty detectors into an active resource-allocation loop. Not a status ledger.

## 1. What it is

**N-10** types two signals **once** and wires them into how much research/reasoning a **this-task** gets:

- **`AmbiguityScore`** (shared type) from B-38 continuous ambiguity (4 dims + floors + streak) + B-40 critic findings + C-29 intent pairs + J-32 front-door confidence.
- **`NoveltyScore`** (shared type, **joined**) from D-15 `NoveltyClass` (labels **claims**) + Phase-4F / B-21 novel-step detection (labels **plan steps**).

**Consumers:** 4D research-tree depth (`Simple` / `Moderate` / `Complex` breadth/depth); J-84 reasoning-effort tier **via N-5**; J-39 decomposition node-sizing (bound-and-escalate); J-79 grill/clarify offer. High-ambiguity **OR** off-map → deeper research + more clarify + **smaller** node bound + **earlier** escalation — not just a chip (`N-plan/N-10_clarity_novelty_budget_wires.md:8-11`, `N-plan/N-SUMMARY_INDEX.md:195-209`).

Ambiguity = “we don’t understand the ask.” Novelty = “the ask is off our map.” **Both buy depth** (`:11`).

EXCLUDED: re-detecting; a new detector; a non-deterministic depth choice. The mapping score→budget is **deterministic** (TD-3) (`:12`).

N-10 **adds consumers, not a producer**. No new SSB action_type: reuse B-38 / D-15 rows (baseline+0) (`N-plan/N-0_INDEX.md:278-279`, `:45`).

## 2. Why it exists

Four clarity detectors and two novelty detectors already run, but they only feed a **consent** surface (J-79 grill offer) or a **human queue** (D-15 `DiscoveryCandidate`). Nothing feeds measured ambiguity/novelty of **this** task into research/reasoning spend (W-15, W-24) (`:5,15-17`). If this organ is missing, a vague/off-map task gets the same 4D `Simple` tree and J-84 `Zero`/`Depth` as a clear in-map task. If it is faked (LLM “this seems hard, think more”, or a second scorer), TD-3 and GD-1 both die (`:12,39-40`).

4D already picks `QueryComplexity` from **other** inputs; J-84 already has a COST proxy (H-39, C-45/C-47, J-38, C-29); J-39 already sizes from cone-vs-ceiling. N-10 **joins** ambiguity/novelty onto those deciders; it does not replace them (`:17-18`).

This is **not** L-24’s explore-bench NLI novelty column (meaning-cluster quality under CGG ON/OFF). That measures survivor-map **diversity**. N-10 allocates **budget** from D-15 / 4F novelty (`L-24-conjecture-memory.md` §3 step 9 vs `:10`).

## 3. How it works (runtime)

1. **Who calls it.** Production planning seam: a live task where the research-tree / effort tier is chosen. A `#[cfg(test)]` caller never satisfies R7 (`:35-37`). The model does not pick depth or tier.
2. **Input.** Outputs of the **existing** detectors — do not re-run a parallel scorer (`:39`):
   - B-38 `AmbiguityScore` / `ScoreBreakdown` (`B-38-ambiguity-score-gate.md` §4).
   - B-40 critic findings (`:9,33`).
   - C-29 intent pairs (`:9`; also already a J-84 COST input — compose, don’t double-detect).
   - J-32 front-door confidence (`:9,15`).
   - D-15 `NoveltyClass` on Mode A/D claim results (`D-15-novelty-class-labeling.md` §1).
   - 4F / B-21 novel-step on plan steps (`:10,16`).
3. **Join (deterministic) into two shared types.** `AmbiguityScore` and `NoveltyScore` are the **once-typed** rust objects N-10 **declares**. B-38 already names `AmbiguityScore` — **extend/join**, do not mint a second struct with the same name (`:8-10,30` vs `B-38-ambiguity-score-gate.md` §1). How B-40/C-29/J-32 fold into one number, and how `NoveltyClass` + novel-step become `NoveltyScore`, is **UNCERTAIN** (§16). Rebuild: **one** join per signal, **no** LLM in the join, **no** new detector.
4. **Map (deterministic, TD-3).** `score → budget`. Direction is load-bearing (`:11,45-46`):

| Signal | Consumer effect |
|---|---|
| high ambiguity **OR** off-map novelty | 4D complexity **up** (toward `Complex`: breadth 4 / depth 5) (`04D-flashresearch-grc-research-agent.md:30-34`) |
| same | J-84 effort **up** (more Depth/Width / earlier EscalateOrAbstain — via **N-5**, not a second J-84) (`:18`, `J-84-grcv3-effort-classifier.md` §1) |
| same | J-39 node bound **smaller** + earlier escalation (Tier-2) (`:11,22`) |
| same | J-79 grill/clarify **more** (already the only consumer today; keep it, don’t drop it) (`:15,11`) |
| low ambiguity **AND** on-map | must **not** be forced to Complex; clear baseline keeps baseline depth (`:45`) |

Exact numeric thresholds / 4D enum cutovers are **not** in N-10 — **UNCERTAIN** (§16). Do **not** invent a table. Do **not** let the model fill one.

5. **Tier-1 (ship now):** shared types + wire **2–3 consumers**: 4D depth + J-84 effort via N-5 + J-79 offer (`:20-21`).
6. **Tier-2 (defer):** J-39 node-sizing consumer, with the decomposition unit (`:22`).
7. **Commit.** **Reuse** shipped B-38 `ambiguity_scored` (and siblings) + D-15 `NoveltyLabeled` / queue rows. **No new type** (`:45`, `N-plan/N-0_INDEX.md:278`). Depth/tier **change** is visible as those consumers’ existing rows (4D tree params, J-84 tier, J-79 offer) — N-10 does not add a `budget_wired` event unless a later plan says so.
8. **Fail-closed.** High-ambiguity task getting **baseline** depth **FAILS** (`:41,46`). Non-deterministic mapping **FAILS** (`:40`). Re-detection **FAILS** (`:39`).
9. **The model may generate** plan text that B-38/D-15 already score, and may answer a J-79 grill. It may **not** choose `QueryComplexity`, J-84 tier, J-39 admit fraction, or override the mapping.

## 4. Types, enums, objects

Copy from `:8-11,30` plus the **existing** detector types this organ **joins**. Do not invent detector fields.

```
// SHARED types N-10 declares (join, don't re-detect):
AmbiguityScore          // B-38 already: overall_score = 1 − Σ(clarity_i × weight_i)
                        // N-10 JOINS B-40 + C-29 + J-32 into this once-typed signal
NoveltyScore            // JOIN of D-15 NoveltyClass (claims)
                        //      + 4F/B-21 novel-step (plan steps)

// Existing detector enums (consumed, not replaced):
// B-38
AmbiguityScore { overall_score, breakdown, floor_failures, ... }  // B-38-ambiguity-score-gate.md §4
// ready iff overall_score <= 0.2  (HIGH overall_score = MORE ambiguous)

// D-15
NoveltyClass { Search, DistantSearch, DiscoveryCandidate }        // closed; Mode C label
DiscoveryCandidate → human queue (unchanged)

// 4D consumer (existing)
QueryComplexity { Simple, Moderate, Complex }
  Simple:   breadth 2 depth 3
  Moderate: breadth 3 depth 4
  Complex:  breadth 4 depth 5

// J-84 consumer (existing, via N-5)
tier { Zero, Depth, Width, EscalateOrAbstain }   // WINDOW is never a tier

// J-39 consumer (Tier-2)
Admitted | SplitForced | EscalatedToCloud
  // high ambiguity/novelty → smaller node bound + earlier escalation

fn score_to_budget(ambiguity: AmbiguityScore, novelty: NoveltyScore) -> BudgetDecision
  // DETERMINISTIC (TD-3). Formula UNCERTAIN (§16). Direction: high/off-map → more depth.
```

Do **not** add a third novelty detector. Do **not** add an SSB `ClarityWired` type (`N-plan/N-0_INDEX.md:278`).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator mapping. Detectors already ran; N-10 **consumes** their rows and **sets** consumer parameters before 4D/J-84 spend.

```
task intake
  → B-38 / B-40 / C-29 / J-32   → join AmbiguityScore
  → D-15 / 4F / B-21            → join NoveltyScore
  → score_to_budget (pure)
       ├─ 4D QueryComplexity     (Tier-1)
       ├─ J-84 tier via N-5      (Tier-1)
       ├─ J-79 grill intensity   (Tier-1)
       └─ J-39 node bound        (Tier-2)
  → consumers run as today (4D tree, GRC budgets, grill offer)
```

**Forced:**
- Mapping is a **function**, not a model call (`:12,40`).
- High-ambiguity **OR** off-map is enough to buy depth (OR, not AND) (`:11`).
- J-79 remains a consumer — wiring to budgets does **not** remove the grill offer (`:11,15`).
- D-15 still never autonomously claims discovery (`D-15-novelty-class-labeling.md` §1). Budget may rise; marketing copy may not.
- N-5 is the **home** for the effort-tier consumer; do not fork a second J-84 router (`:18`).

**B-38 vs N-10 polarity:** B-38 **blocks** PLANNING advance when `overall_score > 0.2`. N-10 **spends more** when ambiguity is high. Those can both be true: a still-ambiguous plan may WAIT/BLOCK **and** (once allowed, or on the research tree that precedes it) get a deeper 4D envelope. Exact sequencing vs GroundingSweepGate is **UNCERTAIN** (§16).

## 6. Grounding, receipts, gates

N-10 **does not** ground claims. It **allocates** based on already-journaled detector receipts.

**Selector hierarchy for budget:** joined AmbiguityScore + NoveltyScore (deterministic map) > J-84’s other COST proxies (compose via N-5) > **not** LLM “think harder” > **not** window size as a tier (J-84 RD-4).

**Never allowed as a budget chooser:** LLM self-rating of difficulty; non-deterministic sampling of depth; a new detector; paper knees (J-84/K-15 still own measured effort knees; N-10 mapping is a **wire**, ND-4 if it claims a number — route through N-1 if a threshold is later measured).

**D-15 cosine ≥ 0.85 near-duplicate** is **that** organ’s heuristic (`D-15-novelty-class-labeling.md` §3). N-10 **wires** the resulting `NoveltyClass`. It does **not** re-introduce embeddings as an orbit detector (L-23 EX-2 is a different organ).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| new SSB types | **0** | organ (`N-plan/N-0_INDEX.md:278`) |
| new detectors | **0** | (`:12,39`) |
| Tier-1 consumers | 4D + J-84/N-5 + J-79 | (`:21`) |
| Tier-2 consumer | J-39 | (`:22`) |
| 4D Simple/Moderate/Complex | 2×3 / 3×4 / 4×5 breadth×depth | existing (`04D-flashresearch-grc-research-agent.md:30-34`) |
| B-38 ready threshold | `overall_score <= 0.2` | existing (`B-38-ambiguity-score-gate.md` §3) |
| mapping | deterministic; numeric cuts **unspecified** | this organ |
| N-5 | effort-tier consumer home | (`:18`) |

N-10 does not change 4D GRC 10/1200 or Designer 8/1200. It changes **which QueryComplexity** (and thus breadth/depth) is selected (`04D-flashresearch-grc-research-agent.md:10`).

## 8. Roles and neighbors

**Consumes (wire, never re-detect):** B-38, B-40, C-29, J-32, D-15, 4F, B-21 (`:33`).

**Consumes (budget deciders):** 4D research-tree, J-84 via N-5, J-39, J-79 (`:34`).

**Produces:** shared rust types + the mapping + the wires (`:29-31`). **Not** a new journal family.

**Must not fork:** a second AmbiguityScore module; a second J-84; LLM depth pick; L-24 NLI novelty as this `NoveltyScore`; N-5 arbitration of **routers** is a different organ — N-10 **feeds** N-5’s effort axis, it does not re-author precedence (`N-plan/N-SUMMARY_INDEX.md:104-123`).

**Frontend:** N/A (internal allocation), unless depth choice is surfaced on the canvas; justified (`:26,42`).

## 9. Config / flags / defaults

- N-10 names **no** ENABLE flag. Detectors keep theirs (D-15 `ELAI_NOVELTY_LABEL_ENABLE` default-OFF is a live cousin — wiring a dark D-15 cannot invent novelty; fail-closed to “no novelty signal”, not to a guessed Complex).
- Mapping is always-on composition once wired (`:21`).
- No new dependency implied.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| shared `AmbiguityScore` / `NoveltyScore` | join types (`:30`) |
| `score_to_budget` | deterministic mapping (`:21,30`) |
| 4D `research_tree.rs` / `QueryComplexity` | depth consumer (`:25,34`) |
| J-84 via N-5 intensity resolver | effort consumer (`:18,25`) |
| J-39 `node_admission.rs` | Tier-2 (`:22,25`) |
| J-79 grill offer | clarify consumer (`:34`) |
| B-38 `crates/reasoning/src/ambiguity.rs` | detector (`B-38-ambiguity-score-gate.md` header) |
| D-15 `novelty.rs` | detector (`D-15-novelty-class-labeling.md` header) |

Exact mapping-function filename is **not** named (`:25` lists seams). Do not treat `clarity_budget.rs` as specified.

## 11. Journal / SSB events

From `:45` and `N-plan/N-0_INDEX.md:278`:

```
// NO new action_type. Reuse:
B-38  ambiguity_scored / ambiguity_floor_failed / ambiguity_streak_advanced
D-15  NoveltyLabeled / DiscoveryCandidateQueued / DiscoveryRatified / DiscoveryRejected

// Consumer rows already specified by those organs (4D tree, J-84 tier, J-79 offer)
// must MOVE for a vague/novel task vs a clear baseline.
```

Do not mint `clarity_wired` / `novelty_budgeted`.

## 12. Tests and observables

From `:43-46,37-41`:

| Assert | Notes |
|---|---|
| deliberately **vague OR off-map** task vs **clear baseline** | live planning seam |
| research depth **and** effort tier **higher** on vague/novel | reading the **same** B-38 / D-15 scores |
| high-ambiguity task at **baseline** depth | **FAIL** |
| no re-detection | wire shipped detectors |
| mapping deterministic | same scores → same budget under `ELAI_SEED` (implied GD-2; N-10 does not restated-say it, but TD-3 + journal reuse require it) |
| no new SSB type | reuse B-38 / D-15 |
| `#[cfg(test)]` sole caller | **FAILS** R7 |

N-10’s source does **not** name a `n10_*_contract.rs`. Rebuild from observable + stub_ban.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- **No** new detector. B-38/B-40/C-29/J-32 join to **one** `AmbiguityScore`. D-15 + 4F/B-21 join to **one** `NoveltyScore`.
- Deterministic `score_to_budget`. LLM never picks depth/tier. High-ambiguity **OR** off-map buys **more** research + **more** clarify + (Tier-2) **smaller** nodes + earlier escalation.
- Consumers: 4D `QueryComplexity`, J-84 via N-5, J-79; J-39 deferred.
- **Zero** new SSB action_type. Observable: vague/novel task journals **higher** depth/tier than a clear baseline from the **same** detector rows.
- High-ambiguity → baseline depth is a **FAIL**.
- J-79 grill remains; D-15 discovery honesty remains.

A prompt “if confused, think longer”, a second ambiguity scorer, or only a grill chip with unchanged 4D/J-84 is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Re-detecting (`:12,39`).
- A new detector (`:12`).
- Non-deterministic score→budget (`:12,40`).
- High-ambiguity at baseline depth (`:41`).
- New SSB type (`N-plan/N-0_INDEX.md:278`).
- A second J-84 beside N-5 (`:18`).
- Using L-24 NLI cluster novelty as this budget signal.
- Using D-15 embeddings as L-23 orbit detection (wrong organ).
- WINDOW as a reasoning tier (J-84).
- Autonomously claiming discovery because budget rose (D-15).

## 15. Source citations

Primary: `docs/thefinalglance/N-plan/N-10_clarity_novelty_budget_wires.md` (full file). Index: `N-plan/N-SUMMARY_INDEX.md:195-209`. Doctrine: `N-plan/N-0_INDEX.md:192-196,278-279`. Feature map: `ELAI-FEATURE-MAP.md:3299-3306`. Overview: `ELAI-SPEC-OVERVIEW.md:419-428`. Neighbors: `B-38-ambiguity-score-gate.md`, `D-15-novelty-class-labeling.md`, `04D-flashresearch-grc-research-agent.md`, `J-84-grcv3-effort-classifier.md`, `J-39-decomposition-node-admission.md`. Interlace: W-15 (ambiguity never feeds budgets), W-24 (novelty never feeds budgets); W-15 extension = J-39 fourth customer (`:48-49`).

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** N-10 atom block. Do not mint N10-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric `score_to_budget` thresholds and 4D `Simple|Moderate|Complex` cutovers (`:11` names direction only).
- How B-40 / C-29 / J-32 fold into B-38’s existing `AmbiguityScore` (weights? max? any-high?).
- How `NoveltyClass` + novel-step become `NoveltyScore` (enum vs scalar). What “off-map” is: `DistantSearch` vs `DiscoveryCandidate` vs 4F novel-step.
- Sequencing vs B-38 BLOCK/WAIT: does N-10 allocate 4D **before** the sweep gate, or only after ALLOW? (`B-38-ambiguity-score-gate.md` §3 vs N-10 planning seam).
- Dark D-15 (`ELAI_NOVELTY_LABEL_ENABLE`): mapping behavior when novelty rows are absent.
- Mapping-function filename (`:25` lists seams only).
- Whether N-5 precedence can **cap** a high-ambiguity depth request (cost-ceiling vs this wire) — N-5 exists to arbitrate; N-10 is silent (`N-plan/N-SUMMARY_INDEX.md:104-123`).
- J-79 “more clarify” intensity (repeat grill? extra question? already one-question grill in other skills — J-79’s own spec owns that).
- Named contract-test file (none in source).
- Whether C-29 is counted twice (J-84 COST axis **and** N-10 AmbiguityScore join) without a specified de-dupe.
