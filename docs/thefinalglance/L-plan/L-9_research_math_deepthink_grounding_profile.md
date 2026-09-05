# L-9: Research/math deep-think grounding profile — token-perfect via the constructive checker

STATUS: PLANNED (WAVE 4; S-M; EXTENSION — a /reason profile, NOT a new engine)

Objective: researchers working on mathematical papers need token-perfect accuracy a probabilistic model cannot give by generation. The answer is not a separate reasoning engine — it is a PROFILE of ELAI's existing "deep think" (the Chat Reflect Pipeline B-39, /reason: TRIAGE > REASONER N=4 > AUDITOR REFINE > SYNTHESIZE). L-9 wires a `--profile research` arm that runs /reason in GRC Mode A (the D-11 Constructive-Checker Registry sympy/z3/compile + the E-53 Formal Verification Lane) over Cosmos-ingested papers (B-42 + the J-33 MinerU lane + paper-ingestor), with the CGG predictors over the N=4 reasoners: P1 (L-3) round-trip checks each factual claim against the ingested paper bytes (H-53 closed set); P3 (L-4) semantic-entropy over the N=4 reasoners flags a derivation they disagree on. Token-perfect = the symbolic checker ADJUDICATES the math, the model only PROPOSES (TD-3); a derivation sympy/z3 cannot confirm is ABSTAINED, never asserted as proven (HG-4). HARD BOUNDARY: this is configuration + a thin profile selector; it BUILDS no new reasoning engine and no new checker. GD-1: do NOT re-implement /reason, the checkers, or the Cosmos ingest — compose them.

Estimated effort: ~1 session (~S-M). The /reason N=4 + checker legs are local; the live model arm is the user's chosen backend (B-37).
Pre-requisites (re-grep live per W-7): B-39 ChatReflectPipeline (/reason + ReasoningDrawer); D-11 constructive-checker registry (ELAI_CHECKER_ENABLE) + E-53 formal lane; B-42 Cosmos artifact tier + J-33 MinerU lane + paper-ingestor (workflow/papers/); L-2 ClaimRisk + L-3 round-trip + L-4 semantic-entropy.
Parallel-safe with: L-10. Depends L-2, L-3, L-4.
Source: cgg-hallucination-causal-system-2026-06-27.md S4(research) + the user's follow-up + L-CGG-GROUNDING-MAP NN-6/TS-16/TS-17/TS-18. Doctrine: L-0_INDEX.md (HG-4, TD-3).

## Context
Why this matters: the user asked whether ELAI's saved grounding sources + deep-think could serve research people needing token-perfect math. They can, by composition: (1) the grounding source is already persisted — paper-ingestor writes to workflow/papers/, /pdf+MinerU (J-33) parses a PDF into the B-35 artifact pipeline > DKG + Cosmos (B-42); (2) the deep-think already exists — /reason (B-39); (3) the token-perfect adjudicator already exists — Mode A constructive checkers (D-11 sympy/z3, E-53 formal lane), which are EXACT, not probabilistic. L-9 is the thin profile that points all three at the research case + runs the CGG over the reasoners. The decisive doctrine alignment: TD-3 (the LLM never adjudicates) is EXACTLY what token-perfect math needs — the model proposes a derivation, the symbolic checker confirms or the system abstains (HG-4). The CGG P3 over the N=4 reasoners is the bonus: if the four reasoners derive four different answers, that disagreement is surfaced, not silently averaged.

## Existing state (B/D/E coverage)
- B-39 ChatReflectPipeline [WIRED] — TRIAGE > REASONER N=4 > AUDITOR REFINE cap2 > SYNTHESIZE; /reason + ReasoningDrawer. ELAI's deep-think. [MAP]
- D-11 constructive-checker registry [DARK(ELAI_CHECKER_ENABLE)] — sympy/z3/compile/registry-check for Mode A; E-53 formal verification lane (z3/Kani/Verus/Dafny). The token-perfect adjudicator. [MAP]
- B-42 Cosmos artifact tier [WIRED] + J-33 MinerU lane [PLANNED] + paper-ingestor + B-35 Context Artifacts Pipeline [WIRED] — the ingested-paper grounding store (workflow/papers/, ~/.elai/cosmos/). [MAP]
- L-2/L-3/L-4 — the ClaimRisk + round-trip + semantic-entropy the profile runs over the reasoners.

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-6, TS-16/17/18); cgg-hallucination-causal-system-2026-06-27.md S4(research); the user's follow-up.
2. the ChatReflectPipeline (/reason); the D-11 checker dispatch + E-53 lane; the Cosmos ingest + paper-ingestor output path; L-2/L-3/L-4 interfaces.
3. F-0 W-1; G-0 GD-1; CLAUDE.md TD-3 (the checker adjudicates), ID-5, no-.unwrap.

## Tasks (WAVE 4 — the research profile, compose only)
1. the ReasoningProfile selector (~40 LOC): a `--profile research` arm on /reason that (a) sets the grounding source to the Cosmos-ingested papers (B-42), (b) routes claims through Mode A constructive checkers (D-11/E-53), (c) enables the CGG over the reasoners (L-2/L-3/L-4). NO new engine — a selector over BUILT paths.
2. the checker-adjudicates wiring (~40 LOC): a mathematical/derivation claim from a reasoner is sent to sympy/z3 (D-11); a claim the checker confirms is asserted, one it cannot confirm is ABSTAINED (HG-4) with "unverified by constructive check". The model never self-grades the math (TD-3).
3. the CGG-over-reasoners wiring (~30 LOC): run L-3 round-trip (each factual claim vs the ingested paper bytes, H-53 closed set) + L-4 semantic-entropy (over the N=4 reasoners) on the /reason output; surface a divergent derivation.
4. workflow/reasoning/L-9-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/reasoning/research_profile.rs — the `--profile research` selector + the checker-adjudicates wiring + the CGG-over-reasoners wiring
consumes_symbols:
  - B-39 ChatReflectPipeline (/reason); D-11 checker registry + E-53 formal lane; B-42 Cosmos + paper-ingestor; L-2/L-3/L-4 (the CGG predictors) (COMPOSE all — never re-implement)
reachable_from:
  - the /reason CLI + the ReasoningDrawer > run_reason(profile=research) [PRODUCTION research deep-think]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO new reasoning engine and NO new checker (compose B-39 + D-11/E-53)
  - NO model self-grading the math (TD-3 — the symbolic checker adjudicates)
  - NO asserting an unverified derivation (HG-4 — abstain "unverified by constructive check")
  - NO re-implementation of /reason / the checkers / the Cosmos ingest (GD-1 P0); NO .unwrap on prod
frontend_pair:
  the ReasoningDrawer (a research-profile badge + the per-claim checker verdict; decide at edit, W-4).
observable_effect:
  trigger: "`/reason --profile research \"<a claim about an ingested paper + a derivation with a wrong arithmetic step>\"` over a Cosmos-ingested paper (real /reason path)"
  expect_ssb: "prefer-reuse the ChatReflect / grounding-verification rows; net 0 expected from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "each factual claim is round-tripped against the ingested paper bytes; the wrong arithmetic step is caught by sympy/z3 (NOT the model); a derivation the checker cannot verify is ABSTAINED (not asserted as proven); NEGATIVE: an unchecked derivation is never asserted token-perfect (HG-4/TD-3)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-9_research_math_deepthink_grounding_profile.md --run-trigger
rg "profile.*research|ReasoningProfile|Mode ?A|sympy|z3" .elai_cc/crates/orchestrator/src/reasoning/research_profile.rs   # the profile present
rg -ci "self.?grade|model.*adjudicat" .elai_cc/crates/orchestrator/src/reasoning/research_profile.rs   # MUST be 0 (checker adjudicates)
test -f workflow/reasoning/L-9-decisions.md
```

## ACCEPTANCE
- [ ] TD-3: the symbolic checker (sympy/z3, D-11/E-53) adjudicates the math; the model only proposes.
- [ ] HG-4: an unverified derivation is abstained, never asserted as proven.
- [ ] the grounding source is the Cosmos-ingested paper (B-42); claims round-trip against the paper bytes (L-3/H-53).
- [ ] GD-1: /reason + checkers + Cosmos ingest composed, not re-implemented (no new engine).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = `/reason --profile research` catches a wrong arithmetic step via the checker on a real ingested paper (not cargo-green).
- W-2: reuse ELAI_CHECKER_ENABLE for Mode A; the profile is a selector, no new behavior gate.
- W-3: composes BUILT /reason + checkers, not a summary-only port.
- W-4: frontend_pair = the ReasoningDrawer badge (decide at edit).
- W-5/W-10: /reason + checker legs local; the live model arm is the user's backend.
- W-7: B-39 + D-11/E-53 + the Cosmos ingest path re-grepped at edit.
- W-9: SSB prefer-reuse ChatReflect/grounding rows; net 0.
- GD-2: the checker verdict is recomputable (sympy/z3 are deterministic).
- ID-5: any "token-perfect" claim is backed by the checker's exact verdict, never a model confidence.

## Tests to add
- crates/contracts/tests/l9_research_profile_contract.rs (~70 LOC): (a) `--profile research` routes claims through Mode A checkers; (b) a wrong arithmetic step is caught by sympy/z3, not the model; (c) an unverifiable derivation is abstained (not asserted); (d) a factual claim about the ingested paper round-trips against the paper bytes (H-53 closed set); (e) the N=4 reasoners diverging on a derivation is surfaced (L-4); (f) no new engine/checker is created (composition asserted).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/reasoning/research_profile.rs (~110 LOC)
- MODIFY: the /reason CLI arg + the ReasoningDrawer badge (~30 LOC)
- CREATE: crates/contracts/tests/l9_research_profile_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/L-9-decisions.md

## Citations / Grounding
- the user's follow-up (research deep-think + token-perfect math over saved grounding sources); cgg-hallucination-causal-system-2026-06-27.md S4(research).
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-6/TS-16/17/18): B-39 ChatReflectPipeline, D-11 checkers + E-53, B-42 Cosmos + paper-ingestor + J-33 MinerU (re-grepped at edit, W-7).
- CLAUDE.md: TD-3 (the checker adjudicates), ID-5 (token-perfect = the exact checker verdict, never model confidence), no .unwrap on prod.

## Open questions / risks
- Checker coverage: sympy/z3 cover algebra/logic/arithmetic but not every research math (e.g. some analysis/topology); for out-of-coverage derivations the profile ABSTAINS (HG-4) rather than faking confidence — the honest residue, named. The checker-coverage instrument (H-41) measures the gap.
- Mode A is DARK (ELAI_CHECKER_ENABLE): the profile activates it for the research lane; verify the checker_runtime is wired in the boot path (D-11's open residue) before the live arm.

## Next
L-9 makes ELAI's deep-think safe for research/math by letting the exact checker adjudicate and abstaining otherwise. L-10 measures the whole CGG's hallucination-reduction delta, including this profile's abstain-vs-assert accuracy.

End of L-9.
