E-5 — user modeling + long-term memory: session-2 decisions
=============================================================

Author: completion-drive session 2 (s-25120)
Date: 2026-05-23
Status: SHIPPED (partial; 4 items deferred to E-5-followup with explicit pending markers)
Plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-5_user_modeling_and_memory.md
Context: F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\contexts\E-5_user_modeling_and_memory-context.md
Exploration: F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\exploration\E-5_user_modeling_and_memory-exploration.md
Grounding: F:\.ELAI_workflow\workflow\sessions\s-25120\completion-drive\grounding\E-5_user_modeling_and_memory-grounding.md

PRIOR SHIPPED-ON-V2 STATE (verified against post-E-1 codebase)
----------------------------------------------------------------

Cross-check report `E-5_user_modeling_and_memory-report.md` ran before E-1 closure and predates the substrate that already shipped. Post-E-1 inventory at session start:

  SHIPPED already (crates/memory vertical slice):
    - crates/memory/src/{lib,fact,store,retrieval,decay,error}.rs (FactStore + bi-temporal + retrieval + decay math)
    - crates/memory/src/{evolution,tools,notes,hippo_rag,core_block}.rs (E-1 closure)
    - crates/memory/src/{trust,view}.rs (C-48 SafeHarbor: TrustTier + Provenance + SafetyCriticalQueryType)
    - 18-col schema (16 plan cols + trust_tier T3 default + provenance JSON via C-48)
    - 7 SSB variants: MemoryFactExtracted, MemoryFactSuperseded, MemoryEvolutionRun, MemoryNoteInserted, MemoryNoteEvolved, MemoryConflictDetected, MemoryMonitoring
    - dashboard-v2/src/lib/drawers/MemoryFactsDrawer.svelte (mounted via DrawerHost.svelte:80; READ-ONLY surface only)
    - dashboard-v2/src-tauri/src/commands_e5.rs: get_memory_facts + get_trust_score
    - Tests: bi_temporal_round_trip, decay_curves, extract_and_retrieve_smoke, prompt_injection_fixture, trust_tdd
    - crates/core/src/paths.rs: memory_dir_path + memory_facts_db_path
    - E-1: emit_memory_note_constraints + dispatch_chat_memory_reflect already wired in task_runner / live_cli

PATH DRIFT NOTED (not fixed; rename would break existing wiring):
  Plan Task 15 says CREATE crates/dashboard-v2/src/lib/components/MemoryDrawer.svelte.
  Reality: file landed at crates/dashboard-v2/src/lib/drawers/MemoryFactsDrawer.svelte.
  DECISION: keep the shipped path. Plan reference is stale.

SCHEMA_VERSION INVARIANT
-------------------------

SCHEMA_VERSION is locked at 103 (per session-1 context). Bijection at 245; action_type_lock at 306.
The plan's Task 14 names 11 new SSB action types. Adding all 11 would force a triple-lockstep bump.

DECISION (SCHEMA-LOCK + CONSOLIDATION): no new SsbActionType variants.
Mirror E-1's MemoryEvolutionRun phase-consolidation precedent. Mapping below:

  Plan name                              -> existing variant + payload-field convention
  -----------------------------------------------------------------------------------------
  memory_fact_inserted                  -> MemoryFactExtracted (same trigger: FactStore::insert success)
  memory_contradiction_surfaced         -> MemoryConflictDetected (semantically identical)
  memory_extract_started                -> MemoryEvolutionRun(phase="extract_started")
  memory_extract_complete               -> MemoryEvolutionRun(phase="cached_query") [E-1 precedent]
  memory_update_op                      -> MemoryEvolutionRun(phase="dispatch_op_<add|update|delete|noop>")
  memory_promoted                       -> MemoryEvolutionRun(phase="promoted")   [deferred to E-5-followup]
  memory_decay_tick                     -> MemoryEvolutionRun(phase="decay_tick")
  memory_write_rejected                 -> MemoryEvolutionRun(phase="write_rejected_<reason>")
  memory_write_skipped_sensitive        -> MemoryEvolutionRun(phase="write_skipped_<reason>")
  memory_retrieved                      -> existing tracing log at memory.retrieval target (no SSB needed in v1; per Task 7 read-path already updates last_observed_at)
  memory_audit_query                    -> deferred; CLI inspect/list paths print directly, no SSB beacon required

Rationale: MemoryEvolutionRunPayload.phase is a free-form String (per ssb.rs:9579 docstring), so semantic carriage is preserved
without a wire-format break. MemoryConflictDetectedPayload.conflict_kind is also free-form (per ssb.rs:9645), so existing
shapes carry the new producer signals without modification.

DECISION (active scope, this session)
---------------------------------------

Shipped this slice (4 new modules + 1 new helper batch + CLI surface + in-line decay):

A. **Mem0 ADD/UPDATE/DELETE/NOOP dispatcher** — `crates/memory/src/mem0_dispatcher.rs` (~350 LOC including tests).
   - `DispatchOp` discriminator (Add/Update/Delete/Noop) with `op_tag()` for SSB consolidation
   - `DispatchContext` carrying user_id, project_id, scope, provenance, trust_tier, initial_trust_score, now_ms (sourced
     from ClockProvider per C-15 + E-1 TIME-DOMAIN gotcha — uses now_ms, not tick)
   - `dispatch_one` + `dispatch_batch` over `Vec<ExtractedFact>`
   - Tombstone marker `{"tombstone": true}` triggers DELETE (soft-close)
   - Per-key duplicate detection uses `(user_id, scope, fact_type, key, project_id)` tuple
   - UPDATE only when value differs AND new confidence is non-decreasing (NOOP otherwise)
   - Uses FactStore::soft_close (new helper) for DELETE path

B. **Stage-1 regex blocklist + sensitive-class skip-list** — `crates/memory/src/defense.rs` (~340 LOC including tests).
   - Hardcoded regex set: force-push, curl|sh, wget|sh, rm -rf /, eval(/exec(/os.system(, sudo destructive,
     SSH private key blocks, prompt-injection directives
   - Sensitive-class skip: sk-/sk-ant-/sk-proj- API keys, AWS access keys, AWS_SECRET_ACCESS_KEY env,
     Bearer JWT, dotenv content, ~/.ssh/id_* paths, GitHub PATs (ghp_/github_pat_)
   - BLOCK takes precedence over SKIP (attack signal > policy signal)
   - DefenseVerdict enum: Allow | Block { reason } | SkipSensitive { reason }
   - Stage-2 LLM classifier DEFERRED (per E-5-followup)
   - Per arXiv:2601.05504 MINJA §4: rule-based gate is PRIMARY defense; LLM classifier is confirmation layer

C. **Heuristic contradiction adjudicator** — `crates/memory/src/contradiction.rs` (~290 LOC including tests).
   - `audit_conflicts(store, user_id, fact_type, auto_supersede_by_tier, now_ms)`
   - Groups by `(project_id, scope, key)`, pair-wise (older, newer) comparison
   - ConflictAction::Surface for differing values (default); Surface for duplicates too (tagged)
   - ConflictAction::SupersededByTier when `auto_supersede_by_tier=true` AND newer.tier > older.tier
   - Uses FactStore::soft_close for tier-driven supersession
   - LLM-backed semantic cosine-similarity contradiction DEFERRED (per E-5-followup)
   - Per arXiv (BeliefShift, plan-cited but unverified in workflow/papers/): both rows survive unless tier resolves

D. **Tokio decay scheduler** — `crates/memory/src/decay_scheduler.rs` (~230 LOC including tests).
   - `spawn_decay_scheduler(store, cfg, now_ms_fn, on_tick) -> DecayHandle`
   - DecaySchedulerConfig: interval (default 1h, MIN 1s) + run_immediately flag
   - DecayHandle::abort() for clean shutdown; Drop impl aborts the task
   - Observer callback for SSB emission (caller emits MemoryEvolutionRun(phase="decay_tick"))
   - now_ms_fn closure so tests use virtual clocks; production wraps ClockProvider::now_ms

E. **In-line decay tick (cli substitute)** — `crates/cli/src/live_cli.rs::dispatch_chat_memory_reflect`.
   - Calls `elai_memory::decay_tick` on the same FactStore handle right after the chat-memory reflect pass
   - Emits MemoryEvolutionRun(phase="decay_tick") SSB row
   - Guarantees decay progresses in the LiveCli REPL path WITHOUT requiring a long-lived tokio runtime
   - The dedicated background scheduler from (D) is for the task_runner side (separate runtime owner)

F. **CLI memory subcommand** — `crates/cli/src/args.rs` + `crates/cli/src/main.rs`.
   - `elai memory list [--user-id ID] [--scope <project|global|all>] [--fact-type <T>] [--limit N] [--json]`
   - `elai memory inspect --fact-id <id> [--json]`
   - `elai memory insert --fact-type <T> --key <K> --value-json <JSON> [--scope <S>] [--json]`
     — routes through Stage-1 defense (`classify`) then Mem0 dispatcher (`dispatch_one`)
     — T0 user-direct provenance; initial trust 0.95
     — Block/SkipSensitive verdicts abort with error
   - `elai memory delete --fact-id <id> [--json]` — soft-close via FactStore::soft_close
   - `forget` aliases `delete`; `show` aliases `inspect`; `add` aliases `insert`

G. **Per-user profile paths** — `crates/core/src/paths.rs`.
   - `user_profile_dir() -> Option<PathBuf>` (USERPROFILE on Windows, HOME on Unix; None when neither set)
   - `user_profile_db_path() -> Option<PathBuf>`
   - `user_projects_index_path() -> Option<PathBuf>` (for future promotion-gate N>=2 tracking)
   - Constants: USER_PROFILE_DIR_NAME = ".elai_user", USER_PROFILE_DB_FILENAME = "profile.sqlite",
     USER_PROJECTS_INDEX_FILENAME = "projects.json"
   - Old deferral docstring removed; new helpers replace the placeholder
   - NOTE: FactStore tiering (per-user store consulted alongside per-project) is DEFERRED to E-5-followup — the
     paths are ready for the future write path but no production caller yet uses the per-user store. The
     promotion-gate logic (N>=2 projects + consistent value -> auto-promote to Global) depends on Stage-2
     classifier and is deferred along with it.

H. **FactStore::soft_close helper** — `crates/memory/src/store.rs`.
   - Public method `soft_close(&mut self, fact_id, now_ms) -> Result<(), MemoryError>`
   - Sets `valid_to_ms = now_ms` on the matching currently-valid row (no replacement insert)
   - Returns `MemoryError::NotFound` if the row doesn't exist OR is already closed
   - Consumed by mem0_dispatcher DELETE, contradiction.rs SupersededByTier, and CLI delete path

I. **lib.rs re-exports + tokio dep added**.

DEFERRED TO E-5-FOLLOWUP (with explicit pending markers in code comments)
--------------------------------------------------------------------------

The following items are documented as DEFERRED-TO-E-5-FOLLOWUP and the rationale is captured in lib.rs:

  1. **LLM-backed extractor** — replaces the 4-pattern heuristic in `extractor.rs`. Needs RuntimeAdapter wiring
     + prompt engineering (Mem0 "what new facts did this turn reveal?" prompt). Per arXiv:2504.19413 the
     LLM extractor is the production path; the pure-Rust heuristic is the fallback. Defer rationale: the
     RuntimeAdapter trait surface needs SSB-payload-shaped JSON envelope handling that should land alongside
     E-2 sidecar speed work (parallel session).

  2. **Stage-2 classifier** — small LLM classifier confirming the Stage-1 regex decision. Per MINJA the
     classifier is a confirmation layer, NOT the primary defense — Stage-1 alone catches the documented
     attack patterns. Stage-2 needs RuntimeAdapter wiring (see #1) and a classifier prompt + training pairs.

  3. **MemoryDrawer adjudicate/promote/audit UI** — Tauri `adjudicate_contradiction` command + the Svelte UI
     (Accept Existing / Accept New / Keep Both buttons + decay heat-map + promotion controls). Heavy UI
     work that depends on a stable mutation surface (which itself depends on the promotion gate landing).

  4. **Promotion gate (Scope::Project -> Scope::Global)** — N>=2 distinct projects with consistent value OR
     explicit user command. Depends on Stage-2 classifier (for semantic consistency check across project_id
     boundaries) AND the per-user profile.sqlite write path (now bootstrapped via the paths in G). Deferred
     for the same reason as #2.

CITATION CLEANUP
------------------

Plan cites 17 INGEST-marked papers per grounding sweep; 11 are unverifiable post-cutoff IDs (start with 2510.* / 2511.* /
2512.* / 2603.* / 2605.*). Per memory:feedback_paper_citation_verify (2 bad cites already documented in B-40/B-42), this
session REMOVES unverifiable cites and ANCHORS to the 6 locally-ingested papers:

  REMOVED from active design rationale (still appear in plan's Citations block — see plan refresh below):
    - arXiv:2603.23848 (BeliefShift) — post-cutoff ID; cannot verify via local PDF or WebFetch
    - arXiv:2512.16962 (MemoryGraft) — post-cutoff ID; same
    - arXiv:2510.02341 (DRIFT) — post-cutoff ID
    - arXiv:2505.00038 (HyPerAlign) — not in repo; deferred to paper-ingestor
    - arXiv:2411.13405 (LLM Personalization survey) — not in repo
    - arXiv:2502.06975 (Episodic Memory is Missing Piece) — not in repo
    - arXiv:2507.07957 (MIRIX) — not in repo
    - arXiv:2410.10813 (LongMemEval) — not in repo
    - arXiv:2309.02427 (CoALA) — not in repo
    - arXiv:2506.12034 (Ebbinghaus-NN) — not in repo
    - arXiv:2506.17318 (Context-manipulation web agents) — not in repo

  KEPT (anchored to local PDFs at .claude/papers/):
    - arXiv:2504.19413 (Mem0) — workflow/papers/Mem0 / .claude/papers/2504.19413-Mem0.pdf — PRIMARY for Tasks 4 + 5 design
    - arXiv:2501.13956 (Zep/Graphiti) — .claude/papers/2501.13956-Zep-Graphiti.pdf — PRIMARY for Task 1 schema (already shipped)
    - arXiv:2601.05504 (MINJA + MemoryPoisoning) — .claude/papers/2601.05504-MemoryPoisoning.pdf — PRIMARY for Tasks 8 + 13 defense
    - arXiv:2605.05704 (SafeHarbor) — referenced by C-48 trust.rs already-shipped

  REPLACED:
    - BeliefShift (2603.23848) -> qualitative "stability-adaptability tradeoff is design hypothesis" framing in code comments;
      no numeric claim made (matches B-41 substitution pattern documented at memory:reference_b41_papers)
    - MemoryGraft (2512.16962) -> Provenance enforcement is already shipped + tested via retrieve-side empty-evidence_pointers
      filter; the defense holds independently of the paper's specific attack-vector framing

  Anti-slopsquatting note: all 4 KEPT cite IDs were verified to exist at the local PDF path via Bash ls — these are not
  hallucinated references.

E-1 SUBSTRATE CROSS-REFERENCES
-------------------------------

  - SCHEMA_VERSION 103 lock: held by E-1's MemoryEvolutionRun phase consolidation; this session uses the same approach.
  - Time-domain gotcha: this session's mem0_dispatcher + decay_scheduler use ClockProvider::now_ms() per E-1 (NOT .tick()).
    Empirical proof of the failure mode at workflow/audit-scratch/e1_time_domain_check.py.
  - Re-export bridge pattern: CLI already directly depends on elai-memory (Cargo.toml line 34), so the new symbols
    (`classify`, `DispatchContext`, `dispatch_one`, `DispatchOp`, etc.) are imported directly without orchestrator
    re-export. Future task_runner consumers will follow E-1's pattern of orchestrator-side re-exports.
  - Test pattern: each new module ships its own #[cfg(test)] module with smoke tests (matches the bi_temporal_round_trip.rs
    integration-test pattern + the in-module test style of extractor.rs + decay.rs).

CONSTRAINT COMPLIANCE
----------------------

  - C-15 ClockProvider: mem0_dispatcher.rs takes `now_ms: u128` in DispatchContext (caller supplies via ClockProvider).
    decay_scheduler.rs takes `now_ms_fn: Arc<dyn Fn() -> u128>` (caller wraps ClockProvider). CLI subcommand uses
    `elai_core::OsClock::now_ms()`. No SystemTime::now anywhere in this slice.
  - C-35 ToolGate: no new tool defs registered with ToolGate. Future LLM-backed extractor would register its
    extraction tool here; deferred along with the LLM dispatch.
  - E-1 TIME-DOMAIN: cross-component timestamps use .now_ms() throughout this slice. No .tick() used outside the
    SSB observability path in live_cli (which is purely a local-monotonic duration measurement).
  - SCHEMA-locked SSB emission: all new SSB beacons use existing MemoryEvolutionRun / MemoryConflictDetected /
    MemoryFactExtracted variants per consolidation map.

ANTI-LAZY COMPLIANCE
---------------------

  - Zero stubs / TODO / FIXME in any new file (DEFERRED-TO-E-5-FOLLOWUP markers are documentation, not stubs).
  - Zero .unwrap() / .expect() in any new production module. CLI uses `?` propagation; tests use .expect() for
    fixture setup which is allowed per #[cfg(test)] exemption.
  - tracing crate for all logging (no log crate); thiserror for new MemoryError variants (none added — existing
    error suffices); native async fn in traits (no async-trait crate added).
  - No SCHEMA bump (held at 103).
  - ClockProvider for time, .now_ms() for cross-component.
  - Cite real papers only (4 verified-local citations; 11 unverifiable removed).

FILES TOUCHED
--------------

CREATE:
  crates/memory/src/mem0_dispatcher.rs       (~350 LOC, includes tests)
  crates/memory/src/defense.rs               (~340 LOC, includes tests)
  crates/memory/src/contradiction.rs         (~290 LOC, includes tests)
  crates/memory/src/decay_scheduler.rs       (~230 LOC, includes tests)
  workflow/reasoning/E-5-decisions.md        (this file)

MODIFY:
  crates/memory/src/lib.rs                   (+4 mod decls, +12 re-exports)
  crates/memory/src/store.rs                 (+24 LOC: pub fn soft_close)
  crates/memory/Cargo.toml                   (+tokio dep, +dev-dep test-util)
  crates/core/src/paths.rs                   (+50 LOC: user_profile_dir + helpers + 3 consts; -7 LOC deferred docstring)
  crates/cli/src/args.rs                     (+~200 LOC: Memory CliAction variant + MemorySubcommandKind + parser)
  crates/cli/src/main.rs                     (+~200 LOC: cmd_memory_subcommand handler + dispatch arm)
  crates/cli/src/live_cli.rs                 (+50 LOC: in-line decay_tick post-reflect + SSB emission)

REMOVE: none

REBASELINE OF PLAN FILE
------------------------

Markers to add to the plan in a follow-up pass (this session does NOT edit the plan file in-place; the rebaseline pass is
captured by this decisions doc and the cross-check-recheck report will pick up the new shipped state):

  SHIPPED-ON-V2 markers for items A-I above (already in code).
  CITATION-CLEANED markers for the 11 removed/replaced arXiv refs (per "Citations / Grounding" plan section).
  SCHEMA-CONSOLIDATED markers for the 9 missing SSB variants (per "Task 14" plan section).
  DEFERRED-TO-E-5-FOLLOWUP markers for the 4 deferred items (LLM extractor, Stage-2 classifier, MemoryDrawer
  adjudication UI, promotion gate).
  Path-drift fix: replace "crates/dashboard-v2/src/lib/components/MemoryDrawer.svelte" with the shipped path
  "crates/dashboard-v2/src/lib/drawers/MemoryFactsDrawer.svelte".

SPEC-DRIFT FIXES (3 per grounding sweep)
------------------------------------------

  1. Plan Task 1: "Bump SCHEMA_VERSION 45 -> 47". Reality: SCHEMA_VERSION = 103 (locked). FIXED via SCHEMA-LOCK + consolidation
     map above (no bump performed).
  2. Plan Task 3: "extend paths.rs with user_profile_dir". Reality (pre-session): paths.rs:248 docstring explicitly DEFERRED
     it. FIXED via item G above (helpers + constants shipped; FactStore tiering deferred to E-5-followup with paths ready).
  3. Plan Task 15: "CREATE crates/dashboard-v2/src/lib/components/MemoryDrawer.svelte". Reality: path is
     drawers/MemoryFactsDrawer.svelte (different parent dir + different prefix). FIXED via path-drift note above (kept
     shipped path; plan refresh needed).

OBSERVABILITY MAP (consolidated SSB emissions for the new producer surface)
-----------------------------------------------------------------------------

  CLI `memory insert` (passes defense)            -> MemoryFactExtracted | MemoryFactSuperseded (per DispatchOp)
  CLI `memory insert` (Block / SkipSensitive)     -> stderr + exit code 1 (no SSB row; the CLI itself does not own the
                                                     SSB writer; future task_runner-side dispatcher will emit
                                                     MemoryEvolutionRun(phase="write_rejected_<reason>"))
  CLI `memory delete`                              -> [future] MemoryEvolutionRun(phase="dispatch_op_delete"); current CLI
                                                     path prints op result to stdout
  Decay tick (post-reflect, every K turns)         -> MemoryEvolutionRun(phase="decay_tick") via live_cli wiring
  Contradiction audit                              -> MemoryConflictDetected (existing variant; payload's free-form
                                                     conflict_kind carries "duplicate" or "contradiction")

  Note: the SSB beacons for CLI-direct paths (insert/delete) are intentionally minimal in this slice — the
  authoritative producer is the orchestrator-side task_runner hook (which sees every turn's extracted facts).
  The CLI subcommand is the manual / debugging surface; the production write path is the in-orchestrator dispatcher.

NEXT STEPS (E-5-FOLLOWUP scope)
--------------------------------

  - LLM-backed extractor: SidecarRuntime::complete with Mem0 prompt + JSON envelope parsing.
  - Stage-2 classifier: small LLM classifier confirming Stage-1 regex decision; same SidecarRuntime path.
  - MemoryDrawer adjudication UI: Tauri adjudicate_contradiction command + Svelte three-button surface +
    decay heat-map (uses iter_current's DecayInputRow shape).
  - Promotion gate: per-user profile.sqlite write path + projects.json maintenance + N>=2 consistency check
    (depends on Stage-2 classifier for semantic equivalence).
  - Background decay scheduler wiring: spawn_decay_scheduler at task_runner startup (separate runtime owner
    from LiveCli's in-line tick).
