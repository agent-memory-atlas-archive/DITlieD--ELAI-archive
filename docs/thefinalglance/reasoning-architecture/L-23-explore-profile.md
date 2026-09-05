# `/reason --profile explore` conjecture-criticism map (L-23)

- **ELAI ids:** L-23 / L-T5 / EX-1..EX-4 / S2 / S3 / S8 / PS-3
- **Spec status (as designed):** PLANNED (WAVE 2; NEW surface; ~90% composition). Depends on L-22. S2 orbit-detector leg is PROOF_SPINE: none (discipline / waste-reduction; user-acked 2026-07-08) riding inside this PS-3 unit (`L-plan/L-23_reason_explore_profile_conjecture_criticism.md:1-12`, `L-plan/L-0_INDEX.md:121-122`).
- **Crate / module (spec):** `crates/orchestrator/src/reason/explore.rs`; `StagnationPattern::Orbiting(target)` in `anti_spiral_extended.rs`; `/reason --profile explore` (equivalently `elai explore`) on the B-39 profile seam
- **Purpose of this file:** enough architecture to rebuild the exploratory-reasoning **profile** — round loop, survivor map, gate-block-rate orbit detector, negative-case-law tabu. Not a status ledger.

## 1. What it is

**`/reason --profile explore`** is a **profile**, not a new engine (L-9 precedent on B-39). It is the one new **surface** in L-T5: a lane for the question class “hard open explanatory question → a **MAP** of rival explanations with their refutation status + deciding-evidence gaps.” B-39 REFLECT refines **one** answer; L-9 grounds **one** token-perfect research answer; the idea-framer critic loop attacks **one** framing. Explore composes already-planned pieces: L-22 VS conjecture spread + claimant falsifiers, anti-spiral stance rotation, I-8/I-26 problem turnover, D-13 + I-24/N-15 cross-family survivor judging, K-11/N-7 render — all journaled (SSB) and replayable (RRA) (`L-plan/L-23_reason_explore_profile_conjecture_criticism.md:5-7`, `ELAI-FEATURE-MAP.md:5110-5116`, `ELAI-SPEC-OVERVIEW.md:380-386`).

This file also lands two **general** anti-spiral upgrades whose first consumer is explore: **S2** `StagnationPattern::Orbiting(target)` as a 5th pattern from a **gate-block RATE** (zero embeddings), and **S3** negative case law **at the registration gate, never in the prompt** (`:7,35-36`, `L-plan/L-0_INDEX.md:70-71`).

HARD BOUNDARY: compose `/reason`, personas, D-13, I-8, and the local-serving stack — do **not** re-implement them (GD-1). Do **not** adopt DeepReason’s Dung argument-graph adjudicator (REJECTED steal §4.3 — ELAI verdicts are execution-grounded) (`:7,58,86`).

This is a **runtime** reasoning organ. The improvement-conjecture **planning** tool is not this profile: no FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, or bridge from that archive into L-23/L-24 (`ELAI-SPEC-OVERVIEW.md:393-399`).

## 2. Why it exists

ELAI had no lane for open explanatory questions whose honest output is **rivals + kill-conditions**, not a winner. DeepReason’s measured findings (n=1, DESIGN-RATIONALE / ID-5) say what works is **structural**, not prompt-content: stance rotation and problem turnover beat prompt tricks; “avoid the likely continuation” complement directives are a **placebo** (they move output relative to the **prompt**, not out of the model’s **repertoire**) — that is S3. His clean orbit detector is a **zero-embedding** counter: gate blocks per window separated healthy from refuted-attractor-orbiting (0 vs 54 blocks) where an embedding detector could not (medians 0.645 vs 0.671) — that is S2 (`L-plan/L-23_reason_explore_profile_conjecture_criticism.md:15`).

The differentiator is the **local-7B conjecturer**: grammar-masked skeleton (E-35/N-13) on a frozen-prefix / KV-reused pack (H-2/I-10/J-36). That is the offline-for-cents demo DeepReason could not ship, and the integration smoke for that whole substrate (`:7,15`, `L-plan/L-0_INDEX.md:73`). If this organ is missing, open questions collapse to B-39’s single answer. If it is faked (prompt “list alternatives”, embedding orbit, Dung graph, or banned ideas pasted into the conjecture prompt), the harness>model thesis is not being tested (`:58-61`).

Honest residue (GD-5): the harness catches **stalled / orbiting** dynamics, **not** wrong-but-stable shared-blind-spot consensus. Exogenous anchors (falsifier programs, retrieval, human ruling) are the only lever against that, named in-file (`:100`).

## 3. How it works (runtime)

1. **Who calls it.** Production surfaces only (R7): `elai reason --profile explore "<hard open question>"` and B-39 `/reason` profile dispatch → `explore.rs` (`:54-56`). The model does not select the profile or close the round.
2. **Input.** A hard open explanatory question. FIXTURE arm: recorded conjecture rounds. Live-7B arm: sidecar-healthy local 7B (`:9,39`).
3. **Round loop (profile state machine, host-owned):**

```
CONJECTURE
  L-22 propose_distribution(K) → K falsifier-carrying skeletons
    → FREE-CRITICISM
         run compiled Predicate/Program falsifiers (D-11, zero LLM)
    → STANCE ROTATION on stagnation / Orbiting
         anti-spiral persona switch (EX-1 structural; names the attractor)
    → PROBLEM TURNOVER on a dry attractor
         I-26 plateau collapse / I-8 voluntary abort (never-loop-a-dry-problem)
    → CROSS-FAMILY JUDGE the SURVIVORS
         D-13 adversary + I-24/N-15 order-swap + verbosity-penalty
    → EMIT survivor map
```

(`:37`, `ELAI-FEATURE-MAP.md:5110-5113`)

4. **CONJECTURE (model generates; host constrains).** VS-shaped spread via L-22 `propose_distribution`. Each candidate is a falsifier-carrying skeleton. Local-7B: E-35 grammar mask so the skeleton is valid **by construction** (EX-4), not by template-following (`:37,39`).
5. **FREE-CRITICISM (deterministic, zero LLM).** Run compiled L-22 Predicate/Program falsifiers through D-11. A firing falsifier → Kind `RefutedByFalsifier`; that rival **drops** from the survivor set (`:37,105`). Rubric stays offline (L-22 boundary).
6. **S2 orbit detector (deterministic, general anti-spiral upgrade).** Windowed counter over **existing** signals: I-8 dead-end prunes + L-7 gate blocks + L-22 / anti-relapse refusals, **keyed by refuted-target**. `rate >= floor` → `StagnationPattern::Orbiting(target)`. The persona switch **names the attractor** it is escaping. **Zero embeddings.** Journal-recomputable (GD-2) (`:35`, `L-plan/L-0_INDEX.md:71`, `ELAI-FEATURE-MAP.md:5090-5094`).
7. **S3 tabu at the gate (deterministic).** Refuted / banned content is blocked at **registration** (I-8 hash + S2 orbit). It is **never** rendered into a conjecture prompt (priming grows unbounded). I-5 contrastive constraint stays **only** for typed error-avoidance (failure class + location), never as a diversity instruction. Diversity levers: stance rotation, problem turnover, L-22 tail-weighted VS. A guard: a diversity directive is **never assembled from gate/tabu content** (`:36,59`, `L-plan/L-0_INDEX.md:70`).
8. **Stance rotation / problem turnover (compose, do not re-author).** On `Orbiting` / other stagnation: C-10 persona affinity plus the named attractor. On a dry attractor: I-26 abort / turnover (`:37,49`). Complement “be different” is **banned** (`:59`).
9. **Cross-family judge of survivors (not of the generator).** D-13 foreign-reviewer + I-24/N-15 order-swap and verbosity-penalty **before** a survivor verdict. Same-family critique is D-13’s ban, not relaxed here. Retention scoring is **Pareto over (hard-to-vary, reach, coverage)** and **advisory** (TD-3); the model never adjudicates (`:37,50,85`).
10. **Survivor-map artifact (EX-3).** `{ surviving explanations, refuted explanations + the falsifier that killed each, deciding-evidence gaps between survivors }`. Emitted as an SSB round + rendered via K-11/N-7. **Never** a single collapsed answer (`:38,64,83`).
11. **Local-lane arm (W-10 tiered).** Conjecturer = local 7B; E-35 mask; H-2/I-10 frozen-prefix pack; J-36 head-KV reuse (re-decode **only the tail** per cycle). FIXTURE arm ships now. Live-7B **refuses** without the sidecar-protocol health check (`:39,62,105`).
12. **Commit.** `ExploreRoundCompleted` Kind `SurvivorRetained` / `RefutedByFalsifier` / `OrbitDetected` / `ProblemTurnedOver`. Reuse `anti_spiral_hit` for the Orbiting rotation (`L-plan/L-0_INDEX.md:145`, `:67`).
13. **Abstain / fail-closed.** Banned idea in the conjecture prompt → **FAIL** (S3). Single collapsed answer → **FAIL** (EX-3). Embedding orbit detector → **FAIL** (S2). Dung/argument-graph symbol → **FAIL**. Live-7B without sidecar health → **refuse**. Observe-first before any explore verdict **acts** (GD-4) (`:68,87`).
14. **The model may generate** conjecture skeletons, falsifier bodies, and advisory retention prose. It may **not** pick survivors, write `Orbiting`, assemble tabu into the prompt, collapse the map to one answer, or run the live-7B arm around the health check.

## 4. Types, enums, objects

Copy from `L-plan/L-23_reason_explore_profile_conjecture_criticism.md:35-38,64` and C-10’s `StagnationPattern` this organ **extends**. Do not invent fields.

```
// C-10 four patterns PLUS L-23 fifth:
enum StagnationPattern {
  Spinning,
  Oscillation,
  NoDrift,
  DiminishingReturns,
  Orbiting(target),                 // L-23 S2 ADD; target = refuted-attractor key
}

// windowed rate over EXISTING signals, keyed by refuted-target
fn orbit_rate(window, target) -> u32
  // count: I-8 dead-end prunes + L-7 gate blocks + L-22/anti-relapse refusals
  // rate >= floor => Orbiting(target)
  // ZERO embeddings / cosine

ReasonProfile::Explore              // B-39 profile registration (name as grepped)

SurvivorMap {
  surviving: ...,                   // explanations that were not killed
  refuted:  ... + killing falsifier // each dead rival WITH the falsifier that killed it
  deciding_evidence_gaps: ...,      // what would decide among survivors
}

ExploreRoundCompleted.Kind {
  SurvivorRetained,
  RefutedByFalsifier,
  OrbitDetected,
  ProblemTurnedOver,
}

// retention (advisory, TD-3) — Pareto axes named, formula not specified:
//   hard-to-vary, reach, coverage
```

Do **not** add: `GroundedExtension`, Dung graph types, embedding similarity types, schools/lineage islands (`:58,123`).

C-10 live `StagnationPattern` is `{ Spinning, Oscillation, NoDrift, DiminishingReturns }` (`C-10-anti-spiral-lateral-thinking.md` §4). Rebuild **adds** `Orbiting(target)` to that enum; do not fork a second detector.

## 5. Control flow / state machine

**Owner of the loop:** `reason/explore.rs`. The model proposes skeletons; the host runs falsifiers, counts orbits, refuses tabu-in-prompt, emits the map. No token-level explore FSM. No FDL / ProbeUtility scheduler (`ELAI-SPEC-OVERVIEW.md:393-399`).

**Forced transitions:**
- Falsifier fires → rival **out** of survivor set (`RefutedByFalsifier`) (`:105`).
- Re-propose refuted-equivalent **≥ floor** → `Orbiting(target)` + persona rotation naming the attractor (`:68,105`).
- Dry attractor → I-26 problem turnover (`ProblemTurnedOver`) (`:37`).
- Tabu content in prompt assembly → **hard fail**, not a warning (`:59,68`).
- Live-7B without sidecar health → **do not run** (`:62`).
- Profile is **opt-in by invocation**; no ENABLE flag. Local arm behind a **health gate**, not an ENABLE flag (`:92`).

**Decision table (must be unit-tested):**

| Condition | Effect |
|---|---|
| predicate falsifier fires | drop from survivors; Kind RefutedByFalsifier |
| re-propose refuted-equivalent ≥ floor | OrbitDetected + named-attractor persona rotation |
| orbit counter implementation | gate-block RATE only; `rg embedding\|cosine` = 0 |
| deliverable | multi-rival map; single answer **illegal** |
| prompt assembly | never contains a refuted-idea string |
| survivor verdict | I-24/N-15 order-swap + verbosity-penalty **first** |
| FIXTURE replay | byte-identical under `ELAI_SEED` |
| Dung / argument_graph / grounded_extension | absent (`rg` = 0) |

(`:105`)

**Persona on Orbiting:** C-10 affinity table does not name Orbiting. Spec says the switch **names the attractor** (`:35`). Which of the five personas is default for Orbiting is **UNCERTAIN** (§16); the named-attractor requirement is load-bearing.

## 6. Grounding, receipts, gates

**Selector hierarchy for a rival’s status:** compiled L-22 Predicate/Program firing (execution) > D-13 cross-family structured `CounterEvidence` (auditor-owned) > I-8 hash relapse > advisory Pareto retention. LLM “this one is best” is **never** a status. Embedding cosine is **never** an orbit or equivalence signal.

DeepReason §11 anti-relapse chain was `hash → embedding-neighbor → battery-equivalence`. ELAI takes **hash + S2 rate** and **rejects** the embedding neighbor (`:30,60`). Do not “complete” the DeepReason chain.

**Never allowed as a confidence / survivor signal:** embedding orbit, Dung grounded extensions, complement “be different”, typicality as adjudication (L-22 already advisory), same-family D-13, model-set retention score, DeepReason’s 0-vs-54 / 0.973 figures as ELAI gates (ID-5) (`:58-61,101`).

**Registration gate is the tabu store.** Prompt is not a memory of what was refuted (EX-1). L-24 later feeds the **refuted set to this gate**, never to the prompt (`L-plan/L-SUMMARY_INDEX.md:258`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| profile invocation | opt-in CLI / `/reason --profile explore` | per call |
| enable flag | **none** (W-2) | organ |
| orbit-rate floor | unspecified numeric; seed conservatively; OBSERVE-first; measure at L-10 before it **acts** | per window (`:121`) |
| DeepReason 0 vs 54 / 4.3× token burn | DESIGN-RATIONALE only | not an ELAI constant (`:15,101`) |
| VS K | L-22’s `propose_distribution` (K unspecified there) | per CONJECTURE step |
| free-criticism | zero LLM; D-11 timeouts | per compiled falsifier |
| J-36 | re-decode **tail only** per cycle | local-7B arm (`:39`) |
| FIXTURE vs live-7B | FIXTURE = fail-closed in-process done-gate; live = sidecar/RunPod (W-10) | (`:9,95`) |
| schema | `+ExploreRoundCompleted` net +1; authoring-era ~268/577; re-read at edit | `L-plan/L-0_INDEX.md:145` |
| LOC budget (plan) | anti_spiral ~50, explore.rs ~180, CLI ~25, I-5 doctrine ~10, contract ~110 | (`:107-112`) |
| schools/lineage | **DEFERRED**; do not build here | (`:123`) |

**Overflow:** I-26 85% / plateau collapse still bounds search-shaped spend if explore rides I-4; L-23 does not replace I-26 (`:21,49`). Unspecified: max rivals per round, max rounds per invocation (§16).

## 8. Roles and neighbors

**Consumes (compose, never re-implement):**
- L-22 `Falsifier` + `propose_distribution` (`:48`).
- C-10 personas + I-8 dead-end hash + I-26 abort (`:49`).
- D-13 adversary + J-45 + I-24/N-15 (`:50`).
- E-35 / N-13 / H-2 / I-10 / J-36 (`:51`).
- K-11 / N-7 render; SSB; RRA (`:52`).
- B-39 `/reason` profile registration (`:10,55`).
- L-7 gate blocks as an S2 **signal** (L-7 is not re-authored here) (`:35`).

**Produces:** `StagnationPattern::Orbiting`; `SurvivorMap`; `ExploreRoundCompleted`; CLI/profile `Explore`.

**Must not fork:** a second `/reason` engine; a Dung adjudicator; an embedding orbit detector; a prompt-side tabu list; an FDL / ProbeUtility / operator-allocator cousin; a bridge from the planning-harness improvement-conjecture archive (`ELAI-SPEC-OVERVIEW.md:393-399`).

**Parallel-safe with N-16** (disjoint files) (`:11`). L-24 is the memory consumer of this pack; L-10 explore-arm measures survivor-map quality/cost (`:127`).

**Local-lane D-13 residue:** with only one local 7B, foreign-reviewer needs a **second family**. Decide at edit: cloud second family **or** FIXTURE recorded cross-family verdicts until a 2×-family local setup exists (`:122`). Do not silently use same-family Qwen (D-13).

## 9. Config / flags / defaults

- No `ELAI_EXPLORE_ENABLE`. Opt-in by invocation (`:92`).
- Live-7B: sidecar-protocol **3-step health check** first; never an ENABLE flag (`:32,62,92`).
- Orbit floor: conservative seed + observe-first (GD-4) until L-10 measures (`:121`).
- S3 doctrine paragraph on the I-5 / context path (`:110`).
- Decision doc: `workflow/reasoning/L-23-decisions.md` (`:40,112`).
- No new dependency (`:88`). blake3; no `.unwrap()` / `.expect()` on prod (`:62`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/anti_spiral_extended.rs` | `Orbiting(target)` + gate-block-rate counter (`:44,108`) |
| `.elai_cc/crates/orchestrator/src/reason/explore.rs` | round loop + survivor-map builder + local-lane arm (`:45,109`) |
| B-39 `/reason` profile registration + CLI verb | `ReasonProfile::Explore` / `elai reason --profile explore` (`:46,110`) |
| I-5 / context doctrine | S3 tabu-at-gate-never-in-prompt paragraph (`:110`) |
| `crates/contracts/tests/l23_explore_profile_contract.rs` | named contract (`:111,105`) |
| `workflow/reasoning/L-23-decisions.md` | required (`:112`) |

`rg` greps that are load-bearing (`:73-77`): `Orbiting` present; `profile.*explore|ReasonProfile::Explore` present; `embedding|cosine` count **0** in the orbit counter; `Dung|grounded_extension|argument_graph` count **0** in `explore.rs`.

## 11. Journal / SSB events

From `L-plan/L-0_INDEX.md:145` and `:67`:

```
ExploreRoundCompleted {  // Pattern-2; net +1; baseline re-read (W-9)
  Kind: SurvivorRetained | RefutedByFalsifier | OrbitDetected | ProblemTurnedOver
}

// reuse for the Orbiting rotation (do not mint a second stagnation type):
anti_spiral_hit
```

Every round lands in SSB + RRA byte-replay (`:5,99`). Payload fields beyond Kind are unspecified (§16). Do not rewind a live schema number (authoring-era ~268/577).

## 12. Tests and observables

Named contract `l23_explore_profile_contract.rs` (`:105`):

| Id | Assert |
|---|---|
| (a) | rival with firing predicate falsifier → `RefutedByFalsifier`, dropped from survivors |
| (b) | re-propose refuted-equivalent ≥ floor → `Orbiting(target)` + persona rotation naming the attractor |
| (c) | orbit counter: **zero** embeddings (no cosine/embedding call) |
| (d) | deliverable is a **multi-survivor map**, not a single answer (EX-3) |
| (e) | diversity directive never assembled from gate/tabu; prompt never contains a refuted-idea string (S3/EX-1) |
| (f) | cross-family judge runs order-swap + verbosity-penalty **before** a survivor verdict |
| (g) | FIXTURE arm replays **byte-identically** under `ELAI_SEED` (GD-2/RRA) |
| (h) | Dung/argument-graph symbol **absent**; live-7B arm **refuses** without sidecar health |

**Observable effect** (`:65-68`): explore on a hard open question with ≥2 rival mechanisms, one of which carries a firing predicate falsifier (FIXTURE now) → survivor map lists survivors, each refuted rival **with** its killing falsifier, and the deciding-evidence gap; re-propose ≥ floor → `OrbitDetected` + named rotation.

**Negatives:** banned/refuted idea back in the conjecture prompt **FAILS**; single collapsed answer **FAILS**; embedding orbit detector **FAILS**.

**Done-gates:** W-1 live `elai reason --profile explore` emits survivor-map SSB + artifact; EX-1..EX-4; TD-3; GD-1 (Dung not adopted); GD-4/W-10 FIXTURE now / live sidecar-gated; `cargo check --workspace` GREEN; auditor PASS `-p orchestrator` + CLI crate (`:80-88,90-101`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A `/reason --profile explore` (or equivalent) **profile** on the existing reason surface, not a new engine.
- Deliverable is a **survivor MAP**: survivors + each refuted rival **with** its killing falsifier + deciding-evidence gaps. A single collapsed answer is illegal (EX-3).
- Round loop order: CONJECTURE (L-22 VS) → free-criticism (compiled falsifiers, zero LLM) → stance rotation on orbit/stagnation → problem turnover on dry attractor → cross-family judge of **survivors** → emit map.
- `StagnationPattern::Orbiting(target)` from a **windowed gate-block RATE** over I-8 prunes + L-7 blocks + anti-relapse refusals. **Zero embeddings.**
- Tabu at **registration** (I-8 hash + orbit). **No** refuted/banned text in the conjecture prompt. **No** complement “be different” diversity lever (EX-1).
- Local-7B conjecturer skeleton valid **by construction** (E-35 mask + frozen-prefix + J-36 tail decode) (EX-4). FIXTURE arm in-process; live arm health-gated.
- Dung / grounded_extension / argument_graph **absent**. Model does not adjudicate survivors (TD-3).
- `ExploreRoundCompleted` journaled; FIXTURE byte-replay under `ELAI_SEED`.
- No FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, or planning-archive bridge.

A prompt “give several theories”, an embedding clusterer, or a Dung graph scorer is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Dung argument-graph adjudicator (REJECTED steal §4.3) (`:58`).
- Embedding / cosine orbit detector (S2) (`:60`).
- Rendering refuted/banned content into prompts; complement “be different” (S3/EX-1) (`:59`).
- Model-adjudicated survivor / retention score (TD-3) (`:61`).
- Re-implementation of `/reason`, personas, D-13, I-8, or the serving stack (GD-1) (`:61`).
- Schools / lineage islands / reseed (DEFERRED steal §4.3) (`:123`).
- Untyped-artifact ontology (`ELAI-FEATURE-MAP.md:3662-3664`).
- FDL frontier controller, ProbeUtility scheduler, multi-cycle operator allocator, planning-harness → L-23 bridge (`ELAI-SPEC-OVERVIEW.md:393-399`).
- Using DeepReason numeric findings as ELAI floors (ID-5) (`:101`).
- Live-7B without sidecar health check (`:62`).
- Acting on orbit/survivor verdicts before observe-first / L-10 measure (GD-4) (`:87,121`).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-23_reason_explore_profile_conjecture_criticism.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:247-252`. Doctrine: `L-plan/L-0_INDEX.md:70-73,121-122,145`. Overview: `ELAI-SPEC-OVERVIEW.md:380-399`. Feature map: `ELAI-FEATURE-MAP.md:3653-3666,5090-5116`. Steal artifact (design-rationale): `workflow/reasoning/steal-deepreason-2026-07-08.md` S8/S2/S3/§3.1/§3.2. Neighbors in this pack: `L-22-claimant-falsifiers.md`, `C-10-anti-spiral-lateral-thinking.md`, `B-39-chat-reflect-pipeline.md`, `D-13-adversary-role-cross-family.md`, `I-26-dynamic-thinking-budgets.md`.

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** L-23 atom block. Do not mint L23-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Numeric orbit-rate floor and window length (`:121`).
- Default C-10 persona for `Orbiting` (spec requires naming the attractor, not a persona table row) (`:35` vs `C-10-anti-spiral-lateral-thinking.md` §3 affinity table).
- Local-arm cross-family: cloud D-13 vs FIXTURE-recorded verdicts (`:122`).
- `SurvivorMap` rust field types; Pareto (hard-to-vary, reach, coverage) formula / weights (`:37-38`).
- Max rivals per round, max rounds per invocation, halt condition besides I-26/I-8.
- `target` key type for `Orbiting(target)` (hash? claim-id? string?).
- `ExploreRoundCompleted` payload fields beyond Kind.
- Exact B-39 registration symbol (`ReasonProfile::Explore` is the grep, not a cited existing enum) (`:74`).
- Whether `elai explore` is an alias or a second verb (`:5,46`).
- How “refuted-equivalent” is decided **without** embeddings (hash only vs L-4/F-12 NLI — L-24 novelty column is NLI, but L-23 relapse is I-8 hash) (`:30,35` vs L-24 NLI).
- Q-10 research-coupling packet: overview forbids a planning-archive bridge; a later one-way `SOURCE_ONLY` packet “may inspire a transfer” only after locked gates (`ELAI-SPEC-OVERVIEW.md:401-408`). L-23 must not grow that interface.
