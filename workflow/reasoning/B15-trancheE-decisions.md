B-15 Tranche E decisions (s-22600 sub-phase trancheE; 2026-05-31)

Scope
- Three bounded burndown items carved out of s-22300 stubs.md B-15 deferrals:
  E.1 CLI crate entropy migration (SystemTime::now / Instant::now -> ClockProvider) for the production-side subset that does not carry an existing exempt marker.
  E.2 rules crate .unwrap() / .expect() burndown (revealed-as-null-result; see B15-E2-rules-unwrap-null-result.md).
  E.3 dashboard-v2 BTreeMap DTO migration for 4 IPC-wire fields (FxHashMap -> BTreeMap on serialization-observed maps).
- cli/main.rs 1269 LOC regrowth refactor EXPLICITLY OUT-OF-SCOPE per task brief; remains s-22300 stub B15-cli-regrowth.

Authoritative inputs
- F:\.ELAI_workflow\workflow\sessions\s-22600\context-package-trancheE.md (10 DE# decisions, per-file table for all three items, validation warnings W1-W8).
- F:\.ELAI_workflow\workflow\sessions\s-22300\stubs.md (canonical scope source for B15-entropy-burndown, B15-unwrap-burndown, B15-dashboard-BTreeMap rows).
- F:\.ELAI_workflow\workflow\reasoning\Tranche-C-B15-decisions.md (DECISION-5 / DECISION-7 / DECISION-8 / DECISION-9 / DECISION-10 -- per-site judgment precedent + BTreeMap rationale).
- F:\.ELAI_workflow\workflow\memory-bank\gotchas.md (B15_DETERMINISM_MAP_TYPES @L1159-1164; E38_CLOCK_PROVIDER_SHIPPED @L1278-1283; C15_ENTROPY_ALLOW_RENDER_ADAPTERS_REPLACE_WHEN_THREADED @L1516-1521).
- F:\.ELAI_workflow\.elai_cc\crates\core\src\clock.rs (ClockProvider trait @L43; OsClock @L81 Copy zero-sized; SeededClock @L122; default now_ms / now_secs impls @L59 / L67 with .unwrap_or(0) saturation built in).

DE1 -- CLI ENTROPY: INLINE OsClock OVER ARC THREADING
- Use INLINE `elai_core::OsClock.now_ms()` / `OsClock.now_secs()` / `OsClock.instant_now()` reads at fn body for all single-shot sites (archive_verify::current_ms, bench_arc::now_ms_safe, bench_conformal::run_async now_ms capture, credit_cmd::now_ms, eval_cmd::current_ms, handoff::emit_session_fork ts_ms, dkg_lifecycle::ensure_dkg_or_rebuild now_secs + started_at).
- Build `let clock: Arc<dyn ClockProvider> = Arc::new(OsClock);` at fn-entry ONLY for task_cmd::run_task_async (5 reads across one fn body: started_at + deadline + remaining + tail_deadline + tail loop probe).
- Rejected: full Arc<dyn ClockProvider> threading through every CLI fn signature -- adds noise for one-shot reads when OsClock is Copy zero-sized; matches existing CLI precedent (handoff.rs already calls `elai_core::OsClock.now_ms()` inline at 3 sites @L357/385/565 per ctx-compiler verification).
- Grounding: OsClock is `#[derive(Debug, Clone, Copy)]` zero-sized struct @clock.rs:80-81; `OsClock.now_ms()` is the trait default impl absorbing `.unwrap_or(0)` saturation @clock.rs:59-64; lib.rs ground truth: zero allocation, identical semantics to direct `SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)` pattern that currently appears in every migrated site.

DE2 -- STRUCTURED_OUTPUT_CMD::NOW_MS: EXEMPT-MARKER OVER MIGRATE
- Add `// entropy-allow: cli-smoke-observability-ts-ms-only` marker above the existing `fn now_ms` body @L467-478. Keep the SystemTime::now() body as-is.
- Matches the live_cli.rs:274 cascade-ssb-emit-free-fn-boundary exempt precedent per C-15-decisions.md DECISION-CASCADE-TS; the fn doc-comment already grounds the rationale (this `ts_ms` is operator-observability only; the SSB writer's injected OsClock is the canonical timestamp). Minimum-diff path matching the C-15 exempt pattern.
- Rejected: migration to inline OsClock would change the wire-shape return type pathway through CodeAct sandbox + cloud strict-schema marker emit; the producer is already canonical via ssb.rs injected OsClock.

DE3 -- MEASURE.RS MODULE-SCOPE EXEMPT PRESERVED (no edits)
- Per C-15-decisions.md DECISION-MEASURE: cli/measure.rs has module-scope `#![allow(clippy::disallowed_methods)]` @L23-43 + scripts/check_entropy_quarantine.sh excludes the file from the entropy scan. All 17 SystemTime::now / Instant::now sites are benchmark-intrinsic measurement intent; replacement with a ClockProvider would invalidate every phase_4*.toml metrics output.
- Rejected: per-site migration on measure.rs; only the module-scope exempt is the authoritative path.

DE4 -- LIVE_CLI.RS:274 CASCADE-EXEMPT PRESERVED (no edits)
- Per C-15-decisions.md DECISION-CASCADE-TS: the `entropy-allow: cascade-ssb-emit-free-fn-boundary` marker @live_cli.rs:274 stays. Migrating to ClockProvider would re-introduce the determinism leak path that the exempt was created to bound (gotchas.md gotcha 1904 same shape).
- The other two SystemTime::now mentions @live_cli.rs:266 + L337 are doc-comments referencing the symbol (not actual call sites); no edit.

DE5 -- SYMBOL_GRAPH.RS CLI-ENTRYPOINT EXEMPT PRESERVED (no edits)
- CLAUDE.md "main()/entrypoint/bootloader init where panic = won't start, you'll see it immediately" + entropy-allow markers @L55-56 + L207 cover the 2 call sites. CLI entrypoint exemption is the authoritative path for `built_at` timestamps.

DE6 -- RULES CRATE NULL-RESULT VERIFIED CLEAN
- All 287 .unwrap() + 13 .expect() in the rules crate sit inside cfg(test) blocks (per-file Bash verification: head -n $((cfg(test)_line - 1)) | grep -c '\.unwrap()' returns 0 for all 8 files with unwraps; verified self_discovery.rs:828, contract_tests.rs:7, rule_engine.rs:890, lifecycle.rs:572, interrogation_store.rs:657, predicate.rs:496, schema.rs:570, project_rule_seeder.rs:500).
- Production error surface already canonical thiserror: RulesDbError @error.rs:3-15 + LifecycleError @error.rs:18-63; both use `#[from] rusqlite::Error` for upstream propagation.
- CLAUDE.md "No .unwrap() in production / library Rust code" is satisfied; the test-code exception is the documented carve-out.
- Action: write workflow/reasoning/B15-E2-rules-unwrap-null-result.md + update s-22300 stubs.md row B15-unwrap-burndown to VERIFIED-COMPLIANT-NULL-RESULT (preserve row; add prefix to marker_text + pointer to the new doc).

DE7 -- DASHBOARD 4 IPC FIELDS BTREEMAP NOT FXHASHMAP
- Migrate the 4 serialization-observed DTO fields per gotchas.md B15_DETERMINISM_MAP_TYPES per-site judgment rule (@L1159-1164: SERIALIZATION-EXPOSED maps belong in BTreeMap; the Tranche C-1 FxHashMap migration was the safe-no-regression mechanical step; BTreeMap is the correct end-state).
- HeldOutStatsRow::by_source (commands_d6.rs)
- CanaryBreakdown::by_report (commands_d7.rs)
- LockedKeysRecentAudits::violations_by_audit (commands_d9.rs)
- GoalDriftStatusDto::firings_by_canary (commands_e46.rs)
- Per-file: ADD `use std::collections::BTreeMap;` at module head; REMOVE `use rustc_hash::FxHashMap;` ONLY when no other FxHashMap usage remains in the file (verified per-file via Grep -- d6/d7/d9 drop the import; e46 keeps it for `latest: FxHashMap<String, DualLedgerTaskDto>` @L110, a lookup-only intermediate not exposed as a wire DTO field).
- Serde wire shape: BTreeMap<String, T> serializes via serde_json identically to FxHashMap<String, T> (both emit a JSON object); wire compat PRESERVED.
- Rejected: HashMap (banned by check_hashmap_ban.sh); raw std::collections::HashMap is the deny token.

DE8 -- SVELTE FOLLOWUP DOC-MARKER ONLY (no UI edits this tranche)
- 4 Svelte panels (HeldOutPanel / CanaryPanel / LockedKeysPanel / PlanningLedgerPanel) NOT modified this tranche per task brief.
- HeldOutPanel.svelte:106 already does `.sort((a, b) => b[1] - a[1])` (value-DESC sort); BTreeMap key-order is orthogonal (sort happens in JS after deserialize); ZERO observable change.
- CanaryPanel.svelte / LockedKeysPanel.svelte / PlanningLedgerPanel.svelte use direct keyed lookup or empty-init only; no iteration patterns observed.
- Followup logged in B15-E3-trancheE-decisions.md: Svelte-side client-sort removal awaits dashboard owner confirmation; if future UI work adds `.entries()` iteration, the iteration semantic is now BTreeMap-alphabetical (key-sorted ascending) -- not a regression.

DE9 -- CLI/MAIN.RS REGROWTH DEFERRED (s-22300 stub B15-cli-regrowth stays open)
- 1269 LOC regrowth refactor EXPLICITLY OUT-OF-SCOPE per task brief; needs separate HEAVY-tier plan with impact-mapper sweep + run_subcommand.rs precedent reading.

DE10 -- ZERO SCHEMA / VARIANT / KIND CHANGES
- All 3 items add ZERO new SsbActionType / Kind arms / payload shape changes. SCHEMA_VERSION stays at 195. TOTAL_VARIANTS unchanged. ZERO golden-file regen.

Migration scope summary (verified, post-exemption)
- E.1: ~21 production sites across 8 CLI files (archive_verify, bench_arc, bench_conformal, credit_cmd, dkg_lifecycle, eval_cmd, handoff, task_cmd) + 1 exempt-marker on structured_output_cmd.
- E.2: 0 source edits; 1 null-result decision doc; 1 stubs.md row update.
- E.3: 4 IPC DTO fields + their construction sites across 4 dashboard files.

Build verdict (auditor-deferred per CLAUDE.md "Main model never runs tests")
- Executor ran scoped `cargo check -p elai-cli --tests` + `cargo check -p elai-dashboard-v2-tauri` after each item; auditor runs full `cargo test` in AUDITING substate.

Closure of s-22300 stub rows
- B15-entropy-burndown: PARTIAL-CLI-CRATE (CLI subset shipped; orchestrator / knowledge / providers / plugins still deferred per separate HEAVY plans).
- B15-unwrap-burndown: VERIFIED-COMPLIANT-NULL-RESULT (rules crate; orchestrator / knowledge / plugins still deferred per separate plans).
- B15-dashboard-BTreeMap: RUST-SIDE-DONE-SVELTE-PENDING (Rust DTO + accumulator types migrated; UI client-sort removal awaits dashboard owner sign-off).
- B15-cli-regrowth: UNCHANGED (still deferred to a separate HEAVY-tier plan).
