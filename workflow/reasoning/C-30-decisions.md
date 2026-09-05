C-30 Decisions
==============

Session: s-25672
Plan: docs/thefinalglance/C-plan/C-30_surgical_oracle_mode.md
Context package: workflow/sessions/s-25672/context-package-C30.md

Atomic SCHEMA bump: 61 -> 62. Five new SSB variants land in one patch
(9-tuple lockstep: SCHEMA_VERSION, CHECK constraint, SsbActionType enum,
as_sql_str arms, ssb_action_type_from_str arms, SsbPayload variant,
action_type() match arm, test_payload arm, all_test_payloads list).
all_test_payloads().len() goes 126 -> 131.

Decisions
---------

DECISION-1 (DRIFT-1 resolved): `parse_task_flags` lives at
crates/cli/src/commands.rs:2414, not :2006. Already destructures
`--surgical` + `--new` and returns them in `SlashCommand::Task`. C-30
consumed the existing flag bits directly; no change to parse_task_flags.

DECISION-2 (DRIFT-2 resolved): `run_research_pipeline` is called at
crates/cli/src/live_cli.rs:2578 inside the
`SlashCommand::Research { question } | SlashCommand::Ask { question }` arm
(the /research handler block starts ~L2666). Cache lookup + insert both
wired in this arm. C-30 emits `oracle_cache_hit` before the pipeline call
on a fresh-TTL hit, and `oracle_cache_miss` after a successful pipeline
return.

DECISION-3 (DRIFT-3 resolved): the surgical branch INTERCEPTS in
live_cli.rs:2632 BEFORE the `DefaultWebResearchRunner` is constructed. The
default ConceptDecomposition pipeline is skipped when `surgical=true` and
the SurgicalDecomposer output replaces it. When the DKG is unavailable or
returns no matches, the plan is still emitted (Simple, 0 LOC, 0 steps) so
the dashboard observes the surgical-mode selection. Wiring through
task_runner.rs (a deeper second option) deferred — the live_cli intercept
covers the same observable_effect with less surface area.

DECISION-4 (DRIFT-4 resolved): `build_tool_catalog` lives at
crates/orchestrator/src/production_agent_rpc.rs:232 (was advertised as
:207 in the plan). Doc comment updated: "The catalog has ten tools" ->
"The catalog has fourteen tools". The four DKG tool IDs already existed
in `crates/ipc-types/src/lib.rs` (TOOL_DKG_CALLERS / TOOL_DKG_DEPENDENTS /
TOOL_DKG_DATA_FLOW / TOOL_DKG_SUBGRAPH at L487-490) — C-30 wired them
into the catalog vec.

DECISION-5 (DRIFT-5 resolved): extended the `DkgQuerier` trait
(crates/rules/src/predicate.rs) with four new methods —
`query_callers`, `query_dependents`, `query_data_flow`, `query_subgraph` —
each with a DEFAULT IMPL that returns empty. `EmptyDkgQuerier` +
`HotSwappableDkgQuerier` + all in-process test stubs compile unchanged.
`DkgQuerierAdapter` overrides all four with real SQLite-backed BFS
traversals (cycle-guarded with `HashSet<i64>`, depth-capped at
`MAX_C30_DEPTH = 8`). Added three new types to the trait surface:
`DkgNodeRef`, `DkgEdgeRef`, `DkgDataFlowEdgeRef`, `DkgSubgraphSnapshot`.

DECISION-6 (DRIFT-6 resolved): SSB SCHEMA bumped 61 -> 62 with the
canonical 9-tuple atomic edits. all_test_payloads count assertion bumped
126 -> 131. CHECK constraint extended with 5 new strings. Contract test
`c30_surgical_oracle_contract.rs::project_detection_signal_round_trips`
guards the round-trip path for ProjectDetectionSignal explicitly; the
in-crate `test_typed_payloads` covers all five new variants implicitly via
the 131-entry inventory.

DECISION-7 (DRIFT-7 resolved): `ExecutionMode` is at
`crates/orchestrator/src/context_budget.rs:865` (variants `Interactive` /
`Autonomous`). `SurgicalDecomposer::decompose` accepts `mode:
ExecutionMode` directly via this import path. No move/relocation of the
enum was needed.

DECISION-8 (DRIFT-8 resolved): `ResearchAnswer` field is `confidence`
(NOT `answer_confidence`). `oracle_cache.rs::source_tier_for_confidence`
and the live_cli cache-miss arm both read `answer.confidence`. The
TaskRunner's separate "answer_confidence" usage (if any) is unaffected.

DECISION-9 (DRIFT-9 + DRIFT-11 resolved): `AnswerConfidence::Unknown` is
mapped to `SourceTier::T4Rejected` by
`source_tier_for_confidence`, which yields `ttl_for_tier = 0` -> never
cached. The `OracleAnswerCache::insert` early-returns `false` on
`ttl_secs == 0` so T4 / Unknown answers never enter the cache. T1/T2/T3
TTLs match spec: 30d/7d/1d.

DECISION-10 (DRIFT-10 resolved): the TOML `[research.vram_tiers.<tier>]`
section mirrors the existing `ResearchTierConfig` struct field names
verbatim — `model`, `quant`, `context_ceiling`, `research_context_budget`,
`chunk_size`, `breadth_factor`, `depth_limit`, `phi_min`, `psi_min`,
`embedding_device`, `max_sources_per_node`, `max_concurrent_fetches`,
`speculative_depth`. The plan's alternative field-name list
(max_search_results / max_depth / max_breadth / max_token_budget /
enable_speculative) was rejected because the existing struct has no such
fields and a future loader needs to deserialize the existing struct.
DETERMINISTIC_CHECK #4 passes: 6 tier entries present (vram_8gb,
vram_12gb, vram_16gb, vram_24gb, vram_48gb, cloud).

DECISION-11 (DRIFT-11 resolved): EscalationPolicy gains exactly two new
LOC-gate fields:
  - `autonomous_confirm_above_loc: u32` (default 500, mirrors
    SURGICAL_LOC_WARN_THRESHOLD)
  - `autonomous_refuse_above_loc: u32` (default 1000, mirrors
    SURGICAL_LOC_ESCALATE_THRESHOLD)
The existing 3-field struct grows to 5 fields. The contract test
`escalation_policy_carries_loc_gates` pins both the defaults and the
constants alignment.

DECISION-12 (DRIFT-12 resolved): edge_type filter normalisation is
done by listing both case variants in the SQL `IN(...)` clause inside
`DkgQuerierAdapter::query_callers` / `query_dependents`:
`'CALLS', 'calls'`, `'IMPORTS', 'imports', 'USES', 'uses'`. No DKG
schema change required. The `SurgicalDecomposer` path uses the same
double-cased `edge_types` vec when calling `query_reachability`.

DECISION-13 (DRIFT-13 resolved): `SourceTier` variant names
(T1Authoritative / T2Strong / T3Weak / T4Rejected) are EXACT match in
crates/core/src/source_tier.rs. The plan's variant references were
correct; no adapter shim needed. `source_tier_str` provides the
lowercase form for SSB payload serialization.

DECISION-14 (DRIFT-14 resolved): `dkg_nodes` has no per-node `loc`
column (verified at crates/knowledge/src/dkg/mod.rs L25-38). The
SurgicalDecomposer computes LOC as `end_line - start_line + 1` via
direct SQL queries against the production DKG db path. When the db is
not available (no `dkg_db_path` argument, or open-with-flags failure),
`total_loc_estimate` falls back to 0 — no LOC gate fires, and the
caller (live_cli) emits the canonical `surgical_plan_created` row with
loc=0.

DECISION-15 (DRIFT-15 resolved): VramTier loader is OUT OF SCOPE for
C-30. The [research.vram_tiers] section is presence-only; the runtime
source of truth remains the hardcoded `research_config_for_tier`
function in crates/orchestrator/src/research_tier.rs. A future loader
that wires the TOML data into runtime is a separate plan.

Additional implementation notes
-------------------------------

NOTE-A (live_cli LiveCli struct): added two session-scoped fields
`oracle_cache: Option<Arc<OracleAnswerCache>>` and
`project_detection_done: bool`. Both default to `None` / `false` at all
three LiveCli construction sites. Oracle cache is lazily initialised on
the first `/research` invocation in the session. project_detection_done
flips `true` after the first `/task` runs the signal scan (regardless of
indexes_exist).

NOTE-B (live_cli imports): no new imports needed in live_cli.rs — all
new types resolve via existing wildcard `use elai_orchestrator::*` and
direct paths like `elai_orchestrator::surgical_decomposer::*` and
`elai_orchestrator::oracle_cache::*`. `DkgQuerierAdapter`, `Path`, and
`PathBuf` were already imported.

NOTE-C (contract test parent path): the
`vram_tiers_toml_has_five_entries` test resolves elai_cc.toml from
`CARGO_MANIFEST_DIR.parent().parent()` because the contracts crate lives
at .elai_cc/crates/contracts/ and elai_cc.toml lives at .elai_cc/.

NOTE-D (LOC gate vs. live_cli surgical branch): the live_cli surgical
branch DEFAULTS to Interactive mode and `confirmed=false` unless the
caller passed `--new` (treated as a best-effort opt-in signal). The LOC
gate inside `SurgicalDecomposer::decompose` will return
`ConfirmationRequired` for blast > 500 LOC and the branch prints a
human-readable prompt + emits `surgical_loc_warning` SSB. Wiring the
gate to a true stdin Y/n confirmation prompt is deferred.

NOTE-E (no new dependencies): blake3 is already in workspace deps
(via elai-core + elai-orchestrator). No new external crate added.

NOTE-F (test counts): 17/17 C-30 contract tests PASS. 10/10
surgical_decomposer unit tests PASS. 10/10 oracle_cache unit tests
PASS. 29/29 ssb unit tests PASS (round-trip + 131-payload count
assertion). 14/14 C-2 contract tests still PASS (no regression).
525/525 elai-rules unit tests PASS (trait extension is backward
compatible). `cargo check --workspace` clean.

Files added
-----------

- .elai_cc/crates/orchestrator/src/surgical_decomposer.rs (new)
- .elai_cc/crates/orchestrator/src/oracle_cache.rs (new)
- .elai_cc/crates/contracts/tests/c30_surgical_oracle_contract.rs (new)
- workflow/reasoning/C-30-decisions.md (this file)

Files modified
--------------

- .elai_cc/crates/orchestrator/src/ssb.rs (SCHEMA bump + 5 new variants
  + 5 new payload structs + atomic 9-tuple updates)
- .elai_cc/crates/orchestrator/src/lib.rs (register surgical_decomposer +
  oracle_cache modules)
- .elai_cc/crates/orchestrator/src/autonomous_gate.rs (EscalationPolicy
  gains 2 LOC-gate fields + matching defaults)
- .elai_cc/crates/orchestrator/src/production_agent_rpc.rs (doc comment
  ten -> fourteen tools; 4 new ToolDefinition entries; 4 new wire ID
  imports)
- .elai_cc/crates/rules/src/predicate.rs (DkgQuerier trait gains 4 new
  methods with default impls; 4 new struct types)
- .elai_cc/crates/knowledge/src/dkg_adapter.rs (DkgQuerierAdapter
  overrides the 4 new trait methods with real SQLite BFS; 4 internal
  helpers for cycle-guarded traversal)
- .elai_cc/crates/cli/src/live_cli.rs (LiveCli struct gains 2 fields,
  initialised in 3 constructors; surgical branch intercepts /task BEFORE
  DefaultWebResearchRunner; Oracle cache wired into /research; project
  detection signal scan runs once on first /task in a session)
- .elai_cc/elai_cc.toml ([research.vram_tiers.*] section with 6 tiers)
