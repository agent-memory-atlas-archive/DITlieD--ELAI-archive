# L-22: claimant-authored falsifiers + a demarcation gate + the Verbalized-Sampling propose contract

STATUS: PLANNED (WAVE 1; M-L; BUILD-on-shipped + EXTENSION — the generation-time half of the CGG the L-T1..L-T4 verify-time machinery was missing)

PROOF_SPINE (PSR-2): feeds PS-3 (hallucination / grounded-claims delta, L-10) — the L-1/L-2 corpus gains a falsifier-carrying claim class, and a Causal/Behavior claim that forbids nothing scores a High risk-floor (an empty attack surface is un-grounded by construction). S5's VS spread also cuts the propose-leg token cost that gates how many claims L-10 can score per run.

Objective: today ELAI EXTRACTS claims off-agent (L-1) and SCORES them with converging predictors (L-2..L-5) AFTER the model has spoken. DeepReason's measured move (steal-deepreason-2026-07-08.md S1) inverts the authorship: the GENERATOR emits, WITH every design-claim / hypothesis / diagnosis, the specific machine-checkable condition that would REFUTE it (the skeleton's `forbidden[]`), and the harness compiles that falsifier into a runnable predicate and executes it at ZERO LLM cost — the only criticism that measured cost-positive at low base error in his ablations ("free criticism"). L-22 adds that elicitation to ELAI's claim substrate: (a) an OPTIONAL falsifier field on the L-1 ClaimSet, elicited AT GENERATION TIME and grammar-enforced (N-13/E-35) so the weak lane pays no adherence cost; (b) a DEMARCATION verdict fused into L-2's ClaimRisk — a Causal/Behavior claim that forbids NOTHING has an empty attack surface and scores risk-floor High; (c) the same contract on hypothesis-class GRC-v3 steps (debug forks, architecture choices): no declared falsifier, no Converge. It ALSO lands the Verbalized-Sampling propose contract (S5, EXTENSION of I-4): elicit a DISTRIBUTION of K candidates each with a self-assessed typicality in ONE call, and — only when the stagnation/orbit flag is up — spend criticism budget on the candidates the model itself marks ATYPICAL. HARD BOUNDARY: L-22 elicits + compiles + tiers; it does NOT run the checkers (D-11 owns that), does NOT set a tier by model judgment (TD-3), and typicality is ADVISORY-only, never an adjudication. GD-1: extend L-1's ClaimSet + L-2's ClaimRisk + D-11's registry + I-4's candidate ladder — re-implement none.

Estimated effort: ~1 session (~M-L). In-process at the claim-intake + generation seams; the falsifier PREDICATE eval runs through D-11's existing sandbox; no network leg (the VS elicitation is one normal model call, cheaper than I-4's N calls).
Pre-requisites (re-grep live per W-7): L-1 ClaimSet (crates/orchestrator/src/claim/mod.rs — PLANNED, land L-1 first); L-2 ClaimRisk fuse (crates/orchestrator/src/claim/risk.rs — PLANNED); D-11 constructive-checker registry [DARK(ELAI_CHECKER_ENABLE)]; E-35 grammar substrate (crates/providers/src/runtime/sidecar_xgrammar_hook.rs + crates/providers/src/decoding.rs:229 DecodingMode::from_mode); N-13 OutputBoundary resolver (PLANNED — provider-strict > native-tool > local-CFG > regex > parse-retry); J-83 GRC-v3 BeliefState (PLANNED); I-4 best-of-N candidate ladder; LIVE SCHEMA/TV (ssb.rs:1519 + action_type_lock.rs).
Parallel-safe with: L-23 (disjoint files; L-23 consumes L-22's falsifier-carrying claim class). Depends: L-1, L-2 land first.
Source: workflow/reasoning/steal-deepreason-2026-07-08.md S1 + S5. arXiv:2510.01171 (Verbalized Sampling) VERIFIED 2026-07-08. Doctrine: L-0_INDEX.md (HG-1, HG-5-new, TD-3, ID-5).

## Context
Why this matters: DeepReason's `forbidden[]` makes Popperian DEMARCATION structural at zero LLM cost — a candidate that forbids nothing is refuted by a program on arrival, because a claim with no possible refuter is not a scientific claim, it is decoration with an empty attack surface. ELAI already blocks claims WITHOUT a measurable procedure at the PLAN grain (plan-standard EXPECTED_OBSERVATION/ON_FAIL, D-12 MEASURABLE_PROCEDURE) and round-trips claims against its OWN checkers (L-1/L-3), but it never asks the GENERATOR to name its own refuter. The measured payoff (his ablations): claimant-authored predicate falsifiers were the one criticism class that was cost-POSITIVE at low base error — the model naming the check is cheaper than a separate critic reconstructing it, and a Forbid-nothing candidate is caught by a program, never by a paid critic. Verbalized Sampling (arXiv:2510.01171: prompting for a distribution over responses restores the pretraining diversity that RLHF mode-collapses, with distribution-level controls including capping the max weight / emphasizing tail candidates) gives the propose leg a diversity lever that is prompt-SHAPE, not prompt-CONTENT — so it composes with L-23's S3 rule that complement/"be different" directives are a diversity placebo. Every number here is DeepReason's or the VS paper's = DESIGN-RATIONALE only (ID-5); ELAI re-measures its own falsifier-hit and diversity deltas at L-10.

## Existing state (B/C/D/E/F/G/H/I/J/K coverage)
- L-1 ClaimSet [PLANNED, crates/orchestrator/src/claim/mod.rs] — Claim{kind, span, refs, taint}; the object L-22 adds an optional `falsifier` field to. [steal-deepreason S1]
- L-2 ClaimRisk fuse [PLANNED, crates/orchestrator/src/claim/risk.rs] — RiskTier{Low,Medium,High,Critical}; L-22 fuses a DEMARCATION signal into the tier (forbids-nothing => High floor). [steal-deepreason S1]
- D-11 constructive-checker registry [DARK(ELAI_CHECKER_ENABLE)] — the per-kind check backends; L-22 compiles a predicate-class falsifier through it, runs nothing itself. [L-1 MAP]
- E-35 grammar substrate [WIRED/SHIPPED-class, sidecar_xgrammar_hook.rs + decoding.rs:229] — the logit-mask that makes the falsifier field valid BY CONSTRUCTION on the weak lane (zero adherence retries). [elai.map]
- N-13 OutputBoundary resolver [PLANNED] — the strongest-available-enforcement ladder the elicitation routes through. [N-plan]
- I-4 best-of-N candidate ladder [PARTIAL] — the propose leg VS replaces (one distribution call vs N resamples). [steal-deepreason S5]
- J-83 GRC-v3 BeliefState [PLANNED] — the reasoning carrier that gets the no-falsifier-no-Converge contract on hypothesis-class steps. [steal-deepreason S1]
- anti-spiral [WIRED, crates/orchestrator/src/anti_spiral_extended.rs] — the stagnation/orbit flag that gates when VS spends budget on the atypical tail (composes L-23 S2). [elai.map]

## Reads (mandatory before coding)
1. workflow/reasoning/steal-deepreason-2026-07-08.md §3.1/§3.5/S1/S5; the DeepReason spec §0/§11 mechanics (harness-spec-v1.3.md) — skeleton {claim, mechanism, scope, forbidden[]}; compiled-commitment types predicate:/program:/rubric:.
2. crates/orchestrator/src/claim/mod.rs (L-1 ClaimSet) + risk.rs (L-2 fuse); D-11 registry interface; decoding.rs:229 DecodingMode + sidecar_xgrammar_hook.rs (the grammar mask); the I-4 candidate-ladder interface; J-83 BeliefState step schema.
3. F-0 W-1/W-7; G-0 GD-1/GD-2/GD-4; L-0 HG-1 (causal-means-interventional) + the new HG-5 (demarcation); CLAUDE.md TD-3, ID-5, no-.unwrap-on-prod, blake3-only.

## Tasks (WAVE 1 — elicit the refuter, compile it, tier the empty attack surface)
1. the Falsifier type + the ClaimSet field (~40 LOC): `Falsifier{ kind: FalsifierKind{Predicate, Program, Rubric}, body: String, compiled: Option<CheckerRef> }`; add `falsifier: Option<Falsifier>` to Claim (L-1). A Predicate/Program falsifier COMPILES to a D-11 CheckerRef at registration (zero LLM); a Rubric falsifier is judged offline (deferred, never in the hot loop — matches his rubric: class + ELAI's I-24/N-15 offline judge audits).
2. the generation-time grammar-enforced elicitation (~70 LOC): at the generation seam, when a step emits a design-claim / hypothesis / diagnosis, elicit the falsifier field through the N-13 resolver + the E-35 mask so the output is valid by construction on ANY backend (HG-3 black-box; the weak lane pays zero adherence retries — his <50B failure mode fixed structurally). NEVER a second free-form model call to "reconstruct" the falsifier — it is emitted WITH the claim in one masked decode.
3. the DEMARCATION fuse into ClaimRisk (~50 LOC): a Causal/Behavior/Reachability claim whose `falsifier` is None (forbids nothing) has an empty attack surface -> risk-floor High (HG-5); a claim carrying a compiled Predicate/Program falsifier that PASSES its own check is scored at its evidenced tier; a Rubric-only falsifier caps the causal_supported bit false (HG-1 — rubric is not interventional). The tier is deterministic (TD-3); the model never sets it.
4. the GRC-v3 no-falsifier-no-Converge gate (~35 LOC): a hypothesis-class GRC step (debug fork, architecture choice) may not transition to Converge unless its BeliefState entry carries a declared falsifier (a RECON_NEEDED check or a compiled predicate); missing => the step stays open / routes to L-6 targeted effort (HG-4). This is plan-standard's RECON_NEEDED made a runtime reasoning gate.
5. the Verbalized-Sampling propose contract (~60 LOC): a `propose_distribution(prompt, k)` call eliciting K candidates each with a self-assessed typicality in ONE masked decode (replaces an I-4 N-resample for the propose leg); a max-weight cap + tail-emphasis knob (the VS paper's distribution-level controls); typicality is ADVISORY (TD-3) — it steers criticism budget only when the anti-spiral/orbit flag (L-23 S2) is up, spending on the atypical tail, and NEVER adjudicates a tier or a selection. Cold-start / flag-down => behaves as I-4's existing best-of-N (GD-1, no regression).
6. workflow/reasoning/L-22-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/falsifier.rs — Falsifier / FalsifierKind + the compile-to-D-11 path + the DEMARCATION fuse
  - crates/orchestrator/src/claim/verbalized_sampling.rs — propose_distribution(K, typicality) + the tail-weighted-under-stagnation selector
  - the generation-seam elicitation hook (grammar-enforced falsifier + VS distribution)
consumes_symbols:
  - claim/mod.rs::ClaimSet (L-1 — ADD a field, never re-type); claim/risk.rs::ClaimRisk (L-2 — fuse a signal, never re-fuse)
  - D-11 checker registry (compile + run the predicate — COMPOSE, never re-implement the sandbox)
  - decoding.rs::DecodingMode + sidecar_xgrammar_hook.rs (E-35 mask); N-13 OutputBoundary resolver (strongest-available)
  - I-4 candidate ladder (the propose leg VS replaces); J-83 BeliefState (the GRC step carrier); anti_spiral_extended.rs (the orbit/stagnation flag)
reachable_from:
  - the orchestrator generation seam > elicit_falsifier + propose_distribution [PRODUCTION generation-intake]
  - the L-2 claim-risk fuse > demarcation_signal [PRODUCTION risk-scoring]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO second free-form model call to reconstruct a falsifier (one masked decode, emitted WITH the claim)
  - NO model setting/overriding a tier or a demarcation verdict (TD-3); typicality is advisory only, never a gate
  - NO rubric falsifier running in the hot loop (offline judge only, I-24/N-15)
  - NO re-implementation of L-1/L-2/D-11/I-4 (GD-1 P0); NO .unwrap()/.expect() on prod; blake3 not sha2
frontend_pair:
  a falsifier chip on the L-2 trust/claim tile (the claim's declared refuter + its pass/forbids-nothing state), decide at edit (W-4) OR "N/A with justification — the demarcation state rides the existing ClaimRisk tile. Justified."
observable_effect:
  trigger: "run the orchestrator generation seam on a step that emits a causal claim WITH a compiled predicate falsifier and a second causal claim that forbids nothing (real generation + claim-intake seam)"
  expect_ssb: "+FalsifierAuthored (Pattern-2; Kind: PredicateCompiled/ProgramCompiled/RubricDeferred/ForbidsNothing); prefer-reuse the L-2 ClaimRiskScored row for the tier; baseline+1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the forbids-nothing claim scores risk-floor High (HG-5, empty attack surface); the predicate-carrying claim compiles a D-11 CheckerRef and is scored at its evidenced tier; a GRC hypothesis step with no falsifier is REFUSED Converge. NEGATIVE: a claim with no falsifier scored below High FAILS; a Rubric-only claim marked causal_supported=true FAILS (HG-1)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-22_claimant_authored_falsifiers_demarcation.md --run-trigger
rg "struct Falsifier|enum FalsifierKind|fn propose_distribution" .elai_cc/crates/orchestrator/src/claim/falsifier.rs .elai_cc/crates/orchestrator/src/claim/verbalized_sampling.rs   # produced surfaces present
rg -c "reconstruct.*falsifier|second.*model call" .elai_cc/crates/orchestrator/src/claim/falsifier.rs   # MUST be 0 (one masked decode)
rg -ci "set_tier|override_tier" .elai_cc/crates/orchestrator/src/claim/falsifier.rs   # MUST be 0 (TD-3; deterministic tier)
test -f workflow/reasoning/L-22-decisions.md
```

## ACCEPTANCE
- [ ] HG-5 (new): a Causal/Behavior/Reachability claim that forbids nothing scores risk-floor High (empty attack surface).
- [ ] HG-1: a Rubric-only falsifier never sets causal_supported; only a compiled interventional predicate/program does.
- [ ] HG-3: the falsifier field is elicited grammar-enforced on a backend exposing no logprobs (black-box; weak-lane zero adherence retries).
- [ ] TD-3: the tier + the demarcation verdict are deterministic; typicality is advisory and steers budget only, never adjudicates.
- [ ] GD-1: L-1 ClaimSet + L-2 fuse + D-11 registry + I-4 ladder are EXTENDED, not re-implemented.
- [ ] GD-4: ships OBSERVE-only (records falsifier state + demarcation) until L-7's gate flag acts on it.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live generation seam emits a falsifier-carrying claim + the L-2 fuse floors a forbids-nothing claim at High on a real run (not cargo-green).
- W-2: no enable-flag (elicitation is always-on intake; the ACTING gate is L-7's kill-switch).
- W-3: adds a field + a fuse signal to BUILT L-1/L-2 objects, not a summary-only port.
- W-4: falsifier chip decided at edit or justified N/A (rides the ClaimRisk tile).
- W-5/W-10: in-process; the predicate eval runs through D-11's existing sandbox; no network leg.
- W-7: claim/mod.rs, claim/risk.rs, D-11, decoding.rs:229, the I-4 + J-83 interfaces re-grepped at edit.
- W-9: +FalsifierAuthored or prefer-reuse; the SCHEMA baseline re-read live.
- GD-2: the demarcation tier is recomputable from the (falsifier present? compiled? passed?) signals under ELAI_SEED.
- ID-5: any falsifier-hit / diversity threshold is ELAI-measured (L-10), never DeepReason's or the VS paper's number.

## Tests to add
- crates/contracts/tests/l22_falsifier_demarcation_contract.rs (~90 LOC): (a) a causal claim with no falsifier scores risk-floor High (HG-5); (b) a claim with a compiled predicate falsifier that passes is scored at its evidenced tier; (c) a Rubric-only falsifier leaves causal_supported=false (HG-1); (d) the falsifier field is emitted in ONE decode (mock backend asserts a single call, no reconstruction pass); (e) a GRC hypothesis step with no falsifier is refused Converge; (f) propose_distribution returns K typicality-tagged candidates and the tail-weighted selector fires ONLY under the orbit flag (advisory, TD-3); (g) flag-down VS is byte-identical to the I-4 best-of-N path (GD-1 no regression); (h) the model cannot override a tier (TD-3).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/falsifier.rs (~120 LOC)
- CREATE: .elai_cc/crates/orchestrator/src/claim/verbalized_sampling.rs (~80 LOC)
- MODIFY: .elai_cc/crates/orchestrator/src/claim/mod.rs (~10 LOC: the optional falsifier field on Claim)
- MODIFY: .elai_cc/crates/orchestrator/src/claim/risk.rs (~20 LOC: the demarcation fuse signal)
- MODIFY: the generation seam (~35 LOC: grammar-enforced elicitation hook + the GRC no-falsifier-no-Converge gate)
- CREATE: crates/contracts/tests/l22_falsifier_demarcation_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-22-decisions.md

## Citations / Grounding
- workflow/reasoning/steal-deepreason-2026-07-08.md S1 (claimant-authored falsifiers compiled to checks + demarcation) + S5 (VS contract + tail-weighted selection under stagnation); DeepReason harness-spec-v1.3.md §0/§11 (the skeleton + compiled-commitment types) [EXTERNAL/DESIGN-RATIONALE, ID-5].
- arXiv:2510.01171 "Verbalized Sampling: How to Mitigate Mode Collapse and Unlock LLM Diversity" — VERIFIED via WebSearch 2026-07-08; the distribution-level controls (max-weight cap, tail emphasis) are the S5 knobs [EXTERNAL/DESIGN-RATIONALE, ID-5].
- ELAI ground truth (re-grepped at edit, W-7): claim/mod.rs + risk.rs (L-1/L-2 PLANNED), D-11 registry, decoding.rs:229 DecodingMode, sidecar_xgrammar_hook.rs (E-35), I-4 ladder, J-83 BeliefState, anti_spiral_extended.rs.
- CLAUDE.md: TD-3 (LLM never adjudicates), ID-5 (ELAI-measured thresholds), no .unwrap on prod, blake3-only.

## Open questions / risks
- The exact set of claim kinds that REQUIRE a falsifier (Causal/Behavior/Reachability yes; File/Symbol/Dependency are already round-trip-checkable by L-1/L-3 so a falsifier is optional there): seed from the DeepReason forbidden-case discipline, OBSERVE-first before HG-5 floors anything (GD-4). Decide the required-kind set at authoring.
- Predicate-falsifier safety: a claimant-authored predicate is untrusted input — it MUST compile through D-11's existing sandbox (his checks.py AST-guard is the weak version; ELAI's is stronger), never eval'd raw. Verify the sandbox boundary at edit.
- VS typicality honesty: a model's self-assessed typicality can be miscalibrated; it is ADVISORY-only precisely because of this (steers budget, never gates). The L-10 measure checks whether tail-spend actually raises novelty on ELAI's own runs before the flag-gated path is trusted.

## Next
L-22 gives the CGG a generation-time attack surface: claims arrive already carrying their own refuter, demarcation is structural, and the propose leg has a prompt-shape diversity lever. L-23 (the /reason explore profile) is its first heavy consumer — it runs the falsifier-carrying claim class + the VS spread + the orbit detector + cross-family judging as one exploratory-reasoning mode on the local lane.

End of L-22.
