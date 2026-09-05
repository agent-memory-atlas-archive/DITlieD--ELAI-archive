# L-3: Evidence round-trip predictor (P1) — does a claim survive re-grounding against its evidence

STATUS: PLANNED (WAVE 2; M; BUILD-on-shipped — ACTIVATION of the D-10 grounding methods over arbitrary claims)

Objective: the paper's predictor 1 is the tokenizer round-trip residual — a predicted latent whose decoded frame does not survive re-encoding signals perceptual hallucination (§4.2). The LLM analogue (LS-2): a claim that cannot be RECONSTRUCTED/supported from evidence = high residual. L-3 runs each typed claim from L-1's ClaimSet through the SHIPPED grounding methods (D-10/D-11 FILE/SOURCE/TEST/CosmosCheck) + the H-53 closed-set citation membership + DKG validate-citation (H-32), and emits a P1 PredictorSignal{RoundTrip, residual} into L-2. A claim whose cited file/symbol/line/source cannot be reconstructed scores high residual. This is ACTIVATION: the verification methods exist and run today inside the agent's own grounding-sweep; L-3 points them at ANY model's ClaimSet (HG-3). GD-1: do NOT re-implement any grounding method or the citation pre-binder — call them.

Estimated effort: ~1 session (~M). In-process; CosmosCheck/WEB legs are network-touching but the FILE/SOURCE/SYMBOL legs (the bulk) are local.
Pre-requisites (re-grep live per W-7): D-10 select_grounding_mode (grounding_mode.rs:157); D-11 checker registry; H-53 citation_prebind (citation_prebind.rs, closed-set membership); H-32 DKG validate-citation; the DKG symbol graph; L-1 ClaimSet; L-2 PredictorSignal interface.
Parallel-safe with: L-4, L-5 (disjoint files; all three feed L-2).
Source: cgg-hallucination-causal-system-2026-06-27.md S2 + paper §4.2 (predictor 1) + L-CGG-GROUNDING-MAP TS-1/TS-2 + NN(P1). Doctrine: L-0_INDEX.md (HG-3).

## Context
Why this matters: round-trip is the cheapest and most decisive coding-claim check — most agent hallucinations are invented files/symbols/packages/signatures and fake citations (the user notes' catch-list), all of which fail a reconstruction against the repo. ELAI's grounding verification is binary pass/fail per method (MiniCheck-style, D-10); H-53's closed-set citation membership makes a ghost citation structurally impossible. L-3 is the round-trip residual = the inverse of how many methods reconstruct the claim. It is a CHEAP signal (HG-2 cost-gate: it runs first; the expensive predictors L-4/L-5 fire only when L-3 already flags medium residual).

## Existing state (D/G/H coverage)
- D-10/D-11 grounding verification [WIRED/DARK] — 6-8 methods FILE/SOURCE/TEST/WEB/API/API_REGISTRY/CosmosCheck/VisualArtifactCheck, binary pass/fail; select_grounding_mode deterministic (grounding_mode.rs:157). [MAP]
- H-53 citation pre-binding [PARTIAL, citation_prebind.rs] — closed-set EvidenceAnchorSet membership; out-of-set > D-16 refusal. [MAP]
- H-32 TIER-A code-grounded fact memory [PARTIAL] — validate-citation-before-use on DKG, 28d TTL; SSB FactValidationFailed. [MAP]
- the DKG [WIRED] — the symbol graph the symbol/reachability legs resolve against. [MAP]

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (TS-1/TS-2); cgg-hallucination-causal-system-2026-06-27.md S2; paper §4.2 predictor 1.
2. grounding_mode.rs (select_grounding_mode); the D-11 checker dispatch; citation_prebind.rs (closed-set); H-32 validate-citation; the DKG query surface; L-1 ClaimSet + L-2 PredictorSignal.
3. F-0 W-1; G-0 GD-1; CLAUDE.md TD-3, no-.unwrap, blake3.

## Tasks (WAVE 2 — round-trip each claim against its evidence)
1. the per-kind round-trip dispatcher (~70 LOC): map each ClaimKind to its grounding method (File>FILE, Symbol>DKG/SOURCE, Dependency>API_REGISTRY, Api>SOURCE/compile, Behavior>TEST, Reachability>DKG call-graph, Causal>deferred to L-5); run via the SHIPPED D-10/D-11 methods. NO re-implementation.
2. the residual score (~40 LOC): residual = 1 - (reconstructing-methods / applicable-methods), blended with the H-53 closed-set membership result (a citation out of the closed set = max residual). Emit a P1 PredictorSignal{RoundTrip, residual} into L-2.
3. wire into L-2's cost-gated predictor call (~20 LOC): L-3 is the CHEAP first predictor; its residual is the medium-risk trigger for L-4/L-5.
4. workflow/reasoning/L-3-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/roundtrip.rs — the per-kind round-trip dispatcher + residual score + P1 PredictorSignal
consumes_symbols:
  - grounding_mode.rs::select_grounding_mode + the D-11 checker dispatch (the grounding methods — COMPOSE, never re-implement)
  - citation_prebind.rs (H-53 closed-set membership); H-32 validate-citation; the DKG query surface
  - claim/mod.rs::ClaimSet (L-1); claim/risk.rs::PredictorSignal (L-2)
reachable_from:
  - claim/risk.rs::score_claim_risk > round_trip_residual [PRODUCTION P1 predictor, via the L-2 cost-gate]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO re-implementation of any grounding method or the citation pre-binder (GD-1 P0)
  - NO generative paraphrase as the "round-trip" (HD-2 — exact evidence reconstruction, not a re-statement)
  - NO model call to judge reconstruction (TD-3 — the grounding methods are deterministic)
  - NO .unwrap on prod; blake3 not sha2
frontend_pair:
  N/A with justification — feeds L-2's risk tile; no standalone surface. Justified.
observable_effect:
  trigger: "round-trip a ClaimSet with a claim citing a real symbol at a real line + a claim citing a nonexistent symbol (via L-2's cost-gate)"
  expect_ssb: "prefer-reuse the D-10 grounding-verification rows (FILE/SOURCE/CosmosCheck); net 0 expected from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the real-symbol claim scores low residual (reconstructed); the nonexistent-symbol claim scores high residual AND its [N] is rejected by the H-53 closed set (NEGATIVE: an out-of-evidence citation cannot be emitted as supported)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md --run-trigger
rg "residual|round_trip|RoundTrip" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs   # P1 present
rg -ci "generate|paraphrase|summari" .elai_cc/crates/orchestrator/src/claim/roundtrip.rs # MUST be 0 (exact reconstruction)
test -f workflow/reasoning/L-3-decisions.md
```

## ACCEPTANCE
- [ ] HG-3: round-trip uses repo/source/DKG evidence, no model internals.
- [ ] GD-1: the D-10 grounding methods + H-53 + H-32 are consumed, not re-implemented.
- [ ] HD-2: the round-trip is exact evidence reconstruction, never a generative paraphrase.
- [ ] TD-3: reconstruction is deterministic; no model judges it.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live cost-gate emits a P1 residual that flips a citation to refusal on a fake symbol (not cargo-green).
- W-2: no flag (a predictor; the gate flag is L-7).
- W-3: activates BUILT grounding methods, not a summary-only port.
- W-4: frontend_pair N/A (feeds L-2); justified.
- W-5/W-10: FILE/SYMBOL legs in-process; CosmosCheck/WEB legs network-touching (the L-10 fixture tier covers them deterministically).
- W-7: grounding_mode.rs:157 + citation_prebind.rs re-grepped at edit.
- W-9: SSB prefer-reuse the grounding-verification rows; net 0.
- GD-2: the residual is recomputable from the per-method pass/fail under ELAI_SEED.

## Tests to add
- crates/contracts/tests/l3_round_trip_contract.rs (~70 LOC): (a) a claim citing a real symbol/line scores low residual; (b) a nonexistent symbol scores high residual; (c) an out-of-closed-set citation is rejected (H-53); (d) the round-trip uses no model call (mock asserts none); (e) an invented dependency fails the API_REGISTRY leg; (f) the residual is recomputable from the recorded per-method results.

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/roundtrip.rs (~110 LOC)
- MODIFY: claim/risk.rs (~20 LOC: wire P1 into the cost-gate)
- CREATE: crates/contracts/tests/l3_round_trip_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/L-3-decisions.md

## Citations / Grounding
- arXiv:2606.27326 §4.2 predictor 1 (round-trip residual) [EXTERNAL/DESIGN-RATIONALE]; cgg-hallucination-causal-system-2026-06-27.md S2.
- ELAI ground truth (L-CGG-GROUNDING-MAP TS-1/TS-2): grounding_mode.rs:157, citation_prebind.rs, H-32 validate-citation, the DKG (re-grepped at edit, W-7).
- CLAUDE.md: TD-3, HD-2 (no generative round-trip), no .unwrap, blake3-only.

## Open questions / risks
- WEB/CosmosCheck legs add latency + network; keep them in the cost-gated path (only on medium cheap-risk) and cover them deterministically in the L-10 fixture tier so the bench stays reproducible.
- Applicable-methods denominator: a claim with zero applicable methods (e.g. a pure opinion) must score neutral, not falsely low/high residual — decide the neutral handling at authoring.

## Next
L-3 is the cheap first predictor that gates the expensive ones. L-4 adds the orthogonal inter-seed semantic-entropy signal; L-5 adds the interventional do-test. L-2 fuses all three under HG-2.

End of L-3.
