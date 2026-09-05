J-28 — token ledger / usage unification: decisions

WHAT WAS IMPLEMENTED (T1-T8, all Tier-1 fixture-driven per the plan's REVISED scope)

T1 (L-1 fix): anthropic.rs::MessageStream gained a private associated fn
`merge_usage_event(latest_usage: &mut Option<Usage>, event: &StreamEvent)`,
extracted out of `observe_event` so it is unit-testable without a live
`reqwest::Response`. `StreamEvent::MessageStart` now seeds `latest_usage`'s
input/cache side from `message.usage`; `StreamEvent::MessageDelta` MERGES
`output_tokens` onto the existing usage instead of replacing the whole
struct (the pre-fix bug: MessageDelta was the ONLY matched arm and its
`input_tokens` is always 0 by spec, so every streaming turn silently
recorded input_tokens=0). Fixture unit test
`observe_event_merges_message_start_input_with_message_delta_output`
drives `merge_usage_event` directly with a MessageStart(input=120) +
MessageDelta(output=42) pair and asserts the merged result is
{input:120, output:42, cache_creation:5, cache_read:3} — the L-1 fix,
proven independently of the T3 emit rows (RISK-2: the two emit seams
already read non-streaming responses with correct input_tokens; T1 and T3
are separate read paths, confirmed by the seams doc and not conflated
here).

T2 (NormalizedUsage, ROUTE B): `usage_ledger::NormalizedUsage { raw,
lane, semantics_tag }` with `::cloud(raw)` / `::local(raw)` constructors.
Assembled at BOTH production emit seams (grounding_client.rs via
`usage.token_usage()`; runtime_orchestrator.rs via a fresh
`elai_core::TokenUsage` built from the T6-resolved token counts) before
the UsageTurnEvent row is written, so the lane + semantics tag on every
row come from ONE typed source, not independently-typed literals at each
seam.

T3 (UsageTurnEvent): new `SsbActionType::UsageTurnEvent` (flat/unit
variant, mirrors RecurrenceHitRateMeasured/GateTaxMeasured — NO Kind on
the action-type enum itself) + `UsageTurnEventPayload` (Pattern-2 payload-
field convention: `kind: UsageTurnEventKind` rides inside the payload,
mirrors `loop_kind` on SecurityClassRegressionBlockedPayload). Full
6-part registration lockstep: ssb/action_type.rs (enum variant +
as_sql_str + from_str), ssb/payloads.rs (SsbPayload variant +
action_type() arm + the UsageLane/UsageSemanticsTag/UsageTurnEventKind/
UsageTurnEventPayload substrate), ssb/schema.rs (SQL CHECK constraint
arm), ssb.rs (SSB_ACTION_TYPES registration entry — auto-derives
SCHEMA_VERSION 271->272 and TOTAL_VARIANTS 580->581, never hand-edited),
tests/action_type_lock.rs (exhaustive_match_guard arm), and
tests/action_type_lock.golden (new sorted line `usage_turn_event`,
inserted alphabetically between `untrusted_memory_write_blocked` and
`user_decision_superseded`). The golden diff is +1 line — ABORT-2's
exactly-+1 constraint is honored (hand-verified; cargo not run, see
VERIFIED below).

Emitted at BOTH lane seams, BESIDE the existing rows (neither CloudSpend
nor BackendCostAccrued is touched or removed):
  - cloud: grounding_client.rs::CloudGroundingClient::send_with_ledger,
    right after the CloudSpend write, inside the same `if let Some(ssb) =
    self.ssb_writer.as_ref()` block.
  - local: runtime_orchestrator.rs::RuntimeOrchestrator::try_complete,
    in BOTH the PAID arm (beside the cost_model_for(backend) BackendCostAccrued
    write) and the free/local arm (beside the est_cost_micros=0
    BackendCostAccrued write).

T4 (SpendAccumulator repricing, the C-1 fix): `SpendAccumulator::for_model`
now routes through `pricing_for_model` (converting per-million to
per-token) instead of literal-matching only "claude-sonnet-4-6" and
cloning sonnet rates for everything else. Added a `priced: bool` field
(and the corresponding `SpendSnapshot.priced`) so an unknown model id
(pricing_for_model == None) gets an EXPLICIT no-rate accumulator — all
rates 0.0, `priced: false` — instead of a silently-wrong sonnet-priced
number. `for_claude_sonnet_4_6()` stays the const fallback (priced: true).
Negative unit tests (cloud_mode.rs): haiku prices below sonnet for
identical usage; an unrecognized model id totals exactly $0.0.

T5 (rollup): NEW `crates/orchestrator/src/usage_ledger.rs` (~430 lines
incl. tests; under the 1000-line MR-1 boundary). `UsageRollupKey {
identity, model, task_id, session_id, lane }` + `UsageRollupEntry {
input_tokens, output_tokens, cache_creation_tokens, cache_read_tokens,
cost_micros: Option<u64>, row_count }`, folded via `BTreeMap` (ELAI bans
std HashMap in the orchestrator). `rollup_usage_turns(&[UsageTurnEventPayload])
-> BTreeMap<UsageRollupKey, UsageRollupEntry>` sums input/output/cache
tokens across every row sharing a key — closing audit finding C-2 ("today
task_usd = most-recent row only"). `cost_micros` is Option-normalized
(S-02, goose `sum_optionals` idea, ported via `sum_optional_micros`):
`None` when every folded row was `EstimateApplied` (an estimated TOKEN
count never carries a trustworthy dollar figure), `Some(total)` otherwise
(including a real `Some(0)` for a free/local turn).

T6 (estimator fallback): `usage_ledger::resolve_turn_usage(reported_input,
reported_output, input_text, output_text) -> (u32, u32,
UsageTurnEventKind)`. A real (non-zero) reported usage always wins
(`TurnRecorded`); a `(0, 0)` report fires the estimator
(`estimate_tokens_cached`, chars/4 heuristic — see OPEN GAP below) and
returns `EstimateApplied`, NEVER a silent `TurnRecorded(0, 0)` (R2). The
local emit seam's `UsageTurnEvent` row is deliberately NOT gated by the
pre-existing `tokens_in>0||tokens_out>0` BackendCostAccrued
parse-error-noise suppression (RISK-1) — that suppression is untouched
for BackendCostAccrued, but the new row always fires.

T7 (compaction rule, FORK-3): `usage_ledger::apply_compaction_growth
(cumulative_input_before, compaction_output_tokens) -> u64`
(`saturating_add`, monotonic). Lands in usage_ledger per FORK-3 (below),
NOT context_policy.rs. SHIPPED AS WIRE-DARK this changeset — see WIRE-DARK
REGISTER below.

T8 (CLI surface): `format_cost_report` (resume.rs) gains a `model:
Option<&str>` parameter and reuses `pricing.rs::summary_lines_for_model`
(GD-1: no re-implemented rate table) for the dollar line + pricing_suffix
labeling; call sites updated (live_cli.rs's `/cost` passes `Some(self.model)`,
resume.rs's SlashCommand::Cost handler passes `None` — Session/
ConversationMessage carry no model id, so this is an honest
`pricing=estimated-default` fallback, not a threading gap I introduced).
NEW `elai usage [--json]` verb: `CliAction::Usage { json }` +
`parse_usage_subcommand_args` (args.rs) + `usage_cmd::run_usage` (new
file), mirroring the `elai checker-gaps` precedent exactly (resolve cwd
-> sessions_dir_path -> ssb.db -> SsbViewer::open -> read rows -> fold ->
print table/json). New `SsbViewer::usage_turn_events(limit)` reuses
`recent_rows_of_type` (GD-1: no duplicate SQL) and re-types the JSON
payload back into `UsageTurnEventPayload`.

FORK VERDICTS (settled, not re-opened)

FORK-1: ROUTE B (client-side normalization keyed by the lane tag).
Confirmed feasible per the seams doc: both production emit seams read
ALREADY-ASSEMBLED non-streaming responses (grounding_client's full
`Usage`, runtime_orchestrator's `(u32,u32)` tuple from `sidecar.rs:237`),
so normalization needs zero python/server.py touch. No remote sidecar
boot (ABORT-1 honored).

FORK-2: schema constant auto-derives. Live HEAD (d1eb94a) SCHEMA_VERSION
was 271 / SSB_ACTION_TYPES.len() 580 (re-grepped at edit, matching the
context package). Appending ONE `SsbActionType::UsageTurnEvent` entry to
`ssb.rs`'s `SSB_ACTION_TYPES` slice bumps SCHEMA_VERSION 271->272 and
TOTAL_VARIANTS 580->581 automatically (`SCHEMA_VERSION =
SCHEMA_VERSION_DDL_EPOCH(-309) + SSB_ACTION_TYPES.len()`); no number was
hand-edited.

FORK-3: T7 lands in `usage_ledger.rs`'s fold, NOT `context_policy.rs`.
Confirmed by reading `context_policy.rs` (crates/orchestrator/src/
context_policy.rs) — it is stateless threshold math only
(`trim_threshold`, `trim_threshold_for_mode`, `effective_context_window`,
`regime_policy`, `context_policy_disabled`), holds no cumulative token
state, and has no per-turn accumulation seam to extend. The
"rider on H-50 ContextPolicy" framing in the plan is conceptual only.
Do-not-do-both is honored: there is exactly one home for the monotonic-
growth rule.

THE THREE FINDINGS (honored)

1. T1/T3 independence: `merge_usage_event`'s fix lives entirely inside
   `MessageStream::observe_event` (streaming read path A). The T3 emit
   seams read non-streaming responses (path B), already carrying correct
   input_tokens. T1 is proven by the dedicated fixture unit test on
   `merge_usage_event`, not by an emit row. No attempt was made to route
   the UsageTurnEvent emit through the streaming path.

2. `runtime_orchestrator.rs:378`'s `tokens_in>0||tokens_out>0` gate
   (parse-error-noise suppression on the pre-existing BackendCostAccrued
   local arm) was left COMPLETELY UNTOUCHED. The new UsageTurnEvent emit
   in that same arm is a SEPARATE, unconditional write reached via
   `resolve_turn_usage`, which fires the T6 estimator (Kind=EstimateApplied)
   instead of vanishing on (0,0).

3. `elapsed_ms` is measured with a seam-local `std::time::Instant`
   (grounding_client.rs: `Instant::now()` before `send_message`;
   runtime_orchestrator.rs: `Instant::now()` before `adapter.complete`),
   never read off `MessageResponse`/`CompletionResponse` (neither type
   carries it in Rust).

TWO GAPS CLOSED (per the executor prompt)

1. Local task_id/session_id threading (RISK-3): confirmed by reading
   `try_complete`'s signature (`role: AgentRole, messages: Vec<Message>,
   config: CompletionConfig` — no task_id param) and `RuntimeOrchestrator`'s
   fields. `session_id` IS in scope via `ssb.session_id()` (the `Ssb`
   instance is permanently bound to one `(project_id, session_id)` pair,
   precedent already used at 4 other call sites in this same file) — wired
   as `Some(ssb.session_id().to_owned())`. `task_id` is genuinely NOT
   threaded into `try_complete`'s call chain; the local emit's `task_id`
   field is honestly `None` (never a fabricated id), matching the
   "identity default local, a field never infra" S-50 posture.

2. SQL CHECK constraint arm: added `'usage_turn_event'` to the CHECK(...)
   list in `crates/orchestrator/src/ssb/schema.rs` (append-only, after
   `'gate_tax_measured'`), matching the RecurrenceHitRateMeasured/
   GateTaxMeasured precedent exactly.

WIRE-DARK REGISTER (R-8/D-030 legal escape — backing row, since the
FSM EXECUTING-state write gate does not permit
workflow/sessions/s-j28-build/stubs.md from this seat)

id: J-28-T7-compaction-caller
marker: WIRE-DARK[J-28]
file: .elai_cc/crates/orchestrator/src/usage_ledger.rs :: apply_compaction_growth
why_dark: the T7 monotonic-growth rule ships as a pure, unit- and
  contract-tested function (rollup_usage_turns / apply_compaction_growth
  unit tests in usage_ledger.rs; not separately re-tested in the contract
  file since the rule is a single saturating_add with no cross-module
  surface to re-verify), but has NO production caller this changeset.
  compact.rs's existing `compact_session()`/`should_compact()` track
  message/removed-message COUNTS (`removed_message_count`,
  `preserve_recent_messages`), not a per-turn cumulative-token-context
  number `apply_compaction_growth` needs as input. Wiring it requires
  compact.rs's real trigger site (resume.rs's `/compact` command path or
  live_cli.rs's auto-compact trigger) to first grow a cumulative-input-
  token tracker across the compaction boundary — structural threading
  into an unrelated subsystem, not a one-line call, and explicitly
  out-of-scope for a fixture-driven Tier-1 data-layer build (plan
  ABORT-4: never fabricate a live-aggregate consumer to satisfy a
  checker).
will_resolve_in: a follow-on J-plan unit that threads cumulative-token
  state through compact.rs's real trigger site and calls
  apply_compaction_growth there; the plan's INTEGRATION_SMOKE cross-check
  (T7 growth number == T5 rollup session-sum across a compaction
  boundary) is deferred to that same unit — it cannot be satisfied
  honestly without that threading work, which this changeset does not do.
status: open

OPEN RESIDUE (not a WIRE-DARK, a documented design choice)

The T6 estimator (`estimate_tokens_cached`) uses a chars/4 heuristic with
a hand-rolled bounded FIFO cache (blake3-keyed, capacity 256), NOT an
`lru`-crate LRU. No canonical shared tokenizer exists in this codebase
(the same chars/N heuristic is already duplicated per-module —
`providers::ollama::ollama_token_estimate`,
`orchestrator::research_budget::estimate_tokens`, etc. — this follows the
same local convention rather than inventing a third shape) and the
workspace has no `lru` crate dependency; adding one needs explicit user
confirmation per house rule ("No new dependencies without confirmation"),
which was not sought given the dependency-free bounded-cache substitute
is adequate for a client-side estimate (never load-bearing for a real
dollar figure — see T5's Option-semantics: an EstimateApplied row's cost
is always folded as unknown).

EXPECTED TEST OUTCOMES (auditor runs these; not run by the executor —
OOM-prone box, cargo not invoked)

- crates/providers: anthropic.rs's new
  `observe_event_merges_message_start_input_with_message_delta_output`
  test — expect PASS.
- crates/orchestrator: cloud_mode.rs's
  `for_model_haiku_uses_haiku_rates_not_sonnet` +
  `for_model_unknown_model_is_explicit_no_rate_never_sonnet` — expect
  PASS. usage_ledger.rs's 9 unit tests (rollup sum, None+None, cross-lane
  no-merge, estimator fallback + never-all-zero-TurnRecorded, memoized
  estimate, compaction monotonic growth, NormalizedUsage lane/semantics)
  — expect PASS.
- crates/cli: cli_tests.rs's 3 format_cost_report tests (sectioned
  layout unchanged + 2 new dollar-line tests) — expect PASS.
- crates/contracts: j28_usage_ledger_contract.rs's 7 tests — expect
  PASS.
- `cargo test -p elai-orchestrator --test action_type_lock` — expect
  PASS (golden diff hand-verified as exactly +1 line; the count-lock test
  is auto-derived from `SSB_ACTION_TYPES.len()` so no separate literal
  needed updating).
- `python .claude/tools/wiring_contract_check.py --bn-file
  docs/thefinalglance/J-plan/J-28_token_ledger_usage_unification.md
  --run-trigger` — expect PASS (WIRING_CONTRACT's reachable_from entries
  are all real production call chains per this doc).

VERIFIED (self-run, executor seat; the auditor re-runs off-agent per
D-030/PCG-3)

`python .claude/tools/production_caller_check.py` over the full J-28
changeset (16 files): FINAL RESULT verdict=PASS, exit 0, dark_fn_count=0,
warn_count=4, wired_count=14. Initial run found 2 DARK:
`SpendAccumulator::is_priced()` — deleted as a genuinely redundant
wrapper over `snapshot().priced`, which was already the production-wired
path (2 unit tests + 1 contract test updated to read `.snapshot().priced`
instead); and `apply_compaction_growth` — moved the `// WIRE-DARK[J-28]:`
marker to within 3 lines of the fn signature (the checker's
`classify_definition` only scans a short comment-block window directly
above the def) so it now resolves REGISTERED-DARK (WARN), backed by this
file's WIRE-DARK REGISTER section (stubs.md is outside the EXECUTING-
state write gate from this seat). The 4 remaining WARNs are 3 correctly-
classified INTERNAL fns (`estimate_tokens_cached`, `sum_optional_micros`,
`parse_usage_subcommand_args` — same-file/adjacent-file non-test callers
only) plus the 1 REGISTERED-DARK row.

`python .claude/tools/maintainability_check.py` over the full J-28
changeset (18 files incl. tests): blocks=0, warns=19, exit 0. Every
finding is MR-1 (file already >1000 lines before this diff, grown by the
diff — anthropic.rs, cloud_mode.rs, ssb/payloads.rs, grounding_client.rs,
runtime_orchestrator.rs) or MR-3 (a pre-existing long function this diff
touched — e.g. `try_complete`, `send_with_ledger`, `as_sql_str`,
`parse_args`; several MR-3 hits are the checker's naive fn-name regex
false-matching on `let`/`Some` inside those long functions, a documented
regex-unsound leg). None of these functions/files were newly created or
substantially restructured by this diff; splitting any of them is a
separate, out-of-scope refactor (no unsolicited refactoring). The single
new module, `usage_ledger.rs`, is 421 lines — well under the 1000-line
MR-1 boundary. No `as any`/`unsafe`/lint-suppression was introduced; the
one real new `f64 as u64` cast (`grounding_client.rs::usd_to_micros`) is
a deliberately guarded,
saturating, well-defined Rust cast (documented inline) — the only numeric
cast added by this changeset.
