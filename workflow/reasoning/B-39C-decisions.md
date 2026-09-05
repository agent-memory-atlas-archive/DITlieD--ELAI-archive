B-39C decisions
Session: s-34648
Date: 2026-05-11
Predecessor: B-39 (commit 6dd4af5, observability-only REFLECT pipeline)

WHAT
Wire ReflectOutput.final_reply into the user-visible reply path in LiveCli::run_turn.
Add 3 additive delegates on ConversationRuntime (push_user_text, push_assistant_text,
record_pipeline_usage). Add 4 contract tests. Update /reason help text. Document
text-only tradeoff in gotchas.md.

KEY DECISIONS (user-approved 2026-05-11)
D1: Three additive thin delegates on ConversationRuntime — chosen over
    re-constructing the runtime via into_session() + ::new(...) because the
    constructor requires re-threading api_client / tools / hooks / feature_config.
    Delegate path is ~15 LOC additive with zero behavior change for any existing
    caller.
D2 (D2a variant): Text-only /reason on. The substitute path bypasses the
    run_turn tool loop by design. REFLECT pipeline is a best-of-N text answer
    selector (TRIAGE -> REASONER N=4 -> AUDITOR -> SYNTHESIZE) and produces only
    text. Tool execution remains available via /reason off. Documented in
    gotchas.md and /reason help text. Future B-39E may explore D2f
    (system-hint injection) as opt-in.

EXPECTED TEST OUTCOME
- cargo check -p elai-cli -p elai-orchestrator -p elai-contracts: 0 errors,
  0 new warnings.
- cargo test -p elai-contracts --test chat_reflect_contract: 16/16 passing
  (12 existing + 4 new tests 13-16).
- cargo test -p elai-cli: no regressions.
- Manually: /reason on followed by a problem-class question shows pipeline
  reply (not a fresh model call); session history includes both user+assistant
  messages for the substitute turn so turn N+1 sees turn N coherently.

GROUNDING (PG-1 through PG-6, all resolved 2026-05-11 from current source)
PG-1 render.rs:251 — render_markdown(&self, markdown: &str) -> String CONFIRMED.
PG-2 live_cli.rs:1484 try_save_snapshot returns (); 1573 try_incremental_profile_refresh
     returns () — both fire-and-forget. No Result propagation needed.
PG-3 live_cli.rs:1716 — Spinner is created AFTER the reflect gate (line 1685-1713).
     Returning Ok(()) inside the if-let-Ok(rt) block exits before the spinner is
     constructed. No spinner cleanup needed in the substitute path.
PG-4 chat_reflect.rs:299-325 — pipe.run() returns ReflectOutput directly (NOT
     Result). Errors are caught in run() and collapsed into skipped=true
     passthrough. No caller-side error handling required for the .run call.
     SSB events emitted internally (Triaged at line 341, plus Candidate/Audited/
     Synthesized in subsequent stages); no caller-side emission needed.
PG-5 conversation.rs:17 — `use crate::usage::{TokenUsage, UsageTracker};`
     already imported. record_pipeline_usage delegate needs no new imports.
PG-6 grep persist_session 17 occurrences across cli/src; no test asserts an
     exact per-turn call count. Substitute path's persist_session() call is
     safely additive — same idempotent disk write as the runtime-Ok path.

UNKNOWNS
None blocking. The pipeline takes 12-30s on local hardware; user awareness is
already established (handoff line 113-118). No new spinner is added in B-39C
(B-39D will add per-stage progress UI).

OUT OF SCOPE (do not touch)
- reasoning/src/chat_reflect.rs (pipeline impl final for v0+v1)
- reasoning/src/config.rs (chat_reasoner / chat_auditor arms final)
- orchestrator/src/ssb.rs / ssb_viewer.rs (SSB action_types final)
- dashboard-v2/* (REFLECT panel reads via existing 10s polling)
- B-39E system-hint injection (separate B-N when scoped)
- B-39D per-stage progress UI (separate B-N)
