# E-29: Agentic observability (OTel GenAI semconv 1.40.0, transcript drawer, replay primitive, MAST taxonomy)
Objective: turn SSB into an OTel-exportable view (OTel GenAI semantic conventions 1.40.0, April 2026) without rewriting it; ship a Laminar-style transcript drawer in the dashboard; promote checkpoint rows to a LangGraph-style time-travel `ssb_resume_from(checkpoint_id, patch)` Tauri command; and auto-classify closed runs against MAST's 14-mode failure taxonomy.
Estimated effort: 3 sessions (~12-15 h)
Pre-requisites: B-2 PhaseTransition wiring (SHIPPED per CLAUDE.md context). B-19 (Hook System Unification, PLANNED) is NOT a blocker — OTel emission is orthogonal to hook unification, but coordinate column-add windows. C-44 / D-9 (frozen-surface enforcement, PLANNED) — ssb_resume_from must NOT bypass frozen-surface predicates when replaying patched prompts.
Parallel-safe with: E-30 (E-30 = non-LLM destructive-verb argv allowlist + PreToolUse trust dialog; touches `crates/orchestrator/src/hook_dispatcher.rs` argv parser; orthogonal to E-29 OTel column emission), E-24 (MCP protocols), E-25 (Claude Code architecture).
Source report: workflow/improvement-sweep-2026-05-19/reports/29-agentic-observability.md

## Context

Per report TL;DR: emit OTel GenAI semconv 1.40.0 attributes on every SSB row that mirrors an LLM/tool/agent op (`gen_ai.operation.name`, `gen_ai.agent.name`, `gen_ai.tool.name`, `gen_ai.usage.input_tokens|output_tokens|cache_*`, `gen_ai.response.finish_reasons`) so SSB becomes OTLP-exportable without a rewrite. SSB stays canonical; OTel becomes an export view (this is exactly what AgentFS does per turso.tech/agentfs: SQLite backing store, structured observability as a view).

Long agent runs (BSP supersteps, 74-action_type SSB) are unreadable as flat span trees per Laminar's 2026 ranking (laminar.sh/article/2026-04-23-top-6-agent-observability-platforms). Transcripts (what the agent said + tool calls + tool results, rendered as a chat) plus filters on action_type are the SOTA for "where did it go wrong" debugging.

Replay primitive: per tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-deterministic-ai-agents — temperature=0 alone is insufficient; full I/O capture (model_id, decode params, seed, system clock, tool I/O, safety/guardrail config) is the only reliable reproduction primitive. dev.to/sreeni5018 LangGraph checkpoint replay demonstrates the snapshot-per-superstep pattern. ELAI's BSP supersteps + checkpoint rows are the natural unit — needs a `ssb_resume_from(checkpoint_id, patch={prompt, model, tool_overrides})` Tauri command.

MAST failure taxonomy (Cemri et al. NeurIPS 2025, arXiv:2503.13657): 14 failure modes across 3 categories (Specification 41.8%, Coordination 36.9%, Verification 21.3%) from 1600+ traces. Worst modes: task misinterpretation, context collapse, incorrect verification, premature termination. Auto-classification on closed traces lets the dashboard surface root-cause categories.

OTel GenAI semconv 1.40.0 status (opentelemetry.io/blog/2026/genai-observability/, callsphere.ai/blog/vw3c-opentelemetry-genai-conventions-ai-agents-2026): client spans exited experimental in early 2026 (stable); agent/framework spans still experimental-but-stable. Datadog v1.37, Grafana/Loki, Honeycomb, Phoenix, Langfuse all ingest. Vendor lock-in is the wrong concern — OTel is the floor, vendors are the ceiling.

## Existing state (B/C/D coverage)

- B-2 (SHIPPED): PhaseTransition wiring connects FSM transitions to SSB. E-29 builds on top — `phase_transition` rows are exactly the workflow-span source for `invoke_workflow` OTel spans.
- B-19 (PLANNED): Hook System Unification specifies T1/T2/T3 hook latency budgets + `hook_latency_violation` emit. E-29 ADDS OTel `gen_ai.latency_ms` annotation but does NOT change hook gating; coordinate column additions.
- B-37 (SHIPPED): RuntimeAdapter trait + `runtime_fallback`/`runtime_health_changed`/`cli_subprocess_spawned`/`cli_subprocess_killed` SSB action types. E-29 annotates these with `gen_ai.provider.name`/`gen_ai.request.model` so they roll up into OTel agent spans.
- B-40 (SHIPPED per memory): Idea-Framer + Research Presets + Critic Loop, FsmSubstate::Framing, 5 SSB variants, SCHEMA_VERSION 41. E-29 must coexist with SCHEMA_VERSION bump.
- B-41 (SHIPPED per memory): Per-Level Auditor Modes, AgentRole +5, Evidence +5, AuditTimelineWindow, SCHEMA_VERSION 45. E-29 must bump SCHEMA_VERSION again (likely to 46+).
- B-42 (SHIPPED per memory): Cosmos Artifact Tier, CosmosCheck VerificationMethod, SCHEMA_VERSION 44.
- C-44 / D-9 (PLANNED): frozen-surface access control. E-29 `ssb_resume_from` Tauri command MUST honor frozen-surface predicates when patching prompts/tools (the replay can't bypass the same guards production does).
- ELAI today: `crates/orchestrator/src/ssb.rs:5037-5053` exposes `write` and `write_payload` on the writer. New OTel columns extend the row schema, not the writer API. SsbViewer at `crates/orchestrator/src/ssb_viewer.rs` already exposes SQL-over-traces — surface as a "Query" drawer per Laminar's flagship feature.
- `crates/orchestrator/src/regression.rs:157` carries `failure_taxonomy_misses: vec![]`; `crates/orchestrator/src/self_mod_types.rs:564-614` carries 13 categories (per CLAUDE.md "13 → spec 9" delta in B-20 PLANNED). E-29 ADDS MAST's 14 modes as an ORTHOGONAL classifier (not replacing the 13 — MAST is failure-cause taxonomy; the existing 13 is failure-symptom taxonomy).
- Hook dispatcher at `crates/orchestrator/src/hook_dispatcher.rs:626-1036` carries the PreToolUse/PostToolUse/PreCompact wiring. E-29 does NOT touch hook gating; only adds OTel attribute emission.

## Reads (mandatory before coding)

- `F:\.ELAI_workflow\workflow\improvement-sweep-2026-05-19\reports\29-agentic-observability.md` (source report, full)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb.rs` schema section (find via `rg "CREATE TABLE.*ssb_events"`); writer at lines 5030-5070
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\ssb_viewer.rs:1180-2800` (SQL-over-traces surface)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\regression.rs:150-170` (failure_taxonomy_misses field)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\self_mod_types.rs:560-620` (existing failure category enum)
- `F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\hook_dispatcher.rs:620-1040` (hook spans source for `execute_tool` OTel emit)
- `F:\.ELAI_workflow\.elai_cc\crates\dashboard-v2\src\lib\windows\SsbViewer.svelte` (existing — Laminar-style transcript drawer goes alongside)
- opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/ (online, do not load — 1.40.0 attribute schema)
- opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans.md (online — client span attrs)
- arXiv:2503.13657 Cemri et al. MAST taxonomy (cited)
- github.com/traceloop/openllmetry — reference for `TRACELOOP_TRACE_CONTENT` content-gating pattern

## Tasks

1. **Add OTel GenAI column aliases to SSB schema (~180 LOC).** New nullable columns on `ssb_events`: `otel_op_name` TEXT, `otel_provider` TEXT, `otel_agent_id` TEXT, `otel_agent_name` TEXT, `otel_workflow_name` TEXT, `otel_tool_name` TEXT, `otel_request_model` TEXT, `otel_response_finish_reason` TEXT, `otel_usage_input_tokens` INTEGER, `otel_usage_output_tokens` INTEGER, `otel_cache_creation_input_tokens` INTEGER, `otel_cache_read_input_tokens` INTEGER, `otel_conversation_id` TEXT, `otel_latency_ms` INTEGER, `otel_error_type` TEXT. SCHEMA_VERSION bump (likely 46+; coordinate with B-41's 45 bump per memory). Migration is additive only — never drop existing columns. Update `SsbActionType::as_sql_str` for the mapping helper that returns the OTel op_name string per action_type (e.g. `phase_transition` → `invoke_workflow`, `runtime_fallback` → `chat`, `hook_pre/post` → `execute_tool`).

2. **Create `crates/ssb-otel-export` crate (~350 LOC).** New crate that reads SSB rows and emits OTLP gRPC spans. Uses `opentelemetry` + `opentelemetry-otlp` Rust crates (verify pypi/cargo at pin time per anti-slopsquatting). Maps each SSB row to a span with `parent_id` resolved from existing `checkpoint_id` / `phase_id` columns; span hierarchy follows report Section "OpenTelemetry GenAI semantic conventions": `invoke_workflow {phase}` → `invoke_agent {agent_role}` → `chat {model}` + `execute_tool {tool}` (siblings under agent). CLI flag `--otlp-endpoint http://...` plumbs through `crates/cli/src/args.rs`. Content gating via env `ELAI_OTEL_TRACE_CONTENT=false` default; matches OpenLLMetry's `TRACELOOP_TRACE_CONTENT` pattern.

3. **Add TranscriptDrawer dashboard component (~220 LOC).** New file `crates/dashboard-v2/src/lib/windows/TranscriptDrawer.svelte`. Tauri command `ssb_transcript(session_id, filter={role,action_type,window})` that joins prompt/tool_use/tool_result rows in temporal order and renders as chat-shaped UI (role-bubbles, tool-call drawers, monospace for code-output). Filters: action_type whitelist, time window, agent role, free-text search. Backend command lives in `crates/dashboard-v2/src-tauri/src/lib.rs`.

4. **Add `ssb_resume_from(checkpoint_id, patch)` Tauri command (~280 LOC).** New action_type `replay_resume` with payload `{checkpoint_id, patch:{prompt?:string, model?:string, tool_overrides?:Vec<ToolOverride>}, replay_session_id}`. Per report Section "Replay / debugging patterns": (a) Load checkpoint state, (b) Apply patch (validated against C-44/D-9 frozen-surface predicates — DO NOT allow patches that would bypass these), (c) Re-run from checkpoint, (d) Record outputs to a NEW replay-session under the same project root, (e) Surface as a new RESEARCH-mode orbit node "replay branches" showing parent run + replay deltas as siblings. Out-of-scope for E-29: clock+RNG interception (report Section "Replay 4"); document as Open Question 3 measurement target.

5. **Add MAST failure-mode auto-classifier (~250 LOC).** New module `crates/orchestrator/src/mast_classifier.rs` exposing `classify_closed_run(session_id) -> Vec<MastModeHit>` where `MastMode` is an enum with 14 variants per arXiv:2503.13657 Table 2 (Specification: TaskMisinterpretation, GoalConflict, MissingConstraint, ScopeCreep, AmbiguousInstruction; Coordination: ContextCollapse, RoleConfusion, HandoffFailure, ResourceContention, DependencyMismatch, ProtocolViolation; Verification: IncorrectVerification, PrematureTermination, MissingValidation). Classifier runs as a post-run agent — coordinate with planning-workflow SKILL.md. New SSB action_type `failure_classified` with payload `{mode:MastMode, evidence_row_ids:Vec<i64>, confidence:f32}`. New dashboard drawer "Failure modes" with sparkline per category per phase.

6. **Add Signals (NL outcome extraction) action_type (~150 LOC).** Per Laminar's flagship feature: user writes free-text query "agent asked user for clarification and got useful answer" → LLM classifier writes `signal_match` rows backfilled across SSB. New action_type `signal_match` with payload `{signal_label, matched_row_ids, classifier_model, confidence}`. Backfill is opt-in via Tauri command `ssb_backfill_signals(signal_label, query)`.

7. **Add per-stage cost + cache-attribution rollup (~120 LOC).** Per OTel `gen_ai.usage.cache_creation.input_tokens` / `cache_read.input_tokens` — exactly the Anthropic prompt-caching attribution columns. Compute `cost_usd` per row using `crates/providers/`-side per-model pricing tables (need NEW pricing table file `crates/providers/src/pricing.rs` if absent; verify by grep). New drawer "Spend" with sparkline per phase/agent/tool. Read-only; does not block any path.

8. **Eval-dataset promote command (~140 LOC).** Per Langfuse/LangSmith pattern: production traces become eval datasets. New SSB action_type `eval_dataset_member` with payload `{trace_id, dataset_name, label, scoring_rubric}`. New Tauri command `ssb_promote_to_dataset(trace_ids, dataset_name)`. Optional CI-gate via env `ELAI_EVAL_GATE_REGRESSION_THRESHOLD=0.05` — runs in COMMITTING substate before push (per report Section "Best techniques 7").

9. **SQL-over-traces "Query" drawer (~120 LOC).** Read-only SQL surface against `ssb_events` (existing SsbViewer already supports this) with prepared queries (10 starter queries: "all phase_transitions in last hour", "failed audits last 24h", "top-10 most-expensive turns") + free-form SQL bounded to read-only role (no INSERT/UPDATE/DELETE allowed via SQL — enforced by sqlite role connection).

10. **Decision doc at `workflow/reasoning/E-29-decisions.md`** covering: (a) SSB canonical, OTel as view (do NOT migrate SSB to spans), (b) content gating default OFF per OpenLLMetry pattern, (c) MAST orthogonal to existing 13-category failure taxonomy (one is symptom, one is cause), (d) frozen-surface predicates honored in ssb_resume_from (no bypass via replay), (e) explicit OUT-OF-SCOPE: clock+RNG interception (Open Question 3 measurement target), vendor APM integration (use OTLP endpoint instead — Datadog/Grafana/Honeycomb all ingest natively).

## WIRING_CONTRACT

produces_symbols:
  - crates/ssb-otel-export (NEW crate)::SsbOtelExporter
  - crates/orchestrator/src/mast_classifier.rs::{MastMode (enum, 14 variants), MastModeHit, classify_closed_run}
  - crates/orchestrator/src/ssb.rs (15 new nullable columns + SCHEMA_VERSION bump)
  - crates/dashboard-v2/src/lib/windows/TranscriptDrawer.svelte
  - crates/dashboard-v2/src/lib/windows/FailureModesDrawer.svelte
  - crates/dashboard-v2/src/lib/windows/SpendDrawer.svelte
  - crates/dashboard-v2/src/lib/windows/QueryDrawer.svelte
  - crates/dashboard-v2/src-tauri/src/lib.rs::{ssb_transcript, ssb_resume_from, ssb_promote_to_dataset, ssb_backfill_signals}
  - SsbActionType::{ReplayResume, FailureClassified, SignalMatch, EvalDatasetMember}
consumes_symbols:
  - crates/orchestrator/src/ssb.rs::SsbWriter::write_payload (existing — emits new columns as nullable attrs)
  - crates/orchestrator/src/ssb_viewer.rs (existing SQL surface)
  - crates/orchestrator/src/hook_dispatcher.rs (existing — annotated with OTel execute_tool attrs)
  - crates/orchestrator/src/runtime_orchestrator.rs (existing — annotated with OTel chat attrs)
  - C-44/D-9 frozen-surface predicates (consumed by ssb_resume_from validation path)
reachable_from:
  - User opens dashboard TranscriptDrawer for session X → ssb_transcript Tauri → SQL join → rendered chat-shaped UI
  - User runs `elai --otlp-endpoint http://otel-collector:4317 task ...` → SsbOtelExporter spawned → OTLP gRPC spans flow
  - User clicks "Resume from checkpoint" in dashboard → ssb_resume_from(checkpoint_id, patch) → C-44/D-9 validates → new replay-session writes replay_resume row
  - Post-run agent calls classify_closed_run(session_id) → 14-mode MAST classification → failure_classified rows + Failure-modes drawer surface
stub_ban:
  - "TODO: OTel later" literal absent
  - "// MAST classifier deferred" literal absent (E-29 includes a working classifier)
  - No bypass of C-44/D-9 frozen-surface predicates from ssb_resume_from (this is the load-bearing safety property)
  - No content emission when ELAI_OTEL_TRACE_CONTENT is unset or "false"
frontend_pair: crates/dashboard-v2/src/lib/windows/TranscriptDrawer.svelte + FailureModesDrawer.svelte + SpendDrawer.svelte + QueryDrawer.svelte
observable_effect:
  trigger: "ELAI_OTEL_TRACE_CONTENT=false ELAI_OTLP_ENDPOINT=http://127.0.0.1:4317 elai task 'small fixture', then open TranscriptDrawer, then click Resume-from-checkpoint"
  expect_ssb: "phase_transition rows with otel_op_name='invoke_workflow' populated; replay_resume row emitted on resume; failure_classified rows emitted post-run by mast_classifier"
  expect_artifact: "OTLP collector receives spans with gen_ai.* attribute schema; TranscriptDrawer renders the chat with role-bubbles; ssb_resume_from refuses any patch that would touch a frozen surface"

## DETERMINISTIC_CHECK

```bash
# 1. SSB schema migration applied (15 new columns + version bump)
sqlite3 .elai_cc/.elai_cc/ssb.db "PRAGMA table_info(ssb_events);" | grep -E "otel_op_name|otel_provider|otel_usage_input_tokens|otel_latency_ms"
sqlite3 .elai_cc/.elai_cc/ssb.db "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1;"

# 2. New crate exists and builds
test -d .elai_cc/crates/ssb-otel-export
cd .elai_cc && cargo build -p ssb-otel-export

# 3. MAST classifier module exists with 14 variants
rg "enum MastMode" .elai_cc/crates/orchestrator/src/mast_classifier.rs
rg "TaskMisinterpretation|ContextCollapse|PrematureTermination|MissingValidation" .elai_cc/crates/orchestrator/src/mast_classifier.rs

# 4. Tauri commands registered
rg "ssb_transcript|ssb_resume_from|ssb_promote_to_dataset|ssb_backfill_signals" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs

# 5. New SSB action types
rg "replay_resume|failure_classified|signal_match|eval_dataset_member" .elai_cc/crates/orchestrator/src/ssb.rs

# 6. Dashboard drawers exist
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/TranscriptDrawer.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/FailureModesDrawer.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/SpendDrawer.svelte
test -f .elai_cc/crates/dashboard-v2/src/lib/windows/QueryDrawer.svelte

# 7. Frozen-surface predicate respected in resume
rg "frozen_surface|locked_keys_auditor" .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs

# 8. Decision doc exists
test -f workflow/reasoning/E-29-decisions.md
```

## ACCEPTANCE

- [ ] All 15 new OTel columns added to `ssb_events`; SCHEMA_VERSION bumped; migration is additive (no existing column dropped)
- [ ] `ssb-otel-export` crate emits OTLP gRPC spans for a known fixture trace; spans validate against opentelemetry-collector schema
- [ ] Content gating: when `ELAI_OTEL_TRACE_CONTENT` unset, `gen_ai.input.messages` and `gen_ai.output.messages` attrs are NOT emitted
- [ ] TranscriptDrawer renders a session's chat in temporal order with role-bubbles + tool-call expandable sections
- [ ] `ssb_resume_from(checkpoint_id, patch)` refuses patches that touch a frozen surface (C-44/D-9 predicate check); refused patches emit `frozen_surface_patch_refused` row (or equivalent existing rejection action_type)
- [ ] MAST classifier produces non-empty `failure_classified` rows on at least one known-failed historical session; 14 variants are exhaustive (per arXiv:2503.13657 Table 2)
- [ ] FailureModesDrawer surfaces % per MAST category over time with sparkline
- [ ] SpendDrawer rolls up cost_usd per phase/agent/tool with cache_read/cache_creation attribution
- [ ] QueryDrawer SQL surface is READ-ONLY (sqlite role enforced); 10 prepared queries return non-empty for at least one session
- [ ] `cargo check --workspace` GREEN; `cargo test -p elai-orchestrator -p ssb-otel-export -p dashboard-v2-tauri` PASS
- [ ] Cross-language pair: TranscriptDrawer.svelte + src-tauri/src/lib.rs commands stay in sync (cross_lang_diff.py exits 0)

## Tests to add

- `crates/ssb-otel-export/tests/otlp_export_contract.rs`: fixture SSB → OTLP gRPC bytes → validate against opentelemetry-collector schema.
- `crates/contracts/tests/ssb_otel_columns_contract.rs`: assert 15 new columns exist + are nullable.
- `crates/contracts/tests/replay_resume_frozen_surface_contract.rs`: assert ssb_resume_from refuses patches touching frozen surfaces (consumes C-44/D-9 predicates).
- `crates/orchestrator/tests/mast_classifier_14_variants.rs`: exhaustive match against MAST 14 variants; assert classification on 3 fixtures (one Specification, one Coordination, one Verification).
- `crates/contracts/tests/transcript_drawer_chat_order_contract.rs`: temporal-order assertion on ssb_transcript output.
- `crates/contracts/tests/otel_content_gating_contract.rs`: assert `gen_ai.input.messages` NOT emitted when ELAI_OTEL_TRACE_CONTENT unset.

## Files touched

- CREATE: `crates/ssb-otel-export/` (NEW crate, ~350 LOC across lib.rs + tests)
- CREATE: `crates/orchestrator/src/mast_classifier.rs` (~250 LOC)
- CREATE: `crates/dashboard-v2/src/lib/windows/TranscriptDrawer.svelte` (~220 LOC)
- CREATE: `crates/dashboard-v2/src/lib/windows/FailureModesDrawer.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/windows/SpendDrawer.svelte` (~140 LOC)
- CREATE: `crates/dashboard-v2/src/lib/windows/QueryDrawer.svelte` (~120 LOC)
- CREATE: `workflow/reasoning/E-29-decisions.md` (~100 LOC)
- MODIFY: `crates/orchestrator/src/ssb.rs` (~180 LOC: 15 new columns + 4 new action_types + SCHEMA_VERSION bump + as_sql_str helper)
- MODIFY: `crates/orchestrator/src/ssb_viewer.rs` (~80 LOC: SQL prepared-query surface)
- MODIFY: `crates/orchestrator/src/regression.rs:157` (~20 LOC: MAST-orthogonal carve-out doc + classifier hookup)
- MODIFY: `crates/orchestrator/src/hook_dispatcher.rs:620-1040` (~60 LOC: emit otel_op_name='execute_tool' on existing hook rows)
- MODIFY: `crates/orchestrator/src/runtime_orchestrator.rs` (~50 LOC: emit otel_op_name='chat' on runtime rows)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (~280 LOC: 4 new Tauri commands)
- MODIFY: `crates/cli/src/args.rs` (~20 LOC: `--otlp-endpoint` flag)
- MODIFY: `crates/providers/src/pricing.rs` (CREATE if absent, ~80 LOC: per-model USD/1k pricing table)
- MODIFY: `Cargo.toml` workspace members (~3 LOC: add `crates/ssb-otel-export`)

## Citations / Grounding

- opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans/ — GenAI agent span semconv 1.40.0 April 2026 (cited; PRIMARY)
- opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-spans/ — client spans (cited)
- opentelemetry.io/blog/2026/genai-observability/ — client spans stable early 2026 (cited)
- callsphere.ai/blog/vw3c-opentelemetry-genai-conventions-ai-agents-2026 — semconv 1.40.0 April 2026 (cited)
- arXiv:2503.13657 — Cemri et al. MAST taxonomy NeurIPS 2025 (cited; PRIMARY for Task 5)
- arXiv:2604.08906v1 — Empirical study of bugs across 5 agentic frameworks 2026 (cited)
- arXiv:2602.23193 — ESAA event sourcing for autonomous agents (cited)
- laminar.sh/article/2026-04-23-top-6-agent-observability-platforms — Laminar transcript-first + Signals + SQL-over-traces (cited; PRIMARY for Tasks 3 + 6 + 9)
- dev.to/sreeni5018/debugging-non-deterministic-llm-agents-implementing-checkpoint-based-state-replay-with-langgraph-5171 — LangGraph checkpoint replay (cited; PRIMARY for Task 4)
- tianpan.co/blog/2026-04-12-deterministic-replay-debugging-non-deterministic-ai-agents — temp=0+seed insufficient, full I/O required (cited)
- traceloop.com/docs/openllmetry/introduction — OpenLLMetry + TRACELOOP_TRACE_CONTENT pattern (cited; PRIMARY for content gating)
- github.com/langfuse/langfuse — datasets + LLM-as-judge + free self-host (cited)
- arize.com/docs/phoenix — Phoenix OpenInference (cited)
- microsoft.com/en-us/security/blog/2026/03/18/observability-ai-systems-strengthening-visibility-proactive-risk-detection/ — unified tracing argument (cited)

## Open questions / risks

- Report Open Question 1: actual span volume from a HEAVY-tier session is unknown — if >10K SSB rows, OTLP export must batch + sample (semconv allows head/tail sampling). Mitigation: ssb-otel-export defaults to batch + tail-sampling; expose env knobs.
- Report Open Question 2: MAST 14-mode taxonomy may not generalize to ELAI's FSM-driven harness (Stuck-in-substate, Anti-spiral-hit may be novel modes). Mitigation: run MAST classifier on last 10 closed sessions before promoting drawer to default-visible.
- Report Open Question 3: replay fidelity ceiling on remote sidecar — tool outputs are recorded but the sidecar's KV-cache state isn't. Cache-warmup before replay may change outputs. Measurement TODO: structural-equivalence rate on 20 replayed traces. E-29 records this as a known-imperfection; ssb_resume_from is "best-effort replay" not "deterministic replay" until KV-state capture lands.
- Report Open Question 4: token cost vs. observability cost — full `gen_ai.input.messages` capture overhead on 100K-token session is unknown. Default OFF (ELAI_OTEL_TRACE_CONTENT) bounds this risk.
- Report Open Question 5: useful regression threshold for continuous-eval gate before COMMITTING is unknown. E-29 ships the eval-dataset surface but leaves the gate threshold as a manual env knob (ELAI_EVAL_GATE_REGRESSION_THRESHOLD).
- Anti-Stale-Training: `opentelemetry-otlp` Rust crate version verify mandatory before pin (anti-slopsquatting).
- SCHEMA_VERSION coordination: B-41 shipped at 45, B-40 at 41, B-42 at 44 per memory. E-29 likely bumps to 46. Coordinate with any in-flight schema work.
- C-44/D-9 frozen-surface predicates are PLANNED. If they ship after E-29, ssb_resume_from validation must be a stub that fails closed (refuse all patches that look risky) until the predicates land — DO NOT ship a permissive default.

## Next

E-30 (public agent incidents) is the natural follow-on: incidents include "rollback impossible" deception, "test passed" fakery (DGM), and prompt-injection cascade — every one of these benefits from the OTel + transcript + replay primitives in E-29. The MAST classifier's "IncorrectVerification" mode flags exactly the DGM failure pattern. Future E-N: KV-cache state capture for true deterministic replay (Open Question 3); clock+RNG interception (Open Question 4 from report); MAST-mode-specific dashboard alerts.
