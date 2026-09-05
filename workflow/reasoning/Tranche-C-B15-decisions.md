Tranche C — B-15 burndown decisions

Session: s-22300
Date: 2026-05-31
Tranche: C (HashMap mechanical + Box<dyn Error> single-site)
Outcome: 14 production files migrated (C-1), 1 Box<dyn Error> site migrated (C-2), 3 follow-up categories DEFERRED (C-3)

SCOPE
- C-1 (HashMap -> FxHashMap mechanical sweep): 14 production files SHIPPED (exploration listed 16 candidates; 2 deferred per DECISION-2 below)
- C-2 (Box<dyn Error> -> anyhow::Result): 1 binary entrypoint SHIPPED (eval-corpus/src/bin/migrate_held_out.rs main())
- C-3 (.unwrap burndown + SystemTime entropy migration + cli/main.rs regrowth + dashboard DTO BTreeMap migration): DEFERRED per scope (see DECISION-7..10)

DECISIONS

DECISION-1 — Master-Index numbers are STALE. Exploration verified the live workspace counts vs the Master-Index #12 claims:
  HashMap-use-imports: actual 27 production / Master-Index claimed 48 (1.7x over-count by Master-Index)
  Box<dyn Error>: actual ~8 real signatures workspace-wide / Master-Index claimed 124 (~15x over-count after filtering Box<dyn Future>, Box<dyn AgentSandbox>, comment lines)
  SystemTime::now: actual 270 / Master-Index claimed 199 (35% UNDER-count by Master-Index — entropy migration has REGRESSED, not progressed)
  Outcome: Tranche C scope was right-sized down from the Master-Index claim. Tranche C is NOT the work of fully closing B-15.

DECISION-2 — Two dual-import orchestrator files DEFERRED (not migrated):
  (a) orchestrator/src/dual_ledger.rs has BOTH `use std::collections::HashMap;` (line 41) AND `use rustc_hash::FxHashMap;` (line 43). Exploration claimed the std import was "UNUSED RESIDUAL, just delete". Verification by direct grep: line 402 defines `pub fn by_task(&self) -> HashMap<String, (&TaskLedger, &ProgressLedger)>` — std HashMap is genuinely the return type of an IPC-boundary function (Svelte panel consumer uses it). Per CLAUDE.md HARD rule "verify the real implementation before removing", the std import IS in use. NOT migrated.
  (b) orchestrator/src/reasoning_template.rs has the same dual-import pattern. Verification: lines 685-687 define `fn _ensure_hashmap_alias_for_doc_consumers() -> HashMap<(), ()>` — a deliberate doc-anchor stub that uses std HashMap to keep the rustdoc cross-link alive after the rest of the module migrated to FxHashMap. NOT migrated.
  Both files retain BOTH imports. Recommendation: a future patch-N can audit whether these IPC-boundary / doc-anchor uses should migrate to FxHashMap (wire shape would not change for the IPC case since FxHashMap serializes via the same serde wire surface; the doc-anchor case needs the doc-string author's intent).

DECISION-3 — task_schema.rs migration CASCADED to 3 additional files (not in original C-1 file list):
  Changing `Task.cutoffs: HashMap<String, String>` to FxHashMap forced cascade updates in:
    crates/eval-corpus/src/ingest.rs (test fixture inside #[cfg(test)] mod tests)
    crates/eval-corpus/src/test_executor.rs (test fixture inside #[cfg(test)] mod tests)
    crates/eval-corpus/tests/storage_uses_family_shards.rs (integration test fixture)
  Exploration flagged "test code KEEP std::HashMap" as an idiom, but that rule applies to test-only files whose types are independent. When a test fixture CONSTRUCTS a production type that we just migrated, the test fixture has to follow. All 3 cascade edits were 1-line `HashMap::new()` -> `FxHashMap::default()` swaps; no behavior change. The integration test (storage_uses_family_shards.rs) compiles against `[dependencies]` (not [dev-dependencies]) so the rustc-hash dep added to eval-corpus Cargo.toml inherits without a [dev-dependencies] line.

DECISION-4 — Box<dyn Error> ground-truth count is 8 not 34. Exploration verified the Master-Index "34 occurrences" by reading every hit:
  22 hits = comment lines documenting "// no Box<dyn Error>" or "// thiserror NOT Box<dyn Error>" (documentation of the migrated state, not call signatures).
  Several = `Box<dyn Future<...>>` / `Box<dyn AgentSandbox>` / `Box<dyn RuntimeAdapter>` (trait-object returns over DOMAIN traits, not std::error::Error trait). Legitimate; NOT migration candidates.
  Some = mock-corpus string literals in knowledge/tests/splade_*.rs (test-fixture source code that intentionally contains the pattern to test classifier behavior).
  Real Box<dyn (std::error::)?Error> signatures: 8 workspace-wide.
  Of those 8: 1 mechanical (eval-corpus/src/bin/migrate_held_out.rs main()), 4 are in #[test] contract test signatures (KEEP — test idiom), 1 was already typed-error and miscategorized in the grep, 2 are intentional public-API trait-object returns at sandbox/runtime adapter boundaries (memory/src/error.rs, knowledge/src/error.rs pattern — typed errors, not std::Error).
  C-2 scope = the 1 mechanical site. Done.

DECISION-5 — Dashboard DTO HashMap -> BTreeMap migration NOT in C-1 scope (judgment call deferred). The 4 dashboard files migrated (commands_d6.rs, commands_d7.rs, commands_d9.rs, commands_e46.rs) carry IPC-wire fields (by_source, by_report, violations_by_audit, firings_by_canary). For UI determinism, BTreeMap would be the stronger choice — Svelte side gets stable alphabetical list ordering. FxHashMap preserves the EXISTING non-deterministic behavior (no regression), but doesn't FIX the underlying flakiness either. Rationale for shipping FxHashMap now: the change is mechanical / safe; BTreeMap is a judgment call that needs UI alphabetical-sort confirmation from the dashboard tile owner. Recommend a follow-up patch-N proposal once the dashboard tile rendering is reviewed.

DECISION-6 — Contract test pattern: explicit MIGRATED_FILES list rather than workspace scan. The b15_tranche_c_burndown_contract.rs test uses a hardcoded list of 14 file paths and per-file regression checks, NOT the e25-style workspace-walker collect_hits. Rationale: the negative-space at C-1 is "these specific files must STAY migrated", not "this pattern is banned workspace-wide" — the latter would false-positive on the 11 test/bench files that legitimately keep std::HashMap as Rust idiom.

DECISION-7 — C-3 .unwrap() burndown DEFERRED. 1720 occurrences across 202 files. Per CLAUDE.md HARD rule no .unwrap() in production/library code with exceptions for tests/main/const. The true production count is materially lower than 1720 (estimate 600-900 after stripping tests/benches), but still high enough that mechanical-replacement requires per-site judgment (some unwraps are provably safe assertions; others are panic-on-None bugs). Not a clean mechanical burndown. Recommend a separate HEAVY-tier plan that scopes to ONE crate at a time (e.g. rules crate has 287 unwraps in 7 files, hand-auditable in one session).

DECISION-8 — C-3 SystemTime::now / Instant::now -> ClockProvider migration DEFERRED. 270 occurrences across 128 files (35% HIGHER than Master-Index baseline 199 — entropy has REGRESSED). ClockProvider trait IS shipped at crates/core/src/clock.rs (gotchas.md L1244-1246). The residual is the long tail of bare SystemTime::now() / Instant::now() that did not migrate. Each call site needs a clock argument; signatures cascade through the call graph. NOT mechanical. Heaviest residual concentrations: orchestrator (49 files, session_dag.rs 171 sites worst-single), knowledge (5 files), cli (15 files including measure.rs 17 sites alone). Recommend a separate HEAVY-tier plan that scopes to ONE crate at a time (cli would be the bounded-blast-radius starting point at ~30 sites).

DECISION-9 — C-3 cli/main.rs regrowth (252 -> 1269 LOC) DEFERRED. 5x baseline regrowth. Companion regrowth: args.rs 8013 LOC (massive), live_cli.rs 6016 LOC, run_subcommand.rs 501 LOC. NOT mechanical — requires reading main.rs end-to-end + identifying which command handlers + helper fns extract to dedicated modules. cli/src/ already has 50+ subcommand modules; the extraction pattern is established. Recommend a separate HEAVY-tier plan that opens with an impact-mapper sweep + reads run_subcommand.rs precedent to plan the next handler family extraction.

DECISION-10 — Dashboard DTO HashMap -> BTreeMap migration DEFERRED (mentioned in DECISION-5). Judgment call; not mechanical. Follow-up patch-N candidate.

DRIFT FROM EXPLORATION
- Exploration claimed C-1 is 16 files; reality is 14 SHIPPED + 2 DEFERRED (dual_ledger + reasoning_template per DECISION-2). The mechanical-not-mechanical boundary moved during execution after per-file verification.
- Exploration's claim "task_schema.rs is mechanical 1-line swap" missed the 3-file cascade (DECISION-3). Cascade was absorbed; net result still mechanical (just 4 files instead of 1).

STUBS ADDED (workflow/sessions/s-22300/stubs.md)
- B15-entropy-burndown DEFERRED (270 SystemTime/Instant call sites; needs ClockProvider injection cascade)
- B15-unwrap-burndown DEFERRED (~700 prod sites estimate; per-site judgment)
- B15-cli-regrowth DEFERRED (cli/main.rs 252 -> 1269 LOC; needs module extraction plan)
- B15-dashboard-BTreeMap DEFERRED (UI determinism judgment; not mechanical)

VERIFICATION
Build status: NOT RUN by main LLM (cargo / svelte-check / cargo test all reserved for the auditor per HARD RULE). Contract test added at crates/contracts/tests/b15_tranche_c_burndown_contract.rs (4 tests, ~140 LOC).

REFS
- workflow/sessions/s-22300/context-package-tranche-C-B15.md (master-index residual counts)
- workflow/sessions/s-22300/exploration/tranche-C-B15-scoped.md (file-by-file plan)
- gotchas.md L866-870 (BTreeMap fence at C-45 -> C-46 SSB boundary)
- gotchas.md L1244-1246 (ClockProvider shipped, residual sites are migration not new build)
- gotchas.md L1385-1387 (pipeline.rs:813 HashMap nondeterminism is a BTreeMap candidate, NOT FxHashMap; kept out of C-1 scope)
