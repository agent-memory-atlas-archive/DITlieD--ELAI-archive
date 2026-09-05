# I-24: isomorphic canary anti-gaming

- Track 15 — DETERMINISTIC GATES + ANTI-GAMING VERIFICATION | lever ANTI-GAMING | grounding STRONG | nature EXTENSION | effort S-M | WAVE 4 | binds ID-4
- Premise: a neural L2 scorer (the D-2 PPM seam, i-17 layer) can develop style bias: rating syntactically
  equivalent code differently based on surface features (variable naming conventions, loop idiom) rather than
  correctness. IS-8 RelationLibrary ships metamorphic relations; H-44 ships behavioral trap detection on the
  candidate side. i-24 closes the SCORER side: inject AST-preserving obfuscated canary pairs into the L2 stream;
  a scorer whose rating diverges across a semantically-identical pair is DETECTED as style-biased, its weight
  neutralized for the run. i-24 COMPOSES H-44's behavioral detector, never re-implements it.

## Steal/build
CANARY INJECTION: before L2 scoring a branch, with probability p (sampled uniformly, deterministic under ELAI_SEED)
inject a canary pair: the candidate plus an AST-preserving obfuscated variant (for<->while substitution, variable
renames, constant extraction — same semantics, different surface). both variants pass through L2 scoring. the score
delta between the pair is the bias signal.
AST-PRESERVING OBFUSCATION: the obfuscations are semantics-preserving by construction (they do NOT change control
flow, data flow, or type constraints). the obfuscation library is built on the IS-8 RelationLibrary (add new
AST-preserving relation types: ForWhileEquiv, RenameIdempotent, ConstantExtraction). the relation semantics are
verified by running the pair through L1 (a pair that fails L1 differently on the two variants is NOT a valid canary
— an obfuscation bug, not a scorer bug).
BIAS DETECTION: a scorer is flagged as style-biased if score(original) - score(obfuscated) exceeds a measured
threshold (the threshold is ELAI-measured on known-unbiased scorers, not a paper constant — ID-5, writes to
metrics/canary_bias_threshold.toml). a single positive detection is a WARNING; N consecutive detections within one
run NEUTRALIZE the scorer's weight (set to 0 for the run, logged with PrmCanaryEvent(ScorerNeutralized)).
H-44 COMPOSITION: H-44's TrapBehaviorDetected fires on the CANDIDATE side (detects candidates gaming the eval by
mimicking training patterns). i-24 fires on the SCORER side (detects the scorer being gamed by style). they are
complementary — run in parallel through the L2 path. i-24 MUST NOT re-implement H-44's detector (GD-1): compose
H-44's output as a signal that jointly informs the run verdict, but the trap detection logic stays in H-44.
CANARY SET FROZEN OUT-OF-BAND (R8): the canary pair set is authored once, stored as a static fixture file
(fixtures/canary_pairs.json), and is FROZEN — the running system never generates new canaries on the fly. this is
the R8 compliance: the evaluated model never authors its own canaries. the canary file is git-versioned and
gate_freeze_check.py covers it.
EXCLUDED: canaries generated dynamically by the model being scored (R8); canaries that change program semantics
(they are then not isomorphic — they test for correctness difference, not style bias); any re-implementation of
H-44's trap detection logic.

## Existing ground truth
- IS-8: metamorphic_testing.rs RelationLibrary:181 — SHIPPED; the new AST-preserving relation types register here (GD-1 compose via the existing registry)
- IS-8: MutationRunner:354 — SHIPPED; canary obfuscation uses the mutation operator vocabulary, scoped to semantics-preserving operators only
- H-22 perturbation precedents (IS-8, H-22/H-26 plans): the perturbation framework i-24 extends (PLAN-asserted; executor re-greps at EXECUTING, W-7)
- H-44 TrapBehaviorDetected — SHIPPED per H-plan Track 7; i-24 COMPOSES this (GD-1 do NOT re-implement)
- i-17 L2 path — the injection point for canaries; the canary pair is inserted here before the PPM score call
- D-2 PpmClient (the L2 scorer being tested) — SHIPPED (PLAN-asserted; executor re-greps at EXECUTING, W-7)
- IS-19 gate_freeze_check.py — the canary fixture file is added to the gate manifest (R8 compliance)
- ID-4 (evidence-ranked-search, veto-is-supreme): the ScorerNeutralized verdict is an L2 output; it cannot override an L1 HardVeto

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- AST-preserving relation types in RelationLibrary: ForWhileEquiv, RenameIdempotent, ConstantExtraction (three relations, each with a L1-verified semantic-equivalence test)
- canary injector in the i-17 L2 path (probabilistic injection at probability p, deterministic under ELAI_SEED)
- bias detection threshold (ELAI-measured from known-unbiased scorers on fixture corpus; writes metrics/canary_bias_threshold.toml — ID-5)
- ScorerNeutralized logic (N consecutive detections within one run -> weight zeroed for the run + logged)
- canary fixture file (fixtures/canary_pairs.json, frozen, added to gate-manifest.txt)
- gate_freeze_check.py coverage of the canary fixture file (R8)
- PrmCanaryEvent SSB action_type (Kind: CanaryInjected / StyleBiasDetected / ScorerNeutralized)
DEFER (Tier-2 / gated):
- automated canary generation for new programming languages (requires per-language AST library verification + new-dep approval — the seam in RelationLibrary accommodates new language adapters; the three shipped relations cover Rust)
- cross-scorer correlation analysis (if multiple L2 scorers are deployed simultaneously, detect correlated style bias across them): deferred until multiple scorers are live

## Integration
modify: crates/orchestrator/src/metamorphic_testing.rs RelationLibrary (add three new AST-preserving relation types with L1-verified semantic-equivalence tests).
new: crates/orchestrator/src/canary_injector.rs (the canary pair selector + injector + bias detector; called within the i-17 L2 path).
modify: i-17 verifier_stack.rs L2 invocation (add canary injection before the PPM score call).
modify: .claude/gate-manifest.txt (add fixtures/canary_pairs.json to the frozen gate set — R8).
SSB action_type: +PrmCanaryEvent (Kind: CanaryInjected / StyleBiasDetected / ScorerNeutralized) — baseline+1.
frontend_pair: scorer-health badge on the branch board or scoring panel (shows SCORER OK / BIAS DETECTED / NEUTRALIZED
per run, composing the existing branch board surface). cross_lang_diff PASS required (W-4).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/canary_injector.rs::CanaryInjector (select pair + inject + detect bias + neutralize)
  - new AST-preserving relation types in metamorphic_testing.rs::RelationLibrary (ForWhileEquiv, RenameIdempotent, ConstantExtraction)
  - fixtures/canary_pairs.json (frozen canary pair set — R8 artifact)
  - PrmCanaryEvent SSB action_type with CanaryInjected / StyleBiasDetected / ScorerNeutralized discriminators
  - metrics/canary_bias_threshold.toml (ID-5 baseline measurement artifact)
consumes_symbols:
  - crates/orchestrator/src/metamorphic_testing.rs::RelationLibrary (IS-8 — why: new relations register here; GD-1 do NOT fork the library)
  - H-44 TrapBehaviorDetected output (why: complementary signal to i-24's scorer-side detection; GD-1 do NOT re-implement trap detection)
  - i-17 verifier_stack.rs L2 invocation seam (why: canary pairs are injected before the PPM score call; GD-1 do NOT duplicate L2 invocation logic)
  - D-2 PpmClient (why: the scorer being tested for bias; GD-1 do NOT re-implement PPM)
  - IS-19 gate_freeze_check.py (why: canary fixture added to the gate manifest for R8 compliance; GD-1)
reachable_from:
  - crates/orchestrator/src/verifier_stack.rs::TriLayerStack::run_verdict L2 path (the canary injector fires here before every L2 scorer call on a sampled branch)  [PRODUCTION verifier stack L2]
  (contracts/ callers never satisfy R7 — production reachability is via the TriLayerStack L2 invocation; executor re-traces at EXECUTING)
stub_ban:
  - NO model-authored canaries: the canary pair set is FROZEN in fixtures/canary_pairs.json (R8); any runtime generation of canaries by the model being scored is a BLOCK
  - NO canaries that change program semantics (a canary pair that fails L1 on different variants is a malformed canary, not a valid injection)
  - NO re-implementation of H-44's TrapBehaviorDetected logic inside canary_injector.rs (compose H-44's output — GD-1)
  - NO paper-derived bias detection threshold (ID-5): threshold comes from metrics/canary_bias_threshold.toml
  - NO L1-veto override by ScorerNeutralized (a neutralized scorer affects L2 ordering only; a HardVeto from L1 is still absolute — ID-4)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2; NO fabricated bias detection results
frontend_pair:
  scorer-health badge on the branch board (composing the existing i-4 / i-17 branch board surface). badge shows
  SCORER OK (CanaryInjected only, no divergence) / BIAS DETECTED (StyleBiasDetected row, threshold exceeded) /
  NEUTRALIZED (ScorerNeutralized, weight zeroed). Tauri IPC: PrmCanaryEvent SSB rows feed the badge via the existing
  SSB subscription. cross_lang_diff PASS required (W-4). Justified — the branch board already carries per-branch
  verdict chips (i-17); this adds a scorer-health dimension.
observable_effect:
  trigger: "elai run --search --task-fixture fixtures/canary_seeded.toml --emit-ssb (fixture uses ELAI_SEED to guarantee canary injection on run 1)"
  expect_ssb: "PrmCanaryEvent(CanaryInjected) for injected pair. PrmCanaryEvent(StyleBiasDetected) for the seeded style-biased scorer fixture (score delta exceeds threshold, metrics/canary_bias_threshold.toml). PrmCanaryEvent(ScorerNeutralized) after N consecutive detections. the pair + scores are recomputable under ELAI_SEED (GD-2)."
  expect_artifact: "an unbiased scorer fixture (for which the score delta is below threshold) emits only CanaryInjected with no StyleBiasDetected (positive arm — proves the gate discriminates and does not over-fire). fixtures/canary_pairs.json is git-versioned and gate_freeze_check.py lists it in the manifest (R8 compliance). a dynamic-canary broken arm (patches canary_injector to generate canaries from the model at runtime) fails gate_freeze_check.py which detects the manifest violation — FAILS (R2/R8)."

## Grounding
- IS-8 (RelationLibrary:181): the SHIPPED substrate for the new AST-preserving relations; GD-1 composition
- H-44 TrapBehaviorDetected: SHIPPED per H-plan Track 7; i-24 is the scorer-side complement; GD-1 (never re-implement)
- ID-4 (evidence-ranked-search, veto-is-supreme): ScorerNeutralized is an L2 signal; it cannot override L1 HardVeto
- ID-5 (local-baseline-before-paper-threshold): the bias detection threshold is ELAI-measured
- R8 (gate frozen): canary fixture file is the frozen artifact; gate_freeze_check.py is the enforcer
- H-22 perturbation precedents (IS-8): the perturbation framework extended by i-24 (PLAN-asserted; re-grep at EXECUTING)
- C-29 metamorphic runner (the perturbation engine — adds AST-preserving relations per summary): PLAN-asserted; executor confirms C-29 vs IS-8 RelationLibrary relationship at EXECUTING (W-7)
- Consolidated Part IV + review 1.1 "isomorphic canary anti-gaming for the PRM scorer side"
- arXiv: structure-snowballing 2604.06066 (constrained decoding side effects on scoring quality) — WebSearch-confirm before headline citation; CRANE 2502.09061 similarly
- SSB: +PrmCanaryEvent (Kind: CanaryInjected/StyleBiasDetected/ScorerNeutralized) — baseline+1 from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9).

## COMPOSES-WITH
IS-8 RelationLibrary (EXTENDS with three new AST-preserving relation types — do NOT fork the library or duplicate
the dispatcher); H-44 TrapBehaviorDetected (scorer-side complement — do NOT re-implement the trap detection logic);
i-17 verifier_stack.rs L2 path (the injection site — do NOT duplicate the L2 invocation); D-2 PpmClient (the scorer
under test — do NOT re-implement); IS-19 gate_freeze_check.py (the R8 enforcement tool — do NOT rewrite the freeze
checker to accommodate dynamic canaries). the canary injector, bias detector, and frozen fixture set are net-new.

End of I-24.
