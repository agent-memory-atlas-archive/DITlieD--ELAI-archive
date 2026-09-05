Track A.4-finish — Decisions Document
Session: s-934436
Date: 2026-05-05
Predecessor handoff: workflow/sessions/s-155876/SESSION_HANDOFF.md
Context package: workflow/sessions/s-934436/context-package-track-A4finish.md

WHAT
Wire the real fetcher backend label ("reqwest" or "obscura") through the
fetch_and_extract -> node_loop -> emit_step -> WebResearchStepEvent ->
WebResearchStepPayload chain so that the SSB row carries the actual
fetcher backend used for each per-source row, replacing the
hardcoded String::new() at orchestrator/src/task_runner.rs:1387.

KEY DECISIONS (with the WHY)

1. Return-type extension (not field on a new struct).
   fetch_and_extract today returns
     Result<(String, SourceTier), WebResearchOrchestratorError>
   (NOT Result<FetchedDoc, FetchError> as the parent prompt assumed —
   verified by context-compiler against
   crates/web_research/src/web_research_orchestrator.rs:475-499).
   Decision: extend to a 3-tuple
     Result<(String, SourceTier, &'static str), WebResearchOrchestratorError>
   where the third element is the static-str fetcher tag ("obscura" or
   "reqwest"). Why a 3-tuple over a new FetchedDoc struct: minimal LOC;
   keeps the function pure; matches the existing tuple pattern; avoids
   introducing a new public type that other crates would need to import.

2. Tag values are static strings.
   "obscura" set inside the cfg-gated obscura branch
   (web_research_orchestrator.rs:479-488). "reqwest" set on the
   non-obscura code path. Why static strings: zero alloc; the tag
   semantics are closed (only two backends today); SSB writer converts
   to owned String once at WebResearchStepPayload assembly.

3. New field on WebResearchStepEvent (not a tuple at every call site).
   WebResearchStepEvent today has no `fetcher` field
   (web_research_orchestrator.rs:82-90). Decision: add
   `pub fetcher: String` to that struct, set it in node_loop after
   capturing the third tuple element from fetch_and_extract.
   Why a field, not threading the tuple deeper: emit_step + task_runner
   already consume WebResearchStepEvent shape; one struct extension
   keeps the dataflow uniform. SerDe defaulting is already standard in
   adjacent structs (verified at ssb.rs:870-883 for WebResearchStepPayload).

4. Doc-comment fix is the only "doc-rot" change.
   ssb.rs:31 already runtime-equals 24 (verified by context-compiler).
   Two doc-comments still claim "stays at 23":
     orchestrator/tests/integration_checkpoint_success.rs:6
     orchestrator/src/task_runner.rs:164
   Fix to "stays at 24" matching runtime. NO SCHEMA_VERSION runtime bump
   in this patch (Track B item B.3 is the next bump 24->25 when MCP
   variants land).

5. Test upgrade scope.
   fetcher_used_ssb_contract.rs today is struct-round-trip-only.
   Upgrade to end-to-end: drive the writer with each backend set to
   reqwest then obscura; sqlite3-query the resulting SSB row and assert
   .fetcher matches. Use the existing test crate harness; no new deps.

EXPECTED TEST OUTCOMES
- cargo check --workspace clean
- cargo test -p elai_cc_contracts --test fetcher_used_ssb_contract: 2/2 pass
  (one assertion per backend)
- task_runner.rs no longer contains String::new() at the fetcher field
  assignment line
- Both doc-comments read "stays at 24"

UNKNOWNS / RISKS
- Whether the obscura cfg gate is feature-default-on or feature-default-off
  on the test harness. If off, the obscura assertion in the upgraded test
  must be feature-gated (#[cfg(feature = "obscura")]) so CI without the
  feature stays green. Auditor must verify.
- ssb_viewer.rs ResearchStepEntry lacks a fetcher field — INFO-level
  finding logged by context-compiler, OUT OF SCOPE for A.4-finish.
  Track D's dashboard wiring may surface it; Track A.4 leaves the SSB
  row populated so it is queryable downstream.

PRE-IMPLEMENTATION TOP-3 FAILURE MODES
1. Forgetting to thread the fetcher through ALL sibling call sites of
   fetch_and_extract. Mitigation: context-compiler already enumerated
   call sites; execution agent must verify with an elai.map grep.
2. Breaking the WebResearchStepEvent serialization shape. Mitigation:
   add fetcher with #[serde(default)] in case any older serialized
   events get replayed.
3. Doc-comment "fix" introducing a true comment vs. a string literal
   mismatch (ie editing a string that's compared at runtime). Mitigation:
   verified by context-compiler that both sites are doc-comments only.

OUT OF SCOPE
- Adding fetcher to ssb_viewer.rs (logged as INFO; for Track D)
- SCHEMA_VERSION runtime bump (Track B.3)
- Any change to the obscura.exe shim itself

NO commits. The user collects + commits manually.
