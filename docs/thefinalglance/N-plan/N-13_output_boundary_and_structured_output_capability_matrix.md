# N-13: OutputBoundary + structured-output capability matrix — one provider-agnostic boundary every machine-actionable model output crosses

STATUS: PLANNED (WAVE 4-adjacent; Tier-2 HIGH; nature BUILD/UNIFY — the one fragmentation the N-1..N-12 authoring missed)

Objective: declare ONE per-type OutputBoundary registry that every machine-actionable model output crosses, unifying the >=8 scattered constrained-output implementations the corpus ships (C-26 DecodingMode + per-role defaults + retries, E-35 XGrammar + INSTRUCTOR, I-34 CRANE + grammar pinning + post-parse re-validation + D-16 routing, J-53 tool-input repair, J-51 adherence floor, H-13 canvas per-mode grammars + sanitizeAction, B-24 VLM constrained JSON, L-1 typed ClaimSet, I-33 CEFL decision schemas) into one canonical object, PLUS a structured-output capability matrix (fields on the ONE catalog) + a strongest-available-enforcement resolver. This is ND-1 UNIFY-NOT-FORK on the constrained-output concern: existing mechanisms become parameterizations of the boundary, NO new validation engine is minted (GD-1). Hard boundary it must NOT cross: it does not re-implement any grammar/parser/validator — it BINDS them behind one registry + one resolver.

Estimated effort: ~1-2 sessions (~L). Tier-1 in-process (the boundary verdict is computed at emission time, no network); the capability resolver is pure DATA + a small dispatcher.
Pre-requisites (re-greped at edit, W-7): C-26 DecodingMode, E-35 XGrammar/INSTRUCTOR, I-34 CRANE + grammar-pin + post-parse re-validation, J-53 tool-input repair, J-51 adherence floor + ModelCapabilityProfile, H-13 canvas grammars, B-24 VLM JSON, L-1 ClaimSet, I-33 CEFL schemas, B-37 RuntimeCapabilities, I-51 models.toml. All PLAN-asserted from ELAI-FEATURE-MAP + the source steal file; executor re-greps each anchor per W-7.
Parallel-safe with: N-14, N-15 (disjoint files/seams).
Source: workflow/reasoning/four-source-steal-2026-07-05.md (Artifact 3 verdicts S3-1 + S3-2; the gaps-doc's P1 + P2, re-verified against the live corpus at the UNIFICATION grain). Doctrine: N-0_INDEX.md.

## Context
Why this matters: the derived gaps doc (elai-any-model-research-gaps.md) named the ONE hole its filter found that the corpus has only in fragments — there is no single named invariant "every machine-actionable model output crosses ONE provider-agnostic boundary," and no per-type registry {schema, parser, repair policy, retry budget, fail-closed action, journaled invalid-rate, backend enforcement mode}. The corpus HAS the machinery in >=8 places (above); it lacks the canonical unifier. Its best design line, kept verbatim: "Schema validation after generation catches bad structure. Constrained generation prevents bad structure." — with ELAI's addition (I-34): post-parse re-validation distrusts the constrainer itself. Mechanism: one OutputBoundary registry keyed by machine-actionable output TYPE (ToolDirective, ClaimSet, PlanSkeleton/BabyStepSpec intake, AuditFinding, MemoryCandidate, canvas ops, VLM verdicts, decision cards); each row parameterizes the existing mechanisms; a resolver picks the strongest enforcement the (type, backend) pair supports.

## Existing state (B..M coverage — PLAN-asserted, executor re-greps per W-7)
- Constrained-output producers, scattered: C-26 (DecodingMode + per-role defaults + retries), E-35 (XGrammar substrate + INSTRUCTOR), I-34 (CRANE + grammar pinning + post-parse re-validation + D-16 routing), J-53 (OpenAI-compat tool-input repair), J-51 (tool-call adherence floor + ModelCapabilityProfile), H-13 (canvas per-mode grammars + sanitizeAction), B-24 (VLM constrained JSON), L-1 (typed ClaimSet), I-33 (CEFL decision schemas). >=8 same-concern machines, no unifier — the exact N-2 duplicate-concept class.
- Capability catalog, scattered: B-37 RuntimeCapabilities, I-51 models.toml, J-51 ModelCapabilityProfile exist as the ONE catalog/profile but none enumerates structured-output FIELDS (supports_json_schema / tool_call_schema / regex_grammar / cfg_grammar / logit_bias / streaming_validation / strict_mode), so the strongest-available-enforcement choice is implicit in per-unit code, not a DATA-driven resolver.
- Security anchor (binding): I-34 grammar pinning (grammars = blake3-pinned control-plane artifacts; enum-bearing schemas from untrusted sources REJECTED), H-13 reject-never-silently-repair, J-53/I-46 repair-is-a-FROZEN-deterministic-fixup-table (never an LLM in the gate, TD-3).

## Reads (mandatory before coding)
- N-0_INDEX.md (ND-1..ND-5, the DONE definition), N-SUMMARY_INDEX.md (N-2 authoring-check + N-1 measured-invalid-rate consumers).
- workflow/reasoning/four-source-steal-2026-07-05.md Artifact 3 (S3-1/S3-2 verdicts + the ADVERSARIAL READ security rider).
- I-34 detail (grammar pinning) + H-13 (reject-never-repair) + J-53 (deterministic repair table) — the three binding riders.
- ELAI-FEATURE-MAP.md group 1.17/1.19 constrained-output + N entries (the composition edges).

## Tasks (WAVE 4-adjacent)
1. Declare the OutputBoundary registry type (~120 LOC): one row per machine-actionable output type = {schema/grammar ref (blake3-PINNED per I-34), parser fn ref, deterministic repair-policy ref (frozen table, TD-3), retry budget, fail-closed action = D-16 refusal or typed error, journaled invalid-output-rate sink = N-1 spine, enforcement_mode resolved per backend}. Existing mechanisms are referenced, never re-implemented (GD-1); a row that inlines new validation logic is an ND-1 fork.
2. Add structured-output capability FIELDS to the ONE catalog (~40 LOC DATA + accessors): supports_json_schema / tool_call_schema / regex_grammar / cfg_grammar / logit_bias / streaming_validation / strict_mode on B-37 RuntimeCapabilities / I-51 models.toml (the single catalog — do NOT mint a second).
3. Strongest-available-enforcement resolver (~50 LOC pure dispatch): provider strict schema > native tool schema > local CFG > regex > parse-validate-retry-fail-closed. The model NEVER picks the enforcement mode (TD-3); the resolver reads the capability fields only.
4. Wire the boundary at the machine-actionable emission seams (the vertical-slice consumers, per R-d): tool-directive parse, claim extraction (L-1), decision card (I-33), audit finding — each emits an OutputBoundary verdict row. Producer + in-plan consumer + smoke in the same unit.
5. Emit +output_boundary_verdict to the SSB; per-(type, backend) invalid-rate becomes an N-1 measured number (the doc's "native-schema vs external-validation delta" made an ELAI-measured fact, not folklore).
6. Decision doc: workflow/reasoning/N-13-decisions.md (what, the ND-1 unify-not-fork justification, the I-34 pinning rider, the enforcement-precedence order, unknowns).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/output_boundary.rs::OutputBoundary (the per-type registry + resolver)
  - crates/orchestrator/src/output_boundary.rs::resolve_enforcement_mode (strongest-available resolver)
  - SSB +output_boundary_verdict (Pattern-2 single-variant; Kind: SchemaEnforced / GrammarEnforced / RepairedThenValidated / RefusedFailClosed)
consumes_symbols:
  - C-26 DecodingMode / E-35 XGrammar+INSTRUCTOR / I-34 CRANE+grammar-pin+post-parse-revalidate / J-53 repair-table / J-51 adherence-floor+ModelCapabilityProfile / H-13 canvas-grammars / B-24 VLM-JSON / L-1 ClaimSet / I-33 CEFL-schemas (the >=8 unified machines — parameterized, NEVER re-implemented, GD-1/ND-1)
  - B-37 RuntimeCapabilities + I-51 models.toml (the ONE catalog the capability fields extend)
  - N-1 measurement spine (the invalid-rate sink, ND-4) + D-16 refusal (the fail-closed action)
reachable_from:
  - the machine-actionable output emission sites: tool-directive parse seam, L-1 claim-extraction seam, I-33 decision-card seam, audit-finding emit [PRODUCTION orchestrator output path]
  - a #[cfg(test)]/contracts caller NEVER satisfies this (R7)
stub_ban:
  - NO new grammar/parser/validator engine (re-implementing any unified machine = P0 ND-1)
  - NO model-settable enforcement_mode (TD-3; the resolver reads capability DATA only)
  - NO silent repair (H-13 reject-never-silently-repair; repair is the FROZEN deterministic table only)
  - NO unpinned or untrusted-sourced enum-bearing schema admitted (I-34 pinning rider)
  - NO second bespoke validation path for a type already in the registry (N-2 duplicate-concept)
  - NO .unwrap()/.expect() on prod paths; blake3 only for the schema pin
frontend_pair:
  a boundary-verdict badge on the existing claim/decision/tool drawers reading the +output_boundary_verdict row (cross_lang_diff PASS), OR N/A justified if G-5 cockpit already surfaces the invalid-rate tile.
observable_effect:
  trigger: "emit a machine-actionable output (a tool directive / a ClaimSet / a decision card) through the real orchestrator path with a backend whose capability row lacks strict schema"
  expect_ssb: "one +output_boundary_verdict row with enforcement_mode = the RESOLVED strongest-available for that (type, backend), Kind matching (e.g. RepairedThenValidated on a repair, RefusedFailClosed on an unrepairable invalid)"
  expect_artifact: "the per-(type, backend) invalid-rate lands in the N-1 spine; NEGATIVE arm — a new machine-actionable output type shipped WITHOUT a registry row fails the N-2 authoring check, and a second bespoke validation path re-appearing for a registered type fails the N-2 duplicate-concept check"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/N-plan/N-13_output_boundary_and_structured_output_capability_matrix.md --run-trigger
rg "struct OutputBoundary" crates/orchestrator/src/output_boundary.rs      # the registry exists
rg -c "fn.*validate|fn.*parse.*grammar" crates/orchestrator/src/output_boundary.rs   # MUST be 0 (no re-implemented engine)
rg "output_boundary_verdict" crates/orchestrator/src/ssb.rs                 # the SSB row is declared
test -f workflow/reasoning/N-13-decisions.md
```

## ACCEPTANCE
- [ ] ND-1: >=2 (here >=8) fragmented constrained-output impls collapsed to ONE registry; a 9th parallel machine is refused.
- [ ] ND-4: the invalid-rate routes THROUGH the N-1 spine, not a private store.
- [ ] I-34 pinning rider active: every registered schema/grammar is blake3-pinned; untrusted enum-bearing schemas REJECTED.
- [ ] TD-3: enforcement_mode is resolver-chosen from capability DATA; the model cannot set it; repair is the frozen table only.
- [ ] W-2: default ON behind ELAI_OUTPUT_BOUNDARY_LEGACY (old scattered paths survive one release, then ND-2 retires them with user confirmation).
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai_cc_orchestrator + contracts only. No new dependency (compose-not-restate; a new dep is an ND-1 red flag).

## WIRING DONE-GATE (clause by clause)
- W-1: --run-trigger PASS (produces DEF, consumes CALL/REF, reachable BFS, stub_ban clean, observable_effect RAN + emitted the verdict row). W-2: kill-switch LEGACY, default ON. W-3: reachable from the real orchestrator output path. W-4: frontend_pair badge (or N/A justified). W-5: LIVE emission trigger, not a unit test. W-6: the invalid-rate tile verified through the dashboard, or N/A justified. W-7: every unified anchor re-greped at authoring + re-validated by the executor. W-8: OutputBoundary has in-plan consumers at the four seams. W-9: +output_boundary_verdict via the action_type golden lock; SCHEMA bump named (baseline+1 from LIVE 267/576 as of 2026-07-04, re-read ssb.rs:1519 + action_type_lock.rs at edit). W-10: in-process Tier-1 boundary is the fail-closed done-gate. ND-1..ND-5: unify-not-fork (8->1), retirement map (clause 7), edge-as-data (registry rows are DATA), measured-through-spine (N-1), adapter-thinness (rows parameterize, never re-implement).

## Tests to add
- crates/contracts/tests/n13_output_boundary_contract.rs (~90 LOC): (a) a registered type resolves the strongest enforcement its backend supports; (b) a backend lacking strict schema falls to repair-then-validate then RefusedFailClosed on unrepairable input; (c) an untrusted enum-bearing schema is REJECTED (I-34); (d) enforcement_mode cannot be set from model output (TD-3 confinement); (e) a corrupt/oversized output degrades to RefusedFailClosed, never a silent pass.

## Files touched
- CREATE: crates/orchestrator/src/output_boundary.rs (~210 LOC)
- MODIFY: crates/orchestrator/src/ssb.rs (~10 LOC: +output_boundary_verdict, SCHEMA bump re-read at edit)
- MODIFY: the ONE catalog (B-37 RuntimeCapabilities / I-51 models.toml) (~40 LOC: capability fields)
- MODIFY: the four emission seams (~40 LOC total: boundary call + verdict emit)
- CREATE: crates/contracts/tests/n13_output_boundary_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/N-13-decisions.md (~40 LOC)

## Citations / Grounding
- Source: four-source-steal-2026-07-05.md Artifact 3 (S3-1 OutputBoundary, S3-2 capability matrix; the gaps-doc P1/P2 at the unification grain). The gaps-doc's paper-side claims are [EXTERNAL-UNVERIFIED]; the fragmentation finding (8 scattered machines, no unifier) = HIGH confidence from the corpus read, subject to the J-detail-body sampling caveat in the steal file header.
- ELAI ground truth: all code anchors PLAN-asserted from ELAI-FEATURE-MAP.md; executor re-greps per W-7 before coding.
- CLAUDE.md rules invoked: GD-1 compose-not-restate, ND-1 unify-not-fork, TD-3 deterministic-not-LLM-gate, I-34 grammar pinning, blake3-only.

## Open questions / risks
- The exact enum of machine-actionable output types (decide at authoring by grepping the real emission sites; start with ToolDirective/ClaimSet/DecisionCard/AuditFinding, extend as N-2 flags new ones).
- Whether B-37 RuntimeCapabilities or I-51 models.toml owns the capability fields (decide at authoring — whichever the resolver already reads; do NOT split across both).
- ND-2 retirement of the 8 scattered paths is USER-CONFIRMED (done-gate clause 7), not auto-applied.

## Next
Closes the P1/P2 gap the gaps-doc named. Pairs with N-2 (the authoring check that keeps the registry the only path) and N-1 (the invalid-rate sink). After this, "every machine-actionable model output crosses one provable boundary" is a checkable invariant, not a doctrine.

End of N-13.
