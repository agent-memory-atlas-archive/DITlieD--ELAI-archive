E-29 — Agentic observability — decisions
session: s-orch-e29 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-29_agentic_observability.md

(a) SCOPE DETERMINATION — TIER-A OWNED-ONLY SLICE

E-29 plan has 8 drift findings flagged by both context-compiler + integration-explorer-elai (s-orch-e29 batch):

  DRIFT-1 (HIGH): plan DETERMINISTIC_CHECK queries `ssb_events` — real table is `ssb` (ssb.rs:983). Any SQL test using the plan's name silently passes via grep without ever hitting the real table.

  DRIFT-2 (HIGH): plan Task 5 names new action_type `failure_classified` — that name already exists at ssb.rs:1007 + payload at ssb.rs:5110-5117 for the 13-category symptom taxonomy. MAST classifier would collide. Recommend rename to `mast_failure_classified` or `failure_cause_classified` in Tier-B.

  DRIFT-3 (HIGH): plan Task 7 says "CREATE pricing.rs if absent". pricing.rs already exists at providers/src/pricing.rs (C-14 shipped) with ModelPricing carrying input/output/cache_creation/cache_read per-million columns. Tier-A SpendDrawer reuses this directly.

  DRIFT-4 (HIGH): plan claims SCHEMA_VERSION 45 baseline. Real value is 97. Plan migrator premise off by 50+ bumps.

  DRIFT-5 (HIGH): "Reads" line cites crates/dashboard-v2/src/lib/windows/SsbViewer.svelte — no such file exists. SsbViewer is a Rust struct only (orchestrator/src/ssb_viewer.rs).

  DRIFT-6 (HIGH): 4 OTel Rust crates ABSENT from workspace (opentelemetry / opentelemetry-otlp / opentelemetry-semantic-conventions / tracing-opentelemetry). Anti-slopsquatting verify required before adding.

  DRIFT-7 (MEDIUM): plan ssb.rs line citations are stale (cites 5037-5053 for write; reality is 6898-6914). Implementer should not anchor on those line numbers.

  DRIFT-8 (RISK): OTel GenAI semconv 1.40.0 status — verified via WebSearch by integration-explorer (May 2026) the spec is still labeled "Development" not "Stable" on opentelemetry.io. Plan's "client spans exited experimental in early 2026 (stable)" overstates current status.

Given the campaign-wide pattern (E-44, E-16, E-31 all returned the same verdict before E-29), Tier-A is the path that ships value without resolving every drift first.

(b) WHAT TIER-A E-29 SHIPPED THIS RUN

Three new dashboard drawers, all over EXISTING Tauri commands and producer SSB rows. Zero new backend code, zero new dependencies, zero new SsbActionType variants.

  - crates/dashboard-v2/src/lib/drawers/SpendDrawer.svelte (NEW, ~150 LOC)
    Surfaces existing `get_cloud_spend` Tauri command (commands.rs:4335) which rolls up CloudSpend SSB rows via SsbViewer::cloud_spend_summary. Renders task_usd / day_usd / cache_creation_tokens / cache_read_tokens / redaction_count in a Wong-toned grid with 10s polling. Pricing model from providers/src/pricing.rs.

  - crates/dashboard-v2/src/lib/drawers/QueryDrawer.svelte (NEW, ~200 LOC)
    Surfaces existing `get_query_trace_recent` Tauri command (commands.rs:924) which reads C-1 QueryTrace SSB rows via SsbViewer::recent_rows_of_type. Renders the last 20 traces with expandable per-stage breakdown. Mode-tinted (FAST/STANDARD/DEEP via Wong tones). 5s polling. Independent surface from KnowledgeDrawer's same-data render — focused inspector view.

  - crates/dashboard-v2/src/lib/drawers/TranscriptDrawer.svelte (NEW, ~190 LOC)
    Surfaces existing `get_chat_history` Tauri command (commands.rs:4479) which reads ChatMessage + ChatResponse SSB rows via SsbViewer::chat_history. Renders user/agent turns sorted by seq with Wong-toned left-borders (signal-blue user / signal-green agent). 5s polling. Reflect* row joining deferred to Tier-B (would need a new joined Tauri command).

  - workflow/reasoning/E-29-decisions.md (NEW, this file)

(c) WHAT TIER-B DEFERRED + WHY

Per the 8 DRIFT findings and the OWNED-vs-AVOIDED partition:

  Task 1 — OTel semconv SSB columns (gen_ai.system / gen_ai.operation.name / gen_ai.input.tokens / etc) + tracing-opentelemetry attach. Defer reasons: (a) ZERO existing OTel crates in workspace — adding 4 new deps requires anti-slopsquatting WebSearch + version pin + workspace approval; (b) SCHEMA bump 97 -> 98+ atomic with new columns is a 7+ touchpoint surgery in ssb.rs; (c) GenAI semconv 1.40.0 is still Development not Stable — attribute schema may move.

  Task 2 — ssb-otel-export crate. Defer reasons: requires Tier-B Task 1 columns + the new dependencies. Also: tracing init currently exists ONLY in dashboard-v2/src-tauri/lib.rs:181; CLI binary has no tracing_subscriber init. An OTel export layer attached only to the Tauri subscriber would miss every CLI span. CLI binary init is its own sub-task.

  Task 4 — ssb_resume_from + replay primitive. Defer reasons: NEW code (no replay/snapshot/SnapshotReplay symbol exists anywhere). LockedKeysAuditor frozen-surface gating still PLANNED. Belongs in the orchestrator/{root} grey-zone but big enough to deserve its own E-N or to ride with E-44 SAGA work.

  Task 5 — MAST classifier. Defer reasons: requires action_type RENAME because `failure_classified` is already taken (DRIFT-2). Also: MAST 14-mode cause taxonomy vs the existing 13-category symptom taxonomy at failure_taxonomy.py is orthogonal — design decision needed on whether to merge or coexist before adding the variant. Carries forward to plan-improvement input.

  Task 6 — signal_match LLM classifier. Defer reasons: model choice unspecified, sidecar vs cloud routing decision needed, blocked on the LocalClassifier substrate that ELAI's reasoning crate owns (AVOIDED this run).

  Task 8 — eval-dataset member SSB row. Defer reasons: depends on Task 5 + Task 6 producers landing first; the row is the harness output that connects evals to telemetry, but evals need the classifiers above.

  Task 3 follow-ups — FailureModesDrawer rendering Task 5's MAST output. Defer reasons: rides with Task 5.

  Task 7 follow-ups — SpendDrawer extensions for redaction-event detail. Defer reasons: needs the new redaction_event SSB action type (C-4-decisions.md Q5 has it as PLANNED; not currently producing rows).

  Task 9 follow-ups — QueryDrawer extensions surfacing rrf_quality_gate_fired / hyde_skipped_code_query / rrf_component_weights / deep_pool_capped (the 4 E-17 SP4 RAG surfaces). Defer: those Tauri commands exist at commands.rs:1018/1051/1085/1123; rendering them in QueryDrawer is a follow-on patch not a Tier-B requirement.

  Tier-A OTel-LITE substitute: the three drawers ship the user-visible value of OTel observability (transcript / spend / queries) WITHOUT the OTel dep tree. A future Tier-B run can wrap the existing SSB rows with OTel spans without changing the dashboard.

(d) WHY THIS ORDERING

The three drawers are pure read-side dashboard surfaces over PRODUCERS THAT EXIST. They cost no new Rust code, no new deps, no SCHEMA bumps, no action_type collisions. They surface real value (cost transparency, query-trace inspection, transcript history) from data the orchestrator already writes. The OTel attribute schema can rebase on this surface later — adding tracing-opentelemetry as a layer is additive to the SSB rows, not a replacement.

The campaign pattern of plan-drift forced a Tier-A/Tier-B split discipline this run. The Tier-A discipline: "ship over what exists, defer the new producers."

(e) ZERO NEW SSB VARIANTS — DELIBERATE

E-29 plan would add 4 new SsbActionType variants (ReplayResume / FailureClassified-collision / SignalMatch / EvalDatasetMember). This run adds NONE. Reasons: (a) failure_classified is already taken; (b) the other 3 producers are Tier-B blocked; (c) 7+ atomic touchpoints in ssb.rs × 4 variants is the same expensive surgery deferred in E-33.B and E-44.

When Tier-B lands, the SCHEMA bump 97 -> 98+ rides atomically with the producers per the ssb.rs B-23 4I.1 precedent ("co-staging variants without emitters creates dead schema rows the writer never produces — declined").

(f) ANTI-SLOPSQUATTING SEED FOR TIER-B

Before adding the OTel deps in Tier-B, run cargo search + WebSearch verification on each. Integration-explorer already established opentelemetry-otlp 0.32.0 latest per docs.rs (May 2026), MSRV 1.75.0. Document the chosen crate set + pinned versions in workflow/reasoning/E-29-tier-b-improvement.md when running the plan-improvement protocol pre-EXECUTING.

(g) NEW WONG-PALETTE CONSUMERS

The 3 drawers consistently use the E-33-introduced --signal-blue (cache_read / FAST mode / user-turn border / latency badge), --signal-green (verified / DEEP mode / agent-turn border), --signal-orange (low / cache_creation warning), --signal-vermilion (error). This continues the E-33 design language: brand teal/purple stays for chrome; signal layers use Wong. The drawers also honor the @media (prefers-reduced-motion: reduce) block in tokens.css implicitly (only the QueryDrawer hover transitions are animated; everything else is static).

---

E-29 TIER-B (cluster s-25120, 2026-05-28) — agentic-observability substrate landed

(1) SCOPE OF TIER-B THIS COMMIT

Tier-B lands the observability/ subtree + the SSB substrate the producers ride on:
  * orchestrator::observability/{mod, mast_classify, replay, signal_match, dataset_promote, pricing_rollup} — NEW subtree.
  * SsbActionType::AgenticObservabilityEvent — NEW single-variant Pattern-2 (25th application), SCHEMA 172 -> 173, TOTAL_VARIANTS 445 -> 446.
  * AgenticObservabilityEventKind 4-arm discriminator (MastFailureClassified / ReplayResume / SignalMatch / EvalDatasetMember).
  * AgenticObservabilityEventPayload + 2 convenience constructors.
  * ELAI_AGENTIC_OBSERVABILITY_ENABLE env gate (wire-dark default-off).
  * ELAI_OTEL_TRACE_CONTENT env gate (content-gating default-off, OpenLLMetry pattern).
  * ELAI_OTLP_ENDPOINT env var (read by cli/args.rs --otlp-endpoint flag).
  * crates/ssb-otel-export — NEW crate, feature-gated behind `otlp-export`. 4 OTel deps OPTIONAL.
  * dashboard-v2/src-tauri/src/commands_e29.rs + 4 new Tauri commands.
  * dashboard-v2/src/lib/drawers/FailureModesDrawer.svelte — NEW (sibling of MastFailurePanel).
  * cli/src/args.rs --otlp-endpoint flag.

Tier-A (TranscriptDrawer / SpendDrawer / QueryDrawer) was shipped earlier this cluster; Tier-B does NOT touch those.

(2) DECISION-1: Pattern-2 single-variant atomic bump (25th application)

4 new SSB fire-points batch as ONE SsbActionType variant + a 4-arm kind discriminator. Precedent: 24 prior Pattern-2 applications (D-19..D-30 + E-2/E-3/E-7/E-9/E-10/E-11/E-19/E-22/E-23/E-25/E-26/E-42/E-43/E-54/E-55). Saves 7 lockstep updates. Wire labels: `mast_failure_classified` / `replay_resume` / `signal_match` / `eval_dataset_member`. Action_type wire string: `agentic_observability_event` (alphabetical golden position 0006).

(3) DECISION-2: action_type collision avoidance (DRIFT-2)

The pre-existing SsbActionType::FailureClassified at ssb.rs:5110 carries the 13-category SYMPTOM taxonomy {category, confidence_bp, step_id, source} from crate::regression. The plan body asked for the same `failure_classified` action_type for MAST hits — collision on wire tag AND payload shape. Decision: distinct variant AgenticObservabilityEvent with Kind::MastFailureClassified (wire label `mast_failure_classified`). Orthogonal channels: regression.rs = SYMPTOM, MAST = CAUSE.

(4) DECISION-3: REUSE orchestrator::mast (E-6 SHIPPED), do NOT create mast_classifier.rs

The canonical 14-variant MastMode enum + finding_to_mast_mode + ALL[14] + as_sql_str / from_sql_str / mode_category surface ALREADY EXISTS at crate::mast (E-6). Decision: place new classifier under `crate::observability::mast_classify` and REUSE the E-6 surface verbatim. `classify_closed_run(session_id, viewer)` walks SsbViewer rows + projects via `finding_to_mast_mode` + 2 row-pattern heuristics (step-repetition >= 3 -> FM-1.3, premature-termination on PASS+prior-FAILURE -> FM-3.1).

(5) DECISION-4: ssb_resume_from FAIL-CLOSED on locked_keys_auditor (D-9 SHIPPED)

`crate::observability::replay::ssb_resume_from` FAIL-CLOSED on any LockedKeysAuditError:
  * RegistryUnreadable -> ReplayOutcome::PredicateUnavailable (refuse all patches).
  * audit_diff returns violations -> ReplayOutcome::Refused.
  * Clean audit -> ReplayOutcome::Accepted, mint deterministic ReplaySessionId.

ReplaySessionId derivation uses blake3 (NOT blake2/sha2) per anti-stale-training. Hex prefix 16 chars + parent checkpoint id for idempotent dispatch.

(6) DECISION-5: REUSE providers::pricing::ModelPricing (C-14 SHIPPED), do NOT create pricing.rs

The plan body asked for a new `crates/providers/src/pricing.rs` "if absent". The file PRESENT at providers/src/pricing.rs (C-14). Decision: observability::pricing_rollup REUSES elai_providers::pricing::pricing_for_model. Unknown models yield 0.0 USD; caller surfaces unknown-model count as audit warning.

(7) DECISION-6: drawers/ path convention (NOT windows/)

Plan body says `crates/dashboard-v2/src/lib/windows/` for new drawers. Actual convention since Tier-A: `crates/dashboard-v2/src/lib/drawers/`. FailureModesDrawer.svelte lands under drawers/.

(8) DECISION-7: ssb-otel-export crate behind `otlp-export` feature (autonomous-mode APPROVAL)

Anti-slopsquatting verification (May 2026):
  * opentelemetry 0.31.0 — apache-2.0, open-telemetry org, last published Apr 2026.
  * opentelemetry_sdk 0.31.0 — apache-2.0.
  * opentelemetry-otlp 0.32.0 — apache-2.0, MSRV 1.75.0.
  * tracing-opentelemetry 0.32.x — mit.

All 4 apache-2.0 / mit by open-telemetry GitHub org. No slopsquatting risk. Per autonomous-mode override (session-prompt explicit authorisation), the deps are APPROVED. Pinned with caret strings in `crates/ssb-otel-export/Cargo.toml`, GATED behind `otlp-export` cargo feature. Default builds pull NONE of the OTel tree.

Live OTLP gRPC integration test is RUNPOD-PENDING — requires spun-up otelcol-contrib instance. Crate ships type-level translator + opt-in SsbOtelExporter constructor; live emit path is follow-up.

(9) DECISION-8: OTel content gating default-OFF (OpenLLMetry pattern)

`gen_ai.input.messages` and `gen_ai.output.messages` SUPPRESSED when ELAI_OTEL_TRACE_CONTENT is unset / "false". Matches Traceloop's OpenLLMetry `TRACELOOP_TRACE_CONTENT` default-off pattern. TranscriptDrawer + ssb_transcript Tauri command BOTH honor the gate — `body` blanked + `redacted=true` so the dashboard renders a placeholder. SsbOtelSpan::redacted() constructor mints zero-message-body shape.

(10) DECISION-9: RUNPOD-PENDING items (carry-forward to follow-up)

  * RUNPOD-PENDING-A — Live OTLP gRPC integration test (otelcol-contrib).
  * RUNPOD-PENDING-B — ELAI_EVAL_GATE_REGRESSION_THRESHOLD enforcement in COMMITTING substate.
  * RUNPOD-PENDING-C — Replay KV-cache state capture (Open Question 3; sidecar-instrumentation).
  * RUNPOD-PENDING-D — Clock + RNG interception (Open Question 4; sidecar-instrumentation).
  * RUNPOD-PENDING-E — Live LLM-backed signal classifier (offline sweep ships; LLM dispatch gated behind per-role runtime).

(11) DECISION-10: drift recount

Original cross-check (2026-05-19) marked 3 Tier-A drawers + MastMode as MISSING — both shipped already (E-6 + s-orch-e29 Tier-A). Actual still-missing scope at start of this run = 9 items, all landed in this commit:
  1. 4 Tauri commands.
  2. ssb-otel-export crate (behind otlp-export feature).
  3. AgenticObservabilityEvent SSB action_type + 4-arm kind + payload + ENV gate (counts as 1 substrate, not "15 OTel columns" — Pattern-2 payload supersedes flat-column schema).
  4. observability/ subtree (5 modules).
  5. FailureModesDrawer.svelte.
  6. classify_closed_run function.
  7. Decisions update (this section).
  8. --otlp-endpoint CLI flag.
  9. ssb-otel-export type-level translator (CLI tracing-subscriber init is RUNPOD-PENDING-A, out-of-scope).

(12) PAPERS / CITATIONS USED

  * arXiv:2503.13657 (Cemri et al. NeurIPS 2025, MAST) — REUSED via crate::mast (E-6).
  * laminar.sh/article/2026-04-23-top-6-agent-observability-platforms — Signals + transcript-first.
  * github.com/langfuse/langfuse — traces-to-datasets promotion.
  * dev.to/sreeni5018 — LangGraph checkpoint replay.
  * tianpan.co/blog/2026-04-12 — full I/O capture motivation (documented OUT-OF-SCOPE).
  * traceloop.com/docs/openllmetry — TRACELOOP_TRACE_CONTENT content-gating pattern.
  * opentelemetry.io/docs/specs/semconv/gen-ai/gen-ai-agent-spans — 1.40.0 semconv (status: Development per May 2026 sweep).

END E-29 Tier-B decisions
END E-29-decisions.md
