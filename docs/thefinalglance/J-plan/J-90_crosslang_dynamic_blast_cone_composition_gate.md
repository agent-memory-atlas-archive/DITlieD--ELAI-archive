# J-90: the cone-composition gate trio — cross-lang+dynamic blast cone (L5, user's #1) + usage-weighted reachability (L4) + cant-lie x mutation floor (L7)

- Track J (lacuna-fill track) | lever CANT-WRECK / CANT-VACUOUSLY-PASS | grounding STRONG (live J-16/J-56/elai.map/cross_lang_diff.py/F-4/C-28/I-20 grep) | nature BUILD (cone fusion + corpus fusion + gate composition) | effort L | WAVE 1 (L5) / WAVE 2 (L4/L7 enforcement) | depends J-88 E2 (runtime edges), J-16 cone, J-56 corpus, J-34 (the ENFORCEMENT thresholds)
- Grouped per reasoning §10 ("they share the J-16/J-56 cone substrate"). L5 is the USER'S #1 priority (the ruthless-3, reasoning §9). L4/L7 ENFORCEMENT is J-34-gated; their MECHANISM ships observe-only now (RULE 1).

## LEG A — L5 cross-language + dynamic-dispatch blast cone (the user's stated #1)
- Premise: J-16's cone uses B-29's STATIC SINGLE-LANGUAGE impact API -> false-negatives on cross-language seams (Rust<->TS Tauri, Rust<->Python sidecar) AND dynamic dispatch. A change that breaks via a cross-lang or dynamic edge passes the blast gate. This is the exact "small fix breaks something else across the IPC seam" class single-language static cones are blind to.
- Build (report 9 §cross-language — SCIP/Glean + seam extractors + DIVER hybrid; design-rationale): ELAI uniquely ALREADY HAS elai.map (all-langs, SCIP-enriched) + cross_lang_diff.py (Rust<->Python, Tauri<->Svelte pairs). Feed those cross-language edges PLUS J-88/E2's runtime-observed edges into J-16's reverse-dep cone. Add SEAM EXTRACTORS for the typed boundaries (Tauri generate_handler! <-> TS invoke(), sidecar route <-> Rust client — both already golden-locked). OPAQUE SEAMS WIDEN, NEVER NARROW, the cone; the LLM never fills the gap (report 9's safety rule).
- EXCLUDED: an LLM filling an opaque-seam gap (TD-3 — widen-or-fall-back-to-broader-tests instead); a runtime edge that shrinks the cone (J-88 widen-only); a new cross-lang graph (consume elai.map + cross_lang_diff.py, GD-1).

## LEG B — L4 usage-weighted reachability (not binary)
- Premise: the wiring gate proves a symbol reachable + emits a row, but a fn wired into a reachable-but-never-taken branch passes; coverage>0 is satisfiable by a smoke driving a path NO REAL USER drives.
- Build (report 9 §representative workloads — ELAI uniquely HAS the missing input): fuse the living eval corpus (E-54/J-56) into the wiring gate. Replace the single smoke with a FROZEN WEIGHTED representative-workload corpus (prior journeys, incident repros, boundary cases); the gate metric becomes USAGE-WEIGHTED EXERCISEDNESS — for each changed cone, the total frozen weight of scenarios whose traces hit it, requiring a MINIMUM weighted score AND >=2 scenario families (report 9's exact anti-smoke-exploit rule). Pair with a SAFE off-agent regression-test selector (impacted U new U previously-failing U always-run canary; fall back to the FULL suite on unsupported changes — never probabilistic in the blocking path).
- MECHANISM/ENFORCEMENT (RULE 1): ship the weighted-exercisedness MEASURE OBSERVE-ONLY; gate the minimum-weight threshold ONLY after J-34 measurement.
- EXCLUDED: a probabilistic selector in the blocking path (deterministic + full-suite fallback); the minimum-weight threshold as a paper constant (J-34-measured; ID-5); a single-family weighted score passing (>=2 families required).

## LEG C — L7 compose cant-lie with a mutation-survival floor
- Premise: F-4's blake3 stdout-hash proves the RESULT honest; it is blind to whether the TEST is meaningful. A vacuous test (the bulk of agent-written tests lack meaningful assertions) passes TRUTHFULLY and the hash matches TRUTHFULLY — defeating cant-lie WITHOUT lying.
- Build (report 9 §test meaningfulness — the 3-layer gate, honest-pass X meaningful-test): (1) ASSERTION-VALIDITY LINT off-agent on every changed test (ban print-only / no-exception-only / self-referential oracles); (2) CHANGED-CONE MUTATION (ELAI's C-28 + I-20 budgeted scheduler, scoped to J-16/L5's diff cone): passed=true ADDITIONALLY requires the tests killed >=k injected mutants in the changed cone, no surviving critical mutants on changed lines; (3) ANTI-VACUITY HEURISTICS (deterministic).
- MECHANISM/ENFORCEMENT (RULE 1): the mutation FLOOR (k) is a measured threshold -> OBSERVE-ONLY until J-34-calibrated. Tier it: changed-LINE blocking in PRs, changed-CONE in the merge queue, FULL nightly.
- EXCLUDED: treating mutation as an ORACLE (it is a strong LOWER BOUND; equivalent-mutant problem — tier it); the floor k as a paper constant (J-34-measured); an LLM judging test meaningfulness (deterministic lint + mutation; TD-3).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- BLAST-RADIUS / ANTI-REGRESSION GATE J-16 [PLANNED, KEYSTONE] blast_radius.rs reverse-dep cone via B-29; the cone L5 fuses cross-lang + runtime edges into.
- elai.map (all-langs, SCIP-enriched) + cross_lang_diff.py (Rust<->Python, Tauri<->Svelte) [.claude harness, WIRED] — the cross-language edges + the golden-locked seams.
- LIVING EVAL CORPUS J-56 [PLANNED] + E-54/G-6 — the frozen weighted corpus L4 fuses in.
- TEST-INTEGRITY CANT-LIE GATE F-4 [WIRED] AntiFabricationGate.check() blake3 stdout_hash; MUTATION TESTING GATE C-28 [PARTIAL] + I-20 budgeted mutation scheduler [WIRED] (MutationBudgetedRun SSB).
- J-88/E2 observed_calls runtime edges (this wave) — the dynamic-dispatch input to L5's cone.
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- L5: extend crates/orchestrator/src/blast_radius.rs (J-16) with a cross-lang + runtime-edge cone fusion (consume elai.map + cross_lang_diff.py seam edges + J-88 observed_calls); reuse BlastRadiusEvent +Kind (CrossLangConeWidened / OpaqueSeamWidened). net 0 (prefer-reuse).
- L4: a usage-weighted-exercisedness scorer over the J-56 corpus + the safe regression selector, feeding the wiring gate. reuse the wiring-gate evidence path; +UsageWeightExercised Kind or prefer-reuse. OBSERVE-ONLY.
- L7: compose F-4 + C-28/I-20 scoped to the L5 diff cone (assertion-validity lint + changed-cone mutation + anti-vacuity). reuse TestIntegrityEvent / MutationBudgetedRun +Kind. OBSERVE-ONLY until J-34.
- FEATURE FLAGS: ELAI_CROSSLANG_BLAST_CONE_ENABLE (L5; observe-only first) + ELAI_USAGE_WEIGHTED_REACH_ENABLE (L4; J-34-gated enforcement) + ELAI_MUTATION_FLOOR_ENABLE (L7; J-34-gated); each a wire-dark-allowlist row with promotion plan (W-2/R9). All AUDITOR-owned, FROZEN to the executing run (R8).
- frontend_pair: the cross-lang cone + weighted-exercisedness + mutation-floor on the existing audit-timeline / blast-radius panel (J-16 precedent); cross_lang_diff if surfaced (W-4).

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/blast_radius.rs::fuse_crosslang_cone (elai.map + cross_lang_diff seams + J-88 runtime edges; widen-only)
- crates/orchestrator/src/blast_radius.rs::extract_typed_seams (Tauri generate_handler!<->invoke, sidecar route<->Rust client)
- crates/orchestrator/src/usage_weight.rs::weighted_exercisedness (J-56 corpus weight over the changed cone; >=2 families)
- crates/orchestrator/src/usage_weight.rs::select_safe_regression (impacted U new U prev-failing U canary; full-suite fallback)
- crates/orchestrator/src/test_meaningfulness.rs::{assertion_validity_lint, changed_cone_mutation_floor, anti_vacuity}
consumes_symbols:
- J-16 blast_radius cone + B-29 traversal (extended, not reauthored)
- elai.map + cross_lang_diff.py cross-language edges + J-88 observed_calls (the cross-lang/dynamic input)
- J-56 living eval corpus (the frozen weighted workloads)
- F-4 AntiFabricationGate + C-28/I-20 mutation scheduler (composed, not reauthored)
reachable_from:
- the AUDITING transition on a real `elai` fix run that edits a symbol with a cross-language or dynamic dependent  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM filling an opaque-seam gap or judging test meaningfulness (widen-or-broaden + deterministic lint/mutation; TD-3)
- NO runtime/cross-lang edge that SHRINKS the cone (widen-only; log what was widened, R-no-silent-caps)
- NO single-family weighted score passing L4 (>=2 scenario families required)
- NO minimum-weight / mutation-floor threshold as a paper constant (J-34-measured; ID-5/RULE-1)
- NO probabilistic regression selector in the blocking path (deterministic + full-suite fallback)
- NO BLOCK default until FP bounded on >=3 runs (GD-4) + a wire-dark-allowlist row (W-2/R9)
frontend_pair: cross-lang cone + weighted-exercisedness + mutation-floor on the audit/blast surface, or W-6-justified absence.
observable_effect:
- trigger: "an `elai` fix that changes a Rust fn exposed via a Tauri generate_handler! that a Svelte component invoke()s (a cross-language seam); AND a fix whose new test only print!s without asserting"
- expect_ssb: "CrossLangConeWidened listing the TS invoke() caller the single-language cone missed; OpaqueSeamWidened if a string-built route forces a widen; the L7 lint flags the print-only test"
- expect_artifact: "the cross-lang cone INCLUDES the Svelte consumer; its regression set RUNS and a pass->fail there BLOCKS the advance. The print-only test FAILS the assertion-validity lint; a surviving critical mutant on a changed line FAILS the mutation floor (when enforcement is J-34-on). NEGATIVE (R2): an unmappable cross-lang seam WIDENS to broader tests + log()s it, never silently narrows; a weighted score from ONE family does not satisfy L4; the mutation gate run with no corpus FAILS its self-test, never SKIPs."

## Grounding
- L5: report 9 §cross-language (SCIP/Glean + seam extractors + DIVER hybrid; design-rationale). Done-gate = the ELAI cone catches a real cross-language dependent a single-language cone missed. Zero published regression-selection benchmark for agent edits = open whitespace (J-16 grounding, carried).
- L4: report 9 §representative workloads (usage-weighted exercisedness + >=2-family anti-smoke rule). Done-gate = ELAI-measured weighted score on the J-56 gold corpus; threshold J-34-measured.
- L7: report 9 §test meaningfulness (3-layer assertion-lint + changed-cone mutation + anti-vacuity). Done-gate = a real vacuous test FAILS; floor k J-34-measured.
- substrate: J-16/J-56 [PLANNED] + elai.map/cross_lang_diff.py [WIRED .claude] + F-4 [WIRED] + C-28/I-20 [PARTIAL/WIRED] + J-88/E2 (this wave) — feature-map 1.1/1.6/1.14.
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
J-16 (extends its cone), J-88/E2 (consumes runtime edges), elai.map + cross_lang_diff.py (the cross-lang edges), J-56 + E-54/G-6 (the corpus), F-4 + C-28/I-20 (L7's composition), J-34 (the enforcement thresholds), J-89 L8 (plan-time prevention back-stopped by this detection). Restates none.

## Open questions / residue (reasoning §7)
- L5 OPAQUE SEAMS: ad-hoc shell-outs / string-built routes / unregistered event buses stay un-modeled -> WIDEN-or-fall-back-to-broader-tests, and log() what was widened so silent truncation cannot read as "covered".
- L4 FROZEN-CORPUS OVERFIT: a frozen corpus overfits if static too long -> a hidden holdout slice + incident-derived rotation on a SEPARATE cadence; the threshold is J-34-measured, never a paper constant.
- L7 EQUIVALENT-MUTANT: mutation is a strong LOWER BOUND, not an oracle -> tier it (changed-line in PRs, changed-cone in the merge queue, full nightly); never claim it proves test sufficiency.
- ENFORCEMENT WAITS ON J-34 (RULE 1/R4): L4's min-weight + L7's floor k stay OBSERVE-ONLY until measured — still a real improvement (the signal is visible), not yet a blocking gate.

## Next
L5 closes the user's #1 "small fix breaks something across the IPC seam" class; L4/L7 harden the cone into a meaningfulness floor once J-34 measures the knees. Together with J-89's prevention pair, "cant wreck" and "cant vacuously pass" become gates.

End of J-90.
