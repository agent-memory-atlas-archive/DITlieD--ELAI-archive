# L-1: Typed claim extraction substrate — turn any model output into a typed, off-agent ClaimSet

STATUS: PLANNED (WAVE 1; M; BUILD — foundation; the intake every CGG predictor reads)

Objective: the CGG cannot score what it cannot name. Today claim "extraction" is implicit inside the D-10 grounding-sweep, coupled to the agent's own output and not a reusable object. L-1 lifts it OUT into a first-class, deterministic, OFF-AGENT extractor: any model output (answer / plan / patch / bug-explanation / dependency recommendation) parses into a ClaimSet = Vec<Claim{kind: ClaimKind, span, refs}>, ClaimKind in {File, Symbol, Dependency, Api, Behavior, Causal, Reachability} (the user-notes' claim table, LS-2). Parsed STRUCTURALLY from cited paths / symbols / package-names / signatures, NEVER by asking a model "what did you claim" (HG-3 black-box, TD-3 the model never types its own claims). The model that emitted the output never types its own claims (wiring R8, off-agent). HARD BOUNDARY: L-1 only TYPES and resolves refs; it does NOT score risk (that is L-2) and does NOT verify (L-3/L-4/L-5). GD-1: do NOT re-implement the D-10 grounding methods — extract the claim object the sweep already builds implicitly.

Estimated effort: ~1 session (~M). In-process at the report/claim-emission seam; no network.
Pre-requisites (re-grep live per W-7): the report/claim seam (report_fns role-classification); D-10 select_grounding_mode (grounding_mode.rs:157, deterministic); G-1 taint Provenance (the 4 ingest sites — a claim sourced from untrusted input is tagged at extraction); LIVE SCHEMA/TV (ssb.rs / action_type_lock.rs).
Parallel-safe with: none in WAVE 1 (L-2 depends on L-1); disjoint from L-3..L-10 files.
Source: cgg-hallucination-causal-system-2026-06-27.md S8 + L-CGG-GROUNDING-MAP.md NN-1/TS-1/TS-13. Doctrine: L-0_INDEX.md (HG-3, TD-3).

## Context
Why this matters: the paper (arXiv:2606.27326) and the user notes agree the gate's first job is to turn fluent output into discrete CHECKABLE claims; you cannot apply the round-trip / inter-seed / do-test predictors to a paragraph, only to a typed claim. The notes' claim-type table maps each kind to its check method (File>filesystem, Symbol>AST/LSP, Dependency>lockfile/registry, Api>typecheck/source, Behavior>test/trace, Causal>counterfactual, Reachability>call-graph/trace). Typing is deterministic and structural — the LLM never decides what its claims were (the same anti-self-grading principle as RR-1/F-4). A claim whose refs cannot be resolved is NOT dropped; it is typed and flagged refs-unresolved (that flag is the cheapest medium-risk signal L-2 reads).

## Existing state (B/C/D/E/F/G coverage)
- D-10 select_grounding_mode [WIRED, grounding_mode.rs:157] — deterministic mode routing; the grounding-sweep already walks claims implicitly but does not expose a typed ClaimSet. [MAP]
- D-11 constructive-checker registry [DARK(ELAI_CHECKER_ENABLE)] — the per-kind check backends (consumed later by L-3/L-5). [MAP]
- G-1 taint propagation [WIRED] — Provenance at 4 ingest sites; a claim derived from an untrusted source is taggable at extraction time. [MAP]
- report_fns role-classification [MAP] — the seam where a role's output is assembled; the extraction hook point.

## Reads (mandatory before coding)
1. _grounding/L-CGG-GROUNDING-MAP.md (NN-1, TS-1, TS-13); cgg-hallucination-causal-system-2026-06-27.md S8 + the claim-type table in §1 of the user notes.
2. the report/claim-emission seam (report_fns); grounding_mode.rs (select_grounding_mode); the G-1 Provenance type.
3. F-0_INDEX W-1/W-7; G-0 GD-1/GD-2; CLAUDE.md TD-3 (LLM never adjudicates), no-.unwrap-on-prod, blake3-only.

## Tasks (WAVE 1 — type the claims, off-agent)
1. ClaimKind + Claim + ClaimSet types (~40 LOC): enum ClaimKind{File,Symbol,Dependency,Api,Behavior,Causal,Reachability}; Claim{kind, span: ByteSpan, refs: Vec<ClaimRef>, taint: Option<Provenance>}; ClaimSet wrapping a Vec<Claim>. ClaimRef = a resolved-or-unresolved pointer (path/symbol/pkg/signature).
2. the deterministic extractor (~90 LOC): parse a model output into a ClaimSet by structural cues (cited backtick paths, `Symbol::method` forms, package-name + version forms, signature forms, "fixes/causes/improves" behavior/causal verbs, "reaches/used/unused" reachability verbs). NO model call (TD-3). Tag taint from G-1 when the output derives from an untrusted ingest. Refs that do not resolve to a real path/symbol are kept with resolved=false (never dropped).
3. wire the extractor at the report/claim seam OFF-AGENT (~25 LOC): the orchestrator runs it on a worker's returned output; the worker never types its own claims (wiring R8). Emit the ClaimSet for L-2 to score.
4. workflow/reasoning/L-1-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/claim/mod.rs — ClaimKind / Claim / ClaimSet types
  - crates/orchestrator/src/claim/extract.rs — deterministic structural extractor (off-agent)
consumes_symbols:
  - grounding_mode.rs::select_grounding_mode (the existing claim-walk it lifts the object out of — COMPOSE, never re-implement)
  - G-1 Provenance (taint tag at extraction)
reachable_from:
  - the orchestrator's post-worker report/claim seam (report_fns) > extract_claims [PRODUCTION claim-intake]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO model call to type claims (TD-3 — structural parse only)
  - NO dropping unresolved-ref claims (flag resolved=false, never silently discard)
  - NO worker-side extraction (off-agent only, wiring R8)
  - NO .unwrap()/.expect() on prod; blake3 not sha2
frontend_pair:
  a claims tile on the trust panel (decide at edit, W-4) OR "N/A with justification — extraction feeds L-2's risk tile, no standalone surface. Justified."
observable_effect:
  trigger: "run the orchestrator on a worker output containing a file-claim + an invented-package-claim (real report seam)"
  expect_ssb: "+1 ClaimSetExtracted (Pattern-2; Kind: ClaimTyped / ClaimUntypable) OR reuse a grounding-verification row; baseline+0..1 from LIVE ~268/577 (re-grep, W-9)"
  expect_artifact: "the ClaimSet types both claims; the invented-package claim is typed Dependency with refs resolved=false (NEGATIVE: it is NOT silently dropped, NOT typed as supported)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md --run-trigger
rg "enum ClaimKind|struct ClaimSet" .elai_cc/crates/orchestrator/src/claim/mod.rs   # types present
rg -c "llm|generate|complete\(" .elai_cc/crates/orchestrator/src/claim/extract.rs    # MUST be 0 (structural parse, no model call)
test -f workflow/reasoning/L-1-decisions.md
```

## ACCEPTANCE
- [ ] HG-3: extraction uses zero model internals and zero model calls; runs on any backend's output.
- [ ] TD-3: the model never types its own claims; the extractor is deterministic + off-agent (R8).
- [ ] unresolved refs are flagged resolved=false, never dropped.
- [ ] GD-1: the D-10 grounding methods are consumed downstream, not re-implemented here.
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live report seam produces a typed ClaimSet on a real worker output (not cargo-green).
- W-2: no enable-flag (extraction is always-on intake; no behavior gate yet — that is L-7).
- W-3: lifts a real object out of the BUILT grounding-sweep, not a summary-only port.
- W-4: frontend_pair decided at edit (claims tile) or justified N/A.
- W-5/W-10: in-process; no network leg.
- W-6: dashboard surface optional (decided at edit).
- W-7: grounding_mode.rs:157 + the report seam re-grepped at edit.
- W-9: SSB prefer-reuse or +1 from the LIVE baseline.
- GD-2: the ClaimSet is recomputable from the worker output under ELAI_SEED.

## Tests to add
- crates/contracts/tests/l1_claim_extraction_contract.rs (~70 LOC): (a) a file-claim is typed File with a resolved ref when the path exists; (b) an invented-package claim is typed Dependency with resolved=false (not dropped); (c) a "this fixes X" claim is typed Behavior/Causal; (d) extraction makes zero model calls (mock backend asserts no invocation); (e) an untrusted-sourced claim carries a G-1 taint tag; (f) extraction runs off-agent (worker mock never calls it).

## Files touched
- CREATE: .elai_cc/crates/orchestrator/src/claim/mod.rs (~40 LOC)
- CREATE: .elai_cc/crates/orchestrator/src/claim/extract.rs (~90 LOC)
- MODIFY: the report/claim seam (report_fns) (~25 LOC: off-agent extraction hook)
- CREATE: crates/contracts/tests/l1_claim_extraction_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/L-1-decisions.md

## Citations / Grounding
- cgg-hallucination-causal-system-2026-06-27.md S8 + the user notes' claim-type table (LS-2).
- ELAI ground truth (L-CGG-GROUNDING-MAP TS-1/TS-13/NN-1): grounding_mode.rs:157, the report seam, G-1 Provenance (re-grepped at edit, W-7).
- CLAUDE.md: TD-3 (LLM never adjudicates), no .unwrap on prod, blake3-only.

## Open questions / risks
- The exact structural-cue set (which verbs/forms map to which ClaimKind): seed from the notes' table, OBSERVE-first to bound mis-typing before any downstream gate blocks (GD-4). Decide the cue lexicon at authoring.
- Span fidelity across streaming output: the ByteSpan must survive the streaming assembly; verify against the live report seam.

## Next
L-1 gives the CGG its unit of work. L-2 fuses the predictor scores over this ClaimSet into the ClaimRisk object; L-3/L-4/L-5 each read a ClaimSet and emit a per-claim predictor score.

End of L-1.
