# H-20: Leakage static-analysis COMPOSITE on the taint graph (T-4) — the trading-specific killer

STATUS: PLANNED (TRACK 3 TRUST-HARDENING; WAVE 3 CLUSTER B; in-process BUILD on the shipped taint primitive; G-1 prereq flagged)

Objective: "compiles + tests pass" is STRUCTURALLY BLIND to look-ahead/leakage (design report 08) — a pipeline that
fits a scaler on full data before the train/test split, normalises across the boundary, runs full-series PCA, or
omits the signal->execution shift produces a beautiful in-sample edge that vanishes live. This is TB-2, the
trading-specific killer. H-20 builds a T1 COMPOSITE that CONSUMES ELAI's taint/provenance graph and asserts: no
fit-site reaches BOTH a train and a test sink; no normalisation/scaling statistic taints earlier-timestamp data;
no full-series PCA / feature-selection before split. Paired with a SCHEMA_REQUIRED split-boundary declaration (so
the static pass can locate the partition) and a T2 AST_PATTERN for the missing `.shift(1)` signal->execution shift.
Fail-closed with a REGISTERED suppression allowlist (never a silent skip — pandas-idiom false positives are the
real cost). The off-agent artifact is the dataflow proof (the reachability result the agent did not author).

Estimated effort: ~1 session (M-L, ~8-10 h). In-process; sound for fit/normalise/PCA classes, partial for
  target-derived features (those route to T3 advisory — TD-3/TD-5).
Pre-requisites: the taint PRIMITIVE SHIPPED but PARTIAL (provenance.rs:145, capability_gate.rs:84, capability_flow
  taint_text :534 — wire-dark/replay-only per C17); the rules-engine AST_PATTERN/COMPOSITE/SchemaRequired SHIPPED.
  HARD PREREQ FLAG (TD-5/GD-1): G-1 (live harness-wide taint propagation, taint_flow.rs) is ABSENT — H-20 needs
  either G-1 live taint OR builds a leakage-scoped dataflow pass on the DKG. State which at authoring; do not assume G-1.
Parallel-safe with: H-21 (runtime differential — the un-gameable backstop that catches what the static pass misses), H-22.
Source: design T-4 + PART 0 (TB-2) + PART 4 (R1-R5 -> T-4); _grounding/H-TRUST-GROUNDING-MAP.md TS-2 + TS-3 + NN-4 + TD-3/5. Doctrine: H-0_INDEX.md.

## Context

TB-2 LOOK-AHEAD/LEAKAGE: fit-before-split, full-series normalisation/PCA, missing signal->execution shift,
target-derived features. ELAI has the taint PRIMITIVE (Provenance/Provenanced, allow_tool_with_origin, taint_text)
but no leakage-specific predicate on it. The design's T-4 is a sound dataflow check over the taint graph for the
fit/normalise/PCA classes (Subotic & Yang ASE'22 Datalog-over-SSA hit 92.9% accuracy and found ~30% of 100k notebooks
leak; Drobnjakovic/Subotic/Urban prove absence across disjoint splits via abstract interpretation; Kapoor &
Narayanan documented the class across 294 papers). The honest scope (TD-5): target-derived-feature and sampling-bias
leakage need T3 judgment; H-20 is sound for fit/normalise/PCA only and routes the rest to a human-advisory T3, never
faking a sound verdict. The static pass is necessary-not-sufficient — the runtime differential (H-21) is the
un-gameable backstop (R5: a name appearing in text is not a dataflow edge; the runtime leg is authoritative).

## Existing state (B/C/D/E/F/G coverage)
- taint primitive (crates/safety/src/provenance.rs:145 from_web; capability_gate.rs:84 allow_tool_with_origin;
  crates/orchestrator/src/capability_flow.rs:187 check_against_commitment, :515 taint_from_verdict, :534 taint_text)
  — SHIPPED but PARTIAL/wire-dark (C17: no live prod propagation; taint_text only via `elai safety` replay; F-3p1
  CommittedToolSet leg IS cross-wired). H-20 consumes the primitive (GD-1).
- rules engine AST_PATTERN + COMPOSITE + SchemaRequired (predicate.rs:102; rule_engine.rs:98; SchemaRequired arm) —
  SHIPPED. The leakage COMPOSITE + the `.shift(1)` AST_PATTERN + the split-boundary SCHEMA_REQUIRED instantiate here.
- AstEvaluator trait (rules depends on knowledge/ for tree-sitter) — SHIPPED. The dataflow pass uses it.
- G-1 taint_flow (live harness-wide propagation) — ABSENT. The prereq H-20 must name (TD-5/GD-1).
- a leakage train/test-sink reachability predicate — ABSENT (NN-4). The pure-BUILD surface.

## Reads (mandatory before coding)
- _grounding/H-TRUST-GROUNDING-MAP.md: TS-2, TS-3 (incl C17 carry-over), NN-4, TD-3, TD-5, the H-20 symbol table.
- design T-4 + PART 0 (TB-2) + PART 4 (the R1-R5 -> T-4 mapping); H-21_runtime_lookahead_differential.md (the runtime backstop).
- F:\.ELAI_workflow\.elai_cc\crates\safety\src\provenance.rs:131-200 + capability_gate.rs:84-160 (the taint primitive)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\capability_flow.rs:180-540 (check_against_commitment / taint_text / taint_from_verdict)
- F:\.ELAI_workflow\.elai_cc\crates\rules\src\predicate.rs:100-120 (AstPattern + Composite + SchemaRequired)
- the G-1 plan (docs/thefinalglance/G-plan/G-1*.md) — to decide the live-taint dependency (compose or build a DKG dataflow pass)
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-3 (T3 advisory only), TD-5 (honest residue).

## Tasks (WAVE 3 — in-process BUILD)

1. **Resolve the G-1 dependency (decision, ~0 LOC).** Decide and DOCUMENT: (a) compose with G-1 live taint
   (taint_flow.rs) if it has shipped by Track-3 time, OR (b) build a leakage-scoped dataflow pass on the DKG +
   the taint primitive (provenance labels at the data-ingest sites). State which in H-20-decisions.md. Do NOT assume
   G-1 is live (TD-5/GD-1) — the grounding map flags it ABSENT.

2. **SCHEMA_REQUIRED split-boundary declaration (~40 LOC).** A SchemaRequired predicate forcing every backtest/
   strategy module on the execution path to DECLARE its train/test partition boundary (or decision-time) in a typed
   block, so the static pass can locate the partition. Missing declaration = BLOCK (W-1/R1 — the static pass cannot
   reason about an undeclared split, so absence is FAIL not SKIP).

3. **Leakage COMPOSITE over the taint graph (~120 LOC).** A T1 COMPOSITE asserting: (a) no fit-site (fit/fit_transform/
   train) reaches BOTH a train sink AND a test sink; (b) no normalisation/scaling statistic taints earlier-timestamp
   data; (c) no full-series PCA / feature-selection before the declared split. Sound for these classes (abstract
   interpretation across the disjoint splits). FAIL-CLOSED with a REGISTERED suppression allowlist (a flagged pandas
   idiom requires an explicit allowlist row with a reason — never a silent skip).

4. **T2 AST_PATTERN for the missing signal->execution shift (~30 LOC).** An AstPattern catching a signal used at the
   same bar it is computed (the missing `.shift(1)`): a signal column consumed by the execution/position function
   without a lag. WARN-then-route-to-T3 for ambiguous cases (TD-3); BLOCK for the unambiguous same-bar use.

5. **Off-agent artifact + suppression registry (~30 LOC).** Emit `leakage_blocked` (Pattern-2 with Kind: FitReachesBothSinks
   / NormalizationAcrossBoundary / FullSeriesPCA / MissingShift; baseline+1 from LIVE 207/484, re-read + bump e45:294)
   carrying the tainting fit-site (file:line:symbol). The dataflow reachability result is the off-agent artifact
   (the agent did not author it). The suppression allowlist is a frozen registry (gate-manifest), not agent-editable.

6. **Decision doc workflow/reasoning/H-20-decisions.md (~100 LOC).** Records: (a) the G-1 dependency resolution;
   (b) sound classes (fit/normalise/PCA) vs T3-advisory classes (target-derived, sampling-bias) — TD-3/TD-5;
   (c) the suppression-allowlist policy (registered, never silent); (d) the runtime differential (H-21) is the
   authoritative backstop the static pass defers to (R5).

## WIRING_CONTRACT
produces_symbols:
  - crates/rules/src/leakage_composite.rs::evaluate_leakage (the T1 COMPOSITE over the taint graph)
  - crates/rules/seed/finance/leakage.toml (the COMPOSITE + the .shift AST_PATTERN + the split SCHEMA_REQUIRED, mutable=0)
  - crates/orchestrator/src/ssb.rs::SsbActionType::LeakageBlocked (Pattern-2 4-Kind; baseline+1)
  - the registered suppression allowlist (frozen registry file)
consumes_symbols:
  - crates/safety/src/provenance.rs + crates/orchestrator/src/capability_flow.rs::taint_text (the taint primitive — REUSE, GD-1)
  - crates/rules/src/predicate.rs::{AstPattern, Composite, SchemaRequired} (REUSE)
  - the DKG / AstEvaluator (the dataflow pass) ; (conditional) G-1 taint_flow if live
reachable_from:
  - the PreToolUse / pre-promotion path on any backtest/strategy module [PRODUCTION gate]
  - evaluate_leakage -> the taint graph reachability -> LeakageBlocked SSB row [off-agent artifact]
stub_ban:
  - NO silent skip of a flagged idiom (a suppression requires a REGISTERED allowlist row with a reason — W-1/R1)
  - NO BLOCK without the dataflow proof (the reachability result is the load-bearing artifact, not a name-grep — R5)
  - NO sound-verdict claim for target-derived/sampling-bias classes (those route to T3 advisory — TD-3/TD-5)
  - NO re-implementation of the taint primitive or the rules engine (GD-1 P0); blake3 / no .unwrap() in prod
frontend_pair:
  the tainting fit-site + reachability path in a leakage drawer. cross_lang_diff PASS. W-6: a blocked leak shows the
  fit-site in the dashboard.
observable_effect:
  trigger: "cd .elai_cc && cargo run -p elai-cli -- <feed a fixture pipeline that fits a StandardScaler on full data before split, NO env override>"
  expect_ssb: "a LeakageBlocked(NormalizationAcrossBoundary) row lands naming the tainting fit-site (file:line), recomputable from the journal (GD-2)"
  expect_artifact: "the fit-before-split pipeline is BLOCKED with the fit-site reported; a clean split-then-fit pipeline PASSES; a flagged
    pandas idiom requires a REGISTERED suppression row to pass (a silent skip is impossible)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/H-plan/H-20_leakage_taint_composite.md --run-trigger
rg "evaluate_leakage|taint_text|from_web" .elai_cc/crates --glob '!*test*'              # the COMPOSITE consumes the taint primitive
rg -n "leakage|SchemaRequired" .elai_cc/crates/rules/seed/finance/leakage.toml          # the LOCKED seed present
rg -c "blake2|sha2" .elai_cc/crates/rules/src/leakage_composite.rs                       # MUST be 0
test -f workflow/reasoning/H-20-decisions.md
```

## ACCEPTANCE
- [ ] W-1 done-gate: --run-trigger PASS (a fit-before-split fixture BLOCKED with the fit-site reported; a registered
  suppression required to pass a flagged idiom). cargo-green != DONE.
- [ ] W-2 default-ON: the COMPOSITE runs with no env override; kill-switch only.
- [ ] W-3 production entrypoint: evaluate_leakage fires from PreToolUse/pre-promotion, not a #[cfg(test)] caller.
- [ ] TD-3: target-derived/sampling-bias classes route to T3 advisory, never a faked sound verdict.
- [ ] TD-5: the G-1 dependency is resolved + documented; honest residue (sound classes only) recorded.
- [ ] GD-1: taint primitive + rules engine REUSED. GD-2: the reachability result re-computes under ELAI_SEED.
- [ ] cargo check --workspace GREEN; auditor PASS. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger is ACCEPTANCE item 1; evaluate_leakage (produced) consumed on the live gate path (W-8) reached by PreToolUse/promotion (W-3).
- W-2: no override; kill-switch only. W-3: live gate path; not test-only.
- W-4: frontend_pair = leakage drawer; cross_lang_diff PASS. W-5: runnable fixture -> real row; Tier-1 in-process.
- W-6: blocked fit-site visible in dashboard. W-7: provenance.rs:145 / capability_flow taint_text:534 / predicate.rs:102 grep-confirmed (TS-2/TS-3); leakage predicate absent (NN-4).
- W-8: evaluate_leakage (produced) consumed on the gate path; taint_text consumed by the COMPOSITE.
- W-9: +LeakageBlocked Pattern-2 baseline+1; bump e45:294.
- W-10: Tier-1 in-process (static dataflow); the runtime backstop is H-21 (a SEPARATE gate). TD-3/5: advisory + honest residue.

## Tests to add
- crates/contracts/tests/h20_leakage_contract.rs (~160 LOC): (a) fit-before-split -> Block(NormalizationAcrossBoundary)
  with fit-site; (b) full-series PCA before split -> Block(FullSeriesPCA); (c) missing .shift(1) same-bar use -> Block(MissingShift);
  (d) clean split-then-fit -> Allow; (e) a flagged pandas idiom requires a registered suppression to pass (no silent skip);
  (f) target-derived feature -> T3 advisory (WARN + route), not a sound Block; (g) ELAI_SEED twice -> identical reachability result.
- crates/contracts/tests/fixtures/leakage/ (fit-before-split, full-series-PCA, missing-shift, clean, target-derived).

## Files touched
- CREATE: crates/rules/src/leakage_composite.rs (~120 LOC: the dataflow COMPOSITE over the taint graph)
- CREATE: crates/rules/seed/finance/leakage.toml (COMPOSITE + .shift AST_PATTERN + split SCHEMA_REQUIRED, mutable=0)
- MODIFY: crates/orchestrator/src/ssb.rs (+LeakageBlocked Pattern-2) + action_type_lock.rs + e45 :294
- MODIFY: the PreToolUse / pre-promotion path (~20 LOC: route backtest modules through evaluate_leakage)
- CREATE: the suppression allowlist registry + add to .claude/gate-manifest.txt
- CREATE: crates/contracts/tests/h20_leakage_contract.rs (~160 LOC)
- CREATE: workflow/reasoning/H-20-decisions.md (~100 LOC)

## Citations / Grounding
- design T-4 + PART 4; Subotic & Yang ASE'22 (Datalog-over-SSA, 92.9% acc, ~30% of 100k notebooks leak);
  Drobnjakovic/Subotic/Urban (arXiv:2211.16073, abstract interpretation proves absence across disjoint splits);
  Kapoor & Narayanan "Leakage and the Reproducibility Crisis" (294 papers). Paper-ingest before coding the dataflow.
- ELAI ground truth (TS-2/TS-3/NN-4, grep-confirmed 2026-06-04): provenance.rs:145, capability_flow taint_text:534,
  predicate.rs:102; G-1 taint_flow ABSENT; leakage predicate ABSENT.
- CLAUDE.md HARD RULES: blake3, no .unwrap()/.expect(); TD-3/5; C17 (E-37 spine wire-dark — carry the honest caveat).

## Open questions / risks
- The G-1 dependency is the load-bearing risk: if G-1 live taint has NOT shipped, H-20 must build a leakage-scoped
  dataflow pass on the DKG — that is L-effort itself (the abstract-interpretation pass). Scope this decision in Task 1
  BEFORE committing to the COMPOSITE; the runtime differential (H-21) is the un-gameable fallback if static taint is incomplete.
- Pandas-idiom false positives are the real cost (the design names them explicitly) — bound the FP rate on >=3 real
  backtest modules before default-ON (GD-4); the registered suppression allowlist is the escape valve, never a silent skip.
- Static dataflow over Python with dynamic dispatch is unsound in general (R5) — H-20 is necessary-not-sufficient;
  H-21 (runtime differential) is the authoritative backstop and MUST ship for the leakage class to be trustworthy.

## Next
H-21 (runtime look-ahead differential) is the un-gameable backstop: run the pipeline on full vs future-truncated
data and assert the feature columns are identical — catching the leakage class static analysis is structurally blind
to. H-22 (trading nulls) catches the edge that survives a shuffled future. Together they close TB-2.

End of H-20.
