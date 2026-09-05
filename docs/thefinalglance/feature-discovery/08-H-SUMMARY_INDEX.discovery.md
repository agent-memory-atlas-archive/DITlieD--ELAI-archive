# H-plan exhaustive feature discovery — summary-index pass

## Scope, method, and coverage

- Primary source read completely: docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md, 1,038/1,038 lines.
- Orientation source read: docs/thefinalglance/ELAI-SPEC-OVERVIEW.md. It is used only to understand ELAI-wide terms; it is not treated as additional H detail.
- This pass intentionally did not read the numbered H detail plans or grounding maps. They are enumerated in the deeper-source manifest for later agents.
- Authored-session coverage: 54/54 summary entries, H-1 through H-54.
- Track coverage: Track 1 (9/9), Track 2 (7/7), Track 3 (13/13), Track 4 (9/9), Track 5 (0 authored; explicitly reserved), Track 6 (4/4), Track 7 (6/6), Track 8 (1/1), Track 9 (2/2), Track 10 (1/1), Track 11 (2/2).
- Files present in H-plan at inventory time: 62 total = H-SUMMARY_INDEX.md + H-0_INDEX.md + 54 numbered detail plans + H-28-WIRE-1 supplement + 5 grounding maps.
- Classification legend:
  - REQ = explicit planned behavior, interface, invariant, or acceptance condition.
  - STATE = explicit shipped, dark, absent, partial, or corrected-current-state claim in the summary.
  - TEST = explicit observable effect, broken arm, or operational done-gate.
  - DEFER = explicitly deferred or sequenced-later work.
  - LIMIT = explicit honesty boundary, exclusion, or non-guarantee.
  - CORRECTION = explicit correction of an earlier premise, count, path, or novelty claim.
  - AMB = unresolved choice, inconsistent statement, or source not named precisely enough to resolve.
  - INFER = inference. No feature requirement below is tagged INFER; cross-plan composition is reported only where the summary states it.

## Global H doctrine and whole-letter behavior

Source anchors: H-SUMMARY_INDEX.md:1-23 (header/premise), :471-490 (Tracks 1-3 cross-cutting), :601-617 (Track 4), :698-708 (Track 6), :807-818 (Track 7), :1027-1037 (Track 11).

- G-001 [REQ] H is organized as efficiency, canvas-workbench, autonomous-trading trust, UX/user-memory, grounded-reasoning, DGM verification, reversibility, context lifecycle, review anchoring, and external-system steals.
- G-002 [STATE] Track 5 is reserved and unauthored; the 54 authored sessions are in Tracks 1-4 and 6-11.
- G-003 [REQ] Every H session must compose with an existing artifact by consuming its symbol or verdict, or build a genuinely absent surface; re-authoring shipped primitives is a P0 error.
- G-004 [REQ] A session is not done until wiring_contract_check.py --run-trigger passes against a real production entrypoint.
- G-005 [REQ] Cargo/build/test green alone is not a sufficient done condition.
- G-006 [REQ] A default-off flag is not an acceptable terminal state; experimental default-off paths need an allowlist row, expiry/promotion ownership, and a real ON-path smoke.
- G-007 [REQ] Missing off-agent evidence fails; it never skips or silently succeeds.
- G-008 [REQ] Every instrument must have a broken arm that makes the metric visibly wrong; a metric that remains healthy when its target is broken fails validation.
- G-009 [REQ] Efficiency numbers are measured per configuration and recomputable from the SSB journal under ELAI_SEED.
- G-010 [REQ] Literature figures cannot substitute for ELAI measurements and independent efficiency levers cannot be multiplied into a claimed product gain.
- G-011 [REQ] Cache optimizations must preserve the byte-stable prefix; observation reduction drops or masks trailing material rather than rewriting the prefix.
- G-012 [REQ] Output-token savings and cache savings are accounted separately; output tokens use the stated 5x billing weight.
- G-013 [REQ] Coder/reasoning roles are not force-JSONed merely to reduce output.
- G-014 [REQ] Canvas/model-driving acceptance is auditor-driven using a task frozen before implementation.
- G-015 [REQ] Canvas proof consists of an off-agent SSB event plus a rendered artifact, not implementer stdout.
- G-016 [REQ] A rendered picture is not semantic proof: dead-code views need coverage_count/entrypoint backstops and model-drawn artifacts remain valid-not-verified until VLM or executed-artifact validation.
- G-017 [REQ] A feature cannot become default-on until false-positive, over-block, over-escalation, or corruption rate (as applicable) is bounded on at least three real runs.
- G-018 [REQ] A fail-open or cloud-opaque path cannot advertise a guarantee it does not deliver.
- G-019 [REQ] Track 3 trust metrics use a persisted, agent-unresettable effective trial count N.
- G-020 [REQ] Track 3 evaluators, metric harnesses, thresholds, and the trial ledger are LOCKED/frozen.
- G-021 [REQ] LLM/auditor judgment is Tier-3 advisory and cannot be the promotion gate.
- G-022 [REQ] Each trust gate must kill its own frozen mutant corpus and satisfy the mandatory null-band-sidedness relation before ACTIVE.
- G-023 [LIMIT] Cross-hardware floating-point identity, pre-label concept drift, target-derived leakage, genuine-vs-lucky edge, and structural self-mod safety are undecidable residue routed to human Layer 3.
- G-024 [REQ] Track 4 memory is retrieval-on-demand in an approximately 100-500-token slice, never an always-injected block.
- G-025 [REQ] Bi-temporal supersession/soft-close preserves replay history; deletion from the append-only truth plane is banned.
- G-026 [REQ] User-memory does not enter auditor, critic, or adversary prompts.
- G-027 [REQ] Host-side QOL surfaces should add approximately zero local-model token cost.
- G-028 [REQ] Track 6 retains sound-checker-first grounding and adds only measured refinements: abstention credit, decorrelation retention, checker reach, and depth-scaled compute.
- G-029 [REQ] Track 7 replaces byte-derived fitness and keyword-only traps with execution-grounded, behaviorally measured verification.
- G-030 [REQ] All Track 7 work remains behind cfg(feature="darwinian-evolve") plus ELAI_DARWINIAN_LOOP_ENABLE.
- G-031 [DEFER] Track 7 live multi-generation bench/regret is RunPod-pending Tier 2; frozen deterministic fixtures are the immediate done-gate.
- G-032 [STATE] Schema/version counts in the summary are time-local snapshots: early Tracks 1-3 cite 207/484, Track 4 and 6 cite 220/497, and Tracks 7-11 cite 224/501. Executors must re-read live constants and apply ship-order-cumulative deltas.
- G-033 [AMB] The summary identifies a stale cross-language variant pin (about 482) but does not give its exact current reconciliation value; detail inspection and live-code read are required.

## Track 1 — Efficiency

### H-1 — Prompt caching on the production path (close the RuntimeAdapter split-brain + activate the hit-rate monitor)

Source: H-SUMMARY_INDEX.md:28-43 § H-1. Integration anchors: anthropic_api.rs:101,137,189; cache_check.rs:32; hook_prompt.rs:242; focus_tunnel/orchestrator.rs:218,280.

- H1-001 [STATE] The main handle_infer_request -> AnthropicClient::send_message path already calls apply_anthropic_cache_markers.
- H1-002 [STATE] Existing main-path markers are ephemeral and attach to the last system block and last tool definition.
- H1-003 [STATE] AnthropicApiRuntime::complete hardcodes cache_control=None while advertising supports_caching=true.
- H1-004 [STATE] RuntimeAdapter secondary callers hook_prompt, focus_tunnel orchestration, and rlaif_judge therefore receive no markers.
- H1-005 [STATE] cache_check.rs contains a hit-rate monitor but has no production caller and is default-off.
- H1-006 [REQ] Reuse and unify the existing marker logic on the RuntimeAdapter path; do not reimplement marker semantics.
- H1-007 [REQ] Activate live cache-hit measurement and make its result journal-recomputable.
- H1-008 [REQ] The feature is default-on and has no environment override.
- H1-009 [REQ] Feed the H-9 hit-rate dashboard tile.
- H1-010 [TEST] In a real two-turn task, the second RuntimeAdapter request carries cache_control markers.
- H1-011 [TEST] The live monitor reports a non-zero cache-read fraction recomputable from journal data.
- H1-012 [TEST] A single-turn fixture exposes the first cache write at the stated 1.25x price.

### H-2 — Cache-stable layout + breakpoints + 1h TTL (place real markers on the cloud path; assert byte-stability)

Source: H-SUMMARY_INDEX.md:44-58 § H-2. Integration anchors: context_budget.rs:1191-1200,1481,1507; runtime_client.rs:376-385.

- H2-001 [STATE] The allocator specifies two breakpoints after PhaseSpec and CodeContext.
- H2-002 [STATE] update_breakpoints computes blake3 byte-prefix stability in bp.valid.
- H2-003 [STATE] bp.valid is inert except for eprintln output.
- H2-004 [STATE] Slots 2-7 are currently fed empty slices at the runtime site.
- H2-005 [STATE] The allocator path is reached only for BudgetPolicy::Local; Cloud/ApiDefault do not call it.
- H2-006 [REQ] Feed real bytes for context slots into breakpoint calculation.
- H2-007 [REQ] Consume bp.valid to place cache_control markers only on a verified-stable cloud prefix.
- H2-008 [REQ] Add a second tail breakpoint with a 20-block lookback.
- H2-009 [REQ] Opt into a one-hour cache TTL using ttl:"1h".
- H2-010 [CORRECTION] The one-hour TTL does not require a beta header.
- H2-011 [REQ] Key minimum-prefix policy on the exact model identifier.
- H2-012 [REQ] Add the HD-2 debug assertion so a cache bust is a test failure.
- H2-013 [REQ] Feed measured cache behavior to H-9.
- H2-014 [TEST] A real multi-turn cloud task has a byte-identical cached prefix across turns.
- H2-015 [TEST] Its marker is placed only after stability verification and the measured hit rate rises.
- H2-016 [TEST] A deliberate prefix perturbation trips the cache-bust assertion.

### H-3 — Output-token stack: diff/FIM emission + streaming early-abort (the 5x-billed lever)

Source: H-SUMMARY_INDEX.md:59-74 § H-3. Integration anchors: focus_tunnel/orchestrator.rs:266; mode.rs:184; task_runner.rs:7762,7871; runtime_builder.rs:306; runtime_client.rs:508-531.

- H3-001 [STATE] FocusTunnel DIFF does not emit in production because editor_fs_override=None forces UnsupportedMode and whole-output STRUCTURED emission.
- H3-002 [STATE] FIM is double-gated and dark.
- H3-003 [STATE] Streaming early-abort is dark on all paths because runtime_builder sets streaming_validator=None.
- H3-004 [CORRECTION] The prior "sidecar-only" characterization of streaming early-abort is false.
- H3-005 [REQ] Relax ModeSelector Rule 1 so appropriate cloud edits can use DIFF.
- H3-006 [REQ] Thread a real BufferEditor through ResolvedConfig.
- H3-007 [REQ] Emit unified diff or SEARCH-REPLACE rather than whole-file rewrites where applicable.
- H3-008 [REQ] Construct Some(validator) at runtime builder setup.
- H3-009 [REQ] Invoke streaming validation approximately every 50 output tokens.
- H3-010 [REQ] On a terminal streaming condition, stop decoding rather than merely log a warning.
- H3-011 [REQ] Persist a StreamingEarlyTerminated SSB row.
- H3-012 [REQ] Reuse B-20 try_edit_tiers on the STRUCTURED path; do not rebuild its three-tier matcher.
- H3-013 [REQ] Never force JSON for the coder role.
- H3-014 [TEST] A real cloud edit emits a diff/SEARCH-REPLACE and journal accounting shows fewer output tokens.
- H3-015 [TEST] A forced early-abort condition both writes StreamingEarlyTerminated and terminates decoding.

### H-4 — Tool Search Tool + advanced-tool-use beta header (per-request fixed-cost cut)

Source: H-SUMMARY_INDEX.md:75-90 § H-4. Integration anchors: production_agent_rpc.rs:266,970,1163; anthropic.rs:695-701; telemetry_types.rs:69-72.

- H4-001 [STATE] ELAI sends no advanced-tool-use header.
- H4-002 [STATE] The full 16-tool built-in catalog is included in every request.
- H4-003 [STATE] The production OAuth bearer path emits only oauth-2025-04-20 and ignores configured profile betas.
- H4-004 [REQ] Join advanced-tool-use-2025-11-20 into the OAuth beta header at the mandatory production path.
- H4-005 [REQ] Put the catalog behind tool_search_tool_regex_20251119.
- H4-006 [REQ] Enable deferral only when the catalog-size gate says it is worthwhile.
- H4-007 [REQ] Keep a tiny catalog inline.
- H4-008 [REQ] When a custom ANTHROPIC_BASE_URL disables server-side deferral, use ELAI's ToolSearch emulation.
- H4-009 [REQ] Repair C-30 dkg_* tools using the full three-arm catalog + conversion + dispatch pattern.
- H4-010 [CORRECTION] The stated full-to-deferred definition reduction is approximately 77K to 8.7K, corrected from 72K.
- H4-011 [TEST] A real OAuth task shows the beta header on the wire.
- H4-012 [TEST] Tool definitions load on demand.
- H4-013 [TEST] Measured tool-definition tokens per request decrease while the small-tool-set case remains inline.

### H-5 — Batch API for non-interactive lanes (flat 50% off, zero quality cost)

Source: H-SUMMARY_INDEX.md:91-104 § H-5. Integration anchors: anthropic.rs:440,678; cosmos_worker.rs:853; task_runner.rs:2172-2188.

- H5-001 [STATE] There is no Anthropic Batch API transport; all LLM lanes run synchronously at full price.
- H5-002 [REQ] Add a batch sibling using POST /v1/messages/batches.
- H5-003 [REQ] Use the stated 50% batch pricing and one-hour cache TTL.
- H5-004 [REQ] Route Cosmos refresh through batching, including its sequential per-artifact tick loop.
- H5-005 [REQ] Route web-research fan-out through batching.
- H5-006 [REQ] Route bench and evaluation lanes, including GEPA/SecCodePlt/SAFEDELTA-like noninteractive work, through batching where eligible.
- H5-007 [REQ] Preserve synchronous streaming for interactive /task and chat REPL calls.
- H5-008 [REQ] Apply W-10 tiers to RunPod-dependent evals: deterministic fixture now, separately gated live checkbox.
- H5-009 [REQ] Poll submitted batches to completion.
- H5-010 [REQ] Journal batch_lifecycle transitions Submitted -> Completed.
- H5-011 [REQ] Make the 50%-priced spend recomputable from journal entries.
- H5-012 [DEFER] A batch-status dashboard tile is optional.
- H5-013 [TEST] An elai Cosmos refresh reaches the batch endpoint, completes, and records lifecycle/spend while a human-waited call never batches.

### H-6 — Deterministic observation masking + memory read-back (cache-preserving context efficiency)

Source: H-SUMMARY_INDEX.md:105-120 § H-6. Integration anchors: context_budget.rs:1525; session_dag.rs:1507; retrieval.rs:46; live_cli.rs:1252-1264; task_runner.rs:9379; compaction_ladder.rs:45.

- H6-001 [STATE] adaptive_reduce exists with no non-test caller.
- H6-002 [STATE] recover_stubs can recover blake3-verified content from SSB but production substitutes the literal [stubbed].
- H6-003 [STATE] get_relevant_facts exists but memory read-back is dark.
- H6-004 [REQ] Make deterministic adaptive reduction the default long-session manager.
- H6-005 [REQ] Enable the E-25 compaction ladder by default in cheapest-first order.
- H6-006 [REQ] Wire recover_stubs into the live /continue/read path.
- H6-007 [REQ] Wire get_relevant_facts into the applicable live memory path.
- H6-008 [REQ] Mask trailing observations without changing the cache prefix.
- H6-009 [REQ] Reuse the existing ContextReduction SSB payload; add no action type.
- H6-010 [REQ] Keep masked tokens retrievable from the journal.
- H6-011 [TEST] A long real session writes ContextReduction while the HD-2 prefix assertion still holds.
- H6-012 [TEST] /continue restores real verified content rather than the placeholder.
- H6-013 [TEST] A memory-relevant turn retrieves stored facts.

### H-7 — Calibrated routing: architect / editor / verifier (smarter orchestration, not a smarter model)

Source: H-SUMMARY_INDEX.md:121-138 § H-7. Integration anchors: runtime_setup.rs:533-536; task_runner.rs:7606; escalation_policy.rs:261; cloud_mode.rs:416.

- H7-001 [STATE] RuntimeOrchestrator and PerRoleRuntime are implemented but have no task_runner dispatch references.
- H7-002 [STATE] RuntimeRoutingPolicy::evaluate is default-off and observational; it does not alter the provider.
- H7-003 [STATE] RouteDecisionEvent machinery exists without a production emitter.
- H7-004 [STATE] The only live routing leg is CloudModeRouter cap enforcement plus EscalateToCloud.
- H7-005 [REQ] Build on the live cap-enforcement leg rather than replacing it.
- H7-006 [REQ] Route planning/reasoning to an architect on a flagship cloud model.
- H7-007 [REQ] Route cheaply checkable edits to a cheap 7B editor.
- H7-008 [REQ] Run an independent-context verifier as the auditor.
- H7-009 [REQ] Replace hardcoded layer_failed:1 with an externally calibrated signal.
- H7-010 [REQ] Never use 7B self-confidence as the calibration signal.
- H7-011 [REQ] Feed evaluate a real RoutingContext and make its decision effective.
- H7-012 [REQ] Emit RouteDecisionEvent through SSB.
- H7-013 [REQ] Gate routing jointly on cost and correctness; a cheap edit that fails an off-agent check is not cheaper.
- H7-014 [REQ] Bound over-escalation on at least three runs before default-on.
- H7-015 [TEST] A real task separates architect/editor work, runs verifier in separate context, persists a recomputable decision, and measures over-escalation.
- H7-016 [LIMIT] The +9.6 percentage-point Aider figure is supporting prior art, not an ELAI measurement.

### H-8 — Persist OracleAnswerCache (+ idempotent grounding cache)

Source: H-SUMMARY_INDEX.md:139-151 § H-8. Integration anchors: oracle_cache.rs:173; repl_command.rs:286-459.

- H8-001 [STATE] OracleAnswerCache is blake3-keyed and uses tier-derived TTLs.
- H8-002 [STATE] It is live in /research and /ask but held only in an in-process Mutex<CacheInner>.
- H8-003 [REQ] Persist this cache to disk so answers survive process restart.
- H8-004 [REQ] Limit durable entries to idempotent grounding queries.
- H8-005 [REQ] Never serve context-dependent code generation from the durable cache because staleness threatens correctness.
- H8-006 [STATE] PromptCache already persists at $HOME/.elai/cache/prompt-cache.
- H8-007 [CORRECTION] The earlier framing of PromptCache as ephemeral is wrong; do not reimplement it.
- H8-008 [REQ] Preserve deterministic TTL behavior via cli.clock.
- H8-009 [TEST] A /research answer stored in one process is an OracleCacheHit in a fresh process with a corresponding SSB row.
- H8-010 [TEST] The confinement guard rejects code-generation cache serving.

### H-9 — Cost cockpit dashboard (activate the dark spend ledger; the believe-the-number instrument)

Source: H-SUMMARY_INDEX.md:152-167 § H-9. Integration anchors: grounding_client.rs:887-940; commands.rs:5088; CloudPanel.svelte.

- H9-001 [STATE] SpendAccumulator and CloudSpend emission are gated on an optional SSB writer.
- H9-002 [STATE] The sole cloud-client constructor does not call with_ssb_writer, so the end-to-end ledger is dark.
- H9-003 [STATE] CloudPanel therefore polls and renders zeros.
- H9-004 [STATE] cost_per_task_cents has only a test feeder and remains None in production.
- H9-005 [STATE] No token-tracker UI exists.
- H9-006 [REQ] Attach with_ssb_writer to the cloud client, following the shipped TemplateCrystallizer pattern.
- H9-007 [REQ] Build the production cost feeder.
- H9-008 [REQ] Show per-response input, output, and cached-token counts.
- H9-009 [REQ] Show a cache-expiry timer.
- H9-010 [REQ] Show dollars per resolved task.
- H9-011 [REQ] Show live cache-hit rate.
- H9-012 [REQ] Make every displayed value recomputable from the journal.
- H9-013 [REQ] Serve as the falsifiability instrument for H-1 and H-2.
- H9-014 [TEST] A real cloud task makes all values non-zero/meaningful and a dashboard smoke confirms live updates.

## Track 2 — Canvas-workbench

### H-10 — Canvas shell + CODE-GRAPH (read-only) [the thesis slice]

Source: H-SUMMARY_INDEX.md:171-188 § H-10. Integration anchors: commands.rs:2507; views.rs:175; entry_points/detect.rs:142; WindowHost.svelte; Canvas.svelte.

- H10-001 [REQ] Add a 2D generative canvas as a new WindowHost window type.
- H10-002 [REQ] Render the codebase graph with disconnected dead-code islands.
- H10-003 [CORRECTION] The symbol call-graph view already reaches a canvas through build_symbol_view -> get_graph_view_data(mode=symbol) -> GraphWindow.
- H10-004 [STATE] PageRank is producer-dark: get_pagerank_top reads dkg_nodes_pagerank but nothing materializes that table.
- H10-005 [STATE] dkg_edges contains no call edges despite the existing symbol-view surface.
- H10-006 [REQ] Before later slices ship, measure the worst-case DKG node and edge count (blocking B0).
- H10-007 [REQ] Select cytoscape-WebGL 3.34 provisionally with graceful fallback, or sigma 3.0.3, using the cited section-4 rule and measured corpus.
- H10-008 [LIMIT] Sigma v4 alpha is blocked.
- H10-009 [REQ] Support semantic zoom and level-of-detail rendering.
- H10-010 [REQ] Supplement topology with the entrypoints map and coverage_count.
- H10-011 [REQ] Render topology-suspected dead code differently from coverage-confirmed dead code.
- H10-012 [REQ] Build the persistent PageRank materializer.
- H10-013 [TEST] The B0 worst case renders at at least 60fps, evidenced by the showFps HUD.
- H10-014 [TEST] A known orphan separates as an island.
- H10-015 [TEST] A Tauri-invoked-only function is attached by the entrypoints map and is not labeled dead.
- H10-016 [TEST] The PageRank table and panel are non-empty.

### H-11 — Live SSB layer + file explorer + scoped fs surface

Source: H-SUMMARY_INDEX.md:189-205 § H-11. Integration anchors: graph.svelte.ts:31; commands.rs:594,5936; default.json:6; Cargo.toml:22; lib.rs:435.

- H11-001 [REQ] Overlay CanvasActivityEvent data on whichever canvas mode is active.
- H11-002 [REQ] Show an active-node halo.
- H11-003 [REQ] Show traversed edges with dash-flow.
- H11-004 [REQ] Offer a follow-agent toggle.
- H11-005 [REQ] Represent an unindexed-file resolution failure as a transient ghost node.
- H11-006 [REQ] Reconcile a ghost when indexing catches up or fade it into a dropped marker; never leave a silent gap.
- H11-007 [REQ] Add a VS Code-style file explorer.
- H11-008 [REQ] Synchronize explorer selection and canvas focus bidirectionally over existing focus_node/consume_focus keyed by path.
- H11-009 [STATE] list_projects/create_project are registered but have zero UI callers.
- H11-010 [REQ] Activate list_projects/create_project.
- H11-011 [STATE] No filesystem surface, fs capability, or tauri-plugin-fs dependency is present; only dialogs exist.
- H11-012 [REQ] Build filesystem commands scoped to each project root.
- H11-013 [REQ] Verify the plugin version instead of assuming one.
- H11-014 [REQ] A within-root node selects/highlights its file.
- H11-015 [REQ] A cross-root node shows a mount affordance.
- H11-016 [REQ] An external node shows an explicit external state rather than a no-op.
- H11-017 [TEST] A folder tree renders from disk and a live task's focus marker follows real SSB rows, including ghost reconciliation.

### H-12 — PLAN + RESEARCH modes + shared chrome (orbit feature-parity, flat)

Source: H-SUMMARY_INDEX.md:206-220 § H-12. Integration anchors: plan.svelte.ts; WindowHost.svelte; workspace.svelte.ts:82; canvas.svelte.ts.

- H12-001 [REQ] Port OrbitMode1 PLAN into a flat clipping-frame mode.
- H12-002 [REQ] Render concept -> phase -> step -> babystep as nested contains nodes.
- H12-003 [REQ] Light the FSM-active plan node from real plan state.
- H12-004 [REQ] Port OrbitMode3 RESEARCH into a flat research-tree mode.
- H12-005 [REQ] Render citations and red contradiction edges.
- H12-006 [REQ] Share one zoom control across modes.
- H12-007 [REQ] Share one search control across modes.
- H12-008 [REQ] Share one inspector across modes.
- H12-009 [REQ] Provide one mode bar.
- H12-010 [REQ] Model canvas mode as a plain flag in canvas.svelte.ts, not an FSM substate.
- H12-011 [REQ] Make Canvas the default window behind a workspace flag.
- H12-012 [REQ] Enforce single-consumer SSB handover: the active window owns the subscription and orbit pauses.
- H12-013 [STATE] Canvas-store isolation already passes; the canvas store imports no three.js state.
- H12-014 [TEST] Real plan and research runs render their live structures and switching never double-applies events into a shared store.

### H-13 — Model drives the canvas (ops + dispatcher + SSB action_types + ARCHITECTURE mode)

Source: H-SUMMARY_INDEX.md:221-238 § H-13. Integration anchors: production_agent_rpc.rs:266,970,1163; ssb.rs lockstep sites; task_runner.rs:6208; CanvasPanel.

- H13-001 [REQ] Expose incremental canvas operations such as createNode, connect, updateNode, annotate, and fork.
- H13-002 [REQ] Add canvas_* ToolDefinitions through the three-arm catalog + conversion + dispatch pattern.
- H13-003 [REQ] Add CanvasDispatcher with per-mode namespace gating.
- H13-004 [REQ] Apply sanitizeAction repair/validation.
- H13-005 [REQ] Reject dangling-ID operations and report them; never silently repair them into a different action.
- H13-006 [REQ] Add Pattern-2 CanvasOpApplied and CanvasActivityEvent action types with all nine SSB lockstep sites updated (stated baseline +2).
- H13-007 [REQ] Add ARCHITECTURE mode.
- H13-008 [REQ] Support canvas_render_diagram.
- H13-009 [REQ] Bridge Mermaid import into the canvas operation model.
- H13-010 [REQ] Give each mode a constrained grammar.
- H13-011 [REQ] Enforce a CI grammar-compilation ceiling.
- H13-012 [REQ] On grammar unavailability/timeout, block the operation with an explicit reason rather than fall back silently to free-form.
- H13-013 [REQ] Add canvas_screenshot using Tauri-native capture.
- H13-014 [REQ] Route screenshots to the already-shipped media_validator Tier 2 consumer.
- H13-015 [REQ] Apply canvas_* journal rows live in CanvasPanel.
- H13-016 [TEST] An auditor uses a real task authored before H-13; CanvasOpApplied rows land and corresponding nodes render.
- H13-017 [TEST] A unit-only path with no production dispatcher connection fails the gate.
- H13-018 [TEST] A forced grammar timeout visibly blocks an op with a reason.

### H-14 — APP/WEBSITE DESIGN mode (web-first; RN/Expo deferred)

Source: H-SUMMARY_INDEX.md:239-256 § H-14. Integration anchors: dev-preview proxy; DesignPageNode.svelte; build-time tagger; recast writeback command.

- H14-001 [REQ] Represent each app screen as a design.page node with a live preview.
- H14-002 [REQ] Use navigation edges to connect pages.
- H14-003 [REQ] Let a user click an element inside a preview to edit it.
- H14-004 [REQ] Build a frameable-origin dev-preview proxy before the mode ships.
- H14-005 [REQ] At that proxy, intentionally manage X-Frame-Options/frame-ancestors and scope the Service Worker.
- H14-006 [REQ] Add build-time data-oid tagging based on mirrored @onlook/parser id behavior.
- H14-007 [CORRECTION] Do not depend on the nonexistent/phantom @onlook/babel-plugin-react.
- H14-008 [REQ] Keep DOM ids opaque; store id -> {file, range, scope} on the server.
- H14-009 [REQ] Never encode line/column in the DOM attribute.
- H14-010 [REQ] Use deterministic recast 0.23.11 patches for style and text edits without a model call.
- H14-011 [REQ] Use the LLM only for structural edits.
- H14-012 [REQ] Preserve formatting through recast on structural writeback.
- H14-013 [REQ] First prove recast writeback against an actual large React codebase.
- H14-014 [LIMIT] Current prior art cannot reliably diff against an existing arbitrary codebase; no claim is allowed before the prove-it spike.
- H14-015 [REQ] Measure and bound writeback-corruption rate.
- H14-016 [CORRECTION] The novelty claim is sandbox/proxy control, not iframe rendering; Onlook is also iframe-based.
- H14-017 [DEFER] React Native/Expo support is deferred; ship web-first.
- H14-018 [TEST] Off-agent capture shows an actually painted app, not merely HTTP 200.
- H14-019 [TEST] Clicking resolves through opaque id to the correct AST range.
- H14-020 [TEST] Style edit uses deterministic recast only; structural edit uses model plus formatting-preserving recast on a real codebase.

### H-15 — spawn_view reply channel + taint-gated deixis + snapshot/delta [the trust-spine moat]

Source: H-SUMMARY_INDEX.md:257-276 § H-15. Integration anchors: production_agent_rpc.rs three-arm path; ssb.rs; G-1 capability_flow::taint_text; renderer registry.

- H15-001 [REQ] Add spawn_view{view_type, seed_ops[]} as a general-purpose visual reply verb composed from existing operations.
- H15-002 [REQ] Add Pattern-2 CanvasViewEvent with ViewSpawned, UiAction, and StateCheckpoint variants (stated baseline +1).
- H15-003 [REQ] H15-1 ships a comparison/branch board using existing operations.
- H15-004 [REQ] H15-2 accepts inbound CanvasUiAction::Select.
- H15-005 [REQ] Selection hands the exact current node state back to the model.
- H15-006 [REQ] Selection uses tiered read-back.
- H15-007 [REQ] Run the G-1 taint check at the selection handoff.
- H15-008 [REQ] Flag web-sourced/tainted node content at handoff rather than treating it as trusted instruction.
- H15-009 [REQ] H15-3 maintains per-tab sequence numbers.
- H15-010 [REQ] Write periodic state checkpoints.
- H15-011 [REQ] Reconcile snapshot plus delta so a tab can rebuild from the journal tail after restart.
- H15-012 [REQ] Bind a view to its producing run and make it reproducible through G-2 RRA.
- H15-013 [LIMIT] AG-UI/MCP-Apps convergence is an internal-contract influence, not an interoperability claim.
- H15-014 [TEST] A real reply writes ViewSpawned, renders the board, and produces off-agent Layer-4 evidence.
- H15-015 [TEST] Tainted selection is marked at the model handoff.
- H15-016 [TEST] Killing/reopening the dashboard mid-run reconstructs the tab from SSB.

### H-16 — Retire the orbit

Source: H-SUMMARY_INDEX.md:277-291 § H-16. Integration anchors: workspace.svelte.ts:82; WindowHost.svelte:81; package.json:23; scene/ and orbit/ trees.

- H16-001 [REQ] Delete orbit only after Canvas-as-default has shipped for a full release without regressions.
- H16-002 [REQ] Delete scene/three-scene.ts.
- H16-003 [REQ] Delete three-orbit* files.
- H16-004 [REQ] Delete StarCanvas.svelte, sky-dome.ts, star-points.ts, and orbit/node-mandala.ts.
- H16-005 [REQ] Delete OrbitCanvas and ActivityWindow's orbit renderer.
- H16-006 [REQ] Remove three, troika-three-text, and @types/three dependencies.
- H16-007 [STATE] Store isolation makes this mechanically clean because the canvas store does not import three.js.
- H16-008 [REQ] Preserve the already-2D starfield/canvas experience after removal.
- H16-009 [TEST] Build remains green and a zero-grep finds no imports matching from.*scene/three.
- H16-010 [TEST] Dashboard canvas behavior has no regression.

## Track 3 — Trust-hardening for autonomous trading (T-1 through T-13)

### H-17 — Persisted untamperable trial ledger (T-1) [the selection-bias KEYSTONE]

Source: H-SUMMARY_INDEX.md:295-309 § H-17. Integration anchors: new trial_ledger.rs; all backtest/eval paths; SSB TrialRegistered.

- H17-001 [REQ] Treat optimizer search itself as a selection-bias source even when every agent is honest.
- H17-002 [REQ] Persist the cumulative number N of tried strategies/configurations.
- H17-003 [REQ] Store strategy_trials on the shipped append-only SSB WAL.
- H17-004 [REQ] Route every backtest and evaluation through one register_trial() choke point.
- H17-005 [REQ] Expose trial_count() as the read API.
- H17-006 [REQ] Make the ledger agent-unresettable: no agent delete, reset, or update operation.
- H17-007 [REQ] Use ONC effective-N clustering for correlated trials rather than blindly treating all trials as independent.
- H17-008 [STATE] register_trial and strategy_trials are absent (NN-1), so this is genuine build work.
- H17-009 [REQ] Add ledger and registration function to the frozen gate manifest.
- H17-010 [REQ] Emit TrialRegistered for each registered trial.
- H17-011 [REQ] Surface N in the H-18/H-29 cockpit.
- H17-012 [TEST] At least three distinct fixture configurations create N rows and a monotone count recomputable from SSB.
- H17-013 [TEST] Zero ledger rows fail closed downstream with trial_count_missing.

### H-18 — Deflated-Sharpe / PBO / MinBTL promotion gate (T-2) [#1 net-new trust gate]

Source: H-SUMMARY_INDEX.md:310-322 § H-18. Integration anchors: rule_engine.rs:98; new deflation_gate.rs; numpy/scipy sidecar; finance/deflation_promotion.toml; promotion path.

- H18-001 [STATE] No finance-deflation evaluator is wired to strategy promotion (NN-2).
- H18-002 [REQ] Build one LOCKED COMPOSITE promotion gate.
- H18-003 [REQ] Require Deflated Sharpe Ratio to clear its threshold.
- H18-004 [REQ] Require PBO computed with CSCV to clear its threshold.
- H18-005 [REQ] Require MinBTL to clear its threshold.
- H18-006 [REQ] Read effective N only from H-17.
- H18-007 [REQ] Fail closed with trial_count_missing when N is unavailable.
- H18-008 [REQ] Run DSR/PBO/MinBTL math in a frozen NumPy/SciPy Tier-2 sidecar.
- H18-009 [REQ] Enforce the sidecar verdict with deterministic Tier-1 Rust METRIC_GATE logic.
- H18-010 [REQ] Lock thresholds mutable=0; the stated default is Harvey-Liu t > 3.0.
- H18-011 [REQ] Persist DSR, PBO, and N in DeflationGateEvaluated.
- H18-012 [REQ] Expose {DSR, PBO, N} in a reviewer drawer.
- H18-013 [TEST] A high raw-Sharpe strategy that fails DSR-at-N is blocked with the triplet persisted.
- H18-014 [TEST] A genuine low-N fixture passes.
- H18-015 [TEST] No-trial-row input is blocked.
- H18-016 [LIMIT] DSR is necessary, not sufficient; it does not prove genuine edge.

### H-19 — Reusable-holdout query budget (T-3)

Source: H-SUMMARY_INDEX.md:323-334 § H-19. Integration anchors: new holdout_budget.rs; finance/holdout_budget.toml; OOS evaluation; HoldoutBudgetEvent.

- H19-001 [STATE] Adaptive reuse of the OOS holdout is currently unlimited.
- H19-002 [REQ] Build a differentially private reusable-holdout budget using Thresholdout/Ladder behavior.
- H19-003 [REQ] Persist its spend counter append-only under the same no-reset discipline as H-17.
- H19-004 [REQ] Execute a Thresholdout-style DP OOS query.
- H19-005 [REQ] Refuse OOS queries with budget_exhausted after budget_total is consumed.
- H19-006 [REQ] Resume only after a fresh holdout is provisioned.
- H19-007 [REQ] Restrict provisioning to a frozen/human path; an agent reset API must not exist or compile.
- H19-008 [REQ] Use closed-form DP noise; no sidecar is needed.
- H19-009 [REQ] Lock holdout_budget.toml mutable=0.
- H19-010 [REQ] Emit HoldoutBudgetEvent kinds QuerySpent, Exhausted, and Provisioned.
- H19-011 [REQ] Expose a remaining-budget gauge.
- H19-012 [TEST] Exactly budget_total queries spend, the next refuses, provisioning requires the frozen path, and all spend recomputes from SSB.

### H-20 — Leakage static-analysis COMPOSITE on the taint graph (T-4) [the trading-specific killer]

Source: H-SUMMARY_INDEX.md:335-348 § H-20. Integration anchors: provenance.rs:145; capability_flow taint_text:534; proposed taint_flow.rs or leakage-scoped DKG pass; new leakage_composite.rs; finance/leakage.toml.

- H20-001 [REQ] Add a Tier-1 COMPOSITE leakage check over taint/provenance dataflow.
- H20-002 [REQ] Block any fit-site that reaches both train and test sinks.
- H20-003 [REQ] Block cross-boundary normalization that taints earlier-timestamp data.
- H20-004 [REQ] Block full-series PCA before the split.
- H20-005 [REQ] Require a declared train/test split boundary through SCHEMA_REQUIRED.
- H20-006 [REQ] Add a Tier-2 AST_PATTERN for a missing signal-to-execution shift(1).
- H20-007 [REQ] Fail closed.
- H20-008 [REQ] Allow suppression only through a registered allowlist; never silently suppress.
- H20-009 [REQ] Run before tool use and/or promotion as specified.
- H20-010 [REQ] Lock leakage.toml mutable=0.
- H20-011 [REQ] Emit LeakageBlocked with the four-kind discriminator and report the fit site.
- H20-012 [REQ] Show the tainted fit site in a UI drawer.
- H20-013 [STATE] The provenance/taint primitive exists but is partial/wire-dark; the G-1 live taint_flow.rs prerequisite is absent.
- H20-014 [AMB] At authoring time choose between composing with live G-1 taint flow and building a finance-scoped DKG dataflow pass; the summary does not decide.
- H20-015 [LIMIT] Static analysis is intended to be sound for fit/normalize/PCA patterns; target-derived leakage remains Tier 3/human residue.
- H20-016 [REQ] H-21 is the runtime backstop for dynamic cases static analysis misses.
- H20-017 [TEST] Fit-before-split blocks with a location, split-then-fit passes, and a flagged pandas idiom cannot proceed without registered suppression.

### H-21 — Runtime look-ahead / causality differential gate (T-5) [the un-gameable leakage backstop]

Source: H-SUMMARY_INDEX.md:349-360 § H-21. Integration anchors: determinism.rs:317; new lookahead_differential.rs; frozen feature-runner sidecar.

- H21-001 [STATE] No perturb-and-diff leakage backstop exists (NN-5).
- H21-002 [REQ] Execute the feature/signal pipeline once on full data.
- H21-003 [REQ] Execute it again with future observations truncated.
- H21-004 [REQ] Compare every output column.
- H21-005 [REQ] Require equality within configured ULP/relative tolerance.
- H21-006 [REQ] Treat any column altered by removal of future data as a future leak.
- H21-007 [REQ] Run feature generation in a frozen Tier-2 runner.
- H21-008 [REQ] Perform the deterministic column diff in Tier-1 Rust.
- H21-009 [REQ] Reuse DriftComparator's first-divergence reporting shape.
- H21-010 [REQ] Emit LookaheadDifferential with Causal or FutureLeak.
- H21-011 [REQ] Show the first changing column in the leakage drawer.
- H21-012 [TEST] iloc[t+1] is blocked; a causal pipeline has zero diff and passes.
- H21-013 [LIMIT] Cross-hardware floating-point residue is handled with relative tolerance, not claimed bitwise identity.

### H-22 — Trading metamorphic null-test COMPOSITE (T-6) [does the edge survive perturbation]

Source: H-SUMMARY_INDEX.md:361-373 § H-22. Integration anchors: metamorphic_testing.rs; permutation null-band sidecar; finance/trading_nulls.toml; pre-promotion/commit hook.

- H22-001 [STATE] Existing gates can admit an overfit/leaking backtest; named trading relations are absent (NN-6).
- H22-002 [REQ] Build a frozen COMPOSITE of named metamorphic relations.
- H22-003 [REQ] Future-shuffle must collapse apparent edge to noise.
- H22-004 [REQ] Include time-reversal.
- H22-005 [REQ] Include a zero-drift permutation null.
- H22-006 [REQ] Compare DSR against the permutation null using N from H-17.
- H22-007 [REQ] Include multiplicative scale invariance.
- H22-008 [REQ] Include additive invariance.
- H22-009 [REQ] Require cost monotonicity and concavity under a square-root-law model.
- H22-010 [REQ] Require seed determinism using the blake3 hash of the equity curve.
- H22-011 [REQ] Add relations to the shipped RelationLibrary and reuse MrViolated.
- H22-012 [REQ] Only the human-set alpha is LOCKED.
- H22-013 [REQ] Subject the relations themselves to H-27 mutation validation.
- H22-014 [REQ] Persist the permutation null band; its absence fails closed.
- H22-015 [REQ] Emit TradingNullTest with its four-kind discriminator and show the null band.
- H22-016 [TEST] A leaky strategy whose edge survives shuffled future blocks; a genuine fixture collapses to noise and passes.

### H-23 — Same-seed re-run hash-equality METRIC_GATE (T-7) [reproducibility centerpiece]

Source: H-SUMMARY_INDEX.md:374-387 § H-23. Integration anchors: rra.rs:200; run_subcommand.rs:584; determinism.rs:317; new rerun_equality.rs; seed_coverage/env_pin_manifest rule files.

- H23-001 [STATE] ELAI_SEED records intent, but no gate reruns an entrypoint and compares declared metric outputs (NN-7).
- H23-002 [REQ] Reinvoke the real entrypoint under the recorded environment and ELAI_SEED through shipped RRA/replay.
- H23-003 [REQ] Hash declared metric outputs, not stdout.
- H23-004 [REQ] Compare run 2 against run 1 with blake3.
- H23-005 [REQ] Report the first divergence via DriftComparator.
- H23-006 [REQ] Add seed-coverage AST rules for unseeded NumPy, PyTorch, random, and DataLoader shuffle usage.
- H23-007 [REQ] Require an environment-pin manifest through SCHEMA_REQUIRED.
- H23-008 [REQ] Add elai replay --verify --metric-digest.
- H23-009 [REQ] Emit RerunHashEquality as Equal or Divergent.
- H23-010 [REQ] Show the metric digest in the RRA viewer.
- H23-011 [CORRECTION] RRA and replay are shipped; any G-summary statement that they are absent is stale.
- H23-012 [LIMIT] The assertion applies only to same-environment/same-seed; cross-hardware FP is recorded but not asserted.
- H23-013 [TEST] Hidden unseeded RNG diverges and fails; seeded code hashes equal twice; the AST gate blocks unseeded calls at TESTING_FIRST.

### H-24 — Sim-to-real parity gate (T-8)

Source: H-SUMMARY_INDEX.md:388-400 § H-24. Integration anchors: ssb.rs:23502; run_subcommand.rs:584; new sim_real_parity.rs; repro/sim_real_parity.toml.

- H24-001 [STATE] No gate ensures research path equals live path or replay equals golden (NN-8).
- H24-002 [REQ] Part A: AST_PATTERN bans mode-divergent branches such as if live:/if backtest:.
- H24-003 [REQ] Part A: ban duplicate fill/router implementations.
- H24-004 [REQ] Part A: require a single shared entrypoint manifest through SCHEMA_REQUIRED.
- H24-005 [REQ] Part C: replay the recorded event log under ELAI_SEED.
- H24-006 [REQ] Part C: compare blake3(replay) with blake3(golden).
- H24-007 [REQ] Report the first mismatch.
- H24-008 [REQ] Run Part A at PreToolUse and Part C pre-promotion.
- H24-009 [REQ] Lock sim_real_parity.toml mutable=0.
- H24-010 [REQ] Emit SimRealParity with its three-kind discriminator and show a parity drawer.
- H24-011 [DEFER] Part B behavioral numeric parity requires a live/paper run and locked thresholds; it is deferred Tier 2.
- H24-012 [LIMIT] Do not claim numeric sim-to-real parity until Part B exists.
- H24-013 [TEST] if backtest fast-path blocks at build; divergent replay fails at first mismatch; shared-entrypoint clean replay passes.

### H-25 — Result manifest + re-derivation verifier (T-9) [bind the empirical NUMBER, not just the test]

Source: H-SUMMARY_INDEX.md:401-414 § H-25. Integration anchors: evidence.rs:845; gate.rs:441; new result_manifest.rs; finance/result_manifest.toml.

- H25-001 [STATE] F-4 binds a test PASS to stdout but no mechanism binds an empirical value such as Sharpe/P&L to its provenance (NN-9).
- H25-002 [REQ] Require a Result Manifest.
- H25-003 [REQ] Manifest fields are value, captured_output_blake3, parser_expr, data_hash, code_commit, env_lock_hash, elai_seed, run_id, n_trials, dsr_pvalue, and pbo.
- H25-004 [REQ] Reparse the reported number from orchestrator-captured output.
- H25-005 [REQ] Rehash all pinned inputs.
- H25-006 [REQ] Bind n_trials/DSR/PBO to H-17/H-18 outputs.
- H25-007 [REQ] Add an AST rule forbidding strategy code from importing, redefining, or monkeypatching the frozen metric harness.
- H25-008 [REQ] Extend F-4/RRA evidence rather than restate its stdout-hash leg.
- H25-009 [REQ] Lock result_manifest.toml mutable=0.
- H25-010 [REQ] Emit ResultManifestVerified kinds ReDerived, FabricationDetected, and MonkeypatchBlocked.
- H25-011 [REQ] Surface this manifest in the reviewer cockpit composed with G-5.
- H25-012 [TEST] A hand-edited value whose digest cannot be rederived fails; a clean manifest passes; harness monkeypatching blocks.
- H25-013 [REQ] Route visual empirical results to media_validator VLM rather than treating pixels as numeric proof.

### H-26 — Runtime differential / shadow exec + financial-invariant MR library (T-10)

Source: H-SUMMARY_INDEX.md:415-427 § H-26. Integration anchors: metamorphic_testing.rs; mutation_testing.rs; new shadow_exec.rs; finance/financial_invariants.toml.

- H26-001 [STATE] Static/wiring gates cannot detect wrong numeric values such as P&L sign flips or cancellation (NN-10).
- H26-002 [REQ] Run a seeded shadow executor on shared inputs.
- H26-003 [REQ] Compare new implementation against the prior implementation and/or a QuantLib oracle.
- H26-004 [REQ] Gate on bounded ULP/relative divergence through FPDiff.
- H26-005 [REQ] Add a put-call-parity metamorphic relation.
- H26-006 [REQ] Add strike monotonicity.
- H26-007 [REQ] Add P&L additivity.
- H26-008 [REQ] Add scale invariance.
- H26-009 [REQ] Add numeric-stability behavior.
- H26-010 [REQ] Ban naked floating-point == via AST.
- H26-011 [REQ] Admit Daikon-mined invariants only as PROPOSED until each kills a seeded mutant.
- H26-012 [REQ] Lock financial_invariants.toml mutable=0 and run pre-promotion plus refactor hook.
- H26-013 [REQ] Emit DifferentialDivergence with its three-kind discriminator and show a divergence drawer.
- H26-014 [TEST] A sign-flipped refactor fails; a clean refactor stays within bound; put-call violation and naked float equality are caught.
- H26-015 [LIMIT] QuantLib is Tier-2 advisory and not the hard promotion gate.

### H-27 — Gate self-validation (T-11) [verify-the-verifiers; the GBM-band fix generalized]

Source: H-SUMMARY_INDEX.md:428-441 § H-27. Integration anchors: mutation_testing.rs; metamorphic_testing.rs; trap_tasks.rs:184; lifecycle.rs:871; new gate_self_validation.rs.

- H27-001 [REQ] Validate each gate revision during PROPOSED -> ACTIVE.
- H27-002 [REQ] Require it to kill 100% of a frozen out-of-band mutant corpus targeting its own logic.
- H27-003 [REQ] Require it to satisfy all declared metamorphic relations.
- H27-004 [REQ] Make null_band_sidedness a mandatory relation.
- H27-005 [REQ] Require rejection of every frozen trap.
- H27-006 [REQ] Block gate promotion on any escape.
- H27-007 [REQ] Increment a persisted escaped-mutant counter on escape.
- H27-008 [REQ] Add a non-defaultable null_band_sidedness schema field.
- H27-009 [REQ] Add an AST ban on abs(x)<band when a gate declares a one-sided band.
- H27-010 [REQ] Use only mechanical verification; no LLM verdict.
- H27-011 [REQ] Reuse shipped MutationRunner, RelationLibrary, TrapTask, and lifecycle hook recursively on gates.
- H27-012 [REQ] Keep frozen per-gate mutant corpora.
- H27-013 [REQ] Emit GateSelfValidation with its four-kind discriminator and expose a meta cockpit.
- H27-014 [TEST] A symmetric abs-band implementation declared one-sided cannot become ACTIVE and increments the counter; the correct one-sided implementation kills 100% and promotes.

### H-28 — Current-RF reward-tampering gate + harness-in-the-frozen-set (T-12) [self-mod safety capstone]

Source: H-SUMMARY_INDEX.md:442-456 § H-28. Integration anchors: ssb.rs:30840; promotion_gate.rs:109; darwin_admit_loop.rs:298; self_mod_layer1a.rs:383; reward_tampering.rs; meta/reward_tampering.toml.

- H28-001 [STATE] The shipped blake3 tripwire detects reward-function/harness edits but not manipulation of reward inputs that raises score without changing the hash.
- H28-002 [REQ] Put the evaluator in the frozen gate-manifest set.
- H28-003 [REQ] Put the metric harness in the frozen set.
- H28-004 [REQ] Put gate thresholds in the frozen set.
- H28-005 [REQ] Put the H-17 trial ledger in the frozen set.
- H28-006 [REQ] Hard-block an evaluation-harness blake3 mismatch.
- H28-007 [REQ] Flag/block an implausible score increase after self-modification when the harness hash remains intact.
- H28-008 [REQ] Ensure a deterministic Layer-3a block can never be overridden by a probabilistic Layer-3b green signal.
- H28-009 [REQ] Run before darwin_admit_loop inserts a variant.
- H28-010 [REQ] Lock reward_tampering.toml mutable=0.
- H28-011 [REQ] Emit RewardTampering with its three-kind discriminator and show the self-mod cockpit.
- H28-012 [TEST] Harness edit hard-blocks; intact-hash score jump flags/blocks for Layer 3; a clean configuration-only variant promotes.
- H28-013 [LIMIT] Structural self-modification is undecidable and is refused to human Layer 3.

### H-29 — LOCKED fail-closed drift state machine (T-13)

Source: H-SUMMARY_INDEX.md:457-470 § H-29. Integration anchors: model_drift.rs; canary.rs; conformal_calibration.rs; new drift_interlock.rs; finance/drift_interlock.toml; FSM live-trading edge.

- H29-001 [STATE] G-10's model-drift canary is only one signal; it lacks a 90-day TTL and label-fed concept-drift interlock.
- H29-002 [REQ] Build LOCKED mutable=0 warning -> drift -> halt states.
- H29-003 [REQ] Monitor PSI on inputs.
- H29-004 [REQ] Monitor PSI on outputs.
- H29-005 [REQ] Monitor vendor/schema fingerprint changes.
- H29-006 [REQ] Monitor rolling 14-day live-vs-backtest deviation.
- H29-007 [REQ] Enforce a 90-day model TTL.
- H29-008 [REQ] Re-run H-18 deflated-Sharpe evaluation at TTL/promotion.
- H29-009 [REQ] Gate the FSM transition that resumes or continues live trading.
- H29-010 [REQ] Missing evidence means HALT, not SKIP.
- H29-011 [REQ] Require a full backtest rerun before clearing drift-triggered halt.
- H29-012 [REQ] Route conditional P(Y|X) residual assessment to the conformal Tier-3 advisory.
- H29-013 [REQ] Emit DriftInterlock kinds Warning, Drift, and Halt and expose the deployment cockpit.
- H29-014 [TEST] Schema change, output-PSI breach, or 14-day deviation blocks resume; a clean deployment resumes; 90-day TTL forces H-18.
- H29-015 [LIMIT] The honest guarantee is halt within one realized-label cycle, not never trading a regime break.

## Track 4 — UX + user-memory (instrument first)

Track source preamble: H-SUMMARY_INDEX.md:494-521. Track-order requirement: H-30 measures staleness before H-31/H-32 can be trusted. Citation caveat: arXiv 2605.06527, 2603.23848, and 2512.16962 were future-dated beyond the stated author cutoff and not web-verified during authoring; 2501.13956, 2504.19413, 2505.07509, and 2510.09905 also require execution-time reverification because of ELAI's prior bad-cite history.

### H-30 -- staleness / intent-drift INSTRUMENT (the measure-before-trust keystone)

Source: H-SUMMARY_INDEX.md:522-529 § H-30. Integration anchors: fact.rs:138,167,179; store.rs:208,303; retrieval.rs:46; live_cli.rs:2034,2076; new staleness_runner.rs and retrieval_metrics.rs; bench_cmd.rs:139,149.

- H30-001 [REQ] Build a staleness instrument before enabling trustworthy durable-memory read/write.
- H30-002 [REQ] Measure stale_fact_reached_prompt / total_retrievals at runtime.
- H30-003 [REQ] Emit measurement rows to SSB and make the ratio byte-recomputable under ELAI_SEED.
- H30-004 [REQ] Build a deterministic conflict-resolution-accuracy harness.
- H30-005 [REQ] Its fixture states X, then contradicting Y.
- H30-006 [REQ] Retrieval must act on Y and never surface X.
- H30-007 [REQ] Separately measure recognition_rate, application_rate, and their gap.
- H30-008 [REQ] Reimplement the order-independent fingerprint short circuit using blake3.
- H30-009 [REQ] Reimplement the referenced read-time relevance gate in Rust; do not port Python.
- H30-010 [REQ] Instrument get_relevant_facts as the retrieval choke point before a prompt slice exists.
- H30-011 [REQ] Reuse the same count_retrieval shim when H-32 wires live prompt injection.
- H30-012 [REQ] Define a stale leak as a returned row where valid_to_ms.is_some() or superseded_by.is_some().
- H30-013 [REQ] Build offline Tier-1 StalenessEvalRunner with include_str! fixture corpus and no model/sidecar.
- H30-014 [REQ] Compute StalenessEvalReport fields: conflict_resolution_accuracy, recognition_rate, application_rate, gap, stale_fact_reached_prompt, total_retrievals, seed, corpus_blake3.
- H30-015 [REQ] Add --broken-retrieval; it must increase stale_fact_reached_prompt above zero and lower accuracy.
- H30-016 [REQ] Use offline DeterministicNliJudge; do not spend a local 7B or cloud call in H-30.
- H30-017 [REQ] Ship default-on with no ELAI_*_ENABLE flag.
- H30-018 [REQ] Add elai bench staleness --json and one StalenessEvalMeasured action type.
- H30-019 [STATE] FactStore, supersession, contradiction adjudication, decay, and trust-filter retrieval exist; get_relevant_facts is production-dark and no Memory ContextSlot/prompt call exists.
- H30-020 [DEFER] Live retrieval-to-prompt wiring is H-32/H-31 work, not part of this instrument.
- H30-021 [TEST] Clean fixtures create exactly one computed row with zero leaks; broken retrieval creates leaks and lower accuracy.
- H30-022 [TEST] H-31/H-32 cannot ship until both recognition and application pass on supersession.

### H-31 -- Tier-B user-preference bi-temporal ledger (activate wire-dark crates/memory into a production user-decision path)

Source: H-SUMMARY_INDEX.md:530-538 § H-31. Integration anchors: store.rs:147,208,303; mem0_dispatcher.rs:144; retrieval.rs:46; context_budget.rs:1066; conversation.rs; live_cli.rs:2034,2076.

- H31-001 [REQ] Let users state durable work preferences once and honor them across sessions.
- H31-002 [REQ] On an explicit contradiction, retire the old preference immediately.
- H31-003 [REQ] Never narrate the dead/superseded rule.
- H31-004 [REQ] Never silently auto-learn a preference.
- H31-005 [REQ] Never inject a stale or flattering preference into auditor/critic/adversary contexts.
- H31-006 [STATE] Bi-temporal storage, atomic supersede, soft_close, list_current, contradiction, and decay already exist.
- H31-007 [STATE] dispatch_one supports ADD, UPDATE-via-supersede, DELETE-via-soft_close, and NOOP but is test-only.
- H31-008 [STATE] get_relevant_facts is test-only and SQL already excludes rows where valid_to_ms is not NULL.
- H31-009 [AMB/REQ] Add explicit /remember capture; the summary first calls the test-only target dispatch_batch but its grounded/integration paragraphs name dispatch_one, so the original must settle the exact API.
- H31-010 [REQ] A proposal from repeated corrections requires explicit user confirmation before capture.
- H31-011 [REQ] Add retrieval into prompt assembly through a new ContextSlot::Memory.
- H31-012 [REQ] Keep the Memory slice approximately 100-500 tokens and only retrieve on demand.
- H31-013 [REQ] Use write-time supersession so old.valid_to_ms == new.valid_from_ms.
- H31-014 [REQ] Use HALO-style decay as a backstop.
- H31-015 [REQ] Hard-exclude secrets, PII, and file contents at capture.
- H31-016 [REQ] For a high-stakes flip affecting a hard rule, dependency, or routing, ask Y/N through InteractiveGate.
- H31-017 [REQ] Lower-stakes contradictions may auto-supersede once capture itself was authorized.
- H31-018 [REQ] Default the feature off behind ELAI_USER_MEMORY_ENABLE with a wire-dark allowlist row and frozen production smoke.
- H31-019 [REQ] Emit Pattern-2 UserDecisionSuperseded with UserDecisionEventKind Superseded, Captured, or ProposalSurfaced.
- H31-020 [REQ] The event payload includes only old_fact_id, new_fact_id, fact_type, and key; never the value.
- H31-021 [REQ] Surface facts in MemoryFactsDrawer.
- H31-022 [DEFER] DKG/rerank ranking, scope promotion, periodic tidy, and the armed >50%-deletion guard move to H-32/later work.
- H31-023 [TEST] /remember X then Y causes an actual store row boundary and an off-agent supersession event.
- H31-024 [TEST] Frozen auditor-authored smoke runs the real flag-on retrieval/prompt path, finds X absent, and finds zero memory in auditor-role assembly.
- H31-025 [TEST] Missing ssb.db or zero rows fails rather than skips.

### H-32 -- Tier A code-grounded fact memory (validate-citation-before-use)

Source: H-SUMMARY_INDEX.md:539-547 § H-32. Integration anchors: fact.rs:35-43,138,177; retrieval.rs:17,46,97; cache.rs:276; detect.rs:104; dkg/mod.rs:818; new citation_validity.rs.

- H32-001 [REQ] Store durable code claims only with an evidence pointer to code.
- H32-002 [REQ] Revalidate the citation against current DKG state before use.
- H32-003 [REQ] A renamed/deleted/unresolved symbol prevents silent model steering.
- H32-004 [REQ] On failed/unknown validity, exclude the fact, emit an off-agent row, and ask for confirmation.
- H32-005 [STATE] MemoryFact.evidence_pointers exists and retrieval already excludes empty-evidence rows.
- H32-006 [STATE] Tier A maps to DomainKnowledge/Mistake; Tier B maps to Preference.
- H32-007 [STATE] DKG has ensure_name_index, find_symbol_at, node_by_id, and SymbolNode resolution surfaces.
- H32-008 [REQ] Add last_validated_at_ms as a validity axis separate from last_observed_at_ms recency.
- H32-009 [REQ] Implement revalidate_citation and fail closed on Unknown.
- H32-010 [REQ] Implement write-path classify_fact_tier.
- H32-011 [REQ] Gate get_relevant_facts by validity at read time.
- H32-012 [REQ] Treat Stale as exclude + event + confirmation, not an [OUTDATED] narration to the model.
- H32-013 [REQ] Soft-close a Tier-A fact unused for 28 days.
- H32-014 [REQ] Reset the 28-day timer on a validated use.
- H32-015 [REQ] Never hard-delete stale facts.
- H32-016 [REQ] Keep secrets and PII excluded; citations are symbol pointers only.
- H32-017 [REQ] Keep all Tier-A memory out of auditor/critic contexts.
- H32-018 [REQ] Emit fact_validation_failed and display a per-fact validity badge.
- H32-019 [REQ] Prefer folding validation into H-31's gate; if ELAI_FACT_VALIDATION_ENABLE is introduced it requires allowlist and ON-path smoke.
- H32-020 [DEFER] ONNX fuzzy citation matching and Scope::Global promotion.
- H32-021 [TEST] After rename/delete and DKG rebuild, a real dispatch emits failure, excludes the fact, and requests confirmation.
- H32-022 [TEST] A valid citation injects and stamps last_validated_at_ms; a 28-day-unused one soft-closes.

### H-33 -- memory hub, governance, and provenance surface

Source: H-SUMMARY_INDEX.md:548-556 § H-33. Integration anchors: args_memory.rs:10; args.rs:1310; main.rs:1019-1024; store.rs:128,147,303; decay.rs:88; MemoryFactsDrawer.svelte:83; commands_e5.rs:136,194-195.

- H33-001 [REQ] Let the user view every active and superseded stored decision.
- H33-002 [REQ] Let the user inspect, insert/edit, forget, pin, and unpin decisions.
- H33-003 [REQ] Show which decisions will influence a run before it acts.
- H33-004 [REQ] On /continue, show a resume summary that explicitly flags superseded decisions.
- H33-005 [REQ] Product language is "your saved decisions (editable)", not anthropomorphic "remembers you".
- H33-006 [REQ] Governance/provenance reads FactStore and SSB directly and injects zero model tokens.
- H33-007 [STATE] CLI List/Inspect/Insert/Delete parses but main dispatch is an explicit unrouted stub.
- H33-008 [REQ] Route that stub through a new memory_cmd.rs driver.
- H33-009 [REQ] Map "forget" to soft_close, not deletion.
- H33-010 [REQ] Add Pin/Unpin/Provenance parser variants.
- H33-011 [REQ] Implement pin/unpin without SQLite migration via append-only MemoryGovernanceEvent.
- H33-012 [REQ] Make decay skip pinned fact ids.
- H33-013 [REQ] If governance-beacon reading fails, fail toward decay rather than silently immortalize facts.
- H33-014 [REQ] Compute "which preferences fired" using a read-only recomputation of the trust-floor predicate.
- H33-015 [REQ] Extend the existing F-12 drawer and add Tauri memory_forget, memory_pin, and get_memory_facts_with_history.
- H33-016 [REQ] Emit MemoryGovernanceEvent kinds Forgotten, Pinned, and Unpinned.
- H33-017 [REQ] Keep pin filtering default-off behind ELAI_MEMORY_PIN_ENABLE with 90-day expiry/promotion smoke; read surfaces are default-on.
- H33-018 [REQ] Never inject provenance/resume-summary into auditor/critic prompts.
- H33-019 [DEFER] A standalone MemoryHub panel and vector-ranked provenance.
- H33-020 [TEST] memory list reads active+superseded journal truth with zero prompt tokens.
- H33-021 [TEST] memory forget makes valid_to_ms non-NULL and writes Forgotten.
- H33-022 [TEST] With the flag on, pinned trust_score remains unchanged through decay while a control decays.
- H33-023 [TEST] /continue names influencing and superseded decisions before action.

### H-34 -- MCP defer-loading + Tool-Search emulation for the live STDIO MCP servers

Source: H-SUMMARY_INDEX.md:557-565 § H-34. Integration anchors: plugins/mcp/lazy.rs:30,50,96-98; mcp_bridge.rs:959; mcp_stdio.rs:124,484,493,498,545-552,559; mcp_glue.rs:62,70.

- H34-001 [STATE] Live STDIO McpServerManager eagerly discovers and carries every inputSchema per turn.
- H34-002 [STATE] LazySchemaCache and resolve_schema_on_demand exist only on a production-dark HTTP bridge.
- H34-003 [STATE] The live STDIO insertion loop has no size/defer gate and no per-turn schema-token metric.
- H34-004 [REQ] Measure the eager baseline before changing behavior.
- H34-005 [REQ] Build an offline deterministic mcp_schema_budget_runner with include_str! corpus.
- H34-006 [REQ] Compute baseline tokens, ELAI_SEED, and corpus_blake3; never hardcode expected metric literals.
- H34-007 [REQ] On the live STDIO path, inject the union of an always-available floor, RAG top-K, and keyword force-includes.
- H34-008 [REQ] Cap the always floor at six tools.
- H34-009 [REQ] Cap its schema slice at approximately 500 tokens and assert the cap in the runner.
- H34-010 [REQ] Hold other schemas in Arc<LazySchemaCache>.
- H34-011 [REQ] Resolve a withheld schema on first tool use at the live call_tool path.
- H34-012 [REQ] Use ELAI's ONNX vector stack and blake3; reimplement, do not port, the Odysseus union assembler.
- H34-013 [REQ] Gate with default-off ELAI_MCP_DEFER_SCHEMA_ENABLE and a complete allowlist/ON-path smoke record.
- H34-014 [REQ] Write after-run injected tokens and delta to benchmarks/mcp_schema_budget/<ts>.json.
- H34-015 [REQ] Emit mcp_tool_deferred for each deferred resolve.
- H34-016 [REQ] Emit mcp_schema_budget_measured for the computed budget.
- H34-017 [REQ] Treat server definitions as current; staleness handling is not applicable.
- H34-018 [DEFER] Schema deduplication, cross-server sharing, and default-on promotion.
- H34-019 [TEST] elai bench mcp-schema-budget computes an off-agent baseline.
- H34-020 [TEST] A real --via-mcp invocation names a withheld tool, resolves it, writes a row, keeps the floor, stays under cap, and reports a positive reduction.
- H34-021 [LIMIT] H-34 targets external MCP server schemas; H-4 independently targets the built-in production_agent_rpc catalog.

### H-35 -- /rewind over the SSB journal

Source: H-SUMMARY_INDEX.md:566-573 § H-35. Integration anchors: ssb.rs:15353,23345,23562; replay.rs:84,99; commands_e29.rs:221; session_dag.rs:1507,1543; rra.rs:384,474; compact.rs:96.

- H35-001 [REQ] Add elai rewind <target>.
- H35-002 [REQ] Support --scope conversation, code, or both.
- H35-003 [REQ] Support optional --summarize.
- H35-004 [REQ] Build rewind over deterministic SSB replay under ELAI_SEED.
- H35-005 [REQ] Conversation rewind uses compact_session/trim behavior.
- H35-006 [REQ] Code rewind uses recover_stubs plus fail-closed ssb_resume_from.
- H35-007 [REQ] Rebuild and verify an RRA after restore.
- H35-008 [REQ] Add CLI action/parser, Tauri rewind_session, and CommandWindow affordance.
- H35-009 [REQ] Emit exactly one SessionRewound on an effective rewind.
- H35-010 [REQ] Use target==head as an idempotent no-op with zero rows.
- H35-011 [REQ] Ship default-on with opt-out ELAI_REWIND_DISABLE.
- H35-012 [REQ] Inject no rewind data into a prompt; summarize only shrinks the window.
- H35-013 [STATE] Ascending journal query, ReplaySessionId, locked-key resume, ReplayResume, RRA build/verify, and compact_session are production-wired.
- H35-014 [CORRECTION] replay_resume is not a function in session_dag.rs; that earlier brief reference was wrong.
- H35-015 [DEFER] Branch/fork with both timelines, multi-checkpoint diff, and cloud-model VerifyDivergence restore.
- H35-016 [TEST] A seeded both-scope rewind writes exactly one row with correct sequences and post-rewind RRA verifies.
- H35-017 [TEST] Missing row or non-Verified RRA fails.

### H-36 -- completion / needs-input dev-event notifications

Source: H-SUMMARY_INDEX.md:574-582 § H-36. Integration anchors: task_runner.rs:1157,1162,9610,9613; hook_dispatcher.rs:1146; NotificationManager.svelte:102-118.

- H36-001 [REQ] Notify a developer when a long autonomous run completes.
- H36-002 [REQ] Notify when it blocks awaiting human input.
- H36-003 [REQ] Emit a terminal OSC9 notification.
- H36-004 [REQ] Fall back to terminal BEL.
- H36-005 [REQ] Dispatch a native OS notification using std::process::Command and no new crate.
- H36-006 [REQ] Drive completion from the real dispatch_stop/PhaseComplete path.
- H36-007 [REQ] Drive needs-input from AutonomyDecision::RequestHuman.
- H36-008 [REQ] Emit agent_notification_emitted from the live notifier.
- H36-009 [REQ] Activate the dashboard's explicit intervention-notification no-op by consuming needs_input rows.
- H36-010 [REQ] Use one binary opt-out gate and avoid engagement/vacation-mode/learned-threshold behavior.
- H36-011 [REQ] Keep token cost zero and treat notifications as instantaneous, with no memory staleness/decay.
- H36-012 [REQ] ELAI_NOTIFY_ENABLE is default-off for OS notification, requires allowlist and ON-path smoke; terminal escape plus SSB beacon fire unconditionally.
- H36-013 [STATE] Completion, RequestHuman, SSB writer, stop hook, and completion dashboard toasts are live; intervention dashboard handling is a no-op.
- H36-014 [TEST] Real completion and fix-and-rerun-cap needs-input each create an off-agent row from the production trigger; missing rows fail.

### H-37 -- plan-approval gate at PLANNING -> TESTING_FIRST

Source: H-SUMMARY_INDEX.md:583-591 § H-37. Integration anchors: gate.rs:196,230,733-956; runtime_builder.rs:429; run_subcommand.rs:118-126; autonomous_gate.rs:88; fsm.rs:570,925-928; task_runner.rs:1084-1087,14164.

- H37-001 [REQ] Ask for explicit approve, redirect, or reject before PLANNING/FEASIBILITY_PROBE enters TESTING_FIRST.
- H37-002 [REQ] Use the existing interactive gate protocol.
- H37-003 [REQ] In interactive mode, prompt through the installed CliInteractiveGate.
- H37-004 [REQ] In autonomous mode, use AutonomousGate::decide.
- H37-005 [REQ] Low-risk autonomous decisions may auto-advance; medium/high risk must surface.
- H37-006 [REQ] Use default_risk_level(Planning, TestingFirst)=MEDIUM.
- H37-007 [REQ] Reject keeps the FSM in PLANNING.
- H37-008 [REQ] Redirect validates the requested target before transition.
- H37-009 [REQ] Compute the decision inside the orchestrator at the actual FSM edge, not in a CLI-only shim.
- H37-010 [REQ] Surface only EvidenceSummary and inject no user memory.
- H37-011 [REQ] Emit plan_decision_event with PlanDecisionKind Approve, Redirect, Reject, or AutoAdvance.
- H37-012 [REQ] Mirror the existing GateDecisionPayload shape.
- H37-013 [REQ] Add PlanApprovalCard.
- H37-014 [REQ] Gate with default-off ELAI_PLAN_APPROVAL_GATE_ENABLE and the specified owner/expiry/promotion/ON-path allowlist.
- H37-015 [STATE] run_interactive_gate is test-only, compliance_provider defaults None, and task_runner currently holds AutonomousGate only as a presence signal.
- H37-016 [DEFER] Multi-round MODIFY, learning from rejection, and installing a real ComplianceProvider.
- H37-017 [TEST] Real flag-on transition writes a decision row; Reject is followed by no TESTING_FIRST transition.

### H-38 -- cost-confirmation gate on TaskStop / pod_stop

Source: H-SUMMARY_INDEX.md:592-600 § H-38. Integration anchors: cloud_mode.rs:357,521,646; ssb_viewer.rs:5062; grounding_client.rs:848-905; hook_dispatcher.rs:1146; gate.rs:126,129.

- H38-001 [REQ] Before a billable or unsaved stop, show cost-so-far and unsaved progress.
- H38-002 [REQ] Require user confirmation for that stop.
- H38-003 [REQ] A zero-cost fully saved stop proceeds without friction.
- H38-004 [REQ] Build pure evaluate_stop -> Pass | ConfirmRequired.
- H38-005 [REQ] Read day_usd fresh from cloud_spend_summary.
- H38-006 [REQ] Run at dispatch_stop.
- H38-007 [REQ] On ConfirmRequired, reuse present_gate.
- H38-008 [REQ] Reject means hold.
- H38-009 [REQ] Headless ConfirmRequired fails closed by holding.
- H38-010 [REQ] Emit StopConfirmationEvent kinds ConfirmedProceed, RejectedHeld, or PassedNoFriction.
- H38-011 [REQ] The event's cost remains recomputable from the spend ledger and never enters a model prompt.
- H38-012 [REQ] Extend the orchestration Tier-1 tripwire list for billable pod_stop.
- H38-013 [REQ] Keep safety behavior unconditionally active even if its default-off beacon experiment is disabled.
- H38-014 [REQ] ELAI_STOP_COST_CONFIRM_ENABLE controls the beacon experiment and requires allowlist/ON-path smoke.
- H38-015 [STATE] SpendAccumulator, daily cap, CloudSpendSnapshot, cloud_spend_summary, and the interactive gate seam are production-wired; dispatch_stop has no cost awareness.
- H38-016 [CORRECTION] pod_stop and pod_cost_status are harness MCP tools, not ELAI Rust; the pod-specific half belongs in a skill document.
- H38-017 [DEFER] RunPod billing reconciliation, dashboard modal, and automatic checkpoint.
- H38-018 [TEST] Billable/unsaved stop requires decision and writes its event; nonbillable saved stop writes PassedNoFriction and proceeds.

## Track 6 — Grounded-reasoning hardening

Track source preamble: H-SUMMARY_INDEX.md:620-648. Explicit positioning: this track confirms and extends the shipped sound-checker/cross-family/refusal/orchestration spine rather than redesigning it. Its doctrine is RD-1 abstention-credit, RD-2 decorrelation-over-accuracy, RD-3 sound-checker reach, and RD-4 depth-not-width.

### H-39 -- abstention-aware promotion + eval scoring [TRACK 6 KEYSTONE]

Source: H-SUMMARY_INDEX.md:649-661 § H-39. Anchors: promotion_gate::two_gate_promote; bai::sample; coding_metrics pass_at_k; darwin_admit_loop; D-16 RefusalDecision.

- H39-001 [STATE] Promotion, BAI, coding metrics, and bench scorers are abstention-blind.
- H39-002 [STATE] RefusalDecision and refusal SLA exist but their abstain signal does not feed optimized reward.
- H39-003 [REQ] Define one AbstentionAwareScore.
- H39-004 [REQ] Score correct as +1.
- H39-005 [REQ] Score abstain as 0.
- H39-006 [REQ] Score confident-wrong as -t/(1-t).
- H39-007 [AMB/REQ] Lock t with stated default 0.75; the summary writes "0.75->2x" alongside -t/(1-t), which is arithmetically unclear and must be resolved from H-39 before implementation.
- H39-008 [REQ] Read abstention from shipped RefusalDecision.
- H39-009 [REQ] Feed the score into self-mod promotion reward.
- H39-010 [REQ] Feed it into coding_metrics accumulation.
- H39-011 [REQ] Feed it into bench scorers using the SAFEDELTA/SecCodePlt pattern.
- H39-012 [REQ] Emit AbstentionScored and make it recomputable under ELAI_SEED.
- H39-013 [REQ] Ensure GEPA/BAI/DGM no longer select for confident guessing while inference selects for refusal.
- H39-014 [TEST] A correct-abstaining V2 outranks confident-wrong V1 in both score and promotion.
- H39-015 [TEST] --reward-confident-guessing must invert the fixture result and therefore trigger the broken-arm failure.

### H-40 -- decorrelation-keyed verifier/role retention

Source: H-SUMMARY_INDEX.md:662-671 § H-40. Anchor: E-6 iso_budget_harness insufficient_lift:93 and IsoBudgetReport.

- H40-001 [STATE] Role pruning currently uses marginal lift only.
- H40-002 [REQ] Add per-role unique_catch_rate.
- H40-003 [REQ] Add per-role decorrelation_score.
- H40-004 [REQ] Define insufficient_contribution as low_lift AND low_unique_catch.
- H40-005 [REQ] Retain a low-lift verifier/auditor if it uniquely catches errors.
- H40-006 [REQ] Reuse iso_budget_measured; add no action type.
- H40-007 [TEST] Low-lift/high-unique-catch R remains; low/low C is prunable.
- H40-008 [TEST] --prune-on-lift-only incorrectly prunes both and must fail.
- H40-009 [LIMIT] Pass@K oracle quality is the honest ceiling; decorrelation does not prove verifier correctness.

### H-41 -- live checker-coverage + checker-gap INSTRUMENT

Source: H-SUMMARY_INDEX.md:672-685 § H-41. Integration anchor: grounding_mode.rs:157; agent_pool.rs:2030; CheckerResultEmitted.

- H41-001 [STATE] Sound-checker-first Mode A selection is already shipped.
- H41-002 [STATE] D-21 has an offline F1 benchmark but no live Mode-A fraction or checker-gap backlog.
- H41-003 [CORRECTION] H-41 instruments the existing A > B preference; it does not add that preference.
- H41-004 [REQ] Compute live mode_a_coverage over grounding-mode and CheckerResultEmitted rows.
- H41-005 [REQ] Do not add a separate row merely for the ratio.
- H41-006 [REQ] Emit CheckerGapDetected at the routing site when a claim lacks a sound checker.
- H41-007 [REQ] Include claim_class, complexity_taxon, and fell_to_mode.
- H41-008 [REQ] Classify the gap using the stated Xu Table-3 complexity taxonomy.
- H41-009 [REQ] Route NP-complete/undecidable work to solver or refusal, never LLM grading as a sound check.
- H41-010 [REQ] Add elai checker-gaps.
- H41-011 [REQ] Rank backlog entries by frequency times taxon priority.
- H41-012 [TEST] Checker-backed factual claim selects Constructive with no gap row.
- H41-013 [TEST] Missing checker creates a row and the Mode-A fraction recomputes under ELAI_SEED.
- H41-014 [TEST] --ignore-checker-gaps misreports and must fail.
- H41-015 [LIMIT] Coverage measures reach to a sound checker, not correctness of that checker.

### H-42 -- serial-depth-scaled reasoning budget + the depth-not-width guard

Source: H-SUMMARY_INDEX.md:686-697 § H-42. Anchors: Phase-2C GRC; E-19; agent_grc_config_for_role; decomposition tree; DKG reachability.

- H42-001 [STATE] GRC budget is static per role.
- H42-002 [REQ] Estimate serial depth from decomposition-tree depth and/or DKG reachability-chain length.
- H42-003 [REQ] Scale the base GRC budget as a function of depth.
- H42-004 [REQ] Never scale below the role's base budget.
- H42-005 [REQ] Cap scaling at a LOCKED mutable=0 ceiling.
- H42-006 [REQ] Distinguish DepthBound from AccessBound exhaustion.
- H42-007 [REQ] For DepthBound, recommend more serial steps, not more context.
- H42-008 [REQ] Emit ReasoningBudgetScaled.
- H42-009 [REQ] Keep context-window expansion separate because context buys access, not compute depth.
- H42-010 [TEST] Same-role deep work gets more budget than shallow work and both remain under ceiling.
- H42-011 [TEST] High-depth exhaustion records DepthBound/remedy=steps.
- H42-012 [TEST] --flat-budget equalizes the fixtures and must fail.

## Track 7 — DGM self-improvement verification hardening

Track source preamble: H-SUMMARY_INDEX.md:711-743. The explicit spine is that recursive self-improvement collapses without an ungameable verifier stronger than the generator; the immediate work replaces the live byte-hash fitness and keyword trap placeholders, then composes evaluation and selection upgrades.

### H-43 -- execution-grounded DGM fitness [TRACK 7 KEYSTONE]

Source: H-SUMMARY_INDEX.md:744-754 § H-43. Integration anchors: darwin_admit_loop.rs:186; derived_alpha; CodeRM/CodeT harness; FitnessEvaluated.

- H43-001 [STATE] derived_alpha currently maps blake3(config_blob) to [0,1], so fitness is a function of candidate bytes rather than behavior.
- H43-002 [REQ] Replace the proxy with ExecutionGroundedFitness.
- H43-003 [REQ] Include CodeRM pass rate over a frozen generated-test set.
- H43-004 [REQ] Include CodeT dual agreement.
- H43-005 [REQ] Lock the blend weights.
- H43-006 [REQ] Make derived_alpha call the real fitness while preserving force_alpha and its signature.
- H43-007 [REQ] Apply a cross-family guard to any model-graded component.
- H43-008 [REQ] Emit FitnessEvaluated kinds ExecutionPassRate, DualAgreement, and FitnessRegression.
- H43-009 [REQ] Provide the correct bit consumed by H-39.
- H43-010 [REQ] Make E-19 recombination runnable and leave D-2/E-19/E-53 as upgrade/ceiling composition rather than reauthoring them.
- H43-011 [TEST] Passing C_good scores above failing C_bad, C_bad records regression, and promotion ranks C_good higher.
- H43-012 [TEST] --fitness-from-bytes inverts the fixture and must fail.

### H-44 -- behavioral trap detection + measured recall

Source: H-SUMMARY_INDEX.md:755-765 § H-44. Integration anchors: trap_tasks.rs:123,139,143; H-43 execution harness; frozen TRACE corpus.

- H44-001 [STATE] score_candidate_for_traps is substring matching over keyword arrays and self-identifies as unsound.
- H44-002 [REQ] Replace hard gating with behavioral detect_trap_behavior.
- H44-003 [REQ] Add an ImpossibleBench-style spec/test-conflict canary that stops on flawed tests.
- H44-004 [REQ] Reverify under an isomorphic perturbation.
- H44-005 [REQ] Add CodeHalu execution classification using H-43's harness.
- H44-006 [REQ] Require at least two different-family views.
- H44-007 [REQ] Measure recall on a frozen TRACE corpus.
- H44-008 [REQ] Keep keyword matching only as KeywordFallback advisory.
- H44-009 [REQ] Emit TrapBehaviorDetected.
- H44-010 [TEST] A keyword-free verifier hack is flagged and blocked; a clean case passes; recall is journal-recomputable.
- H44-011 [TEST] --trap-keyword-only misses the hack and must fail.

### H-45 -- worst-case + lineage-diverse promotion scoring + PRMBench tau

Source: H-SUMMARY_INDEX.md:766-775 § H-45. Integration anchors: two_gate_promote; D-5/D-13 verifier views; PromotionScoreAggregated.

- H45-001 [STATE] Promotion currently uses the candidate's own mean alpha and carries an unconfirmed tau forward.
- H45-002 [REQ] Aggregate over lineage-diverse verifier views.
- H45-003 [REQ] Compute min(verdicts) - kappa * variance.
- H45-004 [REQ] Use pretrained-lineage diversity, not merely different random seeds.
- H45-005 [REQ] Block on cross-view disagreement.
- H45-006 [REQ] Calibrate tau on PRMBench.
- H45-007 [REQ] Lock tau with a calibration date.
- H45-008 [REQ] Feed the aggregate into the existing two_gate_promote margin without rewriting the gate.
- H45-009 [REQ] Emit PromotionScoreAggregated.
- H45-010 [REQ] Keep this worst-case dimension orthogonal to H-39 abstention scoring.
- H45-011 [TEST] High-mean/high-variance M blocks while lower-mean/low-variance S admits.
- H45-012 [TEST] --score-own-mean flips the result and must fail.

### H-46 -- HGM clade-metaproductivity parent selection

Source: H-SUMMARY_INDEX.md:776-784 § H-46. Integration anchors: config_archive.rs:49,274; select_parent; H-43 measured deltas.

- H46-001 [STATE] Parent selection uses immediate sigmoid_perf_score.
- H46-002 [REQ] Compute clade_metaproductivity from aggregated measured execution deltas of descendants.
- H46-003 [REQ] Select parents by descendant-clade productivity.
- H46-004 [REQ] Retain immediate sigmoid score only when no lineage exists.
- H46-005 [REQ] Label that case ImmediateScoreFallback.
- H46-006 [REQ] Emit CladeSelected.
- H46-007 [TEST] Mediocre-but-improving B is selected over high-immediate/regressing A; a fresh leaf uses fallback.
- H46-008 [TEST] --select-immediate-score expands the dead end and must fail.
- H46-009 [LIMIT] HGM 56.7 vs DGM 53.3 and 2.38x compute are paper figures, not ELAI measurements.

### H-47 -- DGM eval firewall (date-gate + template-regen + holdout-budget binding)

Source: H-SUMMARY_INDEX.md:785-796 § H-47. Integration anchors: D-6 date tags; E-54 corpus; H-19 HoldoutBudgetEvent; EvalFirewallEvent.

- H47-001 [REQ] Tie evaluation-item dates to the Darwin generation clock.
- H47-002 [REQ] Reject pre-update items with PreUpdateItemRejected.
- H47-003 [REQ] Score eligible post-update items.
- H47-004 [REQ] Regenerate symbolic/template instances at score time.
- H47-005 [REQ] Bind every loop evaluation query to H-19's Thresholdout DP budget.
- H47-006 [REQ] Refuse after HoldoutBudgetEvent Exhausted.
- H47-007 [REQ] Add an evaluation-awareness IndistinguishabilityProbe.
- H47-008 [REQ] Emit EvalFirewallEvent while reusing H-19 budget events.
- H47-009 [REQ] Compose D-6/E-54/H-19 rather than restating their stores/budgets.
- H47-010 [TEST] Pre-update P rejects, post-update Q scores, template T regenerates, and budget exhaustion refuses.
- H47-011 [TEST] --eval-ignore-dates scores P and inflates fitness, proving the broken arm.
- H47-012 [LIMIT] A strong self-improver may still recognize/sandbag an eval; the firewall mitigates but does not cure it.

### H-48 -- wire the BAI bandit on the real reward

Source: H-SUMMARY_INDEX.md:797-806 § H-48. Integration anchors: bai.rs:286; D-29 hierarchical BAI; darwinian_bai loop; H-45 reward.

- H48-001 [STATE] BAI structs are shipped but darwinian_bai sample/update/run remains a scaffold on byte-hash reward.
- H48-002 [REQ] Activate nonstationary Discounted Thompson Sampling.
- H48-003 [REQ] Activate Successive Rejects selection.
- H48-004 [REQ] Feed H-45's aggregated execution reward, not bytes.
- H48-005 [REQ] Seed arm and guard-task curriculum from ExIt partial histories.
- H48-006 [REQ] Reuse D-5/D-29 bandit machinery and telemetry.
- H48-007 [REQ] Add BanditArmSampled only if there is no usable emitter.
- H48-008 [TEST] Fixed-budget selection converges to high-execution-fitness arm G.
- H48-009 [TEST] --bandit-reward-from-bytes converges to failing B and must fail.
- H48-010 [DEFER] Live multi-arm regret measurement remains RunPod-pending Tier 2.

## Track 8 — Reversibility / time-travel

### H-49 — reversible LLM edits: undo/redo/branching file time-travel + lossless session history

Source: H-SUMMARY_INDEX.md:824-858 § H-49. Integration anchors: file_ops.rs:272,300; edit_checkpoints.rs; session_dag.rs:213; ssb.rs:10644; new architecture/timetravel.rs; timetravel_cmd.rs; commands_h49.rs; BranchTree/TranscriptDrawer.

- H49-001 [REQ] Make every LLM-authored file edit reversible.
- H49-002 [REQ] Make reverted edits re-applicable through redo.
- H49-003 [REQ] Permit navigation at any history node without destroying another timeline.
- H49-004 [REQ] Provide lossless paged transcript access so the start of a long session remains reachable.
- H49-005 [REQ] Use a content-addressed object store with blake3 identifiers.
- H49-006 [REQ] Compress objects with zstd.
- H49-007 [REQ] Deduplicate identical snapshots.
- H49-008 [REQ] Wrap/activate the shipped edit_checkpoints surface rather than replace it wholesale.
- H49-009 [REQ] Maintain an undo-tree HEAD over SessionDag.
- H49-010 [REQ] Undo moves to the parent.
- H49-011 [REQ] Redo follows a chosen forward edge.
- H49-012 [REQ] Editing after undo creates a sibling branch and preserves the former redo branch.
- H49-013 [REQ] Apply RV-1 divergence guard: snapshot current manual changes before any restore so user work is never clobbered.
- H49-014 [REQ] Offer edit, turn, and babystep checkpoint grains.
- H49-015 [REQ] Default to CAS restoration.
- H49-016 [REQ] Permit shadow-git as an optional RestoreEngine backend.
- H49-017 [REQ] Use snapshot restoration rather than model replay, so cloud/model changes do not affect byte restore.
- H49-018 [STATE] file_ops already captures a preimage and discards it; edit_checkpoints is off/path-heuristic/full-copy; SessionDag already has parent_id.
- H49-019 [STATE] FileWritePayload has path/size/hash but no emitter; no redo exists; TranscriptDrawer is capped at 50; handoff drops old messages; recover_stubs is production-dark.
- H49-020 [REQ] Add FileTree manifest and head columns using lazy ALTER.
- H49-021 [REQ] Capture file_ops.original_file at the FileWrite-row build point.
- H49-022 [REQ] Add CLI undo, redo, history, and jump.
- H49-023 [REQ] Add Tauri and CommandWindow/BranchTree/TranscriptDrawer frontends.
- H49-024 [REQ] Reuse SessionRewound for undo.
- H49-025 [REQ] Add EditHistoryEvent kinds SnapshotCaptured, Deduped, Redo, Branched, Jumped, and TooLargeSkipped.
- H49-026 [REQ] Ship default-on with ELAI_TIMETRAVEL_DISABLE opt-out.
- H49-027 [REQ] Add RV-1..RV-4 doctrine, including irreversible-action warning composition with saga compensation.
- H49-028 [TEST] Real agent edit creates off-agent SnapshotCaptured and a blake3-verified preimage blob.
- H49-029 [TEST] undo --scope code restores pre-hash exactly and writes SessionRewound.
- H49-030 [TEST] redo restores post-hash exactly and writes Redo.
- H49-031 [TEST] edit-after-undo yields two children and a Branched event; the old forward line remains.
- H49-032 [TEST] hand-edit-then-undo first creates an auto:pre-restore fork.
- H49-033 [TEST] No-op HEAD==target writes zero rows.
- H49-034 [REQ] Bound divergence-guard false-fork rate on at least three runs before default-on.

## Track 9 — Context lifecycle

### H-50 — regime-aware in-session context management (local unload/load vs cloud cache-stable; one window truth)

Source: H-SUMMARY_INDEX.md:862-884 § H-50. Integration anchors: conversation.rs:22,789; cli_consts.rs:20; live_cli.rs:1238,1317; session_dag.rs:1551; providers/mod.rs:257; anthropic.rs:1094.

- H50-001 [REQ] Select context policy by runtime regime because local and cloud cost models differ.
- H50-002 [REQ] Keep the cloud policy cache-stable and preserve H-1/H-2 markers/prefix.
- H50-003 [REQ] Minimize active local context because the stated local HF-transformers path has no cross-request KV reuse.
- H50-004 [REQ] Introduce effective_context_window(backend) as the single window truth.
- H50-005 [REQ] Respect ELAI_LOCAL_MAX_CONTEXT.
- H50-006 [REQ] Replace conversation's flat 100K auto-compact threshold.
- H50-007 [REQ] Replace the 64K non-Opus window assumption used by the 85% gate.
- H50-008 [REQ] Route compaction ladder, adaptive_reduce, compact_session, and H-6 masking into the interactive local turn.
- H50-009 [REQ] Make recover_stubs selective so trimming is not immediately reinflated.
- H50-010 [REQ] Preserve a lossless reload path through SessionDag/H-49.
- H50-011 [REQ] Add a per-regime context gauge.
- H50-012 [REQ] Show "unloaded N (reloadable)".
- H50-013 [REQ] Reuse ContextReduction; WindowPolicyApplied is optional.
- H50-014 [REQ] Ship default-on with ELAI_CONTEXT_POLICY_DISABLE opt-out.
- H50-015 [STATE] Current flat 100K never fires for 32K local; current 85% threshold fires around 54K; recovery reinflates; adaptive_reduce has no production caller; hard context_window_blocked is the live backstop.
- H50-016 [CORRECTION] Cloud cache markers are already live at all three cited anthropic.rs sites; the H-1 split-brain premise is refuted for that later snapshot and must be reconciled by detail/live inspection.
- H50-017 [REQ] Add CL-1..CL-4 doctrine, including honest cache claims by regime.
- H50-018 [TEST] A 32K local seeded session past trim writes ContextReduction and next-request tokens fall below both window and pre-trim count.
- H50-019 [TEST] Disabled control hits context_window_blocked while enabled run completes.
- H50-020 [TEST] A "caching on" claim fails if journal cache_read is zero; local streaming is explicitly zero in the summary.
- H50-021 [REQ] Bound false-drop rate on at least three runs.

### H-51 — cross-session sleep-time handoff: grounded topic-digest + auto-detect + tiered load

Source: H-SUMMARY_INDEX.md:885-911 § H-51. Integration anchors: .claude/rules/handoff-protocol.md; repl_command.rs:97; handoff.rs:403; session_dag.rs:909-912,1507; main.rs:712; startup.rs:272; compact.rs:96; ssb.rs:15353.

- H51-001 [STATE] Cross-session resume is manual /continue and loads a packed transcript of roughly 20-27K local tokens.
- H51-002 [STATE] A fresh REPL starts empty; automatic startup resume exists only for indexing.
- H51-003 [STATE] The handoff protocol already specifies a roughly 40-100-line topic-keyed digest but is only prose.
- H51-004 [REQ] Author a sleep-time handoff digest from the SSB journal through compact_session.
- H51-005 [REQ] Write it to workflow/handoffs/{slug}.md.
- H51-006 [REQ] Include topic-keyed decisions and files grounded in intervention/FileWrite rows, not model recollection.
- H51-007 [REQ] Maintain an index from slug to digest, DAG tip, and FSM cursor.
- H51-008 [REQ] Detect matching handoff state at fresh REPL startup.
- H51-009 [REQ] Offer resume before the first prompt.
- H51-010 [REQ] On user confirmation, load digest first.
- H51-011 [REQ] Keep full detail lazy and pullable via selective recover_stubs.
- H51-012 [REQ] Repoint /continue to the tiered load.
- H51-013 [REQ] Add CommandWindow resume offer and Tauri commands_h51.rs.
- H51-014 [REQ] Emit HandoffLifecycleEvent kinds DigestWritten, ResumeOffered, ResumeConfirmed, and TieredLoad.
- H51-015 [REQ] Default detection on.
- H51-016 [REQ] Default automatic continuation off.
- H51-017 [REQ] Provide ELAI_HANDOFF_AUTORESUME_DISABLE kill switch.
- H51-018 [REQ] Compose staleness, validate-before-use, and the auditor firewall from H-30/H-31/H-32.
- H51-019 [TEST] Ending topic T writes DigestWritten and a digest whose decisions/files match SSB.
- H51-020 [TEST] Fresh same-project ELAI shows ResumeOffered.
- H51-021 [TEST] Confirmed tiered load records at most approximately 3K loaded tokens and can still recover early detail.
- H51-022 [TEST] No open session yields no offer and zero rows.
- H51-023 [REQ] Make resume token cost recomputable from the journal.

## Track 10 — Review anchoring

### H-52 — deterministic snippet->hunk finding-anchoring + line-less finding schema (the reviewer cockpit's missing primitive)

Source: H-SUMMARY_INDEX.md:912-955, especially :925-954 § H-52. Integration anchors: external open-code-review resolver.go; G-5 score_review_surface/HunkVouch; C-7 ChangeDecisionPayload.hunk_id; new review_anchor.rs; existing elai review --cockpit.

- H52-001 [REQ] Make the model emit a verbatim code snippet and never a line number for a review finding.
- H52-002 [REQ] Normalize the snippet deterministically.
- H52-003 [REQ] Match a consecutive normalized run against the hunk NEW side.
- H52-004 [REQ] Then match against the hunk OLD side.
- H52-005 [REQ] Fall back to a whole-new-file scan.
- H52-006 [REQ] Use a line-less ReviewFinding schema so hallucinated line numbers are structurally impossible.
- H52-007 [REQ] If no anchor exists, fall back to a file-level summary rather than guess an inline position.
- H52-008 [REQ] Add a duplicate-snippet Ambiguous guard absent from the external source.
- H52-009 [REQ] Add a hunk-proximity tie-breaker rather than first-occurrence-wins.
- H52-010 [REQ] Reuse ELAI's existing diff parser.
- H52-011 [REQ] Return AnchorResult with optional hunk_id/line_range and AnchorKind Anchored, FileFallback, Ambiguous, Unanchorable, or Relocated.
- H52-012 [REQ] Join findings to C-7 ChangeDecision at hunk_id grain.
- H52-013 [REQ] Additively feed G-5 score_review_surface/HunkVouch so evidence cannot vouch the wrong hunk.
- H52-014 [REQ] Emit FindingAnchored.
- H52-015 [REQ] Surface through existing review --cockpit and CockpitWindow.
- H52-016 [REQ] Ship default-on with ELAI_REVIEW_ANCHOR_DISABLE.
- H52-017 [REQ] Recompute anchors byte-identically from snippet plus diff.
- H52-018 [DEFER] An LLM relocation retry is Tier 2.
- H52-019 [LIMIT] "Smart bundling" and "reflection module" are grep-verified external vaporware and excluded.
- H52-020 [TEST] Unique occurrence yields Anchored, inline rendering, and hunk join.
- H52-021 [TEST] Duplicate yields Ambiguous plus file fallback, never guessed line.
- H52-022 [TEST] Absent snippet yields Unanchorable plus file fallback.
- H52-023 [TEST] --trust-model-line-numbers misanchors the duplicate and must fail.
- H52-024 [TEST] No findings writes zero rows.
- H52-025 [REQ] Bound over-fallback rate on at least three runs.

## Track 11 — External-system research steals

### H-53 — citation pre-binding + constrained synthesis on the answer path (ghost citations made structurally impossible)

Source: H-SUMMARY_INDEX.md:969-994 § H-53. Integration anchors: E-48 citation_verified; H-52 resolver; D-16/D-17; 3E/H-50 Evidence slot; new citation_prebind.rs; /research, /ask, researcher-oracle.

- H53-001 [STATE] Current E-48/Phase-2B behavior detects free-form ghost citations only after answer generation.
- H53-002 [REQ] Add a first-class pre-generation citation assignment stage.
- H53-003 [REQ] Freeze ranked evidence as EvidenceAnchorSet.
- H53-004 [REQ] Assign rank-stable anchor ids under ELAI_SEED.
- H53-005 [REQ] Pin each excerpt with blake3.
- H53-006 [REQ] Inject anchors after the last cache breakpoint so the stable prefix is not busted.
- H53-007 [REQ] Constrain synthesis semantically to cite the frozen set while generating.
- H53-008 [REQ] Check every [N] by closed-set membership and support against EvidenceAnchorSet.
- H53-009 [REQ] Reuse/extend E-48 citation_verified with anchor_id and pre_bound when sufficient.
- H53-010 [REQ] Add CitationAnchorsBound only if a distinct prebinding row is actually necessary.
- H53-011 [REQ] Reuse H-52 verbatim-to-anchor resolution.
- H53-012 [REQ] Hold any out-of-set citation.
- H53-013 [REQ] Route an ungroundable claim to D-16 refusal rather than inventing a citation.
- H53-014 [REQ] Run on /research, /ask, researcher-oracle, and E-48 answer paths before synthesis.
- H53-015 [REQ] Budget the Evidence slot by backend with 3E/H-50.
- H53-016 [REQ] Publish only ELAI's measured ghost-citation rate and refusal band, journal-recomputable under ELAI_SEED.
- H53-017 [REQ] Ship default-on with ELAI_CITATION_PREBIND_DISABLE.
- H53-018 [DEFER] E-35/C-26 constrained decoding grammar for mechanically un-emittable out-of-set markers.
- H53-019 [DEFER] Final local-window coordination with H-50 is Tier 2.
- H53-020 [LIMIT] Perplexity's precise mechanism is analyst-reported and unconfirmed; it is medium grounding and supplies shape, not guarantee.
- H53-021 [LIMIT] Prebinding secures citation-to-evidence linkage, not truth of the evidence; novel claims still need D-15/human review.
- H53-022 [TEST] Answerable fixture resolves every [N] with pre_bound=true.
- H53-023 [TEST] Out-of-set citation blocks and unanswerable subclaim refuses.
- H53-024 [TEST] Numbering is byte-identical under seed.
- H53-025 [TEST] --allow-freeform-citations permits a ghost and must fail.
- H53-026 [REQ] Bound over-refusal on at least three runs and cross-check D-17.

### H-54 — deterministic obligation-closure escape-hatch gate (the relocated-sorry / hallucinated-known-result defense)

Source: H-SUMMARY_INDEX.md:995-1026 § H-54. Integration anchors: H-44 TrapBehaviorDetected; E-53 verifier obligation surfaces; new verifier/closure_scan.rs; obligation-closure-allowlist.toml; gate_freeze_check.py.

- H54-001 [REQ] Traverse the full verifier-exposed obligation dependency DAG using BFS.
- H54-002 [REQ] Inspect every transitive obligation, not just the top statement.
- H54-003 [REQ] Block any unregistered proof escape hatch anywhere in the closure.
- H54-004 [REQ] Detect Verus assume and external_body.
- H54-005 [REQ] Detect Kani vacuous assume.
- H54-006 [REQ] Detect Dafny assume and axiom.
- H54-007 [REQ] Detect a Z3 assertion that merely restates the goal.
- H54-008 [REQ] Detect Lean sorry, admit, and axiom where that lane is used.
- H54-009 [REQ] Use a per-verifier HatchKind map.
- H54-010 [REQ] Allow only declared-open obligations present in a registered, R8-frozen allowlist.
- H54-011 [REQ] Record a registered exception rather than pretending it is proved.
- H54-012 [REQ] Fingerprint the closure deterministically.
- H54-013 [REQ] Reuse each E-53 verifier's obligation exposure.
- H54-014 [REQ] Run this as the cheapest first leg of E-53 and H-44.
- H54-015 [REQ] Extend TrapBehaviorDetected with ClosureEscapeHatch where H-44 is available.
- H54-016 [REQ] Add ObligationClosureScanned only if H-44 is not shipped/insufficient.
- H54-017 [REQ] Keep H-44 behavioral checks for nonformal cases that have no proof DAG.
- H54-018 [REQ] Demote H-44 keyword matching to advisory fallback.
- H54-019 [REQ] Freeze obligation-closure-allowlist.toml through gate_freeze_check.py.
- H54-020 [REQ] Ship default-on with ELAI_CLOSURE_SCAN_DISABLE.
- H54-021 [DEFER] A cross-verifier unified DAG abstraction is Tier 2.
- H54-022 [TEST] Clean closure returns ClosureClean and continues.
- H54-023 [TEST] A top-level passing proof with a renamed admitted helper returns EscapeHatchAnywhere with obligation id and blocks.
- H54-024 [TEST] Declared-open plus registered row returns RegisteredException without blocking.
- H54-025 [TEST] The same hole without a row blocks.
- H54-026 [TEST] --top-statement-only passes the relocated hole and must fail.
- H54-027 [REQ] Bound false blocks on at least three runs.
- H54-028 [LIMIT] This proves no hidden admitted hole, not statement conformance.
- H54-029 [LIMIT] If the verifier exposes only a boolean, return ClosureUnavailable and require a runtime backstop.
- H54-030 [LIMIT] Formal proof remains a narrow CSQ-Critical ceiling with the cited approximately 2.5-9% pass band; reuse the scan, not routine Lean proof search.

## Explicit interaction and lifecycle map

No arrows in this section are inferred; each is stated by COMPOSES-WITH, depends, prerequisite, follow-on, or sequencing language in the summary.

1. **Cloud efficiency/accounting**
   - H-1 puts markers and live hit-rate observation on the actual RuntimeAdapter path.
   - H-2 permits marker placement only after byte-prefix stability and adds the one-hour/tail-breakpoint policy.
   - H-9 activates CloudSpend and exposes hit rate, token classes, expiry, and dollars/task, making H-1/H-2 falsifiable.
   - H-3 independently reduces the five-times-priced output side through diff/FIM and early termination; its savings cannot be multiplied with cache savings.
   - H-4 cuts fixed built-in-tool schema cost, while H-34 separately cuts external STDIO MCP schema cost.
   - H-5 batches only noninteractive work; H-7 assigns roles to appropriate runtimes; H-8 persists only idempotent grounding results.

2. **Canvas delivery sequence**
   - H-10's measured renderer B0 and persistent PageRank are the thesis slice.
   - H-11 adds live activity, filesystem access, and explorer/focus synchronization.
   - H-12 ports plan/research and makes Canvas the single SSB consumer/default.
   - H-13 supplies model-callable operations, dispatcher, grammar, screenshot, and architecture rendering.
   - H-14 consumes H-13 operations for web design/writeback.
   - H-15 depends on H-13 plus G-1 taint and G-2 RRA to make spawn_view reproducible and safe at selection handoff.
   - H-16 is last and may remove orbit only after one regression-free Canvas-default release.

3. **Autonomous-trading promotion and deployment**
   - Every candidate evaluation first registers in H-17 so effective N cannot be erased.
   - H-18 reads that N for DSR/PBO/MinBTL; H-22 also uses N for its permutation-null DSR.
   - H-19 limits adaptive holdout reuse; H-47 later binds DGM evaluation to the same budget.
   - H-20 blocks recognizable static leakage; H-21 is the runtime causality backstop.
   - H-22 perturbs the claimed edge; H-23 reruns declared outputs; H-24 bans split live/backtest paths and compares replay/golden.
   - H-25 binds reported empirical numbers to output/data/code/environment/seed/N/DSR/PBO.
   - H-26 compares implementation behavior and financial invariants.
   - H-27 recursively tests every gate before ACTIVE.
   - H-28 freezes the evaluator/harness/thresholds/ledger and blocks reward tampering before Darwin admission.
   - H-29 gates live-trading resume/continuation and forces H-18 again at the 90-day TTL.

4. **Memory write/read/governance**
   - H-30 must prove the staleness metric and supersession application before H-31/H-32 ship.
   - H-31 activates explicit Tier-B capture, bi-temporal supersession, and role-filtered on-demand prompt retrieval.
   - H-32 classifies Tier A and adds validate-citation-before-use plus 28-day unused TTL.
   - H-33 exposes both tiers without model injection and governs forget/pin/provenance/resume.
   - H-51 uses H-30/H-31/H-32 staleness and auditor-firewall rules when it builds a cross-session digest.

5. **Rewind, time travel, and context**
   - H-6 activates reduction, stub recovery, and memory retrieval.
   - H-35 provides journal rewind and explicitly defers branching/redo/multi-checkpoint behavior.
   - H-49 is the follow-on that delivers those file-history items, preserves both branches, and adds a lossless transcript.
   - H-50 makes reduction/recovery selective by runtime regime, preserving the cloud prefix while minimizing the local window.
   - H-51 builds digest-first cross-session resume on H-50 selective recovery and H-35/H-49 SessionDag/RRA.

6. **Grounded-reasoning and DGM reward**
   - H-39 defines abstention-aware scoring; H-43 supplies the execution-grounded correct/fail signal it needs.
   - H-40 prevents deletion of decorrelated verifier roles.
   - H-41 measures access to sound checkers and creates a backlog; it does not change the already-shipped router preference.
   - H-42 scales serial reasoning compute while leaving context/access policy to H-50.
   - H-43 replaces byte-hash fitness.
   - H-45 aggregates execution verdicts across lineages; H-48 drives the existing bandit using that aggregate.
   - H-44 replaces keyword traps behaviorally; H-54 supplies its deterministic formal-proof closure leg.
   - H-47 protects H-43's evaluation signal through dates, regeneration, H-19 budgeting, and awareness probing.
   - H-46 chooses productive clades using H-43 deltas.

7. **Review, citations, and proof obligations**
   - H-52 joins a model-supplied verbatim snippet to C-7 hunk ids and supplies G-5/E-41 a deterministic anchor.
   - H-53 reuses that resolver and E-48 but moves citation assignment before answer generation.
   - H-54 reuses E-53 obligation exposure and H-44 telemetry to catch hidden admitted dependencies before accepting a proof.

## Dependency, order, grounding, and delivery status

This is the summary's plan status, not a fresh implementation audit.

| ID | Grounding / nature / effort | Explicit sequencing or dependency | Summary-stated delivery state |
|---|---|---|---|
| H-1 | STRONG; ACTIVATION; M | Wave 1; feeds H-9 | Main path cached; RuntimeAdapter and monitor dark |
| H-2 | STRONG; BUILD; M | Wave 1; H-9 instrument | Breakpoint allocator exists but is inert/local-only |
| H-3 | STRONG; BUILD + one-line activation; L | Wave 1; reuses B-20/B-28 | DIFF/FIM/validator production path dark |
| H-4 | STRONG; BUILD; M | Wave 1; distinct from H-34 | Header/search/dkg dispatch gaps |
| H-5 | STRONG; BUILD; M | Wave 1; noninteractive only | Batch transport absent |
| H-6 | STRONG; ACTIVATION; M | Wave 1; shared by H-49/H-50/H-51 | Three symbols exist but production-dark |
| H-7 | STRONG; BUILD; L | Wave 1; >=3-run over-escalation gate | Per-role router present but unwired/observational |
| H-8 | STRONG; persistence BUILD; S | Wave 1 | Oracle cache live but process-only |
| H-9 | STRONG; ACTIVATE + BUILD; M | Wave 1; instrument for H-1/H-2/H-38 | Ledger/UI feeder end-to-end dark in early snapshot |
| H-10 | STRONG; BUILD; L | Wave 2; BLOCKING B0 | Partial graph view exists; PageRank producer absent |
| H-11 | STRONG; BUILD; M | Wave 2; uses H-10 graph/focus | FS surface absent; project commands UI-dark |
| H-12 | STRONG; port BUILD; M | Wave 2; prerequisite for H-16 | Orbit modes exist; flat/default/single-consumer work planned |
| H-13 | STRONG; BUILD; L | Wave 2; prerequisite H-15; feeds H-14 | Model-op transport/dispatcher absent |
| H-14 | MODERATE; net-new BUILD; XL | Wave 2; B0 proxy + recast spike | Riskiest/new; RN/Expo deferred |
| H-15 | STRONG; BUILD; L | Wave 2; depends H-13, G-1, G-2 | New reply/state/taint integration |
| H-16 | STRONG; DELETE; S | Wave 2 LAST; after H-12 full release | Cleanup only after regression gate |
| H-17 | STRONG; BUILD; S-M | Wave 3 BLOCKING | Ledger/register API absent |
| H-18 | STRONG; BUILD; M | Wave 3; depends H-17 | Finance deflation evaluator absent |
| H-19 | STRONG; BUILD; M | Wave 3; depends H-17; reused H-47 | Holdout budget absent |
| H-20 | STRONG; BUILD on taint; M-L | Wave 3; G-1 or scoped-pass prerequisite | Taint primitive partial; exact dataflow choice open |
| H-21 | STRONG; tiered BUILD; M | Wave 3; runtime backstop H-20 | Differential absent |
| H-22 | STRONG; tiered BUILD; L | Wave 3; depends H-17; validated H-27 | Trading relations absent |
| H-23 | STRONG; BUILD on RRA; M | Wave 3 | RRA shipped; metric equality gate absent |
| H-24 | STRONG; BUILD; M | Wave 3; A+C now, B later | Static/golden parts planned; numeric parity deferred |
| H-25 | STRONG; BUILD extending F-4; M | Wave 3; binds H-17/H-18 | Result provenance binding absent |
| H-26 | STRONG; tiered BUILD; M-L | Wave 3 | Shadow/invariant pack absent |
| H-27 | STRONG; BUILD on shipped test libs; L | Wave 3 capstone | Recursive verifier validation absent |
| H-28 | STRONG; BUILD/reuse; S | Wave 3 capstone; before variant insert | RF hash tripwire exists; input-tamper gate absent |
| H-29 | STRONG; BUILD; M | Wave 3 LAST; depends H-18 | Signals partial; full interlock absent |
| H-30 | STRONG; BUILD; M | Wave 4 BLOCKING H-31/H-32 | Memory substrate exists; instrument absent |
| H-31 | STRONG; ACTIVATION; M-L | Wave 4; depends H-30 | Storage/live reconciliation exists; capture/read wires dark |
| H-32 | STRONG; BUILD on DKG; M | Wave 4; depends H-31 | Citation carrier/DKG exist; validity gate absent |
| H-33 | STRONG; ACTIVATION; M | Wave 4; depends H-31 | CLI parsed but unrouted; drawer read-only |
| H-34 | STRONG; BUILD; M | Wave 4; no H dependency | Lazy HTTP path exists; live STDIO is eager |
| H-35 | STRONG; UI-over-journal BUILD; M | Wave 4; depends H-33, G-2 | Restore primitives live; affordance absent |
| H-36 | MODERATE; BUILD; S | Wave 4; depends H-35 | Completion toast partial; terminal/OS/needs-input absent |
| H-37 | STRONG; BUILD on pre-gate; S-M | Wave 4; soft-depends H-36 | Gate seams exist but runtime-dark |
| H-38 | MODERATE; BUILD; S | Wave 4; depends H-9; composes H-37 | Spend/gate seams live; stop boundary unaware |
| H-39 | STRONG; BUILD; M | Wave 6 BLOCKING reward | Reward is abstention-blind |
| H-40 | STRONG; EXTENSION; S-M | Wave 6 | Iso-budget exists; unique-catch axis absent |
| H-41 | STRONG; instrument BUILD; M | Wave 6 | Router preference shipped; live reach/backlog absent |
| H-42 | STRONG; EXTENSION; M | Wave 6 | Per-role GRC exists; depth scaling absent |
| H-43 | STRONG; BUILD; M-L | Wave 7 BLOCKING reward; used H-39/H-45/H-46/H-47/H-48 | Live derived_alpha is byte hash |
| H-44 | STRONG; BUILD; M-L | Wave 7 parallel H-43; extended H-54 | Live trap score is keyword grep |
| H-45 | STRONG; promotion extension; M | Wave 7; depends H-43; feeds H-48 | Own-mean/unconfirmed tau |
| H-46 | STRONG; selection extension; M | Wave 7; depends H-43 | Immediate-score parent selection |
| H-47 | STRONG; composition BUILD; M | Wave 7; depends H-43, composes H-19 | Eval firewall absent |
| H-48 | STRONG; ACTIVATION; M | Wave 7 LAST; depends H-43/H-45 | Bandit scaffold exists; real-reward loop dark |
| H-49 | STRONG; BUILD on shipped; L | Wave 8; depends H-35/E-25/H-6 | About 70% primitives fragmented/dark; no redo |
| H-50 | STRONG; BUILD on shipped; M-L | Wave 9; composes H-1/H-2/H-6/H-35/H-49 | One wrong cloud-shaped policy; later cloud split-brain correction |
| H-51 | STRONG; BUILD on shipped; M | Wave 9; depends H-50/H-6 | Manual transcript-heavy resume; prose protocol unwired |
| H-52 | STRONG; BUILD; M | Wave 10; depends G-5/C-7 | Deterministic anchor primitive absent |
| H-53 | STRONG on ELAI, external mechanism MEDIUM; BUILD; M-L | Wave 11; depends E-48/E-35/C-26; composes H-52 | Answer path is detect-after |
| H-54 | STRONG; BUILD; M | Wave 11; depends E-53; composes H-44/E-53 | Full dependency-closure escape scan absent |

## Configuration and policy switches

Only switches explicitly named by the summary are included.

| Plan | Switch/default | Required behavior |
|---|---|---|
| H-1 | No env override; default-on | Production cache markers and measurement cannot terminate dark |
| H-30 | No enable flag; default-on | Offline eval always available |
| H-31 | ELAI_USER_MEMORY_ENABLE; default-off initially | Allowlist, expiry/promotion metadata, and real prompt-path smoke |
| H-32 | Prefer no new flag | If ELAI_FACT_VALIDATION_ENABLE is added, it needs allowlist + ON-path smoke |
| H-33 | ELAI_MEMORY_PIN_ENABLE; default-off initially | Pin filtering only; read governance default-on |
| H-34 | ELAI_MCP_DEFER_SCHEMA_ENABLE; default-off initially | Allowlist + withheld-tool production smoke; later default-on promotion |
| H-35 | ELAI_REWIND_DISABLE; opt-out | Rewind default-on; no allowlist row needed |
| H-36 | ELAI_NOTIFY_ENABLE; default-off for OS delivery | Terminal escape and SSB evidence remain unconditional |
| H-37 | ELAI_PLAN_APPROVAL_GATE_ENABLE; default-off initially | Owner H-37, expiry 2026-09-04, promotion record, ON-path smoke |
| H-38 | ELAI_STOP_COST_CONFIRM_ENABLE; default-off beacon | Stop safety itself remains on and cannot fail open |
| H-43..H-48 | cfg(feature="darwinian-evolve") and ELAI_DARWINIAN_LOOP_ENABLE | Existing allowlisted kill switch applies to every Track-7 item |
| H-49 | ELAI_TIMETRAVEL_DISABLE; opt-out | Time travel default-on after false-fork bound |
| H-50 | ELAI_CONTEXT_POLICY_DISABLE; opt-out | Regime-aware context default-on after false-drop bound |
| H-51 | ELAI_HANDOFF_AUTORESUME_DISABLE; opt-out for detection | Detection default-on; auto-continue default-off |
| H-52 | ELAI_REVIEW_ANCHOR_DISABLE; opt-out | Deterministic anchoring default-on after over-fallback bound |
| H-53 | ELAI_CITATION_PREBIND_DISABLE; opt-out | Prebinding default-on after over-refusal bound |
| H-54 | ELAI_CLOSURE_SCAN_DISABLE; opt-out | Closure scan default-on after false-block bound |

## Journal, state, and observability inventory

All names below are explicit in the summary. "Optional" means the summary itself prefers reuse and permits the new row only if reuse is insufficient.

| Plan | Journal/state artifact | Variants or key semantics |
|---|---|---|
| H-1/H-2 | Existing request/cache accounting | Cache write/read fraction and stable-prefix evidence; no named new action type |
| H-3 | StreamingEarlyTerminated | Early-abort row must accompany decoding stop |
| H-5 | batch_lifecycle | Submitted -> Completed; price recomputable |
| H-6 | ContextReduction (reuse) | Mask/reduce while keeping journal re-fetch |
| H-7 | RouteDecisionEvent (reuse/activate) | Calibrated route and real RoutingContext |
| H-8 | OracleCacheHit/Miss (existing consumer rows) | Hit after process restart |
| H-9 | CloudSpend (activate) | Input/output/cached counts and spend |
| H-11/H-13 | CanvasActivityEvent | Live node/edge focus activity |
| H-13 | CanvasOpApplied | Applied/rejected model canvas operation; nine-site lockstep |
| H-15 | CanvasViewEvent | ViewSpawned / UiAction / StateCheckpoint |
| H-17 | TrialRegistered | Monotone effective-trial ledger |
| H-18 | DeflationGateEvaluated | DSR/PBO/N/verdict |
| H-19 | HoldoutBudgetEvent | QuerySpent / Exhausted / Provisioned |
| H-20 | LeakageBlocked | Four-kind leakage discriminator |
| H-21 | LookaheadDifferential | Causal / FutureLeak |
| H-22 | TradingNullTest | Four-kind relation/null result |
| H-23 | RerunHashEquality | Equal / Divergent |
| H-24 | SimRealParity | Three-kind parity result |
| H-25 | ResultManifestVerified | ReDerived / FabricationDetected / MonkeypatchBlocked |
| H-26 | DifferentialDivergence | Three-kind shadow/invariant result |
| H-27 | GateSelfValidation | Four-kind self-validation result plus escaped-mutant counter |
| H-28 | RewardTampering | Three-kind mismatch/delta/clean result |
| H-29 | DriftInterlock | Warning / Drift / Halt |
| H-30 | StalenessEvalMeasured | Computed report, one row per bench run |
| H-31 | UserDecisionSuperseded | EventKind Superseded / Captured / ProposalSurfaced; ids/type/key only |
| H-32 | fact_validation_failed | Invalid/stale/unknown citation excluded before prompt |
| H-33 | MemoryGovernanceEvent | Forgotten / Pinned / Unpinned |
| H-34 | mcp_tool_deferred | One row per deferred live resolve |
| H-34 | mcp_schema_budget_measured | Baseline/injected/delta result |
| H-35/H-49 | SessionRewound | Effective conversation/code undo/rewind |
| H-36 | agent_notification_emitted | Completion or needs-input notification |
| H-37 | plan_decision_event | Approve / Redirect / Reject / AutoAdvance |
| H-38 | StopConfirmationEvent | ConfirmedProceed / RejectedHeld / PassedNoFriction |
| H-39 | AbstentionScored | Reward components/verdict |
| H-40 | iso_budget_measured (reuse) | Adds unique_catch/decorrelation fields |
| H-41 | CheckerGapDetected | claim_class / complexity_taxon / fell_to_mode; Mode-A ratio is a roll-up |
| H-42 | ReasoningBudgetScaled | depth, base/scaled/ceiling, failure class |
| H-43 | FitnessEvaluated | ExecutionPassRate / DualAgreement / FitnessRegression |
| H-44/H-54 | TrapBehaviorDetected | Behavioral trap plus optional ClosureEscapeHatch kind |
| H-45 | PromotionScoreAggregated | Worst-case/variance/lineage aggregation |
| H-46 | CladeSelected | Clade score or ImmediateScoreFallback |
| H-47 | EvalFirewallEvent | Date/template/awareness; holdout spend reuses H-19 |
| H-48 | D-5 telemetry; optional BanditArmSampled | Add new type only if no existing emitter works |
| H-49 | EditHistoryEvent | SnapshotCaptured / Deduped / Redo / Branched / Jumped / TooLargeSkipped |
| H-50 | ContextReduction; optional WindowPolicyApplied | Regime-specific trim/load |
| H-51 | HandoffLifecycleEvent | DigestWritten / ResumeOffered / ResumeConfirmed / TieredLoad |
| H-52 | FindingAnchored | Anchored / FileFallback / Ambiguous / Unanchorable / Relocated |
| H-53 | citation_verified extension; optional CitationAnchorsBound | anchor_id + pre_bound |
| H-54 | TrapBehaviorDetected extension; optional ObligationClosureScanned | ClosureClean / EscapeHatchAnywhere / RegisteredException / ClosureUnavailable |

Schema invariants:

- Every new action type/payload/parser/serializer/lock count must move in the stated Pattern-2 nine-site lockstep.
- Counts shown in individual Track-4 files are per-file deltas, not totals that can all be applied independently; shipping several requires cumulative ordering.
- H-34 is the one Track-4 item that explicitly adds two types; most other Track-4 sessions add one.
- H-40 adds no variant; H-47 reuses H-19 for budget; H-48 prefers existing D-5 telemetry; H-50 prefers ContextReduction; H-53/H-54 prefer composition.
- The executor must read live SCHEMA_VERSION and TOTAL_VARIANTS before editing rather than trusting 207/484, 220/497, or 224/501 snapshots.

## Security, privacy, error, and fallback invariants

- **Filesystem:** H-11 operations are project-root scoped; cross-root and external targets become explicit states/affordances.
- **Canvas:** H-11 resolution errors become ghosts/dropped markers; H-13 dangling IDs and grammar failures surface reasons; H-10 renderer choice must have graceful fallback.
- **Design mode:** H-14 stores opaque element identifiers server-side and does not expose source line/column in DOM.
- **Taint/deixis:** H-15 runs G-1 taint at UI-selection handoff; web content cannot silently become trusted model instruction.
- **Memory privacy:** H-31/H-32 capture no secrets/PII/file contents; H-31 event payload omits values; all user memory is firewalled from auditor/critic/adversary.
- **Memory history:** supersession and forgetting are soft-close operations; no replay-breaking delete.
- **Memory failure:** H-32 Unknown validity excludes; H-33 pin-log failure fails toward ordinary decay.
- **Trading gates:** missing N/null band/evidence is a block; suppressions and open obligations must be registered; locked thresholds cannot be agent-edited.
- **Floating point:** H-21/H-26 use ULP/relative tolerances and explicitly avoid cross-hardware bitwise claims.
- **Rewind/time travel:** an effective change emits evidence; a no-op emits zero; divergence snapshots manual work before restore.
- **Stop control:** headless billable/unsaved stop holds rather than proceeding.
- **Context:** local overflow has context_window_blocked as a disabled-control backstop; unloaded content remains selectively recoverable.
- **Review:** ambiguous/missing snippet anchors fall back to file-level output rather than guessed lines.
- **Citations:** out-of-set anchors are held; ungroundable claims refuse.
- **Proofs:** registered open obligations remain visibly open; boolean-only verifier output yields ClosureUnavailable plus runtime backstop.

## Explicitly deferred, excluded, or unresolved after all authored H sessions

| Source | Item | Classification |
|---|---|---|
| Track 5 | Six Odysseus steals and refinements | DEFER/UNAUTHORED; no H-55+ detail plan present |
| H-5/H-48 | Live RunPod batch/regret or multigeneration figures | DEFER Tier 2; deterministic fixtures ship first |
| H-14 | React Native/Expo design mode | DEFER |
| H-14 | Arbitrary structural diff reliability before recast prove-it | LIMIT; no claim until spike |
| H-20 | Live G-1 taint vs finance-scoped DKG dataflow implementation | AMB; authoring choice unresolved |
| H-23 | Cross-hardware bitwise metric equality | EXCLUDED as physical/honesty limit |
| H-24 | Behavioral numeric sim/live parity | DEFER Tier 2 |
| H-31 | DKG/rerank memory ranking, scope promotion, tidy/delete guard | DEFER to H-32/later; only parts are picked up by H-32 |
| H-32 | ONNX fuzzy citation match and Scope::Global | DEFER |
| H-33 | Standalone MemoryHub and vector-ranked provenance | DEFER |
| H-34 | Schema dedup, cross-server sharing, default-on promotion | DEFER |
| H-35 | Branch/fork, redo, multi-checkpoint diff | Delivered by later H-49, not globally deferred |
| H-35 | Cloud-model VerifyDivergence restore | H-49 snapshot restore supersedes model replay; detail inspection should confirm whether any separate verifier remains |
| H-37 | Multi-round MODIFY, rejection-memory learning, real ComplianceProvider | DEFER |
| H-38 | RunPod billing reconciliation, dashboard modal, auto-checkpoint | DEFER |
| H-47 | Perfect defense against evaluation sandbagging | LIMIT; mitigation only |
| H-52 | LLM relocation retry | DEFER Tier 2 |
| H-52 | External "smart bundling" and "reflection module" | EXCLUDED as vaporware |
| H-53 | Constrained-decode citation grammar and final local-window tuning | DEFER Tier 2 |
| H-54 | Cross-verifier unified obligation-DAG abstraction | DEFER Tier 2 |
| H-54 | Routine formal proof / statement-conformance guarantee | EXCLUDED/LIMIT |

## Corrections and ambiguities requiring deeper inspection

- A-001 [CORRECTION/AMB] H-1 says the RuntimeAdapter cloud path is marker-free; later H-50 says cloud caching is live at all three anthropic.rs sites and explicitly calls H-1's split-brain premise refuted. The later code-read is temporally newer, but originals/live source must determine what H-1 work remains.
- A-002 [AMB] H-10 says the symbol call graph already appears on canvas while also saying dkg_edges carries no call edges. The exact distinction between view-derived call information and stored DKG call edges needs H-10/source inspection.
- A-003 [CORRECTION] H-2's one-hour TTL uses ttl:"1h" without a beta header.
- A-004 [CORRECTION] H-4's tool-definition figure is about 77K -> 8.7K, not 72K.
- A-005 [CORRECTION] H-8 PromptCache is already disk-backed.
- A-006 [CORRECTION] H-14 novelty is the controlled sandbox/proxy/writeback axis, not iframe rendering.
- A-007 [AMB] Track-6 text says future Track 5 would start at H-49, but H-49 through H-54 were subsequently assigned to Tracks 8-11. Later Track-10/11 text says Track 5 was bumped to H-55; treat H-55 as the current reservation pending H-0 detail confirmation.
- A-008 [CORRECTION] H-23 says the older G summary's "RRA absent" statement is stale.
- A-009 [CORRECTION] H-35 says no replay_resume function exists in session_dag.rs despite an earlier brief claim.
- A-010 [CORRECTION] H-38 pod_stop/pod_cost_status are harness MCP tools, not ELAI Rust.
- A-011 [AMB] Track-4 session-local schema deltas conflict if read as simultaneous totals; only ship-order-cumulative application is valid.
- A-012 [AMB] The exact CANVAS document containing §4, §7, and §20.3 is not named in H-SUMMARY_INDEX.
- A-013 [AMB] H-31's detail COMPOSES-WITH reportedly mislabels H-30 as capture/intake; the summary corrects H-30 to a metric and identifies live_cli's FactStore as the actual handle.
- A-014 [LIMIT] Perplexity citation-stage behavior is not officially confirmed; only the shape is borrowed.
- A-015 [AMB] The Track-4 research run wf_c8950d72-45e is named but no exact local artifact path is supplied.
- A-016 [AMB] F-0/G-0 doctrine documents, the stale e45 cross-language drift contract, and the "orchestrate T1 list"/skill doc are referenced by concept but not by exact path.
- A-017 [AMB] H-39 pairs penalty -t/(1-t) and default t=0.75 with the phrase "->2x"; the detail plan must define the intended interpretation rather than silently changing the formula.
- A-018 [AMB] H-31 alternates between dispatch_batch and dispatch_one as the capture activation target.

## Deeper-source manifest

### Canonical summary/doctrine and grounding files

These files exist and were not opened in this summary-only pass:

- docs/thefinalglance/H-plan/H-0_INDEX.md — canonical H doctrine, W/GD/HD/TD/RD/VD/RV/CL rules and likely source of unnamed sections.
- docs/thefinalglance/H-plan/_grounding/H-GROUNDING-MAP.md — Tracks 1-2 symbol grounding.
- docs/thefinalglance/H-plan/_grounding/H-TRUST-GROUNDING-MAP.md — Track 3.
- docs/thefinalglance/H-plan/_grounding/H-UX-MEMORY-GROUNDING-MAP.md — Track 4; present even though the summary preamble does not name it explicitly.
- docs/thefinalglance/H-plan/_grounding/H-REASONING-GROUNDING-MAP.md — Track 6 RG/RS and RD tables.
- docs/thefinalglance/H-plan/_grounding/H-VERIFICATION-GROUNDING-MAP.md — Track 7 DG/PC/VD tables.
- docs/thefinalglance/F-plan/F-0_INDEX.md — inherited W-1..W-10 doctrine.
- docs/thefinalglance/G-plan/G-0_INDEX.md — inherited GD doctrine.

### Numbered H originals and supplement

All 54 numbered originals exist:

- docs/thefinalglance/H-plan/H-1_prompt_caching_production_path.md
- docs/thefinalglance/H-plan/H-2_cache_stable_layout_breakpoints.md
- docs/thefinalglance/H-plan/H-3_output_token_diff_fim_streaming.md
- docs/thefinalglance/H-plan/H-4_tool_search_beta_header.md
- docs/thefinalglance/H-plan/H-5_batch_api_noninteractive_lanes.md
- docs/thefinalglance/H-plan/H-6_observation_masking_memory_readback.md
- docs/thefinalglance/H-plan/H-7_calibrated_routing_architect_editor_verifier.md
- docs/thefinalglance/H-plan/H-8_persist_oracle_answer_cache.md
- docs/thefinalglance/H-plan/H-9_cost_cockpit_dashboard.md
- docs/thefinalglance/H-plan/H-10_canvas_shell_code_graph.md
- docs/thefinalglance/H-plan/H-11_live_layer_file_explorer_fs.md
- docs/thefinalglance/H-plan/H-12_plan_research_modes_chrome.md
- docs/thefinalglance/H-plan/H-13_model_drives_canvas_ops.md
- docs/thefinalglance/H-plan/H-14_design_mode_web_first.md
- docs/thefinalglance/H-plan/H-15_spawn_view_reply_channel.md
- docs/thefinalglance/H-plan/H-16_retire_orbit.md
- docs/thefinalglance/H-plan/H-17_trial_ledger_keystone.md
- docs/thefinalglance/H-plan/H-18_deflated_sharpe_pbo_gate.md
- docs/thefinalglance/H-plan/H-19_reusable_holdout_budget.md
- docs/thefinalglance/H-plan/H-20_leakage_taint_composite.md
- docs/thefinalglance/H-plan/H-21_runtime_lookahead_differential.md
- docs/thefinalglance/H-plan/H-22_trading_metamorphic_null_composite.md
- docs/thefinalglance/H-plan/H-23_same_seed_rerun_hash_equality.md
- docs/thefinalglance/H-plan/H-24_sim_to_real_parity.md
- docs/thefinalglance/H-plan/H-25_result_manifest_rederivation.md
- docs/thefinalglance/H-plan/H-26_shadow_exec_financial_invariant_mr.md
- docs/thefinalglance/H-plan/H-27_gate_self_validation.md
- docs/thefinalglance/H-plan/H-28_reward_tampering_harness_frozen.md
- docs/thefinalglance/H-plan/H-29_drift_state_machine.md
- docs/thefinalglance/H-plan/H-30_staleness_intent_drift_metric.md
- docs/thefinalglance/H-plan/H-31_user_decision_bitemporal_ledger.md
- docs/thefinalglance/H-plan/H-32_code_grounded_validate_before_use.md
- docs/thefinalglance/H-plan/H-33_memory_hub_governance_provenance.md
- docs/thefinalglance/H-plan/H-34_mcp_defer_loading_tool_search.md
- docs/thefinalglance/H-plan/H-35_rewind_over_ssb_journal.md
- docs/thefinalglance/H-plan/H-36_completion_needs_input_notifications.md
- docs/thefinalglance/H-plan/H-37_plan_approval_gate.md
- docs/thefinalglance/H-plan/H-38_cost_confirm_taskstop_podstop.md
- docs/thefinalglance/H-plan/H-39_abstention_aware_promotion_eval.md
- docs/thefinalglance/H-plan/H-40_decorrelation_keyed_verifier_retention.md
- docs/thefinalglance/H-plan/H-41_checker_coverage_gap_instrument.md
- docs/thefinalglance/H-plan/H-42_serial_depth_reasoning_budget.md
- docs/thefinalglance/H-plan/H-43_execution_grounded_dgm_fitness.md
- docs/thefinalglance/H-plan/H-44_behavioral_trap_detection_recall.md
- docs/thefinalglance/H-plan/H-45_worstcase_lineage_promotion_scoring.md
- docs/thefinalglance/H-plan/H-46_clade_metaproductivity_selection.md
- docs/thefinalglance/H-plan/H-47_dgm_eval_firewall_dategate_templateregen.md
- docs/thefinalglance/H-plan/H-48_wire_bandit_on_real_reward.md
- docs/thefinalglance/H-plan/H-49_undo_redo_branching_timetravel.md
- docs/thefinalglance/H-plan/H-50_regime_aware_context_management.md
- docs/thefinalglance/H-plan/H-51_sleeptime_handoff_digest_autodetect.md
- docs/thefinalglance/H-plan/H-52_deterministic_finding_anchoring_cockpit.md
- docs/thefinalglance/H-plan/H-53_citation_prebind_constrained_synthesis.md
- docs/thefinalglance/H-plan/H-54_obligation_closure_escape_hatch_gate.md

Additional H original/supplement:

- docs/thefinalglance/H-plan/H-28-WIRE-1_reward_gate_darwin_admit.md — H-28 production wiring supplement; deep inspection is mandatory because it is non-numbered in the summary inventory.

### Referenced local research, external-code, and protocol artifacts

These paths were resolved by filename/directory inventory only and were not read:

- docs/.source/odysseus-INDEX.md
- docs/.source/odysseus-main/ — Track-4/Track-5 reference implementation corpus.
- .claude/rules/handoff-protocol.md
- workflow/reasoning/ELAI-SELF-VERIFICATION-RESEARCH-DESIGN-2026-06-05.md
- workflow/sessions/oracle/2026-06-06-perplexity-ai-openness.md
- workflow/reasoning/PERPLEXITY-CITATION-SYNTHESIS-STEAL-2026-06-06.md
- workflow/reasoning/ALPHAPROOF-DEPENDENCY-CLOSURE-STEAL-2026-06-06.md
- workflow/papers/alphaproof-nexus-interface.md
- workflow/papers/alphaproof-nexus-HOW-IT-WORKS.md
- workflow/papers/alphaproof-nexus-results-source-interface.md — present companion source map, although not named verbatim in the summary.
- docs/.source/alphaproof-nexus-results/ — cited pinned corpus, summary states commit 0647711.
- docs/.source/open-code-review/
- docs/.source/open-code-review/_SUMMARY.md
- docs/.source/open-code-review/_codebase.map
- docs/.source/open-code-review/internal/diff/resolver.go — likely cited anchoring resolver.
- docs/.source/open-code-review/internal/llm/resolver.go — second same-named candidate; inspect to disambiguate.
- docs/.source/defending-code-reference-harness/
- docs/.source/defending-code-reference-harness/_SUMMARY.md
- docs/.source/defending-code-reference-harness/_codebase.map
- .elai_cc/wire-dark-allowlist.toml
- .elai_cc/crates/contracts/tests/e45_cross_language_drift_contract.rs
- .claude/skills/orchestrate/SKILL.md — likely owner of the H-38 "orchestrate T1 list"; confirm in deep inspection.

Named paper-interface files present and directly relevant:

- workflow/papers/hallucination-stations-interface.md
- workflow/papers/expressive-power-transformers-cot-2310.07923-interface.md
- workflow/papers/hallucination-inevitable-2401.11817-interface.md
- workflow/papers/hallucination-negligible-2502.12187-interface.md
- workflow/papers/compositional-reasoning-transformers-rnn-cot-2503.01544-interface.md
- workflow/papers/weaver-2506.18203-interface.md
- workflow/papers/why-llms-hallucinate-2509.04664-interface.md
- workflow/papers/pot-2211.12588-interface.md
- workflow/papers/codeact-2402.01030-interface.md
- workflow/papers/livecodebench-2403.07974-interface.md
- workflow/papers/prmbench-2501.03124-interface.md
- workflow/papers/rm-overoptimization-2210.10760-interface.md
- workflow/papers/zep-graphiti-2501.13956-interface.md
- workflow/papers/beliefshift-2603.23848-interface.md
- workflow/papers/stale-2605.06527-interface.md

### Referenced external citations not resolved to an exact local source path

The summary cites the following identifiers. Later deep agents should inspect the applicable grounding map/detail plan and verify title/repository identity before using any number:

- Track 4/citation-memory: 2504.19413 (Mem0), 2505.07509 (HALO), 2510.09905 (Personalization-Trap), 2512.16962 (MemoryGraft), 2605.06527 (STALE), 2603.23848 (BeliefShift), plus GitHub Copilot agentic-memory 2026 and Graphiti/Zep claims.
- Track 6/formal reasoning: 2507.07505, 2402.12875, 2511.07378, 2502.02393, 2402.08115, in addition to the locally named interfaces above; YouTube AIYQp1n51ZI is also cited.
- Track 7/self-verification: 1506.02629, 2207.10397, 2209.13085, 2305.10718, 2310.02743, 2312.09244, 2405.00253, 2410.05229, 2501.01054, 2505.23836, 2507.00075, 2508.00410, 2509.04575, 2510.20270, 2510.20487, 2510.21614, 2601.20103, 2604.15149, 2605.02964, alongside locally named PRMBench/LiveCodeBench/overoptimization interfaces.
- Track 11: 2605.22763 AlphaProof Nexus (paper) and the Perplexity oracle report.
- Non-arXiv/named sources: Harvey-Liu threshold, ONC clustering, Thresholdout/Ladder, Freqtrade look-ahead analysis, QuantLib, Daikon, Audibert COLT 2010 Successive Rejects, recast 0.23.11, @onlook/parser ids.ts, Cytoscape WebGL 3.34, Sigma 3.0.3, AG-UI/MCP-Apps, Cursor/Cline/Roo/Aider/undo-tree, Letta/A-MEM/Mem0, Claude Code/Pragmatic Engineer 2026, Anthropic context engineering, vLLM APC/HF-transformers, databow, and alibaba/open-code-review.

### Missing/deferred deep-source candidates

- No authored Track-5 files (current expected start H-55) exist in H-plan.
- No local path is supplied for the Track-4 research run wf_c8950d72-45e.
- The exact document referred to as CANVAS §4/§7/§20.3 is unnamed.
- The summary says Track-6 interface maps live in workflow/papers but does not map every cited identifier to a filename.
- Databow is mentioned as a fetched blog/source but no local path is named in the summary.
- The exact source file for the H-38 pod_stop skill-doc change is unnamed; .claude/skills/orchestrate/SKILL.md is only a resolved candidate.

## Final summary-only coverage statement

- 54/54 H session headings represented.
- 904 globally/session-prefixed atomic inventory statements before the tables/manifests; all are source-classified and none are inferred requirements.
- All explicit cross-track sequences, named flags, journal event families, acceptance/broken-arm behavior, security/privacy constraints, status corrections, ambiguity notes, and deferred work found in H-SUMMARY_INDEX are represented.
- All 54 numbered H detail sources, the H-28 wiring supplement, H-0, five grounding maps, and named non-numbered research/protocol/source artifacts are routed for deeper inspection.
- This document is a summary-index extraction, not a claim that the numbered originals contain no additional minor features.
