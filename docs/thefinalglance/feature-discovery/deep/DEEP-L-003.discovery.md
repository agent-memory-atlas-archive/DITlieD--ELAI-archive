# DEEP-L-003 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1179-L1193`, heading `### DEEP-L-003`; the next chunk starts at L1195.
- Declared chunk: family L, **9 files**, **981 logical lines**, **101,509 bytes**, all nine classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly for the eight non-L-5 paths. L-5 was read from the recovered pinned image (not the live L-plan path); its sha256/line/byte values match the manifest row exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- **L-5 provenance (binding):** L-5 read from the recovered pinned image at `docs/thefinalglance/feature-discovery/_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md` (hash-verified equal to the manifest row); the live L-plan file is newer (272 ln / 7d13e7a1) and was NOT used. All L-5 anchors use `L-5:Lx-Ly` against that pinned image only.
- Read boundary: all nine listed originals (L-5 via recovered pin) were read completely, first through last logical line. No original assigned to another deep chunk was opened. `12-L-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `L-3:Lx-Ly`, `L-5:Lx-Ly` (pinned), `L-10:Lx-Ly`, `L-11:Lx-Ly`, `L-13:Lx-Ly`, `L-15:Lx-Ly`, `L-16:Lx-Ly`, `L-17:Lx-Ly`, and `L-20:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 100 | 9,164 | 28ebb08e8a43609440e155b85bcfd395eb0ee842469d93493406c0a6dd65eb98 | docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md | exact |
| numbered/index original | 103 | 11,568 | b254e847133d4d7cbb095f2f49dfa07d8cd73a8d0b3aff23238c125c595ea14f | docs/thefinalglance/feature-discovery/_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md (manifest path pin; live L-plan path NOT used) | exact (pinned image) |
| numbered/index original | 105 | 11,367 | abbfeceafb19d8e5990965bc9ef25146d4c094a63ed7ced77dd2ac3df4a2a47b | docs/thefinalglance/L-plan/L-10_cgg_hallucination_delta_bench.md | exact |
| numbered/index original | 121 | 15,291 | 23829927375fc942d71e8e98da5b2398c87dc8e2b9331be82744251c420b216a | docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md | exact |
| numbered/index original | 108 | 12,609 | f3a811ffb63919e3914bf7d9517b3d2d01bfbf3385f300e901a6ae2a3d2ded9e | docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md | exact |
| numbered/index original | 201 | 17,531 | c2b41ec4e57e9666348a96933f6a7f318310945ab43dbaf65bec58ecfb346121 | docs/thefinalglance/L-plan/L-15_repair_controller_and_evidence_first_reorder.md | exact |
| numbered/index original | 137 | 11,437 | c9b2613f249cc21b0dcebbb1fb82d05f876d65778371a86394874d601ff72e51 | docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md | exact |
| numbered/index original | 49 | 3,738 | 9432f36e365840e51762961d7b5dc72c218a168af9970523c798fc8ed9b0b774 | docs/thefinalglance/L-plan/L-17_freeze_epoch_controller_prefix_invariance.md | exact |
| numbered/index original | 57 | 8,804 | 5f0f0e227b907cf71ed0e85ea2b3aa5d2af2ab9cee0f3c0fa659378dec59cafd | docs/thefinalglance/L-plan/L-20_commit_mined_eval_supply_competitor_arm.md | exact |
| **Aggregate** | **981** | **101,509** | — | **9 exact paths (L-5 via recovered pin)** | **exact** |

Live L-5 out-of-scope observation (not used): `docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md` measures **272 lines / 34,153 bytes / sha256 `7d13e7a1d9182070e7bb460e29e0d5420f63a5d84fbd34c4c74f0f65f1142005`**.

## Per-file atomic ledgers

### L-3 — evidence round-trip predictor (P1)

Source: `docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md` (100/100 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L3-E001 | E/STATUS | L-3 is PLANNED WAVE 2, effort M, BUILD-on-shipped: activation of D-10 grounding methods over arbitrary claims. | L-3:L1-L3 |
| L3-E002 | E/CAP | For each L-1 ClaimSet typed claim, run shipped D-10/D-11 FILE/SOURCE/TEST/CosmosCheck plus H-53 closed-set citation membership and H-32 DKG validate-citation; emit P1 `PredictorSignal{RoundTrip, residual}` into L-2. | L-3:L5-L5, L27-L28 |
| L3-E003 | E/DATA | Residual formula: `residual = 1 - (reconstructing-methods / applicable-methods)`, blended with H-53 closed-set membership (out-of-set citation = max residual). | L-3:L28-L28 |
| L3-E004 | E/FLOW | Per-kind dispatcher maps ClaimKind to grounding method: File→FILE; Symbol→DKG/SOURCE; Dependency→API_REGISTRY; Api→SOURCE/compile; Behavior→TEST; Reachability→DKG call-graph; Causal→deferred to L-5. | L-3:L27-L27 |
| L3-E005 | E/CFG | L-3 is the CHEAP first predictor in L-2’s cost gate; expensive L-4/L-5 fire only when L-3 already flags medium residual (HG-2). | L-3:L13-L13, L29-L29 |
| L3-E006 | E/DEP | Pre-requisites (W-7 re-grep): `grounding_mode.rs:157` select_grounding_mode; D-11 checker registry; H-53 citation_prebind; H-32 validate-citation; DKG; L-1 ClaimSet; L-2 PredictorSignal. | L-3:L8-L8, L16-L19 |
| L3-E007 | E/STATUS | Existing: D-10/D-11 [WIRED/DARK] 6–8 methods binary pass/fail; H-53 [PARTIAL]; H-32 [PARTIAL] 28d TTL FactValidationFailed; DKG [WIRED]. | L-3:L15-L19 |
| L3-E008 | E/INT | Production path: `claim/risk.rs::score_claim_risk > round_trip_residual` via L-2 cost-gate. A `#[cfg(test)]`/contracts caller never satisfies R7. | L-3:L39-L41 |
| L3-E009 | E/CAP | Module `crates/orchestrator/src/claim/roundtrip.rs`: per-kind dispatcher + residual score + P1 PredictorSignal (~110 LOC). | L-3:L34-L34, L83-L83 |
| L3-E010 | E/LIMIT | Stub bans: no re-implementation of grounding methods or citation pre-binder (GD-1); no generative paraphrase as round-trip (HD-2); no model call to judge reconstruction (TD-3); no `.unwrap` on prod; blake3 not sha2. | L-3:L42-L46, L63-L66 |
| L3-E011 | E/SURFACE | frontend_pair N/A justified: feeds L-2 risk tile; no standalone surface. | L-3:L47-L48, L73-L73 |
| L3-E012 | E/OBS | Trigger: round-trip a ClaimSet with a real-symbol claim plus a nonexistent-symbol claim via L-2 cost-gate. Expect prefer-reuse D-10 grounding-verification rows; net 0 from LIVE ~268/577 (W-9). | L-3:L49-L51, L76-L76 |
| L3-E013 | E/FAIL | Real-symbol claim scores low residual; nonexistent-symbol scores high residual AND its `[N]` is rejected by H-53 closed set. Negative: out-of-evidence citation cannot be emitted as supported. | L-3:L52-L52 |
| L3-E014 | E/TEST | Contract `l3_round_trip_contract.rs` (~70 LOC): (a) real symbol/line low residual; (b) nonexistent high residual; (c) out-of-closed-set rejected; (d) no model call; (e) invented dependency fails API_REGISTRY; (f) residual recomputable from per-method results. | L-3:L79-L80 |
| L3-E015 | E/DET | GD-2: residual recomputable from per-method pass/fail under ELAI_SEED. | L-3:L77-L77 |
| L3-E016 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check.py --run-trigger; rg residual/round_trip/RoundTrip present; rg generate\|paraphrase\|summari MUST be 0; L-3-decisions.md exists. | L-3:L54-L60 |
| L3-E017 | E/FLOW | W-1 done = live cost-gate emits P1 residual that flips a citation to refusal on a fake symbol (not cargo-green). W-2: no flag (predictor; gate flag is L-7). | L-3:L70-L71 |
| L3-E018 | E/DATA | Files: CREATE roundtrip.rs; MODIFY claim/risk.rs (~20 LOC cost-gate wire); CREATE l3_round_trip_contract.rs; CREATE workflow/reasoning/L-3-decisions.md. | L-3:L82-L86 |
| L3-E019 | E/EVIDENCE | Paper arXiv:2606.27326 §4.2 predictor 1 is EXTERNAL/DESIGN-RATIONALE; L-CGG-GROUNDING-MAP TS-1/TS-2 anchors grounding_mode.rs:157, citation_prebind, H-32, DKG. | L-3:L10-L10, L88-L91 |
| L3-E020 | E/DEFER | Causal ClaimKind dispatch deferred to L-5. WEB/CosmosCheck legs stay cost-gated (medium cheap-risk only) and covered by L-10 fixture tier. | L-3:L27-L27, L94-L94 |
| L3-E021 | E/CONC | Parallel-safe with L-4 and L-5 (disjoint files; all three feed L-2). | L-3:L9-L9 |
| L3-E022 | E/LIMIT | W-5/W-10: FILE/SYMBOL legs in-process; CosmosCheck/WEB network-touching. No new dependency. cargo check --workspace GREEN; auditor PASS -p orchestrator. | L-3:L67-L67, L74-L74 |
| L3-E023 | E/CAP | HG-3: round-trip uses repo/source/DKG evidence, no model internals. | L-3:L5-L5, L63-L63 |
| L3-E024 | E/STATUS | Existing methods are binary pass/fail MiniCheck-style; L-3 residual is the inverse of how many methods reconstruct the claim. | L-3:L13-L13, L16-L16 |
| L3-I001 | I/FLOW | Residual “blended” with H-53 implies closed-set failure forces max residual regardless of other method pass rates; exact blend operator is not formalized beyond that clause. | L-3:L28-L28 |
| L3-A001 | A/CFG | Applicable-methods denominator: claim with zero applicable methods (pure opinion) must score neutral, not falsely low/high — decide neutral handling at authoring. | L-3:L95-L95 |
| L3-A002 | A/OBS | Exact LIVE baseline numbers (~268/577) are dated snapshots requiring W-9 re-grep; not fixed constants. | L-3:L51-L51 |

Category coverage: capabilities/predictor ✓; workflows/states ✓; residual schema ✓; cost-gate config ✓; integrations/events ✓; no standalone UI ✓; failures/refusal ✓; determinism ✓; observability ✓; tests/done gates ✓; constraints ✓; dependencies/deferred ✓. No retry/idempotency/privacy API named.

### L-5 — counterfactual do-test on model claims [KEYSTONE]

Source (PINNED image only): `docs/thefinalglance/feature-discovery/_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md` (103/103 lines). Manifest pin path is the L-plan filename; extraction anchors against this recovered image. Live L-plan L-5 was NOT read for atoms.

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L5-E001 | E/STATUS | L-5 is PLANNED WAVE 2, effort M-L, EXTENSION(F-4/C-28/C-29), KEYSTONE — causal core / un-forgeable signal (CANT-LIE). | L-5:L1-L3 |
| L5-E002 | E/CAP | On Causal/Behavior claims, derive and RUN an intervention (revert-only-A / disable-flag / patch-only-fn / add-trace), then compare F-4 cant-lie outcome and/or C-28/I-20 mutation and/or C-29 metamorphic relation WITH vs WITHOUT the intervention. | L-5:L5-L5, L28-L29 |
| L5-E003 | E/STATE | Label `causal_supported` TRUE only when the interventional outcome moves as claimed (HG-1). Static/temporal/import-edge evidence is scored at weaker tier and NEVER called causal (LS-2 fake-precision guard). | L-5:L5-L5, L30-L30, L65-L65 |
| L5-E004 | E/DATA | Emit P-counterfactual `PredictorSignal{Counterfactual, moved: bool, delta}` into L-2. | L-5:L5-L5, L29-L29 |
| L5-E005 | E/FLOW | Intervention planner (~60 LOC): deterministic intervention from claim refs + J-16 cone. NO model call to design the intervention (TD-3). | L-5:L28-L28 |
| L5-E006 | E/FLOW | Do-test runner (~70 LOC): run F-4 (or C-28/C-29) WITH and WITHOUT intervention; compare blake3-bound outcomes. | L-5:L29-L29 |
| L5-E007 | E/FLOW | Tier guard (~30 LOC): non-interventional evidence capped; non-snapshottable claims scored at runtime-trace tier with limit named (GD-5). | L-5:L13-L13, L30-L30, L68-L68 |
| L5-E008 | E/CFG | Wire into L-2 cost-gated predictor as the MOST EXPENSIVE predictor: runs last, only on high cheap+P1+P3 risk (~20 LOC). Default OBSERVE-only (GD-4). | L-5:L31-L31, L98-L98 |
| L5-E009 | E/DEP | Pre-requisites (W-7): F-4 AntiFabricationGate.check() task_runner.rs:9565 blake3 stdout_hash; C-28 + I-20 MutationBudgetedRun; C-29 IntentPair+MetamorphicRelation; CSQ phase-2D; J-16 blast_radius.rs; G-1 taint; L-1 ClaimSet; L-2 PredictorSignal. Depends L-1. Parallel-safe L-3, L-4. | L-5:L8-L9 |
| L5-E010 | E/STATUS | F-4 [WIRED]; C-28 [PARTIAL] + I-20 [WIRED]; C-29 [WIRED]; CSQ [WIRED]; J-16 [PLANNED]. | L-5:L15-L20 |
| L5-E011 | E/SEC | F-4 stdout-hash is captured by the orchestrator, not the worker (wiring R8); model cannot forge the outcome. | L-5:L13-L13, L16-L16, L66-L66 |
| L5-E012 | E/CAP | Module `crates/orchestrator/src/claim/counterfactual.rs`: intervention planner + do-test runner + tier guard + P-counterfactual (~160 LOC). | L-5:L36-L36, L86-L86 |
| L5-E013 | E/INT | Production path: `claim/risk.rs::score_claim_risk > counterfactual_probe` (L-2 cost-gate, only high cheap+P1+P3). Test/contracts caller never R7. | L-5:L41-L43 |
| L5-E014 | E/LIMIT | Stub bans: no causal label without interventional/executable pass (HG-1); no model call to design intervention OR judge outcome (TD-3); no re-implementation of F-4/mutation/metamorphic (GD-1); no `.unwrap`; blake3 not sha2. | L-5:L44-L48 |
| L5-E015 | E/SURFACE | frontend_pair N/A justified: feeds L-2 risk tile; underlying F-4/mutation rows already surface. | L-5:L49-L50, L76-L76 |
| L5-E016 | E/EVENT | expect_ssb: prefer-reuse F-4 TestIntegrityEvent / MutationBudgetedRun / MetamorphicRelation; else +CounterfactualProbed (Pattern-2; Kind: OutcomeMoved/OutcomeUnmoved/NotInterventional); baseline+0..1 from LIVE ~268/577 (W-9). | L-5:L53-L53 |
| L5-E017 | E/OBS | Trigger: do-test a “this patch fixes the timeout” claim (patch-only intervention, real F-4 path) plus an “A causes B” claim where reverting A does NOT change B. | L-5:L52-L52 |
| L5-E018 | E/FAIL | Fix-claim is causal-supported ONLY if F-4-bound outcome flips; “A causes B” with no B change is NOT-causal. Negative: “A imports B” alone scores static-tier, never causal (HG-1). | L-5:L54-L54 |
| L5-E019 | E/TEST | Contract `l5_counterfactual_contract.rs` (~90 LOC): (a) true fix flips → causal-supported; (b) false fix unchanged → NOT-causal; (c) revert A leaves B → NOT-causal; (d) import-only static-tier never causal; (e) intervention deterministic no model; (f) forged passed=true rejected by F-4 hash; (g) non-snapshottable tier-capped runtime-trace with limit recorded. | L-5:L82-L83 |
| L5-E020 | E/FLOW | W-1 done = live do-test flips a fix-claim’s causal label by re-running F-4 under intervention (not cargo-green). W-2: no flag (predictor; gate flag L-7). | L-5:L72-L73 |
| L5-E021 | E/DET | GD-2: moved/unmoved verdict recomputable from the two F-4-bound outcomes under ELAI_SEED. | L-5:L79-L79 |
| L5-E022 | E/EVIDENCE | Paper action-marginalization / action-shuffle ratio ≤1.1 = action ignored (§4.1.ii/§5) is EXTERNAL/DESIGN-RATIONALE; elai-lang counterfactual screenshot; cgg S5; MAP NN-4/TS-7/8/9. | L-5:L10-L10, L91-L94 |
| L5-E023 | E/DATA | Files: CREATE counterfactual.rs; MODIFY claim/risk.rs (~20 LOC wire P-counterfactual + causal_supported); CREATE l5_counterfactual_contract.rs; CREATE L-5-decisions.md. | L-5:L85-L89 |
| L5-E024 | E/CFG | W-5/W-10: in-process where repro is local; RunPod-gated where claim’s repro needs live model. | L-5:L7-L7, L77-L77 |
| L5-E025 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg intervention\|counterfactual\|moved present; rg -ci causal present (read the guard); L-5-decisions.md. | L-5:L56-L62 |
| L5-E026 | E/SEC | Intervention SAFETY: run inside I-2 COW worktree / Tier-C isolation, never on live tree; never apply a destructive intervention (do-test observes, does not ship the change). | L-5:L97-L97 |
| L5-E027 | E/CFG | Cost: budget via J-84; cover deterministically in L-10 fixture tier. | L-5:L98-L98 |
| L5-E028 | E/FLOW | With L-3/L-4/L-5 feeding L-2, convergence fuse is complete; L-6 acts on High tier; L-7 turns Critical into block/abstain. | L-5:L100-L101 |
| L5-E029 | E/LIMIT | cargo check --workspace GREEN; auditor PASS -p orchestrator; no new dependency. | L-5:L69-L69 |
| L5-E030 | E/DEP | Consumes CSQ (phase-2D counterfactual safety questions on fs/net/money/auth) as composition surface alongside mutation/metamorphic. | L-5:L19-L19, L39-L39 |
| L5-I001 | I/STATE | `causal_supported` lives on L-2 ClaimRisk surface (risk.rs), set by the counterfactual probe result rather than by L-5 alone. | L-5:L29-L29, L40-L40 |
| L5-A001 | A/SEC | Exact isolation seam (I-2 COW vs Tier-C) is deferred to authoring; not fixed in this plan. | L-5:L97-L97 |
| L5-A002 | A/CFG | Exact “high cheap+P1+P3” threshold values and J-84 budget numbers are not specified here. | L-5:L31-L31, L98-L98 |
| L5-A003 | A/EVENT | Whether +CounterfactualProbed is required vs full prefer-reuse of F-4/mutation rows is decide-at-edit (baseline+0..1). | L-5:L53-L53 |

Category coverage: intervention capability ✓; surfaces (none standalone) ✓; planner/runner/tier-guard flow ✓; Intervention/DoTestVerdict data ✓; production wiring ✓; security/isolation ✓; failures/abort ✓; determinism ✓; tests ✓; constraints/out-of-scope ✓; deferred live execution/isolation ✓. No privacy/retry API beyond cost-gate ordering. **Pinned image only — no live recon-fold content.**

### L-10 — CGG hallucination-delta bench

Source: `docs/thefinalglance/L-plan/L-10_cgg_hallucination_delta_bench.md` (105/105 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L10-E001 | E/STATUS | L-10 is PLANNED WAVE 4, effort M, BUILD-instrument, BENCH-PENDING for live-model arm (RunPod-gated W-10). Whole-plan done-gate for L. | L-10:L1-L3, L5-L5 |
| L10-E002 | E/SURFACE | CLI: `elai bench hallucination-delta [--scope fixture\|live] [--seed] [--json]`. | L-10:L5-L5, L41-L41, L51-L51 |
| L10-E003 | E/CAP | Gate-ON-vs-OFF ablation over held-out claim corpus: unsupported-claim rate, contradicted-claim rate, reviewer-reaching rate; also over-block rate (benign claim wrongly blocked). | L-10:L5-L5, L13-L13, L29-L29 |
| L10-E004 | E/DET | Under ELAI_SEED, journal-recomputable (GD-2); receipts on y-axis never pass@k (GD-3); deterministic scoring, NO LLM-judge (TD-3). | L-10:L5-L5, L45-L45 |
| L10-E005 | E/CFG | Deterministic FIXTURE tier ships now; live-model arm RunPod/free-pool-gated and refuses to emit a live number until the arm runs (no fabricated delta, ID-5). | L-10:L5-L5, L30-L30, L47-L47, L68-L68 |
| L10-E006 | E/OBS | Report “N fewer unsupported claims per 100 model outputs”, never a leaderboard rank; ELAI-measured gold set, never paper AUROC/rho (ID-5). | L-10:L5-L5, L64-L65 |
| L10-E007 | E/FLOW | CGG ships default-ON (L-7) only after this delta is positive AND over-block/FP rate is bounded (GD-4). | L-10:L5-L5, L81-L81 |
| L10-E008 | E/DEP | Compose G-6 safedelta_runner.rs shape (args_bench.rs:75); I-38 QDELTA ≥5 seeds paired; I-27 bench_convergence bootstrap-CI; E-54 eval corpus; L-1..L-7 under measurement. Do NOT fork SAFEDELTA/QDELTA. | L-10:L8-L8, L16-L20, L39-L39, L47-L47 |
| L10-E009 | E/CAP | Runner `benchmarks/hallucination_delta_runner.rs` + hallucination-delta bench sub (~130+25 LOC). | L-10:L36-L37, L87-L88 |
| L10-E010 | E/DATA | Held-out claim gold set in `data/claim_gold/` per-family shards via E-54 contamination-filtered; labels ground-truth (human/executable oracle), not LLM-judge. | L-10:L28-L28, L89-L89 |
| L10-E011 | E/CFG | Calibration write-back: measured thresholds (tier boundaries, entropy cutoff, BLOCK-by-default promotion) write to L-2/L-4/L-7 config (ID-5). | L-10:L31-L31, L13-L13 |
| L10-E012 | E/TEST | Stats floor: ≥5 seeds paired (I-38), bootstrap-CI (bench_convergence). | L-10:L29-L29, L67-L67 |
| L10-E013 | E/FAIL | Negative: `--gate-off-but-claim-savings` broken arm shows delta COLLAPSE (gate is the cause). Live arm refuses number until RunPod lands. | L-10:L53-L53, L84-L84 |
| L10-E014 | E/EVENT | prefer-reuse SAFEDELTA/QDELTA/ConvergenceCurveMeasured (+Kind HallucinationDeltaArm); baseline+0..1 (W-9). | L-10:L52-L52 |
| L10-E015 | E/SURFACE | frontend_pair: hallucination-delta tile on audit/cost surface (decide at edit) OR N/A via existing bench/convergence panels. | L-10:L48-L49, L76-L76 |
| L10-E016 | E/TEST | Contract `l10_hallucination_delta_contract.rs` (~75 LOC): fixture ON/OFF recomputable; no LLM-judge; collapse arm; live refuse; ≥5 seeds; calibration write-back. | L-10:L83-L84 |
| L10-E017 | E/LIMIT | Stub bans: no LLM-judge; no paper AUROC/rho as done-gate; no fabricated live delta; no runner fork; no `.unwrap`; blake3 not sha2. | L-10:L43-L47 |
| L10-E018 | E/INT | Production: `elai bench hallucination-delta`; test/contracts never R7. W-1 done = fixture scope emits recomputable ON/OFF delta. W-2: no behavior gate (bench; flag is L-7). | L-10:L40-L42, L73-L74 |
| L10-E019 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg hallucination.?delta\|gate_on\|gate_off\|over.?block; rg llm.?judge\|gpt.?score\|model.*grade MUST 0; L-10-decisions.md. | L-10:L55-L61 |
| L10-E020 | E/EVIDENCE | Paper rho≈0.80/AUROC EXTERNAL only; CGG’s own number measured here. | L-10:L93-L94 |
| L10-E021 | E/DEFER | Live-model arm BENCH-PENDING. Parallel-safe with L-9. | L-10:L9-L9, L77-L77 |
| L10-E022 | E/DATA | Files: CREATE hallucination_delta_runner.rs; MODIFY args_bench.rs; CREATE data/claim_gold/; CREATE l10 contract; CREATE L-10-decisions.md. | L-10:L86-L91 |
| L10-E023 | E/OBS | Over-block weighting: report BOTH unsupported-reduction AND over-block friction; never just reduction. | L-10:L100-L100 |
| L10-I001 | I/DEP | Label provenance for behavior/causal claims may use L-5 do-test outcome as oracle (open question states this). | L-10:L99-L99 |
| L10-A001 | A/DATA | Exact label provenance per ClaimKind decided at authoring (human-curated vs executable oracle). | L-10:L99-L99 |
| L10-A002 | A/CFG | Exact over-block acceptance bounds for L-7 promotion not numeric here. | L-10:L100-L100 |
| L10-A003 | A/SURFACE | Whether a dedicated delta tile ships is edit-time W-4 choice. | L-10:L48-L49 |

Category coverage: CLI bench surface ✓; ON/OFF flow ✓; corpus data ✓; calibration config ✓; events ✓; fail-closed live number ✓; determinism/stats ✓; tests ✓; constraints ✓; deferred live arm ✓. No privacy control.

### L-11 — output-quality enforcement system [KEYSTONE]

Source: `docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md` (121/121 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L11-E001 | E/STATUS | L-11 is PLANNED WAVE 1 of L-T2, effort L, BUILD+COMPOSE, KEYSTONE output-quality spine. Applies to EVERY backend (cloud Sonnet, local Qwen-7B, BYOK, free-pool, CLI-bridge). | L-11:L1-L5 |
| L11-E002 | E/CAP | Four seams: (1) plan-time C-31 FilePlan responsibility budget; (2) gen-time C-32 MiniCritic METRIC + StructuralRetry; (3) write-time VibeGate F-1 MR-1/MR-2 floor; (4) audit-time solid_proxy + maintainability + optional power_of_ten + J-23 generated lane. | L-11:L5-L5, L29-L35 |
| L11-E003 | E/SEC | OQ-1: authoritative VERDICT is AUDITOR-owned and frozen (R8); gen-time MiniCritic is self-CORRECTION (retry), not the verdict. | L-11:L5-L5, L13-L13, L78-L78 |
| L11-E004 | E/SURFACE | Net-new checker `.claude/tools/solid_proxy_check.py` (~180 LOC, stdlib-only): CLI `--files/--base HEAD/--json`; exit 1 on BLOCK / 2 on scan-config error; tags C-MAINTAINABILITY-SRP/-DIP/-ISP. | L-11:L5-L5, L30-L30, L69-L70 |
| L11-E005 | E/CAP | SRP god-object BLOCK class: `(fan-out > K_FANOUT AND (method-count > K_METHODS OR lines > K_LINES))`. DIP WARN: downstream concrete TYPE_REF to upstream impl where trait exists. ISP WARN: trait with >K_TRAIT_METHODS where median impl uses < half. OCP/LSP NOT covered (advisory residue in --json). | L-11:L30-L30, L80-L80 |
| L11-E006 | E/CAP | NN-2 god-object MiniCritic METRIC (~50 LOC): fan-out + method-count of filled symbol → WARN/BLOCK so StructuralRetry re-rolls; pair with J-55. Default WARN (observe-only) until FP bound clears. | L-11:L31-L31 |
| L11-E007 | E/CFG | SEAM 1: FileSpec `max_distinct_concerns` default generous + observe-only; no schema bump (serde default). | L-11:L32-L32 |
| L11-E008 | E/CFG | SEAM 3: MR-1/MR-2 through PreToolUse as WARN; BLOCK only on MR-1 crossing / MR-2 BLOCK class. SOLID is NOT a hard PreToolUse block (OQ-2, thrash guard A-1). | L-11:L33-L33, L79-L79 |
| L11-E009 | E/SEC | SEAM 4: auditor runs solid_proxy + maintainability + (when present) power_of_ten over changeset AND J-23 over GENERATED code; freeze solid_proxy_check.py in gate-manifest.txt (R8). | L-11:L34-L34, L94-L94 |
| L11-E010 | E/STATE | Promotion: SOLID proxies register as rules-engine 3F SHADOW; promote god-object to BLOCK-default only on fpr<5%/tpr>80% over ≥3 real runs (OQ-3/GD-4/J-43). DIP/ISP stay WARN; OCP/LSP stay J-89 advisory. | L-11:L35-L35, L80-L80 |
| L11-E011 | E/SURFACE | Compose I-38 QDELTA `--quality` arm (gate-ON-vs-OFF maintainability/SOLID-violation rate); do NOT fork runner. Thresholds ELAI-measured never SOLID/P10 paper constants (ID-5). | L-11:L36-L36, L83-L83 |
| L11-E012 | E/DEP | Compose C-31/C-32/maintainability_check.py/J-65/J-23/B-29/elai.map/rules-3F/VibeGate-F-1; re-implementing any is P0 (GD-1). | L-11:L5-L5, L8-L8, L58-L58 |
| L11-E013 | E/INT | Production reachability: auditor changeset → solid_proxy; C-32 baby-step → MiniCritic METRIC; VibeGate PreToolUse → MR floor. Test/contracts never R7. | L-11:L48-L52 |
| L11-E014 | E/LIMIT | Stub bans: no executor-self verdict (OQ-1); no hard PreToolUse SOLID block (OQ-2); no god-object BLOCK-by-default before FP bound (OQ-3); no deterministic OCP/LSP (OQ-4); no leaf-name-grep call graph (R5 — use B-29/SCIP); no paper thresholds (ID-5). | L-11:L53-L58 |
| L11-E015 | E/SURFACE | frontend_pair: quality-verdict tile on vibesec/audit panel (GatesPanel precedent; decide at edit) OR N/A justified via existing audit surface. | L-11:L59-L60, L90-L90 |
| L11-E016 | E/EVENT | reuse mini_critic_verdict / maintainability row OR +1 OutputQualityVerdict (Pattern-2; Kind SrpBlock/DipWarn/IspWarn/PassClean); baseline+0..1. | L-11:L63-L63 |
| L11-E017 | E/OBS | Trigger: auditor over 1200-line god-module AND clean 200-line module. God-module C-MAINTAINABILITY-SRP (BLOCK in ACTIVE, WARN in SHADOW); clean passes; weak-model god-object retries via StructuralRetry (OQ-5). | L-11:L61-L64 |
| L11-E018 | E/TEST | Contract `l11_output_quality_contract.rs` (~100 LOC): MiniCritic flags+retry; clean pass; C-31 over-concern reject; executor-context verdict rejected; thresholds config/measured not literals. | L-11:L97-L98 |
| L11-E019 | E/TEST | solid_proxy fixtures: god-object .rs exit 1 SRP; clean exit 0; cross-layer-concrete DIP WARN; OCP/LSP case NO block residue. | L-11:L99-L99 |
| L11-E020 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; solid_proxy on god/clean; rg solid_proxy in gate-manifest; rg ocp\|lsp.*block MUST 0; L-11-decisions.md. | L-11:L66-L74 |
| L11-E021 | E/FLOW | W-1 done = auditor seam actually runs solid_proxy over real changeset. W-2: kill-switch not enable-flag — SHADOW lifecycle-gated, no new default-OFF enable graveyard. | L-11:L88-L89 |
| L11-E022 | E/DATA | Files: CREATE solid_proxy_check.py+fixtures; MODIFY mini_critic.rs, file_plan.rs FileSpec, VibeGate PreToolUse, auditor seam, rules-engine seed rows, gate-manifest; MODIFY I-38 --quality arm; CREATE l11 contract; CREATE L-11-decisions.md. | L-11:L101-L106 |
| L11-E023 | E/DEFER | If FP stays high, god-object stays WARN permanently (A-1 thrash guard). Generated TS/Svelte/Python: SOLID-proxy Rust-strongest; J-23 does real work (GD-5 residue). | L-11:L115-L116 |
| L11-E024 | E/CFG | Fan-out/method-count thresholds ELAI-measured at L-10/I-38, seeded conservatively SHADOW-first; decide seed at authoring. | L-11:L114-L114 |
| L11-E025 | E/DET | GD-2: every flag recomputable from changeset under ELAI_SEED. | L-11:L95-L95 |
| L11-E026 | E/CONC | Parallel-safe with L-12/L-13/L-14 (disjoint file scope). | L-11:L9-L9 |
| L11-E027 | E/LIMIT | No new dependency (stdlib-only checker). cargo check GREEN; auditor PASS. | L-11:L84-L84 |
| L11-E028 | E/CAP | OQ-5 any-model leg: generated weak-model god-object retries in-loop via MiniCritic METRIC + StructuralRetry. | L-11:L81-L81 |
| L11-I001 | I/CFG | K_FANOUT/K_METHODS/K_LINES/K_TRAIT_METHODS are named symbols without numeric defaults in this plan; filled by measurement. | L-11:L30-L30, L114-L114 |
| L11-A001 | A/CFG | Exact seed values for K_* and max_distinct_concerns decided at authoring. | L-11:L32-L32, L114-L114 |
| L11-A002 | A/SURFACE | Quality-verdict tile vs N/A is edit-time W-4. | L-11:L59-L60 |

Category coverage: four-seam quality ✓; CLI checker surface ✓; MiniCritic gen-time surface ✓; audit panel tile optional ✓; lifecycle SHADOW→ACTIVE ✓; security OQ-1/R8 ✓; failures/FP ✓; tests ✓; deferred thresholds ✓.

### L-13 — user-authorable rules & hooks surface

Source: `docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md` (108/108 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L13-E001 | E/STATUS | L-13 is PLANNED WAVE 2 of L-T2, effort M, ACTIVATION+COMPOSE. Exposes wired rules engine/hooks as user-authorable DATA. | L-13:L1-L5 |
| L13-E002 | E/CAP | User rule = DATA: predicate (AST/GREP/SCHEMA/METRIC/COMPOSITE) + scope (path glob / tool / FSM state) + verdict (ALLOW/WARN/BLOCK/ESCALATE); never executable code (FS-2). | L-13:L5-L5, L29-L29 |
| L13-E003 | E/SURFACE | CLI verbs: `elai rule add`, `list`, `test`, `promote`, `retire`. | L-13:L5-L5, L33-L33, L38-L38 |
| L13-E004 | E/STATE | New user rules write lifecycle=SHADOW (observe-only); promote to ACTIVE/BLOCK only on engine fpr<5%/tpr>80% over user’s own runs (GD-4). | L-13:L5-L5, L31-L31, L71-L71 |
| L13-E005 | E/FLOW | Hook axes: PreToolUse/PostToolUse/Stop/PreCompact via J-52 path-keyed injection into context_budget Rules slot. SubagentStop only if J-52 leg shipped; else PostToolUse + residue. | L-13:L17-L17, L30-L30, L101-L101 |
| L13-E006 | E/SEC | Admission: J-102 (blake3 manifest, fail-closed) + content scan (dangerous-verb/injection-phrase, F-1/J-102 legs) before live. | L-13:L31-L31, L69-L69 |
| L13-E007 | E/SEC | Reject rule that targets mutable=0 immune-tier check to WEAKEN it (R8). User rules ADD checks only, never weaken frozen ones. Re-define of immune check id must be rejected, not silently shadowed. | L-13:L13-L13, L29-L29, L70-L70, L103-L103 |
| L13-E008 | E/SURFACE | RulesDrawer.svelte / ToolsHooksDrawer.svelte authoring: add-rule form + SHADOW fire-rate + promote/retire; cross_lang_diff PASS or justified N/A. | L-13:L33-L33, L52-L53, L80-L80 |
| L13-E009 | E/STATUS | Existing: RULES ENGINE 3F [WIRED] predicates+verdicts+lifecycle PROPOSED→…→RETIRED, mutable=0, ≥33 seeds; HOOK SYSTEM T1/T2/T3 [WIRED] agents never see hooks; J-52/J-27/J-102 [PLANNED]; drawers [WIRED]. | L-13:L15-L21 |
| L13-E010 | E/INT | Production: CLI/dashboard → elai rule add → J-102 admission → rules-engine SHADOW row; hook dispatch fires SHADOW observe / ACTIVE block. Test/contracts never R7. | L-13:L43-L46 |
| L13-E011 | E/LIMIT | Stub bans: no executable user rule (FS-2); no immune weaken (R8); no live before admission+scan PASS; no block-by-default before FP bound (GD-4); no LLM authoring/approving verdict (TD-3); no re-impl engine/hooks/J-52 (GD-1); blake3 not sha2. | L-13:L47-L51 |
| L13-E012 | E/OBS | Trigger: elai rule add GREP PreToolUse WARN on banned pattern; trigger matching edit; try add immune-weaken rule. Expect SHADOW would-WARN without block; immune weaken REJECTED; promotion refused until FP bound. | L-13:L54-L58 |
| L13-E013 | E/EVENT | reuse rules-engine rule-fired/lifecycle rows; +0..1 UserRuleRegistered Kind if needed; baseline+0..1. | L-13:L56-L56 |
| L13-E014 | E/TEST | Contract `l13_user_rules_contract.rs` (~90 LOC): SHADOW would-WARN; immune weaken rejected; executable content rejected; promotion refused until FP; fire recomputable from journal. | L-13:L86-L87 |
| L13-E015 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; rg rule_add\|user_rule\|register_rule non-test; rg exec\|eval\|shell\|Command::new on user-rule path MUST 0; L-13-decisions.md. | L-13:L59-L65 |
| L13-E016 | E/FLOW | W-1 done = user-added rule actually fires in live hook dispatch (SHADOW record). W-2: kill-switch on user-rule surface, not new enable graveyard. | L-13:L78-L79 |
| L13-E017 | E/DATA | Files: CREATE/MODIFY elai rule CLI arms (~150 LOC) + hook-axis registration (~50 LOC); MODIFY RulesDrawer/ToolsHooksDrawer + Tauri command pair; CREATE l13 contract; CREATE L-13-decisions.md. | L-13:L89-L93 |
| L13-E018 | E/DEFER | Soft-dep J-52 + J-102: fall back to direct rules-engine row + skillspector-class scan and record residue if not shipped. | L-13:L9-L9, L102-L102 |
| L13-E019 | E/DEFER | No SubagentStop axis inventing here if J-52 has not shipped it. | L-13:L101-L101 |
| L13-E020 | E/DEP | Same SHADOW→ACTIVE machine as L-11 SOLID proxies — one promotion path, two consumers. | L-13:L106-L106 |
| L13-E021 | E/LIMIT | In-process/harness; no model; no RunPod; no new dependency; gate-manifest unchanged (no immune weaken). | L-13:L7-L7, L74-L74, L84-L84 |
| L13-E022 | E/DET | GD-2: firings recomputable from journal. | L-13:L57-L57 |
| L13-E023 | E/SEC | Hooks remain IPC-level and agent-invisible (stronger than CC user-shell hooks). | L-13:L13-L13, L17-L17 |
| L13-I001 | I/FLOW | `elai rule test` against a fixture is the pre-promotion dry-run surface for FP measurement, though exact fixture format is not specified. | L-13:L33-L33 |
| L13-A001 | A/DATA | Exact rules table schema field names for user-authored rows beyond predicate/scope/verdict/lifecycle are delegated to the existing engine schema. | L-13:L29-L29 |
| L13-A002 | A/SURFACE | Exact drawer form field set decided at edit. | L-13:L33-L33 |

Category coverage: full user-facing CLI + drawer surfaces ✓; hook axes ✓; lifecycle/admission ✓; security R8/FS-2 ✓; tests ✓; deferred SubagentStop/J-52 ✓.

### L-15 — receipt-minting repair controller + evidence-first reorder [KEYSTONE]

Source: `docs/thefinalglance/L-plan/L-15_repair_controller_and_evidence_first_reorder.md` (201/201 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L15-E001 | E/STATUS | L-15 is BUILD KEYSTONE net-new loop object + binding REVISIONS to L-2/L-3/L-4/L-6/L-10 ordering. Wave W3.5 after L-2 fuse exists. Authored 2026-06-27 after CGG bench measured. | L-15:L1-L13 |
| L15-E002 | E/CAP | RepairController owns catch → ground → re-verify → commit-or-abstain: turns High/Critical ClaimRisk from STOP/abstain into bounded REPAIR that mints a passing external receipt before commit. | L-15:L3-L5, L45-L45 |
| L15-E003 | E/STATE | FSM S0–S6, every transition an SSB row (GD-2): S0 EXTRACT ClaimSet; S1 SCORE ClaimRisk + which predictor fired (P1/P2/P3/P-cf); S2 DECIDE High/Critical → ENTER REPAIR; S3 DIAGNOSE→REMEDY; S4 RE-VERIFY L-2; S5 GATE J-83; S6 COMMIT-AND-FREEZE. | L-15:L46-L71 |
| L15-E004 | E/FLOW | S3 remedy map: P1 residual → deterministic retrieval HippoRAG PPR + J-38 + J-103 + H-53 (NEVER model-driven nav, R3); P3 entropy → D-13 cross-family adversary + J-86 parallel candidates; Behavior/Causal → L-5 do-test or D-11 symbolic; Dependency → API_REGISTRY; unresolvable → I-7 escalation. | L-15:L54-L65 |
| L15-E005 | E/FLOW | S5: J-83 receipt PASSES → S6; else if budget remains → S3; else abstain (L-7 Critical → Abstain, credited by I-8/H-39). S6: J-83 commit-and-freeze; L-8 prunes disproven claim. | L-15:L67-L71 |
| L15-E006 | E/CFG | Budget+stop: J-84 sets effort (Low=ZERO; easy=serial depth; confident-consistent=parallel width+cross-family; very-hard=escalate-or-abstain). J-85 deterministic drift-stop. Three terminations only: receipt PASS \| budget exhausted \| J-85 drift. | L-15:L72-L75 |
| L15-E007 | E/SEC | TRUST: S5 commit gated on EXTERNAL receipt the model did NOT author (TD-3, RR-1, J-83 RD-1): L-3 reconstruction / L-5 F-4 blake3 / D-11 symbolic. No self-correction Reflexion loop. | L-15:L76-L80 |
| L15-E008 | E/EVENT | produces +RepairLoopRun (Pattern-2; Kind: ReceiptMinted/BudgetExhaustedAbstain/DriftAbstain) OR prefer-reuse L-6 RetrievalLoopEvent + L-7 HallucinationGateAction; net 0..+1. | L-15:L83-L85 |
| L15-E009 | E/INT | Production: L-7 policy-gate seam in task_runner — `RepairController.run(claim, risk)` replaces bare abstain. D-030 same-changeset wire; production_caller_check.py confirms. | L-15:L87-L90, L178-L179 |
| L15-E010 | E/CFG | Kill-switch `ELAI_CGG_REPAIR_ENABLE` default-OFF + wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id) — W-2 kill-switch-not-enable-flag. | L-15:L90-L92, L179-L179 |
| L15-E011 | E/SURFACE | Trigger: L-10 bench `--repair` arm drives real entrypoint on held-out claim corpus. No dedicated UI named. | L-15:L96-L96, L188-L188 |
| L15-E012 | E/LIMIT | Stub bans: no model-driven multi-hop nav in repair retrieval (R3); no model self-grading receipt (TD-3); no commit on absence-of-disagreement without passing evidence-receipt for receipt-available claim (R1). | L-15:L97-L99 |
| L15-E013 | E/DEP | R1 amends L-2: Low/Allow on receipt-available class A/B/C factual claim REQUIRES ≥1 PASSING evidence-grounded receipt (P1 or P-cf), not mere absence of resampling flag. | L-15:L104-L110 |
| L15-E014 | E/DEP | R2 reorders WAVE 2: land L-3 + L-5 BEFORE L-4. New W2 order: L-3+L-5 first, L-4 second (still parallel-safe disjoint files). | L-15:L112-L117 |
| L15-E015 | E/DEP | R3 constrains L-6: repair-retrieval MUST be deterministic BM25/HippoRAG PPR + atomic entity-extraction only; added as L-6 stub_ban. | L-15:L119-L124 |
| L15-E016 | E/DEP | R4 extends L-10: third arm gate+REPAIR vs gate-only vs off; measure H_emit AT FIXED COVERAGE (e.g. 90%). | L-15:L126-L131, L188-L188 |
| L15-E017 | E/EVIDENCE | R4 MEASURED MBPP mistral-small n=100: baseline 27% H_emit@100% cov \| abstain-only 9%@82% \| REPAIR 9%@91% — repair strictly dominates abstention. Residual 9% Goodhart gap. | L-15:L132-L137 |
| L15-E018 | E/EVIDENCE | Cross-domain: MATH-500 L5-hard got WORSE under self-consistency (64%→73%); TruthfulQA grounded-repair flipped 4/4 floor cases; D-13 4-family ensemble cut L5 64%→50%, gated 12.5%@62% cov. | L-15:L138-L151 |
| L15-E019 | E/DEP | R5 elevates L-9 to flagship showcase (math/symbolic receipt path depends on L-15 loop). | L-15:L153-L156 |
| L15-E020 | E/DATA | Receipt classes: A code executable; B math symbolic; C repo/cited retrieval; D parametric-recall-only NO receipt → abstention only. Domain router D-10 decides class. | L-15:L161-L173 |
| L15-E021 | E/TEST | Acceptance: (1) production_caller_check D-030 + kill-switch; (2) external-receipt-only commit unit test (self-say-so = P0); (3) three terminations each driven; (4) R3 deterministic retrieval proof; (5) R1 class A/B/C Low requires receipt; (6) L-10 --repair H_emit@90%; (7) GD-4 SHADOW first ≥3 runs; (8) GD-5 residual named + L-15-decisions.md + memory-writer. | L-15:L178-L191 |
| L15-E022 | E/STATE | GD-4: ships SHADOW (logs what it WOULD repair) until over-repair/cost rate bounded on ≥3 real runs; promotes behind kill-switch. | L-15:L189-L190 |
| L15-E023 | E/EVIDENCE | Origin measurements: PopQA A3 AUROC 0.83 > A2 0.81 > A1 0.79 > A0 0.50; detection+abstention ~75% cut hallucination at ~50% coverage cost; code A5 25%→9% at 92% coverage. Weak-model PopQA A3 0.906. | L-15:L17-L40 |
| L15-E024 | E/DEP | Composes L-2/L-6/L-7, J-83/J-84/J-85/J-86, F-4, D-11, D-13, I-7, HippoRAG, J-38, J-103, H-53, B-22, H-39. | L-15:L7-L10 |
| L15-E025 | E/LIMIT | 100%-cov-at-0%-halluc is epistemic impossibility; real target is 0% EMITTED via repair-where-receipt + abstain-on-residual. | L-15:L150-L151 |
| L15-E026 | E/OBS | Observable: hard claim with obtainable receipt → refused-then-repaired commits with passing receipt model did not author; unretrievable → honest-refusal abstain. | L-15:L93-L95 |
| L15-I001 | I/FLOW | S2 maps L-7 High→L-6 handoff into an explicit ENTER REPAIR that closes the loop L-6 only opened. | L-15:L51-L53 |
| L15-A001 | A/EVENT | Exact event family (new RepairLoopRun vs prefer-reuse L-6/L-7 rows) decided at edit. | L-15:L83-L85 |
| L15-A002 | A/CFG | Exact J-84 budget numbers and J-85 drift formula not numeric in this plan. | L-15:L72-L75 |

Category coverage: full repair FSM ✓; no dedicated UI (bench/flag surfaces) ✓; kill-switch surface ✓; L-10 --repair surface ✓; receipts/security ✓; binding revisions ✓; measured evidence ✓; tests ✓; residual honesty ✓. Summary-layer had title-only — this ledger fills it.

### L-16 — immutable transcript + pure fold-view context core [KEYSTONE]

Source: `docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md` (137/137 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L16-E001 | E/STATUS | L-16 is BUILD KEYSTONE, Track L-T3 CONTEXT-CORE Wave W1 foundation; binds CC-1..CC-4; binding REVISION replacing compact_session mutate-then-recover. | L-16:L1-L15 |
| L16-E002 | E/CAP | CC-1 RAW IS IMMUTABLE: no context-shaping may mutate/delete transcript log entries; log is single source of truth. | L-16:L41-L43 |
| L16-E003 | E/CAP | CC-2 VIEW IS PURE: provider-facing array is deterministic pure function `fold(log, budget, policy, freeze) -> view`; ambient/mutable state outside declared inputs is a violation. | L-16:L44-L47 |
| L16-E004 | E/CAP | CC-3 PREFIX MOVES ONLY ON A NAMED CAUSE (L-17 enumerated causes journaled); unexplained prefix-byte change is hard gate failure. | L-16:L48-L50 |
| L16-E005 | E/CAP | CC-4 LOSSLESS BY CONSTRUCTION: log never destroyed; lossless = view preserves K-12 critical atoms; archive-before-drop subsumed. | L-16:L51-L54 |
| L16-E006 | E/DATA | Promote SessionDag + H-49 CAS blake3+zstd as canonical append-only transcript; every ConversationMessage appended; CAS dedup bounds cost. | L-16:L58-L62 |
| L16-E007 | E/CAP | ContextProjection fold in `crates/orchestrator/src/context_projection.rs`: fold + View type + TranscriptLog read over SessionDag/CAS. | L-16:L63-L66, L101-L102 |
| L16-E008 | E/CFG | Strategies inside fold: E-25 ladder as budget strategies; H-6 masking as trailing-observation strategy (VIEW only, never log); 3E allocator per-slot budget; H-50 ContextPolicy by BackendMode (cloud=cache-stable prefix+late compaction; local=window-min+unload-early). | L-16:L66-L72, L86-L97 |
| L16-E009 | E/FLOW | Replace conversation.rs::compact_session mutation: runtime holds immutable log and asks fold() per turn. Compaction = steady-state projection at current budget. | L-16:L73-L76, L102-L103 |
| L16-E010 | E/RECOVERY | recover_stubs live call-site becomes “recompute view with larger budget for requested span” — no SSB round-trip, no blake3 verify (content never gone). recover_stubs stays for legacy DAG-snapshot reads. | L-16:L76-L78, L103-L104 |
| L16-E011 | E/DET | Structural HD-2: pure fold + immutable log prefix → view prefix cannot move between turns unless budget/policy/freeze change. | L-16:L79-L82 |
| L16-E012 | E/CFG | Default-ON behind kill-switch `ELAI_CONTEXT_PROJECTION_DISABLE`; mutate path stays available under flag for one release as rollback, then retired. | L-16:L109-L110, L123-L124 |
| L16-E013 | E/EVENT | prefer-reuse ContextReduction; optional +1 ContextProjected (Pattern-2; Kind ProjectedHotReuse/ProjectedRecomputed) if L-17 freeze audit needs it; re-grep SCHEMA_VERSION first. | L-16:L104-L107 |
| L16-E014 | E/SURFACE | Context gauge (reuse H-50’s) gains “view: hot-reuse \| recomputed” badge; or justified N/A. | L-16:L107-L108 |
| L16-E015 | E/TEST | Done-gate long multi-turn crossing compaction twice: byte-identical view across non-epoch turns; turn-2 cache_read covers unchanged prefix; skeletonized span reconstructable by larger-budget recompute without SSB; raw log byte-length monotonic non-decreasing. | L-16:L114-L119 |
| L16-E016 | E/FAIL | Negative: injected log mutation rejected/flagged; fold reading wall clock fails byte-identity; prefix-byte change without journaled cause hard fails (CC-3, proven at L-17). Under DISABLE, legacy mutate runs and byte-identity recorded-not-enforced. | L-16:L119-L124 |
| L16-E017 | E/DEP | Compose 3E context_budget, H-50 context_policy, H-49 session_dag+CAS, K-12 check_lossless_compaction re-pointed at VIEW, I-11 belief_state as projection VARIANT, E-25 as strategy — do NOT re-author. | L-16:L9-L12, L84-L97 |
| L16-E018 | E/EVIDENCE | CWD prior art (FoldSession.prepare, foldFreeze.ts, rollingFold.ts) measured 89.6/93.2% cache-read — EXTERNAL design rationale; ELAI done-gate is local byte-identity + cache-read (ID-5). | L-16:L15-L16, L128-L137 |
| L16-E019 | E/INT | Wire conversation.rs to hold log and call fold per turn; re-point live recover_stubs call-site (live_cli.rs). | L-16:L102-L104 |
| L16-E020 | E/DEP | L-17/L-18/L-19 parameterize this projection (freeze, losslessness/time-travel, page-in ladder). | L-16:L8-L8, L35-L37 |
| L16-I001 | I/STATE | I-11 BeliefState projection variant is the cross-tier handoff path when budget is the typed BeliefState shape rather than full window. | L-16:L94-L95 |
| L16-A001 | A/EVENT | Whether ContextProjected is added vs pure ContextReduction reuse is edit-time after registry re-grep. | L-16:L104-L107 |
| L16-A002 | A/SURFACE | Gauge badge vs N/A is edit-time. | L-16:L107-L108 |

Category coverage: context-core capability ✓; gauge badge surface ✓; kill-switch ✓; CC-1..4 rules ✓; projection strategies ✓; recovery recompute ✓; determinism/HD-2 ✓; tests ✓; rollback flag ✓.

### L-17 — freeze-epoch controller (prefix invariance)

Source: `docs/thefinalglance/L-plan/L-17_freeze_epoch_controller_prefix_invariance.md` (49/49 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L17-E001 | E/STATUS | L-17 is BUILD, Track L-T3, Wave W2, depends L-16; binds CC-2/CC-3; concrete prefix-invariance + cache-hit hard guard. | L-17:L1-L11 |
| L17-E002 | E/DATA | FreezeState holds: frozen view; char/token count; raw-log length at compute time; boundary fingerprint (role + length + fast non-crypto hash of boundary message); reuse/epoch counters. | L-17:L15-L17 |
| L17-E003 | E/CFG | Hash family: CWD uses FNV-1a; ELAI reuses xxhash/FxHash prefix hash from 3E (one hash family). | L-17:L16-L17 |
| L17-E004 | E/CAP | Pure `evaluate_freeze(state, log, budget, policy) -> Reuse | Recompute(cause)`. | L-17:L17-L19 |
| L17-E005 | E/STATE | Enumerated causes (CC-3): FirstCall; ColdGapTtl (idle > policy TTL); TailOverflow (raw tail since boundary exceeds cap); BoundaryMismatch (boundary hash diverged); HistoryRewound (log shrank — corruption tripwire under CC-1); PolicyChanged (BackendMode or budget strategy); PressureCeiling (input tokens hit ceiling → hand to L-16 BeliefState projection variant). | L-17:L19-L23 |
| L17-E006 | E/FLOW | On Reuse: return frozen view prefix + raw tail appended verbatim (prefix bytes unchanged → cache stays hot). | L-17:L23-L24 |
| L17-E007 | E/FLOW | Append-tail-without-recompute: when only tail grew, re-fold ONLY the tail and extend frozen prefix; never recompute existing prefix. | L-17:L24-L25 |
| L17-E008 | E/OBS | Every Recompute journals its cause (CC-3). | L-17:L25-L26 |
| L17-E009 | E/DEP | IS the concrete prefix-invariance guard I-10 specifies; cache-hit hard guard = realized cache_read covered frozen prefix on Reuse, or miss explained by journaled cause. | L-17:L30-L32 |
| L17-E010 | E/CFG | H-50 supplies TTL and late-vs-early compaction (cloud long TTL; local short / no freeze where no provider cache). | L-17:L32-L33 |
| L17-E011 | E/INT | Implement FreezeState + evaluate_freeze in context_projection.rs alongside fold; consume cache_economics.rs. | L-17:L37-L38 |
| L17-E012 | E/EVENT | prefer-reuse I-10 CacheEconomicsEvent (TtlPlanned + CacheMiss/PrefixRecomputed variant carrying enumerated cause); net 0..+1 at edit. | L-17:L38-L39 |
| L17-E013 | E/SURFACE | Context gauge shows last epoch cause; or justified N/A. | L-17:L39-L40 |
| L17-E014 | E/TEST | Two-turn cloud run no boundary change → Reuse + prefix bytes unchanged + turn-2 cache_read covers frozen prefix. TTL cross → Recompute(ColdGapTtl) journaled. | L-17:L44-L46 |
| L17-E015 | E/FAIL | Deliberately injected prefix-byte change with NO recompute cause = hard fail (CC-3). | L-17:L46-L47 |
| L17-E016 | E/FAIL | Negative: tail-only growth does NOT move prefix (unchanged prefix hash); cache miss never reported as hit (I-10 hard guard). | L-17:L47-L49 |
| L17-E017 | E/DEP | evaluate_freeze is one of fold’s inputs (L-16). Do NOT re-author I-10/H-50/L-16. | L-17:L28-L33 |
| L17-I001 | I/STATE | HistoryRewound is a tripwire for corruption rather than a normal control path under CC-1 immutability. | L-17:L20-L21 |
| L17-A001 | A/CFG | Exact TTL values, tail cap sizes, and token ceiling thresholds come from H-50 policy and are not numeric here. | L-17:L19-L23, L32-L33 |
| L17-A002 | A/EVENT | Exact CacheEconomicsEvent kind naming and net +0/+1 decided after SCHEMA re-grep. | L-17:L38-L39 |

Category coverage: freeze state machine ✓; gauge surface ✓; seven causes ✓; reuse/recompute flow ✓; I-10 composition ✓; hard-fail ✓; tests ✓.

### L-20 — commit-mined eval task supply + competitor-arm paired harness

Source: `docs/thefinalglance/L-plan/L-20_commit_mined_eval_supply_competitor_arm.md` (57/57 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| L20-E001 | E/STATUS | L-20 is PLANNED L-T4 WAVE 1, EXTENSION(J-56/E-54)+BUILD-instrument, effort M-L, grounding STRONG (codebuff source-read 2026-07-03). Measurement machinery, not agent machinery. | L-20:L1-L5 |
| L20-E002 | E/SURFACE | LEG A CLI: `elai eval mine-commits` — walk git history, deterministic pre-filter, optional advisory LLM HARD screen (ranks only), emit J-56 corpus rows. | L-20:L12-L12, L40-L40 |
| L20-E003 | E/LIMIT | Deterministic filters OUT: merge/revert/deps/lockfile/docs-only/formatting-only/mass-rename; floor+ceiling on files-changed/insertions; failure_taxonomy.py categories for typing. | L-20:L12-L12 |
| L20-E004 | E/DATA | Row shape: `{task_prompt from commit message + touched-surface summary, base_sha (parent tree), reference_diff (blake3-hashed), category, provenance}`. Flow through `elai eval ingest` (J-56) + E-54 13-gram filter (public only). | L-20:L12-L12 |
| L20-E005 | E/SEC | Private-repo rows contamination-resistant by construction; public rows E-54 filtered; asymmetry journaled per row. | L-20:L12-L12 |
| L20-E006 | E/CFG | Advisory LLM screen is ADVISORY tier (TD-3): ranks candidates, never fabricates rows; admissibility = deterministic filter verdict. | L-20:L12-L12 |
| L20-E007 | E/SURFACE | LEG B CLI: `elai bench competitor` with CompetitorArm{backend} over shipped bridges: claude_code / codex_cli / opencode / gemini_cli via cli_common::spawn_isolated_with_event (cli_common.rs:330), Tier-C. NO second spawner. | L-20:L14-L14, L40-L40 |
| L20-E008 | E/FLOW | Each task in disposable workspace (J-80 pool when lands; temp worktree meanwhile); ELAI arm + N competitor arms on SAME row set; identical envelope (prompt, workspace, wall-clock cap, journaled cost per E-49). | L-20:L14-L14 |
| L20-E009 | E/TEST | Deterministic scoring is the GATE: build green; scoped tests green where row carries them; reference-diff file-set overlap structural DKG/AST-level advisory-weighted; completion-vs-abstention. LLM judge ADVISORY only, never load-bearing (TD-3). | L-20:L14-L14 |
| L20-E010 | E/DET | Statistics per I-38: ≥5 seeds, paired, pass^k floor. Output: internal paired delta under reproducible envelope (GD-2 where arm permits); competitor arms honestly non-seed-reproducible journaled as such. NEVER leaderboard-rank (GD-3). Public numbers gate on KD-1. | L-20:L14-L14 |
| L20-E011 | E/CFG | Competitor arms run vendor’s own permissive flag INSIDE disposable workspace only; ELAI arm always runs normal gates (ELAI-with-gates vs competitor-without is honest product framing, stated in envelope). | L-20:L14-L14 |
| L20-E012 | E/CAP | LEG C: observe-only bench lessons rows from failed arm-tasks `{what_went_wrong, what_should_have_been_done}` from trace + deterministic score deltas; consumers J-10/I-12/J-96. | L-20:L16-L16 |
| L20-E013 | E/SEC | H-31 firewall: lessons NEVER injected into auditor context; K-19 SF-1 binds promotion (execution-grounded outcomes only). | L-20:L16-L16 |
| L20-E014 | E/LIMIT | EXCLUDED: 0-10 judge as gate (S10); set_messages history mutation (S11); agent store (S12); handleSteps user code agents (S13); ads/streaks (S15); repo-claimed 61%-vs-53% number (S18 design-rationale only, never cited). | L-20:L18-L18 |
| L20-E015 | E/CAP | produces: benchmarks/commit_miner + `elai eval mine-commits`; benchmarks/competitor_arm_runner.rs; bench-lessons emitter. | L-20:L31-L34 |
| L20-E016 | E/LIMIT | Stub bans: no second CLI spawner; no LLM verdict on row admissibility or arm scoring; no leaderboard-rank string; no silent skip of unavailable vendor CLI (named FAIL, R1); no `.unwrap`; blake3 only. | L-20:L41-L42 |
| L20-E017 | E/OBS | Trigger: `elai eval mine-commits --repo <path> --max 20 && elai bench competitor --arms elai,claude_code --rows <subset> --seeds 5`. | L-20:L45-L46 |
| L20-E018 | E/EVENT | EvalCorpusEvent Kind=RowIngested with base_sha + reference_diff blake3; SuiteRun (or +Kind CompetitorArmRun) per arm with cost + deterministic score; prefer-reuse net 0..+1. | L-20:L46-L47 |
| L20-E019 | E/FAIL | Negative: merge/docs-only filtered OUT (row never minted); absent vendor CLI fails arm with named reason never silent skip; leaderboard-rank string in report = contract test FAILS. | L-20:L47-L48 |
| L20-E020 | E/SURFACE | frontend_pair: paired-arm tile on bench/audit surface (decide at edit) or justified N/A while E-49 dashboard bench panel PARTIAL. | L-20:L28-L28, L43-L43 |
| L20-E021 | E/DEP | PLAN-asserted: cli_common.rs:330; claude_code.rs:68/90 stream-json; J-56 PLANNED; E-54 DARK; G-6 BENCH-PENDING; I-38 PLANNED; I-27; E-49 PARTIAL; J-80 PLANNED; failure_taxonomy WIRED. Executor re-greps W-7. | L-20:L20-L24 |
| L20-E022 | E/DEP | COMPOSES-WITH: J-56, E-54, G-6/I-38/I-27/K-15/J-107 runner family (REUSE), agent-CLI bridges B-37, E-49, J-80, J-10/I-12/J-96, K-19 SF-1 + H-31, M-5/K-1 KD-1. | L-20:L54-L55 |
| L20-E023 | E/EVIDENCE | codebuff Apache-2.0 shapes ported (pick-commits.ts, runners/claude.ts, judge.ts, lessons-extractor.ts, setup-test-repo.ts); no code copied. 61%/53% tier-2 nowhere in ELAI output. | L-20:L50-L52 |
| L20-E024 | E/INT | Integration: competitor_arm_runner.rs + commit_miner mirror K-15/J-107 over I-27 shape; args_bench.rs arm; mine-commits beside J-56 verbs. | L-20:L27-L28 |
| L20-E025 | E/CONC | Arms sequential-or-capped, never racing (batch-racing IC-1 untouched). | L-20:L51-L51 |
| L20-I001 | I/CFG | ELAI-gates-vs-competitor-permissive envelope is the product comparison frame, not a claim that competitors are weaker by construction. | L-20:L14-L14 |
| L20-A001 | A/SURFACE | Paired-arm tile vs N/A decided at edit (W-4). | L-20:L28-L28 |
| L20-A002 | A/DEP | Live anchor line numbers are PLAN-asserted and must be re-grepped (W-7). | L-20:L20-L20, L5-L5 |

Category coverage: mine-commits + competitor CLI surfaces ✓; optional bench tile ✓; three legs ✓; scoring/discipline ✓; firewall ✓; exclusions ✓; tests/negatives ✓; W-7 re-grep ✓.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | L-3, L-4, L-5 all feed L-2 PredictorSignal under HG-2 cost gate: L-3 cheap first; L-5 most expensive last on high cheap+P1+P3. | L3-E005, L3-E021; L5-E008, L5-E028 |
| IX-002 | E | L-3 Causal ClaimKind is deferred to L-5; L-5 enforces HG-1 causal-only-on-interventional for the whole CGG. | L3-E004, L3-E020; L5-E003, L5-E028 |
| IX-003 | E | L-5 reuses F-4/C-28/C-29 outcomes; L-15 S3 Behavior/Causal remedy re-invokes L-5 do-test as execution receipt. | L5-E002, L5-E006; L15-E004, L15-E007 |
| IX-004 | E | L-15 R2 reorders WAVE 2: L-3+L-5 before L-4; L-3/L-5 remain parallel-safe and still feed L-2. | L15-E014; L3-E021; L5-E009 |
| IX-005 | E | L-15 R1 hardens L-2 Low/Allow to require evidence receipt (P1 or P-cf) on receipt-available claims. | L15-E013; L3-E002; L5-E003 |
| IX-006 | E | L-15 R3 constrains L-6 retrieval; L-15 S3 P1 residual path is that deterministic retrieval. | L15-E004, L15-E015 |
| IX-007 | E | L-10 measures L-1..L-7; calibrates L-2/L-4/L-7 thresholds; L-7 default-ON waits on L-10 positive delta + bounded over-block. | L10-E007, L10-E008, L10-E011 |
| IX-008 | E | L-15 R4 adds L-10 `--repair` third arm (H_emit at fixed coverage); L-15 production trigger is that arm. | L15-E016, L15-E011; L10-E002 |
| IX-009 | E | L-10 label provenance for causal/behavior may use L-5 do-test as oracle. | L10-I001; L5-E019 |
| IX-010 | E | L-5 deterministic fixture coverage is assigned to the L-10 fixture tier; L-3 WEB/CosmosCheck likewise. | L5-E027; L3-E020; L10-E005 |
| IX-011 | E | L-11 and L-13 share the rules-engine 3F SHADOW→ACTIVE promotion machine (fpr/tpr ≥3 runs). | L11-E010; L13-E004, L13-E020 |
| IX-012 | E | L-11 OQ-1 auditor-owned frozen verdict parallels L-13 R8 immune-tier and L-15 external-receipt gate (no self-grade). | L11-E003; L13-E007; L15-E007 |
| IX-013 | E | L-11 `--quality` composes I-38 QDELTA; L-10/L-20 also compose I-38 paired-seed floor — shared statistical family, no runner forks. | L11-E011; L10-E008, L10-E012; L20-E010, L20-E022 |
| IX-014 | E | L-16 is foundation for L-17/L-18/L-19; L-17 evaluate_freeze is a fold input and implements CC-3 causes. | L16-E001, L16-E004, L16-E020; L17-E001, L17-E004, L17-E017 |
| IX-015 | E | L-16 kill-switch ELAI_CONTEXT_PROJECTION_DISABLE (default-ON path) is distinct from L-15 ELAI_CGG_REPAIR_ENABLE (default-OFF). | L16-E012; L15-E010 |
| IX-016 | E | L-17 PressureCeiling hands off to L-16 BeliefState projection variant (I-11). | L17-E005; L16-I001, L16-E017 |
| IX-017 | E | L-5 intervention isolation prefers I-2 COW / Tier-C (open seam); not implemented inside L-5. | L5-E026, L5-A001 |
| IX-018 | E | L-20 lessons consumers include I-12 curator; H-31/K-19 firewall binds auditor context. | L20-E012, L20-E013 |
| IX-019 | E | L-20 and L-10 both ban leaderboard-rank claims (GD-3) and require ELAI-measured envelopes (ID-5/KD-1). | L20-E010, L20-E016; L10-E006, L10-E017 |
| IX-020 | E | L-3 residual max on H-53 out-of-set citation and L-13 immune-tier rejection both hard-fail unsupported authority paths before downstream fusion/enforcement. | L3-E013; L13-E007 |
| IX-021 | I | L-15 S1 “which predictor fired” diagnosis couples L-3/L-4/L-5 signals into a single remedy branch table. | L15-E003, L15-E004; L3-E002; L5-E004 |
| IX-022 | E | Prefer-reuse SSB discipline is common: L-3 net 0 grounding rows; L-5 prefer F-4/mutation; L-10 SAFEDELTA family; L-11 mini_critic/maintainability; L-13 rule-fired; L-15 L-6/L-7 rows; L-16 ContextReduction; L-17 CacheEconomicsEvent; L-20 EvalCorpusEvent. | L3-E012; L5-E016; L10-E014; L11-E016; L13-E013; L15-E008; L16-E013; L17-E012; L20-E018 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity/config | L-3 zero-applicable-methods neutral residual handling undecided. | L3-A001 |
| AC-002 | ambiguity/security | L-5 isolation seam (I-2 COW vs Tier-C) deferred to authoring. | L5-A001 |
| AC-003 | ambiguity/config | L-5 high cheap+P1+P3 thresholds and J-84 budget numbers unspecified. | L5-A002 |
| AC-004 | ambiguity/event | L-5 CounterfactualProbed vs pure prefer-reuse is edit-time. | L5-A003 |
| AC-005 | ambiguity/data | L-10 per-ClaimKind gold-label provenance undecided. | L10-A001 |
| AC-006 | ambiguity/config | L-10 over-block acceptance bounds for L-7 promotion not numeric. | L10-A002 |
| AC-007 | ambiguity/config | L-11 K_* threshold seeds and max_distinct_concerns defaults undecided. | L11-A001, L11-I001 |
| AC-008 | ambiguity/surface | L-11 quality tile and L-10/L-20 bench tiles are W-4 edit choices. | L11-A002; L10-A003; L20-A001 |
| AC-009 | ambiguity/data | L-13 exact user-row schema fields beyond predicate/scope/verdict delegated to engine. | L13-A001 |
| AC-010 | ambiguity/event | L-15 RepairLoopRun vs prefer-reuse L-6/L-7 decided at edit. | L15-A001 |
| AC-011 | ambiguity/config | L-15 J-84/J-85 numeric budgets unspecified. | L15-A002 |
| AC-012 | ambiguity/event | L-16 ContextProjected optional; L-17 CacheEconomicsEvent kind names post re-grep. | L16-A001; L17-A002 |
| AC-013 | ambiguity/config | L-17 TTL/tail-cap/ceiling values owned by H-50, not numeric here. | L17-A001 |
| AC-014 | dependency | L-20 live line anchors PLAN-asserted; W-7 re-grep mandatory before wire. | L20-A002, L20-E021 |
| AC-015 | orientation | Summary L-15 is title-only; deep atoms from the 201-line original are authoritative (not a source conflict — a summary gap closed here). | L15-E001–L15-E026 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/12-L-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### L-3 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L3-001 | **ADD** residual formula, per-ClaimKind dispatcher table (including Causal→L-5), and cost-gate “medium residual triggers L-4/L-5” rule. | 12-L:L67-L78 | L3-E003–L3-E005 |
| SD-L3-002 | **ADD** exact module path roundtrip.rs, production score_claim_risk path, stub bans (HD-2/TD-3/GD-1), and contract matrix (a–f). | 12-L:L67-L78 | L3-E008–L3-E010, L3-E014 |
| SD-L3-003 | **ADD** W-1/W-2/W-9 done-gate clauses, GD-2 recomputation, and zero-applicable-methods open question. | 12-L:L67-L78 | L3-E015, L3-E017, L3-A001 |
| SD-L3-004 | **CONFIRM** summary L-3.01–L-3.10: re-ground claims, D-10/D-11/H-53/H-32, high residual, no reimpl, feeds L-2, no dedicated UI. | 12-L:L69-L78 | L3-E002, L3-E006, L3-E011, L3-E013 |

### L-5 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L5-001 | **ADD** from **pinned 103-line image**: full planner/runner/tier-guard task split, PredictorSignal{Counterfactual,moved,delta}, OBSERVE-only default, cost-gate last position. | 12-L:L98-L113 | L5-E004–L5-E008 |
| SD-L5-002 | **ADD** CounterfactualProbed kinds OutcomeMoved/OutcomeUnmoved/NotInterventional, contract cases (a–g), I-2/Tier-C isolation open question, J-84 budget note. | 12-L:L98-L113 | L5-E016–L5-E019, L5-E026–L5-E027 |
| SD-L5-003 | **ADD** CSQ composition, RunPod-gated live-repro path, frontend N/A justification, W-1 live flip done-gate. | 12-L:L98-L113 | L5-E015, L5-E020, L5-E024, L5-E030 |
| SD-L5-004 | **CONFIRM** summary L-5.01–L-5.13: Causal/Behavior interventions, F-4 hash, C-28/C-29, HG-1, J-16/G-1, no reimplement. | 12-L:L100-L112 | L5-E002–L5-E003, L5-E009–L5-E011 |
| SD-L5-005 | **CORRECT prior deep-ledger drift:** this rewrite anchors L-5 solely to the recovered pin (103 ln / b254e847…); the live 272-line recon-fold image was NOT used. Summary table citing 103-line plan remains consistent with this pin. | 12-L:L98-L113 | header L-5 provenance; L5-E001–L5-A003 |

### L-10 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L10-001 | **ADD** over-block rate as first-class metric, calibration write-back to L-2/L-4/L-7, I-38 ≥5 paired seeds + bootstrap-CI, and data/claim_gold/ shards. | 12-L:L182-L200 | L10-E003, L10-E010–L10-E012 |
| SD-L10-002 | **ADD** exact CLI flags --scope/--seed/--json, refuse-live-number, negative gate-off-but-claim-savings arm, and NO LLM-judge rg ban. | 12-L:L182-L200 | L10-E002, L10-E005, L10-E013, L10-E017, L10-E019 |
| SD-L10-003 | **ADD** label-provenance open question (L-5 oracle for causal/behavior) and dual reduction/over-block reporting rule. | 12-L:L182-L200 | L10-E023, L10-I001, L10-A001 |
| SD-L10-004 | **CONFIRM** summary L-10.01–L-10.17: ON/OFF, rates, seed, receipts, fixture/live, ELAI-measured, collapse arm, SAFEDELTA compose, optional tile. | 12-L:L184-L200 | L10-E002–L10-E008, L10-E015 |

### L-11 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L11-001 | **ADD** exact solid_proxy CLI exits/tags, METRIC default WARN, FileSpec max_distinct_concerns observe-only, MR-1/MR-2 PreToolUse floor only, and I-38 --quality arm. | 12-L:L204-L226 | L11-E004–L11-E011 |
| SD-L11-002 | **ADD** OQ-1..OQ-5 as acceptance checkboxes, gate-manifest freeze step, thrash guard A-1 permanent-WARN path, and J-23 multilang GD-5 residue. | 12-L:L204-L226 | L11-E003, L11-E009–L11-E010, L11-E023, L11-E028 |
| SD-L11-003 | **ADD** contract + fixture matrix and W-2 lifecycle-not-enable-flag rule. | 12-L:L204-L226 | L11-E018–L11-E021 |
| SD-L11-004 | **CONFIRM** summary L-11.01–L-11.21: four seams, solid_proxy, SRP/DIP/ISP/OCP-LSP split, MiniCritic, auditor-owned, SHADOW promotion, optional tile. | 12-L:L206-L226 | L11-E002–L11-E016 |

### L-13 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L13-001 | **ADD** full field shape (predicate+scope+verdict), J-102+content-scan admission order, SubagentStop residue, J-52 fall-back, and kill-switch-on-surface W-2. | 12-L:L247-L265 | L13-E002, L13-E005–L13-E006, L13-E016, L13-E018–L13-E019 |
| SD-L13-002 | **ADD** exact CLI lifecycle verbs list/test/promote/retire, drawer form actions, immune-tier re-define reject, and exec/eval/shell ban in DETERMINISTIC_CHECK. | 12-L:L247-L265 | L13-E003, L13-E007–L13-E008, L13-E015 |
| SD-L13-003 | **CONFIRM** summary L-13.01–L-13.17: rules-as-DATA, predicate kinds, hook axes, CLI verbs, SHADOW, J-52, J-102, no executable, no immune weaken, drawer authoring. | 12-L:L249-L265 | L13-E002–L13-E012 |

### L-15 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L15-001 | **ADD entire body** — summary had title-only (L-15.01/L-15.02). Full RepairController S0–S6, remedy map, J-83/J-84/J-85 terminations, external-receipt trust, ELAI_CGG_REPAIR_ENABLE, production task_runner seam. | 12-L:L285-L288 | L15-E001–L15-E012, L15-E021–L15-E022 |
| SD-L15-002 | **ADD** binding revisions R1–R5 (L-2 receipt requirement, WAVE-2 reorder L-3/L-5 before L-4, L-6 deterministic retrieval ban, L-10 --repair H_emit@coverage, L-9 flagship). | 12-L:L285-L288 | L15-E013–L15-E019 |
| SD-L15-003 | **ADD** measured CGG-bench tables (MBPP repair dominance, MATH L5 D-13, TruthfulQA ground-repair) and receipt-class frontier A–D with GD-5 residual. | 12-L:L285-L288 | L15-E017–L15-E018, L15-E020, L15-E023, L15-E025 |
| SD-L15-004 | **CLOSE** summary ambiguity L-15.02: original plan fully inspected; deep atoms are authoritative for repair controller behavior. | 12-L:L288, L720 | L15-E001–L15-E026 |

### L-16 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L16-001 | **ADD** full CC-1..CC-4 text, strategy set (E-25/H-6/3E/H-50 BackendMode), CAS dedup, recover_stubs legacy dual-path, and CWD prior-art anchors. | 12-L:L292-L313 | L16-E002–L16-E011, L16-E017–L16-E018 |
| SD-L16-002 | **ADD** context_projection.rs integration, ContextProjected optional event, gauge hot-reuse badge, one-release mutate rollback under DISABLE. | 12-L:L292-L313 | L16-E007, L16-E012–L16-E014, L16-E019 |
| SD-L16-003 | **CONFIRM** summary L-16.01–L-16.19: immutable log, pure fold, strategy params, compact_session replace, HD-2 structural, DISABLE kill-switch, byte-identity, reject mutation. | 12-L:L294-L313 | L16-E002–L16-E016 |

### L-17 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L17-001 | **ADD** FreezeState field list (char/token counts, raw-log length, hash family xxhash/FxHash), HistoryRewound tripwire, PressureCeiling→BeliefState handoff, I-10 hard-guard reformulation. | 12-L:L315-L334 | L17-E002–L17-E005, L17-E009 |
| SD-L17-002 | **ADD** cloud-long/local-short TTL policy, CacheEconomicsEvent kinds, and negative miss-as-hit ban. | 12-L:L315-L334 | L17-E010, L17-E012, L17-E016 |
| SD-L17-003 | **CONFIRM** summary L-17.01–L-17.18: seven causes, evaluate_freeze, append-tail optimization, journal causes, hard-fail unexplained prefix change. | 12-L:L317-L334 | L17-E004–L17-E008, L17-E014–L17-E015 |

### L-20 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-L20-001 | **ADD** full three-leg structure (mine / competitor / lessons), advisory LLM rank-only, private/public contamination asymmetry, gemini_cli arm, ELAI-gates-vs-competitor-permissive envelope framing. | 12-L:L374-L402 | L20-E002–L20-E013, L20-I001 |
| SD-L20-002 | **ADD** exclusions list (judge gate, set_messages, 61/53 claim), H-31/K-19 lesson firewall, and explicit CLI trigger with --max/--arms/--seeds. | 12-L:L374-L402 | L20-E014, L20-E016–L20-E019 |
| SD-L20-003 | **CONFIRM** summary L-20.01–L-20.27: mine-commits filters, J-56 rows, CompetitorArm bridges, I-38 seeds, no leaderboard, named CLI fail, W-7 re-grep. | 12-L:L376-L402 | L20-E002–L20-E016, L20-E021–L20-E025 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **9/9 files** read completely. Pinned aggregate **981/981 lines, 101,509/101,509 bytes**. All nine SHA/line/byte checks **exact**. **L-5 read from the recovered pinned image at `_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md` (hash-verified equal to the manifest row); the live L-plan file is newer (272 ln / 7d13e7a1) and was NOT used.** Non-authorized SHA mismatches: **0**.
- Atomic source ledger: **245 unique atoms** = **216 explicit**, **009 inferred**, **020 ambiguous**, **000 conflicting/corrective**.
- Per-file atom counts: L-3 **027** (24E+1I+2A); L-5 **034** (30E+1I+3A); L-10 **027** (23E+1I+3A); L-11 **031** (28E+1I+2A); L-13 **026** (23E+1I+2A); L-15 **029** (26E+1I+2A); L-16 **023** (20E+1I+2A); L-17 **020** (17E+1I+2A); L-20 **028** (25E+1I+2A). Sum **245**.
- Relational interaction records: **22** (`IX-001`–`IX-022`). Summary-delta records: **033** (`SD-*`). Ambiguity/conflict register: **15** (`AC-001`–`AC-015`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `L3/L5/L10/L11/L13/L15/L16/L17/L20-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as distinct SURFACE atoms: `elai bench hallucination-delta`; solid_proxy_check.py CLI; quality-verdict/audit tile (optional); `elai rule add/list/test/promote/retire`; RulesDrawer/ToolsHooksDrawer authoring; L-2 risk tile consumers (L-3/L-5); context gauge hot-reuse/recompute and last-epoch-cause badges; ELAI_CONTEXT_PROJECTION_DISABLE; ELAI_CGG_REPAIR_ENABLE; `elai eval mine-commits`; `elai bench competitor`; paired-arm tile (optional).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the nine exact DEEP-L-003 manifest rows, with L-5 substituted by the recovered pinned image whose hash equals the manifest row. The only orientation artifact consulted was `12-L-SUMMARY_INDEX.discovery.md`. The live L-5 path was hash-probed only to document non-use and was not opened for extraction.
- **L-5 PINNED-IMAGE PROVENANCE (footer restatement):** recovered file `docs/thefinalglance/feature-discovery/_recovered/L-5_counterfactual_do_test_on_model_claims.PINNED-b254e847.md` = **103 lines / 11,568 bytes / sha256 `b254e847133d4d7cbb095f2f49dfa07d8cd73a8d0b3aff23238c125c595ea14f`** (exact match to DEEP-SOURCE-MANIFEST DEEP-L-003 L-5 row). Live L-plan L-5 = **272 lines / 34,153 bytes / sha256 `7d13e7a1d9182070e7bb460e29e0d5420f63a5d84fbd34c4c74f0f65f1142005`** — **NOT used**. All L-5 records and anchors use the pinned image only.
- Output metrics after finalization: `output_lines=0502`; `output_bytes=075197`.
- Canonical normalized output SHA-256: 53b6ca9165aa97e3262d8778516fe73f9ec6409788a5a7619a87867579385e20.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.
