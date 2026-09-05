# DEEP-C-005 — exhaustive feature and behavior discovery

## Scope, evidence rules, and integrity

This ledger deep-reads exactly the ten originals assigned to manifest chunk `DEEP-C-005`. `03-C-SUMMARY_INDEX.discovery.md` was consulted only as an orientation/comparison artifact; it is not treated as evidence for a source feature. No original from any other chunk was read.

Labels:

- `[E]` — explicit in the named original.
- `[I]` — conservative inference from two or more explicit statements; not an implementation guarantee.
- `[A]` — underspecified or internally unclear.
- `[C]` — direct conflict within the original, or between the original and the orientation summary.
- `[D+]` — material source detail absent from the orientation summary.
- `[D~]` — orientation-summary statement that needs correction or qualification.

Line anchors are one-based lines in the verified file named by the surrounding section. An “atomic item” is one independently checkable capability, state rule, schema, gate, constraint, test obligation, ambiguity, conflict, or conservative interaction inference. A schema item may list its fields as one atom because the fields form one indivisible declared contract.

## Exact processed manifest and verification

The manifest declares **10 files, 2,553 logical lines, and 139,392 bytes**. Fresh `wc -l`, `wc -c`, and `sha256sum` results matched every row exactly.

| # | Class | Lines | Bytes | SHA-256 | Exact absolute path | Result |
|---:|---|---:|---:|---|---|---|
| 1 | numbered/index original | 335 | 17924 | `49325f3ae8452c9e2d3a8ac22641e32410087b73eff0caf18d586c84f0ab0c2d` | `docs/thefinalglance/C-plan/C-9_autonomous_infrastructure.md` | exact |
| 2 | numbered/index original | 262 | 16202 | `5f49d3bbdd19d0cb15738b3aea26f479fcabd66d4fe6da5588f9baef69b594fe` | `docs/thefinalglance/C-plan/C-16_cosmos_pipeline_integration.md` | exact |
| 3 | numbered/index original | 316 | 18649 | `f322c804a752911069e837d7dbf84edfee0b0b9c2363d23a89230c9e2ad7b8a7` | `docs/thefinalglance/C-plan/C-24_symbol_impact_dashboard_surface.md` | exact |
| 4 | numbered/index original | 302 | 17454 | `973bf9b1d3f6f1d91dcc04261fc04b96bfbb41c8cc6d47bf2153c08c730dbe3d` | `docs/thefinalglance/C-plan/C-28_testing_trust_t1_mutation.md` | exact |
| 5 | numbered/index original | 335 | 19191 | `d43769eaae135e5a0ccbd1e5e59e786dd7ba6e77354080fbbc459734ab8eb1b2` | `docs/thefinalglance/C-plan/C-32_code_completeness_skeleton_first.md` | exact |
| 6 | numbered/index original | 188 | 9628 | `1c8d63fea0654ea21a247fb899cb39fa4bc6af16dcbc89b4327080660a8fee28` | `docs/thefinalglance/C-plan/C-34_empire_integration_deferred.md` | exact |
| 7 | numbered/index original | 274 | 14425 | `6526502744d6a24ecc8f19ef89feaa3835fd0207c1614f2d67c050a72f558851` | `docs/thefinalglance/C-plan/C-35_tool_attention_dynamic_gating.md` | exact |
| 8 | numbered/index original | 239 | 12294 | `f56e13556ce81e8725188277d640c47f77ba00854aa30e702c262eef419377ae` | `docs/thefinalglance/C-plan/C-36_mucoco_mutation_operator_catalog.md` | exact |
| 9 | numbered/index original | 166 | 7535 | `e7540ed1c1ce1a0e237b43caf6a441fb2a8e2e9f7912c8562c555a51cb8b181e` | `docs/thefinalglance/C-plan/C-44_composer2_runtime_backend_tier.md` | exact |
| 10 | numbered/index original | 136 | 6090 | `5957e84a819e79366a1eee19b06fa84742ea2391b199ac30d943ced3c03ff4d4` | `docs/thefinalglance/C-plan/C-49_iterative_distillation_prompt_policy.md` | exact |

## Per-file ledgers

### C-9 — Autonomous Mode Core Infrastructure

#### Explicit capabilities, workflows, data, and lifecycle

- [E][C9-E01] The plan is a 2–3-session implementation of seven named Phase-4G gaps plus per-role backend config, runtime measurements, and live `SplitTask` dispatch; it depends on C-1 and C-5 and is parallel-safe with C-10. [L3-L16]
- [E][C9-E02] `EvidenceVerdict` has `Pass`, `Fail { reason: String }`, and `Missing`, derives debug/clone/equality/serde traits, and exposes `as_str()` for SSB labels. [L68-L70]
- [E][C9-E03] `EvidenceCheck` holds `artifact_path: Option<PathBuf>`, `description: &'static str`, and `check_fn: fn(&Evidence) -> EvidenceVerdict`; it is synchronous and only inspects the already-derived `Evidence` snapshot. [L72-L76]
- [E][C9-E04] `AutoAdvanceEvaluator` owns `gate_checks: HashMap<FsmTransition, EvidenceCheck>`; `evaluate(transition,evidence)` runs the matching check or returns `Missing` when no transition entry exists. [L78-L82]
- [E][C9-E05] Its default catalog is to be pre-populated with checks associated with `AntiSpiral`, `HighRisk`, `ReplanMajor`, and `CircuitBreaker`; `evaluate_all()` iterates all checks and becomes the path used by `AutonomousGate::decide()` around `Evidence::all_passed()`. [L83-L87]
- [E][C9-E06] `EscalationPolicy` declares `anti_spiral_threshold`, `high_risk_pauses`, and `max_auto_advances_per_phase`; defaults are `3`, `true`, and `10`. [L89-L96]
- [E][C9-E07] `AutonomousGateConfig` gains `escalation_policy` while retaining `circuit_breaker_limit` for backward compatibility; the default phase maximum mirrors the old limit. [L91-L96]
- [E][C9-E08] `EscalationTrigger` gains `SubstituteModelFailed` and `SplitTaskFailed`, serialized/labeled as `substitute_model_failed` and `split_task_failed`. [L98-L101]
- [E][C9-E09] Those triggers fire when the model fallback chain is empty or a split collapses to `sub_task_count=0`, respectively. [L102-L103]
- [E][C9-E10] `MergeStrategy` initially has only `Sequential`; a `Parallel` variant is explicitly deferred until C-10’s anti-spiral fan-out is stable. [L105-L108]
- [E][C9-E11] `SubTask` contains `task_id`, `scope`, `context_budget`, and `depends_on`, with debug/clone/equality/serde derives. [L110-L112]
- [E][C9-E12] `SplitTaskPlan` contains `parent_phase`, `sub_tasks`, `merge_strategy`, `max_depth`, and `current_depth`. [L114-L116]
- [E][C9-E13] `AutonomyDecision::SplitTask` replaces bare count/depth fields with `split_plan: SplitTaskPlan`, and every destructuring match in `task_runner.rs` must change with it. [L117-L119]
- [E][C9-E14] `ModelPool::can_fit_llm(model_size_bytes)` is a default trait method implementing `available_vram() >= model_size_bytes`, preserving existing mock implementations. [L121-L126]
- [E][C9-E15] Before returning `SubstituteModel`, `AutonomousGate::decide()` applies the fit check and escalates with `SubstituteModelFailed` if estimated model size exceeds VRAM. [L127-L129]
- [E][C9-E16] CLI startup replaces `autonomous_gate: None` with a real `AutonomousGate` configured from the parsed mode and otherwise-default config. [L131-L135]
- [E][C9-E17] `GateMode::from_cli_args()` consumes C-5’s `--mode` value and defaults to `Interactive` when the flag is absent. [L134-L136]
- [E][C9-E18] Example runtime config adds `[agents.executor.backend] backend="auto"`, `[agents.auditor.backend] backend="anthropic_api"`, and `[agents.designer.backend] backend="anthropic_api"`. [L138-L149]
- [E][C9-E19] `BackendStanza { backend: String }` is added to runtime config; each role receives an optional stanza whose absent value defaults to `auto`. [L150-L152]
- [E][C9-E20] The live split arm first retains its optional splitter and otherwise calls `ResearchTree::split_step(&SplitTaskPlan)`. [L154-L157]
- [E][C9-E21] `split_step` locates the active node through `parent_phase`, clones planned subtasks, and corrects `task_id` sequencing. [L157-L159]
- [E][C9-E22] If no research-tree handle is reachable at that dispatch site, the path emits `SplitTaskFailed` and escalates rather than manufacturing a split. [L160-L162]
- [E][C9-E23] `[runtime_measurements]` records auto-advance accuracy, false-escalation rate, mode-switch latency, split overhead, substitute-model swap latency, autonomous/interactive end-to-end ratio, gate-evaluation overhead, escalation count per session, and `completion=true` after the smoke run. [L164-L175]
- [E][C9-E24] The cloud Sonnet 4.6 smoke invokes autonomous mode to build a tested four-method Rust calculator, captures SSB rows, and supplies the runtime measurements. [L177-L181]
- [E][C9-E25] The decision artifact records why the fit method is a trait default, why only sequential merge exists, and why headless cloud wiring has no `interactive_gate`/`InterventionCard` route. [L183-L187]

#### Explicit integration, evidence, failures, constraints, and done gates

- [E][C9-E26] Produced/reachable surfaces are `AutoAdvanceEvaluator`, `EvidenceCheck`, `EvidenceVerdict`, `EscalationPolicy`, split types, two triggers, the fit method, CLI mode parser, `ResearchTree::split_step`, backend TOML/runtime types, and Phase-4G measurements, reachable from CLI config, gate decision, split dispatch, and the existing MonitorWindow statistics panel. [L189-L217]
- [E][C9-E27] The evaluator may not return `Pass` when `Evidence::all_passed()` is false. [L218-L220]
- [E][C9-E28] The default fit implementation may not always return true. [L221]
- [E][C9-E29] A non-empty split plan may not become an empty vector; that condition escalates. [L222-L223]
- [E][C9-E30] Observable evidence requires a `gate_decision` row with `decision=AutoAdvance` and `source=AutoAdvanceEvaluator`; `escalation_fired` is conditional on stagnation. [L224-L229]
- [E][C9-E31] Artifact/UI evidence requires populated measurements with completion true and a non-zero MonitorWindow `AutoAdvanceStats.last_5min_count`. [L230-L232]
- [E][C9-E32] Deterministic inspection checks every new named type/variant/config path, proves the old `autonomous_gate: None` text is absent, verifies `split_step`, builds the workspace, runs scoped orchestrator/contracts tests, and confirms measurement keys. [L234-L274]
- [E][C9-E33] Acceptance additionally requires six escalation variants, the fit check on the actual substitution path, real gate construction, parsed backend stanzas, live split dispatch, the autonomous run to complete without prompting, the sidebar to display `AUTONOMOUS`, clean scoped builds/tests, and the three-decision document. [L276-L304]
- [E][C9-E34] Implementation touches autonomous gate, model pool, research tree, task runner, CLI startup/runtime config, example TOML, Phase-4G metrics, and one new decision document. [L306-L328]
- [E][C9-E35] The next direct continuation is C-10, although C-11 is also permitted by prioritization; the stated non-overlap is that C-10 does not touch `autonomous_gate.rs` or `task_runner.rs`. [L330-L335]
- [E][C9-E36] Every acceptance box is unchecked; this original specifies planned work and does not claim the C-9 implementation is shipped. [L276-L304]

#### Inferred, ambiguous, and conflicting

- [I][C9-I01] Backward compatibility is intended at config construction and mock-trait implementation boundaries, not merely at serialization, because both the old circuit-breaker field and a trait default are retained. [L91-L96, L121-L126]
- [I][C9-I02] Split recovery is fail-closed: inability to materialize a non-empty plan becomes an escalation rather than silent continuation. [L154-L162, L222-L223]
- [A][C9-A01] `gate_checks` is keyed by `FsmTransition`, but the default population is described as “one entry per `EscalationTrigger` variant”; the mapping/key construction is not specified. [L78-L87]
- [A][C9-A02] `evaluate_all()` is said both to iterate the gate map and to “wrap” `Evidence::all_passed()`; exact aggregation rules for `Pass`, `Fail`, and `Missing` are absent. [L83-L87, L192-L203]
- [C][C9-C01] The task defines exactly three `EscalationPolicy` fields, while acceptance says “4 fields”; the orientation summary repeats four but omits names. No fourth field is named. [L89-L96, L283]
- [C][C9-C02] The measurements block lists nine TOML assignments including `completion`, while acceptance and the orientation summary call them “all 8 keys.” [L164-L175, L299-L300]
- [A][C9-A03] `SubstituteModelFailed` is described for both an empty fallback chain and insufficient VRAM; precedence and whether the attempted target/fallback identity is retained are unspecified. [L102-L103, L127-L129]
- [A][C9-A04] `split_step` “corrects task_id sequencing,” but its sequencing algorithm, duplicate handling, dependency rewrite, depth enforcement, merge operation, and idempotency are not defined. [L154-L162]
- [C][C9-C03] The objective/context call the missing production object `TreeAutoAdvancePolicy`, but Task 10 constructs `AutonomousGate`; the orientation summary says a real `TreeAutoAdvancePolicy` is passed. The relationship between those identifiers is not resolved. [L7, L37-L38, L131-L136]
- [A][C9-A05] The new backend section syntax nests `.backend` and also contains a key named `backend`; exact serde nesting for every role is not shown. [L138-L152]

#### Summary additions and corrections

- [D+][C9-D01] Add exact verdict/check/evaluator schemas, missing-transition behavior, aggregation uncertainty, policy defaults, trigger labels/firing conditions, split schemas/semantics, fit formula, CLI default mode, exact backend defaults, tree-unavailable escalation, decision rationale, and the three stub bans. [L68-L162, L183-L187, L218-L223]
- [D+][C9-D02] Add all nine measurement assignments and the conditional nature of `escalation_fired`; the summary only says eight keys. [L164-L181, L224-L232]
- [D~][C9-D03] Replace the summary’s unqualified “EscalationPolicy with four fields” with the explicit three-field schema plus the unresolved acceptance conflict. [L89-L96, L283]
- [D~][C9-D04] Qualify the summary’s `TreeAutoAdvancePolicy` wiring statement with the source’s concrete `AutonomousGate::new(...)` construction and unresolved identifier relationship. [L131-L136]
- [D~][C9-D05] Treat “closes phase-4G and B-22” as orientation shorthand: the source says it implements named gaps from those reports, not that every report item is already closed. [L3-L10, L20-L50]

### C-16 — Cosmos Pipeline Integration

#### Explicit capabilities, workflows, data, and lifecycle

- [E][C16-E01] The two-session plan closes five B-42 gaps: tiered pipeline insertion, hourly worker scheduling, a real manual refresh command, SSB emission, and five standalone tests; it depends on C-2/C-8 and is parallel-safe with C-17 through C-20. [L1-L9]
- [E][C16-E02] The source preserves the live `3 * cadence` decay threshold as an intentional grace period before shadow serving, rather than reverting to the older `2 * cadence` specification. [L30-L34]
- [E][C16-E03] The decision record must capture stage position, 3x rationale, optional SSB injection into `tick`, and an in-process one-shot Tokio task with no extra thread. [L49-L53]
- [E][C16-E04] `QueryPipeline` gains `cosmos_stage: Option<CosmosStage>` and `with_cosmos_stage()`. [L55-L56]
- [E][C16-E05] `CosmosStage` wraps an `Arc<CosmosWorker>`, calls `cosmos_get()` for query-relevant artifacts, and returns `Vec<RetrievalResult>` with `tool_used="cosmos"` and stage label `cosmos`. [L56-L58]
- [E][C16-E06] Stage order is `ArtifactStage -> CosmosStage -> ARagStage`; FAST skips Cosmos while STANDARD and DEEP execute it. [L58-L60]
- [E][C16-E07] `CosmosWorker::new()` receives an optional shared `Ssb` handle (or an emitter trait), allowing tests to use `None` and production to use `Some`. [L62-L68]
- [E][C16-E08] `tick()` emits `CosmosRefreshStarted` before its manifest loop with `artifacts_queued=stale+decayed count` and `manifest_size`. [L62-L65]
- [E][C16-E09] `refresh_one()` emits `CosmosArtifactUpdated` on `Ok(true)` and `CosmosArtifactDecayed` when `cosmos_get()` reports decayed. [L65-L66]
- [E][C16-E10] SSB emission augments rather than replaces tracing, preserving non-SSB observability. [L66-L68]
- [E][C16-E11] Task-runner startup spawns a repeating `tick -> info TickReport -> sleep(3600s)` Tokio task. [L70-L73]
- [E][C16-E12] Cancellation checks before sleep and uses `tokio::select!` between cancellation notification and sleeping so `CancelHandle::cancel()` terminates the loop. [L73-L75]
- [E][C16-E13] Production construction supplies manifest, root, clock, and a cloud or sidecar `ResearchDispatcher` selected by `ExecutionMode`. [L75-L77]
- [E][C16-E14] `trigger_cosmos_refresh` immediately returns `TriggerCosmosRefreshResponse { queued, artifacts_due }` while a one-shot `tick()` runs in the background with the same startup-injected dispatcher. [L79-L83]
- [E][C16-E15] Cold start is best-effort: when the dispatcher is not initialized, the command returns `queued=false` and a reason rather than raising `NotImplemented`. [L82-L84]
- [E][C16-E16] `CosmosDrawer` must subscribe to `CosmosRefreshStarted`, `CosmosArtifactUpdated`, and `CosmosArtifactDecayed` so updates need not wait for its 30-second poll. [L86-L90]
- [E][C16-E17] `CosmosGroundingUsed` is informational: log it to the browser console and do not mutate UI state. [L89-L90]
- [E][C16-E18] Refresh callers must acquire `IndexLock(LockScope::Index)` before `tick()`, and refresh-start emission must precede the loop. [L121-L126]
- [E][C16-E19] Observable smoke configures a daily knowledge artifact, performs a cloud research query, expects queued/updated/query-stage SSB evidence, fresh status within a tick, live drawer timestamp, and an `AVAILABLE_COSMOS` briefing block. [L127-L136]

#### Explicit tests, recovery, determinism, constraints, and done gates

- [E][C16-E20] `cosmos_worker_tick.rs` covers first-write body/meta, fresh skip, stale/decayed refresh, unchanged-body idempotence, failure recording without panic, TickReport counts, concurrent workers, and `Some`/`None` SSB behavior. [L212-L219]
- [E][C16-E21] `cosmos_lock_contention.rs` requires atomic temp+rename writes, post-concurrency blake3/body agreement, and a 4-thread × 5-tick stress over non-overlapping manifest slices with zero `Blake3Mismatch`. [L221-L226]
- [E][C16-E22] `cosmos_pipeline_stage.rs` proves STANDARD visibility, empty-before-refresh, non-empty-after-refresh, FAST omission, and non-fatal `MissingBody` stage errors with pipeline continuation. [L228-L234]
- [E][C16-E23] `cosmos_decay_contract.rs` fixes statuses at Fresh below 1x, Stale from 1x to below 3x, Decayed at/after 3x for all four cadences; decayed lookup warns and returns a value, while blake3 mismatch is an error. [L236-L241]
- [E][C16-E24] `cosmos_briefing_budget.rs` caps 200 tokens at 800 chars, drops lower relevance when over budget, excludes below-threshold entries, orders Fresh before Stale, excludes Decayed when a Fresh same-class alternative exists, and maps an empty manifest to an empty string. [L243-L248]
- [E][C16-E25] Acceptance requires the stage/builder and tier behavior, all three conditional emissions, `None` safety, cancel-safe 3600-second loop, lock acquisition, immediate manual response, live UI, documented 3x divergence, all five no-live-inference suites, workspace check, and a manual cloud Sonnet smoke. [L179-L210]
- [E][C16-E26] Deterministic commands inspect stage wiring/emissions/scheduler/manual-stub removal, check all five files, confirm the 3x comment, and run only knowledge/orchestrator/contracts Cosmos scopes plus workspace check. [L138-L177]
- [E][C16-E27] Out of scope are vector embedding of whole Cosmos briefs, config-driven manifest merging, verifier changes, artifact CRUD/cadence UI, a worker-owned async `spawn_loop`, and the full workspace test suite. [L250-L262]
- [E][C16-E28] Every acceptance box is unchecked; B-42 is described as previously mostly done, but this C-16 wiring/test closure is not claimed shipped. [L13-L28, L179-L210]

#### Inferred, ambiguous, and conflicting

- [I][C16-I01] The design has two refresh initiators—periodic and manual—but a shared dispatcher and caller-owned index lock are intended to serialize the same underlying worker path. [L70-L84, L121-L124]
- [I][C16-I02] Stage errors are deliberately fail-soft for retrieval, whereas data-integrity mismatch remains fail-hard in the worker contract. [L228-L241]
- [A][C16-A01] The declared response shape has only `queued` and `artifacts_due`, but cold-start behavior also requires a `reason` field; its optionality/type is not defined. [L79-L84, L105-L106]
- [A][C16-A02] The ownership/location and contention behavior of `IndexLock` around both periodic and manual one-shot refresh are not specified beyond “caller responsibility.” [L70-L84, L121-L124]
- [A][C16-A03] “Relevant artifacts given the query” has no matching/ranking rule, budget, ordering, or duplicate policy. [L55-L60]
- [A][C16-A04] Cancellation is specified for the loop around calls but not for an in-progress synchronous `tick()` or manual one-shot task. [L70-L84]
- [A][C16-A05] `CosmosArtifactDecayed` is assigned to `refresh_one()` based on a `cosmos_get()` result, but the exact call point and whether one artifact may emit both decayed and updated in a refresh are not defined. [L62-L66]
- [C][C16-C01] The orientation summary says this plan “closes B-42 and B-35”; the original’s objective names B-42, while B-35 appears only as part of the divergence audit trail, not an explicit closure target. [L1-L5, L92-L94]

#### Summary additions and corrections

- [D+][C16-D01] Add `CosmosStage` input/output semantics, `tool_used`, exact refresh payload counts, dispatcher selection, cold-start behavior, console-only grounding event, and the continued tracing channel. [L55-L90]
- [D+][C16-D02] Add every standalone test behavior, especially idempotence, atomic concurrent writes, fail-soft stage errors, exact 1x/3x status boundaries, relevance/budget ordering, and blake3 mismatch failure. [L212-L248]
- [D+][C16-D03] Add the six explicit non-goals and the fact that the scoped suite, not a full workspace test run, is owned by C-16. [L250-L262]
- [D~][C16-D04] Remove or qualify “closes B-35”; source evidence supports a B-35 audit-trail reference, not an explicit closure statement. [L92-L94]

### C-24 — Symbol Impact Dashboard Surface

#### Explicit capabilities, schemas, workflows, and surfaces

- [E][C24-E01] The two-session plan closes B-29/B-33 reachability, incremental update, CLI/dispatcher export, decomposition styling, blast-radius UI, and missing tests; it depends on C-17 and is parallel-safe with C-21/22/23/25. [L1-L9]
- [E][C24-E02] The existing six-tool agent catalog gains `symbol_impact(target:string, depth:u8=2)`, `symbol_flow(target:string, depth:u8=5)`, `symbol_context(target:string)`, and `symbol_list(prefix:string)`. [L72-L78]
- [E][C24-E03] Incremental indexing calls `update_changed_files_symbol_graph(changed_paths)` when `changed_file_count <= INCREMENTAL_SYMBOL_THRESHOLD` (declared as 50); larger changes use full `build_symbol_graph`. [L80-L87]
- [E][C24-E04] The incremental branch emits `SsbActionType::SymbolGraphUpdated` with `is_incremental=true`. [L85-L87]
- [E][C24-E05] KnowledgeDrawer’s impact panel adds Svelte-5-state target input and `Run`, invoking `submit_task` with `"/symbol_impact target=" + target` and rendering the returned `blast_radius`. [L89-L94]
- [E][C24-E06] CLI gains `SlashCommand::GraphMermaid { mode: GraphMode, max_nodes: usize }`, delegates via IPC to `export_graph_mermaid`, and prints returned Mermaid to stdout. [L96-L103]
- [E][C24-E07] CLI modes are `file`, `symbol`, and `decomposition`; `max_nodes` defaults to 200. [L101-L103]
- [E][C24-E08] Dispatcher gains `DispatchTool::ExportGraphMermaid` with input `{project_id:string, mode:string, max_nodes:usize}` and output `{mermaid:string}`, delegating to the knowledge generator used by Tauri. [L105-L112]
- [E][C24-E09] Decomposition style maps `Concept` to a 180-wide 3px large rounded rectangle, `Phase` to 140/2px medium rounded, `Step` to 100/1px small rounded, and `BabyStep` to a 60-wide circle. [L114-L119]
- [E][C24-E10] Decomposition statuses map pending to `#9e9e9e`, running to `#1976d2` with a transition/pulse intent, passed to `#388e3c`, failed to `#d32f2f`, and blocked to `#f57c00`. [L120-L124]
- [E][C24-E11] Level/status classes apply only when `graphMode === "decomposition"`. [L125]
- [E][C24-E12] Every file-write SSB row gains `Show blast radius`; clicking runs graph-store `request_impact(filePath)`, sets file mode, and opens GraphWindow through workspace state. [L127-L133]
- [E][C24-E13] A decision document records tool-schema shape, threshold-branch placement, decomposition guard, and cross-link store-call choices. [L151-L155]

#### Explicit tests, observability, determinism, constraints, and done gates

- [E][C24-E14] `graph_export_dispatch.rs` constructs a minimal IPC request, requires an OK Mermaid header, byte-equal repeat output for identical input, and a non-empty `%% empty graph` representation for `VizData::empty()`. [L135-L142]
- [E][C24-E15] `graph_commands.rs` requires file-mode fixture data, a non-empty known-symbol impact overlay, symbol-mode `classDef`, and no panic/error for all three commands on an empty fixture. [L144-L149]
- [E][C24-E16] Catalog completion forbids exposing only a subset of the four symbol tools. [L189-L191]
- [E][C24-E17] Incremental indexing may no longer call full graph construction unconditionally. [L192-L193]
- [E][C24-E18] CLI export must use the real generator, and dispatcher export must delegate rather than duplicate generation. [L194-L197]
- [E][C24-E19] ActivityWindow must use the shared graph store rather than invoke Tauri directly. [L198-L199]
- [E][C24-E20] Cloud rename smoke expects a `SymbolQuery` row with `tool="symbol_impact"`, target `SsbActionType`, and non-empty blast radius; UI/artifact evidence includes decomposition styling, deterministic zero-LLM CLI Mermaid, Tauri parity, and at least ten callers in the drawer. [L204-L213]
- [E][C24-E21] Acceptance raises the catalog to at least ten tools, tests both threshold branches/event, checks the form, CLI, cross-layer deterministic export, guarded styles, shared-store action, both four-case suites, scoped Rust builds, Svelte check, and the decision document. [L255-L291]
- [E][C24-E22] Implementation modifies production RPC catalog, indexing service, dispatcher, CLI, three Svelte surfaces, and adds two test files plus one decision artifact. [L293-L311]
- [E][C24-E23] Deterministic Mermaid repeatability is attributed to `BTreeMap` iteration in the existing generator. [L135-L142]
- [E][C24-E24] Every acceptance box is unchecked; the plan does not claim these reachability/UI/test additions are shipped. [L255-L291]

#### Inferred, ambiguous, and conflicting

- [I][C24-I01] The tool catalog and dispatcher arms form a two-step reachability contract: existing implementations are not agent-usable until their JSON definitions are prompt-visible. [L13-L19, L72-L78]
- [I][C24-I02] Dispatcher and Tauri export are intended as two adapters over one deterministic knowledge implementation, not two serialization authorities. [L105-L112, L194-L197]
- [A][C24-A01] Examples pass `SsbActionType` after `/graph mermaid`, but `GraphMermaid` declares no target argument and the dispatcher schema declares no target field; parsing/meaning of that positional token is unresolved. [L96-L112, L248-L249, L272-L274]
- [A][C24-A02] CLI does not expose `project_id` even though dispatcher input requires it; implicit current-project resolution is not stated. [L96-L112]
- [A][C24-A03] The consumed graph store mentions `trigger_refresh`, but the click workflow names only `request_impact`, `set_mode`, and `openWindow`; whether impact implicitly refreshes is unspecified. [L127-L133, L178-L179]
- [A][C24-A04] A CSS/Cytoscape `transition` property is called a running “pulse,” but no timing/keyframe behavior is specified. [L120-L124]
- [A][C24-A05] The cloud trigger is an impact/file graph while the expected artifact specifically calls for decomposition styles; no mode transition to decomposition is defined in that workflow. [L204-L213]
- [A][C24-A06] The empty-graph sentinel is called a “stub” in acceptance even though the stub ban requires real generation; it appears to be an intentional valid empty representation. [L135-L142, L194-L197, L284-L285]

#### Summary additions and corrections

- [D+][C24-D01] Add all four JSON schemas/defaults, the full-graph `> threshold` branch, exact submit string, CLI modes/max-node default, dispatcher request/response schema, exact Cytoscape shapes/colors, and store-call ordering. [L72-L133]
- [D+][C24-D02] Add all eight named test assertions, zero/empty behavior, Tauri/dispatcher parity obligation, catalog completeness prohibition, and exact build/UI gates. [L135-L149, L189-L213, L255-L291]
- [D~][C24-D03] Qualify the summary’s CLI example: the original does not define where its `SsbActionType` positional argument maps. [L96-L112, L248-L249]

### C-28 — Testing-Trust T-1 Mutation Testing

#### Explicit capabilities, schemas, configuration, and workflow

- [E][C28-E01] This two-session T-1 extension depends on C-27, is parallel-safe with C-29/C-21..26, and inserts mutation after an existing five-check deterministic chain. [L1-L9, L13-L22]
- [E][C28-E02] The source pins `cargo-mutants 27.0.0` as an installed CLI/dev tool, explicitly not a Cargo dependency, and `mutmut 3.5.0` in the sidecar Python dev environment. [L24-L26, L46-L50, L257-L258]
- [E][C28-E03] `ValidationTier::Mutation` sits between `Unit` and `Integration`; `as_str()` is `mutation`, timeout sentinel is `0` for runner-owned timing, and `ordered()` becomes six tiers with Mutation at index 3. [L52-L59]
- [E][C28-E04] `MutantOutcome` is `Caught | Survived | Timeout | Skipped(String)`. [L61-L70]
- [E][C28-E05] `MutantRecord` contains file, line, mutant kind, and outcome. [L65-L70]
- [E][C28-E06] `MutationResult` contains records, `mutation_score_pct`, `total_run`, and `cache_hits`; score is `caught/(caught+survived+timeout)*100`. [L71-L76]
- [E][C28-E07] Subprocess failures become a `thiserror` `MutationError`; production mutation code forbids unwrap/expect. [L78, L94-L96]
- [E][C28-E08] Async `MutationRunner::run(cfg, project_dir, cache)` returns `Result<MutationResult, MutationError>` and is `Send + Sync`. [L80-L87]
- [E][C28-E09] `RustMutationRunner` uses Tokio to run `cargo mutants --timeout N --output tmp --jobs 1 --file files`, parses caught/missed output, caps work at `max_mutants_per_session`, and scores the partial subset. [L88-L91]
- [E][C28-E10] `PythonMutationRunner` uses Tokio to run `mutmut run --paths-to-mutate files --runner "pytest -x -q"`, then reads `mutmut results`. [L92-L94]
- [E][C28-E11] Both runners enforce a per-mutant Tokio timeout. [L94-L96]
- [E][C28-E12] `EquivalentMutantCache` reuses the SSB SQLite-pool pattern and keys blake3 over pre-mutation bytes + mutant kind + line. [L98-L101]
- [E][C28-E13] Cache schema is `equiv_mutant_cache(key TEXT PRIMARY KEY, outcome TEXT, ts INTEGER)` at `<project>/.elai/mutation_cache.sqlite3`; it is created on demand. [L101-L104]
- [E][C28-E14] Cache hits become `Skipped("cache_hit")`; new outcomes are inserted, duplicate keys use upsert semantics in tests, and rows older than 30 days are rerun. [L102-L104, L157-L159]
- [E][C28-E15] `[testing.mutation]` defaults are `enabled=true`, `threshold_pct=70.0`, `timeout_per_mutant_sec=30`, and `max_mutants_per_session=100`; absent config deserializes to these defaults without error. [L106-L117]
- [E][C28-E16] `SsbActionType::MutationSurvived` and `MutationSurvivedPayload { file, line, mutant_kind }` are added across enum/SQL CHECK/string/from-string sites. [L119-L126]
- [E][C28-E17] One event is emitted per survivor and none for caught, timeout, or skipped records. [L123-L125]
- [E][C28-E18] Mutation executes only after Build, VersionGate, FileSizeCensus, LazyCode, and Completeness all pass. [L128-L138]
- [E][C28-E19] Changed Rust and Python files select the corresponding runners by extension; scores below threshold create blocking `DeterministicFinding` with `CheckKind::Mutation` and the numeric threshold message. [L130-L138]
- [E][C28-E20] Below-threshold mutation blocks the `AUDITING -> MEMORIZING` transition rather than warning. [L133-L138, L192-L197]
- [E][C28-E21] `MutationGaugeInfo` carries score, threshold, cache hits, total run, and last-updated milliseconds. [L140-L145]
- [E][C28-E22] MonitorWindow polls `get_mutation_gauge` every five seconds and renders green at/above threshold, amber below it, and grey for no result; the Tauri stub returns `None` until backend session-state wiring exists. [L145-L150]

#### Explicit tests, observability, recovery, and done gates

- [E][C28-E23] The four runner-contract tests cover public schema 47, a 7/2/1 = 70% score, cache skip/upsert, and serde payload round-trip. [L152-L162]
- [E][C28-E24] The three schema-contract tests cover public schema 47, SSB SQL/from-string round-trip, and Mutation’s ordered index. [L163-L169]
- [E][C28-E25] The wiring contract bans mocked production runners, non-blake3 cache keys, and warning-only auditing behavior. [L171-L197]
- [E][C28-E26] Normal observable behavior has zero survivor rows, score at least 70, transition proceeds, green gauge, and faster repeat from cache hits. [L198-L203]
- [E][C28-E27] The adversarial tautology scenario yields score 0, a blocking finding, blocked transition, and amber gauge; replay expects every second-run mutant to be a cache hit and `total_run == cache_hits`. [L239-L253]
- [E][C28-E28] Deterministic checks inspect version/types/config/blake3/no-unwrap and run the two contract binaries without live subprocess mutation. [L205-L237]
- [E][C28-E29] Acceptance covers exact dependency installs, tier/order, runner/types/formula, persistent TTL cache, event cardinality, four config keys/defaults, hard gate ordering, three-state gauge, null stub, no unwrap/expect, 4+3 contracts, normal/adversarial smoke, and faster all-cache replay. [L255-L281]
- [E][C28-E30] Implementation adds mutation code and two test files and modifies tier dispatch, auditor, SSB, runtime config/TOML, dashboard types/window/Tauri state, plus a decision document. [L283-L297]
- [E][C28-E31] Every acceptance box is unchecked; dependency versions are reported verified, but the T-1 implementation is not claimed shipped. [L24-L28, L255-L281]

#### Inferred, ambiguous, and conflicting

- [I][C28-I01] The cache is intended to make repeated identical pre-mutation inputs idempotent with respect to execution cost while preserving a record of skipped outcomes. [L98-L104, L250-L253]
- [I][C28-I02] Mutation is both a validation tier and an FSM gate: the ordered tier exposes it to generic validation, while deterministic-auditor integration owns the state-blocking decision. [L52-L59, L128-L138]
- [C][C28-C01] The original says C-28 bumps schema 46→47, while the orientation summary says Mutation shares and keeps C-27’s already-bumped 47. Cross-plan migration order cannot be resolved from this file. [L28, L119-L126]
- [C][C28-C02] The adversarial paragraph first says no surviving-mutant rows are emitted because outcomes are `Survived`, then immediately says such rows are emitted one per survivor; Tasks/Acceptance consistently require emission. [L241-L248]
- [A][C28-A01] `cargo-mutants --output` is described as JSON output and then as `caught.txt + missed.txt`; the exact parser format/version contract is unclear. [L88-L90]
- [A][C28-A02] A single external command is shown, but timeout is required “per mutant”; process subdivision/cancellation and partial-output recovery are not specified. [L88-L96]
- [A][C28-A03] Task routing says detect languages by changed-file extension, while the wiring contract says consume `ProjectProfile.commands`; their authority/precedence is unspecified. [L130-L132, L183-L186]
- [A][C28-A04] Score denominator behavior when no mutant runs (all cached/skipped, no applicable files, or tool failure) is not defined. [L71-L76, L98-L104]
- [A][C28-A05] Cache `outcome` persistence does not define which outcomes qualify as “equivalent,” how timeout/survivor rows are updated, or whether 30-day TTL cleanup is transactional under concurrent runs. [L98-L104]
- [A][C28-A06] `get_mutation_gauge` is explicitly a stub until live state wiring, yet acceptance otherwise describes a live cloud gauge; the handoff that makes it non-stub is not a separate task. [L140-L150, L272-L280]

#### Summary additions and corrections

- [D+][C28-D01] Add exact record/result/cache/config/gauge schemas, runner commands, partial-cap behavior, event cardinality exclusions, five predecessor checks, and every contract-test case. [L52-L169]
- [D+][C28-D02] Add the all-cache replay invariant, zero-result score ambiguity, parser/timeout uncertainties, and the self-contradictory adversarial event sentence. [L239-L253]
- [D~][C28-D03] Preserve the unresolved schema chronology instead of stating without qualification that C-28 merely keeps 47. [L28, L119-L126]
- [D~][C28-D04] Correct the summary’s claim that both tools are non-Cargo CLI installs: `cargo-mutants` is a Cargo-installed CLI, while `mutmut` is a Python package in sidecar `pyproject.toml` dev dependencies. [L24-L26, L46-L50, L257-L258]

### C-32 — Code-Completeness Layer 2

#### Explicit types, algorithms, workflow, and surfaces

- [E][C32-E01] This 2–3-session layer depends on C-31 FilePlan and C-26 decoding config, is parallel-safe with C-21..30, and adds skeleton-first generation, per-fill criticism, structural recovery, deterministic bridge files, two dashboard views, and five SSB types. [L1-L13]
- [E][C32-E02] `BlankId` wraps `u8` and displays `BLANK_{n}`. [L56-L59]
- [E][C32-E03] `SkeletonSpec` contains file path, language, blank IDs, and skeleton text; `FillResult` contains blank ID, body, and critic verdict; both are serializable. [L59-L62]
- [E][C32-E04] `FilledBuffer` stores the spec, fills, and `f32` completeness; ratio is non-BLOCK fills / blank count, clamped to `[0,1]`, with zero blanks equal to 1.0. [L63-L65]
- [E][C32-E05] `SkeletonGeneratorConfig` defaults to `max_blanks=32` and `min_body_lines=3`. [L66]
- [E][C32-E06] `generate(FilePlan)` preserves signatures, replaces each body with `// BLANK_N`, and returns `TooManyBlanks` above the cap. [L67-L68]
- [E][C32-E07] `fill_blanks` holds all fills in memory, runs `MiniCritic::check` for each, returns successfully even with BLOCK verdicts, and performs no disk writes. [L69-L71]
- [E][C32-E08] `flush_to_disk` substitutes accepted bodies, leaves BLOCK entries as `// BLANK_N — not filled`, performs one `fs::write`, and returns the path. [L72-L74]
- [E][C32-E09] `SkeletonError` includes `TooManyBlanks`, `FlushFailed`, and `NoFillForBlank`; skeleton generation/flushing emits `skeleton_generated(blank_count,language)` and `skeleton_flushed(path,ratio)`. [L75-L78]
- [E][C32-E10] `MiniCriticVerdict` is `PASS | WARN | BLOCK`; `MiniCriticResult` carries blank ID, verdict, and reason; `MiniCritic` has `tree_sitter_enabled`. [L80-L87]
- [E][C32-E11] Critic order is lazy-pattern regex, minimum lines, then optional tree-sitter, with first BLOCK short-circuiting. [L87-L97]
- [E][C32-E12] Lazy matches (`todo!`, `unimplemented!`, `pass`, ellipsis, TODO/FIXME forms) block with `lazy_pattern:<pattern>` using shared Tier-1 regex statics. [L88-L91]
- [E][C32-E13] Fewer than three lines blocks as `too_short:<N>_lines`, except a trimmed body with no newline is treated as a single-expression exemption. [L92-L93]
- [E][C32-E14] Enabled tree-sitter wraps fragments in a language shell; an ERROR node blocks with `parse_error:<kind>`, while an unregistered language warns with `ts_unavailable`. [L94-L96]
- [E][C32-E15] Every critic call emits `mini_critic_verdict(blank_id,verdict,reason)`. [L97]
- [E][C32-E16] Structural stages are `Decompose`, `Template`, `FewShot`, and `BestOfN`; a plan stores stages/current index/blank, and `current_stage()` is optional without unwrap. [L100-L106]
- [E][C32-E17] Outcomes are `Converged { fill }`, `AcceptAndComplete { ratio }`, or `Exhausted`; config defaults `max_best_of_n=3`. [L107-L110]
- [E][C32-E18] Each retry stage runs MiniCritic; PASS converges. Decompose creates 2–3 substeps, Template uses `FixTemplateRegistry`, FewShot prepends `few_shot_for_symbol()`, and BestOfN generates N candidates. [L111-L116]
- [E][C32-E19] BestOfN scores compile +2, critic PASS +2, and lines +1, chooses the highest, converges when score > 0, otherwise accepts completeness at least 0.5 or exhausts. [L116-L118]
- [E][C32-E20] Every stage emits `structural_retry_stage_attempt`; no prompt may contain instructional “try again” text. [L119-L120]
- [E][C32-E21] `RetryContext` contains project root, language, function signature, and existing fills. [L121-L123]
- [E][C32-E22] Bridge kinds are Rust `mod.rs`, Python `__init__.py`, TypeScript index barrel, and Go package doc; a spec carries kind/path/module names and the template carries `ELAI_SEED` defaulting to 0. [L125-L132]
- [E][C32-E23] Bridge generation is LLM-free and deterministic: module names sort lexically, each language gets its declared line form, and output gets an auto-generated/do-not-edit header. [L132-L136]
- [E][C32-E24] `write_bridge` does generate + one write, returns `WriteFailed { path, source }` on I/O failure, and emits `bridge_file_generated(kind,path,module_count)`. [L137-L141]
- [E][C32-E25] For baby steps with FilePlan, task runner generates a skeleton before the first LLM call, makes one LLM call per blank, criticizes between fills, retries BLOCK, accepts converged output, and flushes only after all fills or opted-in partial acceptance. [L143-L152]
- [E][C32-E26] `AcceptAndComplete` requires ratio at least 0.5 **and** a user opt-in flag. [L149-L151, L306-L307]
- [E][C32-E27] MonitorWindow calls `get_completeness_ratio() -> f32`, renders red below .5/yellow .5..<.9/green >=.9, and updates from `skeleton_flushed` without a new dependency. [L154-L160]
- [E][C32-E28] ReasoningDrawer renders blank/verdict/reason, color maps PASS/WARN/BLOCK to green/yellow/red, and has a default-on `Show MiniCritic` filter using the existing SSB stream. [L162-L170, L251-L254]
- [E][C32-E29] SSB registry adds `skeleton_generated`, `skeleton_flushed`, `mini_critic_verdict`, `structural_retry_stage_attempt`, and `bridge_file_generated`. [L171-L175]

#### Explicit tests, determinism, constraints, and done gates

- [E][C32-E30] The contract list verifies blank count/sequential IDs/all placeholders, exactly one flush write, all/partial/zero ratios, TODO and short blocks, multiline pass, retry order, deterministic/sorted/no-network bridge generation, and 0.5 versus 0.49 acceptance. [L177-L204]
- [E][C32-E31] Hard bans require a single disk write, no bridge LLM/network call, reuse rather than duplication of Tier-1 regexes, no acceptance below .5, and structural-only retry prompts. [L240-L249]
- [E][C32-E32] Main smoke at seed 42 requires `skeleton_generated` first, at least four verdict rows, flush ratio >= .9, no blank markers, and successful cargo check. [L256-L261]
- [E][C32-E33] Adversarial TODO fill must block and emit stage attempts before convergence or opted-in partial acceptance; bridge output must be byte-identical across two seed-42 runs. [L262-L265]
- [E][C32-E34] Deterministic inspection checks all public types, exactly one skeleton-file write, zero local regex construction, no bridge backend/network symbols, all SSB names, and the scoped contract test. [L267-L294]
- [E][C32-E35] Acceptance additionally requires no unwrap in production and a clean workspace check. [L296-L317]
- [E][C32-E36] Implementation adds four orchestrator modules and one contract file, modifies task runner/SSB/Monitor/Tauri, locates and modifies ReasoningDrawer, and adds a decision artifact. [L319-L335]
- [E][C32-E37] Every acceptance box is unchecked; this is a planned completeness layer, not a shipped-status record. [L296-L317]

#### Inferred, ambiguous, and conflicting

- [I][C32-I01] The intended write lifecycle is transactional at file granularity: all generated/critic/retry state remains memory-only until one final materialization. [L69-L74, L143-L152, L240-L242]
- [I][C32-I02] WARN counts as complete because the ratio excludes only BLOCK verdicts. [L63-L65]
- [A][C32-A01] The critic API receives only blank/body/language, yet the single-line exemption depends on whether `x` is an “expression context”; the test explicitly expects body `x` to block in a non-expression context that the API cannot represent. [L87-L93, L190-L192]
- [A][C32-A02] `min_body_lines` is configurable, but the critic rule is hard-coded as three and no config parameter appears in `MiniCritic::check`. [L66, L87-L93]
- [A][C32-A03] `NoFillForBlank` exists, but fewer/more fills, duplicate blank IDs, and substitution ordering are not defined; `fill_blanks` only says it returns OK despite BLOCK. [L69-L75]
- [A][C32-A04] BestOfN has no tie-breaker, definition of the binary/continuous “lines +1” score, candidate ordering, or deterministic generation rule. [L111-L118]
- [A][C32-A05] `AcceptAndComplete` is computed inside a per-blank retry sequencer, but ratio source/update from `existing_fills` and user-opt-in representation are not part of its declared config/context. [L107-L123, L143-L151]
- [A][C32-A06] Bridge template declares one `//` header for every kind, which is not a valid Python comment; per-kind header syntax is not reconciled. [L132-L136]
- [A][C32-A07] `ELAI_SEED` is stored but the described generators are pure sorting/string assembly; seed parsing, invalid values, and behavioral use are unspecified. [L131-L138]
- [C][C32-C01] The source says “minimum 14” tests but enumerates 15 named tests; deterministic check expects exactly 14. The orientation summary repeats 14. [L177-L204, L291-L294]
- [C][C32-C02] The contract file is placed under `crates/contracts/tests`, but the deterministic command runs it as an `elai-orchestrator` test target. [L177-L204, L291-L293]
- [C][C32-C03] General stage logic says only critic PASS converges, but BestOfN says any top candidate with score > 0 converges; compile/line points can make a BLOCK candidate positive. [L111-L118]
- [A][C32-A08] Task 7’s “File” line names only Tauri `lib.rs` even though the task is a ReasoningDrawer renderer; the concrete drawer path is deferred to “locate” in the files list. [L162-L170, L332-L333]

#### Summary additions and corrections

- [D+][C32-D01] Add every declared field/error/event payload, exact lazy patterns and reason strings, single-expression exemption, tree-sitter warning behavior, per-stage mechanics, retry context, per-kind bridge text, user opt-in, and the default-on drawer filter. [L56-L175]
- [D+][C32-D02] Add all 15 enumerated test cases and the unresolved context/config/tie-break/materialization issues. [L177-L204]
- [D~][C32-D03] Correct “14 contract tests” to “15 enumerated despite a stated/expected count of 14,” and flag the crate/path mismatch. [L177-L204, L291-L294]

### C-34 — Phase 4J Empire Integration (DEFERRED)

#### Explicit status, capabilities, workflow, and surfaces

- [E][C34-E01] The entire plan is externally blocked: do not execute until an upstream maintainer grants Empire source access and `[empire.codebase_path]` resolves to a readable local clone; estimated work after unblock is three sessions. [L1-L8]
- [E][C34-E02] It is orthogonal to and non-blocking for every other C-plan. [L7-L8]
- [E][C34-E03] Intended architecture is a named-pipe `FactoryLineTransport`, an `ElaiNeuron` MCP endpoint, a registered `BrainManifest`, and a three-pathology repair pipeline, isolated in a feature-gated `crates/empire` member. [L12-L18]
- [E][C34-E04] Before type definitions, a source-mode paper ingestor must ground the actual Empire transport/manifest interfaces in `workflow/papers/empire-protocol-interface.md`, with reliability tied to source quality. [L24-L36]
- [E][C34-E05] `[empire]` config declares `codebase_path`, `pipe_name`, `connect_timeout_ms`, and `brain_id`; startup validates it only for an enabled Empire build and otherwise warns/skips. [L38-L40]
- [E][C34-E06] Planned crate types are `FactoryLineTransport`, `FactoryLineClient`, `BrainManifest`, `EmpireEventBridge`, `ElaiNeuron`, `EmpireIssue`, and `FactoryCmd`; default builds compile no Empire code. [L42-L45]
- [E][C34-E07] Shared IPC adds an Empire protocol variant to `IpcMessageType` non-destructively, using a non-exhaustive extension or nested variant to preserve forward compatibility and existing sidecar/API IPC. [L47-L50]
- [E][C34-E08] Repair types are `CircularDep`, `HardImport`, `StdlibShadow`, `RepairStrategy`, and ordered `RepairPipeline`; each pathology has an independent pass and per-step SSB emission. [L52-L54]
- [E][C34-E09] Five SSB strings are `empire_connected`, `empire_disconnected`, `empire_capability_invoked`, `empire_repair_started`, and `empire_repair_complete`; each row is declared to carry `brain_id` and `capability_name`, and a schema-version bump is mandatory. [L56-L59]
- [E][C34-E10] `ElaiNeuron` is an HTTP capability-invocation handler that registers the manifest on connect, runs in its own Tokio task, and shuts down on an `empire_disconnected` SSB row. [L61-L63]
- [E][C34-E11] TDD precedes production: mock transport round trip, manifest-schema serialization, per-pathology repair events, MCP response shape, and lossless event forwarding are steps 4J.3–4J.7. [L65-L71]
- [E][C34-E12] Production step 4J.8 wires the neuron into feature-gated startup, binds the event bridge to the SSB bus, and connects repair execution to `empire_repair_started`. [L73-L75]
- [E][C34-E13] CLI adds `--empire`, which activates the Empire path and logs connection status at startup. [L77-L78]
- [E][C34-E14] `EmpireBridgePanel.svelte` shows red/green connection state, active brain ID, last capability, and repair history from the existing SSB stream. [L80-L82]
- [E][C34-E15] Contract tests cover transport, bridge, and repair interface shapes and enforce the wiring-contract stub bans. [L84-L86]

#### Explicit security/failure rules, observability, acceptance, and lifecycle

- [E][C34-E16] Production transport cannot be a no-op, repair errors cannot be silently swallowed and must emit rows, and `ElaiNeuron` must validate the manifest schema before registration. [L108-L112]
- [E][C34-E17] Observable smoke expects connected and capability-invoked rows with non-null identifiers and a green panel displaying the capability. [L113-L117]
- [E][C34-E18] Deferred deterministic checks require the feature declaration, all seven types, all five events, Empire package tests, CLI flag, and panel file. [L119-L139]
- [E][C34-E19] Deferred acceptance requires conditional-only compilation, no placeholder types, preserved IPC tests, independent pathology handling, all events and migration, TDD-before-wiring evidence, contract success, reachable/validated MCP, graceful missing-pipe error, live panel, and cloud smoke. [L141-L158]
- [E][C34-E20] Planned files include six Empire modules, shared IPC/task-runner/CLI/workspace/config edits, panel, orchestrator contract test, and the source-grounding artifact; several paths remain explicitly TBD. [L160-L175]
- [E][C34-E21] B-25 and Phase-4J verification reports remain incomplete until a full audit checks every acceptance item. [L177-L183]
- [E][C34-E22] After unblock and implementation, the lifecycle is full auditor + wiring validator, memory writer, then `MEMORIZING -> COMPLETE -> IDLE`. [L185-L188]

#### Inferred, ambiguous, and conflicting

- [I][C34-I01] The plan treats source grounding as a hard protocol-compatibility/security gate: implementation details are intentionally provisional until actual upstream interfaces are available. [L20-L36, L109-L112]
- [A][C34-A01] Cargo feature activation is compile-time, while `--empire` is described as enabling the feature path at runtime; the build/distribution mechanism that makes the code available to the flag is not specified. [L42-L45, L77-L78]
- [A][C34-A02] A workspace member itself is not normally conditional; “feature-gated workspace member” and “no empire code compiled in default build” need an exact Cargo membership/default-members design. [L42-L45, L143-L145]
- [A][C34-A03] Requiring `capability_name` on **every** event leaves its value for connect/disconnect and generic repair events undefined. [L56-L59, L147-L149]
- [A][C34-A04] Wiring repair execution to `empire_repair_started` while the repair pipeline itself emits per-step/start events risks a feedback cycle unless producer/consumer ownership is distinguished. [L52-L54, L73-L75]
- [A][C34-A05] `ElaiNeuron` authentication, authorization, request schema, transport binding, privacy handling, retry/backoff, and protection of `brain_id`/capability inputs are deferred to source grounding and absent here. [L24-L36, L61-L63]
- [A][C34-A06] Shutdown depends on observing a disconnect row, but the producer/order of that row relative to transport failure and event-bridge shutdown is unspecified. [L56-L63]
- [C][C34-C01] Task 12 names `tests/contracts/empire_contract.rs`, while the planned file list names `.elai_cc/crates/orchestrator/tests/empire_contract.rs`. [L84-L86, L172-L173]
- [A][C34-A07] Named-pipe platform coverage, reconnect policy, timeouts beyond initial connect, repair rollback, and at-least/exactly-once event delivery are not defined. [L12-L18, L38-L75]

#### Summary additions and corrections

- [D+][C34-D01] Add the exact source-grounding reliability gate, config validation behavior, forward-compatible IPC alternatives, event payload requirement, Tokio shutdown trigger, all five TDD cases, stub bans, audit/report lifecycle, and post-audit FSM transitions. [L24-L188]
- [D+][C34-D02] Add the compile-time/runtime flag ambiguity, event payload mismatch, possible repair-event feedback loop, contract path conflict, and absence of endpoint auth/retry/privacy contracts. [L42-L86]
- [D~][C34-D03] Keep the orientation summary’s deferred status, but qualify “feature-gated workspace member” and `--empire` wiring as unresolved build mechanics rather than completed design. [L42-L45, L77-L78]

### C-35 — Tool Attention Dynamic Gating + Lazy MCP Schema Loading

#### Explicit evidence, capabilities, configuration, and workflow

- [E][C35-E01] The objective is per-FSM-state tool visibility plus on-demand full schemas to cut repeated MCP catalog prompt cost; Phase-4K is required, C-35 precedes overlapping C-42, and C-36/37/38/39 are parallel-safe. [L1-L12]
- [E][C35-E02] The cited paper’s roughly 95% tool-token reduction is treated as measured, but downstream quality claims are explicitly projections requiring ELAI’s own SSB accuracy comparison. [L14-L22]
- [E][C35-E03] Dynamic gating intersects registered tools with a state allowlist; tools outside the active visible set are represented by one aggregate “N tools available on request” marker rather than full schemas. [L33-L37]
- [E][C35-E04] Lazy loading puts only tool name + one-line summary in the prompt and resolves full parameter/return JSON only after a call references that tool, targeting a catalog reduction from roughly 6–10K to 300–500 tokens. [L38-L41]
- [E][C35-E05] Source grounding of arXiv:2604.21816 into `tool-attention-interface.md` is required before code changes. [L55-L60]
- [E][C35-E06] `[tool_gating]` declares `enabled` (true after acceptance), `gate_manifest_path="plugins/tool-gate.toml"`, `lazy_schema_default=true`, and `max_eager_schemas=4`, validated at startup. [L62-L66]
- [E][C35-E07] Each `[gate.STATE]` manifest section separates `eager` (full schema), `gated` (name/summary), and `denied` (omitted and structured-error on attempted call). [L68-L82]
- [E][C35-E08] `ToolGate::load_gate` and `filter_for_state(fsm_state,full_catalog)` produce `GatedCatalog { eager_schemas: Vec<FullSchema>, gated_summaries: Vec<SummaryRow>, denied_names: HashSet<String> }`. [L84-L87]
- [E][C35-E09] `server.rs` replaces full-catalog rendering with state-gated rendering and resolves a referenced gated tool through `resolve_schema_on_demand(tool_name)`. [L89-L92]
- [E][C35-E10] A lazily resolved schema is injected only for the current call response cycle unless active-use LRU promotion keeps it eager; the LRU capacity is `max_eager_schemas`. [L92-L94]
- [E][C35-E11] `LazySchemaCache` uses LRU eviction and a per-tool first-load lock to prevent a concurrent thundering herd, and records hit/miss/resolve-latency metrics. [L96-L98]
- [E][C35-E12] SSB adds `tool_gated`, `tool_eager`, `tool_lazy_resolved`, and `tool_denied`, with `tool_name`, `fsm_state`, and `prompt_tokens_saved`; a schema bump is required by the plan. [L100-L102]
- [E][C35-E13] TDD proves a three-way split over 10 tools, LRU behavior over state transitions, byte-for-byte resolved-schema equality, structured denied errors with SSB, and exact positive serialization savings. [L104-L115]
- [E][C35-E14] Production binds the gate/cache into `DefaultMcpBridge` prompt assembly and wires emissions. [L116-L118, L205-L214]
- [E][C35-E15] `--tool-gate-off` is a runtime debugging escape hatch that logs a 5–10x prompt-cost warning. [L120-L122]
- [E][C35-E16] `ToolGateInspector` displays active state/gate, eager/gated sets, and a rolling 60-second token-savings histogram through the existing SSB stream. [L124-L127]
- [E][C35-E17] Missing-manifest behavior is fail-safe for availability and least-schema exposure: emit a `tool_gated` row with `reason="manifest_absent"`, then use eager empty / gated all / denied empty, never the full catalog. [L152-L155]
- [E][C35-E18] Cache activity may not be silent, and denied calls return structured errors rather than panicking. [L156-L158]

#### Explicit shipped status, implementation identifiers, deferred gates, and evidence

- [E][C35-E19] The acceptance section declares the library tier **SHIPPED-ON-V2**, including gate/cache/manifest/config, four events, five unit + three contract tests, mounted inspector, and CLI parse/warn. [L199-L205]
- [E][C35-E20] Completion-drive identifiers include free-function aliases `render_gated_catalog`/`resolve_schema_on_demand`, `mcp::server` re-export of `mcp_bridge`, `Summarize` + `McpToolSpec` implementation, and `DefaultMcpBridge::assemble_prompt`. [L205-L208]
- [E][C35-E21] `DefaultMcpBridge` owns `ToolGate` and `LazySchemaCache`; production has four emit sites through `ToolGateSsbEmitter`, with `OrchestratorToolGateSsbEmitter` as adapter. [L208-L214]
- [E][C35-E22] Dashboard completion includes inspector mounting in SettingsWindow, `get_tool_gate_snapshot` and `list_tool_gate_rows` registered with Tauri, panel `invoke()` calls, and two DTO round-trip tests. [L210-L214]
- [E][C35-E23] Runtime bypass implementation includes an `ELAI_TOOL_GATE_OFF` environment reader at bridge construction and warning on a double-set path. [L212-L214, L237-L238]
- [E][C35-E24] Two additional wiring tests cover assemble emission and a SafeDefault emission-skip case. [L213-L214]
- [E][C35-E25] The original ship bumped schema at version 88; current schema is declared locked at 103, so this completion session does not bump again. [L201-L204, L228-L229]
- [E][C35-E26] Manifest coverage, non-placeholder code, denied handling, byte-equal resolution, all event identifiers, TDD-before-wiring, contract tests, bypass warning, and live inspector are checked complete. [L220-L240]
- [E][C35-E27] Deferred A5 is a 1,000-iteration LRU stress test. [L216, L224-L225]
- [E][C35-E28] Deferred A10 is a live 30-tool run establishing 5–10K prompt-token savings. [L216-L218, L233-L234]
- [E][C35-E29] Deferred A15 is C-54/RunPod gate-on versus gate-off SWE-bench measurement requiring solve-rate delta within ±2 percentage points. [L217-L218, L241-L244]
- [E][C35-E30] Observable design expects a `tool_gated` row on every prompt assembly, first-reference lazy resolution, and a non-empty inspector histogram. [L160-L167]
- [E][C35-E31] C-35 must be live before C-54 so benchmark results use the production prompt shape; C-36 is orthogonal but benefits from the lower baseline prompt cost. [L268-L274]

#### Inferred, ambiguous, and conflicting

- [I][C35-I01] The safe default favors tool availability while minimizing eager token exposure: every registered tool remains callable on demand, and none is denied solely because the manifest is absent. [L152-L155]
- [I][C35-I02] Tool visibility and call authorization are separate layers because C-35 explicitly precedes C-42’s zero-trust authorization work. [L9-L12, L268-L274]
- [A][C35-A01] `max_eager_schemas` tools are said to be full-schema “regardless of gate,” which can conflict with a state’s `denied` list; precedence is not defined. [L62-L66, L68-L82]
- [A][C35-A02] A model must emit arguments when first naming a gated tool despite not seeing its full schema; “inject for that call’s response cycle” does not specify whether execution is paused/retried or how the initial arguments are validated. [L38-L41, L89-L94]
- [A][C35-A03] Hit/miss “SSB rows are mandatory,” but the four declared action types do not include `cache_hit`/`cache_miss`; payload mapping to `tool_lazy_resolved` or another row is not stated. [L96-L102, L156-L157]
- [A][C35-A04] Aggregate missing-manifest emission requires `reason`, while the declared event payload lists only tool name/state/tokens saved; the tool name and reason schema for an aggregate event are unresolved. [L100-L102, L152-L155]
- [C][C35-C01] Stub-ban behavior mandates `tool_gated` emission for SafeDefault, while the shipped-test list names “SafeDefault emission-skip”; without test semantics these appear inconsistent. [L152-L155, L213-L214]
- [C][C35-C02] Task/config/file lists call the schema interface a `SummaryRow`/“Summary trait,” while shipped identifiers say `Summarize trait + McpToolSpec impl`. [L84-L87, L205-L208, L256]
- [C][C35-C03] The file list still labels `ToolGateInspector.svelte` PLANNED, but shipped acceptance says it is mounted and invoking Tauri commands. [L239-L240, L246-L254]
- [C][C35-C04] The original declares parallel safety only with C-36 through C-39, while the orientation summary extends that claim through C-40 without source support. [L9-L12]
- [A][C35-A05] LRU active-use retention, ordering under concurrent state changes, token-count method, and cache invalidation when a registered schema changes are not specified. [L89-L115]
- [A][C35-A06] The visible CLI flag and implemented `ELAI_TOOL_GATE_OFF` environment reader are both reported, but their precedence and mapping at bridge construction are not defined. [L120-L122, L212-L214, L237-L238]

#### Summary additions and corrections

- [D+][C35-D01] Add exact config defaults, three-state manifest semantics, all `GatedCatalog` fields, call-cycle/LRU behavior, per-tool concurrency lock, safe-default algorithm, all TDD cases, and implementation identifiers M1–M5/N1–N7. [L62-L158, L199-L214]
- [D+][C35-D02] Add the source’s current shipped status, version-88/locked-103 migration history, completed dashboard/Tauri adapter surfaces, and the three separately deferred acceptance items. [L199-L244]
- [D~][C35-D03] The orientation summary presents 5–10K savings and ±2pp accuracy as smoke requirements without status; the original marks both unmeasured/deferred (A10/A15), while most library/UI work is shipped. [L216-L244]
- [D~][C35-D04] Replace the stale `ToolGateInspector` “planned” file annotation with the acceptance section’s mounted/working status, while retaining the internal file-list conflict. [L239-L254]
- [D~][C35-D05] Restrict the parallel-safe set to the source’s C-36/C-37/C-38/C-39 list; C-40 is not named. [L9-L12]

### C-36 — MUCOCO Mutation-Operator Catalog

#### Explicit capabilities, flags, applicability, events, and workflow

- [E][C36-E01] This C-29 extension is a compile-time-gated, runtime-opt-in catalog plus measured pilot; C-29 must be shipped, C-35/37/38/39/40 are parallel-safe, and other edits to `MetamorphicRelation` must sequence after C-36. [L1-L10]
- [E][C36-E02] The source cites a 15% inconsistency-discovery claim and treats the transferable artifact as operator definitions because an open-source release was not confirmed. [L12-L19]
- [E][C36-E03] The plan explicitly interprets less than 85% pass rate under the catalog as a reproducibility problem. [L29-L36]
- [E][C36-E04] Paper ingestion must capture verbatim descriptions, language applicability, and false-positive notes before enum extension. [L50-L55]
- [E][C36-E05] Nine `MetamorphicRelation` variants behind `#[cfg(feature="mucoco_operators")]` are `VariableRename`, `WhitespaceNormalize`, `CommentStrip`, `LoopFormRewrite`, `BranchFormRewrite`, `RedundantCastInject`, `TypeAnnotationToggle`, `DeadCodeInject`, and `ImportOrderRewrite`. [L57-L68]
- [E][C36-E06] Every operator module exposes `apply(ast: &TreeSitterNode, rng: &mut Rng) -> Result<MutatedSource, SkipReason>` and `supports_language(Language) -> bool`; unsupported shapes return `NotApplicable` rather than panicking. [L70-L73]
- [E][C36-E07] `[testing.metamorphic.advanced_operators]` defaults to `enabled=false`, chain cap 3, empty exclusion list, and `require_explicit_in_test=true`. [L75-L82]
- [E][C36-E08] Rust is declared to support all nine; Python supports eight with loop-rewrite iterator caveats; TypeScript supports eight with type-annotation toggling in the inverse direction; all other named languages use only paper-documented subsets and explicit unsupported pairs. [L84-L90]
- [E][C36-E09] SSB adds `mucoco_mutation_applied`, `mucoco_inconsistency_detected`, and `mucoco_skip_reason`, each with operator name, target test ID, and language, plus a schema bump. [L92-L94]
- [E][C36-E10] TDD requires every apply result to be syntactically valid or skipped, chain length four to stop at three, scope-safe variable renaming, runtime-semantic-safe TypeScript toggling, and exclusion-list enforcement. [L96-L105]
- [E][C36-E11] Production enables operators only when the Cargo feature, runtime `enabled=true`, and per-test marker are all present. [L107-L110]
- [E][C36-E12] Per-test markers are `#[mucoco]` for applicable test functions or `@mucoco` docstring tags depending on language. [L141-L144]
- [E][C36-E13] The pilot runs over the existing 3,500+ tests and publishes per-operator/per-language detection rates plus a human-audited 50-case false-positive rate. [L111-L117]
- [E][C36-E14] Pilot target is at least one operator at/above 15%; a lower result files an implementation bug but explicitly does not block C-36 closure. [L117-L119]
- [E][C36-E15] Inconsistency emission must precede test failure, and the 50-case audit must exist at `workflow/sessions/{id}/mucoco-pilot.md` before closure. [L145-L150]
- [E][C36-E16] There is no new frontend; events reuse C-29’s testing-trust dashboard. [L151-L152]
- [E][C36-E17] Production must remain default-off until C-54 locks its baseline, even after C-36 lands. [L233-L239]

#### Explicit acceptance, determinism, files, and status

- [E][C36-E18] Deterministic gates inspect all nine variants/modules/events/config, run feature-enabled tests/contracts, and require the current-session pilot artifact. [L162-L185]
- [E][C36-E19] Acceptance requires grounded catalog, nine non-placeholder operators, complete skip taxonomy, paper-matching matrix, validating default-off config, working per-test opt-in, non-null event fields/migration, TDD-before-wiring, pilot/audit, documentation, and contracts. [L187-L203]
- [E][C36-E20] The implementation surface is eleven new operator/applicability files, two tests, one paper artifact, relation/module/Cargo/config/SSB/docs edits. [L205-L226]
- [E][C36-E21] All acceptance boxes remain unchecked in this source; status is planned, not claimed shipped. [L187-L203]

#### Inferred, ambiguous, and conflicting

- [I][C36-I01] Default activation requires three independent gates, making accidental blanket mutation unlikely. [L57-L59, L75-L82, L107-L110]
- [I][C36-I02] Syntax validation is automated, while semantic preservation is partly empirical/human-audited through the pilot. [L96-L105, L111-L117]
- [A][C36-A01] TypeScript is said to support eight operators but the excluded ninth is not named; Python’s “supports 8” is also phrased as an edge case rather than an explicit unsupported mapping. [L84-L90]
- [A][C36-A02] RNG seeding, mutation ordering, repeat-run determinism, chain composition semantics, and shrinking/reproduction of a detected inconsistency are not specified. [L70-L82, L96-L119]
- [A][C36-A03] `SkipReason` is promised to cover every documented unsupported AST shape, but only `NotApplicable` is named. [L70-L73, L129-L135, L189-L192]
- [C][C36-C01] Task 9 says falling below 15% does not block closure, while the acceptance checkbox says the pilot “matches or exceeds” 15% (then says file a bug); the authoritative done-gate interpretation is unclear. [L117-L119, L199-L200]
- [C][C36-C02] `TypeAnnotationToggle` is labeled dynamic-language-only, yet the applicability section says Rust supports all nine operators. [L65-L68, L84-L90]
- [C][C36-C03] The original’s parallel-safe list stops at C-40 and warns against any relation-enum patch; the orientation summary expands safety through C-53. [L7-L10]
- [A][C36-A04] The source asserts semantic-preserving behavior yet its core TDD gate guarantees only syntactic validity for most operators; oracle/equivalence rules beyond two singled-out cases are absent. [L29-L36, L96-L105]
- [A][C36-A05] Event ordering under parallel tests, duplicate mutation identity, isolation of source mutations, rollback, and sensitive source content in pilot artifacts are not specified. [L92-L119, L145-L160]

#### Summary additions and corrections

- [D+][C36-D01] Add all nine exact variant/module names, function signatures, three activation gates, four config defaults, language claims/caveats, five TDD behaviors, exact marker forms, 50-sample audit, and default-off-until-C-54 rule. [L57-L150, L233-L239]
- [D+][C36-D02] Add planned/unshipped status and the explicit conflict over whether missing the 15% target blocks closure. [L117-L119, L187-L203]
- [D~][C36-D03] Replace the orientation summary’s blanket “pilot acceptance requires 15%” with the source’s conflicting target/non-blocking-bug language. [L117-L119, L199-L200]
- [D~][C36-D04] Correct the orientation summary’s C-37..C-53 parallel-safe range to the original’s C-37..C-40 plus the explicit enum-edit exclusion. [L7-L10]

### C-44 — Composer 2 Runtime Backend Tier

#### Explicit status, configuration, routing, failure, and observability

- [E][C44-E01] The intended third tier sits between local Qwen2.5-Coder and Sonnet 4.6, adopting B-37’s backend abstraction and a cost position between free/local and premium/cloud. [L1-L6]
- [E][C44-E02] B-37 and C-39 are shipped prerequisites; API access is a hard gate, sequencing is after C-39 and C-42, and routing integration separately depends on C-41. [L8-L13, L68-L73]
- [E][C44-E03] The cited report claims SWE-bench Multilingual 73.7, Terminal-Bench 61.7, CursorBench 61.3, $0.50/M input, $2.50/M output, and Kimi K2.5 base. [L15-L18]
- [E][C44-E04] Public API availability is unresolved in the source; if verification finds none, closure is an infeasibility/deferral note rather than code integration. [L29-L32, L43-L45]
- [E][C44-E05] Operational source grounding records prompt format, system-prompt conventions, and token-count quirks in `composer2-interface.md`. [L47-L49]
- [E][C44-E06] `[backends.composer2]` defaults disabled and declares key env `COMPOSER2_API_KEY`, provisional model ID `cursor-composer-2`, provisional 200,000 input/8,192 output limits, default executor/auditor roles, and the two token prices. [L51-L62]
- [E][C44-E07] `Composer2Backend` implements async `Backend::invoke(prompt,params) -> Response`; API failures become structured `BackendError`, never panic. [L64-L66]
- [E][C44-E08] Default skill routing prefers Sonnet for `heavy_plan`, Composer 2 for `focused_edit`, local Qwen 7B for `simple_query`, and C-54-controlled routing for `benchmark_run`. [L68-L73]
- [E][C44-E09] Events are `backend_composer2_invoked`, `backend_composer2_error`, and `backend_composer2_cost_accrued`, the cost row carrying `usd_cents`. [L75-L76]
- [E][C44-E10] TDD covers mocked HTTP response parsing, missing-key structured error with no silent fallback, usage/token-count agreement, cost calculation, and failover on HTTP 503 to a configured next tier. [L78-L85]
- [E][C44-E11] Every successful invocation emits cost, while failover must first emit a Composer-2 error; missing API key is explicitly not a silent-fallback case. [L110-L113]
- [E][C44-E12] `BackendCostPanel` reports rolling 24-hour cost by backend and session. [L87-L89]
- [E][C44-E13] Contract tests verify `Backend` conformance. [L91-L92]
- [E][C44-E14] Operational smoke enables Composer 2 and expects invoked + cost rows per routed LLM call. [L115-L118]
- [E][C44-E15] Conditional acceptance requires an API-status record; operational path adds grounded interface, config/backend/errors/cost/failover/events/tests/panel/contracts, while infeasible path writes `composer2-deferral.md` with API status and revisit trigger. [L130-L144]
- [E][C44-E16] The source does not claim current operational or infeasible completion; all acceptance boxes remain unchecked. [L130-L144]
- [E][C44-E17] Either closure outcome clears C-54 to compare local-only, hybrid Sonnet/Composer/local, and full-cloud configurations. [L162-L166]
- [E][C44-E18] Context characterizes local Qwen as deterministic/free at 30–50 tok/s, Sonnet as expensive with 1M context, and Composer as roughly 6x cheaper on input and 3.6x on output than Sonnet. [L20-L27]

#### Inferred, ambiguous, and conflicting

- [I][C44-I01] Error policy distinguishes configuration failure from transient service failure: absent credentials stop with a structured error, while 503/rate-limit may use configured failover after telemetry. [L78-L85, L110-L113, L136-L139]
- [A][C44-A01] `model_id` and input limit are explicitly pending Task-1 confirmation, so they are placeholders rather than verified defaults. [L51-L58]
- [A][C44-A02] Config uses `role_routing`, while Task 5 specifies per-**skill** routing; override precedence among role, skill, explicit user choice, and failover tier is absent. [L51-L62, L68-L73, L107-L109]
- [A][C44-A03] `usd_cents` type, fractional-cent rounding, input/output accounting, cached-token pricing, and retry/double-charge behavior are not specified. [L51-L62, L75-L85]
- [A][C44-A04] The plan calls Composer 2 a “non-zero default cost vector” even though its backend config defaults disabled; this likely means enabled local-first sessions, but wording is unclear. [L53-L55, L87-L89]
- [C][C44-C01] Task 1 permits immediate deferral when no API exists, but the acceptance list mixes that conditional branch with unconditional-looking backend/test/UI checkboxes; minimum done gates for deferral are not cleanly separated. [L43-L45, L130-L144]
- [A][C44-A05] API authentication/header format, endpoint, streaming/tool-call protocol, retries/backoff, rate-limit parsing, privacy/data-retention expectations, model-version pinning, and schema migration for its three new events are not defined. [L34-L40, L64-L92]
- [A][C44-A06] C-42 is a required sequencing dependency in prose but not listed in the initial prerequisites; the orientation summary promotes it to a dependency. [L8-L13]

#### Summary additions and corrections

- [D+][C44-D01] Add the conditional infeasibility closure, provisional config values, exact invoke/error contract, all five TDD cases, missing-key versus 503 behavior, 24-hour panel scope, and three benchmark arms. [L29-L166]
- [D+][C44-D02] Add role-versus-skill routing ambiguity, undefined cost rounding/version pin, absence of event migration rules, and currently unchecked status. [L51-L144]
- [D~][C44-D03] Qualify the summary’s “depends on C-42” as a prose sequencing requirement absent from the formal prerequisite list, and do not imply an integration exists before Task-1 API verification. [L8-L13, L29-L45]

### C-49 — Iterative Distillation of Experience for Prompt-Engineer

#### Explicit capabilities, data lifecycle, events, and gates

- [E][C49-E01] The plan turns successful task traces into reusable prompt-policy fragments; prompt-engineer is required, C-41 is recommended rather than mandatory, and C-35..48/C-50..53 are parallel-safe. [L1-L10]
- [E][C49-E02] Per-project/per-skill storage is `.elai/experience/{skill_name}.md` with `Distilled prompt fragments`, `Common parameter shapes`, and `Avoid patterns` sections. [L19-L35, L48-L50]
- [E][C49-E03] Successful runs append deduplicated learned material; failures append to avoid patterns. [L33-L35]
- [E][C49-E04] `Distiller` runs after successful task completion with skill name, full execution trace, and SSB log, appending to the corpus. [L52-L55]
- [E][C49-E05] Deduplication collapses exact five-gram shingle matches and is required to be deterministic rather than random sampling. [L54-L55, L93-L95]
- [E][C49-E06] Distillation also consults `mistakes.md`; avoid patterns flow into prompt scaffolds, and new mistakes update the avoid section. [L57-L59]
- [E][C49-E07] Prompt engineering consults the selected skill’s corpus after `SkillRegistry` lookup and injects roughly 500 tokens, preferring newest fragments. [L61-L64]
- [E][C49-E08] Over-budget behavior evicts oldest material, not newest. [L69-L74, L93-L96]
- [E][C49-E09] Greenfield/missing corpus is a transparent no-op and must not panic. [L69-L74, L93-L96]
- [E][C49-E10] SSB adds `experience_distilled`, `experience_consulted`, and `experience_token_budget_exceeded`. [L66-L67]
- [E][C49-E11] TDD covers Markdown round-trip, shingle dedup, mistakes-to-scaffold flow, oldest-first budget enforcement, and empty-corpus safety. [L69-L75]
- [E][C49-E12] Contract tests live at `tests/contracts/experience_corpus_contract.rs`. [L76]
- [E][C49-E13] Reachability is post-successful-task distillation and every prompt-engineer invocation. [L78-L92]
- [E][C49-E14] Acceptance requires grounding, Markdown schema, append/dedup, bidirectional avoid integration, post-SkillRegistry consultation, budget, all events, TDD/greenfield, and contracts. [L107-L117]
- [E][C49-E15] A non-empty distilled section after ten same-codebase sessions is explicitly a smoke test, **not** a formal acceptance gate. [L118-L119]
- [E][C49-E16] Implementation adds experience/distiller modules and two tests, modifies prompt engineer/post-task hook/SSB schema, and adds the paper artifact. [L121-L130]
- [E][C49-E17] All acceptance boxes remain unchecked; no shipped status is claimed. [L107-L119]
- [E][C49-E18] Source grounding runs paper-ingestor against arXiv:2605.14443 and produces `workflow/papers/iterative-distillation.md`. [L37-L46, L109, L127]
- [E][C49-E19] The stated expected lifecycle is that the first project session pays full prompt-construction cost and later sessions pay less as the corpus accumulates. [L132-L136]

#### Inferred, ambiguous, and conflicting

- [I][C49-I01] Project-local storage is intended to amortize repeated prompt construction without creating a global cross-project memory. [L19-L35, L121-L135]
- [I][C49-I02] Recency selection and oldest-first eviction define a simple deterministic retention policy once entries have a stable order. [L61-L74, L93-L96]
- [C][C49-C01] Failures are said to append avoid patterns, but the only distiller/post-task trigger is successful completion; no failed-task hook is defined. [L33-L35, L52-L59, L90-L92, L128-L130]
- [A][C49-A01] “5-gram” does not define whether grams are words, tokens, characters, or normalized Markdown, nor the comparison scope across sections/sessions. [L52-L55]
- [A][C49-A02] The approximately-500-token cap has no tokenizer, exact boundary, config key, truncation unit, or treatment of an individually oversized fragment. [L61-L64, L69-L74]
- [A][C49-A03] Extraction/scoring of “consistently appeared” fragments and common parameter shapes from full traces/SSB is not specified. [L26-L35, L52-L55]
- [A][C49-A04] Concurrent/atomic corpus writes, crash recovery, corrupt Markdown handling, file locking, and idempotence across repeated post-task hooks are unspecified. [L48-L59]
- [A][C49-A05] No secret/PII/source-code redaction, provenance, trust boundary, prompt-injection filtering, retention limit, or user deletion control is specified before persisting full-trace-derived content. [L19-L35, L52-L64]
- [A][C49-A06] SSB payload schemas, schema-version bump/migration, emission timing on transparent skip, and failure/retry behavior are absent. [L66-L67, L78-L96]
- [A][C49-A07] `{skill_name}` filename sanitization and fallback binning when C-41 is absent are not defined even though C-41 is optional. [L8-L9, L26-L35, L48-L64]

#### Summary additions and corrections

- [D+][C49-D01] Add the exact three-section corpus schema, success inputs/trigger, deterministic exact-shingle rule, bidirectional mistake behavior, newest-first selection/oldest-first eviction, five TDD cases, and transparent greenfield skip. [L19-L96]
- [D+][C49-D02] Add the failed-run trigger conflict and all unspecified persistence/privacy/concurrency/tokenization/event-lifecycle rules. [L33-L96]
- [D~][C49-D03] Correct the orientation summary’s “ten sessions must produce” wording: the original labels it smoke evidence, not a formal acceptance gate. [L118-L119]
- [D+][C49-D04] Restore the original’s broader parallel-safe range C-35..C-48; the orientation summary mentions only C-46..C-48 plus C-50..C-53. [L8-L10]

## Cross-source interactions and system-level behavior

### Directly explicit interactions

- [E][X-E01] C-9 consumes C-1 gate-decision emission and C-5 cloud/autonomous mode; its calculator smoke is impossible until both exist. [C-9 L11-L13, L177-L181]
- [E][X-E02] C-16 consumes C-2’s already-defined Cosmos SSB variants and C-8’s stage stub; it preserves C-17..20 parallelism. [C-16 L7-L9]
- [E][X-E03] C-24 consumes C-17’s stable incremental watcher/lock path and pre-existing B-29/B-33 symbol, graph-store, Mermaid, VizData, and event symbols. [C-24 L7-L9, L157-L181]
- [E][X-E04] C-28 is the sixth deterministic check after C-27’s wiring/determinism layer, and C-36 later extends the C-29 metamorphic layer that follows it. [C-28 L7-L9, L128-L138; C-36 L7-L10, L107-L110]
- [E][X-E05] C-32 consumes C-31 file plans and C-26 grammar/free-text decoding/retry policy; its generated files later remain subject to deterministic/mutation/metamorphic auditing. [C-32 L9-L13, L223-L231]
- [E][X-E06] C-34 is deliberately disconnected from the rest of C delivery until external protocol access exists. [C-34 L1-L8]
- [E][X-E07] C-35 must land before overlapping C-42 and before C-54 benchmarking, while C-36 is orthogonal and merely benefits from lower prompt cost. [C-35 L9-L12, L268-L274]
- [E][X-E08] C-44 uses the B-37 backend abstraction, C-39 dispatch, post-C-42 authorization coverage, and optional C-41 skill registry; it feeds C-54’s third benchmark arm. [C-44 L8-L13, L68-L73, L162-L166]
- [E][X-E09] C-49’s experience lookup occurs after C-41 SkillRegistry when available and modifies the prompt-engineer scaffold on every later invocation. [C-49 L8-L9, L61-L64, L85-L92]

### Conservative composition inferences

- [I][X-I01] The intended generation/trust chain is FilePlan -> C-32 skeleton/fill/critic/retry/single flush -> existing five deterministic checks -> C-28 mutation -> C-29/C-36 metamorphic checks. Only the adjacent links are explicit; a single end-to-end orchestrator order across all plans is not specified here. [C-32 L143-L152; C-28 L128-L138; C-36 L107-L110]
- [I][X-I02] C-16 and C-24 both depend on correct index locking/incremental state, but they operate on different graph/artifact paths; their event streams converge only at shared SSB/dashboard infrastructure. [C-16 L70-L90, L121-L136; C-24 L80-L94, L157-L188]
- [I][X-I03] C-35 reduces the schema context presented before calls, while C-49 later adds bounded experience context; total prompt-budget arbitration between those savings/additions is not defined. [C-35 L33-L41; C-49 L61-L64]
- [I][X-I04] C-9’s per-role backend stanza and C-44’s provider/skill routing are complementary configuration layers, but their precedence and even shared `BackendStanza` version are not established by these sources. [C-9 L138-L152; C-44 L1-L6, L51-L73]
- [I][X-I05] SSB is the common evidence plane for refresh, graph updates, mutation, skeleton/retry, Empire, tool gating, Composer cost, and experience, while each frontend chooses polling, subscription, or both. [C-16 L62-L90; C-24 L85-L94; C-28 L119-L150; C-32 L76-L175; C-34 L56-L82; C-35 L100-L127; C-36 L92-L94; C-44 L75-L89; C-49 L66-L67]

### Cross-cutting ambiguities and conflicts

- [A][X-A01] Schema migration is not globally coherent in this chunk: C-28 claims 46→47 (while summary says shared 47), C-32 names five new events without a bump, C-34/C-35/C-36 require bumps, C-35 reports historical 88/current locked 103, and C-44/C-49 name new events without migration instructions. [C-28 L28, L119-L126; C-32 L171-L175; C-34 L56-L59; C-35 L100-L102, L201-L204; C-36 L92-L94; C-44 L75-L76; C-49 L66-L67]
- [A][X-A02] No cross-plan rule resolves backend routing among C-9 role stanzas, C-44 skill/role settings, explicit user selection, missing credentials, 503 failover, or model-fit escalation. [C-9 L121-L152; C-44 L51-L85, L107-L113]
- [A][X-A03] Concurrency guarantees are uneven: C-16 specifies index locks/cancel and atomic-write stress; C-35 specifies per-tool locks; C-28 and C-49 persist mutable SQLite/Markdown state without full concurrent-update semantics. [C-16 L70-L77, L221-L226; C-28 L98-L104; C-35 L96-L98; C-49 L48-L59]
- [A][X-A04] Privacy/security rules are local rather than systemic: C-35 denies hidden tools and C-44 uses an env key, but C-34’s HTTP endpoint and C-49’s trace-derived durable memory lack explicit auth/redaction/retention controls. [C-34 L61-L63; C-35 L68-L82; C-44 L51-L66; C-49 L19-L64]
- [A][X-A05] Statuses are mixed and must not be flattened: C-9/16/24/28/32/C-36/C-49 are unchecked plans, C-34 is externally deferred, C-35 claims shipped library/UI work with three deferred measurements, and C-44 is conditional on API feasibility. [C-9 L276-L304; C-16 L179-L210; C-24 L255-L291; C-28 L255-L281; C-32 L296-L317; C-34 L1-L8; C-35 L199-L244; C-36 L187-L203; C-44 L130-L144; C-49 L107-L119]

## Taxonomy coverage audit

Every requested dimension was checked in every original. “None” below means the original defines no dedicated contract in that dimension; it is not an inference that the implementation is safe.

| Source | Capability/action/surface | Workflow/state/lifecycle | Schema/config/events | Security/privacy/compliance | Failure/retry/concurrency/determinism | Evidence/tests/done/non-goal/status |
|---|---|---|---|---|---|---|
| C-9 | C9-E02..E25 | gate/split/model state, C9-E05..E25 | verdict/check/policy/split/backend/metrics, C9-E02..E23 | no dedicated auth/privacy/compliance contract | fail-closed split/model rules and unresolved aggregation, C9-E27..E29, C9-A01..A05 | C9-E30..E36; defers parallel merge |
| C-16 | C16-E04..E19 | tier, periodic/manual refresh, cancel, decay | stage/response/events, C16-E04..E18 | no dedicated auth/privacy/compliance contract | index lock, atomic writes, idempotence, fail-soft stage, C16-E20..E24 | C16-E25..E28; six non-goals |
| C-24 | C24-E02..E13 | threshold branches and UI/IPC flows | four tool schemas, dispatcher schema, event/style state | no dedicated auth/privacy/compliance contract | deterministic BTreeMap export; fail-soft empty fixture; C24-E14..E19/E23 | C24-E20..E24 |
| C-28 | C28-E03..E22 | sixth audit check and FSM block | result/cache/config/event/gauge schemas | no privacy/compliance contract; production no-panic/no-mock bans only | subprocess errors/timeouts, TTL/upsert/replay; concurrency underspecified | C28-E23..E31 |
| C-32 | C32-E02..E29 | in-memory fill/retry/partial-accept/single flush | skeleton/critic/retry/bridge/event types and defaults | user opt-in for partial acceptance; no privacy/compliance contract | structural retry, deterministic bridge, single write; C32-E30..E35 | C32-E30..E37; explicit stub bans |
| C-34 | C34-E03..E15 | externally gated connect/repair/shutdown/audit lifecycle | Empire config/types/IPC/events | manifest validation explicit; endpoint auth/privacy otherwise absent | graceful missing pipe and no silent repair errors; reconnect/delivery unclear | C34-E16..E22; wholly deferred |
| C-35 | C35-E03..E18, E20..E24 | per-state visibility, lazy-call/LRU lifecycle | gate config/manifest/catalog/cache/events | denied tools and safe default; C-42 owns later authz; no privacy contract | per-tool lock, LRU, structured denial, safe default | C35-E19..E31; A5/A10/A15 deferred |
| C-36 | C36-E05..E17 | compile/runtime/per-test three-gate lifecycle | variants/config/applicability/events | per-test opt-in; no source/privacy/compliance policy | skip-not-panic, chain cap, RNG determinism/concurrency unspecified | C36-E18..E21; pilot/audit and default-off constraint |
| C-44 | C44-E04..E14 | API feasibility branch, routing/failover/cost lifecycle | backend config/events | env-held API key; no endpoint privacy/retention contract | structured errors, 503/rate failover, cost/determinism gaps | C44-E15..E18; operational-or-deferral status |
| C-49 | C49-E02..E13 | post-success append and per-prompt consult lifecycle | Markdown corpus/events/budget policy | no redaction/provenance/retention/deletion contract | deterministic shingling/eviction; atomic/concurrent recovery absent | C49-E14..E19; ten-session item is non-gating smoke |

## Summary-delta audit

There are **38 uniquely identified summary-delta records** across ten per-source delta sections: 5 C-9, 4 C-16, 3 C-24, 4 C-28, 3 C-32, 3 C-34, 5 C-35, 4 C-36, 3 C-44, and 4 C-49. They cover both additions (`[D+]`) and corrections/qualifications (`[D~]`). Material corrections include the C-9 field/key/identifier conflicts, C-16’s unsupported B-35 closure wording, C-28’s tool-install and schema chronology, C-32’s 14-versus-15 tests, C-35’s shipped/deferred split and parallel range, C-36’s target/parallel range, C-44’s conditional API branch, and C-49’s non-gating ten-session smoke.

## Zero-omission / zero-duplicate audit and counts

| Ledger scope | Source lines | `[E]` | `[I]` | `[A]` | `[C]` | Atomic items | Summary deltas |
|---|---:|---:|---:|---:|---:|---:|---:|
| C-9 | 335 | 36 | 2 | 5 | 3 | 46 | 5 |
| C-16 | 262 | 28 | 2 | 5 | 1 | 36 | 4 |
| C-24 | 316 | 24 | 2 | 6 | 0 | 32 | 3 |
| C-28 | 302 | 31 | 2 | 6 | 2 | 41 | 4 |
| C-32 | 335 | 37 | 2 | 8 | 3 | 50 | 3 |
| C-34 | 188 | 22 | 1 | 7 | 1 | 31 | 3 |
| C-35 | 274 | 31 | 2 | 6 | 4 | 43 | 5 |
| C-36 | 239 | 21 | 2 | 5 | 3 | 31 | 4 |
| C-44 | 166 | 18 | 1 | 6 | 1 | 26 | 3 |
| C-49 | 136 | 19 | 2 | 7 | 1 | 29 | 4 |
| **Ten-source subtotal** | **2,553** | **267** | **18** | **61** | **19** | **365** | **38** |
| Cross-source interactions | — | 9 | 5 | 5 | 0 | 19 | — |
| **Discovery total** | **2,553** | **276** | **23** | **66** | **19** | **384** | **38** |

Audit results:

- Manifest files expected/processed/unique: **10 / 10 / 10**; omitted **0**, duplicated **0**, extraneous originals **0**.
- Manifest lines and bytes expected/observed: **2,553 / 2,553** lines and **139,392 / 139,392** bytes.
- Hashes expected/observed: **10 / 10 exact**.
- Per-source ledger sections expected/observed: **10 / 10**; per-source summary-delta sections: **10 / 10**.
- Atomic identifiers expected unique/observed unique: **384 / 384**; duplicate atomic IDs **0**. Delta identifiers expected unique/observed unique: **38 / 38**; duplicate delta IDs **0**.
- Repeated wording across Tasks, Wiring Contract, Acceptance, and Files was coalesced under a canonical capability atom; acceptance-suite atoms record the distinct done-gate packaging/status rather than pretending the repeated capability is new. Cross-source items are explicitly counted as interactions, not duplicate source features.
- Every requested taxonomy dimension was reviewed, including explicit absence/underspecification for permission, privacy, compliance, migration, concurrency, retry, and recovery contracts.
