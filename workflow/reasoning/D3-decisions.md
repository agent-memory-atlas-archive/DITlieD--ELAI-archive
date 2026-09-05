D3: LLM-backed Phase F synthesis

WHAT: Replace deterministic stub in phase_f_synthesize with async LLM call via SidecarGroundingClient.

KEY DECISIONS:

1. Async runtime: The /research handler at cli/main.rs:4068 is in a sync fn (handle_slash_command).
   Decision: build a new_current_thread tokio runtime inside the /research handler (same pattern as
   D2 /task at line 4205 and fsm_transition_sync at line 3622). Avoids nesting runtimes (which panics).
   phase_f_synthesize and run_research_pipeline become async fn; runtime.block_on() at the call site.

2. Trait object vs generic: GroundingClient uses RPIT (impl Future in return), NOT object-safe.
   Decision: use generic C: GroundingClient for phase_f_synthesize and run_research_pipeline.
   Cannot use &dyn GroundingClient (spec says so but trait definition prohibits it).

3. LLM call output format: ground() expects JSON ResearchReport from sidecar, extracts .answer field.
   Decision: prompt instructs LLM to output JSON with {"answer": "<cited prose>", ...other fields...}.
   Parse via existing ground() -> extract report.answer -> check for [N] markers.
   On any GroundingError OR missing [N] markers OR DETERMINISTIC_FALLBACK sentinel: use deterministic path.

4. Prompt structure: inline in research_pipeline.rs per MVP scope. Passes all claim strings with [N] markers.
   Instructs: "Output JSON with answer field containing 2-4 paragraphs, every sentence ending with [N]."
   Includes DETERMINISTIC_FALLBACK sentinel instruction.

5. ResearchReport.answer field: confirmed at research_tree.rs:682 (pub answer: String).

6. Send+Sync: GroundingClient trait explicitly declares Send + Sync at grounding_client.rs:101. Verified.

EXPECTED TEST OUTCOME: cargo test -p elai-orchestrator -- research_pipeline phase_f passes.
  Existing 178/180 elai-cli tests should hold (only /research handler changed).

UNKNOWNS: Qwen 3B JSON compliance without constrained decoding. Mitigated by deterministic fallback.

FAILURE MODES:
  1. LLM returns malformed JSON -> caught by ground() returning Err -> deterministic fallback fires.
  2. LLM returns valid JSON but missing [N] markers -> second guard catches it -> deterministic fallback.
  3. Runtime nesting panic -> avoided by new_current_thread in sync handler (no existing runtime there).
