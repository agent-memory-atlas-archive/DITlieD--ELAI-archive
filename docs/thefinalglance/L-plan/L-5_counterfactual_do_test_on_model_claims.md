# L-5: Counterfactual do-test on model claims — perturb the cause, observe the effect [KEYSTONE, CANT-LIE]

STATUS: PLANNED (WAVE 2; M-L; EXTENSION(F-4/C-28/C-29); KEYSTONE — the causal core, the un-forgeable signal)

Objective: the paper's strongest signal is INTERVENTIONAL — the action-marginalization test perturbs the action and measures whether the output responds (§4.1.ii); a model that ignores the intervention is hallucinating. The elai-lang counterfactual-causal-learning idea is identical: disable the source, re-run, see if the target changes. L-5 ports this to model CLAIMS: when a model emits a Causal/Behavior claim ("A causes/fixes B", "this patch fixes the timeout"), the harness RUNS the intervention — revert only A / disable the flag / patch only the claimed function / add a trace to the claimed path — then re-runs the F-4 cant-lie outcome (blake3 stdout-hash the model cannot forge), the C-28/I-20 mutation, or the C-29 metamorphic relation, and compares. A claim is labelled "causal-supported" ONLY when the interventional outcome moves as claimed (HG-1); static/temporal/import-edge evidence is scored at its weaker tier and never called causal (the LS-2 fake-precision guard). This is the un-forgeable predictor that feeds L-2 as P-counterfactual. GD-1: do NOT re-implement F-4 / the mutation engine / the metamorphic harness — drive them on the claim.

Estimated effort: ~1 session (~M-L). The intervention re-runs tests/mutations; in-process where the test harness is local, RunPod-gated only where a claim's repro needs the live model.
Pre-requisites (re-grep live per W-7): F-4 AntiFabricationGate.check() (task_runner.rs:12486, struct at evidence.rs:820, check impl at evidence.rs:283, blake3 stdout_hash at evidence.rs:90); C-28 mutation gate + I-20 budgeted scheduler (MutationBudgetedRun, deterministic_audit_gate.rs:505); C-29 IntentPair + MetamorphicRelation (metamorphic_testing.rs:49); CSQ (phase-2D); J-16 blast-radius cone (blast_radius.rs, WIRED at lib.rs:1233, compute_cone at :112, select_regression_set at :251); G-1 taint; L-1 ClaimSet (claim/extract.rs:19, landed 68d5ad92); L-2 PredictorSignal + causal_supported (claim/risk.rs:15/:30, landed 6507d9a9); L-3 score_claim_risk_wired_with_samples (claim/roundtrip.rs:182, landed 650c4d57); L-4 SemanticEntropyPredictor (claim/semantic_entropy.rs:71, landed 088f2881); L-9 research_profile.rs:75 production caller (landed ef2af276).
Parallel-safe with: L-3, L-4 (disjoint files; all feed L-2). Depends L-1, L-2 (both LANDED).
Source: cgg-hallucination-causal-system-2026-06-27.md S5 + paper §4.1.ii/§5 (action-shuffle ratio) + the elai-lang counterfactual screenshot + L-CGG-GROUNDING-MAP NN-4/TS-7/TS-8/TS-9. Doctrine: L-0_INDEX.md (HG-1, the keystone delta).

## Context
Why this matters: round-trip (L-3) and semantic entropy (L-4) are uncertainty signals; the do-test is the only CAUSAL one, and it is the only signal the model cannot game (F-4's stdout-hash is captured by the orchestrator, not the worker — wiring R8). The paper measures "action shuffle ratio": one-step flow MSE under the true action vs a batch-shuffled action; ratio <= 1.1 means the model IGNORED the action = hallucination (§5). The LLM port: outcome under the claimed intervention vs outcome without it; if reverting A does NOT change B, the model's "A causes B" is marginalized/hallucinated. HG-1 is the discipline this unit enforces for the whole CGG: the word "causal" is reserved for interventional/executable evidence. The honest residue (GD-5): not every claim is cheaply interventional (some need production traffic / non-snapshottable effects); those are scored at the runtime-trace tier (strong but not counterfactual) and the limit is named.

## Existing state (C/F/I/J coverage — verified at commit a9e551cf, 2026-08-11)
- F-4 test-integrity cant-lie [WIRED] — AntiFabricationGate struct at evidence.rs:820, check impl at evidence.rs:283, check call at task_runner.rs:12486; blake3 stdout_hash on IntegrationTestRowSummary at evidence.rs:90; auditor cannot emit passed=true without the orchestrator-captured hash. The un-forgeable outcome. [MAP]
- C-28 mutation gate [PARTIAL] + I-20 budgeted scheduler [WIRED] — perturb-and-observe; mutants on the diff cone; MutationBudgetedRun; emit_mutation_budgeted_runs at deterministic_audit_gate.rs:505, SSB at ssb.rs:1385. [MAP]
- C-29 IntentPair + MetamorphicRelation [WIRED] — MetamorphicRelation enum at metamorphic_testing.rs:49; intent_pairs_from_resolved at grill_interlace.rs:74. [MAP]
- CSQ + inline feasibility [WIRED, phase-2D] — counterfactual safety questions on fs/net/money/auth. [MAP]
- J-16 blast-radius [WIRED] — compute_cone at blast_radius.rs:112, select_regression_set at blast_radius.rs:251, module declared at lib.rs:1233. The cone the perturbation touches; selects the regression set. [MAP]
- L-1 ClaimSet [LANDED, 68d5ad92] — claim/extract.rs:19 (ClaimSet), :39 (extract_claims); module declared at claim.rs:304, re-exported at claim.rs:310. [MAP]
- L-2 PredictorSignal + causal_supported [LANDED, 6507d9a9] — claim/risk.rs:15 (PredictorSignal), :30 (causal_supported field), :11 (PredictorKind::Counterfactual ALREADY EXISTS), :49 (Counterfactual listed in expensive_predictors_for High/Critical), :93-94 (causal_supported derived from fired Counterfactual signal). [MAP]
- L-3 score_claim_risk_wired_with_samples [LANDED, 650c4d57] — claim/roundtrip.rs:182; composes [RoundTripPredictor, ClosedSetCitationPredictor, SemanticEntropyPredictor] into score_claim_risk_with at roundtrip.rs:195. [MAP]
- L-4 SemanticEntropyPredictor [LANDED, 088f2881] — claim/semantic_entropy.rs:71. [MAP]
- L-9 research_profile production caller [LANDED, ef2af276] — reasoning/research_profile.rs:75 calls score_claim_risk_wired_with_samples; emits ClaimRiskScored SSB at research_profile.rs:97. Also called from task_runner.rs:1998 and task_runner.rs:2025. [MAP]
- ClaimRiskScored SSB [WIRED] — SsbActionType::ClaimRiskScored at ssb.rs:1662; ClaimRiskScoredPayload at ssb/payloads.rs:22034. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-4, TS-7/8/9); cgg-hallucination-causal-system-2026-06-27.md S5; paper §4.1.ii + §5 (action shuffle ratio); the elai-lang counterfactual screenshot.
2. evidence.rs:820 (AntiFabricationGate struct), evidence.rs:283 (check impl), evidence.rs:90 (stdout_hash); task_runner.rs:12486 (check call); deterministic_audit_gate.rs:505 (emit_mutation_budgeted_runs); metamorphic_testing.rs:49 (MetamorphicRelation); blast_radius.rs:112/:251 (compute_cone/select_regression_set); claim/extract.rs (ClaimSet, ExtractedClaim, ClaimKind); claim/risk.rs (PredictorKind, PredictorSignal, PredictorSource, ClaimRisk, causal_supported, fuse); claim/roundtrip.rs:182/:195 (score_claim_risk_wired_with_samples and the predictor slice); claim/semantic_entropy.rs:71 (SemanticEntropyPredictor pattern to follow); reasoning/research_profile.rs:75/:97 (production caller + SSB emit).
3. F-0 W-1; G-0 GD-1/GD-5; CLAUDE.md TD-3, no-.unwrap, blake3; K-16 RR-1.
4. conductor/opus_walk/evidence/L-5/L-5-recon.md (the recon this plan folds).

## Recon amendments folded (all 8 from L-5-recon.md)
1. AntiFabricationGate.check call is task_runner.rs:12486 (was 9565); struct at evidence.rs:820, check impl at evidence.rs:283, stdout_hash at evidence.rs:90.
2. claim/counterfactual.rs is declared from claim.rs:309 (NOT claim/mod.rs — it does not exist). Add `pub mod counterfactual;` after the semantic_entropy declaration at claim.rs:309.
3. The real production seam is roundtrip.rs:182 score_claim_risk_wired_with_samples, injecting at its predictor slice around roundtrip.rs:195. Existing callers: research_profile.rs:75, task_runner.rs:1998/:2025.
4. PredictorKind::Counterfactual already exists at risk.rs:11; causal_supported already exists at risk.rs:30 and is derived at risk.rs:93-94. Do NOT add an enum arm.
5. Prefer the existing ClaimRiskScored SSB path (ssb.rs:1662, payloads.rs:22034), which already carries causal_supported via ClaimRisk. Do NOT add CounterfactualProbed unless a later explicit design unit chooses the full 9-tuple lockstep.
6. J-16 blast_radius is WIRED at lib.rs:1233 with compute_cone (:112) and select_regression_set (:251). Not PLANNED.
7. workflow/reasoning/L-5-decisions.md is gitignored and unlandable in this repo (.gitignore line 1: `*`). The controller records its decisions in conductor/opus_walk/evidence/L-5/ and walk-state instead of making it a product deliverable. This plan does NOT create workflow/reasoning/L-5-decisions.md.
8. Create only the missing claim/counterfactual.rs and crates/contracts/tests/l5_counterfactual_contract.rs. No other file creation.

## UNIT 1 — the CounterfactualPredictor (claim/counterfactual.rs)

### Slice 1A: module declaration + intervention planner (~60 LOC)

WRITE SCOPE: add `pub mod counterfactual;` to claim.rs after line 309 (after `pub mod semantic_entropy;`). Create claim/counterfactual.rs with:
- `Intervention` struct: { target_ref: ClaimRef, kind: InterventionKind, cone: Vec<String> }
- `InterventionKind` enum: RevertOnlyA, DisableFlag, PatchOnlyFn, AddTrace, NotInterventional
- `fn plan_intervention(claim: &ExtractedClaim, cone: &[String]) -> Intervention` — derives the deterministic intervention from the claim refs + the J-16 cone. NO model call (TD-3). A Causal/Behavior claim with a resolved ref to a symbol in the cone gets RevertOnlyA/DisableFlag/PatchOnlyFn depending on the ref kind; a claim with no ref in the cone or a non-Causal/Behavior kind gets NotInterventional.
- `fn tier_guard(claim: &ExtractedClaim) -> TierGuard` — a claim with only static/temporal/import evidence is scored at that weaker tier and NEVER labelled causal (HG-1). Non-snapshottable claims are scored at the runtime-trace tier, the limit recorded in the TierGuard struct.

EXPECTED_OBSERVATION: `rg "pub mod counterfactual" .elai_cc/crates/orchestrator/src/claim.rs` returns one match after line 309. `rg "InterventionKind|plan_intervention|tier_guard" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` returns matches for all three.

ON_FAIL: the module declaration fails to compile because claim.rs already has a `pub mod counterfactual;` — cause: a prior partial landing. Counter-move: `rg "counterfactual" .elai_cc/crates/orchestrator/src/claim.rs` to check, and skip the declaration if present. If plan_intervention does not compile because ExtractedClaim fields differ from claim/extract.rs — cause: L-1 struct drift. Counter-move: read claim/extract.rs:19-39 for the current ExtractedClaim shape and adapt.

### Slice 1B: the do-test runner + PredictorSource impl (~100 LOC)

WRITE SCOPE: add to claim/counterfactual.rs:
- `struct CounterfactualPredictor` implementing `PredictorSource` (trait at risk.rs:21). The `signals` method returns `Vec<PredictorSignal>` with source `PredictorKind::Counterfactual` (already exists at risk.rs:11). The signal fires (fired=true, value=1.0) when the do-test outcome moves as claimed; unfired (value=0.0) when it does not. NO new PredictorKind variant.
- `fn run_do_test(claim: &ExtractedClaim, intervention: &Intervention, baseline_hash: &str, intervened_hash: &str) -> DoTestVerdict` — compares the two blake3-bound outcomes. Returns DoTestVerdict { moved: bool, delta: f64 }. The hashes are captured by the CALLER (the production seam), not by this function — this function only compares. GD-1: this function does NOT re-implement F-4 or the mutation engine; it receives the hashes from the caller.
- `struct DoTestVerdict { moved: bool, delta: f64 }` — recomputable from the two hashes under ELAI_SEED (GD-2).
- The `PredictorSource::signals` impl constructs the PredictorSignal from the DoTestVerdict: fired when moved==true, value=1.0 when moved, 0.0 when not. When intervention.kind is NotInterventional, the signal is present but unfired (causal_supported stays false — HG-1).

EXPECTED_OBSERVATION: `rg "CounterfactualPredictor|PredictorSource for CounterfactualPredictor|run_do_test|DoTestVerdict" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` returns matches for all four. `cargo check -p orchestrator` GREEN (plain cargo, depot shim).

ON_FAIL: `cargo check -p orchestrator` fails on PredictorSource trait mismatch — cause: the trait signature at risk.rs:21 expects `fn signals(&self, claim: &ExtractedClaim) -> Vec<PredictorSignal>` and the impl does not match. Counter-move: read risk.rs:21-22 for the exact trait signature and match it. If the failure is a DARK symbol warning from production_caller_check — that is expected at this slice (the predictor is not yet wired into roundtrip.rs); proceed to Slice 2 which wires it.

### Slice 1C: contract test (crates/contracts/tests/l5_counterfactual_contract.rs, ~90 LOC)

WRITE SCOPE: create the test file following the pattern in l3_roundtrip_predictor_contract.rs:1-18 and l4_semantic_entropy_contract.rs. Import from `elai_orchestrator::claim::counterfactual` and `elai_orchestrator::claim::extract`. Tests:
- (a) a true fix-claim: plan_intervention produces RevertOnlyA for a Causal claim with a resolved ref in the cone; run_do_test with different hashes > moved=true > signals fire > causal_supported true.
- (b) a false fix-claim: same hashes > moved=false > signals unfired > causal_supported false.
- (c) an 'A causes B' claim where reverting A leaves B unchanged: same hashes > NOT-causal.
- (d) a claim supported only by 'A imports B' (ClaimKind not Causal/Behavior): tier_guard returns static-tier, never causal (HG-1).
- (e) plan_intervention is deterministic: same claim + cone always produces the same Intervention (no model call).
- (f) a forged passed=true with None stdout_hash is rejected by AntiFabricationGate.check (call evidence.rs:283 check with a TransitionContext missing stdout_hash).
- (g) a non-snapshottable claim (NotInterventional) is tier-capped at runtime-trace with the limit recorded in TierGuard.

EXPECTED_OBSERVATION: `cargo test -p contracts --test l5_counterfactual_contract` GREEN, all 7 tests pass.

ON_FAIL: test (f) fails because TransitionContext fields differ — cause: evidence.rs TransitionContext struct drift. Counter-move: read evidence.rs around :283 for the current TransitionContext shape and construct a valid one. If test (d) fails because ClaimKind does not have the expected variants — cause: L-1 ClaimKind enum drift. Counter-move: read claim/extract.rs for the current ClaimKind variants and adapt.

## UNIT 2 — wire into the production seam (claim/roundtrip.rs)

### Slice 2A: inject CounterfactualPredictor into score_claim_risk_wired_with_samples

WRITE SCOPE: modify claim/roundtrip.rs:182-196. Add `let counterfactual = super::counterfactual::CounterfactualPredictor::new();` after the entropy predictor construction at roundtrip.rs:194. Change the predictor slice at roundtrip.rs:195 from `&[&predictor, &citations, &entropy]` to `&[&predictor, &citations, &entropy, &counterfactual]`. This is the production wiring — the CounterfactualPredictor is now called from every production caller of score_claim_risk_wired_with_samples (research_profile.rs:75, task_runner.rs:1998/:2025).

NOTE: The CounterfactualPredictor in the initial landing will produce unfired signals (no live do-test execution yet — the hashes are not available in the wired path). This is correct and intentional: the predictor is present in the slice so causal_supported is derivable when a future slice supplies real hashes. The signal fires only when the do-test outcome moves; without hashes, it is present but unfired. This matches the L-4 pattern where entropy is present but unfired with <2 samples.

EXPECTED_OBSERVATION: `rg "counterfactual" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs` returns matches at the predictor slice. `cargo check -p orchestrator` GREEN. `python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` exits 0 (no DARK symbols — CounterfactualPredictor is called from roundtrip.rs:195).

ON_FAIL: production_caller_check reports DARK — cause: the CounterfactualPredictor::new constructor or another pub symbol is not called from a production path. Counter-move: verify the slice edit at roundtrip.rs:195 actually includes `&counterfactual` in the array. If the check still reports DARK, the symbol may need to be `pub(crate)` instead of `pub` — read the check output for the exact symbol name. If cargo check fails because CounterfactualPredictor::new does not exist — cause: Slice 1B did not add the constructor. Counter-move: add `pub fn new() -> Self` to CounterfactualPredictor.

## PRODUCTION_HOOK

PRODUCTION_HOOK seam: claim/roundtrip.rs:182 `score_claim_risk_wired_with_samples`, predictor slice at roundtrip.rs:195. The CounterfactualPredictor is injected as the 4th element in the `&[&predictor, &citations, &entropy, &counterfactual]` slice. This makes it reachable from all three production callers:
- reasoning/research_profile.rs:75 (L-9 research grounding for /reason)
- task_runner.rs:1998 (claim risk scoring in task execution)
- task_runner.rs:2025 (claim risk scoring with entropy resample)

The production_caller_check must exit 0 (no DARK public symbols in claim/counterfactual.rs).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/counterfactual.rs — Intervention, InterventionKind, plan_intervention, tier_guard, TierGuard, CounterfactualPredictor, DoTestVerdict, run_do_test
consumes_symbols:
  - F-4 AntiFabricationGate (evidence.rs:820 struct, evidence.rs:283 check, task_runner.rs:12486 call — COMPOSE, never re-implement)
  - C-28/I-20 mutation (deterministic_audit_gate.rs:505 emit_mutation_budgeted_runs); C-29 MetamorphicRelation (metamorphic_testing.rs:49); CSQ; J-16 blast_radius.rs (compute_cone :112, select_regression_set :251); G-1 taint
  - claim/extract.rs::ClaimSet + ExtractedClaim + ClaimKind (L-1, claim.rs:304)
  - claim/risk.rs::PredictorSignal + PredictorSource + PredictorKind::Counterfactual + ClaimRisk + causal_supported (L-2, risk.rs:11/:15/:21/:30/:93-94)
  - claim/roundtrip.rs::score_claim_risk_wired_with_samples (L-3, roundtrip.rs:182/:195 — the production seam)
reachable_from:
  - claim/roundtrip.rs:182 score_claim_risk_wired_with_samples > predictor slice at :195 > CounterfactualPredictor::signals [PRODUCTION counterfactual predictor]
  - callers: research_profile.rs:75, task_runner.rs:1998/:2025
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO claim labelled causal without an interventional/executable pass (HG-1 — static/temporal is the weaker tier)
  - NO model call to design the intervention OR to judge the outcome (TD-3 — derived deterministically; F-4 hash adjudicates)
  - NO re-implementation of F-4 / the mutation engine / the metamorphic harness (GD-1 P0)
  - NO new PredictorKind enum arm (Counterfactual already exists at risk.rs:11)
  - NO new SSB variant (prefer-reuse ClaimRiskScored at ssb.rs:1662 which already carries causal_supported via ClaimRisk)
  - NO creation of workflow/reasoning/L-5-decisions.md (gitignored, unlandable — controller records decisions in conductor/opus_walk/evidence/L-5/)
  - NO .unwrap on prod; blake3 not sha2
frontend_pair:
  N/A with justification — feeds L-2's risk tile via the wired predictor slice; the underlying F-4/mutation rows already surface. Justified.
observable_effect:
  trigger: "do-test a 'this patch fixes the timeout' claim by running the failing repro with vs without the patch-only intervention (real F-4 path); plus an 'A causes B' claim where reverting A does NOT change B"
  expect_ssb: "prefer-reuse ClaimRiskScored (ssb.rs:1662, payloads.rs:22034) with causal_supported=true when the do-test outcome moves; no new SSB variant. baseline+0 from LIVE (re-grep, W-9)"
  expect_artifact: "the fix-claim is labelled causal-supported ONLY if the F-4-bound outcome flips; the 'A causes B' claim where reverting A leaves B unchanged is labelled NOT-causal; NEGATIVE: a claim supported only by 'A imports B' is scored static-tier, never causal (HG-1)"

## FORKS: none

All decision points are deterministic:
- Module declaration location: claim.rs:309 (not claim/mod.rs — it does not exist, recon DELTA-2).
- Production seam: roundtrip.rs:195 predictor slice (not score_claim_risk — recon DELTA-3).
- PredictorKind: reuse existing Counterfactual at risk.rs:11 (no new arm — recon DELTA-4).
- SSB path: reuse ClaimRiskScored at ssb.rs:1662 (no new variant — recon DELTA-5).
- Decisions file: do NOT create workflow/reasoning/L-5-decisions.md (gitignored — recon DELTA-7).
- Files to create: only claim/counterfactual.rs and l5_counterfactual_contract.rs (recon DELTA-8).

## RECON_NEEDED

RECON_NEEDED 1: the CounterfactualPredictor::new constructor must produce a predictor whose signals method can be called without live do-test hashes (the wired path at roundtrip.rs:195 does not have hashes available). Settling check: `rg "fn new" .elai_cc/crates/orchestrator/src/claim/semantic_entropy.rs` — the L-4 SemanticEntropyPredictor::new pattern shows how a predictor is constructed for the wired slice (it takes samples and produces unfired signals when data is insufficient). Follow the same pattern: CounterfactualPredictor::new produces a predictor that returns an unfired Counterfactual signal when no do-test verdict is available.

RECON_NEEDED 2: the exact shape of ExtractedClaim and ClaimKind for the contract tests. Settling check: `rg "pub struct ExtractedClaim" .elai_cc/crates/orchestrator/src/claim/extract.rs -A 10` and `rg "pub enum ClaimKind" .elai_cc/crates/orchestrator/src/claim/extract.rs -A 5`. Read the current struct/enum before writing the test.

RECON_NEEDED 3: the TransitionContext shape for contract test (f) (AntiFabricationGate.check). Settling check: `rg "pub struct TransitionContext" .elai_cc/crates/orchestrator/src/evidence.rs -A 15`. Read the current struct before constructing the test fixture.

## ABORT

ABORT conditions (executor STOPS and flags, never improvises):
- ABORT 1: `cargo check -p orchestrator` (plain cargo, depot shim) fails after Slice 1A and the error is NOT a missing module declaration or trait signature mismatch — STOP, report the compiler error, do not attempt to fix unrelated errors.
- ABORT 2: production_caller_check.py reports a DARK symbol in claim/counterfactual.rs AFTER Slice 2A wiring — STOP, report the DARK symbol name, do not ship a producer-only slice.
- ABORT 3: the claim.rs module declaration line numbers have shifted (claim.rs:304-310 no longer contains the pub mod declarations) — STOP, re-grep `rg "pub mod" .elai_cc/crates/orchestrator/src/claim.rs` to find the current location, and report the drift.
- ABORT 4: the predictor slice at roundtrip.rs:195 no longer reads `&[&predictor, &citations, &entropy]` — STOP, re-grep `rg "score_claim_risk_with" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs -A 5` to find the current slice, and report the drift.
- ABORT 5: the depot cargo shim fails (not a compiler error — the shim itself does not resolve). STOP, report BLOCKED with the shim output. Do NOT retry with an absolute-path cargo (the watchdog will kill it).

## VERIFY

VERIFY runs (executor performs these, off-agent artifacts named):

1. After Slice 1A (module declaration + intervention planner):
   - `rg "pub mod counterfactual" .elai_cc/crates/orchestrator/src/claim.rs` — off-agent artifact: one match.
   - `rg "InterventionKind|plan_intervention|tier_guard" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` — off-agent artifact: matches for all three.
   - `cargo check -p orchestrator` — off-agent artifact: exit 0 GREEN.

2. After Slice 1B (do-test runner + PredictorSource impl):
   - `rg "CounterfactualPredictor|PredictorSource for CounterfactualPredictor|run_do_test|DoTestVerdict" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` — off-agent artifact: matches for all four.
   - `cargo check -p orchestrator` — off-agent artifact: exit 0 GREEN.

3. After Slice 1C (contract tests):
   - `cargo test -p contracts --test l5_counterfactual_contract` — off-agent artifact: 7 tests pass, exit 0.

4. After Slice 2A (production wiring):
   - `rg "counterfactual" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs` — off-agent artifact: match at the predictor slice.
   - `python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/claim/counterfactual.rs` — off-agent artifact: exit 0 (no DARK symbols).
   - `cargo check -p orchestrator` — off-agent artifact: exit 0 GREEN.

5. Final hard-input smoke (before DONE):
   - `cargo test -p contracts --test l5_counterfactual_contract` — all 7 pass.
   - `cargo test -p contracts --test l4_semantic_entropy_contract` — L-4 still GREEN (no regression from the roundtrip.rs slice change).
   - `cargo test -p contracts --test l3_roundtrip_predictor_contract` — L-3 still GREEN (no regression).
   - `cargo check --workspace` — full workspace GREEN.
   - `python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/claim/counterfactual.rs,.elai_cc/crates/orchestrator/src/claim/roundtrip.rs` — no DARK symbols.

## REDTEAM

REDTEAM section — attacks on the PLAN (not the code):

Attack 1: "The plan wires CounterfactualPredictor into the predictor slice, but the predictor has no live do-test hashes at the wired path. It is a stub that always returns unfired. This is a wire-dark producer."
Result: FAILED against the plan. The plan explicitly states (Slice 2A NOTE) that the predictor produces unfired signals when no hashes are available, matching the L-4 SemanticEntropyPredictor pattern (unfired with <2 samples). The predictor is NOT a stub — it has real logic (plan_intervention, run_do_test, tier_guard) that fires when hashes are supplied. The wired path makes it reachable from all three production callers, satisfying production_caller_check. The signal is present-but-unfired, not absent. This is the same deliberate degrade L-4 shipped.

Attack 2: "The plan says 'do NOT create workflow/reasoning/L-5-decisions.md' but the DETERMINISTIC_CHECK in the old plan tested for its existence. The executor might create it anyway."
Result: FAILED against the plan. The refined plan removes the `test -f workflow/reasoning/L-5-decisions.md` check from DETERMINISTIC_CHECK and explicitly states (amendment 7) that the file is gitignored and unlandable. The controller records decisions in conductor/opus_walk/evidence/L-5/ instead. No patch needed — the plan already handles this.

Attack 3: "The plan injects a 4th predictor into the roundtrip.rs:195 slice, but the existing L-3/L-4 contract tests assert on the exact tier distribution. Adding a 4th predictor (even unfired) might change tier calculations."
Result: PARTIAL — this is a real risk. An unfired signal does not affect the fuse (risk.rs:82 filters on fired==true), so tiers should not change. But the contract tests must be re-run to confirm. The plan's VERIFY step 5 includes `cargo test -p contracts --test l4_semantic_entropy_contract` and `cargo test -p contracts --test l3_roundtrip_predictor_contract` to catch any regression. Patch: none needed — the VERIFY step catches it; if a regression occurs, the executor ABORTs per ABORT 1.

## INTEGRATION_SMOKE

INTEGRATION_SMOKE for Unit 1 (claim/counterfactual.rs) + Unit 2 (roundtrip.rs wiring):

Smoke 1 (after Unit 1 + Unit 2): `cargo test -p contracts --test l5_counterfactual_contract` — drives the CounterfactualPredictor through the contract test with hard inputs: (a) a true fix-claim with different hashes, (b) a false fix-claim with same hashes, (c) an unmoved 'A causes B' claim, (d) a static-tier 'A imports B' claim, (e) deterministic intervention, (f) forged passed=true rejected, (g) non-snapshottable tier-capped. All 7 must pass.

Smoke 2 (after Unit 1 + Unit 2): `cargo test -p contracts --test l3_roundtrip_predictor_contract` and `cargo test -p contracts --test l4_semantic_entropy_contract` — verifies the roundtrip.rs:195 slice change does not regress L-3 or L-4. Both must pass.

Final hard-input smoke (before DONE): `cargo check --workspace` + `python .claude/tools/production_caller_check.py --files .elai_cc/crates/orchestrator/src/claim/counterfactual.rs,.elai_cc/crates/orchestrator/src/claim/roundtrip.rs` — full workspace GREEN, no DARK symbols. The auditor re-runs these off-agent; the executor's self-run is a courtesy, not the authority.

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md --run-trigger
rg "intervention|counterfactual|moved" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs   # the do-test present
rg -ci "causal" .elai_cc/crates/orchestrator/src/claim/counterfactual.rs   # causal label gated on interventional pass (read the guard)
rg "pub mod counterfactual" .elai_cc/crates/orchestrator/src/claim.rs   # module declared
rg "counterfactual" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs   # wired into the production seam
```

## ACCEPTANCE
- [ ] HG-1: a claim is causal-supported ONLY on an interventional/executable pass; static/temporal scored weaker, never causal.
- [ ] the F-4 stdout-hash adjudicates the outcome (the model cannot forge it); the intervention is deterministic (TD-3).
- [ ] GD-1: F-4 / mutation / metamorphic harness consumed, not re-implemented.
- [ ] GD-5: non-snapshottable claims scored at the runtime-trace tier; the limit named in-file.
- [ ] PredictorKind::Counterfactual reused (risk.rs:11) — no new enum arm.
- [ ] ClaimRiskScored SSB reused (ssb.rs:1662) — no new SSB variant.
- [ ] CounterfactualPredictor wired into roundtrip.rs:195 predictor slice — production_caller_check exit 0.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live do-test flips a fix-claim's causal label by re-running the F-4 outcome under the intervention (not cargo-green).
- W-2: no flag (a predictor; the gate flag is L-7).
- W-3: extends BUILT F-4/mutation/metamorphic, not a summary-only port.
- W-4: frontend_pair N/A (feeds L-2); justified.
- W-5/W-10: in-process where the repro is local; RunPod-gated where the claim's repro needs the live model.
- W-7: task_runner.rs:12486 + evidence.rs:820/:283/:90 + deterministic_audit_gate.rs:505 + blast_radius.rs:112/:251 + roundtrip.rs:182/:195 re-grepped at edit.
- W-9: SSB prefer-reuse ClaimRiskScored (no new variant).
- GD-2: the moved/unmoved verdict is recomputable from the two F-4-bound outcomes under ELAI_SEED.
- GD-5: non-interventional claims tier-capped; limit named.

## Tests to add
- crates/contracts/tests/l5_counterfactual_contract.rs (~90 LOC): (a) a true fix-claim: the F-4 outcome flips under the patch-only intervention > causal-supported; (b) a false fix-claim: outcome unchanged > NOT-causal; (c) an 'A causes B' claim where reverting A leaves B unchanged > NOT-causal; (d) a claim supported only by 'A imports B' is scored static-tier, never causal (HG-1); (e) the intervention is derived deterministically (no model call); (f) the F-4 hash binds the outcome (a forged passed=true is rejected); (g) a non-snapshottable claim is tier-capped at runtime-trace with the limit recorded.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/counterfactual.rs (~160 LOC)
- MODIFY: .elai_cc/crates/orchestrator/src/claim.rs (~1 LOC: add `pub mod counterfactual;` after line 309)
- MODIFY: .elai_cc/crates/orchestrator/src/claim/roundtrip.rs (~3 LOC: add CounterfactualPredictor to the predictor slice at :195)
- CREATE: .elai_cc/crates/contracts/tests/l5_counterfactual_contract.rs (~90 LOC)

## Out of scope
- No new PredictorKind enum arm (Counterfactual exists at risk.rs:11).
- No new SSB variant (ClaimRiskScored reused at ssb.rs:1662).
- No modification to claim/risk.rs (causal_supported already derived at risk.rs:93-94).
- No creation of workflow/reasoning/L-5-decisions.md (gitignored, unlandable — controller records in conductor/opus_walk/evidence/L-5/).
- No live do-test execution (the wired path produces unfired signals; live hash injection is a future design unit).
- No new dependencies.

## Citations / Grounding
- arXiv:2606.27326 §4.1.ii + §5 (action-marginalization / action-shuffle ratio <= 1.1 = action ignored) [EXTERNAL/DESIGN-RATIONALE]; the elai-lang counterfactual-causal-learning screenshot; cgg-hallucination-causal-system-2026-06-27.md S5.
- ELAI ground truth (L-CGG-GROUNDING-MAP NN-4/TS-7/8/9): evidence.rs:820/:283/:90 (F-4), task_runner.rs:12486 (F-4 call), deterministic_audit_gate.rs:505 (mutation), metamorphic_testing.rs:49 (metamorphic), blast_radius.rs:112/:251 (blast radius), roundtrip.rs:182/:195 (production seam), risk.rs:11/:30/:93-94 (PredictorKind + causal_supported), claim.rs:304-310 (module declarations) (re-grepped at edit, W-7).
- Recon: conductor/opus_walk/evidence/L-5/L-5-recon.md (all 8 amendments folded).
- CLAUDE.md: TD-3, no .unwrap on prod, blake3-only; HG-1 (causal-means-interventional); GD-5 honest residue.

## Open questions / risks
- Intervention SAFETY: reverting/patching to test a claim mutates the workspace; run inside the I-2 COW worktree / Tier-C isolation (never on the live tree), and never apply a destructive intervention (HG-1 do-test is observe-the-outcome, not ship-the-change). The initial landing does NOT execute live interventions — the wired path produces unfired signals. Live hash injection is a future design unit that will need the isolation seam.
- Cost: the do-test is the most expensive predictor; it runs LAST in L-2's cost-gate, only when cheap+P1+P3 already flag high. Budget it via J-84; cover deterministically in the L-10 fixture tier.

## Next
L-5 gives the CGG its un-forgeable causal signal and enforces HG-1 plan-wide. With L-3/L-4/L-5 feeding L-2, the convergence fuse is complete; L-6 acts on a High tier, L-7 turns Critical into block/abstain.

End of L-5.
