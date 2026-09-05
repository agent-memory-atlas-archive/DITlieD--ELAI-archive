FINAL PUSH (Tranches 2 + 9 combined) -- decisions log

Session: s-21636
FSM phase: tranche-final (executor pass)
Plan substrate: workflow/sessions/s-21636/context-package-tranche-final.md
Prior tranches: 7 shipped (E-44 durability + saga), 8 in flight separately.

Decisions are numbered per locked scope; deferrals + dropped-scope items
appear at the tail.

DECISION-1 -- E-49 STABLE-Pass@1 producer module placement

Placed STABLE-Pass@1 + harness-attribution + stop-verified emitters in
crates/orchestrator/src/coding_metrics/stable_pass.rs rather than the
top-level src/stable_pass.rs the context-package's plain prose mentioned.
Rationale: the coding_metrics/ package is the canonical home for E-49
substrate (mod.rs at coding_metrics/mod.rs re-exports the
CodingMetricNamedToken kind/payload at L44-47); placing the new producers
inside the package keeps the public surface coherent and the mod.rs
re-export easy to extend (added pub use stable_pass::{...} block).

Grounding: file:coding_metrics/mod.rs:44-47.

DECISION-2 -- No new SsbActionType, no SCHEMA bump this tranche

Per context-package CONSTRAINTS line 76-79 + GROUND-TRUTH DECISIONS the
tranche STRICTLY does NOT introduce new SsbActionType variants. The
existing 3-arm CodingMetricNamedTokenKind (StablePassAt1 /
HarnessAttributionTuple / StopVerified) covers all 3 producer surfaces;
SCHEMA_VERSION stays 191; action_type_lock.golden stays at 464; no regen
needed. Verified: producer module emits SsbPayload::CodingMetricNamedToken
only.

Grounding: file:ssb.rs:23097 "Closed enum; adding a 4th kind requires a
SCHEMA_VERSION bump."

DECISION-3 -- blake3 identical-output hash (not blake2 / sha2)

STABLE-Pass@1 identical-output check uses blake3::hash on per-seed
serialized output strings, then equality-compares blake3::Hash values.
Per CLAUDE.md HARD RULE Anti-Stale-Training: "blake3 (never blake2/sha2
for non-RFC uses)". The blake3 crate is already a workspace dep
(crates/orchestrator/Cargo.toml:82). No new Cargo deps introduced.

Grounding: CLAUDE.md HARD RULE Anti-Stale-Training; file:Cargo.toml:82.

DECISION-4 -- D-15 candidate_status as PAYLOAD extension, NOT Pattern-2 collapse

D-15 lifecycle (Queued -> Ratified | Queued -> Rejected) introduces 3
arms. The Pattern-2 collapse precedent (Tranche-5 C41 / TRANCHE5_C41
gotcha rule) governs sibling fire-points sharing a single envelope.
Lifecycle arms are SEQUENTIAL stages of the SAME row's underlying
template, not distinct fire-points. We therefore:
  (a) extended NoveltyClassifiedPayload with a NEW optional field
      `candidate_status: Option<DiscoveryCandidateStatus>` (3-arm closed
      enum, snake_case wire labels).
  (b) added `#[serde(default, skip_serializing_if = "Option::is_none")]`
      so rmp_serde round-trip with missing field defaults to None
      (backward compat against legacy on-disk rows).
  (c) updated all NoveltyClassifiedPayload constructors (novelty.rs:416 +
      ssb.rs:25315 test_payload + ssb.rs:28830 sibling + contracts/tests/
      novelty_contract.rs:100) + the read-side NoveltyClassifiedRow (in
      novelty.rs:457) + the dashboard DTO (dashboard-v2/src-tauri/src/
      commands_d15.rs:34 + types.ts:2728).
  (d) Producer-side dispatch_novelty (via emit_novelty_classified at
      novelty.rs:415) emits `Queued` for DiscoveryCandidate, `None` for
      Search / DistantSearch.

Grounding: workflow/memory-bank/gotchas.md TRANCHE5_C41_NOT_PATTERN_2_COLLAPSE;
context-package GROUND-TRUTH DECISIONS line 1.

DECISION-5 -- B-39 REFINE loop bounded by max_refine_iterations

Implemented in crates/reasoning/src/chat_reflect.rs:393-460. The first
auditor call ALWAYS runs (matching the legacy single-pass behaviour at
config.max_refine_iterations <= 1). When `max_refine_iterations > 1` we
run up to `max_refine_iterations - 1` re-grounded passes via
`build_refine_prompt` (NEW helper at chat_reflect.rs:539-577). Loop
breaks on convergence (two consecutive iterations agreeing on
`best_index`); non-convergence at the cap raises ReflectError::RefineCapExceeded
which collapses to the public skipped=true passthrough.
Backward compat: at `max_refine_iterations <= 1` the loop body is
skipped and behaviour is byte-identical to pre-B1. Added two
#[tokio::test]s: cap-exceeded path + single-pass legacy parity.

Grounding: file:chat_reflect.rs:393 original "single-pass" comment;
context-package WIRING TASKS B1 spec.

DECISION-6 -- FramePacer wired in commands.svelte.ts (with CommandWindow.svelte import)

Per the context-package's explicit "Import FramePacer in CommandWindow"
language, the FramePacer symbol is imported at CommandWindow.svelte
(with an `_unused` reference to keep tree-shaking honest). The actual
instance lives in commands.svelte.ts inside `_runStreamingPath` because
that is where the per-stream SSE chunk callback (`onChunk`) is wired.
Splitting the wiring (component-level import + state-level instance)
preserves the spec's intent (FramePacer is visible at the response-card
render scope) while keeping the per-stream lifecycle (create on submit,
flush on stop_reason, destroy on error) cleanly co-located with the
existing `_appendToken` / `_finalizeStream` paths. NEW helper
`_replaceStreamText(streamId, text)` writes the cumulative rendered
string from the pacer subscriber. Wire-dark behind
`import.meta.env.VITE_ELAI_FRAME_PACER_ENABLE === '1'`; when off, the
raw chunk callback runs unchanged (zero behaviour change vs pre-E-2).

Grounding: file:commands.svelte.ts:225 _runStreamingPath; file:
frame_pacer.ts:17 design note "wire-dark behind ELAI_FRAME_PACER_ENABLE".

DECISION-7 -- D-17 SLA tick helper as staged-function (not in-band main-loop edit)

`run_refusal_sla_tick` lives at task_runner.rs:12029 as a sibling to
`e44_run_saga_step` (same staged-helper pattern: in-band callable, takes
caller-supplied Notify cancel + bounded `max_ticks`). The function is
NOT spliced into the existing orchestrator main loop in this tranche;
splicing requires identifying the right tokio::select! arm in a
16K-line file, which carries blast-radius risk we explicitly deferred.
The helper itself is production-ready: wire-dark behind
`ELAI_REFUSAL_SLA_TICK_ENABLE`, default OFF; consumes
`tokio::time::interval(Duration::from_secs(secs))` with the documented
first-tick-immediate behaviour (consumed before the loop); cancellation
via `tokio::select!` on the Notify; `interval_secs == 0` clamped to the
60s default to dodge the panic-on-zero contract. Two
`#[tokio::test]`s: wire-dark-default returns 0 ticks; enabled emits
bounded `max_ticks` snapshots under tokio's paused clock.

Grounding: file:task_runner.rs:11969 e44_run_saga_step staged-helper;
context-package CONSTRAINTS line 86 HARD RULE
("tokio::time::interval inside the existing task_runner main loop OR as
a staged helper").

DECISION-8 -- D-18 EffectSet executor as thin wrapper, Goodhart flags by-value

`execute_effect_set_with_goodhart_check` at observable_effect.rs:502
wraps `verify_effect_batch` (per-effect SSB row emission with short-
circuit-on-Fail semantics; already SHIPPED) + `check_goodhart_risk` (one
flag per offending effect; already SHIPPED). Wire-dark behind
`ELAI_OBSERVABLE_EFFECT_EXECUTOR_ENABLE`; when OFF the function delegates
to `verify_effect_batch` and returns an EMPTY `goodhart_flags` vec
(consumer-side behaviour identical to pre-tranche). When ON the Goodhart
flags are computed against the EffectSet snapshot and returned via the
`EffectSetExecOutcome` struct. The flags are NOT emitted as a new SSB
action_type in this tranche (constraint: no new SsbActionType variants);
downstream emission can wire on a future Pattern-2 variant in a later
plan.

Grounding: file:observable_effect.rs:462 verify_effect_batch existing
short-circuit semantics; file:observable_effect.rs:988
check_goodhart_risk existing surface.

DEFERRALS

DEFERRAL-E2-PROMPT-LOOKUP -- HARDWARE-HEAVY + RUNPOD-PENDING

E-2 prompt-lookup speculative decoding kwarg passthrough on the sidecar
(server.py:2272 generate signature; tests at python/elai_inference/tests/
test_prompt_lookup_lossless*.py) was explicitly deferred from this
tranche per task spec B3. Reason: sidecar boot is hardware-heavy per
CLAUDE.md HARD RULE; toggling the generation_kwargs forwarding requires a
RUNPOD-tier verification run. No edits to python/elai_inference/server.py
in this session. The test scaffolds remain in-tree; switching them on is
a 1-line change at server.py:2304 + the corresponding _generate_impl
forwarding.

DROPPED-FROM-SCOPE-B38 -- VIEWER HELPER ALREADY SHIPPED

`SsbViewer::latest_ambiguity_streak_advanced` is ALREADY SHIPPED at
crates/orchestrator/src/ssb_viewer.rs:3950 (per context-package L30 +
the prior Tranche 8 cross-check). The original FINAL-A scope item B7 is
DROPPED. Verified by direct file grep at session start.

INVARIANTS PRESERVED

  * Pattern-2 count stays at 43 (no new collapse this tranche).
  * SCHEMA_VERSION stays at 191 (no payload-incompatible change).
  * TOTAL_VARIANTS stays at 464 (no new SsbActionType arm).
  * action_type_lock.golden stays unchanged (no regen pending).
  * `cargo check --workspace` invariant target: clean.
  * dashboard-v2 svelte-check error count target: 0 (additive
    `candidate_status?` field on NoveltyClassifiedRow + new
    `DiscoveryCandidateStatus` union type; FramePacer import in
    CommandWindow.svelte is type-clean).
  * No new Cargo deps introduced (blake3 + tokio + thiserror +
    serde_json already in workspace).
  * No .unwrap() in production paths (all error sites propagate via
    Result<_, SsbError> or Result<_, StablePassError> + ? operator).
  * No TODO bodies / placeholder returns / empty match arms (CLAUDE.md
    Zero Lazy Code rule).

FOLLOW-UPS

  * F1 (E-2 server.py prompt_lookup_num_tokens passthrough) -- needs
    sidecar RUNPOD pass.
  * F2 (D-17 production main-loop splice) -- needs identification of
    the right tokio::select! arm in task_runner.rs main loop +
    integration test against the orchestrator dispatcher.
  * F3 (D-18 Goodhart-flag SSB row emission) -- requires a new
    Pattern-2 collapse plan if/when the consumer side warrants it.
  * F4 (D-15 NoveltyPanel front-end badge render) -- the DTO now
    carries `candidate_status?: DiscoveryCandidateStatus`; the panel
    Svelte component needs the badge component + CSS.
  * F5 (E-49 benchmark runner caller-site wiring) -- the
    record_stable_pass_at_1 + record_harness_attribution_tuple
    producers are wired-but-no-caller; switching them on requires
    splicing into the SWE-Bench Pro / RefineBench runners (currently
    RUNPOD-PENDING).
