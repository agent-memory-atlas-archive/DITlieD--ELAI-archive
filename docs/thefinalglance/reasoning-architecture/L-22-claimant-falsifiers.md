# Claimant-authored falsifiers + demarcation + Verbalized-Sampling propose (L-22)

- **ELAI ids:** L-22 / L-T5 / HG-5 / EX-4 / S1 / S5 / PS-3
- **Spec status (as designed):** PLANNED (WAVE 1; BUILD-on-shipped + EXTENSION). Depends on L-1 ClaimSet and L-2 ClaimRisk landing first (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:1-11`, `L-plan/L-0_INDEX.md:119-120`).
- **Crate / module (spec):** `crates/orchestrator/src/claim/falsifier.rs`; `crates/orchestrator/src/claim/verbalized_sampling.rs`; field add on `claim/mod.rs`; fuse signal in `claim/risk.rs`; generation-seam elicitation hook
- **Purpose of this file:** enough architecture to rebuild the generation-time attack surface of the causal grounding gate — claimant falsifiers, the HG-5 demarcation floor, the GRC no-falsifier-no-Converge gate, and the Verbalized-Sampling propose contract. Not a status ledger.

## 1. What it is

The **claimant-authored falsifier organ** inverts claim authorship. After L-1 extracts claims off-agent and L-2 scores them, L-22 requires the **generator** to emit, **with** every design-claim / hypothesis / diagnosis, the machine-checkable condition that would **refute** it (DeepReason skeleton `forbidden[]`). The harness compiles that falsifier into a D-11 `CheckerRef` and the checker runs at **zero LLM cost**. Three load-bearing contracts sit on that field: (a) optional `falsifier` on the L-1 `Claim`, grammar-enforced at generation time; (b) a **demarcation fuse** into L-2 so a Causal/Behavior/Reachability claim that forbids nothing scores a High risk-floor (HG-5); (c) **no-falsifier-no-Converge** on hypothesis-class GRC-v3 steps. A fourth contract is the **Verbalized-Sampling propose** path: one distribution call of K typicality-tagged candidates instead of I-4’s N resamples, with tail-spend **only** under the anti-spiral/orbit flag (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:5-6`, `ELAI-FEATURE-MAP.md:5078-5088`, `L-plan/L-0_INDEX.md:54`).

HARD BOUNDARY: L-22 **elicits + compiles + tiers**. It does **not** run checkers (D-11 owns execution), does **not** set a tier by model judgment (TD-3), and typicality is **advisory only**, never an adjudication (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:5-6`).

This is a **runtime reasoning organ**. It is not the planning-harness improvement-conjecture tool, and it does not grow an FDL frontier controller (`ELAI-SPEC-OVERVIEW.md:393-399`).

## 2. Why it exists

ELAI already blocks claims without a measurable procedure at the **plan** grain (EXPECTED_OBSERVATION / ON_FAIL, D-12) and round-trips claims against its own checkers (L-1/L-3). It never asks the **generator** to name its own refuter (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:14`). DeepReason’s measured move: claimant-authored predicate falsifiers were the one criticism class that was **cost-positive** at low base error — the model naming the check is cheaper than a separate critic reconstructing it, and a forbids-nothing candidate is caught by a **program**, never a paid critic (`:5,14`).

If this organ is missing, Causal/Behavior claims can Converge with an empty attack surface, and the CGG stays verify-time only. If it is faked (a second free-form call “please invent a falsifier”, or the model setting its own tier), Popperian demarcation is prompt theater (`:5,55-57`). Verbalized Sampling restores pretraining diversity that RLHF mode-collapses via **prompt shape**, not “be different” content — so it composes with L-23’s S3 rule that complement directives are a diversity placebo (`:14`, `L-plan/L-0_INDEX.md:70`).

Every DeepReason / VS-paper number is **DESIGN-RATIONALE only** (ID-5). ELAI re-measures falsifier-hit and diversity deltas at L-10 (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:14,93`).

## 3. How it works (runtime)

1. **Who calls it.** The orchestrator **generation seam** (production intake, not a `#[cfg(test)]` stand-in — R7) when a step emits a design-claim / hypothesis / diagnosis. The L-2 fuse reads the compiled field. J-83’s Converge transition consults it on hypothesis-class steps (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:34-37,50-53`). The model does **not** call `compile` or `score`.
2. **Input.** A model output that is about to become an L-1 `Claim` (kinds that can carry a falsifier), plus the current anti-spiral/orbit flag for the VS tail selector (`:25-26,34-38`).
3. **Elicit (one masked decode).** At generation time, elicit `falsifier` through the N-13 OutputBoundary resolver + E-35 grammar mask so the field is valid **by construction** on any B-37 backend (HG-3; EX-4). Weak lane pays **zero** adherence retries. **Never** a second free-form model call to reconstruct the falsifier — it is emitted **WITH** the claim in one masked decode (`:34-35,55`, `L-plan/L-0_INDEX.md:73`).
4. **Compile (deterministic, zero LLM).** `FalsifierKind::Predicate` and `Program` compile to a D-11 `CheckerRef` at registration. The predicate is **untrusted input**: it MUST compile through D-11’s existing sandbox, never `eval`’d raw (`:33,115`). `FalsifierKind::Rubric` is **deferred** — judged offline by I-24/N-15, never in the hot loop (`:33,57`).
5. **Execute the compiled check (D-11, not this organ).** D-11 runs the `CheckerRef`. L-22 consumes Pass/Fail for the fuse. L-22 does not re-implement the sandbox (`:5-6,47`).
6. **Demarcation fuse into ClaimRisk (deterministic, TD-3).**
   - Causal / Behavior / Reachability with `falsifier = None` (forbids nothing) → **risk-floor High** (HG-5 empty attack surface) (`:36`, `L-plan/L-0_INDEX.md:54`).
   - Claim carrying a compiled Predicate/Program that **PASSES** its own check → scored at its **evidenced** L-2 tier (no automatic High) (`:36`).
   - Rubric-only falsifier → `causal_supported` stays **false** (HG-1: rubric is not interventional) (`:36,77`).
   The model never sets or overrides the tier (`:56`).
7. **GRC-v3 no-falsifier-no-Converge (deterministic).** A hypothesis-class GRC step (debug fork, architecture choice) may not transition to `Converge` unless its J-83 BeliefState entry carries a declared falsifier (a `RECON_NEEDED` check **or** a compiled predicate). Missing → step stays open / routes to L-6 targeted effort (HG-4). This is plan-standard `RECON_NEEDED` made a runtime reasoning gate (`:37`, `L-plan/L-0_INDEX.md:54`).
8. **Verbalized-Sampling propose (one distribution call).** `propose_distribution(prompt, k)` elicits K candidates each with a self-assessed typicality in **one** masked decode. Replaces an I-4 N-resample **for the propose leg only**. Distribution-level controls: max-weight cap + tail-emphasis knob (`:38`, `ELAI-FEATURE-MAP.md:5085-5088`).
9. **Tail-weighted selector (advisory, flag-gated).** Typicality **steers criticism budget only when** the L-23 S2 orbit / anti-spiral flag is up, spending on the atypical tail. It **never** adjudicates a tier or a selection (TD-3). Cold-start / flag-down → byte-identical to I-4’s existing best-of-N (GD-1, no regression) (`:38,56,96`).
10. **Commit.** Emit `FalsifierAuthored` (or the Kind on a reused L-2 row). Prefer-reuse `ClaimRiskScored` for the demarcation tier. Prefer-reuse I-4 candidate rows for the VS spread, else `VerbalizedDistributionDrawn` (`L-plan/L-0_INDEX.md:144`, `L-plan/L-22_claimant_authored_falsifiers_demarcation.md:63-64`).
11. **Abstain / fail-closed / observe-only.** Ships **OBSERVE-only** (records falsifier state + demarcation) until L-7’s gate flag **acts** on it (GD-4, W-2: elicitation is always-on intake; the **acting** kill-switch is L-7) (`:81,86-87`). Forbids-nothing below High **fails**. Rubric-only with `causal_supported=true` **fails**. Reconstruction-pass **fails**.
12. **The model may generate** claim text, `Falsifier.body`, and advisory typicality tags. It may **not** compile the predicate, choose D-11, set `RiskTier`, set `causal_supported`, force Converge, or pick the VS winner.

## 4. Types, enums, objects

Copy from `L-plan/L-22_claimant_authored_falsifiers_demarcation.md:33-38` and L-1/L-2 objects this organ **extends**. Do not invent fields.

```
// L-1 Claim — ADD a field; never re-type (GD-1)
Claim {
  kind: ClaimKind,                    // L-1: File, Symbol, Dependency, Api,
                                      //      Behavior, Causal, Reachability
  span: ByteSpan,
  refs: Vec<ClaimRef>,
  taint: Option<Provenance>,          // G-1
  falsifier: Option<Falsifier>,       // L-22 ADD
}

Falsifier {
  kind: FalsifierKind,                // Predicate | Program | Rubric
  body: String,
  compiled: Option<CheckerRef>,       // D-11; None until compile, always None for Rubric
}

enum FalsifierKind { Predicate, Program, Rubric }

CheckerRef                            // D-11 registry handle; L-22 does not define the checker

// L-2 ClaimRisk — fuse a signal, never re-fuse
ClaimRisk {
  tier: RiskTier,                     // Low | Medium | High | Critical
  signals: Vec<PredictorSignal>,
  causal_supported: bool,             // ONLY interventional Predicate/Program pass
                                      //   or L-5 do-test; Rubric never sets true
}

enum RiskTier { Low, Medium, High, Critical }

fn elicit_falsifier(...) -> Claim     // one masked decode via N-13 + E-35
fn compile_to_d11(f: &Falsifier) -> Result<CheckerRef, ...>
fn demarcation_signal(claim: &Claim, check: Option<CheckerResult>) -> RiskFloor | Evidenced
fn propose_distribution(prompt, k) -> Vec<TypicalityTaggedCandidate>
fn tail_weighted_select(cands, orbit_flag) -> ...   // fires ONLY if orbit_flag
```

**HG-5 required-kind seed (observe-first):** Causal / Behavior / Reachability **require** a falsifier for the High floor. File / Symbol / Dependency are already L-1/L-3 round-trip-checkable; falsifier is **optional** there. Exact required-kind set is an authoring decision under GD-4 (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:114`).

**DeepReason compiled-commitment types (design-rationale, not ELAI enums):** `predicate:` / `program:` / `rubric:` map onto `FalsifierKind`. Do not import the skeleton `{claim, mechanism, scope, forbidden[]}` as a second claim type (`:28`, `ELAI-FEATURE-MAP.md:5078-5084`).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator generation seam + L-2 fuse + J-83 Converge gate. The model proposes claim + falsifier body; the engine compiles, floors, and refuses Converge. No token-level FSM.

**Falsifier lifecycle:**

| Stage | Owner | Accept | Reject / defer |
|---|---|---|---|
| elicit (one decode) | generation seam + N-13/E-35 | well-formed field by grammar | reconstruction second call **banned** |
| compile | L-22 → D-11 sandbox | Predicate/Program → `CheckerRef` | raw eval **banned**; Rubric never compiles to hot-loop checker |
| run | D-11 | Pass/Fail/Inapplicable | L-22 does not run |
| fuse | L-2 + `demarcation_signal` | High floor **or** evidenced tier | model-set tier **banned** |
| GRC Converge | J-83 hypothesis-class | declared falsifier present | missing → stay open / L-6 |
| VS propose | `propose_distribution` | K tagged cands, one call | tail-spend without orbit flag **banned** |

**Forced transitions:**
- Hypothesis-class GRC step **without** falsifier → **not** `Converge` (`:37`).
- Forbids-nothing Causal/Behavior/Reachability → High **floor**, not a suggestion (`:36`).
- Orbit flag **down** → VS path **byte-identical** to I-4 best-of-N (`:38,96`).
- Rubric → offline judge only; `causal_supported` remains false (`:33,36`).

**HG-2 interaction (floor, not a second fuse):** L-2 High/Critical from **predictors** still needs ≥2 mechanistically-distinct predictors (`L-plan/L-2_claimrisk_score_convergence_fuse.md:5,66`). HG-5 is specified as a **risk-floor** on empty attack surface, not as PredictorKind #6. Whether the floor **bypasses** HG-2 is **UNCERTAIN** (§16); rebuild the floor as specified, do not invent a Demarcation predictor just to satisfy HG-2.

## 6. Grounding, receipts, gates

**Selector hierarchy for this organ:** compiled Predicate/Program Pass (D-11 receipt) > evidenced L-2 predictors > HG-5 High floor on empty attack surface > Rubric (offline, never causal). LLM typicality is **not** in this hierarchy.

**Receipts:**
- Compile success → `FalsifierAuthored` Kind `PredicateCompiled` / `ProgramCompiled`.
- Rubric → `RubricDeferred` (not a hot-loop pass).
- Missing falsifier on a required kind → `ForbidsNothing`.
- Checker run → D-11 `CheckerPass` / `CheckerFail` / `CheckerInapplicable` (neighbor organ).
- Demarcation tier → prefer-reuse `ClaimRiskScored`.

**Never allowed as a confidence signal / tier chooser:** LLM self-rating of the falsifier, typicality as a gate, “the model said it forbids X”, rubric as `causal_supported`, a reconstructed falsifier from a critic call, DeepReason’s or VS-paper’s published numbers (ID-5) (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:5-6,56-57,93`).

**Sandbox is the safety gate.** Claimant-authored predicates are untrusted. DeepReason `checks.py` AST-guard is the **weak** version; ELAI compiles through D-11’s existing sandbox (`:115`).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| falsifier elicitation | **one** masked decode with the claim | per claim |
| reconstruction calls | **0** | per claim (`rg` must be 0) |
| D-11 compile+run | zero LLM; D-11 timeout owns wall time | per compiled falsifier |
| Rubric in hot loop | **0** | always deferred |
| VS propose | **one** distribution call of K candidates | per propose leg (replaces I-4 N calls) |
| VS tail-spend | **only** when orbit/stagnation flag is up | per round |
| VS flag-down | byte-identical I-4 best-of-N | GD-1 no regression |
| max-weight cap / tail-emphasis | VS paper distribution-level controls | **DESIGN-RATIONALE**; ELAI-measured at L-10 (`:14,38,93`) |
| numeric K | unspecified in L-22 | **UNCERTAIN** (§16); I-4 live default width is 4 (`I-04-evidence-ranked-search-ladder.md` BEST_OF_N_DEFAULT_WIDTH) — do not silently copy |
| acting on High floor | observe-only until L-7 | GD-4 |
| schema | `+FalsifierAuthored` net +1..+2; baseline re-read at edit (W-9); authoring-era ~268/577 | `L-plan/L-0_INDEX.md:129,144` |
| LOC budget (plan) | falsifier.rs ~120, verbalized_sampling.rs ~80, field +10, fuse +20, seam +35, contract ~90 | `L-plan/L-22_claimant_authored_falsifiers_demarcation.md:99-105` |

**Overflow:** grammar-mask fail-closed via N-13 (strongest-available-enforcement: provider-strict > native-tool > local-CFG > regex > parse-retry). Weak lane must not pay adherence retries (EX-4). Unspecified: truncate-vs-reject on `Falsifier.body` length (§16).

## 8. Roles and neighbors

**Consumes (extend, never re-implement — GD-1):**
- L-1 `ClaimSet` / `Claim` — ADD `falsifier` field (`:46`).
- L-2 `ClaimRisk` — fuse demarcation signal (`:46`).
- D-11 checker registry — compile + run the predicate (`:47`).
- E-35 `DecodingMode` + `sidecar_xgrammar_hook.rs` — grammar mask (`:47`).
- N-13 OutputBoundary resolver — strongest-available enforcement (`:47`).
- I-4 candidate ladder — propose leg VS **replaces** (`:48`).
- J-83 BeliefState — GRC step carrier for no-falsifier-no-Converge (`:48`).
- `anti_spiral_extended.rs` — orbit/stagnation flag (L-23 S2 consumes/extends the same flag) (`:48`).

**Produces:** `Falsifier` on claims; `CheckerRef` at registration; demarcation floor into L-2; Converge refusal; VS distribution; `FalsifierAuthored` SSB.

**First heavy consumer:** L-23 `/reason --profile explore` runs the falsifier-carrying class + VS spread (`:119`). L-24 stores unrun L-22 falsifiers as declared missing tests (`L-plan/L-SUMMARY_INDEX.md:257`).

**Must not fork:** a second Claim type for “hypotheses”; a critic that reconstructs falsifiers; a typicality-gated selector; Dung argument-graph (L-23 rejection, do not sneak it in here); the planning-harness improvement-conjecture archive (`ELAI-SPEC-OVERVIEW.md:393-399`).

**Parallel-safe with L-23** (disjoint files; L-23 consumes this class) (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:11`).

## 9. Config / flags / defaults

- **No enable-flag for elicitation.** Always-on intake (W-2). The **acting** gate is L-7’s kill-switch (`:86-87`).
- GD-4: observe-only until L-7 acts (`:81`).
- VS tail path: gated by anti-spiral / L-23 `Orbiting` flag; flag-down = I-4 path (`:38`).
- D-11 remains behind `ELAI_CHECKER_ENABLE` in live boot (`D-11-constructive-checker-registry.md` header). L-22 **compile** still targets that registry; a dark D-11 cannot silently skip the sandbox — fail-closed / Inapplicable is D-11’s rule, not a license to `eval`.
- Decision doc required: `workflow/reasoning/L-22-decisions.md` (`:39,103`).
- No new dependency (`:82`). blake3 not sha2; no `.unwrap()` / `.expect()` on prod (`:58`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `.elai_cc/crates/orchestrator/src/claim/falsifier.rs` | `Falsifier` / `FalsifierKind` + compile-to-D-11 + demarcation fuse (`:41,99`) |
| `.elai_cc/crates/orchestrator/src/claim/verbalized_sampling.rs` | `propose_distribution` + tail-weighted-under-stagnation selector (`:43,100`) |
| `.elai_cc/crates/orchestrator/src/claim/mod.rs` | optional `falsifier` field on `Claim` (~10 LOC) (`:101`) |
| `.elai_cc/crates/orchestrator/src/claim/risk.rs` | demarcation fuse signal (~20 LOC) (`:102`) |
| generation seam | grammar-enforced elicitation + GRC no-falsifier-no-Converge (~35 LOC) (`:103`) |
| `crates/contracts/tests/l22_falsifier_demarcation_contract.rs` | named contract (`:104,96`) |
| `workflow/reasoning/L-22-decisions.md` | required (`:105`) |
| D-11 registry + sandbox | compile/run neighbor (`:47`) |
| `decoding.rs:229` `DecodingMode` + `sidecar_xgrammar_hook.rs` | E-35 mask (`:10,25`) |
| `anti_spiral_extended.rs` | orbit/stagnation flag (`:26`) |

Live L-1/L-2 paths are **PLANNED** at authoring (`claim/mod.rs`, `risk.rs`). Rebuild the **plan** symbols; do not wait on a cousin extractor inside D-10 grounding-sweep.

## 11. Journal / SSB events

From `L-plan/L-0_INDEX.md:144` and `L-plan/L-22_claimant_authored_falsifiers_demarcation.md:63-64`:

```
FalsifierAuthored {  // Pattern-2; net +1..+2; baseline re-read at edit (W-9)
  Kind: PredicateCompiled | ProgramCompiled | RubricDeferred | ForbidsNothing
}

// prefer-reuse (do not mint a second tier event):
ClaimRiskScored { Kind: LowRisk | MediumRisk | HighRisk | CriticalRisk }

// VS spread: prefer-reuse I-4 candidate rows, else:
VerbalizedDistributionDrawn
```

Authoring-era SCHEMA baseline ~268/577 (`L-plan/L-0_INDEX.md:129`); L-22 text also cites ~268/577 (`:64`). **Do not rewind** a live schema number. Payload field list beyond Kind is unspecified (§16).

## 12. Tests and observables

Named contract `l22_falsifier_demarcation_contract.rs` (`:96`):

| Id | Assert |
|---|---|
| (a) HG-5 | causal claim with no falsifier → risk-floor High |
| (b) evidenced | compiled predicate that passes → scored at evidenced tier (not auto-High) |
| (c) HG-1 | Rubric-only → `causal_supported=false` |
| (d) one decode | mock backend asserts a **single** call; no reconstruction pass |
| (e) GRC | hypothesis step with no falsifier → Converge **refused** |
| (f) VS + orbit | `propose_distribution` returns K typicality-tagged candidates; tail-weighted selector fires **ONLY** under orbit flag (advisory, TD-3) |
| (g) GD-1 | flag-down VS **byte-identical** to I-4 best-of-N |
| (h) TD-3 | model cannot override a tier |

**Deterministic checks** (`:68-72`): `struct Falsifier` / `enum FalsifierKind` / `fn propose_distribution` present; `rg` reconstruction / second-model-call count **0**; `set_tier` / `override_tier` count **0**; `L-22-decisions.md` exists.

**Observable effect** (`:61-64`): generation seam on a causal claim **with** compiled predicate **and** a second causal claim that forbids nothing → `FalsifierAuthored` rows; forbids-nothing scores High; predicate compiles a D-11 `CheckerRef` and scores at evidenced tier; GRC hypothesis with no falsifier refused Converge.

**Negatives:** forbids-nothing below High **FAILS**; Rubric-only `causal_supported=true` **FAILS**; second free-form reconstruct **FAILS**.

**Done-gates:** W-1 live generation seam + L-2 High floor on a real run (not cargo-green); HG-5/HG-1/HG-3/TD-3/GD-1/GD-4; `cargo check --workspace` GREEN; auditor PASS `-p orchestrator` (`:76-82,85`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- L-1 `Claim` carries `falsifier: Option<Falsifier>` with `FalsifierKind { Predicate, Program, Rubric }` and `compiled: Option<CheckerRef>`. **No second claim type.**
- Falsifier is elicited **with** the claim in **one** grammar-masked decode (N-13/E-35). Zero reconstruction calls.
- Predicate/Program compile through the **existing** D-11 sandbox; never raw eval. Rubric never runs in the hot loop.
- Causal/Behavior/Reachability with no falsifier scores **High floor** (HG-5). Compiled-and-passing Predicate/Program scores at evidenced L-2 tier.
- Rubric-only never sets `causal_supported` (HG-1).
- Hypothesis-class GRC-v3 may not `Converge` without a declared falsifier.
- `propose_distribution` is one call of K typicality-tagged candidates. Tail-weighted spend **only** under the orbit flag. Typicality never adjudicates. Flag-down ≡ I-4 best-of-N.
- Model cannot set/override `RiskTier`. Elicitation always-on; acting is L-7.
- `FalsifierAuthored` (or equivalent Kind) is journaled. Contract tests (a)–(h) exist and pass.
- Local-lane skeleton is valid by construction (EX-4), not by asking the model to follow a template.

A prompt “please also list how you could be wrong”, an LLM-judge typicality gate, or a critic that writes the falsifier after the fact is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- Running checkers inside `falsifier.rs` (D-11 owns run) (`:5-6`).
- Second free-form reconstruct call (`:55`).
- Model-set tier / demarcation / `causal_supported` (TD-3) (`:56`).
- Rubric in the hot loop (`:57`).
- Re-implementation of L-1 / L-2 / D-11 / I-4 (GD-1) (`:58`).
- Typicality as a selection or risk gate (`:38,56`).
- Using DeepReason or VS-paper numeric results as ELAI thresholds (ID-5) (`:93`).
- Importing Dung argument-graph, untyped-artifact ontology, embedding convergence, or schools/lineage (L-T5 rejections; L-23 owns the explore bans) (`ELAI-FEATURE-MAP.md:3662-3664`).
- FDL frontier controller / ProbeUtility scheduler / multi-cycle operator allocator / bridge from the planning-harness improvement-conjecture archive into this organ (`ELAI-SPEC-OVERVIEW.md:393-399`).
- Acting on the High floor before L-7 (GD-4) (`:81`).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-22_claimant_authored_falsifiers_demarcation.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:240-246`. Doctrine: `L-plan/L-0_INDEX.md:54,70-73,119-120,144`. Overview: `ELAI-SPEC-OVERVIEW.md:380-399`. Feature map: `ELAI-FEATURE-MAP.md:3653-3666,5073-5088`. Neighbors: `L-plan/L-1_typed_claim_extraction_substrate.md`, `L-plan/L-2_claimrisk_score_convergence_fuse.md`. Steal artifact (design-rationale): `workflow/reasoning/steal-deepreason-2026-07-08.md` S1/S5. VS paper: arXiv:2510.01171 (WebSearch-verified 2026-07-08; ID-5). Rebuild kit neighbors: `D-11-constructive-checker-registry.md`, `I-04-evidence-ranked-search-ladder.md`, `J-83-grcv3-beliefstate-carrier.md`, `C-10-anti-spiral-lateral-thinking.md`.

`ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` has **no** L-22 atom block (L-T5 post-dates that list). Do not mint L22-E### ids here.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Exact required-kind set beyond the Causal/Behavior/Reachability seed; File/Symbol/Dependency optional (`L-plan/L-22_claimant_authored_falsifiers_demarcation.md:114`).
- Whether HG-5 High **floor** bypasses L-2 HG-2’s ≥2-predictor rule for High/Critical (`L-plan/L-2_claimrisk_score_convergence_fuse.md:66` vs HG-5 as floor).
- Numeric `k` for `propose_distribution`; max-weight cap and tail-emphasis values (VS paper = design-rationale only) (`:38,93`).
- `Falsifier.body` length bound / truncate-vs-reject.
- `CheckerRef` rust shape (owned by D-11; L-22 only names the handle).
- `RECON_NEEDED` as a falsifier-equivalent vs a distinct BeliefState flag (`:37`).
- Precise generation-seam file (plan says “the generation seam”, not a path) (`:103`).
- `FalsifierAuthored` payload fields beyond Kind; whether VS always adds `VerbalizedDistributionDrawn` or only when I-4 rows cannot be reused (`L-plan/L-0_INDEX.md:144`).
- Behavior when D-11 is dark (`ELAI_CHECKER_ENABLE` unset): compile-fail vs Inapplicable vs observe-only High floor still applied.
- How “PASSES its own check” composes with D-11 `Inapplicable` (evidenced tier vs treat as forbids-nothing).
- Hypothesis-class GRC detector: which `GrcState` / BeliefState entries count as “debug fork / architecture choice” (`:37`).
- Typicality type (string vs numeric vs enum) and honesty bound (named as a risk, not a schema) (`:116`).
