B-4 Knowledge Producer Wiring — Decision Log

Session: s-25120
Phase: B-4 (completion-drive closure)
Date: 2026-05-22

CONTEXT

B-4 plan was drafted against an older codebase snapshot. Cross-check-recheck
report (workflow/sessions/s-25120/cross-check-recheck/B-4_knowledge_producer_wiring-report.md)
classified 19 IMPLEMENTED / 5 PARTIAL / 3 MISSING / 2 NOT_WIRED of 29
claims. Several plan tasks (#17, #18, #3 partial) had already landed on V2
via B-16A graceful shutdown + anti-stale sweep. The closure session
focused on:
1. REBASELINE: mark already-shipped tasks SHIPPED ON V2 + correct line
   citations.
2. SUBSTANTIVE: close real partials (#8 toml read, #11 freshness wire,
   #12 vector cache, #14 canary drift, #16 latency=0) + 3 MISSING wires
   (CanaryDrift variant, periodic cascade evaluator, this decisions doc).
3. WIRES: 1 new Tauri command (get_vector_cache_hit_rate) + 3 stub.ts
   wrappers + 2 new KnowledgeDrawer panels + 1 SettingsWindow canary row.

DECISION 1 — CanaryDrift variant added now (not deferred to C-10)

Question: defer CanaryDrift to C-10 (per the C-3 stub docstring at
commands.rs:1469-1475 "When C-10 emits the row, swap the body to
viewer.latest_action_of_type(...)") or land it inside B-4 closure?

Choice: land in B-4 closure via QUINTUPLE-UPDATE pattern.

Rationale:
- The decisions briefing from the session controller pre-authorized
  adding CanaryDrift in B-4 ("Apply the B-3 QUINTUPLE-UPDATE pattern to
  add CanaryDrift ourselves. Same 7 sites as the 4 variants B-3 added").
- C-10 has no schedule; deferring leaves get_canary_drift returning
  hardcoded zeros indefinitely (current state per stub docstring).
- Adding the variant + payload struct + producer + dashboard reader is
  ~150 LOC end-to-end. Skip cost > land cost.
- The producer (cli::live_cli::spawn_cascade_evaluator_loop) needs
  somewhere to emit the row regardless; building it without the
  variant just defers the QUINTUPLE-UPDATE risk.

Tradeoff: SCHEMA_VERSION bumps 98 -> 99 inside B-4 instead of C-10.
all_test_payloads().len() asserts now require updating (231 -> 232 +
count string). The bump is mechanical and the bijection test catches
any of the 7 sites being missed (per gotchas.md QUINTUPLE-UPDATE rule).

DECISION 2 — Periodic cascade evaluator uses synthetic breaker states

Question: how does the periodic cascade evaluator get access to the
real EmbeddingCircuitBreaker handles that live inside
pipeline.vector_module (which is moved into LiveCli.knowledge_pipeline
and therefore not borrowable from a tokio::spawn)?

Choice: emit CircuitBreakerStateChange rows derived from the cascade's
current SearchMode (HYBRID -> all CLOSED; CODE_ONLY -> text breaker
OPEN; TEXT_ONLY -> code OPEN; SPARSE_ONLY -> both dense OPEN;
BM25_ONLY -> all OPEN). Real breaker introspection is tracked for a
follow-up B-N once VectorModule exposes a `breakers() -> Vec<Arc<...>>`
method.

Rationale:
- Real breaker introspection requires either (a) exposing breakers via
  the VectorModule trait (cross-cutting refactor) or (b) using
  Arc<Mutex<EmbeddingCircuitBreaker>> shared between the pipeline and
  the spawn task (lifetime tangle with the consumed pipeline). Both
  are out of scope for B-4 closure.
- Synthesizing from cascade mode preserves the dashboard contract:
  get_embedding_breakers stops returning the hardcoded fallback list
  once the periodic loop emits its first snapshot row.
- The mapping is grounded in degradation.rs::DegradationCascade::evaluate
  itself — same arms.

DECISION 3 — Latency=0 fix preserves entropy quarantine

Question: pipeline.rs:747/763/870 had `latency_ms: 0`. Use raw
std::time::Instant or self.clock for the gate-fire emit duration?

Choice: self.clock.now() / elapsed_since — uniform with the existing
per-stage measurement convention (pipeline.rs:333,344,383,392,420).

Rationale:
- Entropy quarantine rule (pipeline.rs:60): "The pipeline never
  touches std::time::Instant directly; all clock reads go through
  core::clock::ClockProvider."
- Tests use SeededClock for deterministic latencies; raw Instant would
  break determinism.
- The deep_cap / rrf_gate stages do not have independent heavy work —
  the work was done inside rrf_rerank_with_tier above. The new latency
  surfaces the emit-side cost (sink dispatch + stage push), which is
  the only meaningful per-row cost for those synthetic stages.

DECISION 4 — get_recall_metrics probes multiple relative paths

Question: dashboard process CWD is not guaranteed; how does
get_recall_metrics find `metrics/phase_3K_rag.toml` reliably without
baking an absolute path?

Choice: probe 5 canonical relative paths in order (`metrics/`, `../metrics/`,
`../../metrics/`, `../../../metrics/`, `.elai_cc/metrics/`). First
parse-success wins; fall through to hardcoded literals on no match.

Rationale:
- Tauri binaries are typically launched from `.elai_cc/crates/dashboard-v2/`
  (during `pnpm tauri:dev`) or from `.elai_cc/` (production exe). Both
  paths must work.
- An absolute path bakes deployment knowledge into the binary; the
  Tauri shell could be relocated by ops.
- The hardcoded fallback is the last-known measured values
  (2026-04-15 run from phase_3K_rag.toml at session-time) so the
  dashboard renders sane defaults even when the toml moves.

DECISION 5 — Plan rebaseline strategy: rewrite tasks in place

Question: how to surface the V1 -> V2 task drift (stale paths, already
shipped sub-tasks) — write a separate REBASELINE diff, or rewrite the
tasks in place?

Choice: rewrite each affected task with a SHIPPED ON V2 prefix +
explicit "V1 cite WAS X / V2 cite IS Y" mapping in a top-level
REBASELINE NOTE.

Rationale:
- The plan is canonical for future B-N runs — leaving stale citations
  forces every subsequent reader to do the same diff work the closure
  session did.
- The REBASELINE NOTE at the top of the file gives a one-glance
  V1/V2 mapping; the per-task prefix is the searchable marker for
  cross-check-recheck classifiers.
- Idempotent: re-running the plan would re-detect SHIPPED ON V2 tasks
  and skip them (no churn).

OUTCOMES

- Build: cargo check --workspace green (warnings only, no errors).
- Tests: cargo test -p elai-orchestrator --lib ssb -> 55/55 PASS
  (includes test_typed_payloads with 232 payloads). Pipeline tests
  -> 29/29 PASS. C1 SSB contracts -> 16/16 PASS.
- Schema: SCHEMA_VERSION 98 -> 99 (CanaryDrift QUINTUPLE add).
- New surface: 1 SsbActionType variant + 1 payload struct + 1 Tauri
  command + 3 stub.ts wrappers + 3 KnowledgeDrawer panels (vector
  cache + canary drift + real freshness wire) + 1 SettingsWindow
  canary drift row.
- Removed: `#[allow(dead_code)]` from SsbDegradationNotifier::new.
- Plan: 6 tasks marked SHIPPED ON V2 with current line citations;
  REBASELINE NOTE added; DETERMINISTIC_CHECK rewritten to point at
  the actual modified files.

FOLLOW-UP

- Real EmbeddingCircuitBreaker introspection (replace synthetic
  mapping in cascade_mode_to_breaker_states) — track as B-N follow-up
  after VectorModule exposes a `breakers() -> Vec<Arc<...>>` accessor.
- DEEP MRR < STANDARD MRR (still 0.5017 < 0.6000 in current
  phase_3K_rag.toml) — algorithmic fix is outside B-4 scope;
  dashboard now surfaces the violation via DEEP-MRR-warn badge.
- get_freshness_state remains `known:false` until a
  freshness-decay-emit producer lands (orthogonal anti-staleness
  extension). The disabled-fallback string is now visible in the
  KnowledgeDrawer FRESHNESS DECAY panel.
