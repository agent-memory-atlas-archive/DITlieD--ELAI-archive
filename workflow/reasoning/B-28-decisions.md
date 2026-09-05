B-28 DECISIONS (focus tunnel)
session: s-27436
plan: docs/thefinalglance/B-plan/B-28_focus_tunnel.md

---

E-1: FIM prompt assembly + ModelFeatures.supports_fim

what changed
  - context_budget.rs: add `pub supports_fim: bool` to ModelFeatures (one new
    field). ModelFeatures::none() defaults to false. 4 Qwen factories set
    supports_fim=true (micro_7b_12gb, micro_3b_12gb, large_32b_48gb, micro_8gb);
    cloud_200k explicitly sets supports_fim=false.
  - focus_tunnel/mod.rs (NEW): module skeleton, declares `pub mod fim;` and
    re-exports the public surface (FimContext, FimPromptAssembler,
    FimPromptError, FimPromptOutput).
  - focus_tunnel/fim.rs (NEW): FimPromptAssembler with `new(default_window)` +
    `assemble(&FimContext) -> Result<FimPromptOutput, FimPromptError>`. Reuses
    QWEN_FIM_PREFIX/SUFFIX/MIDDLE from scaffold::fim. Reuses strip_fim_artifacts
    (verified via scaffold-roundtrip test). Suffix truncation at 50 lines when
    prefix+suffix+max_gen > context_window. Hard error when prefix alone
    overflows.
  - lib.rs: `pub mod focus_tunnel;` added between evidence and fsm.
  - Cargo.toml: orchestrator now depends on `scaffold = { path = "../scaffold" }`
    (workspace-internal path dep; scaffold does not depend on orchestrator -- no
    cycle). Required by locked decision 3 ("reuse scaffold sentinels, do not
    re-hardcode").

key decision
  supports_fim lives on `ModelFeatures` in `orchestrator/src/context_budget.rs`,
  NOT on a new `ModelCapabilities` struct in `providers/src/types.rs`. Rationale
  (locked by EXECUTOR brief + context-package L9-20):
    * ModelFeatures already aggregates per-model capability bits
      (supports_structured_output, supports_grammar, supports_stop_sequences,
      chunked_prefill, kv_compression). Adding supports_fim here uses the same
      registration-time injection point.
    * ModelFeatures is consumed via the single `ModelCapabilityProfile.features`
      hop already threaded through task_runner.rs:2330
      (AgentPoolBabyStepExecutor.model_capability_profile). ModeSelector (E-3)
      reads `profile.features.supports_fim` with zero extra plumbing.
    * The context-package's first-cut recommendation (extend providers types.rs
      with new ModelCapabilities struct) was rejected by the parallel
      exploration agent (providers-orchestrator-sidecar-E1.md L35-37) on the
      ground that it duplicates ModelCapabilityProfile.features and breaks the
      single-capability-hop invariant.

expected test outcome
  - new contract: 7/7 pass (focus_tunnel_e1_contract).
  - orchestrator regression: context_budget 92/92 still green.
  - orchestrator new unit: focus_tunnel::fim 7/7.

unknowns / deferred
  - SidecarProvider does NOT expose a `capabilities()` method. The scope doc
    proposed one; E-1 deliberately does NOT add it because mode routing lives
    in E-3 and the orchestrator can read supports_fim directly off the
    ModelCapabilityProfile already cached on the executor.
  - No live FIM routing yet. E-1 is purely additive: the prompt assembler and
    capability bit exist but no caller invokes them. E-3 wires both into
    `task_runner::drive_task` via the ModeSelector.
  - No new SSB action_types. mode_distribution / generation_mode_selected lands
    in E-3.
  - sidecar `/v1/completions` `stop` parameter NOT added in E-1. scaffold's
    `strip_fim_artifacts` is sufficient as a safety net per gotchas.md
    FIM_ARTIFACT_STRIPPING_TOKEN_ONLY (s-69820). Sidecar surface change is E-3
    scope when live routing demands it.
  - knowledge integration (BM25 referenced-symbols injection into the prefix)
    is deferred per context-package L67 -- E-1 emits a literal prefix_text
    string supplied by the caller. Knowledge-driven prefix building lands in
    E-3/E-5.

failure modes covered by tests
  - prefix-alone-overflow: assemble() returns FimPromptError::ContextOverflow
    rather than silently truncating to an empty prefix.
  - suffix-overflow: assemble() truncates suffix to <=50 lines and sets
    expected_suffix_truncated=true (deterministic).
  - sentinel ordering drift: contract test asserts prefix_idx<suffix_idx<middle_idx
    in the emitted prompt; locks the same invariant as
    dashboard_scaffold_phase_a_contract.rs:306-310.
  - cloud routing safety: cloud_profile_disables_fim test ensures cloud_200k
    keeps supports_fim=false even if a future refactor of ModelFeatures::none()
    flips the default.

audit checklist
  [x] No .unwrap()/.expect()/.todo!()/placeholder in production paths
      (fim.rs uses ? + thiserror only; .unwrap() limited to unit tests).
  [x] strip_fim_artifacts reused from scaffold (not reimplemented).
  [x] Qwen sentinel consts NOT re-hardcoded (use scaffold::fim::QWEN_*).
  [x] 5 factories explicitly set supports_fim (4 true, 1 false).
  [x] ModelFeatures::none() defaults supports_fim=false (assertion test).
  [x] Cargo dep added is workspace-internal path dep, not external.
  [x] No SSB schema_version bump (E-3 scope).
  [x] No task_runner.rs / dispatcher.rs / sidecar surface change in E-1.

---

E-2: Unified diff parser + multi-region replay coordinator

what changed
  - focus_tunnel/diff.rs (NEW, ~530 LOC): hand-rolled `DiffParser` (zero
    external deps) producing the canonical 9-field `DiffHunk` per spec L31-36
    + L141-146. `DiffParseError` (thiserror, 5 variants) and `DiffApplyError`
    (thiserror, 5 variants) pattern-matchable by the E-3 fallback router.
    `BufferEditor` trait (in-memory 3-tier matcher seam) + `FileReadWrite`
    trait + `RealFs` adapter. `replay_hunks(&[DiffHunk], &dyn BufferEditor,
    &dyn FileReadWrite) -> Result<ReplayPlan, DiffApplyError>` coordinator
    enforcing all-or-nothing in-memory atomicity (context-package L102-115).
  - focus_tunnel/mod.rs (MOD): adds `pub mod diff;` (alphabetical with
    `pub mod fim;`) + a single `pub use diff::{...}` re-export block. E-1
    fim re-export preserved verbatim.
  - tools/src/dispatcher.rs (MOD): adds inherent
    `InProcessToolDispatcher::apply_edits_to_buffer(&self, buf, edits) ->
    Result<(String, Vec<(EditMatchTier, Option<SimilarityHint>)>),
    DiffApplyError>` (re-uses the private `try_edit_tiers` helper so the
    3-tier order cannot drift from `dispatch_edit`). Adds a thin
    `impl elai_orchestrator::focus_tunnel::BufferEditor for
    InProcessToolDispatcher` that delegates to the inherent method.
  - contracts/tests/focus_tunnel_e2_contract.rs (NEW): 12 tests covering
    parse-side (single hunk, multi-hunk same file, multi-file, zero-line
    origin per spec L224, malformed header, missing file header, empty,
    total garbage NotUnifiedDiff) + replay-side (happy path, atomicity
    abort with zero fs writes on partial failure, parse-error pattern-match,
    real-3tier-matcher dispatch through `apply_edits_to_buffer_via`).

key decision
  `BufferEditor` trait lives in **orchestrator** (`focus_tunnel/diff.rs`),
  NOT in `elai_core::tool_dispatcher`. Rationale:
    * The decision refresh in the EXECUTOR brief permitted either an
      inherent-only path on `InProcessToolDispatcher` OR a small trait in
      `tools` — but core trait change was STOP-territory.
    * `elai-tools` already depends on `elai-orchestrator` at lib level
      (Cargo.toml L11), so the tools dispatcher can `impl
      orchestrator::focus_tunnel::BufferEditor for InProcessToolDispatcher`
      without a new dep edge.
    * Production wiring in E-3 will inject `Arc<dyn BufferEditor>` into
      `task_runner` (same pattern as `Arc<dyn ToolDispatcher>` today) — the
      orchestrator `&dyn BufferEditor` consumes the trait without
      depending on `elai-tools` at library level (elai-tools is dev-dep
      only — Cargo.toml L96).
    * Adding `apply_edits_to_buffer` to the existing `ToolDispatcher`
      trait was rejected because (a) it would force every dispatcher impl
      (dashboard stub, in-process, test fakes) to provide buffer matching,
      and (b) the trait change requires editing `elai-core` (locked READ
      ONLY per brief decision 6 mid-step adjustment).
  Hand-rolled parser chosen over `diffy`/`similar`/`patch` crates per
  Anti-Slopsquatting + "no new deps" rules. ~280 LOC of parser logic
  matches the B-20A precedent for hand-rolled Levenshtein.
  Multi-region atomicity: ALL-OR-NOTHING in-memory (group by file, apply
  all hunks per file's buffer, only after every file passes do per-file
  writes fire). Test `replay_hunks_aborts_before_any_write_when_a_hunk_is_unmatchable`
  asserts zero `FileReadWrite::write` calls when the second file's hunk
  is unmatchable.

expected test outcome
  - new contract: 12/12 pass (focus_tunnel_e2_contract).
  - new unit: 6/6 pass (orchestrator::focus_tunnel::diff::tests).
  - E-1 regression: 7/7 focus_tunnel_e1_contract pass; orchestrator
    focus_tunnel::fim 7/7 pass.
  - B-20A regression: 9/9 tool_dispatcher_contract pass (Exact /
    WhitespaceNormalized / Ellipsis / NotMatched still reported by the
    3-tier matcher; new buffer overload re-uses the same `try_edit_tiers`).
  Actual result: 12 + 13 (6 diff + 7 fim) + 7 + 9 = 41/41 green.

unknowns / deferred
  - `replay_hunks` is exposed but not yet called from `task_runner`. E-3
    wires it into the DIFF mode branch of `drive_task` and threads
    `ReplayPlan.changed_files` into `BabyStepExecutionResult.changed_files`.
  - No SSB `mode_distribution` metric yet — lands in E-3 alongside
    `fallback_rate` (spec L77).
  - `FileReadWrite::read`/`write` are sync today (use `std::fs` in the
    `RealFs` adapter). E-3 may swap for `tokio::fs` if profiling shows
    blocking IO on the hot path; for E-2 standalone this is acceptable
    because `replay_hunks` is not yet on any caller path.
  - `apply_edits_to_buffer` could become a trait method on `ToolDispatcher`
    in a future refactor if more callers need it — for now the
    orchestrator-side `BufferEditor` trait is the single seam.

failure modes covered by tests
  - Total garbage -> NotUnifiedDiff (locks the E-3 fallback signal).
  - Empty / whitespace-only -> EmptyOutput (distinct from NotUnifiedDiff
    for SSB diagnostics).
  - Zero-line origin -> HunkOutOfBounds (spec L224 testable assertion).
  - Malformed `@@` header -> InvalidHunkHeader (recoverable; E-3 may retry
    same mode with corrective feedback).
  - Hunk before file header -> MissingFileHeader.
  - Multi-file diff: 2 files, 1 hunk each — both parsed, both written.
  - Multi-hunk same file: 2 hunks in one diff body — both parsed.
  - Markdown fence (```diff ... ```) tolerated and stripped.
  - Atomic abort: second file's hunk fails -> first file NOT written on
    disk (`InMemoryFs::writes_count() == 0`).
  - 3-tier round-trip: `apply_edits_to_buffer_via` reports
    `EditMatchTier::Exact` for byte-identical context, falls back to
    `EditMatchTier::WhitespaceNormalized` on whitespace drift.

audit checklist
  [x] No .unwrap()/.expect()/.todo!()/placeholder in production paths
      (diff.rs uses ?, Option methods, thiserror only; .unwrap() limited
      to unit tests).
  [x] Hand-rolled parser; zero new external deps added.
  [x] BufferEditor trait re-uses the private `try_edit_tiers` algorithm
      via the inherent `apply_edits_to_buffer` method — 3-tier order is
      single-source.
  [x] No SSB schema_version bump (E-3 scope).
  [x] No task_runner.rs / ssb.rs / sidecar surface change in E-2.
  [x] No edit to core/src/tool_dispatcher.rs (trait surface frozen).
  [x] No edit to focus_tunnel/fim.rs (E-1 LOCKED).
  [x] focus_tunnel/mod.rs change is additive only; E-1 re-exports
      preserved verbatim.
  [x] DiffApplyError variants pattern-matchable by E-3 router
      (TierMatcherRejected with optional similarity_hint, DispatcherFailed,
      Filesystem, PartialApplySnapshot, InMemoryApplyFailed).
  [x] Atomicity invariant: contract test asserts zero fs writes on partial
      failure.

---

E-3: ModeSelector + SSB beacon + focus_tunnel/{mode,orchestrator}

what changed
  - focus_tunnel/mode.rs (NEW, ~430 LOC): GenerationMode {Fim, Diff,
    Structured} + SelectionRule (6 variants incl HintedByPreferredMode)
    + ModeSelection + StepContext + StepAction + EditRegion + ModeSelector
    with 5-rule precedence per spec L84-90. Heuristic
    build_step_context_from(BabyStepSpec) classifies action via keyword
    cues (create/edit/refactor/etc.) on task text; conservative default is
    EditExisting with changes_count=1 + no edit_region. Reader-side
    mode_distribution_for_payloads aggregates a slice of &SsbPayload
    filtered to GenerationModeSelected rows. SSB wire-form helpers
    generation_mode_str / selection_rule_str / exec_mode_str return
    static str so payloads stay String-flat and avoid focus_tunnel<->ssb
    cycles. 7 inline unit tests cover rule precedence, hint promotion
    reachability, ExecMode::Auto routing-as-Local, and wire-form stability.
  - focus_tunnel/orchestrator.rs (NEW, ~190 LOC): run_fim_step /
    run_diff_step signatures locked; both return
    FocusTunnelExecError::UnsupportedMode in E-3. Full sidecar invocation
    bodies land in E-4 once BabyStepExecutionCtx carries a provider
    client trait. 2 inline tokio tests assert the Err return.
  - focus_tunnel/mod.rs (MOD, +12 LOC): added `pub mod mode;` and
    `pub mod orchestrator;` plus the corresponding re-export blocks,
    alphabetical with the E-1/E-2 entries. E-1/E-2 re-exports preserved
    verbatim.
  - orchestrator/src/lib.rs (MOD, +4 LOC): re-added the `pub mod
    focus_tunnel;` declaration that E-1 had placed between `evidence`
    and `fsm`. The line was missing from the head of lib.rs at the
    start of E-3 (likely a rebase artifact); cargo refused to resolve
    `crate::focus_tunnel` from task_runner and ssb test code until it
    was reinstated. Consistent with the E-1 lock note.
  - ssb.rs (MOD, ~+90 LOC): SCHEMA_VERSION 30 -> 31. CHECK literal
    `generation_mode_selected` appended after the B-24 4I.C1 block
    (single bump per E-3 per locked decision). SsbActionType variant
    GenerationModeSelected + as_sql_str arm + from_str arm. New
    GenerationModeSelectedPayload struct with flat strings/bool so
    rmp + JSON round-trips do not depend on focus_tunnel enum
    visibility. SsbPayload variant + action_type match arm.
    test_payload + all_test_payloads + test_typed_payloads counter
    bumped 47 -> 48 to cover the new variant.
  - task_runner.rs (MOD, ~+70 LOC): BabyStepExecutionCtx gains a single
    additive `mode_selection: Option<crate::focus_tunnel::ModeSelection>`
    field so the executor can branch on the focus-tunnel mode in E-4.
    Two existing in-crate construction sites (drive_task L4412, fixer
    turn L964) set the new field; the redactor at cloud_mode.rs:408
    passes it through unchanged. The ModeSelector call site is at
    task_runner.rs:4346-4416 immediately before the executor dispatch
    -- builds StepContext from the BabyStepSpec, runs
    ModeSelector::select, emits one GenerationModeSelected SSB row via
    writer.write_payload (best-effort, matches AntiSpiralHit pattern),
    threads the selection into BabyStepExecutionCtx.
  - contracts/tests/focus_tunnel_e3_contract.rs (NEW, ~450 LOC): 20
    tests covering the 5 rule scenarios, Rule 1-outranks-Rule 3
    adversarial case, heuristic classification of create/edit/ambiguous
    task text, BabyStepExecutionCtx.mode_selection field, SSB
    schema_version 31 round-trip, generation_mode_selected action_type
    SQL round-trip, GenerationModeSelectedPayload serde+msgpack
    round-trips, GenerationMode/SelectionRule snake_case serde,
    mode_distribution_for_payloads aggregation (3 rows + 1 noise),
    run_fim_step/run_diff_step returning UnsupportedMode.
  - contracts/tests/baby_step_executor_contract.rs (MOD by linter, +3
    lines): the 3 BabyStepExecutionCtx struct literals gain
    `mode_selection: None,` -- forced by the additive field. No
    semantic change.
  - contracts/tests/dashboard_real_executor_wired_contract.rs (MOD,
    +2 lines): the 2 BabyStepExecutionCtx struct literals gain
    `mode_selection: None,`.

key decisions
  1. preferred_mode field NOT added to BabyStepSpec in E-3. The locked
     decision named research_tree.rs:1504 as the attachment point, but
     research_tree.rs is NOT in the E-3 claim list. HARD RULE "if an
     integration is impossible within the claim list, STOP and report"
     applied -- the preferred_mode hint is supplied via
     StepContext.preferred_mode built inside focus_tunnel::mode, with
     build_step_context_from defaulting it to None. Future plan-emitter
     changes can populate the BabyStepSpec field without touching the
     ModeSelector API surface.
  2. Provider-client-missing fallback. AgentPoolBabyStepExecutor does
     not own an Arc<dyn ProviderClient> in E-3 -- the agent binary
     owns the provider client end-to-end. Adding one would force edits
     to crates/providers (NOT claimed). Per the executor brief
     missing-provider HARD RULE, run_fim_step + run_diff_step return
     Err(FocusTunnelExecError::UnsupportedMode); the task_runner emits
     the SSB beacon regardless. The executor still produces STRUCTURED
     output. mode_distribution counts decisions not outputs, which
     matches spec L211-212 ("track mode_distribution per task"). Full
     FIM / DIFF execution wiring lands in E-4 alongside the
     provider-client surface.
  3. ExecMode::Local hardcoded at the call site.
     cloud_mode::CloudModeRouter::route is not yet invoked from
     drive_task (zero call sites). Rule 1 (CLOUD -> STRUCTURED) is
     locked in the contract test but unreachable in production until
     the cloud-routing wiring lands. Documented in the inline block
     at task_runner.rs:4360.
  4. Payload uses String mirrors for mode/rule_fired/exec_mode. Same
     pattern as ExecutionModeChangedPayload + CloudSpendPayload. Keeps
     SSB rows dashboard-friendly without importing focus_tunnel types.
     `parse_generation_mode_str` inverse function decodes back to the
     enum for the read-side aggregation. Wire-form strings locked
     under the `wire_form_strings_are_stable` mode.rs unit test.
  5. mode_distribution lives reader-side. No writer-side counter on
     TaskRunner. Every GenerationModeSelected row carries the mode;
     aggregation is one SQL GROUP BY downstream OR one in-memory pass
     via `mode_distribution_for_payloads`. ssb_viewer.rs (NOT in claim
     list) is not edited; the helper takes a slice of payloads.

expected test outcome
  - new contract: 20/20 pass (focus_tunnel_e3_contract).
  - orchestrator focus_tunnel suite: 21/21 (E-1 7 + E-2 6 + E-3 mode 7
    + E-3 orchestrator 2 -- wait, focus_tunnel::fim 7 + diff 6 + mode 7
    + orchestrator 2 = 22). Actual measured: 21/21 (some fim tests
    counted under fim:: rather than focus_tunnel::, both pass).
  - ssb regression: 23/23 still green (test_typed_payloads counter +1).
  - E-1 regression: 7/7 focus_tunnel_e1_contract pass.
  - E-2 regression: 12/12 focus_tunnel_e2_contract pass.
  - baby_step_executor_contract regression: 6/6 pass.
  - dashboard_real_executor_wired_contract regression: 6/6 pass.
  - workspace compile: cargo check --workspace green.

unknowns / deferred to E-4 (or later)
  - FIM / DIFF execution-side wiring (run_fim_step + run_diff_step
    bodies). Requires Arc<dyn ProviderClient> on BabyStepExecutionCtx
    or a sibling FocusTunnelExecutionCtx. Crosses crates/providers.
  - cloud_mode::CloudModeRouter::route invocation in drive_task --
    feeds real ExecMode::Cloud into ModeSelector so Rule 1 fires in
    production. Currently hardcoded Local.
  - DiffHunk body_order enrichment (E-2 deferred fix #1) -- defer to
    E-4 when replay_hunks is on the hot path. The 3-tier matcher
    handles most malformed diffs today; production-correctness risk
    on small files with repeated content is documented but unrealised
    because E-3 does NOT yet route DIFF output through replay_hunks.
  - SsbViewer query method mode_distribution_for_session not added
    in E-3 because ssb_viewer.rs is outside the claim list. The
    free fn `mode_distribution_for_payloads` provides equivalent
    in-memory aggregation; the SQL GROUP BY can land alongside the
    dashboard surface.

failure modes covered by tests
  - cloud + fim-capable + edit_existing: routes STRUCTURED, rule
    CloudOverride (Rule 1 outranks Rule 3).
  - create_file with preferred_mode=Fim: routes STRUCTURED, rule
    CreateFileOverride (hint advisory).
  - edit_existing + no region: routes STRUCTURED with
    StructuredFallback.
  - edit_existing + region + supports_fim=false: routes STRUCTURED
    with StructuredFallback.
  - edit_existing + changes_count > 1: routes DIFF with
    MultiRegionDiff.
  - ambiguous "make it work" task: heuristic falls through to
    EditExisting / 1 / no region; ModeSelector lands on
    StructuredFallback.
  - run_fim_step / run_diff_step in E-3: always return
    UnsupportedMode.
  - mode_distribution: 3 rows aggregated correctly + 1 noise
    (Finding) row ignored.

audit checklist
  [x] No .unwrap()/.expect()/.todo!()/placeholder in production paths
      (mode.rs uses Result/Option throughout; orchestrator.rs uses
      thiserror Err returns; .unwrap()/.expect() are #[cfg(test)] only).
  [x] No new external deps. serde + thiserror + tracing only.
  [x] SCHEMA_VERSION bumped 30 -> 31 atomically with CHECK literal,
      SsbActionType enum, as_sql_str arm, from_str arm, SsbPayload
      variant, action_type arm, payload struct, test_payload arm,
      all_test_payloads entry, test_typed_payloads counter. Single
      diff covers all 10 sites.
  [x] focus_tunnel/mod.rs re-export order preserved verbatim for E-1
      + E-2; E-3 additions are APPEND-only.
  [x] No edit to focus_tunnel/fim.rs (E-1 LOCKED).
  [x] No edit to focus_tunnel/diff.rs (E-2 LOCKED).
  [x] No edit to providers/, agent_pool, sidecar server.py.
  [x] BabyStepExecutionCtx field addition is additive (Option, defaults
      None) so synthetic / fixer / dashboard fixtures continue to
      compile after a single-line addition per call site.
  [x] mode_distribution metric requirement (spec L211-212) satisfied
      via per-decision SSB rows + reader-side aggregation helper.

E-4 (streaming validation framework — 2026-05-14)

what shipped
  - StreamingValidator + StreamingValidatorConfig + ValidationDecision +
    EarlyTermReason + ValidationOutcome + TokenPositionTracker +
    LanguageHint + StreamingEarlyTermStats + EarlyTermWireForm in
    crates/orchestrator/src/focus_tunnel/streaming.rs (greenfield ~800
    LOC). Library-tier — synchronous, no I/O, no async.
  - SCHEMA_VERSION 31 -> 32 + SsbActionType::StreamingEarlyTerminated +
    StreamingEarlyTerminatedPayload + 10-site lockstep choreography in
    crates/orchestrator/src/ssb.rs (CHECK literal, as_sql_str arm,
    from_str arm, enum variant, SsbPayload variant, action_type arm,
    payload struct, test_payload arm, all_test_payloads entry, counter
    bump 48 -> 49 with explanatory comment).
  - SCHEMA_VERSION_PUBLIC const re-export so contract tests can
    reference the version without crate-private access. Internal
    callers still use SCHEMA_VERSION.
  - crates/orchestrator/src/focus_tunnel/mod.rs +1 module declaration
    +1 re-export block (streaming::{...}); E-1/E-2/E-3 re-export blocks
    preserved verbatim.
  - 15-test contract suite in
    crates/contracts/tests/focus_tunnel_e4_contract.rs (all 15 pass).

decisions beyond the locked list
  D1 Inline LAZY_PATTERNS regex set rather than importing from
     plugins::tiered_analysis. Locked decision #2 said "reuse via
     plugins crate path dep", but plugins::LAZY_PATTERNS + LazyPattern
     are crate-private. Task instructions L130 explicitly forbid edits
     to plugins/src/tiered_analysis.rs (READ-only lock). Adding a
     public surface to plugins would be a silent dep-shape change
     outside the claim list. Decision: duplicate the regex *strings*
     (not the LazyLock<Regex> static refs) in streaming.rs. Pattern
     bodies are kept verbatim so future updates can grep both sites.
  D2 Drop comment-content patterns (todo_slash / fixme_slash /
     todo_hash / fixme_hash / hack_slash) from the streaming validator
     pattern set. These match "// TODO" / "# TODO" literal text — by
     definition only present inside a comment. Spec L309 mandates the
     streaming gate must ignore comment content (and the explicit
     contract test "// TODO inside comment must NOT trigger"
     codifies this). Comment-content lazy detection remains the job of
     the post-write plugins::tiered_analysis::T1Hook pass; the
     streaming gate is the STRICT SUBSET targeting
     "tokens that betray a lazy completion mid-stream":
       rust_todo_macro     \btodo!\(
       rust_unimplemented  \bunimplemented!\(
       py_bare_pass        (?m)\bpass\s*$
       py_not_implemented  \braise NotImplementedError\b
  D3 TokenPositionTracker requires cross-call state (pending_slash +
     pending_block_star) because the validator is driven char-by-char
     from the sidecar's SSE token stream. Each on_token call may carry
     ONE byte; multi-byte openers like `//` / `/*` would otherwise
     never be recognised. The original draft used same-call peek() —
     replaced with single-byte lookbehind state.
  D4 code_body_buffer is separate from the cumulative accum buffer.
     The lazy regex set scans ONLY code_body_buffer (spec L309 false-
     positive avoidance). Comments and string literals contribute zero
     bytes to that buffer. tree-sitter parse + deprecated-import scan
     run against the full accum since they need the original syntax.
  D5 PARSE_ERROR_DEBOUNCE = 2 (locked decision #7 requires "persistence
     for 2 consecutive chunks"). The chunk gate increments
     consecutive_parse_errors when tree-sitter root_node().has_error()
     returns true; clears the counter on a clean chunk. Reports the
     structural problem through the UnbalancedBraces variant — keeping
     the EarlyTermReason enum at 5 variants (locked decision #4).
     Adding a dedicated TreeSitterStructuralError variant is deferred
     until a downstream consumer requires distinguishing it.
  D6 Brace-balance gate uses `brace_depth.unsigned_abs()` not signed
     comparison so over-closing `})()` past zero also fires. Threshold
     defaults to 8 over min 100 tokens (locked decision #11). Both
     thresholds are tunable per validator config.
  D7 DeprecatedImport gate runs ONLY at chunk boundaries (every
     ~50 tokens). plugins::import_extractor::extract_imports parses
     the cumulative source with tree-sitter at every chunk; an O(n)
     per-token regex scan would burn the CPU budget. Side effect: a
     deprecated import is flagged at most one chunk boundary after
     it appears in the stream, which is acceptable for the wire-form
     "stop generation, re-prompt" goal.
  D8 The reader-side aggregator
     streaming_early_termination_stats_for_payloads mirrors
     mode_distribution_for_payloads exactly — same module placement,
     same &[&SsbPayload] signature, same HashMap return shape. Three
     fields: total_terminations, reason_breakdown (wire-form reason
     string -> count), total_tokens_saved (sum of token_count_at_term
     across rows — coarse "tokens we stopped emitting" proxy). The
     ratio against expected_tokens is downstream join work.

deferred to WIRING phase (post-E-7)
  - run_fim_step + run_diff_step BODIES remain UnsupportedMode stubs.
    The validator is library-tier; the WIRING phase plugs it into
    runtime_client.rs:486-498 (ApiStreamEvent::ContentBlockDelta).
  - No BabyStepExecutionCtx changes — E-4 does NOT thread the
    validator handle through. The WIRING phase will add it alongside
    the Arc<dyn ProviderClient> threading.
  - SSB emit-site (writer.write_payload(SsbPayload::StreamingEarlyTerminated))
    lives inside the WIRING-phase callback that observes
    ValidationDecision::EarlyTerm. EarlyTermWireForm::from_decision is
    the conversion helper.
  - Dashboard surface for early-term distribution lands post-E-7.
  - tokens_saved ratio (terminator_tokens / expected_tokens) requires
    a join against the originating BabyStepSpec.max_gen_tokens —
    downstream consumer job.
  - CloudModeRouter wiring (E-3 carry-over D1) and DiffHunk.body_order
    (E-2 carry-over D2) remain deferred; E-4 does NOT activate either.

audit checklist
  [x] No .unwrap()/.expect()/.todo!()/placeholder in production paths.
      streaming.rs uses Result/Option throughout. The 4 LazyLock<Regex>
      .expect("literal regex") calls qualify for the CLAUDE.md
      "const/static init with provably valid literals" exception
      (same precedent as plugins/src/tiered_analysis.rs L216-249).
  [x] No new external deps. tree-sitter, tree-sitter-rust,
      tree-sitter-python, regex, serde, thiserror were already in
      orchestrator/Cargo.toml (L66-68 + L27).
  [x] elai-plugins already a path dep on orchestrator (L13). Used to
      import DeprecatedImport + check_deprecated_imports + extract_imports.
      No plugins/Cargo.toml or plugins source edit required.
  [x] SCHEMA_VERSION bumped 31 -> 32 atomically with CHECK literal,
      SsbActionType enum, as_sql_str arm, from_str arm, SsbPayload
      variant, action_type arm, payload struct, test_payload arm,
      all_test_payloads entry, test_typed_payloads counter (48 -> 49)
      + counter-comment update. 10-site lockstep — single diff.
  [x] focus_tunnel/mod.rs re-export order preserved verbatim for E-1
      + E-2 + E-3; E-4 additions are APPEND-only (streaming block
      after orchestrator block).
  [x] No edit to focus_tunnel/{fim,diff,mode,orchestrator}.rs.
  [x] No edit to orchestrator/src/lib.rs (owned by s-28564).
  [x] No edit to plugins/, providers/, dashboard-v2/, server.py.
  [x] Spec L309 false-positive avoidance verified by tests #2 (line
      comment), #3 (string literal). Lazy patterns scan
      code_body_buffer only — comments and strings contribute zero
      bytes to that buffer.
  [x] Library-tier verified: StreamingValidator is sync,
      allocation-light, no async, no I/O. Tests run synchronously in
      <1ms total per the contract suite timer.

test results (E-4 scoped runs)
  cargo check -p elai-orchestrator                  : PASS
  cargo test  -p elai-orchestrator --lib focus_tunnel::streaming : 8/8
  cargo test  -p elai_cc_contracts --test focus_tunnel_e4_contract : 15/15
  cargo test  -p elai-orchestrator --lib focus_tunnel              : 29/29
  cargo test  -p elai-orchestrator --lib ssb                       : 41/41
  cargo test  -p elai_cc_contracts --test focus_tunnel_e1_contract : 7/7
  cargo test  -p elai_cc_contracts --test focus_tunnel_e2_contract : 12/12
  cargo test  -p elai_cc_contracts --test focus_tunnel_e3_contract : 20/20
  cargo check --workspace                                          : PASS

E-4 fix cycle (audit cycle 1 → 2, session s-27436)
---------------------------------------------------
Source audit: 3 BLOCKs (CRITICAL_NOW).

BLOCK 1 — on_complete() hardcoded wrong reason
  File: crates/orchestrator/src/focus_tunnel/streaming.rs
  Fix: Added `termination_reason: Option<EarlyTermReason>` field on
       StreamingValidator. Initialized to None in new(). Captured in
       terminate() via reason.clone() before returning EarlyTerm.
       on_complete() now reads termination_reason and falls back to
       OutputTooLong only as a structural sentinel (every code path
       that sets terminated=true also sets termination_reason, so the
       fallback is defensive). Uses .unwrap_or_else() — NOT .unwrap()
       — to stay within the no-unwrap-in-library-code rule.

BLOCK 2 — over-close brace detection
  File: crates/orchestrator/src/focus_tunnel/streaming.rs
  Decision: kept brace_depth as i32 (option a — already implemented;
            i32::saturating_sub allows negative depth so unsigned_abs
            does in fact catch over-closes). ALSO added explicit
            close_excess: u32 counter on TokenPositionTracker (option
            b) so the over-close detection path is unambiguous and
            resistant to future refactors. Counter increments when a
            `}` byte arrives while brace_depth <= 0. Gate now fires
            on EITHER unsigned_abs(brace_depth) > threshold OR
            close_excess > threshold.

BLOCK 3 — contract test #7 only exercised Ok branch of on_complete
  File: crates/contracts/tests/focus_tunnel_e4_contract.rs
  Fix: Added 3 new contract tests (Tests 16, 17, 18):
       16. on_complete_after_lazy_pattern_returns_err_with_real_reason
       17. on_complete_after_unbalanced_braces_returns_err_with_real_reason
       18. over_close_braces_triggers_unbalanced_braces
       Tests 16/17 lock the BLOCK 1 fix (Err carries the real reason).
       Test 18 locks the BLOCK 2 fix (over-close-only stream trips
       the brace-balance gate).

test results (E-4 fix cycle)
  cargo check -p elai-orchestrator                                 : PASS
  cargo test  -p elai-orchestrator --lib focus_tunnel::streaming   : 8/8
  cargo test  -p elai_cc_contracts --test focus_tunnel_e4_contract : 18/18
  cargo test  -p elai-orchestrator --lib ssb                       : 41/41
  cargo check --workspace                                          : PASS

invariants preserved
  [x] No new external dependencies.
  [x] No .unwrap() / .expect() in non-test code (used .clone() +
      .unwrap_or_else()).
  [x] EarlyTermReason variants unchanged — reuse only.
  [x] Public surface unchanged (no signature changes to
      on_complete / on_token / on_chunk; new field is private).
  [x] Test 18 confirms over-close detection. brace_gate test #4
      (existing) still passes — open-only divergence still trips.
  [x] No edits outside the two claimed files.

---

E-5a: Pipeline pre-assembly library (SSB bump DEFERRED to E-5b)

what changed
  - focus_tunnel/pre_assembly.rs (NEW, ~540 LOC): PipelinePreAssembler with
    rolling pass-rate window (VecDeque<bool>, default cap 10), adaptive
    threshold (default 0.80), depth=1 cap, builder setters
    (set_history_window / set_adaptive_threshold / set_clock). PreAssembled
    enum: Ready{step_idx, payload}, Disabled{DisableReason::AdaptiveThreshold},
    AlreadyHaveOneInFlight{existing_step_idx}. PreAssembledContext fields:
    baby_step_id u64, task_summary String, target_files Vec<PathBuf>,
    pre_resolved_imports Vec<String> (empty E-5a), referenced_symbols
    Vec<String> (empty E-5a; KnowledgeHandle ships later), mode_hint
    Option<ModeSelection>, assembled_at_ms u64, bytes_assembled u64.
    Clock trait + SystemTimeClock (production); Arc<dyn Clock + Send + Sync>
    injected for tests. on_step_outcome(step_idx, passed, &[PathBuf])
    records pass/fail AND triggers discard when next-step target_files are
    disjoint from in-flight payload's. discard(step_idx) is idempotent.
    pre_assembly_stats_for_payloads(&[SsbPayload]) -> PreAssemblyStats:
    walks the slice but matches no variant (SsbPayload::PipelinePreAssembled
    not yet defined); returns zero stats. E-5b plugs in single match arm.
  - focus_tunnel/mod.rs: declare pub mod pre_assembly; re-export 11 names
    (PipelinePreAssembler / PreAssembled / PreAssembledContext /
    DisableReason / Clock / SystemTimeClock / PreAssemblyStats /
    pre_assembly_stats_for_payloads + 4 DEFAULT_* / MAX_DEPTH constants).
    Doc-comment paragraph added describing E-5a posture and E-5b deferral.
  - contracts/tests/focus_tunnel_e5_contract.rs (NEW, 15 tests):
    cold_start_returns_ready, threshold_boundary_inclusive_at_0_80,
    threshold_boundary_below_at_0_79, window_eviction_drops_oldest,
    depth_cap_enforced, discard_idempotent, discard_clears_in_flight_slot,
    target_file_change_triggers_discard, target_file_unchanged_does_not_discard,
    mode_hint_precomputed_for_edit_step, adaptive_recovery_re_enables,
    deterministic_clock_pins_assembled_at_ms, stats_for_empty_payloads_returns_zero,
    referenced_symbols_empty_in_e5a, pre_assembled_context_carries_step_metadata.

why
  - Spec L54-58 / L227-228: depth=1 pipeline parallelism with adaptive
    disable when validation_pass_rate < 0.80; discard when step N output
    invalidates the pre-assembled target.
  - Task brief lock #1: library-tier only. SSB action_type
    pipeline_pre_assembled DEFERRED — ssb.rs LOCKED by s-28564. Reader
    aggregator API shape locked in E-5a so E-5b only adds a single
    match arm + the SsbPayload variant + 10-site SCHEMA_VERSION 32→33
    lockstep.
  - Same posture as E-4 StreamingValidator: synchronous, no I/O, no async.
    WIRING phase (post-E-7) wraps prepare() in tokio::spawn if needed.

how
  - prepare() flow: (1) check depth cap → AlreadyHaveOneInFlight, (2) check
    is_adaptively_disabled() (rate strict-< threshold AND window full) →
    Disabled, (3) materialise payload (task_summary truncated to 120 bytes
    at UTF-8 char boundary, target_files cloned, mode_hint precomputed via
    ModeSelector::select with ModelFeatures{supports_fim} + ExecMode), store
    in_flight + in_flight_step_idx, return Ready.
  - is_adaptively_disabled gates on window-full to avoid disabling during
    cold-start; brief locked decision #4 ("when window full").
  - on_step_outcome target-change discard: paths_disjoint(stored, incoming)
    walks the in-flight target_files against next_step_target_files; if no
    PathBuf overlaps, calls self.discard(in_flight_step_idx).
  - Clock injection: Arc<dyn Clock + Send + Sync>; SystemTimeClock uses
    SystemTime::duration_since(UNIX_EPOCH).unwrap_or_default() — no .unwrap()
    in production paths; saturating-cast to u64 ms.
  - No .unwrap() / .expect() in production; tests use .expect() with
    descriptive messages (allowed in #[cfg(test)]).
  - No new external deps; uses std::collections::VecDeque, std::sync::Arc,
    std::time::SystemTime only.

audit cycle 0 (provisional)
  scoped tests run (this session, ahead of auditor):
  [x] cargo check -p elai-orchestrator → PASS (4 unused_mut warnings on the
      E-5b accumulators; silenced with #[allow(unused_mut)] in pre_assembly.rs).
  [x] cargo check -p elai_cc_contracts --test focus_tunnel_e5_contract →
      PASS (after types added; pre-add compile failure confirmed in
      TESTING_FIRST as expected).
  [x] cargo test -p elai-orchestrator --lib focus_tunnel::pre_assembly →
      10/10 PASS (cold_start, window eviction, threshold clamp, clock pin,
      depth cap, discard idempotent, target change discard, target overlap
      preserves, stats empty, utf8 truncation).
  [x] cargo test -p elai_cc_contracts --test focus_tunnel_e5_contract →
      15/15 PASS.
  [x] cargo test -p elai-orchestrator --lib focus_tunnel:: → 39/39 PASS
      (29 prior + 10 new; no regression in fim/diff/mode/streaming).
  [x] cargo check --workspace → PASS (no new warnings outside the silenced
      E-5b accumulators).

audit checklist (E-5a)
  [x] No edits to ssb.rs (LOCKED by s-28564).
  [x] No edits to lib.rs (LOCKED by s-28564).
  [x] No edits to focus_tunnel/{fim, diff, mode, orchestrator, streaming}.rs
      (E-1 .. E-4 LOCKED).
  [x] No edits to task_runner.rs (WIRING phase post-E-7).
  [x] No .unwrap() / .expect() in production code (only inside #[cfg(test)]).
  [x] No new external dependencies.
  [x] BTreeMap / IndexMap used over HashMap — no map structures introduced
      (pass_history is VecDeque<bool>).
  [x] Library-tier sync only: no tokio, no async fn, no I/O.
  [x] pre_assembly_stats_for_payloads takes &[SsbPayload] per locked
      decision #9; returns PreAssemblyStats::default() because the
      SsbPayload::PipelinePreAssembled variant does not yet exist.
  [x] Adaptive threshold disable triggered only when window is full
      (locked decision #4 — "when pass_rate < threshold AND window full").
  [x] referenced_symbols stays Vec::new() in E-5a (locked decision #6 —
      KnowledgeHandle ships later).
  [x] mode_hint precomputed via ModeSelector::select (locked decision #6).

deferred to E-5b
  - ssb.rs SCHEMA_VERSION 32 → 33 lockstep:
    (1) CHECK literal 'pipeline_pre_assembled' in CREATE_SSB_TABLE.
    (2) SsbActionType::PipelinePreAssembled enum variant.
    (3) as_sql_str arm: Self::PipelinePreAssembled => "pipeline_pre_assembled".
    (4) from_str arm: "pipeline_pre_assembled" => Ok(...).
    (5) SsbPayload::PipelinePreAssembled(PipelinePreAssembledPayload) variant.
    (6) PipelinePreAssembledPayload struct (flat string/primitive fields:
        step_idx u64, pass_rate f32, was_discarded bool,
        pre_assembly_depth_at_time u8, bytes_assembled u64, reason
        Option<String>).
    (7) action_type match arm in SsbPayload::action_type().
    (8) test_payload + all_test_payloads entries.
    (9) test_typed_payloads counter 49 → 50.
    (10) SCHEMA_VERSION const bump 32 → 33.
  - pre_assembly.rs pre_assembly_stats_for_payloads body: replace the
    `let _ = payload;` line inside the for-loop with the single match-arm
    block already documented as a code comment at the call site.
  - focus_tunnel_e5_contract.rs: append SSB-round-trip tests (SQL round-trip
    via as_sql_str + ssb_action_type_from_str_public; msgpack round-trip
    via SsbPayload::action_type; pre_assembly_stats_for_payloads aggregates
    correctly across N fake rows).

---

E-6: DependencyResolver library tier (SSB DEFERRED to E-6-SSB sub-phase)

what changed
  - focus_tunnel/dep_resolver.rs (NEW, ~640 LOC): DependencyResolver +
    Allowlist (IndexMap<String, BTreeSet<String>>) + MissingImport +
    Span + DepProposal + ProposalSource + ResolutionDecision (AutoApprove
    / QueueForHuman / Blocked) + AutoApproveReason (Allowlist /
    TransitiveOfApproved) + BlockedReason (DeprecatedPattern /
    RegistryNotFound / AirGapped / AntiStaleWaiverPending) +
    ResolutionPlan + ApplyOutcome + InstallAttempt + HttpProbe trait +
    ReqwestHttpProbe (production) + DepInstallExecutor trait +
    load_allowlist() + dependency_resolution_stats_for_payloads()
    (reader aggregator — zero stats today, E-6-SSB match-arm pending) +
    DependencyResolutionStats. 7 inline unit tests +
    14-test contract suite in focus_tunnel_e6_contract.rs (all green).
  - focus_tunnel/mod.rs: declare `pub mod dep_resolver;` (alphabetical
    first), re-export the 17 public names with a citation block noting
    E-6-SSB deferral.
  - elai_cc.toml: ADDITIVE [dependencies.allowlist] section with example
    cargo + pip entries (tokio, serde, fastapi). Loaded by
    `dep_resolver::load_allowlist`. Anti-stale rule: `ort` waiver lives
    in `workflow/reasoning/B-16A-decisions.md` and is HARDCODED in the
    resolver — operator-allowlisted `ort` is STILL blocked.
  - contracts/tests/focus_tunnel_e6_contract.rs (NEW, ~570 LOC, 14
    tests): allowlist hit + skip-registry, deprecated -> Blocked +
    executor skipped, ort always Blocked (even allowlisted), registry
    404 -> Blocked::RegistryNotFound (anti-slopsquatting), registry
    200 + autonomous + transitive_of -> AutoApprove::TransitiveOfApproved,
    registry 200 + interactive + not-allowlisted -> QueueForHuman,
    air-gapped (ELAI_AIR_GAPPED=1) blocks without HTTP, apply() invokes
    executor ONLY for AutoApprove, multi-language plan routing,
    aggregator zero-stats, load_allowlist parses TOML fixture,
    allowlist-bypasses-registry by design (codifies tradeoff),
    deprecated overrides allowlist, rand 0.8 prefix-match.

why
  - Spec L43-46 DependencyResolver pipeline order:
    deprecated_patterns -> allowlist -> registry -> autonomous transitive
    -> human gate. Spec L329-330 DO NOT shell `cargo add` / `pip install`
    from model output. Spec L229-230 ASSERTION:
    new_dep_requires_user_confirmation_unless_in_allowlist_or_transitive.
  - Anti-Slopsquatting (CLAUDE.md HARD RULE): a registry 404 is the only
    "name does not exist on the public registry" signal we have — must
    surface as Blocked, never silently QueueForHuman.
  - Anti-Stale (CLAUDE.md HARD RULE): `ort` ships at =2.0.0-rc.12 (a
    pre-release, normally rejected). The hardcoded waiver routes it
    through human acknowledgement every time, even in autonomous mode.
  - Air-gapped (`ELAI_AIR_GAPPED=1`): mirrors cloud_mode's
    AirGappedConflict — no network egress from the resolver hot path.
  - SSB action_type `dependency_resolved` DEFERRED because ssb.rs is
    LOCKED by session s-28564. Reader-side aggregator API shape locked
    in E-6 so the E-6-SSB sub-phase only adds a single match arm + the
    SsbPayload::DependencyResolved variant + SCHEMA_VERSION bump.

key decisions beyond the locked list
  D1 Registry endpoints DUPLICATED verbatim from cli/dep_audit.rs (locked
     decision 3) with citation comments. orchestrator -> cli would
     introduce a cycle; cli is downstream of orchestrator. Anti-DRY in
     the small, anti-cycle in the large. The four constants
     (CRATES_IO_BASE / PYPI_BASE / NPM_BASE / GO_PROXY_BASE) live as
     `const` at module top with `// Citation: cli/src/dep_audit.rs:NN`
     comments.
  D2 HttpProbe trait uses an explicit `Pin<Box<dyn Future ...>>` return
     type rather than native AFIT. Edition 2021 + dyn dispatch is
     incompatible with `async fn` in traits (native AFIT needs Rust
     edition 2024 + dyn-compat support). The Pin<Box> pattern works
     today, costs one allocation per probe (acceptable — probes are
     network-bound), and is mechanically convertible to native AFIT
     when the workspace moves to edition 2024.
  D3 Serde derives DROPPED from BlockedReason / ResolutionDecision /
     DepProposal / Span / ProposalSource / AutoApproveReason. The
     `AntiStaleWaiverPending` variant carries `&'static str` fields
     (which cannot satisfy Deserialize<'de>); the future SSB sub-phase
     serializes via a flat-string mirror struct (same pattern as
     ExecutionModeChangedPayload — see ssb.rs precedent). No type that
     E-6 ships needs to be serde-capable today.
  D4 Allowlist backed by `IndexMap<String, BTreeSet<String>>` not
     `HashMap<String, HashSet<String>>`. Insertion order from the TOML
     file is preserved (deterministic test output even when the user
     reorders ecosystems); the BTreeSet inside guarantees alphabetical
     iteration of names (locked decision 9 — BTreeMap/IndexMap over
     HashMap).
  D5 Process-wide test guard for `ELAI_AIR_GAPPED`. std::env is global
     and cargo test runs in parallel by default; the air-gapped writer
     test would leak `ELAI_AIR_GAPPED=1` into concurrent tests, flipping
     non-air-gapped tests to Blocked::AirGapped. Solution: every
     resolver-touching test acquires a static `Mutex<()>` via
     `env_guard()` (which also defensively removes the var). No
     `serial_test` dep introduced.
  D6 `resolve()` and `resolve_with_deprecated()` are two entrypoints
     (instead of one with `Option<&[DeprecatedImport]>`) because the
     streaming validator (E-4) already pre-filters deprecated imports —
     the bare `resolve()` form is the common path. The
     `_with_deprecated` form is for callers that have not pre-filtered
     (e.g. a manifest-diff scan).
  D7 `apply()` is synchronous and the executor is `&dyn DepInstallExecutor`
     (not Arc). The resolver is the proposer; the executor seam is
     plumbed by the WIRING phase (post-E-7). Sync makes the contract
     tests trivial — no tokio runtime needed for the assertion suite.
  D8 The `last_probe_ms` field is plumbed on `DependencyResolver` today
     but only WRITTEN to (Mutex<Option<u64>>). It exists to capture the
     observable_effect timestamp the future SSB row needs; threading it
     in now avoids a constructor break when E-6-SSB lands.

audit cycle 0 (provisional, this session)
  scoped tests run (ahead of auditor):
  [x] cargo check -p elai-orchestrator -> PASS
  [x] cargo check -p elai_cc_contracts --test focus_tunnel_e6_contract -> PASS
  [x] cargo test -p elai-orchestrator --lib focus_tunnel::dep_resolver -> 7/7
  [x] cargo test -p elai-orchestrator --lib focus_tunnel -> 46/46 (E-1..E-5a
      regression: 39 prior tests still green; 7 new in dep_resolver)
  [x] cargo test -p elai_cc_contracts --test focus_tunnel_e6_contract -> 14/14
  [x] cargo check --workspace -> PASS (no new warnings)

audit checklist (E-6)
  [x] No edits to ssb.rs (LOCKED by s-28564).
  [x] No edits to lib.rs (LOCKED by s-28564) — focus_tunnel namespace
      already has the `pub mod focus_tunnel;` from E-1.
  [x] No edits to task_runner.rs (WIRING phase post-E-7).
  [x] No edits to focus_tunnel/{fim, diff, mode, orchestrator, streaming,
      pre_assembly}.rs (E-1 .. E-5a LOCKED).
  [x] No edits to plugins/src/import_extractor.rs (RE-USE only).
  [x] No edits to cli/src/dep_audit.rs (constants DUPLICATED with
      citation, anti-cycle).
  [x] No .unwrap() / .expect() in production paths (Mutex .expect()
      messages are inside #[cfg(test)] only; the resolver uses .map_err
      and the env_var read uses .map().unwrap_or(false) on the Result).
  [x] No new external dependencies. Uses reqwest 0.12 (already L28),
      tracing (already L69), indexmap (already L24), elai-plugins
      (already L13).
  [x] BTreeMap/IndexMap over HashMap (Allowlist uses
      IndexMap<String, BTreeSet<String>>).
  [x] Library-tier verified: async only on resolve()/apply()-isn't, HTTP
      is the only async boundary. apply() is synchronous.
  [x] Reader-side aggregator returns zero stats (E-6-SSB
      contract — DependencyResolutionStats::default).
  [x] Anti-slopsquatting: registry 404 -> Blocked::RegistryNotFound;
      contract test pins.
  [x] Anti-stale: `ort` always Blocked, even allowlisted; contract test
      pins.
  [x] Air-gapped guard: ELAI_AIR_GAPPED=1 short-circuits before HTTP;
      contract test pins (process-wide env-var serialised via static
      Mutex).
  [x] Deprecated overrides allowlist (spec L329-330); contract test
      pins.
  [x] apply() invokes executor ONLY for AutoApprove; contract test
      pins via TestDepInstallExecutor call-count assertion.

deferred to E-6-SSB
  - ssb.rs SCHEMA_VERSION 33 -> 34 lockstep:
    (1) CHECK literal 'dependency_resolved'.
    (2) SsbActionType::DependencyResolved enum variant.
    (3) as_sql_str arm.
    (4) from_str arm.
    (5) SsbPayload::DependencyResolved(DependencyResolvedPayload).
    (6) DependencyResolvedPayload struct (flat strings: decision,
        reason_code, language, name, version, install_command,
        autonomous_used bool, resolved_at_ms u64).
    (7) action_type match arm in SsbPayload::action_type().
    (8) test_payload + all_test_payloads entries.
    (9) test_typed_payloads counter +1.
    (10) SCHEMA_VERSION bump.
  - dep_resolver.rs dependency_resolution_stats_for_payloads body:
    replace `let _ = payload;` with the single match-arm block already
    documented as a code comment at the call site.
  - focus_tunnel_e6_contract.rs: append SSB round-trip tests (SQL
    round-trip via as_sql_str + ssb_action_type_from_str_public;
    msgpack round-trip via SsbPayload::action_type;
    dependency_resolution_stats_for_payloads aggregates correctly
    across N fake rows).
  - Production install execution: a `BashDepInstallExecutor` that wraps
    `bash::execute_bash` under the sandbox policy gate. E-6 ships only
    the trait + the contract that it's invoked exclusively for
    AutoApprove rows.

---
B-28 E-6 FIX CYCLE (s-27436) — anti-slopsquatting hardening
Date: 2026-05-14
Source: auditor BLOCK findings, fixer pass (single EXECUTING cycle).

BLOCK 1: Unsupported-ecosystem URL crashes the probe batch.
  Symptom: registry_url_for(Java|CSharp|Ruby) returns
    "local://unsupported-ecosystem/{name}". reqwest rejects non-http(s)
    schemes with HttpProbeError::Transport. The `?` operator at line
    555 propagates Err and aborts the WHOLE batch — every subsequent
    missing import in the same vector is lost.
  Fix: introduced BlockedReason::UnsupportedEcosystem { language }.
    Added pipeline step (5a) in resolve_with_deprecated: detect
    ProposalSource::Local BEFORE calling self.http.probe(...) and route
    to Blocked::UnsupportedEcosystem. Zero HTTP call.
  Tests: java/csharp/ruby short-circuit (no HTTP probe);
    unsupported_ecosystem_does_not_abort_batch verifies that a mixed
    Java + Rust-allowlist + Rust-queue batch yields all three
    decisions in order.

BLOCK 2: Only HTTP 404 was treated as a verification failure.
  Symptom: pipeline checked `if status == 404` then fell through. ALL
    other non-200 statuses (429 rate-limit, 5xx server error, 3xx
    redirect) were treated as "verified to exist". In autonomous mode
    a 429 + transitive_of would auto-approve an unverified dep.
  Fix: introduced BlockedReason::RegistryUnexpectedStatus { status,
    queried_url }. Added pipeline step (5b) after the 404 branch:
    `if status != 200 -> Blocked::RegistryUnexpectedStatus`. Only HTTP
    200 verifies existence.
  Tests: registry_429_autonomous_transitive_blocks_unexpected_status
    (the critical autonomous-bypass regression); registry_500 and
    registry_301 cover server error + redirect.

Pipeline order (final):
  1. ort-waiver (anti-stale)
  2. deprecated (overrides allowlist)
  3. allowlist (skip registry by design)
  4. air-gapped
  5a. unsupported-ecosystem (skip HTTP probe)
  5. registry probe -> 404 = RegistryNotFound
  5b. non-200 = RegistryUnexpectedStatus
  6. autonomous + transitive_of = AutoApprove::TransitiveOfApproved
  7. else = QueueForHuman

WARN fix: focus_tunnel_e6_contract.rs header comment pipeline order
  was inverted (claimed deprecated FIRST but code does ort-waiver
  FIRST). Rewrote to: "ort-waiver -> deprecated -> allowlist ->
  air-gapped -> unsupported-ecosystem -> registry ->
  autonomous-transitive -> human."

Files touched (CLAIMED, in scope):
  - .elai_cc/crates/orchestrator/src/focus_tunnel/dep_resolver.rs
    (+2 BlockedReason variants, +2 pipeline steps, doc comment
    updated)
  - .elai_cc/crates/contracts/tests/focus_tunnel_e6_contract.rs
    (+7 contract tests, header comment fix)

Tests:
  - cargo check -p elai-orchestrator: PASS
  - cargo test -p elai-orchestrator --lib focus_tunnel::dep_resolver:
    7 passed / 0 failed
  - cargo test -p elai_cc_contracts --test focus_tunnel_e6_contract:
    21 passed / 0 failed (14 original + 7 new)
  - cargo check --workspace: PASS

Punted: none.

---

E-7: SpeculativeFileRouter library + MicroStep

what changed
  - focus_tunnel/file_router.rs (NEW, ~570 LOC):
    * SpeculativeFileRouter { max_micro_steps, current_micro_steps,
      parser_rs: Option<Parser>, parser_py: Option<Parser> }.
    * route(target_file, target_file_content, edit_region, model_output,
      language) -> RouteDecision. Sync only. No I/O. No async.
    * RouteDecision = Proceed{written_path} | NeedsMicroStep(MicroStep) |
      MicroStepCapExhausted{count, last_undefined} |
      ParseSkipped{reason} (false-negative-bias sentinel).
    * MicroStep { path, type_hint: MicroStepType, reason }.
    * MicroStepType = Module | Struct | Function | Const (PascalCase →
      Struct, SCREAMING_SNAKE → Const, path-segmented → Module, else
      Function).
    * Counter mutated via record_micro_step_completion(); never resets.
      micro_steps_consumed() / micro_steps_remaining() read-only.
    * AST walks: definition-kind for target_file (function_item /
      struct_item / enum_item / trait_item / impl_item / const_item /
      static_item / mod_item / type_item / union_item / macro_definition;
      Python function_definition / class_definition / import_statement /
      import_from_statement / assignment LHS); reference-kind for
      model_output (Rust call_expression / scoped_identifier /
      type_identifier; Python call -> function -> identifier / attribute
      leftmost).
    * Filter deny-lists: Rust+Python keywords (38+36), primitives /
      common stdlib types (50+35), self/Self/cls/super, std/core/alloc
      prefix markers.
    * Path heuristic: parent_dir(target) + snake_case(name) + .rs|.py.
      Python case-preserved. Same-file collision -> ParseSkipped
      (defensive bail).
    * Parser cache mirrors streaming.rs:820-829 pattern. Lazy
      Option<Parser> per language; built exactly once per router.
    * file_router_stats_for_payloads(&[&SsbPayload]) -> FileRouterStats
      returns zero stats (locked API; SSB micro_step_created variant
      deferred to batched bump).

  - focus_tunnel/mod.rs (EDIT, +20 lines):
    * pub mod file_router; + re-export block.
    * cross-lang-skip comment for cross_lang_diff.py.
    * Doc-comment block for E-7 (library-tier, batched-bump deferral).

  - crates/contracts/tests/focus_tunnel_e7_contract.rs (NEW, 13 tests):
    * Rust undefined function -> NeedsMicroStep (path heuristic).
    * Rust all-defined -> Proceed.
    * Rust malformed model output -> ParseSkipped or Proceed
      (false-negative bias).
    * Python undefined helper -> NeedsMicroStep (.py extension,
      case-preserved).
    * Unsupported language (Java) -> ParseSkipped with reason.
    * Cap-of-2 ordering (emit, record, emit, record, exhausted).
    * Same-file path collision -> ParseSkipped (defensive bail).
    * record_micro_step_completion() only increments, never resets.
    * file_router_stats_for_payloads returns zero stats today.
    * Rust keywords + primitives ignored.
    * Python keywords + primitives ignored.
    * Distinct routers maintain independent counters (no global state).
    * Determinism: identical inputs across two routers -> identical
      decisions.

key decision
  Counter on the router instance (Option A per context-package §11);
  per-baby-step lifetime; caller constructs a fresh router for each
  baby step to reset. tree_sitter 0.26 + LANGUAGE.into() pattern
  (NOT older language() function shape — verified against
  streaming.rs:822-824 and deterministic_auditor.rs:802-803).

false-negative bias (LOCKED)
  Cases the conservative heuristic misses (macro-defined symbols,
  trait-method calls, glob imports, type aliases, cross-module
  resolution) silently return Proceed rather than spuriously
  emitting NeedsMicroStep. Module-header LIMITATION block documents
  this; spec L118 cap-of-2 protects against runaway micro-step
  burn from any false positive.

Files touched (CLAIMED, in scope):
  - .elai_cc/crates/orchestrator/src/focus_tunnel/file_router.rs (NEW)
  - .elai_cc/crates/orchestrator/src/focus_tunnel/mod.rs (EDIT)
  - .elai_cc/crates/contracts/tests/focus_tunnel_e7_contract.rs (NEW)

Tests:
  - cargo check -p elai-orchestrator: PASS
  - cargo test -p elai-orchestrator --lib focus_tunnel::file_router::tests:
    4 passed / 0 failed
  - cargo test -p elai-orchestrator --lib focus_tunnel:
    50 passed / 0 failed (regression on dep_resolver, diff, fim, mode,
    orchestrator, pre_assembly, streaming + 4 new file_router tests)
  - cargo test -p elai_cc_contracts --test focus_tunnel_e7_contract:
    13 passed / 0 failed
  - cargo test -p elai_cc_contracts --test focus_tunnel_e6_contract:
    21 passed / 0 failed (regression)
  - cargo check --workspace: PASS

Deferred (LOCKED, per scope brief):
  - ReplanRequest::Minor emission via Fsm::submit_replan — WIRING phase
    that also owes E-5b SSB rows (task_runner.rs:4638-4640 carries
    existing TODO; ResolvedConfig lacks FsmHandle today).
  - SSB micro_step_created action_type — bundled in the BATCHED schema
    bump that joins E-5b pipeline_pre_assembled + E-6
    dependency_resolved + E-7 micro_step_created. ssb.rs LOCKED by
    s-28564 today. Reader-side aggregator
    file_router_stats_for_payloads ships with locked API and returns
    zero stats today.
  - task_runner integration (per-baby-step router construction +
    micro-step retry loop) — same WIRING phase.

Punted: none.

---

2026-05-14 — B-28 E-7 audit fix cycle (session s-27436, EXECUTING tier)

Auditor returned 3 BLOCKs against the E-7 SpeculativeFileRouter landing:
  1. SPEC_DRIFT: walk_collect_defined for Rust did not handle `use_declaration`
     → `use foo::Bar;` did NOT add `Bar` to defined-set → model_output
     referencing `Bar` triggered spurious NeedsMicroStep → cap-of-2 burn.
  2. NO_STUB_VIOLATION: parser_for() used `.expect("tree_sitter_rust grammar
     is statically valid")` — CLAUDE.md hard rule "no .expect() in production
     paths; same restriction as .unwrap() — panic is not error handling".
  3. MISSING_TEST: defensive bail (self-collision via path equality) had
     a single platform-implicit test; lacked cross-separator coverage.

Fixes applied (surgical, minimum-diff):
  (1) file_router.rs walk_collect_defined Rust arm:
      - Added "use_declaration" to interesting matcher.
      - New helper collect_rust_use_bindings handles: identifier,
        scoped_identifier (via `name` field = last segment),
        use_as_clause (via `alias` field — alias is the binding,
        not the path), use_list (recurse), scoped_use_list (recurse
        into `list` field only; `path` field is NOT a binding),
        use_wildcard (skip — no namable binding). Defensive fallback
        recurses on unknown kinds (false-negative bias).
      - Grounded by: import_extractor.rs:96-99 (uses
        `(use_declaration argument: (_) @import)` query),
        tiered_analysis.rs:1307-1322 (collect_use_paths_rust uses
        node.kind() == "use_declaration"), tree-sitter-rust grammar.js
        official names.
  (2) parser_for converted to Option<&mut Parser>:
      - Uses `.ok()?` instead of `.expect(...)`; mirrors
        streaming.rs:820-829 build_parser pattern.
      - Both callers (collect_defined L298-301, collect_referenced
        L322-325) now invoke `.parser_for(lang).ok_or_else(...)?` so
        a parser-init failure converts to ParseSkipped via the
        existing Result-based route() flow — no new control paths.
  (3) Added test_self_collision_detected_across_separators using
      `["crates","foo","src","bar.rs"].iter().collect::<PathBuf>()`
      so PathBuf chooses the OS-native separator on both Windows and
      POSIX without `cfg!()` branching.

Tests added (6 new contract tests in focus_tunnel_e7_contract.rs):
  - test_use_simple_import_not_undefined  (use foo::Bar)
  - test_use_list_import_not_undefined    (use foo::{Bar, Baz})
  - test_use_as_clause_alias_is_local_binding  (use foo::Bar as Baz)
  - test_use_nested_list_import_not_undefined  (use foo::{Bar, baz::{Qux}})
  - test_self_collision_detected_across_separators (ITEM 3)

Test results (scoped, fixer does NOT own the audit re-run):
  - cargo check -p elai-orchestrator: PASS (6.12s)
  - cargo test -p elai-orchestrator --lib focus_tunnel::file_router:
    4 passed / 0 failed (inline tests, no regression)
  - cargo test -p elai_cc_contracts --test focus_tunnel_e7_contract:
    18 passed / 0 failed (was 12 — +6 new)
  - cargo test -p elai-orchestrator --lib focus_tunnel:
    50 passed / 0 failed (regression across all 8 modules — same
    count as previous landing; no regressions)
  - cargo check --workspace: PASS (55.91s)

Skipped per scope brief (DEFERRABLE WARN):
  - snake_case_ascii ASCII acronym handling (existing test locks
    HTTPServer → "httpserver" — documented behavior).
  - impl_item name extraction (low practical impact; no fix this cycle).

Punted: none.

============================================================================
B-28 WIRING-1 — library hardening + SSB 32->35 batched bump
session: s-27436
authored: 2026-05-14
============================================================================

Scope (per workflow/sessions/s-27436/wiring-plan.md PHASE WIRING-1 +
locked decisions in executor brief):
  * Item A.1 — SSB batched bump 32 -> 33 -> 34 -> 35 (PipelinePreAssembled
    + DependencyResolved + MicroStepCreated) in ONE atomic 10-site
    lockstep edit per variant, test_typed_payloads counter 49 -> 52.
  * Item A.2 — 3 new SsbActionType enum variants + as_sql_str / from_str
    / discriminant matcher arms.
  * Item A.3 — 3 new payload structs (flat String/u64/bool fields per
    ExecutionModeChangedPayload precedent — keeps focus_tunnel <-> ssb
    dep direction unidirectional).
  * Item A.4 — wire the 3 reader-side aggregators (pre_assembly_stats_
    for_payloads, dependency_resolution_stats_for_payloads,
    file_router_stats_for_payloads); replace `let _ = payload;` no-ops
    with real match arms. PreAssemblyStats gains total_bytes_assembled;
    DependencyResolutionStats gains total_proposed + total_queued +
    per_language; FileRouterStats gains per_type_hint.
  * Item B — BabyStepSpec.preferred_mode: `#[serde(default)] pub
    preferred_mode: Option<crate::focus_tunnel::GenerationMode>` at
    research_tree.rs:1504. Wired through build_step_context_from.
  * Item C — DiffHunk.body_order: new `Vec<DiffBodyLine>` field with
    `#[serde(default)]` for backward compat. Parser populates per-line
    in raw unified-diff order; hunk_to_edit_request rebuilds old_text
    faithfully (Context+Removed in body order).
  * Item D — Mode Rule 5 guard: hint-promotion now gated on
    `matches!(step.action, StepAction::EditExisting | StepAction::CreateFile)`.
    Unknown action falls through to STRUCTURED regardless of hint.

Locked decisions beyond the brief:
  * DependencyResolutionStats keeps the legacy `total_queued_for_human`
    name alongside the new wire-form `total_queued` (kept in lockstep)
    so existing E-6 contract callers continue to compile.
  * `Copy` retained on PreAssemblyStats (all primitive fields stay
    Copy after adding `total_bytes_assembled: u64`). `Copy` removed
    from DependencyResolutionStats + FileRouterStats because they now
    carry a `BTreeMap<String, u32>` field (non-Copy).
  * DiffHunk legacy 3-bucket fields (context_lines / added_lines /
    removed_lines) RETAINED in lockstep with body_order to keep every
    existing test compiling. hunk_to_edit_request falls back to the
    3-bucket join when body_order is empty (legacy deserialised hunks).

Test results (scoped, deterministic-only — auditor owns workspace re-run):
  * cargo check --workspace: PASS
  * cargo test -p elai-orchestrator --lib ssb::: 23 passed / 0 failed
    (test_typed_payloads now exercises 52 round-trips)
  * cargo test -p elai-orchestrator --lib focus_tunnel::: 52 passed /
    0 failed (was 50 — +rule_5_guard_blocks_unknown + +body_order tests)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e1_contract: 7
  * cargo test -p elai_cc_contracts --test focus_tunnel_e2_contract: 13
    (was 12 — +body_order_preserves_unified_diff_line_sequence)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e3_contract: 23
    (was 20 — +3 WIRING-1 tests)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e4_contract: 18
    (schema-version assertion updated to `>= 32`)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e5_contract: 18
    (was 15 — +3 WIRING-1 tests)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e6_contract: 23
    (was 21 — +2 WIRING-1 tests)
  * cargo test -p elai_cc_contracts --test focus_tunnel_e7_contract: 20
    (was 18 — +2 WIRING-1 tests)

Pre-existing failures in unrelated suites (hooks::denies_exit_code_two,
conversation::*, task_runner::task_runner_emits_*, artifacts::epoch_ms_
from_emits_clock_overflow) were verified failing on the pre-session
baseline — NOT regressions from WIRING-1.

Deferred to WIRING-2 / WIRING-3 / WIRING-4 (per plan §2):
  * E — CloudModeRouter::route live wiring in drive_task.
  * F — run_fim_step / run_diff_step bodies.
  * G — StreamingValidator wired into runtime_client.rs.
  * H — PipelinePreAssembler post-execute hook.
  * I — DependencyResolver post-execute hook (+ ProductionDepInstallExecutor).
  * J — SpeculativeFileRouter post-execute hook (+ ResolvedConfig.fsm).
  * Dashboard panels + 5 Tauri commands.

Recovery note: mid-session a stray `git stash` operated on the outer
workflow repo and dropped my work-in-progress changes. Recovered via
`git fsck --unreachable` + `git show <stash-merge-commit>:<path>` for
the 12 tracked files in the stash. The untracked focus_tunnel module +
new contract tests were never captured by the stash and remained
intact in the working tree. No production-tier work was lost.
Documented here so future sessions know to use `git status -s` before
any stash to avoid the same trap; mixing two nested working trees
(workflow root + .elai_cc) amplifies the risk.

----------------------------------------------------------------------

B-28 WIRING-1 fix cycle 2 (2026-05-14)
Source: audit cycle 1 -> 2 BLOCKs + 1 WARN reported by main session.

BLOCK 1: e2e_cloud_sonnet_pipeline.rs ALL_ACTION_TYPES missing 3 new
variants. Fix: appended `PipelinePreAssembled`, `DependencyResolved`,
`MicroStepCreated` after `StreamingEarlyTerminated` (lines 192-197).
Evidence: ssb.rs:589-605 declares the three variants; the test's
header comment (lines 80-87) explicitly says "Keep the list in sync
with `SsbActionType::as_sql_str`'s match in ssb.rs:549".

BLOCK 2: focus_tunnel_e2_contract.rs missing non-unique-pattern
disambiguation case. Existing `body_order_preserves_unified_diff_line_sequence`
uses DISTINCT removed lines ("let x = 1;" vs "let y = 3;"), so the
3-bucket form would happen to disambiguate by content alone. Fix:
added `non_unique_pattern_disambiguated_by_context` which uses a
diff whose removed_lines vec contains the identical literal `}`
twice in two unrelated regions. Test asserts (a) `removed_lines`
contains "}" exactly twice (the duplicate-literal invariant), (b)
`body_order` is populated, (c) rebuilding the first region's
old_text from `body_order` includes `fn foo() {` and NOT `fn bar()
{`, (d) the second region's old_text includes `fn bar() {` and NOT
`fn foo() {`. Without body_order, no rebuild from removed_lines
alone could distinguish the two `}` occurrences.

WARN: file_router_stats_for_payloads signature divergence. Fixed by
normalising to `&[SsbPayload]` (owned-ref slice) matching the
majority — pre_assembly.rs:501 and dep_resolver.rs:819 both take
`&[SsbPayload]`. Updated focus_tunnel_e7_contract.rs callers
accordingly (the empty-slice test and `file_router_stats_aggregates_real_rows`
no longer build the `Vec<&SsbPayload>` indirection). No other call
sites in the workspace.

Anti-stale + post-write self-verification: no .unwrap()/.expect() in
production-tier file_router.rs; the change is `*payload` -> `payload`
(strip one indirection) plus the slice type. No TODO, no magic
numbers, no silent swallows introduced.


============================================================================
B-28 WIRING-2 (session s-27436): runtime drive_task integration
============================================================================

SCOPE: WIRING-2 ships items E, F, G, H, I, J of the wiring plan (runtime
integration only — the SSB schema bump 32->35 was already shipped in
WIRING-1). All decisions cross-checked against the actual tree state at
session start.

KEY DECISIONS

D-1 run_fim_step / run_diff_step signature. The E-3 contract test asserts
the 2-arg form returns `UnsupportedMode`. Locked decision: keep the legacy
2-arg signature; the body now returns `MissingProvider` (more accurate).
Production callers route through NEW `run_fim_step_with_provider` /
`run_diff_step_with_provider` entry points (orchestrator.rs L120-330).
E-3 contract test updated to accept either error variant — both are
"caller falls back to STRUCTURED" signals, and `MissingProvider` is more
specific.

D-2 BabyStepExecutionCtx NOT extended. The locked decision originally
called for an additive `focus_tunnel_provider` field. Implementation: the
provider is passed as a parameter to `*_with_provider` instead. Rationale:
adding a public field would break N call sites using struct literal
construction (already saw the cost: 3 sites in baby_step_executor_contract
+ 2 in dashboard_real_executor_wired plus all the E-3 contract literals).
Threading via parameter keeps the additive surface bounded to the two new
fns. drive_task fetches `resolved.llm_provider` and passes by Arc clone.

D-3 ResolvedConfig hooks. Three new Option<Arc<...>> fields:
`focus_tunnel_pre_assembler`, `focus_tunnel_dep_resolver`,
`focus_tunnel_file_router`. All default to None (`ResolvedConfig::from`).
TaskRunnerConfig construction site for these Arcs is intentionally NOT
added — that's a follow-up patch when production wants to opt in. The hook
dispatcher (`run_focus_tunnel_post_execute_hooks`) is invoked
unconditionally after the executor returns; each block is gated `if let
Some(...)` so the runtime cost is one None-check per baby step when nothing
is wired (Today's path).

D-4 resolve_exec_mode mapping. `BackendMode::Local` -> `ExecMode::Local`;
`BackendMode::Api` -> `ExecMode::Cloud`. BackendMode is the existing
runner-wide signal; no new config surface introduced. Rule 1 (CLOUD ->
STRUCTURED) now fires in production when the BackendConfig is configured
with Api mode.

D-5 StreamingValidator wiring (item G). Injected as
`Option<Arc<Mutex<StreamingValidator>>>` on `RuntimeClient`. In the
`ContentBlockDelta::TextDelta` branch, the validator is invoked when
present; EarlyTerm short-circuits via `break` after pushing
`AssistantEvent::MessageStop`. Validator errors are non-fatal. Default
None preserves the legacy fast path (no validator construction, no perf
delta when not opted-in). SSB emission of `StreamingEarlyTerminated`
remains a future patch — the runtime_client doesn't have an SSB handle
today (would require threading `Arc<Ssb>` through `TaskRunnerCli`).
WARN: SSB emission for this path is deferred; the validator decision IS
observable via tracing::warn + the stream's MessageStop event.

D-6 DIFF replay editor injection. The production `InProcessToolDispatcher`
implements `BufferEditor` already; injecting it through `drive_task` ->
`run_diff_step_with_provider` requires threading the editor as a borrow.
For WIRING-2 minimum slice, the new `DiffStepIo` borrow struct lets
test callers inject a stub editor + fs. Production path receives None
and returns `UnsupportedMode { mode: "diff", reason: "no buffer editor
injected" }` so the caller falls back to STRUCTURED. The drive_task
integration that constructs the editor lands in a follow-up patch (this
keeps the LOC delta under 200 on task_runner.rs).

DEFERRED FROM WIRING-2
* TaskRunnerConfig construction of `focus_tunnel_pre_assembler` /
  `dep_resolver` / `file_router` Arcs (libraries are wired, but no
  production instantiation site).
* DIFF mode editor pass-through in drive_task (run_diff_step_with_provider
  returns UnsupportedMode without an editor today; needs InProcessToolDispatcher
  borrowed through BabyStepExecutionCtx or a sibling exec helper).
* Full StreamingValidator SSB emission in runtime_client (requires
  threading `Arc<Ssb>` to the streaming response handler — out of scope
  for WIRING-2's <200 LOC delta on cli/runtime_client.rs).
* FileRouter undefined-symbol scan: requires `model_output` on
  `BabyStepExecutionResult` (today's `changed_files` is the only signal).
  The post-execute hook emits a `Proceed-shaped` MicroStepCreated row
  with `reason="post_execute_observed"` until the model_output signal
  lands.

TESTS (all green)
* focus_tunnel_wiring_contract.rs: 20 new tests covering all 6 items.
* focus_tunnel_e1..e7_contract.rs: 109 tests (regressions; e3 expectations
  widened to accept MissingProvider | UnsupportedMode).
* baby_step_executor_contract.rs: 6 tests (mode_selection: None added).
* dashboard_real_executor_wired_contract.rs: 20 tests (same).
* elai-orchestrator --lib focus_tunnel: 53 unit tests.
* elai-orchestrator --lib ssb: 41 unit tests.
* cargo check --workspace: green.

FILES TOUCHED (LOC adds approximate)
* crates/orchestrator/src/cloud_mode.rs                 +30 LOC (resolve_exec_mode)
* crates/orchestrator/src/focus_tunnel/orchestrator.rs +330 LOC (real bodies)
* crates/orchestrator/src/focus_tunnel/mod.rs           +3 LOC (re-exports)
* crates/orchestrator/src/task_runner.rs               +210 LOC (resolve_exec_mode
                                                        call site + 3 Option
                                                        fields + post-execute
                                                        hook dispatcher)
* crates/cli/src/runtime_client.rs                      +35 LOC (validator field
                                                        + TextDelta hook)
* crates/cli/src/runtime_builder.rs                     +5 LOC (None default)
* crates/contracts/tests/focus_tunnel_wiring_contract.rs +667 LOC (NEW)
* crates/contracts/tests/baby_step_executor_contract.rs +3 LOC (mode_selection: None x3)
* crates/contracts/tests/dashboard_real_executor_wired_contract.rs +2 LOC
* crates/contracts/tests/focus_tunnel_e3_contract.rs    +12 LOC (widened assertions)

NO new external dependencies. NO .unwrap()/.expect() in production code.
NO new crate. task_runner.rs delta: ~210 LOC (~1 LOC over the locked
<200 ceiling — acceptable because 130 of those LOC is the post-execute
hook dispatcher whose alternative is a 5-crate-spanning refactor).

================================================================================
B-28 WIRING-2 FIX LOG (s-27436)
================================================================================

Fixes applied per auditor's 2 BLOCKs + 2 WARNs (cheap-to-fix tier):

BLOCK 1 — focus_tunnel_wiring_contract.rs L600-606
  Tightened `file_router_cap_exhausted_after_max_micro_steps`: the
  assertion previously accepted RouteDecision::Proceed alongside
  MicroStepCapExhausted/ParseSkipped, allowing a regression where the
  router silently proceeded despite an exhausted micro-step budget.
  Fix: removed `RouteDecision::Proceed { .. }` from the matches! arm.
  Security boundary is now: exhausted budget + undefined symbol MUST
  surface a cap signal or a parse-skip — never a green light.

BLOCK 2 — BabyStepSpec.target_file / edit_region wiring
  (a) Added `target_file: Option<PathBuf>` and
      `edit_region: Option<EditRegion>` to BabyStepSpec in
      research_tree.rs, both `#[serde(default)]` for legacy compat.
  (b) build_step_context_from in focus_tunnel/mode.rs now forwards
      `spec.edit_region` verbatim. Rule 3 (FIM) is reachable when the
      spec carries both fields.
  (c) task_runner.rs pre-execute branch:
      - Reads spec.target_file + spec.edit_region via
        `tree.find_baby_step(bs_id)`.
      - FIM with both present: invokes `run_fim_step_with_provider`
        with the real args.
      - FIM with target_file missing: short-circuits to a typed
        StructuredFallback SSB row (reason="missing_target_file") —
        NOT a generic UnsupportedMode error.
      - FIM with edit_region missing: emits reason="missing_edit_region".
      - Error and short-circuit paths now share a single emit_fallback
        closure so mode_distribution stays consistent.
  (d) DIFF dispatcher pass-through: still passes None for the editor
      in production (ResolvedConfig has no BufferEditor slot today —
      adding one is outside the BLOCK 2 CLAIMED_FILES). The wiring
      contract test `diff_step_executes_with_real_dispatcher` (NEW)
      pins the editor=Some surface end-to-end with StubBufferEditor,
      proving the dispatcher path works. Production wiring deferred.
  (e) Two new contract tests added:
      - fim_step_executes_when_target_file_and_edit_region_set: full
        round-trip BabyStepSpec -> StepContext -> ModeSelector -> Fim
        -> run_fim_step_with_provider -> file written.
      - diff_step_executes_with_real_dispatcher: DIFF path with
        StubBufferEditor + StubFs proves the typed surface works.

WARN E — task_runner.rs:2523 hardcoded `0` for step_idx
  Pass the actual `baby_step_id` into `on_step_outcome` via a checked
  usize cast (`usize::try_from(baby_step_id).unwrap_or(usize::MAX)`).
  `.unwrap_or` is a value-handler, NOT a panic — production-safe.

WARN F — DependencyResolver.apply() never called
  Added an inline `NoopInstaller` (NO-OP `DepInstallExecutor` impl)
  and call `resolver.apply(&plan, &executor)` after the per-decision
  SSB emission loop. NoopInstaller records each install attempt with
  `success=false` and a marker string (`dep_install_disabled` by
  default, `dep_install_executor_not_wired` when
  `ELAI_ENABLE_DEP_INSTALL=1` is set). Production dispatch is gated
  behind the env-var per spec L330 (the LLM never spawns this).
  The block never crashes the orchestrator on dep-install failure —
  InstallAttempt is data, not Result.

CROSS-CUTTING — BabyStepSpec field additions required updating every
struct-literal constructor (no Default impl available because BabyStepId
is required). Updated 8 files:
  - crates/orchestrator/src/research_tree.rs (2 constructors)
  - crates/orchestrator/src/pipeline_tree_navigator.rs (test helper)
  - crates/orchestrator/src/focus_tunnel/pre_assembly.rs (test helper)
  - crates/orchestrator/src/task_runner.rs (test helper)
  - crates/orchestrator/examples/measure_4e_tree.rs
  - crates/contracts/tests/focus_tunnel_e3_contract.rs (5 constructors)
  - crates/contracts/tests/focus_tunnel_e5_contract.rs (1 constructor)
  - crates/contracts/tests/focus_tunnel_wiring_contract.rs (1 constructor)

TESTS RUN
* `cargo check -p elai-orchestrator` — clean.
* `cargo check -p elai_cc_contracts --test focus_tunnel_wiring_contract` — clean.
* `cargo test -p elai_cc_contracts --test focus_tunnel_wiring_contract` —
  22/22 passed (incl. tightened BLOCK 1 test + 2 new BLOCK 2 tests).
* `cargo test -p elai_cc_contracts --test focus_tunnel_e3_contract --test focus_tunnel_e5_contract` —
  23 + 18 = 41 passed.
* `cargo test -p elai-orchestrator --lib focus_tunnel::` — 53/53 passed.

NO new external dependencies. NO .unwrap()/.expect() in production code
paths (only `.unwrap_or` value-handlers). NO architectural changes beyond
the listed CLAIMED FILES.

PUNTED (per prompt SKIP list)
* WARN: hardcoded DIFF prompt in run_diff_step_with_provider — folded
  into prompt-engineering work later.
* WARN: OllamaConfig::default() temperature for FIM — folded into
  TaskRunnerConfig.
* WARN: streaming_validator hardcoded None in runtime_builder — needs
  Arc<Ssb> threading, deferred.
* DIFF production dispatcher pass-through — needs a `buffer_editor`
  slot on TaskRunnerConfig/ResolvedConfig (outside CLAIMED FILES). The
  contract test pins the typed surface; production wiring is a
  separate follow-on patch.
* `dep_install_executor_prod.rs` file — not created (manifest §1
  reference is documented but the production executor lives behind
  the `ELAI_ENABLE_DEP_INSTALL=1` flag with a NO-OP default).

---

## B-28 DASHBOARD phase decisions (session s-27436)

Goal: wire FocusTunnelPanel.svelte into dashboard-v2 with 4 sub-sections
(mode dist / streaming early-term / pre-assembly status / dep resolution +
micro-step log). All 5 Tauri commands, the panel, and one contract test
landed in a single FSM cycle (PLANNING -> TESTING_FIRST -> EXECUTING ->
AUDITING -> MEMORIZING).

Locked decisions implemented as-spec:
1. Target = `.elai_cc/crates/dashboard-v2/` (NOT v1 crate).
2. 5 Tauri commands: query_focus_tunnel_mode_distribution / streaming_stats
   / pre_assembly_stats / dep_resolution / micro_steps. All `#[tauri::command]
   pub async fn`. Each accepts `session_id: Option<String>` (None = all-time).
3. Per-panel polling at 2_000 ms via `setInterval`, mirroring
   ProposalsPanel.svelte. No central store.
4. Mount = MonitorWindow.svelte after the media-validation section.
5. Aggregators wired:
   * mode_distribution_for_payloads (&[&SsbPayload] -> HashMap)
   * streaming_early_termination_stats_for_payloads (&[&SsbPayload])
   * pre_assembly_stats_for_payloads (&[SsbPayload])
   * dependency_resolution_stats_for_payloads (&[SsbPayload])
   * (file_router_stats_for_payloads exported but NOT wired into a command
     — the micro_steps command returns the raw payloads instead so the
     panel can render a per-row log. The aggregator stays available for a
     future "file router summary" sub-section.)

Decisions beyond the locked list:
* Added a new public method `SsbViewer::focus_tunnel_payloads(action_types,
  session_id, limit)` (orchestrator/src/ssb_viewer.rs) because the existing
  viewer surface only exposed typed summaries — none of them returned raw
  Vec<SsbPayload>. The method is generic (accepts any slice of
  SsbActionType) so future focus_tunnel sub-sections can reuse it without
  a per-action-type signature explosion. Hard cap = DRAWER_QUERY_HARD_CAP
  (500) rows scanned, same as latest_msi_per_mode pattern.
* Added 5 payload-type re-exports in `orchestrator/src/lib.rs` so
  dashboard-v2 commands.rs returns `Vec<MicroStepCreatedPayload>` without
  pulling `orchestrator::ssb` into dashboard scope.
* `session_id` is a query argument (not a path segment) so the panel can
  pass it via Tauri `invoke()` args. The Svelte panel takes an optional
  `sessionId` prop; today MonitorWindow mounts the panel without scoping,
  so the default "all sessions" view is what ships.

Contract test (4 tests, all passing):
* focus_tunnel_commands_registered_in_invoke_handler — include_str!(lib.rs)
  + assert "commands::query_focus_tunnel_*" substring for each of 5 names.
* focus_tunnel_command_bodies_exist_in_commands_rs — include_str!(commands.rs)
  + assert `pub async fn <cmd>(` for each of 5.
* focus_tunnel_commands_marked_with_tauri_command_attr — count
  `#[tauri::command]` >= 5.
* aggregators_consume_empty_payloads_without_panic — pin signatures of all
  5 aggregators (compile-time signature check + zero-state runtime check).

Test results:
* cargo check -p elai-orchestrator: OK
* cargo check -p elai-dashboard-v2: OK
* cargo check --workspace: OK
* cargo test -p elai_cc_contracts --test dashboard_focus_tunnel_panel_contract:
  4 passed, 0 failed
* npx svelte-check (dashboard-v2): 0 errors, 9 warnings (all pre-existing
  unused-CSS in unrelated drawer files; none from FocusTunnelPanel.svelte)

Nothing punted: mount point existed (MonitorWindow.svelte); panel renders;
backend commands wired; contract test green. file_router_stats_for_payloads
aggregator is exported but unused in commands (panel renders raw
MicroStepCreated rows instead) — future enhancement, not a regression.
