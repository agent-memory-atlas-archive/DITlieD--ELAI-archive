# C-28: Testing-Trust T-1 Mutation Testing
Objective: implement Tier T-1 Mutation Testing as specified in testing-trust-design.txt
T-1 task — MutationRunner (cargo-mutants + mutmut), Mutation ValidationTier variant,
MutationResult struct, MUTATION_SURVIVED SSB action type, EquivalentMutantCache,
AUDITING->MEMORIZING gate, elai_cc.toml config, MonitorWindow gauge.
Estimated effort: 2 sessions.
Pre-requisites: C-27 (T-0 WiringContract wiring; mutation is the next tier up in the
5-check deterministic gate chain).
Parallel-safe with: C-29 (T-2/T-3 intent + metamorphic tiers), C-21..C-26.

## Context

testing-trust-design.report.md (2026-05-17) classifies the mutation gate as PARTIAL:
ValidationTier enum stops at Stress (5 tiers); no MutationRunner, no MutationResult,
no EquivalentMutantCache, no MUTATION_SURVIVED SSB action type, no mutation_score gate
on the AUDITING->MEMORIZING transition. The DeterministicAuditor 5-check chain and the
AUDITING substate infrastructure are both present (deterministic_auditor.rs,
deterministic_audit_gate.rs) and are the direct extension point.

The report's action list item for T-1 is: "MutationRunner wrapping cargo-mutants (Rust)
+ mutmut (Python); add Mutation tier 3.5 to ValidationTier enum; add mutation_score gate
to AUDITING -> MEMORIZING transition." This C-plan implements exactly that scope.

Dependency versions verified via crates.io and PyPI (2026-05-17):
- cargo-mutants 27.0.0 (latest stable; install as dev-tool, not lib dependency)
- mutmut 3.5.0 (PyPI; install into sidecar venv via pyproject.toml dev deps)

SCHEMA_VERSION is currently 46 (B-37C). C-28 bumps to 47 for MUTATION_SURVIVED.

## Reads

- F:\.ELAI_workflow\docs\thefinalglance\verification\reports\guide\testing-trust-design.report.md
  (section "Mechanisms PARTIAL" -> Layer 4 / ValidationTier mutation gate)
- F:\.ELAI_workflow\docs\specs\guide\testing-trust-design.txt (Layer 4 full spec)
- .elai_cc/crates/core/src/tool_dispatcher.rs L175-232 (ValidationTier enum + ordered())
- .elai_cc/crates/orchestrator/src/deterministic_auditor.rs (5-check chain — insertion point)
- .elai_cc/crates/orchestrator/src/deterministic_audit_gate.rs (AUDITING gate — block hook)
- .elai_cc/crates/orchestrator/src/ssb.rs L184 (SCHEMA_VERSION 46 — bump to 47)
- .elai_cc/crates/tools/src/tiered_validation.rs (TieredValidator dispatch — Mutation slot)
- .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (gauge insertion point)
- .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (MutationGaugeInfo type addition)
- .elai_cc/elai_cc.toml ([testing.mutation] config section)

## Tasks

1. **Verify cargo-mutants install surface**
   Run `cargo mutants --version` in a tmp dir; confirm 27.0.0 responds. If absent,
   document install command: `cargo install cargo-mutants --version 27.0.0`. This is a
   CLI tool, NOT a Cargo.toml dependency — do not add it to [dependencies] or
   [dev-dependencies]. Record install confirmation in workflow/reasoning/C-28-decisions.md.

2. **ValidationTier::Mutation variant** in
   `.elai_cc/crates/core/src/tool_dispatcher.rs`
   Insert `Mutation` between `Unit` and `Integration` (tier 3.5 per spec).
   Update `as_str()` -> `"mutation"`, `timeout_ms()` -> `max_mutants * timeout_per_mutant`
   (runtime-computed; return sentinel `0` meaning "use MutationRunner timeout").
   Update `ordered()` -> `[Determinism, QualityAst, Unit, Mutation, Integration, Stress]`.
   Update every exhaustive match in tools/src/tiered_validation.rs that iterates
   ValidationTier::ordered(). Impact: grep elai.map for ValidationTier before editing.

3. **MutationResult + MutantOutcome types** in
   `.elai_cc/crates/orchestrator/src/mutation_testing.rs` (new file)
   ```
   pub enum MutantOutcome { Caught, Survived, Timeout, Skipped(String) }
   pub struct MutantRecord {
       pub file: PathBuf,
       pub line: u32,
       pub mutant_kind: String,   // e.g. "replace_+_with_-"
       pub outcome: MutantOutcome,
   }
   pub struct MutationResult {
       pub records: Vec<MutantRecord>,
       pub mutation_score_pct: f64,  // caught / (caught + survived + timeout) * 100
       pub total_run: u32,
       pub cache_hits: u32,
   }
   ```
   No .unwrap(). All subprocess errors propagate as MutationError (thiserror).

4. **MutationRunner trait + RustMutationRunner + PythonMutationRunner** in
   `.elai_cc/crates/orchestrator/src/mutation_testing.rs`
   ```
   pub trait MutationRunner: Send + Sync {
       async fn run(&self, cfg: &MutationConfig, project_dir: &Path,
                    cache: &EquivalentMutantCache) -> Result<MutationResult, MutationError>;
   }
   ```
   RustMutationRunner: shells to `cargo mutants --timeout <N> --output <tmp>
   --jobs 1 --file <files>`, parses JSON output (cargo-mutants --output writes
   caught.txt + missed.txt; parse both). Respects max_mutants_per_session cap by
   stopping after cap is reached (partial result, score computed over ran subset).
   PythonMutationRunner: shells to `mutmut run --paths-to-mutate <files>
   --runner "pytest -x -q"`, then `mutmut results` for JSON output.
   Both runners: tokio::process::Command (NOT std::process::Command — async context).
   Both runners: timeout enforced per-mutant via tokio::time::timeout.
   No .unwrap() / .expect() outside test cfg.

5. **EquivalentMutantCache** in `.elai_cc/crates/orchestrator/src/mutation_testing.rs`
   SQLite-backed (reuse existing Ssb connection pool pattern from ssb.rs).
   Key: blake3 hash of (pre-mutation file bytes + mutant_kind + line).
   Schema: table `equiv_mutant_cache (key TEXT PRIMARY KEY, outcome TEXT, ts INTEGER)`.
   On cache hit: MutantRecord { outcome: MutantOutcome::Skipped("cache_hit") }.
   On new outcome: insert row. TTL: 30 days (rows older than TTL are re-run).
   DB path: `<project_dir>/.elai/mutation_cache.sqlite3` (create if absent).

6. **MutationConfig** struct + elai_cc.toml wiring in
   `.elai_cc/elai_cc.toml`:
   ```toml
   [testing.mutation]
   enabled = true
   threshold_pct = 70.0
   timeout_per_mutant_sec = 30
   max_mutants_per_session = 100
   ```
   Load via `load_elai_config()` in `crates/cli/src/runtime_config.rs` — add
   `MutationConfig` field to `ElaiConfig`. Deserialize with serde defaults matching
   the above values so absent section = defaults (no hard error).

7. **MUTATION_SURVIVED SSB action type** in
   `.elai_cc/crates/orchestrator/src/ssb.rs`
   Bump SCHEMA_VERSION 46 -> 47. Add variant `MutationSurvived`. Add to:
   - `SsbActionType` enum (4 lockstep sites: enum / SQL CHECK / as_sql_str /
     ssb_action_type_from_str). Emit one row per surviving mutant with payload:
     `{ file, line, mutant_kind }`. No row emitted for Caught/Timeout/Skipped.
   SQL payload struct: `MutationSurvivedPayload { file: String, line: u32, mutant_kind: String }`.
   Update SCHEMA_VERSION comment block with C-28 entry.

8. **Wire MutationRunner into DeterministicAuditor pipeline** in
   `.elai_cc/crates/orchestrator/src/deterministic_auditor.rs`
   After the existing 5-check gate passes, if `MutationConfig.enabled` is true:
   - Call `RustMutationRunner` on changed Rust files, `PythonMutationRunner` on
     changed Python files (detect by extension on the file list from ProjectProfile).
   - If `mutation_score_pct < threshold_pct`: emit `DeterministicFinding { check:
     CheckKind::Mutation, severity: Block, message: "mutation_score N% < threshold M%" }`.
   - Emit one `SsbActionType::MutationSurvived` row per surviving mutant (zero rows
     if all caught). This preserves existing early-exit semantics: Mutation runs only
     if Build/VersionGate/FileSizeCensus/LazyCode/Completeness all pass first.
   Add `CheckKind::Mutation` variant to the `CheckKind` enum in deterministic_auditor.rs.

9. **MonitorWindow mutation_score_pct gauge** in
   `.elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte` and
   `.elai_cc/crates/dashboard-v2/src/lib/api/types.ts`
   types.ts: add `MutationGaugeInfo { score_pct: number; threshold_pct: number;
     cache_hits: number; total_run: number; last_updated_ms: number }` to types.ts.
   MonitorWindow.svelte: add `mutationGauge = $state<MutationGaugeInfo | null>(null)`.
   Render: gauge row shows `mutation N% (threshold M%)` in amber when score < threshold,
   green when score >= threshold, grey when null (no run yet). Polled every 5s from
   Tauri command `get_mutation_gauge` (add stub to dashboard-v2/src-tauri/src/lib.rs
   returning `Option<MutationGaugeInfo>`; reads from last MutationResult stored in
   session state). Stub returns null until C-28 backend is wired live.

10. **Contract tests** (two files):
    `.elai_cc/crates/orchestrator/tests/mutation_runner_contract.rs`
    - `test_rust_mutation_runner_schema_47`: assert SCHEMA_VERSION_PUBLIC == 47.
    - `test_mutation_result_score_calculation`: construct MutationResult with 7 caught,
      2 survived, 1 timeout; assert mutation_score_pct == 70.0 (7/(7+2+1)*100). No subprocess.
    - `test_equivalent_mutant_cache_skip`: open in-memory SQLite, insert a known key,
      call cache.get(); assert returns Skipped("cache_hit"). Insert same key again; assert
      no duplicate rows (upsert semantics).
    - `test_mutation_survived_ssb_payload_roundtrip`: serialize MutationSurvivedPayload
      { file: "src/math/sum.rs", line: 3, mutant_kind: "replace_+_with_-" } via
      serde_json; assert deserialize roundtrip equals original.
    `.elai_cc/crates/contracts/tests/mutation_t1_schema_contract.rs`
    - `test_schema_version_47`: assert SCHEMA_VERSION_PUBLIC == 47.
    - `test_mutation_survived_in_sql_check`: open in-memory SSB, write one
      MutationSurvived row, read back via ssb_action_type_from_str("mutation_survived");
      assert Ok(SsbActionType::MutationSurvived).
    - `test_validation_tier_mutation_position`: assert ValidationTier::ordered() has
      Mutation between Unit and Integration (index 3).

## WIRING_CONTRACT

produces_symbols:
  - `MutationRunner` trait + `RustMutationRunner` + `PythonMutationRunner`
  - `MutationResult` + `MutantRecord` + `MutantOutcome` + `MutationError`
  - `MutationConfig` (loaded from [testing.mutation] in elai_cc.toml)
  - `EquivalentMutantCache` (SQLite-backed, blake3-keyed)
  - `ValidationTier::Mutation` variant + `CheckKind::Mutation` variant
  - `SsbActionType::MutationSurvived` + `MutationSurvivedPayload`
  - `MutationGaugeInfo` TypeScript type + `get_mutation_gauge` Tauri command stub
  - SCHEMA_VERSION bumped to 47
consumes_symbols:
  - `DeterministicAuditor` 5-check chain (mutation is 6th check, runs after all pass)
  - `ValidationTier::ordered()` dispatch in `tiered_validation.rs`
  - `ProjectProfile.commands` (detect language to route Rust vs Python runner)
  - `ElaiConfig` in `runtime_config.rs` (MutationConfig field addition)
  - Existing SSB lockstep pattern (ssb.rs enum/CHECK/as_sql_str/from_str 4-site update)
reachable_from:
  - elai_cc.toml `[testing.mutation]`
  - AUDITING FSM substate (DeterministicAuditor caller)
  - MonitorWindow mutation_score_pct gauge
stub_ban:
  - RustMutationRunner and PythonMutationRunner MUST shell to real cargo-mutants /
    mutmut binaries; no mock subprocess output in production paths
  - EquivalentMutantCache MUST use blake3 for key hashing (not sha256, not md5)
  - AUDITING->MEMORIZING block MUST be enforced via DeterministicFinding::Block (not warn)
frontend_pair: MonitorWindow.svelte gauge row + get_mutation_gauge Tauri command stub
observable_effect:
  trigger: elai /task --mode cloud "implement sum function with 5 unit tests"
  expect_ssb: zero MutationSurvived rows when all mutants caught; AUDITING->MEMORIZING
              proceeds; mutation_score_pct >= 70
  expect_artifact: MonitorWindow shows green gauge; second identical run is faster
                   (EquivalentMutantCache hits reported in MutationResult.cache_hits)

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION is 47
rg "const SCHEMA_VERSION: i64 = 47" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. MutationRunner trait + two impls present
rg "trait MutationRunner|struct RustMutationRunner|struct PythonMutationRunner" \
    .elai_cc/crates/orchestrator/src/mutation_testing.rs

# 3. Mutation variant inserted between Unit and Integration
rg "Mutation" .elai_cc/crates/core/src/tool_dispatcher.rs

# 4. CheckKind::Mutation added to deterministic_auditor
rg "Mutation" .elai_cc/crates/orchestrator/src/deterministic_auditor.rs

# 5. MutationSurvived in SSB enum and from_str
rg "MutationSurvived|mutation_survived" .elai_cc/crates/orchestrator/src/ssb.rs

# 6. [testing.mutation] section present in elai_cc.toml
rg "testing.mutation" .elai_cc/elai_cc.toml

# 7. Contract tests compile and pass (no subprocess needed)
cargo test -p elai_orchestrator --test mutation_runner_contract 2>&1 | grep "test result"
cargo test -p elai_contracts --test mutation_t1_schema_contract 2>&1 | grep "test result"

# 8. No .unwrap() in production mutation_testing.rs (only test cfg allowed)
rg "\.unwrap\(\)" .elai_cc/crates/orchestrator/src/mutation_testing.rs
# Expected: zero matches (or only inside #[cfg(test)] blocks)

# 9. EquivalentMutantCache uses blake3
rg "blake3" .elai_cc/crates/orchestrator/src/mutation_testing.rs
```

## ADVERSARIAL_CHECK

Tautological tests scenario: agent submits a task whose tests contain only `assert!(true)`.
MutationRunner runs cargo-mutants; all arithmetic/boolean mutants survive (nothing caught).
mutation_score_pct = 0%. DeterministicFinding::Block emitted with message
"mutation_score 0% < threshold 70%". AUDITING->MEMORIZING transition is BLOCKED.
User sees in dashboard: MonitorWindow gauge amber at 0% with threshold 70% shown.
No surviving-mutant SSB rows are emitted for the tautological case because the
MutantOutcome is Survived — but MUTATION_SURVIVED rows ARE emitted (one per surviving
mutant), which is correct behavior. The gate check uses mutation_score_pct, not row count.

EquivalentMutantCache replay: if same task is rerun with same pre-mutation AST,
cache keys match; MutantRecord.outcome == Skipped("cache_hit"); cache_hits > 0 in
MutationResult; second run completes faster than first. Assert: total_run on second run
equals cache_hits (all mutants from cache).

## ACCEPTANCE

- [ ] cargo-mutants 27.0.0 install confirmed (CLI tool, not Cargo dep)
- [ ] mutmut 3.5.0 install confirmed in sidecar venv pyproject.toml dev deps
- [ ] ValidationTier::Mutation variant present between Unit and Integration
- [ ] ValidationTier::ordered() returns 6-element array with Mutation at index 3
- [ ] MutationRunner trait + RustMutationRunner + PythonMutationRunner in mutation_testing.rs
- [ ] MutationResult.mutation_score_pct formula: caught/(caught+survived+timeout)*100
- [ ] EquivalentMutantCache SQLite table `equiv_mutant_cache` with blake3 key column
- [ ] EquivalentMutantCache TTL: rows older than 30 days re-run
- [ ] SsbActionType::MutationSurvived present; SCHEMA_VERSION == 47
- [ ] Exactly one MUTATION_SURVIVED SSB row emitted per surviving mutant; zero rows when all caught
- [ ] [testing.mutation] section in elai_cc.toml with all 4 config keys
- [ ] MutationConfig loaded with defaults when section absent (no panic)
- [ ] DeterministicFinding::Block emitted when mutation_score_pct < threshold_pct
- [ ] AUDITING->MEMORIZING blocked when mutation score below threshold
- [ ] Mutation check runs AFTER all 5 existing deterministic checks pass (not before)
- [ ] MonitorWindow renders mutation_score_pct gauge (green/amber/grey states)
- [ ] get_mutation_gauge Tauri command stub returns null when no run yet
- [ ] No .unwrap()/.expect() in production paths of mutation_testing.rs
- [ ] Contract tests: mutation_runner_contract 4/4 pass; mutation_t1_schema_contract 3/3 pass
- [ ] Cloud smoke test: elai /task --mode cloud "implement and test sum function" ->
      MutationRunner ran on src/math/sum.rs; mutation_score_pct >= 70; SSB has zero
      MUTATION_SURVIVED rows; AUDITING->MEMORIZING proceeds
- [ ] Adversarial: tautological-test task -> MutationRunner catches many survivors;
      AUDITING->MEMORIZING BLOCKED; dashboard gauge shows amber at score < 70
- [ ] Cache replay: second identical run has cache_hits == total_run; completes faster

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/mutation_testing.rs
- ADD: .elai_cc/crates/orchestrator/tests/mutation_runner_contract.rs
- ADD: .elai_cc/crates/contracts/tests/mutation_t1_schema_contract.rs
- MODIFY: .elai_cc/crates/core/src/tool_dispatcher.rs (ValidationTier::Mutation + ordered())
- MODIFY: .elai_cc/crates/tools/src/tiered_validation.rs (Mutation arm in dispatch)
- MODIFY: .elai_cc/crates/orchestrator/src/deterministic_auditor.rs (Mutation check + CheckKind)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA_VERSION 47 + MutationSurvived)
- MODIFY: .elai_cc/crates/cli/src/runtime_config.rs (MutationConfig field in ElaiConfig)
- MODIFY: .elai_cc/elai_cc.toml ([testing.mutation] section)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/api/types.ts (MutationGaugeInfo)
- MODIFY: .elai_cc/crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (gauge row)
- MODIFY: .elai_cc/crates/dashboard-v2/src-tauri/src/lib.rs (get_mutation_gauge stub)
- CREATE: workflow/reasoning/C-28-decisions.md

## Next

C-29 (T-2/T-3 — intent formalization + metamorphic relation seed library), which builds
the PLANNING->TESTING_FIRST gate requiring >= 3 IntentPairs. C-28 and C-29 are parallel-safe.
