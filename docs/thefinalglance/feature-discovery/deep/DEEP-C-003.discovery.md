# DEEP-C-003 deep feature discovery

## Scope, evidence notation, and processing proof

This ledger is a normalization of the ten `numbered/index original` sources assigned to manifest chunk `DEEP-C-003`. Repeated task, wiring-contract, deterministic-check, acceptance, and file-list statements are represented once as a single atomic record and cross-classified rather than counted again. Source anchors are one-based physical/logical line numbers in the immutable files whose hashes are recorded below.

- **E** — explicit in the original source.
- **I** — inference from two or more explicit statements; not itself promised.
- **A** — underspecified or ambiguous in the original.
- **C** — two explicit statements conflict.
- **DELTA+** — material detail absent from `03-C-SUMMARY_INDEX.discovery.md`.
- **DELTA!** — correction to that orientation summary.

The summary index was used only to orient the deep read and to identify deltas. No claim below is promoted to **E** solely from the summary.

### Exact processed manifest and verification

Manifest section: `DEEP-SOURCE-MANIFEST.md` lines 386-401. All values were independently recomputed with `wc -l -c` and `sha256sum`; every row matched.

| # | Exact absolute path | Lines | Bytes | SHA-256 | Verification |
|---:|---|---:|---:|---|---|
| 1 | `docs/thefinalglance/C-plan/C-3_tauri_commands_knowledge_surface.md` | 270 | 14,092 | `86d8423136fbb574f47360c399e940ab2e1ea6213d90b1638515e41e05ab3989` | exact match |
| 2 | `docs/thefinalglance/C-plan/C-7_dashboard_ux_completeness.md` | 238 | 15,464 | `1c360b38a1360fae66c7b6e0e852d2c7bc2c3dce8ba4ed15334cacecb879dc9a` | exact match |
| 3 | `docs/thefinalglance/C-plan/C-10_anti_spiral_primitives.md` | 323 | 18,055 | `3b771f327a2b494502a2bb4b78a29c4aff8edeeb6534b981bb090513e3e18023` | exact match |
| 4 | `docs/thefinalglance/C-plan/C-11_sandbox_completeness.md` | 257 | 14,067 | `21de47602b05c8e2d7f7f5b7a656b056aecd6d055bdcc78cb0799fae92513445` | exact match |
| 5 | `docs/thefinalglance/C-plan/C-14_phase0_debt_dead_code_strip.md` | 364 | 20,302 | `8924390ecd517fe1cc5af0d4f042bc73f10b98e2c74e23665f3da600aec0414a` | exact match |
| 6 | `docs/thefinalglance/C-plan/C-20_test_infra_backfill_metrics.md` | 314 | 18,888 | `616c65fec4fc18a753580277a1be34802d9904d384e4e3a683142450143122d5` | exact match |
| 7 | `docs/thefinalglance/C-plan/C-29_testing_trust_t2_t3_intent_metamorphic.md` | 297 | 17,417 | `a48ec359ca59b19112f892e3543111fd9432d7965b1eb266b213ff46e2c03cf2` | exact match |
| 8 | `docs/thefinalglance/C-plan/C-45_cantante_contrastive_credit_attribution.md` | 149 | 6,874 | `bb840830ec7ea3ce082bee5a1dc04965965ad127c01f40e50244a2dd025c8566` | exact match |
| 9 | `docs/thefinalglance/C-plan/C-48_safeharbor_memory_guardrail.md` | 146 | 6,363 | `16ecc5b9415144f0d872075eb7c28c85a4b26600f4c7c1b165628c12e538275f` | exact match |
| 10 | `docs/thefinalglance/C-plan/C-59_cross_config_benchmark_master_report.md` | 194 | 8,854 | `db01ce1db99412d565e2477c1231de7e2777ef5f02c879cd3fc4c68f6c6909b2` | exact match |
| **Total** | **10 files** | **2,552** | **140,376** | — | **manifest total exact** |

## Per-file explicit feature ledgers

### C-3 — Tauri Commands: Knowledge / Reasoning Dashboard Surface

Source status: implementation plan, estimated 1-2 sessions; C-1 must ship first because it produces `query_trace` and `csq_evaluated`; parallel-safe with C-4 because their command/file scopes are asserted disjoint (lines 1-8).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C3-001 | capability | Replace hardcoded zero, empty-array, and stub returns in the knowledge/reasoning Tauri layer with live SSB or DB reads; add six wholly missing commands. | L1-L4 |
| E-C3-002 | existing-state | `AppState.writer` is `Option<Arc<Ssb>>`; reads are intended to use `AppState.viewer: Option<Arc<SsbViewer>>`, with no new writer trait. | L40-L42 |
| E-C3-003 | schema | Existing frontend `QueryTrace` fields are `id`, `query`, `mode`, `stages[]`, `total_ms`, `ts`, and `hit_count`. | L44-L45 |
| E-C3-004 | data/workflow | Add `SsbViewer::recent_query_traces(limit: u32) -> Result<Vec<QueryTraceRow>, SsbViewerError>`; select `action_type='query_trace'`, `ORDER BY seq DESC`, limit the result, and decode payloads with `rmp_serde`. | L70-L74 |
| E-C3-005 | surface/fallback | Replace `get_query_trace_recent`'s unconditional `json!([])` with a viewer call and default to an empty array on missing viewer/read failure; set `unwired.recentTraces=false`. | L76-L80 |
| E-C3-006 | data/fallback | `SsbViewer::pagerank_top` queries `dkg_nodes_pagerank(symbol,path,score)`, orders descending by score, and returns an empty vector rather than an error when the table is absent. | L82-L87 |
| E-C3-007 | surface/default | `get_pagerank_top(n: u32)` defaults `n` to 20, calls the viewer, and is registered in `lib.rs`. | L89-L91 |
| E-C3-008 | data/state | `query_tier_stats` aggregates the latest 500 `query_trace` rows by `QueryTrace.mode` (`fast`, `standard`, `deep`), including average `total_ms` and `sample_size`; no rows means the zero/unknown sentinel `sample_size=0`. | L93-L98 |
| E-C3-009 | data/fallback | `freshness_state` reads the newest `freshness_meta`/`freshness_computed` SSB source if C-1 defines it, otherwise falls back to `ConfigManager` values `decay_rate` and `access_boost`; `known=true` only for a real source. | L100-L104 |
| E-C3-010 | surface | Add and register `get_freshness_state`, delegating to `SsbViewer::freshness_state()`. | L106-L107 |
| E-C3-011 | event/surface/fallback | Add/register `get_canary_drift`; read the newest `action_type='canary_drift'` row and return `{delta_ms,baseline_ms,current_ms,drift_pct,ts}`, or `{known:false}` if absent. | L109-L114 |
| E-C3-012 | event/surface/fallback | Add/register `get_csq_log`; return the latest 50 `csq_evaluated` rows as `{csq_id,conclusion,guard_present,ts_ms}`, with `[]` as the no-row result. | L116-L119 |
| E-C3-013 | event/surface/fallback | Add/register `get_feasibility_log`, distinct from `get_feasibility_report`; return the latest 20 `feasibility_check` rows as `{task_id,verdict,score,ts_ms}`, with `[]` when absent. | L121-L124 |
| E-C3-014 | config/surface/fallback | Add/register `get_agent_grc_config`; read the `AgentGrcConfig` table through SSB DB or config store and return `{role,max_calls,max_chars,topic_prefix}` rows; use compiled defaults if no runtime override exists. | L126-L130 |
| E-C3-015 | surface/lifecycle | Add/register `get_research_budget`, wrapping the existing viewer method; keep `known:false` until a `research_budget_updated` row exists, then update the viewer to consume it. | L132-L137 |
| E-C3-016 | surface | Add/register `get_source_tier_distribution` by wrapping the existing viewer method. | L139-L141 |
| E-C3-017 | integration | The live consumer path is `KnowledgeDrawer.svelte` and `ReasoningDrawer.svelte` polling through `drawers.svelte.ts`. | L161-L169 |
| E-C3-018 | constraint | After C-3, `pagerank_top` may not remain an unconditional empty-vector stub; after C-1, `get_query_trace_recent` may not remain an unconditional empty array. | L164-L166 |
| E-C3-019 | failure | Missing canary data must yield `{known:false}` and never panic; all eight new/updated commands must be registered. | L167-L168 |
| E-C3-020 | observability | Cloud task `search the codebase for foo and explain` must produce query traces; the Tauri query returns at least one trace and `unwired.recentTraces=false`. | L170-L173 |
| E-C3-021 | acceptance | Live trace acceptance additionally requires non-null `stage_latencies` matching the frontend interface. | L201-L206 |
| E-C3-022 | acceptance | PageRank acceptance requires a positive scored row when the table has data and an empty non-error array when empty. | L207-L208 |
| E-C3-023 | acceptance | Freshness accepts `known:false` only when no freshness source is readable; canary accepts the same sentinel only when no row exists. | L209-L213 |
| E-C3-024 | acceptance | GRC acceptance requires at least eight named-role entries; budget/tier-distribution may remain `known:false` pending C-1 producers. | L214-L219 |
| E-C3-025 | quality gate | `cargo check --workspace` must pass, all eight handlers must be registered, and new handlers/viewer methods may not use `.unwrap()`. | L220-L224 |
| E-C3-026 | test | `test_get_query_trace_recent_empty`: no SSB rows returns `[]`. | L238-L241 |
| E-C3-027 | test | `test_get_query_trace_recent_live`: a synthetic row round-trips with expected fields. | L240-L241 |
| E-C3-028 | test | `test_get_pagerank_top_empty_table`: absent table returns `[]` without panic. | L242-L243 |
| E-C3-029 | test | `test_get_pagerank_top_with_rows`: three rows preserve descending-score order. | L244-L245 |
| E-C3-030 | test | `test_get_freshness_state_stub`: absent source returns `known:false`. | L246 |
| E-C3-031 | test | `test_get_canary_drift_no_rows`: absent rows return a `{known:false}` object. | L247 |
| E-C3-032 | test | `test_get_query_tier_stats_no_rows`: absent rows return `sample_size:0`. | L248 |
| E-C3-033 | test | `test_get_csq_log_empty` and `test_get_csq_log_with_rows` cover `[]` plus the exact four-field result shape. | L250-L253 |
| E-C3-034 | test | `test_get_feasibility_log_empty` requires a non-error empty array. | L254 |
| E-C3-035 | test | `test_get_agent_grc_config_returns_eight_roles` requires length at least eight. | L255 |
| E-C3-036 | non-goal | C-1 owns writes for `query_trace`, `csq_evaluated`, `research_budget`, and `source_tier_distribution`. | L257-L260, L268-L269 |
| E-C3-037 | deferred | Per-stage latency-zero fixes, `NoopSsbNotifier` replacement, and canary `embed("fn main()")` correction remain B-4 work. | L261-L263 |
| E-C3-038 | deferred | `ts-rs` generation and `MonitorWindow` feature-gating remain B-3 work; `CsqBlockCard` remains B-5 work. | L264-L266 |
| E-C3-039 | deferred | Real `get_grounding_sweep` waits for B-8, and `ELAI_DASHBOARD_OFFLINE` gating remains B-36 work. | L267, L270 |

Explicit anomalies:

- **C-C3-01:** The source says seven viewer methods are stubbed but enumerates only five (`pagerank_top`, `query_tier_stats`, `freshness_state`, `research_budget_snapshot`, `source_tier_distribution`). `get_query_trace_recent` is separately a command stub, not a sixth/seventh viewer method (L12-L24, L35-L38).
- **C-C3-02:** The objective says six wholly missing commands, while the implementation/registration set contains eight commands. Six are explicitly called “missing entirely”; `get_pagerank_top` and `get_freshness_state` are also added but omitted from that six-command inventory (L2-L4, L26-L33, L89-L114, L150-L152).
- **A-C3-01:** `recent_query_traces` is declared with `limit: u32`, but the suggested caller casts `n as usize`; the intended final parameter type is unresolved (L71-L79).
- **A-C3-02:** Freshness action type is conditional (“if C-1 defines it”), and GRC ownership is a three-way DB/config/compiled-default choice without precedence beyond “runtime override first” (L100-L104, L126-L130).
- **I-C3-01:** `.unwrap_or_default()` collapses absent viewer and read/deserialize failure into the same empty result, so this suggested implementation loses failure provenance (L76-L80).

Summary deltas:

- **DELTA! C3-1:** The summary’s “seven stubs, six visibly named” is not the original’s structure: the original names five viewer stubs and separately one Tauri command stub; the seventh remains unidentified.
- **DELTA+ C3-2:** The summary omits all response schemas, limits (500/50/20), missing-table and no-row fallbacks, GRC default path, exact contract tests, and deferred/non-goal list captured in E-C3-003 through E-C3-039.
- **DELTA+ C3-3:** The original conditionally depends on C-1 defining freshness/canary/research-budget row details rather than treating every producer schema as settled.

### C-7 — Dashboard UX Completeness

Source status: implementation plan, estimated 1-2 sessions; depends on C-6 visual tokens and C-2 edit events; asserted parallel-safe with C-5 and C-1 through C-4 (lines 1-7).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C7-001 | surface | Add `'explore'` to `WindowType`, `VALID_WINDOW_TYPES`, and `SIDEBAR_WINDOWS`; register `ExploreWindow.svelte`; bind `Ctrl+8` via `sidebar-8`. | L40-L47, L109-L123 |
| E-C7-002 | capability/stub | `ExploreWindow` renders a complete Q&A UI (input, submit, scrollable responses), logs `stub.logStub('explore.query',{query})`, and appends a placeholder response; only live backend search remains intentionally stubbed and marked by `UnwiredNote`. | L44-L48 |
| E-C7-003 | state/lifecycle | `closedTabs` is a maximum-20 stack of `{type: WindowType,title:String,closedAt:number}`; `closeTab()` records `Date.now()` before removal; overflow uses FIFO eviction. | L50-L54, L202 |
| E-C7-004 | state/surface | Export `closedTabs`, `showRecoveryModal`, and `openWindowRecovery()`; the function sets the modal flag. | L50-L55 |
| E-C7-005 | surface/workflow | `RecoveryModal.svelte` lists real closures most-recent first with type, title, relative time, and `Restore`; restore calls `workspace.openTab('primary',entry.type)`; mount beside `CommandPalette` in `AppShell`. | L56-L60 |
| E-C7-006 | shortcut/recovery | Keep shortcut id/combo/description for `recover-closed` but change `Ctrl+Shift+H` from command palette to `workspace.openWindowRecovery()`. | L61-L64 |
| E-C7-007 | shortcut/security | `Ctrl+Shift+A` and `Ctrl+Shift+R` call `reviewStore.acceptAllHunks()`/`rejectAllHunks()` only when the primary active tab is `review`; no global accept/reject. | L66-L70, L135-L139 |
| E-C7-008 | state | Create a minimal Svelte 5 `review.svelte.ts` store exposing accept-all and reject-all methods for `ReviewWindow` subscription. | L66-L70 |
| E-C7-009 | surface/state | Every `kind==='hunk'` separator gets `Accept`, `Decline`, and `Retry`; accept/decline toggle `hunkState[hunkContent]`, and retry exposes an inline note using `toggleRetryNote`. | L72-L76 |
| E-C7-010 | integration | Replace static `FILES` with a derived `reviewStore.files`, remove `UnwiredNote`, invoke `get_file_changes({session_id})` on mount, and invoke `accept_change`, `decline_change`, or `retry_audit` from hunk/file actions. | L78-L83 |
| E-C7-011 | failure/state | An empty `get_file_changes` result renders `<Empty>` rather than fixture data; `EXECUTING` with no changes yet renders a loading spinner. | L83-L84 |
| E-C7-012 | responsive/config | At `max-width:999px`, hide `nav-list`, show/collapse to a hamburger icon strip, and manage hamburger state with `Sidebar.svelte` `ResizeObserver`. | L86-L92 |
| E-C7-013 | responsive/state | At `max-width:799px`, `Workspace.svelte` uses `ResizeObserver` to set `secondaryCollapsed=true`, giving the primary panel full width. | L89-L92 |
| E-C7-014 | test | Playwright snapshot (a) loads the dashboard and renders `ProjectSelector`. | L94-L100 |
| E-C7-015 | test | Snapshot (b) opens and renders `ExploreWindow` with `Ctrl+8`. | L94-L100 |
| E-C7-016 | test | Snapshot (c) renders `ReviewWindow` with hunk controls visible. | L94-L100 |
| E-C7-017 | test | Snapshot (d) closes a window and opens `RecoveryModal` with `Ctrl+Shift+H`. | L94-L100 |
| E-C7-018 | test | Snapshot (e) verifies sidebar collapse at an 800px viewport; baselines use `page.screenshot` and later regression uses `toHaveScreenshot()`. | L94-L100 |
| E-C7-019 | dependency/config | Add Playwright only after checking whether it is already a dev dependency. | L98-L100 |
| E-C7-020 | conditional/deferred | Add `Ctrl+Shift+W` only if a `shell`/`tier_0` window type exists; otherwise add only the exact deferral comment and no shortcut code. | L102-L105 |
| E-C7-021 | integration | Review consumes C-2 `FileWrite`/`DispatchEditHunks` rows and four Tauri commands attributed by this plan to C-3. | L124-L129 |
| E-C7-022 | observability | Cloud task `modify hello.rs to add a goodbye function` must show live hunks, action controls, applied state on the next query/poll, Explore, recovery, responsive collapse, and written baselines. | L144-L151 |
| E-C7-023 | constraint | Recovery must use real `closedTabs`; Review must not retain static `FILES`; Explore may stub only backend search; review-wide shortcuts must remain review-scoped. | L135-L139 |
| E-C7-024 | acceptance | Restore reopens a window through `openTab`; accept/reject/retry invoke live handlers; refresh reflects applied state. | L199-L216 |
| E-C7-025 | quality gate | Five snapshots, `svelte-check`, and `cargo check --workspace` must pass. | L214-L218 |
| E-C7-026 | artifact set | Adds `review.svelte.ts`, `ExploreWindow.svelte`, `RecoveryModal.svelte`, `visual.spec.ts`, and `workflow/reasoning/C-7-decisions.md`; modifies the named API/store/window/shell/sidebar/workspace/CSS files. | L220-L234 |

Explicit anomalies:

- **C-C7-01:** C-7 says C-3 supplies `get_file_changes`, `accept_change`, `decline_change`, and `retry_audit`, but the complete C-3 source defines none of them and asserts its file scope is disjoint from C-4 (C-7 L78-L82/L127; C-3 L1-L8/L68-L141).
- **A-C7-01:** Restore records a title but calls `openTab` with only the type; removal of a restored entry, duplicate handling, recovery-stack persistence, and modal-close behavior are not defined (L50-L60).
- **A-C7-02:** `reviewStore` subscription/event semantics, polling cadence after the on-mount fetch, hunk identity collisions from `hunkContent`, and retry-note payload shape are not specified (L66-L84, L145-L151).
- **A-C7-03:** At exactly 800px, the `max-width:799px` single-panel rule does not apply even though the visual test is described as sidebar collapse at 800px; that test exercises the 999px rule only (L86-L100).
- **I-C7-01:** `Date.now()` makes closure timestamps runtime-variable; no deterministic clock or persistence contract is supplied (L50-L54).

Summary deltas:

- **DELTA+ C7-1:** The summary omits that Explore deliberately remains backend-stubbed, the exact stack schema/cap/FIFO rule, most-recent ordering, empty/loading states, all four live invokes, and conditional `Ctrl+Shift+W` behavior.
- **DELTA! C7-2:** The summary presents live review wiring without flagging that its alleged C-3 commands do not exist in the C-3 source.
- **DELTA+ C7-3:** Exact breakpoint inclusivity is `<=999px` and `<=799px`, not merely conceptual “below 1000/800”; the snapshot at 800px tests hamburger collapse, not the single-panel threshold.

### C-10 — Phase 4G.9 Anti-Spiral Primitives

Source status: implementation plan, estimated two sessions; depends on C-9 retry/escalation infrastructure and C-2's five named SSB variants; asserted parallel-safe with C-11 through C-15 (lines 1-12).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C10-001 | capability | Add the Rust anti-spiral runtime, wire it into autonomous decisions/step boundaries, emit events, create five persona agents, and surface stagnation in the orchestration dashboard. | L1-L7 |
| E-C10-002 | schema | `StagnationPattern = Spinning | Oscillation | NoDrift | DiminishingReturns`, deriving `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize`. | L42-L47 |
| E-C10-003 | schema | `AttemptResult` fields are `attempt_index:u32`, `error_class:String`, `output_hash:u64`, `error_count:u32`, and `timestamp_ms:u64`. | L48-L49 |
| E-C10-004 | data/lifecycle | `ExecutionHistory` is a stdlib `VecDeque<AttemptResult>` ring of capacity 32 with `push`, `is_full`, `last_n`, and `clear`; pushing past capacity evicts. | L50-L51, L163 |
| E-C10-005 | defaults | `StagnationDetection` defaults: `spin_threshold=3`, `oscillation_window=4`, `diminishing_min_window=4`, `diminishing_min_improvement_pct=5.0f32`. | L52-L54 |
| E-C10-006 | workflow/precedence | Detection order is `NoDrift` then `Spinning` then `Oscillation` then `DiminishingReturns`; fewer than two history entries returns `None`. | L54-L58 |
| E-C10-007 | detection | `NoDrift` is detected when the final two output hashes are equal. | L54-L55 |
| E-C10-008 | detection | `Spinning` is detected when the last threshold-count error classes are identical. | L55-L56 |
| E-C10-009 | detection | `Oscillation` is detected when the final four entries alternate between two strings. | L56 |
| E-C10-010 | detection | `DiminishingReturns` is detected when error-count improvement is below 5% per step across a four-attempt window. | L56-L57 |
| E-C10-011 | schema | `ThinkingPersona = Hacker | Researcher | Simplifier | Architect | Contrarian` with the same derives as `StagnationPattern`. | L59-L60 |
| E-C10-012 | config/data | `PersonaStrategy` contains a persona and `&'static str` prompt prefix; `PERSONA_STRATEGIES` is a five-entry static table of short imperative prefixes. | L61-L63 |
| E-C10-013 | deterministic routing | `suggest_persona_for_pattern`: Spinning→Contrarian, Oscillation→Architect, NoDrift→Hacker, DiminishingReturns→Simplifier. | L63-L65 |
| E-C10-014 | fallback | Researcher is reserved as a fallback rotation when `index % 5 == 4`. | L66 |
| E-C10-015 | defaults | `GoalDriftCanary` thresholds are green `0.7f32` and yellow `0.4f32`; verdicts are `Green | Yellow | Red`. | L67-L71 |
| E-C10-016 | normalization | Drift tokenization whitespace-splits, strips non-alphanumeric characters, lowercases, and returns `Vec<String>`. | L68-L70 |
| E-C10-017 | determinism | Drift similarity is Jaccard over `BTreeSet`, expressly not `HashSet`. | L69-L70 |
| E-C10-018 | schema | `WatchdogTimeoutKind = Idle | NoMaterialProgress | Safety`. | L72 |
| E-C10-019 | defaults/state | Per-task `GenerationWatchdog` defaults: idle 120,000ms, no-progress 600,000ms, safety 1,800,000ms, and `workflow_fingerprint:Option<u64>`. | L73-L74, L106-L111 |
| E-C10-020 | fingerprint | Fingerprint is blake3 over the 16-byte concatenation of `ssb_row_count:u64` and `output_hash:u64`, reduced to the first eight bytes as `u64`; absence of the crate is a hard error. | L75-L77 |
| E-C10-021 | watchdog/precedence | `check(last_event_ms,now_ms,current_fingerprint,start_ms)` returns at most one kind in precedence `Safety > NoMaterialProgress > Idle` and updates the stored fingerprint after checking. | L78-L80 |
| E-C10-022 | data | `AdaptiveFutilityTracker` stores per-error-class success and attempt counts in `HashMap<String,u32>` and exposes `record`, `success_rate`, and `is_futile`. | L81-L85 |
| E-C10-023 | fallback/default | Unseen error classes get a 0.5 prior; futility means success rate below the supplied threshold after at least four attempts; a trigger serializes tracker evidence to SSB. | L82-L85 |
| E-C10-024 | state/integration | Extend `GateContext` with empty-default `execution_history`, zero-default `attempt_index`, and `suggested_prompt_prefix:Option<&'static str>` defaulting to `None`. | L87-L90 |
| E-C10-025 | event/workflow | In `decide()`, detect after the existing `EscalationTrigger::AntiSpiral` check; emit a spinning/oscillation stagnation row, use a payload pattern for NoDrift/DiminishingReturns, emit `LateralThinkingActivated`, and attach the selected prefix. | L91-L97 |
| E-C10-026 | event/workflow/privacy | At each completed baby step, compare original task tokens with current subtask; red drift emits `GoalDriftWarning` with score plus the first 80 characters of original/current text. | L99-L104 |
| E-C10-027 | failure/recovery/event | Each heartbeat checks the watchdog; timeout emits kind/elapsed/fingerprint-hex and sets `AutonomyDecision::Escalate(EscalationTrigger::AntiSpiral)` using `tokio::time::Instant`. | L106-L111 |
| E-C10-028 | failure/recovery | Before repair retries, a futile error class at threshold 0.15 skips remaining retries and escalates `CircuitBreaker`, including serialized tracker detail in an SSB row. | L113-L118 |
| E-C10-029 | agent/config | All five persona files have about 30 lines, front matter (`name`, `description`, `model:sonnet`, tools `[Read,Grep,Glob,Bash,WebSearch]`), and instructions. | L120-L122 |
| E-C10-030 | agent behavior | Hacker proposes three shortest `DIRTY/CLEAN/HYBRID` paths and one concrete baby step; Researcher performs three web searches and outputs the best reference/next action. | L123-L126 |
| E-C10-031 | agent behavior | Simplifier reduces to a minimal skeleton and restores layers until failure, naming the bad layer; Architect maps the oscillating-error dependency graph and emits a decoupling step. | L127-L130 |
| E-C10-032 | agent behavior | Contrarian reverses the assumption, derives an alternative baby step, and does not hedge. | L131-L132 |
| E-C10-033 | surface/event | Add `get_stagnation_events`, returning newest-first rows for `stagnation_spinning_detected`, `stagnation_oscillation_detected`, `lateral_thinking_activated`, `goal_drift_warning`, and `watchdog_timeout`. | L134-L143 |
| E-C10-034 | dashboard | Collapsed `StagnationSection` shows timestamp, pattern/persona, detail, and existing-token badges: amber stagnation, blue lateral, orange drift, red watchdog. | L134-L143 |
| E-C10-035 | config/metrics | Add `[stagnation]` with enabled=true, counters initially zero, `spin_threshold=3`, and `adaptive_futility_threshold=0.15`. | L174-L178 |
| E-C10-036 | test | Extended-contract tests cover spinning threshold, identical-hash NoDrift, alternating-pair oscillation, sub-5% diminishing return, and single-entry `None`. | L145-L151 |
| E-C10-037 | test | Extended-contract tests cover all four pattern→persona affinities. | L152-L155 |
| E-C10-038 | test | Drift tests cover green above .7, red below .4, and punctuation stripping. | L156-L158 |
| E-C10-039 | test | Watchdog tests cover safety ceiling, unchanged-fingerprint no-progress, and no-event idle. | L159-L161 |
| E-C10-040 | test | Futility and ring-buffer tests cover below-threshold futility and capacity eviction. | L162-L163 |
| E-C10-041 | test/integration | Five integration tests require spinning emit after three errors, lateral emit with persona, attached prefix, red-drift emit, and watchdog emit plus escalation. | L165-L172 |
| E-C10-042 | quality gate | Exactly 17 extended plus five integration tests are planned; existing autonomous-gate (at least 14) and phase-4G (at least 18) suites must not regress. | L145-L172, L257-L262, L286-L287 |
| E-C10-043 | observability | Autonomous impossible-bug smoke expects iteration-3 spinning, persona activation and next-prompt prefix, drift below .4 if applicable, and no-progress termination at ten minutes. | L218-L227, L291-L302 |
| E-C10-044 | constraint | No `.unwrap()` in new paths; BTreeSet and blake3 are mandatory; futility evidence cannot be silently dropped; persona front matter must match the existing format. | L211-L216 |
| E-C10-045 | artifact set | Add the anti-spiral module, two contract files, five agent files, and decision doc; modify gate, task runner, phase metrics, Tauri commands, and drawer. | L304-L319 |

Explicit anomalies:

- **C-C10-01:** Drift determinism bans `HashSet`, but the production futility tracker explicitly uses std `HashMap`, whose serialized iteration order is not constrained (L69-L70, L81-L85, L211-L215).
- **A-C10-01:** “Researcher fallback rotation (`index % 5 == 4`)” defines neither the index nor when fallback runs; no `StagnationPattern` maps to Researcher (L63-L66).
- **A-C10-02:** Boundary semantics for exactly 0.7/0.4, empty-set Jaccard, diminishing-return percentage (including zero denominator), oscillation string field, and equal-hash precedence over spinning are not further specified (L52-L71).
- **A-C10-03:** Byte endianness for fingerprint input/output, exact no-progress comparison semantics, and reconciliation of millisecond arguments with `tokio::time::Instant` are absent (L73-L80, L106-L111).
- **A-C10-04:** Only two stagnation event variants exist for four patterns; the source says payload disambiguates the other two but does not say which variant carries each (L91-L96).
- **A-C10-05:** The SSB action type carrying `AdaptiveFutilityTracker` detail is unnamed, and the fate of a timed-out task beyond “sets Escalate/terminates” is not defined (L113-L118, L218-L225).

Summary deltas:

- **DELTA+ C10-1:** Adds the complete four-pattern enum, ordered detector rules, full schemas, all timeout values, exact persona affinities/instructions, futility prior/minimum/threshold, event query list, badge map, and 22-test coverage.
- **DELTA+ C10-2:** The summary omits precedence (`NoDrift` first; `Safety > NoProgress > Idle`) and fallback behavior (skip retries→CircuitBreaker versus watchdog→AntiSpiral).
- **DELTA! C10-3:** The summary's “11 types” is correct only for the acceptance enumeration; the module also contains the `PERSONA_STRATEGIES` static and multiple functions. Researcher is not selected by any explicit pattern mapping.

### C-11 — Sandbox Completeness (Tier C Isolation)

Source status: safety-critical implementation plan, estimated 2-3 sessions; depends on C-9 for multi-agent smoke execution; merge requires explicit user review (lines 1-9, 245-252).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C11-001 | security | In Linux `clone_cb`, apply capability bounding-set drops, five resource limits, `/proc` masking, and existing seccomp, with seccomp required last. | L1-L5, L30-L31, L47-L67 |
| E-C11-002 | limit/failure | `RLIMIT_CPU` soft/hard = 60/90 seconds; `RLIMIT_NOFILE=256`, `RLIMIT_NPROC=32`, `RLIMIT_AS=512MB`, `RLIMIT_FSIZE=100MB` (last four soft=hard); any setrlimit failure `_exit(124)`. | L49-L54 |
| E-C11-003 | implementation constraint | Rlimits use stack `libc::rlimit` plus named constants; no heap allocation or magic numbers. | L49-L54 |
| E-C11-004 | isolation/failure | Mount `proc` on `/proc` with `MS_NOSUID|MS_NOEXEC|MS_NODEV` and `hidepid=2` in the private `CLONE_NEWNS` child; every mount failure, including `EPERM`, `_exit(123)`. | L56-L60 |
| E-C11-005 | test fallback | Unprivileged CI is expected to hit `EPERM`; existing `CLONE_NEWNS::need_cap` guards skip those tests. | L58-L60 |
| E-C11-006 | capability/failure | After `PR_SET_NO_NEW_PRIVS`, loop capability 0 through `CAP_LAST_CAP` (40 for 5.15+), call `prctl(PR_CAPBSET_DROP,...)`, continue on `EINVAL`, and `_exit(122)` on any other error. | L62-L66 |
| E-C11-007 | compatibility | Use libc's `PR_CAPBSET_DROP` when present or verified literal fallback 24 when absent. | L63-L67 |
| E-C11-008 | integration/observability | `run_agent_session` gains `Option<Arc<dyn IpcHookDispatcher>>`; call `on_frame_routed(&frame,&route_result)` after routing and before writing the response. | L68-L74 |
| E-C11-009 | production wiring | `ProductionAgentRpcServer` supplies `Some(Arc::new(IpcHookDispatcher::default()))`; dependency-cycle absence must be verified first, and production may not feature-disable the hook. | L70-L79, L215-L216 |
| E-C11-010 | Linux test | Extend `test_4a_isolation_pid_namespace_real`: read `/proc/{child_pid}/status`, parse `NSpid`, and assert the innermost PID is 1 under Linux/EPERM guards. | L82-L85 |
| E-C11-011 | Linux test | Read `/proc/{child_pid}/limits`, parse `Max address space`, and assert hard limit `536870912`. | L87-L88 |
| E-C11-012 | Windows test/security | Under `cfg(windows)`, call `GetTokenInformation(TokenAppContainerSid)` on the spawned process token/handle and require a non-null AppContainer SID. | L90-L93 |
| E-C11-013 | adversarial/security | `test_4a_adv_ipc_u32_max_length_rejected`: `u32::MAX` length yields `IpcFrameError::PayloadTooLarge` before allocation. | L97-L100 |
| E-C11-014 | adversarial/security | `test_4a_adv_selfmod_path_traversal`: `../metrics.rs` through `IpcGateway::route_frame` yields `SelfModViolation`. | L101-L104 |
| E-C11-015 | adversarial/security | `test_4a_adv_selfmod_bash_write_locked`: a Bash write to a locked path is blocked before dispatch. | L106-L107 |
| E-C11-016 | adversarial/security | `test_4a_adv_selfmod_git_commit_locked`: a `git commit` targeting a locked path is blocked. | L109-L110 |
| E-C11-017 | adversarial/security/fallback | `test_4a_adv_selfmod_symlink_resolved`: a symlink redirect to a locked path is realpath-blocked; use platform-specific symlink APIs and skip on permission error. | L112-L116 |
| E-C11-018 | metrics/evidence | After smoke, `[runtime_measurements_b18b]` records rlimit/hidepid verification, cap-drop count, dispatcher wiring=true, adversarial count=5, NSpid/Windows results, and `completion=true`. | L118-L123 |
| E-C11-019 | decision/evidence | Decision doc records 512MB over the earlier 2GB, child-side hidepid rather than parent `nsenter`, and cap-drop→rlimit→seccomp ordering. | L125-L128 |
| E-C11-020 | observability/security | Linux smoke expects namespace evidence, `/etc/passwd` `EACCES`, no `SANDBOX_VIOLATION`, and visible 512MB address-space hard limit. | L157-L164 |
| E-C11-021 | observability/security | Windows smoke expects an AppContainer SID and task completion without escape. | L165-L167 |
| E-C11-022 | quality gate | Existing 22 adversarial tests plus five additions yield at least 27 passing; isolation tests and workspace check must pass. | L168-L169, L195-L206, L221-L224 |
| E-C11-023 | constraint | Rlimits must be real OS calls, hook callback must execute in the live frame loop, NSpid test must read `/proc`, and adversarial tests must exercise real routing. | L152-L156 |
| E-C11-024 | permission/review | Explicit review is required before merge for clone/fork syscall ordering, tasks 8-13, symlink creation, and raw/undocumented Win32 handle/API paths; PR description must carry the specified safety-critical flag. | L227, L245-L252 |
| E-C11-025 | artifact set | Modify `sandbox.rs`, `ipc_accept.rs`, `production_agent_rpc.rs`, both integration-test files, phase-4A metrics; add `C-11-decisions.md`. | L229-L243 |
| E-C11-026 | done gate | C-11 closes only after end-to-end Tier-C proof on Linux and Windows; the stated next step is C-12/prioritization. | L254-L257 |

Explicit anomalies:

- **C-C11-01:** The summary supplies the total order cap-drop→rlimit→hidepid→seccomp, but the original only fixes cap-drop→rlimit→seccomp; hidepid is merely “before execvp” and is absent from both explicit ordering statements (L30-L31, L56-L67, L125-L128).
- **C-C11-02:** The context identifies a missing Windows test “via `NtQueryInformationProcess`,” the task specifies `GetTokenInformation`, and the safety gate names both APIs; no single required API path is consistent (L24-L28, L90-L93, L245-L250).
- **A-C11-01:** Smoke says `NSpid != 1`, while the actual test requires the innermost PID to equal 1; it is unclear whether smoke means the outer PID, the full multi-value field, or a different check (L82-L85, L157-L164, L217-L222).
- **A-C11-02:** The plan attributes `/etc/passwd` `EACCES` to filesystem masking, but this patch's new mount only masks `/proc`; the pre-existing mechanism expected to deny `/etc/passwd` is not identified (L56-L60, L157-L164).
- **A-C11-03:** Production behavior on an unprivileged hidepid mount is a hard child exit, while only tests are allowed to skip; deployment capability/fallback policy is unstated (L56-L60).
- **A-C11-04:** `cap_drop_count` semantics (attempted, successful, or supported caps), hook callback failure behavior, and whether hook dispatch is sync/async are unspecified (L62-L79, L120-L123).

Summary deltas:

- **DELTA+ C11-1:** Adds exact exit codes 122/123/124, mount flags, cap-loop compatibility rule, hook timing/signature, five adversarial identifiers/expectations, measurement fields, and all review targets.
- **DELTA! C11-2:** The summary's exact hidepid placement is not supported by the original, and its unqualified Windows wording masks the `NtQueryInformationProcess`/`GetTokenInformation` inconsistency.
- **DELTA+ C11-3:** The original explicitly makes merge authority a user safety gate; test skipping is limited to capability/permission conditions and is not a production fallback.

### C-14 — Phase 0 Debt Part 1, re-scoped after live-caller audit

Source status: partially executed and explicitly re-scoped on 2026-05-17. The rewrite at lines 1-60 and annotated tasks/acceptance supersede the preserved original plan at lines 61 onward wherever they differ.

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C14-001 | status/correction | Pre-execution session `s-32952` found the draft based on stale evidence: `compact.rs`, `oauth.rs`, and `prompt.rs` are live production code with 40+ aggregate call sites, so direct deletion would break shipped behavior. | L1-L12 |
| E-C14-002 | live capability | `compact.rs` powers `/compact`, `ConversationRuntime::compact`, and `CompactionConfig` imports across eight CLI files. | L14-L16 |
| E-C14-003 | live capability | `oauth.rs` powers interactive `/login`/`/logout`, doctor login diagnostics, and two contract suites. | L17-L19 |
| E-C14-004 | live capability | `prompt.rs` provides `SystemPromptBuilder` for every conversation turn, REPL boot, auth/doctor, and `chat_reflect_contract`. | L20-L22 |
| E-C14-005 | deferred/status | Original deletion tasks 1-3, deletion metrics task 7, and absence-test task 8 are deferred to a separate multi-session refactor. | L24-L29 |
| E-C14-006 | executed migration | The four banned pricing identifiers and pricing-dependent `TokenUsage` methods moved from `elai_core::usage` to new `elai_providers::pricing`; behavior and five production call sites were preserved. | L31-L37, L150-L172 |
| E-C14-007 | schema/API | Post-move core `usage.rs` contains `TokenUsage` (four fields plus `total_tokens`) and cumulative `UsageTracker`; core re-exports only those two. | L161-L170 |
| E-C14-008 | compatibility | `elai_orchestrator::usage::*` re-exports provider pricing so existing CLI imports stay valid; provider/type/Anthropic and live-CLI call sites are rewired. | L168-L170, L331-L341 |
| E-C14-009 | evidence | `crates/core/src` is asserted to contain zero occurrences of `ModelPricing`, `UsageCostEstimate`, `pricing_for_model`, or `format_usd`. | L171-L172 |
| E-C14-010 | executed determinism | Existing POSIX script already used real target `--test bsp_determinism` and never used `--workspace`; it is now explicitly documented POSIX-only. | L38-L45, L174-L195 |
| E-C14-011 | platform parity | Add PowerShell 7+ `determinism_proof.ps1` using native temporary-file, SHA-256, and filtering APIs while preserving `ELAI_DETERMINISM_SEED` and two-run byte comparison. | L43-L45, L183-L195 |
| E-C14-012 | executed manifest | Append 78 `[[command]]` entries, one per `SlashCommand` variant at the audited enum, each with name, one-line rationale, and `KEPT | STRIPPED | DEFERRED`. | L46-L49, L197-L207, L313-L314 |
| E-C14-013 | replaced acceptance | The original entropy-quarantine cloud acceptance is replaced by workspace compilation plus zero banned-symbol matches because the dead code remains. | L51-L54, L319-L324 |
| E-C14-014 | evidence | Drift decisions live in `workflow/reasoning/C-14-decisions.md`; the audit package is `workflow/sessions/s-32952/context-package-C14.md`. | L56-L58 |
| E-C14-015 | superseded intent | The preserved draft intended deletion of 624-line `compact.rs`, 591-line `oauth.rs`, 744-line `prompt.rs`, usage stripping, a Windows script fix, and 28+ command classifications, with no prerequisite and C-15 parallel safety. | L61-L72 |
| E-C14-016 | deferred gate | `compact.rs` can only be deleted after a caller sweep proves zero production callers; the re-scope records 30+ callers and defers it. | L113-L122 |
| E-C14-017 | deferred gate | `oauth.rs` can only be deleted after proving no external OAuth/Cloud dependency; the re-scope identifies it as the live login backend and defers it. | L124-L134 |
| E-C14-018 | deferred gate | `prompt.rs` can only be deleted after zero callers and a config-loaded replacement; audit found live per-turn use, no replacement, and no stale “interactive agent” string, so deletion is deferred. | L136-L148 |
| E-C14-019 | superseded/replaced | The original task said delete pricing helpers and leave a B-27 note; the executed structural variant relocates them instead and is authoritative. | L150-L172 |
| E-C14-020 | superseded/replaced | The original script task proposed a possibly fictional `seeded_determinism` target; audit proved `bsp_determinism` and POSIX portability were the real issues. | L174-L195 |
| E-C14-021 | deferred metrics | `[dead_code_strip]` with four actual LOC counts is postponed; only usage changed (279→81), the three file-strip counts are zero, and writing the section now was judged misleading. | L209-L218 |
| E-C14-022 | deferred tests | All seven `dead_code_strip_contract` tests are postponed because three assert absence of files that must remain; safe banned-symbol/re-export checks are covered by deterministic grep. | L220-L227 |
| E-C14-023 | reachability | Current work is observable from compilation, both determinism scripts, and the migration manifest; it is not a runtime feature removal. | L229-L266 |
| E-C14-024 | quality gate/status | Scoped orchestrator+CLI and full workspace compilation are recorded passing on 2026-05-17; banned-symbol grep and 78-status count are recorded done. | L268-L324 |
| E-C14-025 | artifact set | Adds provider pricing and PowerShell proof; modifies core/provider/orchestrator/CLI imports, POSIX proof, and migration manifest; deletion/metrics/test artifacts remain explicitly deferred. | L326-L347 |
| E-C14-026 | future authority | Follow-up deletion requires explicit product confirmation that compaction, OAuth login, and `SystemPromptBuilder` are to be removed rather than relocated. | L354-L359 |
| E-C14-027 | future dependency | Follow-up requires the phase-0 six-step compact refactor, OAuth migration to `OAuthTokenSet` or deprecation of `/login`, and a new config-loaded prompt resolver. | L359-L364 |

Explicit anomalies:

- **C-C14-01:** The preserved original labels the three files dead/safe to delete, but the authoritative re-scope proves all three live and postpones deletion (L7-L29 versus L64-L92/L113-L148).
- **C-C14-02:** Original task 4 says the pricing API should be removed without inline replacement, while executed status says it was behavior-preservingly relocated and re-exported (L150-L170).
- **C-C14-03:** Original task 5 diagnoses `--workspace` and suggests `seeded_determinism`; executed audit states neither claim was true (L174-L195).
- **A-C14-01:** Exact distribution of the “40+ aggregate” and `compact.rs` “30+” caller counts is not enumerated, and the 78 per-command classifications are not reproduced in this plan (L7-L22, L121-L122, L203-L207).

Summary deltas:

- **DELTA! C14-1:** The orientation summary's deletion of all three files, stripped LOC metrics, and seven required tests is stale and false for the authoritative source state; each is postponed.
- **DELTA! C14-2:** The script was not changed from workspace-wide to `seeded_determinism`; it already targeted `bsp_determinism`, was documented POSIX-only, and gained a PowerShell sibling.
- **DELTA! C14-3:** Pricing helpers were moved, not removed; 78 rather than “at least 28” command entries were completed.
- **DELTA+ C14-4:** Adds concrete shipped-feature dependencies, completed artifact list, audit evidence paths, replacement acceptance, and explicit product authority required for follow-up.

### C-20 — Metrics TOML Backfill and Measurement Suite

Source status: implementation plan, estimated 1-2 sessions; depends on C-9, C-13, and C-17; parallel-safe with C-18/C-19 (lines 1-9).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C20-001 | capability | Add `elai measure` (or equivalent), populate placeholder metrics from actual measurements, gate a debug batch delay, and include a live cloud Sonnet 4.6 smoke. | L1-L5 |
| E-C20-002 | gap/data | `phase_4G.toml` lacks seven named runtime categories: auto-advance accuracy, false escalation, mode-switch latency, split overhead, model-swap time, end-to-end time, and gate overhead. | L13-L20 |
| E-C20-003 | gap/data | `phase_4F.toml` has four named unmeasured values and lacks six of nine spec metrics. | L22-L24 |
| E-C20-004 | gap/data | `phase_4B.toml` is absent; required keys include IPC, four-language compile latencies, ShortCoder savings, test count, and tool LOC. | L26-L28 |
| E-C20-005 | gap/data | `phase_4M.toml` lacks per-backend latency, cost, fallback rate, and uptime; `phase_4C.toml` lacks nine quantitative categories. | L30-L35 |
| E-C20-006 | gap/data | Scale metrics contain only synthetic `elai_self`; real Linux, Chromium, VS Code, and PostgreSQL runs are absent. | L37-L39 |
| E-C20-007 | gap/status | `phase_1F.toml` and `phase_1G.toml` are absent; 1F must be `UNAVAILABLE`, 1G needs a quantization sweep, and populated DROP-result `phase_1H` needs completeness verification. | L41-L47 |
| E-C20-008 | schema/surface | Add `SlashCommand::Measure {scope:MeasureScope}` with `All`, `Phase(String)`, `Scale`, and `Cloud`, handled in new `measure.rs`. | L67-L75, L172-L189 |
| E-C20-009 | workflow | `--all` runs task passes 2-9 sequentially; `--phase 4G` runs only 4G; `--scale` launches `run_all_tiers.sh` through Tokio; `--cloud` runs hybrid smoke and asserts 4M data. | L70-L74 |
| E-C20-010 | artifact/schema | Each completion writes uncommitted `metrics/last_measure_run.toml` with timestamp, `scopes_run`, `files_written:Vec<String>`, and per-file pass/fail. | L74-L75, L189 |
| E-C20-011 | config/lifecycle | Read `ELAI_DEBUG_BATCH_DELAY_SECS` once at startup and sleep that many seconds between measurement-batch baby-step dispatches only when `ELAI_MEASURE_MODE=1` and delay is set. | L77-L83 |
| E-C20-012 | constraint/test | Production with absent env vars must take a no-op/claimed zero-overhead path; a unit test covers both-var delay and all other skip cases. | L81-L83, L206-L211 |
| E-C20-013 | measurement | 4B IPC benchmark dispatches ten tool types using Criterion or existing-style `Instant`, recording p50/p99 `ipc_overhead_ms`. | L85-L95 |
| E-C20-014 | measurement | 4B compile-check benchmarks known-good Rust, Python, TypeScript, and Go fixtures, recording per-language p50/p99. | L89-L94 |
| E-C20-015 | measurement | 4B measures ShortCoder R1-R10 percent reduction on a 300-line Rust file, reads `test_count`, and computes `loc_tools` with line counts over tool Rust files. | L91-L95 |
| E-C20-016 | measurement | 4G auto-advance accuracy uses 20 deterministic cases (10 yes/10 no); false-escalation is recorded as 0/20 by fixture construction. | L97-L110 |
| E-C20-017 | measurement | 4G times `set_mode` over 100 calls and `decide(Evidence::all_passed)` over 200 calls, reporting p50/p99. | L101-L103 |
| E-C20-018 | measurement/event | 4G drives five synthetic circuit-breaker trips and counts actual `EscalationFiredPayload` events. | L104-L105 |
| E-C20-019 | unmeasured/fallback | 4G `split_task_overhead_ms` may use `unmeasured_reason="requires_tree_wiring_C9"`; model substitution times config-only switching and expects p50 below 1ms. | L107-L110 |
| E-C20-020 | measurement | 4F runs `DefaultSystemProber::probe()` 20 times for p50/p99 and `SyntheticStubExecutor` on a five-line stub 100 times for p50/p99. | L112-L116 |
| E-C20-021 | measurement | 4F performs ten cycles with `MAX_ALTERNATIVES_PER_STEP=3`, records mean alternatives, and records `novel_step_skip_rate=0.0` with synthetic rationale. | L117-L120 |
| E-C20-022 | measurement | 4F times `check_constraints` with three resources over 1,000 calls, reporting p50 microseconds with expected value below 100µs. | L121-L122 |
| E-C20-023 | measurement/health | 4M health-checks each of 11 backends for p50/p99 and marks a backend `Unavailable` when its CLI is absent. | L124-L127 |
| E-C20-024 | live integration | For `Sidecar` and `AnthropicApi`, 4M sends canary prompt `ok` under measure mode and collects completion latency. | L127-L128 |
| E-C20-025 | failure/measurement | Force primary unavailable 100 times, count fallback rows, and extrapolate `fallback_fire_rate_per_1000`; derive uptime from snapshots. | L129-L131 |
| E-C20-026 | config/economics | `cost_per_1m_tokens` comes from static config and is annotated `source=config`, not represented as a dynamic measurement. | L132-L133 |
| E-C20-027 | cloud workflow | Hybrid CLI execution must populate actual `AnthropicApi` and `Sidecar` rows and append retry-derived fallback rate (0.0 is valid if no retry). | L134-L136, L286-L288 |
| E-C20-028 | measurement | 4C declares nine categories: hook latency, tier-analysis p50/p99, T1/T2/T3 splits, block/warn rates, sanitization overhead, lock contention, SSB latency, and pre-compact duration. | L138-L146 |
| E-C20-029 | measurement | 4C runs 100 synthetic `PreToolUse` hook events, 30 tier fixtures split 10/10/10, and measures SSB latency or cites phase 2A. | L143-L146 |
| E-C20-030 | scale dependency | Real scale runs require `ELAI_SCALE_FIXTURE_ROOT` containing `linux/chromium/vscode/postgres`, prepared by `clone_scale_fixtures.sh`. | L147-L152 |
| E-C20-031 | scale measurement | For each tier, collect p50/p95/p99 for four traversal tools plus peak RSS and write under the tier key. | L151-L155 |
| E-C20-032 | scale gate | Every scale p99 must be ≤500ms; run the normally ignored scale-determinism test once. | L155-L157 |
| E-C20-033 | scale fallback | Missing fixture root records `skipped_reason="fixture_root_not_set"` and does not fail the overall run. | L158-L159 |
| E-C20-034 | status artifact | Create 1F with `phase=1F`, `decision=UNAVAILABLE`, reason `prompt_lookup_decoding_not_instrumented`, and `recommended_action=no_further_work`. | L161-L164 |
| E-C20-035 | model measurement | 1G runs `quantize_cross_encoder.py` and records, per level, model MB, p50/p99 latency, and quality delta. | L165-L167 |
| E-C20-036 | verification | 1H must contain vocabulary coverage, chosen variant, p50, recall delta, Wilcoxon p-value, and `decision=DROP`; add `generated_at` if missing. | L168-L170 |
| E-C20-037 | truth constraint | Every written field must come from an actual measurement or carry `unmeasured_reason`; fabricated/bare placeholder numbers are forbidden. | L206-L209, L253-L256 |
| E-C20-038 | reachability | Measurement is reachable from CLI, existing scale-budget CI, and phase auditors. | L202-L205 |
| E-C20-039 | acceptance | `--all` expects scopes `4B,4C,4F,4G,4M,1F,1G,1H,scale,cloud`; every scope passes except the explicitly skippable scale fixture. | L259-L263 |
| E-C20-040 | acceptance | 4B requires every named field and disallows zero/placeholder; 4G requires six of seven categories with the one C-9 reason allowed. | L264-L268 |
| E-C20-041 | acceptance | 4F requires all six named runtime outputs non-placeholder. | L269-L271 |
| E-C20-042 | acceptance | 4M acceptance expects seven CLI adapters to report `Unavailable` when binaries are absent. | L272-L273 |
| E-C20-043 | acceptance | 4C acceptance specifically names hook p50/p99, three tier percentages, block rate, and sanitization overhead, allowing zero only when genuinely measured. | L274-L276 |
| E-C20-044 | acceptance | Preserve `elai_self`; populate the four real scale tiers when configured and pass the scale contract. | L277-L279 |
| E-C20-045 | quality gate | Workspace compilation must add zero errors, and production measure handler may add neither `.unwrap()` nor `.expect()`. | L284-L290 |
| E-C20-046 | artifact set | Add measure module, three new phase files, optional scale script, and decisions doc; modify commands, task runner, six metric files. | L292-L308 |

Explicit anomalies:

- **C-C20-01:** Context says “seven phase files” but explicitly inventories nine metric files/areas (4G, 4F, 4B, 4M, 4C, B29.5, 1F, 1G, 1H) (L13-L47).
- **C-C20-02:** C-9 is a prerequisite, yet split overhead may remain unmeasured because it “requires_tree_wiring_C9” (L7-L9, L107-L110).
- **C-C20-03:** `--all` must include/pass cloud, while cloud explicitly requires a live sidecar and Anthropic API key; unlike scale, no missing-credential skip/failure policy exists (L70-L75, L213-L218, L259-L263).
- **C-C20-04:** The plan says 11 backends, but acceptance quantifies seven absent CLI adapters and names only two live backends; disposition of the other two is not enumerated (L124-L136, L272-L273).
- **A-C20-01:** The new 4B file is supposed to read `test_count` “from phase_4B.toml notes,” although that file does not exist; source of the count is unresolved (L26-L28, L91-L95).
- **A-C20-02:** A single canary/task is said to populate p50/p99, which does not define a sample distribution; percentile calculation/interpolation and warm-up rules are absent throughout (L124-L136).
- **A-C20-03:** 4C declares nine categories, but tasks only give procedures for hooks, tier splits, and SSB latency; warn/block/sanitization/contention/pre-compact procedures and several acceptance keys are missing (L138-L146, L274-L276).
- **A-C20-04:** `end_to_end_pipeline_time` is one of the seven 4G gaps but has no task; the “six of seven” acceptance allows one omission in addition to split's unmeasured value without saying which (L17-L20, L97-L110, L267-L268).
- **A-C20-05:** “Read env once at startup,” environment-mutating unit tests, actual-measurement reproducibility, generated timestamps, model/fixture versions, and concurrent measure-run serialization are unspecified (L74-L83, L147-L170).
- **C-C20-05:** Files-touched paths omit the otherwise pervasive `.elai_cc/` prefix for CLI/orchestrator additions, leaving repository-relative placement inconsistent (L172-L201 versus L292-L307).

Summary deltas:

- **DELTA+ C20-1:** Adds exact CLI grammar, run-summary schema, sample counts, fixtures, percentiles, limits, skip reason, static-cost provenance, and all 1F/1G/1H fields.
- **DELTA! C20-2:** The summary's broad “all phase fields non-placeholder” hides explicit allowed unmeasured/Unavailable/skipped outcomes and the unresolved cloud-credential path.
- **DELTA+ C20-3:** The source does not fully specify all nine 4C procedures, 11 backend identities, percentile method, or the seventh 4G measurement; these are design gaps rather than completed details.

### C-29 — Testing-Trust T-2 IntentPair + T-3 basic metamorphic relations

Source status: implementation plan, estimated 2-3 sessions; C-27 and C-28 are true prerequisites/blockers; parallel-safe with C-21 through C-26 (lines 1-11, 292-297).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C29-001 | capability/gate | Add IntentPair formalization and identity/inverse/permutation metamorphic checks; require at least three pairs for `PLANNING -> TESTING_FIRST`; run metamorphic checks after mutation; add PlanWindow editing. | L1-L8 |
| E-C29-002 | schema | `IntentPair` fields: `natural_language_intent:String`, `concrete_test_assertion:String`, `baby_step_ref:BabyStepId`. | L43-L47 |
| E-C29-003 | schema/state | `IntentPairSet` fields: `pairs:Vec<IntentPair>`, `confirmed:bool`, `confirmed_at:Option<DateTime<Utc>>`; methods `len`, `is_confirmed`, and `confirm` (sets bool plus timestamp). | L46-L48 |
| E-C29-004 | schema | `IntentConfirmation` fields: `step_id`, `pair_count:usize`, `confirmed_by:ConfirmedBy`, `timestamp:DateTime<Utc>`; `ConfirmedBy = User | AutonomousGate`. | L49-L52 |
| E-C29-005 | compatibility | All intent types derive debug/clone/serde; no unwrap; `BabyStepSpec.intent_pairs` is `#[serde(default)]`, producing an empty unconfirmed set so old JSON loads without a BabyStepSpec migration. | L51-L57 |
| E-C29-006 | schema/version | Bump the SSB constant from 46 to 47 with a C-29 comment, add payload structs/match arms, and preserve serde round trips. | L59-L64, L224-L233 |
| E-C29-007 | event/schema | `IntentConfirmed` carries `step_id:u64`, `pair_count:u8`, and `confirmed_by:String`. | L59-L64 |
| E-C29-008 | event/schema | `MrViolated` carries `step_id:u64`, `relation:String`, non-empty `counterexample:String`, and `sample_size:u32`. | L61-L64, L161-L166 |
| E-C29-009 | config/default | Add `[testing.intent] min_pairs=3` and `IntentConfig {min_pairs:usize}` using the shared config hierarchy. | L66-L69 |
| E-C29-010 | gate/failure/event | After existing gate checks, fewer than `min_pairs` emits `gate_decision` reason `intent_pairs_below_threshold` and returns `Reject`; every rejection must be observable. | L71-L75, L161-L165 |
| E-C29-011 | gate/event | A threshold pass emits `IntentConfirmed`; autonomous flow auto-confirms qualifying pairs with `confirmed_by=AutonomousGate`. | L71-L75 |
| E-C29-012 | schema | `MetamorphicRelation = Identity | Inverse | PermutationInvariance`. | L77-L78 |
| E-C29-013 | schema | `MrResult` contains relation, pass bool, sample size, optional counterexample, and `duration_ms:u64`; `RelationLibrary` holds enabled relations. | L79-L81 |
| E-C29-014 | config/failure | `RelationLibrary::from_config(&[String])` parses names, warns and skips unknown values without panic. | L80-L82 |
| E-C29-015 | config/default | Default relation set is Identity+Inverse; permutation invariance is excluded/deny-by-default. | L81-L82, L234-L241 |
| E-C29-016 | runner/state | `MetamorphicTestRunner` holds `RelationLibrary`, `SsbSender`, and `sample_size:u32`. | L84-L85 |
| E-C29-017 | test behavior | `run_identity` calls the tested function twice on identical bytes; mismatch stores the hex input counterexample. | L84-L87 |
| E-C29-018 | test behavior | `run_inverse` checks `decode(encode(x))==x` for each sample and records the first failing sample as counterexample. | L87-L89 |
| E-C29-019 | test behavior/config | `run_permutation_invariance` executes only when enabled; `run_all` executes all enabled relations, emits one `MrViolated` per failure, never panics, and returns the full result set. | L89-L91 |
| E-C29-020 | workflow/config | Run metamorphic tests after C-28 mutation only when `[testing.metamorphic] enabled=true`; default config is enabled with `relations=["identity","inverse"]`. | L93-L100 |
| E-C29-021 | gate/scoring | MR violations contribute to `AUDITING -> MEMORIZING` scoring alongside mutation score and are not independently blocking. | L93-L97, L125-L129 |
| E-C29-022 | surface | Inline PlanWindow panel lists existing pairs on step selection; its three fields are NL intent, assertion, and auto-filled step ref. | L102-L108 |
| E-C29-023 | surface/state | Add calls `add_intent_pair`; Confirm is enabled at count ≥3 and calls `confirm_intent_pairs`; both map `Result<(),String>` errors to toast. | L102-L108 |
| E-C29-024 | API/state | `add_intent_pair(step_id,natural_language_intent,concrete_test_assertion)` appends a pair and errors if the step is absent. | L110-L115, L248-L249 |
| E-C29-025 | API/gate/event | `confirm_intent_pairs(step_id)` calls `IntentPairSet::confirm`, errors below configured minimum (acceptance string: `need N more pairs`), and emits `IntentConfirmed`; register both commands in `generate_handler!`. | L110-L115, L250-L251 |
| E-C29-026 | observability | Smoke begins with the below-threshold event, then editor-added/confirmed three pairs lead to approval plus `IntentConfirmed(pair_count=3)`. | L117-L123 |
| E-C29-027 | test/observability | Correct encoder/decoder over 100 random strings returns inverse pass with no violation; off-by-one bug yields inverse violation and non-empty counterexample. | L117-L123, L242-L256 |
| E-C29-028 | decision | Decision doc justifies minimum three, default exclusion of permutation for non-commutative false positives, and non-blocking standalone MR due to relation noise. | L125-L129 |
| E-C29-029 | quality gate | Workspace check, orchestrator intent/metamorphic tests (at least eight new functions), contract intent tests, and both SSB serde round trips must pass. | L180-L222, L224-L260 |
| E-C29-030 | constraint | Inverse must return rather than panic; confirmation must error below threshold; gate rejects and MR failures must never be silent/empty; relation choice must come from config. | L161-L166 |
| E-C29-031 | artifact set | Add `intent_pair.rs`, `metamorphic_testing.rs`, and decisions; modify research tree, SSB, gate, deterministic auditor, config, Tauri lib, and PlanWindow. | L262-L282 |
| E-C29-032 | deferred/non-goal | Testing-trust tiers T-4 through T-13 (risk, equivalent cache, regression corpus, Kani, provenance, CiteAudit, PRPO, Rulebreakers, XGrammar/CRANE, FACET R-DAG) are future/post-launch work. | L284-L290 |

Explicit anomalies:

- **C-C29-01:** `BabyStepSpec` compatibility says no schema bump is needed, while the same task explicitly bumps SSB schema 46→47; these are different schemas but the source uses “schema” without consistently qualifying the boundary (L54-L64).
- **A-C29-01:** Gate enforcement checks only pair count and emits confirmation on pass; it never explicitly requires `IntentPairSet.confirmed=true`. The separate UI command also confirms/emits, so user confirmation persistence and duplicate-event behavior are unresolved (L71-L75, L102-L115).
- **A-C29-02:** `add_intent_pair` says it appends “to SSB” yet acceptance says it mutates `BabyStepSpec.intent_pairs`; no pair-added event, load path, storage transaction, restart persistence, or concurrent-edit policy is defined (L102-L115, L248-L253).
- **A-C29-03:** `pair_count` narrows from `usize` to SSB `u8` without maximum or overflow behavior (L49-L50, L61-L63).
- **A-C29-04:** Identity means repeatability (`f(x)==f(x)`), not a stated identity transform; permutation input semantics, `run_all` function arguments, sample-size default, duration clock, and failure isolation are unspecified (L84-L91).
- **A-C29-05:** “100 random strings” has no seed/generator/corpus pin, so deterministic replay and counterexample stability are not promised (L117-L123).
- **A-C29-06:** The formula, weights, threshold, and missing-result behavior for combining MR outcomes with mutation score are absent (L93-L97, L125-L129).
- **A-C29-07:** C-29 says current SSB is 46 and bumps it to 47; the orientation summary separately identifies a cross-plan collision with C-1/C-27, but this source contains no migration ordering that resolves it (L21-L25, L59-L64).

Summary deltas:

- **DELTA+ C29-1:** Adds all exact field types, methods, event payloads, runner state/results, command signatures/errors, counterexample rules, decision rationale, and deferred tiers.
- **DELTA! C29-2:** “Confirmation gate” is stronger than the original algorithm: the source explicitly gates on count, not on the persisted `confirmed` flag, and may emit confirmation from both gate and command.
- **DELTA+ C29-3:** Sample generation/seeding and MR+mutation aggregate scoring remain undefined; the summary should not imply a closed scoring or persistence design.

### C-45 — CANTANTE contrastive role credit attribution

Source status: implementation plan. The GPU pilot is explicitly deferred to RunPod; C-41 is recommended but not mandatory; all eight role implementations are required (lines 1-17, 58-61).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C45-001 | objective | Measure causal contribution/noise per `AgentRole`; feed scores to C-41 routing and future consolidation evidence. | L1-L6 |
| E-C45-002 | dependency/schema | Required roles are Executor, Auditor, Researcher, Designer, PromptEngineer, ContextCompiler, MemoryWriter, and MediaValidator. | L8-L10 |
| E-C45-003 | methodology | For each role/task, compare role available against role forcibly dropped, route around the drop with merged fallback, and aggregate success deltas over N tasks. | L19-L34 |
| E-C45-004 | dependency/artifact | Run paper-ingestor for arXiv:2605.13295 and produce `workflow/papers/cantante-credit-attribution.md`. | L36-L45 |
| E-C45-005 | state/API | `AgentRoleDropper`/`RoleFallback` uses `RuntimeContext.dropped_role:Option<AgentRole>`; FSM checks it and never spawns the dropped role. | L47-L51, L84-L102 |
| E-C45-006 | fallback | Every role must have a documented non-empty fallback; example: dropped Researcher uses cached prior research only; panic/silent skip are banned. | L47-L51, L71-L79, L100-L104 |
| E-C45-007 | harness | `credit_attribution_harness` accepts a task set plus nine variants (baseline + eight single-role drops), runs each, and aggregates success/audit-pass rates. | L53-L56 |
| E-C45-008 | artifact | Harness produces `credit-attribution.md`; deferred pilot produces `workflow/sessions/{id}/cantante-pilot.md`. | L53-L61 |
| E-C45-009 | deferred/cost | RunPod pilot uses 50 SWE-bench Verified tasks ×9 configurations =450 runs and projects about $50-$200 depending on local/hybrid mix. | L58-L61 |
| E-C45-010 | config/integration | Expose per-role score as SkillRegistry routing metadata; automatically deprioritize low-credit roles only behind config, default off until validated. | L63-L66 |
| E-C45-011 | event | Add `agent_role_dropped`, `credit_attribution_run`, and `credit_attribution_result`; result contains the per-role delta map and confidence interval over N runs. | L68-L69, L84-L104 |
| E-C45-012 | math | Credit delta is `baseline_success_rate - dropped_success_rate`, computed both per task and aggregate. | L71-L77 |
| E-C45-013 | ordering/observability | Emit `agent_role_dropped` before dispatch reroutes; harness emits one run event per task and one result at aggregation. | L78-L79, L106-L109 |
| E-C45-014 | determinism/test | Synthetic harness must be deterministic/no-GPU; tests require all fallbacks non-empty and correct delta math. | L71-L79, L121-L131 |
| E-C45-015 | documentation/status | After pilot, add the credit table/CI to `cantante-pilot.md` and annotate `DETAILED_SUMMARY_01` AgentRoles; pilot and those post-pilot artifacts remain deferred. | L81-L82, L121-L142 |
| E-C45-016 | decision gate | Role consolidation remains a future decision and requires measured low independent credit; C-45 itself does not consolidate roles. | L144-L149 |

Explicit anomalies:

- **A-C45-01:** The required “per-role fallback table” is not present; only the Researcher example is specified, leaving seven fallback behaviors open (L47-L51, L100-L104).
- **A-C45-02:** Confidence level, interval estimator, paired-task treatment, success/audit aggregation, missing-run handling, random seeds, task-subset pin, and execution order are unspecified (L26-L34, L53-L61, L103-L104).
- **C-C45-01:** `DETERMINISTIC_CHECK` requires the deferred RunPod pilot artifact to exist, while status/acceptance explicitly defer it (L58-L61, L111-L127).
- **A-C45-03:** The source names both `credit-attribution.md` and `cantante-pilot.md` without an exact path for the first or a statement that one supersedes/embeds the other (L53-L61).
- **A-C45-04:** It says no frontend pair but mentions surfacing through existing `AgentsPanel`, without a wiring task or display contract (L105).

Summary deltas:

- **DELTA+ C45-1:** Adds the exact eight roles, nine-arm/450-run design, score formula, cost range, event ordering, config-default-off routing feedback, and distinction between synthetic and deferred pilot evidence.
- **DELTA! C45-2:** Neither a complete fallback table nor confidence calculation is supplied by the source; they are required outputs, not recovered details.
- **DELTA+ C45-3:** Pilot-artifact deterministic check conflicts with deferred status, and the source has two differently named report artifacts.

### C-48 — SafeHarbor hierarchical memory guardrail

Source status: implementation plan; memory-writer and C-42 are stated SHIPPED prerequisites; parallel-safe with the rest of its wave except itself (lines 1-14).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C48-001 | objective/security | Stratify memory by provenance and refuse T2/T3 memory for safety-critical decisions to mitigate prompt injection/context poisoning. | L1-L6 |
| E-C48-002 | threat model | Explicit threats are malicious commit/changelog memory, web/paper prompt injection, and model-fabricated persistent claims. | L16-L26 |
| E-C48-003 | schema/trust | `TrustTier = T0_UserDirect | T1_CodebaseDerived | T2_WebSourced | T3_ModelEmitted`. | L28-L37, L52-L54 |
| E-C48-004 | policy | T0 includes CLI/dashboard/explicit user writes; T1 includes local source and git history; T2 includes WebFetch/paper ingestor; T3 is unreviewed model output. | L28-L37 |
| E-C48-005 | policy | Safety-critical queries consult only T0+T1; enrichment may consult T2+T3 but must retain provenance labels. | L39-L40 |
| E-C48-006 | write lifecycle | Every memory write is tagged at write time. | L52-L54, L95-L101 |
| E-C48-007 | schema/migration | Extend `MemoryRow` with non-optional `trust_tier:TrustTier` and `provenance:Provenance` (source URL or filepath), bump schema, and migrate every pre-C-48 row to pessimistic T3. | L56-L59, L98-L102 |
| E-C48-008 | classifier/config | Safety-critical query types are `destructive_command_intent`, `merge_decision`, `auth_decision`, and `credential_fetch`. | L61-L64 |
| E-C48-009 | filter | Those query types use `MemoryView::T0_T1_only()` to transparently exclude T2/T3. | L61-L64 |
| E-C48-010 | provenance/read | Every memory read surfaces provenance; safety-critical agent prompts label each cited fact, e.g. `user_direct` versus `web_sourced`. | L66-L69, L95-L102 |
| E-C48-011 | event | Add exact action identifiers `memory_tier_assigned`, `memory_query_filtered`, and `safety_critical_blocked`. | L71-L73, L85-L101 |
| E-C48-012 | event/observability | When an excluded T2/T3 row would have influenced a safety query, emit `safety_critical_blocked`; silent exclusion is prohibited. | L71-L73, L98-L101 |
| E-C48-013 | test | TDD covers trust-tier schema round trip, exclusion of T2/T3, visible prompt provenance, and pessimistic legacy migration. | L75-L80 |
| E-C48-014 | adversarial/test | Test 4D.X.7 places malicious text in T2 memory and requires no influence on a safety-critical decision. | L80-L82, L113-L126 |
| E-C48-015 | contract | Add `tests/contracts/memory_trust_contract.rs`; all three actions must emit on relevant behavior. | L83, L113-L126 |
| E-C48-016 | artifact set | Add trust/view modules, TDD/adversarial/contract tests, shell migration, and paper artifact; modify `MemoryRow`, memory writer, and orchestrator SSB schema. | L128-L139 |
| E-C48-017 | non-replacement | The existing memory layer is extended, not replaced. | L92-L94 |
| E-C48-018 | claimed outcome | Closure claims a malicious third-party commit message cannot influence a safety-critical decision and calls C-48 the wave's only explicit adversarial threat model. | L141-L146 |

Explicit anomalies:

- **C-C48-01:** A malicious commit message is the motivating threat and is classified by the stated taxonomy as git-history/codebase-derived T1; T1 is allowed in `T0_T1_only`. Therefore the closure claim that such a commit cannot influence a safety-critical decision does not follow from the explicit policy (L21-L22, L32-L33, L39-L40, L141-L144).
- **C-C48-02:** T1 is called trustworthy for technical decisions “but not safety policy,” yet all safety-critical views include T1 without an additional policy filter (L32-L33, L39-L40).
- **A-C48-01:** `Provenance` variants/fields for user/model sources, URL/path sanitization, sensitive-path exposure, inheritance through derived/merged/summarized rows, and validation are undefined (L56-L69).
- **A-C48-02:** Query classification location, behavior for unknown/multi-purpose queries, caller bypass prevention, and whether the four-item list is closed/configurable are unspecified (L61-L64).
- **A-C48-03:** Determining that an excluded row “would have influenced” a decision is counterfactual and no algorithm distinguishes `memory_query_filtered` from `safety_critical_blocked` emission (L71-L73).
- **A-C48-04:** Schema-version number/scope, migration transaction/idempotency/rollback, write failure for absent tier/provenance, and concurrent reads during migration are unspecified (L56-L59, L128-L139).

Summary deltas:

- **DELTA! C48-1:** The exact third action is `safety_critical_blocked`, not the summary's `memory_safety_critical_blocked`.
- **DELTA+ C48-2:** Adds the closed four-query list, write/read universality, non-optional provenance, pessimistic migration rule, exact adversarial fixture, and “extends, not replaces” constraint.
- **DELTA! C48-3:** The summary omits the internal threat-model contradiction: T1 git history is admitted to safety decisions even though malicious commit messages motivate the guardrail and T1 is said not to govern safety policy.

### C-59 — Cross-config benchmark master report capstone

Source status: implementation/publication plan. It is blocked until C-54 through C-58 have shipped completed runs, is not parallel-safe with benchmark patches, and closes C-35 through C-59 only on acceptance (lines 1-17, 181-194).

| Atomic ID | Class | Explicit feature or behavior | Source anchor |
|---|---|---|---|
| E-C59-001 | objective | Aggregate C-54 SWE-bench, C-55 Aider, C-56 Codebase-Memory, C-57's three evaluations, and C-58's two evaluations into one public-facing master report. | L1-L7 |
| E-C59-002 | optional research | If RunPod time permits, pilot Test-Time Speculation (arXiv:2605.09329), whose cited prior result is up to 72%/41% average acceptance-length improvement. | L6-L7, L14-L17 |
| E-C59-003 | input/dependency | Consume 7+ scattered scorecards/results; exact glob families include one C-54, one C-55, one C-56, three C-57, and two C-58 paths. | L19-L23, L49-L59 |
| E-C59-004 | report/schema | Cross-config matrix rows are benchmarks; columns are local 7B, dual-local 7B, hybrid Sonnet+Composer2+local, and hybrid Sonnet-only; each cell is `(solve-rate,cost,wall-clock-p99)`. | L25-L28 |
| E-C59-005 | report/schema | Cross-competitor table compares best ELAI against Devin, OpenHands, Aider, Claude Code, GPT-5.4, Composer 2, AgentForge, and Codebase-Memory, each pinned by version/SHA, date, and citation URL. | L29-L31, L71-L74 |
| E-C59-006 | report/economics | Cost-per-task analysis reports ELAI dollars/task across all benchmarks/configs; use a plot only if dashboard support exists, otherwise a table. | L32-L33 |
| E-C59-007 | honesty | Mandatory gap narrative covers losses, wins, and ambiguous/config-dependent comparisons rather than only favorable results. | L34-L39 |
| E-C59-008 | planning output | Rank concrete improvement targets by leverage, derived from each losing benchmark's delta, and feed the next planning cycle. | L38-L39, L76-L80 |
| E-C59-009 | optional workflow | TTS reruns config A local 7B on a 50-task SWE-bench Verified subset, compares wall time with C-54 A, documents a positive result, and archives a non-positive result. | L45-L47, L94-L100 |
| E-C59-010 | artifact | Canonical output is `.elai_cc/docs/benchmark_master_report.md`; a talk slide deck is only a planned separate follow-up. | L61-L64, L90-L92 |
| E-C59-011 | aggregator/lifecycle | `MasterReportAggregator` validates all scorecard schemas and regenerates Markdown solely from machine scorecards plus baseline table. | L66-L69, L115-L136 |
| E-C59-012 | external integration | `.elai_cc/benchmarks/external_baselines.toml` stores published numbers/version/date/URL from the prior oracle plus fresh verification at render time. | L71-L74 |
| E-C59-013 | constraint | Exclude any external number lacking version and date; report build fails if honesty is absent/empty; improvement list must be non-empty whenever ELAI loses. | L104-L113, L130-L136 |
| E-C59-014 | honesty detail | Limitations must name invalid config comparisons, differing benchmark methodologies, one-shot versus multi-shot mismatch, and cloud sampling stochasticity. | L82-L89 |
| E-C59-015 | event | Add `master_report_aggregated`, `tts_pilot_run`, and `tts_pilot_decision`. | L101-L102, L115-L126 |
| E-C59-016 | test/determinism | Fixture aggregator must produce a deterministic matrix from N scorecards; same inputs must regenerate byte/logically identical output idempotently. | L104-L110 |
| E-C59-017 | test/schema | Validate required baseline fields; ensure loss derivation yields at least one target; make honesty a build gate. | L107-L113 |
| E-C59-018 | CLI/surface | Reach from `elai benchmark master-report` and optional `--tts-pilot`. | L127-L129, L166-L174 |
| E-C59-019 | acceptance | Report must contain all five sections, all three events must emit, TDD/idempotency must pass, and optional TTS appears only if run. | L150-L164 |
| E-C59-020 | publication boundary | “Public-ready” means linkable from README/blog/paper; this task creates the artifact but does not itself publish externally. | L163-L164 |
| E-C59-021 | traceability | Each C-54..C-58 verification report receives a closure annotation pointing to the master. | L176-L179 |
| E-C59-022 | lifecycle/source of truth | After closure, the master becomes source of truth for external competitiveness claims; reruns regenerate from scorecards+baseline TOML, and targets seed C-60+. | L181-L191 |

Explicit anomalies:

- **C-C59-01:** The report must be deterministic/idempotent for the same inputs, while baseline inputs are fresh-fetched at render time and may change; no snapshot/update boundary reconciles live fetch with stable inputs (L71-L74, L104-L110).
- **A-C59-01:** Scorecard schema/version, discovery order/deduplication, incomplete/corrupt/mixed-run handling, required benchmark set, and whether glob results are sorted are unspecified (L49-L69).
- **A-C59-02:** Units/currency/date normalization, cost allocation, percentile aggregation, “best config” tie-breaks, missing-cell representation, and cross-benchmark comparability are absent (L25-L39).
- **A-C59-03:** Machine generation forbids handwritten narrative drift, but algorithms/templates for honesty prose and one concrete next-step per loss are not specified (L68-L80, L130-L136).
- **A-C59-04:** External baseline refresh failure/retry/cache behavior and version-vs-SHA requirements are not defined; context says “version SHA” while the table task says version (L29-L31, L71-L74).
- **A-C59-05:** TTS budget threshold, 50-task subset pin, speculative implementation/config, statistical decision threshold, archive location, and negative-run event semantics are unspecified (L45-L47, L94-L102).
- **A-C59-06:** Event payload schemas, emission ordering/idempotency, and whether skipped TTS emits `tts_pilot_decision` are unstated (L101-L102).

Summary deltas:

- **DELTA+ C59-1:** Adds the exact four configurations, per-cell triple, eight scorecard glob families, complete limitation taxonomy, loss→one-target rule, TTS positive/archive behavior, CLI, and verification-report annotations.
- **DELTA! C59-2:** The source names eight competitor families; this confirms the summary's correction to the earlier “seven anchors” narrative.
- **DELTA+ C59-3:** “Deterministic regeneration” is conditional on identical pinned inputs and remains in tension with render-time fresh fetch; input schema and normalization/failure policies are not recovered by the original.

## Cross-source interactions and lifecycle composition

### Explicit interaction edges

| Edge | Explicit interaction | Anchors |
|---|---|---|
| C-1 → C-3 | C-1 is expected to produce query/CSQ and possible freshness/canary/budget rows; C-3 reads and exposes them, with absence sentinels until producers exist. | C-3 L6-L7, L70-L141, L257-L269 |
| C-2/C-3 → C-7 | C-7 consumes C-2 edit rows and attributes four review commands to C-3; the command attribution conflicts with C-3's complete task list. | C-7 L6-L7, L78-L84, L124-L129; C-3 L68-L141 |
| C-2/C-9 → C-10 | C-2 supplies five SSB variants and C-9 supplies escalation/retry inputs; C-10 turns them into detection, persona, watchdog, and dashboard behavior. | C-10 L9-L12, L20-L26, L198-L209 |
| C-9 → C-11 | C-11 relies on C-9 multi-agent spawn for the Tier-C smoke executor but otherwise modifies sandbox/IPC paths disjoint from C-12–C-15. | C-11 L6-L9 |
| C-14 ∥ C-15 | Current C-14 relocation/manifest/script work claims disjoint file scope and may overlap C-15; deferred future deletion becomes a multi-feature refactor and is not covered by that simple claim. | C-14 L69-L72, L349-L364 |
| C-9/C-13/C-17 → C-20 | Measurement consumes autonomous gate, production IPC, and harness scaffolding, then feeds phase auditors and existing scale-budget CI. | C-20 L7-L9, L190-L205 |
| C-27/C-28 → C-29 | Wiring and mutation tiers must ship before intent/metamorphic smoke; C-29 runs MR after mutation and contributes to the later audit gate. | C-29 L9-L11, L93-L97, L292-L297 |
| C-45 → C-41 | C-45 scores individual role drops; only after validation may config-gated, default-off scores influence SkillRegistry weights. | C-45 L3-L10, L63-L66 |
| memory-writer/C-42 → C-48 | Shipped memory writing and zero-trust fuzzing precede universal write tagging/read filtering and the adversarial fixture. | C-48 L8-L9, L44-L46, L95-L102 |
| C-54…C-58 → C-59 | All five predecessor patches and their completed runs are hard prerequisites; C-59 consumes their scorecards and annotates their verification reports. | C-59 L3-L12, L49-L59, L176-L179 |

### Inferred composed behavior (not an independent promise)

- **I-X-01:** The dashboard evidence loop in this chunk is producer/event or DB row → viewer/Tauri query → store/window/drawer: C-3 supplies knowledge/reasoning reads, C-7 supplies review/recovery interactions, C-10 supplies stagnation display, and C-29 supplies intent editing. Delivery is mixed polling/on-mount invocation and has no single shared consistency contract.
- **I-X-02:** Anti-spiral recovery can short-circuit retries via C-10 futility, escalate via watchdog, or change persona; C-11 constrains the spawned execution environment. Neither source specifies ordering between a sandbox launch failure and anti-spiral retry/escalation.
- **I-X-03:** The evidence ladder is C-14 deterministic-script portability → C-20 measured runtime artifacts → C-29 mutation-following intent/MR evidence → C-45 ablation evidence → C-59 public aggregate. Only C-59 claims public-source-of-truth status; earlier artifacts retain their own scope.
- **I-X-04:** C-48 provenance filtering can protect later agent decisions, but no source wires its trust metadata into C-45's task scoring or C-59's benchmark input provenance; that propagation must not be assumed.

### Shared schema, determinism, security, and observability constraints

- SSB consumers/producers overlap but registration/version ownership is fragmented: C-3 consumes C-1 types; C-10 consumes C-2 types; C-29 explicitly claims 46→47; C-45/C-48/C-59 each add three action types without a numeric bump. No chunk source supplies a merged enum order or migration sequence.
- Determinism has four distinct meanings: ordered sets/blake3/rule precedence (C-10), cross-shell repeat-byte proof (C-14), honest real measurements that are inherently environment-dependent (C-20), deterministic synthetic ablation (C-45), and same-input report idempotency (C-59). These are not interchangeable.
- Explicit security/authority gates are narrow: C-11 requires user review before merge; C-29 requires pair count and offers user/autonomous confirmation; C-48 filters memory tiers. C-3/C-7 dashboard commands define no authorization model, C-45 role drops are harness/config controlled, and C-59 creates a public-ready file but does not authorize publication.
- Failure evidence favors sentinels or explicit records: C-3 uses `[]`/`known:false`; C-7 uses Empty/loading; C-10 emits/escalates; C-11 hard-exits the child; C-20 writes Unavailable/unmeasured/skipped reasons; C-29 rejects/emits/warns; C-45 forbids empty fallback; C-48 forbids silent filtering; C-59 excludes unpinned baselines and fails missing honesty.

## Status and deferred/non-goal roll-up

| Source | Explicit status boundary |
|---|---|
| C-3 | Plan; producer writes and several UI/latency/offline items remain assigned to C-1/B-3/B-4/B-5/B-8/B-36. |
| C-7 | Plan; Explore's live search intentionally remains stubbed; optional shell shortcut depends on registered window type. |
| C-10 | Plan; requires C-9 and C-2 before smoke. |
| C-11 | Plan; safety review is a merge gate and proof is required on Linux and Windows. |
| C-14 | **Partially executed/re-scoped:** pricing relocation, script parity, 78-command manifest, and compilation checks are done; three deletions, LOC metrics, and seven tests are postponed. |
| C-20 | Plan; scale may skip for absent fixtures, 1F is intentionally unavailable, and cloud has an unresolved credential dependency. |
| C-29 | Plan; C-27/C-28 block full acceptance; T-4 through T-13 are deferred. |
| C-45 | Plan; synthetic harness is local, real 50-task/450-run pilot and post-pilot docs are RunPod-deferred. |
| C-48 | Plan; no staged migration/rollback state is defined. |
| C-59 | Plan/capstone; blocked on completed C-54–C-58; TTS and slide deck are optional/follow-up. |

## Zero-omission and zero-duplicate audit

### Read coverage

- Manifest rows processed: **10/10**, once each, in manifest order.
- Complete line spans read: C-3 **1-270**; C-7 **1-238**; C-10 **1-323**; C-11 **1-257**; C-14 **1-364**; C-20 **1-314**; C-29 **1-297**; C-45 **1-149**; C-48 **1-146**; C-59 **1-194**.
- Source total: **10 files / 2,552 lines / 140,376 bytes**; recomputed hashes, bytes, and lines match the manifest exactly.
- No `numbered/index original` outside `DEEP-C-003` was opened or used. `03-C-SUMMARY_INDEX.discovery.md` was consulted only for orientation/delta comparison; the manifest itself was read only at the exact C-003 section.

### Atomic and anomaly counts

| Source | Explicit normalized atoms |
|---|---:|
| C-3 | 39 |
| C-7 | 26 |
| C-10 | 45 |
| C-11 | 26 |
| C-14 | 27 |
| C-20 | 46 |
| C-29 | 32 |
| C-45 | 16 |
| C-48 | 18 |
| C-59 | 22 |
| **Total** | **297** |

- Atomic-ID audit: **297 ledger rows / 297 unique IDs / 0 duplicate IDs / 0 sequence gaps** within every per-file namespace.
- Separation audit: **19 conflicts**, **41 ambiguities**, and **2 per-file inferences** are labeled outside explicit tables; four additional cross-source inferences are separately labeled `I-X-*` and are not counted as explicit atoms.
- Summary audit: **33 labeled deltas** (**19 additions**, **14 corrections**) record details the orientation summary omitted or misstated.
- Requested-axis audit: capabilities/actions/surfaces; workflow/state; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failure/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; and dependencies/status/deferred were all checked in every source. A source with no explicit statement in an axis was not padded with an inference.
- Deduplication rule: repeated objective, task, `WIRING_CONTRACT`, deterministic check, acceptance, and file-list claims were merged into their first atomic semantic record. Superseded C-14 text is retained only where needed to show the authoritative correction and is labeled `superseded/replaced/deferred`, never counted as current behavior.
