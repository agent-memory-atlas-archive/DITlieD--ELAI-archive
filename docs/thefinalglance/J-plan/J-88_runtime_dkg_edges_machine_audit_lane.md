# J-88: the two Layer-0 enablers — runtime-observed DKG edges (E2) + the machine-derived-first audit lane (E3)

- Track J (lacuna-fill track) | lever ENABLE-THE-CONSUMERS | grounding STRONG (live DKG/B-29/G-2/auditor grep) | nature BUILD (composition + one edge-kind + one input-routing) | effort M | WAVE 1 | depends none (E2 feeds J-90's L5; E3 hardens every binary gate)
- Two cross-cutting ENABLERS grouped per reasoning §2/§10 (fewest files): both precede their consumers in the unlock-DAG (RULE 2).

## LEG A — E2 runtime-observed DKG edges (the enabler for J-90's cross-language/dynamic blast cone)
- Premise: the DKG is STATIC-PARSE only. Dynamic dispatch (trait objects, IPC-strings, event-bus, reflection) yields edges the static graph misses — ELAI's own wiring doctrine names this as the residual unsoundness that breaks blast-radius (the SCIP-has-no-call-graph-edges false-negative class).
- Build (report 9 §dynamic dispatch; Chianti/DIVER prior art, design-rationale only): feed runtime-observed call edges into the DKG as a typed `observed_calls` edge KIND (a DKG edge kind, NOT an SSB row), sourced from what ELAI ALREADY captures — the RRA action-stream (G-2), L3 entrypoint coverage, L4 traces. STATIC is the conservative backbone; runtime edges are ADDED with provenance and MAY NEVER SHRINK the static cone (report 9's load-bearing safety rule). DIVER's measured 35-50% impact-set reduction is DESIGN-RATIONALE, never an ELAI number (HD-1/ID-5).
- EXCLUDED: a runtime edge that NARROWS/removes a static edge (forbidden — widen-only); an LLM proposing edges (TD-3); a new graph engine (extend the DKG, GD-1).

## LEG B — E3 the machine-derived-first audit lane (hardens every binary gate)
- Premise: the auditor is OS-isolated but still reads EXECUTOR-AUTHORED artifacts (code, comments, naming, PR prose). A persuasive comment or a misleading fn name biases the BINARY verdict across the process boundary.
- Build (report 8 §verifier-isolation): a two-lane reviewer. The BINARY VERDICT LANE consumes ONLY machine-derived artifacts (coverage/branch %, SARIF rule-IDs + severities, syscall classes from J-87's observer, unmatched-observer counts, bundle/attestation digests). Report 8's sharpest finding: DO NOT SHOW SOURCE AT ALL in that lane — code stylometry deanonymizes authors from AST STRUCTURE, not just comments, so comment-stripping is insufficient; show NUMERICAL/STRUCTURAL summaries. The LLM-auditor's prose stays ADVISORY (TD-3, already ELAI doctrine). Make the de-biased machine view the DEFAULT input for the binary gates ELAI already runs (wiring / cant-lie F-4 / blast J-16 / mutation C-28).
- EXCLUDED: showing source in the binary lane (the whole point); the advisory LLM prose entering a deterministic verdict (TD-3); rebuilding the gates (E3 only re-routes their INPUT, GD-1 reuse of existing checker outputs).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- DKG [WIRED] tree-sitter -> SQLite symbol graph, typed edges; crates/knowledge; phase-1A. E2 adds the `observed_calls` edge kind here.
- SYMBOL-LEVEL IMPACT API B-29 [WIRED] impact_radius/call_flow/symbol_context — the cone E2 enriches (and J-90 consumes).
- REPRODUCIBLE-RUN ARTIFACT G-2 [WIRED] the action-stream that SOURCES the runtime edges; rra.rs.
- the AUDITOR (OS-isolated; reads executor-authored artifacts today) + the deterministic checker outputs (maintainability_check.py / production_caller_check.py / wiring_contract_check.py + coverage/SARIF) — E3 routes these as the binary lane's ONLY input.
- live SSB baseline ~268/577 (grep 2026-06-26).

## Integration
- E2: crates/knowledge — an `observed_calls` edge kind in the DKG schema + an ingest path from the G-2 action-stream / coverage traces (provenance-tagged, widen-only). NO new SSB (a DKG edge kind, not a journal row); a DDL epoch bump only if the edge schema is versioned.
- E3: crates/orchestrator audit path — a `machine_view` projection feeding the binary gates' input; the source-bearing view stays the LLM-auditor's ADVISORY lane. NO new SSB (input-routing).
- FEATURE FLAGS: ELAI_RUNTIME_DKG_EDGES_ENABLE + ELAI_MACHINE_AUDIT_LANE_ENABLE (kill-switches, OBSERVE-ONLY first), each a wire-dark-allowlist row with promotion plan (W-2/R9).
- frontend_pair: E2 surfaces in the existing GraphViewer (edge-kind legend); E3 is a default-input change, no new surface. W-6-justified where absent.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge DKG `observed_calls` edge-kind + ingest_observed_edges (from G-2 action-stream; widen-only, provenance-tagged)
- crates/orchestrator audit machine_view projection (machine-derived-only binary-lane input)
consumes_symbols:
- the DKG static graph (E2 adds to it, never shrinks it; not reauthored)
- G-2 RRA action-stream + L3/L4 coverage traces (the runtime-edge source)
- the existing deterministic checker outputs + coverage/SARIF (E3's binary-lane input; not reauthored)
reachable_from:
- E2: a J-90 blast-cone query that returns a runtime-only edge a static cone missed (PRODUCTION, via the auditor's J-90 run)
- E3: the AUDITING transition rendering a binary verdict off the machine_view on a real run  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies either (R7)
stub_ban:
- NO runtime edge that shrinks the static cone (widen-only; the static graph is the conservative backbone)
- NO source shown in the binary verdict lane (machine-derived artifacts ONLY; AST stylometry)
- NO LLM proposing edges or rendering the binary verdict (TD-3; the advisory prose stays advisory)
- NO new graph/gate engine (extend the DKG + re-route gate input; re-implementing either = GD-1 P0)
frontend_pair: GraphViewer edge-kind legend (E2) / none for E3 (W-6-justified).
observable_effect:
- trigger: "E2: an `elai` run that calls a trait-object method the static parser cannot resolve, then a J-90 blast query over the changed symbol. E3: an audit run where a misleadingly-named fn (e.g. `safe_delete` that is unsafe) is in the changeset."
- expect_ssb: "E2 emits no SSB (DKG edge); the J-90 cone query returns the runtime `observed_calls` dependent the static cone missed. E3 emits no SSB; the binary verdict is rendered off coverage/SARIF, NOT the fn name."
- expect_artifact: "E2: the blast cone INCLUDES the dynamic dependent (the cone WIDENED, provenance=runtime); a static-only cone would have false-negatived it. NEGATIVE (R2): a runtime edge that would shrink the cone is REJECTED, not applied; an observer-fed edge with no provenance FAILS ingest. E3: the binary verdict is IDENTICAL whether the fn is named `safe_delete` or `x9`; feeding source into the binary lane FAILS the lane's input contract."

## Grounding
- E2: report 9 §cross-language/§dynamic dispatch; Chianti/DIVER prior-art NAMES (design-rationale; the 35-50% reduction is NOT an ELAI number). Done-gate = the ELAI cone WIDENS on a real dynamic dependent.
- E3: report 8 §verifier-isolation (the AST-stylometry deanonymization finding). Done-gate = verdict-invariance under a renamed fn.
- substrate: DKG + B-29 + G-2 [WIRED] + the checker stack [WIRED .claude] — feature-map 1.1/1.6/1.14.
- live SSB baseline ~268/577 (grep 2026-06-26).

## COMPOSES-WITH
DKG / B-29 (E2 extends the graph + cone), G-2 RRA (the edge source), J-90 L5 (the primary consumer of E2's runtime edges), the binary gates wiring/F-4/J-16/C-28 (E3 hardens their input), J-87 observer feed (E3's syscall-class artifacts). Restates none.

## Open questions / residue
- E2 RESIDUAL UNSOUNDNESS: runtime edges only cover paths that ACTUALLY RAN under the observed corpus; an unexercised dynamic edge stays missing. That is fine BECAUSE static is the backbone (widen-only) — but the cone must DECLARE that its runtime-edge coverage is corpus-bounded, never claim dynamic completeness.
- E3 ADVISORY TENSION: stripping source from the binary lane can hide a real bug the prose would catch — that is why the LLM-auditor ADVISORY lane (source-bearing) still runs in parallel; the binary verdict is just no longer biasable by it. Document both lanes.

## Next
E2 is consumed by J-90's L5 cross-language/dynamic cone (the user's #1); E3 hardens J-90's L4/L7 and J-87's reconcile verdict. Both are roots the rest of the program builds on.

End of J-88.
