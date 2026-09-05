# DEEP-C-006 exhaustive feature discovery

## Scope, provenance, and integrity

This ledger is a line-by-line deep read of exactly the ten originals assigned to manifest chunk DEEP-C-006. The family summary was used only to orient comparison and to identify summary deltas; it is not treated as implementation evidence. No original from another manifest chunk was read.

### Exact processed manifest

| # | Original | Lines | Bytes | SHA-256 | Verification |
|---:|---|---:|---:|---|---|
| 1 | docs/thefinalglance/C-plan/C-13_ipc_dispatcher_production_swap.md | 295 | 17145 | 5d717b6ef26424d585fd9c4324e68d40c0e5852b586ca359a507b8198e94c2ba | exact |
| 2 | docs/thefinalglance/C-plan/C-15_phase0_debt_clock_entropy.md | 322 | 18230 | 493315b967fc286b7a7dcaadf36e03eb7796777a7c67972940c116b3b2ce0aed | exact |
| 3 | docs/thefinalglance/C-plan/C-23_sessiondag_branch_ui.md | 249 | 14681 | 529b7fab69c5eb5ee27c2544a660874d7985ea83f5950dd610433a0dcc40aec4 | exact |
| 4 | docs/thefinalglance/C-plan/C-31_code_completeness_file_plan.md | 314 | 17772 | a3e0aa3f1eb719f1de4e27671044ad6f59eb1dc2148166c769c28455d1e44ae0 | exact |
| 5 | docs/thefinalglance/C-plan/C-33_docs_phase_map_updates.md | 392 | 22068 | 36f9b52ee5652f5b6b4d1cab9be1eaad0a074078d2fcbcff0b73dd7ada018627 | exact |
| 6 | docs/thefinalglance/C-plan/C-37_per_role_decoding_defaults_reeval.md | 202 | 10275 | 8847c1fc4babe514140a170f777b1a0deefea73b1dc6482de60257f5aef9854d | exact |
| 7 | docs/thefinalglance/C-plan/C-39_mcp_bridge_cli_surface.md | 215 | 10297 | f0a4a122795fa7261252284f200bb92fc3b7dc56fe87f2a21f94bb338da3dcc7 | exact |
| 8 | docs/thefinalglance/C-plan/C-40_github_pr_adapter.md | 267 | 13250 | c452840361dc473d320f4d587499e83091c057c5e18be12e7114df5267b74d7f | exact |
| 9 | docs/thefinalglance/C-plan/C-53_nf4_kernel_watch_in_context_counter_test.md | 152 | 6387 | 396ad0ef90bedd79873a8bc1e84dffc8dd54822ce52375d2fe06a6678c5e92fb | exact |
| 10 | docs/thefinalglance/C-plan/C-55_aider_polyglot_leaderboard_submission.md | 145 | 6600 | 4827f148f2d72b3d7652254721ecfd35cf6caf28a44e1bd7d50a9a318f1144c8 | exact |

Manifest total independently reproduced: **10 unique files, 2,553 logical lines, 136,705 bytes, 10/10 hashes exact, 0 missing, 0 extra, 0 duplicate paths**.

### Record conventions

- E records are explicit source requirements or facts. Each E record is counted once even when Tasks, WIRING_CONTRACT, checks, and ACCEPTANCE repeat it; all relevant line anchors are consolidated.
- I records are inferences, never promoted to source fact.
- A records are ambiguities or underspecified contracts.
- X records are direct internal or cross-source conflicts.
- Delta tags: S = already represented by 03-C-SUMMARY_INDEX.discovery.md; A+ = material addition; C! = correction; Q = qualification/status refinement.
- Line anchors use the immutable, hash-verified source line numbers above.

## C-13 — IPC Dispatcher Production Swap

### Explicit atomic ledger

- **C13-E001 [S]** Objective: replace the two production InProcessToolDispatcher sites with IpcToolRegistry, install production IpcHookDispatcher plus RuffPostToolUseRunner, add AdaptiveTool<T> for ten missing DAIRA markers, and squash WIP commits at AUDITING -> MEMORIZING. [C-13 L1-L6]
- **C13-E002 [A+]** Estimated two sessions; C-11 sandbox completeness is a hard prerequisite; parallel-safe with C-12, C-14, and C-15. [C-13 L7-L9, L293-L295]
- **C13-E003 [A+]** Verified baseline: InProcessToolDispatcher is live at run_subcommand.rs:149 and :221, while IpcToolRegistry appears only in crates/tools/tests/tool_dispatch.rs. [C-13 L13-L17]
- **C13-E004 [A+]** Verified hook baseline: IpcHookDispatcher has no production caller, apply_task_runner_wiring leaves the hook unset, and with_ruff_runner is test-only. [C-13 L18-L20, L29-L30]
- **C13-E005 [C!]** The five pre-existing DAIRA emissions are Call, Return, Error, HookPre, and HookPost; the ten absent emissions are Mutation, Branch, Finding, Read, Write, ToolCallStart, ToolCallEnd, Timeout, Escalate, and Converge. This resolves the summary's unnamed “five pre-existing markers.” [C-13 L22-L25, L88-L89, L271-L275]
- **C13-E006 [A+]** No wip_commit, squash_commit, or WipSquash implementation existed at the verified baseline. [C-13 L27]
- **C13-E007 [S]** RuntimeWiring gains hook_dispatcher: Option<Arc<dyn HookDispatcher + Send + Sync>>, defaults it to None, and apply_task_runner_wiring copies it into TaskRunnerConfig; the path forbids unwrap, expect, and panic. [C-13 L47-L53, L158-L168, L259-L260]
- **C13-E008 [A+]** build_production_hook_dispatcher(ssb: Arc<Ssb>) -> Arc<IpcHookDispatcher> constructs RuffPostToolUseRunner, installs it with with_ruff_runner, and is called by both run_subcommand wiring paths using wiring.ssb.clone(). [C-13 L55-L62, L163-L164, L260-L261]
- **C13-E009 [S]** At run_subcommand.rs:149 and :221, IpcToolRegistry::new(Arc::new(validator)) replaces InProcessToolDispatcher; its existing FSM gate at ipc_registry.rs:112-146 applies automatically. [C-13 L64-L71, L166, L255-L256]
- **C13-E010 [A+]** InProcessToolDispatcher remains available only under cfg(any(test, feature = "in_process_dispatcher")); elai_tools Cargo.toml adds in_process_dispatcher = []; non-test production references are banned. [C-13 L71-L73, L96-L102, L167, L184-L186, L257-L258]
- **C13-E011 [A+]** AdaptiveTool<T: ToolHandler> delegates to an inner handler, is exported from elai_tools::adaptive_tool and the crate root, wraps every handler at IpcToolRegistry::register time, and uses only fallible/if-let emission paths. [C-13 L75-L78, L90-L94, L161, L262-L263]
- **C13-E012 [A+]** AdaptiveTool emits ToolCallStart before dispatch. [C-13 L79-L80]
- **C13-E013 [A+]** A file-read path emits Read. [C-13 L81]
- **C13-E014 [A+]** A file-write path emits Write. [C-13 L82]
- **C13-E015 [A+]** A file-write path additionally emits Mutation only when content changed. [C-13 L82]
- **C13-E016 [A+]** A test-assertion path always emits Branch. [C-13 L83]
- **C13-E017 [A+]** A test-assertion result emits Finding. [C-13 L83]
- **C13-E018 [A+]** A deadline breach emits Timeout. [C-13 L84, L152-L153]
- **C13-E019 [A+]** Escalation/error handling emits Escalate. [C-13 L85, L153]
- **C13-E020 [A+]** A clean converge/done outcome emits Converge. [C-13 L86, L154]
- **C13-E021 [A+]** AdaptiveTool emits ToolCallEnd after dispatch. [C-13 L87]
- **C13-E022 [A+]** AdaptiveTool must not duplicate Call, Return, Error, HookPre, or HookPost; task_runner remains their owner, and Call/Return/Error must each occur exactly once in coverage tests. [C-13 L88-L89, L155-L156, L188-L189]
- **C13-E023 [A+]** squash_wip_commits(project_root: &Path) runs git log --oneline --no-decorate, captures stdout, and counts consecutive leading commit lines matching a WIP subject after the hexadecimal hash. [C-13 L104-L110]
- **C13-E024 [A+]** If the leading WIP count is greater than five, it runs git reset --soft HEAD~{count}, then git commit --no-verify -m "phase: squash {count} WIP commits". [C-13 L111-L112, L142-L143]
- **C13-E025 [A+]** A count of five or fewer is a successful no-op that leaves all commits unchanged. [C-13 L113, L144-L145]
- **C13-E026 [A+]** Every git subprocess captures stderr and maps non-zero exit to anyhow error; on success the SSB payload is {wip_count, squashed: bool}. [C-13 L114-L116]
- **C13-E027 [A+]** The squash runs before handoff at the AUDITING -> MEMORIZING transition. [C-13 L117, L179-L182, L264]
- **C13-E028 [S]** SsbActionType gains WipSquashComplete and WIP_SQUASH_COMPLETE = "wip_squash_complete", following the existing enum/category pattern. [C-13 L119-L123, L165, L265]
- **C13-E029 [A+]** ipc_dispatcher_production_contract.rs defines ten named contracts: test_ipc_tool_registry_is_production_dispatcher; test_in_process_tool_dispatcher_is_cfg_test_only; test_runtime_wiring_has_hook_dispatcher_field; test_apply_task_runner_wiring_sets_hook_dispatcher; test_build_production_hook_dispatcher_returns_ipc_hook_dispatcher; test_ruff_runner_installed_in_production_hook_dispatcher; test_adaptive_tool_emits_tool_call_start_and_end; test_adaptive_tool_emits_read_write_mutation; test_wip_squash_triggers_at_6_wip_commits; test_wip_squash_noop_at_5_or_fewer. [C-13 L125-L145]
- **C13-E030 [A+]** adaptive_tool_daira_coverage_contract.rs defines five named contracts: test_all_15_daira_markers_emitted_across_tool_lifecycle; test_timeout_marker_emitted_on_deadline_breach; test_escalate_marker_emitted_on_error; test_converge_marker_emitted_on_success; test_adaptive_tool_does_not_duplicate_call_return_error. [C-13 L147-L156]
- **C13-E031 [A+]** Consumed contracts are IpcToolRegistry plus its FSM gate, IpcHookDispatcher, RuffPostToolUseRunner, with_ruff_runner, emit_daira_trace, the 15-entry DAIRA_MARKERS constant, and C-11's process boundary. [C-13 L170-L177]
- **C13-E032 [A+]** Reachability is from both elai run / elai /task wiring sites, apply_task_runner_wiring as production chokepoint, and the AUDITING -> MEMORIZING transition. [C-13 L179-L182]
- **C13-E033 [A+]** Stub/security constraints prohibit a production InProcessToolDispatcher, an unset production hook dispatcher, a non-git squash implementation, duplicated base DAIRA markers, and unwrap/expect/panic in new production paths. [C-13 L184-L189]
- **C13-E034 [S]** Cloud-Sonnet smoke implements and tests src/sum.rs, proves IPC-only dispatch, all 15 named DAIRA strings, Ruff hook_post_tool_use on Python writes, and one squash commit when more than five WIPs preceded transition. [C-13 L191-L197, L268-L278]
- **C13-E035 [A+]** Static gates require at least two IpcToolRegistry references, zero run_subcommand InProcessToolDispatcher references, a feature gate, at least three hook_dispatcher references, one builder, one AdaptiveTool struct, at least ten new marker strings, one squash function, and at least two WipSquash references. [C-13 L199-L240]
- **C13-E036 [A+]** Done gates require all 15 contracts and cargo check across elai_tools, elai_orchestrator, elai_cli, and elai_contracts with zero errors. [C-13 L242-L250, L266-L267]
- **C13-E037 [A+]** Planned artifacts comprise seven production-file modifications, adaptive_tool.rs, two contract files, an elai_tools feature declaration, and workflow/reasoning/C-13-decisions.md. [C-13 L280-L291]
- **C13-E038 [Q]** The plan itself records no completed checkbox; its closing instruction still says to confirm C-11 shipped before opening the session. [C-13 L253-L278, L293-L295]

### Inferred

- **C13-I001** After a successful squash, the replacement subject begins with “phase:” rather than “WIP”; rerunning against unchanged history should therefore be an effective no-op, but idempotency is not an explicit contract.
- **C13-I002** The IPC registry's pre-existing FSM gate is intended to preserve authorization ordering across the production swap; the plan does not restate the gate's policy.

### Ambiguous

- **C13-A001** The source does not say whether wip_squash_complete is emitted for the <=5 no-op; “on success” and its squashed boolean suggest yes, while acceptance says it is emitted “on squash.” [C-13 L113-L115, L264-L265]
- **C13-A002** The proposed downcast assertion for Arc<IpcHookDispatcher> does not specify an Any/downcast facility on the HookDispatcher trait. [C-13 L134-L137]
- **C13-A003** Marker classification for arbitrary handlers—how a dispatch is recognized as read, write, assertion, escalation, or convergence—is not specified beyond path labels. [C-13 L75-L93]
- **C13-A004** No locking or re-entry policy protects the git history mutation if multiple transitions target one project concurrently.

### Conflicting

- **C13-X001** build_production_hook_dispatcher is specified to call with_ruff_runner on IpcHookDispatcher, while the Reads/context text associates with_ruff_runner with the existing in-process hook dispatcher/test path; source verification is required before assuming the method exists on the IPC type. [C-13 L18-L20, L37-L40, L55-L60]

## C-15 — ClockProvider, HashMap ban, and entropy quarantine

### Explicit atomic ledger

- **C15-E001 [S]** Initial objective: migrate about 68 production entropy sites, replace std HashMap in ten production files, re-enable Windows entropy CI, add ClockProvider::instant_now, and support reasoned entropy exceptions. The disabled workflow comment still said 16 sites while the audit measured 68; the existing quarantine script already worked. [C-15 L1-L7, L43-L48]
- **C15-E002 [A+]** Estimated two sessions; C-14 is prerequisite so dead HashMap-bearing functions are stripped first; initially declared parallel-safe with C-16 through C-25. [C-15 L9-L12]
- **C15-E003 [A+]** Baseline ClockProvider is object-safe and exposes now -> LogicalInstant, elapsed_since, and system_now -> SystemTime; instant_now was absent at audit and then noted as landing at clock.rs:54 during C-15.1. [C-15 L18-L23]
- **C15-E004 [A+]** SeededClock's AtomicU64 tick starts at zero regardless of ELAI_SEED; ELAI_SEED seeds only the SeedableRng path. [C-15 L50-L52]
- **C15-E005 [A+]** Initial scan found ten production source files importing std::collections::HashMap; rustc-hash 2.1 was already a workspace dependency. [C-15 L25-L29]
- **C15-E006 [C!]** A 2026-05-23 closure rebaseline narrows actual closure to CLI plus knowledge batches; 52 post-closure leaks in orchestrator/plugins/tools/dashboard producer wiring are routed to an unauthored B-15-regression-burndown. [C-15 L31-L41]
- **C15-E007 [A+]** Change ordering is HashMap migration before clock injection, with a separate commit per group/batch for bisectability. [C-15 L54-L56]
- **C15-E008 [A+]** ClockProvider gains object-safe instant_now; OsClock returns Instant::now, while SeededClock uses a lazily established base Instant plus tick milliseconds rather than invalid SystemTime-to-Instant arithmetic. [C-15 L71-L78, L186-L188]
- **C15-E009 [A+]** SeededClock receives an exact documentation contract: ELAI_SEED affects RNG only, the first now call returns tick zero, and correctly injected derived/non-clock fields are byte-identical for the same seed. [C-15 L80-L87]
- **C15-E010 [A+]** Mechanical map migration removes the std import, imports rustc_hash::FxHashMap, replaces the type, and replaces HashMap::new with FxHashMap::default. [C-15 L89-L91]
- **C15-E011 [A+]** The ten reviewed map sites are plugins/tiered_analysis.rs, plugins/import_extractor.rs, dashboard media_validation.rs, focus_tunnel/streaming.rs, focus_tunnel/mode.rs, rules/self_discovery.rs, knowledge/reranker.rs, orchestrator/session_dag.rs, orchestrator/self_mod_layer1a.rs, and media_validator/render_adapters/mod.rs. [C-15 L92-L108]
- **C15-E012 [A+]** Exceptions: leave session_dag's map if test-only and document it under the existing INFO allowlist; do not replace self_mod_layer1a string-template text that merely mentions HashMap. [C-15 L101-L107]
- **C15-E013 [A+]** Map commits are grouped into plugins, dashboard-v2, orchestrator, rules, and knowledge, with C-15.3a through C-15.3e prefixes. [C-15 L109-L111]
- **C15-E014 [A+]** Unavoidable SystemTime sites receive same-line “// entropy-allow: <reason>” only after injection is attempted, plus a file-header manifest of allowed sites; example legitimate classes are filesystem mtime, HTTP timestamp, and TLS validity. [C-15 L113-L128]
- **C15-E015 [A+]** Identified CLI exception candidates are commands.rs cache expiry and live_cli.rs/runtime_cmd.rs SSB timestamp milliseconds; inject through session ClockProvider when possible, otherwise use the specified reason. [C-15 L119-L127]
- **C15-E016 [A+]** Clock injection batches are: A knowledge reranker/indexing; B rules self_discovery; C plugins tiered_analysis/import_extractor; D focus_tunnel streaming/mode; E top-level orchestrator session_dag/self_mod_layer1a/render adapters; F CLI commands/live_cli/remote_sidecar/runtime_cmd/runtime_config. [C-15 L130-L146]
- **C15-E017 [A+]** Injection replaces Instant::now/SystemTime::now with self.clock.instant_now/self.clock.system_now; absent clocks add Arc<dyn ClockProvider> fields and constructor args, except deserializable config types receive injection one call layer above. [C-15 L140-L145]
- **C15-E018 [S]** check_entropy_quarantine.sh filters same-line entropy-allow annotations with grep -v. [C-15 L148-L152, L269-L272]
- **C15-E019 [S]** windows.yml reactivates the entropy quarantine step and documents source annotations plus clock.rs/entropy.rs globs as its allowlist. [C-15 L154-L158, L273]
- **C15-E020 [S]** A separate named Anti-stale entropy scan runs both check_entropy_quarantine.sh and check_hashmap_ban.sh after the quarantine step to block regressions. [C-15 L160-L170]
- **C15-E021 [A+]** clock_provider_injection_contract.rs checks first SeededClock tick zero, identical tick sequences across two instances, SeededClock instant monotonicity from a captured reference, and OsClock instant within one second of Instant::now. [C-15 L172-L182]
- **C15-E022 [A+]** Production bans prohibit increasing post-migration std HashMap count, bare time reads outside allowlists, and edits to tests, main.rs entrypoints, or const/static initialization. [C-15 L203-L209]
- **C15-E023 [C!]** Rebaselined HashMap gate is not zero: it allows at most five violations, all in dashboard-v2 commands_d6/d7/d9/e46/d26 until that dashboard quiesces. [C-15 L220-L224, L263-L265]
- **C15-E024 [C!]** Rebaselined entropy gate is not zero: it allows at most 52 residual sites; the measured 2026-05-23 result was 51. Explicit exclusions/allowances cover measure.rs (16), bench_*.rs (2), live_cli.rs (two cited sites), knowledge/startup.rs, and knowledge/cosmos_worker.rs. [C-15 L226-L236, L266-L272]
- **C15-E025 [A+]** The observable script contract originally says both quarantine scripts exit zero and the combined gate passes on PR. [C-15 L210-L211]
- **C15-E026 [A+]** Determinism smoke runs the same cloud counter task twice at ELAI_SEED=42 and compares byte-identical SSB action_type sequences; raw timestamps are excluded, but derived/non-clock fields must match. [C-15 L277-L282]
- **C15-E027 [Q]** That cloud smoke is CLOUD-BILLABLE-PENDING because live billable calls require user authorization; only harness scaffolding is claimed ready. [C-15 L277-L287]
- **C15-E028 [Q]** Checked closure items are the reduced map/entropy counts, reasoned annotations, annotation filtering, active quarantine step, and the clock contract; the combined anti-stale step remains explicitly missing. [C-15 L263-L276]
- **C15-E029 [Q]** Verification-report updates for B-15, phase-0, and B-16 are explicitly deferred out of C-15 closure. [C-15 L288-L292]
- **C15-E030 [A+]** Additional touched exception files are knowledge/startup.rs and knowledge/cosmos_worker.rs; the latter defers a CosmosClock extension. The script also gains measure.rs and bench_*.rs glob exclusions. [C-15 L312-L315]
- **C15-E031 [A+]** The decisions artifact must record per-file rationale, measure.rs Option A/B treatment, billable-smoke status, and the dashboard carve-out. [C-15 L39-L41, L317]
- **C15-E032 [Q]** Final plan status is partial/mixed rather than wholly PLANNED or DONE: several acceptance boxes are checked, several remain open, and residual cleanup is deferred. [C-15 L258-L292]

### Inferred

- **C15-I001** Once callers use a per-instance SeededClock base and deterministic tick order, instant values can be stable in relative behavior, but byte identity of serialized Instant is neither stated nor generally available.
- **C15-I002** The carve-outs convert the “ban” into a ratcheted budget; future anti-stale behavior depends on the scripts enforcing a non-increasing baseline, which the source does not explicitly specify.

### Ambiguous

- **C15-A001** The contract-test task lists four bullet tests, while acceptance calls them “5 assertions”; the fifth assertion is not named. [C-15 L172-L182, L276]
- **C15-A002** “Major” production entropy scope changed during execution, but the source never publishes a final per-file list of all 51 residual occurrences—only buckets and selected sites. [C-15 L226-L236]
- **C15-A003** The block manifest at the top of a file is documentary only; the actual filter is same-line grep-v, so the source does not define validation that every header entry matches a trailing annotation.

### Conflicting

- **C15-X001** Initial/WIRING language says all ten production maps migrate and the post-Batch-F ban/leak counts reach zero, but the rebaselined deterministic and checked acceptance gates explicitly tolerate five map violations and 51/52 entropy leaks. The later dated rebaseline is the controlling closure claim. [C-15 L3-L7, L189-L207, L220-L236, L263-L275]
- **C15-X002** The observable contract says both scripts exit zero, while checked acceptance says check_hashmap_ban.sh returns FAIL with up to five accepted violations. The combined gate cannot both run that unchanged failing script and pass. [C-15 L210-L211, L263-L275]
- **C15-X003** Context first says instant_now does not exist and then records it as already landed in C-15.1; this is a time-layered plan, not a single clean pre-implementation state. [C-15 L18-L23]

## C-23 — SessionDag Branch UI

### Explicit atomic ledger

- **C23-E001 [S]** Objective: close the B-9 partial/missing surface by adding merge/close CLI actions, live BranchTree, real Tauri mutations, load_handoff, root-to-tip ancestry, 100-run determinism, handoff-schema verification, and branch-event banners. [C-23 L1-L6]
- **C23-E002 [A+]** Estimated one to two sessions; C-3 and C-4 are prerequisites; parallel-safe with C-21, C-22, and C-24. [C-23 L7-L9]
- **C23-E003 [A+]** Verified gaps are: parser supports only list/switch/fork; BranchTree is unmounted; three Tauri commands return CLI-delegation errors because AppState lacks SessionDag; /continue only prepopulates; parent_chain is reversed; and only a two-seed determinism comparison exists. [C-23 L13-L31]
- **C23-E004 [A+]** parse_session_command adds exact-arity merge and close arms, returns usage errors when the target is absent, sets action plus branch_id, extends argument_hint, and routes both actions through the live REPL matcher. [C-23 L41-L49, L203-L204]
- **C23-E005 [A+]** fork_session_branch resolves the DAG from AppState, calls create_branch(name), and emits an SSB Checkpoint through ssb_writer. [C-23 L51-L56]
- **C23-E006 [A+]** merge_session_branch calls the async merge_branch_with_ssb(branch_id, ssb_writer). [C-23 L57, L190-L191]
- **C23-E007 [A+]** close_session_branch calls close_branch(branch_id) and emits an SSB Checkpoint. [C-23 L58]
- **C23-E008 [A+]** All three Tauri mutations return typed DagBranchError, never raw strings. [C-23 L59]
- **C23-E009 [C!]** AppState must expose either a DAG handle or a configured dag_db_path: Option<PathBuf>; if absent, populate the path from elai_cc.toml session.dag_db_path and never hardcode it. The summary's unconditional “holds a DAG handle” is too narrow. [C-23 L21-L23, L122-L123, L185-L189]
- **C23-E010 [S]** SessionsDrawer imports and mounts BranchTree beneath/replacing the branch list, passes $sessionBranches, invokes fork/merge/close with branchId payloads, and refreshes the list after success. [C-23 L61-L69]
- **C23-E011 [A+]** CommandWindow /continue invokes load_handoff and pushes a success status card; invocation failure restores the previous prepopulation behavior and adds an error card; stub.ts is extended only if needed. [C-23 L71-L77, L209-L210]
- **C23-E012 [S]** parent_chain adds chain.reverse and changes all assertions/callers to [snap-root, snap-mid, snap-leaf]. [C-23 L79-L84, L195-L197, L211]
- **C23-E013 [A+]** The determinism contract repeats create_handoff plus load_handoff 100 times on identical input with a fixed SeededRng and requires identical snap_ids and trim output; if C-18 already proves it, the result may be documented DONE instead. [C-23 L86-L92, L198-L199, L212-L213]
- **C23-E014 [A+]** SettingsWindow is grep-audited for SessionHandoffEntry/HandoffEntry/handoff_entry; stale V1 fields such as chat_window_state are either absent and documented DONE or aligned with the Rust struct. [C-23 L94-L99, L214]
- **C23-E015 [S]** Mutations emit branch_forked, branch_merged, and branch_closed through AppState ssb_writer, following the existing branch_merge/checkpoint pattern. [C-23 L101-L106, L110-L116]
- **C23-E016 [S]** ActivityWindow or SessionsDrawer renders banners for all three new branch action types. [C-23 L105-L106, L215]
- **C23-E017 [A+]** Consumed symbols are create_branch, merge_branch_with_ssb, close_branch, load_handoff, get_session_branches, AppState ssb_writer/DAG configuration, and BranchTree callback props. [C-23 L117-L124]
- **C23-E018 [A+]** Reachable surfaces are /session merge <branch_id>, /session close <branch_id>, SessionsDrawer BranchTree, and CommandWindow /continue. [C-23 L125-L128]
- **C23-E019 [A+]** Stub bans remove every “use CLI: /session …” return and forbid a prepopulate-only /continue path. [C-23 L129-L136]
- **C23-E020 [A+]** The observable lifecycle requires Checkpoint plus branch_* rows, live BranchTree state, and actual load_handoff behavior. [C-23 L137-L142]
- **C23-E021 [A+]** Deterministic/static evidence checks parser arms, absence of stub text, BranchTree import/mount, load_handoff, chain.reverse, root index zero, a 100-loop, all branch action strings, and both session-dag test commands. [C-23 L144-L183]
- **C23-E022 [A+]** Frontend compatibility must verify Svelte 5 $props callback syntax; parent-chain consumers must be swept before reordering. [C-23 L192-L197]
- **C23-E023 [S]** Cloud smoke runs a 20-file task past 85% context, observes create_handoff, resumes in another terminal through /continue/load_handoff, then exercises fork/merge/close while the live drawer reflects state. [C-23 L219-L226]
- **C23-E024 [A+]** Planned file surface is CLI commands, session_dag.rs, Tauri commands, SessionsDrawer, CommandWindow, SettingsWindow verify/modify, optional session_dag_contract.rs, and C-23-decisions.md. [C-23 L228-L244]
- **C23-E025 [Q]** Every acceptance checkbox is open; the 100-run test may be delegated/documented to C-18, and the next step may be C-24 or C-18. [C-23 L201-L218, L246-L249]

### Inferred

- **C23-I001** Refresh-after-success makes UI mutations eventually consistent with SessionDag; no optimistic update is specified.
- **C23-I002** Typed DagBranchError is intended to preserve machine-readable failure semantics across Tauri, although its serialization format is not specified.

### Ambiguous

- **C23-A001** BranchTree's onfork callback receives a name but the invocation payload key is branchId; the Tauri parameter name/rename contract is not reconciled. [C-23 L54-L56, L65-L68]
- **C23-A002** The 100 iterations do not say whether state is reset or each run gets a fresh DAG; repeated writes to one DB could intentionally change snap IDs. [C-23 L86-L92]
- **C23-A003** load_handoff is invoked with no named session/branch argument in the plan; selection semantics are left to the existing command.
- **C23-A004** branch_merged may be emitted inside merge_branch_with_ssb and again in the wrapper task; the source does not define a single emission owner. [C-23 L57, L101-L106]

### Conflicting

- **C23-X001** The smoke orders fork foo -> merge foo -> close foo, but the plan never states whether a merged branch remains closeable; branch lifecycle/state-transition semantics are not defined. [C-23 L138-L142, L224-L225]
- **C23-X002** The Files section calls the SettingsWindow check “Task 8,” although it is Task 7; Task 8 is SSB banners. [C-23 L94-L101, L240-L241]

## C-31 — Code-completeness Layer 1: FilePlan and file accounting

### Explicit atomic ledger

- **C31-E001 [S]** Objective: add FilePlan/FileSpec/FileModSpec/FunctionSpec, a BabyStepSpec FILE_PLAN field, crash-persistent cumulative LOC with a generation guard, MistakeBook recording from T1/T2 BLOCKs, T2 function-count Check 3, and ProjectIntelligence-derived file limits. [C-31 L1-L7]
- **C31-E002 [A+]** Estimated two to three sessions; depends on C-13 for stable mistake-book wiring and C-9 for the autonomous gate; declared parallel-safe with C-32 because file scope is disjoint. [C-31 L8-L11]
- **C31-E003 [A+]** Baseline reports all Layer-1 plan/tracker/guard pieces absent, while FailureCategory::LazyCode, EmptyBody, and FileSizeExceeded and SSB-backed mistake storage already exist but are not called by T1/T2 BLOCK paths. The implementation-order report also says ModuleTree and tier-gated file-count limits are absent, but this plan assigns neither an implementation task. [C-31 L13-L25, L33-L43]
- **C31-E004 [C!]** Source text says BabyStepSpec has eight current fields but explicitly enumerates nine—id, task, estimated_tokens, grounding, tdd, acceptance, preferred_mode, target_file, edit_region—before calling FILE_PLAN the ninth. [C-31 L27-L31]
- **C31-E005 [A+]** Existing LanguageLimits defaults are Rust 400, Go 400, Python 480, TypeScript/JavaScript 600, and Java/C# 800; FileDisciplineConfig currently has only cyclomatic-complexity and coupling controls. [C-31 L38-L46]
- **C31-E006 [A+]** FunctionSpec schema is {name: String, signature: String, max_body_lines: u8}. [C-31 L65-L67]
- **C31-E007 [A+]** FileSpec schema is {path: PathBuf, language: String, max_lines: u16, functions: Vec<FunctionSpec>}. [C-31 L67-L68]
- **C31-E008 [A+]** FileModSpec schema is {path: PathBuf, language: String, max_added_lines: u16, functions: Vec<FunctionSpec>}. [C-31 L69-L70]
- **C31-E009 [A+]** FilePlan schema is {new_files: Vec<FileSpec>, modified_files: Vec<FileModSpec>, version: u8}. [C-31 L71-L72]
- **C31-E010 [A+]** All four schemas derive Debug, Clone, Serialize, and Deserialize; fallible constructors return FilePlanError and production paths forbid unwrap. [C-31 L73-L76, L265-L266]
- **C31-E011 [A+]** FilePlanError is a thiserror enum with LineBudgetExceeded {file, limit: u16, got: u16} and FunctionCountExceeded {file, limit: u8, got: u8}. [C-31 L74-L76]
- **C31-E012 [A+]** FilePlan::validate checks new max_lines and modified max_added_lines against LanguageLimitsMap by language, falls back to global 400, and rejects more than MAX_FUNCTIONS_PER_FILE = 20 functions per file. [C-31 L89-L97]
- **C31-E013 [A+]** FilePlan must survive a serde_json serialize/deserialize equality round trip. [C-31 L95-L97, L269]
- **C31-E014 [S]** BabyStepSpec appends #[serde(default)] file_plan: Option<FilePlan> after edit_region; old JSON remains readable without a schema bump, and both parse_research_tree construction sites explicitly set None. [C-31 L78-L87, L270-L273]
- **C31-E015 [S]** TreeNavigatorState adds cumulative_lines_by_step: HashMap<BabyStepId,u32>, records LOC at the record_result completion site, exposes total_lines_written as the sum, and serializes state for crash recovery. [C-31 L99-L110, L274-L276]
- **C31-E016 [A+]** The TOCTOU contract adds monotonic step_write_generation: u64, increments before a write, checks that it is unchanged after, and returns NavigatorError::WriteGeneration on mismatch through record_lines_written. [C-31 L103-L109]
- **C31-E017 [S]** MistakeRecorder is a synchronous injectable trait with record_mistake_event(baby_step_id: &str, category: FailureCategory, file_path: &Path, snippet: &str); default behavior is no-op, while production task_runner wiring must call write_mistake_entry. [C-31 L112-L125, L206-L208]
- **C31-E018 [A+]** T1HookConfig and T2HookConfig receive Option<Arc<dyn MistakeRecorder + Send + Sync>>, default None. [C-31 L124-L125, L183-L185, L277-L278]
- **C31-E019 [S]** BLOCK recording maps lazy placeholders to LazyCode, no statements to EmptyBody, and oversize files to FileSizeExceeded; acceptance specifically requires T1 LazyCode, T1 FileSizeExceeded, and T2 EmptyBody calls. [C-31 L112-L123, L279-L282]
- **C31-E020 [A+]** T2HookConfig also receives file_plan: Option<Arc<FilePlan>>; Check 3 executes after Check 2 and before Check 4. [C-31 L127-L131]
- **C31-E021 [A+]** Check 3 matches the current path to FileSpec, counts Rust item_function or Python function_definition AST nodes, and BLOCKs when actual count exceeds planned functions plus FUNCTION_COUNT_SLACK = 2. [C-31 L131-L139]
- **C31-E022 [A+]** The exact BLOCK message is “function count {actual} exceeds FILE_PLAN limit {limit}+2 in {file}”; equality at plan+2 is allowed; no FilePlan silently skips the check. [C-31 L135-L139, L166-L168, L283-L284]
- **C31-E023 [A+]** FileDisciplineConfig adds language_limit_overrides: HashMap<String,LanguageLimits>; matching entries override hard-coded defaults. [C-31 L141-L145]
- **C31-E024 [A+]** task_runner derives overrides only for languages with detected_file_count >=5: production_warn = min(spec_default, median LOC * 1.5 as u16), hard_block = min(spec_hard_block, p95 LOC * 1.2 as u16); missing ProjectProfile leaves defaults intact. [C-31 L145-L151, L285-L286]
- **C31-E025 [A+]** c31_file_plan_contract.rs specifies sixteen named cases: type construction; JSON round trip; accept within limit; reject over limit; reject >20 functions; absent/populated FILE_PLAN round trips; legacy JSON; tracker total; generation mismatch; Check 3 excessive functions; no-plan skip; slack-two allow; LazyCode recorder; FileSizeExceeded recorder; language override precedence. [C-31 L153-L171]
- **C31-E026 [A+]** Produced public surface also includes FilePlanError, FilePlan::validate, the two tracker methods, step_write_generation, MistakeRecorder, both hook-config fields, both named constants, and the override map. [C-31 L173-L188]
- **C31-E027 [A+]** Consumed contracts are the three existing mistake categories, write_mistake_entry, BabyStepSpec, TreeNavigatorState, LanguageLimits, T1/T2 BLOCK paths, and ProjectProfile.language_stats. [C-31 L190-L197]
- **C31-E028 [A+]** Reachability is LLM BabyStepSpec JSON, task_runner config construction, T1/T2 hot-path analysis, TreeNavigator::record_result, and a read-only PlanWindow FILE_PLAN block. [C-31 L199-L205, L228]
- **C31-E029 [A+]** Stub bans require real SSB mistake persistence, AST-derived function counts, an enforced generation mismatch, and rejection rather than an always-pass FilePlan validator. [C-31 L206-L213]
- **C31-E030 [S]** Cloud CRUD smoke requires a populated plan for each step, cumulative line tracking, evidence for any lazy offense, and workspace compilation; an adversarial Rust plan above 400 LOC must fail before baby-step dispatch and emit no BabyStepSpec. [C-31 L215-L228, L287-L293]
- **C31-E031 [A+]** Done gates are all sixteen contracts, an explicit oversize test, workspace cargo check, no new warnings, and the read-only PlanWindow surface. [C-31 L230-L261, L294-L295]
- **C31-E032 [A+]** Planned artifacts are core/file_plan.rs, two orchestrator edits, tiered_analysis.rs, c31_file_plan_contract.rs, and C-31-decisions.md. [C-31 L297-L309]
- **C31-E033 [Q]** All acceptance boxes remain open; the plan describes intended work, not verified delivery. [C-31 L263-L295]

### Inferred

- **C31-I001** FilePlan validation is a pre-dispatch admission-control layer; runtime tracker and T1/T2 checks are defense-in-depth after admission.
- **C31-I002** serde(default) intentionally favors backward read compatibility, but forward readers that do not know file_plan may drop it; no forward-compatibility policy is stated.

### Ambiguous

- **C31-A001** “FileLineTracker” is named as a feature, but no FileLineTracker struct is defined; the fields/methods are placed directly on TreeNavigatorState. [C-31 L4-L6, L99-L110, L180-L182]
- **C31-A002** LanguageLimitsMap is defined as String -> u16, yet acceptance speaks of a language hard_block and the override path uses a richer LanguageLimits type; which threshold validate consumes is unclear. [C-31 L89-L95, L141-L151, L267-L268]
- **C31-A003** The logical generation guard holds &mut self during record_lines_written but does not define acquisition/return of a token or an async boundary; the described concurrent mismatch mechanism is incomplete. [C-31 L103-L109]
- **C31-A004** The observable count “>=0 MistakeBook entries” is vacuous and cannot prove wiring when no offense occurs. [C-31 L215-L222, L287-L290]
- **C31-A005** FunctionSpec.max_body_lines is modeled but no validator or runtime check for it is specified. [C-31 L65-L76, L89-L97]

### Conflicting

- **C31-X001** Adding file_plan after the nine enumerated existing fields makes it the tenth field, not the ninth; the summary repeats the source's “ninth” label and should not be used for schema-position logic. [C-31 L27-L31, L78-L87]
- **C31-X002** The test file is under crates/contracts, but the deterministic command names package elai-orchestrator and acceptance names c31_file_plan_contract without reconciling the actual Cargo package. [C-31 L153-L155, L251-L253, L294]
- **C31-X003** The task text says both T1 and T2 record all three BLOCK categories, acceptance assigns specific categories per hook, and cloud smoke refers to “T2 LazyCode”; exact hook/category ownership is inconsistent. [C-31 L112-L125, L279-L290]

## C-33 — Documentation and phase-map reconciliation

### Explicit atomic ledger

- **C33-E001 [S]** Objective: create four missing phase specs, append two shipped-extension addenda, add five B-plan status blocks, extend failure taxonomy by three variants, reconcile B-39/B-21, and replace a vague slash-command manifest entry with per-command decisions. [C-33 L1-L5]
- **C33-E002 [A+]** Estimated one to two sessions; requires all other C-N files to exist and C-30 for the Surgical implementation reference; claimed parallel-safe with every C-N because failure_taxonomy.rs is the only code touch. [C-33 L6-L10]
- **C33-E003 [S]** docs/specs is gitignored: all four new specs and both edits are local-only, uncommitted, and must be called out in the session handoff. [C-33 L12-L13, L367-L368]
- **C33-E004 [A+]** Baseline discrepancies are four missing spec files, two specs missing already-shipped behavior, absent B-37/B-38/B-40/B-41/B-42 index blocks, and a stale “SCHEMA_VERSION 26” against code version 46. [C-33 L15-L24]
- **C33-E005 [A+]** phase-3L Step 1 defines ConventionStats/NamingConventions, public extract_conventions, four SQL pattern queries, and module-init regex compilation. [C-33 L69-L75]
- **C33-E006 [A+]** phase-3L Step 2 puts conventions in DKG node metadata and migrates an existing conventions.db schema. [C-33 L75-L77]
- **C33-E007 [A+]** phase-3L Step 3 tests Rust, Python, TypeScript, and Svelte extraction and deterministic output for identical input. [C-33 L77-L78]
- **C33-E008 [A+]** phase-3L Step 4 has T1Hook load rules.db conventions and inject a CONVENTIONS block into the prompt SYSTEM section through a ConventionStats summary. [C-33 L79-L80]
- **C33-E009 [A+]** phase-3L Step 5 adds T2 AST import extraction, ProjectRuleSeeder-generated rules.db rules, and VERSION_GATE. [C-33 L81-L82]
- **C33-E010 [A+]** phase-3L Step 6 aggregates ConventionStats, FreshnessConfig, and ProjectClass into ProjectProfile stored in knowledge/project_profile.rs. [C-33 L83-L84]
- **C33-E011 [A+]** phase-3L Step 7 integrates the context telescope and orders convention signals by token cost versus recall, with pi_telescope_contract and pi_tier3. [C-33 L85-L86]
- **C33-E012 [A+]** phase-3L Step 8 calibrates convention extraction to p50/p99 <=50ms and writes metrics/phase_3L.toml; target spec length is 80-120 lines. [C-33 L87-L89]
- **C33-E013 [A+]** phase-3J-BASELINE specifies standalone benches/model_capability/run_benchmark.py with a fixture TOML containing at least ten prompts per ModelCapabilityProfile tier. [C-33 L91-L96]
- **C33-E014 [A+]** Each baseline fixture records total_tokens_in, total_tokens_out, slot_utilization_pct, ShortCoder compression_ratio, ttft_ms, p99_latency_ms, and cost_per_run_usd. [C-33 L96-L98]
- **C33-E015 [A+]** The runner reads [model_capability] from elai_cc.toml and emits metrics/capability_profile.toml. [C-33 L99-L100]
- **C33-E016 [A+]** Baseline floors are micro_7b_12gb slot utilization 0.82 and compression 0.65, and cloud_200k slot utilization 0.90. [C-33 L101-L102]
- **C33-E017 [A+]** REPAIR_CONTEXT may not increase total_tokens_in by more than 15% over the same-fixture non-repair baseline. [C-33 L103-L104]
- **C33-E018 [A+]** A 20-prompt JSON-schema constrained-decoding fixture must yield zero invalid JSON outputs. [C-33 L105-L106]
- **C33-E019 [A+]** The benchmark runs without elai runtime, SSB, or DKG and writes TOML only; target spec length is 60-90 lines. [C-33 L107-L109]
- **C33-E020 [A+]** phase-4D-SURGICAL records SurgicalPlan, AffectedSet, ComplexityClass, RegressionGate, TestBaseline, and SurgicalStep from C-30. [C-33 L111-L115]
- **C33-E021 [A+]** SurgicalDecomposer traverses DKG blast radius and classifies Simple as files<=3 AND LOC<=200, Medium as files<=8 OR LOC<=500, otherwise Complex; step caps are 1/4/8 respectively. [C-33 L116-L120]
- **C33-E022 [A+]** Each surgical step receives RegressionGate from DKG coverage; >500 LOC emits surgical_loc_warning and requires interactive confirmation, while autonomous >1000 LOC escalates. [C-33 L120-L125]
- **C33-E023 [A+]** Surgical CLI/evidence is --surgical, surgical_plan_created {complexity_class, blast_radius_files}, surgical_loc_warning {loc_estimate, threshold}, and an rg check; target spec length 80-120. [C-33 L121-L127]
- **C33-E024 [A+]** phase-4D-ORACLE records /research -> run_research_pipeline and CacheKey {query_hash:[u8;32], project_hash:[u8;32]} using blake3 plus CachedAnswer {answer,cached_at,source_tier}. [C-33 L129-L135]
- **C33-E025 [A+]** Oracle TTLs are T1Authoritative 2,592,000s, T2Strong 604,800s, T3Weak 86,400s, and T4Rejected zero/never cache. [C-33 L135-L136]
- **C33-E026 [A+]** Cache miss runs research, maps HIGH/MEDIUM/LOW to T1/T2/T3, inserts, and emits oracle_cache_miss; cache hit returns the answer and emits oracle_cache_hit {cached_age_secs,source_tier}. [C-33 L137-L140]
- **C33-E027 [A+]** Oracle eviction is LRU above configurable max_entries; required action types are oracle_cache_hit/miss, its check greps OracleAnswerCache, and target spec length is 70-100. [C-33 L141-L144]
- **C33-E028 [A+]** The append-only 3J addendum documents ETCLogitsProcessor entropy stopping/calibration; GG mean_logprob and entropy_mean consumed by research confidence; ConstrainedDecodingConfig grammar/JSON modes; routing_rules; bash safety_filter; git_commit; action_classifier; and steering-vector --batch plus contrast-pair data. [C-33 L146-L161]
- **C33-E029 [A+]** The append-only 4G addendum makes derive_framing_evidence the seventh AutoAdvanceEvaluator and declares exhaustion at evidence_rounds >= evidence_rounds_max. [C-33 L163-L169]
- **C33-E030 [A+]** CloudSpendTracker emits cloud_spend_warning at 50% spent, cloud_spend_critical at 20% remaining, and below 20% escalates INTERACTIVE or halts AUTONOMOUS, using [cloud.session_budget_usd]. [C-33 L169-L174]
- **C33-E031 [S]** tools/failure_taxonomy.rs adds LazyCode, EmptyBody, and FileSizeExceeded after RuntimePanic, making 12 variants, and adds complete detect_category/suggest_fix arms. [C-33 L176-L188]
- **C33-E032 [A+]** Detection patterns are TODO|placeholder|unimplemented!|todo!() for LazyCode, “empty function body|no statements” for EmptyBody, and “file exceeds|FILE_SIZE” for FileSizeExceeded. [C-33 L183-L187]
- **C33-E033 [A+]** ToolFailureCategory in elai_core must receive the same variants, but only after locating it through elai.map. [C-33 L189-L190, L284-L288]
- **C33-E034 [S]** B-39 is corrected from cap two to REFINE_CAP = 3, citing RefineBench arXiv:2511.22173 Table 3's reported +12-20pp from t=2 to t=3 and the existing equality contract. [C-33 L26-L29, L192-L198]
- **C33-E035 [S]** B-21 action reconciliation maps feasibility_probe_started -> system_probe, feasibility_probe_complete -> feasibility_report, alternative_explored -> stub_run as nearest shipped equivalent, and marks feasibility_escalated absent/deferred to C-22. [C-33 L31-L36, L200-L210]
- **C33-E036 [A+]** The B-21 divergence note also says novel_step_detected matches the plan and must be retained in the accepted action list. [C-33 L208-L210, L361-L362]
- **C33-E037 [A+]** B-37's new index block is DONE at fd5c2e5 with RuntimeAdapter, nine adapters, PerRoleRuntime, RuntimeOrchestrator, /runtime, RuntimePanel, and final schema 46. [C-33 L212-L218]
- **C33-E038 [A+]** B-38's block cannot be fixed from this plan alone: consult MEMORY.md/auditor and choose SHIPPED or PARTIAL. [C-33 L219-L221]
- **C33-E039 [A+]** B-40's block is DONE with idea_framing.rs 1358 LOC, Framing substate, five actions, IdeaFramingWindow, 51 tests, schema 40->41, and code citation arXiv:2409.04109; its original plan retains wrong arXiv:2407.21787 as debt. [C-33 L38-L40, L222-L225]
- **C33-E040 [A+]** B-41's block is DONE with auditor_mode, per_level_audit_dispatch, AgentRole+5, Evidence+5, AuditTimelineWindow, schema 45, 80 green tests, and navigator-spawn deferred. [C-33 L226-L228]
- **C33-E041 [A+]** B-42's block is DONE with about 700 LOC CosmosWorker, briefing compiler, CosmosCheck, schema 44, CosmosDrawer, and spawn_loop dispatcher deferred. [C-33 L229-L231]
- **C33-E042 [A+]** B-0 replaces “SCHEMA_VERSION=26 confirmed” with current 46 and records B-6->26, B-24->30, B-34->35, and later B-38/B-42/B-41 progression through 44/45/46 with ssb.rs history as authority. [C-33 L232-L235]
- **C33-E043 [A+]** Migration manifest explicitly classifies KEEP: /task, /research, /mode, /model, /runtime, /reason, /plan, /memory, /context, /search, /audit, /help, /status, /log, /export, /import, /reset, /history, /session, /config, /budget, /branch, /checkpoint, /supervise, /agents, /tools, /version, /health. [C-33 L237-L250]
- **C33-E044 [A+]** It classifies STRIP: /debug, /trace, /benchmark, /mock; /resume and /cancel are KEEP-pending until live_cli dispatch is verified; no command may be invented. [C-33 L251-L256]
- **C33-E045 [A+]** Reachability/done evidence is spec_link_check on new/edited specs, wiring_contract_check on B-21/B-39, index greps, taxonomy compile, and manifest comparison to commands.rs. [C-33 L258-L347]
- **C33-E046 [A+]** Non-goals forbid edits to phase specs 1A, 1E, 3A, 3C, 3E, 3F, 4B, 4C, 4E, and 4H. [C-33 L284-L287]
- **C33-E047 [C!]** The migration task demands the entire verified command list, although acceptance weakens the machine gate to at least ten named entries. [C-33 L237-L256, L365-L366]
- **C33-E048 [Q]** All acceptance boxes remain open; the optional cloud /research discovery check is non-blocking, and local-only file delivery must be flagged. [C-33 L349-L370]
- **C33-E049 [A+]** Planned writes comprise six local-only spec files/edits, taxonomy code, three B-plan docs, B-0_INDEX, migration_manifest, and C-33-decisions.md. [C-33 L372-L387]
- **C33-E050 [A+]** After all C-N work, the documented terminal FSM path is MEMORIZING -> COMPLETE -> IDLE. [C-33 L389-L392]

### Inferred

- **C33-I001** C-33 is a documentation/schema-reconciliation release gate: most behavior is already implemented elsewhere, while this plan makes specifications and evidence discoverable.
- **C33-I002** Because specs are gitignored, durable team delivery requires an out-of-band handoff or a repository policy change; merely creating them cannot preserve them in normal commits.

### Ambiguous

- **C33-A001** B-38 status is deliberately unresolved, so “add five status blocks” cannot be deterministic until MEMORY/auditor evidence is available. [C-33 L219-L221]
- **C33-A002** The exact schema-version progression phrase “B-38/42/41 to 44/45/46” does not uniquely map every plan to every intermediate version and omits the final B-37C bump mechanics. [C-33 L232-L235]
- **C33-A003** alternative_explored -> stub_run is explicitly only the “nearest shipped equivalent”; semantic equivalence is not proven. [C-33 L200-L209]
- **C33-A004** The command source comment says slash_command_specs “assert 149,” but the enumerated manifest candidates total 34, and the plan does not explain what 149 counts. [C-33 L237-L256]

### Conflicting

- **C33-X001** Context's list of B-21 “plan claims” omits novel_step_detected, while the requested divergence note says novel_step_detected matches the plan. [C-33 L31-L36, L208-L210]
- **C33-X002** The header says the only code touch is failure_taxonomy.rs, yet it also changes migration_manifest.toml and conditionally the elai_core ToolFailureCategory surface; “pure docs/spec” is only approximately true. [C-33 L7-L10, L189-L190, L237-L257]
- **C33-X003** Summary orientation names C-22 as a prerequisite, but the original header requires all C-N files to exist and specifically names C-30; C-22 is only the owner of a deferred B-21 event. [C-33 L7-L10, L200-L210]

## C-37 — Per-role decoding defaults re-evaluation

### Explicit atomic ledger

- **C37-E001 [S]** Objective: experimentally re-evaluate C-26's critic/auditor JsonSchema defaults against structure-snowballing evidence before C-54 and either keep, relax, or flip the auditor mode. [C-37 L1-L7]
- **C37-E002 [S]** C-26 must be shipped; work is parallel-safe with C-35/C-36/C-38/C-39/C-40 but must precede C-54 so publication uses the measured default. [C-37 L9-L12]
- **C37-E003 [A+]** Grounding is arXiv:2604.06066, dated 2026-04-07 and checked 2026-05-17, plus github.com/hongxuzhou/agentic_llm_structured_self_critique; the claimed 8B failure mode is superficial syntactic alignment without semantic correction. [C-37 L14-L22]
- **C37-E004 [A+]** Current C-26 defaults are critic=JsonSchema and auditor=JsonSchema in elai_cc.toml; structured downstream parsing was the motivation. [C-37 L24-L31]
- **C37-E005 [A+]** Existing anti-rubber-stamp FINDING may mitigate the paper's result, but the source labels that a hypothesis and forbids a config flip before measuring Qwen2.5-Coder-7B with ELAI's actual prompt. [C-37 L33-L40]
- **C37-E006 [A+]** paper-ingestor runs in paper mode for the exact contract-test shape and source mode for the GitHub JsonSchema-vs-FreeText harness, producing structure-snowballing-interface.md plus a source interface map. [C-37 L51-L57, L161-L163]
- **C37-E007 [S]** structure_snowballing.rs uses intentional off-by-one, null-deref, and wrong-operator bugs; it runs the same fixture in JsonSchema and FreeText and outputs per-mode detection rates. [C-37 L59-L64]
- **C37-E008 [Q]** The Qwen2.5-Coder-7B NF4+FA2 pilot and detection-rate delta are GPU/RunPod-dependent and deferred to the C-54-adjacent window. [C-37 L65-L67, L164-L166]
- **C37-E009 [A+]** If FreeText beats JsonSchema by at least 10 percentage points, auditor flips to FreeText while critic stays JsonSchema. [C-37 L69-L71]
- **C37-E010 [A+]** If FreeText wins by more than zero but less than 10 points, add LightlyConstrained: require a non-empty findings array but do not constrain each finding's internal structure; auditor flips to it. [C-37 L71-L74]
- **C37-E011 [A+]** If FreeText loses to JsonSchema, retain defaults, document FINDING as mitigation, and close C-37. [C-37 L74-L75]
- **C37-E012 [C!]** A decoding-default migration script is conditional on a flip: rewrite existing [decoding.per_role] blocks once, preserve explicit overrides, and change only users who retained the old default. [C-37 L77-L80, L168-L169]
- **C37-E013 [A+]** Add decoding_mode_pilot_run for each pilot and decoding_default_flipped only when applicable; payload fields are role, old_mode, new_mode, and detection_rate_delta; schema bumps only if a default flips. [C-37 L82-L84, L173]
- **C37-E014 [A+]** TDD gates are: both modes produce a rate; migration is idempotent; and the anti-rubber-stamp FINDING requirement still fires under FreeText. [C-37 L86-L93]
- **C37-E015 [A+]** DETAILED_SUMMARY_01's C-26 section receives a dated session-artifact link recording “ratified/flipped” outcome. [C-37 L95-L97]
- **C37-E016 [A+]** decoding_role_contract.rs proves PER_ROLE_DECODING_DEFAULTS exposes the auditor mode through SSB and outside the runtime crate. [C-37 L99-L102]
- **C37-E017 [A+]** Same-fixture/same-session comparison is mandatory; the decision matrix is deterministic, and any human override requires an SSB reason; FINDING must survive every mode. [C-37 L120-L125]
- **C37-E018 [A+]** There is no required frontend; observable evidence is one decoding_mode_pilot_run row per synthetic bug per mode and a side-by-side pilot.md with per-bug table and summary delta. [C-37 L126-L134]
- **C37-E019 [A+]** Static/done checks require the harness, executable dry-run/idempotent migration script, both action strings, current-session pilot artifact, anti-rubber-stamp test, and decoding-role contract. [C-37 L136-L157]
- **C37-E020 [S]** Either “flip” or “keep” is an accepted closing result; closure is the measured pilot artifact, not a particular preferred default. [C-37 L159-L177]
- **C37-E021 [Q]** Planned writes include the harness, contract, conditional migration/config/runtime changes, two SSB actions, paper/session artifacts, summary annotation, and a C-26 verification-report note; all acceptance boxes are open. [C-37 L159-L194]
- **C37-E022 [A+]** pilot.md becomes the canonical public methodological reason for the auditor's future decoding mode and feeds C-54. [C-37 L196-L202]

### Inferred

- **C37-I001** The plan treats semantic-bug detection rate, not syntax validity, as the decision metric; downstream parser reliability remains an unmeasured trade-off.
- **C37-I002** Keeping critic JsonSchema in the >=10pp branch assumes critic and auditor roles are behaviorally distinct enough not to share the result; that distinction is asserted, not separately tested.

### Ambiguous

- **C37-A001** A zero-point tie is not covered by any branch: branches are >=10, >0 and <10, or <0. [C-37 L69-L75]
- **C37-A002** Fixture cardinality N, exact semantic-success oracle, sampling parameters, seeds, repetitions, and confidence intervals are unspecified. [C-37 L59-L67, L129-L134]
- **C37-A003** Adding two new action types normally changes a persisted schema, but the plan says schema bump only if defaults flip even though decoding_mode_pilot_run exists in every outcome. [C-37 L82-L84]
- **C37-A004** The migration script is listed as an unconditional produced/file artifact while Tasks and ACCEPTANCE make it conditional. [C-37 L77-L80, L106-L111, L168-L169, L181-L187]
- **C37-A005** The documentation task says “ratified/flipped,” but acceptance only requires the summary edit on flip; a keep/ratification outcome's documentation gate is unclear. [C-37 L95-L97, L170]

### Conflicting

- **C37-X001** No direct numerical conflict exists, but the plan's “deterministic” decision does not specify deterministic model execution; same-session fixtures alone do not control stochastic decoding. [C-37 L120-L124]

## C-39 — MCP bridge CLI surface

### Explicit atomic ledger

- **C39-E001 [S]** Objective: let CLI task and explore invoke registered MCP tools without the dashboard, using the existing HTTP JSON-RPC 2.0 bridge semantics for transport parity with Apache-2.0 OpenHands (cited at about 38K stars). OpenHands supplies the fastmcp HTTP pattern, not a standalone CLI architecture. [C-39 L1-L7, L15-L22]
- **C39-E002 [S]** Requires shipped Phase 4K MCP bridge and C-35's gated/lazy catalog; parallel-safe with C-36 through C-40 but sequenced after C-35. [C-39 L9-L13]
- **C39-E003 [A+]** Source correction: phase-4K.txt contains no MCP content; the real bridge is plugins/src/mcp_bridge.rs plus plugins/src/mcp/{mod,gate,lazy}.rs and a C-35 alias. [C-39 L23-L26]
- **C39-E004 [A+]** Baseline gaps are no CLI bridge coverage, no CLI visibility of externally registered MCP tools, and dashboard/CLI CI exercising different transports. [C-39 L28-L38]
- **C39-E005 [A+]** MCP routing is optional and backward-compatible: --via-mcp defaults off and the existing path remains when absent; bridge behavior becomes the parity source of truth. [C-39 L40-L44]
- **C39-E006 [A+]** Source grounding extracts only the OpenHands MCP client and tool-dispatch subset into workflow/papers/openhands-mcp-integration.md. [C-39 L59-L63]
- **C39-E007 [S]** New cli/src/mcp_client.rs is a minimal localhost HTTP JSON-RPC 2.0 client with list_tools, invoke_tool(name,args), and lazy get_tool_schema(name). [C-39 L65-L68, L168-L170]
- **C39-E008 [S]** --via-mcp is threaded through both /task and /explore/CliAction::Run startup routing; when set, FSM dispatch uses MCP rather than the default path. [C-39 L70-L72, L95-L98, L121-L124]
- **C39-E009 [S]** If no listener exists, CLI launches the dashboard bridge backend as a co-process on a random localhost port and tears it down at process exit. [C-39 L74-L77, L171-L173]
- **C39-E010 [A+]** If an external listener already exists, it is reused rather than launching a co-process. [C-39 L172-L174]
- **C39-E011 [A+]** The selected port is surfaced through SSB. [C-39 L74-L77]
- **C39-E012 [S]** A blocking CI matrix runs every existing CLI integration test once per route and compares outputs, permitting only expected dispatch-specific SSB action-name differences. [C-39 L79-L82, L125-L130, L176-L177]
- **C39-E013 [S]** New SSB actions are cli_mcp_dispatch, cli_mcp_server_started, and cli_mcp_server_stopped; source says each carries tool_name and port. [C-39 L84-L85, L109-L115, L174]
- **C39-E014 [A+]** Five TDD cases are C39.1 arbitrary/default port construction, C39.2 both Dispatch implementations satisfy the trait, C39.3 three action-type SQL round trips, C39.4 three SsbPayload construction/action reports, and C39.5 unreachable-port fail-closed behavior. [C-39 L87-L94]
- **C39-E015 [S]** A shared Dispatch trait has McpClientDispatch and InProcessDispatch implementations; startup chooses one based on the flag. [C-39 L95-L98, L109-L113]
- **C39-E016 [A+]** cli_mcp_contract.rs checks trait-shape equivalence and equivalent SSB rows for equivalent input. [C-39 L100-L102, L178]
- **C39-E017 [A+]** CLI docs gain usage/help for --via-mcp; the listed new docs also include --export-ssb. [C-39 L104-L105, L199]
- **C39-E018 [A+]** Consumed contracts are the C-35 alias to the MCP endpoint, ToolGate catalog filtering, and SSB; reachable commands include via-MCP task/explore and unchanged default task. [C-39 L116-L124]
- **C39-E019 [A+]** Network errors are never silently retried; they emit/fail closed. Failure to discover or launch a bridge emits a structured SSB error and never panics. [C-39 L125-L130]
- **C39-E020 [A+]** Existing dashboard MCP UI must still work after the Dispatch refactor. [C-39 L131-L132]
- **C39-E021 [A+]** Observable proof is started-if-coprocess plus one dispatch row per call, parity-equivalent output, and an empty normalized diff. [C-39 L133-L138]
- **C39-E022 [A+]** Deterministic gates check modules, both impls, flag, actions, workflow, contract test, and local ci-mcp-parity.sh. [C-39 L140-L164]
- **C39-E023 [A+]** File surface adds CLI client/lib/tests, three orchestrator dispatch modules, parity script/workflow, source artifact and docs, and modifies main/commands/SSB. [C-39 L183-L202]
- **C39-E024 [Q]** The file ledger records the three SSB actions as already SHIPPED in s-68264 at schema v89->v90 and intact through v103, even though the broader acceptance list remains unchecked. [C-39 L166-L181, L196-L198]
- **C39-E025 [A+]** Closing this surface enables external MCP clients without dashboard, unified catalog CI, and the MCP prerequisite for C-44's third runtime. [C-39 L209-L215]

### Inferred

- **C39-I001** Correct composition with C-13 requires InProcessDispatch to be an abstraction over the production IpcToolRegistry, not a resurrection of InProcessToolDispatcher.
- **C39-I002** Random localhost binding reduces accidental network exposure, but it is not an authentication policy.

### Ambiguous

- **C39-A001** The type is called MCPClient in Tasks/WIRING and CliMcpClient in test/file commentary; the canonical public identifier is not stable. [C-39 L65-L68, L89-L90, L109-L110, L200-L202]
- **C39-A002** Listener discovery, port configuration, readiness timeout, co-process binary invocation, graceful shutdown, orphan recovery, and crash cleanup are not specified. [C-39 L74-L77]
- **C39-A003** Server start/stop actions are required to carry tool_name even though they are not per-tool events; nullability/default semantics are absent. [C-39 L84-L85, L174]
- **C39-A004** Parity normalization permits action-name differences but does not enumerate the mapping or treatment of nondeterministic IDs, ports, order, or timestamps. [C-39 L79-L82, L133-L138]
- **C39-A005** No bridge authentication, caller authorization, origin restriction beyond localhost, or policy for externally registered tools is specified.

### Conflicting

- **C39-X001** TDD cases are deliberately renamed C39.1..C39.5, but acceptance still requires stale 4K.X.3..4K.X.7 identifiers. [C-39 L87-L94, L175]
- **C39-X002** Context describes the default CLI path as direct in-process plugin dispatch and unchanged, while C-13 bans InProcessToolDispatcher in production. The types are not necessarily identical, but implementation must explicitly reconcile this intent collision. [C-39 L30-L44; C-13 L64-L73, L184-L189]

## C-40 — GitHub PR adapter

### Explicit atomic ledger

- **C40-E001 [S]** Objective: extend /session merge from local history to an opt-in GitHub review loop: open a PR, compose a SessionDAG body, attach audit evidence, watch review comments, and route responses through SSB sessions. [C-40 L1-L8]
- **C40-E002 [A+]** Requires shipped C-13 squash/merge primitives and C-23 SessionDAG/branch_merged surface; parallel-safe with C-35 through C-39 and C-41 through C-53. [C-40 L10-L13]
- **C40-E003 [A+]** Competitive grounding treats Devin's PR/review loop as a feature target, explicitly rejects unverified marketing/benchmark imitation, and notes B-25/Empire does not provide GitHub. [C-40 L15-L24]
- **C40-E004 [A+]** The adapter is additive: absent [github] configuration leaves all existing sessions unchanged. [C-40 L35-L50]
- **C40-E005 [A+]** [github] defaults are enabled=false, adapter="gh", default_base_branch="main", pr_body_template="default", review_poll_interval_s=60, and auto_respond_review_comments=false. [C-40 L62-L71]
- **C40-E006 [C!]** The implementation is a feature="github_pr" module at crates/orchestrator/src/github_pr, not the summary's separate crates/github_adapter package. It exports GithubAdapter, GhCliAdapter, RestApiAdapter, PrBodyComposer, ReviewCommentWatcher, and ReviewCommentRouter. [C-40 L73-L76, L143-L153, L234-L246]
- **C40-E007 [A+]** GhCliAdapter executes gh pr create/view/comment, parses --json output, and reports missing binary or missing authentication as structured SSB errors. [C-40 L78-L80]
- **C40-E008 [A+]** RestApiAdapter uses reqwest plus GITHUB_TOKEN for create PR/get comments/post comment and is selected only by adapter="rest" or explicit fallback opt-in after gh absence. [C-40 L82-L85]
- **C40-E009 [A+]** PrBodyComposer title is the first non-empty baby-step description; summary lists steps with acceptance badges. [C-40 L87-L91]
- **C40-E010 [A+]** PR audit trail links .elai/audit/{session_id}.md; test trust includes mutation score, metamorphic-relation coverage, and IntentPair coverage. [C-40 L91-L95]
- **C40-E011 [A+]** PR body includes open/closed stub status from C-1 through C-8. [C-40 L95]
- **C40-E012 [S]** The honesty section identifies ELAI/session, states mutation score and wiring pass/fail, and instructs reviewer to verify the acceptance checklist; marketing language is forbidden. [C-40 L96-L98, L169]
- **C40-E013 [A+]** --watch-reviews spawns an async ReviewCommentWatcher for /session merge --to-pr; it polls gh pr view every configured interval and emits on each newly seen comment. [C-40 L100-L103]
- **C40-E014 [A+]** ReviewCommentRouter opens a Review-Comment-Triggered session scoped to the file mentioned by a comment only when auto_respond_review_comments=true. [C-40 L105-L108]
- **C40-E015 [A+]** Automatic routing defaults off; enabling it requires explicit config and a first-hit confirmation prompt. [C-40 L105-L108, L167-L168, L219-L220]
- **C40-E016 [S]** Six action strings are github_pr_created, github_pr_body_posted, github_comment_received, github_comment_routed_to_session, github_adapter_error, and github_review_poll_tick; adding them bumps SCHEMA_VERSION. [C-40 L110-L112, L145-L150]
- **C40-E017 [S]** Merge flags are --to-pr, --base <branch>, --watch-reviews, and --draft. [C-40 L114-L118, L151]
- **C40-E018 [A+]** TDD 4A.X.3 requires deterministic PrBodyComposer output from a synthetic SessionDAG. [C-40 L120-L122]
- **C40-E019 [A+]** TDD 4A.X.4 uses a PATH-injected mock gh wrapper and verifies create_pr JSON parsing. [C-40 L123-L124]
- **C40-E020 [A+]** TDD 4A.X.5 proves first-poll new-comment SSB emission. [C-40 L125-L126]
- **C40-E021 [A+]** TDD 4A.X.6 proves no auto-trigger at the default false setting. [C-40 L127-L128]
- **C40-E022 [A+]** TDD 4A.X.7 proves explicit RestApi fallback when gh is absent and refusal when fallback was not opted into. [C-40 L129-L130]
- **C40-E023 [A+]** CLI flags flow through the session-merge path; adapter and watcher lifetime is bounded to the session and watcher tears down at session end. [C-40 L132-L134]
- **C40-E024 [Q]** GithubPanel.svelte is planned/optional after Rust types; intended fields are PR URL, comment count, last poll, and watcher status. [C-40 L136-L138, L170, L229-L230]
- **C40-E025 [A+]** github_adapter_contract.rs requires GhCliAdapter and RestApiAdapter to satisfy one trait shape. [C-40 L140-L141]
- **C40-E026 [A+]** gh absence must never silently select REST; REST refuses missing GITHUB_TOKEN and may never embed a placeholder/hardcoded token. [C-40 L162-L166, L214-L215]
- **C40-E027 [A+]** Observable lifecycle for merge --to-pr --watch-reviews --base main is PR-created/body-posted, poll ticks while active, comment-received, stdout URL, and optional panel state. [C-40 L171-L178]
- **C40-E028 [A+]** Static/done gates check feature declaration, all six adapter symbols, all six actions, flags, feature-enabled tests, contract, mock fixture, and schema bump. [C-40 L180-L206]
- **C40-E029 [A+]** Secret-gated E2E must open, comment on, and close a draft PR in a real repository. [C-40 L226-L228]
- **C40-E030 [A+]** File surface adds seven github_pr module files, CLI command, contract, gh fixture, default template, optional panel; modifies feature config, CLI, merge path, SSB schema, project config, and docs. [C-40 L232-L251]
- **C40-E031 [A+]** C-13 and C-23 verification reports are updated to record the new upstream consumer/surface. [C-40 L253-L258]
- **C40-E032 [A+]** Non-goals: no automatic merge; comment auto-response remains off; public PR stays under human maintainer control. [C-40 L260-L267]
- **C40-E033 [Q]** Every acceptance box is open; GithubPanel is explicitly allowed as a separate follow-up. [C-40 L208-L230]

### Inferred

- **C40-I001** The required config flag plus --to-pr command constitutes the intended authorization boundary for external write, but an explicit one-time “open this PR?” confirmation is not stated.
- **C40-I002** Structured SSB errors are intended to make external side effects auditable, but the plan does not define redaction of tokens, comment bodies, repository URLs, or user identities.

### Ambiguous

- **C40-A001** The watcher emits “review_comment_received” in Task 6, but the declared action enum is github_comment_received; only the latter is in WIRING/acceptance. [C-40 L100-L112]
- **C40-A002** “Scoped to the file mentioned” has no parser, multi-file policy, handling for general comments, authorization rule, dedup key, or already-routed-comment cursor. [C-40 L105-L108]
- **C40-A003** Poll retries, rate-limit/backoff, pagination, edit/delete handling, ordering, and crash resume are unspecified; repeat routing could violate idempotency.
- **C40-A004** REST token scopes, secret storage, log redaction, transport timeout, and GitHub Enterprise/base-URL support are unspecified. Context mentions a git2 + raw-REST fallback, while the concrete RestApiAdapter task specifies reqwest only; git2's role is not resolved. [C-40 L35-L39, L82-L85]
- **C40-A005** The static feature check targets .elai_cc/Cargo.toml while Files says the feature is added to crates/orchestrator/Cargo.toml. [C-40 L183-L185, L246]
- **C40-A006** C-40 cites session/dag.rs and session/merge.rs, while C-23 works in orchestrator/session_dag.rs; source paths may reflect an unrecorded refactor. [C-40 L52-L58; C-23 L79-L92]

### Conflicting

- **C40-X001** Task/acceptance expose --draft as an option, implying non-draft is possible, while the closing non-goal says the PR “remains draft unless the user marks ready”; no default-draft rule exists in config. [C-40 L114-L118, L221, L265-L267]
- **C40-X002** E2E requires closing a PR, but neither adapter's operation subset nor CLI flags define close_pr. [C-40 L78-L85, L114-L141, L226-L228]
- **C40-X003** C-13 supplies WIP squash but C-23 supplies branch merge; describing “session merge primitive (C-13)” is a dependency attribution error. [C-40 L10-L11, L154-L158; C-13 L104-L117; C-23 L51-L59]

## C-53 — NF4 upstream watch and in-context counter-test

### Explicit atomic ledger

- **C53-E001 [S]** Objective: one low-priority tracker combines a passive bitsandbytes NF4 optimization watch with an orchestration-vs-single-prompt procedural-task ablation; neither item merits a separate C-N. [C-53 L1-L7]
- **C53-E002 [S]** Header declares no prerequisites and parallel safety with every C-N. [C-53 L9-L10]
- **C53-E003 [A+]** NF4 grounding is arXiv:2604.02556, dated 2026-04-02, reporting 2.0-2.2x kernel and up to 1.54x end-to-end gains from NVIDIA shared-memory patterns; the integration is research-grade CUDA/Triton. [C-53 L14-L20]
- **C53-E004 [A+]** ELAI consumes bitsandbytes NF4 through python/elai_inference/server.py, so an upstream adoption can arrive through a pinned dependency upgrade. [C-53 L22-L26]
- **C53-E005 [S]** Add huggingface/bitsandbytes to .elai_cc/docs/upstream-watch.md with paper reference and trigger “major release, run benchmark delta.” [C-53 L45-L50]
- **C53-E006 [A+]** nf4_e2e is a trigger-only, non-normal-CI benchmark over a fixed prompt set, comparing current bitsandbytes with upstream-latest and measuring sidecar tokens/second end to end. [C-53 L51-L54]
- **C53-E007 [A+]** On a bitsandbytes major bump, adopt/pin and changelog the version only if end-to-end speedup is at least 1.2x; the paper's 1.54x is treated as an upper bound. [C-53 L56-L59]
- **C53-E008 [A+]** In-context grounding is arXiv:2604.27891, dated 2026-05-02/forum item 10, claiming orchestration is obsolete for procedural tasks; the plan treats it as direct counter-evidence to Phase 4A-4G. [C-53 L28-L41]
- **C53-E009 [S]** paper-ingestor paper mode produces workflow/papers/in-context-prompting-counter.md. [C-53 L61-L64]
- **C53-E010 [S]** Define and document a 30-task SWE-bench Verified procedural subset, exemplified by mechanical API-v1-to-v2 conversion. [C-53 L66-L69]
- **C53-E011 [S]** --baseline-single-prompt bypasses Phase 4A-4G decomposition, performs one prompt-engineering call, and uses the same model as the default/full arm. [C-53 L71-L73]
- **C53-E012 [Q]** Run both arms on all 30 tasks, compare solve rate and cost, and write the pilot artifact; execution is deferred to RunPod alongside C-54. [C-53 L75-L76, L127-L134]
- **C53-E013 [A+]** If single-prompt wins on solve rate and cost, record it and use C-41 skill_selected classification to route procedural work to a single-prompt path. [C-53 L78-L81]
- **C53-E014 [A+]** Otherwise document decomposition's win and archive the counter-evidence. [C-53 L80-L81]
- **C53-E015 [S]** Add counter_test_run and procedural_routing_decision actions. [C-53 L83-L84, L87-L94]
- **C53-E016 [A+]** A routing change is exposed as [routing.procedural_single_prompt], is config-gated, and defaults off so speculation never changes routing. [C-53 L98-L107, L134]
- **C53-E017 [A+]** Consumed contracts are bitsandbytes, existing FSM dispatch, C-41 SkillRegistry only for a winning-route flip, and the SSB bus. [C-53 L94-L101]
- **C53-E018 [A+]** Evidence bans require wall-clock end-to-end rather than synthetic NF4 measurement and a per-task pilot artifact containing both arms. [C-53 L102-L107]
- **C53-E019 [A+]** Done gates check the watch doc/entry, bench file, CLI flag, procedural fixture, two actions, compilable local baseline, paper artifact, RunPod pilot, matrix outcome, and any default-off route. [C-53 L109-L145]
- **C53-E020 [Q]** Every acceptance box is open; closure is evidence that either defends decomposition or justifies a pivot, and a future pivot may feed C-46's tier-zero SelectiveDelegator. [C-53 L120-L152]

### Inferred

- **C53-I001** “Current” versus “upstream-latest” must be pinned in the result artifact for reproducibility, although the plan names only the current project pin implicitly.
- **C53-I002** The routing outcome is an ablation result for a selected procedural subset, not evidence that orchestration is globally useful or obsolete.

### Ambiguous

- **C53-A001** “Subscribe” and “major release” have no automation, cadence, owner, notification channel, or semantic-version policy. [C-53 L22-L26, L47-L50]
- **C53-A002** A Rust cargo bench measuring a Python GPU sidecar lacks a stated process protocol, hardware pin, warm-up, sample count, variance calculation, or current/upstream dependency isolation. [C-53 L51-L59]
- **C53-A003** The 30-task “pick” process has no deterministic classifier, frozen IDs/SHA, reviewer protocol, or anti-cherry-pick rule. [C-53 L66-L69]
- **C53-A004** counter_test_run and procedural_routing_decision payload schemas, schema-version behavior, and emission cardinality are absent. [C-53 L83-L84]
- **C53-A005** The decision matrix has no branch for equal solve rate, solve/cost trade-offs, inconclusive uncertainty, or failed tasks. [C-53 L78-L81]

### Conflicting

- **C53-X001** “single-prompt > full-decomposition on solve-rate AND cost” uses greater-than for cost even though lower cost is normally better; the intended rule is likely higher solve rate plus lower cost, but the source is literal. [C-53 L78-L81]
- **C53-X002** Task text names tests/fixtures/procedural_subset.toml, while deterministic checks and Files require crates/task_runner/tests/fixtures/procedural_subset.toml. [C-53 L66-L69, L91-L92, L116, L140]

## C-55 — Aider Polyglot leaderboard submission

### Explicit atomic ledger

- **C55-E001 [S]** Objective: run all 225 Aider Polyglot problems under local-32B and hybrid configurations, compare against public raw-model/cost baselines, and submit leaderboard data. [C-55 L1-L7]
- **C55-E002 [S]** Header requires shipped C-51 harness plus live RunPod; declared parallel-safe with C-54/C-56/C-57/C-58 but sequential when sharing one RunPod. [C-55 L9-L11]
- **C55-E003 [Q]** Reference numbers are a 2026-05-17 snapshot: GPT-5 high 88.0%/$29.08, GPT-5 low 81.3%/$10.37, and ollama qwen2.5-coder:32b 72.9%. [C-55 L13-L16]
- **C55-E004 [A+]** Hypothesis: ELAI plus the same local 32B beats raw ollama/Qwen at per-dollar performance; a loss means the local-only “infrastructure substitutes for parameters” thesis needs revision. [C-55 L18-L28]
- **C55-E005 [S]** Config L uses backends=[local_qwen_32b] and is the direct local comparison; Config H uses [sonnet_46, composer2, local_qwen_32b] and is compared with GPT-5 high by cost per task. [C-55 L30-L34]
- **C55-E006 [A+]** Immediately before submission, fetch aider.chat/docs/benchmarks afresh for JSON shape, required fields, and public PR template. [C-55 L36-L45, L115]
- **C55-E007 [S]** Author config-L-local-32B.toml and config-H-hybrid.toml under .elai_cc/benchmarks/aider_polyglot/configs. [C-55 L47-L50]
- **C55-E008 [A+]** Execute 225 x 2 = 450 task runs and collect solved, wall_time_s, llm_calls, tokens_in, tokens_out, cost_usd, and repaired_after_fail per task; the last field signals C-31/C-32 StructuralRetry. [C-55 L51-L54]
- **C55-E009 [A+]** Config L cost is GPU hourly rate multiplied by wall-time fraction, with RunPod A100/H100/4090 rate documented. [C-55 L56-L59]
- **C55-E010 [A+]** Config H cost combines the local GPU share and metered Sonnet 4.6 plus Composer 2 API cost. [C-55 L59-L60]
- **C55-E011 [A+]** Both cost calculations must match Aider's published methodology for apples-to-apples comparison and be machine-derived from time/token inputs, never estimated by a person. [C-55 L61-L62, L96-L98]
- **C55-E012 [S]** Save exact-format scorecards at workflow/sessions/{id}/aider_polyglot_run_L/scorecard.json and ..._H/scorecard.json. [C-55 L64-L65, L80-L83]
- **C55-E013 [S]** Fork Aider-AI/aider, add leaderboard entries, and use C-40 LeaderboardPRComposer to open a PR whose body cites methodology, ELAI commit SHA, and elai_cc.toml versions. [C-55 L67-L70]
- **C55-E014 [S]** Generate workflow/sessions/{id}/aider_polyglot_methodology.md and .elai_cc/docs/aider_polyglot_results.md. [C-55 L72-L73]
- **C55-E015 [C!]** Source action identifiers are polyglot_submission_started, polyglot_submission_completed, and polyglot_pr_opened; the summary's polyglot_submission_pr_opened is incorrect. polyglot_pr_opened consumes C-40 github_pr_created. [C-55 L75-L76, L80-L86]
- **C55-E016 [A+]** CLI reachability is elai benchmark aider-polyglot --config <name> --runpod-id <id> and --submit only after both configs complete. [C-55 L92-L95]
- **C55-E017 [A+]** Public PR must remain DRAFT and never auto-merge; results must cite methodology and the public PR URL. [C-55 L96-L100, L121]
- **C55-E018 [A+]** Deterministic checks are artifact-existence checks for both configs/scorecards/results and grep for all three SSB actions. [C-55 L102-L111]
- **C55-E019 [S]** Acceptance requires 225-problem scorecards for both configs, machine-derived cost, both docs, a public draft PR, all actions, honest local comparison to 72.9%, and honest hybrid comparison to GPT-5 low/high at matched cost. [C-55 L113-L126]
- **C55-E020 [A+]** File surface adds two configs, two post-run scorecards, methodology/results, benchmark CLI/submit, and three SSB actions. [C-55 L128-L138]
- **C55-E021 [Q]** C-55 is explicitly RUNPOD-DEPENDENT and every acceptance box is open; scorecards/docs are marked post-run. [C-55 L1, L113-L138]
- **C55-E022 [Q]** A draft submission is not itself a public leaderboard entry: actual publication depends on external Aider review/merge. [C-55 L67-L70, L99-L100, L121, L140-L145]
- **C55-E023 [A+]** C-55 is intended to pair with C-54 so ELAI has an issue-resolution axis and a cost-per-task axis. [C-55 L140-L145]

### Inferred

- **C55-I001** repaired_after_fail is observability for structural recovery, not a scoring dimension unless the fresh Aider format includes it.
- **C55-I002** Honest better/worse/equal reporting means there is no adoption/pass threshold; completing a noncompetitive run still closes the evidence task.

### Ambiguous

- **C55-A001** Exact Aider JSON/schema/PR fields are deliberately deferred to a fresh fetch, so they cannot be extracted from this source. [C-55 L36-L45, L64-L70]
- **C55-A002** The quoted leaderboard values are historical and may change; only the submission protocol is explicitly fresh-fetched, although comparisons also need a dated baseline snapshot. [C-55 L13-L16, L43-L45]
- **C55-A003** “Same local 32B” does not pin quantization, serving stack, prompt format, hardware, context length, or commit, so raw-model comparability needs the methodology artifact. [C-55 L20-L34]
- **C55-A004** Seed, task order, retry cap, timeout/failure accounting, resumed-run behavior, idempotency, duplicate submissions, and anti-cherry-pick controls are not specified.
- **C55-A005** Public fork/PR is an external write, but this plan does not add an explicit user approval gate beyond invoking --submit and C-40's draft behavior.
- **C55-A006** Sonnet/Composer credentials, rate limits, pricing snapshot, error fallback, and data-sharing/privacy policy for benchmark tasks are unspecified.

### Conflicting

- **C55-X001** Header prerequisites name only C-51 and RunPod, but Config H consumes C-44 Composer 2 and public submission consumes C-40; both are functional prerequisites for full closure. The summary adds C-40 but still omits C-44. [C-55 L9-L11, L30-L34, L87-L91]
- **C55-X002** “Produces ELAI's first PUBLIC entry” overstates the guaranteed effect: acceptance only opens a draft PR, whose merge/publication is controlled by Aider maintainers. [C-55 L99-L100, L121, L140-L145]

## Cross-source interactions and composed behavior

| Interaction | Classification | Exact composed contract |
|---|---|---|
| C-15 -> C-23/C-31 | Explicit dependency substrate plus conflict | C-15 supplies clock/map determinism used by stateful orchestration. C-31 nevertheless introduces std HashMap aliases/fields in production-looking core/plugin/orchestrator code; these must use FxHashMap or be explicitly reconciled with C-15's ban/carve-out. [C-15 L89-L111, L203-L207; C-31 L89-L101, L141-L151] |
| C-13 -> C-31 | Explicit prerequisite, ambiguous symbol ownership | C-31 declares C-13 prerequisite for stable mistake-book trait, but C-13 never defines MistakeRecorder or mistake-book APIs; C-31 itself creates MistakeRecorder. Treat the dependency as dispatcher/production-wiring stability, not a consumed named trait. [C-31 L8-L10, L112-L125; C-13 L158-L177] |
| C-13 -> C-39 | Potential production-dispatch conflict | C-13 mandates IpcToolRegistry as sole production dispatcher and test-gates InProcessToolDispatcher. C-39 adds default InProcessDispatch and calls the old route “unchanged.” InProcessDispatch must wrap the C-13 production registry or the plans conflict. [C-13 L64-L73, L184-L189; C-39 L40-L44, L95-L98] |
| C-13 + C-23 -> C-40 | Explicit workflow | C-13 squashes WIP history before memorizing; C-23 supplies branch merge, summaries/events, and UI; C-40 consumes the clean local merge/SessionDAG summary to create an auditable upstream PR. [C-13 L104-L117; C-23 L51-L59, L101-L106; C-40 L1-L13, L87-L99] |
| C-23 -> C-40 | Explicit event/data coupling | branch_merged and SessionDAG summary trigger/feed PrBodyComposer; path/schema stability is required, but C-40 cites a different SessionDAG file layout. [C-23 L108-L124; C-40 L52-L58, L145-L158] |
| C-31/C-32 -> C-55 | Explicit telemetry coupling | C-55's repaired_after_fail field attributes recovery to C-31/C-32 StructuralRetry, though StructuralRetry is actually Layer 2/C-32 and not defined in C-31. [C-55 L51-L54] |
| C-33 <-> C-31 | Related but distinct failure enums | C-31 consumes core mistake_book FailureCategory variants already present. C-33 adds same-named variants to tools/failure_taxonomy and synchronizes ToolFailureCategory. Do not treat C-33 as providing C-31's enum. [C-31 L15-L21, L190-L197; C-33 L42-L45, L176-L190] |
| C-33 -> documentation consumers | Explicit evidence mapping | C-33 creates local specs for C-30 and already-shipped surfaces, reconciles plan names to code, and updates the index; gitignore means these links are not durable through normal Git. [C-33 L12-L13, L69-L174, L258-L295] |
| C-37 -> C-54 publication | Explicit sequencing | C-37's measured pilot/default must settle before C-54 so a flagship number does not bake in an untested auditor constraint. [C-37 L9-L12, L196-L202] |
| C-39 -> C-44 -> C-55 | Explicit/indirect dependency | C-39 says its MCP dispatch unblocks C-44 Composer 2; C-55 Config H requires Composer 2. [C-39 L209-L215; C-55 L30-L34, L87-L91] |
| C-40 -> C-55 | Explicit external-write workflow | C-55 uses C-40 to create a non-auto-merged draft leaderboard PR after both scorecards complete. [C-40 L162-L170; C-55 L67-L76, L92-L100] |
| C-53 -> C-41/C-46 | Conditional routing | Only a winning single-prompt ablation consumes C-41 skill classification and may extend C-46 tier-zero routing; default routing stays off until measured/configured. [C-53 L78-L81, L94-L107, L147-L152] |
| C-37/C-53/C-55 RunPod scheduling | Explicit status constraint | C-37 and C-53 pilots are deferred around C-54; C-55 is fully RunPod-dependent and can be logically parallel but physically sequential on one pod. [C-37 L65-L67; C-53 L75-L76; C-55 L9-L11] |

## Cross-cutting security, privacy, failure, and determinism ledger

### Explicit

- IPC production dispatch is FSM-gated, legacy in-process construction is test/feature-only, and new paths ban panic/unwrap. [C-13 L64-L102, L184-L189]
- C-15 treats time and unordered maps as reproducibility hazards, introduces reasoned exceptions, and makes CI the enforcement surface, subject to its later residual budgets. [C-15 L113-L170, L203-L236]
- Session branch dashboard writes use configured state rather than hardcoded paths and return typed errors. [C-23 L51-L59, L185-L191]
- C-31 validates plans before dispatch, preserves tracker state over crash recovery, and blocks actual AST excess rather than mock counts. [C-31 L89-L110, L127-L139, L206-L228]
- C-39 network failures fail closed and emit structured SSB evidence; silent retry and informational-only parity are banned. [C-39 L125-L130]
- GitHub is opt-in/default-off, REST requires GITHUB_TOKEN, fallback requires explicit consent, auto-response defaults off plus first-use confirmation, PR copy is factual, and auto-merge is out of scope. [C-40 L49-L50, L62-L71, L82-L99, L162-L170, L260-L267]
- C-53 routing remains config-gated/default-off until a two-arm artifact exists. [C-53 L102-L107]
- C-55 cost must be machine-derived, results must report wins/losses honestly, and public submission is a draft that cannot auto-merge. [C-55 L96-L100, L113-L126]

### Missing or underspecified

- C-13 has no concurrency guard around git history mutation.
- C-15's tolerated-leak scripts do not define a consistent exit-code/CI-ratchet contract.
- C-23 does not specify branch-operation idempotency, merge-conflict recovery, transactionality between DAG and SSB, or retry behavior.
- C-31's generation guard does not describe real concurrent ownership or recovery after a partially recorded file write.
- C-37 lacks seed/sampling/repetition/statistical power and a tie branch.
- C-39 lacks MCP authentication, permission negotiation, readiness/backoff, shutdown recovery, and exact parity normalization.
- C-40 lacks token-scope/redaction guidance, comment dedup/cursor persistence, API backoff, and a defined PR-close operation.
- C-53 lacks a frozen subset/pin and statistically reproducible benchmark procedure.
- C-55 lacks run locking/resume/idempotency, failed-task policy, anti-cherry-pick rules, secret/data-sharing policy, and an explicit approval prompt for the external PR.

## Summary-index additions and corrections

The delta tag on every explicit atom is the exhaustive reconciliation against 03-C-SUMMARY_INDEX.discovery.md: S records need no addition; every A+, C!, and Q record is a summary addition, correction, or qualification. The highest-impact corrections are:

| Source | Correction/qualification to summary orientation |
|---|---|
| C-13 | Names the five already-emitted DAIRA markers; adds exact per-marker triggers, squash algorithm/payload/error rules, feature gate, and all fifteen test identifiers. |
| C-15 | The later 2026-05-23 rebaseline supersedes a zero-leak reading: accepted closure is up to five dashboard HashMaps and 51 measured/52 allowed entropy leaks; combined anti-stale CI and billable smoke remain open, reports deferred. |
| C-23 | AppState may hold a DAG handle or configured DB path, not necessarily a handle; adds typed errors, usage behavior, UI fallback, async/Svelte hazards, and determinism setup uncertainty. |
| C-31 | “Ninth field” is arithmetically inconsistent with the nine fields listed before it; no FileLineTracker type is actually specified; adds exact schemas, formulas, errors, test names, and hook/category inconsistency. |
| C-33 | C-22 is not a specifically declared header prerequisite; full local-only spec contents, exact command classifications, conditional B-38 status, and weaker >=10 acceptance gate are additions. |
| C-37 | Migration/config flip is conditional, not unconditional; adds exact >=10pp / 0-10pp / loss matrix, tie gap, event fields, conditional schema bump, and measured-artifact closure semantics. |
| C-39 | Adds default-off transport behavior, exact client methods, random co-process lifecycle, fail-closed/no-retry rules, five TDD identifiers, stale test numbering, and the already-shipped v89->v90 event status. |
| C-40 | Implementation is orchestrator/src/github_pr, not a standalone crates/github_adapter package; adds exact config/auth/consent/body/poll contracts and records event-name, draft-default, and unsupported-close inconsistencies. |
| C-53 | Adds the 1.2x adoption floor, two-config end-to-end benchmark, exact routing decision, default-off gate, subset/path gaps, and the inverted/ambiguous “> cost” comparison. |
| C-55 | Correct event name is polyglot_pr_opened, not polyglot_submission_pr_opened; C-44 is an omitted functional dependency, baselines are dated snapshots, and a draft PR does not guarantee leaderboard publication. |

## Status and deferred-work rollup

| Source | Source-level status |
|---|---|
| C-13 | Planned; all checkboxes open; blocked on confirmed C-11 shipment. |
| C-15 | Partially executed/rebaselined; several boxes checked; combined CI, billable smoke, and report updates remain open/deferred; residual debt routed to unauthored B-15-regression-burndown. |
| C-23 | Planned; all boxes open; 100-run test may be credited to C-18 if already present. |
| C-31 | Planned; all boxes open. |
| C-33 | Planned; all boxes open; six spec changes are gitignored/local-only; B-38 needs status evidence. |
| C-37 | Planned/RunPod-deferred; either measured keep or flip closes it. |
| C-39 | Mixed: three SSB actions recorded shipped at v90, broader CLI/parity acceptance still open. |
| C-40 | Planned; all boxes open; GithubPanel explicitly follow-up-capable. |
| C-53 | Planned; pilot and decision deferred to RunPod/C-54 window. |
| C-55 | Entire run/publication task RunPod-dependent; all boxes open; external merge remains outside ELAI control. |

## Completeness, omission, duplicate, and atomic-count audit

- Originals processed: **10/10**; unique originals: **10**; duplicate manifest paths: **0**; missing originals: **0**; extra originals: **0**.
- Logical lines read: **2,553/2,553**; bytes hash-verified: **136,705/136,705**; hashes matched: **10/10**.
- Explicit feature/behavior atoms: **301**, by file: C-13 **38**, C-15 **32**, C-23 **25**, C-31 **33**, C-33 **50**, C-37 **22**, C-39 **25**, C-40 **33**, C-53 **20**, C-55 **23**.
- Interpretive records are outside the explicit-atom count and remain visibly separated: **20 inferred**, **47 ambiguous**, **22 per-file conflicting**, plus **13 cross-source interaction records**.
- Atom-ID uniqueness: every E/I/A/X identifier is unique within its source. Repeated source statements across Objective, Tasks, WIRING_CONTRACT, DETERMINISTIC_CHECK, ACCEPTANCE, Files, and Next are merged into one explicit atom rather than counted twice.
- Coverage audit: each source's title/objective, context/baseline, every numbered task, WIRING_CONTRACT (produces/consumes/reachability/stub bans/frontend/effects), deterministic/adversarial checks, acceptance/status, file surface, affected reports, and Next/deferred clauses is represented.
- Summary delta audit: every explicit atom is tagged S, A+, C!, or Q; all A+/C!/Q atoms are additions/corrections/qualifications to the summary orientation. No summary-only assertion was silently promoted to original-source fact.
- Scope audit: only the exact DEEP-C-006 originals were deep-read; 03-C-SUMMARY_INDEX.discovery.md was used only for orientation/delta comparison; no other original chunk was processed.
