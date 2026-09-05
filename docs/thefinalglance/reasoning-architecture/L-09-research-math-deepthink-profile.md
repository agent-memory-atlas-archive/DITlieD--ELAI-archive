# Research/math deep-think grounding profile (L-9)

- **ELAI ids:** L-9 / L-T1 / NN-6 / WAVE 4 / HG-4 / TD-3
- **Spec status (as designed):** PLANNED (`L-plan/L-9_research_math_deepthink_grounding_profile.md:3`). Live `reasoning/research_profile.rs` is a **cousin producer** (`research_grounding_pass`): checker-adjudicates derivation lines and writes `ClaimRiskScored`. It is **not** the `/reason --profile research` selector over B-39 + Cosmos + CGG-on-N=4.
- **Crate / module (spec):** `crates/orchestrator/src/reasoning/research_profile.rs`; production caller is `/reason` CLI + ReasoningDrawer `run_reason(profile=research)` (`L-plan/L-9_research_math_deepthink_grounding_profile.md:33-38`).
- **Purpose of this file:** enough architecture to rebuild the research profile — configuration over existing organs, not a new engine. Not a status ledger.

## 1. What it is

The **research/math deep-think grounding profile** is a **PROFILE**, not a new engine. It is a `--profile research` arm on `/reason` (B-39 Chat Reflect: TRIAGE → REASONER **N=4** → AUDITOR REFINE → SYNTHESIZE) that (a) sets the grounding source to **Cosmos-ingested papers** (B-42 + J-33 MinerU + paper-ingestor → `workflow/papers/`), (b) routes claims through **GRC Mode A** constructive checkers (D-11 sympy/z3/compile + E-53 formal lane), (c) runs CGG P1 (L-3) against paper bytes (H-53 closed set) and P3 (L-4) **over the N=4 reasoners**. Token-perfect means the **symbolic checker adjudicates**; the model only **proposes**. A derivation sympy/z3 cannot confirm is **ABSTAINED** (“unverified by constructive check”), never asserted as proven (HG-4, TD-3) (`L-plan/L-9_research_math_deepthink_grounding_profile.md:5,13`, `L-plan/L-SUMMARY_INDEX.md:67-72`, `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:60-65,91-92`).

## 2. Why it exists

Researchers need exact math a generator cannot certify. ELAI already has the three pieces: persisted papers, `/reason` N=4, Mode A checkers. L-9 **points them together** (`L-plan/L-9_research_math_deepthink_grounding_profile.md:13`). If this organ is missing, `/reason` still synthesizes by auditor `best_index` (B-39 stage 4 is **local string selection**, not a checker) (`B-39-chat-reflect-pipeline.md` §3 stage 4). If it is faked (a new “research engine”, or the model marking a derivation proven), TD-3/GD-1 fail. P3 over N=4 is the bonus: four different derivations are **surfaced**, not silently averaged (`:13`).

L-15 R5: this profile is the **flagship near-0 hallucination demo** where a receipt exists (symbolic check) (`L-plan/L-0_INDEX.md:189`, `L-plan/L-15_repair_controller_and_evidence_first_reorder.md` R5 named in L-0). Honest residue: sympy/z3 do not cover analysis/topology — **abstain**, do not fake; H-41 measures the gap (`:93`).

Do **not** confuse with L-23 `/reason --profile explore` (conjecture-criticism map). Different profile, same `/reason` door (`L-plan/L-SUMMARY_INDEX.md:247-250`).

## 3. How it works (runtime)

1. **Who calls it.** `/reason --profile research "<claim about ingested paper + derivation>"` and the ReasoningDrawer. Not contracts-only (R7) (`:38-39,48`).
2. **Selector (thin, ~40 LOC)** (`:27`):
   - grounding source = Cosmos-ingested papers (B-42 / `workflow/papers/`)
   - Mode A = D-11/E-53 (`ELAI_CHECKER_ENABLE` reused; **no new flag**) (`:69`)
   - CGG on = L-2/L-3/L-4 over reasoner outputs
3. **B-39 run (compose, do not fork).** Same pipeline: TRIAGE / REASONER N=4 / AUDITOR REFINE cap=2 / SYNTHESIZE (`B-39-chat-reflect-pipeline.md` §1). L-9 does **not** change N, cap, or SYNTHESIZE-is-local. It **adds** Mode A + paper closed-set + CGG **on top** of those stages.
4. **Checker adjudicates (~40 LOC)** (`:28`). Each mathematical/derivation claim → D-11 sympy/z3 (E-53 when in coverage). Checker **Pass** → may assert. Else **Abstain** with `"unverified by constructive check"`. Model never self-grades (`:42-43`).
5. **CGG over reasoners (~30 LOC)** (`:29`):
   - L-3: each factual claim vs ingested **paper bytes**; H-53 ghost citation cannot be “supported”
   - L-4: semantic entropy over the **N=4 reasoner texts** (same claim); diverge → surface, do not average
6. **Commit.** Prefer-reuse ChatReflect / grounding-verification rows. Net 0 (`:49`, `L-plan/L-0_INDEX.md:138`). Live cousin also writes one aggregate `ClaimRiskScored` (`research_profile.rs:94-110`) — allowed as L-2 reuse, not a new engine event.
7. **Fail-closed.** Checker missing / gate off / Fail / Inapplicable → Abstain, never Asserted (`research_profile.rs:163-203` matches HG-4). Out-of-coverage math → Abstain (`:93`). Mode A DARK: profile **activates** `checker_runtime` for this lane; verify boot wiring before live arm (`:94`).
8. **The model may generate** TRIAGE class, N=4 derivations, auditor scores (B-39). It may **not** mark proven, pick Mode A off, expand H-53 after generation, or average away P3 disagreement.

Live cousin: `research_grounding_pass(ssb, agent_id, superstep, reply, candidates)` (`research_profile.rs:51-113`) extracts via paper-store resolver, scores `score_claim_risk_wired_with_samples`, `derivation_lines` = lines with `=` and digits both sides (cap 16) (`:139-160`), `adjudicate_line` through first Sympy/Z3/NumericRange checker (`:165-211`). **No** `/reason --profile research` CLI in this tree (grep of that flag hits I-42 run profiles, not B-39). Rebuild the **selector + B-39 hook**, not only the pass.

## 4. Types, enums, objects

Plan names a selector, not a large type block (`:27,33`).

```
ReasoningProfile { Research, /* other profiles are other organs */ }

fn run_reason(profile=research, query, papers) -> ResearchReasonResult

// implied:
//   Mode A checker verdict per derivation
//   L-3 residual per factual claim vs paper bytes
//   L-4 entropy over N=4 reasoners
//   abstain string "unverified by constructive check"
```

**Live cousin types** (`research_profile.rs:21-43`) — compatible fillers, not a license to skip the CLI selector:

```
AbstainReason { CheckerGateOff, CheckerMissing, CheckerFailed }
ResearchVerdict { Asserted, Abstained(AbstainReason) }
ResearchClaimVerdict { subject, verdict }
ResearchGroundingOutcome { verdicts, row_written }
```

**Consumed:** `ChatReflectPipeline` (`B-39-chat-reflect-pipeline.md` §4); D-11 `CheckerRegistry` / `CheckerResult`; E-53 formal lane; B-42 Cosmos; H-53 anchors from paper bytes; L-2/L-3/L-4.

**B-39 N=4** is Liu inflection for 3B–7B (`B-39-chat-reflect-pipeline.md:31-32` wait: N=4 at stage 2 `:7`). L-9 **uses** that N for P3; it does not invent N=5 from L-4’s fixture.

## 5. Control flow / state machine

**Owner:** `/reason` profile selector. B-39 owns the loop; L-9 owns **which grounding + which adjudicator**.

```
/reason --profile research
  → B-39 TRIAGE (Chat may still skip — §16)
  → REASONER N=4 (propose only)
  → L-3 round-trip vs paper bytes (H-53)
  → D-11/E-53 Mode A on derivation claims
       Pass → may assert that line
       else → Abstain "unverified by constructive check"
  → L-4 entropy over the 4 reasoner texts (surface diverge)
  → AUDITOR REFINE cap=2  (B-39; must not override checker Fail)
  → SYNTHESIZE local select  (must not select a checker-failed line as proven)
```

**Forced:** no new engine (`:41`). Reuse `ELAI_CHECKER_ENABLE` (`:69`). Parallel-safe with L-10; depends L-2/L-3/L-4 (`:9`). L-5 do-test is **not** in the L-9 compose list (depends L-2,L-3,L-4 only) — Causal paper claims stay Mode A / abstain, not a workspace revert.

**AUDITOR vs checker:** B-39 auditor may pick `best_index` by generative score. L-9 **must not** let that pick assert a line the checker failed. How that veto is wired is **UNCERTAIN** (§16); rebuild must still satisfy HG-4.

## 6. Grounding, receipts, gates

Receipts: D-11 Pass/Fail, H-53 Bound, L-4 cluster entropy. **ID-5:** “token-perfect” is the checker’s exact verdict, never model confidence (`:76`).

**Selector hierarchy:**

1. sympy/z3/E-53 Pass → assert
2. checker Fail / missing / gate off / out of coverage → Abstain
3. H-53 Ghost → cannot emit as paper-supported
4. P3 diverge → surface disagreement (does not by itself prove)
5. B-39 auditor confidence / SYNTHESIZE pick → **never** overrides 1–2

**Never allowed:** model self-grade; averaging N=4 into a fake consensus; claiming token-perfect without a checker Pass.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| selector | ~40 LOC | `research_profile.rs` |
| checker wire | ~40 LOC | same |
| CGG wire | ~30 LOC | same |
| file | ~110 LOC | (`:82`) |
| CLI + drawer | ~30 LOC | (`:83`) |
| contract | ~70 LOC | (`:79`) |
| B-39 N | 4 (1..=8) | reasoners / P3 samples |
| B-39 REFINE | cap 2 | do not raise here |
| live cousin derivation cap | 16 lines (`research_profile.rs:143`) | not in L-9 file |
| SSB | reuse ChatReflect/grounding; net 0 | W-9 |
| Mode A flag | `ELAI_CHECKER_ENABLE` | W-2 reuse |

## 8. Roles and neighbors

Chat path: `chat_reasoner` / `chat_auditor` configs from B-39. Profile does not add a research role; it **selects Mode A + paper store**.

**Consumes:** B-39, D-11, E-53, B-42, J-33, paper-ingestor, L-2/L-3/L-4, H-53 (`:35-36`).

**Produces:** profiled `/reason` output with checker verdicts + CGG signals; reused Reflect*/grounding rows.

**Must not fork:** a second reflect pipeline; a new sympy; L-23 explore profile.

**Frontend:** ReasoningDrawer research-profile **badge** + per-claim checker verdict (W-4, decide at edit) (`:45-46`).

**H-41:** coverage instrument for the honest residue (`:93`).

## 9. Config / flags / defaults

- **No new behavior gate.** Mode A stays `ELAI_CHECKER_ENABLE` (`:69`).
- `/reason` remains opt-in (`B-39-chat-reflect-pipeline.md` §3.2 default off). Profile is an arm **when** reason is on.
- Decision doc: `workflow/reasoning/L-9-decisions.md`. **Absent**.
- J-33 MinerU may still be PLANNED — papers already in `workflow/papers/` still ground (`:18`).
- No `.unwrap()` (`:44`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `crates/orchestrator/src/reasoning/research_profile.rs` | selector + checker wire + CGG wire (`:34`) |
| `run_reason(profile=research)` | production name (`:38`) |
| `/reason` CLI `--profile research` | ~30 LOC (`:83`) |
| ReasoningDrawer badge | W-4 |
| B-39 `ChatReflectPipeline` | `crates/reasoning/src/chat_reflect.rs` |
| D-11 registry | `checker_registry.rs` |
| Cosmos / papers | B-42, `workflow/papers/` |
| `l9_research_profile_contract.rs` | tests (a)–(f) (`:84`) |

**Live neighbors:** `research_grounding_pass` (`research_profile.rs:51`); L-5 recon named `research_profile.rs:75` as a `score_claim_risk_wired_with_samples` caller (line **75** is now score, not the function entry — re-grep). No `l9_research_profile_contract.rs` at architecture time.

## 11. Journal / SSB events

Prefer-reuse ChatReflect (`ReflectTriaged` / `ReflectCandidate` / `ReflectAuditVerdict` / `ReflectRefined`) and D-10/D-11 grounding-verification / CheckerPass-Fail (`:49`, `L-plan/L-0_INDEX.md:138`). Net 0.

Live cousin extra: one `ClaimRiskScored` when claims or derivations exist (`research_profile.rs:94-110`). Compatible with L-2 reuse. Empty reply writes **no** row (`:50`).

GD-2: checker verdict recomputable (sympy/z3 deterministic) (`:75`).

## 12. Tests and observables

Named (`:79`):

| Id | Assert |
|---|---|
| (a) | `--profile research` routes through Mode A checkers |
| (b) | wrong arithmetic caught by sympy/z3, **not** the model |
| (c) | unverifiable derivation abstained, not asserted |
| (d) | factual paper claim round-trips against paper bytes (H-53) |
| (e) | N=4 reasoners diverging is surfaced (L-4) |
| (f) | no new engine/checker (composition) |

Wiring (`:53-57`): `rg profile.*research|ReasoningProfile|Mode A|sympy|z3`; `rg -ci self.?grade|model.*adjudicat` MUST be 0.

**observable_effect** (`:47-50`, `L-plan/L-SUMMARY_INDEX.md:72`): `/reason --profile research` over an ingested paper round-trips facts, checker catches a wrong arithmetic step, unverifiable derivation abstained. Negative: unchecked derivation never asserted token-perfect.

**W-1:** live `/reason --profile research` catches wrong arithmetic via the checker on a **real ingested paper** (`:68`).

Adversarial: new engine; model “QED”; average of 4 wrong derivations; token-perfect without Pass; explore profile mixed in.

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `/reason --profile research` is a **selector** over B-39 + D-11/E-53 + Cosmos papers + L-2/L-3/L-4 — **no** new reasoning engine or checker.
- Model proposes; sympy/z3/E-53 adjudicates (TD-3).
- Unverified derivation abstains with a constructive-check reason (HG-4); never asserted proven.
- Factual claims round-trip against **ingested paper bytes** (H-53).
- P3 runs over the **N=4 reasoners** and surfaces divergence (no silent average).
- Mode A uses existing `ELAI_CHECKER_ENABLE`, no new gate.
- SSB prefer-reuse ChatReflect/grounding.
- Tests (a)–(f). Production path is `/reason` + ReasoningDrawer, not only `research_grounding_pass`.

A producer that checker-grades `a=b` lines on an arbitrary reply **without** the `/reason` profile and paper store is a **cousin**, not this organ.

## 14. Non-goals / banned designs

- New reasoning engine or new checker (`:41`).
- Model self-grading math (`:42`).
- Asserting unverified derivations (`:43`).
- Re-implementing `/reason`, D-11, Cosmos ingest (GD-1 P0).
- Raising B-39 REFINE cap or adding a 5th inference SYNTHESIZE.
- Using paper AUROC as “token-perfect” (ID-5).
- L-23 explore profile behavior on this arm.
- Workspace L-5 do-test as the math adjudicator (not in L-9 compose list).

## 15. Source citations

Primary: `docs/thefinalglance/L-plan/L-9_research_math_deepthink_grounding_profile.md` (full file). Index: `L-plan/L-SUMMARY_INDEX.md:67-72`. Doctrine: `L-plan/L-0_INDEX.md:103-104,138,189`. Grounding map: `L-plan/_grounding/L-CGG-GROUNDING-MAP.md:60-65,91-92`. Neighbors: `B-39-chat-reflect-pipeline.md`, `D-11-constructive-checker-registry.md`, `L-03-evidence-roundtrip-predictor.md`, `L-04-semantic-entropy-predictor.md`. Live: `reasoning/research_profile.rs`. L-23 contrast: `L-plan/L-SUMMARY_INDEX.md:247-250`.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- Whether B-39 TRIAGE `Chat` skip still bypasses Mode A (would skip the whole point).
- Auditor `best_index` vs checker Fail veto wiring.
- `ReasoningProfile` enum vs a string flag.
- NumericRange as a math checker (live includes it; plan names sympy/z3).
- E-53 Kani/Verus/Dafny vs sympy/z3 which claims they take.
- Paper-store path when Cosmos is empty (live falls back to workspace resolver then untyped extract — `research_profile.rs:59-65`).
- Derivation-line heuristic (`=` + digits) vs ClaimKind::Causal/Api.
- Drawer badge vs N/A details.
- Whether L-4 N=5 fixture applies or strictly B-39 N=4.
- Boot `checker_runtime` residue D-11 (`:94`).
