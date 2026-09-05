# B-34: Entry-Point Detection Tool
Objective: ship a deterministic entry-point detector — orphans-with-
outgoing-calls + conventional names per language + framework-route
heuristics — exposed as an orchestrator dispatcher tool so the Designer
agent grounds Recursive Decomposition in real entry points and the
Researcher's Oracle pipeline (/research) gets accurate "where does this
codebase actually start?" answers.
Estimated effort: 1 session (~5-7 hours).
Pre-requisites: B-29 (consumes the same SymbolGraphCache + per-file
payloads).
Parallel-safe with: B-30, B-31, B-32, B-33, B-35.

## Context

The Designer agent's Recursive Decomposition (4D.7-4D.10 — DONE) outputs
ConceptDecomposition → ConceptNode → PhaseNode → StepNode → BabyStepSpec.
TechStackDecision distinguishes RESEARCH vs EXISTING_PROJECT vs
USER_SPECIFIED vs FALLBACK sources. When DecisionSource is
EXISTING_PROJECT, the designer should ground the decomposition in the
actual structure of the target project — but today there is no tool that
answers "what are this project's entry points?". Designer hallucinates,
or relies on text matches in `main` / `index.ts` / `app.py` without
distinguishing real entry points from utility files named the same.

Researcher's Oracle pipeline (A-43 done) does deep web research with a
7-phase pipeline, but for project-internal context grounding it only has
file/symbol search — no equivalent of "find all framework routes" or
"find all CLI binaries". /research questions like "how does
authentication work in this project?" lack a starting set of files to
trace from.

SocratiCode's three-heuristic union
(`src/services/graph-entrypoints.ts:22-139`, 140 LOC) solves this
deterministically with no LLM: orphans-with-outgoing-calls + conventional
names per language + framework router-call detection + test-name
patterns. Each detected entry carries a `reason` string ("orphan" /
"well-known-name:main" / "framework:http-get" / "test") so the LLM
consumer sees not just the entry point but why it's classified as one.

This B-step ports the algorithm to ELAI's per-file payload contract
(B-29) and exposes it as a dispatcher tool. Small, self-contained, high
leverage for Designer + Researcher.

## Reads (mandatory before coding)

- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\services\graph-entrypoints.ts
  (FULL — 140 LOC; the algorithm is here, port verbatim)
- F:\.ELAI_workflow\docs\.source\SocratiCode-main\SocratiCode-main\src\constants.ts
  lines 151-167 (ENTRY_POINT_NAMES per language — 14 languages with
  conventional `main`/`Main`/`__main__` style)
- F:\.ELAI_workflow\.elai_cc\crates\knowledge\src\symbol_graph\
  (B-29 module — entry detection consumes SymbolGraphFilePayload + DKG
  file_graph)
- F:\.ELAI_workflow\.elai_cc\crates\orchestrator\src\research_tree.rs
  (Designer's research_decompose path — find the 4D.7-4D.10 entry point)
- F:\.ELAI_workflow\docs\specs\guide\update-implementation-order-summary.txt
  A-43 (Oracle Research Pipeline) — confirms /research consumer existence

## Tasks

1. Add `ENTRY_POINT_NAMES` constant table in
   `crates/knowledge/src/symbol_graph/entry_points.rs`:
   ```rust
   use std::collections::BTreeMap;

   pub fn entry_point_names_per_language() -> BTreeMap<&'static str, &'static [&'static str]> {
       let mut m = BTreeMap::new();
       m.insert("javascript", &["main"][..]);
       m.insert("typescript", &["main"][..]);
       m.insert("python",     &["main"][..]); // also __main__ via separate check
       m.insert("go",         &["main"][..]);
       m.insert("rust",       &["main"][..]);
       m.insert("java",       &["main"][..]);
       m.insert("kotlin",     &["main"][..]);
       m.insert("scala",      &["main"][..]);
       m.insert("c",          &["main"][..]);
       m.insert("cpp",        &["main"][..]);
       m.insert("csharp",     &["Main"][..]);
       m.insert("swift",      &["main"][..]);
       m.insert("ruby",       &["main"][..]);
       m.insert("php",        &["main"][..]);
       m
   }
   ```
   Mirror SocratiCode's table verbatim (14 languages); add `&["__main__"]`
   special-case for Python in the matcher itself.

2. Define types in
   `crates/knowledge/src/symbol_graph/entry_points/types.rs`:
   ```rust
   pub struct EntryPoint {
       pub id: String,                 // SymbolNode.id or file path for orphan
       pub name: String,               // qualified name or basename
       pub file: String,               // relative path
       pub line: Option<u32>,
       pub reason: EntryPointReason,
   }

   pub enum EntryPointReason {
       Orphan,
       WellKnownName(String),          // e.g. "main", "Main", "__main__"
       FrameworkRoute(String),         // e.g. "framework:http-get",
                                       //      "framework:route"
       Test,                           // names matching test_/Test/it/describe
   }
   ```

3. Implement `detect_entry_points` in
   `crates/knowledge/src/symbol_graph/entry_points/detect.rs`:
   ```rust
   pub fn detect_entry_points(
       file_graph: &CodeGraph,
       payloads: &[SymbolGraphFilePayload],
   ) -> Vec<EntryPoint> { ... }
   ```
   Mirror SocratiCode's `detectEntryPoints` algorithm
   (`graph-entrypoints.ts:22-86`):
   - **Heuristic 1 — Orphans with outgoing calls:**
     - Iterate `file_graph.nodes`; skip files where `dependents.is_empty()`
       returns false (i.e. only consider files NOTHING imports)
     - Cross-reference `payloads` for that file; only include if the file
       has at least one outgoing call (otherwise it's a library file no
       one uses)
     - Emit `EntryPoint { id: relative_path, name: relative_path,
       file: relative_path, line: None, reason: Orphan }`
   - **Heuristic 2 — Conventional names:**
     - For each payload, look up `entry_point_names_per_language()` for
       the file's language
     - For each symbol where `name != "<module>"` and the name is in the
       conventional set (case-sensitive — `Main` for csharp, `main` for
       others; `__main__` matched separately for Python via
       `s.name == "__main__"`), emit
       `EntryPoint { id: sym.id, name: qualified, file, line: Some,
       reason: WellKnownName(name) }`
   - **Heuristic 3 — Framework routes:**
     - For each payload's `outgoing_calls`, examine `callee_name`. Map
       lowercased name to reason:
       - `get`, `post`, `put`, `delete`, `patch`, `head`, `options`,
         `all`, `use` → `FrameworkRoute(format!("framework:http-{}", lower))`
       - `route` → `FrameworkRoute("framework:route")`
     - For each match, find the enclosing symbol via
       `find_symbol_at(payloads_symbols, call_site.line)` (line-range
       containment) and emit `EntryPoint` with that enclosing symbol's
       id.
     - Mirror SocratiCode's `framework` helper (`graph-entrypoints.ts:108-115`).
   - **Heuristic 4 — Test functions:**
     - For each symbol where `name != "<module>"` and the name matches:
       - `^test[_A-Z]` (Python convention `test_foo`, JVM `testFoo`)
       - `^Test[A-Z_]` (Go `TestFoo`, Java `TestFoo`)
       - exactly `it` or `describe` (Mocha/Jest BDD)
     - Emit `EntryPoint { ..., reason: Test }`.

   - **Deduplication:** key on `(id, reason)` — same symbol can be both
     "orphan" and "framework:get". Use `BTreeSet<(String, String)>`
     visited set (mirror SocratiCode's `seen` set).

4. Add `find_symbol_at(symbols: &[SymbolNode], line: u32) -> Option<&SymbolNode>`
   helper:
   - Sort symbols by `line` ascending (do once outside loop).
   - Linear scan for symbols where `line <= target_line <= end_line`.
   - Return deepest enclosing (highest start `line`) — mirrors
     SocratiCode's `findSymbolAt` (`graph-entrypoints.ts:96-104`).

5. Expose as orchestrator dispatcher tool in
   `crates/orchestrator/src/dispatcher.rs`:
   ```rust
   "entry_points" => {
       let project_id = args.project_id;
       let cache = get_symbol_graph_cache(project_id).await?;
       let file_graph = load_file_graph(project_id).await?;
       // Materialize payloads for files that look like potential entries
       // (orphans + files with conventional symbol names) — DON'T load
       // ALL payloads; that defeats B-29's lazy-loading invariant.
       let payloads = collect_candidate_payloads(&cache, &file_graph).await?;
       let entries = detect_entry_points(&file_graph, &payloads);
       Ok(json!({ "entries": entries }))
   }
   ```
   Performance budget: <500ms on a 10K-file project. Achieved by:
   - Loading orphan files' payloads only (orphans are typically <5% of
     files).
   - Loading payloads for files whose name matches conventional entry
     filenames (`main.*`, `index.*`, `__main__.py`, `Main.kt` etc.) as
     a fast filter.
   - Files without conventional names AND not orphans skip payload load.

6. Auto-inject for Designer's Recursive Decomposition:
   - In `crates/orchestrator/src/research_tree.rs`, when
     `decision_source == TechStackDecision::ExistingProject`, call
     `entry_points` tool and inject results into the Designer's
     context as `EXISTING_ENTRY_POINTS` block.
   - Designer prompt: "The target project has these entry points: [list
     with reasons]. Ground your decomposition in these — every Phase
     should map to exactly one entry point group, every Step to a code
     path traced from one entry point."

7. Auto-inject for Researcher's Oracle pipeline:
   - In `/research` command handler (cli/main.rs), pre-query
     `entry_points` and pass results to the Oracle as `PROJECT_STARTING_POINTS`
     in the formulation phase. Oracle's tree expansion uses these as
     seed nodes.

8. Add `EntryPointsDetected` SSB action_type with payload
   `{ project_id, count, by_reason: BTreeMap<String, u32> }` for one-shot
   logging when Designer or Researcher invokes the tool.

## WIRING_CONTRACT

produces_symbols:
  - knowledge::symbol_graph::entry_points::detect::detect_entry_points
  - knowledge::symbol_graph::entry_points::detect::find_symbol_at
  - knowledge::symbol_graph::entry_points::types::{EntryPoint,
    EntryPointReason}
  - knowledge::symbol_graph::entry_points::names::entry_point_names_per_language
  - core::ssb::SsbActionType::EntryPointsDetected (new variant)
  - dispatcher tool entry: `entry_points`
consumes_symbols:
  - orchestrator::research_tree::research_decompose → uses
    `entry_points` tool when decision_source == ExistingProject, injects
    EXISTING_ENTRY_POINTS into Designer briefing
  - cli::main::handle_research_command → uses `entry_points` tool to
    seed Oracle's project-context formulation
  - dashboard::commands::list_entry_points → Tauri wrapper for UI display
reachable_from:
  - cli::main::handle_research_command
  - cli::main::handle_dispatcher_call (`entry_points` tool name)
  - orchestrator::task_runner::run_designer_briefing (auto-injection
    site)
stub_ban:
  - detect_entry_points returns Vec<EntryPoint> (not Result<Vec, Error>)
    — empty vec is the "no entries detected" signal, not an error
  - dispatch tool budget: <500ms p99 on 10K-file project (perf test)
  - dedup keyed on (id, reason) — duplicate entries with same reason
    impossible
frontend_pair: B-29 (consumes SymbolGraphCache); B-33 (Decomposition
  mode highlights entry-point nodes)
observable_effect:
  trigger: "elai.bat repl '/research how does authentication work'" on a
    fixture web project
  expect_ssb: EntryPointsDetected action with by_reason populated (at
    least framework:http-get, framework:http-post, test entries)
  expect_artifact: Oracle research output starts from real entry-point
    files, not random matches

## DETERMINISTIC_CHECK

```bash
# 1. Tool returns expected entries for a known fixture
elai.bat repl "/entry_points --project docs/fixtures/multilang_project"
# Expected stdout: lists main (rust), main (go), Main (csharp),
# index.ts (orphan), test_* functions, framework:http-get for express
# routes (if fixture has them).

# 2. Performance budget on real ELAI repo
time elai.bat repl "/entry_points"
# Expected: <500ms wall.

# 3. Designer auto-injection works
# Run a task that triggers ExistingProject path, e.g.:
elai.bat repl "/task 'add a new feature to docs/fixtures/multilang_project'"
# Wait for Designer briefing to be written, then:
grep "EXISTING_ENTRY_POINTS" workflow/sessions/*/designer-briefing.log
# Expected: at least one entry per heuristic.

# 4. Researcher auto-injection works
elai.bat repl "/research how is HTTP routing handled"
# In /research output, the formulation phase should reference at least
# one framework route detected by entry_points.

# 5. Dedup verified
# In a project where main.rs is also an orphan (no other file imports
# it) AND has the conventional name "main", verify only ONE entry per
# (id, reason); the same symbol may appear twice with reason=Orphan and
# reason=WellKnownName but never twice with the same reason.

# 6. Performance — payload selectivity
# On a 10K-file project, count payloads loaded by entry_points:
ELAI_DEBUG_PAYLOAD_LOAD=1 elai.bat repl "/entry_points"
# Expected log: "loaded N payloads for entry detection (out of 10000)"
# with N << 1000 (orphans + conventional names < 10% of files).
```

## ACCEPTANCE

- [ ] `detect_entry_points` implemented with all four heuristics (Orphan,
      WellKnownName, FrameworkRoute, Test)
- [ ] `ENTRY_POINT_NAMES` table covers all 14 languages from SocratiCode +
      Python `__main__` special case
- [ ] `find_symbol_at` returns deepest enclosing symbol per line
- [ ] Dispatcher tool `entry_points` registered, callable from CLI and
      IPC
- [ ] Designer's research_decompose auto-injects EXISTING_ENTRY_POINTS
      when DecisionSource is ExistingProject
- [ ] Researcher's /research command auto-injects
      PROJECT_STARTING_POINTS in Oracle formulation phase
- [ ] EntryPointsDetected SSB action emitted per call
- [ ] Performance: <500ms p99 on 10K-file project, <100ms on ELAI repo
- [ ] Payload selectivity: only orphan files + files matching
      conventional filenames are payload-loaded (not all files)
- [ ] Dedup: same symbol with same reason never appears twice
- [ ] Zero `.unwrap()` in non-test source per CLAUDE.md hard rule
- [ ] No new external crates without confirmation

## Tests to add

- `crates/knowledge/tests/entry_points_orphan.rs`:
  - fixture with 3 files: A imported by B; B imports A only; C imports
    nothing. Assert: A is NOT orphan (has dependents), B is orphan iff
    has outgoing calls, C is orphan only if has outgoing calls.
- `crates/knowledge/tests/entry_points_well_known.rs`:
  - per-language fixture with `main` / `Main` / `__main__` symbols.
    Assert one EntryPoint per match with correct WellKnownName reason.
  - test case-sensitivity: `Main` in Rust file (lowercase expected) is
    NOT an entry; `Main` in C# IS.
- `crates/knowledge/tests/entry_points_framework.rs`:
  - fixture with `app.get("/route", handler)`, `router.post(...)`,
    `Route::get(...)` — assert FrameworkRoute reasons emitted with
    correct enclosing-symbol attribution.
- `crates/knowledge/tests/entry_points_test_names.rs`:
  - symbols `test_foo`, `Test_bar`, `it`, `describe`, `regular_func`.
    Assert first 4 detected as Test, last not.
- `crates/knowledge/tests/entry_points_dedup.rs`:
  - symbol that triggers both Orphan and WellKnownName. Assert two
    EntryPoint entries with different reasons (NOT one combined entry,
    NOT duplicates).
- `crates/knowledge/tests/entry_points_performance.rs`:
  - synthetic 10K-file fixture. Time detect_entry_points;
    assert wall < 500ms (skip on CI machines too slow; run locally).
- `crates/knowledge/tests/entry_points_payload_selectivity.rs`:
  - mock SymbolGraphCache that records load_payload calls; assert
    fewer than 10% of file count is loaded.
- `tests/contracts/entry_points_dispatcher_contract.rs` — dispatcher
  tool registered, JSON shape stable.

## Files touched

- CREATE: `crates/knowledge/src/symbol_graph/entry_points/mod.rs`
- CREATE: `crates/knowledge/src/symbol_graph/entry_points/types.rs`
- CREATE: `crates/knowledge/src/symbol_graph/entry_points/names.rs`
- CREATE: `crates/knowledge/src/symbol_graph/entry_points/detect.rs`
- MODIFY: `crates/knowledge/src/symbol_graph/mod.rs` (export
  entry_points module)
- MODIFY: `crates/orchestrator/src/dispatcher.rs` (`entry_points` tool
  entry)
- MODIFY: `crates/orchestrator/src/research_tree.rs` (auto-inject for
  ExistingProject decomposition)
- MODIFY: `crates/orchestrator/src/task_runner.rs` (Designer briefing
  EXISTING_ENTRY_POINTS block)
- MODIFY: `crates/cli/src/main.rs` (`/research` Oracle pre-seeding;
  add `/entry_points` REPL slash command)
- MODIFY: `crates/dashboard-v2/src-tauri/src/lib.rs` (V2 — was V1
  `crates/dashboard/src/commands.rs`; add `list_entry_points` Tauri
  command. B-36 wires it into stub.ts. Feeds into B-33 GraphWindow
  decomposition mode highlight or — if Option B/C from B-33 V2 REBASELINE
  is chosen — directly into the orbit Mode 2 MAP rendering)
- MODIFY: `crates/core/src/ssb/action_types.rs` (EntryPointsDetected
  variant)
- CREATE: `workflow/reasoning/B-34-decisions.md`

## Decision doc

Write `workflow/reasoning/B-34-decisions.md` documenting:
- Which framework patterns to recognize (HTTP verbs + `route` is
  SocratiCode's set; ELAI may want to add `subscribe`/`on` for event
  emitters, `register` for plugin systems — discuss but default to
  parity with SocratiCode v1)
- Whether ENTRY_POINT_NAMES should be configurable per project via
  `[entry_points]` block in `elai_cc.toml` (e.g. project may use
  `setup` instead of `main` as convention). Recommend yes — append-only
  override with built-in defaults.
- Test-name regex: SocratiCode uses `^test[_A-Z]` and `^Test[A-Z_]` —
  document why these specific anchors (avoid false positives like
  `testimony`, `Testament`)
- Payload selectivity strategy: pre-load orphan payloads + payloads
  whose filename matches `entry_point_filenames` set (`main.*`,
  `index.*`, `__main__.py`, etc.). The fast-filter set should be
  documented.
- Whether Test reason should be opt-in (`include_tests: bool` parameter
  on tool). Researcher cares about tests as starting points; Designer
  usually doesn't. Default to including; let the consumer filter.

## Side benefits unlocked

- Designer's existing-project decomposition becomes accurate instead of
  hallucinated
- Researcher's /research questions like "how does X work" start from
  real entry points
- Dashboard GraphWindow decomposition mode (B-33) can highlight entry
  points with a special node style
- B-25 Empire integration benefits — Empire's brains have known entry
  points (FactoryLine handlers); detection finds them automatically
- Self-modification (B-23) Layer 1A could grow new
  ENTRY_POINT_NAMES based on observed patterns (configurable expansion)
- /entry_points slash command lets users explore "where does this
  codebase start" interactively

## Next

B-29 (Symbol-Level Impact API) — required dependency.
B-33 (Interactive Graph Viewer) — consumes entry points for highlight.
B-35 (Context Artifacts Pipeline) — independent; both feed Designer
  context.
